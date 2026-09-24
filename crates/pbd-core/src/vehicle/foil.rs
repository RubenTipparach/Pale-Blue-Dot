//! One foil model for every surface that makes lift: wing panels, tail and
//! fin, the sail, the keel and rudder, and a canoe's own lateral plane.
//!
//! The flow is taken at the foil's own point, relative to the fluid moving
//! there, so roll damping, weathercocking and a gust arriving on one wing
//! first all come out without being written down.

use super::body::RigidBody;
use glam::DVec3;
use serde::{Deserialize, Serialize};

/// A foil, in the craft's reference frame (metres; +x right, +y up, +z aft).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct FoilSpec {
    /// Where its force acts.
    pub at: [f32; 3],
    /// Toward its leading edge, unit.
    pub chord: [f32; 3],
    /// The side positive lift pushes toward, unit and square to the chord.
    pub normal: [f32; 3],
    pub area_m2: f32,
    pub aspect: f32,
    pub cl_max: f32,
    pub cd0: f32,
}

impl FoilSpec {
    pub fn validate(&self, name: &str) -> Result<(), String> {
        let c = glam::Vec3::from(self.chord);
        let n = glam::Vec3::from(self.normal);
        let ok = self.area_m2 > 0.0
            && self.aspect > 0.0
            && self.cl_max > 0.0
            && self.cd0 >= 0.0
            && (c.length() - 1.0).abs() < 0.01
            && (n.length() - 1.0).abs() < 0.01
            && c.dot(n).abs() < 0.01;
        ok.then_some(()).ok_or_else(|| {
            format!("{name}: a foil needs positive sizes and a unit chord square to a unit normal")
        })
    }
}

/// What a foil did this substep.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FoilForce {
    pub force: DVec3,
    /// Angle of attack, rad, deflection included.
    pub alpha: f64,
    /// 0 attached, 1 fully stalled.
    pub stall: f64,
    /// Where it acted, planet frame.
    pub at: DVec3,
    /// Dynamic pressure, Pa.
    pub pressure: f64,
}

/// Lift and drag coefficients at angle of attack `alpha`: the thin-aerofoil
/// slope reduced for aspect ratio, a stall that blends over 0.18 rad into
/// flat-plate flow, and induced drag.
pub fn coefficients(alpha: f64, aspect: f64, cl_max: f64, cd0: f64) -> (f64, f64, f64) {
    let slope = std::f64::consts::TAU * aspect / (aspect + 2.0);
    let stall_at = cl_max / slope;
    let t = smoothstep(stall_at, stall_at + 0.18, alpha.abs());
    let attached = (slope * alpha).clamp(-cl_max, cl_max);
    let cl = attached * (1.0 - t) + 1.05 * (2.0 * alpha).sin() * t;
    let cd = cd0
        + (1.0 - t) * attached * attached / (std::f64::consts::PI * 0.8 * aspect)
        + 1.25 * alpha.sin().powi(2) * t;
    (cl, cd, t)
}

/// Apply a foil to a body. `com` is the centre of mass in the reference frame;
/// `fluid` the fluid's velocity at the foil; `scale` multiplies CLmax and
/// `drag` multiplies cd0 (a wet wing); `deflection` shifts the angle of
/// attack (a control surface).
#[allow(clippy::too_many_arguments)]
pub fn apply(
    body: &mut RigidBody,
    spec: &FoilSpec,
    com: DVec3,
    fluid: DVec3,
    density: f64,
    deflection: f64,
    lift_scale: f64,
    drag_scale: f64,
) -> FoilForce {
    apply_oriented(
        body,
        DVec3::from(spec.at.map(f64::from)) - com,
        DVec3::from(spec.chord.map(f64::from)),
        DVec3::from(spec.normal.map(f64::from)),
        spec,
        fluid,
        density,
        deflection,
        lift_scale,
        drag_scale,
    )
}

/// `apply` with the position, chord and normal given in body axes relative
/// to the centre of mass: a sail on a boom that swings.
#[allow(clippy::too_many_arguments)]
pub fn apply_oriented(
    body: &mut RigidBody,
    at: DVec3,
    chord: DVec3,
    normal: DVec3,
    spec: &FoilSpec,
    fluid: DVec3,
    density: f64,
    deflection: f64,
    lift_scale: f64,
    drag_scale: f64,
) -> FoilForce {
    let at = body.point(at);
    let c = body.axis(chord);
    let n = body.axis(normal);
    let span = c.cross(n);
    let mut rel = fluid - body.velocity_at(at);
    rel -= span * rel.dot(span);
    let v2 = rel.length_squared();
    if v2 < 1e-6 {
        return FoilForce {
            at,
            ..Default::default()
        };
    }
    let d = rel / v2.sqrt();
    let alpha = d.dot(n).atan2(-d.dot(c)) + deflection;
    let (cl, cd, stall) = coefficients(
        alpha,
        spec.aspect as f64,
        spec.cl_max as f64 * lift_scale,
        spec.cd0 as f64 * drag_scale,
    );
    let lift_dir = (n - d * d.dot(n)).normalize_or_zero();
    let pressure = 0.5 * density * v2;
    let q = pressure * spec.area_m2 as f64;
    let force = lift_dir * (cl * q) + d * (cd * q);
    body.push(force, at);
    FoilForce {
        force,
        alpha,
        stall,
        at,
        pressure,
    }
}

pub(crate) fn smoothstep(lo: f64, hi: f64, x: f64) -> f64 {
    let t = ((x - lo) / (hi - lo)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_slope_is_thin_aerofoil_theory_for_the_aspect_ratio() {
        let (cl, _, stall) = coefficients(0.05, 6.0, 1.4, 0.01);
        let want = std::f64::consts::TAU * 6.0 / 8.0 * 0.05;
        assert!((cl - want).abs() < 1e-9);
        assert_eq!(stall, 0.0);
    }

    #[test]
    fn past_the_stall_lift_falls_and_drag_rises() {
        let (cl_peak, cd_peak, _) = coefficients(0.3, 6.0, 1.4, 0.01);
        let (cl_deep, cd_deep, stall) = coefficients(0.7, 6.0, 1.4, 0.01);
        assert!(stall > 0.99);
        assert!(cl_deep < cl_peak, "{cl_deep} vs {cl_peak}");
        assert!(cd_deep > cd_peak * 3.0);
        // And the flat plate: no lift face-on, most drag.
        let (cl_90, cd_90, _) = coefficients(std::f64::consts::FRAC_PI_2, 6.0, 1.4, 0.01);
        assert!(cl_90.abs() < 1e-9 && cd_90 > 1.2);
    }

    #[test]
    fn a_wing_meeting_air_from_below_lifts_up() {
        let spec = FoilSpec {
            at: [0.0; 3],
            chord: [0.0, 0.0, -1.0],
            normal: [0.0, 1.0, 0.0],
            area_m2: 10.0,
            aspect: 6.0,
            cl_max: 1.4,
            cd0: 0.01,
        };
        let mut body = RigidBody::new(100.0, DVec3::ONE);
        // Flying forward and slightly down through still air: the air comes
        // from ahead and below.
        body.velocity = DVec3::new(0.0, -2.0, -40.0);
        let f = apply(
            &mut body,
            &spec,
            DVec3::ZERO,
            DVec3::ZERO,
            1.225,
            0.0,
            1.0,
            1.0,
        );
        assert!(f.alpha > 0.0);
        assert!(f.force.y > 0.0, "lift {:?}", f.force);
        assert!(f.force.z > 0.0, "drag holds it back");
    }
}
