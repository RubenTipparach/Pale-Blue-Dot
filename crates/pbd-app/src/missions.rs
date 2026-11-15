//! The starter missions in the game (`starter-missions` design 7-17): the
//! chain loaded and resolved, the messages every system raises turned into
//! counts, and the player's progress kept as a durable record.
//!
//! The rules are `pbd_core::missions`; this module only feeds them. The HUD
//! line and the Missions tab are `desktop/missions.rs`.

use crate::config::{FaunaConfig, HusbandryConfig, Recipes};
use crate::hotbar::{Gained, Hotbar, How, Planted};
use crate::husbandry::Deed;
use crate::kitchen::Cooked;
use crate::saves::WorldSave;
use bevy::prelude::*;
use pbd_core::inventory::Item;
use pbd_core::missions::{
    Catalog, Chain, Happened, Kind, MISSIONS_ID, MISSIONS_KIND, MISSIONS_SCHEMA, MissionFile,
    Objective, Progress, Want, item_code,
};
use pbd_core::records::{Author, Record as StoredRecord};
use pbd_core::terrain::Material;

/// Something the player did that no other message says (design 15): a block
/// dug or placed, the float on water, a fish landed, a cell tilled, a station
/// recipe collected (`recipe:<code>`). Raised on the frame the durable write
/// was accepted.
#[derive(Message, Clone, Debug, PartialEq, Eq)]
pub struct Did(pub Happened);

impl Did {
    pub fn new(kind: Kind, what: impl Into<String>, count: u32) -> Self {
        Self(Happened::new(kind, what, count))
    }

    /// A block, by its canonical name.
    pub fn block(kind: Kind, material: Material) -> Self {
        Self::new(kind, item_code(Item::Block(material)), 1)
    }
}

/// The chain, resolved against this build's tables at startup.
#[derive(Resource, Clone, Debug, Default)]
pub struct MissionChain(pub Chain);

/// The player's progress in the open world, and what the HUD says about it.
#[derive(Resource, Clone, Debug, Default)]
pub struct Missions {
    pub progress: Progress,
    /// The mission just finished and the seconds its "Done" line has left.
    pub notice: Option<(String, f32)>,
    /// Which opening of a world has joined the chain (design 4):
    /// [`WorldSave::serial`].
    joined: Option<u64>,
}

/// How long a finished mission's "Done" line stays up, seconds.
pub const NOTICE_S: f32 = 4.0;

/// The progress a save holds: none for a world before missions, or a new
/// one, which starts the chain.
pub fn progress_of(save: &WorldSave) -> Progress {
    save.records
        .get(MISSIONS_KIND, MISSIONS_ID)
        .and_then(|r| r.read::<Progress>())
        .unwrap_or_default()
}

/// Read and resolve `missions.ron` against the recipe book and the rosters,
/// logging each pending mission. A chain naming what this build cannot know
/// (and does not await) is refused, as every config is.
pub fn load_chain(recipes: &Recipes, fauna: &FaunaConfig, land: &HusbandryConfig) -> Chain {
    let path = crate::config::config_dir().join("missions.ron");
    let file: MissionFile = match std::fs::read_to_string(&path) {
        Ok(text) => ron::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display())),
        Err(_) => {
            warn!("{} not found; no missions", path.display());
            MissionFile::default()
        }
    };
    let fish = fauna.0.bodies.values().flatten().map(|s| s.name.as_str());
    let animals = land.0.land.values().flatten().map(|s| s.name.as_str());
    let catalog = Catalog::new(&recipes.0, fish, animals);
    let chain = file
        .resolve(&catalog)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    for (id, missing) in chain.pending() {
        info!("mission {id} is pending: no {missing:?} in this build");
    }
    chain
}

/// How many of an objective the world already holds (design 10): a tool
/// owned or an item carried for `craft` (a block carried and placed
/// together), and a block the world's edits hold for `place`. Nought for
/// every other kind, which counts only what happens.
pub fn held(
    objective: &Objective,
    slots: &pbd_core::inventory::Slots,
    owned: &[pbd_core::inventory::Tool],
    placed: &[Material],
) -> u32 {
    let want = |item: Item| objective.want.matches(objective.kind, &item_code(item));
    let placed_count = || placed.iter().filter(|m| want(Item::Block(**m))).count() as u32;
    match objective.kind {
        Kind::Craft => {
            if matches!(objective.want, Want::Any) {
                return 0;
            }
            let tools = owned.iter().filter(|t| want(Item::Tool(**t))).count() as u32;
            let carried: u32 = slots
                .iter()
                .flatten()
                .filter(|s| !matches!(s.item, Item::Fish(_)) && want(s.item))
                .map(|s| u32::from(s.count))
                .sum();
            tools + carried + placed_count()
        }
        Kind::Place if !matches!(objective.want, Want::Any) => placed_count(),
        _ => 0,
    }
}

/// Everything the counting reads and writes, in one parameter.
#[derive(bevy::ecs::system::SystemParam)]
pub struct Counting<'w, 's> {
    chain: Res<'w, MissionChain>,
    missions: ResMut<'w, Missions>,
    save: Option<ResMut<'w, WorldSave>>,
    hotbar: Option<Res<'w, Hotbar>>,
    tools: Option<Res<'w, crate::fish::ToolSlot>>,
    land: Res<'w, HusbandryConfig>,
    body: Option<Res<'w, crate::fish::Body>>,
    time: Res<'w, Time>,
    gained: MessageReader<'w, 's, Gained>,
    planted: MessageReader<'w, 's, Planted>,
    deeds: MessageReader<'w, 's, Deed>,
    cooked: MessageReader<'w, 's, Cooked>,
    did: MessageReader<'w, 's, Did>,
}

/// What the messages of this frame mean to the chain.
fn happenings(c: &mut Counting) -> Vec<Happened> {
    let body = c
        .body
        .as_ref()
        .map_or(pbd_core::fauna::HOME_BODY.to_string(), |b| b.0.clone());
    let animal = |species: u16| {
        c.land
            .0
            .roster(&body)
            .get(species as usize)
            .map_or("animal".to_string(), |s| s.name.to_lowercase())
    };
    let mut out = Vec::new();
    for gained in c.gained.read() {
        let what = item_code(gained.item);
        let n = u32::from(gained.count);
        let kinds: &[Kind] = match gained.how {
            How::Found => &[Kind::Find, Kind::Gather],
            How::Picked => &[Kind::Find],
            How::Harvested => &[Kind::Harvest],
            How::Collected => &[Kind::Collect],
            How::Made if what.ends_with("-ingot") => &[Kind::Craft, Kind::Smelt],
            How::Made => &[Kind::Craft],
        };
        out.extend(kinds.iter().map(|k| Happened::new(*k, what.clone(), n)));
    }
    for planted in c.planted.read() {
        out.push(Happened::new(
            Kind::Plant,
            planted.crop.row().seed.code(),
            1,
        ));
    }
    let deeds: Vec<Deed> = c.deeds.read().copied().collect();
    for deed in deeds {
        out.push(match deed {
            Deed::Tamed { species } => Happened::new(Kind::Tame, animal(species), 1),
            Deed::Penned { species } => Happened::new(Kind::Pen, animal(species), 1),
            Deed::PenBuilt { .. } => Happened::new(Kind::BuildPen, "pen", 1),
        });
    }
    for cooked in c.cooked.read() {
        out.push(Happened::new(
            Kind::Cook,
            cooked.recipe.clone(),
            u32::from(cooked.count),
        ));
    }
    out.extend(c.did.read().map(|d| d.0.clone()));
    out
}

/// Count this frame's happenings, join a newly opened world to the chain,
/// and store the record when it moved. After every system that raises one.
pub fn count(mut c: Counting) {
    let dt = c.time.delta_secs();
    if let Some((_, left)) = c.missions.notice.as_mut() {
        *left -= dt;
        if *left <= 0.0 {
            c.missions.notice = None;
        }
    }
    let happened = happenings(&mut c);
    let Some(mut save) = c.save else {
        return;
    };
    let joining = c.missions.joined != Some(save.serial());
    if happened.is_empty() && !joining {
        return;
    }
    let slots = c.hotbar.as_deref().map(|h| h.0.clone()).unwrap_or_default();
    let owned = c.tools.as_deref().map(|t| t.0.tools()).unwrap_or_default();
    let placed: Vec<Material> = save
        .edits
        .all()
        .into_iter()
        .map(|e| e.material)
        .filter(|m| *m != Material::Air)
        .collect();
    let state = |o: &Objective| held(o, &slots, &owned, &placed);
    let chain = &c.chain.0;
    let missions = &mut *c.missions;
    let mut done = Vec::new();
    let mut changed = false;
    // A world just opened (every load replaces the save): read its record
    // and join the chain, closing what the world already holds.
    if joining {
        missions.progress = progress_of(&save);
        missions.notice = None;
        missions.joined = Some(save.serial());
        let step = missions.progress.settle(chain, &state);
        for id in &step.done {
            info!("mission {id} is met by what the world holds");
        }
        changed |= step.changed;
    }
    for happening in &happened {
        let step = missions.progress.record(chain, happening, &state);
        changed |= step.changed;
        done.extend(step.done);
    }
    for id in &done {
        let title = chain.get(id).map_or(id.as_str(), |m| m.title.as_str());
        info!("mission done: {id} ({title})");
        missions.notice = Some((title.to_string(), NOTICE_S));
    }
    if changed {
        let record = StoredRecord::of(
            MISSIONS_KIND,
            MISSIONS_ID,
            MISSIONS_SCHEMA,
            &missions.progress,
        );
        if save.store(&Author::Player(0), vec![record]).is_some() {
            save.note_record_kinds(&[(MISSIONS_KIND, MISSIONS_SCHEMA)]);
        } else {
            warn!("missions not saved: the save writer has failed");
        }
    }
}

/// Load the chain, unless one is already given (a test's).
fn start(
    mut commands: Commands,
    given: Option<Res<MissionChain>>,
    recipes: Res<Recipes>,
    fauna: Res<FaunaConfig>,
    land: Res<HusbandryConfig>,
) {
    if given.is_none() {
        commands.insert_resource(MissionChain(load_chain(&recipes, &fauna, &land)));
    }
}

/// The systems that raise what the missions count, so the counting can run
/// after them.
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct MissionsSet;

pub struct MissionsPlugin;

impl Plugin for MissionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<Did>()
            .add_message::<Gained>()
            .add_message::<Planted>()
            .add_message::<Deed>()
            .add_message::<Cooked>()
            .init_resource::<Missions>()
            .add_systems(PreStartup, start)
            .add_systems(
                PostUpdate,
                count.in_set(MissionsSet).run_if(crate::world_open),
            );
    }
}

#[cfg(test)]
mod tests;
