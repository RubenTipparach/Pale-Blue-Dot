//! A town's building as pieces (`player-building` decision 5), and the roof
//! tool's fit (decision 6).
//!
//! [`blueprint`] charts a template's building onto the real cells at a spot
//! and turns it into the pieces a player could have built it from: a floor
//! in every cell of every storey (but a stair's well), the outer walls with
//! their doors and windows, a stair per upper storey, a lantern on every
//! storey, and the roof. The same rules check it as check a player's piece,
//! which is what makes one rule set serve the towns and the player.

use super::{Cells, Opening, Piece, RoofShape, STOREY, Shape, Site};
use crate::settlement::chart::{Patch, chart};
use crate::settlement::{BuildingDef, Kit, RoofKind, neighbour};
use std::collections::{BTreeSet, VecDeque};

/// The most cells the roof tool fills before it calls a room open.
pub const ROOM_MAX: usize = 24;

/// Whether a template's building is one the pieces can make: no newel, no
/// stilts, no raised floor, no walked roof, no open side and no dome.
pub fn plain(def: &BuildingDef, kit: &Kit) -> bool {
    def.newel.is_none()
        && def.stilts.is_none()
        && !def.raised
        && !def.parapet
        && def.open.is_empty()
        && def.parapet_gaps.is_none()
        && kit.roof != RoofKind::Dome
        && def.roof != "dome"
        && !kit.hut
        || (kit.hut && def.cells.len() == 1 && def.storeys == 1 && def.newel.is_none())
}

/// The roof a kit fits over `cells`: a cone over one cell, the kit's flat
/// roof, or a gable.
pub fn fit_roof(kit: &Kit, cells: Vec<u32>, layer: u16) -> Shape {
    let shape = if cells.len() == 1 || kit.roof == RoofKind::Cone {
        RoofShape::Cone
    } else if matches!(kit.roof, RoofKind::Flat | RoofKind::Dome) {
        RoofShape::Flat
    } else {
        RoofShape::Gable
    };
    Shape::roof(cells, layer, shape)
}

/// The room the roof tool finds from `cell` on the storey whose floor is
/// `floor`: every cell reached without crossing a wall of that storey. `None`
/// when the walls do not close round it within [`ROOM_MAX`] cells, or it runs
/// off the cells known.
pub fn room_at(site: &Site, pieces: &[Piece], cell: u32, floor: u16) -> Option<Vec<u32>> {
    let walled = |a: u32, b: u32| {
        pieces.iter().any(|p| {
            matches!(p.shape, Shape::Wall { a: x, b: y, layer, .. }
                if layer == floor && x == a.min(b) && y == a.max(b))
        })
    };
    let mut seen = BTreeSet::from([cell]);
    let mut queue = VecDeque::from([cell]);
    while let Some(c) = queue.pop_front() {
        for n in site.cells.neighbours(c) {
            let n = n?;
            if walled(c, n) || seen.contains(&n) {
                continue;
            }
            if seen.len() >= ROOM_MAX {
                return None;
            }
            seen.insert(n);
            queue.push_back(n);
        }
    }
    Some(seen.into_iter().collect())
}

/// A template building charted with its first cell on patch cell `at`, its
/// direction 0 on that cell's side `turn` (a sixth of a turn each), its
/// ground floor on `base`: the pieces, in the order a player would build
/// them (each storey's floors and walls, then the stairs, the lanterns and
/// the roof). An error names what
/// could not be charted.
pub fn blueprint(
    patch: &Patch,
    def: &BuildingDef,
    kit: &Kit,
    at: usize,
    turn: usize,
    base: u16,
) -> Result<Vec<Piece>, String> {
    let wanted: BTreeSet<(i32, i32)> = def.cells.iter().map(|&[c, r]| (c, r)).collect();
    let first = (def.cells[0][0], def.cells[0][1]);
    let charted = chart(patch, first, at, turn % 6, &wanted)?;
    let key_of = |c: i32, r: i32| -> Option<u32> {
        charted.cell(c, r).and_then(|i| patch.keys.get(i).copied())
    };
    // The key across offset edge `d` of `(c, r)`, in or out of the building.
    let across = |c: i32, r: i32, d: usize| -> Option<u32> {
        let here = charted.cell(c, r)?;
        let side = charted.side(c, r, d)?;
        let n = *patch.cells[here].neighbors.get(side)?;
        patch.keys.get(n).copied()
    };
    let cells_of = |list: &[[i32; 2]]| -> Result<Vec<u32>, String> {
        list.iter()
            .map(|&[c, r]| key_of(c, r).ok_or_else(|| format!("({c}, {r}) is not charted")))
            .collect()
    };
    let keys = cells_of(&def.cells)?;
    let storeys = def.storeys.max(1) as u16;
    let k = kit.name.as_str();
    let cells = Cells::new(patch);
    let mut out = Vec::new();

    // The stairs first, as a plan: which cells each upper storey's well
    // takes.
    let mut stairs: Vec<Shape> = Vec::new();
    if storeys > 1 {
        let (foot, head) = stair_cells(&cells, def, &keys, &key_of)
            .ok_or_else(|| format!("{}: no straight flight fits", def.name))?;
        // Each storey's flight turns back over the last one's: from the
        // landing the last one came up onto, through the same head, so it
        // starts on a floor and its well is clear of the one below.
        let on = cells
            .straight_on(foot, head)
            .ok_or_else(|| format!("{}: the flight has no landing", def.name))?;
        for s in 0..storeys - 1 {
            let foot = if s % 2 == 0 { foot } else { on };
            stairs.push(Shape::Stair {
                foot,
                head,
                layer: base + s * STOREY,
            });
        }
    }
    let in_well = |cell: u32, layer: u16| {
        stairs.iter().any(|st| {
            matches!(*st, Shape::Stair { foot, head, layer: l }
                if (cell == foot || cell == head) && layer == l + STOREY)
        })
    };
    let openings = |list: &[[i32; 4]], c: i32, r: i32, d: usize, s: u16| {
        list.iter()
            .any(|o| o[0] == c && o[1] == r && o[2] as usize == d && o[3] as u16 == s)
    };
    // A storey at a time, floors then walls, as a player builds up.
    for s in 0..storeys {
        let layer = base + s * STOREY;
        for &cell in &keys {
            if !in_well(cell, layer) {
                out.push(Piece::new(k, Shape::Floor { cell, layer }));
            }
        }
        for &[c, r] in &def.cells {
            let here = key_of(c, r).expect("charted above");
            for d in 0..6 {
                let (nc, nr) = neighbour(c, r, d);
                if wanted.contains(&(nc, nr)) {
                    continue;
                }
                let other = across(c, r, d)
                    .ok_or_else(|| format!("({c}, {r}) edge {d} is off the patch"))?;
                let opening =
                    if openings(&def.doors, c, r, d, s) || openings(&def.archways, c, r, d, s) {
                        Opening::Door
                    } else if openings(&def.windows, c, r, d, s) {
                        Opening::Window
                    } else {
                        Opening::None
                    };
                out.push(Piece::new(k, Shape::wall(here, other, layer, opening)));
            }
        }
    }
    out.extend(stairs.iter().cloned().map(|s| Piece::new(k, s)));
    // A lantern on every storey, in the first cell clear of the stairs.
    for s in 0..storeys {
        let layer = base + s * STOREY;
        let clear = keys.iter().copied().find(|&c| {
            !out.iter().any(|p| {
                matches!(p.shape, Shape::Stair { foot, head, layer: l }
                    if (foot == c || head == c) && l <= layer && layer < l + STOREY + 1)
            }) && !in_well(c, layer)
        });
        if let Some(cell) = clear {
            out.push(Piece::new(k, Shape::Lantern { cell, layer }));
        }
    }
    let top = base + storeys * STOREY;
    let roof = match def.roof.as_str() {
        "cone" => Shape::roof(keys.clone(), top, RoofShape::Cone),
        "flat" => Shape::roof(keys.clone(), top, RoofShape::Flat),
        _ => fit_roof(kit, keys.clone(), top),
    };
    out.push(Piece::new(k, roof));
    Ok(out)
}

/// A straight flight inside the building: a foot among its stair cells, a
/// head beside it in the building, and the cell straight on from the head in
/// the building too, so the flight lands on a floor. The template's own
/// pair is tried first, either way round.
fn stair_cells(
    cells: &Cells,
    def: &BuildingDef,
    keys: &[u32],
    key_of: &dyn Fn(i32, i32) -> Option<u32>,
) -> Option<(u32, u32)> {
    let named: Vec<u32> = def
        .stair_cells
        .iter()
        .filter_map(|&[c, r]| key_of(c, r))
        .collect();
    let fits = |foot: u32, head: u32| {
        keys.contains(&head)
            && cells.adjacent(foot, head)
            && cells
                .straight_on(foot, head)
                .is_some_and(|on| keys.contains(&on) && on != foot)
    };
    let mut tries: Vec<(u32, u32)> = Vec::new();
    if let [a, b] = named[..] {
        tries.push((a, b));
        tries.push((b, a));
    }
    for &foot in named.iter().chain(keys) {
        for &head in keys {
            tries.push((foot, head));
        }
    }
    tries.into_iter().find(|&(f, h)| f != h && fits(f, h))
}

/// The layer a blueprint's ground floor would stand on at `cells`: the
/// highest ground among them, so no floor is under the ground.
pub fn base_over(site: &Site, cells: &[u32]) -> Option<u16> {
    cells
        .iter()
        .map(|&c| (site.ground)(c))
        .try_fold(0u16, |m, g| Some(m.max(g?)))
}
