//! The herd's bodies, built from boxes as M26's Farmstead and Hatchery
//! animals are (`animal-husbandry` design 17, "The owner's pick";
//! `land-transport` designs 11 to 14 for the carts' animal and the ridden
//! one): one family of models.
//!
//! Every animal is a few boxes sized and placed as one creature, each box
//! wearing a tile of its species' hide strip (`herd_hide.png`, drawn by
//! `tools/art/herd_hide.py` from M26's own hides) laid by the metre on every
//! face, as the mockup's hide shader lays it: 0.9 tiles a metre on the body,
//! 1.3 on every other part. The blocky silhouette is the owner's choice
//! ("make animals like this! I love this look"); detail is the hide's
//! pixels, never a part stuck on. A genome's coat tints the hide.
//!
//! The neck is a pose matter (design 17): standing, the head is carried at
//! about the withers; grazing, the neck lowers until the muzzle is at the
//! grass and the face turns down to at most 75 degrees, never past vertical.
//!
//! The boxes are laid out in the mockup's frame (+X forward, +Y up, +Z to
//! the side) and written in the carts' model space: +X right, +Y up, -Z
//! forward, the feet on the ground at the origin. `husbandry::rig::build`
//! turns that into rig space.

use super::rig::Builder;
use bevy::prelude::*;
use pbd_core::husbandry::Sex;

/// How an animal stands this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stance {
    /// The gait's phase, rad: ground distance over its stride.
    pub phase: f32,
    /// How far its legs swing, 0 standing to 1 a full walk.
    pub walk: f32,
    /// Its head down to graze or peck, rad (1.85 a full graze).
    pub head_down: f32,
    /// Its head turned, rad left of ahead.
    pub glance: f32,
    /// A bird's head forward and back with its steps, m at size 1.
    pub bob: f32,
    /// Its tail's lift (a bird) or swish (a beast), rad.
    pub tail: f32,
    /// Its weight shifted side to side, rad of roll.
    pub shift: f32,
    /// 0 standing to 1 lying down or roosting.
    pub down: f32,
    /// A rider on its back (`land-transport` design 14): a blanket over the
    /// back where the rider sits.
    pub saddled: bool,
}

/// An animal's colours, sRGB: its body, its pale accent (a belly, a belt),
/// its legs and points, and its head; and `tint`, its coat against its
/// species' own, which multiplies the hide.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coat {
    pub body: [f32; 3],
    pub pale: [f32; 3],
    pub legs: [f32; 3],
    pub head: [f32; 3],
    /// A pattern from the genome: 0 none.
    pub pattern: usize,
    /// The coat's tint on the hide's body, head and mane: 1 the species'
    /// own colour.
    pub tint: [f32; 3],
}

impl Default for Coat {
    /// A bay horse.
    fn default() -> Self {
        Self {
            body: [0.45, 0.28, 0.16],
            pale: [0.62, 0.46, 0.32],
            legs: [0.16, 0.11, 0.08],
            head: [0.40, 0.25, 0.15],
            pattern: 0,
            tint: [1.0; 3],
        }
    }
}

/// The four-legged bodies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Beast {
    Horse,
    Plodder,
    Woolback,
    // `survival-mode` design 17 (task 4.11): the game, M26's `KINDS` where
    // M26 has them (the ridge goat, the red fox, the rootling; the dust
    // hound on its herd hound), the hare in the same family.
    RidgeGoat,
    RedFox,
    Rootling,
    Hare,
    DustHound,
}

impl Beast {
    /// Its height at the withers (the top of its body) at size 1, m.
    pub fn withers_m(self) -> f32 {
        match self {
            Beast::Horse => 1.55,
            Beast::Plodder => 1.35,
            Beast::Woolback => 0.8,
            Beast::RidgeGoat => 0.75,
            Beast::RedFox => 0.42,
            Beast::Rootling => 0.72,
            Beast::Hare => 0.4,
            Beast::DustHound => 0.55,
        }
    }

    /// Every beast, in roster order.
    pub const ALL: [Beast; 8] = [
        Beast::Horse,
        Beast::Plodder,
        Beast::Woolback,
        Beast::RidgeGoat,
        Beast::RedFox,
        Beast::Rootling,
        Beast::Hare,
        Beast::DustHound,
    ];

    /// The form a species id is drawn by.
    pub fn of_id(id: &str) -> Option<Beast> {
        match id {
            "horse" => Some(Beast::Horse),
            "plodder" => Some(Beast::Plodder),
            "woolback" => Some(Beast::Woolback),
            "ridge-goat" => Some(Beast::RidgeGoat),
            "red-fox" => Some(Beast::RedFox),
            "rootling" => Some(Beast::Rootling),
            "hare" => Some(Beast::Hare),
            "dust-hound" => Some(Beast::DustHound),
            _ => None,
        }
    }
}

/// Tiles of a hide strip (M26's layout).
mod tile {
    pub const BODY: u8 = 0;
    pub const LEGS: u8 = 1;
    pub const HEAD: u8 = 2;
    pub const HORN: u8 = 3;
    pub const TAIL: u8 = 4;
    pub const MANE: u8 = 5;
    pub const SNOUT: u8 = 6;
    pub const EARS: u8 = 7;
}

/// Rows of `herd_hide.png`, one species' strip each.
const ROW_HORSE: u8 = 0;
const ROW_PLODDER: u8 = 1;
const ROW_WOOLBACK: u8 = 2;
const ROW_HEN: u8 = 3;
// `survival-mode` design 17: the game's hides, appended rows.
const ROW_RIDGE_GOAT: u8 = 4;
const ROW_RED_FOX: u8 = 5;
const ROW_ROOTLING: u8 = 6;
const ROW_HARE: u8 = 7;
const ROW_DUST_HOUND: u8 = 8;
const ROWS: f32 = 9.0;

#[derive(Clone, Copy, PartialEq)]
enum Ears {
    Flop,
    Point,
    /// A hare's: long, upright and laid back a little.
    Long,
}

#[derive(Clone, Copy, PartialEq)]
enum Tail {
    Stub,
    Tuft,
    /// A fox's or a hound's brush, its tip in the mane tile.
    Bush,
    /// A pig's: thin and short, no tuft.
    Thin,
}

#[derive(Clone, Copy, PartialEq)]
enum Horns {
    None,
    /// The plodder's: straight up and out.
    Spike,
    /// The ridge goat's: swept back and down.
    Curl,
}

/// A beast's proportions (M26's `KINDS`), m at the mockup's size 1: body
/// length and height, leg length, neck and head sizes, snout length
/// (0 none); and the game's carriage: the standing neck angle above level
/// (rad, M26's own but the horse's, carried lower), and how far the neck
/// stretches as it lowers to graze, against M26's length, so the muzzle
/// reaches the grass (design 17): standing, it is M26's neck.
struct Form {
    l: f32,
    h: f32,
    leg: f32,
    neck: f32,
    head: f32,
    snout: f32,
    horns: Horns,
    ears: Ears,
    tail: Tail,
    wool: bool,
    mane: bool,
    row: u8,
    carry: f32,
    neck_k: f32,
}

const HORSE: Form = Form {
    l: 1.9,
    h: 0.8,
    leg: 1.15,
    neck: 0.75,
    head: 0.32,
    snout: 1.3,
    horns: Horns::None,
    ears: Ears::Point,
    tail: Tail::Tuft,
    wool: false,
    mane: true,
    row: ROW_HORSE,
    carry: 0.65,
    neck_k: 1.15,
};

const PLODDER: Form = Form {
    l: 2.0,
    h: 1.0,
    leg: 0.8,
    neck: 0.2,
    head: 0.38,
    snout: 0.6,
    horns: Horns::Spike,
    ears: Ears::Flop,
    tail: Tail::Tuft,
    wool: false,
    mane: false,
    row: ROW_PLODDER,
    carry: 0.55,
    neck_k: 2.1,
};

const WOOLBACK: Form = Form {
    l: 1.15,
    h: 0.75,
    leg: 0.42,
    neck: 0.12,
    head: 0.3,
    snout: 0.0,
    horns: Horns::None,
    ears: Ears::Flop,
    tail: Tail::Stub,
    wool: true,
    mane: false,
    row: ROW_WOOLBACK,
    carry: 0.55,
    neck_k: 1.9,
};

// The game (`survival-mode` design 17): M26's `KINDS` numbers.
const RIDGE_GOAT: Form = Form {
    l: 1.1,
    h: 0.6,
    leg: 0.55,
    neck: 0.25,
    head: 0.28,
    snout: 0.6,
    horns: Horns::Curl,
    ears: Ears::Point,
    tail: Tail::Stub,
    wool: false,
    mane: false,
    row: ROW_RIDGE_GOAT,
    carry: 0.55,
    neck_k: 1.6,
};

const RED_FOX: Form = Form {
    l: 0.9,
    h: 0.42,
    leg: 0.38,
    neck: 0.14,
    head: 0.24,
    snout: 1.0,
    horns: Horns::None,
    ears: Ears::Point,
    tail: Tail::Bush,
    wool: false,
    mane: false,
    row: ROW_RED_FOX,
    carry: 0.55,
    neck_k: 1.9,
};

const ROOTLING: Form = Form {
    l: 1.25,
    h: 0.72,
    leg: 0.3,
    neck: 0.04,
    head: 0.38,
    snout: 0.7,
    horns: Horns::None,
    ears: Ears::Flop,
    tail: Tail::Thin,
    wool: false,
    mane: false,
    row: ROW_ROOTLING,
    carry: 0.55,
    neck_k: 2.2,
};

/// The hare: not in M26, drawn in its family: a short body on short legs,
/// a small head with a short snout, long ears and a puff of a tail.
const HARE: Form = Form {
    l: 0.55,
    h: 0.3,
    leg: 0.18,
    neck: 0.05,
    head: 0.15,
    snout: 0.4,
    horns: Horns::None,
    ears: Ears::Long,
    tail: Tail::Stub,
    wool: false,
    mane: false,
    row: ROW_HARE,
    carry: 0.35,
    neck_k: 2.0,
};

/// The dust hound: M26's herd hound, in its own sand-coloured hide.
const DUST_HOUND: Form = Form {
    l: 0.95,
    h: 0.42,
    leg: 0.45,
    neck: 0.15,
    head: 0.26,
    snout: 1.0,
    horns: Horns::None,
    ears: Ears::Point,
    tail: Tail::Bush,
    wool: false,
    mane: false,
    row: ROW_DUST_HOUND,
    carry: 0.55,
    neck_k: 1.9,
};

fn form(b: Beast) -> &'static Form {
    match b {
        Beast::Horse => &HORSE,
        Beast::Plodder => &PLODDER,
        Beast::Woolback => &WOOLBACK,
        Beast::RidgeGoat => &RIDGE_GOAT,
        Beast::RedFox => &RED_FOX,
        Beast::Rootling => &ROOTLING,
        Beast::Hare => &HARE,
        Beast::DustHound => &DUST_HOUND,
    }
}

/// The face's angle at a full graze: 75 degrees below level, never past
/// vertical.
pub const GRAZE_FACE: f32 = -75.0 * std::f32::consts::PI / 180.0;
/// The lowest the neck goes, rad below level.
const NECK_LOWEST: f32 = -100.0 * std::f32::consts::PI / 180.0;
/// How far above the grass a grazing muzzle stops, m.
const GRAZE_CLEAR: f32 = 0.03;
/// The head_down of a full graze (the herd's).
const FULL_GRAZE: f32 = 1.85;

/// A place and a turn in the mockup's frame.
#[derive(Clone, Copy)]
struct Frame {
    o: Vec3,
    q: Quat,
}

impl Frame {
    const ROOT: Frame = Frame {
        o: Vec3::ZERO,
        q: Quat::IDENTITY,
    };
    fn at(&self, p: Vec3) -> Vec3 {
        self.o + self.q * p
    }
    fn child(&self, offset: Vec3, turn: Quat) -> Frame {
        Frame {
            o: self.at(offset),
            q: self.q * turn,
        }
    }
}

/// The mockup's frame (+X forward, +Z side) to model space (-Z forward).
fn out_of(p: Vec3) -> Vec3 {
    Vec3::new(p.z, p.y, -p.x)
}

/// sRGB to the linear vertex colour the builder writes.
fn lin(c: [f32; 3]) -> [f32; 4] {
    let [r, g, b, _] = Color::srgb(c[0], c[1], c[2]).to_linear().to_f32_array();
    [r, g, b, 1.0]
}

/// The boxes of one animal: its hide's row and its tint.
struct Hide<'a> {
    out: &'a mut Builder,
    row: u8,
}

impl Hide<'_> {
    /// A box of size `dims` (x, y, z in the frame), centred at `c` in `f`
    /// and turned by `turn` about its centre, wearing hide tile `cell`
    /// tinted by `tint` (linear multiplier, 1 the hide's own).
    fn bx(&mut self, f: &Frame, cell: u8, dims: Vec3, c: Vec3, turn: Quat, tint: [f32; 3]) {
        let b = f.child(c, turn);
        let rep = if cell == tile::BODY { 0.9 } else { 1.3 };
        let colour = [tint[0], tint[1], tint[2], 1.0];
        let half = dims * 0.5;
        // Each face: its axis, and the two across it in the mockup's box
        // UV order (three.js BoxGeometry: +-x faces (z, y), +-y (x, z),
        // +-z (x, y)).
        for (axis, a, b2) in [(0usize, 2usize, 1usize), (1, 0, 2), (2, 0, 1)] {
            for sign in [1.0f32, -1.0] {
                let mut n = Vec3::ZERO;
                n[axis] = sign;
                let (la, lb) = (dims[a], dims[b2]);
                // Split the face where the tile repeats, so each piece's UV
                // lies inside one tile of the strip.
                let cuts = |len: f32| {
                    let tiles = len * rep;
                    let mut at = vec![0.0f32];
                    let mut k = 1.0;
                    while k < tiles - 1e-3 {
                        at.push(k / rep);
                        k += 1.0;
                    }
                    at.push(len);
                    at
                };
                let (ca, cb) = (cuts(la), cuts(lb));
                for i in 0..ca.len() - 1 {
                    for j in 0..cb.len() - 1 {
                        let (a0, a1, b0, b1) = (ca[i], ca[i + 1], cb[j], cb[j + 1]);
                        let corner = |sa: f32, sb: f32| {
                            let mut p = Vec3::ZERO;
                            p[axis] = sign * half[axis];
                            p[a] = sa - half[a];
                            p[b2] = sb - half[b2];
                            p
                        };
                        // The tile's fraction across this piece.
                        let ta = (a0 * rep).floor();
                        let tb = (b0 * rep).floor();
                        let uv = |sa: f32, sb: f32| {
                            let fu = (sa * rep - ta).clamp(0.0, 1.0);
                            let fv = (sb * rep - tb).clamp(0.0, 1.0);
                            [
                                (cell as f32 * 32.0 + 0.5 + fu * 31.0) / 256.0,
                                (self.row as f32 * 32.0 + 0.5 + (1.0 - fv) * 31.0) / (32.0 * ROWS),
                            ]
                        };
                        let quad = [(a0, b0), (a1, b0), (a1, b1), (a0, b1)];
                        let pts = quad.map(|(sa, sb)| b.at(corner(sa, sb)));
                        let normal = (b.q * n).normalize();
                        let face = (pts[1] - pts[0]).cross(pts[2] - pts[0]);
                        let order: [usize; 4] = if face.dot(normal) >= 0.0 {
                            [0, 1, 2, 3]
                        } else {
                            [0, 3, 2, 1]
                        };
                        let base = self.out.positions.len() as u32;
                        for k in order {
                            self.out.positions.push(out_of(pts[k]).to_array());
                            self.out.normals.push(out_of(normal).to_array());
                            self.out.colours.push(colour);
                            self.out.uvs.push(uv(quad[k].0, quad[k].1));
                        }
                        self.out.indices.extend_from_slice(&[
                            base,
                            base + 1,
                            base + 2,
                            base,
                            base + 2,
                            base + 3,
                        ]);
                    }
                }
            }
        }
        self.out.textured = true;
    }
}

/// A coat's tint, linear, on the hide's own colours.
fn tints(c: &Coat) -> [f32; 3] {
    let [r, g, b, _] = lin(c.tint.map(|x| x.clamp(0.0, 4.0)));
    [r, g, b]
}

const WHITE: [f32; 3] = [1.0; 3];
const EYE: [f32; 3] = [0.12, 0.11, 0.1];

/// Where a beast's neck goes and how its head turns, rad above level: the
/// neck's angle, the face's, and how far into a graze it is (0 to 1).
fn carriage(f: &Form, k: f32, pose: &Stance) -> (f32, f32, f32) {
    let walk = pose.walk.clamp(0.0, 1.0);
    let down = pose.down.clamp(0.0, 1.0);
    let nod = (pose.phase * 2.0 + 0.6).sin() * 0.05 * walk;
    let g = (pose.head_down / FULL_GRAZE).clamp(0.0, 1.0) * (1.0 - 0.6 * down);
    if g <= 0.0 {
        return (f.carry + nod, 0.0, 0.0);
    }
    // The full graze: the neck lowered until the muzzle is at the grass.
    let reach = |neck: f32| lowest_of_head(f, k, neck, GRAZE_FACE);
    let (mut lo, mut hi) = (NECK_LOWEST, f.carry);
    if reach(lo) > GRAZE_CLEAR {
        hi = lo;
    } else {
        for _ in 0..32 {
            let m = 0.5 * (lo + hi);
            if reach(m) > GRAZE_CLEAR {
                hi = m;
            } else {
                lo = m;
            }
        }
    }
    let graze = hi;
    (
        f.carry + (graze - f.carry) * g + nod,
        (GRAZE_FACE * g).max(GRAZE_FACE),
        g,
    )
}

/// The neck's root, its length, the head's size and the snout's box, at
/// scale `k`.
struct Head {
    root: Vec3,
    nl: f32,
    hr: f32,
    snout: Option<(Vec3, Vec3)>,
}

/// `g` is how far into a graze: the neck stretches toward the grass.
fn head_of(f: &Form, k: f32, g: f32) -> Head {
    let (l, h, leg) = (f.l * k, f.h * k, f.leg * k);
    let hr = f.head * k;
    Head {
        root: Vec3::new(l * 0.44, leg + h * 0.72, 0.0),
        nl: (f.neck * k + h * 0.3) * (1.0 + (f.neck_k - 1.0) * g),
        hr,
        snout: (f.snout > 0.0).then(|| {
            (
                Vec3::new(hr * 0.55 * f.snout + 0.02 * k, hr * 0.56, hr * 0.62),
                Vec3::new(hr * 0.92 + hr * 0.27 * f.snout, -hr * 0.16, 0.0),
            )
        }),
    }
}

/// The lowest point of the head and snout with the neck at `neck`, fully
/// stretched to graze, and the face at `face` (rad above level), standing.
fn lowest_of_head(f: &Form, k: f32, neck: f32, face: f32) -> f32 {
    let hd = head_of(f, k, 1.0);
    let nk = Frame::ROOT.child(hd.root, Quat::from_rotation_z(neck));
    let head = nk.child(Vec3::X * hd.nl, Quat::from_rotation_z(face - neck));
    let mut boxes = vec![(
        Vec3::new(hd.hr * 1.25, hd.hr, hd.hr * 0.9),
        Vec3::X * (hd.hr * 0.3),
    )];
    boxes.extend(hd.snout);
    let mut low = f32::MAX;
    for (d, c) in boxes {
        for sx in [-0.5f32, 0.5] {
            for sy in [-0.5f32, 0.5] {
                low = low.min(head.at(c + Vec3::new(d.x * sx, d.y * sy, 0.0)).y);
            }
        }
    }
    low
}

/// Where a beast's muzzle (its snout's, or its head's, front lower edge)
/// and its face's direction are at a stance, in model space: for the tests
/// and the records.
pub fn muzzle(kind: Beast, pose: &Stance, size: f32) -> (Vec3, Vec3) {
    let f = form(kind);
    let k = size * kind.withers_m() / (f.leg + f.h);
    let (neck, face, g) = carriage(f, k, pose);
    let hd = head_of(f, k, g);
    let nk = Frame::ROOT.child(hd.root, Quat::from_rotation_z(neck));
    let head = nk.child(Vec3::X * hd.nl, Quat::from_rotation_z(face - neck));
    let tip = match hd.snout {
        Some((d, c)) => c + Vec3::new(d.x * 0.5, -d.y * 0.5, 0.0),
        None => Vec3::new(hd.hr * 0.925, -hd.hr * 0.5, 0.0),
    };
    (out_of(head.at(tip)), out_of(head.q * Vec3::X))
}

/// A four-legged animal at its stance, scaled by `size` (1 its species'
/// own withers height).
pub fn beast(out: &mut Builder, kind: Beast, c: &Coat, pose: &Stance, size: f32) {
    let f = form(kind);
    let k = size * kind.withers_m() / (f.leg + f.h);
    let (l, h, leg) = (f.l * k, f.h * k, f.leg * k);
    let w = h * 0.82;
    let walk = pose.walk.clamp(0.0, 1.0);
    let down = pose.down.clamp(0.0, 1.0);
    let tint = tints(c);
    let mut hide = Hide { out, row: f.row };
    // The body: lying, it sinks onto its folded legs; its weight shifts as
    // a roll about its length at the hips' height.
    let body = Frame::ROOT.child(
        Vec3::new(0.0, leg - leg * 0.85 * down, 0.0),
        Quat::from_rotation_x(pose.shift),
    );
    let body = body.child(Vec3::NEG_Y * leg, Quat::IDENTITY);
    // Legs: a pivot at each hip, swung in diagonal pairs, folded under to
    // lie.
    let swing = 0.5 * walk;
    let lw = (h * 0.2).max(0.05);
    for (i, (fx, s)) in [
        (l * 0.33, -1.0f32),
        (l * 0.33, 1.0),
        (-l * 0.33, -1.0),
        (-l * 0.33, 1.0),
    ]
    .into_iter()
    .enumerate()
    {
        let diagonal = if i == 0 || i == 3 {
            0.0
        } else {
            std::f32::consts::PI
        };
        let fold = if i < 2 { -1.35 } else { 1.35 } * down;
        let a = (pose.phase + diagonal).sin() * swing * (1.0 - down) + fold;
        let piv = body.child(Vec3::new(fx, leg, s * w * 0.3), Quat::from_rotation_z(a));
        hide.bx(
            &piv,
            tile::LEGS,
            Vec3::new(lw, leg, lw),
            Vec3::NEG_Y * (leg * 0.5),
            Quat::IDENTITY,
            WHITE,
        );
    }
    hide.bx(
        &body,
        tile::BODY,
        Vec3::new(l, h, w),
        Vec3::new(0.0, leg + h * 0.5, 0.0),
        Quat::IDENTITY,
        tint,
    );
    if f.wool {
        hide.bx(
            &body,
            tile::BODY,
            Vec3::new(l * 1.08, h * 0.55, w * 1.14),
            Vec3::new(0.0, leg + h * 0.68, 0.0),
            Quat::IDENTITY,
            tint,
        );
        hide.bx(
            &body,
            tile::BODY,
            Vec3::new(l * 0.4, h * 0.3, w * 0.9),
            Vec3::new(l * 0.1, leg + h * 1.05, 0.0),
            Quat::IDENTITY,
            tint,
        );
    }
    // A belt round the middle from the genome's pattern.
    if c.pattern > 0 && kind != Beast::Horse {
        let belt = [
            c.pale[0] / c.body[0].max(0.05),
            c.pale[1] / c.body[1].max(0.05),
            c.pale[2] / c.body[2].max(0.05),
        ];
        let [r, g, b, _] = lin(belt.map(|x| x.clamp(0.0, 3.0)));
        let grow = if f.wool { 1.16 } else { 1.04 };
        hide.bx(
            &body,
            tile::BODY,
            Vec3::new(l * 0.22, h * 1.04, w * grow),
            Vec3::new(0.0, leg + h * 0.5, 0.0),
            Quat::IDENTITY,
            [tint[0] * r, tint[1] * g, tint[2] * b],
        );
    }
    // A blanket where a rider sits.
    if pose.saddled {
        hide.bx(
            &body,
            tile::EARS,
            Vec3::new(l * 0.34, h * 0.08, w * 1.08),
            Vec3::new(0.0, leg + h * 1.0, 0.0),
            Quat::IDENTITY,
            [0.9, 0.12, 0.08],
        );
    }
    // The neck from the chest, the head at its end (design 17's carriage).
    let (neck, face, g) = carriage(f, k, pose);
    let hd = head_of(f, k, g);
    let nk = body.child(
        hd.root,
        Quat::from_rotation_y(pose.glance) * Quat::from_rotation_z(neck),
    );
    hide.bx(
        &nk,
        tile::BODY,
        Vec3::new(hd.nl, h * 0.36, w * 0.42),
        Vec3::X * (hd.nl * 0.5),
        Quat::IDENTITY,
        tint,
    );
    if f.mane {
        hide.bx(
            &nk,
            tile::MANE,
            Vec3::new(hd.nl * 0.95, h * 0.14, w * 0.18),
            Vec3::new(hd.nl * 0.5, h * 0.22, 0.0),
            Quat::IDENTITY,
            tint,
        );
    }
    let hr = hd.hr;
    let head = nk.child(Vec3::X * hd.nl, Quat::from_rotation_z(face - neck));
    hide.bx(
        &head,
        tile::HEAD,
        Vec3::new(hr * 1.25, hr, hr * 0.9),
        Vec3::X * (hr * 0.3),
        Quat::IDENTITY,
        tint,
    );
    if let Some((d, centre)) = hd.snout {
        hide.bx(&head, tile::SNOUT, d, centre, Quat::IDENTITY, WHITE);
    }
    for s in [-1.0f32, 1.0] {
        match f.ears {
            Ears::Flop => hide.bx(
                &head,
                tile::EARS,
                Vec3::new(hr * 0.32, hr * 0.12, hr * 0.42),
                Vec3::new(hr * 0.05, hr * 0.32, s * hr * 0.6),
                Quat::from_rotation_x(s * 0.7),
                WHITE,
            ),
            Ears::Point => hide.bx(
                &head,
                tile::EARS,
                Vec3::new(hr * 0.22, hr * 0.48, hr * 0.18),
                Vec3::new(0.0, hr * 0.68, s * hr * 0.28),
                Quat::IDENTITY,
                WHITE,
            ),
            Ears::Long => hide.bx(
                &head,
                tile::EARS,
                Vec3::new(hr * 0.26, hr * 0.85, hr * 0.14),
                Vec3::new(-hr * 0.2, hr * 0.75, s * hr * 0.24),
                Quat::from_rotation_z(0.3) * Quat::from_rotation_x(s * 0.12),
                WHITE,
            ),
        }
        match f.horns {
            Horns::None => {}
            Horns::Spike => hide.bx(
                &head,
                tile::HORN,
                Vec3::new(hr * 0.14, hr * 0.62, hr * 0.14),
                Vec3::new(-hr * 0.05, hr * 0.78, s * hr * 0.36),
                Quat::from_rotation_x(-s * 0.45),
                WHITE,
            ),
            // M26's curl: back along the crown, then down behind it.
            Horns::Curl => {
                hide.bx(
                    &head,
                    tile::HORN,
                    Vec3::new(hr * 0.6, hr * 0.2, hr * 0.2),
                    Vec3::new(-hr * 0.3, hr * 0.6, s * hr * 0.36),
                    Quat::IDENTITY,
                    WHITE,
                );
                hide.bx(
                    &head,
                    tile::HORN,
                    Vec3::new(hr * 0.2, hr * 0.55, hr * 0.2),
                    Vec3::new(-hr * 0.6, hr * 0.32, s * hr * 0.4),
                    Quat::IDENTITY,
                    WHITE,
                );
            }
        }
        hide.bx(
            &head,
            tile::LEGS,
            Vec3::new(hr * 0.16, hr * 0.16, hr * 0.06),
            Vec3::new(hr * 0.62, hr * 0.18, s * hr * 0.46),
            Quat::IDENTITY,
            EYE,
        );
    }
    // The tail from the rump, swishing about its root.
    let swish = pose.tail + pose.phase.sin() * 0.08 * walk;
    let rump = body.child(
        Vec3::new(-l * 0.5, leg + h * 0.8, 0.0),
        Quat::from_rotation_y(swish),
    );
    match f.tail {
        Tail::Stub => hide.bx(
            &rump,
            tile::TAIL,
            Vec3::splat(h * 0.26),
            Vec3::new(-l * 0.03, -h * 0.02, 0.0),
            Quat::IDENTITY,
            tint,
        ),
        // M26's brush: down and back from the rump, its tip the mane tile.
        Tail::Bush => {
            hide.bx(
                &rump,
                tile::TAIL,
                Vec3::new(l * 0.5, h * 0.32, h * 0.32),
                Vec3::new(-l * 0.16, -h * 0.35, 0.0),
                Quat::from_rotation_z(0.45),
                tint,
            );
            hide.bx(
                &rump,
                tile::MANE,
                Vec3::new(l * 0.16, h * 0.3, h * 0.3),
                Vec3::new(-l * 0.42, -h * 0.58, 0.0),
                Quat::IDENTITY,
                WHITE,
            );
        }
        Tail::Thin => hide.bx(
            &rump,
            tile::TAIL,
            Vec3::new(l * 0.36, h * 0.12, h * 0.12),
            Vec3::new(-l * 0.1, -h * 0.25, 0.0),
            Quat::from_rotation_z(1.0),
            tint,
        ),
        Tail::Tuft => {
            hide.bx(
                &rump,
                tile::TAIL,
                Vec3::new(l * 0.36, h * 0.12, h * 0.12),
                Vec3::new(-l * 0.1, -h * 0.25, 0.0),
                Quat::from_rotation_z(1.0),
                tint,
            );
            hide.bx(
                &rump,
                tile::MANE,
                Vec3::new(h * 0.2, h * 0.3, h * 0.2),
                Vec3::new(-l * 0.2, -h * 0.58, 0.0),
                Quat::IDENTITY,
                tint,
            );
        }
    }
}

/// A bird (the meadow hen) at its stance, scaled by `size` (1 a grown hen:
/// 0.42 m long).
pub fn bird(out: &mut Builder, c: &Coat, sex: Sex, pose: &Stance, size: f32) {
    let ll = 0.42 * size;
    let leg_h = ll * 0.42;
    let down = pose.down.clamp(0.0, 1.0);
    let tint = tints(c);
    let male = sex == Sex::M;
    let (crest, tail_up) = if male { (1.6, true) } else { (1.0, false) };
    let mut hide = Hide { out, row: ROW_HEN };
    // Roosting, she settles onto her legs folded under her.
    let body = Frame::ROOT.child(
        Vec3::new(0.0, -leg_h * 0.85 * 0.6 * down, 0.0),
        Quat::from_rotation_x(pose.shift),
    );
    if down < 0.9 {
        let swing = 0.7 * pose.walk.clamp(0.0, 1.0) * (1.0 - down);
        for (i, s) in [-1.0f32, 1.0].into_iter().enumerate() {
            let a = (pose.phase + i as f32 * std::f32::consts::PI).sin() * swing;
            let piv = body.child(
                Vec3::new(0.0, leg_h, s * ll * 0.1),
                Quat::from_rotation_z(a),
            );
            hide.bx(
                &piv,
                tile::LEGS,
                Vec3::new(ll * 0.07, leg_h, ll * 0.07),
                Vec3::NEG_Y * (leg_h * 0.5),
                Quat::IDENTITY,
                WHITE,
            );
            hide.bx(
                &piv,
                tile::LEGS,
                Vec3::new(ll * 0.22, ll * 0.04, ll * 0.14),
                Vec3::new(ll * 0.06, -leg_h + ll * 0.024, 0.0),
                Quat::IDENTITY,
                WHITE,
            );
        }
    }
    let (bl, bh) = (ll, ll * 0.58);
    hide.bx(
        &body,
        tile::BODY,
        Vec3::new(bl, bh, ll * 0.55),
        Vec3::new(0.0, leg_h + bh * 0.5, 0.0),
        Quat::IDENTITY,
        tint,
    );
    hide.bx(
        &body,
        tile::EARS,
        Vec3::new(bl * 0.6, ll * 0.14, ll * 0.5),
        Vec3::new(ll * 0.06, leg_h + ll * 0.04, 0.0),
        Quat::IDENTITY,
        WHITE,
    );
    for s in [-1.0f32, 1.0] {
        hide.bx(
            &body,
            tile::MANE,
            Vec3::new(bl * 0.62, bh * 0.6, ll * 0.06),
            Vec3::new(-ll * 0.04, leg_h + bh * 0.58, s * ll * 0.29),
            Quat::IDENTITY,
            tint,
        );
    }
    let tail_tint = if male {
        // A cockerel's tail, dark and glossed green.
        [0.25, 0.45, 0.35]
    } else {
        tint
    };
    hide.bx(
        &body,
        tile::TAIL,
        Vec3::new(ll * 0.4, ll * 0.1, ll * 0.34),
        Vec3::new(
            -bl * 0.55,
            leg_h + bh * if tail_up { 1.05 } else { 0.8 },
            0.0,
        ),
        Quat::from_rotation_z(if tail_up { 0.9 } else { 0.35 } + pose.tail),
        tail_tint,
    );
    if tail_up {
        hide.bx(
            &body,
            tile::TAIL,
            Vec3::new(ll * 0.3, ll * 0.08, ll * 0.1),
            Vec3::new(-bl * 0.6, leg_h + bh * 1.3, 0.0),
            Quat::from_rotation_z(1.3 + pose.tail),
            tail_tint,
        );
    }
    // The head on its short neck: turned to glance, dipped to peck, bobbing.
    let hp = body.child(
        Vec3::new(bl * 0.38 + pose.bob * size, leg_h + bh * 0.8, 0.0),
        Quat::from_rotation_y(pose.glance) * Quat::from_rotation_z(-pose.head_down * (1.0 - down)),
    );
    let hs = ll * 0.36;
    hide.bx(
        &hp,
        tile::HEAD,
        Vec3::new(hs, hs, hs * 0.9),
        Vec3::new(hs * 0.25, hs * 0.55, 0.0),
        Quat::IDENTITY,
        tint,
    );
    hide.bx(
        &hp,
        tile::SNOUT,
        Vec3::new(hs * 0.4, hs * 0.22, hs * 0.26),
        Vec3::new(hs * 0.62, hs * 0.5, 0.0),
        Quat::IDENTITY,
        WHITE,
    );
    hide.bx(
        &hp,
        tile::HORN,
        Vec3::new(hs * 0.55 * crest, hs * 0.3 * crest, hs * 0.12),
        Vec3::new(hs * 0.18, hs * 1.12, 0.0),
        Quat::IDENTITY,
        WHITE,
    );
    hide.bx(
        &hp,
        tile::HORN,
        Vec3::new(hs * 0.14, hs * 0.26 * crest, hs * 0.1),
        Vec3::new(hs * 0.48, hs * 0.18, 0.0),
        Quat::IDENTITY,
        WHITE,
    );
    for s in [-1.0f32, 1.0] {
        hide.bx(
            &hp,
            tile::LEGS,
            Vec3::new(hs * 0.14, hs * 0.14, hs * 0.04),
            Vec3::new(hs * 0.38, hs * 0.66, s * hs * 0.46),
            Quat::IDENTITY,
            EYE,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds(b: &Builder) -> (Vec3, Vec3) {
        b.positions
            .iter()
            .fold((Vec3::MAX, Vec3::MIN), |(lo, hi), p| {
                let p = Vec3::from(*p);
                (lo.min(p), hi.max(p))
            })
    }

    fn whole(b: &Builder) {
        assert!(!b.positions.is_empty());
        assert!(b.textured);
        assert_eq!(b.positions.len(), b.normals.len());
        assert_eq!(b.positions.len(), b.colours.len());
        assert_eq!(b.positions.len(), b.uvs.len());
        assert_eq!(b.indices.len() % 3, 0);
        assert!(b.indices.iter().all(|&i| (i as usize) < b.positions.len()));
        assert!(b.positions.iter().flatten().all(|v| v.is_finite()));
        assert!(
            b.normals
                .iter()
                .all(|n| (Vec3::from(*n).length() - 1.0).abs() < 1e-3)
        );
        // The winding agrees with the normal.
        for t in b.indices.chunks(3) {
            let [a, c, d] = [t[0], t[1], t[2]].map(|i| Vec3::from(b.positions[i as usize]));
            let n = Vec3::from(b.normals[t[0] as usize]);
            assert!((c - a).cross(d - a).dot(n) > -1e-9);
        }
    }

    /// Each beast is a whole textured mesh about its height, standing on the
    /// ground, and lying down brings it lower.
    #[test]
    fn every_beast_is_whole_and_its_size() {
        for kind in Beast::ALL {
            let mut b = Builder::default();
            beast(&mut b, kind, &Coat::default(), &Stance::default(), 1.0);
            whole(&b);
            let (lo, hi) = bounds(&b);
            assert!(lo.y > -0.05 && lo.y < 0.05, "{kind:?} feet at {}", lo.y);
            let withers = kind.withers_m();
            assert!(
                (withers * 0.95..withers * 1.6).contains(&hi.y),
                "{kind:?} {}",
                hi.y
            );
            let mut lying = Builder::default();
            beast(
                &mut lying,
                kind,
                &Coat::default(),
                &Stance {
                    down: 1.0,
                    ..default()
                },
                1.0,
            );
            // A short-legged beast (the rootling, the hare) has less to
            // fold: it sinks a fifth of its height.
            let sinks = if matches!(kind, Beast::Rootling | Beast::Hare) {
                0.2
            } else {
                0.3
            };
            assert!(
                bounds(&lying).1.y < hi.y - withers * sinks,
                "{kind:?} lies down"
            );
        }
        let mut b = Builder::default();
        bird(&mut b, &Coat::default(), Sex::M, &Stance::default(), 1.0);
        whole(&b);
        let (lo, hi) = bounds(&b);
        assert!(lo.y > -0.03 && (0.35..0.65).contains(&hi.y), "{lo} {hi}");
    }

    /// Every face wears one tile of its species' row of the hide: its UVs
    /// lie inside a single 32 px tile, never across two.
    #[test]
    fn every_face_wears_one_hide_tile() {
        let mut b = Builder::default();
        beast(
            &mut b,
            Beast::Plodder,
            &Coat::default(),
            &Stance::default(),
            1.0,
        );
        for quad in b.uvs.chunks(4) {
            let cell = |uv: &[f32; 2]| ((uv[0] * 8.0).floor(), (uv[1] * ROWS).floor());
            assert!(quad.iter().all(|uv| cell(uv) == cell(&quad[0])), "{quad:?}");
            assert_eq!(cell(&quad[0]).1, f32::from(ROW_PLODDER));
        }
    }

    /// Design 17's neck: grazing, the muzzle is at the grass and the face
    /// never turned past vertical, for every beast at every part of the
    /// graze, walking or lying; standing, the plodder's head is carried at
    /// its withers.
    #[test]
    fn the_head_grazes_at_the_grass_and_the_face_never_turns_back() {
        for kind in Beast::ALL {
            let graze = Stance {
                head_down: FULL_GRAZE,
                ..default()
            };
            let (tip, _) = muzzle(kind, &graze, 1.0);
            assert!((0.0..0.10).contains(&tip.y), "{kind:?} muzzle {}", tip.y);
            for step in 0..=20 {
                for (walk, down) in [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)] {
                    let pose = Stance {
                        head_down: FULL_GRAZE * step as f32 / 20.0,
                        walk,
                        down,
                        phase: 1.3,
                        ..default()
                    };
                    let (_, face) = muzzle(kind, &pose, 1.0);
                    // Forward is -Z: a face past vertical points back (+Z).
                    assert!(face.z <= 1e-4, "{kind:?} {step} face {face}");
                }
            }
        }
        let mut b = Builder::default();
        beast(
            &mut b,
            Beast::Plodder,
            &Coat::default(),
            &Stance::default(),
            1.0,
        );
        let (tip, _) = muzzle(Beast::Plodder, &Stance::default(), 1.0);
        let withers = Beast::Plodder.withers_m();
        assert!(
            (tip.y - withers).abs() < 0.3,
            "the plodder's muzzle {} against its withers {withers}",
            tip.y
        );
    }

    /// Grazing puts the head down, not up.
    #[test]
    fn grazing_lowers_the_head() {
        for kind in Beast::ALL {
            let y = |head_down: f32| {
                muzzle(
                    kind,
                    &Stance {
                        head_down,
                        ..default()
                    },
                    1.0,
                )
                .0
                .y
            };
            assert!(y(0.8) < y(0.0) - 0.1, "{kind:?}");
        }
    }

    /// The genome's colours tint the hide: a different coat changes the
    /// vertex colours and nothing else about the mesh.
    #[test]
    fn a_coat_tints_its_hide() {
        let mut a = Builder::default();
        let mut b = Builder::default();
        beast(
            &mut a,
            Beast::Horse,
            &Coat::default(),
            &Stance::default(),
            1.0,
        );
        let other = Coat {
            tint: [1.4, 1.5, 1.6],
            ..Coat::default()
        };
        beast(&mut b, Beast::Horse, &other, &Stance::default(), 1.0);
        assert_eq!(a.positions, b.positions);
        assert_eq!(a.uvs, b.uvs);
        assert_ne!(a.colours, b.colours);
    }
}
