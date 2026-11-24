//! The town tree in the game (`town-missions-tree`): its chain loaded and
//! resolved, the frame's happenings counted into it by the starter tree's
//! own function, its coins paid, and its record kept.
//!
//! The rules are `pbd_core::missions` and `pbd_core::missions::town`; this
//! module only feeds them. The tab and the HUD line are
//! `desktop/mission_tree.rs` and `desktop/missions.rs`.
//!
//! - **Opening** (design 1): the first `reach town` writes the record, the
//!   game day with it. An old world that shows a town visit (a purse,
//!   goodwill, a job, a lost villager, "Find a town" done) opens the tree as
//!   it is opened, at its root, with nothing paid.
//! - **What it counts**: the messages the starter tree counts
//!   ([`Happenings`]) and what the town raised that the jobs counted already
//!   (a talk, a deal, a job handed in, a parcel, a goodwill step).
//! - **Coins**: a count that finishes missions writes the purse, holding
//!   their coins and their paid marks, then the record, in one store.

use crate::config::{FaunaConfig, HusbandryConfig, Recipes};
use crate::economy::Economy;
use crate::hotbar::Hotbar;
use crate::missions::{Happenings, Missions, catalog_of, held, load_tree};
use crate::saves::WorldSave;
use bevy::prelude::*;
use pbd_core::economy::coins::{PURSE_ID, PURSE_KIND, PURSE_SCHEMA};
use pbd_core::missions::town::{
    TOWN_MISSIONS_ID, TOWN_MISSIONS_KIND, TOWN_MISSIONS_SCHEMA, TownRecord, been_to_town, pay,
};
use pbd_core::missions::{Chain, Happened, Kind, Objective, Progress};
use pbd_core::records::{Author, Record as StoredRecord};
use pbd_core::terrain::Material;

/// The town tree, resolved against this build's tables at startup.
#[derive(Resource, Clone, Debug, Default)]
pub struct TownChain(pub Chain);

/// The town tree's record in the open world, and what the HUD says.
#[derive(Resource, Clone, Debug, Default)]
pub struct TownMissions {
    /// `None` until the tree opens.
    pub record: Option<TownRecord>,
    /// The mission just finished, the coins it paid, and the seconds its
    /// line has left.
    pub notice: Option<(String, u64, f32)>,
    /// Which opening of a world was read: [`WorldSave::serial`].
    joined: Option<u64>,
    /// The focus was picked on the tree and is to be stored.
    pub refocused: bool,
    /// The tree has just opened: the desktop shows its toast and lowers
    /// this (`town-missions-tree` design 12).
    pub opened_toast: bool,
}

impl TownMissions {
    /// The tree's progress, once it is open.
    pub fn progress(&self) -> Option<&Progress> {
        self.record.as_ref().map(|r| &r.progress)
    }

    pub fn progress_mut(&mut self) -> Option<&mut Progress> {
        self.record.as_mut().map(|r| &mut r.progress)
    }

    /// Whether the tree is open in this world.
    pub fn is_open(&self) -> bool {
        self.record.is_some()
    }
}

/// How long a finished town mission's line stays up, seconds.
pub const NOTICE_S: f32 = 4.0;

/// The record a save holds, if the tree has opened in it.
pub fn record_of(save: &WorldSave) -> Option<TownRecord> {
    save.records
        .get(TOWN_MISSIONS_KIND, TOWN_MISSIONS_ID)
        .and_then(|r| r.read::<TownRecord>())
}

/// Read and resolve `town_missions.ron`.
pub fn load_town_chain(recipes: &Recipes, fauna: &FaunaConfig, land: &HusbandryConfig) -> Chain {
    load_tree(
        "town_missions.ron",
        "town_mission",
        &catalog_of(recipes, fauna, land),
    )
}

/// Everything the counting reads and writes.
#[derive(bevy::ecs::system::SystemParam)]
pub struct TownCounting<'w> {
    chain: Res<'w, TownChain>,
    starter: Res<'w, Missions>,
    town: ResMut<'w, TownMissions>,
    eco: Option<ResMut<'w, Economy>>,
    save: Option<ResMut<'w, WorldSave>>,
    heard: Res<'w, Happenings>,
    sun: Option<Res<'w, crate::sky::Sun>>,
    time: Res<'w, Time>,
    hotbar: Option<Res<'w, Hotbar>>,
    tools: Option<Res<'w, crate::fish::ToolSlot>>,
}

/// Open the tree, count this frame's happenings into it, pay what they
/// finish, and store the record when it moved. After the town jobs'
/// counting, which reads the world's purse first.
pub fn count_town(mut c: TownCounting) {
    if c.town.notice.is_some() {
        let dt = c.time.delta_secs();
        let left = c.town.notice.as_ref().map_or(0.0, |n| n.2) - dt;
        if left <= 0.0 {
            c.town.notice = None;
        } else if let Some(n) = c.town.bypass_change_detection().notice.as_mut() {
            n.2 = left;
        }
    }
    let (Some(mut save), Some(mut eco)) = (c.save, c.eco) else {
        return;
    };
    let raised = if eco.raised.is_empty() {
        Vec::new()
    } else {
        std::mem::take(&mut eco.raised)
    };
    let mut events: Vec<Happened> = c.heard.0.clone();
    events.extend(raised);
    let joining = c.town.joined != Some(save.serial());
    if events.is_empty() && !joining && !c.town.refocused {
        return;
    }
    let day = c.sun.as_deref().map_or(0, |s| s.clock.day());
    let town = &mut *c.town;
    let chain = &c.chain.0;
    let mut changed = std::mem::take(&mut town.refocused);
    let mut opened = false;
    if joining {
        town.record = record_of(&save);
        town.notice = None;
        town.joined = Some(save.serial());
        if town.record.is_none() && been_to_town(&save.records, &c.starter.progress) {
            info!("town missions: opened on a world that has been to a town");
            opened = true;
        }
    }
    let reached = events
        .iter()
        .any(|e| e.kind == Kind::Reach && e.what == "town");
    if town.record.is_none() && reached {
        info!("town missions: opened on reaching a town");
        opened = true;
    }
    if opened {
        town.opened_toast = true;
        town.record = Some(TownRecord {
            opened: day,
            progress: Progress::default(),
        });
        changed = true;
    }
    let Some(record) = town.record.as_mut() else {
        return;
    };
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
    let mut done = Vec::new();
    if joining || opened {
        let step = record.progress.settle(chain, &state);
        changed |= step.changed;
        done.extend(step.done);
    }
    for happened in &events {
        let step = record.progress.record(chain, happened, &state);
        changed |= step.changed;
        done.extend(step.done);
    }
    let mut purse = eco.purse.clone();
    let coins = pay(chain, &done, &mut purse);
    for id in &done {
        record.progress.focus_past(chain, id);
        let m = chain.get(id);
        let title = m.map_or(id.as_str(), |m| m.title.as_str());
        info!("town mission done: {id} ({title})");
        let paid = if purse.paid.contains(&pbd_core::missions::town::paid_key(id)) {
            m.map_or(0, |m| u64::from(m.coins))
        } else {
            0
        };
        town.notice = Some((title.to_string(), paid, NOTICE_S));
    }
    if !changed {
        return;
    }
    // The purse first: its line marks the coins paid, so a torn write after
    // it finishes the mission again on the next count and pays nothing.
    let mut lines = Vec::new();
    let paying = purse != eco.purse;
    if paying {
        lines.push(StoredRecord::of(PURSE_KIND, PURSE_ID, PURSE_SCHEMA, &purse));
    }
    lines.push(StoredRecord::of(
        TOWN_MISSIONS_KIND,
        TOWN_MISSIONS_ID,
        TOWN_MISSIONS_SCHEMA,
        &*record,
    ));
    if save.store(&Author::Player(0), lines).is_some() {
        save.note_record_kinds(&[
            (TOWN_MISSIONS_KIND, TOWN_MISSIONS_SCHEMA),
            (PURSE_KIND, PURSE_SCHEMA),
        ]);
        if paying {
            info!("town missions paid {coins}: purse {}", purse.coins);
            eco.purse = purse;
            // The coins, beside the coin count.
            eco.notice = Some((
                crate::ui_kit::tr_with("missions.coins", &[("count", &coins)]),
                crate::economy::NOTICE_S,
            ));
        }
    } else {
        warn!("town missions not saved: the save writer has failed");
    }
}

/// A building piece's name as a `build` objective counts it: its class, a
/// wall with a door in it a `door`.
pub fn build_name(shape: &pbd_core::building::Shape) -> &'static str {
    use pbd_core::building::{Opening, Shape};
    match shape {
        Shape::Wall {
            opening: Opening::Door,
            ..
        } => "door",
        Shape::Wall {
            opening: Opening::Window,
            ..
        } => "window",
        other => other.class().id(),
    }
}

/// Load the town tree, unless one is already given (a test's).
fn start(
    mut commands: Commands,
    given: Option<Res<TownChain>>,
    recipes: Res<Recipes>,
    fauna: Res<FaunaConfig>,
    land: Res<HusbandryConfig>,
) {
    if given.is_none() {
        commands.insert_resource(TownChain(load_town_chain(&recipes, &fauna, &land)));
    }
}

pub struct TownMissionsPlugin;

impl Plugin for TownMissionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TownMissions>()
            .add_systems(PreStartup, start)
            .add_systems(
                PostUpdate,
                count_town
                    .after(crate::missions::MissionsSet)
                    .after(crate::economy::count_jobs)
                    .run_if(crate::world_open),
            );
    }
}

#[cfg(test)]
mod tests;
