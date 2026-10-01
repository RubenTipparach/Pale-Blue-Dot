//! A town that stands on the sea (`cities-in-the-world` slice 4d): the
//! harbour. Its template's 0 m is the sea's surface. No cell its ground puts
//! under the sea is laid, so the water there is the planet's own, and what
//! stands over it (piers, fish huts on stilts) is cut on cells charted but
//! not laid. It is turned and shifted to lie with its sea over the planet's.

use super::chart::{Chart, Patch, chart_reach};
use super::{Template, offset};
use glam::Vec3;
use std::collections::{BTreeMap, BTreeSet};

/// How far from the site's cell a harbour's anchor may be shifted, in rings
/// of cells.
pub const SHIFT_RINGS: i32 = 24;

/// The mockup's offset cell holding `(x, z)`, its metres: its `cellAt`.
pub fn cell_at(x: f32, z: f32, cell_m: f32) -> (i32, i32) {
    let r_m = cell_m / 3f32.sqrt();
    let qf = (3f32.sqrt() / 3.0 * x - z / 3.0) / r_m;
    let rf = (2.0 / 3.0 * z) / r_m;
    let sf = -qf - rf;
    let (mut q, mut r, s) = (qf.round(), rf.round(), sf.round());
    let (dq, dr, ds) = ((q - qf).abs(), (r - rf).abs(), (s - sf).abs());
    if dq > dr && dq > ds {
        q = -r - s;
    } else if dr > ds {
        r = -q - s;
    }
    offset(q as i32, r as i32)
}

/// The mockup's centre of offset cell `(c, r)`, its `cx` and `cz`.
pub fn centre(c: i32, r: i32, cell_m: f32) -> (f32, f32) {
    let row = 1.5 * cell_m / 3f32.sqrt();
    (
        cell_m * (c as f32 + 0.5 * r.rem_euclid(2) as f32),
        row * r as f32,
    )
}

/// Where the mockup's point `(x, z)` stands on the planet, a direction: in
/// its cell, as far toward the real centres across the cell's edges 0 and 1
/// as the mockup puts it toward its own, so a point follows the sphere's
/// cells as a building's corners do. `None` where the cell is not charted.
pub fn point(chart: &Chart, patch: &Patch, x: f32, z: f32, cell_m: f32) -> Option<Vec3> {
    let (c, r) = cell_at(x, z, cell_m);
    let here = chart.cell(c, r)?;
    let (x0, z0) = centre(c, r, cell_m);
    let (dx, dz) = (x - x0, z - z0);
    // Edge 1's neighbour is half a cell east and a row south.
    let b = dz / (cell_m * 3f32.sqrt() / 2.0);
    let a = (dx - b * cell_m / 2.0) / cell_m;
    let toward = |d: usize| -> Option<Vec3> {
        let side = chart.side(c, r, d)?;
        let n = *patch.cells[here].neighbors.get(side)?;
        patch.cells.get(n).map(|n| n.direction)
    };
    let o = patch.cells[here].direction;
    let (e0, e1) = (toward(0)? - o, toward(1)? - o);
    Some((o + e0 * a + e1 * b).normalize())
}

/// Where a harbour's boat lies (task 4.2b): its direction on the planet,
/// and the tangent its bow points along there, placed as [`point`] places
/// any of the mockup's points. `None` where its cell is not charted.
pub fn boat_pose(
    chart: &Chart,
    patch: &Patch,
    boat: &super::Boat,
    cell_m: f32,
) -> Option<(Vec3, Vec3)> {
    let at = point(chart, patch, boat.x, boat.z, cell_m)?;
    let (x, z) = (boat.x + boat.heading.cos(), boat.z + boat.heading.sin());
    let ahead = point(chart, patch, x, z, cell_m)?;
    let bow = ahead - at;
    Some((at, (bow - at * bow.dot(at)).normalize_or_zero()))
}

/// The cells under what a sea template stands over the water: its
/// buildings' and decks' cells, every cell a pier, the slip or the
/// gangplank crosses, and its lanterns', boats', light's, dressing's and
/// cog's. Some are dry, and those
/// are laid as well.
pub fn over_water(template: &Template) -> BTreeSet<(i32, i32)> {
    let cell_m = template.grid.cell_m;
    let mut out = BTreeSet::new();
    for b in &template.buildings {
        out.extend(b.cells.iter().map(|&[c, r]| (c, r)));
        if let Some(s) = &b.stilts {
            out.extend(s.deck.iter().map(|&[c, r]| (c, r)));
        }
    }
    let slip = template.shipyard.as_ref().map(|y| &y.slip);
    let gangplank = template.cog.as_ref().map(|c| &c.gangplank);
    for p in template.piers.iter().chain(slip).chain(gangplank) {
        let (dx, dz) = (p.to[0] - p.from[0], p.to[2] - p.from[2]);
        let len = (dx * dx + dz * dz).sqrt();
        let (ux, uz) = if len > 0.0 {
            (dx / len, dz / len)
        } else {
            (0.0, 0.0)
        };
        let steps = (len / 0.25).ceil() as i32;
        for k in 0..=steps {
            let t = k as f32 / steps.max(1) as f32;
            for side in [-0.5f32, 0.0, 0.5] {
                let w = side * (p.width_m + 0.4);
                let x = p.from[0] + dx * t - uz * w;
                let z = p.from[2] + dz * t + ux * w;
                out.insert(cell_at(x, z, cell_m));
            }
        }
    }
    for l in &template.lanterns {
        out.insert(cell_at(l[0], l[2], cell_m));
    }
    // A boat's cell, and the cell a metre ahead of it, where its bow is read.
    for b in &template.boats {
        out.insert(cell_at(b.x, b.z, cell_m));
        out.insert(cell_at(
            b.x + b.heading.cos(),
            b.z + b.heading.sin(),
            cell_m,
        ));
    }
    if let Some(l) = &template.light {
        out.insert(cell_at(l.x, l.z, cell_m));
    }
    // A dressing thing's cell and the cells a metre east and south of it,
    // where its frame is read (task 4.2c).
    let things = template.dressing.iter().map(super::Dress::at);
    let yard = template
        .shipyard
        .iter()
        .flat_map(|y| [(y.x, y.z), (y.planks[0], y.planks[2])]);
    let cog = template
        .cog
        .iter()
        .flat_map(super::pieces::cog::plan_points);
    for (x, z) in things.chain(yard).chain(cog) {
        for (dx, dz) in [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0)] {
            out.insert(cell_at(x + dx, z + dz, cell_m));
        }
    }
    let (nc, nr) = (template.grid.columns, template.grid.rows);
    out.retain(|&(c, r)| c >= 0 && r >= 0 && c < nc && r < nr);
    out
}

/// Where a sea template lies best on a patch: its anchor on a cell within
/// [`SHIFT_RINGS`] of `site`, turned to one of six sides, so the most of its
/// cells agree with the planet about being sea. `natural` is the ground's
/// height at a direction and `sea_m` the sea's, metres over the radius.
/// `side` is the site's own turn (the side of `site` that is the layout's
/// east); ties go to it, then to the anchor nearest the site. Returns the
/// anchor's patch cell and its side for the layout's east, and the share of
/// cells that agree.
///
/// Each turn is charted once, over the template grown by the shift on every
/// side. A shift is a translation on the hex grid, so the template shifted
/// by `delta` lies where the grown chart has the cells `delta` from its own.
pub fn placement(
    template: &Template,
    patch: &Patch,
    site: usize,
    side: usize,
    anchor: (i32, i32),
    natural: impl Fn(Vec3) -> f32,
    sea_m: f32,
) -> Result<(usize, usize, f32), String> {
    let pad = SHIFT_RINGS;
    let (nc, nr) = (template.grid.columns, template.grid.rows);
    let (wide, high) = (nc + 2 * pad, nr + 2 * pad);
    let grown: BTreeSet<(i32, i32)> = (-pad..nr + pad)
        .flat_map(|r| (-pad..nc + pad).map(move |c| (c, r)))
        .collect();
    let index = |(c, r): (i32, i32)| -> Option<usize> {
        ((-pad..nc + pad).contains(&c) && (-pad..nr + pad).contains(&r))
            .then(|| ((c + pad) + (r + pad) * wide) as usize)
    };
    let cells: Vec<((i32, i32), bool)> = template
        .ground
        .iter()
        .map(|g| (super::axial(g.c, g.r), g.h < 0))
        .collect();
    let mut shifts = Vec::new();
    for dq in -pad..=pad {
        for dr in -pad..=pad {
            if (dq + dr).abs() <= pad {
                shifts.push((dq, dr));
            }
        }
    }
    // Nearest the site first, so a tie goes to the smaller shift.
    shifts.sort_by_key(|&(dq, dr)| (dq.abs().max(dr.abs()).max((dq + dr).abs()), dq, dr));
    let mut sea_at: Vec<Option<bool>> = vec![None; patch.cells.len()];
    let mut best: Option<(usize, usize, usize, usize)> = None;
    let sides = patch.cells.get(site).map_or(0, |c| c.corners.len());
    if sides != 6 {
        return Err("the harbour's site is on a pentagon".into());
    }
    for turn in 0..6 {
        let d0 = (side + turn) % 6;
        let chart = chart_reach(patch, anchor, site, d0, &grown);
        let mut dense: Vec<Option<(usize, usize)>> = vec![None; (wide * high) as usize];
        for (&at, c) in &chart.cells {
            if let Some(i) = index(at) {
                dense[i] = Some((c.cell, c.d0));
            }
        }
        for &(dq, dr) in &shifts {
            let (aq, ar) = super::axial(anchor.0, anchor.1);
            let Some(Some((at, d0_at))) = index(offset(aq + dq, ar + dr)).map(|i| dense[i]) else {
                continue;
            };
            // A placement counts only where every cell of the template can
            // be laid: on the patch, off any pentagon.
            let mut agree = 0;
            let mut whole = true;
            for &((q, r), wet) in &cells {
                let Some(Some((cell, _))) = index(offset(q + dq, r + dr)).map(|i| dense[i]) else {
                    whole = false;
                    break;
                };
                let is_sea = *sea_at[cell]
                    .get_or_insert_with(|| natural(patch.cells[cell].direction) < sea_m);
                if is_sea == wet {
                    agree += 1;
                }
            }
            if whole && best.is_none_or(|(a, ..)| agree > a) {
                best = Some((agree, at, d0_at, turn));
            }
        }
    }
    let (agree, at, d0, _) = best.ok_or("no placement for the harbour")?;
    Ok((at, d0, agree as f32 / cells.len().max(1) as f32))
}

/// The dry cells of a sea template's ground, and how high each stands:
/// the template's own layers over its 0 m.
pub fn dry(template: &Template) -> BTreeMap<(i32, i32), i32> {
    template
        .ground
        .iter()
        .filter(|g| g.h >= 0)
        .map(|g| ((g.c, g.r), g.h))
        .collect()
}
