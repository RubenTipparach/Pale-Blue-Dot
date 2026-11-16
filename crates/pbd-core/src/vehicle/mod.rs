//! Vehicles: a sailing keelboat (Tern), a paddle canoe (Loon), the harbour's
//! rowboat and cog (`sail-the-cog`), stepped here and drawn by the app
//! (`vehicles` change). The VTOL tiltrotor (Kestrel) is retired
//! (`retire-kestrel`): a save's record of one is held aside, never flown.
//!
//! A craft steps itself at the substep rate against its `Surroundings`: the sea
//! function, the wind at a point, gravity and the ground. Nothing here knows
//! about Bevy; the app samples the world once per tick and hands it in, and
//! places the Avian body where the craft ends up. The prototype this was held
//! to is `docs/mockups/vehicles.html`.

pub mod body;
pub mod cog;
pub mod foil;
pub mod hold;
pub mod hull;
mod loon;
pub mod record;
pub mod spec;
mod tern;
#[cfg(test)]
mod tests;

pub use cog::{CogState, CogTelemetry};
pub use loon::{LoonState, LoonTelemetry, Stroke};
pub use tern::{SailState, TernState, TernTelemetry};

use crate::sea::{LocalSea, SeaState, SeaTable};
use crate::wind::{AirHere, GustSettings, wind_at};
use body::RigidBody;
use glam::{DQuat, DVec3};
use hull::Hull;
use spec::{ContactSpec, InertiaBox, LoonSpec, Part, VehicleSpecs, WindageSpec};
use std::sync::Arc;

pub const AIR_DENSITY: f64 = 1.225;
pub const SEA_DENSITY: f64 = 1025.0;
/// How far an open boat's floorboards stand over its loaded waterline, m.
pub const FLOOR_OVER_M: f32 = 0.03;

/// Body axes.
pub const FORWARD: DVec3 = DVec3::NEG_Z;
pub const RIGHT: DVec3 = DVec3::X;
pub const UP: DVec3 = DVec3::Y;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Tern,
    Loon,
    Cog,
    /// The harbour's rowboat: the Loon's stroke on the towns' wider,
    /// deeper hull.
    Rowboat,
    /// The inner tube (`small-boats`): a ring the player sits in and
    /// paddles by hand, on the Loon's stroke.
    Tube,
    /// The raft (`small-boats`): logs lashed side by side, paddled, with a
    /// hold.
    Raft,
}

impl Kind {
    pub const ALL: [Kind; 6] = [
        Kind::Tern,
        Kind::Loon,
        Kind::Cog,
        Kind::Rowboat,
        Kind::Tube,
        Kind::Raft,
    ];

    /// The boats paddled on the Loon's stroke, whose paddler can also fish
    /// (`small-boats` design 6).
    pub fn paddled(self) -> bool {
        matches!(self, Kind::Loon | Kind::Rowboat | Kind::Tube | Kind::Raft)
    }

    /// The good a made boat is carried as, by its code (`small-boats`
    /// design 3); none for a kind that is not made.
    pub fn good_code(self) -> Option<&'static str> {
        match self {
            Kind::Tube => Some("inner-tube"),
            Kind::Raft => Some("raft"),
            Kind::Rowboat => Some("rowboat"),
            _ => None,
        }
    }

    /// The kind a boat good launches.
    pub fn of_good_code(code: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.good_code() == Some(code))
    }

    pub fn name(self) -> &'static str {
        match self {
            Kind::Tern => "Tern",
            Kind::Loon => "Loon",
            Kind::Rowboat => "Rowboat",
            Kind::Cog => "Cog",
            Kind::Tube => "Inner tube",
            Kind::Raft => "Raft",
        }
    }

    /// The saved name.
    pub fn key(self) -> &'static str {
        match self {
            Kind::Tern => "tern",
            Kind::Loon => "loon",
            Kind::Rowboat => "rowboat",
            Kind::Cog => "cog",
            Kind::Tube => "tube",
            Kind::Raft => "raft",
        }
    }

    pub fn from_key(key: &str) -> Option<Kind> {
        match Kind::parse(key)? {
            KindKey::Live(kind) => Some(kind),
            KindKey::Retired => None,
        }
    }

    /// The saved names of kinds this build has retired (`retire-kestrel`):
    /// a save's record of one is held aside as read, never drawn, boarded or
    /// stepped, and written back unchanged.
    pub const RETIRED: [&'static str; 1] = ["kestrel"];

    /// A saved name: a kind this build flies, a retired one, or `None` for
    /// one it has never known.
    pub fn parse(key: &str) -> Option<KindKey> {
        if Kind::RETIRED.contains(&key) {
            return Some(KindKey::Retired);
        }
        Kind::ALL
            .into_iter()
            .find(|k| k.key() == key)
            .map(KindKey::Live)
    }
}

/// What a saved kind name is to this build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KindKey {
    Live(Kind),
    /// A kind the game has retired: its records are kept, not flown.
    Retired,
}

/// The world round one craft for one tick, sampled by the app.
pub struct Surroundings<'a> {
    pub sea: &'a SeaTable,
    /// The sea state at the craft.
    pub sea_state: SeaState,
    /// The tide (`crate::sea::tide_m`): the moon's direction and the range.
    pub tide: glam::Vec4,
    pub sea_radius: f64,
    /// Water depth under the craft, m; zero on land.
    pub depth: f32,
    pub air: AirHere,
    pub gusts: &'a GustSettings,
    /// Gravity's acceleration at the craft, m/s^2.
    pub gravity: DVec3,
    /// The ocean's surface current at the craft, m/s.
    pub current: DVec3,
    /// The radius of the solid ground (land, seabed, a placed block) under a
    /// body-local point.
    pub ground: &'a dyn Fn(DVec3) -> f64,
    /// World time at the start of the tick, s.
    pub seconds: f64,
}

/// What a pilot asks for. Each craft reads its own fields, all -1..1.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Input {
    /// Tern: turn left (+). Loon: turn left (+), by paddling on the right.
    pub steer: f32,
    /// Tern: sheet in (+) or ease (-). Cog: brace the yard round, its
    /// starboard arm forward (+) or aft (-).
    pub sheet: f32,
    /// Tern: crew to starboard (+) or port (-).
    pub crew: f32,
    /// Loon: forward strokes (+) or back-paddling (-).
    pub forward: f32,
    /// Loon: the blade at the stern as a rudder, left (+) or right (-).
    pub rudder: f32,
    /// Boats: bail.
    pub bail: bool,
}

/// A line to a bollard or an anchor's rode: slack until taut.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mooring {
    /// Where it is made fast, body-local.
    pub at: DVec3,
    pub length: f64,
    /// An anchor rather than a bollard.
    pub anchored: bool,
}

/// Each craft's own controls and what they are doing.
#[derive(Clone, Debug, PartialEq)]
pub enum CraftState {
    Tern(TernState),
    Loon(LoonState),
    Cog(CogState),
}

/// What the instruments read.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Telemetry {
    #[default]
    None,
    Tern(TernTelemetry),
    Loon(LoonTelemetry),
    Cog(CogTelemetry),
}

/// Hulls are the same for every craft of a kind: built once.
#[derive(Clone, Debug)]
pub struct Hulls {
    pub tern: Arc<Hull>,
    pub loon: Arc<Hull>,
    pub cog: Arc<Hull>,
    pub rowboat: Arc<Hull>,
    pub tube: Arc<Hull>,
    pub raft: Arc<Hull>,
}

impl Hulls {
    pub fn new(specs: &VehicleSpecs) -> Self {
        Self {
            tern: Arc::new(Hull::new(specs.tern.hull)),
            loon: Arc::new(Hull::new(specs.loon.hull)),
            rowboat: Arc::new(Hull::new(specs.rowboat.hull)),
            tube: Arc::new(Hull::new(specs.tube.hull)),
            raft: Arc::new(Hull::new(specs.raft.hull)),
            cog: Arc::new(Hull::new(specs.cog.hull)),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Craft {
    /// Stable for the craft's life and saved with it.
    pub id: u64,
    pub kind: Kind,
    pub body: RigidBody,
    /// The centre of mass in the craft's reference frame: it moves when the
    /// crew boards or water comes aboard.
    pub com: DVec3,
    pub occupied: bool,
    /// Water aboard, kg.
    pub bilge_kg: f64,
    pub mooring: Option<Mooring>,
    /// A harbour boat's berth (`cities-in-the-world` task 4.2b): its
    /// harbour's site and its number there, so the harbour never makes it
    /// twice. `None` for every other craft.
    pub berth: Option<(u32, u32)>,
    /// Made by the player and launched from a boat good (`small-boats`
    /// design 4): only such a craft is taken up again.
    pub made: bool,
    pub state: CraftState,
    pub telemetry: Telemetry,
    specs: Arc<VehicleSpecs>,
    hulls: Hulls,
    /// Seconds since the mass was last worked out.
    reweigh: f64,
    /// How far down a Tern's keel was when the mass was last worked out.
    keel_weighed: f64,
}

/// What every force in a substep shares.
pub(crate) struct Context<'a> {
    pub env: &'a Surroundings<'a>,
    pub sea: LocalSea,
    /// Local up at the craft.
    pub up: DVec3,
    /// Ground radius under the craft.
    pub ground: f64,
    pub seconds: f64,
    pub dt: f64,
}

impl Context<'_> {
    /// The wind at a planet-frame point.
    pub fn wind(&self, at: DVec3) -> DVec3 {
        let r = at.length();
        let height = r - self.env.sea_radius.max(self.ground);
        wind_at(
            self.env.gusts,
            &self.env.air,
            at.as_vec3(),
            height.max(0.0) as f32,
            self.seconds,
        )
        .as_dvec3()
    }

    /// The sea over a planet-frame point: its surface's height over the sea
    /// radius, and the water's velocity `depth` metres under it.
    pub fn water(&self, at: DVec3, depth: f64) -> (f64, DVec3) {
        let p = self
            .sea
            .at(at.as_vec3(), self.env.sea_radius as f32, depth as f32);
        (p.height as f64, p.velocity.as_dvec3() + self.env.current)
    }

    /// How far above the sea surface a point is, m.
    pub fn above_sea(&self, at: DVec3) -> f64 {
        at.length()
            - self.env.sea_radius
            - self.sea.height(at.as_vec3(), self.env.sea_radius as f32) as f64
    }

    /// A hydrofoil samples immersion and water velocity at its own force
    /// point. Another surface's wetness cannot switch this one on or off.
    pub fn wet_foil(
        &self,
        body: &mut RigidBody,
        spec: &foil::FoilSpec,
        com: DVec3,
        deflection: f64,
    ) -> foil::FoilForce {
        let at = body.point(v3(spec.at) - com);
        let depth = -self.above_sea(at);
        if depth <= 0.0 {
            return foil::FoilForce {
                at,
                ..Default::default()
            };
        }
        let (_, velocity) = self.water(at, depth);
        foil::apply(body, spec, com, velocity, SEA_DENSITY, deflection, 1.0, 1.0)
    }

    pub fn water_view(&self) -> hull::Water<'_> {
        hull::Water {
            sea: &self.sea,
            radius: self.env.sea_radius,
            density: SEA_DENSITY,
            gravity: self.env.gravity.length(),
        }
    }
}

/// Mass, centre of mass and principal inertia from point masses and a box.
pub fn mass_model(parts: &[Part], spread: &InertiaBox) -> (f64, DVec3, DVec3) {
    let mass: f64 = parts.iter().map(|p| p.mass_kg as f64).sum();
    let com = parts
        .iter()
        .map(|p| DVec3::from(p.at.map(f64::from)) * p.mass_kg as f64)
        .sum::<DVec3>()
        / mass.max(1e-9);
    let [w, h, l] = spread.size_m.map(f64::from);
    let m = spread.mass_kg as f64;
    let mut inertia = DVec3::new(
        m / 12.0 * (h * h + l * l),
        m / 12.0 * (w * w + l * l),
        m / 12.0 * (w * w + h * h),
    );
    for p in parts {
        let d = DVec3::from(p.at.map(f64::from)) - com;
        let pm = p.mass_kg as f64;
        inertia += DVec3::new(
            pm * (d.y * d.y + d.z * d.z),
            pm * (d.x * d.x + d.z * d.z),
            pm * (d.x * d.x + d.y * d.y),
        );
    }
    (mass, com, inertia)
}

pub(crate) fn v3(a: [f32; 3]) -> DVec3 {
    DVec3::from(a.map(f64::from))
}

/// A spring-damper against the ground under a point, with friction along and
/// across the craft. Returns the normal force, N.
pub(crate) fn contact(
    body: &mut RigidBody,
    spec: &ContactSpec,
    com: DVec3,
    ground: &dyn Fn(DVec3) -> f64,
    braked: bool,
) -> f64 {
    let at = body.point(v3(spec.at) - com);
    let r = at.length();
    let up = at / r;
    let depth = ground(at) - r;
    if depth <= 0.0 {
        return 0.0;
    }
    let v = body.velocity_at(at);
    let vn = v.dot(up);
    let normal = (spec.stiffness as f64 * depth - spec.damping as f64 * vn).max(0.0);
    let tangent = v - up * vn;
    let forward = body.axis(FORWARD);
    let forward = (forward - up * forward.dot(up)).normalize_or_zero();
    let along = tangent.dot(forward);
    let side = tangent - forward * along;
    let mu_along = if braked {
        spec.friction_braked
    } else {
        spec.friction_rolling
    } as f64;
    let mut force = up * normal;
    force -= forward * along.signum() * (mu_along * normal).min(6000.0 * along.abs());
    let side_speed = side.length();
    if side_speed > 1e-4 {
        force -= side / side_speed * (spec.friction_side as f64 * normal).min(6000.0 * side_speed);
    }
    body.push(force, at);
    normal
}

/// Drag of a craft's parts in the air, per body axis.
pub(crate) fn windage(
    body: &mut RigidBody,
    spec: &WindageSpec,
    com: DVec3,
    cx: &Context,
    area: [f32; 3],
) {
    let at = body.point(v3(spec.at) - com);
    let air = cx.wind(at) - body.velocity_at(at);
    let local = body.local(air);
    let a = v3(area);
    let force = DVec3::new(
        a.x * local.x.abs() * local.x,
        a.y * local.y.abs() * local.y,
        a.z * local.z.abs() * local.z,
    ) * (0.5 * AIR_DENSITY);
    body.push(body.axis(force), at);
}

impl Craft {
    /// A new craft of a kind, its reference point at `position` (body-local)
    /// turned by `orientation`.
    pub fn new(
        kind: Kind,
        id: u64,
        specs: Arc<VehicleSpecs>,
        hulls: Hulls,
        position: DVec3,
        orientation: DQuat,
    ) -> Self {
        let state = match kind {
            Kind::Tern => CraftState::Tern(TernState::default()),
            Kind::Loon | Kind::Rowboat | Kind::Tube | Kind::Raft => {
                CraftState::Loon(LoonState::default())
            }
            Kind::Cog => CraftState::Cog(CogState::default()),
        };
        let mut craft = Self {
            id,
            kind,
            body: RigidBody::new(1.0, DVec3::ONE),
            com: DVec3::ZERO,
            occupied: false,
            bilge_kg: 0.0,
            mooring: None,
            berth: None,
            made: false,
            state,
            telemetry: Telemetry::None,
            specs,
            hulls,
            reweigh: 0.0,
            keel_weighed: 1.0,
        };
        craft.weigh();
        craft.set_reference_pose(position, orientation);
        craft
    }

    pub fn specs(&self) -> &VehicleSpecs {
        &self.specs
    }

    /// Where an open boat's floorboards are laid (`cities-in-the-world`
    /// task 4.2b): [`FLOOR_OVER_M`] over the still water it floats in with
    /// its paddler aboard, along the trim the paddler gives it, so the sea's
    /// one sheet stays under them. Their height in the reference frame
    /// amidships, m, and how far they rise per metre aft. Only the Loon and
    /// the rowboat are open.
    pub fn floor(&self) -> Option<(f32, f32)> {
        if !matches!(self.kind, Kind::Loon | Kind::Rowboat) {
            return None;
        }
        let s = self.loon_spec();
        let loaded: Vec<Part> = s.parts.iter().copied().chain([s.paddler]).collect();
        let kg: f32 = loaded.iter().map(|p| p.mass_kg).sum();
        let com_z = loaded.iter().map(|p| p.mass_kg * p.at[2]).sum::<f32>() / kg;
        let (middle, rise) = self.hull()?.waterline(kg as f64, com_z as f64, SEA_DENSITY);
        Some((middle + FLOOR_OVER_M, rise))
    }

    /// The paddled spec this craft is rowed by: the Loon's own, or the
    /// rowboat's. Only meaningful for those two kinds.
    pub fn loon_spec(&self) -> &LoonSpec {
        match self.kind {
            Kind::Rowboat => &self.specs.rowboat,
            Kind::Tube => &self.specs.tube,
            Kind::Raft => &self.specs.raft,
            _ => &self.specs.loon,
        }
    }

    pub fn hull(&self) -> Option<&Arc<Hull>> {
        match self.kind {
            Kind::Tern => Some(&self.hulls.tern),
            Kind::Loon => Some(&self.hulls.loon),
            Kind::Rowboat => Some(&self.hulls.rowboat),
            Kind::Tube => Some(&self.hulls.tube),
            Kind::Raft => Some(&self.hulls.raft),
            Kind::Cog => Some(&self.hulls.cog),
        }
    }

    /// Where the reference point is: what the drawing is built about.
    pub fn reference_position(&self) -> DVec3 {
        self.body.position - self.body.orientation * self.com
    }

    pub fn set_reference_pose(&mut self, position: DVec3, orientation: DQuat) {
        self.body.orientation = orientation;
        self.body.position = position + orientation * self.com;
    }

    /// A point given in the reference frame, planet frame.
    pub fn reference_point(&self, local: [f32; 3]) -> DVec3 {
        self.body.point(v3(local) - self.com)
    }

    /// Work the mass out again: the crew, the water aboard. The body stays
    /// where it is; only its centre of mass moves.
    pub(crate) fn weigh(&mut self) {
        let specs = self.specs.clone();
        let bilge = self.bilge_kg as f32;
        let (mass, com, inertia) = match self.kind {
            Kind::Tern => {
                let s = &specs.tern;
                let mut parts = s.parts.to_vec();
                // The ballast rides in the keel (`TernSpec::lift`).
                let down = match &self.state {
                    CraftState::Tern(st) => st.keel,
                    _ => 1.0,
                };
                parts[s.lift.ballast].at[1] += s.lift.raised_m(down) as f32;
                self.keel_weighed = down;
                parts.push(Part {
                    mass_kg: bilge,
                    at: s.bilge.at,
                });
                if self.occupied {
                    parts.push(Part {
                        mass_kg: s.crew.mass_kg,
                        at: s.crew.at,
                    });
                }
                mass_model(&parts, &s.inertia_box)
            }
            Kind::Loon | Kind::Rowboat | Kind::Tube | Kind::Raft => {
                let s = self.loon_spec();
                let mut parts = s.parts.to_vec();
                parts.push(Part {
                    mass_kg: bilge,
                    at: s.bilge.at,
                });
                if self.occupied {
                    parts.push(s.paddler);
                }
                mass_model(&parts, &s.inertia_box)
            }
            Kind::Cog => {
                let s = &specs.cog;
                let mut parts = s.parts.to_vec();
                parts.push(Part {
                    mass_kg: bilge,
                    at: s.bilge.at,
                });
                if self.occupied {
                    parts.push(s.helmsman);
                }
                mass_model(&parts, &s.inertia_box)
            }
        };
        let reference = self.reference_position();
        self.body.mass = mass;
        self.body.inertia = inertia;
        self.com = com;
        self.body.position = reference + self.body.orientation * com;
    }

    /// Someone takes the controls.
    pub fn board(&mut self) {
        self.occupied = true;
        self.weigh();
    }

    /// They leave; the craft stays, under the unattended policy.
    pub fn leave(&mut self) {
        self.occupied = false;
        match &mut self.state {
            CraftState::Tern(s) => {
                s.sheet = 1.0;
                s.crew = 0.0;
            }
            CraftState::Loon(_) | CraftState::Cog(_) => {}
        }
        self.weigh();
    }

    /// The seat's eye, planet frame.
    pub fn eye(&self) -> DVec3 {
        let seat = match self.kind {
            Kind::Tern => self.specs.tern.seat,
            Kind::Loon | Kind::Rowboat | Kind::Tube | Kind::Raft => self.loon_spec().seat,
            Kind::Cog => self.specs.cog.seat,
        };
        let mut eye = seat.eye;
        if let CraftState::Tern(s) = &self.state {
            eye[0] += s.crew as f32;
        }
        self.reference_point(eye)
    }

    /// Where someone stepping off is put, planet frame.
    pub fn exit(&self) -> DVec3 {
        let seat = self.seat();
        self.reference_point(seat.exit)
    }

    pub fn seat(&self) -> spec::SeatSpec {
        match self.kind {
            Kind::Tern => self.specs.tern.seat,
            Kind::Loon | Kind::Rowboat | Kind::Tube | Kind::Raft => self.loon_spec().seat,
            Kind::Cog => self.specs.cog.seat,
        }
    }

    /// Where a mooring line is made fast on board, planet frame.
    pub fn bow(&self) -> DVec3 {
        match self.kind {
            Kind::Tern => self.reference_point(self.specs.tern.bow),
            Kind::Loon | Kind::Rowboat | Kind::Tube | Kind::Raft => {
                self.reference_point(self.loon_spec().bow)
            }
            Kind::Cog => self.reference_point(self.specs.cog.bow),
        }
    }

    /// Advance `dt` seconds in `substeps` equal steps.
    pub fn step(&mut self, dt: f64, substeps: u32, input: &Input, env: &Surroundings) {
        let substeps = substeps.max(1);
        let h = dt / substeps as f64;
        let input = if self.occupied {
            *input
        } else {
            Input::default()
        };
        for i in 0..substeps {
            let seconds = env.seconds + h * i as f64;
            let up = self.body.position.normalize_or(DVec3::Y);
            let sea = env
                .sea
                .local(&env.sea_state, up.as_vec3(), env.depth, seconds)
                .with_tide(env.tide);
            let cx = Context {
                env,
                sea,
                up,
                ground: (env.ground)(self.body.position),
                seconds,
                dt: h,
            };
            self.body.push_centre(env.gravity * self.body.mass);
            match self.kind {
                Kind::Tern => tern::forces(self, &input, &cx),
                Kind::Loon | Kind::Rowboat | Kind::Tube | Kind::Raft => {
                    loon::forces(self, &input, &cx)
                }
                Kind::Cog => cog::forces(self, &input, &cx),
            }
            self.moor();
            self.body.integrate(h);
        }
        self.reweigh += dt;
        // A keel that moved moves the ballast: weigh again at once.
        let keel_moved =
            matches!(&self.state, CraftState::Tern(s) if (s.keel - self.keel_weighed).abs() > 0.05);
        if self.hull().is_some() && (self.reweigh >= 0.5 || keel_moved) {
            self.reweigh = 0.0;
            self.weigh();
        }
    }

    /// The pull of a taut line.
    fn moor(&mut self) {
        let Some(line) = self.mooring else {
            return;
        };
        let bow = self.bow();
        let reach = line.at - bow;
        let length = reach.length();
        if length <= line.length {
            return;
        }
        let k = (40.0 * self.body.mass).min(4000.0);
        let n = reach / length;
        let closing = self.body.velocity_at(bow).dot(n);
        let pull =
            (k * (length - line.length) - 1.6 * (k * self.body.mass).sqrt() * closing).max(0.0);
        self.body.push(n * pull, bow);
    }

    /// Guard: a craft whose state left its domain is put back where it was
    /// made, rather than carrying a NaN into every picture after it.
    pub fn is_finite(&self) -> bool {
        self.body.is_finite() && self.bilge_kg.is_finite()
    }
}
