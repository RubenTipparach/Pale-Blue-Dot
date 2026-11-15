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
    pub flow: Vec<[f32; 2]>,
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
            let [fx, fy] = self.flow[i];
            out.push([x, y, z, self.sky[i]]);
            out.push([nx, ny, nz, self.kind[i]]);
            out.push([fx, fy, 0.0, 0.0]);
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
                    for v in [middle, p, q] {
                        let id = out.vertex(v, n, pan, KIND_TOP, lit);
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
                let kind = if falling || above_water && flow.is_some() {
                    KIND_FALLING
                } else {
                    KIND_STILL
                };
                let pts = [
                    (at(a, bottom_a), floor + bottom_a),
                    (at(z, bottom_z), floor + bottom_z),
                    (at(z, top_z), floor + top_z),
                    (at(a, top_a), floor + top_a),
                ];
                let ids: Vec<u32> = pts
                    .iter()
                    .map(|&(p, _)| out.vertex(p, normal, Vec2::ZERO, kind, lit))
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
            .add_systems(Startup, spawn_arrows)
            .add_systems(Update, (tick_water, rebuild_water, show_arrows).chain());
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
