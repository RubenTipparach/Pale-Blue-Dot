//! Gathering the sea floor (`sea-gathering`): F on a reef piece, a kelp
//! stand or a mussel bed aimed at as it is drawn takes it by hand (design
//! 1 to 3); F on a crab, lobster, clam, octopus or tube worm takes it, and
//! on a protected creature says it is protected (design 4); right-click with
//! a coral, sponge, sea fan or anemone held sets it down as an ornament,
//! and F takes one back up (design 8).
//!
//! The rules are the core's (`pbd_core::sea_gather`); where the pieces
//! stand and the aim at them are `fish::places`'.

use super::farm::FarmNote;
use super::menu::Screen;
use super::slots::Hotbar;
use bevy::prelude::*;
use pbd_app::fish::Fishery;
use pbd_app::fish::places::{self, Pickable, SeaAim, SeaTakes};
use pbd_app::hotbar::{Gained, How};
use pbd_app::interact::{Act, Interact, Offer, Reader};
use pbd_app::planet::{PlanetContact, PlanetFine, PlanetRenderFrame};
use pbd_app::saves::WorldSave;
use pbd_core::daylight::DAY_S;
use pbd_core::fauna::{Mover, Species, Status};
use pbd_core::inventory::Item;
use pbd_core::locale::{Localized, tr};
use pbd_core::records::Record;
use pbd_core::sea_gather::{
    ORNAMENT_RECORD, ORNAMENT_SCHEMA, Ornament, SEA_TAKE_RECORD, SEA_TAKE_SCHEMA, SeaPiece,
    SeaTake, hand_refusal,
};

/// The world day the sun's clock reads.
fn day_of(sun: Option<&pbd_app::sky::Sun>) -> f64 {
    sun.map_or(0.0, |s| s.clock.seconds / f64::from(DAY_S))
}

/// The walking camera's eye and look, planet-local.
fn eye_ray(
    cameras: &Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    frame: Option<&PlanetRenderFrame>,
) -> Option<(Vec3, Vec3, Vec3)> {
    let (transform, _) = cameras.iter().find(|(_, c)| c.is_active)?;
    let render = transform.translation();
    Some((
        pbd_app::render_origin::local_of(frame, render),
        transform.forward().as_vec3(),
        render,
    ))
}

/// A crawler, or a protected creature, on the ray within reach: its school,
/// its place in it, its species' index and how far along the ray.
fn creature_on_ray(
    fishery: &Fishery,
    roster: &[Species],
    eye_sim: Vec3,
    look: Vec3,
    alpha: f32,
) -> Option<(usize, usize, u16, f32)> {
    let mut best: Option<(usize, usize, u16, f32)> = None;
    for (s, school) in fishery.schools.iter().enumerate() {
        let Some(species) = roster.get(school.species as usize) else {
            continue;
        };
        if species.mover != Mover::Crawler && species.status != Status::Protected {
            continue;
        }
        let r = (species.length_m * 0.6).clamp(0.25, 0.6);
        for i in 0..school.len() {
            let at = school.drawn(i, alpha);
            let along = (at - eye_sim).dot(look);
            if !(0.0..=places::REACH_M).contains(&along) {
                continue;
            }
            let gap = (eye_sim + look * along).distance(at);
            if gap <= r && best.is_none_or(|b| along < b.3) {
                best = Some((s, i, school.species, along));
            }
        }
    }
    best
}

/// An ornament on the ray within reach: its record's id, its body and how
/// far along the ray.
fn ornament_on_ray(save: &WorldSave, eye: Vec3, look: Vec3) -> Option<(u64, Ornament, f32)> {
    let mut best: Option<(u64, Ornament, f32)> = None;
    for (id, body, piece) in places::standing_ornaments(save) {
        let at = Vec3::new(body.at.0 as f32, body.at.1 as f32, body.at.2 as f32);
        let (height, radius) = places::ornament_extent(piece);
        let stem = Pickable {
            id,
            what: piece,
            foot: at,
            up: at.normalize_or(Vec3::Y),
            height,
            radius,
        };
        if let Some(along) = places::ray_meets(eye, look, &stem, places::REACH_M)
            && best.as_ref().is_none_or(|b| along < b.2)
        {
            best = Some((id, body, along));
        }
    }
    best
}

/// What F does here: the nearest of a piece, a creature and an ornament.
enum SeaAct {
    Piece(places::Aimed),
    Creature {
        school: usize,
        index: usize,
        species: u16,
    },
    Ornament(u64, Ornament),
}

fn sea_act(
    aim: &SeaAim,
    creature: Option<(usize, usize, u16, f32)>,
    ornament: Option<(u64, Ornament, f32)>,
) -> Option<SeaAct> {
    let mut best: Option<(f32, SeaAct)> = aim.0.map(|a| (a.along_m, SeaAct::Piece(a)));
    if let Some((school, index, species, along)) = creature
        && best.as_ref().is_none_or(|b| along < b.0)
    {
        best = Some((
            along,
            SeaAct::Creature {
                school,
                index,
                species,
            },
        ));
    }
    if let Some((id, body, along)) = ornament
        && best.as_ref().is_none_or(|b| along < b.0)
    {
        best = Some((along, SeaAct::Ornament(id, body)));
    }
    best.map(|(_, act)| act)
}

/// The world round F's offers and press.
#[derive(bevy::ecs::system::SystemParam)]
pub struct SeaWorld<'w, 's> {
    cameras: Query<'w, 's, (&'static GlobalTransform, &'static Camera), With<Camera3d>>,
    frame: Option<Res<'w, PlanetRenderFrame>>,
    aim: Res<'w, SeaAim>,
    fishery: Option<ResMut<'w, Fishery>>,
    fauna: Option<Res<'w, pbd_app::config::FaunaConfig>>,
    body: Option<Res<'w, pbd_app::fish::Body>>,
    walking: Option<Res<'w, pbd_app::walking::WalkingReadout>>,
}

impl SeaWorld<'_, '_> {
    fn act(&self, save: Option<&WorldSave>) -> Option<SeaAct> {
        if !self.walking.as_ref().is_some_and(|w| w.active) {
            return None;
        }
        let (eye, look, render) = eye_ray(&self.cameras, self.frame.as_deref())?;
        let creature = match (&self.fishery, &self.fauna, &self.body, &self.frame) {
            (Some(fishery), Some(fauna), Some(body), Some(frame)) => {
                let roster = fauna.0.roster(&body.0);
                let eye_sim = pbd_app::sim_origin::local_from_render(frame, render);
                creature_on_ray(
                    fishery,
                    roster,
                    eye_sim,
                    look,
                    fishery.step_alpha(fauna.0.spawn.step_hz),
                )
            }
            _ => None,
        };
        let ornament = save.and_then(|s| ornament_on_ray(s, eye, look));
        sea_act(&self.aim, creature, ornament)
    }

    fn species(&self, index: u16) -> Option<&Species> {
        let (fauna, body) = (self.fauna.as_ref()?, self.body.as_ref()?);
        fauna.0.roster(&body.0).get(index as usize)
    }
}

/// Offer what F would do on the sea floor: `Gather <item>`, `Take up
/// <item>`.
pub fn offer(world: SeaWorld, save: Option<Res<WorldSave>>, mut interact: ResMut<Interact>) {
    let Some(act) = world.act(save.as_deref()) else {
        return;
    };
    let offer = match act {
        SeaAct::Piece(aimed) => Offer::new(Act::GatherSea, aimed.what.name()),
        SeaAct::Creature { species, .. } => {
            let Some(sp) = world.species(species) else {
                return;
            };
            Offer::new(Act::GatherSea, sp.name())
        }
        SeaAct::Ornament(_, body) => {
            let Some(piece) = body.piece() else {
                return;
            };
            Offer::new(Act::TakeOrnament, piece.name())
        }
    };
    interact.offer(offer);
}

/// F on the sea floor: take the piece, the crawler or the ornament aimed at,
/// with the pack, in one durable line; refuse a protected creature.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn use_key(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    screen: Res<Screen>,
    mut world: SeaWorld,
    interact: Option<Res<Interact>>,
    (mut save, mut slots, mut note, mut takes): (
        Option<ResMut<WorldSave>>,
        Option<ResMut<Hotbar>>,
        Option<ResMut<FarmNote>>,
        ResMut<SeaTakes>,
    ),
    sun: Option<Res<pbd_app::sky::Sun>>,
    time: Res<Time>,
    mut gained: MessageWriter<Gained>,
) {
    if !keys.just_pressed(KeyCode::KeyF) || *screen != Screen::Playing {
        return;
    }
    if !world
        .walking
        .as_ref()
        .is_some_and(|w| w.active && w.captured)
    {
        return;
    }
    if !pbd_app::interact::may(interact.as_deref(), Reader::Sea) {
        return;
    }
    let (Some(save), Some(slots), Some(note)) = (
        save.as_deref_mut(),
        slots.as_deref_mut(),
        note.as_deref_mut(),
    ) else {
        return;
    };
    let Some(act) = world.act(Some(save)) else {
        return;
    };
    keys.clear_just_pressed(KeyCode::KeyF);
    let wall_s = time.elapsed_secs();
    let say = |note: &mut FarmNote, text: Localized| note.say(wall_s, text);
    match act {
        SeaAct::Piece(aimed) => {
            let item = Item::Good(aimed.what.good());
            let mut moved = slots.0.clone();
            if moved.give(item, 1) != 0 {
                say(note, tr("farm.no_room"));
                return;
            }
            let day = day_of(sun.as_deref());
            let line = Record::of(
                SEA_TAKE_RECORD,
                aimed.id,
                SEA_TAKE_SCHEMA,
                &SeaTake {
                    day,
                    code: aimed.what.code().into(),
                },
            );
            if !save.store_with_hands(line, SEA_TAKE_SCHEMA, &moved) {
                error!("sea take BLOCKED: the save did not accept it");
                return;
            }
            slots.0 = moved;
            takes.taken.insert(aimed.id);
            takes.dirty.push(aimed.key);
            gained.write(Gained {
                item,
                count: 1,
                how: How::Gathered,
            });
            info!(
                "SEA_TAKE {} id {} tile {:?} day {day:.2}",
                aimed.what.code(),
                aimed.id,
                aimed.key
            );
        }
        SeaAct::Creature {
            school,
            index,
            species,
        } => {
            let Some(sp) = world.species(species).cloned() else {
                return;
            };
            if let Some(why) = hand_refusal(&sp) {
                say(note, why);
                return;
            }
            let Some(fishery) = world.fishery.as_deref_mut() else {
                return;
            };
            let Some(length_cm) = fishery.schools.get(school).map(|s| s.length_cm(index, &sp))
            else {
                return;
            };
            let item = Item::Fish(species);
            let mut moved = slots.0.clone();
            if moved.give(item, 1) != 0 {
                say(note, tr("farm.no_room"));
                return;
            }
            if !save.record_catch(species, length_cm, &moved) {
                error!("sea creature BLOCKED: the save did not accept it");
                return;
            }
            slots.0 = moved;
            fishery.schools[school].remove(index);
            gained.write(Gained {
                item,
                count: 1,
                how: How::Gathered,
            });
            info!("SEA_CREATURE {} {length_cm} cm", sp.id);
        }
        SeaAct::Ornament(id, body) => {
            let Some(piece) = body.piece() else {
                return;
            };
            let item = Item::Good(piece.good());
            let mut moved = slots.0.clone();
            if moved.give(item, 1) != 0 {
                say(note, tr("farm.no_room"));
                return;
            }
            let line = Record::of(
                ORNAMENT_RECORD,
                id,
                ORNAMENT_SCHEMA,
                &Ornament {
                    standing: false,
                    ..body
                },
            );
            if !save.store_with_hands(line, ORNAMENT_SCHEMA, &moved) {
                error!("ornament take BLOCKED: the save did not accept it");
                return;
            }
            slots.0 = moved;
            gained.write(Gained {
                item,
                count: 1,
                how: How::Collected,
            });
            info!("ORNAMENT_TAKEN {} cell {id}", piece.code());
        }
    }
}

/// Right-click with a coral, sponge, sea fan or anemone held: set it on the
/// aimed ground, on land or the sea floor, one to a cell (design 8).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn place(
    buttons: Res<ButtonInput<MouseButton>>,
    screen: Res<Screen>,
    walking: Option<Res<pbd_app::walking::WalkingReadout>>,
    aimed: Option<Res<pbd_app::husbandry::AimedGround>>,
    (fine, contact): (Option<Res<PlanetFine>>, Option<Res<PlanetContact>>),
    (mut save, mut slots, mut note): (
        Option<ResMut<WorldSave>>,
        Option<ResMut<Hotbar>>,
        Option<ResMut<FarmNote>>,
    ),
    time: Res<Time>,
) {
    if !buttons.just_pressed(MouseButton::Right) || *screen != Screen::Playing {
        return;
    }
    if !walking.is_some_and(|w| w.active && w.captured) {
        return;
    }
    let (Some(save), Some(slots), Some(note)) = (
        save.as_deref_mut(),
        slots.as_deref_mut(),
        note.as_deref_mut(),
    ) else {
        return;
    };
    let Some(piece) = slots.0.held().and_then(|s| match s.item {
        Item::Good(good) => SeaPiece::of_good(good).filter(|p| p.ornament()),
        _ => None,
    }) else {
        return;
    };
    let Some(point) = aimed.and_then(|a| a.0) else {
        return;
    };
    let (Some(fine), Some(contact)) = (fine, contact) else {
        return;
    };
    let wall_s = time.elapsed_secs();
    if !contact.serves(&fine.set) {
        return;
    }
    let Some(cell) = contact
        .finest_cell(point.normalize_or(Vec3::Y))
        .and_then(|i| fine.set.finest_records().get(i).map(|r| r.key()))
    else {
        return;
    };
    let id = u64::from(cell);
    if save
        .records
        .get(ORNAMENT_RECORD, id)
        .and_then(|r| r.read::<Ornament>())
        .is_some_and(|o| o.standing)
    {
        note.say(wall_s, tr("sea.occupied"));
        return;
    }
    let mut moved = slots.0.clone();
    if moved.take_held().is_none() {
        return;
    }
    let p = point.as_dvec3();
    let yaw = (pbd_core::fauna::hash2(u64::from(cell), 0x0a4e) >> 40) as f32 / (1u64 << 24) as f32
        * std::f32::consts::TAU;
    let body = Ornament {
        code: piece.code().into(),
        at: (p.x, p.y, p.z),
        yaw,
        standing: true,
    };
    if !save.store_with_hands(
        Record::of(ORNAMENT_RECORD, id, ORNAMENT_SCHEMA, &body),
        ORNAMENT_SCHEMA,
        &moved,
    ) {
        error!("ornament BLOCKED: the save did not accept it");
        return;
    }
    slots.0 = moved;
    info!("ORNAMENT_SET {} cell {cell}", piece.code());
}

/// Keep [`SeaTakes`] to the save: the takes not yet grown back, read again
/// when the save's records change and every few seconds for a regrowth. A
/// save read after tiles were built has them rebuilt.
pub fn refresh_takes(
    save: Option<Res<WorldSave>>,
    sun: Option<Res<pbd_app::sky::Sun>>,
    mut takes: ResMut<SeaTakes>,
    time: Res<Time>,
    mut seen: Local<(usize, f32, Option<String>)>,
) {
    let Some(save) = save else {
        return;
    };
    let (count, wait, slot) = &mut *seen;
    *wait -= time.delta_secs();
    let this_slot = save.slot().map(|s| s.id.clone());
    let opened = *slot != this_slot;
    if !opened && save.records.len() == *count && *wait > 0.0 {
        return;
    }
    *count = save.records.len();
    *wait = 3.0;
    *slot = this_slot;
    let day = day_of(sun.as_deref());
    let now: std::collections::HashSet<u64> = save
        .records
        .of_kind(SEA_TAKE_RECORD)
        .filter(|r| r.read::<SeaTake>().is_some_and(|t| !t.due_at(day)))
        .map(|r| r.id)
        .collect();
    let grew = now.iter().any(|id| !takes.taken.contains(id));
    if opened && grew {
        takes.rebuild_all = true;
    }
    if takes.taken != now {
        takes.taken = now;
    }
}

/// `PBD_SEA_SHOT=<code|any>:<frame>[:<code to hold>]`, a capture instrument
/// (`sea-gathering` 4.1): once the near tiles stand round the walker, the
/// nearest gatherable piece of that kind is chosen; the look turns onto it
/// every frame and W swims or walks toward it until it is in reach; F is
/// pressed on the FRAME-th frame after arriving, and [`use_key`] does the
/// rest as it does for a player. A third part puts one of that good in the
/// hand first, so a right-click (the key script's) sets it down. Nothing is
/// stored by the script.
pub fn shot_wanted() -> Option<(Option<SeaPiece>, u32, Option<SeaPiece>)> {
    let value = std::env::var("PBD_SEA_SHOT").ok()?;
    let mut parts = value.split(':');
    let what = parts.next()?;
    let frame = parts.next()?.parse().ok()?;
    let hold = parts.next().and_then(SeaPiece::by_code);
    Some((SeaPiece::by_code(what), frame, hold))
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn shot_script(
    mut commands: Commands,
    near: Option<Res<places::SeaNear>>,
    (cameras, frame): (
        Query<(&GlobalTransform, &Camera), With<Camera3d>>,
        Option<Res<PlanetRenderFrame>>,
    ),
    (mut keys, mut mouse): (
        ResMut<ButtonInput<KeyCode>>,
        ResMut<ButtonInput<MouseButton>>,
    ),
    mut state: ResMut<pbd_app::walking::WalkingState>,
    mut slots: Option<ResMut<Hotbar>>,
    mut count: Local<u32>,
    mut target: Local<Option<Vec3>>,
    mut arrived: Local<bool>,
) {
    let Some((want, press_at, hold)) = shot_wanted() else {
        return;
    };
    let Some(near) = near else {
        commands.insert_resource(places::SeaNear { want, found: None });
        return;
    };
    if !state.active {
        return;
    }
    state.scripted = true;
    state.captured = true;
    if let (Some(hold), Some(slots)) = (hold, slots.as_deref_mut())
        && slots.0.count(Item::Good(hold.good())) == 0
    {
        slots.0.give(Item::Good(hold.good()), 1);
        let at = (0..pbd_core::inventory::SLOTS).find(|i| {
            slots
                .0
                .get(*i)
                .is_some_and(|s| s.item == Item::Good(hold.good()))
        });
        if let Some(at) = at {
            slots.0.select(at);
        }
    }
    let Some((transform, _)) = cameras.iter().find(|(_, c)| c.is_active) else {
        return;
    };
    let eye = pbd_app::render_origin::local_of(frame.as_deref(), transform.translation());
    // Setting one down: look at the ground two metres ahead and right-click
    // on the frame.
    if hold.is_some() {
        let up = eye.normalize_or(Vec3::Y);
        let ahead = transform
            .forward()
            .as_vec3()
            .reject_from(up)
            .normalize_or(Vec3::X);
        state.look_toward(ahead * 2.0 - up * 1.4);
        *count += 1;
        if *count == press_at {
            mouse.press(MouseButton::Right);
            info!("SEA_SHOT right-clicked on frame {press_at}");
        } else if *count == press_at + 1 {
            mouse.release(MouseButton::Right);
        }
        return;
    }
    if target.is_none() {
        *count += 1;
        if *count < 60 {
            return;
        }
        let Some(found) = near.found else {
            if (*count).is_multiple_of(120) {
                warn!("SEA_SHOT nothing of {want:?} within 30 m yet");
            }
            return;
        };
        let pbd_app::place::Place { lat, lon, .. } =
            pbd_app::place::Place::of(found.normalize_or(Vec3::Y));
        info!(
            "SEA_SHOT going to {want:?} {:.2} m from the eye, at {lat:.5} {lon:.5}",
            found.distance(eye)
        );
        *target = Some(found);
        *count = 0;
    }
    let Some(at) = *target else {
        return;
    };
    state.look_toward(at - eye);
    let d = (at - eye).length();
    if !*arrived {
        *count += 1;
        if d > 1.6 && *count < 900 {
            keys.press(KeyCode::KeyW);
            return;
        }
        keys.release(KeyCode::KeyW);
        *arrived = true;
        *count = 0;
        info!("SEA_SHOT in reach, {d:.2} m from the eye");
    }
    *count += 1;
    if *count == press_at {
        keys.press(KeyCode::KeyF);
        info!("SEA_SHOT pressed F on frame {press_at} in reach");
    } else if *count == press_at + 1 {
        keys.release(KeyCode::KeyF);
    }
}
