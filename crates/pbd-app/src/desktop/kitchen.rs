//! Cooking in play (`cooking` design S2 to S6): the campfire's panel, eating
//! a dish, the effects over the hotbar, and the capture instrument.
//!
//! The station rules are the farming change's (`pbd_core::station`): a
//! campfire's queue is a `station` record and its jobs run on world time.
//! What is the kitchen's own is "any fish" resolved at queue time
//! (`pbd_core::cooking::concrete`), the `Cooked` and `Ate` events, and the
//! `meal` record that eating writes beside the slots in one line.

use super::farm::{FarmNote, StationAt, station_state};
use super::menu::{self, Panel, Screen};
use super::slots::{self, ItemIcons};
use bevy::prelude::*;
use pbd_app::config::{KitchenConfig, Recipes};
use pbd_app::hotbar::{Gained, How};
use pbd_app::kitchen::{Ate, Cooked, meals};
use pbd_app::saves::WorldSave;
use pbd_app::ui_kit::{self, KitText, Plaque, Set, UiKit};
use pbd_app::ui_kit::{Localized, tr};
use pbd_core::cooking::{self, ANY_FISH, MEAL_ID, MEAL_KIND, MEAL_SCHEMA};
use pbd_core::goods::Good;
use pbd_core::inventory::{Item, Slots, Stack};
use pbd_core::records::Record as StoredRecord;
use pbd_core::station::{self, STATION_KIND, STATION_SCHEMA, StationRow, StationState};

/// The kitchen's station, the one this panel lists.
const CAMPFIRE: &str = "campfire";

/// Which panel a station opens: the campfire the kitchen's, every other the
/// forge's.
pub fn screen_for(row: &StationRow) -> Screen {
    if row.code == CAMPFIRE {
        Screen::Kitchen
    } else {
        Screen::Forge
    }
}

/// A recipe row's plaque.
#[derive(Component)]
pub struct CookPlaque(pub String);

/// An item picture in a recipe row. "Any fish" is drawn as a fish.
#[derive(Component)]
pub struct CookIcon(pub Item, pub u16);

#[derive(Component)]
pub struct CookLine;
#[derive(Component)]
pub struct CookFill;

/// The panel: the stone-and-wood kit's stone frame, the lintel, the campfire's
/// recipes in two columns, the job being worked and its bar, and a line
/// saying how a dish is eaten.
pub fn spawn(
    mut commands: Commands,
    icons: Res<ItemIcons>,
    kit: Res<UiKit>,
    recipes: Res<Recipes>,
) {
    let kit = &*kit;
    let atlas = icons.atlas.clone();
    let bar = kit.nine(Set::Stone, "bar");
    let fill = kit.whole(Set::Stone, "fill-progress");
    commands
        .spawn((menu::layer(), GlobalZIndex(10), Panel(Screen::Kitchen)))
        .with_children(|layer| {
            // The stone moulding: no baked rock is this panel's size.
            layer
                .spawn(menu::framed(kit, Set::Stone, 372.0))
                .with_children(|panel| {
                    menu::lintel(panel, kit, "CAMPFIRE");
                    panel
                        .spawn(Node {
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: px(10),
                            ..default()
                        })
                        .with_children(|grid| {
                            for recipe in recipes.0.at(CAMPFIRE) {
                                grid.spawn(Node {
                                    align_items: AlignItems::Center,
                                    margin: UiRect::bottom(px(4)),
                                    column_gap: px(3),
                                    width: px(167),
                                    ..default()
                                })
                                .with_children(|row| {
                                    row.spawn((
                                        ui_kit::plaque(kit, Set::Stone, 80.0),
                                        CookPlaque(recipe.code.clone()),
                                    ))
                                    .with_children(|p| {
                                        p.spawn(ui_kit::plaque_label(
                                            Set::Stone,
                                            Localized::legacy(recipe.name.clone()),
                                        ));
                                    });
                                    for &input in &recipe.inputs {
                                        icon(row, &atlas, input, &recipe.code, true);
                                    }
                                    row.spawn(
                                        KitText::new(Localized::verbatim("→"), ui_kit::TEXT)
                                            .shadow(ui_kit::INK),
                                    );
                                    for &output in &recipe.outputs {
                                        icon(row, &atlas, output, &recipe.code, false);
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
                        });
                    panel.spawn((
                        KitText::new(Localized::empty(), ui_kit::TEXT).shadow(ui_kit::INK),
                        Node {
                            margin: UiRect::new(px(0), px(0), px(2), px(3)),
                            ..default()
                        },
                        CookLine,
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
                                CookFill,
                            ));
                        });
                    panel.spawn((
                        KitText::new(tr("kitchen.eat_hint"), ui_kit::TEXT_DIM).shadow(ui_kit::INK),
                        Node {
                            margin: UiRect::top(px(4)),
                            ..default()
                        },
                    ));
                });
        });
}

/// One item picture with its count.
fn icon(
    row: &mut ChildSpawnerCommands,
    atlas: &Handle<Image>,
    (it, n): (Item, u16),
    code: &str,
    need: bool,
) {
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
        CookIcon(it, n),
        super::craft::LockTint(code.to_string()),
    ))
    .with_children(|icon| {
        icon.spawn((
            KitText::new(
                Localized::legacy(super::craft::need_count(need, n)),
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
}

/// Store a station's queue with the slots after it, in one line.
fn store(save: &mut WorldSave, cell: u32, layer: u16, state: &StationState, slots: &Slots) -> bool {
    let record = StoredRecord::of(
        STATION_KIND,
        station::record_id(cell, layer),
        STATION_SCHEMA,
        state,
    );
    save.store_with_hands(record, STATION_SCHEMA, slots)
}

/// Queue a recipe at a campfire: "any fish" becomes the species the slots
/// hold, its inputs leave the slots and the job joins the queue, in one
/// saved line. What the capture rig calls too.
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
    let Some(real) = cooking::concrete(recipe, &hotbar.0) else {
        note.say(wall_s, format!("No fish for {}", recipe.name));
        return false;
    };
    let mut state = station_state(save, cell, layer, recipe.station);
    let mut moved = hotbar.0.clone();
    match state.queue(&mut moved, &real, now_s) {
        Ok(()) => {}
        Err(station::Refused::Short) => {
            note.say(wall_s, format!("Not enough for {}", recipe.name));
            return false;
        }
        Err(station::Refused::Full) => {
            note.say(wall_s, "The fire is full");
            return false;
        }
        Err(station::Refused::Elsewhere | station::Refused::NotBetter) => return false,
    }
    if !store(save, cell, layer, &state, &moved) {
        error!("cook BLOCKED: the save did not accept it");
        return false;
    }
    hotbar.0 = moved;
    info!(
        "cooking {code} at cell {cell} layer {layer}, {} on the fire",
        state.jobs.len()
    );
    true
}

/// Take every finished dish off a campfire into the slots, in one line,
/// raising `Cooked` and `Gained` for each. Returns how many were collected.
#[allow(clippy::too_many_arguments)]
pub fn collect(
    save: &mut WorldSave,
    hotbar: &mut slots::Hotbar,
    recipes: &Recipes,
    (cell, layer): (u32, u16),
    now_s: f64,
    note: &mut FarmNote,
    wall_s: f32,
    cooked: &mut MessageWriter<Cooked>,
    gained: &mut MessageWriter<Gained>,
) -> usize {
    let Some(row) = station::station_by_code(CAMPFIRE) else {
        return 0;
    };
    let mut state = station_state(save, cell, layer, row);
    let before = state.jobs.clone();
    let mut moved = hotbar.0.clone();
    let done = state.collect(&mut moved, &recipes.0, now_s);
    if done == 0 {
        return 0;
    }
    if !store(save, cell, layer, &state, &moved) {
        error!("collect BLOCKED: the save did not accept it");
        return 0;
    }
    hotbar.0 = moved;
    for job in &before[..done] {
        let Some(recipe) = recipes.0.get(&job.recipe) else {
            continue;
        };
        for &(item, count) in &recipe.outputs {
            gained.write(Gained {
                item,
                count,
                how: How::Made,
            });
            cooked.write(Cooked {
                recipe: recipe.code.clone(),
                dish: item,
                count,
                fish: cooking::fish_in(recipe),
            });
        }
        note.say(wall_s, format!("{} is ready", recipe.name));
        info!("cooked {} at cell {cell} layer {layer}", recipe.code);
    }
    done
}

/// The panel's work: a pressed plaque cooks; finished dishes are taken; the
/// line and the bar follow the fire.
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
    (mut gained, mut cooked): (MessageWriter<Gained>, MessageWriter<Cooked>),
    plaques: Query<(&Interaction, &CookPlaque), Changed<Interaction>>,
    mut line: Query<&mut KitText, With<CookLine>>,
    mut fill: Query<&mut Node, With<CookFill>>,
    locks: Res<pbd_app::missions::RecipeLocks>,
) {
    if *screen != Screen::Kitchen {
        return;
    }
    let Some(at) = at.0 else { return };
    let now_s = sun.clock.seconds;
    let wall_s = time.elapsed_secs();
    for (interaction, plaque) in &plaques {
        if *interaction == Interaction::Pressed {
            if let Some(opens) = locks.locked(&plaque.0) {
                note.say(wall_s, format!("Opens with: {opens}"));
                continue;
            }
            queue(
                &mut save,
                &mut hotbar,
                &recipes,
                at,
                &plaque.0,
                now_s,
                &mut note,
                wall_s,
            );
        }
    }
    collect(
        &mut save,
        &mut hotbar,
        &recipes,
        at,
        now_s,
        &mut note,
        wall_s,
        &mut cooked,
        &mut gained,
    );
    let Some(row) = station::station_by_code(CAMPFIRE) else {
        return;
    };
    let state = station_state(&save, at.0, at.1, row);
    let progress = state.head_progress(&recipes.0, now_s);
    let text = match state.jobs.first().and_then(|j| recipes.0.get(&j.recipe)) {
        Some(recipe) => {
            let waiting = state.jobs.len() - 1;
            if waiting > 0 {
                format!("COOKING {}  +{waiting} WAITING", recipe.name)
            } else {
                format!("COOKING {}", recipe.name)
            }
        }
        None => "PICK A DISH".into(),
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

/// Grey a plaque whose inputs the slots lack or whose fire is full.
pub fn grey(
    screen: Res<Screen>,
    at: Res<StationAt>,
    recipes: Res<Recipes>,
    save: Res<WorldSave>,
    hotbar: Res<slots::Hotbar>,
    locks: Res<pbd_app::missions::RecipeLocks>,
    mut plaques: Query<(&CookPlaque, &mut Plaque)>,
) {
    if *screen != Screen::Kitchen {
        return;
    }
    let Some((cell, layer)) = at.0 else { return };
    for (plaque, mut state) in &mut plaques {
        let Some(recipe) = recipes.0.get(&plaque.0) else {
            continue;
        };
        let queue = station_state(&save, cell, layer, recipe.station);
        let disabled = !cooking::have(recipe, &hotbar.0)
            || queue.jobs.len() >= recipe.station.max_jobs
            || locks.locked(&recipe.code).is_some();
        if state.disabled != disabled {
            state.disabled = disabled;
        }
    }
}

/// The picture "any fish" is drawn with: the first fish the slots hold, or
/// the roster's first.
fn shown(item: Item, slots: &Slots) -> Item {
    if item != ANY_FISH {
        return item;
    }
    slots
        .iter()
        .flatten()
        .map(|s| s.item)
        .find(|i| matches!(i, Item::Fish(_)))
        .unwrap_or(Item::Fish(0))
}

/// Paint the recipe rows' icons once the art has loaded.
pub fn paint(
    screen: Res<Screen>,
    icons: Option<Res<ItemIcons>>,
    images: Res<Assets<Image>>,
    hotbar: Res<slots::Hotbar>,
    mut nodes: Query<(&CookIcon, &mut ImageNode)>,
) {
    let Some(icons) = icons else { return };
    if *screen != Screen::Kitchen {
        return;
    }
    for (icon, mut image) in &mut nodes {
        slots::paint_icon(
            &mut image,
            Some(Stack::new(shown(icon.0, &hotbar.0), icon.1)),
            &icons,
            &images,
        );
    }
}

// ---------------------------------------------------------------- eating

/// Eat one of the dish in the selected slot at `now_s`: the slot loses it
/// and the effect runs, in one saved line. `None`, with nothing changed,
/// when the slot holds no dish or the save refused.
pub fn eat_held(
    save: &mut WorldSave,
    hotbar: &mut slots::Hotbar,
    kitchen: &KitchenConfig,
    now_s: f64,
) -> Option<cooking::Dish> {
    let Some(Item::Good(good)) = hotbar.0.held().map(|s| s.item) else {
        return None;
    };
    let dish = *kitchen.0.dish(good)?;
    let mut moved = hotbar.0.clone();
    moved.take_held()?;
    let mut running = meals(save);
    running.eat(&dish, now_s, kitchen.0.max_effects);
    let record = StoredRecord::of(MEAL_KIND, MEAL_ID, MEAL_SCHEMA, &running);
    if !save.store_with_hands(record, MEAL_SCHEMA, &moved) {
        error!("eat BLOCKED: the save did not accept it");
        return None;
    }
    hotbar.0 = moved;
    info!(
        "ate a {}: {} for {} min",
        good.name(),
        dish.effect.name(),
        dish.minutes
    );
    Some(dish)
}

/// The right button with a dish in the selected slot eats it (design S5),
/// as it plants a seed. Not with a line out, which the button winds in.
#[allow(clippy::too_many_arguments)]
pub fn eat(
    buttons: Res<ButtonInput<MouseButton>>,
    screen: Res<Screen>,
    walking: Option<Res<pbd_app::walking::WalkingReadout>>,
    fishery: Option<Res<pbd_app::fish::Fishery>>,
    kitchen: Res<KitchenConfig>,
    sun: Res<pbd_app::sky::Sun>,
    time: Res<Time>,
    mut save: ResMut<WorldSave>,
    mut hotbar: ResMut<slots::Hotbar>,
    mut note: ResMut<FarmNote>,
    mut ate: MessageWriter<Ate>,
    mut did: MessageWriter<pbd_app::missions::Did>,
) {
    if !buttons.just_pressed(MouseButton::Right) || *screen != Screen::Playing {
        return;
    }
    if !walking.is_some_and(|w| w.active && w.captured) {
        return;
    }
    if fishery.is_some_and(|f| f.line.phase != pbd_core::fishing::Phase::Ready) {
        return;
    }
    if let Some(dish) = eat_held(&mut save, &mut hotbar, &kitchen, sun.clock.seconds) {
        ate.write(Ate {
            dish: dish.good,
            effect: dish.effect,
        });
        // What the missions count: a dish eaten (`starter-missions` H.1).
        did.write(pbd_app::missions::Did::new(
            pbd_core::missions::Kind::Eat,
            dish.good.code(),
            1,
        ));
        note.say(
            time.elapsed_secs(),
            format!(
                "Ate the {}: {} for {} min",
                dish.good.name(),
                dish.effect.name(),
                dish.minutes
            ),
        );
    }
}

// ---------------------------------------------------------------- the HUD

/// One running effect's place over the hotbar.
#[derive(Component)]
pub struct EffectSlot(pub usize);
#[derive(Component)]
pub struct EffectIcon(pub usize);
#[derive(Component)]
pub struct EffectText(pub usize);

/// The effects row: a slot well with the dish's icon and the effect's name
/// and minutes left, one for each running effect, over the hotbar's left
/// end. Hidden while nothing runs.
pub fn spawn_effects(
    mut commands: Commands,
    kit: Res<UiKit>,
    icons: Res<ItemIcons>,
    kitchen: Res<KitchenConfig>,
) {
    let atlas = icons.atlas.clone();
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: px(slots::HOTBAR_BOTTOM + slots::SLOT + 6.0),
            left: percent(50),
            margin: UiRect::left(px(-(pbd_core::inventory::SLOTS as f32 * slots::SLOT) / 2.0)),
            column_gap: px(8),
            ..default()
        })
        .with_children(|row| {
            for index in 0..kitchen.0.max_effects {
                row.spawn((
                    Node {
                        align_items: AlignItems::Center,
                        column_gap: px(3),
                        display: Display::None,
                        ..default()
                    },
                    EffectSlot(index),
                ))
                .with_children(|entry| {
                    entry
                        .spawn((
                            kit.whole(Set::Stone, slots::slot_piece(false)),
                            Node {
                                width: px(slots::SLOT),
                                height: px(slots::SLOT),
                                flex_shrink: 0.0,
                                ..default()
                            },
                        ))
                        .with_children(|well| {
                            well.spawn((
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
                                EffectIcon(index),
                            ));
                        });
                    entry.spawn((
                        KitText::new(Localized::empty(), ui_kit::LABEL).outline(ui_kit::INK),
                        EffectText(index),
                    ));
                });
            }
        });
}

/// Keep the effects row in step with the save's `meal` record and the clock.
#[allow(clippy::type_complexity)]
pub fn show_effects(
    save: Res<WorldSave>,
    sun: Res<pbd_app::sky::Sun>,
    icons: Option<Res<ItemIcons>>,
    images: Res<Assets<Image>>,
    mut slots_q: Query<(&EffectSlot, &mut Node)>,
    mut icon_q: Query<(&EffectIcon, &mut ImageNode)>,
    mut text_q: Query<(&EffectText, &mut KitText)>,
) {
    let Some(icons) = icons else { return };
    let now_s = sun.clock.seconds;
    let mut running: Vec<_> = meals(&save)
        .running
        .into_iter()
        .filter(|m| m.until_s > now_s)
        .collect();
    running.sort_by(|a, b| b.until_s.total_cmp(&a.until_s));
    for (slot, mut node) in &mut slots_q {
        let want = if slot.0 < running.len() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != want {
            node.display = want;
        }
    }
    for (icon, mut image) in &mut icon_q {
        let Some(meal) = running.get(icon.0) else {
            continue;
        };
        let good = Good::by_code(&meal.dish);
        slots::paint_icon(
            &mut image,
            good.map(|g| Stack::new(Item::Good(g), 1)),
            &icons,
            &images,
        );
    }
    for (text, mut words) in &mut text_q {
        let Some(meal) = running.get(text.0) else {
            continue;
        };
        let left = (meal.until_s - now_s).max(0.0);
        let line = format!(
            "{} {}:{:02}",
            meal.effect.name().to_uppercase(),
            (left / 60.0).floor() as u32,
            (left % 60.0).floor() as u32
        );
        if words.text != line {
            words.text = Localized::legacy(line);
        }
    }
}

// ---------------------------------------------------------------- capture rig

/// `--campfire [metres]` (`cooking` task 9.5, a capture instrument): a
/// campfire that far ahead through the same durable paths a player takes,
/// the slots stocked with fish, wheat and chili; flatbread put on the fire
/// long enough ago to be done and a fish three-quarters grilled behind it.
/// With `--campfire-panel` the panel opens, which collects the flatbread;
/// without it the flatbread is collected at once.
/// `--eat <dish,dish>` gives each dish and eats it, so the effects row
/// shows. The capture's save is memory-only, so nothing is kept.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn scripted(
    launch: Res<super::Launch>,
    (cameras, frame): (
        Query<(&GlobalTransform, &Camera), With<Camera3d>>,
        Option<Res<pbd_app::planet::PlanetRenderFrame>>,
    ),
    refresh: Res<pbd_app::planet::LodRefresh>,
    (mut fine, mut contact, mut save, mut hotbar, mut drops, sun): (
        ResMut<pbd_app::planet::PlanetFine>,
        ResMut<pbd_app::planet::PlanetContact>,
        ResMut<WorldSave>,
        ResMut<slots::Hotbar>,
        ResMut<pbd_app::drops::Drops>,
        Res<pbd_app::sky::Sun>,
    ),
    (recipes, kitchen, mut station_at, mut screen, mut note, time, mut ate): (
        Res<Recipes>,
        Res<KitchenConfig>,
        ResMut<StationAt>,
        ResMut<Screen>,
        ResMut<FarmNote>,
        Res<Time>,
        MessageWriter<Ate>,
    ),
    (mut cooked, mut gained): (MessageWriter<Cooked>, MessageWriter<Gained>),
    mut done: Local<bool>,
) {
    if *done
        || launch.capture.is_none()
        || (launch.campfire.is_none() && launch.eat.is_empty())
        || refresh.in_flight_s().is_some()
    {
        return;
    }
    let Some((transform, _)) = cameras.iter().find(|(_, c)| c.is_active) else {
        return;
    };
    let eye = pbd_app::render_origin::local_of(frame.as_deref(), transform.translation());
    let up = eye.normalize_or(Vec3::Y);
    let forward = transform.forward().as_vec3();
    let ahead = (forward - up * forward.dot(up)).normalize_or(Vec3::X);
    let metres = launch.campfire.unwrap_or(4.0);
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
    let now_s = sun.clock.seconds;
    let wall_s = time.elapsed_secs();
    if launch.campfire.is_some() {
        let layer = top + 1;
        super::digging::apply_edit(
            &mut super::digging::Edited {
                fine: &mut fine,
                contact: &mut contact,
                save: &mut save,
                slots: &mut hotbar,
                drops: &mut drops,
                now_s,
            },
            super::digging::Hands::Empty,
            cell,
            layer,
            pbd_core::terrain::Material::Campfire,
        );
        hotbar.give(Item::Fish(0), 2);
        hotbar.give(Item::Fish(1), 1);
        hotbar.give(Item::Good(Good::WHEAT), 6);
        hotbar.give(Item::Good(Good::CHILI), 2);
        let at = (cell, layer as u16);
        // Flatbread done a while ago; a fish three-quarters grilled behind it.
        queue(
            &mut save,
            &mut hotbar,
            &recipes,
            at,
            "flatbread",
            now_s - 60.0,
            &mut note,
            wall_s,
        );
        queue(
            &mut save,
            &mut hotbar,
            &recipes,
            at,
            "grilled-fish",
            now_s - 15.0,
            &mut note,
            wall_s,
        );
        if launch.campfire_panel {
            station_at.0 = Some(at);
            *screen = Screen::Kitchen;
        } else {
            // Without the panel, the cook takes what is done, so the pack
            // (`--menu pack`) shows the flatbread.
            collect(
                &mut save,
                &mut hotbar,
                &recipes,
                at,
                now_s,
                &mut note,
                wall_s,
                &mut cooked,
                &mut gained,
            );
        }
        info!("scripted campfire at cell {cell} layer {layer}");
    }
    for code in &launch.eat {
        let Some(good) = Good::by_code(code) else {
            warn!("--eat: no good {code}");
            continue;
        };
        // In the selected slot, as a player holds a dish to eat it.
        let selected = hotbar.selected();
        let mut slots = hotbar.0.clone();
        slots.set(selected, Some(Stack::new(Item::Good(good), 2)));
        hotbar.0 = slots;
        if let Some(dish) = eat_held(&mut save, &mut hotbar, &kitchen, now_s) {
            ate.write(Ate {
                dish: dish.good,
                effect: dish.effect,
            });
            note.say(
                wall_s,
                format!(
                    "Ate the {}: {} for {} min",
                    dish.good.name(),
                    dish.effect.name(),
                    dish.minutes
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every dish's icon and the campfire's are on disk.
    #[test]
    fn every_dish_has_its_icon() {
        let assets = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../assets"));
        for dish in &pbd_core::cooking::Kitchen::default().dishes {
            assert!(
                assets.join(dish.good.icon()).exists(),
                "{}",
                dish.good.icon()
            );
        }
        assert!(assets.join("items/lights/campfire.png").exists());
    }

    /// The campfire opens the kitchen's panel and the forge its own.
    #[test]
    fn each_station_opens_its_panel() {
        let campfire = station::station_by_code("campfire").unwrap();
        let forge = station::station_by_code("forge").unwrap();
        assert_eq!(screen_for(campfire), Screen::Kitchen);
        assert_eq!(screen_for(forge), Screen::Forge);
    }
}
