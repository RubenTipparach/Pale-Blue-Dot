//! The first-person hand as chamfered blocks (`openspec/changes/hand-model`):
//! each bone of the hand is a block with an octagonal section and bevelled
//! ends, every face flat with its own normal, so the silhouette reads rounder
//! while the faces stay hard. The blocks come from `tools/gen_held_tools.py`
//! (the Tools in Hand mockup's `proposedHandGeometry`), already placed on
//! each tool's grip; this builds their triangles, texture coordinates and
//! knuckle shade, and gives the wrapped light the hand is shaded by.
//!
//! Engine-free: blocks in, triangles out. The app draws the result.

use glam::{Vec2, Vec3};
use serde::Deserialize;

/// Tool units per texel: the textures are 16 px and tile, about 9 mm a
/// texel on the pickaxe's hand (the mockup's `TEX_UNIT`).
pub const TEX_UNIT: f32 = 0.24;
/// The hand's textures are this many texels on a side.
pub const TEX_SIZE: u32 = 16;

/// Which texture a block is drawn with.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum Texture {
    Skin,
    Nail,
    Cloth,
    Cuff,
}

impl Texture {
    pub const ALL: [Texture; 4] = [Texture::Skin, Texture::Nail, Texture::Cloth, Texture::Cuff];

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
        }
    }
}

/// One chamfered block of the hand, in tool units.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Block {
    /// Which bone (`back`, `finger2.1`, `thumb.0`, `wrist`, `hem`,
    /// `finger4.2.nail`, ...).
    pub part: String,
    pub at: [f32; 3],
    /// Along the block.
    pub axis: [f32; 3],
    /// Across it: the half size `w` runs along `u`, `h` along `axis x u`.
    pub u: [f32; 3],
    pub w: f32,
    pub h: f32,
    pub len: f32,
    /// How far the octagon's corners are cut, and the ends bevelled.
    pub chamfer: f32,
    pub texture: Texture,
    /// The shade of the bevel at the near and the far end: below 1 at a
    /// knuckle, the crease.
    pub crease: [f32; 2],
    /// The texture runs along the block from end to end (the cuff, so its
    /// stitch rows sit at its edges), not flat on each face.
    pub along: bool,
    /// Part of the arm (the wrist, the cuff, the sleeve and its hem), which
    /// may turn about the wrist apart from the hand.
    pub arm: bool,
    /// The sleeve: lengthened along this way by the held settings' sleeve.
    pub grow: Option<[f32; 3]>,
}

impl Block {
    /// Whether every number is finite and every size positive.
    pub fn is_sound(&self) -> bool {
        let finite = self
            .at
            .iter()
            .chain(&self.axis)
            .chain(&self.u)
            .chain(&self.crease)
            .chain(self.grow.iter().flatten())
            .chain([self.w, self.h, self.len, self.chamfer].iter())
            .all(|v| v.is_finite());
        finite
            && Vec3::from(self.axis).length() > 1e-3
            && Vec3::from(self.u).length() > 1e-3
            && self.w > 0.0
            && self.h > 0.0
            && self.len > 0.0
            && self.chamfer > 0.0
    }

    /// Its two end centres, along the axis.
    pub fn ends(&self) -> (Vec3, Vec3) {
        let axis = Vec3::from(self.axis).normalize();
        let at = Vec3::from(self.at);
        (at - axis * self.len / 2.0, at + axis * self.len / 2.0)
    }
}

/// A hand's triangles for one texture: a list, not indexed, each triangle
/// wound to face out from its block's centre, with a flat normal, a texture
/// coordinate (wgpu's: v down the image) and the knuckle shade per vertex.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HandMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub crease: Vec<f32>,
}

impl HandMesh {
    pub fn triangles(&self) -> usize {
        self.positions.len() / 3
    }

    fn tri(
        &mut self,
        mut t: [Vec3; 3],
        centre: Vec3,
        crease: f32,
        along: Option<(Vec3, f32, f32)>,
    ) {
        let n = (t[1] - t[0]).cross(t[2] - t[0]);
        if n.length_squared() < 1e-14 {
            return;
        }
        let mut n = n.normalize();
        let mid = (t[0] + t[1] + t[2]) / 3.0 - centre;
        if n.dot(mid) < 0.0 {
            t.swap(1, 2);
            n = -n;
        }
        let tangent = (t[1] - t[0]).normalize();
        let bitangent = n.cross(tangent);
        let k = TEX_UNIT * TEX_SIZE as f32;
        for p in t {
            let uv = match along {
                Some((axis, z0, len)) => {
                    let z = (p - centre).dot(axis);
                    let ta = n.cross(axis).normalize_or_zero();
                    Vec2::new(p.dot(ta) / k, (z - z0) / len)
                }
                None => Vec2::new(p.dot(tangent) / k, p.dot(bitangent) / k),
            };
            self.positions.push(p.to_array());
            self.normals.push(n.to_array());
            // The mockup's coordinates have v up the image (three.js flips
            // its textures); wgpu's run down it.
            self.uvs.push([uv.x, 1.0 - uv.y]);
            self.crease.push(crease);
        }
    }

    fn face(&mut self, p: &[Vec3], centre: Vec3, crease: f32, along: Option<(Vec3, f32, f32)>) {
        self.tri([p[0], p[1], p[2]], centre, crease, along);
        if p.len() == 4 {
            self.tri([p[0], p[2], p[3]], centre, crease, along);
        }
    }
}

/// The meshes of a hand, one per texture, by `Texture::index`.
pub type HandMeshes = [HandMesh; 4];

/// A block's faces, added to the mesh of its texture: the octagonal sides,
/// the bevel bands at each end (shaded by the crease) and the end caps.
/// `sleeve_extra` lengthens a block that grows (the sleeve) at its far end,
/// tool units.
pub fn block(b: &Block, sleeve_extra: f32, out: &mut HandMeshes) {
    let axis = Vec3::from(b.axis).normalize();
    let u = Vec3::from(b.u).normalize();
    let v = axis.cross(u);
    let mut at = Vec3::from(b.at);
    let mut len = b.len;
    if let Some(grow) = b.grow {
        at += Vec3::from(grow).normalize_or_zero() * sleeve_extra.max(0.0) / 2.0;
        len += sleeve_extra.max(0.0);
    }
    let (w, h, c) = (b.w, b.h, b.chamfer);
    // The octagon, scaled inward by s at the bevels.
    let section = |s: f32| -> [Vec2; 8] {
        let (ww, hh, cc) = (w - s, h - s, (c - s * 0.4).max(0.0001));
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
    let bevel = c.min(len * 0.3);
    let ring =
        |z: f32, s: f32| -> [Vec3; 8] { section(s).map(|p| at + u * p.x + v * p.y + axis * z) };
    let (z0, z1) = (-len / 2.0, len / 2.0);
    let (r0, r1, r2, r3) = (
        ring(z0, bevel),
        ring(z0 + bevel, 0.0),
        ring(z1 - bevel, 0.0),
        ring(z1, bevel),
    );
    let (bottom, top) = (at + axis * z0, at + axis * z1);
    let mesh = &mut out[b.texture.index()];
    let along = b.along.then_some((axis, z0, len));
    for i in 0..8 {
        let j = (i + 1) % 8;
        mesh.face(&[r1[i], r1[j], r2[j], r2[i]], at, 1.0, along);
        mesh.face(&[r0[i], r0[j], r1[j], r1[i]], at, b.crease[0], None);
        mesh.face(&[r2[i], r2[j], r3[j], r3[i]], at, b.crease[1], None);
        mesh.face(&[bottom, r0[j], r0[i]], at, b.crease[0], None);
        mesh.face(&[top, r3[i], r3[j]], at, b.crease[1], None);
    }
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

    fn finger() -> Block {
        Block {
            part: "finger1.0".into(),
            at: [1.0, 2.0, 3.0],
            axis: [0.0, 0.0, 1.0],
            u: [1.0, 0.0, 0.0],
            w: 0.5,
            h: 0.4,
            len: 2.0,
            chamfer: 0.15,
            texture: Texture::Skin,
            crease: [1.0, 0.9],
            along: false,
            arm: false,
            grow: None,
        }
    }

    /// A block is closed and every face looks out from its centre: 8 sides,
    /// 16 bevel quads and 16 cap triangles, 64 triangles; its bevels carry
    /// the crease and it stays inside its half sizes.
    #[test]
    fn a_block_is_closed_and_faces_out() {
        let b = finger();
        let mut out = HandMeshes::default();
        block(&b, 0.0, &mut out);
        let m = &out[0];
        assert_eq!(m.triangles(), 64);
        assert!(out[1..].iter().all(|m| m.positions.is_empty()));
        let at = Vec3::from(b.at);
        for t in 0..m.triangles() {
            let p: Vec<Vec3> = (0..3).map(|i| Vec3::from(m.positions[t * 3 + i])).collect();
            let n = Vec3::from(m.normals[t * 3]);
            let mid = (p[0] + p[1] + p[2]) / 3.0 - at;
            assert!(n.dot(mid) > 0.0, "triangle {t} faces in");
            let wound = (p[1] - p[0]).cross(p[2] - p[0]).normalize();
            assert!(
                wound.dot(n) > 0.999,
                "triangle {t}: wound against its normal"
            );
            for q in &p {
                let d = *q - at;
                assert!(d.x.abs() <= b.w + 1e-5 && d.y.abs() <= b.h + 1e-5);
                assert!(d.z.abs() <= b.len / 2.0 + 1e-5);
            }
        }
        // The sides are plain; the near bevel and cap are creased at the
        // far end only.
        assert_eq!(
            m.crease.iter().filter(|k| **k == 1.0).count(),
            3 * (16 + 16 + 8)
        );
        assert_eq!(
            m.crease.iter().filter(|k| (**k - 0.9).abs() < 1e-6).count(),
            3 * 24
        );
        // Every edge is shared by exactly two triangles: closed.
        let key = |a: [f32; 3]| a.map(|v| (v * 1e4).round() as i64);
        let mut edges = std::collections::HashMap::new();
        for t in 0..m.triangles() {
            for e in 0..3 {
                let (a, b) = (
                    key(m.positions[t * 3 + e]),
                    key(m.positions[t * 3 + (e + 1) % 3]),
                );
                *edges
                    .entry(if a < b { (a, b) } else { (b, a) })
                    .or_insert(0) += 1;
            }
        }
        assert!(edges.values().all(|n| *n == 2), "an open edge");
    }

    /// The texture tiles at one texel per `TEX_UNIT`: along a side face, one
    /// tool unit is 1 / (16 * 0.24) of the texture.
    #[test]
    fn a_texel_is_a_fixed_size() {
        let mut out = HandMeshes::default();
        block(&finger(), 0.0, &mut out);
        let m = &out[0];
        let (p0, p1) = (Vec3::from(m.positions[0]), Vec3::from(m.positions[1]));
        let (t0, t1) = (Vec2::from(m.uvs[0]), Vec2::from(m.uvs[1]));
        let ratio = t0.distance(t1) / p0.distance(p1);
        assert!((ratio - 1.0 / (TEX_UNIT * 16.0)).abs() < 1e-4, "{ratio}");
    }

    /// A growing block lengthens at its far end only.
    #[test]
    fn the_sleeve_grows_at_its_far_end() {
        let mut b = finger();
        b.grow = Some([0.0, 0.0, 1.0]);
        let mut out = HandMeshes::default();
        block(&b, 6.0, &mut out);
        let zs: Vec<f32> = out[0].positions.iter().map(|p| p[2]).collect();
        let lo = zs.iter().cloned().fold(f32::INFINITY, f32::min);
        let hi = zs.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        assert!(
            (lo - 2.0).abs() < 1e-5 && (hi - 10.0).abs() < 1e-5,
            "{lo} {hi}"
        );
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
}
