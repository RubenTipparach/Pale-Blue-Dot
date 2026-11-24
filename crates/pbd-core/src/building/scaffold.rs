//! Scaffolding round a site's upper storey (`building-planner` design 6):
//! a board walk at the storey's floor line outside each outer edge, carried
//! on timber poles at the edge's corners, each pole bound with rope where the
//! boards cross it. Cut in one tangent frame like a plot, so the walker
//! stands on the boards; drawn in the settlement set's poles, planks and
//! rope.
//!
//! A pole is one tapered hexagonal post, its normals smooth round it and
//! hard only at its ends; a board is one bevelled slab. Nothing is placed on
//! a surface as a separate box: the rope is a collar fitted round its pole.

use super::Cells;
use super::cut::frame_at;
use crate::column::layer_altitude;
use crate::settlement::pieces::{BuildingSolids, Frame, MeshBuf, Meshes, Solid};
use glam::{Vec2, Vec3};
use std::collections::BTreeMap;

/// How far out from the wall the boards start and end, metres.
pub const BOARD_IN_M: f32 = 0.35;
pub const BOARD_OUT_M: f32 = 1.05;
/// A board's thickness, metres.
pub const BOARD_M: f32 = 0.06;
/// How far over the boards a pole stands, metres: a hand rail's height.
pub const POLE_OVER_M: f32 = 1.1;
/// A pole's radius at its foot and its head, metres.
pub const POLE_FOOT_M: f32 = 0.07;
pub const POLE_HEAD_M: f32 = 0.05;
/// The rope collar's length along the pole, metres.
pub const ROPE_M: f32 = 0.14;

/// One bay: the outer edge it stands outside, as the two cells it divides
/// (`inside` the storey's, `outside` beyond).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bay {
    pub inside: u32,
    pub outside: u32,
}

/// The scaffolding of `bays` round a storey whose floor is on `floor`, its
/// poles standing on the ground at `ground`: its meshes by texture (points
/// planet-centred) and its solids in a frame at `anchor`. `None` where a
/// bay's cells are not in `cells`.
pub fn scaffold(
    cells: &Cells,
    anchor: u32,
    bays: &[Bay],
    ground: u16,
    floor: u16,
    radius_m: f32,
) -> Option<(Meshes, BuildingSolids)> {
    let frame = frame_at(cells, anchor, ground, radius_m)?;
    let floor_y = layer_altitude(usize::from(floor)) - layer_altitude(usize::from(ground));
    let mut meshes = Meshes::new();
    let mut solids = Vec::new();
    let mut poles: BTreeMap<(i32, i32), Vec2> = BTreeMap::new();
    let mut reach = 0.0f32;
    for bay in bays {
        let (a, b) = cells.edge(bay.inside, bay.outside)?;
        let inside = frame.plane(cells.cell(bay.inside)?.direction);
        let (pa, pb) = (frame.plane(a), frame.plane(b));
        let mid = (pa + pb) * 0.5;
        let out = (mid - inside).normalize_or(Vec2::X);
        // The board: a slab outside the edge, its ends cut square to it.
        let quad = [
            pa + out * BOARD_IN_M,
            pb + out * BOARD_IN_M,
            pb + out * BOARD_OUT_M,
            pa + out * BOARD_OUT_M,
        ];
        slab(
            meshes.entry("plank".into()).or_default(),
            &frame,
            &quad,
            floor_y - BOARD_M,
            floor_y,
        );
        solids.push(Solid {
            outline: ccw(quad.to_vec()),
            y0: floor_y - BOARD_M,
            y1: floor_y,
        });
        // A pole at each end, outside the boards' middle line, shared with
        // the next bay round.
        for p in [pa, pb] {
            let at = p + out * (BOARD_IN_M + BOARD_OUT_M) * 0.5;
            let key = ((at.x * 10.0).round() as i32, (at.y * 10.0).round() as i32);
            poles.entry(key).or_insert(at);
            reach = reach.max(at.length() + 1.0);
        }
    }
    let top = floor_y + POLE_OVER_M;
    for at in poles.values() {
        post(
            meshes.entry("poles".into()).or_default(),
            &frame,
            *at,
            0.0,
            top,
            POLE_FOOT_M,
            POLE_HEAD_M,
        );
        // Bound with rope where the boards cross it.
        let r = POLE_FOOT_M + (POLE_HEAD_M - POLE_FOOT_M) * (floor_y / top) + 0.012;
        post(
            meshes.entry("rope".into()).or_default(),
            &frame,
            *at,
            floor_y - ROPE_M * 0.5,
            floor_y + ROPE_M * 0.5,
            r,
            r,
        );
        let s = POLE_FOOT_M;
        solids.push(Solid {
            outline: ccw(vec![
                *at + Vec2::new(-s, -s),
                *at + Vec2::new(s, -s),
                *at + Vec2::new(s, s),
                *at + Vec2::new(-s, s),
            ]),
            y0: 0.0,
            y1: top,
        });
    }
    Some((
        meshes,
        BuildingSolids {
            frame,
            reach_m: reach,
            solids,
            roof_plan: Vec::new(),
            surfaces: Vec::new(),
            doors: Vec::new(),
            rooms: Meshes::new(),
            lights: Vec::new(),
            top_m: 0.0,
            furniture: Vec::new(),
        },
    ))
}

/// An outline wound counter-clockwise about its middle.
fn ccw(mut points: Vec<Vec2>) -> Vec<Vec2> {
    let area: f32 = points
        .iter()
        .zip(points.iter().cycle().skip(1))
        .map(|(a, b)| a.perp_dot(*b))
        .sum();
    if area < 0.0 {
        points.reverse();
    }
    points
}

fn push(buf: &mut MeshBuf, frame: &Frame, p: Vec3, n: Vec3, uv: Vec2) {
    let w = frame.world(p);
    let wn = (frame.x * n.x + frame.y * n.y + frame.z * n.z).normalize_or_zero();
    buf.positions.push(w.to_array());
    buf.normals.push(wn.to_array());
    buf.uvs.push(uv.to_array());
}

/// A triangle, wound to face `facing`.
fn tri(buf: &mut MeshBuf, frame: &Frame, p: [Vec3; 3], n: [Vec3; 3], uv: [Vec2; 3], facing: Vec3) {
    let order = if (p[1] - p[0]).cross(p[2] - p[0]).dot(facing) >= 0.0 {
        [0, 1, 2]
    } else {
        [0, 2, 1]
    };
    for i in order {
        push(buf, frame, p[i], n[i], uv[i]);
    }
}

/// A quad as two triangles, wound to face `n`.
fn quad(buf: &mut MeshBuf, frame: &Frame, p: [Vec3; 4], n: Vec3, uv: [Vec2; 4]) {
    let facing = (p[1] - p[0]).cross(p[2] - p[0]).dot(n) >= 0.0;
    let order: [usize; 6] = if facing {
        [0, 1, 2, 0, 2, 3]
    } else {
        [0, 2, 1, 0, 3, 2]
    };
    for i in order {
        push(buf, frame, p[i], n, uv[i]);
    }
}

/// A slab over an outline of four points between `y0` and `y1`, its texture
/// laid at a metre a unit along the outline's first side (the boards' run).
fn slab(buf: &mut MeshBuf, frame: &Frame, outline: &[Vec2; 4], y0: f32, y1: f32) {
    let along = (outline[1] - outline[0]).normalize_or(Vec2::X);
    let across = Vec2::new(-along.y, along.x);
    let uv = |p: Vec2| Vec2::new(p.dot(along), p.dot(across)) * 0.5;
    let at = |p: Vec2, y: f32| Vec3::new(p.x, y, p.y);
    let o = outline;
    quad(
        buf,
        frame,
        [at(o[0], y1), at(o[1], y1), at(o[2], y1), at(o[3], y1)],
        Vec3::Y,
        [uv(o[0]), uv(o[1]), uv(o[2]), uv(o[3])],
    );
    quad(
        buf,
        frame,
        [at(o[0], y0), at(o[1], y0), at(o[2], y0), at(o[3], y0)],
        -Vec3::Y,
        [uv(o[0]), uv(o[1]), uv(o[2]), uv(o[3])],
    );
    let middle = (o[0] + o[1] + o[2] + o[3]) * 0.25;
    for k in 0..4 {
        let (a, b) = (o[k], o[(k + 1) % 4]);
        let mid = (a + b) * 0.5;
        let n2 = (mid - middle).normalize_or(Vec2::X);
        let n = Vec3::new(n2.x, 0.0, n2.y);
        let run = (b - a).length() * 0.5;
        let t = (y1 - y0) * 0.5;
        quad(
            buf,
            frame,
            [at(a, y0), at(b, y0), at(b, y1), at(a, y1)],
            n,
            [
                Vec2::new(0.0, 0.0),
                Vec2::new(run, 0.0),
                Vec2::new(run, t),
                Vec2::new(0.0, t),
            ],
        );
    }
}

/// A tapered post of six sides between `y0` and `y1` at `at`, its normals
/// smooth round it, capped flat at the top.
fn post(buf: &mut MeshBuf, frame: &Frame, at: Vec2, y0: f32, y1: f32, r0: f32, r1: f32) {
    const SIDES: usize = 6;
    let ring = |k: usize| {
        let a = k as f32 / SIDES as f32 * std::f32::consts::TAU;
        Vec2::new(a.cos(), a.sin())
    };
    let height = y1 - y0;
    for k in 0..SIDES {
        let (d0, d1) = (ring(k), ring(k + 1));
        let p = |d: Vec2, r: f32, y: f32| Vec3::new(at.x + d.x * r, y, at.y + d.y * r);
        let n = |d: Vec2| Vec3::new(d.x, (r0 - r1) / height.max(1e-3), d.y).normalize();
        let u0 = k as f32 / SIDES as f32;
        let u1 = (k + 1) as f32 / SIDES as f32;
        let pts = [p(d0, r0, y0), p(d1, r0, y0), p(d1, r1, y1), p(d0, r1, y1)];
        let ns = [n(d0), n(d1), n(d1), n(d0)];
        let uvs = [
            Vec2::new(u0 * 0.5, 0.0),
            Vec2::new(u1 * 0.5, 0.0),
            Vec2::new(u1 * 0.5, height * 0.5),
            Vec2::new(u0 * 0.5, height * 0.5),
        ];
        let out = Vec3::new((d0 + d1).x, 0.0, (d0 + d1).y);
        for t in [[0, 1, 2], [0, 2, 3]] {
            tri(
                buf,
                frame,
                t.map(|i| pts[i]),
                t.map(|i| ns[i]),
                t.map(|i| uvs[i]),
                out,
            );
        }
        // The cap.
        let c = Vec3::new(at.x, y1, at.y);
        tri(
            buf,
            frame,
            [c, pts[3], pts[2]],
            [Vec3::Y; 3],
            [Vec2::ZERO, d0 * r1, d1 * r1],
            Vec3::Y,
        );
    }
}
