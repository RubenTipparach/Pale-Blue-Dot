//! A player's survival state and its tick (`survival-mode` design 10): the
//! bars of [`super::health`], [`super::hunger`], [`super::water`],
//! [`super::stamina`], [`super::breath`] and [`super::temperature`] put
//! together, as pure functions of the state, the seconds of play stepped and
//! what happened. The in-process server ticks it for each player in a
//! Survival world; a client may tick a copy to predict, and the server's
//! numbers win.
//!
//! The only clock is the `dt` of play handed to [`Vitals::tick`]: nothing
//! here reads a wall clock, so a closed game changes nothing.

use serde::{Deserialize, Serialize};

use super::breath::{Air, Breath, head_air};
use super::config::SurvivalConfig;
use super::health::{Cause, empty_bar_drain, regen_pts_per_s};
use super::hunger::{eaten_water, fills, idle_burn_fp_per_s};
use super::mode::WorldMode;
use super::passout::{Consciousness, Control, Wake};
use super::stamina::{Burst, Effort, Limits, Stamina};
use super::temperature::{self, BodyState, Exposure, Wet};
use super::water::{Note, Sip, drain_per_s, thirst_note};

/// A player's survival state, as the player record keeps it.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Vitals {
    /// Health, pts.
    pub health: f32,
    /// Food, fp. f64, as water is: a tick takes a few ten-thousandths of
    /// a point from a bar near 100, which an f32 would round by a percent.
    pub food_fp: f64,
    /// Water, pts.
    pub water: f64,
    pub stamina: Stamina,
    pub breath: Breath,
    /// Body temperature, deg C.
    pub body_c: f32,
    /// Seconds of being wet left.
    pub wet_s: f32,
    pub consciousness: Consciousness,
}

/// What is going on at the player this tick: the walker's and the world's
/// part, read where the game already knows it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Conditions {
    pub effort: Effort,
    /// The weather and surroundings at the player ([`Exposure`]); its `wet`
    /// is set from the state's own wet timer.
    pub exposure: Exposure,
    /// The head's cell is water.
    pub head_in_water: bool,
    /// The body is in water deep enough to swim in.
    pub in_deep_water: bool,
    /// The air is below a breathable density (and not in a cabin).
    pub thin_air: bool,
    /// Inside a sealed craft cabin.
    pub in_cabin: bool,
}

/// Something the tick, a hit or a meal raised, for the game to show or act
/// on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// The stamina bar ran out.
    Winded,
    /// Back to a quarter.
    Recovered,
    /// A heat stroke: its blow is in the tick's damage.
    PassedOut,
    Woke(Wake),
    /// Health reached 0. The game drops the pack (design 8) and calls
    /// [`Vitals::wake_up`] when the player presses Wake up.
    Died(Cause),
    Note(Note),
    /// The heat stroke warning above the hotbar comes on (true) or goes.
    HeatWarning(bool),
}

/// What a tick did.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Tick {
    /// Health taken, by cause, this tick.
    pub damage: Vec<(Cause, f32)>,
    /// Health refilled.
    pub healed: f32,
    pub events: Vec<Event>,
    /// How the place feels, deg C.
    pub feel_c: f32,
}

/// Which survival bars the HUD's left stack shows (design 9): a full bar is
/// hidden, hunger and water only at half or lower.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shown {
    pub health: bool,
    pub stamina: bool,
    pub food: bool,
    pub water: bool,
    pub thermometer: bool,
    pub bubbles: bool,
    /// The heat stroke warning above the hotbar.
    pub heat_warning: bool,
}

/// What eating a good did.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Eaten {
    pub food_fp: f32,
    pub water: f32,
}

impl Vitals {
    /// A new player in a world of `mode`: full bars in Survival, nothing at
    /// all in Explorer (no health, hunger, stamina, water or breath).
    pub fn for_mode(cfg: &SurvivalConfig, mode: WorldMode) -> Option<Self> {
        mode.is_survival().then(|| Self::new(cfg))
    }

    /// Full health, food, water, stamina and breath, at 37 C.
    pub fn new(cfg: &SurvivalConfig) -> Self {
        Self {
            health: cfg.health.max_pts,
            food_fp: f64::from(cfg.food.start_fp),
            water: f64::from(cfg.water.start),
            stamina: Stamina::full(&cfg.stamina),
            breath: Breath::full(cfg),
            body_c: cfg.body.start_c,
            wet_s: 0.0,
            consciousness: Consciousness::Awake,
        }
    }

    /// Waking after a death (design 8): full health, half hunger, full
    /// breath, `water.wake` water.
    pub fn wake_up(&mut self, cfg: &SurvivalConfig) {
        *self = Self {
            food_fp: f64::from(cfg.food.wake_fp),
            water: f64::from(cfg.water.wake),
            ..Self::new(cfg)
        };
    }

    pub fn state(&self, cfg: &SurvivalConfig) -> BodyState {
        BodyState::of(&cfg.body, self.body_c)
    }

    pub fn is_dead(&self) -> bool {
        self.health <= 0.0
    }

    /// One tick of `dt` seconds of play.
    pub fn tick(&mut self, cfg: &SurvivalConfig, c: &Conditions, dt: f32) -> Tick {
        let mut out = Tick::default();
        if self.is_dead() || dt <= 0.0 || !dt.is_finite() {
            return out;
        }
        if let Some(wake) = self.consciousness.step(cfg, dt) {
            out.events.push(Event::Woke(wake));
        }
        let out_cold = self.consciousness.is_out();
        let warned = self.state(cfg).heat_warning();

        // Wet, and how the place feels.
        let mut wet = Wet { left_s: self.wet_s };
        let soaking = c.exposure.in_water || c.exposure.rain_on;
        let by_fire = temperature::near_fire(&cfg.body, &c.exposure.fires);
        wet.step(&cfg.body, soaking, by_fire, dt);
        self.wet_s = wet.left_s;
        let mut exposure = c.exposure.clone();
        exposure.wet = wet.is_wet() && !exposure.in_water;
        let feel = temperature::feel(&cfg.body, &exposure);
        out.feel_c = feel;
        let shivering = self.state(cfg).shivering();
        let (food, water) = (self.food_fp as f32, self.water as f32);

        // Stamina and food. Lying out cold spends nothing.
        let effort = if out_cold { Effort::None } else { c.effort };
        let st = self.stamina.step(&cfg.stamina, effort, food, shivering, dt);
        if st.became_winded {
            out.events.push(Event::Winded);
        }
        if st.recovered {
            out.events.push(Event::Recovered);
        }
        let burn = f64::from(idle_burn_fp_per_s(cfg, shivering) * dt) + f64::from(st.food_fp);
        self.food_fp = (self.food_fp - burn).clamp(0.0, f64::from(cfg.food.max_fp));

        // Water.
        let drain = f64::from(drain_per_s(cfg, feel, effort.is_hard()) * dt);
        self.water = (self.water - drain).max(0.0);
        if let Some(note) = thirst_note(cfg, water, self.water as f32) {
            out.events.push(Event::Note(note));
        }

        // Body temperature.
        let dry = (self.water as f32) < cfg.water.dry_below;
        let target = temperature::target_c(cfg, feel, dry, effort.is_hard(), out_cold);
        self.body_c = temperature::approach(&cfg.body, self.body_c, target, dt);
        let state = self.state(cfg);
        if state == BodyState::Stroke && !out_cold {
            out.damage.push((Cause::HeatStroke, cfg.body.stroke_pts));
            self.consciousness = Consciousness::pass_out(cfg);
            out.events.push(Event::PassedOut);
        }
        if state.heat_warning() != warned {
            out.events.push(Event::HeatWarning(state.heat_warning()));
        }
        match state {
            BodyState::Exhaustion => out.damage.push((Cause::Heat, cfg.body.hurt_per_s * dt)),
            BodyState::Hypothermia => out.damage.push((Cause::Cold, cfg.body.hurt_per_s * dt)),
            _ => {}
        }

        // Breath.
        let floats = self.stamina.limits(&cfg.stamina).floats
            || self
                .consciousness
                .control(cfg, c.in_deep_water)
                .floats_face_up;
        let air = if c.in_cabin {
            Air::Cabin
        } else if c.thin_air {
            Air::Thin
        } else {
            head_air(c.head_in_water, floats)
        };
        let short = self.breath.step(cfg, air, dt);
        if short > 0.0 {
            let cause = if air == Air::Thin {
                Cause::Suffocation
            } else {
                Cause::Drowning
            };
            out.damage.push((cause, short));
        }

        // Empty bars.
        for (cause, per_s) in empty_bar_drain(cfg, self.food_fp as f32, self.water as f32) {
            if per_s > 0.0 {
                out.damage.push((cause, per_s * dt));
            }
        }

        // Health: the harm, or else the refill.
        let harm: f32 = out.damage.iter().map(|(_, pts)| pts).sum();
        if harm > 0.0 {
            self.health = (self.health - harm).max(0.0);
        } else {
            let heal = (regen_pts_per_s(cfg, self.food_fp as f32, self.water as f32) * dt)
                .min(cfg.health.max_pts - self.health);
            self.health += heal;
            out.healed = heal;
        }
        if self.is_dead() {
            let cause = out
                .damage
                .iter()
                .max_by(|a, b| a.1.total_cmp(&b.1))
                .map_or(Cause::Fall, |(cause, _)| *cause);
            out.events.push(Event::Died(cause));
        }
        out
    }

    /// A hit from outside the tick (a fall, a creature, a townsperson): its
    /// damage lands, and a passed-out player wakes.
    pub fn hit(&mut self, cfg: &SurvivalConfig, cause: Cause, pts: f32) -> Vec<Event> {
        let mut events = Vec::new();
        if self.is_dead() || pts.is_nan() || pts <= 0.0 {
            return events;
        }
        if self.consciousness.wake(cfg) {
            events.push(Event::Woke(Wake::Hit));
        }
        self.health = (self.health - pts).max(0.0);
        if self.is_dead() {
            events.push(Event::Died(cause));
        }
        events
    }

    /// Another player held use on this one for `revive_hold_s` with water
    /// in hand (design 14): awake, dazed, + `revive_water`.
    pub fn revive(&mut self, cfg: &SurvivalConfig) -> bool {
        if self.consciousness.wake(cfg) {
            self.water =
                (self.water + f64::from(cfg.pass_out.revive_water)).min(f64::from(cfg.water.max));
            true
        } else {
            false
        }
    }

    /// Eating `good` at `stars`: its food (design 4) and its water (milk,
    /// fruit, stew). The buff is `cooking`'s, in both modes.
    pub fn eat(&mut self, cfg: &SurvivalConfig, good: &str, stars: u8) -> Eaten {
        let food = fills(cfg, good, stars)
            .unwrap_or(0.0)
            .min(cfg.food.max_fp - self.food_fp as f32);
        let water = eaten_water(cfg, good).min(cfg.water.max - self.water as f32);
        self.food_fp += f64::from(food);
        self.water += f64::from(water);
        Eaten {
            food_fp: food,
            water,
        }
    }

    /// A sip's water into the bar.
    pub fn drink(&mut self, cfg: &SurvivalConfig, sip: &Sip) {
        self.water = (self.water + f64::from(sip.water)).min(f64::from(cfg.water.max));
    }

    /// A one-off spend of stamina (a jump, a climb, a throw).
    pub fn burst(&mut self, cfg: &SurvivalConfig, burst: Burst) -> Option<Event> {
        self.stamina
            .burst(&cfg.stamina, burst)
            .became_winded
            .then_some(Event::Winded)
    }

    /// What the walker may do: winded's limits and the pass-out's control.
    pub fn limits(&self, cfg: &SurvivalConfig, in_deep_water: bool) -> (Limits, Control) {
        (
            self.stamina.limits(&cfg.stamina),
            self.consciousness.control(cfg, in_deep_water),
        )
    }

    /// Which bars the HUD shows (design 9).
    pub fn shown(&self, cfg: &SurvivalConfig) -> Shown {
        let state = self.state(cfg);
        Shown {
            health: self.health < cfg.health.max_pts,
            stamina: self.stamina.pts < cfg.stamina.max_pts,
            food: self.food_fp as f32 <= cfg.food.max_fp / 2.0,
            water: self.water as f32 <= cfg.water.max / 2.0,
            thermometer: state.shown(),
            bubbles: self.breath.shown(cfg),
            heat_warning: state.heat_warning(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> SurvivalConfig {
        SurvivalConfig::shipped()
    }

    fn desert_noon() -> Conditions {
        Conditions {
            exposure: Exposure {
                air_c: 38.0,
                sunlight_wm2: 900.0,
                humidity: 0.15,
                wind_mps: 2.0,
                ..Exposure::default()
            },
            ..Conditions::default()
        }
    }

    fn run(v: &mut Vitals, cfg: &SurvivalConfig, c: &Conditions, secs: f32) -> Vec<Event> {
        let mut events = Vec::new();
        for _ in 0..(secs * 10.0).round() as usize {
            events.extend(v.tick(cfg, c, 0.1).events);
        }
        events
    }

    #[test]
    fn explorer_has_no_bars() {
        assert_eq!(Vitals::for_mode(&cfg(), WorldMode::Explorer), None);
        assert!(Vitals::for_mode(&cfg(), WorldMode::Survival).is_some());
    }

    #[test]
    fn time_with_the_game_closed_changes_nothing() {
        let cfg = cfg();
        let mut v = Vitals::new(&cfg);
        run(&mut v, &cfg, &desert_noon(), 120.0);
        let quit = v.clone();
        // Saved, a day passes with the game closed, opened again: the same.
        let text = ron::to_string(&quit).unwrap();
        let mut back: Vitals = ron::from_str(&text).unwrap();
        assert_eq!(back, quit);
        // And a tick of no play time is no change.
        back.tick(&cfg, &desert_noon(), 0.0);
        assert_eq!(back, quit);
    }

    #[test]
    fn hunger_drains_at_the_idle_rate_and_health_refills() {
        let cfg = cfg();
        let mut v = Vitals::new(&cfg);
        v.health = 50.0;
        let calm = Conditions {
            exposure: Exposure {
                air_c: 18.0,
                ..Exposure::default()
            },
            ..Conditions::default()
        };
        run(&mut v, &cfg, &calm, 60.0);
        assert!(
            (100.0 - v.food_fp - 60.0 / 45.0).abs() < 0.01,
            "{}",
            v.food_fp
        );
        assert!((v.health - 65.0).abs() < 0.01, "{}", v.health);
        // Hungry, half the refill.
        v.food_fp = 20.0;
        let before = v.health;
        run(&mut v, &cfg, &calm, 40.0);
        assert!((v.health - before - 5.0).abs() < 0.01);
    }

    #[test]
    fn starving_drains_health_a_point_every_ten_seconds() {
        let cfg = cfg();
        let mut v = Vitals::new(&cfg);
        v.food_fp = 0.0;
        let calm = Conditions {
            exposure: Exposure {
                air_c: 18.0,
                ..Exposure::default()
            },
            ..Conditions::default()
        };
        run(&mut v, &cfg, &calm, 100.0);
        assert!((v.health - 90.0).abs() < 0.01, "{}", v.health);
    }

    #[test]
    fn a_long_dive_drains_health_and_surfacing_refills_breath() {
        let cfg = cfg();
        let mut v = Vitals::new(&cfg);
        let dive = Conditions {
            head_in_water: true,
            in_deep_water: true,
            exposure: Exposure {
                air_c: 20.0,
                in_water: true,
                ..Exposure::default()
            },
            ..Conditions::default()
        };
        run(&mut v, &cfg, &dive, 40.0);
        assert!((v.health - 50.0).abs() < 0.5, "{}", v.health);
        let surfaced = Conditions {
            head_in_water: false,
            ..dive
        };
        run(&mut v, &cfg, &surfaced, 3.0);
        assert!((v.breath.breath_s - cfg.breath.under_s).abs() < 1e-3);
    }

    #[test]
    fn a_dry_water_bar_in_the_desert_sun_passes_out() {
        // Water below `dry_below` heats the body at the dry rate. It is held
        // just above empty here; the empty bar is the next test.
        let cfg = cfg();
        let mut v = Vitals::new(&cfg);
        let mut events = Vec::new();
        let mut t = 0.0;
        while !events.contains(&Event::PassedOut) && t < 900.0 {
            v.water = 10.0;
            events.extend(v.tick(&cfg, &desert_noon(), 0.1).events);
            t += 0.1;
        }
        assert!(events.contains(&Event::PassedOut), "never passed out");
        assert!(events.contains(&Event::HeatWarning(true)));
        assert!((t / 60.0 - 7.8_f32).abs() < 0.3, "{t} s");
        assert!(v.body_c >= 40.0 && v.consciousness.is_out());
        // The blow and the exhaustion's drain.
        assert!(v.health < 100.0 - cfg.body.stroke_pts && v.health > 0.0);
        // Out for 30 s, cooling: it wakes dazed.
        let later = run(&mut v, &cfg, &desert_noon(), 30.0);
        assert!(later.contains(&Event::Woke(Wake::Time)));
        assert!(matches!(v.consciousness, Consciousness::Dazed { .. }));
        assert!(v.body_c < 40.0);
    }

    #[test]
    fn an_empty_water_bar_in_the_desert_sun_has_the_stroke_before_it_kills() {
        // design "As built": the empty bar's drain is halved (1 pt every
        // 12 s), so the stroke comes first (about 7.8 min) and the player
        // is still alive when it does.
        let cfg = cfg();
        let mut v = Vitals::new(&cfg);
        let mut events = Vec::new();
        let mut t = 0.0;
        while !events.contains(&Event::PassedOut) && !v.is_dead() && t < 900.0 {
            v.water = 0.0;
            events.extend(v.tick(&cfg, &desert_noon(), 0.1).events);
            t += 0.1;
        }
        assert!(events.contains(&Event::PassedOut), "died first at {t} s");
        assert!(!v.is_dead(), "{}", v.health);
        assert!((t / 60.0 - 7.8_f32).abs() < 0.3, "{t} s");
    }

    #[test]
    fn a_watered_body_in_the_desert_never_has_a_stroke() {
        let cfg = cfg();
        let mut v = Vitals::new(&cfg);
        for _ in 0..60 {
            let events = run(&mut v, &cfg, &desert_noon(), 60.0);
            assert!(!events.contains(&Event::PassedOut));
            // Drinking from a river now and then.
            while v.water < 90.0 {
                let sip = super::super::water::sip(
                    &cfg,
                    super::super::water::WaterSource::Fresh,
                    v.water as f32,
                );
                v.drink(&cfg, &sip);
            }
        }
        assert!(v.body_c < cfg.body.exhaustion_c && v.body_c > cfg.body.hot_c);
    }

    /// Past either end of the body's range health drains: heat exhaustion
    /// in the desert sun, hypothermia on a tundra night.
    #[test]
    fn past_either_end_health_drains() {
        let cfg = cfg();
        let mut hot = Vitals::new(&cfg);
        hot.body_c = cfg.body.exhaustion_c + 0.1;
        run(&mut hot, &cfg, &desert_noon(), 2.0);
        assert_eq!(hot.state(&cfg), BodyState::Exhaustion);
        assert!(hot.health < cfg.health.max_pts, "{}", hot.health);
        let mut cold = Vitals::new(&cfg);
        cold.body_c = cfg.body.hypothermia_c - 1.0;
        let night = Conditions {
            exposure: Exposure {
                air_c: -30.0,
                wind_mps: 8.0,
                ..Exposure::default()
            },
            ..Conditions::default()
        };
        run(&mut cold, &cfg, &night, 5.0);
        assert_eq!(cold.state(&cfg), BodyState::Hypothermia);
        assert!(cold.health < cfg.health.max_pts, "{}", cold.health);
    }

    #[test]
    fn a_hit_wakes_and_its_damage_lands() {
        let cfg = cfg();
        let mut v = Vitals::new(&cfg);
        v.consciousness = Consciousness::pass_out(&cfg);
        let events = v.hit(&cfg, Cause::Creature, 8.0);
        assert_eq!(events, vec![Event::Woke(Wake::Hit)]);
        assert_eq!(v.health, 92.0);
        assert!(matches!(v.consciousness, Consciousness::Dazed { .. }));
        // A revive with water.
        v.consciousness = Consciousness::pass_out(&cfg);
        v.water = 10.0;
        assert!(v.revive(&cfg));
        assert_eq!(v.water, 30.0);
    }

    #[test]
    fn a_stroke_in_deep_water_floats_and_does_not_drown() {
        let cfg = cfg();
        let mut v = Vitals::new(&cfg);
        v.consciousness = Consciousness::pass_out(&cfg);
        let lake = Conditions {
            head_in_water: true,
            in_deep_water: true,
            exposure: Exposure {
                air_c: 38.0,
                in_water: true,
                ..Exposure::default()
            },
            ..Conditions::default()
        };
        assert!(v.limits(&cfg, true).1.floats_face_up);
        let before = v.health;
        run(&mut v, &cfg, &lake, 29.0);
        assert_eq!(v.breath.breath_s, cfg.breath.under_s);
        assert!(v.health >= before);
    }

    #[test]
    fn a_winded_swimmer_floats_and_breathes() {
        let cfg = cfg();
        let mut v = Vitals::new(&cfg);
        let swim = Conditions {
            effort: Effort::Swim,
            in_deep_water: true,
            exposure: Exposure {
                air_c: 20.0,
                in_water: true,
                ..Exposure::default()
            },
            ..Conditions::default()
        };
        let events = run(&mut v, &cfg, &swim, 130.0);
        assert!(events.contains(&Event::Winded));
        assert!(v.limits(&cfg, true).0.floats);
        // Water over the head's cell: a winded swimmer floats head out and
        // does not drown.
        let over = Conditions {
            head_in_water: true,
            ..swim
        };
        run(&mut v, &cfg, &over, 5.0);
        assert!((v.breath.breath_s - cfg.breath.under_s).abs() < 1e-3);
        assert!(!v.is_dead());
    }

    #[test]
    fn dying_and_waking_up() {
        let cfg = cfg();
        let mut v = Vitals::new(&cfg);
        let events = v.hit(&cfg, Cause::Fall, 150.0);
        assert_eq!(events, vec![Event::Died(Cause::Fall)]);
        assert!(v.is_dead());
        v.wake_up(&cfg);
        assert_eq!((v.health, v.food_fp), (100.0, 50.0));
        assert_eq!(v.breath, Breath::full(&cfg));
    }

    #[test]
    fn the_hud_hides_full_bars_and_shows_food_and_water_from_half() {
        let cfg = cfg();
        let mut v = Vitals::new(&cfg);
        v.food_fp = 60.0;
        let s = v.shown(&cfg);
        assert!(!(s.health || s.stamina || s.food || s.water || s.thermometer || s.bubbles));
        v.food_fp = 40.0;
        assert!(v.shown(&cfg).food);
        v.body_c = 39.6;
        assert!(v.shown(&cfg).heat_warning && v.shown(&cfg).thermometer);
    }

    #[test]
    fn meals_fill_food_and_stew_fills_water() {
        let cfg = cfg();
        let mut v = Vitals::new(&cfg);
        v.food_fp = 10.0;
        v.water = 10.0;
        let e = v.eat(&cfg, "meat-stew", 1);
        assert_eq!((e.food_fp, e.water), (45.0, 20.0));
        assert_eq!((v.food_fp, v.water), (55.0, 30.0));
    }
}
