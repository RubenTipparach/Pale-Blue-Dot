//! Survival's panels (`survival-mode` tasks 4.6 and 4.8; design 9 and 11,
//! "The rest of the non-combat pieces"): the pack's stats panel, where every
//! stat is read at any time (survey G7), and the repair panels, at the
//! workbench for half the tool's head and at a town's trader or smith for
//! coins. Survival only: in Explorer none of them shows.
//!
//! The look is the approved HUD mockup's pack (`docs/mockups/survival.html
//! #hud`, "Pack open"): the four bars with their numbers, the thermometer,
//! and the two worn slots, in the kit's stone.

use super::menu::{self, Panel, Screen};
use super::slots::{self, Hotbar, ItemIcons};
use super::survival_play::SurvivalIcons;
use bevy::prelude::*;
use pbd_app::saves::WorldSave;
use pbd_app::survival::{self as play, Survival};
use pbd_app::ui_kit::{self, KitText, Localized, Plaque, Set, UiKit, tr};
use pbd_core::inventory::{Item, Stack, Tool};
use pbd_core::locale::tr_with;
use pbd_core::survival::wear::Wearer;

/// A stat bar's width in the pack, native px (the mockup's 86 at 2x is 43;
/// wider here, the pack is roomy).
const STAT_W: f32 = 52.0;
/// The stats panel's width, rim included.
pub const STATS_PANEL_W: f32 = 112.0;
/// How many price icons a repair panel shows at most.
const PRICE_MAX: usize = 4;

/// The pack's stats panel.
#[derive(Component)]
pub struct StatsPanel;
/// A stat row's icon, fill and number, by the HUD's bar.
#[derive(Component)]
pub struct StatIcon(usize);
#[derive(Component)]
pub struct StatFill(usize);
#[derive(Component)]
pub struct StatNumber(usize);
/// The thermometer's fill, reading and cause in the pack.
#[derive(Component)]
pub struct PackThermoFill;
#[derive(Component)]
pub struct PackThermoText;
#[derive(Component)]
pub struct PackThermoCause;
/// A worn slot's icon: the head (`true`) or the body.
#[derive(Component)]
pub struct WornIcon(bool);

/// The workbench's repair panel, its plaque, the tool's icon and bar, and
/// its price.
#[derive(Component)]
pub struct BenchRepair;
#[derive(Component)]
pub struct BenchRepairPlaque;
#[derive(Component)]
pub struct RepairToolIcon;
#[derive(Component)]
pub struct RepairWearFill;
#[derive(Component)]
pub struct PriceIcon(usize);
#[derive(Component)]
pub struct PriceCount(usize);
/// The town's repair panel, its plaque and its price in coins.
#[derive(Component)]
pub struct TownRepair;
#[derive(Component)]
pub struct TownRepairPlaque;
#[derive(Component)]
pub struct TownRepairCoins;

/// The four stats in the HUD's order: icon, colour.
fn colours() -> [Color; 4] {
    [
        Color::srgb_u8(0xd8, 0x44, 0x3a),
        Color::srgb_u8(0xe8, 0xc8, 0x4a),
        Color::srgb_u8(0xc9, 0x7a, 0x3a),
        Color::srgb_u8(0x4a, 0xa8, 0xe8),
    ]
}

/// A stone slot well holding one icon.
fn well(parent: &mut ChildSpawnerCommands, kit: &UiKit, icon: impl Bundle) {
    parent
        .spawn((
            kit.nine(Set::Stone, slots::slot_piece(false)),
            Node {
                width: px(slots::SLOT),
                height: px(slots::SLOT),
                flex_shrink: 0.0,
                ..default()
            },
        ))
        .with_children(|cell| {
            cell.spawn((
                ImageNode {
                    color: Color::NONE,
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    left: px(slots::ICON_AT),
                    top: px(slots::ICON_AT),
                    width: px(slots::ICON),
                    height: px(slots::ICON),
                    ..default()
                },
                icon,
            ));
        });
}

/// The pack's stats panel, left of the pack (G7), hidden until a Survival
/// world's pack opens. Called by the pack's own spawn.
pub fn spawn_stats(pair: &mut ChildSpawnerCommands, kit: &UiKit, height: f32) {
    pair.spawn((
        kit.nine(Set::Stone, "frame"),
        Node {
            width: px(STATS_PANEL_W),
            height: px(height),
            flex_shrink: 0.0,
            padding: UiRect::all(px(super::pack::RIM)),
            flex_direction: FlexDirection::Column,
            row_gap: px(3),
            display: Display::None,
            ..default()
        },
        StatsPanel,
    ))
    .with_children(|panel| {
        menu::lintel(panel, kit, tr("pack.stats"));
        for (k, colour) in colours().into_iter().enumerate() {
            panel
                .spawn(Node {
                    align_items: AlignItems::Center,
                    column_gap: px(2),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        ImageNode::default(),
                        Node {
                            width: px(11),
                            height: px(11),
                            ..default()
                        },
                        StatIcon(k),
                    ));
                    row.spawn((
                        kit.nine(Set::Stone, "bar"),
                        Node {
                            width: px(STAT_W),
                            height: px(11),
                            padding: UiRect::all(px(3)),
                            ..default()
                        },
                    ))
                    .with_children(|bar| {
                        bar.spawn((
                            Node {
                                width: percent(100),
                                height: percent(100),
                                ..default()
                            },
                            BackgroundColor(colour),
                            StatFill(k),
                        ));
                    });
                    row.spawn((
                        KitText::new(Localized::verbatim("100"), ui_kit::LABEL)
                            .outline(ui_kit::INK),
                        Node {
                            width: px(16),
                            justify_content: JustifyContent::FlexEnd,
                            ..default()
                        },
                        StatNumber(k),
                    ));
                });
        }
        // The thermometer, with its reading and cause.
        panel
            .spawn(Node {
                align_items: AlignItems::Center,
                column_gap: px(2),
                margin: UiRect::top(px(1)),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Node {
                        width: px(5),
                        height: px(13),
                        border: UiRect::all(px(1)),
                        justify_content: JustifyContent::FlexEnd,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    BorderColor::all(ui_kit::STONE_MID),
                    BackgroundColor(ui_kit::WELL),
                ))
                .with_children(|tube| {
                    tube.spawn((
                        Node {
                            width: percent(100),
                            height: percent(50),
                            ..default()
                        },
                        BackgroundColor(Color::srgb_u8(0xc9, 0xd2, 0x4a)),
                        PackThermoFill,
                    ));
                });
                row.spawn((
                    KitText::new(Localized::empty(), ui_kit::LABEL).outline(ui_kit::INK),
                    PackThermoText,
                ));
                row.spawn((
                    ImageNode::default(),
                    Node {
                        width: px(10),
                        height: px(10),
                        display: Display::None,
                        ..default()
                    },
                    PackThermoCause,
                ));
            });
        // The two worn slots: the hat and the cloak (design 13).
        panel
            .spawn(Node {
                column_gap: px(2),
                margin: UiRect::top(px(2)),
                ..default()
            })
            .with_children(|row| {
                well(row, kit, WornIcon(true));
                well(row, kit, WornIcon(false));
            });
    });
}

/// Keep the stats panel current while the pack is open in a Survival world.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn paint_stats(
    screen: Res<Screen>,
    survival: Res<Survival>,
    save: Res<WorldSave>,
    icons: Option<Res<SurvivalIcons>>,
    items: Option<Res<ItemIcons>>,
    images: Res<Assets<Image>>,
    mut parts: ParamSet<(
        Query<(&StatIcon, &mut ImageNode)>,
        Query<(&StatFill, &mut Node)>,
        Query<(&StatNumber, &mut KitText)>,
        Query<(&mut Node, &mut BackgroundColor), With<PackThermoFill>>,
        Query<&mut KitText, With<PackThermoText>>,
        Query<(&mut Node, &mut ImageNode), With<PackThermoCause>>,
        Query<(&WornIcon, &mut ImageNode)>,
        Query<&mut Node, With<StatsPanel>>,
    )>,
) {
    if *screen != Screen::Pack {
        return;
    }
    let shown = survival
        .vitals
        .as_ref()
        .filter(|_| save.mode().is_survival());
    for mut node in &mut parts.p7() {
        let want = if shown.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != want {
            node.display = want;
        }
    }
    let (Some(v), Some(icons)) = (shown, icons) else {
        return;
    };
    let cfg = play::rules();
    let shares = [
        v.health / cfg.health.max_pts,
        v.stamina.pts / cfg.stamina.max_pts,
        v.food_fp as f32 / cfg.food.max_fp,
        v.water as f32 / cfg.water.max,
    ];
    let handles = [
        icons.heart.clone(),
        icons.stamina.clone(),
        icons.food.clone(),
        icons.water.clone(),
    ];
    for (k, mut image) in &mut parts.p0() {
        if image.image != handles[k.0] {
            image.image = handles[k.0].clone();
        }
    }
    for (k, mut node) in &mut parts.p1() {
        node.width = percent(shares[k.0].clamp(0.0, 1.0) * 100.0);
    }
    for (k, mut text) in &mut parts.p2() {
        let want = Localized::verbatim(format!("{:.0}", (shares[k.0] * 100.0).clamp(0.0, 100.0)));
        if text.text != want {
            text.text = want;
        }
    }
    let t = v.body_c;
    let colour = if t >= 39.5 {
        Color::srgb_u8(0xe0, 0x50, 0x2a)
    } else if t >= 38.0 {
        Color::srgb_u8(0xe0, 0x89, 0x3a)
    } else if t <= 35.5 {
        Color::srgb_u8(0x7f, 0xc4, 0xff)
    } else {
        Color::srgb_u8(0xc9, 0xd2, 0x4a)
    };
    for (mut node, mut bg) in &mut parts.p3() {
        node.height = percent(((t - 33.0) / 8.0 * 100.0).clamp(8.0, 100.0));
        bg.0 = colour;
    }
    let arrow = if survival.trend_c_per_min > 0.05 {
        "↑"
    } else if survival.trend_c_per_min < -0.05 {
        "↓"
    } else {
        ""
    };
    let reading = pbd_core::locale::tr_with("survival.hud.body_c", &[("c", &format!("{t:.1}"))]);
    let want = Localized::verbatim(format!("{reading}{arrow}"));
    for mut text in &mut parts.p4() {
        if text.text != want {
            text.text = want.clone();
        }
        text.colour = colour;
    }
    for (mut node, mut image) in &mut parts.p5() {
        match survival.cause {
            Some(c) => {
                node.display = Display::Flex;
                image.image = icons.cause(c);
            }
            None => node.display = Display::None,
        }
    }
    let worn = play::WornRecord::of(&save);
    if let Some(items) = items {
        for (slot, mut image) in &mut parts.p6() {
            let code = if slot.0 { &worn.head } else { &worn.body };
            let stack = code
                .as_deref()
                .and_then(pbd_core::goods::Good::by_code)
                .map(|g| Stack::new(Item::Good(g), 1));
            slots::paint_icon(&mut image, stack, &items, &images);
        }
    }
}

// ------------------------------------------------------------ repairs (4.8)

/// The repair panel's frame and its plaque, the tool's icon with its wear
/// bar, and room for a price, top centre over a screen.
fn spawn_repair<P: Component, Pl: Component>(
    commands: &mut Commands,
    kit: &UiKit,
    atlas: &Handle<Image>,
    screen: Screen,
    marker: P,
    plaque: Pl,
    coins: Option<Handle<Image>>,
) {
    commands
        .spawn((
            Node {
                align_items: AlignItems::FlexStart,
                justify_content: JustifyContent::Center,
                padding: UiRect::top(px(14)),
                ..menu::layer()
            },
            GlobalZIndex(11),
            Panel(screen),
        ))
        .with_children(|layer| {
            layer
                .spawn((
                    kit.nine(Set::Stone, "frame"),
                    Node {
                        padding: UiRect::new(px(10), px(10), px(8), px(8)),
                        align_items: AlignItems::Center,
                        column_gap: px(4),
                        display: Display::None,
                        ..default()
                    },
                    marker,
                ))
                .with_children(|panel| {
                    // The tool in hand, its wear under it.
                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: px(1),
                            ..default()
                        })
                        .with_children(|tool| {
                            tool.spawn((
                                ImageNode::default(),
                                Node {
                                    width: px(16),
                                    height: px(16),
                                    ..default()
                                },
                                RepairToolIcon,
                            ));
                            tool.spawn((
                                Node {
                                    width: px(16),
                                    height: px(2),
                                    ..default()
                                },
                                BackgroundColor(ui_kit::WELL),
                            ))
                            .with_children(|bar| {
                                bar.spawn((
                                    Node {
                                        width: percent(50),
                                        height: percent(100),
                                        ..default()
                                    },
                                    BackgroundColor(ui_kit::RUNE),
                                    RepairWearFill,
                                ));
                            });
                        });
                    panel
                        .spawn((ui_kit::plaque(kit, Set::Stone, 64.0), plaque))
                        .with_children(|p| {
                            p.spawn(ui_kit::plaque_label(Set::Stone, tr("craft.repair")));
                        });
                    match coins {
                        // A town's price: the coin and the number.
                        Some(coin) => {
                            panel.spawn((
                                ImageNode::new(coin),
                                Node {
                                    width: px(16),
                                    height: px(16),
                                    ..default()
                                },
                            ));
                            panel.spawn((
                                KitText::new(Localized::empty(), ui_kit::LABEL)
                                    .outline(ui_kit::INK),
                                TownRepairCoins,
                            ));
                        }
                        // The bench's price: up to four items with counts.
                        None => {
                            for k in 0..PRICE_MAX {
                                panel
                                    .spawn((
                                        ImageNode {
                                            image: atlas.clone(),
                                            color: Color::NONE,
                                            ..default()
                                        },
                                        Node {
                                            width: px(16),
                                            height: px(16),
                                            flex_shrink: 0.0,
                                            display: Display::None,
                                            ..default()
                                        },
                                        PriceIcon(k),
                                    ))
                                    .with_children(|icon| {
                                        icon.spawn((
                                            KitText::new(Localized::empty(), ui_kit::LABEL)
                                                .outline(ui_kit::INK),
                                            Node {
                                                position_type: PositionType::Absolute,
                                                bottom: px(-3),
                                                right: px(-2),
                                                ..default()
                                            },
                                            PriceCount(k),
                                        ));
                                    });
                            }
                        }
                    }
                });
        });
}

fn spawn_panels(
    mut commands: Commands,
    kit: Res<UiKit>,
    icons: Res<ItemIcons>,
    coin: Option<Res<super::talk::CoinIcon>>,
) {
    spawn_repair(
        &mut commands,
        &kit,
        &icons.atlas,
        Screen::Forge,
        BenchRepair,
        BenchRepairPlaque,
        None,
    );
    let coin = coin.map(|c| c.0.clone()).unwrap_or_default();
    spawn_repair(
        &mut commands,
        &kit,
        &icons.atlas,
        Screen::Talk,
        TownRepair,
        TownRepairPlaque,
        Some(coin),
    );
}

/// The tool in hand and its tier, if it is worn (below full) in a Survival
/// world.
fn worn_in_hand(
    save: &WorldSave,
    tools: &pbd_app::fish::ToolSlot,
) -> Option<(Tool, pbd_core::inventory::Tier, f32)> {
    if !save.mode().is_survival() {
        return None;
    }
    let tool = tools.0.held()?;
    let tier = tools.0.tier(tool)?;
    let wear = play::WearBook::of(save).wear(tool, tier);
    let share = wear.share(play::rules(), &Wearer::Tool(tool, tier));
    (share < 1.0).then_some((tool, tier, share))
}

/// Show the tool in hand and its wear on a repair panel.
fn paint_tool(
    items: &ItemIcons,
    tool: Tool,
    tier: pbd_core::inventory::Tier,
    share: f32,
    icon: &mut Query<&mut ImageNode, (With<RepairToolIcon>, Without<PriceIcon>)>,
    fill: &mut Query<(&mut Node, &mut BackgroundColor), With<RepairWearFill>>,
) {
    for mut image in icon.iter_mut() {
        image.image = items.tool_at(tool, tier);
        image.color = if share <= 0.0 {
            Color::srgb(0.45, 0.45, 0.45)
        } else {
            Color::WHITE
        };
    }
    for (mut node, mut colour) in fill.iter_mut() {
        node.width = percent(share.clamp(0.0, 1.0) * 100.0);
        colour.0 = if share < play::rules().wear.red_below_share {
            Color::srgb_u8(0xd8, 0x44, 0x3a)
        } else {
            ui_kit::RUNE
        };
    }
}

/// The workbench's repair (design 11): while the workbench is open in a
/// Survival world and the tool in hand is worn, its price is half the head
/// of its recipe; the plaque is greyed while the pack lacks it, and a press
/// takes it and restores every use.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn bench_repair(
    screen: Res<Screen>,
    at: Res<super::farm::StationAt>,
    recipes: Res<pbd_app::config::Recipes>,
    mut save: ResMut<WorldSave>,
    mut hotbar: ResMut<Hotbar>,
    tools: Res<pbd_app::fish::ToolSlot>,
    mut survival: ResMut<Survival>,
    (items, images): (Option<Res<ItemIcons>>, Res<Assets<Image>>),
    mut panel: Query<
        &mut Node,
        (
            With<BenchRepair>,
            Without<PriceIcon>,
            Without<RepairWearFill>,
        ),
    >,
    mut plaque: Query<(&Interaction, &mut Plaque), With<BenchRepairPlaque>>,
    mut tool_icon: Query<&mut ImageNode, (With<RepairToolIcon>, Without<PriceIcon>)>,
    mut fill: Query<(&mut Node, &mut BackgroundColor), With<RepairWearFill>>,
    mut prices: Query<
        (&PriceIcon, &mut Node, &mut ImageNode),
        (
            Without<BenchRepair>,
            Without<RepairToolIcon>,
            Without<RepairWearFill>,
        ),
    >,
    mut counts: Query<(&PriceCount, &mut KitText)>,
) {
    if *screen != Screen::Forge {
        return;
    }
    let at_bench =
        at.0.and_then(|(cell, layer)| super::craft::station_row_at(&save, cell, layer))
            .is_some_and(|row| row.code == "workbench");
    let worn = worn_in_hand(&save, &tools).filter(|_| at_bench);
    let price = worn.and_then(|(tool, tier, _)| {
        play::places::repair_price(&recipes.0, play::rules(), tool, tier)
    });
    let show = worn.is_some() && price.is_some();
    for mut node in &mut panel {
        let want = if show { Display::Flex } else { Display::None };
        if node.display != want {
            node.display = want;
        }
    }
    let (Some((tool, tier, share)), Some(price), Some(items)) = (worn, price, items) else {
        return;
    };
    paint_tool(&items, tool, tier, share, &mut tool_icon, &mut fill);
    for (k, mut node, mut image) in &mut prices {
        match price.get(k.0) {
            Some(&(item, n)) => {
                node.display = Display::Flex;
                slots::paint_icon(&mut image, Some(Stack::new(item, n)), &items, &images);
            }
            None => node.display = Display::None,
        }
    }
    for (k, mut text) in &mut counts {
        let want = price
            .get(k.0)
            .map(|&(_, n)| Localized::verbatim(super::craft::need_count(true, n)))
            .unwrap_or_default();
        if text.text != want {
            text.text = want;
        }
    }
    let short = price
        .iter()
        .any(|&(item, n)| hotbar.0.count(item) < u32::from(n));
    for (interaction, mut state) in &mut plaque {
        if state.disabled != short {
            state.disabled = short;
        }
        if *interaction == Interaction::Pressed
            && !short
            && play::places::repair_at_bench(&mut save, &mut hotbar.0, tool, tier, &price)
        {
            info!("SURVIVAL repaired the {tool:?} at the workbench");
            survival.say(tr_with(
                "survival.repaired",
                &[("tool", &pbd_core::inventory::tool_title(tool, tier))],
            ));
        }
    }
}

/// A town's repair (design 11): on the talk with a trader or a smith in a
/// Survival world, while the tool in hand is worn, a REPAIR plaque with its
/// price in coins; a press pays and restores every use.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn town_repair(
    screen: Res<Screen>,
    mut talking: ResMut<super::talk::Talking>,
    mut save: ResMut<WorldSave>,
    tools: Res<pbd_app::fish::ToolSlot>,
    cfg: Option<Res<pbd_app::economy::EconomyConfig>>,
    mut eco: Option<ResMut<pbd_app::economy::Economy>>,
    items: Option<Res<ItemIcons>>,
    mut panel: Query<&mut Node, (With<TownRepair>, Without<RepairWearFill>)>,
    mut plaque: Query<(&Interaction, &mut Plaque), With<TownRepairPlaque>>,
    mut tool_icon: Query<&mut ImageNode, (With<RepairToolIcon>, Without<PriceIcon>)>,
    mut fill: Query<(&mut Node, &mut BackgroundColor), With<RepairWearFill>>,
    mut coins_text: Query<&mut KitText, With<TownRepairCoins>>,
) {
    if *screen != Screen::Talk {
        return;
    }
    let repairer = talking
        .with
        .as_ref()
        .is_some_and(|w| w.trader || w.talker.role == "smith");
    let worn = worn_in_hand(&save, &tools).filter(|_| repairer);
    let price = worn.and_then(|(tool, tier, _)| {
        let value = cfg
            .as_deref()?
            .values
            .get(&format!("tool:{}", tool.code()))?
            .coins;
        Some(play::places::repair_coins_for(&save, tool, tier, value))
    });
    let show = worn.is_some() && price.is_some();
    for mut node in &mut panel {
        let want = if show { Display::Flex } else { Display::None };
        if node.display != want {
            node.display = want;
        }
    }
    let (Some((tool, tier, share)), Some(coins), Some(items)) = (worn, price, items) else {
        return;
    };
    paint_tool(&items, tool, tier, share, &mut tool_icon, &mut fill);
    let want = Localized::verbatim(coins.to_string());
    for mut text in &mut coins_text {
        if text.text != want {
            text.text = want.clone();
        }
    }
    let Some(eco) = eco.as_deref_mut() else {
        return;
    };
    let short = !eco.purse.covers(u64::from(coins));
    for (interaction, mut state) in &mut plaque {
        if state.disabled != short {
            state.disabled = short;
        }
        if *interaction != Interaction::Pressed {
            continue;
        }
        talking.said = Some(
            match play::places::repair_for_coins(&mut save, &mut eco.purse, tool, tier, coins) {
                Ok(()) => {
                    info!("SURVIVAL repaired the {tool:?} in town for {coins} coins");
                    tr_with(
                        "survival.repaired",
                        &[("tool", &pbd_core::inventory::tool_title(tool, tier))],
                    )
                }
                Err(_) => tr("trade.short"),
            },
        );
    }
}

pub struct SurvivalPanelsPlugin;

impl Plugin for SurvivalPanelsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostStartup,
            spawn_panels
                .after(super::talk::spawn)
                .after(super::slots::load_icons),
        )
        .add_systems(
            Update,
            (paint_stats, bench_repair, town_repair).run_if(pbd_app::world_open),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_panel_word_is_in_the_locale_file() {
        for key in [
            "pack.stats",
            "craft.repair",
            "survival.repaired",
            "survival.cactus",
            "survival.note.empty",
            "survival.note.needs_fire",
            "trade.short",
        ] {
            assert_ne!(tr(key).to_string(), key, "{key}");
        }
    }
}
