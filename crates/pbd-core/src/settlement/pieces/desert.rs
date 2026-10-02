//! The desert's pieces outside its buildings (`cities-in-the-world` slice
//! 4e): the sandstone houses' stairs up to their roofs.

use super::super::OutsideStair;
use super::super::chart::{Chart, Patch};
use super::dressing::{Place, finish};
use super::{BuildingSolids, Meshes, Sink, Solid, Surface, ccw};
use glam::{Vec2, Vec3};

/// The rise the mockup's `stairRun` divides a flight by.
const RISE_M: f32 = 0.19;
/// How high a walker steps up without climbing: a step lower than this is
/// stepped onto from beside the flight, and only the part of a step over it
/// stops a walker beside it (the walker's own climb).
const STEP_UP_M: f32 = 1.05;
/// How far the walker's floor at a stair's head runs on over the roof.
const LANDING_M: f32 = 0.4;

/// An outside stair, the mockup's `stairRun`: solid steps of its material
/// from the ground at its foot, each up to its tread, `width_m` wide, from
/// `from` up to `to`, the mockup's metres. It is cut flat in a frame at its
/// foot, `zero_m` being the game's height of the template's 0 m, and the
/// walker climbs it as a straight flight. A step taller than a walker
/// steps is solid to that height under its tread, so it is not walked
/// into from beside the flight.
///
/// `head`, the `[c, r, d]` edge of the roof it climbs to, ends it on that
/// edge's real corners, 5 cm in over the roof: a house is cut on its
/// cells' real corners, and the mockup's metres alone left a 16 cm gap
/// between the last tread and the roof.
#[allow(clippy::too_many_arguments)]
pub fn outside_stair(
    meshes: &mut Meshes,
    repeat_m: &dyn Fn(&str) -> f32,
    patch: &Patch,
    chart: &Chart,
    stair: &OutsideStair,
    cell_m: f32,
    radius_m: f32,
    zero_m: f32,
    head: Option<[i32; 3]>,
) -> Option<BuildingSolids> {
    let [x0, y0, z0] = stair.from;
    let [x1, y1, z1] = stair.to;
    let place = Place::new(chart, patch, cell_m, (x0, z0), radius_m, zero_m + y0)?;
    let foot = place.plan(x0, z0);
    let to = head
        .and_then(|[c, r, d]| {
            let hex = super::harbour::corners(patch, chart, &place.frame, c, r)?;
            let mid = (hex[d as usize % 6] + hex[(d as usize + 1) % 6]) * 0.5;
            let centre = hex.iter().fold(Vec2::ZERO, |s, p| s + *p) / 6.0;
            Some(mid + (centre - mid).normalize_or_zero() * 0.05)
        })
        .unwrap_or_else(|| place.plan(x1, z1));
    let along = to - foot;
    let len = along.length();
    let height = y1 - y0;
    if len < 0.1 || height <= 0.0 {
        return None;
    }
    let dir = along / len;
    let risers = ((height / RISE_M).round() as u32).max(2);
    let (rise, run) = (height / risers as f32, len / risers as f32);
    let ang = dir.y.atan2(dir.x);
    let across = Vec2::new(-dir.y, dir.x) * (stair.width_m / 2.0);
    let mut sink = Sink::new(meshes, repeat_m, place.frame);
    for k in 0..risers {
        let top = (k + 1) as f32 * rise;
        let p = foot + dir * ((k as f32 + 0.5) * run);
        sink.plain_box(
            &stair.material,
            p.x,
            0.0,
            p.y,
            Vec3::new(run + 0.01, top, stair.width_m),
            ang,
        );
        if top > STEP_UP_M + 0.05 {
            let half = dir * (run / 2.0);
            sink.solids.push(Solid {
                outline: ccw(vec![
                    p - half - across,
                    p + half - across,
                    p + half + across,
                    p - half + across,
                ]),
                y0: 0.0,
                y1: top - STEP_UP_M,
            });
        }
    }
    sink.surfaces.push(Surface::Flight {
        foot,
        dir,
        len,
        half_width: stair.width_m / 2.0,
        base: 0.0,
        rise,
        risers,
    });
    // A landing [`LANDING_M`] on past the head, drawn by the roof: the roof
    // is the house's, flat in the house's own frame, and the two frames
    // part by up to 10 cm here on the smallest body.
    let (a, b) = (to - across, to + across);
    sink.surfaces.push(Surface::Floor {
        outline: ccw(vec![a, b, b + dir * LANDING_M, a + dir * LANDING_M]),
        top: height,
        bottom: height - 0.3,
    });
    Some(finish(sink, place.frame, len + 1.0))
}
