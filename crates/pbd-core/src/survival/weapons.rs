//! Weapons (`survival-mode` design 16): the damage table, and a thrown
//! spear's and an arrow's flight.
//!
//! A projectile flies in the body's frame in f64 (CLAUDE.md, "All position
//! math stays in the floating origin"): its position is a [`WorldPos`]
//! relative to the body's centre, its velocity in that frame's axes, and up
//! and gravity come from that f64 point before anything is cast. The game
//! converts to the local frame only to draw it and to test it against the
//! creatures and the column tier, through `SimOrigin`.

use glam::DVec3;

use super::config::SurvivalConfig;
use crate::inventory::{Tier, Tool};
use crate::world_frame::WorldPos;

/// What is in the hand when a swing lands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Held {
    Hand,
    Tool(Tool, Tier),
    /// A melee weapon of `weapons.melee`, by good code.
    Weapon(String),
    /// A worn-out tool or weapon: it hits as bare hands.
    Broken,
}

/// A swing's damage at its hit moment.
pub fn melee_damage(cfg: &SurvivalConfig, held: &Held) -> f32 {
    let w = &cfg.weapons;
    match held {
        Held::Hand | Held::Broken | Held::Tool(Tool::Rod, _) => w.hand,
        Held::Tool(tool, tier) => {
            let axe = w.axe_by_tier[usize::from(tier.digit())];
            match tool {
                Tool::Axe => axe,
                Tool::Pickaxe => axe * w.pickaxe_share,
                Tool::Shovel => axe * w.shovel_share,
                Tool::Rod => w.hand,
            }
        }
        Held::Weapon(code) => w.melee.get(code).map_or(w.hand, |m| m.damage),
    }
}

/// A swing's reach, m.
pub fn melee_reach_m(cfg: &SurvivalConfig, held: &Held) -> f32 {
    match held {
        Held::Weapon(code) => cfg.weapons.melee.get(code).map_or(1.6, |m| m.reach_m),
        _ => 1.6,
    }
}

/// What flies.
#[derive(Clone, Debug, PartialEq)]
pub enum Missile {
    /// A thrown spear: always lands as a drop.
    Spear { code: String, damage: f32 },
    /// An arrow: kept `bow.kept_share` of the time where it lands.
    Arrow { code: String, damage: f32 },
}

impl Missile {
    pub fn damage(&self) -> f32 {
        match self {
            Missile::Spear { damage, .. } | Missile::Arrow { damage, .. } => *damage,
        }
    }

    /// The good it is, for the drop it lands as.
    pub fn code(&self) -> &str {
        match self {
            Missile::Spear { code, .. } | Missile::Arrow { code, .. } => code,
        }
    }
}

/// A projectile in flight, in a body's frame.
#[derive(Clone, Debug, PartialEq)]
pub struct Projectile {
    pub missile: Missile,
    /// Where it is: body-relative, f64.
    pub at: WorldPos,
    /// Its velocity in the body frame's axes, m/s.
    pub vel: DVec3,
    /// Seconds of flight so far.
    pub flown_s: f64,
    /// The launch, for the keep-or-break hash.
    pub launched: (DVec3, DVec3),
}

/// What a projectile met.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Struck<T> {
    /// A creature, at a point along the step.
    Creature(T, DVec3),
    /// The ground (the column tier), at a point.
    Ground(DVec3),
}

/// The end of a flight.
#[derive(Clone, Debug, PartialEq)]
pub enum Flight<T> {
    Flying,
    /// It hit a creature for its damage. An arrow or a spear that hits drops
    /// where the creature stood, kept as a landing is.
    Hit {
        creature: T,
        damage: f32,
        drop: Option<WorldPos>,
    },
    /// It landed: a drop where it stopped, or broken.
    Landed {
        drop: Option<WorldPos>,
    },
    /// It flew past `flight_max_s` and is given up.
    Lost,
}

/// The body's up at a body-frame point, f64.
fn up_at(body_point: DVec3) -> DVec3 {
    DVec3::normalize_or_zero(body_point)
}

impl Projectile {
    /// A spear thrown from `from` toward `dir` (body frame), at
    /// `spear_throw_mps`.
    pub fn throw_spear(
        cfg: &SurvivalConfig,
        code: &str,
        from: WorldPos,
        dir: DVec3,
    ) -> Option<Self> {
        let damage = cfg.weapons.melee.get(code)?.thrown?;
        Some(Self::launch(
            Missile::Spear {
                code: code.into(),
                damage,
            },
            from,
            dir.normalize_or_zero() * f64::from(cfg.weapons.spear_throw_mps),
        ))
    }

    /// An arrow loosed after `drawn_s` of draw: its speed and damage scale
    /// by the share of a full draw.
    pub fn loose_arrow(
        cfg: &SurvivalConfig,
        code: &str,
        from: WorldPos,
        dir: DVec3,
        drawn_s: f32,
    ) -> Option<Self> {
        let full = *cfg.weapons.arrows.get(code)?;
        let share = f64::from((drawn_s / cfg.weapons.bow.draw_s).clamp(0.0, 1.0));
        Some(Self::launch(
            Missile::Arrow {
                code: code.into(),
                damage: full * share as f32,
            },
            from,
            dir.normalize_or_zero() * f64::from(cfg.weapons.bow.arrow_mps) * share,
        ))
    }

    fn launch(missile: Missile, from: WorldPos, vel: DVec3) -> Self {
        Self {
            missile,
            at: from,
            vel,
            flown_s: 0.0,
            launched: (from.pos, vel),
        }
    }

    /// Whether it is kept where it lands: a spear always, an arrow
    /// `bow.kept_share` of the time, by a hash of its flight so a replay
    /// agrees.
    pub fn kept(&self, cfg: &SurvivalConfig) -> bool {
        match self.missile {
            Missile::Spear { .. } => true,
            Missile::Arrow { .. } => {
                let (p, v) = self.launched;
                let mut h: u64 = 0xcbf2_9ce4_8422_2325;
                for x in [p.x, p.y, p.z, v.x, v.y, v.z] {
                    for b in x.to_bits().to_le_bytes() {
                        h ^= u64::from(b);
                        h = h.wrapping_mul(0x0100_0000_01b3);
                    }
                }
                ((h >> 11) as f64 / (1u64 << 53) as f64) < f64::from(cfg.weapons.bow.kept_share)
            }
        }
    }

    /// One step of `dt` seconds under the body's surface gravity
    /// `gravity_mps2`, pulling toward its centre. `hit` tests the segment
    /// from the old point to the new, both in the body frame, against the
    /// creatures and the ground, and answers the first thing on it.
    pub fn step<T>(
        &mut self,
        cfg: &SurvivalConfig,
        gravity_mps2: f64,
        dt: f64,
        mut hit: impl FnMut(DVec3, DVec3) -> Option<Struck<T>>,
    ) -> Flight<T> {
        let from = self.at.pos;
        let accel = -up_at(from) * gravity_mps2;
        let to = from + self.vel * dt + accel * (0.5 * dt * dt);
        self.vel += accel * dt;
        self.flown_s += dt;
        let frame = self.at.frame;
        let drop_at = |p: DVec3, kept: bool| kept.then_some(WorldPos::new(frame, p));
        match hit(from, to) {
            Some(Struck::Creature(creature, p)) => {
                self.at = WorldPos::new(frame, p);
                Flight::Hit {
                    creature,
                    damage: self.missile.damage(),
                    drop: drop_at(p, self.kept(cfg)),
                }
            }
            Some(Struck::Ground(p)) => {
                self.at = WorldPos::new(frame, p);
                Flight::Landed {
                    drop: drop_at(p, self.kept(cfg)),
                }
            }
            None => {
                self.at = WorldPos::new(frame, to);
                if self.flown_s > f64::from(cfg.weapons.flight_max_s) {
                    Flight::Lost
                } else {
                    Flight::Flying
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world_frame::FrameId;

    #[test]
    fn the_damage_table() {
        let cfg = SurvivalConfig::shipped();
        let d = |h: Held| melee_damage(&cfg, &h);
        assert_eq!(d(Held::Hand), 3.0);
        assert_eq!(d(Held::Broken), 3.0);
        assert_eq!(d(Held::Tool(Tool::Axe, Tier::Stone)), 10.0);
        assert_eq!(d(Held::Tool(Tool::Axe, Tier::Iron)), 18.0);
        assert!((d(Held::Tool(Tool::Pickaxe, Tier::Bronze)) - 9.8).abs() < 1e-5);
        assert_eq!(d(Held::Tool(Tool::Shovel, Tier::Stone)), 5.0);
        assert_eq!(d(Held::Weapon("stone-spear".into())), 14.0);
        assert_eq!(d(Held::Weapon("bronze-hunting-axe".into())), 24.0);
        assert_eq!(d(Held::Weapon("iron-sword".into())), 36.0);
        assert_eq!(
            melee_reach_m(&cfg, &Held::Weapon("stone-spear".into())),
            2.6
        );
    }

    /// A level arrow at full draw from 1.6 m over a body of `radius` and
    /// `g`; how far it fell after `t`.
    fn drop_after(radius: f64, g: f64, t: f64) -> f64 {
        let cfg = SurvivalConfig::shipped();
        let from = WorldPos::new(FrameId(2), DVec3::new(0.0, radius + 1.6, 0.0));
        let mut a = Projectile::loose_arrow(&cfg, "stone-arrow", from, DVec3::X, 0.8).unwrap();
        assert_eq!(a.missile.damage(), 18.0);
        let dt = 1.0 / 60.0;
        for _ in 0..(t / dt).round() as usize {
            assert_eq!(a.step::<()>(&cfg, g, dt, |_, _| None), Flight::Flying);
        }
        // Measured along the launch's up (the body's +Y here).
        radius + 1.6 - a.at.pos.y
    }

    #[test]
    fn an_arrow_drops_by_its_bodys_gravity_on_two_bodies() {
        for (radius, g) in [(9600.0, 25.0), (2400.0, 12.5)] {
            let t = 0.5;
            let fell = drop_after(radius, g, t);
            let want = 0.5 * g * t * t;
            assert!(
                (fell - want).abs() < 0.01 * want,
                "fell {fell}, want {want}"
            );
        }
        assert!(drop_after(9600.0, 25.0, 0.5) > drop_after(2400.0, 12.5, 0.5) * 1.9);
    }

    #[test]
    fn a_part_draw_is_slower_and_weaker() {
        let cfg = SurvivalConfig::shipped();
        let from = WorldPos::new(FrameId(2), DVec3::new(0.0, 9601.6, 0.0));
        let a = Projectile::loose_arrow(&cfg, "bone-arrow", from, DVec3::X, 0.4).unwrap();
        assert_eq!(a.missile.damage(), 11.0);
        assert!((a.vel.length() - 22.5).abs() < 1e-9);
    }

    #[test]
    fn a_thrown_spear_hits_and_lands_as_a_drop() {
        let cfg = SurvivalConfig::shipped();
        let from = WorldPos::new(FrameId(2), DVec3::new(0.0, 9601.6, 0.0));
        let mut s = Projectile::throw_spear(&cfg, "stone-spear", from, DVec3::X).unwrap();
        let mut out = Flight::Flying;
        for _ in 0..120 {
            out = s.step(&cfg, 25.0, 1.0 / 60.0, |a: DVec3, b: DVec3| {
                (a.x < 5.0 && b.x >= 5.0).then_some(Struck::Creature(7u32, b))
            });
            if out != Flight::Flying {
                break;
            }
        }
        match out {
            Flight::Hit {
                creature,
                damage,
                drop,
            } => {
                assert_eq!((creature, damage), (7, 22.0));
                assert_eq!(drop.unwrap().frame, FrameId(2));
            }
            other => panic!("{other:?}"),
        }
        assert!(Projectile::throw_spear(&cfg, "bronze-sword", from, DVec3::X).is_none());
    }

    #[test]
    fn about_seven_arrows_in_ten_are_kept_and_a_replay_agrees() {
        let cfg = SurvivalConfig::shipped();
        let mut kept = 0;
        for i in 0..1000 {
            let from = WorldPos::new(FrameId(2), DVec3::new(f64::from(i) * 0.37, 9601.6, 0.0));
            let a = Projectile::loose_arrow(&cfg, "stone-arrow", from, DVec3::X, 0.8).unwrap();
            assert_eq!(a.kept(&cfg), a.clone().kept(&cfg));
            kept += usize::from(a.kept(&cfg));
        }
        assert!((640..760).contains(&kept), "{kept}");
    }
}
