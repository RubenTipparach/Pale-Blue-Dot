//! Engine-independent contracts. Distances are metres, times seconds, angles radians.
//! Planet positions and frame transforms stay f64; local Avian bodies use f32.

pub mod aim;
pub mod atmosphere;
pub mod column;
pub mod daylight;
pub mod dig;
pub mod edits;
pub mod fauna;
pub mod fishing;
pub mod flight;
pub mod frame;
pub mod gravity;
pub mod hex;
pub mod inventory;
pub mod light;
pub mod orbit;
pub mod overlay;
pub mod planet_gen;
pub mod sea;
pub mod terrain;
pub mod topology;
pub mod vehicle;
pub mod weather;
pub mod wind;
pub mod worms;

pub use glam::{DQuat, DVec3};
