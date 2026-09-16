//! Parse and semantically validate the standalone WGSL modules, then enforce
//! the shared storage ABI and required entry-point/dispatch contracts.
use naga::{
    Module, ShaderStage, TypeInner,
    valid::{Capabilities, ValidationFlags, Validator},
};
use std::{
    collections::BTreeSet,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

const STANDALONE_SHADERS: [&str; 7] = [
    "atmosphere.wgsl",
    "hex_faces.wgsl",
    "hex_terrain.wgsl",
    "planet_surface.wgsl",
    "planet_visibility.wgsl",
    "voxel_light.wgsl",
    "water.wgsl",
];
// These assets require Bevy's composer, mesh/view imports and substitutions.
// Explicit names keep newly added shaders from silently escaping validation.
const BEVY_COMPOSED_SHADERS: [&str; 1] = ["sky_atmosphere.wgsl"];

fn parse_and_validate(source: &str) -> Result<Module, String> {
    let module = naga::front::wgsl::parse_str(source).map_err(|e| e.emit_to_string(source))?;
    Validator::new(ValidationFlags::all(), Capabilities::empty())
        .validate(&module)
        .map_err(|e| e.emit_to_string(source))?;
    Ok(module)
}

fn check_struct(module: &Module, name: &str, size: u32, offsets: &[u32]) -> Result<(), String> {
    let ty = module
        .types
        .iter()
        .find(|(_, ty)| ty.name.as_deref() == Some(name))
        .map(|(_, ty)| ty)
        .ok_or_else(|| format!("missing ABI struct {name}"))?;
    let TypeInner::Struct { members, span } = &ty.inner else {
        return Err(format!("{name} is not a struct"));
    };
    let actual: Vec<_> = members.iter().map(|m| m.offset).collect();
    if *span != size || actual != offsets {
        return Err(format!(
            "{name}: expected size {size} offsets {offsets:?}; got size {span} offsets {actual:?}"
        ));
    }
    Ok(())
}

fn check_entries(module: &Module, names: &[(&str, ShaderStage, [u32; 3])]) -> Result<(), String> {
    if module.entry_points.len() != names.len() {
        return Err("unexpected number of entry points".into());
    }
    for (name, stage, workgroup) in names {
        let ep = module
            .entry_points
            .iter()
            .find(|e| e.name == *name)
            .ok_or_else(|| format!("missing entry point {name}"))?;
        if ep.stage != *stage || ep.workgroup_size != *workgroup {
            return Err(format!(
                "{name}: unexpected stage/workgroup {:?} {:?}",
                ep.stage, ep.workgroup_size
            ));
        }
    }
    Ok(())
}

fn check_bindings(module: &Module, expected: &[(u32, u32)]) -> Result<(), String> {
    let actual: BTreeSet<_> = module
        .global_variables
        .iter()
        .filter_map(|(_, var)| var.binding.as_ref().map(|b| (b.group, b.binding)))
        .collect();
    let expected: BTreeSet<_> = expected.iter().copied().collect();
    if actual != expected {
        return Err(format!(
            "binding mismatch: expected {expected:?}, actual {actual:?}"
        ));
    }
    Ok(())
}

fn validate(path: &Path) -> Result<(), String> {
    let source = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let module = parse_and_validate(&source)?;
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("invalid filename")?;
    let render_entries = [
        ("vertex", ShaderStage::Vertex, [0; 3]),
        ("fragment", ShaderStage::Fragment, [0; 3]),
    ];
    match filename {
        "hex_faces.wgsl" | "hex_terrain.wgsl" => {
            check_struct(&module, "Cell", 32, &[0, 4, 8, 12, 16, 24])?;
            check_struct(&module, "Column", 112, &[0, 96])?;
            check_struct(&module, "Face", 16, &[0, 4, 8, 12])?;
            if filename == "hex_faces.wgsl" {
                check_struct(&module, "Neighbors", 32, &[0])?;
                check_struct(&module, "Counts", 16, &[0, 4, 8, 12])?;
                check_struct(&module, "DrawIndirect", 16, &[0, 4, 8, 12])?;
                check_struct(&module, "Params", 16, &[0, 4, 8, 12])?;
                check_entries(
                    &module,
                    &[
                        ("reset", ShaderStage::Compute, [1, 1, 1]),
                        ("emit", ShaderStage::Compute, [64, 1, 1]),
                        ("finalize", ShaderStage::Compute, [1, 1, 1]),
                    ],
                )?;
                check_bindings(
                    &module,
                    &[
                        (0, 0),
                        (0, 1),
                        (0, 2),
                        (0, 3),
                        (0, 4),
                        (0, 5),
                        (0, 6),
                        (0, 7),
                    ],
                )?;
            } else {
                check_struct(
                    &module,
                    "TerrainView",
                    208,
                    &[0, 64, 80, 96, 112, 128, 144, 160, 176, 192],
                )?;
                check_entries(&module, &render_entries)?;
                check_bindings(&module, &[(0, 0), (0, 1), (0, 2), (0, 3), (1, 0), (1, 1)])?;
            }
        }
        "voxel_light.wgsl" => {
            check_struct(&module, "LightCell", 16, &[0, 4, 8, 12])?;
            check_struct(&module, "Neighbors", 32, &[0])?;
            check_struct(&module, "Params", 16, &[0, 4, 8, 12])?;
            check_entries(
                &module,
                &[
                    ("initialize", ShaderStage::Compute, [64, 1, 1]),
                    ("relax", ShaderStage::Compute, [64, 1, 1]),
                ],
            )?;
            check_bindings(&module, &[(0, 0), (0, 1), (0, 2), (0, 3), (0, 4)])?;
        }
        "water.wgsl" => {
            check_struct(
                &module,
                "WaterView",
                352,
                &[
                    0, 64, 128, 144, 160, 176, 192, 208, 224, 240, 256, 272, 288, 304, 320, 336,
                ],
            )?;
            check_entries(&module, &render_entries)?;
            check_bindings(&module, &[(0, 0), (1, 0), (1, 1), (1, 2)])?;
        }
        "atmosphere.wgsl" => {
            check_struct(
                &module,
                "AtmosphereView",
                176,
                &[0, 16, 32, 48, 64, 80, 96, 112, 128, 144, 160],
            )?;
            check_entries(&module, &render_entries)?;
            check_bindings(&module, &[(0, 0)])?;
        }
        "planet_surface.wgsl" | "planet_visibility.wgsl" => {
            check_struct(&module, "Cell", 128, &[0, 16, 112])?;
            check_struct(&module, "Params", 112, &[0, 64, 80, 96])?;
            check_bindings(&module, &[(0, 0), (0, 1), (0, 2), (0, 3)])?;
            if filename == "planet_surface.wgsl" {
                check_entries(&module, &render_entries)?;
            } else {
                check_struct(&module, "DrawArgs", 16, &[0, 4, 8, 12])?;
                check_entries(
                    &module,
                    &[
                        ("clear_indirect", ShaderStage::Compute, [1, 1, 1]),
                        ("compact_visible", ShaderStage::Compute, [128, 1, 1]),
                    ],
                )?;
            }
        }
        _ => return Err(format!("no interface contract registered for {filename}")),
    }
    println!("PASS {filename}: WGSL semantics, bindings, entry points, storage/uniform layout");
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let directory = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/shaders"));
    let mut paths: Vec<_> = fs::read_dir(&directory)?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<Result<_, _>>()?;
    paths.retain(|path| path.extension().is_some_and(|ext| ext == "wgsl"));
    paths.sort();
    let mut failures = Vec::new();
    let mut validated = BTreeSet::new();
    for path in paths {
        let Some(filename) = path.file_name().and_then(|name| name.to_str()) else {
            failures.push(format!("invalid shader filename: {}", path.display()));
            continue;
        };
        if BEVY_COMPOSED_SHADERS.contains(&filename) {
            println!("DEFER {filename}: requires Bevy composer and runtime pipeline validation");
            continue;
        }
        if let Err(error) = validate(&path) {
            failures.push(format!("{}:\n{error}", path.display()));
        } else {
            validated.insert(filename.to_owned());
        }
    }
    for required in STANDALONE_SHADERS {
        if !validated.contains(required) {
            failures.push(format!(
                "required standalone shader not validated: {required}"
            ));
        }
    }
    if !failures.is_empty() {
        return Err(failures.join("\n").into());
    }
    println!(
        "All seven standalone shaders validated with Naga 27.0.3; Bevy materials require separate runtime validation."
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_type_error() {
        assert!(
            parse_and_validate(
                "@compute @workgroup_size(1) fn main() { let x: u32 = vec2<f32>(1.0); }"
            )
            .is_err()
        );
    }
    #[test]
    fn rejects_invalid_uniform_layout() {
        let source = "struct P { items: array<u32, 4> } @group(0) @binding(0) var<uniform> p: P; @compute @workgroup_size(1) fn main() { let value = p.items[0]; }";
        assert!(parse_and_validate(source).is_err());
    }
    #[test]
    fn catches_vector_padding_regression() {
        let source = "struct Counter { value: u32, pad: vec3<u32> }";
        let module = parse_and_validate(source).unwrap();
        assert!(check_struct(&module, "Counter", 16, &[0, 4]).is_err());
        check_struct(&module, "Counter", 32, &[0, 16]).unwrap();
    }
}
