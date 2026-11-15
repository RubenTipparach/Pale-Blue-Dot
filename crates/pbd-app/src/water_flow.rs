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

use crate::planet::column::{ColumnTier, sea_layer};
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

/// The game's own water as the water pass draws it (`water-flow` E6): three
/// lanes a vertex, triangles end to end (`vertex_flowing` in `water.wgsl`),
/// and which make of it this is.
#[derive(Resource, Clone, Default, bevy::render::extract_resource::ExtractResource)]
pub struct FlowingWater {
    pub lanes: Arc<Vec<[f32; 4]>>,
    pub version: u64,
}

/// When the next tick is due.
#[derive(Resource, Default)]
pub struct WaterClock {
    since_s: f32,
    /// Ticks run, for the log.
    pub ticks: u64,
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
/// moves if the save refuses.
pub fn run_ticks(
    fine: &mut PlanetFine,
    contact: &mut PlanetContact,
    save: &mut WorldSave,
    ticks: usize,
) -> usize {
    if fine.set.columns.water.queue.is_empty() || ticks == 0 {
        return 0;
    }
    let mut set = (*fine.set).clone();
    let mut changed = 0;
    for _ in 0..ticks {
        if set.columns.water.queue.is_empty() {
            break;
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
        if !save.accept_water(&edits) {
            error!("water BLOCKED: the save did not take {} cells", edits.len());
            return 0;
        }
        changed += made.len();
    }
    let set = Arc::new(set);
    contact.set_fine(&set);
    fine.set = set;
    fine.version += 1;
    changed
}

/// Once a second, while anything is queued and no set is in flight (a set
/// built before a tick would land without it).
fn tick_water(
    time: Res<Time>,
    mut clock: ResMut<WaterClock>,
    fine: Option<ResMut<PlanetFine>>,
    contact: Option<ResMut<PlanetContact>>,
    save: Option<ResMut<WorldSave>>,
    refresh: Option<Res<crate::planet::LodRefresh>>,
) {
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
    let changed = run_ticks(&mut fine, &mut contact, &mut save, 1);
    clock.ticks += 1;
    debug!(
        "water tick {}: {changed} cells changed, {} queued",
        clock.ticks,
        fine.set.columns.water.queue.len()
    );
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

/// What the mesher makes: the water's triangles and the arrows'.
#[derive(Default, Debug)]
pub struct LiquidMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    /// The flow in the face's tangent frame as the sea's fragment reads it.
    /// The flow lane: a top's pan (xy) and how near a fall's lip it is (z);
    /// a face's froth weight (x) and how far under its fall's lip it is (z).
    pub flow: Vec<[f32; 3]>,
    /// The face's kind, and the sky's share at it, 0..1.
    pub kind: Vec<f32>,
    pub sky: Vec<f32>,
    pub indices: Vec<u32>,
    pub arrow_positions: Vec<[f32; 3]>,
    pub arrow_indices: Vec<u32>,
    /// Water cells drawn, tops drawn, and tops with a flow.
    pub cells: usize,
    pub tops: usize,
    pub flowing_tops: usize,
    /// The foot of every fall: where it lands (body-local), how many layers
    /// it fell, and the sky's share there. What the splash draws from.
    pub feet: Vec<(Vec3, f32, f32)>,
}

/// The tangent frame the sea's fragment builds off a face (`fragment` in
/// `water.wgsl`): X, or Y where the face is near X, off the face.
fn shader_frame(face: Vec3) -> (Vec3, Vec3) {
    let reference = if face.x.abs() > 0.9 { Vec3::Y } else { Vec3::X };
    let u = (reference - face * reference.dot(face)).normalize_or(Vec3::X);
    (u, face.cross(u))
}

impl LiquidMesh {
    fn vertex(&mut self, p: Vec3, n: Vec3, flow: Vec2, kind: f32, sky: f32) -> u32 {
        self.vertex_with(p, n, flow.extend(0.0), kind, sky)
    }

    fn vertex_with(&mut self, p: Vec3, n: Vec3, flow: Vec3, kind: f32, sky: f32) -> u32 {
        self.positions.push(p.to_array());
        self.normals.push(n.to_array());
        self.flow.push(flow.to_array());
        self.kind.push(kind);
        self.sky.push(sky);
        (self.positions.len() - 1) as u32
    }

    /// The triangles as the water pass reads them: three lanes a vertex.
    pub fn lanes(&self) -> Vec<[f32; 4]> {
        let mut out = Vec::with_capacity(self.indices.len() * 3);
        for &i in &self.indices {
            let i = i as usize;
            let [x, y, z] = self.positions[i];
            let [nx, ny, nz] = self.normals[i];
            let [fx, fy, fz] = self.flow[i];
            out.push([x, y, z, self.sky[i]]);
            out.push([nx, ny, nz, self.kind[i]]);
            out.push([fx, fy, fz, 0.0]);
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

/// Mesh every water cell of the tier the sea sheet does not draw.
pub fn liquid_mesh(
    tier: &ColumnTier,
    finest: &[GpuCell],
    radius: f32,
    sea_level_m: f32,
) -> LiquidMesh {
    let mut out = LiquidMesh::default();
    let sea = sea_layer();
    let material = |slot: usize, layer: usize| -> Material {
        tier.columns
            .get(slot)
            .map_or(Material::Stone, |c| c.material(layer))
    };
    for slot in 0..tier.columns.len() {
        let Some(&record) = tier.water.records.get(slot) else {
            continue;
        };
        let Some(cell) = finest.get(record) else {
            continue;
        };
        let column = &tier.columns[slot];
        let degree = cell.degree().min(6);
        let centre = Vec3::from_slice(&cell.direction_height[..3]).normalize_or(Vec3::Y);
        let corners: Vec<Vec3> = (0..degree)
            .map(|i| Vec3::from_slice(&cell.corners[i][..3]).normalize_or(centre))
            .collect();
        let sides = tier.sides_of(slot);
        let under_sea = cell.direction_height[3] < sea_level_m;
        let (t, b) = shader_frame(centre);
        for layer in 1..LAYERS - 1 {
            let here = column.material(layer);
            if !here.is_water() {
                continue;
            }
            let floor = column::layer_altitude(layer);
            let above_water = column.material(layer + 1).is_water();
            let flow = here.flow();
            let falling = flow.is_some_and(|(_, f)| f);
            // The sea's own top: the sheet draws it.
            let sheet = here == Material::Water && layer + 1 == sea && under_sea;
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
            let levels: Vec<f32> = (0..degree)
                .map(|i| {
                    if above_water {
                        1.0
                    } else {
                        corner_level(&[me, side_of((i + degree - 1) % degree), side_of(i)])
                    }
                })
                .collect();
            let lit = {
                let own = tier.light_at(slot, layer);
                let up = tier.light_at(slot, layer + 1);
                f32::from(own.sky().max(up.sky())) / 15.0
            };
            let at = |dir: Vec3, fraction: f32| dir * (radius + floor + fraction);
            out.cells += 1;
            // The top, through the corners, unless water stands on it.
            let mean = levels.iter().sum::<f32>() / degree.max(1) as f32;
            let middle = at(centre, mean);
            // Luanti's flow, down the corner slope: the least-squares slope
            // of the corners about the middle.
            let mut gradient = Vec3::ZERO;
            let mut spread = 0.0;
            for (i, &k) in corners.iter().enumerate() {
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
            // The foot of a fall: water that is not falling, with falling
            // water on it. How far it fell is the falling run above.
            let is_falling = |m: Material| m.flow().is_some_and(|(_, f)| f);
            if !falling && is_falling(column.material(layer + 1)) {
                let height = (layer + 1..LAYERS)
                    .take_while(|&l| is_falling(column.material(l)))
                    .count();
                out.feet.push((at(centre, 1.0), height as f32, lit));
            }
            if !above_water && !sheet {
                out.tops += 1;
                // The waves stream along the flow, faster on a steeper top.
                let pan = if flowing {
                    let speed = PAN_K * (slope.length() / PAN_SLOPE).clamp(0.35, 1.5);
                    let dir = slope.normalize();
                    Vec2::new(dir.dot(t), dir.dot(b)) * speed / SHADER_FLOW
                } else {
                    Vec2::ZERO
                };
                // How near each corner is to a fall's lip (owner, 2026-11-15:
                // "can we have the edge transition into froth?"): one at a
                // corner on a side whose water goes over an edge, so the top
                // whitens and speeds up toward the fall across its last metre
                // or so. An edge goes over where the side's water falls, or
                // falls from under it, or this cell's own water falls below.
                let over = |s: usize| -> bool {
                    sides[s].is_some_and(|n| {
                        is_falling(material(n, layer))
                            || (!tier.columns[n].solid(layer) && is_falling(material(n, layer - 1)))
                    })
                };
                let own_lip = is_falling(column.material(layer - 1));
                let lips: Vec<f32> = (0..degree)
                    .map(|i| {
                        if own_lip || over((i + degree - 1) % degree) || over(i) {
                            1.0
                        } else {
                            0.0
                        }
                    })
                    .collect();
                let middle_lip = if own_lip { 0.6 } else { 0.0 };
                // Each triangle lit by its own slope, so a top running down to
                // a sheet's edge reads as water tilted, not a mirror on edge.
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
                        let id = out.vertex_with(v, n, pan.extend(lip), KIND_TOP, lit);
                        out.indices.push(id);
                    }
                }
                if flowing {
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
                let normal = ((a + z) * 0.5 - centre).normalize_or(centre);
                let normal = (normal - centre * normal.dot(centre)).normalize_or(normal);
                let edge = (z - a).normalize_or(t);
                // A falling cell's face is white water streaming down; a
                // sheet's edge and a source's face stand still.
                // A fall's froth is densest at its lip and its foot.
                let froth = if !falling {
                    0.5
                } else if !is_falling(column.material(layer + 1))
                    || !is_falling(column.material(layer - 1))
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
                // How far under its fall's lip each corner is, metres: the
                // falling run over this layer, and the fraction into it.
                let run = if falling {
                    (layer + 1..LAYERS)
                        .take_while(|&l| is_falling(column.material(l)))
                        .count() as f32
                } else {
                    0.0
                };
                let under = |fraction: f32| if falling { run + 1.0 - fraction } else { 0.0 };
                let pts = [
                    (at(a, bottom_a), floor + bottom_a),
                    (at(z, bottom_z), floor + bottom_z),
                    (at(z, top_z), floor + top_z),
                    (at(a, top_a), floor + top_a),
                ];
                let ids: Vec<u32> = pts
                    .iter()
                    .zip([bottom_a, bottom_z, top_z, top_a])
                    .map(|(&(p, _), fraction)| {
                        out.vertex_with(
                            p,
                            normal,
                            Vec3::new(froth, 0.0, under(fraction)),
                            kind,
                            lit,
                        )
                    })
                    .collect();
                out.indices
                    .extend([ids[0], ids[1], ids[2], ids[0], ids[2], ids[3]]);
                if falling && !there.is_water() {
                    let mid = (pts[0].0 + pts[1].0 + pts[2].0 + pts[3].0) * 0.25;
                    out.arrow(mid + normal * ARROW_LIFT, -centre, edge, 0.7);
                }
            }
            // The bottom, over air.
            if column.material(layer - 1) == Material::Air {
                let c = out.vertex(at(centre, 0.0), -centre, Vec2::ZERO, KIND_STILL, lit);
                let ring: Vec<u32> = (0..degree)
                    .map(|i| {
                        let p = at(corners[i], 0.0);
                        out.vertex(p, -centre, Vec2::ZERO, KIND_STILL, lit)
                    })
                    .collect();
                for i in 0..degree {
                    out.indices.extend([c, ring[(i + 1) % degree], ring[i]]);
                }
            }
        }
    }
    out
}

/// Mesh the tier's water again when the fine set changes.
fn rebuild_water(
    fine: Option<Res<PlanetFine>>,
    mut meshed: ResMut<Meshed>,
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
    let made = liquid_mesh(
        &fine.set.columns,
        fine.set.finest_records(),
        PLANET_RADIUS,
        crate::planet::terrain_config().sea_level_m,
    );
    if made.cells > 0 {
        debug!(
            "water mesh: {} cells, {} tops ({} flowing), {} triangles in {:.2} ms",
            made.cells,
            made.tops,
            made.flowing_tops,
            made.indices.len() / 3,
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
            .add_plugins(bevy::render::extract_resource::ExtractResourcePlugin::<
                FlowingWater,
            >::default())
            .init_resource::<FallFeet>()
            .add_systems(Startup, (spawn_arrows, spawn_splash))
            .add_systems(
                Update,
                (tick_water, rebuild_water, show_arrows, animate_splash).chain(),
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
}

// ---- The splash at a fall's foot (owner, 2026-11-15: "can you also add some
// particle fx to bottom of the waterfall? more fx based on distance it fell")

/// Every fall's foot in the tier, as the mesher found them: where, how many
/// layers it fell, and the sky's share there.
#[derive(Resource, Default)]
pub struct FallFeet(pub Vec<(Vec3, f32, f32)>);

#[derive(Component)]
struct Splash;

/// The most quads the splash draws in a frame, drops and mist together.
const SPLASH_BUDGET: usize = 4000;
/// Feet are drawn out to this far, fading out over the last 15 m.
const SPLASH_REACH_M: f32 = 70.0;

/// A soft round dot, white, alpha falling off to its edge.
fn splash_dot() -> Image {
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    let size = 32u32;
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let dx = (x as f32 + 0.5) / size as f32 * 2.0 - 1.0;
            let dy = (y as f32 + 0.5) / size as f32 * 2.0 - 1.0;
            let r = (dx * dx + dy * dy).sqrt();
            let a = (1.0 - r).clamp(0.0, 1.0);
            data.extend_from_slice(&[255, 255, 255, (a * a * 255.0) as u8]);
        }
    }
    Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

fn splash_mesh(positions: Vec<[f32; 3]>, uv: Vec<[f32; 2]>, colours: Vec<[f32; 4]>) -> Mesh {
    let n = positions.len();
    let indices: Vec<u32> = (0..n as u32 / 4)
        .flat_map(|q| [0, 1, 2, 0, 2, 3].map(|i| q * 4 + i))
        .collect();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0f32, 1.0, 0.0]; n])
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
    .with_inserted_indices(Indices::U32(indices))
}

/// The splash's one mesh, born with one invisible quad so the loading screen
/// builds its pipeline.
fn spawn_splash(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let dot = images.add(splash_dot());
    commands.spawn((
        Name::new("Waterfall splash"),
        Splash,
        Mesh3d(meshes.add(splash_mesh(
            vec![[0.0; 3]; 4],
            vec![[0.0; 2]; 4],
            vec![[0.0; 4]; 4],
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: Some(dot),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            double_sided: true,
            ..default()
        })),
        Transform::IDENTITY,
        bevy::camera::visibility::NoFrustumCulling,
        bevy::light::NotShadowCaster,
        bevy::light::NotShadowReceiver,
    ));
}

fn hash01(a: u32, b: u32) -> f32 {
    let mut h = a.wrapping_mul(0x9E37_79B9) ^ b.wrapping_mul(0x85EB_CA6B);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h & 0xFFFF) as f32 / 65535.0
}

/// Spray and mist at every fall's foot near the eye: thrown up and out and
/// falling back, a low white puff, more of both, higher and wider the further
/// the water fell. A pure function of the time, so nothing is stored.
#[allow(clippy::too_many_arguments)]
fn animate_splash(
    time: Res<Time>,
    sun: Option<Res<crate::sky::Sun>>,
    frame: Option<Res<PlanetRenderFrame>>,
    feet: Res<FallFeet>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut splash: Query<(&Mesh3d, &mut Transform), With<Splash>>,
) {
    let (Some(frame), Ok((handle, mut transform))) = (frame, splash.single_mut()) else {
        return;
    };
    let centre = frame.center.as_vec3();
    if transform.translation != centre {
        transform.translation = centre;
    }
    let Some((camera, _)) = cameras.iter().find(|(_, c)| c.is_active) else {
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
    let mut near: Vec<&(Vec3, f32, f32)> = feet
        .0
        .iter()
        .filter(|(p, ..)| p.distance(eye) < SPLASH_REACH_M)
        .collect();
    near.sort_by(|a, b| a.0.distance(eye).total_cmp(&b.0.distance(eye)));
    let (mut positions, mut uv, mut colours) = (Vec::new(), Vec::new(), Vec::new());
    let mut quad = |p: Vec3, size: f32, rgba: [f32; 4], positions: &mut Vec<[f32; 3]>| {
        let (r, u) = (right * size, up_cam * size);
        for (c, tex) in [
            (-r - u, [0.0, 1.0]),
            (r - u, [1.0, 1.0]),
            (r + u, [1.0, 0.0]),
            (u - r, [0.0, 0.0]),
        ] {
            positions.push((p + c).to_array());
            uv.push(tex);
            colours.push(rgba);
        }
    };
    for (foot, height, sky) in near {
        if positions.len() / 4 >= SPLASH_BUDGET {
            break;
        }
        let distance = foot.distance(eye);
        let fade = 1.0 - ((distance - (SPLASH_REACH_M - 15.0)) / 15.0).clamp(0.0, 1.0);
        let up = foot.normalize_or(Vec3::Y);
        let east = Vec3::Y.cross(up).normalize_or(Vec3::X);
        let north = up.cross(east);
        let light = (0.12 + 0.88 * day) * sky.max(0.15);
        let h = height.max(1.0);
        let seed = (foot.x * 7.0) as i32 as u32 ^ ((foot.z * 13.0) as i32 as u32).rotate_left(11);
        // Spray: drops thrown up and out, falling back under gravity.
        let drops = (3.0 + 6.0 * h).min(160.0) as u32;
        for i in 0..drops {
            let r = |k: u32| hash01(seed, i * 8 + k);
            let period = 0.7 + 0.5 * r(0);
            let phase = (t / period + r(1)).fract();
            let s = phase * period;
            let rise = (1.5 + 0.35 * h).min(9.0) * (0.5 + 0.5 * r(2));
            let out = (0.6 + 0.15 * h).min(4.0) * (0.3 + 0.7 * r(3));
            let a = r(4) * std::f32::consts::TAU;
            let dir = east * a.cos() + north * a.sin();
            let lift = rise * s - 0.5 * 15.0 * s * s;
            if lift < -0.2 {
                continue;
            }
            let p = *foot + up * lift + dir * (out * s + 0.4 * r(5));
            let alpha = 0.75 * (1.0 - phase) * fade;
            quad(
                p,
                0.08 + 0.1 * r(6),
                [light, light, light * 1.02, alpha],
                &mut positions,
            );
        }
        // Mist: a low white puff, bigger and denser from a taller fall.
        let puffs = ((h - 1.0) * 1.5).clamp(1.0, 24.0) as u32;
        for i in 0..puffs {
            let r = |k: u32| hash01(seed ^ 0xA5A5, i * 8 + k);
            let period = 2.0 + 1.5 * r(0);
            let phase = (t / period + r(1)).fract();
            let s = phase * period;
            let a = r(2) * std::f32::consts::TAU;
            let dir = east * a.cos() + north * a.sin();
            let spread = (0.5 + 0.12 * h).min(5.0);
            let p =
                *foot + up * ((0.25 + 0.04 * h) * s + 0.2) + dir * spread * (0.3 + phase) * r(3);
            let size = (0.5 + 0.18 * h).min(5.0) * (0.6 + 0.8 * phase);
            let alpha = (0.12 + 0.33 * (h / 8.0).min(1.0)) * (1.0 - phase) * fade;
            quad(p, size, [light, light, light, alpha], &mut positions);
        }
    }
    let Some(mesh) = meshes.get_mut(&handle.0) else {
        return;
    };
    if positions.is_empty() {
        positions = vec![[0.0; 3]; 4];
        uv = vec![[0.0; 2]; 4];
        colours = vec![[0.0; 4]; 4];
    }
    *mesh = splash_mesh(positions, uv, colours);
}
