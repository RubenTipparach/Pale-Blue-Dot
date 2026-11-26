//! The planner's tunables (`building-planner` design 12) and its checks
//! (design 2): a plan is charted onto a flat stand-in patch and held to build
//! mode's own rule functions, plus the three things a plan has and a piece
//! does not (its size, its storeys, a way in).

use super::plan::{Charted, Hex, Plan, PlanPiece, chart_plan, distance, flat_patch};
use super::rules::{Rule, first_broken};
use super::{Cells, Opening, Site};
use serde::{Deserialize, Serialize};

/// What a dropped built piece does when a site's plan is changed (design 7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Removal {
    /// Marked, and taken down with the mallet, top first.
    Mark,
    /// Taken down at once on save, its goods to the pack.
    Return,
}

/// Mallet strokes a piece takes (design 6).
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Strokes {
    pub wall: u32,
    pub floor: u32,
    /// A roof, per cell it covers.
    pub roof_cell: u32,
    pub stair: u32,
    pub post: u32,
    pub lantern: u32,
    /// A doorway's door, hung with its wall.
    pub door: u32,
}

/// What scaffolding costs (design 6): planks a bay, and a rope for every so
/// many bays.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScaffoldCost {
    pub planks_per_bay: u32,
    pub bays_per_rope: u32,
}

/// `assets/config/building_planner.ron` (design 12).
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Tunables {
    /// Cells, the biggest plan.
    pub max_cells: usize,
    /// Cells from the origin: a site always cuts in one plot.
    pub max_radius: i32,
    /// Storeys, the tallest plan.
    pub max_storeys: usize,
    /// Sixths a shift-wheel turns a ghost.
    pub rotation_step: u8,
    /// Layers between the highest and lowest ground under a ground floor.
    pub max_footing: u16,
    /// 0..1, a ghost that can be built now.
    pub ghost_opacity: f32,
    /// 0..1, a ghost waiting on others.
    pub ghost_waiting_opacity: f32,
    /// Linear RGB, the ghost's tint.
    pub ghost_tint: [f32; 3],
    /// Metres: how near the ghost and the HUD show.
    pub ghost_reach_m: f32,
    /// 0..1: the ghost's brightness at night against the day's. A ghost is
    /// a preview, not lit by the field (design 5), so it is dimmed with the
    /// sun instead, never left glowing in the dark.
    pub ghost_night: f32,
    pub strokes: Strokes,
    /// Seconds, the mallet's beat.
    pub stroke_s: f32,
    /// Metres F reaches a ghost.
    pub reach_m: f32,
    pub scaffold: bool,
    pub scaffold_cost: ScaffoldCost,
    pub removal: Removal,
    /// 0..1, goods back from a piece taken down.
    pub refund_share: f32,
    /// Every cost is scaled by this.
    pub cost_scale: f32,
}

impl Tunables {
    /// Every number in its range.
    pub fn validate(&self) -> Result<(), String> {
        let unit = |name: &str, v: f32| {
            if (0.0..=1.0).contains(&v) {
                Ok(())
            } else {
                Err(format!("{name} {v} is not in 0..1"))
            }
        };
        unit("ghost_opacity", self.ghost_opacity)?;
        unit("ghost_waiting_opacity", self.ghost_waiting_opacity)?;
        unit("ghost_night", self.ghost_night)?;
        unit("refund_share", self.refund_share)?;
        for c in &self.ghost_tint {
            unit("ghost_tint", *c)?;
        }
        if self.max_cells == 0 || self.max_radius < 1 || self.max_storeys == 0 {
            return Err("a plan must have room for a cell and a storey".into());
        }
        if !(1..=5).contains(&self.rotation_step) {
            return Err("rotation_step is 1 to 5 sixths".into());
        }
        for (name, v) in [
            ("ghost_reach_m", self.ghost_reach_m),
            ("stroke_s", self.stroke_s),
            ("reach_m", self.reach_m),
            ("cost_scale", self.cost_scale),
        ] {
            if !(v.is_finite() && v > 0.0) {
                return Err(format!("{name} {v} must be above zero"));
            }
        }
        if self.scaffold_cost.bays_per_rope == 0 {
            return Err("scaffold_cost.bays_per_rope must be above zero".into());
        }
        Ok(())
    }

    /// The tunables the game ships.
    pub fn shipped() -> Self {
        ron::from_str(include_str!(
            "../../../../assets/config/building_planner.ron"
        ))
        .expect("building_planner.ron parses")
    }

    /// The strokes a piece takes.
    pub fn strokes_of(&self, piece: &super::Piece) -> u32 {
        use super::Shape;
        let s = &self.strokes;
        match &piece.shape {
            Shape::Wall { .. } => s.wall,
            Shape::Floor { .. } => s.floor,
            Shape::Stair { .. } => s.stair,
            Shape::Post { .. } | Shape::Beam { .. } => s.post,
            Shape::Lantern { .. } => s.lantern,
            Shape::Door { .. } => s.door,
            Shape::Roof { cells, .. } => s.roof_cell * cells.len().max(1) as u32,
            Shape::Furniture { .. } => 0,
        }
        .max(u32::from(!matches!(piece.shape, Shape::Furniture { .. })))
    }
}

/// What a planner check is about: one of build mode's rules, or what only a
/// plan has.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Check {
    Rule(Rule),
    /// More cells than `max_cells`, or wider than `max_radius`.
    PlanSize,
    /// More storeys than `max_storeys`.
    Storeys,
    /// No door on the ground storey (a warning).
    WayIn,
    /// Nothing painted yet.
    Empty,
    /// The plan does not chart (a cell past the stand-in's reach).
    Chart,
}

impl Check {
    /// Its reason's key in the locale file.
    pub fn key(self) -> String {
        match self {
            Check::Rule(r) => format!("build.refused.{}", r.id()),
            Check::PlanSize => "planner.check.plan_size".into(),
            Check::Storeys => "planner.check.storeys".into(),
            Check::WayIn => "planner.check.way_in".into(),
            Check::Empty => "planner.check.empty".into(),
            Check::Chart => "planner.check.plan_size".into(),
        }
    }

    /// Whether it keeps a plan from being set out (a warning does not).
    pub fn refuses(self) -> bool {
        self != Check::WayIn
    }
}

/// A broken check, where it breaks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Issue {
    pub check: Check,
    /// The storey it shows on; `None` for the whole plan.
    pub storey: Option<u8>,
    /// The plan cells to show it on.
    pub cells: Vec<Hex>,
    /// The plan piece that breaks it, by its place in [`Plan::pieces`].
    pub piece: Option<usize>,
}

/// The layer the stand-in's ground is at.
pub const FLAT_GROUND: u16 = 120;

/// The plan charted onto the flat stand-in (design 2), and the stand-in.
pub fn on_flat(
    plan: &Plan,
    reach: i32,
) -> Result<(crate::settlement::chart::Patch, Charted), String> {
    let extent = plan
        .footprint()
        .iter()
        .map(|&c| distance(c))
        .max()
        .unwrap_or(0);
    let patch = flat_patch(extent.max(reach) + 2);
    let charted = chart_plan(&patch, plan, 0, 0, FLAT_GROUND)?;
    Ok((patch, charted))
}

/// Every check the plan breaks (design 2): build mode's rules on every
/// piece against all the others, then the plan's own size, storeys and way
/// in. An empty list is a plan that may be set out.
pub fn check_plan(plan: &Plan, tun: &Tunables) -> Vec<Issue> {
    let mut out = Vec::new();
    if plan.is_empty() {
        out.push(Issue {
            check: Check::Empty,
            storey: None,
            cells: Vec::new(),
            piece: None,
        });
        return out;
    }
    let footprint = plan.footprint();
    let far: Vec<Hex> = footprint
        .iter()
        .copied()
        .filter(|&c| distance(c) > tun.max_radius)
        .collect();
    if footprint.len() > tun.max_cells || !far.is_empty() {
        out.push(Issue {
            check: Check::PlanSize,
            storey: None,
            cells: far,
            piece: None,
        });
    }
    if plan.storeys.len() > tun.max_storeys {
        out.push(Issue {
            check: Check::Storeys,
            storey: Some(plan.storeys.len() as u8 - 1),
            cells: Vec::new(),
            piece: None,
        });
    }
    match on_flat(plan, tun.max_radius) {
        Ok((patch, charted)) => {
            let cells = Cells::new(&patch);
            let ground = |_: u32| Some(FLAT_GROUND);
            let site = Site {
                cells: &cells,
                ground: &ground,
            };
            let pieces: Vec<_> = charted.iter().map(|(_, p)| p.clone()).collect();
            for (i, (pp, piece)) in charted.iter().enumerate() {
                let others = pieces
                    .iter()
                    .enumerate()
                    .filter(move |(j, _)| *j != i)
                    .map(|(_, p)| p);
                if let Some(rule) = first_broken(&site, others, piece) {
                    out.push(Issue {
                        check: Check::Rule(rule),
                        storey: Some(pp.storey()),
                        cells: pp.cells(),
                        piece: Some(i),
                    });
                }
            }
        }
        Err(_) => out.push(Issue {
            check: Check::Chart,
            storey: None,
            cells: Vec::new(),
            piece: None,
        }),
    }
    let door = plan.pieces().iter().any(|p| {
        matches!(
            p,
            PlanPiece::Wall {
                storey: 0,
                opening: Opening::Door,
                ..
            }
        )
    });
    if !door {
        out.push(Issue {
            check: Check::WayIn,
            storey: Some(0),
            cells: Vec::new(),
            piece: None,
        });
    }
    out
}

/// Whether a plan may be set out: nothing it breaks refuses it.
pub fn plan_ok(plan: &Plan, tun: &Tunables) -> bool {
    !check_plan(plan, tun).iter().any(|i| i.check.refuses())
}
