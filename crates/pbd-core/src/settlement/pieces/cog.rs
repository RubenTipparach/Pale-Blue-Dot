//! The harbour's cog at its mooring (`sail-the-cog` design 6, step 1): the
//! ship piece (`tenebris-towns` 3d) as the mockup's `cog` draws it, 15 m by
//! 5 m with its deck 1.9 m over the water. Its hull, deck, rail with the
//! gangway gap, castles, stair to the aftcastle, mast with the sail furled
//! on its yard, shrouds, a barrel and a crate, and the gangplank up from
//! the pier. It is cut on the harbour's chart in its own frame at the sea's
//! surface, placed from the template and never saved, and it does not move.

use super::super::chart::{Chart, Patch};
use super::super::{Cog, Template};
use super::dressing::{Hull, Place, both, cylinder, finish, hull_skin, ramp};
use super::{BuildingSolids, Frame, Meshes, Sink, Solid, Surface, ccw, rail};
use glam::{Vec2, Vec3};

/// The cog's hull (the mockup's `L`, `B`, `D` and the sheer it lofts).
const HULL: Hull = Hull {
    l: 15.0,
    b: 5.0,
    d: 3.2,
    sheer: 1.0,
};
/// How deep it floats, metres.
const DRAFT_M: f32 = 1.2;
/// Its deck over the water: the hull's depth less its draught, and a hand
/// under the gunwale's middle.
const DECK_M: f32 = HULL.d - DRAFT_M - 0.1;
/// Its castles' decks over the main deck.
const CASTLE_M: f32 = 1.6;
/// The gangway's half-width in the rail amidships.
const GANGWAY_M: f32 = 0.7;
/// How far the gangplank's landing laps onto the deck.
const LANDING_M: f32 = 0.3;

/// The deck's half-beam `lx` along the cog from its middle: the hull's own
/// plan, a little inside it (the mockup's `wAt`).
fn beam_at(lx: f32) -> f32 {
    let (_, b, _, _) = HULL.section(lx / HULL.l + 0.5);
    b * 0.96
}

/// The cog's ship, cut where the template moors it. `terrace_m` is the
/// sea's surface over the radius, which a harbour's terrace is. `None`
/// where it is off the chart.
#[allow(clippy::too_many_arguments)]
pub fn ship(
    meshes: &mut Meshes,
    repeat_m: &dyn Fn(&str) -> f32,
    patch: &Patch,
    chart: &Chart,
    template: &Template,
    cog: &Cog,
    radius_m: f32,
    terrace_m: f32,
) -> Option<BuildingSolids> {
    let cell_m = template.grid.cell_m;
    let place = Place::new(chart, patch, cell_m, (cog.x, cog.z), radius_m, terrace_m)?;
    Some(cut(meshes, repeat_m, &place, cog))
}

/// Which way the cog's bow points where the template moors it, a
/// planet-local unit tangent: the axis it rolls about.
pub fn bow(patch: &Patch, chart: &Chart, template: &Template, cog: &Cog) -> Option<Vec3> {
    let cell_m = template.grid.cell_m;
    let place = Place::new(chart, patch, cell_m, (cog.x, cog.z), 1.0, 0.0)?;
    let t = place.turn(cog.heading);
    let f = place.frame;
    Some((f.x * t.cos() + f.z * t.sin()).normalize())
}

/// The cog's ship cut in `frame` as it stands, its waterline's middle at the
/// frame's origin and its bow along `heading` in the frame's plan (from `x`
/// toward `z`): a deck off any chart, for a walker's tests.
pub fn ship_in(
    meshes: &mut Meshes,
    repeat_m: &dyn Fn(&str) -> f32,
    frame: Frame,
    heading: f32,
    gang_side: i32,
) -> BuildingSolids {
    let cog = Cog {
        x: 0.0,
        z: 0.0,
        heading,
        gang_side,
        gangplank: super::super::Pier {
            from: [0.0; 3],
            to: [0.0; 3],
            width_m: 1.0,
        },
    };
    cut(meshes, repeat_m, &Place::flat(frame), &cog)
}

/// The ship, in `place`'s frame.
fn cut(
    meshes: &mut Meshes,
    repeat_m: &dyn Fn(&str) -> f32,
    place: &Place,
    cog: &Cog,
) -> BuildingSolids {
    let (ca, sa) = (cog.heading.cos(), cog.heading.sin());
    // The cog's own `(along, across)` in the mockup's metres (its `tw`), and
    // in the frame's plan.
    let tw = |lx: f32, lz: f32| (cog.x + lx * ca - lz * sa, cog.z + lx * sa + lz * ca);
    let plan = |lx: f32, lz: f32| {
        let (x, z) = tw(lx, lz);
        place.plan(x, z)
    };
    let turn = place.turn(cog.heading);
    let mut sink = Sink::new(meshes, repeat_m, place.frame);

    // The hull, its gunwale just under the deck, so the gangplank clears it.
    let gunwale = DECK_M - 0.1;
    let to = |p: Vec3| {
        let (x, z) = tw(p.x, p.z);
        place.at(x, gunwale + p.y, z)
    };
    hull_skin(&mut sink, "boards", HULL, 1.0, &to);

    // The deck, following the hull's plan: a floor, and under it the hull
    // as a solid down to its keel.
    let hl = HULL.l / 2.0 * 0.94;
    let n = 14;
    let side: Vec<(f32, f32)> = (0..=n)
        .map(|i| {
            let lx = -hl + 2.0 * hl * i as f32 / n as f32;
            (lx, beam_at(lx))
        })
        .collect();
    let deck: Vec<(f32, f32)> = side
        .iter()
        .map(|&(a, b)| (a, -b))
        .chain(side.iter().rev().map(|&(a, b)| (a, b)))
        .fold(Vec::new(), |mut out: Vec<(f32, f32)>, p| {
            if out
                .last()
                .is_none_or(|q| (p.0 - q.0).hypot(p.1 - q.1) > 1e-3)
            {
                out.push(p);
            }
            out
        });
    let outline: Vec<Vec2> = deck.iter().map(|&(a, b)| plan(a, b)).collect();
    let floor: Vec<Vec3> = outline
        .iter()
        .map(|p| Vec3::new(p.x, DECK_M, p.y))
        .collect();
    sink.face("plank", &floor, Vec3::Y, None);
    sink.surfaces.push(Surface::Floor {
        outline: ccw(outline.clone()),
        top: DECK_M,
        bottom: DECK_M - 0.1,
    });
    sink.solids.push(Solid {
        outline: ccw(outline),
        y0: DECK_M - HULL.d - 0.2,
        y1: DECK_M - 0.1,
    });

    // The rail round the deck, with the gangway's gap amidships.
    let rail_between = |sink: &mut Sink, (a, b): (f32, f32), (c, d): (f32, f32), y: f32| {
        let (p, q) = (plan(a, b), plan(c, d));
        let e = q - p;
        if e.length() > 0.05 {
            rail(sink, (p + q) * 0.5, y, e.length(), e.y.atan2(e.x));
        }
    };
    let gang = cog.gang_side as f32;
    for i in 0..deck.len() {
        let ((a, b), (c, d)) = (deck[i], deck[(i + 1) % deck.len()]);
        let across = (b + d).signum();
        if across == gang && a.min(c) < GANGWAY_M && a.max(c) > -GANGWAY_M {
            let at = |u: f32| (u, b + (d - b) * (u - a) / (c - a));
            let (lo, hi) = (a.min(c), a.max(c));
            if lo < -GANGWAY_M {
                rail_between(&mut sink, at(lo), at(-GANGWAY_M), DECK_M);
            }
            if hi > GANGWAY_M {
                rail_between(&mut sink, at(GANGWAY_M), at(hi), DECK_M);
            }
            continue;
        }
        rail_between(&mut sink, (a, b), (c, d), DECK_M);
    }

    // The castles: solid decks over the ends, railed but on the side that
    // looks onto the main deck.
    let castle = |sink: &mut Sink, x0: f32, x1: f32, open: usize| {
        let lxs: Vec<f32> = (0..=4).map(|k| x0 + (x1 - x0) * k as f32 / 4.0).collect();
        let pts: Vec<(f32, f32)> = lxs
            .iter()
            .map(|&lx| (lx, -beam_at(lx) * 0.97))
            .chain(lxs.iter().rev().map(|&lx| (lx, beam_at(lx) * 0.97)))
            .collect();
        let outline: Vec<Vec2> = pts.iter().map(|&(a, b)| plan(a, b)).collect();
        sink.prism("plank", "boards", &outline, DECK_M, DECK_M + CASTLE_M, None);
        sink.solids.push(Solid {
            outline: ccw(outline.clone()),
            y0: DECK_M,
            y1: DECK_M + CASTLE_M,
        });
        sink.surfaces.push(Surface::Floor {
            outline: ccw(outline),
            top: DECK_M + CASTLE_M,
            bottom: DECK_M + CASTLE_M - 0.1,
        });
        for i in (0..pts.len()).filter(|&i| i != open) {
            rail_between(sink, pts[i], pts[(i + 1) % pts.len()], DECK_M + CASTLE_M);
        }
    };
    let aft_x = -hl * 0.36;
    castle(&mut sink, -hl * 0.97, aft_x, 4);
    castle(&mut sink, hl * 0.42, hl * 0.97, 9);

    // The stair up to the aftcastle, open treads on the deck's middle line.
    let (foot, top) = (plan(aft_x + 2.55, 0.0), plan(aft_x, 0.0));
    let len = foot.distance(top);
    let dir = (top - foot) / len;
    let risers = (CASTLE_M / 0.19).round() as u32;
    let (rise, run) = (CASTLE_M / risers as f32, len / risers as f32);
    let stair_turn = dir.y.atan2(dir.x);
    for k in 0..risers {
        let p = foot + dir * ((k as f32 + 0.5) * run);
        let y = DECK_M + (k + 1) as f32 * rise;
        sink.plain_box(
            "plank",
            p.x,
            y - 0.07,
            p.y,
            Vec3::new(run + 0.02, 0.07, 1.0),
            stair_turn,
        );
    }
    sink.surfaces.push(Surface::Flight {
        foot,
        dir,
        len,
        half_width: 0.5,
        base: DECK_M,
        rise,
        risers,
    });

    // The mast, the yard across the ship with its sail furled on it, and
    // the shrouds from the masthead to the rail either side.
    let mast = plan(1.5, 0.0);
    cylinder(&mut sink, "timber", "timber", mast, DECK_M, 0.28, 13.0, 8);
    sink.plain_box(
        "timber",
        mast.x,
        DECK_M + 11.0,
        mast.y,
        Vec3::new(0.25, 0.25, 9.5),
        turn,
    );
    let furl = plan(1.7, 0.0);
    sink.plain_box(
        "linen",
        furl.x,
        DECK_M + 10.55,
        furl.y,
        Vec3::new(0.45, 0.45, 8.8),
        turn,
    );
    let head = Vec3::new(mast.x, DECK_M + 12.6, mast.y);
    let ahead = plan(1.56, 0.0);
    let head2 = Vec3::new(ahead.x, DECK_M + 12.6, ahead.y);
    for k in [-1.0, 1.0] {
        for f in [-0.6, 0.6] {
            let p = plan(1.5 + f, k * beam_at(1.5 + f));
            let foot = Vec3::new(p.x, DECK_M + 0.95, p.y);
            let uv = |q: Vec3| Vec2::new(0.0, q.y);
            both(&mut sink, "rope", &[head, head2, foot], &uv);
        }
    }

    // A barrel and a crate on the deck.
    let b = plan(-1.0, -1.3);
    cylinder(&mut sink, "barrel", "timber", b, DECK_M, 0.34, 0.9, 10);
    let c = plan(2.1, -1.5);
    let size = Vec3::splat(0.6);
    sink.plain_box("timber", c.x, DECK_M, c.y, size, turn);
    sink.solid_box(c.x, DECK_M, c.y, size, turn);

    finish(sink, place.frame, HULL.l / 2.0 + 1.0)
}

/// The cog's gangplank, from the pier up to its deck. The mockup ends it 2
/// cm over the deck's edge, which two frames on the chart do not keep: a
/// landing the walker stands on, not drawn, laps [`LANDING_M`] onto the
/// deck at its height, as a pier's stretches lap. `None` where it is off
/// the chart.
#[allow(clippy::too_many_arguments)]
pub fn gangplank(
    meshes: &mut Meshes,
    repeat_m: &dyn Fn(&str) -> f32,
    patch: &Patch,
    chart: &Chart,
    template: &Template,
    cog: &Cog,
    radius_m: f32,
    terrace_m: f32,
) -> Option<BuildingSolids> {
    let g = &cog.gangplank;
    let ends = (terrace_m + g.from[1], terrace_m + g.to[1]);
    let cell_m = template.grid.cell_m;
    let mut plank = ramp(
        meshes, repeat_m, patch, chart, cell_m, g, ends, radius_m, "boards",
    )?;
    if let Some(&Surface::Ramp {
        foot,
        dir,
        len,
        half_width,
        to,
        ..
    }) = plank.surfaces.first()
    {
        let side = dir.perp() * half_width;
        let (a, b) = (foot + dir * (len - 0.05), foot + dir * (len + LANDING_M));
        plank.surfaces.push(Surface::Floor {
            outline: ccw(vec![a - side, b - side, b + side, a + side]),
            top: to,
            bottom: to - 0.1,
        });
        plank.reach_m += LANDING_M;
    }
    Some(plank)
}

/// Where the cog stands, as the mockup's metres a ship piece covers: its
/// plan sampled every half metre, for the harbour's cells over the water.
pub fn plan_points(cog: &Cog) -> Vec<(f32, f32)> {
    let (ca, sa) = (cog.heading.cos(), cog.heading.sin());
    let mut out = Vec::new();
    let (hl, hb) = (HULL.l / 2.0, HULL.b / 2.0);
    let (nl, nb) = ((HULL.l / 0.5) as i32, (HULL.b / 0.5) as i32);
    for i in 0..=nl {
        for j in 0..=nb {
            let (lx, lz) = (-hl + i as f32 * 0.5, -hb + j as f32 * 0.5);
            out.push((cog.x + lx * ca - lz * sa, cog.z + lx * sa + lz * ca));
        }
    }
    out
}
