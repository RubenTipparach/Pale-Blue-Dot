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
use bevy::render::storage::ShaderStorageBuffer;
use bevy::shader::ShaderRef;
use pbd_core::light;

/// Put on the root of anything lit PBR that should take the field: a craft,
/// a fish school, the float. Every mesh under it is given its own copy of its
/// material, extended with the field (`FieldLit`).
#[derive(Component, Default, Clone, Copy)]
pub struct LitByField;

/// Put beside [`LitByField`] on a root whose meshes are lit as the terrain
/// is, by the terrain's own fill and sun rather than Bevy's picture: a town's
/// pieces (`sun-shadows` decision 7), which are the ground's own kind of
/// thing and stand beside it.
#[derive(Component, Default, Clone, Copy)]
pub struct LitLikeTerrain;

/// The most lights a building's rooms carry (`cities-in-the-world` decision
/// 7a): a hearth, a stair's sconces and its candles. `field_lit.wgsl`'s
/// arrays are the same length, and a test holds them together.
pub const MAX_ROOM_LIGHTS: usize = 24;

/// What burns in a building's rooms, on each of its room meshes: the lights
/// its faces take on top of the field's (`cities-in-the-world` decision 7a).
#[derive(Component, Clone, Debug, Default)]
pub struct RoomLights(pub Vec<pbd_core::settlement::pieces::RoomLight>);

impl RoomLights {
    /// As `field_lit.wgsl` reads them, in the planet's frame: where and how
    /// far, colour times power and whether only by night, and the band of
    /// height each lights. The fires first, so a building with more candles
    /// than room keeps its hearth and sconces.
    pub fn pack(&self, field: &mut FieldUniform) {
        let mut lights: Vec<_> = self.0.iter().collect();
        lights.sort_by_key(|l| !l.kind.all_day());
        let n = lights.len().min(MAX_ROOM_LIGHTS);
        for (i, l) in lights.into_iter().take(n).enumerate() {
            let [r, g, b] = l.kind.colour();
            field.lights[i] = l.at.extend(l.kind.reach_m());
            field.light_colour[i] = Vec4::new(
                r * l.power,
                g * l.power,
                b * l.power,
                if l.kind.all_day() { 0.0 } else { 1.0 },
            );
            field.light_span[i] = Vec4::new(-l.below_m, l.above_m, 0.0, 0.0);
        }
        field.look.w = n as f32;
    }
}

/// How far a field-lit thing has faded out, 0 drawn whole to 1 gone, on it
/// or on its parent: a town standing up or dropped at the edge of its range
/// (`cities-in-the-world` slice 4a). It is drawn through the terrain's own
/// screen-door mask, so it never appears or vanishes in one frame.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct Faded(pub f32);

/// How a room's faces are lit where the field cannot say (`sun-shadows`
/// decision 7): the voxel field has never heard of a house's walls.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct SkyShare {
    /// The share of the sky that reaches it, 0..1.
    pub sky: f32,
    /// How much of the sun that came in by its door and windows bounces
    /// round it, warm, times the shader's `ROOM_BOUNCE`: zero outdoors.
    pub bounce: f32,
}

/// The field at a mesh's bounds, as `field_lit.wgsl` reads it.
#[derive(Clone, Copy, Debug, Default, ShaderType, Reflect)]
pub struct FieldUniform {
    pub low: Vec4,
    pub high: Vec4,
    /// The planet's centre in the render frame; w how far it has faded out
    /// ([`Faded`]).
    pub centre: Vec4,
    pub sun: Vec4,
    pub sky: [Vec4; 2],
    pub block: [Vec4; 2],
    /// x one where it is lit as the terrain is ([`LitLikeTerrain`]); y the
    /// share of the sky that reaches it and z the sun bounced round a room
    /// ([`SkyShare`]; one and zero outdoors); w how many room lights.
    pub look: Vec4,
    /// A room's own lights ([`RoomLights::pack`]): where (planet frame) and
    /// how far; colour times power, w one for a candle; the band of height
    /// round each that it lights.
    pub lights: [Vec4; MAX_ROOM_LIGHTS],
    pub light_colour: [Vec4; MAX_ROOM_LIGHTS],
    pub light_span: [Vec4; MAX_ROOM_LIGHTS],
}

/// The extension that lights a PBR material from the field, and shades it
/// with the sun's cascades (`sun-shadows` decision 4).
#[derive(Asset, AsBindGroup, Reflect, Debug, Clone, Default)]
pub struct FieldLit {
    #[uniform(100)]
    pub field: FieldUniform,
    #[texture(101, sample_type = "depth", dimension = "2d_array")]
    #[sampler(102, sampler_type = "comparison")]
    pub sun_map: Handle<Image>,
    #[storage(103, read_only)]
    pub sun_cascades: Handle<ShaderStorageBuffer>,
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
            )
            .add_systems(Update, fill_follows_the_day);
    }
}

/// Give each new mesh under a `LitByField` root its own field-lit copy of its
/// material. Its own, because each mesh is lit at its own bounds; the
/// builders share one material between parts and never need to know.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn take_the_field(
    mut commands: Commands,
    added: Query<
        (
            Entity,
            &MeshMaterial3d<StandardMaterial>,
            Option<&RoomLights>,
        ),
        Added<MeshMaterial3d<StandardMaterial>>,
    >,
    roots: Query<(), With<LitByField>>,
    terrain_like: Query<(), With<LitLikeTerrain>>,
    parents: Query<&ChildOf>,
    standard: Res<Assets<StandardMaterial>>,
    shadows: Option<Res<crate::planet::shadow::SunShadowMaps>>,
    mut lit: ResMut<Assets<FieldLitMaterial>>,
) {
    for (entity, material, room_lights) in &added {
        let marked = std::iter::once(entity)
            .chain(parents.iter_ancestors(entity))
            .any(|at| roots.contains(at));
        if !marked {
            continue;
        }
        let Some(base) = standard.get(&material.0) else {
            continue;
        };
        // A flame draws its own light, unlit.
        if base.unlit {
            continue;
        }
        let like_terrain = std::iter::once(entity)
            .chain(parents.iter_ancestors(entity))
            .any(|at| terrain_like.contains(at));
        let mut field = FieldUniform {
            look: Vec4::new(if like_terrain { 1.0 } else { 0.0 }, 1.0, 0.0, 0.0),
            ..default()
        };
        if let Some(room_lights) = room_lights {
            room_lights.pack(&mut field);
        }
        let handle = lit.add(ExtendedMaterial {
            base: base.clone(),
            extension: FieldLit {
                field,
                sun_map: shadows.as_ref().map(|s| s.map.clone()).unwrap_or_default(),
                sun_cascades: shadows
                    .as_ref()
                    .map(|s| s.cascades.clone())
                    .unwrap_or_default(),
            },
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
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn light_from_the_field(
    fine: Option<Res<PlanetFine>>,
    contact: Option<Res<PlanetContact>>,
    sun: Option<Res<crate::sky::Sun>>,
    frame: Option<Res<crate::planet::PlanetRenderFrame>>,
    meshes: Query<(
        &GlobalTransform,
        Option<&Aabb>,
        &MeshMaterial3d<FieldLitMaterial>,
        Option<&SkyShare>,
        Option<&ChildOf>,
    )>,
    faded: Query<&Faded>,
    mut lit: ResMut<Assets<FieldLitMaterial>>,
) {
    let centre = frame
        .map(|frame| frame.center.as_vec3())
        .unwrap_or(Vec3::ZERO);
    let toward_sun = sun.map(|sun| sun.direction()).unwrap_or(Vec3::Y);
    for (transform, aabb, material, share, parent) in &meshes {
        let (low, high) = world_bounds(transform, aabb);
        let gone = parent
            .and_then(|p| faded.get(p.parent()).ok())
            .map_or(0.0, |f| f.0.clamp(0.0, 1.0));
        let mut field = FieldUniform {
            low: low.extend(0.0),
            high: high.extend(0.0),
            centre: centre.extend(gone),
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
        field.look = current.look;
        field.lights = current.lights;
        field.light_colour = current.light_colour;
        field.light_span = current.light_span;
        field.look.y = share.map_or(1.0, |s| s.sky);
        field.look.z = share.map_or(0.0, |s| s.bounce);
        let moved = (current.low - field.low).abs().max_element() > 1e-3
            || current.look != field.look
            || current.centre.w != field.centre.w
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

/// Bevy's own ambient, what lights a PBR face turned from the sun, is the
/// terrain's cap fill for the hour where the camera is (`sun-shadows`
/// decision 7): a shaded wall of a ship is lit as a shaded cliff is, not by
/// Bevy's default. Divided by the camera's exposure, which Bevy multiplies
/// every light by.
fn fill_follows_the_day(
    sun: Option<Res<crate::sky::Sun>>,
    frame: Option<Res<crate::planet::PlanetRenderFrame>>,
    cameras: Query<(&Camera, &GlobalTransform, Option<&bevy::camera::Exposure>), With<Camera3d>>,
    ambient: Option<ResMut<GlobalAmbientLight>>,
) {
    let (Some(sun), Some(frame), Some(mut ambient)) = (sun, frame, ambient) else {
        return;
    };
    let Some((_, at, exposure)) = cameras.iter().find(|(camera, ..)| camera.is_active) else {
        return;
    };
    let up = (at.translation().as_dvec3() - frame.center).as_vec3();
    let fill = light::sky_fill(1.0, sun.clock.daylight(up));
    let [r, g, b] = light::SKY_FILL;
    let brightness = fill / exposure.copied().unwrap_or_default().exposure();
    if (ambient.brightness - brightness).abs() > 1e-3 * brightness.max(1.0) {
        ambient.color = Color::linear_rgb(r, g, b);
        ambient.brightness = brightness;
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

    /// A fading town is drawn through the terrain's own mask
    /// (`cities-in-the-world` slice 4a), so the two dissolve alike.
    #[test]
    fn a_fading_town_takes_the_terrains_mask() {
        let town = include_str!("../../../assets/shaders/field_lit.wgsl");
        let terrain = include_str!("../../../assets/shaders/planet_surface.wgsl");
        let body = |s: &str| {
            let i = s.find("fn bayer4").expect("a bayer4");
            let j = s[i..].find("\n}\n").expect("its end");
            s[i..i + j].to_string()
        };
        assert_eq!(body(town), body(terrain));
        assert!(town.contains("if bayer4(in.position.xy) < field.centre.w {"));
    }

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
            "let daylight = smoothstep(-0.13, 0.20, elevation);".to_string(),
            // `sun-shadows` decision 5: the sun ends at the horizon, the
            // terrain's own curve for its direct term.
            "let sunlight = smoothstep(-0.0145, 0.02, elevation);".to_string(),
            // Decision 14 and `sun-shadows` decision 4: the sun only where it
            // is up, the sky reaches and the cascades see it.
            "let sun_up = sunlight * sky * shadow;".to_string(),
            // `sun-shadows` decision 7: a town is lit as the terrain is, by
            // its numbers.
            "const SUN_TINT: vec3<f32> = vec3<f32>(1.12, 1.03, 0.87);".to_string(),
            "const WALL_FILL: vec3<f32> = vec3<f32>(0.30, 0.32, 0.34);".to_string(),
            "const WALL_NIGHT: f32 = 0.20;".to_string(),
            "const WALL_GAIN: f32 = 0.95;".to_string(),
            format!(
                "const SKY_FILL: vec3<f32> = vec3<f32>({:.2}, {:.2}, {:.2});",
                light::SKY_FILL[0],
                light::SKY_FILL[1],
                light::SKY_FILL[2]
            ),
            "let ambient = base * SKY_FILL * max(AMBIENT_FLOOR, mix(NIGHT_FILL, 1.0, daylight) * sky);"
                .to_string(),
            "out.color.rgb * sun_up + ambient * (1.0 - sun_up) + lamp".to_string(),
            "@binding(100) var<uniform> field: Field;".to_string(),
            // `cities-in-the-world` decision 7a: a building's own lights, as
            // many as the uniform carries, lit the mockup's way.
            format!("lights: array<vec4<f32>, {MAX_ROOM_LIGHTS}>,"),
            format!("light_colour: array<vec4<f32>, {MAX_ROOM_LIGHTS}>,"),
            format!("light_span: array<vec4<f32>, {MAX_ROOM_LIGHTS}>,"),
            format!("let count = min(u32(field.look.w), {MAX_ROOM_LIGHTS}u);"),
            "q * q * (0.3 + 0.7 * max(facing, 0.0)) * d * d / (d * d + 0.36)".to_string(),
            "let fire = 0.9 + 0.3 * night;".to_string(),
            "let candle = 1.8 * clamp((night - 0.25) / 0.35, 0.0, 1.0);".to_string(),
        ] {
            assert!(
                shader.contains(&line),
                "field_lit.wgsl should carry `{line}`"
            );
        }
        // The fill's colour is the terrain's cap fill, which that shader
        // writes without spaces.
        let terrain = include_str!("../../../assets/shaders/planet_surface.wgsl");
        let fill = format!(
            "var fill = vec3({:.2},{:.2},{:.2});",
            light::SKY_FILL[0],
            light::SKY_FILL[1],
            light::SKY_FILL[2]
        );
        assert!(
            terrain.contains(&fill),
            "planet_surface.wgsl should carry `{fill}`"
        );
        // And its wall's fill, night and gain, and its sun's tint and curve,
        // which a town takes as its own.
        for line in [
            "fill = vec3(0.30,0.32,0.34);",
            "night = 0.20;",
            "gain = 0.95;",
            "vec3(1.12,1.03,0.87)*direct",
            "let sunlight = smoothstep(-0.0145,0.02,sun_elevation);",
        ] {
            assert!(
                terrain.contains(line),
                "planet_surface.wgsl should carry `{line}`"
            );
        }
    }

    /// `sun-shadows` decision 7: Bevy's ambient, what lights a PBR face
    /// turned from the sun, is the terrain's cap fill for the hour where the
    /// camera stands: the whole of it at noon, the night's share at midnight.
    #[test]
    fn a_face_turned_from_the_sun_takes_the_skys_fill() {
        let sun = crate::sky::Sun::default();
        let overhead = sun.direction();
        let mut world = World::new();
        world.insert_resource(sun);
        world.insert_resource(crate::planet::PlanetRenderFrame::default());
        world.insert_resource(GlobalAmbientLight::default());
        let camera = world
            .spawn((
                Camera3d::default(),
                GlobalTransform::from_translation(overhead * PLANET_RADIUS),
            ))
            .id();
        let mut follow = IntoSystem::into_system(fill_follows_the_day);
        follow.initialize(&mut world);
        follow.run((), &mut world).unwrap();
        let exposure = bevy::camera::Exposure::default().exposure();
        let at_noon = world.resource::<GlobalAmbientLight>().clone();
        assert!(
            (at_noon.brightness * exposure - 1.0).abs() < 1e-4,
            "{at_noon:?}"
        );
        assert_eq!(
            at_noon.color.to_linear().to_f32_array_no_alpha(),
            light::SKY_FILL
        );
        world
            .entity_mut(camera)
            .insert(GlobalTransform::from_translation(-overhead * PLANET_RADIUS));
        follow.run((), &mut world).unwrap();
        let at_midnight = world.resource::<GlobalAmbientLight>().brightness * exposure;
        assert!(
            (at_midnight - light::NIGHT_FILL).abs() < 1e-4,
            "{at_midnight}"
        );
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
