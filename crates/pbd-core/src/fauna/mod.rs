//! What lives in the water: the species a body carries, where each one may
//! spawn, and the schools they swim in.
//!
//! Where a species spawns is two facts read off the world, never authored per
//! place (`openspec/changes/fishing-and-equipment` design section 7):
//!
//! - its **water class**, from the terrain generator: a river channel, or sea
//!   by depth (shallows, shelf, deep);
//! - the **water temperature now**, which the app reads from the atmosphere.
//!
//! A species lists the classes it lives in and a temperature window; a school
//! of it can spawn only where both match, and never in frozen water. Because
//! the temperature is read at spawn, ranges follow the climate by
//! construction. `tools/fish_ranges.py` draws the same rule as range maps.
//!
//! The roster is per body and explicit: a body with no species has no fish,
//! and no species appears on two bodies (`CLAUDE.md`'s life-roster rule).

pub mod school;

use crate::planet_gen::{TerrainConfig, surface_altitude};
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub use school::{School, Water, crawl_lift, pose, under_share};

/// The home planet's key in the roster. The world has one body with water
/// today; a second gets its own key and its own species.
pub const HOME_BODY: &str = "home";

/// Where a fish can live, from the terrain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum WaterClass {
    /// A channel the river carve cut below the sea: water only because of it.
    River,
    /// Sea no deeper than `WaterLimits::shallows_max_m`: what a cast from the
    /// shore reaches.
    Shallows,
    /// Sea down to `WaterLimits::shelf_max_m`.
    Shelf,
    /// Deeper sea.
    Deep,
}

impl WaterClass {
    pub fn name(self) -> &'static str {
        match self {
            WaterClass::River => "rivers",
            WaterClass::Shallows => "shallows",
            WaterClass::Shelf => "shelf",
            WaterClass::Deep => "deep",
        }
    }
}

/// The class of the water over a direction, or `None` on dry land.
///
/// A river is water that is there only because of the river carve: the same
/// generator with the carve switched off calls it land. That is the one rule,
/// shared by the spawner and the range instrument, rather than a guess from
/// depth that could not tell a river from a shallow bay.
pub fn water_class(
    terrain: &TerrainConfig,
    direction: Vec3,
    depth_m: f32,
    limits: &WaterLimits,
) -> Option<WaterClass> {
    // Version 8's rivers stand above the sea, so their water is not the
    // sea's depth: a river is a river at any height.
    if terrain.rivers.is_some() && is_river(terrain, direction) && depth_m.is_finite() {
        return Some(WaterClass::River);
    }
    if depth_m.is_nan() || depth_m <= 0.0 {
        return None;
    }
    if is_river(terrain, direction) {
        return Some(WaterClass::River);
    }
    Some(if depth_m <= limits.shallows_max_m {
        WaterClass::Shallows
    } else if depth_m <= limits.shelf_max_m {
        WaterClass::Shelf
    } else {
        WaterClass::Deep
    })
}

/// Whether the ground at a direction is below the sea only because the river
/// carve put it there.
pub fn is_river(terrain: &TerrainConfig, direction: Vec3) -> bool {
    // Version 8: the river's own water, fresh, above the sea or at it.
    if terrain.rivers.is_some()
        && crate::rivers::at(terrain, direction)
            .is_some_and(|here| here.across_m < here.half_width_m)
    {
        return true;
    }
    let mut uncut = *terrain;
    uncut.rivers = None;
    // The channel field never exceeds one, so no carve fires.
    uncut.river_threshold = 2.0;
    surface_altitude(terrain, direction) < terrain.sea_level_m
        && surface_altitude(&uncut, direction) >= terrain.sea_level_m
}

/// The depth limits between the sea's classes, and the freezing point.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct WaterLimits {
    /// Deepest water that is still shallows, m.
    pub shallows_max_m: f32,
    /// Deepest water that is still shelf, m.
    pub shelf_max_m: f32,
    /// Sea water freezes below this, deg C, and nothing spawns in it.
    pub freezes_c: f32,
}

impl Default for WaterLimits {
    fn default() -> Self {
        Self {
            shallows_max_m: 6.0,
            shelf_max_m: 40.0,
            freezes_c: -1.8,
        }
    }
}

/// Where a species came from: reused from Tenebris (its `AnimalKind` name,
/// with its icon copied beside the others) or new to this game.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Origin {
    Tenebris(String),
    New,
}

/// A species' body form (`animal-husbandry` decision 1): what the body is
/// like, which is how the Kingdom groups every world's animals.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum Class {
    Mammal,
    Bird,
    ReptileAmphibian,
    #[default]
    Fish,
    Arthropod,
    MolluscSoft,
    /// Sequoia's two-legged tailed walkers (alien branch).
    Twostrider,
    /// Sequoia's fungal animals (alien branch).
    Sporekin,
    /// Wick's twilight-ring life (alien branch).
    Ringling,
}

/// Whether a species can be kept (`animal-husbandry` decision 2).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum Status {
    /// Never tamed: predators, migrants, the too-skittish, swarms of light,
    /// and every fish (kept in tanks, not pens).
    #[default]
    WildOnly,
    /// Can become domestic by its species' way.
    Tameable,
    /// Born on a farm or tamed.
    Domestic,
    /// May not be caught (M25).
    Protected,
}

impl Status {
    /// Why taming is refused, or `None` where it is not.
    pub fn refusal(self, class: Class) -> Option<&'static str> {
        match (self, class) {
            (Status::Tameable | Status::Domestic, _) => None,
            (Status::Protected, _) => Some("protected: it may not be caught"),
            (Status::WildOnly, Class::Fish) => Some("wild only: fish are kept in tanks, not pens"),
            (Status::WildOnly, _) => Some("wild only: it cannot be tamed"),
        }
    }
}

/// How a creature moves (`ocean-environments` design 7): in a school of
/// boids (every fish today), walking the bed, or riding the sea's surface.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum Mover {
    #[default]
    School,
    /// Walks the tops of the cells: crabs, lobsters, clams (which barely do).
    Crawler,
    /// Rides the one sea function: ducks, otters.
    Floater,
}

/// The water places a species lives in (`ocean-environments` design 4 and 7).
/// Answered by `ocean::places::environment` (6.1); `None` (not known) allows
/// every place.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Environment {
    Open,
    Coast,
    Reef,
    Kelp,
    Vent,
    UnderIce,
    Pool,
    Riffle,
    Marsh,
    Estuary,
    Mudflat,
    /// Where plates close under the sea, the floor carved toward the column's
    /// (B10).
    Trench,
}

impl Environment {
    /// Whether `ocean::places::environment` answers it today: the sea's
    /// places. The fresh-water ones (pools, riffles, marsh, estuary,
    /// mudflat) wait for volumetric water (`ocean-environments` group 8).
    pub fn is_placed(self) -> bool {
        matches!(
            self,
            Environment::Open
                | Environment::Coast
                | Environment::Reef
                | Environment::Kelp
                | Environment::Vent
                | Environment::UnderIce
                | Environment::Trench
        )
    }

    /// Whether it exists only where the generator puts it: nowhere on a
    /// world made before the sea's places.
    pub fn needs_generator(self) -> bool {
        matches!(self, Environment::Vent | Environment::Trench)
    }
}

/// What a creature does when the player comes close, or by the clock
/// (`ocean-environments` 7.2). Each is a small rule on the CPU, deterministic
/// like the schools.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum Behaviour {
    #[default]
    None,
    /// Shuts within `shy_m` of the eye and opens again after a while: clams.
    Shuts,
    /// Tips up to feed now and then, paddles off within `shy_m` of the eye,
    /// sleeps on the water at night: dabbling ducks.
    Dabbles,
    /// Dives now and then and comes up elsewhere: diving ducks.
    Dives,
    /// Crawls near its den; within `shy_m` of the eye it inks and jets home,
    /// and hides there for [`HIDE_S`] and until the eye is [`HIDE_UNTIL`]
    /// times `shy_m` off: the octopus (`ocean-environments` B9).
    Hides,
    /// Lies out on the water (a floe, once there is sea ice) when calm;
    /// within `shy_m` slides in and swims under for [`UNDER_S`]: seals.
    HaulsOut,
}

/// How long a startled octopus stays hidden in its den, s.
pub const HIDE_S: f32 = 12.0;
/// How far off, as a multiple of its `shy_m`, the eye must be before a hidden
/// octopus comes out (6 m shy, 8 m to come out).
pub const HIDE_UNTIL: f32 = 4.0 / 3.0;
/// How long a startled seal stays under before it hauls out again, s.
pub const UNDER_S: f32 = 20.0;
/// How much faster than its fastest cruise a startled octopus jets home.
pub const JET: f32 = 4.0;

impl Behaviour {
    /// How long a startle lasts after the eye has gone, s: the caller's
    /// default, or the behaviour's own where it has one.
    pub fn hold_s(self, default_s: f32) -> f32 {
        match self {
            Behaviour::Hides => default_s.max(HIDE_S),
            Behaviour::HaulsOut => default_s.max(UNDER_S),
            _ => default_s,
        }
    }
}

/// How a creature is posed now, for the drawing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pose {
    /// Swimming, walking or floating as it does.
    Moving,
    /// A clam shut.
    Shut,
    /// A dabbler upended, feeding.
    TippedUp,
    /// Under, for a diver; the drawing hides it.
    Under,
    /// A duck asleep on the water, head tucked.
    Asleep,
    /// An octopus jetting home in a cloud of ink.
    Inking,
    /// An octopus hidden in its den: drawn small and flat.
    Hidden,
    /// A seal lying out, still.
    HauledOut,
}

/// A species' field-guide entry: the text the guide shows beside the numbers
/// it reads off the record itself.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Guide {
    pub entry: String,
    pub tip: String,
}

/// One species: everything the spawner, the school, the hook and the field
/// guide read, side by side, so none of them can disagree with the fish.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Species {
    pub id: String,
    pub name: String,
    pub from: Origin,
    /// Its body form (`animal-husbandry` decision 1); every fish is `Fish`.
    #[serde(default)]
    pub class: Class,
    /// Whether it can be kept; every fish is `WildOnly`.
    #[serde(default)]
    pub status: Status,
    /// The water classes it lives in.
    pub water: Vec<WaterClass>,
    /// The water temperature it spawns in, deg C, inclusive.
    pub temp_c: (f32, f32),
    /// A typical fish's length, m.
    pub length_m: f32,
    /// Slowest and fastest cruise, m/s.
    pub speed_mps: (f32, f32),
    /// How far under the surface it swims, m; for a bed dweller, how deep the
    /// bed it lives on may be.
    pub depth_m: (f32, f32),
    /// Lives on the bottom rather than in the water column.
    pub bed: bool,
    /// Fish in a school, fewest and most.
    pub school: (u16, u16),
    /// Schools of it the spawner keeps at once around the player.
    pub max_schools: u8,
    /// How far it notices a lure, m. A school three times as far turns toward
    /// one.
    pub sense_m: f32,
    /// How readily it bites, 0..1.
    pub bite: f32,
    /// How hard it runs on the line.
    pub pull: f32,
    /// Tenebris's fishing strength, 1..5: it sets the hook window.
    pub strength: u8,
    /// Body colour, linear-ish RGB 0..1.
    pub colour: [f32; 3],
    /// Width, height and length of the body relative to `length_m`.
    pub shape: [f32; 3],
    /// Its icon, relative to `assets/items/`, without the extension.
    pub icon: String,
    pub guide: Guide,
    /// How it moves; every fish is `School` (`ocean-environments` 7.1).
    #[serde(default)]
    pub mover: Mover,
    /// The water places it lives in; empty is anywhere its classes fit.
    #[serde(default)]
    pub env: Vec<Environment>,
    /// What it does when the player comes close (7.2), and how close, m.
    #[serde(default)]
    pub behaviour: Behaviour,
    #[serde(default)]
    pub shy_m: f32,
}

impl Species {
    /// Whether it takes a lure: a fish that is not protected and swims in a
    /// school. Crawlers, floaters and protected creatures never bite.
    pub fn can_bite(&self) -> bool {
        self.class == Class::Fish
            && self.status != Status::Protected
            && self.mover == Mover::School
            && self.bite > 0.0
    }

    /// Why catching it is refused, or `None` where it may be caught
    /// (`ocean-environments` 7.3): a protected creature never is, whatever
    /// takes it; anything else that is not a fish is not taken on a line.
    pub fn catch_refusal(&self) -> Option<&'static str> {
        if self.status == Status::Protected {
            Some("protected: it may not be caught")
        } else if !self.can_bite() {
            Some("it is not taken on a line")
        } else {
            None
        }
    }

    /// Whether it may live in this environment (`ocean-environments` 6.1).
    ///
    /// - A place the generator makes (`Some`): a species with no tags, or
    ///   one tagged with that place, or one whose tags are all places the
    ///   generator does not make yet (the fresh-water places wait for
    ///   volumetric water, group 8: ducks keep to their water classes until
    ///   then).
    /// - Unknown (`None`, a world made before the sea's places): anything
    ///   but a species that lives only where the generator puts it (a vent,
    ///   a trench), so the old worlds' life is as it was.
    pub fn env_allows(&self, here: Option<Environment>) -> bool {
        match here {
            Some(env) => {
                self.env.is_empty()
                    || self.env.contains(&env)
                    || !self.env.iter().any(|e| e.is_placed())
            }
            None => self.env.is_empty() || !self.env.iter().all(|e| e.needs_generator()),
        }
    }

    /// Whether a school of it may spawn in this water at this temperature.
    pub fn lives_in(&self, class: WaterClass, temperature_c: f32, limits: &WaterLimits) -> bool {
        temperature_c >= limits.freezes_c
            && self.water.contains(&class)
            && temperature_c >= self.temp_c.0
            && temperature_c <= self.temp_c.1
    }

    /// Whether the water column here is one a school of it fits in: deep
    /// enough for its band, and for a bed dweller, no deeper than the bed it
    /// lives on.
    pub fn fits(&self, depth_m: f32) -> bool {
        depth_m >= self.depth_m.0 + 0.3 && (!self.bed || depth_m <= self.depth_m.1)
    }
}

/// How a school moves: the three boid rules, a goal it wanders between, and
/// the surface and the bed as walls.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct FlockSettings {
    /// Weight of separation, of alignment and of cohesion.
    pub separation: f32,
    pub alignment: f32,
    pub cohesion: f32,
    /// Weight of the wandering goal, and of a goal the school has scented a
    /// lure at.
    pub goal: f32,
    pub lured: f32,
    /// Weight of a lure on the one fish that has left its school for it.
    pub lure: f32,
    /// Push off the surface, the bed and too-shallow water, per metre inside.
    pub wall: f32,
    /// Neighbours closer than this push apart, m.
    pub separation_m: f32,
    /// Neighbours within this align and cohere, m.
    pub neighbour_m: f32,
    /// A new goal after this long, s, fewest and most.
    pub goal_every_s: (f32, f32),
    /// How far from where the school spawned a goal may be, m.
    pub goal_reach_m: f32,
    /// A splash scatters fish within this, m, pushing this hard at its
    /// centre.
    pub splash_m: f32,
    pub splash_push: f32,
    /// A spooked fish stays spooked this long, s, and swims up to twice as
    /// fast meanwhile.
    pub spook_s: f32,
}

impl Default for FlockSettings {
    /// The mockup's weights (`docs/mockups/fishing.html`).
    fn default() -> Self {
        Self {
            separation: 2.6,
            alignment: 1.0,
            cohesion: 0.7,
            goal: 0.45,
            lured: 1.4,
            lure: 3.0,
            wall: 6.0,
            separation_m: 0.55,
            neighbour_m: 2.2,
            goal_every_s: (6.0, 12.0),
            goal_reach_m: 25.0,
            splash_m: 4.0,
            splash_push: 12.0,
            spook_s: 2.0,
        }
    }
}

/// Where and how often schools are made and dropped around the player.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct SpawnSettings {
    /// A new school spawns this far from the player, m, nearest and furthest.
    pub ring_m: (f32, f32),
    /// A school further than this is dropped, m.
    pub despawn_m: f32,
    /// Spawn attempts a second.
    pub attempts_per_s: f32,
    /// Schools of fish (what bites) at once, at most.
    pub max_schools: u8,
    /// Groups of everything else in the water at once (crawlers, floaters,
    /// protected swimmers), at most: their own cap, so they never crowd
    /// the fish a line is cast for (`ocean-environments` 7.1).
    pub max_life: u8,
    /// The schools step at this rate, Hz, on the simulation clock.
    pub step_hz: f32,
}

impl Default for SpawnSettings {
    fn default() -> Self {
        Self {
            ring_m: (20.0, 60.0),
            despawn_m: 90.0,
            attempts_per_s: 4.0,
            max_schools: 8,
            max_life: 6,
            step_hz: 30.0,
        }
    }
}

/// Every number the rod works by. Tenebris's where it has one (the hook
/// window); the mockup's otherwise.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct FishingSettings {
    /// Holding the button this long charges a full cast, s.
    pub charge_s: f32,
    /// Cast speed along the look, m/s, from a tap to a full charge.
    pub speed_mps: (f32, f32),
    /// And up off the ground, m/s.
    pub lift_mps: (f32, f32),
    /// A cast still flying after this long comes back empty, s (Tenebris).
    pub flight_max_s: f32,
    /// The line comes back if the angler walks this far from the float, m.
    pub line_max_m: f32,
    /// Fish ignore the float for this long after the splash, s.
    pub splash_s: f32,
    /// How often a floating line is checked for interest, s.
    pub scent_every_s: f32,
    /// A school within this many of its sense ranges turns toward the float.
    pub scent_reach: f32,
    /// Chance per check, before species and weather, that a school in range
    /// sends a fish.
    pub bite_chance: f32,
    /// A fish this close to the lure starts nibbling, m.
    pub nibble_m: f32,
    /// Nibbles before the bite, fewest and most, and how long each lasts, s.
    pub nibbles: (u8, u8),
    pub nibble_s: (f32, f32),
    /// The hook window: `window_s * (1 - (strength - 1) * step)`, never under
    /// `floor_s` (Tenebris, `client fishing.rs:338-343`).
    pub hook_window_s: f32,
    pub hook_step: f32,
    pub hook_floor_s: f32,
    /// How far the float goes under on a bite, and on a nibble, m.
    pub bite_dip_m: f32,
    pub nibble_dip_m: f32,
    /// Line taken in a second while reeling, m/s.
    pub reel_mps: f32,
    /// Tension added a second while reeling: this, plus the pull times the
    /// fish's stamina times `tension_pull`.
    pub tension_base: f32,
    pub tension_pull: f32,
    /// Tension shed a second with the button up.
    pub tension_fall: f32,
    /// Stamina a running fish loses a second against a taut line, and the
    /// least it keeps.
    pub stamina_drain: f32,
    pub stamina_floor: f32,
    /// Line out below which the fish is landed, m, and water shallower than
    /// which it is beached, m.
    pub land_line_m: f32,
    pub land_depth_m: f32,
    /// Bite multipliers: full cloud cover adds this, full rain this, rain
    /// counting as full at `rain_full_mm_h`.
    pub cover_bite: f32,
    pub rain_bite: f32,
    pub rain_full_mm_h: f32,
}

impl Default for FishingSettings {
    fn default() -> Self {
        Self {
            charge_s: 1.2,
            speed_mps: (5.0, 18.0),
            lift_mps: (3.0, 6.0),
            flight_max_s: 5.0,
            line_max_m: 45.0,
            splash_s: 1.2,
            scent_every_s: 0.5,
            scent_reach: 3.0,
            bite_chance: 0.175,
            nibble_m: 0.45,
            nibbles: (1, 3),
            nibble_s: (0.6, 1.4),
            hook_window_s: 0.9,
            hook_step: 0.13,
            hook_floor_s: 0.25,
            bite_dip_m: 0.14,
            nibble_dip_m: 0.035,
            reel_mps: 1.8,
            tension_base: 0.35,
            tension_pull: 0.8,
            tension_fall: 0.7,
            stamina_drain: 0.08,
            stamina_floor: 0.15,
            land_line_m: 1.3,
            land_depth_m: 0.15,
            cover_bite: 0.35,
            rain_bite: 0.7,
            rain_full_mm_h: 5.0,
        }
    }
}

impl FishingSettings {
    /// The hook window for a fish of this strength, s.
    pub fn hook_window(&self, strength: u8) -> f32 {
        let steps = strength.max(1) as f32 - 1.0;
        (self.hook_window_s * (1.0 - steps * self.hook_step)).max(self.hook_floor_s)
    }

    /// How much more readily fish bite under this sky: one in clear weather,
    /// more under cloud and more again in rain.
    pub fn bite_factor(&self, cover: f32, rain_mm_h: f32) -> f32 {
        let cover = if cover.is_finite() {
            cover.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let rain = if rain_mm_h.is_finite() && self.rain_full_mm_h > 0.0 {
            (rain_mm_h / self.rain_full_mm_h).clamp(0.0, 1.0)
        } else {
            0.0
        };
        1.0 + self.cover_bite * cover + self.rain_bite * rain
    }
}

/// `assets/config/fauna.ron`: the water limits, how schools move and spawn,
/// how the rod works, and every body's species.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct FaunaSettings {
    pub water: WaterLimits,
    pub flock: FlockSettings,
    pub spawn: SpawnSettings,
    pub fishing: FishingSettings,
    /// Each body's species, in roster order. Append only: a caught fish is
    /// saved as its index here.
    pub bodies: BTreeMap<String, Vec<Species>>,
}

impl Default for FaunaSettings {
    fn default() -> Self {
        let mut bodies = BTreeMap::new();
        bodies.insert(HOME_BODY.to_string(), home_roster());
        Self {
            water: WaterLimits::default(),
            flock: FlockSettings::default(),
            spawn: SpawnSettings::default(),
            fishing: FishingSettings::default(),
            bodies,
        }
    }
}

impl FaunaSettings {
    /// A body's species; empty for a body with none, which spawns nothing.
    pub fn roster(&self, body: &str) -> &[Species] {
        self.bodies.get(body).map_or(&[], Vec::as_slice)
    }

    pub fn validate(&self) -> Result<(), String> {
        let w = &self.water;
        finite_positive("water limits", &[w.shallows_max_m, w.shelf_max_m])?;
        if w.shelf_max_m <= w.shallows_max_m || !w.freezes_c.is_finite() {
            return Err("water limits: the shelf must be deeper than the shallows".into());
        }
        let f = &self.flock;
        finite_positive(
            "flock",
            &[
                f.separation_m,
                f.neighbour_m,
                f.goal_every_s.0,
                f.goal_every_s.1,
                f.goal_reach_m,
                f.splash_m,
                f.spook_s,
            ],
        )?;
        let s = &self.spawn;
        finite_positive(
            "spawn",
            &[
                s.ring_m.0,
                s.ring_m.1,
                s.despawn_m,
                s.attempts_per_s,
                s.step_hz,
            ],
        )?;
        if s.ring_m.1 < s.ring_m.0 || s.despawn_m <= s.ring_m.1 {
            return Err("spawn: the ring must be ordered and inside the despawn range".into());
        }
        let r = &self.fishing;
        finite_positive(
            "fishing",
            &[
                r.charge_s,
                r.speed_mps.0,
                r.speed_mps.1,
                r.flight_max_s,
                r.line_max_m,
                r.scent_every_s,
                r.scent_reach,
                r.nibble_m,
                r.hook_window_s,
                r.hook_floor_s,
                r.reel_mps,
                r.tension_fall,
                r.land_line_m,
                r.rain_full_mm_h,
            ],
        )?;
        let mut seen = std::collections::BTreeSet::new();
        for (body, species) in &self.bodies {
            for sp in species {
                let at = format!("{body}/{}", sp.id);
                if !seen.insert(sp.id.clone()) {
                    return Err(format!("{at}: a species lives on one body only"));
                }
                if sp.water.is_empty() {
                    return Err(format!("{at}: lives in no water"));
                }
                if sp.temp_c.0.is_nan()
                    || sp.temp_c.0 >= sp.temp_c.1
                    || sp.temp_c.0 < w.freezes_c
                    || !sp.temp_c.1.is_finite()
                {
                    return Err(format!(
                        "{at}: the temperature window must be ordered and above freezing"
                    ));
                }
                finite_positive(
                    &at,
                    &[
                        sp.length_m,
                        sp.speed_mps.0,
                        sp.speed_mps.1,
                        sp.depth_m.1,
                        sp.sense_m,
                        sp.pull,
                    ],
                )?;
                if sp.speed_mps.1 < sp.speed_mps.0
                    || sp.depth_m.1 < sp.depth_m.0
                    || sp.depth_m.0 < 0.0
                {
                    return Err(format!("{at}: speeds and depths must be ordered"));
                }
                if sp.school.0 == 0 || sp.school.1 < sp.school.0 || sp.max_schools == 0 {
                    return Err(format!("{at}: a school has at least one fish"));
                }
                if !(1..=5).contains(&sp.strength) || !(0.0..=1.0).contains(&sp.bite) {
                    return Err(format!("{at}: strength is 1..5 and bite 0..1"));
                }
                if sp.guide.entry.trim().is_empty()
                    || sp.guide.tip.trim().is_empty()
                    || sp.icon.trim().is_empty()
                    || sp.name.trim().is_empty()
                {
                    return Err(format!(
                        "{at}: every species has a name, an icon and a field-guide entry"
                    ));
                }
            }
        }
        Ok(())
    }
}

fn finite_positive(name: &str, values: &[f32]) -> Result<(), String> {
    if values.iter().all(|v| v.is_finite() && *v > 0.0) {
        Ok(())
    } else {
        Err(format!("{name}: every value must be finite and positive"))
    }
}

/// The home planet's eight species: five from Tenebris, three new. The table
/// and the reasons are design section 7; the windows overlap so that on day
/// one no open water has none (measured by the range atlas).
fn home_roster() -> Vec<Species> {
    #[allow(clippy::too_many_arguments)]
    fn sp(
        id: &str,
        name: &str,
        from: Origin,
        water: &[WaterClass],
        temp_c: (f32, f32),
        length_m: f32,
        speed_mps: (f32, f32),
        depth_m: (f32, f32),
        bed: bool,
        school: (u16, u16),
        max_schools: u8,
        sense_m: f32,
        bite: f32,
        pull: f32,
        strength: u8,
        colour: [f32; 3],
        shape: [f32; 3],
        entry: &str,
        tip: &str,
    ) -> Species {
        Species {
            id: id.into(),
            name: name.into(),
            from,
            class: Class::Fish,
            status: Status::WildOnly,
            water: water.to_vec(),
            temp_c,
            length_m,
            speed_mps,
            depth_m,
            bed,
            school,
            max_schools,
            sense_m,
            bite,
            pull,
            strength,
            colour,
            shape,
            icon: format!("fish/{id}"),
            guide: Guide {
                entry: entry.into(),
                tip: tip.into(),
            },
            mover: Mover::School,
            env: Vec::new(),
            behaviour: Behaviour::None,
            shy_m: 0.0,
        }
    }
    use WaterClass::*;
    let tenebris = |kind: &str| Origin::Tenebris(kind.into());
    let mut roster = vec![
        sp(
            "minnow",
            "Minnow",
            tenebris("Fish"),
            &[River, Shallows],
            (10.0, 30.0),
            0.14,
            (1.0, 2.6),
            (0.3, 1.8),
            false,
            (24, 36),
            2,
            8.0,
            0.6,
            0.4,
            1,
            [0.47, 0.59, 0.71],
            [0.5, 0.5, 1.0],
            "The first fish most anglers land. Minnows crowd the shallows and river mouths in schools of thirty or more and turn as one when a shadow crosses them. They bite quickly and give up easily.",
            "Cast anywhere near a warm shore. A school finds the lure in seconds.",
        ),
        sp(
            "silverfin",
            "Silverfin",
            Origin::New,
            &[Shallows, Shelf],
            (-1.8, 17.0),
            0.22,
            (1.2, 3.2),
            (0.4, 2.5),
            false,
            (18, 30),
            2,
            7.0,
            0.55,
            0.55,
            1,
            [0.81, 0.89, 0.92],
            [0.5, 0.5, 1.0],
            "A bright, fast surface fish of cool coasts, from the edge of the ice to water as warm as 17 degrees. Silverfin skim the top two metres and scatter at a splash, then close ranks again a few seconds later.",
            "Let the splash settle before you expect a bite.",
        ),
        sp(
            "perch",
            "Banded perch",
            Origin::New,
            &[River],
            (-1.8, 24.0),
            0.38,
            (0.8, 2.4),
            (0.5, 2.0),
            false,
            (8, 14),
            2,
            9.0,
            0.4,
            0.85,
            2,
            [0.85, 0.63, 0.23],
            [0.55, 0.62, 1.0],
            "Striped, deep-bodied and curious. Perch hold the river channels in loose groups of a dozen, in any river that does not freeze, and one will follow a lure a long way before it commits.",
            "Nibbles come in threes. Wait for the float to go under.",
        ),
        sp(
            "ray",
            "Ray",
            tenebris("FlatFish"),
            &[Shallows, Shelf],
            (16.0, 32.0),
            0.6,
            (0.5, 1.2),
            (1.5, 40.0),
            true,
            (1, 2),
            1,
            10.0,
            0.35,
            0.9,
            2,
            [0.73, 0.65, 0.49],
            [1.7, 0.22, 0.8],
            "A flat grazer that lies on the sand and lifts off in slow wingbeats. Rays travel alone or in pairs along the bed of warm shallows and shelf, well out from the beach.",
            "Cast long, over sand. A ray never schools, so a bite is one fish, not one of many.",
        ),
        sp(
            "eel",
            "Eel",
            tenebris("LongFish"),
            &[River, Shallows],
            (6.0, 28.0),
            0.9,
            (0.7, 1.9),
            (1.0, 6.0),
            true,
            (1, 2),
            1,
            9.0,
            0.3,
            1.1,
            3,
            [0.47, 0.51, 0.28],
            [0.28, 0.28, 1.8],
            "Long, dark and patient. Eels keep to the bed of rivers and shallows in ones and twos and fight in hard, twisting runs.",
            "Ease off whenever it runs. More lines are lost to eels than to anything else.",
        ),
        sp(
            "reef",
            "Reef fish",
            tenebris("LargeFish"),
            &[Shallows],
            (23.0, 32.0),
            0.42,
            (0.6, 1.6),
            (0.8, 3.0),
            false,
            (6, 12),
            1,
            8.0,
            0.25,
            1.2,
            5,
            [0.24, 0.59, 0.59],
            [0.5, 0.8, 1.0],
            "The prize of warm water. Tall-finned and nervous, reef fish shoal in small groups over tropical shallows that never fall below 23 degrees, and give the shortest bite window of anything in the sea.",
            "The hook window is under half a second. Watch the float, not the fish.",
        ),
        sp(
            "deepback",
            "Deepback",
            Origin::New,
            &[Shelf, Deep],
            (-1.5, 9.0),
            0.7,
            (0.6, 1.8),
            (2.5, 6.0),
            false,
            (4, 7),
            1,
            12.0,
            0.22,
            1.35,
            4,
            [0.31, 0.44, 0.56],
            [0.55, 0.55, 1.0],
            "A heavy cold-water fish of the shelf and the deep. Small schools cruise below two and a half metres, and a hooked deepback runs long and tires slowly.",
            "Fish where the shelf drops close to the shore, where the water turns dark.",
        ),
        sp(
            "serpent",
            "Sea serpent",
            tenebris("SerpentFish"),
            &[Deep],
            (-1.8, 32.0),
            1.3,
            (0.8, 2.2),
            (3.5, 6.0),
            false,
            (1, 1),
            1,
            14.0,
            0.12,
            1.45,
            4,
            [0.19, 0.47, 0.42],
            [0.35, 0.4, 2.2],
            "Rare and solitary, found only past the drop-off in any deep sea that does not freeze. A serpent seldom takes a lure and pulls like a boat. Every angler has a story about one that got away.",
            "There is at most one in any water. Deep water, a long cast and patience.",
        ),
    ];
    roster.extend(water_life());
    roster
}

/// The water life appended after the eight fish (`ocean-environments` 7.1):
/// crawlers, floaters and protected swimmers. APPEND ONLY, like the fish.
fn water_life() -> Vec<Species> {
    use Environment::*;
    use WaterClass::*;
    vec![
        Species {
            id: "shore_crab".into(),
            name: "Shore crab".into(),
            from: Origin::New,
            class: Class::Arthropod,
            status: Status::WildOnly,
            water: vec![River, Shallows],
            temp_c: (2.0, 30.0),
            length_m: 0.1,
            speed_mps: (0.08, 0.35),
            depth_m: (0.3, 6.0),
            bed: true,
            school: (2, 4),
            max_schools: 2,
            sense_m: 3.0,
            bite: 0.0,
            pull: 0.2,
            strength: 1,
            colour: [0.8, 0.36, 0.18],
            shape: [1.6, 0.4, 0.9],
            icon: "water_life/shore_crab".into(),
            guide: Guide {
                entry: "A small green-brown crab of every warm or cool shore, in rock pools, river mouths and the sand of the shallows. It walks sideways along the bed and scuttles off when a shadow comes near.".into(),
                tip: "Look in the shallows by the rocks. It does not take a lure.".into(),
            },
            mover: Mover::Crawler,
            env: vec![Coast, Pool, Estuary, Mudflat],
            behaviour: Behaviour::None,
            shy_m: 2.5,
        },
        Species {
            id: "lobster".into(),
            name: "Spiny lobster".into(),
            from: Origin::New,
            class: Class::Arthropod,
            status: Status::WildOnly,
            water: vec![Shallows, Shelf],
            temp_c: (4.0, 26.0),
            length_m: 0.4,
            speed_mps: (0.05, 0.25),
            depth_m: (2.0, 40.0),
            bed: true,
            school: (1, 2),
            max_schools: 1,
            sense_m: 4.0,
            bite: 0.0,
            pull: 0.5,
            strength: 1,
            colour: [0.55, 0.2, 0.12],
            shape: [0.5, 0.35, 1.0],
            icon: "water_life/lobster".into(),
            guide: Guide {
                entry: "A long-feelered crawler of rocky shelf and reef, from cool kelp to warm coral. It keeps to the bed in ones and twos and backs away from anything that swims too close.".into(),
                tip: "On the bed of the shelf, under kelp or along a reef. It does not take a lure.".into(),
            },
            mover: Mover::Crawler,
            env: vec![Kelp, Reef],
            behaviour: Behaviour::None,
            shy_m: 3.0,
        },
        Species {
            id: "giant_clam".into(),
            name: "Giant clam".into(),
            from: Origin::New,
            class: Class::MolluscSoft,
            status: Status::WildOnly,
            water: vec![Shallows],
            temp_c: (23.0, 32.0),
            length_m: 0.6,
            speed_mps: (0.001, 0.002),
            depth_m: (1.0, 6.0),
            bed: true,
            school: (1, 3),
            max_schools: 1,
            sense_m: 4.0,
            bite: 0.0,
            pull: 0.1,
            strength: 1,
            colour: [0.35, 0.55, 0.75],
            shape: [1.2, 0.6, 0.8],
            icon: "water_life/giant_clam".into(),
            guide: Guide {
                entry: "A clam as wide as a cart wheel, set in the sand of warm reef shallows with its blue mantle open to the sun. It shuts when anything comes within four metres and opens again once it has gone.".into(),
                tip: "Swim up slowly and it still shuts. Wait a while at a distance and it opens.".into(),
            },
            mover: Mover::Crawler,
            env: vec![Reef],
            behaviour: Behaviour::Shuts,
            shy_m: 4.0,
        },
        Species {
            id: "reef_turtle".into(),
            name: "Reef turtle".into(),
            from: Origin::New,
            class: Class::ReptileAmphibian,
            status: Status::Protected,
            water: vec![Shallows, Shelf],
            temp_c: (20.0, 32.0),
            length_m: 0.9,
            speed_mps: (0.4, 1.2),
            depth_m: (1.0, 8.0),
            bed: false,
            school: (1, 1),
            max_schools: 1,
            sense_m: 10.0,
            bite: 0.0,
            pull: 1.0,
            strength: 3,
            colour: [0.36, 0.45, 0.25],
            shape: [1.0, 0.35, 1.1],
            icon: "water_life/reef_turtle".into(),
            guide: Guide {
                entry: "A slow, old swimmer of warm reefs and the shelf beyond them, alone and unhurried. Reef turtles are protected: none may be caught.".into(),
                tip: "Watch it go by. A line cast at one is refused.".into(),
            },
            mover: Mover::School,
            env: vec![Reef, Coast],
            behaviour: Behaviour::None,
            shy_m: 0.0,
        },
        Species {
            id: "sea_otter".into(),
            name: "Sea otter".into(),
            from: Origin::New,
            class: Class::Mammal,
            status: Status::Protected,
            water: vec![Shallows, Shelf],
            temp_c: (2.0, 16.0),
            length_m: 1.2,
            speed_mps: (0.2, 0.8),
            depth_m: (1.0, 40.0),
            bed: false,
            school: (1, 3),
            max_schools: 1,
            sense_m: 8.0,
            bite: 0.0,
            pull: 1.0,
            strength: 3,
            colour: [0.36, 0.24, 0.15],
            shape: [0.35, 0.25, 1.0],
            icon: "water_life/sea_otter".into(),
            guide: Guide {
                entry: "Rafts of two or three float on their backs over cool kelp and rocky coasts, and paddle off when a swimmer comes near. Sea otters are protected: none may be caught.".into(),
                tip: "Cool coasts, over kelp. Keep your distance and they stay.".into(),
            },
            mover: Mover::Floater,
            env: vec![Kelp, Coast],
            behaviour: Behaviour::None,
            shy_m: 8.0,
        },
        Species {
            id: "reed_dabbler".into(),
            name: "Reed dabbler".into(),
            from: Origin::New,
            class: Class::Bird,
            status: Status::Protected,
            water: vec![River, Shallows],
            temp_c: (2.0, 30.0),
            length_m: 0.5,
            speed_mps: (0.15, 0.6),
            depth_m: (0.3, 6.0),
            bed: false,
            school: (3, 8),
            max_schools: 2,
            sense_m: 6.0,
            bite: 0.0,
            pull: 0.3,
            strength: 1,
            colour: [0.42, 0.36, 0.22],
            shape: [0.45, 0.4, 1.0],
            icon: "water_life/reed_dabbler".into(),
            guide: Guide {
                entry: "The common duck of rivers, marsh pools and quiet shallows. Dabblers float in small flocks, tip up to feed with their tails in the air, paddle off when you come close and sleep on the water at night. Ducks are protected; their eggs and feathers are gathered, not taken.".into(),
                tip: "Slow water near reeds. Come up quietly or they paddle off.".into(),
            },
            mover: Mover::Floater,
            env: vec![Marsh, Pool, Estuary, Riffle],
            behaviour: Behaviour::Dabbles,
            shy_m: 9.0,
        },
        Species {
            id: "pond_diver".into(),
            name: "Pond diver".into(),
            from: Origin::New,
            class: Class::Bird,
            status: Status::Protected,
            water: vec![River, Shallows],
            temp_c: (2.0, 28.0),
            length_m: 0.42,
            speed_mps: (0.2, 0.7),
            depth_m: (1.0, 6.0),
            bed: false,
            school: (1, 3),
            max_schools: 1,
            sense_m: 6.0,
            bite: 0.0,
            pull: 0.3,
            strength: 1,
            colour: [0.12, 0.12, 0.14],
            shape: [0.42, 0.38, 1.0],
            icon: "water_life/pond_diver".into(),
            guide: Guide {
                entry: "A dark little duck that dives for its food and comes up somewhere else. Pond divers keep to deeper pools and slow river reaches in ones and twos. Ducks are protected.".into(),
                tip: "Watch the water where it went under; it surfaces a few metres off.".into(),
            },
            mover: Mover::Floater,
            env: vec![Marsh, Pool],
            behaviour: Behaviour::Dives,
            shy_m: 9.0,
        },
        // The octopus, the plume worm and the seal (B9), appended.
        Species {
            id: "octopus".into(),
            name: "Den octopus".into(),
            from: Origin::New,
            class: Class::MolluscSoft,
            status: Status::WildOnly,
            water: vec![Shallows, Shelf],
            temp_c: (6.0, 32.0),
            length_m: 0.5,
            speed_mps: (0.05, 0.3),
            depth_m: (1.0, 40.0),
            bed: true,
            school: (1, 1),
            max_schools: 1,
            sense_m: 6.0,
            bite: 0.0,
            pull: 0.6,
            strength: 2,
            colour: [0.83, 0.27, 0.17],
            shape: [0.8, 0.4, 1.0],
            icon: "water_life/octopus".into(),
            guide: Guide {
                entry: "A clever crawler of reefs and rocky shelf that keeps a den with shells piled at its door. It takes the colour of the rock when still and flushes red when it moves. Come within six metres and it inks and jets home, and stays hidden until you have gone.".into(),
                tip: "Look for the shells at its door, then keep back and wait.".into(),
            },
            mover: Mover::Crawler,
            env: vec![Reef],
            behaviour: Behaviour::Hides,
            shy_m: 6.0,
        },
        Species {
            id: "tube_worm".into(),
            name: "Plume worm".into(),
            from: Origin::New,
            class: Class::MolluscSoft,
            status: Status::WildOnly,
            water: vec![Deep],
            temp_c: (-1.8, 32.0),
            length_m: 1.4,
            speed_mps: (0.0005, 0.001),
            depth_m: (90.0, 145.0),
            bed: true,
            school: (10, 16),
            max_schools: 1,
            sense_m: 5.0,
            bite: 0.0,
            pull: 0.1,
            strength: 1,
            colour: [0.82, 0.18, 0.25],
            shape: [0.15, 0.05, 0.15],
            icon: "water_life/tube_worm".into(),
            guide: Guide {
                entry: "White tubes taller than a person crowd round the deep vents, each with a red plume out in the warm water. The plume snaps back into its tube when anything comes within five metres. They live on the vent, not on the light.".into(),
                tip: "Only at the vents on the deepest floor. Nothing worth taking; the vent is the prize.".into(),
            },
            mover: Mover::Crawler,
            env: vec![Vent],
            behaviour: Behaviour::Shuts,
            shy_m: 5.0,
        },
        Species {
            id: "seal".into(),
            name: "Ice seal".into(),
            from: Origin::New,
            class: Class::Mammal,
            status: Status::Protected,
            water: vec![Shallows, Shelf],
            temp_c: (-1.8, 14.0),
            length_m: 1.6,
            speed_mps: (0.3, 1.5),
            depth_m: (1.0, 40.0),
            bed: false,
            school: (1, 3),
            max_schools: 1,
            sense_m: 9.0,
            bite: 0.0,
            pull: 1.0,
            strength: 3,
            colour: [0.36, 0.47, 0.55],
            shape: [0.4, 0.35, 1.0],
            icon: "water_life/seal".into(),
            guide: Guide {
                entry: "Grey seals of the cold seas and the kelp. They lie out on the floes and slide in when you come within nine metres, then surface somewhere else a while later. Seals are protected: none may be caught.".into(),
                tip: "Cold water, near the ice or the kelp. Where seals are, the fish are.".into(),
            },
            mover: Mover::Floater,
            env: vec![UnderIce, Kelp],
            behaviour: Behaviour::HaulsOut,
            shy_m: 9.0,
        },
    ]
}

/// A small, fast, seeded generator, so a school and a line are functions of
/// their seed and the steps they were given, and nothing here reads a global.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// SplitMix64.
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        mix(self.0)
    }

    /// Uniform in [0, 1).
    pub fn unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    pub fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.unit()
    }
}

/// SplitMix64's finaliser: a well-mixed hash of one word.
pub fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Two words hashed together.
pub fn hash2(a: u64, b: u64) -> u64 {
    mix(a ^ mix(b.wrapping_add(0x9E37_79B9_7F4A_7C15)))
}

#[cfg(test)]
mod tests;
