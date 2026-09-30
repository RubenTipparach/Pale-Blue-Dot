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

/// A floor the walker stands on (`tenebris-towns` section 4): a region in
/// plan whose top is a function of position, and its underside, in its
/// building's frame. A stair's top is its pitch line, not the tread under
/// the point, so the eye climbs at the stair's slope.
#[derive(Clone, Debug, PartialEq)]
pub enum Surface {
    /// A flat floor over a convex outline, counter-clockwise.
    Floor {
        outline: Vec<Vec2>,
        top: f32,
        bottom: f32,
    },
    /// A straight flight: the foot of its first riser at `foot`, climbing
    /// along `dir` for `len`, `half_width` either side, from `base` by
    /// `risers` of `rise`; the last tread is the landing.
    Flight {
        foot: Vec2,
        dir: Vec2,
        len: f32,
        half_width: f32,
        base: f32,
        rise: f32,
        risers: u32,
    },
    /// A newel stair in one cell: `start` the angle it climbs from, turning
    /// `sense` (+1 or -1), a turn every `turn_m` from `base` to `top`, with
    /// `landing` radians of floor past the top and `margin` before the
    /// foot; nothing within `newel_r` of the centre, nothing outside the
    /// cell's `outline`.
    Newel {
        centre: Vec2,
        start: f32,
        sense: f32,
        base: f32,
        top: f32,
        turn_m: f32,
        newel_r: f32,
        landing: f32,
        margin: f32,
        outline: Vec<Vec2>,
    },
}

/// A tread's depth under its top: what a stair's underside is.
pub const TREAD_M: f32 = 0.26;

impl Surface {
    /// Every `(underside, top)` the surface has over `p` in plan.
    pub fn intervals(&self, p: Vec2, out: &mut Vec<(f32, f32)>) {
        match self {
            Surface::Floor {
                outline,
                top,
                bottom,
            } => {
                if inside(outline, p) {
                    out.push((*bottom, *top));
                }
            }
            Surface::Flight {
                foot,
                dir,
                len,
                half_width,
                base,
                rise,
                risers,
            } => {
                let d = p - *foot;
                let u = d.dot(*dir);
                if u < 0.0 || u > *len || d.perp_dot(*dir).abs() > *half_width {
                    return;
                }
                let n = *risers as f32;
                let run = len / n;
                let top = base + (u / ((n - 1.0) * run)).clamp(0.0, 1.0) * n * rise;
                // Boxed below, down to the floor it stands on.
                out.push((base - 0.01, top));
            }
            Surface::Newel {
                centre,
                start,
                sense,
                base,
                top,
                turn_m,
                newel_r,
                landing,
                margin,
                outline,
            } => {
                let d = p - *centre;
                if d.length() < *newel_r || !inside(outline, p) {
                    return;
                }
                let tau = std::f32::consts::TAU;
                let a = (sense * (d.y.atan2(d.x) - start)).rem_euclid(tau);
                let end = (top - base) / turn_m * tau;
                let mut k = -1.0f32;
                while k * tau + a <= end + landing {
                    let phi = a + k * tau;
                    k += 1.0;
                    if phi < -margin {
                        continue;
                    }
                    let t = base + phi.clamp(0.0, end) / tau * turn_m;
                    out.push((t - TREAD_M, t));
                }
            }
        }
    }
}

/// Whether a convex, counter-clockwise outline holds `p`.
fn inside(outline: &[Vec2], p: Vec2) -> bool {
    let n = outline.len();
    n >= 3
        && (0..n).all(|i| {
            let (a, b) = (outline[i], outline[(i + 1) % n]);
            (b - a).perp_dot(p - a) >= 0.0
        })
}

/// A door's leaf (`tenebris-towns` section 8): hinged at the inner face of
/// its jamb, `width` along the doorway when shut, swung inward against the
/// wall when open. Shut, it is a solid; open, it is none. `index` is the
/// door's number in its building's doors.
#[derive(Clone, Debug, PartialEq)]
pub struct DoorLeaf {
    pub index: usize,
    pub hinge: Vec2,
    /// Along the doorway from the hinge.
    pub along: Vec2,
    /// Out of the building, square to the wall.
    pub out: Vec2,
    pub width: f32,
    pub y0: f32,
    pub y1: f32,
    /// The doorway's middle in plan, where E reaches for it.
    pub middle: Vec2,
    pub open: bool,
}

/// A door leaf's thickness.
pub const LEAF_M: f32 = 0.07;

impl DoorLeaf {
    /// The leaf's direction from the hinge and its thickness's, swung
    /// `angle` from shut (0) toward open (a quarter turn inward).
    fn axes(&self, angle: f32) -> (Vec2, Vec2) {
        let (c, s) = (angle.cos(), angle.sin());
        (self.along * c - self.out * s, self.out * c + self.along * s)
    }

    /// The shut leaf as a solid.
    pub fn solid(&self) -> Solid {
        let (d, t) = self.axes(0.0);
        let h = self.hinge;
        Solid {
            outline: ccw(vec![
                h,
                h + d * self.width,
                h + d * self.width + t * LEAF_M,
                h + t * LEAF_M,
            ]),
            y0: self.y0,
            y1: self.y1,
        }
    }

    /// The leaf drawn swung `angle` from shut, in planet-local metres.
    pub fn mesh(&self, frame: Frame, angle: f32, repeat_m: &dyn Fn(&str) -> f32) -> Meshes {
        let (d, t) = self.axes(angle);
        let c = self.hinge + d * (self.width / 2.0) + t * (LEAF_M / 2.0);
        let mut meshes = Meshes::new();
        let mut sink = Sink::new(&mut meshes, repeat_m, frame);
        sink.plain_box(
            "timber",
            c.x,
            self.y0,
            c.y,
            Vec3::new(self.width - 0.04, self.y1 - self.y0 - 0.02, LEAF_M),
            d.y.atan2(d.x),
        );
        meshes
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
    /// Its upper floors and stairs (slice 2b).
    pub surfaces: Vec<Surface>,
    /// Its door leaves, open or shut.
    pub doors: Vec<DoorLeaf>,
    /// The faces seen from inside it, by texture: its rooms' walls, floors,
    /// ceilings and stairs (`sun-shadows` decision 7). They are drawn apart
    /// from the town's outside, under the room's own share of the sky.
    pub rooms: Meshes,
    /// What burns in its rooms (`cities-in-the-world` decision 7a).
    pub lights: Vec<RoomLight>,
    /// Its top storey's ceiling over its floor, metres: under it, and under
    /// the roof's plan, the rain does not fall.
    pub top_m: f32,
}

impl BuildingSolids {
    /// Whether the roof keeps the rain off a planet-local point: over the
    /// roof's plan, eaves and all, between the floor and the top storey's
    /// ceiling. A hut's cone and a gable's attic are over that ceiling, so
    /// a point in the room under them is sheltered too.
    pub fn shelters(&self, point: Vec3) -> bool {
        let p = self.frame.local(point);
        if p.y < -0.5 || p.y > self.top_m || self.roof_plan.len() < 3 {
            return false;
        }
        let q = Vec2::new(p.x, p.z);
        let n = self.roof_plan.len();
        let turn = (self.roof_plan[1] - self.roof_plan[0])
            .perp_dot(self.roof_plan[2] - self.roof_plan[0])
            .signum();
        (0..n).all(|k| {
            let (a, b) = (self.roof_plan[k], self.roof_plan[(k + 1) % n]);
            turn * (b - a).perp_dot(q - a) >= 0.0
        })
    }

    /// Whether a body centred at `centre` (planet-local), `half_height`
    /// tall each way and `radius` round, is in any solid.
    pub fn holds(&self, centre: Vec3, half_height: f32, radius: f32) -> bool {
        let p = self.frame.local(centre);
        if Vec2::new(p.x, p.z).length() > self.reach_m + radius {
            return false;
        }
        let held = |s: &Solid| s.holds(p.x, p.z, p.y - half_height, p.y + half_height, radius);
        self.solids.iter().any(held) || self.doors.iter().any(|d| !d.open && held(&d.solid()))
    }

    /// At a planet-local point, a foot: the highest top of a surface within
    /// `reach` above it, the floor, and the lowest underside of a surface
    /// above that, the ceiling; both as planet-local radii.
    pub fn stand(&self, point: Vec3, reach: f32) -> (Option<f32>, Option<f32>) {
        let p = self.frame.local(point);
        let plan = Vec2::new(p.x, p.z);
        if self.surfaces.is_empty() || plan.length() > self.reach_m {
            return (None, None);
        }
        let mut spans = Vec::new();
        for s in &self.surfaces {
            s.intervals(plan, &mut spans);
        }
        let floor = spans
            .iter()
            .map(|s| s.1)
            .filter(|&top| top <= p.y + reach)
            .max_by(f32::total_cmp);
        let ceiling = spans
            .iter()
            .filter(|s| s.1 > p.y + reach && s.0 > p.y)
            .map(|s| s.0)
            .min_by(f32::total_cmp);
        let radius = |y: f32| self.frame.world(Vec3::new(p.x, y, p.z)).length();
        (floor.map(radius), ceiling.map(radius))
    }

    /// The way out of whatever solid holds a body centred at `centre`: from
    /// the nearest point of its outline toward the body, planet-local and
    /// along the ground. A refused move slides along the face this is the
    /// normal of (`tenebris-towns` section 4).
    pub fn push_normal(&self, centre: Vec3, half_height: f32, radius: f32) -> Option<Vec3> {
        let p = self.frame.local(centre);
        let plan = Vec2::new(p.x, p.z);
        let shut: Vec<Solid> = self
            .doors
            .iter()
            .filter(|d| !d.open)
            .map(DoorLeaf::solid)
            .collect();
        let mut best: Option<(f32, Vec2)> = None;
        for s in self.solids.iter().chain(&shut) {
            if !s.holds(p.x, p.z, p.y - half_height, p.y + half_height, radius) {
                continue;
            }
            let n = s.outline.len();
            let within = inside(&s.outline, plan);
            for i in 0..n {
                let (a, b) = (s.outline[i], s.outline[(i + 1) % n]);
                let e = b - a;
                let t = ((plan - a).dot(e) / e.length_squared().max(1e-9)).clamp(0.0, 1.0);
                let q = a + e * t;
                let d = q.distance(plan);
                // Inside the outline the way out is the edge's own outward
                // normal; outside it, from the nearest point to the body.
                let away = if within {
                    Vec2::new(e.y, -e.x).normalize_or_zero()
                } else {
                    (plan - q).normalize_or_zero()
                };
                if best.is_none_or(|(bd, _)| d < bd) {
                    best = Some((d, away));
                }
            }
        }
        best.map(|(_, n)| self.frame.world_dir(Vec3::new(n.x, 0.0, n.y)))
            .filter(|n| *n != Vec3::ZERO)
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
    pub surfaces: Vec<Surface>,
    pub doors: Vec<DoorLeaf>,
    /// The faces seen from inside the building, apart from `meshes`.
    pub rooms: Meshes,
    /// What is inside: the plan's cells and the top of the top storey. Unset,
    /// every face is the town's outside.
    pub indoors: Option<Indoors>,
    /// What burns in its rooms (decision 7a).
    pub lights: Vec<RoomLight>,
}

/// What burns in a room (`cities-in-the-world` decision 7a), as the towns
/// mockup's `hearth`, `sconce` and window candles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LightKind {
    Hearth,
    Sconce,
    Candle,
}

impl LightKind {
    /// Its colour in linear light: the mockup's `#ff9a4a`, `#ffb870` and
    /// `#ffb060`.
    pub fn colour(self) -> [f32; 3] {
        srgb_linear(match self {
            LightKind::Hearth => 0xff9a4a,
            LightKind::Sconce => 0xffb870,
            LightKind::Candle => 0xffb060,
        })
    }

    /// How far it reaches, metres: the mockup's `LIGHT_REACH`.
    pub fn reach_m(self) -> f32 {
        match self {
            LightKind::Hearth => 7.0,
            LightKind::Sconce => 5.5,
            LightKind::Candle => 5.0,
        }
    }

    /// Whether it burns all day, as a fire does, or only by night, as a
    /// candle does.
    pub fn all_day(self) -> bool {
        !matches!(self, LightKind::Candle)
    }
}

/// An sRGB colour as linear light, as the mockup's `lin`.
pub fn srgb_linear(hex: u32) -> [f32; 3] {
    let channel = |shift: u32| {
        let c = ((hex >> shift) & 0xff) as f32 / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    [channel(16), channel(8), channel(0)]
}

/// A light of a building's own: where it burns (planet-local), how strongly
/// (the mockup's `power`), and the band of height round it that it lights,
/// metres below and above it: its storey, or a stairwell's run.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RoomLight {
    pub kind: LightKind,
    pub at: Vec3,
    pub power: f32,
    pub below_m: f32,
    pub above_m: f32,
}

/// The share of a town's windows with a candle behind them: the mockup's
/// `litWindows` for its town.
pub const CANDLE_SHARE: f32 = 0.55;
/// A hearth's width between its cheeks, metres (the mockup's `w`).
const HEARTH_W_M: f32 = 1.2;

/// Which of a building's windows has a candle behind it, 0..1 against
/// [`CANDLE_SHARE`]: a hash of the window's place in the building's
/// definition, so a town's candles are the same on every load.
pub fn window_roll(c: i32, r: i32, d: usize, storey: u32) -> f32 {
    let mut x = (c as u32).wrapping_mul(0x9E37_79B1)
        ^ (r as u32).wrapping_mul(0x85EB_CA77)
        ^ (d as u32).wrapping_mul(0xC2B2_AE3D)
        ^ storey.wrapping_mul(0x27D4_EB2F);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    x = x.wrapping_mul(0x297A_2D39);
    x ^= x >> 15;
    (x >> 8) as f32 / (1u32 << 24) as f32
}

/// A building's inside, in its frame: the air over its plan's cells from its
/// floor to its ceiling, and under a roof it is open to.
#[derive(Clone, Debug)]
pub struct Indoors {
    cells: Vec<[Vec2; 6]>,
    /// The plan's outer edges, where its walls stand.
    walls: Vec<(Vec2, Vec2)>,
    top: f32,
    /// Whether its top storey is open to the roof, as a hut is to its cone:
    /// otherwise a ceiling closes it and the roof's underside is the eaves'.
    open_roof: bool,
}

/// How far in front of a face its air is looked for, metres: past a wall's
/// face and short of the next.
const FRONT_M: f32 = 0.05;
/// How far inside the wall line a face's air must be to be a room's: a
/// window's reveal and a door's jamb stand across the wall, in the opening
/// the sky comes through, and are the outside's.
const WALL_LINE_M: f32 = 0.02;

impl Indoors {
    pub fn new(cells: Vec<[Vec2; 6]>, top: f32, open_roof: bool) -> Self {
        let near = |a: Vec2, b: Vec2| a.distance(b) < 0.05;
        let mut walls = Vec::new();
        for (i, hex) in cells.iter().enumerate() {
            for k in 0..6 {
                let (a, b) = (hex[k], hex[(k + 1) % 6]);
                let shared = cells.iter().enumerate().any(|(j, other)| {
                    j != i
                        && (0..6).any(|m| {
                            let (c, d) = (other[m], other[(m + 1) % 6]);
                            (near(a, c) && near(b, d)) || (near(a, d) && near(b, c))
                        })
                });
                if !shared {
                    walls.push((a, b));
                }
            }
        }
        Self {
            cells,
            walls,
            top,
            open_roof,
        }
    }

    /// Whether a face at `centre` (in the frame), facing `n`, is seen from
    /// inside: the air in front of it is over a cell of the plan, inside the wall
    /// line, and the face stands from the ground floor to the ceiling of the
    /// top storey, or it is the underside of a roof the room is open to, as a
    /// hut's is. A wall's outer face has its air outside the plan, and a
    /// roof's top and the soffit under a ceiled roof stand over the ceiling.
    pub fn holds(&self, centre: Vec3, n: Vec3) -> bool {
        let front = centre + n * FRONT_M;
        if front.y < -0.3 {
            return false;
        }
        if centre.y > self.top + 0.005 && (n.y > -0.1 || !self.open_roof) {
            return false;
        }
        let p = Vec2::new(front.x, front.z);
        let over = self.cells.iter().any(|hex| {
            let turn = (hex[1] - hex[0]).perp_dot(hex[2] - hex[0]).signum();
            (0..6).all(|k| turn * (hex[(k + 1) % 6] - hex[k]).perp_dot(p - hex[k]) >= 0.0)
        });
        over && self.walls.iter().all(|&(a, b)| {
            let t = ((p - a).dot(b - a) / (b - a).length_squared()).clamp(0.0, 1.0);
            p.distance(a + (b - a) * t) >= WALL_LINE_M
        })
    }
}

impl<'a> Sink<'a> {
    pub fn new(meshes: &'a mut Meshes, repeat_m: &'a dyn Fn(&str) -> f32, frame: Frame) -> Self {
        Self {
            meshes,
            repeat_m,
            frame,
            solids: Vec::new(),
            roof_plan: Vec::new(),
            surfaces: Vec::new(),
            doors: Vec::new(),
            rooms: Meshes::new(),
            indoors: None,
            lights: Vec::new(),
        }
    }

    /// A light at `p` in the frame.
    fn light(&mut self, kind: LightKind, p: Vec3, power: f32, below_m: f32, above_m: f32) {
        self.lights.push(RoomLight {
            kind,
            at: self.frame.world(p),
            power,
            below_m,
            above_m,
        });
    }

    /// A flame standing at `p` in the frame, `h` tall: three blades crossed,
    /// drawn unlit in `material` (`flame`).
    fn flame(&mut self, material: &str, p: Vec3, h: f32) {
        for k in 0..3 {
            let a = k as f32 * std::f32::consts::FRAC_PI_3;
            let along = Vec3::new(a.cos(), 0.0, a.sin()) * (h * 0.3);
            let facing = Vec3::new(-a.sin(), 0.0, a.cos());
            self.face(
                material,
                &[p - along, p + along, p + Vec3::Y * h],
                facing,
                None,
            );
        }
    }

    /// A sconce on a wall at `wall` (plan), `y` up, facing into the room
    /// along `n`: a timber bracket, a flame, and its light, which reaches up
    /// and down a stairwell as the mockup's does. Timber, not the mockup's
    /// iron: a bracket this close under its own flame takes none of its
    /// light, and iron read as a black box on the wall.
    fn sconce(&mut self, wall: Vec2, y: f32, n: Vec2) {
        let ang = n.y.atan2(n.x);
        let b = wall + n * 0.08;
        self.plain_box(
            "timber",
            b.x,
            y - 0.14,
            b.y,
            Vec3::new(0.16, 0.05, 0.12),
            ang,
        );
        let f = wall + n * 0.14;
        self.flame("flame", Vec3::new(f.x, y - 0.09, f.y), 0.22);
        let l = wall + n * 0.2;
        self.light(LightKind::Sconce, Vec3::new(l.x, y, l.y), 0.75, 1.2, 2.8);
    }

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
        let centre = pts.iter().fold(Vec3::ZERO, |s, p| s + *p) / pts.len() as f32;
        let inside = self
            .indoors
            .as_ref()
            .is_some_and(|rooms| rooms.holds(centre, n));
        let meshes = if inside {
            &mut self.rooms
        } else {
            &mut *self.meshes
        };
        let buf = meshes.entry(material.to_string()).or_default();
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
/// A door has its number in its building's doors, and a leaf; a doorway
/// with no door (a stair's) has neither.
#[derive(Clone, Copy, Debug)]
struct Opening {
    yb: f32,
    ye: f32,
    w: f32,
    door: Option<usize>,
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
    let mut sink = Sink::new(meshes, repeat_m, frame);
    cut(&mut sink, &plan, def, kit)?;
    Ok(BuildingSolids {
        frame,
        reach_m,
        solids: sink.solids,
        roof_plan: sink.roof_plan,
        surfaces: sink.surfaces,
        doors: sink.doors,
        top_m: sink.indoors.as_ref().map_or(0.0, |i| i.top),
        rooms: sink.rooms,
        lights: sink.lights,
    })
}

fn edge_ends(plan: &Plan, i: usize, d: usize) -> (Vec2, Vec2) {
    (plan.corners[i][d % 6], plan.corners[i][(d + 1) % 6])
}

/// A corner post where walls meet: its place in plan, its bottom and top,
/// its wall's thickness and its material.
type Post = (Vec2, f32, f32, f32, String);

fn cut(sink: &mut Sink, plan: &Plan, def: &BuildingDef, kit: &Kit) -> Result<(), String> {
    let storeys = def.storeys.max(1);
    let storey_m = if kit.hut {
        HUT_STOREY_M
    } else {
        STOREY_M * def.tall.max(1) as f32
    };
    let top = storeys as f32 * storey_m;
    let open_roof = def.roof == "cone" && plan.cells.len() == 1;
    sink.indoors = Some(Indoors::new(plan.corners.clone(), top, open_roof));
    let (door_w, door_h) = kit.door_m;
    let inside = |c: i32, r: i32| plan.index(c, r).is_some();
    // The stair, from the building's stair cells (slice 2b).
    let stair = stair_of(plan, def)?;
    // Floors: the ground floor's boards over the terrace, and a slab and a
    // beam under every floor above. No floor is cut over a newel's cell,
    // and a straight flight's two cells are floored only either side of its
    // strip: the well.
    for (i, &(_, _)) in plan.cells.iter().enumerate() {
        let hex: Vec<Vec2> = plan.corners[i].to_vec();
        sink.prism(&kit.floor, &kit.floor, &hex, -0.05, LIFT_M, None);
        if !kit.hut {
            let in_stair = stair.as_ref().is_some_and(|s| s.holds(i));
            for s in 1..storeys {
                let fy = s as f32 * storey_m;
                let parts: Vec<Vec<Vec2>> = match &stair {
                    Some(Stair::Newel { cell, .. }) if *cell == i => Vec::new(),
                    Some(st @ Stair::Flight { .. }) if st.holds(i) && s == 1 => {
                        let (foot, dir, hw) = flight_strip(plan, st);
                        let side = Vec2::new(-dir.y, dir.x);
                        [1.0f32, -1.0]
                            .iter()
                            .map(|&k| clip_half(&hex, foot + side * (k * hw), side * k))
                            .filter(|p| p.len() >= 3)
                            .collect()
                    }
                    _ => vec![hex.clone()],
                };
                for part in parts {
                    sink.prism(
                        "plank",
                        "timber",
                        &part,
                        fy - SLAB_M,
                        fy + LIFT_M,
                        Some("plank"),
                    );
                    // The slab is a ceiling to the storey under it, and the
                    // floor of the one it carries.
                    sink.solids.push(Solid {
                        outline: ccw(part.clone()),
                        y0: fy - SLAB_M,
                        y1: fy + LIFT_M,
                    });
                    sink.surfaces.push(Surface::Floor {
                        outline: ccw(part),
                        top: fy + LIFT_M,
                        bottom: fy - SLAB_M,
                    });
                }
            }
            for s in 1..=storeys {
                if in_stair && s < storeys {
                    continue;
                }
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
                if let Some(index) = def.doors.iter().position(|x| *x == at) {
                    openings.push(Opening {
                        yb: y0,
                        ye: y0 + door_h,
                        w: door_w,
                        door: Some(index),
                        sill: false,
                        shutters: false,
                    });
                } else if def.windows.contains(&at) {
                    if def.tall > 1 {
                        openings.push(Opening {
                            yb: y0 + 1.6,
                            ye: y0 + 4.4,
                            w: 0.9,
                            door: None,
                            sill: true,
                            shutters: false,
                        });
                    } else if let Some((w, h)) = kit.window_m {
                        openings.push(Opening {
                            yb: y0 + 1.0,
                            ye: y0 + 1.0 + h,
                            w,
                            door: None,
                            sill: !kit.hut,
                            shutters: false,
                        });
                    } else if !kit.hut {
                        openings.push(Opening {
                            yb: y0 + 1.0,
                            ye: y0 + 1.0 + WINDOW_M.1,
                            w: WINDOW_M.0,
                            door: None,
                            sill: true,
                            shutters: true,
                        });
                    }
                }
                let (a, b) = edge_ends(plan, i, d);
                edge_wall(sink, plan.centres[i], a, b, y0, y1, wall, &openings);
                // A candle in the room behind about half the windows, 0.8 m
                // in and 0.2 m over the sill (decision 7a).
                let roll = window_roll(c, r, d, s);
                if let Some(op) = openings.first().filter(|o| o.door.is_none())
                    && roll < CANDLE_SHARE
                {
                    let mid = (a + b) * 0.5;
                    let e = (b - a).normalize();
                    let mut inward = Vec2::new(-e.y, e.x);
                    if inward.dot(plan.centres[i] - mid) < 0.0 {
                        inward = -inward;
                    }
                    let at = mid + inward * 0.8;
                    let y = op.yb + 0.2;
                    let k = 0.55 + 0.45 * (roll * 7.31).fract();
                    sink.light(
                        LightKind::Candle,
                        Vec3::new(at.x, y, at.y),
                        0.45 * k,
                        y - y0 + 0.05,
                        y1 - SLAB_M - y + 0.05,
                    );
                }
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
                sink,
                &plan.corners,
                min,
                max,
                top,
                def.pitch,
                &material,
                &kit.gable,
                overhang,
                under,
            );
        }
    }
    // The hearth under the chimney, against the first of its cell's outer
    // walls with no door or window, edge 0 first as the mockup's (decision
    // 7a). Not in a stair's cell.
    if let Some([hc, hr]) = def.chimney
        && let Some(i) = plan.index(hc, hr)
        && !stair.as_ref().is_some_and(|s| s.holds(i))
    {
        let opened = |d: usize| {
            let at = [hc, hr, d as i32, 0];
            def.doors.contains(&at) || def.windows.contains(&at)
        };
        let wall = (0..6).find(|&d| {
            let (c2, r2) = neighbour(hc, hr, d);
            !inside(c2, r2) && !opened(d)
        });
        if let Some(d) = wall {
            hearth(sink, plan, i, d, storey_m);
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
    if let Some(stair) = &stair {
        cut_stair(sink, plan, kit, stair, storeys, storey_m);
    }
    Ok(())
}

/// The mockup's `hearth`: against edge `d` of plan cell `i`, inside it, a
/// stone hearth with cheeks, a hood to the ceiling, logs and a fire, whose
/// light fills the ground storey.
fn hearth(sink: &mut Sink, plan: &Plan, i: usize, d: usize, storey_m: f32) {
    let c = plan.centres[i];
    let (a, b) = edge_ends(plan, i, d);
    let m = (a + b) * 0.5;
    let u = (m - c).normalize();
    let across = Vec2::new(-u.y, u.x);
    let p = c + u * ((m - c).length() - 0.55);
    let ang = u.y.atan2(u.x);
    let w = HEARTH_W_M;
    let stone = |sink: &mut Sink, q: Vec2, y0: f32, s: Vec3, solid: bool| {
        sink.plain_box("stone", q.x, y0, q.y, s, ang);
        if solid {
            sink.solid_box(q.x, y0, q.y, s, ang);
        }
    };
    stone(sink, p, 0.0, Vec3::new(0.7, 0.35, w + 0.2), true);
    for side in [-1.0f32, 1.0] {
        stone(
            sink,
            p + across * (side * w / 2.0),
            0.35,
            Vec3::new(0.7, 1.05, 0.2),
            true,
        );
    }
    let ceiling = storey_m - SLAB_M;
    stone(
        sink,
        p + u * 0.1,
        1.4,
        Vec3::new(0.5, ceiling - 1.4, w + 0.2),
        false,
    );
    for k in [-1.0f32, 0.0, 1.0] {
        let q = p - across * (k * 0.2);
        sink.plain_box(
            "bark",
            q.x,
            0.35,
            q.y,
            Vec3::new(0.5, 0.1, 0.1),
            ang + 0.4 * k,
        );
    }
    let fire = p - u * 0.05;
    sink.flame("flame", Vec3::new(fire.x, 0.45, fire.y), 0.5);
    sink.light(
        LightKind::Hearth,
        Vec3::new(fire.x, 0.9, fire.y),
        1.0,
        0.95,
        ceiling - 0.9 + 0.05,
    );
}

/// A newel's winders a turn and their rise (`tenebris-towns` section 3).
const NEWEL_WINDERS: u32 = 15;
/// The newel post's radius.
const NEWEL_R: f32 = 0.2;
/// The newel's own walls, on its edges inside the building.
const NEWEL_WALL_M: f32 = 0.2;
/// The floor past a newel's last winder before its rail, radians.
const NEWEL_LANDING: f32 = std::f32::consts::PI / 6.0;
/// A straight flight's risers.
const FLIGHT_RISERS: u32 = 16;
/// A stair's doorway, wide by high (the mockup's newel exits).
const STAIR_DOOR_M: (f32, f32) = (0.95, 2.2);

/// A building's stair, from its stair cells as the mockup's `townHouse`
/// makes it: one cell is a newel stair, two in a row a straight flight.
enum Stair {
    /// In plan cell `cell`, climbing from its edge `entry`.
    Newel { cell: usize, entry: usize },
    /// From plan cell `from` to its neighbour `to`, across edge `d`.
    Flight { from: usize, to: usize, d: usize },
}

impl Stair {
    fn holds(&self, i: usize) -> bool {
        match self {
            Stair::Newel { cell, .. } => *cell == i,
            Stair::Flight { from, to, .. } => *from == i || *to == i,
        }
    }
}

fn stair_of(plan: &Plan, def: &BuildingDef) -> Result<Option<Stair>, String> {
    let find = |[c, r]: [i32; 2]| {
        plan.index(c, r)
            .ok_or_else(|| format!("{}: stair cell ({c}, {r}) is not the building's", def.name))
    };
    match def.stair_cells.as_slice() {
        [] => Ok(None),
        [one] => {
            let cell = find(*one)?;
            let (c, r) = plan.cells[cell];
            // The first edge onto another of the building's cells that is not
            // the front door's.
            let door_cells: Vec<(i32, i32)> = def
                .doors
                .iter()
                .filter(|d| d[3] == 0)
                .map(|d| (d[0], d[1]))
                .collect();
            let entry = (0..6)
                .find(|&d| {
                    let n = neighbour(c, r, d);
                    plan.index(n.0, n.1).is_some() && !door_cells.contains(&n)
                })
                .ok_or_else(|| format!("{}: the newel at ({c}, {r}) has no way in", def.name))?;
            Ok(Some(Stair::Newel { cell, entry }))
        }
        [a, b] => {
            let (from, to) = (find(*a)?, find(*b)?);
            let (c, r) = plan.cells[from];
            let d = (0..6)
                .find(|&d| neighbour(c, r, d) == plan.cells[to])
                .ok_or_else(|| format!("{}: the flight's cells are not neighbours", def.name))?;
            Ok(Some(Stair::Flight { from, to, d }))
        }
        more => Err(format!("{}: {} stair cells", def.name, more.len())),
    }
}

/// A flight's foot (the middle of its first cell's far flat), its
/// direction up, and its half width.
fn flight_strip(plan: &Plan, stair: &Stair) -> (Vec2, Vec2, f32) {
    let Stair::Flight { from, to, d } = *stair else {
        unreachable!("a flight");
    };
    let (a, b) = edge_ends(plan, from, (d + 3) % 6);
    let (a2, b2) = edge_ends(plan, to, d);
    let (foot, end) = ((a + b) * 0.5, (a2 + b2) * 0.5);
    let hw = ((b - a).length() + (b2 - a2).length()) / 4.0;
    (foot, (end - foot).normalize(), hw)
}

/// The part of a convex polygon on the side of the line through `at` that
/// `keep` points to.
fn clip_half(poly: &[Vec2], at: Vec2, keep: Vec2) -> Vec<Vec2> {
    let side = |p: Vec2| (p - at).dot(keep);
    let mut out = Vec::new();
    for i in 0..poly.len() {
        let (p, q) = (poly[i], poly[(i + 1) % poly.len()]);
        let (sp, sq) = (side(p), side(q));
        if sp >= 0.0 {
            out.push(p);
        }
        if (sp >= 0.0) != (sq >= 0.0) {
            out.push(p + (q - p) * (sp / (sp - sq)));
        }
    }
    out
}

/// Where two lines meet, each a point and a direction.
fn meet((p0, e0): (Vec2, Vec2), (p1, e1): (Vec2, Vec2)) -> Vec2 {
    p0 + e0 * ((p1 - p0).perp_dot(e1) / e0.perp_dot(e1))
}

/// The tread material a kit's stairs are made of, as the mockup chooses.
fn tread_of(kit: &Kit) -> &'static str {
    if matches!(
        kit.name.as_str(),
        "stone" | "fieldstone" | "ashlar" | "clay"
    ) {
        "stone"
    } else {
        "timber"
    }
}

/// A rail at `yt`: its bar, a solid a metre high, and posts along it.
fn rail(sink: &mut Sink, c: Vec2, yt: f32, len: f32, ang: f32) {
    sink.plain_box(
        "timber",
        c.x,
        yt + 0.95,
        c.y,
        Vec3::new(len, 0.08, 0.08),
        ang,
    );
    sink.solid_box(c.x, yt, c.y, Vec3::new(len, 1.0, 0.1), ang);
    let posts = ((len / 0.7).round() as i32).max(2);
    let dir = Vec2::new(ang.cos(), ang.sin());
    for k in 0..=posts {
        let p = c + dir * ((k as f32 / posts as f32 - 0.5) * (len - 0.06));
        sink.plain_box("timber", p.x, yt, p.y, Vec3::new(0.07, 0.95, 0.07), ang);
    }
}

fn cut_stair(sink: &mut Sink, plan: &Plan, kit: &Kit, stair: &Stair, storeys: u32, storey_m: f32) {
    let tread = tread_of(kit);
    let inner = kit.walls[0].inside.clone();
    match *stair {
        Stair::Newel { cell, entry } => {
            let c = plan.centres[cell];
            let corners = plan.corners[cell];
            let (cc, cr) = plan.cells[cell];
            let own = |d: usize| {
                let n = neighbour(cc, cr, d);
                plan.index(n.0, n.1).is_some()
            };
            // The cell's inside: each edge's line moved in by half its wall.
            let lines: Vec<(Vec2, Vec2)> = (0..6)
                .map(|d| {
                    let (a, b) = (corners[d], corners[(d + 1) % 6]);
                    let e = (b - a).normalize();
                    let mut n = Vec2::new(-e.y, e.x);
                    if n.dot(c - a) < 0.0 {
                        n = -n;
                    }
                    let t = if own(d) {
                        NEWEL_WALL_M
                    } else {
                        kit.walls[0].thickness_m
                    };
                    (a + n * (t / 2.0), e)
                })
                .collect();
            let verts: Vec<Vec2> = (0..6).map(|k| meet(lines[(k + 5) % 6], lines[k])).collect();
            let reach = |a: f32| -> f32 {
                let dir = Vec2::new(a.cos(), a.sin());
                lines
                    .iter()
                    .filter_map(|&(p0, e)| {
                        let den = dir.perp_dot(e);
                        (den.abs() > 1e-6)
                            .then(|| (p0 - c).perp_dot(e) / den)
                            .filter(|r| *r > 0.0)
                    })
                    .fold(f32::MAX, f32::min)
            };
            // It climbs from its entry edge toward the edge numbered next, as
            // the mockup's does, measured from the real edge midpoints.
            let angle = |d: usize| {
                let (a, b) = edge_ends(plan, cell, d);
                let m = (a + b) * 0.5 - c;
                m.y.atan2(m.x)
            };
            let tau = std::f32::consts::TAU;
            let start = angle(entry);
            let turn = (angle((entry + 1) % 6) - start + std::f32::consts::PI).rem_euclid(tau)
                - std::f32::consts::PI;
            let sense = turn.signum();
            let at = |phi: f32| start + sense * phi;
            let pt = |a: f32, r: f32| c + Vec2::new(a.cos(), a.sin()) * r;
            let wedge = |p0: f32, p1: f32| -> Vec<Vec2> {
                let mut pts = vec![pt(at(p0), NEWEL_R * 0.9), pt(at(p0), reach(at(p0)))];
                let mut mids: Vec<(f32, Vec2)> = verts
                    .iter()
                    .filter_map(|v| {
                        let d = *v - c;
                        let phi = (sense * (d.y.atan2(d.x) - start)).rem_euclid(tau);
                        let phi = phi + ((p0 - phi) / tau).ceil() * tau;
                        (phi > p0 && phi < p1).then_some((phi, *v))
                    })
                    .collect();
                mids.sort_by(|a, b| a.0.total_cmp(&b.0));
                pts.extend(mids.into_iter().map(|m| m.1));
                pts.push(pt(at(p1), reach(at(p1))));
                pts.push(pt(at(p1), NEWEL_R * 0.9));
                pts
            };
            let top = (storeys - 1) as f32 * storey_m;
            let rise = storey_m / NEWEL_WINDERS as f32;
            let dphi = tau / NEWEL_WINDERS as f32;
            let winders = (top / rise).round() as u32;
            for i in 1..=winders {
                let y = i as f32 * rise;
                let p0 = (i as f32 - 0.5) * dphi;
                sink.prism(
                    tread,
                    tread,
                    &wedge(p0, p0 + dphi),
                    y - TREAD_M,
                    y,
                    Some(tread),
                );
            }
            let wall_top = storeys as f32 * storey_m;
            let post: Vec<Vec2> = (0..8).map(|k| pt(k as f32 * tau / 8.0, NEWEL_R)).collect();
            sink.prism(tread, tread, &post, 0.0, wall_top, None);
            sink.solids.push(Solid {
                outline: ccw(post),
                y0: 0.0,
                y1: wall_top,
            });
            // The top: 30 degrees of floor past the last winder, then a rail,
            // so a walker who keeps turning meets the rail, not the well.
            let end = top / storey_m * tau;
            let step = (NEWEL_LANDING - dphi * 0.5) / 4.0;
            for k in 0..4 {
                let p0 = end + dphi * 0.5 + k as f32 * step;
                sink.prism(
                    tread,
                    tread,
                    &wedge(p0, p0 + step),
                    top - TREAD_M,
                    top,
                    Some(tread),
                );
            }
            let ar = at(end + NEWEL_LANDING);
            let len = reach(ar) - NEWEL_R;
            rail(sink, pt(ar, NEWEL_R + len / 2.0), top, len, ar);
            sink.surfaces.push(Surface::Newel {
                centre: c,
                start,
                sense,
                base: 0.0,
                top,
                turn_m: storey_m,
                newel_r: NEWEL_R,
                landing: NEWEL_LANDING,
                margin: dphi * 0.5,
                outline: ccw(corners.to_vec()),
            });
            // Its own walls on its edges inside the building, a doorway at
            // the foot and one onto every floor above, all on the entry edge:
            // a storey is three layers, and a layer turns two edges.
            let edge = if tread == "timber" {
                "timber".to_string()
            } else {
                inner.clone()
            };
            let faces = super::WallFaces {
                outside: inner.clone(),
                inside: inner.clone(),
                edge,
                thickness_m: NEWEL_WALL_M,
                per_face: false,
            };
            for d in (0..6).filter(|&d| own(d)) {
                let openings: Vec<Opening> = if d == entry {
                    (0..storeys)
                        .map(|s| Opening {
                            yb: s as f32 * storey_m,
                            ye: s as f32 * storey_m + STAIR_DOOR_M.1,
                            w: STAIR_DOOR_M.0,
                            door: None,
                            sill: false,
                            shutters: false,
                        })
                        .collect()
                } else {
                    Vec::new()
                };
                let (a, b) = edge_ends(plan, cell, d);
                edge_wall(sink, c, a, b, 0.0, wall_top, &faces, &openings);
            }
            // A sconce a storey, 2.2 m over the tread, on the first of the
            // stair's own walls round from its entry without a doorway, the
            // mockup's order (decision 7a).
            if let Some(k) = [2usize, 4, 1, 5]
                .into_iter()
                .find(|&k| own((entry + k) % 6))
            {
                let (a, b) = edge_ends(plan, cell, (entry + k) % 6);
                let m = (a + b) * 0.5;
                let u = (m - c).normalize();
                let wall = c + u * ((m - c).length() - NEWEL_WALL_M / 2.0 - 0.02);
                for s in 0..storeys.saturating_sub(1) {
                    let y = (s as f32 + k as f32 / 6.0) * storey_m + 2.2;
                    sink.sconce(wall, y, -u);
                }
            }
        }
        Stair::Flight { .. } => {
            let (foot, dir, hw) = flight_strip(plan, stair);
            let Stair::Flight { to, d, .. } = *stair else {
                unreachable!("a flight");
            };
            let (a2, b2) = edge_ends(plan, to, d);
            let len = ((a2 + b2) * 0.5 - foot).length();
            let n = FLIGHT_RISERS;
            let (rise, run) = (storey_m / n as f32, len / n as f32);
            let ang = dir.y.atan2(dir.x);
            let side = Vec2::new(-dir.y, dir.x);
            for k in 0..n {
                let y = (k + 1) as f32 * rise;
                let p = foot + dir * (k as f32 * run + run / 2.0);
                let m = if k == n - 1 { "plank" } else { tread };
                sink.plain_box(m, p.x, 0.0, p.y, Vec3::new(run, y, 2.0 * hw - 0.02), ang);
            }
            sink.surfaces.push(Surface::Flight {
                foot,
                dir,
                len,
                half_width: hw,
                base: 0.0,
                rise,
                risers: n,
            });
            // Boxed below, both sides, and the landing's end.
            let mid = foot + dir * (len / 2.0);
            for s in [-1.0f32, 1.0] {
                let p = mid + side * (s * (hw + 0.07));
                let size = Vec3::new(len, storey_m - SLAB_M, 0.14);
                sink.plain_box(&inner, p.x, 0.0, p.y, size, ang);
                sink.solid_box(p.x, 0.0, p.y, size, ang);
            }
            // A sconce on the boxed side, a quarter of the way up, clear of
            // the head (decision 7a).
            sink.sconce(foot + dir * (len * 0.25) + side * (hw - 0.01), 2.45, -side);
            let p = foot + dir * (len - run / 2.0);
            sink.solid_box(
                p.x,
                0.0,
                p.y,
                Vec3::new(run, storey_m - SLAB_M, 2.0 * hw),
                ang,
            );
            // Up top, the well railed along both sides and across its foot,
            // open at the landing.
            for s in [-1.0f32, 1.0] {
                let p = foot + dir * ((len - run) / 2.0) + side * (s * hw);
                rail(sink, p, storey_m, len - run, ang);
            }
            rail(
                sink,
                foot,
                storey_m,
                2.0 * hw,
                ang + std::f32::consts::FRAC_PI_2,
            );
        }
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
        if let Some(index) = op.door {
            // The leaf is the door's own, drawn and walked into by its state
            // (slice 2b), hinged at the inner face of the jamb; and the
            // threshold.
            sink.doors.push(DoorLeaf {
                index,
                hinge: mid - tan * (w / 2.0) - out * (t / 2.0),
                along: tan,
                out,
                width: w,
                y0: yb,
                y1: ye - 0.02,
                middle: mid,
                open: false,
            });
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
    cells: &[[Vec2; 6]],
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
    // The top storey's ceiling over each cell, and the soffit over the box
    // a lift above it: the room sees the one and the street the other where
    // the box stands past the walls (`sun-shadows` decision 7).
    for hex in cells {
        let ceiling: Vec<Vec3> = hex.iter().map(|p| Vec3::new(p.x, wall_y, p.y)).collect();
        sink.face("plank", &ceiling, -Vec3::Y, None);
    }
    let soffit = wall_y + LIFT_M;
    sink.face(
        "plank",
        &[
            Vec3::new(x0, soffit, z0),
            Vec3::new(x1, soffit, z0),
            Vec3::new(x1, soffit, z1),
            Vec3::new(x0, soffit, z1),
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
