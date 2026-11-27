//! Body temperature (`survival-mode` design 13 and 14): how a place feels,
//! from the atmosphere's own sample at the player and what is around them,
//! and the body's slow approach to the temperature that feel sets.
//!
//! The shade test is a short CPU march toward the sun through the terrain
//! ([`shaded`]), never a GPU read (CLAUDE.md, "Architecture and authority").

use glam::DVec3;

use super::config::{BodyConfig, SurvivalConfig};
use crate::atmosphere::Sample;

/// What is worn in the two slots (design 13): good codes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Worn {
    pub head: Option<String>,
    pub body: Option<String>,
}

impl Worn {
    fn items(&self) -> impl Iterator<Item = &str> {
        self.head.iter().chain(self.body.iter()).map(String::as_str)
    }
}

/// A fire near the player: its `fires` source (`block:campfire`,
/// `held:torch`) and its distance, m (0 for one in hand).
#[derive(Clone, Debug, PartialEq)]
pub struct FireNear {
    pub source: String,
    pub distance_m: f32,
}

/// Everything at the player that sets how the place feels.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Exposure {
    /// The air at the ground's height, deg C (`Sample::air_c`).
    pub air_c: f32,
    /// Sunlight reaching the ground, W/m^2.
    pub sunlight_wm2: f32,
    /// Relative humidity, 0..1.
    pub humidity: f32,
    /// Surface wind speed, m/s.
    pub wind_mps: f32,
    /// Rain or snow falling on the player.
    pub rain_on: bool,
    /// A solid cell between the player and the sun ([`shaded`]).
    pub shaded: bool,
    /// The head's cell is water.
    pub in_water: bool,
    /// Under a roof with walls (no sky above; the light field's sky term
    /// below a tenth).
    pub sheltered: bool,
    /// Wet ([`Wet`]): from water or rain in the last `wet_s`.
    pub wet: bool,
    pub fires: Vec<FireNear>,
    pub worn: Worn,
}

impl Exposure {
    /// The weather's part from an atmosphere sample at the player; the rest
    /// (shade, water, shelter, fires, worn) is the caller's to set.
    pub fn from_sample(sample: &Sample) -> Self {
        Self {
            air_c: sample.air_c,
            sunlight_wm2: sample.sunlight,
            humidity: sample.humidity,
            wind_mps: sample.wind.length(),
            rain_on: sample.rain_rate > 0.0 || sample.rain_seen > 0.05,
            ..Self::default()
        }
    }
}

/// The warmth of the fires within reach, deg C: each fire's `c` falling in a
/// straight line to 0 at its reach; a fire in hand (reach 0) counts whole.
pub fn fire_warmth(cfg: &BodyConfig, fires: &[FireNear]) -> f32 {
    fires
        .iter()
        .filter_map(|near| {
            let heat = cfg.fires.iter().find(|f| f.source == near.source)?;
            if heat.reach_m <= 0.0 {
                return Some(heat.c);
            }
            let k = 1.0 - near.distance_m.max(0.0) / heat.reach_m;
            (k > 0.0).then_some(heat.c * k)
        })
        .sum()
}

/// Whether any fire listed is within its reach (a wet body dries faster
/// there).
pub fn near_fire(cfg: &BodyConfig, fires: &[FireNear]) -> bool {
    fire_warmth(cfg, fires) > 0.0
}

/// How the place feels, deg C (design 13's table).
pub fn feel(cfg: &BodyConfig, e: &Exposure) -> f32 {
    let f = &cfg.feel;
    let worn: Vec<_> = e
        .worn
        .items()
        .filter_map(|item| cfg.worn.iter().find(|w| w.item == item))
        .collect();
    let mut c = if e.in_water {
        e.air_c.clamp(f.water_c.0, f.water_c.1) + f.in_water_c
    } else {
        let mut c = e.air_c;
        if !e.shaded {
            let sun_mult: f32 = worn.iter().map(|w| w.sun_mult).product();
            c += f.sun_c_at_1000_wm2 * (e.sunlight_wm2.max(0.0) / 1000.0) * sun_mult;
        }
        if e.air_c > f.humid_above_c {
            c += (e.humidity - 0.5).max(0.0) * f.humid_c_per_unit;
        }
        if e.air_c < f.chill_below_c {
            c -= f.chill_c_per_mps * (e.wind_mps - f.chill_above_mps).max(0.0);
        }
        if e.rain_on {
            c += f.rain_c;
        }
        if e.wet {
            c += f.wet_c;
        }
        c
    };
    c += fire_warmth(cfg, &e.fires);
    if e.sheltered {
        c += (f.shelter_c - c) * f.shelter_pull;
    }
    for w in &worn {
        if c < f.shelter_c {
            c += w.cold_c;
        } else if c > cfg.comfort_c.1 {
            c += w.hot_c;
        }
    }
    c
}

/// The temperature the body heads for at `feel_c` (design 13). `dry` is
/// water below `water.dry_below`; `hard` a sprint or a hard swim; `passed_out`
/// holds it at `pass_out.cool_to_c` at most.
pub fn target_c(cfg: &SurvivalConfig, feel_c: f32, dry: bool, hard: bool, passed_out: bool) -> f32 {
    let b = &cfg.body;
    let (lo, hi) = b.comfort_c;
    let mut t = if feel_c > hi {
        b.start_c
            + (feel_c - hi)
                * if dry {
                    b.hot_dry_c_per_c
                } else {
                    b.hot_c_per_c
                }
    } else if feel_c < lo {
        b.start_c - (lo - feel_c) * b.cold_c_per_c
    } else {
        b.start_c
    };
    if hard && feel_c > hi {
        t += b.exertion_c;
    }
    if passed_out {
        t = t.min(cfg.pass_out.cool_to_c);
    }
    t
}

/// The body after `dt` seconds of play heading for `target_c`, with the time
/// constant `body.time_constant_s`: exact for a target held over the step.
pub fn approach(cfg: &BodyConfig, body_c: f32, target_c: f32, dt: f32) -> f32 {
    target_c + (body_c - target_c) * (-dt / cfg.time_constant_s).exp()
}

/// The body's state (design 14's table).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BodyState {
    Fine,
    /// Water used faster.
    Hot,
    /// Health drains; the screen's edge warms; the HUD's heat stroke warning.
    Exhaustion,
    /// At or past `stroke_c`: the blow and the pass-out.
    Stroke,
    Cool,
    /// Stamina refills at half rate; food burns faster (shivering).
    Cold,
    /// As cold, and health drains.
    Hypothermia,
}

impl BodyState {
    pub fn of(cfg: &BodyConfig, body_c: f32) -> Self {
        if body_c >= cfg.stroke_c {
            BodyState::Stroke
        } else if body_c >= cfg.exhaustion_c {
            BodyState::Exhaustion
        } else if body_c >= cfg.hot_c {
            BodyState::Hot
        } else if body_c < cfg.hypothermia_c {
            BodyState::Hypothermia
        } else if body_c < cfg.cold_c {
            BodyState::Cold
        } else if body_c < cfg.cool_c {
            BodyState::Cool
        } else {
            BodyState::Fine
        }
    }

    /// Shivering: cold or worse.
    pub fn shivering(self) -> bool {
        matches!(self, BodyState::Cold | BodyState::Hypothermia)
    }

    /// The heat stroke warning above the hotbar (design 9, G8).
    pub fn heat_warning(self) -> bool {
        matches!(self, BodyState::Exhaustion | BodyState::Stroke)
    }

    /// The thermometer shows outside fine (design 9).
    pub fn shown(self) -> bool {
        !matches!(self, BodyState::Fine | BodyState::Cool)
    }
}

/// Being wet (design 13): for `wet_s` after leaving water or rain, drying
/// faster within a fire's reach.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Wet {
    pub left_s: f32,
}

impl Wet {
    /// `dt` of play: soaked again while in water or rain, drying otherwise.
    pub fn step(&mut self, cfg: &BodyConfig, soaking: bool, by_fire: bool, dt: f32) {
        if soaking {
            self.left_s = cfg.feel.wet_s;
        } else {
            let rate = if by_fire {
                cfg.feel.wet_fire_dry_mult
            } else {
                1.0
            };
            self.left_s = (self.left_s - dt * rate).max(0.0);
        }
    }

    pub fn is_wet(self) -> bool {
        self.left_s > 0.0
    }
}

/// The shade test (design 13): whether a solid cell stands between the
/// point `from` and the sun within `shade_march_layers` layers of height.
/// `from`, `sun` and `up` are in one frame (the body's, in f64: a query,
/// never a stored state); `sun` points toward the sun. `solid` answers for a
/// point, from the column tier. A sun at or below the horizon is shade.
pub fn shaded(
    cfg: &BodyConfig,
    from: DVec3,
    sun: DVec3,
    up: DVec3,
    mut solid: impl FnMut(DVec3) -> bool,
) -> bool {
    let sun = sun.normalize_or_zero();
    let rise = sun.dot(up);
    if rise <= 0.0 {
        return true;
    }
    let f = &cfg.feel;
    let top = f64::from(f.shade_march_layers);
    let step = f64::from(f.shade_step_m);
    let max = f64::from(f.shade_max_m);
    let mut along = step;
    while along <= max && along * rise <= top {
        if solid(from + sun * along) {
            return true;
        }
        along += step;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn desert() -> Exposure {
        Exposure {
            air_c: 38.0,
            sunlight_wm2: 900.0,
            humidity: 0.15,
            wind_mps: 2.0,
            ..Exposure::default()
        }
    }

    fn tundra() -> Exposure {
        Exposure {
            air_c: -18.0,
            wind_mps: 6.0,
            humidity: 0.6,
            ..Exposure::default()
        }
    }

    /// Where the body settles: the target, as `approach` reaches it.
    fn settles(cfg: &SurvivalConfig, e: &Exposure, dry: bool) -> f32 {
        target_c(cfg, feel(&cfg.body, e), dry, false, false)
    }

    #[test]
    fn the_desert_noon_example() {
        let cfg = SurvivalConfig::shipped();
        assert!((feel(&cfg.body, &desert()) - 43.4).abs() < 0.01);
        // Watered: hot, never a stroke.
        assert!((settles(&cfg, &desert(), false) - 39.1).abs() < 0.05);
        // Out of water it heads for 40.5 and reaches 40.0 in about 7.8 min.
        let dry = settles(&cfg, &desert(), true);
        assert!((dry - 40.5).abs() < 0.05);
        let (mut body, mut t) = (37.0, 0.0);
        while body < 40.0 {
            body = approach(&cfg.body, body, dry, 1.0);
            t += 1.0;
        }
        assert!((t / 60.0 - 7.8_f32).abs() < 0.2, "{t} s");
    }

    #[test]
    fn shade_is_cooler_than_sun() {
        let cfg = SurvivalConfig::shipped();
        let shade = Exposure {
            shaded: true,
            ..desert()
        };
        let in_shade = settles(&cfg, &shade, false);
        assert!((in_shade - 38.4).abs() < 0.05, "{in_shade}");
        assert!(in_shade < settles(&cfg, &desert(), false));
        // A straw hat halves the sun.
        let hat = Exposure {
            worn: Worn {
                head: Some("straw-hat".into()),
                body: None,
            },
            ..desert()
        };
        let f = feel(&cfg.body, &hat);
        assert!((f - 40.7).abs() < 0.01, "{f}");
    }

    #[test]
    fn the_tundra_night_example_a_cloak_and_a_fire_each_warmer() {
        let cfg = SurvivalConfig::shipped();
        let bare = tundra();
        assert!((feel(&cfg.body, &bare) - -20.0).abs() < 0.01);
        let cloak = Exposure {
            worn: Worn {
                head: None,
                body: Some("wool-cloak".into()),
            },
            ..tundra()
        };
        let fire = Exposure {
            fires: vec![FireNear {
                source: "block:campfire".into(),
                distance_m: 2.0,
            }],
            ..cloak.clone()
        };
        let (a, b, c) = (
            settles(&cfg, &bare, false),
            settles(&cfg, &cloak, false),
            settles(&cfg, &fire, false),
        );
        assert!((a - 34.0).abs() < 0.05 && (b - 35.0).abs() < 0.05 && (c - 36.1).abs() < 0.05);
        assert!(a < b && b < c);
        assert_eq!(BodyState::of(&cfg.body, a), BodyState::Hypothermia);
        assert_eq!(BodyState::of(&cfg.body, b), BodyState::Cold);
        assert_eq!(BodyState::of(&cfg.body, c), BodyState::Fine);
    }

    #[test]
    fn water_and_wet_and_rain_cool() {
        let cfg = SurvivalConfig::shipped();
        let swim = Exposure {
            in_water: true,
            ..desert()
        };
        assert_eq!(feel(&cfg.body, &swim), 24.0);
        let mut wet = Wet::default();
        wet.step(&cfg.body, true, false, 1.0);
        assert!(wet.is_wet());
        wet.step(&cfg.body, false, true, 60.0);
        assert!(!wet.is_wet());
        let rain = Exposure {
            rain_on: true,
            ..tundra()
        };
        assert!(feel(&cfg.body, &rain) < feel(&cfg.body, &tundra()));
    }

    #[test]
    fn the_shade_march_finds_a_rock_toward_the_sun() {
        let cfg = SurvivalConfig::shipped();
        let up = DVec3::Y;
        let sun = DVec3::new(1.0, 1.0, 0.0);
        // A rock overhang 3 m up and 3 m sunward.
        let rock = |p: DVec3| (2.5..3.5).contains(&p.x) && (2.5..3.5).contains(&p.y);
        assert!(shaded(&cfg.body, DVec3::ZERO, sun, up, rock));
        assert!(!shaded(&cfg.body, DVec3::ZERO, sun, up, |_| false));
        // Past 8 layers up nothing counts.
        let high = |p: DVec3| p.y > 9.0;
        assert!(!shaded(&cfg.body, DVec3::ZERO, sun, up, high));
        // Night is shade.
        assert!(shaded(&cfg.body, DVec3::ZERO, -sun, up, |_| false));
    }
}
