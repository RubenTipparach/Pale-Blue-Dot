//! `assets/config/fire.ron`: every number fire runs on (design 11), parsed
//! and validated. [`FireConfig::shipped`] is the one source of defaults.

use serde::{Deserialize, Serialize};

/// The shipped file, built in.
pub const SHIPPED_RON: &str = include_str!("../../../../assets/config/fire.ron");

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GroundWetRates {
    pub per_mm: f32,
    pub dry_per_hour: f32,
    pub sun_ref_wm2: f32,
    pub dry_from_c: f32,
    pub dry_span_c: f32,
}

impl Default for GroundWetRates {
    /// The shipped file's.
    fn default() -> Self {
        FireConfig::shipped().ground_wet
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WeatherTerms {
    pub humid_dry: f32,
    pub humid_wet: f32,
    pub humid_keep: f32,
    pub warm_from_c: f32,
    pub warm_full_c: f32,
    pub warm_floor: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeasonTerms {
    pub dry_peak: f32,
    pub dry_half: f32,
    pub straw_share: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cure {
    pub base: f32,
    pub per_cure: f32,
    #[serde(default)]
    pub per_litter: f32,
}

impl Cure {
    pub fn of(&self, c: f32, litter: f32) -> f32 {
        self.base + self.per_cure * c + self.per_litter * litter
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cures {
    pub grass: Cure,
    pub shrub: Cure,
    pub deciduous: Cure,
    pub conifer: Cure,
    pub evergreen: Cure,
    pub cactus: Cure,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shade {
    pub per_tree: f32,
    pub own_tree: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WaterTerm {
    pub bank: f32,
    pub from_cells: f32,
    pub to_cells: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hollow {
    pub per_layer: f32,
    pub clamp_layers: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Patch {
    pub base: f32,
    pub span: f32,
    pub field_m: f32,
    pub cell_base: f32,
    pub cell_span: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Grazed {
    pub mult: f32,
    pub cells: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Dryness {
    pub dry_min: f32,
    pub burn_power: f32,
    pub season: SeasonTerms,
    pub cure: Cures,
    pub shade: Shade,
    pub water: WaterTerm,
    pub hollow: Hollow,
    pub patch: Patch,
    pub grazed: Grazed,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct FuelRow {
    pub spread_per_s: f32,
    pub burn_s: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fuels {
    pub grass: FuelRow,
    pub shrub: FuelRow,
    pub deciduous: FuelRow,
    pub conifer: FuelRow,
    pub evergreen: FuelRow,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Crown {
    pub mult: f32,
    pub dry: f32,
    pub wind_mps: f32,
    pub crown_s: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WindTerm {
    pub down_per_mps: f32,
    pub up_per_mps: f32,
    pub floor: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SlopeTerm {
    pub up_per_layer: f32,
    pub down: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct RainTerm {
    pub spread_scale_mmh: f32,
    pub douse_s_per_mmh: f32,
    pub no_catch_mmh: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ignite {
    pub lightning_dry: f32,
    pub lightning_rain_below_mmh: f32,
    pub lightning_quiet_days: f32,
    pub lightning_quiet_m: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Limits {
    pub live_blazes: usize,
    pub cells: u32,
    pub burning: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Regrow {
    pub shoots: f64,
    pub grass: f64,
    pub grass_whole: f64,
    pub shrub: f64,
    pub sapling: f64,
    pub young: f64,
    pub tree: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Snag {
    pub good: String,
    pub count: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Torch {
    pub placed_out_s: f32,
    pub held_out_s: f32,
    pub rain_seen: f32,
    pub sky_layers: u32,
    pub relight_s: f32,
    pub damp_flame_s: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Harm {
    pub in_cell_per_s: f32,
    pub beside_per_s: f32,
    pub flee_m: f32,
}

/// The whole file.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FireConfig {
    pub tick_s: f32,
    pub ground_wet: GroundWetRates,
    pub weather: WeatherTerms,
    pub dryness: Dryness,
    pub fuel: Fuels,
    pub crown: Crown,
    pub wind: WindTerm,
    pub slope: SlopeTerm,
    pub rain: RainTerm,
    pub ignite: Ignite,
    pub limits: Limits,
    pub regrow_days: Regrow,
    pub snag: Snag,
    pub torch: Torch,
    pub harm: Harm,
    pub bucket_wet_days: f64,
}

fn check(name: &str, v: f32, lo: f32, hi: f32) -> Result<(), String> {
    if v.is_finite() && (lo..=hi).contains(&v) {
        Ok(())
    } else {
        Err(format!("fire.ron {name} = {v}, wants {lo} to {hi}"))
    }
}

fn positive(name: &str, v: f32) -> Result<(), String> {
    if v.is_finite() && v > 0.0 {
        Ok(())
    } else {
        Err(format!("fire.ron {name} = {v}, wants above 0"))
    }
}

impl FireConfig {
    /// The file built into the binary, parsed and validated. Panics only if
    /// the shipped file is broken, which a test stops before a push.
    pub fn shipped() -> Self {
        Self::parse(SHIPPED_RON).expect("assets/config/fire.ron parses and validates")
    }

    /// A file's text, parsed and validated.
    pub fn parse(text: &str) -> Result<Self, String> {
        let file: Self = ron::from_str(text).map_err(|e| format!("fire.ron: {e}"))?;
        file.validate()?;
        Ok(file)
    }

    /// Every number finite and in its range, every threshold in order.
    pub fn validate(&self) -> Result<(), String> {
        positive("tick_s", self.tick_s)?;
        let w = &self.ground_wet;
        check("ground_wet.per_mm", w.per_mm, 0.0, 1.0)?;
        check("ground_wet.dry_per_hour", w.dry_per_hour, 0.0, 1.0)?;
        positive("ground_wet.sun_ref_wm2", w.sun_ref_wm2)?;
        check("ground_wet.dry_from_c", w.dry_from_c, -50.0, 50.0)?;
        positive("ground_wet.dry_span_c", w.dry_span_c)?;
        let t = &self.weather;
        check("weather.humid_dry", t.humid_dry, 0.0, 1.0)?;
        check("weather.humid_wet", t.humid_wet, 0.0, 1.0)?;
        if t.humid_dry >= t.humid_wet {
            return Err("fire.ron weather.humid_dry wants below humid_wet".into());
        }
        check("weather.humid_keep", t.humid_keep, 0.0, 1.0)?;
        if t.warm_from_c >= t.warm_full_c {
            return Err("fire.ron weather.warm_from_c wants below warm_full_c".into());
        }
        check("weather.warm_floor", t.warm_floor, 0.0, 1.0)?;
        let d = &self.dryness;
        check("dryness.dry_min", d.dry_min, 0.01, 0.99)?;
        positive("dryness.burn_power", d.burn_power)?;
        check("dryness.season.dry_peak", d.season.dry_peak, 0.0, 1.0)?;
        check("dryness.season.dry_half", d.season.dry_half, 0.01, 0.5)?;
        check("dryness.season.straw_share", d.season.straw_share, 0.0, 1.0)?;
        for (name, c) in [
            ("grass", d.cure.grass),
            ("shrub", d.cure.shrub),
            ("deciduous", d.cure.deciduous),
            ("conifer", d.cure.conifer),
            ("evergreen", d.cure.evergreen),
            ("cactus", d.cure.cactus),
        ] {
            check(&format!("dryness.cure.{name}.base"), c.base, 0.0, 1.0)?;
            check(
                &format!("dryness.cure.{name}.per_cure"),
                c.per_cure,
                0.0,
                1.0,
            )?;
            check(
                &format!("dryness.cure.{name}.per_litter"),
                c.per_litter,
                0.0,
                1.0,
            )?;
        }
        if d.cure.cactus.of(1.0, 1.0) != 0.0 {
            return Err("fire.ron dryness.cure.cactus wants nought: a cactus never burns".into());
        }
        check("dryness.shade.per_tree", d.shade.per_tree, 0.0, 0.16)?;
        check("dryness.shade.own_tree", d.shade.own_tree, 0.0, 1.0)?;
        check("dryness.water.bank", d.water.bank, 0.0, 1.0)?;
        if d.water.from_cells >= d.water.to_cells {
            return Err("fire.ron dryness.water.from_cells wants below to_cells".into());
        }
        check("dryness.hollow.per_layer", d.hollow.per_layer, 0.0, 0.5)?;
        check(
            "dryness.hollow.clamp_layers",
            d.hollow.clamp_layers,
            0.0,
            8.0,
        )?;
        positive("dryness.patch.field_m", d.patch.field_m)?;
        check("dryness.patch.base", d.patch.base, 0.0, 2.0)?;
        check("dryness.patch.span", d.patch.span, 0.0, 2.0)?;
        check("dryness.patch.cell_base", d.patch.cell_base, 0.0, 2.0)?;
        check("dryness.patch.cell_span", d.patch.cell_span, 0.0, 2.0)?;
        check("dryness.grazed.mult", d.grazed.mult, 0.0, 1.0)?;
        for (name, f) in [
            ("grass", self.fuel.grass),
            ("shrub", self.fuel.shrub),
            ("deciduous", self.fuel.deciduous),
            ("conifer", self.fuel.conifer),
            ("evergreen", self.fuel.evergreen),
        ] {
            check(
                &format!("fuel.{name}.spread_per_s"),
                f.spread_per_s,
                0.0,
                5.0,
            )?;
            positive(&format!("fuel.{name}.burn_s"), f.burn_s)?;
        }
        check("crown.mult", self.crown.mult, 1.0, 10.0)?;
        check("crown.dry", self.crown.dry, d.dry_min, 1.0)?;
        check("crown.wind_mps", self.crown.wind_mps, 0.0, 50.0)?;
        check(
            "crown.crown_s",
            self.crown.crown_s,
            0.0,
            self.fuel.conifer.burn_s,
        )?;
        check("wind.down_per_mps", self.wind.down_per_mps, 0.0, 1.0)?;
        check("wind.up_per_mps", self.wind.up_per_mps, 0.0, 1.0)?;
        check("wind.floor", self.wind.floor, 0.0, 1.0)?;
        check("slope.up_per_layer", self.slope.up_per_layer, 0.0, 1.0)?;
        check("slope.down", self.slope.down, 0.0, 1.0)?;
        positive("rain.spread_scale_mmh", self.rain.spread_scale_mmh)?;
        check(
            "rain.douse_s_per_mmh",
            self.rain.douse_s_per_mmh,
            0.0,
            100.0,
        )?;
        positive("rain.no_catch_mmh", self.rain.no_catch_mmh)?;
        check(
            "ignite.lightning_dry",
            self.ignite.lightning_dry,
            d.dry_min,
            1.0,
        )?;
        check(
            "ignite.lightning_rain_below_mmh",
            self.ignite.lightning_rain_below_mmh,
            0.0,
            self.rain.no_catch_mmh,
        )?;
        positive(
            "ignite.lightning_quiet_days",
            self.ignite.lightning_quiet_days,
        )?;
        positive("ignite.lightning_quiet_m", self.ignite.lightning_quiet_m)?;
        let l = &self.limits;
        if l.live_blazes == 0 || l.cells == 0 || l.burning == 0 {
            return Err("fire.ron limits want each above 0".into());
        }
        let r = &self.regrow_days;
        let ladder = [
            r.shoots,
            r.grass,
            r.grass_whole,
            r.shrub,
            r.sapling,
            r.young,
            r.tree,
        ];
        if ladder.iter().any(|v| !v.is_finite() || *v < 0.0)
            || ladder.windows(2).any(|w| w[0] > w[1])
        {
            return Err("fire.ron regrow_days wants each in order, shoots to tree".into());
        }
        if self.snag.good.is_empty() || self.snag.count == 0 {
            return Err("fire.ron snag wants a good and a count".into());
        }
        let t = &self.torch;
        positive("torch.placed_out_s", t.placed_out_s)?;
        positive("torch.held_out_s", t.held_out_s)?;
        check("torch.rain_seen", t.rain_seen, 0.0, 1.0)?;
        positive("torch.relight_s", t.relight_s)?;
        positive("torch.damp_flame_s", t.damp_flame_s)?;
        check("harm.in_cell_per_s", self.harm.in_cell_per_s, 0.0, 100.0)?;
        check("harm.beside_per_s", self.harm.beside_per_s, 0.0, 100.0)?;
        positive("harm.flee_m", self.harm.flee_m)?;
        if !self.bucket_wet_days.is_finite() || self.bucket_wet_days < 0.0 {
            return Err("fire.ron bucket_wet_days wants 0 or more".into());
        }
        Ok(())
    }
}
