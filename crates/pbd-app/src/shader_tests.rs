//! The shipped shaders parse and validate on the CPU, with naga, exactly as
//! the pipeline cache will have to compile them. Bevy reports a shader that
//! fails there as an error in the log while the app keeps running, drawing
//! nothing from that pipeline: a capture harness gets a picture with the planet
//! missing and no failing check anywhere. This is that check. Only the shaders
//! with no `#ifdef` can be validated here; the terrain's one `#import` (the
//! cloud module's weather-map read) is spliced in by `planet_surface_source`,
//! and the sky and water passes go through Bevy's preprocessor and are not.

use naga::valid::{Capabilities, ValidationFlags, Validator};

/// Parses and validates one shipped WGSL source and returns the entry points it
/// declares, so a pipeline's entry name is held against the file it names.
fn validated_entry_points(label: &str, source: &str) -> Vec<String> {
    let module = naga::front::wgsl::parse_str(source)
        .unwrap_or_else(|error| panic!("{label} must parse:\n{}", error.emit_to_string(source)));
    Validator::new(ValidationFlags::all(), Capabilities::all())
        .validate(&module)
        .unwrap_or_else(|error| {
            panic!(
                "{label} must validate:\n{}",
                error.emit_to_string_with_path(source, label)
            )
        });
    module
        .entry_points
        .iter()
        .map(|entry| entry.name.clone())
        .collect()
}

/// `planet_surface.wgsl` as Bevy's composer hands it to naga: its import of
/// the cloud module replaced by that module's own items. The terrain imports
/// only functions and names nothing it does not also define, so splicing the
/// module whole is the composition.
pub(crate) fn planet_surface_source() -> String {
    let surface = include_str!("../../../assets/shaders/planet_surface.wgsl");
    let clouds: String = include_str!("../../../assets/shaders/clouds.wgsl")
        .lines()
        .filter(|line| !line.starts_with("#define_import_path"))
        .collect::<Vec<_>>()
        .join("\n");
    let import = "#import pbd::clouds::cloud_map_smooth";
    assert!(
        surface.contains(import),
        "planet_surface.wgsl's cloud import moved; update the splice"
    );
    let shadow = "#import pbd::sun_shadow::{SunCascades, sun_shadow_soft}";
    assert!(
        surface.contains(shadow),
        "planet_surface.wgsl's shadow import moved; update the splice"
    );
    surface
        .replacen(import, &clouds, 1)
        .replacen(shadow, &sun_shadow_source(), 1)
}

/// `sun_shadow.wgsl` as naga reads it: the module's items, without the
/// composer's import path.
pub(crate) fn sun_shadow_source() -> String {
    include_str!("../../../assets/shaders/sun_shadow.wgsl")
        .lines()
        .filter(|line| !line.starts_with("#define_import_path"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// `sea.wgsl` as naga reads it: its import path is Bevy's composer's, not
/// WGSL.
pub(crate) fn sea_source() -> String {
    include_str!("../../../assets/shaders/sea.wgsl")
        .lines()
        .filter(|line| !line.starts_with("#define_import_path"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_sea_module_compiles() {
    validated_entry_points("sea.wgsl", &sea_source());
}

#[test]
fn the_planet_surface_shader_compiles_with_the_entry_points_its_pipeline_names() {
    let entries = validated_entry_points("planet_surface.wgsl", &planet_surface_source());
    for entry in ["vertex", "fragment", "shadow_fragment"] {
        assert!(
            entries.iter().any(|name| name == entry),
            "planet_surface.wgsl declares {entries:?}, and the draw pipeline asks for `{entry}`"
        );
    }
}

#[test]
fn the_visibility_shader_compiles_with_the_entry_points_its_pipelines_name() {
    let entries = validated_entry_points(
        "planet_visibility.wgsl",
        include_str!("../../../assets/shaders/planet_visibility.wgsl"),
    );
    for entry in ["clear_indirect", "compact_visible"] {
        assert!(
            entries.iter().any(|name| name == entry),
            "planet_visibility.wgsl declares {entries:?}, and a compute pipeline asks for `{entry}`"
        );
    }
}

/// The byte size naga lays a named struct out at in a shipped WGSL source, so
/// a Rust uniform can be held against the struct the shader actually declares
/// rather than against a number somebody typed after counting vec4s by hand.
pub(crate) fn wgsl_struct_size(label: &str, source: &str, name: &str) -> u32 {
    let module = naga::front::wgsl::parse_str(source)
        .unwrap_or_else(|error| panic!("{label} must parse:\n{}", error.emit_to_string(source)));
    let mut layouter = naga::proc::Layouter::default();
    layouter
        .update(module.to_ctx())
        .unwrap_or_else(|error| panic!("{label} must lay out: {error}"));
    let (handle, _) = module
        .types
        .iter()
        .find(|(_, ty)| ty.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("{label} declares no struct `{name}`"));
    layouter[handle].size
}

/// The drawn sea is the table the hulls float on, read from the uniform: the
/// water pass's vertex stage displaces by `sea_surface(view.sea, ...)` and
/// writes no wave of its own. A literal sine left in it would be a second sea
/// the physics knows nothing about.
#[test]
fn the_water_vertex_draws_the_sea_table_and_no_literal_waves() {
    // Line endings as committed, whatever the checkout made of them.
    let source = include_str!("../../../assets/shaders/water.wgsl").replace("\r\n", "\n");
    let start = source
        .find("@vertex")
        .expect("water.wgsl has a vertex stage");
    let body = &source[start..];
    let end = body.find("\n}\n").expect("the vertex stage ends");
    let body = &body[..end];
    assert!(
        body.contains("sea_surface(view.sea,"),
        "the displacement reads the table from the view uniform"
    );
    for literal in ["sin(", "cos("] {
        assert!(
            !body.contains(literal),
            "the vertex stage computes a wave of its own: {literal}"
        );
    }
}

/// `lighting-model` L1: the terrain's sun constants are the core's, so the
/// sun the CPU computes (for the sky's light) and the one each fragment does
/// are one sun.
#[test]
fn the_terrains_sun_after_the_air_is_the_cores() {
    for src in [
        include_str!("../../../assets/shaders/planet_surface.wgsl"),
        include_str!("../../../assets/shaders/field_lit.wgsl"),
    ] {
        sun_constants_match(src);
    }
}

fn sun_constants_match(src: &str) {
    let vec_of = |name: &str| -> [f32; 3] {
        let at = src
            .find(&format!("const {name} = vec3<f32>("))
            .unwrap_or_else(|| panic!("{name} is gone from planet_surface.wgsl"));
        let rest = &src[at..];
        let open = rest.find('(').unwrap() + 1;
        let close = rest.find(')').unwrap();
        let v: Vec<f32> = rest[open..close]
            .split(',')
            .map(|x| x.trim().parse().unwrap())
            .collect();
        [v[0], v[1], v[2]]
    };
    use pbd_core::sky_light::{SUN_NOW, TAU, sun_norm};
    let close = |a: [f32; 3], b: [f32; 3]| a.iter().zip(b).all(|(x, y)| (x - y).abs() < 1e-5);
    assert!(close(vec_of("SKY_TAU"), TAU));
    assert!(close(vec_of("SUN_NOW"), SUN_NOW));
    assert!(close(vec_of("SUN_NORM"), sun_norm()), "{:?}", sun_norm());
    assert!(src.contains(&format!(
        "const SKY_TAU_M = {};",
        pbd_core::sky_light::TAU_M
    )));
}

/// `wet-near-detail`: the new model's wet film draws the rain's fine detail
/// only within the reach, and skips it past there; today's model is not
/// gated.
#[test]
fn rain_detail_is_gated_near_the_eye_in_the_new_model_only() {
    let src = include_str!("../../../assets/shaders/planet_surface.wgsl");
    let gate = src
        .find("var near_detail = 1.;")
        .expect("the near-detail gate is gone from the wet branch");
    let after = &src[gate..];
    let new_only = after.find("if params.light.x > 0.5 {").unwrap();
    let fade = after
        .find("smoothstep(reach-max(params.rain_near.y,0.),reach,distance_to_camera)")
        .unwrap();
    assert!(
        new_only < fade,
        "the fade must sit inside the new model's branch"
    );
    for skipped in [
        "if rings > 0.001 && near_detail > 0.001 {",
        "pg *= near_detail;",
        "if side_w > 0.001 && near_detail > 0.001 {",
        "side_w*near_detail",
    ] {
        assert!(after.contains(skipped), "missing `{skipped}`");
    }
}
