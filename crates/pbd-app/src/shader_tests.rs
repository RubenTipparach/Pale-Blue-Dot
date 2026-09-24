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
    surface.replacen(import, &clouds, 1)
}

#[test]
fn the_planet_surface_shader_compiles_with_the_entry_points_its_pipeline_names() {
    let entries = validated_entry_points("planet_surface.wgsl", &planet_surface_source());
    for entry in ["vertex", "fragment"] {
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
