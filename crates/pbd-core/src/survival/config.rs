//! `assets/config/survival.ron` (`survival-mode` design 20): every number of
//! the survival rules, with its unit in its name, validated at load. The file
//! is the one source of these values: [`SurvivalConfig::shipped`] parses the
//! copy built into the binary, and the app may read the same file from disk.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::inventory::Tier;

/// The shipped file, built in.
pub const SHIPPED_RON: &str = include_str!("../../../../assets/config/survival.ron");

/// `assets/config/survival.ron`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SurvivalConfig {
    pub health: HealthConfig,
    pub fall: FallConfig,
    pub food: FoodConfig,
    pub stamina: StaminaConfig,
    pub water: WaterConfig,
    pub body: BodyConfig,
    pub pass_out: PassOutConfig,
    pub breath: BreathConfig,
    pub suit_air: SuitAirConfig,
    pub drinks: DrinksConfig,
    pub still: StillConfig,
    /// Food a dish fills at one star, fp, by good code.
    pub fills: BTreeMap<String, f32>,
    /// Times the one-star fill at one, two and three stars.
    pub star_fill_mult: [f32; 3],
    pub weapons: WeaponsConfig,
    pub hunting: HuntingConfig,
    pub wear: WearConfig,
    /// The hunt (`survival-mode` combat C2), by species id.
    pub predators: BTreeMap<String, Predator>,
    /// The hunt's shared numbers.
    pub hunt: HuntConfig,
    /// Townsfolk who fight back (combat C3).
    pub townsfolk: TownsfolkConfig,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HealthConfig {
    pub max_pts: f32,
    pub regen_s_per_pt: f32,
    pub regen_hungry_mult: f32,
    pub hungry_below_fp: f32,
    pub starving_drain_s_per_pt: f32,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FallConfig {
    pub safe_height_m: f32,
    pub pts_per_mps: f32,
    pub water_depth_layers: u32,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FoodConfig {
    pub max_fp: f32,
    pub idle_empty_min: f32,
    pub start_fp: f32,
    pub wake_fp: f32,
    pub cold_mult: f32,
}

impl FoodConfig {
    /// Food burned a second standing about, fp/s.
    pub fn idle_fp_per_s(&self) -> f32 {
        self.max_fp / (self.idle_empty_min * 60.0)
    }
}

/// What a continuing effort spends.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EffortRate {
    pub pts_per_s: f32,
    pub fp_per_s: f32,
    /// Whether the points it spends are owed as strain.
    pub strain: bool,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SpendConfig {
    pub sprint: EffortRate,
    pub swim: EffortRate,
    pub swim_sprint: EffortRate,
    pub tread: EffortRate,
    pub bow_drawn: EffortRate,
    pub jump_pts: f32,
    pub climb_pts_per_m: f32,
    pub spear_throw_pts: f32,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WindedConfig {
    pub swim_speed_mult: f32,
    pub jump_mult: f32,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StaminaConfig {
    pub max_pts: f32,
    pub regen_pts_per_s: f32,
    pub regen_delay_s: f32,
    pub cost_low_fp_per_pt: f32,
    pub cost_high_fp_per_pt: f32,
    pub strain_mult: f32,
    pub full_rate_above_fp: f32,
    pub rate_at_empty_food: f32,
    pub starving_cap_pts: f32,
    pub winded_until_pts: f32,
    pub winded: WindedConfig,
    pub cold_regen_mult: f32,
    pub spend: SpendConfig,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WaterConfig {
    pub max: f32,
    pub start: f32,
    pub wake: f32,
    pub comfort_empty_min: f32,
    pub exertion_per_s: f32,
    pub heat_above_c: f32,
    pub heat_per_c_above: f32,
    pub cold_below_c: f32,
    pub cold_mult: f32,
    pub empty_drain_s_per_pt: f32,
    pub dry_below: f32,
    pub thirsty_at: f32,
    pub very_thirsty_at: f32,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FeelConfig {
    pub sun_c_at_1000_wm2: f32,
    pub humid_above_c: f32,
    pub humid_c_per_unit: f32,
    pub chill_c_per_mps: f32,
    pub chill_above_mps: f32,
    pub chill_below_c: f32,
    pub rain_c: f32,
    pub wet_c: f32,
    pub wet_s: f32,
    pub wet_fire_dry_mult: f32,
    pub water_c: (f32, f32),
    pub in_water_c: f32,
    pub shelter_pull: f32,
    pub shelter_c: f32,
    pub shade_march_layers: u32,
    pub shade_step_m: f32,
    pub shade_max_m: f32,
}

/// A fire's warmth.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FireHeat {
    /// `block:<name>` for a placed fire, `held:<good>` for one in hand.
    pub source: String,
    /// deg C at the fire.
    pub c: f32,
    /// Metres to nothing; 0 for a fire in hand.
    pub reach_m: f32,
}

/// What a worn item does to feel.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WornHeat {
    pub item: String,
    pub cold_c: f32,
    pub hot_c: f32,
    pub sun_mult: f32,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BodyConfig {
    pub start_c: f32,
    pub time_constant_s: f32,
    pub comfort_c: (f32, f32),
    pub hot_c_per_c: f32,
    pub hot_dry_c_per_c: f32,
    pub exertion_c: f32,
    pub cold_c_per_c: f32,
    pub feel: FeelConfig,
    pub fires: Vec<FireHeat>,
    pub worn: Vec<WornHeat>,
    pub hot_c: f32,
    pub exhaustion_c: f32,
    pub stroke_c: f32,
    pub cool_c: f32,
    pub cold_c: f32,
    pub hypothermia_c: f32,
    pub hurt_per_s: f32,
    pub stroke_pts: f32,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PassOutConfig {
    pub fade_s: f32,
    pub out_s: f32,
    pub cool_to_c: f32,
    pub dazed_s: f32,
    pub dazed_walk_mult: f32,
    pub revive_hold_s: f32,
    pub revive_water: f32,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BreathConfig {
    pub under_s: f32,
    pub refill_s: f32,
    pub drown_pts_per_s: f32,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SuitAirConfig {
    pub reserve_s: f32,
    pub refill_s: f32,
}

/// A sip from a source.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Sip {
    pub water: f32,
    pub sip_s: f32,
}

/// A sip from a source that holds a few.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HeldSips {
    pub water: f32,
    pub sip_s: f32,
    pub sips: u8,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CactusSips {
    pub water: f32,
    pub sip_s: f32,
    pub sips: u8,
    pub regrow_days: f64,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DrinksConfig {
    pub fresh: Sip,
    pub bucket: HeldSips,
    pub cactus: CactusSips,
    pub still: HeldSips,
    pub rain_per_s: f32,
    /// Water a good gives when eaten, by good code.
    pub eaten: BTreeMap<String, f32>,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StillConfig {
    pub boil_s_per_sip: f32,
    pub rain_s_per_sip: f32,
    pub fire_reach_cells: u32,
}

/// A melee weapon.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Melee {
    pub damage: f32,
    /// Damage when thrown, for a weapon that can be.
    pub thrown: Option<f32>,
    pub reach_m: f32,
    pub time_s: f32,
    /// A tier digit (0 stone, 1 bronze, 2 iron, 3 steel).
    pub tier: u8,
}

impl Melee {
    pub fn tier(&self) -> Tier {
        Tier::of_digit(self.tier).unwrap_or(Tier::Stone)
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BowConfig {
    pub draw_s: f32,
    pub arrow_mps: f32,
    pub kept_share: f32,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponsConfig {
    pub hand: f32,
    pub axe_by_tier: [f32; 4],
    pub pickaxe_share: f32,
    pub shovel_share: f32,
    pub melee: BTreeMap<String, Melee>,
    pub spear_throw_mps: f32,
    pub bow: BowConfig,
    pub arrows: BTreeMap<String, f32>,
    pub flight_max_s: f32,
    /// A swing's hit lands this share of its `time_s` after the press
    /// (combat C1).
    pub hit_share: f32,
    /// A tool's or bare hands' swing, s.
    pub tool_swing_s: f32,
    /// A spear is raised at least this long before a release throws it, s.
    pub throw_min_s: f32,
}

/// One predator's hunt (combat C2): distances m, speeds m/s, times s,
/// damage pts.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Predator {
    pub notice_m: f32,
    pub stalk_mps: f32,
    pub warn_m: f32,
    pub warn_s: f32,
    pub lunge_m: f32,
    pub lunge_mps: f32,
    pub bite_m: f32,
    pub damage: f32,
}

/// What every predator shares (combat C2).
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HuntConfig {
    /// Notice reaches this many times further at night.
    pub night_mult: f32,
    /// The least a warning is held before the first attack, s (design 6).
    pub warn_min_s: f32,
    /// A shorter warning between attacks, s.
    pub rewarn_s: f32,
    /// Flees below this share of its health.
    pub flee_below: f32,
    pub flee_s: f32,
    /// After fleeing it notices nothing this long, s.
    pub calm_s: f32,
    /// Gives up when the player is past 1.5 x notice this long, s.
    pub give_up_s: f32,
    /// A lunge is over after this long, s.
    pub lunge_s: f32,
    /// How the lunge is drawn (drawn only: no rule reads it).
    pub lunge_draw: LungeDraw,
}

/// The lunge's drawing (`survival-mode` combat C2, the Hunt tab's draft):
/// it gathers, low and still, for the first `gather_share` of the lunge,
/// springs on an arc `lift_m` high with its nose up `pitch_rad` until
/// `land_share`, and lands with a squash. Drawn only.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LungeDraw {
    pub gather_share: f32,
    pub land_share: f32,
    pub gather_crouch: f32,
    pub lift_m: f32,
    pub pitch_rad: f32,
    pub land_squash: f32,
}

/// Townsfolk who fight back (combat C3).
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TownsfolkConfig {
    /// Standing lost a strike on a townsperson.
    pub struck_standing: i32,
    /// Guards within this of the struck come, m.
    pub call_m: f32,
    pub guard_mps: f32,
    pub guard_reach_m: f32,
    pub guard_windup_s: f32,
    pub guard_damage: f32,
    pub guard_every_s: f32,
    /// Anyone else runs from the player this long, s.
    pub flee_s: f32,
    /// The fight ends this far past the town's walls, m.
    pub leave_m: f32,
}

/// One hunted species.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Game {
    pub health: f32,
    /// It runs once a hunter is this close, m (0 for a predator).
    pub flee_m: f32,
    pub flee_mps: f32,
    /// What a kill drops at once: (good, count).
    pub drops: Vec<(String, u16)>,
    /// Hides it gives when dressed.
    pub hide: u16,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HuntingConfig {
    pub carcass_s: f32,
    pub herd_back_days: f64,
    pub dress_bonus_meat: u16,
    pub other_blade_hide_share: f32,
    /// The good codes of meat and hide.
    pub meat: String,
    pub hide: String,
    pub game: BTreeMap<String, Game>,
}

/// Uses by tier.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UsesByTier {
    pub stone: u32,
    pub bronze: u32,
    pub iron: u32,
    pub steel: u32,
}

impl UsesByTier {
    pub fn of(&self, tier: Tier) -> u32 {
        match tier {
            Tier::Stone => self.stone,
            Tier::Bronze => self.bronze,
            Tier::Iron => self.iron,
            Tier::Steel => self.steel,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WearConfig {
    pub tool_uses: UsesByTier,
    pub rod_uses: UsesByTier,
    pub straw_uses: u32,
    pub bow_uses: u32,
    pub harder_block_uses: u32,
    pub handle_goods: Vec<String>,
    pub red_below_share: f32,
}

/// A number that must be finite and in `lo..=hi`.
fn within(name: &str, v: f32, lo: f32, hi: f32) -> Result<(), String> {
    if v.is_finite() && v >= lo && v <= hi {
        Ok(())
    } else {
        Err(format!("survival.ron {name} = {v}, wants {lo}..={hi}"))
    }
}

/// A number that must be finite and above zero.
fn positive(name: &str, v: f32) -> Result<(), String> {
    if v.is_finite() && v > 0.0 {
        Ok(())
    } else {
        Err(format!("survival.ron {name} = {v}, wants above 0"))
    }
}

/// A number that must be finite and zero or more.
fn non_negative(name: &str, v: f32) -> Result<(), String> {
    if v.is_finite() && v >= 0.0 {
        Ok(())
    } else {
        Err(format!("survival.ron {name} = {v}, wants 0 or more"))
    }
}

fn ordered(name: &str, (lo, hi): (f32, f32)) -> Result<(), String> {
    if lo.is_finite() && hi.is_finite() && lo < hi {
        Ok(())
    } else {
        Err(format!(
            "survival.ron {name} = ({lo}, {hi}), wants low below high"
        ))
    }
}

impl SurvivalConfig {
    /// The file built into the binary, parsed and validated. Panics only if
    /// the shipped file is broken, which a test stops before a push.
    pub fn shipped() -> Self {
        Self::parse(SHIPPED_RON).expect("assets/config/survival.ron parses and validates")
    }

    /// A file's text, parsed and validated.
    pub fn parse(text: &str) -> Result<Self, String> {
        let file: Self = ron::from_str(text).map_err(|e| format!("survival.ron: {e}"))?;
        file.validate()?;
        Ok(file)
    }

    /// Every number finite and in its range, every threshold in order.
    pub fn validate(&self) -> Result<(), String> {
        let h = &self.health;
        positive("health.max_pts", h.max_pts)?;
        positive("health.regen_s_per_pt", h.regen_s_per_pt)?;
        within("health.regen_hungry_mult", h.regen_hungry_mult, 0.0, 1.0)?;
        non_negative("health.hungry_below_fp", h.hungry_below_fp)?;
        positive("health.starving_drain_s_per_pt", h.starving_drain_s_per_pt)?;

        positive("fall.safe_height_m", self.fall.safe_height_m)?;
        non_negative("fall.pts_per_mps", self.fall.pts_per_mps)?;

        let f = &self.food;
        positive("food.max_fp", f.max_fp)?;
        positive("food.idle_empty_min", f.idle_empty_min)?;
        within("food.start_fp", f.start_fp, 0.0, f.max_fp)?;
        within("food.wake_fp", f.wake_fp, 0.0, f.max_fp)?;
        within("food.cold_mult", f.cold_mult, 1.0, 10.0)?;
        within("health.hungry_below_fp", h.hungry_below_fp, 0.0, f.max_fp)?;

        let s = &self.stamina;
        positive("stamina.max_pts", s.max_pts)?;
        positive("stamina.regen_pts_per_s", s.regen_pts_per_s)?;
        non_negative("stamina.regen_delay_s", s.regen_delay_s)?;
        non_negative("stamina.cost_low_fp_per_pt", s.cost_low_fp_per_pt)?;
        non_negative("stamina.cost_high_fp_per_pt", s.cost_high_fp_per_pt)?;
        within("stamina.strain_mult", s.strain_mult, 1.0, 10.0)?;
        within(
            "stamina.full_rate_above_fp",
            s.full_rate_above_fp,
            0.0,
            f.max_fp,
        )?;
        within("stamina.rate_at_empty_food", s.rate_at_empty_food, 0.0, 1.0)?;
        within(
            "stamina.starving_cap_pts",
            s.starving_cap_pts,
            0.0,
            s.max_pts,
        )?;
        within(
            "stamina.winded_until_pts",
            s.winded_until_pts,
            0.0,
            s.max_pts,
        )?;
        within(
            "stamina.winded.swim_speed_mult",
            s.winded.swim_speed_mult,
            0.0,
            1.0,
        )?;
        within("stamina.winded.jump_mult", s.winded.jump_mult, 0.0, 1.0)?;
        within("stamina.cold_regen_mult", s.cold_regen_mult, 0.0, 1.0)?;
        let sp = &s.spend;
        for (name, e) in [
            ("sprint", sp.sprint),
            ("swim", sp.swim),
            ("swim_sprint", sp.swim_sprint),
            ("tread", sp.tread),
            ("bow_drawn", sp.bow_drawn),
        ] {
            non_negative(&format!("stamina.spend.{name}.pts_per_s"), e.pts_per_s)?;
            non_negative(&format!("stamina.spend.{name}.fp_per_s"), e.fp_per_s)?;
        }
        non_negative("stamina.spend.jump_pts", sp.jump_pts)?;
        non_negative("stamina.spend.climb_pts_per_m", sp.climb_pts_per_m)?;
        non_negative("stamina.spend.spear_throw_pts", sp.spear_throw_pts)?;

        let w = &self.water;
        positive("water.max", w.max)?;
        within("water.start", w.start, 0.0, w.max)?;
        within("water.wake", w.wake, 0.0, w.max)?;
        positive("water.comfort_empty_min", w.comfort_empty_min)?;
        non_negative("water.exertion_per_s", w.exertion_per_s)?;
        non_negative("water.heat_per_c_above", w.heat_per_c_above)?;
        within("water.cold_mult", w.cold_mult, 0.0, 10.0)?;
        positive("water.empty_drain_s_per_pt", w.empty_drain_s_per_pt)?;
        within("water.dry_below", w.dry_below, 0.0, w.max)?;
        within("water.thirsty_at", w.thirsty_at, 0.0, w.max)?;
        within(
            "water.very_thirsty_at",
            w.very_thirsty_at,
            0.0,
            w.thirsty_at,
        )?;
        if !(w.heat_above_c.is_finite() && w.cold_below_c.is_finite()) {
            return Err("survival.ron water thresholds must be finite".into());
        }

        let b = &self.body;
        within("body.start_c", b.start_c, 30.0, 45.0)?;
        positive("body.time_constant_s", b.time_constant_s)?;
        ordered("body.comfort_c", b.comfort_c)?;
        non_negative("body.hot_c_per_c", b.hot_c_per_c)?;
        non_negative("body.hot_dry_c_per_c", b.hot_dry_c_per_c)?;
        non_negative("body.exertion_c", b.exertion_c)?;
        non_negative("body.cold_c_per_c", b.cold_c_per_c)?;
        let ladder = [
            ("hypothermia_c", b.hypothermia_c),
            ("cold_c", b.cold_c),
            ("cool_c", b.cool_c),
            ("start_c", b.start_c),
            ("hot_c", b.hot_c),
            ("exhaustion_c", b.exhaustion_c),
            ("stroke_c", b.stroke_c),
        ];
        for pair in ladder.windows(2) {
            if !(pair[0].1.is_finite() && pair[0].1 < pair[1].1) {
                return Err(format!(
                    "survival.ron body.{} must be below body.{}",
                    pair[0].0, pair[1].0
                ));
            }
        }
        non_negative("body.hurt_per_s", b.hurt_per_s)?;
        non_negative("body.stroke_pts", b.stroke_pts)?;
        let fe = &b.feel;
        non_negative("body.feel.sun_c_at_1000_wm2", fe.sun_c_at_1000_wm2)?;
        non_negative("body.feel.humid_c_per_unit", fe.humid_c_per_unit)?;
        non_negative("body.feel.chill_c_per_mps", fe.chill_c_per_mps)?;
        non_negative("body.feel.chill_above_mps", fe.chill_above_mps)?;
        positive("body.feel.wet_s", fe.wet_s)?;
        within(
            "body.feel.wet_fire_dry_mult",
            fe.wet_fire_dry_mult,
            1.0,
            10.0,
        )?;
        ordered("body.feel.water_c", fe.water_c)?;
        within("body.feel.shelter_pull", fe.shelter_pull, 0.0, 1.0)?;
        positive("body.feel.shade_step_m", fe.shade_step_m)?;
        positive("body.feel.shade_max_m", fe.shade_max_m)?;
        for v in [
            fe.humid_above_c,
            fe.chill_below_c,
            fe.rain_c,
            fe.wet_c,
            fe.in_water_c,
            fe.shelter_c,
        ] {
            if !v.is_finite() {
                return Err("survival.ron body.feel has a number that is not finite".into());
            }
        }
        for fire in &b.fires {
            non_negative(&format!("body.fires.{}.c", fire.source), fire.c)?;
            non_negative(&format!("body.fires.{}.reach_m", fire.source), fire.reach_m)?;
        }
        for worn in &b.worn {
            within(
                &format!("body.worn.{}.sun_mult", worn.item),
                worn.sun_mult,
                0.0,
                1.0,
            )?;
            if !(worn.cold_c.is_finite() && worn.hot_c.is_finite()) {
                return Err(format!(
                    "survival.ron body.worn.{} is not finite",
                    worn.item
                ));
            }
        }

        let p = &self.pass_out;
        non_negative("pass_out.fade_s", p.fade_s)?;
        positive("pass_out.out_s", p.out_s)?;
        within("pass_out.cool_to_c", p.cool_to_c, b.start_c, b.stroke_c)?;
        non_negative("pass_out.dazed_s", p.dazed_s)?;
        within("pass_out.dazed_walk_mult", p.dazed_walk_mult, 0.0, 1.0)?;
        non_negative("pass_out.revive_hold_s", p.revive_hold_s)?;
        non_negative("pass_out.revive_water", p.revive_water)?;

        positive("breath.under_s", self.breath.under_s)?;
        positive("breath.refill_s", self.breath.refill_s)?;
        non_negative("breath.drown_pts_per_s", self.breath.drown_pts_per_s)?;
        positive("suit_air.reserve_s", self.suit_air.reserve_s)?;
        positive("suit_air.refill_s", self.suit_air.refill_s)?;

        let d = &self.drinks;
        non_negative("drinks.fresh.water", d.fresh.water)?;
        positive("drinks.fresh.sip_s", d.fresh.sip_s)?;
        for (name, held) in [("bucket", &d.bucket), ("still", &d.still)] {
            non_negative(&format!("drinks.{name}.water"), held.water)?;
            positive(&format!("drinks.{name}.sip_s"), held.sip_s)?;
            if held.sips == 0 {
                return Err(format!("survival.ron drinks.{name}.sips must be 1 or more"));
            }
        }
        non_negative("drinks.cactus.water", d.cactus.water)?;
        positive("drinks.cactus.sip_s", d.cactus.sip_s)?;
        if d.cactus.sips == 0 || !(d.cactus.regrow_days.is_finite() && d.cactus.regrow_days > 0.0) {
            return Err("survival.ron drinks.cactus needs sips and a regrow time".into());
        }
        non_negative("drinks.rain_per_s", d.rain_per_s)?;
        for (good, v) in &d.eaten {
            non_negative(&format!("drinks.eaten.{good}"), *v)?;
        }
        positive("still.boil_s_per_sip", self.still.boil_s_per_sip)?;
        positive("still.rain_s_per_sip", self.still.rain_s_per_sip)?;

        for (good, v) in &self.fills {
            within(&format!("fills.{good}"), *v, 0.0, f.max_fp)?;
        }
        for (i, m) in self.star_fill_mult.iter().enumerate() {
            within(&format!("star_fill_mult.{i}"), *m, 0.0, 10.0)?;
        }

        let wp = &self.weapons;
        non_negative("weapons.hand", wp.hand)?;
        for (i, v) in wp.axe_by_tier.iter().enumerate() {
            non_negative(&format!("weapons.axe_by_tier.{i}"), *v)?;
        }
        within("weapons.pickaxe_share", wp.pickaxe_share, 0.0, 1.0)?;
        within("weapons.shovel_share", wp.shovel_share, 0.0, 1.0)?;
        for (id, m) in &wp.melee {
            non_negative(&format!("weapons.melee.{id}.damage"), m.damage)?;
            if let Some(t) = m.thrown {
                non_negative(&format!("weapons.melee.{id}.thrown"), t)?;
            }
            positive(&format!("weapons.melee.{id}.reach_m"), m.reach_m)?;
            positive(&format!("weapons.melee.{id}.time_s"), m.time_s)?;
            if Tier::of_digit(m.tier).is_none() {
                return Err(format!(
                    "survival.ron weapons.melee.{id}.tier must be 0..=3"
                ));
            }
        }
        positive("weapons.spear_throw_mps", wp.spear_throw_mps)?;
        positive("weapons.bow.draw_s", wp.bow.draw_s)?;
        positive("weapons.bow.arrow_mps", wp.bow.arrow_mps)?;
        within("weapons.bow.kept_share", wp.bow.kept_share, 0.0, 1.0)?;
        for (id, v) in &wp.arrows {
            non_negative(&format!("weapons.arrows.{id}"), *v)?;
        }
        positive("weapons.flight_max_s", wp.flight_max_s)?;
        within("weapons.hit_share", wp.hit_share, 0.05, 0.95)?;
        positive("weapons.tool_swing_s", wp.tool_swing_s)?;
        non_negative("weapons.throw_min_s", wp.throw_min_s)?;

        for (id, p) in &self.predators {
            let n = |f: &str| format!("predators.{id}.{f}");
            positive(&n("notice_m"), p.notice_m)?;
            positive(&n("stalk_mps"), p.stalk_mps)?;
            positive(&n("warn_m"), p.warn_m)?;
            positive(&n("lunge_m"), p.lunge_m)?;
            positive(&n("lunge_mps"), p.lunge_mps)?;
            positive(&n("bite_m"), p.bite_m)?;
            non_negative(&n("damage"), p.damage)?;
            if p.warn_s < self.hunt.warn_min_s {
                return Err(format!(
                    "survival.ron predators.{id}.warn_s is below hunt.warn_min_s"
                ));
            }
            if !(p.bite_m < p.lunge_m && p.lunge_m <= p.warn_m && p.warn_m < p.notice_m) {
                return Err(format!(
                    "survival.ron predators.{id}: bite < lunge <= warn < notice"
                ));
            }
            if !self.hunting.game.contains_key(id) {
                return Err(format!(
                    "survival.ron predators.{id} has no hunting.game row"
                ));
            }
        }
        let hn = &self.hunt;
        within("hunt.night_mult", hn.night_mult, 1.0, 4.0)?;
        within("hunt.warn_min_s", hn.warn_min_s, 1.5, 10.0)?;
        positive("hunt.rewarn_s", hn.rewarn_s)?;
        within("hunt.flee_below", hn.flee_below, 0.0, 1.0)?;
        positive("hunt.flee_s", hn.flee_s)?;
        non_negative("hunt.calm_s", hn.calm_s)?;
        positive("hunt.give_up_s", hn.give_up_s)?;
        positive("hunt.lunge_s", hn.lunge_s)?;
        let ld = &hn.lunge_draw;
        if !(0.0 < ld.gather_share && ld.gather_share < ld.land_share && ld.land_share <= 1.0) {
            return Err("survival.ron hunt.lunge_draw: 0 < gather_share < land_share <= 1".into());
        }
        within("hunt.lunge_draw.gather_crouch", ld.gather_crouch, 0.0, 1.0)?;
        within("hunt.lunge_draw.lift_m", ld.lift_m, 0.0, 1.0)?;
        within("hunt.lunge_draw.pitch_rad", ld.pitch_rad, 0.0, 1.0)?;
        within("hunt.lunge_draw.land_squash", ld.land_squash, 0.0, 1.0)?;
        let tf = &self.townsfolk;
        if !(0..=100).contains(&tf.struck_standing) {
            return Err("survival.ron townsfolk.struck_standing must be 0..=100".into());
        }
        positive("townsfolk.call_m", tf.call_m)?;
        positive("townsfolk.guard_mps", tf.guard_mps)?;
        positive("townsfolk.guard_reach_m", tf.guard_reach_m)?;
        non_negative("townsfolk.guard_windup_s", tf.guard_windup_s)?;
        non_negative("townsfolk.guard_damage", tf.guard_damage)?;
        positive("townsfolk.guard_every_s", tf.guard_every_s)?;
        positive("townsfolk.flee_s", tf.flee_s)?;
        non_negative("townsfolk.leave_m", tf.leave_m)?;

        let hu = &self.hunting;
        non_negative("hunting.carcass_s", hu.carcass_s)?;
        if !(hu.herd_back_days.is_finite() && hu.herd_back_days >= 0.0) {
            return Err("survival.ron hunting.herd_back_days must be 0 or more".into());
        }
        within(
            "hunting.other_blade_hide_share",
            hu.other_blade_hide_share,
            0.0,
            1.0,
        )?;
        for (id, g) in &hu.game {
            positive(&format!("hunting.game.{id}.health"), g.health)?;
            non_negative(&format!("hunting.game.{id}.flee_m"), g.flee_m)?;
            non_negative(&format!("hunting.game.{id}.flee_mps"), g.flee_mps)?;
        }

        let we = &self.wear;
        for (name, u) in [("tool_uses", &we.tool_uses), ("rod_uses", &we.rod_uses)] {
            for tier in Tier::ALL {
                if u.of(tier) == 0 {
                    return Err(format!(
                        "survival.ron wear.{name}.{} must be 1 or more",
                        tier.code()
                    ));
                }
            }
        }
        if we.straw_uses == 0 || we.bow_uses == 0 || we.harder_block_uses == 0 {
            return Err("survival.ron wear uses must be 1 or more".into());
        }
        within("wear.red_below_share", we.red_below_share, 0.0, 1.0)?;
        Ok(())
    }

    /// Every good this file names, for the app to check against the goods
    /// it has (`survival-mode` design 20): fills, eaten drinks, weapons,
    /// arrows, drops and worn items.
    pub fn named_goods(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        out.extend(self.fills.keys().cloned());
        out.extend(self.drinks.eaten.keys().cloned());
        out.extend(self.weapons.melee.keys().cloned());
        out.extend(self.weapons.arrows.keys().cloned());
        out.push(self.hunting.meat.clone());
        out.push(self.hunting.hide.clone());
        for g in self.hunting.game.values() {
            out.extend(g.drops.iter().map(|(good, _)| good.clone()));
        }
        out.extend(self.body.worn.iter().map(|w| w.item.clone()));
        out.sort();
        out.dedup();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shipped_file_parses_and_validates() {
        let cfg = SurvivalConfig::shipped();
        assert_eq!(cfg.health.max_pts, 100.0);
        assert!(cfg.named_goods().contains(&"raw-meat".to_string()));
    }

    #[test]
    fn a_number_out_of_range_is_refused() {
        let mut cfg = SurvivalConfig::shipped();
        cfg.stamina.strain_mult = f32::NAN;
        assert!(cfg.validate().is_err());
        let mut cfg = SurvivalConfig::shipped();
        cfg.body.stroke_c = 39.0;
        assert!(cfg.validate().is_err());
        let mut cfg = SurvivalConfig::shipped();
        cfg.wear.tool_uses.iron = 0;
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn an_unknown_field_is_refused() {
        let text = SHIPPED_RON.replacen("max_pts: 100.0,", "max_pts: 100.0, maxx: 1.0,", 1);
        assert!(SurvivalConfig::parse(&text).is_err());
    }
}
