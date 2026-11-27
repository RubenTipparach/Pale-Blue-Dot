//! The held weapons' swings (`survival-mode` combat C1, task 1.6), ported
//! from the drafted Weapons tab's In hand view (`docs/mockups/survival.html`
//! `#weapons`, `survival/swing.js`: `DEF`, `times`, `poseAt`, `place`).
//!
//! Four stages by the `animation` skill: windup (eased out), anticipation (a
//! slow ease into a held pose), the hit (t cubed: fastest at contact, then a
//! held frame) and the recoil (a quick kick with an overshoot), then the ease
//! home. Keyed poses on the fist's arc about the stroke's virtual shoulder,
//! the weapon's own pitch, yaw and roll on top. The rule owns when the hit
//! lands (`pbd_core::survival::melee`); a melee swing's stage times are
//! fitted to it, a held one (a thrown spear, the bow) runs as written with
//! its release where the button was let go. Drawn only: nothing reads it.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// One keyed pose: the arm raised about the shoulder (`phi`, rad, up
/// positive) and swung across (`yaw`, rad, left positive), the fist moved
/// along the aim (`pull_m`, ahead positive), and the weapon's own pitch (tip
/// up and back positive), yaw and roll (tip left positive), rad.
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Key {
    pub phi: f32,
    pub yaw: f32,
    pub pull_m: f32,
    pub pitch: f32,
    pub roll: f32,
}

impl Key {
    const fn new(phi: f32, yaw: f32, pull_m: f32, pitch: f32, roll: f32) -> Self {
        Self {
            phi,
            yaw,
            pull_m,
            pitch,
            roll,
        }
    }

    fn lerp(self, to: Self, u: f32) -> Self {
        let l = |a: f32, b: f32| a + (b - a) * u;
        Self {
            phi: l(self.phi, to.phi),
            yaw: l(self.yaw, to.yaw),
            pull_m: l(self.pull_m, to.pull_m),
            pitch: l(self.pitch, to.pitch),
            roll: l(self.roll, to.roll),
        }
    }

    fn scaled(self, k: f32) -> Self {
        Self {
            phi: self.phi * k,
            yaw: self.yaw * k,
            pull_m: self.pull_m * k,
            pitch: self.pitch * k,
            roll: self.roll * k,
        }
    }

    fn finite(&self) -> bool {
        [self.phi, self.yaw, self.pull_m, self.pitch, self.roll]
            .iter()
            .all(|v| v.is_finite() && v.abs() < 3.2)
    }
}

/// One swing: its stage times, s, and its keyed poses.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Swing {
    pub windup_s: f32,
    pub anticipate_s: f32,
    pub strike_s: f32,
    pub impact_hold_s: f32,
    pub recoil_s: f32,
    pub recover_s: f32,
    pub top: Key,
    pub coil: Key,
    pub hit: Key,
    pub recoil: Key,
}

impl Swing {
    fn validate(&self, name: &str) -> Result<(), String> {
        let times = [
            self.windup_s,
            self.anticipate_s,
            self.strike_s,
            self.impact_hold_s,
            self.recoil_s,
            self.recover_s,
        ];
        if times.iter().any(|t| !t.is_finite() || *t < 0.0 || *t > 2.0)
            || self.windup_s <= 0.0
            || self.strike_s <= 0.0
        {
            return Err(format!(
                "held.weapon_swing.{name}: times 0..2 s, windup and strike above 0"
            ));
        }
        if ![self.top, self.coil, self.hit, self.recoil]
            .iter()
            .all(Key::finite)
        {
            return Err(format!(
                "held.weapon_swing.{name}: every key finite and under 3.2"
            ));
        }
        Ok(())
    }
}

/// The swings of `held.ron`'s `weapon_swing`: the drafted mockup's values.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct WeaponSwings {
    pub spear: Swing,
    pub axe: Swing,
    pub sword: Swing,
    pub throw: Swing,
    pub bow: Swing,
    /// Scales the raise, the coil and the recoil together.
    pub exaggerate: f32,
}

impl Default for WeaponSwings {
    fn default() -> Self {
        let k = Key::new;
        Self {
            spear: Swing {
                windup_s: 0.14,
                anticipate_s: 0.10,
                strike_s: 0.07,
                impact_hold_s: 0.05,
                recoil_s: 0.08,
                recover_s: 0.26,
                top: k(0.04, 0.06, -0.22, -0.58, 0.0),
                coil: k(0.06, 0.08, -0.28, -0.62, 0.0),
                hit: k(-0.05, -0.08, 0.4, -0.74, 0.0),
                recoil: k(-0.03, -0.05, 0.3, -0.7, 0.0),
            },
            axe: Swing {
                windup_s: 0.11,
                anticipate_s: 0.07,
                strike_s: 0.07,
                impact_hold_s: 0.05,
                recoil_s: 0.07,
                recover_s: 0.23,
                top: k(0.55, -0.1, -0.08, 0.95, 0.1),
                coil: k(0.62, -0.12, -0.1, 1.08, 0.12),
                hit: k(-0.3, 0.14, 0.14, -1.45, -0.05),
                recoil: k(-0.2, 0.1, 0.1, -1.15, 0.0),
            },
            sword: Swing {
                windup_s: 0.09,
                anticipate_s: 0.06,
                strike_s: 0.06,
                impact_hold_s: 0.03,
                recoil_s: 0.08,
                recover_s: 0.18,
                top: k(0.4, -0.5, -0.06, 0.55, -0.85),
                coil: k(0.46, -0.6, -0.08, 0.65, -0.95),
                hit: k(-0.25, 0.7, 0.12, -1.1, 0.7),
                recoil: k(-0.32, 0.85, 0.08, -1.2, 0.8),
            },
            throw: Swing {
                windup_s: 0.25,
                anticipate_s: 0.15,
                strike_s: 0.08,
                impact_hold_s: 0.0,
                recoil_s: 0.1,
                recover_s: 0.3,
                top: k(0.42, -0.08, -0.2, -0.5, 0.0),
                coil: k(0.46, -0.1, -0.28, -0.52, 0.0),
                hit: k(0.12, 0.2, 0.35, -0.72, 0.0),
                recoil: k(-0.2, 0.28, 0.2, -0.9, 0.0),
            },
            bow: Swing {
                windup_s: 0.15,
                anticipate_s: 0.8,
                strike_s: 0.04,
                impact_hold_s: 0.02,
                recoil_s: 0.1,
                recover_s: 0.3,
                top: k(0.12, 0.36, 0.08, 0.32, -0.12),
                coil: k(0.14, 0.36, 0.06, 0.32, -0.12),
                hit: k(0.15, 0.36, 0.12, 0.34, -0.12),
                recoil: k(0.08, 0.34, 0.06, 0.32, -0.1),
            },
            exaggerate: 1.0,
        }
    }
}

impl WeaponSwings {
    pub fn validate(&self) -> Result<(), String> {
        self.spear.validate("spear")?;
        self.axe.validate("axe")?;
        self.sword.validate("sword")?;
        self.throw.validate("throw")?;
        self.bow.validate("bow")?;
        if !(0.25..=2.0).contains(&self.exaggerate) {
            return Err("held.weapon_swing.exaggerate: 0.25..2".into());
        }
        Ok(())
    }

    pub fn of(&self, kind: SwingKind) -> &Swing {
        match kind {
            SwingKind::Thrust => &self.spear,
            SwingKind::Chop => &self.axe,
            SwingKind::Slash => &self.sword,
            SwingKind::Throw => &self.throw,
            SwingKind::Bow => &self.bow,
        }
    }
}

/// Which swing a weapon makes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SwingKind {
    /// A spear held: the thrust.
    Thrust,
    /// A hunting axe: the overhand chop.
    Chop,
    /// A sword: the slash.
    Slash,
    /// A spear raised and thrown.
    Throw,
    /// The bow drawn and loosed.
    Bow,
}

/// Where in a swing the weapon is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Rest,
    Windup,
    Anticipation,
    Hit,
    Recoil,
    Recover,
}

/// What the rule says of the swing under way, as the drawing reads it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Timing {
    /// A melee swing: seconds since the press, its period and its hit, s.
    Melee { t: f32, period_s: f32, hit_s: f32 },
    /// A spear raised or a bow drawn: seconds since the press, and when the
    /// button was let go (none while it is held).
    Held { t: f32, release_s: Option<f32> },
}

/// The swing under way, as the rule (`crate::combat`) last said: which
/// swing and where in it. Written by the rule, read only to draw.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct SwingNow(pub Option<(SwingKind, Timing)>);

/// The stage times a swing is drawn with: a melee one fitted so its hit's
/// end lands on the rule's hit, the hit itself never shortened.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Times {
    pub windup: f32,
    pub anticipate: f32,
    pub strike: f32,
    pub hold: f32,
    pub recoil: f32,
    pub recover: f32,
    /// When the hit stage starts, s, if it does yet.
    pub release: Option<f32>,
    /// When the whole swing ends, s.
    pub period: f32,
}

impl Times {
    pub fn of(s: &Swing, timing: Timing) -> Self {
        let mut t = Self {
            windup: s.windup_s,
            anticipate: s.anticipate_s,
            strike: s.strike_s,
            hold: s.impact_hold_s,
            recoil: s.recoil_s,
            recover: s.recover_s,
            release: None,
            period: f32::INFINITY,
        };
        match timing {
            Timing::Melee {
                period_s, hit_s, ..
            } => {
                let k1 = ((hit_s - t.strike) / (t.windup + t.anticipate).max(1e-3)).max(0.1);
                let post = (t.hold + t.recoil + t.recover).max(1e-3);
                let k2 = ((period_s - hit_s) / post).max(0.1);
                t.windup *= k1;
                t.anticipate *= k1;
                t.hold *= k2;
                t.recoil *= k2;
                t.recover *= k2;
                t.release = Some(t.windup + t.anticipate);
                t.period = period_s;
            }
            Timing::Held { release_s, .. } => {
                t.release = release_s;
                if let Some(r) = release_s {
                    t.period = r + t.strike + t.hold + t.recoil + t.recover;
                }
            }
        }
        t
    }
}

fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn back_out(t: f32) -> f32 {
    let c = 1.7;
    let u = t - 1.0;
    1.0 + (c + 1.0) * u * u * u + c * u * u
}

/// The pose at a moment of a swing, its stage, and how far a held swing is
/// drawn (0..1 over its anticipation).
pub fn pose_at(s: &Swing, exaggerate: f32, timing: Timing) -> (Key, Stage, f32) {
    let t = match timing {
        Timing::Melee { t, .. } | Timing::Held { t, .. } => t,
    };
    let tm = Times::of(s, timing);
    let (top, coil, rec) = (
        s.top.scaled(exaggerate),
        s.coil.scaled(exaggerate),
        s.recoil.scaled(exaggerate),
    );
    let rest = Key::default();
    if t < 0.0 || t >= tm.period {
        return (rest, Stage::Rest, 0.0);
    }
    if t < tm.windup {
        return (rest.lerp(top, ease_out(t / tm.windup)), Stage::Windup, 0.0);
    }
    let release = tm.release.unwrap_or(f32::INFINITY);
    if t < release {
        let u = ((t - tm.windup) / tm.anticipate.max(1e-3)).min(1.0);
        return (top.lerp(coil, smooth(u)), Stage::Anticipation, u);
    }
    let mut e = t - release;
    if e < tm.strike {
        return (coil.lerp(s.hit, (e / tm.strike).powi(3)), Stage::Hit, 1.0);
    }
    e -= tm.strike;
    if e < tm.hold {
        return (s.hit, Stage::Hit, 1.0);
    }
    e -= tm.hold;
    if e < tm.recoil {
        return (s.hit.lerp(rec, back_out(e / tm.recoil)), Stage::Recoil, 0.0);
    }
    e -= tm.recoil;
    (
        rec.lerp(rest, smooth((e / tm.recover.max(1e-3)).min(1.0))),
        Stage::Recover,
        0.0,
    )
}

/// The aim the pull runs along, eye space: the mockup's view, a little down.
fn aim() -> Vec3 {
    Vec3::new(0.0, -0.18, -1.0).normalize()
}

/// A key applied to a rest pose: the fist on its arc about `shoulder` and
/// pulled along the aim, the weapon turned about it.
pub fn place(key: Key, rest: Transform, fist: Vec3, shoulder: Vec3) -> Transform {
    let arm = Quat::from_rotation_y(key.yaw) * Quat::from_rotation_x(key.phi);
    let moved = shoulder + arm * (fist - shoulder) + aim() * key.pull_m;
    let turn = Quat::from_rotation_y(key.yaw)
        * Quat::from_rotation_x(key.pitch)
        * Quat::from_rotation_z(key.roll);
    Transform {
        translation: moved + turn * (rest.translation - fist),
        rotation: turn * rest.rotation,
        scale: rest.scale,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn melee(t: f32) -> Timing {
        Timing::Melee {
            t,
            period_s: 0.7,
            hit_s: 0.7 * 0.45,
        }
    }

    #[test]
    fn the_hit_stage_ends_on_the_rules_hit() {
        let s = WeaponSwings::default();
        let tm = Times::of(&s.spear, melee(0.0));
        let end = tm.release.unwrap() + tm.strike;
        assert!((end - 0.315).abs() < 1e-5, "{end}");
        assert!(
            (tm.strike - s.spear.strike_s).abs() < 1e-6,
            "the hit is never shortened"
        );
        assert_eq!(pose_at(&s.spear, 1.0, melee(0.30)).1, Stage::Hit);
        assert_eq!(pose_at(&s.spear, 1.0, melee(0.32)).1, Stage::Hit);
        assert_eq!(pose_at(&s.spear, 1.0, melee(0.32)).0, s.spear.hit);
        assert_eq!(pose_at(&s.spear, 1.0, melee(0.1)).1, Stage::Windup);
        assert_eq!(pose_at(&s.spear, 1.0, melee(0.69)).1, Stage::Recover);
        assert_eq!(pose_at(&s.spear, 1.0, melee(0.71)).1, Stage::Rest);
    }

    #[test]
    fn the_hit_is_fastest_at_contact() {
        let s = WeaponSwings::default().axe;
        let timing = |t| Timing::Melee {
            t,
            period_s: 0.6,
            hit_s: 0.27,
        };
        let tm = Times::of(&s, timing(0.0));
        let r = tm.release.unwrap();
        let at = |t: f32| pose_at(&s, 1.0, timing(t)).0.pitch;
        let early = (at(r + 0.01) - at(r)).abs();
        let late = (at(r + tm.strike - 0.001) - at(r + tm.strike - 0.011)).abs();
        assert!(late > early * 5.0, "early {early}, late {late}");
    }

    #[test]
    fn nothing_pops_from_stage_to_stage() {
        let sw = WeaponSwings::default();
        for s in [&sw.spear, &sw.axe, &sw.sword] {
            let mut last = pose_at(s, 1.0, melee(0.0)).0;
            let mut t = 0.0;
            while t < 0.75 {
                t += 0.002;
                let now = pose_at(s, 1.0, melee(t)).0;
                // The hit's t cubed is fast but continuous: no jump of a
                // quarter radian in 2 ms anywhere.
                assert!((now.pitch - last.pitch).abs() < 0.25, "pitch jumps at {t}");
                assert!((now.pull_m - last.pull_m).abs() < 0.08, "pull jumps at {t}");
                last = now;
            }
        }
    }

    #[test]
    fn a_held_swing_waits_for_the_release() {
        let s = WeaponSwings::default();
        let held = |t, release_s| Timing::Held { t, release_s };
        let (k, stage, drawn) = pose_at(&s.bow, 1.0, held(5.0, None));
        assert_eq!(stage, Stage::Anticipation);
        assert_eq!(drawn, 1.0);
        assert_eq!(k, s.bow.coil);
        assert_eq!(pose_at(&s.throw, 1.0, held(0.9, Some(0.9))).1, Stage::Hit);
        assert_eq!(pose_at(&s.throw, 1.0, held(2.0, Some(0.9))).1, Stage::Rest);
    }

    #[test]
    fn the_shipped_swings_are_the_drafted_mockups() {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/config/held.ron"
        ))
        .unwrap();
        let shipped: crate::held::HeldSettings = ron::from_str(&text).unwrap();
        // One source of defaults: the file's block is the code's, which is
        // the mockup's printout.
        assert_eq!(shipped.weapon_swing, WeaponSwings::default());
        shipped.weapon_swing.validate().unwrap();
    }

    /// The drawn swing's system: its holders and arms are disjoint.
    #[test]
    fn holding_a_weapon_has_no_conflicting_queries() {
        use bevy::ecs::system::IntoSystem;
        let mut world = World::new();
        let mut sys = IntoSystem::into_system(super::super::weapon::hold_weapons);
        sys.initialize(&mut world);
    }
}
