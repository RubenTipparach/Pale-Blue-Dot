//! The Loon: a canoe, paddled. A stroke is a blade dragged through the water:
//! its velocity is the canoe's point velocity plus the stroke's own, taken
//! against the moving water, so a stroke on one side turns the canoe and a
//! canoe cannot outrun its own blade. Rain and waves over the gunwale fill it,
//! and the water sloshes to the low side, which is what swamps a canoe.

use super::foil::FoilForce;
use super::hull::{bilge_flow, float, resist};
use super::{
    Context, Craft, CraftState, FORWARD, Input, RIGHT, SEA_DENSITY, Telemetry, contact, v3, windage,
};
use glam::DVec3;

/// A stroke in progress.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stroke {
    /// Which side: -1 left, +1 right.
    pub side: f64,
    /// Forward (+1) or back-paddling (-1).
    pub direction: f64,
    /// 0..1 through the power phase, then 1..2 through the recovery.
    pub phase: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoonState {
    pub stroke: Option<Stroke>,
    /// The side the last stroke was on, for alternating.
    last_side: f64,
    /// Strokes begun, and the world times of the recent ones.
    pub strokes: Vec<f64>,
}

impl Default for LoonState {
    fn default() -> Self {
        Self {
            stroke: None,
            last_side: -1.0,
            strokes: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LoonTelemetry {
    /// Speed ahead and sideways drift over ground (starboard positive), m/s.
    pub speed: f64,
    pub drift: f64,
    /// Forward and sideways motion through local water, m/s.
    pub water_speed: f64,
    pub water_drift: f64,
    /// Heel, rad, starboard down positive.
    pub heel: f64,
    /// The lowest the gunwale stands over the sea, m.
    pub freeboard: f64,
    /// The blade's force and where it pulled.
    pub blade: FoilForce,
    pub strokes_per_minute: f64,
    pub displaced_m3: f64,
    pub stroke: Option<Stroke>,
}

pub(super) fn forces(craft: &mut Craft, input: &Input, cx: &Context) {
    let specs = craft.specs.clone();
    let s = &specs.loon;
    let p = &s.paddle;
    let hull = craft.hulls.loon.clone();
    let occupied = craft.occupied;
    let com = craft.com;
    let bilge_kg = craft.bilge_kg;
    let h = cx.dt;
    let g = cx.env.gravity.length();
    let up = cx.up;
    let Craft {
        body,
        state,
        telemetry,
        bilge_kg: bilge,
        ..
    } = craft;
    let CraftState::Loon(st) = state else {
        return;
    };

    // The stroke: begun when the last has recovered and a key is held.
    if let Some(stroke) = &mut st.stroke {
        let phase_time = if stroke.phase < 1.0 {
            p.power_s
        } else {
            p.recovery_s
        } as f64;
        stroke.phase += h / phase_time;
        if stroke.phase >= 2.0 {
            st.stroke = None;
        }
    }
    if occupied && st.stroke.is_none() {
        let begin = |side: f64, direction: f64| Stroke {
            side,
            direction,
            phase: 0.0,
        };
        let next = if input.steer > 0.0 {
            // Turning left: paddle on the right.
            Some(begin(1.0, 1.0))
        } else if input.steer < 0.0 {
            Some(begin(-1.0, 1.0))
        } else if input.forward > 0.0 {
            Some(begin(-st.last_side, 1.0))
        } else if input.forward < 0.0 {
            Some(begin(-st.last_side, -1.0))
        } else {
            None
        };
        if let Some(stroke) = next {
            st.last_side = stroke.side;
            st.strokes.push(cx.seconds);
            st.stroke = Some(stroke);
        }
    }
    st.strokes.retain(|t| cx.seconds - t < 6.0);

    let water = cx.water_view();
    let displaced = float(body, &hull, com, &water, s.heave_damping as f64);
    let immersed = (displaced / (body.mass / SEA_DENSITY)).clamp(0.0, 1.5);

    // The blade: dragged through the water during the power phase, or held at
    // the stern as a rudder.
    let blade_at = |x: f64, z: f64| DVec3::new(x, p.depth_m as f64, z) - com;
    let mut blade = FoilForce::default();
    let pull = |body: &mut super::body::RigidBody, local: DVec3, stroke_speed: f64, area: f64| {
        let at = body.point(local);
        let (height, _) = cx.water(at, 0.0);
        let depth = height - (at.length() - cx.env.sea_radius);
        if depth <= -0.05 {
            return FoilForce {
                at,
                ..Default::default()
            };
        }
        let share = ((depth + 0.05) / 0.25).clamp(0.0, 1.0);
        let (_, water) = cx.water(at, depth.max(0.0));
        let blade_velocity = body.velocity_at(at) + body.axis(DVec3::Z * stroke_speed);
        let rel = blade_velocity - water;
        let force = rel * (-0.5 * SEA_DENSITY * p.blade_cd as f64 * area * share * rel.length());
        body.push(force, at);
        FoilForce {
            force,
            at,
            ..Default::default()
        }
    };
    if let Some(stroke) = st.stroke.filter(|s| s.phase < 1.0) {
        let u = stroke.phase;
        let travel = p.stroke_m as f64 * (1.0 - (std::f64::consts::PI * u).cos()) / 2.0;
        let z = if stroke.direction > 0.0 {
            p.catch_z as f64 + travel
        } else {
            p.catch_z as f64 + p.stroke_m as f64 - travel
        };
        let speed = stroke.direction * p.stroke_m as f64 / p.power_s as f64
            * std::f64::consts::FRAC_PI_2
            * (std::f64::consts::PI * u).sin();
        blade = pull(
            body,
            blade_at(stroke.side * p.reach_m as f64, z),
            speed,
            p.blade_m2 as f64,
        );
    } else if occupied && input.rudder != 0.0 {
        let side = -(input.rudder as f64).signum();
        blade = pull(
            body,
            blade_at(side * p.rudder_at[0] as f64, p.rudder_at[1] as f64),
            0.0,
            p.rudder_m2 as f64,
        );
    }

    // The hull's own grip on the water, and its resistance.
    cx.wet_foil(body, &s.lateral, com, 0.0);
    cx.wet_foil(body, &s.skeg, com, 0.0);
    let (_, surface_water) = cx.water(body.position, 0.1);
    resist(
        body,
        &s.resistance,
        com,
        surface_water,
        g,
        SEA_DENSITY,
        immersed.min(1.0),
    );
    let area = if occupied {
        s.windage.area_m2
    } else {
        s.windage_empty_m2
    };
    windage(body, &s.windage, com, cx, area);
    body.twist(body.angular_velocity * -(s.angular_damping as f64));

    // Water aboard.
    let right = body.axis(RIGHT);
    let heel = (-right.dot(up)).clamp(-1.0, 1.0).asin();
    let mut freeboard = f64::INFINITY;
    for point in &hull.rim {
        freeboard = freeboard.min(cx.above_sea(body.point(point.as_dvec3() - com)));
    }
    let flow = bilge_flow(
        &s.bilge,
        &hull,
        cx.env.air.rain_mmh as f64,
        |q| -cx.above_sea(body.point(q.as_dvec3() - com)),
        occupied && input.bail,
    );
    *bilge = (bilge_kg + flow * h).clamp(0.0, s.bilge.capacity_kg as f64);
    if *bilge > 1.0 {
        let slosh = (-right.dot(up) * s.bilge.slosh as f64)
            .clamp(-(s.bilge.slosh_max_m as f64), s.bilge.slosh_max_m as f64);
        let at = v3(s.bilge.at) - com;
        let moved = body.point(at + DVec3::X * slosh) - body.point(at);
        body.twist(moved.cross(cx.env.gravity * *bilge));
    }
    for leg in &s.contacts {
        contact(body, leg, com, cx.env.ground, true);
    }

    let forward = body.axis(FORWARD);
    let flat = |v: DVec3| v - up * v.dot(up);
    let over_ground = flat(body.velocity);
    let through_water = flat(body.velocity - surface_water);
    *telemetry = Telemetry::Loon(LoonTelemetry {
        speed: over_ground.dot(flat(forward).normalize_or_zero()),
        drift: over_ground.dot(flat(right).normalize_or_zero()),
        water_speed: through_water.dot(flat(forward).normalize_or_zero()),
        water_drift: through_water.dot(flat(right).normalize_or_zero()),
        heel,
        freeboard,
        blade,
        strokes_per_minute: st.strokes.len() as f64 * 10.0,
        displaced_m3: displaced,
        stroke: st.stroke,
    });
}
