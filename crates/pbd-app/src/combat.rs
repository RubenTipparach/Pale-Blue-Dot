//! Combat in the game (`survival-mode` combat C1 to C4, tasks 4.3 to 4.5):
//! the swing's rule run each frame, its hit landed at the rule's moment on
//! what the aim meets then, a thrown spear's and an arrow's flight, the
//! predators' bites, and townsfolk who fight back.
//!
//! - **The rule owns the clock** (`pbd_core::survival::melee`): a press
//!   starts a swing, its hit lands `hit_share` of the weapon's time later,
//!   and holding swings on one beat. The drawing reads [`SwingNow`]
//!   (`held::weapon_swing`) and never feeds it back.
//! - **What the click is.** With a weapon in hand the click is always a
//!   strike's ([`CombatClaim`]); with a tool or bare hands only in a Survival
//!   world and while a creature or a townsperson is within the swing's reach
//!   on the aim. A claimed click digs nothing (the digging reads the claim)
//!   and a tool draws today's stroke at the air.
//! - **Strikes and kills** go through the core (`hunting::strike`, `dress`);
//!   drops and records take the save's durable path on the frame they are
//!   made. Bites and guards' blows are hits through the server
//!   (`crate::survival::hit`).
//! - **Projectiles** fly in the body's frame in f64 (`weapons::Projectile`);
//!   the game draws them at the local point the origin gives.
//! - **No player is ever a target** (C4): the aim looks at the yard's
//!   animals and the townsfolk only.
//!
//! None of it does anything in an Explorer world.

use crate::drops::Drops;
use crate::held::stroke::{DigBeat, Meets};
use crate::held::weapon_swing::{SwingKind, SwingNow, Timing};
use crate::hotbar::Hotbar;
use crate::husbandry::Farmyard;
use crate::husbandry::hunt::{HUNTED_KIND, HUNTED_SCHEMA, Outcome};
use crate::saves::WorldSave;
use crate::townsfolk::Townsfolk;
use crate::walking::{Walker, WalkingCamera, WalkingReadout};
use avian3d::prelude::Position;
use bevy::math::DVec3;
use bevy::prelude::*;
use pbd_core::goods::Good;
use pbd_core::husbandry::LandSpecies;
use pbd_core::inventory::{Item, Tool};
use pbd_core::missions::Kind;
use pbd_core::records::{Author, Record};
use pbd_core::survival::health::Cause;
use pbd_core::survival::hunting::Blade;
use pbd_core::survival::melee::{self, Draw, Release, SwingClock};
use pbd_core::survival::standing::{self, Guard, GuardAct, Standing};
use pbd_core::survival::weapons::{self, Flight, Held, Missile, Projectile, Struck};
use pbd_core::survival::wear::{Action, Spend, Wear, Wearer};
use pbd_core::survival::{SurvivalConfig, WorldMode};
use pbd_core::townsfolk::PersonId;
use pbd_core::world_frame::{SimOrigin, WorldPos};

/// The click this frame is a strike's, not a dig's: the digging reads it
/// and leaves the block alone.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CombatClaim(pub bool);

/// A strike that landed, for the log, the missions and the tests.
#[derive(Message, Clone, Debug, PartialEq)]
pub enum Landed {
    /// On a creature: its species, the damage and whether it died.
    Creature {
        species: String,
        damage: f32,
        killed: bool,
    },
    /// A carcass dressed.
    Dressed { species: String },
    /// On a townsperson: their town and its standing after.
    Townsperson { site: u32, standing: i32 },
    /// The swing met nothing.
    Air,
}

/// The aim and the swing run before the digging reads the claim.
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CombatSet;

/// A projectile's drawn model.
#[derive(Component)]
pub struct InFlight;

/// A projectile in flight and its drawn model.
struct Flying {
    projectile: Projectile,
    good: Good,
    /// Where it left the hand (local), for a long shot.
    from: Vec3,
    entity: Option<Entity>,
}

/// A fight with a town's guards (C3).
#[derive(Clone, Debug)]
struct TownFight {
    site: u32,
    centre: DVec3,
    walls_m: f32,
    guards: Vec<(PersonId, Guard, DVec3)>,
    fleeing: Vec<(PersonId, f32, DVec3)>,
}

/// The rule's state between frames.
#[derive(Resource, Default)]
pub struct Combat {
    clock: SwingClock,
    draw: Draw,
    /// A spear raised or the bow drawn: which, seconds since the press, and
    /// when it was let go (drawn on through its follow-through).
    held: Option<(SwingKind, f32, Option<f32>)>,
    flights: Vec<Flying>,
    fight: Option<TownFight>,
    /// A capture's presses (`--combat`): the left button held this frame,
    /// the right held this frame. A capture instrument.
    pub scripted: Option<(bool, bool)>,
}

impl Combat {
    /// Projectiles in flight now.
    pub fn flying(&self) -> usize {
        self.flights.len()
    }

    /// The guards in a fight now.
    pub fn guards(&self) -> usize {
        self.fight.as_ref().map_or(0, |f| f.guards.len())
    }
}

/// What is in hand, as the rule and the drawing name it.
#[derive(Clone, Debug, PartialEq)]
pub struct InHand {
    pub held: Held,
    /// The weapon's good, if a weapon.
    pub weapon: Option<Good>,
    /// The swing the left button makes, and the right's.
    pub left: Option<SwingKind>,
    pub right: Option<SwingKind>,
}

/// The goods a weapon is, by its swing.
pub fn in_hand(
    item: Option<Item>,
    tool: Option<(Tool, pbd_core::inventory::Tier)>,
    broken: bool,
    weapon_broken: bool,
) -> InHand {
    let weapon = item.and_then(crate::held::weapon::weapon_of);
    if let Some(good) = weapon {
        let code = good.code();
        let (left, right) = if code.ends_with("-spear") {
            (Some(SwingKind::Thrust), Some(SwingKind::Throw))
        } else if code.ends_with("-hunting-axe") {
            (Some(SwingKind::Chop), None)
        } else if code.ends_with("-sword") {
            (Some(SwingKind::Slash), None)
        } else if good == Good::BOW {
            (Some(SwingKind::Bow), None)
        } else {
            (None, None)
        };
        let held = if left == Some(SwingKind::Bow) || left.is_none() {
            Held::Hand
        } else if weapon_broken {
            // A worn-out weapon hits as bare hands (design 11).
            Held::Broken
        } else {
            Held::Weapon(code.to_string())
        };
        return InHand {
            held,
            weapon,
            left,
            right,
        };
    }
    let held = match (tool, broken) {
        (_, true) => Held::Broken,
        (Some((t, tier)), false) => Held::Tool(t, tier),
        (None, false) => Held::Hand,
    };
    InHand {
        held,
        weapon: None,
        left: None,
        right: None,
    }
}

/// The weapons' wear (design 11, the same model as the tools'): one record
/// of the player's, the uses each weapon kind has left by its good's code; a
/// weapon with no entry is full, so an old save reads full.
pub const WEAPON_WEAR_KIND: &str = "weapon_wear";
pub const WEAPON_WEAR_SCHEMA: u32 = 1;

#[derive(Clone, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WeaponWearBook {
    pub weapons: std::collections::BTreeMap<String, Wear>,
}

/// What wears for a weapon's good code.
pub fn wearer_of(code: &str) -> Wearer {
    if code == Good::BOW.code() {
        Wearer::Bow
    } else {
        Wearer::Weapon(code.to_string())
    }
}

impl WeaponWearBook {
    pub fn of(save: &WorldSave) -> Self {
        save.records
            .get(WEAPON_WEAR_KIND, 0)
            .and_then(|r| r.read())
            .unwrap_or_default()
    }

    pub fn wear(&self, code: &str) -> Wear {
        Wear::of_saved(
            crate::survival::rules(),
            &wearer_of(code),
            self.weapons.get(code).copied(),
        )
    }

    pub fn broken(&self, code: &str) -> bool {
        self.wear(code).is_broken()
    }
}

/// Spend a weapon's wear for `action`, through the durable path. Nothing in
/// Explorer.
pub fn spend_weapon_wear(save: &mut WorldSave, code: &str, action: Action) -> Spend {
    if !save.mode().is_survival() {
        return Spend::None;
    }
    let mut book = WeaponWearBook::of(save);
    let mut wear = book.wear(code);
    let spent = wear.spend(
        crate::survival::rules(),
        save.mode(),
        &wearer_of(code),
        action,
    );
    if spent == Spend::None {
        return spent;
    }
    book.weapons.insert(code.to_string(), wear);
    let rec = Record::of(WEAPON_WEAR_KIND, 0, WEAPON_WEAR_SCHEMA, &book);
    if save.store(&Author::Player(0), vec![rec]).is_none() {
        return Spend::None;
    }
    save.note_record_kinds(&[(WEAPON_WEAR_KIND, WEAPON_WEAR_SCHEMA)]);
    spent
}

/// A landed hit's wear on what was in hand: a weapon's or a tool's.
fn wear_hit(save: &mut WorldSave, held: &Held) -> Option<pbd_core::locale::Localized> {
    let (spent, name) = match held {
        Held::Weapon(code) => (
            spend_weapon_wear(save, code, Action::HitLanded),
            Good::by_code(code).map(|g| g.name()),
        ),
        Held::Tool(tool, tier) => (
            crate::survival::spend_wear(save, *tool, *tier, Action::HitLanded),
            Some(pbd_core::inventory::tool_title(*tool, *tier)),
        ),
        _ => return None,
    };
    (spent == Spend::Broke).then(|| broke_line(name))
}

fn broke_line(name: Option<pbd_core::locale::Localized>) -> pbd_core::locale::Localized {
    let name = name.map(|n| n.to_string()).unwrap_or_default();
    pbd_core::locale::tr_with("survival.broke", &[("tool", &name)])
}

/// The blade a held thing dresses a carcass with, if it can.
pub fn blade_of(held: &Held) -> Option<Blade> {
    match held {
        Held::Weapon(code) if code.ends_with("-hunting-axe") => Some(Blade::HuntingAxe),
        Held::Weapon(_) | Held::Tool(Tool::Axe, _) => Some(Blade::Other),
        _ => None,
    }
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

/// The townsperson on the aim within `reach_m`, nearest first: their id and
/// distance. `eye` local.
fn aimed_person(
    folk: &Townsfolk,
    origin: &SimOrigin,
    eye: Vec3,
    look: Vec3,
    reach_m: f32,
) -> Option<(PersonId, f32)> {
    let mut best: Option<(PersonId, f32)> = None;
    for person in folk.everyone() {
        let Some(placed) = folk.placed(person.id) else {
            continue;
        };
        let feet = origin.to_local(WorldPos::new(origin.frame, placed.feet));
        let centre = feet + origin.up_at(feet) * 0.95;
        if let Some(d) = ray_sphere(
            eye,
            look,
            centre,
            0.45 + crate::husbandry::hunt::SWING_SLACK_M,
        ) && crate::husbandry::hunt::level_m(origin, eye, look, d) <= reach_m
            && best.is_none_or(|(_, e)| d < e)
        {
            best = Some((person.id, d));
        }
    }
    best
}

/// The town's centre (a unit direction scaled to the radius) and its walls'
/// reach, m, from its people's posts.
fn town_bounds(folk: &Townsfolk, site: u32) -> Option<(DVec3, f32)> {
    let posts: Vec<Vec3> = folk
        .people(site)
        .map(|p| p.at.normalize_or(Vec3::Y))
        .collect();
    if posts.is_empty() {
        return None;
    }
    let centre = posts.iter().copied().sum::<Vec3>().normalize_or(Vec3::Y);
    let r = crate::planet::PLANET_RADIUS;
    let walls = posts
        .iter()
        .map(|p| p.angle_between(centre) * r)
        .fold(0.0f32, f32::max);
    Some((centre.as_dvec3() * f64::from(r), walls + 6.0))
}

/// A planet point moved `step_m` toward `to` along the ground, at its own
/// height: a guard's or a fleeing person's stride.
fn stride_toward(from: DVec3, to: DVec3, step_m: f64) -> (DVec3, DVec3) {
    let r = from.length();
    let up = from / r.max(1e-9);
    let flat = (to - from) - up * (to - from).dot(up);
    let d = flat.length();
    if d < 1e-6 {
        return (from, up.any_orthonormal_vector());
    }
    let dir = flat / d;
    let next = (from + dir * step_m.min(d)).normalize() * r;
    (next, dir)
}

/// The standing a town has in the save, 0 for none.
pub fn standing_of(save: &WorldSave, site: u32) -> i32 {
    standing::of_saved(
        save.records
            .get(standing::KIND, u64::from(site))
            .and_then(|r| r.read::<Standing>()),
    )
}

/// Write a town's standing through the durable path.
fn store_standing(save: &mut WorldSave, site: u32, value: i32) -> bool {
    let rec = Record::of(
        standing::KIND,
        u64::from(site),
        standing::SCHEMA,
        &Standing { site, value },
    );
    let kept = save.store(&Author::Player(0), vec![rec]).is_some();
    if kept {
        save.note_record_kinds(&[(standing::KIND, standing::SCHEMA)]);
    }
    kept
}

/// A strike on a townsperson in a world of `mode` (C3): in Survival the
/// town's standing drops and a fight starts; in Explorer nothing happens.
/// The standing after, or `None`.
pub fn strike_person(
    cfg: &SurvivalConfig,
    save: &mut WorldSave,
    mode: WorldMode,
    site: u32,
) -> Option<i32> {
    let now = standing_of(save, site);
    let after = standing::struck(cfg, mode, now)?;
    store_standing(save, site, after).then_some(after)
}

/// Drops of `(good, count)` at a local point, through the durable path.
#[allow(clippy::too_many_arguments)]
fn drop_goods(
    save: &mut WorldSave,
    drops: &mut Drops,
    carried: &pbd_core::inventory::Slots,
    origin: &SimOrigin,
    at: Vec3,
    up: Vec3,
    goods: &[(String, u16)],
    now_s: f64,
) {
    let centre = origin.to_world(at + up * 0.3).pos;
    for (code, count) in goods {
        let Some(good) = Good::by_code(code) else {
            continue;
        };
        let mut drop = Drops::make(save, Item::Good(good), centre, now_s);
        drop.count = *count;
        if save.record_toss(&drop, carried) {
            drops.live.push(drop);
        } else {
            error!("combat: the save did not accept a drop of {count} {code}");
        }
    }
}

/// The yard's roster on this body.
fn roster(config: &crate::config::HusbandryConfig, body: &crate::fish::Body) -> Vec<LandSpecies> {
    config.0.roster(&body.0).to_vec()
}

/// What a landed strike on the yard did, acted on: drops, records, missions.
#[allow(clippy::too_many_arguments)]
fn act_on(
    outcome: Outcome,
    damage: f32,
    species: Option<String>,
    long_shot: bool,
    save: &mut WorldSave,
    drops: &mut Drops,
    carried: &pbd_core::inventory::Slots,
    origin: &SimOrigin,
    now_s: f64,
    data_name: Option<String>,
    landed: &mut MessageWriter<Landed>,
    did: &mut MessageWriter<crate::missions::Did>,
    say: &mut Option<pbd_core::locale::Localized>,
) {
    let cfg = crate::survival::rules();
    match outcome {
        Outcome::Nothing => {}
        Outcome::Refused(note) => *say = Some(note.text()),
        Outcome::Hurt => {
            if let Some(species) = species {
                landed.write(Landed::Creature {
                    species,
                    damage,
                    killed: false,
                });
            }
        }
        Outcome::Killed {
            drops: goods,
            at,
            hunted,
            species,
        } => {
            info!("COMBAT_KILL {species} dropping {goods:?}");
            drop_goods(
                save,
                drops,
                carried,
                origin,
                at,
                origin.up_at(at),
                &goods,
                now_s,
            );
            if let Some((seed, h)) = hunted {
                let rec = Record::of(HUNTED_KIND, seed, HUNTED_SCHEMA, &h);
                if save.store(&Author::Player(0), vec![rec]).is_some() {
                    save.note_record_kinds(&[(HUNTED_KIND, HUNTED_SCHEMA)]);
                }
            }
            if let Some(name) = data_name {
                did.write(crate::missions::Did::new(Kind::Kill, name, 1));
            }
            if long_shot {
                did.write(crate::missions::Did::new(Kind::Kill, "long-shot", 1));
            }
            if cfg.predators.contains_key(&species) {
                did.write(crate::missions::Did::new(Kind::DriveOff, "predator", 1));
            }
            landed.write(Landed::Creature {
                species,
                damage,
                killed: true,
            });
        }
        Outcome::Dressed { drops: goods, at } => {
            drop_goods(
                save,
                drops,
                carried,
                origin,
                at,
                origin.up_at(at),
                &goods,
                now_s,
            );
            landed.write(Landed::Dressed {
                species: species.unwrap_or_default(),
            });
        }
    }
}

/// The world's mode now: Explorer with no world.
fn mode_now(save: Option<&WorldSave>) -> WorldMode {
    save.map_or(WorldMode::Explorer, |s| s.mode())
}

/// The eye (local) and the look.
fn eye_and_look(
    frame: &crate::planet::PlanetRenderFrame,
    cameras: &Query<&GlobalTransform, With<WalkingCamera>>,
) -> Option<(Vec3, Vec3)> {
    let t = cameras.iter().next()?;
    Some((
        crate::sim_origin::local_from_render(frame, t.translation()),
        t.forward().as_vec3(),
    ))
}

/// The aim, the claim and the swing's rule: one frame.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn swing(
    time: Res<Time>,
    buttons: Res<ButtonInput<MouseButton>>,
    walking: Option<Res<WalkingReadout>>,
    item: Option<Res<crate::carried_light::HeldItem>>,
    tools: Option<Res<crate::fish::ToolSlot>>,
    broken: Option<Res<crate::survival::HeldBroken>>,
    cameras: Query<&GlobalTransform, With<WalkingCamera>>,
    frame: Option<Res<crate::planet::PlanetRenderFrame>>,
    space: crate::SimSpace,
    (mut combat, mut claim, mut now, mut swinging): (
        ResMut<Combat>,
        ResMut<CombatClaim>,
        ResMut<SwingNow>,
        Option<ResMut<crate::held::Swinging>>,
    ),
    (mut yard, folk, config, body): (
        Option<ResMut<Farmyard>>,
        Option<ResMut<Townsfolk>>,
        Option<Res<crate::config::HusbandryConfig>>,
        Option<Res<crate::fish::Body>>,
    ),
    (mut save, mut drops, mut hotbar, sun, mut survival): (
        Option<ResMut<WorldSave>>,
        Option<ResMut<Drops>>,
        Option<ResMut<Hotbar>>,
        Option<Res<crate::sky::Sun>>,
        Option<ResMut<crate::survival::Survival>>,
    ),
    (mut landed, mut did, mut sfx): (
        MessageWriter<Landed>,
        MessageWriter<crate::missions::Did>,
        Option<ResMut<crate::audio::Sfx>>,
    ),
) {
    let cfg = crate::survival::rules();
    let dt = time.delta_secs().clamp(0.0, 0.1);
    let mode = mode_now(save.as_deref());
    let origin = space.origin();
    let captured = walking.as_deref().is_some_and(|w| w.active && w.captured);
    let (left_down, right_down) = match combat.scripted {
        Some(pressed) => pressed,
        None => (
            captured && buttons.pressed(MouseButton::Left),
            captured && buttons.pressed(MouseButton::Right),
        ),
    };
    let tool = tools.as_deref().and_then(|t| {
        t.held()
            .and_then(|tool| t.tier(tool).map(|tier| (tool, tier)))
    });
    let item_now = item.as_deref().and_then(|i| i.0);
    let weapon_broken = match (
        item_now.and_then(crate::held::weapon::weapon_of),
        save.as_deref(),
    ) {
        (Some(good), Some(save)) => WeaponWearBook::of(save).broken(good.code()),
        _ => false,
    };
    let hand = in_hand(item_now, tool, broken.is_some_and(|b| b.0), weapon_broken);
    let roster = match (config.as_deref(), body.as_deref()) {
        (Some(c), Some(b)) => roster(c, b),
        _ => Vec::new(),
    };
    let eye = frame.as_deref().and_then(|f| eye_and_look(f, &cameras));
    let reach = weapons::melee_reach_m(cfg, &hand.held);
    // What the aim meets now.
    let aimed_beast = match (eye, yard.as_deref()) {
        (Some((e, l)), Some(y)) => y.aim(&origin, &roster, e, l, reach),
        _ => None,
    };
    let aimed_folk = match (eye, folk.as_deref()) {
        (Some((e, l)), Some(f)) => aimed_person(f, &origin, e, l, reach),
        _ => None,
    };
    let something = aimed_beast.is_some() || aimed_folk.is_some();
    // A weapon that swings (not an arrow, the straw or something worn).
    let weapon = hand.left.is_some();
    // The claim: a weapon's click is always a strike's; a tool's only on a
    // creature or a townsperson in Survival, or while its swing runs on.
    let running = combat.clock.under().is_some();
    claim.0 = (weapon && (captured || combat.scripted.is_some()))
        || (mode.is_survival() && (something || running) && (left_down || running));
    // The melee swing's rule.
    let melee_pressed =
        left_down && claim.0 && !matches!(hand.left, Some(SwingKind::Bow)) && combat.held.is_none();
    let hits = combat.clock.step(cfg, &hand.held, melee_pressed, dt);
    // A tool or bare hands draw today's stroke at the air while claimed.
    if let Some(sw) = swinging.as_deref_mut()
        && claim.0
        && !weapon
    {
        let beat = tool_beat(combat.clock.under());
        if sw.0 != beat {
            sw.0 = beat;
        }
    }
    // A spear raised to throw, or the bow drawn: the rule's draw.
    let draw_kind = if hand.right == Some(SwingKind::Throw) {
        Some((SwingKind::Throw, right_down, cfg.weapons.throw_min_s))
    } else if hand.left == Some(SwingKind::Bow) {
        Some((SwingKind::Bow, left_down, 0.05))
    } else {
        None
    };
    let mut release = None;
    if let Some((kind, pressed, min_s)) = draw_kind {
        let started = combat.draw.held_s.is_none() && pressed;
        if let Some(r) = combat.draw.step(pressed, dt, min_s) {
            release = Some((kind, r));
        }
        if started {
            combat.held = Some((kind, 0.0, None));
        }
    } else {
        combat.draw = Draw::default();
    }
    if let Some((kind, t, rel)) = combat.held.as_mut() {
        *t += dt;
        if let Some((k, r)) = release
            && k == *kind
        {
            *rel = Some(match r {
                Release::Go { .. } => *t,
                // Too brief: put down at once.
                Release::Nothing => -10.0,
            });
        }
    }
    // What the drawing reads.
    now.0 = if let Some((kind, t, rel)) = combat.held {
        Some((kind, Timing::Held { t, release_s: rel }))
    } else {
        combat.clock.under().and_then(|u| {
            let kind = match combat.clock.held() {
                Some(Held::Weapon(code)) if code.ends_with("-spear") => SwingKind::Thrust,
                Some(Held::Weapon(code)) if code.ends_with("-hunting-axe") => SwingKind::Chop,
                Some(Held::Weapon(code)) if code.ends_with("-sword") => SwingKind::Slash,
                _ => return None,
            };
            Some((
                kind,
                Timing::Melee {
                    t: u.t,
                    period_s: u.period_s,
                    hit_s: u.hit_s,
                },
            ))
        })
    };
    // A held swing's follow-through ends.
    if let Some((kind, t, Some(rel))) = combat.held {
        let sw = crate::held::weapon_swing::WeaponSwings::default();
        let s = sw.of(kind);
        if t > rel + s.strike_s + s.impact_hold_s + s.recoil_s + s.recover_s || rel < 0.0 {
            combat.held = None;
        }
    }
    let (Some(save), Some(drops), Some(hotbar), Some((eye, look))) = (
        save.as_deref_mut(),
        drops.as_deref_mut(),
        hotbar.as_deref_mut(),
        eye,
    ) else {
        return;
    };
    let now_s = sun.as_deref().map_or(0.0, |s| s.clock.seconds);
    let day = now_s / f64::from(pbd_core::daylight::DAY_S);
    let mut say = None;
    let mut broke = false;
    // Each hit lands now, on what the aim meets now (staging).
    for held in hits {
        let damage = weapons::melee_damage(cfg, &held);
        let reach = weapons::melee_reach_m(cfg, &held);
        let beast = yard
            .as_deref()
            .and_then(|y| y.aim(&origin, &roster, eye, look, reach));
        let person = folk
            .as_deref()
            .and_then(|f| aimed_person(f, &origin, eye, look, reach));
        match (beast, person) {
            (Some(b), p) if p.is_none_or(|(_, d)| b.distance <= d) => {
                let Some(yard) = yard.as_deref_mut() else {
                    continue;
                };
                let species = yard.health_of(cfg, &roster, b.index).map(|(s, _)| s);
                let data_name = yard
                    .species_of(b.index)
                    .and_then(|i| roster.get(i as usize))
                    .map(|s| s.data_name());
                let outcome =
                    yard.strike(cfg, &roster, mode, b.index, damage, blade_of(&held), day);
                info!("COMBAT_HIT {species:?} for {damage} -> {outcome:?}");
                if !matches!(outcome, Outcome::Nothing | Outcome::Refused(_))
                    && let Some(line) = wear_hit(save, &held)
                {
                    broke = true;
                    say = Some(line);
                }
                act_on(
                    outcome,
                    damage,
                    species,
                    false,
                    save,
                    drops,
                    &hotbar.0,
                    &origin,
                    now_s,
                    data_name,
                    &mut landed,
                    &mut did,
                    &mut say,
                );
            }
            (_, Some((id, _))) => {
                if let Some(after) = strike_person(cfg, save, mode, id.site) {
                    info!("COMBAT_TOWNSPERSON {id:?} standing {after}");
                    if let Some(line) = wear_hit(save, &held) {
                        broke = true;
                        say = Some(line);
                    }
                    landed.write(Landed::Townsperson {
                        site: id.site,
                        standing: after,
                    });
                    if let Some(folk) = folk.as_deref() {
                        start_fight(&mut combat, folk, cfg, id);
                    }
                }
            }
            _ => {
                let near = yard
                    .as_deref()
                    .map(|y| y.near_for_log(&origin, &roster, eye, look))
                    .unwrap_or_default();
                info!("COMBAT_HIT air; near {near:?}");
                landed.write(Landed::Air);
            }
        }
    }
    // A release throws or looses.
    if let Some((kind, Release::Go { drawn_s })) = release {
        let from = origin.to_world(eye + look * 0.3);
        let dir = look.as_dvec3();
        let launched = match kind {
            SwingKind::Throw => hand.weapon.and_then(|good| {
                let mut slots = hotbar.0.clone();
                slots.take_held()?;
                let p = Projectile::throw_spear(cfg, good.code(), from, dir)?;
                save.record_pack(&slots).then(|| {
                    hotbar.0 = slots;
                    // A throw spends stamina through the server (design 12).
                    let _ = save.survival_act(pbd_server::survival::Act::Burst(
                        pbd_core::survival::stamina::Burst::SpearThrow,
                    ));
                    (p, good)
                })
            }),
            SwingKind::Bow if weapon_broken => None,
            SwingKind::Bow => [Good::STONE_ARROW, Good::BONE_ARROW, Good::BRONZE_ARROW]
                .into_iter()
                .find(|a| hotbar.0.count(Item::Good(*a)) > 0)
                .and_then(|arrow| {
                    let mut slots = hotbar.0.clone();
                    slots.take_item(Item::Good(arrow), 1).then_some(())?;
                    let p = Projectile::loose_arrow(cfg, arrow.code(), from, dir, drawn_s)?;
                    save.record_pack(&slots).then(|| {
                        hotbar.0 = slots;
                        // Each arrow loosed is a use of the bow.
                        if spend_weapon_wear(save, Good::BOW.code(), Action::Used) == Spend::Broke {
                            broke = true;
                            say = Some(broke_line(Some(Good::BOW.name())));
                        }
                        (p, arrow)
                    })
                }),
            _ => None,
        };
        if let Some((projectile, good)) = launched {
            info!("COMBAT_LAUNCH {} drawn {drawn_s:.2}", good.code());
            combat.flights.push(Flying {
                projectile,
                good,
                from: eye,
                entity: None,
            });
        }
    }
    if broke {
        info!("COMBAT_WEAPON_BROKE");
        // The crack, in the hand (design 11).
        if let Some(sfx) = sfx.as_deref_mut() {
            sfx.play("tool_break", crate::audio::Where::At(eye + look * 0.6));
        }
    }
    if let (Some(text), Some(sv)) = (say, survival.as_deref_mut()) {
        sv.say(text);
    }
}

/// The tool stroke's beat for a strike (combat C1): the dig's own beat with
/// the rule's swing as its block, so the drawn impact lands on the rule's
/// hit frame. A block of the hit's time and a pause of the rest of the
/// period make one stroke of the whole period, its impact on the hit.
pub fn tool_beat(under: Option<melee::Under>) -> DigBeat {
    match under {
        Some(u) => DigBeat {
            working: true,
            meets: Meets::Breaks,
            break_s: u.hit_s,
            between_s: (u.period_s - u.hit_s).max(0.0),
            local_s: u.t,
        },
        None => DigBeat::default(),
    }
}

/// Start or join a fight with the town of `struck` (C3): its guards within
/// `call_m` come, everyone else near flees.
fn start_fight(combat: &mut Combat, folk: &Townsfolk, cfg: &SurvivalConfig, struck: PersonId) {
    let Some((centre, walls_m)) = town_bounds(folk, struck.site) else {
        return;
    };
    let Some(at) = folk.placed(struck).map(|p| p.feet) else {
        return;
    };
    let fight = combat.fight.get_or_insert_with(|| TownFight {
        site: struck.site,
        centre,
        walls_m,
        guards: Vec::new(),
        fleeing: Vec::new(),
    });
    for person in folk.people(struck.site) {
        let Some(placed) = folk.placed(person.id) else {
            continue;
        };
        let near = placed.feet.distance(at) <= f64::from(cfg.townsfolk.call_m);
        if person.spear {
            if near && !fight.guards.iter().any(|(id, _, _)| *id == person.id) {
                fight
                    .guards
                    .push((person.id, Guard::default(), placed.feet));
            }
        } else if (person.id == struck || placed.feet.distance(at) < 10.0)
            && !fight.fleeing.iter().any(|(id, _, _)| *id == person.id)
        {
            fight
                .fleeing
                .push((person.id, cfg.townsfolk.flee_s, placed.feet));
        }
    }
}

/// The town's guards in a fight, and those fleeing (C3).
#[allow(clippy::type_complexity)]
pub fn town_fight(
    time: Res<Time>,
    space: crate::SimSpace,
    walkers: Query<&Position, With<Walker>>,
    mut combat: ResMut<Combat>,
    folk: Option<ResMut<Townsfolk>>,
    mut save: Option<ResMut<WorldSave>>,
    mut survival: Option<ResMut<crate::survival::Survival>>,
) {
    let (Some(mut folk), Some(fight)) = (folk, combat.fight.as_mut()) else {
        return;
    };
    let cfg = crate::survival::rules();
    let dt = time.delta_secs().clamp(0.0, 0.1);
    let origin = space.origin();
    let Some(walker) = walkers.iter().next() else {
        return;
    };
    let feet_local = origin.with_radius(
        walker.0,
        origin.radius_at(walker.0) - f64::from(crate::walking::HALF_HEIGHT),
    );
    // The player's feet in the body's frame, f64.
    let player_at = origin.to_world(feet_local).pos;
    let down = survival
        .as_deref()
        .is_some_and(|sv| sv.dead || sv.vitals.as_ref().is_some_and(|v| v.consciousness.is_out()));
    let r = f64::from(crate::planet::PLANET_RADIUS);
    let from_centre = (player_at
        .normalize()
        .angle_between(fight.centre.normalize())
        * r) as f32;
    let on = standing::fight_on(cfg, from_centre, fight.walls_m, down)
        && save.as_deref().is_some_and(|s| s.mode().is_survival());
    if !on {
        for (id, _, _) in fight.guards.drain(..) {
            folk.release(id);
        }
        for (id, _, _) in fight.fleeing.drain(..) {
            folk.release(id);
        }
        info!("COMBAT_TOWN fight with town {} over", fight.site);
        combat.fight = None;
        return;
    }
    for (id, guard, at) in &mut fight.guards {
        // A guard in a fight looks it; drawn only.
        folk.set_expression(*id, pbd_core::townsfolk::face::Expression::Cross, 0.5);
        let d = at.distance(player_at) as f32;
        match guard.step(cfg, d, dt) {
            GuardAct::Close(mps) => {
                let (next, dir) = stride_toward(*at, player_at, f64::from(mps * dt));
                *at = next;
                folk.drive(*id, next, dir, true);
            }
            act => {
                let (_, dir) = stride_toward(*at, player_at, 0.0);
                let dir = if dir.length_squared() > 0.5 {
                    dir
                } else {
                    let up = at.normalize();
                    let flat = (player_at - *at) - up * (player_at - *at).dot(up);
                    flat.normalize_or(up.any_orthonormal_vector())
                };
                folk.drive(*id, *at, dir, false);
                if let GuardAct::Blow(pts) = act
                    && let (Some(save), Some(sv)) = (save.as_deref_mut(), survival.as_deref_mut())
                {
                    info!("COMBAT_GUARD_BLOW {pts}");
                    crate::survival::hit(save, sv, Cause::Townsfolk, pts);
                }
            }
        }
    }
    let mut done = Vec::new();
    for (id, left, at) in &mut fight.fleeing {
        folk.set_expression(*id, pbd_core::townsfolk::face::Expression::Worried, 0.5);
        *left -= dt;
        let away = *at + (*at - player_at).normalize_or(DVec3::X) * 10.0;
        let (next, dir) = stride_toward(*at, away, f64::from(3.0 * dt));
        *at = next;
        folk.drive(*id, next, dir, true);
        if *left <= 0.0 {
            done.push(*id);
        }
    }
    for id in done {
        folk.release(id);
        fight.fleeing.retain(|(i, _, _)| *i != id);
    }
}

/// Projectiles in flight: stepped in the body's frame, tested against the
/// yard and the ground, landed as drops (C1, design 16).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn fly(
    time: Res<Time>,
    space: crate::SimSpace,
    mut commands: Commands,
    mut combat: ResMut<Combat>,
    contact: Option<Res<crate::planet::PlanetContact>>,
    frame: Option<Res<crate::planet::PlanetRenderFrame>>,
    looks: Option<Res<crate::drops::DropLooks>>,
    mut views: Query<&mut Transform, With<InFlight>>,
    (mut yard, config, body): (
        Option<ResMut<Farmyard>>,
        Option<Res<crate::config::HusbandryConfig>>,
        Option<Res<crate::fish::Body>>,
    ),
    (mut save, mut drops, hotbar, sun): (
        Option<ResMut<WorldSave>>,
        Option<ResMut<Drops>>,
        Option<Res<Hotbar>>,
        Option<Res<crate::sky::Sun>>,
    ),
    (mut landed, mut did): (MessageWriter<Landed>, MessageWriter<crate::missions::Did>),
) {
    if combat.flights.is_empty() {
        return;
    }
    let cfg = crate::survival::rules();
    let dt = f64::from(time.delta_secs().clamp(0.0, 0.1));
    let origin = space.origin();
    let roster = match (config.as_deref(), body.as_deref()) {
        (Some(c), Some(b)) => roster(c, b),
        _ => Vec::new(),
    };
    let g = pbd_core::gravity::SURFACE_GRAVITY_MPS2_PER_G;
    let now_s = sun.as_deref().map_or(0.0, |s| s.clock.seconds);
    let day = now_s / f64::from(pbd_core::daylight::DAY_S);
    let mode = mode_now(save.as_deref());
    let mut flights = std::mem::take(&mut combat.flights);
    let mut keep = Vec::new();
    for mut f in flights.drain(..) {
        // Substeps so a 45 m/s arrow tests every 0.75 m or so.
        let n = ((f.projectile.vel.length() * dt / 0.75).ceil() as usize).clamp(1, 16);
        let mut out = Flight::Flying;
        for _ in 0..n {
            let solid = |p: DVec3| {
                let local = origin.to_local(WorldPos::new(origin.frame, p));
                contact
                    .as_deref()
                    .is_some_and(|c| crate::survival::solid_at(c, origin.query_point(local)))
            };
            let yard_ref = yard.as_deref();
            out = f.projectile.step(cfg, g, dt / n as f64, |a, b| {
                let la = origin.to_local(WorldPos::new(origin.frame, a));
                let lb = origin.to_local(WorldPos::new(origin.frame, b));
                if let Some((i, at)) = yard_ref.and_then(|y| y.crossed(&origin, &roster, la, lb)) {
                    return Some(Struck::Creature(i, origin.to_world(at).pos));
                }
                if solid(b) {
                    // The surface between, by halving.
                    let (mut lo, mut hi) = (a, b);
                    for _ in 0..8 {
                        let mid = (lo + hi) * 0.5;
                        if solid(mid) {
                            hi = mid;
                        } else {
                            lo = mid;
                        }
                    }
                    return Some(Struck::Ground(lo));
                }
                None
            });
            if out != Flight::Flying {
                break;
            }
        }
        let at_local = origin.to_local(f.projectile.at);
        match out {
            Flight::Flying => {
                draw_flight(
                    &mut commands,
                    &mut f,
                    looks.as_deref(),
                    frame.as_deref(),
                    &mut views,
                    at_local,
                );
                keep.push(f);
                continue;
            }
            Flight::Lost => {}
            Flight::Hit {
                creature,
                damage,
                drop,
            } => {
                if let (Some(yard), Some(save), Some(drops), Some(hotbar)) = (
                    yard.as_deref_mut(),
                    save.as_deref_mut(),
                    drops.as_deref_mut(),
                    hotbar.as_deref(),
                ) {
                    let species = yard.health_of(cfg, &roster, creature).map(|(s, _)| s);
                    let data_name = yard
                        .species_of(creature)
                        .and_then(|i| roster.get(i as usize))
                        .map(|s| s.data_name());
                    let long = matches!(f.projectile.missile, Missile::Arrow { .. })
                        && at_local.distance(f.from) >= 20.0;
                    let outcome = yard.strike(cfg, &roster, mode, creature, damage, None, day);
                    info!("COMBAT_PROJECTILE_HIT {species:?} for {damage} -> {outcome:?}");
                    let mut say = None;
                    act_on(
                        outcome,
                        damage,
                        species,
                        long,
                        save,
                        drops,
                        &hotbar.0,
                        &origin,
                        now_s,
                        data_name,
                        &mut landed,
                        &mut did,
                        &mut say,
                    );
                    if let Some(d) = drop {
                        land(save, drops, &hotbar.0, &origin, d, f.good, now_s);
                    }
                }
            }
            Flight::Landed { drop } => {
                info!(
                    "COMBAT_PROJECTILE_LANDED {} kept {}",
                    f.good.code(),
                    drop.is_some()
                );
                if let (Some(d), Some(save), Some(drops), Some(hotbar)) = (
                    drop,
                    save.as_deref_mut(),
                    drops.as_deref_mut(),
                    hotbar.as_deref(),
                ) {
                    land(save, drops, &hotbar.0, &origin, d, f.good, now_s);
                }
            }
        }
        if let Some(e) = f.entity.take() {
            commands.entity(e).despawn();
        }
    }
    combat.flights = keep;
}

/// A landed spear or arrow as a drop where it stopped.
fn land(
    save: &mut WorldSave,
    drops: &mut Drops,
    carried: &pbd_core::inventory::Slots,
    origin: &SimOrigin,
    at: WorldPos,
    good: Good,
    now_s: f64,
) {
    let local = origin.to_local(at);
    let up = origin.up_at(local);
    drop_goods(
        save,
        drops,
        carried,
        origin,
        local,
        up,
        &[(good.code().to_string(), 1)],
        now_s,
    );
}

/// Draw a projectile as its own model, along its flight, lit as the ground.
fn draw_flight(
    commands: &mut Commands,
    f: &mut Flying,
    looks: Option<&crate::drops::DropLooks>,
    frame: Option<&crate::planet::PlanetRenderFrame>,
    views: &mut Query<&mut Transform, With<InFlight>>,
    at: Vec3,
) {
    let Some(frame) = frame else {
        return;
    };
    let dir = f.projectile.vel.as_vec3().normalize_or(Vec3::Y);
    let place = Transform::from_translation(crate::render_origin::render_from_physics(frame, at))
        .with_rotation(Quat::from_rotation_arc(Vec3::Y, dir));
    if let Some(e) = f.entity {
        if let Ok(mut t) = views.get_mut(e) {
            *t = place;
        }
        return;
    }
    let Some((parts, _)) = looks.and_then(|l| l.models.get(&Item::Good(f.good))) else {
        return;
    };
    let length =
        crate::weapon_models::model(f.good).map_or(1.0, |(p, _)| crate::weapon_models::length(&p));
    let e = commands
        .spawn((
            Name::new("in flight"),
            InFlight,
            crate::field_light::LitByField,
            crate::field_light::LitLikeTerrain,
            place,
            Visibility::default(),
        ))
        .with_children(|root| {
            for (mesh, material) in parts {
                root.spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    // Its head ahead: the model runs up +Y from its foot.
                    Transform::from_translation(Vec3::NEG_Y * length),
                ));
            }
        })
        .id();
    f.entity = Some(e);
}

/// The predators' bites, through the server (C2).
pub fn bites(
    yard: Option<ResMut<Farmyard>>,
    mut save: Option<ResMut<WorldSave>>,
    mut survival: Option<ResMut<crate::survival::Survival>>,
    mut sfx: Option<ResMut<crate::audio::Sfx>>,
) {
    let Some(mut yard) = yard else {
        return;
    };
    let bites = yard.take_bites();
    // A predator's warning growls where it crouches (design 6).
    let growls = yard.take_growls();
    if let Some(sfx) = sfx.as_deref_mut() {
        for at in growls {
            info!("COMBAT_GROWL");
            sfx.play("predator_growl", crate::audio::Where::At(at));
        }
    }
    let (Some(save), Some(sv)) = (save.as_deref_mut(), survival.as_deref_mut()) else {
        return;
    };
    for pts in bites {
        info!("COMBAT_BITE {pts}");
        crate::survival::hit(save, sv, Cause::Creature, pts);
    }
}

/// A capture's combat scene (`--combat <scene>`), a measurement instrument:
/// `fox-stalk` (a red fox noticing from 11 m), `fox-warn` (one at 6 m),
/// `hound-warn` (a dust hound at 6 m), `swing-hit` (the weapon in hand
/// pressed at a fox held 1.6 m ahead), `spear-throw` (a spear raised and
/// thrown at a fox 12 m ahead), `guard` (the nearest townsperson struck).
/// The world is made a Survival one; nothing a player meets.
#[derive(Resource, Clone, Debug)]
pub struct CombatScript {
    pub scene: String,
    frame: u32,
}

impl CombatScript {
    pub fn new(scene: impl Into<String>) -> Self {
        Self {
            scene: scene.into(),
            frame: 0,
        }
    }
}

/// Play the capture's scene: the frames of its presses and what it sets.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn script(
    script: Option<ResMut<CombatScript>>,
    space: crate::SimSpace,
    mut walkers: Query<&mut Position, With<Walker>>,
    cameras: Query<&GlobalTransform, With<WalkingCamera>>,
    frame: Option<Res<crate::planet::PlanetRenderFrame>>,
    mut combat: ResMut<Combat>,
    mut walking: Option<ResMut<crate::walking::WalkingState>>,
    (mut yard, folk, config, body): (
        Option<ResMut<Farmyard>>,
        Option<Res<Townsfolk>>,
        Option<Res<crate::config::HusbandryConfig>>,
        Option<Res<crate::fish::Body>>,
    ),
    mut save: Option<ResMut<WorldSave>>,
    contact: Option<Res<crate::planet::PlanetContact>>,
) {
    let Some(mut script) = script else {
        return;
    };
    let origin = space.origin();
    let (Some(walker), Some(frame)) = (walkers.iter().next().copied(), frame.as_deref()) else {
        return;
    };
    let Some((eye, look)) = eye_and_look(frame, &cameras) else {
        return;
    };
    script.frame += 1;
    let n = script.frame;
    if let Some(save) = save.as_deref_mut()
        && !save.mode().is_survival()
    {
        save.identity.mode = Some(WorldMode::Survival);
    }
    let up = origin.up_at(walker.0);
    let ahead = (look - up * look.dot(up)).normalize_or(up.any_orthonormal_vector());
    let feet = origin.with_radius(
        walker.0,
        origin.radius_at(walker.0) - f64::from(crate::walking::HALF_HEIGHT),
    );
    // A point `m` ahead stood on the ground (the yard's own rule).
    // A capture's spot: `m` ahead, turned `turn` rad to the right.
    let right = ahead.cross(up);
    let toward = |turn: f32| ahead * turn.cos() + right * turn.sin();
    let stood_at = |m: f32, turn: f32| {
        let p = feet + toward(turn) * m;
        let d = origin.query_point(p).normalize_or(Vec3::Y);
        let surface = crate::planet::PLANET_RADIUS + crate::planet::surface_height(d).max(0.0);
        let r = contact.as_deref().map_or(f64::from(surface), |c| {
            c.stand(d * (surface + 1.5)).floor_precise
        });
        origin.with_radius(p, r)
    };
    let roster = match (config.as_deref(), body.as_deref()) {
        (Some(c), Some(b)) => roster(c, b),
        _ => return,
    };
    let index = |id: &str| roster.iter().position(|s| s.id == id).map(|i| i as u16);
    let scene = script.scene.clone();
    // Dry ground: a spot in water is moved in toward the player.
    let dry = |m: f32, turn: f32| {
        let mut m = m;
        while m > 1.5 {
            let at = stood_at(m, turn);
            let above = origin.query_point(at + origin.up_at(at) * 0.3);
            let wet = contact
                .as_deref()
                .and_then(|c| c.column_material(above))
                .is_some_and(|mat| mat.is_water());
            if !wet {
                break;
            }
            m -= 0.5;
        }
        m
    };
    let place = |yard: &mut Farmyard, id: &str, m: f32, turn: f32, still: bool| {
        if let Some(sp) = index(id) {
            let m = dry(m, turn);
            let at = stood_at(m, turn);
            let i = yard.place_wild(sp, at, origin.up_at(at), -toward(turn), 1);
            yard.hold_still(i, still);
            info!("COMBAT_SCRIPT {id} at {m} m");
        }
    };
    if n == 30
        && let Some(yard) = yard.as_deref_mut()
    {
        match scene.as_str() {
            "fox-stalk" => place(yard, "red-fox", 9.0, 0.5, false),
            "fox-warn" => place(yard, "red-fox", 6.0, 0.45, false),
            "hound-warn" => place(yard, "dust-hound", 6.5, 0.45, false),
            "swing-hit" => place(yard, "red-fox", 1.7, 0.0, true),
            "spear-throw" => place(yard, "red-fox", 9.0, 0.0, true),
            "fox-lunge" => place(yard, "red-fox", 3.0, 0.0, false),
            _ => {}
        }
    }
    combat.scripted = match scene.as_str() {
        "swing-hit" => Some((n == 60, false)),
        "spear-throw" => Some((false, (60..90).contains(&n))),
        "bow-draw" => Some(((60..400).contains(&n), false)),
        _ => None,
    };
    // The guard scene stands the player outside the town, a few metres out
    // from its gate guard, and looks at the guard.
    let gate_guard = folk.as_deref().and_then(|f| {
        let g = f.everyone().find(|p| p.role == "guard")?;
        Some((g.id, f.placed(g.id)?.feet, town_bounds(f, g.id.site)?.0))
    });
    if scene == "guard"
        && n == 20
        && let (Some((_, g, centre)), Some(contact)) = (gate_guard, contact.as_deref())
        && let Some(mut pos) = walkers.iter_mut().next()
    {
        let gd = g.normalize();
        let out = (gd - centre.normalize()) - gd * (gd - centre.normalize()).dot(gd);
        let r = f64::from(crate::planet::PLANET_RADIUS);
        let d = (gd + out.normalize_or(DVec3::X) * (7.0 / r)).normalize();
        let df = d.as_vec3();
        let surface = crate::planet::PLANET_RADIUS + crate::planet::surface_height(df).max(0.0);
        let floor = contact.stand(df * (surface + 1.5)).floor_precise;
        pos.0 = origin.at(d, floor + f64::from(crate::walking::HALF_HEIGHT) + 0.05);
        info!("COMBAT_SCRIPT stood 7 m out from the gate guard");
    }
    if scene == "guard"
        && (22..40).contains(&n)
        && let (Some((_, g, _)), Some(walking)) = (gate_guard, walking.as_deref_mut())
    {
        let at = origin.to_local(WorldPos::new(origin.frame, g));
        walking.look_toward(at + origin.up_at(at) * 1.0 - eye);
    }
    if scene == "guard"
        && n == 30
        && let Some(folk) = folk.as_deref()
    {
        // Where each of the town's people stands, for choosing a capture's
        // spot (the log).
        for p in folk.everyone() {
            if let Some(pl) = folk.placed(p.id) {
                let (lat, lon) = pbd_core::geo::lat_lon(pl.feet.as_vec3().normalize()).degrees();
                info!(
                    "COMBAT_SCRIPT person {:?} {} spear {} at {lat:.6} {lon:.6}",
                    p.id, p.role, p.spear
                );
            }
        }
    }
    if scene == "guard" && n == 40 {
        let Some(folk) = folk.as_deref() else {
            return;
        };
        let at = origin.to_world(feet).pos;
        let nearest = folk
            .everyone()
            .filter_map(|p| folk.placed(p.id).map(|pl| (p.id, pl.feet.distance(at))))
            .min_by(|a, b| a.1.total_cmp(&b.1));
        if let (Some((id, d)), Some(save)) = (nearest, save.as_deref_mut()) {
            let after = strike_person(crate::survival::rules(), save, WorldMode::Survival, id.site);
            start_fight(&mut combat, folk, crate::survival::rules(), id);
            info!(
                "COMBAT_SCRIPT struck {id:?} {d:.1} m away, standing {after:?}, {} guards",
                combat.guards()
            );
        }
    }
    if n % 10 == 0 || (scene == "fox-lunge" && (130..190).contains(&n)) {
        let hunts = yard
            .as_deref()
            .map(|y| y.hunts(&roster))
            .unwrap_or_default();
        info!(
            "COMBAT_SCRIPT frame {n} hunts {hunts:?} flying {}",
            combat.flying()
        );
    }
}

/// The combat rule's systems.
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Combat>()
            .init_resource::<CombatClaim>()
            .init_resource::<SwingNow>()
            .add_message::<Landed>()
            .add_message::<crate::missions::Did>()
            .add_systems(
                Update,
                (
                    script.run_if(resource_exists::<CombatScript>),
                    swing.in_set(CombatSet),
                    fly,
                    town_fight,
                    bites,
                )
                    .chain()
                    .run_if(crate::world_open),
            );
    }
}

#[cfg(test)]
mod tests;
