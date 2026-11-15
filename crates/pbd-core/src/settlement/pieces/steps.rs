//! Stairs where a town's path takes a step (`path-stairs`): the owner,
//! 2026-11-14, "where theres a step on a path there should be staris".
//!
//! A **path cell** is a footprint cell the walker is meant to walk along: a
//! paved top (`cobble`, `flag`, `dirt`) on no building's cell and no field,
//! or a gate's passage. A **riser** is an edge between two path cells where
//! one stands higher than the other by more than the walker holds,
//! [`HOLD_M`], and no more than [`MAX_RISE_M`]. Each riser gets a stair in
//! its lower cell, in one of the mockup's two forms:
//!
//! - **street steps** (the mockup's `streetSteps`) across the whole cell,
//!   where the mockup cut them (an area ending ", steps") or where a cell
//!   climbs to two neighbouring edges;
//! - **a flight against the riser** anywhere else, its head on the edge.
//!
//! The stairs are pieces cut from the stored town and its template each
//! time the town is cut, as the masonry is: never saved, and they change no
//! level and no ground. A riser whose cells hold a player's edit is skipped.

use super::super::chart::{Chart, Patch};
use super::super::record::Town;
use super::super::{Template, neighbour};
use super::dressing::finish;
use super::harbour::corners;
use super::{BuildingSolids, Frame, Meshes, Sink, Surface, ccw, clip_half};
use glam::{Vec2, Vec3};
use std::collections::BTreeMap;

/// How far under a grounded walker a floor still holds it: the app's
/// `walking::HOLD_M`. A rise up to this is walked without a stair.
pub const HOLD_M: f32 = 0.35;
/// The tallest riser a path stair climbs: one layer and a pier's planks.
/// Nothing between two path cells in any template is taller.
pub const MAX_RISE_M: f32 = 1.25;
/// Street steps' risers to a layer (the mockup's `STREET`: 6 of 0.167 m).
pub const STREET_RISERS: u32 = 6;
/// A flight's riser and tread, as the porch stair's and the desert's.
pub const FLIGHT_RISE_M: f32 = 0.19;
pub const FLIGHT_RUN_M: f32 = 0.3;
/// A flight against a riser, wide: the mockup's `cliffStair`.
pub const FLIGHT_WIDTH_M: f32 = 1.3;

/// What a path cell is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathCell {
    /// A lane, street, square, quay or plaza.
    Street,
    /// A cell the mockup cut street steps in: its area ends ", steps".
    Steps,
    /// A gate's passage, under its vault.
    Gate,
}

/// A template's path cells, by layout cell.
pub fn path_cells(template: &Template) -> BTreeMap<(i32, i32), PathCell> {
    let buildings: std::collections::BTreeSet<(i32, i32)> = template
        .buildings
        .iter()
        .flat_map(|b| b.cells.iter().map(|&[c, r]| (c, r)))
        .collect();
    let height: BTreeMap<(i32, i32), i32> =
        template.ground.iter().map(|g| ((g.c, g.r), g.h)).collect();
    let masonry: BTreeMap<(i32, i32), bool> = template
        .masonry
        .iter()
        .map(|m| {
            let h = height.get(&(m.c, m.r)).copied().unwrap_or(m.from);
            ((m.c, m.r), m.from > h)
        })
        .collect();
    let mut out = BTreeMap::new();
    for g in &template.ground {
        let at = (g.c, g.r);
        if buildings.contains(&at) {
            continue;
        }
        let kind = match masonry.get(&at) {
            Some(true) => PathCell::Gate,
            Some(false) => continue,
            None => {
                let paved = matches!(g.top.as_str(), "cobble" | "flag" | "dirt");
                if !paved || g.area.to_lowercase().contains("field") {
                    continue;
                }
                if g.area.ends_with(", steps") {
                    PathCell::Steps
                } else {
                    PathCell::Street
                }
            }
        };
        out.insert(at, kind);
    }
    out
}

/// A lower path cell and its risers: each edge `d` up to a path cell
/// `rise` metres higher.
#[derive(Clone, Debug, PartialEq)]
pub struct Riser {
    pub cell: (i32, i32),
    pub kind: PathCell,
    /// Its ground, metres over the radius.
    pub base_m: f32,
    pub ups: Vec<(usize, f32)>,
}

impl Riser {
    /// Whether it takes street steps across the cell: where the mockup cut
    /// them, or where it climbs to two neighbouring edges (flights from
    /// both would cross). Never in a gate's passage.
    pub fn street_steps(&self) -> bool {
        if self.kind == PathCell::Gate {
            return false;
        }
        let adjacent = self.ups.iter().any(|&(a, _)| {
            self.ups
                .iter()
                .any(|&(b, _)| (a + 1) % 6 == b || (b + 1) % 6 == a)
        });
        self.kind == PathCell::Steps || adjacent
    }
}

/// Every riser between a stored town's path cells: its cells at their
/// stored levels, which cells are paths read from its template.
pub fn risers(town: &Town, template: &Template) -> Vec<Riser> {
    let paths = path_cells(template);
    let level: BTreeMap<(i32, i32), f32> = town
        .cells
        .iter()
        .enumerate()
        .map(|(i, c)| ((c.0, c.1), town.terrace_of(i)))
        .collect();
    let mut out = Vec::new();
    for (&at, &kind) in &paths {
        let Some(&base) = level.get(&at) else {
            continue;
        };
        let ups: Vec<(usize, f32)> = (0..6)
            .filter_map(|d| {
                let n = neighbour(at.0, at.1, d);
                paths.get(&n)?;
                let rise = level.get(&n)? - base;
                (rise > HOLD_M && rise <= MAX_RISE_M).then_some((d, rise))
            })
            .collect();
        if !ups.is_empty() {
            out.push(Riser {
                cell: at,
                kind,
                base_m: base,
                ups,
            });
        }
    }
    out
}

/// One stair cut, and what it climbs.
#[derive(Clone, Debug, PartialEq)]
pub struct Cut {
    pub cell: (i32, i32),
    /// The edges it climbs to.
    pub edges: Vec<usize>,
    pub street_steps: bool,
}

/// The stairs of a stored town (`path-stairs`), each a piece with its
/// meshes added to `meshes`: street steps or flights in each riser's lower
/// cell, of the template's masonry's dressed stone (`stone`; the tundra's
/// ice). A riser is skipped where `yielded` holds the exact key of its lower
/// cell or the cell it climbs to: the player has dug or built there. What
/// was cut, and how many risers were skipped.
#[allow(clippy::too_many_arguments)]
pub fn path_stairs(
    meshes: &mut Meshes,
    repeat_m: &dyn Fn(&str) -> f32,
    patch: &Patch,
    chart: &Chart,
    town: &Town,
    template: &Template,
    radius_m: f32,
    yielded: &dyn Fn(u32) -> bool,
) -> (Vec<BuildingSolids>, Vec<Cut>, usize) {
    let material = template
        .masonry_material
        .clone()
        .unwrap_or_default()
        .vault()
        .to_string();
    let key = |c: (i32, i32)| chart.cell(c.0, c.1).map(|i| patch.keys[i]);
    let mut pieces = Vec::new();
    let mut cuts = Vec::new();
    let mut skipped = 0;
    for riser in risers(town, template) {
        let (c, r) = riser.cell;
        let Some(cell) = chart.cell(c, r) else {
            continue;
        };
        let held = |cells: &[(i32, i32)]| cells.iter().any(|&x| key(x).is_none_or(yielded));
        let direction = patch.cells[cell].direction;
        let x = direction.any_orthonormal_vector();
        let frame = Frame {
            origin: direction * (radius_m + riser.base_m),
            x,
            y: direction,
            z: x.cross(direction),
        };
        let Some(hex) = corners(patch, chart, &frame, c, r) else {
            continue;
        };
        let up_cells: Vec<(i32, i32)> =
            riser.ups.iter().map(|&(d, _)| neighbour(c, r, d)).collect();
        let rise = riser.ups.iter().map(|u| u.1).fold(f32::MAX, f32::min);
        if riser.street_steps() {
            let mut all = up_cells.clone();
            all.push((c, r));
            if held(&all) {
                skipped += riser.ups.len();
                continue;
            }
            let edges: Vec<usize> = riser.ups.iter().map(|u| u.0).collect();
            match street_steps(meshes, repeat_m, frame, &hex, &edges, rise, &material) {
                Some(piece) => {
                    pieces.push(piece);
                    cuts.push(Cut {
                        cell: (c, r),
                        edges,
                        street_steps: true,
                    });
                }
                None => skipped += riser.ups.len(),
            }
            continue;
        }
        for (&(d, rise), &up) in riser.ups.iter().zip(&up_cells) {
            if held(&[(c, r), up]) {
                skipped += 1;
                continue;
            }
            let (a, b) = (hex[d], hex[(d + 1) % 6]);
            let centre = hex.iter().fold(Vec2::ZERO, |s, p| s + *p) / 6.0;
            let piece = flight_to_edge(meshes, repeat_m, frame, (a, b), centre, rise, &material);
            pieces.push(piece);
            cuts.push(Cut {
                cell: (c, r),
                edges: vec![d],
                street_steps: false,
            });
        }
    }
    (pieces, cuts, skipped)
}

/// Street steps across a cell (the mockup's `streetSteps`), in `frame` on
/// the cell's ground, its corners `hex`: [`STREET_RISERS`] bands of the
/// hexagon from its foot, climbing `rise` toward the mean of the outward
/// directions of `edges`. The pitch line reaches the full rise five sixths
/// of the way to the nearest up edge's middle, so each up edge's middle is
/// at the full rise; with one up edge, that is the mockup's 6 treads across
/// the hexagon. `None` where the up edges cancel out.
fn street_steps(
    meshes: &mut Meshes,
    repeat_m: &dyn Fn(&str) -> f32,
    frame: Frame,
    hex: &[Vec2; 6],
    edges: &[usize],
    rise: f32,
    material: &str,
) -> Option<BuildingSolids> {
    let centre = hex.iter().fold(Vec2::ZERO, |s, p| s + *p) / 6.0;
    let mid = |d: usize| (hex[d] + hex[(d + 1) % 6]) * 0.5;
    let dir = edges.iter().fold(Vec2::ZERO, |s, &d| {
        s + (mid(d) - centre).normalize_or_zero()
    });
    if dir.length() < 0.1 {
        return None;
    }
    let dir = dir.normalize();
    let along = |p: Vec2| (p - centre).dot(dir);
    let u0 = hex.iter().map(|&p| along(p)).fold(f32::MAX, f32::min);
    let u1 = hex.iter().map(|&p| along(p)).fold(f32::MIN, f32::max);
    let foot = centre + dir * u0;
    let to_edge = edges
        .iter()
        .map(|&d| along(mid(d)) - u0)
        .fold(f32::MAX, f32::min);
    let n = STREET_RISERS;
    let full = to_edge * (n - 1) as f32 / n as f32;
    let run = full / (n - 1) as f32;
    let mut sink = Sink::new(meshes, repeat_m, frame);
    for k in 0..n {
        let lo = if k == 0 { -0.01 } else { k as f32 * run };
        let hi = if k == n - 1 {
            u1 - u0 + 0.01
        } else {
            (k + 1) as f32 * run
        };
        let band = clip_half(&clip_half(hex, foot + dir * lo, dir), foot + dir * hi, -dir);
        if band.len() < 3 {
            continue;
        }
        let top = rise * (k + 1) as f32 / n as f32;
        sink.prism(material, material, &ccw(band), 0.0, top, None);
    }
    sink.surfaces.push(Surface::Steps {
        outline: ccw(hex.to_vec()),
        foot,
        dir,
        full,
        base: 0.0,
        rise,
    });
    let reach = hex.iter().map(|p| p.length()).fold(0.0f32, f32::max) + 1.0;
    Some(finish(sink, frame, reach))
}

/// A straight flight in a cell against its edge `(a, b)` (`centre` the
/// cell's middle), in `frame` on the cell's ground: [`FLIGHT_WIDTH_M`] wide
/// on the edge's middle, its head on the edge, climbing square to it by
/// `rise` in risers of about [`FLIGHT_RISE_M`] on [`FLIGHT_RUN_M`] treads.
fn flight_to_edge(
    meshes: &mut Meshes,
    repeat_m: &dyn Fn(&str) -> f32,
    frame: Frame,
    (a, b): (Vec2, Vec2),
    centre: Vec2,
    rise: f32,
    material: &str,
) -> BuildingSolids {
    let mid = (a + b) * 0.5;
    let e = b - a;
    let mut dir = e.perp().normalize();
    if dir.dot(mid - centre) < 0.0 {
        dir = -dir;
    }
    let width = FLIGHT_WIDTH_M.min(e.length() - 0.2);
    let mut sink = Sink::new(meshes, repeat_m, frame);
    let foot = steps_along(
        &mut sink,
        mid,
        dir,
        rise,
        FLIGHT_RUN_M,
        width,
        (material, material),
    );
    let reach = foot.length().max(mid.length()) + width + 1.0;
    finish(sink, frame, reach)
}

/// Solid steps of a straight flight in `sink`'s frame, on its ground at 0:
/// its head at `head`, climbing along `dir` by `rise` in risers of about
/// [`FLIGHT_RISE_M`] on treads of `run`, `width` wide, their tops and sides
/// of `(top, side)`; and the walker's flight over them. Where it starts, its
/// foot.
pub(super) fn steps_along(
    sink: &mut Sink,
    head: Vec2,
    dir: Vec2,
    rise: f32,
    run: f32,
    width: f32,
    (top, side): (&str, &str),
) -> Vec2 {
    let risers = ((rise / FLIGHT_RISE_M).round() as u32).max(2);
    let each = rise / risers as f32;
    let len = risers as f32 * run;
    let foot = head - dir * len;
    let ang = dir.y.atan2(dir.x);
    let (top, side) = (top.to_string(), side.to_string());
    for k in 0..risers {
        let h = (k + 1) as f32 * each;
        let p = foot + dir * ((k as f32 + 0.5) * run);
        let pick = |n: Vec3| {
            if n.y > 0.5 { top.clone() } else { side.clone() }
        };
        sink.boxed(&pick, p.x, 0.0, p.y, run + 0.01, h, width, ang, None);
    }
    sink.surfaces.push(Surface::Flight {
        foot,
        dir,
        len,
        half_width: width / 2.0,
        base: 0.0,
        rise: each,
        risers,
    });
    foot
}
