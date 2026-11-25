//! An animal's motion: its steps, its poses and what it does next
//! (`ocean-environments` design B28, B29), ported from M25's
//! `docs/mockups/ocean/critters.js` (`Actor`, `Move`, `Act`, `Hold`, `Face`,
//! the shared poses and each animal's plans).
//!
//! Everything here is drawn only. The actor lives in a small flat frame
//! round its creature (metres: x and z along the ground, y up); the game
//! hands it where its creature is ([`Actor::anchor`]) and whether it is
//! startled, and reads back a pose. Nothing reads the actor.

#![allow(clippy::needless_range_loop)]

use super::motion::CreatureMotion;
use super::rig::{
    Animal, Limits, Pose, Spec, Target, c01, clamp_forelimbs, clampf, ease_out, frac, lerp, smooth,
    solve_leg, solve_legs,
};
use bevy::math::Vec3;
use std::collections::VecDeque;
use std::f32::consts::PI;

const DEG: f32 = PI / 180.0;
/// The beaver's tail standing: down from the hips, the paddle flat.
const TAIL_DRAG: [f32; 3] = [-0.95, 0.6, 0.32];
/// How long a change of step blends, s.
const BLEND_S: f32 = 0.28;

/// The ground and the water round an actor, in its frame.
pub trait Ground {
    /// The ground's height at (x, z).
    fn ground(&self, x: f32, z: f32) -> f32;
    /// The water's surface at (x, z), where there is water.
    fn level(&self, x: f32, z: f32) -> Option<f32>;
}

/// The page's seeded draw (`mulberry32`).
#[derive(Clone, Copy, Debug)]
pub struct Mulberry(u32);

impl Mulberry {
    pub fn new(seed: u32) -> Self {
        Self(seed)
    }

    pub fn draw(&mut self) -> f32 {
        self.0 = self.0.wrapping_add(0x6D2B_79F5);
        let seed = self.0;
        let mut t = (seed ^ (seed >> 15)).wrapping_mul(1 | seed);
        t = (t.wrapping_add((t ^ (t >> 7)).wrapping_mul(61 | t))) ^ t;
        ((t ^ (t >> 14)) as f64 / 4_294_967_296.0) as f32
    }

    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.draw()
    }
}

/// A strike's stage (the animation skill).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Windup,
    Anticipation,
    Hit,
    Recoil,
    Rest,
}

/// One strike in four stages: windup to `top`, anticipation easing to
/// `coil` and held, the hit to `contact` (t cubed: fastest at contact), the
/// recoil kicking past rest by `kick` and easing back to nought. `d` is
/// [windup, anticipation, hit, recoil], s.
pub fn strike(
    mut t: f32,
    d: [f32; 4],
    top: f32,
    coil: f32,
    contact: f32,
    kick: f32,
) -> (f32, Stage) {
    let [w, a, h, r] = d;
    if t < w {
        return (top * ease_out(t / w), Stage::Windup);
    }
    t -= w;
    if t < a {
        return (lerp(top, coil, smooth(t / a)), Stage::Anticipation);
    }
    t -= a;
    if t < h {
        let s = t / h;
        return (lerp(coil, contact, s * s * s), Stage::Hit);
    }
    t -= h;
    let s = c01(t / r.max(1e-4));
    let v = lerp(contact, 0.0, ease_out(s)) + kick * (PI * (s * 1.7).min(1.0)).sin() * (1.0 - s);
    (v, if s < 1.0 { Stage::Recoil } else { Stage::Rest })
}

/// A rear-up and the sit back down on one clock (critters.js `rear`).
#[derive(Clone, Copy, Debug, Default)]
pub struct Rear {
    pub crouch: f32,
    pub body: f32,
    pub arm: f32,
    pub give: f32,
    pub curl: f32,
    pub d: f32,
}

/// What a beaver or an otter holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Held {
    Nothing,
    /// A stick in the beaver's teeth.
    Stick,
}

/// Where a move goes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Goal {
    Point(Vec3),
    /// Back to the creature's own place in the water.
    Anchor,
}

/// A step: what it does, and its clock.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    /// Stay with the creature's place in the water for a while, swimming
    /// (or rafting, for the sea otter), `depth` under the surface.
    Follow {
        dur: f32,
        depth: f32,
    },
    Move {
        goal: Goal,
        speed: f32,
        tol: f32,
    },
    Hold {
        dur: f32,
    },
    Face {
        to: Vec3,
    },
    Hop {
        from: Vec3,
        to: Vec3,
    },
    GnawLow {
        bites: u32,
        last: i32,
    },
    TakeStick,
    DropStick,
    BeaverSit,
    Shake,
    Slap,
    Porpoise {
        a0: f32,
        d: f32,
        ph: f32,
        dy: f32,
    },
    OtterGroom,
    Look,
    Crack {
        last: i32,
    },
    EatUrchin,
    UrchinDone,
    SeaDive {
        went: bool,
        up: bool,
    },
    SeaGroom,
    Sleep,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Step {
    pub kind: Kind,
    pub t: f32,
    /// The water under the animal the last time it moved, 0 to 1.
    sw: f32,
}

impl Step {
    fn new(kind: Kind) -> Self {
        Self {
            kind,
            t: 0.0,
            sw: 0.0,
        }
    }

    /// Its id, for tests and logs (the mockup's HUD names); never shown in
    /// the game.
    pub fn name(&self) -> &'static str {
        match self.kind {
            Kind::Follow { depth, .. } if depth > 0.0 => "under_water",
            Kind::Follow { .. } => "swim",
            Kind::Move { .. } => "move",
            Kind::Hold { .. } => "rest",
            Kind::Face { .. } => "turn",
            Kind::Hop { .. } => "hop",
            Kind::GnawLow { .. } => "gnaw_a_stick",
            Kind::TakeStick => "take_the_stick",
            Kind::DropStick => "let_the_stick_go",
            Kind::BeaverSit => "sit_up_and_groom",
            Kind::Shake => "shake_off",
            Kind::Slap => "tail_slap",
            Kind::Porpoise { .. } => "porpoise",
            Kind::OtterGroom => "roll_and_groom",
            Kind::Look => "look_around",
            Kind::Crack { .. } => "crack_an_urchin",
            Kind::EatUrchin => "eat_the_urchin",
            Kind::UrchinDone => "done",
            Kind::SeaDive { .. } => "roll_over_and_dive",
            Kind::SeaGroom => "groom",
            Kind::Sleep => "sleep",
        }
    }
}

/// The overlapping action: the head and each tail part a beat behind.
#[derive(Clone, Copy, Debug, Default)]
struct Lag {
    hr: f32,
    ty: [f32; 3],
    tp: [f32; 3],
    yaw_rate: f32,
    last_yaw: f32,
}

/// One drawn animal.
#[derive(Clone, Debug)]
pub struct Actor {
    pub animal: Animal,
    /// Where it is drawn, in its frame: the root on the ground (or the
    /// water's surface), turned to `yaw` about up.
    pub pos: Vec3,
    pub yaw: f32,
    pub v: f32,
    pub vy: f32,
    /// The gait's phase (on the distance moved) and the swim's.
    pub phi: f32,
    pub swim_ph: f32,
    pub step: Option<Step>,
    pub queue: VecDeque<Step>,
    rng: Mulberry,
    pub wet: f32,
    snap: Pose,
    pub cur: Pose,
    bt: f32,
    lag: Lag,
    /// The pose as drawn this frame: after the blend, the lag and the
    /// forelimb's limits.
    pub out: Pose,
    pub held: Held,
    /// The sea otter's urchin: whole (1) to eaten, or none.
    pub urchin: Option<f32>,
    /// A stick lying on the ground for the beaver to gnaw.
    pub stick_ground: Option<Vec3>,
    /// Where its creature is now, in the frame: the game's, every frame.
    pub anchor: Vec3,
    /// Its creature is startled this frame.
    pub spooked: bool,
    was_spooked: bool,
    /// The bites, slaps and cracks landed so far (each on its beat).
    pub hits: u32,
    /// The actor's own clock, s.
    pub clock: f32,
    pub stage: &'static str,
    /// The forelimb's limits this frame.
    lim: Limits,
}

impl Actor {
    pub fn new(animal: Animal, pos: Vec3, yaw: f32, seed: u32) -> Self {
        let mut a = Self {
            animal,
            pos,
            yaw,
            v: 0.0,
            vy: 0.0,
            phi: 0.0,
            swim_ph: 0.0,
            step: None,
            queue: VecDeque::with_capacity(16),
            rng: Mulberry::new(seed.wrapping_mul(977).wrapping_add(1)),
            wet: 1.0,
            snap: Pose::default(),
            cur: Pose::default(),
            bt: 1.0,
            lag: Lag {
                last_yaw: yaw,
                ..Lag::default()
            },
            out: Pose::default(),
            held: Held::Nothing,
            urchin: None,
            stick_ground: None,
            anchor: pos,
            spooked: false,
            was_spooked: false,
            hits: 0,
            clock: 0.0,
            stage: "",
            lim: Limits::of(&super::motion::Forelimb::default()),
        };
        a.cur.y = a.spec().h;
        a.snap = a.cur;
        a
    }

    pub fn spec(&self) -> &'static Spec {
        self.animal.spec()
    }

    fn beaver(&self) -> bool {
        self.animal == Animal::Beaver
    }

    fn otter(&self) -> bool {
        self.animal == Animal::RiverOtter
    }

    /// Everything a frame moves by `d`: the frame recentred under it.
    pub fn shift(&mut self, d: Vec3) {
        self.pos += d;
        self.anchor += d;
        if let Some(s) = self.stick_ground.as_mut() {
            *s += d;
        }
        let shift_step = |s: &mut Step| match &mut s.kind {
            Kind::Move {
                goal: Goal::Point(p),
                ..
            } => *p += d,
            Kind::Face { to } => *to += d,
            Kind::Hop { from, to } => {
                *from += d;
                *to += d;
            }
            _ => {}
        };
        if let Some(s) = self.step.as_mut() {
            shift_step(s);
        }
        self.queue.iter_mut().for_each(shift_step);
    }

    fn fwd(&self) -> Vec3 {
        Vec3::new(self.yaw.cos(), 0.0, -self.yaw.sin())
    }

    /// The ground at a point of the root's frame (x forward, z left),
    /// over the root.
    fn ground_rel(&self, g: &impl Ground, lx: f32, lz: f32) -> f32 {
        let (s, c) = self.yaw.sin_cos();
        g.ground(self.pos.x + lx * c + lz * s, self.pos.z - lx * s + lz * c) - self.pos.y
    }

    fn level(g: &impl Ground, x: f32, z: f32) -> f32 {
        g.level(x, z).unwrap_or(-1000.0)
    }

    /// How deep in the water it is: 0 standing to 1 swimming.
    pub fn water_at(&self, g: &impl Ground) -> f32 {
        let gr = g.ground(self.pos.x, self.pos.z);
        let l = Self::level(g, self.pos.x, self.pos.z);
        c01((l - gr - self.spec().leg_len * 0.55) / 0.22)
    }

    fn limits(&self, m: &CreatureMotion) -> Limits {
        Limits::of(match self.animal {
            Animal::Beaver => &m.beaver.forelimb,
            Animal::RiverOtter => &m.river_otter.forelimb,
            Animal::SeaOtter => &m.sea_otter.forelimb,
        })
    }

    /// Start a step at once, in place of whatever it was doing (a button
    /// on the page; the tests).
    pub fn start(&mut self, kind: Kind) {
        self.queue.clear();
        self.queue.push_back(Step::new(kind));
        self.step = None;
    }

    fn push(&mut self, kind: Kind) {
        self.queue.push_back(Step::new(kind));
    }

    fn set_step(&mut self, s: Option<Step>) {
        self.step = s;
        self.snap = self.cur;
        self.bt = 0.0;
    }

    /// A frame: the step moves the animal, its pose is solved and blended,
    /// the lag added and the limits held.
    pub fn update(&mut self, g: &impl Ground, m: &CreatureMotion, dt: f32) {
        let dt = dt.clamp(0.0, 0.1);
        self.clock += dt;
        // Startled: whatever it was doing gives way at once (a beaver slaps
        // its tail and dives, a sea otter dives, a river otter swims off).
        let startled = self.spooked && !self.was_spooked;
        self.was_spooked = self.spooked;
        let diving = matches!(
            self.step.map(|s| s.kind),
            Some(Kind::Slap | Kind::SeaDive { .. })
        ) || matches!(
            self.step.map(|s| s.kind),
            Some(Kind::Follow { depth, .. }) if depth > 0.0
        );
        if startled && !diving {
            self.queue.clear();
            self.stick_ground = None;
            self.next(g, m);
        }
        if self.step.is_none() {
            self.next(g, m);
        }
        let Some(mut s) = self.step else {
            return;
        };
        s.t += dt;
        let lim = self.limits(m);
        self.lim = lim;
        let done = self.advance(&mut s, g, m, dt);
        let mut p = Pose::default();
        self.pose(&mut s, g, m, &lim, &mut p);
        solve_legs(self.spec(), &lim, &mut p);
        self.stage = s.name();
        self.step = Some(s);
        self.bt = (self.bt + dt / BLEND_S).min(1.0);
        let mut cur = self.cur;
        cur.lerp_from(&self.snap, &p, smooth(self.bt));
        cur.lt = [None; 4];
        self.cur = cur;
        self.out = self.lagged(m, dt);
        clamp_forelimbs(&lim, &mut self.out);
        // A wet coat: wet while in the water, drying on land.
        let lv = Self::level(g, self.pos.x, self.pos.z);
        if lv > self.pos.y + self.cur.y - 0.05 {
            self.wet = 1.0;
        } else {
            let dry = if self.beaver() {
                m.beaver.shake.dry_s
            } else {
                14.0
            };
            self.wet = (self.wet - dt / dry).max(0.0);
        }
        if done {
            self.next(g, m);
        }
    }

    /// The head counters the body's roll a beat late; each tail part
    /// follows the hips' roll and the turn a beat behind the one before.
    fn lagged(&mut self, m: &CreatureMotion, dt: f32) -> Pose {
        let mut out = self.cur;
        let (hl, tl) = if self.beaver() {
            (m.beaver.walk.head_lag_s, m.beaver.walk.tail_lag_s)
        } else {
            (0.08, 0.12)
        };
        let l = &mut self.lag;
        let yr = super::rig::ang_wrap(self.yaw - l.last_yaw) / dt.max(1e-3);
        l.last_yaw = self.yaw;
        l.yaw_rate += (yr - l.yaw_rate) * c01(dt * 6.0);
        let roll = self.cur.roll;
        let small = roll.abs() < 1.2;
        let k = c01(dt / hl);
        l.hr += (-roll * 0.6 * if small { 1.0 } else { 0.0 } - l.hr) * k;
        out.hr += l.hr;
        let mut drive = roll * if small { 1.1 } else { 0.0 } - l.yaw_rate * 0.22;
        let mut drive_p = -self.vy * 0.4;
        for i in 0..3 {
            let kk = c01(dt / tl);
            l.ty[i] += (drive - l.ty[i]) * kk;
            l.tp[i] += (drive_p - l.tp[i]) * kk;
            out.ty[i] += l.ty[i];
            out.tp[i] += l.tp[i];
            drive = l.ty[i];
            drive_p = l.tp[i];
        }
        out
    }

    /// The next step: the queue's, or a new plan.
    fn next(&mut self, g: &impl Ground, m: &CreatureMotion) {
        if self.queue.is_empty() {
            self.plan(g, m);
        }
        let s = self.queue.pop_front().unwrap_or(Step::new(Kind::Follow {
            dur: 1.0,
            depth: 0.0,
        }));
        if let Kind::GnawLow { .. } = s.kind {
            // the stick lies on the ground under its forepaws
            let ahead = self.pos + self.fwd() * (self.spec().legs[0].at.x + 0.12);
            self.stick_ground = Some(Vec3::new(ahead.x, g.ground(ahead.x, ahead.z), ahead.z));
        }
        if let Kind::TakeStick = s.kind {
            self.stick_ground = None;
            self.held = Held::Stick;
        }
        if let Kind::DropStick = s.kind {
            self.held = Held::Nothing;
        }
        if let Kind::UrchinDone = s.kind {
            self.urchin = None;
        }
        self.set_step(Some(s));
    }

    // ---------------------------------------------------------------- plans

    /// What it does next: a seeded draw from what is open to it where it is
    /// (critters.js `brain`), the game's version of each plan.
    fn plan(&mut self, g: &impl Ground, m: &CreatureMotion) {
        let startled = self.spooked;
        let in_water = self.water_at(g) > 0.5;
        if !in_water && self.animal != Animal::SeaOtter {
            self.back_to_water(g);
            return;
        }
        match self.animal {
            Animal::Beaver => self.plan_beaver(g, startled),
            Animal::RiverOtter => self.plan_otter(g, startled),
            Animal::SeaOtter => self.plan_sea(m, startled),
        }
    }

    fn follow(&mut self, lo: f32, hi: f32) {
        let dur = self.rng.range(lo, hi);
        self.push(Kind::Follow { dur, depth: 0.0 });
    }

    fn plan_beaver(&mut self, g: &impl Ground, startled: bool) {
        if startled {
            self.push(Kind::Slap);
            self.push(Kind::Follow {
                dur: 2.5,
                depth: 0.75,
            });
            self.follow(2.0, 4.0);
            return;
        }
        if self.held == Held::Stick {
            // hauls the stick through the water, then lets it go
            self.follow(6.0, 10.0);
            self.push(Kind::DropStick);
            return;
        }
        let r = self.rng.draw();
        if r < 0.4 {
            self.follow(5.0, 10.0);
        } else if r < 0.58 {
            self.push(Kind::Slap);
            self.push(Kind::Follow {
                dur: 2.5,
                depth: 0.75,
            });
            self.follow(2.0, 4.0);
        } else {
            let gnaw = self.rng.draw() < 0.45;
            if !self.ashore(g, if gnaw { "gnaw" } else { "sit" }) {
                self.follow(4.0, 8.0);
            }
        }
    }

    /// Out onto a bank point a few metres off and back to its creature:
    /// what it does there is `what`. False where no bank is in reach.
    fn ashore(&mut self, g: &impl Ground, what: &str) -> bool {
        let Some((bank, edge)) = self.bank_near(g) else {
            return false;
        };
        self.push(Kind::Move {
            goal: Goal::Point(edge),
            speed: 1.0,
            tol: 0.15,
        });
        self.push(Kind::Hop {
            from: Vec3::ZERO,
            to: bank,
        });
        match what {
            "gnaw" => {
                self.push(Kind::Shake);
                self.push(Kind::GnawLow { bites: 5, last: -1 });
                self.push(Kind::TakeStick);
            }
            "sit" => {
                self.push(Kind::Shake);
                self.push(Kind::BeaverSit);
            }
            "groom" => self.push(Kind::OtterGroom),
            _ => self.push(Kind::Look),
        }
        self.push(Kind::Hop {
            from: Vec3::ZERO,
            to: edge,
        });
        self.push(Kind::Move {
            goal: Goal::Anchor,
            speed: 1.0,
            tol: 0.12,
        });
        true
    }

    /// Begin with `plan` in place of the first draw (`PBD_CRITTER_PLAN`, a
    /// capture instrument): `bank` (ashore: the beaver gnaws a stick, the
    /// river otter looks round), `sit`, `slap`, `haul`, `porpoise`, `groom`,
    /// `look`, `crack`, `dive`, `sleep` or `swim`, as the animal can.
    pub fn force(&mut self, g: &impl Ground, plan: &str) {
        self.queue.clear();
        self.step = None;
        let ok = match (self.animal, plan) {
            (Animal::Beaver, "bank") => self.ashore(g, "gnaw"),
            (Animal::Beaver, "sit") => self.ashore(g, "sit"),
            (Animal::Beaver, "slap") => {
                self.plan_beaver(g, true);
                true
            }
            (Animal::Beaver, "haul") => {
                self.held = Held::Stick;
                self.follow(20.0, 30.0);
                true
            }
            (Animal::RiverOtter, "bank") => self.ashore(g, "look"),
            (Animal::RiverOtter, "porpoise") => {
                self.push(Kind::Porpoise {
                    a0: 0.0,
                    d: 0.0,
                    ph: 0.0,
                    dy: 0.0,
                });
                true
            }
            (Animal::RiverOtter, "groom") => {
                self.push(Kind::OtterGroom);
                true
            }
            (Animal::RiverOtter, "look") => {
                self.push(Kind::Look);
                true
            }
            (Animal::SeaOtter, "crack") => {
                self.urchin = Some(1.0);
                true
            }
            (Animal::SeaOtter, "dive") => {
                self.push(Kind::SeaDive {
                    went: false,
                    up: false,
                });
                true
            }
            (Animal::SeaOtter, "groom") => {
                self.push(Kind::SeaGroom);
                true
            }
            (Animal::SeaOtter, "sleep") => {
                self.push(Kind::Sleep);
                true
            }
            _ => false,
        };
        if !ok || plan == "swim" {
            self.follow(20.0, 30.0);
        }
    }

    fn plan_otter(&mut self, g: &impl Ground, startled: bool) {
        if startled {
            // it swims off with its creature
            self.follow(3.0, 5.0);
            return;
        }
        let r = self.rng.draw();
        if r < 0.35 {
            self.follow(5.0, 10.0);
        } else if r < 0.55 {
            self.push(Kind::Porpoise {
                a0: 0.0,
                d: 0.0,
                ph: 0.0,
                dy: 0.0,
            });
        } else if r < 0.7 {
            self.push(Kind::OtterGroom);
        } else if r < 0.8 {
            self.push(Kind::Look);
        } else {
            let what = if self.rng.draw() < 0.5 {
                "look"
            } else {
                "groom"
            };
            if !self.ashore(g, what) {
                self.follow(4.0, 8.0);
            }
        }
    }

    fn plan_sea(&mut self, m: &CreatureMotion, startled: bool) {
        let _ = m;
        if self.urchin.is_some() {
            // an urchin brought up is cracked and eaten before anything else
            self.push(Kind::Crack { last: -1 });
            self.push(Kind::EatUrchin);
            self.push(Kind::UrchinDone);
            return;
        }
        if startled {
            self.push(Kind::SeaDive {
                went: false,
                up: false,
            });
            return;
        }
        let r = self.rng.draw();
        if r < 0.35 {
            self.follow(6.0, 12.0);
        } else if r < 0.6 {
            self.push(Kind::SeaDive {
                went: false,
                up: false,
            });
        } else if r < 0.8 {
            self.push(Kind::SeaGroom);
        } else {
            self.push(Kind::Sleep);
        }
    }

    /// Out of the water and back: a hop to the nearest water, then home.
    fn back_to_water(&mut self, g: &impl Ground) {
        if let Some(w) = self.water_near(g) {
            self.push(Kind::Hop {
                from: Vec3::ZERO,
                to: w,
            });
        }
        self.push(Kind::Move {
            goal: Goal::Anchor,
            speed: 1.2,
            tol: 0.12,
        });
    }

    fn water_near(&self, g: &impl Ground) -> Option<Vec3> {
        for r in [0.6f32, 1.2, 1.8, 2.6, 3.4] {
            for k in 0..8 {
                let a = k as f32 * PI / 4.0;
                let (x, z) = (self.pos.x + r * a.cos(), self.pos.z + r * a.sin());
                if let Some(l) = g.level(x, z)
                    && l - g.ground(x, z) > self.spec().leg_len
                {
                    return Some(Vec3::new(x, l, z));
                }
            }
        }
        None
    }

    /// A dry bank point a few metres off, no more than a step over the
    /// water, and the last water before it on the same bearing.
    fn bank_near(&mut self, g: &impl Ground) -> Option<(Vec3, Vec3)> {
        let here = Self::level(g, self.pos.x, self.pos.z);
        if here < -100.0 {
            return None;
        }
        let turn = self.rng.draw() * 2.0 * PI;
        for k in 0..8 {
            let a = turn + k as f32 * PI / 4.0;
            let (dx, dz) = (a.cos(), a.sin());
            let mut edge: Option<Vec3> = None;
            let mut r = 0.4;
            while r <= 4.0 {
                let (x, z) = (self.pos.x + dx * r, self.pos.z + dz * r);
                let gr = g.ground(x, z);
                let wet = g.level(x, z).is_some_and(|l| l - gr > self.spec().leg_len);
                if wet {
                    edge = Some(Vec3::new(x, here, z));
                } else {
                    let up = gr - here;
                    if (-0.15..=1.15).contains(&up) && edge.is_some() {
                        // a little way in, on flat ground
                        let (x2, z2) = (x + dx * 0.4, z + dz * 0.4);
                        if (g.ground(x2, z2) - gr).abs() < 0.05 {
                            return Some((Vec3::new(x2, g.ground(x2, z2), z2), edge?));
                        }
                    }
                    break;
                }
                r += 0.3;
            }
        }
        None
    }

    // ---------------------------------------------------------------- moving

    /// The step moves the animal; true when it is done.
    fn advance(&mut self, s: &mut Step, g: &impl Ground, m: &CreatureMotion, dt: f32) -> bool {
        let t = s.t;
        match &mut s.kind {
            Kind::Follow { dur, depth } => {
                let (dur, depth) = (*dur, *depth);
                s.sw = self.follow_anchor(g, m, dt, depth);
                t >= dur
            }
            Kind::Move { goal, speed, tol } => {
                let to = match *goal {
                    Goal::Point(p) => p,
                    Goal::Anchor => self.anchor,
                };
                let (speed, tol) = (*speed, *tol);
                let (sw, dist) = self.move_toward(g, m, dt, to, speed, 0.0);
                s.sw = sw;
                dist < tol || t > 20.0
            }
            Kind::Hold { dur } => t >= *dur,
            Kind::Face { to } => {
                let want = (-(to.z - self.pos.z)).atan2(to.x - self.pos.x);
                let err = super::rig::ang_wrap(want - self.yaw);
                let tr = 2.5 * dt;
                let turn = clampf(err, -tr, tr);
                self.yaw += turn;
                self.phi += turn.abs() * 0.12 / 0.36;
                err.abs() < 0.04
            }
            Kind::Hop { from, to } => {
                if t - dt <= 0.0 {
                    *from = self.pos;
                    let gr = g.ground(to.x, to.z);
                    to.y = g.level(to.x, to.z).map_or(gr, |l| l.max(gr));
                    self.yaw = (-(to.z - self.pos.z)).atan2(to.x - self.pos.x);
                }
                let k = c01((t - 0.3) / 0.45);
                if t >= 0.3 {
                    let before = self.pos;
                    self.pos = from.lerp(*to, k);
                    self.pos.y += 0.38 * (PI * k).sin();
                    self.v = (self.pos - before).length() / dt.max(1e-3);
                }
                t >= 0.95
            }
            Kind::GnawLow { bites, last } => {
                let d = gnaw_d(m);
                let beat: f32 = d.iter().sum();
                let k = (t / beat).floor();
                let into = t - k * beat;
                if into >= d[0] + d[1] + d[2] && (*last as f32) < k && (k as u32) < *bites {
                    *last = k as i32;
                    self.hits += 1;
                }
                t >= beat * *bites as f32
            }
            Kind::TakeStick | Kind::DropStick => t >= 0.45,
            Kind::BeaverSit => t >= 3.6,
            Kind::Shake => {
                let sh = &m.beaver.shake;
                if t > 0.2 && t < 0.2 + sh.time_s {
                    self.wet = self.wet.min(0.45);
                }
                t >= 0.2 + sh.time_s
            }
            Kind::Slap => {
                self.follow_anchor(g, m, dt, 0.0);
                let d = slap_d(m);
                let hit = d[0] + d[1] + d[2];
                if t >= hit && t - dt < hit {
                    self.hits += 1;
                }
                t >= d.iter().sum::<f32>() + 0.5
            }
            Kind::Porpoise { a0, d, ph, dy } => {
                let c = self.anchor;
                if t - dt <= 0.0 {
                    *a0 = ((self.pos.z - c.z) / 0.75).atan2(self.pos.x - c.x);
                }
                const DUR: f32 = 8.0;
                let r = 1.2 * smooth(c01(t)) * smooth(c01(DUR - t));
                let sp = 1.4;
                *d += sp * dt;
                let a = *a0 - *d / 1.2;
                let len = 1.8;
                *ph = 2.0 * PI * *d / len;
                let arc = m.river_otter.porpoise.arc_m
                    * m.exaggerate
                    * smooth(c01(t))
                    * smooth(c01(DUR - t));
                let x = c.x + a.cos() * r;
                let z = c.z + a.sin() * r * 0.75;
                let y = c.y - 0.22 * smooth(c01(t)) + arc * ph.sin();
                let ny = c.y - 0.22 + arc * (*ph + 0.1).sin();
                *dy = (ny - y) / (0.1 * len / (2.0 * PI));
                if (x - self.pos.x).abs() + (z - self.pos.z).abs() > 1e-4 {
                    self.yaw = (-(z - self.pos.z)).atan2(x - self.pos.x);
                }
                self.pos = Vec3::new(x, y, z);
                self.v = sp;
                self.swim_ph += dt * 9.0;
                t >= DUR
            }
            Kind::OtterGroom => {
                self.follow_anchor_if_wet(g, m, dt);
                t >= 3.6
            }
            Kind::Look => {
                self.follow_anchor_if_wet(g, m, dt);
                t >= 1.3 + m.river_otter.look.hold_s * 3.0
            }
            Kind::Crack { last } => {
                self.follow_anchor(g, m, dt, 0.0);
                let c = &m.sea_otter.crack;
                let d = crack_d(m);
                let beat: f32 = d.iter().sum();
                let k = (t / beat).floor();
                if t % beat >= d[0] + d[1] + d[2] && (*last as f32) < k && (k as u32) < c.cracks {
                    *last = k as i32;
                    self.hits += 1;
                }
                t >= beat * c.cracks as f32 + 0.4
            }
            Kind::EatUrchin => {
                self.follow_anchor(g, m, dt, 0.0);
                self.urchin = Some((1.0 - t / 3.2).max(0.05));
                t >= 3.2
            }
            Kind::UrchinDone => {
                self.follow_anchor(g, m, dt, 0.0);
                t >= 0.1
            }
            Kind::SeaDive { went, up } => {
                let r = m.sea_otter.roll.roll_s;
                let big_d = 1.6 + m.sea_otter.dive.dive_s;
                let u = t - 0.25 - r;
                if u < 0.0 {
                    self.follow_anchor(g, m, dt, 0.0);
                } else {
                    if !*went {
                        *went = true;
                        self.urchin = None;
                    }
                    let dep =
                        2.2 * smooth(c01(u / 1.0)) * (1.0 - smooth(c01((u - (big_d - 1.3)) / 1.2)));
                    let surf = self.anchor.y;
                    self.pos.y = surf - 0.02 - if u < big_d { dep } else { 0.0 };
                    let fwd = self.fwd();
                    self.pos += fwd * (if u < big_d { 0.7 } else { 0.1 }) * dt;
                    if u > 1.0 && u < big_d - 1.0 {
                        self.yaw += 0.55 * dt;
                    }
                    self.swim_ph += dt * 7.0;
                    self.v = if u < big_d { 0.7 } else { 0.0 };
                    if u > big_d - 0.25 && !*up {
                        *up = true;
                        self.urchin = Some(1.0);
                    }
                }
                t >= 0.25 + 2.0 * r + big_d + 0.2
            }
            Kind::SeaGroom => {
                self.follow_anchor(g, m, dt, 0.0);
                t >= 4.2
            }
            Kind::Sleep => {
                self.follow_anchor(g, m, dt, 0.0);
                t >= m.sea_otter.sleep.kelp_wrap_s + 7.0 + 1.5
            }
        }
    }

    fn follow_anchor_if_wet(&mut self, g: &impl Ground, m: &CreatureMotion, dt: f32) {
        if self.water_at(g) > 0.5 {
            self.follow_anchor(g, m, dt, 0.0);
        }
    }

    /// Swim with the creature's place: toward it at up to what it needs,
    /// turned along the way it goes, the phases on the distance moved.
    fn follow_anchor(&mut self, g: &impl Ground, m: &CreatureMotion, dt: f32, depth: f32) -> f32 {
        let to = self.anchor;
        let (dx, dz) = (to.x - self.pos.x, to.z - self.pos.z);
        let dist = dx.hypot(dz);
        let catch = (1.5f32).max(3.0 * dist) * dt;
        let step_l = dist.min(catch);
        if dist > 1e-5 {
            self.pos.x += dx / dist * step_l;
            self.pos.z += dz / dist * step_l;
        }
        if dist > 0.03 {
            let want = (-dz).atan2(dx);
            let err = super::rig::ang_wrap(want - self.yaw);
            let turn = 3.0 * dt;
            self.yaw += clampf(err, -turn, turn);
        }
        let v = step_l / dt.max(1e-3);
        self.v += (v - self.v) * c01(dt * 3.0);
        let sw = self.water_at(g);
        let swim = self.swim_speed(m);
        self.swim_ph += dt
            * 2.0
            * PI
            * self.scull_hz(m)
            * (0.4 + 0.6 * sw)
            * lerp(m.shared.paddle.rate_idle, 1.0, c01(self.v / swim));
        self.phi += step_l / self.stride(m) * (1.0 - sw);
        let y0 = self.pos.y;
        let target = to.y - depth;
        self.pos.y += (target - self.pos.y) * c01(dt * if depth > 0.0 { 2.0 } else { 5.0 });
        let gr = g.ground(self.pos.x, self.pos.z);
        if self.pos.y < gr {
            self.pos.y = gr;
        }
        self.vy += ((self.pos.y - y0) / dt.max(1e-3) - self.vy) * c01(dt * 5.0);
        sw
    }

    fn swim_speed(&self, m: &CreatureMotion) -> f32 {
        if self.beaver() {
            m.beaver.swim.speed_mps
        } else {
            1.1
        }
    }

    fn scull_hz(&self, m: &CreatureMotion) -> f32 {
        if self.beaver() {
            m.beaver.swim.scull_hz
        } else {
            1.8
        }
    }

    fn stride(&self, m: &CreatureMotion) -> f32 {
        if self.beaver() {
            m.beaver.walk.stride_m
        } else {
            m.river_otter.bound.stride_m
        }
    }

    /// critters.js `Move`'s update: walks or bounds on land, swims in
    /// water, turning on the way. The water fraction and the distance left.
    fn move_toward(
        &mut self,
        g: &impl Ground,
        m: &CreatureMotion,
        dt: f32,
        to: Vec3,
        speed: f32,
        depth: f32,
    ) -> (f32, f32) {
        let (dx, dz) = (to.x - self.pos.x, to.z - self.pos.z);
        let dist = dx.hypot(dz);
        let want = (-dz).atan2(dx);
        let err = super::rig::ang_wrap(want - self.yaw);
        let turn = if self.otter() { 5.0 } else { 3.0 } * dt;
        if dist > 1e-4 {
            self.yaw += clampf(err, -turn, turn);
        }
        let sw = self.water_at(g);
        let walk = if self.beaver() {
            m.beaver.walk.speed_mps
        } else if self.otter() {
            m.river_otter.bound.speed_mps
        } else {
            0.5
        };
        let swim = self.swim_speed(m);
        let vmax = lerp(walk, swim, sw) * speed;
        let vw = vmax * clampf(dist / 0.5, 0.3, 1.0) * clampf(err.cos(), 0.12, 1.0);
        self.v += (vw - self.v) * c01(dt * 3.0);
        let step_l = (self.v * dt).min(dist);
        self.pos += self.fwd() * step_l;
        self.phi += step_l / self.stride(m) * (1.0 - sw);
        self.swim_ph += dt
            * 2.0
            * PI
            * self.scull_hz(m)
            * (0.4 + 0.6 * sw)
            * lerp(m.shared.paddle.rate_idle, 1.0, c01(self.v / swim));
        let gr = g.ground(self.pos.x, self.pos.z);
        let l = Self::level(g, self.pos.x, self.pos.z);
        let y_w = l - depth;
        let y_t = if sw > 0.0 { lerp(gr, y_w, sw) } else { gr };
        let y0 = self.pos.y;
        self.pos.y = if sw > 0.01 {
            self.pos.y + (y_t - self.pos.y) * c01(dt * if depth > 0.0 { 2.0 } else { 5.0 })
        } else {
            y_t
        };
        if self.pos.y < gr {
            self.pos.y = gr;
        }
        self.vy += ((self.pos.y - y0) / dt.max(1e-3) - self.vy) * c01(dt * 5.0);
        (sw, dist)
    }

    // ---------------------------------------------------------------- poses

    fn pose(
        &mut self,
        s: &mut Step,
        g: &impl Ground,
        m: &CreatureMotion,
        lim: &Limits,
        p: &mut Pose,
    ) {
        let t = s.t;
        let ex = m.exaggerate;
        match s.kind {
            Kind::Follow { depth, .. } if self.animal == Animal::SeaOtter && depth <= 0.0 => {
                self.raft_pose(p, self.clock)
            }
            Kind::Follow { depth, .. } => self.move_pose(g, m, lim, p, s.sw, depth > 0.0, 1.0),
            Kind::Move { .. } => self.move_pose(g, m, lim, p, s.sw, false, 1.0),
            Kind::Face { .. } => {
                let sw = self.water_at(g);
                self.move_pose(g, m, lim, p, sw, false, 0.6)
            }
            Kind::Hold { .. } => self.hold_pose(g, m, lim, p),
            Kind::Hop { .. } => {
                let k = c01((t - 0.3) / 0.45);
                if t < 0.3 {
                    if self.water_at(g) > 0.5 {
                        self.swim_pose(m, lim, p, false);
                    } else {
                        self.stand_pose(g, p);
                    }
                    p.y -= 0.04 * smooth(t / 0.3);
                    p.pitch += 0.25 * smooth(t / 0.3);
                } else if k < 1.0 {
                    self.stand_pose(g, p);
                    p.lt = [None; 4];
                    p.la = [0.9, 0.9, -1.2, -1.2];
                    p.lk = [0.25, 0.25, 0.3, 0.3];
                    p.lf = [0.0, 0.0, 0.8, 0.8];
                    p.pitch = 0.5 - k * 0.9;
                    p.a = -0.2;
                } else {
                    self.stand_pose(g, p);
                    let l = c01((t - 0.75) / 0.2);
                    p.y = self.spec().h - 0.05 * (PI * l).sin();
                }
            }
            Kind::GnawLow { .. } => {
                let d = gnaw_d(m);
                let beat: f32 = d.iter().sum();
                let pull = m.beaver.gnaw.pull_rad * ex;
                let (sv, _) = strike(t % beat, d, pull, pull * 1.2, -0.14, 0.1);
                self.stand_pose(g, p);
                p.pitch = -0.12;
                p.y = self.spec().h - 0.02;
                p.hp = -0.55 + sv * 0.8;
                p.hr = 0.3 * c01(sv / pull);
                // the forepaws hold the stick, pressing on each bite
                for i in 0..2 {
                    let l = self.spec().legs[i];
                    let x = l.at.x + 0.05 + 0.012 * c01(-sv / 0.14);
                    p.lt[i] = Some(
                        Target::root(x, self.ground_rel(g, x, l.at.z) + 0.012)
                            .curl(-0.3 - 0.25 * c01(sv / pull)),
                    );
                }
            }
            Kind::TakeStick | Kind::DropStick => {
                self.hold_pose(g, m, lim, p);
                let k = c01(t / 0.45);
                p.hp = -0.6 + 0.5 * smooth(k);
                p.pitch -= 0.1 * (1.0 - k);
            }
            Kind::BeaverSit => self.sit_pose(g, m, lim, p, t),
            Kind::Shake => {
                self.stand_pose(g, p);
                let sh = &m.beaver.shake;
                let t0 = 0.2;
                if t < t0 {
                    p.y = self.spec().h - 0.03 * smooth(t / t0);
                } else {
                    let s = (t - t0) / sh.time_s;
                    let env = (PI * s.min(1.0)).sin().max(0.0).powf(0.6);
                    p.roll = sh.roll_rad * ex * env * (2.0 * PI * sh.hz * (t - t0)).sin();
                    p.y = self.spec().h - 0.015;
                    p.hp = 0.1;
                }
            }
            Kind::Slap => {
                self.swim_pose(m, lim, p, false);
                let d = slap_d(m);
                let r = m.beaver.slap.raise_rad * ex;
                let (sv, _) = strike(t, d, r, r * 1.12, -0.25, 0.3);
                let after = c01((t - d[0] - d[1] - d[2]) / (d[3] + 0.5));
                p.tp = [sv * 0.55, sv * 0.2, sv * 0.06];
                p.ty = [0.0; 3];
                p.pitch = -0.12 * c01(sv / r) - 0.85 * smooth(after);
                p.y = -self.spec().float - 0.03 * c01(sv / r) - 0.35 * after;
                p.hp = 0.3 - 0.6 * after;
            }
            Kind::Porpoise { ph, dy, .. } => {
                self.swim_pose(m, lim, p, false);
                p.pitch = dy.atan2(1.4) * 1.1;
                p.a = -0.18 * ph.sin();
                p.hp = 0.1;
            }
            Kind::OtterGroom => self.otter_groom_pose(g, m, lim, p, t),
            Kind::Look => self.look_pose(g, m, lim, p, t),
            Kind::Crack { .. } => {
                self.raft_pose(p, self.clock);
                let c = &m.sea_otter.crack;
                let d = crack_d(m);
                let beat: f32 = d.iter().sum();
                if t <= beat * c.cracks as f32 {
                    let r = c.raise_m * ex;
                    let (sv, _) = strike(t % beat, d, r, r * 1.15, -0.035, 0.03);
                    p.lt[0] = Some(Target::seg(-0.03, -0.22 - sv, -1.55));
                    p.lt[1] = Some(Target::seg(-0.03, -0.22 - sv, -1.55));
                    p.hp = -0.7 + sv * 0.6;
                    p.a = -0.06 - sv * 0.4;
                }
            }
            Kind::EatUrchin => {
                self.raft_pose(p, self.clock);
                let (sv, _) = strike(t % 0.8, [0.25, 0.1, 0.1, 0.35], 0.2, 0.25, -0.35, 0.08);
                let lift = c01(-sv / 0.35);
                let tg = Target::seg(0.02 + 0.035 * lift, -0.1 - 0.015 * lift, -1.7);
                p.lt[0] = Some(tg);
                p.lt[1] = Some(tg);
                p.hp = -0.9 + sv;
            }
            Kind::UrchinDone => self.raft_pose(p, self.clock),
            Kind::SeaDive { .. } => self.dive_pose(m, lim, p, t),
            Kind::SeaGroom => {
                self.raft_pose(p, self.clock);
                p.roll = PI + 0.55 * (t * 2.3).sin() * (PI * t / 4.2).sin();
                let gk = smooth(c01(t / 0.4)) * smooth(c01((4.2 - t) / 0.4));
                let a = t * 7.0;
                let along = 0.05 * (t * 1.3).sin();
                let sq = 0.02 * (t * 3.5).sin().max(0.0);
                for i in 0..2 {
                    let b = a + i as f32 * 3.0;
                    self.blend_leg(
                        lim,
                        p,
                        i,
                        Target::seg(
                            0.07 + along + 0.025 * b.cos(),
                            -0.055 + 0.015 * b.sin(),
                            -1.75 + 0.3 * b.sin(),
                        ),
                        gk,
                    );
                }
                if p.lt[0].is_none() {
                    p.la[0] += sq * 3.0;
                    p.la[1] -= sq * 3.0;
                }
                p.hy = 0.35 * (t * 2.3).sin();
            }
            Kind::Sleep => {
                self.raft_pose(p, self.clock);
                let w0 = m.sea_otter.sleep.kelp_wrap_s;
                let gk =
                    smooth(c01((t - w0 - 0.3) / 0.9)) * (1.0 - smooth(c01((t - w0 - 6.2) / 0.9)));
                self.blend_leg(lim, p, 0, Target::seg(0.12, -0.13, -1.6), gk);
                self.blend_leg(lim, p, 1, Target::seg(0.12, -0.12, -1.6), gk);
                p.hp = -0.25 - 0.6 * gk;
                p.y = 0.02 + 0.01 * (t * 1.4).sin();
            }
        }
    }

    /// A forepaw sent to the ground that its elbow's limit cannot bring
    /// that close to the shoulder lifts the body by the shortfall, so the paw
    /// stands on the ground rather than hovering over it (B29: at 80 degrees
    /// a beaver's forepaw stopped 1.7 cm short under its shoulder).
    fn reach_ground(&self, g: &impl Ground, p: &mut Pose) {
        let lim = &self.lim;
        let mut lift = 0.0f32;
        for i in 0..2 {
            let Some(t) = p.lt[i] else {
                continue;
            };
            let l = self.spec().legs[i];
            if !t.root || t.y - self.ground_rel(g, t.x, l.at.z) > 0.03 {
                continue;
            }
            let dmin = (l.a * l.a + l.b * l.b + 2.0 * l.a * l.b * lim.e_max.cos()).sqrt() + 0.002;
            let (sx, sy) = self.seg_to_root(p, i, 0.0, 0.0);
            let dx = (t.x - sx).abs();
            if dx < dmin {
                let need = (dmin * dmin - dx * dx).sqrt() - (sy - t.y);
                lift = lift.max(need);
            }
        }
        p.y += lift.clamp(0.0, 0.05);
    }

    /// critters.js `Move`'s pose: the gait and the swim blended by the
    /// water under it.
    #[allow(clippy::too_many_arguments)]
    fn move_pose(
        &self,
        g: &impl Ground,
        m: &CreatureMotion,
        lim: &Limits,
        p: &mut Pose,
        sw: f32,
        deep: bool,
        lift: f32,
    ) {
        let mut a = Pose::default();
        let mut b = Pose::default();
        if sw < 1.0 {
            self.gait_pose(g, m, &mut a, lift);
            solve_legs(self.spec(), lim, &mut a);
        }
        if sw > 0.0 {
            self.swim_pose(m, lim, &mut b, deep);
        }
        p.lerp_from(&a, &b, sw);
        p.lt = [None; 4];
    }

    /// Standing still, or treading water, blended by the depth.
    fn hold_pose(&mut self, g: &impl Ground, m: &CreatureMotion, lim: &Limits, p: &mut Pose) {
        let sw = self.water_at(g);
        if sw > 0.0 {
            self.swim_ph += 0.02 * m.shared.paddle.rate_idle;
            let mut a = Pose::default();
            let mut b = Pose::default();
            if sw < 1.0 {
                self.stand_pose(g, &mut a);
                solve_legs(self.spec(), lim, &mut a);
            }
            self.swim_pose(m, lim, &mut b, false);
            p.lerp_from(&a, &b, sw);
            p.lt = [None; 4];
        } else {
            self.stand_pose(g, p);
        }
    }

    /// A gait: feet in stance stay put, in swing they lift on an arc and
    /// land half a reach ahead (critters.js `gaitPose`).
    fn gait_pose(&self, g: &impl Ground, m: &CreatureMotion, p: &mut Pose, lift_k: f32) {
        let ex = m.exaggerate;
        let walk = self.beaver();
        let (stride, stance, lift, offs) = if walk {
            let w = &m.beaver.walk;
            (
                w.stride_m,
                w.stance,
                w.foot_lift_m,
                [-0.25, -0.75, 0.0, -0.5],
            )
        } else {
            let b = &m.river_otter.bound;
            (b.stride_m, b.stance, b.lift_m, [-0.5, -0.56, 0.0, -0.06])
        };
        let lift = lift * lift_k;
        let reach = stance * stride;
        for i in 0..4 {
            let l = self.spec().legs[i];
            let u = frac(self.phi + offs[i]);
            let (x, y, mut c);
            if u < stance {
                let s = u / stance;
                x = reach / 2.0 - reach * s;
                y = 0.0;
                c = 0.0;
            } else {
                let s = (u - stance) / (1.0 - stance);
                x = -reach / 2.0 + reach * smooth(s);
                y = lift * (PI * s).sin();
                c = 0.0;
                if l.front {
                    c = -m.shared.forelegs.swing_curl_rad * ex * (PI * s).sin() * (1.2 - s);
                }
            }
            let fx = l.at.x + x;
            p.lt[i] = Some(Target::root(fx, self.ground_rel(g, fx, l.at.z) + y).curl(c));
        }
        let slope = (self.ground_rel(g, 0.25, 0.0) - self.ground_rel(g, -0.25, 0.0)).atan2(0.5);
        p.pitch = slope;
        p.y = self.spec().h;
        if walk {
            let w = &m.beaver.walk;
            p.y += w.hip_bob_m * ex * (4.0 * PI * self.phi).cos();
            p.roll = w.waddle_roll_rad * ex * (2.0 * PI * self.phi).sin();
            p.hp = -slope * 0.6 + 0.04 * (4.0 * PI * self.phi - 1.0).sin();
        } else {
            let b = &m.river_otter.bound;
            let arch = b.arch_rad * ex * (0.55 * (2.0 * PI * (self.phi - 0.95)).cos() + 0.25);
            p.a = arch;
            p.y += lift * ex * (4.0 * PI * (self.phi - 0.44)).cos().max(0.0);
            p.pitch += 0.07 * ex * (2.0 * PI * (self.phi - 0.2)).sin();
            p.hp = -p.pitch * 0.8 - 0.05;
        }
        for i in 0..3 {
            p.tp[i] = if walk {
                TAIL_DRAG[i]
            } else {
                0.12 - i as f32 * 0.05
            };
        }
        self.reach_ground(g, p);
    }

    /// Standing still: each foot under its hip.
    fn stand_pose(&self, g: &impl Ground, p: &mut Pose) {
        for i in 0..4 {
            let l = self.spec().legs[i];
            p.lt[i] = Some(Target::root(l.at.x, self.ground_rel(g, l.at.x, l.at.z)));
        }
        p.pitch = (self.ground_rel(g, 0.25, 0.0) - self.ground_rel(g, -0.25, 0.0)).atan2(0.5);
        p.y = self.spec().h;
        for i in 0..3 {
            p.tp[i] = if self.beaver() {
                TAIL_DRAG[i]
            } else {
                0.12 - i as f32 * 0.05
            };
        }
        self.reach_ground(g, p);
    }

    /// Swimming: the forepaws paddling like the feet, the hind feet kicking,
    /// the tail sculling, each part a beat behind (version 5).
    fn swim_pose(&self, m: &CreatureMotion, lim: &Limits, p: &mut Pose, deep: bool) {
        let ph = self.swim_ph;
        let ex = m.exaggerate;
        let beaver = self.beaver();
        p.y = -self.spec().float;
        p.pitch = if beaver { 0.07 } else { 0.04 } + self.vy.atan2(self.v.max(0.25)) * 0.9;
        p.hp = if deep {
            -0.05
        } else if beaver {
            0.32 + m.beaver.swim.head_above_m * 2.0
        } else {
            0.38
        };
        let pd = &m.shared.paddle;
        for i in 0..4 {
            let side = if i % 2 == 1 { PI } else { 0.0 };
            if i < 2 {
                let sp = if beaver { m.beaver.swim.speed_mps } else { 1.1 };
                let spd = if deep { 1.0 } else { c01(self.v / sp) };
                let amp = m.shared.forelegs.paddle_rad * ex * lerp(pd.paddle_idle, 1.0, spd);
                let q = ph + side + pd.paddle_phase_rad;
                let rec = (-(q - PI / 2.0).sin()).max(0.0);
                let e0 = 0.25;
                p.la[i] = -0.4 + amp * q.sin();
                p.lk[i] = e0 + (lim.e_max * pd.paddle_tuck - e0) * rec;
                p.lf[i] = lerp(-1.35, -2.05, rec);
            } else {
                p.la[i] = -0.85 + 0.5 * (ph + side).sin();
                p.lk[i] = -0.6 - 0.45 * (ph + side + 1.2).sin();
                p.lf[i] = 0.4 + 0.5 * (ph + side + 2.0).sin();
            }
        }
        let amp = if beaver {
            m.beaver.swim.scull_rad
        } else {
            0.28
        } * ex
            * c01(self.v / 0.3 + 0.25);
        for i in 0..3 {
            let fi = i as f32;
            if beaver {
                p.ty[i] = amp * (ph - fi * 0.9).sin() * if i == 2 { 0.6 } else { 1.0 };
                p.tp[i] = 0.02;
            } else {
                p.tp[i] = amp * 0.6 * (ph - fi * 0.9).sin();
                p.ty[i] = 0.1 * (ph * 0.5 - fi).sin();
            }
        }
        if !beaver {
            p.a = 0.12 * ex * ph.sin();
        }
    }

    /// The sea otter on its back: the head up looking at its chest, the
    /// hind flippers up, the forepaws resting on the chest toward the chin.
    fn raft_pose(&self, p: &mut Pose, t: f32) {
        p.roll = PI;
        p.pitch = 0.16;
        p.y = 0.02;
        p.hp = -0.55 + 0.05 * (t * 0.7).sin();
        p.a = -0.06;
        for i in 2..4 {
            p.la[i] = -1.1 + 0.08 * (t * 1.3).sin();
            p.lk[i] = 0.4;
            p.lf[i] = -2.1 + 0.2 * (t * 1.3 + 1.0).sin();
        }
        p.lt[0] = Some(Target::seg(0.085, -0.045 - 0.004 * (t * 1.4).sin(), -1.75));
        p.lt[1] = Some(Target::seg(
            0.085,
            -0.045 - 0.004 * (t * 1.4 + 0.4).sin(),
            -1.75,
        ));
        p.tp = [-0.12, 0.05, 0.05];
    }

    /// A point in a foreleg's segment frame (from the shoulder) in the
    /// root's frame, for this pose.
    fn seg_to_root(&self, p: &Pose, i: usize, x: f32, y: f32) -> (f32, f32) {
        let l = self.spec().legs[i];
        let seg_p = -p.a;
        let (s1, c1) = seg_p.sin_cos();
        let lx = l.at.x * c1 - l.at.y * s1;
        let ly = l.at.x * s1 + l.at.y * c1;
        let (s2, c2) = p.pitch.sin_cos();
        let hx = lx * c2 - ly * s2;
        let hy = lx * s2 + ly * c2 + p.y;
        let par = p.pitch + seg_p;
        (
            hx + x * par.cos() - y * par.sin(),
            hy + x * par.sin() + y * par.cos(),
        )
    }

    /// Where an upright animal's hands hang, in the root's frame (version 5).
    fn hang_at(&self, m: &CreatureMotion, p: &Pose, i: usize, dx: f32, dy: f32) -> (f32, f32) {
        let l = self.spec().legs[i];
        let sh = self.seg_to_root(p, i, 0.0, 0.0);
        let r = (l.a + l.b) * m.shared.upright.reach;
        let dr = m.shared.upright.drop_deg * DEG;
        (sh.0 + r * dr.cos() + dx, sh.1 - r * dr.sin() + dy)
    }

    fn hang_w(m: &CreatureMotion) -> f32 {
        -(90.0 + m.shared.upright.curl_deg) * DEG
    }

    /// A forepaw from the ground under its shoulder toward `u`, `k` of the
    /// way, on an arc that bulges up and forward.
    #[allow(clippy::too_many_arguments)]
    fn arm_to(
        &self,
        g: &impl Ground,
        p: &mut Pose,
        i: usize,
        u: (f32, f32),
        k: f32,
        c: f32,
        bulge: f32,
        w: Option<f32>,
        add: Option<f32>,
    ) {
        let l = self.spec().legs[i];
        let gx = l.at.x + 0.01;
        let gy = self.ground_rel(g, gx, l.at.z);
        let kk = clampf(k, -0.2, 1.3);
        let b = (PI * c01(kk)).sin() * bulge;
        p.lt[i] = Some(Target {
            root: true,
            x: lerp(gx, u.0, kk) + b * 0.6,
            y: lerp(gy, u.1, kk) + b,
            f: None,
            c,
            w,
            wk: c01(kk),
        });
        if let Some(add) = add {
            p.lx[i] = add * c01(kk);
        }
    }

    /// One leg blended from whatever the pose asks of it toward `tg` by
    /// `gk`, both solved and their angles mixed.
    fn blend_leg(&self, lim: &Limits, p: &mut Pose, i: usize, tg: Target, gk: f32) {
        if gk <= 0.0 {
            return;
        }
        if p.lt[i].is_some() {
            solve_leg(self.spec(), lim, p, i);
        }
        let a = (p.la[i], p.lk[i], p.lf[i]);
        p.lt[i] = Some(tg);
        solve_leg(self.spec(), lim, p, i);
        p.lt[i] = None;
        p.la[i] = lerp(a.0, p.la[i], gk);
        p.lk[i] = lerp(a.1, p.lk[i], gk);
        p.lf[i] = lerp(a.2, p.lf[i], gk);
    }

    /// The beaver sitting up on its tail to groom (critters.js `sitAct`).
    fn sit_pose(&self, g: &impl Ground, m: &CreatureMotion, lim: &Limits, p: &mut Pose, t: f32) {
        const D: f32 = 3.6;
        self.stand_pose(g, p);
        let r = rear(m, t, D, false);
        let up = r.body;
        let gk = smooth(c01((t - 0.95) / 0.3)) * (1.0 - smooth(r.d));
        p.pitch = lerp(p.pitch, 1.05, up) - 0.08 * r.crouch;
        p.y = self.spec().h + 0.07 * up - 0.02 * r.crouch - 0.012 * r.give;
        p.tp = [
            lerp(TAIL_DRAG[0], 0.95, up),
            lerp(TAIL_DRAG[1], 0.1, up),
            lerp(TAIL_DRAG[2], 0.0, up),
        ];
        p.hp = -0.8 * up + 0.15 * (t * 5.0).sin() * gk + 0.15 * r.crouch;
        let hx = -0.08 * up - 0.2 * (1.0 - up);
        p.lt[2] = Some(Target::root(hx, self.ground_rel(g, -0.1, 0.1)));
        p.lt[3] = Some(Target::root(hx, self.ground_rel(g, -0.1, -0.1)));
        for i in 0..2 {
            let a = t * 7.0 + i as f32 * PI;
            let h = self.hang_at(
                m,
                p,
                i,
                (0.012 + 0.012 * a.cos()) * gk,
                (0.045 + 0.045 * a.sin()) * gk,
            );
            let w = Self::hang_w(m) - 0.2 * gk * (0.5 + 0.5 * a.sin());
            self.arm_to(g, p, i, h, r.arm, r.curl, 0.05, Some(w), Some(lim.add));
        }
        p.hp -= 0.25 * gk;
    }

    /// The river otter rolling onto its back to groom, its paws on its
    /// chest, and back.
    fn otter_groom_pose(
        &self,
        g: &impl Ground,
        m: &CreatureMotion,
        lim: &Limits,
        p: &mut Pose,
        t: f32,
    ) {
        let in_w = self.water_at(g) > 0.5;
        if in_w {
            self.swim_pose(m, lim, p, false);
        } else {
            self.stand_pose(g, p);
        }
        let r = if t < 0.25 {
            -0.2 * smooth(t / 0.25)
        } else if t < 0.8 {
            lerp(-0.2, PI, smooth((t - 0.25) / 0.55))
        } else if t < 2.9 {
            PI + 0.25 * ((t - 0.8) * 3.0).sin()
        } else {
            lerp(PI, 2.0 * PI, smooth(c01((t - 2.9) / 0.6)))
        };
        p.roll = r;
        let gk = smooth(c01((t - 0.45) / 0.35)) * smooth(c01((3.15 - t) / 0.35));
        if gk > 0.0 {
            let a = t * 8.0;
            for i in 0..2 {
                let b = a + i as f32 * 2.5;
                self.blend_leg(
                    lim,
                    p,
                    i,
                    Target::seg(
                        0.072 + 0.012 * b.cos(),
                        -0.03 + 0.01 * b.sin(),
                        -1.85 + 0.25 * b.sin(),
                    ),
                    gk,
                );
            }
            for i in 2..4 {
                if p.lt[i].is_some() {
                    solve_leg(self.spec(), lim, p, i);
                }
                p.lt[i] = None;
                p.la[i] = lerp(p.la[i], 0.6 + 0.3 * (t * 5.0).sin(), gk);
                p.lk[i] = lerp(p.lk[i], -0.8, gk);
            }
            p.hp = -0.35 * gk;
            p.hy = 0.3 * (t * 2.2).sin() * gk;
            p.y = lerp(p.y, if in_w { 0.0 } else { self.spec().h * 0.55 }, gk);
        }
    }

    /// The river otter rearing to look round: on land the paws hang in
    /// front of its belly; in the water it treads.
    fn look_pose(&self, g: &impl Ground, m: &CreatureMotion, lim: &Limits, p: &mut Pose, t: f32) {
        let in_w = self.water_at(g) > 0.5;
        let hold = m.river_otter.look.hold_s;
        let big_d = 1.3 + hold * 3.0;
        let r = rear_with(m, t, big_d, false, None, Some(0.36), None);
        let up = r.body;
        if in_w {
            self.swim_pose(m, lim, p, false);
            p.pitch = 1.25 * up;
            p.y = -self.spec().float + 0.12 * up;
            for i in 0..2 {
                let a = t * 9.0 + i as f32 * 3.0;
                self.blend_leg(
                    lim,
                    p,
                    i,
                    Target::seg(
                        0.06 + 0.02 * a.cos(),
                        -0.05 + 0.02 * a.sin(),
                        -0.6 + 0.3 * a.sin(),
                    ),
                    up,
                );
            }
        } else {
            self.stand_pose(g, p);
            p.pitch = lerp(p.pitch, 1.22, up) - 0.08 * r.crouch;
            p.y = self.spec().h + 0.1 * up - 0.02 * r.crouch - 0.01 * r.give;
            let hx = -0.08 * up - 0.22 * (1.0 - up);
            p.lt[2] = Some(Target::root(hx, self.ground_rel(g, -0.1, 0.06)));
            p.lt[3] = Some(Target::root(hx, self.ground_rel(g, -0.1, -0.06)));
            for i in 0..2 {
                let h = self.hang_at(m, p, i, 0.0, 0.004 * (t * 2.4 + i as f32).sin());
                self.arm_to(
                    g,
                    p,
                    i,
                    h,
                    r.arm,
                    r.curl,
                    0.04,
                    Some(Self::hang_w(m)),
                    Some(lim.add),
                );
            }
            p.tp = [
                lerp(0.12, 0.9, up),
                lerp(0.07, 0.1, up),
                lerp(0.02, 0.0, up),
            ];
        }
        p.hp = -1.15 * up + 0.12 * r.crouch;
        let seg_t = (t - 0.35 - m.shared.forelegs.anticipate_s) / hold;
        let yaws = [0.0, 0.8, 0.8, -0.8, -0.8, 0.0, 0.0];
        let i = clampf((seg_t * 2.0).floor(), 0.0, 5.0) as usize;
        p.hy = lerp(yaws[i], yaws[i + 1], smooth(c01(seg_t * 2.0 - i as f32))) * up;
    }

    /// The sea otter rolling over belly down, diving and coming up with an
    /// urchin, then onto its back again.
    fn dive_pose(&self, m: &CreatureMotion, lim: &Limits, p: &mut Pose, t: f32) {
        let r = m.sea_otter.roll.roll_s;
        let u = t - 0.25 - r;
        let big_d = 1.6 + m.sea_otter.dive.dive_s;
        if t < 0.25 {
            self.raft_pose(p, self.clock);
            p.roll = PI + 0.25 * smooth(t / 0.25);
        } else if u < 0.0 {
            self.raft_pose(p, self.clock);
            p.roll = lerp(PI + 0.25, 2.0 * PI, smooth((t - 0.25) / r));
            p.lt = [None; 4];
            p.la = [-1.2, -1.2, -0.6, -0.6];
            p.lk = [1.2, 1.2, -0.4, -0.4];
        } else if u < big_d {
            self.swim_pose(m, lim, p, true);
            p.roll = 0.0;
            p.a = 0.2 * (u * 7.0).sin();
            for i in 0..3 {
                p.tp[i] = 0.35 * (u * 7.0 - i as f32 * 0.9).sin();
            }
            if u < 0.45 {
                p.tp = [0.9 * (PI * u / 0.45).sin(), 0.4, 0.2];
            }
            p.pitch = -1.1 * smooth(c01(u / 0.45)) * (1.0 - smooth(c01((u - 0.45) / 0.8)))
                + 0.9
                    * smooth(c01((u - (big_d - 1.3)) / 0.6))
                    * (1.0 - smooth(c01((u - (big_d - 0.35)) / 0.35)));
        } else {
            self.raft_pose(p, self.clock);
            p.roll = lerp(0.0, PI, smooth(c01((u - big_d) / r)));
        }
    }
}

/// A bite's four stages on the gnaw's one beat.
pub fn gnaw_d(m: &CreatureMotion) -> [f32; 4] {
    let b = m.beaver.gnaw.beat_s;
    [b * 0.3, b * 0.2, b * 0.12, b * 0.38]
}

fn slap_d(m: &CreatureMotion) -> [f32; 4] {
    let s = &m.beaver.slap;
    [s.windup_s, s.anticipate_s, s.strike_s, s.recoil_s]
}

fn crack_d(m: &CreatureMotion) -> [f32; 4] {
    let c = &m.sea_otter.crack;
    [c.windup_s, c.anticipate_s, c.strike_s, c.recoil_s]
}

/// The rear-up's schedule (critters.js `rear`), `dur` when the forepaws are
/// down again (infinite to stay up).
pub fn rear(m: &CreatureMotion, t: f32, dur: f32, from_up: bool) -> Rear {
    rear_with(m, t, dur, from_up, None, None, None)
}

fn rear_with(
    m: &CreatureMotion,
    t: f32,
    dur: f32,
    from_up: bool,
    ant: Option<f32>,
    up: Option<f32>,
    down: Option<f32>,
) -> Rear {
    let ex = m.exaggerate;
    let f = &m.shared.forelegs;
    let ant = if from_up {
        0.0
    } else {
        ant.unwrap_or(f.anticipate_s)
    };
    let up = if from_up { 1.0 } else { up.unwrap_or(f.rise_s) };
    let dn = down.unwrap_or(f.lower_s);
    let g = (dn * 0.3).min(0.14);
    let t1 = dur - dn;
    let lag = up * 0.3;
    let crouch = if from_up {
        0.0
    } else if t < ant {
        smooth(t / ant.max(1e-3))
    } else {
        1.0 - smooth(c01((t - ant) / (up * 0.4)))
    };
    let rise = if from_up {
        1.0
    } else {
        smooth(c01((t - ant) / up))
    };
    let fall = smooth(c01((t - t1) / (dn - g)));
    let a = if from_up {
        1.0
    } else {
        c01((t - ant - lag) / (up * 0.9))
    };
    let s = t - ant - lag - up * 0.9;
    let settle = if !from_up && s > 0.0 {
        f.overshoot * ex * (-s * 6.0).exp() * (s * 13.0).sin()
    } else {
        0.0
    };
    let d = c01((t - t1 + 0.05) / ((dn - g) * 0.75));
    let arm = (smooth(a) + settle) * (1.0 - smooth(d));
    let give = if dur < 1e8 {
        (PI * c01((t - (dur - g)) / g)).sin()
    } else {
        0.0
    };
    let curl = (if from_up {
        0.0
    } else {
        -0.9 * ex * (PI * a).sin()
    }) * (1.0 - d)
        + 0.5 * ex * (PI * d).sin();
    Rear {
        crouch,
        body: rise * (1.0 - fall),
        arm,
        give,
        curl,
        d,
    }
}
