//! A building cut from its cells' real corners (`tenebris-towns` sections 2
//! and 6): the mockup's `building()`, `edgeWall()`, `gableRoof()` and
//! `pyramidRoof()` (`docs/mockups/towns.html`), on the planet's cells.
//!
//! Each building is cut in its own tangent frame: origin on its terrace
//! under its centre, `y` up along the radius there, `x` along the layout's
//! rows (the mockup's direction 0), and `z = x x y`. Every corner is the real
//! cell corner, projected into that plane: over a house's few metres the
//! sphere falls away by millimetres, so the pieces meet as they do on the
//! mockup's flat grid. The mesh comes out in planet-local metres.

use super::chart::{Chart, Patch};
use super::{BuildingDef, HUT_STOREY_M, Kit, RoofKind, STOREY_M, neighbour};
use glam::{Vec2, Vec3};
use std::collections::BTreeMap;

/// Floor boards and joists, metres.
pub const SLAB_M: f32 = 0.2;
/// A floor is drawn this far over where it stands, so a wall top in its
/// plane never fights it (the mockup's ZLIFT).
pub const LIFT_M: f32 = 0.01;
/// A window, wide by high, on a one-metre sill, where the kit does not say.
pub const WINDOW_M: (f32, f32) = (0.8, 1.0);

/// Triangles in one texture: planet-local positions, normals, and texture
/// coordinates in repeats (the sampler repeats).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MeshBuf {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
}

/// A town's meshes, by texture name.
pub type Meshes = BTreeMap<String, MeshBuf>;

/// A building's tangent frame.
#[derive(Clone, Copy, Debug)]
pub struct Frame {
    pub origin: Vec3,
    pub x: Vec3,
    pub y: Vec3,
    pub z: Vec3,
}

impl Frame {
    pub fn world(&self, p: Vec3) -> Vec3 {
        self.origin + self.x * p.x + self.y * p.y + self.z * p.z
    }

    /// A planet-local point in the frame.
    pub fn local(&self, p: Vec3) -> Vec3 {
        let d = p - self.origin;
        Vec3::new(d.dot(self.x), d.dot(self.y), d.dot(self.z))
    }

    fn world_dir(&self, n: Vec3) -> Vec3 {
        (self.x * n.x + self.y * n.y + self.z * n.z).normalize_or_zero()
    }

    /// A direction's point in the frame's plane, as (x, z).
    pub fn plane(&self, direction: Vec3) -> Vec2 {
        let d = direction.normalize_or_zero();
        let along = d.dot(self.y);
        let w = d * (self.origin.length() / along.max(1e-6)) - self.origin;
        Vec2::new(w.dot(self.x), w.dot(self.z))
    }
}

/// A thin solid (`tenebris-towns` section 4): a convex outline in its
/// building's plan, counter-clockwise, over a height range, metres in the
/// building's frame.
#[derive(Clone, Debug, PartialEq)]
pub struct Solid {
    pub outline: Vec<Vec2>,
    pub y0: f32,
    pub y1: f32,
}

impl Solid {
    /// Whether a body of `radius`, its centre at `(x, z)` in plan and its
    /// feet and head at `feet` and `head`, is in the solid: its outline
    /// grown by the radius holds the centre, the heights cross, and the feet
    /// are not already on the solid's top.
    pub fn holds(&self, x: f32, z: f32, feet: f32, head: f32, radius: f32) -> bool {
        if head <= self.y0 || feet >= self.y1 - 0.03 {
            return false;
        }
        self.covers(x, z, radius)
    }

    /// Whether the outline, grown by `radius`, holds `(x, z)` in plan.
    pub fn covers(&self, x: f32, z: f32, radius: f32) -> bool {
        let p = Vec2::new(x, z);
        let n = self.outline.len();
        let mut inside = true;
        let mut nearest = f32::MAX;
        for i in 0..n {
            let (a, b) = (self.outline[i], self.outline[(i + 1) % n]);
            let e = b - a;
            if e.perp_dot(p - a) < 0.0 {
                inside = false;
            }
            let t = ((p - a).dot(e) / e.length_squared().max(1e-9)).clamp(0.0, 1.0);
            nearest = nearest.min((a + e * t).distance(p));
        }
        inside || nearest < radius
    }
}

/// One building's solids, in its frame.
#[derive(Clone, Debug)]
pub struct BuildingSolids {
    pub frame: Frame,
    /// The plan's extent from the frame's origin, metres, for a quick miss.
    pub reach_m: f32,
    pub solids: Vec<Solid>,
    /// The roof's plan, eaves included, in the frame: what no other roof
    /// may overlap (`tenebris-towns` section 2).
    pub roof_plan: Vec<Vec2>,
}

impl BuildingSolids {
    /// Whether a body centred at `centre` (planet-local), `half_height`
    /// tall each way and `radius` round, is in any solid.
    pub fn holds(&self, centre: Vec3, half_height: f32, radius: f32) -> bool {
        let p = self.frame.local(centre);
        if Vec2::new(p.x, p.z).length() > self.reach_m + radius {
            return false;
        }
        self.solids
            .iter()
            .any(|s| s.holds(p.x, p.z, p.y - half_height, p.y + half_height, radius))
    }

    /// What a body centred at `centre` (planet-local), `radius` round, meets
    /// rising: the lowest underside of a solid over its plan that is above
    /// its middle, as a planet-local radius. An upper floor's slab, a
    /// door's lintel (the design's slice 2a).
    pub fn ceiling(&self, centre: Vec3, radius: f32) -> Option<f32> {
        let p = self.frame.local(centre);
        if Vec2::new(p.x, p.z).length() > self.reach_m + radius {
            return None;
        }
        self.solids
            .iter()
            .filter(|s| s.y0 >= p.y && s.covers(p.x, p.z, radius))
            .map(|s| self.frame.world(Vec3::new(p.x, s.y0, p.z)).length())
            .min_by(f32::total_cmp)
    }
}

/// Where the meshes and the solids are written, and how a texture repeats.
pub struct Sink<'a> {
    pub meshes: &'a mut Meshes,
    /// Metres one repeat of a texture covers (the manifest's `repeat_m`).
    pub repeat_m: &'a dyn Fn(&str) -> f32,
    pub frame: Frame,
    pub solids: Vec<Solid>,
    pub roof_plan: Vec<Vec2>,
}

impl Sink<'_> {
    /// A convex planar polygon in frame coordinates, turned to face `want`,
    /// with texture coordinates from `uv` or, by default, the mockup's
    /// `uvWorld`: floors by their plan, walls by their run and height.
    fn face(
        &mut self,
        material: &str,
        pts: &[Vec3],
        want: Vec3,
        uv: Option<&dyn Fn(Vec3) -> Vec2>,
    ) {
        if pts.len() < 3 {
            return;
        }
        let mut n = (pts[1] - pts[0]).cross(pts[2] - pts[0]).normalize_or_zero();
        if n == Vec3::ZERO {
            return;
        }
        let mut pts = pts.to_vec();
        if n.dot(want) < 0.0 {
            pts.reverse();
            n = -n;
        }
        let rep = (self.repeat_m)(material);
        let uv_of = |p: Vec3| -> [f32; 2] {
            let t = match uv {
                Some(f) => f(p),
                None => uv_world(p, n, rep),
            };
            // A texture's first row is its top in the game and its bottom in
            // the mockup's canvas.
            [t.x, -t.y]
        };
        let world_n = self.frame.world_dir(n).to_array();
        let buf = self.meshes.entry(material.to_string()).or_default();
        for i in 1..pts.len() - 1 {
            for &k in &[0, i, i + 1] {
                buf.positions.push(self.frame.world(pts[k]).to_array());
                buf.normals.push(world_n);
                buf.uvs.push(uv_of(pts[k]));
            }
        }
    }

    /// An oriented box: centre `(x, z)` on the floor `y0`, `sx` along the
    /// heading `ang` (radians from `x` toward `z`), `sy` high, `sz` across.
    /// Each face's texture is `pick` of its outward normal.
    #[allow(clippy::too_many_arguments)]
    fn boxed(
        &mut self,
        pick: &dyn Fn(Vec3) -> String,
        x: f32,
        y0: f32,
        z: f32,
        sx: f32,
        sy: f32,
        sz: f32,
        ang: f32,
        uv: Option<&dyn Fn(Vec3, Vec3) -> Vec2>,
    ) {
        let (ca, sa) = (ang.cos(), ang.sin());
        let ex = Vec3::new(ca, 0.0, sa);
        let ez = Vec3::new(-sa, 0.0, ca);
        let c = Vec3::new(x, y0, z);
        let (hx, hz) = (sx / 2.0, sz / 2.0);
        let p = |a: f32, b: f32, h: f32| c + ex * a + ez * b + Vec3::Y * h;
        let quads = [
            (
                ex,
                [
                    p(hx, hz, 0.0),
                    p(hx, -hz, 0.0),
                    p(hx, -hz, sy),
                    p(hx, hz, sy),
                ],
            ),
            (
                -ex,
                [
                    p(-hx, -hz, 0.0),
                    p(-hx, hz, 0.0),
                    p(-hx, hz, sy),
                    p(-hx, -hz, sy),
                ],
            ),
            (
                ez,
                [
                    p(-hx, hz, 0.0),
                    p(hx, hz, 0.0),
                    p(hx, hz, sy),
                    p(-hx, hz, sy),
                ],
            ),
            (
                -ez,
                [
                    p(hx, -hz, 0.0),
                    p(-hx, -hz, 0.0),
                    p(-hx, -hz, sy),
                    p(hx, -hz, sy),
                ],
            ),
            (
                Vec3::Y,
                [
                    p(-hx, -hz, sy),
                    p(hx, -hz, sy),
                    p(hx, hz, sy),
                    p(-hx, hz, sy),
                ],
            ),
            (
                -Vec3::Y,
                [
                    p(-hx, -hz, 0.0),
                    p(hx, -hz, 0.0),
                    p(hx, hz, 0.0),
                    p(-hx, hz, 0.0),
                ],
            ),
        ];
        for (n, pts) in quads {
            let material = pick(n);
            match uv {
                Some(f) => {
                    let g = |q: Vec3| f(q, n);
                    self.face(&material, &pts, n, Some(&g));
                }
                None => self.face(&material, &pts, n, None),
            }
        }
    }

    /// An oriented box's outline, as a solid, `sx` along `ang` and `sz`
    /// across.
    fn solid_box(&mut self, x: f32, y0: f32, z: f32, s: Vec3, ang: f32) {
        let (ca, sa) = (ang.cos(), ang.sin());
        let (hx, hz) = (s.x / 2.0, s.z / 2.0);
        let outline = [(-hx, -hz), (hx, -hz), (hx, hz), (-hx, hz)]
            .map(|(a, b)| Vec2::new(x + a * ca - b * sa, z + a * sa + b * ca))
            .to_vec();
        self.solids.push(Solid {
            outline: ccw(outline),
            y0,
            y1: y0 + s.y,
        });
    }

    fn plain_box(&mut self, material: &str, x: f32, y0: f32, z: f32, s: Vec3, ang: f32) {
        let m = material.to_string();
        self.boxed(&move |_| m.clone(), x, y0, z, s.x, s.y, s.z, ang, None);
    }

    /// A prism over a convex polygon in plan, from `y0` to `y1`.
    fn prism(
        &mut self,
        top: &str,
        side: &str,
        pts: &[Vec2],
        y0: f32,
        y1: f32,
        bottom: Option<&str>,
    ) {
        let up: Vec<Vec3> = pts.iter().map(|p| Vec3::new(p.x, y1, p.y)).collect();
        self.face(top, &up, Vec3::Y, None);
        if let Some(bottom) = bottom {
            let down: Vec<Vec3> = pts.iter().map(|p| Vec3::new(p.x, y0, p.y)).collect();
            self.face(bottom, &down, -Vec3::Y, None);
        }
        let centre = pts.iter().fold(Vec2::ZERO, |s, p| s + *p) / pts.len() as f32;
        for i in 0..pts.len() {
            let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
            let mid = (a + b) * 0.5 - centre;
            self.face(
                side,
                &[
                    Vec3::new(a.x, y0, a.y),
                    Vec3::new(b.x, y0, b.y),
                    Vec3::new(b.x, y1, b.y),
                    Vec3::new(a.x, y1, a.y),
                ],
                Vec3::new(mid.x, 0.0, mid.y),
                None,
            );
        }
    }
}

/// A convex outline turned counter-clockwise in plan, as `Solid::holds`
/// reads it.
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

/// The mockup's `uvWorld`: a floor by its plan, a wall by its run and its
/// height, `rep` metres a repeat.
fn uv_world(p: Vec3, n: Vec3, rep: f32) -> Vec2 {
    let s = 1.0 / rep.max(0.1);
    if n.y.abs() > 0.7 {
        return Vec2::new(p.x * s, p.z * s);
    }
    let l = n.x.hypot(n.z).max(1e-6);
    Vec2::new((p.x * -n.z + p.z * n.x) / l * s, p.y * s)
}

/// An opening in a wall: a door or a window, `w` wide from `yb` to `ye`.
#[derive(Clone, Copy, Debug)]
struct Opening {
    yb: f32,
    ye: f32,
    w: f32,
    door: bool,
    sill: bool,
    shutters: bool,
}

/// One building's cells as charted: each layout cell, its patch cell, and
/// its corners in the building's frame.
struct Plan {
    cells: Vec<(i32, i32)>,
    corners: Vec<[Vec2; 6]>,
    centres: Vec<Vec2>,
}

impl Plan {
    fn index(&self, c: i32, r: i32) -> Option<usize> {
        self.cells.iter().position(|&x| x == (c, r))
    }
}

/// Cut one building. `terrace_m` is the height its ground floor stands at,
/// metres over `radius_m`.
#[allow(clippy::too_many_arguments)]
pub fn cut_building(
    meshes: &mut Meshes,
    repeat_m: &dyn Fn(&str) -> f32,
    patch: &Patch,
    chart: &Chart,
    def: &BuildingDef,
    kit: &Kit,
    radius_m: f32,
    terrace_m: f32,
) -> Result<BuildingSolids, String> {
    let charted: Vec<(i32, i32, usize, usize)> = def
        .cells
        .iter()
        .map(|&[c, r]| {
            let at = chart
                .cells
                .get(&(c, r))
                .ok_or_else(|| format!("{}: cell ({c}, {r}) is not charted", def.name))?;
            Ok((c, r, at.cell, at.d0))
        })
        .collect::<Result<_, String>>()?;
    let centre = charted
        .iter()
        .fold(Vec3::ZERO, |s, x| s + patch.cells[x.2].direction)
        .normalize();
    let y = centre;
    // x along the rows: the steps from each cell to the next one along its
    // row inside the building, centre to centre, as the rows actually run.
    // A lone cell (a hut) takes its own direction-0 edge. One distorted
    // hexagon's edges are off its row by a few degrees, which over a house
    // two rows deep grew its roof's box past the plan (the owner's "roof
    // should not intersect like that yo").
    let mut along = Vec3::ZERO;
    for &(c, r, index, _) in &charted {
        let (c2, r2) = neighbour(c, r, 0);
        if let Some(&(_, _, next, _)) = charted.iter().find(|x| x.0 == c2 && x.1 == r2) {
            along += patch.cells[next].direction - patch.cells[index].direction;
        }
    }
    if along.length_squared() < 1e-12 {
        let (_, _, first, d0) = charted[0];
        let cell = &patch.cells[first];
        along = (cell.corners[d0] + cell.corners[(d0 + 1) % 6]) * 0.5 - cell.direction;
    }
    let x = (along - y * along.dot(y)).normalize();
    let z = x.cross(y);
    let frame = Frame {
        origin: y * (radius_m + terrace_m),
        x,
        y,
        z,
    };
    // Each cell's corners in the order of the mockup's `corner(c, r, k)`:
    // corner k is where the mockup's edges k - 1 and k meet, so edge d runs
    // from corner d to corner d + 1.
    let mut plan = Plan {
        cells: Vec::new(),
        corners: Vec::new(),
        centres: Vec::new(),
    };
    for &(c, r, index, _) in &charted {
        let cell = &patch.cells[index];
        let mut corners = [Vec2::ZERO; 6];
        for (d, corner) in corners.iter_mut().enumerate() {
            // Edge d is the patch's side s, from its corner s to s + 1; the
            // mockup walks its edges the other way round, so its edge d runs
            // from the patch's corner s + 1 to s.
            let s = chart.side(c, r, d).expect("charted");
            *corner = frame.plane(cell.corners[(s + 1) % 6]);
        }
        plan.cells.push((c, r));
        plan.corners.push(corners);
        plan.centres.push(frame.plane(cell.direction));
    }
    let reach_m = plan
        .corners
        .iter()
        .flatten()
        .map(|p| p.length())
        .fold(0.0f32, f32::max)
        + 1.0;
    let mut sink = Sink {
        meshes,
        repeat_m,
        frame,
        solids: Vec::new(),
        roof_plan: Vec::new(),
    };
    cut(&mut sink, &plan, def, kit);
    Ok(BuildingSolids {
        frame,
        reach_m,
        solids: sink.solids,
        roof_plan: sink.roof_plan,
    })
}

fn edge_ends(plan: &Plan, i: usize, d: usize) -> (Vec2, Vec2) {
    (plan.corners[i][d % 6], plan.corners[i][(d + 1) % 6])
}

/// A corner post where walls meet: its place in plan, its bottom and top,
/// its wall's thickness and its material.
type Post = (Vec2, f32, f32, f32, String);

fn cut(sink: &mut Sink, plan: &Plan, def: &BuildingDef, kit: &Kit) {
    let storeys = def.storeys.max(1);
    let storey_m = if kit.hut {
        HUT_STOREY_M
    } else {
        STOREY_M * def.tall.max(1) as f32
    };
    let top = storeys as f32 * storey_m;
    let (door_w, door_h) = kit.door_m;
    let inside = |c: i32, r: i32| plan.index(c, r).is_some();
    // Floors: the ground floor's boards over the terrace, and a slab and a
    // beam under every floor above.
    for (i, &(_, _)) in plan.cells.iter().enumerate() {
        let hex: Vec<Vec2> = plan.corners[i].to_vec();
        sink.prism(&kit.floor, &kit.floor, &hex, -0.05, LIFT_M, None);
        if !kit.hut {
            for s in 1..storeys {
                let fy = s as f32 * storey_m;
                sink.prism(
                    "plank",
                    "timber",
                    &hex,
                    fy - SLAB_M,
                    fy + LIFT_M,
                    Some("plank"),
                );
                // The slab is a ceiling to the storey under it.
                sink.solids.push(Solid {
                    outline: ccw(hex.clone()),
                    y0: fy - SLAB_M,
                    y1: fy + LIFT_M,
                });
            }
            for s in 1..=storeys {
                let fy = s as f32 * storey_m;
                let c = plan.centres[i];
                let run = (plan.corners[i][0] - plan.corners[i][3]).length();
                sink.plain_box(
                    "timber",
                    c.x,
                    fy - SLAB_M - 0.14,
                    c.y,
                    Vec3::new(run, 0.14, 0.16),
                    0.0,
                );
            }
        }
    }
    // Walls on every edge the building shares with the outside, storey by
    // storey, with their doors and windows; a post where walls meet.
    let mut posts: BTreeMap<(i32, i32), Post> = BTreeMap::new();
    for (i, &(c, r)) in plan.cells.iter().enumerate() {
        for d in 0..6 {
            let (c2, r2) = neighbour(c, r, d);
            if inside(c2, r2) {
                continue;
            }
            for s in 0..storeys {
                let wall = &kit.walls[(s as usize).min(kit.walls.len() - 1)];
                let y0 = s as f32 * storey_m;
                let y1 = y0 + storey_m;
                let at = [c, r, d as i32, s as i32];
                let mut openings = Vec::new();
                if def.doors.contains(&at) {
                    openings.push(Opening {
                        yb: y0,
                        ye: y0 + door_h,
                        w: door_w,
                        door: true,
                        sill: false,
                        shutters: false,
                    });
                } else if def.windows.contains(&at) {
                    if def.tall > 1 {
                        openings.push(Opening {
                            yb: y0 + 1.6,
                            ye: y0 + 4.4,
                            w: 0.9,
                            door: false,
                            sill: true,
                            shutters: false,
                        });
                    } else if let Some((w, h)) = kit.window_m {
                        openings.push(Opening {
                            yb: y0 + 1.0,
                            ye: y0 + 1.0 + h,
                            w,
                            door: false,
                            sill: !kit.hut,
                            shutters: false,
                        });
                    } else if !kit.hut {
                        openings.push(Opening {
                            yb: y0 + 1.0,
                            ye: y0 + 1.0 + WINDOW_M.1,
                            w: WINDOW_M.0,
                            door: false,
                            sill: true,
                            shutters: true,
                        });
                    }
                }
                let (a, b) = edge_ends(plan, i, d);
                edge_wall(sink, plan.centres[i], a, b, y0, y1, wall, &openings);
                for p in [a, b] {
                    let key = ((p.x * 20.0).round() as i32, (p.y * 20.0).round() as i32);
                    let e = posts.entry(key).or_insert((
                        p,
                        y0,
                        y1,
                        wall.thickness_m,
                        wall.edge.clone(),
                    ));
                    e.1 = e.1.min(y0);
                    e.2 = e.2.max(y1);
                    e.3 = e.3.max(wall.thickness_m);
                }
            }
        }
    }
    for (p, y0, y1, t, m) in posts.values() {
        let hex: Vec<Vec2> = (0..6)
            .map(|k| {
                let a = k as f32 * std::f32::consts::FRAC_PI_3;
                *p + Vec2::new(a.cos(), a.sin()) * t * 0.6
            })
            .collect();
        sink.prism(m, m, &hex, *y0, *y1, None);
        sink.solids.push(Solid {
            outline: ccw(hex.clone()),
            y0: *y0,
            y1: *y1,
        });
    }
    // The roof, over the footprint's box in the frame: the plan's extent
    // along the rows and across them.
    let (mut min, mut max) = (Vec2::splat(f32::MAX), Vec2::splat(f32::MIN));
    for corners in &plan.corners {
        for p in corners {
            min = min.min(*p);
            max = max.max(*p);
        }
    }
    let cone = matches!(def.roof.as_str(), "cone") && plan.cells.len() == 1;
    let roof_kind = if cone {
        RoofKind::Cone
    } else if def.roof == "flat" {
        RoofKind::Flat
    } else {
        RoofKind::Gable
    };
    let material = if matches!(def.roof.as_str(), "cone" | "flat" | "dome") {
        kit.roof_material.clone()
    } else {
        def.roof.clone()
    };
    let overhang = if kit.hut { 0.55 } else { 0.45 };
    sink.roof_plan = match roof_kind {
        RoofKind::Cone => plan.corners[0]
            .iter()
            .map(|p| *p + (*p - plan.centres[0]).normalize_or_zero() * overhang)
            .collect(),
        _ => vec![
            Vec2::new(min.x - overhang, min.y - overhang),
            Vec2::new(max.x + overhang, min.y - overhang),
            Vec2::new(max.x + overhang, max.y + overhang),
            Vec2::new(min.x - overhang, max.y + overhang),
        ],
    };
    match roof_kind {
        RoofKind::Cone => {
            cone_roof(
                sink,
                plan.centres[0],
                &plan.corners[0],
                top,
                2.6,
                &material,
                overhang,
            );
        }
        RoofKind::Flat => {
            for corners in &plan.corners {
                sink.prism(
                    &material,
                    &kit.walls[0].outside,
                    corners,
                    top,
                    top + 0.3,
                    None,
                );
            }
        }
        RoofKind::Gable => {
            let under = if kit.hut { "thatch" } else { "plank" };
            gable_roof(
                sink, min, max, top, def.pitch, &material, &kit.gable, overhang, under,
            );
        }
    }
    // The chimney: up through the roof at its cell.
    if let (Some([c, r]), Some(m)) = (def.chimney, kit.chimney.as_ref())
        && let Some(i) = plan.index(c, r)
    {
        let p = plan.centres[i];
        let h = (max.y - min.y) / 2.0 * def.pitch + 1.2;
        sink.plain_box(m, p.x, top, p.y, Vec3::new(0.8, h, 0.8), 0.0);
        sink.solid_box(p.x, top, p.y, Vec3::new(0.8, h, 0.8), 0.0);
    }
}

/// The mockup's `edgeWall`: a wall centred on the edge from `a` to `b`,
/// `y0` to `y1` high, cut round its openings; its outside faces away from
/// the cell's centre `inner`.
#[allow(clippy::too_many_arguments)]
fn edge_wall(
    sink: &mut Sink,
    inner: Vec2,
    a: Vec2,
    b: Vec2,
    y0: f32,
    y1: f32,
    wall: &super::WallFaces,
    openings: &[Opening],
) {
    let t = wall.thickness_m;
    let len = (b - a).length();
    let tan = (b - a) / len;
    let mid = (a + b) * 0.5;
    let mut out = Vec2::new(-tan.y, tan.x);
    if out.dot(mid - inner) < 0.0 {
        out = -out;
    }
    let ang = tan.y.atan2(tan.x);
    let (outside, inside_m, edge) = (wall.outside.clone(), wall.inside.clone(), wall.edge.clone());
    let pick = move |n: Vec3| -> String {
        let s = n.x * out.x + n.z * out.y;
        if s > 0.5 {
            outside.clone()
        } else if s < -0.5 {
            inside_m.clone()
        } else {
            edge.clone()
        }
    };
    let per_face = wall.per_face;
    // Half-timbering is one picture across the face, a storey high.
    let face_uv = move |p: Vec3, n: Vec3| -> Vec2 {
        let s = n.x * out.x + n.z * out.y;
        if per_face && n.y.abs() < 0.5 && s > 0.5 {
            let along = (Vec2::new(p.x, p.z) - a).dot(tan) / len;
            Vec2::new(along, ((p.y - y0) / STOREY_M).clamp(0.0, 1.0))
        } else {
            uv_world(p, n, 2.0)
        }
    };
    let seg = |sink: &mut Sink, along: f32, run: f32, lo: f32, hi: f32| {
        if hi - lo < 0.005 || run < 0.005 {
            return;
        }
        let c = mid + tan * along;
        let uv: Option<&dyn Fn(Vec3, Vec3) -> Vec2> = if per_face { Some(&face_uv) } else { None };
        sink.boxed(&pick, c.x, lo, c.y, run, hi - lo, t, ang, uv);
        sink.solid_box(c.x, lo, c.y, Vec3::new(run, hi - lo, t), ang);
    };
    let mut ops = openings.to_vec();
    ops.sort_by(|p, q| p.yb.total_cmp(&q.yb));
    let mut y = y0;
    for op in ops {
        let yb = op.yb.max(y0);
        let ye = op.ye.min(y1);
        let w = op.w.min(len - 0.2);
        let jamb = (len - w) / 2.0;
        seg(sink, 0.0, len, y, yb);
        seg(sink, -(w / 2.0 + jamb / 2.0), jamb, yb, ye);
        seg(sink, w / 2.0 + jamb / 2.0, jamb, yb, ye);
        if op.door {
            // The leaf, swung in against its jamb (the design's slice 2a:
            // doorways are open until doors open and shut as world state),
            // and the threshold.
            let hinge = mid - tan * (w / 2.0) - out * (t / 2.0);
            let leaf = hinge - out * (w / 2.0) + tan * 0.05;
            let into = (-out).y.atan2((-out).x);
            sink.plain_box(
                "timber",
                leaf.x,
                yb,
                leaf.y,
                Vec3::new(w - 0.04, ye - yb - 0.02, 0.07),
                into,
            );
            sink.plain_box(
                "timber",
                mid.x,
                yb,
                mid.y,
                Vec3::new(w + 0.02, 0.025, t + 0.16),
                ang,
            );
        }
        if op.shutters {
            for sgn in [-1.0f32, 1.0] {
                let hinge = mid + tan * (sgn * w / 2.0) + out * (t / 2.0 + 0.03);
                let a2 = ang + sgn * 1.9;
                let dir = Vec2::new(a2.cos(), a2.sin());
                let c = hinge + dir * (w / 4.0 * sgn);
                sink.plain_box(
                    "timber",
                    c.x,
                    yb,
                    c.y,
                    Vec3::new(w / 2.0, ye - yb, 0.05),
                    a2,
                );
            }
        }
        if op.sill {
            let c = mid + out * 0.02;
            sink.plain_box(
                "timber",
                c.x,
                yb - 0.04,
                c.y,
                Vec3::new(w + 0.1, 0.07, t + 0.12),
                ang,
            );
        }
        y = ye;
    }
    seg(sink, 0.0, len, y, y1);
}

/// The mockup's `gableRoof`: two slopes over the box `min`..`max` from the
/// wall top `wall_y`, the ridge along x, eaves `oh` out, gable ends of the
/// kit's `gable` face, a soffit under the wall top and a ridge beam.
#[allow(clippy::too_many_arguments)]
fn gable_roof(
    sink: &mut Sink,
    min: Vec2,
    max: Vec2,
    wall_y: f32,
    pitch: f32,
    material: &str,
    gable: &str,
    oh: f32,
    under: &str,
) {
    let (x0, x1, z0, z1) = (min.x, max.x, min.y, max.y);
    let zc = (z0 + z1) / 2.0;
    let hw = (z1 - z0) / 2.0;
    let ridge = wall_y + hw * pitch;
    let eave = wall_y - oh * pitch;
    let th = 0.18;
    let slope_uv = move |p: Vec3| {
        Vec2::new(
            p.x / 2.0,
            (p.z - zc).abs() * (1.0 + pitch * pitch).sqrt() / 2.0,
        )
    };
    for s in [-1.0f32, 1.0] {
        let ze = zc + s * (hw + oh);
        let v = |x: f32, y: f32, z: f32| Vec3::new(x, y, z);
        sink.face(
            material,
            &[
                v(x0 - oh, eave + th, ze),
                v(x1 + oh, eave + th, ze),
                v(x1 + oh, ridge + th, zc),
                v(x0 - oh, ridge + th, zc),
            ],
            Vec3::new(0.0, 1.0, s * 0.01),
            Some(&slope_uv),
        );
        sink.face(
            under,
            &[
                v(x0 - oh, eave, ze),
                v(x1 + oh, eave, ze),
                v(x1 + oh, ridge, zc),
                v(x0 - oh, ridge, zc),
            ],
            -Vec3::Y,
            None,
        );
        sink.face(
            material,
            &[
                v(x0 - oh, eave, ze),
                v(x1 + oh, eave, ze),
                v(x1 + oh, eave + th, ze),
                v(x0 - oh, eave + th, ze),
            ],
            Vec3::new(0.0, 0.0, s),
            None,
        );
        for xe in [x0 - oh, x1 + oh] {
            let side = if xe < x0 { -1.0 } else { 1.0 };
            sink.face(
                material,
                &[
                    v(xe, eave, ze),
                    v(xe, eave + th, ze),
                    v(xe, ridge + th, zc),
                    v(xe, ridge, zc),
                ],
                Vec3::new(side, 0.0, 0.0),
                None,
            );
        }
    }
    let gable_uv = move |p: Vec3| {
        Vec2::new(
            (p.z - z0) / (z1 - z0).max(0.01) * 2.0,
            ((p.y - wall_y) / STOREY_M).clamp(0.0, 1.0),
        )
    };
    for xe in [x0, x1] {
        for sgn in [-1.0f32, 1.0] {
            let outer = sgn * (xe - (x0 + x1) / 2.0) > 0.0;
            let m = if outer { gable } else { "plaster" };
            let tri = [
                Vec3::new(xe, wall_y, z0),
                Vec3::new(xe, wall_y, z1),
                Vec3::new(xe, ridge, zc),
            ];
            if outer && gable == "halftimber" {
                sink.face(m, &tri, Vec3::new(sgn, 0.0, 0.0), Some(&gable_uv));
            } else {
                sink.face(m, &tri, Vec3::new(sgn, 0.0, 0.0), None);
            }
        }
    }
    sink.face(
        "plank",
        &[
            Vec3::new(x0, wall_y, z0),
            Vec3::new(x1, wall_y, z0),
            Vec3::new(x1, wall_y, z1),
            Vec3::new(x0, wall_y, z1),
        ],
        -Vec3::Y,
        None,
    );
    sink.plain_box(
        "timber",
        (x0 + x1) / 2.0,
        ridge + th - 0.05,
        zc,
        Vec3::new(x1 - x0 + 2.0 * oh + 0.1, 0.16, 0.22),
        0.0,
    );
}

/// The mockup's `pyramidRoof`, open to the hut under it: six faces from the
/// eave, pushed out by `oh`, to the apex `h` over the wall top.
fn cone_roof(
    sink: &mut Sink,
    centre: Vec2,
    corners: &[Vec2; 6],
    wall_y: f32,
    h: f32,
    material: &str,
    oh: f32,
) {
    let apex = Vec3::new(centre.x, wall_y + h, centre.y);
    let half_width =
        corners.iter().map(|p| (*p - centre).length()).sum::<f32>() / 6.0 * (3.0f32).sqrt() / 2.0;
    let slope = h / half_width.max(0.1);
    let pts: Vec<Vec3> = corners
        .iter()
        .map(|p| {
            let away = (*p - centre).normalize_or_zero();
            let q = *p + away * (oh / (std::f32::consts::FRAC_PI_6).cos());
            Vec3::new(q.x, wall_y - oh * slope, q.y)
        })
        .collect();
    let uv = |p: Vec3| Vec2::new((p.x + p.z) / 2.0, p.y / 2.0);
    for k in 0..6 {
        let (a, b) = (pts[k], pts[(k + 1) % 6]);
        let outward = (a + b) * 0.5 - Vec3::new(centre.x, wall_y, centre.y);
        sink.face(material, &[a, b, apex], outward, Some(&uv));
        sink.face(material, &[a, b, apex], -Vec3::Y, Some(&uv));
    }
}
