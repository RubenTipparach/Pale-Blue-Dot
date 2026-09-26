//! A hull is cells. One shape function says what is inside the envelope; the
//! same function makes the cells whose displaced water is the buoyancy and
//! the loft that is drawn, so what floats is what is seen.
//!
//! The envelope is the hull's outer shape, open or decked: what a boat
//! displaces is the envelope, and the water it has shipped is a separate load
//! (`Bilge`), which is how a canoe can swamp and still float awash.

use super::body::RigidBody;
use crate::sea::LocalSea;
use glam::{DVec3, Vec3};
use serde::{Deserialize, Serialize};

/// A hull's shape in the craft's reference frame: +z aft, waterline near y 0.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HullSpec {
    pub length_m: f32,
    pub beam_m: f32,
    /// From the sheer down to the keel line amidships, m.
    pub depth_m: f32,
    /// Height of the sheer (the top edge), m.
    pub sheer_m: f32,
    /// Side of one buoyancy cell, m.
    pub cell_m: f32,
    /// How full the ends are: the half-beam falls as 1 - |z|^power, forward
    /// and aft. Higher is fuller.
    pub bow_power: f32,
    pub stern_power: f32,
    /// How the section rises from keel to sheer: 1 a V, 2 round-bilged.
    pub section_power: f32,
    /// Whether the hull is decked over (drawn with a deck).
    pub deck: bool,
}

impl HullSpec {
    pub fn validate(&self, name: &str) -> Result<(), String> {
        let v = [
            self.length_m,
            self.beam_m,
            self.depth_m,
            self.cell_m,
            self.bow_power,
            self.stern_power,
            self.section_power,
        ];
        if v.iter().any(|x| !(x.is_finite() && *x > 0.0)) || !self.sheer_m.is_finite() {
            return Err(format!("{name}.hull: sizes must be positive"));
        }
        if self.cell_m > self.beam_m / 3.0 {
            return Err(format!(
                "{name}.hull: cell_m must be at most a third of the beam"
            ));
        }
        Ok(())
    }

    /// Half the beam at `zn` (-1 bow, +1 stern), m.
    pub fn half_beam(&self, zn: f32) -> f32 {
        let power = if zn < 0.0 {
            self.bow_power
        } else {
            self.stern_power
        };
        self.beam_m / 2.0 * (1.0 - zn.abs().powf(power)).max(0.0)
    }

    /// The keel line's height at `zn`: it rises toward the ends.
    pub fn keel(&self, zn: f32) -> f32 {
        self.sheer_m - self.depth_m * (1.0 - 0.55 * zn.abs().powi(3))
    }

    /// The bottom of the section at `zn`, `x` off the centreline.
    pub fn bottom(&self, zn: f32, x: f32) -> f32 {
        let half = self.half_beam(zn).max(1e-4);
        let keel = self.keel(zn);
        keel + (self.sheer_m - keel) * (x.abs() / half).min(1.0).powf(self.section_power)
    }

    pub fn contains(&self, p: Vec3) -> bool {
        let zn = p.z / (self.length_m / 2.0);
        if zn.abs() > 1.0 {
            return false;
        }
        let half = self.half_beam(zn);
        if half < 0.02 || p.x.abs() > half {
            return false;
        }
        p.y >= self.bottom(zn, p.x) && p.y <= self.sheer_m
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Hull {
    pub spec: HullSpec,
    /// Cell centres, reference frame.
    pub cells: Vec<Vec3>,
    /// Points along the sheer, both sides: where water comes aboard.
    pub rim: Vec<Vec3>,
    /// The envelope's volume, m^3.
    pub volume: f64,
}

impl Hull {
    pub fn new(spec: HullSpec) -> Self {
        let s = spec.cell_m;
        let mut cells = Vec::new();
        let steps = |extent: f32| (extent / s).floor() as i32;
        for iz in -steps(spec.length_m / 2.0)..=steps(spec.length_m / 2.0) {
            for ix in -steps(spec.beam_m / 2.0)..=steps(spec.beam_m / 2.0) {
                let mut y = spec.sheer_m - spec.depth_m + s / 2.0;
                while y < spec.sheer_m {
                    let p = Vec3::new(ix as f32 * s, y, iz as f32 * s);
                    if spec.contains(p) {
                        cells.push(p);
                    }
                    y += s;
                }
            }
        }
        let mut rim = Vec::new();
        for i in 0..11 {
            let zn = -0.85 + i as f32 * 0.17;
            for side in [-1.0, 1.0] {
                rim.push(Vec3::new(
                    side * spec.half_beam(zn) * 0.97,
                    spec.sheer_m,
                    zn * spec.length_m / 2.0,
                ));
            }
        }
        let volume = cells.len() as f64 * (s as f64).powi(3);
        Self {
            spec,
            cells,
            rim,
            volume,
        }
    }

    /// The hull's surface for drawing: `stations` sections along it, `across`
    /// points round each. Positions in the reference frame; triangles wound
    /// outward.
    pub fn loft(&self, stations: usize, across: usize) -> (Vec<Vec3>, Vec<u32>) {
        let s = &self.spec;
        let mut positions = Vec::new();
        for i in 0..=stations {
            let zn = -1.0 + 2.0 * i as f32 / stations as f32;
            let half = s.half_beam(zn).max(0.001);
            for j in 0..=across {
                let x = -half + 2.0 * half * j as f32 / across as f32;
                positions.push(Vec3::new(x, s.bottom(zn, x), zn * s.length_m / 2.0));
            }
        }
        let mut indices = Vec::new();
        let row = (across + 1) as u32;
        for i in 0..stations as u32 {
            for j in 0..across as u32 {
                let a = i * row + j;
                let c = a + row;
                indices.extend_from_slice(&[a, a + 1, c, a + 1, c + 1, c]);
            }
        }
        (positions, indices)
    }

    /// The deck, for a decked hull: a strip across the sheer at every station.
    pub fn deck(&self, stations: usize) -> (Vec<Vec3>, Vec<u32>) {
        let s = &self.spec;
        let mut positions = Vec::new();
        for i in 0..=stations {
            let zn = -1.0 + 2.0 * i as f32 / stations as f32;
            let half = s.half_beam(zn);
            let z = zn * s.length_m / 2.0;
            positions.push(Vec3::new(-half, s.sheer_m, z));
            positions.push(Vec3::new(half, s.sheer_m, z));
        }
        let mut indices = Vec::new();
        for i in 0..stations as u32 {
            let a = i * 2;
            indices.extend_from_slice(&[a, a + 2, a + 1, a + 1, a + 2, a + 3]);
        }
        (positions, indices)
    }
}

/// Where the sea is, for a hull: the local sea and the sea radius.
pub struct Water<'a> {
    pub sea: &'a LocalSea,
    pub radius: f64,
    pub density: f64,
    pub gravity: f64,
}

/// Buoyancy and heave damping over every cell; returns the displaced volume,
/// m^3. `damping` is N s / m per m^3 submerged, against each cell's velocity
/// along the local up relative to the water's.
pub fn float(body: &mut RigidBody, hull: &Hull, com: DVec3, water: &Water, damping: f64) -> f64 {
    let s = hull.spec.cell_m as f64;
    let cell_volume = s * s * s;
    let reach = water.sea.reach() as f64 + s;
    let mut displaced = 0.0;
    for cell in &hull.cells {
        let world = body.point(cell.as_dvec3() - com);
        let r = world.length();
        let up = world / r;
        let height = r - water.radius;
        // A cell above the highest crest is dry whatever the sea does.
        if height - s / 2.0 > reach {
            continue;
        }
        let point = water.sea.at(world.as_vec3(), water.radius as f32, 0.0);
        let depth = point.height as f64 - height + s / 2.0;
        if depth <= 0.0 {
            continue;
        }
        let submerged = cell_volume * (depth / s).min(1.0);
        displaced += submerged;
        let relative = (body.velocity_at(world) - point.velocity.as_dvec3()).dot(up);
        let lift = water.density * water.gravity * submerged - damping * submerged * relative;
        body.push(up * lift, world);
    }
    displaced
}

/// Where the hull's resistance acts and how big the hull is to the water.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResistanceSpec {
    pub at: [f32; 3],
    /// Wetted area, m^2.
    pub wetted_m2: f32,
    /// Waterline length, m: sets hull speed.
    pub waterline_m: f32,
    /// Side-on area to the water, m^2, drag coefficient one.
    pub lateral_m2: f32,
    /// Skin-friction coefficient.
    pub friction: f32,
}

/// Skin friction and wave-making along the keel line, which climbs a wall
/// near hull speed (Froude number about 0.4), and drag side-on. Scaled by how
/// much of the hull is in the water. Returns the Froude number.
pub fn resist(
    body: &mut RigidBody,
    spec: &ResistanceSpec,
    com: DVec3,
    water_velocity: DVec3,
    gravity: f64,
    density: f64,
    immersed: f64,
) -> f64 {
    if immersed <= 0.0 {
        return 0.0;
    }
    let at = body.point(DVec3::from(spec.at.map(f64::from)) - com);
    let up = at.normalize();
    let mut rel = body.velocity_at(at) - water_velocity;
    rel -= up * rel.dot(up);
    let forward = body.axis(DVec3::NEG_Z);
    let forward = (forward - up * forward.dot(up)).normalize_or_zero();
    let u = rel.dot(forward);
    let side = rel - forward * u;
    let froude = u.abs() / (gravity * spec.waterline_m as f64).sqrt();
    let wave = 0.0015
        + 0.05 * super::foil::smoothstep(0.28, 0.46, froude)
        + 0.12 * (froude - 0.46).max(0.0);
    let along =
        0.5 * density * spec.wetted_m2 as f64 * (spec.friction as f64 + wave) * u * u * immersed;
    let force = forward * (-u.signum() * along)
        - side * (0.5 * density * spec.lateral_m2 as f64 * side.length() * immersed);
    body.push(force, at);
    froude
}

/// The water a boat has shipped.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BilgeSpec {
    /// Area open to the rain, m^2.
    pub open_m2: f32,
    /// Share of the rim's weir that lets water in: a decked boat's coaming
    /// is less of its sheer than an open canoe's whole gunwale.
    pub rim_share: f32,
    /// Only rim points aft of this z admit water (a cockpit), m.
    pub rim_from_z: f32,
    /// What drains away on its own, kg/s (scuppers).
    pub drain_kgps: f32,
    /// What bailing removes, kg/s.
    pub bail_kgps: f32,
    /// The most it can hold, kg.
    pub capacity_kg: f32,
    /// Where it lies, reference frame.
    pub at: [f32; 3],
    /// How far it runs to the low side per unit of heel sine, and at most, m.
    pub slosh: f32,
    pub slosh_max_m: f32,
}

/// Water in and out over one substep, kg of change. `over` gives, for each
/// rim point, how far the sea stands over it (negative: under it).
pub fn bilge_flow(
    spec: &BilgeSpec,
    hull: &Hull,
    rain_mmh: f64,
    over: impl Fn(Vec3) -> f64,
    bailing: bool,
) -> f64 {
    let mut inflow = rain_mmh / 3600.0 * spec.open_m2 as f64;
    let spacing = hull.spec.length_m as f64 * 0.17 / 2.0;
    for point in &hull.rim {
        if point.z < spec.rim_from_z {
            continue;
        }
        let head = over(*point);
        if head > 0.0 {
            // The weir equation, Q = 1.7 b H^1.5, in kg/s of sea water.
            inflow += 1025.0 * 1.7 * spacing * head.powf(1.5) * spec.rim_share as f64;
        }
    }
    inflow - spec.drain_kgps as f64 - if bailing { spec.bail_kgps as f64 } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canoe() -> HullSpec {
        HullSpec {
            length_m: 5.0,
            beam_m: 0.92,
            depth_m: 0.36,
            sheer_m: 0.30,
            cell_m: 0.14,
            bow_power: 1.8,
            stern_power: 1.8,
            section_power: 1.6,
            deck: false,
        }
    }

    #[test]
    fn the_cells_fill_the_envelope_and_stay_inside_it() {
        let hull = Hull::new(canoe());
        assert!(hull.cells.len() > 150, "{} cells", hull.cells.len());
        assert!(hull.cells.iter().all(|c| canoe().contains(*c)));
        // The envelope is a fair share of its bounding box, not a sliver.
        let bounding = 5.0 * 0.92 * 0.36;
        let share = hull.volume / bounding;
        assert!((0.25..0.7).contains(&share), "{share}");
    }

    #[test]
    fn the_loft_closes_at_the_ends_and_wraps_the_cells() {
        let hull = Hull::new(canoe());
        let (positions, indices) = hull.loft(24, 10);
        assert_eq!(indices.len(), 24 * 10 * 6);
        let bow = positions[0];
        assert!(bow.x.abs() < 0.01, "a canoe's bow is a point");
        let widest = positions.iter().map(|p| p.x.abs()).fold(0.0f32, f32::max);
        assert!((widest - 0.46).abs() < 0.02);
    }

    #[test]
    fn rain_fills_an_open_hull_by_its_area() {
        let spec = BilgeSpec {
            open_m2: 3.6,
            rim_share: 1.0,
            rim_from_z: -99.0,
            drain_kgps: 0.0,
            bail_kgps: 6.0,
            capacity_kg: 700.0,
            at: [0.0; 3],
            slosh: 1.2,
            slosh_max_m: 0.3,
        };
        let hull = Hull::new(canoe());
        let per_s = bilge_flow(&spec, &hull, 60.0, |_| -1.0, false);
        // 60 mm/h over 3.6 m^2 is 3.6 kg a minute.
        assert!((per_s * 60.0 - 3.6).abs() < 1e-5);
        assert!(
            bilge_flow(&spec, &hull, 0.0, |_| 0.05, false) > 10.0,
            "a gunwale under floods"
        );
        assert!(bilge_flow(&spec, &hull, 0.0, |_| -1.0, true) < 0.0);
    }
}
