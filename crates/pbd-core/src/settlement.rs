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
use std::collections::BTreeMap;

pub mod chart;
pub mod ground;
pub mod pieces;
pub mod record;
pub mod sea;

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
    /// Whether the town stands on the sea (slice 4d): the template's 0 m is
    /// the sea's surface, and no cell its ground puts under the sea is laid.
    #[serde(default)]
    pub sea: bool,
    /// Its piers (slice 4d): planks on piles, as the mockup's `bridge`.
    #[serde(default)]
    pub piers: Vec<Pier>,
    /// Lanterns off the grid, `[x, y, z]` in the mockup's metres: the
    /// harbour's along its quay and at its piers' ends.
    #[serde(default)]
    pub lanterns: Vec<[f32; 3]>,
    /// The harbour's light on its mole (slice 4d).
    #[serde(default)]
    pub light: Option<RoundTower>,
    /// The harbour's boats on the water (task 4.2b).
    #[serde(default)]
    pub boats: Vec<Boat>,
    /// The things it stands about its lanes, quay and piers (task 4.2c).
    #[serde(default)]
    pub dressing: Vec<Dress>,
    /// The harbour's shipyard (task 4.2c).
    #[serde(default)]
    pub shipyard: Option<Shipyard>,
    /// The harbour's cog at its mooring (`sail-the-cog` design 6, step 1).
    #[serde(default)]
    pub cog: Option<Cog>,
    /// The areas whose cells keep the planet's ground (slices 4e and 4g):
    /// the desert's dunes, the open tundra. Where a template names some,
    /// every other cell is built, whatever its top.
    #[serde(default)]
    pub wild: Vec<String>,
    /// Its tops on the terrain's (`sand`, `stone`, `snow`, `dirt`), by the
    /// mockup's texture; a top it does not name keeps the planet's. Where a
    /// template names none, [`record::Top`]'s own rule holds.
    #[serde(default)]
    pub tops: BTreeMap<String, String>,
    /// Its fires (slices 4e and 4g): braziers, torches and fire pits, each
    /// a lamp in the first air layer over where it stands (task 5.2).
    #[serde(default)]
    pub fires: Vec<Fire>,
    /// Its outside stairs (slice 4e): the sandstone houses' flights up to
    /// their roofs.
    #[serde(default)]
    pub stairs: Vec<OutsideStair>,
    /// What its masonry is made of (slice 4g), where it is not the walled
    /// town's rubble.
    #[serde(default)]
    pub masonry_material: Option<MasonryMaterial>,
    /// Its frozen lake (slice 4g): the cells laid a layer under the shore,
    /// floored with ice at `top_m` over the template's 0 m.
    #[serde(default)]
    pub frozen: Option<Frozen>,
}

/// What a town's masonry is made of (slice 4g): its walls and merlons, and
/// its walk's top. The walled town's is rubble under flagstones, with
/// stone under a gate's vault; the tundra's is ice under snow, ice under
/// its gate too.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct MasonryMaterial {
    pub wall: String,
    pub top: String,
}

impl Default for MasonryMaterial {
    fn default() -> Self {
        Self {
            wall: "rubble".into(),
            top: "flag".into(),
        }
    }
}

impl MasonryMaterial {
    /// Under a gate's vault: the walled town's dressed stone, or the wall's
    /// own.
    pub fn vault(&self) -> &str {
        if self.wall == "rubble" {
            "stone"
        } else {
            &self.wall
        }
    }
}

/// A frozen lake (slice 4g): its cells, and its ice's top in the mockup's
/// metres.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Frozen {
    pub top_m: f32,
    pub cells: Vec<[i32; 2]>,
}

/// A fire a town burns (slices 4e and 4g), where the mockup stands it:
/// `(x, z)` in its metres, `y` its height there.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Fire {
    /// `brazier`, `torch` or `fire` (a fire pit).
    pub kind: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Fire {
    /// The lamp it is in the voxel field: a brazier's for a brazier and a
    /// fire pit, a torch's for a torch.
    pub fn material(&self) -> crate::terrain::Material {
        match self.kind.as_str() {
            "torch" => crate::terrain::Material::Torch,
            _ => crate::terrain::Material::Brazier,
        }
    }
}

/// A straight flight of solid steps outside (slice 4e), the mockup's
/// `stairRun`: from its foot `from` up to its head `to`, `[x, y, z]` in the
/// mockup's metres, `width_m` wide, its steps of `material`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct OutsideStair {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub width_m: f32,
    pub material: String,
}

/// The harbour's cog (`sail-the-cog` design 6, step 1): moored with its
/// middle at `(x, z)` in the mockup's metres, its bow along `heading` (from
/// `x` toward `z`), its gangway on `gang_side` (+1 or -1 across it), and
/// its gangplank up from the pier to its deck.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Cog {
    pub x: f32,
    pub z: f32,
    pub heading: f32,
    pub gang_side: i32,
    pub gangplank: Pier,
}

/// A thing a town stands about (task 4.2c), where the mockup puts it:
/// `(x, z)` in its metres, `y` the height it stands at there (the game
/// stands it on what is under it), and `angle` its turn from `x` toward
/// `z`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Dress {
    /// A market stall: posts, a counter, a sloped awning of `cloth` and its
    /// goods on the counter.
    Stall {
        x: f32,
        y: f32,
        z: f32,
        cloth: String,
        goods: Vec<Goods>,
    },
    /// Nets hung to dry on three posts and a bar.
    NetRack {
        x: f32,
        y: f32,
        z: f32,
        angle: f32,
    },
    /// Fish hung to dry on two posts and a bar.
    FishRack {
        x: f32,
        y: f32,
        z: f32,
        angle: f32,
    },
    /// A lobster pot, `lift_m` over its pile's foot at `y`.
    Pot {
        x: f32,
        y: f32,
        z: f32,
        angle: f32,
        lift_m: f32,
    },
    Crate {
        x: f32,
        y: f32,
        z: f32,
        angle: f32,
        side_m: f32,
    },
    Barrel {
        x: f32,
        y: f32,
        z: f32,
    },
    Bollard {
        x: f32,
        y: f32,
        z: f32,
        radius_m: f32,
        height_m: f32,
    },
    /// An oar stood on end.
    Oar {
        x: f32,
        y: f32,
        z: f32,
    },
    /// A boat on land: keel up on trestles where `beached`, else on its
    /// keel. `boat` is the mockup's kind (`rowboat`, `sail`, `canoe`).
    Boat {
        boat: String,
        x: f32,
        y: f32,
        z: f32,
        angle: f32,
        beached: bool,
    },
    /// A cactus (slice 4e), `scale` times the mockup's 2.4 m one.
    Cactus {
        x: f32,
        y: f32,
        z: f32,
        scale: f32,
    },
}

impl Dress {
    /// Where it stands in the mockup's metres, `(x, z)`.
    pub fn at(&self) -> (f32, f32) {
        match *self {
            Dress::Stall { x, z, .. }
            | Dress::NetRack { x, z, .. }
            | Dress::FishRack { x, z, .. }
            | Dress::Pot { x, z, .. }
            | Dress::Crate { x, z, .. }
            | Dress::Barrel { x, z, .. }
            | Dress::Bollard { x, z, .. }
            | Dress::Oar { x, z, .. }
            | Dress::Boat { x, z, .. }
            | Dress::Cactus { x, z, .. } => (x, z),
        }
    }

    /// Its height in the mockup, metres over the template's 0 m.
    pub fn y(&self) -> f32 {
        match *self {
            Dress::Stall { y, .. }
            | Dress::NetRack { y, .. }
            | Dress::FishRack { y, .. }
            | Dress::Pot { y, .. }
            | Dress::Crate { y, .. }
            | Dress::Barrel { y, .. }
            | Dress::Bollard { y, .. }
            | Dress::Oar { y, .. }
            | Dress::Boat { y, .. }
            | Dress::Cactus { y, .. } => y,
        }
    }
}

/// A box of goods on a stall's counter, of `material`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Goods {
    pub material: String,
    pub x: f32,
    pub z: f32,
}

/// The harbour's shipyard (task 4.2c): a hull in frame on its keel blocks
/// at `(x, z)`, along `x`, its slip down into the water, and a stack of
/// planks at `planks` (`[x, y, z]`), all in the mockup's metres.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Shipyard {
    pub x: f32,
    pub z: f32,
    pub slip: Pier,
    pub planks: [f32; 3],
}

/// A boat on the water (task 4.2b): the mockup's kind (`rowboat`, `sail`,
/// `canoe`), where it lies in the mockup's metres, and its heading, the bow
/// along `(cos, sin)` in `(x, z)`.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Boat {
    pub kind: String,
    pub x: f32,
    pub z: f32,
    pub heading: f32,
}

/// A pier (slice 4d): a deck of planks `width_m` wide from `from` to `to`,
/// `[x, y, z]` in the mockup's metres (x east, z south, y over the sea), on
/// piles down to the ground under it.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Pier {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub width_m: f32,
}

/// A round stone tower standing on the ground at `(x, z)`, the mockup's
/// metres: the harbour's light.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct RoundTower {
    pub x: f32,
    pub z: f32,
    pub radius_m: f32,
    /// Its foot, layers over the template's 0 m.
    pub base_m: i32,
    /// Its top, metres over the template's 0 m.
    pub top_m: f32,
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
    /// A stair tower's or a keep's newel (slice 4c), where it is not a
    /// house's: its entry, how high it climbs and its walls rise, and its
    /// ways out.
    #[serde(default)]
    pub newel: Option<NewelDef>,
    /// A flat roof that is walked on, merlons on its outer edges: the keep's.
    #[serde(default)]
    pub parapet: bool,
    /// `[c, r, d]`: an outer edge with no wall (slice 4d), a boathouse's
    /// seaward side, the mockup's `skipWall`.
    #[serde(default)]
    pub open: Vec<[i32; 3]>,
    /// A walked flat roof's parapet wall, but at these `[c, r, d]` edges
    /// (slice 4e), where an outside stair comes up: the mockup's
    /// `flatRoof` with its `parapetGaps`. None for a roof not walked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parapet_gaps: Option<Vec<[i32; 3]>>,
    /// `[c, r, d, storey]`: doorways with no leaf (slice 4e), the mockup's
    /// open doors.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub archways: Vec<[i32; 4]>,
    /// A house on piles (slice 4d): the harbour's fish huts.
    #[serde(default)]
    pub stilts: Option<Stilts>,
}

/// A house on stilts (slice 4d), the mockup's `stiltHouse`: its floor on
/// piles down to the ground under it, a deck on the same piles at its door,
/// and an open stair from the deck down to where it lands.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Stilts {
    /// The deck's cells.
    pub deck: Vec<[i32; 2]>,
    /// `[c, r, d]`: the deck edge the porch stair goes down from.
    pub porch: [i32; 3],
    /// Where the stair lands, metres over the template's 0 m.
    pub foot_m: f32,
}

/// A newel that is not a house's (slice 4c): the mockup's `newelStair` as a
/// stair tower or the keep calls it. Heights are metres over the building's
/// ground floor.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct NewelDef {
    /// The edge it climbs from.
    pub entry: u8,
    /// How high it climbs.
    pub top_m: f32,
    /// How high its own walls rise; over a flat roof, they are a turret.
    pub wall_top_m: f32,
    /// Each way out, `[edge, metres]`: a doorway at that height in that
    /// edge's wall.
    #[serde(default)]
    pub exits: Vec<(u8, f32)>,
    /// How high the spire over it rises (slice 4g): the ice keep's and its
    /// towers' 5 m. None for the walled town's, a tower's 2.6 m and the
    /// keep's turret's 2.4 m.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spire_m: Option<f32>,
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
        // A template that names its wild areas builds every other cell
        // (slices 4e and 4g); one that does not, every top but grass and
        // sand (slice 1).
        let built = |g: &&GroundCell| {
            if self.wild.is_empty() {
                g.top != "grass" && g.top != "sand"
            } else {
                !self.wild.contains(&g.area)
            }
        };
        // There, the cells its fires, lanterns and dressing stand on are
        // built too: the tundra's camp fire stands on the open tundra.
        let things: Vec<[i32; 2]> = if self.wild.is_empty() {
            Vec::new()
        } else {
            self.fires
                .iter()
                .map(|f| (f.x, f.z))
                .chain(self.lanterns.iter().map(|l| (l[0], l[2])))
                .chain(self.dressing.iter().map(Dress::at))
                .map(|(x, z)| {
                    let (c, r) = sea::cell_at(x, z, self.grid.cell_m);
                    [c, r]
                })
                .collect()
        };
        let mut cells: Vec<[i32; 2]> = self
            .buildings
            .iter()
            .flat_map(|b| b.cells.iter().copied())
            .chain(self.ground.iter().filter(built).map(|g| [g.c, g.r]))
            .chain(things)
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
    /// A cap over its cells and a half-ellipsoid on it (slice 4e), the
    /// mockup's `domeCap`: the adobe houses' and the caravan hall's.
    Dome,
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
