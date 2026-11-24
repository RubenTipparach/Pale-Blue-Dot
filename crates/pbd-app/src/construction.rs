//! Construction sites in the world (`building-planner` design 5 to 9): the
//! planner's data read from the assets, the player's sites read back from
//! the save and charted onto their cells, and every change to a site or a
//! blueprint written through the save's durable path before anything is
//! drawn. The rules are the core's (`pbd_core::building::{plan, planner,
//! site, costs}`); the desktop draws the ghosts and strikes the pieces.

use crate::builds::{PATCH_REACH_M, patch_round};
use crate::saves::WorldSave;
use bevy::prelude::*;
use pbd_core::building::costs::{CostTable, Goods};
use pbd_core::building::plan::{Charted, chart_plan};
use pbd_core::building::planner::Tunables;
use pbd_core::building::presets::Presets;
use pbd_core::building::site::{
    BLUEPRINT_RECORD, BLUEPRINT_SCHEMA, BlueprintBody, SITE_RECORD, SITE_SCHEMA, SiteBody,
    blueprint_record, next_id, site_record, sites,
};
use pbd_core::inventory::Slots;
use pbd_core::records::Author;
use pbd_core::settlement::chart::Patch;
use std::collections::BTreeMap;
use std::sync::Arc;

/// The planner's tunables, cost table and presets
/// (`assets/config/building_planner.ron`, `building_costs.ron`,
/// `blueprints.ron`), checked.
#[derive(Resource, Clone)]
pub struct PlannerData {
    pub tun: Tunables,
    pub costs: CostTable,
    pub presets: Presets,
}

impl Default for PlannerData {
    fn default() -> Self {
        load()
    }
}

/// Read and check the planner's three files.
pub fn load() -> PlannerData {
    fn read<T: serde::de::DeserializeOwned>(name: &str) -> T {
        let path = crate::paths::assets_dir().join("config").join(name);
        let text =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        ron::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }
    let tun: Tunables = read("building_planner.ron");
    tun.validate()
        .unwrap_or_else(|e| panic!("building_planner.ron: {e}"));
    let costs: CostTable = read("building_costs.ron");
    costs
        .validate()
        .unwrap_or_else(|e| panic!("building_costs.ron: {e}"));
    PlannerData {
        tun,
        costs,
        presets: read("blueprints.ron"),
    }
}

/// A site as it stands: its record, and its plan charted onto its cells.
#[derive(Clone)]
pub struct SiteShown {
    pub body: SiteBody,
    pub patch: Arc<Patch>,
    pub charted: Charted,
}

/// The player's sites, read from the save whenever their records change.
#[derive(Resource, Default)]
pub struct Sites {
    pub by_id: BTreeMap<u64, SiteShown>,
    /// The save opening and the site records' size when last read.
    stamp: (u64, usize),
    /// Bumped by every change, for whatever is drawn from the sites.
    pub revision: u64,
}

/// A site's plan charted where it stands: its origin cell's patch, and the
/// pieces. `None` where the cells there no longer chart it (a world whose
/// cells cannot change, so a corrupt record).
pub fn chart_site(body: &SiteBody) -> Option<(Arc<Patch>, Charted)> {
    let direction = pbd_core::cell_key::key_direction(body.chart.origin)?;
    let patch = Arc::new(patch_round(direction, PATCH_REACH_M));
    let at = patch.keys.iter().position(|&k| k == body.chart.origin)?;
    let charted = chart_plan(
        &patch,
        &body.plan,
        at,
        usize::from(body.chart.turn),
        body.chart.base,
    )
    .ok()?;
    Some((patch, charted))
}

impl Sites {
    /// Read the sites again when the save's site records have changed.
    pub fn follow(&mut self, save: &WorldSave) {
        let stamp = (
            save.serial(),
            save.records
                .of_kind(SITE_RECORD)
                .map(|r| r.body.len() + 1)
                .sum::<usize>(),
        );
        if stamp == self.stamp {
            return;
        }
        self.stamp = stamp;
        let mut by_id = BTreeMap::new();
        for (id, body) in sites(&save.records) {
            // A site whose plan is the same as before keeps its chart.
            if let Some(old) = self.by_id.remove(&id)
                && old.body.plan == body.plan
                && old.body.chart == body.chart
            {
                by_id.insert(id, SiteShown { body, ..old });
                continue;
            }
            match chart_site(&body) {
                Some((patch, charted)) => {
                    by_id.insert(
                        id,
                        SiteShown {
                            body,
                            patch,
                            charted,
                        },
                    );
                }
                None => warn!("building site {id} does not chart where it stands"),
            }
        }
        self.by_id = by_id;
        self.revision += 1;
    }
}

/// Write a site's record as the player's, the first time noting the kind in
/// the world's identity. `false`, and nothing kept, once the save's writer
/// has failed.
pub fn store_site(save: &mut WorldSave, id: u64, body: &SiteBody) -> bool {
    if save
        .store(&Author::Player(0), vec![site_record(id, body)])
        .is_none()
    {
        return false;
    }
    save.note_record_kinds(&[(SITE_RECORD, SITE_SCHEMA)]);
    true
}

/// Write a site's record with the pack after it, in one line: scaffolding
/// paid for or given back.
pub fn store_site_with_hands(
    save: &mut WorldSave,
    id: u64,
    body: &SiteBody,
    carried: &Slots,
) -> bool {
    save.store_with_hands(site_record(id, body), SITE_SCHEMA, carried)
}

/// Set a site out (design 5): its record written, the next id counted up.
pub fn set_out(save: &mut WorldSave, body: &SiteBody) -> Option<u64> {
    let id = next_id(&save.records, SITE_RECORD);
    store_site(save, id, body).then_some(id)
}

/// Keep a blueprint as the player's own (design 9): `id` to change one, or
/// the next id.
pub fn store_blueprint(save: &mut WorldSave, id: Option<u64>, body: &BlueprintBody) -> Option<u64> {
    let id = id.unwrap_or_else(|| next_id(&save.records, BLUEPRINT_RECORD));
    save.store(&Author::Player(0), vec![blueprint_record(id, body)])?;
    save.note_record_kinds(&[(BLUEPRINT_RECORD, BLUEPRINT_SCHEMA)]);
    Some(id)
}

/// `carried` with `goods` taken out, or the first good it lacks and how many.
pub fn take_goods(carried: &Slots, goods: &Goods) -> Result<Slots, (String, u32)> {
    let mut after = carried.clone();
    for (name, n) in goods {
        let Some(item) = pbd_core::station::item_named(name) else {
            continue;
        };
        if after.count(item) < *n || !after.take_item(item, *n as u16) {
            return Err((name.clone(), *n));
        }
    }
    Ok(after)
}

/// `carried` with `goods` given back, or `None` where the pack has no room.
pub fn give_goods(carried: &Slots, goods: &Goods) -> Option<Slots> {
    let mut after = carried.clone();
    for (name, n) in goods {
        let Some(item) = pbd_core::station::item_named(name) else {
            continue;
        };
        if after.give(item, *n as u16) > 0 {
            return None;
        }
    }
    Some(after)
}

/// The lime lines and survey stakes of every unfinished site (design 5):
/// each cell along a ground storey's outside, marked from the cell before
/// it round the room to the cell after, in the paving marks' bits
/// (`townsfolk` 39a); and the cells the stakes stand in. A finished site
/// has neither.
pub fn site_marks(
    records: &pbd_core::records::Records,
) -> (Vec<(u32, (Option<u32>, Option<u32>))>, Vec<u32>) {
    use pbd_core::building::plan::PlanCells;
    use pbd_core::building::site::{outline, progress, stake_spots};
    type Marks = (Vec<(u32, (Option<u32>, Option<u32>))>, Vec<u32>);
    // Asked every frame by the stakes: worked out again only when a site
    // or a piece record changes.
    static LAST: std::sync::Mutex<Option<(u64, Marks)>> = std::sync::Mutex::new(None);
    let mut text = String::new();
    for r in records.of_kind(SITE_RECORD) {
        text.push_str(&r.body);
    }
    text.push_str(
        &records
            .of_kind(pbd_core::building::PIECE_RECORD)
            .count()
            .to_string(),
    );
    let stamp = pbd_core::river_cache::fnv(text.as_bytes());
    if let Some((at, marks)) = LAST.lock().unwrap_or_else(|e| e.into_inner()).as_ref()
        && *at == stamp
    {
        return marks.clone();
    }
    let pieces = pbd_core::building::from_records(records);
    let mut marks = Vec::new();
    let mut stakes = Vec::new();
    for body in sites(records).into_values() {
        let Some((patch, charted)) = chart_site(&body) else {
            continue;
        };
        if progress(&charted, body.marked.len(), &pieces).finished {
            continue;
        }
        let Some(at) = patch.keys.iter().position(|&k| k == body.chart.origin) else {
            continue;
        };
        let cells = PlanCells::new(&patch, at, usize::from(body.chart.turn), 6);
        let ring: Vec<u32> = outline(&body.plan)
            .into_iter()
            .filter_map(|h| cells.key(h).ok())
            .collect();
        let n = ring.len();
        for (i, &key) in ring.iter().enumerate() {
            let (before, after) = if n < 2 {
                (None, None)
            } else {
                (Some(ring[(i + n - 1) % n]), Some(ring[(i + 1) % n]))
            };
            marks.push((key, (before, after)));
        }
        stakes.extend(
            stake_spots(&body.plan)
                .into_iter()
                .filter_map(|(h, d)| cells.across(h, d).ok()),
        );
    }
    *LAST.lock().unwrap_or_else(|e| e.into_inner()) =
        Some((stamp, (marks.clone(), stakes.clone())));
    (marks, stakes)
}

/// A good's name for a line the player reads: its item's name.
pub fn good_name(name: &str) -> pbd_core::locale::Localized {
    match pbd_core::station::item_named(name) {
        Some(item) => item.name(),
        None => pbd_core::locale::Localized::verbatim(name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbd_core::building::site::{BuildingSetting, SiteChart, Size, blueprints};
    use std::collections::BTreeSet;

    /// Task 2.6 and 2.7: a site and a blueprint come back from the disk, an
    /// old world's records read as they did, and a world with no building
    /// setting is Free.
    #[test]
    fn a_site_survives_a_restart_and_an_old_save_opens_unchanged() {
        let root = std::env::temp_dir().join(format!("pbd-sites-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let slot = crate::saves::create(&root, "Planned", 31).unwrap();
        // An old world: an identity with no building setting, and a log.
        let identity_path = root.join(&slot.id).join(crate::saves::IDENTITY);
        let mut identity = crate::saves::format::Identity::legacy(31, 1);
        identity.tools = 1;
        std::fs::write(&identity_path, identity.to_ron()).unwrap();
        std::fs::write(root.join(&slot.id).join("edits.v1.log"), "77 200 0\n").unwrap();
        let data = load();
        let plan = data.presets.get("cottage").unwrap().plan(Size::Standard);
        let body = SiteBody {
            name: "blueprint.preset.cottage".into(),
            preset: Some(("cottage".into(), Size::Standard)),
            plan: plan.clone(),
            chart: SiteChart {
                origin: 5,
                turn: 1,
                base: 150,
            },
            marked: BTreeMap::new(),
            scaffold: BTreeSet::from([1]),
            cleared: false,
        };
        let records_before;
        {
            let mut save = WorldSave::open(root.clone(), slot.clone());
            assert_eq!(save.identity.building(), BuildingSetting::Free);
            records_before = save.records.len();
            assert_eq!(set_out(&mut save, &body), Some(1));
            let mine = BlueprintBody {
                name: "My cottage".into(),
                preset: Some(("cottage".into(), Size::Standard)),
                plan: Some(plan.clone()),
            };
            assert_eq!(store_blueprint(&mut save, None, &mine), Some(1));
            save.drain();
        }
        let reopened = WorldSave::open(root.clone(), crate::saves::list(&root)[0].clone());
        assert_eq!(sites(&reopened.records).get(&1), Some(&body));
        assert_eq!(blueprints(&reopened.records)[&1].name, "My cottage");
        assert_eq!(reopened.records.len(), records_before + 2);
        assert_eq!(
            reopened.edits.for_cell(77),
            &[(200, pbd_core::terrain::Material::Air)]
        );
        assert_eq!(
            reopened.identity.records.get(SITE_RECORD),
            Some(&SITE_SCHEMA)
        );
        assert_eq!(
            reopened.identity.building(),
            BuildingSetting::Free,
            "an old world stays Free"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A world this build makes is planned.
    #[test]
    fn a_new_world_is_planned() {
        assert_eq!(
            crate::saves::format::Identity::new(1).building(),
            BuildingSetting::Planned
        );
    }

    #[test]
    fn goods_come_out_of_the_pack_and_back() {
        let mut carried = Slots::default();
        carried.give(
            pbd_core::inventory::Item::Good(pbd_core::goods::Good::PLANKS),
            5,
        );
        let need: Goods = vec![("planks".into(), 4)];
        let after = take_goods(&carried, &need).unwrap();
        assert_eq!(
            after.count(pbd_core::inventory::Item::Good(
                pbd_core::goods::Good::PLANKS
            )),
            1
        );
        assert_eq!(
            take_goods(&after, &need).unwrap_err(),
            ("planks".to_string(), 4)
        );
        let back = give_goods(&after, &need).unwrap();
        assert_eq!(back, carried);
    }
}
