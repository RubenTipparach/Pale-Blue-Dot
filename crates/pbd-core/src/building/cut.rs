//! A plot's pieces cut into meshes and solids (`player-building` design B4):
//! the same [`Meshes`] and [`BuildingSolids`] a town's building is cut into,
//! so the walker, the shadows and the field light take a player's house as
//! they take a town's.
//!
//! The plot is cut in one tangent frame at its anchor cell. Every point is
//! placed from the real cell corners at its layer's real altitude and then
//! read in that frame, so a wall stands on its edge wherever on the sphere
//! the plot is.

use super::{Cells, Opening, Piece, RoofShape, STOREY, Shape};
use crate::column::layer_altitude;
use crate::settlement::pieces::{
    BuildingSolids, Frame, LightKind, MeshBuf, Meshes, RoomLight, SLAB_M, Solid, Surface, WINDOW_M,
};
use crate::settlement::{Kit, Kits};
use glam::{Vec2, Vec3};

/// A ground floor is drawn this far over the ground it covers, metres, so
/// the two never fight.
pub const FLOOR_LIFT_M: f32 = 0.03;
/// How far a roof's eaves reach past its walls, metres.
pub const EAVES_M: f32 = 0.45;
/// A stair's half width, metres, and its risers in a storey.
pub const STAIR_HALF_M: f32 = 0.65;
pub const RISERS: u32 = 15;
/// A window's sill over its storey's floor, metres.
pub const SILL_M: f32 = 1.0;
/// A post's half side, metres.
pub const POST_HALF_M: f32 = 0.13;
/// The texture an unlit flame is drawn in (the towns' `FLAME`).
pub const FLAME: &str = "flame";
/// A footing's stone (`player-building` design 8): the fieldstone kit's
/// wall, under every kit's floor that stands over lower ground.
pub const FOOTING: &str = "fieldstone";
/// How far a footing runs into the ground under it, metres, so no seam
/// shows where the two meet.
pub const FOOTING_SINK_M: f32 = 0.1;

/// A plot cut: its meshes by texture, and its solids for the walker.
#[derive(Clone, Debug)]
pub struct Cut {
    pub meshes: Meshes,
    pub solids: BuildingSolids,
}

/// The tangent frame at `anchor`, its origin at the altitude of `layer`.
pub fn frame_at(cells: &Cells, anchor: u32, layer: u16, radius_m: f32) -> Option<Frame> {
    let cell = cells.cell(anchor)?;
    let y = cell.direction.normalize();
    let toward = cell.corners[0] - y * cell.corners[0].dot(y);
    let x = toward.normalize_or(y.any_orthonormal_vector());
    let z = x.cross(y);
    Some(Frame {
        origin: y * (radius_m + layer_altitude(layer as usize)),
        x,
        y,
        z,
    })
}

struct Cutter<'a> {
    cells: &'a Cells<'a>,
    /// The first layer over the ground in a cell, as the rules read it.
    ground: &'a dyn Fn(u32) -> Option<u16>,
    /// Every floor and stair of the plot, by cell and layer: a floor with
    /// neither under it stands on a footing.
    floors: std::collections::BTreeSet<(u32, u16)>,
    repeat: &'a dyn Fn(&str) -> f32,
    radius_m: f32,
    frame: Frame,
    base: u16,
    meshes: Meshes,
    solids: Vec<Solid>,
    surfaces: Vec<Surface>,
    roof_plan: Vec<Vec2>,
    lights: Vec<RoomLight>,
    top_m: f32,
    reach_m: f32,
}

/// Cut `pieces` in the frame at `anchor`. A piece whose cells are not all in
/// `cells`, or whose kit is unknown, is left out (and is refused before it
/// could be placed). `None` with nothing to cut. No ground is known, so no
/// floor stands on a footing: [`cut_on`].
pub fn cut(
    cells: &Cells,
    anchor: u32,
    pieces: &[&Piece],
    kits: &Kits,
    repeat: &dyn Fn(&str) -> f32,
    radius_m: f32,
) -> Option<Cut> {
    cut_on(cells, anchor, pieces, kits, repeat, radius_m, &|_| None)
}

/// [`cut`] over `ground`, the first layer over the ground in a cell: a
/// floor with no floor of the plot under it, over ground lower than it,
/// stands on a fieldstone footing down to that ground (design 8).
pub fn cut_on(
    cells: &Cells,
    anchor: u32,
    pieces: &[&Piece],
    kits: &Kits,
    repeat: &dyn Fn(&str) -> f32,
    radius_m: f32,
    ground: &dyn Fn(u32) -> Option<u16>,
) -> Option<Cut> {
    let base = pieces.iter().map(|p| p.shape.layer()).min()?;
    let frame = frame_at(cells, anchor, base, radius_m)?;
    // What stands in each cell, by layer: a floor or a stair's foot or head.
    let floors = pieces
        .iter()
        .flat_map(|p| match p.shape {
            Shape::Floor { cell, layer } => vec![(cell, layer)],
            Shape::Stair { foot, head, layer } => vec![(foot, layer), (head, layer)],
            _ => Vec::new(),
        })
        .collect();
    let mut c = Cutter {
        cells,
        ground,
        floors,

        repeat,
        radius_m,
        frame,
        base,
        meshes: Meshes::new(),
        solids: Vec::new(),
        surfaces: Vec::new(),
        roof_plan: Vec::new(),
        lights: Vec::new(),
        top_m: 0.0,
        reach_m: 1.0,
    };
    for piece in pieces {
        let Some(kit) = kits.get(&piece.kit).or_else(|| kits.kits.first()) else {
            continue;
        };
        c.piece(kit, &piece.shape);
    }
    let reach_m = c.reach_m + 1.0;
    Some(Cut {
        meshes: c.meshes,
        solids: BuildingSolids {
            frame: c.frame,
            reach_m,
            solids: c.solids,
            roof_plan: c.roof_plan,
            surfaces: c.surfaces,
            doors: Vec::new(),
            rooms: Meshes::new(),
            lights: c.lights,
            top_m: c.top_m,
        },
    })
}

/// Counter-clockwise as `Surface` and `Solid` read it.
fn ccw(mut outline: Vec<Vec2>) -> Vec<Vec2> {
    let n = outline.len();
    let area: f32 = (0..n)
        .map(|i| outline[i].perp_dot(outline[(i + 1) % n]))
        .sum();
    if area < 0.0 {
        outline.reverse();
    }
    outline
}

impl Cutter<'_> {
    /// A direction at a layer's altitude, in the frame.
    fn at(&self, direction: Vec3, layer: u16) -> Vec3 {
        let alt = layer_altitude(layer as usize);
        self.frame
            .local(direction.normalize() * (self.radius_m + alt))
    }

    /// A direction's plan point and the height of `layer` there.
    fn plan_y(&self, direction: Vec3, layer: u16) -> (Vec2, f32) {
        let p = self.at(direction, layer);
        (Vec2::new(p.x, p.z), p.y)
    }

    fn reach(&mut self, p: Vec2) {
        self.reach_m = self.reach_m.max(p.length());
    }

    fn buf(&mut self, texture: &str) -> &mut MeshBuf {
        self.meshes.entry(texture.to_string()).or_default()
    }

    /// One triangle in the frame, wound to face `normal`, its texture
    /// coordinates given.
    fn tri(&mut self, texture: &str, p: [Vec3; 3], uv: [Vec2; 3], normal: Vec3) {
        let (mut p, mut uv) = (p, uv);
        if (p[1] - p[0]).cross(p[2] - p[0]).dot(normal) < 0.0 {
            p.swap(1, 2);
            uv.swap(1, 2);
        }
        let frame = self.frame;
        let n = (frame.x * normal.x + frame.y * normal.y + frame.z * normal.z).normalize_or_zero();
        let buf = self.buf(texture);
        for k in 0..3 {
            buf.positions.push(frame.world(p[k]).to_array());
            buf.normals.push(n.to_array());
            buf.uvs.push(uv[k].to_array());
        }
    }

    /// A flat face of four points in order round it.
    fn quad(&mut self, texture: &str, p: [Vec3; 4], uv: [Vec2; 4], normal: Vec3) {
        self.tri(texture, [p[0], p[1], p[2]], [uv[0], uv[1], uv[2]], normal);
        self.tri(texture, [p[0], p[2], p[3]], [uv[0], uv[2], uv[3]], normal);
    }

    /// A prism over a convex plan `outline` from `y0` to `y1`: its sides in
    /// `side`, its top and bottom in `cap`. A solid too when `solid`.
    fn prism(&mut self, outline: &[Vec2], y0: f32, y1: f32, side: &str, cap: &str, solid: bool) {
        let outline = ccw(outline.to_vec());
        let n = outline.len();
        let rs = (self.repeat)(side).max(0.1);
        let rc = (self.repeat)(cap).max(0.1);
        let centre = outline.iter().fold(Vec2::ZERO, |s, p| s + *p) / n as f32;
        let mut run = 0.0;
        for i in 0..n {
            let (a, b) = (outline[i], outline[(i + 1) % n]);
            let e = b - a;
            let out = Vec2::new(e.y, -e.x).normalize_or_zero();
            let out = if out.dot((a + b) * 0.5 - centre) < 0.0 {
                -out
            } else {
                out
            };
            let len = e.length();
            self.quad(
                side,
                [
                    Vec3::new(a.x, y0, a.y),
                    Vec3::new(b.x, y0, b.y),
                    Vec3::new(b.x, y1, b.y),
                    Vec3::new(a.x, y1, a.y),
                ],
                [
                    Vec2::new(run / rs, -y0 / rs),
                    Vec2::new((run + len) / rs, -y0 / rs),
                    Vec2::new((run + len) / rs, -y1 / rs),
                    Vec2::new(run / rs, -y1 / rs),
                ],
                Vec3::new(out.x, 0.0, out.y),
            );
            run += len;
            self.reach(a);
        }
        for (y, up) in [(y1, 1.0), (y0, -1.0)] {
            for i in 1..n - 1 {
                let ps = [outline[0], outline[i], outline[i + 1]];
                self.tri(
                    cap,
                    ps.map(|p| Vec3::new(p.x, y, p.y)),
                    ps.map(|p| p / rc),
                    Vec3::Y * up,
                );
            }
        }
        if solid {
            self.solids.push(Solid { outline, y0, y1 });
        }
    }

    /// A box along `a` to `b` in plan, `half` either side, from `y0` to `y1`.
    #[allow(clippy::too_many_arguments)]
    fn slab(&mut self, a: Vec2, b: Vec2, half: f32, y0: f32, y1: f32, side: &str, cap: &str) {
        let along = (b - a).normalize_or_zero();
        let across = Vec2::new(-along.y, along.x) * half;
        self.prism(
            &[a - across, b - across, b + across, a + across],
            y0,
            y1,
            side,
            cap,
            true,
        );
    }

    fn piece(&mut self, kit: &Kit, shape: &Shape) {
        match shape {
            Shape::Wall {
                a,
                b,
                layer,
                opening,
            } => self.wall(kit, *a, *b, *layer, *opening),
            Shape::Floor { cell, layer } => self.floor(kit, *cell, *layer),
            Shape::Stair { foot, head, layer } => self.stair(kit, *foot, *head, *layer),
            Shape::Post { cells, layer } => {
                let Some(corner) = self.cells.corner(*cells) else {
                    return;
                };
                let (p, y0) = self.plan_y(corner, *layer);
                let (_, y1) = self.plan_y(corner, layer + STOREY);
                let h = POST_HALF_M;
                let square = [
                    p + Vec2::new(-h, -h),
                    p + Vec2::new(h, -h),
                    p + Vec2::new(h, h),
                    p + Vec2::new(-h, h),
                ];
                let edge = wall_of(kit, 0).edge.clone();
                self.prism(&square, y0, y1, &edge, &edge, true);
            }
            Shape::Lantern { cell, layer } => self.lantern(kit, *cell, *layer),
            Shape::Roof {
                cells,
                layer,
                shape,
            } => self.roof(kit, cells, *layer, *shape),
        }
    }

    fn wall(&mut self, kit: &Kit, a: u32, b: u32, layer: u16, opening: Opening) {
        let Some((e0, e1)) = self.cells.edge(a, b) else {
            return;
        };
        let storey = usize::from(layer.saturating_sub(self.base) / STOREY);
        let faces = wall_of(kit, storey).clone();
        let mid = (e0 + e1) * 0.5;
        let (p0, _) = self.plan_y(e0, layer);
        let (p1, _) = self.plan_y(e1, layer);
        let (_, y0) = self.plan_y(mid, layer);
        let (_, y1) = self.plan_y(mid, layer + STOREY);
        let half = faces.thickness_m * 0.5;
        // Run past both corners, so walls meeting at an angle close it.
        let along = (p1 - p0).normalize_or_zero();
        let (p0, p1) = (p0 - along * half * 0.6, p1 + along * half * 0.6);
        let len = p0.distance(p1);
        let at = |u: f32| p0 + along * u;
        let (side, cap) = (faces.outside.as_str(), faces.edge.as_str());
        match opening {
            Opening::None => self.slab(p0, p1, half, y0, y1, side, cap),
            Opening::Door | Opening::Window => {
                let (w, h, sill) = match opening {
                    Opening::Door => (kit.door_m.0, kit.door_m.1, 0.0),
                    _ => {
                        let (w, h) = kit.window_m.unwrap_or(WINDOW_M);
                        (w, h, SILL_M)
                    }
                };
                let (u0, u1) = ((len - w) * 0.5, (len + w) * 0.5);
                self.slab(at(0.0), at(u0), half, y0, y1, side, cap);
                self.slab(at(u1), at(len), half, y0, y1, side, cap);
                self.slab(at(u0), at(u1), half, y0 + sill + h, y1, side, cap);
                if sill > 0.0 {
                    self.slab(at(u0), at(u1), half, y0, y0 + sill, side, cap);
                }
            }
        }
    }

    fn floor(&mut self, kit: &Kit, cell: u32, layer: u16) {
        let Some(c) = self.cells.cell(cell) else {
            return;
        };
        let (_, top) = self.plan_y(c.direction, layer);
        let top = top + FLOOR_LIFT_M;
        let outline: Vec<Vec2> = c.corners.iter().map(|&k| self.plan_y(k, layer).0).collect();
        let edge = wall_of(kit, 0).edge.clone();
        // A footing down to lower ground, where no floor of the plot is
        // under this one (design 8).
        let below = self.floors.iter().any(|&(c2, l)| c2 == cell && l < layer);
        if !below && let Some(g) = (self.ground)(cell).filter(|&g| g < layer) {
            let (_, foot) = self.plan_y(c.direction, g);
            self.prism(
                &outline,
                foot - FOOTING_SINK_M,
                top - SLAB_M,
                FOOTING,
                FOOTING,
                true,
            );
        }
        self.prism(&outline, top - SLAB_M, top, &edge, &kit.floor, false);
        self.surfaces.push(Surface::Floor {
            outline: ccw(outline),
            top,
            bottom: top - SLAB_M,
        });
    }

    fn stair(&mut self, kit: &Kit, foot: u32, head: u32, layer: u16) {
        let (Some(f), Some(h)) = (self.cells.cell(foot), self.cells.cell(head)) else {
            return;
        };
        let (Some(sf), Some(sh)) = (self.cells.side(foot, head), self.cells.side(head, foot))
        else {
            return;
        };
        let mid = |cell: &crate::topology::DualCell, s: usize| {
            let n = cell.corners.len();
            (cell.corners[s % n] + cell.corners[(s + 1) % n]) * 0.5
        };
        let (start, base) = self.plan_y(mid(f, sf + 3), layer);
        let (end, _) = self.plan_y(mid(h, sh + 3), layer);
        let base = base + FLOOR_LIFT_M;
        let dir = (end - start).normalize_or_zero();
        let len = start.distance(end);
        let rise = (self.plan_y(f.direction, layer + STOREY).1 - self.plan_y(f.direction, layer).1)
            / RISERS as f32;
        self.surfaces.push(Surface::Flight {
            foot: start,
            dir,
            len,
            half_width: STAIR_HALF_M,
            base,
            rise,
            risers: RISERS,
        });
        let run = len / RISERS as f32;
        let edge = wall_of(kit, 0).edge.clone();
        for i in 0..RISERS {
            let a = start + dir * (run * i as f32);
            let b = start + dir * (run * (i + 1) as f32);
            let y1 = base + rise * (i + 1) as f32;
            let across = Vec2::new(-dir.y, dir.x) * STAIR_HALF_M;
            self.prism(
                &[a - across, b - across, b + across, a + across],
                base - SLAB_M,
                y1,
                &edge,
                &kit.floor,
                false,
            );
        }
    }

    fn lantern(&mut self, kit: &Kit, cell: u32, layer: u16) {
        let Some(c) = self.cells.cell(cell) else {
            return;
        };
        let (p, y) = self.plan_y(c.direction, layer);
        let y = y + FLOOR_LIFT_M;
        let edge = wall_of(kit, 0).edge.clone();
        let post = |h: f32| {
            [
                p + Vec2::new(-h, -h),
                p + Vec2::new(h, -h),
                p + Vec2::new(h, h),
                p + Vec2::new(-h, h),
            ]
        };
        self.prism(&post(0.05), y, y + 1.3, &edge, &edge, false);
        self.prism(&post(0.14), y + 1.3, y + 1.34, "iron", "iron", false);
        self.prism(&post(0.09), y + 1.34, y + 1.56, FLAME, FLAME, false);
        self.prism(&post(0.15), y + 1.56, y + 1.62, "iron", "iron", false);
        self.lights.push(RoomLight {
            kind: LightKind::Sconce,
            at: self.frame.world(Vec3::new(p.x, y + 1.45, p.y)),
            power: 0.9,
            below_m: 1.5,
            above_m: 1.7,
        });
    }

    fn roof(&mut self, kit: &Kit, keys: &[u32], layer: u16, shape: RoofShape) {
        let cells: Vec<&crate::topology::DualCell> =
            keys.iter().filter_map(|&k| self.cells.cell(k)).collect();
        if cells.is_empty() {
            return;
        }
        let y0 = keys
            .iter()
            .filter_map(|&k| self.cells.cell(k))
            .map(|c| self.plan_y(c.direction, layer).1)
            .fold(f32::MIN, f32::max);
        let centres: Vec<Vec2> = cells
            .iter()
            .map(|c| self.plan_y(c.direction, layer).0)
            .collect();
        let centroid = centres.iter().fold(Vec2::ZERO, |s, p| s + *p) / centres.len() as f32;
        // A corner on the footprint's rim is pushed out by the eaves, away
        // from the middle, so a corner two cells share moves as one.
        let rim = |key: u32, k: usize| -> bool {
            self.cells
                .corner_cells(key, k)
                .is_none_or(|three| three.iter().any(|c| !keys.contains(c)))
        };
        let mut corners: Vec<Vec<Vec2>> = Vec::new();
        for (&key, cell) in keys.iter().zip(&cells) {
            let ring = (0..cell.corners.len())
                .map(|k| {
                    let p = self.plan_y(cell.corners[k], layer).0;
                    if rim(key, k) {
                        p + (p - centroid).normalize_or_zero() * EAVES_M
                    } else {
                        p
                    }
                })
                .collect();
            corners.push(ring);
        }
        // The ridge: along the footprint's long axis.
        let mut cov = [0.0f32; 3];
        for p in &centres {
            let d = *p - centroid;
            cov[0] += d.x * d.x;
            cov[1] += d.x * d.y;
            cov[2] += d.y * d.y;
        }
        let angle = 0.5 * (2.0 * cov[1]).atan2(cov[0] - cov[2]);
        let axis = Vec2::new(angle.cos(), angle.sin());
        let all: Vec<Vec2> = corners.iter().flatten().copied().collect();
        let span = all
            .iter()
            .map(|p| (*p - centroid).perp_dot(axis).abs())
            .fold(0.0f32, f32::max);
        let radius = all
            .iter()
            .map(|p| p.distance(centroid))
            .fold(0.0f32, f32::max);
        let height = |p: Vec2| -> f32 {
            match shape {
                RoofShape::Gable => y0 + kit.pitch * (span - (p - centroid).perp_dot(axis).abs()),
                RoofShape::Cone => y0 + kit.pitch.max(1.2) * (radius - p.distance(centroid)),
                RoofShape::Flat => y0 + 0.25,
            }
        };
        let material = kit.roof_material.clone();
        let rr = (self.repeat)(&material).max(0.1);
        for (cell, ring) in cells.iter().zip(&corners) {
            let centre = self.plan_y(cell.direction, layer).0;
            let n = ring.len();
            for k in 0..n {
                let ps = [centre, ring[k], ring[(k + 1) % n]];
                let top = ps.map(|p| Vec3::new(p.x, height(p), p.y));
                let uv = ps.map(|p| p / rr);
                self.tri(&material, top, uv, Vec3::Y);
                let under = top.map(|p| p - Vec3::Y * 0.08);
                self.tri(&material, under, uv, -Vec3::Y);
            }
        }
        // Gable ends and the band over the walls: on every rim edge, from
        // the wall top up to the roof.
        let gable = kit.gable.clone();
        let rg = (self.repeat)(&gable).max(0.1);
        for (&key, cell) in keys.iter().zip(&cells) {
            let n = cell.corners.len();
            for (k, other) in self.cells.neighbours(key).into_iter().enumerate().take(n) {
                if other.is_some_and(|o| keys.contains(&o)) {
                    continue;
                }
                let a = self.plan_y(cell.corners[k], layer).0;
                let b = self.plan_y(cell.corners[(k + 1) % n], layer).0;
                let (ha, hb) = (height(a) - 0.08, height(b) - 0.08);
                if ha < y0 + 0.02 && hb < y0 + 0.02 {
                    continue;
                }
                let e = b - a;
                let out = Vec2::new(e.y, -e.x).normalize_or_zero();
                let out = if out.dot((a + b) * 0.5 - centroid) < 0.0 {
                    -out
                } else {
                    out
                };
                let len = e.length();
                self.quad(
                    &gable,
                    [
                        Vec3::new(a.x, y0, a.y),
                        Vec3::new(b.x, y0, b.y),
                        Vec3::new(b.x, hb, b.y),
                        Vec3::new(a.x, ha, a.y),
                    ],
                    [
                        Vec2::new(0.0, -y0 / rg),
                        Vec2::new(len / rg, -y0 / rg),
                        Vec2::new(len / rg, -hb / rg),
                        Vec2::new(0.0, -ha / rg),
                    ],
                    Vec3::new(out.x, 0.0, out.y),
                );
            }
        }
        let mut plan = self.roof_plan.clone();
        plan.extend(all);
        self.roof_plan = hull(&plan);
        self.top_m = self.top_m.max(y0);
        for p in self.roof_plan.clone() {
            self.reach(p);
        }
    }
}

/// A kit's wall for a storey, the last repeating.
fn wall_of(kit: &Kit, storey: usize) -> &crate::settlement::WallFaces {
    &kit.walls[storey.min(kit.walls.len() - 1)]
}

/// The convex hull of plan points, counter-clockwise.
pub fn hull(points: &[Vec2]) -> Vec<Vec2> {
    let mut p: Vec<Vec2> = points.to_vec();
    p.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
    p.dedup_by(|a, b| a.distance(*b) < 1e-4);
    if p.len() < 3 {
        return p;
    }
    let cross = |o: Vec2, a: Vec2, b: Vec2| (a - o).perp_dot(b - o);
    let mut lower: Vec<Vec2> = Vec::new();
    for &q in &p {
        while lower.len() >= 2 && cross(lower[lower.len() - 2], lower[lower.len() - 1], q) <= 0.0 {
            lower.pop();
        }
        lower.push(q);
    }
    let mut upper: Vec<Vec2> = Vec::new();
    for &q in p.iter().rev() {
        while upper.len() >= 2 && cross(upper[upper.len() - 2], upper[upper.len() - 1], q) <= 0.0 {
            upper.pop();
        }
        upper.push(q);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}
