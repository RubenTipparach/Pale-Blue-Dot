//! Settlements: what a town is made of and how it stands on the planet's
//! cells (`tenebris-towns`, `cities-in-the-world`).
//!
//! A **template** is a settlement as the towns mockup lays it out, on a flat
//! hex grid in offset coordinates (`docs/mockups/towns.html`, exported by
//! `tools/export_town_templates.js` into `assets/settlements/v1/`). A
//! **kit** is what a building is made of (`assets/config/kits.ron`).
//!
//! At a site the template is **charted** onto the real finest cells round the
//! site's anchor (`cities-in-the-world` decision 2), and each building is
//! **cut** from those cells' real corners ([`pieces`]): walls on the edges
//! it shares with the outside, with their doors and windows, floors in its
//! cells, and its roof.

use serde::{Deserialize, Serialize};

pub mod chart;
pub mod ground;
pub mod pieces;
pub mod record;

/// Metres from one layer to the next: a storey is three.
pub const LAYER_M: f32 = 1.0;
/// Floor to floor.
pub const STOREY_M: f32 = 3.0;
/// A hut's one storey (`tenebris-towns` section 9).
pub const HUT_STOREY_M: f32 = 2.0;
/// A door, wide by high, where the kit does not say.
pub const DOOR_M: (f32, f32) = (1.0, 2.2);

/// The mockup's axial directions, by the direction index its edges use:
/// direction `d`'s edge normal is at `60 d` degrees from east, turning
/// toward the south.
pub const DIRS: [(i32, i32); 6] = [(1, 0), (0, 1), (-1, 1), (-1, 0), (0, -1), (1, -1)];

/// Offset `(c, r)`, odd rows half a cell east, to axial `(q, r)`, as the
/// mockup's `toAx`.
pub fn axial(c: i32, r: i32) -> (i32, i32) {
    (c - (r - r.rem_euclid(2)) / 2, r)
}

/// Axial back to offset, as the mockup's `toOff`.
pub fn offset(q: i32, r: i32) -> (i32, i32) {
    (q + (r - r.rem_euclid(2)) / 2, r)
}

/// The offset cell beyond edge `d`, as the mockup's `nb`.
pub fn neighbour(c: i32, r: i32, d: usize) -> (i32, i32) {
    let (q, r) = axial(c, r);
    let (dq, dr) = DIRS[d % 6];
    offset(q + dq, r + dr)
}

/// A settlement as the mockup lays it out.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Template {
    pub scene: String,
    pub grid: Grid,
    pub buildings: Vec<BuildingDef>,
    /// Every cell of the mockup's grid, with the ground it painted.
    pub ground: Vec<GroundCell>,
    /// Cells with a street lamp.
    #[serde(default)]
    pub lamps: Vec<[i32; 2]>,
    /// Whether the town stands on several levels, each built cell at its
    /// own height (slice 4b): the walled town's terraces. A template without
    /// it is laid flat on one terrace, as the village always has been.
    #[serde(default)]
    pub terraced: bool,
    /// Its masonry cells (slice 4c): the walled town's curtain wall and its
    /// gates, as the mockup's `buildWalls` raises them.
    #[serde(default)]
    pub masonry: Vec<MasonryCell>,
}

/// One cell of masonry: a prism from `from` to `to`, layers on the mockup's
/// grid, with two merlons on each edge in `merlons` (the mockup's
/// directions). A gate's `from` is over its ground, and the passage under it
/// is open.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct MasonryCell {
    pub c: i32,
    pub r: i32,
    pub from: i32,
    pub to: i32,
    #[serde(default)]
    pub merlons: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
pub struct Grid {
    pub columns: i32,
    pub rows: i32,
    pub cell_m: f32,
}

/// One building, as the mockup's `building()` was asked for it.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct BuildingDef {
    pub name: String,
    pub kit: String,
    /// Offset cells.
    pub cells: Vec<[i32; 2]>,
    /// The ground floor's layer on the mockup's grid.
    pub base: i32,
    pub storeys: u32,
    /// Storeys of double height (the hall's one tall room).
    #[serde(default = "one")]
    pub tall: u32,
    /// `[c, r, d, storey]`: a door in edge `d` of cell `(c, r)`.
    pub doors: Vec<[i32; 4]>,
    /// `[c, r, d, storey]`: a window, likewise.
    pub windows: Vec<[i32; 4]>,
    /// A roofing material for a gable, or `cone`, `flat` or `dome`.
    pub roof: String,
    #[serde(default = "unit")]
    pub pitch: f32,
    pub chimney: Option<[i32; 2]>,
    #[serde(default)]
    pub stair_cells: Vec<[i32; 2]>,
}

fn one() -> u32 {
    1
}

fn unit() -> f32 {
    1.0
}

/// One cell of the mockup's ground.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct GroundCell {
    pub c: i32,
    pub r: i32,
    /// Layers over the mockup's datum.
    pub h: i32,
    /// Its top's texture: `grass`, `dirt` (a lane), `flag` (a floor), ...
    pub top: String,
    #[serde(default)]
    pub area: String,
}

impl Template {
    /// The cells a building stands on or a lane crosses: what the ground is
    /// terraced under. Offset coordinates, sorted.
    pub fn built_cells(&self) -> Vec<[i32; 2]> {
        let mut cells: Vec<[i32; 2]> = self
            .buildings
            .iter()
            .flat_map(|b| b.cells.iter().copied())
            .chain(
                self.ground
                    .iter()
                    .filter(|g| g.top != "grass" && g.top != "sand")
                    .map(|g| [g.c, g.r]),
            )
            .collect();
        cells.sort();
        cells.dedup();
        cells
    }
}

/// How a building's roof is made.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum RoofKind {
    Gable,
    Cone,
    Flat,
}

/// One storey's wall.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WallFaces {
    pub outside: String,
    pub inside: String,
    pub edge: String,
    pub thickness_m: f32,
    /// A texture mapped once across each wall's outside face, a storey
    /// high (half-timbering), rather than repeated by the metre.
    #[serde(default)]
    pub per_face: bool,
}

/// What a building is made of (`tenebris-towns` section 9).
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Kit {
    pub name: String,
    pub label: String,
    pub walls: Vec<WallFaces>,
    pub roof: RoofKind,
    pub roof_material: String,
    #[serde(default = "unit")]
    pub pitch: f32,
    pub gable: String,
    pub floor: String,
    #[serde(default)]
    pub hut: bool,
    #[serde(default = "door")]
    pub door_m: (f32, f32),
    #[serde(default)]
    pub window_m: Option<(f32, f32)>,
    #[serde(default)]
    pub chimney: Option<String>,
    #[serde(default)]
    pub columns: bool,
}

fn door() -> (f32, f32) {
    DOOR_M
}

/// `assets/config/kits.ron`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Kits {
    pub kits: Vec<Kit>,
}

impl Kits {
    pub fn get(&self, name: &str) -> Option<&Kit> {
        self.kits.iter().find(|k| k.name == name)
    }

    /// Every kit has a wall, a positive thickness, and a door that fits a
    /// wall; every name is used once. The first problem, named.
    pub fn validate(&self) -> Result<(), String> {
        let mut seen = std::collections::BTreeSet::new();
        for kit in &self.kits {
            if !seen.insert(kit.name.as_str()) {
                return Err(format!("kits: {} twice", kit.name));
            }
            if kit.walls.is_empty() {
                return Err(format!("{}: no walls", kit.name));
            }
            for wall in &kit.walls {
                if !(wall.thickness_m > 0.0 && wall.thickness_m < 1.0) {
                    return Err(format!(
                        "{}: wall thickness {} m",
                        kit.name, wall.thickness_m
                    ));
                }
            }
            let (w, h) = kit.door_m;
            if !(w > 0.0 && w < 1.4 && h > 1.5 && h < STOREY_M) {
                return Err(format!("{}: a door of {w} by {h} m", kit.name));
            }
            if !(kit.pitch > 0.0 && kit.pitch <= 2.0) {
                return Err(format!("{}: pitch {}", kit.name, kit.pitch));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
