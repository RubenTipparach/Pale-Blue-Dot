//! A town building's furniture (`furniture` design 8): laid by one rule from
//! its stored record (cells, doors, stair, chimney, storeys, name), derived
//! each time the building is cut and never saved. Every piece is tried by
//! the same footprint rules a player's furniture is (`crate::furniture`) and
//! left out where it does not fit; nothing else in the cut moves.

use super::Plan;
use crate::furniture::{self, Kind, MIDDLE};
use crate::settlement::BuildingDef;

/// What kind of building it is, for its furniture.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Use {
    House,
    Inn,
    Hall,
    Store,
}

/// A building's use by its name, if it is furnished at all.
pub fn use_of(name: &str) -> Option<Use> {
    let n = name.to_lowercase();
    let has = |w: &[&str]| w.iter().any(|x| n.contains(x));
    if has(&["inn"]) {
        Some(Use::Inn)
    } else if has(&["hall", "exchange", "longhouse", "keep"]) {
        Some(Use::Hall)
    } else if has(&["boathouse", "fish hut", "tower", "lookout"]) {
        Some(Use::Store)
    } else if has(&["house", "cottage", "hut", "igloo"]) {
        Some(Use::House)
    } else {
        None
    }
}

/// One piece laid: its kind, the plan cell it stands in, its storey and its
/// side (0 to 5 for the mockup's edge `d`, or the middle).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Laid {
    pub kind: Kind,
    pub cell: usize,
    pub storey: u32,
    pub side: u8,
}

/// The neighbour of town cell `(c, r)` across edge `d`.
fn neighbour(c: i32, r: i32, d: usize) -> (i32, i32) {
    super::neighbour(c, r, d)
}

/// A small, stable generator off the building.
fn mix(mut z: u64) -> u64 {
    z = z.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

struct Layout<'a> {
    plan: &'a Plan,
    def: &'a BuildingDef,
    out: Vec<Laid>,
}

impl Layout<'_> {
    fn town_cell(&self, i: usize) -> (i32, i32) {
        self.plan.cells[i]
    }

    fn stair(&self, i: usize) -> bool {
        let (c, r) = self.town_cell(i);
        self.def.stair_cells.contains(&[c, r])
            || self
                .def
                .newel
                .as_ref()
                .is_some_and(|_| self.plan.cells.len() == 1)
    }

    fn hearth(&self, i: usize, storey: u32) -> bool {
        let (c, r) = self.town_cell(i);
        storey == 0 && self.def.chimney == Some([c, r])
    }

    fn doors(&self, i: usize, storey: u32) -> Vec<u8> {
        let (c, r) = self.town_cell(i);
        self.def
            .doors
            .iter()
            .filter(|d| d[0] == c && d[1] == r && d[3] as u32 == storey)
            .map(|d| d[2] as u8)
            .collect()
    }

    fn outer(&self, i: usize, side: u8) -> bool {
        let (c, r) = self.town_cell(i);
        let (c2, r2) = neighbour(c, r, usize::from(side));
        self.plan.index(c2, r2).is_none()
    }

    fn window(&self, i: usize, side: u8, storey: u32) -> bool {
        let (c, r) = self.town_cell(i);
        self.def
            .windows
            .contains(&[c, r, i32::from(side), storey as i32])
    }

    /// Whether a cell is free to furnish on a storey.
    fn usable(&self, i: usize, storey: u32) -> bool {
        !self.stair(i) && !self.hearth(i, storey)
    }

    /// Lay `kind` if it fits: inside its cell, clear of its doors and of
    /// every piece already in the cell. A piece that backs on a wall wants
    /// an outer side; a tall one a side with no window.
    fn try_lay(&mut self, kind: Kind, cell: usize, storey: u32, side: u8) -> bool {
        if !self.usable(cell, storey) {
            return false;
        }
        let backs = matches!(
            kind,
            Kind::Bed | Kind::Drawers | Kind::Closet | Kind::Chest | Kind::BandedChest | Kind::Crate
        );
        if side < MIDDLE && backs && !self.outer(cell, side) {
            return false;
        }
        if side < MIDDLE && kind == Kind::Closet && self.window(cell, side, storey) {
            return false;
        }
        let mine = furniture::footprint(kind, side);
        if !furniture::fits(&mine) || furniture::in_doorway(kind, side, &self.doors(cell, storey)) {
            return false;
        }
        // A door's cell keeps its middle clear, so the way in is open.
        if side == MIDDLE && !self.doors(cell, storey).is_empty() {
            return false;
        }
        let crosses = self.out.iter().any(|l| {
            l.cell == cell
                && l.storey == storey
                && (l.side == side || furniture::overlaps(&mine, &furniture::footprint(l.kind, l.side)))
        });
        if crosses {
            return false;
        }
        self.out.push(Laid {
            kind,
            cell,
            storey,
            side,
        });
        true
    }

    /// Lay `kind` on the first side, from `from` round, where it fits.
    fn round(&mut self, kind: Kind, cell: usize, storey: u32, from: u64) -> bool {
        (0..6).any(|k| self.try_lay(kind, cell, storey, ((from + k) % 6) as u8))
    }

    /// The cells usable on a storey, the ones with most neighbours inside the
    /// building first (the middle of the house), then by their order.
    fn cells_by_middle(&self, storey: u32) -> Vec<usize> {
        let mut cells: Vec<usize> = (0..self.plan.cells.len())
            .filter(|&i| self.usable(i, storey))
            .collect();
        cells.sort_by_key(|&i| {
            let inside = (0..6).filter(|&d| !self.outer(i, d)).count();
            (std::cmp::Reverse(inside), i)
        });
        cells
    }

    /// A table with seats round it in `cell`: benches in a wide room, chairs
    /// or stools otherwise.
    fn table_set(&mut self, cell: usize, storey: u32, seed: u64, benches: bool) {
        if !self.try_lay(Kind::Table, cell, storey, MIDDLE) {
            return;
        }
        let mut seats = 0;
        for k in 0..6u64 {
            let side = ((seed + k * 3 + k / 2) % 6) as u8;
            let kind = if benches {
                Kind::Bench
            } else if mix(seed ^ k) % 3 == 0 {
                Kind::Stool
            } else {
                Kind::Chair
            };
            if self.try_lay(kind, cell, storey, side) {
                seats += 1;
            }
            if seats >= if benches { 2 } else { 3 } {
                break;
            }
        }
    }
}

/// A building's furniture, in its plan.
pub(super) fn layout(plan: &Plan, def: &BuildingDef, seed: u64) -> Vec<Laid> {
    let Some(kind) = use_of(&def.name) else {
        return Vec::new();
    };
    let storeys = def.storeys.max(1);
    let top = storeys - 1;
    let mut l = Layout {
        plan,
        def,
        out: Vec::new(),
    };
    let n = plan.cells.len();
    let wide = n >= 6;
    match kind {
        Use::House => {
            let ground = l.cells_by_middle(0);
            if storeys > 1 || n > 1 {
                if let Some(&t) = ground.iter().find(|&&i| l.doors(i, 0).is_empty()) {
                    l.table_set(t, 0, seed, wide && mix(seed) % 2 == 0);
                }
            }
            // A crate or drawers by the ground floor's walls.
            for (k, &i) in ground.iter().rev().enumerate().take(2) {
                let piece = if (seed + k as u64) % 2 == 0 {
                    Kind::Crate
                } else {
                    Kind::Drawers
                };
                if l.round(piece, i, 0, seed + k as u64) {
                    break;
                }
            }
            // A house of one cell has no room for a bed clear of its door:
            // a chest and a stool by its walls.
            if n == 1 {
                l.round(Kind::Chest, 0, 0, seed + 1);
                l.round(Kind::Stool, 0, 0, seed + 3);
                return l.out;
            }
            // Beds above (or beside, in a house of one storey).
            let rooms = l.cells_by_middle(top);
            let beds = if n >= 6 { 2 } else { 1 };
            let mut laid = 0;
            for (k, &i) in rooms.iter().rev().enumerate() {
                if laid >= beds {
                    break;
                }
                if l.round(Kind::Bed, i, top, seed + 2 * k as u64) {
                    laid += 1;
                    l.round(Kind::Chest, i, top, seed + 1);
                }
            }
            {
                let piece = if mix(seed ^ 7) % 2 == 0 {
                    Kind::Closet
                } else {
                    Kind::Drawers
                };
                for &i in &rooms {
                    if l.round(piece, i, top, seed + 4) {
                        break;
                    }
                }
            }
        }
        Use::Inn => {
            let ground = l.cells_by_middle(0);
            let mut sets = 0;
            for &i in &ground {
                if sets >= 3 {
                    break;
                }
                if l.doors(i, 0).is_empty() {
                    let before = l.out.len();
                    l.table_set(i, 0, seed + i as u64, true);
                    if l.out.len() > before {
                        sets += 1;
                    }
                }
            }
            if storeys > 1 {
                let mut beds = 0;
                for &i in l.cells_by_middle(top).iter().rev() {
                    if beds >= 4 {
                        break;
                    }
                    if l.round(Kind::Bed, i, top, seed + i as u64) {
                        beds += 1;
                        l.round(Kind::Chest, i, top, seed + 2);
                    }
                }
            }
        }
        Use::Hall => {
            let ground = l.cells_by_middle(0);
            let mut sets = 0;
            for &i in &ground {
                if sets >= 3 {
                    break;
                }
                if l.doors(i, 0).is_empty() {
                    let before = l.out.len();
                    l.table_set(i, 0, seed + i as u64, true);
                    if l.out.len() > before {
                        sets += 1;
                    }
                }
            }
            for &i in ground.iter().rev() {
                if l.round(Kind::BandedChest, i, 0, seed) {
                    break;
                }
            }
        }
        Use::Store => {
            let cells = l.cells_by_middle(0);
            let mut laid = 0;
            for (k, &i) in cells.iter().rev().enumerate() {
                for piece in [Kind::Crate, Kind::Crate, Kind::Chest] {
                    if laid < 3 && l.round(piece, i, 0, seed + k as u64 + laid) {
                        laid += 1;
                    }
                }
            }
        }
    }
    l.out
}
