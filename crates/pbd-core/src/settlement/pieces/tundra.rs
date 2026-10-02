//! The tundra's pieces (`cities-in-the-world` slice 4g): the igloo, the
//! first building not cut to its cell, and the frozen lake's ice.

use super::super::chart::{Chart, Patch};
use super::super::{BuildingDef, Frozen, Kit};
use super::dressing::{Place, finish};
use super::{
    BuildingSolids, Indoors, LIFT_M, LightKind, Meshes, Plan, Sink, Solid, Surface, ccw, edge_ends,
};
use glam::{Vec2, Vec3};
use std::f32::consts::{FRAC_PI_2, PI, TAU};

/// The igloo's dome: its radius at the ground and its height, the mockup's
/// `A` and `H`.
const DOME_R: f32 = 2.3;
const DOME_H: f32 = 2.5;
/// Its tunnel, the mockup's `T`: the shell's outer and inner half-widths and
/// tops, and where its arch springs from its short walls.
const TUNNEL_WO: f32 = 1.0;
const TUNNEL_TO: f32 = 2.25;
const TUNNEL_WI: f32 = 0.72;
const TUNNEL_TI: f32 = 1.95;
const TUNNEL_SPRING: f32 = 1.0;
/// How far the tunnel's mouth is from the dome's middle.
const TUNNEL_MOUTH: f32 = DOME_R + 1.6;

/// The tunnel's cross-section, the mockup's `profile`: from one wall's
/// foot up to the arch's spring, round the arch, and down the other wall,
/// `(across, up)`.
fn profile(w: f32, top: f32) -> Vec<Vec2> {
    let mut pts = vec![Vec2::new(w, 0.0), Vec2::new(w, TUNNEL_SPRING)];
    for k in 1..12 {
        let th = k as f32 / 12.0 * PI;
        pts.push(Vec2::new(
            w * th.cos(),
            TUNNEL_SPRING + (top - TUNNEL_SPRING) * th.sin(),
        ));
    }
    pts.push(Vec2::new(-w, TUNNEL_SPRING));
    pts.push(Vec2::new(-w, 0.0));
    pts
}

/// How far along the door's way the dome's surface is at `(across, up)`.
fn on_dome(q: Vec2) -> f32 {
    (DOME_R * DOME_R * (1.0 - (q.y / DOME_H).powi(2)) - q.x * q.x)
        .max(0.0)
        .sqrt()
}

/// One cut of a convex polygon by the line through `a` and `b`, the
/// mockup's `half`: the part on its left where `left`, else on its right.
fn half(poly: &[Vec2], a: Vec2, b: Vec2, left: bool) -> Vec<Vec2> {
    let sign = if left { 1.0 } else { -1.0 };
    let side = |p: Vec2| (b - a).perp_dot(p - a) * sign;
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

/// The angle from `b` to `a`, folded into (-PI, PI].
fn ang_diff(a: f32, b: f32) -> f32 {
    let mut d = a - b;
    while d > PI {
        d -= TAU;
    }
    while d < -PI {
        d += TAU;
    }
    d
}

/// An igloo (slice 4g), the mockup's `igloo`: a dome of snow blocks 4.6 m
/// across and 2.5 m high on its cell, spilling over its ring, and a vaulted
/// tunnel out along its door's edge, the dome cut to the tunnel's outer
/// profile and the tunnel's shell reaching 2 cm into it, so no seam opens
/// where they meet (12b). Inside, a snow bench at the back with its furs,
/// and a lamp. The walker is held by a ring of wall inside the dome's low
/// edge, open at the tunnel, and by the tunnel's walls and roof; a second
/// ring outside the dome's foot keeps a walker outside from walking into
/// its shell, which the mockup leaves open.
pub(super) fn igloo(
    sink: &mut Sink,
    plan: &Plan,
    def: &BuildingDef,
    kit: &Kit,
) -> Result<(), String> {
    let centre = plan.centres[0];
    // Its inside is the dome's own air, set before anything is cut so that
    // what faces into it is cut as its room's and its candle lights it.
    sink.indoors = Some(Indoors::dome(centre, DOME_R, DOME_H));
    // A floor of the kit's snow under the dome, a centimetre over the
    // ground as a building's is: the planet's snow is no room's.
    let floor: Vec<Vec2> = (0..24)
        .map(|k| centre + Vec2::from_angle(k as f32 / 24.0 * TAU) * (DOME_R - 0.05))
        .collect();
    sink.prism(&kit.floor, &kit.floor, &floor, -0.05, LIFT_M, None);
    let [_, _, door, _] = *def
        .doors
        .first()
        .ok_or_else(|| format!("{}: an igloo needs its door", def.name))?;
    let (a, b) = edge_ends(plan, 0, door as usize % 6);
    let dir = ((a + b) * 0.5 - centre).normalize_or_zero();
    let across = Vec2::new(-dir.y, dir.x);
    let da = dir.y.atan2(dir.x);
    // A point `u` along the tunnel and `q` (across, up) in its section.
    let at = |u: f32, q: Vec2| {
        let p = centre + dir * u + across * q.x;
        Vec3::new(p.x, q.y, p.y)
    };
    let start = |q: Vec2| on_dome(q) - 0.02;
    let (outer, inner) = (profile(TUNNEL_WO, TUNNEL_TO), profile(TUNNEL_WI, TUNNEL_TI));
    // Each face's texture: along the tunnel, and round its section by the
    // arc to that point.
    let arcs = |prof: &[Vec2]| {
        let mut s = vec![0.0f32];
        for k in 1..prof.len() {
            s.push(s[k - 1] + prof[k].distance(prof[k - 1]));
        }
        s
    };
    let (arc_out, arc_in) = (arcs(&outer), arcs(&inner));
    let block = "snowblock";
    for k in 0..outer.len() - 1 {
        let (p, q) = (outer[k], outer[k + 1]);
        let (pi, qi) = (inner[k], inner[k + 1]);
        let vm = (p.x + q.x) / 2.0;
        let ym = (p.y + q.y) / 2.0 - TUNNEL_SPRING / 2.0;
        let out = Vec3::new(across.x * vm, ym, across.y * vm);
        // Each strip's texture runs along the tunnel, and round its section
        // by the arc to each of its two profile points.
        let shell = |sink: &mut Sink, p: Vec2, q: Vec2, arc: (f32, f32), n: Vec3| {
            let pts = [
                at(start(p), p),
                at(TUNNEL_MOUTH, p),
                at(TUNNEL_MOUTH, q),
                at(start(q), q),
            ];
            let uvf = move |pt: Vec3| {
                let along = (Vec2::new(pt.x, pt.z) - centre).dot(dir);
                let near_p = pts[0].distance(pt).min(pts[1].distance(pt));
                let near_q = pts[2].distance(pt).min(pts[3].distance(pt));
                let v = if near_p <= near_q { arc.0 } else { arc.1 };
                Vec2::new(along / 2.0, v / 1.6)
            };
            sink.face(block, &pts, n, Some(&uvf));
        };
        shell(sink, p, q, (arc_out[k], arc_out[k + 1]), out);
        shell(sink, pi, qi, (arc_in[k], arc_in[k + 1]), -out);
        let ring = [
            at(start(pi), pi),
            at(start(qi), qi),
            at(start(q), q),
            at(start(p), p),
        ];
        let back = Vec3::new(dir.x, 0.0, dir.y);
        let end_uv = |pt: Vec3| {
            let v = (Vec2::new(pt.x, pt.z) - centre).dot(across);
            Vec2::new(v / 2.0, pt.y / 1.6)
        };
        sink.face(block, &ring, -back, Some(&end_uv));
        sink.face(block, &ring, back, Some(&end_uv));
        let mouth = [
            at(TUNNEL_MOUTH, pi),
            at(TUNNEL_MOUTH, qi),
            at(TUNNEL_MOUTH, q),
            at(TUNNEL_MOUTH, p),
        ];
        sink.face(block, &mouth, back, Some(&end_uv));
    }
    // The dome, less the tunnel's outer profile: each face taken into the
    // tunnel's section, the profile cut out of it an edge at a time, and
    // what is left put back on the dome.
    let mut hole = outer.clone();
    let last = hole.len() - 1;
    hole[0].y = -0.5;
    hole[last].y = -0.5;
    let area: f32 = (0..hole.len())
        .map(|k| hole[k].perp_dot(hole[(k + 1) % hole.len()]))
        .sum();
    if area < 0.0 {
        hole.reverse();
    }
    let clip = |pts: &[Vec3]| -> Option<Vec<Vec<Vec3>>> {
        let q: Vec<(Vec2, f32)> = pts
            .iter()
            .map(|p| {
                let d = Vec2::new(p.x, p.z) - centre;
                (Vec2::new(d.dot(across), p.y), d.dot(dir))
            })
            .collect();
        if q.iter().any(|(_, along)| *along < 0.3)
            || q.iter().all(|(s, _)| s.x > TUNNEL_WO)
            || q.iter().all(|(s, _)| s.x < -TUNNEL_WO)
            || q.iter().all(|(s, _)| s.y > TUNNEL_TO)
        {
            return None;
        }
        let mut rest: Vec<Vec2> = q.iter().map(|(s, _)| *s).collect();
        let mut pieces = Vec::new();
        for i in 0..hole.len() {
            if rest.len() < 3 {
                break;
            }
            let (a, b) = (hole[i], hole[(i + 1) % hole.len()]);
            let piece = half(&rest, a, b, false);
            if piece.len() >= 3 {
                pieces.push(piece.iter().map(|s| at(on_dome(*s), *s)).collect());
            }
            rest = half(&rest, a, b, true);
        }
        Some(pieces)
    };
    const SEG: usize = 48;
    const RINGS: usize = 16;
    let wraps = (PI * DOME_R).round().max(1.0);
    let point = |i: usize, j: usize| {
        let a = i as f32 / SEG as f32 * TAU;
        let t = j as f32 / RINGS as f32 * FRAC_PI_2;
        Vec3::new(
            centre.x + DOME_R * t.cos() * a.cos(),
            DOME_H * t.sin(),
            centre.y + DOME_R * t.cos() * a.sin(),
        )
    };
    for j in 0..RINGS {
        for i in 0..SEG {
            let am = (i as f32 + 0.5) / SEG as f32 * TAU;
            let quad = [
                point(i, j),
                point(i + 1, j),
                point(i + 1, j + 1),
                point(i, j + 1),
            ];
            let pts: &[Vec3] = if j == RINGS - 1 {
                &quad[..3]
            } else {
                &quad[..]
            };
            let m = pts.iter().fold(Vec3::ZERO, |s, q| s + *q) / pts.len() as f32;
            let out = Vec3::new(m.x - centre.x, m.y + 0.001, m.z - centre.y);
            let uv = |q: Vec3| {
                let an = (q.z - centre.y).atan2(q.x - centre.x);
                Vec2::new((am + ang_diff(an, am)) / TAU * wraps, q.y / 1.6)
            };
            let faces = clip(pts).unwrap_or_else(|| vec![pts.to_vec()]);
            for f in faces {
                sink.face(block, &f, out, Some(&uv));
                sink.face(block, &f, -out, Some(&uv));
            }
        }
    }
    // Inside: the ring the walker slides on, open at the tunnel.
    let rect = |c: Vec2, len: f32, wide: f32, ang: f32| {
        let (u, v) = (
            Vec2::from_angle(ang) * (len / 2.0),
            Vec2::from_angle(ang + FRAC_PI_2) * (wide / 2.0),
        );
        ccw(vec![c - u - v, c + u - v, c + u + v, c - u + v])
    };
    for k in 0..16 {
        let a = k as f32 / 16.0 * TAU;
        if ang_diff(a, da).abs() < 0.45 {
            continue;
        }
        let c = centre + Vec2::from_angle(a) * 1.69;
        sink.solids.push(Solid {
            outline: rect(c, 0.72, 0.2, a + FRAC_PI_2),
            y0: 0.0,
            y1: TUNNEL_TI,
        });
    }
    // Outside: the same round the dome's foot, open at the tunnel.
    for k in 0..20 {
        let a = k as f32 / 20.0 * TAU;
        if ang_diff(a, da).abs() < 0.62 {
            continue;
        }
        let c = centre + Vec2::from_angle(a) * (DOME_R - 0.1);
        sink.solids.push(Solid {
            outline: rect(c, 0.8, 0.2, a + FRAC_PI_2),
            y0: 0.0,
            y1: 1.2,
        });
    }
    // The tunnel's walls and roof, as its inner profile.
    let ua = on_dome(Vec2::new(TUNNEL_WI, 0.0)) - 0.1;
    let um = (ua + TUNNEL_MOUTH) / 2.0;
    for s in [-1.0f32, 1.0] {
        let c = centre + dir * um + across * (s * (TUNNEL_WI + 0.1));
        sink.solids.push(Solid {
            outline: rect(c, TUNNEL_MOUTH - ua, 0.2, da),
            y0: 0.0,
            y1: 2.2,
        });
    }
    let ur = on_dome(Vec2::new(0.0, TUNNEL_TI)) - 0.05;
    let umr = (ur + TUNNEL_MOUTH) / 2.0;
    sink.solids.push(Solid {
        outline: rect(centre + dir * umr, TUNNEL_MOUTH - ur, 2.0 * TUNNEL_WI, da),
        y0: TUNNEL_TI,
        y1: TUNNEL_TI + 0.3,
    });
    // A sleeping bench of snow at the back, furs on it, and a lamp.
    let back = da + PI;
    let bench: Vec<Vec2> = (0..=6)
        .map(|k| centre + Vec2::from_angle(back - 1.1 + k as f32 / 6.0 * 2.2) * 1.55)
        .collect();
    let bench = ccw(bench);
    sink.prism(block, block, &bench, 0.0, 0.45, None);
    sink.solids.push(Solid {
        outline: bench,
        y0: 0.0,
        y1: 0.45,
    });
    let fur = centre + Vec2::from_angle(back) * 1.1;
    sink.plain_box(
        "wool",
        fur.x,
        0.45,
        fur.y,
        Vec3::new(1.6, 0.08, 0.7),
        back + FRAC_PI_2,
    );
    let lamp = centre + Vec2::from_angle(da + 1.9) * 1.1;
    sink.plain_box(
        "granite",
        lamp.x,
        0.0,
        lamp.y,
        Vec3::new(0.4, 0.3, 0.4),
        0.0,
    );
    sink.flame("flame", Vec3::new(lamp.x, 0.3, lamp.y), 0.2);
    sink.light(
        LightKind::Candle,
        Vec3::new(lamp.x, 0.45, lamp.y),
        0.6,
        0.45,
        DOME_H,
    );
    // Its plan for shadows and roof checks.
    sink.roof_plan = (0..12)
        .map(|k| centre + Vec2::from_angle(k as f32 / 12.0 * TAU) * DOME_R)
        .collect();
    Ok(())
}

/// The frozen lake's ice (slice 4g): a sheet over each of its cells at
/// `top_m` in the mockup's metres, walked on, cut flat in a frame at its
/// middle, `zero_m` being the game's height of the template's 0 m.
#[allow(clippy::too_many_arguments)]
pub fn ice_sheet(
    meshes: &mut Meshes,
    repeat_m: &dyn Fn(&str) -> f32,
    patch: &Patch,
    chart: &Chart,
    frozen: &Frozen,
    cell_m: f32,
    radius_m: f32,
    zero_m: f32,
) -> Option<BuildingSolids> {
    let n = frozen.cells.len().max(1) as f32;
    let (x, z) = frozen.cells.iter().fold((0.0, 0.0), |(x, z), &[c, r]| {
        let (cx, cz) = super::super::sea::centre(c, r, cell_m);
        (x + cx / n, z + cz / n)
    });
    let place = Place::new(
        chart,
        patch,
        cell_m,
        (x, z),
        radius_m,
        zero_m + frozen.top_m,
    )?;
    let mut sink = Sink::new(meshes, repeat_m, place.frame);
    let mut reach = 0.0f32;
    for &[c, r] in &frozen.cells {
        let Some(hex) = super::harbour::corners(patch, chart, &place.frame, c, r) else {
            continue;
        };
        let hex = ccw(hex.to_vec());
        reach = hex.iter().map(|p| p.length()).fold(reach, f32::max);
        sink.prism("ice", "ice", &hex, -0.15, 0.0, None);
        sink.surfaces.push(Surface::Floor {
            outline: hex,
            top: 0.0,
            bottom: -0.15,
        });
    }
    Some(finish(sink, place.frame, reach + 1.0))
}
