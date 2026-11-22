//! How a townsperson's body moves (`townsfolk` decisions 17 and 23): the
//! Faces tab's walk (a bob and squash on each step, the limbs swinging
//! against each other), the towns mockup's work loops on the right arm
//! (`animateNpc`: the hammer stroke, the wipe), the idle sway, and the acts
//! (wave, hop, shrug, tilt) as keyed poses blended by its curves. Drawn
//! only: nothing reads a pose.

use super::face::Act;

/// What a person's body is doing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Doing {
    /// Walking at this pace, m/s.
    Walking(f32),
    Hammering,
    Wiping,
    Standing,
}

/// The pose: each pivot's turn, rad, and the body's bob and squash. Arms
/// and legs are left first; an arm's `[x, z]` is its swing forward and its
/// spread from the body.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pose {
    /// The body pivot's rise over its rest, m, and its sway across, m.
    pub body_y: f32,
    pub body_x: f32,
    /// The body's scale across and up (the squash).
    pub scale: [f32; 2],
    /// Its roll, rad.
    pub roll: f32,
    pub legs: [f32; 2],
    pub arms: [[f32; 2]; 2],
    /// The whole figure's lift off the ground (a hop), m.
    pub hop: f32,
    /// The head's extra nod and roll, and its drop, m.
    pub head_nod: f32,
    pub head_roll: f32,
    pub head_drop: f32,
}

/// The body's own clock and the act it is playing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Body {
    phase: f32,
    t: f32,
    act: Option<(Act, f32)>,
    /// A per-person offset in its idle sways.
    ph: f32,
}

fn ease(x: f32) -> f32 {
    x * x * (3.0 - 2.0 * x)
}

/// The mockup's `env(t, d, a, r)`: up over `a`, held, down over `r`.
fn env(t: f32, d: f32, a: f32, r: f32) -> f32 {
    if t < 0.0 || t > d {
        0.0
    } else {
        ease((t / a).min((d - t) / r).clamp(0.0, 1.0))
    }
}

/// The Faces tab walks 0.85 m/s at 2.3 steps a second; other paces step in
/// proportion, within these.
const WALK_M_S: f32 = 0.85;
const STEPS_HZ: f32 = 2.3;

impl Body {
    pub fn new(seed: u32) -> Self {
        Self {
            phase: 0.0,
            t: (seed % 1000) as f32 / 100.0,
            act: None,
            ph: (seed % 628) as f32 / 100.0,
        }
    }

    pub fn play(&mut self, act: Act) {
        self.act = Some((act, 0.0));
    }

    pub fn acting(&self) -> Option<Act> {
        self.act.map(|(a, _)| a)
    }

    pub fn step(&mut self, dt: f32, doing: Doing, tilt: f32) -> Pose {
        self.t += dt;
        let t_now = self.t;
        let (mut wave, mut hop_y, mut squash, mut shrug, mut tilt_a, mut arms_up) =
            (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let mut wave_t = 0.0;
        if let Some((act, t)) = &mut self.act {
            *t += dt;
            let (d, t) = (act.seconds(), *t);
            match act {
                Act::Wave => {
                    wave = env(t, d, 0.22, 0.3);
                    wave_t = t;
                }
                Act::Shrug => shrug = env(t, d, 0.25, 0.35),
                Act::Tilt => tilt_a = env(t, d, 0.3, 0.4),
                Act::Hop => {
                    if t < 0.15 {
                        squash = 0.12 * ease(t / 0.15);
                    } else if t < 0.55 {
                        let u = (t - 0.15) / 0.4;
                        let s = (std::f32::consts::PI * u).sin();
                        hop_y = 0.3 * s;
                        squash = -0.07 * s;
                        arms_up = s;
                    } else {
                        squash = 0.1 * (1.0 - ease(((t - 0.55) / 0.23).min(1.0)));
                    }
                }
            }
            if t > d {
                self.act = None;
            }
        }
        let tilt = tilt_a.max(tilt);
        let stride = match doing {
            Doing::Walking(v) => {
                let rate = STEPS_HZ * (v / WALK_M_S).clamp(0.6, 1.6);
                self.phase += dt * rate * std::f32::consts::TAU;
                1.0
            }
            _ => {
                self.phase = 0.0;
                0.0
            }
        };
        let sp = self.phase.sin();
        let as_ = sp.abs();
        let step_sq = stride * 0.05 * (1.0 - as_ * 3.0).max(0.0);
        let sq = squash + step_sq;
        let breathe = (t_now * 2.1 + self.ph).sin();
        let mut pose = Pose {
            body_y: stride * as_ * 0.035 - sq.max(0.0) * 0.25 + breathe * 0.003,
            body_x: (t_now * 0.5 + self.ph).sin() * 0.01,
            scale: [1.0 + sq * 0.5, 1.0 - sq + breathe * 0.006],
            roll: (t_now * 1.3 + self.ph).sin() * 0.02 + stride * sp * 0.045,
            legs: [stride * sp * 0.55, -stride * sp * 0.55],
            arms: [[0.0; 2]; 2],
            hop: hop_y,
            head_nod: 0.12 * shrug,
            head_roll: 0.26 * tilt - 0.05 * shrug,
            head_drop: 0.025 * shrug,
        };
        for (i, s) in [(0usize, -1.0f32), (1, 1.0)] {
            let mut z = s * (0.12 + 0.75 * shrug + 1.5 * arms_up);
            let mut x = stride * if i == 1 { sp } else { -sp } * 0.6 - 0.45 * shrug;
            if s < 0.0 && wave > 0.0 {
                z = z * (1.0 - wave) + (-2.55 + 0.32 * (wave_t * 13.0).sin()) * wave;
                x *= 1.0 - wave;
            }
            pose.arms[i] = [x, z];
        }
        // The work loops, on the right arm (towns `animateNpc`).
        match doing {
            Doing::Hammering => pose.arms[1][0] = -1.2 + (t_now * 4.0).sin().abs() * 1.4,
            Doing::Wiping => {
                pose.arms[1][0] = -0.9 + (t_now * 3.0).sin() * 0.2;
                pose.arms[1][1] = (t_now * 3.0).sin() * 0.3 + 0.12;
            }
            _ => {}
        }
        pose
    }
}
