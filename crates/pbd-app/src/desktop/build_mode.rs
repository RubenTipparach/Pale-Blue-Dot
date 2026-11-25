//! Build mode (`player-building` design B5, B10): B on foot swaps the hotbar for
//! the kit's palette, the aim snaps the chosen piece to the cut, the ghost
//! shows it green or red with the rule it breaks, a left click places it and
//! a right click takes the aimed piece away. K opens the kit picker, where
//! the wheel steps the kit.
//!
//! The Door slot places a doorway, or, aimed at a doorway with no door, hangs
//! a door from the pack in it (design B8). Z raises architecture mode's
//! camera over the plot (design B9, `pbd_app::build_view`): the cursor is
//! the aim, the move keys pan, a right drag orbits, ctrl and the wheel zoom,
//! Page Up and Page Down move the build layer a storey.
//!
//! The rules are the core's (`pbd_core::building::rules`), the pieces are
//! kept by `pbd_app::builds`, and nothing here writes the save but through
//! [`Builds::place`] and [`Builds::remove`].

use super::digging::{material_of, record_of};
use bevy::input::mouse::{AccumulatedMouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use pbd_app::build_view::BuildView;
use pbd_app::builds::{Builds, PATCH_REACH_M, patch_round};
use pbd_app::controls::MenuOpen;
use pbd_app::planet::{PLANET_RADIUS, PlanetContact, PlanetFine, PlanetRenderFrame};
use pbd_app::saves::WorldSave;
use pbd_app::towns::{TownAssets, TownRoot};
use pbd_app::ui_kit::{self, KitText, Set, UiKit};
use pbd_core::building::blueprint::{blueprint, fit_roof, plain, room_at};
use pbd_core::building::cut::cut;
use pbd_core::building::rules::{Rule, dependant, placing_order};
use pbd_core::building::{Cells, Opening, Piece, STOREY, Shape, Site};
use pbd_core::column;
use pbd_core::locale::{tr, tr_with};
use pbd_core::settlement::pieces::MeshBuf;
use pbd_core::settlement::{BuildingDef, Kits};
use pbd_core::terrain::Material;
use std::collections::HashMap;

/// What a palette slot holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Wall,
    Door,
    Window,
    Floor,
    Roof,
    Stairs,
    Post,
    Lantern,
    Blueprint,
    Remove,
    /// A piece of furniture carried in the pack (`furniture` design 5): the
    /// eleventh slot, on the minus key.
    Furniture,
}

/// The palette, in M6's order, and the furniture after it.
pub const PALETTE: [Tool; 11] = [
    Tool::Wall,
    Tool::Door,
    Tool::Window,
    Tool::Floor,
    Tool::Roof,
    Tool::Stairs,
    Tool::Post,
    Tool::Lantern,
    Tool::Blueprint,
    Tool::Remove,
    Tool::Furniture,
];

impl Tool {
    /// Its id: its icon's name and its label's key, `build.tool.<id>`.
    pub fn id(self) -> &'static str {
        match self {
            Tool::Wall => "wall",
            Tool::Door => "door",
            Tool::Window => "window",
            Tool::Floor => "floor",
            Tool::Roof => "roof",
            Tool::Stairs => "stairs",
            Tool::Post => "post",
            Tool::Lantern => "lantern",
            Tool::Blueprint => "blueprint",
            Tool::Remove => "remove",
            Tool::Furniture => "furniture",
        }
    }
}

/// How far the build aim reaches, metres.
pub const REACH_M: f32 = 7.0;
/// How far a right press may drag, pixels, and still be a click that takes
/// a piece away rather than an orbit (architecture mode).
pub const CLICK_SLOP_PX: f32 = 6.0;
/// How fast the move keys pan architecture mode's camera, metres a second
/// at its starting distance (scaled with the distance).
pub const PAN_M_S: f32 = 12.0;
/// A pixel of right drag turns the camera this many radians.
pub const ORBIT_RAD_PX: f32 = 0.006;
/// The aim's step, metres.
const STEP_M: f32 = 0.08;

/// Build mode's state.
#[derive(Resource, Default, Debug)]
pub struct BuildMode {
    pub on: bool,
    /// The palette slot chosen.
    pub slot: usize,
    /// The kit, by its place in the kits.
    pub kit: usize,
    /// A stair's or a blueprint's turn, in sixths.
    pub turn: usize,
    /// Which blueprint.
    pub choice: usize,
    /// Which of the furniture carried, by its place among the kinds carried.
    pub piece: usize,
    /// Whether the kit picker is open.
    pub picker: bool,
    /// The hotbar's selection when build mode opened, given back on leaving.
    selected: usize,
    /// This frame's clicks, taken from the world's readers while building.
    place: bool,
    take: bool,
    /// Architecture mode was asked for or left this frame (Z).
    pub view_toggle: bool,
    /// This frame's zoom, in wheel steps (ctrl and the wheel).
    pub zoom: i32,
    /// How far the right button has dragged since it went down, pixels.
    drag_px: f32,
    /// Architecture mode's left presses (design B11): a single click places
    /// once the double-click wait has passed; a double-click walks there.
    clicks: pbd_app::build_view::Clicks,
    /// A double-click asked to walk to where the cursor stands
    /// ([`BuildMode::stand`]) this frame.
    pub walk: bool,
    /// Where the cursor's ray came down on a top a walker can stand on, in
    /// architecture mode (design B11); none on a wall's side or in open air.
    pub stand: Option<Vec3>,
    /// Where the aim last met something, planet-local: where architecture
    /// mode starts over.
    pub aim: Option<Vec3>,
    /// No ghost is aimed or drawn: set only by the capture instrument
    /// (`PBD_BUILD_SHOT`'s `quiet`), for a shot of what is built.
    pub quiet: bool,
    /// What the ghost shows: its pieces, whether they may go, and the piece
    /// a removal aims at.
    pub ghost: Option<GhostShown>,
    /// O was pressed: open the planner on what is aimed at
    /// (`building-planner` design 3, read by `construction`).
    pub plan_asked: bool,
    /// The rule a red ghost breaks, or the dependant a removal is refused
    /// for, as a locale key: for the HUD line and the log.
    pub reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GhostShown {
    pub pieces: Vec<Piece>,
    pub ok: bool,
    pub remove: Option<u64>,
}

/// The ghost's mesh: one green, one red.
#[derive(Component)]
pub struct Ghost(pub bool);

/// The palette row, each of its slots, and the kit picker's panel.
#[derive(Component)]
pub struct PaletteRow;
#[derive(Component)]
pub struct PaletteCell(pub usize);
#[derive(Component)]
pub struct KitPicker;
#[derive(Component)]
pub struct KitCard(pub usize);

/// Every plain building of the templates, for the blueprint slot.
#[derive(Resource, Default)]
pub struct Blueprints(pub Vec<BuildingDef>);

/// The digit keys, in the palette's order.
const DIGITS: [KeyCode; 10] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
    KeyCode::Digit0,
];

/// B on foot opens and closes build mode; while it is open the clicks, the
/// number row and K are build mode's, taken from every other reader before
/// `Update` (so a click does not also dig, cast or eat).
#[allow(clippy::too_many_arguments)]
pub fn take_input(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut buttons: ResMut<ButtonInput<MouseButton>>,
    mut wheel: MessageReader<MouseWheel>,
    (motion, view): (Option<Res<AccumulatedMouseMotion>>, Option<Res<BuildView>>),
    menu: Option<Res<MenuOpen>>,
    walking: Option<Res<pbd_app::walking::WalkingReadout>>,
    aboard: Option<Res<pbd_app::vehicles::Aboard>>,
    mut hotbar: ResMut<super::slots::Hotbar>,
    mut mode: ResMut<BuildMode>,
    kits: Option<Res<TownAssets>>,
    (choices, data): (
        Option<Res<super::construction::PlanChoices>>,
        Option<Res<pbd_app::construction::PlannerData>>,
    ),
    (time, windows): (Res<Time>, Query<&Window, With<PrimaryWindow>>),
) {
    mode.place = false;
    mode.take = false;
    mode.walk = false;
    mode.view_toggle = false;
    mode.zoom = 0;
    let viewing = view.as_deref().is_some_and(|v| v.on);
    let steps: i32 = wheel
        .read()
        .map(|m| (m.y < 0.0) as i32 - (m.y > 0.0) as i32)
        .sum();
    let on_foot = walking.is_some_and(|w| w.active) && aboard.is_none_or(|a| a.0.is_none());
    if menu.is_some_and(|m| m.0) {
        return;
    }
    if keys.just_pressed(KeyCode::KeyB) && (on_foot || mode.on) {
        mode.on = !mode.on;
        mode.picker = false;
        if mode.on {
            mode.selected = hotbar.0.selected();
            info!("BUILD on");
        } else {
            hotbar.0.select(mode.selected);
            mode.ghost = None;
            mode.reason = None;
            info!("BUILD off");
        }
    }
    if mode.on && !on_foot {
        mode.on = false;
        hotbar.0.select(mode.selected);
        mode.ghost = None;
    }
    if !mode.on {
        return;
    }
    for (index, key) in DIGITS.iter().enumerate() {
        if keys.just_pressed(*key) {
            mode.slot = index;
            mode.turn = 0;
        }
        keys.reset(*key);
    }
    // The furniture, past the digits (`furniture` design 5).
    if keys.just_pressed(KeyCode::Minus) {
        mode.slot = DIGITS.len();
    }
    keys.reset(KeyCode::Minus);
    if keys.just_pressed(KeyCode::KeyK) {
        mode.picker = !mode.picker;
    }
    // O: the planner (`building-planner` design 3).
    if keys.just_pressed(KeyCode::KeyO) {
        mode.plan_asked = true;
    }
    keys.reset(KeyCode::KeyO);
    keys.reset(KeyCode::KeyK);
    keys.reset(KeyCode::KeyG);
    // Z: architecture mode (design B9), read by `drive_view`.
    mode.view_toggle = keys.just_pressed(KeyCode::KeyZ);
    keys.reset(KeyCode::KeyZ);
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if steps != 0 && viewing && ctrl {
        mode.zoom = -steps;
    } else if steps != 0 && mode.picker {
        let n = kits.as_ref().map_or(1, |k| k.kits.kits.len().max(1)) as i32;
        mode.kit = (mode.kit as i32 + steps).rem_euclid(n) as usize;
    } else if steps != 0 {
        let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        if PALETTE[mode.slot] == Tool::Blueprint && !shift {
            // The wheel steps the plans; shift and the wheel turn the ghost
            // (`building-planner` design 5).
            let n = choices.as_ref().map_or(1, |c| c.0.len().max(1)) as i32;
            mode.choice = (mode.choice as i32 + steps).rem_euclid(n) as usize;
        } else if PALETTE[mode.slot] == Tool::Blueprint {
            let step = data.as_ref().map_or(1, |d| i32::from(d.tun.rotation_step));
            mode.turn = (mode.turn as i32 + steps * step).rem_euclid(6) as usize;
        } else if PALETTE[mode.slot] == Tool::Furniture {
            // The wheel steps through the furniture carried.
            let n = carried_furniture(&hotbar.0).len().max(1) as i32;
            mode.piece = (mode.piece as i32 + steps).rem_euclid(n) as usize;
        } else {
            mode.turn = (mode.turn as i32 + steps).rem_euclid(6) as usize;
        }
    }
    if let Some(kits) = kits {
        mode.kit = mode.kit.min(kits.kits.kits.len().saturating_sub(1));
    }
    if !viewing {
        mode.clicks.clear();
    }
    if !mode.picker {
        mode.place = buttons.just_pressed(MouseButton::Left);
        if viewing {
            // A single click places once the double-click wait has passed;
            // a double-click walks there (design B11).
            use pbd_app::build_view::Click;
            let now = time.elapsed_secs_f64();
            let mut click = mode.clicks.wait(now);
            if buttons.just_pressed(MouseButton::Left) {
                let at = windows
                    .single()
                    .ok()
                    .and_then(Window::cursor_position)
                    .unwrap_or_default();
                match mode.clicks.press(now, at) {
                    Click::None => {}
                    k => click = k,
                }
            }
            mode.place = click == Click::Single;
            mode.walk = click == Click::Double;
            // A right press that does not drag is a click that takes a
            // piece away; one that drags orbits the camera.
            if buttons.just_pressed(MouseButton::Right) {
                mode.drag_px = 0.0;
            }
            if buttons.pressed(MouseButton::Right) {
                mode.drag_px += motion.as_deref().map_or(0.0, |m| m.delta.length());
            }
            mode.take = buttons.just_released(MouseButton::Right) && mode.drag_px <= CLICK_SLOP_PX;
        } else {
            mode.take = buttons.just_pressed(MouseButton::Right);
        }
        buttons.reset(MouseButton::Left);
        buttons.reset(MouseButton::Right);
    }
}

/// The furniture carried, each kind once, in `Kind::ALL` order, with how
/// many of it.
pub fn carried_furniture(
    slots: &pbd_core::inventory::Slots,
) -> Vec<(pbd_core::furniture::Kind, u32)> {
    pbd_core::furniture::Kind::ALL
        .into_iter()
        .map(|k| (k, slots.count(pbd_core::inventory::Item::Good(k.good()))))
        .filter(|(_, n)| *n > 0)
        .collect()
}

/// Architecture mode's camera (design B9): Z in build mode raises it over
/// where the aim was (or ahead of the eye), and lowers it again; leaving
/// build mode leaves it. While it is on the move keys pan it, a right drag
/// orbits it, ctrl and the wheel zoom, and Page Up and Page Down move the
/// build layer a storey. The camera eases toward its pose every frame.
#[allow(clippy::too_many_arguments)]
pub fn drive_view(
    mode: Res<BuildMode>,
    mut view: ResMut<BuildView>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Option<Res<AccumulatedMouseMotion>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    frame: Option<Res<PlanetRenderFrame>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    if view.on && !mode.on {
        view.leave();
        info!("BUILD view off");
        return;
    }
    if mode.view_toggle && mode.on {
        if view.on {
            view.leave();
            info!("BUILD view off");
            return;
        }
        let Some((transform, _)) = cameras.iter().find(|(_, c)| c.is_active) else {
            return;
        };
        let eye = pbd_app::render_origin::local_of(frame.as_deref(), transform.translation());
        let look = transform.forward().as_vec3();
        let site = mode.aim.unwrap_or(eye + look * 8.0);
        // The build layer: the line the site stands on.
        let layer = column::layer_at(site.length() - PLANET_RADIUS + 0.02).unwrap_or(0) as u16;
        view.enter(
            site.as_dvec3(),
            layer,
            look.as_dvec3(),
            eye.as_dvec3(),
            (eye + look * 8.0).as_dvec3(),
        );
        info!("BUILD view on, layer {layer}");
    }
    if !view.on {
        return;
    }
    let dt = time.delta_secs().min(0.1);
    let axis = |plus: KeyCode, minus: KeyCode| {
        f32::from(u8::from(keys.pressed(plus))) - f32::from(u8::from(keys.pressed(minus)))
    };
    let speed = PAN_M_S * view.distance / pbd_app::build_view::START_DISTANCE * dt;
    let (right, forward) = (
        axis(KeyCode::KeyD, KeyCode::KeyA),
        axis(KeyCode::KeyW, KeyCode::KeyS),
    );
    if right != 0.0 || forward != 0.0 {
        view.pan(right * speed, forward * speed);
    }
    if buttons.pressed(MouseButton::Right) {
        let d = motion.as_deref().map_or(Vec2::ZERO, |m| m.delta);
        if d != Vec2::ZERO && d.is_finite() {
            view.orbit(d.x * ORBIT_RAD_PX, d.y * ORBIT_RAD_PX);
        }
    }
    if mode.zoom != 0 {
        view.zoom(mode.zoom);
    }
    if keys.just_pressed(KeyCode::PageUp) {
        view.raise(i32::from(STOREY));
    }
    if keys.just_pressed(KeyCode::PageDown) {
        view.raise(-i32::from(STOREY));
    }
    // End: the cutaway over the build layer (design B11).
    if keys.just_pressed(KeyCode::End) {
        view.cut = !view.cut;
        info!("BUILD cutaway {}", if view.cut { "on" } else { "off" });
    }
    // A double-click on a top walks there (design B11).
    if mode.walk
        && let Some(feet) = mode.stand
    {
        let look = view.heading.as_vec3();
        info!("BUILD walk there");
        commands.queue(move |world: &mut World| {
            pbd_app::build_view::walk_there(world, feet, look);
        });
        return;
    }
    view.step(dt);
}

/// The ground's first air layer in a cell, cached for a fine set and an edit
/// revision.
#[derive(Default)]
pub struct GroundCache {
    key: (u64, u64),
    tops: HashMap<u32, Option<u16>>,
}

pub(super) fn ground_top(
    fine: &PlanetFine,
    save: &WorldSave,
    cache: &mut GroundCache,
    cell: u32,
) -> Option<u16> {
    let key = (fine.version, save.edits.revision());
    if cache.key != key {
        cache.key = key;
        cache.tops.clear();
    }
    *cache.tops.entry(cell).or_insert_with(|| {
        // Off the fine set, the generated ground.
        let Some(record) = record_of(fine, cell) else {
            let d = pbd_core::cell_key::key_direction(cell)?;
            let surface = pbd_app::planet::surface_height(d);
            return column::layer_at(surface - 0.5).map(|l| (l + 1) as u16);
        };
        let surface = fine.set.finest_records()[record].direction_height[3];
        let start = column::layer_at(surface + 12.0)?;
        // A column the tier holds, or would adopt, answers with its edits;
        // past it, the ground the heightfield draws.
        if material_of(fine, save, (cell, start)).is_none() {
            return column::layer_at(surface - 0.5).map(|l| (l + 1) as u16);
        }
        (start.saturating_sub(40)..=start)
            .rev()
            .find(|&l| {
                material_of(fine, save, (cell, l))
                    .is_some_and(|m| m != Material::Air && !m.is_water())
            })
            .map(|l| (l + 1) as u16)
    })
}

/// Where the eye ray meets the ground or a building: the last point before
/// it, and whether it came down onto a top.
struct Hit {
    point: Vec3,
    from_above: bool,
}

fn march(
    fine: &PlanetFine,
    contact: &PlanetContact,
    builds: &pbd_app::walking::PlayerStructures,
    eye: Vec3,
    look: Vec3,
    reach: f32,
    cut: Option<pbd_app::build_view::Cutaway>,
) -> Option<Hit> {
    let solid = |p: Vec3| -> bool {
        super::digging::sample_at(fine, contact, p).is_some_and(|s| s.solid)
            || !cut.is_some_and(|c| c.hides(p))
                && builds.0.iter().any(|b| {
                    let q = b.frame.local(p);
                    b.holds_in(q, 0.0, 0.02) || {
                        let (floor, _) = b.stand_in(q, 0.0);
                        floor.is_some_and(|f| f > q.y - 0.12 && f <= q.y + 0.0)
                    }
                })
    };
    let mut travelled = 0.0;
    let mut last = eye;
    while travelled <= reach {
        let p = eye + look * travelled;
        if solid(p) {
            let up = last.normalize_or(Vec3::Y);
            return Some(Hit {
                point: last,
                from_above: solid(last - up * 0.15),
            });
        }
        last = p;
        travelled += STEP_M;
    }
    None
}

/// The cell side nearest a planet point, and the corner nearest it.
fn nearest_side(cells: &Cells, cell: u32, point: Vec3) -> Option<(usize, usize)> {
    let c = cells.cell(cell)?;
    let d = point.normalize_or(Vec3::Y);
    let n = c.corners.len();
    let side = (0..n).max_by(|&a, &b| {
        let mid = |s: usize| ((c.corners[s] + c.corners[(s + 1) % n]) * 0.5).normalize();
        mid(a).dot(d).total_cmp(&mid(b).dot(d))
    })?;
    let corner = (0..n).max_by(|&a, &b| c.corners[a].dot(d).total_cmp(&c.corners[b].dot(d)))?;
    Some((side, corner))
}

/// A point to aim a removal at for a piece: the middle of what it is.
pub(super) fn reference(piece: &Piece) -> Option<Vec3> {
    let cells = piece.shape.cells();
    let mean = cells
        .iter()
        .filter_map(|&c| pbd_core::cell_key::key_direction(c))
        .fold(Vec3::ZERO, |s, d| s + d)
        .normalize_or_zero();
    if mean == Vec3::ZERO {
        return None;
    }
    let lift = match piece.shape {
        Shape::Wall { .. } | Shape::Post { .. } | Shape::Stair { .. } => 1.5,
        Shape::Door { .. } => 1.1,
        Shape::Lantern { .. } => 1.0,
        Shape::Roof { .. } => 1.0,
        Shape::Floor { .. } => 0.0,
        Shape::Furniture { .. } => 0.5,
    };
    let alt = column::layer_altitude(piece.shape.layer() as usize) + lift;
    Some(mean * (PLANET_RADIUS + alt))
}

/// Aim, check, show and build: one frame of build mode.
#[allow(clippy::too_many_arguments)]
pub fn aim_and_build(
    mut mode: ResMut<BuildMode>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    frame: Option<Res<PlanetRenderFrame>>,
    (fine, contact): (Res<PlanetFine>, Res<PlanetContact>),
    mut save: ResMut<WorldSave>,
    mut builds: ResMut<Builds>,
    structures: Res<pbd_app::walking::PlayerStructures>,
    assets: Res<TownAssets>,
    mut note: ResMut<super::farm::FarmNote>,
    time: Res<Time>,
    (mut hotbar, view, windows): (
        ResMut<super::slots::Hotbar>,
        Res<BuildView>,
        Query<&Window, With<PrimaryWindow>>,
    ),
    mut cache: Local<GroundCache>,
    mut patch_cache: Local<Option<(u32, std::sync::Arc<pbd_core::settlement::chart::Patch>)>>,
    (mut did, choices, data, mut in_hand): (
        MessageWriter<pbd_app::missions::Did>,
        Res<super::construction::PlanChoices>,
        Res<pbd_app::construction::PlannerData>,
        ResMut<super::construction::PlanInHand>,
    ),
) {
    if !mode.on || mode.picker || mode.quiet {
        if mode.ghost.is_some() {
            mode.ghost = None;
        }
        return;
    }
    let Some((transform, camera)) = cameras.iter().find(|(_, c)| c.is_active) else {
        return;
    };
    // The aim: the eye's look on foot; in architecture mode the cursor's
    // ray, no further than the build layer (design B9).
    let (eye, look, plane) = if view.on {
        let cursor = windows.single().ok().and_then(Window::cursor_position);
        let Some(ray) = cursor.and_then(|c| camera.viewport_to_world(transform, c).ok()) else {
            mode.ghost = None;
            return;
        };
        let eye = pbd_app::render_origin::local_of(frame.as_deref(), ray.origin);
        let look = ray.direction.as_vec3();
        (eye, look, Some(view.radius() as f32))
    } else {
        let eye = pbd_app::render_origin::local_of(frame.as_deref(), transform.translation());
        (eye, transform.forward().as_vec3(), None)
    };
    let reach = match plane {
        // Where the ray meets the build layer's sphere, from above.
        Some(radius) => {
            let b = eye.dot(look);
            let c = eye.length_squared() - radius * radius;
            let disc = b * b - c;
            (disc >= 0.0)
                .then(|| -b - disc.sqrt())
                .filter(|t| *t > 0.0)
                .unwrap_or(view.distance + 40.0)
                .min(view.distance + 40.0)
        }
        None => REACH_M,
    };
    let cut = view.cutaway();
    let marched = march(&fine, &contact, &structures, eye, look, reach, cut);
    // Where a double-click would stand the walker (design B11): a top the
    // ray came down on, never a wall's side or the open air.
    mode.stand = marched
        .as_ref()
        .filter(|h| view.on && h.from_above)
        .map(|h| h.point);
    let hit = marched.or_else(|| {
        // In open air the cursor lands on the build layer.
        plane.map(|_| Hit {
            point: eye + look * (reach - 0.01),
            from_above: true,
        })
    });
    let Some(hit) = hit else {
        mode.ghost = None;
        mode.reason = None;
        return;
    };
    mode.aim = Some(hit.point);
    let Some(cell) = contact.finest_key(hit.point.normalize_or(Vec3::Y)) else {
        mode.ghost = None;
        return;
    };
    // The cells round the aim, kept while the aim stays in one cell.
    let patch = match patch_cache.as_ref() {
        Some((at, p)) if *at == cell => p.clone(),
        _ => {
            let p = std::sync::Arc::new(patch_round(hit.point.normalize(), PATCH_REACH_M));
            *patch_cache = Some((cell, p.clone()));
            p
        }
    };
    let cells = Cells::new(&patch);
    let mut ground = |c: u32| ground_top(&fine, &save, &mut cache, c);
    // The ground is read once a cell, before the rules borrow it.
    let mut tops: HashMap<u32, Option<u16>> = HashMap::new();
    for (i, &k) in patch.keys.iter().enumerate() {
        if patch.cells[i].direction.dot(hit.point.normalize()) > (12.0 / PLANET_RADIUS).cos() {
            tops.insert(k, ground(k));
        }
    }
    let ground_of = |c: u32| tops.get(&c).copied().flatten();
    let site = Site {
        cells: &cells,
        ground: &ground_of,
    };
    let near = builds.near(hit.point, PATCH_REACH_M);
    let kits: &Kits = &assets.kits;
    let kit = &kits.kits[mode.kit.min(kits.kits.len() - 1)];
    // The storey: a top met from above is its own line; a face met from the
    // side is the storey whose floor or ground is under the aim.
    let alt = hit.point.length() - PLANET_RADIUS;
    let at_layer = column::layer_at(alt + 0.02).unwrap_or(0) as u16;
    let storey = if hit.from_above {
        at_layer
    } else {
        let floors = near.iter().filter_map(|p| match p.shape {
            Shape::Floor { cell: c, layer } if c == cell && layer <= at_layer => Some(layer),
            _ => None,
        });
        floors
            .chain(ground_of(cell).filter(|&g| g <= at_layer))
            .max()
            .unwrap_or(at_layer)
    };
    let tool = PALETTE[mode.slot];
    let pick = nearest_side(&cells, cell, hit.point);
    let neighbours = cells.neighbours(cell);
    let k = kit.name.as_str();
    let wall = |opening: Opening| -> Option<Vec<Piece>> {
        let (side, _) = pick?;
        let other = (*neighbours.get(side)?)?;
        Some(vec![Piece::new(
            k,
            Shape::wall(cell, other, storey, opening),
        )])
    };
    let mut remove = None;
    let planned = save.identity.building() == pbd_core::building::site::BuildingSetting::Planned;
    let mut setting_out = None;
    let candidate: Option<Vec<Piece>> = match tool {
        Tool::Wall => wall(Opening::None),
        // A door from the pack, aimed at a doorway with none (design B8);
        // anywhere else the doorway.
        Tool::Door => wall(Opening::Door).map(|pieces| {
            let Shape::Wall { a, b, layer, .. } = pieces[0].shape else {
                return pieces;
            };
            let doorway = near.iter().any(|p| {
                matches!(p.shape, Shape::Wall { a: a2, b: b2, layer: l, opening: Opening::Door }
                    if a2 == a && b2 == b && l == layer)
            });
            let door = Shape::door(a, b, layer);
            if doorway && !near.iter().any(|p| p.shape == door) {
                vec![Piece::new(k, door)]
            } else {
                pieces
            }
        }),
        Tool::Window => wall(Opening::Window),
        Tool::Floor => Some(vec![Piece::new(
            k,
            Shape::Floor {
                cell,
                layer: storey,
            },
        )]),
        Tool::Lantern => Some(vec![Piece::new(
            k,
            Shape::Lantern {
                cell,
                layer: storey,
            },
        )]),
        Tool::Post => pick
            .and_then(|(_, corner)| cells.corner_cells(cell, corner))
            .map(|three| vec![Piece::new(k, Shape::post(three, storey))]),
        Tool::Stairs => {
            // Up the way the eye looks, turned by the wheel.
            let ahead = nearest_side(&cells, cell, hit.point + look * 3.0).map(|(s, _)| s);
            ahead
                .and_then(|s| {
                    neighbours
                        .get((s + mode.turn) % neighbours.len().max(1))
                        .copied()
                        .flatten()
                })
                .map(|head| {
                    vec![Piece::new(
                        k,
                        Shape::Stair {
                            foot: cell,
                            head,
                            layer: storey,
                        },
                    )]
                })
        }
        Tool::Roof => match room_at(&site, &near, cell, storey) {
            Some(room) => Some(vec![Piece::new(k, fit_roof(kit, room, storey + STOREY))]),
            None => Some(vec![Piece::new(
                k,
                Shape::roof(
                    vec![cell],
                    storey + STOREY,
                    pbd_core::building::RoofShape::Cone,
                ),
            )]),
        },
        Tool::Blueprint => {
            // A plan: the planner's, a preset or the player's own
            // (`building-planner` design 5), charted on the ground here.
            let hand = choices.0.get(mode.choice % choices.0.len().max(1)).cloned();
            hand.and_then(|hand| {
                let (site_patch, _, base, charted) = super::construction::chart_here(
                    &fine, &save, &mut cache, &hand.plan, cell, mode.turn,
                )?;
                let pieces: Vec<Piece> = charted
                    .iter()
                    .map(|(_, p)| p.clone())
                    .filter(|p| planned || !matches!(p.shape, Shape::Furniture { .. }))
                    .collect();
                setting_out = Some((site_patch, base, charted, hand));
                Some(pieces)
            })
        }
        Tool::Furniture => {
            let carried = carried_furniture(&hotbar.0);
            match carried.get(mode.piece % carried.len().max(1)) {
                Some(&(kind, count)) => {
                    let side = if kind == pbd_core::furniture::Kind::Table {
                        pbd_core::furniture::MIDDLE
                    } else {
                        pick.map_or(0, |(s, _)| s as u8)
                    };
                    note.say_for(
                        time.elapsed_secs(),
                        tr_with(
                            "build.furniture_held",
                            &[("piece", &kind.name()), ("count", &count.to_string())],
                        ),
                        0.25,
                    );
                    Some(vec![Piece::new(
                        k,
                        Shape::Furniture {
                            cell,
                            layer: storey,
                            side,
                            kind,
                        },
                    )])
                }
                None => {
                    note.say_for(time.elapsed_secs(), tr("build.no_furniture"), 0.25);
                    None
                }
            }
        }
        Tool::Remove => {
            let aimed = near
                .iter()
                .filter_map(|p| Some((reference(p)?.distance(hit.point), p)))
                .filter(|(d, _)| *d < 2.0)
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(_, p)| p.clone());
            remove = aimed.as_ref().map(Piece::id);
            aimed.map(|p| vec![p])
        }
    };
    let Some(candidate) = candidate else {
        mode.ghost = None;
        mode.reason = None;
        return;
    };
    // Checked: a removal against what rests on it, a placement piece by
    // piece against what stands and what it brings.
    let (ok, reason) = if let Some(id) = remove {
        match near
            .iter()
            .find(|p| p.id() == id)
            .and_then(|p| dependant(&near, p))
        {
            Some(d) => (false, Some(format!("build.piece.{}", d.shape.class().id()))),
            // A container with anything in it is emptied first.
            None if !container_empty(&save, &near, id) => {
                (false, Some("build.not_empty".to_string()))
            }
            None => (true, None),
        }
    } else if let Some((site_patch, base, charted, hand)) = &setting_out {
        // A whole plan, checked against the world (`building-planner` 5).
        match super::construction::check_here(
            &fine, &save, &mut cache, site_patch, charted, *base, &near, &data, &hand.plan,
        ) {
            Ok(()) => (true, None),
            Err((why, _)) => (false, Some(why.key())),
        }
    } else if planned && !matches!(tool, Tool::Furniture) {
        // A planned world builds from plans (`building-planner` 11).
        (false, Some("site.planned_only".to_string()))
    } else {
        let broke: Option<Rule> = placing_order(&site, &near, &candidate).err();
        (
            broke.is_none(),
            broke.map(|r| format!("build.refused.{}", r.id())),
        )
    };
    let shown = GhostShown {
        pieces: candidate.clone(),
        ok,
        remove,
    };
    if mode.ghost.as_ref() != Some(&shown) {
        mode.ghost = Some(shown);
    }
    if mode.reason != reason {
        if let Some(r) = &reason {
            debug!("BUILD red: {r}");
        }
        mode.reason = reason.clone();
    }
    let now = time.elapsed_secs();
    if let Some(r) = &reason {
        let text = if remove.is_some() && r.starts_with("build.piece.") {
            tr_with("build.needed_by", &[("thing", &tr(r))])
        } else {
            tr(r)
        };
        note.say_for(now, text, 0.25);
    }
    let is_door = |p: &Piece| matches!(p.shape, Shape::Door { .. });
    if mode.place && remove.is_none() {
        if ok
            && planned
            && let Some((_, base, _, hand)) = &setting_out
        {
            let chart = pbd_core::building::site::SiteChart {
                origin: cell,
                turn: mode.turn as u8,
                base: *base,
            };
            if super::construction::set_out_here(&mut save, hand, chart, &mut in_hand).is_some() {
                note.say(now, tr("site.set_out"));
            }
        } else if ok && candidate.len() == 1 && is_door(&candidate[0]) {
            // A door is crafted: one from the pack (design B8).
            match builds.place_door(&mut save, candidate[0].clone(), &mut hotbar.0) {
                Ok(()) => info!("BUILD hung a door"),
                Err(e) => {
                    info!("BUILD door refused: {e:?}");
                    note.say(now, e.text());
                }
            }
        } else if ok {
            let n = candidate.len();
            let ordered = placing_order(&site, &near, &candidate).unwrap_or(candidate);
            let named: Vec<&'static str> = ordered
                .iter()
                .map(|p| pbd_app::town_missions::build_name(&p.shape))
                .collect();
            // A piece of furniture is taken from the pack, in the same line
            // as its record (`furniture` design 5).
            let furniture = ordered.iter().find_map(|p| match p.shape {
                Shape::Furniture { kind, .. } => Some((p.clone(), kind)),
                _ => None,
            });
            let placed = match furniture {
                Some((piece, kind)) => {
                    let mut slots = hotbar.0.clone();
                    slots.take_item(pbd_core::inventory::Item::Good(kind.good()), 1)
                        && builds.place_with_hands(&mut save, piece, &slots)
                        && {
                            hotbar.0 = slots;
                            true
                        }
                }
                None => builds.place(&mut save, ordered),
            };
            if placed {
                info!("BUILD placed {n} pieces with {}", kit.name);
                // A house's pieces, as the town tree counts them.
                for name in named {
                    did.write(pbd_app::missions::Did::new(
                        pbd_core::missions::Kind::Build,
                        name,
                        1,
                    ));
                }
            } else {
                error!("build BLOCKED: the save did not accept the piece");
            }
        } else if let Some(r) = &reason {
            info!("BUILD refused: {r}");
        }
    }
    if (mode.take || (mode.place && tool == Tool::Remove))
        && let Some(id) = remove
    {
        if ok {
            if let Some(piece) = near.iter().find(|p| p.id() == id) {
                take_away(&mut builds, &mut save, piece, &mut hotbar.0, &mut note, now);
            }
        } else if let Some(r) = &reason {
            info!("BUILD refused removal: {r}");
        }
    }
    // A right click with a piece selected takes the aimed piece away too.
    if mode.take && remove.is_none() {
        let aimed = near
            .iter()
            .filter_map(|p| Some((reference(p)?.distance(hit.point), p)))
            .filter(|(d, _)| *d < 2.0)
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, p)| p.clone());
        if let Some(p) = aimed {
            match dependant(&near, &p) {
                Some(d) => {
                    let thing = tr(&format!("build.piece.{}", d.shape.class().id()));
                    note.say(now, tr_with("build.needed_by", &[("thing", &thing)]));
                }
                None if !container_empty(&save, &near, p.id()) => {
                    note.say(now, tr("build.not_empty"));
                }
                None => take_away(&mut builds, &mut save, &p, &mut hotbar.0, &mut note, now),
            }
        }
    }
}

/// Whether piece `id` among `near` is no container, or an empty one.
fn container_empty(save: &WorldSave, near: &[Piece], id: u64) -> bool {
    match near.iter().find(|p| p.id() == id).map(|p| &p.shape) {
        Some(Shape::Furniture { kind, .. }) if kind.slots().is_some() => {
            pbd_core::furniture::store::Store::of(&save.records, id, *kind, || {
                pbd_core::furniture::store::Store::empty(0)
            })
            .is_empty()
        }
        _ => true,
    }
}

/// Take a piece away; a door or a piece of furniture goes back in the
/// pack (design B8; `furniture` design 5).
fn take_away(
    builds: &mut Builds,
    save: &mut WorldSave,
    piece: &Piece,
    carried: &mut pbd_core::inventory::Slots,
    note: &mut super::farm::FarmNote,
    now: f32,
) {
    if matches!(piece.shape, Shape::Door { .. }) {
        match builds.remove_door(save, piece.id(), carried) {
            Ok(()) => info!("BUILD took a door down"),
            Err(e) => note.say(now, e.text()),
        }
    } else if let Shape::Furniture { kind, .. } = piece.shape {
        // A piece of furniture back into the pack, in the same line as its
        // removal (`furniture` design 5).
        let mut slots = carried.clone();
        if slots.give(pbd_core::inventory::Item::Good(kind.good()), 1) > 0 {
            note.say(now, tr("furniture.full"));
        } else if builds.remove_with_hands(save, piece.id(), &slots) {
            *carried = slots;
            info!("BUILD took up {}", kind.code());
        }
    } else if builds.remove(save, piece.id()) {
        info!("BUILD removed piece {}", piece.id());
    }
}

/// The ghost's meshes, made at startup so their pipelines are warmed with
/// the world's, then given the ghost's cut whenever it changes.
pub fn spawn_ghost(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for ok in [true, false] {
        let colour = if ok {
            Color::srgba(0.56, 0.86, 0.6, 0.45)
        } else {
            Color::srgba(1.0, 0.42, 0.34, 0.5)
        };
        let material = materials.add(StandardMaterial {
            base_color: colour,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        });
        let mesh = meshes.add(ghost_mesh(&MeshBuf {
            positions: vec![[0.0; 3]; 3],
            normals: vec![[0.0, 1.0, 0.0]; 3],
            uvs: vec![[0.0; 2]; 3],
        }));
        commands.spawn((
            Name::new("Build ghost"),
            Ghost(ok),
            TownRoot,
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::default(),
            Visibility::Hidden,
            bevy::light::NotShadowCaster,
            bevy::light::NotShadowReceiver,
        ));
    }
}

fn ghost_mesh(buf: &MeshBuf) -> Mesh {
    let n = buf.positions.len() as u32;
    Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, buf.positions.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, buf.normals.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, buf.uvs.clone())
    .with_inserted_indices(bevy::mesh::Indices::U32((0..n).collect()))
}

/// Cut the ghost's pieces and show them, a hair larger than they will be.
#[allow(clippy::type_complexity)]
pub fn draw_ghost(
    mode: Res<BuildMode>,
    assets: Res<TownAssets>,
    mut ghosts: Query<(&Ghost, &Mesh3d, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut shown: Local<Option<GhostShown>>,
    mut patch_cache: Local<Option<(u32, std::sync::Arc<pbd_core::settlement::chart::Patch>)>>,
) {
    if shown.as_ref() == mode.ghost.as_ref() {
        return;
    }
    *shown = mode.ghost.clone();
    let Some(ghost) = mode.ghost.as_ref() else {
        for (_, _, mut v) in &mut ghosts {
            *v = Visibility::Hidden;
        }
        return;
    };
    let Some(anchor) = ghost.pieces.iter().flat_map(|p| p.shape.cells()).min() else {
        return;
    };
    let patch = match patch_cache.as_ref() {
        Some((at, p)) if *at == anchor => p.clone(),
        _ => {
            let Some(d) = pbd_core::cell_key::key_direction(anchor) else {
                return;
            };
            let p = std::sync::Arc::new(patch_round(d, PATCH_REACH_M));
            *patch_cache = Some((anchor, p.clone()));
            p
        }
    };
    let cells = Cells::new(&patch);
    let of: Vec<&Piece> = ghost.pieces.iter().collect();
    let repeat = |_: &str| 2.0;
    let Some(c) = cut(&cells, anchor, &of, &assets.kits, &repeat, PLANET_RADIUS) else {
        return;
    };
    let mut buf = MeshBuf::default();
    // A door's leaf is drawn apart from the plot (design B8): shut, here.
    let leaves = c.solids.doors.iter().flat_map(|leaf| {
        pbd_core::building::door::leaf_mesh(leaf, c.solids.frame, 0.0).into_values()
    });
    for b in c.meshes.into_values().chain(leaves) {
        buf.positions.extend(&b.positions);
        buf.normals.extend(&b.normals);
        buf.uvs.extend(&b.uvs);
    }
    // A hair out along each face, so the ghost of a piece being taken away
    // is seen over the piece itself.
    for (p, n) in buf.positions.iter_mut().zip(&buf.normals) {
        for k in 0..3 {
            p[k] += n[k] * 0.02;
        }
    }
    for (g, mesh, mut v) in &mut ghosts {
        if g.0 == ghost.ok {
            if let Some(m) = meshes.get_mut(&mesh.0) {
                *m = ghost_mesh(&buf);
            }
            *v = Visibility::Visible;
        } else {
            *v = Visibility::Hidden;
        }
    }
}

/// The palette row and the kit picker, built once and hidden.
pub fn spawn_ui(
    mut commands: Commands,
    kit: Res<UiKit>,
    server: Res<AssetServer>,
    assets: Res<TownAssets>,
    existing: Query<(), With<PaletteRow>>,
) {
    if !existing.is_empty() {
        return;
    }
    use super::slots::{HOTBAR_BOTTOM, SLOT};
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: px(HOTBAR_BOTTOM),
                left: percent(50),
                margin: UiRect::left(px(-(PALETTE.len() as f32 * SLOT) / 2.0)),
                display: Display::None,
                ..default()
            },
            PaletteRow,
            super::menu::InPlayHud,
        ))
        .with_children(|row| {
            for (index, tool) in PALETTE.iter().enumerate() {
                row.spawn((
                    Node {
                        width: px(SLOT),
                        height: px(SLOT),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    kit.whole(Set::Stone, super::slots::slot_piece(index == 0)),
                    PaletteCell(index),
                ))
                .with_children(|cell| {
                    cell.spawn((
                        ImageNode::new(server.load(format!("textures/ui/build/{}.png", tool.id()))),
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(4),
                            top: px(4),
                            width: px(16),
                            height: px(16),
                            ..default()
                        },
                    ));
                });
            }
        });
    // The kit picker: every kit side by side, its wall, floor and roof.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(24),
                left: percent(50),
                margin: UiRect::left(px(-250.0)),
                width: px(500),
                flex_wrap: FlexWrap::Wrap,
                padding: UiRect::all(px(8)),
                column_gap: px(4),
                row_gap: px(4),
                display: Display::None,
                ..default()
            },
            kit.nine(Set::Stone, "frame"),
            KitPicker,
            super::menu::InPlayHud,
        ))
        .with_children(|panel| {
            panel.spawn((
                KitText::new(tr("build.kits"), ui_kit::LABEL),
                Node {
                    width: percent(100),
                    ..default()
                },
            ));
            for (index, k) in assets.kits.kits.iter().enumerate() {
                let texture = |name: &str| server.load(format!("textures/settlement/{name}.png"));
                panel
                    .spawn((
                        Node {
                            width: px(92),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(px(2)),
                            ..default()
                        },
                        KitCard(index),
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|card| {
                        card.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            ..default()
                        })
                        .with_children(|faces| {
                            for name in [&k.walls[0].outside, &k.floor, &k.roof_material] {
                                faces.spawn((
                                    ImageNode::new(texture(name)),
                                    Node {
                                        width: px(18),
                                        height: px(18),
                                        margin: UiRect::all(px(1)),
                                        ..default()
                                    },
                                ));
                            }
                        });
                        card.spawn(KitText::new(
                            tr(&format!("build.kit.{}", k.name)),
                            ui_kit::LABEL,
                        ));
                    });
            }
        });
}

/// Show the palette in the hotbar's place while building, mark the chosen
/// slot, and open the picker on K with the chosen kit lit.
#[allow(clippy::type_complexity)]
pub fn paint_ui(
    mode: Res<BuildMode>,
    kit: Res<UiKit>,
    mut rows: ParamSet<(
        Query<&mut Node, With<PaletteRow>>,
        Query<&mut Node, With<super::slots::HotbarRow>>,
        Query<&mut Node, With<KitPicker>>,
    )>,
    mut cells: Query<(&PaletteCell, &mut ImageNode)>,
    mut cards: Query<(&KitCard, &mut BackgroundColor)>,
    mut was: Local<Option<(bool, usize, bool, usize)>>,
) {
    let now = (mode.on, mode.slot, mode.picker, mode.kit);
    if *was == Some(now) {
        return;
    }
    *was = Some(now);
    let show = |on: bool| if on { Display::Flex } else { Display::None };
    for mut n in &mut rows.p0() {
        n.display = show(mode.on);
    }
    for mut n in &mut rows.p1() {
        n.display = show(!mode.on);
    }
    for mut n in &mut rows.p2() {
        n.display = show(mode.on && mode.picker);
    }
    for (cell, mut image) in &mut cells {
        *image = kit.whole(Set::Stone, super::slots::slot_piece(cell.0 == mode.slot));
    }
    for (card, mut bg) in &mut cards {
        *bg = BackgroundColor(if card.0 == mode.kit {
            Color::srgba(1.0, 0.9, 0.6, 0.3)
        } else {
            Color::NONE
        });
    }
}

/// The blueprints: every plain building of the shipped templates, one of
/// each kit and size.
pub fn load_blueprints(assets: Res<TownAssets>, mut blueprints: ResMut<Blueprints>) {
    if !blueprints.0.is_empty() {
        return;
    }
    let mut seen = std::collections::BTreeSet::new();
    for t in [
        &assets.village,
        &assets.walled,
        &assets.harbour,
        &assets.desert,
        &assets.tundra,
        &assets.jungle,
    ] {
        for def in &t.buildings {
            let Some(kit) = assets.kits.get(&def.kit) else {
                continue;
            };
            if plain(def, kit) && seen.insert((def.kit.clone(), def.cells.len(), def.storeys)) {
                blueprints.0.push(def.clone());
            }
        }
    }
    info!("BUILD {} blueprints", blueprints.0.len());
}

/// `PBD_BUILD_SHOT`: a capture instrument (`player-building` 4.2 to 4.6's
/// captures), never read in play. A comma list, done once at its frame:
/// `at:<frame>` (default 30), `mode:<slot>` opens build mode on that palette
/// slot, `kit:<index>`, `choice:<blueprint>`, `turn:<sixths>`, `picker`, and
/// `house:<n>` places the first `n` pieces of the chosen blueprint
/// `ahead:<metres>` (default 9) along the look, each through the rules and
/// [`Builds::place`] as a click would. After the house (design B8, B9):
/// `doors` puts that many doors in the pack (logged) and hangs one in each
/// doorway through [`Builds::place_door`]; `open` opens them as F would;
/// `view` raises architecture mode over the house, `view:door` over its
/// first door from outside, with `dist:<m>`, `tilt:<degrees>` and
/// `yaw:<degrees>` (an orbit), and `layer:<n>` layers up, `cut` the
/// cutaway (design B11); `quiet` shows no
/// ghost.
#[allow(clippy::too_many_arguments)]
pub fn scripted(
    state: Res<super::CaptureState>,
    mut mode: ResMut<BuildMode>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    frame: Option<Res<PlanetRenderFrame>>,
    fine: Res<PlanetFine>,
    mut save: ResMut<WorldSave>,
    mut builds: ResMut<Builds>,
    assets: Res<TownAssets>,
    blueprints: Res<Blueprints>,
    (mut hotbar, mut view): (ResMut<super::slots::Hotbar>, ResMut<BuildView>),
    mut cache: Local<GroundCache>,
    mut done: Local<bool>,
) {
    let Ok(plan) = std::env::var("PBD_BUILD_SHOT") else {
        return;
    };
    if *done {
        return;
    }
    let mut at = 30u32;
    let mut ahead = 9.0f32;
    let mut house = None;
    for step in plan.split(',') {
        let (k, v) = step.split_once(':').unwrap_or((step, ""));
        let n: usize = v.parse().unwrap_or(0);
        match k {
            "at" => at = n as u32,
            "ahead" => ahead = v.parse().unwrap_or(ahead),
            "house" => house = Some(n),
            _ => {}
        }
    }
    if state.frame < at {
        return;
    }
    *done = true;
    for step in plan.split(',') {
        let (k, v) = step.split_once(':').unwrap_or((step, ""));
        let n: usize = v.parse().unwrap_or(0);
        match k {
            "mode" => {
                if !mode.on {
                    mode.selected = hotbar.0.selected();
                }
                mode.on = true;
                mode.slot = n.min(PALETTE.len() - 1);
            }
            "kit" => mode.kit = n,
            "choice" => mode.choice = n,
            "turn" => mode.turn = n,
            "picker" => {
                mode.on = true;
                mode.picker = true;
            }
            "quiet" => mode.quiet = true,
            _ => {}
        }
    }
    let Some(n) = house else {
        return;
    };
    let Some((transform, _)) = cameras.iter().find(|(_, c)| c.is_active) else {
        return;
    };
    let eye = pbd_app::render_origin::local_of(frame.as_deref(), transform.translation());
    let up = eye.normalize_or(Vec3::Y);
    let look = transform.forward().as_vec3();
    let level = (look - up * look.dot(up)).normalize_or(up.any_orthonormal_vector());
    let target = eye + level * ahead;
    let patch = patch_round(target.normalize(), PATCH_REACH_M);
    let Some(cell) = patch
        .nearest(target.normalize())
        .and_then(|i| patch.keys.get(i).copied())
    else {
        warn!("PBD_BUILD_SHOT: no cell ahead");
        return;
    };
    let cells = Cells::new(&patch);
    let mut tops: HashMap<u32, Option<u16>> = HashMap::new();
    for &k in &patch.keys {
        tops.insert(k, ground_top(&fine, &save, &mut cache, k));
    }
    let ground_of = |c: u32| tops.get(&c).copied().flatten();
    let site = Site {
        cells: &cells,
        ground: &ground_of,
    };
    let Some(def) = blueprints
        .0
        .get(mode.choice.min(blueprints.0.len().saturating_sub(1)))
    else {
        return;
    };
    let Some(kit) = assets.kits.get(&def.kit) else {
        return;
    };
    let Some(index) = cells.index(cell) else {
        return;
    };
    // The ground floor on the highest ground under it.
    let base = blueprint(&patch, def, kit, index, mode.turn, 0)
        .ok()
        .and_then(|p| {
            let footprint: Vec<u32> = p
                .iter()
                .filter(|q| matches!(q.shape, Shape::Floor { layer: 0, .. }))
                .flat_map(|q| q.shape.cells())
                .collect();
            pbd_core::building::blueprint::base_over(&site, &footprint)
        });
    let Some(base) = base else {
        warn!(
            "PBD_BUILD_SHOT: no ground under the blueprint ({} of {} cells known)",
            tops.values().filter(|t| t.is_some()).count(),
            tops.len()
        );
        return;
    };
    let pieces = match blueprint(&patch, def, kit, index, mode.turn, base) {
        Ok(p) => p,
        Err(e) => {
            warn!("PBD_BUILD_SHOT: {e}");
            return;
        }
    };
    let near = builds.near(target, PATCH_REACH_M);
    let wanted: Vec<Piece> = pieces.into_iter().take(n).collect();
    // Pass after pass, what can go goes: the first `n` pieces of a
    // blueprint need not all stand on their own.
    let mut so_far = near.clone();
    let mut left = wanted;
    let mut placed = Vec::new();
    loop {
        let before = left.len();
        left.retain(|piece| {
            if pbd_core::building::rules::can_place(&site, &so_far, piece).is_ok() {
                so_far.push(piece.clone());
                placed.push(piece.clone());
                false
            } else {
                true
            }
        });
        if left.is_empty() || left.len() == before {
            break;
        }
    }
    for piece in &left {
        if let Err(rule) = pbd_core::building::rules::can_place(&site, &so_far, piece) {
            info!("PBD_BUILD_SHOT: {:?} refused: {}", piece.shape, rule.id());
        }
    }
    let count = placed.len();
    if builds.place(&mut save, placed.clone()) {
        info!(
            "PBD_BUILD_SHOT: placed {count} pieces of {} at layer {base}",
            def.name
        );
    }
    let has = |key: &str| {
        plan.split(',')
            .any(|step| step.split(':').next() == Some(key))
    };
    let value = |key: &str| {
        plan.split(',')
            .find_map(|step| step.strip_prefix(&format!("{key}:")))
            .and_then(|v| v.parse::<f32>().ok())
    };
    // Doors in each doorway, from the pack.
    let mut doors = Vec::new();
    if has("doors") {
        let doorways: Vec<Piece> = placed
            .iter()
            .filter_map(|p| match p.shape {
                Shape::Wall {
                    a,
                    b,
                    layer,
                    opening: Opening::Door,
                } => Some(Piece::new(&p.kit, Shape::door(a, b, layer))),
                _ => None,
            })
            .collect();
        hotbar.0.give(
            pbd_core::inventory::Item::Good(pbd_core::goods::Good::DOOR),
            doorways.len() as u16,
        );
        info!("PBD_BUILD_SHOT: {} doors put in the pack", doorways.len());
        for door in doorways {
            match builds.place_door(&mut save, door.clone(), &mut hotbar.0) {
                Ok(()) => doors.push(door),
                Err(e) => warn!("PBD_BUILD_SHOT: door refused: {e:?}"),
            }
        }
        info!("PBD_BUILD_SHOT: hung {} doors", doors.len());
    }
    if has("open") {
        for door in &doors {
            Builds::set_door(&mut save, door.id(), true);
        }
        // Cut again, so the leaves stand as their records say.
        builds.revision += 1;
        info!("PBD_BUILD_SHOT: opened {} doors", doors.len());
    }
    if has("view") {
        let centre = placed
            .iter()
            .flat_map(|p| p.shape.cells())
            .filter_map(pbd_core::cell_key::key_direction)
            .fold(Vec3::ZERO, |a, d| a + d)
            .normalize_or(target.normalize());
        let (focus, heading) = match (plan.contains("view:door"), doors.first()) {
            (true, Some(door)) => {
                let mid = door
                    .shape
                    .cells()
                    .iter()
                    .filter_map(|&c| pbd_core::cell_key::key_direction(c))
                    .fold(Vec3::ZERO, |a, d| a + d)
                    .normalize();
                (mid, (centre - mid).normalize_or(level))
            }
            _ => (centre, level),
        };
        let layer = base + value("layer").unwrap_or(0.0) as u16;
        let r = PLANET_RADIUS + column::layer_altitude(layer as usize);
        mode.on = true;
        view.enter(
            (focus * r).as_dvec3(),
            layer,
            heading.as_dvec3(),
            eye.as_dvec3(),
            target.as_dvec3(),
        );
        if let Some(d) = value("dist") {
            view.distance = d;
        }
        if let Some(t) = value("tilt") {
            view.tilt = t.to_radians();
        }
        if let Some(y) = value("yaw") {
            view.orbit(y.to_radians(), 0.0);
        }
        // `cut`: the cutaway over the build layer (design B11), as End.
        view.cut = has("cut");
        info!(
            "PBD_BUILD_SHOT: architecture mode, layer {layer}, cutaway {}",
            view.cut
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbd_core::inventory::Item;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .add_message::<MouseWheel>()
            .insert_resource(pbd_app::walking::WalkingReadout {
                active: true,
                ..default()
            })
            .insert_resource(pbd_app::vehicles::Aboard(None))
            .init_resource::<super::super::slots::Hotbar>()
            .init_resource::<BuildMode>()
            .add_systems(Update, take_input);
        app
    }

    fn press(app: &mut App, key: KeyCode) {
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.release_all();
        keys.clear();
        keys.press(key);
        app.update();
        let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        keys.release_all();
        keys.clear();
    }

    /// `player-building` task 4.1 (design B5, B10): B on foot opens build
    /// mode; while it is open the number row picks the palette, not the
    /// hotbar; B again closes it, and the hotbar and its selection are as
    /// they were. Aboard a craft B does not open it (it brakes there).
    #[test]
    fn build_mode_gives_the_hotbar_and_its_selection_back() {
        let mut app = app();
        {
            let mut hotbar = app
                .world_mut()
                .resource_mut::<super::super::slots::Hotbar>();
            hotbar.0.give(Item::Block(Material::Dirt), 12);
            hotbar.0.give(Item::Good(pbd_core::goods::Good::LOG), 5);
            hotbar.0.select(1);
        }
        let before: Vec<_> = {
            let h = &app.world().resource::<super::super::slots::Hotbar>().0;
            (0..10).map(|i| h.get(i)).collect()
        };
        app.update();
        press(&mut app, KeyCode::KeyB);
        assert!(app.world().resource::<BuildMode>().on, "B opens it");
        press(&mut app, KeyCode::Digit4);
        assert_eq!(app.world().resource::<BuildMode>().slot, 3, "the palette's");
        assert_eq!(
            app.world()
                .resource::<super::super::slots::Hotbar>()
                .0
                .selected(),
            1,
            "the number row is build mode's while it is open"
        );
        press(&mut app, KeyCode::KeyB);
        assert!(!app.world().resource::<BuildMode>().on, "B closes it");
        let h = &app.world().resource::<super::super::slots::Hotbar>().0;
        assert_eq!(h.selected(), 1, "the selection comes back");
        let after: Vec<_> = (0..10).map(|i| h.get(i)).collect();
        assert_eq!(after, before, "the slots are untouched");
        // Aboard: B is the brake, not build mode.
        app.insert_resource(pbd_app::vehicles::Aboard(Some(Entity::PLACEHOLDER)));
        press(&mut app, KeyCode::KeyB);
        assert!(!app.world().resource::<BuildMode>().on, "not aboard");
    }
}
