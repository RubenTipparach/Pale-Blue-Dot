//! Furniture in the game (`furniture` design 5 to 11): every seat, bed and
//! container the player can reach, in the towns and of their own; F to sit,
//! lie down or open one; the townsfolk in their seats by day and their beds
//! by night, held to one occupancy rule with the player; and a container's
//! panel under the pack, its moves written with the slots in one line.
//!
//! The rules are the core's (`pbd_core::furniture`). A town's pieces stand
//! where its cut laid them (`BuildingSolids::furniture`); a player's where
//! their `piece` record says (`building::cut::furniture_stand`).

use super::farm::FarmNote;
use super::menu::Screen;
use super::slots::{self, Hotbar, ItemIcons, SlotCell};
use bevy::math::DVec3;
use bevy::prelude::*;
use pbd_app::builds::{Builds, PATCH_REACH_M, patch_round};
use pbd_app::interact::{Act, Interact, Offer, Reader};
use pbd_app::planet::PLANET_RADIUS;
use pbd_app::saves::WorldSave;
use pbd_app::towns::Towns;
use pbd_app::townsfolk::{Posture, Townsfolk};
use pbd_app::ui_kit::{self, KitText, Localized, Plaque, UiKit, tr};
use pbd_app::walking::{EYE_HEIGHT, HALF_HEIGHT, SeatedEye, Structures, Walker, WalkingState};
use pbd_core::building::{Cells, Shape};
use pbd_core::economy::coins::{PURSE_ID, PURSE_KIND, PURSE_SCHEMA, Reason};
use pbd_core::furniture::store::{STORE_KIND, STORE_SCHEMA, Store, Trade, loot};
use pbd_core::furniture::{
    Kind, Occupancy, Place, Spot, Use, Want, Who, assign, keeps_watch, town_piece_id, want,
};
use pbd_core::inventory::{CARRIED, SLOTS};
use pbd_core::locale::tr_with;
use pbd_core::missions::Kind as MissionKind;
use pbd_core::records::{Author, Record as StoredRecord};
use pbd_core::townsfolk::{PersonId, Work};

/// How far F reaches for a piece, metres, from the eye.
pub const REACH_M: f32 = 2.4;
/// How far a townsperson goes for a seat from their post, and for a bed
/// within their town, metres.
pub const SEAT_REACH_M: f64 = 14.0;
pub const BED_REACH_M: f64 = 160.0;
/// The seated eye over the seat, and the lying eye over the mattress, m.
pub const SEATED_EYE_M: f32 = 0.7;
pub const LYING_EYE_M: f32 = 0.24;
/// The first pack square of a container's panel: past the raft hold's.
pub const STORE_AT: usize = CARRIED + 64;
/// The most slots a container has (`Kind::BandedChest`'s 24, in rows of ten).
pub const STORE_SQUARES: usize = 30;

/// Whose a piece is: a town's (its site, its building's name and its
/// household's trade, for its loot) or the player's.
#[derive(Clone, Debug, PartialEq)]
pub enum Owner {
    Town {
        site: u32,
        house: String,
        trade: Trade,
    },
    Player,
}

/// A piece the player can reach: its id, kind, its middle on its floor and
/// its front (planet frame), and whose it is.
#[derive(Clone, Debug, PartialEq)]
pub struct Piece {
    pub id: u64,
    pub kind: Kind,
    pub at: Vec3,
    pub front: Vec3,
    pub owner: Owner,
}

impl Piece {
    fn up(&self) -> Vec3 {
        self.at.normalize_or(Vec3::Y)
    }

    /// The across of its own frame, in the planet frame.
    fn across(&self) -> Vec3 {
        self.up().cross(self.front).normalize_or_zero()
    }

    /// A point of its own frame (x across, y up, z front) on the planet.
    pub fn point(&self, local: Vec3) -> Vec3 {
        self.at + self.across() * local.x + self.up() * local.y + self.front * local.z
    }

    fn spot(&self) -> Spot {
        Spot {
            id: self.id,
            kind: self.kind,
            at: self.at.as_dvec3(),
        }
    }
}

/// Every piece standing near enough to matter: the standing towns' and the
/// player's, rebuilt when either changes.
#[derive(Resource, Default)]
pub struct Pieces {
    pub all: Vec<Piece>,
    key: Option<(usize, Vec<u32>, u64)>,
}

/// Who holds each seat and bed (`furniture` design 7).
#[derive(Resource, Default)]
pub struct Seats(pub Occupancy);

/// Where the player sits or lies, while they do.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq)]
pub struct Sitting(pub Option<(Place, Kind)>);

/// The container open under the pack, and what it holds.
#[derive(Resource, Default)]
pub struct StoreView {
    pub open: Option<OpenStore>,
}

pub struct OpenStore {
    pub id: u64,
    pub title: Localized,
    pub site: Option<u32>,
    pub store: Store,
}

#[derive(Component)]
pub struct StorePanel;
#[derive(Component)]
pub struct StoreTitle;
#[derive(Component)]
pub struct StoreCoins;
#[derive(Component)]
pub struct StoreCell(pub usize);

/// Rebuild the pieces when the standing towns or the player's pieces change.
pub fn gather(
    mut pieces: ResMut<Pieces>,
    structures: Option<Res<Structures>>,
    towns: Res<Towns>,
    builds: Res<Builds>,
    folk: Option<Res<Townsfolk>>,
) {
    let sites: Vec<u32> = towns.standing.iter().map(|s| s.site).collect();
    let len = structures.as_ref().map_or(0, |s| s.0.len());
    let key = (len, sites.clone(), builds.revision);
    if pieces.key.as_ref() == Some(&key) {
        return;
    }
    let mut all = Vec::new();
    if let Some(structures) = &structures {
        for &site in &sites {
            let held = towns.held.iter().find(|h| h.site.id == site);
            let roles: Vec<String> = folk
                .as_ref()
                .map(|f| f.people(site).map(|p| p.role.clone()).collect())
                .unwrap_or_default();
            let trader = roles.iter().any(|r| Trade::of_role(r) == Trade::Trader);
            for n in 0.. {
                let Some(index) = towns.index(site, n) else {
                    break;
                };
                let Some(b) = structures.0.get(index) else {
                    break;
                };
                if b.furniture.is_empty() {
                    continue;
                }
                let house = held
                    .and_then(|h| h.town.buildings.get(n))
                    .map(|b| b.name.clone())
                    .unwrap_or_default();
                let public = matches!(
                    pbd_core::settlement::pieces::furnish::use_of(&house),
                    Some(pbd_core::settlement::pieces::furnish::Use::Inn)
                        | Some(pbd_core::settlement::pieces::furnish::Use::Hall)
                );
                let trade = if public && trader {
                    Trade::Trader
                } else if roles.is_empty() {
                    Trade::Common
                } else {
                    Trade::of_role(&roles[(n * 7 + 3) % roles.len()])
                };
                for (k, f) in b.furniture.iter().enumerate() {
                    all.push(Piece {
                        id: town_piece_id(site, n as u32, k as u32),
                        kind: f.kind,
                        at: f.at,
                        front: f.front,
                        owner: Owner::Town {
                            site,
                            house: house.clone(),
                            trade,
                        },
                    });
                }
            }
        }
    }
    // The player's: each piece of furniture stood where its record says.
    for piece in builds.pieces.values() {
        let Shape::Furniture {
            cell,
            layer,
            side,
            kind,
        } = piece.shape
        else {
            continue;
        };
        let Some(direction) = pbd_core::cell_key::key_direction(cell) else {
            continue;
        };
        let patch = patch_round(direction, PATCH_REACH_M.min(8.0));
        let cells = Cells::new(&patch);
        if let Some((at, front)) =
            pbd_core::building::cut::furniture_stand(&cells, cell, layer, side, kind, PLANET_RADIUS)
        {
            all.push(Piece {
                id: piece.id(),
                kind,
                at,
                front,
                owner: Owner::Player,
            });
        }
    }
    info!("FURNITURE {} pieces within reach of play", all.len());
    pieces.all = all;
    pieces.key = Some(key);
}

/// The eye and the look of the walker, in the planet frame.
fn eye_and_look(walking: &WalkingState, position: Vec3) -> (Vec3, Vec3) {
    let up = position.normalize_or(Vec3::Y);
    let eye = position + up * (EYE_HEIGHT - HALF_HEIGHT);
    let (heading, pitch) = walking.view();
    (eye, heading * pitch.cos() + up * pitch.sin())
}

/// The piece the eye is on: in reach and in front, the nearest to the look.
fn aimed(pieces: &Pieces, eye: Vec3, look: Vec3) -> Option<&Piece> {
    pieces
        .all
        .iter()
        .filter(|p| p.kind.use_() != Use::Surface)
        .filter_map(|p| {
            let middle = p.point(Vec3::new(0.0, p.kind.height_m().min(1.0) * 0.6, 0.0));
            let to = middle - eye;
            let dist = to.length();
            let ahead = to.dot(look);
            let off = (to - look * ahead).length();
            let room = p.kind.size_m().max_element() * 0.6 + 0.2;
            (dist <= REACH_M && ahead > 0.0 && off <= room).then_some((off, p))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, p)| p)
}

/// The act F does on a piece.
fn act_of(kind: Kind) -> Option<Act> {
    match kind.use_() {
        Use::Seat { .. } => Some(Act::Sit),
        Use::Bed { .. } => Some(Act::Lie),
        Use::Store { .. } => Some(Act::OpenStore),
        Use::Surface => None,
    }
}

/// Offer the piece in reach to the interact key.
pub fn offer(
    walking: Option<Res<WalkingState>>,
    walker: Query<&avian3d::prelude::Position, With<Walker>>,
    pieces: Res<Pieces>,
    sitting: Res<Sitting>,
    mut interact: ResMut<Interact>,
    space: pbd_app::SimSpace,
) {
    let (Some(walking), Ok(position)) = (walking, walker.single()) else {
        return;
    };
    if !walking.active || sitting.0.is_some() {
        return;
    }
    let at = space.origin().query_point(position.0);
    let (eye, look) = eye_and_look(&walking, at);
    if let Some(p) = aimed(&pieces, eye, look)
        && let Some(act) = act_of(p.kind)
    {
        interact.offer(Offer::new(act, p.kind.name()));
    }
}

/// The player's seat's or bed's eye, and the way they face, for a place.
fn eye_of(piece: &Piece, place: u8) -> Option<(Vec3, Vec3)> {
    let (local, _) = piece.kind.place_at(place)?;
    Some(match piece.kind.use_() {
        Use::Bed { .. } => (
            piece.point(local + Vec3::new(0.0, LYING_EYE_M, 0.1)),
            piece.front,
        ),
        _ => (
            piece.point(local + Vec3::new(0.0, SEATED_EYE_M, 0.02)),
            piece.front,
        ),
    })
}

/// F: sit, lie down or open the piece in reach; F again (or a movement key,
/// or a jump) gets the player up.
#[allow(clippy::too_many_arguments)]
pub fn use_key(
    keys: Res<ButtonInput<KeyCode>>,
    mut walking: Option<ResMut<WalkingState>>,
    walker: Query<&avian3d::prelude::Position, With<Walker>>,
    pieces: Res<Pieces>,
    mut seats: ResMut<Seats>,
    mut sitting: ResMut<Sitting>,
    mut eye: ResMut<SeatedEye>,
    mut view: ResMut<StoreView>,
    mut screen: ResMut<Screen>,
    save: Res<WorldSave>,
    interact: Option<Res<Interact>>,
    mut note: ResMut<FarmNote>,
    time: Res<Time>,
    mut did: MessageWriter<pbd_app::missions::Did>,
    space: pbd_app::SimSpace,
) {
    let Some(walking) = walking.as_mut() else {
        return;
    };
    if *screen != Screen::Playing {
        return;
    }
    // Up again: F, a movement key or a jump.
    if sitting.0.is_some() {
        let up = keys.just_pressed(KeyCode::KeyF)
            || keys.any_just_pressed([
                KeyCode::KeyW,
                KeyCode::KeyA,
                KeyCode::KeyS,
                KeyCode::KeyD,
                KeyCode::Space,
            ]);
        if up {
            seats.0.release(Who::Player);
            sitting.0 = None;
            eye.0 = None;
            info!("FURNITURE up");
        }
        return;
    }
    if !keys.just_pressed(KeyCode::KeyF) || !walking.active {
        return;
    }
    if !(walking.captured || walking.scripted) {
        return;
    }
    if !pbd_app::interact::may(interact.as_deref(), Reader::Furniture) {
        return;
    }
    let Ok(position) = walker.single() else {
        return;
    };
    let at = space.origin().query_point(position.0);
    let (eye_at, look) = eye_and_look(walking, at);
    let Some(piece) = aimed(&pieces, eye_at, look).cloned() else {
        return;
    };
    let now = time.elapsed_secs();
    match piece.kind.use_() {
        Use::Seat { .. } | Use::Bed { .. } => {
            // The free place nearest the eye (a bench's two).
            let place = (0..piece.kind.places())
                .filter(|&k| seats.0.holder((piece.id, k)).is_none())
                .min_by(|&a, &b| {
                    let d = |k| {
                        piece
                            .kind
                            .place_at(k)
                            .map_or(f32::MAX, |(l, _)| piece.point(l).distance(eye_at))
                    };
                    d(a).total_cmp(&d(b))
                });
            let Some(place) = place else {
                note.say(now, tr("furniture.occupied"));
                return;
            };
            if seats.0.claim((piece.id, place), Who::Player).is_err() {
                note.say(now, tr("furniture.occupied"));
                return;
            }
            let Some((seat_eye, facing)) = eye_of(&piece, place) else {
                return;
            };
            sitting.0 = Some(((piece.id, place), piece.kind));
            eye.0 = Some(seat_eye.as_dvec3());
            walking.look_toward(facing);
            note.say(now, tr("furniture.get_up"));
            let (kind, what) = match piece.kind.use_() {
                Use::Bed { .. } => (MissionKind::Lie, "bed"),
                _ => (MissionKind::Sit, piece.kind.code()),
            };
            did.write(pbd_app::missions::Did::new(kind, what, 1));
            info!("FURNITURE {} on {} {}", what, piece.kind.code(), piece.id);
        }
        Use::Store { .. } => {
            let (site, fresh): (Option<u32>, Box<dyn FnOnce() -> Store>) = match &piece.owner {
                Owner::Town { site, trade, .. } => {
                    let (id, trade, kind) = (piece.id, *trade, piece.kind);
                    (Some(*site), Box::new(move || loot(id, trade, kind)))
                }
                Owner::Player => {
                    let slots = piece.kind.slots().unwrap_or(0);
                    (None, Box::new(move || Store::empty(slots)))
                }
            };
            let store = Store::of(&save.records, piece.id, piece.kind, fresh);
            let title = match &piece.owner {
                Owner::Town { house, .. } if !house.is_empty() => Localized::verbatim(tr_with(
                    "furniture.of_house",
                    &[("thing", &piece.kind.name()), ("house", house)],
                )),
                _ => piece.kind.name(),
            };
            info!(
                "FURNITURE open {} {} holding {} stacks and {} coins",
                piece.kind.code(),
                piece.id,
                store.slots.iter().flatten().count(),
                store.coins
            );
            view.open = Some(OpenStore {
                id: piece.id,
                title,
                site,
                store,
            });
            *screen = Screen::Pack;
        }
        Use::Surface => {}
    }
}

/// The townsfolk in their seats by day and beds by night (`furniture`
/// design 7), once a second: the core's rule over each standing town's
/// people and pieces, the player's place left alone.
pub fn seat_townsfolk(
    time: Res<Time>,
    mut since: Local<f32>,
    folk: Option<ResMut<Townsfolk>>,
    pieces: Res<Pieces>,
    mut seats: ResMut<Seats>,
    towns: Res<Towns>,
    sun: Option<Res<pbd_app::sky::Sun>>,
) {
    let Some(mut folk) = folk else {
        return;
    };
    *since += time.delta_secs();
    if *since < 1.0 {
        return;
    }
    *since = 0.0;
    let radius = f64::from(pbd_app::planet::terrain_config().radius_m);
    for standing in &towns.standing {
        let site = standing.site;
        let night = sun
            .as_deref()
            .is_some_and(|s| s.clock.daylight(standing.anchor.normalize_or(Vec3::Y)) < 0.12);
        let people: Vec<(Who, DVec3, Want, PersonId)> = folk
            .people(site)
            .map(|p| {
                let post = p.at.as_dvec3().normalize() * (radius + f64::from(p.altitude_m));
                let work = p.work();
                let w = want(
                    night,
                    keeps_watch(&p.role),
                    work == Some(Work::Sit),
                    work == Some(Work::Idle),
                    p.seed,
                );
                (
                    Who::Person {
                        site,
                        index: p.id.index,
                    },
                    post,
                    w,
                    p.id,
                )
            })
            .collect();
        let mine: Vec<&Piece> = pieces
            .all
            .iter()
            .filter(|p| matches!(&p.owner, Owner::Town { site: s, .. } if *s == site))
            .collect();
        let spots: Vec<Spot> = mine.iter().map(|p| p.spot()).collect();
        let reach = if night { BED_REACH_M } else { SEAT_REACH_M };
        let wanted: Vec<(Who, DVec3, Want)> = people.iter().map(|p| (p.0, p.1, p.2)).collect();
        let placed = assign(&wanted, &spots, &mut seats.0, reach);
        for (who, _, _, id) in &people {
            let Some((_, spot, place)) = placed.iter().find(|(w, _, _)| w == who) else {
                if folk.resting(*id).is_some() {
                    folk.release(*id);
                }
                continue;
            };
            let Some(piece) = mine.iter().find(|p| p.id == spot.id) else {
                continue;
            };
            let Some((local, _)) = piece.kind.place_at(*place) else {
                continue;
            };
            let feet = piece.point(Vec3::new(local.x, 0.0, local.z)).as_dvec3();
            let posture = match piece.kind.use_() {
                Use::Bed { top_m } => Posture::Lie { top_m },
                Use::Seat { seat_m, .. } => Posture::Sit { seat_m },
                _ => continue,
            };
            folk.rest(*id, feet, piece.front.as_dvec3(), posture);
        }
    }
}

/// Close the container with the pack.
pub fn follow_screen(screen: Res<Screen>, mut view: ResMut<StoreView>) {
    if *screen != Screen::Pack && view.open.is_some() {
        view.open = None;
    }
}

/// The container's panel, under the pack: its name, its squares in rows of
/// ten, and its coins.
pub fn spawn_panel(layer: &mut ChildSpawnerCommands, kit: &UiKit, atlas: &Handle<Image>) {
    let (rock, (w, _)) = kit.rock("pack");
    layer
        .spawn((
            rock,
            Node {
                width: px(w),
                padding: UiRect::all(px(super::pack::RIM)),
                flex_direction: FlexDirection::Column,
                display: Display::None,
                ..default()
            },
            StorePanel,
        ))
        .with_children(|panel| {
            panel.spawn((
                KitText::new(Localized::empty(), ui_kit::TITLE).shadow(ui_kit::INK),
                Node {
                    margin: UiRect::bottom(px(4)),
                    ..default()
                },
                StoreTitle,
            ));
            for row in 0..STORE_SQUARES / SLOTS {
                panel.spawn(Node::default()).with_children(|line| {
                    for k in row * SLOTS..(row + 1) * SLOTS {
                        slots::cell(
                            line,
                            kit,
                            STORE_AT + k,
                            atlas,
                            (Button, super::pack::PackCell(STORE_AT + k), StoreCell(k)),
                        );
                    }
                });
            }
            panel
                .spawn((
                    ui_kit::plaque(kit, pbd_app::ui_kit::Set::Stone, 0.0),
                    StoreCoins,
                ))
                .with_child(ui_kit::plaque_label(
                    pbd_app::ui_kit::Set::Stone,
                    Localized::empty(),
                ));
        });
}

/// A click while a container is open: a carried stack goes into it, one of
/// its stacks comes out (a right-click half of it), its coins to the purse.
/// Each move is one line with the slots after it. Taken before the pack's
/// own click, which it clears.
#[allow(clippy::too_many_arguments)]
pub fn press(
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    screen: Res<Screen>,
    cells: Query<(&Interaction, &super::pack::PackCell)>,
    coins: Query<&Interaction, (With<StoreCoins>, Changed<Interaction>)>,
    mut view: ResMut<StoreView>,
    mut hotbar: ResMut<Hotbar>,
    mut save: ResMut<WorldSave>,
    mut economy: Option<ResMut<pbd_app::economy::Economy>>,
    mut did: MessageWriter<pbd_app::missions::Did>,
    mut note: ResMut<FarmNote>,
    time: Res<Time>,
) {
    if *screen != Screen::Pack {
        return;
    }
    let Some(open) = view.open.as_mut() else {
        return;
    };
    let now = time.elapsed_secs();
    if coins.iter().any(|i| *i == Interaction::Pressed) && open.store.coins > 0 {
        let mut store = open.store.clone();
        let found = store.take_coins();
        let Some(economy) = economy.as_mut() else {
            return;
        };
        let mut purse = economy.purse.clone();
        purse
            .change(Reason::Found, found)
            .expect("coins found are never refused");
        // The container first: a torn write loses the coins, never pays them
        // twice.
        let lines = vec![
            store.record(open.id),
            StoredRecord::of(PURSE_KIND, PURSE_ID, PURSE_SCHEMA, &purse),
        ];
        if save.store(&Author::Player(0), lines).is_none() {
            error!("store move BLOCKED: the save did not accept it");
            return;
        }
        save.note_record_kinds(&[(STORE_KIND, STORE_SCHEMA), (PURSE_KIND, PURSE_SCHEMA)]);
        economy.purse = purse;
        open.store = store;
        if let Some(site) = open.site {
            did.write(pbd_app::missions::Did::new(
                MissionKind::Loot,
                format!("{site}:coins"),
                1,
            ));
        }
        info!("FURNITURE took {found} coins from {}", open.id);
        mouse.clear_just_pressed(MouseButton::Left);
        return;
    }
    let half = if mouse.just_pressed(MouseButton::Right) {
        true
    } else if mouse.just_pressed(MouseButton::Left) {
        false
    } else {
        return;
    };
    let Some(index) = cells
        .iter()
        .find(|(i, _)| **i == Interaction::Pressed)
        .or_else(|| cells.iter().find(|(i, _)| **i != Interaction::None))
        .map(|(_, c)| c.0)
    else {
        return;
    };
    let mut slots = hotbar.0.clone();
    let mut store = open.store.clone();
    let mut taken = None;
    let moved = if index >= STORE_AT {
        let k = index - STORE_AT;
        let item = store.slots.get(k).copied().flatten().map(|s| s.item);
        let n = store.take_into(k, &mut slots, half);
        taken = item.filter(|_| n > 0).map(|i| (i, n));
        if n == 0 && item.is_some() {
            note.say(now, tr("furniture.full"));
        }
        n
    } else if index < CARRIED {
        let n = store.put_from(&mut slots, index, half);
        if n == 0 && slots.get(index).is_some() {
            note.say(now, tr("furniture.full"));
        }
        n
    } else {
        0
    };
    mouse.clear_just_pressed(MouseButton::Left);
    mouse.clear_just_pressed(MouseButton::Right);
    if moved == 0 {
        return;
    }
    if !save.store_with_hands(store.record(open.id), STORE_SCHEMA, &slots) {
        error!("store move BLOCKED: the save did not accept it");
        return;
    }
    hotbar.0 = slots;
    open.store = store;
    if let (Some(site), Some((item, n))) = (open.site, taken) {
        let code = pbd_core::missions::item_code(item);
        did.write(pbd_app::missions::Did::new(
            MissionKind::Loot,
            format!("{site}:{code}"),
            u32::from(n),
        ));
    }
    info!("FURNITURE moved {moved} at square {index} of {}", open.id);
}

/// Show the panel with an open container, and paint its squares, its name
/// and its coins. After the pack's painters.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn paint(
    screen: Res<Screen>,
    view: Res<StoreView>,
    icons: Option<Res<ItemIcons>>,
    images: Res<Assets<Image>>,
    mut panels: Query<&mut Node, (With<StorePanel>, Without<StoreCell>)>,
    mut cells: Query<(&StoreCell, &mut Node), Without<StorePanel>>,
    mut icon_nodes: Query<(&slots::SlotIcon, &mut ImageNode)>,
    mut counts: Query<(&slots::SlotCount, &mut KitText), Without<StoreTitle>>,
    mut tips: Query<(&SlotCell, &mut ui_kit::Tooltip)>,
    mut title: Query<&mut KitText, (With<StoreTitle>, Without<slots::SlotCount>)>,
    mut coins: Query<(&mut Plaque, &Children), With<StoreCoins>>,
    mut labels: Query<
        &mut KitText,
        (
            Without<StoreTitle>,
            Without<slots::SlotCount>,
            With<ui_kit::PlaqueLabel>,
        ),
    >,
) {
    let open = view.open.as_ref().filter(|_| *screen == Screen::Pack);
    let want = if open.is_some() {
        Display::Flex
    } else {
        Display::None
    };
    for mut node in &mut panels {
        if node.display != want {
            node.display = want;
        }
    }
    let (Some(open), Some(icons)) = (open, icons) else {
        return;
    };
    let n = open.store.slots.len();
    for (cell, mut node) in &mut cells {
        let want = if cell.0 < n {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != want {
            node.display = want;
        }
    }
    let stack = |i: usize| open.store.slots.get(i - STORE_AT).copied().flatten();
    for (icon, mut node) in &mut icon_nodes {
        if icon.0 >= STORE_AT {
            slots::paint_icon(&mut node, stack(icon.0), &icons, &images);
        }
    }
    for (count, mut text) in &mut counts {
        if count.0 < STORE_AT {
            continue;
        }
        let label = match stack(count.0) {
            Some(s) if s.count > 1 => Localized::verbatim(s.count.to_string()),
            _ => Localized::empty(),
        };
        if text.text != label {
            text.text = label;
        }
    }
    for (cell, mut tip) in &mut tips {
        if cell.0 < STORE_AT {
            continue;
        }
        let want = slots::stack_tip(stack(cell.0), &icons);
        if tip.0 != want {
            tip.0 = want;
        }
    }
    for mut text in &mut title {
        if text.text != open.title {
            text.text = open.title.clone();
        }
    }
    for (mut plaque, children) in &mut coins {
        let none = open.store.coins == 0;
        if plaque.disabled != none {
            plaque.disabled = none;
        }
        for child in children.iter() {
            if let Ok(mut label) = labels.get_mut(child) {
                let want = Localized::verbatim(tr_with(
                    "furniture.coins",
                    &[("count", &open.store.coins.to_string())],
                ));
                if label.text != want {
                    label.text = want;
                }
            }
        }
    }
}

/// The furniture's systems.
pub fn plugin(app: &mut App) {
    app.init_resource::<Pieces>()
        .init_resource::<Seats>()
        .init_resource::<Sitting>()
        .init_resource::<StoreView>()
        .add_systems(
            Update,
            (
                gather,
                offer.in_set(pbd_app::interact::InteractSet::Offer),
                use_key.after(pbd_app::interact::InteractSet::Resolve),
                seat_townsfolk,
                follow_screen,
            )
                .chain(),
        )
        .add_systems(Update, scripted.after(gather));
}

/// Done once: the capture instrument below has acted.
#[derive(Resource)]
struct ShotDone;

/// `PBD_FURNITURE_SHOT=<room|seated|asleep|chest>[:<frame>]`: a capture
/// instrument (`furniture` task 3.8), never read in play. At its frame
/// (default 140), once a town's furniture stands, it puts the walker where a
/// player would be for the shot and does what F would: `room` stands at the
/// chair nearest a table and looks across it; `seated` sits on that chair;
/// `asleep` stands at the foot of a bed a townsperson lies in (or the nearest
/// bed) looking at the pillow; `chest` stands at the nearest town container
/// and opens it.
pub fn scripted(world: &mut World) {
    let Ok(plan) = std::env::var("PBD_FURNITURE_SHOT") else {
        return;
    };
    if world.contains_resource::<ShotDone>() {
        return;
    }
    let (mode, at) = match plan.split_once(':') {
        Some((m, f)) => (m.to_string(), f.parse().unwrap_or(140u32)),
        None => (plan.clone(), 140),
    };
    let frame = world
        .get_resource::<super::CaptureState>()
        .map_or(0, |c| c.frame);
    let pieces = world.resource::<Pieces>().all.clone();
    if frame < at || pieces.is_empty() {
        return;
    }
    let Some(position) = world
        .query_filtered::<&avian3d::prelude::Position, With<Walker>>()
        .iter(world)
        .next()
        .map(|p| p.0)
    else {
        return;
    };
    let o = pbd_app::sim_origin::origin_of(world);
    let me = o.query_point(position);
    let town: Vec<Piece> = pieces
        .into_iter()
        .filter(|p| matches!(p.owner, Owner::Town { .. }))
        .collect();
    let nearest = |pick: &dyn Fn(&Piece) -> bool| {
        town.iter()
            .filter(|p| pick(p))
            .min_by(|a, b| a.at.distance(me).total_cmp(&b.at.distance(me)))
            .cloned()
    };
    let has_table = |p: &Piece| {
        town.iter()
            .any(|t| t.kind == Kind::Table && t.at.distance(p.at) < 1.6)
    };
    let seats = &world.resource::<Seats>().0;
    let target = match mode.as_str() {
        "room" | "seated" => nearest(&|p| p.kind == Kind::Chair && has_table(p)),
        "asleep" => nearest(&|p| {
            p.kind == Kind::Bed && matches!(seats.holder((p.id, 0)), Some(Who::Person { .. }))
        })
        .or_else(|| nearest(&|p| p.kind == Kind::Bed)),
        "chest" => nearest(&|p| matches!(p.kind, Kind::Chest | Kind::BandedChest)),
        _ => None,
    };
    let Some(piece) = target else {
        return;
    };
    world.insert_resource(ShotDone);
    let (eye, look_at) = match mode.as_str() {
        "asleep" => (
            piece.point(Vec3::new(0.35, 1.6, 1.45)),
            piece.point(Vec3::new(0.0, 0.5, -0.55)),
        ),
        "chest" => (
            piece.point(Vec3::new(0.2, 1.6, 1.1)),
            piece.point(Vec3::new(0.0, 0.3, 0.0)),
        ),
        _ => (
            piece.point(Vec3::new(0.0, 1.6, -0.15)),
            piece.point(Vec3::new(0.0, 0.5, 1.6)),
        ),
    };
    let up = eye.normalize_or(Vec3::Y);
    let local = o.to_local(pbd_core::world_frame::WorldPos::new(
        o.frame,
        eye.as_dvec3(),
    ));
    let view = Transform::from_translation(local)
        .looking_to((look_at - eye).normalize_or(Vec3::X), up)
        .rotation;
    pbd_app::walking::drop_walker(world, local, Vec3::ZERO, view);
    info!(
        "FURNITURE shot {mode}: {} {} at {:.1} m",
        piece.kind.code(),
        piece.id,
        piece.at.distance(me)
    );
    match mode.as_str() {
        "seated" => {
            let _ = world
                .resource_mut::<Seats>()
                .0
                .claim((piece.id, 0), Who::Player);
            if let Some((seat_eye, facing)) = eye_of(&piece, 0) {
                world.resource_mut::<Sitting>().0 = Some(((piece.id, 0), piece.kind));
                world.resource_mut::<SeatedEye>().0 = Some(seat_eye.as_dvec3());
                world.resource_mut::<WalkingState>().look_toward(facing);
            }
        }
        "chest" => {
            let Owner::Town { site, trade, house } = piece.owner.clone() else {
                return;
            };
            let store = Store::of(
                &world.resource::<WorldSave>().records,
                piece.id,
                piece.kind,
                || loot(piece.id, trade, piece.kind),
            );
            let title = Localized::verbatim(tr_with(
                "furniture.of_house",
                &[("thing", &piece.kind.name()), ("house", &house)],
            ));
            world.resource_mut::<StoreView>().open = Some(OpenStore {
                id: piece.id,
                title,
                site: Some(site),
                store,
            });
            *world.resource_mut::<Screen>() = Screen::Pack;
        }
        _ => {}
    }
}
