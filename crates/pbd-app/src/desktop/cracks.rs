//! The cracks on the block being broken: Minecraft's breaking overlay.
//!
//! One small prism over the targeted cell, built on the CPU from the cell's
//! own record (its five or six corner rays) between the layer's bottom and
//! top, pushed out a centimetre so it lies on the block's faces rather than
//! fighting them for depth. It wears one of ten crack stages
//! (`assets/textures/break/`, from `tools/gen_break_stages.py`) chosen by how
//! far along the break is, and uses the terrain shader's own face UVs
//! (`planet_surface.wgsl`), so a crack pixel lands on exactly one block pixel.
//!
//! It is its own entity rather than a term in the terrain shader: the terrain
//! pass touches every visible cell, and this is one block. The mesh is rebuilt
//! only when the target changes and the material only when the stage does.

use super::digging::{Block, Mining};
use bevy::asset::RenderAssetUsages;
use bevy::light::NotShadowCaster;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use pbd_app::planet::{GpuCell, PLANET_RADIUS, PlanetFine};
use pbd_core::column;
use pbd_core::dig::{STAGES, stage};

/// How far the overlay stands off the block's faces, m: enough to win the
/// depth test at arm's length, far too little to see as a gap.
const LIFT_M: f32 = 0.01;

/// The ten stage materials, made once.
#[derive(Resource)]
pub struct CrackArt {
    stages: Vec<Handle<StandardMaterial>>,
}

/// The overlay entity, and what it is showing.
#[derive(Component, Default)]
pub struct Cracks {
    block: Option<Block>,
    stage: Option<u8>,
}

/// Load the stages and spawn the (hidden) overlay.
pub fn spawn(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let stages = (0..STAGES)
        .map(|k| {
            materials.add(StandardMaterial {
                base_color_texture: Some(assets.load(format!("textures/break/stage_{k}.png"))),
                // Unlit and blended over the block, as Tenebris blends its
                // crack channel: a crack is a shadow in the surface, and it
                // reads the same in the sun and in a cave.
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                fog_enabled: false,
                ..default()
            })
        })
        .collect::<Vec<_>>();
    commands.spawn((
        Mesh3d(meshes.add(prism(None))),
        MeshMaterial3d(stages[0].clone()),
        Transform::IDENTITY,
        Visibility::Hidden,
        NotShadowCaster,
        Cracks::default(),
    ));
    commands.insert_resource(CrackArt { stages });
}

/// Follow the break: show the stage for its progress over the block being
/// broken, and nothing when nothing is.
pub fn show(
    mining: Res<Mining>,
    fine: Res<PlanetFine>,
    art: Option<Res<CrackArt>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut overlay: Query<(
        &mut Cracks,
        &mut Visibility,
        &Mesh3d,
        &mut MeshMaterial3d<StandardMaterial>,
    )>,
) {
    let Some(art) = art else {
        return;
    };
    let Ok((mut cracks, mut visibility, mesh, mut material)) = overlay.single_mut() else {
        return;
    };
    let Some((block, progress)) = mining.progress() else {
        if *visibility != Visibility::Hidden {
            *visibility = Visibility::Hidden;
            cracks.block = None;
            cracks.stage = None;
        }
        return;
    };
    if cracks.block != Some(block) {
        let record = fine
            .set
            .finest_records()
            .iter()
            .find(|record| record.metadata[3] == block.0);
        let Some(record) = record else {
            *visibility = Visibility::Hidden;
            return;
        };
        if let Some(mesh) = meshes.get_mut(&mesh.0) {
            *mesh = prism(Some((record, block.1)));
        }
        cracks.block = Some(block);
        cracks.stage = None;
    }
    let k = stage(progress);
    if cracks.stage != Some(k) {
        material.0 = art.stages[k as usize].clone();
        cracks.stage = Some(k);
    }
    *visibility = Visibility::Visible;
}

/// The overlay's geometry: the cell's prism for one layer, lifted off its
/// faces, with the terrain's UVs. `None` is an empty mesh.
fn prism(cell: Option<(&GpuCell, usize)>) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    if let Some((cell, layer)) = cell {
        let faces = faces(cell, layer);
        for [a, b, c] in faces {
            let normal = (b.0 - a.0).cross(c.0 - a.0).normalize_or_zero();
            for (p, uv) in [a, b, c] {
                positions.push(p.to_array());
                normals.push(normal.to_array());
                uvs.push(uv.to_array());
            }
        }
    }
    let count = positions.len() as u32;
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32((0..count).collect()));
    mesh
}

type Corner = (Vec3, Vec2);

/// The prism's triangles, each wound to face out: the top, the sides and the
/// bottom. A face the neighbouring blocks cover is still built; the depth
/// test hides it, as it hides the block's own.
fn faces(cell: &GpuCell, layer: usize) -> Vec<[Corner; 3]> {
    let degree = cell.degree().clamp(3, 6);
    let level = cell.metadata[0] >> 8;
    let axis = Vec3::from_slice(&cell.direction_height[..3]).normalize_or(Vec3::Y);
    let bottom = PLANET_RADIUS + column::layer_altitude(layer);
    let top = bottom + 1.0;
    let middle = axis * (bottom + 0.5);
    // The terrain's own top UV frame (`planet_surface.wgsl`, the cap).
    let tile = 1.2087 * PLANET_RADIUS / (1u32 << level) as f32;
    let reference = if axis.y.abs() > 0.95 {
        Vec3::X
    } else {
        Vec3::Y
    };
    let tangent = reference.cross(axis).normalize_or(Vec3::X);
    let bitangent = axis.cross(tangent);
    let top_uv = |ray: Vec3| {
        let local = (ray - axis) * PLANET_RADIUS;
        Vec2::new(local.dot(tangent), local.dot(bitangent)) / (1.5 * tile) + 0.5
    };
    let ray = |i: usize| Vec3::from_slice(&cell.corners[i % degree][..3]).normalize_or(axis);
    // A corner pushed out sideways, off the cell's axis, by the lift.
    let out = |r: Vec3| (r - axis * r.dot(axis)).normalize_or_zero() * LIFT_M;
    let at = |r: Vec3, radius: f32| r * radius + out(r);
    let mut triangles = Vec::with_capacity(degree * 4);
    let mut push = |mut tri: [Corner; 3]| {
        let centroid = (tri[0].0 + tri[1].0 + tri[2].0) / 3.0;
        let normal = (tri[1].0 - tri[0].0).cross(tri[2].0 - tri[0].0);
        if normal.dot(centroid - middle) < 0.0 {
            tri.swap(1, 2);
        }
        triangles.push(tri);
    };
    let lid = top + LIFT_M;
    let floor = bottom - LIFT_M;
    for i in 0..degree {
        let (a, b) = (ray(i), ray(i + 1));
        // The cap and the underside: fans about the axis.
        push([
            (axis * lid, top_uv(axis)),
            (at(a, lid), top_uv(a)),
            (at(b, lid), top_uv(b)),
        ]);
        push([
            (axis * floor, top_uv(axis)),
            (at(a, floor), top_uv(a)),
            (at(b, floor), top_uv(b)),
        ]);
        // A wall: u along the edge, v down the metre, as the terrain's walls.
        let (a0, b0, b1, a1) = (at(a, bottom), at(b, bottom), at(b, top), at(a, top));
        let (uv_a0, uv_b0, uv_b1, uv_a1) = (
            Vec2::new(0.0, 1.0),
            Vec2::new(1.0, 1.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 0.0),
        );
        push([(a0, uv_a0), (b0, uv_b0), (b1, uv_b1)]);
        push([(a0, uv_a0), (b1, uv_b1), (a1, uv_a1)]);
    }
    triangles
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};

    fn stage_image(k: u8) -> Image {
        let path = format!(
            "{}/../../assets/textures/break/stage_{k}.png",
            env!("CARGO_MANIFEST_DIR")
        );
        let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        Image::from_buffer(
            &bytes,
            ImageType::Extension("png"),
            CompressedImageFormats::NONE,
            true,
            ImageSampler::Default,
            RenderAssetUsages::default(),
        )
        .unwrap()
    }

    fn crack_pixels(image: &Image) -> Vec<bool> {
        let data = image.data.as_ref().expect("decoded pixels");
        data.chunks(4).map(|px| px[3] > 128 && px[0] < 64).collect()
    }

    /// The shipped stages are 32 px like the atlas tiles, there are ten, and
    /// each holds every crack pixel of the one before, so the block reads as
    /// one crack growing.
    #[test]
    fn the_stages_grow_and_match_the_atlas_texel() {
        let mut before: Option<Vec<bool>> = None;
        let mut counts = Vec::new();
        for k in 0..STAGES {
            let image = stage_image(k);
            assert_eq!((image.width(), image.height()), (32, 32), "stage {k}");
            let now = crack_pixels(&image);
            if let Some(before) = &before {
                for (i, (&was, &is)) in before.iter().zip(&now).enumerate() {
                    assert!(!was || is, "stage {k} lost crack pixel {i}");
                }
            }
            counts.push(now.iter().filter(|c| **c).count());
            before = Some(now);
        }
        assert!(counts.windows(2).all(|w| w[1] > w[0]), "{counts:?}");
    }

    fn hexagon() -> GpuCell {
        let axis = Vec3::Y;
        let mut cell = GpuCell {
            direction_height: [0.0, 1.0, 0.0, 0.0],
            corners: [[0.0; 4]; 6],
            metadata: [6 | (11 << 8), 0, 0, 42],
            owner_a: [0.0; 4],
            owner_b: [0.0; 4],
            floors: [0.0; 4],
            spare: [0.0; 4],
        };
        let spread = 1.4 / PLANET_RADIUS;
        for (i, corner) in cell.corners.iter_mut().enumerate() {
            let angle = i as f32 * std::f32::consts::TAU / 6.0;
            let r = (axis + Vec3::new(angle.cos(), 0.0, angle.sin()) * spread).normalize();
            *corner = [r.x, r.y, r.z, 0.0];
        }
        cell
    }

    /// A hexagon's overlay is a closed prism around exactly the layer asked
    /// for: every face faces out, and it stands off the block by the lift and
    /// no more.
    #[test]
    fn the_prism_wraps_the_layer_and_faces_out() {
        let cell = hexagon();
        let layer = 150;
        let tris = faces(&cell, layer);
        assert_eq!(tris.len(), 6 * 4);
        let bottom = PLANET_RADIUS + column::layer_altitude(layer);
        let middle = Vec3::Y * (bottom + 0.5);
        for tri in &tris {
            let normal = (tri[1].0 - tri[0].0).cross(tri[2].0 - tri[0].0);
            let centroid = (tri[0].0 + tri[1].0 + tri[2].0) / 3.0;
            assert!(normal.dot(centroid - middle) > 0.0, "a face turned inward");
            for (p, uv) in tri {
                let height = p.length() - PLANET_RADIUS;
                let lo = column::layer_altitude(layer) - LIFT_M - 1e-3;
                let hi = column::layer_altitude(layer) + 1.0 + LIFT_M + 1e-3;
                assert!((lo..=hi).contains(&height), "{height}");
                assert!(
                    (0.0..=1.0).contains(&uv.x) && (0.0..=1.0).contains(&uv.y),
                    "{uv}"
                );
            }
        }
    }
}
