//! The shipped shaders parse and validate on the CPU, with naga, exactly as
//! the pipeline cache will have to compile them. Bevy reports a shader that
//! fails there as an error in the log while the app keeps running, drawing
//! nothing from that pipeline: a capture harness gets a picture with the planet
//! missing and no failing check anywhere. This is that check. Only the shaders
//! with no `#import` or `#ifdef` can be validated raw; the sky and water passes
//! go through Bevy's preprocessor and are not here.

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

#[test]
fn the_planet_surface_shader_compiles_with_the_entry_points_its_pipeline_names() {
    let entries = validated_entry_points(
        "planet_surface.wgsl",
        include_str!("../../../assets/shaders/planet_surface.wgsl"),
    );
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
