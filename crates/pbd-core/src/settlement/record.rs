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
use super::pieces::{BuildingSolids, Meshes, cut_building};
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
    pub buildings: Vec<Building>,
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
    let cells = footprint
        .iter()
        .filter_map(|&(c, r)| {
            let at = charted.cells.get(&(c, r))?;
            let top = dirt.contains(&(c, r)).then_some(Top::Dirt);
            Some(TownCell(c, r, patch.keys[at.cell], at.d0 as u8, top))
        })
        .collect();
    // The mockup's datum: the level most of its village stands on.
    let mut levels: Vec<i32> = template
        .ground
        .iter()
        .filter(|g| built.contains(&(g.c, g.r)))
        .map(|g| g.h)
        .collect();
    levels.sort();
    let datum = levels.get(levels.len() / 2).copied().unwrap_or(0);
    Ok(Town {
        site,
        template: template.scene.clone(),
        layout: LAYOUT_VERSION,
        terrace: terrace as i32,
        anchor,
        cells,
        buildings: template
            .buildings
            .iter()
            .map(|b| Building::laid(b, datum))
            .collect(),
    })
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
    let chart = chart_of(town, patch)?;
    let terrace = town.terrace as f32;
    let footprint: Vec<(usize, Option<Material>)> = town
        .cells
        .iter()
        .map(|&TownCell(c, r, _, _, top)| (chart.cells[&(c, r)].cell, top.map(Top::material)))
        .collect();
    let ground = TownGround::new(patch, radius_m, &footprint, terrace, natural);
    let mut meshes = Meshes::new();
    let mut solids = Vec::new();
    let mut rooms = Vec::new();
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
        solids.push(cut);
    }
    Ok(Built {
        chart,
        ground,
        meshes,
        solids,
        rooms,
    })
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
    records.push(Record::of(
        SETTLEMENT_RECORD,
        u64::from(town.site),
        RECORD_SCHEMA,
        &SettlementBody {
            template: town.template.clone(),
            layout: town.layout,
            terrace: town.terrace,
            anchor: town.anchor,
            cells: town.cells.clone(),
            buildings: ids,
        },
    ));
    records
}

/// What a save holds of a site's town.
#[derive(Clone, Debug, PartialEq)]
pub enum Stored {
    /// No settlement record: the town has not been laid.
    None,
    Town(Town),
    /// A settlement record this build cannot read whole: a schema it does
    /// not know, a body that does not parse, or a building it names
    /// missing. The town is not built, and nothing is written over it.
    Damaged(String),
}

/// A site's town as its save holds it.
pub fn from_records(records: &Records, site: u32) -> Stored {
    let Some(record) = records.get(SETTLEMENT_RECORD, u64::from(site)) else {
        return Stored::None;
    };
    if record.schema != RECORD_SCHEMA {
        return Stored::Damaged(format!(
            "settlement {site} is schema {}, and this build reads {RECORD_SCHEMA}",
            record.schema
        ));
    }
    let Some(body) = record.read::<SettlementBody>() else {
        return Stored::Damaged(format!("settlement {site} does not read"));
    };
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
