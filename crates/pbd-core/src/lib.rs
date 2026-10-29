//! Engine-independent contracts. Distances are metres, times seconds, angles radians.
//! Planet positions and frame transforms stay f64; local Avian bodies use f32.

pub mod aim;
pub mod atmosphere;
pub mod bounce;
pub mod cell_key;
pub mod cloud_puffs;
pub mod column;
pub mod daylight;
pub mod dig;
pub mod drops;
pub mod edits;
pub mod fauna;
pub mod fishing;
pub mod flight;
pub mod flora;
pub mod frame;
pub mod frustum;
pub mod geo;
pub mod gravity;
pub mod hex;
pub mod hexel;
pub mod inventory;
pub mod lamp_bake;
pub mod light;
pub mod map;
pub mod orbit;
pub mod overlay;
pub mod planet_gen;
pub mod records;
pub mod sea;
pub mod settlement;
pub mod shadow;
pub mod sites;
pub mod sky_light;
pub mod terrain;
pub mod topology;
pub mod vehicle;
pub mod weather;
pub mod wind;
pub mod worms;

pub use glam::{DQuat, DVec3};
