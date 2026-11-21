//! Build mode (`player-building` design B5): N on foot swaps the hotbar for
//! the kit's palette, the aim snaps the chosen piece to the cut, the ghost
//! shows it green or red with the rule it breaks, a left click places it and
//! a right click takes the aimed piece away. K opens the kit picker, where
//! the wheel steps the kit.
//!
//! The rules are the core's (`pbd_core::building::rules`), the pieces are
//! kept by `pbd_app::builds`, and nothing here writes the save but through
//! [`Builds::place`] and [`Builds::remove`].

use super::digging::{material_of, record_of};
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
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
}

/// The palette, in M6's order.
pub const PALETTE: [Tool; 10] = [
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
        }
    }
}

/// How far the build aim reaches, metres.
pub const REACH_M: f32 = 7.0;
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
    /// Whether the kit picker is open.
    pub picker: bool,
    /// The hotbar's selection when build mode opened, given back on leaving.
    selected: usize,
    /// This frame's clicks, taken from the world's readers while building.
    place: bool,
    take: bool,
    /// What the ghost shows: its pieces, whether they may go, and the piece
    /// a removal aims at.
    pub ghost: Option<GhostShown>,
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

/// N on foot opens and closes build mode; while it is open the clicks, the
/// number row and K are build mode's, taken from every other reader before
/// `Update` (so a click does not also dig, cast or eat).
#[allow(clippy::too_many_arguments)]
pub fn take_input(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut buttons: ResMut<ButtonInput<MouseButton>>,
    mut wheel: MessageReader<MouseWheel>,
    menu: Option<Res<MenuOpen>>,
    walking: Option<Res<pbd_app::walking::WalkingReadout>>,
    aboard: Option<Res<pbd_app::vehicles::Aboard>>,
    mut hotbar: ResMut<super::slots::Hotbar>,
    mut mode: ResMut<BuildMode>,
    kits: Option<Res<TownAssets>>,
    blueprints: Res<Blueprints>,
) {
    mode.place = false;
    mode.take = false;
    let steps: i32 = wheel
        .read()
        .map(|m| (m.y < 0.0) as i32 - (m.y > 0.0) as i32)
        .sum();
    let on_foot = walking.is_some_and(|w| w.active) && aboard.is_none_or(|a| a.0.is_none());
    if menu.is_some_and(|m| m.0) {
        return;
    }
    if keys.just_pressed(KeyCode::KeyN) && (on_foot || mode.on) {
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
    if keys.just_pressed(KeyCode::KeyK) {
        mode.picker = !mode.picker;
    }
    keys.reset(KeyCode::KeyK);
    keys.reset(KeyCode::KeyG);
    if steps != 0 && mode.picker {
        let n = kits.as_ref().map_or(1, |k| k.kits.kits.len().max(1)) as i32;
        mode.kit = (mode.kit as i32 + steps).rem_euclid(n) as usize;
    } else if steps != 0 {
        if PALETTE[mode.slot] == Tool::Blueprint && keys.pressed(KeyCode::ShiftLeft) {
            let n = blueprints.0.len().max(1) as i32;
            mode.choice = (mode.choice as i32 + steps).rem_euclid(n) as usize;
        } else {
            mode.turn = (mode.turn as i32 + steps).rem_euclid(6) as usize;
        }
    }
    if let Some(kits) = kits {
        mode.kit = mode.kit.min(kits.kits.kits.len().saturating_sub(1));
    }
    if !mode.picker {
        mode.place = buttons.just_pressed(MouseButton::Left);
        mode.take = buttons.just_pressed(MouseButton::Right);
        buttons.reset(MouseButton::Left);
        buttons.reset(MouseButton::Right);
    }
}

/// The ground's first air layer in a cell, cached for a fine set and an edit
/// revision.
#[derive(Default)]
pub struct GroundCache {
    key: (u64, u64),
    tops: HashMap<u32, Option<u16>>,
}

fn ground_top(
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
        let record = record_of(fine, cell)?;
        let surface = fine.set.finest_records()[record].direction_height[3];
        let start = column::layer_at(surface + 12.0)?;
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
) -> Option<Hit> {
    let solid = |p: Vec3| -> bool {
        super::digging::sample_at(fine, contact, p).is_some_and(|s| s.solid)
            || builds.0.iter().any(|b| {
                let q = b.frame.local(p);
                b.holds_in(q, 0.0, 0.02) || {
                    let (floor, _) = b.stand_in(q, 0.0);
                    floor.is_some_and(|f| f > q.y - 0.12 && f <= q.y + 0.0)
                }
            })
    };
    let mut travelled = 0.0;
    let mut last = eye;
    while travelled <= REACH_M {
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
fn reference(piece: &Piece) -> Option<Vec3> {
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
        Shape::Lantern { .. } => 1.0,
        Shape::Roof { .. } => 1.0,
        Shape::Floor { .. } => 0.0,
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
    blueprints: Res<Blueprints>,
    mut note: ResMut<super::farm::FarmNote>,
    time: Res<Time>,
    mut cache: Local<GroundCache>,
    mut patch_cache: Local<Option<(u32, std::sync::Arc<pbd_core::settlement::chart::Patch>)>>,
) {
    if !mode.on || mode.picker {
        if mode.ghost.is_some() {
            mode.ghost = None;
        }
        return;
    }
    let Some((transform, _)) = cameras.iter().find(|(_, c)| c.is_active) else {
        return;
    };
    let eye = pbd_app::render_origin::local_of(frame.as_deref(), transform.translation());
    let look = transform.forward().as_vec3();
    let hit = march(&fine, &contact, &structures, eye, look);
    let Some(hit) = hit else {
        mode.ghost = None;
        mode.reason = None;
        return;
    };
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
    let candidate: Option<Vec<Piece>> = match tool {
        Tool::Wall => wall(Opening::None),
        Tool::Door => wall(Opening::Door),
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
            let def = blueprints
                .0
                .get(mode.choice.min(blueprints.0.len().saturating_sub(1)));
            def.and_then(|def| {
                let kit = kits.get(&def.kit)?;
                let at = cells.index(cell)?;
                blueprint(&patch, def, kit, at, mode.turn, storey).ok()
            })
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
            None => (true, None),
        }
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
        let text = if remove.is_some() {
            tr_with("build.needed_by", &[("thing", &tr(r))])
        } else {
            tr(r)
        };
        note.say_for(now, text, 0.25);
    }
    if mode.place && remove.is_none() {
        if ok {
            let n = candidate.len();
            let ordered = placing_order(&site, &near, &candidate).unwrap_or(candidate);
            if builds.place(&mut save, ordered) {
                info!("BUILD placed {n} pieces with {}", kit.name);
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
            if builds.remove(&mut save, id) {
                info!("BUILD removed piece {id}");
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
                None => {
                    if builds.remove(&mut save, p.id()) {
                        info!("BUILD removed piece {}", p.id());
                    }
                }
            }
        }
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
    for b in c.meshes.values() {
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
                margin: UiRect::left(px(-210.0)),
                width: px(420),
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
                            width: px(64),
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
/// [`Builds::place`] as a click would.
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
    hotbar: Res<super::slots::Hotbar>,
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
        warn!("PBD_BUILD_SHOT: no ground under the blueprint");
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
    let placed = match placing_order(&site, &near, &wanted) {
        Ok(placed) => placed,
        Err(rule) => {
            info!("PBD_BUILD_SHOT: refused: {}", rule.id());
            return;
        }
    };
    let count = placed.len();
    if builds.place(&mut save, placed) {
        info!(
            "PBD_BUILD_SHOT: placed {count} pieces of {} at layer {base}",
            def.name
        );
    }
}
