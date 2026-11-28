//! The pause menu and the settings page.
//!
//! A menu is not a panel. A panel is a `Node` with children; a MENU is a
//! second claimant on the pointer, and everything awkward here is that claim.
//! The world's input readers take the pointer on a left click, and a click on
//! RESUME is still a left click, so the rule lives one level up:
//! [`pbd_app::controls::MenuOpen`] says who holds it and every reader stands
//! down while it is set. What is in this file is the drawing, the three
//! buttons and the key that opens them.
//!
//! Both panels are built at startup and hidden with `Display::None` rather
//! than `Visibility::Hidden`, which is the slot row's own lesson: a hidden
//! node is still laid out and still PICKED, so an invisible panel goes on
//! swallowing clicks over the middle of the screen for as long as it is shut.

use bevy::ui::RelativeCursorPosition;
use bevy::{
    app::AppExit,
    ecs::system::SystemParam,
    input::keyboard::{Key, KeyboardInput},
    prelude::*,
};
use pbd_app::config::{Reflections, ShadowQuality};
use pbd_app::controls::{self, BINDINGS, KeyBinds, MenuOpen, Tab};
use pbd_app::saves::{self, Slot, WorldSave};
use pbd_app::settings::{PlayerSettings, SettingsFile};
use pbd_app::ui_kit::{self, KitPiece, KitText, Plaque, Set, UiKit, plaque, plaque_label};
use pbd_core::locale::{Localized, tr, tr_with};

/// Which screen is open. One enum and not two flags, because two flags is how
/// a settings page ends up open over a closed pause menu.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    #[default]
    Playing,
    /// The title screen, the front door of a plain launch (`title-screen`).
    Title,
    Pause,
    Settings,
    Saves,
    /// The title's MULTIPLAYER: the hosted server and a game by its
    /// address (`multiplayer` design 28).
    Multiplayer,
    /// The chat line, open over play (`multiplayer` design 31).
    Chat,
    /// The field guide, opened with J.
    Guide,
    /// The pack, opened with I (`inventory-grid`).
    Pack,
    /// The world map, opened with M (`world-map`).
    Map,
    /// A station's panel, opened with F on it (`farming-and-smelting`).
    Forge,
    /// A kitchen station's panel, opened with F on a campfire (`cooking`).
    Kitchen,
    /// The starter missions, opened with O (`starter-missions`).
    Missions,
    /// The herd's breeding screen, opened with N on foot (`animal-husbandry`).
    Breeding,
    /// Talking to a townsperson, opened with F on them (`townsfolk`).
    Talk,
    /// A town trader's goods, from the talk (`townsfolk` design 6).
    Trade,
    /// A town's jobs, from the talk (`townsfolk` design 8).
    Board,
    /// The building planner, opened with F at a drafting table or O in
    /// build mode (`building-planner` design 3).
    Planner,
    /// "You died" in a Survival world, until Wake up (`survival-mode`
    /// design 8).
    Dead,
}

impl Screen {
    /// One step back out. Settings to pause, pause to the world, and the world
    /// to the menu - which is what `Escape` does from wherever it is pressed.
    ///
    /// The title is the FRONT DOOR of a plain launch (`title-screen`): the
    /// saves page and settings step back to it while it is, and it stays
    /// where it is. Once the world is entered they step back to the pause
    /// menu, and the title is not seen again.
    fn back(self, front_door: bool, _has_world: bool) -> Self {
        if front_door
            && matches!(
                self,
                Screen::Title | Screen::Saves | Screen::Settings | Screen::Multiplayer
            )
        {
            return Screen::Title;
        }
        match self {
            Screen::Playing => Screen::Pause,
            Screen::Title | Screen::Chat => Screen::Playing,
            Screen::Pause
            | Screen::Guide
            | Screen::Pack
            | Screen::Map
            | Screen::Forge
            | Screen::Kitchen
            | Screen::Missions
            | Screen::Breeding
            | Screen::Talk
            | Screen::Planner => Screen::Playing,
            Screen::Trade | Screen::Board => Screen::Talk,
            // Only Wake up leaves it.
            Screen::Dead => Screen::Dead,
            Screen::Settings | Screen::Saves | Screen::Multiplayer => Screen::Pause,
        }
    }
}

/// What a row does when it is pressed. On the entity rather than implied by
/// its position in the list, so reordering the panel cannot silently swap two
/// buttons.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Resume,
    /// The title's NEW WORLD: the saves page, with the keyboard in the name.
    NewWorld,
    Settings,
    Saves,
    /// The pause menu's MAIN MENU: the title, as the front door again.
    MainMenu,
    /// The title's MULTIPLAYER page (`multiplayer` design 28).
    Multiplayer,
    /// Join the Pale Blue Dot server (`join_ui::press` acts).
    JoinHosted,
    /// Join the address typed in the field (`join_ui::press` acts).
    JoinAddress,
    /// Put the keyboard in the address field.
    EditAddress,
    Quit,
    Back,
    /// Open the slot at this row.
    Load(usize),
    /// Ask about the slot at this row. A delete is not done on one press.
    Ask(usize),
    /// Do it.
    Delete(usize),
    /// Do not.
    Keep,
    New,
    /// Put the keyboard in the name field.
    EditName,
    /// Set the storm forcing to this percentage: a weather preset.
    Weather(u8),
    /// Set the clock to this hour: a time preset (`time_ui` acts).
    Time(u8),
    /// Run the clock this many times its own pace (`time-speed`).
    Speed(u16),
    /// Move the clock this many hours ahead (`time-speed`).
    Skip(u8),
    /// Show this overlay: 0 is off, then `Overlay::ALL` in order.
    Overlay(u8),
    /// Open this species' entry in the field guide (`guide::press` acts).
    Species(u16),
    /// Step a row of the Character page (`character_ui::press` acts): the
    /// field's index in `Field::ALL`, and which way.
    Look(u8, i8),
    /// Roll a fresh look on the Character page.
    LookRandom,
    /// Turn bloom on or off (`press_graphics` acts).
    Bloom,
    /// Cycle the frame limit: off, 60, 120 (`player-settings`).
    FrameLimit,
    /// Cycle the render scale through `settings::SCALES`.
    RenderScale,
    /// Turn auto step on or off.
    AutoStep,
    /// Turn the rain shadow (the rain volume) on or off.
    RainShadows,
    /// Set REFLECTIONS: off, low, medium, high (`reflection-quality`).
    Reflections,
    /// Set SHADOW QUALITY: low, medium, high, ultra (`shadow-quality`).
    Shadows,
    /// Turn ANTIALIASING (SMAA) on or off (`smaa-option`).
    Antialiasing,
    /// The starter mission's HUD line, on or off.
    MissionLine,
    /// A sound volume (`audio-and-ambience` 7.2): 0 master, 1 effects,
    /// 2 ambience, 3 music, 4 menus, each 0..=100 % in steps of 10.
    Volume(u8),
    /// Arrows on the flowing water, on or off (`water-flow` 3.4).
    FlowArrows,
    /// The DEBUG page's time controls (`time-speed`).
    TimeControls,
    /// Put the open Input sub-tab's keys back to their defaults.
    ResetTab,
    /// Make the next new world a Survival one (true) or an Explorer one
    /// (`survival-mode` design 1).
    Mode(bool),
    /// The breeding screen's species tab, by roster index.
    BreedSpecies(u16),
    /// A kept animal of the herd pressed: sire or dam by its sex.
    BreedAnimal(u64),
    /// Pair the chosen sire and dam.
    BreedPair,
}

/// A panel the mouse wheel scrolls.
#[derive(Component)]
pub struct WheelScrolls;

/// The worlds list's scroll area (`save-slots` 9): the rows, under the
/// WORLDS lintel and over the NAME row.
#[derive(Component)]
pub struct WorldsScroll;

/// The stone groove at the worlds list's right edge: shown only while the
/// rows overflow, and pressed or dragged to scroll.
#[derive(Component)]
pub struct WorldsTrack;

/// The oak grip in the worlds groove, as far down it as the list is
/// scrolled.
#[derive(Component)]
pub struct WorldsKnob;

/// The tallest the worlds list stands before it scrolls, in native pixels:
/// ten rows, so NAME, NEW WORLD and BACK stay on a 450-pixel-high screen
/// (`ui_kit`'s smallest) under it.
const WORLDS_LIST_HEIGHT: f32 = 270.0;

/// The worlds grip's height, native pixels (`oak/slider-knob`, 9 x 13).
const WORLDS_KNOB_HEIGHT: f32 = 13.0;

/// How far a scroll area can scroll, in logical pixels, by last frame's
/// layout: `None` while it is not laid out (hidden by `Display::None`).
fn scroll_room(node: &ComputedNode) -> Option<f32> {
    (node.size.y > 0.0).then(|| {
        (node.content_size.y - node.size.y + node.scrollbar_size.y).max(0.0)
            * node.inverse_scale_factor
    })
}

/// Where the worlds grip's top sits down its groove, for a list scrolled
/// `at` of `room` logical pixels, in a groove `track` tall.
fn knob_top(at: f32, room: f32, track: f32) -> f32 {
    let t = if room > 0.0 {
        (at / room).clamp(0.0, 1.0)
    } else {
        0.0
    };
    t * (track - WORLDS_KNOB_HEIGHT).max(0.0)
}

/// The scroll a press `y` down the groove asks for (0 at the top, 1 at the
/// bottom): the grip's middle follows the cursor.
fn scroll_for(y: f32, room: f32, track: f32) -> f32 {
    let run = (track - WORLDS_KNOB_HEIGHT).max(1.0);
    ((y * track - WORLDS_KNOB_HEIGHT / 2.0) / run).clamp(0.0, 1.0) * room
}

/// The worlds list's groove and grip, which the wheel's system also keeps.
#[allow(clippy::type_complexity)] // Bevy system queries.
#[derive(SystemParam)]
pub struct WorldsBar<'w, 's> {
    tracks: Query<
        'w,
        's,
        (
            &'static Interaction,
            &'static RelativeCursorPosition,
            &'static ComputedNode,
            &'static mut Node,
        ),
        (With<WorldsTrack>, Without<WorldsKnob>),
    >,
    knobs: Query<'w, 's, &'static mut Node, (With<WorldsKnob>, Without<WorldsTrack>)>,
    index: Option<Res<'w, SaveIndex>>,
}

/// The wheel scrolls a panel that is taller than the screen, as far as its
/// content goes and no further. The worlds list (`save-slots` 9) also
/// answers its groove: a press or drag on it scrolls to the cursor, and the
/// groove shows only while the rows overflow, its grip as far down as the
/// list is scrolled.
#[allow(clippy::type_complexity)]
pub fn wheel_scroll(
    mut wheel: MessageReader<bevy::input::mouse::MouseWheel>,
    mut panels: Query<
        (
            &mut ScrollPosition,
            &InheritedVisibility,
            Option<&ComputedNode>,
            Has<WorldsScroll>,
        ),
        With<WheelScrolls>,
    >,
    mut bar: WorldsBar,
) {
    let dy: f32 = wheel
        .read()
        .map(|w| match w.unit {
            bevy::input::mouse::MouseScrollUnit::Line => w.y * 36.0,
            bevy::input::mouse::MouseScrollUnit::Pixel => w.y,
        })
        .sum();
    // A DELETE confirm stands in the list's place: its offset is left alone
    // under it, so CANCEL comes back to the row it was asked from.
    let asking = bar.index.as_ref().is_some_and(|i| i.asking.is_some());
    for (mut at, shown, computed, worlds) in &mut panels {
        let room = computed.and_then(scroll_room);
        // Clamped only as the wheel turns: last frame's layout is a frame
        // behind a list just rebuilt, and clamping to it every frame would
        // throw away the offset a rebuild is meant to keep.
        if dy != 0.0 && shown.get() && !(worlds && asking) {
            at.0.y = (at.0.y - dy).max(0.0);
            if let Some(room) = room {
                at.0.y = at.0.y.min(room);
            }
        }
        if !worlds {
            continue;
        }
        let room = room.unwrap_or(0.0);
        let overflows = room > 0.0 && !asking;
        for (interaction, cursor, track, mut node) in &mut bar.tracks {
            let display = if overflows {
                Display::Flex
            } else {
                Display::None
            };
            if node.display != display {
                node.display = display;
            }
            let tall = track.size.y * track.inverse_scale_factor;
            if overflows
                && *interaction == Interaction::Pressed
                && let Some(c) = cursor.normalized
            {
                // Normalised is centred on the node: -0.5 at the top.
                at.0.y = scroll_for(c.y + 0.5, room, tall);
            }
            let top = px(knob_top(at.0.y, room, tall));
            for mut knob in &mut bar.knobs {
                if knob.top != top {
                    knob.top = top;
                }
            }
        }
    }
}

/// The frame limit's steps along its slider, low to high: the plaque's own
/// values (`FrameLimit`), off first.
const FRAME_STEPS: [pbd_app::settings::FrameLimit; 3] = [
    pbd_app::settings::FrameLimit::Off,
    pbd_app::settings::FrameLimit::Fps60,
    pbd_app::settings::FrameLimit::Fps120,
];

/// The render scale's steps along its slider, low to high: `SCALES`.
fn scale_steps() -> Vec<u32> {
    let mut steps = pbd_app::settings::SCALES.to_vec();
    steps.sort_unstable();
    steps
}

/// The volume a `Volume` slider sets.
fn volume_of(volumes: &mut pbd_app::settings::Volumes, which: u8) -> &mut u32 {
    match which {
        0 => &mut volumes.master,
        1 => &mut volumes.effects,
        2 => &mut volumes.ambience,
        3 => &mut volumes.music,
        _ => &mut volumes.menus,
    }
}

/// How many steps a settings slider has, and which one the player is on.
fn slider_step(action: MenuAction, player: &PlayerSettings) -> Option<(usize, usize)> {
    Some(match action {
        MenuAction::Volume(which) => {
            let mut volumes = player.volumes;
            (11, (*volume_of(&mut volumes, which) / 10).min(10) as usize)
        }
        MenuAction::FrameLimit => (
            FRAME_STEPS.len(),
            FRAME_STEPS
                .iter()
                .position(|f| *f == player.frame_limit)
                .unwrap_or(0),
        ),
        MenuAction::Reflections => (
            Reflections::ALL.len(),
            Reflections::ALL
                .iter()
                .position(|r| *r == player.reflections)
                .unwrap_or(Reflections::ALL.len() - 1),
        ),
        MenuAction::Shadows => (
            ShadowQuality::ALL.len(),
            ShadowQuality::ALL
                .iter()
                .position(|r| *r == player.shadows)
                .unwrap_or(0),
        ),
        MenuAction::RenderScale => {
            let steps = scale_steps();
            (
                steps.len(),
                steps
                    .iter()
                    .position(|s| *s == player.render_scale)
                    .unwrap_or(steps.len() - 1),
            )
        }
        _ => return None,
    })
}

/// The settings with a slider set to step `step`.
fn with_step(action: MenuAction, player: PlayerSettings, step: usize) -> PlayerSettings {
    let mut changed = player;
    match action {
        MenuAction::FrameLimit => {
            changed.frame_limit = FRAME_STEPS[step.min(FRAME_STEPS.len() - 1)];
        }
        MenuAction::RenderScale => {
            let steps = scale_steps();
            changed.render_scale = steps[step.min(steps.len() - 1)];
        }
        MenuAction::Volume(which) => {
            *volume_of(&mut changed.volumes, which) = step.min(10) as u32 * 10;
        }
        MenuAction::Reflections => {
            changed.reflections = Reflections::ALL[step.min(Reflections::ALL.len() - 1)];
        }
        MenuAction::Shadows => {
            changed.shadows = ShadowQuality::ALL[step.min(ShadowQuality::ALL.len() - 1)];
        }
        _ => {}
    }
    changed
}

/// What a settings slider says beside it.
fn settings_value(action: MenuAction, player: &PlayerSettings) -> Option<Localized> {
    Some(match action {
        MenuAction::FrameLimit => player.frame_limit.label(),
        MenuAction::RenderScale => Localized::verbatim(format!("{}%", player.render_scale)),
        MenuAction::Reflections => player.reflections.label(),
        MenuAction::Shadows => player.shadows.label(),
        MenuAction::Volume(which) => {
            let mut volumes = player.volumes;
            Localized::verbatim(format!("{}%", volume_of(&mut volumes, which)))
        }
        _ => return None,
    })
}

/// The step nearest a cursor `x` across a slider of `count` steps (0 at the
/// left edge, 1 at the right).
fn nearest_step(x: f32, count: usize) -> usize {
    let last = count.saturating_sub(1) as f32;
    (x.clamp(0.0, 1.0) * last).round() as usize
}

/// A settings slider's groove, by what it sets.
#[derive(Component)]
pub struct SettingSlider(pub MenuAction);

/// The lit part of a settings slider.
#[derive(Component)]
pub struct SettingFill(pub MenuAction);

/// A settings slider's grip.
#[derive(Component)]
pub struct SettingKnob(pub MenuAction);

/// The value beside a settings slider.
#[derive(Component)]
pub struct SettingValue(pub MenuAction);

/// A press on a settings slider, and every frame it is held, sets the step
/// nearest the cursor, and keeps it.
pub fn drag_settings(
    bars: Query<(&Interaction, &RelativeCursorPosition, &SettingSlider)>,
    mut player: ResMut<PlayerSettings>,
    keys: Res<KeyBinds>,
    file: Res<SettingsFile>,
) {
    for (interaction, cursor, slider) in &bars {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let (Some(at), Some((count, _))) = (cursor.normalized, slider_step(slider.0, &player))
        else {
            continue;
        };
        // Normalised is centred on the node: -0.5 at the left edge.
        let changed = with_step(slider.0, *player, nearest_step(at.x + 0.5, count));
        if changed != *player {
            *player = changed;
            file.keep(&player, &keys);
        }
    }
}

/// Each settings slider's grip and lit groove at its step, the value beside.
#[allow(clippy::type_complexity)]
pub fn show_settings(
    player: Res<PlayerSettings>,
    mut fills: Query<(&SettingFill, &mut Node), Without<SettingKnob>>,
    mut knobs: Query<(&SettingKnob, &mut Node), Without<SettingFill>>,
    mut values: Query<(&SettingValue, &mut KitText)>,
) {
    let at = |action: MenuAction| {
        slider_step(action, &player).map(|(count, step)| {
            percent(step as f32 / count.saturating_sub(1).max(1) as f32 * 100.0)
        })
    };
    for (fill, mut node) in &mut fills {
        if let Some(at) = at(fill.0)
            && node.width != at
        {
            node.width = at;
        }
    }
    for (knob, mut node) in &mut knobs {
        if let Some(at) = at(knob.0)
            && node.left != at
        {
            node.left = at;
        }
    }
    for (value, mut text) in &mut values {
        if let Some(label) = settings_value(value.0, &player)
            && text.text != label
        {
            text.text = label;
        }
    }
}

/// Whether a checkbox on the settings page is ticked: bloom, the rain
/// shadow (off by default, CLAUDE.md "What the owner means by rain shadow")
/// and auto step.
fn settings_checked(
    action: MenuAction,
    graphics: &pbd_app::tone::Graphics,
    player: &PlayerSettings,
) -> Option<bool> {
    Some(match action {
        MenuAction::Bloom => graphics.bloom,
        MenuAction::RainShadows => player.rain_shadows,
        MenuAction::Antialiasing => player.antialiasing,
        MenuAction::MissionLine => player.mission_line,
        MenuAction::FlowArrows => player.flow_arrows,
        MenuAction::TimeControls => player.time_controls,
        MenuAction::AutoStep => player.auto_step,
        _ => return None,
    })
}

/// A press on GRAPHICS' and MOVEMENT's rows changes the player's choice and
/// keeps it.
pub fn press_graphics(
    rows: Query<(&Interaction, &MenuAction), Changed<Interaction>>,
    mut graphics: ResMut<pbd_app::tone::Graphics>,
    mut player: ResMut<PlayerSettings>,
    keys: Res<KeyBinds>,
    file: Res<SettingsFile>,
) {
    for (interaction, action) in &rows {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let mut changed = *player;
        match action {
            MenuAction::Bloom => {
                graphics.bloom = !graphics.bloom;
                graphics.keep();
            }
            MenuAction::AutoStep => changed.auto_step = !changed.auto_step,
            MenuAction::RainShadows => changed.rain_shadows = !changed.rain_shadows,
            MenuAction::Antialiasing => changed.antialiasing = !changed.antialiasing,
            MenuAction::MissionLine => changed.mission_line = !changed.mission_line,
            MenuAction::FlowArrows => changed.flow_arrows = !changed.flow_arrows,
            MenuAction::TimeControls => changed.time_controls = !changed.time_controls,
            _ => {}
        }
        if changed != *player {
            *player = changed;
            file.keep(&player, &keys);
        }
    }
}

/// The settings checkboxes show their ticks.
pub fn label_graphics(
    kit: Res<UiKit>,
    graphics: Res<pbd_app::tone::Graphics>,
    player: Res<PlayerSettings>,
    mut checks: Query<(&MenuAction, &mut ImageNode), With<Check>>,
) {
    for (action, mut image) in &mut checks {
        if let Some(on) = settings_checked(*action, &graphics, &player) {
            let want = kit.image(if on { "check-on" } else { "check-off" });
            if image.image != want {
                image.image = want;
            }
        }
    }
}

/// A checkbox on the settings page.
#[derive(Component)]
pub struct Check;

/// The settings page's tabs: Picture (with the rain shadow and the
/// reflections), Weather (the storm forcing), World (the clock and the globe overlay), Movement, Input.
/// Each is an id: its label is `menu.tab.<id>` in the locale file.
pub const SETTINGS_TABS: [&str; 8] = [
    "picture",
    "weather",
    "world",
    "movement",
    "input",
    "sound",
    "debug",
    "character",
];

/// A settings tab's label.
fn settings_tab_label(id: &str) -> Localized {
    match id {
        "picture" => tr("menu.tab.picture"),
        "weather" => tr("menu.tab.weather"),
        "world" => tr("menu.tab.world"),
        "movement" => tr("menu.tab.movement"),
        "input" => tr("menu.tab.input"),
        "debug" => tr("menu.tab.debug"),
        "character" => tr("menu.tab.character"),
        _ => tr("menu.tab.sound"),
    }
}

/// Which settings tab is open.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct SettingsTab(pub u8);

/// A settings tab's own oak tab, by index.
#[derive(Component)]
pub struct SettingsTabButton(pub u8);

/// The body of a settings tab, by index.
#[derive(Component)]
pub struct SettingsPage(pub u8);

/// A press on a tab opens its page; the open tab wears the oak cap, the shut
/// ones the plain stone (design 3, accent 2), and only the open page is laid
/// out.
#[allow(clippy::type_complexity)]
pub fn settings_tabs(
    kit: Res<UiKit>,
    mut tab: ResMut<SettingsTab>,
    pressed: Query<(&Interaction, &SettingsTabButton), Changed<Interaction>>,
    mut tabs: Query<(&SettingsTabButton, &mut ImageNode, &mut KitPiece, &Children)>,
    mut labels: Query<&mut KitText>,
    mut pages: Query<(&SettingsPage, &mut Node)>,
) {
    for (interaction, button) in &pressed {
        if *interaction == Interaction::Pressed && tab.0 != button.0 {
            tab.0 = button.0;
        }
    }
    if !tab.is_changed() {
        return;
    }
    for (button, mut image, mut piece, children) in &mut tabs {
        let open = button.0 == tab.0;
        let key = Set::Oak.key(if open { "tab-open" } else { "tab-shut" });
        if piece.0 != key {
            image.image = kit.image(&key);
            piece.0 = key;
        }
        for child in children.iter() {
            if let Ok(mut text) = labels.get_mut(child) {
                text.colour = if open {
                    ui_kit::OAK_LABEL
                } else {
                    ui_kit::LABEL
                };
            }
        }
    }
    for (page, mut node) in &mut pages {
        node.display = if page.0 == tab.0 {
            Display::Flex
        } else {
            Display::None
        };
    }
}

/// Whether the saves page is the screen the game opened on and no world has
/// been chosen from it yet. Cleared by CONTINUE, LOAD and NEW WORLD, so a
/// later visit through the pause menu steps back to the pause menu.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct FrontDoor(pub bool);

/// The screen a launch opens on: what `--menu` names, else the title screen
/// for a plain desktop launch (`title-screen`), else the world. A launch that names its world, or
/// takes a capture, does not want a menu in front of it.
pub fn opening_screen(menu: Option<&str>, world_named: bool, capture: bool) -> Screen {
    match menu {
        Some("title") => Screen::Title,
        Some("pause") => Screen::Pause,
        Some("settings") => Screen::Settings,
        Some("saves") => Screen::Saves,
        Some("multiplayer") => Screen::Multiplayer,
        Some("pack") => Screen::Pack,
        Some("map") => Screen::Map,
        Some("guide") => Screen::Guide,
        Some("missions") => Screen::Missions,
        Some("breeding") => Screen::Breeding,
        _ if !world_named && !capture => Screen::Title,
        _ => Screen::Playing,
    }
}

/// The longest name a world can be given. `saves::slot_id` derives a legal
/// directory from anything; this only keeps the row readable.
pub const NAME_MAX: usize = 32;

/// The name field on the saves page: what has been typed, whether the
/// keyboard is in it, and whether Enter asked for the world to be made.
#[derive(Resource, Default, Clone, PartialEq, Eq, Debug)]
pub struct NameField {
    pub text: String,
    pub focused: bool,
    /// The field was typed in, so the page's prefill leaves it alone.
    pub edited: bool,
    /// Enter was pressed in the field; `press` makes the world.
    pub submit: bool,
    /// The mode the next new world is made in (`survival-mode` design 1):
    /// Explorer unless Survival is picked.
    pub survival: bool,
}

impl NameField {
    /// One typed key. Characters and spaces go in up to `NAME_MAX`, Backspace
    /// takes one off, and anything else is not text.
    pub fn typed(&mut self, key: &Key) {
        match key {
            Key::Character(text) => {
                for c in text.chars().filter(|c| !c.is_control()) {
                    if self.text.chars().count() < NAME_MAX {
                        self.text.push(c);
                        self.edited = true;
                    }
                }
            }
            Key::Space => {
                if self.text.chars().count() < NAME_MAX {
                    self.text.push(' ');
                    self.edited = true;
                }
            }
            Key::Backspace => {
                self.text.pop();
                self.edited = true;
            }
            _ => {}
        }
    }

    /// What the page shows in the field: the text, and a caret while the
    /// keyboard is in it.
    pub fn shown(&self) -> String {
        if self.focused {
            format!("{}_", self.text)
        } else {
            self.text.clone()
        }
    }

    /// The name a world would be made with: what was typed, or the number
    /// the page offered.
    pub fn name(&self, worlds: usize) -> String {
        let typed = self.text.trim();
        if typed.is_empty() {
            tr_with("menu.world_name", &[("number", &(worlds + 1))]).into()
        } else {
            typed.to_string()
        }
    }

    /// Offer the next number unless the player has typed something.
    pub fn prefill(&mut self, worlds: usize) {
        if !self.edited {
            self.text = tr_with("menu.world_name", &[("number", &(worlds + 1))]).into();
        }
    }
}

/// The name field's text node.
#[derive(Component)]
pub struct NameText;

/// The slots as the screen last listed them, and which one is being asked
/// about.
///
/// The list is a RESOURCE rather than read off the disk while drawing, because
/// a row's button carries an index into it: reading the directory twice, once
/// to draw and once to act, is two lists that can disagree about what row
/// three is.
#[derive(Resource, Default)]
pub struct SaveIndex {
    pub slots: Vec<Slot>,
    pub asking: Option<usize>,
    /// Why the last load or delete did not happen.
    pub trouble: Option<String>,
}

/// A load the next frame will carry out, since it touches more of the world
/// than a button press can borrow at once.
#[derive(Resource, Default)]
pub struct LoadRequest(pub Option<Slot>);

/// The saves page's own state, which a press reads and writes together: the
/// list and what is asked about it, the load it asked for, whether it is the
/// front door, and the name field.
#[derive(SystemParam)]
pub struct SavesPage<'w> {
    pub index: ResMut<'w, SaveIndex>,
    pub load: ResMut<'w, LoadRequest>,
    pub front: ResMut<'w, FrontDoor>,
    pub name: ResMut<'w, NameField>,
}

/// The part of the saves panel that is rebuilt when the list changes: the
/// rows, in the worlds scroll area.
#[derive(Component)]
pub struct SaveList;

/// The fixed part under the worlds list, rebuilt with it: why the last load
/// or delete did not happen, the name field and NEW WORLD.
#[derive(Component)]
pub struct SaveFoot;

/// Marks a panel with the screen it belongs to.
#[derive(Component)]
pub struct Panel(pub(super) Screen);

/// The full-screen layer a panel is centred in.
///
/// Centring by flex rather than by a margin of half the panel's own height,
/// because the height is the CONTENT's: the settings page grows by a row every
/// time a binding is added, and a magic offset would be wrong the day it does.
/// The first cut used `top: 50%` and the input list ran off the bottom of the
/// window.
pub(crate) fn layer() -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(0),
        top: px(0),
        width: percent(100.0),
        height: percent(100.0),
        align_items: AlignItems::Center,
        justify_content: JustifyContent::Center,
        display: Display::None,
        ..default()
    }
}

/// The title screen (`title-screen`): the game's logo over the open world,
/// and under it the way in.
fn spawn_title(commands: &mut Commands, kit: &UiKit, images: &mut Assets<Image>) {
    let logo = {
        let path = pbd_app::paths::assets_dir().join("ui/logo.png");
        let pixels = image::open(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let mut img = Image::from_dynamic(pixels, true, bevy::asset::RenderAssetUsages::default());
        img.sampler = bevy::image::ImageSampler::linear();
        let aspect = img.width() as f32 / img.height().max(1) as f32;
        (images.add(img), aspect)
    };
    // The splash: deep space under the logo, opaque, so the title never
    // shows the world behind it (`title-screen`, "The splash").
    let splash = {
        let path = pbd_app::paths::assets_dir().join("ui/splash-space.png");
        let pixels = image::open(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let mut img = Image::from_dynamic(pixels, true, bevy::asset::RenderAssetUsages::default());
        img.sampler = bevy::image::ImageSampler::linear();
        images.add(img)
    };
    let logo_w = 300.0;
    // Under every front-door page (the title, its saves page and settings),
    // over the world and the HUD: `show_splash` shows it.
    commands.spawn((
        SplashBackdrop,
        ImageNode::new(splash),
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: percent(100.0),
            height: percent(100.0),
            ..default()
        },
        BackgroundColor(Color::BLACK),
        GlobalZIndex(9),
        Visibility::Hidden,
    ));
    commands
        .spawn(timber_layer(Screen::Title))
        .insert(BackgroundColor(Color::NONE))
        .with_children(|layer| {
            layer
                // The logo near the top of the screen, the way in centred in
                // the space under it (owner, 2026-11-17: "this logo should be
                // closer to top of the screen").
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    height: percent(100),
                    padding: UiRect::top(px(10)),
                    ..default()
                })
                .with_children(|column| {
                    column.spawn((
                        ImageNode::new(logo.0),
                        Node {
                            width: px(logo_w),
                            height: px(logo_w / logo.1),
                            flex_shrink: 0.0,
                            ..default()
                        },
                    ));
                    // The way in right under the logo (owner: "move the menu
                    // text up too").
                    column
                        .spawn(Node {
                            margin: UiRect::top(px(8)),
                            ..default()
                        })
                        .with_children(|gap| {
                            gap.spawn(framed(kit, Set::Oak, 120.0))
                                .with_children(|panel| {
                                    // `multiplayer` design 28: Continue is the
                                    // worlds, the latest on top.
                                    let go =
                                        button(panel, kit, tr("menu.continue"), MenuAction::Saves);
                                    panel.commands().entity(go).insert(Plaque {
                                        focus: true,
                                        ..Plaque::new(Set::Oak)
                                    });
                                    button(panel, kit, tr("menu.new_world"), MenuAction::NewWorld);
                                    button(
                                        panel,
                                        kit,
                                        tr("menu.multiplayer"),
                                        MenuAction::Multiplayer,
                                    );
                                    button(panel, kit, tr("menu.options"), MenuAction::Settings);
                                    button(panel, kit, tr("menu.quit"), MenuAction::Quit);
                                });
                        });
                });
        });
}

/// The in-play HUD the title screen hides: the hotbar, the hand and the
/// crosshair (owner, 2026-11-17: "why would you have hot bar in title
/// screen?").
#[derive(Component)]
pub struct InPlayHud;

/// The splash behind the front door's pages (`title-screen`, "The splash").
#[derive(Component)]
pub struct SplashBackdrop;

/// Whether the screen is one of the front door's: the title, or its saves
/// page or settings while the title is the front door.
fn on_front_door(screen: Screen, front: FrontDoor) -> bool {
    screen == Screen::Title
        || (front.0
            && matches!(
                screen,
                Screen::Saves | Screen::Settings | Screen::Multiplayer
            ))
}

/// Show the splash on the front door's pages and nowhere else.
pub fn show_splash(
    screen: Res<Screen>,
    front: Res<FrontDoor>,
    mut splash: Query<&mut Visibility, With<SplashBackdrop>>,
) {
    let want = if on_front_door(*screen, *front) {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut visibility in &mut splash {
        if *visibility != want {
            *visibility = want;
        }
    }
}

/// Hide the in-play HUD while the front door is up.
pub fn hide_hud_on_title(
    screen: Res<Screen>,
    front: Res<FrontDoor>,
    mut hud: Query<&mut Visibility, With<InPlayHud>>,
) {
    let want = if on_front_door(*screen, *front) {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    };
    for mut visibility in &mut hud {
        if *visibility != want {
            *visibility = want;
        }
    }
}

/// An out-of-play menu's layer: the world dimmed behind the timber.
pub(super) fn timber_layer(screen: Screen) -> impl Bundle {
    (
        layer(),
        BackgroundColor(ui_kit::DIMMER),
        GlobalZIndex(10),
        Panel(screen),
    )
}

/// A panel's frame and layout: `width` native pixels, the frame's 14 px
/// margin as its padding, children in a column. Oak out of play, the stone
/// moulding (`frame`) for an in-play panel with no baked rock of its size.
pub(super) fn framed(kit: &UiKit, set: Set, width: f32) -> impl Bundle {
    (
        kit.nine(set, "frame"),
        KitPiece(set.key("frame")),
        Node {
            width: px(width),
            padding: UiRect::all(px(14)),
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            ..default()
        },
    )
}

/// The carved oak lintel over a panel, its title cut into it (design 3,
/// accent 1): pegged on the frame's top edge. With it, the panel's X.
pub(super) fn lintel(parent: &mut ChildSpawnerCommands, kit: &UiKit, text: Localized) {
    lintel_in(parent, kit, text, Set::Stone);
}

/// [`lintel`] on a timber menu (pause, settings, worlds): its X is oak, as
/// every plaque out of play is.
pub(super) fn timber_lintel(parent: &mut ChildSpawnerCommands, kit: &UiKit, text: Localized) {
    lintel_in(parent, kit, text, Set::Oak);
}

fn lintel_in(parent: &mut ChildSpawnerCommands, kit: &UiKit, text: Localized, set: Set) {
    close_x(parent, kit, set);
    parent
        .spawn(Node {
            justify_content: JustifyContent::Center,
            margin: UiRect::new(px(0), px(0), px(-15), px(4)),
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                kit.nine(Set::Stone, "lintel"),
                KitPiece("lintel".into()),
                Node {
                    min_height: px(16),
                    min_width: px(40),
                    padding: UiRect::new(px(11), px(11), px(5), px(4)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
            ))
            .with_children(|board| {
                board.spawn(KitText::new(text, ui_kit::OAK_LABEL));
            });
        });
}

/// The little X in a stone panel's top-right corner (`stone-and-wood-ui`,
/// "an X on every stone panel"): Esc's step back, pressed. Placed on the
/// lintel's parent, so every panel with a lintel has one.
fn close_x(parent: &mut ChildSpawnerCommands, kit: &UiKit, set: Set) {
    parent
        .spawn((plaque(kit, set, 18.0), MenuAction::Back))
        .insert(Node {
            position_type: PositionType::Absolute,
            top: px(-10),
            right: px(-10),
            min_width: px(18),
            height: px(22),
            padding: UiRect::new(px(6), px(6), px(8), px(0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            ..default()
        })
        .with_children(|p| {
            p.spawn(plaque_label(set, tr("menu.close")));
        });
}

/// A section label inside a panel, carved in the kit's font.
pub(super) fn section(parent: &mut ChildSpawnerCommands, text: Localized) {
    parent.spawn((
        KitText::new(text, ui_kit::TITLE).shadow(ui_kit::INK),
        Node {
            margin: UiRect::top(px(3)),
            ..default()
        },
    ));
}

/// A line of the body face (VT323) on a stone face.
pub(super) fn body(text: impl Into<String>, size: f32, colour: Color) -> impl Bundle {
    (
        Text::new(text.into()),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(colour),
    )
}

/// A row that can be clicked: an oak plaque the panel's width. Every one is a
/// real `Button` with a hit box and hover and press states: a control that
/// only responds to a key is a control nobody can find, which is why the
/// bindings were on screen in the first place.
pub(super) fn button(
    parent: &mut ChildSpawnerCommands,
    kit: &UiKit,
    label: Localized,
    action: MenuAction,
) -> Entity {
    parent
        .spawn((plaque(kit, Set::Oak, 48.0), action))
        .insert(Node {
            width: percent(100.0),
            ..plaque_node()
        })
        .with_children(|row| {
            row.spawn(plaque_label(Set::Oak, label));
        })
        .id()
}

/// A plaque's own node, as `ui_kit::plaque` lays it out.
fn plaque_node() -> Node {
    Node {
        min_width: px(30),
        height: px(22),
        padding: UiRect::new(px(10), px(10), px(8), px(0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::FlexStart,
        flex_shrink: 0.0,
        ..default()
    }
}

pub fn spawn(mut commands: Commands, kit: Res<UiKit>, mut images: ResMut<Assets<Image>>) {
    let kit = &*kit;
    spawn_saves(&mut commands, kit);
    spawn_title(&mut commands, kit, &mut images);
    super::join_ui::spawn(&mut commands, kit);
    commands
        // Above the slot row, which is spawned a schedule later and would
        // otherwise draw over the panel: within a UI, later is on top.
        .spawn(timber_layer(Screen::Pause))
        .with_children(|layer| {
            layer
                // The owner's menu and nothing else (`menus-and-a-quiet-screen`:
                // "Pause menu should have resume, settings, and quit", and the
                // saves): the weather, the clock and the globe overlay moved to
                // their own tabs in Settings, so the panel fits every window.
                .spawn(framed(kit, Set::Oak, 120.0))
                .with_children(|panel| {
                    timber_lintel(panel, kit, tr("menu.paused"));
                    let resume = button(panel, kit, tr("menu.resume"), MenuAction::Resume);
                    panel.commands().entity(resume).insert(Plaque {
                        focus: true,
                        ..Plaque::new(Set::Oak)
                    });
                    button(panel, kit, tr("menu.saves"), MenuAction::Saves);
                    button(panel, kit, tr("menu.settings"), MenuAction::Settings);
                    button(panel, kit, tr("menu.main_menu"), MenuAction::MainMenu);
                    button(panel, kit, tr("menu.quit"), MenuAction::Quit);
                });
        });

    commands
        .spawn(timber_layer(Screen::Settings))
        .with_children(|layer| {
            layer
                // 420 px: eight tabs across (CHARACTER, `player-character`).
                .spawn(framed(kit, Set::Oak, 420.0))
                .with_children(|panel| {
                    timber_lintel(panel, kit, tr("menu.settings"));
                    // The tabs, oak, the open one capped (design 8).
                    panel
                        .spawn(Node {
                            column_gap: px(1),
                            margin: UiRect::bottom(px(3)),
                            ..default()
                        })
                        .with_children(|tabs| {
                            for (i, name) in SETTINGS_TABS.iter().enumerate() {
                                let open = i == 0;
                                let piece = if open { "tab-open" } else { "tab-shut" };
                                tabs.spawn((
                                    Button,
                                    kit.nine(Set::Oak, piece),
                                    KitPiece(Set::Oak.key(piece)),
                                    SettingsTabButton(i as u8),
                                    Node {
                                        height: px(18),
                                        // Six tabs fit the panel at 5 px a side (the SOUND tab,
                                        // `audio-and-ambience` 7.2).
                                        padding: UiRect::new(px(5), px(5), px(7), px(0)),
                                        flex_shrink: 0.0,
                                        ..default()
                                    },
                                ))
                                .with_children(|tab| {
                                    tab.spawn(KitText::new(
                                        settings_tab_label(name),
                                        if open {
                                            ui_kit::OAK_LABEL
                                        } else {
                                            ui_kit::LABEL
                                        },
                                    ));
                                });
                            }
                        });
                    let page = |panel: &mut ChildSpawnerCommands, i: u8| {
                        panel
                            .spawn((
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(2),
                                    min_height: px(176),
                                    display: if i == 0 { Display::Flex } else { Display::None },
                                    ..default()
                                },
                                SettingsPage(i),
                            ))
                            .id()
                    };
                    let picture = page(panel, 0);
                    panel.commands().entity(picture).with_children(|rows| {
                        // `player-settings`: each control shows what is chosen,
                        // kept up by `label_graphics`.
                        check_row(rows, kit, tr("menu.bloom"), MenuAction::Bloom);
                        slider_row(rows, kit, tr("menu.frame_limit"), MenuAction::FrameLimit);
                        slider_row(rows, kit, tr("menu.render_scale"), MenuAction::RenderScale);
                        slider_row(rows, kit, tr("menu.reflections"), MenuAction::Reflections);
                        slider_row(rows, kit, tr("menu.shadow_quality"), MenuAction::Shadows);
                        check_row(rows, kit, tr("menu.antialiasing"), MenuAction::Antialiasing);
                        // On PICTURE beside Reflections (owner, 2026-11-16:
                        // "rain shadow should be under picture").
                        check_row(rows, kit, tr("menu.rain_shadow"), MenuAction::RainShadows);
                        check_row(rows, kit, tr("menu.flow_arrows"), MenuAction::FlowArrows);
                    });
                    let weather = page(panel, 1);
                    panel.commands().entity(weather).with_children(|rows| {
                        super::weather_ui::spawn(rows, kit);
                    });
                    let world = page(panel, 2);
                    panel.commands().entity(world).with_children(|rows| {
                        super::time_ui::spawn(rows, kit);
                        super::overlay_ui::spawn_row(rows, kit);
                        check_row(rows, kit, tr("menu.mission_line"), MenuAction::MissionLine);
                    });
                    let movement = page(panel, 3);
                    panel.commands().entity(movement).with_children(|rows| {
                        check_row(rows, kit, tr("menu.auto_step"), MenuAction::AutoStep);
                    });
                    let input = page(panel, 4);
                    panel.commands().entity(input).with_children(|page| {
                        spawn_input_page(page, kit);
                    });
                    // `audio-and-ambience` 7.2: the five volumes, heard at
                    // once and kept as every setting is.
                    let sound = page(panel, 5);
                    panel.commands().entity(sound).with_children(|rows| {
                        let labels = [
                            tr("menu.sound.master"),
                            tr("menu.sound.effects"),
                            tr("menu.sound.ambience"),
                            tr("menu.sound.music"),
                            tr("menu.sound.menus"),
                        ];
                        for (which, label) in labels.into_iter().enumerate() {
                            slider_row(rows, kit, label, MenuAction::Volume(which as u8));
                        }
                    });
                    // DEBUG (`time-speed`): tools for testing, each off by
                    // default; first the clock's speed and jumps.
                    let debug = page(panel, 6);
                    panel.commands().entity(debug).with_children(|rows| {
                        check_row(
                            rows,
                            kit,
                            tr("menu.time_controls"),
                            MenuAction::TimeControls,
                        );
                        super::time_ui::spawn_speed(rows, kit);
                    });
                    // CHARACTER (`player-character` design 7).
                    let character = page(panel, 7);
                    panel.commands().entity(character).with_children(|rows| {
                        super::character_ui::spawn(rows, kit);
                    });
                    panel
                        .spawn(Node {
                            justify_content: JustifyContent::FlexEnd,
                            margin: UiRect::top(px(4)),
                            ..default()
                        })
                        .with_children(|foot| {
                            small(foot, kit, tr("menu.back"), MenuAction::Back);
                        });
                });
        });
}

/// A slider in the timber menus: the stone groove, the rune in it as far as
/// the value, and the oak grip (design 8). `track` is the clickable groove
/// (it carries `RelativeCursorPosition`), `fill` and `knob` are moved by the
/// caller's `show`.
pub(super) fn slider(
    parent: &mut ChildSpawnerCommands,
    kit: &UiKit,
    width: f32,
    track: impl Bundle,
    fill: impl Bundle,
    knob: impl Bundle,
) {
    parent
        .spawn((
            kit.nine(Set::Stone, "slider-track"),
            KitPiece("slider-track".into()),
            Node {
                width: px(width),
                height: px(8),
                padding: UiRect::all(px(3)),
                margin: UiRect::vertical(px(3)),
                flex_shrink: 0.0,
                ..default()
            },
            track,
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    width: percent(0),
                    height: percent(100),
                    ..default()
                },
                BackgroundColor(ui_kit::RUNE_DIM),
                fill,
            ));
            bar.spawn((
                kit.whole(Set::Oak, "slider-knob"),
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(0),
                    top: px(-2),
                    width: px(9),
                    height: px(13),
                    margin: UiRect::left(px(-4)),
                    ..default()
                },
                knob,
            ));
        });
}

/// A settings row: what it is, a dim note, and its control on the right.
fn setting_row(
    rows: &mut ChildSpawnerCommands,
    label: Localized,
    control: impl FnOnce(&mut ChildSpawnerCommands),
) {
    rows.spawn(Node {
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        min_height: px(22),
        column_gap: px(4),
        ..default()
    })
    .with_children(|row| {
        row.spawn(Node {
            column_gap: px(4),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|words| {
            words.spawn(body(label, 10.0, ui_kit::TEXT));
        });
        control(row);
    });
}

/// A row with a checkbox: the stone well, the tick a glowing rune.
fn check_row(rows: &mut ChildSpawnerCommands, kit: &UiKit, label: Localized, action: MenuAction) {
    setting_row(rows, label, |row| {
        row.spawn((
            Button,
            kit.whole(Set::Stone, "check-off"),
            Node {
                width: px(12),
                height: px(12),
                flex_shrink: 0.0,
                ..default()
            },
            Check,
            action,
        ));
    });
}

/// The width of a settings slider's groove, native px.
const SETTING_SLIDER: f32 = 70.0;

/// Between a settings slider's groove and its value, native px: the grip's
/// overhang past the groove's end (half its 9 px, rounded up) and 4 clear.
const SLIDER_VALUE_GAP: f32 = 9.0;

/// A row whose stone slider steps through values, the value beside it.
fn slider_row(rows: &mut ChildSpawnerCommands, kit: &UiKit, label: Localized, action: MenuAction) {
    setting_row(rows, label, |row| {
        row.spawn(Node {
            // The grip (9 px, centred on its step) stands 5 px past the
            // groove's end at the top value: the gap keeps 4 px clear of the
            // value beside it (the coordinator's review of the first
            // capture, where the grip covered "100%").
            column_gap: px(SLIDER_VALUE_GAP),
            align_items: AlignItems::Center,
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|control| {
            slider(
                control,
                kit,
                SETTING_SLIDER,
                (
                    SettingSlider(action),
                    Button,
                    RelativeCursorPosition::default(),
                ),
                SettingFill(action),
                SettingKnob(action),
            );
            control.spawn((
                KitText::new(Localized::empty(), ui_kit::LABEL).shadow(ui_kit::INK),
                Node {
                    min_width: px(42),
                    ..default()
                },
                SettingValue(action),
            ));
        });
    });
}

/// Which Input sub-tab is open, by `Tab::ALL` index.
#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct InputTab(pub u8);

/// The slot a cap is waiting on a key for, by name (`controls::Slot::name`).
#[derive(Resource, Default, Clone, PartialEq, Eq, Debug)]
pub struct Rebinding(pub Option<String>);

/// An Input sub-tab's own oak tab.
#[derive(Component)]
pub struct InputTabButton(pub u8);

/// An Input sub-tab's list of rows, rebuilt when a key changes.
#[derive(Component)]
pub struct InputList(pub u8);

/// A key cap that changes its slot's key when pressed.
#[derive(Component, Clone)]
pub struct RebindCap(pub String);

/// The Input page: the sub-tabs, a list for each (one shown, the wheel
/// scrolls it), and RESET TAB. The rows are drawn by `rebuild_input`.
fn spawn_input_page(page: &mut ChildSpawnerCommands, kit: &UiKit) {
    page.spawn(Node {
        column_gap: px(1),
        margin: UiRect::bottom(px(2)),
        ..default()
    })
    .with_children(|tabs| {
        for (i, tab) in Tab::ALL.iter().enumerate() {
            let open = i == 0;
            let piece = if open { "tab-open" } else { "tab-shut" };
            tabs.spawn((
                Button,
                kit.nine(Set::Oak, piece),
                KitPiece(Set::Oak.key(piece)),
                InputTabButton(i as u8),
                Node {
                    height: px(18),
                    padding: UiRect::new(px(8), px(8), px(7), px(0)),
                    flex_shrink: 0.0,
                    ..default()
                },
            ))
            .with_children(|cap| {
                cap.spawn(KitText::new(
                    tab.label(),
                    if open {
                        ui_kit::OAK_LABEL
                    } else {
                        ui_kit::LABEL
                    },
                ));
            });
        }
    });
    for i in 0..Tab::ALL.len() {
        page.spawn((
            Node {
                height: px(150),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::scroll_y(),
                row_gap: px(1),
                display: if i == 0 { Display::Flex } else { Display::None },
                ..default()
            },
            ScrollPosition::default(),
            WheelScrolls,
            InputList(i as u8),
        ));
    }
    page.spawn(Node {
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        margin: UiRect::top(px(2)),
        ..default()
    })
    .with_children(|foot| {
        foot.spawn(body(
            tr("menu.click_key_then_press_new"),
            9.0,
            ui_kit::TEXT_DIM,
        ));
        small(foot, kit, tr("menu.reset_tab"), MenuAction::ResetTab);
    });
}

/// Draw each sub-tab's rows off the one binding table, with the player's
/// keys: a heading per group, a row per action, a cap per key (an oak
/// plaque where it can be changed, a flat cap where it cannot), and on a
/// row whose key another live action reads, "also:" and that action.
/// Rebuilt when the keys or the waiting cap change.
pub fn rebuild_input(
    mut commands: Commands,
    kit: Res<UiKit>,
    binds: Res<KeyBinds>,
    waiting: Res<Rebinding>,
    lists: Query<(Entity, &InputList)>,
) {
    if !binds.is_changed() && !waiting.is_changed() {
        return;
    }
    let kit = &*kit;
    for (entity, list) in &lists {
        let tab = Tab::ALL[list.0 as usize];
        commands.entity(entity).despawn_related::<Children>();
        commands.entity(entity).with_children(|rows| {
            for group in BINDINGS.iter().filter(|g| g.tab == tab) {
                section(rows, group.heading());
                for binding in group.rows {
                    let clashes: Vec<Localized> = (0..binding.keys.len())
                        .filter_map(|i| controls::slot_at(group, binding, i))
                        .flat_map(|slot| binds.clashes(&slot))
                        .fold(Vec::new(), |mut all, does| {
                            if !all.contains(&does) {
                                all.push(does);
                            }
                            all
                        });
                    rows.spawn(Node {
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        min_height: px(20),
                        column_gap: px(4),
                        flex_shrink: 0.0,
                        ..default()
                    })
                    .with_children(|line| {
                        line.spawn((
                            body(binding.does(), 10.0, ui_kit::TEXT),
                            Node {
                                flex_shrink: 1.0,
                                ..default()
                            },
                        ));
                        line.spawn(Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::FlexEnd,
                            flex_shrink: 0.0,
                            ..default()
                        })
                        .with_children(|right| {
                            right
                                .spawn(Node {
                                    column_gap: px(1),
                                    align_items: AlignItems::Center,
                                    ..default()
                                })
                                .with_children(|caps| {
                                    for (i, key) in binding.keys.iter().enumerate() {
                                        if i > 0 && binding.range {
                                            caps.spawn(body("-", 10.0, ui_kit::TEXT_DIM));
                                        } else if i > 0 && binding.chord {
                                            caps.spawn(body("+", 10.0, ui_kit::TEXT_DIM));
                                        }
                                        match controls::slot_at(group, binding, i) {
                                            Some(slot) => {
                                                let name = slot.name();
                                                let label = if waiting.0.as_ref() == Some(&name) {
                                                    tr("menu.input.press_a_key")
                                                } else {
                                                    controls::key_label(binds.key(&slot))
                                                };
                                                rebind_cap(caps, kit, label, name, &waiting);
                                            }
                                            None => key_cap(caps, kit, key.label()),
                                        }
                                    }
                                });
                            if !clashes.is_empty() {
                                right.spawn(body(
                                    tr_with(
                                        "menu.input.also",
                                        &[(
                                            "actions",
                                            &clashes
                                                .iter()
                                                .map(Localized::as_str)
                                                .collect::<Vec<_>>()
                                                .join(", "),
                                        )],
                                    ),
                                    8.0,
                                    ui_kit::TITLE,
                                ));
                            }
                        });
                    });
                }
            }
        });
    }
}

/// A key cap that can be changed: a small oak plaque, focused with the rune
/// while it waits for a key.
fn rebind_cap(
    parent: &mut ChildSpawnerCommands,
    kit: &UiKit,
    label: Localized,
    slot: String,
    waiting: &Rebinding,
) {
    let focus = waiting.0.as_ref() == Some(&slot);
    parent
        .spawn((plaque(kit, Set::Oak, 18.0), RebindCap(slot)))
        .insert((
            Plaque {
                focus,
                ..Plaque::new(Set::Oak)
            },
            Node {
                min_width: px(18),
                height: px(18),
                padding: UiRect::new(px(6), px(6), px(6), px(0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::FlexStart,
                flex_shrink: 0.0,
                ..default()
            },
        ))
        .with_children(|cap| {
            cap.spawn(plaque_label(Set::Oak, label));
        });
}

/// The Input sub-tabs: a press opens one (and stops any cap waiting), the
/// open one wears the oak cap, and only its list is laid out.
#[allow(clippy::type_complexity)]
pub fn input_tabs(
    kit: Res<UiKit>,
    mut tab: ResMut<InputTab>,
    mut waiting: ResMut<Rebinding>,
    pressed: Query<(&Interaction, &InputTabButton), Changed<Interaction>>,
    mut tabs: Query<(&InputTabButton, &mut ImageNode, &mut KitPiece, &Children)>,
    mut labels: Query<&mut KitText>,
    mut lists: Query<(&InputList, &mut Node)>,
) {
    for (interaction, button) in &pressed {
        if *interaction == Interaction::Pressed && tab.0 != button.0 {
            tab.0 = button.0;
            if waiting.0.is_some() {
                waiting.0 = None;
            }
        }
    }
    if !tab.is_changed() {
        return;
    }
    for (button, mut image, mut piece, children) in &mut tabs {
        let open = button.0 == tab.0;
        let key = Set::Oak.key(if open { "tab-open" } else { "tab-shut" });
        if piece.0 != key {
            image.image = kit.image(&key);
            piece.0 = key;
        }
        for child in children.iter() {
            if let Ok(mut text) = labels.get_mut(child) {
                text.colour = if open {
                    ui_kit::OAK_LABEL
                } else {
                    ui_kit::LABEL
                };
            }
        }
    }
    for (list, mut node) in &mut lists {
        node.display = if list.0 == tab.0 {
            Display::Flex
        } else {
            Display::None
        };
    }
}

/// A press on a cap makes it wait for a key (a second press, or a press on
/// another, moves or stops the wait); RESET TAB puts the open sub-tab's keys
/// back and keeps the file.
#[allow(clippy::too_many_arguments)]
pub fn press_bindings(
    caps: Query<(&Interaction, &RebindCap), Changed<Interaction>>,
    resets: Query<(&Interaction, &MenuAction), Changed<Interaction>>,
    tab: Res<InputTab>,
    mut waiting: ResMut<Rebinding>,
    mut binds: ResMut<KeyBinds>,
    player: Res<PlayerSettings>,
    file: Res<SettingsFile>,
) {
    for (interaction, cap) in &caps {
        if *interaction == Interaction::Pressed {
            waiting.0 = if waiting.0.as_ref() == Some(&cap.0) {
                None
            } else {
                Some(cap.0.clone())
            };
        }
    }
    for (interaction, action) in &resets {
        if *interaction == Interaction::Pressed && *action == MenuAction::ResetTab {
            let before = binds.clone();
            binds.reset(Tab::ALL[tab.0 as usize]);
            waiting.0 = None;
            if *binds != before {
                file.keep(&player, &binds);
            }
        }
    }
}

/// The key a waiting cap takes: the first key pressed this frame that a
/// slot can be given, in `controls::ASSIGNABLE`'s order (one order, whatever
/// the keyboard's), or none.
pub fn key_taken(keys: &ButtonInput<KeyCode>) -> Option<KeyCode> {
    controls::ASSIGNABLE
        .iter()
        .copied()
        .find(|key| keys.just_pressed(*key))
}

/// While a cap waits: Esc cancels (and is consumed, so the page stays), a
/// key a slot can take is taken, kept, and consumed, so nothing else acts on
/// it; anything else is waited through. Leaving the settings stops the wait.
/// Runs in `PreUpdate` before `toggle`, with the remap off (a menu is open).
pub fn capture_rebind(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    screen: Res<Screen>,
    mut waiting: ResMut<Rebinding>,
    mut binds: ResMut<KeyBinds>,
    player: Res<PlayerSettings>,
    file: Res<SettingsFile>,
) {
    let Some(name) = waiting.0.clone() else {
        return;
    };
    if *screen != Screen::Settings {
        waiting.0 = None;
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        keys.clear_just_pressed(KeyCode::Escape);
        waiting.0 = None;
        return;
    }
    let Some(key) = key_taken(&keys) else {
        return;
    };
    keys.clear_just_pressed(key);
    if let Some(slot) = controls::slots().find(|s| s.name() == name) {
        binds.set(&slot, key);
        file.keep(&player, &binds);
    }
    waiting.0 = None;
}

/// A key in a cap: the oak-rimmed tooltip with the key cut in the kit's font.
fn key_cap(parent: &mut ChildSpawnerCommands, kit: &UiKit, key: Localized) {
    parent
        .spawn((
            kit.nine(Set::Oak, "tooltip"),
            KitPiece(Set::Oak.key("tooltip")),
            Node {
                min_width: px(18),
                height: px(18),
                padding: UiRect::new(px(6), px(6), px(5), px(0)),
                justify_content: JustifyContent::Center,
                flex_shrink: 0.0,
                ..default()
            },
        ))
        .with_children(|cap| {
            cap.spawn(KitText::new(key, ui_kit::TEXT));
        });
}

/// The saves panel: a title, the worlds in a scroll area with its groove on
/// the right (`save-slots` 9), the name field and NEW WORLD under it (both
/// rebuilt when the list changes), and BACK.
fn spawn_saves(commands: &mut Commands, kit: &UiKit) {
    commands
        .spawn(timber_layer(Screen::Saves))
        .with_children(|layer| {
            layer
                .spawn(framed(kit, Set::Oak, 280.0))
                .with_children(|panel| {
                    timber_lintel(panel, kit, tr("menu.worlds"));
                    panel
                        .spawn(Node {
                            column_gap: px(3),
                            flex_shrink: 0.0,
                            ..default()
                        })
                        .with_children(|well| {
                            well.spawn((
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    flex_grow: 1.0,
                                    row_gap: px(3),
                                    max_height: px(WORLDS_LIST_HEIGHT),
                                    overflow: Overflow::scroll_y(),
                                    ..default()
                                },
                                ScrollPosition::default(),
                                WheelScrolls,
                                WorldsScroll,
                                SaveList,
                            ));
                            well.spawn((
                                kit.nine(Set::Stone, "slider-track"),
                                KitPiece("slider-track".into()),
                                Node {
                                    width: px(8),
                                    flex_shrink: 0.0,
                                    display: Display::None,
                                    ..default()
                                },
                                Button,
                                RelativeCursorPosition::default(),
                                WorldsTrack,
                            ))
                            .with_children(|groove| {
                                groove.spawn((
                                    kit.whole(Set::Oak, "slider-knob"),
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px(-1),
                                        top: px(0),
                                        width: px(9),
                                        height: px(WORLDS_KNOB_HEIGHT),
                                        ..default()
                                    },
                                    WorldsKnob,
                                ));
                            });
                        });
                    panel.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: px(3),
                            ..default()
                        },
                        SaveFoot,
                    ));
                    panel
                        .spawn(Node {
                            justify_content: JustifyContent::FlexEnd,
                            margin: UiRect::top(px(4)),
                            ..default()
                        })
                        .with_children(|foot| {
                            small(foot, kit, tr("menu.back"), MenuAction::Back);
                        });
                });
        });
}

/// A line of small text, for a slot's date or a reason something did not
/// happen.
fn note(text: String, colour: Color) -> impl Bundle {
    body(text, 9.0, colour)
}

/// How long ago, in words a player reads rather than a timestamp.
fn since(then: u64) -> String {
    let now = saves::now_unix_s();
    let seconds = now.saturating_sub(then);
    match seconds {
        0..=90 => tr("menu.just_now").into(),
        s if s < 5400 => tr_with("menu.since.minutes", &[("count", &(s / 60))]).into(),
        s if s < 172_800 => tr_with("menu.since.hours", &[("count", &(s / 3600))]).into(),
        s => tr_with("menu.since.days", &[("count", &(s / 86_400))]).into(),
    }
}

/// The order the worlds are drawn in, as indices into the list: the open
/// world first (its CONTINUE row), then the rest as the list has them. The
/// indices themselves are what a row's button carries, so they are never
/// renumbered.
fn draw_order(slots: &[Slot], playing: Option<&String>) -> Vec<usize> {
    let open = slots.iter().position(|slot| Some(&slot.id) == playing);
    open.into_iter()
        .chain((0..slots.len()).filter(|row| Some(*row) != open))
        .collect()
}

/// Draw the list. Rebuilt rather than updated, because the list changes SHAPE
/// when a world is made or deleted and a row's button carries its index.
///
/// Only the rows are rebuilt, inside the scroll area, whose `ScrollPosition`
/// is the area's own and survives it: a rebuild keeps the player where they
/// were scrolled, and a DELETE confirm, which stands in the rows' place,
/// leaves it untouched so CANCEL returns to the row it was asked from.
pub fn rebuild_saves(
    mut commands: Commands,
    kit: Res<UiKit>,
    index: Res<SaveIndex>,
    name: Res<NameField>,
    open: Option<Res<WorldSave>>,
    lists: Query<Entity, With<SaveList>>,
    foots: Query<Entity, With<SaveFoot>>,
) {
    if !index.is_changed() {
        return;
    }
    let kit = &*kit;
    let playing = open
        .as_ref()
        .and_then(|save| save.slot().map(|slot| slot.id.clone()));
    let asking = index.asking.is_some_and(|row| row < index.slots.len());
    for foot in &foots {
        commands.entity(foot).despawn_related::<Children>();
        if asking {
            continue;
        }
        commands.entity(foot).with_children(|rows| {
            if let Some(trouble) = index.trouble.as_ref() {
                rows.spawn(note(trouble.clone(), ui_kit::TITLE));
            }
            rows.spawn(Node {
                height: px(3),
                ..default()
            });
            // The name for a new world, typed in the game's own box: a
            // native prompt over the window is the thing the inherited UI
            // rule forbids. Prefilled with the number the page would have
            // chosen, so Enter alone still makes one.
            rows.spawn(Node {
                column_gap: px(4),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|line| {
                line.spawn(KitText::new(tr("menu.name"), ui_kit::TITLE).shadow(ui_kit::INK));
                line.spawn((
                    Button,
                    kit.nine(Set::Oak, "tooltip"),
                    KitPiece(Set::Oak.key("tooltip")),
                    Node {
                        flex_grow: 1.0,
                        height: px(18),
                        padding: UiRect::new(px(6), px(6), px(3), px(0)),
                        ..default()
                    },
                    MenuAction::EditName,
                ))
                .with_children(|field| {
                    field.spawn((body(name.shown(), 10.0, ui_kit::TEXT), NameText));
                });
            });
            // The new world's mode (`survival-mode` design 1): Explorer or
            // Survival, the one picked lit as the focused plaque.
            rows.spawn(Node {
                column_gap: px(3),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|line| {
                for (survival, mode) in [
                    (false, pbd_core::survival::WorldMode::Explorer),
                    (true, pbd_core::survival::WorldMode::Survival),
                ] {
                    let plaque = small(line, kit, mode.name(), MenuAction::Mode(survival));
                    if name.survival == survival {
                        line.commands().entity(plaque).insert(Plaque {
                            focus: true,
                            ..Plaque::new(Set::Oak)
                        });
                    }
                }
            });
            rows.spawn(Node {
                justify_content: JustifyContent::FlexStart,
                ..default()
            })
            .with_children(|line| {
                small(line, kit, tr("menu.new_world"), MenuAction::New);
            });
        });
    }
    for list in &lists {
        commands.entity(list).despawn_related::<Children>();
        commands.entity(list).with_children(|rows| {
            // A delete REPLACES the list while it is pending, and says what
            // is lost rather than only asking. Tenebris's own dialog, ported
            // for its reason: a row that says "delete?" beside nine other
            // rows is a question a player answers without reading it.
            if let Some(row) = index.asking
                && let Some(slot) = index.slots.get(row)
            {
                rows.spawn(body(
                    tr_with("menu.delete_world", &[("world", &slot.file.name)]),
                    11.0,
                    ui_kit::TITLE,
                ));
                rows.spawn(note(
                    tr("menu.everything_dug_built_carried_gone").into(),
                    ui_kit::TEXT,
                ));
                rows.spawn(Node {
                    height: px(4),
                    ..default()
                });
                rows.spawn(Node {
                    column_gap: px(3),
                    ..default()
                })
                .with_children(|line| {
                    small(
                        line,
                        kit,
                        tr("menu.delete_forever"),
                        MenuAction::Delete(row),
                    );
                    let keep = small(line, kit, tr("menu.cancel"), MenuAction::Keep);
                    line.commands().entity(keep).insert(Plaque {
                        focus: true,
                        ..Plaque::new(Set::Oak)
                    });
                });
                return;
            }
            if index.slots.is_empty() {
                rows.spawn(note(tr("menu.no_worlds_yet").into(), ui_kit::TEXT_DIM));
            }
            for row in draw_order(&index.slots, playing.as_ref()) {
                let slot = &index.slots[row];
                let here = Some(&slot.id) == playing.as_ref();
                rows.spawn(Node {
                    column_gap: px(3),
                    align_items: AlignItems::Center,
                    min_height: px(24),
                    // Rows keep their height in the scroll area: it scrolls
                    // rather than squeezing them to fit.
                    flex_shrink: 0.0,
                    ..default()
                })
                .with_children(|line| {
                    line.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        flex_grow: 1.0,
                        row_gap: px(1),
                        ..default()
                    })
                    .with_children(|words| {
                        let shown = if here {
                            tr_with("menu.world_open", &[("world", &slot.file.name)])
                        } else {
                            Localized::verbatim(slot.file.name.clone())
                        };
                        words.spawn(KitText::new(shown, ui_kit::TITLE).shadow(ui_kit::INK));
                        // Its mode beside when it was played (`survival-mode`
                        // design 1): a world from before modes is Explorer.
                        let mode = slot
                            .identity
                            .as_ref()
                            .map_or(pbd_core::survival::WorldMode::Explorer, |i| i.mode());
                        words
                            .spawn(Node {
                                column_gap: px(4),
                                ..default()
                            })
                            .with_children(|under| {
                                under.spawn(note(since(slot.file.played_unix_s), ui_kit::TEXT_DIM));
                                under.spawn(note(mode.name().into(), ui_kit::TEXT_DIM));
                            });
                    });
                    if here {
                        small(line, kit, tr("menu.continue"), MenuAction::Resume);
                    } else {
                        small(line, kit, tr("menu.load"), MenuAction::Load(row));
                    }
                    small(line, kit, tr("menu.delete"), MenuAction::Ask(row));
                });
            }
        });
    }
}

/// An oak plaque that sits in a row rather than filling the panel.
pub(super) fn small(
    parent: &mut ChildSpawnerCommands,
    kit: &UiKit,
    label: Localized,
    action: MenuAction,
) -> Entity {
    parent
        .spawn((plaque(kit, Set::Oak, 30.0), action))
        .with_children(|row| {
            row.spawn(plaque_label(Set::Oak, label));
        })
        .id()
}

/// `Escape` steps back one screen, and is CONSUMED.
///
/// It runs in `PreUpdate` after the input systems and before
/// `RunFixedMainLoop`, where both of the world's input readers live, so
/// clearing it here means they never see it. That is why neither of them has
/// an `Escape` arm any more: a key with one owner needs no agreement about who
/// acts on it.
pub fn toggle(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut screen: ResMut<Screen>,
    front: Res<FrontDoor>,
    save: Option<Res<WorldSave>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    keys.clear_just_pressed(KeyCode::Escape);
    let has_world = save.as_ref().is_some_and(|save| save.slot().is_some());
    let next = screen.back(front.0, has_world);
    if next != *screen {
        *screen = next;
    }
}

/// The keyboard while it is in the name field. Runs before `toggle` so that
/// Escape leaves the field rather than the page, and before the world's
/// readers, which stand down anyway while a menu is open.
pub fn name_input(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut typed: MessageReader<KeyboardInput>,
    mut name: ResMut<NameField>,
    mut texts: Query<&mut Text, With<NameText>>,
) {
    if !name.focused {
        typed.clear();
        return;
    }
    if keys.just_pressed(KeyCode::Escape) {
        keys.clear_just_pressed(KeyCode::Escape);
        name.focused = false;
    }
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
        keys.clear_just_pressed(KeyCode::Enter);
        keys.clear_just_pressed(KeyCode::NumpadEnter);
        name.submit = true;
    }
    for event in typed.read() {
        if event.state.is_pressed() {
            name.typed(&event.logical_key);
        }
    }
    if name.is_changed() {
        for mut text in &mut texts {
            text.0 = name.shown();
        }
    }
}

/// A press does what its own `MenuAction` says.
///
/// The world-changing ones (load, delete, new) go through `SaveIndex` and
/// `LoadRequest` rather than acting here, because a press cannot borrow the
/// planet, the tier, the walker and the hotbar at once - and because a list
/// that is read to draw and read again to act is two lists.
pub fn press(
    mut screen: ResMut<Screen>,
    mut exit: MessageWriter<AppExit>,
    mut page: SavesPage,
    save: Option<Res<WorldSave>>,
    rows: Query<(&Interaction, &MenuAction), Changed<Interaction>>,
    mut forcing: ResMut<pbd_app::weather::StormForcing>,
    mut overlay: ResMut<pbd_app::overlay::OverlayMode>,
) {
    let SavesPage {
        index,
        load,
        front,
        name,
    } = &mut page;
    let root = save
        .as_ref()
        .map_or_else(pbd_app::paths::saves_dir, |s| s.root().into());
    let seed = pbd_app::planet::terrain_config().seed;
    // Enter in the name field is the NEW WORLD button pressed.
    let submitted = std::mem::take(&mut name.submit);
    let pressed = rows
        .iter()
        .filter(|(interaction, _)| **interaction == Interaction::Pressed)
        .map(|(_, action)| *action)
        .chain(submitted.then_some(MenuAction::New));
    for action in pressed {
        // A press anywhere but the field takes the keyboard out of it.
        if action != MenuAction::EditName && name.focused {
            name.focused = false;
        }
        match action {
            MenuAction::EditName => name.focused = true,
            MenuAction::Mode(survival) => {
                name.survival = survival;
                // The page is drawn from the index: redraw it.
                index.set_changed();
            }
            MenuAction::Weather(percent) => forcing.0 = f32::from(percent.min(100)) / 100.0,
            // The clock's own control answers it (`time_ui::press`).
            MenuAction::Time(_) | MenuAction::Speed(_) | MenuAction::Skip(_) => {}
            MenuAction::Overlay(row) => overlay.0 = super::overlay_ui::mode_for(row),
            MenuAction::Resume => {
                front.0 = false;
                *screen = Screen::Playing;
            }
            MenuAction::Settings => *screen = Screen::Settings,
            MenuAction::Multiplayer => *screen = Screen::Multiplayer,
            // The Multiplayer page's own (`join_ui::press` acts).
            MenuAction::JoinHosted | MenuAction::JoinAddress | MenuAction::EditAddress => {}
            MenuAction::NewWorld => {
                name.focused = true;
                *screen = Screen::Saves;
            }
            // Only the screen: what is ON it is filled by `paint`, from the
            // one place that knows the screen has changed. Filling it here
            // meant the list was empty for anything that opened the screen
            // another way, which is exactly what `--menu saves` found on its
            // first run - and is this project's own lesson about two paths to
            // one job, where the one nobody presses is the one that is wrong.
            MenuAction::Saves => *screen = Screen::Saves,
            // Back to the title as the front door (`title-screen`, "The
            // splash"). Every change is already saved as it is made.
            MenuAction::MainMenu => {
                front.0 = true;
                *screen = Screen::Title;
            }
            MenuAction::Back => {
                let has_world = save.as_ref().is_some_and(|s| s.slot().is_some());
                *screen = screen.back(front.0, has_world);
            }
            MenuAction::Quit => {
                exit.write(AppExit::Success);
            }
            MenuAction::Ask(row) => {
                index.asking = Some(row);
                index.trouble = None;
            }
            MenuAction::Keep => index.asking = None,
            MenuAction::Delete(row) => {
                index.asking = None;
                let Some(slot) = index.slots.get(row).cloned() else {
                    continue;
                };
                let open = save
                    .as_ref()
                    .and_then(|s| s.slot().map(|s| s.id.clone()))
                    .is_some_and(|id| id == slot.id);
                if open {
                    // Deleting the world you are standing in would leave the
                    // writer appending to a directory that is not there.
                    index.trouble = Some(tr("menu.that_world_open_load_another").into());
                } else {
                    index.trouble = saves::delete(&root, &slot.id).err().map(|error| {
                        tr_with(
                            "menu.could_not_delete",
                            &[("world", &slot.file.name), ("error", &error)],
                        )
                        .into()
                    });
                }
                index.slots = saves::list(&root);
            }
            MenuAction::New => {
                let wanted = name.name(index.slots.len());
                index.asking = None;
                let mode = if name.survival {
                    pbd_core::survival::WorldMode::Survival
                } else {
                    pbd_core::survival::WorldMode::Explorer
                };
                match saves::create_in(&root, &wanted, seed, mode) {
                    Ok(slot) => {
                        // Made, and entered: a world a player just named is
                        // the world they want to be in. It is made by the
                        // newest generator, and the load switches the planet
                        // to it in place if an older world's is up
                        // (`bigger-biomes` decision 8).
                        index.trouble = None;
                        name.edited = false;
                        name.focused = false;
                        front.0 = false;
                        load.0 = Some(slot);
                        *screen = Screen::Playing;
                    }
                    Err(error) => {
                        index.trouble =
                            Some(tr_with("menu.could_not_make_world", &[("error", &error)]).into());
                    }
                }
                index.slots = saves::list(&root);
            }
            // The guide's own system turns the page.
            MenuAction::Species(_) => {}
            // The breeding screen's own (`breeding::press` acts).
            MenuAction::BreedSpecies(_) | MenuAction::BreedAnimal(_) | MenuAction::BreedPair => {}
            // `press_graphics` acts.
            MenuAction::Bloom
            | MenuAction::FrameLimit
            | MenuAction::RenderScale
            | MenuAction::AutoStep
            | MenuAction::RainShadows
            | MenuAction::Reflections
            | MenuAction::Shadows
            | MenuAction::Antialiasing
            | MenuAction::MissionLine
            | MenuAction::Volume(_)
            | MenuAction::FlowArrows
            | MenuAction::TimeControls
            | MenuAction::ResetTab => {}
            // The Character page's own (`character_ui::press` acts).
            MenuAction::Look(..) | MenuAction::LookRandom => {}
            MenuAction::Load(row) => {
                let Some(slot) = index.slots.get(row).cloned() else {
                    continue;
                };
                if slot.file.seed != seed {
                    // A save is OF a world. Loading it into a different one
                    // would make it silently become somebody else's.
                    index.trouble = Some(
                        tr_with(
                            "menu.other_world",
                            &[("world", &slot.file.name), ("seed", &slot.file.seed)],
                        )
                        .into(),
                    );
                    continue;
                }
                if let Some(why) = saves::refusal(&slot) {
                    // Opened, it would be generated by rules it was not made
                    // by: say which version this build lacks instead.
                    index.trouble = Some(
                        tr_with(
                            "menu.made_with",
                            &[("world", &slot.file.name), ("why", &why)],
                        )
                        .into(),
                    );
                    continue;
                }
                // A world of another generator than the planet's is loaded
                // like any other: the load switches the planet in place
                // (`bigger-biomes` decision 8).
                front.0 = false;
                load.0 = Some(slot);
                *screen = Screen::Playing;
            }
        }
    }
}

/// Show the open panel, hide the others, and publish who holds the pointer.
///
/// `MenuOpen` is written HERE and nowhere else, so there is one answer to the
/// question every input reader asks.
pub fn paint(
    screen: Res<Screen>,
    mut menu: ResMut<MenuOpen>,
    mut index: ResMut<SaveIndex>,
    mut name: ResMut<NameField>,
    save: Option<Res<WorldSave>>,
    mut panels: Query<(&Panel, &mut Node)>,
    mut worlds: Query<&mut ScrollPosition, With<WorldsScroll>>,
) {
    if screen.is_changed() {
        menu.0 = *screen != Screen::Playing;
        if *screen == Screen::Saves {
            // Opened at the top, where the open world's CONTINUE is.
            for mut at in &mut worlds {
                at.0 = Vec2::ZERO;
            }
            // Read the directory on the way IN, however the screen was
            // opened, so the list is always what is on disk right now.
            let root = save
                .as_ref()
                .map_or_else(pbd_app::paths::saves_dir, |s| s.root().into());
            index.slots = saves::list(&root);
            index.asking = None;
            index.trouble = None;
            name.focused = false;
            let worlds = index.slots.len();
            name.prefill(worlds);
        }
        for (panel, mut node) in &mut panels {
            node.display = if panel.0 == *screen {
                Display::Flex
            } else {
                Display::None
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Out of play is timber (owner, 2026-11-14: "pause menu = timber too"):
    /// the pause, settings and saves panels are framed in the carved oak, and
    /// every plaque on them is oak.
    #[test]
    fn the_timber_menus_are_oak() {
        let mut app = App::new();
        pbd_app::ui_kit::install_bare(&mut app);
        app.add_systems(Update, spawn);
        app.update();
        let world = app.world_mut();
        let oak = world.resource::<UiKit>().image("oak/frame");
        let mut panels = world.query::<(&Panel, &Children)>();
        let found: Vec<(Screen, Entity)> = panels
            .iter(world)
            .map(|(panel, children)| (panel.0, children[0]))
            .collect();
        let screens: Vec<Screen> = found.iter().map(|f| f.0).collect();
        for screen in [Screen::Pause, Screen::Settings, Screen::Saves] {
            assert!(screens.contains(&screen), "{screen:?} has a panel");
        }
        for (screen, frame) in found {
            // The title's first child is its logo and panel, in a column.
            if screen == Screen::Title {
                continue;
            }
            let image = world.get::<ImageNode>(frame).expect("a framed panel");
            assert_eq!(image.image, oak, "{screen:?} is framed in oak");
        }
        let mut plaques = world.query::<&Plaque>();
        assert!(plaques.iter(world).count() >= 5);
        for plaque in plaques.iter(world) {
            assert_eq!(plaque.set, Set::Oak, "a timber menu's plaque is oak");
        }
    }

    /// Escape steps out one screen at a time, and from the world it steps IN.
    /// A settings page that resumed the game on its way past the pause menu is
    /// the reason this is one enum rather than a pair of booleans.
    #[test]
    fn escape_steps_one_screen_at_a_time() {
        assert_eq!(Screen::Playing.back(false, true), Screen::Pause);
        assert_eq!(Screen::Settings.back(false, true), Screen::Pause);
        assert_eq!(Screen::Pause.back(false, true), Screen::Playing);
        assert_eq!(Screen::Saves.back(false, true), Screen::Pause);
        assert_eq!(Screen::Guide.back(false, true), Screen::Playing);
        assert_eq!(Screen::Pack.back(false, true), Screen::Playing);
    }

    /// The title as the front door (`title-screen`): Escape stays on it, and
    /// the saves page and settings step back to it. Through the pause menu
    /// they are the pages they always were.
    #[test]
    fn the_front_door_steps_back_to_the_title() {
        assert_eq!(Screen::Title.back(true, true), Screen::Title);
        assert_eq!(Screen::Saves.back(true, true), Screen::Title);
        assert_eq!(Screen::Settings.back(true, true), Screen::Title);
        assert_eq!(Screen::Saves.back(false, false), Screen::Pause);
        assert_eq!(Screen::Settings.back(false, true), Screen::Pause);
    }

    /// A plain launch opens on the title screen; a named world or a capture
    /// opens on the world; `--menu` names its own.
    #[test]
    fn a_plain_launch_opens_on_the_title() {
        assert_eq!(opening_screen(None, false, false), Screen::Title);
        assert_eq!(opening_screen(Some("title"), true, true), Screen::Title);
        assert_eq!(opening_screen(None, true, false), Screen::Playing);
        assert_eq!(opening_screen(None, false, true), Screen::Playing);
        assert_eq!(opening_screen(Some("pause"), true, true), Screen::Pause);
        assert_eq!(opening_screen(Some("saves"), true, true), Screen::Saves);
    }

    /// The name field: typed text goes in, Backspace takes it out, the
    /// prefill yields to anything typed, and an empty field makes the
    /// numbered world the page offered.
    #[test]
    fn the_name_field_takes_typing_and_offers_a_number() {
        let mut field = NameField::default();
        field.prefill(2);
        assert_eq!(field.text, "World 3");
        assert_eq!(field.name(2), "World 3");
        field.text.clear();
        field.typed(&Key::Character("Ca".into()));
        field.typed(&Key::Character("v".into()));
        field.typed(&Key::Space);
        field.typed(&Key::Character("e".into()));
        field.typed(&Key::Backspace);
        field.typed(&Key::Enter);
        assert_eq!(field.text, "Cav ");
        assert_eq!(field.name(2), "Cav");
        field.prefill(5);
        assert_eq!(field.text, "Cav ", "typed text is not overwritten");
        field.focused = true;
        assert_eq!(field.shown(), "Cav _");
        for _ in 0..100 {
            field.typed(&Key::Character("x".into()));
        }
        assert_eq!(field.text.chars().count(), NAME_MAX);
        let mut blank = NameField::default();
        blank.typed(&Key::Character("   ".into()));
        assert_eq!(blank.name(0), "World 1", "blank is the offered number");
    }

    /// Anything but `Playing` holds the pointer. Written as the test of the
    /// rule rather than of the line, because this is what every input reader
    /// in the app is actually asking.
    #[test]
    fn only_the_world_leaves_the_pointer_alone() {
        for (screen, held) in [
            (Screen::Playing, false),
            (Screen::Pause, true),
            (Screen::Settings, true),
        ] {
            assert_eq!(screen != Screen::Playing, held, "{screen:?}");
        }
    }

    /// The frame limit and render scale sliders keep the plaques' values and
    /// steps: off, 60, 120; 50, 67, 75, 85, 100; a press lands on the step
    /// nearest the cursor.
    #[test]
    fn the_sliders_keep_the_plaques_values_and_steps() {
        use pbd_app::settings::FrameLimit;
        let player = PlayerSettings::default();
        assert_eq!(scale_steps(), vec![50, 67, 75, 85, 100]);
        assert_eq!(slider_step(MenuAction::FrameLimit, &player), Some((3, 1)));
        assert_eq!(slider_step(MenuAction::RenderScale, &player), Some((5, 3)));
        assert_eq!(nearest_step(0.0, 3), 0);
        assert_eq!(nearest_step(0.74, 3), 1);
        assert_eq!(nearest_step(1.2, 3), 2);
        assert_eq!(nearest_step(0.49, 5), 2);
        let off = with_step(MenuAction::FrameLimit, player, 0);
        assert_eq!(off.frame_limit, FrameLimit::Off);
        assert_eq!(
            with_step(MenuAction::FrameLimit, player, 2).frame_limit,
            FrameLimit::Fps120
        );
        assert_eq!(
            with_step(MenuAction::RenderScale, player, 0).render_scale,
            50
        );
        assert_eq!(
            with_step(MenuAction::RenderScale, player, 4).render_scale,
            100
        );
        assert_eq!(
            settings_value(MenuAction::FrameLimit, &off).as_deref(),
            Some("OFF")
        );
        // REFLECTIONS (`reflection-quality`): four steps, HIGH at the top
        // and the default, each reading its own name.
        assert_eq!(slider_step(MenuAction::Reflections, &player), Some((4, 3)));
        for (step, word) in ["OFF", "LOW", "MEDIUM", "HIGH"].iter().enumerate() {
            let chosen = with_step(MenuAction::Reflections, player, step);
            assert_eq!(chosen.reflections, Reflections::ALL[step]);
            assert_eq!(
                settings_value(MenuAction::Reflections, &chosen).as_deref(),
                Some(*word)
            );
        }
        // Every offered value is a step, and every step an offered value.
        for scale in pbd_app::settings::SCALES {
            assert!(scale_steps().contains(&scale));
        }
    }

    fn rebind_app(waiting: Option<&str>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(Screen::Settings)
            .insert_resource(Rebinding(waiting.map(String::from)))
            .init_resource::<KeyBinds>()
            .init_resource::<PlayerSettings>()
            .init_resource::<SettingsFile>()
            .add_systems(Update, capture_rebind);
        app
    }

    /// Click a binding, press a key: the slot takes it and the key is
    /// consumed. Esc cancels and is consumed, so the page stays open. A key
    /// no slot can take is waited through.
    #[test]
    fn a_cap_takes_the_next_key_and_esc_cancels() {
        let mut app = rebind_app(Some("on_foot:ShiftLeft"));
        let press = |app: &mut App, key: KeyCode| {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.clear();
            keys.press(key);
            app.update();
        };
        press(&mut app, KeyCode::Lang1);
        assert!(
            app.world().resource::<Rebinding>().0.is_some(),
            "still waiting"
        );
        press(&mut app, KeyCode::KeyE);
        assert_eq!(app.world().resource::<Rebinding>().0, None);
        let sprint = controls::slots()
            .find(|s| s.name() == "on_foot:ShiftLeft")
            .unwrap();
        assert_eq!(
            app.world().resource::<KeyBinds>().key(&sprint),
            KeyCode::KeyE
        );
        assert!(
            !app.world()
                .resource::<ButtonInput<KeyCode>>()
                .just_pressed(KeyCode::KeyE),
            "the key is the cap's, not the world's"
        );

        app.world_mut().resource_mut::<Rebinding>().0 = Some("interact:KeyF".into());
        press(&mut app, KeyCode::Escape);
        assert_eq!(app.world().resource::<Rebinding>().0, None, "Esc cancels");
        assert!(
            !app.world()
                .resource::<ButtonInput<KeyCode>>()
                .just_pressed(KeyCode::Escape),
            "and is consumed, so the page does not step back"
        );
        let door = controls::slots()
            .find(|s| s.name() == "interact:KeyF")
            .unwrap();
        assert_eq!(app.world().resource::<KeyBinds>().key(&door), KeyCode::KeyF);

        // Leaving the settings stops a wait.
        app.world_mut().resource_mut::<Rebinding>().0 = Some("interact:KeyF".into());
        *app.world_mut().resource_mut::<Screen>() = Screen::Pause;
        press(&mut app, KeyCode::KeyK);
        assert_eq!(app.world().resource::<Rebinding>().0, None);
        assert_eq!(app.world().resource::<KeyBinds>().key(&door), KeyCode::KeyF);
    }

    /// The Input page: five oak sub-tabs, one list each, a changeable cap
    /// for every slot, and a clash flagged on its row (the change's
    /// "Rebinding a key" scenario, with the game's own rows).
    #[test]
    fn the_input_page_lists_every_slot_and_flags_a_clash() {
        let mut app = App::new();
        pbd_app::ui_kit::install_bare(&mut app);
        app.init_resource::<KeyBinds>()
            .init_resource::<Rebinding>()
            .add_systems(Startup, spawn)
            .add_systems(Update, rebuild_input);
        app.update();
        app.update();
        let world = app.world_mut();
        // A sub-tab and a list for each of `Tab::ALL`.
        let tabs = controls::Tab::ALL.len();
        assert_eq!(world.query::<&InputTabButton>().iter(world).count(), tabs);
        assert_eq!(world.query::<&InputList>().iter(world).count(), tabs);
        let caps = world.query::<&RebindCap>().iter(world).count();
        assert_eq!(caps, controls::slots().count());
        let also = |world: &mut World| {
            world
                .query::<&Text>()
                .iter(world)
                .filter(|t| t.0.starts_with("also:"))
                .map(|t| t.0.clone())
                .collect::<Vec<_>>()
        };
        assert!(also(world).is_empty(), "the defaults never clash");
        let sprint = controls::slots()
            .find(|s| s.name() == "on_foot:ShiftLeft")
            .unwrap();
        world.resource_mut::<KeyBinds>().set(&sprint, KeyCode::KeyF);
        app.update();
        let world = app.world_mut();
        let flagged = also(world);
        assert!(
            flagged
                .contains(&"also: open, harvest, pick, feed, till; board or step off".to_string()),
            "{flagged:?}"
        );
        assert!(flagged.contains(&"also: sprint".to_string()), "{flagged:?}");
        // The rebound cap reads its new key.
        let labels: Vec<String> = world
            .query::<&KitText>()
            .iter(world)
            .map(|t| t.text.to_string())
            .collect();
        assert!(labels.contains(&"F".to_string()));
    }

    /// The worlds page (`save-slots` 9): the rows sit in the scroll area and
    /// NAME and NEW WORLD in the fixed foot under it; a rebuild keeps the
    /// scroll offset; a DELETE confirm stands in the rows' place without
    /// touching it, so CANCEL comes back to the same rows.
    #[test]
    fn the_worlds_list_scrolls_and_keeps_its_place() {
        let mut app = App::new();
        pbd_app::ui_kit::install_bare(&mut app);
        let slots: Vec<Slot> = (0..30)
            .map(|i| Slot {
                id: format!("w{i}"),
                file: saves::format::WorldFile::new(format!("w{i}"), 1, 0),
                identity: None,
            })
            .collect();
        app.insert_resource(SaveIndex {
            slots,
            asking: None,
            trouble: None,
        })
        .init_resource::<NameField>()
        .add_systems(Startup, spawn)
        .add_systems(Update, rebuild_saves);
        app.update();
        let world = app.world_mut();
        let list = world
            .query_filtered::<Entity, With<SaveList>>()
            .single(world)
            .unwrap();
        let foot = world
            .query_filtered::<Entity, With<SaveFoot>>()
            .single(world)
            .unwrap();
        assert!(world.get::<WorldsScroll>(list).is_some());
        let rows = |world: &World| world.get::<Children>(list).map_or(0, |c| c.len());
        assert_eq!(rows(world), 30, "one row a world, nothing else");
        assert!(world.get::<Children>(foot).is_some_and(|c| c.len() >= 3));
        let name_in = |world: &mut World, root: Entity| {
            let mut stack = vec![root];
            while let Some(e) = stack.pop() {
                if world.get::<NameText>(e).is_some() {
                    return true;
                }
                if let Some(c) = world.get::<Children>(e) {
                    stack.extend(c.iter());
                }
            }
            false
        };
        assert!(name_in(world, foot) && !name_in(world, list));
        world.get_mut::<ScrollPosition>(list).unwrap().0.y = 120.0;
        world.resource_mut::<SaveIndex>().trouble = None;
        app.update();
        let world = app.world_mut();
        assert_eq!(world.get::<ScrollPosition>(list).unwrap().0.y, 120.0);
        world.resource_mut::<SaveIndex>().asking = Some(7);
        app.update();
        let world = app.world_mut();
        assert!(rows(world) < 30, "the confirm stands in the rows' place");
        assert!(world.get::<Children>(foot).is_none_or(|c| c.is_empty()));
        assert_eq!(world.get::<ScrollPosition>(list).unwrap().0.y, 120.0);
        world.resource_mut::<SaveIndex>().asking = None;
        app.update();
        let world = app.world_mut();
        assert_eq!(rows(world), 30);
        assert_eq!(world.get::<ScrollPosition>(list).unwrap().0.y, 120.0);
    }

    /// The open world is drawn first whatever its place in the list, and the
    /// rows keep their list indices.
    #[test]
    fn the_open_world_is_drawn_first() {
        let slots: Vec<Slot> = ["a", "b", "c"]
            .iter()
            .map(|id| Slot {
                id: id.to_string(),
                file: saves::format::WorldFile::new(id.to_string(), 1, 0),
                identity: None,
            })
            .collect();
        assert_eq!(draw_order(&slots, Some(&"c".to_string())), vec![2, 0, 1]);
        assert_eq!(draw_order(&slots, None), vec![0, 1, 2]);
    }

    /// The worlds grip runs the groove's length less its own height, and a
    /// press puts the grip's middle under the cursor.
    #[test]
    fn the_worlds_grip_follows_the_scroll() {
        let track = 270.0;
        let run = track - WORLDS_KNOB_HEIGHT;
        assert_eq!(knob_top(0.0, 500.0, track), 0.0);
        assert_eq!(knob_top(500.0, 500.0, track), run);
        assert_eq!(knob_top(250.0, 500.0, track), run / 2.0);
        assert_eq!(
            knob_top(900.0, 500.0, track),
            run,
            "past the end is the end"
        );
        assert_eq!(knob_top(50.0, 0.0, track), 0.0);
        assert_eq!(scroll_for(0.0, 500.0, track), 0.0);
        assert_eq!(scroll_for(1.0, 500.0, track), 500.0);
        assert!((scroll_for(0.5, 500.0, track) - 250.0).abs() < 1e-3);
    }
}
