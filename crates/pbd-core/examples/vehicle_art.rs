//! Export the authoritative authoring inputs. No renderer or duplicated hull math.
use pbd_core::vehicle::{hull::Hull, spec::VehicleSpecs};
use serde_json::json;
use std::{error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let config = args.first().map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/config/vehicles.ron")
    });
    let output = args.get(1).map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/models/vehicles/inputs.json")
    });
    let specs: VehicleSpecs = ron::from_str(&std::fs::read_to_string(config)?)?;
    specs.validate().map_err(std::io::Error::other)?;
    let shape = |spec| {
        let hull = Hull::new(spec);
        let (points, indices) = hull.loft(20, 8);
        let (deck, deck_indices) = hull.deck(20);
        json!({
            "positions": points.iter().map(|p| p.to_array()).collect::<Vec<_>>(),
            "indices": indices,
            "deck": deck.iter().map(|p| p.to_array()).collect::<Vec<_>>(),
            "deck_indices": deck_indices,
            "stations": 20, "across": 8,
        })
    };
    let data = json!({
        "schema": 1,
        "axes": "+X right, +Y up, +Z aft; metres",
        "specs": specs,
        "wing_panels": specs.kestrel.wing.panels(),
        "hulls": {"tern": shape(specs.tern.hull), "loon": shape(specs.loon.hull)},
    });
    std::fs::create_dir_all(output.parent().ok_or("output needs a parent directory")?)?;
    std::fs::write(&output, serde_json::to_string_pretty(&data)? + "\n")?;
    println!("Wrote {}", output.display());
    Ok(())
}
