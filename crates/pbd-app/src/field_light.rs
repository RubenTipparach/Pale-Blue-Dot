//! What lights the things the terrain pass does not draw: the held tool and
//! hand, the ship, a fish, the float (`lamps-and-lanterns` decision 10).
//!
//! One field lights everything. The terrain pass reads the column tier's
//! light field directly; these read the same field through
//! `ColumnTier::sample` and turn it into light with the terrain's own terms
//! (`pbd_core::light::sky_fill`, `lamp_light`), so a tool in a cave is as
//! dark as the cave and a ship beside a lantern is lit on the lantern's side.

use crate::planet::{PLANET_RADIUS, PlanetContact, PlanetFine};
use bevy::camera::primitives::Aabb;
use bevy::pbr::{ExtendedMaterial, MaterialExtension};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use pbd_core::light;

/// Put on the root of anything lit PBR that should take the field: a craft,
/// a fish school, the float. Every mesh under it is given its own copy of its
/// material, extended with the field (`FieldLit`).
#[derive(Component, Default, Clone, Copy)]
pub struct LitByField;

/// The field at a mesh's bounds, as `field_lit.wgsl` reads it.
#[derive(Clone, Copy, Debug, Default, ShaderType, Reflect)]
pub struct FieldUniform {
    pub low: Vec4,
    pub high: Vec4,
    pub centre: Vec4,
    pub sun: Vec4,
    pub sky: [Vec4; 2],
    pub block: [Vec4; 2],
}

/// The extension that lights a PBR material from the field.
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct FieldLit {
    #[uniform(100)]
    pub field: FieldUniform,
}

impl MaterialExtension for FieldLit {
    fn fragment_shader() -> ShaderRef {
        "shaders/field_lit.wgsl".into()
    }
}

/// A PBR material lit by the field.
pub type FieldLitMaterial = ExtendedMaterial<StandardMaterial, FieldLit>;

/// Lights the ship, the fish and the float from the field.
pub struct FieldLightPlugin;

impl Plugin for FieldLightPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<FieldLitMaterial>::default())
            .add_systems(
                PostUpdate,
                (take_the_field, light_from_the_field)
                    .chain()
                    .after(bevy::transform::TransformSystems::Propagate),
            );
    }
}

/// Give each new mesh under a `LitByField` root its own field-lit copy of its
/// material. Its own, because each mesh is lit at its own bounds; the
/// builders share one material between parts and never need to know.
fn take_the_field(
    mut commands: Commands,
    added: Query<
        (Entity, &MeshMaterial3d<StandardMaterial>),
        Added<MeshMaterial3d<StandardMaterial>>,
    >,
    roots: Query<(), With<LitByField>>,
    parents: Query<&ChildOf>,
    standard: Res<Assets<StandardMaterial>>,
    mut lit: ResMut<Assets<FieldLitMaterial>>,
) {
    for (entity, material) in &added {
        let marked = std::iter::once(entity)
            .chain(parents.iter_ancestors(entity))
            .any(|at| roots.contains(at));
        if !marked {
            continue;
        }
        let Some(base) = standard.get(&material.0) else {
            continue;
        };
        let handle = lit.add(ExtendedMaterial {
            base: base.clone(),
            extension: FieldLit::default(),
        });
        commands
            .entity(entity)
            .remove::<MeshMaterial3d<StandardMaterial>>()
            .insert(MeshMaterial3d(handle));
    }
}

/// The bounds of a mesh in the render frame, from its local box and where it
/// is; half a metre round its origin where it has no box yet.
fn world_bounds(transform: &GlobalTransform, aabb: Option<&Aabb>) -> (Vec3, Vec3) {
    let Some(aabb) = aabb else {
        let at = transform.translation();
        return (at - Vec3::splat(0.5), at + Vec3::splat(0.5));
    };
    let (centre, half) = (Vec3::from(aabb.center), Vec3::from(aabb.half_extents));
    let mut low = Vec3::splat(f32::MAX);
    let mut high = Vec3::splat(f32::MIN);
    for i in 0..8 {
        let sign = Vec3::new(
            if i & 1 == 0 { -1.0 } else { 1.0 },
            if i & 2 == 0 { -1.0 } else { 1.0 },
            if i & 4 == 0 { -1.0 } else { 1.0 },
        );
        let corner = transform.transform_point(centre + half * sign);
        low = low.min(corner);
        high = high.max(corner);
    }
    (low, high)
}

/// Sample the field at every field-lit mesh's eight corners, each frame
/// (`lamps-and-lanterns` decision 2), and hand them to its material.
#[allow(clippy::too_many_arguments)]
fn light_from_the_field(
    fine: Option<Res<PlanetFine>>,
    contact: Option<Res<PlanetContact>>,
    sun: Option<Res<crate::sky::Sun>>,
    frame: Option<Res<crate::planet::PlanetRenderFrame>>,
    meshes: Query<(
        &GlobalTransform,
        Option<&Aabb>,
        &MeshMaterial3d<FieldLitMaterial>,
    )>,
    mut lit: ResMut<Assets<FieldLitMaterial>>,
) {
    let centre = frame
        .map(|frame| frame.center.as_vec3())
        .unwrap_or(Vec3::ZERO);
    let toward_sun = sun.map(|sun| sun.direction()).unwrap_or(Vec3::Y);
    for (transform, aabb, material) in &meshes {
        let (low, high) = world_bounds(transform, aabb);
        let mut field = FieldUniform {
            low: low.extend(0.0),
            high: high.extend(0.0),
            centre: centre.extend(0.0),
            sun: toward_sun.extend(0.0),
            ..default()
        };
        for i in 0..8 {
            let corner = Vec3::new(
                if i & 1 == 0 { low.x } else { high.x },
                if i & 2 == 0 { low.y } else { high.y },
                if i & 4 == 0 { low.z } else { high.z },
            );
            let (sky, block) = match (&fine, &contact) {
                (Some(fine), Some(contact)) => field_at(fine, contact, corner - centre),
                _ => (1.0, 0.0),
            };
            field.sky[i / 4][i % 4] = sky;
            field.block[i / 4][i % 4] = block;
        }
        let Some(current) = lit.get(&material.0).map(|m| m.extension.field) else {
            continue;
        };
        let moved = (current.low - field.low).abs().max_element() > 1e-3
            || (current.high - field.high).abs().max_element() > 1e-3
            || (current.sun - field.sun).abs().max_element() > 1e-4
            || (0..2).any(|k| {
                (current.sky[k] - field.sky[k]).abs().max_element() > 1e-3
                    || (current.block[k] - field.block[k]).abs().max_element() > 1e-3
            });
        if moved && let Some(material) = lit.get_mut(&material.0) {
            material.extension.field = field;
        }
    }
}

/// Both channels of the field at a point in the planet's frame, 0..1
/// `(sky, block)`. Off the tier, or while the contact serves another set, it
/// answers the open sky and no lamp, which is what the far terrain is drawn
/// with.
pub fn field_at(fine: &PlanetFine, contact: &PlanetContact, point: Vec3) -> (f32, f32) {
    let Some(direction) = point.try_normalize() else {
        return (1.0, 0.0);
    };
    if !contact.serves(&fine.set) {
        return (1.0, 0.0);
    }
    let record = contact.finest_cell(direction);
    fine.set
        .columns
        .sample(record, direction, point.length() - PLANET_RADIUS)
}

/// The light a sample lays over a white surface, linear RGB: the sky's fill
/// for how much of it is day where the point is, and the lamps' warm light.
pub fn light_of((sky, block): (f32, f32), daylight: f32) -> Vec3 {
    let fill = light::sky_fill(sky, daylight);
    let lamp = light::lamp_light(block);
    Vec3::splat(fill) + Vec3::from_array(lamp)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `field_lit.wgsl` carries the terrain's light constants, and they are
    /// the core's: a ship and the ground it stands on are lit by one set of
    /// numbers.
    #[test]
    fn the_field_lit_shader_carries_the_terrain_light_constants() {
        let shader = include_str!("../../../assets/shaders/field_lit.wgsl");
        for line in [
            format!("const AMBIENT_FLOOR: f32 = {:.2};", light::AMBIENT_FLOOR),
            format!("const NIGHT_FILL: f32 = {:.2};", light::NIGHT_FILL),
            format!(
                "const TORCH_TINT: vec3<f32> = vec3<f32>({:.2}, {:.2}, {:.2});",
                light::TORCH_TINT[0],
                light::TORCH_TINT[1],
                light::TORCH_TINT[2]
            ),
            format!("const TORCH_GAIN: f32 = {:.2};", light::TORCH_GAIN),
            "    let g = f * (2.0 - f);\n    return g * g;".to_string(),
            "let daylight = smoothstep(-0.13, 0.20, dot(up, field.sun.xyz));".to_string(),
            "@binding(100) var<uniform> field: Field;".to_string(),
        ] {
            assert!(
                shader.contains(&line),
                "field_lit.wgsl should carry `{line}`"
            );
        }
    }

    /// A box's bounds in the render frame hold all eight of its corners,
    /// turned and moved.
    #[test]
    fn a_turned_box_is_bounded_by_its_corners() {
        let aabb = Aabb::from_min_max(Vec3::new(-1.0, -0.5, -2.0), Vec3::new(1.0, 0.5, 2.0));
        let transform = GlobalTransform::from(
            Transform::from_xyz(10.0, 0.0, 0.0).with_rotation(Quat::from_rotation_y(0.5)),
        );
        let (low, high) = world_bounds(&transform, Some(&aabb));
        for corner in [
            Vec3::new(-1.0, -0.5, -2.0),
            Vec3::new(1.0, 0.5, 2.0),
            Vec3::new(1.0, -0.5, -2.0),
        ] {
            let at = transform.transform_point(corner);
            assert!(
                at.cmpge(low - 1e-4).all() && at.cmple(high + 1e-4).all(),
                "{at}"
            );
        }
    }

    /// The open sky at noon is the full day, unchanged; a sealed cave is the
    /// floor whatever the hour; night in the open is the night's fill; and a
    /// torch beside it adds warm light, more red than blue.
    #[test]
    fn the_sky_the_cave_the_night_and_a_torch() {
        assert_eq!(light_of((1.0, 0.0), 1.0), Vec3::ONE, "noon in the open");
        let cave = light_of((0.0, 0.0), 1.0);
        assert_eq!(cave, Vec3::splat(light::AMBIENT_FLOOR), "a cave at noon");
        assert_eq!(light_of((0.0, 0.0), 0.0), cave, "and at midnight");
        let night = light_of((1.0, 0.0), 0.0);
        assert_eq!(
            night,
            Vec3::splat(light::NIGHT_FILL),
            "midnight in the open"
        );
        let torch = light_of((1.0, 14.0 / 15.0), 0.0);
        assert!(torch.x > night.x * 5.0, "a torch lights the night: {torch}");
        assert!(torch.x > torch.z, "warm: {torch}");
    }
}
