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

/// Read `G` into [`PickerKey`]. Only on foot: aboard or flying, G does
/// nothing, and a menu forgets the key so Escape closes the picker without
/// equipping anything.
pub fn read_picker_key(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    menu: Option<Res<MenuOpen>>,
    walking: Option<Res<crate::walking::WalkingReadout>>,
    aboard: Option<Res<crate::vehicles::Aboard>>,
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
    let seated = aboard.is_some_and(|a| a.0.is_some());
    let on_foot = walking.is_some_and(|w| w.active) && !seated;
    if keys.just_pressed(KeyCode::KeyG) && on_foot {
        key.holding = true;
    }
    if key.holding && (!keys.pressed(KeyCode::KeyG) || !on_foot) {
        key.holding = false;
        key.let_go = on_foot;
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
                KeyCode::KeyC => "C",
                KeyCode::KeyF => "F",
                KeyCode::KeyG => "G",
                KeyCode::KeyH => "H",
                KeyCode::KeyJ => "J",
                KeyCode::KeyP => "P",
                KeyCode::KeyQ => "Q",
                KeyCode::KeyR => "R",
                KeyCode::KeyS => "S",
                KeyCode::KeyT => "T",
                KeyCode::KeyV => "V",
                KeyCode::KeyW => "W",
                KeyCode::KeyX => "X",
                KeyCode::KeyZ => "Z",
                KeyCode::Space => "SPACE",
                KeyCode::ShiftLeft | KeyCode::ShiftRight => "SHIFT",
                KeyCode::ControlLeft | KeyCode::ControlRight => "CTRL",
                KeyCode::Escape => "ESC",
                KeyCode::F12 => "F12",
                KeyCode::F3 => "F3",
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

const ON_FOOT: [Binding; 9] = [
    row(&WASD, " ", "move"),
    row(
        &[Key::Board(KeyCode::Space)],
        " ",
        "jump, and rise in water",
    ),
    row(&[Key::Board(KeyCode::ShiftLeft)], " ", "sprint"),
    row(
        &[Key::Mouse(MouseButton::Left)],
        " ",
        "use the tool in hand: dig, or cast, hook and reel",
    ),
    row(
        &[Key::Mouse(MouseButton::Right)],
        " ",
        "place the held block, or wind the line in",
    ),
    row(
        &[Key::Board(KeyCode::Digit1), Key::Board(KeyCode::Digit0)],
        " - ",
        "select a slot",
    ),
    row(&[Key::Wheel], " ", "step through the slots"),
    row(
        &[Key::Board(KeyCode::KeyG)],
        " ",
        "hold for the tools; the wheel picks one",
    ),
    row(&[Key::Board(KeyCode::KeyJ)], " ", "the field guide"),
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

const VEHICLES: [Binding; 3] = [
    row(&[Key::Board(KeyCode::KeyF)], " ", "board, or step off"),
    row(&[Key::Board(KeyCode::KeyV)], " ", "seat or chase view"),
    row(
        &[Key::Board(KeyCode::KeyT)],
        " ",
        "anchor a boat, or weigh it",
    ),
];

const KESTREL: [Binding; 7] = [
    row(&W_S, "/", "pitch"),
    row(&A_D, "/", "roll"),
    row(&Q_E, "/", "yaw"),
    row(
        &[Key::Board(KeyCode::Space), Key::Board(KeyCode::ControlLeft)],
        "/",
        "power",
    ),
    row(
        &[Key::Board(KeyCode::KeyZ), Key::Board(KeyCode::KeyC)],
        "/",
        "nacelles ahead or up",
    ),
    row(&[Key::Board(KeyCode::KeyX)], " ", "assist"),
    row(&[Key::Board(KeyCode::KeyB)], " ", "wheel brake"),
];

const TERN: [Binding; 4] = [
    row(&A_D, "/", "tiller"),
    row(&W_S, "/", "sheet in or ease"),
    row(&Q_E, "/", "hike to port or starboard"),
    row(&[Key::Board(KeyCode::KeyB)], " ", "bail"),
];

const LOON: [Binding; 4] = [
    row(&W_S, "/", "paddle ahead or back"),
    row(&A_D, "/", "steer"),
    row(&Q_E, "/", "blade as a rudder"),
    row(&[Key::Board(KeyCode::KeyB)], " ", "bail"),
];

const WORLD: [Binding; 3] = [
    row(&[Key::Board(KeyCode::KeyR)], " ", "walk or fly"),
    row(&[Key::Board(KeyCode::KeyH)], " ", "return to the spawn"),
    row(&[Key::Board(KeyCode::KeyP)], " ", "cycle the weather"),
];

const SCREEN: [Binding; 3] = [
    row(&[Key::Board(KeyCode::Escape)], " ", "menu"),
    row(&[Key::Board(KeyCode::F12)], " ", "screenshot"),
    row(&[Key::Board(KeyCode::F3)], " ", "frame graph"),
];

/// Every control in the game, once.
pub const BINDINGS: [Group; 8] = [
    Group {
        heading: "ON FOOT",
        rows: &ON_FOOT,
    },
    Group {
        heading: "FLYING",
        rows: &FLYING,
    },
    Group {
        heading: "VEHICLES",
        rows: &VEHICLES,
    },
    Group {
        heading: "KESTREL",
        rows: &KESTREL,
    },
    Group {
        heading: "TERN",
        rows: &TERN,
    },
    Group {
        heading: "LOON",
        rows: &LOON,
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

/// The controls under one heading, as one line: what an instrument panel
/// prints, off the same table the settings page reads.
pub fn line(heading: &str) -> String {
    BINDINGS
        .iter()
        .filter(|group| group.heading == heading)
        .flat_map(|group| group.rows)
        .map(|binding| format!("{} {}", binding.keys_label(), binding.does))
        .collect::<Vec<_>>()
        .join("   ")
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
    const READERS: [(&str, &str); 11] = [
        ("controls.rs", include_str!("controls.rs")),
        ("fish.rs", include_str!("fish.rs")),
        ("desktop/guide.rs", include_str!("desktop/guide.rs")),
        ("vehicles.rs", include_str!("vehicles.rs")),
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
}
