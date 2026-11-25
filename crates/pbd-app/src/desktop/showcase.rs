//! `--showcase <name>` (`step-videos` 2.1, 2.2, 5.1, 5.4): a bot plays the
//! game toward the goal a shot list states, through the player's own input
//! path, in real time, so OBS can film it.
//!
//! What "the player's own input path" means here, concretely:
//! - keys and mouse buttons are pressed and released in Bevy's
//!   `ButtonInput`, in `PreUpdate` after the input systems and before any
//!   reader, which is where the physical keyboard's presses land;
//! - the look moves by the mouse's motion (`AccumulatedMouseMotion`), at a
//!   turn rate a hand would make, so the walker's own mouse-look turns it;
//! - the hotbar is picked with its number keys, the tool with G held and
//!   the wheel, a panel opened with its key (I, F, O) and shut with it;
//! - a panel's button is pressed as the pointer presses it: its
//!   `Interaction` goes to `Pressed` after the UI's focus pass, which is what
//!   the button's own handler reads.
//!
//! Nothing here sets an item, a count, a mission or a world state. The bot
//! reads the screen's state (what is aimed, what is carried, where the
//! animals are, the float's phase) the way a player looks at the screen.
//!
//! It prints `SHOWCASE_START <name>`, `SHOT <n> <caption>` at each shot,
//! and `SHOWCASE_END <name>` when its goal is met, then quits. A shot that
//! cannot finish fails the run: `SHOWCASE_STALL` and no `SHOWCASE_END`.
//!
//! locale: dev

pub mod control;
pub mod file;
pub mod path;

use super::digging::{self, Aim};
use super::menu::Screen;
use avian3d::prelude::Position;
use bevy::input::mouse::{AccumulatedMouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use file::{Act, Showcase, Until};
use pbd_app::planet::{PLANET_RADIUS, PlanetContact, PlanetFine, surface_height};
use pbd_app::saves::WorldSave;
use pbd_app::ui_kit::Localized;
use pbd_core::inventory::{CARRIED, Item, SLOTS, Tool};
use pbd_core::terrain::Material;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// The radians one pixel of mouse motion turns the look: the walker's own.
const PER_PIXEL: f32 = pbd_app::flight_view::MOUSE_LOOK_SENSITIVITY;
/// The fastest the bot turns, radians a second: a quick hand, not a snap.
const TURN_RATE: f32 = 2.6;
/// How close an aim has to be, radians.
const AIMED: f32 = 0.035;

/// The showcase being played, and the bot playing it.
#[derive(Resource)]
pub struct Player {
    pub name: String,
    pub file: Showcase,
    pub stills: Option<PathBuf>,
    bot: Bot,
}

impl Player {
    pub fn new(name: String, file: Showcase, stills: Option<PathBuf>) -> Self {
        Self {
            name,
            file,
            stills,
            bot: Bot::default(),
        }
    }

    /// The shot being played, counted from 1 (`play-frame-budget`'s
    /// frame-log column).
    pub fn shot(&self) -> usize {
        self.bot.shot + 1
    }

    /// Start at shot `n`, counted from 1.
    pub fn starting_at(mut self, n: usize) -> Self {
        self.bot.shot = n.saturating_sub(1);
        self
    }
}

/// What the game said happened, counted, so the bot knows a deed took (the
/// same messages the missions count).
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tally {
    dug: u32,
    placed: u32,
    cast: u32,
    caught: u32,
    tilled: u32,
    planted: u32,
    picked: u32,
    harvested: u32,
    collected: u32,
    tamed: u32,
    penned: u32,
    cooked: u32,
    ate: u32,
    gathered: u32,
}

/// The caption bar's text (`step-videos` 2.2).
#[derive(Component)]
pub struct CaptionText;

/// A panel control to press this frame, after the UI's focus pass.
#[derive(Clone, Debug, PartialEq)]
enum Click {
    /// A row of the pack's recipe list, by the hands' recipe index.
    Recipe(usize),
    CraftHand,
    Well(usize),
    CraftBench,
    Cook(String),
    Smelt(String),
    Cell(usize),
    /// A mission tree card, by its mission index.
    Card(usize),
    /// The pack's trash plaque.
    Trash,
}

/// One frame's inputs.
#[derive(Default)]
struct Intent {
    held: HashSet<KeyCode>,
    taps: Vec<KeyCode>,
    lmb: bool,
    lmb_tap: bool,
    rmb_tap: bool,
    look: Vec2,
    wheel: i32,
    click: Option<Click>,
    /// The pointer moved to this place in the window, logical pixels, as a
    /// player's hand moves the mouse over an open screen.
    pointer: Option<Vec2>,
}

/// What an act says after a frame.
enum Status {
    Running,
    Done,
    Fail(String),
}

/// Digging through what holds the walker up: the tool it had to put back,
/// the tool it is fetching (G and the wheel), and how long it has been at it.
#[derive(Clone, Copy, Debug)]
struct Through {
    was: Option<Tool>,
    fetching: Option<Option<Tool>>,
    fetch_s: f32,
    since: f32,
    block_s: f32,
}

/// A station farther than this is made again where the bot stands, m.
const STATION_NEAR_M: f32 = 25.0;

/// A plan being walked (`bot-path-planner` design 3).
struct PlanRun {
    plan: path::Plan,
    goal: Vec3,
    made: f32,
    step_since: f32,
    dig_since: f32,
    /// The tool in hand before the first dig, put back on arrival.
    was: Option<Tool>,
}

/// The live ground as the planner reads it: finest cells by index, blocks
/// by the fine set and the save's edits, a standing tree's trunk a block
/// never dug, unloaded ground never entered.
struct LiveGround<'a, 'b> {
    s: &'a Sight<'b>,
    axe_logs: u16,
    cache: std::cell::RefCell<HashMap<(usize, usize), path::Block>>,
    /// Cells the walker was held short of for this goal: never entered.
    avoid: &'a [usize],
}

impl path::Ground for LiveGround<'_, '_> {
    fn neighbours(&self, cell: usize) -> Vec<usize> {
        let records = self.s.fine.set.finest_records();
        let Some(r) = records.get(cell) else {
            return Vec::new();
        };
        self.s.fine.set.finest_neighbors[cell][..r.degree()]
            .iter()
            .map(|&n| n as usize)
            .collect()
    }

    fn centre(&self, cell: usize) -> Vec3 {
        let r = &self.s.fine.set.finest_records()[cell];
        Vec3::from_slice(&r.direction_height[..3]).normalize()
    }

    fn block(&self, cell: usize, layer: usize) -> path::Block {
        if self.avoid.contains(&cell) {
            return path::Block::Never;
        }
        if let Some(b) = self.cache.borrow().get(&(cell, layer)) {
            return *b;
        }
        let key = self.s.fine.set.finest_records()[cell].key();
        // The column by its record index (no search by key per block).
        let _ = key;
        let material = match self.s.fine.set.columns.column(cell) {
            Some(column) => Some(column.material(layer)),
            None => self
                .s
                .fine
                .set
                .adoptable(cell, &self.s.save.edits)
                .map(|column| column.material(layer)),
        };
        let b = match material {
            None => path::Block::Never,
            // The sea is not in the columns: open air under its level is the
            // sea, so nothing is dug or walked under it.
            Some(Material::Air)
                if pbd_core::column::layer_altitude(layer) + 0.5 < 0.0
                    && surface_height(path::Ground::centre(self, cell)) < 0.0 =>
            {
                path::Block::Water
            }
            Some(Material::Air) => {
                // A standing tree's trunk over its ground block.
                // Only the open block just over a cell's ground can hold a
                // trunk; the tree test searches the set, so it is asked
                // only there.
                if layer > 0
                    && path::Ground::block(self, cell, layer - 1).solid()
                    && digging::under_standing_tree(
                        self.s.fine,
                        self.s.save,
                        self.axe_logs,
                        (key, layer - 1),
                    )
                {
                    path::Block::Never
                } else {
                    path::Block::Open
                }
            }
            Some(m) if m.is_water() => path::Block::Water,
            Some(m) if m.is_lamp() || pbd_core::station::station_of(m).is_some() => {
                path::Block::Never
            }
            // A standing tree's own ground is not dug until it is felled.
            Some(_)
                if digging::under_standing_tree(
                    self.s.fine,
                    self.s.save,
                    self.axe_logs,
                    (key, layer),
                ) =>
            {
                path::Block::Never
            }
            Some(m) => path::Block::Dig(dig_cost(m)),
        };
        self.cache.borrow_mut().insert((cell, layer), b);
        b
    }
}

/// What digging a block costs the planner, metres of walking: earth is
/// quick with the shovel, stone slow with the pickaxe. Dear enough that a
/// walk round beats a tunnel unless the way round is much longer (seen
/// 2026-11-20: at 2 and 4 the bot tunnelled after a hen, 20 digs a plan,
/// and never tamed her).
fn dig_cost(m: Material) -> f32 {
    3.0 * match m {
        Material::Soil
        | Material::Grass
        | Material::Dirt
        | Material::Sand
        | Material::DryGrass
        | Material::JungleGrass
        | Material::Snow
        | Material::Tilled
        | Material::DesertSand => 2.0,
        Material::Stone | Material::Rock | Material::Ore | Material::Basalt => 3.3,
        _ => 2.5,
    }
}

/// What a bot is walking to or working on.
#[derive(Clone, Copy, Debug)]
enum Goal {
    Cell(u32),
    Tree(u32),
    Point(Vec3, Vec3),
}

#[derive(Default)]
struct Bot {
    started: Option<f32>,
    ended: Option<f32>,
    failed: bool,
    shot: usize,
    act: usize,
    shot_s: f32,
    act_s: f32,
    phase: u32,
    phase_s: f32,
    count: u32,
    tries: u32,
    mark: Tally,
    goal: Option<Goal>,
    used: HashSet<u32>,
    gave_up: HashSet<u32>,
    field: Vec<u32>,
    pen_centre: Option<Vec3>,
    pen_edges: Vec<(Vec3, Vec3)>,
    pieces: usize,
    sel: (u32, f32),
    /// Making room (`drop-and-trash` design 4): the step and its clock.
    room: (u32, f32),
    sel_from: usize,
    walk_anchor: Option<(Vec3, f32)>,
    /// The watchdog: the goal walked to, the nearest it came, and when.
    nearest: Option<(Vec3, f32, f32)>,
    detour_s: f32,
    roam_to: Option<Vec3>,
    climb_s: f32,
    /// Where an `Away` began, and the point it walks to next.
    away_from: Option<Vec3>,
    away_to: Option<Vec3>,
    /// The place `Mark` puts the map's mark on.
    mark_goal: Option<Vec3>,
    /// The piece of clutter `PickUp` is going for: its cell's key, kind and
    /// middle.
    clutter_goal: Option<(u32, pbd_core::scatter::Kind, Vec3)>,
    /// Pieces it gave up on, by cell key and kind.
    clutter_gave_up: HashSet<(u32, pbd_core::scatter::Kind)>,
    /// Where the bot last got stuck, and until when it builds itself up.
    stuck_at: Option<Vec3>,
    climb_until: f32,
    fail_frames: u32,
    /// Where the bot started: what it roams round before it has a bench.
    home: Option<Vec3>,
    roam_s: f32,
    /// The first cast's water and stand, for casting round it.
    fish_from: Option<(Vec3, Vec3)>,
    /// The tops of the cells the bot dug holes in, to walk round.
    holes: Vec<Vec3>,
    look_goal: Option<(Vec3, f32, f32)>,
    index: Option<(u64, HashMap<u32, usize>)>,
    released: Vec<KeyCode>,
    lmb_down: bool,
    /// The walk being followed (`bot-path-planner`).
    plan: Option<PlanRun>,
    /// Cells the walker could not get into on the way to `plan_avoid_for`
    /// (an invisible wall: the collision and the blocks disagreeing).
    plan_avoid: Vec<usize>,
    /// Going up and out of a hole (`bot-path-planner` design 5.1): until the
    /// sky is open over the head.
    climbing_out: bool,
    /// Up to this altitude, m, when the watchdog sends it up: the goal's
    /// height, so a pit under a tree on its rim is climbed out of too.
    climb_to: Option<f32>,
    climb_out_s: f32,
    plan_avoid_for: Option<Vec3>,
    rmb_down: bool,
    tap_lmb_down: bool,
    still: u32,
    line_was_out: bool,
    bite_s: f32,
    /// `Fish`: how long to hold the cast, s, chosen so the float lands on
    /// water; and how many bearings it has tried from this stand.
    cast_hold: f32,
    cast_turns: u32,
    /// `Dig` for bait: only earth the game calls by the water.
    want_wet: bool,
    /// `Fresh`: the shore sought is a river's or a lake's, not the sea's,
    /// for this act and the `Fish` after it, until a `Shore`.
    fresh: bool,
    /// Live control (`bot-control`): paused, presses nothing; acts queued
    /// by `do`, run before the shot's own.
    paused: bool,
    injected: std::collections::VecDeque<file::Act>,
    /// The shot file's last change seen, and when it was last looked at:
    /// an edit is played at once (`bot-control` hot reload).
    file_seen: Option<std::time::SystemTime>,
    file_checked: f32,
    /// Seconds the feet have made no headway while walking.
    stuck_s: f32,
    /// A tool being fetched by G and the wheel, and for how long.
    fetching: Option<(Option<Tool>, f32)>,
    /// Digging its way through, when stuck too long (owner, 2026-11-19:
    /// "point towards where you want to go, and equip the right tool, shovel
    /// or pick axe to mine your way").
    through: Option<Through>,
    /// `Mine`: the heading the stair goes down along, and the layer the
    /// walker's feet started on.
    mine_dir: Option<Vec3>,
    mine_from: Option<usize>,
    /// The hex route being walked (`mission-playthrough` 3.7), the goal it
    /// was planned to, and when.
    route: Option<(pbd_app::river_walk::HexRoute, Vec3, f32)>,
    /// Where the feet were, newest last, for "moved in the last second".
    trail: Vec<(f32, Vec3)>,
    /// Cells the walker could not get into, by centre: planned round.
    route_block: Vec<Vec3>,
    /// Blocked steps on the way to the current target: only after a few is
    /// the target given up as out of reach.
    blocked_here: u32,
    /// `Mine`: the ore block it is cutting out of the wall, and the ones it
    /// could not get the crosshair on.
    ore_goal: Option<(u32, usize)>,
    ore_gave_up: HashSet<(u32, usize)>,
    /// An act's moment worth a still, taken after its frame.
    want_still: bool,
    /// The acts that have asked for a still this run, by kind.
    stilled: HashSet<&'static str>,
}

/// Everything the bot looks at, borrowed for a frame.
struct Sight<'a> {
    now: f32,
    dt: f32,
    feet: Vec3,
    up: Vec3,
    eye: Vec3,
    look: Vec3,
    heading: Vec3,
    pitch: f32,
    fine: &'a PlanetFine,
    contact: &'a PlanetContact,
    save: &'a WorldSave,
    slots: &'a pbd_core::inventory::Slots,
    tools: &'a pbd_core::inventory::Equipment,
    aim: Option<pbd_core::aim::Target>,
    screen: Screen,
    fishery: Option<&'a pbd_app::fish::Fishery>,
    yard: Option<&'a pbd_app::husbandry::Farmyard>,
    /// The local origin, for the yard's local feet as planet points.
    origin: pbd_core::world_frame::SimOrigin,
    missions: &'a pbd_app::missions::Missions,
    chain: &'a pbd_core::missions::Chain,
    recipes: &'a pbd_app::config::Recipes,
    land: &'a [pbd_core::husbandry::LandSpecies],
    farm: &'a pbd_app::config::FarmConfig,
    gather: &'a pbd_app::config::GatherConfig,
    picker: Option<Tool>,
    picking: bool,
    tally: Tally,
    hour: f32,
    /// The map's view and the window's size, for pointing at the map.
    map_view: Option<super::map_screen::MapView>,
    window: Option<Vec2>,
    /// The ground clutter's odds and the world's day (`gather-by-hand`).
    odds: Option<pbd_core::scatter::Odds>,
    day: f64,
    /// Each mission card's middle on screen, logical pixels, by its place in
    /// the chain.
    cards: Vec<(usize, Vec2)>,
    /// The rod's cast: charge time, speeds and lift (`fauna.ron`).
    fishing: &'a pbd_core::fauna::FishingSettings,
}

fn tangent(v: Vec3, up: Vec3) -> Vec3 {
    v - up * v.dot(up)
}

fn signed_angle(from: Vec3, to: Vec3, up: Vec3) -> f32 {
    up.dot(from.cross(to)).atan2(from.dot(to))
}

fn tool_named(name: &str) -> Option<Option<Tool>> {
    Some(match name {
        "hand" => None,
        "rod" => Some(Tool::Rod),
        "shovel" => Some(Tool::Shovel),
        "pickaxe" => Some(Tool::Pickaxe),
        "axe" => Some(Tool::Axe),
        _ => return None,
    })
}

/// Whether `item` is what a name asks for: an item's code, or `seed` for
/// any seed, `fish` for any fish, `dish` for anything cooked.
fn wanted(name: &str, item: Item) -> bool {
    match (name, item) {
        ("fish", Item::Fish(_)) => true,
        ("seed", Item::Good(good)) => pbd_core::farming::Crop::of_seed(good).is_some(),
        _ => pbd_core::station::item_named(name) == Some(item),
    }
}

fn carried(slots: &pbd_core::inventory::Slots, name: &str) -> u32 {
    (0..CARRIED)
        .filter_map(|k| slots.get(k))
        .filter(|s| wanted(name, s.item))
        .map(|s| u32::from(s.count))
        .sum()
}

const DIGITS: [KeyCode; SLOTS] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
    KeyCode::Digit0,
];

impl Bot {
    fn record<'a>(&mut self, s: &Sight<'a>, key: u32) -> Option<&'a pbd_app::planet::GpuCell> {
        let version = s.fine.version;
        if self.index.as_ref().is_none_or(|(v, _)| *v != version) {
            let map = s
                .fine
                .set
                .finest_records()
                .iter()
                .enumerate()
                .map(|(i, r)| (r.key(), i))
                .collect();
            self.index = Some((version, map));
        }
        let i = *self.index.as_ref()?.1.get(&key)?;
        s.fine.set.finest_records().get(i)
    }

    fn top_of(&mut self, s: &Sight, key: u32) -> Option<Vec3> {
        let r = self.record(s, key)?;
        let d = Vec3::from_slice(&r.direction_height[..3]).normalize_or(s.up);
        Some(d * (PLANET_RADIUS + r.direction_height[3]))
    }

    /// Turn the look toward a direction and pitch, smoothly. Whether it is
    /// there.
    fn steer(&mut self, s: &Sight, out: &mut Intent, toward: Vec3, pitch: f32) -> bool {
        let want = tangent(toward, s.up).normalize_or(s.heading);
        let yaw = signed_angle(s.heading, want, s.up);
        let dp = pitch.clamp(-1.45, 1.45) - s.pitch;
        let ease = (s.dt * 7.0).min(1.0);
        let max = TURN_RATE * s.dt;
        let step_yaw = (yaw * ease).clamp(-max, max);
        let step_pitch = (dp * ease).clamp(-max, max);
        out.look = Vec2::new(-step_yaw / PER_PIXEL, -step_pitch / PER_PIXEL);
        yaw.abs() < AIMED && dp.abs() < AIMED
    }

    /// Look at a point.
    fn aim_at(&mut self, s: &Sight, out: &mut Intent, point: Vec3) -> bool {
        let d = (point - s.eye).normalize_or(s.look);
        let pitch = d.dot(s.up).clamp(-1.0, 1.0).asin();
        self.steer(s, out, d, pitch)
    }

    /// Walk toward a point on the ground, hopping what stalls it. Whether it
    /// has arrived; `stuck` after seconds of not getting anywhere.
    fn walk_to(&mut self, s: &Sight, out: &mut Intent, point: Vec3, within: f32) -> Walked {
        let to = tangent(point - s.feet, s.up);
        let dist = to.length();
        if dist < within {
            self.walk_anchor = None;
            self.route = None;
            self.nearest = None;
            return Walked::There;
        }
        // The watchdog: a minute and not three metres from where it was,
        // whatever it walks to (the same thing tried over and over, a new
        // tree each time, tools swapped back and forth): up and out, and
        // every plan and avoided cell forgotten.
        match self.nearest {
            Some((anchor, _, since)) if tangent(s.feet - anchor, s.up).length() < 3.0 => {
                if s.now - since > 60.0 && !self.climbing_out {
                    warn!(
                        "showcase: not three metres in a minute ({dist:.1} m from the goal); up and out, then plan again"
                    );
                    self.climbing_out = true;
                    self.climb_to = Some(point.length() - PLANET_RADIUS);
                    self.plan = None;
                    self.plan_avoid.clear();
                    self.through = None;
                    self.route = None;
                    self.nearest = Some((s.feet, dist, s.now));
                }
            }
            _ => self.nearest = Some((s.feet, dist, s.now)),
        }
        // No headway for a while: face the way and dig through.
        self.trail.push((s.now, s.feet.normalize_or(s.up)));
        let keep = s.now - 3.0;
        self.trail.retain(|(t, _)| *t >= keep);
        let headway = pbd_app::river_walk::moved_in_last_second(&self.trail);
        if headway.is_some_and(|m| m < 0.5) {
            self.stuck_s += s.dt;
        } else {
            self.stuck_s = 0.0;
        }
        // Stuck under ground: up and out first, a block under the feet at a
        // time (owner, 2026-11-20: "dig straight up and place dirt blocks
        // below feet").
        if (self.climbing_out || self.stuck_s > 6.0) && self.up_and_out(s, out) {
            return Walked::Going;
        }
        // Under water (a flooded hole, off a bank): up and on toward the
        // point, as a player swims out.
        if s.eye.length() - PLANET_RADIUS < -0.2 && surface_height(s.up) < 0.5 {
            out.held.insert(KeyCode::Space);
            out.held.insert(KeyCode::KeyW);
            self.steer(s, out, to, 0.3);
            if (s.now % 3.0) < s.dt {
                info!("showcase: under water; swimming up and on");
            }
            return Walked::Going;
        }
        // The planned walk over cells and layers (`bot-path-planner`); the
        // older heuristics below only when no plan is found.
        if let Some(walked) = self.follow_plan(s, out, point) {
            return walked;
        }
        if (self.through.is_some() || self.stuck_s > 4.0) && self.dig_through(s, out, point) {
            return Walked::Going;
        }
        if !self.in_pit(s)
            && s.now >= self.climb_until
            && let Some(walked) = self.follow_route(s, out, point)
        {
            return walked;
        }
        if self.detour_s > 0.0 {
            self.detour_s -= s.dt;
            out.held.insert(KeyCode::KeyW);
            let right = s.heading.cross(s.up);
            self.steer(s, out, s.heading + right * 0.02, -0.1);
            return Walked::Going;
        }
        // Round the nearest hole the bot dug that lies on the way.
        let dir = to / dist;
        let blocking = self
            .holes
            .iter()
            .filter_map(|h| {
                let rel = tangent(*h - s.feet, s.up);
                let along = rel.dot(dir);
                let lateral = rel - dir * along;
                (along > 0.3 && along < dist && lateral.length() < 1.9)
                    .then_some((along, *h, lateral))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0));
        let to = match blocking {
            Some((_, hole, lateral)) => {
                let side = if lateral.length() > 0.05 {
                    -lateral.normalize()
                } else {
                    dir.cross(s.up)
                };
                tangent(hole + side * 2.8 - s.feet, s.up)
            }
            None => to,
        };
        let facing = self.steer(s, out, to, -0.12);
        let yaw = signed_angle(s.heading, to.normalize_or(s.heading), s.up);
        let _ = facing;
        if yaw.abs() < 0.6 {
            out.held.insert(KeyCode::KeyW);
            if dist > 18.0 {
                out.held.insert(KeyCode::ShiftLeft);
            }
        }
        // In a hole of its own digging: up out of it, as a player does,
        // a block set under the feet at the top of each jump.
        // In the sea (roamed off the shore): up and back to dry land.
        if surface_height(s.up) < -0.3 && (point.length() - PLANET_RADIUS) > 0.0 {
            out.taps.push(KeyCode::Space);
        }
        // Or stuck twice over in the same spot: a trench of its own digging
        // the pit test misses. Up a few blocks, then on.
        if self.in_pit(s) || s.now < self.climb_until {
            self.climb(s, out);
            return Walked::Going;
        }
        self.climb_s = 0.0;
        let (anchor, since) = *self.walk_anchor.get_or_insert((s.feet, s.now));
        // Ground covered, not a jump up and down.
        if tangent(s.feet - anchor, s.up).length() > 0.8 {
            self.walk_anchor = Some((s.feet, s.now));
        } else {
            let stalled = s.now - since;
            if stalled > 0.8 && (stalled % 0.8) < s.dt * 1.5 {
                out.taps.push(KeyCode::Space);
            }
            if stalled > 4.0 {
                // Round it: a quarter turn and a few steps.
                self.walk_anchor = Some((s.feet, s.now));
                self.detour_s = 1.5;
                let here = self
                    .stuck_at
                    .filter(|p| tangent(*p - s.feet, s.up).length() < 2.0);
                if here.is_some() {
                    self.climb_until = s.now + 4.5;
                    self.stuck_at = None;
                } else {
                    self.stuck_at = Some(s.feet);
                }
                return Walked::Stuck;
            }
        }
        Walked::Going
    }

    /// Up and out of a hole (`bot-path-planner` design 5.1, `place-underfoot`):
    /// the block over the head dug if solid, then a block placed under the
    /// feet (the game lifts the player onto it), until no solid block stands
    /// over the head within eight metres. Whether it is still at it.
    fn up_and_out(&mut self, s: &Sight, out: &mut Intent) -> bool {
        use path::Ground as _;
        let records = s.fine.set.finest_records();
        let Some(here) = s
            .contact
            .finest_key(s.up)
            .and_then(|key| records.iter().position(|r| r.key() == key))
        else {
            self.climbing_out = false;
            return false;
        };
        let ground = LiveGround {
            s,
            axe_logs: s.gather.0.axe_logs,
            cache: Default::default(),
            avoid: &[],
        };
        let altitude = s.feet.length() - PLANET_RADIUS;
        let Some(feet) = pbd_core::column::layer_at(altitude + 0.2) else {
            self.climbing_out = false;
            return false;
        };
        let covered = (feet + 2..feet + 10).any(|l| ground.block(here, l).solid());
        // An open pit: every neighbour solid at the chest, nowhere to step.
        let walled = ground
            .neighbours(here)
            .into_iter()
            .all(|n| ground.block(n, feet + 1).solid());
        // Sent up by the watchdog: until the feet are at the goal's height.
        let below = self.climb_to.is_some_and(|to| altitude < to - 0.5);
        let covered = covered || walled || below;
        if !covered {
            if self.climbing_out {
                info!("showcase: up and out: open sky over the head");
            }
            self.climbing_out = false;
            self.climb_to = None;
            self.climb_out_s = 0.0;
            return false;
        }
        if !self.climbing_out {
            info!("showcase: up and out: stuck under ground; building up");
            self.climbing_out = true;
            self.climb_out_s = 0.0;
        }
        self.climb_out_s += s.dt;
        // Ninety seconds, and five more for every metre still to climb.
        let to_climb = self.climb_to.map_or(0.0, |to| (to - altitude).max(0.0));
        if self.climb_out_s > 90.0 + 5.0 * to_climb {
            warn!("showcase: up and out: gave up after 90 s");
            self.climbing_out = false;
            self.climb_to = None;
            self.climb_out_s = 0.0;
            self.stuck_s = 0.0;
            return false;
        }
        let centre = Vec3::from_slice(&records[here].direction_height[..3]).normalize();
        let key = records[here].key();
        // The head room first: the block over the head, dug.
        let over = feet + 2;
        if let path::Block::Dig(_) = ground.block(here, over) {
            let material =
                digging::material_of(s.fine, s.save, (key, over)).unwrap_or(Material::Soil);
            if self.fetch_tool(s, out, Self::tool_for(material)) {
                return true;
            }
            let at = centre * (PLANET_RADIUS + pbd_core::column::layer_altitude(over) + 0.5);
            let aimed = self.aim_at(s, out, at);
            out.lmb = aimed || self.lmb_down;
            return true;
        }
        if ground.block(here, over) == path::Block::Never {
            // A station or unloaded ground over the head: not this way.
            self.climbing_out = false;
            return false;
        }
        // Then a block under the feet: bare hands, a block in the slot,
        // the look straight down.
        if self.fetch_tool(s, out, None) {
            return true;
        }
        let block = |item: Item| {
            matches!(
                item,
                Item::Block(
                    Material::Soil
                        | Material::Dirt
                        | Material::Grass
                        | Material::Stone
                        | Material::Sand
                        | Material::DryGrass
                )
            )
        };
        match self.select(s, out, &block) {
            Status::Running => return true,
            Status::Fail(why) => {
                warn!("showcase: up and out: no block to stand on ({why})");
                self.climbing_out = false;
                return false;
            }
            Status::Done => {}
        }
        let below = centre * (PLANET_RADIUS + pbd_core::column::layer_altitude(feet) - 0.05);
        let aimed = self.aim_at(s, out, below);
        if aimed && (s.now % 0.5) < s.dt * 1.5 {
            out.rmb_tap = true;
        }
        true
    }

    /// Follow a plan to `point` over cells and layers (`bot-path-planner`
    /// design 3): each step's blocks dug with their own tool, then the step
    /// walked into; planned again when the goal moves, a leg ends, a step is
    /// not reached in 6 s or a block will not break in 8 s. `None` when no
    /// plan is found, or the plan has arrived (the last metre is walked
    /// straight at the point).
    fn follow_plan(&mut self, s: &Sight, out: &mut Intent, point: Vec3) -> Option<Walked> {
        // Cells by key: the contact tier holds its own (maybe older) set, so
        // its indices are not the fine set's; a key is the same in both.
        let records = s.fine.set.finest_records();
        let index_of = |key: u32| records.iter().position(|r| r.key() == key);
        let Some(here) = s.contact.finest_key(s.up).and_then(index_of) else {
            if (s.now % 5.0) < s.dt {
                info!("showcase: plan: the feet's cell is not in the fine set");
            }
            return None;
        };
        let Some(goal_cell) = s
            .contact
            .finest_key(point.normalize_or(s.up))
            .and_then(index_of)
        else {
            if (s.now % 5.0) < s.dt {
                info!("showcase: plan: the goal's cell is not loaded");
            }
            return None;
        };
        if self.plan_avoid_for.is_none_or(|g| g.distance(point) > 1.5) {
            self.plan_avoid.clear();
            self.plan_avoid_for = Some(point);
        }
        let avoid = std::mem::take(&mut self.plan_avoid);
        let walked = self.follow_plan_with(s, out, point, here, goal_cell, &avoid);
        if self.plan_avoid.is_empty() {
            self.plan_avoid = avoid;
        } else {
            let mut more = avoid;
            more.append(&mut self.plan_avoid);
            self.plan_avoid = more;
        }
        walked
    }

    fn follow_plan_with(
        &mut self,
        s: &Sight,
        out: &mut Intent,
        point: Vec3,
        here: usize,
        goal_cell: usize,
        avoid: &[usize],
    ) -> Option<Walked> {
        use path::Ground as _;
        let altitude = s.feet.length() - PLANET_RADIUS;
        let ground = LiveGround {
            s,
            axe_logs: s.gather.0.axe_logs,
            cache: Default::default(),
            avoid,
        };
        // The layer the feet stand in: the one over the first solid block
        // under them (the walker's feet ride a little over the block's top,
        // and the layer of the raw altitude can be air over air).
        let mut feet = pbd_core::column::layer_at(altitude + 0.2)?;
        // Standing on the rim of a cell beside its own (a narrow trench, a
        // step's edge): the plan starts from the cell that bears the feet,
        // the nearest neighbour with ground at the feet.
        let mut here = here;
        if feet > 0 && !ground.block(here, feet - 1).solid() {
            let bearer = ground
                .neighbours(here)
                .into_iter()
                .filter(|&n| ground.block(n, feet - 1).solid() && !ground.block(n, feet).solid())
                .max_by(|&a, &b| {
                    ground
                        .centre(a)
                        .dot(s.up)
                        .total_cmp(&ground.centre(b).dot(s.up))
                });
            if let Some(n) = bearer {
                here = n;
            }
        }
        for _ in 0..3 {
            if feet == 0 || ground.block(here, feet - 1).solid() {
                break;
            }
            feet -= 1;
        }
        let stale = self.plan.as_ref().is_none_or(|run| {
            run.goal.distance(point) > 1.5
                || (run.plan.steps.is_empty() && !run.plan.arrives)
                || s.now - run.step_since > 6.0
                || run.dig_since > 8.0
        });
        if stale {
            if self.plan.as_ref().is_some_and(|run| s.now - run.made < 0.5) {
                return Some(Walked::Going);
            }
            let was = self
                .plan
                .as_ref()
                .filter(|run| run.goal.distance(point) <= 1.5)
                .map_or(s.tools.held(), |run| run.was);
            let Some(plan) = path::plan(&ground, here, feet, goal_cell, PLANET_RADIUS) else {
                info!("showcase: plan: no plan from {here}/{feet} to {goal_cell}");
                self.plan = None;
                return None;
            };
            info!(
                "showcase: planned {} steps ({} digs){}",
                plan.steps.len(),
                plan.steps.iter().map(|st| st.digs.len()).sum::<usize>(),
                if plan.arrives { "" } else { ", a leg" }
            );
            // Nowhere to go from here (every way round avoided): forget the
            // avoided cells and let the plain walk try this frame.
            if plan.steps.is_empty() && !plan.arrives {
                self.plan_avoid.clear();
                self.plan = None;
                return None;
            }
            if let Some(first) = plan.steps.first() {
                let d = ground.centre(first.cell);
                info!(
                    "showcase: plan: first step layer {} (feet {feet}, here {here}) {:.1} m off, digs {:?}; there {:?} / {:?} / {:?}; feet column here {:?} / {:?}",
                    first.layer,
                    tangent(
                        d * PLANET_RADIUS - s.feet.normalize_or(s.up) * PLANET_RADIUS,
                        s.up
                    )
                    .length(),
                    first.digs,
                    ground.block(first.cell, first.layer.saturating_sub(1)),
                    ground.block(first.cell, first.layer),
                    ground.block(first.cell, first.layer + 1),
                    ground.block(here, feet.saturating_sub(1)),
                    ground.block(here, feet),
                );
            }
            self.plan = Some(PlanRun {
                plan,
                goal: point,
                made: s.now,
                step_since: s.now,
                dig_since: 0.0,
                was,
            });
        }
        let mut run = self.plan.take()?;
        // Steps already stood in, their digs done, are done.
        while run.plan.steps.first().is_some_and(|st| {
            // In its cell, or within 0.6 m of its middle (the feet on a
            // cell's edge read as the neighbour's).
            (st.cell == here
                || tangent(
                    ground.centre(st.cell) * PLANET_RADIUS
                        - s.feet.normalize_or(s.up) * PLANET_RADIUS,
                    s.up,
                )
                .length()
                    < 0.6)
                && st.layer.abs_diff(feet) <= 1
                && st
                    .digs
                    .iter()
                    .all(|&(c, l)| !matches!(ground.block(c, l), path::Block::Dig(_)))
        }) {
            run.plan.steps.remove(0);
            run.step_since = s.now;
        }
        let Some(step) = run.plan.steps.first().cloned() else {
            let arrived = run.plan.arrives;
            // Arrived: the act's own tool back in hand, then the last metre
            // straight at the point.
            if arrived && s.tools.held() != run.was && self.fetch_tool(s, out, run.was) {
                self.plan = Some(run);
                return Some(Walked::Going);
            }
            self.plan = Some(run);
            return if arrived { None } else { Some(Walked::Going) };
        };
        let records = s.fine.set.finest_records();
        let centre = |c: usize| Vec3::from_slice(&records[c].direction_height[..3]).normalize();
        let solid_dig = step
            .digs
            .iter()
            .copied()
            .find(|&(c, l)| matches!(ground.block(c, l), path::Block::Dig(_)));
        // A block out of reach: walked toward first (a step behind, or the
        // walker short of where the plan began).
        let solid_dig = solid_dig.filter(|&(c, l)| {
            let at = centre(c) * (PLANET_RADIUS + pbd_core::column::layer_altitude(l) + 0.5);
            if at.distance(s.eye) <= 3.5 {
                return true;
            }
            let to = tangent(at - s.feet, s.up);
            self.steer(s, out, to, -0.12);
            out.held.insert(KeyCode::KeyW);
            false
        });
        if solid_dig.is_none()
            && step
                .digs
                .iter()
                .any(|&(c, l)| matches!(ground.block(c, l), path::Block::Dig(_)))
        {
            self.plan = Some(run);
            return Some(Walked::Going);
        }
        if let Some((cell, layer)) = solid_dig {
            let key = records[cell].key();
            let planned =
                digging::material_of(s.fine, s.save, (key, layer)).unwrap_or(Material::Soil);
            // A block in front of the planned one, met by the look for a
            // second: it is in the way, and dug first with its own tool
            // (seen 2026-11-20: stone in front of the planned stone).
            let blocker = self
                .aimed_cell(s)
                .filter(|&c| c != (key, layer) && run.dig_since > 1.0)
                .and_then(|c| digging::material_of(s.fine, s.save, c))
                .filter(|m| {
                    m != &Material::Air
                        && !m.is_water()
                        && !m.is_lamp()
                        && pbd_core::station::station_of(*m).is_none()
                });
            let material = blocker.unwrap_or(planned);
            let want = Self::tool_for(material);
            if self.fetch_tool(s, out, want) {
                self.plan = Some(run);
                return Some(Walked::Going);
            }
            let at = centre(cell) * (PLANET_RADIUS + pbd_core::column::layer_altitude(layer) + 0.5);
            let aimed = self.aim_at(s, out, at);
            let on = self.aimed_cell(s) == Some((key, layer)) || blocker.is_some();
            out.lmb = on && (aimed || self.lmb_down || blocker.is_some());
            run.dig_since += s.dt;
            run.step_since = s.now;
            if (run.dig_since % 2.0) < s.dt {
                info!("showcase: plan: digging {material:?} at {key}/{layer} (on {on})");
            }
            self.plan = Some(run);
            return Some(Walked::Going);
        }
        run.dig_since = 0.0;
        let target =
            centre(step.cell) * (PLANET_RADIUS + pbd_core::column::layer_altitude(step.layer));
        let to = tangent(target - s.feet, s.up);
        // More than a cell off (the feet on the far side of their own cell):
        // to the middle of the cell first, so the walk does not cut a corner
        // of a taller neighbour.
        let to_middle = tangent(
            centre(here) * PLANET_RADIUS - s.feet.normalize_or(s.up) * PLANET_RADIUS,
            s.up,
        );
        let to = if to.length() > 3.2 && to_middle.length() > 1.2 {
            to_middle
        } else {
            to
        };
        let to = if to.length() < 0.2 { s.heading } else { to };
        self.steer(s, out, to, -0.12);
        let yaw = signed_angle(s.heading, to.normalize_or(s.heading), s.up);
        if yaw.abs() < 0.7 {
            out.held.insert(KeyCode::KeyW);
        }
        if step.layer > feet
            && (to.length() < 3.0 || self.stuck_s > 0.5)
            && (s.now % 0.6) < s.dt * 1.5
        {
            out.taps.push(KeyCode::Space);
        }
        if self.stuck_s > 3.0 && step.cell != goal_cell {
            info!(
                "showcase: plan: held short of {} for {:.1} s; planning round it",
                step.cell, self.stuck_s
            );
            self.plan_avoid.push(step.cell);
            self.stuck_s = 0.0;
            self.plan = None;
            return Some(Walked::Going);
        }
        if (s.now % 2.0) < s.dt {
            info!(
                "showcase: plan: walking to {}/{} {:.1} m off, yaw {yaw:.2}, feet {feet}, stuck {:.1} s, W {}",
                step.cell,
                step.layer,
                to.length(),
                self.stuck_s,
                out.held.contains(&KeyCode::KeyW)
            );
        }
        self.plan = Some(run);
        Some(Walked::Going)
    }

    /// Walk the hex route to `point` (`mission-playthrough` 3.7): planned
    /// cell by cell on the live ground under the walker's own step rule,
    /// keeping to dry ground, round standing trees, the bot's own pits and
    /// the stations; planned again when the goal moves, the route runs out
    /// or the feet stop getting anywhere. `None` when no route is found, and
    /// the straight walk takes over.
    fn follow_route(&mut self, s: &Sight, out: &mut Intent, point: Vec3) -> Option<Walked> {
        use pbd_app::river_walk::{HexRoute, moved_in_last_second};
        let moved = moved_in_last_second(&self.trail);
        let stuck = moved.is_some_and(|m| m < 0.6)
            && self
                .route
                .as_ref()
                .is_some_and(|(_, _, at)| s.now - at > 1.5);
        if stuck && let Some((route, _, _)) = &self.route {
            // The cell it could not get into: round it next time.
            let next = route.cells[route.next];
            info!(
                "showcase: route blocked at a cell {:.1} m off; planning round it",
                tangent(
                    next * PLANET_RADIUS - s.feet.normalize_or(s.up) * PLANET_RADIUS,
                    s.up
                )
                .length()
            );
            self.route_block.push(next);
            self.blocked_here += 1;
            if self.route_block.len() > 24 {
                self.route_block.remove(0);
            }
            out.taps.push(KeyCode::Space);
        }
        // A route that has reached the target's own cell is done: the last
        // metre is walked straight at the target, never planned again.
        let in_last = |route: &HexRoute| route.next + 1 >= route.cells.len();
        let replan = stuck
            || self.route.as_ref().is_none_or(|(route, goal, at)| {
                goal.distance(point) > 1.5
                    || (in_last(route)
                        && tangent(point - s.feet, s.up).length() > 4.0
                        && s.now - at > 0.5)
            });
        if replan
            && self
                .route
                .as_ref()
                .is_none_or(|(_, _, at)| s.now - at > 0.5)
        {
            let records = s.fine.set.finest_records();
            let holes = self.holes.clone();
            let block = self.route_block.clone();
            let axe_logs = s.gather.0.axe_logs;
            let goal_dir = point.normalize_or(s.up);
            let avoid = |d: Vec3| {
                if d.dot(goal_dir) > 1.0 - 1e-9 {
                    return false;
                }
                if block
                    .iter()
                    .any(|b| b.dot(d) > 1.0 - (0.4 / PLANET_RADIUS).powi(2))
                {
                    return true;
                }
                let Some(r) = s.contact.finest_cell(d).and_then(|i| records.get(i)) else {
                    return false;
                };
                let top = d * (PLANET_RADIUS + r.direction_height[3]);
                if holes.iter().any(|h| h.distance(top) < 1.2) {
                    return true;
                }
                let key = r.key();
                if s.save
                    .edits
                    .for_cell(key)
                    .iter()
                    .any(|&(_, m)| pbd_core::station::station_of(m).is_some())
                {
                    return true;
                }
                pbd_core::column::layer_at(r.direction_height[3] - 0.5).is_some_and(|layer| {
                    digging::under_standing_tree(s.fine, s.save, axe_logs, (key, layer))
                })
            };
            let planned = HexRoute::plan_where(
                s.feet.normalize_or(s.up),
                goal_dir,
                Some(s.contact),
                true,
                &avoid,
            );
            match planned {
                Some(route) => {
                    if self
                        .route
                        .as_ref()
                        .is_none_or(|(_, goal, _)| goal.distance(point) > 1.5)
                    {
                        self.blocked_here = 0;
                    }
                    self.route = Some((route, point, s.now));
                }
                None => {
                    self.route = None;
                    return None;
                }
            }
        }
        let (route, _, _) = self.route.as_mut()?;
        let aim = route.aim(s.feet);
        let rise = route.rise_ahead();
        let left = route.left_m();
        let at = aim * (PLANET_RADIUS + route.heights[route.next].max(0.0));
        // On the last cell, or with the cell's middle underfoot: straight at
        // the target itself, not at a cell's middle the target is beside.
        let cell = tangent(at - s.feet, s.up);
        let to = if in_last(route) || cell.length() < 0.6 {
            tangent(point - s.feet, s.up)
        } else {
            cell
        };
        // Nothing to turn toward right under the feet: keep the heading.
        let to = if to.length() < 0.3 { s.heading } else { to };
        self.steer(s, out, to, -0.12);
        let yaw = signed_angle(s.heading, to.normalize_or(s.heading), s.up);
        if yaw.abs() < 0.7 {
            out.held.insert(KeyCode::KeyW);
            if left > 18.0 {
                out.held.insert(KeyCode::ShiftLeft);
            }
        }
        // A two-layer rise ahead, close: jump it.
        if rise > 0.6 && to.length() < 2.2 {
            out.taps.push(KeyCode::Space);
        }
        // A blocked step is planned round; four on the way to one target
        // and it is out of reach.
        if stuck && self.blocked_here >= 4 {
            self.blocked_here = 0;
            return Some(Walked::Stuck);
        }
        Some(Walked::Going)
    }

    /// Dig a way through toward `point`: face it, take the shovel for soil
    /// or sand and the pickaxe for stone, rock and ore (G and the wheel, as
    /// a player picks a tool), cut the blocks at head and feet in the cell
    /// ahead, step in; once moving again, put the act's own tool back.
    /// Whether it is still at it.
    fn dig_through(&mut self, s: &Sight, out: &mut Intent, point: Vec3) -> bool {
        let th = *self.through.get_or_insert_with(|| {
            info!("showcase: no headway for 4 s; digging through toward the goal");
            Through {
                was: s.tools.held(),
                fetching: None,
                fetch_s: 0.0,
                since: s.now,
                block_s: 0.0,
            }
        });
        let mut th = th;
        // Fetching a tool: G held, the wheel turned until the picker shows it.
        if let Some(want) = th.fetching {
            th.fetch_s += s.dt;
            if s.tools.held() == want || th.fetch_s > 6.0 {
                th.fetching = None;
            } else {
                out.held.insert(KeyCode::KeyG);
                if s.picking && s.picker != want && (th.fetch_s % 0.25) < s.dt {
                    out.wheel = 1;
                }
                if s.picking && s.picker == want {
                    // Let G go: the picked tool comes to hand.
                    out.held.remove(&KeyCode::KeyG);
                }
            }
            self.through = Some(th);
            return true;
        }
        let records = s.fine.set.finest_records();
        let Some(c) = s.contact.finest_cell(s.up) else {
            self.through = Some(th);
            return true;
        };
        let altitude = s.feet.length() - PLANET_RADIUS;
        let Some(feet_layer) = pbd_core::column::layer_at(altitude + 0.2) else {
            self.through = None;
            return false;
        };
        let dir = tangent(point - s.feet, s.up).normalize_or(s.heading);
        let nb = &s.fine.set.finest_neighbors;
        let centre_of = |i: usize| Vec3::from_slice(&records[i].direction_height[..3]).normalize();
        let here = centre_of(c);
        let next = nb[c][..records[c].degree()]
            .iter()
            .map(|&n| n as usize)
            .max_by(|&a, &b| {
                let da = tangent(centre_of(a) - here, s.up)
                    .normalize_or_zero()
                    .dot(dir);
                let db = tangent(centre_of(b) - here, s.up)
                    .normalize_or_zero()
                    .dot(dir);
                da.total_cmp(&db)
            });
        let Some(next) = next else {
            self.through = Some(th);
            return true;
        };
        let key = records[next].key();
        let solid = |layer: usize| {
            digging::material_of(s.fine, s.save, (key, layer))
                .filter(|m| m != &Material::Air && !m.is_water() && !m.is_lamp())
        };
        // Up to a goal well over the feet (a bench up a bluff): steps cut up
        // the face, the block at the feet left as the step and the two over
        // it cleared, climbed by auto step; else straight on at feet and head.
        let rise = (point - s.feet).dot(s.up);
        let climbing = rise > 1.5;
        let layers = if climbing {
            feet_layer + 1..=feet_layer + 2
        } else {
            feet_layer..=feet_layer + 1
        };
        let block = layers.rev().find_map(|l| solid(l).map(|m| (l, m)));
        // A station in the way is the player's own: never dug through.
        if block.is_some_and(|(_, m)| pbd_core::station::station_of(m).is_some()) {
            self.through = None;
            self.route = None;
            return false;
        }
        // The planned cell clear but the walker held: the block the look
        // meets at the feet or the head is what holds it (a corner of another
        // cell); that is dug, with its own tool.
        if block.is_none() && self.stuck_s > 1.0 {
            let looked = self
                .aimed_cell(s)
                .filter(|(_, l)| (feet_layer..=feet_layer + 1).contains(l))
                .and_then(|c| digging::material_of(s.fine, s.save, c))
                .filter(|m| {
                    m != &Material::Air
                        && !m.is_water()
                        && !m.is_lamp()
                        && pbd_core::station::station_of(*m).is_none()
                });
            if let Some(material) = looked {
                let want = if matches!(material, Material::Stone | Material::Rock | Material::Ore) {
                    Some(Tool::Pickaxe)
                } else {
                    Some(Tool::Shovel)
                };
                if want.is_some_and(|t| s.tools.tools().contains(&t)) && s.tools.held() != want {
                    th.fetching = Some(want);
                    th.fetch_s = 0.0;
                } else {
                    out.lmb = true;
                    th.block_s += s.dt;
                    if th.block_s > 10.0 {
                        out.taps.push(KeyCode::Space);
                        th.block_s = 0.0;
                    }
                }
                if (s.now % 2.0) < s.dt {
                    info!("showcase: through: the look meets {material:?}; digging it");
                }
                self.through = Some(th);
                return true;
            }
        }
        match block {
            Some((layer, material)) => {
                let at = centre_of(next)
                    * (PLANET_RADIUS + pbd_core::column::layer_altitude(layer) + 0.5);
                let aimed = self.aim_at(s, out, at);
                // What the look actually meets at the feet or the head: dug
                // with its own tool (seen 2026-11-20: the planned block was
                // grass, the look met stone in front of it, and the bot stood
                // with a shovel pressing nothing for minutes).
                let hit = self
                    .aimed_cell(s)
                    .filter(|(_, l)| (feet_layer..=feet_layer + 2).contains(l));
                let hit_material = hit
                    .and_then(|c| digging::material_of(s.fine, s.save, c))
                    .filter(|m| {
                        m != &Material::Air
                            && !m.is_water()
                            && !m.is_lamp()
                            && pbd_core::station::station_of(*m).is_none()
                    });
                let material = hit_material.unwrap_or(material);
                let want = if matches!(material, Material::Stone | Material::Rock | Material::Ore) {
                    Some(Tool::Pickaxe)
                } else {
                    Some(Tool::Shovel)
                };
                let owned = want.is_some_and(|t| s.tools.tools().contains(&t));
                if owned && s.tools.held() != want {
                    th.fetching = Some(want);
                    th.fetch_s = 0.0;
                    self.through = Some(th);
                    return true;
                }
                // The look held on a block in front of the planned one for a
                // second: that block is in the way, and is dug first.
                let on = hit.is_some_and(|(k, l)| k == key && l == layer)
                    || (hit_material.is_some() && (aimed || th.block_s > 1.0));
                if (th.block_s % 2.0) < s.dt {
                    info!(
                        "showcase: through: planned {key}/{layer} {:?}, feet {feet_layer}, aimed {aimed} at {:?} {hit_material:?}, want {want:?}, held {:?}, on {on}, climbing {climbing}",
                        solid(layer),
                        self.aimed_cell(s),
                        s.tools.held()
                    );
                }
                out.lmb = on && (aimed || self.lmb_down || th.block_s > 1.0);
                th.block_s += s.dt;
                if th.block_s > 10.0 {
                    // It will not break: jump and try again later.
                    out.taps.push(KeyCode::Space);
                    th.block_s = 0.0;
                }
            }
            None => {
                // Clear ahead: step through.
                th.block_s = 0.0;
                out.held.insert(KeyCode::KeyW);
                self.steer(s, out, dir, -0.2);
                if self.stuck_s == 0.0 || s.now - th.since > 40.0 {
                    // Moving again (or it has tried long enough): the act's
                    // own tool back in hand, and on with the walk.
                    if s.tools.held() != th.was
                        && th.was.is_some_and(|t| s.tools.tools().contains(&t))
                    {
                        th.fetching = Some(th.was);
                        th.fetch_s = 0.0;
                        th.since = s.now - 41.0;
                        self.through = Some(th);
                        return true;
                    }
                    self.through = None;
                    self.route = None;
                    return false;
                }
            }
        }
        self.through = Some(th);
        true
    }

    /// Have `want` in hand, fetched by G and the wheel as a player picks a
    /// tool. Whether it is still fetching (the act waits for it).
    fn fetch_tool(&mut self, s: &Sight, out: &mut Intent, want: Option<Tool>) -> bool {
        if s.tools.held() == want || want.is_some_and(|t| !s.tools.tools().contains(&t)) {
            self.fetching = None;
            return false;
        }
        let (_, since) = self.fetching.get_or_insert((want, 0.0));
        *since += s.dt;
        if *since > 6.0 {
            self.fetching = None;
            return false;
        }
        out.held.insert(KeyCode::KeyG);
        if s.picking && s.picker != want && (*since % 0.25) < s.dt {
            out.wheel = 1;
        }
        if s.picking && s.picker == want {
            out.held.remove(&KeyCode::KeyG);
        }
        true
    }

    /// The tool that cuts a block fastest: the shovel for earth, the
    /// pickaxe for stone, rock and ore.
    fn tool_for(material: Material) -> Option<Tool> {
        if matches!(material, Material::Stone | Material::Rock | Material::Ore) {
            Some(Tool::Pickaxe)
        } else {
            Some(Tool::Shovel)
        }
    }

    /// Whether the ground all round stands well over the feet: a pit.
    fn in_pit(&self, s: &Sight) -> bool {
        let ground = s.eye.length() - PLANET_RADIUS - pbd_app::walking::EYE_HEIGHT;
        let east = Vec3::Y.cross(s.up).normalize_or(Vec3::X);
        let north = s.up.cross(east);
        let records = s.fine.set.finest_records();
        (0..6)
            .filter(|k| {
                let a = *k as f32 / 6.0 * std::f32::consts::TAU;
                let d = (s.feet + (east * a.cos() + north * a.sin()) * 2.6).normalize();
                s.contact
                    .finest_cell(d)
                    .and_then(|i| records.get(i))
                    .is_some_and(|r| r.direction_height[3] - ground > 1.3)
            })
            .count()
            >= 5
    }

    /// One step of climbing out of a pit: dirt in hand, look down, jump,
    /// and set the block under the feet at the jump's top.
    fn climb(&mut self, s: &Sight, out: &mut Intent) {
        if self.climb_s == 0.0 {
            info!("showcase: climbing out of a pit");
        }
        self.climb_s += s.dt;
        // Building up has not lifted it (a roof over the hole, or no room
        // to jump): dig a step out instead, ahead and a little up, and walk
        // into it, as a player cuts stairs.
        let cycle = self.climb_s % 14.0;
        if cycle > 7.0 {
            if cycle - s.dt <= 7.0 {
                info!("showcase: cutting steps out of a pit");
            }
            self.steer(s, out, s.heading, 0.25);
            out.lmb = true;
            out.held.insert(KeyCode::KeyW);
            if (cycle % 1.0) < s.dt * 1.5 {
                out.taps.push(KeyCode::Space);
            }
            return;
        }
        let dirt = (0..SLOTS).find(|&k| {
            s.slots.get(k).is_some_and(|st| {
                matches!(
                    st.item,
                    Item::Block(Material::Dirt | Material::Grass | Material::Soil)
                )
            })
        });
        if let Some(k) = dirt
            && s.slots.selected() != k
        {
            out.taps.push(DIGITS[k]);
            return;
        }
        self.steer(s, out, s.heading, -1.45);
        let looking = s.pitch < -1.2;
        let t = self.climb_s % 1.0;
        if looking && t < s.dt * 1.5 {
            out.taps.push(KeyCode::Space);
        } else if looking && (0.22..0.6).contains(&t) && !self.rmb_down {
            // Through the jump's top: whichever frame the feet clear the
            // layer, the block goes under them.
            out.rmb_tap = true;
        }
    }

    /// With fewer than two free slots carried, a junk stack (sand, soil,
    /// dirt, grass) into the pack's trash, as a player clears a full pack
    /// (`drop-and-trash` design 4). Whether it is still at it.
    fn make_room(&mut self, s: &Sight, out: &mut Intent) -> bool {
        let free = (0..CARRIED).filter(|&k| s.slots.get(k).is_none()).count();
        // Only a full stack of junk: a handful is the slot being filled by
        // the dig (owner, 2026-11-20: "you cant be trashing stuff at 2, you
        // should way at least till 64 or 99 whatever full stacks are").
        let junk = (0..CARRIED).find(|&k| {
            s.slots.get(k).is_some_and(|st| {
                st.count >= st.item.stack_limit()
                    && matches!(
                        st.item,
                        Item::Block(
                            Material::Sand
                                | Material::Soil
                                | Material::Dirt
                                | Material::Grass
                                | Material::DryGrass
                                | Material::JungleGrass
                                | Material::DesertSand
                        )
                    )
            })
        });
        let (phase, since) = &mut self.room;
        if *phase == 0 && (free >= 2 || junk.is_none()) {
            return false;
        }
        *since += s.dt;
        match *phase {
            0 => {
                info!("showcase: the pack is full; a stack of junk into the trash");
                if s.screen != Screen::Pack {
                    out.taps.push(KeyCode::KeyI);
                }
                *phase = 1;
                *since = 0.0;
            }
            1 if *since > 0.6 => {
                if let Some(k) = junk {
                    out.click = Some(Click::Cell(k));
                    out.lmb_tap = true;
                }
                *phase = 2;
                *since = 0.0;
            }
            2 if *since > 0.5 => {
                out.click = Some(Click::Trash);
                out.lmb_tap = true;
                *phase = 3;
                *since = 0.0;
            }
            3 if *since > 0.5 => {
                if s.screen == Screen::Pack {
                    out.taps.push(KeyCode::KeyI);
                }
                *phase = 0;
                *since = 0.0;
            }
            _ => {}
        }
        true
    }

    /// Put an item in the selected slot: its number key, or the pack first
    /// when it is not in the hotbar. `Done` when it is in hand.
    fn select(&mut self, s: &Sight, out: &mut Intent, pick: &dyn Fn(Item) -> bool) -> Status {
        let (phase, since) = &mut self.sel;
        *since += s.dt;
        // A step that never finishes starts the move over (seen: the pack
        // held open with a stack in it and the hotbar not taking it).
        if *phase != 0 && *since > 5.0 {
            warn!(
                "showcase: select stuck at step {} (from {}, screen {:?}); starting over",
                *phase, self.sel_from, s.screen
            );
            *phase = 0;
            *since = 0.0;
        }
        let in_hotbar = (0..SLOTS).find(|&k| s.slots.get(k).is_some_and(|st| pick(st.item)));
        if *since == s.dt {
            info!(
                "showcase: select step {} (in hotbar {in_hotbar:?}, from {}, screen {:?})",
                *phase, self.sel_from, s.screen
            );
        }
        match *phase {
            0 => {
                if let Some(k) = in_hotbar {
                    if s.slots.selected() == k && s.screen == Screen::Playing {
                        *phase = 0;
                        *since = 0.0;
                        return Status::Done;
                    }
                    if s.screen == Screen::Pack {
                        out.taps.push(KeyCode::KeyI);
                    } else if *since > 0.2 {
                        out.taps.push(DIGITS[k]);
                        *since = 0.0;
                    }
                    return Status::Running;
                }
                let Some(from) =
                    (SLOTS..CARRIED).find(|&k| s.slots.get(k).is_some_and(|st| pick(st.item)))
                else {
                    return Status::Fail("not carried".into());
                };
                self.sel_from = from;
                if s.screen != Screen::Pack {
                    out.taps.push(KeyCode::KeyI);
                }
                // A full hotbar: first a stack out of it into the pack, as a
                // player makes room (a drop onto a different stack does not
                // swap).
                let full = (0..SLOTS).all(|k| s.slots.get(k).is_some());
                *phase = if full { 10 } else { 1 };
                *since = 0.0;
                Status::Running
            }
            10 if *since > 0.6 => {
                let junk = [
                    Item::Block(Material::Sand),
                    Item::Block(Material::Dirt),
                    Item::Block(Material::Grass),
                    Item::Block(Material::Soil),
                ];
                let out_of = junk
                    .iter()
                    .find_map(|j| {
                        (0..SLOTS).find(|&k| s.slots.get(k).is_some_and(|st| st.item == *j))
                    })
                    .unwrap_or(SLOTS - 1);
                out.click = Some(Click::Cell(out_of));
                out.lmb_tap = true;
                *phase = 11;
                *since = 0.0;
                Status::Running
            }
            11 if *since > 0.5 => {
                let Some(into) = (SLOTS..CARRIED).find(|&k| s.slots.get(k).is_none()) else {
                    return Status::Fail("the pack is full".into());
                };
                out.click = Some(Click::Cell(into));
                out.lmb_tap = true;
                *phase = 1;
                *since = 0.0;
                Status::Running
            }
            1 if *since > 0.6 => {
                out.click = Some(Click::Cell(self.sel_from));
                out.lmb_tap = true;
                *phase = 2;
                *since = 0.0;
                Status::Running
            }
            2 if *since > 0.5 => {
                // Onto a hotbar slot holding the kit's least used block.
                let junk = [
                    Item::Block(Material::Sand),
                    Item::Block(Material::Candle),
                    Item::Block(Material::Brazier),
                    Item::Block(Material::LanternHanging),
                    Item::Block(Material::LanternWall),
                    Item::Block(Material::Dirt),
                ];
                let to = (0..SLOTS)
                    .find(|&k| s.slots.get(k).is_none())
                    .or_else(|| {
                        junk.iter().find_map(|j| {
                            (0..SLOTS).find(|&k| s.slots.get(k).is_some_and(|st| st.item == *j))
                        })
                    })
                    .unwrap_or(SLOTS - 1);
                out.click = Some(Click::Cell(to));
                out.lmb_tap = true;
                *phase = 3;
                *since = 0.0;
                Status::Running
            }
            3 if *since > 0.6 => {
                if s.screen == Screen::Pack {
                    out.taps.push(KeyCode::KeyI);
                }
                *phase = 0;
                *since = 0.0;
                Status::Running
            }
            _ => Status::Running,
        }
    }

    /// Grassy cells near the feet, between two distances, not used yet.
    fn fresh_cells(&mut self, s: &Sight, near: f32, far: f32) -> Vec<(u32, Vec3, f32)> {
        let toward = s.feet.normalize_or(Vec3::Y);
        let cos = (far / PLANET_RADIUS).cos();
        let mut out = Vec::new();
        // Not beside a station: a hole there is a hole the bot must stand
        // in to use it.
        let stations: Vec<u32> = s
            .save
            .edits
            .all()
            .into_iter()
            .filter(|e| pbd_core::station::station_of(e.material).is_some())
            .map(|e| e.cell)
            .collect();
        let stations: Vec<Vec3> = stations
            .into_iter()
            .filter_map(|k| self.top_of(s, k))
            .collect();
        for r in s.fine.set.finest_records() {
            let d = Vec3::from_slice(&r.direction_height[..3]);
            let Some(d) = d.try_normalize() else { continue };
            if d.dot(toward) < cos {
                continue;
            }
            let key = r.key();
            let material = r.metadata[1] & 0xff;
            // Earth a spade takes: soil, grass of any kind, sand and dirt
            // (bait worms come out of a beach's sand), down to just over
            // the water's edge.
            if !matches!(material, 2 | 3 | 6 | 7 | 8 | 11)
                || r.direction_height[3] < 1.2
                || pbd_core::flora::grows_tree(
                    &pbd_app::planet::terrain_config().trees,
                    r.spare[0],
                    r.metadata[1],
                )
                || self.used.contains(&key)
                || !s.save.edits.for_cell(key).is_empty()
            {
                continue;
            }
            // Never a cell by the water: a hole there floods, and the bot
            // ends under the sea (seen 2026-11-20). Every earth gives bait,
            // so nothing is lost (owner, 2026-11-20: "all dirt blocks
            // should have a chance to get bait").
            if pbd_core::column::layer_at(r.direction_height[3] - 0.5)
                .is_some_and(|layer| digging::by_water(s.fine, s.save, &s.gather.0, (key, layer)))
            {
                continue;
            }
            // Digging for bait: earth by a river or the sea, as the game's
            // own bait rule has it (`dig-fixes` 2).
            if self.want_wet
                && !pbd_core::column::layer_at(r.direction_height[3] - 0.5).is_some_and(|layer| {
                    digging::by_water(s.fine, s.save, &s.gather.0, (key, layer))
                })
            {
                continue;
            }
            let top = d * (PLANET_RADIUS + r.direction_height[3]);
            let dist = tangent(top - s.feet, s.up).length();
            let rise = (top - s.feet).dot(s.up);
            if dist < near
                || dist > far
                // A rise matters only within reach: further off, the route
                // walks up to it.
                || (dist < 6.0 && rise.abs() > 1.6)
                || stations.iter().any(|st| st.distance(top) < 4.5)
            {
                continue;
            }
            // Not at the water's edge, where a roam ends in the sea.
            let east = Vec3::Y.cross(d).normalize_or(Vec3::X);
            let north = d.cross(east);
            let wet = [east, -east, north, -north]
                .iter()
                .any(|o| surface_height((top + *o * 3.5).normalize()) < 0.3);
            if wet {
                continue;
            }
            out.push((key, top, dist));
        }
        out
    }

    /// The fresh cell ahead most nearly along the heading.
    fn cell_ahead(&mut self, s: &Sight, near: f32, far: f32) -> Option<(u32, Vec3)> {
        let cells = self.fresh_cells(s, near, far);
        cells
            .into_iter()
            .map(|(k, top, _)| {
                let a = signed_angle(s.heading, tangent(top - s.feet, s.up), s.up).abs();
                (k, top, a)
            })
            .min_by(|a, b| a.2.total_cmp(&b.2))
            .map(|(k, top, _)| (k, top))
    }

    fn aimed_cell(&self, s: &Sight) -> Option<(u32, usize)> {
        s.aim.map(|t| (t.dig.cell, t.dig.layer))
    }

    fn stations(&mut self, s: &Sight, material: Material) -> Option<(u32, usize, Vec3)> {
        let mut best: Option<(u32, usize, Vec3, f32)> = None;
        for edit in s.save.edits.all() {
            if edit.material != material {
                continue;
            }
            let Some(top) = self.top_of(s, edit.cell) else {
                continue;
            };
            let at = top.normalize()
                * (PLANET_RADIUS + pbd_core::column::layer_altitude(usize::from(edit.layer)) + 0.5);
            let d = at.distance(s.feet);
            if best.is_none_or(|b| d < b.3) {
                best = Some((edit.cell, usize::from(edit.layer), at, d));
            }
        }
        best.map(|(c, l, at, _)| (c, l, at))
    }

    /// Walk on to fresh ground: the nearest grassy cell a few metres off
    /// whose straight way passes no hole the bot dug, so it never steps
    /// into its own pits.
    fn roam(&mut self, s: &Sight, out: &mut Intent) {
        if self.roam_to.is_none() {
            let holes: Vec<Vec3> = self
                .used
                .clone()
                .into_iter()
                .filter_map(|k| self.top_of(s, k))
                .collect();
            let clear = |to: Vec3| {
                let way = tangent(to - s.feet, s.up);
                let len = way.length().max(0.01);
                let dir = way / len;
                holes.iter().all(|h| {
                    let rel = tangent(*h - s.feet, s.up);
                    let along = rel.dot(dir).clamp(0.0, len);
                    (rel - dir * along).length() > 2.0
                })
            };
            let home = self
                .stations(s, Material::Workbench)
                .map(|(_, _, at)| at)
                .or(self.home);
            let pick = self
                // After bait, along the coast: earth by the water may be a
                // long way round a rocky shore.
                .fresh_cells(s, 5.0, if self.want_wet { 120.0 } else { 30.0 })
                .into_iter()
                .filter(|(_, top, _)| clear(*top))
                // Near home as well as near the feet, so the bot digs round
                // its workbench rather than wandering off across the map.
                .min_by(|a, b| {
                    let cost = |c: &(u32, Vec3, f32)| c.2 + home.map_or(0.0, |h| h.distance(c.1));
                    cost(a).total_cmp(&cost(b))
                })
                .map(|c| c.1);
            info!(
                "showcase: roaming to fresh ground ({})",
                if pick.is_some() {
                    "a clear cell"
                } else {
                    "anywhere"
                }
            );
            self.roam_to = Some(pick.unwrap_or_else(|| dry_ground(s)));
        }
        let to = self.roam_to.expect("set above");
        self.roam_s += s.dt;
        // Somewhere it cannot get to in time: somewhere else.
        if self.walk_to(s, out, to, 1.2) != Walked::Going || self.roam_s > 12.0 {
            self.roam_to = None;
            self.roam_s = 0.0;
        }
    }

    fn next_phase(&mut self) {
        self.phase += 1;
        self.phase_s = 0.0;
    }

    fn until_met(&self, s: &Sight, until: &Until) -> bool {
        match until {
            Until::Have(name, n) => carried(s.slots, name) >= *n,
            Until::Mission(id) => s.missions.progress.is_done(id),
            Until::Times(n) => self.count >= *n,
        }
    }
}

#[derive(PartialEq, Eq)]
enum Walked {
    There,
    Going,
    Stuck,
}

/// Count this frame's deeds from the game's own messages.
#[allow(clippy::too_many_arguments)]
pub fn tally(
    mut tally: ResMut<Tally>,
    mut did: MessageReader<pbd_app::missions::Did>,
    mut gained: MessageReader<pbd_app::hotbar::Gained>,
    mut planted: MessageReader<pbd_app::hotbar::Planted>,
    mut deeds: MessageReader<pbd_app::husbandry::Deed>,
    mut cooked: MessageReader<pbd_app::kitchen::Cooked>,
    mut ate: MessageReader<pbd_app::kitchen::Ate>,
) {
    use pbd_core::missions::Kind;
    for d in did.read() {
        match d.0.kind {
            Kind::Gather => tally.dug += 1,
            Kind::Place => tally.placed += 1,
            Kind::Cast => tally.cast += 1,
            Kind::Catch => tally.caught += 1,
            Kind::Till => tally.tilled += 1,
            _ => {}
        }
    }
    for g in gained.read() {
        match g.how {
            pbd_app::hotbar::How::Picked => tally.picked += 1,
            pbd_app::hotbar::How::Harvested => tally.harvested += 1,
            pbd_app::hotbar::How::Collected => tally.collected += 1,
            pbd_app::hotbar::How::Gathered => tally.gathered += 1,
            _ => {}
        }
    }
    tally.planted += planted.read().count() as u32;
    for deed in deeds.read() {
        match deed {
            pbd_app::husbandry::Deed::Tamed { .. } => tally.tamed += 1,
            pbd_app::husbandry::Deed::Penned { .. } => tally.penned += 1,
            pbd_app::husbandry::Deed::PenBuilt { .. } => {}
        }
    }
    tally.cooked += cooked.read().map(|c| u32::from(c.count)).sum::<u32>();
    tally.ate += ate.read().count() as u32;
}

/// The input side of the bot's frame.
#[derive(bevy::ecs::system::SystemParam)]
pub struct Hands<'w, 's> {
    keys: ResMut<'w, ButtonInput<KeyCode>>,
    mouse: ResMut<'w, ButtonInput<MouseButton>>,
    motion: ResMut<'w, AccumulatedMouseMotion>,
    wheel: MessageWriter<'w, MouseWheel>,
    /// The window, for the pointer's place over an open screen.
    pointer: Query<'w, 's, &'static mut Window, With<PrimaryWindow>>,
}

/// The world side of the bot's frame.
#[derive(bevy::ecs::system::SystemParam)]
pub struct World<'w, 's> {
    time: Res<'w, Time>,
    /// Absent in a test's app, which has a camera and no walker plugin.
    state: Option<ResMut<'w, pbd_app::walking::WalkingState>>,
    walkers: Query<'w, 's, &'static Position, With<pbd_app::walking::Walker>>,
    /// The walker's `Position` is local; the bot steers by planet points.
    space: pbd_app::SimSpace<'w>,
    cameras: Query<'w, 's, &'static GlobalTransform, With<pbd_app::walking::WalkingCamera>>,
    /// Render space to planet-local (`movement-smoothness` design 6.1).
    frame: Option<Res<'w, pbd_app::planet::PlanetRenderFrame>>,
    windows: Query<'w, 's, Entity, With<PrimaryWindow>>,
    fine: Option<Res<'w, PlanetFine>>,
    contact: Option<Res<'w, PlanetContact>>,
    save: Res<'w, WorldSave>,
    hotbar: Res<'w, super::slots::Hotbar>,
    tools: Res<'w, pbd_app::fish::ToolSlot>,
    aim: Res<'w, Aim>,
    screen: Res<'w, Screen>,
    fishery: Option<Res<'w, pbd_app::fish::Fishery>>,
    yard: Option<Res<'w, pbd_app::husbandry::Farmyard>>,
    missions: Res<'w, pbd_app::missions::Missions>,
    chain: Option<Res<'w, pbd_app::missions::MissionChain>>,
    recipes: Res<'w, pbd_app::config::Recipes>,
    land: Res<'w, pbd_app::config::HusbandryConfig>,
    farm: Res<'w, pbd_app::config::FarmConfig>,
    gather: Res<'w, pbd_app::config::GatherConfig>,
    picker: Res<'w, super::equipment::Picker>,
    key: Option<Res<'w, pbd_app::controls::PickerKey>>,
    tally: Res<'w, Tally>,
    sun: Res<'w, pbd_app::sky::Sun>,
    map_view: Option<Res<'w, super::map_screen::MapView>>,
    scatter: Option<Res<'w, pbd_app::config::ScatterSettings>>,
    /// What F does now, for the status line (a stall reads off it).
    interact: Option<Res<'w, pbd_app::interact::Interact>>,
    /// The cast's speeds and charge, to see where a float would land.
    fauna: Res<'w, pbd_app::config::FaunaConfig>,
    /// The mission tree's cards, where they are drawn (the `Focus` act
    /// clicks one).
    cards: Query<
        'w,
        's,
        (
            &'static super::mission_tree::Card,
            &'static bevy::ui::UiGlobalTransform,
        ),
    >,
}

/// The bot's frame: look, decide, press.
#[allow(clippy::too_many_arguments)]
pub fn drive(
    mut commands: Commands,
    mut player: ResMut<Player>,
    mut pending: ResMut<PendingClick>,
    mut hands: Hands,
    mut w: World,
    mut captions: Query<&mut pbd_app::ui_kit::KitText, With<CaptionText>>,
    mut exit: MessageWriter<AppExit>,
    control: Option<Res<control::Control>>,
) {
    let live = control.is_some();
    let player = &mut *player;
    let bot = &mut player.bot;
    // Last frame's taps come up.
    for key in std::mem::take(&mut bot.released) {
        hands.keys.release(key);
    }
    if bot.tap_lmb_down {
        hands.mouse.release(MouseButton::Left);
        bot.tap_lmb_down = false;
    }
    if bot.rmb_down {
        hands.mouse.release(MouseButton::Right);
        bot.rmb_down = false;
    }
    if bot.failed {
        // A few frames for the stall's still, then out with an error.
        bot.fail_frames += 1;
        if bot.fail_frames == 20 {
            exit.write(AppExit::error());
        }
        return;
    }
    if w.state.as_ref().is_some_and(|s| !s.active) {
        return;
    }
    let now = w.time.elapsed_secs();
    let dt = w.time.delta_secs().min(0.1);
    if let Some(ended) = bot.ended {
        if now - ended > 3.0 {
            exit.write(AppExit::Success);
        }
        return;
    }
    let (Some(fine), Some(contact), Some(chain)) =
        (w.fine.as_deref(), w.contact.as_deref(), w.chain.as_deref())
    else {
        return;
    };
    let (Ok(position), Ok(camera)) = (w.walkers.single(), w.cameras.single()) else {
        return;
    };
    // The walker reads the bot's keys with the pointer captured; while a
    // panel is open it lets go, as it does for a player.
    if let Some(state) = w.state.as_mut() {
        state.scripted = *w.screen == Screen::Playing;
    }
    if bot.home.is_none()
        && let Ok(p) = w.walkers.single()
    {
        bot.home = Some(w.space.origin().query_point(p.0));
    }
    let started = *bot.started.get_or_insert_with(|| {
        info!("SHOWCASE_START {}", player.name);
        info!("SHOWCASE_GOAL {}", player.file.goal);
        now
    });
    let set_caption = |captions: &mut Query<&mut pbd_app::ui_kit::KitText, With<CaptionText>>,
                       text: &str| {
        for mut t in captions.iter_mut() {
            if t.text != text {
                t.text = Localized::verbatim(text.to_string());
            }
        }
    };
    // The goal, said first, for a few seconds.
    if now - started < 4.0 {
        set_caption(&mut captions, &player.file.goal);
        return;
    }
    // Where the player looks, off the camera the walker's mouse-look turns.
    let feet = w.space.origin().query_point(position.0);
    let up = feet.normalize_or(Vec3::Y);
    let look = camera.forward().as_vec3();
    let heading = tangent(look, up).normalize_or(up.any_orthonormal_vector());
    let pitch = look.dot(up).clamp(-1.0, 1.0).asin();
    let sight = Sight {
        now,
        dt,
        feet,
        up: feet.normalize_or(Vec3::Y),
        eye: pbd_app::render_origin::local_of(w.frame.as_deref(), camera.translation()),
        look: camera.forward().as_vec3(),
        heading,
        pitch,
        fine,
        contact,
        save: &w.save,
        slots: &w.hotbar.0,
        tools: &w.tools.0,
        aim: w.aim.target,
        screen: *w.screen,
        fishery: w.fishery.as_deref(),
        yard: w.yard.as_deref(),
        origin: w.space.origin(),
        missions: &w.missions,
        chain: &chain.0,
        recipes: &w.recipes,
        land: w.land.0.roster(pbd_core::fauna::HOME_BODY),
        farm: &w.farm,
        gather: &w.gather,
        picker: w.picker.highlight,
        picking: w.key.as_deref().is_some_and(|k| k.holding),
        tally: *w.tally,
        hour: w.sun.clock.hour(),
        map_view: w.map_view.as_deref().copied(),
        window: hands
            .pointer
            .single()
            .ok()
            .map(|w| Vec2::new(w.width(), w.height())),
        odds: w.scatter.as_deref().map(pbd_app::clutter::odds),
        day: pbd_app::clutter::day_of(w.sun.clock.seconds),
        fishing: &w.fauna.0.fishing,
        cards: {
            let scale = hands.pointer.single().map_or(1.0, |w| w.scale_factor());
            w.cards
                .iter()
                .map(|(card, t)| (card.0, t.translation / scale))
                .collect()
        },
    };
    // Hot reload (`bot-control`): the shot file edited while the run plays
    // is read again once a second; the current shot starts over on the new
    // acts, the bot keeping its memory. A file that does not parse is
    // logged and the old one kept.
    if live && now - bot.file_checked > 1.0 {
        bot.file_checked = now;
        let path = file::dir().join(format!("{}.ron", player.name));
        let changed = std::fs::metadata(&path).and_then(|m| m.modified()).ok();
        if bot.file_seen.is_none() {
            bot.file_seen = changed;
        } else if changed.is_some() && changed != bot.file_seen {
            bot.file_seen = changed;
            match file::load(&player.name) {
                Ok(file) => {
                    info!(
                        "BOT_CONTROL hot reload: {} shots; shot {} starts over",
                        file.shots.len(),
                        bot.shot + 1
                    );
                    player.file = file;
                    bot.act = 0;
                    bot.injected.clear();
                    bot.paused = false;
                    bot.reset_act(&sight);
                }
                Err(error) => warn!("BOT_CONTROL hot reload refused: {error}"),
            }
        }
    }
    // Live control (`bot-control`): this frame's commands, answered now.
    if let Some(control) = control.as_deref() {
        for command in control.drain() {
            let cmd = command.body["cmd"].as_str().unwrap_or("").to_string();
            let answer = match cmd.as_str() {
                "status" => {
                    let shot = player.file.shots.get(bot.shot);
                    let pbd_app::place::Place { lat, lon, .. } =
                        pbd_app::place::Place::of(sight.up);
                    let mission = sight.missions.progress.focused(sight.chain).map(|m| {
                        serde_json::json!({
                            "id": m.id,
                            "title": m.title,
                            "counts": sight.missions.progress.counts(m),
                            "of": m.objectives.iter().map(|o| o.count).collect::<Vec<_>>(),
                        })
                    });
                    serde_json::json!({
                        "ok": true,
                        "shot": bot.shot + 1,
                        "shots": player.file.shots.len(),
                        "caption": shot.map(|s| s.caption.clone()),
                        "act": bot.act + 1,
                        "acts": shot.map(|s| s.acts.iter().map(|a| format!("{a:?}")).collect::<Vec<_>>()),
                        "injected": bot.injected.iter().map(|a| format!("{a:?}")).collect::<Vec<_>>(),
                        "phase": bot.phase,
                        "phase_s": bot.phase_s,
                        "act_s": bot.act_s,
                        "shot_s": bot.shot_s,
                        "paused": bot.paused,
                        "lat": lat,
                        "lon": lon,
                        "height_m": sight.feet.length() - PLANET_RADIUS,
                        "aim": sight.aim.map(|t| (t.dig.cell, t.dig.layer)),
                        "aim_material": sight.aim
                            .and_then(|t| digging::material_of(sight.fine, sight.save, (t.dig.cell, t.dig.layer)))
                            .map(|m| format!("{m:?}")),
                        "f": w.interact.as_deref().and_then(|i| i.now.as_ref()).map(|o| String::from(o.words())),
                        "in_hand": format!("{:?}", sight.tools.held()),
                        "selected": sight.slots.held().map(|st| format!("{:?} x{}", st.item, st.count)),
                        "screen": format!("{:?}", sight.screen),
                        "mission": mission,
                        "hour": sight.hour,
                    })
                }
                "do" => match command.body["act"].as_str().map(ron::from_str::<file::Act>) {
                    Some(Ok(act)) => {
                        info!("BOT_CONTROL do {act:?}");
                        bot.injected.push_back(act);
                        bot.reset_act(&sight);
                        serde_json::json!({"ok": true, "queued": bot.injected.len()})
                    }
                    Some(Err(error)) => {
                        serde_json::json!({"ok": false, "error": error.to_string()})
                    }
                    None => serde_json::json!({"ok": false, "error": "do takes \"act\""}),
                },
                "skip" => {
                    if bot.injected.pop_front().is_none() {
                        bot.act += 1;
                    }
                    bot.reset_act(&sight);
                    info!("BOT_CONTROL skip");
                    serde_json::json!({"ok": true, "act": bot.act + 1})
                }
                "skip_shot" => {
                    bot.injected.clear();
                    bot.shot += 1;
                    bot.act = 0;
                    bot.shot_s = 0.0;
                    bot.reset_act(&sight);
                    info!("BOT_CONTROL skip_shot");
                    serde_json::json!({"ok": true, "shot": bot.shot + 1})
                }
                "pause" => {
                    bot.paused = true;
                    serde_json::json!({"ok": true})
                }
                "resume" => {
                    bot.paused = false;
                    bot.reset_act(&sight);
                    serde_json::json!({"ok": true})
                }
                "still" => {
                    let path = command.body["path"]
                        .as_str()
                        .map(PathBuf::from)
                        .unwrap_or_else(|| {
                            std::env::temp_dir()
                                .join(format!("pbd-bot-still-{}.png", (now * 1000.0) as u64))
                        });
                    commands
                        .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
                        .observe(bevy::render::view::screenshot::save_to_disk(path.clone()));
                    serde_json::json!({"ok": true, "path": path.display().to_string(), "note": "written next frame"})
                }
                "reload" => match file::load(&player.name) {
                    Ok(file) => {
                        player.file = file;
                        info!("BOT_CONTROL reload: {} shots", player.file.shots.len());
                        serde_json::json!({"ok": true, "shots": player.file.shots.len()})
                    }
                    Err(error) => serde_json::json!({"ok": false, "error": error}),
                },
                other => serde_json::json!({"ok": false, "error": format!("no command {other:?}")}),
            };
            let _ = command.reply.send(answer.to_string());
        }
    }
    let bot = &mut player.bot;
    // Paused: nothing pressed, every held key up, the clocks stopped.
    if bot.paused {
        apply(bot, &mut hands, &w, &mut pending, Intent::default());
        return;
    }
    let Some(shot) = player.file.shots.get(bot.shot).cloned() else {
        bot.ended = Some(now);
        let done: Vec<&String> = sight.missions.progress.done.iter().collect();
        info!("SHOWCASE_DONE missions in the save: {done:?}");
        info!("SHOWCASE_END {}", player.name);
        return;
    };
    if bot.shot_s == 0.0 && bot.act == 0 && bot.act_s == 0.0 {
        info!("SHOT {} {}", bot.shot + 1, shot.caption);
        // One goal on screen at a time: the game's own mission tracker says
        // what is being done; a caption only says when time is sped up
        // (`mission-playthrough` 3.5).
        let caption = if shot.rate.is_some_and(|r| r > 1.0) {
            shot.caption.as_str()
        } else {
            ""
        };
        set_caption(&mut captions, caption);
        match shot.rate {
            Some(rate) => commands.insert_resource(pbd_app::sky::ClockRate(rate)),
            None => commands.remove_resource::<pbd_app::sky::ClockRate>(),
        }
        bot.mark = sight.tally;
    }
    // Where the bot is in its acts, every ten seconds, so a stall can be
    // read off the log.
    if (bot.shot_s / 10.0).floor() != ((bot.shot_s + dt) / 10.0).floor() {
        info!(
            "SHOWCASE_STATUS shot {} act {} phase {} ({:.1} s) goal {:?} screen {:?} aim {:?} in hand {:?} F {:?}",
            bot.shot + 1,
            bot.act + 1,
            bot.phase,
            bot.phase_s,
            bot.goal,
            sight.screen,
            sight.aim.map(|t| (t.dig.cell, t.dig.layer)),
            sight.tools.held(),
            w.interact
                .as_deref()
                .and_then(|i| i.now.as_ref())
                .map(|o| o.words()),
        );
    }
    // A still a few seconds into the first shot played, so a run's setup
    // (where it stands, what is on screen) is checked before it plays on.
    if bot.still == 0 && bot.shot_s < 3.0 && bot.shot_s + dt >= 3.0 {
        bot.want_still = true;
    }
    bot.shot_s += dt;
    bot.act_s += dt;
    bot.phase_s += dt;
    // The shot's missions done: the shot is over, whatever acts are left
    // (a world resumed part way through a mission, or a mission met sooner
    // than its acts expected). Not with a panel open: it is shut first.
    if !shot.until.is_empty()
        && *w.screen == Screen::Playing
        && bot.act < shot.acts.len()
        && shot
            .until
            .iter()
            .all(|id| sight.missions.progress.is_done(id))
    {
        bot.act = shot.acts.len();
    }
    let mut out = Intent::default();
    if let Some(act) = bot.injected.front().cloned() {
        let status = run(
            bot,
            &act,
            &sight,
            &mut out,
            &mut commands,
            player.stills.as_ref(),
        );
        match status {
            Status::Running => {}
            Status::Done => {
                info!("BOT_CONTROL done {act:?}");
                bot.injected.pop_front();
                bot.reset_act(&sight);
            }
            Status::Fail(why) => {
                warn!("BOT_CONTROL failed {act:?}: {why}");
                bot.injected.pop_front();
                bot.reset_act(&sight);
            }
        }
        apply(bot, &mut hands, &w, &mut pending, out);
        return;
    }
    let status = match shot.acts.get(bot.act) {
        Some(act) => {
            let act = act.clone();
            run(
                bot,
                &act,
                &sight,
                &mut out,
                &mut commands,
                player.stills.as_ref(),
            )
        }
        None => {
            if shot
                .until
                .iter()
                .all(|id| sight.missions.progress.is_done(id))
            {
                Status::Done
            } else if shot.repeat {
                bot.act = 0;
                bot.reset_act(&sight);
                Status::Running
            } else {
                Status::Fail(format!(
                    "its acts ended with {:?} not done",
                    shot.until
                        .iter()
                        .filter(|id| !sight.missions.progress.is_done(id))
                        .collect::<Vec<_>>()
                ))
            }
        }
    };
    // A still an act asked for in the middle of its work (a panel in
    // use, a fish on the line).
    if bot.want_still {
        still(bot, &mut commands, player.stills.as_ref());
    }
    let status = if bot.shot_s > shot.max_s && !live {
        Status::Fail(format!("over its {} s", shot.max_s))
    } else {
        status
    };
    match status {
        Status::Running => {}
        Status::Done if bot.act >= shot.acts.len() => {
            info!(
                "SHOT_DONE {} in {:.1} s{}",
                bot.shot + 1,
                bot.shot_s,
                if shot.until.is_empty() {
                    String::new()
                } else {
                    format!(", missions {:?} done", shot.until)
                }
            );
            bot.shot += 1;
            bot.act = 0;
            bot.shot_s = 0.0;
            bot.reset_act(&sight);
        }
        Status::Done => {
            bot.act += 1;
            bot.reset_act(&sight);
        }
        Status::Fail(why) => {
            let current = sight
                .missions
                .progress
                .current(sight.chain)
                .map(|m| {
                    format!(
                        "{} at {:?} of {:?}",
                        m.id,
                        sight.missions.progress.counts(m),
                        m.objectives.iter().map(|o| o.count).collect::<Vec<_>>()
                    )
                })
                .unwrap_or_default();
            error!(
                "SHOWCASE_STALL shot {} ({}) act {} {:?}: {why}; mission {current}",
                bot.shot + 1,
                shot.caption,
                bot.act + 1,
                shot.acts.get(bot.act)
            );
            if live {
                // Driven live: hold here, keys up, for a look and a nudge.
                bot.paused = true;
                bot.reset_act(&sight);
                apply(bot, &mut hands, &w, &mut pending, Intent::default());
                return;
            }
            bot.failed = true;
            // What it was looking at, for whoever reads the stall.
            if let Some(dir) = player.stills.as_ref() {
                commands
                    .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
                    .observe(bevy::render::view::screenshot::save_to_disk(
                        dir.join("stall.png"),
                    ));
            }
            return;
        }
    }
    apply(bot, &mut hands, &w, &mut pending, out);
}

impl Bot {
    fn reset_act(&mut self, s: &Sight) {
        self.act_s = 0.0;
        self.phase = 0;
        self.phase_s = 0.0;
        self.count = 0;
        self.tries = 0;
        self.goal = None;
        self.sel = (0, 0.0);
        self.mark = s.tally;
        self.walk_anchor = None;
        self.plan = None;
        self.look_goal = None;
        self.mine_dir = None;
        self.mine_from = None;
        self.away_from = None;
        self.route = None;
        self.route_block.clear();
        self.want_wet = false;
        self.stuck_s = 0.0;
        self.through = None;
        self.ore_goal = None;
    }
}

/// Press what the frame asks for.
fn apply(bot: &mut Bot, hands: &mut Hands, w: &World, pending: &mut PendingClick, out: Intent) {
    for key in [
        KeyCode::KeyW,
        KeyCode::KeyS,
        KeyCode::ShiftLeft,
        KeyCode::KeyG,
    ] {
        if out.held.contains(&key) {
            hands.keys.press(key);
        } else if hands.keys.pressed(key) {
            hands.keys.release(key);
        }
    }
    for key in out.taps {
        hands.keys.press(key);
        bot.released.push(key);
    }
    if out.lmb {
        hands.mouse.press(MouseButton::Left);
        bot.lmb_down = true;
    } else if bot.lmb_down {
        hands.mouse.release(MouseButton::Left);
        bot.lmb_down = false;
    }
    if out.lmb_tap && !out.lmb {
        hands.mouse.press(MouseButton::Left);
        bot.tap_lmb_down = true;
    }
    if out.rmb_tap {
        hands.mouse.press(MouseButton::Right);
        bot.rmb_down = true;
    }
    hands.motion.delta = out.look;
    if out.wheel != 0
        && let Ok(window) = w.windows.single()
    {
        hands.wheel.write(MouseWheel {
            unit: MouseScrollUnit::Line,
            x: 0.0,
            y: -(out.wheel as f32),
            window,
        });
    }
    if out.click.is_some() {
        pending.0 = out.click;
    }
    if let Some(at) = out.pointer
        && let Ok(mut window) = hands.pointer.single_mut()
    {
        window.set_cursor_position(Some(at));
    }
}

/// The panel control `drive` pressed this frame, for [`press_controls`]
/// after the UI's focus pass.
#[derive(Resource, Default)]
pub struct PendingClick(Option<Click>);

/// A pending click lands on its control after the UI's focus pass, as the
/// pointer's press does, so the control's own handler reads it in `Update`.
#[allow(clippy::type_complexity)]
pub fn press_controls(
    mut pending: ResMut<PendingClick>,
    mut controls: Query<(
        &mut Interaction,
        Option<&super::craft::RecipeRow>,
        Option<&super::craft::CraftButton>,
        Option<&super::craft::Well>,
        Option<&super::kitchen::CookPlaque>,
        Option<&super::pack::PackCell>,
        Option<&super::forge::RecipePlaque>,
        (
            Option<&super::mission_tree::Card>,
            Option<&super::pack::TrashCell>,
        ),
    )>,
) {
    let Some(click) = pending.0.take() else {
        return;
    };
    let mut hit = false;
    for (mut interaction, row, button, well, cook, cell, smelt, (card, trash)) in &mut controls {
        let this = match &click {
            Click::Recipe(k) => row.is_some_and(|r| r.0 == *k),
            Click::CraftHand => button.is_some_and(|b| b.0 == "hand"),
            Click::CraftBench => button.is_some_and(|b| b.0 == "workbench"),
            Click::Well(k) => well.is_some_and(|w| {
                w.station == "workbench" && w.role == super::craft::Role::Pick(*k)
            }),
            Click::Cook(code) => cook.is_some_and(|c| &c.0 == code),
            Click::Smelt(code) => smelt.is_some_and(|c| &c.0 == code),
            Click::Cell(k) => cell.is_some_and(|c| c.0 == *k),
            Click::Card(k) => card.is_some_and(|c| c.0 == *k),
            Click::Trash => trash.is_some(),
        };
        if this {
            *interaction = Interaction::Pressed;
            hit = true;
        } else if *interaction == Interaction::Pressed {
            // The last click's press, left standing (no pointer comes to
            // clear it): cleared, so this click is the only one pressed.
            *interaction = Interaction::None;
        }
    }
    if !hit {
        warn!("showcase: no control for {click:?}");
    }
}

/// One frame of an act.
fn run(
    bot: &mut Bot,
    act: &Act,
    s: &Sight,
    out: &mut Intent,
    commands: &mut Commands,
    stills: Option<&PathBuf>,
) -> Status {
    // An act out in the world with a panel left open (a failed smelt left
    // the forge's up, and the mining after it stood still ten minutes):
    // Escape first, one step back to the world.
    let in_world = matches!(
        act,
        Act::Walk(_)
            | Act::Away(_)
            | Act::ToMark
            | Act::Shore
            | Act::Fresh
            | Act::Fish(_)
            | Act::Chop(_)
            | Act::Dig(..)
            | Act::Mine(..)
            | Act::Till(_)
            | Act::Plant(_)
            | Act::Pick(_)
            | Act::PickUp(_)
            | Act::Harvest(_)
            | Act::Tame(_)
            | Act::Pen
            | Act::Lead(_)
            | Act::Collect(_)
    );
    // Not while making room in the pack (it opened it on purpose).
    if in_world
        && bot.room.0 == 0
        && matches!(
            s.screen,
            Screen::Pack
                | Screen::Forge
                | Screen::Kitchen
                | Screen::Missions
                | Screen::Map
                | Screen::Guide
                | Screen::Breeding
        )
    {
        if (bot.act_s % 0.6) < s.dt * 1.5 {
            out.taps.push(KeyCode::Escape);
        }
        return Status::Running;
    }
    match act {
        Act::Wait(secs) => {
            if bot.act_s >= *secs {
                Status::Done
            } else {
                Status::Running
            }
        }
        Act::Away(metres) => {
            let from = *bot.away_from.get_or_insert(s.feet);
            let gone = from.normalize_or(Vec3::Y).distance(s.up) * PLANET_RADIUS;
            if gone >= *metres {
                bot.away_from = None;
                return Status::Done;
            }
            let reached = bot
                .away_to
                .is_none_or(|to| tangent(to - s.feet, s.up).length() < 2.0);
            if reached || bot.phase_s > 12.0 {
                bot.away_to = Some(dry_ground(s));
                bot.phase_s = 0.0;
            }
            if let Some(to) = bot.away_to {
                bot.walk_to(s, out, to, 1.5);
            }
            Status::Running
        }
        Act::Use => {
            if bot.phase == 0 && bot.act_s > 0.4 {
                out.rmb_tap = true;
                bot.next_phase();
            }
            if bot.phase == 1 && bot.phase_s > 0.6 {
                Status::Done
            } else {
                Status::Running
            }
        }
        Act::Hour(hour) => {
            let behind = (hour - s.hour).rem_euclid(24.0);
            if !(0.5..=23.5).contains(&behind) {
                Status::Done
            } else {
                Status::Running
            }
        }
        Act::Look(yaw, pitch, secs) => {
            let (heading, from_pitch, _) = *bot.look_goal.get_or_insert((
                Quat::from_axis_angle(s.up, -yaw.to_radians()) * s.heading,
                s.pitch,
                0.0,
            ));
            let k = (bot.act_s / secs.max(0.05)).min(1.0);
            let k = k * k * (3.0 - 2.0 * k);
            let target = s.heading.slerp(heading, k.min(1.0));
            let p = from_pitch + (pitch.to_radians() - from_pitch) * k;
            bot.steer(s, out, target, p);
            let there = k >= 1.0 && bot.steer(s, out, heading, pitch.to_radians());
            if there || bot.act_s > secs + 2.0 {
                Status::Done
            } else {
                Status::Running
            }
        }
        Act::Walk(secs) => {
            out.held.insert(KeyCode::KeyW);
            bot.steer(s, out, s.heading, -0.08);
            let (anchor, since) = *bot.walk_anchor.get_or_insert((s.feet, s.now));
            if s.feet.distance(anchor) > 0.8 {
                bot.walk_anchor = Some((s.feet, s.now));
            } else if s.now - since > 0.8 && ((s.now - since) % 0.8) < s.dt * 1.5 {
                out.taps.push(KeyCode::Space);
            }
            if bot.act_s >= *secs {
                Status::Done
            } else {
                Status::Running
            }
        }
        Act::Hand(name) => hand(bot, name, s, out),
        Act::Select(name) => {
            let name = name.clone();
            bot.select(s, out, &|item| wanted(&name, item))
        }
        Act::Chop(until) => chop(bot, until, s, out),
        Act::Dig(..) | Act::Mine(..) if bot.make_room(s, out) => Status::Running,
        Act::Dig(until, depth) => dig(bot, until, *depth, s, out),
        Act::Mine(until, depth) => mine(bot, until, *depth, s, out),
        Act::Focus(id) => focus(bot, id, s, out),
        Act::Craft(code, times) => craft(bot, code, *times, s, out),
        Act::Place(name) => place(bot, name, s, out),
        Act::Bench(code, times) => station(bot, Material::Workbench, code, *times, s, out),
        Act::Cook(code, times) => station(bot, Material::Campfire, code, *times, s, out),
        Act::Smelt(code, times) => station(bot, Material::Forge, code, *times, s, out),
        Act::Map(secs) => match bot.phase {
            0 => {
                if s.screen != Screen::Map {
                    out.taps.push(KeyCode::KeyM);
                }
                bot.next_phase();
                Status::Running
            }
            1 if bot.phase_s > *secs => {
                if s.screen == Screen::Map {
                    out.taps.push(KeyCode::KeyM);
                }
                bot.next_phase();
                Status::Running
            }
            2 if bot.phase_s > 0.4 => Status::Done,
            _ => Status::Running,
        },
        Act::Mark(metres) => mark(bot, *metres, s, out),
        Act::ToMark => match super::map_screen::marks_of(s.save).last().copied() {
            None => Status::Fail("no mark on the map".into()),
            Some(mark) => {
                let d = mark.direction();
                let to = d * (PLANET_RADIUS + surface_height(d).max(0.0));
                if mark.reached_far(s.feet, PLANET_RADIUS)
                    || bot.walk_to(s, out, to, pbd_core::map::MARK_NEAR_M * 0.5) == Walked::There
                {
                    Status::Done
                } else {
                    Status::Running
                }
            }
        },
        Act::Shore | Act::Fresh => match bot.phase {
            0 => {
                bot.fresh = matches!(act, Act::Fresh);
                match shore(s, bot.fresh) {
                    Some((water, stand)) => {
                        bot.goal = Some(Goal::Point(water, stand));
                        bot.next_phase();
                        Status::Running
                    }
                    None => Status::Fail("no sea within 900 m".into()),
                }
            }
            1 => {
                let Some(Goal::Point(_, stand)) = bot.goal else {
                    return Status::Fail("lost the shore".into());
                };
                if bot.walk_to(s, out, stand, 1.2) == Walked::There {
                    if feet_wet(s) {
                        if let Some(Goal::Point(water, stand)) = bot.goal {
                            info!("showcase: standing in water at the shore; two metres inland");
                            bot.goal = Some(Goal::Point(water, inland(stand, water, s.up, 2.0)));
                        }
                    } else {
                        bot.next_phase();
                    }
                }
                Status::Running
            }
            _ => {
                let Some(Goal::Point(water, stand)) = bot.goal else {
                    return Status::Fail("lost the shore".into());
                };
                let toward = tangent(water - stand, s.up);
                if bot.steer(s, out, toward, -0.15) || bot.phase_s > 3.0 {
                    Status::Done
                } else {
                    Status::Running
                }
            }
        },
        Act::Fish(until) => fish(bot, until, s, out),
        Act::Till(n) => till(bot, *n, s, out),
        Act::Plant(n) => plant(bot, *n, s, out),
        Act::Pick(n) => pick(bot, *n, s, out),
        Act::PickUp(until) => pick_up(bot, until, s, out),
        Act::Harvest(n) => harvest(bot, *n, s, out),
        Act::Tame(species) => tame(bot, species, s, out),
        Act::Pen => pen(bot, s, out),
        Act::Lead(species) => lead(bot, species, s, out),
        Act::Collect(species) => collect(bot, species, s, out),
        Act::Eat(name) => eat(bot, name, s, out),
        Act::Missions(secs) => match bot.phase {
            0 => {
                if s.screen != Screen::Missions {
                    out.taps.push(KeyCode::KeyO);
                }
                bot.next_phase();
                Status::Running
            }
            1 if bot.phase_s > 2.0 && bot.stilled.insert("missions") => {
                bot.want_still = true;
                Status::Running
            }
            1 if bot.phase_s > *secs => {
                if s.screen == Screen::Missions {
                    out.taps.push(KeyCode::KeyO);
                }
                bot.next_phase();
                Status::Running
            }
            2 if bot.phase_s > 0.4 => Status::Done,
            _ => Status::Running,
        },
        Act::Still => {
            still(bot, commands, stills);
            Status::Done
        }
    }
}

/// A still into `--showcase-stills`' directory, if it names one.
fn still(bot: &mut Bot, commands: &mut Commands, stills: Option<&PathBuf>) {
    bot.want_still = false;
    if let Some(dir) = stills {
        bot.still += 1;
        let path = dir.join(format!("{:02}-shot{:02}.png", bot.still, bot.shot + 1));
        info!("SHOWCASE_STILL {}", path.display());
        commands
            .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(path));
    }
}

fn hand(bot: &mut Bot, name: &str, s: &Sight, out: &mut Intent) -> Status {
    let Some(want) = tool_named(name) else {
        return Status::Fail(format!("no tool called {name}"));
    };
    if let Some(tool) = want
        && !s.tools.tools().contains(&tool)
    {
        return Status::Fail(format!("no {name} owned"));
    }
    match bot.phase {
        0 => {
            if s.tools.held() == want {
                return Status::Done;
            }
            out.held.insert(KeyCode::KeyG);
            bot.next_phase();
            Status::Running
        }
        1 => {
            out.held.insert(KeyCode::KeyG);
            if s.picking && s.picker == want {
                if bot.phase_s > 0.35 {
                    bot.next_phase();
                }
            } else if s.picking && bot.phase_s > 0.22 {
                out.wheel = 1;
                bot.phase_s = 0.0;
            }
            // A missed turn of the wheel: let G go and pick again.
            if bot.phase_s > 4.0 {
                bot.phase = 0;
                bot.phase_s = 0.0;
            }
            if bot.act_s > 20.0 {
                return Status::Fail(format!("the picker never showed the {name}"));
            }
            Status::Running
        }
        _ => {
            if s.tools.held() == want {
                Status::Done
            } else if bot.phase_s > 0.5 {
                bot.phase = 0;
                Status::Running
            } else {
                Status::Running
            }
        }
    }
}

/// Work trees for logs: the nearest one with wood left, walked up to, its
/// trunk aimed at and the button held until a log comes.
fn chop(bot: &mut Bot, until: &Until, s: &Sight, out: &mut Intent) -> Status {
    if bot.until_met(s, until) {
        return Status::Done;
    }
    let by_axe = s.tools.held() == Some(Tool::Axe);
    let Some(Goal::Tree(key)) = bot.goal else {
        // The nearest tree that gives.
        let toward = s.feet.normalize_or(Vec3::Y);
        let cos = (60.0 / PLANET_RADIUS).cos();
        let mut best: Option<(u32, f32)> = None;
        for r in s.fine.set.finest_records() {
            let d = Vec3::from_slice(&r.direction_height[..3]);
            let Some(d) = d.try_normalize() else { continue };
            if d.dot(toward) < cos
                || !pbd_core::flora::grows_tree(
                    &pbd_app::planet::terrain_config().trees,
                    r.spare[0],
                    r.metadata[1],
                )
                || bot.gave_up.contains(&r.key())
            {
                continue;
            }
            let taken: pbd_core::gather::TreeTaken = s
                .save
                .records
                .get(pbd_core::gather::TREE_KIND, u64::from(r.key()))
                .and_then(|rec| rec.read())
                .unwrap_or_default();
            if s.gather.0.tree_gives(taken.logs, by_axe).is_err() {
                continue;
            }
            let base = d * (PLANET_RADIUS + r.direction_height[3]);
            let dist = base.distance(s.feet);
            if best.is_none_or(|b| dist < b.1) {
                best = Some((r.key(), dist));
            }
        }
        let Some((key, _)) = best else {
            return Status::Fail("no tree with wood left within 60 m".into());
        };
        bot.goal = Some(Goal::Tree(key));
        bot.phase = 0;
        bot.phase_s = 0.0;
        return Status::Running;
    };
    let Some(base) = bot.top_of(s, key) else {
        bot.gave_up.insert(key);
        bot.goal = None;
        return Status::Running;
    };
    let taken: pbd_core::gather::TreeTaken = s
        .save
        .records
        .get(pbd_core::gather::TREE_KIND, u64::from(key))
        .and_then(|rec| rec.read())
        .unwrap_or_default();
    if s.gather.0.tree_gives(taken.logs, by_axe).is_err() {
        bot.gave_up.insert(key);
        bot.goal = None;
        bot.count += 1;
        return Status::Running;
    }
    let up = base.normalize();
    match bot.phase {
        0 => {
            // Stand a pace off the trunk, on this side.
            let away = tangent(s.feet - base, up).normalize_or(-s.heading);
            let stand = base + away * 1.7;
            match bot.walk_to(s, out, stand, 0.5) {
                Walked::There => bot.next_phase(),
                Walked::Stuck if bot.act_s > 40.0 => {
                    bot.gave_up.insert(key);
                    bot.goal = None;
                }
                _ => {}
            }
            if bot.phase_s > 30.0 {
                bot.gave_up.insert(key);
                bot.goal = None;
            }
            Status::Running
        }
        _ => {
            let trunk = base + up * 1.3;
            let aimed = bot.aim_at(s, out, trunk);
            if aimed || bot.lmb_down {
                out.lmb = true;
            }
            if bot.phase_s > 9.0 {
                warn!("showcase: no log from tree {key}");
                bot.gave_up.insert(key);
                bot.goal = None;
            }
            Status::Running
        }
    }
}

/// Dig fresh ground ahead, `depth` blocks down each cell, until.
fn dig(bot: &mut Bot, until: &Until, depth: u32, s: &Sight, out: &mut Intent) -> Status {
    if bot.until_met(s, until) {
        return Status::Done;
    }
    let dug = s.tally.dug - bot.mark.dug;
    if dug > bot.count {
        // A block came out: the next one down, or the next cell.
        bot.count = dug;
        bot.tries += 1;
        bot.phase_s = 0.0;
        if let Some(Goal::Cell(key)) = bot.goal
            && let Some(top) = bot.top_of(s, key)
            && !bot.holes.iter().any(|h| h.distance(top) < 1.0)
        {
            bot.holes.push(top);
        }
        if bot.tries >= depth {
            if let Some(Goal::Cell(key)) = bot.goal {
                bot.used.insert(key);
            }
            bot.goal = None;
            bot.tries = 0;
        }
        return Status::Running;
    }
    let Some(Goal::Cell(key)) = bot.goal else {
        bot.tries = 0;
        match bot.cell_ahead(s, 2.2, 4.2) {
            Some((key, _)) => {
                bot.goal = Some(Goal::Cell(key));
                bot.phase = 0;
                bot.phase_s = 0.0;
            }
            None => {
                // Somewhere fresh, round the holes already dug.
                bot.goal = None;
                bot.roam(s, out);
            }
        }
        return Status::Running;
    };
    let Some(top) = bot.top_of(s, key) else {
        bot.used.insert(key);
        bot.goal = None;
        return Status::Running;
    };
    let point = top - s.up * 0.3;
    let aimed = bot.aim_at(s, out, point);
    let on = bot.aimed_cell(s).is_some_and(|(c, _)| c == key);
    // What the aimed block is: soft ground by hand or a shovel, stone by a
    // pickaxe; anything else is left.
    let breakable = bot
        .aimed_cell(s)
        .and_then(|b| digging::material_of(s.fine, s.save, b))
        .is_some_and(|m| match s.tools.held() {
            // A pickaxe goes down through the soil to the stone under it, as
            // a player digging into the ground does.
            Some(Tool::Pickaxe) => matches!(
                m,
                Material::Stone
                    | Material::Rock
                    | Material::Ore
                    | Material::Grass
                    | Material::DryGrass
                    | Material::JungleGrass
                    | Material::Dirt
                    | Material::Soil
                    | Material::Sand
            ),
            _ => matches!(
                m,
                Material::Grass
                    | Material::DryGrass
                    | Material::JungleGrass
                    | Material::Dirt
                    | Material::Soil
                    | Material::Sand
                    | Material::DesertSand
            ),
        });
    if aimed && on && breakable {
        out.lmb = true;
    } else if aimed && on && !bot.lmb_down && bot.phase_s > 0.6 {
        bot.used.insert(key);
        bot.goal = None;
    } else if bot.lmb_down && on {
        out.lmb = true;
    }
    // Aimed and something else in the way (a nearer rise): another cell.
    if aimed && !on && !bot.lmb_down && bot.phase_s > 1.5 {
        bot.used.insert(key);
        bot.goal = None;
    }
    if bot.phase_s > 8.0 {
        bot.used.insert(key);
        bot.goal = None;
    }
    Status::Running
}

/// `Mine`: a stair cut down into the ground with the tool in hand, as a
/// player mines: the cell ahead cleared at head and feet (and one lower
/// while the stair still goes down), then a step into it; level once it is
/// `depth` layers under where it began. A block it cannot break turns the
/// stair a sixth to the right.
fn mine(bot: &mut Bot, until: &Until, depth: u32, s: &Sight, out: &mut Intent) -> Status {
    if bot.until_met(s, until) {
        return Status::Done;
    }
    let Some(c) = s.contact.finest_cell(s.up) else {
        return Status::Running;
    };
    let records = s.fine.set.finest_records();
    let altitude = s.feet.length() - PLANET_RADIUS;
    let Some(feet_layer) = pbd_core::column::layer_at(altitude + 0.2) else {
        return Status::Running;
    };
    let dir = *bot.mine_dir.get_or_insert(s.heading);
    let dir = tangent(dir, s.up).normalize_or(s.heading);
    // Mining stays near where it began (the forge and the bench are there):
    // past 25 m the stair turns back round toward the start, a winding
    // working rather than a tunnel off across the land.
    let start = *bot.away_from.get_or_insert(s.feet);
    let back = tangent(start - s.feet, s.up);
    let dir = if back.length() > 25.0 && back.normalize().dot(dir) < 0.3 {
        let home = Quat::from_axis_angle(s.up, -0.6) * back.normalize();
        bot.mine_dir = Some(home);
        home
    } else {
        dir
    };
    // The neighbour most along the stair.
    let nb = &s.fine.set.finest_neighbors;
    let centre_of = |i: usize| Vec3::from_slice(&records[i].direction_height[..3]).normalize();
    let here = centre_of(c);
    let Some(&next) = nb[c][..records[c].degree()].iter().max_by(|a, b| {
        let da = tangent(centre_of(**a as usize) - here, s.up)
            .normalize_or_zero()
            .dot(dir);
        let db = tangent(centre_of(**b as usize) - here, s.up)
            .normalize_or_zero()
            .dot(dir);
        da.total_cmp(&db)
    }) else {
        return Status::Running;
    };
    let next = next as usize;
    let key = records[next].key();
    // Down until the feet are `depth` layers under the ground the cell
    // ahead was made with.
    let surface =
        pbd_core::column::layer_at(records[next].direction_height[3]).unwrap_or(feet_layer);
    let down = feet_layer + depth as usize > surface + 1;
    let floor = if down {
        feet_layer.saturating_sub(1)
    } else {
        feet_layer
    };
    // Top first, so nothing is left hanging over the step.
    let solid = |layer: usize| {
        digging::material_of(s.fine, s.save, (key, layer))
            .filter(|m| m != &Material::Air && !m.is_water() && !m.is_lamp())
    };
    let block = (floor..=feet_layer + 1).rev().find(|&l| solid(l).is_some());
    let point_of = |layer: usize| {
        centre_of(next) * (PLANET_RADIUS + pbd_core::column::layer_altitude(layer) + 0.5)
    };
    // Ore showing in the walls round the stair is cut first, as a player
    // follows a vein.
    let ore_at =
        |k: u32, l: usize| digging::material_of(s.fine, s.save, (k, l)) == Some(Material::Ore);
    if bot.ore_goal.is_some_and(|(k, l)| !ore_at(k, l)) {
        bot.ore_goal = None;
        bot.phase_s = 0.0;
    }
    if bot.ore_goal.is_none() {
        let mut best: Option<((u32, usize), f32)> = None;
        for i in std::iter::once(c).chain(nb[c][..records[c].degree()].iter().map(|&n| n as usize))
        {
            let k = records[i].key();
            for l in feet_layer.saturating_sub(1)..=feet_layer + 2 {
                if !ore_at(k, l) || bot.ore_gave_up.contains(&(k, l)) {
                    continue;
                }
                let at = centre_of(i) * (PLANET_RADIUS + pbd_core::column::layer_altitude(l) + 0.5);
                let d = at.distance(s.eye);
                if d < 3.2 && best.is_none_or(|b| d < b.1) {
                    best = Some(((k, l), d));
                }
            }
        }
        if let Some((goal, _)) = best {
            bot.ore_goal = Some(goal);
            bot.phase_s = 0.0;
        }
    }
    if let Some((k, l)) = bot.ore_goal {
        if bot.fetch_tool(s, out, Some(Tool::Pickaxe)) {
            return Status::Running;
        }
        let i = records.iter().position(|r| r.key() == k);
        if let Some(i) = i {
            let at = centre_of(i) * (PLANET_RADIUS + pbd_core::column::layer_altitude(l) + 0.5);
            let aimed = bot.aim_at(s, out, at);
            let on = bot.aimed_cell(s) == Some((k, l));
            if on && (aimed || bot.lmb_down) {
                out.lmb = true;
            } else if bot.phase_s > 2.5 {
                // Out of the crosshair's reach from here.
                bot.ore_gave_up.insert((k, l));
                bot.ore_goal = None;
            }
            let dug = s.tally.dug - bot.mark.dug;
            if dug > bot.count {
                bot.count = dug;
                bot.phase_s = 0.0;
            }
            return Status::Running;
        }
        bot.ore_goal = None;
    }
    // Ground under a standing tree does not dig ("Cut down the tree
    // first"), and a station is the player's own: the stair turns from
    // either at once.
    if let Some(layer) = block
        && (digging::under_standing_tree(s.fine, s.save, s.gather.0.axe_logs, (key, layer))
            || solid(layer).is_some_and(|m| pbd_core::station::station_of(m).is_some()))
    {
        bot.mine_dir = Some(Quat::from_axis_angle(s.up, -std::f32::consts::FRAC_PI_3) * dir);
        bot.tries += 1;
        bot.away_from = Some(s.feet);
        if bot.tries > 6 {
            // Trees all round: walk on until the ground ahead is clear.
            out.held.insert(KeyCode::KeyW);
            bot.steer(s, out, s.heading, -0.2);
        }
        return Status::Running;
    }
    match block {
        Some(layer) => {
            bot.tries = 0;
            // The right tool for this block (owner, 2026-11-19, on a
            // pickaxe in soil: "kinda wrong tool").
            if let Some(m) = solid(layer)
                && bot.fetch_tool(s, out, Bot::tool_for(m))
            {
                return Status::Running;
            }
            let aimed = bot.aim_at(s, out, point_of(layer));
            // Any block of the stair's cell the crosshair is on is one to cut.
            let on = bot.aimed_cell(s).is_some_and(|(k, l)| {
                k == key && l >= floor && l <= feet_layer + 1 && solid(l).is_some()
            });
            if (aimed || bot.lmb_down) && on {
                out.lmb = true;
            } else if bot.lmb_down {
                // Aimed elsewhere mid-break: a nearer block in the way.
                let near = bot
                    .aimed_cell(s)
                    .is_some_and(|(k, l)| k == key || (l > floor && l <= feet_layer + 1));
                out.lmb = near;
            }
            if bot.phase_s > 10.0 {
                // A block the tool cannot break: turn the stair.
                info!(
                    "showcase: mine: cell {key} layer {layer} ({:?}) will not break; feet on layer {feet_layer}                      in cell {}, aimed {:?} ({:?}), aimed there {aimed}; turning",
                    solid(layer),
                    records[c].key(),
                    bot.aimed_cell(s),
                    bot.aimed_cell(s)
                        .and_then(|b| digging::material_of(s.fine, s.save, b)),
                );
                bot.mine_dir =
                    Some(Quat::from_axis_angle(s.up, -std::f32::consts::FRAC_PI_3) * dir);
                bot.phase_s = 0.0;
                bot.away_from = Some(s.feet);
            }
        }
        None => {
            // Step in; a hop if it catches, and if it still will not go
            // (a trunk, a ledge), the stair turns a sixth to the right.
            let stand = point_of(floor) - s.up * 0.5;
            out.held.insert(KeyCode::KeyW);
            bot.steer(s, out, tangent(stand - s.feet, s.up), -0.35);
            if bot.phase_s > 1.5 && bot.phase_s - s.dt <= 1.5 {
                out.taps.push(KeyCode::Space);
            }
            if bot.phase_s > 3.5 {
                info!("showcase: mine: cannot step into cell {key}; turning the stair");
                bot.mine_dir =
                    Some(Quat::from_axis_angle(s.up, -std::f32::consts::FRAC_PI_3) * dir);
                bot.phase_s = 0.0;
                // Home is here now: the stay-near rule must not turn it
                // straight back into the cell it could not enter.
                bot.away_from = Some(s.feet);
            }
        }
    }
    // Any block out resets the clock.
    let dug = s.tally.dug - bot.mark.dug;
    if dug > bot.count {
        bot.count = dug;
        bot.phase_s = 0.0;
    }
    // Each new cell stood in resets it too.
    if !matches!(bot.goal, Some(Goal::Cell(k)) if k == records[c].key()) {
        bot.goal = Some(Goal::Cell(records[c].key()));
        bot.phase_s = 0.0;
    }
    Status::Running
}

/// `Focus`: pick the mission to be on, as a player does: open the mission
/// tree (O), fit it to the screen (F), click the mission's card, shut it.
/// Nothing to do when it is already the one on the HUD.
fn focus(bot: &mut Bot, id: &str, s: &Sight, out: &mut Intent) -> Status {
    let on = |s: &Sight| {
        s.missions
            .progress
            .focused(s.chain)
            .is_some_and(|m| m.id == id)
    };
    match bot.phase {
        0 => {
            // Already the one shown, or already done: nothing to pick.
            if (on(s) || s.missions.progress.is_done(id)) && s.screen != Screen::Missions {
                return Status::Done;
            }
            if s.screen != Screen::Missions {
                out.taps.push(KeyCode::KeyO);
            }
            bot.next_phase();
        }
        1 if bot.phase_s > 0.8 => {
            out.taps.push(KeyCode::KeyF);
            bot.next_phase();
        }
        2 if bot.phase_s > 0.6 => {
            let Some(k) = s.chain.missions.iter().position(|m| m.id == id) else {
                return Status::Fail(format!("no mission {id}"));
            };
            let Some(&(_, at)) = s.cards.iter().find(|(c, _)| *c == k) else {
                return Status::Fail(format!("no card for {id}"));
            };
            out.pointer = Some(at);
            bot.next_phase();
        }
        3 if bot.phase_s > 0.4 => {
            // The card itself pressed, as every other control is: a pointer
            // the real mouse moves off made the click a pan.
            if let Some(k) = s.chain.missions.iter().position(|m| m.id == id) {
                out.click = Some(Click::Card(k));
            }
            out.lmb_tap = true;
            bot.next_phase();
        }
        4 if bot.phase_s > 1.2 => {
            if !on(s) {
                // Twice missed: on without it. A mission counts whether or
                // not it is the HUD's; the five-minute check flags the HUD.
                if bot.tries >= 1 {
                    warn!("showcase: clicking {id} did not make it the mission; going on");
                    // The tree shut first: no walking with a menu up.
                    if s.screen == Screen::Missions {
                        out.taps.push(KeyCode::KeyO);
                    }
                    return Status::Done;
                }
                bot.tries += 1;
                bot.phase = 0;
                bot.phase_s = 0.0;
                return Status::Running;
            }
            if bot.stilled.insert("focus") {
                bot.want_still = true;
            }
            out.taps.push(KeyCode::KeyO);
            bot.next_phase();
        }
        5 if bot.phase_s > 0.4 && s.screen != Screen::Missions => return Status::Done,
        _ => {}
    }
    if bot.act_s > 15.0 {
        return Status::Fail(format!("could not pick {id} on the tree"));
    }
    Status::Running
}

/// The pack's Craft panel: open the pack, click the recipe's row in the
/// list, press CRAFT once per make and wait for the hands to finish, shut it.
/// Whether a recipe input takes a carried item: the same item, or any fish
/// for the cook's "fish" (`cooking::ANY_FISH`).
fn input_takes(input: Item, carried: Item) -> bool {
    input == carried || (input == pbd_core::cooking::ANY_FISH && matches!(carried, Item::Fish(_)))
}

/// What a recipe of `station` lacks from what is carried: the first input
/// short, and how many it needs. `None` when it can be made.
fn short_of(s: &Sight, station: &str, code: &str) -> Option<(Item, u16)> {
    let recipe = s.recipes.0.at(station).find(|r| r.code == code)?;
    recipe.inputs.iter().copied().find(|&(item, n)| {
        (0..CARRIED)
            .filter_map(|i| s.slots.get(i))
            .filter(|st| input_takes(item, st.item))
            .map(|st| u32::from(st.count))
            .sum::<u32>()
            < u32::from(n)
    })
}

fn craft(bot: &mut Bot, code: &str, times: u32, s: &Sight, out: &mut Intent) -> Status {
    let Some(k) = s.recipes.0.at("hand").position(|r| r.code == code) else {
        return Status::Fail(format!("no hand recipe {code}"));
    };
    let busy = !super::craft::state_of(
        s.save,
        pbd_core::station::HAND_RECORD_ID,
        pbd_core::station::hand(),
    )
    .jobs
    .is_empty();
    match bot.phase {
        0 => {
            if s.screen != Screen::Pack {
                out.taps.push(KeyCode::KeyI);
            }
            bot.next_phase();
        }
        1 if bot.phase_s > 0.7 => {
            out.click = Some(Click::Recipe(k));
            bot.next_phase();
        }
        2 if bot.phase_s > 0.5 => bot.next_phase(),
        3 if bot.phase_s > 0.5 => {
            if bot.count >= times {
                bot.phase = 5;
                bot.phase_s = 0.0;
            } else if let Some((item, n)) = short_of(s, "hand", code) {
                // A click with an input short makes nothing: not counted.
                return Status::Fail(format!("{code} needs {n} {item:?}"));
            } else if !busy {
                out.click = Some(Click::CraftHand);
                bot.count += 1;
                if bot.stilled.insert("pack") {
                    bot.want_still = true;
                }
                bot.next_phase();
            }
        }
        4 if bot.phase_s > 0.4 && !busy => {
            bot.phase = 3;
            bot.phase_s = 0.0;
        }
        5 if bot.phase_s > 0.6 => {
            if s.screen == Screen::Pack {
                out.taps.push(KeyCode::KeyI);
            }
            bot.next_phase();
        }
        6 if bot.phase_s > 0.4 => return Status::Done,
        _ => {}
    }
    if bot.act_s > 20.0 + times as f32 * 8.0 {
        return Status::Fail(format!("{code} never finished"));
    }
    Status::Running
}

/// Set the named block down on fresh ground ahead.
fn place(bot: &mut Bot, name: &str, s: &Sight, out: &mut Intent) -> Status {
    // A station already standing, and none carried: nothing to place
    // (a shot that sets it down again only if it was taken up).
    if bot.phase == 0
        && carried(s.slots, name) == 0
        && let Some(Item::Block(m)) = pbd_core::station::item_named(name)
        && pbd_core::station::station_of(m).is_some()
        && bot
            .stations(s, m)
            .is_some_and(|(_, _, at)| at.distance(s.feet) < STATION_NEAR_M)
    {
        return Status::Done;
    }
    if bot.phase == 0 {
        // How many were carried, one over (0 is unset): placed means one
        // fewer of this block, not any block placed (an up-and-out on the
        // way set dry grass, and the forge was taken as placed).
        if bot.count == 0 {
            bot.count = carried(s.slots, name) + 1;
        }
        let name = name.to_string();
        match bot.select(s, out, &|item| wanted(&name, item)) {
            Status::Done => bot.next_phase(),
            Status::Fail(why) => return Status::Fail(format!("{name}: {why}")),
            Status::Running => {}
        }
        return Status::Running;
    }
    if s.tally.placed > bot.mark.placed
        && (bot.count == 0 || carried(s.slots, name) + 1 < bot.count)
    {
        return Status::Done;
    }
    if bot.act_s > 90.0 {
        return Status::Fail(format!("{name} was never placed"));
    }
    let Some(Goal::Cell(key)) = bot.goal else {
        // Level ground or a step down: a block set on a rise is set
        // against its wall, beside the player.
        let mine = s
            .contact
            .finest_cell(s.up)
            .and_then(|i| s.fine.set.finest_records().get(i))
            .map(|r| r.key());
        let best = bot
            .fresh_cells(s, 2.6, 4.2)
            .into_iter()
            .filter(|(k, top, _)| (*top - s.feet).dot(s.up) < 0.3 && Some(*k) != mine)
            .map(|(k, top, _)| {
                let a = signed_angle(s.heading, tangent(top - s.feet, s.up), s.up).abs();
                (k, a)
            })
            .min_by(|a, b| a.1.total_cmp(&b.1));
        match best {
            Some((key, _)) => {
                bot.goal = Some(Goal::Cell(key));
                bot.phase_s = 0.0;
                bot.tries = 0;
            }
            None => bot.roam(s, out),
        }
        return Status::Running;
    };
    let Some(top) = bot.top_of(s, key) else {
        bot.goal = None;
        return Status::Running;
    };
    let aimed = bot.aim_at(s, out, top - s.up * 0.2);
    let mine = s
        .contact
        .finest_cell(s.up)
        .and_then(|i| s.fine.set.finest_records().get(i))
        .map(|r| r.key());
    let on = bot.aimed_cell(s).is_some_and(|(c, _)| c == key)
        && s.aim
            .is_some_and(|t| t.place.is_some_and(|p| Some(p.cell) != mine));
    if aimed && on && bot.phase_s > 0.3 && bot.tries < 3 {
        out.rmb_tap = true;
        bot.tries += 1;
        bot.phase_s = 0.0;
    }
    if bot.phase_s > 3.0 {
        bot.used.insert(key);
        bot.goal = None;
        bot.tries = 0;
    }
    if bot.act_s > 30.0 {
        return Status::Fail(format!("{name} was never placed"));
    }
    Status::Running
}

/// A station's panel: walk to the nearest placed one, F, press the recipe
/// once per make and wait for it, F to shut.
fn station(
    bot: &mut Bot,
    material: Material,
    code: &str,
    times: u32,
    s: &Sight,
    out: &mut Intent,
) -> Status {
    let Some((cell, layer, at)) = bot.stations(s, material) else {
        return Status::Fail(format!("no {material:?} placed"));
    };
    // The nearest workbench or campfire far off (across water, past a
    // hill): a new one here, as a player makes one rather than walk back
    // (seen 2026-11-20: a bench 92 m off through the sea, a fire 57 m).
    let (made, block) = match material {
        Material::Workbench => ("workbench", "block:workbench"),
        Material::Campfire => ("campfire", "block:campfire"),
        // A forge is made at the bench: only placed here when carried.
        Material::Forge if carried(s.slots, "block:forge") > 0 => ("forge", "block:forge"),
        _ => ("", ""),
    };
    // Only when it can be made (or is carried): else the walk to the far one.
    let affordable = carried(s.slots, block) > 0
        || (short_of(s, "hand", made).is_none()
            && (material != Material::Workbench || carried(s.slots, "planks") >= 6));
    if bot.phase == 0
        && !made.is_empty()
        && at.distance(s.feet) > STATION_NEAR_M
        && bot.injected.is_empty()
        && affordable
    {
        info!(
            "showcase: the {made} is {:.0} m off; making one here",
            at.distance(s.feet)
        );
        if carried(s.slots, block) == 0 {
            if material == Material::Workbench && carried(s.slots, "planks") < 6 {
                bot.injected.push_back(file::Act::Craft("planks".into(), 2));
            }
            bot.injected.push_back(file::Act::Craft(made.into(), 1));
            // The craft is queued, not done: its seconds pass before the
            // block is in the pack to place (seen 2026-11-20: placed at
            // once, "not carried", and a second campfire queued).
            bot.injected.push_back(file::Act::Wait(5.0));
        }
        bot.injected.push_back(file::Act::Place(block.into()));
        return Status::Running;
    }
    let row = pbd_core::station::station_of(material).expect("a station");
    let busy = !super::craft::state_of(
        s.save,
        super::craft::station_id(Some((cell, layer as u16))),
        row,
    )
    .jobs
    .is_empty();
    let open = if material == Material::Campfire {
        Screen::Kitchen
    } else {
        Screen::Forge
    };
    let k = s.recipes.0.at(row.code).position(|r| r.code == code);
    let Some(k) = k else {
        return Status::Fail(format!("the {} makes no {code}", row.name()));
    };
    match bot.phase {
        0 => {
            // A station up a wall too high to step (the bot is in a hole
            // beside it): build up to it.
            let rise = (at - s.feet).dot(s.up);
            if (bot.phase_s % 5.0) < s.dt {
                info!(
                    "showcase: walking to the {}: {:.1} m off, {:.1} m up",
                    row.name(),
                    tangent(at - s.feet, s.up).length(),
                    rise
                );
            }
            if rise > 0.9 && tangent(at - s.feet, s.up).length() < 3.5 {
                bot.climb(s, out);
            } else if bot.walk_to(s, out, at, 2.0) == Walked::There {
                info!(
                    "showcase: at the {} in cell {cell} layer {layer}, {:.1} m from the eye, {:.1} m over the feet",
                    row.name(),
                    at.distance(s.eye),
                    (at - s.feet).dot(s.up)
                );
                bot.next_phase();
            }
        }
        1 => {
            // Its top half: from below, the middle sits on the block under it.
            // A campfire is a light the eye ray passes: F goes on the
            // ground it stands on.
            let (point, want) = if material.is_lamp() {
                (at - s.up * 0.6, (cell, layer - 1))
            } else {
                (at + s.up * 0.3, (cell, layer))
            };
            bot.aim_at(s, out, point);
            let on = bot.aimed_cell(s) == Some(want);
            if on && bot.phase_s > 0.3 {
                out.taps.push(KeyCode::KeyF);
                bot.next_phase();
            } else if bot.phase_s > 3.0 {
                bot.phase = 0;
                bot.phase_s = 0.0;
                bot.walk_anchor = None;
            }
        }
        2 => {
            if s.screen == open && bot.phase_s > 0.6 {
                bot.next_phase();
            } else if bot.phase_s > 2.0 {
                bot.phase = 1;
                bot.phase_s = 0.0;
            }
        }
        3 => {
            if bot.count >= times {
                if !busy && bot.phase_s > 0.6 {
                    bot.phase = 5;
                    bot.phase_s = 0.0;
                }
            } else if !busy && bot.phase_s > 0.5 {
                // A job the station cannot start is not one made: say what
                // is short (a gate's rope, seen 2026-11-20).
                if let Some(recipe) = s.recipes.0.at(row.code).find(|r| r.code == code) {
                    let held = |item: Item| -> u32 {
                        (0..CARRIED)
                            .filter_map(|i| s.slots.get(i))
                            .filter(|st| input_takes(item, st.item))
                            .map(|st| u32::from(st.count))
                            .sum()
                    };
                    if let Some((item, n)) = recipe
                        .inputs
                        .iter()
                        .find(|(item, n)| held(*item) < u32::from(*n))
                    {
                        return Status::Fail(format!("{code} needs {n} {item:?}"));
                    }
                }
                out.click = Some(match material {
                    Material::Campfire => Click::Cook(code.to_string()),
                    Material::Forge => Click::Smelt(code.to_string()),
                    _ => Click::Well(k),
                });
                bot.next_phase();
            }
        }
        4 if bot.phase_s > 0.5 => {
            if material == Material::Workbench {
                out.click = Some(Click::CraftBench);
            }
            if bot.stilled.insert(if material == Material::Campfire {
                "cook"
            } else {
                "bench"
            }) {
                bot.want_still = true;
            }
            bot.count += 1;
            bot.phase = 3;
            bot.phase_s = 0.0;
        }
        5 => {
            if s.screen == open {
                out.taps.push(KeyCode::KeyF);
            }
            bot.next_phase();
        }
        6 if bot.phase_s > 0.4 => {
            if s.screen == Screen::Playing {
                return Status::Done;
            }
            bot.phase = 5;
        }
        _ => {}
    }
    // The walk there counts too, and a station may be a long walk back.
    if bot.act_s > 600.0 + times as f32 * 40.0 {
        return Status::Fail(format!("{code} at the {} never finished", row.name()));
    }
    Status::Running
}

/// Dry ground a few metres off, never the sea: of sixteen headings at ten
/// metres, the one with the highest ground that is not a climb, turned
/// from the heading where it can be; the way inland when the bot is in the
/// water.
/// The `Mark` act: open the map, put the pointer on dry ground `metres`
/// away, right-click it, and close the map, as a player marks a place
/// (`starter-missions` G5).
fn mark(bot: &mut Bot, metres: f32, s: &Sight, out: &mut Intent) -> Status {
    match bot.phase {
        0 => {
            let Some(goal) = far_dry_ground(s, metres) else {
                return Status::Fail(format!("no dry ground {metres} m away"));
            };
            bot.mark_goal = Some(goal);
            if s.screen != Screen::Map {
                out.taps.push(KeyCode::KeyM);
            }
            bot.next_phase();
            Status::Running
        }
        1 if bot.phase_s > 0.8 && s.screen == Screen::Map => {
            let (Some(view), Some(size), Some(goal)) = (s.map_view, s.window, bot.mark_goal) else {
                return Status::Fail("the map is not up".into());
            };
            let at = view
                .clamped(size)
                .to_screen(pbd_core::geo::project(goal), size);
            out.pointer = Some(at);
            bot.next_phase();
            Status::Running
        }
        2 if bot.phase_s > 0.3 => {
            out.rmb_tap = true;
            bot.next_phase();
            Status::Running
        }
        3 if bot.phase_s > 1.0 => {
            let set = super::map_screen::marks_of(s.save)
                .last()
                .zip(bot.mark_goal)
                .is_some_and(|(m, goal)| m.direction().distance(goal.normalize()) < 0.01);
            if !set {
                return Status::Fail("the right click set no waypoint".into());
            }
            out.taps.push(KeyCode::KeyM);
            bot.next_phase();
            Status::Running
        }
        4 if bot.phase_s > 0.4 && s.screen != Screen::Map => Status::Done,
        _ => Status::Running,
    }
}

/// Dry ground `metres` from the feet with dry ground all the way to it, the
/// bearing nearest the way the bot faces first; `None` when no bearing of
/// sixteen has it.
fn far_dry_ground(s: &Sight, metres: f32) -> Option<Vec3> {
    let east = Vec3::Y.cross(s.up).normalize_or(Vec3::X);
    let north = s.up.cross(east);
    // Dry at the end and mostly dry on the way: the route goes round or
    // wades a creek, so a straight line need not be dry all the way (a
    // pasture by the sea had no such line in 16 bearings).
    (0..32)
        .filter_map(|k| {
            let a = k as f32 / 32.0 * std::f32::consts::TAU;
            let along = east * a.cos() + north * a.sin();
            let at = |m: f32| {
                let t = m / PLANET_RADIUS;
                (s.up * t.cos() + along * t.sin()).normalize()
            };
            let wet = (1..=20)
                .filter(|&i| surface_height(at(metres * i as f32 / 20.0)) <= 0.5)
                .count();
            let dry = surface_height(at(metres)) > 0.5
                && surface_height(at(metres * 0.95)) > 0.5
                && wet <= 3;
            dry.then(|| (at(metres), signed_angle(s.heading, along, s.up).abs()))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(d, _)| d)
}

fn dry_ground(s: &Sight) -> Vec3 {
    let east = Vec3::Y.cross(s.up).normalize_or(Vec3::X);
    let north = s.up.cross(east);
    let here = surface_height(s.up);
    (0..16)
        .map(|k| {
            let a = k as f32 / 16.0 * std::f32::consts::TAU;
            let along = east * a.cos() + north * a.sin();
            let d = (s.up * PLANET_RADIUS + along * 10.0).normalize();
            let h = surface_height(d);
            let turn = signed_angle(s.heading, along, s.up).abs();
            let score = if h < 0.5 {
                -100.0
            } else {
                -(h - here - 1.0).abs() - turn * 0.5
            };
            (d * (PLANET_RADIUS + h.max(0.0)), score)
        })
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .map_or(s.feet, |(p, _)| p)
}

/// The sea's edge nearest the feet: a point on the water and where to stand
/// to cast at it.
fn shore(s: &Sight, fresh: bool) -> Option<(Vec3, Vec3)> {
    let up = s.up;
    let east = Vec3::Y.cross(up).normalize_or(Vec3::X);
    let north = up.cross(east);
    if fresh {
        return fresh_shore(s, east, north);
    }
    for r in (4..900).step_by(3) {
        let r = r as f32;
        let mut best: Option<(Vec3, f32)> = None;
        for a in 0..48 {
            let a = a as f32 / 48.0 * std::f32::consts::TAU;
            let along = east * a.cos() + north * a.sin();
            // Open water: this point and the next eight metres on are all
            // sea, not a damp hollow at the foot of a bank.
            let open = [0.0_f32, 4.0, 8.0]
                .iter()
                .all(|k| surface_height((up * PLANET_RADIUS + along * (r + k)).normalize()) < -0.6);
            if open {
                let turn = signed_angle(s.heading, along, up).abs();
                if best.is_none_or(|b| turn < b.1) {
                    best = Some((along, turn));
                }
            }
        }
        if let Some((along, _)) = best {
            // Step back toward the feet until the ground is dry.
            let mut stand = r - 5.0;
            while stand > 0.0 {
                let d = (up * PLANET_RADIUS + along * stand).normalize();
                if surface_height(d) > 0.3 {
                    break;
                }
                stand -= 1.0;
            }
            let water = (up * PLANET_RADIUS + along * (r + 4.0)).normalize() * PLANET_RADIUS;
            let ground = (up * PLANET_RADIUS + along * stand.max(0.0)).normalize();
            let ground = ground * (PLANET_RADIUS + surface_height(ground).max(0.0));
            info!("showcase: the sea is {r:.0} m off");
            return Some((water, ground));
        }
    }
    None
}

/// The nearest river or lake bank in the live ground: water standing at
/// least 0.6 m above the sea (what a catch counts as fresh) and 1.2 m deep,
/// the stand the dry cell before it on the same bearing.
fn fresh_shore(s: &Sight, east: Vec3, north: Vec3) -> Option<(Vec3, Vec3)> {
    let up = s.up;
    let at = |along: Vec3, m: f32| (up * PLANET_RADIUS + along * m).normalize();
    let heights = |d: Vec3| pbd_app::river_walk::cell_heights(Some(s.contact), d);
    let open = |d: Vec3| {
        let (ground, top) = heights(d);
        top > 0.6 && top - ground > 1.2
    };
    for r in (4..400).step_by(2) {
        let r = r as f32;
        let mut best: Option<(Vec3, f32)> = None;
        for a in 0..64 {
            let a = a as f32 / 64.0 * std::f32::consts::TAU;
            let along = east * a.cos() + north * a.sin();
            if open(at(along, r)) && open(at(along, r + 2.0)) {
                let turn = signed_angle(s.heading, along, up).abs();
                if best.is_none_or(|b| turn < b.1) {
                    best = Some((along, turn));
                }
            }
        }
        if let Some((along, _)) = best {
            let mut stand = r - 2.0;
            while stand > 0.0 {
                let (ground, top) = heights(at(along, stand));
                if top - ground < 0.05 {
                    break;
                }
                stand -= 1.0;
            }
            let d = at(along, r + 1.0);
            let water = d * (PLANET_RADIUS + heights(d).1);
            let g = at(along, stand.max(0.0));
            let ground = g * (PLANET_RADIUS + heights(g).0);
            info!("showcase: fresh water {r:.0} m off");
            return Some((water, ground));
        }
    }
    None
}

/// Whether the walker stands in water: the block at its feet, as the live
/// ground has it (a bait hole the sea has flowed into is water, whatever the
/// generated ground says). No one fishes from in the water.
fn feet_wet(s: &Sight) -> bool {
    let Some(c) = s.contact.finest_cell(s.up) else {
        return false;
    };
    let key = s.fine.set.finest_records()[c].key();
    let altitude = s.feet.length() - PLANET_RADIUS;
    pbd_core::column::layer_at(altitude + 0.3)
        .and_then(|l| digging::material_of(s.fine, s.save, (key, l)))
        .is_some_and(|m| m.is_water())
}

/// Where a float cast from `from` along `look`, after the button was held
/// `hold` seconds, comes down, as the game flies it (`fishing::Line::step`):
/// the charge's speed along the look and its lift, under gravity, until it
/// meets the water's top or dry ground on the live hexes. The landing point,
/// and the water's depth there (nought on dry ground).
fn landing(s: &Sight, from: Vec3, look: Vec3, hold: f32) -> (Vec3, f32) {
    let f = s.fishing;
    let power = (hold / f.charge_s).clamp(0.0, 1.0);
    let lerp = |(a, b): (f32, f32)| a + (b - a) * power;
    let g = pbd_app::sea::surface_gravity();
    let mut at = from;
    let mut v = look.normalize_or(s.up) * lerp(f.speed_mps) + at.normalize() * lerp(f.lift_mps);
    let dt = 1.0 / 60.0;
    for _ in 0..600 {
        let up = at.normalize();
        v -= up * g * dt;
        at += v * dt;
        let (ground, top) = pbd_app::river_walk::cell_heights(Some(s.contact), up);
        let r = at.length() - PLANET_RADIUS;
        if r <= top {
            return (at, (top - ground).max(0.0));
        }
    }
    (at, 0.0)
}

/// The shortest hold, s, whose float lands in water at least `depth` deep
/// and four metres off: a cast just far enough, as a player makes it, not
/// every one at full strength (owner, 2026-11-19); `None` when no hold does
/// from this look.
fn hold_for_water(s: &Sight, look: Vec3, depth: f32) -> Option<f32> {
    let from = s.eye;
    (1..=24)
        .map(|k| k as f32 / 24.0 * s.fishing.charge_s)
        .find(|&hold| {
            let (at, deep) = landing(s, from, look, hold);
            deep >= depth && tangent(at - s.feet, s.up).length() > 4.0
        })
}

/// The stand moved this many metres inland, away from `water`.
fn inland(stand: Vec3, water: Vec3, up: Vec3, metres: f32) -> Vec3 {
    let away = tangent(stand - water, up).normalize_or_zero();
    let moved = (stand + away * metres).normalize();
    moved * (PLANET_RADIUS + surface_height(moved).max(0.0))
}

/// Fish: to the shore, face the water, cast, watch the float, hook on the
/// bite, reel with the tension, again until.
fn fish(bot: &mut Bot, until: &Until, s: &Sight, out: &mut Intent) -> Status {
    use pbd_core::fishing::Phase;
    // No bait, no cast (the game says "Needs bait"): said at once, not
    // after forty casts the game refused.
    if carried(s.slots, "bait") == 0 && s.fishery.is_none_or(|f| f.line.phase == Phase::Ready) {
        return Status::Fail("out of bait".into());
    }
    // The rod back in hand (a walk that dug its way put the shovel there).
    if s.tools.held() != Some(Tool::Rod) {
        if !s.tools.tools().contains(&Tool::Rod) {
            return Status::Fail("no rod".into());
        }
        bot.fetch_tool(s, out, Some(Tool::Rod));
        return Status::Running;
    }
    let caught = s.tally.caught - bot.mark.caught;
    if caught > bot.count {
        bot.count = caught;
    }
    let phase = s.fishery.map(|f| f.line.phase);
    if bot.until_met(s, until) && phase.is_none_or(|p| p == Phase::Ready) {
        return Status::Done;
    }
    match bot.phase {
        0 => {
            let Some((water, stand)) = shore(s, bot.fresh) else {
                return Status::Fail("no shore found".into());
            };
            bot.goal = Some(Goal::Point(water, stand));
            bot.next_phase();
        }
        1 => {
            let Some(Goal::Point(_, stand)) = bot.goal else {
                bot.phase = 0;
                return Status::Running;
            };
            match bot.walk_to(s, out, stand, 1.2) {
                Walked::There if feet_wet(s) => {
                    // In the water: back up the shore and try again.
                    if let Some(Goal::Point(water, stand)) = bot.goal {
                        info!("showcase: standing in water at the shore; two metres inland");
                        bot.goal = Some(Goal::Point(water, inland(stand, water, s.up, 2.0)));
                    }
                }
                Walked::There => bot.next_phase(),
                _ if bot.phase_s > 60.0 => return Status::Fail("never reached the shore".into()),
                _ => {}
            }
        }
        2 => {
            // Face the water, a little down, and cast.
            let Some(Goal::Point(water, stand)) = bot.goal else {
                bot.phase = 0;
                return Status::Running;
            };
            // The bearing from the stand out to the water, wherever the feet
            // ended up: a bot that waded past the point still faces the sea.
            let toward = tangent(water - stand, s.up);
            if bot.steer(s, out, toward, -0.12) && bot.phase_s > 0.5 {
                // Where would the float land? Only cast at water: a hold that
                // lands it in water half a metre deep or more, else another
                // bearing along the shore, else another stand.
                match hold_for_water(s, s.look, 1.5) {
                    Some(hold) => {
                        bot.cast_hold = hold;
                        bot.cast_turns = 0;
                        bot.next_phase();
                    }
                    None if bot.cast_turns % 100 < 8 => {
                        bot.cast_turns += 1;
                        let k = (bot.cast_turns % 100) as f32;
                        let side = if bot.cast_turns.is_multiple_of(2) {
                            1.0
                        } else {
                            -1.0
                        };
                        let turned =
                            Quat::from_axis_angle(s.up, side * 0.35 * k.div_euclid(2.0).max(1.0))
                                * (water - stand);
                        info!(
                            "showcase: no water where the float would land; turning along the shore"
                        );
                        bot.goal = Some(Goal::Point(stand + turned, stand));
                        bot.phase_s = 0.0;
                    }
                    // Every bearing tried from here: fifteen metres along the
                    // shore and try again (a pit of its own bait digging, a
                    // bank in the way), four stands at most.
                    None if bot.cast_turns / 100 < 4 => {
                        let along =
                            s.up.cross(tangent(water - stand, s.up)).normalize_or_zero() * 15.0;
                        let moved = (stand + along).normalize();
                        let next = moved * (PLANET_RADIUS + surface_height(moved).max(0.0));
                        info!(
                            "showcase: no water within a cast here; fifteen metres along the shore"
                        );
                        bot.cast_turns = (bot.cast_turns / 100 + 1) * 100;
                        bot.goal = Some(Goal::Point(water + along, next));
                        bot.phase = 1;
                        bot.phase_s = 0.0;
                    }
                    None => return Status::Fail("no water within a cast of this shore".into()),
                }
            }
        }
        3 if feet_wet(s) => {
            bot.phase = 1;
            bot.phase_s = 0.0;
        }
        3 => {
            // Charge: the button held as long as lands the float on water,
            // then let go.
            out.lmb = bot.phase_s < bot.cast_hold;
            if bot.phase_s > bot.cast_hold + 0.2 {
                bot.line_was_out = false;
                bot.next_phase();
            }
        }
        _ => {
            let Some(phase) = phase else {
                return Status::Running;
            };
            let line = &s.fishery.expect("a fishery").line;
            match phase {
                Phase::Bite => {
                    bot.bite_s += s.dt;
                    // A player's reaction time.
                    if bot.bite_s > 0.25 && !bot.tap_lmb_down {
                        out.lmb_tap = true;
                    }
                }
                Phase::Hooked => {
                    bot.bite_s = 0.0;
                    if bot.stilled.insert("hooked") {
                        bot.want_still = true;
                    }
                    // Reel while the line takes it; ease off when it runs.
                    let hold = if bot.lmb_down {
                        line.tension < 0.8
                    } else {
                        line.tension < 0.45
                    };
                    out.lmb = hold;
                }
                Phase::Ready if bot.line_was_out || bot.phase_s > 3.0 => {
                    // Landed, lost or never out: cast again.
                    bot.phase = 2;
                    bot.phase_s = 0.0;
                    bot.tries += 1;
                }
                Phase::Floating | Phase::Nibble if bot.phase_s > 35.0 => {
                    // Nothing biting here: wind in and try along the shore.
                    out.rmb_tap = true;
                    bot.phase = 2;
                    bot.phase_s = 0.0;
                    bot.tries += 1;
                    // Cast along the shore instead, at open water only: a
                    // turn from the first cast, one side then the other.
                    if let Some(Goal::Point(water, stand)) = bot.goal {
                        let first = *bot.fish_from.get_or_insert((water, stand));
                        let sides = [0.45_f32, -0.45, 0.9, -0.9, 0.0];
                        let k = bot.tries as usize % sides.len();
                        let turned = Quat::from_axis_angle(s.up, sides[k]) * (first.0 - first.1);
                        let at = first.1 + turned;
                        let open = (1..=3).all(|m| {
                            surface_height((first.1 + turned * (m as f32 / 3.0)).normalize()) < -0.6
                                || m == 1
                        });
                        let at = if open { at } else { first.0 };
                        bot.goal = Some(Goal::Point(at, first.1));
                    }
                }
                _ => bot.bite_s = 0.0,
            }
            if phase != Phase::Ready {
                bot.line_was_out = true;
            }
            // Twelve casts and nothing: the fish here are caught or shy;
            // fifteen metres along the shore, as a player moves on (seen
            // 2026-11-20: two caught, then forty casts and no bite).
            if bot.tries > 0
                && bot.tries.is_multiple_of(12)
                && phase == Phase::Ready
                && let Some(Goal::Point(water, stand)) = bot.goal
            {
                let along = s.up.cross(tangent(water - stand, s.up)).normalize_or_zero() * 15.0;
                let moved = (stand + along).normalize();
                let stand = moved * (PLANET_RADIUS + surface_height(moved).max(0.0));
                info!("showcase: no bite in twelve casts; fifteen metres along the shore");
                bot.goal = Some(Goal::Point(water + along, stand));
                bot.fish_from = None;
                bot.tries += 1;
                bot.phase = 1;
                bot.phase_s = 0.0;
                return Status::Running;
            }
            if bot.tries > 60 {
                return Status::Fail("sixty casts and no catch".into());
            }
        }
    }
    Status::Running
}

/// Till a small field with the shovel: a fresh cell ahead and its grassy
/// neighbours, F on each.
fn till(bot: &mut Bot, n: u32, s: &Sight, out: &mut Intent) -> Status {
    if s.tools.held() != Some(Tool::Shovel) {
        return Status::Fail("the shovel is not in hand".into());
    }
    let tilled = s.tally.tilled - bot.mark.tilled;
    if tilled > bot.count {
        bot.count = tilled;
        if let Some(Goal::Cell(key)) = bot.goal {
            bot.field.push(key);
            bot.used.insert(key);
        }
        bot.goal = None;
    }
    if bot.count >= n {
        return Status::Done;
    }
    let Some(Goal::Cell(key)) = bot.goal else {
        // Next to the field so far, or ahead for the first.
        let cells = bot.fresh_cells(s, 1.8, 4.6);
        let last = bot.field.last().copied();
        let anchor = last.and_then(|k| bot.top_of(s, k));
        let next = match anchor {
            Some(a) => cells
                .into_iter()
                .min_by(|x, y| x.1.distance(a).total_cmp(&y.1.distance(a)))
                .map(|c| c.0),
            None => bot.cell_ahead(s, 2.4, 4.0).map(|c| c.0),
        };
        match next {
            Some(key) => {
                bot.goal = Some(Goal::Cell(key));
                bot.phase_s = 0.0;
            }
            None => {
                bot.roam(s, out);
            }
        }
        return Status::Running;
    };
    let Some(top) = bot.top_of(s, key) else {
        bot.used.insert(key);
        bot.goal = None;
        return Status::Running;
    };
    let aimed = bot.aim_at(s, out, top - s.up * 0.25);
    let on = bot.aimed_cell(s).is_some_and(|(c, _)| c == key);
    if aimed && on && bot.phase_s > 0.35 && bot.tries < 2 {
        out.taps.push(KeyCode::KeyF);
        bot.tries += 1;
        bot.phase_s = 0.0;
    }
    if bot.phase_s > 2.0 {
        bot.used.insert(key);
        bot.goal = None;
        bot.tries = 0;
    }
    if bot.act_s > 60.0 {
        return Status::Fail(format!("tilled {} of {n}", bot.count));
    }
    Status::Running
}

/// Sow the field's tilled cells with the seed in hand.
fn plant(bot: &mut Bot, n: u32, s: &Sight, out: &mut Intent) -> Status {
    if bot.phase == 0 {
        // Wheat when there is any: the field's bread comes from it.
        let seed = if carried(s.slots, "wheat-seed") >= n {
            "wheat-seed"
        } else {
            "seed"
        };
        match bot.select(s, out, &|item| wanted(seed, item)) {
            Status::Done => bot.next_phase(),
            Status::Fail(why) => return Status::Fail(format!("seed: {why}")),
            Status::Running => {}
        }
        return Status::Running;
    }
    let planted = s.tally.planted - bot.mark.planted;
    if planted > bot.count {
        bot.count = planted;
        bot.tries = 0;
        bot.phase_s = 0.0;
    }
    if bot.count >= n {
        return Status::Done;
    }
    // Round the field again for a cell it could not see or reach.
    let unplanted = bot.field.len().saturating_sub(bot.count as usize).max(1);
    let at = bot.count as usize + (bot.tries as usize / 3) % unplanted;
    let Some(&key) = bot.field.get(at) else {
        return Status::Fail(format!("planted {} of {n}: no tilled cell left", bot.count));
    };
    let Some(top) = bot.top_of(s, key) else {
        bot.tries += 3;
        return Status::Running;
    };
    let off = tangent(top - s.feet, s.up).length();
    if off > 3.6 {
        bot.walk_to(s, out, top, 2.6);
        bot.phase_s = 0.0;
        return Status::Running;
    }
    if off < 2.2 {
        // On it or at its edge, where the body is in the cell the seed
        // goes in: a step back.
        out.held.insert(KeyCode::KeyS);
    }
    bot.aim_at(s, out, top - s.up * 0.2);
    let on = bot.aimed_cell(s).is_some_and(|(c, _)| c == key);
    if on && off >= 2.0 && bot.phase_s > 0.5 {
        out.rmb_tap = true;
        bot.tries += 1;
        bot.phase_s = 0.0;
    } else if bot.phase_s > 4.0 {
        // Out of sight from here: the next cell.
        bot.tries += 3;
        bot.phase_s = 0.0;
    }
    if bot.act_s > 120.0 {
        return Status::Fail(format!("planted {} of {n}", bot.count));
    }
    Status::Running
}

/// Pick wild plants: walk to each, F.
fn pick(bot: &mut Bot, n: u32, s: &Sight, out: &mut Intent) -> Status {
    let picked = s.tally.picked - bot.mark.picked;
    if picked / 2 > bot.count {
        bot.count = picked / 2;
        if let Some(Goal::Cell(key)) = bot.goal {
            bot.used.insert(key);
        }
        bot.goal = None;
    }
    if bot.count >= n {
        return Status::Done;
    }
    let Some(Goal::Cell(key)) = bot.goal else {
        let toward = s.feet.normalize_or(Vec3::Y);
        let cos = (70.0 / PLANET_RADIUS).cos();
        let mut best: Option<(u32, f32)> = None;
        for r in s.fine.set.finest_records() {
            let d = Vec3::from_slice(&r.direction_height[..3]);
            let Some(d) = d.try_normalize() else { continue };
            if d.dot(toward) < cos || bot.gave_up.contains(&r.key()) {
                continue;
            }
            let Some(layer) = pbd_core::column::layer_at(r.direction_height[3] - 0.5) else {
                continue;
            };
            if super::farm::wild_at(s.fine, s.save, s.farm, r.key(), layer).is_none()
                || digging::under_standing_tree(
                    s.fine,
                    s.save,
                    s.gather.0.axe_logs,
                    (r.key(), layer),
                )
            {
                continue;
            }
            let dist = d.distance(toward) * PLANET_RADIUS;
            if best.is_none_or(|b| dist < b.1) {
                best = Some((r.key(), dist));
            }
        }
        let Some((key, _)) = best else {
            return Status::Fail("no wild plant within 70 m".into());
        };
        bot.goal = Some(Goal::Cell(key));
        bot.phase = 0;
        bot.phase_s = 0.0;
        bot.tries = 0;
        return Status::Running;
    };
    let Some(top) = bot.top_of(s, key) else {
        bot.gave_up.insert(key);
        bot.goal = None;
        return Status::Running;
    };
    // Three presses of F that picked nothing: F offers nothing there (the
    // prompt is the player's tell), so try another plant.
    if bot.tries >= 3 && bot.phase_s > 0.5 {
        info!("showcase: F picks nothing on cell {key}; another plant");
        bot.gave_up.insert(key);
        bot.goal = None;
        return Status::Running;
    }
    match bot.phase {
        0 => match bot.walk_to(s, out, top, 2.2) {
            Walked::There => bot.next_phase(),
            _ if bot.phase_s > 40.0 => {
                bot.gave_up.insert(key);
                bot.goal = None;
            }
            _ => {}
        },
        _ => {
            let aimed = bot.aim_at(s, out, top - s.up * 0.25);
            let on = bot.aimed_cell(s).is_some_and(|(c, _)| c == key);
            if aimed && on && bot.phase_s > 0.4 {
                out.taps.push(KeyCode::KeyF);
                bot.phase_s = -1.0;
                bot.tries += 1;
            }
            if bot.phase_s > 3.0 {
                bot.gave_up.insert(key);
                bot.goal = None;
            }
        }
    }
    Status::Running
}

/// `PickUp` (`gather-by-hand`): walk to the nearest stone, or bush, shrub or
/// fern, that gives what `until` wants, look at it and press F, until.
fn pick_up(bot: &mut Bot, until: &Until, s: &Sight, out: &mut Intent) -> Status {
    use pbd_core::scatter::Kind;
    let gathered = s.tally.gathered - bot.mark.gathered;
    if gathered > bot.count {
        bot.count = gathered;
        bot.clutter_goal = None;
        bot.phase = 0;
        bot.tries = 0;
    }
    if bot.until_met(s, until) {
        return Status::Done;
    }
    let Some(odds) = s.odds else {
        return Status::Fail("no clutter odds".into());
    };
    let wants = |kind: Kind| match until {
        Until::Have(name, _) => kind.gives().0 == name,
        _ => true,
    };
    let Some((key, kind, centre)) = bot.clutter_goal else {
        let toward = s.feet.normalize_or(Vec3::Y);
        let cos = (80.0 / PLANET_RADIUS).cos();
        let mut best: Option<((u32, Kind, Vec3), f32)> = None;
        for r in s.fine.set.finest_records() {
            let Some(d) = Vec3::from_slice(&r.direction_height[..3]).try_normalize() else {
                continue;
            };
            if d.dot(toward) < cos || r.direction_height[3] < 0.3 {
                continue;
            }
            let taken = pbd_app::clutter::taken_at(s.save, r.key());
            for piece in pbd_app::clutter::pieces_of(r, &odds) {
                if !wants(piece.kind)
                    || taken.gone(piece.kind, s.day)
                    || bot.clutter_gave_up.contains(&(r.key(), piece.kind))
                {
                    continue;
                }
                let dist = piece.seat.distance(s.feet);
                if best.is_none_or(|b| dist < b.1) {
                    best = Some(((r.key(), piece.kind, piece.centre), dist));
                }
            }
        }
        let Some((goal, _)) = best else {
            return Status::Fail("no stone or shrub within 80 m".into());
        };
        bot.clutter_goal = Some(goal);
        bot.phase = 0;
        bot.phase_s = 0.0;
        bot.tries = 0;
        return Status::Running;
    };
    let give_up = |bot: &mut Bot| {
        bot.clutter_gave_up.insert((key, kind));
        bot.clutter_goal = None;
        bot.phase = 0;
        bot.tries = 0;
    };
    match bot.phase {
        0 => match bot.walk_to(s, out, centre, 1.4) {
            Walked::There => bot.next_phase(),
            _ if bot.phase_s > 40.0 => give_up(bot),
            _ => {}
        },
        _ => {
            let aimed = bot.aim_at(s, out, centre);
            // A piece that three presses of F do not take (out of reach,
            // or already gone) is left for the next.
            if aimed && bot.phase_s > 0.4 && bot.tries < 3 {
                out.taps.push(KeyCode::KeyF);
                bot.tries += 1;
                bot.phase_s = -1.0;
            }
            if bot.phase_s > 3.0 || (bot.tries >= 3 && bot.phase_s > 1.0) {
                give_up(bot);
            }
        }
    }
    Status::Running
}

/// Harvest the field: F on each of its cells.
fn harvest(bot: &mut Bot, n: u32, s: &Sight, out: &mut Intent) -> Status {
    // No field in mind (a run resumed after the sowing): the crops planted
    // within 150 m, as a player goes back to their field.
    if bot.field.is_empty() {
        let toward = s.feet.normalize_or(Vec3::Y);
        let cos = (150.0 / PLANET_RADIUS).cos();
        bot.field = s
            .fine
            .set
            .finest_records()
            .iter()
            .filter(|r| {
                Vec3::from_slice(&r.direction_height[..3])
                    .try_normalize()
                    .is_some_and(|d| d.dot(toward) >= cos)
            })
            .map(|r| r.key())
            .filter(|&k| super::farm::live_crop(s.fine, s.save, k).is_some())
            .collect();
        info!(
            "showcase: {} planted cells found to harvest",
            bot.field.len()
        );
    }
    let got = (s.tally.harvested - bot.mark.harvested) / 2;
    if got > bot.count {
        bot.count = got;
        bot.tries = 0;
    }
    if bot.count >= n {
        return Status::Done;
    }
    let at = bot.count as usize + bot.tries as usize / 2;
    let Some(&key) = bot.field.get(at) else {
        return Status::Fail(format!("harvested {} of {n}", bot.count));
    };
    let Some(top) = bot.top_of(s, key) else {
        bot.tries += 2;
        return Status::Running;
    };
    if tangent(top - s.feet, s.up).length() > 2.6 {
        if bot.walk_to(s, out, top, 2.2) == Walked::Stuck {
            bot.tries += 2;
        }
        bot.phase_s = 0.0;
        return Status::Running;
    }
    bot.aim_at(s, out, top - s.up * 0.2);
    let on = bot.aimed_cell(s).is_some_and(|(c, _)| c == key);
    if on && bot.phase_s > 0.6 {
        out.taps.push(KeyCode::KeyF);
        bot.tries += 1;
        bot.phase_s = 0.0;
    } else if bot.phase_s > 4.0 {
        // Out of sight from here: the next cell.
        bot.tries += 2;
        bot.phase_s = 0.0;
    }
    if bot.act_s > 90.0 {
        return Status::Fail(format!("harvested {} of {n}", bot.count));
    }
    Status::Running
}

/// The animals in view of a species (`any` for every one).
fn animals<'a>(
    s: &'a Sight,
    species: &'a str,
) -> impl Iterator<
    Item = (
        pbd_app::husbandry::Seen,
        &'a pbd_core::husbandry::LandSpecies,
    ),
> + 'a {
    s.yard
        .into_iter()
        .flat_map(|y| y.seen(s.origin))
        .filter_map(move |a| {
            let sp = s.land.get(a.species as usize)?;
            (species == "any" || sp.id == species).then_some((a, sp))
        })
}

/// Tame: find a wild animal the carried food suits, food in hand, come near
/// and stand still, and feed it when it comes, until it trusts.
/// The tame act's phase while no animal it wants is in sight.
const LOST: u32 = 99;

fn tame(bot: &mut Bot, species: &str, s: &Sight, out: &mut Intent) -> Status {
    if s.tally.tamed > bot.mark.tamed {
        return Status::Done;
    }
    // The nearest wild one whose food is carried.
    let target = animals(s, species)
        .filter(|(a, sp)| {
            a.wild
                && sp.refusal().is_none()
                && sp.keep.as_ref().is_some_and(|k| {
                    (0..CARRIED).any(|i| s.slots.get(i).is_some_and(|st| k.eats(st.item)))
                })
        })
        // A female first, for the eggs, milk or wool the herding asks
        // for later; the nearest of those.
        // The nearest, a hen before a cock unless the cock is much nearer
        // (a hen lays the eggs the herding asks for later).
        .min_by(|a, b| {
            let cost = |x: &pbd_app::husbandry::Seen| {
                x.pos.distance(s.feet) + if x.female { 0.0 } else { 15.0 }
            };
            cost(&a.0).total_cmp(&cost(&b.0))
        });
    let Some((animal, sp)) = target else {
        // None in sight: look further, for up to 90 s from losing them.
        if bot.phase != LOST {
            bot.phase = LOST;
            bot.phase_s = 0.0;
        }
        out.held.insert(KeyCode::KeyW);
        bot.steer(s, out, s.heading, -0.05);
        if bot.phase_s > 90.0 {
            return Status::Fail(format!("no wild {species} found"));
        }
        return Status::Running;
    };
    if bot.phase == LOST {
        // Found again: food in hand, then walk up as before.
        bot.phase = 0;
        bot.phase_s = 0.0;
    }
    let keep = sp.keep.clone().expect("tameable");
    if bot.phase == 0 {
        match bot.select(s, out, &|item| keep.eats(item)) {
            Status::Done => bot.next_phase(),
            Status::Fail(why) => return Status::Fail(format!("food: {why}")),
            Status::Running => {}
        }
        return Status::Running;
    }
    let dist = tangent(animal.pos - s.feet, s.up).length();
    let centre = animal.pos + animal.pos.normalize() * 0.35;
    match bot.phase {
        1 => {
            // Near, but outside the distance a moving player scares it at
            // (`taming.fear_moving_m`), then still; or a few steps in from
            // where it would not come.
            // Inside the distance it comes to a still player with food
            // (`taming.fear_moving_m`, 9 m) and outside the one food in
            // hand still scares it at (`fear_still_m`, 4.5 m).
            let far = dist > 12.0;
            let stop = if far || bot.tries == 0 {
                7.0
            } else {
                (dist - 2.5).max(2.5)
            };
            // Waiting in water bobs (a moving player to it): only on dry feet.
            if (bot.walk_to(s, out, animal.pos, stop) == Walked::There
                || (!far && bot.phase_s > 8.0))
                && !feet_wet(s)
            {
                bot.next_phase();
            }
        }
        _ => {
            // Still, food in hand, looking at it; feed it when it comes.
            bot.aim_at(s, out, centre);
            if (bot.phase_s % 4.0) < s.dt {
                info!(
                    "showcase: waiting for the {} at {dist:.1} m, {:?} in hand, {} wild in sight",
                    sp.name(),
                    s.slots.held().map(|st| st.item),
                    animals(s, species).filter(|(a, _)| a.wild).count()
                );
            }
            if dist < 2.3 && bot.phase_s > 1.1 && s.screen == Screen::Playing {
                out.taps.push(KeyCode::KeyF);
                bot.phase_s = 0.0;
            }
            if dist > 3.0 && bot.phase_s > 10.0 {
                bot.phase = 1;
                bot.phase_s = 0.0;
                bot.tries += 1;
            }
        }
    }
    if bot.act_s > 240.0 {
        return Status::Fail(format!("the {} never trusted", sp.name()));
    }
    Status::Running
}

/// A ring of fence round the cell underfoot and its neighbours, walked
/// round, the gate last.
fn pen(bot: &mut Bot, s: &Sight, out: &mut Intent) -> Status {
    let records = s.fine.set.finest_records();
    if bot.phase == 0 {
        let Some(c) = s.contact.finest_cell(s.up) else {
            return Status::Running;
        };
        let nb = &s.fine.set.finest_neighbors;
        let inner: HashSet<usize> = std::iter::once(c)
            .chain(nb[c][..records[c].degree()].iter().map(|&n| n as usize))
            .collect();
        let centre = Vec3::from_slice(&records[c].direction_height[..3]).normalize();
        let east = Vec3::Y.cross(centre).normalize_or(Vec3::X);
        let north = centre.cross(east);
        let mut edges: Vec<(f32, Vec3, Vec3)> = Vec::new();
        for &i in &inner {
            let ci = Vec3::from_slice(&records[i].direction_height[..3]).normalize();
            for &n in &nb[i][..records[i].degree()] {
                if inner.contains(&(n as usize)) {
                    continue;
                }
                let cn = Vec3::from_slice(&records[n as usize].direction_height[..3]).normalize();
                let mid = (ci + cn).normalize();
                // Well inside the inner cell, so the ray's hit is on its
                // ground and this edge is the nearest of its edges.
                let aim = (mid * 0.8 + ci * 0.2).normalize();
                let aim = aim * (PLANET_RADIUS + records[i].direction_height[3]);
                let stand = ci * (PLANET_RADIUS + records[i].direction_height[3]);
                let rel = tangent(mid - centre, centre);
                let angle = rel.dot(north).atan2(rel.dot(east));
                edges.push((angle, aim, stand));
            }
        }
        edges.sort_by(|a, b| a.0.total_cmp(&b.0));
        bot.pen_edges = edges.into_iter().map(|e| (e.1, e.2)).collect();
        bot.pen_centre = Some(centre * (PLANET_RADIUS + records[c].direction_height[3]));
        bot.pieces = pbd_core::husbandry::record::pieces(&s.save.records).len();
        bot.count = 0;
        info!("showcase: a pen of {} edges", bot.pen_edges.len());
        bot.next_phase();
        return Status::Running;
    }
    let total = bot.pen_edges.len() as u32;
    let pieces = pbd_core::husbandry::record::pieces(&s.save.records).len();
    if pieces > bot.pieces {
        bot.pieces = pieces;
        bot.count += 1;
        bot.tries = 0;
        bot.phase = 1;
        bot.phase_s = 0.0;
        bot.sel = (0, 0.0);
    }
    if bot.count >= total {
        return Status::Done;
    }
    let gate = bot.count + 1 == total;
    let (aim, stand) = bot.pen_edges[bot.count as usize];
    match bot.phase {
        1 => {
            let piece = if gate { "gate" } else { "fence" };
            match bot.select(s, out, &|item| wanted(piece, item)) {
                Status::Done => bot.next_phase(),
                Status::Fail(why) => return Status::Fail(format!("{piece}: {why}")),
                Status::Running => {}
            }
        }
        2 => {
            if bot.walk_to(s, out, stand, 0.9) != Walked::Going || bot.phase_s > 10.0 {
                bot.next_phase();
            }
        }
        _ => {
            let aimed = bot.aim_at(s, out, aim);
            if aimed && bot.phase_s > 0.3 && bot.tries < 3 {
                out.rmb_tap = true;
                bot.phase_s = -0.6;
                bot.tries += 1;
            }
            if bot.phase_s > 3.0 || (bot.tries >= 3 && bot.phase_s > 0.5) {
                bot.tries = 0;
                warn!("showcase: no piece on pen edge {}", bot.count);
                bot.count += 1;
                bot.phase = 1;
                bot.phase_s = 0.0;
            }
        }
    }
    Status::Running
}

/// Lead a kept animal into the pen: F to have it follow, walk in, F to let
/// it stay.
fn lead(bot: &mut Bot, species: &str, s: &Sight, out: &mut Intent) -> Status {
    if s.tally.penned > bot.mark.penned {
        return Status::Done;
    }
    let Some(centre) = bot.pen_centre else {
        return Status::Fail("no pen built".into());
    };
    let kept = animals(s, species).filter(|(a, _)| !a.wild).min_by(|a, b| {
        let fa = !a.0.following as u8;
        let fb = !b.0.following as u8;
        fa.cmp(&fb).then(
            a.0.pos
                .distance(s.feet)
                .total_cmp(&b.0.pos.distance(s.feet)),
        )
    });
    let Some((animal, _)) = kept else {
        return Status::Fail(format!("no kept {species} in sight"));
    };
    let centre_of = animal.pos + animal.pos.normalize() * 0.35;
    match bot.phase {
        0 => {
            if animal.following {
                bot.phase = 2;
                bot.phase_s = 0.0;
            } else if bot.walk_to(s, out, animal.pos, 1.8) == Walked::There {
                bot.next_phase();
            }
        }
        1 => {
            let aimed = bot.aim_at(s, out, centre_of);
            if aimed && bot.phase_s > 0.4 {
                out.taps.push(KeyCode::KeyF);
                bot.phase_s = -0.8;
            }
            if animal.following {
                bot.next_phase();
            } else if bot.phase_s > 4.0 {
                bot.phase = 0;
                bot.phase_s = 0.0;
            }
        }
        2 => {
            if bot.walk_to(s, out, centre, 0.8) == Walked::There {
                bot.next_phase();
            }
        }
        _ => {
            let near = animal.pos.distance(s.feet) < 2.4;
            let aimed = bot.aim_at(s, out, centre_of);
            if near && aimed && bot.phase_s > 1.0 {
                out.taps.push(KeyCode::KeyF);
                bot.phase_s = -1.0;
            }
            if !animal.following && bot.phase_s > 2.0 {
                bot.phase = 0;
                bot.phase_s = 0.0;
            }
        }
    }
    if bot.act_s > 90.0 {
        return Status::Fail(format!("the {species} never went in the pen"));
    }
    Status::Running
}

/// F on a kept animal until it gives what it has.
fn collect(bot: &mut Bot, species: &str, s: &Sight, out: &mut Intent) -> Status {
    if s.tally.collected > bot.mark.collected {
        return Status::Done;
    }
    let kept = animals(s, species).filter(|(a, _)| !a.wild).min_by(|a, b| {
        (!a.0.female as u8).cmp(&(!b.0.female as u8)).then(
            a.0.pos
                .distance(s.feet)
                .total_cmp(&b.0.pos.distance(s.feet)),
        )
    });
    let Some((animal, _)) = kept else {
        return Status::Fail(format!("no kept {species} in sight"));
    };
    let centre = animal.pos + animal.pos.normalize() * 0.35;
    match bot.phase {
        0 => {
            if bot.walk_to(s, out, animal.pos, 1.8) == Walked::There {
                bot.next_phase();
            }
        }
        _ => {
            let aimed = bot.aim_at(s, out, centre);
            if aimed && bot.phase_s > 0.5 {
                out.taps.push(KeyCode::KeyF);
                bot.phase_s = -1.5;
            }
            if animal.pos.distance(s.feet) > 2.6 {
                bot.phase = 0;
            }
        }
    }
    if bot.act_s > 60.0 {
        return Status::Fail(format!("nothing collected from the {species}"));
    }
    Status::Running
}

/// Eat a dish: in hand, the right button.
fn eat(bot: &mut Bot, name: &str, s: &Sight, out: &mut Intent) -> Status {
    if s.tally.ate > bot.mark.ate {
        return Status::Done;
    }
    if bot.phase == 0 {
        let name = name.to_string();
        match bot.select(s, out, &|item| wanted(&name, item)) {
            Status::Done => bot.next_phase(),
            Status::Fail(why) => return Status::Fail(format!("{name}: {why}")),
            Status::Running => {}
        }
        return Status::Running;
    }
    bot.steer(s, out, s.heading, 0.0);
    if bot.phase_s > 0.6 {
        out.rmb_tap = true;
        bot.phase_s = -1.0;
    }
    if bot.act_s > 10.0 {
        return Status::Fail(format!("the {name} was never eaten"));
    }
    Status::Running
}

/// The caption bar: small, bottom-centre, over the hotbar, in the kit's
/// lettering (`step-videos` 2.2). Showcase runs only.
pub fn spawn_caption(mut commands: Commands) {
    commands.spawn((
        pbd_app::ui_kit::KitText::new(Localized::empty(), pbd_app::ui_kit::LABEL)
            .outline(pbd_app::ui_kit::INK),
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            bottom: px(super::slots::HOTBAR_BOTTOM + super::slots::SLOT * 2.0 + 14.0),
            margin: UiRect::left(px(-300)),
            width: px(600),
            justify_content: JustifyContent::Center,
            ..default()
        },
        CaptionText,
    ));
}

/// No debug readout in a showcase (`step-videos` 5.4): the near-field and
/// position lines at the top left are hidden.
pub fn hide_readout(mut lines: Query<&mut Visibility, With<super::hud::NearFieldText>>) {
    for mut v in &mut lines {
        if *v != Visibility::Hidden {
            *v = Visibility::Hidden;
        }
    }
}

/// With `--showcase-stills`, one throwaway screenshot while the loading
/// screen is up, so the screenshot's pipeline is made in the prewarm and
/// not as a `LATE_PIPELINE` at the first still.
pub fn warm_stills(mut commands: Commands, player: Res<Player>, mut done: Local<bool>) {
    if *done || player.stills.is_none() {
        return;
    }
    *done = true;
    commands
        .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
        .observe(|_: On<bevy::render::view::screenshot::ScreenshotCaptured>| {});
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    /// Turn the camera by the mouse's motion, as the walker's mouse-look
    /// does in the game: yaw about up, pitch about the camera's right.
    fn look(
        motion: Res<AccumulatedMouseMotion>,
        mut cameras: Query<&mut Transform, With<pbd_app::walking::WalkingCamera>>,
    ) {
        for mut t in &mut cameras {
            let up = t.translation.normalize();
            let yaw = Quat::from_axis_angle(up, -motion.delta.x * PER_PIXEL);
            let right = t.right().as_vec3();
            let pitch = Quat::from_axis_angle(right, -motion.delta.y * PER_PIXEL);
            t.rotation = (yaw * pitch * t.rotation).normalize();
        }
    }

    /// What the input systems do at the top of a frame: last frame's edges
    /// cleared, held buttons kept.
    fn clear_edges(mut k: ResMut<ButtonInput<KeyCode>>, mut m: ResMut<ButtonInput<MouseButton>>) {
        k.clear();
        m.clear();
    }

    fn fields() -> Vec3 {
        super::super::spawn_direction(&super::super::Launch::parse(&[
            "--walk".into(),
            "--spawn".into(),
            "fields".into(),
        ]))
    }

    /// A tree on dry, level ground near the fields spawn, and a place to
    /// stand a pace east of it.
    fn tree_and_stand(fine: &PlanetFine, anchor: Vec3) -> (u32, Vec3, Vec3) {
        fine.set
            .finest_records()
            .iter()
            .filter(|r| {
                pbd_core::flora::grows_tree(
                    &pbd_app::planet::terrain_config().trees,
                    r.spare[0],
                    r.metadata[1],
                )
            })
            .filter(|r| r.direction_height[3] > 2.0)
            .filter_map(|r| {
                let d = Vec3::from_slice(&r.direction_height[..3]).normalize();
                let base = d * (PLANET_RADIUS + r.direction_height[3]);
                let east = Vec3::Y.cross(d).normalize();
                let stand_d = (base + east * 1.7).normalize();
                let h = surface_height(stand_d);
                ((h - r.direction_height[3]).abs() < 0.3).then(|| {
                    (
                        r.key(),
                        base,
                        stand_d * (PLANET_RADIUS + h),
                        d.distance(anchor),
                    )
                })
            })
            .min_by(|a, b| a.3.total_cmp(&b.3))
            .map(|(k, base, stand, _)| (k, base, stand))
            .expect("a tree near the spawn")
    }

    /// `step-videos` 5.1's verify: a short showcase's chop and craft go
    /// through the player's input path - the bot's held button on the
    /// trunk through `dig_and_place`, its I, drop-down and CRAFT through
    /// the pack's own systems - and land in the save.
    #[test]
    fn a_short_showcases_chop_and_craft_land_in_the_save() {
        let anchor = fields();
        let (fine, contact) = pbd_app::planet::test_ground(anchor);
        let (tree, base, stand) = tree_and_stand(&fine, anchor);
        let up = stand.normalize();
        let file = file::parse(
            r#"(
                goal: "Goal: a log, and planks from it",
                shots: [
                    (caption: "Chop", max_s: 40.0, acts: [Chop(Have("log", 1))]),
                    (caption: "Craft", max_s: 40.0, acts: [Craft("planks", 1)]),
                ],
            )"#,
        )
        .unwrap();
        let (recipes, _) = pbd_app::config::recipe_book();
        let hand: Vec<String> = recipes.0.at("hand").map(|r| r.code.clone()).collect();
        let mut save = WorldSave::memory_only();
        let hotbar = super::super::slots::Hotbar::restore(&mut save);
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TransformPlugin))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(
                1.0 / 60.0,
            )))
            .insert_resource(fine)
            .insert_resource(contact)
            .insert_resource(save)
            .insert_resource(hotbar)
            .insert_resource(recipes)
            .insert_resource(Player::new("test".into(), file, None))
            .insert_resource(pbd_app::walking::WalkingReadout {
                active: true,
                captured: true,
                ..default()
            })
            .insert_resource(pbd_app::sky::Sun::default())
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<ButtonInput<MouseButton>>()
            .init_resource::<AccumulatedMouseMotion>()
            .init_resource::<Tally>()
            .init_resource::<PendingClick>()
            .init_resource::<Aim>()
            .init_resource::<digging::Mining>()
            .init_resource::<Screen>()
            .init_resource::<pbd_app::planet::NearField>()
            .init_resource::<pbd_app::fish::ToolSlot>()
            .init_resource::<pbd_app::drops::Drops>()
            .init_resource::<pbd_app::config::DigConfig>()
            .init_resource::<pbd_app::config::FarmConfig>()
            .init_resource::<pbd_app::config::GatherConfig>()
            .init_resource::<pbd_app::config::HusbandryConfig>()
            .init_resource::<pbd_app::config::FaunaConfig>()
            .init_resource::<pbd_app::missions::Missions>()
            .init_resource::<pbd_app::missions::MissionChain>()
            .init_resource::<super::super::equipment::Picker>()
            .init_resource::<super::super::farm::FarmNote>()
            .init_resource::<super::super::farm::StationAt>()
            .init_resource::<super::super::craft::CraftPick>()
            .init_resource::<pbd_app::missions::RecipeLocks>()
            .add_message::<MouseWheel>()
            .add_message::<pbd_app::missions::Did>()
            .add_message::<pbd_app::hotbar::Gained>()
            .add_message::<pbd_app::hotbar::Planted>()
            .add_message::<pbd_app::husbandry::Deed>()
            .add_message::<pbd_app::kitchen::Cooked>()
            .add_message::<pbd_app::kitchen::Ate>()
            .add_systems(First, clear_edges)
            .add_systems(
                PreUpdate,
                (tally, drive, press_controls, super::super::pack::open).chain(),
            )
            .add_systems(
                Update,
                (
                    look,
                    pbd_app::sky::run_clock,
                    digging::dig_and_place,
                    super::super::craft::press,
                    super::super::craft::collect_hands,
                )
                    .chain(),
            );
        let eye = stand + up * pbd_app::walking::EYE_HEIGHT;
        let toward = tangent(base - stand, up).normalize();
        app.world_mut()
            .spawn((pbd_app::walking::Walker, Position(stand)));
        app.world_mut().spawn((
            pbd_app::walking::WalkingCamera,
            Camera3d::default(),
            Camera::default(),
            Transform::from_translation(eye).looking_to(toward, up),
        ));
        // The pack's Craft panel: its recipe rows and CRAFT.
        for k in 0..hand.len() {
            app.world_mut()
                .spawn((Interaction::None, Button, super::super::craft::RecipeRow(k)));
        }
        app.world_mut().spawn((
            Interaction::None,
            super::super::craft::CraftButton("hand"),
            pbd_app::ui_kit::Plaque {
                set: pbd_app::ui_kit::Set::Stone,
                focus: false,
                disabled: false,
                latched: false,
            },
        ));
        for _ in 0..60 * 90 {
            app.update();
            let bot = &app.world().resource::<Player>().bot;
            if bot.ended.is_some() || bot.failed {
                break;
            }
        }
        let bot = &app.world().resource::<Player>().bot;
        assert!(!bot.failed, "the showcase stalled");
        assert!(bot.ended.is_some(), "the showcase never ended");
        let save = app.world().resource::<WorldSave>();
        // The chop: the tree's record, written with the log in hand.
        let taken: pbd_core::gather::TreeTaken = save
            .records
            .get(pbd_core::gather::TREE_KIND, u64::from(tree))
            .and_then(|r| r.read())
            .expect("the tree's record is in the save");
        assert_eq!(taken.logs, 1);
        // The craft: the hands' queue in the save, its job collected, and
        // the planks carried.
        assert!(
            save.records
                .get(
                    pbd_core::station::STATION_KIND,
                    pbd_core::station::HAND_RECORD_ID
                )
                .is_some(),
            "the hands' queue is in the save"
        );
        let slots = &app.world().resource::<super::super::slots::Hotbar>().0;
        assert_eq!(carried(slots, "planks"), 4);
        assert_eq!(carried(slots, "log"), 0);
    }
}
