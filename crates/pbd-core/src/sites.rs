//! Where the planet's settlements stand (`openspec/changes/city-sites`): a
//! list of sites that follows from the seed, the generator and
//! `assets/config/sites.ron`, and from nothing else.
//!
//! The rules, in the design's order:
//! 1. **Candidates** are the centres of the level-7 cells, 163,842 of them
//!    about 45 m apart. A candidate's id is its cell's index, so the id is
//!    stable for the life of a save (decision 1).
//! 2. **The screen** reads each candidate's biome and keeps the kind that
//!    belongs to it, if its ground passes that kind's coarse test: seven
//!    samples for the flat kinds, the water for a harbour, the rise for a
//!    cliff village, the rock for a cave town (decisions 2, 3 and 5). A
//!    candidate near a pentagon is dropped.
//! 3. **The score** is flatness, a river nearby and a seeded jitter.
//! 4. **The full check** samples the footprint at the terrain's own cell
//!    spacing (the spec's "flat, dry ground").
//! 5. **The greedy pass** keeps candidates by score, then id, while the kind's
//!    count is not reached and the spacing is clear. The owner's pins come
//!    first, and the small town near the spawn after them (survey C2).
//! 6. **Names** are drawn from each people's syllables, seeded by the id
//!    (decision 4), and the capital is marked (survey C3).
//!
//! One cell holds one kind: the first of its biome's kinds whose screen and
//! full check both pass. In the fields a cell is a walled town when its
//! ground holds a town, and a village only when it does not; in the
//! mountains a cave town comes before a cliff village. So a change of counts
//! never turns one site into another kind, and the full check, the costly
//! part, runs on the screening threads.
//!
//! The screen runs on as many threads as it is given. Each thread takes a
//! fixed run of cells and the runs are joined in order, and everything after
//! the screen is sequential over a total order, so the list is the same on
//! any thread count.

use crate::fauna::WaterClass;
use crate::geo::{self, LatLon};
use crate::planet_gen::{Biome, TerrainConfig, biome_at, river_channel, surface_altitude};
use crate::records::{Record, Records};
use crate::topology;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// The terrain's finest cells are this far apart (CLAUDE.md, the Tenebris
/// gold standard); the full check samples a footprint at this spacing.
pub const FINE_SPACING_M: f32 = 2.833;

/// The settlements `tenebris-towns` designs, each with the one biome it takes
/// its materials from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Deserialize, Serialize)]
pub enum SiteKind {
    /// A walled town in the fields; one of them is the capital.
    Walled,
    Village,
    Desert,
    Tundra,
    Jungle,
    Swamp,
    /// Where the beach meets the fields, with water shelving off its quay.
    Harbour,
    /// Terraces climbing a mountainside (survey T12).
    Cliff,
    /// A town in a chamber under the rock (survey T12).
    Cave,
}

impl SiteKind {
    pub const ALL: [SiteKind; 9] = [
        SiteKind::Walled,
        SiteKind::Village,
        SiteKind::Desert,
        SiteKind::Tundra,
        SiteKind::Jungle,
        SiteKind::Swamp,
        SiteKind::Harbour,
        SiteKind::Cliff,
        SiteKind::Cave,
    ];

    /// The biome a site of this kind stands in.
    pub fn biome(self) -> Biome {
        match self {
            SiteKind::Walled | SiteKind::Village => Biome::Fields,
            SiteKind::Desert => Biome::Desert,
            SiteKind::Tundra => Biome::Tundra,
            SiteKind::Jungle => Biome::Jungle,
            SiteKind::Swamp => Biome::Swamp,
            SiteKind::Harbour => Biome::Beach,
            SiteKind::Cliff | SiteKind::Cave => Biome::Mountains,
        }
    }

    /// The kinds a biome's cell is tried as, in order: the first whose
    /// screen passes is the cell's one kind.
    pub fn for_biome(biome: Biome) -> &'static [SiteKind] {
        match biome {
            Biome::Fields => &[SiteKind::Walled, SiteKind::Village],
            Biome::Desert => &[SiteKind::Desert],
            Biome::Tundra => &[SiteKind::Tundra],
            Biome::Jungle => &[SiteKind::Jungle],
            Biome::Swamp => &[SiteKind::Swamp],
            Biome::Beach => &[SiteKind::Harbour],
            Biome::Mountains => &[SiteKind::Cave, SiteKind::Cliff],
            Biome::Ocean => &[],
        }
    }

    /// A lower-case name for readouts: "walled town", "cave town".
    pub fn name(self) -> &'static str {
        match self {
            SiteKind::Walled => "walled town",
            SiteKind::Village => "village",
            SiteKind::Desert => "desert town",
            SiteKind::Tundra => "tundra camp",
            SiteKind::Jungle => "jungle village",
            SiteKind::Swamp => "swamp village",
            SiteKind::Harbour => "harbour",
            SiteKind::Cliff => "cliff village",
            SiteKind::Cave => "cave town",
        }
    }
}

/// One kind's footprint and how many of it the planet wants.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct KindRule {
    pub kind: SiteKind,
    /// The footprint's radius, m.
    pub radius_m: f32,
    /// The largest surface range under the footprint, m. The flat kinds'
    /// test; a harbour, a cliff village and a cave town have their own.
    pub flatness_m: f32,
    /// How far this kind keeps from its own kind, m, between the footprints'
    /// edges. From another kind it keeps half the smaller of the two.
    pub spacing_m: f32,
    /// The target: a seed with too little of the biome falls short.
    pub count: u32,
    /// Whose names it takes: one of `peoples`.
    pub people: String,
}

/// Decision 3: a harbour is found from the water.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HarbourRule {
    /// The highest dry ground under the footprint, m above the sea.
    pub shore_max_m: f32,
    /// How far from the anchor shelf water must be found, m.
    pub shelf_reach_m: f32,
}

/// Decision 5: a cliff village climbs.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CliffRule {
    /// The rise across the footprint, uphill edge over downhill edge, m.
    pub rise_min_m: f32,
    pub rise_max_m: f32,
    /// Samples along the fall line are this far apart, m, and no two
    /// neighbours may differ by more than `step_rise_max_m`.
    pub step_m: f32,
    pub step_rise_max_m: f32,
}

/// Decision 5: a cave town has rock over its chamber.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CaveRule {
    /// The rock over the chamber, m above the entrance's level.
    pub rock_m: f32,
    /// The chamber, m: along the fall line, and across it.
    pub chamber_length_m: f32,
    pub chamber_width_m: f32,
    /// How far below the chamber's downhill end the ground must fall to the
    /// entrance's level, m.
    pub mouth_reach_m: f32,
}

/// One people's syllables (decision 4).
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct People {
    pub name: String,
    pub onsets: Vec<String>,
    pub middles: Vec<String>,
    pub endings: Vec<String>,
    /// A harbour's endings, drawn from in place of `endings` ("haven",
    /// "port"). Empty: a harbour draws from `endings`.
    #[serde(default)]
    pub harbour_endings: Vec<String>,
    /// A village on a river's endings, drawn from in place of `endings`
    /// ("ford", "bridge"). Empty: it draws from `endings`.
    #[serde(default)]
    pub river_endings: Vec<String>,
}

/// A site the owner placed by hand (decision 5).
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Pin {
    pub kind: SiteKind,
    /// Degrees, as the map reads them.
    pub lat: f32,
    pub lon: f32,
    #[serde(default)]
    pub name: Option<String>,
    /// This pin is the capital (survey C3). Only a walled town may be.
    #[serde(default)]
    pub capital: bool,
}

/// `assets/config/sites.ron`: the rules, the names and the owner's overrides.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SitesConfig {
    /// The sites version a world records with its list.
    pub version: u32,
    /// The candidates' topology level.
    pub level: u32,
    /// A footprint keeps this far from a pentagon, m: two cells of the
    /// candidates' level (`tenebris-towns` design section 6).
    pub pentagon_margin_m: f32,
    /// The small town near the spawn is within this, m (survey C2).
    pub home_within_m: f32,
    /// The score's seeded jitter, and the bonus for a river within
    /// `river_reach_m` of a town's or village's footprint.
    pub jitter: f32,
    pub river_bonus: f32,
    pub river_reach_m: f32,
    pub kinds: Vec<KindRule>,
    pub harbour: HarbourRule,
    pub cliff: CliffRule,
    pub cave: CaveRule,
    pub peoples: Vec<People>,
    #[serde(default)]
    pub pins: Vec<Pin>,
    /// Generated sites struck out, by id.
    #[serde(default)]
    pub strikes: Vec<u32>,
}

impl SitesConfig {
    /// Refuse a config the rules cannot run on, naming the field.
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=8).contains(&self.level) {
            return Err(format!("level {} is outside 1..=8", self.level));
        }
        let positive = |name: &str, v: f32| {
            if v.is_finite() && v > 0.0 {
                Ok(())
            } else {
                Err(format!("{name} must be finite and above zero, not {v}"))
            }
        };
        positive("home_within_m", self.home_within_m)?;
        positive("river_reach_m", self.river_reach_m)?;
        positive("harbour.shelf_reach_m", self.harbour.shelf_reach_m)?;
        positive("harbour.shore_max_m", self.harbour.shore_max_m)?;
        positive("cliff.step_m", self.cliff.step_m)?;
        positive("cliff.step_rise_max_m", self.cliff.step_rise_max_m)?;
        positive("cave.rock_m", self.cave.rock_m)?;
        positive("cave.chamber_length_m", self.cave.chamber_length_m)?;
        positive("cave.chamber_width_m", self.cave.chamber_width_m)?;
        positive("cave.mouth_reach_m", self.cave.mouth_reach_m)?;
        for (name, v) in [
            ("pentagon_margin_m", self.pentagon_margin_m),
            ("jitter", self.jitter),
            ("river_bonus", self.river_bonus),
        ] {
            if !(v.is_finite() && v >= 0.0) {
                return Err(format!("{name} must be finite and not below zero, not {v}"));
            }
        }
        if !(self.cliff.rise_min_m >= 0.0 && self.cliff.rise_max_m > self.cliff.rise_min_m) {
            return Err("cliff.rise_max_m must exceed cliff.rise_min_m".into());
        }
        let mut seen = BTreeSet::new();
        for (i, rule) in self.kinds.iter().enumerate() {
            let at = format!("kinds[{i}] ({:?})", rule.kind);
            if !seen.insert(rule.kind) {
                return Err(format!("{at}: listed twice"));
            }
            positive(&format!("{at}.radius_m"), rule.radius_m)?;
            positive(&format!("{at}.flatness_m"), rule.flatness_m)?;
            positive(&format!("{at}.spacing_m"), rule.spacing_m)?;
            if !self.peoples.iter().any(|p| p.name == rule.people) {
                return Err(format!("{at}.people: no people named {:?}", rule.people));
            }
        }
        if let Some(kind) = SiteKind::ALL.iter().find(|k| !seen.contains(k)) {
            return Err(format!("kinds: {kind:?} is missing"));
        }
        for (i, people) in self.peoples.iter().enumerate() {
            if people.onsets.is_empty() || people.middles.is_empty() || people.endings.is_empty() {
                return Err(format!(
                    "peoples[{i}] ({}): onsets, middles and endings each need an entry",
                    people.name
                ));
            }
        }
        for (i, pin) in self.pins.iter().enumerate() {
            if !(pin.lat.is_finite() && pin.lon.is_finite() && pin.lat.abs() <= 90.0) {
                return Err(format!(
                    "pins[{i}]: lat and lon must be degrees on the planet"
                ));
            }
            if pin.capital && pin.kind != SiteKind::Walled {
                return Err(format!("pins[{i}]: only a walled town can be the capital"));
            }
        }
        if self.pins.iter().filter(|p| p.capital).count() > 1 {
            return Err("pins: more than one capital".into());
        }
        Ok(())
    }

    /// Refuse a pin nothing can be built on: at sea, or on a pentagon
    /// (decision 5), naming the pin.
    pub fn validate_pins(&self, terrain: &TerrainConfig) -> Result<(), String> {
        let pentagons = pentagons();
        for (i, pin) in self.pins.iter().enumerate() {
            let d = pin.direction();
            let label = pin.name.clone().unwrap_or_else(|| pin.kind.name().into());
            if surface_altitude(terrain, d) < terrain.sea_level_m {
                return Err(format!("pins[{i}] ({label}) stands at sea"));
            }
            let reach = self.rule(pin.kind).radius_m + self.pentagon_margin_m;
            if near_pentagon(&pentagons, d, reach, terrain.radius_m) {
                return Err(format!("pins[{i}] ({label}) stands on a pentagon"));
            }
        }
        Ok(())
    }

    /// The rule for a kind. Every kind has one once validated.
    pub fn rule(&self, kind: SiteKind) -> &KindRule {
        self.kinds
            .iter()
            .find(|r| r.kind == kind)
            .expect("a validated config lists every kind")
    }

    fn people(&self, kind: SiteKind) -> &People {
        let name = &self.rule(kind).people;
        self.peoples
            .iter()
            .find(|p| &p.name == name)
            .expect("a validated config names its peoples")
    }
}

impl Pin {
    pub fn direction(&self) -> Vec3 {
        geo::direction(LatLon {
            lat: self.lat.to_radians(),
            lon: self.lon.to_radians(),
        })
    }
}

/// One place the screen kept, before the greedy pass.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Candidate {
    /// The level-7 cell's index.
    pub id: u32,
    pub kind: SiteKind,
    pub direction: Vec3,
    pub score: f32,
    /// A river within reach of the footprint.
    pub river: bool,
}

/// One settlement's place.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Site {
    /// The level-7 cell its anchor stands in.
    pub id: u32,
    pub kind: SiteKind,
    /// The anchor: the cell's centre, or the pin's place.
    #[serde(skip)]
    pub direction: Vec3,
    pub name: String,
    pub capital: bool,
    /// The small town near the spawn (survey C2).
    pub home: bool,
    pub pinned: bool,
    pub river: bool,
}

/// The whole planet's list, with each kind's shortfall against its target.
#[derive(Clone, Debug, PartialEq)]
pub struct SiteList {
    pub sites: Vec<Site>,
    pub shortfall: Vec<(SiteKind, u32)>,
}

/// The level-7 cells: their centres and neighbours.
pub struct Cells {
    pub directions: Vec<Vec3>,
    pub neighbours: Vec<Vec<u32>>,
}

impl Cells {
    pub fn new(level: u32) -> Self {
        let cells = topology::dual_sphere(level);
        Cells {
            directions: cells.iter().map(|c| c.direction).collect(),
            neighbours: cells
                .iter()
                .map(|c| c.neighbors.iter().map(|&n| n as u32).collect())
                .collect(),
        }
    }

    /// The cell whose centre is nearest a direction, by walking the
    /// neighbours downhill in distance from a start.
    pub fn nearest(&self, direction: Vec3) -> u32 {
        let mut at = 0u32;
        // Start from the nearest of a coarse sample, then walk.
        let step = (self.directions.len() / 512).max(1);
        let mut best = f32::MIN;
        for i in (0..self.directions.len()).step_by(step) {
            let dot = self.directions[i].dot(direction);
            if dot > best {
                best = dot;
                at = i as u32;
            }
        }
        loop {
            let here = self.directions[at as usize].dot(direction);
            let next = self.neighbours[at as usize]
                .iter()
                .copied()
                .max_by(|&a, &b| {
                    let da = self.directions[a as usize].dot(direction);
                    let db = self.directions[b as usize].dot(direction);
                    da.total_cmp(&db).then(b.cmp(&a))
                })
                .expect("every cell has neighbours");
            if self.directions[next as usize].dot(direction) > here {
                at = next;
            } else {
                return at;
            }
        }
    }
}

/// The twelve pentagons: the icosahedron's corners, at every level.
pub fn pentagons() -> Vec<Vec3> {
    topology::icosahedron().0
}

fn near_pentagon(pentagons: &[Vec3], d: Vec3, reach_m: f32, radius_m: f32) -> bool {
    pentagons.iter().any(|p| arc_m(*p, d, radius_m) < reach_m)
}

/// The great-circle distance between two directions, m.
pub fn arc_m(a: Vec3, b: Vec3, radius_m: f32) -> f32 {
    // atan2 of the cross and dot keeps its precision at short range, where
    // acos of a dot near one does not.
    a.cross(b).length().atan2(a.dot(b)) * radius_m
}

/// The direction `east_m` east and `north_m` north of `d`, along the ground.
pub fn offset(d: Vec3, east_m: f32, north_m: f32, radius_m: f32) -> Vec3 {
    let (north, east) = geo::north_east(d);
    let along = east * east_m + north * north_m;
    let length = along.length();
    if length < 1e-6 {
        return d;
    }
    let angle = length / radius_m;
    (d * angle.cos() + along / length * angle.sin()).normalize()
}

/// A seeded hash of integers to `[0, 1)`: splitmix64 folded over the parts.
fn hash01(parts: &[u64]) -> f32 {
    let mut h = 0x9E37_79B9_7F4A_7C15u64;
    for &p in parts {
        h ^= p
            .wrapping_add(0x9E37_79B9_7F4A_7C15)
            .wrapping_add(h << 6)
            .wrapping_add(h >> 2);
        h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        h ^= h >> 31;
        h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
        h ^= h >> 29;
    }
    (h >> 40) as f32 / (1u64 << 24) as f32
}

/// The ground's altitude, m, and whether it is dry.
fn ground(terrain: &TerrainConfig, d: Vec3) -> f32 {
    surface_altitude(terrain, d)
}

/// The direction the ground falls in at `d`, along the surface, from a
/// central difference `step_m` wide; `None` on level ground.
fn downhill(terrain: &TerrainConfig, d: Vec3, step_m: f32) -> Option<Vec3> {
    let r = terrain.radius_m;
    let (north, east) = geo::north_east(d);
    let de =
        ground(terrain, offset(d, step_m, 0.0, r)) - ground(terrain, offset(d, -step_m, 0.0, r));
    let dn =
        ground(terrain, offset(d, 0.0, step_m, r)) - ground(terrain, offset(d, 0.0, -step_m, r));
    let fall = -(east * de + north * dn);
    (fall.length() > 1e-4).then(|| fall.normalize())
}

/// The point `along_m` down the fall line and `across_m` to its right.
fn on_line(d: Vec3, fall: Vec3, along_m: f32, across_m: f32, radius_m: f32) -> Vec3 {
    let (north, east) = geo::north_east(d);
    let right = d.cross(fall).normalize();
    let v = fall * along_m + right * across_m;
    offset(d, v.dot(east), v.dot(north), radius_m)
}

/// Decision 5, a cliff village: along the fall line through the footprint,
/// the ground rises between the rule's bounds with no sheer step, and every
/// sample on it is dry. `step_m` is the spacing the rule holds; the screen
/// and the full check read the same line.
fn cliff_passes(sites: &SitesConfig, terrain: &TerrainConfig, d: Vec3, radius_m: f32) -> bool {
    let rule = sites.cliff;
    let Some(fall) = downhill(terrain, d, rule.step_m) else {
        return false;
    };
    let steps = (2.0 * radius_m / rule.step_m).round().max(1.0) as i32;
    let mut profile = Vec::with_capacity(steps as usize + 1);
    for k in 0..=steps {
        let along = -radius_m + k as f32 * 2.0 * radius_m / steps as f32;
        profile.push(ground(
            terrain,
            on_line(d, fall, along, 0.0, terrain.radius_m),
        ));
    }
    let sea = terrain.sea_level_m;
    if profile.iter().any(|&h| h < sea) {
        return false;
    }
    // `along` runs downhill, so the first sample is the uphill edge.
    let rise = profile[0] - profile[profile.len() - 1];
    let steep = profile
        .windows(2)
        .any(|w| (w[0] - w[1]).abs() > rule.step_rise_max_m);
    (rule.rise_min_m..=rule.rise_max_m).contains(&rise) && !steep
}

/// Decision 5, a cave town: the chamber, laid down the fall line from the
/// anchor, has the rule's rock over it above the entrance's level, and the
/// ground below its downhill end falls to that level, dry, within reach.
/// `step_m` is the sampling across the chamber: coarse for the screen, fine
/// for the full check.
fn cave_passes(sites: &SitesConfig, terrain: &TerrainConfig, d: Vec3, step_m: f32) -> bool {
    let rule = sites.cave;
    let Some(fall) = downhill(terrain, d, 10.0) else {
        return false;
    };
    let r = terrain.radius_m;
    let half_l = rule.chamber_length_m / 2.0;
    let half_w = rule.chamber_width_m / 2.0;
    // The entrance: the lowest ground within reach below the downhill end.
    let mut entrance = f32::MAX;
    let mouth_steps = (rule.mouth_reach_m / step_m).ceil().max(1.0) as i32;
    for k in 1..=mouth_steps {
        let along = half_l + k as f32 * rule.mouth_reach_m / mouth_steps as f32;
        entrance = entrance.min(ground(terrain, on_line(d, fall, along, 0.0, r)));
    }
    if entrance < terrain.sea_level_m {
        return false;
    }
    let (nl, nw) = (
        (rule.chamber_length_m / step_m).ceil().max(1.0) as i32,
        (rule.chamber_width_m / step_m).ceil().max(1.0) as i32,
    );
    for i in 0..=nl {
        for j in 0..=nw {
            let along = -half_l + i as f32 * rule.chamber_length_m / nl as f32;
            let across = -half_w + j as f32 * rule.chamber_width_m / nw as f32;
            if ground(terrain, on_line(d, fall, along, across, r)) - entrance < rule.rock_m {
                return false;
            }
        }
    }
    true
}

/// The flat kinds' ground, over any set of samples: dry, within the kind's
/// flatness. A swamp village stands on stilts over its water, so its
/// samples may be under the sea as deep as the shallows, and the flatness is
/// its dry ground's; its anchor is dry.
fn flat_ground(
    sites: &SitesConfig,
    terrain: &TerrainConfig,
    kind: SiteKind,
    alts: &[f32],
) -> Option<f32> {
    let rule = sites.rule(kind);
    let sea = terrain.sea_level_m;
    let (lo, hi) = if kind == SiteKind::Swamp {
        let shallows = crate::fauna::WaterLimits::default().shallows_max_m;
        if alts[0] < sea || alts.iter().any(|&h| sea - h > shallows) {
            return None;
        }
        let dry = alts.iter().copied().filter(|&h| h >= sea);
        let (lo, hi) = dry.fold((f32::MAX, f32::MIN), |(lo, hi), h| (lo.min(h), hi.max(h)));
        (lo, hi)
    } else {
        let (lo, hi) = alts
            .iter()
            .fold((f32::MAX, f32::MIN), |(lo, hi), &h| (lo.min(h), hi.max(h)));
        if lo < sea {
            return None;
        }
        (lo, hi)
    };
    let range = hi - lo;
    (range <= rule.flatness_m).then_some(range)
}

/// How a harbour's water reads: shelf water within reach of the anchor.
fn shelf_within(terrain: &TerrainConfig, d: Vec3, reach_m: f32) -> bool {
    let limits = crate::fauna::WaterLimits::default();
    (0..8).any(|k| {
        let bearing = k as f32 * std::f32::consts::FRAC_PI_4;
        let p = offset(
            d,
            reach_m * bearing.sin(),
            reach_m * bearing.cos(),
            terrain.radius_m,
        );
        let depth = terrain.sea_level_m - ground(terrain, p);
        crate::fauna::water_class(terrain, p, depth, &limits) == Some(WaterClass::Shelf)
    })
}

/// A river within reach of a footprint: the carve fires somewhere on a ring
/// just outside it.
fn river_near(terrain: &TerrainConfig, d: Vec3, reach_m: f32) -> bool {
    (0..8).any(|k| {
        let bearing = k as f32 * std::f32::consts::FRAC_PI_4;
        let p = offset(
            d,
            reach_m * bearing.sin(),
            reach_m * bearing.cos(),
            terrain.radius_m,
        );
        river_channel(terrain, p) > terrain.river_threshold
    })
}

/// The screen, full check and score for one cell: its one kind, if any
/// passes.
fn screen(
    sites: &SitesConfig,
    terrain: &TerrainConfig,
    pentagons: &[Vec3],
    id: u32,
    d: Vec3,
) -> Option<Candidate> {
    let r = terrain.radius_m;
    let sea = terrain.sea_level_m;
    let centre = ground(terrain, d);
    let biome = biome_at(terrain, d, centre);
    for &kind in SiteKind::for_biome(biome) {
        let rule = sites.rule(kind);
        if near_pentagon(pentagons, d, rule.radius_m + sites.pentagon_margin_m, r) {
            return None;
        }
        let ring: Vec<Vec3> = (0..6)
            .map(|k| {
                let bearing = k as f32 * std::f32::consts::FRAC_PI_3;
                offset(
                    d,
                    rule.radius_m * bearing.sin(),
                    rule.radius_m * bearing.cos(),
                    r,
                )
            })
            .collect();
        let alts: Vec<f32> = std::iter::once(centre)
            .chain(ring.iter().map(|&p| ground(terrain, p)))
            .collect();
        let lo = alts.iter().copied().fold(f32::MAX, f32::min);
        let hi = alts.iter().copied().fold(f32::MIN, f32::max);
        let range = hi - lo;
        let (passes, flatness) = match kind {
            SiteKind::Harbour => {
                let fields = ring
                    .iter()
                    .zip(&alts[1..])
                    .any(|(&p, &h)| biome_at(terrain, p, h) == Biome::Fields);
                let shore = hi - sea <= sites.harbour.shore_max_m;
                let ok = fields && shore && shelf_within(terrain, d, sites.harbour.shelf_reach_m);
                (ok, 1.0 - (hi - sea).max(0.0) / sites.harbour.shore_max_m)
            }
            SiteKind::Cliff => (
                lo >= sea && cliff_passes(sites, terrain, d, rule.radius_m),
                1.0 - range / sites.cliff.rise_max_m,
            ),
            SiteKind::Cave => (
                lo >= sea && cave_passes(sites, terrain, d, 15.0),
                (range / (4.0 * sites.cave.rock_m)).min(1.0),
            ),
            _ => match flat_ground(sites, terrain, kind, &alts) {
                Some(range) => (true, 1.0 - range / rule.flatness_m),
                None => (false, 0.0),
            },
        };
        if !passes || !full_check(sites, terrain, kind, d) {
            continue;
        }
        let river = matches!(kind, SiteKind::Walled | SiteKind::Village)
            && river_near(terrain, d, rule.radius_m + sites.river_reach_m);
        let score = flatness
            + if river { sites.river_bonus } else { 0.0 }
            + sites.jitter * hash01(&[terrain.seed, u64::from(id), 1]);
        return Some(Candidate {
            id,
            kind,
            direction: d,
            score,
            river,
        });
    }
    None
}

/// Stages 1 to 4: every cell screened, checked in full and scored, on
/// `threads` threads, in cell order.
pub fn candidates(
    sites: &SitesConfig,
    terrain: &TerrainConfig,
    cells: &Cells,
    threads: usize,
) -> Vec<Candidate> {
    let pentagons = pentagons();
    let n = cells.directions.len();
    let threads = threads.clamp(1, 64);
    let chunk = n.div_ceil(threads);
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                let (start, end) = ((t * chunk).min(n), ((t + 1) * chunk).min(n));
                let pentagons = &pentagons;
                scope.spawn(move || {
                    (start..end)
                        .filter_map(|i| {
                            screen(sites, terrain, pentagons, i as u32, cells.directions[i])
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().expect("a screening thread"))
            .collect()
    })
}

/// The footprint's samples at the terrain's cell spacing: a hexagonal
/// lattice over the disc, as the terrain's own cells lie.
pub fn footprint(d: Vec3, radius_m: f32, planet_radius_m: f32) -> Vec<Vec3> {
    let s = FINE_SPACING_M;
    let row = s * 3f32.sqrt() / 2.0;
    let rows = (radius_m / row).floor() as i32;
    let mut out = Vec::new();
    for j in -rows..=rows {
        let y = j as f32 * row;
        let shift = if j.rem_euclid(2) == 1 { s / 2.0 } else { 0.0 };
        let cols = ((radius_m + s) / s).ceil() as i32;
        for i in -cols..=cols {
            let x = i as f32 * s + shift;
            if x * x + y * y <= radius_m * radius_m {
                out.push(offset(d, x, y, planet_radius_m));
            }
        }
    }
    out
}

/// The spec's "flat, dry ground", sampled at the terrain's cell spacing: the
/// check a kept site passes. A harbour's water rows are allowed, and it must
/// hold shallows; the mountain kinds hold their own rule at the fine step.
pub fn full_check(sites: &SitesConfig, terrain: &TerrainConfig, kind: SiteKind, d: Vec3) -> bool {
    let rule = sites.rule(kind);
    let sea = terrain.sea_level_m;
    let alts: Vec<f32> = footprint(d, rule.radius_m, terrain.radius_m)
        .into_iter()
        .map(|p| ground(terrain, p))
        .collect();
    let lo = alts.iter().copied().fold(f32::MAX, f32::min);
    let hi = alts.iter().copied().fold(f32::MIN, f32::max);
    match kind {
        SiteKind::Harbour => {
            let limits = crate::fauna::WaterLimits::default();
            let shallows = alts
                .iter()
                .any(|&h| h < sea && sea - h <= limits.shallows_max_m);
            hi - sea <= sites.harbour.shore_max_m && shallows
        }
        SiteKind::Cliff => lo >= sea && cliff_passes(sites, terrain, d, rule.radius_m),
        SiteKind::Cave => lo >= sea && cave_passes(sites, terrain, d, 5.0),
        _ => {
            // The anchor first: the swamp's rule reads it as the dry centre.
            let mut ordered = Vec::with_capacity(alts.len() + 1);
            ordered.push(ground(terrain, d));
            ordered.extend_from_slice(&alts);
            flat_ground(sites, terrain, kind, &ordered).is_some()
        }
    }
}

/// Whether two sites stand clear of each other: a kind's own spacing from
/// its own kind, and half the smaller spacing from any other, between the
/// footprints' edges.
pub fn clear_of(
    sites: &SitesConfig,
    radius_m: f32,
    a: (SiteKind, Vec3),
    b: (SiteKind, Vec3),
) -> bool {
    let (ra, rb) = (sites.rule(a.0), sites.rule(b.0));
    let gap = arc_m(a.1, b.1, radius_m) - ra.radius_m - rb.radius_m;
    let need = if a.0 == b.0 {
        ra.spacing_m
    } else {
        0.5 * ra.spacing_m.min(rb.spacing_m)
    };
    gap >= need
}

/// The land masses: the level-7 cells whose centre is dry, joined through
/// their neighbours. `None` at sea.
pub fn land_masses(terrain: &TerrainConfig, cells: &Cells) -> Vec<Option<u32>> {
    let n = cells.directions.len();
    let dry: Vec<bool> = cells
        .directions
        .iter()
        .map(|&d| ground(terrain, d) >= terrain.sea_level_m)
        .collect();
    let mut label = vec![None; n];
    let mut next = 0u32;
    let mut stack = Vec::new();
    for start in 0..n {
        if !dry[start] || label[start].is_some() {
            continue;
        }
        label[start] = Some(next);
        stack.push(start as u32);
        while let Some(i) = stack.pop() {
            for &j in &cells.neighbours[i as usize] {
                if dry[j as usize] && label[j as usize].is_none() {
                    label[j as usize] = Some(next);
                    stack.push(j);
                }
            }
        }
        next += 1;
    }
    label
}

/// Stages 5 and 6: the owner's pins, the small town near the spawn, then the
/// greedy pass over the candidates; then the capital and the names.
pub fn select(
    sites: &SitesConfig,
    terrain: &TerrainConfig,
    cells: &Cells,
    candidates: &[Candidate],
    spawn: Vec3,
) -> SiteList {
    let r = terrain.radius_m;
    let mut kept: Vec<Site> = Vec::new();
    let mut counts: BTreeMap<SiteKind, u32> = BTreeMap::new();
    let struck: BTreeSet<u32> = sites.strikes.iter().copied().collect();
    let clear = |kept: &[Site], kind: SiteKind, d: Vec3| {
        kept.iter()
            .all(|s| clear_of(sites, r, (s.kind, s.direction), (kind, d)))
    };

    // The owner's pins first, not held to the rules (decision 5).
    for pin in &sites.pins {
        let d = pin.direction();
        *counts.entry(pin.kind).or_default() += 1;
        kept.push(Site {
            id: cells.nearest(d),
            kind: pin.kind,
            direction: d,
            name: String::new(),
            capital: pin.capital,
            home: false,
            pinned: true,
            river: false,
        });
    }

    // The small town near the spawn (survey C2): a pinned one if there is
    // one, else the nearest candidate that holds a town.
    let small = |k: SiteKind| matches!(k, SiteKind::Village | SiteKind::Walled);
    if let Some(home) = kept
        .iter_mut()
        .find(|s| small(s.kind) && arc_m(s.direction, spawn, r) <= sites.home_within_m)
    {
        home.home = true;
    } else {
        let mut near: Vec<&Candidate> = candidates
            .iter()
            .filter(|c| small(c.kind) && !struck.contains(&c.id))
            .filter(|c| arc_m(c.direction, spawn, r) <= sites.home_within_m)
            .collect();
        near.sort_by(|a, b| {
            arc_m(a.direction, spawn, r)
                .total_cmp(&arc_m(b.direction, spawn, r))
                .then(a.id.cmp(&b.id))
        });
        if let Some(c) = near.into_iter().find(|c| clear(&kept, c.kind, c.direction)) {
            *counts.entry(c.kind).or_default() += 1;
            kept.push(Site {
                id: c.id,
                kind: c.kind,
                direction: c.direction,
                name: String::new(),
                capital: false,
                home: true,
                pinned: false,
                river: c.river,
            });
        }
    }

    // The greedy pass: by score, then id.
    let mut order: Vec<&Candidate> = candidates
        .iter()
        .filter(|c| !struck.contains(&c.id))
        .collect();
    order.sort_by(|a, b| b.score.total_cmp(&a.score).then(a.id.cmp(&b.id)));
    for c in order {
        let count = counts.get(&c.kind).copied().unwrap_or(0);
        if count >= sites.rule(c.kind).count
            || kept.iter().any(|s| s.id == c.id)
            || !clear(&kept, c.kind, c.direction)
        {
            continue;
        }
        *counts.entry(c.kind).or_default() += 1;
        kept.push(Site {
            id: c.id,
            kind: c.kind,
            direction: c.direction,
            name: String::new(),
            capital: false,
            home: false,
            pinned: false,
            river: c.river,
        });
    }

    // The capital (survey C3): a pinned one, or the walled town furthest
    // from the spawn on another land mass, or the furthest of all.
    if !kept.iter().any(|s| s.capital) {
        let masses = land_masses(terrain, cells);
        let home_mass = masses[cells.nearest(spawn) as usize];
        let mass_of = |s: &Site| masses[cells.nearest(s.direction) as usize];
        let far = |s: &&Site| arc_m(s.direction, spawn, r);
        let towns: Vec<&Site> = kept.iter().filter(|s| s.kind == SiteKind::Walled).collect();
        let pick = towns
            .iter()
            .filter(|s| mass_of(s) != home_mass)
            .max_by(|a, b| far(a).total_cmp(&far(b)).then(b.id.cmp(&a.id)))
            .or_else(|| {
                towns
                    .iter()
                    .max_by(|a, b| far(a).total_cmp(&far(b)).then(b.id.cmp(&a.id)))
            })
            .map(|s| s.id);
        if let Some(id) = pick
            && let Some(s) = kept
                .iter_mut()
                .find(|s| s.id == id && s.kind == SiteKind::Walled)
        {
            s.capital = true;
        }
    }

    // Names: the pins' first, reserved, then each in list order.
    let mut taken: BTreeSet<String> = sites.pins.iter().filter_map(|p| p.name.clone()).collect();
    for (i, site) in kept.iter_mut().enumerate() {
        if site.pinned
            && let Some(name) = sites.pins[i].name.clone()
        {
            site.name = name;
            continue;
        }
        site.name = name_for(sites, terrain.seed, site, &mut taken);
    }

    let shortfall = SiteKind::ALL
        .iter()
        .filter_map(|&k| {
            let want = sites.rule(k).count;
            let have = counts.get(&k).copied().unwrap_or(0);
            (have < want).then_some((k, want - have))
        })
        .collect();
    SiteList {
        sites: kept,
        shortfall,
    }
}

/// A name from the site's people, drawn from a stream seeded by the id; a
/// name already taken takes the next draw (decision 4).
fn name_for(sites: &SitesConfig, seed: u64, site: &Site, taken: &mut BTreeSet<String>) -> String {
    let people = sites.people(site.kind);
    let pick = |list: &[String], k: u64, part: u64| -> String {
        let i = (hash01(&[seed, u64::from(site.id), k, part]) * list.len() as f32) as usize;
        list[i.min(list.len() - 1)].clone()
    };
    let endings = match site.kind {
        SiteKind::Harbour if !people.harbour_endings.is_empty() => &people.harbour_endings,
        SiteKind::Village if site.river && !people.river_endings.is_empty() => {
            &people.river_endings
        }
        _ => &people.endings,
    };
    for k in 0..64u64 {
        let name = join(
            &join(&pick(&people.onsets, k, 1), &pick(&people.middles, k, 2)),
            &pick(endings, k, 3),
        );
        let mut letters = name.chars();
        let name: String = match letters.next() {
            Some(first) => first
                .to_uppercase()
                .chain(letters.flat_map(char::to_lowercase))
                .collect(),
            None => continue,
        };
        if taken.insert(name.clone()) {
            return name;
        }
    }
    let fallback = format!("Site {}", site.id);
    taken.insert(fallback.clone());
    fallback
}

/// Two parts of a name, with a vowel they meet on written once ("Sedge" and
/// "e" make "Sedge", "Mire" and "ewade" make "Mirewade"; finding 5).
fn join(a: &str, b: &str) -> String {
    let meet = a.chars().last().zip(b.chars().next());
    match meet {
        Some((x, y)) if x.eq_ignore_ascii_case(&y) && "aeiouy".contains(x.to_ascii_lowercase()) => {
            format!("{a}{}", &b[y.len_utf8()..])
        }
        _ => format!("{a}{b}"),
    }
}

/// A site's record kind in a world's save (`world-persistence` decision
/// 11), and the list's.
pub const SITE_RECORD: &str = "site";
pub const LIST_RECORD: &str = "site-list";
/// The schema both are written in.
pub const RECORD_SCHEMA: u32 = 1;

/// A site as its record holds it: everything but the id, which is the
/// record's. The direction is the unit vector itself, so the place does not
/// round through degrees.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
struct SiteBody {
    kind: SiteKind,
    name: String,
    capital: bool,
    home: bool,
    pinned: bool,
    river: bool,
    direction: (f32, f32, f32),
}

/// The list's record: which rules made it, and its sites' ids in order. A
/// site record the list does not name is not one of the world's sites.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
struct ListBody {
    version: u32,
    generator: u32,
    ids: Vec<u32>,
}

/// A world's list as the records its save stores: a record per site, then
/// the list's, last, so a list torn by a crash has none and is made again
/// whole (the design's group 3).
pub fn to_records(list: &SiteList, sites_version: u32, generator: u32) -> Vec<Record> {
    let mut records: Vec<Record> = list
        .sites
        .iter()
        .map(|s| {
            let d = s.direction;
            Record::of(
                SITE_RECORD,
                u64::from(s.id),
                RECORD_SCHEMA,
                &SiteBody {
                    kind: s.kind,
                    name: s.name.clone(),
                    capital: s.capital,
                    home: s.home,
                    pinned: s.pinned,
                    river: s.river,
                    direction: (d.x, d.y, d.z),
                },
            )
        })
        .collect();
    records.push(Record::of(
        LIST_RECORD,
        0,
        RECORD_SCHEMA,
        &ListBody {
            version: sites_version,
            generator,
            ids: list.sites.iter().map(|s| s.id).collect(),
        },
    ));
    records
}

/// A world's stored list, in its order, or `None` where the save holds no
/// complete one: no list record, a schema this build does not read, or a
/// site the list names missing.
pub fn from_records(records: &Records) -> Option<Vec<Site>> {
    let list = records.get(LIST_RECORD, 0)?;
    if list.schema != RECORD_SCHEMA {
        return None;
    }
    let list: ListBody = list.read()?;
    list.ids
        .iter()
        .map(|&id| {
            let record = records.get(SITE_RECORD, u64::from(id))?;
            if record.schema != RECORD_SCHEMA {
                return None;
            }
            let body: SiteBody = record.read()?;
            let (x, y, z) = body.direction;
            Some(Site {
                id,
                kind: body.kind,
                direction: Vec3::new(x, y, z),
                name: body.name,
                capital: body.capital,
                home: body.home,
                pinned: body.pinned,
                river: body.river,
            })
        })
        .collect()
}

/// The whole list: [`candidates`] on `threads` threads, then [`select`].
pub fn generate(
    sites: &SitesConfig,
    terrain: &TerrainConfig,
    spawn: Vec3,
    threads: usize,
) -> SiteList {
    let cells = Cells::new(sites.level);
    let found = candidates(sites, terrain, &cells, threads);
    select(sites, terrain, &cells, &found, spawn)
}

#[cfg(test)]
mod tests;
