//! Beavers and their dams: the world changing itself (`ocean-environments`
//! decision 8 and design B25).
//!
//! A colony is a stored record (`rec @... colony <n> 1 {...}`) and `beaver`
//! is the world process that acts for it, once an in-game hour while the
//! world is played. Each hour it proposes at most one change, through the
//! same durable path as a player's and authored `@wbeaver:colony/<n>`:
//!
//! - **fell**: a tree near its site is felled, the `tree` record the
//!   player's axe would write (`gather`), so the draw leaves a stump;
//! - **dam**: a cell of [`Material::Sticks`] across the stream's lip, bottom
//!   layer first, up to the bank;
//! - **lodge**: sticks in the reach beside the bank, once the dam is whole.
//!
//! Every proposal is checked whole against what the player has touched
//! ([`YieldSet`]). A refused dam (the player built on the lip first, or
//! broke the dam, which makes those cells theirs) moves the colony to its
//! next site up the river; with none left it is gone. A broken dam is never
//! rebuilt on the player's cells.
//!
//! Where the sites are, and which cells and trees they hold, is the app's to
//! find on the ground (the river, its lip, the trees); everything here is a
//! pure function of the record, the hour and the yield set, so a colony left
//! for forty days catches up in one call to where hourly steps put it.

use crate::edits::Edit;
use crate::gather::{TREE_KIND, TREE_SCHEMA, TreeTaken};
use crate::records::{Author, Proposal, Record, Records, YieldSet};
use crate::terrain::Material;
use serde::{Deserialize, Serialize};

/// The record kind and its schema.
pub const KIND: &str = "colony";
pub const SCHEMA: u32 = 1;
/// The world process's name.
pub const PROCESS: &str = "beaver";

/// One place a colony can settle: a still reach of a river.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Site {
    /// Where it is: the unit direction of the lip's middle, for finding a
    /// colony near the player. A query direction, never integrated.
    pub at: [f32; 3],
    /// The cells across the stream where the reach's water leaves it: the
    /// dam's line, by exact cell key.
    pub lip: Vec<u32>,
    /// Cells of the reach beside the bank, where the lodge is built.
    pub pond: Vec<u32>,
    /// The stream bed's top layer at the lip (the dam's first layer is the
    /// one over it) and the bank's top layer (the dam stops under it).
    pub bed: u16,
    pub bank: u16,
    /// The generator's trees within reach of the site, nearest first, by
    /// tree id (the draw's and the axe's id).
    pub trees: Vec<u64>,
}

impl Site {
    /// How many layers the dam stands: from the bed up to the bank, never
    /// more than `max`.
    pub fn dam_layers(&self, max: u16) -> u16 {
        self.bank.saturating_sub(self.bed).clamp(1, max)
    }
}

/// What the colony is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum State {
    Felling,
    Damming,
    Lodging,
    Lodged,
    Gone,
}

/// The colony record's body.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Colony {
    /// Which of `sites` it is at.
    pub site: u16,
    /// Its site and the next ones up the same river, where it moves on.
    pub sites: Vec<Site>,
    pub members: u8,
    /// Trees felled at this site.
    pub felled: Vec<u64>,
    /// Cells it laid at this site, `(cell, layer)`, in the order laid.
    pub built: Vec<(u32, u16)>,
    /// The last world hour it acted for.
    pub last_h: u64,
    pub state: State,
}

/// The process's rules, with their numbers (design B25).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rules {
    /// A tree is felled every this many hours while felling.
    pub fell_every_h: u64,
    /// Trees down before the dam is begun, and at most felled at a site.
    pub fell_before_dam: usize,
    pub fell_most: usize,
    /// The dam's most layers.
    pub dam_layers: u16,
    /// Sticks cells in the lodge.
    pub lodge_cells: usize,
    /// How far back a catch-up reaches, hours: forty days.
    pub catch_up_h: u64,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            fell_every_h: 6,
            fell_before_dam: 3,
            fell_most: 6,
            dam_layers: 4,
            lodge_cells: 3,
            catch_up_h: 40 * 24,
        }
    }
}

/// One hour's act.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    Fell(u64),
    Lay {
        cell: u32,
        layer: u16,
    },
    /// Moved on to its next site (the dam was refused).
    Moved,
    /// Nowhere left to go.
    Gone,
}

impl Colony {
    /// A new colony at the first of `sites`, acting from `hour`.
    pub fn new(sites: Vec<Site>, members: u8, hour: u64) -> Self {
        Self {
            site: 0,
            sites,
            members,
            felled: Vec::new(),
            built: Vec::new(),
            last_h: hour,
            state: State::Felling,
        }
    }

    pub fn here(&self) -> Option<&Site> {
        self.sites.get(usize::from(self.site))
    }

    /// The dam's cells, in the order they are laid: bottom layer first,
    /// across the lip.
    pub fn dam(&self, rules: &Rules) -> Vec<(u32, u16)> {
        let Some(site) = self.here() else {
            return Vec::new();
        };
        (1..=site.dam_layers(rules.dam_layers))
            .flat_map(|k| site.lip.iter().map(move |&c| (c, site.bed + k)))
            .collect()
    }

    /// The lodge's cells: on the reach's bed beside the bank.
    pub fn lodge(&self, rules: &Rules) -> Vec<(u32, u16)> {
        let Some(site) = self.here() else {
            return Vec::new();
        };
        site.pond
            .iter()
            .take(rules.lodge_cells)
            .map(|&c| (c, site.bed + 1))
            .collect()
    }

    /// What it would do at `hour`, if anything, and the colony after.
    pub fn step(&self, hour: u64, rules: &Rules) -> Option<(Act, Colony)> {
        let site = self.here()?;
        let mut next = self.clone();
        next.last_h = hour;
        match self.state {
            State::Gone | State::Lodged => None,
            State::Felling => {
                if self.felled.len() >= rules.fell_before_dam.min(site.trees.len()) {
                    next.state = State::Damming;
                    return next.step(hour, rules);
                }
                if !hour.is_multiple_of(rules.fell_every_h) {
                    return None;
                }
                let tree = *site.trees.iter().find(|t| !self.felled.contains(t))?;
                next.felled.push(tree);
                Some((Act::Fell(tree), next))
            }
            State::Damming => {
                let dam = self.dam(rules);
                if let Some(&(cell, layer)) = dam.iter().find(|c| !self.built.contains(c)) {
                    next.built.push((cell, layer));
                    return Some((Act::Lay { cell, layer }, next));
                }
                next.state = State::Lodging;
                next.step(hour, rules)
            }
            State::Lodging => {
                let lodge = self.lodge(rules);
                if let Some(&(cell, layer)) = lodge.iter().find(|c| !self.built.contains(c)) {
                    next.built.push((cell, layer));
                    // The last of the lodge: it is lodged, in the same entry.
                    if lodge.iter().all(|c| next.built.contains(c)) {
                        next.state = State::Lodged;
                    }
                    return Some((Act::Lay { cell, layer }, next));
                }
                next.state = State::Lodged;
                None
            }
        }
    }

    /// The colony moved on from a refused dam: its next site, nothing
    /// built or felled there yet; or gone.
    pub fn moved(&self, hour: u64) -> (Act, Colony) {
        let mut next = self.clone();
        next.last_h = hour;
        next.built.clear();
        next.felled.clear();
        if usize::from(self.site) + 1 < self.sites.len() {
            next.site += 1;
            next.state = State::Felling;
            (Act::Moved, next)
        } else {
            next.state = State::Gone;
            (Act::Gone, next)
        }
    }

    pub fn record(&self, id: u64) -> Record {
        Record::of(KIND, id, SCHEMA, self)
    }
}

/// The process's author for colony `id`.
pub fn author(id: u64) -> Author {
    Author::World {
        process: PROCESS.into(),
        kind: KIND.into(),
        id,
    }
}

/// What an act writes: the colony's record, and the tree's or the cell's.
pub fn proposal(id: u64, act: Act, after: &Colony, axe_logs: u16) -> Proposal {
    let mut proposal = Proposal {
        edits: Vec::new(),
        records: vec![after.record(id)],
    };
    match act {
        Act::Fell(tree) => proposal.records.push(Record::of(
            TREE_KIND,
            tree,
            TREE_SCHEMA,
            &TreeTaken { logs: axe_logs },
        )),
        Act::Lay { cell, layer } => proposal.edits.push(Edit {
            cell,
            layer,
            material: Material::Sticks,
        }),
        Act::Moved | Act::Gone => {}
    }
    proposal
}

/// Every proposal the colonies in `records` make from their last hour up to
/// `now_h`, in order, each checked against `yields` as the save will check
/// it and taken into a working copy of the records, so the next hour sees
/// it. A dam cell the player holds moves the colony on rather than being
/// laid. At most `rules.catch_up_h` hours are run per colony: a colony left
/// longer resumes from there.
pub fn catch_up(
    records: &Records,
    yields: &YieldSet,
    now_h: u64,
    rules: &Rules,
    axe_logs: u16,
) -> Vec<(Author, Proposal)> {
    let mut working = records.clone();
    let mut out = Vec::new();
    let colonies: Vec<(u64, Colony)> = records
        .of_kind(KIND)
        .filter_map(|r| Some((r.id, r.read::<Colony>()?)))
        .collect();
    for (id, mut colony) in colonies {
        let from = colony.last_h.max(now_h.saturating_sub(rules.catch_up_h));
        for hour in (from + 1)..=now_h {
            // A dam or lodge the player has broken into is theirs: the
            // colony moves on, whatever it was doing (a lodged colony too).
            if colony.state != State::Gone
                && colony.built.iter().any(|&(cell, _)| yields.has_cell(cell))
            {
                let (moved, next) = colony.moved(hour);
                let proposal = proposal(id, moved, &next, axe_logs);
                if yields.check(&proposal, &working).is_err() {
                    break;
                }
                for record in &proposal.records {
                    working.put(record.clone());
                }
                out.push((author(id), proposal));
                colony = next;
                if moved == Act::Gone {
                    break;
                }
                continue;
            }
            let Some((act, after)) = colony.step(hour, rules) else {
                // Nothing to do this hour; a state change with no act is
                // still the colony's, kept for the next.
                if let Some(next) = settled(&colony, hour, rules) {
                    colony = next;
                }
                continue;
            };
            let mut proposal = proposal(id, act, &after, axe_logs);
            let (act, after) = match yields.check(&proposal, &working) {
                Ok(()) => (act, after),
                Err(_) if matches!(act, Act::Lay { .. }) => {
                    let (moved, next) = colony.moved(hour);
                    proposal = self::proposal(id, moved, &next, axe_logs);
                    if yields.check(&proposal, &working).is_err() {
                        break;
                    }
                    (moved, next)
                }
                // A tree the player had a hand in is left standing: it is
                // passed over (kept in `felled`, which is the trees dealt
                // with) and nothing is written for it.
                Err(_) => {
                    let mut next = after;
                    next.last_h = hour;
                    colony = next;
                    continue;
                }
            };
            for record in &proposal.records {
                working.put(record.clone());
            }
            out.push((author(id), proposal));
            colony = after;
            if act == Act::Gone {
                break;
            }
        }
    }
    out
}

/// A colony whose state moves on with no act (a whole lodge): the colony
/// after, or `None` where nothing changes.
fn settled(colony: &Colony, hour: u64, rules: &Rules) -> Option<Colony> {
    if colony.state == State::Lodging
        && colony.lodge(rules).iter().all(|c| colony.built.contains(c))
    {
        let mut next = colony.clone();
        next.state = State::Lodged;
        next.last_h = hour;
        return Some(next);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn site(base: u32) -> Site {
        Site {
            at: [0.0, 1.0, 0.0],
            lip: vec![base, base + 1, base + 2],
            pond: vec![base + 10, base + 11, base + 12, base + 13],
            bed: 140,
            bank: 143,
            trees: (1..=4).map(|k| u64::from(base) * 100 + k).collect(),
        }
    }

    fn world(id: u64, colony: &Colony) -> Records {
        let mut records = Records::new();
        records.put(colony.record(id));
        records
    }

    /// Takes every proposal into the records and the cells, as the save does.
    fn apply(records: &mut Records, cells: &mut Vec<Edit>, proposals: &[(Author, Proposal)]) {
        for (_, p) in proposals {
            for r in &p.records {
                records.put(r.clone());
            }
            cells.extend(p.edits.iter().copied());
        }
    }

    /// Forty days in hourly steps and in one catch-up come to the same
    /// colony and the same journal; and by then it has felled, dammed the
    /// lip from the bed to the bank and built its lodge.
    #[test]
    fn a_colony_catches_up_forty_days_in_one_call_as_in_hourly_steps() {
        let rules = Rules::default();
        let start = Colony::new(vec![site(1000), site(2000)], 4, 0);
        let yields = YieldSet::new();
        let day40 = 40 * 24;

        let mut hourly = world(7, &start);
        let mut hourly_cells = Vec::new();
        let mut hourly_journal = Vec::new();
        for hour in 1..=day40 {
            let out = catch_up(&hourly, &yields, hour, &rules, 4);
            apply(&mut hourly, &mut hourly_cells, &out);
            hourly_journal.extend(out);
        }

        let mut once = world(7, &start);
        let mut once_cells = Vec::new();
        let out = catch_up(&once, &yields, day40, &rules, 4);
        apply(&mut once, &mut once_cells, &out);

        assert_eq!(hourly_cells, once_cells);
        assert_eq!(hourly_journal, out);
        let colony: Colony = once.get(KIND, 7).unwrap().read().unwrap();
        assert_eq!(colony.state, State::Lodged, "{colony:?}");
        assert_eq!(colony.site, 0);
        assert_eq!(colony.felled.len(), 3);
        // Three layers across a three-cell lip, then three lodge cells.
        assert_eq!(once_cells.len(), 3 * 3 + 3);
        assert!(once_cells.iter().all(|e| e.material == Material::Sticks));
        let dam: Vec<(u32, u16)> = once_cells.iter().map(|e| (e.cell, e.layer)).collect();
        assert_eq!(
            &dam[..3],
            &[(1000, 141), (1001, 141), (1002, 141)],
            "bottom first"
        );
        assert!(dam[..9].iter().all(|&(_, l)| l > 140 && l <= 143));
        // Three trees felled, as tree records the draw reads.
        for tree in &colony.felled {
            let taken: TreeTaken = once.get(TREE_KIND, *tree).unwrap().read().unwrap();
            assert_eq!(taken.logs, 4);
        }
        // Every entry is the process's, for this colony.
        assert!(out.iter().all(|(a, _)| *a == author(7)));
        assert_eq!(author(7).token(), "@wbeaver:colony/7");
        // A colony seen again later acts no more for hours already run.
        assert!(catch_up(&once, &yields, day40, &rules, 4).is_empty());
    }

    /// A dam the player breaks is theirs: the colony never lays a cell on
    /// it again, and moves to its next site, where it starts afresh; with
    /// no site left, it is gone.
    #[test]
    fn a_broken_dam_is_not_rebuilt_on_the_players_cells_and_the_colony_moves_on() {
        let rules = Rules::default();
        let start = Colony::new(vec![site(1000), site(2000)], 4, 0);
        let mut records = world(3, &start);
        let mut cells = Vec::new();
        let mut yields = YieldSet::new();
        // Three trees felled (hours 6, 12, 18) and four dam cells laid.
        let out = catch_up(&records, &yields, 22, &rules, 4);
        apply(&mut records, &mut cells, &out);
        let colony: Colony = records.get(KIND, 3).unwrap().read().unwrap();
        assert_eq!(colony.state, State::Damming);
        assert!(!colony.built.is_empty());
        // The player breaks the dam: every cell of it is theirs now, and
        // the cells it had not yet laid on the lip too (they dug the lip).
        for &(cell, _) in &colony.dam(&rules) {
            yields.touch_cell(cell);
        }
        let out = catch_up(&records, &yields, 23, &rules, 4);
        apply(&mut records, &mut cells, &out);
        let colony: Colony = records.get(KIND, 3).unwrap().read().unwrap();
        assert_eq!(colony.site, 1, "moved to its next site: {colony:?}");
        assert!(
            out.iter()
                .flat_map(|(_, p)| &p.edits)
                .all(|e| !yields.has_cell(e.cell)),
            "no cell laid on the player's"
        );
        // It builds at the next site, and never again on the first.
        let out = catch_up(&records, &yields, 40 * 24, &rules, 4);
        apply(&mut records, &mut cells, &out);
        let colony: Colony = records.get(KIND, 3).unwrap().read().unwrap();
        assert_eq!(colony.state, State::Lodged);
        let laid_later: Vec<u32> = out
            .iter()
            .flat_map(|(_, p)| &p.edits)
            .map(|e| e.cell)
            .collect();
        assert!(laid_later.iter().all(|c| *c >= 2000), "{laid_later:?}");
        // The player holds the second lip before it is dammed: with no
        // third site the colony is gone, and lays nothing.
        let start = Colony::new(vec![site(1000), site(2000)], 4, 0);
        let mut records = world(4, &start);
        let mut yields = YieldSet::new();
        for c in [1000, 1001, 1002, 2000, 2001, 2002] {
            yields.touch_cell(c);
        }
        let mut cells = Vec::new();
        let out = catch_up(&records, &yields, 40 * 24, &rules, 4);
        apply(&mut records, &mut cells, &out);
        let colony: Colony = records.get(KIND, 4).unwrap().read().unwrap();
        assert_eq!(colony.state, State::Gone);
        assert!(cells.is_empty(), "{cells:?}");
    }

    /// A tree the player has cut from is never felled by the colony: the
    /// proposal would change a record field the player changed.
    #[test]
    fn a_colony_leaves_the_players_trees() {
        let rules = Rules::default();
        let start = Colony::new(vec![site(1000)], 4, 0);
        let mut records = world(5, &start);
        let mut yields = YieldSet::new();
        let theirs = Record::of(TREE_KIND, 100_001, TREE_SCHEMA, &TreeTaken { logs: 1 });
        yields.touch_record(None, &theirs);
        records.put(theirs);
        let out = catch_up(&records, &yields, 40 * 24, &rules, 4);
        let felled: Vec<u64> = out
            .iter()
            .flat_map(|(_, p)| &p.records)
            .filter(|r| r.kind == TREE_KIND)
            .map(|r| r.id)
            .collect();
        assert!(!felled.contains(&100_001), "{felled:?}");
        // The player's tree was passed over, and two more were felled
        // before the dam was begun.
        assert_eq!(felled, vec![100_002, 100_003]);
    }
    /// A lodged colony whose dam the player breaks moves on too: it is not
    /// rebuilt, and the record names the next site.
    #[test]
    fn a_lodged_colony_moves_on_when_its_dam_is_broken() {
        let rules = Rules::default();
        let start = Colony::new(vec![site(1000), site(2000)], 4, 0);
        let mut records = world(9, &start);
        let mut cells = Vec::new();
        let out = catch_up(&records, &YieldSet::new(), 100, &rules, 4);
        apply(&mut records, &mut cells, &out);
        let colony: Colony = records.get(KIND, 9).unwrap().read().unwrap();
        assert_eq!(colony.state, State::Lodged);
        let mut yields = YieldSet::new();
        yields.touch_cell(1001);
        let out = catch_up(&records, &yields, 101, &rules, 4);
        apply(&mut records, &mut cells, &out);
        let colony: Colony = records.get(KIND, 9).unwrap().read().unwrap();
        // It moved at the first hour it acted for after the break, and has
        // built at its next site since; nothing on the first.
        assert_eq!(colony.site, 1);
        assert!(
            out.iter()
                .flat_map(|(_, p)| &p.edits)
                .all(|e| e.cell >= 2000),
            "nothing laid on the broken site"
        );
    }
}
