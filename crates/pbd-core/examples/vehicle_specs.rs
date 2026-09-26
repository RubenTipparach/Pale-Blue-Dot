//! Writes the craft defaults out as RON: `assets/config/vehicles.ron` is this
//! program's output, and `pbd-app`'s config test holds the file to the code.
//!
//! cargo run -p pbd-core --example vehicle_specs > assets/config/vehicles.ron

fn main() {
    let specs = pbd_core::vehicle::spec::VehicleSpecs::default();
    let pretty = ron::ser::PrettyConfig::new()
        .depth_limit(4)
        .struct_names(false)
        .separate_tuple_members(false);
    println!(
        "// The three craft (pbd_core::vehicle::spec): every number each is made of.\n\
         // The code defaults written out by `cargo run -p pbd-core --example\n\
         // vehicle_specs`; a test holds the two together. Metres, kilograms,\n\
         // seconds and radians unless a name says otherwise. Coordinates are the\n\
         // craft's reference frame: +x right, +y up, +z aft (forward is -z).\n\
         // The doc comments in crates/pbd-core/src/vehicle/spec.rs say what each\n\
         // field is."
    );
    println!(
        "{}",
        ron::ser::to_string_pretty(&specs, pretty).expect("the specs serialise")
    );
}
