//! A town as its world's save stores it (`cities-in-the-world` decision 8,
//! slice 3a).
//!
//! A town is laid out ONCE, from its template, when its world first needs
//! it: [`lay`] charts the template onto the real cells, fixes the terrace,
//! and takes each building as the template has it. The result, a [`Town`],
//! is written into the save as records and built into the world from those
//! records ever after ([`build`]), so a revised template never moves a house
//! in a world that has one.
//!
//! What is stored is what the town was laid as: the cells it stands on (by
//! exact key, with the side of each that is the layout's direction 0), the
//! terrace, the lanes' tops, and each building's definition. What is
//! derived is everything the rules make of that: the pieces, solids and
//! meshes, cut afresh each time so a fix to a sill reaches every town, and
//! the ground's margin, which is generation and pinned as a generator is.

use super::chart::{Chart, Charted, Patch, chart};
use super::ground::TownGround;
use super::pieces::{BuildingSolids, Meshes, RoomLight, cut_building, cut_masonry};
use super::{BuildingDef, Kits, Template, neighbour};
use crate::records::{Record, Records};
use crate::terrain::Material;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A town's record kind (`world-persistence` decision 11); its id is its
/// site's.
pub const SETTLEMENT_RECORD: &str = "settlement";
/// A building's record kind; its id is [`building_id`].
pub const BUILDING_RECORD: &str = "building";
/// The schema both are written in.
pub const RECORD_SCHEMA: u32 = 1;
/// The settlement schema of a town on several levels (slice 4b): schema 1
/// and a level for each footprint cell. A town on one level is still
/// written in schema 1, so every village's records are as they were, and a
/// build that reads only schema 1 refuses a terraced town rather than laying
/// it flat.
pub const TERRACED_SCHEMA: u32 = 2;
/// The laying-out rules' version, stored for people to read: a town is
/// never rebuilt by it.
pub const LAYOUT_VERSION: u32 = 1;
/// Rings of layout cells round the built ones that are terraced with them,
/// so the yards between the houses are level too.
pub const YARD_RINGS: usize = 2;

/// A building's record id: its site's id over its number in the town.
pub fn building_id(site: u32, n: u32) -> u64 {
    (u64::from(site) << 16) | u64::from(n & 0xffff)
}

/// The top a town gives a footprint cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum Top {
    /// A lane, or the ground under a building's floor.
    Dirt,
}

impl Top {
    pub fn material(self) -> Material {
        match self {
            Top::Dirt => Material::Dirt,
        }
    }
}

/// One cell of a town's footprint: its layout cell `(c, r)`, its exact
/// cell key, which of its sides is the layout's direction 0, and the top
/// it takes. A tuple, so the record carries no field names per cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct TownCell(pub i32, pub i32, pub u32, pub u8, pub Option<Top>);

/// A building's standing. Only [`BuildingState::Standing`] is generated;
/// the others are set by the world's processes (`world-persistence`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum BuildingState {
    #[default]
    Standing,
    Abandoned,
    Ruined,
}

/// A building as it was laid: the template's building, in the town's
/// layout cells, with its ground floor in layers over the terrace.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Building {
    pub name: String,
    pub kit: String,
    pub cells: Vec<[i32; 2]>,
    /// The ground floor, layers over the town's terrace.
    pub floor: i32,
    pub storeys: u32,
    pub tall: u32,
    pub doors: Vec<[i32; 4]>,
    pub windows: Vec<[i32; 4]>,
    pub roof: String,
    pub pitch: f32,
    pub chimney: Option<[i32; 2]>,
    pub stair_cells: Vec<[i32; 2]>,
    #[serde(default)]
    pub state: BuildingState,
    /// A stair tower's or the keep's newel (slice 4c). Written only where
    /// there is one, so every house's record is as it was.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub newel: Option<super::NewelDef>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub parapet: bool,
    /// A boathouse's open edges (slice 4d), written only where there are
    /// some.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub open: Vec<[i32; 3]>,
    /// A fish hut's stilts, deck and porch stair (slice 4d), its foot in
    /// metres over the town's terrace.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stilts: Option<super::Stilts>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

impl Building {
    fn laid(def: &BuildingDef, datum: i32) -> Self {
        Self {
            name: def.name.clone(),
            kit: def.kit.clone(),
            cells: def.cells.clone(),
            floor: def.base - datum,
            storeys: def.storeys,
            tall: def.tall,
            doors: def.doors.clone(),
            windows: def.windows.clone(),
            roof: def.roof.clone(),
            pitch: def.pitch,
            chimney: def.chimney,
            stair_cells: def.stair_cells.clone(),
            state: BuildingState::Standing,
            newel: def.newel.clone(),
            parapet: def.parapet,
            open: def.open.clone(),
            stilts: def.stilts.as_ref().map(|s| super::Stilts {
                foot_m: s.foot_m - datum as f32,
                ..s.clone()
            }),
        }
    }

    /// The definition the cutter takes, standing on the terrace.
    pub fn def(&self) -> BuildingDef {
        BuildingDef {
            name: self.name.clone(),
            kit: self.kit.clone(),
            cells: self.cells.clone(),
            base: self.floor,
            storeys: self.storeys,
            tall: self.tall,
            doors: self.doors.clone(),
            windows: self.windows.clone(),
            roof: self.roof.clone(),
            pitch: self.pitch,
            chimney: self.chimney,
            stair_cells: self.stair_cells.clone(),
            newel: self.newel.clone(),
            parapet: self.parapet,
            open: self.open.clone(),
            stilts: self.stilts.clone(),
        }
    }
}

/// A town as it was laid.
#[derive(Clone, Debug, PartialEq)]
pub struct Town {
    pub site: u32,
    /// The template it was laid from, and the rules' version: for people.
    pub template: String,
    pub layout: u32,
    /// The terrace, whole metres over the radius.
    pub terrace: i32,
    /// The layout cell on the site's anchor.
    pub anchor: (i32, i32),
    /// Every footprint cell, by layout cell.
    pub cells: Vec<TownCell>,
    /// Each footprint cell's level, layers over the terrace, in the order of
    /// `cells`; empty where the town stands on one level (slice 4b).
    pub levels: Vec<i8>,
    pub buildings: Vec<Building>,
}

impl Town {
    /// A footprint cell's terrace, metres over the radius.
    pub fn terrace_of(&self, cell: usize) -> f32 {
        self.terrace as f32 + f32::from(self.levels.get(cell).copied().unwrap_or(0))
    }
}

/// Which way a site's layout turns, 0..6 sides from the anchor cell's side
/// nearest east (decision 4, slice 4a): a mix of the site's id, so two
/// villages seldom face the same way and a site always faces the same one.
/// The home village does not turn: it was laid facing east before sites
/// turned, and a new world lays it as every old one has it.
pub fn turn(site: u32) -> usize {
    let mut z = u64::from(site).wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    ((z ^ (z >> 31)) % 6) as usize
}

/// The layout cell a template is anchored by: the middle of what it builds
/// on.
pub fn template_anchor(template: &Template) -> (i32, i32) {
    let built = template.built_cells();
    let (mut lo, mut hi) = ([i32::MAX; 2], [i32::MIN; 2]);
    for [c, r] in &built {
        lo = [lo[0].min(*c), lo[1].min(*r)];
        hi = [hi[0].max(*c), hi[1].max(*r)];
    }
    ((lo[0] + hi[0]) / 2, (lo[1] + hi[1]) / 2)
}

/// Lay a template at a site: its anchor cell on patch cell `at`, the
/// layout's direction 0 on that cell's side `d0`, charted through the
/// neighbour tables (decision 2). The footprint is what is built on and
/// [`YARD_RINGS`] of yard round it; the terrace is the median natural layer
/// under what is built, so it cuts as much as it fills. `natural` is the
/// natural ground's height at a direction, metres over the radius.
pub fn lay(
    template: &Template,
    site: u32,
    patch: &Patch,
    at: usize,
    d0: usize,
    natural: impl Fn(Vec3) -> f32,
) -> Result<Town, String> {
    let wanted: BTreeSet<(i32, i32)> = template.ground.iter().map(|g| (g.c, g.r)).collect();
    let anchor = template_anchor(template);
    let charted = chart(patch, anchor, at, d0, &wanted)?;
    let built: BTreeSet<(i32, i32)> = template
        .built_cells()
        .into_iter()
        .map(|[c, r]| (c, r))
        .collect();
    let mut footprint = built.clone();
    let mut ring = built.clone();
    for _ in 0..YARD_RINGS {
        let mut next = BTreeSet::new();
        for &(c, r) in &ring {
            for d in 0..6 {
                let n = neighbour(c, r, d);
                if charted.cells.contains_key(&n) && footprint.insert(n) {
                    next.insert(n);
                }
            }
        }
        ring = next;
    }
    let mut heights: Vec<f32> = built
        .iter()
        .filter_map(|&(c, r)| charted.cell(c, r))
        .map(|i| natural(patch.cells[i].direction).floor())
        .collect();
    heights.sort_by(f32::total_cmp);
    let terrace = *heights.get(heights.len() / 2).ok_or("nothing built")?;
    let dirt: BTreeSet<(i32, i32)> = template
        .ground
        .iter()
        .filter(|g| g.top != "grass" && g.top != "sand")
        .map(|g| (g.c, g.r))
        .collect();
    let cells: Vec<TownCell> = footprint
        .iter()
        .filter_map(|&(c, r)| {
            let at = charted.cells.get(&(c, r))?;
            let top = dirt.contains(&(c, r)).then_some(Top::Dirt);
            Some(TownCell(c, r, patch.keys[at.cell], at.d0 as u8, top))
        })
        .collect();
    let datum = datum(template);
    let levels = if template.terraced {
        levels_of(template, &built, &cells, datum)
    } else {
        Vec::new()
    };
    Ok(Town {
        site,
        template: template.scene.clone(),
        layout: LAYOUT_VERSION,
        terrace: terrace as i32,
        anchor,
        cells,
        levels,
        buildings: template
            .buildings
            .iter()
            .map(|b| Building::laid(b, datum))
            .collect(),
    })
}

/// The mockup's datum: the level most of what a template builds on stands
/// at. A town's terrace is this level, and every height in the template is
/// over it.
pub fn datum(template: &Template) -> i32 {
    let built: BTreeSet<(i32, i32)> = template
        .built_cells()
        .into_iter()
        .map(|[c, r]| (c, r))
        .collect();
    let mut heights: Vec<i32> = template
        .ground
        .iter()
        .filter(|g| built.contains(&(g.c, g.r)))
        .map(|g| g.h)
        .collect();
    heights.sort();
    heights.get(heights.len() / 2).copied().unwrap_or(0)
}

/// Each footprint cell's level over the datum, for a town on several levels
/// (slice 4b): a built cell (a building's, a street's) at its own height, and
/// a yard cell level with the built cell nearest it, so a yard never follows
/// the mockup's ground past the town (the lake, the hillside) down or up.
fn levels_of(
    template: &Template,
    built: &BTreeSet<(i32, i32)>,
    cells: &[TownCell],
    datum: i32,
) -> Vec<i8> {
    let height: BTreeMap<(i32, i32), i32> =
        template.ground.iter().map(|g| ((g.c, g.r), g.h)).collect();
    let inside: BTreeSet<(i32, i32)> = cells.iter().map(|c| (c.0, c.1)).collect();
    let mut level: BTreeMap<(i32, i32), i32> = built
        .iter()
        .filter(|at| inside.contains(at))
        .map(|&at| (at, height.get(&at).copied().unwrap_or(datum) - datum))
        .collect();
    let mut ring: Vec<(i32, i32)> = level.keys().copied().collect();
    while !ring.is_empty() {
        let mut next = Vec::new();
        for &(c, r) in &ring {
            let l = level[&(c, r)];
            for d in 0..6 {
                let n = neighbour(c, r, d);
                if inside.contains(&n) && !level.contains_key(&n) {
                    level.insert(n, l);
                    next.push(n);
                }
            }
        }
        ring = next;
    }
    cells
        .iter()
        .map(|c| {
            level
                .get(&(c.0, c.1))
                .copied()
                .unwrap_or(0)
                .clamp(-100, 100) as i8
        })
        .collect()
}

/// A town's chart on a patch, from its stored cells: each key found in
/// the patch, with its stored side. The first cell the patch does not hold
/// is named.
pub fn chart_of(town: &Town, patch: &Patch) -> Result<Chart, String> {
    let by_key: BTreeMap<u32, usize> = patch
        .keys
        .iter()
        .enumerate()
        .map(|(i, &k)| (k, i))
        .collect();
    let mut cells = BTreeMap::new();
    for &TownCell(c, r, key, side, _) in &town.cells {
        let cell = *by_key
            .get(&key)
            .ok_or_else(|| format!("layout cell ({c}, {r}): cell {key} is not in the patch"))?;
        cells.insert(
            (c, r),
            Charted {
                cell,
                d0: usize::from(side),
            },
        );
    }
    Ok(Chart {
        anchor: town.anchor,
        cells,
    })
}

/// A town built from its definition.
pub struct Built {
    pub chart: Chart,
    pub ground: TownGround,
    pub meshes: Meshes,
    /// Each building's solids, for the walker.
    pub solids: Vec<BuildingSolids>,
    /// Each building's inside faces, by texture (`sun-shadows` decision 7),
    /// taken out of its solids.
    pub rooms: Vec<Meshes>,
    /// What burns in each building's rooms (decision 7a), taken out of its
    /// solids.
    pub lights: Vec<Vec<RoomLight>>,
}

/// A town's chart and ground from its definition, without cutting a piece:
/// the stored terrace and footprint, the margin eased to `natural`. What a
/// world installs for every town it holds, near or far (slice 4a).
pub fn ground_of(
    town: &Town,
    patch: &Patch,
    radius_m: f32,
    natural: impl Fn(Vec3) -> f32,
) -> Result<(Chart, TownGround), String> {
    let chart = chart_of(town, patch)?;
    let footprint: Vec<(usize, Option<Material>, f32)> = town
        .cells
        .iter()
        .enumerate()
        .map(|(i, &TownCell(c, r, _, _, top))| {
            (
                chart.cells[&(c, r)].cell,
                top.map(Top::material),
                town.terrace_of(i),
            )
        })
        .collect();
    let ground = TownGround::terraced(patch, radius_m, &footprint, natural);
    Ok((chart, ground))
}

/// Build a town from its definition: its chart from the stored cells, its
/// ground from the stored terrace and footprint (the margin eased to
/// `natural`), and each building cut from its cells' real corners with its
/// kit. `repeat_m` is how far a texture repeats.
pub fn build(
    town: &Town,
    patch: &Patch,
    kits: &Kits,
    repeat_m: &dyn Fn(&str) -> f32,
    radius_m: f32,
    natural: impl Fn(Vec3) -> f32,
) -> Result<Built, String> {
    let (chart, ground) = ground_of(town, patch, radius_m, natural)?;
    let terrace = town.terrace as f32;
    let mut meshes = Meshes::new();
    let mut solids = Vec::new();
    let mut rooms = Vec::new();
    let mut lights = Vec::new();
    for b in &town.buildings {
        let kit = kits
            .get(&b.kit)
            .ok_or_else(|| format!("{}: no kit {}", b.name, b.kit))?;
        let mut cut = cut_building(
            &mut meshes,
            repeat_m,
            patch,
            &chart,
            &b.def(),
            kit,
            radius_m,
            terrace + b.floor as f32,
        )?;
        rooms.push(std::mem::take(&mut cut.rooms));
        lights.push(std::mem::take(&mut cut.lights));
        solids.push(cut);
    }
    Ok(Built {
        chart,
        ground,
        meshes,
        solids,
        rooms,
        lights,
    })
}

/// Build a town from its definition and its template's masonry (slice 4c):
/// [`build`], and then each masonry cell of the template the town was laid
/// from, cut on the town's stored chart. The masonry is the template's, not
/// the town's record: a `v1` template is frozen, so it is the same wall in
/// every world. A masonry cell stands on its own cell's terrace and rises to
/// the template's top over the datum; a gate's vault starts as far over its
/// ground as the template has it. Masonry comes after the buildings, each
/// with no rooms and no lights.
pub fn build_town(
    town: &Town,
    template: Option<&Template>,
    patch: &Patch,
    kits: &Kits,
    repeat_m: &dyn Fn(&str) -> f32,
    radius_m: f32,
    natural: impl Fn(Vec3) -> f32,
) -> Result<Built, String> {
    let mut built = build(town, patch, kits, repeat_m, radius_m, natural)?;
    let Some(template) = template.filter(|t| !t.masonry.is_empty()) else {
        return Ok(built);
    };
    let datum = datum(template);
    let height: BTreeMap<(i32, i32), i32> =
        template.ground.iter().map(|g| ((g.c, g.r), g.h)).collect();
    let terrace: BTreeMap<(i32, i32), f32> = town
        .cells
        .iter()
        .enumerate()
        .map(|(i, c)| ((c.0, c.1), town.terrace_of(i)))
        .collect();
    let walls: BTreeSet<(i32, i32)> = template.masonry.iter().map(|m| (m.c, m.r)).collect();
    for m in &template.masonry {
        let Some(&ground) = terrace.get(&(m.c, m.r)) else {
            return Err(format!("masonry ({}, {}) is not in the town", m.c, m.r));
        };
        let h = height.get(&(m.c, m.r)).copied().unwrap_or(datum);
        let bottom = ground + (m.from - h).max(0) as f32;
        let top = town.terrace as f32 + (m.to - datum) as f32;
        let walled = |d: usize| walls.contains(&neighbour(m.c, m.r, d));
        let cut = cut_masonry(
            &mut built.meshes,
            repeat_m,
            patch,
            &built.chart,
            m,
            &walled,
            radius_m,
            ground,
            bottom,
            top,
        )?;
        built.rooms.push(Meshes::new());
        built.lights.push(Vec::new());
        built.solids.push(cut);
    }
    Ok(built)
}

/// The settlement's record body: everything of the town but its buildings,
/// which are records of their own, named here in order.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
struct SettlementBody {
    template: String,
    layout: u32,
    terrace: i32,
    anchor: (i32, i32),
    cells: Vec<TownCell>,
    /// Schema 2 only: each cell's level (slice 4b).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    levels: Vec<i8>,
    buildings: Vec<u64>,
}

/// A town as the records its save stores: a record per building, then the
/// settlement's, last, so a write torn by a crash leaves no settlement and
/// the town is laid again whole.
pub fn to_records(town: &Town) -> Vec<Record> {
    let ids: Vec<u64> = (0..town.buildings.len() as u32)
        .map(|n| building_id(town.site, n))
        .collect();
    let mut records: Vec<Record> = town
        .buildings
        .iter()
        .zip(&ids)
        .map(|(b, &id)| Record::of(BUILDING_RECORD, id, RECORD_SCHEMA, b))
        .collect();
    let levels = if town.levels.iter().any(|&l| l != 0) {
        town.levels.clone()
    } else {
        Vec::new()
    };
    let schema = if levels.is_empty() {
        RECORD_SCHEMA
    } else {
        TERRACED_SCHEMA
    };
    records.push(Record::of(
        SETTLEMENT_RECORD,
        u64::from(town.site),
        schema,
        &SettlementBody {
            template: town.template.clone(),
            layout: town.layout,
            terrace: town.terrace,
            anchor: town.anchor,
            cells: town.cells.clone(),
            levels,
            buildings: ids,
        },
    ));
    records
}

/// A site left unsettled (task 4.4): the player had changed its ground
/// before a town was laid there, so none ever is. Its id is its site's.
pub const UNSETTLED_RECORD: &str = "unsettled";

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
struct UnsettledBody {
    /// What the world found there, for people.
    why: String,
}

/// The record that leaves a site unsettled.
pub fn unsettled_record(site: u32, why: &str) -> Record {
    Record::of(
        UNSETTLED_RECORD,
        u64::from(site),
        RECORD_SCHEMA,
        &UnsettledBody { why: why.into() },
    )
}

/// What a save holds of a site's town.
#[derive(Clone, Debug, PartialEq)]
pub enum Stored {
    /// No settlement record: the town has not been laid.
    None,
    Town(Town),
    /// No town, for good: the site was unsettled when its town would have
    /// been laid (task 4.4).
    Unsettled,
    /// A settlement record this build cannot read whole: a schema it does
    /// not know, a body that does not parse, or a building it names
    /// missing. The town is not built, and nothing is written over it.
    Damaged(String),
}

/// A site's town as its save holds it.
pub fn from_records(records: &Records, site: u32) -> Stored {
    let Some(record) = records.get(SETTLEMENT_RECORD, u64::from(site)) else {
        return if records.get(UNSETTLED_RECORD, u64::from(site)).is_some() {
            Stored::Unsettled
        } else {
            Stored::None
        };
    };
    if record.schema != RECORD_SCHEMA && record.schema != TERRACED_SCHEMA {
        return Stored::Damaged(format!(
            "settlement {site} is schema {}, and this build reads {RECORD_SCHEMA} and {TERRACED_SCHEMA}",
            record.schema
        ));
    }
    let Some(body) = record.read::<SettlementBody>() else {
        return Stored::Damaged(format!("settlement {site} does not read"));
    };
    let terraced = record.schema == TERRACED_SCHEMA;
    // Schema 2 carries a level for every cell, and schema 1 none.
    if terraced == body.levels.is_empty() || (terraced && body.levels.len() != body.cells.len()) {
        return Stored::Damaged(format!(
            "settlement {site}: {} levels for {} cells in schema {}",
            body.levels.len(),
            body.cells.len(),
            record.schema
        ));
    }
    let mut buildings = Vec::with_capacity(body.buildings.len());
    for &id in &body.buildings {
        let Some(b) = records.get(BUILDING_RECORD, id) else {
            return Stored::Damaged(format!("settlement {site}: building {id} is missing"));
        };
        if b.schema != RECORD_SCHEMA {
            return Stored::Damaged(format!(
                "settlement {site}: building {id} is schema {}",
                b.schema
            ));
        }
        let Some(b) = b.read::<Building>() else {
            return Stored::Damaged(format!("settlement {site}: building {id} does not read"));
        };
        buildings.push(b);
    }
    Stored::Town(Town {
        site,
        template: body.template,
        layout: body.layout,
        terrace: body.terrace,
        anchor: body.anchor,
        cells: body.cells,
        levels: body.levels,
        buildings,
    })
}

/// A door's record kind (slice 2b): whether it stands open. No record is a
/// shut door. It is the player's, written when they open or shut it.
pub const DOOR_RECORD: &str = "door";

/// A door's record id: its building's record id over its number there.
pub fn door_id(building: u64, index: usize) -> u64 {
    building * 16 + (index as u64 & 15)
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
struct DoorBody {
    open: bool,
}

/// A door's state as its record.
pub fn door_record(id: u64, open: bool) -> Record {
    Record::of(DOOR_RECORD, id, RECORD_SCHEMA, &DoorBody { open })
}

/// Whether a save holds a door open. A record this build cannot read is a
/// shut door, as no record is.
pub fn door_open(records: &Records, id: u64) -> bool {
    records
        .get(DOOR_RECORD, id)
        .filter(|r| r.schema == RECORD_SCHEMA)
        .and_then(|r| r.read::<DoorBody>())
        .is_some_and(|b| b.open)
}
