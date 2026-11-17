//! The game's flowing water (`openspec/changes/water-flow`, "In the engine"):
//! the rule's tick over the column tier (E3, E4), the water mesher (E6), the
//! texture panned along the flow (E7) and the flow-arrow overlay.
//!
//! The rule is `pbd_core::liquid`; the tier is its grid
//! (`planet::column::TierGrid`). What is here is the app's half: when a tick
//! runs, how its changes reach the save and the GPU, and how every water cell
//! the sea sheet does not draw is drawn, as Luanti's liquid mesher draws a
//! node (`content_mapblock.cpp`, `drawLiquidNode`), on the hex grid (design
//! 7e).
//!
//! Between two ticks the water is drawn easing from the last tick to this one
//! (design, "Smooth water between ticks", M31's Pour tab, approved
//! 2026-11-15): the mesher keeps the tick before, every vertex carries where
//! it was as well as where it is, and `vertex_flowing` eases between them. A
//! level eases (S1, smoothstepped); a newly wet cell sweeps in from the side
//! its water came from (S2) or grows down from the water over it (S3); a
//! dried cell shrinks back toward where its water went and is drawn for that
//! one tick only (S4). Fronts ease linearly, so a pour creeps at one speed.
//! The rule, its tick and the save are untouched. `PBD_WATER_SMOOTH=0` draws
//! each tick as it lands, as before.

use crate::planet::column::{ColumnTier, sea_layer};
use crate::planet::lod::FineSet;
use crate::planet::{GpuCell, PLANET_RADIUS, PlanetContact, PlanetFine, PlanetRenderFrame};
use crate::saves::WorldSave;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use pbd_core::column::{self, LAYERS};
use pbd_core::edits::Edit;
use pbd_core::liquid;
use pbd_core::terrain::Material;
use std::sync::Arc;

/// Metres a second a flowing top's waves stream at a slope of `PAN_SLOPE`
/// (the mockups' 1.6 m/s, design 7c).
const PAN_K: f32 = 1.6;
/// The slope, metres a metre, that streams at `PAN_K`.
const PAN_SLOPE: f32 = 0.12;
/// The sea's fragment streams a top by `flow_uv * 0.35` metres a second.
const SHADER_FLOW: f32 = 0.35;
/// How far over the water the arrows float, m.
const ARROW_LIFT: f32 = 0.05;
/// The kinds of face the sea's fragment is told (`vertex_flowing`).
const KIND_TOP: f32 = 1.0;
const KIND_FALLING: f32 = 2.0;
const KIND_STILL: f32 = 3.0;
/// How thin a cell that flows in (S2) starts, or one that dries (S4) ends,
/// as a share of its height: M31's `POUR.THIN`.
pub const THIN: f32 = 0.25;
/// A vertex's ease flags (`vertex_flowing`): a front (S2-S4 and a piece
/// that appears or goes in S1), eased linearly; else a level, smoothstepped.
pub const EASE_FRONT: u32 = 1;
/// A dried cell's piece, or one a cell lost this tick: hidden once the
/// tick has played.
pub const EASE_GONE: u32 = 2;
/// Lanes (vec4s) a vertex in the water pass's buffer.
pub const LANES_PER_VERTEX: usize = 4;

/// The game's own water as the water pass draws it (`water-flow` E6): four
/// lanes a vertex, triangles end to end (`vertex_flowing` in `water.wgsl`),
/// and which make of it this is.
#[derive(Resource, Clone, Default, bevy::render::extract_resource::ExtractResource)]
pub struct FlowingWater {
    pub lanes: Arc<Vec<[f32; 4]>>,
    pub version: u64,
}

/// How far the drawing is through the tick that last landed: `linear`
/// 0..1 for the fronts, `smooth` its smoothstep for the levels. Both 1 when
/// the water is drawn as it is (`PBD_WATER_SMOOTH=0`, or the tick played).
#[derive(
    Resource, Clone, Copy, Debug, PartialEq, bevy::render::extract_resource::ExtractResource,
)]
pub struct WaterEase {
    pub linear: f32,
    pub smooth: f32,
}

impl Default for WaterEase {
    fn default() -> Self {
        Self {
            linear: 1.0,
            smooth: 1.0,
        }
    }
}

/// When the next tick is due.
#[derive(Resource, Default)]
pub struct WaterClock {
    since_s: f32,
    /// Ticks run, for the log.
    pub ticks: u64,
}

/// The tick the drawing eases across: the fine set before it, which set it
/// made, and how long since it landed.
#[derive(Resource)]
pub struct WaterStep {
    /// The set before the last tick, until the mesher has used it.
    prev: Option<Arc<FineSet>>,
    /// The version and set the last tick made, which `prev` eases into.
    after: Option<(u64, usize)>,
    /// Seconds since the last tick landed.
    pub since_s: f32,
    /// A capture's frozen fraction of the tick (`--water-tick-frac`): the
    /// rule does not tick on its own while it is set.
    pub hold: Option<f32>,
    /// Whether to ease at all (`PBD_WATER_SMOOTH=0` turns it off).
    pub smooth: bool,
}

impl Default for WaterStep {
    fn default() -> Self {
        Self {
            prev: None,
            after: None,
            since_s: liquid::TICK_S,
            hold: None,
            smooth: std::env::var("PBD_WATER_SMOOTH").map_or(true, |v| v.trim() != "0"),
        }
    }
}

/// Whether the flow arrows are drawn: the player's setting, or the debug key
/// (F5) for this session.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlowArrows {
    pub setting: bool,
    pub debug: bool,
}

impl FlowArrows {
    pub fn shown(self) -> bool {
        self.setting || self.debug
    }
}

#[derive(Component)]
struct ArrowMesh;

/// Which fine set the mesher last made the water for.
#[derive(Resource, Default)]
struct Meshed {
    version: u64,
    set: usize,
}

/// Run `ticks` of the rule over the tier now, as a tick does: every change
/// journaled under the water, the changed records repacked, the contact and
/// the fine set's version moved. Returns how many cells changed. Nothing
/// moves if the save refuses. The set before the last of them is kept in
/// `step` for the drawing to ease from.
pub fn run_ticks(
    fine: &mut PlanetFine,
    contact: &mut PlanetContact,
    save: &mut WorldSave,
    ticks: usize,
    step: &mut WaterStep,
) -> usize {
    if fine.set.columns.water.queue.is_empty() || ticks == 0 {
        return 0;
    }
    let before = fine.set.clone();
    let mut set = (*fine.set).clone();
    let mut last_before: Option<Arc<FineSet>> = None;
    let mut ran = 0;
    let mut changed = 0;
    for i in 0..ticks {
        if set.columns.water.queue.is_empty() {
            break;
        }
        // Only the last tick is eased across: keep the set before it (the
        // one already published when it is the only tick).
        if step.smooth && i > 0 && i + 1 == ticks {
            last_before = Some(Arc::new(set.clone()));
        }
        let made = set.columns.tick_water(liquid::LOOP_MAX);
        let edits: Vec<Edit> = made
            .iter()
            .filter_map(|&(slot, layer, material)| {
                Some(Edit {
                    cell: *set.columns.water.keys.get(slot)?,
                    layer: layer as u16,
                    material,
                })
            })
            .collect();
        // The rule moves water through air and water only, so a player's
        // block or light under a water change is a bug, and the save is about
        // to lose it: said with what was there and what replaces it.
        for edit in &edits {
            if let Some(&(_, placed)) = save
                .edits
                .for_cell(edit.cell)
                .iter()
                .find(|&&(l, m)| l == edit.layer && m != Material::Air && !m.is_water())
            {
                error!(
                    "REMOVED {placed:?} at cell {} layer {} by the water rule, which has no \
                     business there: was {placed:?}, now {:?}",
                    edit.cell, edit.layer, edit.material
                );
            }
        }
        if !save.accept_water(&edits) {
            error!("water BLOCKED: the save did not take {} cells", edits.len());
            return 0;
        }
        changed += made.len();
        ran += 1;
    }
    let set = Arc::new(set);
    contact.set_fine(&set);
    let made = Arc::as_ptr(&set) as usize;
    fine.set = set;
    fine.version += 1;
    step.prev = if !step.smooth {
        None
    } else if ran == 1 && ticks == 1 {
        Some(before)
    } else if ran == ticks {
        last_before
    } else {
        None
    };
    step.after = step.prev.as_ref().map(|_| (fine.version, made));
    step.since_s = 0.0;
    changed
}

/// Once a second, while anything is queued and no set is in flight (a set
/// built before a tick would land without it).
fn tick_water(
    time: Res<Time>,
    mut clock: ResMut<WaterClock>,
    mut step: ResMut<WaterStep>,
    fine: Option<ResMut<PlanetFine>>,
    contact: Option<ResMut<PlanetContact>>,
    save: Option<ResMut<WorldSave>>,
    refresh: Option<Res<crate::planet::LodRefresh>>,
) {
    step.since_s += time.delta_secs();
    if step.hold.is_some() {
        return;
    }
    let (Some(mut fine), Some(mut contact), Some(mut save), Some(refresh)) =
        (fine, contact, save, refresh)
    else {
        return;
    };
    if fine.set.columns.water.queue.is_empty() {
        clock.since_s = 0.0;
        return;
    }
    clock.since_s += time.delta_secs();
    if clock.since_s < liquid::TICK_S || refresh.in_flight_s().is_some() {
        return;
    }
    clock.since_s -= liquid::TICK_S;
    clock.since_s = clock.since_s.min(liquid::TICK_S);
    let changed = run_ticks(&mut fine, &mut contact, &mut save, 1, &mut step);
    clock.ticks += 1;
    debug!(
        "water tick {}: {changed} cells changed, {} queued",
        clock.ticks,
        fine.set.columns.water.queue.len()
    );
}

/// How far through the tick the drawing is, for the water pass and the
/// splash.
fn ease_water(step: Res<WaterStep>, mut ease: ResMut<WaterEase>) {
    let linear = if step.smooth {
        (step.hold.unwrap_or(step.since_s) / liquid::TICK_S).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let want = WaterEase {
        linear,
        smooth: linear * linear * (3.0 - 2.0 * linear),
    };
    if *ease != want {
        *ease = want;
    }
}

/// A mesh of one degenerate triangle with the arrows' attributes: what the
/// arrows are born with, so the loading screen builds their pipeline.
fn placeholder() -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0f32; 3]; 3])
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0f32, 1.0, 0.0]; 3])
    .with_inserted_indices(Indices::U32(vec![0, 1, 2]))
}

fn spawn_arrows(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Name::new("Flow arrows"),
        ArrowMesh,
        Mesh3d(meshes.add(placeholder())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.95, 0.12, 0.10),
            unlit: true,
            cull_mode: None,
            double_sided: true,
            depth_bias: 4.0,
            ..default()
        })),
        Transform::IDENTITY,
        Visibility::Hidden,
        bevy::camera::visibility::NoFrustumCulling,
        bevy::light::NotShadowCaster,
        bevy::light::NotShadowReceiver,
    ));
}

/// One vertex of the water as the pass draws it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlowVertex {
    /// Where it is this tick, body-local.
    pub position: Vec3,
    /// Where it was the tick before; `position` when it has not moved.
    pub prev: Vec3,
    pub normal: Vec3,
    /// The flow in the face's tangent frame as the sea's fragment reads it.
    /// The flow lane: a top's pan (xy) and how near a fall's lip it is (z);
    /// a face's froth weight (x) and how far under its fall's lip it is (z).
    pub flow: Vec3,
    /// The face's kind, and the sky's share at it, 0..1.
    pub kind: f32,
    pub sky: f32,
    /// `EASE_FRONT` and `EASE_GONE`.
    pub flags: u32,
}

/// A fall's foot: where it lands (body-local), how many layers it fell, the
/// sky's share there, and whether it landed this tick (its splash starts
/// when the column reaches it, not when the tick starts).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Foot {
    pub at: Vec3,
    pub height: f32,
    pub sky: f32,
    pub born: bool,
    /// The metres from the lip's water to the water it lands in, `D` of the
    /// proposed mist rule (`water-flow` 6.6): a part layer counts.
    pub drop_m: f32,
    /// The way the water goes over, a unit tangent at the foot pointing away
    /// from the side the fall is fed from; zero where no side feeds it.
    pub over: Vec3,
    /// How deep the water is under the foot's surface, m: how deep a bubble
    /// can be born.
    pub pool_m: f32,
    /// The foot's column in the tier, which keys its fade across frames.
    pub column: u32,
    /// The landing cell's layer: with the column, which foot this is (a
    /// cascade can stack two in one column).
    pub layer: u32,
}

impl Foot {
    /// The key its fades are kept under across frames.
    pub fn key(&self) -> u64 {
        u64::from(self.column) << 32 | u64::from(self.layer)
    }
}

/// What `cell_pieces` finds of a fall's foot, before the tick's own fields.
#[derive(Clone, Copy, Debug, PartialEq)]
struct FootFound {
    at: Vec3,
    height: f32,
    drop_m: f32,
    over: Vec3,
    pool_m: f32,
}

impl FootFound {
    fn foot(self, sky: f32, born: bool, column: usize, layer: usize) -> Foot {
        Foot {
            at: self.at,
            height: self.height,
            sky,
            born,
            drop_m: self.drop_m,
            over: self.over,
            pool_m: self.pool_m,
            column: column as u32,
            layer: layer as u32,
        }
    }
}

/// What the mesher makes: the water's triangles and the arrows'.
#[derive(Default, Debug)]
pub struct LiquidMesh {
    /// Triangles end to end.
    pub vertices: Vec<FlowVertex>,
    pub arrow_positions: Vec<[f32; 3]>,
    pub arrow_indices: Vec<u32>,
    /// Water cells drawn, tops drawn, and tops with a flow.
    pub cells: usize,
    pub tops: usize,
    pub flowing_tops: usize,
    /// Cells wet this tick and not the last (S2, S3), and the other way
    /// round (S4, drawn this one tick more).
    pub born: usize,
    pub dried: usize,
    /// The foot of every fall: what the splash draws from.
    pub feet: Vec<Foot>,
}

/// The tangent frame the sea's fragment builds off a face (`fragment` in
/// `water.wgsl`): X, or Y where the face is near X, off the face.
fn shader_frame(face: Vec3) -> (Vec3, Vec3) {
    let reference = if face.x.abs() > 0.9 { Vec3::Y } else { Vec3::X };
    let u = (reference - face * reference.dot(face)).normalize_or(Vec3::X);
    (u, face.cross(u))
}

impl LiquidMesh {
    /// The triangles as the water pass reads them: four lanes a vertex.
    pub fn lanes(&self) -> Vec<[f32; 4]> {
        let mut out = Vec::with_capacity(self.vertices.len() * LANES_PER_VERTEX);
        for v in &self.vertices {
            out.push(v.position.extend(v.sky).to_array());
            out.push(v.normal.extend(v.kind).to_array());
            out.push(v.flow.extend(0.0).to_array());
            out.push(v.prev.extend(v.flags as f32).to_array());
        }
        out
    }

    fn arrow(&mut self, at: Vec3, along: Vec3, across: Vec3, length: f32) {
        let half = along * length * 0.5;
        let shaft = across * 0.07;
        let head = across * 0.28;
        let base = self.arrow_positions.len() as u32;
        let tail = at - half;
        let neck = at + half * 0.25;
        let tip = at + half;
        for p in [
            tail - shaft,
            tail + shaft,
            neck + shaft,
            neck - shaft,
            neck - head,
            neck + head,
            tip,
        ] {
            self.arrow_positions.push(p.to_array());
        }
        self.arrow_indices
            .extend([0, 1, 2, 0, 2, 3, 4, 5, 6].map(|i| base + i));
    }

    /// One piece's vertices: where each is (`now`), where it was (`was`).
    fn emit(&mut self, piece: &Piece, now: &[Vec3], was: &[Vec3], sky: f32, flags: u32) {
        for ((v, &position), &prev) in piece.v[..piece.n].iter().zip(now).zip(was) {
            self.vertices.push(FlowVertex {
                position,
                prev,
                normal: v.n,
                flow: v.flow,
                kind: v.kind,
                sky,
                flags,
            });
        }
    }

    /// A piece drawn where it is, not moving.
    fn emit_still(&mut self, piece: &Piece, sky: f32) {
        for v in &piece.v[..piece.n] {
            self.vertices.push(FlowVertex {
                position: v.p,
                prev: v.p,
                normal: v.n,
                flow: v.flow,
                kind: v.kind,
                sky,
                flags: 0,
            });
        }
    }
}

/// What a neighbour across a side is, for the mesher.
#[derive(Clone, Copy)]
struct Side {
    material: Material,
    above_water: bool,
}

/// Luanti's `getCornerLevel` on hex (design 7e): over the three cells round a
/// corner (`None` is off the tier, left out), the corner is full where any has
/// water above it or is a source; at the floor (2 cm up) where two or more are
/// air; else the mean of the flowing cells' levels, or the middle where none
/// flows. A fraction of the layer.
fn corner_level(cells: &[Option<Side>; 3]) -> f32 {
    let (mut sum, mut count, mut air) = (0.0, 0, 0);
    for cell in cells.iter().flatten() {
        if cell.above_water || cell.material == Material::Water {
            return 1.0;
        }
        if let Some((level, _)) = cell.material.flow() {
            sum += (f32::from(level) + 0.5) / 8.0;
            count += 1;
        } else if cell.material == Material::Air {
            air += 1;
        }
    }
    if air >= 2 {
        0.02
    } else if count > 0 {
        sum / count as f32
    } else {
        0.5
    }
}

/// Which piece of a cell: matched by this across two ticks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PieceId {
    /// The top, over no water.
    Top,
    /// A side, over air or a fall's side over lower water.
    Side(usize),
    /// The bottom, over air.
    Bottom,
}

/// One vertex of a piece.
#[derive(Clone, Copy, Debug, Default)]
struct PieceVertex {
    p: Vec3,
    n: Vec3,
    flow: Vec3,
    kind: f32,
}

/// The most vertices a piece has: a hexagonal top or bottom, six triangles.
const PIECE_MAX: usize = 18;

/// One piece of one wet cell in one tick: its triangles' vertices, and the
/// same piece collapsed (a top pressed to the cell's top, a side's bottom
/// raised to its top), which is where a piece that is in only one of two
/// ticks starts or ends (M31's `cellPieces`).
#[derive(Clone, Copy, Debug)]
struct Piece {
    id: PieceId,
    n: usize,
    v: [PieceVertex; PIECE_MAX],
    alt: [Vec3; PIECE_MAX],
}

impl Piece {
    fn new(id: PieceId) -> Self {
        Self {
            id,
            n: 0,
            v: [PieceVertex::default(); PIECE_MAX],
            alt: [Vec3::ZERO; PIECE_MAX],
        }
    }

    fn push(&mut self, v: PieceVertex, alt: Vec3) {
        self.v[self.n] = v;
        self.alt[self.n] = alt;
        self.n += 1;
    }

    /// Its vertices' positions; the first `n` are its own.
    fn positions(&self) -> [Vec3; PIECE_MAX] {
        let mut out = [Vec3::ZERO; PIECE_MAX];
        for (o, v) in out.iter_mut().zip(&self.v[..self.n]) {
            *o = v.p;
        }
        out
    }
}

/// A cell's shape, the same in every tick.
struct CellFrame {
    degree: usize,
    centre: Vec3,
    corners: [Vec3; 6],
    sides: [Option<usize>; 6],
    /// The sea's sheet is listed over the cell, so the sea's own top here is
    /// the sheet's (`planet::column::sheet_listed`, `ocean-environments`
    /// B22): in a cave the sea reaches too, not only under sea level.
    sheet_listed: bool,
    /// The shader's tangent frame at the centre.
    t: Vec3,
    b: Vec3,
}

impl CellFrame {
    fn of(tier: &ColumnTier, slot: usize, cell: &GpuCell) -> Self {
        let degree = cell.degree().min(6);
        let centre = Vec3::from_slice(&cell.direction_height[..3]).normalize_or(Vec3::Y);
        let mut corners = [centre; 6];
        for (i, corner) in corners.iter_mut().enumerate().take(degree) {
            *corner = Vec3::from_slice(&cell.corners[i][..3]).normalize_or(centre);
        }
        let (t, b) = shader_frame(centre);
        Self {
            degree,
            centre,
            corners,
            sides: tier.sides_of(slot),
            sheet_listed: crate::planet::column::sheet_listed(cell),
            t,
            b,
        }
    }

    /// Side `s`'s outward normal, in the tangent plane at the centre.
    fn side_normal(&self, s: usize) -> Vec3 {
        let (a, z) = (self.corners[s], self.corners[(s + 1) % self.degree]);
        let normal = ((a + z) * 0.5 - self.centre).normalize_or(self.centre);
        (normal - self.centre * normal.dot(self.centre)).normalize_or(normal)
    }
}

/// The height the water's top at `slot`, `layer` is DRAWN at, metres over
/// the radius, at the cell's centre: the mean of its corners' levels
/// (Luanti's corner rule, `corner_level`), as `cell_pieces` meshes it. `None`
/// where the layer is not water, water stands on it, or it is the sea's own
/// top, which the sheet draws. What decides whether the eye is under a
/// river's water (`ocean-environments` B17): the surface the eye sees, not
/// the cell's own level, which the drawn top stands over by up to a layer's
/// share where a neighbour is full.
pub fn drawn_top_m(tier: &ColumnTier, slot: usize, layer: usize) -> Option<f32> {
    let column = tier.columns.get(slot)?;
    let here = column.material(layer);
    if !here.is_water() || column.material(layer + 1).is_water() {
        return None;
    }
    let sea = sea_layer();
    if here == Material::Water && layer + 1 == sea {
        return None;
    }
    let degree = tier.degree_of(slot);
    let sides = tier.sides_of(slot);
    let side_of = |s: usize| -> Option<Side> {
        let n = sides[s]?;
        Some(Side {
            material: material_in(tier, n, layer),
            above_water: material_in(tier, n, layer + 1).is_water(),
        })
    };
    let me = Some(Side {
        material: here,
        above_water: false,
    });
    let mean = (0..degree)
        .map(|i| corner_level(&[me, side_of((i + degree - 1) % degree), side_of(i)]))
        .sum::<f32>()
        / degree.max(1) as f32;
    Some(column::layer_altitude(layer) + mean)
}

/// A layer's material in a tier, stone off it.
fn material_in(tier: &ColumnTier, slot: usize, layer: usize) -> Material {
    tier.columns
        .get(slot)
        .map_or(Material::Stone, |c| c.material(layer))
}

fn is_falling(m: Material) -> bool {
    m.flow().is_some_and(|(_, f)| f)
}

/// The pieces of the water cell at `slot`, `layer` in `tier` (which must be
/// water there), into `pieces`. With `out`, the cell is this tick's: it is
/// counted, its arrows drawn and its fall's foot returned (where and how far
/// it fell).
#[allow(clippy::too_many_arguments)]
fn cell_pieces(
    tier: &ColumnTier,
    f: &CellFrame,
    slot: usize,
    layer: usize,
    radius: f32,
    pieces: &mut Vec<Piece>,
    mut out: Option<&mut LiquidMesh>,
) -> Option<FootFound> {
    pieces.clear();
    let sea = sea_layer();
    let material = |slot: usize, layer: usize| material_in(tier, slot, layer);
    let column = &tier.columns[slot];
    let degree = f.degree;
    let (centre, corners, sides) = (f.centre, &f.corners, &f.sides);
    let here = column.material(layer);
    let floor = column::layer_altitude(layer);
    let above_water = column.material(layer + 1).is_water();
    let flow = here.flow();
    let falling = flow.is_some_and(|(_, f)| f);
    // The sea's own top: the sheet draws it wherever it is listed, under a
    // roof too, so a sea cave has one surface at the sheet's tided height.
    let sheet = here == Material::Water && layer + 1 == sea && f.sheet_listed;
    let side_of = |s: usize| -> Option<Side> {
        let n = sides[s]?;
        Some(Side {
            material: material(n, layer),
            above_water: material(n, layer + 1).is_water(),
        })
    };
    let me = Some(Side {
        material: here,
        above_water,
    });
    let mut levels = [0.0f32; 6];
    for (i, level) in levels.iter_mut().enumerate().take(degree) {
        *level = if above_water {
            1.0
        } else {
            corner_level(&[me, side_of((i + degree - 1) % degree), side_of(i)])
        };
    }
    let at = |dir: Vec3, fraction: f32| dir * (radius + floor + fraction);
    let top_of = |p: Vec3| p.normalize_or(centre) * (radius + floor + 1.0);
    if let Some(out) = out.as_deref_mut() {
        out.cells += 1;
    }
    // The top, through the corners, unless water stands on it.
    let mean = levels[..degree].iter().sum::<f32>() / degree.max(1) as f32;
    let middle = at(centre, mean);
    // Luanti's flow, down the corner slope: the least-squares slope of the
    // corners about the middle.
    let mut gradient = Vec3::ZERO;
    let mut spread = 0.0;
    for (i, &k) in corners.iter().enumerate().take(degree) {
        let offset = at(k, mean) - middle;
        let offset = offset - centre * offset.dot(centre);
        gradient += offset * (levels[i] - mean);
        spread += offset.length_squared();
    }
    let slope = if spread > 0.0 {
        -gradient / (spread / 2.0)
    } else {
        Vec3::ZERO
    };
    let flowing = flow.is_some() && slope.length() > 1e-3;
    // The foot of a fall: water that is not falling, with falling water on
    // it. How far it fell is the falling run above. On the landing cell's
    // own surface, so the splash starts at the water, never in it.
    let foot = (!falling && is_falling(column.material(layer + 1))).then(|| {
        let height = (layer + 1..LAYERS)
            .take_while(|&l| is_falling(column.material(l)))
            .count();
        // `D` (`water-flow` 6.6): from the run's lip water, the drawn top of
        // its highest cell (a full block where water stands on it), to the
        // landing's own drawn top.
        let lip = layer + height;
        let lip_top = drawn_top_m(tier, slot, lip).unwrap_or(column::layer_altitude(lip) + 1.0);
        let drop_m = (lip_top - (floor + mean)).max(0.0);
        // The way over: away from the side that feeds the run's top cell, a
        // still or flowing cell beside it that is not itself falling.
        let mut feed: Option<(usize, i32)> = None;
        for (s, side) in sides.iter().enumerate().take(degree) {
            let Some(n) = *side else {
                continue;
            };
            let m = material(n, lip);
            let rank = if m == Material::Water {
                8
            } else if let Some((level, false)) = m.flow() {
                i32::from(level)
            } else {
                continue;
            };
            if feed.is_none_or(|(_, r)| rank > r) {
                feed = Some((s, rank));
            }
        }
        let over = feed.map_or(Vec3::ZERO, |(s, _)| -f.side_normal(s));
        // The pool under the foot: its own water and the water layers under
        // it, down to the first that is not water.
        let under = (1..layer)
            .rev()
            .take_while(|&l| column.material(l).is_water())
            .count();
        FootFound {
            at: middle,
            height: height as f32,
            drop_m,
            over,
            pool_m: mean + under as f32,
        }
    });
    if !above_water && !sheet {
        if let Some(out) = out.as_deref_mut() {
            out.tops += 1;
        }
        // The waves stream along the flow, faster on a steeper top.
        let pan = if flowing {
            let speed = PAN_K * (slope.length() / PAN_SLOPE).clamp(0.35, 1.5);
            let dir = slope.normalize();
            Vec2::new(dir.dot(f.t), dir.dot(f.b)) * speed / SHADER_FLOW
        } else {
            Vec2::ZERO
        };
        // How near each corner is to a fall's lip (owner, 2026-11-15:
        // "can we have the edge transition into froth?"): one at a corner on
        // a side whose water goes over an edge, so the top whitens and speeds
        // up toward the fall across its last metre or so. An edge goes over
        // where the side's water falls, or falls from under it, or this
        // cell's own water falls below.
        let over = |s: usize| -> bool {
            sides[s].is_some_and(|n| {
                is_falling(material(n, layer))
                    || (!tier.columns[n].solid(layer) && is_falling(material(n, layer - 1)))
            })
        };
        let own_lip = is_falling(column.material(layer - 1));
        let mut lips = [0.0f32; 6];
        for (i, lip) in lips.iter_mut().enumerate().take(degree) {
            if own_lip || over((i + degree - 1) % degree) || over(i) {
                *lip = 1.0;
            }
        }
        let middle_lip = if own_lip { 0.6 } else { 0.0 };
        // Each triangle lit by its own slope, so a top running down to a
        // sheet's edge reads as water tilted, not a mirror on edge.
        let mut top = Piece::new(PieceId::Top);
        for i in 0..degree {
            let (p, q) = (
                at(corners[i], levels[i]),
                at(corners[(i + 1) % degree], levels[(i + 1) % degree]),
            );
            let mut n = (p - middle).cross(q - middle).normalize_or(centre);
            if n.dot(centre) < 0.0 {
                n = -n;
            }
            let j = (i + 1) % degree;
            for (v, lip) in [(middle, middle_lip), (p, lips[i]), (q, lips[j])] {
                top.push(
                    PieceVertex {
                        p: v,
                        n,
                        flow: pan.extend(lip),
                        kind: KIND_TOP,
                    },
                    top_of(v),
                );
            }
        }
        pieces.push(top);
        if flowing && let Some(out) = out.as_deref_mut() {
            out.flowing_tops += 1;
            let along = slope.normalize();
            let across = centre.cross(along);
            let length = 0.8 + 0.8 * (slope.length() / PAN_SLOPE).min(1.0);
            out.arrow(middle + centre * ARROW_LIFT, along, across, length);
        }
    }
    // The sides: Luanti's `drawLiquidSides`.
    for s in 0..degree {
        let Some(n) = sides[s] else {
            continue;
        };
        let there = material(n, layer);
        let there_above = material(n, layer + 1).is_water();
        let (a, z) = (corners[s], corners[(s + 1) % degree]);
        let (top_a, top_z) = (levels[s], levels[(s + 1) % degree]);
        let (bottom_a, bottom_z) = if there.is_water() {
            // Only a fall's side over lower water, down to it.
            if !above_water || there_above {
                continue;
            }
            (top_a, top_z)
        } else if tier.columns[n].solid(layer) {
            continue;
        } else {
            (0.0, 0.0)
        };
        let (top_a, top_z) = if above_water {
            (1.0, 1.0)
        } else {
            (top_a, top_z)
        };
        if sheet && layer + 1 == sea {
            // The sea's side at its surface is the sheet's lip.
            continue;
        }
        let normal = f.side_normal(s);
        let edge = (z - a).normalize_or(f.t);
        // A falling cell's face is white water streaming down; a sheet's
        // edge and a source's face stand still. A fall's froth is densest at
        // its lip and its foot.
        let froth = if !falling {
            0.5
        } else if !is_falling(column.material(layer + 1)) || !is_falling(column.material(layer - 1))
        {
            1.0
        } else {
            0.55
        };
        let kind = if falling || above_water && flow.is_some() {
            KIND_FALLING
        } else {
            KIND_STILL
        };
        // How far under its fall's lip each corner is, metres: the falling
        // run over this layer, and the fraction into it.
        // A foot cell's face under a fall (`KIND_FALLING`, not itself
        // falling) is the bottom of that fall: its metres run on from the
        // falling run over it (`water-flow` 6.6), so the Glassy lip and the
        // speed-up read it as one face with the run.
        let measured = falling || kind == KIND_FALLING;
        let run = if measured {
            (layer + 1..LAYERS)
                .take_while(|&l| is_falling(column.material(l)))
                .count() as f32
        } else {
            0.0
        };
        // On the run's top cell the lip is the face's own top edge, at each
        // corner's level (a falling cell's top slopes down to its low
        // corners), so the metres start there; under it, from the top of
        // the cell.
        let under = |fraction: f32, top: f32| {
            if !measured {
                0.0
            } else if run == 0.0 {
                (top - fraction).max(0.0)
            } else {
                run + 1.0 - fraction
            }
        };
        let tops = [top_a, top_z, top_z, top_a];
        let pts = [
            (at(a, bottom_a), bottom_a),
            (at(z, bottom_z), bottom_z),
            (at(z, top_z), top_z),
            (at(a, top_a), top_a),
        ];
        let mut piece = Piece::new(PieceId::Side(s));
        // Two triangles, and the side collapsed: its bottom raised to its top.
        for (k, raised) in [(0, 3), (1, 2), (2, 2), (0, 3), (2, 2), (3, 3)] {
            let (p, fraction) = pts[k];
            piece.push(
                PieceVertex {
                    p,
                    n: normal,
                    flow: Vec3::new(froth, 0.0, under(fraction, tops[k])),
                    kind,
                },
                pts[raised].0,
            );
        }
        pieces.push(piece);
        if falling
            && !there.is_water()
            && let Some(out) = out.as_deref_mut()
        {
            let mid = (pts[0].0 + pts[1].0 + pts[2].0 + pts[3].0) * 0.25;
            out.arrow(mid + normal * ARROW_LIFT, -centre, edge, 0.7);
        }
    }
    // The bottom, over air.
    if column.material(layer - 1) == Material::Air {
        let mut bottom = Piece::new(PieceId::Bottom);
        let c = at(centre, 0.0);
        for i in 0..degree {
            for p in [c, at(corners[(i + 1) % degree], 0.0), at(corners[i], 0.0)] {
                bottom.push(
                    PieceVertex {
                        p,
                        n: -centre,
                        flow: Vec3::ZERO,
                        kind: KIND_STILL,
                    },
                    p,
                );
            }
        }
        pieces.push(bottom);
    }
    foot
}

/// Where a cell that appears (S2, S3) starts, or one that dries (S4) ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Squash {
    /// Pressed up to its top: a fall arriving from the water over it.
    Top,
    /// Pressed down to its floor: a fall's tail draining into the water
    /// under it.
    Floor,
    /// A thin sheet (`THIN` of its height) on side `s`: the side its water
    /// came from or went to.
    Side(usize),
}

impl Squash {
    /// Where `p`, a point of the cell at `floor` metres, starts or ends.
    fn apply(self, f: &CellFrame, radius: f32, floor: f32, p: Vec3) -> Vec3 {
        let base = radius + floor;
        let dir = p.normalize_or(f.centre);
        match self {
            Squash::Top => dir * (base + 1.0),
            Squash::Floor => dir * base,
            Squash::Side(s) => {
                let n = f.side_normal(s);
                let edge = f.corners[s];
                // At or under zero inside the cell: how far in from the side.
                let d = (dir - edge).dot(n);
                let onto = (dir - n * d).normalize_or(dir);
                let height = p.length() - base;
                onto * (base + height * THIN)
            }
        }
    }
}

/// The side of `slot` whose water at `layer` stands highest in `tier` (a
/// source counts 8, a flowing cell its level), or `None` where no side is
/// wet: where a newly wet cell's water came from, or a dried one's went.
fn wettest_side(tier: &ColumnTier, f: &CellFrame, layer: usize) -> Option<usize> {
    let mut best = None;
    let mut rank = -1i32;
    for s in 0..f.degree {
        let Some(n) = f.sides[s] else {
            continue;
        };
        let m = material_in(tier, n, layer);
        let r = if m == Material::Water {
            8
        } else if let Some((level, _)) = m.flow() {
            i32::from(level)
        } else {
            -1
        };
        if r > rank {
            rank = r;
            best = Some(s);
        }
    }
    best
}

/// Whether nothing the cell's pieces are shaped by changed between two
/// ticks: its own column round the layer, and its sides' at, over and under
/// it. Such a cell is drawn still, without meshing it twice.
fn same_shape(
    prev: &ColumnTier,
    tier: &ColumnTier,
    f: &CellFrame,
    slot: usize,
    layer: usize,
) -> bool {
    let same = |s: usize| {
        (layer - 1..=layer + 1).all(|l| material_in(prev, s, l) == material_in(tier, s, l))
    };
    same(slot) && f.sides[..f.degree].iter().flatten().all(|&n| same(n))
}

/// Whether the cell at `slot`, `layer` of `tier` is a fall's foot.
fn is_foot(tier: &ColumnTier, slot: usize, layer: usize) -> bool {
    let m = material_in(tier, slot, layer);
    m.is_water() && !is_falling(m) && is_falling(material_in(tier, slot, layer + 1))
}

/// Mesh every water cell of the tier the sea sheet does not draw, as it is.
pub fn liquid_mesh(tier: &ColumnTier, finest: &[GpuCell], radius: f32) -> LiquidMesh {
    liquid_mesh_eased(tier, None, finest, radius)
}

/// Mesh every water cell of the tier the sea sheet does not draw, each
/// vertex easing from `prev`, the tier the tick before (same slots), when
/// there is one (M31's `pourMesh`):
/// - S1, a cell wet in both: its pieces ease from the last tick's to this
///   one's; a piece it gained grows from its collapsed self, one it lost
///   collapses away and goes;
/// - S2, wet now and not before: squashed onto the side its water came from;
/// - S3, the same under last tick's water: pressed up to its top, so a fall
///   grows down;
/// - S4, wet before and not now: drawn this one tick more, shrinking onto the
///   side its water went to (or down into the water under it), then gone.
pub fn liquid_mesh_eased(
    tier: &ColumnTier,
    prev: Option<&ColumnTier>,
    finest: &[GpuCell],
    radius: f32,
) -> LiquidMesh {
    let mut out = LiquidMesh::default();
    // The tick before is eased from only when it is the same tier.
    let prev = prev
        .filter(|p| p.columns.len() == tier.columns.len() && p.water.records == tier.water.records);
    let mut now = Vec::with_capacity(8);
    let mut was = Vec::with_capacity(8);
    let mut to = [Vec3::ZERO; PIECE_MAX];
    // The columns the tick changed: only cells in or beside one can move.
    let changed: Vec<bool> = prev.map_or_else(Vec::new, |p| {
        p.columns
            .iter()
            .zip(&tier.columns)
            .map(|(a, b)| a != b)
            .collect()
    });
    for slot in 0..tier.columns.len() {
        let Some(&record) = tier.water.records.get(slot) else {
            continue;
        };
        let Some(cell) = finest.get(record) else {
            continue;
        };
        let column = &tier.columns[slot];
        // An unchanged column was wet where it is wet now.
        let before = prev.filter(|_| changed[slot]).map(|p| &p.columns[slot]);
        // The cell's shape, and the tick to ease across when the cell or a
        // side of it changed in it; made at its first wet layer.
        let mut frame: Option<(CellFrame, Option<&ColumnTier>)> = None;
        for layer in 1..LAYERS - 1 {
            let wet_now = column.material(layer).is_water();
            let wet_was = before.map_or(wet_now, |b| b.material(layer).is_water());
            if !wet_now && !wet_was {
                continue;
            }
            let (f, prev) = &*frame.get_or_insert_with(|| {
                let f = CellFrame::of(tier, slot, cell);
                let near = prev.is_some()
                    && (changed[slot] || f.sides[..f.degree].iter().flatten().any(|&n| changed[n]));
                (f, prev.filter(|_| near))
            });
            let prev = *prev;
            let lit = {
                let own = tier.light_at(slot, layer);
                let up = tier.light_at(slot, layer + 1);
                f32::from(own.sky().max(up.sky())) / 15.0
            };
            let floor = column::layer_altitude(layer);
            match prev {
                Some(p) if wet_was && wet_now && !same_shape(p, tier, f, slot, layer) => {
                    // S1: the same cell in both ticks; its pieces ease.
                    cell_pieces(p, f, slot, layer, radius, &mut was, None);
                    let foot = cell_pieces(tier, f, slot, layer, radius, &mut now, Some(&mut out));
                    let mut used = [false; 8];
                    for c in &now {
                        let matched = was
                            .iter()
                            .position(|w| w.id == c.id && w.n == c.n)
                            .filter(|&i| !used[i]);
                        let positions = c.positions();
                        match matched {
                            Some(i) => {
                                used[i] = true;
                                let from = was[i].positions();
                                out.emit(c, &positions, &from, lit, 0);
                            }
                            // A piece new this tick grows from its collapsed self.
                            None => out.emit(c, &positions, &c.alt[..c.n], lit, EASE_FRONT),
                        }
                    }
                    // And one gone this tick collapses away.
                    for (i, w) in was.iter().enumerate() {
                        if !used[i] {
                            let from = w.positions();
                            out.emit(w, &w.alt[..w.n], &from, lit, EASE_FRONT | EASE_GONE);
                        }
                    }
                    if let Some(found) = foot {
                        out.feet
                            .push(found.foot(lit, !is_foot(p, slot, layer), slot, layer));
                    }
                }
                Some(p) if wet_now && !wet_was => {
                    // S2 and S3: newly wet.
                    out.born += 1;
                    let how = if material_in(p, slot, layer + 1).is_water() {
                        Squash::Top
                    } else {
                        wettest_side(p, f, layer).map_or(Squash::Top, Squash::Side)
                    };
                    let foot = cell_pieces(tier, f, slot, layer, radius, &mut now, Some(&mut out));
                    for c in &now {
                        let positions = c.positions();
                        for (t, &p) in to.iter_mut().zip(&positions) {
                            *t = how.apply(f, radius, floor, p);
                        }
                        out.emit(c, &positions, &to[..c.n], lit, EASE_FRONT);
                    }
                    if let Some(found) = foot {
                        out.feet.push(found.foot(lit, true, slot, layer));
                    }
                }
                Some(p) if wet_was && !wet_now => {
                    // S4: dried; drawn this one tick more, shrinking back.
                    out.dried += 1;
                    let how = if material_in(tier, slot, layer - 1).is_water() {
                        Squash::Floor
                    } else {
                        wettest_side(tier, f, layer).map_or(Squash::Floor, Squash::Side)
                    };
                    cell_pieces(p, f, slot, layer, radius, &mut was, None);
                    for w in &was {
                        let from = w.positions();
                        for (t, &p) in to.iter_mut().zip(&from) {
                            *t = how.apply(f, radius, floor, p);
                        }
                        out.emit(w, &to[..w.n], &from, lit, EASE_FRONT | EASE_GONE);
                    }
                }
                _ => {
                    // As it is: no tick to ease across, or nothing round the
                    // cell changed in it.
                    let foot = cell_pieces(tier, f, slot, layer, radius, &mut now, Some(&mut out));
                    for c in &now {
                        out.emit_still(c, lit);
                    }
                    if let Some(found) = foot {
                        out.feet.push(found.foot(lit, false, slot, layer));
                    }
                }
            }
        }
    }
    out
}

/// Mesh the tier's water again when the fine set changes: easing from the
/// set before when the change is the tick that set made.
fn rebuild_water(
    fine: Option<Res<PlanetFine>>,
    mut meshed: ResMut<Meshed>,
    mut step: ResMut<WaterStep>,
    mut flowing: ResMut<FlowingWater>,
    mut feet: ResMut<FallFeet>,
    mut meshes: ResMut<Assets<Mesh>>,
    arrows: Query<&Mesh3d, With<ArrowMesh>>,
) {
    let Some(fine) = fine else {
        return;
    };
    let set = Arc::as_ptr(&fine.set) as usize;
    if meshed.version == fine.version && meshed.set == set {
        return;
    }
    meshed.version = fine.version;
    meshed.set = set;
    let started = std::time::Instant::now();
    let prev = step
        .prev
        .take()
        .filter(|_| step.after == Some((fine.version, set)));
    let made = liquid_mesh_eased(
        &fine.set.columns,
        prev.as_deref().map(|p| &p.columns),
        fine.set.finest_records(),
        PLANET_RADIUS,
    );
    if made.cells > 0 {
        debug!(
            "water mesh: {} cells ({} new, {} dried), {} tops ({} flowing), {} triangles in {:.2} ms",
            made.cells,
            made.born,
            made.dried,
            made.tops,
            made.flowing_tops,
            made.vertices.len() / 3,
            started.elapsed().as_secs_f64() * 1e3
        );
    }
    flowing.lanes = Arc::new(made.lanes());
    flowing.version += 1;
    feet.0 = made.feet.clone();
    if let Ok(handle) = arrows.single()
        && let Some(mesh) = meshes.get_mut(&handle.0)
    {
        if made.arrow_indices.is_empty() {
            *mesh = placeholder();
        } else {
            let normals = vec![[0.0f32, 1.0, 0.0]; made.arrow_positions.len()];
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, made.arrow_positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
            mesh.insert_indices(Indices::U32(made.arrow_indices));
        }
    }
}

/// The arrows on the planet as its frame moves, shown or not.
fn show_arrows(
    frame: Option<Res<PlanetRenderFrame>>,
    arrows_on: Res<FlowArrows>,
    mut arrows: Query<(&mut Transform, &mut Visibility), With<ArrowMesh>>,
) {
    let Some(frame) = frame else {
        return;
    };
    let centre = frame.center.as_vec3();
    let want = if arrows_on.shown() {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    for (mut transform, mut visibility) in &mut arrows {
        if transform.translation != centre {
            transform.translation = centre;
        }
        if *visibility != want {
            *visibility = want;
        }
    }
}

/// The water's systems.
pub struct WaterFlowPlugin;

impl Plugin for WaterFlowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaterClock>()
            .init_resource::<FlowArrows>()
            .init_resource::<Meshed>()
            .init_resource::<FlowingWater>()
            .init_resource::<WaterStep>()
            .init_resource::<WaterEase>()
            .add_plugins(bevy::render::extract_resource::ExtractResourcePlugin::<
                FlowingWater,
            >::default())
            .add_plugins(bevy::render::extract_resource::ExtractResourcePlugin::<
                WaterEase,
            >::default())
            .init_resource::<FallFeet>()
            .init_resource::<SplashQuads>()
            .init_resource::<FallFeetGpu>()
            .add_plugins(bevy::render::extract_resource::ExtractResourcePlugin::<
                FallFeetGpu,
            >::default())
            .add_plugins(bevy::render::extract_resource::ExtractResourcePlugin::<
                SplashQuads,
            >::default())
            .add_systems(Startup, spawn_arrows)
            .add_systems(
                Update,
                (
                    tick_water,
                    rebuild_water,
                    ease_water,
                    show_arrows,
                    animate_splash,
                )
                    .chain(),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn side(material: Material, above_water: bool) -> Option<Side> {
        Some(Side {
            material,
            above_water,
        })
    }

    /// Luanti's corner rule on three cells.
    #[test]
    fn a_corner_is_full_by_a_source_low_by_air_and_the_mean_between() {
        let f = |l| Material::flowing(l, false);
        assert_eq!(
            corner_level(&[side(f(3), false), side(Material::Water, false), None]),
            1.0
        );
        assert_eq!(
            corner_level(&[side(f(3), false), side(Material::Stone, true), None]),
            1.0,
            "water standing on a neighbour fills the corner"
        );
        assert_eq!(
            corner_level(&[
                side(f(7), false),
                side(Material::Air, false),
                side(Material::Air, false)
            ]),
            0.02,
            "two of three air: down to the floor"
        );
        let mean = corner_level(&[
            side(f(7), false),
            side(f(5), false),
            side(Material::Air, false),
        ]);
        assert!((mean - (7.5 + 5.5) / 16.0).abs() < 1e-6);
        assert_eq!(
            corner_level(&[
                side(Material::Stone, false),
                side(Material::Stone, false),
                None
            ]),
            0.5
        );
    }

    // ---- Smooth water between ticks (M31's Pour tab, `water-flow` 5.3)

    /// The ground's top, layers.
    const LAND: usize = 150;
    /// The water's layer, on the ground.
    const AT: usize = LAND + 1;
    const RADIUS: f32 = 1000.0;
    /// Centre to centre, m: the gold standard's tile width.
    const SPACING: f32 = 2.833;

    /// Slot 0 and the six round it, slot `k + 1` across slot 0's side `k`.
    /// Side `s` of a cell spans its corners `s` and `s + 1`, corner `i`
    /// at `60 i` degrees round it.
    fn ring_sides() -> Vec<[u32; 6]> {
        use crate::planet::column::NO_NEIGHBOR;
        let mut sides = vec![[1, 2, 3, 4, 5, 6]];
        for k in 0..6usize {
            let mut s = [NO_NEIGHBOR; 6];
            s[(k + 3) % 6] = 0;
            s[(k + 2) % 6] = ((k + 1) % 6 + 1) as u32;
            s[(k + 4) % 6] = ((k + 5) % 6 + 1) as u32;
            sides.push(s);
        }
        sides
    }

    /// The seven cells' records: hexagons on a sphere of `RADIUS`, over
    /// the sea.
    fn ring_cells() -> Vec<GpuCell> {
        let flat = |x: f32, z: f32| Vec3::new(x, RADIUS, z);
        let centre_of = |slot: usize| {
            if slot == 0 {
                flat(0.0, 0.0)
            } else {
                let a = (60.0 * (slot - 1) as f32 + 30.0).to_radians();
                flat(SPACING * a.cos(), SPACING * a.sin())
            }
        };
        (0..7)
            .map(|slot| {
                let c = centre_of(slot);
                let mut cell: GpuCell = bytemuck::Zeroable::zeroed();
                let d = c.normalize();
                cell.direction_height = [d.x, d.y, d.z, 1000.0];
                for i in 0..6 {
                    let a = (60.0 * i as f32).to_radians();
                    let k =
                        (c + Vec3::new(a.cos(), 0.0, a.sin()) * SPACING / 3f32.sqrt()).normalize();
                    cell.corners[i] = [k.x, k.y, k.z, 0.0];
                }
                cell.metadata[0] = 6;
                cell
            })
            .collect()
    }

    /// The ring on ground at `LAND`, with water where `wet` says.
    fn ring(wet: &[(usize, usize, Material)]) -> ColumnTier {
        let columns = (0..7)
            .map(|slot| {
                let mut column = pbd_core::column::Column::bedrock();
                for layer in 1..=LAND {
                    column.set(layer, Material::Stone);
                }
                for &(s, layer, m) in wet {
                    if s == slot {
                        column.set(layer, m);
                    }
                }
                column
            })
            .collect();
        ColumnTier::for_tests(columns, &ring_sides())
    }

    fn mesh(now: &ColumnTier, before: Option<&ColumnTier>) -> LiquidMesh {
        liquid_mesh_eased(now, before, &ring_cells(), RADIUS)
    }

    /// A fall's foot carries what the proposed mist rule and the pool's
    /// terms need (`water-flow` 6.6): its drop `D` in metres from the lip's
    /// water, the way over (away from the side that feeds it), the pool's
    /// depth and its column; and a foot cell's face under the fall reads
    /// metres under the lip on from the run over it.
    #[test]
    fn a_fall_s_foot_knows_its_drop_its_way_over_and_its_pool() {
        // Slot 0 a ledge three layers up with a source on it; slot 1 falls
        // three layers beside it into a pool on the ground.
        let falling = Material::flowing(7, true);
        let tier = ring(&[
            (0, AT, Material::Stone),
            (0, AT + 1, Material::Stone),
            (0, AT + 2, Material::Stone),
            (0, AT + 3, Material::Water),
            (1, AT + 3, falling),
            (1, AT + 2, falling),
            (1, AT + 1, falling),
            (1, AT, Material::Water),
        ]);
        let made = mesh(&tier, None);
        let foot = made
            .feet
            .iter()
            .find(|f| f.column == 1)
            .expect("the fall has a foot");
        assert_eq!(foot.height, 3.0);
        assert!(foot.drop_m > 2.5 && foot.drop_m < 4.0, "{}", foot.drop_m);
        assert!(foot.pool_m > 0.0 && foot.pool_m <= 1.0, "{}", foot.pool_m);
        let cells = ring_cells();
        let f = CellFrame::of(&tier, 1, &cells[1]);
        assert_eq!(f.sides[3], Some(0));
        assert!(foot.over.dot(-f.side_normal(3)) > 0.99, "{}", foot.over);
        // Every falling face's metres under the lip run from 0 to the run's
        // height, never negative, and not all zero.
        let unders: Vec<f32> = made
            .vertices
            .iter()
            .filter(|v| v.kind == KIND_FALLING)
            .map(|v| v.flow.z)
            .collect();
        assert!(!unders.is_empty());
        assert!(unders.iter().all(|u| (0.0..=4.0).contains(u)), "{unders:?}");
        assert!(unders.iter().any(|u| *u > 2.0), "{unders:?}");
        // The run's top cell starts at its own top edge, every corner of it,
        // however its top slopes: a face's top vertices read 0 there.
        let lip_top = made
            .vertices
            .iter()
            .filter(|v| v.kind == KIND_FALLING && v.flow.z < 1e-4)
            .count();
        assert!(lip_top >= 6, "{unders:?}");
        // Two feet in one column are two keys.
        let mut stacked = *foot;
        stacked.layer += 3;
        assert_ne!(stacked.key(), foot.key());
    }

    /// S2: a cell that turns wet starts as a thin sheet on the side its
    /// water came from: every vertex it eases from lies on that side's edge,
    /// within `THIN` of the cell's floor.
    #[test]
    fn a_new_wet_cell_flows_in_from_its_source_edge() {
        let before = ring(&[(0, AT, Material::Water)]);
        let now = ring(&[
            (0, AT, Material::Water),
            (1, AT, Material::flowing(7, false)),
        ]);
        let made = mesh(&now, Some(&before));
        assert_eq!((made.born, made.dried), (1, 0));
        let cells = ring_cells();
        let f = CellFrame::of(&now, 1, &cells[1]);
        // Slot 1's side toward slot 0 is its side 3.
        assert_eq!(f.sides[3], Some(0));
        let (n, edge) = (f.side_normal(3), f.corners[3]);
        let base = RADIUS + column::layer_altitude(AT);
        let new: Vec<&FlowVertex> = made
            .vertices
            .iter()
            .filter(|v| v.flags == EASE_FRONT)
            .collect();
        assert!(!new.is_empty(), "the new cell is drawn");
        let mut moved = 0;
        for v in &new {
            let dir = v.prev.normalize();
            let off = (dir - edge).dot(n) * RADIUS;
            assert!(off.abs() < 1e-2, "{off} m off the source edge");
            let height = v.prev.length() - base;
            assert!(
                (-1e-3..=THIN + 1e-3).contains(&height),
                "{height} m over the floor"
            );
            assert!(v.position.is_finite() && v.prev.is_finite());
            if v.position.distance(v.prev) > 0.1 {
                moved += 1;
            }
        }
        assert!(moved > 0, "the front sweeps across the cell");
        // The pass reads four lanes a vertex.
        assert_eq!(made.lanes().len(), made.vertices.len() * LANES_PER_VERTEX);
    }

    /// S3: a cell that turns wet under last tick's water grows down from
    /// its top: it starts pressed to the top of its layer.
    #[test]
    fn a_new_cell_under_water_grows_down_from_its_top() {
        let falling = Material::flowing(7, true);
        let before = ring(&[(0, AT + 1, Material::Water), (1, AT + 1, falling)]);
        let now = ring(&[
            (0, AT + 1, Material::Water),
            (1, AT + 1, falling),
            (1, AT, Material::flowing(7, false)),
        ]);
        let made = mesh(&now, Some(&before));
        assert_eq!(made.born, 1);
        let top = RADIUS + column::layer_altitude(AT) + 1.0;
        let grown: Vec<&FlowVertex> = made
            .vertices
            .iter()
            .filter(|v| v.flags == EASE_FRONT && v.position.length() < top - 0.5)
            .collect();
        assert!(!grown.is_empty());
        for v in grown {
            assert!((v.prev.length() - top).abs() < 1e-3, "starts at the top");
        }
    }

    /// S4: a dried cell is drawn for exactly one tick more, shrinking onto
    /// the side its water went to, and is gone from the mesh after.
    #[test]
    fn a_dried_cell_is_kept_exactly_one_tick() {
        let wet = ring(&[
            (0, AT, Material::Water),
            (1, AT, Material::flowing(7, false)),
        ]);
        let dry = ring(&[(0, AT, Material::Water)]);
        let made = mesh(&dry, Some(&wet));
        assert_eq!((made.born, made.dried), (0, 1));
        let gone: Vec<&FlowVertex> = made
            .vertices
            .iter()
            .filter(|v| v.flags & EASE_GONE != 0)
            .collect();
        assert!(!gone.is_empty(), "the dried cell is drawn this tick");
        let f = CellFrame::of(&dry, 1, &ring_cells()[1]);
        let base = RADIUS + column::layer_altitude(AT);
        let mut on_edge = 0;
        for v in &gone {
            assert!(v.flags & EASE_FRONT != 0, "a front: eased linearly");
            let off = (v.position.normalize() - f.corners[3]).dot(f.side_normal(3)) * RADIUS;
            if off.abs() < 1e-2 && v.position.length() - base <= THIN + 1e-3 {
                on_edge += 1;
            }
        }
        // The slot's own pieces end on its edge toward the source; the
        // source's side that the dried cell had covered goes the same way.
        assert!(on_edge > 0);
        // The tick after: nothing of it is left.
        let next = mesh(&dry, Some(&dry));
        assert_eq!(next.dried, 0);
        assert!(next.vertices.iter().all(|v| v.flags & EASE_GONE == 0));
        assert_eq!(next.vertices.len(), mesh(&dry, None).vertices.len());
    }

    /// Water that did not change, or no tick to ease across: every vertex
    /// stands where it is, as today's mesher drew it.
    #[test]
    fn still_water_and_the_pops_path_do_not_move() {
        let pool = ring(&[
            (0, AT, Material::Water),
            (1, AT, Material::flowing(7, false)),
        ]);
        for made in [mesh(&pool, Some(&pool)), mesh(&pool, None)] {
            assert!(!made.vertices.is_empty());
            assert!(
                made.vertices
                    .iter()
                    .all(|v| v.prev == v.position && v.flags == 0)
            );
        }
        assert_eq!(
            mesh(&pool, Some(&pool)).vertices,
            mesh(&pool, None).vertices
        );
    }
}

// ---- The splash at a fall's foot (owner, 2026-11-15: "can you also add some
// particle fx to bottom of the waterfall? more fx based on distance it fell")
// Since `water-flow` 6.6 it is mockup M35's New tab (`docs/mockups/waterfall-
// mist/mist.js`, `splashQuads`), approved 2026-11-16: the proposed mist rule,
// the puff sheet, fall droplets and a reach faded in time.

/// Every fall's foot in the tier, as the mesher found them: where, how many
/// layers it fell, the sky's share there, and whether it landed this tick.
#[derive(Resource, Default)]
pub struct FallFeet(pub Vec<Foot>);

/// The splash's quads as the water pass draws them (`water-flow` 6.1, 6.6):
/// drawn after the water, against its depth, so a splash over a pool stands
/// on the pool's surface rather than under it. `lanes[0]` is the camera's
/// right and `lanes[1]` its up the quads face along, then `SPLASH_LANES` a
/// quad: its centre (body-local) and half size; its colour (rgb the light on
/// it, a its alpha); and its cell and flip (`SPLASH_DOT`, `SPLASH_DROPLET`, or
/// a puff's continuous frame on the sheet, row x 4 + age x 4) and the radius
/// of its pool's surface, under which none of it is drawn. Empty when
/// nothing splashes.
#[derive(Resource, Clone, Default, bevy::render::extract_resource::ExtractResource)]
pub struct SplashQuads {
    pub lanes: Vec<[f32; 4]>,
}

/// Lanes (vec4s) a splash quad: centre and size, colour, cell and flip.
/// `splash_vertex`'s stride (`water.wgsl`); a test holds the two together.
pub const SPLASH_LANES: usize = 3;
/// The cell of a spray drop: the soft round dot.
pub const SPLASH_DOT: f32 = -1.0;
/// The cell of a fall droplet: a small hard drop.
pub const SPLASH_DROPLET: f32 = -2.0;

/// The most feet the water's fragment and the bubbles are told of, nearest
/// first (`water-flow` 6.6): `feet` in `water.wgsl`'s `WaterView`, and the
/// bound on every loop over them.
pub const FALL_FEET_MAX: usize = 8;

/// The feet the water pass is told of (`water-flow` 6.6): per foot, xyz where
/// it lands (body-local) and w its drop `D`, m; then x the pool's depth under
/// it, m, y its seed 0..1, z its fade 0..1 (in time, as it joins or leaves the
/// nearest eight); and how many there are.
#[derive(Resource, Clone, Default, bevy::render::extract_resource::ExtractResource)]
pub struct FallFeetGpu {
    pub feet: [Vec4; FALL_FEET_MAX],
    pub more: [Vec4; FALL_FEET_MAX],
    pub count: u32,
}

/// How far out from a fall's foot the splash starts: the falling column's
/// half width (half the 2.833 m cell, flat to flat), so nothing is drawn
/// inside the falling water (owner, 2026-11-15: "make sure the water splash
/// fx doesnt end up inside the water but only outside").
const SPLASH_EDGE_M: f32 = 1.42;
/// The least height over the pool's surface a splash particle keeps.
const SPLASH_CLEAR_M: f32 = 0.05;

/// The most quads the splash draws in a frame, drops and mist together.
pub const SPLASH_BUDGET: usize = 4000;
/// The splash's own gravity, m/s^2.
const SPLASH_G: f32 = 15.0;

fn hash01(a: u32, b: u32) -> f32 {
    let mut h = a.wrapping_mul(0x9E37_79B9) ^ b.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h & 0xFFFF) as f32 / 65535.0
}

/// How much splash a fall makes, by the mist rule (`RULES` in `mist.js`).
#[derive(Clone, Copy, Debug, PartialEq)]
struct SplashRule {
    /// The measure the sizes run on: `D` m (proposed) or whole layers.
    m: f32,
    puffs: u32,
    alpha: f32,
    size: f32,
    rise: f32,
    drops: u32,
    droplets: f32,
    /// The puffs rise on the fall's open side only.
    open: bool,
}

impl SplashRule {
    fn of(rule: crate::config::MistRule, height: f32, drop_m: f32) -> Self {
        match rule {
            crate::config::MistRule::Proposed => {
                let d = drop_m.max(0.0);
                Self {
                    m: d,
                    puffs: (7.0 + 0.75 * d).round().clamp(8.0, 24.0) as u32,
                    alpha: 0.45,
                    size: (0.25 + 0.19 * d).clamp(0.35, 5.0),
                    rise: 0.15 + 0.05 * d,
                    drops: (3.0 + 6.0 * d).clamp(12.0, 160.0) as u32,
                    droplets: (9.0 * d).clamp(6.0, 160.0),
                    open: true,
                }
            }
            crate::config::MistRule::Layers => {
                let h = height.max(1.0);
                Self {
                    m: h,
                    puffs: ((h - 1.0) * 1.5).clamp(1.0, 24.0) as u32,
                    alpha: 0.12 + 0.33 * (h / 8.0).min(1.0),
                    size: (0.5 + 0.18 * h).min(5.0),
                    rise: 0.25 + 0.04 * h,
                    drops: (3.0 + 6.0 * h).min(160.0) as u32,
                    droplets: (9.0 * h).min(160.0),
                    open: false,
                }
            }
        }
    }
}

/// A foot's fades across frames, in time (CLAUDE.md, "a fade is a moment in
/// time"): the splash's, as it crosses the reach, and the water's, as it
/// joins or leaves the nearest `FALL_FEET_MAX`.
#[derive(Clone, Copy, Default)]
struct FootFade {
    splash: f32,
    near: f32,
    seen: u32,
}

/// What the splash keeps between frames: each foot's fades, keyed by its
/// column and layer (a lookup only; nothing is ordered by it), and a scratch list kept
/// so no frame allocates.
#[derive(Default)]
struct SplashState {
    fades: std::collections::HashMap<u64, FootFade>,
    frame: u32,
    order: Vec<(f32, usize)>,
}

/// Step `v` toward `want` (0 or 1) by `dt` over `fade_s`.
fn fade_toward(v: f32, want: f32, dt: f32, fade_s: f32) -> f32 {
    let step = (dt / fade_s.max(1e-3)).min(1.0);
    if want > v {
        (v + step).min(want)
    } else {
        (v - step).max(want)
    }
}

/// Spray, mist and fall droplets at every fall's foot near the eye, and the
/// feet the water pass is told of. A pure function of the time and the
/// fades, so nothing else is stored.
#[allow(clippy::too_many_arguments)]
fn animate_splash(
    time: Res<Time>,
    sun: Option<Res<crate::sky::Sun>>,
    frame: Option<Res<PlanetRenderFrame>>,
    feet: Res<FallFeet>,
    ease: Res<WaterEase>,
    water: Option<Res<crate::config::WaterSettings>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    mut splash: ResMut<SplashQuads>,
    mut gpu_feet: ResMut<FallFeetGpu>,
    mut state: Local<SplashState>,
) {
    let defaults = crate::config::FallLook::default();
    let look = water.as_deref().map_or(&defaults, |w| &w.falls);
    let Some(frame) = frame else {
        splash.lanes.clear();
        gpu_feet.count = 0;
        return;
    };
    let centre = frame.center.as_vec3();
    let Some((camera, _)) = cameras.iter().find(|(_, c)| c.is_active) else {
        splash.lanes.clear();
        gpu_feet.count = 0;
        return;
    };
    let eye = camera.translation() - centre;
    let right = camera.right().as_vec3();
    let up_cam = camera.up().as_vec3();
    let day = sun.as_ref().map_or(1.0, |s| {
        let e = s.direction().dot(eye.normalize_or(Vec3::Y));
        ((e + 0.12) / 0.32).clamp(0.0, 1.0)
    });
    let t = time.elapsed_secs_wrapped();
    let dt = time.delta_secs().clamp(0.0, 0.1);
    // A fall that lands this tick splashes when its column reaches the
    // foot, at the end of the tick, not when the tick starts (S3).
    let landed = ((ease.linear - 0.95) / 0.05).clamp(0.0, 1.0);
    let state = &mut *state;
    state.frame = state.frame.wrapping_add(1);
    let stamp = state.frame;
    // Every foot, nearest first.
    state.order.clear();
    for (i, f) in feet.0.iter().enumerate() {
        let distance = f.at.distance(eye);
        if distance.is_finite() {
            state.order.push((distance, i));
        }
    }
    state.order.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut near_rank = 0usize;
    gpu_feet.count = 0;
    for &(distance, i) in &state.order {
        let f = &feet.0[i];
        let fade = state.fades.entry(f.key()).or_default();
        let inside = distance < look.splash_reach_m;
        let want_near = inside && near_rank < FALL_FEET_MAX;
        if inside {
            near_rank += 1;
        }
        // Once a frame a foot: a key met twice in one frame is not stepped
        // twice (nor reset by its own second sight).
        if fade.seen != stamp {
            // A foot not seen last frame starts from nothing and fades in.
            if fade.seen != stamp.wrapping_sub(1) {
                fade.splash = 0.0;
                fade.near = 0.0;
            }
            fade.splash = fade_toward(
                fade.splash,
                f32::from(u8::from(inside)),
                dt,
                look.splash_fade_s,
            );
            fade.near = fade_toward(
                fade.near,
                f32::from(u8::from(want_near)),
                dt,
                look.splash_fade_s,
            );
            fade.seen = stamp;
        }
        if fade.near > 0.0 && (gpu_feet.count as usize) < FALL_FEET_MAX {
            let k = gpu_feet.count as usize;
            let gain = if f.born { landed } else { 1.0 };
            let seed = ((f.column ^ f.layer.rotate_left(16)).wrapping_mul(0x9E37_79B9) >> 16)
                as f32
                / 65535.0;
            gpu_feet.feet[k] = f.at.extend(f.drop_m);
            gpu_feet.more[k] = Vec4::new(f.pool_m, seed, fade.near * gain, 0.0);
            gpu_feet.count += 1;
        }
    }
    state.fades.retain(|_, f| f.seen == stamp);
    // The same quads, nearest foot first, as the mesh they were drawn as:
    // the water pass expands each to its corners along the camera's right
    // and up (`splash_vertex`).
    let mut lanes = std::mem::take(&mut splash.lanes);
    lanes.clear();
    lanes.push(right.extend(0.0).to_array());
    lanes.push(up_cam.extend(0.0).to_array());
    let mut quads = 0usize;
    let mut quad = |p: Vec3, size: f32, rgba: [f32; 4], cell: f32, flip: f32, surface: f32| {
        if quads >= SPLASH_BUDGET || !p.is_finite() || !size.is_finite() || rgba[3] <= 0.0 {
            return;
        }
        quads += 1;
        lanes.push(p.extend(size).to_array());
        lanes.push(rgba);
        lanes.push([cell, flip, surface, 0.0]);
    };
    for &(_, i) in &state.order {
        let f = &feet.0[i];
        let Some(fades) = state.fades.get(&f.key()) else {
            continue;
        };
        let gain = if f.born { landed } else { 1.0 };
        let fade = fades.splash * gain;
        if fade <= 0.0 {
            continue;
        }
        let foot = f.at;
        // The pool's surface, a radius: nothing of the splash is drawn under
        // it (a puff's low centre puts part of it there).
        let surface = foot.length();
        let up = foot.normalize_or(Vec3::Y);
        let east = Vec3::Y.cross(up).normalize_or(Vec3::X);
        let north = up.cross(east);
        let light = (0.12 + 0.88 * day) * f.sky.max(0.15);
        let rule = SplashRule::of(look.rule, f.height, f.drop_m);
        let h = rule.m;
        let seed = (foot.x * 7.0) as i32 as u32 ^ ((foot.z * 13.0) as i32 as u32).rotate_left(11);
        // Spray: drops thrown up and out, falling back under gravity, each
        // fading in over the first 15% of its flight.
        for i in 0..rule.drops {
            let r = |k: u32| hash01(seed, i * 8 + k);
            let period = 0.7 + 0.5 * r(0);
            let phase = (t / period + r(1)).fract();
            let s = phase * period;
            let rise = (1.5 + 0.35 * h).min(9.0) * (0.5 + 0.5 * r(2));
            let out = (0.6 + 0.15 * h).min(4.0) * (0.3 + 0.7 * r(3));
            let a = r(4) * std::f32::consts::TAU;
            let dir = east * a.cos() + north * a.sin();
            let lift = rise * s - 0.5 * SPLASH_G * s * s;
            let size = 0.08 + 0.1 * r(6);
            // A drop back down at the pool is gone: none is drawn under it.
            if lift < SPLASH_CLEAR_M + size {
                continue;
            }
            let p = foot + up * lift + dir * (SPLASH_EDGE_M + out * s + 0.4 * r(5));
            let alpha = 0.75 * (1.0 - phase) * (phase / 0.15).min(1.0) * fade;
            quad(
                p,
                size,
                [light, light, light * 1.02, alpha],
                SPLASH_DOT,
                0.0,
                surface,
            );
        }
        // Mist: puffs off the sheet, on the fall's open side under the
        // proposed rule, each fading in over the first fifth of its life,
        // its centre low so its body meets the pool and the wall blend
        // fades it at the waterline.
        let puffs = (rule.puffs as f32 * look.mist_amount).round() as u32;
        let over = Vec2::new(f.over.dot(east), f.over.dot(north));
        let open = rule.open && over.length_squared() > 0.25;
        for i in 0..puffs {
            let r = |k: u32| hash01(seed ^ 0xA5A5, i * 8 + k);
            let period = 2.0 + 1.5 * r(0);
            let phase = (t / period + r(1)).fract();
            let s = phase * period;
            let a = if open {
                over.y.atan2(over.x) + (r(2) * 2.0 - 1.0) * 110f32.to_radians()
            } else {
                r(2) * std::f32::consts::TAU
            };
            let dir = east * a.cos() + north * a.sin();
            let spread = (0.5 + 0.12 * h).min(5.0);
            let size = rule.size * (0.6 + 0.8 * phase) * look.mist_size;
            let p = foot
                + up * (rule.rise * s + 0.2 * size)
                + dir * (SPLASH_EDGE_M + spread * (0.3 + phase) * r(3));
            let born = (phase / 0.2).min(1.0);
            let alpha = rule.alpha * (1.0 - phase) * born * fade;
            // A continuous frame (row * 4 + 0..3): the fragment cross-fades
            // the two frames either side, so a puff never changes shape in
            // one frame.
            let cell = (r(5) * 4.0).floor().min(3.0) * 4.0 + (phase * 4.0).min(3.0);
            let flip = if r(6) < 0.5 { 1.0 } else { 0.0 };
            quad(p, size, [light, light, light, alpha], cell, flip, surface);
        }
        // Fall droplets (owner: "fall mist shouldnt be mist, should be like
        // water droplets fx"): shaken off the sheet at a height along the
        // drop, already falling at the sheet's speed there, flung a little
        // out, falling under gravity into the pool. More low down. One drop
        // each: no streak trails in the game (owner, 2026-11-16: "I dont want
        // them in game yet").
        let droplets = (rule.droplets * look.droplets).round() as u32;
        for i in 0..droplets {
            let r = |k: u32| hash01(seed ^ 0x5C3E, i * 8 + k);
            let along = r(2).powf(0.7);
            let fallen = h * along;
            let y0 = h - fallen;
            let v_down = (2.0 * SPLASH_G * fallen.max(0.2)).sqrt();
            let v_out = (0.6 + 1.6 * r(4)) * (0.5 + 0.5 * along);
            let t_land =
                (-v_down + (v_down * v_down + 2.0 * SPLASH_G * y0.max(0.05)).sqrt()) / SPLASH_G;
            let period = t_land + 0.15 + 0.4 * r(0);
            let s = (t / period + r(1)).fract() * period;
            if s > t_land {
                continue;
            }
            let a = r(3) * std::f32::consts::TAU;
            let dir = east * a.cos() + north * a.sin();
            let size = (0.035 + 0.05 * r(6)) * look.mist_size;
            let d = SPLASH_EDGE_M * 0.95 + v_out * s;
            let y = y0 - v_down * s - 0.5 * SPLASH_G * s * s;
            let head = 0.9 * (s / 0.05).min(1.0) * fade;
            quad(
                foot + up * y + dir * d,
                size,
                [light, light, light * 1.03, head],
                SPLASH_DROPLET,
                0.0,
                surface,
            );
        }
    }
    if quads == 0 {
        lanes.clear();
    }
    splash.lanes = lanes;
}

#[cfg(test)]
mod splash_tests {
    use super::*;
    use crate::config::MistRule;

    /// The proposed rule against the mockup's height comparison
    /// (`water-flow` design, "The height comparison"): puffs and alpha for
    /// the riffle and the 1, 2, 4 and 9-layer falls.
    #[test]
    fn the_proposed_rule_mists_every_fall_as_the_mockup_does() {
        for (d, puffs) in [(1.05, 8), (1.4, 8), (2.4, 9), (4.4, 10), (9.4, 14)] {
            let r = SplashRule::of(MistRule::Proposed, 1.0, d);
            assert_eq!(r.puffs, puffs, "D {d}");
            assert_eq!(r.alpha, 0.45);
            assert!(r.drops >= 12 && r.droplets >= 6.0 && r.open);
        }
        let tall = SplashRule::of(MistRule::Proposed, 9.0, 9.4);
        assert!((tall.size - 2.036).abs() < 1e-3, "{}", tall.size);
        // The rule before M35, kept behind the switch: one puff for a
        // two-layer fall, twelve for nine.
        assert_eq!(SplashRule::of(MistRule::Layers, 2.0, 2.4).puffs, 1);
        assert_eq!(SplashRule::of(MistRule::Layers, 9.0, 9.4).puffs, 12);
    }

    /// A fade runs in time to its end and stops: never held part way.
    #[test]
    fn a_foot_s_fade_is_a_moment_in_time() {
        let mut v = 0.0;
        for _ in 0..30 {
            v = fade_toward(v, 1.0, 1.0 / 60.0, 0.6);
        }
        assert!(v > 0.8 && v < 0.9, "{v}");
        for _ in 0..10 {
            v = fade_toward(v, 1.0, 1.0 / 60.0, 0.6);
        }
        assert_eq!(v, 1.0);
        v = fade_toward(v, 0.0, 1.0, 0.6);
        assert_eq!(v, 0.0);
    }
}
