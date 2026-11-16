//! Sound's rules (`audio-and-ambience`), free of any engine: the cue data and
//! its validation, how loud and how bright a placed sound is at a distance,
//! when it arrives, how thin the air is, which footstep a material makes, the
//! stride, which ambience beds play where, thunder, and which voice gives way
//! when too many play. `pbd-app` turns these into kira calls; nothing here
//! knows kira exists.
//!
//! Distances are metres, times seconds, gains linear amplitude unless a name
//! says `_db`.

use crate::terrain::Material;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Which bus a cue plays on (design section 2).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Bus {
    /// Placed effects, under `world`.
    #[default]
    Effects,
    /// Menu and HUD sounds, never filtered.
    Menus,
}

/// One cue: a sound the code names, with its takes and how it is heard.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Cue {
    /// How many files: one is `<name>.ogg`, more are `<name>_1.ogg` ..
    pub takes: u8,
    /// Level in decibels, the mix between cues.
    pub volume_db: f32,
    /// Playback rate range, picked uniformly per play.
    pub pitch: (f32, f32),
    /// At most this many of this cue at once.
    pub cap: u8,
    /// Full level within this distance, then 6 dB a doubling.
    pub full_within_m: f32,
    /// Silent past this distance.
    pub audible_m: f32,
    /// A loop, kept alive while asked for.
    pub looped: bool,
    pub bus: Bus,
    /// Streamed from disk when played rather than decoded at load: the long
    /// takes (thunder, the long loops), which would put the decoded bank
    /// past its 64 MB budget (design section 8).
    pub stream: bool,
}

impl Default for Cue {
    fn default() -> Self {
        Self {
            takes: 1,
            volume_db: 0.0,
            pitch: (1.0, 1.0),
            cap: 4,
            full_within_m: 3.0,
            audible_m: 60.0,
            looped: false,
            bus: Bus::Effects,
            stream: false,
        }
    }
}

impl Cue {
    /// The cue's files, relative to its folder.
    pub fn files(&self, name: &str) -> Vec<String> {
        if self.takes <= 1 {
            vec![format!("{name}.ogg")]
        } else {
            (1..=self.takes)
                .map(|take| format!("{name}_{take}.ogg"))
                .collect()
        }
    }

    /// Why this cue cannot be used, if it cannot.
    pub fn problem(&self) -> Option<String> {
        if self.takes == 0 {
            return Some("no takes".into());
        }
        if !(self.full_within_m > 0.0 && self.full_within_m.is_finite()) {
            return Some(format!(
                "full_within_m {} is not positive",
                self.full_within_m
            ));
        }
        if !(self.audible_m > self.full_within_m && self.audible_m.is_finite()) {
            return Some(format!(
                "audible_m {} is not past full_within_m {}",
                self.audible_m, self.full_within_m
            ));
        }
        if !(self.pitch.0 > 0.0 && self.pitch.1 >= self.pitch.0) {
            return Some(format!(
                "pitch range {:?} is not rising and positive",
                self.pitch
            ));
        }
        if self.cap == 0 {
            return Some("cap is 0".into());
        }
        if !self.volume_db.is_finite() {
            return Some("volume_db is not finite".into());
        }
        None
    }
}

/// `assets/config/audio/cues.ron`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CueBook {
    pub cues: BTreeMap<String, Cue>,
}

impl CueBook {
    /// Every problem in the book, one line each; empty when it is good.
    pub fn problems(&self) -> Vec<String> {
        self.cues
            .iter()
            .filter_map(|(name, cue)| cue.problem().map(|why| format!("{name}: {why}")))
            .collect()
    }
}

/// A bed's own level (`assets/config/audio/ambience.ron`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BedLevel {
    pub volume_db: f32,
}

impl Default for BedLevel {
    fn default() -> Self {
        Self { volume_db: -6.0 }
    }
}

/// `assets/config/audio/ambience.ron`: each bed's level, by its file stem.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Ambience {
    pub beds: BTreeMap<String, BedLevel>,
}

/// `assets/config/audio/mix.ron` (design sections 3, 5 and 6).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MixConfig {
    pub speed_of_sound_m_s: f32,
    /// A placed sound this near starts at once: its delay is under a frame.
    pub no_delay_within_m: f32,
    /// The air's low pass: `20 kHz * ref / (ref + d)`.
    pub air_ref_m: f32,
    pub air_floor_hz: f32,
    /// The cutoff's factor for a sound straight behind.
    pub behind_factor: f32,
    /// Full air below this height above the surface, none at `air_top_m`.
    pub air_full_m: f32,
    pub air_top_m: f32,
    pub max_voices: usize,
    pub underwater_cutoff_hz: f32,
    pub underwater_db: f32,
    pub shelter_cutoff_hz: f32,
    pub shelter_db: f32,
    /// Underground depth where the outdoor beds start to fade, and where they
    /// are gone and the cave bed is full.
    pub cave_from_m: f32,
    pub cave_full_m: f32,
    /// The reverb send from `world`, as a mix 0..1.
    pub outdoor_reverb: f32,
    pub cave_reverb: f32,
    pub underwater_reverb: f32,
    /// A bed's ease toward its target, seconds.
    pub bed_ease_s: f32,
    /// A loop no longer asked for fades out over this.
    pub loop_fade_s: f32,
    pub thunder_reach_m: f32,
    pub thunder_full_within_m: f32,
    /// A far strike's roll waits this long after the last thunder; the
    /// nearest new strike of a step speaks for it (build decision 11).
    pub thunder_gap_s: f32,
    /// Wetness over which the wet layer goes under a step.
    pub wet_step_from: f32,
    /// The shortest stride: a slow walk steps every this many metres.
    pub stride_walk_m: f32,
    /// Steps a second at the walker's walking speed and at its sprint: the
    /// walker walks at 8 m/s and sprints at 14, where a 0.8 m stride would
    /// be ten steps a second (build decision 10).
    pub step_rate_walk_hz: f32,
    pub step_rate_run_hz: f32,
    /// Unused: a landing is an ordinary step (owner, 2026-11-15, no
    /// thump). Kept so a `mix.ron` that names it still reads.
    pub land_hard_m_s: f32,
    /// A fall slower than this makes no landing sound.
    pub land_from_m_s: f32,
    /// Into the water faster than this is `water_enter_big`.
    pub enter_big_m_s: f32,
    /// Under for longer than this, surfacing plays `breath`.
    pub breath_after_s: f32,
    /// Fires (campfire, brazier) voiced within this distance.
    pub fire_reach_m: f32,
}

impl Default for MixConfig {
    fn default() -> Self {
        Self {
            speed_of_sound_m_s: 343.0,
            no_delay_within_m: 10.0,
            air_ref_m: 250.0,
            air_floor_hz: 300.0,
            behind_factor: 0.6,
            air_full_m: 600.0,
            air_top_m: 960.0,
            max_voices: 40,
            underwater_cutoff_hz: 600.0,
            underwater_db: -9.0,
            shelter_cutoff_hz: 2500.0,
            shelter_db: -6.0,
            cave_from_m: 2.0,
            cave_full_m: 12.0,
            outdoor_reverb: 0.05,
            cave_reverb: 0.35,
            underwater_reverb: 0.25,
            bed_ease_s: 3.0,
            loop_fade_s: 0.15,
            thunder_reach_m: 6000.0,
            thunder_full_within_m: 150.0,
            thunder_gap_s: 4.0,
            wet_step_from: 0.4,
            stride_walk_m: 0.8,
            step_rate_walk_hz: 2.2,
            step_rate_run_hz: 2.8,
            land_hard_m_s: 9.0,
            land_from_m_s: 2.5,
            enter_big_m_s: 6.0,
            breath_after_s: 10.0,
            fire_reach_m: 30.0,
        }
    }
}

impl MixConfig {
    /// Why this mix cannot be used, if it cannot.
    pub fn problems(&self) -> Vec<String> {
        let mut out = Vec::new();
        let positive = [
            ("speed_of_sound_m_s", self.speed_of_sound_m_s),
            ("air_ref_m", self.air_ref_m),
            ("air_floor_hz", self.air_floor_hz),
            ("underwater_cutoff_hz", self.underwater_cutoff_hz),
            ("shelter_cutoff_hz", self.shelter_cutoff_hz),
            ("bed_ease_s", self.bed_ease_s),
            ("loop_fade_s", self.loop_fade_s),
            ("thunder_reach_m", self.thunder_reach_m),
            ("thunder_full_within_m", self.thunder_full_within_m),
            ("stride_walk_m", self.stride_walk_m),
            ("step_rate_walk_hz", self.step_rate_walk_hz),
            ("step_rate_run_hz", self.step_rate_run_hz),
        ];
        for (name, value) in positive {
            if !(value > 0.0 && value.is_finite()) {
                out.push(format!("{name} {value} is not positive"));
            }
        }
        let not_negative = [
            ("no_delay_within_m", self.no_delay_within_m),
            ("air_full_m", self.air_full_m),
            ("cave_from_m", self.cave_from_m),
            ("fire_reach_m", self.fire_reach_m),
        ];
        for (name, value) in not_negative {
            if !(value >= 0.0 && value.is_finite()) {
                out.push(format!("{name} {value} is negative"));
            }
        }
        if self.air_top_m <= self.air_full_m {
            out.push("air_top_m is not above air_full_m".into());
        }
        if self.cave_full_m <= self.cave_from_m {
            out.push("cave_full_m is not below cave_from_m".into());
        }
        if !(0.0..=1.0).contains(&self.behind_factor) {
            out.push("behind_factor is not 0..1".into());
        }
        if self.max_voices == 0 {
            out.push("max_voices is 0".into());
        }
        out
    }
}

/// Decibels to linear amplitude; `-inf` and anything under -100 dB is 0.
pub fn amplitude(db: f32) -> f32 {
    if db <= -100.0 {
        0.0
    } else {
        10f32.powf(db / 20.0)
    }
}

/// Linear amplitude to decibels, floored at -100.
pub fn decibels(amplitude: f32) -> f32 {
    if amplitude <= 1e-5 {
        -100.0
    } else {
        20.0 * amplitude.log10()
    }
}

/// How loud a placed sound is at `d` metres: full within `full_within_m`,
/// then `full / d` (6 dB a doubling), faded to nothing over the last fifth
/// before `audible_m` so a walk away never cuts a sound off.
pub fn distance_gain(d: f32, full_within_m: f32, audible_m: f32) -> f32 {
    let d = d.max(0.0);
    if d >= audible_m {
        return 0.0;
    }
    let law = if d <= full_within_m {
        1.0
    } else {
        full_within_m / d
    };
    let fade_from = audible_m * 0.8;
    let fade = if d > fade_from {
        1.0 - (d - fade_from) / (audible_m - fade_from)
    } else {
        1.0
    };
    law * fade
}

/// The air's low pass at `d` metres, darker by up to `behind_factor` for a
/// sound straight behind (`behind` 0 in front or beside, 1 straight behind).
pub fn air_cutoff_hz(mix: &MixConfig, d: f32, behind: f32) -> f32 {
    let open = 20_000.0 * mix.air_ref_m / (mix.air_ref_m + d.max(0.0));
    let turned = open * (1.0 + (mix.behind_factor - 1.0) * behind.clamp(0.0, 1.0));
    turned.clamp(mix.air_floor_hz, 20_000.0)
}

/// How long a sound takes to arrive from `d` metres; none within
/// `no_delay_within_m`.
pub fn delay_s(mix: &MixConfig, d: f32) -> f32 {
    if d < mix.no_delay_within_m {
        0.0
    } else {
        d / mix.speed_of_sound_m_s
    }
}

/// How much air carries sound at `height_m` above the surface: 1 below
/// `air_full_m`, nothing at the top of the atmosphere and above.
pub fn air_density_scale(mix: &MixConfig, height_m: f32) -> f32 {
    if height_m <= mix.air_full_m {
        1.0
    } else if height_m >= mix.air_top_m {
        0.0
    } else {
        1.0 - (height_m - mix.air_full_m) / (mix.air_top_m - mix.air_full_m)
    }
}

/// Stereo placement of a sound at `local`, the emitter in the listener's own
/// axes (x right, y up, -z ahead, Bevy's camera convention): the pan -1 (left)
/// to 1 (right), and how far behind it is, 0..1. A sound at the ear is
/// centred.
pub fn pan(local: Vec3) -> (f32, f32) {
    let flat = Vec3::new(local.x, 0.0, local.z);
    let length = (flat.length_squared() + 0.25).sqrt();
    let pan = (local.x / length).clamp(-1.0, 1.0) * 0.9;
    let behind = (local.z / local.length().max(0.5)).clamp(0.0, 1.0);
    (pan, behind)
}

/// What a footstep sounds like (design section 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Family {
    Stone,
    Dirt,
    Grass,
    Dry,
    Sand,
    Snow,
    Wood,
    Shallow,
}

impl Family {
    pub const ALL: [Family; 8] = [
        Family::Stone,
        Family::Dirt,
        Family::Grass,
        Family::Dry,
        Family::Sand,
        Family::Snow,
        Family::Wood,
        Family::Shallow,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Family::Stone => "stone",
            Family::Dirt => "dirt",
            Family::Grass => "grass",
            Family::Dry => "dry",
            Family::Sand => "sand",
            Family::Snow => "snow",
            Family::Wood => "wood",
            Family::Shallow => "shallow",
        }
    }

    /// Whether wet ground lays `step_wet` under this family's step.
    pub fn takes_wet(self) -> bool {
        matches!(self, Family::Stone | Family::Dirt | Family::Grass)
    }

    /// Whether this family has dig, break and place cues (water does not).
    pub fn diggable(self) -> bool {
        self != Family::Shallow
    }
}

/// The footstep family of a block, or none for what is not stood on (air, a
/// lamp, a fire).
pub fn surface_family(material: Material) -> Option<Family> {
    Some(match material {
        Material::Stone | Material::Rock | Material::Ore | Material::Forge => Family::Stone,
        Material::Soil | Material::Dirt | Material::Tilled => Family::Dirt,
        Material::Grass | Material::JungleGrass => Family::Grass,
        Material::DryGrass => Family::Dry,
        Material::Sand => Family::Sand,
        Material::Snow => Family::Snow,
        Material::Workbench => Family::Wood,
        Material::Water | crate::flowing_water!() => Family::Shallow,
        Material::Air
        | Material::Torch
        | Material::LanternPost
        | Material::LanternWall
        | Material::LanternHanging
        | Material::Brazier
        | Material::Candle
        | Material::Campfire => return None,
    })
}

/// The cue a block makes when placed: its family's, or the lamp's or fire's.
pub fn place_cue(material: Material) -> Option<String> {
    match material {
        Material::Campfire | Material::Brazier => Some("fire_ignite_small".into()),
        m if m.is_lamp() => Some("place_lamp".into()),
        m => surface_family(m)
            .filter(|f| f.diggable())
            .map(|f| format!("place_{}", f.name())),
    }
}

/// The stride at `speed` m/s for a walker who walks at `walk_m_s` and
/// sprints at `run_m_s`: the step rate runs from the walking cadence to the
/// sprinting one, and the stride is speed over rate, never under
/// `stride_walk_m`.
pub fn stride_m(mix: &MixConfig, speed: f32, walk_m_s: f32, run_m_s: f32) -> f32 {
    let t = ((speed - walk_m_s) / (run_m_s - walk_m_s).max(1e-3)).clamp(0.0, 1.0);
    let rate = mix.step_rate_walk_hz + (mix.step_rate_run_hz - mix.step_rate_walk_hz) * t;
    (speed / rate.max(1e-3)).max(mix.stride_walk_m)
}

/// Counts strides along the ground: a step every `stride_m` walked.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stride {
    walked_m: f32,
}

impl Stride {
    /// Walk `moved_m` with strides of `stride_m`: how many steps fell in it.
    /// The first step comes half a stride in, so a walker who sets off is
    /// heard at once.
    pub fn advance(&mut self, moved_m: f32, stride_m: f32) -> u32 {
        if moved_m.is_nan() || stride_m.is_nan() || moved_m <= 0.0 || stride_m <= 0.0 {
            return 0;
        }
        let before = self.walked_m;
        self.walked_m += moved_m;
        let at = |w: f32| ((w + stride_m * 0.5) / stride_m).floor() as i64;
        let steps = (at(self.walked_m) - at(before)).max(0) as u32;
        // Keep the number small; whole strides only, so the phase stays.
        if self.walked_m > stride_m * 1000.0 {
            self.walked_m -= stride_m * 1000.0;
        }
        steps
    }

    /// Stopped or left the ground: the next step is half a stride away.
    pub fn reset(&mut self) {
        self.walked_m = 0.0;
    }
}

/// The ambience beds (design section 5), by file stem.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Bed {
    WindLow,
    WindHigh,
    RainLight,
    RainHeavy,
    RainRoof,
    Surf,
    OpenWater,
    MeadowDay,
    JungleDay,
    SwampDay,
    Gulls,
    DesertDay,
    NightCrickets,
    NightFrogs,
    TownDay,
    TownNight,
    Cave,
    Underwater,
}

impl Bed {
    pub const COUNT: usize = 18;
    pub const ALL: [Bed; Bed::COUNT] = [
        Bed::WindLow,
        Bed::WindHigh,
        Bed::RainLight,
        Bed::RainHeavy,
        Bed::RainRoof,
        Bed::Surf,
        Bed::OpenWater,
        Bed::MeadowDay,
        Bed::JungleDay,
        Bed::SwampDay,
        Bed::Gulls,
        Bed::DesertDay,
        Bed::NightCrickets,
        Bed::NightFrogs,
        Bed::TownDay,
        Bed::TownNight,
        Bed::Cave,
        Bed::Underwater,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Bed::WindLow => "amb_wind_low",
            Bed::WindHigh => "amb_wind_high",
            Bed::RainLight => "amb_rain_light",
            Bed::RainHeavy => "amb_rain_heavy",
            Bed::RainRoof => "amb_rain_roof",
            Bed::Surf => "amb_surf",
            Bed::OpenWater => "amb_open_water",
            Bed::MeadowDay => "amb_meadow_day",
            Bed::JungleDay => "amb_jungle_day",
            Bed::SwampDay => "amb_swamp_day",
            Bed::Gulls => "amb_gulls",
            Bed::DesertDay => "amb_desert_day",
            Bed::NightCrickets => "amb_night_crickets",
            Bed::NightFrogs => "amb_night_frogs",
            Bed::TownDay => "amb_town_day",
            Bed::TownNight => "amb_town_night",
            Bed::Cave => "amb_cave",
            Bed::Underwater => "amb_underwater",
        }
    }

    pub fn index(self) -> usize {
        Bed::ALL.iter().position(|b| *b == self).expect("every bed")
    }
}

/// The biomes as the beds read them, in `planet_gen::Biome`'s order.
pub const BIOMES: usize = 8;
pub const OCEAN: usize = 0;
pub const BEACH: usize = 1;
pub const FIELDS: usize = 2;
pub const DESERT: usize = 3;
pub const JUNGLE: usize = 4;
pub const SWAMP: usize = 5;

/// Where the listener is, as the beds read it (design section 5).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Surroundings {
    pub height_m: f32,
    pub airspeed_m_s: f32,
    /// Liquid rain 0..1 (none while snowing).
    pub rain: f32,
    pub snowing: bool,
    pub sheltered: bool,
    pub underground_m: f32,
    pub underwater: bool,
    /// Biome weights summing to 1, by `planet_gen::Biome` order.
    pub biomes: [f32; BIOMES],
    pub sun_elevation_deg: f32,
    /// Distance to the nearest shore within 200 m, if any.
    pub shore_m: Option<f32>,
    pub open_water: bool,
    /// Distance to the nearest town's edge (0 inside), if within reach.
    pub town_m: Option<f32>,
    /// `air_density_scale` at the listener.
    pub air: f32,
}

impl Default for Surroundings {
    fn default() -> Self {
        let mut biomes = [0.0; BIOMES];
        biomes[FIELDS] = 1.0;
        Self {
            height_m: 1.7,
            airspeed_m_s: 0.0,
            rain: 0.0,
            snowing: false,
            sheltered: false,
            underground_m: 0.0,
            underwater: false,
            biomes,
            sun_elevation_deg: 45.0,
            shore_m: None,
            open_water: false,
            town_m: None,
            air: 1.0,
        }
    }
}

fn smoothstep(from: f32, to: f32, x: f32) -> f32 {
    let t = ((x - from) / (to - from)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// What the beds should be: each one's POWER weight (amplitude is its square
/// root, so a crossfade between two beds whose weights sum to one keeps
/// loudness), and the wind's playback rate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BedMix {
    pub weights: [f32; Bed::COUNT],
    pub wind_rate: f32,
}

impl BedMix {
    pub fn weight(&self, bed: Bed) -> f32 {
        self.weights[bed.index()]
    }

    /// A bed's level in decibels above its own `volume_db`, the equal-power
    /// rule `10 log10(weight)`.
    pub fn db(&self, bed: Bed) -> f32 {
        let w = self.weight(bed);
        if w <= 1e-4 { -100.0 } else { 10.0 * w.log10() }
    }
}

/// The beds' weights where the listener is (design section 5).
pub fn bed_weights(mix: &MixConfig, s: &Surroundings) -> BedMix {
    let mut w = [0.0f32; Bed::COUNT];
    let mut set = |bed: Bed, value: f32| w[bed.index()] = value.clamp(0.0, 1.0);
    if s.underwater {
        set(Bed::Underwater, 1.0);
        return BedMix {
            weights: w,
            wind_rate: 1.0,
        };
    }
    let cave = smoothstep(mix.cave_from_m, mix.cave_full_m, s.underground_m);
    let outdoor = (1.0 - cave) * s.air.clamp(0.0, 1.0);
    let speed = (s.airspeed_m_s / 30.0).clamp(0.0, 1.0);

    // Wind: never zero below the air's top. Low to high by height, louder
    // with airspeed; snow is the wind alone, 3 dB down.
    let high = smoothstep(5.0, 80.0, s.height_m);
    let wind = outdoor * (0.45 + 0.55 * speed) * if s.snowing { 0.5 } else { 1.0 };
    set(Bed::WindLow, wind * (1.0 - high));
    set(Bed::WindHigh, wind * high);

    // Rain: light to heavy by intensity, on a roof when sheltered.
    let rain = if s.snowing {
        0.0
    } else {
        s.rain.clamp(0.0, 1.0)
    };
    let level = outdoor * (rain * 2.5).min(1.0);
    if s.sheltered {
        set(Bed::RainRoof, level);
    } else {
        let heavy = smoothstep(0.3, 0.8, rain);
        set(Bed::RainLight, level * (1.0 - heavy));
        set(Bed::RainHeavy, level * heavy);
    }

    // Shore and open water.
    let low = 1.0 - smoothstep(20.0, 40.0, s.height_m);
    if let Some(shore) = s.shore_m {
        let near = (1.0 - shore / 200.0).clamp(0.0, 1.0);
        set(Bed::Surf, outdoor * low * near * (0.8 + 0.2 * speed));
    }
    if s.open_water {
        set(Bed::OpenWater, outdoor * low);
    }

    // Life: by biome and the sun, less in rain, gone high up.
    let day = smoothstep(-6.0, 6.0, s.sun_elevation_deg);
    let life = outdoor * (1.0 - rain) * (1.0 - smoothstep(40.0, 60.0, s.height_m));
    let b = &s.biomes;
    set(Bed::MeadowDay, life * day * b[FIELDS]);
    set(Bed::JungleDay, life * day * b[JUNGLE]);
    set(Bed::SwampDay, life * day * b[SWAMP]);
    set(Bed::Gulls, life * day * (b[BEACH] + b[OCEAN]));
    set(Bed::DesertDay, life * day * b[DESERT]);
    // Night is 4 dB under day (0.4 in power), never silent where it plays.
    let night = (1.0 - day) * 0.4;
    set(Bed::NightCrickets, life * night * (b[FIELDS] + b[JUNGLE]));
    set(Bed::NightFrogs, life * night * b[SWAMP]);

    // Towns, by distance to the edge within 150 m.
    if let Some(town) = s.town_m {
        let near = (1.0 - town / 150.0).clamp(0.0, 1.0) * outdoor;
        set(Bed::TownDay, near * day);
        set(Bed::TownNight, near * (1.0 - day));
    }

    set(Bed::Cave, cave);
    BedMix {
        weights: w,
        wind_rate: 0.9 + 0.25 * speed,
    }
}

/// Which thunder take a distance gets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThunderTake {
    Crack,
    Close,
    Roll,
}

impl ThunderTake {
    pub fn cue(self) -> &'static str {
        match self {
            ThunderTake::Crack => "thunder_crack",
            ThunderTake::Close => "thunder_close",
            ThunderTake::Roll => "thunder_roll",
        }
    }
}

/// How a strike is heard from the eye (design section 6).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Thunder {
    /// From the flash to the first sound.
    pub delay_s: f32,
    /// How long the roll lasts.
    pub roll_s: f32,
    pub take: ThunderTake,
    /// The nearest point of the bolt.
    pub near_m: f32,
    /// Linear amplitude, strength included.
    pub gain: f32,
    pub cutoff_hz: f32,
}

/// The thunder of a strike at the unit `direction`, heard from the body-local
/// `eye`: the bolt runs from `cloud_radius` down to `ground_radius` over the
/// strike. None past `thunder_reach_m` along the ground.
pub fn thunder(
    mix: &MixConfig,
    eye: Vec3,
    direction: Vec3,
    ground_radius: f32,
    cloud_radius: f32,
    strength: f32,
) -> Option<Thunder> {
    let direction = direction.try_normalize()?;
    let up = eye.try_normalize()?;
    let along = up.dot(direction).clamp(-1.0, 1.0).acos() * ground_radius;
    if along > mix.thunder_reach_m {
        return None;
    }
    let bottom = direction * ground_radius;
    let top = direction * cloud_radius;
    let bolt = top - bottom;
    let t = ((eye - bottom).dot(bolt) / bolt.length_squared().max(1e-6)).clamp(0.0, 1.0);
    let near = (bottom + bolt * t).distance(eye);
    let far = eye.distance(bottom).max(eye.distance(top));
    let take = if near < 400.0 {
        ThunderTake::Crack
    } else if near < 1500.0 {
        ThunderTake::Close
    } else {
        ThunderTake::Roll
    };
    let boost = amplitude(6.0 * strength.clamp(0.0, 1.0));
    Some(Thunder {
        delay_s: near / mix.speed_of_sound_m_s,
        roll_s: ((far - near) / mix.speed_of_sound_m_s).max(2.0),
        take,
        near_m: near,
        gain: distance_gain(near, mix.thunder_full_within_m, mix.thunder_reach_m * 1.5) * boost,
        cutoff_hz: air_cutoff_hz(mix, near, 0.0),
    })
}

/// What happens to a new voice when a cue or the mixer is at its cap.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Admit {
    Play,
    /// Play, stopping the playing voice at this index first.
    Replace(usize),
    Drop,
}

/// The voice cap's choice (design section 3): under the cap a new voice
/// plays; at it, the quietest playing voice gives way to a louder new one,
/// and a new one no louder than every playing voice is dropped.
pub fn admit(playing: &[f32], cap: usize, gain: f32) -> Admit {
    if playing.len() < cap {
        return Admit::Play;
    }
    match playing.iter().enumerate().min_by(|a, b| a.1.total_cmp(b.1)) {
        Some((index, quietest)) if gain > *quietest => Admit::Replace(index),
        _ => Admit::Drop,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const R: f32 = 4800.0;

    fn mix() -> MixConfig {
        MixConfig::default()
    }

    #[test]
    fn a_cue_with_no_takes_or_a_negative_distance_is_refused() {
        let good = Cue::default();
        assert_eq!(good.problem(), None);
        let none = Cue {
            takes: 0,
            ..Cue::default()
        };
        assert!(none.problem().is_some());
        let negative = Cue {
            full_within_m: -1.0,
            ..Cue::default()
        };
        assert!(negative.problem().is_some());
        let inverted = Cue {
            audible_m: 2.0,
            full_within_m: 3.0,
            ..Cue::default()
        };
        assert!(inverted.problem().is_some());
        assert!(mix().problems().is_empty());
        let bad = MixConfig {
            speed_of_sound_m_s: 0.0,
            ..mix()
        };
        assert_eq!(bad.problems().len(), 1);
    }

    #[test]
    fn takes_name_their_files() {
        let one = Cue::default();
        assert_eq!(one.files("pickup"), ["pickup.ogg"]);
        let six = Cue {
            takes: 3,
            ..Cue::default()
        };
        assert_eq!(
            six.files("dig_dirt"),
            ["dig_dirt_1.ogg", "dig_dirt_2.ogg", "dig_dirt_3.ogg"]
        );
    }

    #[test]
    fn six_decibels_a_doubling() {
        let at = |d| distance_gain(d, 3.0, 100.0);
        assert_eq!(at(1.0), 1.0);
        assert_eq!(at(3.0), 1.0);
        let drop = decibels(at(6.0)) - decibels(at(12.0));
        assert!((drop - 6.02).abs() < 0.05, "{drop}");
        assert_eq!(at(100.0), 0.0);
        assert_eq!(at(150.0), 0.0);
        // Faded, not cut, near the edge.
        assert!(at(90.0) < at(79.0) * 79.0 / 90.0);
        assert!(at(99.0) > 0.0);
    }

    #[test]
    fn the_air_darkens_far_and_behind() {
        let m = mix();
        assert!(air_cutoff_hz(&m, 1.0, 0.0) > 19_000.0);
        let far = air_cutoff_hz(&m, 3000.0, 0.0);
        assert!((1_000.0..2_000.0).contains(&far), "{far}");
        assert!(air_cutoff_hz(&m, 1.0, 1.0) < air_cutoff_hz(&m, 1.0, 0.0) * 0.61);
        assert_eq!(air_cutoff_hz(&m, 1e9, 1.0), m.air_floor_hz);
    }

    #[test]
    fn no_delay_within_ten_metres() {
        let m = mix();
        assert_eq!(delay_s(&m, 9.9), 0.0);
        assert!((delay_s(&m, 343.0) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn silent_above_the_atmospheres_top() {
        let m = mix();
        assert_eq!(air_density_scale(&m, 0.0), 1.0);
        assert_eq!(air_density_scale(&m, 600.0), 1.0);
        assert!((air_density_scale(&m, 780.0) - 0.5).abs() < 1e-6);
        assert_eq!(air_density_scale(&m, 960.0), 0.0);
        assert_eq!(air_density_scale(&m, 5000.0), 0.0);
    }

    #[test]
    fn pan_follows_the_side_and_behind() {
        let (right, front) = pan(Vec3::new(5.0, 0.0, 0.0));
        assert!(right > 0.8 && front < 0.01);
        let (left, _) = pan(Vec3::new(-5.0, 0.0, 0.0));
        assert!((left + right).abs() < 1e-6);
        let (centre, behind) = pan(Vec3::new(0.0, 0.0, 5.0));
        assert!(centre.abs() < 1e-6 && behind > 0.99);
        let (ahead, none) = pan(Vec3::new(0.0, 0.0, -5.0));
        assert!(ahead.abs() < 1e-6 && none == 0.0);
        let (at_ear, _) = pan(Vec3::ZERO);
        assert_eq!(at_ear, 0.0);
    }

    #[test]
    fn every_material_a_walker_stands_on_has_a_family() {
        for material in Material::ALL {
            let stood_on = !matches!(material, Material::Air) && !material.is_lamp();
            assert_eq!(surface_family(material).is_some(), stood_on, "{material:?}");
        }
        assert_eq!(surface_family(Material::Sand), Some(Family::Sand));
        assert_eq!(surface_family(Material::Stone), Some(Family::Stone));
        assert_eq!(place_cue(Material::Torch).as_deref(), Some("place_lamp"));
        assert_eq!(
            place_cue(Material::Campfire).as_deref(),
            Some("fire_ignite_small")
        );
        assert_eq!(place_cue(Material::Water), None);
    }

    #[test]
    fn eight_metres_at_walking_speed_is_ten_steps() {
        let m = mix();
        let mut stride = Stride::default();
        let mut steps = 0;
        // 8 m in 1/60 s frames at 1.4 m/s.
        let frame = 1.4 / 60.0;
        let mut walked = 0.0;
        while walked < 8.0 - 1e-4 {
            steps += stride.advance(frame, m.stride_walk_m);
            walked += frame;
        }
        assert_eq!(steps, 10);
        // At the walker's own speeds, the cadence holds.
        let walk = stride_m(&m, 8.0, 8.0, 14.0);
        assert!((8.0 / walk - 2.2).abs() < 1e-4);
        let run = stride_m(&m, 14.0, 8.0, 14.0);
        assert!((14.0 / run - 2.8).abs() < 1e-4);
        assert_eq!(stride_m(&m, 1.0, 8.0, 14.0), m.stride_walk_m);
        // The first step comes at once.
        let mut fresh = Stride::default();
        assert_eq!(fresh.advance(0.41, 0.8), 1);
    }

    fn at(s: Surroundings) -> BedMix {
        bed_weights(&mix(), &s)
    }

    #[test]
    fn a_meadow_at_noon_and_at_midnight() {
        let noon = at(Surroundings::default());
        assert!(noon.weight(Bed::MeadowDay) > 0.9);
        assert!(noon.weight(Bed::WindLow) > 0.3);
        assert_eq!(noon.weight(Bed::NightCrickets), 0.0);
        let midnight = at(Surroundings {
            sun_elevation_deg: -40.0,
            ..Default::default()
        });
        assert_eq!(midnight.weight(Bed::MeadowDay), 0.0);
        assert!(midnight.weight(Bed::NightCrickets) > 0.3);
        assert!(midnight.weight(Bed::WindLow) > 0.3);
    }

    #[test]
    fn three_hundred_metres_up_is_high_wind_alone() {
        let high = at(Surroundings {
            height_m: 300.0,
            ..Default::default()
        });
        assert!(high.weight(Bed::WindHigh) > 0.4);
        for bed in Bed::ALL {
            if bed != Bed::WindHigh {
                assert_eq!(high.weight(bed), 0.0, "{bed:?}");
            }
        }
    }

    #[test]
    fn a_cave_ten_metres_down_in_rain() {
        let cave = at(Surroundings {
            underground_m: 10.0,
            rain: 1.0,
            sheltered: true,
            ..Default::default()
        });
        assert!(cave.weight(Bed::Cave) > 0.85);
        assert!(cave.weight(Bed::RainRoof) < 0.15);
        assert_eq!(cave.weight(Bed::RainHeavy), 0.0);
        let deep = at(Surroundings {
            underground_m: 15.0,
            rain: 1.0,
            sheltered: true,
            ..Default::default()
        });
        assert_eq!(deep.weight(Bed::RainRoof), 0.0);
        assert_eq!(deep.weight(Bed::WindLow), 0.0);
    }

    #[test]
    fn under_water_is_one_bed() {
        let under = at(Surroundings {
            underwater: true,
            rain: 1.0,
            ..Default::default()
        });
        for bed in Bed::ALL {
            let expect = if bed == Bed::Underwater { 1.0 } else { 0.0 };
            assert_eq!(under.weight(bed), expect, "{bed:?}");
        }
    }

    #[test]
    fn a_beach_and_a_town() {
        let mut biomes = [0.0; BIOMES];
        biomes[BEACH] = 1.0;
        let beach = at(Surroundings {
            biomes,
            shore_m: Some(10.0),
            ..Default::default()
        });
        assert!(beach.weight(Bed::Surf) > 0.7);
        assert!(beach.weight(Bed::Gulls) > 0.9);
        let town = at(Surroundings {
            town_m: Some(0.0),
            ..Default::default()
        });
        assert!(town.weight(Bed::TownDay) > 0.9);
        assert_eq!(town.weight(Bed::TownNight), 0.0);
    }

    #[test]
    fn a_crossfade_keeps_its_power() {
        for h in [0.0, 5.0, 20.0, 40.0, 60.0, 80.0, 200.0] {
            let m = at(Surroundings {
                height_m: h,
                ..Default::default()
            });
            let sum = m.weight(Bed::WindLow) + m.weight(Bed::WindHigh);
            assert!((sum - 0.45).abs() < 1e-5, "{h}: {sum}");
        }
        for rain in [0.4, 0.5, 0.6, 0.7, 1.0] {
            let m = at(Surroundings {
                rain,
                ..Default::default()
            });
            let sum = m.weight(Bed::RainLight) + m.weight(Bed::RainHeavy);
            assert!((sum - 1.0).abs() < 1e-5, "{rain}: {sum}");
        }
        let mut biomes = [0.0; BIOMES];
        biomes[FIELDS] = 0.5;
        biomes[JUNGLE] = 0.5;
        let border = at(Surroundings {
            biomes,
            ..Default::default()
        });
        let sum = border.weight(Bed::MeadowDay) + border.weight(Bed::JungleDay);
        assert!((sum - 1.0).abs() < 1e-5);
    }

    fn on_ground(along_m: f32) -> (Vec3, Vec3) {
        let eye = Vec3::Y * (R + 1.7);
        let angle = along_m / R;
        let strike = Vec3::new(angle.sin(), angle.cos(), 0.0);
        (eye, strike)
    }

    #[test]
    fn thunder_overhead_one_and_three_kilometres_and_past_reach() {
        let m = mix();
        let (eye, strike) = on_ground(0.0);
        let over = thunder(&m, eye, strike, R, R + 300.0, 0.5).unwrap();
        assert!(over.delay_s < 0.02, "{}", over.delay_s);
        assert_eq!(over.take, ThunderTake::Crack);
        assert!(over.roll_s >= 2.0);

        let (eye, strike) = on_ground(1000.0);
        let one = thunder(&m, eye, strike, R, R + 300.0, 0.5).unwrap();
        assert!(
            (one.delay_s - 1000.0 / 343.0).abs() < 0.05,
            "{}",
            one.delay_s
        );
        assert_eq!(one.take, ThunderTake::Close);

        let (eye, strike) = on_ground(3000.0);
        let three = thunder(&m, eye, strike, R, R + 300.0, 0.5).unwrap();
        assert!((three.delay_s - 8.7).abs() < 0.1, "{}", three.delay_s);
        assert_eq!(three.take, ThunderTake::Roll);
        assert!(three.gain < one.gain && one.gain < over.gain);
        assert!(three.cutoff_hz < one.cutoff_hz);

        let (eye, strike) = on_ground(6500.0);
        assert!(thunder(&m, eye, strike, R, R + 300.0, 0.5).is_none());
    }

    #[test]
    fn the_quietest_voice_gives_way() {
        assert_eq!(admit(&[0.5, 0.2], 3, 0.1), Admit::Play);
        assert_eq!(admit(&[0.5, 0.2, 0.3], 3, 0.25), Admit::Replace(1));
        assert_eq!(admit(&[0.5, 0.2, 0.3], 3, 0.2), Admit::Drop);
        assert_eq!(admit(&[], 0, 1.0), Admit::Drop);
    }
}
