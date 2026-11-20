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
//! - **The pack's Craft panel**, at the pack's right (decision 12): what the
//!   hands make as a list of plaques (a recipe short of something has its
//!   icon dimmed, a locked one is a disabled plaque with a silhouette), and
//!   under it the chosen recipe's row: the wells of what it takes lit or dark
//!   against what is carried, an arrow, the result, and CRAFT. The hands
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

/// Which recipe each widget has picked.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CraftPick {
    /// Into the hands' recipes, in file order; none until a row is clicked.
    pub hand: Option<usize>,
    /// Into the workbench's.
    pub bench: usize,
}

/// The number on a well: an input always shows how many it takes, one
/// included; a made thing or a stack shows a number only above one
/// (`pack-tool-slot` decision 1).
pub(super) fn need_count(need: bool, n: u16) -> String {
    if need || n > 1 {
        n.to_string()
    } else {
        String::new()
    }
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

/// A row of the pack's recipe list: the hands' `k`th recipe.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecipeRow(pub usize);
/// A recipe row's picture.
#[derive(Component)]
pub struct RowIcon(pub usize);

/// The Craft panel's width in the pack, native px (decision 12).
pub const CRAFT_COLUMN: f32 = 180.0;
/// A recipe row's height: the stone plaque's own (`button-idle`, 22 px).
pub const ROW: f32 = 22.0;
/// The rows shown before the list scrolls.
#[allow(dead_code)]
pub const ROWS_SHOWN: usize = 6;
/// The CRAFT plaque's width, wide enough for its longest word (LOCKED), so
/// it never grows past the row.
pub const CRAFT_PLAQUE: f32 = 56.0;

/// The workbench panel's queue line and bar.
#[derive(Component)]
pub struct BenchLine;
#[derive(Component)]
pub struct BenchFill;
/// The bar under the pack's craft row: the hands' running job
/// (`mission-playthrough` 4.4; the pack had only the plaque's number).
#[derive(Component)]
pub struct HandFill;

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

/// What a recipe's row says: its name, and the count it makes ("X4") at the
/// row's end. A name that already ends in its count ("TORCH X2") gives it
/// to the count rather than saying it twice.
fn row_words(recipe: &Recipe) -> (String, String) {
    match shown(recipe) {
        Some((_, n, _)) if n > 1 => {
            let count = format!("X{n}");
            let name = recipe
                .name
                .strip_suffix(&format!(" {count}"))
                .unwrap_or(&recipe.name)
                .to_string();
            (name, count)
        }
        _ => (recipe.name.clone(), String::new()),
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
            note.say(wall_s, "You already have this tool");
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
    did: &mut MessageWriter<pbd_app::missions::Did>,
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
        // What the starter missions count: this recipe made (a tool's tier
        // is in its recipe's code, not in the `Gained` item).
        did.write(pbd_app::missions::Did::new(
            pbd_core::missions::Kind::Craft,
            format!("recipe:{}", job.recipe),
            1,
        ));
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

/// The row both widgets share: `needs` wells, an arrow, the result, and
/// CRAFT.
fn craft_row(
    parent: &mut ChildSpawnerCommands,
    kit: &UiKit,
    atlas: &Handle<Image>,
    station: &'static str,
    needs: usize,
) {
    parent
        .spawn(Node {
            align_items: AlignItems::Center,
            column_gap: px(3),
            margin: UiRect::top(px(3)),
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|row| {
            for k in 0..needs {
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
            row.spawn((
                ui_kit::plaque(kit, Set::Stone, CRAFT_PLAQUE),
                CraftButton(station),
            ))
            .with_children(|p| {
                p.spawn((
                    ui_kit::plaque_label(Set::Stone, "CRAFT"),
                    CraftLabel(station),
                ));
            });
        });
}

/// The pack's Craft panel (decision 12), the pack's right-hand column: the
/// CRAFT lintel, the hands' recipes as a list of stone plaques, and the
/// chosen recipe's row at the column's foot, level with the hotbar.
pub fn spawn_pack_craft(
    panel: &mut ChildSpawnerCommands,
    kit: &UiKit,
    atlas: &Handle<Image>,
    recipes: &Recipes,
) {
    let hand: Vec<&Recipe> = recipes.0.at("hand").collect();
    let needs = hand.iter().map(|r| r.inputs.len()).max().unwrap_or(1);
    panel
        .spawn(Node {
            width: px(CRAFT_COLUMN),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            ..default()
        })
        .with_children(|column| {
            column
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    // Never shorter than its list: a scroll box may shrink to
                    // nothing, and the craft row then slid up over the rows.
                    flex_shrink: 0.0,
                    ..default()
                })
                .with_children(|top| {
                    super::menu::lintel(top, kit, "CRAFT");
                    // Six rows show; a longer list scrolls with the wheel.
                    top.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: px(1),
                            // A scroll box has no height of its own: it is
                            // set to the rows it shows.
                            // Every recipe shows (`mission-playthrough` 4.4: its
                            // own panel, as tall as its list). The rows are
                            // placed in it, so it is given its height.
                            height: px(list_height(hand.len())),
                            min_height: px(list_height(hand.len())),
                            flex_shrink: 0.0,
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                        ScrollPosition::default(),
                        super::menu::WheelScrolls,
                    ))
                    .with_children(|list| {
                        for (k, recipe) in hand.iter().enumerate() {
                            recipe_row(list, kit, atlas, k, recipe);
                        }
                    });
                });
            // The chosen recipe's row and the hands' bar, placed under the
            // list at its own height (`mission-playthrough` 4.4): left to
            // the column's flow, the row was drawn over the recipes.
            column
                .spawn(Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(5.0 + list_height(hand.len()) + 6.0),
                    width: percent(100),
                    flex_direction: FlexDirection::Column,
                    ..default()
                })
                .with_children(|foot| {
                    craft_row(foot, kit, atlas, "hand", needs);
                    // The hands' running job, as the workbench shows its own.
                    foot.spawn((
                        kit.nine(Set::Stone, "bar"),
                        Node {
                            width: percent(100),
                            height: px(10),
                            flex_shrink: 0.0,
                            margin: UiRect::top(px(4)),
                            padding: UiRect::new(px(5), px(5), px(3), px(4)),
                            ..default()
                        },
                    ))
                    .with_children(|bar| {
                        bar.spawn((
                            kit.whole(Set::Stone, "fill-progress"),
                            Node {
                                width: percent(0),
                                height: percent(100),
                                ..default()
                            },
                            HandFill,
                        ));
                    });
                });
        });
}

/// The recipe list's height for `rows` recipes, native px: every row up to
/// [`ROWS_SHOWN`], each `ROW` tall with a pixel between.
pub fn list_height(rows: usize) -> f32 {
    let shown = rows.clamp(1, ROWS_SHOWN) as f32;
    ROW * shown + (shown - 1.0)
}

/// One recipe in the pack's list: a stone plaque the column's width, the
/// recipe's icon at its native 16 px, its name, and its count at the end.
fn recipe_row(
    list: &mut ChildSpawnerCommands,
    kit: &UiKit,
    atlas: &Handle<Image>,
    k: usize,
    recipe: &Recipe,
) {
    let (name, count) = row_words(recipe);
    list.spawn((
        Button,
        kit.nine(Set::Stone, "button-idle"),
        ui_kit::KitPiece(Set::Stone.key("button-idle")),
        Plaque::new(Set::Stone),
        Tooltip::default(),
        Node {
            width: px(CRAFT_COLUMN),
            height: px(ROW),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            column_gap: px(4),
            padding: UiRect::new(px(4), px(6), px(0), px(0)),
            ..default()
        },
        RecipeRow(k),
    ))
    .with_children(|row| {
        row.spawn((
            ImageNode {
                image: atlas.clone(),
                color: Color::NONE,
                ..default()
            },
            Node {
                width: px(16),
                height: px(16),
                flex_shrink: 0.0,
                ..default()
            },
            RowIcon(k),
        ));
        row.spawn(ui_kit::plaque_label(Set::Stone, &name));
        row.spawn(Node {
            flex_grow: 1.0,
            ..default()
        });
        row.spawn(ui_kit::plaque_label(Set::Stone, &count));
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
            craft_row(panel, kit, atlas, "workbench", 3);
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
        pick.hand?
    } else {
        pick.bench
    };
    recipes.0.at(station).nth(k)
}

/// Clicks: a recipe row of the pack, a workbench well, and CRAFT. A locked
/// row picks nothing and names the mission that opens it.
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
    rows: Query<(&Interaction, &RecipeRow), Changed<Interaction>>,
    wells: Query<(&Interaction, &Well), Changed<Interaction>>,
    buttons: Query<(&Interaction, &CraftButton, &Plaque), Changed<Interaction>>,
    locks: Res<pbd_app::missions::RecipeLocks>,
) {
    if *screen != Screen::Pack && *screen != Screen::Forge {
        return;
    }
    for (interaction, row) in &rows {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(recipe) = recipes.0.at("hand").nth(row.0) else {
            continue;
        };
        match locks.locked(&recipe.code) {
            Some(opens) => note.say(time.elapsed_secs(), format!("Opens with: {opens}")),
            None => pick.hand = Some(row.0),
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
        // Locked by the mission tree (`starter-missions` T.1): it says
        // which mission opens it.
        if let Some(opens) = locks.locked(&code) {
            note.say(time.elapsed_secs(), format!("Opens with: {opens}"));
            continue;
        }
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
    mut did: MessageWriter<pbd_app::missions::Did>,
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
        &mut did,
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

/// The tint a locked recipe's pictures take: a dim silhouette
/// (`starter-missions` T.4).
pub const SILHOUETTE: Color = Color::srgba(0.02, 0.07, 0.12, 0.85);

/// An icon in a station's recipe row, by its recipe: drawn as a silhouette
/// while the mission tree holds the recipe shut.
#[derive(Component)]
pub struct LockTint(pub String);

/// The line at a recipe row's end that names the mission opening it.
#[derive(Component)]
pub struct LockLine(pub String);

/// The forge's and the campfire's locked rows: their pictures dimmed to a
/// silhouette and the mission that opens them named. Runs after their own
/// paints, which set the pictures each frame.
pub fn paint_locks(
    screen: Res<Screen>,
    locks: Res<pbd_app::missions::RecipeLocks>,
    mut tints: Query<(&LockTint, &mut ImageNode)>,
    mut lines: Query<(&LockLine, &mut KitText, &mut Node)>,
) {
    if !matches!(*screen, Screen::Forge | Screen::Kitchen) {
        return;
    }
    for (tint, mut image) in &mut tints {
        if locks.locked(&tint.0).is_some() && image.color != SILHOUETTE {
            image.color = SILHOUETTE;
        }
    }
    for (line, mut text, mut node) in &mut lines {
        let words = locks
            .locked(&line.0)
            .map(|m| format!("OPENS WITH: {}", m.to_uppercase()));
        let display = if words.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
        let words = words.unwrap_or_default();
        if text.text != words {
            text.text = words;
        }
    }
}

/// Paint both widgets from the picked recipes, the slots and the queues.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn paint(
    (screen, at, pick, locks): (
        Res<Screen>,
        Res<StationAt>,
        Res<CraftPick>,
        Res<pbd_app::missions::RecipeLocks>,
    ),
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
    mut counts: Query<&mut KitText, (With<WellCount>, Without<CraftLabel>)>,
    mut buttons: Query<(&CraftButton, &mut Plaque)>,
    mut labels: Query<(&CraftLabel, &mut KitText), Without<WellCount>>,
    mut rows: (
        Query<(&RecipeRow, &mut Plaque, &mut Tooltip), (Without<Well>, Without<CraftButton>)>,
        Query<(&RowIcon, &mut ImageNode), (Without<Well>, Without<WellIcon>)>,
    ),
    mut bench_line: (
        Query<&mut KitText, (With<BenchLine>, Without<WellCount>, Without<CraftLabel>)>,
        Query<&mut Node, (With<BenchFill>, Without<HandFill>)>,
        Query<&mut Node, (With<HandFill>, Without<BenchFill>)>,
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
        let locked = locks.locked(&recipe.code).is_some();
        Some((recipe, state, short || refused || locked))
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
                    None => (None, "slot-idle"),
                },
                None => (None, "slot-idle"),
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
        // The recipe this well shows, when the mission tree holds it shut.
        let shut = match well.role {
            Role::Pick(k) => recipes.0.at(well.station).nth(k),
            Role::Result => of(well.station).as_ref().map(|(r, _, _)| *r),
            Role::Need(_) => None,
        }
        .and_then(|r| locks.locked(&r.code));
        let words = match (well.role, item) {
            (Role::Pick(_) | Role::Result, Some(_)) if shut.is_some() => {
                format!("Opens with: {}", shut.unwrap_or_default())
            }
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
            Some((_, n, _)) => need_count(matches!(well.role, Role::Need(_)), n),
            None => String::new(),
        };
        for child in children.iter() {
            if let Ok(mut image) = well_icons.get_mut(child) {
                paint_well(&mut image, item, &icons, &images);
                if shut.is_some() && item.is_some() {
                    image.color = SILHOUETTE;
                }
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
    // The hands' plaque counts a running job up; a locked recipe's says so.
    for (label, mut text) in &mut labels {
        let want = match of(label.0) {
            Some((recipe, _, _)) if locks.locked(&recipe.code).is_some() => "LOCKED".to_string(),
            // The bar under the row shows how far; the word says what a press
            // does now: queue another, with how many already wait.
            Some((_, state, _)) if label.0 == "hand" && state.busy() => {
                match state.jobs.len() - 1 {
                    0 => "QUEUE".to_string(),
                    n => format!("QUEUE +{n}"),
                }
            }
            _ => "CRAFT".to_string(),
        };
        if text.text != want {
            text.text = want;
        }
    }
    // The pack's list: the chosen row ringed with the rune (the focus
    // plaque, as a chosen slot wears it), a locked one dead with its
    // picture a silhouette and the mission that opens it in its tooltip, one
    // short of something with its picture dimmed.
    let (row_plaques, row_icons) = &mut rows;
    let hand_recipes: Vec<&Recipe> = recipes.0.at("hand").collect();
    for (row, mut plaque, mut tip) in row_plaques.iter_mut() {
        let shut = hand_recipes.get(row.0).and_then(|r| locks.locked(&r.code));
        let chosen = pick.hand == Some(row.0) && shut.is_none();
        if plaque.disabled != shut.is_some() || plaque.focus != chosen {
            plaque.disabled = shut.is_some();
            plaque.focus = chosen;
        }
        let words = shut.map(|m| format!("Opens with: {m}")).unwrap_or_default();
        if tip.0 != words {
            tip.0 = words;
        }
    }
    for (row, mut image) in row_icons.iter_mut() {
        let recipe = hand_recipes.get(row.0);
        paint_well(&mut image, recipe.and_then(|r| shown(r)), &icons, &images);
        if let Some(recipe) = recipe {
            if locks.locked(&recipe.code).is_some() {
                image.color = SILHOUETTE;
            } else if recipe
                .inputs
                .iter()
                .any(|&(i, n)| carried(i) < u32::from(n))
            {
                image.color = image.color.with_alpha(0.45);
            }
        }
    }
    let (line, fill, hand_fill) = &mut bench_line;
    // The hands' bar: the head job's share done, empty when idle.
    let hand_state = state_of(&save, HAND_RECORD_ID, pbd_core::station::hand());
    let done = hand_state.head_progress(&recipes.0, now_s).unwrap_or(0.0);
    let want = percent(if hand_state.busy() { done * 100.0 } else { 0.0 });
    for mut node in hand_fill.iter_mut() {
        if node.width != want {
            node.width = want;
        }
    }
    if let Some((_, state, _)) = &bench {
        let shut = picked(&recipes, &pick, "workbench").and_then(|r| locks.locked(&r.code));
        let text = match state.jobs.first().and_then(|j| recipes.0.get(&j.recipe)) {
            _ if state.jobs.is_empty() && shut.is_some() => {
                format!("OPENS WITH: {}", shut.unwrap_or_default().to_uppercase())
            }
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

/// `--craft pack|pack-empty|pack-pick|pack-locked|bench|bench-view [metres]`
/// (`tool-crafting` tasks 4.2 and 5.5): a measurement instrument for the
/// headless captures, which have no hands. Through the same durable paths a
/// player's clicks take: `pack` stocks the slots, queues planks by hand part
/// done and opens the pack on the workbench recipe; `pack-empty` stocks the
/// slots and opens the pack with nothing chosen; `pack-pick` opens it on
/// planks; `pack-locked` does too and holds the pointer over the stone
/// axe's row ([`hover_locked`]); `bench` places a workbench that far ahead, stocks the
/// slots, queues a stone pickaxe half done and opens its panel on the iron
/// pickaxe; `bench-view` only places the workbench. The capture's save is
/// memory-only, so nothing is kept.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn scripted(
    launch: Res<super::Launch>,
    (cameras, frame): (
        Query<(&GlobalTransform, &Camera), With<Camera3d>>,
        Option<Res<pbd_app::planet::PlanetRenderFrame>>,
    ),
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
    if scene.starts_with("pack") {
        *done = true;
        slots.give(Item::Good(Good::LOG), 2);
        slots.give(Item::Good(Good::FIBRE), 4);
        slots.give(Item::Good(Good::PLANKS), 6);
        slots.give(Item::Good(Good::LOOSE_STONE), 2);
        *screen = Screen::Pack;
        if scene != "pack" {
            pick.hand = (scene != "pack-empty").then(|| index_of("hand", "planks"));
            info!("scripted craft: the pack, {scene}");
            return;
        }
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
        pick.hand = Some(index_of("hand", "workbench"));
        info!("scripted craft: the pack, planks part made by hand");
        return;
    }
    let eye = pbd_app::render_origin::local_of(frame.as_deref(), transform.translation());
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

/// `--craft pack-locked`'s pointer: the stone axe's row held hovered after
/// the UI's focus pass, as a pointer over it would, so its tooltip shows in
/// a capture (which has no pointer). A capture instrument (task 5.5).
pub fn hover_locked(
    launch: Res<super::Launch>,
    recipes: Res<Recipes>,
    mut rows: Query<(&RecipeRow, &mut Interaction)>,
) {
    if !launch
        .craft
        .as_ref()
        .is_some_and(|(scene, _)| scene == "pack-locked")
    {
        return;
    }
    let Some(k) = recipes.0.at("hand").position(|r| r.code == "stone-axe") else {
        return;
    };
    for (row, mut interaction) in &mut rows {
        if row.0 == k && *interaction != Interaction::Hovered {
            *interaction = Interaction::Hovered;
        }
    }
}

#[cfg(test)]
mod tests {

    /// An input shows its count, one included; a result or a stack of one
    /// shows none (`pack-tool-slot` decision 1: the stone axe's log).
    #[test]
    fn an_input_of_one_shows_its_count() {
        assert_eq!(super::need_count(true, 1), "1");
        assert_eq!(super::need_count(true, 3), "3");
        assert_eq!(super::need_count(false, 1), "");
        assert_eq!(super::need_count(false, 2), "2");
    }

    use super::*;

    fn book() -> Recipes {
        pbd_app::config::recipe_book().0
    }

    fn hand_index(recipes: &Recipes, code: &str) -> usize {
        recipes.0.at("hand").position(|r| r.code == code).unwrap()
    }

    /// A row says the name once and the count at its end: "TORCH X2" gives
    /// its X2 to the count, planks gains one, the workbench has none.
    #[test]
    fn a_row_names_its_recipe_and_counts_once() {
        let recipes = book();
        let words = |code: &str| row_words(recipes.0.get(code).unwrap());
        assert_eq!(words("torch"), ("TORCH".into(), "X2".into()));
        assert_eq!(words("planks"), ("PLANKS".into(), "X4".into()));
        assert_eq!(words("workbench"), ("WORKBENCH".into(), String::new()));
    }

    /// Decision 12: every hand recipe shows at today's count, and the
    /// lintel, the list and the craft row stand inside the pack's rock; the
    /// craft row at its widest (the most inputs, LOCKED on the plaque) fits
    /// the column. In whole native pixels.
    #[test]
    fn the_craft_panel_fits_inside_the_rock() {
        let mut app = App::new();
        ui_kit::install_bare(&mut app);
        let kit = app.world().resource::<UiKit>();
        let recipes = book();
        let hand = recipes.0.at("hand").count();
        // Its own panel now, as tall as it needs (`mission-playthrough`
        // 4.4): every recipe shows without scrolling.
        assert!(hand <= ROWS_SHOWN, "{hand} hand recipes: the list scrolls");
        let needs = recipes.0.at("hand").map(|r| r.inputs.len()).max().unwrap() as f32;
        let arrow = kit.font.width("→") as f32;
        let row = needs * slots::SLOT + arrow + slots::SLOT + CRAFT_PLAQUE + 3.0 * (needs + 2.0);
        assert!(row <= CRAFT_COLUMN, "the craft row is {row} px");
        let locked = kit.font.width("LOCKED") as f32 + 20.0;
        assert!(locked <= CRAFT_PLAQUE, "LOCKED needs {locked} px");
        for size in [ROW, CRAFT_COLUMN, list_height(hand), CRAFT_PLAQUE] {
            assert_eq!(size.fract(), 0.0, "whole pixels");
        }
    }

    /// A click on an open row chooses it; a click on a locked row chooses
    /// nothing and names the mission that opens it.
    #[test]
    fn a_locked_row_picks_nothing() {
        let recipes = book();
        let axe = hand_index(&recipes, "stone-axe");
        let planks = hand_index(&recipes, "planks");
        let mut locks = pbd_app::missions::RecipeLocks::default();
        locks.0.insert("stone-axe".into(), "Gather by hand".into());
        let mut app = App::new();
        app.insert_resource(Screen::Pack)
            .insert_resource(recipes)
            .insert_resource(locks)
            .insert_resource(WorldSave::memory_only())
            .init_resource::<StationAt>()
            .init_resource::<pbd_app::sky::Sun>()
            .init_resource::<Time>()
            .init_resource::<CraftPick>()
            .init_resource::<Hotbar>()
            .init_resource::<ToolSlot>()
            .init_resource::<FarmNote>()
            .add_systems(Update, press);
        app.world_mut()
            .spawn((Interaction::Pressed, RecipeRow(axe)));
        app.update();
        assert_eq!(app.world().resource::<CraftPick>().hand, None);
        assert_eq!(
            app.world().resource::<FarmNote>().text,
            "OPENS WITH: GATHER BY HAND"
        );
        app.world_mut()
            .spawn((Interaction::Pressed, RecipeRow(planks)));
        app.update();
        assert_eq!(app.world().resource::<CraftPick>().hand, Some(planks));
    }
}
