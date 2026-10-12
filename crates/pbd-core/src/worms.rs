//! Perlin worms: the caves are TUBES, carved by walkers.
//!
//! A worm is a seed, a start and a walk. From its seed it takes a length, a
//! radius and a heading; each step it moves `step_m` along its heading, turns
//! by 3D noise sampled where it is, and its radius wanders on the same noise,
//! so a slow bend widens into a room. What it leaves is a chain of capsules.
//!
//! **A share of worms start at the surface**, a layer under the ground and
//! heading down. The first capsule of such a worm cuts the ground open: that is
//! a cave mouth, and it leads somewhere by construction, because the rest of
//! the worm is behind it.
//!
//! **A column stays a pure function of its direction.** Worms are seeded on a
//! fixed cube-sphere lattice and a cell's worms depend on the cell's index and
//! the world seed alone, so a worm walked from anywhere is the same worm. A
//! region gathers every worm that could reach it - every seed cell within the
//! longest worm of any point in the region - walks them once, and hands the
//! paths to every column it builds. A column's carve is then complete whatever
//! region built it.
//!
//! The sheet carve this replaces was one thresholded ridged field: connected,
//! but every cave a slab, sight lines of two metres, and a slab meeting the
//! ground a line of single-cell holes. The owner called it: tubes, not sheets.

use crate::planet_gen::{self, TerrainConfig};
use glam::Vec3;

/// How the worms are grown. Tunable; the defaults are measured in the tests.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WormField {
    /// Seed lattice cell size, metres of arc. About a base tile.
    pub cell_m: f32,
    /// Mean worms per seed cell.
    pub density: f32,
    /// Shortest and longest worm, metres.
    pub length_m: (f32, f32),
    /// Narrowest and widest tunnel radius, metres.
    pub radius_m: (f32, f32),
    /// Metres per step, and so per capsule.
    pub step_m: f32,
    /// Most a worm turns in one step, radians.
    pub turn: f32,
    /// Steepest pitch off the tangent plane, radians, either way.
    pub pitch_max: f32,
    /// Share of worms that start at the surface, heading down.
    pub surface_share: f32,
    /// How far under the ground a buried worm starts, metres.
    pub start_depth_m: (f32, f32),
    /// Metres across the noise a worm steers on.
    pub steer_scale_m: f32,
    /// Layers of solid a worm keeps above the bedrock floor.
    pub floor_layers: usize,
}

impl WormField {
    pub const DEFAULT: WormField = WormField {
        cell_m: 45.0,
        density: 1.2,
        length_m: (50.0, 160.0),
        radius_m: (1.6, 3.4),
        step_m: 2.0,
        // A bend radius of about seventeen metres at a two-metre step: at 0.30
        // a tunnel doubled back inside thirty metres and the view along it
        // was 32 m, measured; a tunnel should be a place a player can see down.
        turn: 0.12,
        pitch_max: 0.45,
        surface_share: 0.30,
        start_depth_m: (8.0, 50.0),
        steer_scale_m: 30.0,
        floor_layers: 3,
    };

    /// The farthest a worm's carve can lie from its seed cell's centre: the
    /// cell's own half-diagonal, the longest walk and the widest radius. This
    /// is the margin a gather adds to its reach so no worm that could touch a
    /// column is missed.
    pub fn reach_m(&self) -> f32 {
        self.cell_m * 0.71 + self.length_m.1 + self.radius_m.1
    }
}

impl Default for WormField {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// One step of a worm: the tunnel between two centres at a radius.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Capsule {
    /// Body-local metres.
    pub a: Vec3,
    pub b: Vec3,
    pub r: f32,
}

impl Capsule {
    /// Distance from a point to the capsule's axis segment.
    pub fn distance(&self, p: Vec3) -> f32 {
        let ab = self.b - self.a;
        let len2 = ab.length_squared();
        let t = if len2 > 1e-9 {
            ((p - self.a).dot(ab) / len2).clamp(0.0, 1.0)
        } else {
            0.0
        };
        (p - (self.a + ab * t)).length()
    }
}

/// A walked worm.
#[derive(Clone, Debug)]
pub struct Worm {
    pub seed: u64,
    /// Started a layer under the ground, heading down: an opening.
    pub surface_start: bool,
    /// Where it started, as a direction.
    pub start: Vec3,
    pub capsules: Vec<Capsule>,
    /// A bounding cone for the whole worm: `cos` of the angle from `axis`
    /// within which every capsule lies, so a column can reject a worm with
    /// one dot product.
    axis: Vec3,
    cos_reach: f32,
}

/// Every worm gathered for a region.
#[derive(Clone, Debug, Default)]
pub struct Worms {
    pub worms: Vec<Worm>,
}

fn mix(mut v: u64) -> u64 {
    v ^= v >> 30;
    v = v.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    v ^= v >> 27;
    v = v.wrapping_mul(0x94d0_49bb_1331_11eb);
    v ^ (v >> 31)
}

/// A unit float from a seed and a channel.
fn roll(seed: u64, channel: u64) -> f32 {
    (mix(seed ^ channel.wrapping_mul(0x9e37_79b9_7f4a_7c15)) >> 40) as f32 / (1u64 << 24) as f32
}

/// The cube-sphere lattice: six faces of `n` by `n` cells.
fn lattice_n(field: &WormField, terrain: &TerrainConfig) -> u32 {
    ((terrain.radius_m * std::f32::consts::FRAC_PI_2) / field.cell_m.max(1.0)).ceil() as u32
}

/// The centre direction of a lattice cell.
fn cell_direction(n: u32, face: u32, i: u32, j: u32) -> Vec3 {
    let u = (i as f32 + 0.5) / n as f32 * 2.0 - 1.0;
    let v = (j as f32 + 0.5) / n as f32 * 2.0 - 1.0;
    let p = match face {
        0 => Vec3::new(1.0, u, v),
        1 => Vec3::new(-1.0, u, v),
        2 => Vec3::new(u, 1.0, v),
        3 => Vec3::new(u, -1.0, v),
        4 => Vec3::new(u, v, 1.0),
        _ => Vec3::new(u, v, -1.0),
    };
    p.normalize()
}

/// The seed a lattice cell's `k`th worm walks from.
fn worm_seed(terrain: &TerrainConfig, n: u32, face: u32, i: u32, j: u32, k: u32) -> u64 {
    let cell = (face as u64) * (n as u64) * (n as u64) + (j as u64) * (n as u64) + i as u64;
    mix(terrain.seed
        ^ 0x0770_7370_0000_0000
        ^ cell.wrapping_mul(0x2545_f491_4f6c_dd1d)
        ^ (k as u64) << 48)
}

/// Walk one worm. `None` when the cell rolls no worm at this index, or when
/// the worm would start under the sea.
pub fn walk(
    field: &WormField,
    terrain: &TerrainConfig,
    cell: Vec3,
    seed: u64,
    k: u32,
) -> Option<Worm> {
    // Density: the cell rolls `floor(density)` worms and one more with the
    // fractional chance.
    let whole = field.density.floor() as u32;
    if k >= whole && (k > whole || roll(seed, 1) >= field.density - whole as f32) {
        return None;
    }
    // A start jittered inside the cell, on the ground.
    let tangent = cell.any_orthonormal_vector();
    let bitangent = cell.cross(tangent);
    let jitter = field.cell_m * 0.5 / terrain.radius_m;
    let start = (cell
        + (tangent * (roll(seed, 2) - 0.5) + bitangent * (roll(seed, 3) - 0.5)) * 2.0 * jitter)
        .normalize();
    let surface = planet_gen::surface_altitude(terrain, start);
    if surface < terrain.sea_level_m + terrain.beach_band_m {
        return None;
    }
    let surface_start = roll(seed, 4) < field.surface_share;
    let length = field.length_m.0 + (field.length_m.1 - field.length_m.0) * roll(seed, 5);
    let steps = (length / field.step_m.max(0.1)).ceil().max(1.0) as usize;
    let floor_m = crate::column::BASE_M as f32 + field.floor_layers as f32 + 1.0;
    let radius_at = |p: Vec3| {
        let n = planet_gen::gradient_noise(
            seed ^ 0xa,
            p.x / field.steer_scale_m,
            p.y / field.steer_scale_m,
            p.z / field.steer_scale_m,
        );
        field.radius_m.0 + (field.radius_m.1 - field.radius_m.0) * (0.5 + 0.5 * n).clamp(0.0, 1.0)
    };
    let altitude = if surface_start {
        surface - 0.5
    } else {
        surface
            - (field.start_depth_m.0
                + (field.start_depth_m.1 - field.start_depth_m.0) * roll(seed, 6))
    };
    let mut position = start * (terrain.radius_m + altitude.max(floor_m));
    let yaw = roll(seed, 7) * std::f32::consts::TAU;
    let mut forward = tangent * yaw.cos() + bitangent * yaw.sin();
    let mut pitch = if surface_start {
        -field.pitch_max * 0.8
    } else {
        (roll(seed, 8) - 0.5) * field.pitch_max
    };
    let mut capsules = Vec::with_capacity(steps);
    for step in 0..steps {
        let up = position.normalize();
        // Steer: yaw about the local up, pitch against it, both off noise
        // sampled where the worm is, so a worm bends rather than jitters.
        let n1 = planet_gen::gradient_noise(
            seed ^ 0xb,
            position.x / field.steer_scale_m,
            position.y / field.steer_scale_m,
            position.z / field.steer_scale_m,
        );
        let n2 = planet_gen::gradient_noise(
            seed ^ 0xc,
            position.x / field.steer_scale_m + 17.3,
            position.y / field.steer_scale_m,
            position.z / field.steer_scale_m,
        );
        let horizontal = (forward - up * forward.dot(up)).normalize_or(tangent);
        let turned = glam::Quat::from_axis_angle(up, n1 * field.turn) * horizontal;
        pitch = (pitch + n2 * field.turn * 0.5).clamp(-field.pitch_max, field.pitch_max);
        let radius = radius_at(position);
        // Under the ground by the radius and a layer, except while a surface
        // worm is still cutting its mouth; and never into the bedrock floor.
        // BOTH ends of the capsule are held: the far end is where the worm
        // will stand next step, and a capsule recorded with an unclamped far
        // end is a capsule whose last metre can poke through the ground where
        // the surface falls away, or at the end of the worm where no next
        // step ever clamps it.
        let hold = |step: usize, p: Vec3, pitch: &mut f32| -> Vec3 {
            let up = p.normalize();
            let surface_here = planet_gen::surface_altitude(terrain, up);
            let ceiling = if surface_start && step < 3 {
                surface_here + 1.0
            } else {
                surface_here - radius - 1.0
            };
            let floor = floor_m + radius;
            let mut altitude = p.length() - terrain.radius_m;
            if altitude > ceiling {
                altitude = ceiling;
                *pitch = pitch.min(-0.1);
            } else if altitude < floor {
                altitude = floor;
                *pitch = pitch.max(0.1);
            }
            up * (terrain.radius_m + altitude)
        };
        position = hold(step, position, &mut pitch);
        forward = (turned * pitch.cos() + up * pitch.sin()).normalize();
        let next = hold(step + 1, position + forward * field.step_m, &mut pitch);
        capsules.push(Capsule {
            a: position,
            b: next,
            r: radius,
        });
        position = next;
    }
    // The bounding cone.
    let axis = capsules
        .iter()
        .fold(Vec3::ZERO, |acc, c| acc + c.a + c.b)
        .normalize_or(start);
    let cos_reach = capsules
        .iter()
        .map(|c| {
            // The angular half-width a capsule can reach: its ends plus its
            // radius over the body radius.
            let end = c.a.normalize().dot(axis).min(c.b.normalize().dot(axis));
            (end.clamp(-1.0, 1.0).acos() + c.r / terrain.radius_m).min(std::f32::consts::PI)
        })
        .fold(0.0f32, f32::max)
        .cos();
    Some(Worm {
        seed,
        surface_start,
        start,
        capsules,
        axis,
        cos_reach,
    })
}

/// Every worm whose carve could lie within `reach_m` of `direction`: each
/// seed cell within that reach plus the field's own margin, walked.
pub fn gather(field: &WormField, terrain: &TerrainConfig, direction: Vec3, reach_m: f32) -> Worms {
    let direction = direction.normalize_or(Vec3::Y);
    let n = lattice_n(field, terrain);
    let within = ((reach_m + field.reach_m()) / terrain.radius_m).cos();
    let mut worms = Vec::new();
    let per_cell = field.density.ceil() as u32;
    for face in 0..6 {
        for j in 0..n {
            for i in 0..n {
                let cell = cell_direction(n, face, i, j);
                if cell.dot(direction) < within {
                    continue;
                }
                for k in 0..per_cell {
                    let seed = worm_seed(terrain, n, face, i, j, k);
                    if let Some(worm) = walk(field, terrain, cell, seed, k) {
                        worms.push(worm);
                    }
                }
            }
        }
    }
    Worms { worms }
}

impl Worms {
    /// The worms whose bounding cone holds `direction`, at a column's own
    /// angular width.
    fn near(
        &self,
        direction: Vec3,
        margin_m: f32,
        terrain: &TerrainConfig,
    ) -> impl Iterator<Item = &Worm> {
        let slack = (margin_m / terrain.radius_m).cos();
        self.worms.iter().filter(move |w| {
            // cos(a + b) >= cos_reach * slack - ... : loosen by the margin.
            direction.dot(w.axis) >= w.cos_reach * slack - (1.0 - slack)
        })
    }

    /// Whether the point on the column at `altitude` is inside any worm.
    pub fn hollow(&self, terrain: &TerrainConfig, direction: Vec3, altitude: f32) -> bool {
        let p = direction * (terrain.radius_m + altitude);
        self.near(direction, 4.0, terrain)
            .any(|w| w.capsules.iter().any(|c| c.distance(p) < c.r))
    }

    /// Carve a column: every layer whose centre lies inside a worm is air.
    /// `solid` is a callback marking a layer air; layers below `floor_layers`
    /// are never touched.
    pub fn carve(
        &self,
        terrain: &TerrainConfig,
        direction: Vec3,
        floor_layers: usize,
        mut open: impl FnMut(usize),
    ) {
        use crate::column::{BASE_M, LAYERS};
        for worm in self.near(direction, 4.0, terrain) {
            for capsule in &worm.capsules {
                // The layers the capsule can reach on this column.
                let lo = capsule.a.length().min(capsule.b.length()) - capsule.r - terrain.radius_m;
                let hi = capsule.a.length().max(capsule.b.length()) + capsule.r - terrain.radius_m;
                let first = ((lo - BASE_M as f32).floor().max(0.0) as usize).max(floor_layers + 1);
                let last = ((hi - BASE_M as f32).ceil().max(0.0) as usize).min(LAYERS);
                for index in first..last {
                    let p = direction * (terrain.radius_m + BASE_M as f32 + index as f32 + 0.5);
                    if capsule.distance(p) < capsule.r {
                        open(index);
                    }
                }
            }
        }
    }

    /// The nearest surface-starting worm to `from`: where a walker can enter
    /// a cave without digging.
    pub fn nearest_opening(&self, from: Vec3) -> Option<Vec3> {
        self.worms
            .iter()
            .filter(|w| w.surface_start)
            .map(|w| w.start)
            .max_by(|a, b| a.dot(from).total_cmp(&b.dot(from)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::column::{BASE_M, LAYERS};

    const TERRAIN: TerrainConfig = TerrainConfig::TENEBRIS;

    #[test]
    fn a_worm_is_the_same_worm_walked_twice() {
        let field = WormField::DEFAULT;
        let a = gather(
            &field,
            &TERRAIN,
            Vec3::new(0.8776, 0.4794, 0.0).normalize(),
            60.0,
        );
        let b = gather(
            &field,
            &TERRAIN,
            Vec3::new(0.8776, 0.4794, 0.0).normalize(),
            60.0,
        );
        assert!(!a.worms.is_empty(), "the spawn's region grows worms");
        assert_eq!(a.worms.len(), b.worms.len());
        for (x, y) in a.worms.iter().zip(&b.worms) {
            assert_eq!(x.seed, y.seed);
            assert_eq!(x.capsules, y.capsules);
        }
    }

    #[test]
    fn a_buried_worm_stays_under_the_ground_and_over_the_bedrock() {
        let field = WormField::DEFAULT;
        let worms = gather(
            &field,
            &TERRAIN,
            Vec3::new(0.8776, 0.4794, 0.0).normalize(),
            200.0,
        );
        let mut checked = 0;
        for worm in worms.worms.iter().filter(|w| !w.surface_start) {
            for c in &worm.capsules {
                for p in [c.a, c.b] {
                    let up = p.normalize();
                    let altitude = p.length() - TERRAIN.radius_m;
                    let surface = planet_gen::surface_altitude(&TERRAIN, up);
                    assert!(
                        altitude + c.r <= surface + 0.01,
                        "a buried worm at {altitude:.1} m, radius {:.1}, under ground at {surface:.1}",
                        c.r
                    );
                    assert!(altitude - c.r >= BASE_M as f32 + field.floor_layers as f32 + 0.99);
                    checked += 1;
                }
            }
        }
        assert!(checked > 100, "only {checked} capsule ends checked");
    }

    #[test]
    fn a_surface_worm_cuts_the_ground_where_it_starts_and_then_goes_under() {
        let field = WormField::DEFAULT;
        let worms = gather(
            &field,
            &TERRAIN,
            Vec3::new(0.8776, 0.4794, 0.0).normalize(),
            400.0,
        );
        let worm = worms
            .worms
            .iter()
            .find(|w| w.surface_start)
            .expect("some worm starts at the surface");
        let first = worm.capsules[0];
        let surface = planet_gen::surface_altitude(&TERRAIN, worm.start);
        let top = first.a.length() - TERRAIN.radius_m + first.r;
        assert!(top > surface, "the first capsule reaches above the ground");
        // And the column at its start is open at the top.
        let mut air = Vec::new();
        worms.carve(&TERRAIN, worm.start, field.floor_layers, |i| air.push(i));
        let surface_layer = ((surface - BASE_M as f32).floor() as usize).min(LAYERS - 1);
        assert!(
            air.iter().any(|&i| i + 1 >= surface_layer),
            "the start column has air at the ground"
        );
        // Deep in, it is under the ground.
        let last = worm.capsules.last().unwrap();
        let up = last.b.normalize();
        assert!(
            last.b.length() - TERRAIN.radius_m + last.r
                <= planet_gen::surface_altitude(&TERRAIN, up) + 0.01
        );
    }

    #[test]
    fn a_column_a_worm_passes_through_is_hollow_there() {
        let field = WormField::DEFAULT;
        let worms = gather(
            &field,
            &TERRAIN,
            Vec3::new(-0.5, 0.6, 0.62).normalize(),
            200.0,
        );
        let worm = worms
            .worms
            .iter()
            .find(|w| !w.surface_start)
            .expect("a buried worm");
        let mid = worm.capsules[worm.capsules.len() / 2];
        let direction = mid.a.normalize();
        let altitude = mid.a.length() - TERRAIN.radius_m;
        assert!(worms.hollow(&TERRAIN, direction, altitude));
        let mut air = Vec::new();
        worms.carve(&TERRAIN, direction, field.floor_layers, |i| air.push(i));
        let layer = (altitude - BASE_M as f32).floor() as usize;
        assert!(
            air.contains(&layer),
            "layer {layer} at the worm's own centre is air"
        );
    }
}
