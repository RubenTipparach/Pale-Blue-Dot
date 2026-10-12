//! What a harbour stands over the water (`cities-in-the-world` slice 4d):
//! a fish hut's stilts, deck and porch stair, the piers on their piles, and
//! the light on the mole. Each is cut in its own frame, as a building is,
//! and its piles reach down to the ground under them, the planet's own
//! where the harbour left it.

use super::super::chart::{Chart, Patch};
use super::super::sea;
use super::super::{Pier, RoundTower, Stilts, neighbour};
use super::{BuildingSolids, Frame, LIFT_M, Meshes, SLAB_M, Sink, Solid, Surface, ccw, rail};
use glam::{Vec2, Vec3};
use std::collections::BTreeSet;

/// A pile's side, metres.
const PILE_M: f32 = 0.24;
/// A pier's planks: their depth under the deck's top.
const DECK_M: f32 = 0.12;
/// The longest stretch of a pier cut in one frame. A pier is cut in
/// stretches about this long, each flat in its own frame, a pile pair at
/// every joint: the mockup's piles every `L / 2.2`.
const PIER_STRETCH_M: f32 = 2.2;
/// How far a pier's stretch reaches past its joint into the next one, so
/// the planks have no seam between two frames.
const PIER_LAP_M: f32 = 0.05;
/// A porch stair's riser and tread, and its width.
const PORCH_RISE_M: f32 = 0.19;
const PORCH_RUN_M: f32 = 0.3;
const PORCH_WIDTH_M: f32 = 1.1;

/// A cell's corners in a frame, in the mockup's corner order (edge `d` from
/// corner `d` to `d + 1`), as `cut_building` takes them.
pub(super) fn corners(
    patch: &Patch,
    chart: &Chart,
    frame: &Frame,
    c: i32,
    r: i32,
) -> Option<[Vec2; 6]> {
    let at = chart.cells.get(&(c, r))?;
    let cell = &patch.cells[at.cell];
    let mut out = [Vec2::ZERO; 6];
    for (d, corner) in out.iter_mut().enumerate() {
        let s = chart.side(c, r, d)?;
        *corner = frame.plane(cell.corners[(s + 1) % 6]);
    }
    Some(out)
}

/// A pile in `sink`'s frame at `(x, z)` in plan, from the ground under it
/// up to `top`, as a solid the walker goes round. `ground` is the ground's
/// height at a direction, metres over the radius, and `floor_m` the
/// frame's origin's.
fn pile(sink: &mut Sink, x: f32, z: f32, top: f32, ground: &dyn Fn(Vec3) -> f32, floor_m: f32) {
    let at = sink.frame.world(Vec3::new(x, 0.0, z));
    let bottom = ground(at.normalize()) - floor_m;
    if bottom >= top - 0.05 {
        return;
    }
    let s = Vec3::new(PILE_M, top - bottom, PILE_M);
    sink.plain_box("timber", x, bottom, z, s, 0.0);
    sink.solid_box(x, bottom, z, s, 0.0);
}

/// A house on stilts (the mockup's `stiltHouse`): its floor a slab the
/// walker stands on, a pile at every outside corner down to the ground, a
/// deck on the same piles at its door, railed but at the stair, and an
/// open stair from the deck down to `stilts.foot_m` over the town's
/// terrace. Added to the building's own cut, `building`, in its frame.
#[allow(clippy::too_many_arguments)]
pub fn stilts(
    building: &mut BuildingSolids,
    meshes: &mut Meshes,
    repeat_m: &dyn Fn(&str) -> f32,
    patch: &Patch,
    chart: &Chart,
    cells: &[[i32; 2]],
    stilts: &Stilts,
    floor_m: f32,
    foot_m: f32,
    ground: &dyn Fn(Vec3) -> f32,
) -> Result<(), String> {
    let frame = building.frame;
    let mut sink = Sink::new(meshes, repeat_m, frame);
    let house: BTreeSet<(i32, i32)> = cells.iter().map(|&[c, r]| (c, r)).collect();
    let deck: BTreeSet<(i32, i32)> = stilts.deck.iter().map(|&[c, r]| (c, r)).collect();
    let mut posts: Vec<Vec2> = Vec::new();
    let mut far = 0.0f32;
    for &(c, r) in house.iter().chain(&deck) {
        let hex = corners(patch, chart, &frame, c, r)
            .ok_or_else(|| format!("stilts: cell ({c}, {r}) is not charted"))?;
        far = hex.iter().map(|p| p.length()).fold(far, f32::max);
        // The floor: the house's planks are its kit's, laid by the cutter
        // on the ground it does not have, so they get an underside and the
        // walker a floor; the deck is planks.
        let top = if deck.contains(&(c, r)) {
            sink.prism("plank", "timber", &hex, -SLAB_M, LIFT_M, Some("plank"));
            LIFT_M
        } else {
            sink.prism("plank", "timber", &hex, -SLAB_M, -0.05, Some("plank"));
            LIFT_M
        };
        sink.surfaces.push(Surface::Floor {
            outline: ccw(hex.to_vec()),
            top,
            bottom: -SLAB_M,
        });
        for d in 0..6 {
            let n = neighbour(c, r, d);
            if house.contains(&n) || deck.contains(&n) {
                continue;
            }
            let (a, b) = (hex[d], hex[(d + 1) % 6]);
            for p in [a, b] {
                if posts.iter().all(|q| q.distance(p) > 0.3) {
                    posts.push(p);
                }
            }
            if deck.contains(&(c, r)) && [c, r, d as i32] != stilts.porch {
                let e = b - a;
                rail(&mut sink, (a + b) * 0.5, LIFT_M, e.length(), e.y.atan2(e.x));
            }
        }
    }
    for p in posts {
        pile(&mut sink, p.x, p.y, -SLAB_M, ground, floor_m);
    }
    // The porch stair: square off the deck's edge, down to its foot.
    let [pc, pr, pd] = stilts.porch;
    let hex = corners(patch, chart, &frame, pc, pr)
        .ok_or_else(|| format!("stilts: the porch's cell ({pc}, {pr}) is not charted"))?;
    let (a, b) = (hex[pd as usize % 6], hex[(pd as usize + 1) % 6]);
    let centre = hex.iter().fold(Vec2::ZERO, |s, p| s + *p) / 6.0;
    let mid = (a + b) * 0.5;
    let out = (mid - centre).normalize();
    let drop = (floor_m - foot_m).max(0.0);
    let risers = ((drop / PORCH_RISE_M).round() as u32).max(1);
    let len = risers as f32 * PORCH_RUN_M;
    let foot = mid + out * len;
    let dir = -out;
    let rise = drop / risers as f32;
    let ang = dir.y.atan2(dir.x);
    for k in 0..risers {
        let y = -drop + (k + 1) as f32 * rise;
        let p = foot + dir * ((k as f32 + 0.5) * PORCH_RUN_M);
        sink.plain_box(
            "plank",
            p.x,
            y - 0.06,
            p.y,
            Vec3::new(PORCH_RUN_M, 0.06, PORCH_WIDTH_M),
            ang,
        );
    }
    sink.surfaces.push(Surface::Flight {
        foot,
        dir,
        len,
        half_width: PORCH_WIDTH_M / 2.0,
        base: -drop,
        rise,
        risers,
    });
    building.solids.extend(sink.solids);
    building.surfaces.extend(sink.surfaces);
    building.reach_m = building.reach_m.max(far.max(foot.length()) + 1.0);
    Ok(())
}

/// A pier (the mockup's `bridge` with piles): planks `width_m` wide from
/// one end to the other at its height over `terrace_m`, cut in stretches of
/// about [`PIER_STRETCH_M`], each flat in its own frame and lapping the
/// next, with a pile either side at every joint down to the ground. Its
/// edges are open. Ends are the mockup's metres, placed on the chart as
/// [`sea::point`] places them.
#[allow(clippy::too_many_arguments)]
pub fn pier(
    meshes: &mut Meshes,
    repeat_m: &dyn Fn(&str) -> f32,
    patch: &Patch,
    chart: &Chart,
    pier: &Pier,
    cell_m: f32,
    radius_m: f32,
    terrace_m: f32,
    ground: &dyn Fn(Vec3) -> f32,
) -> Result<Vec<BuildingSolids>, String> {
    let (from, to) = (pier.from, pier.to);
    let (dx, dz) = (to[0] - from[0], to[2] - from[2]);
    let len = (dx * dx + dz * dz).sqrt();
    if len < 0.1 {
        return Ok(Vec::new());
    }
    let (ux, uz) = (dx / len, dz / len);
    let stretches = ((len / PIER_STRETCH_M).round() as usize).max(1);
    let deck_m = terrace_m + from[1];
    let at = |s: f32| -> Result<Vec3, String> {
        let (x, z) = (from[0] + ux * s, from[2] + uz * s);
        sea::point(chart, patch, x, z, cell_m)
            .ok_or_else(|| format!("a pier at ({x:.1}, {z:.1}) is off the chart"))
    };
    let mut out = Vec::with_capacity(stretches);
    for k in 0..stretches {
        let s0 = len * k as f32 / stretches as f32;
        let s1 = len * (k + 1) as f32 / stretches as f32;
        let (a, b) = (at(s0 - PIER_LAP_M)?, at(s1 + PIER_LAP_M)?);
        let y = (a + b).normalize();
        let along = b - a;
        let x = (along - y * along.dot(y)).normalize();
        let frame = Frame {
            origin: y * (radius_m + deck_m),
            x,
            y,
            z: x.cross(y),
        };
        let (pa, pb) = (frame.plane(a), frame.plane(b));
        let side = (pb - pa).perp().normalize() * (pier.width_m / 2.0);
        let rect = ccw(vec![pa - side, pb - side, pb + side, pa + side]);
        let mut sink = Sink::new(meshes, repeat_m, frame);
        sink.prism("plank", "timber", &rect, -DECK_M, 0.0, Some("timber"));
        sink.surfaces.push(Surface::Floor {
            outline: rect.clone(),
            top: 0.0,
            bottom: -DECK_M,
        });
        // Piles at the joint this stretch starts at, and the last one's end.
        let inset = side.normalize() * (pier.width_m / 2.0 - 0.08);
        let joints: &[f32] = if k + 1 == stretches {
            &[0.0, 1.0]
        } else {
            &[0.0]
        };
        for &t in joints {
            let at = frame.plane(if t == 0.0 { at(s0)? } else { at(s1)? });
            for p in [at + inset, at - inset] {
                pile(&mut sink, p.x, p.y, -DECK_M, ground, deck_m);
            }
        }
        let reach_m = rect.iter().map(|p| p.length()).fold(0.0f32, f32::max) + 1.0;
        out.push(BuildingSolids {
            frame,
            reach_m,
            solids: sink.solids,
            roof_plan: Vec::new(),
            surfaces: sink.surfaces,
            doors: Vec::new(),
            top_m: 0.0,
            rooms: Meshes::new(),
            lights: Vec::new(),
        });
    }
    Ok(out)
}

/// The harbour's light (the mockup's on its mole): a round tower of stone,
/// solid to its flagged top, an iron cage over it and a slate cap.
#[allow(clippy::too_many_arguments)]
pub fn light(
    meshes: &mut Meshes,
    repeat_m: &dyn Fn(&str) -> f32,
    patch: &Patch,
    chart: &Chart,
    tower: &RoundTower,
    cell_m: f32,
    radius_m: f32,
    terrace_m: f32,
) -> Result<BuildingSolids, String> {
    let y =
        sea::point(chart, patch, tower.x, tower.z, cell_m).ok_or("the light is off the chart")?;
    let toward = sea::point(chart, patch, tower.x + 1.0, tower.z, cell_m)
        .ok_or("the light is off the chart")?;
    let along = toward - y;
    let x = (along - y * along.dot(y)).normalize();
    let frame = Frame {
        origin: y * (radius_m + terrace_m + tower.base_m as f32),
        x,
        y,
        z: x.cross(y),
    };
    let ring = |r: f32, n: usize| -> Vec<Vec2> {
        (0..n)
            .map(|k| {
                let a = k as f32 / n as f32 * std::f32::consts::TAU;
                Vec2::new(a.cos(), a.sin()) * r
            })
            .collect()
    };
    let h = tower.top_m - tower.base_m as f32;
    let mut sink = Sink::new(meshes, repeat_m, frame);
    let body = ring(tower.radius_m, 12);
    sink.prism("flag", "stone", &body, 0.0, h, None);
    sink.solids.push(Solid {
        outline: ccw(body.clone()),
        y0: 0.0,
        y1: h,
    });
    sink.surfaces.push(Surface::Floor {
        outline: ccw(body),
        top: h,
        bottom: h - 0.3,
    });
    for p in ring(1.0, 6) {
        sink.plain_box("iron", p.x, h, p.y, Vec3::new(0.08, 1.6, 0.08), 0.0);
    }
    sink.prism(
        "slate",
        "slate",
        &ring(1.3, 10),
        h + 1.6,
        h + 1.85,
        Some("slate"),
    );
    sink.prism("slate", "slate", &ring(0.8, 10), h + 1.85, h + 2.25, None);
    Ok(BuildingSolids {
        frame,
        reach_m: tower.radius_m + 2.0,
        solids: sink.solids,
        roof_plan: Vec::new(),
        surfaces: sink.surfaces,
        doors: Vec::new(),
        top_m: 0.0,
        rooms: Meshes::new(),
        lights: Vec::new(),
    })
}
