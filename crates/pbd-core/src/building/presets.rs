//! The preset library (`building-planner` design 4):
//! `assets/config/blueprints.ron` holds each preset as a footprint and a
//! storey count at three sizes; its doors, windows, stairs, lanterns and
//! furniture slots are laid out by rule ([`Preset::plan`]), as M6 lays them.
//! Every plain building of the settlement templates joins the library too
//! ([`from_template`]), so the towns' houses are presets without being
//! copied by hand.

use super::RoofShape;
use super::plan::{
    Hex, Mark, Plan, PlanLantern, PlanSlot, PlanStair, corner, distance, edge, middle, step,
};
use super::site::Size;
use crate::furniture::{Kind, MIDDLE, fits, footprint, in_doorway};
use crate::settlement::{BuildingDef, Kit, RoofKind, axial};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// A footprint by rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Footprint {
    /// `w` cells along a row, `h` rows, centred on the origin.
    Rect(i32, i32),
    /// `n` cells in a ring round the origin.
    Blob(usize),
}

/// One size of a preset.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct PresetSize {
    pub cells: Footprint,
    pub storeys: u8,
}

/// A preset: its id is saved in a site's record and never renamed once
/// shipped; its name is `blueprint.preset.<id>`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Preset {
    pub id: String,
    pub kit: String,
    pub roof: RoofShape,
    /// Small, standard and large.
    pub sizes: Vec<PresetSize>,
    /// Doors side by side on the front: two for a double door.
    pub doors: usize,
    /// Windows round each storey.
    pub windows: usize,
    /// The furniture slots, beds upstairs.
    pub slots: Vec<Kind>,
}

/// `assets/config/blueprints.ron`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Presets {
    pub presets: Vec<Preset>,
}

impl Presets {
    pub fn shipped() -> Self {
        ron::from_str(include_str!("../../../../assets/config/blueprints.ron"))
            .expect("blueprints.ron parses")
    }

    pub fn get(&self, id: &str) -> Option<&Preset> {
        self.presets.iter().find(|p| p.id == id)
    }
}

/// The cells of a footprint.
pub fn cells_of(f: Footprint) -> Vec<Hex> {
    match f {
        Footprint::Rect(w, h) => {
            let cells: Vec<Hex> = (0..h)
                .flat_map(|r| (0..w).map(move |q| (q - r.div_euclid(2), r)))
                .collect();
            centred(cells)
        }
        Footprint::Blob(n) => {
            let mut out = vec![(0, 0)];
            let mut rad = 1;
            while out.len() < n {
                // From DIRS[4] times the radius round the ring, as M6's
                // `blob`.
                let mut at = (0, -rad);
                for d in 0..6u8 {
                    for _ in 0..rad {
                        if out.len() >= n {
                            break;
                        }
                        out.push(at);
                        at = step(at, d);
                    }
                }
                rad += 1;
            }
            out
        }
    }
}

/// Cells moved so their middle is on the origin cell.
fn centred(cells: Vec<Hex>) -> Vec<Hex> {
    let n = cells.len().max(1) as f32;
    let (mx, mz) = cells.iter().fold((0.0, 0.0), |(x, z), &c| {
        let (a, b) = middle(c);
        (x + a, z + b)
    });
    let (dq, dr) = nearest(mx / n, mz / n);
    cells.into_iter().map(|(q, r)| (q - dq, r - dr)).collect()
}

/// The plan cell under a point of the drawing.
pub fn nearest(x: f32, z: f32) -> Hex {
    let r = z / 1.5;
    let q = x / 3f32.sqrt() - r / 2.0;
    let s = -q - r;
    let (mut rq, mut rr, rs) = (q.round(), r.round(), s.round());
    let (dq, dr, ds) = ((rq - q).abs(), (rr - r).abs(), (rs - s).abs());
    if dq > dr && dq > ds {
        rq = -rr - rs;
    } else if dr > ds {
        rr = -rq - rs;
    }
    (rq as i32, rr as i32)
}

/// The outside edges of a storey, round the room from the back.
fn outer_edges(plan: &Plan, storey: u8) -> Vec<(Hex, u8, f32)> {
    let cells = &plan.storeys[usize::from(storey)];
    let n = cells.len().max(1) as f32;
    let (cx, cz) = cells.iter().fold((0.0, 0.0), |(x, z), &c| {
        let (a, b) = middle(c);
        (x + a / n, z + b / n)
    });
    let mut out: Vec<(Hex, u8, f32, f32)> = cells
        .iter()
        .flat_map(|&c| (0..6u8).map(move |d| (c, d)))
        .filter(|&(c, d)| !cells.contains(&step(c, d)))
        .map(|(c, d)| {
            let (a, b) = (corner(c, d), corner(c, d + 1));
            let (x, z) = ((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0);
            (c, d, (z - cz).atan2(x - cx), x)
        })
        .collect();
    out.sort_by(|a, b| a.2.total_cmp(&b.2).then(a.0.cmp(&b.0)).then(a.1.cmp(&b.1)));
    out.into_iter().map(|(c, d, _, x)| (c, d, x)).collect()
}

/// A straight flight up from each storey but the top (M6's `autoStairs`).
pub fn auto_stairs(plan: &mut Plan) {
    plan.stairs.clear();
    for s in 0..plan.storeys.len().saturating_sub(1) {
        let s8 = s as u8;
        let wells = plan.wells(s8);
        let here = plan.storeys[s].clone();
        let up = plan.storeys[s + 1].clone();
        'cells: for &foot in &here {
            for d in [0u8, 3, 1, 4, 2, 5] {
                let head = step(foot, d);
                let land = step(head, d);
                if wells.contains(&foot)
                    || wells.contains(&head)
                    || !here.contains(&head)
                    || !up.contains(&land)
                    || !up.contains(&foot)
                    || !up.contains(&head)
                {
                    continue;
                }
                plan.stairs.push(PlanStair {
                    foot,
                    side: d,
                    storey: s8,
                });
                break 'cells;
            }
        }
    }
}

/// The sides of `cell` on `storey` with a door in their wall.
fn door_sides(plan: &Plan, cell: Hex, storey: u8) -> Vec<u8> {
    (0..6u8)
        .filter(|&d| plan.edges.get(&edge(storey, cell, d)) == Some(&Mark::Door))
        .collect()
}

/// Whether a cell has a doorway beside it (in its own walls or its
/// neighbours'), where furniture would be in the way.
fn by_a_door(plan: &Plan, cell: Hex, storey: u8) -> bool {
    !door_sides(plan, cell, storey).is_empty()
}

/// The side a slot's back goes to in `cell`: an outer wall with nothing in
/// it, so it stands against the wall; a table stands in the middle.
fn slot_side(plan: &Plan, cell: Hex, storey: u8, kind: Kind) -> Option<u8> {
    let doors = door_sides(plan, cell, storey);
    let ok = |side: u8| fits(&footprint(kind, side)) && !in_doorway(kind, side, &doors);
    if kind == Kind::Table {
        return ok(MIDDLE).then_some(MIDDLE);
    }
    let cells = &plan.storeys[usize::from(storey)];
    // Backs against plain outer walls first, then any side.
    let plain = (0..6u8).filter(|&d| {
        !cells.contains(&step(cell, d)) && !plan.edges.contains_key(&edge(storey, cell, d))
    });
    let any = 0..6u8;
    // Every side of a hexagon is alike to the furniture's footprint, and
    // the door sides are counted the same way, so plan sides serve.
    plain.chain(any).find(|&d| ok(d))
}

impl Preset {
    /// The preset at a size, laid out by rule (design 4).
    pub fn plan(&self, size: Size) -> Plan {
        let spec = self.sizes[size.index().min(self.sizes.len().saturating_sub(1))];
        let cells: BTreeSet<Hex> = cells_of(spec.cells).into_iter().collect();
        let mut plan = Plan::new(&self.kit);
        plan.roof = self.roof;
        plan.storeys = (0..spec.storeys.max(1)).map(|_| cells.clone()).collect();
        auto_stairs(&mut plan);
        lay_out(&mut plan, self.doors, self.windows, &self.slots);
        plan
    }
}

/// Doors on the front's middle, windows spread round each storey, a lantern
/// in each storey's middle and the furniture slots on the cells left, as M6
/// lays a preset out.
fn lay_out(plan: &mut Plan, doors: usize, windows: usize, slots: &[Kind]) {
    let n = plan.storeys.len() as u8;
    let mut front: Vec<(Hex, u8, f32)> = outer_edges(plan, 0)
        .into_iter()
        .filter(|&(_, d, _)| d == 1 || d == 2)
        .collect();
    front.sort_by(|a, b| a.2.total_cmp(&b.2));
    let mid = front.len().saturating_sub(1) / 2;
    let door_edges: BTreeSet<_> = front
        .iter()
        .skip(mid)
        .take(doors)
        .map(|&(c, d, _)| edge(0, c, d))
        .collect();
    for &e in &door_edges {
        plan.edges.insert(e, Mark::Door);
    }
    for s in 0..n {
        if windows == 0 {
            break;
        }
        let es: Vec<_> = outer_edges(plan, s)
            .into_iter()
            .filter(|&(c, d, _)| !door_edges.contains(&edge(s, c, d)))
            .collect();
        let gap = (es.len() / windows).max(1);
        let mut i = gap / 2;
        let mut placed = 0;
        while i < es.len() && placed < windows {
            let (c, d, _) = es[i];
            plan.edges.insert(edge(s, c, d), Mark::Window);
            i += gap;
            placed += 1;
        }
    }
    for s in 0..n {
        let mut taken: BTreeSet<Hex> = plan.wells(s);
        for t in plan.stairs.iter().filter(|t| t.storey == s) {
            taken.insert(t.foot);
            taken.insert(step(t.foot, t.side));
        }
        let mut free: Vec<Hex> = plan.storeys[usize::from(s)]
            .iter()
            .copied()
            .filter(|c| !taken.contains(c))
            .collect();
        free.sort_by_key(|&c| (distance(c), c));
        if let Some(&c) = free.first() {
            plan.lanterns.push(PlanLantern { cell: c, storey: s });
            taken.insert(c);
        }
        let want: Vec<Kind> = slots
            .iter()
            .enumerate()
            .filter(|&(i, &k)| {
                n == 1
                    || (if k == Kind::Bed { s == n - 1 } else { s == 0 })
                    || (k == Kind::Bed && i > 0 && s > 0)
            })
            .map(|(_, &k)| k)
            .collect();
        // The cells furthest from the middle first, away from the doors;
        // then those by a door (the slot's own rule keeps the doorway
        // clear); then the lantern's, so a one-cell hut still has its bed.
        let mut spots: Vec<Hex> = free
            .iter()
            .rev()
            .copied()
            .filter(|c| !taken.contains(c) && !by_a_door(plan, *c, s))
            .collect();
        spots.extend(
            free.iter()
                .rev()
                .copied()
                .filter(|c| !taken.contains(c) && by_a_door(plan, *c, s)),
        );
        spots.extend(free.first().copied().filter(|c| taken.contains(c)));
        for kind in want {
            let Some(at) = spots
                .iter()
                .position(|&c| slot_side(plan, c, s, kind).is_some())
            else {
                continue;
            };
            let cell = spots.remove(at);
            let side = slot_side(plan, cell, s, kind).expect("found above");
            plan.slots.push(PlanSlot {
                cell,
                storey: s,
                side,
                kind,
            });
        }
    }
}

/// A template's plain building as a plan (design 4): its cells, storeys,
/// doors, archways and windows, a stair per upper storey and a lantern a
/// storey. `None` where a stair will not fit.
pub fn from_template(def: &BuildingDef, kit: &Kit) -> Option<Plan> {
    if def.cells.is_empty() {
        return None;
    }
    let to_ax = |c: i32, r: i32| axial(c, r);
    let origin = to_ax(def.cells[0][0], def.cells[0][1]);
    let at = |c: i32, r: i32| {
        let (q, r) = to_ax(c, r);
        (q - origin.0, r - origin.1)
    };
    let cells: BTreeSet<Hex> = def.cells.iter().map(|&[c, r]| at(c, r)).collect();
    let mut plan = Plan::new(&kit.name);
    plan.roof = match def.roof.as_str() {
        "cone" => RoofShape::Cone,
        "flat" => RoofShape::Flat,
        _ => match kit.roof {
            RoofKind::Cone => RoofShape::Cone,
            RoofKind::Flat | RoofKind::Dome => RoofShape::Flat,
            _ => RoofShape::Gable,
        },
    };
    plan.storeys = (0..def.storeys.max(1)).map(|_| cells.clone()).collect();
    let mark = |list: &[[i32; 4]], m: Mark, plan: &mut Plan| {
        for &[c, r, d, s] in list {
            let cell = at(c, r);
            if usize::try_from(s).is_ok_and(|s| s < plan.storeys.len()) {
                plan.edges.insert(edge(s as u8, cell, d as u8), m);
            }
        }
    };
    mark(&def.windows, Mark::Window, &mut plan);
    mark(&def.doors, Mark::Door, &mut plan);
    mark(&def.archways, Mark::Door, &mut plan);
    auto_stairs(&mut plan);
    if plan.stairs.len() + 1 < plan.storeys.len() {
        return None;
    }
    for s in 0..plan.storeys.len() as u8 {
        let mut taken = plan.wells(s);
        for t in plan.stairs.iter().filter(|t| t.storey == s) {
            taken.insert(t.foot);
            taken.insert(step(t.foot, t.side));
        }
        let mut free: Vec<Hex> = plan.storeys[usize::from(s)]
            .iter()
            .copied()
            .filter(|c| !taken.contains(c))
            .collect();
        free.sort_by_key(|&c| (distance(c), c));
        if let Some(&c) = free.first() {
            plan.lanterns.push(PlanLantern { cell: c, storey: s });
        }
    }
    Some(plan)
}
