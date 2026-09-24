//! The planet's cells. The generator lives in the core (`pbd_core::topology`)
//! so the terrain and the atmosphere are built on one implementation; this
//! module keeps the names the renderer has always used.

pub(crate) use pbd_core::topology::{
    DualCell, dual_from_triangles, dual_sphere, icosahedron, midpoint,
};
