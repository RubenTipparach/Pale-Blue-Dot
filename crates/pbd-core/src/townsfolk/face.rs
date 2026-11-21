//! A townsperson's face (`townsfolk` decisions 14, 15 and 22): the ten
//! expressions with M5 version 9's numbers, the mood that picks one from
//! what happens, the live layers (blink, gaze, head follow, talk) and the
//! pixel mouth atlas, drawn as the mockup's canvas draws it.

use super::look::Rng;

/// The ten expressions (design 15).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Expression {
    Neutral,
    Happy,
    Delighted,
    Surprised,
    Worried,
    Sad,
    Cross,
    Sleepy,
    Thinking,
    Talking,
}

impl Expression {
    pub const ALL: [Expression; 10] = [
        Expression::Neutral,
        Expression::Happy,
        Expression::Delighted,
        Expression::Surprised,
        Expression::Worried,
        Expression::Sad,
        Expression::Cross,
        Expression::Sleepy,
        Expression::Thinking,
        Expression::Talking,
    ];

    /// Its id, for logs and capture instruments.
    pub fn id(self) -> &'static str {
        match self {
            Expression::Neutral => "neutral",
            Expression::Happy => "happy",
            Expression::Delighted => "delighted",
            Expression::Surprised => "surprised",
            Expression::Worried => "worried",
            Expression::Sad => "sad",
            Expression::Cross => "cross",
            Expression::Sleepy => "sleepy",
            Expression::Thinking => "thinking",
            Expression::Talking => "talking",
        }
    }

    pub fn of_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|e| e.id() == id)
    }

    /// The mockup's `EXPR` row.
    pub fn params(self) -> Params {
        let p = |eye, pup, lid, by, bt, mouth, blush| Params {
            eye,
            pup,
            lid,
            by,
            bt,
            mouth,
            blush,
            arcs: false,
            asym: 0.0,
            mx: 0.0,
            look: None,
            act: None,
            talk: false,
        };
        use Mouth::*;
        match self {
            Expression::Neutral => p(1.00, 1.00, 0.12, 0.000, 0.00, Flat, 0.0),
            Expression::Happy => p(1.04, 1.06, 0.05, 0.008, 0.06, Smile, 0.7),
            Expression::Delighted => Params {
                arcs: true,
                act: Some(Act::Hop),
                ..p(1.05, 1.00, 0.00, 0.020, 0.08, Grin, 1.0)
            },
            Expression::Surprised => p(1.15, 0.70, 0.00, 0.026, 0.03, O, 0.0),
            Expression::Worried => p(1.02, 0.95, 0.10, 0.012, 0.34, Wobble, 0.0),
            Expression::Sad => Params {
                look: Some([0.1, -0.6]),
                ..p(0.98, 1.08, 0.16, 0.002, 0.38, Frown, 0.0)
            },
            Expression::Cross => p(0.95, 0.90, 0.30, -0.014, -0.36, Frown, 0.0),
            Expression::Sleepy => Params {
                look: Some([0.0, -0.4]),
                ..p(1.00, 1.00, 0.62, -0.006, -0.03, M, 0.0)
            },
            Expression::Thinking => Params {
                asym: 0.022,
                mx: 0.022,
                look: Some([0.75, 0.85]),
                act: Some(Act::Tilt),
                ..p(1.00, 1.00, 0.14, 0.004, 0.00, Flat, 0.0)
            },
            Expression::Talking => Params {
                talk: true,
                ..p(1.03, 1.05, 0.05, 0.010, 0.06, Smile, 0.35)
            },
        }
    }
}

/// An expression's numbers: eye and pupil scale, the lid's rest (0 open to
/// 1 shut), the brows' lift (m) and tilt (rad), the mouth, the blush, the
/// happy arcs in place of the eyes, one brow's extra lift and the mouth's
/// shift (m), a fixed look, an act it plays, and whether it talks.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Params {
    pub eye: f32,
    pub pup: f32,
    pub lid: f32,
    pub by: f32,
    pub bt: f32,
    pub mouth: Mouth,
    pub blush: f32,
    pub arcs: bool,
    pub asym: f32,
    pub mx: f32,
    pub look: Option<[f32; 2]>,
    pub act: Option<Act>,
    pub talk: bool,
}

/// A mouth atlas cell (`CELL`), and the eyes' two drawn shapes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mouth {
    Smile,
    Grin,
    O,
    Flat,
    Frown,
    Wobble,
    A,
    OpenO,
    M,
    /// The upturned eye arc of a delighted face.
    Happy,
    Closed,
}

impl Mouth {
    pub const ALL: [Mouth; 11] = [
        Mouth::Smile,
        Mouth::Grin,
        Mouth::O,
        Mouth::Flat,
        Mouth::Frown,
        Mouth::Wobble,
        Mouth::A,
        Mouth::OpenO,
        Mouth::M,
        Mouth::Happy,
        Mouth::Closed,
    ];

    /// Its cell in the atlas, 0..12, four across.
    pub fn cell(self) -> usize {
        Self::ALL.iter().position(|m| *m == self).unwrap_or(0)
    }
}

/// An act: keyed poses blended by curves (design 17).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    Wave,
    Hop,
    Shrug,
    Tilt,
}

impl Act {
    /// How long it plays, s (`ACTS`).
    pub fn seconds(self) -> f32 {
        match self {
            Act::Wave => 1.9,
            Act::Hop => 0.78,
            Act::Shrug => 1.2,
            Act::Tilt => 1.7,
        }
    }
}

// ---------------------------------------------------------------- the mood

/// Within this of the player a person meets them and looks at them, m.
pub const MEET_M: f32 = 5.0;
/// Past this the meeting is over, and the next one surprises again, m.
pub const PART_M: f32 = 8.0;
/// How long the surprise and then the smile of a meeting last, s.
pub const SURPRISE_S: f32 = 0.5;
pub const GREET_S: f32 = 6.0;

/// What a person's mood is told each frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Seen {
    /// How far the player is, m, if near enough to matter.
    pub player_m: Option<f32>,
    /// Whether it is night where they stand.
    pub night: bool,
    /// Whether they are at hammer work (the smith, the boatwright).
    pub hammering: bool,
}

/// What picks the face (decision 22).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Mood {
    pub resting: Expression,
    /// Seconds since the player came within [`MEET_M`], while met.
    met: Option<f32>,
    /// One set from outside, and for how much longer, s.
    held: Option<(Expression, f32)>,
}

/// The mood's answer: the expression, and whether the brows are drawn down
/// in focus.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chosen {
    pub expression: Expression,
    pub focus: bool,
}

impl Mood {
    pub fn new(resting: Expression) -> Self {
        Self {
            resting,
            met: None,
            held: None,
        }
    }

    /// Hold `expression` for `seconds`, over everything else.
    pub fn hold(&mut self, expression: Expression, seconds: f32) {
        self.held = Some((expression, seconds.max(0.0)));
    }

    pub fn met(&self) -> bool {
        self.met.is_some()
    }

    pub fn step(&mut self, dt: f32, seen: Seen) -> Chosen {
        if let Some((_, left)) = &mut self.held {
            *left -= dt;
        }
        if self.held.is_some_and(|(_, left)| left <= 0.0) {
            self.held = None;
        }
        match (self.met, seen.player_m) {
            (None, Some(d)) if d <= MEET_M => self.met = Some(0.0),
            (Some(_), d) if d.is_none_or(|d| d > PART_M) => self.met = None,
            (Some(t), _) => self.met = Some(t + dt),
            _ => {}
        }
        if let Some((expression, _)) = self.held {
            return Chosen {
                expression,
                focus: false,
            };
        }
        let expression = match self.met {
            Some(t) if t < SURPRISE_S => Expression::Surprised,
            Some(t) if t < SURPRISE_S + GREET_S => Expression::Happy,
            _ if seen.night => Expression::Sleepy,
            _ => self.resting,
        };
        Chosen {
            expression,
            focus: seen.hammering && self.met.is_none() && expression == self.resting,
        }
    }
}

// ---------------------------------------------------------------- the layers

/// What the face's layers draw this frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FacePose {
    /// The eyes' scale (with the person's own), the pupils' scale.
    pub eye: f32,
    pub pup: f32,
    /// The pupils' slide inside the whites, -1..1 across and up.
    pub pupil: [f32; 2],
    /// The lids, 0 open to 1 shut.
    pub lid: f32,
    /// Each brow's lift, m, and tilt, rad (left, right).
    pub brow_y: [f32; 2],
    pub brow_tilt: [f32; 2],
    pub mouth: Mouth,
    /// The mouth's shift across, m.
    pub mouth_x: f32,
    pub blush: f32,
    pub arcs: bool,
    /// The head's turn and nod toward the gaze, rad.
    pub head_yaw: f32,
    pub head_pitch: f32,
    /// The head's tilt, 0..1 (thinking).
    pub tilt: f32,
}

/// The face's own clock and sequences.
#[derive(Clone, Debug, PartialEq)]
pub struct Face {
    rng: Rng,
    cur: Params,
    expression: Expression,
    next_blink: f32,
    blink_t: f32,
    double: bool,
    hold: f32,
    /// The gaze's target, yaw and pitch from the body's forward, rad.
    target: [f32; 2],
    pupil: [f32; 2],
    head: [f32; 2],
    brow_pop: f32,
    talk_t: f32,
    talk_cell: Mouth,
    clock: f32,
}

/// The size of a face's pieces, for the pupils' room to slide.
pub const WR: f32 = 0.08;
pub const PR: f32 = 0.048;
/// The face's pieces are 1.6 times version 8's (`FSC`).
pub const FSC: f32 = 1.6;

const NUMK_BLEND_S: f32 = 0.08;

impl Face {
    pub fn new(seed: u32, expression: Expression) -> Self {
        let mut rng = Rng::new(seed ^ 0x5bd1_e995);
        let next_blink = 1.0 + rng.next() * 3.0;
        Self {
            rng,
            cur: expression.params(),
            expression,
            next_blink,
            blink_t: -1.0,
            double: false,
            hold: 0.0,
            target: [0.0, 0.0],
            pupil: [0.0, 0.0],
            head: [0.0, 0.0],
            brow_pop: 0.0,
            talk_t: 0.0,
            talk_cell: Mouth::A,
            clock: 0.0,
        }
    }

    pub fn expression(&self) -> Expression {
        self.expression
    }

    /// One frame: `chosen` from the mood, `look` the player's eye as yaw and
    /// pitch from the body's forward while it is met (else the face looks
    /// about on its own), `talking` while the person talks.
    pub fn step(
        &mut self,
        dt: f32,
        chosen: Chosen,
        look: Option<[f32; 2]>,
        talking: bool,
    ) -> FacePose {
        self.clock += dt;
        let want = chosen.expression.params();
        self.expression = chosen.expression;
        let k = 1.0 - (-dt / NUMK_BLEND_S).exp();
        let blend = |a: &mut f32, b: f32| *a += (b - *a) * k;
        blend(&mut self.cur.eye, want.eye);
        blend(&mut self.cur.pup, want.pup);
        blend(&mut self.cur.lid, want.lid);
        blend(
            &mut self.cur.by,
            want.by + if chosen.focus { -0.008 } else { 0.0 },
        );
        blend(
            &mut self.cur.bt,
            want.bt + if chosen.focus { -0.22 } else { 0.0 },
        );
        blend(&mut self.cur.blush, want.blush);
        blend(&mut self.cur.asym, want.asym);
        blend(&mut self.cur.mx, want.mx);
        // Blink: every 2 to 6 s, sometimes twice; closes in 60 ms, holds 30,
        // opens in 120.
        let mut blink = 0.0;
        if self.blink_t < 0.0 {
            self.next_blink -= dt;
            if self.next_blink <= 0.0 {
                self.blink_t = 0.0;
                self.double = self.rng.next() < 0.2;
            }
        } else {
            self.blink_t += dt;
            let t = self.blink_t;
            blink = if t < 0.06 {
                t / 0.06
            } else if t < 0.09 {
                1.0
            } else {
                (1.0 - (t - 0.09) / 0.12).max(0.0)
            };
            if t > 0.21 {
                self.blink_t = -1.0;
                self.next_blink = if self.double {
                    0.12
                } else {
                    2.0 + self.rng.next() * 4.0
                };
                self.double = false;
            }
        }
        // Gaze: held targets with saccades between them; the player's eye
        // while met.
        self.hold -= dt;
        if let Some(at) = look {
            if (at[0] - self.target[0]).abs() + (at[1] - self.target[1]).abs() > 0.08
                || self.hold <= 0.0
            {
                self.target = at;
                self.hold = 0.15;
            }
        } else if self.hold <= 0.0 {
            self.hold = 0.5 + self.rng.next() * 1.5;
            self.brow_pop = 1.0;
            self.target = [
                (self.rng.next() - 0.5) * 1.6,
                (self.rng.next() - 0.55) * 0.5,
            ];
        }
        // The head follows the eyes late; the pupils do the rest.
        let kh = 1.0 - (-dt / 0.32).exp();
        let hy = (self.target[0] * 0.55).clamp(-0.6, 0.6);
        let hp = (-self.target[1] * 0.4).clamp(-0.3, 0.3);
        self.head[0] += (hy - self.head[0]) * kh;
        self.head[1] += (hp - self.head[1]) * kh;
        let (mut dx, mut dy) = (
            ((self.target[0] - self.head[0]) / 0.7).clamp(-1.0, 1.0),
            ((self.target[1] + self.head[1]) / 0.5).clamp(-1.0, 1.0),
        );
        if let Some([x, y]) = want.look
            && look.is_none()
        {
            (dx, dy) = (x, y);
        }
        let ks = 1.0 - (-dt / 0.014).exp();
        self.pupil[0] += (dx - self.pupil[0]) * ks;
        self.pupil[1] += (dy - self.pupil[1]) * ks;
        self.brow_pop *= (-dt / 0.15).exp();
        // The mouth: the expression's cell, or talk cycling A, O and M.
        let talk = talking || want.talk;
        if talk {
            self.talk_t -= dt;
            if self.talk_t <= 0.0 {
                self.talk_t = 0.08 + self.rng.next() * 0.09;
                self.talk_cell = self.rng.pick(&[
                    Mouth::A,
                    Mouth::OpenO,
                    Mouth::M,
                    Mouth::A,
                    Mouth::Smile,
                    Mouth::OpenO,
                    Mouth::M,
                ]);
            }
        }
        let mouth = if talk { self.talk_cell } else { want.mouth };
        let lid_gaze = (-self.pupil[1]).max(0.0) * 0.3;
        let lid = blink.max(self.cur.lid + lid_gaze).min(1.0);
        let brow = |s: f32| {
            FSC * (self.cur.by
                + 0.006 * self.brow_pop
                + if s > 0.0 { self.cur.asym } else { 0.0 }
                + if talk {
                    0.004 * (self.clock * 9.0 + s).sin()
                } else {
                    0.0
                })
        };
        let tilt_of = |s: f32| -s * self.cur.bt + s * 0.06;
        FacePose {
            eye: self.cur.eye,
            pup: self.cur.pup,
            pupil: self.pupil,
            lid,
            brow_y: [brow(-1.0), brow(1.0)],
            brow_tilt: [tilt_of(-1.0), tilt_of(1.0)],
            mouth,
            mouth_x: self.cur.mx * FSC,
            blush: self.cur.blush,
            arcs: want.arcs,
            head_yaw: self.head[0],
            head_pitch: self.head[1],
            tilt: if chosen.expression == Expression::Thinking {
                0.6
            } else {
                0.0
            },
        }
    }
}

// ---------------------------------------------------------------- the atlas

/// The mouth atlas: 4 x 3 cells of 24 x 12 px, as M5's canvas (`AC`).
pub const ATLAS_W: usize = 96;
pub const ATLAS_H: usize = 36;
pub const CELL_W: usize = 24;
pub const CELL_H: usize = 12;

const LIP: u32 = 0x5c2228;
const IN: u32 = 0x3e1219;
const TNG: u32 = 0xd8707a;
const TTH: u32 = 0xf4efe6;
const INK: u32 = 0x23203a;

/// The atlas as RGBA, sRGB, transparent where nothing is drawn, rows from
/// the top: the mockup's `curve`, `oval` and hand-set pixels.
pub fn mouth_atlas() -> Vec<[u8; 4]> {
    let mut px = vec![[0u8; 4]; ATLAS_W * ATLAS_H];
    let mut put = |cell: Mouth, x: i32, y: i32, col: u32| {
        if !(0..=23).contains(&x) || !(0..=11).contains(&y) {
            return;
        }
        let c = cell.cell();
        let (ox, oy) = ((c % 4) * CELL_W, (c / 4) * CELL_H);
        px[(oy + y as usize) * ATLAS_W + ox + x as usize] =
            [(col >> 16) as u8, (col >> 8) as u8, col as u8, 255];
    };
    // JavaScript's Math.round: half up.
    let round = |v: f32| (v + 0.5).floor() as i32;
    let curve = |put: &mut dyn FnMut(Mouth, i32, i32, u32),
                 cell: Mouth,
                 x0: i32,
                 x1: i32,
                 y_end: f32,
                 y_mid: f32,
                 th: i32,
                 col: u32| {
        let c = (x0 + x1) as f32 / 2.0;
        let h = (x1 - x0) as f32 / 2.0;
        for x in x0..=x1 {
            let t = (x as f32 - c) / h;
            let y = round(y_end + (y_mid - y_end) * (1.0 - t * t));
            for k in 0..th {
                put(cell, x, y + k, col);
            }
        }
    };
    let oval = |put: &mut dyn FnMut(Mouth, i32, i32, u32),
                cell: Mouth,
                cx: f32,
                cy: f32,
                rx: f32,
                ry: f32,
                teeth: bool,
                tongue: bool| {
        for y in 0..12 {
            for x in 0..24 {
                let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
                let d = ((fx - cx) / rx).powi(2) + ((fy - cy) / ry).powi(2);
                if d > 1.0 {
                    continue;
                }
                let mut c = if d > 0.62 { LIP } else { IN };
                if teeth && c == IN && fy < cy - ry * 0.45 {
                    c = TTH;
                }
                if tongue && c == IN && fy > cy + ry * 0.25 && (fx - cx).abs() < rx * 0.55 {
                    c = TNG;
                }
                put(cell, x, y, c);
            }
        }
    };
    curve(&mut put, Mouth::Smile, 6, 17, 4.0, 7.0, 2, LIP);
    for x in 5..=18 {
        let t = (x as f32 - 11.5) / 6.5;
        let bot = round(4.0 + 6.0 * (1.0 - t * t));
        for y in 3..=bot {
            let mut col = if y == 3 || y == bot { LIP } else { IN };
            if col == IN && y == 4 && t.abs() < 0.8 {
                col = TTH;
            }
            if col == IN && y >= bot - 2 && t.abs() < 0.45 {
                col = TNG;
            }
            put(Mouth::Grin, x, y, col);
        }
    }
    oval(&mut put, Mouth::O, 12.0, 6.5, 3.0, 4.0, false, true);
    for x in 8..=15 {
        put(Mouth::Flat, x, 6, LIP);
        put(Mouth::Flat, x, 7, LIP);
    }
    curve(&mut put, Mouth::Frown, 7, 16, 8.0, 5.0, 2, LIP);
    for x in 6..=17 {
        let y = 6 + round(((x - 6) as f32 * std::f32::consts::PI / 2.6).sin() * 1.1);
        put(Mouth::Wobble, x, y, LIP);
        put(Mouth::Wobble, x, y + 1, LIP);
    }
    oval(&mut put, Mouth::A, 12.0, 6.0, 4.6, 4.2, true, true);
    oval(&mut put, Mouth::OpenO, 12.0, 6.0, 2.8, 3.0, false, true);
    for x in 7..=16 {
        put(Mouth::M, x, 5, LIP);
        put(Mouth::M, x, 6, 0x8a3a42);
    }
    curve(&mut put, Mouth::Happy, 3, 20, 9.0, 2.0, 3, INK);
    curve(&mut put, Mouth::Closed, 4, 19, 3.0, 7.0, 2, INK);
    px
}
