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
pub fn corners(patch: &Patch, chart: &Chart, frame: &Frame, c: i32, r: i32) -> Option<[Vec2; 6]> {
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

/// How far apart, along a pier, the ground under it is read: a run of its
/// planks starts and ends on one of these.
const PIER_READ_M: f32 = 0.25;

/// One flat piece of a pier's planks, `s0` to `s1` metres along it from
/// `from`, its top `top_m` metres over the radius.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stretch {
    pub s0: f32,
    pub s1: f32,
    pub top_m: f32,
}

/// The stretches a pier is cut in (`pier-landfall`). Its planks stand at
/// the template's deck, `terrace_m` and the pier's height, on piles, where
/// the ground under them is lower than the planks; where the ground
/// reaches them (the beach the template's export rounds up to the deck's
/// layer, a bank of the planet's own ground) they lie on it instead, their
/// underside on the highest ground under them, so the planks are never in
/// the ground. The ground is read every [`PIER_READ_M`] along the pier,
/// across its width; a run of one height is cut in pieces of about
/// [`PIER_STRETCH_M`], each flat in its own frame. A pier over the water
/// all the way is cut as it always was. `ground` is the ground's height at
/// a direction, metres over the radius, the town's where it is laid.
pub fn stretches(
    patch: &Patch,
    chart: &Chart,
    pier: &Pier,
    cell_m: f32,
    terrace_m: f32,
    ground: &dyn Fn(Vec3) -> f32,
) -> Vec<Stretch> {
    let deck_m = terrace_m + pier.from[1];
    let (dx, dz) = (pier.to[0] - pier.from[0], pier.to[2] - pier.from[2]);
    let len = (dx * dx + dz * dz).sqrt();
    if len < 0.1 {
        return Vec::new();
    }
    let (ux, uz) = (dx / len, dz / len);
    // The highest ground across the deck at each reading.
    let reads = ((len / PIER_READ_M).ceil() as usize).max(1);
    let high: Vec<f32> = (0..=reads)
        .map(|i| {
            let s = len * i as f32 / reads as f32;
            [-0.5f32, -0.25, 0.0, 0.25, 0.5]
                .iter()
                .filter_map(|w| {
                    let w = w * pier.width_m;
                    let (x, z) = (
                        pier.from[0] + ux * s - uz * w,
                        pier.from[2] + uz * s + ux * w,
                    );
                    sea::point(chart, patch, x, z, cell_m).map(ground)
                })
                .fold(f32::MIN, f32::max)
        })
        .collect();
    // Each reading's interval's top, and runs of one top.
    let mut runs: Vec<(usize, usize, f32)> = Vec::new();
    for i in 0..reads {
        let g = high[i].max(high[i + 1]);
        let top = if g > deck_m - DECK_M {
            g + DECK_M
        } else {
            deck_m
        };
        match runs.last_mut() {
            Some(run) if run.2 == top => run.1 = i + 1,
            _ => runs.push((i, i + 1, top)),
        }
    }
    let mut out = Vec::new();
    for (i0, i1, top_m) in runs {
        let (r0, r1) = (
            len * i0 as f32 / reads as f32,
            len * i1 as f32 / reads as f32,
        );
        let n = (((r1 - r0) / PIER_STRETCH_M).round() as usize).max(1);
        for k in 0..n {
            out.push(Stretch {
                s0: r0 + (r1 - r0) * k as f32 / n as f32,
                s1: r0 + (r1 - r0) * (k + 1) as f32 / n as f32,
                top_m,
            });
        }
    }
    out
}

/// How far along a pier the mockup's point `(x, z)` is, metres from its
/// `from`.
pub fn along(pier: &Pier, x: f32, z: f32) -> f32 {
    let (dx, dz) = (pier.to[0] - pier.from[0], pier.to[2] - pier.from[2]);
    let len = (dx * dx + dz * dz).sqrt().max(1e-6);
    ((x - pier.from[0]) * dx + (z - pier.from[2]) * dz) / len
}

/// The top of the stretch `s` metres along a pier, of `stretches`: the
/// last one starting at or before it.
pub fn top_at(stretches: &[Stretch], s: f32) -> Option<f32> {
    stretches
        .iter()
        .rev()
        .find(|t| t.s0 <= s)
        .or(stretches.first())
        .map(|t| t.top_m)
}

/// A pier (the mockup's `bridge` with piles): planks `width_m` wide from
/// one end to the other, cut in the [`stretches`] the ground under it
/// gives, each flat in its own frame and lapping the next: at its height
/// over `terrace_m` with a pile either side at every joint down to the
/// ground, or lying on the ground where the ground reaches its planks
/// (`pier-landfall`). Its edges are open. Ends are the mockup's metres,
/// placed on the chart as [`sea::point`] places them. Where two stretches
/// differ by a riser the walker does not hold, a timber stair stands on the
/// lower one against it ([`stairs`], `path-stairs`), unless a cell under it
/// is `yielded` (holds a player's edit, by exact key).
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
    yielded: &dyn Fn(u32) -> bool,
) -> Result<Vec<BuildingSolids>, String> {
    let (from, to) = (pier.from, pier.to);
    let (dx, dz) = (to[0] - from[0], to[2] - from[2]);
    let len = (dx * dx + dz * dz).sqrt();
    if len < 0.1 {
        return Ok(Vec::new());
    }
    let (ux, uz) = (dx / len, dz / len);
    let at = |s: f32| -> Result<Vec3, String> {
        let (x, z) = (from[0] + ux * s, from[2] + uz * s);
        sea::point(chart, patch, x, z, cell_m)
            .ok_or_else(|| format!("a pier at ({x:.1}, {z:.1}) is off the chart"))
    };
    let cut = stretches(patch, chart, pier, cell_m, terrace_m, ground);
    let mut out = Vec::with_capacity(cut.len());
    for (k, stretch) in cut.iter().enumerate() {
        let Stretch { s0, s1, top_m } = *stretch;
        let (a, b) = (at(s0 - PIER_LAP_M)?, at(s1 + PIER_LAP_M)?);
        let y = (a + b).normalize();
        let along = b - a;
        let x = (along - y * along.dot(y)).normalize();
        let frame = Frame {
            origin: y * (radius_m + top_m),
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
        // Piles at the joint this stretch starts at, and at its end where
        // the next stretch is at another height or there is none. A stretch
        // on the ground gets none: the ground is under its planks.
        let inset = side.normalize() * (pier.width_m / 2.0 - 0.08);
        let ends = cut.get(k + 1).is_none_or(|next| next.top_m != top_m);
        let joints: &[f32] = if ends { &[s0, s1] } else { &[s0] };
        for &s in joints {
            let at = frame.plane(at(s)?);
            for p in [at + inset, at - inset] {
                pile(&mut sink, p.x, p.y, -DECK_M, ground, top_m);
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
    for stair in stairs(&cut) {
        // The cells under it, read at its foot, middle and head across its
        // width: a player's edit there keeps it out.
        let held = [0.0f32, 0.5, 1.0].iter().any(|&f| {
            let s = stair.head + (stair.foot - stair.head) * f;
            [-0.5f32, 0.0, 0.5].iter().any(|&w| {
                let w = w * pier.width_m;
                let (x, z) = (from[0] + ux * s - uz * w, from[2] + uz * s + ux * w);
                let (c, r) = sea::cell_at(x, z, cell_m);
                chart.cell(c, r).is_none_or(|i| yielded(patch.keys[i]))
            })
        });
        if held {
            continue;
        }
        let (h, f) = (at(stair.head)?, at(stair.foot)?);
        let y = (h + f).normalize();
        let along = h - f;
        let x = (along - y * along.dot(y)).normalize();
        let frame = Frame {
            origin: y * (radius_m + stair.base_m),
            x,
            y,
            z: x.cross(y),
        };
        let (ph, pf) = (frame.plane(h), frame.plane(f));
        let dir = (ph - pf).normalize();
        let mut sink = Sink::new(meshes, repeat_m, frame);
        super::steps::steps_along(
            &mut sink,
            ph,
            dir,
            stair.rise_m,
            stair.run_m,
            pier.width_m - 0.2,
            ("plank", "timber"),
        );
        let reach_m = ph.length().max(pf.length()) + pier.width_m + 1.0;
        out.push(super::dressing::finish(sink, frame, reach_m));
    }
    Ok(out)
}

/// A stair on a pier (`path-stairs`): on the lower of two stretches that
/// differ by more than the walker holds, its head at their joint, `head`
/// metres along the pier, and its foot at `foot`, toward the lower run; it
/// climbs `rise_m` from the lower top, `base_m` over the radius, on treads of
/// `run_m`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PierStair {
    pub head: f32,
    pub foot: f32,
    pub base_m: f32,
    pub rise_m: f32,
    pub run_m: f32,
}

/// The stairs a pier cut in `cut` stands: one against each joint where the
/// planks step by more than [`super::steps::HOLD_M`] and no more than
/// [`super::steps::MAX_RISE_M`], on the lower side, its treads
/// [`super::steps::FLIGHT_RUN_M`] or shorter to fit the run of planks at
/// the lower height (0.22 m at the least).
pub fn stairs(cut: &[Stretch]) -> Vec<PierStair> {
    use super::steps::{FLIGHT_RISE_M, FLIGHT_RUN_M, HOLD_M, MAX_RISE_M};
    let mut out = Vec::new();
    for k in 0..cut.len().saturating_sub(1) {
        let (a, b) = (cut[k], cut[k + 1]);
        let rise = (b.top_m - a.top_m).abs();
        if rise <= HOLD_M || rise > MAX_RISE_M {
            continue;
        }
        let risers = ((rise / FLIGHT_RISE_M).round() as usize).max(2);
        // The run of planks at the lower height, from the joint.
        let (head, base, room) = if b.top_m > a.top_m {
            let start = cut[..=k]
                .iter()
                .rev()
                .take_while(|t| t.top_m == a.top_m)
                .last()
                .map_or(a.s0, |t| t.s0);
            (b.s0, a.top_m, b.s0 - start)
        } else {
            let end = cut[k + 1..]
                .iter()
                .take_while(|t| t.top_m == b.top_m)
                .last()
                .map_or(b.s1, |t| t.s1);
            (a.s1, b.top_m, end - a.s1)
        };
        let run = (room / risers as f32).clamp(0.22, FLIGHT_RUN_M);
        let len = run * risers as f32;
        let foot = if b.top_m > a.top_m {
            head - len
        } else {
            head + len
        };
        out.push(PierStair {
            head,
            foot,
            base_m: base,
            rise_m: rise,
            run_m: run,
        });
    }
    out
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
