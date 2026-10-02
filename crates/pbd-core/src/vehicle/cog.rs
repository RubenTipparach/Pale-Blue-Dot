//! The cog (`sail-the-cog`): the harbour's ship, 15 m by 5 m and about 45 t.
//! One square sail on a yard that the braces turn about the mast; a long keel
//! and a stern rudder. It is the Tern's model, scaled: a hull of cells
//! floated on the sea, the sail one foil, the keel and rudder wet foils.
//! It cannot point high: its sail luffs with the wind along its yard, so its
//! best course to windward is about 60 degrees off, and with its leeway it
//! makes good a track little closer than 70.

use super::foil::{self, FoilForce};
use super::hull::{bilge_flow, float, resist};
use super::spec::SquareSailSpec;
use super::{
    AIR_DENSITY, Context, Craft, CraftState, FORWARD, Input, RIGHT, SEA_DENSITY, Telemetry,
    contact, v3, windage,
};
use glam::DVec3;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CogState {
    /// The yard's angle off square, rad; positive turns its starboard arm
    /// forward.
    pub yard: f64,
    /// The tiller: rudder angle, rad.
    pub tiller: f64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct CogTelemetry {
    /// Speed over the ground and through the water in the tangent plane,
    /// m/s.
    pub speed: f64,
    pub water_speed: f64,
    /// The true and apparent wind at the sail, planet frame, m/s.
    pub true_wind: DVec3,
    pub apparent: DVec3,
    /// Where the true and apparent wind come from, off the bow: positive to
    /// starboard, rad.
    pub true_angle: f64,
    pub apparent_angle: f64,
    /// Heel, rad, starboard down positive; water-relative leeway, rad.
    pub heel: f64,
    pub leeway: f64,
    /// Speed made good straight upwind, m/s.
    pub upwind: f64,
    pub sail: FoilForce,
    /// How full the sail stands, 0 luffing to 1 drawing ([`fill`]).
    pub fill: f64,
    pub keel: FoilForce,
    pub yard: f64,
    pub displaced_m3: f64,
}

/// The sail's chord and normal in body axes for a yard at `yard`: the chord
/// along the yard, athwartships when square, and the normal square to it in
/// the horizontal, aft when square.
pub fn sail_axes(yard: f64) -> (DVec3, DVec3) {
    let (sin, cos) = yard.sin_cos();
    (DVec3::new(cos, 0.0, -sin), DVec3::new(sin, 0.0, cos))
}

/// How full the sail stands, 0 luffing to 1 drawing, for the wind at
/// `alpha` to the yard: the wind's angle to the yard's line, whichever end it
/// comes in at (`SquareSailSpec::luff_deg`).
pub fn fill(sail: &SquareSailSpec, alpha: f64) -> f64 {
    let a = alpha.abs().min(std::f64::consts::PI - alpha.abs());
    foil::smoothstep(
        (sail.luff_deg as f64).to_radians(),
        (sail.fill_deg as f64).to_radians(),
        a,
    )
}

pub(super) fn forces(craft: &mut Craft, input: &Input, cx: &Context) {
    let specs = craft.specs.clone();
    let s = &specs.cog;
    let hull = craft.hulls.cog.clone();
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
    let CraftState::Cog(st) = state else {
        return;
    };
    // The controls: the braces turn the yard (the sheet's keys), the tiller
    // steers. Nobody at the helm, both stay as they were left.
    if occupied {
        let reach = (s.sail.brace_deg as f64).to_radians();
        let rate = (s.sail.brace_rate_deg as f64).to_radians();
        st.yard = (st.yard + input.sheet as f64 * rate * h).clamp(-reach, reach);
        let helm = input.steer as f64 * s.rudder_max_rad as f64;
        let turn = s.rudder_rate as f64 * h;
        st.tiller += (helm - st.tiller).clamp(-turn, turn);
    }

    let water = cx.water_view();
    let displaced = float(body, &hull, com, &water, s.heave_damping as f64);
    let immersed = (displaced / (body.mass / SEA_DENSITY)).clamp(0.0, 1.3);

    // The sail, its force at its middle under the yard.
    let effort = v3(s.sail.mast) - com
        + DVec3::Y * ((s.sail.yard_height_m + s.sail.foot_height_m) as f64 / 2.0);
    let at = body.point(effort);
    let true_wind = cx.wind(at);
    let apparent = true_wind - body.velocity_at(at);
    let (chord, normal) = sail_axes(st.yard);
    let sail_spec = foil::FoilSpec {
        at: [0.0; 3],
        chord: [0.0, 0.0, -1.0],
        normal: [1.0, 0.0, 0.0],
        area_m2: s.sail.area_m2(),
        aspect: s.sail.aspect,
        cl_max: s.sail.cl_max,
        cd0: s.sail.cd0,
    };
    let mut full = 0.0;
    let sail = if cx.above_sea(at) > 0.5 {
        let (mut sail, lift) = foil::oriented(
            body,
            effort,
            chord,
            normal,
            &sail_spec,
            true_wind,
            AIR_DENSITY,
            0.0,
            1.0,
            1.0,
        );
        full = fill(&s.sail, sail.alpha);
        sail.force -= lift * (1.0 - full);
        body.push(sail.force, sail.at);
        sail
    } else {
        FoilForce::default()
    };

    // The keel and the rudder, where they are in the water.
    let keel = cx.wet_foil(body, &s.keel, com, 0.0);
    cx.wet_foil(body, &s.rudder, com, st.tiller);
    let (_, surface_water) = cx.water(body.position, 0.5);
    resist(
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

    // Water aboard: rain in the waist, green water over the rail.
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
    *telemetry = Telemetry::Cog(CogTelemetry {
        speed: over_ground.length(),
        water_speed: through_water.length(),
        true_wind,
        apparent,
        true_angle: off_bow(true_wind),
        apparent_angle: off_bow(apparent),
        heel,
        leeway,
        upwind: over_ground.dot(upwind),
        sail,
        fill: full,
        keel,
        yard: st.yard,
        displaced_m3: displaced,
    });
}
