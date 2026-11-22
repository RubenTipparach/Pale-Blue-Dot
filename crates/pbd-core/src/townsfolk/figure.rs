//! A townsperson's figure (`townsfolk` decision 21): M5 version 9's
//! `friend()`, ported piece for piece. Every piece is flat shaded and
//! carries its colour per vertex (sRGB `0xRRGGBB`), which the game turns
//! into a swatch of its atlas; a part is in its pivot's own space, the
//! figure facing +z with y up.

use super::look::{Beard, Hair, Look, Nose, shade};
use glam::Vec3;

/// The head block, wide by tall by deep, and its bevel (`HW`, `HH`, `HD`,
/// `HC`), metres at head scale 1.
pub const HW: f32 = 0.5;
pub const HH: f32 = 0.6;
pub const HD: f32 = 0.45;
pub const HC: f32 = 0.035;
/// The head's flat front, where the face sits.
pub const HA: f32 = HD / 2.0;
/// The face's heights on the head: the eyes, the mouth, the brows, and the
/// lowest any hair or hat comes at the front.
pub const EYE_Y: f32 = 0.285;
pub const MOUTH_Y: f32 = 0.108;
pub const BROW_Y: f32 = 0.38;
pub const HAIR_Y: f32 = 0.48;
/// The face's pieces: the eye white, the pupil, its catch-light.
pub const WR: f32 = super::face::WR;
pub const PR: f32 = super::face::PR;
pub const GR: f32 = 0.016;
/// The mouth's quad, wide by high.
pub const MOUTH_WH: [f32; 2] = [0.24, 0.12];

/// The face's own colours: the eye white, the pupil, the catch-light, the
/// lash line.
pub const WHITE: u32 = 0xf6f3ea;
pub const PUPIL: u32 = 0x23203a;
pub const GLINT: u32 = 0xffffff;
pub const LASH: u32 = 0x3a2a26;
/// A guard's spear shaft: the towns mockup's timber.
pub const SPEAR: u32 = 0x6b4a2e;
/// The colours every figure may wear whatever its look: the face's and the
/// spear's.
pub const FACE_COLOURS: [u32; 5] = [WHITE, PUPIL, GLINT, LASH, SPEAR];

/// Triangles with a colour each corner, and flat normals.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Part {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub colours: Vec<u32>,
}

impl Part {
    pub fn triangles(&self) -> usize {
        self.positions.len() / 3
    }

    /// A polygon, fanned, its normal turned to `want` (the mockup's
    /// `Geo.face`, with Newell's normal).
    fn face(&mut self, pts: &[Vec3], want: Vec3, colour: u32) {
        let mut n = Vec3::ZERO;
        for i in 0..pts.len() {
            let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
            n += Vec3::new(
                (a.y - b.y) * (a.z + b.z),
                (a.z - b.z) * (a.x + b.x),
                (a.x - b.x) * (a.y + b.y),
            );
        }
        if n.length() < 1e-7 {
            return;
        }
        let mut n = n.normalize();
        let flip = want != Vec3::ZERO && n.dot(want) < 0.0;
        if flip {
            n = -n;
        }
        let at = |k: usize| if flip { pts[pts.len() - 1 - k] } else { pts[k] };
        for i in 1..pts.len() - 1 {
            for k in [0, i, i + 1] {
                self.positions.push(at(k).to_array());
                self.normals.push(n.to_array());
                self.colours.push(colour);
            }
        }
    }

    fn extend(&mut self, other: Part) {
        self.positions.extend(other.positions);
        self.normals.extend(other.normals);
        self.colours.extend(other.colours);
    }
}

/// The mockup's `lathe` options.
#[derive(Clone, Copy, Debug)]
struct LatheOpts {
    sx: f32,
    sz: f32,
    cz: f32,
    open0: bool,
    open1: bool,
}

impl Default for LatheOpts {
    fn default() -> Self {
        Self {
            sx: 1.0,
            sz: 1.0,
            cz: 0.0,
            open0: false,
            open1: false,
        }
    }
}

/// A six-sided lathe: `[y, r]` rings bottom to top, `r` the circumradius.
fn lathe(profile: &[[f32; 2]], o: LatheOpts, colour: u32) -> Part {
    let mut g = Part::default();
    let six = |k: usize| k as f32 * std::f32::consts::PI / 3.0;
    let ring = |[y, r]: [f32; 2]| -> [Vec3; 6] {
        std::array::from_fn(|k| {
            let a = six(k);
            Vec3::new(r * a.cos() * o.sx, y, r * a.sin() * o.sz + o.cz)
        })
    };
    for i in 0..profile.len() - 1 {
        let (a, b) = (ring(profile[i]), ring(profile[i + 1]));
        for k in 0..6 {
            let m = (k as f32 + 0.5) * std::f32::consts::PI / 3.0;
            let want = Vec3::new(m.cos(), 0.0, m.sin());
            let (a0, a1, b0, b1) = (a[k], a[(k + 1) % 6], b[k], b[(k + 1) % 6]);
            if profile[i][1] == 0.0 {
                g.face(&[a0, b1, b0], want, colour);
            } else if profile[i + 1][1] == 0.0 {
                g.face(&[a0, a1, b0], want, colour);
            } else {
                g.face(&[a0, a1, b1, b0], want, colour);
            }
        }
    }
    let (f, l) = (profile[0], profile[profile.len() - 1]);
    if f[1] > 0.0 && !o.open0 {
        g.face(&ring(f), Vec3::NEG_Y, colour);
    }
    if l[1] > 0.0 && !o.open1 {
        g.face(&ring(l), Vec3::Y, colour);
    }
    g
}

/// The mockup's `block` options: where it stands, and its top and bottom
/// bevels where they are not its upright edges'.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct BlockOpts {
    pub x: f32,
    pub z: f32,
    pub c_top: Option<f32>,
    pub c_bot: Option<f32>,
}

/// A `w` x `h` x `d` box standing on `y0`, its upright edges cut by `c`
/// and its top and bottom edges by its own bevels.
pub(crate) fn block(w: f32, h: f32, d: f32, c: f32, y0: f32, o: BlockOpts, colour: u32) -> Part {
    let mut g = Part::default();
    let (ct, cb) = (o.c_top.unwrap_or(c), o.c_bot.unwrap_or(c));
    let ring = |y: f32, i: f32| -> Vec<Vec3> {
        let (a, b) = (w / 2.0 - i, d / 2.0 - i);
        let k = c.min(a * 0.9).min(b * 0.9);
        let p: Vec<[f32; 2]> = if k > 1e-6 {
            vec![
                [a, b - k],
                [a - k, b],
                [-a + k, b],
                [-a, b - k],
                [-a, -b + k],
                [-a + k, -b],
                [a - k, -b],
                [a, -b + k],
            ]
        } else {
            vec![[a, b], [-a, b], [-a, -b], [a, -b]]
        };
        p.into_iter()
            .map(|[x, z]| Vec3::new(x + o.x, y, z + o.z))
            .collect()
    };
    let mut prof = Vec::new();
    if cb > 0.0 {
        prof.push([y0, cb]);
    }
    prof.push([y0 + cb, 0.0]);
    prof.push([y0 + h - ct, 0.0]);
    if ct > 0.0 {
        prof.push([y0 + h, ct]);
    }
    let rings: Vec<Vec<Vec3>> = prof.iter().map(|&[y, i]| ring(y, i)).collect();
    for i in 0..rings.len() - 1 {
        let (a, b) = (&rings[i], &rings[i + 1]);
        let n = a.len();
        for k in 0..n {
            let (a0, a1, b0, b1) = (a[k], a[(k + 1) % n], b[k], b[(k + 1) % n]);
            let want = Vec3::new((a0.x + a1.x) / 2.0 - o.x, 0.0, (a0.z + a1.z) / 2.0 - o.z);
            g.face(&[a0, a1, b1, b0], want, colour);
        }
    }
    g.face(&rings[0], Vec3::NEG_Y, colour);
    g.face(&rings[rings.len() - 1], Vec3::Y, colour);
    g
}

/// A six-sided disc of radius `r` facing +z, at `z` (the mockup's
/// `CircleGeometry(r, 6)`: six triangles round its middle).
pub fn disc(r: f32, colour: u32) -> Part {
    let mut g = Part::default();
    for k in 0..6 {
        let a = k as f32 * std::f32::consts::PI / 3.0;
        let b = (k + 1) as f32 * std::f32::consts::PI / 3.0;
        g.face(
            &[
                Vec3::ZERO,
                Vec3::new(r * a.cos(), r * a.sin(), 0.0),
                Vec3::new(r * b.cos(), r * b.sin(), 0.0),
            ],
            Vec3::Z,
            colour,
        );
    }
    g
}

/// A quad `w` by `h` facing +z, from `y0` to `y0 + h`, centred across.
pub fn quad(w: f32, h: f32, y0: f32, colour: u32) -> Part {
    let mut g = Part::default();
    let (x0, x1) = (-w / 2.0, w / 2.0);
    g.face(
        &[
            Vec3::new(x0, y0, 0.0),
            Vec3::new(x1, y0, 0.0),
            Vec3::new(x1, y0 + h, 0.0),
            Vec3::new(x0, y0 + h, 0.0),
        ],
        Vec3::Z,
        colour,
    );
    g
}

fn prof_at(prof: &[[f32; 2]], t: f32) -> f32 {
    for i in 0..prof.len() - 1 {
        let ([a, ra], [b, rb]) = (prof[i], prof[i + 1]);
        if t <= b {
            return ra + (rb - ra) * (t - a) / (b - a);
        }
    }
    prof[prof.len() - 1][1]
}

fn cut_prof(prof: &[[f32; 2]], t0: f32, t1: f32, h: f32, w: f32) -> Vec<[f32; 2]> {
    let mut out = vec![[t0, prof_at(prof, t0)]];
    out.extend(prof.iter().filter(|p| p[0] > t0 && p[0] < t1).copied());
    out.push([t1, prof_at(prof, t1)]);
    out.into_iter().map(|[t, r]| [t * h, r * w]).collect()
}

/// A person's body as pivots and the parts on them (the mockup's groups):
/// the body pivot over the root at `leg_len`, the legs and arms on it, the
/// head on it (scaled by `head_scale`), and the face's places on the head.
#[derive(Clone, Debug, PartialEq)]
pub struct Figure {
    pub leg_len: f32,
    pub torso_h: f32,
    pub body: Part,
    pub legs: [Part; 2],
    /// Each leg's and arm's pivot in the body's space; left first.
    pub leg_at: [Vec3; 2],
    pub arms: [Part; 2],
    pub arm_at: [Vec3; 2],
    /// Shoulder to wrist, m.
    pub arm_len: f32,
    /// The head's pivot in the body's space, and its scale.
    pub head_at: Vec3,
    pub head_scale: f32,
    pub head: Part,
    /// The face's colours of this person: the lids' skin, the brows' and
    /// the blush at each of [`super::look::BLUSH_LEVELS`].
    pub skin: u32,
    pub brow: u32,
    pub blush: [u32; 3],
}

impl Figure {
    pub fn of(look: &Look) -> Self {
        let (b, hs) = (look.build, look.head);
        let prof = b.profile();
        let head_h = HH * hs;
        let rest = look.height_m - head_h + 0.03;
        let leg_len = rest * b.legs();
        let torso_h = rest - leg_len;
        let w = b.width();
        let skin_s = shade(look.skin, 0.84);
        let shirt_s = shade(look.shirt, 0.74);
        let pants_s = shade(look.trousers, 0.72);
        let lo = LatheOpts::default();
        let mut body = lathe(&cut_prof(prof, 0.0, 0.22, torso_h, w), lo, look.trousers);
        body.extend(lathe(
            &cut_prof(prof, 0.22, 1.0, torso_h, w),
            lo,
            look.shirt,
        ));
        body.extend(lathe(
            &cut_prof(prof, 0.19, 0.26, torso_h, w * 1.045),
            LatheOpts {
                open0: true,
                open1: true,
                ..lo
            },
            pants_s,
        ));
        body.extend(lathe(
            &cut_prof(prof, 0.9, 1.0, torso_h, w * 1.06),
            LatheOpts { open0: true, ..lo },
            shirt_s,
        ));
        let leg = || {
            let mut p = lathe(
                &[[-(leg_len - 0.05), 0.068], [0.0, 0.068]],
                lo,
                look.trousers,
            );
            p.extend(lathe(
                &[[-leg_len - 0.03, 0.085], [-leg_len + 0.06, 0.085]],
                LatheOpts {
                    sz: 1.3,
                    cz: 0.03,
                    ..lo
                },
                look.shoe,
            ));
            p
        };
        let sh_y = torso_h * 0.84;
        let sh_x = prof_at(prof, 0.84) * w + 0.035;
        let arm_l = (torso_h * 0.56).max(0.24);
        let arm = || {
            let mut p = lathe(
                &[[-arm_l + 0.03, 0.05], [-0.02, 0.055], [0.03, 0.045]],
                lo,
                look.shirt,
            );
            p.extend(lathe(
                &[[-arm_l, 0.057], [-arm_l + 0.05, 0.057]],
                lo,
                shirt_s,
            ));
            p.extend(lathe(
                &[
                    [-arm_l - 0.1, 0.0],
                    [-arm_l - 0.075, 0.056],
                    [-arm_l - 0.02, 0.056],
                    [-arm_l + 0.005, 0.03],
                ],
                lo,
                look.skin,
            ));
            p
        };
        let mut head = block(HW, HH, HD, HC, 0.0, BlockOpts::default(), look.skin);
        if look.ears {
            for s in [-1.0, 1.0] {
                head.extend(block(
                    0.05,
                    0.14,
                    0.1,
                    0.0,
                    0.2,
                    BlockOpts {
                        x: s * (HW / 2.0 + 0.02),
                        ..default_block()
                    },
                    skin_s,
                ));
            }
        }
        match look.nose {
            Nose::Button => head.extend(lathe(
                &[[0.14, 0.0], [0.152, 0.032], [0.188, 0.032], [0.2, 0.0]],
                LatheOpts {
                    cz: HA + 0.012,
                    ..lo
                },
                skin_s,
            )),
            Nose::Long => head.extend(lathe(
                &[[0.138, 0.0], [0.152, 0.028], [0.192, 0.028], [0.206, 0.0]],
                LatheOpts {
                    sz: 2.6,
                    cz: HA + 0.046,
                    ..lo
                },
                skin_s,
            )),
            Nose::None => {}
        }
        let hair = look.hair_colour;
        let hair_s = shade(hair, 0.78);
        let at = |x: f32, z: f32| BlockOpts {
            x,
            z,
            ..default_block()
        };
        let back = || {
            block(
                HW + 0.03,
                0.4,
                0.07,
                0.02,
                0.12,
                at(0.0, -HD / 2.0 - 0.012),
                hair,
            )
        };
        let crop = |head: &mut Part| {
            head.extend(block(
                HW + 0.03,
                HH + 0.04 - HAIR_Y,
                HD + 0.03,
                0.04,
                HAIR_Y,
                BlockOpts {
                    c_bot: Some(0.0),
                    ..default_block()
                },
                hair,
            ));
            head.extend(back());
            for s in [-1.0, 1.0] {
                head.extend(block(
                    0.025,
                    0.16,
                    0.2,
                    0.0,
                    0.34,
                    at(s * (HW / 2.0 + 0.01), -0.08),
                    hair,
                ));
            }
        };
        match look.hair {
            Hair::Crop => crop(&mut head),
            Hair::Bun => {
                crop(&mut head);
                head.extend(block(
                    0.18,
                    0.13,
                    0.16,
                    0.035,
                    HH + 0.02,
                    at(0.0, -0.07),
                    hair_s,
                ));
            }
            Hair::Braid => {
                crop(&mut head);
                for i in 0..3 {
                    head.extend(block(
                        0.085,
                        0.1,
                        0.075,
                        0.02,
                        0.1 - i as f32 * 0.1,
                        at(0.0, -HD / 2.0 - 0.04 - i as f32 * 0.008),
                        if i % 2 == 1 { hair_s } else { hair },
                    ));
                }
            }
            Hair::Bald => {
                for s in [-1.0, 1.0] {
                    head.extend(block(
                        0.03,
                        0.12,
                        0.17,
                        0.01,
                        0.35,
                        at(s * (HW / 2.0 + 0.01), -0.1),
                        hair,
                    ));
                }
            }
            Hair::Hat => {
                let (hc, hcs) = (look.hat, shade(look.hat, 0.74));
                let flat = BlockOpts {
                    c_top: Some(0.0),
                    c_bot: Some(0.0),
                    ..default_block()
                };
                head.extend(block(
                    HW + 0.24,
                    0.03,
                    HD + 0.24,
                    0.08,
                    HAIR_Y + 0.03,
                    flat,
                    hcs,
                ));
                head.extend(block(
                    HW + 0.05,
                    0.045,
                    HD + 0.05,
                    0.05,
                    HAIR_Y + 0.06,
                    flat,
                    hcs,
                ));
                head.extend(block(
                    HW + 0.03,
                    0.2,
                    HD + 0.03,
                    0.05,
                    HAIR_Y + 0.06,
                    BlockOpts {
                        c_bot: Some(0.0),
                        ..default_block()
                    },
                    hc,
                ));
                head.extend(back());
            }
        }
        head.extend(beard(look.beard, hair));
        Figure {
            leg_len,
            torso_h,
            body,
            legs: [leg(), leg()],
            leg_at: [-1.0f32, 1.0].map(|s| Vec3::new(s * 0.085 * w, 0.03, 0.0)),
            arms: [arm(), arm()],
            arm_at: [-1.0f32, 1.0].map(|s| Vec3::new(s * sh_x, sh_y, 0.0)),
            arm_len: arm_l,
            head_at: Vec3::new(0.0, torso_h - 0.035, 0.0),
            head_scale: hs,
            head,
            skin: look.skin,
            brow: shade(hair, 0.62),
            blush: super::look::BLUSH_LEVELS.map(|b| super::look::blush(look.skin, b)),
        }
    }

    /// A guard's spear (the towns mockup's 2.3 m shaft) through the right
    /// hand, its foot just off the ground.
    pub fn with_spear(&mut self) {
        let hand = -self.arm_len - 0.05;
        self.arms[1].extend(lathe(
            &[[hand - 0.5, 0.024], [hand + 1.8, 0.024]],
            LatheOpts::default(),
            SPEAR,
        ));
    }

    /// Feet to the top of the head at rest, m.
    pub fn height_m(&self) -> f32 {
        self.leg_len + self.head_at.y + HH * self.head_scale
    }

    /// Triangles in the body, limbs and head (the face's pieces apart).
    pub fn triangles(&self) -> usize {
        self.body.triangles()
            + self.legs.iter().map(Part::triangles).sum::<usize>()
            + self.arms.iter().map(Part::triangles).sum::<usize>()
            + self.head.triangles()
    }

    /// Every colour its parts wear.
    pub fn colours(&self) -> Vec<u32> {
        let mut out: Vec<u32> = std::iter::once(&self.body)
            .chain(&self.legs)
            .chain(&self.arms)
            .chain(std::iter::once(&self.head))
            .flat_map(|p| p.colours.iter().copied())
            .chain([self.skin, self.brow])
            .chain(self.blush)
            .chain(FACE_COLOURS)
            .collect();
        out.sort_unstable();
        out.dedup();
        out
    }
}

/// A beard's pieces as `(w, h, d, bevel, y0, x)`, each standing on the
/// head's flat front (decision 36, the mockup's `BEARDS`): its back 5 mm into
/// the head, its front `d - 0.005` proud of it, nothing on the sides or the
/// back, and nothing between [`MOUTH_CLEAR`]'s heights within its reach.
pub const MOUSTACHE: [f32; 6] = [0.2, 0.035, 0.03, 0.01, 0.15, 0.0];
pub const GOATEE: [[f32; 6]; 2] = [MOUSTACHE, [0.09, 0.095, 0.03, 0.01, -0.04, 0.0]];
pub const FULL_FRONT: [[f32; 6]; 4] = [
    MOUSTACHE,
    [0.4, 0.115, 0.035, 0.012, -0.06, 0.0],
    [0.07, 0.08, 0.035, 0.012, 0.05, -0.165],
    [0.07, 0.08, 0.035, 0.012, 0.05, 0.165],
];
/// How far a beard piece sinks into the head behind its face.
pub const BEARD_SINK: f32 = 0.005;
/// The mouth's opening, as low, high and half wide as any mouth cell draws
/// (with the thinking face's shift): a beard keeps out of it.
pub const MOUTH_CLEAR: [f32; 3] = [0.058, 0.148, 0.105];

/// The pieces of a beard, in the head's space.
pub fn beard_pieces(b: Beard) -> &'static [[f32; 6]] {
    match b {
        Beard::None => &[],
        Beard::Goatee => &GOATEE,
        Beard::FullFront => &FULL_FRONT,
    }
}

fn beard(b: Beard, colour: u32) -> Part {
    let mut p = Part::default();
    for &[w, h, d, c, y0, x] in beard_pieces(b) {
        p.extend(block(
            w,
            h,
            d,
            c,
            y0,
            BlockOpts {
                x,
                z: HA - BEARD_SINK + d / 2.0,
                ..default_block()
            },
            colour,
        ));
    }
    p
}

fn default_block() -> BlockOpts {
    BlockOpts::default()
}

/// The triangles the face's pieces add while every one is shown (the
/// mockup's count: whites, pupils, catch-lights and blush six each a side,
/// lids, lash lines and brows two each, the mouth two).
pub const FACE_TRIANGLES: usize = 2 * (6 + 6 + 6 + 6 + 2 + 2 + 2) + 2;
