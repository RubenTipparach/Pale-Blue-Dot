//! Old saves, whose edits are keyed by the colliding hash, moved once to
//! exact keys (`openspec/changes/exact-cell-keys`, design decision 3).
//!
//! An old `edits.log` names each edit's cell by `cell_key::old_hash`, and
//! about one finest cell in two hundred shares its hash with another. So each
//! edit line is resolved to the cell it was made on:
//!
//! 1. A hash that names one cell names it.
//! 2. A hash that names more is settled by the edit's own layer: an edit is
//!    made in the ground near its surface, so the candidate whose generated
//!    surface lies within [`SURFACE_LAYERS`] of the edit's layer is the one.
//! 3. Failing that, by where the player was when the world was last saved:
//!    the nearest candidate.
//! 4. Failing that, the edit is written once for every candidate, which is
//!    exactly what the game did with it before, and its hash is reported.
//!
//! The result is a new file, `edits.v1.log`, written whole through the save
//! thread before the world is shown. `edits.log` is never written again: an
//! older build still opens it, and never reads an exact key as a hash.

use super::format::{self, Record};
use crate::planet::lattice::{Lattice, LatticePoint};
use bevy::prelude::*;
use pbd_core::cell_key::{self, Address};
use pbd_core::column;
use pbd_core::planet_gen::TerrainConfig;
use std::collections::{BTreeMap, HashSet};

/// The old log's name. Read once, to migrate it, and never written.
pub const LEGACY_LOG: &str = "edits.log";

/// How many layers from its cell's generated surface an edit may be and
/// still be taken as made there. A dig or a placed block is within a few
/// metres of the ground it was made in; a twin's surface is anywhere in the
/// planet's 280 m of relief. An edit deep in a cave is settled by position.
pub const SURFACE_LAYERS: usize = 6;

/// What a migration did, for its log line and its tests.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    /// Edit lines read.
    pub edits: usize,
    /// Settled because the hash names one cell.
    pub unique: usize,
    /// Settled by the ground.
    pub by_surface: usize,
    /// Settled by the saved position.
    pub by_position: usize,
    /// Hashes nothing settled, written once per candidate, in log order.
    pub ambiguous: Vec<u32>,
    /// Hashes no finest cell has, dropped: a key read from them would name
    /// some other cell.
    pub unknown: Vec<u32>,
}

/// Rewrite an old log's text with exact keys. `position` is where the player
/// was when the world was last saved, planet-local metres. Every other line
/// (a kit, a catch, a change of tool, a damaged line) is copied as it was.
pub fn convert(text: &str, position: Option<Vec3>) -> (String, Report) {
    let used: HashSet<u32> = text
        .lines()
        .filter_map(|line| match format::parse_line(line) {
            Some(Record::Edit { edit, .. }) => Some(edit.cell),
            _ => None,
        })
        .collect();
    let table = reverse_table(&used);
    let mut resolver = Resolver {
        lattice: Lattice::default(),
        position: position.and_then(|p| p.try_normalize()),
    };
    let mut report = Report::default();
    let mut out = String::with_capacity(text.len() + text.len() / 4);
    for line in text.lines() {
        let Some(Record::Edit { edit, .. }) = format::parse_line(line) else {
            out.push_str(line);
            out.push('\n');
            continue;
        };
        report.edits += 1;
        let rest = line
            .trim_start()
            .split_once(char::is_whitespace)
            .map_or("", |(_, rest)| rest);
        let candidates = table.get(&edit.cell).map_or(&[][..], Vec::as_slice);
        let keys = match candidates {
            [] => {
                report.unknown.push(edit.cell);
                continue;
            }
            [only] => {
                report.unique += 1;
                vec![*only]
            }
            _ => match resolver.settle(candidates, edit.layer as usize) {
                Settled::Surface(key) => {
                    report.by_surface += 1;
                    vec![key]
                }
                Settled::Position(key) => {
                    report.by_position += 1;
                    vec![key]
                }
                Settled::No(keys) => {
                    report.ambiguous.push(edit.cell);
                    keys
                }
            },
        };
        for key in keys {
            out.push_str(&key.to_string());
            out.push(' ');
            out.push_str(rest);
            out.push('\n');
        }
    }
    (out, report)
}

/// Every finest cell whose old hash is one of `used`, by hash: the keys of
/// the cells an old save's edits may have meant. A point on a face's edge was
/// hashed by whichever of its addresses the tier met first, so each of its
/// addresses is looked up. In key order, which is the enumeration's.
pub fn reverse_table(used: &HashSet<u32>) -> BTreeMap<u32, Vec<u32>> {
    let mut table: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    if used.is_empty() {
        return table;
    }
    cell_key::each_point(cell_key::LAST_LEVEL, |a| {
        let mut add = |address: Address| {
            let hash = cell_key::old_hash(address);
            if used.contains(&hash) {
                let key = cell_key::key(a).expect("the finest level carries keys");
                let keys = table.entry(hash).or_default();
                if keys.last() != Some(&key) {
                    keys.push(key);
                }
            }
        };
        let n = a.side();
        if a.i > 0 && a.j > 0 && a.i + a.j < n {
            add(a);
        } else {
            for address in cell_key::addresses(a) {
                add(address);
            }
        }
    });
    table
}

/// Two finest cells whose old hashes collide: an edit to either was an edit
/// to both before exact keys. For the gate video's shots, and for anyone
/// checking a save by hand (`--colliding-pairs`).
#[derive(Clone, Debug)]
pub struct CollidingPair {
    pub hash: u32,
    /// Each cell's key and centre, nearer to the point asked about first.
    pub cells: [(u32, Vec3); 2],
}

/// The colliding pairs nearest `point`, by their nearer cell, at most
/// `count`. Every finest point's every address is hashed, as the tier may
/// have met a seam point by any of them.
pub fn colliding_pairs_near(point: Vec3, count: usize) -> Vec<CollidingPair> {
    let point = point.normalize_or(Vec3::Y);
    let (corners, faces) = pbd_core::topology::icosahedron();
    // Where a point lies, near enough to rank by: its face's corners,
    // weighted by its lattice address. The exact lattice position is taken
    // only for the pairs kept.
    let rough = |a: Address| {
        let [p, q, r] = faces[a.face as usize];
        let n = a.side() as f32;
        (corners[p] * (n - (a.i + a.j) as f32) + corners[q] * a.i as f32 + corners[r] * a.j as f32)
            .normalize()
    };
    let mut hashed = Vec::with_capacity(cell_key::point_count(cell_key::LAST_LEVEL) as usize);
    cell_key::each_point(cell_key::LAST_LEVEL, |a| {
        let key = cell_key::key(a).expect("the finest level carries keys");
        if a.i > 0 && a.j > 0 && a.i + a.j < a.side() {
            hashed.push((cell_key::old_hash(a), key));
        } else {
            for address in cell_key::addresses(a) {
                hashed.push((cell_key::old_hash(address), key));
            }
        }
    });
    hashed.sort_unstable();
    hashed.dedup();
    let mut pairs: Vec<(f32, u32, u32, u32)> = hashed
        .windows(2)
        .filter(|w| w[0].0 == w[1].0)
        .map(|w| {
            let near = |key: u32| 1.0 - rough(cell_key::unpack(key).unwrap()).dot(point);
            let (a, b) = if near(w[0].1) <= near(w[1].1) {
                (w[0].1, w[1].1)
            } else {
                (w[1].1, w[0].1)
            };
            (near(a), w[0].0, a, b)
        })
        .collect();
    pairs.sort_by(|x, y| x.0.total_cmp(&y.0).then(x.1.cmp(&y.1)));
    let mut resolver = Resolver {
        lattice: Lattice::default(),
        position: None,
    };
    pairs
        .into_iter()
        .take(count)
        .map(|(_, hash, a, b)| CollidingPair {
            hash,
            cells: [(a, resolver.direction(a)), (b, resolver.direction(b))],
        })
        .collect()
}

enum Settled {
    Surface(u32),
    Position(u32),
    No(Vec<u32>),
}

struct Resolver {
    lattice: Lattice,
    position: Option<Vec3>,
}

impl Resolver {
    fn direction(&mut self, key: u32) -> Vec3 {
        let a = cell_key::unpack(key).expect("a table key unpacks");
        self.lattice.position(LatticePoint {
            face: a.face,
            level: a.level,
            i: a.i,
            j: a.j,
        })
    }

    fn settle(&mut self, candidates: &[u32], layer: usize) -> Settled {
        let near: Vec<u32> = candidates
            .iter()
            .copied()
            .filter(|&key| {
                // The world's own ground: every log this reads is from before
                // identities, so from generator 4, whatever the newest is
                // (CLAUDE.md "Saved games survive every change").
                let surface = column::surface_m(&TerrainConfig::TENEBRIS_V4, self.direction(key));
                column::layer_at(surface).is_some_and(|s| s.abs_diff(layer) <= SURFACE_LAYERS)
            })
            .collect();
        if let [only] = near[..] {
            return Settled::Surface(only);
        }
        let pool = if near.is_empty() {
            candidates.to_vec()
        } else {
            near
        };
        if let Some(at) = self.position {
            let mut by_distance: Vec<(f32, u32)> = pool
                .iter()
                .map(|&key| (1.0 - self.direction(key).dot(at), key))
                .collect();
            by_distance.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
            if by_distance.len() == 1 || by_distance[0].0 < by_distance[1].0 {
                return Settled::Position(by_distance[0].1);
            }
        }
        Settled::No(pool)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planet::PLANET_RADIUS;
    use std::sync::OnceLock;

    /// Pairs of finest cells whose old hashes collide, as (hash, key, key),
    /// found once for every test here.
    fn pairs() -> &'static [(u32, u32, u32)] {
        static PAIRS: OnceLock<Vec<(u32, u32, u32)>> = OnceLock::new();
        PAIRS.get_or_init(|| {
            let mut all = Vec::with_capacity(cell_key::point_count(11) as usize);
            cell_key::each_point(11, |a| {
                if a.i > 0 && a.j > 0 && a.i + a.j < a.side() {
                    all.push((cell_key::old_hash(a), cell_key::key(a).unwrap()));
                }
            });
            all.sort_unstable();
            all.windows(2)
                .filter(|w| w[0].0 == w[1].0)
                .map(|w| (w[0].0, w[0].1, w[1].1))
                .take(2_000)
                .collect()
        })
    }

    fn direction(key: u32) -> Vec3 {
        let a = cell_key::unpack(key).unwrap();
        Lattice::default().position(LatticePoint {
            face: a.face,
            level: a.level,
            i: a.i,
            j: a.j,
        })
    }

    fn surface_layer(key: u32) -> usize {
        column::layer_at(column::surface_m(
            &TerrainConfig::TENEBRIS_V4,
            direction(key),
        ))
        .unwrap()
    }

    /// An edit line on `cell` at `layer`, with a hotbar.
    fn line(cell: u32, layer: usize) -> String {
        format!("{cell} {layer} 0 b3,2 - - - - - - - - -")
    }

    #[test]
    fn a_unique_hash_names_its_one_cell() {
        let a = Address {
            face: 7,
            level: 11,
            i: 300,
            j: 400,
        };
        let hash = cell_key::old_hash(a);
        assert!(pairs().iter().all(|p| p.0 != hash), "not a colliding hash");
        let text = format!("{}\nkit 3 - - - - - - - - - -\n", line(hash, 150));
        let (out, report) = convert(&text, None);
        assert_eq!(report.unique, 1);
        assert_eq!(
            out,
            format!(
                "{}\nkit 3 - - - - - - - - - -\n",
                line(cell_key::key(a).unwrap(), 150)
            )
        );
    }

    /// A dig at one twin's surface is that twin's, when the other twin's
    /// surface is well away from it.
    #[test]
    fn the_ground_tells_a_pair_apart() {
        let &(hash, a, b) = pairs()
            .iter()
            .find(|&&(_, a, b)| surface_layer(a).abs_diff(surface_layer(b)) > 3 * SURFACE_LAYERS)
            .expect("a pair with different ground");
        for (made_on, other) in [(a, b), (b, a)] {
            let (out, report) = convert(&line(hash, surface_layer(made_on) - 1), None);
            assert_eq!(report.by_surface, 1, "{report:?}");
            assert!(out.starts_with(&format!("{made_on} ")), "{out}");
            assert!(!out.contains(&format!("{other} ")));
        }
    }

    /// Deep under both twins, the ground says nothing, and the saved position
    /// settles it: the twin the player was near.
    #[test]
    fn the_saved_position_tells_a_pair_apart() {
        let &(hash, a, b) = pairs()
            .iter()
            .find(|&&(_, a, b)| surface_layer(a).min(surface_layer(b)) > 3 * SURFACE_LAYERS + 5)
            .expect("a pair above the column's floor");
        let deep = surface_layer(a).min(surface_layer(b)) - 3 * SURFACE_LAYERS;
        let near_b = direction(b) * (PLANET_RADIUS + 10.0);
        let (out, report) = convert(&line(hash, deep), Some(near_b));
        assert_eq!(report.by_position, 1, "{report:?}");
        assert_eq!(out, format!("{}\n", line(b, deep)));
        let _ = a;
    }

    /// Nothing tells them apart: the edit is written for both, which is what
    /// the game did before, and the hash is reported.
    #[test]
    fn a_truly_ambiguous_pair_keeps_todays_behaviour() {
        let &(hash, a, b) = pairs()
            .iter()
            .find(|&&(_, a, b)| surface_layer(a).min(surface_layer(b)) > 3 * SURFACE_LAYERS + 5)
            .expect("a pair above the column's floor");
        let deep = surface_layer(a).min(surface_layer(b)) - 3 * SURFACE_LAYERS;
        let (out, report) = convert(&line(hash, deep), None);
        assert_eq!(report.ambiguous, vec![hash]);
        assert_eq!(out, format!("{}\n{}\n", line(a, deep), line(b, deep)));
    }

    /// A number no finest cell ever hashed to is dropped, never read as a
    /// key; the lines round it are kept in order.
    #[test]
    fn a_hash_no_cell_has_is_dropped_and_the_rest_kept() {
        // About one number in a hundred is some cell's hash, so one of the
        // first few is not.
        let unused = (1..10u32)
            .find(|&n| reverse_table(&[n].into_iter().collect()).is_empty())
            .expect("a number no cell hashes to");
        let text = format!(
            "hand 1 3\n{}\ngarbage line\ncatch 4 31 - - - - - - - - - -\n",
            line(unused, 150)
        );
        let (out, report) = convert(&text, None);
        assert_eq!(report.unknown, vec![unused]);
        assert_eq!(
            out,
            "hand 1 3\ngarbage line\ncatch 4 31 - - - - - - - - - -\n"
        );
    }

    /// The load an old save pays once: a synthetic log of 100,000 edits on
    /// finest cells spread over the planet, a few on colliding pairs. Timed
    /// for the design; the release figure is the one recorded.
    #[test]
    fn a_hundred_thousand_edits_migrate_in_one_pass() {
        let mut text = String::new();
        let mut state = 0x2545_f491_u32;
        for n in 0..100_000u32 {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            let hash = if n % 1_000 == 0 {
                pairs()[(n / 1_000) as usize].0
            } else {
                let face = (state % 20) as u8;
                let i = 1 + (state >> 5) % 1_000;
                let j = 1 + (state >> 15) % 1_000;
                cell_key::old_hash(Address {
                    face,
                    level: 11,
                    i,
                    j,
                })
            };
            text.push_str(&line(hash, 150));
            text.push('\n');
        }
        let started = std::time::Instant::now();
        let (out, report) = convert(&text, Some(Vec3::Y * PLANET_RADIUS));
        eprintln!(
            "100,000 edits migrated in {:.2} s: {report:?}",
            started.elapsed().as_secs_f64()
        );
        assert_eq!(report.edits, 100_000);
        assert!(report.unknown.is_empty());
        assert!(out.lines().count() >= 100_000);
    }
}
