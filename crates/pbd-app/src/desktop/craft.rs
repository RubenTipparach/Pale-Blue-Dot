//! Making things (`tool-crafting` decisions 4, 5, 6 and 9): the pack's Craft
//! row (M29, the owner's "basic crafting system, with a drop down list"),
//! the workbench's panel, and the one queue-and-collect every station uses,
//! which hands a made tool to the tool slot.
//!
//! The rules are the core's (`pbd_core::station`): a recipe queued takes its
//! inputs, its job runs on world time, and a finished job is collected into
//! the slots and the tool slot. What is here is the durable writes and the
//! stone-and-wood widgets.
//!
//! - **The pack's row**, under the hotbar: a drop-down of what the hands make
//!   (a recipe short of something dimmed), the wells of what it takes lit or
//!   dark against what is carried, an arrow, the result, and CRAFT. The hands
//!   make one thing at a time; the plaque counts the job up while it runs,
//!   and the result lands by the give order.
//! - **The workbench's panel**: its recipes as a grid of wells (a click picks
//!   one), the same row under them, and the queue's line and bar.

use super::farm::{FarmNote, StationAt};
use super::menu::Screen;
use super::slots::{self, Hotbar, ItemIcons};
use bevy::prelude::*;
use pbd_app::config::Recipes;
use pbd_app::fish::ToolSlot;
use pbd_app::hotbar::{Gained, How};
use pbd_app::saves::WorldSave;
use pbd_app::ui_kit::{self, KitText, Plaque, Set, Tooltip, UiKit};
use pbd_core::inventory::{Item, Stack, Tier};
use pbd_core::records::Record as StoredRecord;
use pbd_core::station::{
    self, HAND_RECORD_ID, Recipe, STATION_KIND, STATION_SCHEMA, StationRow, StationState,
};

/// Which recipe each widget has picked, and whether the drop-down is open.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CraftPick {
    /// Into the hands' recipes, in file order.
    pub hand: usize,
    /// Into the workbench's.
    pub bench: usize,
    pub list_open: bool,
}

/// What a well shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// The recipe's `k`th input.
    Need(usize),
    /// What it makes.
    Result,
    /// The workbench's `k`th recipe, to pick.
    Pick(usize),
}

/// A recipe well: whose (the station's code) and what it shows.
#[derive(Component, Clone, Copy)]
pub struct Well {
    pub station: &'static str,
    pub role: Role,
}

#[derive(Component)]
pub struct WellIcon;
#[derive(Component)]
pub struct WellCount;

/// The CRAFT plaque, by station code.
#[derive(Component)]
pub struct CraftButton(pub &'static str);
#[derive(Component)]
pub struct CraftLabel(pub &'static str);

/// The hands' drop-down, its picture and its words.
#[derive(Component)]
pub struct DropDown;
#[derive(Component)]
pub struct DropIcon;
#[derive(Component)]
pub struct DropLabel;
/// The open list, and one of its options by index.
#[derive(Component)]
pub struct DropList;
#[derive(Component)]
pub struct DropOption(pub usize);

/// The workbench panel's queue line and bar.
#[derive(Component)]
pub struct BenchLine;
#[derive(Component)]
pub struct BenchFill;

/// A station's queue record id: a cell's station, or the hands'.
pub fn station_id(at: Option<(u32, u16)>) -> u64 {
    at.map_or(HAND_RECORD_ID, |(cell, layer)| {
        station::record_id(cell, layer)
    })
}

/// A queue's state by its record id; an empty queue where it has none.
pub fn state_of(save: &WorldSave, id: u64, row: &StationRow) -> StationState {
    save.records
        .get(STATION_KIND, id)
        .and_then(|r| r.read::<StationState>())
        .filter(|s| s.station == row.code)
        .unwrap_or_else(|| StationState::new(row))
}

/// The station block standing at a cell and layer: stations are always
/// placed, so it is the edit's material.
pub fn station_row_at(save: &WorldSave, cell: u32, layer: u16) -> Option<&'static StationRow> {
    save.edits
        .for_cell(cell)
        .iter()
        .find(|&&(l, _)| l == layer)
        .and_then(|&(_, m)| station::station_of(m))
}

fn store(
    save: &mut WorldSave,
    id: u64,
    state: &StationState,
    slots: &pbd_core::inventory::Slots,
) -> bool {
    let record = StoredRecord::of(STATION_KIND, id, STATION_SCHEMA, state);
    save.store_with_hands(record, STATION_SCHEMA, slots)
}

/// The recipe's first output as a well shows it: a tool at its tier.
fn shown(recipe: &Recipe) -> Option<(Item, u16, Option<Tier>)> {
    if let Some(&(item, n)) = recipe.outputs.first() {
        return Some((item, n, None));
    }
    recipe
        .tools
        .first()
        .map(|&(tool, tier)| (Item::Tool(tool), 1, Some(tier)))
}

/// What a recipe's name says in a list: "PLANKS X4".
fn label(recipe: &Recipe) -> String {
    match shown(recipe) {
        Some((_, n, _)) if n > 1 => format!("{} X{n}", recipe.name),
        _ => recipe.name.clone(),
    }
}

/// Queue `code` on the queue `id` of station `row`, taking its inputs, in
/// one saved line. A tool that would not be an upgrade is refused first.
#[allow(clippy::too_many_arguments)]
pub fn queue(
    save: &mut WorldSave,
    hotbar: &mut Hotbar,
    tools: Option<&ToolSlot>,
    recipes: &Recipes,
    id: u64,
    code: &str,
    now_s: f64,
    note: &mut FarmNote,
    wall_s: f32,
) -> bool {
    let Some(recipe) = recipes.0.get(code) else {
        return false;
    };
    let mut state = state_of(save, id, recipe.station);
    let mut moved = hotbar.0.clone();
    match state.queue_for(
        &mut moved,
        tools.map(|t| &t.0),
        Some(&recipes.0),
        recipe,
        now_s,
    ) {
        Ok(()) => {}
        Err(station::Refused::Short) => {
            note.say(wall_s, format!("Not enough for {}", recipe.name));
            return false;
        }
        Err(station::Refused::Full) => {
            note.say(
                wall_s,
                if recipe.station.material.is_none() {
                    "Your hands are busy".to_string()
                } else {
                    format!("The {}'s queue is full", recipe.station.name)
                },
            );
            return false;
        }
        Err(station::Refused::NotBetter) => {
            note.say(wall_s, "You have that tool, or a better one");
            return false;
        }
        Err(station::Refused::Elsewhere) => return false,
    }
    if !store(save, id, &state, &moved) {
        error!("queue BLOCKED: the save did not accept it");
        return false;
    }
    hotbar.0 = moved;
    info!(
        "queued {code} at {} queue {id}, {} in it",
        recipe.station.code,
        state.jobs.len()
    );
    true
}

/// Collect every finished job at the head of queue `id`: outputs into the
/// slots, tools into the tool slot (`tool-crafting` decision 8: the `hand`
/// line first, then the queue with the slots), and a `Gained { Made }` for
/// each. Returns how many were collected.
#[allow(clippy::too_many_arguments)]
pub fn collect(
    save: &mut WorldSave,
    hotbar: &mut Hotbar,
    tools: &mut ToolSlot,
    recipes: &Recipes,
    id: u64,
    row: &StationRow,
    now_s: f64,
    note: &mut FarmNote,
    wall_s: f32,
    gained: &mut MessageWriter<Gained>,
) -> usize {
    let mut state = state_of(save, id, row);
    if state.jobs.is_empty() {
        return 0;
    }
    let mut moved = hotbar.0.clone();
    let mut kit = tools.0;
    let done = state.collect_into(&mut moved, Some(&mut kit), &recipes.0, now_s);
    if done.is_empty() {
        return 0;
    }
    if kit != tools.0 {
        if !save.record_hand(&kit) {
            error!("collect BLOCKED: the save did not take the tool");
            return 0;
        }
        tools.0 = kit;
    }
    if !store(save, id, &state, &moved) {
        error!("collect BLOCKED: the save did not accept it");
        return 0;
    }
    hotbar.0 = moved;
    for job in &done {
        let Some(recipe) = recipes.0.get(&job.recipe) else {
            continue;
        };
        for &(item, count) in &recipe.outputs {
            gained.write(Gained {
                item,
                count,
                how: How::Made,
            });
        }
        for &(tool, _) in &recipe.tools {
            gained.write(Gained {
                item: Item::Tool(tool),
                count: 1,
                how: How::Made,
            });
        }
        let what = match recipe.tools.first() {
            Some(&(tool, tier)) => format!("{} made", pbd_core::inventory::tool_title(tool, tier)),
            None => format!("{} done", recipe.name),
        };
        note.say(wall_s, what);
    }
    done.len()
}

// ---------------------------------------------------------------- widgets

/// A recipe well: the stone slot with an icon and a count.
fn well(parent: &mut ChildSpawnerCommands, kit: &UiKit, atlas: &Handle<Image>, well: Well) {
    parent
        .spawn((
            Node {
                width: px(slots::SLOT),
                height: px(slots::SLOT),
                flex_shrink: 0.0,
                ..default()
            },
            kit.whole(Set::Stone, "slot-idle"),
            Button,
            Tooltip::default(),
            well,
        ))
        .with_children(|cell| {
            cell.spawn((
                ImageNode {
                    image: atlas.clone(),
                    color: Color::NONE,
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(4),
                    top: px(4),
                    width: px(16),
                    height: px(16),
                    ..default()
                },
                WellIcon,
            ));
            cell.spawn((
                KitText::new("", ui_kit::LABEL).outline(ui_kit::INK),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: px(1),
                    right: px(1),
                    ..default()
                },
                WellCount,
            ));
        });
}

/// The row both widgets share: up to three needs, an arrow, the result, and
/// CRAFT, after whatever `lead` puts first.
fn craft_row(
    parent: &mut ChildSpawnerCommands,
    kit: &UiKit,
    atlas: &Handle<Image>,
    station: &'static str,
    lead: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn(Node {
            align_items: AlignItems::Center,
            column_gap: px(3),
            margin: UiRect::top(px(3)),
            position_type: PositionType::Relative,
            ..default()
        })
        .with_children(|row| {
            lead(row);
            for k in 0..3 {
                well(
                    row,
                    kit,
                    atlas,
                    Well {
                        station,
                        role: Role::Need(k),
                    },
                );
            }
            row.spawn(KitText::new("→", ui_kit::TEXT).shadow(ui_kit::INK));
            well(
                row,
                kit,
                atlas,
                Well {
                    station,
                    role: Role::Result,
                },
            );
            row.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });
            row.spawn((ui_kit::plaque(kit, Set::Stone, 44.0), CraftButton(station)))
                .with_children(|p| {
                    p.spawn((
                        ui_kit::plaque_label(Set::Stone, "CRAFT"),
                        CraftLabel(station),
                    ));
                });
        });
}

/// The pack's Craft row (M29), spawned inside the pack under its hotbar.
pub fn spawn_pack_row(
    panel: &mut ChildSpawnerCommands,
    kit: &UiKit,
    atlas: &Handle<Image>,
    recipes: &Recipes,
) {
    panel.spawn((
        KitText::new("CRAFT", ui_kit::TEXT_DIM).shadow(ui_kit::INK),
        Node {
            margin: UiRect::new(px(0), px(0), px(6), px(0)),
            ..default()
        },
    ));
    let hand: Vec<&Recipe> = recipes.0.at("hand").collect();
    craft_row(panel, kit, atlas, "hand", |row| {
        row.spawn((
            kit.nine(Set::Stone, "button-idle"),
            Button,
            Node {
                width: px(96),
                height: px(24),
                align_items: AlignItems::Center,
                column_gap: px(3),
                padding: UiRect::new(px(5), px(5), px(0), px(0)),
                flex_shrink: 0.0,
                ..default()
            },
            DropDown,
        ))
        .with_children(|dd| {
            dd.spawn((
                ImageNode {
                    image: atlas.clone(),
                    color: Color::NONE,
                    ..default()
                },
                Node {
                    width: px(12),
                    height: px(12),
                    flex_shrink: 0.0,
                    ..default()
                },
                DropIcon,
            ));
            dd.spawn((
                KitText::new("", ui_kit::LABEL).shadow(ui_kit::WELL),
                DropLabel,
            ));
            dd.spawn(Node {
                flex_grow: 1.0,
                ..default()
            });
            dd.spawn(KitText::new("↓", ui_kit::LABEL).shadow(ui_kit::WELL));
        });
        // The list opens upward over the hotbar, in the kit's frame.
        row.spawn((
            kit.nine(Set::Stone, "frame"),
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                bottom: px(26),
                flex_direction: FlexDirection::Column,
                row_gap: px(1),
                padding: UiRect::all(px(8)),
                min_width: px(120),
                display: Display::None,
                ..default()
            },
            GlobalZIndex(30),
            DropList,
        ))
        .with_children(|list| {
            for (k, recipe) in hand.iter().enumerate() {
                list.spawn((
                    Button,
                    Node {
                        align_items: AlignItems::Center,
                        column_gap: px(3),
                        padding: UiRect::axes(px(2), px(1)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    DropOption(k),
                ))
                .with_children(|option| {
                    option.spawn((
                        ImageNode {
                            image: atlas.clone(),
                            color: Color::NONE,
                            ..default()
                        },
                        Node {
                            width: px(12),
                            height: px(12),
                            flex_shrink: 0.0,
                            ..default()
                        },
                        DropIcon,
                        DropOption(k),
                    ));
                    option.spawn((
                        KitText::new(label(recipe), ui_kit::TEXT).shadow(ui_kit::INK),
                        DropOption(k),
                    ));
                });
            }
        });
    });
}

/// The workbench's panel, a sibling of the forge's in the station layer,
/// shown while a workbench is open.
pub fn spawn_bench(
    layer: &mut ChildSpawnerCommands,
    kit: &UiKit,
    atlas: &Handle<Image>,
    recipes: &Recipes,
) {
    let (rock, (w, h)) = kit.rock("pack");
    let bar = kit.nine(Set::Stone, "bar");
    let fill = kit.whole(Set::Stone, "fill-progress");
    let count = recipes.0.at("workbench").count();
    layer
        .spawn((
            rock,
            Node {
                width: px(w),
                height: px(h),
                padding: UiRect::all(px(14)),
                flex_direction: FlexDirection::Column,
                display: Display::None,
                ..default()
            },
            super::forge::StationPanel("workbench"),
        ))
        .with_children(|panel| {
            super::menu::lintel(panel, kit, "WORKBENCH");
            panel
                .spawn(Node {
                    flex_wrap: FlexWrap::Wrap,
                    width: px(10.0 * slots::SLOT),
                    flex_shrink: 0.0,
                    ..default()
                })
                .with_children(|grid| {
                    for k in 0..count {
                        well(
                            grid,
                            kit,
                            atlas,
                            Well {
                                station: "workbench",
                                role: Role::Pick(k),
                            },
                        );
                    }
                });
            craft_row(panel, kit, atlas, "workbench", |_| {});
            panel.spawn((
                KitText::new("", ui_kit::TEXT).shadow(ui_kit::INK),
                Node {
                    margin: UiRect::new(px(0), px(0), px(4), px(3)),
                    ..default()
                },
                BenchLine,
            ));
            panel
                .spawn((
                    bar,
                    Node {
                        width: percent(100),
                        height: px(10),
                        flex_shrink: 0.0,
                        padding: UiRect::new(px(5), px(5), px(3), px(4)),
                        ..default()
                    },
                ))
                .with_children(|bar| {
                    bar.spawn((
                        fill,
                        Node {
                            width: percent(0),
                            height: percent(100),
                            ..default()
                        },
                        BenchFill,
                    ));
                });
        });
}

/// The recipe a widget shows: the hands' picked one, or the workbench's.
fn picked<'a>(recipes: &'a Recipes, pick: &CraftPick, station: &str) -> Option<&'a Recipe> {
    let k = if station == "hand" {
        pick.hand
    } else {
        pick.bench
    };
    recipes.0.at(station).nth(k)
}

/// Clicks: the drop-down and its options, a workbench well, and CRAFT.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn press(
    screen: Res<Screen>,
    at: Res<StationAt>,
    recipes: Res<Recipes>,
    sun: Res<pbd_app::sky::Sun>,
    time: Res<Time>,
    mut pick: ResMut<CraftPick>,
    mut save: ResMut<WorldSave>,
    mut hotbar: ResMut<Hotbar>,
    tools: Res<ToolSlot>,
    mut note: ResMut<FarmNote>,
    dropdown: Query<&Interaction, (Changed<Interaction>, With<DropDown>)>,
    options: Query<(&Interaction, &DropOption), (Changed<Interaction>, With<Button>)>,
    wells: Query<(&Interaction, &Well), Changed<Interaction>>,
    buttons: Query<(&Interaction, &CraftButton, &Plaque), Changed<Interaction>>,
) {
    if *screen != Screen::Pack && *screen != Screen::Forge {
        if pick.list_open {
            pick.list_open = false;
        }
        return;
    }
    for interaction in &dropdown {
        if *interaction == Interaction::Pressed {
            pick.list_open = !pick.list_open;
        }
    }
    for (interaction, option) in &options {
        if *interaction == Interaction::Pressed && pick.list_open {
            pick.hand = option.0;
            pick.list_open = false;
        }
    }
    for (interaction, well) in &wells {
        if let (Interaction::Pressed, Role::Pick(k)) = (*interaction, well.role) {
            pick.bench = k;
        }
    }
    for (interaction, button, plaque) in &buttons {
        if *interaction != Interaction::Pressed || plaque.disabled {
            continue;
        }
        let (id, station) = if button.0 == "hand" {
            (HAND_RECORD_ID, "hand")
        } else {
            let Some(cell) = at.0 else { continue };
            (station_id(Some(cell)), "workbench")
        };
        let Some(code) = picked(&recipes, &pick, station).map(|r| r.code.clone()) else {
            continue;
        };
        queue(
            &mut save,
            &mut hotbar,
            Some(&tools),
            &recipes,
            id,
            &code,
            sun.clock.seconds,
            &mut note,
            time.elapsed_secs(),
        );
    }
}

/// The hands' jobs land in the slots when done, whether the pack is open or
/// not, a few times a second.
#[allow(clippy::too_many_arguments)]
pub fn collect_hands(
    recipes: Res<Recipes>,
    sun: Res<pbd_app::sky::Sun>,
    time: Res<Time>,
    mut save: ResMut<WorldSave>,
    mut hotbar: ResMut<Hotbar>,
    mut tools: ResMut<ToolSlot>,
    mut note: ResMut<FarmNote>,
    mut gained: MessageWriter<Gained>,
    mut since: Local<f32>,
) {
    *since += time.delta_secs();
    if *since < 0.2 {
        return;
    }
    *since = 0.0;
    collect(
        &mut save,
        &mut hotbar,
        &mut tools,
        &recipes,
        HAND_RECORD_ID,
        station::hand(),
        sun.clock.seconds,
        &mut note,
        time.elapsed_secs(),
        &mut gained,
    );
}

/// Paint one well's icon.
fn paint_well(
    image: &mut ImageNode,
    item: Option<(Item, u16, Option<Tier>)>,
    icons: &ItemIcons,
    images: &Assets<Image>,
) {
    match item {
        Some((Item::Tool(tool), _, Some(tier))) => {
            let want = icons.tool_at(tool, tier);
            image.image = want;
            image.rect = None;
            image.color = Color::WHITE;
        }
        Some((item, n, _)) => slots::paint_icon(image, Some(Stack::new(item, n)), icons, images),
        None => slots::paint_icon(image, None, icons, images),
    }
}

/// Paint both widgets from the picked recipes, the slots and the queues.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn paint(
    (screen, at, pick): (Res<Screen>, Res<StationAt>, Res<CraftPick>),
    (recipes, save, hotbar, tools, sun): (
        Res<Recipes>,
        Res<WorldSave>,
        Res<Hotbar>,
        Res<ToolSlot>,
        Res<pbd_app::sky::Sun>,
    ),
    icons: Option<Res<ItemIcons>>,
    images: Res<Assets<Image>>,
    kit: Res<UiKit>,
    mut wells: Query<(&Well, &Children, &mut ImageNode, &mut Tooltip), Without<WellIcon>>,
    mut well_icons: Query<&mut ImageNode, (With<WellIcon>, Without<Well>)>,
    mut counts: Query<&mut KitText, (With<WellCount>, Without<CraftLabel>, Without<DropLabel>)>,
    mut buttons: Query<(&CraftButton, &mut Plaque)>,
    mut labels: Query<(&CraftLabel, &mut KitText), (Without<WellCount>, Without<DropLabel>)>,
    mut drop: (
        Query<&mut KitText, (With<DropLabel>, Without<WellCount>, Without<CraftLabel>)>,
        Query<
            (&mut ImageNode, Option<&DropOption>),
            (With<DropIcon>, Without<Well>, Without<WellIcon>),
        >,
        Query<&mut Node, With<DropList>>,
    ),
    mut bench_line: (
        Query<
            &mut KitText,
            (
                With<BenchLine>,
                Without<WellCount>,
                Without<CraftLabel>,
                Without<DropLabel>,
            ),
        >,
        Query<&mut Node, (With<BenchFill>, Without<DropList>)>,
    ),
) {
    if *screen != Screen::Pack && *screen != Screen::Forge {
        return;
    }
    let Some(icons) = icons else { return };
    let now_s = sun.clock.seconds;
    let carried = |item: Item| hotbar.count(item);
    // Per station: the recipe shown, the queue, and whether CRAFT works.
    let view = |station: &str| {
        let recipe = picked(&recipes, &pick, station)?;
        let id = if station == "hand" {
            HAND_RECORD_ID
        } else {
            station_id(at.0)
        };
        let state = state_of(&save, id, recipe.station);
        let short = recipe
            .inputs
            .iter()
            .any(|&(item, n)| carried(item) < u32::from(n));
        let mut probe = state.clone();
        let mut slots = hotbar.0.clone();
        let refused = probe
            .queue_for(&mut slots, Some(&tools.0), Some(&recipes.0), recipe, now_s)
            .is_err();
        Some((recipe, state, short || refused))
    };
    let hand = view("hand");
    let bench = if *screen == Screen::Forge {
        view("workbench")
    } else {
        None
    };
    let of = |station: &str| if station == "hand" { &hand } else { &bench };
    for (well, children, mut piece, mut tip) in &mut wells {
        let (item, lit) = match well.role {
            Role::Pick(k) => {
                let recipe = recipes.0.at(well.station).nth(k);
                let item = recipe.and_then(shown);
                let chosen = k == pick.bench;
                (item, if chosen { "slot-selected" } else { "slot-idle" })
            }
            Role::Need(k) => match of(well.station) {
                Some((recipe, _, _)) => match recipe.inputs.get(k) {
                    Some(&(item, n)) => (
                        Some((item, n, None)),
                        if carried(item) >= u32::from(n) {
                            "slot-hover"
                        } else {
                            "slot-disabled"
                        },
                    ),
                    None => (None, "slot-disabled"),
                },
                None => (None, "slot-disabled"),
            },
            Role::Result => match of(well.station) {
                Some((recipe, _, blocked)) => (
                    shown(recipe),
                    if *blocked {
                        "slot-idle"
                    } else {
                        "slot-selected"
                    },
                ),
                None => (None, "slot-idle"),
            },
        };
        let want = kit.image(lit);
        if piece.image != want {
            piece.image = want;
        }
        let words = match (well.role, item) {
            (_, None) => String::new(),
            (Role::Need(_), Some((item, n, _))) => {
                format!("{}: need {n}, have {}", item.name(), carried(item))
            }
            (_, Some((Item::Tool(tool), _, Some(tier)))) => {
                pbd_core::inventory::tool_title(tool, tier)
            }
            (_, Some((item, _, _))) => item.name().to_string(),
        };
        if tip.0 != words {
            tip.0 = words;
        }
        let number = match item {
            Some((_, n, _)) if n > 1 => n.to_string(),
            _ => String::new(),
        };
        for child in children.iter() {
            if let Ok(mut image) = well_icons.get_mut(child) {
                paint_well(&mut image, item, &icons, &images);
            }
            if let Ok(mut text) = counts.get_mut(child)
                && text.text != number
            {
                text.text.clone_from(&number);
            }
        }
    }
    for (button, mut plaque) in &mut buttons {
        let disabled = of(button.0).as_ref().is_none_or(|(_, _, blocked)| *blocked);
        if plaque.disabled != disabled {
            plaque.disabled = disabled;
        }
    }
    // The hands' plaque counts a running job up.
    for (label, mut text) in &mut labels {
        let want = match of(label.0) {
            Some((_, state, _)) if label.0 == "hand" && state.busy() => {
                let p = state.head_progress(&recipes.0, now_s).unwrap_or(0.0);
                format!("{:.0}%", (p * 100.0).min(99.0))
            }
            _ => "CRAFT".to_string(),
        };
        if text.text != want {
            text.text = want;
        }
    }
    let (drop_label, drop_icons, drop_list) = &mut drop;
    let hand_recipes: Vec<&Recipe> = recipes.0.at("hand").collect();
    if let Some((recipe, _, _)) = &hand {
        for mut text in drop_label.iter_mut() {
            let want = label(recipe);
            if text.text != want {
                text.text = want;
            }
        }
    }
    for (mut image, option) in drop_icons.iter_mut() {
        let k = option.map_or(pick.hand, |o| o.0);
        let item = hand_recipes.get(k).and_then(|r| shown(r));
        paint_well(&mut image, item, &icons, &images);
        if let Some(recipe) = hand_recipes.get(k)
            && option.is_some()
            && recipe
                .inputs
                .iter()
                .any(|&(i, n)| carried(i) < u32::from(n))
        {
            // Short of something: dimmed.
            image.color = image.color.with_alpha(0.45);
        }
    }
    for mut node in drop_list.iter_mut() {
        let want = if pick.list_open && *screen == Screen::Pack {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != want {
            node.display = want;
        }
    }
    let (line, fill) = &mut bench_line;
    if let Some((_, state, _)) = &bench {
        let text = match state.jobs.first().and_then(|j| recipes.0.get(&j.recipe)) {
            Some(recipe) => {
                let waiting = state.jobs.len() - 1;
                if waiting > 0 {
                    format!("{}  +{waiting} WAITING", recipe.name)
                } else {
                    recipe.name.clone()
                }
            }
            None => "PICK A RECIPE".into(),
        };
        for mut words in line.iter_mut() {
            if words.text != text {
                words.text.clone_from(&text);
            }
        }
        let want = percent(state.head_progress(&recipes.0, now_s).unwrap_or(0.0) * 100.0);
        for mut node in fill.iter_mut() {
            if node.width != want {
                node.width = want;
            }
        }
    }
}

// ---------------------------------------------------------------- capture rig

/// `--craft pack|bench|bench-view [metres]` (`tool-crafting` task 4.2): a
/// measurement instrument for the headless captures, which have no hands.
/// Through the same durable paths a player's clicks take: `pack` stocks the
/// slots, queues planks by hand part done and opens the pack on the
/// workbench recipe; `bench` places a workbench that far ahead, stocks the
/// slots, queues a stone pickaxe half done and opens its panel on the iron
/// pickaxe; `bench-view` only places the workbench. The capture's save is
/// memory-only, so nothing is kept.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn scripted(
    launch: Res<super::Launch>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    refresh: Res<pbd_app::planet::LodRefresh>,
    (mut fine, mut contact, mut save, mut slots, mut drops, sun): (
        ResMut<pbd_app::planet::PlanetFine>,
        ResMut<pbd_app::planet::PlanetContact>,
        ResMut<WorldSave>,
        ResMut<Hotbar>,
        ResMut<pbd_app::drops::Drops>,
        Res<pbd_app::sky::Sun>,
    ),
    (recipes, mut station_at, mut screen, mut note, time, mut pick, tools): (
        Res<Recipes>,
        ResMut<StationAt>,
        ResMut<Screen>,
        ResMut<FarmNote>,
        Res<Time>,
        ResMut<CraftPick>,
        Res<ToolSlot>,
    ),
    mut done: Local<bool>,
) {
    use pbd_core::goods::Good;
    use pbd_core::terrain::Material;
    let Some((scene, metres)) = launch.craft.clone() else {
        return;
    };
    if *done || launch.capture.is_none() || refresh.in_flight_s().is_some() {
        return;
    }
    let Some((transform, _)) = cameras.iter().find(|(_, c)| c.is_active) else {
        return;
    };
    let now_s = sun.clock.seconds;
    let wall_s = time.elapsed_secs();
    let index_of = |station: &str, code: &str| {
        recipes
            .0
            .at(station)
            .position(|r| r.code == code)
            .unwrap_or(0)
    };
    if scene == "pack" {
        *done = true;
        slots.give(Item::Good(Good::LOG), 2);
        slots.give(Item::Good(Good::FIBRE), 4);
        slots.give(Item::Good(Good::PLANKS), 6);
        slots.give(Item::Good(Good::LOOSE_STONE), 2);
        queue(
            &mut save,
            &mut slots,
            Some(&tools),
            &recipes,
            HAND_RECORD_ID,
            "planks",
            now_s - 0.4,
            &mut note,
            wall_s,
        );
        pick.hand = index_of("hand", "workbench");
        *screen = Screen::Pack;
        info!("scripted craft: the pack, planks part made by hand");
        return;
    }
    let eye = transform.translation();
    let up = eye.normalize_or(Vec3::Y);
    let forward = transform.forward().as_vec3();
    let ahead = (forward - up * forward.dot(up)).normalize_or(Vec3::X);
    let angle = metres / pbd_app::planet::PLANET_RADIUS;
    let direction = (up * angle.cos() + ahead * angle.sin()).normalize();
    let Some(record) = contact.finest_cell(direction) else {
        return;
    };
    let Some(cell) = fine.set.finest_records().get(record).map(|r| r.key()) else {
        return;
    };
    let Some(top) = fine.set.columns.column(record).and_then(|c| c.surface()) else {
        return;
    };
    *done = true;
    let layer = top + 1;
    super::digging::apply_edit(
        &mut super::digging::Edited {
            fine: &mut fine,
            contact: &mut contact,
            save: &mut save,
            slots: &mut slots,
            drops: &mut drops,
            now_s,
        },
        super::digging::Hands::Empty,
        cell,
        layer,
        Material::Workbench,
    );
    info!("scripted craft: a workbench at cell {cell} layer {layer}");
    if scene != "bench" {
        return;
    }
    slots.give(Item::Good(Good::LOOSE_STONE), 9);
    slots.give(Item::Good(Good::PLANKS), 8);
    slots.give(Item::Good(Good::ROPE), 3);
    slots.give(Item::Good(Good::IRON_INGOT), 3);
    let at = (cell, layer as u16);
    queue(
        &mut save,
        &mut slots,
        Some(&tools),
        &recipes,
        station_id(Some(at)),
        "stone-pickaxe",
        now_s - 1.5,
        &mut note,
        wall_s,
    );
    pick.bench = index_of("workbench", "iron-pickaxe");
    station_at.0 = Some(at);
    *screen = Screen::Forge;
}
