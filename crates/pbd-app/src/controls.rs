//! The controls: what every key does, and whether the world is listening.
//!
//! Two facts about one subject, which is the contract between the player's
//! hands and the world.
//!
//! [`BINDINGS`] is the ONE place the bindings are written down. They used to
//! be written three times - a panel behind `H`, the `--help` text, and a
//! one-line strip over the hotbar - and they had gone stale in the way three
//! copies always do: none of them named the dig or the place, months after
//! both shipped. The settings page and `--help` read this table, and
//! [`tests::a_named_key_is_a_key_something_reads`] reads the source of the
//! systems that press the keyboard and asserts the table has not drifted from
//! them.
//!
//! [`MenuOpen`] is the other half. A menu is a second claimant on the pointer,
//! and everything awkward about having one is that claim: the world's readers
//! take the pointer on a click, and a click on RESUME is still a click. One
//! resource answers who has it, and [`hand_over_pointer`] is the rule every
//! reader applies to its own capture flag.

use bevy::{ecs::system::SystemParam, prelude::*};
use pbd_core::vehicle::Kind;
use std::collections::BTreeMap;

/// Whether a menu holds the pointer and the keyboard.
///
/// Set by the desktop menu and read by every system that takes player input.
/// It is initialised by [`crate::PaleBlueDotPlugin`] so a consumer that builds
/// no menu still has the resource and simply never sees it set.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct MenuOpen(pub bool);

/// What an input reader does with its own capture flag across a menu.
///
/// The interesting half is the EDGE. Holding is obvious - while a menu is open
/// nothing may take the pointer, or the click that presses a button would grab
/// the mouse on its way through. RELEASING has to hand the pointer back, or
/// resuming leaves the player looking at a free cursor needing one click to
/// play, and that click lands in the world.
///
/// `was_open` is the caller's own memory of the last answer, because a Bevy
/// system has nowhere else to keep one. The rule itself is written once, here,
/// rather than once in the walker and once in the pilot.
pub fn hand_over_pointer(open: bool, was_open: &mut bool, captured: &mut bool) {
    if open {
        *captured = false;
    } else if *was_open {
        *captured = true;
    }
    *was_open = open;
}

/// Whether a menu holds the pointer, and this reader's own memory of the last
/// answer.
///
/// The two always travel together - the rule needs the previous frame to see
/// the EDGE - and a pair of parameters that are never apart is a struct that
/// was missing. One `SystemParam` also means the walker and the pilot ask the
/// question with the same call rather than with the same three lines.
#[derive(SystemParam)]
pub struct Pointer<'w, 's> {
    menu: Option<Res<'w, MenuOpen>>,
    was_open: Local<'s, bool>,
}

impl Pointer<'_, '_> {
    /// Whether a menu holds the pointer this frame, having first moved
    /// `captured` across the edge: cleared while one is open, and set on the
    /// frame the last one closes.
    pub fn menu_holds(&mut self, captured: &mut bool) -> bool {
        let open = self.menu.as_ref().is_some_and(|open| open.0);
        hand_over_pointer(open, &mut self.was_open, captured);
        open
    }
}

/// `G`, the tool picker's key, read once: held on foot it opens the picker
/// beside the tool slot, and the release that ends the hold equips the
/// highlighted tool.
///
/// It no longer shares a key with boarding (that is `F`), so there is no tap
/// to tell from a hold and the picker opens on the press. One reader still,
/// because the picker's systems and the wheel's both ask whether it is open.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct PickerKey {
    /// Held on foot: the picker is open.
    pub holding: bool,
    /// Released this frame after a hold: the picker commits its choice.
    pub let_go: bool,
}

/// Read `G` into [`PickerKey`]. Only on foot or paddling a small boat:
/// aboard anything else or flying, G does nothing, and a menu forgets the key so Escape closes the picker without
/// equipping anything.
pub fn read_picker_key(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    menu: Option<Res<MenuOpen>>,
    walking: Option<Res<crate::walking::WalkingReadout>>,
    aboard: Option<Res<crate::vehicles::Aboard>>,
    vehicles: Query<&crate::vehicles::Vehicle>,
    mut key: ResMut<PickerKey>,
) {
    key.let_go = false;
    let Some(keys) = keys else {
        return;
    };
    if menu.is_some_and(|m| m.0) {
        *key = PickerKey::default();
        return;
    }
    let seat = aboard.and_then(|a| a.0);
    // A paddled boat's paddler takes up tools too: the rod, to fish
    // (`small-boats` design 6).
    let paddling = seat
        .and_then(|e| vehicles.get(e).ok())
        .is_some_and(|v| v.craft.kind.paddled());
    let on_foot = (walking.is_some_and(|w| w.active) && seat.is_none()) || paddling;
    if keys.just_pressed(KeyCode::KeyG) && on_foot {
        key.holding = true;
    }
    if key.holding && (!keys.pressed(KeyCode::KeyG) || !on_foot) {
        key.holding = false;
        key.let_go = on_foot;
    }
}

/// Whether the developer's overlay is up: F3, the frame graph and the boat
/// instruments with it.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct DebugOverlay(pub bool);

/// A control, as the game reads it.
///
/// The variants carry Bevy's own types rather than a printed name, for the one
/// reason that makes the drift test possible: a `KeyCode` has a spelling a
/// test can look for in the source of the system that presses it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Board(KeyCode),
    Mouse(MouseButton),
    Wheel,
}

/// The keys a slot can be given, and what a cap prints for each: every label
/// is in the kit's font (`ui_kit::tests::the_kit_font_has_every_label_glyph`
/// reads this table). A key outside it is not taken by a rebind, and a cap
/// waits for another.
pub const ASSIGNABLE: [(KeyCode, &str); 87] = [
    (KeyCode::KeyA, "A"),
    (KeyCode::KeyB, "B"),
    (KeyCode::KeyC, "C"),
    (KeyCode::KeyD, "D"),
    (KeyCode::KeyE, "E"),
    (KeyCode::KeyF, "F"),
    (KeyCode::KeyG, "G"),
    (KeyCode::KeyH, "H"),
    (KeyCode::KeyI, "I"),
    (KeyCode::KeyJ, "J"),
    (KeyCode::KeyK, "K"),
    (KeyCode::KeyL, "L"),
    (KeyCode::KeyM, "M"),
    (KeyCode::KeyN, "N"),
    (KeyCode::KeyO, "O"),
    (KeyCode::KeyP, "P"),
    (KeyCode::KeyQ, "Q"),
    (KeyCode::KeyR, "R"),
    (KeyCode::KeyS, "S"),
    (KeyCode::KeyT, "T"),
    (KeyCode::KeyU, "U"),
    (KeyCode::KeyV, "V"),
    (KeyCode::KeyW, "W"),
    (KeyCode::KeyX, "X"),
    (KeyCode::KeyY, "Y"),
    (KeyCode::KeyZ, "Z"),
    (KeyCode::Digit0, "0"),
    (KeyCode::Digit1, "1"),
    (KeyCode::Digit2, "2"),
    (KeyCode::Digit3, "3"),
    (KeyCode::Digit4, "4"),
    (KeyCode::Digit5, "5"),
    (KeyCode::Digit6, "6"),
    (KeyCode::Digit7, "7"),
    (KeyCode::Digit8, "8"),
    (KeyCode::Digit9, "9"),
    (KeyCode::F1, "F1"),
    (KeyCode::F2, "F2"),
    (KeyCode::F3, "F3"),
    (KeyCode::F4, "F4"),
    (KeyCode::F5, "F5"),
    (KeyCode::F6, "F6"),
    (KeyCode::F7, "F7"),
    (KeyCode::F8, "F8"),
    (KeyCode::F9, "F9"),
    (KeyCode::F10, "F10"),
    (KeyCode::F11, "F11"),
    (KeyCode::F12, "F12"),
    (KeyCode::Space, "SPACE"),
    (KeyCode::Tab, "TAB"),
    (KeyCode::Enter, "ENTER"),
    (KeyCode::Backspace, "BACK"),
    (KeyCode::CapsLock, "CAPS"),
    (KeyCode::ShiftLeft, "SHIFT"),
    (KeyCode::ShiftRight, "R SHIFT"),
    (KeyCode::ControlLeft, "CTRL"),
    (KeyCode::ControlRight, "R CTRL"),
    (KeyCode::AltLeft, "ALT"),
    (KeyCode::AltRight, "R ALT"),
    (KeyCode::ArrowUp, "↑"),
    (KeyCode::ArrowDown, "↓"),
    (KeyCode::ArrowLeft, "←"),
    (KeyCode::ArrowRight, "→"),
    (KeyCode::Home, "HOME"),
    (KeyCode::End, "END"),
    (KeyCode::Insert, "INS"),
    (KeyCode::Delete, "DEL"),
    (KeyCode::PageUp, "PG UP"),
    (KeyCode::PageDown, "PG DN"),
    (KeyCode::Minus, "-"),
    (KeyCode::Equal, "="),
    (KeyCode::Semicolon, ";"),
    (KeyCode::Quote, "'"),
    (KeyCode::Comma, ","),
    (KeyCode::Period, "."),
    (KeyCode::Slash, "/"),
    (KeyCode::Numpad0, "NUM 0"),
    (KeyCode::Numpad1, "NUM 1"),
    (KeyCode::Numpad2, "NUM 2"),
    (KeyCode::Numpad3, "NUM 3"),
    (KeyCode::Numpad4, "NUM 4"),
    (KeyCode::Numpad5, "NUM 5"),
    (KeyCode::Numpad6, "NUM 6"),
    (KeyCode::Numpad7, "NUM 7"),
    (KeyCode::Numpad8, "NUM 8"),
    (KeyCode::Numpad9, "NUM 9"),
    (KeyCode::NumpadEnter, "NUM ENTER"),
];

/// What a cap prints for a keyboard key: its `ASSIGNABLE` label, `ESC` for
/// the menu key, and nothing for a key the table does not know.
pub fn key_label(code: KeyCode) -> &'static str {
    if code == KeyCode::Escape {
        return "ESC";
    }
    ASSIGNABLE
        .iter()
        .find(|(k, _)| *k == code)
        .map_or("", |(_, label)| *label)
}

/// How a key is written in the settings file: Bevy's own name (`KeyQ`).
pub fn key_name(code: KeyCode) -> String {
    format!("{code:?}")
}

/// The key a name in the settings file names, if it is one a slot can take.
pub fn key_named(name: &str) -> Option<KeyCode> {
    ASSIGNABLE
        .iter()
        .map(|(k, _)| *k)
        .find(|k| key_name(*k) == name)
}

impl Key {
    /// What the settings page and `--help` print. A keyboard key is printed
    /// as its cap (`key_label`): the default shift and ctrl are the left
    /// ones, and the readers take either side, so they print plain.
    pub fn label(self) -> &'static str {
        match self {
            Key::Board(code) => key_label(code),
            Key::Mouse(MouseButton::Left) => "LEFT CLICK",
            Key::Mouse(MouseButton::Right) => "RIGHT CLICK",
            Key::Mouse(_) => "",
            Key::Wheel => "WHEEL",
        }
    }

    /// How this control is spelled in the source that reads it. Only the
    /// drift test asks, which is the only thing that should: the spelling of a
    /// `KeyCode` is an implementation detail everywhere else.
    #[cfg(test)]
    fn token(self) -> String {
        match self {
            Key::Board(code) => format!("KeyCode::{code:?}"),
            Key::Mouse(button) => format!("MouseButton::{button:?}"),
            Key::Wheel => "MouseWheel".into(),
        }
    }
}

/// One control and what it does.
#[derive(Debug)]
pub struct Binding {
    /// The keys, in the order they are printed. Several means either: `W A S D`
    /// is four keys for one action, and `SPACE / CTRL` is a pair.
    pub keys: &'static [Key],
    /// The separator between them when printed.
    pub joiner: &'static str,
    pub does: &'static str,
    /// A range (the number row's `1 - 0`): its caps are its ends, and it is
    /// read as every key between, so no key of it can be rebound alone.
    pub range: bool,
    /// A chord (`SHIFT + ALT + F`): every key held together. Listed, never
    /// rebindable, and read by its own reader off the physical keyboard, so
    /// it clashes with nothing that reads one of its keys alone.
    pub chord: bool,
}

/// The sub-tab of the settings' Input page a group is listed on (decision 6:
/// "organize the input menu better with tabs or something instead of big uber
/// list").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tab {
    OnFoot,
    Hands,
    Flying,
    Vehicles,
    Menus,
}

impl Tab {
    pub const ALL: [Tab; 5] = [
        Tab::OnFoot,
        Tab::Hands,
        Tab::Flying,
        Tab::Vehicles,
        Tab::Menus,
    ];

    /// The sub-tab's label, in the kit's font.
    pub fn label(self) -> &'static str {
        match self {
            Tab::OnFoot => "ON FOOT",
            Tab::Hands => "HANDS",
            Tab::Flying => "FLYING",
            Tab::Vehicles => "VEHICLES",
            Tab::Menus => "MENUS",
        }
    }
}

/// Where the player is, as far as which keys mean what: the remap's context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Live {
    OnFoot,
    Flying,
    Aboard(Kind),
}

impl Live {
    /// Every context, for asking whether two groups can be live together.
    pub const ALL: [Live; 8] = [
        Live::OnFoot,
        Live::Flying,
        Live::Aboard(Kind::Tern),
        Live::Aboard(Kind::Loon),
        Live::Aboard(Kind::Rowboat),
        Live::Aboard(Kind::Cog),
        Live::Aboard(Kind::Tube),
        Live::Aboard(Kind::Raft),
    ];
}

/// When a group's keys are read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum When {
    Always,
    OnFoot,
    Flying,
    /// On foot (F interacts and boards) and aboard anything (F steps off,
    /// V, T).
    OnFootOrAboard,
    Aboard(Kind),
    /// Aboard a paddled boat, whose paddler can fish (`small-boats`).
    Paddled,
}

impl When {
    pub fn live(self, now: Live) -> bool {
        match (self, now) {
            (When::Always, _) => true,
            (When::OnFoot, Live::OnFoot) | (When::Flying, Live::Flying) => true,
            (When::OnFootOrAboard, Live::OnFoot | Live::Aboard(_)) => true,
            (When::Aboard(kind), Live::Aboard(now)) => kind == now,
            (When::Paddled, Live::Aboard(now)) => now.paddled(),
            _ => false,
        }
    }

    /// Whether two groups' keys can be read at the same moment.
    pub fn together(self, other: When) -> bool {
        Live::ALL
            .iter()
            .any(|&now| self.live(now) && other.live(now))
    }
}

/// A heading and the controls under it.
#[derive(Debug)]
pub struct Group {
    pub heading: &'static str,
    /// The Input sub-tab it is listed on.
    pub tab: Tab,
    /// When its keys are read, which is when a rebind of them applies.
    pub when: When,
    pub rows: &'static [Binding],
}

const fn row(keys: &'static [Key], joiner: &'static str, does: &'static str) -> Binding {
    Binding {
        keys,
        joiner,
        does,
        range: false,
        chord: false,
    }
}

const WASD: [Key; 4] = [
    Key::Board(KeyCode::KeyW),
    Key::Board(KeyCode::KeyA),
    Key::Board(KeyCode::KeyS),
    Key::Board(KeyCode::KeyD),
];

const ON_FOOT: [Binding; 4] = [
    row(&WASD, " ", "move"),
    row(&[Key::Board(KeyCode::KeyB)], " ", "breeding"),
    row(
        &[Key::Board(KeyCode::Space)],
        " ",
        "jump, and rise in water",
    ),
    row(&[Key::Board(KeyCode::ShiftLeft)], " ", "sprint"),
];

/// F: the one interact key (owner, 2026-11-16: "we should use F for
/// interact since E will be used for roll in space"). What one press does is
/// the highest row of `interact::ACTS` the eye is on; with nothing there, it
/// boards a craft in reach, and aboard it steps off.
const INTERACT: [Binding; 1] = [row(
    &[Key::Board(KeyCode::KeyF)],
    " ",
    "open, harvest, pick, feed, till; board or step off",
)];

/// The interact slot's name in the settings file.
pub const INTERACT_SLOT: &str = "interact:KeyF";

/// The slots the interact slot replaced (settings version 8), the first
/// winning: a player who moved "use" keeps that key, else one who moved
/// "board" keeps that.
pub const BEFORE_INTERACT: [&str; 2] = ["hands:KeyE", "vehicles:KeyF"];

const HANDS: [Binding; 5] = [
    row(
        &[Key::Mouse(MouseButton::Left)],
        " ",
        "use the tool in hand: dig, or cast, hook and reel",
    ),
    row(
        &[Key::Mouse(MouseButton::Right)],
        " ",
        "place the held block, plant a seed, or wind the line in",
    ),
    Binding {
        range: true,
        ..row(
            &[Key::Board(KeyCode::Digit1), Key::Board(KeyCode::Digit0)],
            " - ",
            "select a slot",
        )
    },
    row(&[Key::Wheel], " ", "step through the slots"),
    row(
        &[Key::Board(KeyCode::KeyG)],
        " ",
        "hold for the tools; the wheel picks one",
    ),
];

const FLYING: [Binding; 6] = [
    row(&WASD, " ", "move"),
    row(
        &[Key::Board(KeyCode::Space), Key::Board(KeyCode::ControlLeft)],
        " / ",
        "up and down",
    ),
    row(
        &[Key::Board(KeyCode::KeyQ), Key::Board(KeyCode::KeyE)],
        " / ",
        "roll",
    ),
    row(&[Key::Board(KeyCode::ShiftLeft)], " ", "cruise"),
    row(&[Key::Board(KeyCode::KeyX)], " ", "dampeners"),
    row(&[Key::Board(KeyCode::KeyB)], " ", "brake"),
];

const W_S: [Key; 2] = [Key::Board(KeyCode::KeyW), Key::Board(KeyCode::KeyS)];
const A_D: [Key; 2] = [Key::Board(KeyCode::KeyA), Key::Board(KeyCode::KeyD)];
const Q_E: [Key; 2] = [Key::Board(KeyCode::KeyQ), Key::Board(KeyCode::KeyE)];

const VEHICLES: [Binding; 2] = [
    row(&[Key::Board(KeyCode::KeyV)], " ", "change the camera"),
    row(
        &[Key::Board(KeyCode::KeyT)],
        " ",
        "drop or raise the anchor",
    ),
];

const TERN: [Binding; 4] = [
    row(&A_D, "/", "steer"),
    row(&W_S, "/", "pull the sail in or let it out"),
    row(&Q_E, "/", "lean out left or right"),
    row(&[Key::Board(KeyCode::KeyB)], " ", "bail"),
];

const LOON: [Binding; 4] = [
    row(&W_S, "/", "paddle forward or back"),
    row(&A_D, "/", "turn"),
    row(&Q_E, "/", "steer with the paddle"),
    row(&[Key::Board(KeyCode::KeyB)], " ", "bail"),
];

const ROWBOAT: [Binding; 4] = [
    row(&W_S, "/", "row forward or back"),
    row(&A_D, "/", "turn"),
    row(&Q_E, "/", "steer with an oar"),
    row(&[Key::Board(KeyCode::KeyB)], " ", "bail"),
];

const TUBE: [Binding; 2] = [
    row(&W_S, "/", "kick forward or back"),
    row(&A_D, "/", "turn"),
];

const RAFT: [Binding; 3] = [
    row(&W_S, "/", "pole forward or back"),
    row(&A_D, "/", "turn"),
    row(&Q_E, "/", "steer with the pole"),
];

/// The rod from a paddled boat's seat (`small-boats` design 6).
const AFLOAT: [Binding; 3] = [
    row(
        &[Key::Mouse(MouseButton::Left)],
        " ",
        "with the rod: hold to cast, press to strike, hold to reel",
    ),
    row(&[Key::Mouse(MouseButton::Right)], " ", "wind the line in"),
    row(
        &[Key::Board(KeyCode::KeyG)],
        " ",
        "hold for the tools; the wheel picks one",
    ),
];

const COG: [Binding; 3] = [
    row(&A_D, "/", "steer"),
    row(&W_S, "/", "turn the sail"),
    row(&[Key::Board(KeyCode::KeyB)], " ", "bail"),
];

const WORLD: [Binding; 5] = [
    row(&[Key::Board(KeyCode::KeyM)], " ", "the map"),
    row(&[Key::Board(KeyCode::KeyR)], " ", "walk or fly"),
    row(&[Key::Board(KeyCode::KeyH)], " ", "return to the spawn"),
    row(&[Key::Board(KeyCode::KeyP)], " ", "cycle the weather"),
    row(&[Key::Board(KeyCode::KeyL)], " ", "noon or midnight"),
];

/// The repro key (`repro-capture`; the owner, 2026-11-16: "add a button
/// for capturing location data so I can help repro stuff (shift alt F or
/// something)").
const REPRO_KEYS: [Key; 3] = [
    Key::Board(KeyCode::ShiftLeft),
    Key::Board(KeyCode::AltLeft),
    Key::Board(KeyCode::KeyF),
];

const SCREEN: [Binding; 9] = [
    row(&[Key::Board(KeyCode::Escape)], " ", "menu"),
    row(&[Key::Board(KeyCode::KeyJ)], " ", "the field guide"),
    row(
        &[Key::Board(KeyCode::KeyI), Key::Board(KeyCode::Tab)],
        " / ",
        "the pack: thirty more slots",
    ),
    row(&[Key::Board(KeyCode::KeyO)], " ", "the starter missions"),
    row(&[Key::Board(KeyCode::F12)], " ", "screenshot"),
    Binding {
        chord: true,
        ..row(&REPRO_KEYS, " + ", "copy the location for a bug report")
    },
    row(
        &[Key::Board(KeyCode::F3)],
        " ",
        "frame graph and boat instruments",
    ),
    row(&[Key::Board(KeyCode::F5)], " ", "flow arrows"),
    row(&[Key::Board(KeyCode::F6)], " ", "pour water"),
];

const fn group(heading: &'static str, tab: Tab, when: When, rows: &'static [Binding]) -> Group {
    Group {
        heading,
        tab,
        when,
        rows,
    }
}

/// Every control in the game, once. (The Kestrel's group, KESTREL on the
/// Flying sub-tab, went with it in `retire-kestrel`; a saved rebind of one of
/// its slots is kept in the file and read by nothing.)
pub static BINDINGS: [Group; 14] = [
    group("ON FOOT", Tab::OnFoot, When::OnFoot, &ON_FOOT),
    group("INTERACT", Tab::Hands, When::OnFootOrAboard, &INTERACT),
    group("HANDS", Tab::Hands, When::OnFoot, &HANDS),
    group("FLYING", Tab::Flying, When::Flying, &FLYING),
    group("VEHICLES", Tab::Vehicles, When::OnFootOrAboard, &VEHICLES),
    group("TERN", Tab::Vehicles, When::Aboard(Kind::Tern), &TERN),
    group("LOON", Tab::Vehicles, When::Aboard(Kind::Loon), &LOON),
    group(
        "ROWBOAT",
        Tab::Vehicles,
        When::Aboard(Kind::Rowboat),
        &ROWBOAT,
    ),
    group("COG", Tab::Vehicles, When::Aboard(Kind::Cog), &COG),
    group("INNER TUBE", Tab::Vehicles, When::Aboard(Kind::Tube), &TUBE),
    group("RAFT", Tab::Vehicles, When::Aboard(Kind::Raft), &RAFT),
    group("FISHING AFLOAT", Tab::Vehicles, When::Paddled, &AFLOAT),
    group("WORLD", Tab::Menus, When::Always, &WORLD),
    group("SCREEN", Tab::Menus, When::Always, &SCREEN),
];

/// One key of one row that a player can change: named by its group and its
/// default key (`"flying:KeyQ"`), which is the key the game's readers press,
/// so the name says exactly what a rebind stands in for.
#[derive(Clone, Copy, Debug)]
pub struct Slot {
    pub group: &'static Group,
    pub row: &'static Binding,
    /// Where in the row's keys it is.
    pub index: usize,
    pub default: KeyCode,
}

impl Slot {
    /// The slot's name in the settings file.
    pub fn name(&self) -> String {
        format!(
            "{}:{}",
            self.group.heading.to_lowercase().replace(' ', "_"),
            key_name(self.default)
        )
    }
}

/// Whether a key of a row can be given another key: a keyboard key, not the
/// menu's Escape, not part of a range.
fn is_slot(row: &Binding, key: Key) -> Option<KeyCode> {
    match key {
        Key::Board(code) if code != KeyCode::Escape && !row.range && !row.chord => Some(code),
        _ => None,
    }
}

/// Every slot in the table, in its order.
pub fn slots() -> impl Iterator<Item = Slot> {
    BINDINGS.iter().flat_map(|group| {
        group.rows.iter().flat_map(move |row| {
            row.keys
                .iter()
                .enumerate()
                .filter_map(move |(index, &key)| {
                    is_slot(row, key).map(|default| Slot {
                        group,
                        row,
                        index,
                        default,
                    })
                })
        })
    })
}

/// The slot of a group's row at a key, if it is one.
pub fn slot_at(group: &'static Group, row: &'static Binding, index: usize) -> Option<Slot> {
    is_slot(row, *row.keys.get(index)?).map(|default| Slot {
        group,
        row,
        index,
        default,
    })
}

/// The player's rebinds: slot name to key, only for a slot moved off its
/// default, kept in `settings.ron` (`settings::VERSION` 3). Empty for a
/// player who never rebinds, for every capture and for every benchmark, and
/// then [`remap_keys`] touches nothing.
#[derive(Resource, Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyBinds {
    moved: BTreeMap<String, KeyCode>,
    /// The file's rebinds this build does not know (a removed slot's, such
    /// as the retired Kestrel's, or a later build's): read by nothing, and
    /// written back as they were read, so the file is never rewritten
    /// without them (`retire-kestrel` decision 3).
    kept: BTreeMap<String, String>,
}

impl KeyBinds {
    /// The key a slot reads now.
    pub fn key(&self, slot: &Slot) -> KeyCode {
        self.moved
            .get(&slot.name())
            .copied()
            .unwrap_or(slot.default)
    }

    /// Give a slot a key; its own default takes it off the list.
    pub fn set(&mut self, slot: &Slot, key: KeyCode) {
        self.kept.remove(&slot.name());
        if key == slot.default {
            self.moved.remove(&slot.name());
        } else {
            self.moved.insert(slot.name(), key);
        }
    }

    /// Put every slot on a sub-tab back to its default.
    pub fn reset(&mut self, tab: Tab) {
        for slot in slots().filter(|s| s.group.tab == tab) {
            self.moved.remove(&slot.name());
            self.kept.remove(&slot.name());
        }
    }

    /// No rebind is read: a kept one this build does not know moves no key.
    pub fn is_empty(&self) -> bool {
        self.moved.is_empty()
    }

    /// The file's `keys`: slot name to key name, in one order, the ones this
    /// build does not know among them as they were read.
    pub fn to_names(&self) -> BTreeMap<String, String> {
        self.kept
            .iter()
            .map(|(slot, key)| (slot.clone(), key.clone()))
            .chain(
                self.moved
                    .iter()
                    .map(|(slot, key)| (slot.clone(), key_name(*key))),
            )
            .collect()
    }

    /// From the file's `keys`. A slot or a key this build does not know (a
    /// removed slot's, or a later build's) is skipped, named in the second
    /// list, and kept to be written back; the rest load.
    pub fn from_names(names: &BTreeMap<String, String>) -> (Self, Vec<String>) {
        let known: Vec<Slot> = slots().collect();
        let mut binds = Self::default();
        let mut skipped = Vec::new();
        let names = migrate_interact(names);
        for (slot, key) in &names {
            match (known.iter().find(|s| s.name() == *slot), key_named(key)) {
                (Some(slot), Some(key)) => binds.set(slot, key),
                _ => {
                    skipped.push(format!("{slot} = {key}"));
                    binds.kept.insert(slot.clone(), key.clone());
                }
            }
        }
        (binds, skipped)
    }

    /// The other actions that read `slot`'s key while it is read: a clash to
    /// flag on its row. The defaults share keys only between contexts that
    /// are never live together (W is move on foot and sheet in in the Tern),
    /// so with no rebinds nothing is flagged.
    pub fn clashes(&self, slot: &Slot) -> Vec<&'static str> {
        let key = self.key(slot);
        let mut also: Vec<&'static str> = Vec::new();
        for group in &BINDINGS {
            if !group.when.together(slot.group.when) {
                continue;
            }
            for row in group.rows {
                let reads = |other: KeyCode| -> bool {
                    if row.chord {
                        // Read whole, off the physical keyboard: F alone is
                        // still interact's.
                        return false;
                    }
                    if row.range {
                        // The number row reads every digit between its ends.
                        return DIGITS.contains(&other);
                    }
                    row.keys.iter().enumerate().any(|(index, &k)| {
                        let read = slot_at(group, row, index)
                            .map(|s| self.key(&s))
                            .or(match k {
                                Key::Board(code) => Some(code),
                                _ => None,
                            });
                        let same = std::ptr::eq(row, slot.row) && index == slot.index;
                        !same && read == Some(other)
                    })
                };
                if reads(key) && !also.contains(&row.does) {
                    also.push(row.does);
                }
            }
        }
        also
    }
}

/// The rebinds of the two slots `interact:KeyF` replaced, moved onto it
/// (settings version 8): a file that already names it is left alone; else
/// the first of [`BEFORE_INTERACT`] it names is moved, and any other is left
/// in place, unread and written back as it was.
fn migrate_interact(names: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    let mut names = names.clone();
    if names.contains_key(INTERACT_SLOT) {
        return names;
    }
    if let Some(old) = BEFORE_INTERACT.iter().find(|old| names.contains_key(**old)) {
        let key = names.remove(*old).expect("found");
        names.insert(INTERACT_SLOT.to_string(), key);
    }
    names
}

/// The number row, which the slots read one key each.
const DIGITS: [KeyCode; 10] = [
    KeyCode::Digit0,
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
    KeyCode::Digit6,
    KeyCode::Digit7,
    KeyCode::Digit8,
    KeyCode::Digit9,
];

impl Binding {
    /// The key column with the player's keys: what the settings page, an
    /// instrument panel and `--help` print. One function, so they cannot
    /// print a control differently.
    /// Each key's own label, with the player's rebinds: one glyph a key.
    pub fn key_labels(&'static self, group: &'static Group, binds: &KeyBinds) -> Vec<&'static str> {
        self.keys
            .iter()
            .enumerate()
            .map(|(index, key)| match slot_at(group, self, index) {
                Some(slot) => key_label(binds.key(&slot)),
                None => key.label(),
            })
            .collect()
    }

    pub fn keys_label(&'static self, group: &'static Group, binds: &KeyBinds) -> String {
        let this = self;
        this.keys
            .iter()
            .enumerate()
            .map(|(index, key)| match slot_at(group, this, index) {
                Some(slot) => key_label(binds.key(&slot)),
                None => key.label(),
            })
            .collect::<Vec<_>>()
            .join(this.joiner)
    }
}

/// The controls under one heading, as one line: what an instrument panel
/// prints, off the same table the settings page reads, with the player's keys.
/// A group's rows, one `keys  what` string each, off the one table: what a
/// boat's panel lists (`small-boats`).
/// A group's rows as each key's label and what the row does: what a key
/// glyph row draws (`small-boats`).
pub fn caps(heading: &str, binds: &KeyBinds) -> Vec<(Vec<&'static str>, &'static str)> {
    BINDINGS
        .iter()
        .filter(|group| group.heading == heading)
        .flat_map(|group| group.rows.iter().map(move |row| (group, row)))
        .map(|(group, binding)| (binding.key_labels(group, binds), binding.does))
        .collect()
}

pub fn rows(heading: &str, binds: &KeyBinds) -> Vec<String> {
    BINDINGS
        .iter()
        .filter(|group| group.heading == heading)
        .flat_map(|group| group.rows.iter().map(move |row| (group, row)))
        .map(|(group, binding)| format!("{}  {}", binding.keys_label(group, binds), binding.does))
        .collect()
}

pub fn line(heading: &str, binds: &KeyBinds) -> String {
    BINDINGS
        .iter()
        .filter(|group| group.heading == heading)
        .flat_map(|group| group.rows.iter().map(move |row| (group, row)))
        .map(|(group, binding)| format!("{} {}", binding.keys_label(group, binds), binding.does))
        .collect::<Vec<_>>()
        .join("   ")
}

/// The binding list for `--help`, so the terminal and the settings page read
/// the same table. The defaults: `--help` reads no settings file.
pub fn help_text() -> String {
    let binds = KeyBinds::default();
    let mut out = String::new();
    for group in &BINDINGS {
        out.push_str(&format!("\n  {}\n", group.heading));
        for binding in group.rows {
            out.push_str(&format!(
                "    {:<16}{}\n",
                binding.keys_label(group, &binds),
                binding.does
            ));
        }
    }
    out
}

/// Where the player is: aboard a craft of its kind, else on foot while the
/// walker is active, else flying.
fn live_now(world_aboard: Option<Kind>, walking: bool) -> Live {
    match world_aboard {
        Some(kind) => Live::Aboard(kind),
        None if walking => Live::OnFoot,
        None => Live::Flying,
    }
}

/// [`remap_keys`]'s memory between frames: the physical keyboard, and the
/// keys it set last frame with how it set them.
#[derive(Default)]
pub struct RemapState {
    physical: ButtonInput<KeyCode>,
    touched: Vec<KeyCode>,
    held: Vec<KeyCode>,
}

/// The pairs the remap applies now: each live slot's default key and the key
/// that stands for it. None while a menu is open (the menus, the name field
/// and the rebind capture read the real keyboard) or with no rebinds.
pub fn live_pairs(binds: &KeyBinds, now: Live, menu_open: bool) -> Vec<(KeyCode, KeyCode)> {
    if menu_open || binds.is_empty() {
        return Vec::new();
    }
    slots()
        .filter(|slot| slot.group.when.live(now))
        .map(|slot| (slot.default, binds.key(&slot)))
        .collect()
}

/// The pressed state every key the remap owns should have this frame, from
/// the physical keyboard: a live slot's default key is down while any key
/// standing for it is; any other key it owns is its physical self.
pub fn remapped(
    pairs: &[(KeyCode, KeyCode)],
    physical: &ButtonInput<KeyCode>,
    owned: &[KeyCode],
) -> Vec<(KeyCode, bool)> {
    owned
        .iter()
        .map(|&key| {
            let mut stands_for = pairs
                .iter()
                .filter(|(default, _)| *default == key)
                .peekable();
            let down = if stands_for.peek().is_some() {
                stands_for.any(|(_, by)| physical.pressed(*by))
            } else {
                physical.pressed(key)
            };
            (key, down)
        })
        .collect()
}

/// Rebinding without rewriting a reader (design, "how the game obeys it"):
/// after Bevy's input systems and before every reader, set the state of the
/// keys a rebind touches in `ButtonInput<KeyCode>` from the physical
/// keyboard, so each reader goes on reading its default key. With no
/// rebinds, and while a menu is open, it touches nothing but what it touched
/// last frame, which it hands back to the physical keyboard.
#[allow(clippy::too_many_arguments)]
pub fn remap_keys(
    keys: Option<ResMut<ButtonInput<KeyCode>>>,
    typed: Option<MessageReader<bevy::input::keyboard::KeyboardInput>>,
    lost: Option<MessageReader<bevy::input::keyboard::KeyboardFocusLost>>,
    binds: Option<Res<KeyBinds>>,
    menu: Option<Res<MenuOpen>>,
    walking: Option<Res<crate::walking::WalkingReadout>>,
    aboard: Option<Res<crate::vehicles::Aboard>>,
    vehicles: Query<&crate::vehicles::Vehicle>,
    mut state: Local<RemapState>,
) {
    // A world with no keyboard (a test's, a headless tool's) has nothing to
    // remap.
    let (Some(mut keys), Some(mut typed), Some(mut lost)) = (keys, typed, lost) else {
        return;
    };
    // The physical keyboard, kept as Bevy keeps its own.
    state.physical.clear();
    for event in typed.read() {
        if event.state.is_pressed() {
            state.physical.press(event.key_code);
        } else {
            state.physical.release(event.key_code);
        }
    }
    if lost.read().count() > 0 {
        state.physical.release_all();
    }
    let none = KeyBinds::default();
    let binds = binds.as_deref().unwrap_or(&none);
    if binds.is_empty() && state.touched.is_empty() {
        return;
    }
    let kind = aboard
        .and_then(|a| a.0)
        .and_then(|e| vehicles.get(e).ok())
        .map(|v| v.craft.kind);
    let now = live_now(kind, walking.is_some_and(|w| w.active));
    let pairs = live_pairs(binds, now, menu.is_some_and(|m| m.0));
    let mut owned: Vec<KeyCode> = Vec::new();
    for &(default, by) in pairs.iter().filter(|(d, b)| d != b) {
        for key in [default, by] {
            if !owned.contains(&key) {
                owned.push(key);
            }
        }
    }
    let previous = std::mem::take(&mut state.touched);
    let was_held = std::mem::take(&mut state.held);
    let mut all = owned.clone();
    for key in &previous {
        if !all.contains(key) {
            all.push(*key);
        }
    }
    for (key, down) in remapped(&pairs, &state.physical, &all) {
        // Down last frame: as this system left it, or, for a key it did not
        // own, as Bevy's own system kept it.
        let was = if previous.contains(&key) {
            was_held.contains(&key)
        } else {
            (keys.pressed(key) && !keys.just_pressed(key)) || keys.just_released(key)
        };
        keys.release(key);
        keys.clear_just_released(key);
        keys.clear_just_pressed(key);
        if down {
            keys.press(key);
            if was {
                keys.clear_just_pressed(key);
            }
        } else if was {
            keys.press(key);
            keys.release(key);
            keys.clear_just_pressed(key);
        }
        if down && owned.contains(&key) {
            state.held.push(key);
        }
    }
    state.touched = owned;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// G on foot opens the picker on the press and commits on the release;
    /// not on foot it does nothing; an open menu forgets the key.
    #[test]
    fn g_holds_the_picker_open_only_on_foot() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<PickerKey>()
            .insert_resource(MenuOpen(false))
            .insert_resource(crate::walking::WalkingReadout {
                active: true,
                ..default()
            })
            .add_systems(Update, read_picker_key);
        app.update();
        let key = |app: &App| *app.world().resource::<PickerKey>();
        let press = |app: &mut App| {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyG);
            app.update();
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .clear();
        };
        let release = |app: &mut App| {
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .release(KeyCode::KeyG);
            app.update();
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .clear();
        };

        press(&mut app);
        assert!(key(&app).holding, "the picker opens on the press");
        app.update();
        assert!(
            key(&app).holding && !key(&app).let_go,
            "and stays open while held"
        );
        release(&mut app);
        assert!(
            key(&app).let_go && !key(&app).holding,
            "the release commits"
        );
        app.update();
        assert!(!key(&app).let_go, "once");

        // In a menu the key is forgotten, and nothing is committed.
        press(&mut app);
        app.world_mut().resource_mut::<MenuOpen>().0 = true;
        app.update();
        assert!(!key(&app).holding);
        app.world_mut().resource_mut::<MenuOpen>().0 = false;
        release(&mut app);
        assert!(
            !key(&app).let_go,
            "a key pressed before the menu commits nothing"
        );

        // Not on foot (aboard, or flying), G does nothing.
        app.world_mut()
            .resource_mut::<crate::walking::WalkingReadout>()
            .active = false;
        press(&mut app);
        assert!(!key(&app).holding, "no picker unless on foot");
        release(&mut app);
        assert!(!key(&app).let_go);
    }

    /// Every file that presses the keyboard or the mouse. The test reads their
    /// SOURCE rather than a second list in here, which is this repository's
    /// rule for a representation it cannot collapse: check the real artifact.
    const READERS: [(&str, &str); 14] = [
        ("controls.rs", include_str!("controls.rs")),
        ("fish.rs", include_str!("fish.rs")),
        ("desktop/guide.rs", include_str!("desktop/guide.rs")),
        ("desktop/pack.rs", include_str!("desktop/pack.rs")),
        (
            "desktop/map_screen.rs",
            include_str!("desktop/map_screen.rs"),
        ),
        ("vehicles.rs", include_str!("vehicles.rs")),
        ("walking.rs", include_str!("walking.rs")),
        ("flight_view/input.rs", include_str!("flight_view/input.rs")),
        ("weather.rs", include_str!("weather.rs")),
        ("desktop/slots.rs", include_str!("desktop/slots.rs")),
        ("desktop/digging.rs", include_str!("desktop/digging.rs")),
        ("desktop/menu.rs", include_str!("desktop/menu.rs")),
        ("desktop.rs", include_str!("desktop.rs")),
        ("desktop/repro.rs", include_str!("desktop/repro.rs")),
    ];

    /// A binding list that names a key nothing reads is a lie a player acts
    /// on, and it is exactly what happened to the panel this table replaced:
    /// it advertised `H` for a panel and never learned about the shovel.
    ///
    /// What this proves is narrow and worth stating: that the table's KEYS are
    /// real. It cannot prove the prose beside them, because no grep
    /// establishes that `R` swaps to flying. Staleness is the failure that
    /// actually happened, and staleness is what it catches.
    #[test]
    fn a_named_key_is_a_key_something_reads() {
        for group in &BINDINGS {
            for binding in group.rows {
                for key in binding.keys {
                    let token = key.token();
                    let found = READERS.iter().any(|(_, source)| source.contains(&token));
                    assert!(
                        found,
                        "{} is on the {} list as '{}' and no system reads it",
                        token, group.heading, binding.does
                    );
                }
            }
        }
    }

    /// A key with no label prints as a blank row, which is worse than being
    /// absent: the action is there with nothing to press.
    #[test]
    fn every_named_key_prints_something() {
        for group in &BINDINGS {
            for binding in group.rows {
                for key in binding.keys {
                    assert!(
                        !key.label().is_empty(),
                        "{:?} on the {} list has no label",
                        key,
                        group.heading
                    );
                }
                assert!(!binding.does.is_empty(), "a row with no action");
            }
        }
    }

    /// Holding takes the pointer away and releasing gives it straight back,
    /// which is what makes Escape, RESUME and mouse look one motion.
    #[test]
    fn the_pointer_comes_back_when_the_menu_closes() {
        let (mut was_open, mut captured) = (false, true);
        hand_over_pointer(true, &mut was_open, &mut captured);
        assert!(!captured, "a menu holds the pointer");
        hand_over_pointer(true, &mut was_open, &mut captured);
        assert!(!captured, "and goes on holding it");
        hand_over_pointer(false, &mut was_open, &mut captured);
        assert!(captured, "and hands it back on the frame it closes");
    }

    /// With no menu in the picture it must not touch the flag at all, or every
    /// Escape-frees-the-cursor path would re-grab on the next frame.
    #[test]
    fn a_closed_menu_never_takes_the_pointer_back_twice() {
        let (mut was_open, mut captured) = (false, false);
        for _ in 0..4 {
            hand_over_pointer(false, &mut was_open, &mut captured);
        }
        assert!(!captured, "nothing opened, so nothing was handed back");
    }

    /// A slot is named by its group and its default key, so no key may
    /// appear twice in one group, and every name is a different slot.
    #[test]
    fn every_slot_has_its_own_name() {
        let names: Vec<String> = slots().map(|s| s.name()).collect();
        for (i, name) in names.iter().enumerate() {
            assert!(!names[..i].contains(name), "two slots are {name}");
        }
        assert!(names.contains(&"flying:KeyQ".to_string()));
        assert!(names.contains(&"on_foot:ShiftLeft".to_string()));
        // Not slots: the mouse, the wheel, the number row's range and Esc.
        assert!(
            !names
                .iter()
                .any(|n| n.contains("Digit") || n.contains("Escape"))
        );
        // Every default a slot names is a key a cap can print and the file
        // can write back.
        for slot in slots() {
            assert!(!key_label(slot.default).is_empty(), "{}", slot.name());
            assert_eq!(key_named(&key_name(slot.default)), Some(slot.default));
        }
    }

    /// Every sub-tab has rows, and every group is on one (decision 6).
    #[test]
    fn the_input_page_has_five_sub_tabs_and_every_group_is_on_one() {
        for tab in Tab::ALL {
            assert!(
                BINDINGS.iter().any(|g| g.tab == tab),
                "{} is empty",
                tab.label()
            );
        }
        let on = |heading: &str| BINDINGS.iter().find(|g| g.heading == heading).unwrap().tab;
        assert_eq!(on("ON FOOT"), Tab::OnFoot);
        assert_eq!(on("HANDS"), Tab::Hands);
        assert_eq!(on("FLYING"), Tab::Flying);
        assert_eq!(on("COG"), Tab::Vehicles);
        assert_eq!(on("SCREEN"), Tab::Menus);
    }

    /// The defaults share keys only between contexts that are never live
    /// together, so a player who never rebinds sees no clash; a rebind onto
    /// a key a live action reads is flagged with that action.
    #[test]
    fn the_defaults_never_clash_and_a_rebind_onto_a_live_key_does() {
        let none = KeyBinds::default();
        for slot in slots() {
            assert!(
                none.clashes(&slot).is_empty(),
                "{} clashes with {:?}",
                slot.name(),
                none.clashes(&slot)
            );
        }
        let sprint = slots().find(|s| s.name() == "on_foot:ShiftLeft").unwrap();
        let mut binds = KeyBinds::default();
        binds.set(&sprint, KeyCode::KeyF);
        assert_eq!(
            binds.clashes(&sprint),
            vec!["open, harvest, pick, feed, till; board or step off"]
        );
        // E is free on foot (owner, 2026-11-16): the Tern's hiking and the
        // roll read it, but never on foot.
        binds.set(&sprint, KeyCode::KeyE);
        assert!(binds.clashes(&sprint).is_empty());
        // A digit is the number row's, which reads every digit.
        binds.set(&sprint, KeyCode::Digit5);
        assert_eq!(binds.clashes(&sprint), vec!["select a slot"]);
        // The other side of the clash is flagged too.
        binds.set(&sprint, KeyCode::KeyF);
        let door = slots().find(|s| s.name() == INTERACT_SLOT).unwrap();
        assert_eq!(binds.clashes(&door), vec!["sprint"]);
        // Its own default takes a slot off the list; a tab resets its own.
        binds.set(&sprint, KeyCode::ShiftLeft);
        assert!(binds.is_empty());
        binds.set(&sprint, KeyCode::KeyK);
        binds.reset(Tab::Hands);
        assert!(!binds.is_empty(), "another tab's reset leaves it");
        binds.reset(Tab::OnFoot);
        assert!(binds.is_empty());
    }

    /// Interact on F (owner, 2026-11-16): "use" and "board" are one slot.
    /// A player who moved "use" keeps that key as interact; else one who
    /// moved "board" keeps that; with both moved, use's wins and the
    /// board's is kept in the file, unread; a file that already names the
    /// interact slot is read as it is.
    #[test]
    fn an_old_use_or_board_rebind_becomes_the_interact_key() {
        let interact = slots().find(|s| s.name() == INTERACT_SLOT).unwrap();
        let read = |pairs: &[(&str, &str)]| {
            let names: BTreeMap<String, String> = pairs
                .iter()
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect();
            KeyBinds::from_names(&names)
        };
        assert_eq!(KeyBinds::default().key(&interact), KeyCode::KeyF);
        let (binds, skipped) = read(&[("hands:KeyE", "KeyK")]);
        assert_eq!(binds.key(&interact), KeyCode::KeyK, "use's key kept");
        assert!(skipped.is_empty());
        assert_eq!(
            binds.to_names(),
            BTreeMap::from([(INTERACT_SLOT.to_string(), "KeyK".to_string())]),
            "written back under the new name"
        );
        let (binds, _) = read(&[("vehicles:KeyF", "KeyJ")]);
        assert_eq!(binds.key(&interact), KeyCode::KeyJ, "board's key kept");
        let (binds, skipped) = read(&[("hands:KeyE", "KeyK"), ("vehicles:KeyF", "KeyJ")]);
        assert_eq!(binds.key(&interact), KeyCode::KeyK, "use's wins");
        assert_eq!(skipped, vec!["vehicles:KeyF = KeyJ".to_string()]);
        assert_eq!(
            binds.to_names().get("vehicles:KeyF").map(String::as_str),
            Some("KeyJ"),
            "the other is kept in the file"
        );
        // Use moved onto F, where board was: F is the default now.
        let (binds, _) = read(&[("hands:KeyE", "KeyF")]);
        assert!(binds.is_empty());
        assert_eq!(binds.key(&interact), KeyCode::KeyF);
        // A new file is read as it is.
        let (binds, _) = read(&[(INTERACT_SLOT, "KeyG"), ("hands:KeyE", "KeyK")]);
        assert_eq!(binds.key(&interact), KeyCode::KeyG);
        // E reads nothing on foot now; Q/E still roll in flight.
        let on_foot = live_pairs(&binds, Live::OnFoot, false);
        assert!(on_foot.iter().all(|(d, _)| *d != KeyCode::KeyE));
        assert_eq!(
            line("FLYING", &KeyBinds::default()).split("   ").nth(2),
            Some("Q / E roll")
        );
    }

    /// The file's names round trip, and a slot or key this build does not
    /// know is skipped, not fatal, and kept to be written back.
    #[test]
    fn rebinds_round_trip_through_their_names() {
        let mut binds = KeyBinds::default();
        let roll = slots().find(|s| s.name() == "flying:KeyQ").unwrap();
        binds.set(&roll, KeyCode::ArrowLeft);
        let names = binds.to_names();
        assert_eq!(
            names.get("flying:KeyQ").map(String::as_str),
            Some("ArrowLeft")
        );
        let (back, skipped) = KeyBinds::from_names(&names);
        assert_eq!(back, binds);
        assert!(skipped.is_empty());
        let mut later = names.clone();
        later.insert("jetpack:KeyJ".into(), "KeyU".into());
        later.insert("on_foot:KeyW".into(), "Hyper".into());
        let (read, skipped) = KeyBinds::from_names(&later);
        assert_eq!(read.moved, binds.moved, "only the known one is read");
        assert_eq!(skipped.len(), 2);
        assert_eq!(read.to_names(), later, "and the rest are written back");
        assert_eq!(
            line("FLYING", &binds).split("   ").nth(2),
            Some("← / E roll")
        );
    }

    /// The remap: a moved slot's default key is down while its new key is,
    /// not while its old one is; a key of another context is its own self;
    /// a menu hands the real keyboard back; and with no rebinds nothing is
    /// touched at all.
    #[test]
    fn a_rebound_key_presses_the_default_its_readers_read() {
        use bevy::input::{ButtonState, keyboard::Key as Logical, keyboard::KeyboardInput};
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::input::InputPlugin))
            .insert_resource(MenuOpen(false))
            .insert_resource(crate::walking::WalkingReadout {
                active: true,
                ..default()
            })
            .init_resource::<KeyBinds>()
            .add_systems(PreUpdate, remap_keys.after(bevy::input::InputSystems));
        let send = |app: &mut App, key: KeyCode, down: bool| {
            app.world_mut().write_message(KeyboardInput {
                key_code: key,
                logical_key: Logical::Unidentified(bevy::input::keyboard::NativeKey::Unidentified),
                state: if down {
                    ButtonState::Pressed
                } else {
                    ButtonState::Released
                },
                text: None,
                repeat: false,
                window: Entity::PLACEHOLDER,
            });
        };
        let keys = |app: &App| app.world().resource::<ButtonInput<KeyCode>>().clone();

        // No rebinds: the remap leaves what is set alone (a capture's press).
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyW);
        app.update();
        assert!(
            keys(&app).pressed(KeyCode::KeyW),
            "untouched with no rebinds"
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::KeyW);

        // Interact (on foot) moved from F to K.
        let door = slots().find(|s| s.name() == INTERACT_SLOT).unwrap();
        app.world_mut()
            .resource_mut::<KeyBinds>()
            .set(&door, KeyCode::KeyK);
        send(&mut app, KeyCode::KeyK, true);
        app.update();
        assert!(keys(&app).just_pressed(KeyCode::KeyF), "K interacts");
        app.update();
        assert!(keys(&app).pressed(KeyCode::KeyF) && !keys(&app).just_pressed(KeyCode::KeyF));
        send(&mut app, KeyCode::KeyK, false);
        app.update();
        assert!(keys(&app).just_released(KeyCode::KeyF) && !keys(&app).pressed(KeyCode::KeyF));
        send(&mut app, KeyCode::KeyF, true);
        app.update();
        assert!(
            !keys(&app).pressed(KeyCode::KeyF),
            "F no longer interacts on foot"
        );
        assert!(!keys(&app).just_pressed(KeyCode::KeyF));
        send(&mut app, KeyCode::KeyF, false);
        app.update();

        // Flying, interact is not read: F is its physical self, and E rolls.
        app.world_mut()
            .resource_mut::<crate::walking::WalkingReadout>()
            .active = false;
        send(&mut app, KeyCode::KeyE, true);
        app.update();
        assert!(keys(&app).pressed(KeyCode::KeyE), "E rolls in flight");
        send(&mut app, KeyCode::KeyE, false);
        app.update();

        // A menu hands the real keyboard back: K is K and opens nothing.
        app.world_mut()
            .resource_mut::<crate::walking::WalkingReadout>()
            .active = true;
        app.world_mut().resource_mut::<MenuOpen>().0 = true;
        send(&mut app, KeyCode::KeyK, true);
        app.update();
        assert!(keys(&app).just_pressed(KeyCode::KeyK));
        assert!(
            !keys(&app).pressed(KeyCode::KeyF),
            "the menu reads the real keys"
        );
    }
}
