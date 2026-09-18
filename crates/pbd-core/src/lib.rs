//! Engine-independent contracts. Distances are metres, times seconds, angles radians.
//! Planet positions and frame transforms stay f64; local Avian bodies use f32.

pub mod flight;
pub mod frame;
pub mod gravity;
pub mod hex;
pub mod orbit;
pub mod planet_gen;
pub mod terrain;
pub mod weather;

pub use glam::{DQuat, DVec3};
