//! The blocks round the player (`repro-capture` design 6): every cell within
//! three hex rings of the player's cell, and in each the layers from three
//! below the feet's layer to three above the head's, with what the game has
//! there now.
//!
//! The owner, 2026-11-16: "Shift alt F should also read local block data as
//! well within a range of 3 of the player", then "So 3 up 3 down 3 forward,
//! left etc".
//!
//! Distance is HEX RINGS on the finest level's own neighbour table
//! (`FineSet::finest_neighbors`): ring 1 is the six (or five, at a pentagon)
//! cells sharing an edge with the player's, ring 2 the cells one more edge
//! out, and so on. Everything is read from resident data, the fine set's
//! records and columns and the save's edits and records, never generated: a
//! cell whose column is not resident says so and lists no layers.
//!
//! locale: dev

use bevy::prelude::*;
use pbd_core::column;
use pbd_core::edits::Edits;
use pbd_core::liquid::Cell as Liquid;
use pbd_core::records::Records;
use serde::Serialize;

use crate::planet::PlanetContact;
use crate::planet::lod::FineSet;

/// How many rings out, and how many layers past the feet and the head.
pub const RINGS: u32 = 3;
pub const LAYERS_PAST: usize = 3;

/// The biome names, in `planet_gen::Biome`'s `repr(u32)` order: the record's
/// material word carries the index (`terrain::surface_code`).
const BIOMES: [&str; 8] = [
    "ocean",
    "beach",
    "fields",
    "desert",
    "jungle",
    "swamp",
    "mountains",
    "tundra",
];

/// The dump.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BlockDump {
    /// How distance is counted.
    pub distance: &'static str,
    /// The player's cell's key.
    pub player_cell: u32,
    /// Which of the player's cell's sides the look faces most nearly, 0..6
    /// in the neighbour table's order, and that side's compass bearing.
    pub facing_side: usize,
    pub facing_bearing_deg: f32,
    /// The feet's and the head's layers, and the span dumped.
    pub feet_layer: usize,
    pub head_layer: usize,
    pub from_layer: usize,
    pub to_layer: usize,
    pub cells: Vec<CellDump>,
}

/// One cell.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CellDump {
    pub key: u32,
    pub ring: u32,
    /// Its centre from the player's cell's, in cell spacings, along the look
    /// (forward positive) and across it (right positive).
    pub forward: f32,
    pub right: f32,
    /// The record's surface height, metres against sea level.
    pub surface_height_m: f32,
    /// The column's topmost solid layer, where its column is resident.
    pub surface_layer: Option<usize>,
    pub biome: &'static str,
    /// The topmost solid layer's material.
    pub top: Option<String>,
    /// A town's footprint: no tree grows on it.
    pub town_ground: bool,
    /// Whether its column is resident; without one there are no layers.
    pub column_resident: bool,
    /// The top of the water standing over its surface, metres, if any.
    pub water_top_m: Option<f32>,
    pub layers: Vec<LayerDump>,
    /// Every stored record on the cell: `kind id: body`.
    pub records: Vec<String>,
}

/// One layer of one cell.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LayerDump {
    pub layer: usize,
    /// The bottom of the layer, metres against sea level.
    pub altitude_m: f32,
    pub material: String,
    /// A player's (or the world's) edit, not what was generated.
    pub edit: bool,
    /// `source`, `flowing N` or `falling N`, where it is water.
    pub water: Option<String>,
    /// The top of the water run this layer is in.
    pub water_top_m: Option<f32>,
}

impl CellDump {
    /// Where it is from the player, as the table prints it.
    pub fn offset(&self) -> String {
        if self.ring == 0 {
            return "here".into();
        }
        let along = if self.forward >= 0.0 { "fwd" } else { "back" };
        let across = if self.right >= 0.0 { "right" } else { "left" };
        format!(
            "{:.1} {along} {:.1} {across}",
            self.forward.abs(),
            self.right.abs()
        )
    }
}

fn centre(fine: &FineSet, record: usize) -> Vec3 {
    Vec3::from_slice(&fine.finest_records()[record].direction_height[..3]).normalize_or(Vec3::Y)
}

/// The finest record a direction is in: the contact's walk where it serves
/// this set, else the nearest centre.
fn record_at(fine: &FineSet, contact: Option<&PlanetContact>, direction: Vec3) -> Option<usize> {
    if let Some(record) = contact.and_then(|c| c.finest_cell(direction)) {
        return Some(record);
    }
    fine.finest_records()
        .iter()
        .enumerate()
        .max_by(|a, b| {
            let da = Vec3::from_slice(&a.1.direction_height[..3]).dot(direction);
            let db = Vec3::from_slice(&b.1.direction_height[..3]).dot(direction);
            da.total_cmp(&db)
        })
        .map(|(i, _)| i)
}

/// Every record within `rings` of `start`, with its ring, nearest first and
/// in the neighbour table's order within a ring.
pub fn rings_round(fine: &FineSet, start: usize, rings: u32) -> Vec<(usize, u32)> {
    let mut seen = vec![(start, 0u32)];
    let mut at = 0;
    while at < seen.len() {
        let (record, ring) = seen[at];
        at += 1;
        if ring == rings {
            continue;
        }
        let Some(neighbours) = fine.finest_neighbors.get(record) else {
            continue;
        };
        for &next in neighbours {
            let next = next as usize;
            if next < fine.finest_records().len() && !seen.iter().any(|(r, _)| *r == next) {
                seen.push((next, ring + 1));
            }
        }
    }
    seen
}

/// Whether a stored record stands on `cell`, by its kind's id rule.
fn on_cell(record: &pbd_core::records::Record, cell: u32) -> bool {
    use pbd_core::husbandry::{AnimalBody, Edge, record::ANIMAL, record::FENCE};
    match record.kind.as_str() {
        pbd_core::station::STATION_KIND => (record.id >> 16) as u32 == cell,
        FENCE => {
            let (a, b) = Edge(record.id).cells();
            a == cell || b == cell
        }
        ANIMAL => record.read::<AnimalBody>().is_some_and(|a| a.home == cell),
        _ => record.id == u64::from(cell),
    }
}

/// What the water in a layer is.
fn water_of(material: pbd_core::terrain::Material) -> Option<String> {
    match Liquid::of(material) {
        Liquid::Source => Some("source".into()),
        Liquid::Flowing { level, falling } => Some(format!(
            "{} {level}",
            if falling { "falling" } else { "flowing" }
        )),
        _ => None,
    }
}

/// The dump round a player whose feet and eye are at these body-local
/// points, looking along `forward`. `None` where the feet's direction is
/// off the resident fine set.
#[allow(clippy::too_many_arguments)]
pub fn dump(
    fine: &FineSet,
    contact: Option<&PlanetContact>,
    edits: &Edits,
    records: &Records,
    axe_logs: u16,
    feet: Vec3,
    eye: Vec3,
    forward: Vec3,
) -> Option<BlockDump> {
    let up = feet.normalize_or(Vec3::Y);
    let start = record_at(fine, contact, up)?;
    let here = centre(fine, start);
    let heading = (forward - up * forward.dot(up)).normalize_or(Vec3::Y.cross(up));
    let right = heading.cross(up).normalize_or(Vec3::X);
    // One cell spacing: the mean distance to the player's cell's neighbours.
    let neighbours: Vec<usize> = fine.finest_neighbors[start]
        .iter()
        .map(|&n| n as usize)
        .filter(|&n| n < fine.finest_records().len())
        .collect();
    let spacing = neighbours
        .iter()
        .map(|&n| (centre(fine, n) - here).length())
        .sum::<f32>()
        / neighbours.len().max(1) as f32;
    let spacing = spacing.max(1e-9);
    let (facing_side, _) = fine.finest_neighbors[start]
        .iter()
        .enumerate()
        .filter(|(_, n)| (**n as usize) < fine.finest_records().len())
        .map(|(side, &n)| (side, (centre(fine, n as usize) - here).dot(heading)))
        .max_by(|a, b| a.1.total_cmp(&b.1))?;
    let facing_toward = centre(fine, fine.finest_neighbors[start][facing_side] as usize) - here;
    let facing_bearing_deg = pbd_core::geo::compass_heading(here, facing_toward)
        .to_degrees()
        .rem_euclid(360.0);
    let altitude = |p: Vec3| p.length() - crate::planet::PLANET_RADIUS;
    let feet_layer = column::layer_at(altitude(feet) + 0.01)?;
    let head_layer = column::layer_at(altitude(eye))?.max(feet_layer);
    let from_layer = feet_layer.saturating_sub(LAYERS_PAST).max(1);
    let to_layer = (head_layer + LAYERS_PAST).min(column::LAYERS - 1);
    let around = rings_round(fine, start, RINGS);
    let keys: Vec<u32> = around
        .iter()
        .map(|(r, _)| fine.finest_records()[*r].key())
        .collect();
    let mut on: Vec<Vec<String>> = vec![Vec::new(); around.len()];
    for record in records.iter() {
        for (i, key) in keys.iter().enumerate() {
            if on_cell(record, *key) {
                let mut line = format!("{} {}: {}", record.kind, record.id, record.body);
                if record.kind == pbd_core::gather::TREE_KIND
                    && record
                        .read::<pbd_core::gather::TreeTaken>()
                        .is_some_and(|t| t.logs >= axe_logs)
                {
                    line.push_str(" (felled)");
                }
                on[i].push(line);
            }
        }
    }
    let cells = around
        .iter()
        .zip(keys.iter())
        .zip(on)
        .map(|(((record, ring), key), records)| {
            let gpu = &fine.finest_records()[*record];
            let at = centre(fine, *record) - here;
            let column = fine.columns.column(*record);
            let edited = edits.for_cell(*key);
            let layers = column.map_or_else(Vec::new, |column| {
                (from_layer..=to_layer)
                    .map(|layer| {
                        let material = column.material(layer);
                        LayerDump {
                            layer,
                            altitude_m: column::layer_altitude(layer),
                            material: format!("{material:?}"),
                            edit: edited.iter().any(|&(l, _)| usize::from(l) == layer),
                            water: water_of(material),
                            water_top_m: column.water_surface(column::layer_altitude(layer) + 0.5),
                        }
                    })
                    .collect()
            });
            let surface_layer = column.and_then(|c| c.surface());
            let word = gpu.metadata[1];
            CellDump {
                key: *key,
                ring: *ring,
                forward: at.dot(heading) / spacing,
                right: at.dot(right) / spacing,
                surface_height_m: gpu.direction_height[3],
                surface_layer,
                biome: BIOMES
                    .get(((word >> 8) & 0xff) as usize)
                    .copied()
                    .unwrap_or("unknown"),
                top: column
                    .zip(surface_layer)
                    .map(|(c, top)| format!("{:?}", c.material(top))),
                town_ground: word & (1 << crate::planet::terrain::CLEARED_BIT) != 0,
                column_resident: column.is_some(),
                water_top_m: column
                    .zip(surface_layer)
                    .and_then(|(c, top)| c.water_surface(column::layer_altitude(top + 1) + 0.5)),
                layers,
                records,
            }
        })
        .collect();
    Some(BlockDump {
        distance: "hex rings over the finest level's neighbour table",
        player_cell: keys[0],
        facing_side,
        facing_bearing_deg,
        feet_layer,
        head_layer,
        from_layer,
        to_layer,
        cells,
    })
}

impl BlockDump {
    /// The table the text record carries.
    pub fn table(&self) -> String {
        let mut out = format!(
            "blocks: {} cells within {RINGS} rings ({}), layers {}..={} \
             (feet L{}, head L{}; L = metres above {} m); facing side {} \
             (bearing {:.0} deg); * = edited\n",
            self.cells.len(),
            self.distance,
            self.from_layer,
            self.to_layer,
            self.feet_layer,
            self.head_layer,
            column::BASE_M,
            self.facing_side,
            self.facing_bearing_deg
        );
        for cell in &self.cells {
            out.push_str(&format!(
                "cell {} ring {} ({}): surface {:.2} m{}, {}, top {}{}{}\n",
                cell.key,
                cell.ring,
                cell.offset(),
                cell.surface_height_m,
                cell.surface_layer
                    .map_or(String::new(), |l| format!(" (L{l})")),
                cell.biome,
                cell.top.as_deref().unwrap_or("-"),
                if cell.town_ground {
                    ", town ground"
                } else {
                    ""
                },
                cell.water_top_m
                    .map_or(String::new(), |w| format!(", water top {w:.2} m")),
            ));
            if !cell.column_resident {
                out.push_str("  (no resident column)\n");
            }
            if !cell.layers.is_empty() {
                let row: Vec<String> = cell
                    .layers
                    .iter()
                    .rev()
                    .map(|l| {
                        format!(
                            "L{} {}{}",
                            l.layer,
                            l.material,
                            if l.edit { "*" } else { "" }
                        )
                    })
                    .collect();
                out.push_str(&format!("  {}\n", row.join(" | ")));
            }
            for record in &cell.records {
                out.push_str(&format!("  record {record}\n"));
            }
        }
        out
    }
}
