//! Survival's water and places (`survival-mode` design 8, 15 and 18, tasks
//! 6.3 and 6.6; "The rest of the non-combat pieces"): the still and its
//! record, a bucket's sips, a cactus drunk through a straw, the river
//! mouth's salt measured along the flow, a new Survival world's spawn near
//! fresh water, the laid bedroll, and a town's repair for coins. Rules only:
//! the desktop reads the player's keys and draws them.

use super::{WEAR_KIND, WEAR_SCHEMA, WearBook, rules};
use crate::saves::WorldSave;
use bevy::prelude::*;
use pbd_core::economy::coins::{PURSE_ID, PURSE_KIND, PURSE_SCHEMA, Purse, Reason};
use pbd_core::goods::Good;
use pbd_core::inventory::{Item, Slots, Stack, Tier, Tool};
use pbd_core::planet_gen::TerrainConfig;
use pbd_core::records::{Author, Record as StoredRecord};
use pbd_core::survival::water::{CactusDrunk, Sip, Still, sip};
use pbd_core::survival::wear::{Spend, Wear, Wearer};
use pbd_core::survival::{SurvivalConfig, WaterSource};
use pbd_core::world_frame::WorldPos;
use serde::{Deserialize, Serialize};

// ------------------------------------------------------ the river mouth

/// The width of a cell, m: the gold standard's mean tile (CLAUDE.md).
pub const CELL_M: f32 = 2.833;
/// River water this many cells from the sea along its flow is brackish
/// (design 15, "Estuaries").
pub const BRACKISH_CELLS: u32 = 6;

/// Whether the sea stands at `direction`: water the generator did not make
/// fresh, with its ground under the sea's level.
fn sea_at(cfg: &TerrainConfig, direction: Vec3) -> bool {
    !pbd_core::planet_gen::fresh_water_at(cfg, direction)
        && pbd_core::planet_gen::surface_altitude(cfg, direction) < cfg.sea_level_m
}

/// Whether river water at `direction` is brackish: the sea met within
/// [`BRACKISH_CELLS`] cells stepping downstream along the river's own flow
/// (each step against `rivers::at`'s `upstream`). Water that is no river
/// (a lake off the rivers, a swamp pool) is never brackish. Reads the
/// generator only: no world and no version changes.
pub fn brackish_at(cfg: &TerrainConfig, direction: Vec3) -> bool {
    let step = CELL_M / cfg.radius_m;
    let mut d = direction.normalize_or(Vec3::Y);
    let Some(mut down) = pbd_core::rivers::at(cfg, d).map(|r| -r.upstream) else {
        return false;
    };
    for _ in 0..BRACKISH_CELLS {
        let tangent = (down - d * down.dot(d)).normalize_or_zero();
        if tangent == Vec3::ZERO {
            return false;
        }
        d = (d + tangent * step).normalize();
        if sea_at(cfg, d) {
            return true;
        }
        // Past the river's last reach the way on is the way it was going.
        if let Some(here) = pbd_core::rivers::at(cfg, d) {
            down = -here.upstream;
        }
    }
    false
}

/// What water at a planet point is to drink (design 15): the sea, fresh (a
/// river, a lake, a swamp pool, water gathered off the sea's level), or
/// brackish (a river within six cells of the sea along its flow).
/// `top_m` is the water's surface altitude over the radius.
pub fn water_source(point: Vec3, top_m: f32) -> WaterSource {
    let cfg = crate::planet::terrain_config();
    let direction = point.normalize_or(Vec3::Y);
    let fresh = pbd_core::planet_gen::fresh_water_at(cfg, direction);
    if fresh {
        return if brackish_at(cfg, direction) {
            WaterSource::Brackish
        } else {
            WaterSource::Fresh
        };
    }
    if top_m < cfg.sea_level_m + 0.6 {
        WaterSource::Sea
    } else {
        // Water off the sea's level that the generator did not make is
        // water that gathered: a pool, a flooded hole above the sea.
        WaterSource::Fresh
    }
}

// ------------------------------------------------- the spawn near water

/// How near fresh water a new Survival world's start wants to be, m.
pub const FRESH_WITHIN_M: f32 = 150.0;
/// How far the search for fresh water walks, m.
pub const FRESH_SEARCH_M: f32 = 3000.0;

/// Points on rings round `centre` (a unit direction) every `step_m` out to
/// `out_m`, nearest ring first, about `step_m` apart along each ring.
fn rings(centre: Vec3, radius_m: f32, step_m: f32, out_m: f32) -> impl Iterator<Item = Vec3> {
    let c = centre.normalize_or(Vec3::Y);
    let a = c.any_orthonormal_vector();
    let b = c.cross(a);
    let count = (out_m / step_m).ceil() as u32;
    (1..=count).flat_map(move |k| {
        let r = k as f32 * step_m;
        let around = ((std::f32::consts::TAU * r / step_m).ceil() as u32).max(6);
        let angle = r / radius_m;
        (0..around).map(move |i| {
            let t = i as f32 / around as f32 * std::f32::consts::TAU;
            (c * angle.cos() + (a * t.cos() + b * t.sin()) * angle.sin()).normalize()
        })
    })
}

/// Whether `fresh` holds anywhere within `within_m` of `at`.
fn fresh_within(at: Vec3, radius_m: f32, within_m: f32, fresh: &impl Fn(Vec3) -> bool) -> bool {
    fresh(at) || rings(at, radius_m, 15.0, within_m).any(fresh)
}

/// A new Survival world's start (design 15): `start` (today's rule's) when
/// fresh water lies within [`FRESH_WITHIN_M`]; else the start `land_near`
/// gives nearest the nearest fresh water within [`FRESH_SEARCH_M`], or the
/// nearest dry land to it from `dry_near`, if either is within reach of it;
/// else `start` stands. Called for a NEW world's first arrival only.
pub fn start_near_fresh_water(
    start: Vec3,
    radius_m: f32,
    fresh: impl Fn(Vec3) -> bool,
    land_near: impl Fn(Vec3) -> Vec3,
    dry_near: impl Fn(Vec3) -> Vec3,
) -> Vec3 {
    if fresh_within(start, radius_m, FRESH_WITHIN_M, &fresh) {
        return start;
    }
    let Some(water) = rings(start, radius_m, 30.0, FRESH_SEARCH_M).find(|&d| fresh(d)) else {
        return start;
    };
    for candidate in [land_near(water), dry_near(water)] {
        let candidate = candidate.normalize_or(start);
        let gap = candidate.angle_between(water) * radius_m;
        if gap <= FRESH_WITHIN_M && !fresh(candidate) {
            return candidate;
        }
    }
    start
}

// ------------------------------------------------------------ the bucket

/// Sips taken from the fresh water bucket in hand (design 15), one record
/// of the player's: none taken reads as a full bucket.
pub const BUCKET_KIND: &str = "bucket";
pub const BUCKET_SCHEMA: u32 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BucketSips {
    pub taken: u8,
}

/// A sip from the bucket in the selected slot (Survival only): a water
/// bucket gives fresh water and, at its last sip, becomes an empty bucket in
/// the same slot, in one saved line with the hands; a sea bucket gives
/// nothing and "Salt water". `None` when no full bucket is held.
pub fn drink_bucket(save: &mut WorldSave, carried: &mut Slots, water_now: f32) -> Option<Sip> {
    let cfg = rules();
    let Some(Item::Good(good)) = carried.held().map(|s| s.item) else {
        return None;
    };
    if good == Good::SEA_BUCKET {
        return Some(sip(cfg, WaterSource::Bucket { salt: true }, water_now));
    }
    if good != Good::WATER_BUCKET || !save.mode().is_survival() {
        return None;
    }
    let mut sips: BucketSips = save
        .records
        .get(BUCKET_KIND, 0)
        .and_then(|r| r.read())
        .unwrap_or_default();
    let mut after = carried.clone();
    sips.taken += 1;
    if sips.taken >= cfg.drinks.bucket.sips {
        sips.taken = 0;
        let at = after.selected();
        after.set(at, Some(Stack::new(Item::Good(Good::BUCKET), 1)));
    }
    let record = StoredRecord::of(BUCKET_KIND, 0, BUCKET_SCHEMA, &sips);
    if !save.store_with_hands(record, BUCKET_SCHEMA, &after) {
        return None;
    }
    *carried = after;
    Some(sip(cfg, WaterSource::Bucket { salt: false }, water_now))
}

// ------------------------------------------------------------- the still

/// A still's record: the core's [`Still`], keyed by its cell and layer as a
/// station's (`station::record_id`).
pub const STILL_KIND: &str = "still";
pub const STILL_SCHEMA: u32 = 1;

/// The still at `(cell, layer)` as saved, or empty.
pub fn still_at(save: &WorldSave, cell: u32, layer: u16) -> Still {
    save.records
        .get(STILL_KIND, pbd_core::station::record_id(cell, layer))
        .and_then(|r| r.read())
        .unwrap_or_default()
}

fn store_still(save: &mut WorldSave, cell: u32, layer: u16, still: &Still) -> bool {
    let record = StoredRecord::of(
        STILL_KIND,
        pbd_core::station::record_id(cell, layer),
        STILL_SCHEMA,
        still,
    );
    let kept = save.store(&Author::Player(0), vec![record]).is_some();
    if kept {
        save.note_record_kinds(&[(STILL_KIND, STILL_SCHEMA)]);
    }
    kept
}

/// `dt` seconds of play at the still at `(cell, layer)`, beside a fire that
/// is `fire_lit`, in `raining`: the boil's seconds carried in `still`, the
/// record written only when a sip boiled or fell. `true` when it changed.
pub fn step_still(
    save: &mut WorldSave,
    (cell, layer): (u32, u16),
    still: &mut Still,
    fire_lit: bool,
    raining: bool,
    dt: f32,
) -> bool {
    let before = (still.salt_sips, still.fresh_sips);
    still.step(rules(), fire_lit, raining, dt);
    if (still.salt_sips, still.fresh_sips) == before {
        return false;
    }
    store_still(save, cell, layer, still)
}

/// What F at a still did.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StillUse {
    /// A sea bucket poured in: it is an empty bucket now.
    Poured,
    /// A fresh sip drunk.
    Drank(Sip),
    /// Nothing fresh in it, and salt waiting for a fire.
    NeedsFire,
    /// Nothing in it at all.
    Empty,
}

/// F at the still at `(cell, layer)` (Survival only): a sea bucket in hand
/// is poured in (its sips of salt), and with an empty hand a fresh sip is
/// drunk. Saved at once, with the hands where they changed.
pub fn use_still(
    save: &mut WorldSave,
    carried: &mut Slots,
    (cell, layer): (u32, u16),
    water_now: f32,
) -> Option<StillUse> {
    if !save.mode().is_survival() {
        return None;
    }
    let cfg = rules();
    let mut still = still_at(save, cell, layer);
    let held = carried.held().map(|s| s.item);
    if held == Some(Item::Good(Good::SEA_BUCKET)) {
        still.fill_salt(cfg, cfg.drinks.bucket.sips);
        let mut after = carried.clone();
        let at = after.selected();
        after.set(at, Some(Stack::new(Item::Good(Good::BUCKET), 1)));
        let record = StoredRecord::of(
            STILL_KIND,
            pbd_core::station::record_id(cell, layer),
            STILL_SCHEMA,
            &still,
        );
        if !save.store_with_hands(record, STILL_SCHEMA, &after) {
            return None;
        }
        *carried = after;
        return Some(StillUse::Poured);
    }
    if held.is_some() {
        return None;
    }
    match still.drink(cfg, water_now) {
        Some(sipped) => store_still(save, cell, layer, &still).then_some(StillUse::Drank(sipped)),
        None if still.salt_sips > 0 => Some(StillUse::NeedsFire),
        None => Some(StillUse::Empty),
    }
}

// ------------------------------------------------- the cactus and the straw

/// A cactus drunk from, by its cell's key (design 15).
pub const CACTUS_KIND: &str = "cactus";
pub const CACTUS_SCHEMA: u32 = 1;
/// The straw's wear is kept in the `wear` record under this code.
pub const STRAW_WEAR: &str = "straw";

/// The world day `seconds` of world time is, fractional.
pub fn world_day(seconds: f64) -> f64 {
    seconds / f64::from(pbd_core::daylight::DAY_S)
}

/// A sip from the cactus whose cell is `key`, through the straw in hand, at
/// world day `day` (Survival only): the `cactus` record written with it; a
/// sip spends the straw one use, and a straw that breaks is used up (one
/// leaves the stack, in the same line). "Dry" and nothing spent when it has
/// no sips. `None` without a straw in hand.
pub fn sip_cactus(
    save: &mut WorldSave,
    carried: &mut Slots,
    key: u32,
    day: f64,
    water_now: f32,
) -> Option<(Sip, bool)> {
    if !save.mode().is_survival() || carried.held().map(|s| s.item) != Some(Item::Good(Good::STRAW))
    {
        return None;
    }
    let cfg = rules();
    let record: Option<CactusDrunk> = save
        .records
        .get(CACTUS_KIND, u64::from(key))
        .and_then(|r| r.read());
    let (next, sipped) = CactusDrunk::drink(cfg, record.as_ref(), day, water_now);
    let Some(next) = next else {
        return Some((sipped, false));
    };
    // The straw's use, in the same save as the sip.
    let mut book = WearBook::of(save);
    let wearer = Wearer::Straw;
    let mut wear = Wear::of_saved(cfg, &wearer, book.tools.get(STRAW_WEAR).copied());
    let spent = wear.spend(
        cfg,
        save.mode(),
        &wearer,
        pbd_core::survival::wear::Action::Used,
    );
    let mut after = carried.clone();
    let broke = spent == Spend::Broke;
    if broke {
        after.take_item(Item::Good(Good::STRAW), 1);
        wear.repair(cfg, &wearer);
    }
    book.tools.insert(STRAW_WEAR.to_string(), wear);
    let lines = vec![
        StoredRecord::of(CACTUS_KIND, u64::from(key), CACTUS_SCHEMA, &next),
        StoredRecord::of(WEAR_KIND, 0, WEAR_SCHEMA, &book),
    ];
    save.store(&Author::Player(0), lines)?;
    save.note_record_kinds(&[(CACTUS_KIND, CACTUS_SCHEMA), (WEAR_KIND, WEAR_SCHEMA)]);
    if broke {
        // The used-up straw leaves the hands.
        let hands = StoredRecord::of(WEAR_KIND, 0, WEAR_SCHEMA, &book);
        if !save.store_with_hands(hands, WEAR_SCHEMA, &after) {
            return None;
        }
    }
    *carried = after;
    Some((sipped, broke))
}

// ------------------------------------------------------------ the bedroll

/// The laid bedroll (decision 8, task 6.3): one record of the player's.
/// Laid, it is where a death wakes; taken up, `at` is `None`.
pub const BEDROLL_KIND: &str = "bedroll";
pub const BEDROLL_SCHEMA: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bedroll {
    /// Where it lies on the ground, body-relative f64.
    pub at: Option<WorldPos>,
    /// The way its head end points, a unit tangent in the body's frame.
    pub facing: [f32; 3],
}

/// The laid bedroll, if one lies in the world.
pub fn bedroll(save: &WorldSave) -> Option<Bedroll> {
    save.records
        .get(BEDROLL_KIND, 0)
        .and_then(|r| r.read::<Bedroll>())
        .filter(|b| b.at.is_some())
}

/// Lay the bedroll in hand at `at`, its head toward `facing` (Survival
/// only): it leaves the hand, and the record that is the wake's spawn is
/// written in one line with the hands. A bedroll laid before comes back to
/// the pack in the same line.
pub fn lay_bedroll(save: &mut WorldSave, carried: &mut Slots, at: WorldPos, facing: Vec3) -> bool {
    if !save.mode().is_survival()
        || carried.held().map(|s| s.item) != Some(Item::Good(Good::BEDROLL))
    {
        return false;
    }
    let mut after = carried.clone();
    if after.take_held().is_none() {
        return false;
    }
    if bedroll(save).is_some() && after.give(Item::Good(Good::BEDROLL), 1) > 0 {
        // No room for the one laid before: it stays where it lies.
        return false;
    }
    let laid = Bedroll {
        at: Some(at),
        facing: facing.normalize_or(Vec3::X).to_array(),
    };
    let record = StoredRecord::of(BEDROLL_KIND, 0, BEDROLL_SCHEMA, &laid);
    if !save.store_with_hands(record, BEDROLL_SCHEMA, &after) {
        return false;
    }
    *carried = after;
    true
}

/// Take the laid bedroll up into the pack: the wake is the world's spawn
/// again. `false` when none lies or the pack is full.
pub fn take_up_bedroll(save: &mut WorldSave, carried: &mut Slots) -> bool {
    let Some(laid) = bedroll(save) else {
        return false;
    };
    let mut after = carried.clone();
    if after.give(Item::Good(Good::BEDROLL), 1) > 0 {
        return false;
    }
    let taken = Bedroll { at: None, ..laid };
    let record = StoredRecord::of(BEDROLL_KIND, 0, BEDROLL_SCHEMA, &taken);
    if !save.store_with_hands(record, BEDROLL_SCHEMA, &after) {
        return false;
    }
    *carried = after;
    true
}

// ------------------------------------------------------ repairs (4.8)

/// What a workbench repair of `tool` at `tier` takes (design 11): half the
/// head of the recipe that makes it, rounded up, as items. `None` when no
/// recipe makes it.
pub fn repair_price(
    book: &pbd_core::station::RecipeBook,
    cfg: &SurvivalConfig,
    tool: Tool,
    tier: Tier,
) -> Option<Vec<(Item, u16)>> {
    let recipe = book
        .offered()
        .find(|r| r.tools.iter().any(|&(t, k)| t == tool && k == tier))?;
    let named: Vec<(String, u16)> = recipe
        .inputs
        .iter()
        .map(|&(item, n)| (item_code(item), n))
        .collect();
    let price = pbd_core::survival::wear::repair_inputs(cfg, &named)
        .into_iter()
        .filter_map(|(code, n)| Some((pbd_core::station::item_named(&code)?, n)))
        .collect();
    Some(price)
}

/// An item as `recipes.ron` names it.
fn item_code(item: Item) -> String {
    match item {
        Item::Good(good) => good.code().to_string(),
        Item::Block(m) => format!("block:{}", pbd_core::inventory::material_id(m)),
        Item::Tool(tool) => format!("tool:{}", tool.code()),
        Item::Fish(n) => format!("fish:{n}"),
    }
}

/// Repair `tool` at the workbench: `price` leaves the hands and every use
/// comes back, in one saved line. `false` when the hands lack the price.
pub fn repair_at_bench(
    save: &mut WorldSave,
    carried: &mut Slots,
    tool: Tool,
    tier: Tier,
    price: &[(Item, u16)],
) -> bool {
    let mut after = carried.clone();
    for &(item, n) in price {
        if !after.take_item(item, n) {
            return false;
        }
    }
    if !super::repair_tool(save, tool, tier, &after) {
        return false;
    }
    *carried = after;
    true
}

/// What a town's smith or trader asks to repair `tool` at `tier` now, in
/// coins; nothing for a tool at full uses. `value_coins` is its row in
/// `values.ron` (`tool:<code>`).
pub fn repair_coins_for(save: &WorldSave, tool: Tool, tier: Tier, value_coins: u32) -> u32 {
    let cfg = rules();
    let wearer = Wearer::Tool(tool, tier);
    let wear = WearBook::of(save).wear(tool, tier);
    pbd_core::survival::wear::repair_coins(cfg, value_coins, wear, &wearer)
}

/// A town repair (design 11): `coins` leave `purse` and `tool` gets every
/// use back, stored together. `Err` with the purse unchanged when it is
/// short, or when the save refused.
pub fn repair_for_coins(
    save: &mut WorldSave,
    purse: &mut Purse,
    tool: Tool,
    tier: Tier,
    coins: u32,
) -> Result<(), pbd_core::economy::coins::Refused> {
    let mut paid = purse.clone();
    paid.change(Reason::Bought, u64::from(coins))?;
    let cfg = rules();
    let wearer = Wearer::Tool(tool, tier);
    let mut book = WearBook::of(save);
    let mut wear = book.wear(tool, tier);
    wear.repair(cfg, &wearer);
    book.tools.insert(tool.code().to_string(), wear);
    let lines = vec![
        StoredRecord::of(WEAR_KIND, 0, WEAR_SCHEMA, &book),
        StoredRecord::of(PURSE_KIND, PURSE_ID, PURSE_SCHEMA, &paid),
    ];
    if save.store(&Author::Player(0), lines).is_none() {
        return Err(pbd_core::economy::coins::Refused::Short {
            have: purse.coins,
            need: u64::from(coins),
        });
    }
    save.note_record_kinds(&[(WEAR_KIND, WEAR_SCHEMA), (PURSE_KIND, PURSE_SCHEMA)]);
    *purse = paid;
    Ok(())
}
