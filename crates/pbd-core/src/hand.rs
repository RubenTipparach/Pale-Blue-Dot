//! The first-person hand (`openspec/changes/hand-model`, design 5 to 5.3),
//! built from the hand's bones as the Tools in Hand mockup's
//! `proposedHandGeometry` builds it (`docs/mockups/tool-motion.html`):
//!
//! - the palm is one traced piece: its outline taken off the bones in the
//!   palm's plane, extruded through the hand's thickness with bevelled
//!   edges and a slightly domed back (`palmMesh`);
//! - every other bone is a chamfered block, an octagonal section with
//!   bevelled ends, the fingers slimmer and tapering, the thumb's tip a
//!   rounded pad (`chamferBlock`);
//! - the thumb is two equal segments, its root sunk toward the palm;
//! - each skin face is the palm's side or the back's by its normal against
//!   the side the fingertips curl to, and a palm-side face takes its part's
//!   UV island (`HandBuilder`);
//! - an ambient occlusion is baked into each vertex from spheres along the
//!   bones and the handle (`bakeAO`);
//! - the whole hand is turned about the handle through the fist
//!   (`grip_turn`).
//!
//! The bones come from `tools/gen_held_tools.py`, placed on each tool's
//! grip in its units; so do the textures. Every face is flat with its own
//! normal. Engine-free: bones in, triangles out. The app draws the result
//! and lights it with [`wrapped`].

use glam::{Quat, Vec2, Vec3};
use serde::Deserialize;

/// Tool units per texel where a face is textured flat: the 16 px textures
/// tile at about 9 mm a texel on the pickaxe's hand (the mockup's
/// `TEX_UNIT`).
pub const TEX_UNIT: f32 = 0.24;
/// The tiling textures are this many texels on a side.
pub const TEX_SIZE: u32 = 16;
/// The palm's texture is this many on a side: it covers one island.
pub const PALM_TEX_SIZE: u32 = 32;

/// Which texture a face is drawn with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum Texture {
    /// The back of the hand and of the fingers.
    Skin,
    Nail,
    Cloth,
    Cuff,
    /// The palm's side of the back of the hand: one island.
    Palm,
    /// The palm's side of a finger, the thumb or the wrist: an island per
    /// segment, a fold at each joint.
    Fold,
}

impl Texture {
    pub const ALL: [Texture; 6] = [
        Texture::Skin,
        Texture::Nail,
        Texture::Cloth,
        Texture::Cuff,
        Texture::Palm,
        Texture::Fold,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    /// Its file under `assets/textures/hand/`.
    pub fn name(self) -> &'static str {
        match self {
            Texture::Skin => "skin",
            Texture::Nail => "nail",
            Texture::Cloth => "cloth",
            Texture::Cuff => "cuff",
            Texture::Palm => "palm",
            Texture::Fold => "fold",
        }
    }

    /// Its size in texels.
    pub fn size(self) -> u32 {
        if self == Texture::Palm {
            PALM_TEX_SIZE
        } else {
            TEX_SIZE
        }
    }
}

/// One bone of the hand, in tool units: a hex prism's place as the game's
/// rig posed it (`tools/gen_held_hand.py`). The 19 bones are, in order, the
/// back of the hand, four fingers of three segments (index to little,
/// proximal to distal), the thumb's three, the wrist, the cuff and the
/// sleeve.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bone {
    pub at: [f32; 3],
    pub axis: [f32; 3],
    pub radius: f32,
    pub length: f32,
    /// Across `across`, where it differs from the radius (the back of the
    /// hand).
    pub width: Option<f32>,
    pub across: Option<[f32; 3]>,
    /// The back of a finger or thumb bone, where it is given rather than
    /// found from the handle (the bare fist's own, [`fist_bones`]).
    #[serde(default)]
    pub out: Option<[f32; 3]>,
    /// Its section's taper from the near end (along -axis) to the far,
    /// where it has one (the bare fist's wrist).
    #[serde(default)]
    pub taper: Option<[f32; 2]>,
}

/// The bones a hand has.
pub const BONES: usize = 19;
/// The first of the arm's bones (the wrist, the cuff, the sleeve), which
/// may turn about the wrist apart from the hand.
pub const ARM: usize = 16;

impl Bone {
    fn at(&self) -> Vec3 {
        Vec3::from(self.at)
    }

    fn axis(&self) -> Vec3 {
        Vec3::from(self.axis).normalize()
    }

    fn width(&self) -> f32 {
        self.width.unwrap_or(self.radius)
    }

    /// Whether every number is finite and every size positive.
    pub fn is_sound(&self) -> bool {
        let finite = self
            .at
            .iter()
            .chain(&self.axis)
            .chain(self.across.iter().flatten())
            .chain(self.out.iter().flatten())
            .chain(self.taper.iter().flatten())
            .chain([self.radius, self.length, self.width()].iter())
            .all(|v| v.is_finite());
        finite
            && Vec3::from(self.axis).length() > 1e-3
            && self.across.is_none_or(|a| Vec3::from(a).length() > 1e-3)
            && self.out.is_none_or(|a| Vec3::from(a).length() > 1e-3)
            && self.taper.is_none_or(|t| t.iter().all(|v| *v > 0.0))
            && self.radius > 0.0
            && self.length > 0.0
            && self.width() > 0.0
    }

    /// Its end nearer `to`.
    fn end_near(&self, to: Vec3) -> Vec3 {
        let (ax, at) = (self.axis(), self.at());
        let (e1, e2) = (at + ax * self.length / 2.0, at - ax * self.length / 2.0);
        if e1.distance(to) < e2.distance(to) {
            e1
        } else {
            e2
        }
    }
}

/// The hand's look and shape (`held_tools.ron` `hand`): the mockup's
/// `HAND_SHAPE` and the parts of `HAND_LOOK` the geometry and its shade
/// use (the tones are in the textures).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Look {
    /// The thumb's length, a share of the rig's.
    pub thumb_len: f32,
    /// How far the thumb's root sinks from the rig's root toward the palm's
    /// centre, 0..1.
    pub thumb_sink: f32,
    /// The grip turned about the handle through the fist, degrees.
    pub grip_turn_deg: f32,
    /// The baked occlusion's strength, 0..1.
    pub ao: f32,
    /// How far the fingertips flush toward `tip_to`, 0..1.
    pub tip_flush: f32,
    pub tip_to: [u8; 3],
    /// The skin the textures are drawn from, straight sRGB.
    pub skin: [u8; 3],
}

impl Look {
    pub fn is_sound(&self) -> bool {
        let unit = [self.thumb_sink, self.ao, self.tip_flush];
        self.thumb_len.is_finite()
            && self.thumb_len > 0.0
            && self.grip_turn_deg.is_finite()
            && unit.iter().all(|v| (0.0..=1.0).contains(v))
            && self.skin.iter().all(|c| *c > 0)
    }

    /// What a vertex flushed by `flush` is multiplied by, per channel: the
    /// skin moved toward `tip_to` (the mockup's `paintProposedHand`).
    pub fn tint(&self, flush: f32) -> [f32; 3] {
        std::array::from_fn(|q| {
            1.0 + flush * (f32::from(self.tip_to[q]) / f32::from(self.skin[q]) - 1.0)
        })
    }
}

/// A hand's triangles for one texture: a list, not indexed, each triangle
/// wound to face out, with a flat normal, a texture coordinate (wgpu's: v
/// down the image), the shade baked into it (a knuckle's crease times the
/// occlusion), the fingertip's flush and which bone it belongs to.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HandMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub shade: Vec<f32>,
    pub flush: Vec<f32>,
    pub part: Vec<u8>,
}

impl HandMesh {
    pub fn triangles(&self) -> usize {
        self.positions.len() / 3
    }
}

/// The meshes of a hand, one per texture, by `Texture::index`.
pub type HandMeshes = [HandMesh; 6];

/// A built hand: the hand's meshes and the arm's (the wrist, the cuff, the
/// sleeve and its hem), and the wrist joint the arm turns about, tool units.
#[derive(Clone, Debug, Default)]
pub struct Hand {
    pub hand: HandMeshes,
    pub arm: HandMeshes,
    pub wrist: Vec3,
}

/// The thumb in two equal segments (hand-model 5.2, 5.3): the rig's three
/// shortened to `thumb_len` and chained from its root, split half way along,
/// the root then sunk `thumb_sink` toward the palm's centre.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Thumb {
    /// The rig's root, where the palm's outline reaches.
    pub rig_root: Vec3,
    /// The first segment's start, sunk toward the palm.
    pub root: Vec3,
    pub joint: Vec3,
    pub tip: Vec3,
}

impl Thumb {
    pub fn new(bones: &[Bone], look: &Look) -> Self {
        let mut pts = Vec::with_capacity(4);
        let first = &bones[13];
        let mut start = first.at() - first.axis() * first.length / 2.0;
        pts.push(start);
        for bone in &bones[13..=15] {
            start += bone.axis() * bone.length * look.thumb_len;
            pts.push(start);
        }
        let rig_root = pts[0];
        pts[0] = pts[0].lerp(bones[0].at(), look.thumb_sink);
        let lens: Vec<f32> = (0..3).map(|k| pts[k].distance(pts[k + 1])).collect();
        let half = lens.iter().sum::<f32>() / 2.0;
        let (mut joint, mut run) = (pts[3], 0.0);
        for k in 0..3 {
            if run + lens[k] >= half {
                joint = pts[k].lerp(pts[k + 1], (half - run) / lens[k]);
                break;
            }
            run += lens[k];
        }
        Self {
            rig_root,
            root: pts[0],
            joint,
            tip: pts[3],
        }
    }

    /// Segment 13 (root to joint) or 15 (joint to tip): its centre, length
    /// and axis.
    fn segment(&self, bone: usize) -> (Vec3, f32, Vec3) {
        let (a, b) = if bone == 13 {
            (self.root, self.joint)
        } else {
            (self.joint, self.tip)
        };
        ((a + b) / 2.0, a.distance(b), (b - a).normalize())
    }
}

/// A palm-side island: the faces of a part turned toward `dir` (more than
/// 0.35 of the way) take `group` and a UV laid on the plane across `pu`
/// and along `axis`, centred on `at`, `2 e` by `len`.
#[derive(Clone, Copy, Debug)]
struct Island {
    dir: Vec3,
    group: Texture,
    at: Vec3,
    axis: Vec3,
    pu: Vec3,
    e: f32,
    len: f32,
    flip_u: bool,
    flip_v: bool,
}

/// A face's UV along its block (the cuff's, so its stitch rows sit at its
/// edges): across by the face's own tangent, along from end to end.
#[derive(Clone, Copy, Debug)]
struct Along {
    at: Vec3,
    axis: Vec3,
    z0: f32,
    len: f32,
}

/// The builder (`HandBuilder`): flat faces, each wound out from a centre.
#[derive(Default)]
struct Builder {
    p: Vec<Vec3>,
    n: Vec<Vec3>,
    uv: Vec<Vec2>,
    k: Vec<f32>,
    flush: Vec<f32>,
    part: Vec<u8>,
    group: Vec<Texture>,
    now_part: u8,
    now_flush: f32,
}

impl Builder {
    #[allow(clippy::too_many_arguments)]
    fn tri(
        &mut self,
        a: Vec3,
        mut b: Vec3,
        mut c: Vec3,
        centre: Vec3,
        mut group: Texture,
        k: f32,
        along: Option<Along>,
        palm: Option<&Island>,
    ) {
        let n = (b - a).cross(c - a);
        if n.length_squared() < 1e-14 {
            return;
        }
        let mut n = n.normalize();
        let mid = (a + b + c) / 3.0 - centre;
        if n.dot(mid) < 0.0 {
            std::mem::swap(&mut b, &mut c);
            n = -n;
        }
        // A skin face turned toward the palm's side is the palm, and takes
        // its part's island.
        let mut island = None;
        if let Some(palm) = palm
            && group == Texture::Skin
            && n.dot(palm.dir) > 0.35
        {
            group = palm.group;
            island = Some(*palm);
        }
        let t = (b - a).normalize();
        let bt = n.cross(t);
        let k_tex = TEX_UNIT * TEX_SIZE as f32;
        for p in [a, b, c] {
            let uv = if let Some(is) = island {
                let d = p - is.at;
                let u = d.dot(is.pu) / (2.0 * is.e) + 0.5;
                let v = d.dot(is.axis) / is.len + 0.5;
                Vec2::new(
                    if is.flip_u { 1.0 - u } else { u },
                    if is.flip_v { 1.0 - v } else { v },
                )
            } else if let Some(al) = along {
                let z = (p - al.at).dot(al.axis);
                let ta = n.cross(al.axis).normalize();
                Vec2::new(p.dot(ta) / k_tex, (z - al.z0) / al.len)
            } else {
                Vec2::new(p.dot(t) / k_tex, p.dot(bt) / k_tex)
            };
            self.p.push(p);
            self.n.push(n);
            self.uv.push(uv);
            self.k.push(k);
            self.flush.push(self.now_flush);
            self.part.push(self.now_part);
            self.group.push(group);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn face(
        &mut self,
        pts: &[Vec3],
        centre: Vec3,
        group: Texture,
        k: f32,
        along: Option<Along>,
        palm: Option<&Island>,
    ) {
        let along = along.map(|a| Along { at: centre, ..a });
        self.tri(pts[0], pts[1], pts[2], centre, group, k, along, palm);
        if pts.len() == 4 {
            self.tri(pts[0], pts[2], pts[3], centre, group, k, along, palm);
        }
    }

    /// The baked occlusion (`bakeAO`): each vertex darkened by the spheres
    /// of the other bones in front of it, `sum (r/d)^2 max(0, n.d)`, by
    /// `1 - strength min(1, sum)`.
    fn bake_ao(&mut self, spheres: &[Sphere], strength: f32) {
        for i in 0..self.k.len() {
            let (p, n) = (self.p[i], self.n[i]);
            let mut sum = 0.0;
            for sp in spheres {
                if sp.part == Some(self.part[i]) {
                    continue;
                }
                let d = sp.c - p;
                let dd = d.length();
                if dd < 1e-6 {
                    continue;
                }
                let cos = n.dot(d) / dd;
                if cos <= 0.0 {
                    continue;
                }
                sum += (sp.r * sp.r / (dd * dd)).min(1.0) * cos;
            }
            self.k[i] *= 1.0 - strength * sum.min(1.0);
        }
    }
}

/// An occluder: a sphere on a bone (`part`), or on the handle (`None`).
struct Sphere {
    c: Vec3,
    r: f32,
    part: Option<u8>,
}

/// Which way the palm's side of a part looks, and the texture its faces
/// take there.
#[derive(Clone, Copy, Debug)]
struct Side {
    dir: Vec3,
    group: Texture,
}

/// A chamfered block (`chamferBlock`).
struct Block {
    at: Vec3,
    axis: Vec3,
    u: Vec3,
    w: f32,
    h: f32,
    len: f32,
    c: f32,
    group: Texture,
    crease: [f32; 2],
    uv_along: bool,
    palm: Option<Side>,
    round: bool,
    taper: Option<[f32; 2]>,
}

fn chamfer_block(b: &mut Builder, o: &Block) {
    let (at, axis, u, w, h, len, c) = (o.at, o.axis, o.u, o.w, o.h, o.len, o.c);
    let palm = o.palm.map(|s| Island {
        dir: s.dir,
        group: s.group,
        at,
        axis,
        pu: axis.cross(s.dir).normalize(),
        e: w.max(h),
        len,
        flip_u: false,
        flip_v: false,
    });
    let v = axis.cross(u);
    let taper_at = |z: f32| match o.taper {
        Some([t0, t1]) => t0 + (t1 - t0) * (z / len + 0.5),
        None => 1.0,
    };
    // The octagon, scaled inward by s at the bevels.
    let section = |s: f32, z: f32| -> [Vec2; 8] {
        let (ww, hh, cc) = (w * taper_at(z) - s, h - s, (c - s * 0.4).max(0.0001));
        [
            Vec2::new(ww - cc, -hh),
            Vec2::new(ww, -hh + cc),
            Vec2::new(ww, hh - cc),
            Vec2::new(ww - cc, hh),
            Vec2::new(-ww + cc, hh),
            Vec2::new(-ww, hh - cc),
            Vec2::new(-ww, -hh + cc),
            Vec2::new(-ww + cc, -hh),
        ]
    };
    let bev = c.min(len * 0.3);
    let ring =
        |z: f32, s: f32| -> [Vec3; 8] { section(s, z).map(|p| at + u * p.x + v * p.y + axis * z) };
    let (z0, z1) = (-len / 2.0, len / 2.0);
    // A round tip (the thumb's): a longer bevel at the far end in two steps.
    let tb = if o.round {
        (w.min(h) * 0.75).min(len * 0.45)
    } else {
        bev
    };
    let (r0, r1, r2) = (ring(z0, bev), ring(z0 + bev, 0.0), ring(z1 - tb, 0.0));
    let r3 = ring(z1, tb * if o.round { 0.95 } else { 1.0 });
    let rm = o.round.then(|| ring(z1 - tb * 0.35, tb * 0.32));
    let (bottom, top) = (at + axis * z0, at + axis * z1);
    let along = Along { at, axis, z0, len };
    let g = o.group;
    let island = palm.as_ref();
    for i in 0..8 {
        let j = (i + 1) % 8;
        b.face(
            &[r1[i], r1[j], r2[j], r2[i]],
            at,
            g,
            1.0,
            o.uv_along.then_some(along),
            island,
        );
        b.face(
            &[r0[i], r0[j], r1[j], r1[i]],
            at,
            g,
            o.crease[0],
            None,
            island,
        );
        if let Some(rm) = rm {
            b.face(&[r2[i], r2[j], rm[j], rm[i]], at, g, 1.0, None, island);
            b.face(&[rm[i], rm[j], r3[j], r3[i]], at, g, 1.0, None, island);
        } else {
            b.face(
                &[r2[i], r2[j], r3[j], r3[i]],
                at,
                g,
                o.crease[1],
                None,
                island,
            );
        }
        b.face(&[bottom, r0[j], r0[i]], at, g, o.crease[0], None, island);
        b.face(&[top, r3[i], r3[j]], at, g, o.crease[1], None, island);
    }
}

/// The palm as one traced piece (`palmMesh`).
#[allow(clippy::too_many_arguments)]
fn palm_mesh(
    b: &mut Builder,
    bones: &[Bone],
    thumb: &Thumb,
    at: Vec3,
    axis: Vec3,
    h: f32,
    nd: Vec3,
    sided: bool,
) {
    let pu = axis.cross(nd).normalize();
    let to2 = |q: Vec3| {
        let d = q - at;
        Vec2::new(d.dot(pu), d.dot(axis))
    };
    let k: Vec<Vec2> = [1, 4, 7, 10]
        .iter()
        .map(|&i| to2(bones[i].end_near(at)))
        .collect();
    let wr = to2(bones[16].end_near(at));
    let ww = bones[16].width() * 0.95;
    let t = to2(thumb.rig_root);
    let sign = |x: f32| {
        if x > 0.0 {
            1.0
        } else if x < 0.0 {
            -1.0
        } else {
            1.0
        }
    };
    let up = sign(k[1].y - wr.y);
    let ts = sign(k[0].x - k[3].x);
    let fr = bones[1].radius * 0.75;
    let v2 = Vec2::new;
    // The outline, going round: the knuckles, the little finger's side, the
    // wrist, the thumb's mound, the thumb's root, the web.
    let mut o = vec![
        v2(k[0].x + ts * fr * 0.4, k[0].y + up * fr * 0.3),
        v2(k[1].x, k[1].y + up * fr * 0.55),
        v2(k[2].x, k[2].y + up * fr * 0.5),
        v2(k[3].x - ts * fr * 0.2, k[3].y + up * fr * 0.2),
        v2(k[3].x - ts * fr * 0.9, k[3].y - up * fr * 0.9),
        v2(
            k[3].x * 0.55 + wr.x * 0.45 - ts * fr * 1.0,
            (k[3].y + wr.y) / 2.0,
        ),
        v2(wr.x - ts * ww, wr.y + up * fr * 0.4),
        v2(wr.x + ts * ww * 0.9, wr.y + up * fr * 0.3),
        v2((wr.x + t.x) / 2.0 + ts * fr * 1.1, (wr.y + t.y) / 2.0),
        v2(t.x + ts * fr * 0.5, t.y),
        v2(t.x + ts * fr * 0.1, t.y + up * fr * 0.9),
        v2(
            (t.x + k[0].x) / 2.0 + ts * fr * 0.2,
            (t.y + k[0].y) / 2.0 + up * fr * 0.4,
        ),
    ];
    // Counter-clockwise in (pu, axis).
    let n = o.len();
    let area: f32 = (0..n).map(|i| o[i].perp_dot(o[(i + 1) % n])).sum();
    if area < 0.0 {
        o.reverse();
    }
    let centre = o.iter().copied().sum::<Vec2>() / n as f32;
    let (cx, cy) = (centre.x, centre.y);
    let ext = o.iter().map(|q| (q.x - cx).abs()).fold(0.0, f32::max);
    let eyy = o.iter().map(|q| (q.y - cy).abs()).fold(0.0, f32::max);
    let thick = |y: f32| {
        let s = (up * (y - wr.y) / (up * (k[1].y - wr.y)).max(1e-6)).clamp(0.0, 1.0);
        h * (0.82 + 0.18 * s)
    };
    let p3 = |x: f32, y: f32, z: f32| at + pu * x + axis * y + nd * z;
    let c = h * 0.45;
    let inset = |q: Vec2, d: f32| {
        let dir = centre - q;
        let l = dir.length();
        let l = if l > 0.0 { l } else { 1.0 };
        q + dir / l * d
    };
    let ring = |sgn: f32, s: f32| -> Vec<Vec3> {
        o.iter()
            .map(|&q| {
                let r = inset(q, s);
                let z = if s > 0.0 { 1.0 } else { 1.0 - 0.45 * c / h };
                p3(r.x, r.y, sgn * thick(q.y) * z)
            })
            .collect()
    };
    let (bk0, bk1, pm1, pm0) = (ring(-1.0, c), ring(-1.0, 0.0), ring(1.0, 0.0), ring(1.0, c));
    let island = Island {
        dir: nd,
        group: Texture::Palm,
        at: p3(cx, cy, 0.0),
        axis,
        pu,
        e: ext,
        len: 2.0 * eyy,
        flip_u: t.x - cx > 0.0,
        flip_v: up < 0.0,
    };
    let is = sided.then_some(&island);
    let g = Texture::Skin;
    let mid = p3(cx, cy, 0.0);
    let (cb, cp) = (p3(cx, cy, -thick(cy) * 1.12), p3(cx, cy, thick(cy) * 0.98));
    for i in 0..n {
        let m = (i + 1) % n;
        let e = o[m] - o[i];
        let half = (o[i] + o[m]) / 2.0;
        let inside = p3(half.x - e.y, half.y + e.x, 0.0);
        b.face(&[bk0[i], bk0[m], bk1[m], bk1[i]], inside, g, 0.92, None, is);
        b.face(&[bk1[i], bk1[m], pm1[m], pm1[i]], inside, g, 1.0, None, is);
        b.face(&[pm1[i], pm1[m], pm0[m], pm0[i]], inside, g, 1.0, None, is);
        // The back, a little domed; the palm, flat.
        b.face(&[cb, bk0[m], bk0[i]], mid, g, 1.0, None, is);
        b.face(&[cp, pm0[i], pm0[m]], mid, g, 1.0, None, is);
    }
}

/// The handle's direction through the fist: from the index finger's first
/// segment to the little finger's.
pub fn handle_dir(bones: &[Bone]) -> Vec3 {
    (bones[10].at() - bones[1].at()).normalize()
}

/// The wrist joint the arm turns about (`HandPair`): the wrist bone's end
/// nearer the back of the hand, before the grip's turn.
pub fn wrist_joint(bones: &[Bone]) -> Vec3 {
    bones[16].end_near(bones[0].at())
}

/// Build a hand (`proposedHandGeometry`) from its 19 bones, the fist on the
/// handle (tool units), its look, and the sleeve lengthened `sleeve_extra`
/// tool units at its far end.
pub fn build(bones: &[Bone], fist: Vec3, look: &Look, sleeve_extra: f32) -> Hand {
    build_with(bones, fist, look, sleeve_extra, false)
}

/// Build the bare fist ([`fist_bones`]): one skin tone all over, no palm or
/// fold faces, and no handle in the occlusion (`held-swing` decision 10,
/// the fist of version 16).
pub fn build_bare(bones: &[Bone], fist: Vec3, look: &Look, sleeve_extra: f32) -> Hand {
    build_with(bones, fist, look, sleeve_extra, true)
}

fn build_with(bones: &[Bone], fist: Vec3, look: &Look, sleeve_extra: f32, bare: bool) -> Hand {
    assert_eq!(bones.len(), BONES, "a hand has {BONES} bones");
    let mut b = Builder::default();
    let handle = handle_dir(bones);
    let thumb = Thumb::new(bones, look);
    let tips = [3, 6, 9, 12].iter().map(|&i| bones[i].at()).sum::<Vec3>() / 4.0;
    for (i, p) in bones.iter().enumerate() {
        if i == 14 {
            continue; // the thumb is two segments: 13 and 15
        }
        b.now_part = i as u8;
        let (mut at, mut len, mut axis) = (p.at(), p.length, p.axis());
        if i == 13 || i == 15 {
            (at, len, axis) = thumb.segment(i);
        }
        let first =
            p.across
                .map(Vec3::from)
                .unwrap_or(if axis.x.abs() < 0.9 { Vec3::X } else { Vec3::Y });
        let mut u = (first - axis * first.dot(axis)).normalize();
        let finger = (1..=15).contains(&i);
        let distal = finger && i % 3 == 0;
        let seg = if finger { (i - 1) % 3 } else { 0 };
        // A finger block turned so a flat face looks away from the handle
        // (the back of the finger), so the nail and the knuckles sit on it.
        let mut out = None;
        if finger && let Some(o) = p.out {
            // The bare fist's own (version 16).
            let o = Vec3::from(o).normalize();
            u = axis.cross(o).normalize();
            out = Some(o);
        } else if finger {
            let mut rel = at - fist;
            rel -= handle * rel.dot(handle);
            rel -= axis * rel.dot(axis);
            if rel.length_squared() > 1e-8 {
                let o = rel.normalize();
                u = axis.cross(o).normalize();
                out = Some(o);
            }
        }
        let (mut w, mut h) = (p.width(), p.radius);
        let mut group = Texture::Skin;
        let mut uv_along = false;
        let mut crease = [1.0, 1.0];
        let flush = if distal { look.tip_flush } else { 0.0 };
        // The palm's side: a finger's is opposite its back; the back of the
        // hand's and the wrist's where the fingertips curl to, square to
        // the bone.
        let mut palm = None;
        // The bare fist's wrist is one tone, as the forearm it runs into.
        if i <= 16 && !(bare && i == 16) {
            if let Some(o) = out {
                palm = Some(Side {
                    dir: -o,
                    group: Texture::Fold,
                });
            } else if i == 0 || i == 16 {
                let mut d = tips - at;
                d -= axis * d.dot(axis);
                if d.length_squared() > 1e-8 {
                    palm = Some(Side {
                        dir: d.normalize(),
                        group: if i == 0 { Texture::Palm } else { Texture::Fold },
                    });
                }
            }
        }
        if i == 17 {
            group = Texture::Cuff;
            uv_along = true;
        }
        if i == 18 {
            group = Texture::Cloth;
            // Lengthen the sleeve at its far end, away from the cuff.
            let cuff = bones[17].at();
            let far = if (at + axis * len / 2.0).distance(cuff)
                > (at - axis * len / 2.0).distance(cuff)
            {
                1.0
            } else {
                -1.0
            };
            at += axis * far * sleeve_extra / 2.0;
            len += sleeve_extra;
        }
        // Slimmer fingers with flatter sides, each segment tapering.
        let mut taper = p.taper;
        if finger {
            w *= 0.8;
            h *= 0.84;
            crease = if seg < 2 { [1.0, 0.88] } else { [1.0, 1.0] };
            taper = Some(if seg == 2 { [0.95, 0.8] } else { [1.0, 0.92] });
        }
        let is_thumb = (13..=15).contains(&i);
        if is_thumb {
            w *= 1.08;
            h *= 1.04;
        }
        if i == 16 {
            w *= 1.05;
            h *= 0.92;
            len *= 1.15;
        }
        let c = w.min(h) * if finger { 0.26 } else { 0.3 };
        b.now_flush = flush;
        if i == 0 {
            let dir = palm.map_or(axis.cross(handle).normalize(), |s| s.dir);
            palm_mesh(
                &mut b,
                bones,
                &thumb,
                at,
                axis,
                h * 1.05,
                dir,
                palm.is_some() && !bare,
            );
        } else {
            chamfer_block(
                &mut b,
                &Block {
                    at,
                    axis,
                    u,
                    w,
                    h,
                    len,
                    c,
                    group,
                    crease,
                    uv_along,
                    // One tone (the bare fist): no palm or fold faces.
                    palm: if bare { None } else { palm },
                    round: is_thumb && distal,
                    taper,
                },
            );
        }
        b.now_flush = 0.0;
        // The hem: a ring of the sleeve's cloth where it meets the cuff.
        if i == 18 {
            let (pa, pl, cuff) = (p.at(), p.length, bones[17].at());
            let toward =
                if (pa + axis * pl / 2.0).distance(cuff) < (pa - axis * pl / 2.0).distance(cuff) {
                    1.0
                } else {
                    -1.0
                };
            chamfer_block(
                &mut b,
                &Block {
                    at: pa + axis * toward * (pl / 2.0 - 0.3),
                    axis,
                    u,
                    w: w * 1.06,
                    h: h * 1.06,
                    len: 0.5,
                    c: c * 1.06,
                    group,
                    crease: [1.0, 1.0],
                    uv_along: false,
                    palm: None,
                    round: false,
                    taper: None,
                },
            );
        }
        // A nail plate on each tip, on the back face near the end.
        if distal && let Some(o) = out {
            let (nw, nl, nz) = if is_thumb {
                (0.7, 0.5, 0.08)
            } else {
                (0.62, 0.56, 0.14)
            };
            let nat = at + axis * len * 0.14 + o * h * 0.93;
            chamfer_block(
                &mut b,
                &Block {
                    at: at + axis * len * (nz - 0.14) + (nat - at),
                    axis,
                    u,
                    w: w * nw,
                    h: h * 0.14,
                    len: len * nl,
                    c: h * 0.08,
                    group: Texture::Nail,
                    crease: [1.0, 1.0],
                    uv_along: false,
                    palm: None,
                    round: false,
                    taper: None,
                },
            );
        }
    }
    // The occluders: three spheres along every bone and five along the
    // handle through the fist.
    if look.ao > 0.0 {
        let mut spheres = Vec::new();
        for (i, p) in bones.iter().enumerate() {
            if i == 14 {
                continue;
            }
            let (at, len, ax) = if i == 13 || i == 15 {
                thumb.segment(i)
            } else {
                (p.at(), p.length, p.axis())
            };
            let r = p.width().min(p.radius);
            for t in [-0.33, 0.0, 0.33] {
                spheres.push(Sphere {
                    c: at + ax * t * len,
                    r,
                    part: Some(i as u8),
                });
            }
        }
        for t in (-2..=2).filter(|_| !bare) {
            spheres.push(Sphere {
                c: fist + handle * t as f32 * 0.8,
                r: 0.55,
                part: None,
            });
        }
        b.bake_ao(&spheres, look.ao);
    }
    // The grip turned about the handle through the fist.
    let turn = Quat::from_axis_angle(handle, look.grip_turn_deg.to_radians());
    let mut built = Hand {
        wrist: wrist_joint(bones),
        ..Hand::default()
    };
    for i in 0..b.p.len() {
        let set = if (b.part[i] as usize) < ARM {
            &mut built.hand
        } else {
            &mut built.arm
        };
        let mesh = &mut set[b.group[i].index()];
        mesh.positions
            .push((turn * (b.p[i] - fist) + fist).to_array());
        mesh.normals.push((turn * b.n[i]).to_array());
        // The mockup's coordinates have v up the image (three.js flips its
        // textures); wgpu's run down it.
        mesh.uvs.push([b.uv[i].x, 1.0 - b.uv[i].y]);
        mesh.shade.push(b.k[i]);
        mesh.flush.push(b.flush[i]);
        mesh.part.push(b.part[i]);
    }
    built
}

/// The bare fist's shape (`held-swing` decision 10, "The fist, version
/// 16"; the Tools in Hand mockup's `FIST` table): how the pickaxe grip's
/// bones are re-chained into a closed fist. Lengths in the bone's own units
/// (the grip's tool units); offsets along A (index to little finger), N
/// (palm-ward) and D (wrist to knuckles).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FistShape {
    /// Each finger's turn at its knuckle, middle and tip joints, degrees.
    pub curl_deg: [f32; 3],
    /// Each finger segment's length, a share of the approved.
    pub len: [f32; 3],
    /// The thumb's joint and tip from the index knuckle, and how far its
    /// root moves from the approved, (A, N, D).
    pub thumb_joint: [f32; 3],
    pub thumb_tip: [f32; 3],
    pub thumb_root: [f32; 3],
    /// The back of the hand shortened by this, the wrist, cuff and sleeve
    /// moved up with it.
    pub short: f32,
    /// How far the wrist reaches into the heel of the hand, and its half
    /// width and half thickness, shares of the back's.
    pub wrist_in: f32,
    pub wrist_w: f32,
    pub wrist_h: f32,
    /// Eye space: the way the back of the hand faces at twist 0 (to the
    /// right, a little toward the eye).
    pub back_eye: [f32; 3],
}

impl FistShape {
    /// Version 16, as the owner approved it (2026-11-21).
    pub const V16: Self = Self {
        curl_deg: [92.0, 102.0, 62.0],
        len: [0.86, 0.95, 0.9],
        thumb_joint: [-0.9, 1.6, -1.3],
        thumb_tip: [-0.3, 2.1, 0.1],
        thumb_root: [0.0, 0.4, 0.6],
        short: 0.9,
        wrist_in: 0.7,
        wrist_w: 0.72,
        wrist_h: 0.72,
        back_eye: [1.0, 0.1, 0.35],
    };
}

/// The bare fist's own bones from a grip's (the mockup's `fistBones`): the
/// fingers re-chained from each knuckle in a tight curl in the plane of the
/// back (D, wrist to knuckles) and the palm (N); the thumb from its root
/// along the index finger's side to a tip over its middle segment; the back
/// of the hand shortened and the arm moved up with it; the wrist slimmed to
/// the back of the hand, its wide side across the knuckles.
pub fn fist_bones(bones: &[Bone], look: &Look, f: &FistShape) -> Vec<Bone> {
    assert_eq!(bones.len(), BONES, "a hand has {BONES} bones");
    let mut p = bones.to_vec();
    let at0 = p[0].at();
    let knuck = [1, 4, 7, 10].map(|i| p[i].end_near(at0));
    let kc = knuck.iter().copied().sum::<Vec3>() * 0.25;
    let mut d = p[0].axis();
    if d.dot(kc - at0) < 0.0 {
        d = -d;
    }
    let a = knuck[3] - knuck[0];
    let a = (a - d * a.dot(d)).normalize();
    let mut n = [1, 4, 7, 10]
        .iter()
        .map(|&i| Vec3::from(p[i].axis))
        .sum::<Vec3>();
    n -= d * n.dot(d);
    let n = (n - a * n.dot(a)).normalize();
    // A direction in the curl's plane, th from D toward N.
    let dir = |th: f32| d * th.cos() + n * th.sin();
    for (finger, base) in [1, 4, 7, 10].into_iter().enumerate() {
        let (mut at, mut th) = (knuck[finger], 0.0f32);
        for s in 0..3 {
            th += f.curl_deg[s].to_radians();
            let bone = &mut p[base + s];
            let (len, ax) = (bone.length * f.len[s], dir(th));
            bone.at = (at + ax * len / 2.0).to_array();
            bone.axis = ax.to_array();
            bone.length = len;
            bone.out = Some(dir(th - std::f32::consts::FRAC_PI_2).to_array());
            at += ax * len;
        }
    }
    // The back of the hand shortened from the wrist's end, the arm moved up
    // with it.
    p[0].at = (at0 + d * f.short / 2.0).to_array();
    p[0].length -= f.short;
    for bone in &mut p[16..=18] {
        bone.at = (bone.at() + d * f.short).to_array();
    }
    let off = |o: [f32; 3]| a * o[0] + n * o[1] + d * o[2];
    let root = p[13].end_near(at0 - d * 10.0) + d * f.short / 2.0 + off(f.thumb_root);
    let (joint, tip) = (knuck[0] + off(f.thumb_joint), knuck[0] + off(f.thumb_tip));
    let pts = [root, root.lerp(joint, 0.5), joint, tip];
    for s in 0..3 {
        let (from, to) = (pts[s], pts[s + 1]);
        let ax = (to - from).normalize();
        let bone = &mut p[13 + s];
        // The builder shortens the thumb's chain by thumb_len.
        bone.length = from.distance(to) / look.thumb_len;
        bone.axis = ax.to_array();
        bone.at = (from + ax * bone.length / 2.0).to_array();
        // The nail on the thumb's top (the index finger's side).
        let o = -a + n * 0.35;
        bone.out = Some((o - ax * o.dot(ax)).normalize().to_array());
    }
    // The wrist, slimmed to the back of the hand, reaching into its heel.
    let ax = p[16].axis();
    let to_hand = if (p[0].at() - p[16].at()).dot(ax) < 0.0 {
        -1.0
    } else {
        1.0
    };
    p[16].at = (p[16].at() + ax * to_hand * f.wrist_in / 2.0).to_array();
    p[16].length += f.wrist_in;
    p[16].taper = Some([0.85, 1.05]);
    p[16].across = Some(a.to_array());
    p[16].width = Some(p[0].width() * f.wrist_w);
    p[16].radius = p[0].radius * f.wrist_h * 1.4;
    p
}

/// How the hand is lit (`held.ron` `hand_light`): a wrapped light, so the
/// side turned from the light still shows its form, plus a soft fill.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shade {
    pub ambient: f32,
    pub wrap: f32,
    pub fill: f32,
}

/// The light on the hand, eye space: from above, a little right and toward
/// the eye.
pub const LIGHT_EYE: Vec3 = Vec3::new(0.3, 0.8, 0.55);
/// The fill, eye space: from below and behind the hand.
pub const FILL_EYE: Vec3 = Vec3::new(-0.2, -0.75, -0.6);

/// A face's shade by its normal `n`, with the light `l` and the fill `f`
/// as unit vectors in the same frame:
/// `ambient + wrap * (0.5 + 0.5 n.l)^2 + fill * max(n.f, 0)`.
pub fn wrapped(shade: Shade, n: Vec3, l: Vec3, f: Vec3) -> f32 {
    let half = 0.5 + 0.5 * n.dot(l);
    shade.ambient + shade.wrap * half * half + shade.fill * n.dot(f).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(round: bool, taper: Option<[f32; 2]>) -> HandMesh {
        let mut b = Builder::default();
        chamfer_block(
            &mut b,
            &Block {
                at: Vec3::new(1.0, 2.0, 3.0),
                axis: Vec3::Z,
                u: Vec3::X,
                w: 0.5,
                h: 0.4,
                len: 2.0,
                c: 0.15,
                group: Texture::Skin,
                crease: [1.0, 0.9],
                uv_along: false,
                palm: Some(Side {
                    dir: Vec3::Y,
                    group: Texture::Fold,
                }),
                round,
                taper,
            },
        );
        let mut m = HandMesh::default();
        for i in 0..b.p.len() {
            m.positions.push(b.p[i].to_array());
            m.normals.push(b.n[i].to_array());
            m.uvs.push(b.uv[i].to_array());
            m.shade.push(b.k[i]);
            m.part.push(b.group[i].index() as u8);
        }
        m
    }

    /// Every edge of a triangle list is shared by exactly two triangles.
    pub(crate) fn closed(positions: &[[f32; 3]]) -> bool {
        let key = |a: [f32; 3]| a.map(|v| (v * 1e4).round() as i64);
        let mut edges = std::collections::HashMap::new();
        for t in 0..positions.len() / 3 {
            for e in 0..3 {
                let (a, b) = (
                    key(positions[t * 3 + e]),
                    key(positions[t * 3 + (e + 1) % 3]),
                );
                *edges
                    .entry(if a < b { (a, b) } else { (b, a) })
                    .or_insert(0) += 1;
            }
        }
        edges.values().all(|n| *n == 2)
    }

    /// A block is closed and every face looks out from its centre, wound
    /// with its normal; tapered and round-tipped too. Its bevels carry the
    /// crease, and its faces toward the palm's side take the fold's island.
    #[test]
    fn a_block_is_closed_and_faces_out() {
        for (round, taper) in [(false, None), (false, Some([1.0, 0.92])), (true, None)] {
            let m = block(round, taper);
            let tris = if round { 80 } else { 64 };
            assert_eq!(m.triangles(), tris, "{round} {taper:?}");
            let at = Vec3::new(1.0, 2.0, 3.0);
            for t in 0..m.triangles() {
                let p: Vec<Vec3> = (0..3).map(|i| Vec3::from(m.positions[t * 3 + i])).collect();
                let n = Vec3::from(m.normals[t * 3]);
                assert!(n.dot((p[0] + p[1] + p[2]) / 3.0 - at) > 0.0, "{t} faces in");
                let wound = (p[1] - p[0]).cross(p[2] - p[0]).normalize();
                assert!(wound.dot(n) > 0.999, "{t}: wound against its normal");
                let fold = m.part[t * 3] == Texture::Fold.index() as u8;
                assert_eq!(fold, n.y > 0.35, "{t}: the palm's side by its normal");
            }
            assert!(closed(&m.positions), "an open edge");
        }
        let m = block(false, None);
        assert_eq!(
            m.shade.iter().filter(|k| (**k - 0.9).abs() < 1e-6).count(),
            3 * 24
        );
    }

    /// Off the palm's side a face's texture tiles at one texel per
    /// `TEX_UNIT`.
    #[test]
    fn a_texel_is_a_fixed_size() {
        let m = block(false, None);
        let t = (0..m.triangles())
            .find(|&t| m.part[t * 3] == Texture::Skin.index() as u8)
            .unwrap();
        let (p0, p1) = (
            Vec3::from(m.positions[t * 3]),
            Vec3::from(m.positions[t * 3 + 1]),
        );
        let (t0, t1) = (Vec2::from(m.uvs[t * 3]), Vec2::from(m.uvs[t * 3 + 1]));
        let ratio = t0.distance(t1) / p0.distance(p1);
        assert!((ratio - 1.0 / (TEX_UNIT * 16.0)).abs() < 1e-4, "{ratio}");
    }

    /// The wrapped light, `0.45 + 0.55 (0.5 + 0.5 n.l)^2 + 0.12 max(n.f, 0)`:
    /// full toward the light, the ambient alone turned right away from it,
    /// a quarter of the wrap side on, and the fill only on faces toward it.
    #[test]
    fn the_wrapped_light() {
        let s = Shade {
            ambient: 0.45,
            wrap: 0.55,
            fill: 0.12,
        };
        let (l, f) = (Vec3::Y, -Vec3::Y);
        assert!((wrapped(s, Vec3::Y, l, f) - 1.0).abs() < 1e-6);
        assert!((wrapped(s, -Vec3::Y, l, f) - (0.45 + 0.12)).abs() < 1e-6);
        assert!((wrapped(s, Vec3::X, l, f) - (0.45 + 0.55 * 0.25)).abs() < 1e-6);
        let n = Vec3::new(0.6, 0.8, 0.0);
        let half: f32 = 0.5 + 0.5 * 0.8;
        assert!((wrapped(s, n, l, f) - (0.45 + 0.55 * half * half)).abs() < 1e-6);
    }

    /// The fingertip's flush moves the skin toward `tip_to`: none at 0, the
    /// whole way at 1.
    #[test]
    fn the_flush_tints_toward_the_tip_colour() {
        let look = Look {
            thumb_len: 0.72,
            thumb_sink: 0.4,
            grip_turn_deg: 45.0,
            ao: 0.6,
            tip_flush: 0.22,
            tip_to: [226, 140, 128],
            skin: [227, 176, 137],
        };
        assert_eq!(look.tint(0.0), [1.0, 1.0, 1.0]);
        let full = look.tint(1.0);
        for ((got, to), skin) in full.iter().zip(look.tip_to).zip(look.skin) {
            assert!((got - f32::from(to) / f32::from(skin)).abs() < 1e-6);
        }
        let some = look.tint(0.22);
        assert!(some[1] < 1.0 && some[1] > full[1], "redder, not red");
    }
}
