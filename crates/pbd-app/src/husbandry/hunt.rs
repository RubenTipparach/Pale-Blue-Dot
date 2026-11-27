//! The hunt in the yard (`survival-mode` design 6 and 17, combat C2, tasks
//! 4.3 and 4.4): wild game takes the player's strikes and dies, a carcass
//! lies to be dressed, and the predators (`survival.ron`'s `predators`) hunt
//! the player, in a Survival world only.
//!
//! Every rule is the core's: `survival::predator::Hunt` decides a predator's
//! state, `survival::hunting::{strike, dress}` what a strike and a dressing
//! do. This moves the animals by those answers and hands the strikes and
//! bites to the server's path (`crate::combat`, `crate::survival::hit`).
//! Positions are local, relative to the one `SimOrigin` (the yard's rebase
//! moves them).

use super::{Beast, Farmyard, Mode, Who};
use bevy::prelude::*;
use pbd_core::husbandry::LandSpecies;
use pbd_core::survival::hunting::{self, Blade, Hunted, Quarry, Strike};
use pbd_core::survival::predator::{Motion, Phase, Senses};
use pbd_core::survival::{SurvivalConfig, WorldMode};

/// The record kind and schema of a killed herd member (design 17), keyed by
/// the member's seed.
pub const HUNTED_KIND: &str = "hunted";
pub const HUNTED_SCHEMA: u32 = 1;

/// A beast's hunt state, beside the yard's own.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Game {
    /// Health left; `None` is full (it has not been struck).
    pub health: Option<f32>,
    pub hunt: pbd_core::survival::predator::Hunt,
    /// Struck this frame: the hunt reads it once.
    pub struck: bool,
    /// Killed: seconds it has lain.
    pub dead_s: Option<f32>,
    pub dressed: bool,
    /// How low it crouches, 0..1 (drawn).
    pub crouch: f32,
    /// A flinch from a blow, 1 at the blow and down to 0 (drawn).
    pub flinch: f32,
    /// Its herd square and member, for the `hunted` record.
    pub herd: Option<(u64, u32)>,
}

/// What a strike did, for the combat path to act on.
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    /// Nothing at all: Explorer, a kept animal, or a carcass with no blade.
    Nothing,
    /// Refused with a note ("Protected").
    Refused(pbd_core::survival::Note),
    Hurt,
    /// Killed: its drops at once, where it lies (local), and the record of
    /// a herd member to write.
    Killed {
        drops: hunting::Drops,
        at: Vec3,
        hunted: Option<(u64, Hunted)>,
        species: String,
    },
    /// A carcass dressed: its hide (and a hunting axe's meat) where it lies.
    Dressed {
        drops: hunting::Drops,
        at: Vec3,
    },
}

/// A beast found along the aim.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aimed {
    pub index: usize,
    /// Along the aim, m.
    pub distance: f32,
    pub dead: bool,
}

/// A beast's body for a hit test: a sphere over its feet, local.
fn body(b: &Beast, sp: &LandSpecies, up: Vec3) -> (Vec3, f32) {
    let r = (sp.length_m * 0.5).max(sp.height_m * 0.6) * b.genome.size_factor();
    let lie = if b.game.dead_s.is_some() { 0.3 } else { 0.6 };
    (b.pos + up * sp.height_m * lie, r)
}

/// A swing sweeps an arc, not a line: it meets a body the aim passes this
/// close to, m (combat C1). A projectile's flight has none.
pub const SWING_SLACK_M: f32 = 0.35;

/// How far along the ground from the eye a point `d` along the look is: a
/// swing's reach is measured level, so a low animal at the feet is in reach
/// of a blade that reaches the same distance (combat C1).
pub fn level_m(origin: &pbd_core::world_frame::SimOrigin, eye: Vec3, look: Vec3, d: f32) -> f32 {
    let up = origin.up_at(eye);
    let v = look * d;
    (v - up * v.dot(up)).length()
}

fn ray_sphere(from: Vec3, dir: Vec3, centre: Vec3, r: f32) -> Option<f32> {
    let m = from - centre;
    let b = m.dot(dir);
    let c = m.length_squared() - r * r;
    if c > 0.0 && b > 0.0 {
        return None;
    }
    let disc = b * b - c;
    (disc >= 0.0).then(|| (-b - disc.sqrt()).max(0.0))
}

impl Farmyard {
    /// The nearest beast the eye's ray meets within `reach_m`, alive or a
    /// carcass not yet dressed. `eye` local, `look` a unit vector.
    pub fn aim(
        &self,
        origin: &pbd_core::world_frame::SimOrigin,
        roster: &[LandSpecies],
        eye: Vec3,
        look: Vec3,
        reach_m: f32,
    ) -> Option<Aimed> {
        let mut best: Option<Aimed> = None;
        for (index, b) in self.beasts.iter().enumerate() {
            if b.leaving || b.game.dressed {
                continue;
            }
            let Some(sp) = roster.get(b.species as usize) else {
                continue;
            };
            let (c, r) = body(b, sp, origin.up_at(b.pos));
            if let Some(d) = ray_sphere(eye, look, c, r + SWING_SLACK_M)
                && level_m(origin, eye, look, d) <= reach_m
                && best.is_none_or(|a| d < a.distance)
            {
                best = Some(Aimed {
                    index,
                    distance: d,
                    dead: b.game.dead_s.is_some(),
                });
            }
        }
        best
    }

    /// The first living beast the segment `a`..`b` (local) passes through,
    /// and where: a projectile's step.
    pub fn crossed(
        &self,
        origin: &pbd_core::world_frame::SimOrigin,
        roster: &[LandSpecies],
        a: Vec3,
        b: Vec3,
    ) -> Option<(usize, Vec3)> {
        let along = b - a;
        let len = along.length();
        if len <= 1e-6 {
            return None;
        }
        let dir = along / len;
        let mut best: Option<(usize, f32)> = None;
        for (index, beast) in self.beasts.iter().enumerate() {
            if beast.leaving || beast.game.dead_s.is_some() {
                continue;
            }
            let Some(sp) = roster.get(beast.species as usize) else {
                continue;
            };
            let (c, r) = body(beast, sp, origin.up_at(beast.pos));
            if let Some(d) = ray_sphere(a, dir, c, r)
                && d <= len
                && best.is_none_or(|(_, e)| d < e)
            {
                best = Some((index, d));
            }
        }
        best.map(|(i, d)| (i, a + dir * d))
    }

    /// A strike of `damage` on beast `index` in a world of `mode`, through
    /// the core's rule. A carcass is dressed by a blade instead; `day` is the
    /// world day for the `hunted` record.
    #[allow(clippy::too_many_arguments)]
    pub fn strike(
        &mut self,
        cfg: &SurvivalConfig,
        roster: &[LandSpecies],
        mode: WorldMode,
        index: usize,
        damage: f32,
        blade: Option<Blade>,
        day: f64,
    ) -> Outcome {
        let Some(b) = self.beasts.get_mut(index) else {
            return Outcome::Nothing;
        };
        let Some(sp) = roster.get(b.species as usize) else {
            return Outcome::Nothing;
        };
        if !mode.is_survival() {
            return Outcome::Nothing;
        }
        if b.game.dead_s.is_some() {
            if b.game.dressed {
                return Outcome::Nothing;
            }
            let Some(blade) = blade else {
                return Outcome::Nothing;
            };
            b.game.dressed = true;
            b.leaving = true;
            return Outcome::Dressed {
                drops: hunting::dress(cfg, &sp.id, blade, kill_hash(b, day)),
                at: b.pos,
            };
        }
        let quarry = match b.who {
            Who::Kept { .. } => Quarry::Kept,
            Who::Wild { .. } => Quarry::Wild(sp.id.clone()),
        };
        let full = hunting::full_health(cfg, &sp.id).unwrap_or(1.0);
        let health = b.game.health.unwrap_or(full);
        match hunting::strike(cfg, mode, &quarry, health, damage) {
            Strike::Refused(Some(note)) => Outcome::Refused(note),
            Strike::Refused(None) => Outcome::Nothing,
            Strike::Hurt { left } => {
                b.game.health = Some(left);
                b.game.struck = true;
                b.game.flinch = 1.0;
                b.trust = Default::default();
                // Game that is not a predator runs from the blow.
                if !cfg.predators.contains_key(&sp.id) {
                    b.mode = Mode::Flee;
                    b.mode_s = 4.0;
                    b.goal = Some(b.pos + b.facing * -8.0);
                }
                Outcome::Hurt
            }
            Strike::Killed { drops } => {
                b.game.health = Some(0.0);
                b.game.dead_s = Some(0.0);
                b.game.flinch = 1.0;
                b.mode = Mode::Idle;
                b.goal = None;
                b.speed = 0.0;
                let hunted = b.game.herd.and_then(|(square, member)| match b.who {
                    Who::Wild { seed } => Some((
                        seed,
                        Hunted {
                            square,
                            member,
                            day,
                        },
                    )),
                    Who::Kept { .. } => None,
                });
                Outcome::Killed {
                    drops,
                    at: b.pos,
                    hunted,
                    species: sp.id.clone(),
                }
            }
        }
    }

    /// A beast's species id and its health share, for the HUD and tests.
    pub fn health_of(
        &self,
        cfg: &SurvivalConfig,
        roster: &[LandSpecies],
        index: usize,
    ) -> Option<(String, f32)> {
        let b = self.beasts.get(index)?;
        let sp = roster.get(b.species as usize)?;
        let full = hunting::full_health(cfg, &sp.id)?;
        Some((sp.id.clone(), b.game.health.unwrap_or(full) / full))
    }

    /// Stand a wild animal of roster index `species` at `pos` (local),
    /// facing `facing`, held by the yard as a scripted one (its seed's top
    /// byte 0x5c, which the sync never fades): a capture instrument's and a
    /// test's. Returns its index.
    pub fn place_wild(&mut self, species: u16, pos: Vec3, up: Vec3, facing: Vec3, n: u64) -> usize {
        let seed = (0x5c << 56) | (n & 0x00ff_ffff_ffff_ffff);
        let mut b = Beast::new(
            Who::Wild { seed },
            species,
            pbd_core::husbandry::Sex::F,
            pbd_core::husbandry::Genome::default(),
            pos,
            up,
            super::Range::Around(up, 6.0),
            seed,
        );
        b.facing = facing;
        b.shown = 1.0;
        self.beasts.push(b);
        self.beasts.len() - 1
    }

    /// Hold a placed animal still or let it go (a capture's pose).
    pub fn hold_still(&mut self, index: usize, still: bool) {
        if let Some(b) = self.beasts.get_mut(index) {
            b.posed = still;
            if still {
                // Not even pottering about (the yard's own capture does the
                // same).
                b.idle_s = 1.0e6;
                b.goal = None;
            }
        }
    }

    /// Set a predator's hunt phase and crouch at once (a capture's pose).
    pub fn pose_hunt(&mut self, index: usize, phase: Phase, crouch: f32) {
        if let Some(b) = self.beasts.get_mut(index) {
            b.game.hunt.phase = phase;
            b.game.crouch = crouch;
        }
    }

    /// Every beast within 4 m of the eye, for a swing that met the air: its
    /// species, where its body is from the eye and how far the aim passes
    /// it, m (the capture's log).
    pub fn near_for_log(
        &self,
        origin: &pbd_core::world_frame::SimOrigin,
        roster: &[LandSpecies],
        eye: Vec3,
        look: Vec3,
    ) -> Vec<(String, Vec3, f32)> {
        self.beasts
            .iter()
            .filter_map(|b| {
                let sp = roster.get(b.species as usize)?;
                let (c, r) = body(b, sp, origin.up_at(b.pos));
                let to = c - eye;
                (to.length() < 4.0).then(|| {
                    let miss = (to - look * to.dot(look)).length() - r;
                    (sp.id.clone(), to, miss)
                })
            })
            .collect()
    }

    /// A beast's roster index.
    pub fn species_of(&self, index: usize) -> Option<u16> {
        self.beasts.get(index).map(|b| b.species)
    }

    /// The bites landed since the last call, pts each.
    pub fn take_bites(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.bites)
    }

    /// Each predator near and its phase, for the capture's log and tests.
    pub fn hunts(&self, roster: &[LandSpecies]) -> Vec<(String, Phase)> {
        self.beasts
            .iter()
            .filter(|b| !b.leaving && b.game.dead_s.is_none())
            .filter_map(|b| {
                let sp = roster.get(b.species as usize)?;
                Some((sp.id.clone(), b.game.hunt.phase))
            })
            .collect()
    }
}

/// The kill's hash, for dressing's roll: its seed and the world day.
fn kill_hash(b: &Beast, day: f64) -> u64 {
    let seed = match b.who {
        Who::Wild { seed } => seed,
        Who::Kept { id } => id,
    };
    pbd_core::fauna::hash2(seed, day.to_bits())
}

/// What the hunt decides this frame for one beast, before the yard's step
/// moves it.
#[allow(clippy::too_many_arguments)]
pub(super) fn drive(
    b: &mut Beast,
    sp: &LandSpecies,
    cfg: &SurvivalConfig,
    mode: WorldMode,
    feet: Option<Vec3>,
    up: Vec3,
    night: bool,
    player_down: bool,
    dt: f32,
) -> Option<f32> {
    b.game.flinch = (b.game.flinch - dt / 0.3).max(0.0);
    if let Some(s) = b.game.dead_s.as_mut() {
        *s += dt;
        if !hunting::carcass_lies(cfg, *s) {
            b.leaving = true;
        }
        return None;
    }
    let predator = cfg.predators.get(&sp.id);
    let (Some(p), Some(feet), Who::Wild { .. }) = (predator, feet, b.who) else {
        b.game.struck = false;
        b.game.crouch = (b.game.crouch - dt * 3.0).max(0.0);
        return None;
    };
    let full = hunting::full_health(cfg, &sp.id).unwrap_or(1.0);
    let to = feet - b.pos;
    let flat = to - up * to.dot(up);
    let senses = Senses {
        distance_m: flat.length(),
        night,
        player_down,
        health_share: b.game.health.unwrap_or(full) / full,
        struck: std::mem::take(&mut b.game.struck),
    };
    let bite = b.game.hunt.step(cfg, p, mode, &senses, dt);
    let dir = flat.normalize_or(b.facing);
    let crouch_goal = match b.game.hunt.motion(p) {
        Motion::Free => {
            if b.mode == Mode::Hunt {
                b.mode = Mode::Idle;
                b.goal = None;
            }
            0.0
        }
        Motion::Toward { mps, crouch } => {
            b.mode = Mode::Hunt;
            b.hunt_mps = mps;
            // Up to the player, stopping a body short.
            b.goal = Some(feet - dir * (sp.length_m * 0.6).max(0.4));
            crouch
        }
        Motion::Hold { crouch } => {
            b.mode = Mode::Hunt;
            b.goal = None;
            b.speed = 0.0;
            // Face the player as it warns.
            let angle = b.facing.angle_between(dir);
            if angle > 1e-3 {
                let axis = b.facing.cross(dir).normalize_or(up);
                b.facing = Quat::from_axis_angle(axis, angle.min(3.0 * dt)) * b.facing;
            }
            crouch
        }
        Motion::Away => {
            b.mode = Mode::Flee;
            b.mode_s = 1.0;
            b.goal = Some(b.pos - dir * 12.0);
            0.0
        }
    };
    b.game.crouch += (crouch_goal - b.game.crouch).clamp(-dt * 3.0, dt * 3.0);
    bite
}

/// Whether a herd member is still away after its kill: its `hunted` record
/// and the world day now.
pub(super) fn away(
    save: &crate::saves::WorldSave,
    cfg: &SurvivalConfig,
    seed: u64,
    day: f64,
) -> bool {
    save.records
        .get(HUNTED_KIND, seed)
        .and_then(|r| r.read::<Hunted>())
        .is_some_and(|h| !h.back_at(cfg, day))
}

/// A square's id for the `hunted` record.
pub(super) fn square_id(square: pbd_core::husbandry::herds::Square) -> u64 {
    (u64::from(square.row as u32) << 32) | u64::from(square.col as u32)
}

/// The world's mode, for the yard's systems: Explorer when no world is open.
pub(super) fn mode_of(save: Option<&crate::saves::WorldSave>) -> WorldMode {
    save.map_or(WorldMode::Explorer, |s| s.mode())
}

/// Read by the yard's step: the hunt's pace for an animal in [`Mode::Hunt`].
pub(super) fn hunt_pace(b: &Beast) -> f32 {
    b.hunt_mps
}

/// Drawn: how low a crouching predator is, laid into the rig's `down`.
pub(super) fn drawn_down(b: &Beast) -> f32 {
    if b.game.dead_s.is_some() {
        1.0
    } else {
        b.down.max(b.game.crouch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbd_core::inventory::{Tier, Tool};
    use pbd_core::survival::weapons::{Held, melee_damage};

    fn roster() -> Vec<LandSpecies> {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/config/husbandry.ron"
        ))
        .unwrap();
        let settings: pbd_core::husbandry::HusbandrySettings = ron::from_str(&text).unwrap();
        settings.roster("home").to_vec()
    }

    fn index_of(roster: &[LandSpecies], id: &str) -> u16 {
        roster.iter().position(|s| s.id == id).unwrap() as u16
    }

    /// Task 4.4: an axe's hit lowers a predator's health by its table value.
    #[test]
    fn an_axe_hit_lowers_a_predators_health_by_its_table_value() {
        let cfg = SurvivalConfig::shipped();
        let roster = roster();
        let mut yard = Farmyard::default();
        let fox = yard.place_wild(
            index_of(&roster, "red-fox"),
            Vec3::Z * 2.0,
            Vec3::Y,
            Vec3::X,
            1,
        );
        let axe = melee_damage(&cfg, &Held::Tool(Tool::Axe, Tier::Stone));
        assert_eq!(axe, 10.0);
        let out = yard.strike(
            &cfg,
            &roster,
            WorldMode::Survival,
            fox,
            axe,
            Some(Blade::Other),
            1.0,
        );
        assert_eq!(out, Outcome::Hurt);
        let (species, share) = yard.health_of(&cfg, &roster, fox).unwrap();
        assert_eq!(species, "red-fox");
        assert!((share - 20.0 / 30.0).abs() < 1e-6, "{share}");
        // The hunting axe's table value kills what is left, and the carcass
        // dresses for its hide.
        let hunting_axe = melee_damage(&cfg, &Held::Weapon("bronze-hunting-axe".into()));
        let Outcome::Killed { drops, .. } = yard.strike(
            &cfg,
            &roster,
            WorldMode::Survival,
            fox,
            hunting_axe,
            None,
            1.0,
        ) else {
            panic!("not killed");
        };
        assert_eq!(drops, vec![("bone".to_string(), 1)]);
        let Outcome::Dressed { drops, .. } = yard.strike(
            &cfg,
            &roster,
            WorldMode::Survival,
            fox,
            hunting_axe,
            Some(Blade::HuntingAxe),
            1.0,
        ) else {
            panic!("not dressed");
        };
        assert_eq!(drops, vec![("hide".to_string(), 1)]);
    }

    /// A swing at an animal in Explorer does nothing, and a kept one is
    /// never struck.
    #[test]
    fn explorer_and_kept_animals_are_never_struck() {
        let cfg = SurvivalConfig::shipped();
        let roster = roster();
        let mut yard = Farmyard::default();
        let fox = yard.place_wild(
            index_of(&roster, "red-fox"),
            Vec3::Z * 2.0,
            Vec3::Y,
            Vec3::X,
            1,
        );
        assert_eq!(
            yard.strike(&cfg, &roster, WorldMode::Explorer, fox, 30.0, None, 1.0),
            Outcome::Nothing
        );
        assert_eq!(yard.health_of(&cfg, &roster, fox).unwrap().1, 1.0);
        yard.beasts[fox].who = Who::Kept { id: 7 };
        assert_eq!(
            yard.strike(&cfg, &roster, WorldMode::Survival, fox, 30.0, None, 1.0),
            Outcome::Nothing
        );
    }

    /// Task 4.3: the yard's predator moves by the hunt's states, each
    /// change as the core decides it; an Explorer fox never attacks.
    #[test]
    fn a_survival_fox_stalks_warns_and_bites_and_an_explorer_fox_never_does() {
        let cfg = SurvivalConfig::shipped();
        let roster = roster();
        let sp = roster[index_of(&roster, "red-fox") as usize].clone();
        let run = |mode: WorldMode| {
            let mut yard = Farmyard::default();
            let i = yard.place_wild(
                index_of(&roster, "red-fox"),
                Vec3::Z * 12.0,
                Vec3::Y,
                Vec3::X,
                1,
            );
            let b = &mut yard.beasts[i];
            let mut phases = vec![b.game.hunt.phase];
            let mut bites = 0.0;
            // The player stands at the origin; the fox comes in a metre a
            // step of the walk the yard would give it.
            for _ in 0..600 {
                // Once warned the player steps in, a metre from it.
                let feet = Some(match b.game.hunt.phase {
                    Phase::Warn | Phase::Attack => b.pos - b.pos.normalize_or(Vec3::Z),
                    _ => Vec3::ZERO,
                });
                if let Some(d) = drive(b, &sp, &cfg, mode, feet, Vec3::Y, false, false, 1.0 / 30.0)
                {
                    bites += d;
                }
                if let (Mode::Hunt, Some(goal)) = (b.mode, b.goal) {
                    let to = goal - b.pos;
                    b.pos += to.normalize_or_zero() * (b.hunt_mps / 30.0).min(to.length());
                }
                if phases.last() != Some(&b.game.hunt.phase) {
                    phases.push(b.game.hunt.phase);
                }
            }
            (phases, bites, b.mode)
        };
        let (phases, bites, _) = run(WorldMode::Survival);
        assert_eq!(
            &phases[..4],
            &[Phase::Wander, Phase::Stalk, Phase::Warn, Phase::Attack]
        );
        assert!(bites >= 6.0, "{bites}");
        let (phases, bites, mode) = run(WorldMode::Explorer);
        assert_eq!(phases, vec![Phase::Wander]);
        assert_eq!(bites, 0.0);
        assert_ne!(mode, Mode::Hunt);
    }

    /// A kill writes its `hunted` record's key and the herd member is left
    /// out until it is back.
    #[test]
    fn a_kill_names_its_herd_member() {
        let cfg = SurvivalConfig::shipped();
        let roster = roster();
        let mut yard = Farmyard::default();
        let hare = yard.place_wild(
            index_of(&roster, "hare"),
            Vec3::Z * 2.0,
            Vec3::Y,
            Vec3::X,
            3,
        );
        yard.beasts[hare].game.herd = Some((0x51, 2));
        let Outcome::Killed { hunted, drops, .. } =
            yard.strike(&cfg, &roster, WorldMode::Survival, hare, 40.0, None, 4.5)
        else {
            panic!("not killed");
        };
        let (_, h) = hunted.unwrap();
        assert_eq!((h.square, h.member, h.day), (0x51, 2, 4.5));
        assert!(drops.iter().any(|(g, _)| g == "raw-meat"));
    }

    /// The yard's systems with the hunt's parameters have no conflicting
    /// queries (Bevy's B0001).
    #[test]
    fn the_yards_systems_have_no_conflicting_queries() {
        use bevy::ecs::system::IntoSystem;
        let mut world = bevy::ecs::world::World::new();
        let mut step = IntoSystem::into_system(super::super::step);
        step.initialize(&mut world);
        let mut sync = IntoSystem::into_system(super::super::sync);
        sync.initialize(&mut world);
        let mut draw = IntoSystem::into_system(super::super::draw);
        draw.initialize(&mut world);
    }
}
