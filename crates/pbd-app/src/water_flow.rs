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
use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use pbd_core::column::{self, LAYERS};
use pbd_core::edits::Edit;
use pbd_core::liquid;
use pbd_core::terrain::Material;
use std::sync::Arc;

/// Tenebris's water tile (`tenebris-rs/assets/textures/water.png`, 16 x 16,
/// at `RubenTipparach/tenebris` `ef9865166349c7a3c4ae09acdd20640daf85b4fd`),
/// as each pixel's colour index, top row first: the same 256 indices the
/// mockups carry (`docs/mockups/water-flow/water-look.js`, `TEX_IDX`).
const TILE: &str = "0000011000111100100000000000000000010121000000001000000000001012001000000100000000001001000000000000000000110010010000000000000000000101201000001000000000000122000000000000000010112221010001000000000000000000001000000002100020001001100000010000000000000000";
/// The tile's three colours, darkest first: the body, a ripple, a glint.
const TILE_COLOURS: [[u8; 3]; 3] = [[29, 43, 83], [28, 94, 172], [88, 138, 192]];
/// Metres one tile covers: 12.5 cm a pixel, an eighth of a layer.
const TILE_M: f32 = 2.0;
/// Metres a second a flowing top's pattern pans at a slope of `PAN_SLOPE`.
const PAN_K: f32 = 1.6;
/// The slope, metres a metre, that pans at `PAN_K`.
const PAN_SLOPE: f32 = 0.12;
/// Metres a second a falling face's pattern pans down; a flowing cell's wall
/// pans at a third of it.
const FALL_PAN: f32 = 2.4;
/// How far over the water the arrows float, m.
const ARROW_LIFT: f32 = 0.05;
/// The water's opacity.
const WATER_ALPHA: f32 = 0.8;

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
struct WaterMesh;

#[derive(Component)]
struct ArrowMesh;

/// What the mesher made, kept so a frame only moves the texture and the
/// light rather than meshing again.
#[derive(Resource, Default)]
struct Meshed {
    version: u64,
    set: usize,
    uv: Vec<[f32; 2]>,
    pan: Vec<[f32; 2]>,
    /// Sky and block light, 0..1, per vertex.
    light: Vec<[f32; 2]>,
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
    mut fine: ResMut<PlanetFine>,
    mut contact: ResMut<PlanetContact>,
    mut save: ResMut<WorldSave>,
    refresh: Res<crate::planet::LodRefresh>,
) {
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

/// The tile as an image: nearest, repeating, never filtered.
fn tile_image() -> Image {
    let mut data = Vec::with_capacity(16 * 16 * 4);
    // The ripple and the glint eased toward the body, as the mockups draw
    // them as tones of the water they are on (design 7c): the tile's own
    // colours at full contrast read as confetti from a walker's height.
    let body = TILE_COLOURS[0].map(f32::from);
    for c in TILE.bytes() {
        let index = usize::from(c - b'0').min(2);
        let ease = [0.0, 0.45, 0.6][index];
        let tone = TILE_COLOURS[index].map(f32::from);
        let [r, g, b] = [0, 1, 2].map(|k| (body[k] + (tone[k] - body[k]) * ease).round() as u8);
        data.extend_from_slice(&[r, g, b, 255]);
    }
    let mut image = Image::new(
        Extent3d {
            width: 16,
            height: 16,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        mipmap_filter: ImageFilterMode::Nearest,
        ..default()
    });
    image
}

/// A mesh of one degenerate triangle with the water's attributes: what the
/// entities are born with, so the loading screen builds the pipeline the
/// real mesh will use (CLAUDE.md, "A loading screen on boot").
fn placeholder(uv: bool) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vec![[0.0f32; 3]; 3])
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0f32, 1.0, 0.0]; 3]);
    if uv {
        mesh = mesh
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, vec![[0.0f32; 2]; 3])
            .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0f32; 4]; 3]);
    }
    mesh.with_inserted_indices(Indices::U32(vec![0, 1, 2]))
}

fn spawn_water(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let tile = images.add(tile_image());
    commands.spawn((
        Name::new("Flowing water"),
        WaterMesh,
        Mesh3d(meshes.add(placeholder(true))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 1.0, 1.0, WATER_ALPHA),
            base_color_texture: Some(tile),
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
    commands.spawn((
        Name::new("Flow arrows"),
        ArrowMesh,
        Mesh3d(meshes.add(placeholder(false))),
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
    pub uv: Vec<[f32; 2]>,
    /// Texture coordinates a second, per vertex: the pan.
    pub pan: Vec<[f32; 2]>,
    pub light: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    pub arrow_positions: Vec<[f32; 3]>,
    pub arrow_indices: Vec<u32>,
    /// Water cells drawn, tops drawn, and tops with a flow.
    pub cells: usize,
    pub tops: usize,
    pub flowing_tops: usize,
}

impl LiquidMesh {
    fn vertex(&mut self, p: Vec3, n: Vec3, uv: Vec2, pan: Vec2, light: [f32; 2]) -> u32 {
        self.positions.push(p.to_array());
        self.normals.push(n.to_array());
        self.uv.push(uv.to_array());
        self.pan.push(pan.to_array());
        self.light.push(light);
        (self.positions.len() - 1) as u32
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

/// The tangent basis the tops' texture is laid in: the same for every cell
/// near `up`, so a pattern runs on across cells.
fn basis(up: Vec3) -> (Vec3, Vec3) {
    let reference = if up.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
    let t = (reference - up * reference.dot(up)).normalize_or(Vec3::X);
    (t, up.cross(t))
}

/// Mesh every water cell of the tier the sea sheet does not draw.
pub fn liquid_mesh(tier: &ColumnTier, finest: &[GpuCell], radius: f32, sea_level_m: f32) -> LiquidMesh {
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
        let (t, b) = basis(centre);
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
                [
                    f32::from(own.sky().max(up.sky())) / 15.0,
                    f32::from(own.block().max(up.block())) / 15.0,
                ]
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
                let pan = if flowing {
                    let speed = PAN_K * (slope.length() / PAN_SLOPE).clamp(0.35, 1.5);
                    let dir = slope.normalize();
                    -Vec2::new(dir.dot(t), dir.dot(b)) * speed / TILE_M
                } else {
                    Vec2::ZERO
                };
                let uv = |p: Vec3| Vec2::new(p.dot(t), p.dot(b)) / TILE_M;
                let c = out.vertex(middle, centre, uv(middle), pan, lit);
                let ring: Vec<u32> = (0..degree)
                    .map(|i| {
                        let p = at(corners[i], levels[i]);
                        out.vertex(p, centre, uv(p), pan, lit)
                    })
                    .collect();
                for i in 0..degree {
                    out.indices.extend([c, ring[i], ring[(i + 1) % degree]]);
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
                let edge = (z - a).normalize_or(t);
                let pan = Vec2::new(
                    0.0,
                    if falling {
                        FALL_PAN
                    } else if flow.is_some() {
                        FALL_PAN / 3.0
                    } else {
                        0.0
                    } / TILE_M,
                );
                let uv = |p: Vec3, alt: f32| Vec2::new(p.dot(edge) / TILE_M, alt / TILE_M);
                let pts = [
                    (at(a, bottom_a), floor + bottom_a),
                    (at(z, bottom_z), floor + bottom_z),
                    (at(z, top_z), floor + top_z),
                    (at(a, top_a), floor + top_a),
                ];
                let ids: Vec<u32> = pts
                    .iter()
                    .map(|&(p, alt)| out.vertex(p, normal, uv(p, alt), pan, lit))
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
                let uv = |p: Vec3| Vec2::new(p.dot(t), p.dot(b)) / TILE_M;
                let c = out.vertex(at(centre, 0.0), -centre, uv(at(centre, 0.0)), Vec2::ZERO, lit);
                let ring: Vec<u32> = (0..degree)
                    .map(|i| {
                        let p = at(corners[i], 0.0);
                        out.vertex(p, -centre, uv(p), Vec2::ZERO, lit)
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
#[allow(clippy::type_complexity)]
fn rebuild_water(
    fine: Res<PlanetFine>,
    mut meshed: ResMut<Meshed>,
    mut meshes: ResMut<Assets<Mesh>>,
    water: Query<&Mesh3d, (With<WaterMesh>, Without<ArrowMesh>)>,
    arrows: Query<&Mesh3d, (With<ArrowMesh>, Without<WaterMesh>)>,
) {
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
    if let Ok(handle) = water.single()
        && let Some(mesh) = meshes.get_mut(&handle.0)
    {
        if made.indices.is_empty() {
            *mesh = placeholder(true);
            meshed.uv = vec![[0.0; 2]; 3];
            meshed.pan = vec![[0.0; 2]; 3];
            meshed.light = vec![[0.0; 2]; 3];
        } else {
            let colours = vec![[1.0f32; 4]; made.positions.len()];
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, made.positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, made.normals);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, made.uv.clone());
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colours);
            mesh.insert_indices(Indices::U32(made.indices));
            meshed.uv = made.uv;
            meshed.pan = made.pan;
            meshed.light = made.light;
        }
    }
    if let Ok(handle) = arrows.single()
        && let Some(mesh) = meshes.get_mut(&handle.0)
    {
        if made.arrow_indices.is_empty() {
            *mesh = placeholder(false);
        } else {
            let normals = vec![[0.0f32, 1.0, 0.0]; made.arrow_positions.len()];
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, made.arrow_positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
            mesh.insert_indices(Indices::U32(made.arrow_indices));
        }
    }
}

/// Each frame: the pattern panned along the flow, the light by the hour, the
/// meshes on the planet as its frame moves, the arrows shown or not.
#[allow(clippy::type_complexity)]
fn animate_water(
    time: Res<Time>,
    sun: Option<Res<crate::sky::Sun>>,
    frame: Res<PlanetRenderFrame>,
    meshed: Res<Meshed>,
    arrows_on: Res<FlowArrows>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut water: Query<(&Mesh3d, &mut Transform), (With<WaterMesh>, Without<ArrowMesh>)>,
    mut arrows: Query<(&mut Transform, &mut Visibility), (With<ArrowMesh>, Without<WaterMesh>)>,
) {
    let centre = frame.center.as_vec3();
    let shown = arrows_on.shown();
    for (mut transform, mut visibility) in &mut arrows {
        if transform.translation != centre {
            transform.translation = centre;
        }
        let want = if shown {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *visibility != want {
            *visibility = want;
        }
    }
    let Ok((handle, mut transform)) = water.single_mut() else {
        return;
    };
    if transform.translation != centre {
        transform.translation = centre;
    }
    if meshed.uv.is_empty() || meshed.pan.iter().all(|p| *p == [0.0; 2]) && meshed.uv.len() <= 3 {
        return;
    }
    let up = cameras
        .iter()
        .find(|(_, c)| c.is_active)
        .map(|(g, _)| (g.translation() - centre).normalize_or(Vec3::Y))
        .unwrap_or(Vec3::Y);
    let elevation = sun.as_ref().map_or(1.0, |s| s.direction().dot(up));
    let day = smoothstep(-0.12, 0.2, elevation);
    let sky = Vec3::new(0.10, 0.12, 0.20).lerp(Vec3::ONE, day);
    let lamp = Vec3::new(1.0, 0.72, 0.42);
    let t = time.elapsed_secs_wrapped() % 3600.0;
    let Some(mesh) = meshes.get_mut(&handle.0) else {
        return;
    };
    let uv: Vec<[f32; 2]> = meshed
        .uv
        .iter()
        .zip(&meshed.pan)
        .map(|(uv, pan)| [uv[0] + pan[0] * t, uv[1] + pan[1] * t])
        .collect();
    let colours: Vec<[f32; 4]> = meshed
        .light
        .iter()
        .map(|&[s, b]| {
            let c = (sky * s.max(0.08) + lamp * b * b).min(Vec3::splat(1.2));
            [c.x, c.y, c.z, 1.0]
        })
        .collect();
    if uv.len() == colours.len() && mesh.count_vertices() == uv.len() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colours);
    }
}

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The water's systems.
pub struct WaterFlowPlugin;

impl Plugin for WaterFlowPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaterClock>()
            .init_resource::<FlowArrows>()
            .init_resource::<Meshed>()
            .add_systems(Startup, spawn_water)
            .add_systems(Update, (tick_water, rebuild_water, animate_water).chain());
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
        let mean = corner_level(&[side(f(7), false), side(f(5), false), side(Material::Air, false)]);
        assert!((mean - (7.5 + 5.5) / 16.0).abs() < 1e-6);
        assert_eq!(
            corner_level(&[side(Material::Stone, false), side(Material::Stone, false), None]),
            0.5
        );
    }

    #[test]
    fn the_tile_is_sixteen_by_sixteen_in_three_colours() {
        assert_eq!(TILE.len(), 256);
        assert!(TILE.bytes().all(|c| (b'0'..=b'2').contains(&c)));
        let image = tile_image();
        assert_eq!(image.width(), 16);
    }
}
