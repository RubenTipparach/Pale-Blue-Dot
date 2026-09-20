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

impl Key {
    /// What the settings page and `--help` print. ASCII only, which is the
    /// inherited rule for anything a player reads.
    pub fn label(self) -> &'static str {
        match self {
            Key::Board(code) => match code {
                KeyCode::KeyA => "A",
                KeyCode::KeyB => "B",
                KeyCode::KeyD => "D",
                KeyCode::KeyE => "E",
                KeyCode::KeyF => "F",
                KeyCode::KeyP => "P",
                KeyCode::KeyQ => "Q",
                KeyCode::KeyR => "R",
                KeyCode::KeyS => "S",
                KeyCode::KeyW => "W",
                KeyCode::KeyX => "X",
                KeyCode::Space => "SPACE",
                KeyCode::ShiftLeft | KeyCode::ShiftRight => "SHIFT",
                KeyCode::ControlLeft | KeyCode::ControlRight => "CTRL",
                KeyCode::Escape => "ESC",
                KeyCode::F12 => "F12",
                KeyCode::Digit0 => "0",
                KeyCode::Digit1 => "1",
                // A key with no label would print as nothing at all, which is
                // a blank row in the settings page. The test refuses one.
                _ => "",
            },
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
pub struct Binding {
    /// The keys, in the order they are printed. Several means either: `W A S D`
    /// is four keys for one action, and `SPACE / CTRL` is a pair.
    pub keys: &'static [Key],
    /// The separator between them when printed.
    pub joiner: &'static str,
    pub does: &'static str,
}

/// A heading and the controls under it.
pub struct Group {
    pub heading: &'static str,
    pub rows: &'static [Binding],
}

const fn row(keys: &'static [Key], joiner: &'static str, does: &'static str) -> Binding {
    Binding { keys, joiner, does }
}

const WASD: [Key; 4] = [
    Key::Board(KeyCode::KeyW),
    Key::Board(KeyCode::KeyA),
    Key::Board(KeyCode::KeyS),
    Key::Board(KeyCode::KeyD),
];

const ON_FOOT: [Binding; 7] = [
    row(&WASD, " ", "move"),
    row(
        &[Key::Board(KeyCode::Space)],
        " ",
        "jump, and rise in water",
    ),
    row(&[Key::Board(KeyCode::ShiftLeft)], " ", "sprint"),
    row(&[Key::Mouse(MouseButton::Left)], " ", "dig the block ahead"),
    row(
        &[Key::Mouse(MouseButton::Right)],
        " ",
        "place the held block",
    ),
    row(
        &[Key::Board(KeyCode::Digit1), Key::Board(KeyCode::Digit0)],
        " - ",
        "select a slot",
    ),
    row(&[Key::Wheel], " ", "step through the slots"),
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

const WORLD: [Binding; 3] = [
    row(&[Key::Board(KeyCode::KeyF)], " ", "walk or fly"),
    row(&[Key::Board(KeyCode::KeyR)], " ", "return to the spawn"),
    row(&[Key::Board(KeyCode::KeyP)], " ", "cycle the weather"),
];

const SCREEN: [Binding; 2] = [
    row(&[Key::Board(KeyCode::Escape)], " ", "menu"),
    row(&[Key::Board(KeyCode::F12)], " ", "screenshot"),
];

/// Every control in the game, once.
pub const BINDINGS: [Group; 4] = [
    Group {
        heading: "ON FOOT",
        rows: &ON_FOOT,
    },
    Group {
        heading: "FLYING",
        rows: &FLYING,
    },
    Group {
        heading: "WORLD",
        rows: &WORLD,
    },
    Group {
        heading: "SCREEN",
        rows: &SCREEN,
    },
];

impl Binding {
    /// The key column: what the settings page and `--help` both print on the
    /// left. One function, so the two cannot print a control differently.
    pub fn keys_label(&self) -> String {
        self.keys
            .iter()
            .map(|key| key.label())
            .collect::<Vec<_>>()
            .join(self.joiner)
    }
}

/// The binding list for `--help`, so the terminal and the settings page read
/// the same table.
pub fn help_text() -> String {
    let mut out = String::new();
    for group in &BINDINGS {
        out.push_str(&format!("\n  {}\n", group.heading));
        for binding in group.rows {
            out.push_str(&format!(
                "    {:<16}{}\n",
                binding.keys_label(),
                binding.does
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every file that presses the keyboard or the mouse. The test reads their
    /// SOURCE rather than a second list in here, which is this repository's
    /// rule for a representation it cannot collapse: check the real artifact.
    const READERS: [(&str, &str); 7] = [
        ("walking.rs", include_str!("walking.rs")),
        ("flight_view/input.rs", include_str!("flight_view/input.rs")),
        ("weather.rs", include_str!("weather.rs")),
        ("desktop/slots.rs", include_str!("desktop/slots.rs")),
        ("desktop/digging.rs", include_str!("desktop/digging.rs")),
        ("desktop/menu.rs", include_str!("desktop/menu.rs")),
        ("desktop.rs", include_str!("desktop.rs")),
    ];

    /// A binding list that names a key nothing reads is a lie a player acts
    /// on, and it is exactly what happened to the panel this table replaced:
    /// it advertised `H` for a panel and never learned about the shovel.
    ///
    /// What this proves is narrow and worth stating: that the table's KEYS are
    /// real. It cannot prove the prose beside them, because no grep
    /// establishes that `F` swaps to flying. Staleness is the failure that
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
}
