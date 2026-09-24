use super::*;
use pbd_core::vehicle::{Hulls, spec::VehicleSpecs};
use std::{collections::BTreeSet, sync::Arc};

#[test]
fn loaded_kestrel_parts_follow_physics_without_reshaping() {
    use bevy::ecs::system::RunSystemOnce;
    use pbd_core::vehicle::{CraftState, KestrelTelemetry, Telemetry};
    let mut world = World::new();
    world.init_resource::<Assets<Mesh>>();
    world.init_resource::<Assets<Image>>();
    world.init_resource::<Assets<StandardMaterial>>();
    world.init_resource::<Time>();
    world
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(0.1));
    world.init_resource::<crate::planet::PlanetRenderFrame>();
    world.init_resource::<super::super::view::VehicleView>();
    world.init_resource::<super::super::Aboard>();
    let specs = Arc::new(VehicleSpecs::default());
    let mut craft = Craft::new(
        Kind::Kestrel,
        1,
        specs.clone(),
        Hulls::new(&specs),
        pbd_core::DVec3::Y * 4800.0,
        pbd_core::DQuat::IDENTITY,
    );
    craft.occupied = true;
    let CraftState::Kestrel(state) = &mut craft.state else {
        unreachable!()
    };
    state.nacelle = 0.0;
    state.throttle = 0.6;
    let angles = [0.1, -0.2, 0.15, -0.12];
    craft.telemetry = Telemetry::Kestrel(KestrelTelemetry {
        surface_deflections: angles,
        ..default()
    });
    let original = craft.record();
    let owner = world
        .spawn((
            super::super::Vehicle::new(craft.clone()),
            Transform::IDENTITY,
            Visibility::Inherited,
        ))
        .id();
    build(&mut world, owner, &craft).unwrap();
    let count = world.resource::<Assets<Mesh>>().len();
    world.run_system_once(super::super::draw::place).unwrap();
    let mut query = world.query::<(&Name, &Transform)>();
    for (name, axis, angle) in [
        ("nacelle_left", Vec3::X, -std::f32::consts::FRAC_PI_2),
        ("rotor_right", Vec3::Y, 1.08),
        ("flaperon_left", Vec3::X, angles[0] as f32),
        ("flaperon_right", Vec3::X, angles[1] as f32),
        ("elevator", Vec3::X, angles[2] as f32),
        ("rudder", Vec3::X, angles[3] as f32),
    ] {
        let transform = query
            .iter(&world)
            .find(|(n, _)| n.as_str() == name)
            .unwrap()
            .1;
        assert!(
            transform
                .rotation
                .abs_diff_eq(Quat::from_axis_angle(axis, angle), 1e-6),
            "{name}"
        );
        assert_eq!(transform.scale, Vec3::ONE);
    }
    assert_eq!(
        world
            .get::<super::super::Vehicle>(owner)
            .unwrap()
            .craft
            .record(),
        original
    );
    assert_eq!(world.resource::<Assets<Mesh>>().len(), count);
}

#[test]
fn exported_models_match_configured_parts_pivots_axes_and_dimensions() {
    let specs: VehicleSpecs =
        ron::from_str(include_str!("../../../../../assets/config/vehicles.ron")).unwrap();
    for kind in AUTHORED {
        Model::read(*kind)
            .unwrap()
            .validate_config(*kind, &specs)
            .unwrap();
    }
}

#[test]
fn changed_geometry_requires_blender_regeneration() {
    let model = Model::read(Kind::Kestrel).unwrap();
    for change in 0..7 {
        let mut specs = VehicleSpecs::default();
        match change {
            0 => specs.kestrel.wing.panel_area_m2 += 0.5,
            1 => specs.kestrel.wing.dihedral_deg += 1.0,
            2 => specs.kestrel.rotor.radius_m += 0.1,
            3 => specs.kestrel.rotor.at[0] += 0.1,
            4 => specs.kestrel.seat.eye[1] += 0.1,
            5 => specs.kestrel.tail.aspect += 0.2,
            _ => specs.kestrel.gear[0].at[2] += 0.1,
        }
        assert!(
            model.validate_config(Kind::Kestrel, &specs).is_err(),
            "change {change} accepted stale art"
        );
    }
}

#[derive(serde::Deserialize)]
struct Atlas {
    pixels_per_metre: u32,
    width: u32,
    height: u32,
    palette: std::collections::BTreeMap<String, String>,
    swatches: Vec<String>,
    regions: Vec<Region>,
}
#[derive(serde::Deserialize)]
struct Region {
    users: Vec<RegionUser>,
    rect: [u32; 4],
    origin: [u32; 2],
    size: [u32; 2],
    padding: u32,
}

#[derive(serde::Deserialize)]
struct RegionUser {
    object: String,
}

#[derive(serde::Deserialize)]
struct SwatchManifest {
    pixels_per_metre: u32,
    swatches: Vec<Swatch>,
}

#[derive(serde::Deserialize)]
struct Swatch {
    name: String,
    prompt: String,
    generated: String,
    normalized: String,
    source_size: [u32; 2],
    size: [u32; 2],
    repeat_metres: [u32; 2],
    palette: Vec<String>,
}

#[test]
fn generated_source_swatches_have_provenance_exact_palette_scale_and_wrap_edges() {
    let root = folder(Kind::Kestrel).parent().unwrap().join("swatches");
    let manifest: SwatchManifest =
        serde_json::from_slice(&std::fs::read(root.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest.pixels_per_metre, 16);
    let decode = |path| {
        Image::from_buffer(
            &std::fs::read(path).unwrap(),
            ImageType::Extension("png"),
            CompressedImageFormats::NONE,
            true,
            ImageSampler::nearest(),
            RenderAssetUsages::default(),
        )
        .unwrap()
    };
    for s in &manifest.swatches {
        assert!(!s.prompt.is_empty());
        assert!((3..=5).contains(&s.palette.len()));
        let source = decode(root.join(&s.generated));
        assert_eq!([source.width(), source.height()], s.source_size);
        let tile = decode(root.join(&s.normalized));
        assert_eq!([tile.width(), tile.height()], s.size);
        assert_eq!(s.size, s.repeat_metres.map(|m| m * 16));
        let palette: BTreeSet<Vec<u8>> = s
            .palette
            .iter()
            .map(|h| {
                let mut c: Vec<_> = (0..6)
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
                    .collect();
                c.push(255);
                c
            })
            .collect();
        let data = tile.data.as_ref().unwrap();
        let actual: BTreeSet<Vec<u8>> = data.chunks_exact(4).map(|p| p.to_vec()).collect();
        assert!(
            actual.len() >= 2 && actual.is_subset(&palette),
            "{}",
            s.name
        );
        let pixel = |x: u32, y: u32| {
            let offset = ((y * s.size[0] + x) * 4) as usize;
            &data[offset..offset + 4]
        };
        for y in 0..s.size[1] {
            assert_eq!(
                pixel(0, y),
                pixel(s.size[0] - 1, y),
                "{} horizontal wrap",
                s.name
            );
        }
        for x in 0..s.size[0] {
            assert_eq!(
                pixel(x, 0),
                pixel(x, s.size[1] - 1),
                "{} vertical wrap",
                s.name
            );
        }
    }
    for kind in AUTHORED {
        let atlas: Atlas =
            serde_json::from_slice(&std::fs::read(folder(*kind).join("atlas.json")).unwrap())
                .unwrap();
        assert!(!atlas.swatches.is_empty());
        for name in &atlas.swatches {
            assert!(manifest.swatches.iter().any(|s| &s.name == name));
        }
    }
}

#[test]
fn exported_uvs_have_uniform_density_padded_pixel_charts_and_a_compact_palette() {
    for kind in AUTHORED {
        let model = Model::read(*kind).unwrap();
        let atlas: Atlas =
            serde_json::from_slice(&std::fs::read(folder(*kind).join("atlas.json")).unwrap())
                .unwrap();
        assert_eq!(atlas.pixels_per_metre, 16);
        let image = model.image().unwrap();
        assert_eq!((image.width(), image.height()), (atlas.width, atlas.height));
        assert_eq!(image.texture_descriptor.mip_level_count, 1);
        assert_eq!(
            model.png,
            std::fs::read(folder(*kind).join("atlas.png")).unwrap()
        );
        let palette: BTreeSet<Vec<u8>> = atlas
            .palette
            .values()
            .map(|h| {
                let mut color: Vec<u8> = (0..6)
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
                    .collect();
                color.push(255);
                color
            })
            .collect();
        assert!((12..=24).contains(&palette.len()));
        let actual: BTreeSet<Vec<u8>> = image
            .data
            .as_ref()
            .unwrap()
            .chunks_exact(4)
            .map(|p| p.to_vec())
            .collect();
        assert_eq!(actual, palette);
        let occupied: u32 = atlas.regions.iter().map(|r| r.rect[2] * r.rect[3]).sum();
        assert!(occupied as f32 / (atlas.width * atlas.height) as f32 > 0.8);
        assert!(
            image
                .data
                .as_ref()
                .unwrap()
                .chunks_exact(4)
                .all(|p| palette.contains(p))
        );
        for (i, r) in atlas.regions.iter().enumerate() {
            let [x, y, w, h] = r.rect;
            assert!(r.padding >= 2 && x + w <= atlas.width && y + h <= atlas.height);
            assert_eq!(r.origin, [x + r.padding, y + r.padding]);
            assert_eq!(
                [w, h],
                [r.size[0] + 2 * r.padding, r.size[1] + 2 * r.padding]
            );
            assert!(!r.users.is_empty());
            for user in &r.users {
                model.find(&user.object).unwrap();
            }
            if r.size[0] >= 5 && r.size[1] >= 5 {
                let mut colors = BTreeSet::new();
                for y in r.origin[1]..r.origin[1] + r.size[1] {
                    for x in r.origin[0]..r.origin[0] + r.size[0] {
                        let offset = (((atlas.height - 1 - y) * atlas.width + x) * 4) as usize;
                        colors.insert(&image.data.as_ref().unwrap()[offset..offset + 4]);
                    }
                }
                assert!(colors.len() >= 2, "unpainted resolved tile");
            }
            for other in &atlas.regions[..i] {
                let [a, b, c, d] = other.rect;
                assert!(
                    x + w <= a || a + c <= x || y + h <= b || b + d <= y,
                    "overlapping charts"
                );
            }
        }
        for node in &model.nodes {
            for primitive in &node.primitives {
                for tri in primitive.indices.chunks_exact(3) {
                    let positions: [Vec3; 3] =
                        std::array::from_fn(|i| Vec3::from(primitive.positions[tri[i] as usize]));
                    let uv: [Vec2; 3] = std::array::from_fn(|i| {
                        let [u, v] = primitive.uvs[tri[i] as usize];
                        Vec2::new(u * atlas.width as f32, (1.0 - v) * atlas.height as f32)
                    });
                    for i in 0..3 {
                        let j = (i + 1) % 3;
                        assert!(
                            (uv[i].distance(uv[j]) - 16.0 * positions[i].distance(positions[j]))
                                .abs()
                                < 0.003,
                            "{} has stretched UVs",
                            node.name
                        );
                    }
                    assert!(
                        atlas
                            .regions
                            .iter()
                            .filter(|r| r.users.iter().any(|u| u.object == node.name))
                            .any(|r| uv.iter().all(|p| {
                                p.x >= r.origin[0] as f32 - 0.001
                                    && p.y >= r.origin[1] as f32 - 0.001
                                    && p.x < (r.origin[0] + r.size[0]) as f32 + 0.001
                                    && p.y < (r.origin[1] + r.size[1]) as f32 + 0.001
                            })),
                        "{} triangle lies outside its padded charts",
                        node.name
                    );
                }
            }
        }
    }
}

#[test]
fn actual_glb_instantiates_cached_meshes_and_named_moving_parts() {
    let mut world = World::new();
    world.init_resource::<Assets<Mesh>>();
    world.init_resource::<Assets<Image>>();
    world.init_resource::<Assets<StandardMaterial>>();
    let specs = Arc::new(VehicleSpecs::default());
    for kind in AUTHORED {
        let craft = Craft::new(
            *kind,
            1,
            specs.clone(),
            Hulls::new(&specs),
            pbd_core::DVec3::Y * 4800.0,
            pbd_core::DQuat::IDENTITY,
        );
        let owner = world
            .spawn((Transform::IDENTITY, Visibility::Inherited))
            .id();
        build(&mut world, owner, &craft).unwrap();
        let meshes = world.resource::<Assets<Mesh>>().len();
        let images = world.resource::<Assets<Image>>().len();
        let other = world
            .spawn((Transform::IDENTITY, Visibility::Inherited))
            .id();
        build(&mut world, other, &craft).unwrap();
        assert_eq!(world.resource::<Assets<Mesh>>().len(), meshes);
        assert_eq!(world.resource::<Assets<Image>>().len(), images);
        let mut query = world.query::<(&Name, &super::super::draw::Part)>();
        let moving: Vec<_> = query
            .iter(&world)
            .map(|(n, _)| n.as_str().to_owned())
            .collect();
        assert!(moving.iter().any(|n| n == "nacelle_left"));
        assert!(moving.iter().any(|n| n == "flaperon_right"));
    }
}
