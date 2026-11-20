//! The critters' rigs and poses (`ocean-environments` design B28, B29),
//! ported from M25's `docs/mockups/ocean/critters.js`: a root on the ground
//! turned to the heading, a body pivot (the hips' height, pitch and roll)
//! with a hind and a chest segment that bend at the spine, a neck and a
//! jaw, four two-bone legs with feet, and a three-part tail. Twenty bones,
//! the sculpt's (`tools/art/critter_sculpt.py` `SPECS`).
//!
//! Rig space is the page's: metres, x forward, y up, z to the animal's
//! left (legs 0 and 2 on the left), the root at the ground under the body's
//! middle.

#![allow(clippy::needless_range_loop)]

use bevy::math::{Affine3A, Quat, Vec3};
use std::f32::consts::PI;

/// The bones in the mesh's order (`critter_sculpt.py` `BONES`).
pub const BONES: [&str; 20] = [
    "body", "hind", "chest", "neck", "jaw", "hip0", "knee0", "foot0", "hip1", "knee1", "foot1",
    "hip2", "knee2", "foot2", "hip3", "knee3", "foot3", "tail0", "tail1", "tail2",
];
pub const BODY: usize = 0;
pub const HIND: usize = 1;
pub const CHEST: usize = 2;
pub const NECK: usize = 3;
pub const JAW: usize = 4;
/// A leg's hip, knee and foot bones.
pub const fn hip(i: usize) -> usize {
    5 + 3 * i
}
pub const fn knee(i: usize) -> usize {
    6 + 3 * i
}
pub const fn foot(i: usize) -> usize {
    7 + 3 * i
}
pub const fn tail(i: usize) -> usize {
    17 + i
}

/// Which animal: the mesh it wears and the rig its joints are.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Animal {
    Beaver,
    RiverOtter,
    SeaOtter,
}

impl Animal {
    pub const ALL: [Animal; 3] = [Animal::Beaver, Animal::RiverOtter, Animal::SeaOtter];

    /// The fauna species it is drawn for.
    pub fn of_species(id: &str) -> Option<Self> {
        match id {
            "beaver" => Some(Animal::Beaver),
            "river_otter" => Some(Animal::RiverOtter),
            "sea_otter" => Some(Animal::SeaOtter),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Animal::Beaver => "beaver",
            Animal::RiverOtter => "river_otter",
            Animal::SeaOtter => "sea_otter",
        }
    }

    pub fn spec(self) -> &'static Spec {
        match self {
            Animal::Beaver => &BEAVER,
            Animal::RiverOtter => &RIVER_OTTER,
            Animal::SeaOtter => &SEA_OTTER,
        }
    }
}

/// One leg: its hip in its segment's frame, its two bones' lengths, and
/// whether it is a foreleg (on the chest, the elbow pointing back).
#[derive(Clone, Copy, Debug)]
pub struct LegSpec {
    pub at: Vec3,
    pub a: f32,
    pub b: f32,
    pub front: bool,
}

/// An animal's joints (critters.js `SPEC`).
#[derive(Clone, Copy, Debug)]
pub struct Spec {
    /// The body pivot's height over the ground standing.
    pub h: f32,
    /// How far the body pivot sits under the surface swimming.
    pub float: f32,
    /// A leg's length, for how deep the water is to swim in.
    pub leg_len: f32,
    pub neck: (f32, f32),
    pub jaw: Vec3,
    pub legs: [LegSpec; 4],
    pub tail_at: (f32, f32),
    pub tail_len: [f32; 3],
}

const fn leg(x: f32, y: f32, z: f32, a: f32, b: f32, front: bool) -> LegSpec {
    LegSpec {
        at: Vec3::new(x, y, z),
        a,
        b,
        front,
    }
}

pub static BEAVER: Spec = Spec {
    h: 0.17,
    float: 0.13,
    leg_len: 0.17,
    neck: (0.27, 0.035),
    jaw: Vec3::new(0.16, -0.045, 0.0),
    legs: [
        leg(0.16, -0.08, 0.085, 0.07, 0.07, true),
        leg(0.16, -0.08, -0.085, 0.07, 0.07, true),
        leg(-0.2, -0.07, 0.1, 0.09, 0.085, false),
        leg(-0.2, -0.07, -0.1, 0.09, 0.085, false),
    ],
    tail_at: (-0.31, -0.01),
    tail_len: [0.07, 0.06, 0.27],
};

pub static RIVER_OTTER: Spec = Spec {
    h: 0.14,
    float: 0.07,
    leg_len: 0.13,
    neck: (0.245, 0.02),
    jaw: Vec3::new(0.12, -0.03, 0.0),
    legs: [
        leg(0.17, -0.045, 0.055, 0.055, 0.05, true),
        leg(0.17, -0.045, -0.055, 0.055, 0.05, true),
        leg(-0.22, -0.04, 0.062, 0.065, 0.06, false),
        leg(-0.22, -0.04, -0.062, 0.065, 0.06, false),
    ],
    tail_at: (-0.3, 0.0),
    tail_len: [0.13, 0.12, 0.11],
};

pub static SEA_OTTER: Spec = Spec {
    h: 0.0,
    float: 0.0,
    leg_len: 0.12,
    neck: (0.29, 0.0),
    jaw: Vec3::new(0.14, -0.04, 0.0),
    legs: [
        leg(0.2, -0.06, 0.085, 0.065, 0.06, true),
        leg(0.2, -0.06, -0.085, 0.065, 0.06, true),
        leg(-0.3, -0.05, 0.08, 0.07, 0.06, false),
        leg(-0.3, -0.05, -0.08, 0.07, 0.06, false),
    ],
    tail_at: (-0.37, 0.0),
    tail_len: [0.09, 0.09, 0.09],
};

/// Each bone's parent (`None`: the root).
pub const PARENT: [Option<usize>; 20] = [
    None,
    Some(BODY),
    Some(BODY),
    Some(CHEST),
    Some(NECK),
    Some(CHEST),
    Some(hip(0)),
    Some(knee(0)),
    Some(CHEST),
    Some(hip(1)),
    Some(knee(1)),
    Some(HIND),
    Some(hip(2)),
    Some(knee(2)),
    Some(HIND),
    Some(hip(3)),
    Some(knee(3)),
    Some(HIND),
    Some(tail(0)),
    Some(tail(1)),
];

impl Spec {
    /// Each bone's place in its parent's frame at rest, the body standing.
    pub fn offsets(&self) -> [Vec3; 20] {
        let mut o = [Vec3::ZERO; 20];
        o[BODY] = Vec3::new(0.0, self.h, 0.0);
        o[NECK] = Vec3::new(self.neck.0, self.neck.1, 0.0);
        o[JAW] = self.jaw;
        for (i, l) in self.legs.iter().enumerate() {
            o[hip(i)] = l.at;
            o[knee(i)] = Vec3::new(0.0, -l.a, 0.0);
            o[foot(i)] = Vec3::new(0.0, -l.b, 0.0);
        }
        o[tail(0)] = Vec3::new(self.tail_at.0, self.tail_at.1, 0.0);
        o[tail(1)] = Vec3::new(-self.tail_len[0], 0.0, 0.0);
        o[tail(2)] = Vec3::new(-self.tail_len[1], 0.0, 0.0);
        o
    }

    /// Each bone's rest point in the root's frame: the bind pose the sculpt
    /// was skinned round (every bone unrotated).
    pub fn rest(&self) -> [Vec3; 20] {
        let o = self.offsets();
        let mut p = [Vec3::ZERO; 20];
        for k in 0..20 {
            p[k] = PARENT[k].map_or(Vec3::ZERO, |parent| p[parent]) + o[k];
        }
        p
    }
}

/// The forelimb's limits in the rig's terms, radians (version 5): the
/// elbow's flex (0 straight, + the forearm forward and up), the wrist's
/// angle (0 the paw at right angles to the forearm, -90 degrees in line with
/// it, more negative curled under), and the hands drawn in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Limits {
    pub e_max: f32,
    pub e_min: f32,
    pub w_lo: f32,
    pub w_hi: f32,
    pub add: f32,
}

impl Limits {
    pub fn of(f: &super::motion::Forelimb) -> Self {
        let deg = PI / 180.0;
        Self {
            e_max: f.elbow_max_deg * deg,
            e_min: -5.0 * deg,
            w_lo: -(90.0 + f.wrist_flex_deg) * deg,
            w_hi: (f.wrist_ext_deg - 90.0) * deg,
            add: f.hands_in_deg * deg,
        }
    }
}

/// Where a leg is sent: a point in the root's frame (the ground at y = 0
/// under the root) or from the hip in its segment's frame; a paw's curl,
/// and a wrist set straight or blended in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Target {
    pub root: bool,
    pub x: f32,
    pub y: f32,
    /// The foot's angle in the segment's frame (`seg` only).
    pub f: Option<f32>,
    pub c: f32,
    /// The wrist, forearm-relative, blended in by `wk`.
    pub w: Option<f32>,
    pub wk: f32,
}

impl Target {
    pub fn root(x: f32, y: f32) -> Self {
        Self {
            root: true,
            x,
            y,
            f: None,
            c: 0.0,
            w: None,
            wk: 1.0,
        }
    }

    pub fn seg(x: f32, y: f32, f: f32) -> Self {
        Self {
            root: false,
            x,
            y,
            f: Some(f),
            c: 0.0,
            w: None,
            wk: 1.0,
        }
    }

    pub fn curl(mut self, c: f32) -> Self {
        self.c = c;
        self
    }
}

/// A pose: plain numbers, blended between actions so a change never pops.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pose {
    /// The body pivot's height over the root, its pitch and roll.
    pub y: f32,
    pub pitch: f32,
    pub roll: f32,
    /// The spine's bend: the hind segment turns by +a, the chest by -a.
    pub a: f32,
    /// The head's pitch, yaw and roll.
    pub hp: f32,
    pub hy: f32,
    pub hr: f32,
    /// Each leg's hip, knee and foot angle, and the hands drawn in.
    pub la: [f32; 4],
    pub lk: [f32; 4],
    pub lf: [f32; 4],
    pub lx: [f32; 4],
    /// Each leg's tilt across the body that keeps a foot sent to the ground
    /// under itself while the body rolls (B29); nought otherwise.
    pub lz: [f32; 4],
    pub lt: [Option<Target>; 4],
    /// The tail's yaw and pitch, part by part.
    pub ty: [f32; 3],
    pub tp: [f32; 3],
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// The page's clamp: the lower bound wins when the two cross.
pub fn clampf(x: f32, lo: f32, hi: f32) -> f32 {
    lo.max(hi.min(x))
}

pub fn c01(x: f32) -> f32 {
    clampf(x, 0.0, 1.0)
}

pub fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

pub fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t) * (1.0 - t)
}

pub fn frac(x: f32) -> f32 {
    x - x.floor()
}

pub fn ang_wrap(mut a: f32) -> f32 {
    if !a.is_finite() {
        return 0.0;
    }
    while a > PI {
        a -= 2.0 * PI;
    }
    while a < -PI {
        a += 2.0 * PI;
    }
    a
}

impl Pose {
    /// Every number but the targets blended `t` of the way from `a` to `b`
    /// (`lerpPose`): the targets are solved into angles before a blend.
    pub fn lerp_from(&mut self, a: &Pose, b: &Pose, t: f32) {
        self.y = lerp(a.y, b.y, t);
        self.pitch = lerp(a.pitch, b.pitch, t);
        self.roll = lerp(a.roll, b.roll, t);
        self.a = lerp(a.a, b.a, t);
        self.hp = lerp(a.hp, b.hp, t);
        self.hy = lerp(a.hy, b.hy, t);
        self.hr = lerp(a.hr, b.hr, t);
        for i in 0..4 {
            self.la[i] = lerp(a.la[i], b.la[i], t);
            self.lk[i] = lerp(a.lk[i], b.lk[i], t);
            self.lf[i] = lerp(a.lf[i], b.lf[i], t);
            self.lx[i] = lerp(a.lx[i], b.lx[i], t);
            self.lz[i] = lerp(a.lz[i], b.lz[i], t);
        }
        for i in 0..3 {
            self.ty[i] = lerp(a.ty[i], b.ty[i], t);
            self.tp[i] = lerp(a.tp[i], b.tp[i], t);
        }
    }

    pub fn blended(a: &Pose, b: &Pose, t: f32) -> Pose {
        let mut p = *a;
        p.lerp_from(a, b, t);
        p
    }
}

/// Solve every leg sent to a target.
pub fn solve_legs(spec: &Spec, lim: &Limits, p: &mut Pose) {
    for i in 0..4 {
        solve_leg(spec, lim, p, i);
    }
}

/// Two bones to a target (critters.js `solveLeg`). A foreleg's target is
/// held out on its line from the shoulder so the elbow cannot pass its
/// limit, and its elbow and wrist are clamped to the limits.
pub fn solve_leg(spec: &Spec, lim: &Limits, p: &mut Pose, i: usize) {
    let Some(t) = p.lt[i] else {
        return;
    };
    let l = &spec.legs[i];
    let seg_p = if l.front { -p.a } else { p.a };
    let lim = l.front.then_some(lim);
    let (mut hx, mut hy, mut par) = (0.0, 0.0, 0.0);
    let mut lateral = 0.0;
    if t.root {
        // The hip where the body's pitch, roll and spine put it (the page
        // left the roll out, so a waddling foot swayed in its stance; B29).
        let h = Quat::from_rotation_z(p.pitch)
            * Quat::from_rotation_x(p.roll)
            * Quat::from_rotation_z(seg_p)
            * l.at;
        hx = h.x;
        hy = h.y + p.y;
        par = p.pitch + seg_p;
        lateral = h.z - l.at.z;
    }
    let (a, b) = (l.a, l.b);
    let dmin = lim.map_or(0.0, |m| {
        (a * a + b * b + 2.0 * a * b * m.e_max.cos()).sqrt()
    });
    let (dx, mut dy) = (t.x - hx, t.y - hy);
    // A rolled hip stands off to the side of where its foot is planted: the
    // leg is tilted across by the angle that puts the foot back under it,
    // and solved in its tilted plane.
    p.lz[i] = 0.0;
    if t.root && lateral.abs() > 1e-6 {
        let down = dy.hypot(lateral).max(1e-4);
        let tilt = clampf(lateral / down, -0.5, 0.5).asin();
        p.lz[i] = tilt;
        dy = -dy.abs().hypot(lateral);
    }
    let d = clampf(
        dx.hypot(dy),
        ((a - b).abs() + 1e-3).max(dmin + 1e-4),
        (a + b) * 0.999,
    );
    let base = dx.atan2(-dy);
    let c_a = clampf((a * a + d * d - b * b) / (2.0 * a * d), -1.0, 1.0).acos();
    let g = clampf((a * a + b * b - d * d) / (2.0 * a * b), -1.0, 1.0).acos();
    let s = if l.front { -1.0 } else { 1.0 };
    let aw = base + s * c_a;
    let k = -s * (PI - g);
    p.la[i] = aw - par;
    p.lk[i] = k;
    p.lf[i] = (if t.root {
        -(aw + k)
    } else {
        t.f.unwrap_or(-(aw + k) * 0.6)
    }) + t.c;
    if let Some(w) = t.w {
        p.lf[i] = lerp(p.lf[i], w + t.c, c01(t.wk));
    }
    if let Some(m) = lim {
        p.lk[i] = clampf(p.lk[i], m.e_min, m.e_max);
        p.lf[i] = clampf(p.lf[i], m.w_lo, m.w_hi);
    }
}

/// The forelimb's limits held last, after every blend, spring and lag
/// (critters.js `applyPose`).
pub fn clamp_forelimbs(lim: &Limits, p: &mut Pose) {
    for i in 0..2 {
        p.lk[i] = clampf(p.lk[i], lim.e_min, lim.e_max);
        p.lf[i] = clampf(p.lf[i], lim.w_lo, lim.w_hi);
        p.lx[i] = clampf(p.lx[i], 0.0, lim.add);
    }
}

/// Each bone's rotation in its parent's frame for a pose, as the page's
/// Euler orders give it.
pub fn rotations(p: &Pose) -> [Quat; 20] {
    let rx = Quat::from_rotation_x;
    let ry = Quat::from_rotation_y;
    let rz = Quat::from_rotation_z;
    let mut q = [Quat::IDENTITY; 20];
    // body: order ZYX, (roll, 0, pitch)
    q[BODY] = rz(p.pitch) * rx(p.roll);
    q[HIND] = rz(p.a);
    q[CHEST] = rz(-p.a);
    // neck: order YZX, (hr, hy, hp)
    q[NECK] = ry(p.hy) * rz(p.hp) * rx(p.hr);
    // A small roll (the waddle, the shake) is taken back at the hips, so the
    // legs stay upright under a rolling body and a planted foot stays put
    // (B29); a large one (on its back) is the body's alone.
    let keep = 1.0 - smooth(c01((p.roll.abs() - 0.5) / 0.3));
    for i in 0..4 {
        let x = match i {
            0 => p.lx[0],
            1 => -p.lx[1],
            _ => 0.0,
        } + (p.lz[i] - p.roll) * keep;
        q[hip(i)] = rx(x) * rz(p.la[i]);
        q[knee(i)] = rz(p.lk[i]);
        q[foot(i)] = rz(p.lf[i]);
    }
    for i in 0..3 {
        // tail: order YZX, (0, ty, -tp)
        q[tail(i)] = ry(p.ty[i]) * rz(-p.tp[i]);
    }
    q
}

/// Each bone's local translation: its rest offset, the body's height the
/// pose's.
pub fn translations(spec: &Spec, p: &Pose) -> [Vec3; 20] {
    let mut o = spec.offsets();
    o[BODY] = Vec3::new(0.0, p.y, 0.0);
    o
}

/// Every bone's transform in the root's frame for a pose (forward
/// kinematics): for the props a bone holds, and for the tests.
pub fn fk(spec: &Spec, p: &Pose) -> [Affine3A; 20] {
    let q = rotations(p);
    let t = translations(spec, p);
    let mut out = [Affine3A::IDENTITY; 20];
    for k in 0..20 {
        let local = Affine3A::from_rotation_translation(q[k], t[k]);
        out[k] = PARENT[k].map_or(local, |parent| out[parent] * local);
    }
    out
}
