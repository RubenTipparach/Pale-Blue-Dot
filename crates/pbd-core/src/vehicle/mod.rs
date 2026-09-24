//! Vehicles: a VTOL tiltrotor (Kestrel), a sailing keelboat (Tern) and a
//! paddle canoe (Loon), stepped here and drawn by the app (`vehicles` change).
//!
//! A craft steps itself at the substep rate against its `Surroundings`: the sea
//! function, the wind at a point, gravity and the ground. Nothing here knows
//! about Bevy; the app samples the world once per tick and hands it in, and
//! places the Avian body where the craft ends up. The prototype this was held
//! to is `docs/mockups/vehicles.html`.

pub mod body;
pub mod foil;
pub mod hull;
mod kestrel;
mod loon;
pub mod record;
pub mod spec;
mod tern;
#[cfg(test)]
mod tests;

pub use kestrel::{KestrelState, KestrelTelemetry};
pub use loon::{LoonState, LoonTelemetry, Stroke};
pub use tern::{SailState, TernState, TernTelemetry};

use crate::sea::{LocalSea, SeaState, SeaTable};
use crate::wind::{AirHere, GustSettings, wind_at};
use body::RigidBody;
use glam::{DQuat, DVec3};
use hull::Hull;
use spec::{ContactSpec, InertiaBox, Part, VehicleSpecs, WindageSpec};
use std::sync::Arc;

pub const AIR_DENSITY: f64 = 1.225;
pub const SEA_DENSITY: f64 = 1025.0;

/// Body axes.
pub const FORWARD: DVec3 = DVec3::NEG_Z;
pub const RIGHT: DVec3 = DVec3::X;
pub const UP: DVec3 = DVec3::Y;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Kestrel,
    Tern,
    Loon,
}

impl Kind {
    pub const ALL: [Kind; 3] = [Kind::Kestrel, Kind::Tern, Kind::Loon];

    pub fn name(self) -> &'static str {
        match self {
            Kind::Kestrel => "Kestrel",
            Kind::Tern => "Tern",
            Kind::Loon => "Loon",
        }
    }

    /// The saved name.
    pub fn key(self) -> &'static str {
        match self {
            Kind::Kestrel => "kestrel",
            Kind::Tern => "tern",
            Kind::Loon => "loon",
        }
    }

    pub fn from_key(key: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.key() == key)
    }
}

/// The world round one craft for one tick, sampled by the app.
pub struct Surroundings<'a> {
    pub sea: &'a SeaTable,
    /// The sea state at the craft.
    pub sea_state: SeaState,
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
    /// Kestrel: nose up (+), roll right (+), nose right (+), climb (+), and
    /// the nacelles up (+) or forward (-).
    pub pitch: f32,
    pub roll: f32,
    pub yaw: f32,
    pub collective: f32,
    pub tilt: f32,
    /// Tern: turn left (+). Loon: turn left (+), by paddling on the right.
    pub steer: f32,
    /// Tern: sheet in (+) or ease (-).
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
    Kestrel(KestrelState),
    Tern(TernState),
    Loon(LoonState),
}

/// What the instruments read.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Telemetry {
    #[default]
    None,
    Kestrel(KestrelTelemetry),
    Tern(TernTelemetry),
    Loon(LoonTelemetry),
}

/// Hulls are the same for every craft of a kind: built once.
#[derive(Clone, Debug)]
pub struct Hulls {
    pub tern: Arc<Hull>,
    pub loon: Arc<Hull>,
}

impl Hulls {
    pub fn new(specs: &VehicleSpecs) -> Self {
        Self {
            tern: Arc::new(Hull::new(specs.tern.hull)),
            loon: Arc::new(Hull::new(specs.loon.hull)),
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
    pub state: CraftState,
    pub telemetry: Telemetry,
    specs: Arc<VehicleSpecs>,
    hulls: Hulls,
    /// Seconds since the mass was last worked out.
    reweigh: f64,
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
            Kind::Kestrel => CraftState::Kestrel(KestrelState::default()),
            Kind::Tern => CraftState::Tern(TernState::default()),
            Kind::Loon => CraftState::Loon(LoonState::default()),
        };
        let mut craft = Self {
            id,
            kind,
            body: RigidBody::new(1.0, DVec3::ONE),
            com: DVec3::ZERO,
            occupied: false,
            bilge_kg: 0.0,
            mooring: None,
            state,
            telemetry: Telemetry::None,
            specs,
            hulls,
            reweigh: 0.0,
        };
        craft.weigh();
        craft.set_reference_pose(position, orientation);
        craft
    }

    pub fn specs(&self) -> &VehicleSpecs {
        &self.specs
    }

    pub fn hull(&self) -> Option<&Arc<Hull>> {
        match self.kind {
            Kind::Kestrel => None,
            Kind::Tern => Some(&self.hulls.tern),
            Kind::Loon => Some(&self.hulls.loon),
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
            Kind::Kestrel => (
                specs.kestrel.mass_kg as f64,
                DVec3::ZERO,
                v3(specs.kestrel.inertia),
            ),
            Kind::Tern => {
                let s = &specs.tern;
                let mut parts = s.parts.to_vec();
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
            Kind::Loon => {
                let s = &specs.loon;
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
            CraftState::Kestrel(s) => s.assist = true,
            CraftState::Tern(s) => {
                s.sheet = 1.0;
                s.crew = 0.0;
            }
            CraftState::Loon(_) => {}
        }
        self.weigh();
    }

    /// The seat's eye, planet frame.
    pub fn eye(&self) -> DVec3 {
        let seat = match self.kind {
            Kind::Kestrel => self.specs.kestrel.seat,
            Kind::Tern => self.specs.tern.seat,
            Kind::Loon => self.specs.loon.seat,
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
            Kind::Kestrel => self.specs.kestrel.seat,
            Kind::Tern => self.specs.tern.seat,
            Kind::Loon => self.specs.loon.seat,
        }
    }

    /// Where a mooring line is made fast on board, planet frame.
    pub fn bow(&self) -> DVec3 {
        match self.kind {
            Kind::Kestrel => self.body.position,
            Kind::Tern => self.reference_point(self.specs.tern.bow),
            Kind::Loon => self.reference_point(self.specs.loon.bow),
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
                .local(&env.sea_state, up.as_vec3(), env.depth, seconds);
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
                Kind::Kestrel => kestrel::forces(self, &input, &cx),
                Kind::Tern => tern::forces(self, &input, &cx),
                Kind::Loon => loon::forces(self, &input, &cx),
            }
            self.moor();
            self.body.integrate(h);
        }
        self.reweigh += dt;
        if self.hull().is_some() && self.reweigh >= 0.5 {
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
