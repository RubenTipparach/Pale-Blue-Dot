//! The Tern: a cat-rigged keelboat. One sail on a boom that swings free with
//! the apparent wind until the sheet stops it; the angle between the stopped
//! sail and the wind is what drives the boat. A ballasted keel stops her
//! sliding sideways and, heeled, brings her upright; the crew hikes out to
//! help. Resistance climbs a wall near hull speed.

use super::foil::{self, FoilForce};
use super::hull::{bilge_flow, float, resist};
use super::{
    AIR_DENSITY, Context, Craft, CraftState, FORWARD, Input, RIGHT, SEA_DENSITY, Telemetry,
    contact, v3, windage,
};
use glam::DVec3;

#[derive(Clone, Debug, PartialEq)]
pub struct TernState {
    /// How far the sheet is out: 0 hauled in, 1 all the way out.
    pub sheet: f64,
    /// The tiller: rudder angle, rad.
    pub tiller: f64,
    /// The boom's angle off the centreline, rad; positive to starboard.
    pub boom: f64,
    /// Where the crew sits across the boat, m; positive to starboard.
    pub crew: f64,
}

impl Default for TernState {
    fn default() -> Self {
        Self {
            sheet: 1.0,
            tiller: 0.0,
            boom: 0.0,
            crew: 0.0,
        }
    }
}

/// What the sail is doing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SailState {
    /// Eased past the wind, flapping: no drive.
    #[default]
    Luffing,
    Driving,
    /// Sheeted too hard for the wind's angle.
    Stalled,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TernTelemetry {
    /// Speed over the ground, m/s.
    pub speed: f64,
    /// Speed through local water in the tangent plane, m/s.
    pub water_speed: f64,
    /// The true and apparent wind at the sail, planet frame, m/s.
    pub true_wind: DVec3,
    pub apparent: DVec3,
    /// Where the apparent wind comes from, off the bow: positive to
    /// starboard, rad.
    pub apparent_angle: f64,
    pub true_angle: f64,
    /// Heel, rad, starboard down positive; water-relative leeway, rad.
    pub heel: f64,
    pub leeway: f64,
    /// Speed made good straight upwind, m/s.
    pub upwind: f64,
    /// Share of hull speed (Froude number over 0.4).
    pub hull_speed: f64,
    pub sail: FoilForce,
    pub keel: FoilForce,
    pub sail_state: SailState,
    pub boom: f64,
    pub sheet: f64,
    pub displaced_m3: f64,
}

pub(super) fn forces(craft: &mut Craft, input: &Input, cx: &Context) {
    let specs = craft.specs.clone();
    let s = &specs.tern;
    let hull = craft.hulls.tern.clone();
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
    let CraftState::Tern(st) = state else {
        return;
    };
    // The controls.
    if occupied {
        st.sheet = (st.sheet - input.sheet as f64 * s.sail.sheet_rate as f64 * h).clamp(0.0, 1.0);
        st.crew = (st.crew + input.crew as f64 * s.crew.hike_rate as f64 * h)
            .clamp(-(s.crew.hike_m as f64), s.crew.hike_m as f64);
    }
    let helm = input.steer as f64 * s.rudder_max_rad as f64;
    let turn = s.rudder_rate as f64 * h;
    st.tiller += (helm - st.tiller).clamp(-turn, turn);

    let water = cx.water_view();
    let displaced = float(body, &hull, com, &water, s.heave_damping as f64);
    let immersed = (displaced / (body.mass / SEA_DENSITY)).clamp(0.0, 1.3);

    // The sail. It is drawn about the mast in the reference frame; its centre
    // of effort is a third of the way up and out along the boom.
    let mast = v3(s.sail.mast) - com;
    let effort_at = |boom: f64| {
        let height = s.sail.boom_height_m as f64
            + (s.sail.head_height_m - s.sail.boom_height_m) as f64 / 3.0;
        let out = s.sail.boom_length_m as f64 / 3.0;
        mast + DVec3::new(boom.sin() * out, height, boom.cos() * out)
    };
    let effort = body.point(effort_at(st.boom));
    let true_wind = cx.wind(effort);
    let apparent = true_wind - body.velocity_at(effort);
    let local = body.local(apparent);
    // Free, the boom lies along the flow; the sheet stops it.
    let free = local.x.atan2(local.z);
    let [lo, hi] = s.sail.sheet_deg.map(|d| (d as f64).to_radians());
    let reach = lo + (hi - lo) * st.sheet;
    let want = free.clamp(-reach, reach);
    let swing = (s.sail.swing_rate as f64 + s.sail.swing_per_mps as f64 * apparent.length()) * h;
    st.boom += (want - st.boom).clamp(-swing, swing);
    let (sin, cos) = st.boom.sin_cos();
    let sail_spec = foil::FoilSpec {
        at: [0.0; 3],
        chord: [0.0, 0.0, -1.0],
        normal: [1.0, 0.0, 0.0],
        area_m2: s.sail.area_m2,
        aspect: s.sail.aspect,
        cl_max: s.sail.cl_max,
        cd0: s.sail.cd0,
    };
    let above = cx.above_sea(body.point(effort_at(st.boom)));
    let sail = if above > 0.2 {
        foil::apply_oriented(
            body,
            effort_at(st.boom),
            DVec3::new(-sin, 0.0, -cos),
            DVec3::new(cos, 0.0, -sin),
            &sail_spec,
            true_wind,
            AIR_DENSITY,
            0.0,
            1.0,
            1.0,
        )
    } else {
        FoilForce::default()
    };

    // The keel and the rudder, where they are in the water.
    let keel = cx.wet_foil(body, &s.keel, com, 0.0);
    cx.wet_foil(body, &s.rudder, com, st.tiller);
    let (_, surface_water) = cx.water(body.position, 0.3);
    let froude = resist(
        body,
        &s.resistance,
        com,
        surface_water,
        g,
        SEA_DENSITY,
        immersed.min(1.0),
    );
    windage(body, &s.windage, com, cx, s.windage.area_m2);
    body.twist(body.angular_velocity * -(s.angular_damping as f64));

    // The crew's weight where they sit, beyond the centre they were counted at.
    if occupied {
        let seat = v3(s.crew.at) - com;
        let hiked = body.point(seat + DVec3::X * st.crew) - body.point(seat);
        body.twist(hiked.cross(cx.env.gravity * s.crew.mass_kg as f64));
    }

    // Water aboard: rain in the cockpit, green water over the coaming.
    let right = body.axis(RIGHT);
    let heel = (-right.dot(up)).clamp(-1.0, 1.0).asin();
    let flow = bilge_flow(
        &s.bilge,
        &hull,
        cx.env.air.rain_mmh as f64,
        |p| {
            let at = body.point(p.as_dvec3() - com);
            -cx.above_sea(at)
        },
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
    let bow = flat(forward).normalize_or_zero();
    let leeway = through_water
        .dot(flat(right).normalize_or_zero())
        .atan2(through_water.dot(bow).max(0.05));
    let upwind = flat(-true_wind).normalize_or_zero();
    let off_bow = |wind: DVec3| {
        let from = body.local(-wind);
        from.x.atan2(-from.z)
    };
    let sail_state = if sail.stall > 0.5 {
        SailState::Stalled
    } else if sail.alpha.abs() < 0.07 {
        SailState::Luffing
    } else {
        SailState::Driving
    };
    *telemetry = Telemetry::Tern(TernTelemetry {
        speed: over_ground.length(),
        water_speed: through_water.length(),
        true_wind,
        apparent,
        apparent_angle: off_bow(apparent),
        true_angle: off_bow(true_wind),
        heel,
        leeway,
        upwind: over_ground.dot(upwind),
        hull_speed: froude / 0.4,
        sail,
        keel,
        sail_state,
        boom: st.boom,
        sheet: st.sheet,
        displaced_m3: displaced,
    });
}
