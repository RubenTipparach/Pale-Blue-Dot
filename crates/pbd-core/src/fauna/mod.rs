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
pub mod swim;

use crate::locale::{Localized, tr};
use crate::planet_gen::{TerrainConfig, surface_altitude};
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub use school::{School, Water, crawl_lift, pose, under_share};
pub use swim::BehaviourSettings;

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
    /// Its name as the field guide lists it.
    pub fn label(self) -> Localized {
        tr(match self {
            WaterClass::River => "fauna.water.rivers",
            WaterClass::Shallows => "fauna.water.shallows",
            WaterClass::Shelf => "fauna.water.shelf",
            WaterClass::Deep => "fauna.water.deep",
        })
    }

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
    pub fn refusal(self, class: Class) -> Option<Localized> {
        match (self, class) {
            (Status::Tameable | Status::Domestic, _) => None,
            (Status::Protected, _) => Some(tr("fauna.refusal.protected")),
            (Status::WildOnly, Class::Fish) => Some(tr("fauna.refusal.fish_wild")),
            (Status::WildOnly, _) => Some(tr("fauna.refusal.wild")),
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

/// One species: everything the spawner, the school, the hook and the field
/// guide read, side by side, so none of them can disagree with the fish.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Species {
    /// Its id, which its words in the locale file are under:
    /// `species.<id>.name`, `.entry` (the field guide's text) and `.tip`.
    pub id: String,
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
    /// Its name: "Minnow", "Yard hen".
    pub fn name(&self) -> Localized {
        tr(&format!("species.{}.name", self.id))
    }

    /// Its name in capitals, as the field guide's heading shows it.
    pub fn caps_name(&self) -> Localized {
        tr(&format!("species.{}.caps", self.id))
    }

    /// Its name mid-sentence ("the minnow got away"): `species.<id>.lower`.
    pub fn lower_name(&self) -> Localized {
        tr(&format!("species.{}.lower", self.id))
    }

    /// Its name as missions and their saved progress name it: the English
    /// name in lower case, whatever the player's language ("banded perch").
    pub fn data_name(&self) -> String {
        crate::locale::english(&format!("species.{}.name", self.id)).to_lowercase()
    }

    /// Its field-guide entry, beside the numbers the guide reads off it.
    pub fn entry(&self) -> Localized {
        tr(&format!("species.{}.entry", self.id))
    }

    /// Its field-guide tip.
    pub fn tip(&self) -> Localized {
        tr(&format!("species.{}.tip", self.id))
    }

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
    pub fn catch_refusal(&self) -> Option<Localized> {
        if self.status == Status::Protected {
            Some(tr("fauna.refusal.protected"))
        } else if !self.can_bite() {
            Some(tr("fauna.refusal.no_line"))
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
    /// The fastest a fish's swim turns in a step, deg/s (`fish-behaviour`
    /// design 4): a boid that reverses turns round, never back in one step.
    pub turn_deg_s: f32,
    /// Each fish swims a burst-glide cycle of its own: this long, s, the
    /// share of it bursting, and the drag of its glide, 1/s.
    pub burst_s: f32,
    pub burst_share: f32,
    pub glide_drag_per_s: f32,
    /// How far each fish wanders up and down round the school's goal, m.
    pub depth_wander_m: f32,
    /// The fish that has left its school for a lure (design 2): it comes in
    /// at this share of its cruising speed, slowing inside `slow_m`, to a
    /// stand-off point on the side it came from, its mouth `standoff_m` off
    /// the lure, its middle never further than `standoff_max_m` (inside
    /// the line's `nibble_m`, so a long fish still nibbles).
    pub approach: f32,
    pub slow_m: f32,
    pub standoff_m: f32,
    pub standoff_max_m: f32,
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
            turn_deg_s: 220.0,
            burst_s: 1.8,
            burst_share: 0.35,
            glide_drag_per_s: 0.9,
            depth_wander_m: 0.18,
            approach: 0.75,
            slow_m: 1.3,
            standoff_m: 0.15,
            standoff_max_m: 0.36,
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
    /// A baited line's bite factor is the weather's times `1 + bait_bite`
    /// (`starter-missions` G1).
    pub bait_bite: f32,
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
            bait_bite: 0.5,
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

    /// The bite factor of a line, baited or not, under a sky whose factor is
    /// `weather` ([`Self::bite_factor`]).
    pub fn baited(&self, weather: f32, baited: bool) -> f32 {
        if baited {
            weather * (1.0 + self.bait_bite)
        } else {
            weather
        }
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
    /// How a fish is drawn moving (`fish-behaviour`): its heading and bank,
    /// the fish at the lure and on the line, and the body wave.
    pub behaviour: BehaviourSettings,
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
            behaviour: BehaviourSettings::default(),
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
                f.turn_deg_s,
                f.burst_s,
                f.approach,
                f.slow_m,
                f.standoff_max_m,
            ],
        )?;
        if !(0.0..=1.0).contains(&f.burst_share)
            || !(f.glide_drag_per_s.is_finite() && f.glide_drag_per_s >= 0.0)
            || !(f.depth_wander_m.is_finite() && f.depth_wander_m >= 0.0)
            || !(f.standoff_m.is_finite() && f.standoff_m >= 0.0)
        {
            return Err(
                "flock: the burst share is 0..1, and the glide drag, the depth \
                        wander and the stand-off are not negative"
                    .into(),
            );
        }
        self.behaviour.validate()?;
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
        if f.standoff_max_m >= r.nibble_m {
            return Err(format!(
                "flock.standoff_max_m ({}) must be inside fishing.nibble_m ({}), or a fish \
                 at its stand-off never nibbles",
                f.standoff_max_m, r.nibble_m
            ));
        }
        if !(r.bait_bite.is_finite() && r.bait_bite >= 0.0) {
            return Err(format!(
                "fishing.bait_bite must be finite and not negative, got {}",
                r.bait_bite
            ));
        }
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
                if sp.icon.trim().is_empty() {
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
    ) -> Species {
        Species {
            id: id.into(),
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
        ),
        sp(
            "silverfin",
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
        ),
        sp(
            "perch",
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
        ),
        sp(
            "ray",
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
        ),
        sp(
            "eel",
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
        ),
        sp(
            "reef",
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
        ),
        sp(
            "deepback",
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
        ),
        sp(
            "serpent",
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
            mover: Mover::Crawler,
            env: vec![Coast, Pool, Estuary, Mudflat],
            behaviour: Behaviour::None,
            shy_m: 2.5,
        },
        Species {
            id: "lobster".into(),
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
            mover: Mover::Crawler,
            env: vec![Kelp, Reef],
            behaviour: Behaviour::None,
            shy_m: 3.0,
        },
        Species {
            id: "giant_clam".into(),
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
            mover: Mover::Crawler,
            env: vec![Reef],
            behaviour: Behaviour::Shuts,
            shy_m: 4.0,
        },
        Species {
            id: "reef_turtle".into(),
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
            mover: Mover::School,
            env: vec![Reef, Coast],
            behaviour: Behaviour::None,
            shy_m: 0.0,
        },
        Species {
            id: "sea_otter".into(),
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
            mover: Mover::Floater,
            env: vec![Kelp, Coast],
            behaviour: Behaviour::None,
            shy_m: 8.0,
        },
        Species {
            id: "reed_dabbler".into(),
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
            mover: Mover::Floater,
            env: vec![Marsh, Pool, Estuary, Riffle],
            behaviour: Behaviour::Dabbles,
            shy_m: 9.0,
        },
        Species {
            id: "pond_diver".into(),
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
            mover: Mover::Floater,
            env: vec![Marsh, Pool],
            behaviour: Behaviour::Dives,
            shy_m: 9.0,
        },
        // The octopus, the plume worm and the seal (B9), appended.
        Species {
            id: "octopus".into(),
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
            mover: Mover::Crawler,
            env: vec![Reef],
            behaviour: Behaviour::Hides,
            shy_m: 6.0,
        },
        Species {
            id: "tube_worm".into(),
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
            mover: Mover::Crawler,
            env: vec![Vent],
            behaviour: Behaviour::Shuts,
            shy_m: 5.0,
        },
        Species {
            id: "seal".into(),
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
