//! A station's panel (`farming-and-smelting` decisions 7 and 11): the forge
//! first, and any station after it, since what it lists is the station's
//! recipes from `recipes.ron`.
//!
//! The kit's own pieces: the pack's rock, the oak lintel, a stone plaque a
//! recipe with its inputs and outputs drawn as slot icons beside it, the
//! loading screen's bar for the job being worked, and a line for the queue.
//! Pressing a plaque queues its recipe, taking its inputs (fuel included);
//! while the panel is open, finished jobs land in the slots. Each is one
//! saved line with the slots (`WorldSave::store_with_hands`).

use super::farm::{FarmNote, StationAt, station_state};
use super::menu::{self, Panel, Screen};
use super::slots::{self, ItemIcons};
use bevy::prelude::*;
use pbd_app::config::Recipes;
use pbd_app::saves::WorldSave;
use pbd_app::ui_kit::{self, KitText, Plaque, Set, UiKit};
use pbd_core::inventory::{Item, Stack};
use pbd_core::records::Record as StoredRecord;
use pbd_core::station::{self, STATION_KIND, STATION_SCHEMA, StationState};

/// The forge's recipes are the panel's rows; this marks a row's plaque.
#[derive(Component)]
pub struct RecipePlaque(pub String);

/// An item picture in a recipe row.
#[derive(Component)]
pub struct RecipeIcon(pub Item, pub u16);

/// The queue line and the bar's fill.
#[derive(Component)]
pub struct QueueText;
#[derive(Component)]
pub struct QueueFill;

use pbd_app::hotbar::Gained;
use pbd_app::ui_kit::Localized;

/// A station's own panel inside the station layer, by station code: the one
/// whose station is open is shown (`tool-crafting` decision 9).
#[derive(Component)]
pub struct StationPanel(pub &'static str);

/// The panel. Built for the forge's recipes; another station's panel is the
/// same with its own rows.
pub fn spawn(
    mut commands: Commands,
    icons: Res<ItemIcons>,
    kit: Res<UiKit>,
    recipes: Res<Recipes>,
) {
    let kit = &*kit;
    let (rock, (w, h)) = kit.rock("pack");
    let atlas = icons.atlas.clone();
    let bar = kit.nine(Set::Stone, "bar");
    let fill = kit.whole(Set::Stone, "fill-progress");
    commands
        .spawn((menu::layer(), GlobalZIndex(10), Panel(Screen::Forge)))
        .with_children(|layer| {
            layer
                .spawn((
                    rock,
                    Node {
                        width: px(w),
                        height: px(h),
                        padding: UiRect::all(px(14)),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    StationPanel("forge"),
                ))
                .with_children(|panel| {
                    menu::lintel(panel, kit, "FORGE");
                    for recipe in recipes.0.at("forge") {
                        panel
                            .spawn(Node {
                                align_items: AlignItems::Center,
                                margin: UiRect::bottom(px(4)),
                                column_gap: px(3),
                                ..default()
                            })
                            .with_children(|row| {
                                row.spawn((
                                    ui_kit::plaque(kit, Set::Stone, 74.0),
                                    RecipePlaque(recipe.code.clone()),
                                ))
                                .with_children(|p| {
                                    p.spawn(ui_kit::plaque_label(
                                        Set::Stone,
                                        Localized::legacy(recipe.name.clone()),
                                    ));
                                });
                                let item = |row: &mut ChildSpawnerCommands,
                                            (it, n): (Item, u16),
                                            need: bool| {
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
                                        RecipeIcon(it, n),
                                        super::craft::LockTint(recipe.code.clone()),
                                    ))
                                    .with_children(|icon| {
                                        icon.spawn((
                                            KitText::new(
                                                Localized::legacy(super::craft::need_count(
                                                    need, n,
                                                )),
                                                ui_kit::LABEL,
                                            )
                                            .outline(ui_kit::INK),
                                            Node {
                                                position_type: PositionType::Absolute,
                                                bottom: px(-3),
                                                right: px(-2),
                                                ..default()
                                            },
                                        ));
                                    });
                                };
                                for &input in &recipe.inputs {
                                    item(row, input, true);
                                }
                                row.spawn(
                                    KitText::new(Localized::verbatim("→"), ui_kit::TEXT)
                                        .shadow(ui_kit::INK),
                                );
                                for &output in &recipe.outputs {
                                    item(row, output, false);
                                }
                                row.spawn((
                                    KitText::new(Localized::empty(), ui_kit::LABEL_DIM)
                                        .shadow(ui_kit::INK),
                                    Node {
                                        display: Display::None,
                                        ..default()
                                    },
                                    super::craft::LockLine(recipe.code.clone()),
                                ));
                            });
                    }
                    panel.spawn((
                        KitText::new(Localized::empty(), ui_kit::TEXT).shadow(ui_kit::INK),
                        Node {
                            margin: UiRect::new(px(0), px(0), px(4), px(3)),
                            ..default()
                        },
                        QueueText,
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
                                QueueFill,
                            ));
                        });
                });
            super::craft::spawn_bench(layer, kit, &atlas, &recipes);
        });
}

/// Store a station's queue with the slots after it.
fn store(
    save: &mut WorldSave,
    cell: u32,
    layer: u16,
    state: &StationState,
    slots: &pbd_core::inventory::Slots,
) -> bool {
    let record = StoredRecord::of(
        STATION_KIND,
        station::record_id(cell, layer),
        STATION_SCHEMA,
        state,
    );
    save.store_with_hands(record, STATION_SCHEMA, slots)
}

/// Queue a recipe at the open station: its inputs leave the slots and the job
/// joins the queue, in one saved line. What the capture rig calls too.
#[allow(clippy::too_many_arguments)]
pub fn queue(
    save: &mut WorldSave,
    hotbar: &mut slots::Hotbar,
    recipes: &Recipes,
    (cell, layer): (u32, u16),
    code: &str,
    now_s: f64,
    note: &mut FarmNote,
    wall_s: f32,
) -> bool {
    let Some(recipe) = recipes.0.get(code) else {
        return false;
    };
    let mut state = station_state(save, cell, layer, recipe.station);
    let mut moved = hotbar.0.clone();
    match state.queue(&mut moved, recipe, now_s) {
        Ok(()) => {}
        Err(station::Refused::Short) => {
            note.say(wall_s, format!("Not enough for {}", recipe.name));
            return false;
        }
        Err(station::Refused::Full) => {
            note.say(wall_s, "Forge queue is full");
            return false;
        }
        Err(station::Refused::Elsewhere | station::Refused::NotBetter) => return false,
    }
    if !store(save, cell, layer, &state, &moved) {
        error!("queue BLOCKED: the save did not accept it");
        return false;
    }
    hotbar.0 = moved;
    info!(
        "queued {code} at cell {cell} layer {layer}, {} in the queue",
        state.jobs.len()
    );
    true
}

/// The panel's work: a pressed plaque queues; finished jobs are collected;
/// the rows, the line and the bar follow the queue.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn work(
    screen: Res<Screen>,
    at: Res<StationAt>,
    recipes: Res<Recipes>,
    sun: Res<pbd_app::sky::Sun>,
    time: Res<Time>,
    mut save: ResMut<WorldSave>,
    mut hotbar: ResMut<slots::Hotbar>,
    mut note: ResMut<FarmNote>,
    (mut gained, mut did): (MessageWriter<Gained>, MessageWriter<pbd_app::missions::Did>),
    mut plaques: Query<(&Interaction, &RecipePlaque, &mut Plaque), Changed<Interaction>>,
    mut line: Query<&mut KitText, With<QueueText>>,
    mut fill: Query<&mut Node, (With<QueueFill>, Without<StationPanel>)>,
    mut tools: ResMut<pbd_app::fish::ToolSlot>,
    mut panels: Query<(&StationPanel, &mut Node), Without<QueueFill>>,
    locks: Res<pbd_app::missions::RecipeLocks>,
) {
    if *screen != Screen::Forge {
        return;
    }
    let Some((cell, layer)) = at.0 else { return };
    let now_s = sun.clock.seconds;
    let wall_s = time.elapsed_secs();
    // The open station's panel, and its queue collected while it is open,
    // tools to the tool slot (`tool-crafting`).
    let Some(row) = super::craft::station_row_at(&save, cell, layer) else {
        return;
    };
    for (panel, mut node) in &mut panels {
        let want = if panel.0 == row.code {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != want {
            node.display = want;
        }
    }
    super::craft::collect(
        &mut save,
        &mut hotbar,
        &mut tools,
        &recipes,
        station::record_id(cell, layer),
        row,
        now_s,
        &mut note,
        wall_s,
        &mut gained,
        &mut did,
    );
    if row.code != "forge" {
        return;
    }
    for (interaction, plaque, _) in &mut plaques {
        if *interaction == Interaction::Pressed {
            if let Some(opens) = locks.locked(&plaque.0) {
                note.say(wall_s, format!("Opens with: {opens}"));
                continue;
            }
            queue(
                &mut save,
                &mut hotbar,
                &recipes,
                (cell, layer),
                &plaque.0,
                now_s,
                &mut note,
                wall_s,
            );
        }
    }
    let state = station_state(&save, cell, layer, row);
    let progress = state.head_progress(&recipes.0, now_s);
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
    for mut words in &mut line {
        if words.text != text {
            words.text = Localized::legacy(text.clone());
        }
    }
    let want = percent(progress.unwrap_or(0.0) * 100.0);
    for mut node in &mut fill {
        if node.width != want {
            node.width = want;
        }
    }
}

/// Grey a plaque whose inputs the slots lack or whose queue is full.
pub fn grey(
    screen: Res<Screen>,
    at: Res<StationAt>,
    recipes: Res<Recipes>,
    save: Res<WorldSave>,
    hotbar: Res<slots::Hotbar>,
    locks: Res<pbd_app::missions::RecipeLocks>,
    mut plaques: Query<(&RecipePlaque, &mut Plaque)>,
) {
    if *screen != Screen::Forge {
        return;
    }
    let Some((cell, layer)) = at.0 else { return };
    for (plaque, mut state) in &mut plaques {
        let Some(recipe) = recipes.0.get(&plaque.0) else {
            continue;
        };
        let queue = station_state(&save, cell, layer, recipe.station);
        let short = recipe
            .inputs
            .iter()
            .any(|&(item, n)| hotbar.count(item) < u32::from(n));
        let disabled = short
            || queue.jobs.len() >= recipe.station.max_jobs
            || locks.locked(&recipe.code).is_some();
        if state.disabled != disabled {
            state.disabled = disabled;
        }
    }
}

/// Paint the recipe rows' icons once the art has loaded.
pub fn paint(
    screen: Res<Screen>,
    icons: Option<Res<ItemIcons>>,
    images: Res<Assets<Image>>,
    mut nodes: Query<(&RecipeIcon, &mut ImageNode)>,
) {
    let Some(icons) = icons else { return };
    if *screen != Screen::Forge {
        return;
    }
    for (icon, mut image) in &mut nodes {
        slots::paint_icon(
            &mut image,
            Some(Stack::new(icon.0, icon.1)),
            &icons,
            &images,
        );
    }
}
