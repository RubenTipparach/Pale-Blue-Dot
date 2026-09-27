//! Hex pixels: a model made of hexagonal prisms, the way Minecraft makes an
//! item's model out of cubes, and the stubby prisms of the hand that holds it.
//!
//! A model is hexes on a pointy-top grid of any spacing, in axial
//! coordinates: row `r` runs along x, so a row is a straight line and a
//! tool's handle is whole rows (`openspec/changes/hex-held-tools`). Each hex
//! has its own colour and depth. Its front and back sit at plus and minus
//! half its depth, and a side wall is built toward a neighbour only over the
//! part of the hex that stands above that neighbour: a round handle is closed
//! and has no faces inside it.
//!
//! Engine-free: hexes in, triangles out. The shapes come from
//! `tools/gen_held_tools.py`; the app draws the result.

use glam::{Vec2, Vec3};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

const SQRT_3: f32 = 1.732_050_8;

/// One hex of a model: axial position, straight sRGB colour, and depth in the
/// model's units.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(from = "(i32, i32, [u8; 3], f32)")]
pub struct Hex {
    pub q: i32,
    pub r: i32,
    pub colour: [u8; 3],
    pub depth: f32,
}

impl From<(i32, i32, [u8; 3], f32)> for Hex {
    fn from((q, r, colour, depth): (i32, i32, [u8; 3], f32)) -> Self {
        Self {
            q,
            r,
            colour,
            depth,
        }
    }
}

/// A hex's centre on a grid of `spacing` (centre to centre), y up.
pub fn centre(q: i32, r: i32, spacing: f32) -> Vec2 {
    Vec2::new(
        spacing * (q as f32 + r as f32 / 2.0),
        -spacing * r as f32 * SQRT_3 / 2.0,
    )
}

/// The neighbour across each edge, in order: edge `k` runs from corner `k`
/// (at 30 + 60k degrees) to corner `k + 1` and faces 60 + 60k degrees, y up.
pub const NEIGHBOURS: [(i32, i32); 6] = [(1, -1), (0, -1), (-1, 0), (-1, 1), (0, 1), (1, 0)];

/// A model as triangles: a list, not indexed, each triangle wound to face
/// out.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HexelMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    /// Linear RGBA, the colour times the face's shade.
    pub colours: Vec<[f32; 4]>,
}

impl HexelMesh {
    pub fn triangles(&self) -> usize {
        self.positions.len() / 3
    }

    fn push(&mut self, mut tri: [Vec3; 3], outward: Vec3, colour: [u8; 3], shade: f32) {
        if (tri[1] - tri[0]).cross(tri[2] - tri[0]).dot(outward) < 0.0 {
            tri.swap(1, 2);
        }
        let c = [
            linear(colour[0]) * shade,
            linear(colour[1]) * shade,
            linear(colour[2]) * shade,
            1.0,
        ];
        for p in tri {
            self.positions.push(p.to_array());
            self.normals.push(outward.to_array());
            self.colours.push(c);
        }
    }

    fn quad(&mut self, q: [Vec3; 4], outward: Vec3, colour: [u8; 3], shade: f32) {
        self.push([q[0], q[1], q[2]], outward, colour, shade);
        self.push([q[0], q[2], q[3]], outward, colour, shade);
    }
}

/// The shade a hex face takes by the way it faces, as Minecraft's item models
/// do: the front brightest, the back and the downward sides darkest.
pub fn shade(normal: Vec3) -> f32 {
    if normal.z > 0.5 {
        1.0
    } else if normal.z < -0.5 {
        0.72
    } else {
        0.78 + 0.18 * normal.y
    }
}

fn linear(c: u8) -> f32 {
    let c = c as f32 / 255.0;
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// A model's hexes on a grid of `spacing`, as triangles.
pub fn mesh(hexes: &[Hex], spacing: f32) -> HexelMesh {
    let depth: HashMap<(i32, i32), f32> = hexes.iter().map(|h| ((h.q, h.r), h.depth)).collect();
    let radius = spacing / SQRT_3;
    let mut out = HexelMesh::default();
    for h in hexes {
        let c = centre(h.q, h.r, spacing);
        let half = h.depth * 0.5;
        let corner = |k: usize, z: f32| {
            let a = (30.0 + 60.0 * k as f32).to_radians();
            (c + Vec2::new(a.cos(), a.sin()) * radius).extend(z)
        };
        for (k, (dq, dr)) in NEIGHBOURS.into_iter().enumerate() {
            let k1 = (k + 1) % 6;
            out.push(
                [c.extend(half), corner(k, half), corner(k1, half)],
                Vec3::Z,
                h.colour,
                shade(Vec3::Z),
            );
            out.push(
                [c.extend(-half), corner(k, -half), corner(k1, -half)],
                -Vec3::Z,
                h.colour,
                shade(-Vec3::Z),
            );
            // The wall toward this neighbour covers only what stands above
            // it, front and back: all of the hex where there is none.
            let spans: &[(f32, f32)] = match depth.get(&(h.q + dq, h.r + dr)) {
                None => &[(-half, half)],
                Some(d) if d * 0.5 < half => &[(d * 0.5, half), (-half, -d * 0.5)],
                Some(_) => &[],
            };
            let facing = (60.0 + 60.0 * k as f32).to_radians();
            let outward = Vec3::new(facing.cos(), facing.sin(), 0.0);
            for &(z0, z1) in spans {
                out.quad(
                    [corner(k, z0), corner(k1, z0), corner(k1, z1), corner(k, z1)],
                    outward,
                    h.colour,
                    shade(outward),
                );
            }
        }
    }
    out
}

/// The hexes grouped into pieces that share edges, largest first: one piece
/// is a model with nothing floating.
pub fn pieces(hexes: &[Hex]) -> Vec<usize> {
    let index: BTreeMap<(i32, i32), usize> = hexes
        .iter()
        .enumerate()
        .map(|(i, h)| ((h.q, h.r), i))
        .collect();
    let mut seen = vec![false; hexes.len()];
    let mut sizes = Vec::new();
    for start in 0..hexes.len() {
        if seen[start] {
            continue;
        }
        seen[start] = true;
        let mut stack = vec![start];
        let mut size = 0;
        while let Some(i) = stack.pop() {
            size += 1;
            for (dq, dr) in NEIGHBOURS {
                if let Some(&j) = index.get(&(hexes[i].q + dq, hexes[i].r + dr))
                    && !seen[j]
                {
                    seen[j] = true;
                    stack.push(j);
                }
            }
        }
        sizes.push(size);
    }
    sizes.sort_unstable_by(|a, b| b.cmp(a));
    sizes
}

/// One part of the hand: a hexagonal prism along `axis`, centred at `at`,
/// `radius` to a corner and `length` long, in the model's units. With
/// `across` and `width` it is flattened: `width` to a corner along `across`
/// and `radius` across that (the back of the hand).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prism {
    pub axis: [f32; 3],
    pub at: [f32; 3],
    pub radius: f32,
    pub length: f32,
    #[serde(default)]
    pub across: Option<[f32; 3]>,
    #[serde(default)]
    pub width: Option<f32>,
    /// Straight sRGB.
    pub colour: [u8; 3],
}

impl Prism {
    /// Whether every number is finite and every size positive.
    pub fn is_sound(&self) -> bool {
        let finite = self
            .axis
            .iter()
            .chain(&self.at)
            .chain(self.across.iter().flatten())
            .chain(self.width.iter())
            .all(|v| v.is_finite());
        finite
            && Vec3::from(self.axis).length() > 1e-3
            && self.radius > 0.0
            && self.length > 0.0
            && self.width.is_none_or(|w| w > 0.0)
    }
}

/// A prism's triangles, added to `out`, each face shaded by how it faces
/// `light` (a unit vector in the model's frame): from 0.62 facing away to one
/// facing it. The hand is shaded this way, against a light given as the eye
/// sees it, so it reads as lit from above whatever the tool's pose.
pub fn prism(part: &Prism, light: Vec3, out: &mut HexelMesh) {
    let axis = Vec3::from(part.axis).normalize();
    let first = part
        .across
        .map(Vec3::from)
        .unwrap_or_else(|| if axis.x.abs() < 0.9 { Vec3::X } else { Vec3::Y });
    let u = (first - axis * first.dot(axis)).normalize();
    let v = axis.cross(u);
    let at = Vec3::from(part.at);
    let w = part.width.unwrap_or(part.radius);
    let ring = |z: f32| -> [Vec3; 6] {
        std::array::from_fn(|k| {
            let a = (60.0 * k as f32).to_radians();
            at + u * (a.cos() * w) + v * (a.sin() * part.radius) + axis * z
        })
    };
    let (lo, hi) = (ring(-part.length / 2.0), ring(part.length / 2.0));
    let lit = |n: Vec3| 0.62 + 0.38 * n.dot(light).max(0.0);
    let (bottom, top) = (at - axis * part.length / 2.0, at + axis * part.length / 2.0);
    for k in 0..6 {
        let k1 = (k + 1) % 6;
        out.push([bottom, lo[k], lo[k1]], -axis, part.colour, lit(-axis));
        out.push([top, hi[k], hi[k1]], axis, part.colour, lit(axis));
        let mid = (lo[k] + lo[k1]) * 0.5 - (at - axis * part.length / 2.0);
        let side = (mid - axis * mid.dot(axis)).normalize_or(u);
        out.quad([lo[k], lo[k1], hi[k1], hi[k]], side, part.colour, lit(side));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(q: i32, r: i32, depth: f32) -> Hex {
        Hex {
            q,
            r,
            colour: [200, 120, 40],
            depth,
        }
    }

    /// Neighbours are one spacing apart, and a row is a straight line.
    #[test]
    fn neighbours_are_one_spacing_apart_and_a_row_is_straight() {
        for (dq, dr) in NEIGHBOURS {
            let d = centre(dq, dr, 0.25).length();
            assert!((d - 0.25).abs() < 1e-6, "({dq}, {dr}) is {d} away");
        }
        for q in -5..5 {
            assert_eq!(centre(q, 0, 0.25).y, 0.0);
        }
    }

    /// One hex is one closed prism: a hexagon front and back and six sides.
    #[test]
    fn one_hex_is_one_closed_prism() {
        let m = mesh(&[hex(0, 0, 1.0)], 1.0);
        assert_eq!(m.triangles(), 6 + 6 + 12);
        assert_closed(&m);
    }

    /// Two hexes of one depth side by side share no wall.
    #[test]
    fn equal_neighbours_share_no_wall() {
        let m = mesh(&[hex(0, 0, 1.0), hex(1, 0, 1.0)], 1.0);
        assert_eq!(m.triangles(), 2 * 24 - 4);
        assert_closed(&m);
    }

    /// A deeper hex beside a shallower one gets a wall only over what stands
    /// above it, front and back, and the two are still one closed surface.
    #[test]
    fn a_depth_step_walls_only_the_difference() {
        let m = mesh(&[hex(0, 0, 1.0), hex(1, 0, 0.5)], 1.0);
        // The shared edge: the shallow hex builds nothing there, and the deep
        // one two strips (front and back) instead of one full wall.
        assert_eq!(m.triangles(), 2 * 24);
        // The full wall beside a strip meets it in a T at the corner, so the
        // edges do not pair one to one; the volume says it is still sealed
        // and wound outward: a hexagon of area sqrt(3)/2, 1 and 0.5 deep.
        assert_volume(&m, 3.0f32.sqrt() / 2.0 * 1.5);
    }

    /// A round handle (rows of shrinking depth) is closed and one piece.
    #[test]
    fn a_round_handle_is_closed_and_one_piece() {
        let mut hs = Vec::new();
        for q in 0..12 {
            for (r, d) in [(-1, 0.5), (0, 1.0), (1, 0.5)] {
                hs.push(hex(q, r, d));
            }
        }
        assert_eq!(pieces(&hs), vec![36]);
        let area = 3.0f32.sqrt() / 2.0 * 0.25 * 0.25;
        assert_volume(&mesh(&hs, 0.25), area * 12.0 * (0.5 + 1.0 + 0.5));
    }

    #[test]
    fn the_front_is_brightest_and_the_underside_darkest() {
        assert_eq!(shade(Vec3::Z), 1.0);
        assert!(shade(Vec3::Y) > shade(Vec3::X));
        assert!(shade(Vec3::X) > shade(-Vec3::Y));
        assert!(shade(-Vec3::Z) < 1.0);
    }

    /// A prism, round or flattened, is closed, lit from the light's side,
    /// and as long and wide as asked.
    #[test]
    fn a_prism_is_closed_and_lit_from_the_light() {
        for (across, width) in [(None, None), (Some([0.0, 0.0, 1.0]), Some(2.0))] {
            let part = Prism {
                axis: [1.0, 0.0, 0.0],
                at: [1.0, 2.0, 3.0],
                radius: 0.5,
                length: 4.0,
                across,
                width,
                colour: [200, 150, 120],
            };
            assert!(part.is_sound());
            let mut m = HexelMesh::default();
            prism(&part, Vec3::Y, &mut m);
            assert_eq!(m.triangles(), 24);
            assert_closed(&m);
            let xs = m.positions.iter().map(|p| p[0]);
            let (lo, hi) = xs.fold((f32::MAX, f32::MIN), |(a, b), x| (a.min(x), b.max(x)));
            assert!((hi - lo - 4.0).abs() < 1e-5);
            // Its farthest corner from the axis is the width, or the radius.
            let off = |p: &[f32; 3]| Vec2::new(p[1] - 2.0, p[2] - 3.0).length();
            let reach = m.positions.iter().map(off).fold(0.0f32, f32::max);
            assert!((reach - width.unwrap_or(0.5)).abs() < 1e-5, "{reach}");
            // The face turned most toward the light is brighter than the one
            // turned most away.
            let facing = |i: usize| Vec3::from(m.normals[i]).dot(Vec3::Y);
            let most = (0..m.normals.len()).max_by(|a, b| facing(*a).total_cmp(&facing(*b)));
            let least = (0..m.normals.len()).min_by(|a, b| facing(*a).total_cmp(&facing(*b)));
            let (most, least) = (most.unwrap(), least.unwrap());
            assert!(m.colours[most][0] > m.colours[least][0]);
        }
        let bad = Prism {
            axis: [0.0, 0.0, 0.0],
            at: [0.0; 3],
            radius: 1.0,
            length: 1.0,
            across: None,
            width: None,
            colour: [0; 3],
        };
        assert!(!bad.is_sound());
    }

    /// The volume a mesh encloses, from its triangles (the divergence
    /// theorem): right only if it is sealed and wound outward, and blind to
    /// T-junctions, which leave no gap.
    fn assert_volume(m: &HexelMesh, want: f32) {
        let v: f32 = m
            .positions
            .chunks(3)
            .map(|t| Vec3::from(t[0]).dot(Vec3::from(t[1]).cross(Vec3::from(t[2]))) / 6.0)
            .sum();
        assert!((v - want).abs() < want * 1e-4, "volume {v}, want {want}");
    }

    /// A mesh corner, rounded to a ten-thousandth of a unit.
    type Corner = (i64, i64, i64);

    /// Every edge of the mesh is walked once each way, so no face is missing
    /// and none is doubled.
    fn assert_closed(m: &HexelMesh) {
        let key = |p: [f32; 3]| {
            (
                (p[0] * 1e4).round() as i64,
                (p[1] * 1e4).round() as i64,
                (p[2] * 1e4).round() as i64,
            )
        };
        let mut edges: BTreeMap<(Corner, Corner), i32> = BTreeMap::new();
        for tri in m.positions.chunks(3) {
            for (a, b) in [(0, 1), (1, 2), (2, 0)] {
                let (p, q) = (key(tri[a]), key(tri[b]));
                *edges.entry((p, q)).or_default() += 1;
                *edges.entry((q, p)).or_default() -= 1;
            }
        }
        let open = edges.values().filter(|n| **n != 0).count();
        assert_eq!(open, 0, "{open} unmatched directed edges");
    }
}
