//! Hex pixels: a small RGBA picture turned into a model made of hexagonal
//! prisms, the way Minecraft turns an item's sprite into a model of cubes.
//!
//! The hexels sit on an offset hex grid laid over the picture: pointy-top
//! hexagons one pixel flat to flat, in rows `sqrt(3)/2` of a pixel apart,
//! every other row shifted half a hexel. A hexel exists where the picture is
//! opaque under it, and wears the colour most of it covers. The mesh is one
//! pixel deep, with a hexagon front and back and a side only where the
//! neighbouring hexel is empty, so a handle is a closed strip rather than a
//! stack of prisms with their insides drawn.
//!
//! Engine-free: pixels in, triangles out. The app decodes the picture and
//! draws the result (`openspec/changes/fishing-and-equipment/design.md`
//! section 12).

use glam::{Vec2, Vec3};
use std::collections::{BTreeMap, HashSet};

/// Rows are this many pixels apart.
pub const ROW_PITCH: f32 = 0.866_025_4;
/// A hexel's corner radius, in pixels: one pixel flat to flat.
pub const RADIUS: f32 = 0.577_350_3;
/// How far from a hexel's centre its six outer samples sit, in pixels.
///
/// Sampling the centre alone drops rows: the rows are 0.87 px apart, so a
/// one-pixel 45 degree line in the picture came out as separate pieces
/// (measured on the shipped tool icons, design section 12). Six more samples
/// at 0.3 px keep every such line connected without fattening it; at 0.4 px
/// the handles read a hexel thicker than the icon draws them.
pub const SAMPLE_RADIUS: f32 = 0.3;

/// One hex pixel.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hexel {
    pub row: i32,
    pub col: i32,
    /// Its centre, in picture pixels with y DOWN, as the picture is stored.
    pub centre: Vec2,
    /// Straight sRGB and alpha, as the picture has them.
    pub colour: [u8; 4],
}

/// A hexel's centre in picture pixels (y down).
pub fn centre(row: i32, col: i32) -> Vec2 {
    let shift = if row.rem_euclid(2) == 1 { 0.5 } else { 0.0 };
    Vec2::new(col as f32 + 0.5 + shift, 0.5 + row as f32 * ROW_PITCH)
}

/// The hexel whose centre is nearest `point` (picture pixels, y down).
pub fn at(point: Vec2) -> (i32, i32) {
    let row = ((point.y - 0.5) / ROW_PITCH).round() as i32;
    let shift = if row.rem_euclid(2) == 1 { 0.5 } else { 0.0 };
    let col = (point.x - 0.5 - shift).round() as i32;
    (row, col)
}

/// The hexels of a `width` by `height` RGBA picture, row by row. A pixel is
/// opaque at alpha above half.
pub fn hexels(width: u32, height: u32, rgba: &[u8]) -> Vec<Hexel> {
    let (w, h) = (width as i32, height as i32);
    assert_eq!(
        rgba.len(),
        (width * height * 4) as usize,
        "RGBA, row by row"
    );
    let pixel = |p: Vec2| -> Option<[u8; 4]> {
        if p.x < 0.0 || p.y < 0.0 {
            return None;
        }
        let (x, y) = (p.x as i32, p.y as i32);
        if x >= w || y >= h {
            return None;
        }
        let i = ((y * w + x) * 4) as usize;
        let px = [rgba[i], rgba[i + 1], rgba[i + 2], rgba[i + 3]];
        (px[3] > 127).then_some(px)
    };
    let rows = ((h as f32 - 0.5 + RADIUS) / ROW_PITCH).ceil() as i32 + 1;
    let mut out = Vec::new();
    for row in 0..rows {
        for col in -1..=w {
            let c = centre(row, col);
            let samples = std::iter::once(c).chain((0..6).map(|k| {
                let a = (90.0 + 60.0 * k as f32).to_radians();
                c + Vec2::new(a.cos(), a.sin()) * SAMPLE_RADIUS
            }));
            // The colour most of the hexel covers; the centre's on a tie, so
            // an outline pixel does not win over the body it outlines.
            let mut counts: Vec<([u8; 4], u32)> = Vec::new();
            for px in samples.filter_map(pixel) {
                match counts.iter_mut().find(|(c, _)| *c == px) {
                    Some((_, n)) => *n += 1,
                    None => counts.push((px, 1)),
                }
            }
            let Some(&(first, _)) = counts.first() else {
                continue;
            };
            let best = counts.iter().map(|(_, n)| *n).max().unwrap_or(0);
            let colour = pixel(c)
                .filter(|px| counts.iter().any(|(k, n)| k == px && *n == best))
                .or_else(|| counts.iter().find(|(_, n)| *n == best).map(|(k, _)| *k))
                .unwrap_or(first);
            out.push(Hexel {
                row,
                col,
                centre: c,
                colour,
            });
        }
    }
    out
}

/// Whether two hexels share an edge.
pub fn neighbours(a: (i32, i32), b: (i32, i32)) -> bool {
    let d = centre(a.0, a.1) - centre(b.0, b.1);
    (d.length() - 1.0).abs() < 1e-3
}

/// A model of hexels: a triangle list, not indexed, each triangle wound to
/// face out. Model space is in picture pixels, x right, y UP, z toward the
/// viewer, the picture's plane at z = 0.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HexelMesh {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    /// Linear RGBA, the pixel's colour times the face's shade.
    pub colours: Vec<[f32; 4]>,
}

impl HexelMesh {
    pub fn triangles(&self) -> usize {
        self.positions.len() / 3
    }
}

/// The shade a face takes by the way it faces, as Minecraft's item models
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

/// A picture `height` pixels tall, as hexels `depth` pixels deep.
pub fn mesh(hexels: &[Hexel], height: u32, depth: f32) -> HexelMesh {
    let filled: HashSet<(i32, i32)> = hexels.iter().map(|x| (x.row, x.col)).collect();
    let half = depth * 0.5;
    let to_model = |p: Vec2| Vec2::new(p.x, height as f32 - p.y);
    let mut out = HexelMesh::default();
    let mut push = |mut tri: [Vec3; 3], outward: Vec3, colour: [u8; 4]| {
        let normal = (tri[1] - tri[0]).cross(tri[2] - tri[0]);
        if normal.dot(outward) < 0.0 {
            tri.swap(1, 2);
        }
        let s = shade(outward);
        let c = [
            linear(colour[0]) * s,
            linear(colour[1]) * s,
            linear(colour[2]) * s,
            1.0,
        ];
        for p in tri {
            out.positions.push(p.to_array());
            out.normals.push(outward.to_array());
            out.colours.push(c);
        }
    };
    for hexel in hexels {
        let c = to_model(hexel.centre);
        // Corners at 30, 90, ... 330 degrees in model space: pointy top.
        let corner = |k: usize| {
            let a = (30.0 + 60.0 * k as f32).to_radians();
            c + Vec2::new(a.cos(), a.sin()) * RADIUS
        };
        let front = c.extend(half);
        let back = c.extend(-half);
        for k in 0..6 {
            let (a, b) = (corner(k), corner((k + 1) % 6));
            push(
                [front, a.extend(half), b.extend(half)],
                Vec3::Z,
                hexel.colour,
            );
            push(
                [back, a.extend(-half), b.extend(-half)],
                -Vec3::Z,
                hexel.colour,
            );
            // The edge from corner k to k+1 faces 60 + 60k degrees; a side is
            // built only where no hexel lies that way.
            let facing = (60.0 + 60.0 * k as f32).to_radians();
            let outward = Vec2::new(facing.cos(), facing.sin());
            let beyond = hexel.centre + Vec2::new(outward.x, -outward.y);
            if filled.contains(&at(beyond)) {
                continue;
            }
            let side = outward.extend(0.0);
            push(
                [a.extend(-half), b.extend(-half), b.extend(half)],
                side,
                hexel.colour,
            );
            push(
                [a.extend(-half), b.extend(half), a.extend(half)],
                side,
                hexel.colour,
            );
        }
    }
    out
}

/// The hexels grouped into pieces that share edges, largest first: one piece
/// is a model with nothing floating.
pub fn pieces(hexels: &[Hexel]) -> Vec<usize> {
    let index: BTreeMap<(i32, i32), usize> = hexels
        .iter()
        .enumerate()
        .map(|(i, x)| ((x.row, x.col), i))
        .collect();
    let mut seen = vec![false; hexels.len()];
    let mut sizes = Vec::new();
    for start in 0..hexels.len() {
        if seen[start] {
            continue;
        }
        seen[start] = true;
        let mut stack = vec![start];
        let mut size = 0;
        while let Some(i) = stack.pop() {
            size += 1;
            let c = hexels[i].centre;
            for k in 0..6 {
                let a = (60.0 * k as f32).to_radians();
                if let Some(&j) = index.get(&at(c + Vec2::new(a.cos(), a.sin())))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn picture(w: u32, h: u32, opaque: impl Fn(u32, u32) -> bool) -> Vec<u8> {
        let mut rgba = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let on = opaque(x, y);
                rgba.extend_from_slice(&[200, 120, 40, if on { 255 } else { 0 }]);
            }
        }
        rgba
    }

    #[test]
    fn the_grid_is_one_pixel_flat_to_flat() {
        assert!(neighbours((0, 0), (0, 1)));
        assert!(neighbours((0, 0), (1, 0)), "row 1 is shifted right");
        assert!(neighbours((1, 0), (2, 0)));
        assert!(neighbours((1, 0), (2, 1)), "and row 2 is not");
        assert!(!neighbours((0, 0), (2, 0)));
        for (r, c) in [(0, 0), (3, 7), (8, 2)] {
            assert_eq!(at(centre(r, c)), (r, c));
        }
    }

    /// One pixel is one hexel: a hexagon front and back and six sides.
    #[test]
    fn one_pixel_is_one_closed_prism() {
        let rgba = picture(1, 1, |_, _| true);
        let hx = hexels(1, 1, &rgba);
        assert_eq!(hx.len(), 1);
        let m = mesh(&hx, 1, 1.0);
        assert_eq!(m.triangles(), 6 + 6 + 12);
        assert_closed(&m);
    }

    /// Two hexels side by side share no wall.
    #[test]
    fn neighbours_share_no_wall() {
        let rgba = picture(2, 1, |_, _| true);
        let hx = hexels(2, 1, &rgba);
        assert_eq!(hx.len(), 2);
        assert_eq!(mesh(&hx, 1, 1.0).triangles(), 2 * 24 - 4);
        assert_closed(&mesh(&hx, 1, 1.0));
    }

    /// A one-pixel 45 degree line, the icons' handles, stays one piece.
    #[test]
    fn a_thin_diagonal_stays_connected() {
        for flip in [false, true] {
            let rgba = picture(16, 16, |x, y| if flip { x + y == 15 } else { x == y });
            let hx = hexels(16, 16, &rgba);
            assert_eq!(pieces(&hx).len(), 1, "flip {flip}: {:?}", pieces(&hx));
            // And not fattened into a band. The line is 16 * sqrt(2) = 22.6 px
            // long and a hexel row is 0.87 px, so a single file of hexels
            // stepping along it is about 1.6 per unit length, 36 here; a band
            // two hexels wide would pass 45.
            assert!(hx.len() <= 40, "{} hexels for 16 pixels", hx.len());
        }
    }

    /// A blob is watertight: every edge of the mesh is shared by exactly two
    /// triangles, so no face is missing and none is doubled.
    #[test]
    fn a_blob_is_watertight() {
        let rgba = picture(9, 7, |x, y| {
            (x as i32 - 4).pow(2) + (y as i32 - 3).pow(2) < 9
        });
        let hx = hexels(9, 7, &rgba);
        assert!(hx.len() > 20);
        assert_closed(&mesh(&hx, 7, 1.0));
    }

    #[test]
    fn the_front_is_brightest_and_the_underside_darkest() {
        assert_eq!(shade(Vec3::Z), 1.0);
        assert!(shade(Vec3::Y) > shade(Vec3::X));
        assert!(shade(Vec3::X) > shade(-Vec3::Y));
        assert!(shade(-Vec3::Z) < 1.0);
    }

    /// A mesh corner, rounded to a tenth of a millimetre in pixels.
    type Corner = (i64, i64, i64);

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
                // Directed: in a closed, consistently wound mesh each edge is
                // walked once each way.
                *edges.entry((p, q)).or_default() += 1;
                *edges.entry((q, p)).or_default() -= 1;
            }
        }
        let open = edges.values().filter(|n| **n != 0).count();
        assert_eq!(open, 0, "{open} unmatched directed edges");
    }
}
