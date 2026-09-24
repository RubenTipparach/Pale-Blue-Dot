//! The Kestrel: a tiltrotor. Two rotors tilt from lifting straight up to
//! pulling straight ahead, and a wing takes the weight as the speed builds.
//!
//! In the hover the rotors steer it: roll is power moved from one rotor to the
//! other, pitch is cyclic, yaw is the nacelles tilted against each other, all
//! of it scaled by how vertical the nacelles are and by power. In wingborne
//! flight the ailerons, elevator and rudder take over as dynamic pressure
//! grows. Assist is the flight model's dampeners said for a winged craft:
//! attitude and climb held in the hover, drift over the ground cancelled with
//! the stick centred, rates held in wingborne flight.

use super::foil;
use super::{
    AIR_DENSITY, Context, Craft, CraftState, FORWARD, Input, RIGHT, SEA_DENSITY, Telemetry,
    contact, v3,
};
use crate::vehicle::foil::smoothstep;
use glam::DVec3;
use std::f64::consts::FRAC_PI_2;

#[derive(Clone, Debug, PartialEq)]
pub struct KestrelState {
    /// The nacelles' angle: pi/2 lifting straight up, 0 pulling ahead.
    pub nacelle: f64,
    /// Power, 0..1.
    pub throttle: f64,
    /// The lever's position outside the hover's climb hold.
    pub lever: f64,
    climb_integral: f64,
    pub assist: bool,
    pub brake: bool,
    /// Gear legs on the ground.
    pub touching: u32,
}

impl Default for KestrelState {
    fn default() -> Self {
        Self {
            nacelle: FRAC_PI_2,
            throttle: 0.0,
            lever: 0.0,
            climb_integral: 0.0,
            assist: true,
            brake: true,
            touching: 0,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct KestrelTelemetry {
    /// Speed through the air, and over the ground, m/s.
    pub airspeed: f64,
    pub ground_speed: f64,
    pub vertical_speed: f64,
    /// Height over the ground or the sea under it, m.
    pub height: f64,
    /// The wing's angle of attack, rad, and how stalled it is, 0..1.
    pub alpha: f64,
    pub stall: f64,
    /// Share of the lift the wing carries.
    pub wing_share: f64,
    /// Everything holding it up, N.
    pub lift_total: f64,
    pub nacelle: f64,
    pub throttle: f64,
    pub lift: DVec3,
    pub thrust: DVec3,
    pub touching: u32,
    /// Floating: some of its floats are wet.
    pub in_water: bool,
    /// How much CLmax the rain has taken, 0..1.
    pub wet_loss: f64,
    /// The vertical speed at the last touchdown, m/s.
    pub touchdown: f64,
    /// Collective currently commands vertical speed rather than moving the power lever.
    pub climb_hold: bool,
    /// Actual foil deflections, rad: left wing, right wing, elevator, rudder.
    /// Renderers consume these instead of reproducing the assist controller.
    pub surface_deflections: [f64; 4],
}

pub(super) fn forces(craft: &mut Craft, input: &Input, cx: &Context) {
    let specs = craft.specs.clone();
    let s = &specs.kestrel;
    let a = &s.assist;
    let panels = s.wing.panels();
    let occupied = craft.occupied;
    let com = craft.com;
    let Craft {
        body,
        state,
        telemetry,
        ..
    } = craft;
    let CraftState::Kestrel(st) = state else {
        return;
    };
    let h = cx.dt;
    let g = cx.env.gravity.length();
    let up = cx.up;
    let (roll, pitch, yaw, climb) = (
        input.roll as f64,
        input.pitch as f64,
        input.yaw as f64,
        input.collective as f64,
    );
    let tilt = if occupied { input.tilt as f64 } else { 1.0 };
    st.nacelle = (st.nacelle + tilt * s.rotor.tilt_rate as f64 * h).clamp(0.0, FRAC_PI_2);
    if !occupied {
        st.assist = true;
    }
    let forward = body.axis(FORWARD);
    let right = body.axis(RIGHT);
    let air = cx.wind(body.position) - body.velocity;
    let airspeed = air.length();
    let w = body.local(body.angular_velocity);
    let (roll_rate, pitch_rate, yaw_rate) = (-w.z, w.x, -w.y);
    let bank = (-right.dot(up)).clamp(-1.0, 1.0).asin();
    let pitch_angle = forward.dot(up).clamp(-1.0, 1.0).asin();
    let sb = st.nacelle.sin();
    let hover = smoothstep(0.35, 0.85, sb);

    // The stick, raw or through the assist.
    let (mut cr, mut cp, mut cy) = (roll, pitch, yaw);
    if st.assist {
        let flat = |v: DVec3| (v - up * v.dot(up)).normalize_or_zero();
        let drift = body.velocity - up * body.velocity.dot(up);
        let centred = roll == 0.0 && pitch == 0.0;
        let hold = |component: f64| {
            if centred && g > 1e-6 {
                (a.drift_gain as f64 * component / g)
                    .clamp(-a.drift_tilt as f64, a.drift_tilt as f64)
            } else {
                0.0
            }
        };
        let bank_hold = hold(-drift.dot(flat(right)));
        let pitch_hold = hold(drift.dot(flat(forward)));
        let lerp = |x: f64, y: f64, t: f64| x + (y - x) * t;
        let roll_want = lerp(
            roll * a.roll_rate as f64,
            a.attitude_gain[0] as f64 * (roll * a.hover_bank as f64 + bank_hold - bank),
            hover,
        );
        let pitch_want = lerp(
            pitch * a.pitch_rate as f64,
            a.attitude_gain[1] as f64 * (pitch * a.hover_pitch as f64 + pitch_hold - pitch_angle),
            hover,
        );
        cr = ((roll_want - roll_rate) * a.rate_gain[0] as f64).clamp(-1.0, 1.0);
        cp = ((pitch_want - pitch_rate) * a.rate_gain[1] as f64).clamp(-1.0, 1.0);
        cy = ((yaw * a.yaw_rate as f64 - yaw_rate) * a.rate_gain[2] as f64).clamp(-1.0, 1.0);
    }

    // Power: a climb hold in the hover, a lever otherwise.
    let vertical_speed = body.velocity.dot(up);
    let axis = body.axis(DVec3::new(0.0, sb, -st.nacelle.cos()));
    let climb_hold = st.assist && hover > 0.5;
    if climb_hold {
        let want = if occupied {
            climb * a.climb_mps as f64
        } else {
            -(a.unattended_sink_mps as f64)
        };
        if st.touching > 0 && want <= 0.0 {
            st.throttle = (st.throttle - 0.8 * h).max(0.0);
            st.climb_integral = 0.0;
        } else {
            let hold_up = body.mass * g / (2.0 * s.rotor.thrust_n as f64 * axis.dot(up).max(0.3));
            st.climb_integral = (st.climb_integral
                + (want - vertical_speed) * h * a.climb_integral as f64)
                .clamp(-0.4, 0.4);
            st.throttle =
                (hold_up + a.climb_gain as f64 * (want - vertical_speed) + st.climb_integral)
                    .clamp(0.0, 1.0);
        }
        st.lever = st.throttle;
    } else {
        st.lever = if occupied {
            (st.lever + climb * 0.45 * h).clamp(0.0, 1.0)
        } else {
            (st.lever - 0.3 * h).max(0.0)
        };
        st.throttle += (st.lever - st.throttle) * (3.0 * h).min(1.0);
    }

    // The rotors.
    let mut thrust_total = DVec3::ZERO;
    let mut rotor_up = 0.0;
    // Differential thrust redistributes available collective; it cannot
    // start a stopped rotor or demand power beyond either rotor's range.
    let headroom = st.throttle.min(1.0 - st.throttle);
    let differential = (cr * s.rotor.roll_mix as f64 * hover).clamp(-headroom, headroom);
    for side in [-1.0, 1.0] {
        let [x, y, z] = s.rotor.at;
        let at = body.point(DVec3::new(x as f64 * side, y as f64, z as f64) - com);
        let air_here = cx.wind(at) - body.velocity_at(at);
        let inflow = -air_here.dot(axis);
        let lost = (1.0 - inflow / s.rotor.inflow_zero_mps as f64).clamp(0.0, 1.15);
        let surface = (cx.env.ground)(at)
            .max(cx.env.sea_radius + cx.sea.height(at.as_vec3(), cx.env.sea_radius as f32) as f64);
        let clearance = (at.length() - surface).max(0.8);
        let ratio = s.rotor.radius_m as f64 / (4.0 * clearance);
        let ground_effect =
            (1.0 / (1.0 - ratio * ratio)).clamp(1.0, s.rotor.ground_effect_max as f64);
        let power = st.throttle - side * differential;
        let thrust = s.rotor.thrust_n as f64 * power * lost * (1.0 + (ground_effect - 1.0) * sb);
        let across = air_here - axis * air_here.dot(axis);
        let force = axis * thrust + across * (s.rotor.edgewise_drag as f64 * power);
        body.push(force, at);
        thrust_total += axis * thrust;
        rotor_up += force.dot(up);
    }
    let authority = hover * thrust_total.length() / (2.0 * s.rotor.thrust_n as f64);
    body.twist(body.axis(DVec3::new(
        cp * s.rotor.pitch_torque as f64 * authority,
        -cy * s.rotor.yaw_torque as f64 * authority,
        0.0,
    )));

    // The surfaces. Rain wets the wing.
    let wet = (cx.env.air.rain_mmh as f64 / s.wet_full_mmh as f64).min(1.0);
    let lift_scale = 1.0 - s.wet_cl_loss as f64 * wet;
    let drag_scale = 1.0 + s.wet_cd_gain as f64 * wet;
    let flap = s.wing.flap_rad as f64 * sb;
    let aileron = s.wing.aileron_rad as f64 * cr;
    let surface_deflections = [
        flap + aileron,
        flap - aileron,
        -(s.elevator_rad as f64) * cp,
        -(s.rudder_rad as f64) * cy,
    ];
    let mut wing = [foil::FoilForce::default(); 2];
    for (i, (panel, deflect)) in panels.into_iter().zip(surface_deflections).enumerate() {
        let fluid = cx.wind(body.point(v3(panel.at) - com));
        wing[i] = foil::apply(
            body,
            &panel,
            com,
            fluid,
            AIR_DENSITY,
            deflect,
            lift_scale,
            drag_scale,
        );
    }
    let tail_wind = cx.wind(body.point(v3(s.tail.at) - com));
    foil::apply(
        body,
        &s.tail,
        com,
        tail_wind,
        AIR_DENSITY,
        surface_deflections[2],
        1.0,
        1.0,
    );
    let fin_wind = cx.wind(body.point(v3(s.fin.at) - com));
    foil::apply(
        body,
        &s.fin,
        com,
        fin_wind,
        AIR_DENSITY,
        surface_deflections[3],
        1.0,
        1.0,
    );
    let local_air = body.local(air);
    let d = v3(s.body_drag_m2);
    let drag = DVec3::new(
        d.x * local_air.x.abs() * local_air.x,
        d.y * local_air.y.abs() * local_air.y,
        d.z * local_air.z.abs() * local_air.z,
    ) * (0.5 * AIR_DENSITY);
    body.push_centre(body.axis(drag));
    body.twist(body.angular_velocity * -(s.angular_damping as f64));

    // The gear.
    let was = st.touching;
    st.touching = s
        .gear
        .iter()
        .map(|leg| contact(body, leg, com, cx.env.ground, st.brake))
        .filter(|n| *n > 0.0)
        .count() as u32;

    // It floats if it comes down on the sea.
    let mut wet_floats = 0;
    for float in &s.floats {
        let at = body.point(v3(*float) - com);
        let (height, water) = cx.water(at, 0.0);
        let depth = height - (at.length() - cx.env.sea_radius) + 0.3;
        if depth <= 0.0 {
            continue;
        }
        wet_floats += 1;
        let share = (depth / 0.6).min(1.0);
        let dir = at.normalize();
        let rel = body.velocity_at(at) - water;
        let force = dir * (SEA_DENSITY * g * s.float_m3 as f64 * share)
            - rel * (0.5 * SEA_DENSITY * s.float_drag_m2 as f64 * rel.length() * share);
        body.push(force, at);
    }

    let wing_up = wing[0].force.dot(up) + wing[1].force.dot(up);
    let surface = (cx.ground).max(cx.env.sea_radius);
    let previous = match telemetry {
        Telemetry::Kestrel(t) => t.touchdown,
        _ => 0.0,
    };
    *telemetry = Telemetry::Kestrel(KestrelTelemetry {
        airspeed,
        ground_speed: body.velocity.length(),
        vertical_speed,
        height: body.position.length() - surface,
        alpha: (wing[0].alpha + wing[1].alpha) / 2.0 - flap,
        stall: wing[0].stall.max(wing[1].stall),
        wing_share: (wing_up / (wing_up + rotor_up).max(1.0)).clamp(0.0, 1.0),
        lift_total: wing_up + rotor_up,
        nacelle: st.nacelle,
        throttle: st.throttle,
        lift: wing[0].force + wing[1].force,
        thrust: thrust_total,
        touching: st.touching,
        in_water: wet_floats > 2,
        wet_loss: s.wet_cl_loss as f64 * wet,
        touchdown: if was == 0 && st.touching > 0 {
            vertical_speed
        } else {
            previous
        },
        climb_hold,
        surface_deflections,
    });
}
