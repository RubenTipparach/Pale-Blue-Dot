//! A player's door (`player-building` design B8): the leaf and its frame,
//! each ONE shaped form rather than boxes stuck together.
//!
//! Both are an outline drawn in the plane of the doorway, `u` along the
//! wall and `v` up it, extruded through a thickness `t` with its faces
//! bevelled into its sides all round ([`extrude`]). The leaf's outline is
//! a slab with its corners clipped, drawn in its own texture (`door.png`:
//! planks, ledges and a brace, the strap hinges on the hinge side and a ring
//! pull, painted in rather than modelled); the frame's is one U lining the
//! doorway's jambs and head.

use crate::settlement::pieces::{DoorLeaf, Frame, LEAF_M, MeshBuf, Meshes};
use glam::{Vec2, Vec3};

/// The leaf's texture, mapped once over each face.
pub const LEAF_TEXTURE: &str = "door";
/// How wide the frame's casing is, metres, in from each jamb and down from
/// the head.
pub const CASING_M: f32 = 0.07;
/// How far the casing stands proud of each face of the wall, metres.
pub const CASING_PROUD_M: f32 = 0.025;
/// The bevel round the leaf's faces and the casing's, metres.
pub const BEVEL_M: f32 = 0.012;
/// The clip off each corner of the leaf, metres.
pub const LEAF_CLIP_M: f32 = 0.035;

/// One triangle of an extrusion in `(u, v, t)`, its face's outward normal,
/// and whether it is on a face (`t` at an end) or round the side.
pub struct Tri {
    pub p: [Vec3; 3],
    pub normal: Vec3,
    pub face: bool,
}

/// Twice the signed area of an outline: positive counter-clockwise.
fn area2(outline: &[Vec2]) -> f32 {
    let n = outline.len();
    (0..n)
        .map(|i| outline[i].perp_dot(outline[(i + 1) % n]))
        .sum()
}

/// The outline moved `by` inward at every corner, along the corner's
/// bisector so each edge moves `by` (counter-clockwise outlines).
fn inset(outline: &[Vec2], by: f32) -> Vec<Vec2> {
    let n = outline.len();
    (0..n)
        .map(|i| {
            let (p, v, q) = (outline[(i + n - 1) % n], outline[i], outline[(i + 1) % n]);
            let a = (v - p).normalize_or_zero();
            let b = (q - v).normalize_or_zero();
            let (na, nb) = (a.perp(), b.perp());
            let bis = (na + nb).normalize_or(na);
            let cos = bis.dot(na).max(0.2);
            v + bis * (by / cos)
        })
        .collect()
}

/// An outline cut into triangles by clipping ears (counter-clockwise,
/// simple, possibly not convex).
fn ears(outline: &[Vec2]) -> Vec<[Vec2; 3]> {
    let mut idx: Vec<usize> = (0..outline.len()).collect();
    let mut out = Vec::new();
    let inside = |a: Vec2, b: Vec2, c: Vec2, p: Vec2| {
        (b - a).perp_dot(p - a) > 1e-7
            && (c - b).perp_dot(p - b) > 1e-7
            && (a - c).perp_dot(p - c) > 1e-7
    };
    let mut guard = 0;
    while idx.len() > 3 && guard < 10_000 {
        guard += 1;
        let n = idx.len();
        let mut cut = None;
        for k in 0..n {
            let (a, b, c) = (
                outline[idx[(k + n - 1) % n]],
                outline[idx[k]],
                outline[idx[(k + 1) % n]],
            );
            if (b - a).perp_dot(c - b) <= 1e-9 {
                continue;
            }
            let blocked = idx
                .iter()
                .map(|&j| outline[j])
                .any(|p| p != a && p != b && p != c && inside(a, b, c, p));
            if !blocked {
                out.push([a, b, c]);
                cut = Some(k);
                break;
            }
        }
        match cut {
            Some(k) => {
                idx.remove(k);
            }
            None => break,
        }
    }
    if idx.len() == 3 {
        out.push([outline[idx[0]], outline[idx[1]], outline[idx[2]]]);
    }
    out
}

/// `outline` (in `u`, `v`) extruded from `t0` to `t1`, each face bevelled
/// `bevel` into the sides: one closed form.
pub fn extrude(outline: &[Vec2], t0: f32, t1: f32, bevel: f32) -> Vec<Tri> {
    let mut outline = outline.to_vec();
    if area2(&outline) < 0.0 {
        outline.reverse();
    }
    let n = outline.len();
    let small = inset(&outline, bevel);
    let mut out = Vec::new();
    let at = |p: Vec2, t: f32| Vec3::new(p.x, p.y, t);
    for (t, dir) in [(t0, -1.0f32), (t1, 1.0)] {
        for [a, b, c] in ears(&small) {
            out.push(Tri {
                p: [at(a, t), at(b, t), at(c, t)],
                normal: Vec3::Z * dir,
                face: true,
            });
        }
    }
    let (b0, b1) = (t0 + bevel, t1 - bevel);
    for i in 0..n {
        let j = (i + 1) % n;
        let e = outline[j] - outline[i];
        let side = Vec2::new(e.y, -e.x).normalize_or_zero();
        let side3 = Vec3::new(side.x, side.y, 0.0);
        let mut quad = |p: [Vec3; 4], normal: Vec3| {
            out.push(Tri {
                p: [p[0], p[1], p[2]],
                normal,
                face: false,
            });
            out.push(Tri {
                p: [p[0], p[2], p[3]],
                normal,
                face: false,
            });
        };
        quad(
            [
                at(outline[i], b0),
                at(outline[j], b0),
                at(outline[j], b1),
                at(outline[i], b1),
            ],
            side3,
        );
        quad(
            [
                at(small[i], t0),
                at(small[j], t0),
                at(outline[j], b0),
                at(outline[i], b0),
            ],
            (side3 - Vec3::Z).normalize(),
        );
        quad(
            [
                at(outline[i], b1),
                at(outline[j], b1),
                at(small[j], t1),
                at(small[i], t1),
            ],
            (side3 + Vec3::Z).normalize(),
        );
    }
    out
}

/// The leaf's outline in its own plane: `width` by `height`, its corners
/// clipped.
pub fn leaf_outline(width: f32, height: f32) -> Vec<Vec2> {
    let c = LEAF_CLIP_M.min(width * 0.2).min(height * 0.2);
    vec![
        Vec2::new(c, 0.0),
        Vec2::new(width - c, 0.0),
        Vec2::new(width, c),
        Vec2::new(width, height - c),
        Vec2::new(width - c, height),
        Vec2::new(c, height),
        Vec2::new(0.0, height - c),
        Vec2::new(0.0, c),
    ]
}

/// The frame's outline: one U lining a doorway `width` wide and `height`
/// high, centred on `u = 0`, `CASING_M` in from each jamb and down from the
/// head.
pub fn casing_outline(width: f32, height: f32) -> Vec<Vec2> {
    let (h, c) = (width * 0.5, CASING_M);
    vec![
        Vec2::new(-h, 0.0),
        Vec2::new(-h + c, 0.0),
        Vec2::new(-h + c, height - c),
        Vec2::new(h - c, height - c),
        Vec2::new(h - c, 0.0),
        Vec2::new(h, 0.0),
        Vec2::new(h, height),
        Vec2::new(-h, height),
    ]
}

/// Push a triangle into `buf` in planet-local metres, wound to face
/// `normal` (in the frame).
fn push(buf: &mut MeshBuf, frame: Frame, p: [Vec3; 3], uv: [Vec2; 3], normal: Vec3) {
    let (mut p, mut uv) = (p, uv);
    if (p[1] - p[0]).cross(p[2] - p[0]).dot(normal) < 0.0 {
        p.swap(1, 2);
        uv.swap(1, 2);
    }
    let n = (frame.x * normal.x + frame.y * normal.y + frame.z * normal.z).normalize_or_zero();
    for k in 0..3 {
        buf.positions.push(frame.world(p[k]).to_array());
        buf.normals.push(n.to_array());
        buf.uvs.push(uv[k].to_array());
    }
}

/// The leaf drawn swung `angle` from shut (0) toward open (a quarter turn
/// into the room), in planet-local metres: one bevelled slab in
/// [`LEAF_TEXTURE`], the texture's left edge on the hinge side. Its faces
/// take the texture once; its edges the colour of the face's border.
pub fn leaf_mesh(leaf: &DoorLeaf, frame: Frame, angle: f32) -> Meshes {
    let (c, s) = (angle.cos(), angle.sin());
    let d = leaf.along * c - leaf.out * s;
    let t = leaf.out * c + leaf.along * s;
    let (width, height) = (leaf.width, leaf.y1 - leaf.y0);
    let to_frame = |q: Vec3| {
        let plan = leaf.hinge + d * q.x + t * q.z;
        Vec3::new(plan.x, leaf.y0 + q.y, plan.y)
    };
    let dir = |n: Vec3| {
        let plan = d * n.x + t * n.z;
        Vec3::new(plan.x, n.y, plan.y)
    };
    let uv = |q: Vec3| {
        Vec2::new(
            (q.x / width).clamp(0.0, 1.0),
            1.0 - (q.y / height).clamp(0.0, 1.0),
        )
    };
    let mut buf = MeshBuf::default();
    for tri in extrude(&leaf_outline(width, height), 0.0, LEAF_M, BEVEL_M) {
        push(
            &mut buf,
            frame,
            tri.p.map(to_frame),
            tri.p.map(uv),
            dir(tri.normal),
        );
    }
    let mut meshes = Meshes::new();
    meshes.insert(LEAF_TEXTURE.to_string(), buf);
    meshes
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every triangle of an extrusion faces the way it says, and the U and
    /// the clipped slab are each cut into the right number of triangles.
    #[test]
    fn the_leaf_and_the_casing_are_closed_bevelled_forms() {
        for (outline, faces) in [(leaf_outline(0.9, 2.0), 6), (casing_outline(1.0, 2.1), 6)] {
            let tris = extrude(&outline, -0.1, 0.1, BEVEL_M);
            let caps = tris.iter().filter(|t| t.face).count();
            assert_eq!(caps, 2 * faces, "each face cut into n - 2 triangles");
            // Three quads a side: the side and a bevel at each face.
            assert_eq!(tris.len() - caps, outline.len() * 6);
            for tri in &tris {
                let n = (tri.p[1] - tri.p[0]).cross(tri.p[2] - tri.p[0]);
                assert!(n.length() > 1e-9, "no sliver");
            }
            // The caps cover the inset outline's area, front and back.
            let small = inset(
                &{
                    let mut o = outline.clone();
                    if area2(&o) < 0.0 {
                        o.reverse();
                    }
                    o
                },
                BEVEL_M,
            );
            let want = area2(&small) * 0.5;
            let have: f32 = tris
                .iter()
                .filter(|t| t.face && t.normal.z > 0.0)
                .map(|t| {
                    let (a, b, c) = (t.p[0].truncate(), t.p[1].truncate(), t.p[2].truncate());
                    ((b - a).perp_dot(c - a)).abs() * 0.5
                })
                .sum();
            assert!((have - want).abs() < 1e-4, "{have} against {want}");
        }
    }
}
