//! A placed torch drawn as the held one (playtest-fixes design 4; owner,
//! 2026-11-25: "torch on the ground should match my torch I'm carrying").
//!
//! The terrain pass no longer draws a torch's body. Within the clutter's
//! reach each torch the field lights from is an entity: the held torch's
//! bark haft and wound rag head ([`super::light`]), with their normals made
//! smooth round the haft, lit like the terrain (`LitByField`,
//! `LitLikeTerrain`) and faded in and out (`Faded`), and the held flame's
//! three pixel cards, unlit, facing the eye, flickering on the held torch's
//! flicker seeded by the torch's place.
//!
//! The light the torch casts is the field's and the shader's, read from the
//! column record: nothing here changes it. Drawn only; no rule reads it.

use super::light;
use crate::field_light::{Faded, LitByField, LitLikeTerrain};
use crate::planet::{PLANET_RADIUS, PlanetFine, PlanetRenderFrame};
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use pbd_core::column;
use pbd_core::terrain::Material;
use std::collections::BTreeMap;

/// The held torch at the size it stands on the ground: its head where the
/// shader's head was, 0.5 to 0.7 m up.
pub const PLACED_SCALE: f32 = 1.4;
/// The most torches drawn at once, nearest first.
pub const MOST: usize = 48;
/// How long a torch takes to fade in or out, seconds (a crop's).
const FADE_S: f32 = 0.6;
/// How often the near torches are looked for, seconds.
const LOOK_S: f32 = 0.25;

/// One placed torch drawn: where it stands (planet frame) and its seed.
#[derive(Component)]
pub struct PlacedTorch {
    key: (u32, u32, u32),
    foot: Vec3,
    up: Vec3,
    seed: f32,
    leaving: bool,
}

/// A placed torch's flame: its own mesh, rebuilt each frame facing the eye.
#[derive(Component)]
pub struct PlacedFlame(Entity);

/// What the placed torches are drawn with.
#[derive(Resource)]
pub struct PlacedLooks {
    haft: Handle<Mesh>,
    head: Handle<Mesh>,
    wood: Handle<StandardMaterial>,
    rag: Handle<StandardMaterial>,
    flame: Handle<StandardMaterial>,
}

/// A held mesh made for the world: the baked shading taken out (the field
/// shades it) and the normals round the haft made smooth, so the six sides
/// read as one round stick rather than six flat faces.
fn for_the_world(mut mesh: Mesh) -> Mesh {
    use bevy::mesh::VertexAttributeValues as V;
    let positions: Vec<[f32; 3]> = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(V::Float32x3(p)) => p.clone(),
        _ => return mesh,
    };
    if let Some(V::Float32x3(normals)) = mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL) {
        for (n, p) in normals.iter_mut().zip(&positions) {
            let face = Vec3::from(*n);
            // A side (not a cap): its normal goes out from the axis at the
            // vertex, keeping the face's lean along it.
            if face.y.abs() < 0.9 {
                let out = Vec2::new(p[0], p[2]).normalize_or_zero();
                let side = Vec2::new(face.x, face.z).length();
                *n = Vec3::new(out.x * side, face.y, out.y * side)
                    .normalize_or(face)
                    .to_array();
            }
        }
    }
    let n = positions.len();
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0f32; 4]; n]);
    mesh
}

/// Build the looks, and stand one torch behind the loading screen so its
/// pipelines are built before play.
pub fn make_placed_looks(
    mut commands: Commands,
    looks: Option<Res<PlacedLooks>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    if looks.is_some() {
        return;
    }
    let (haft, head, wood, rag, flame) = light::torch_parts(&mut images);
    let lit = |texture| StandardMaterial {
        base_color_texture: Some(texture),
        perceptual_roughness: 0.92,
        ..default()
    };
    let looks = PlacedLooks {
        haft: meshes.add(for_the_world(haft)),
        head: meshes.add(for_the_world(head)),
        wood: materials.add(lit(wood)),
        rag: materials.add(lit(rag)),
        flame: materials.add(StandardMaterial {
            base_color: Color::linear_rgb(0.75, 0.75, 0.75),
            base_color_texture: Some(flame),
            alpha_mode: AlphaMode::Mask(0.5),
            unlit: true,
            fog_enabled: false,
            cull_mode: None,
            double_sided: true,
            ..default()
        }),
    };
    let seed = light::flame_mesh(Vec3::new(0.0, 0.0, -1.0), Vec3::Y, 0.01, 1.0, 0.0, 11.0);
    commands
        .spawn((
            LitByField,
            LitLikeTerrain,
            Transform::from_translation(Vec3::new(0.0, -1.0e4, 0.0)),
            Visibility::Visible,
            crate::loading::PrewarmProxy,
            Name::new("Placed torch prewarm"),
        ))
        .with_children(|root| {
            root.spawn((
                Mesh3d(looks.haft.clone()),
                MeshMaterial3d(looks.wood.clone()),
            ));
            root.spawn((
                Mesh3d(looks.head.clone()),
                MeshMaterial3d(looks.rag.clone()),
            ));
            root.spawn((
                Mesh3d(meshes.add(seed)),
                MeshMaterial3d(looks.flame.clone()),
            ));
        });
    commands.insert_resource(looks);
}

/// The torch's frame on the ground: its foot on the floor of its layer,
/// standing up the cell's axis and turned about it by its seed.
fn stand(foot: Vec3, up: Vec3, seed: f32) -> Transform {
    let rotation = Quat::from_rotation_arc(Vec3::Y, up) * Quat::from_rotation_y(seed * 6.283);
    Transform::from_translation(foot).with_rotation(rotation)
}

/// Where the flame stands on a torch, in the torch's frame.
pub fn flame_at() -> Vec3 {
    (light::flame_seat() - Vec3::Y * light::haft_bottom()) * PLACED_SCALE
}

/// The torches near the eye: every lamp spot the field lights from whose
/// cell holds a torch, within `reach_m` of `eye` (planet frame), nearest
/// first, at most [`MOST`]. Each as its key, its foot and its up.
pub fn near_torches(
    tier: &crate::planet::column::ColumnTier,
    eye: Vec3,
    reach_m: f32,
) -> Vec<((u32, u32, u32), Vec3, Vec3)> {
    let mut near: Vec<(f32, ((u32, u32, u32), Vec3, Vec3))> = tier
        .lamps()
        .iter()
        .filter_map(|spot| {
            let column = tier.columns.get(spot.slot)?;
            let layer = column::layer_at(spot.altitude)?;
            (column.material(layer) == Material::Torch).then_some(())?;
            let up = spot.direction.normalize_or_zero();
            let foot = up * (PLANET_RADIUS + column::layer_altitude(layer));
            let d = foot.distance(eye);
            (d <= reach_m).then_some((
                d,
                (
                    (
                        spot.direction.x.to_bits(),
                        spot.direction.y.to_bits(),
                        layer as u32,
                    ),
                    foot,
                    up,
                ),
            ))
        })
        .collect();
    near.sort_by(|a, b| a.0.total_cmp(&b.0));
    near.truncate(MOST);
    near.into_iter().map(|(_, t)| t).collect()
}

/// Keep the drawn torches in step with the near ones: new ones fade in,
/// gone ones fade out; and each flame faces the eye and flickers.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn show_placed(
    mut commands: Commands,
    looks: Option<Res<PlacedLooks>>,
    fine: Option<Res<PlanetFine>>,
    frame: Option<Res<PlanetRenderFrame>>,
    scatter: Option<Res<crate::config::ScatterSettings>>,
    settings: Res<super::HeldConfig>,
    time: Res<Time>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    mut torches: Query<(Entity, &mut PlacedTorch, &mut Faded)>,
    mut flames: Query<
        (
            Entity,
            &PlacedFlame,
            &Mesh3d,
            &mut Transform,
            &mut Visibility,
        ),
        Without<PlacedTorch>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut since: Local<f32>,
) {
    let Some(looks) = looks else { return };
    let dt = time.delta_secs();
    let step = dt / FADE_S;
    for (entity, torch, mut faded) in &mut torches {
        let goal = if torch.leaving { 1.0 } else { 0.0 };
        faded.0 = (faded.0 + (goal - faded.0).clamp(-step, step)).clamp(0.0, 1.0);
        if torch.leaving && faded.0 >= 1.0 {
            commands.entity(entity).despawn();
        }
    }
    let Some(eye) = cameras
        .iter()
        .find(|(_, c)| c.is_active)
        .map(|(t, _)| t.translation())
    else {
        return;
    };
    let frame = frame.as_deref();
    // The flames: each faces the eye about its torch's up.
    let s = &settings.0.light;
    let t = time.elapsed_secs();
    for (entity, flame, mesh, mut transform, mut visibility) in &mut flames {
        let Ok((_, torch, faded)) = torches.get(flame.0) else {
            commands.entity(entity).despawn();
            continue;
        };
        let at = crate::render_origin::drawn_of(frame, torch.foot)
            + Quat::from_rotation_arc(Vec3::Y, torch.up) * flame_at();
        let want = if faded.0 < 0.5 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != want {
            *visibility = want;
        }
        transform.translation = at;
        let flick = light::flicker(t, torch.seed, s.torch_flicker, s.flame_fps);
        if let Some(mesh) = meshes.get_mut(&mesh.0) {
            *mesh = light::flame_mesh(
                at - eye,
                torch.up,
                s.flame_m * PLACED_SCALE,
                flick,
                t + torch.seed,
                s.flame_fps,
            )
            .translated_by(eye - at);
        }
    }
    *since += dt;
    if *since < LOOK_S {
        return;
    }
    *since = 0.0;
    let Some(fine) = fine else { return };
    let reach = scatter.map_or(120.0, |s| s.clutter_radius_m);
    let eye_planet = crate::render_origin::local_of(frame, eye);
    let mut wanted: BTreeMap<(u32, u32, u32), (Vec3, Vec3)> =
        near_torches(&fine.set.columns, eye_planet, reach)
            .into_iter()
            .map(|(k, foot, up)| (k, (foot, up)))
            .collect();
    for (_, mut torch, _) in &mut torches {
        if torch.leaving {
            continue;
        }
        if wanted.remove(&torch.key).is_none() {
            torch.leaving = true;
        }
    }
    for (key, (foot, up)) in wanted {
        let seed = ((key.0 ^ key.1.rotate_left(13) ^ key.2) % 1000) as f32 / 1000.0;
        let mut transform = stand(foot, up, seed);
        transform.translation = crate::render_origin::drawn_of(frame, foot);
        let model = Transform::from_translation(Vec3::Y * (-light::haft_bottom() * PLACED_SCALE))
            .with_scale(Vec3::splat(PLACED_SCALE));
        let torch = commands
            .spawn((
                transform,
                Visibility::default(),
                LitByField,
                LitLikeTerrain,
                Faded(1.0),
                PlacedTorch {
                    key,
                    foot,
                    up,
                    seed,
                    leaving: false,
                },
                Name::new("placed torch"),
            ))
            .with_children(|root| {
                root.spawn((
                    Mesh3d(looks.haft.clone()),
                    MeshMaterial3d(looks.wood.clone()),
                    model,
                ));
                root.spawn((
                    Mesh3d(looks.head.clone()),
                    MeshMaterial3d(looks.rag.clone()),
                    model,
                ));
            })
            .id();
        let first = light::flame_mesh(Vec3::new(0.0, 0.0, -1.0), up, 0.01, 1.0, 0.0, 11.0);
        commands.spawn((
            Transform::from_translation(transform.translation),
            Visibility::Hidden,
            Mesh3d(meshes.add(first)),
            MeshMaterial3d(looks.flame.clone()),
            NotShadowCaster,
            // Rebuilt every frame: never culled by a stale box.
            bevy::camera::visibility::NoFrustumCulling,
            PlacedFlame(torch),
            Name::new("placed torch flame"),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The placed torch stands where the shader's did: its foot on the
    /// floor, its head 0.5 to 0.7 m up, its flame on the head.
    #[test]
    fn the_placed_torch_stands_where_the_shaders_did() {
        let foot = -light::haft_bottom() * PLACED_SCALE;
        assert!(foot > 0.0);
        let flame = flame_at();
        assert!((0.6..0.72).contains(&flame.y), "{flame:?}");
        assert!(flame.x.abs() < 1e-6 && flame.z.abs() < 1e-6);
    }

    /// The world's haft has smooth normals round the stick: two vertices at
    /// the same corner of neighbouring faces share their normal.
    #[test]
    fn the_haft_is_not_flat_shaded() {
        use bevy::mesh::VertexAttributeValues as V;
        let mut images = Assets::<Image>::default();
        let (haft, ..) = light::torch_parts(&mut images);
        let haft = for_the_world(haft);
        let (Some(V::Float32x3(p)), Some(V::Float32x3(n))) = (
            haft.attribute(Mesh::ATTRIBUTE_POSITION),
            haft.attribute(Mesh::ATTRIBUTE_NORMAL),
        ) else {
            panic!("a haft has positions and normals");
        };
        let mut shared = 0;
        for i in 0..p.len() {
            for j in (i + 1)..p.len() {
                let same = Vec3::from(p[i]).distance(Vec3::from(p[j])) < 1e-5;
                let side = n[i][1].abs() < 0.9 && n[j][1].abs() < 0.9;
                if same && side {
                    assert!(
                        Vec3::from(n[i]).distance(Vec3::from(n[j])) < 1e-4,
                        "a corner keeps one normal"
                    );
                    shared += 1;
                }
            }
        }
        assert!(shared > 0);
    }
}
