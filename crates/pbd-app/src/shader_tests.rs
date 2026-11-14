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
    for entry in [
        "vertex",
        "fragment",
        "shadow_fragment",
        // `lighting-perf` 2.8: the terrain that leaves its traces, and the
        // half-size pass that runs them (`planet_reflect.rs`).
        "fragment_split",
        "reflect_gather",
    ] {
        assert!(
            entries.iter().any(|name| name == entry),
            "planet_surface.wgsl declares {entries:?}, and the draw pipeline asks for `{entry}`"
        );
    }
}

/// `reflect_compose.wgsl` as Bevy's composer hands it to naga: its import of
/// the full-screen vertex's output replaced by that struct.
pub(crate) fn reflect_compose_source() -> String {
    let source = include_str!("../../../assets/shaders/reflect_compose.wgsl");
    let import = "#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput";
    assert!(
        source.contains(import),
        "reflect_compose.wgsl's import moved; update the splice"
    );
    source.replacen(
        import,
        "struct FullscreenVertexOutput { @builtin(position) position: vec4<f32>, @location(0) uv: vec2<f32> }",
        1,
    )
}

#[test]
fn the_reflection_compose_compiles_with_the_entry_point_its_pipeline_names() {
    let entries = validated_entry_points("reflect_compose.wgsl", &reflect_compose_source());
    assert!(entries.iter().any(|name| name == "fragment"), "{entries:?}");
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

/// The lens drops' section of `water.wgsl` (`lens-drop-refraction`): the drop
/// field, its mapping from the lens's uv and where a pixel looks through it,
/// from the "Rain on glass" banner to the refraction that samples the scene.
/// It reads only the view's `lens`, `screen` and `mist_look` lanes, so with a
/// stand-in uniform for those it is WGSL that naga reads on its own and a GPU
/// test runs (`lens_gpu_tests`). The stand-in's `probe` lane is the test's.
pub(crate) fn lens_drops_source() -> String {
    let water = include_str!("../../../assets/shaders/water.wgsl").replace("\r\n", "\n");
    let start = water
        .find("// ---- Rain on glass")
        .expect("water.wgsl's lens drops section moved; update the slice");
    let end = start
        + water[start..]
            .find("// `wipe` receives")
            .expect("water.wgsl's refraction through the drops moved; update the slice");
    format!(
        "struct LensView {{ lens: vec4<f32>, screen: vec4<f32>, mist_look: vec4<f32>, probe: vec4<f32> }}\n\
         @group(0) @binding(0) var<uniform> view: LensView;\n{}",
        &water[start..end]
    )
}

/// The lens drops read no lane of the view but the three the stand-in has,
/// and the shipped lens samples the scene where `rd_lens` says, toward the
/// drop's centre: `uv + offset`, never Tenebris's `uv - n`.
#[test]
fn the_lens_drops_compile_on_their_own_and_the_lens_looks_where_they_say() {
    validated_entry_points("water.wgsl's lens drops", &lens_drops_source());
    let water = include_str!("../../../assets/shaders/water.wgsl").replace("\r\n", "\n");
    let start = water
        .find("fn refract_through_drops(")
        .expect("the refraction is gone");
    let body = &water[start..start + water[start..].find("\n}\n").unwrap()];
    assert!(body.contains("let lens = rd_lens(drop_uv,t,l0,l1,l2);"));
    assert!(body.contains("clamp(uv+lens.offset,vec2(0.),vec2(1.))"));
    assert!(!body.contains("uv-n"), "Tenebris's outward sample is back");
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

/// The partition rule (`detail-fade`, `distance-lod-fade`): the visibility
/// pass lists a cell for a partition by it and the surface pass builds and
/// keeps the cell's pixels by it, so the two copies are one rule only while
/// they are the same text. A landing's dissolve draws each partition from its
/// own set's records on that agreement (`two-set-dissolve`).
#[test]
fn the_partition_rule_is_the_same_in_both_passes() {
    let visibility =
        include_str!("../../../assets/shaders/planet_visibility.wgsl").replace("\r\n", "\n");
    let surface = include_str!("../../../assets/shaders/planet_surface.wgsl").replace("\r\n", "\n");
    for name in [
        "fn partition_of(",
        "fn band_t(",
        "fn band_cos_in(",
        "fn owner_fine_in(",
        "fn covered_by_finer_in(",
    ] {
        let body = |source: &str| {
            let start = source.find(name).expect("the function is defined");
            let end = source[start..].find("\n}\n").expect("the function ends");
            source[start..start + end].to_string()
        };
        assert_eq!(body(&visibility), body(&surface), "{name} differs");
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

/// `dusk-sky`: seen from the ground the dome is lit on the ground's twilight,
/// not through the small planet's shadow, which kept high air lit to 34
/// degrees under and drew its edge across the sky.
#[test]
fn the_sky_near_the_ground_keeps_the_grounds_twilight() {
    let src = include_str!("../../../assets/shaders/sky_atmosphere.wgsl");
    for line in [
        "let twilight = smoothstep(-0.12,0.0,sun_height);",
        "let near_ground = 1.0-smoothstep(shell*0.3,shell,altitude);",
        "let visible_sun = mix(sun_visibility(p,sun),twilight,near_ground);",
    ] {
        assert!(src.contains(line), "missing `{line}`");
    }
    // The ground's sky light leaves over the same range (`sun_high`).
    let core = include_str!("../../pbd-core/src/sky_light.rs");
    assert!(
        core.contains("let k = smooth(-0.12, 0.0, e)"),
        "the ground's twilight moved; move the sky's with it"
    );
}

/// `water-lit`: the water's body and murk are albedos lit by the light on open
/// water, not colours drawn as they are under a day/night floor.
#[test]
fn the_water_is_lit_not_luminous() {
    let src = include_str!("../../../assets/shaders/water.wgsl");
    for line in [
        "return view.deep_color.rgb*view.water_light.rgb*exp(-absorption*max(eye_depth,0.0));",
        "let transmitted = mix(albedo*water_lit,scene,exp(-absorption*path_length));",
        "var reflected = water_sky_radiance(water_sky_frame(reflected_ray,radial,sun_direction));",
    ] {
        assert!(src.contains(line), "missing `{line}`");
    }
    assert!(
        !src.contains("view.absorption.w"),
        "the night floor is gone"
    );
}

/// One function's body in a WGSL source, from `fn name(` to its closing brace
/// at the start of a line.
fn wgsl_fn_body<'a>(src: &'a str, name: &str) -> &'a str {
    let start = src
        .find(&format!("fn {name}("))
        .unwrap_or_else(|| panic!("`fn {name}` is gone"));
    let end = src[start..]
        .find("\n}\n")
        .unwrap_or_else(|| panic!("`fn {name}` does not end"));
    &src[start..start + end]
}

/// `puff-haze`: each puff is hazed to its own distance in its own pass, writes
/// its hazed light and its own coverage to the history and its hazed coverage
/// to the puff cover, and the composite takes that cover and hazes only the
/// march. Hazed once for the texel at the nearest puff's distance, a near
/// puff's edge drew an outline over the puff behind it.
#[test]
fn each_puff_is_hazed_once_to_its_own_distance() {
    let src = include_str!("../../../assets/shaders/water.wgsl").replace("\r\n", "\n");
    assert!(src.contains("@group(3) @binding(16) var cloud_puff_cover: texture_2d<f32>;"));
    assert!(
        src.contains("@location(2) cover: vec4<f32>,"),
        "PuffOut writes the cover"
    );

    let puff = wgsl_fn_body(&src, "cloud_puff_fragment");
    let discard = puff
        .find("if (a < 0.004) { discard; }")
        .expect("the discard is on the plain coverage");
    let seen = puff
        .find("let seen = a*cloud_air(eye, look, dist);")
        .expect("the air is measured to this puff, along this ray");
    assert!(
        discard < seen,
        "the distances are written wherever the puff is"
    );
    for line in [
        "out.cloud = vec4<f32>(lit*seen, a);",
        "out.cover = vec4<f32>(seen, 0.0, 0.0, a);",
    ] {
        assert!(puff.contains(line), "missing `{line}`");
    }

    let upsampled = wgsl_fn_body(&src, "cloud_upsampled");
    assert!(upsampled.contains(
        "if (view.cloud_puff_fade.w > 0.5) { cloud.w = textureLoad(cloud_puff_cover, texel, 0).r; }"
    ));

    let composite = wgsl_fn_body(&src, "clouds");
    let gate = composite
        .find("if (view.cloud_puff_fade.w < 0.5) {")
        .expect("the composite hazes the march alone");
    let air = composite
        .find("air = cloud_air(eye, ray.direction,")
        .expect("the march's haze is kept");
    assert!(gate < air);
    assert!(composite.contains("let cloud = upsampled*air;"));
    assert!(
        !composite.contains("upsampled*cloud_air("),
        "the composite must not haze the puffs a second time"
    );
}

/// `puff-haze` design 3a, mirrored: what the puff pass's blend states and the
/// composite make of a near puff A over a far puff B, against what the old
/// one-distance haze made. The puff pass blends both colour targets
/// premultiplied "over" (`dst' = src + (1 - src.a) dst`); the fragment writes
/// `(lit*seen, a)` and `(seen, 0, 0, a)` with `seen = a*T`; the composite lays
/// `S (1 - cover) + light` over the scene.
#[test]
fn the_puff_blend_is_the_air_over_each_puff_with_no_step() {
    // The shipped haze from the spawn, 25 degrees up (design 2): a puff
    // 800 m out keeps 0.50, one 1600 m out 0.32. Sky and puff light are
    // brightness on a 0..1 scale; the far puff is whole.
    let (sky, lit_a, lit_b, air_a, air_b, a_b) = (0.25, 0.9, 0.9, 0.50, 0.32, 1.0);
    // A puff writes nothing below the discard's coverage.
    let discard = 0.004;
    let over = |dst: f32, src: f32, src_a: f32| src + (1.0 - src_a) * dst;
    let new = |a_a: f32| -> f32 {
        let a_a = if a_a < discard { 0.0 } else { a_a };
        // Far to near, as the instances are sorted.
        let light = over(
            over(0.0, lit_b * a_b * air_b, a_b),
            lit_a * a_a * air_a,
            a_a,
        );
        let cover = over(over(0.0, a_b * air_b, a_b), a_a * air_a, a_a);
        sky * (1.0 - cover) + light
    };
    let old = |a_a: f32| -> f32 {
        let a_a = if a_a < discard { 0.0 } else { a_a };
        // One haze for the texel, at the nearest distance any puff wrote.
        let air = if a_a > 0.0 { air_a } else { air_b };
        let light = over(lit_b * a_b, lit_a * a_a, a_a) * air;
        let cover = over(a_b, a_a, a_a) * air;
        sky * (1.0 - cover) + light
    };
    let b_alone = sky * (1.0 - a_b * air_b) + lit_b * a_b * air_b;
    let a_alone = sky * (1.0 - air_a) + lit_a * air_a;

    // At the near puff's edge, just inside and just outside its footprint.
    let (inside, outside) = (discard * 1.01, discard * 0.99);
    assert!(
        (new(inside) - new(outside)).abs() < 0.002,
        "no step at the edge"
    );
    assert!((new(outside) - b_alone).abs() < 1e-6);
    assert!(
        (old(inside) - old(outside)).abs() > 0.1,
        "the old haze stepped there: {} against {}",
        old(inside),
        old(outside)
    );
    // Inside a whole near puff it is the near puff alone, as the old haze
    // drew it, and nothing of the far one comes through.
    assert!((new(1.0) - a_alone).abs() < 1e-6);
    assert!((new(1.0) - old(1.0)).abs() < 1e-6);
    // And from the edge inward it moves smoothly between the two.
    let mut last = new(inside);
    for k in 1..=100 {
        let next = new(inside + (1.0 - inside) * k as f32 / 100.0);
        assert!((next - last).abs() < 0.01, "a step inside the puff");
        last = next;
    }
}
