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
//!   the wheel, a panel opened with its key (I, E, O) and shut with it;
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

pub mod file;

use super::digging::{self, Aim};
use super::menu::Screen;
use avian3d::prelude::Position;
use bevy::input::mouse::{AccumulatedMouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use file::{Act, Showcase, Until};
use pbd_app::planet::{PLANET_RADIUS, PlanetContact, PlanetFine, surface_height};
use pbd_app::saves::WorldSave;
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
}

/// The caption bar's text (`step-videos` 2.2).
#[derive(Component)]
pub struct CaptionText;

/// A panel control to press this frame, after the UI's focus pass.
#[derive(Clone, Debug, PartialEq)]
enum Click {
    DropDown,
    DropOption(usize),
    CraftHand,
    Well(usize),
    CraftBench,
    Cook(String),
    Smelt(String),
    Cell(usize),
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
}

/// What an act says after a frame.
enum Status {
    Running,
    Done,
    Fail(String),
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
    sel_from: usize,
    walk_anchor: Option<(Vec3, f32)>,
    detour_s: f32,
    roam_to: Option<Vec3>,
    climb_s: f32,
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
    rmb_down: bool,
    tap_lmb_down: bool,
    still: u32,
    line_was_out: bool,
    bite_s: f32,
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
            return Walked::There;
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

    /// Put an item in the selected slot: its number key, or the pack first
    /// when it is not in the hotbar. `Done` when it is in hand.
    fn select(&mut self, s: &Sight, out: &mut Intent, pick: &dyn Fn(Item) -> bool) -> Status {
        let (phase, since) = &mut self.sel;
        *since += s.dt;
        let in_hotbar = (0..SLOTS).find(|&k| s.slots.get(k).is_some_and(|st| pick(st.item)));
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
                let to = junk
                    .iter()
                    .find_map(|j| {
                        (0..SLOTS).find(|&k| s.slots.get(k).is_some_and(|st| st.item == *j))
                    })
                    .or_else(|| (0..SLOTS).find(|&k| s.slots.get(k).is_none()))
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
            if !matches!(material, 2 | 3 | 7)
                || r.direction_height[3] < 0.6
                || pbd_core::flora::grows_tree(r.spare[0], r.metadata[1])
                || self.used.contains(&key)
                || !s.save.edits.for_cell(key).is_empty()
            {
                continue;
            }
            let top = d * (PLANET_RADIUS + r.direction_height[3]);
            let dist = tangent(top - s.feet, s.up).length();
            let rise = (top - s.feet).dot(s.up);
            if dist < near
                || dist > far
                || rise.abs() > 1.6
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
                .fresh_cells(s, 5.0, 30.0)
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
pub struct Hands<'w> {
    keys: ResMut<'w, ButtonInput<KeyCode>>,
    mouse: ResMut<'w, ButtonInput<MouseButton>>,
    motion: ResMut<'w, AccumulatedMouseMotion>,
    wheel: MessageWriter<'w, MouseWheel>,
}

/// The world side of the bot's frame.
#[derive(bevy::ecs::system::SystemParam)]
pub struct World<'w, 's> {
    time: Res<'w, Time>,
    /// Absent in a test's app, which has a camera and no walker plugin.
    state: Option<ResMut<'w, pbd_app::walking::WalkingState>>,
    walkers: Query<'w, 's, &'static Position, With<pbd_app::walking::Walker>>,
    cameras: Query<'w, 's, &'static GlobalTransform, With<pbd_app::walking::WalkingCamera>>,
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
) {
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
        bot.home = Some(p.0);
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
                t.text = text.to_string();
            }
        }
    };
    // The goal, said first, for a few seconds.
    if now - started < 4.0 {
        set_caption(&mut captions, &player.file.goal);
        return;
    }
    // Where the player looks, off the camera the walker's mouse-look turns.
    let feet = position.0;
    let up = feet.normalize_or(Vec3::Y);
    let look = camera.forward().as_vec3();
    let heading = tangent(look, up).normalize_or(up.any_orthonormal_vector());
    let pitch = look.dot(up).clamp(-1.0, 1.0).asin();
    let sight = Sight {
        now,
        dt,
        feet,
        up: feet.normalize_or(Vec3::Y),
        eye: camera.translation(),
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
    };
    let Some(shot) = player.file.shots.get(bot.shot).cloned() else {
        bot.ended = Some(now);
        let done: Vec<&String> = sight.missions.progress.done.iter().collect();
        info!("SHOWCASE_DONE missions in the save: {done:?}");
        info!("SHOWCASE_END {}", player.name);
        return;
    };
    if bot.shot_s == 0.0 && bot.act == 0 && bot.act_s == 0.0 {
        info!("SHOT {} {}", bot.shot + 1, shot.caption);
        set_caption(&mut captions, &shot.caption);
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
            "SHOWCASE_STATUS shot {} act {} phase {} ({:.1} s) goal {:?} screen {:?} aim {:?} in hand {:?}",
            bot.shot + 1,
            bot.act + 1,
            bot.phase,
            bot.phase_s,
            bot.goal,
            sight.screen,
            sight.aim.map(|t| (t.dig.cell, t.dig.layer)),
            sight.tools.held(),
        );
    }
    bot.shot_s += dt;
    bot.act_s += dt;
    bot.phase_s += dt;
    let mut out = Intent::default();
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
    let status = if bot.shot_s > shot.max_s {
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
        self.look_goal = None;
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
        Option<&super::craft::DropDown>,
        Option<&super::craft::DropOption>,
        Option<&super::craft::CraftButton>,
        Option<&super::craft::Well>,
        Option<&super::kitchen::CookPlaque>,
        Option<&super::pack::PackCell>,
        Option<&super::forge::RecipePlaque>,
    )>,
) {
    let Some(click) = pending.0.take() else {
        return;
    };
    let mut hit = false;
    for (mut interaction, drop, option, button, well, cook, cell, smelt) in &mut controls {
        let this = match &click {
            Click::DropDown => drop.is_some(),
            Click::DropOption(k) => option.is_some_and(|o| o.0 == *k),
            Click::CraftHand => button.is_some_and(|b| b.0 == "hand"),
            Click::CraftBench => button.is_some_and(|b| b.0 == "workbench"),
            Click::Well(k) => well.is_some_and(|w| {
                w.station == "workbench" && w.role == super::craft::Role::Pick(*k)
            }),
            Click::Cook(code) => cook.is_some_and(|c| &c.0 == code),
            Click::Smelt(code) => smelt.is_some_and(|c| &c.0 == code),
            Click::Cell(k) => cell.is_some_and(|c| c.0 == *k),
        };
        if this {
            *interaction = Interaction::Pressed;
            hit = true;
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
    match act {
        Act::Wait(secs) => {
            if bot.act_s >= *secs {
                Status::Done
            } else {
                Status::Running
            }
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
        Act::Dig(until, depth) => dig(bot, until, *depth, s, out),
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
        Act::Shore => match bot.phase {
            0 => match shore(s) {
                Some((water, stand)) => {
                    bot.goal = Some(Goal::Point(water, stand));
                    bot.next_phase();
                    Status::Running
                }
                None => Status::Fail("no sea within 900 m".into()),
            },
            1 => {
                let Some(Goal::Point(_, stand)) = bot.goal else {
                    return Status::Fail("lost the shore".into());
                };
                if bot.walk_to(s, out, stand, 1.2) == Walked::There {
                    bot.next_phase();
                }
                Status::Running
            }
            _ => {
                let Some(Goal::Point(water, _)) = bot.goal else {
                    return Status::Fail("lost the shore".into());
                };
                let toward = tangent(water - s.feet, s.up);
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
            if bot.act_s > 8.0 {
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
                || !pbd_core::flora::grows_tree(r.spare[0], r.metadata[1])
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
            Some(Tool::Pickaxe) => matches!(m, Material::Stone | Material::Rock | Material::Ore),
            _ => matches!(
                m,
                Material::Grass
                    | Material::DryGrass
                    | Material::JungleGrass
                    | Material::Dirt
                    | Material::Soil
                    | Material::Sand
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

/// The pack's Craft row: open the pack, pick the recipe in the drop-down,
/// press CRAFT once per make and wait for the hands to finish, shut it.
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
            out.click = Some(Click::DropDown);
            bot.next_phase();
        }
        2 if bot.phase_s > 0.5 => {
            out.click = Some(Click::DropOption(k));
            bot.next_phase();
        }
        3 if bot.phase_s > 0.5 => {
            if bot.count >= times {
                bot.phase = 5;
                bot.phase_s = 0.0;
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
    if bot.phase == 0 {
        let name = name.to_string();
        match bot.select(s, out, &|item| wanted(&name, item)) {
            Status::Done => bot.next_phase(),
            Status::Fail(why) => return Status::Fail(format!("{name}: {why}")),
            Status::Running => {}
        }
        return Status::Running;
    }
    if s.tally.placed > bot.mark.placed {
        return Status::Done;
    }
    if bot.act_s > 45.0 {
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

/// A station's panel: walk to the nearest placed one, E, press the recipe
/// once per make and wait for it, E to shut.
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
        return Status::Fail(format!("the {} makes no {code}", row.name));
    };
    match bot.phase {
        0 => {
            // A station up a wall too high to step (the bot is in a hole
            // beside it): build up to it.
            let rise = (at - s.feet).dot(s.up);
            if (bot.phase_s % 5.0) < s.dt {
                info!(
                    "showcase: walking to the {}: {:.1} m off, {:.1} m up",
                    row.name,
                    tangent(at - s.feet, s.up).length(),
                    rise
                );
            }
            if rise > 0.9 && tangent(at - s.feet, s.up).length() < 3.5 {
                bot.climb(s, out);
            } else if bot.walk_to(s, out, at, 2.0) == Walked::There {
                info!(
                    "showcase: at the {} in cell {cell} layer {layer}, {:.1} m from the eye, {:.1} m over the feet",
                    row.name,
                    at.distance(s.eye),
                    (at - s.feet).dot(s.up)
                );
                bot.next_phase();
            }
        }
        1 => {
            // Its top half: from below, the middle sits on the block under it.
            // A campfire is a light the eye ray passes: E goes on the
            // ground it stands on.
            let (point, want) = if material.is_lamp() {
                (at - s.up * 0.6, (cell, layer - 1))
            } else {
                (at + s.up * 0.3, (cell, layer))
            };
            bot.aim_at(s, out, point);
            let on = bot.aimed_cell(s) == Some(want);
            if on && bot.phase_s > 0.3 {
                out.taps.push(KeyCode::KeyE);
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
                out.taps.push(KeyCode::KeyE);
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
    if bot.act_s > 40.0 + times as f32 * 40.0 {
        return Status::Fail(format!("{code} at the {} never finished", row.name));
    }
    Status::Running
}

/// Dry ground a few metres off, never the sea: of sixteen headings at ten
/// metres, the one with the highest ground that is not a climb, turned
/// from the heading where it can be; the way inland when the bot is in the
/// water.
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
fn shore(s: &Sight) -> Option<(Vec3, Vec3)> {
    let up = s.up;
    let east = Vec3::Y.cross(up).normalize_or(Vec3::X);
    let north = up.cross(east);
    for r in (4..900).step_by(3) {
        let r = r as f32;
        let mut best: Option<(Vec3, f32)> = None;
        for a in 0..48 {
            let a = a as f32 / 48.0 * std::f32::consts::TAU;
            let along = east * a.cos() + north * a.sin();
            let d = (up * PLANET_RADIUS + along * r).normalize();
            if surface_height(d) < -0.6 {
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

/// Fish: to the shore, face the water, cast, watch the float, hook on the
/// bite, reel with the tension, again until.
fn fish(bot: &mut Bot, until: &Until, s: &Sight, out: &mut Intent) -> Status {
    use pbd_core::fishing::Phase;
    if s.tools.held() != Some(Tool::Rod) {
        return Status::Fail("the rod is not in hand".into());
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
            let Some((water, stand)) = shore(s) else {
                return Status::Fail("no sea within 900 m".into());
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
                Walked::There => bot.next_phase(),
                _ if bot.phase_s > 60.0 => return Status::Fail("never reached the shore".into()),
                _ => {}
            }
        }
        2 => {
            // Face the water, a little down, and cast.
            let Some(Goal::Point(water, _)) = bot.goal else {
                bot.phase = 0;
                return Status::Running;
            };
            let toward = tangent(water - s.feet, s.up);
            if bot.steer(s, out, toward, -0.12) && bot.phase_s > 0.5 {
                bot.next_phase();
            }
        }
        3 => {
            // Charge: the button held, then let go.
            // A long charge: the float lands well out, in deeper water.
            out.lmb = bot.phase_s < 1.4;
            if bot.phase_s > 1.6 {
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
            if bot.tries > 40 {
                return Status::Fail("forty casts and no catch".into());
            }
        }
    }
    Status::Running
}

/// Till a small field with the shovel: a fresh cell ahead and its grassy
/// neighbours, E on each.
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
        out.taps.push(KeyCode::KeyE);
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
    let Some(&key) = bot.field.get(bot.count as usize + bot.tries as usize / 3) else {
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
    if off < 1.4 {
        // Standing on it: a step back to see it.
        out.held.insert(KeyCode::KeyS);
    }
    bot.aim_at(s, out, top - s.up * 0.2);
    let on = bot.aimed_cell(s).is_some_and(|(c, _)| c == key);
    if on && bot.phase_s > 0.5 {
        out.rmb_tap = true;
        bot.tries += 1;
        bot.phase_s = 0.0;
    } else if bot.phase_s > 4.0 {
        // Out of sight from here: the next cell.
        bot.tries += 3;
        bot.phase_s = 0.0;
    }
    if bot.act_s > 60.0 {
        return Status::Fail(format!("planted {} of {n}", bot.count));
    }
    Status::Running
}

/// Pick wild plants: walk to each, E.
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
            if super::farm::wild_at(s.fine, s.save, s.farm, r.key(), layer).is_none() {
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
        return Status::Running;
    };
    let Some(top) = bot.top_of(s, key) else {
        bot.gave_up.insert(key);
        bot.goal = None;
        return Status::Running;
    };
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
                out.taps.push(KeyCode::KeyE);
                bot.phase_s = -1.0;
            }
            if bot.phase_s > 3.0 {
                bot.gave_up.insert(key);
                bot.goal = None;
            }
        }
    }
    Status::Running
}

/// Harvest the field: E on each of its cells.
fn harvest(bot: &mut Bot, n: u32, s: &Sight, out: &mut Intent) -> Status {
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
        out.taps.push(KeyCode::KeyE);
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
        .flat_map(|y| y.seen())
        .filter_map(move |a| {
            let sp = s.land.get(a.species as usize)?;
            (species == "any" || sp.id == species).then_some((a, sp))
        })
}

/// Tame: find a wild animal the carried food suits, food in hand, come near
/// and stand still, and feed it when it comes, until it trusts.
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
        .min_by(|a, b| {
            (!a.0.female as u8).cmp(&(!b.0.female as u8)).then(
                a.0.pos
                    .distance(s.feet)
                    .total_cmp(&b.0.pos.distance(s.feet)),
            )
        });
    let Some((animal, sp)) = target else {
        // None in sight: look further.
        out.held.insert(KeyCode::KeyW);
        bot.steer(s, out, s.heading, -0.05);
        if bot.act_s > 90.0 {
            return Status::Fail(format!("no wild {species} found"));
        }
        return Status::Running;
    };
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
            let far = dist > 12.0;
            let stop = if far || bot.tries == 0 {
                10.0
            } else {
                (dist - 2.5).max(2.5)
            };
            if bot.walk_to(s, out, animal.pos, stop) == Walked::There || (!far && bot.phase_s > 8.0)
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
                    sp.name,
                    s.slots.held().map(|st| st.item),
                    animals(s, species).filter(|(a, _)| a.wild).count()
                );
            }
            if dist < 2.3 && bot.phase_s > 1.1 && s.screen == Screen::Playing {
                out.taps.push(KeyCode::KeyE);
                bot.phase_s = 0.0;
            }
            if dist > 3.0 && bot.phase_s > 10.0 {
                bot.phase = 1;
                bot.phase_s = 0.0;
                bot.tries += 1;
            }
        }
    }
    if bot.act_s > 150.0 {
        return Status::Fail(format!("the {} never trusted", sp.name));
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

/// Lead a kept animal into the pen: E to have it follow, walk in, E to let
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
                out.taps.push(KeyCode::KeyE);
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
                out.taps.push(KeyCode::KeyE);
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

/// E on a kept animal until it gives what it has.
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
                out.taps.push(KeyCode::KeyE);
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
        pbd_app::ui_kit::KitText::new("", pbd_app::ui_kit::LABEL).outline(pbd_app::ui_kit::INK),
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
            .filter(|r| pbd_core::flora::grows_tree(r.spare[0], r.metadata[1]))
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
        // The pack's Craft row: the drop-down, its options and CRAFT.
        app.world_mut()
            .spawn((Interaction::None, super::super::craft::DropDown));
        for k in 0..hand.len() {
            app.world_mut().spawn((
                Interaction::None,
                Button,
                super::super::craft::DropOption(k),
            ));
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
