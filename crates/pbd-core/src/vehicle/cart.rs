//! Carts on land (`land-transport` design 9 to 11): the hand cart a player
//! pulls and the horse cart a player drives from its seat.
//!
//! A cart is a vehicle record in the journal (kind [`CART_KIND`]), its id
//! from the fleet's counter so its hold is the boats' own `hold` record.
//! Nothing here floats: a cart has no hull, and is moved kinematically along
//! its heading over the ground the app hands in, at its road speed times the
//! wheel factor of the surface under its wheels (`roads::wheel_factor`).
//!
//! Its pose is in the body's frame, in `f64`, with the frame named (as the
//! townsfolk are, `townsfolk` design 25): `at` is the ground point midway
//! between its wheels and `heading` the unit tangent it faces. Up is taken
//! from `at` in `f64` before any cast.

use crate::locale::{Localized, tr};
use crate::records::{Record, Records};
use crate::vehicle::record::PLANET_FRAME;
use glam::DVec3;
use serde::{Deserialize, Serialize};

/// The record kind a cart is stored as, and the schema this build writes.
pub const CART_KIND: &str = "cart";
pub const CART_SCHEMA: u32 = 1;

/// The two carts of phase 3.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CartKind {
    /// Two wheels and two handles, pulled on foot.
    #[default]
    HandCart,
    /// Two wheels, a seat, and a horse in the shafts.
    HorseCart,
}

impl CartKind {
    pub const ALL: [CartKind; 2] = [CartKind::HandCart, CartKind::HorseCart];

    /// The good it is carried as, and the recipe that makes it.
    pub fn code(self) -> &'static str {
        match self {
            CartKind::HandCart => "hand-cart",
            CartKind::HorseCart => "horse-cart",
        }
    }

    /// The kind a good sets down.
    pub fn of_code(code: &str) -> Option<CartKind> {
        CartKind::ALL.into_iter().find(|k| k.code() == code)
    }

    /// Its locale key part.
    pub fn key(self) -> &'static str {
        match self {
            CartKind::HandCart => "hand_cart",
            CartKind::HorseCart => "horse_cart",
        }
    }

    /// Its name, `vehicle.<key>.name`.
    pub fn name(self) -> Localized {
        tr(&format!("vehicle.{}.name", self.key()))
    }

    /// Its name as a panel's title, `vehicle.<key>.title`.
    pub fn title(self) -> Localized {
        tr(&format!("vehicle.{}.title", self.key()))
    }
}

/// How one kind of cart is made and runs (`vehicles.ron`, `carts`).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CartSpec {
    /// Its speed on gravel, m/s (design 10).
    pub road_mps: f32,
    /// Slots in its hold.
    pub hold: usize,
    /// How fast it comes up to speed and slows, m/s^2.
    pub accel_mps2: f32,
    /// Backing, as a share of its road speed.
    pub reverse_share: f32,
    /// How fast it turns at full lock, rad/s.
    pub turn_rad_s: f32,
    /// Its wheels' radius, m: what their turn is worked from.
    pub wheel_radius_m: f32,
    /// How far each wheel stands to the side of `at`, m.
    pub track_m: f32,
    /// From `at` to the handles (a hand cart) or the horse's chest (a horse
    /// cart), m, forward.
    pub reach_m: f32,
    /// The most the ground may rise under it between two steps, m: a step
    /// of more than this stops a wheel, as a wall does (design 9).
    pub max_step_m: f32,
}

impl CartSpec {
    pub fn validate(&self, at: &str) -> Result<(), String> {
        let positive = [
            self.road_mps,
            self.accel_mps2,
            self.reverse_share,
            self.turn_rad_s,
            self.wheel_radius_m,
            self.track_m,
            self.reach_m,
            self.max_step_m,
        ];
        if positive.iter().any(|v| !(v.is_finite() && *v > 0.0)) {
            return Err(format!("{at}: every number is positive"));
        }
        if self.hold == 0 || self.hold > crate::vehicle::hold::HOLD_MAX {
            return Err(format!(
                "{at}: a hold has 1 to {} slots",
                crate::vehicle::hold::HOLD_MAX
            ));
        }
        Ok(())
    }
}

/// The species that may be put in a horse cart's shafts (`land-transport`
/// designs 11 and 13), by id in the home roster.
pub const PULLERS: [&str; 2] = ["horse", "plodder"];

/// Both carts' specs.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CartSpecs {
    pub hand_cart: CartSpec,
    pub horse_cart: CartSpec,
    /// A plodder's speed in the shafts on gravel, m/s (design 13): the
    /// cart runs at the lower of this and its own road speed.
    pub plodder_mps: f32,
    /// Riding a plodder or a horse (design 14).
    pub riding: super::ride::RideSpecs,
}

impl Default for CartSpecs {
    /// Design 10's numbers: a hand cart 6 m/s on the road with 9 slots, a
    /// horse cart 9 m/s with 24.
    fn default() -> Self {
        Self {
            hand_cart: CartSpec {
                road_mps: 6.0,
                hold: 9,
                accel_mps2: 6.0,
                reverse_share: 0.5,
                turn_rad_s: 2.2,
                wheel_radius_m: 0.45,
                track_m: 0.62,
                reach_m: 1.35,
                max_step_m: 0.55,
            },
            horse_cart: CartSpec {
                road_mps: 9.0,
                hold: 24,
                accel_mps2: 2.5,
                reverse_share: 0.25,
                turn_rad_s: 0.9,
                wheel_radius_m: 0.7,
                track_m: 0.85,
                reach_m: 2.6,
                max_step_m: 0.55,
            },
            plodder_mps: 6.0,
            riding: Default::default(),
        }
    }
}

impl CartSpecs {
    pub fn of(&self, kind: CartKind) -> &CartSpec {
        match kind {
            CartKind::HandCart => &self.hand_cart,
            CartKind::HorseCart => &self.horse_cart,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        self.hand_cart.validate("carts.hand_cart")?;
        self.horse_cart.validate("carts.horse_cart")?;
        if !(self.plodder_mps.is_finite() && self.plodder_mps > 0.0) {
            return Err("carts.plodder_mps: a positive speed".into());
        }
        self.riding.validate()
    }

    /// The horse cart's spec with the animal of species `id` in its shafts:
    /// its road speed the lower of the cart's and the animal's own (design
    /// 13). A horse pulls at the cart's own.
    pub fn pulled_by(&self, id: &str) -> CartSpec {
        let mut spec = self.horse_cart;
        if id == "plodder" {
            spec.road_mps = spec.road_mps.min(self.plodder_mps);
        }
        spec
    }
}

/// A cart's record body: `rec @p cart <id> 1 (...)`. Every field but the
/// kind has a default, so a body written by an earlier schema reads under a
/// later build.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CartBody {
    pub kind: CartKind,
    /// The frame `at` and `heading` are in.
    pub frame: String,
    /// The ground point between its wheels, body-local, m.
    pub at: [f64; 3],
    /// The tangent it faces, unit.
    pub heading: [f64; 3],
    /// The kept animal in its shafts (a horse cart's), by its record id.
    pub horse: Option<u64>,
    /// Who it belongs to; nobody yet, in a world with one player.
    pub owner: Option<String>,
    /// Damage taken, 0 to 1; nothing damages a cart yet.
    pub damage: f32,
    /// Taken up again into the pack: the record is kept, the cart is not
    /// in the world.
    pub gone: bool,
}

impl Default for CartBody {
    fn default() -> Self {
        Self {
            kind: CartKind::HandCart,
            frame: PLANET_FRAME.into(),
            at: [0.0, 1.0, 0.0],
            heading: [1.0, 0.0, 0.0],
            horse: None,
            owner: None,
            damage: 0.0,
            gone: false,
        }
    }
}

/// What a driver asks for, each -1..1: forward (+) or back, and a turn to
/// the left (+).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Drive {
    pub forward: f32,
    pub turn: f32,
}

/// A cart in the world.
#[derive(Clone, Debug, PartialEq)]
pub struct Cart {
    /// Stable for the cart's life and saved with it; shared with the fleet's
    /// ids, so its hold is `hold <id>`.
    pub id: u64,
    pub kind: CartKind,
    /// The ground point between its wheels, body-local, m.
    pub at: DVec3,
    /// The unit tangent it faces.
    pub heading: DVec3,
    pub horse: Option<u64>,
    pub owner: Option<String>,
    pub damage: f32,
    /// Its speed along its heading now, m/s. Not saved: a loaded cart is
    /// standing.
    pub speed: f64,
    /// How far its wheels have turned, rad: the drawing's. Not saved.
    pub wheel_turn: f64,
    /// How far it has rolled, m: what a horse's gait is worked from.
    pub rolled_m: f64,
}

/// The unit up at a body-local point, in `f64`.
fn up_of(point: DVec3) -> DVec3 {
    point.normalize_or(DVec3::Y)
}

/// `v` laid along the ground at `up`, unit; `fallback` where it stands
/// straight up.
fn tangent(v: DVec3, up: DVec3, fallback: DVec3) -> DVec3 {
    let t = v - up * v.dot(up);
    t.try_normalize().unwrap_or_else(|| {
        let f = fallback - up * fallback.dot(up);
        f.try_normalize().unwrap_or(up.any_orthonormal_vector())
    })
}

impl Cart {
    /// A new cart of `kind` standing at `at`, facing along `facing` (made
    /// tangent there).
    pub fn new(id: u64, kind: CartKind, at: DVec3, facing: DVec3) -> Self {
        let up = up_of(at);
        Self {
            id,
            kind,
            at,
            heading: tangent(facing, up, DVec3::X),
            horse: None,
            owner: None,
            damage: 0.0,
            speed: 0.0,
            wheel_turn: 0.0,
            rolled_m: 0.0,
        }
    }

    /// Up where it stands.
    pub fn up(&self) -> DVec3 {
        up_of(self.at)
    }

    /// To its right, along the ground.
    pub fn right(&self) -> DVec3 {
        self.heading.cross(self.up())
    }

    /// Where its two wheels touch the ground, left then right.
    pub fn wheels(&self, spec: &CartSpec) -> [DVec3; 2] {
        let side = self.right() * f64::from(spec.track_m);
        [self.at - side, self.at + side]
    }

    /// Where its handles (a hand cart) or the horse's chest (a horse cart)
    /// are, on the ground ahead of it.
    pub fn front(&self, spec: &CartSpec) -> DVec3 {
        self.at + self.heading * f64::from(spec.reach_m)
    }

    /// Driven for `dt` seconds (a horse cart from its seat): the speed eased
    /// toward the ask at its road speed times `factor`, the wheel factor of
    /// the ground under its wheels, then moved along the ground `ground`
    /// gives (a radius under a body-local point). A rise of more than
    /// `max_step_m` between steps stops it where it was. Metres moved.
    pub fn drive(
        &mut self,
        spec: &CartSpec,
        input: Drive,
        factor: f32,
        dt: f64,
        ground: &dyn Fn(DVec3) -> f64,
    ) -> f64 {
        if !(dt.is_finite() && dt > 0.0) {
            return 0.0;
        }
        let forward = input.forward.clamp(-1.0, 1.0);
        let top = f64::from(spec.road_mps * factor.clamp(0.0, 1.0));
        let want = if forward >= 0.0 {
            f64::from(forward) * top
        } else {
            f64::from(forward) * top * f64::from(spec.reverse_share)
        };
        let step = f64::from(spec.accel_mps2) * dt;
        self.speed += (want - self.speed).clamp(-step, step);
        // A cart turns as it rolls: on the spot, barely.
        let rolling = (self.speed.abs() / f64::from(spec.road_mps).max(0.1)).min(1.0);
        let turn =
            f64::from(input.turn.clamp(-1.0, 1.0) * spec.turn_rad_s) * dt * (0.25 + 0.75 * rolling);
        if turn != 0.0 {
            let q = glam::DQuat::from_axis_angle(self.up(), turn);
            self.heading = tangent(q * self.heading, self.up(), self.heading);
        }
        let moved = self.speed * dt;
        self.roll_to(self.at + self.heading * moved, spec, ground)
    }

    /// Pulled by its handles toward `handles` (a hand cart on foot): it
    /// turns to face them and rolls until they are `reach_m` ahead of it,
    /// over the ground, never faster than `most_mps` times `dt`. Metres
    /// moved.
    pub fn pull(
        &mut self,
        spec: &CartSpec,
        handles: DVec3,
        most_mps: f64,
        dt: f64,
        ground: &dyn Fn(DVec3) -> f64,
    ) -> f64 {
        if !(dt.is_finite() && dt > 0.0) {
            return 0.0;
        }
        let up = self.up();
        let toward = handles - self.at;
        let flat = toward - up * toward.dot(up);
        let length = flat.length();
        if length < 1e-6 {
            self.speed = 0.0;
            return 0.0;
        }
        self.heading = tangent(flat, up, self.heading);
        let gap = length - f64::from(spec.reach_m);
        let moved = gap.clamp(-most_mps * dt, most_mps * dt);
        self.speed = moved / dt;
        self.roll_to(self.at + self.heading * moved, spec, ground)
    }

    /// Roll to `next` (body-local) along the ground, unless the ground
    /// steps up there; the wheels turned by the distance. Metres moved.
    fn roll_to(&mut self, next: DVec3, spec: &CartSpec, ground: &dyn Fn(DVec3) -> f64) -> f64 {
        let here = ground(self.at);
        let up = up_of(next);
        let there = ground(next);
        if !(there.is_finite() && here.is_finite()) {
            self.speed = 0.0;
            return 0.0;
        }
        if there - here > f64::from(spec.max_step_m) {
            self.speed = 0.0;
            return 0.0;
        }
        let stood = up * there;
        let moved = (stood - self.at).dot(self.heading);
        self.at = stood;
        self.heading = tangent(self.heading, up, self.heading);
        self.wheel_turn += moved / f64::from(spec.wheel_radius_m);
        self.rolled_m += moved.abs();
        moved.abs()
    }

    /// The cart as its record holds it.
    pub fn body(&self) -> CartBody {
        CartBody {
            kind: self.kind,
            frame: PLANET_FRAME.into(),
            at: self.at.to_array(),
            heading: self.heading.to_array(),
            horse: self.horse,
            owner: self.owner.clone(),
            damage: self.damage,
            gone: false,
        }
    }

    /// Its record.
    pub fn record(&self) -> Record {
        Record::of(CART_KIND, self.id, CART_SCHEMA, &self.body())
    }

    /// A cart back from its record body, or why it cannot be.
    pub fn from_body(id: u64, body: &CartBody) -> Result<Cart, String> {
        if body.frame != PLANET_FRAME {
            return Err(format!("cart {id} is in unknown frame '{}'", body.frame));
        }
        let at = DVec3::from(body.at);
        let heading = DVec3::from(body.heading);
        if !at.is_finite() || !heading.is_finite() || at.length() < 1.0 {
            return Err(format!("cart {id} holds a value that cannot be"));
        }
        let mut cart = Cart::new(id, body.kind, at, heading);
        cart.horse = body.horse;
        cart.owner = body.owner.clone();
        cart.damage = body.damage.clamp(0.0, 1.0);
        Ok(cart)
    }
}

/// The record of a cart taken up into the pack: kept, marked gone.
pub fn gone_record(cart: &Cart) -> Record {
    let mut body = cart.body();
    body.gone = true;
    Record::of(CART_KIND, cart.id, CART_SCHEMA, &body)
}

/// Every cart in the world, by id, that is not gone; a record this build
/// cannot read is left in the save as it was.
pub fn carts(records: &Records) -> Vec<Cart> {
    records
        .of_kind(CART_KIND)
        .filter(|r| r.schema == CART_SCHEMA)
        .filter_map(|r| {
            let body: CartBody = r.read()?;
            (!body.gone).then(|| Cart::from_body(r.id, &body).ok())?
        })
        .collect()
}

/// The highest id any cart record has used, so the fleet's counter never
/// hands it out again.
pub fn highest_id(records: &Records) -> Option<u64> {
    records.of_kind(CART_KIND).map(|r| r.id).max()
}

/// The kept animals in a cart's shafts: a horse is hitched when, and only
/// when, a cart standing in the world names it (design 11).
pub fn hitched(records: &Records) -> std::collections::BTreeSet<u64> {
    carts(records).iter().filter_map(|c| c.horse).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const R: f64 = 4800.0;

    fn flat(p: DVec3) -> f64 {
        let _ = p;
        R
    }

    fn cart(kind: CartKind) -> Cart {
        Cart::new(7, kind, DVec3::new(0.0, R, 0.0), DVec3::X)
    }

    /// Design 10: driven flat out on gravel a horse cart reaches 9 m/s and
    /// a hand cart pulled at most 6; off the road a quarter of that.
    #[test]
    fn a_cart_runs_at_its_road_speed_times_the_factor() {
        let specs = CartSpecs::default();
        specs.validate().unwrap();
        let s = specs.of(CartKind::HorseCart);
        let mut c = cart(CartKind::HorseCart);
        for _ in 0..600 {
            c.drive(
                s,
                Drive {
                    forward: 1.0,
                    turn: 0.0,
                },
                1.0,
                1.0 / 60.0,
                &flat,
            );
        }
        assert!((c.speed - 9.0).abs() < 1e-6, "{}", c.speed);
        let before = c.at;
        for _ in 0..600 {
            c.drive(
                s,
                Drive {
                    forward: 1.0,
                    turn: 0.0,
                },
                0.25,
                1.0 / 60.0,
                &flat,
            );
        }
        assert!((c.speed - 2.25).abs() < 1e-6, "{}", c.speed);
        assert!((c.at.length() - R).abs() < 1e-6, "on the ground");
        assert!(c.at.distance(before) > 20.0);
        assert!(c.wheel_turn > 0.0, "the wheels turned");
    }

    /// A cart turns while it rolls and its heading stays on the ground.
    #[test]
    fn a_cart_turns_along_the_ground() {
        let specs = CartSpecs::default();
        let s = specs.of(CartKind::HorseCart);
        let mut c = cart(CartKind::HorseCart);
        for _ in 0..300 {
            c.drive(
                s,
                Drive {
                    forward: 1.0,
                    turn: 1.0,
                },
                1.0,
                1.0 / 60.0,
                &flat,
            );
        }
        assert!(c.heading.dot(c.up()).abs() < 1e-9);
        assert!((c.heading.length() - 1.0).abs() < 1e-9);
        assert!(c.heading.dot(DVec3::X) < 0.99, "it turned");
    }

    /// A step of more than a layer stops a wheel, as a wall does.
    #[test]
    fn a_step_stops_a_cart() {
        let specs = CartSpecs::default();
        let s = specs.of(CartKind::HorseCart);
        let mut c = cart(CartKind::HorseCart);
        let wall = |p: DVec3| if p.x > 2.0 { R + 1.0 } else { R };
        for _ in 0..600 {
            c.drive(
                s,
                Drive {
                    forward: 1.0,
                    turn: 0.0,
                },
                1.0,
                1.0 / 60.0,
                &wall,
            );
        }
        assert!(
            c.at.x <= 2.0 && c.at.x > 1.0,
            "stopped at the step: {}",
            c.at.x
        );
        assert_eq!(c.speed, 0.0);
    }

    /// A hand cart pulled by its handles follows a handle's length behind
    /// them, no faster than it is allowed.
    #[test]
    fn a_hand_cart_follows_its_handles() {
        let specs = CartSpecs::default();
        let s = specs.of(CartKind::HandCart);
        let mut c = cart(CartKind::HandCart);
        let handles = (DVec3::new(10.0, 0.0, 3.0) + DVec3::new(0.0, R, 0.0)).normalize() * R;
        let mut last = f64::MAX;
        for _ in 0..400 {
            c.pull(s, handles, 6.0, 1.0 / 60.0, &flat);
            assert!(c.speed.abs() <= 6.0 + 1e-9);
            last = c.front(s).distance(handles);
        }
        assert!(last < 0.05, "{last}");
    }

    /// A cart's record reads back as the same cart; a gone one is not in
    /// the world, and the horse it names is hitched only while it stands.
    /// Design 13: a plodder in the shafts runs at its own speed, slower
    /// than a horse's, and an old `vehicles.ron` without the row takes it.
    #[test]
    fn a_plodder_pulls_at_its_own_pace() {
        let specs = CartSpecs::default();
        let mut by_plodder = cart(CartKind::HorseCart);
        let mut by_horse = cart(CartKind::HorseCart);
        let go = Drive {
            forward: 1.0,
            turn: 0.0,
        };
        for _ in 0..600 {
            by_plodder.drive(&specs.pulled_by("plodder"), go, 1.0, 1.0 / 60.0, &flat);
            by_horse.drive(&specs.pulled_by("horse"), go, 1.0, 1.0 / 60.0, &flat);
        }
        assert!(
            (by_plodder.speed - 6.0).abs() < 1e-6,
            "{}",
            by_plodder.speed
        );
        assert!((by_horse.speed - 9.0).abs() < 1e-6, "{}", by_horse.speed);
        let old: CartSpecs = ron::from_str("(hand_cart: (road_mps: 6.0, hold: 9, accel_mps2: 6.0, reverse_share: 0.5, turn_rad_s: 2.2, wheel_radius_m: 0.45, track_m: 0.62, reach_m: 1.35, max_step_m: 0.55))").unwrap();
        assert_eq!(old.plodder_mps, 6.0);
        assert!(old.validate().is_ok());
        assert!(PULLERS.contains(&"plodder") && PULLERS.contains(&"horse"));
    }

    #[test]
    fn a_cart_record_reads_back_and_names_its_horse() {
        let mut c = cart(CartKind::HorseCart);
        c.horse = Some(31);
        c.heading = DVec3::new(0.0, 0.0, -1.0);
        let mut records = Records::new();
        records.put(c.record());
        let text = c.record().line(&crate::records::Author::Player(0));
        let (_, parsed) = Record::parse_line(&text).unwrap();
        let back = Cart::from_body(parsed.id, &parsed.read().unwrap()).unwrap();
        assert_eq!(back.id, 7);
        assert_eq!(back.kind, CartKind::HorseCart);
        assert!(back.at.distance(c.at) < 1e-9);
        assert!(back.heading.distance(c.heading) < 1e-9);
        assert_eq!(back.horse, Some(31));
        assert_eq!(carts(&records).len(), 1);
        assert!(hitched(&records).contains(&31));
        records.put(gone_record(&c));
        assert!(carts(&records).is_empty());
        assert!(hitched(&records).is_empty());
        assert_eq!(highest_id(&records), Some(7));
        let mut bad = c.body();
        bad.at = [f64::NAN, 0.0, 0.0];
        assert!(Cart::from_body(1, &bad).is_err());
    }
}
