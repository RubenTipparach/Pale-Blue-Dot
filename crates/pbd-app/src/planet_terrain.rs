//! Stable generation inputs shared by the flight clearance policy and GPU
//! upload. The GPU shades derived geometry; it never decides terrain heights.

use bevy::prelude::*;

/// Sea-level radius in metres. This preview planet is eight kilometres wide.
pub const PLANET_RADIUS: f32 = 4_000.0;

/// Vertical quantum of the surface, in metres: one column cap sits this far
/// above the next. It is the world's height resolution, so it belongs beside
/// the radius rather than inline in the generator. The target engine layers
/// near-player terrain at 1 m (see the voxel-engine-foundation change); this
/// preview
/// steps six times coarser because one height per 19 m column cannot carry
/// metre-scale relief anyway. Read with `PLANET_RADIUS` when judging scale:
/// together they are why a 1.6 m walker reads as small here.
pub const ELEVATION_STEP: f32 = 6.0;

fn hash(x: i32, y: i32, z: i32) -> f32 {
    let mut n = (x as u32).wrapping_mul(0x8da6b343)
        ^ (y as u32).wrapping_mul(0xd8163841)
        ^ (z as u32).wrapping_mul(0xcb1ab31f)
        ^ 0x5eed2026;
    n ^= n >> 13;
    n = n.wrapping_mul(0x85ebca6b);
    n ^= n >> 16;
    (n & 0x00ffffff) as f32 / 8_388_607.5 - 1.0
}

pub(super) fn noise(p: Vec3) -> f32 {
    let i = p.floor().as_ivec3();
    let f = p - p.floor();
    let u = f * f * (Vec3::splat(3.) - 2. * f);
    let mix = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let mut layers = [0.; 2];
    for (z, layer) in layers.iter_mut().enumerate() {
        let z = i.z + z as i32;
        let a = mix(hash(i.x, i.y, z), hash(i.x + 1, i.y, z), u.x);
        let b = mix(hash(i.x, i.y + 1, z), hash(i.x + 1, i.y + 1, z), u.x);
        *layer = mix(a, b, u.y);
    }
    mix(layers[0], layers[1], u.z)
}

/// Quantized terrain elevation above sea level, in metres, on a unit ray.
/// Normalizing here also makes the collision query safe for arbitrary poses.
pub fn surface_height(direction: Vec3) -> f32 {
    let d = direction.normalize_or(Vec3::Y);
    let warp = Vec3::new(
        noise(d * 3.1 + Vec3::X * 17.),
        noise(d * 3.1 + Vec3::Y * 23.),
        noise(d * 3.1 + Vec3::Z * 9.),
    );
    let p = d * 2.35 + warp * 0.32 + Vec3::new(5.2, 1.7, 8.4);
    let continent = noise(p) * 0.70 + noise(p * 2.07) * 0.24 + noise(p * 4.31) * 0.09;
    let ridge = (1. - noise(d * 12.7 + Vec3::splat(33.)).abs()).powi(4);
    let land = ((continent + 0.02) * 6.).clamp(0., 1.);
    let mountains = ridge * (noise(d * 5.4 + Vec3::splat(71.)) * 1.3 + 0.20).max(0.) * 430. * land;
    let detail = noise(d * 62.3) * 7. + noise(d * 124.9) * 3.;
    let generated_height = continent * 630. - 18. + mountains + detail;
    // Keep the seeded coastline and ocean depths while fitting terrestrial
    // relief to this eight-kilometre world. The former ~850m summits overwhelmed
    // its silhouette; the same ranges now peak around425m with broad lowlands.
    let height = generated_height.min(0.) + generated_height.max(0.) * 0.5;
    (height / ELEVATION_STEP).floor() * ELEVATION_STEP
}

/// Solid terrain or water surface radius for assisted-flight clearance.
pub fn terrain_radius(direction: Vec3) -> f32 {
    PLANET_RADIUS + surface_height(direction).max(0.)
}

/// Explicit material roster for this temperate preview world.
pub(super) fn biome(direction: Vec3, height: f32) -> u32 {
    if height < 0. {
        return 0;
    }
    if height < 12. {
        return 1;
    }
    let temperature = 1. - direction.y.abs() - height * 0.00045;
    let moisture = noise(direction * 7.3 + Vec3::splat(91.));
    if temperature < 0.06 || height > 300. {
        6
    } else if height > 155. {
        5
    } else if temperature < 0.22 {
        7
    } else if moisture < -0.15 && temperature > 0.45 {
        4
    } else if moisture > 0.0 && temperature > 0.45 {
        3
    } else {
        2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clearance_is_finite_quantized_and_covers_land_and_ocean() {
        let mut land = 0;
        let mut ocean = 0;
        for cell in super::super::topology::dual_sphere(3) {
            let h = surface_height(cell.direction);
            assert!(h.is_finite());
            assert_eq!(h % ELEVATION_STEP, 0.);
            assert_eq!(terrain_radius(cell.direction), PLANET_RADIUS + h.max(0.));
            assert_eq!(h, surface_height(cell.direction));
            if h < 0. {
                ocean += 1;
            } else {
                land += 1;
            }
        }
        assert!(
            land > 100 && ocean > 100,
            "a world needs substantial oceans and continents"
        );
        assert!(surface_height(Vec3::ZERO).is_finite());
    }
}
