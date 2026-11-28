//! Survival in the game (`survival-mode` tasks 4.1, 4.2, 4.8, 4.9, 6.6):
//! the core's rules (`pbd_core::survival`) fed by what the walker and the
//! world know, and their answers put back where the game acts on them.
//!
//! - **The bars** are the in-process server's ([`WorldSave::survive`]):
//!   [`tick`] gathers the [`Conditions`] at the player each frame (the
//!   walker's effort, the water at the head, the weather at the player, the
//!   shade by a march through the column tier, fires near, what is worn)
//!   and hands them over; [`Survival`] is what came back, for the HUD.
//! - **The walker** reports what it did in [`Activity`] (its effort, its
//!   jumps) and reads what it may do in [`WalkerLimits`] (winded, passed
//!   out, dazed). Falls and climbs are read off its motion here.
//! - **Death** drops the pack as one bag record ([`drop_pack`]), picked up
//!   whole ([`pick_bag`]); waking puts the player back at their bedroll or
//!   the spawn (the desktop's).
//! - **Wear** is a record of the uses each tool has left ([`WearBook`]).
//! - **Water and places** ([`places`]): the still, a bucket's sips, the
//!   cactus and the straw, the river mouth's salt, the spawn near fresh
//!   water, the laid bedroll, and a town's repair.
//!
//! None of it runs in an Explorer world: [`WorldSave::survive`] answers
//! `None` there, and nothing else is shown or spent.

use crate::saves::{WorldSave, format};
use crate::walking::{GroundState, Walker, WalkingConfig, WalkingReadout};
use avian3d::prelude::{LinearVelocity, Position};
use bevy::prelude::*;
use pbd_core::inventory::{CARRIED, Slots, Tool};
use pbd_core::records::Record as StoredRecord;
use pbd_core::survival::fall::{self, Landing};
use pbd_core::survival::health::Cause;
use pbd_core::survival::passout::Control;
use pbd_core::survival::stamina::{Burst, Effort, Limits};
use pbd_core::survival::temperature::{Exposure, FireNear, Worn};
use pbd_core::survival::wear::{Spend, Wear, Wearer};
use pbd_core::survival::{Conditions, Event, SurvivalConfig, Vitals};
use pbd_core::world_frame::WorldPos;
use pbd_server::survival::Act;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::OnceLock;

/// The shipped survival rules (`assets/config/survival.ron`), parsed and
/// validated once.
pub fn rules() -> &'static SurvivalConfig {
    static RULES: OnceLock<SurvivalConfig> = OnceLock::new();
    RULES.get_or_init(SurvivalConfig::shipped)
}

// ------------------------------------------------------------- the walker

/// What the walker did since the survival tick last read it.
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct Activity {
    pub effort: Effort,
    /// Jumps taken from the ground.
    pub jumps: u32,
    /// The head is under water.
    pub head_in_water: bool,
    /// In water deep enough to swim in, the feet off the bottom.
    pub in_deep_water: bool,
}

/// What survival lets the walker do now: winded's limits (design 12) and
/// the pass-out's control (design 14). Free and awake in Explorer.
#[derive(Resource, Clone, Copy, Debug, PartialEq)]
pub struct WalkerLimits {
    pub stamina: Limits,
    pub control: Control,
}

impl Default for WalkerLimits {
    fn default() -> Self {
        Self {
            stamina: Limits::FREE,
            control: Control {
                controls: true,
                can_sprint: true,
                walk_mult: 1.0,
                floats_face_up: false,
            },
        }
    }
}

impl WalkerLimits {
    pub fn can_sprint(&self) -> bool {
        self.stamina.can_sprint && self.control.can_sprint && self.control.controls
    }

    /// The jump's take-off speed share for its height share: a jump at 60%
    /// height leaves at sqrt(0.6) of the speed.
    pub fn jump_speed_mult(&self) -> f32 {
        self.stamina.jump_mult.max(0.0).sqrt()
    }

    /// The body floats head out: winded in the water, or passed out in it.
    pub fn floats(&self) -> bool {
        self.stamina.floats || self.control.floats_face_up
    }
}

// --------------------------------------------------------------- the HUD

/// A short line above the hotbar ("Salt water", "Thirsty", "Pickaxe
/// broke"), and the seconds it has left.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub text: pbd_core::locale::Localized,
    pub left_s: f32,
}

/// Seconds a line above the hotbar stays.
pub const LINE_S: f32 = 3.0;
/// Seconds the red edge of a hit stays.
pub const HIT_S: f32 = 0.6;

/// The survival state the HUD and the rest of the game read: the server's
/// bars as of this frame and what happened to them.
#[derive(Resource, Clone, Debug, Default)]
pub struct Survival {
    /// `None`: an Explorer world (no bars, nothing shown).
    pub vitals: Option<Vitals>,
    /// How the place feels, deg C.
    pub feel_c: f32,
    /// The body's temperature trend, deg C a minute.
    pub trend_c_per_min: f32,
    /// What the place's temperature comes from, for the thermometer's icon.
    pub cause: Option<TempCause>,
    /// Seconds of the red edge left.
    pub hit_s: f32,
    /// The line above the hotbar.
    pub line: Option<Line>,
    /// The player is dead: the HUD shows "You died" until Wake up.
    pub dead: bool,
    /// Set once on the frame of a death, for the desktop to drop the pack.
    pub died_now: bool,
    /// Where the player is lying, local (for the pass-out and death).
    pub at: Option<Vec3>,
}

impl Survival {
    pub fn say(&mut self, text: pbd_core::locale::Localized) {
        self.line = Some(Line {
            text,
            left_s: LINE_S,
        });
    }
}

/// The small icon beside the thermometer (design 13): the strongest cause.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TempCause {
    Sun,
    Wind,
    Wet,
    Fire,
    Snow,
}

/// The cause an [`Exposure`] reads as, if any stands out.
pub fn cause_of(e: &Exposure, body_c: f32) -> Option<TempCause> {
    if !e.fires.is_empty() {
        return Some(TempCause::Fire);
    }
    if e.wet || e.in_water {
        return Some(TempCause::Wet);
    }
    if body_c >= 37.5 && e.sunlight_wm2 > 200.0 && !e.shaded {
        return Some(TempCause::Sun);
    }
    if body_c < 36.5 && e.air_c < 0.0 {
        return Some(TempCause::Snow);
    }
    if body_c < 36.5 && e.wind_mps > 4.0 {
        return Some(TempCause::Wind);
    }
    None
}

// ------------------------------------------------------------- worn slots

/// What is worn in the two slots (design 13), a record of the player's.
pub const WORN_KIND: &str = "worn";
pub const WORN_SCHEMA: u32 = 1;

/// The two worn slots as the record keeps them: good codes.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WornRecord {
    pub head: Option<String>,
    pub body: Option<String>,
}

impl WornRecord {
    pub fn of(save: &WorldSave) -> Self {
        save.records
            .get(WORN_KIND, 0)
            .and_then(|r| r.read())
            .unwrap_or_default()
    }

    pub fn worn(&self) -> Worn {
        Worn {
            head: self.head.clone(),
            body: self.body.clone(),
        }
    }

    /// Which slot a good is worn in, if it is worn at all.
    pub fn slot_of(code: &str) -> Option<bool> {
        match code {
            "straw-hat" => Some(true),
            "wool-cloak" => Some(false),
            _ => None,
        }
    }
}

/// A dish just eaten also fills the bars in Survival (design "Eating in a
/// Survival world"): food and water through the server, at one star, since
/// items carry no stars. `None` in Explorer.
pub fn fill_from(
    save: &mut WorldSave,
    good: pbd_core::goods::Good,
) -> Option<pbd_core::survival::body::Eaten> {
    save.survival_act(Act::Eat(good.code().to_string(), 1))?
        .eaten
}

/// Eat one of the raw food in the selected slot (Survival only): a good
/// `survival.ron` fills or waters that is not a cooked dish (raw meat, an
/// egg, milk, a tree fruit). The slot loses it in one saved line, then the
/// bars fill. `None`, with nothing changed, in Explorer, for a good that is
/// not eaten, or when the save refused.
pub fn eat_raw(
    save: &mut WorldSave,
    carried: &mut Slots,
) -> Option<(pbd_core::goods::Good, pbd_core::survival::body::Eaten)> {
    if !save.mode().is_survival() {
        return None;
    }
    let Some(pbd_core::inventory::Item::Good(good)) = carried.held().map(|s| s.item) else {
        return None;
    };
    if !pbd_core::survival::hunger::is_eaten(rules(), good.code()) {
        return None;
    }
    let mut after = carried.clone();
    after.take_held()?;
    if !save.record_pack(&after) {
        return None;
    }
    *carried = after;
    let eaten = fill_from(save, good)?;
    Some((good, eaten))
}

/// Put on the good in the selected slot (Survival only): it leaves the
/// hand for its worn slot, and what was there comes back into the hand, in
/// one saved line. `false` when the held item is not worn or the save
/// refused it.
pub fn wear_held(save: &mut WorldSave, carried: &mut Slots) -> bool {
    if !save.mode().is_survival() {
        return false;
    }
    let Some(pbd_core::inventory::Item::Good(good)) = carried.held().map(|s| s.item) else {
        return false;
    };
    let Some(head) = WornRecord::slot_of(good.code()) else {
        return false;
    };
    let mut worn = WornRecord::of(save);
    let mut after = carried.clone();
    if after.take_held().is_none() {
        return false;
    }
    let slot = if head { &mut worn.head } else { &mut worn.body };
    let off = slot.replace(good.code().to_string());
    if let Some(off) = off.and_then(|c| pbd_core::goods::Good::by_code(&c)) {
        after.give(pbd_core::inventory::Item::Good(off), 1);
    }
    let record = StoredRecord::of(WORN_KIND, 0, WORN_SCHEMA, &worn);
    if !save.store_with_hands(record, WORN_SCHEMA, &after) {
        return false;
    }
    *carried = after;
    true
}

// --------------------------------------------------------- tool wear (4.8)

/// The uses each owned tool has left (design 11), one record of the
/// player's: a tool with no entry is full, so an old save reads full.
pub const WEAR_KIND: &str = "wear";
pub const WEAR_SCHEMA: u32 = 1;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WearBook {
    /// By the tool's code (`tool:pickaxe`'s `pickaxe`).
    pub tools: BTreeMap<String, Wear>,
}

impl WearBook {
    pub fn of(save: &WorldSave) -> Self {
        save.records
            .get(WEAR_KIND, 0)
            .and_then(|r| r.read())
            .unwrap_or_default()
    }

    /// `tool`'s wear at `tier`: the saved count, or full.
    pub fn wear(&self, tool: Tool, tier: pbd_core::inventory::Tier) -> Wear {
        let wearer = Wearer::Tool(tool, tier);
        Wear::of_saved(rules(), &wearer, self.tools.get(tool.code()).copied())
    }

    /// Whether `tool` is broken (works as bare hands).
    pub fn broken(&self, tool: Tool, tier: pbd_core::inventory::Tier) -> bool {
        self.wear(tool, tier).is_broken()
    }
}

/// Whether the tool in hand is broken this frame (Survival only): it works
/// as bare hands (design 11). Kept by the desktop's wear systems.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HeldBroken(pub bool);

/// Spend `tool`'s uses for a finished `action` (Survival only), saved at
/// once. Returns what was spent; [`Spend::Broke`] the use it broke on.
pub fn spend_wear(
    save: &mut WorldSave,
    tool: Tool,
    tier: pbd_core::inventory::Tier,
    action: pbd_core::survival::wear::Action,
) -> Spend {
    if !save.mode().is_survival() {
        return Spend::None;
    }
    let mut book = WearBook::of(save);
    let wearer = Wearer::Tool(tool, tier);
    let mut wear = book.wear(tool, tier);
    let spent = wear.spend(rules(), save.mode(), &wearer, action);
    if spent == Spend::None {
        return spent;
    }
    book.tools.insert(tool.code().to_string(), wear);
    let record = StoredRecord::of(WEAR_KIND, 0, WEAR_SCHEMA, &book);
    if save
        .store(&pbd_core::records::Author::Player(0), vec![record])
        .is_none()
    {
        return Spend::None;
    }
    save.note_record_kinds(&[(WEAR_KIND, WEAR_SCHEMA)]);
    spent
}

/// Repair `tool` to full (design 11): the caller has taken its price (the
/// workbench's materials or the smith's coins) in `carried`, which is saved
/// with it in one line.
pub fn repair_tool(
    save: &mut WorldSave,
    tool: Tool,
    tier: pbd_core::inventory::Tier,
    carried: &Slots,
) -> bool {
    let mut book = WearBook::of(save);
    let wearer = Wearer::Tool(tool, tier);
    let mut wear = book.wear(tool, tier);
    wear.repair(rules(), &wearer);
    book.tools.insert(tool.code().to_string(), wear);
    let record = StoredRecord::of(WEAR_KIND, 0, WEAR_SCHEMA, &book);
    save.store_with_hands(record, WEAR_SCHEMA, carried)
}

// ------------------------------------------------- death and the bag (4.2)

/// A dropped pack: one record per death, its id the death's number.
pub const BAG_KIND: &str = "pack-bag";
pub const BAG_SCHEMA: u32 = 1;
/// How near the walker's centre a bag is picked up, m.
pub const BAG_REACH_M: f32 = 1.6;

/// A bag as the record keeps it: where it lies (body-relative f64) and
/// every stack in it, in its slot's order, as the save's slot text.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bag {
    pub at: WorldPos,
    pub stacks: Vec<String>,
}

impl Bag {
    pub fn is_empty(&self) -> bool {
        self.stacks.iter().all(|s| s == "-")
    }

    /// The stacks as slots, in order.
    pub fn slots(&self) -> Slots {
        let mut slots = Slots::new();
        for text in &self.stacks {
            if let Some(Some(stack)) = format::stack_of(text) {
                slots.give(stack.item, stack.count);
            }
        }
        slots
    }
}

/// Every bag still lying in the world, by id.
pub fn bags(save: &WorldSave) -> Vec<(u64, Bag)> {
    save.records
        .of_kind(BAG_KIND)
        .filter_map(|r| Some((r.id, r.read::<Bag>()?)))
        .filter(|(_, b)| !b.is_empty())
        .collect()
}

/// At a death: the whole pack, the hotbar included, into one bag at `at`,
/// in one saved line with the emptied slots. `None` when there was nothing
/// to drop or the save refused it (the pack stays).
pub fn drop_pack(save: &mut WorldSave, carried: &mut Slots, at: WorldPos) -> Option<u64> {
    let stacks: Vec<String> = (0..CARRIED)
        .map(|i| format::stack_text(carried.get(i)))
        .collect();
    let bag = Bag { at, stacks };
    if bag.is_empty() {
        return None;
    }
    let id = save
        .records
        .of_kind(BAG_KIND)
        .map(|r| r.id + 1)
        .max()
        .unwrap_or(0);
    let mut emptied = Slots::new();
    emptied.select(carried.selected());
    let record = StoredRecord::of(BAG_KIND, id, BAG_SCHEMA, &bag);
    if !save.store_with_hands(record, BAG_SCHEMA, &emptied) {
        return None;
    }
    *carried = emptied;
    Some(id)
}

/// Walking into bag `id`: everything that fits goes into `carried`, what
/// does not stays in the bag, in one saved line. `true` when anything moved.
pub fn pick_bag(save: &mut WorldSave, carried: &mut Slots, id: u64) -> bool {
    let Some(mut bag) = save.records.get(BAG_KIND, id).and_then(|r| r.read::<Bag>()) else {
        return false;
    };
    let mut after = carried.clone();
    let mut moved = false;
    for text in &mut bag.stacks {
        let Some(Some(stack)) = format::stack_of(text) else {
            continue;
        };
        let left = after.give(stack.item, stack.count);
        if left < stack.count {
            moved = true;
        }
        *text = format::stack_text(
            (left > 0).then(|| pbd_core::inventory::Stack::new(stack.item, left)),
        );
    }
    if !moved {
        return false;
    }
    let record = StoredRecord::of(BAG_KIND, id, BAG_SCHEMA, &bag);
    if !save.store_with_hands(record, BAG_SCHEMA, &after) {
        return false;
    }
    *carried = after;
    true
}

// ------------------------------------------------------- water (task 6.6)

/// What water at a planet point is to drink: [`places::water_source`].
pub use places::{water_source, water_source_in};

// ------------------------------------------------------------- the tick

/// Whether the air on foot is too thin at `radius_m` from the planet's
/// centre (design "Thin air on foot"): above its mean radius by more than
/// `suit_air.thin_above_m`.
pub fn thin_air_at(radius_m: f64) -> bool {
    let mean = f64::from(crate::planet::terrain_config().radius_m);
    pbd_core::survival::breath::thin_at(rules(), (radius_m - mean) as f32)
}

/// What the tick keeps between frames to read falls and climbs off the
/// walker's motion.
#[derive(Default)]
pub struct Watch {
    /// The highest downward speed along up while airborne and dry, m/s.
    falling_mps: f32,
    was_grounded: bool,
    was_wet: bool,
    /// The radius the feet stood at last frame, while grounded.
    stood_at: Option<f64>,
    /// The highest radius the feet reached while airborne, m from the
    /// body's centre: a fall is judged against the drop from it.
    air_top: Option<f64>,
    /// Climbed metres not yet spent.
    climbed_m: f32,
    body_c_was: Option<f32>,
    /// The body's state last frame: a change to fine counts `cool` or
    /// `warm` for the missions.
    state_was: Option<pbd_core::survival::temperature::BodyState>,
}

/// The world at the player the tick reads.
type AtPlayer<'w> = (
    Option<Res<'w, crate::atmosphere::Air>>,
    Option<Res<'w, crate::planet::PlanetContact>>,
    Option<Res<'w, crate::CelestialScene>>,
    Option<Res<'w, crate::PhysicsFrame>>,
    Option<Res<'w, crate::sky::Sun>>,
);

/// Each frame of play: gather the conditions, tick the server's bars, and
/// spend what the walker did. Nothing in an Explorer world.
#[allow(clippy::too_many_arguments)]
pub(crate) fn tick(
    time: Res<Time>,
    save: Option<ResMut<WorldSave>>,
    mut activity: ResMut<Activity>,
    mut survival: ResMut<Survival>,
    mut limits: ResMut<WalkerLimits>,
    readout: Option<Res<WalkingReadout>>,
    walkers: Query<(&Position, &LinearVelocity, &GroundState), With<Walker>>,
    config: Option<Res<WalkingConfig>>,
    sea: Option<crate::walking::Sea>,
    (air, contact, scene, frame, sun): AtPlayer,
    space: crate::SimSpace,
    fires: Option<Res<FiresNear>>,
    mut did: MessageWriter<crate::missions::Did>,
    mut watch: Local<Watch>,
) {
    let Some(mut save) = save else { return };
    let cfg = rules();
    let dt = time.delta_secs().clamp(0.0, 0.25);
    if let Some(line) = survival.line.as_mut() {
        line.left_s -= dt;
    }
    if survival.line.as_ref().is_some_and(|l| l.left_s <= 0.0) {
        survival.line = None;
    }
    survival.hit_s = (survival.hit_s - dt).max(0.0);
    if !save.mode().is_survival() {
        survival.vitals = None;
        *limits = WalkerLimits::default();
        *activity = Activity::default();
        return;
    }
    let o = space.origin();
    let walking = readout.as_deref().is_some_and(|r| r.active);
    let walker = walkers.single().ok();
    // The effort stands until the walker says otherwise; jumps are spent.
    let act = activity.clone();
    activity.jumps = 0;
    // ---- the conditions at the player
    let mut conditions = Conditions {
        effort: act.effort,
        head_in_water: act.head_in_water,
        in_deep_water: act.in_deep_water,
        // Not walking is flying: a craft's cabin.
        in_cabin: !walking,
        ..Conditions::default()
    };
    let mut wet_now = false;
    if let Some((position, velocity, ground)) = walker.filter(|_| walking) {
        let local = position.0;
        let up = o.up_at(local);
        let point = o.query_point(local);
        survival.at = Some(local);
        if let Some(air) = air.as_deref() {
            conditions.exposure = Exposure::from_sample(&air.now.sample(up));
        }
        conditions.exposure.in_water = act.head_in_water;
        conditions.thin_air = thin_air_at(o.radius_at(local));
        if let (Some(contact), Some(sun)) = (contact.as_deref(), sun.as_deref()) {
            let sun_dir = sun.clock.sun().as_dvec3();
            let head = point.as_dvec3() + up.as_dvec3() * 0.8;
            conditions.exposure.shaded = pbd_core::survival::temperature::shaded(
                &cfg.body,
                head,
                sun_dir,
                up.as_dvec3(),
                |p| solid_at(contact, p.as_vec3()),
            );
            conditions.exposure.sheltered = !contact.open_to_sky(point + up * 1.0);
            if conditions.exposure.sheltered {
                conditions.exposure.rain_on = false;
            }
        }
        if let Some(fires) = fires.as_deref() {
            conditions.exposure.fires = fires.0.clone();
        }
        conditions.exposure.worn = WornRecord::of(&save).worn();
        // ---- falls (design 3), read off the motion along up
        let vertical = velocity.0.dot(up);
        let wet = match (sea.as_ref(), config.as_deref()) {
            (Some(sea), Some(config)) => sea.state(config, point),
            _ => Default::default(),
        };
        wet_now = wet.body;
        let radius = o.radius_at(local);
        if !ground.grounded && !wet.body {
            watch.falling_mps = watch.falling_mps.max(-vertical);
            watch.air_top = Some(watch.air_top.map_or(radius, |top| top.max(radius)));
        }
        let gravity = match (scene.as_deref(), frame.as_deref()) {
            (Some(scene), Some(frame)) => scene
                .gravity_at(frame.0.origin + local.as_dvec3())
                .acceleration()
                .length() as f32,
            _ => 0.0,
        };
        let landed = if ground.grounded && !watch.was_grounded {
            Some(Landing::Ground)
        } else if wet.body && !watch.was_wet && !watch.was_grounded {
            let layers = (wet.depth / crate::planet::column::STEP_M).floor().max(0.0) as u32;
            Some(Landing::Water {
                depth_layers: layers,
            })
        } else {
            None
        };
        if let Some(landing) = landed {
            let dropped = watch.air_top.map_or(0.0, |top| (top - radius) as f32);
            let impact = fall::bounded_impact(watch.falling_mps, dropped, gravity);
            let pts = fall::fall_damage(&cfg.fall, impact, gravity, landing);
            if pts > 0.0 {
                hit(&mut save, &mut survival, Cause::Fall, pts);
            }
            watch.falling_mps = 0.0;
        }
        if ground.grounded || wet.body {
            watch.falling_mps = 0.0;
            watch.air_top = None;
        }
        // ---- climbs: the feet rising while they stand (a layer stepped up)
        if ground.grounded {
            if let Some(was) = watch.stood_at {
                let rise = (radius - was) as f32;
                if rise > 0.2 && rise < 3.0 {
                    watch.climbed_m += rise;
                }
            }
            watch.stood_at = Some(radius);
        } else {
            watch.stood_at = None;
        }
        watch.was_grounded = ground.grounded;
        watch.was_wet = wet.body;
    }
    conditions.exposure.wet = wet_now;
    // ---- bursts: jumps and climbs
    for _ in 0..act.jumps {
        save.survival_act(Act::Burst(Burst::Jump));
    }
    if watch.climbed_m >= 0.5 {
        let m = std::mem::take(&mut watch.climbed_m);
        save.survival_act(Act::Burst(Burst::Climb(m)));
    }
    // ---- the tick itself
    let Some(tick) = save.survive(&conditions, dt) else {
        survival.vitals = None;
        return;
    };
    survival.feel_c = tick.feel_c;
    let vitals = save.vitals();
    if let Some(v) = vitals.as_ref() {
        if let Some(was) = watch.body_c_was
            && dt > 0.0
        {
            let per_min = (v.body_c - was) / dt * 60.0;
            survival.trend_c_per_min += (per_min - survival.trend_c_per_min) * (dt * 0.5).min(1.0);
        }
        watch.body_c_was = Some(v.body_c);
        // Back to fine from hot or from cold (the missions' `cool`, `warm`).
        use pbd_core::survival::temperature::BodyState as B;
        let state = v.state(cfg);
        if state == B::Fine
            && let Some(was) = watch.state_was
        {
            use pbd_core::missions::Kind;
            match was {
                B::Hot | B::Exhaustion | B::Stroke => {
                    did.write(crate::missions::Did::new(Kind::Cool, "heat", 1));
                }
                B::Cold | B::Hypothermia => {
                    let by = if conditions.exposure.fires.is_empty() {
                        "other"
                    } else {
                        "fire"
                    };
                    did.write(crate::missions::Did::new(Kind::Warm, by, 1));
                }
                _ => {}
            }
        }
        if state != B::Cool {
            watch.state_was = Some(state);
        }
        survival.cause = cause_of(&conditions.exposure, v.body_c);
        let (stamina, mut control) = v.limits(cfg, act.in_deep_water);
        // Dead, the body lies where it fell until Wake up.
        if v.is_dead() {
            control.controls = false;
            control.walk_mult = 0.0;
        }
        *limits = WalkerLimits { stamina, control };
    }
    let mut took = 0.0;
    for (_, pts) in &tick.damage {
        took += pts;
    }
    if took >= 1.0 {
        survival.hit_s = HIT_S;
    }
    for event in tick.events {
        on_event(&mut survival, event);
    }
    survival.vitals = vitals;
}

/// A hit from outside the tick, through the server.
pub fn hit(save: &mut WorldSave, survival: &mut Survival, cause: Cause, pts: f32) {
    info!("SURVIVAL hit {cause:?} for {pts:.1}");
    if let Some(acted) = save.survival_act(Act::Hit(cause, pts)) {
        survival.hit_s = HIT_S;
        for event in acted.events {
            on_event(survival, event);
        }
        survival.vitals = save.vitals();
    }
}

fn on_event(survival: &mut Survival, event: Event) {
    match event {
        Event::Died(_) => {
            survival.dead = true;
            survival.died_now = true;
        }
        Event::Note(note) => survival.say(note.text()),
        _ => {}
    }
}

/// Whether a body-local point is in solid ground: the column tier's answer
/// where it has one, the height field's below its surface elsewhere. The
/// shade march's closure (design 13).
pub fn solid_at(contact: &crate::planet::PlanetContact, point: Vec3) -> bool {
    let direction = point.normalize_or(Vec3::Y);
    match contact.column_material(point) {
        Some(material) => material != pbd_core::terrain::Material::Air && !material.is_water(),
        None => point.length() < contact.sample(direction).radius,
    }
}

/// The fires near the player this frame (design 13), for [`tick`]: the
/// desktop fills it from the lit campfires and the torch in hand.
#[derive(Resource, Clone, Debug, Default)]
pub struct FiresNear(pub Vec<FireNear>);

/// The tick's set: what reads the tick's answer runs after it.
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct SurvivalTick;

/// Where a death wakes the player (design 8, decision 18): a laid
/// bedroll's spawn record, or `None` for the world's spawn.
pub const SPAWN_KIND: &str = "spawn";
pub const SPAWN_SCHEMA: u32 = 1;

/// Where a death wakes: the laid bedroll (`places::lay_bedroll`, task 6.3),
/// else a `spawn` record, else `None` for the world's spawn.
pub fn spawn_point(save: &WorldSave) -> Option<WorldPos> {
    if let Some(laid) = places::bedroll(save) {
        return laid.at;
    }
    save.records.get(SPAWN_KIND, 0).and_then(|r| r.read())
}

/// Set a spawn a death wakes at, under any laid bedroll.
pub fn set_spawn(save: &mut WorldSave, at: WorldPos) -> bool {
    let record = StoredRecord::of(SPAWN_KIND, 0, SPAWN_SCHEMA, &at);
    let kept = save
        .store(&pbd_core::records::Author::Player(0), vec![record])
        .is_some();
    if kept {
        save.note_record_kinds(&[(SPAWN_KIND, SPAWN_SCHEMA)]);
    }
    kept
}

/// Survival's resources and its tick.
pub struct SurvivalPlugin;

impl Plugin for SurvivalPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Activity>()
            .init_resource::<WalkerLimits>()
            .init_resource::<Survival>()
            .init_resource::<FiresNear>()
            .init_resource::<HeldBroken>()
            .add_systems(Update, tick.in_set(SurvivalTick).run_if(crate::world_open));
    }
}

pub mod places;

#[cfg(test)]
mod tests;
