//! Survival on screen and at hand (`survival-mode` tasks 4.2, 4.6, 4.8,
//! 6.6): the HUD's left stack, the heat stroke warning and the line above
//! the hotbar, the red edge of a hit and the pass-out's dark, "You died" and
//! Wake up, the dropped pack lying where the player fell, drinking on F, rain
//! caught in an empty hand, and each tool's wear. The rules are
//! `pbd_app::survival`'s and the core's; this only reads what the player
//! does and shows what came back. Nothing here shows or acts in Explorer.
//!
//! The look is the approved mockup's (`docs/mockups/survival.html#hud`,
//! survey G7 and G8): short stone bars with their icons stacked at the left
//! edge, a full bar hidden, hunger and water only at half or lower, a small
//! thermometer under them only outside fine, breath as bubbles over the
//! hotbar, and "Heat stroke" as the one word above the hotbar.

use super::menu::{InPlayHud, Panel, Screen};
use super::slots::Hotbar;
use bevy::prelude::*;
use pbd_app::interact::{Act as Verb, Interact, Offer, Reader};
use pbd_app::saves::WorldSave;
use pbd_app::survival::{self as play, Survival, TempCause};
use pbd_app::ui_kit::{self, KitText, Localized, Set, UiKit, tr};
use pbd_app::walking::{EYE_HEIGHT, HALF_HEIGHT, Walker, WalkingState};
use pbd_core::inventory::Tool;
use pbd_core::survival::WaterSource;
use pbd_core::survival::temperature::FireNear;
use pbd_core::survival::wear::{Action, Spend};
use pbd_core::terrain::Material;
use pbd_server::survival::Act;

/// A bar of the left stack, native px (the mockup's 104 x 22 at its 2x).
const BAR_W: f32 = 52.0;
const BAR_H: f32 = 11.0;
const ICON: f32 = 11.0;
/// How far a sip reaches from the eye, m.
const DRINK_REACH_M: f32 = 3.0;
/// The share of a second rain is caught for one count of `drink rain`.
const RAIN_COUNT_S: f32 = 1.0;

/// Which bar a row of the left stack is.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bar {
    Health,
    Stamina,
    Food,
    Water,
}

/// The fill inside a bar.
#[derive(Component)]
pub struct BarFill(pub Bar);
/// The stamina bar's strain, hatched past its fill.
#[derive(Component)]
pub struct StrainFill;
/// The thermometer's row, its fill and its reading.
#[derive(Component)]
pub struct Thermo;
#[derive(Component)]
pub struct ThermoFill;
#[derive(Component)]
pub struct ThermoText;
#[derive(Component)]
pub struct ThermoCause;
/// The row of breath bubbles over the hotbar.
#[derive(Component)]
pub struct Bubbles;
#[derive(Component)]
pub struct Bubble(pub usize);
/// "Heat stroke" above the hotbar, and the line under it.
#[derive(Component)]
pub struct HeatWarning;
#[derive(Component)]
pub struct HotbarLine;
#[derive(Component)]
pub struct HotbarLineText;
/// The red edge of a hit, and the dark of passing out.
#[derive(Component)]
pub struct HitEdge;
#[derive(Component)]
pub struct Dark;
/// The left stack itself.
#[derive(Component)]
pub struct LeftStack;
/// The Wake up plaque.
#[derive(Component)]
pub struct WakeUp;
/// A tool slot's wear bar, and its fill.
#[derive(Component)]
pub struct WearBar;
#[derive(Component)]
pub struct WearFill;
/// A dropped pack drawn in the world, by its record id.
#[derive(Component)]
pub struct BagShown(pub u64);

/// The survival icons (from the approved mockup's set, `assets/ui/survival`).
#[derive(Resource, Clone)]
pub struct SurvivalIcons {
    pub heart: Handle<Image>,
    pub stamina: Handle<Image>,
    pub food: Handle<Image>,
    pub water: Handle<Image>,
    pub breath: Handle<Image>,
    pub sun: Handle<Image>,
    pub snow: Handle<Image>,
    pub fire: Handle<Image>,
}

impl SurvivalIcons {
    pub fn cause(&self, cause: TempCause) -> Handle<Image> {
        match cause {
            TempCause::Sun => self.sun.clone(),
            TempCause::Snow => self.snow.clone(),
            TempCause::Fire => self.fire.clone(),
            TempCause::Wet => self.water.clone(),
            // The mockup's set has no wind of its own: wind chill reads as cold.
            TempCause::Wind => self.snow.clone(),
        }
    }
}

pub struct SurvivalPlayPlugin;

impl Plugin for SurvivalPlayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(pbd_app::survival::SurvivalPlugin)
            .add_plugins(super::survival_places::SurvivalPlacesPlugin)
            .add_plugins(super::survival_panels::SurvivalPanelsPlugin)
            .add_systems(PostStartup, spawn_hud)
            .add_systems(
                Update,
                (
                    instrument,
                    recipes_for_mode,
                    fires_near.before(play::SurvivalTick),
                    offer_drink.in_set(pbd_app::interact::InteractSet::Offer),
                    drink.after(pbd_app::interact::InteractSet::Resolve),
                    rain_catch,
                    put_on,
                    wear_from_deeds,
                    (on_death, wake_up, pick_bags).after(play::SurvivalTick),
                    draw_bags,
                    (paint_stack, paint_overlay, paint_wear).after(play::SurvivalTick),
                )
                    .run_if(pbd_app::world_open),
            );
    }
}

// --------------------------------------------------------------- the HUD

fn spawn_hud(mut commands: Commands, kit: Res<UiKit>, assets: Res<AssetServer>) {
    let icons = SurvivalIcons {
        heart: assets.load("ui/survival/heart.png"),
        stamina: assets.load("ui/survival/stamina.png"),
        food: assets.load("ui/survival/roast-meat.png"),
        water: assets.load("ui/survival/water-drop.png"),
        breath: assets.load("ui/survival/breath.png"),
        sun: assets.load("ui/survival/sun.png"),
        snow: assets.load("ui/survival/snowflake.png"),
        fire: assets.load("ui/survival/campfire.png"),
    };
    let kit = &*kit;
    // The red edge and the dark, full screen, under everything else.
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            border: UiRect::all(px(6)),
            ..default()
        },
        BorderColor::all(Color::NONE),
        BackgroundColor(Color::NONE),
        HitEdge,
        InPlayHud,
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            right: px(0),
            top: px(0),
            bottom: px(0),
            ..default()
        },
        BackgroundColor(Color::NONE),
        GlobalZIndex(5),
        Dark,
        InPlayHud,
    ));
    // The left stack, vertically centred at the left edge (G7).
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(6),
                top: percent(50),
                margin: UiRect::top(px(-30)),
                flex_direction: FlexDirection::Column,
                row_gap: px(3),
                align_items: AlignItems::FlexStart,
                ..default()
            },
            // Not `InPlayHud`, whose rule shows it whenever the title is
            // down: this shows itself, in play only.
            Visibility::Hidden,
            LeftStack,
        ))
        .with_children(|stack| {
            for (bar, icon, colour) in [
                (Bar::Health, &icons.heart, hp_colour()),
                (Bar::Stamina, &icons.stamina, st_colour()),
                (Bar::Food, &icons.food, fd_colour()),
                (Bar::Water, &icons.water, wt_colour()),
            ] {
                stack
                    .spawn((
                        Node {
                            align_items: AlignItems::Center,
                            column_gap: px(2),
                            display: Display::None,
                            ..default()
                        },
                        bar,
                    ))
                    .with_children(|row| {
                        row.spawn((
                            ImageNode::new(icon.clone()),
                            Node {
                                width: px(ICON),
                                height: px(ICON),
                                ..default()
                            },
                        ));
                        row.spawn((
                            kit.nine(Set::Stone, "bar"),
                            Node {
                                width: px(BAR_W),
                                height: px(BAR_H),
                                padding: UiRect::all(px(3)),
                                ..default()
                            },
                        ))
                        .with_children(|well| {
                            if bar == Bar::Stamina {
                                well.spawn((
                                    Node {
                                        position_type: PositionType::Absolute,
                                        left: px(3),
                                        top: px(3),
                                        bottom: px(3),
                                        width: px(0),
                                        ..default()
                                    },
                                    BackgroundColor(colour.with_alpha(0.45)),
                                    StrainFill,
                                ));
                            }
                            well.spawn((
                                Node {
                                    height: percent(100),
                                    width: percent(100),
                                    ..default()
                                },
                                BackgroundColor(colour),
                                BarFill(bar),
                            ));
                        });
                    });
            }
            // The thermometer: a tube, the reading and the cause.
            stack
                .spawn((
                    Node {
                        align_items: AlignItems::Center,
                        column_gap: px(2),
                        display: Display::None,
                        ..default()
                    },
                    Thermo,
                ))
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
                            BackgroundColor(fine_colour()),
                            ThermoFill,
                        ));
                    });
                    row.spawn((
                        KitText::new(Localized::empty(), ui_kit::LABEL).outline(ui_kit::INK),
                        ThermoText,
                    ));
                    row.spawn((
                        ImageNode::new(icons.sun.clone()),
                        Node {
                            width: px(10),
                            height: px(10),
                            display: Display::None,
                            ..default()
                        },
                        ThermoCause,
                    ));
                });
        });
    // Breath bubbles over the hotbar.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                margin: UiRect::left(px(-55)),
                bottom: px(super::slots::HOTBAR_BOTTOM + super::slots::SLOT + 22.0),
                column_gap: px(1),
                ..default()
            },
            Visibility::Hidden,
            Bubbles,
        ))
        .with_children(|row| {
            for k in 0..10 {
                row.spawn((
                    ImageNode::new(icons.breath.clone()),
                    Node {
                        width: px(10),
                        height: px(10),
                        ..default()
                    },
                    Bubble(k),
                ));
            }
        });
    // The heat stroke warning, and the line above the hotbar under it.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                bottom: px(super::slots::HOTBAR_BOTTOM + super::slots::SLOT + 6.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            InPlayHud,
        ))
        .with_children(|band| {
            band.spawn((
                Node {
                    align_items: AlignItems::Center,
                    column_gap: px(3),
                    padding: UiRect::new(px(3), px(5), px(2), px(1)),
                    display: Display::None,
                    ..default()
                },
                BackgroundColor(ui_kit::NIGHT.with_alpha(0.55)),
                HeatWarning,
            ))
            .with_children(|w| {
                w.spawn((
                    ImageNode::new(icons.sun.clone()),
                    Node {
                        width: px(11),
                        height: px(11),
                        ..default()
                    },
                ));
                w.spawn(
                    KitText::new(tr("survival.hud.heat_stroke"), heat_colour()).shadow(ui_kit::INK),
                );
            });
            band.spawn((
                Node {
                    padding: UiRect::new(px(4), px(4), px(2), px(1)),
                    display: Display::None,
                    ..default()
                },
                BackgroundColor(ui_kit::NIGHT.with_alpha(0.55)),
                HotbarLine,
            ))
            .with_children(|l| {
                l.spawn((
                    KitText::new(Localized::empty(), ui_kit::LABEL).shadow(ui_kit::INK),
                    HotbarLineText,
                ));
            });
        });
    // "You died" and Wake up: a screen of its own, so it holds the pointer.
    commands
        .spawn((
            super::menu::layer(),
            BackgroundColor(Color::srgba(40.0 / 255.0, 0.0, 0.0, 0.6)),
            GlobalZIndex(10),
            Panel(Screen::Dead),
        ))
        .with_children(|layer| {
            layer
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: px(8),
                    ..default()
                })
                .with_children(|col| {
                    col.spawn(
                        KitText::new(tr("survival.hud.you_died"), ui_kit::LABEL)
                            .shadow(ui_kit::INK)
                            .size(2),
                    );
                    col.spawn((ui_kit::plaque(kit, Set::Stone, 70.0), WakeUp))
                        .with_children(|p| {
                            p.spawn(ui_kit::plaque_label(Set::Stone, tr("survival.hud.wake_up")));
                        });
                });
        });
    commands.insert_resource(icons);
}

fn hp_colour() -> Color {
    Color::srgb_u8(0xd8, 0x44, 0x3a)
}
fn st_colour() -> Color {
    Color::srgb_u8(0xe8, 0xc8, 0x4a)
}
fn fd_colour() -> Color {
    Color::srgb_u8(0xc9, 0x7a, 0x3a)
}
fn wt_colour() -> Color {
    Color::srgb_u8(0x4a, 0xa8, 0xe8)
}
fn fine_colour() -> Color {
    Color::srgb_u8(0xc9, 0xd2, 0x4a)
}
fn heat_colour() -> Color {
    Color::srgb_u8(0xff, 0xb2, 0x7a)
}
/// The thermometer's colour by the body's temperature (the mockup's).
fn temp_colour(t: f32) -> Color {
    if t >= 39.5 {
        Color::srgb_u8(0xe0, 0x50, 0x2a)
    } else if t >= 38.0 {
        Color::srgb_u8(0xe0, 0x89, 0x3a)
    } else if t <= 35.5 {
        Color::srgb_u8(0x7f, 0xc4, 0xff)
    } else {
        fine_colour()
    }
}

/// Seconds a flashing bar spends lit, then dim.
const FLASH_S: f32 = 0.25;

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn paint_stack(
    time: Res<Time>,
    survival: Res<Survival>,
    screen: Res<Screen>,
    icons: Option<Res<SurvivalIcons>>,
    mut stack: Query<&mut Visibility, With<LeftStack>>,
    mut rows: Query<(&Bar, &mut Node), (Without<BarFill>, Without<StrainFill>, Without<Thermo>)>,
    mut fills: Query<(&BarFill, &mut Node, &mut BackgroundColor), Without<Bar>>,
    mut strain: Query<
        (&mut Node, &mut BackgroundColor),
        (With<StrainFill>, Without<Bar>, Without<BarFill>),
    >,
    mut thermo: Query<
        &mut Node,
        (
            With<Thermo>,
            Without<Bar>,
            Without<BarFill>,
            Without<StrainFill>,
            Without<ThermoFill>,
            Without<ThermoCause>,
        ),
    >,
    mut tube: Query<
        (&mut Node, &mut BackgroundColor),
        (
            With<ThermoFill>,
            Without<BarFill>,
            Without<Bar>,
            Without<Thermo>,
            Without<StrainFill>,
        ),
    >,
    mut reading: Query<&mut KitText, With<ThermoText>>,
    mut cause: Query<
        (&mut Node, &mut ImageNode),
        (
            With<ThermoCause>,
            Without<Thermo>,
            Without<ThermoFill>,
            Without<Bar>,
            Without<BarFill>,
            Without<StrainFill>,
        ),
    >,
) {
    let cfg = play::rules();
    let shown_vitals = survival
        .vitals
        .as_ref()
        .filter(|_| !survival.dead && *screen == Screen::Playing);
    for mut v in &mut stack {
        let want = if shown_vitals.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *v != want {
            *v = want;
        }
    }
    let Some(v) = shown_vitals else { return };
    let shown = v.shown(cfg);
    let lit = ((time.elapsed_secs() / FLASH_S) as u32).is_multiple_of(2);
    let share = |bar: Bar| match bar {
        Bar::Health => v.health / cfg.health.max_pts,
        Bar::Stamina => v.stamina.pts / cfg.stamina.max_pts,
        Bar::Food => v.food_fp as f32 / cfg.food.max_fp,
        Bar::Water => v.water as f32 / cfg.water.max,
    };
    let flashing = |bar: Bar| match bar {
        Bar::Health => share(bar) < 0.3,
        Bar::Stamina => v.stamina.winded,
        Bar::Food => share(bar) < 0.15,
        Bar::Water => (v.water as f32) < cfg.water.dry_below,
    };
    for (bar, mut node) in &mut rows {
        let on = match bar {
            Bar::Health => shown.health,
            Bar::Stamina => shown.stamina,
            Bar::Food => shown.food,
            Bar::Water => shown.water,
        };
        let display = if on { Display::Flex } else { Display::None };
        if node.display != display {
            node.display = display;
        }
    }
    for (fill, mut node, mut colour) in &mut fills {
        let s = share(fill.0).clamp(0.0, 1.0);
        node.width = percent(s * 100.0);
        let base = match fill.0 {
            Bar::Health => hp_colour(),
            Bar::Stamina => st_colour(),
            Bar::Food => fd_colour(),
            Bar::Water => wt_colour(),
        };
        colour.0 = if flashing(fill.0) && !lit {
            base.with_alpha(0.3)
        } else {
            base
        };
    }
    // The strain owed, a faint band past the fill (the mockup's hatching),
    // flashing with the bar while winded.
    let strain_alpha = if flashing(Bar::Stamina) && !lit {
        0.08
    } else {
        0.28
    };
    for (mut node, mut colour) in &mut strain {
        let s = ((v.stamina.pts + v.stamina.strain) / cfg.stamina.max_pts).clamp(0.0, 1.0);
        node.width = px((BAR_W - 6.0) * s);
        colour.0 = st_colour().with_alpha(strain_alpha);
    }
    for mut node in &mut thermo {
        let display = if shown.thermometer {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    for (mut node, mut colour) in &mut tube {
        node.height = percent(((v.body_c - 33.0) / 8.0 * 100.0).clamp(8.0, 100.0));
        colour.0 = temp_colour(v.body_c);
    }
    // The kit font's own arrows (it has no degree sign: the unit is the
    // locale's).
    let arrow = if survival.trend_c_per_min > 0.05 {
        "↑"
    } else if survival.trend_c_per_min < -0.05 {
        "↓"
    } else {
        ""
    };
    let c = format!("{:.1}", v.body_c);
    for mut text in &mut reading {
        let reading = pbd_core::locale::tr_with("survival.hud.body_c", &[("c", &c)]);
        let want = Localized::verbatim(format!("{reading}{arrow}"));
        if text.text != want {
            text.text = want;
        }
        text.colour = temp_colour(v.body_c);
    }
    if let Some(icons) = icons {
        for (mut node, mut image) in &mut cause {
            match survival.cause {
                Some(c) => {
                    node.display = Display::Flex;
                    image.image = icons.cause(c);
                }
                None => node.display = Display::None,
            }
        }
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn paint_overlay(
    survival: Res<Survival>,
    screen: Res<Screen>,
    mut bubbles: Query<&mut Visibility, With<Bubbles>>,
    mut bubble: Query<(&Bubble, &mut ImageNode)>,
    mut warning: Query<&mut Node, (With<HeatWarning>, Without<HotbarLine>)>,
    mut line: Query<&mut Node, (With<HotbarLine>, Without<HeatWarning>)>,
    mut line_text: Query<&mut KitText, With<HotbarLineText>>,
    mut edge: Query<&mut BorderColor, With<HitEdge>>,
    mut dark: Query<&mut BackgroundColor, With<Dark>>,
) {
    let cfg = play::rules();
    let playing = *screen == Screen::Playing;
    let v = survival.vitals.as_ref();
    let shown = v.map(|v| v.shown(cfg));
    let breath_shown = playing && shown.is_some_and(|s| s.bubbles);
    for mut vis in &mut bubbles {
        let want = if breath_shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *vis != want {
            *vis = want;
        }
    }
    if let Some(v) = v {
        let air =
            (v.breath.breath_s / cfg.breath.under_s).min(v.breath.suit_s / cfg.suit_air.reserve_s);
        let left = (air.clamp(0.0, 1.0) * 10.0).ceil() as usize;
        for (b, mut image) in &mut bubble {
            image.color = Color::WHITE.with_alpha(if b.0 < left { 1.0 } else { 0.18 });
        }
    }
    let warn = playing && shown.is_some_and(|s| s.heat_warning);
    for mut node in &mut warning {
        node.display = if warn { Display::Flex } else { Display::None };
    }
    let said = survival.line.as_ref().filter(|_| playing && v.is_some());
    for mut node in &mut line {
        node.display = if said.is_some() {
            Display::Flex
        } else {
            Display::None
        };
    }
    if let Some(said) = said {
        for mut text in &mut line_text {
            if text.text != said.text {
                text.text = said.text.clone();
            }
        }
    }
    let red = (survival.hit_s / play::HIT_S).clamp(0.0, 1.0);
    for mut border in &mut edge {
        *border = BorderColor::all(Color::srgba(0.85, 0.1, 0.05, 0.6 * red));
    }
    // The heat's warm edge, then the pass-out's dark (design 14).
    let darkness = v.map_or(0.0, |v| v.consciousness.darkness(cfg));
    for mut bg in &mut dark {
        bg.0 = if darkness > 0.0 {
            Color::srgba(0.0, 0.0, 0.0, 0.92 * darkness)
        } else if shown.is_some_and(|s| s.heat_warning) {
            Color::srgba(0.86, 0.43, 0.12, 0.12)
        } else {
            Color::NONE
        };
    }
}

// -------------------------------------------------------- death (4.2)

/// At a death: the pack drops as one bag where the player fell, and the
/// "You died" screen comes up.
fn on_death(
    mut dawn: Option<ResMut<pbd_app::missions::DawnWatch>>,
    mut survival: ResMut<Survival>,
    mut save: ResMut<WorldSave>,
    mut hotbar: ResMut<Hotbar>,
    mut screen: ResMut<Screen>,
    space: pbd_app::SimSpace,
) {
    if survival.dead && *screen == Screen::Playing {
        *screen = Screen::Dead;
    }
    if !survival.died_now {
        return;
    }
    // A night counts only if it was lived through (design 18).
    if let Some(dawn) = dawn.as_mut() {
        dawn.since_dusk = false;
    }
    let Some(at) = survival.at else { return };
    let o = space.origin();
    let feet = at - o.up_at(at) * HALF_HEIGHT;
    let place = o.to_world(feet);
    if let Some(id) = play::drop_pack(&mut save, &mut hotbar.0, place) {
        info!("SURVIVAL died: the pack dropped as bag {id}");
    }
    survival.died_now = false;
    *screen = Screen::Dead;
}

/// Wake up: full health, half food and water, at the laid bedroll where
/// there is one, else the world's spawn.
fn wake_up(
    mut commands: Commands,
    presses: Query<&Interaction, (Changed<Interaction>, With<WakeUp>)>,
    keys: Res<ButtonInput<KeyCode>>,
    screen: Res<Screen>,
) {
    if *screen != Screen::Dead {
        return;
    }
    let pressed =
        presses.iter().any(|i| *i == Interaction::Pressed) || keys.just_pressed(KeyCode::Enter);
    if pressed {
        commands.queue(|world: &mut World| wake(world));
    }
}

/// Wake the player (exclusive: the walker is moved).
pub fn wake(world: &mut World) {
    let spawn = {
        let mut save = world.resource_mut::<WorldSave>();
        save.survival_act(Act::WakeUp);
        play::spawn_point(&save)
    };
    match spawn {
        Some(at) => pbd_app::walking::restore(
            world,
            pbd_app::walking::RestoredPose {
                position: at.pos.as_vec3(),
                heading: Vec3::Z,
                pitch: 0.0,
            },
        ),
        None => pbd_app::walking::respawn(world),
    }
    {
        let vitals = world.resource::<WorldSave>().vitals();
        let mut survival = world.resource_mut::<Survival>();
        survival.dead = false;
        survival.died_now = false;
        survival.vitals = vitals;
    }
    *world.resource_mut::<Screen>() = Screen::Playing;
    info!("SURVIVAL woke up");
}

/// Walking into a bag picks it up: everything that fits.
fn pick_bags(
    survival: Res<Survival>,
    mut save: ResMut<WorldSave>,
    mut hotbar: ResMut<Hotbar>,
    walkers: Query<&avian3d::prelude::Position, With<Walker>>,
    space: pbd_app::SimSpace,
) {
    if survival.dead || !save.mode().is_survival() {
        return;
    }
    let Ok(position) = walkers.single() else {
        return;
    };
    let o = space.origin();
    let feet = position.0 - o.up_at(position.0) * HALF_HEIGHT;
    for (id, bag) in play::bags(&save) {
        if bag.at.frame != o.to_world(feet).frame {
            continue;
        }
        if o.to_local(bag.at).distance(feet) < play::BAG_REACH_M
            && play::pick_bag(&mut save, &mut hotbar.0, id)
        {
            info!("SURVIVAL picked up bag {id}");
        }
    }
}

/// Every bag lying in the world, drawn as the pack's own icon (a dropped
/// item in the world is its icon): a sack on a card that stands upright on
/// the ground, its foot on it, and turns to the eye, lit by the field as the
/// ground. Placed in the render frame from its body-relative f64 point, as a
/// drop is.
#[allow(clippy::too_many_arguments)]
fn draw_bags(
    mut commands: Commands,
    save: Res<WorldSave>,
    frame: Option<Res<pbd_app::planet::PlanetRenderFrame>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut card: Local<Option<(Handle<Mesh>, Handle<StandardMaterial>)>>,
    mut shown: Query<(Entity, &BagShown, &mut Transform)>,
) {
    let Some(frame) = frame else {
        return;
    };
    let bags = play::bags(&save);
    for (entity, bag, _) in &shown {
        if !bags.iter().any(|(id, _)| *id == bag.0) {
            commands.entity(entity).despawn();
        }
    }
    let eye = cameras
        .iter()
        .find(|(_, camera)| camera.is_active)
        .map(|(transform, _)| transform.translation());
    // Twice a drop's card: a whole pack, standing on its foot.
    let (mesh, material) = card
        .get_or_insert_with(|| {
            let icon = assets.load_with_settings(
                "items/ui/pack-bag.png",
                |s: &mut bevy::image::ImageLoaderSettings| {
                    s.sampler = bevy::image::ImageSampler::nearest();
                },
            );
            (
                meshes.add(Rectangle::new(0.62, 0.62)),
                materials.add(StandardMaterial {
                    base_color_texture: Some(icon),
                    alpha_mode: AlphaMode::Mask(0.5),
                    double_sided: true,
                    cull_mode: None,
                    perceptual_roughness: 0.9,
                    ..default()
                }),
            )
        })
        .clone();
    for (id, bag) in bags {
        let up64 = bag.at.pos.normalize_or(bevy::math::DVec3::Y);
        let up = up64.as_vec3();
        // The card's foot on the ground: the icon's sack stands on its
        // bottom row of pixels (one in sixteen is empty under it).
        let at = (frame.center + bag.at.pos + up64 * 0.29).as_vec3();
        let facing = eye
            .map(|eye| eye - at)
            .map(|toward| toward - up * toward.dot(up))
            .and_then(|flat| flat.try_normalize());
        let transform = match facing {
            Some(facing) => Transform::from_translation(at).looking_to(-facing, up),
            None => {
                Transform::from_translation(at).with_rotation(Quat::from_rotation_arc(Vec3::Y, up))
            }
        };
        if let Some((_, _, mut t)) = shown.iter_mut().find(|(_, b, _)| b.0 == id) {
            *t = transform;
            continue;
        }
        info!("SURVIVAL bag {id} drawn");
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            transform,
            BagShown(id),
            pbd_app::field_light::LitByField,
            pbd_app::field_light::LitLikeTerrain,
        ));
    }
}

// --------------------------------------------------------- drinking (6.6)

/// What the eye is on to drink from.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Drinkable {
    /// Water at a planet point, its surface's altitude.
    Water(Vec3, f32),
}

fn eye_and_look(walking: &WalkingState, at: Vec3, up: Vec3) -> (Vec3, Vec3) {
    let eye = at + up * (EYE_HEIGHT - HALF_HEIGHT);
    let (heading, pitch) = walking.view();
    (eye, heading * pitch.cos() + up * pitch.sin())
}

/// The water the eye is on within reach, or the water the player stands
/// in: a planet point and the surface's altitude.
fn aimed_water(
    contact: &pbd_app::planet::PlanetContact,
    eye: Vec3,
    look: Vec3,
    feet: Vec3,
) -> Option<Drinkable> {
    let radius = pbd_app::planet::PLANET_RADIUS;
    let probe = |p: Vec3| -> Option<f32> {
        let top = contact.column_water(p)??;
        (p.length() - radius < top).then_some(top)
    };
    if let Some(top) = probe(feet + feet.normalize_or(Vec3::Y) * 0.2) {
        return Some(Drinkable::Water(feet, top));
    }
    let mut along = 0.25;
    while along <= DRINK_REACH_M {
        let p = eye + look * along;
        if let Some(top) = probe(p) {
            return Some(Drinkable::Water(p, top));
        }
        if play::solid_at(contact, p) {
            return None;
        }
        along += 0.25;
    }
    None
}

#[allow(clippy::too_many_arguments)]
fn offer_drink(
    save: Res<WorldSave>,
    walking: Option<Res<WalkingState>>,
    hotbar: Res<Hotbar>,
    tools: Res<pbd_app::fish::ToolSlot>,
    contact: Option<Res<pbd_app::planet::PlanetContact>>,
    walkers: Query<&avian3d::prelude::Position, With<Walker>>,
    mut interact: ResMut<Interact>,
    space: pbd_app::SimSpace,
) {
    if !save.mode().is_survival() {
        return;
    }
    let (Some(walking), Some(contact)) = (walking, contact) else {
        return;
    };
    // An empty hand, as the design has it.
    if hotbar.0.held().is_some() || tools.0.held().is_some() {
        return;
    }
    let Ok(position) = walkers.single() else {
        return;
    };
    let o = space.origin();
    let up = o.up_at(position.0);
    let at = o.query_point(position.0);
    let (eye, look) = eye_and_look(&walking, at, up);
    if aimed_water(&contact, eye, look, at - up * HALF_HEIGHT).is_some() {
        interact.offer(Offer::new(Verb::Drink, tr("survival.water")));
    }
}

/// F at water with an empty hand: one sip. Salt water gives nothing and
/// says so.
#[allow(clippy::too_many_arguments)]
fn drink(
    keys: Res<ButtonInput<KeyCode>>,
    interact: Option<Res<Interact>>,
    mut save: ResMut<WorldSave>,
    walking: Option<Res<WalkingState>>,
    hotbar: Res<Hotbar>,
    tools: Res<pbd_app::fish::ToolSlot>,
    contact: Option<Res<pbd_app::planet::PlanetContact>>,
    walkers: Query<&avian3d::prelude::Position, With<Walker>>,
    mut survival: ResMut<Survival>,
    mut did: MessageWriter<pbd_app::missions::Did>,
    space: pbd_app::SimSpace,
    screen: Res<Screen>,
) {
    if !keys.just_pressed(KeyCode::KeyF) || *screen != Screen::Playing {
        return;
    }
    if !save.mode().is_survival() || !pbd_app::interact::may(interact.as_deref(), Reader::Survival)
    {
        return;
    }
    // The still, the cactus and the bedroll are the places' to answer.
    if interact
        .as_deref()
        .and_then(|i| i.now.as_ref())
        .is_some_and(|o| o.act != Verb::Drink)
    {
        return;
    }
    let (Some(walking), Some(contact)) = (walking, contact) else {
        return;
    };
    if hotbar.0.held().is_some() || tools.0.held().is_some() {
        return;
    }
    let Ok(position) = walkers.single() else {
        return;
    };
    let o = space.origin();
    let up = o.up_at(position.0);
    let at = o.query_point(position.0);
    let (eye, look) = eye_and_look(&walking, at, up);
    let Some(Drinkable::Water(point, top)) =
        aimed_water(&contact, eye, look, at - up * HALF_HEIGHT)
    else {
        return;
    };
    let source = play::water_source_in(&save, point, top);
    sip_from(&mut save, &mut survival, &mut did, source);
}

/// A sip from `source`: into the bar through the server, its note shown,
/// and a drink counted for the missions where it is fresh.
fn sip_from(
    save: &mut WorldSave,
    survival: &mut Survival,
    did: &mut MessageWriter<pbd_app::missions::Did>,
    source: WaterSource,
) {
    let cfg = play::rules();
    let now = save.vitals().map_or(0.0, |v| v.water as f32);
    let sip = pbd_core::survival::water::sip(cfg, source, now);
    info!("SURVIVAL drink {source:?}: +{}", sip.water);
    if let Some(note) = sip.note {
        survival.say(note.text());
    }
    let counts = sip.counts;
    save.survival_act(Act::Drink(sip));
    survival.vitals = save.vitals();
    if counts && let Some(name) = pbd_core::missions::drink_name(source) {
        did.write(pbd_app::missions::Did::new(
            pbd_core::missions::Kind::Drink,
            name,
            1,
        ));
    }
}

/// Rain caught in an empty hand held up (design 15): the right button held
/// in the rain, under open sky, gives the rain's catch a second.
#[allow(clippy::too_many_arguments)]
fn rain_catch(
    time: Res<Time>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut save: ResMut<WorldSave>,
    hotbar: Res<Hotbar>,
    tools: Res<pbd_app::fish::ToolSlot>,
    air: Option<Res<pbd_app::atmosphere::Air>>,
    contact: Option<Res<pbd_app::planet::PlanetContact>>,
    walkers: Query<&avian3d::prelude::Position, With<Walker>>,
    mut survival: ResMut<Survival>,
    mut did: MessageWriter<pbd_app::missions::Did>,
    mut caught_s: Local<f32>,
    space: pbd_app::SimSpace,
    screen: Res<Screen>,
) {
    if *screen != Screen::Playing || !save.mode().is_survival() {
        return;
    }
    if !buttons.pressed(MouseButton::Right) || hotbar.0.held().is_some() || tools.0.held().is_some()
    {
        return;
    }
    let (Some(air), Some(contact), Ok(position)) = (air, contact, walkers.single()) else {
        return;
    };
    let o = space.origin();
    let up = o.up_at(position.0);
    let sample = air.now.sample(up);
    let raining = sample.rain_rate > 0.0 || sample.rain_seen > 0.05;
    let open = contact.open_to_sky(o.query_point(position.0) + up);
    let dt = time.delta_secs();
    let water = pbd_core::survival::water::rain_catch(play::rules(), raining && open, dt);
    if water <= 0.0 {
        return;
    }
    let sip = pbd_core::survival::water::Sip {
        water,
        sip_s: dt,
        note: None,
        counts: true,
    };
    save.survival_act(Act::Drink(sip));
    survival.vitals = save.vitals();
    *caught_s += dt;
    if *caught_s >= RAIN_COUNT_S {
        *caught_s = 0.0;
        did.write(pbd_app::missions::Did::new(
            pbd_core::missions::Kind::Drink,
            "rain",
            1,
        ));
    }
}

/// The right button with a straw hat or a wool cloak in the selected slot
/// puts it on (design 13): what was worn there comes back to the hand.
fn put_on(
    buttons: Res<ButtonInput<MouseButton>>,
    screen: Res<Screen>,
    walking: Option<Res<pbd_app::walking::WalkingReadout>>,
    mut save: ResMut<WorldSave>,
    mut hotbar: ResMut<Hotbar>,
    mut did: MessageWriter<pbd_app::missions::Did>,
) {
    if *screen != Screen::Playing || !buttons.just_pressed(MouseButton::Right) {
        return;
    }
    if !walking.is_some_and(|w| w.active && w.captured) {
        return;
    }
    let Some(pbd_core::inventory::Item::Good(good)) = hotbar.0.held().map(|s| s.item) else {
        return;
    };
    if play::wear_held(&mut save, &mut hotbar.0) {
        info!("SURVIVAL put on: {:?}", play::WornRecord::of(&save));
        did.write(pbd_app::missions::Did::new(
            pbd_core::missions::Kind::Wear,
            good.code(),
            1,
        ));
    }
}

/// The fires near the player (design 13): the placed campfires within
/// their reach, looked for twice a second.
fn fires_near(
    time: Res<Time>,
    save: Res<WorldSave>,
    fine: Option<Res<pbd_app::planet::PlanetFine>>,
    walkers: Query<&avian3d::prelude::Position, With<Walker>>,
    mut fires: ResMut<play::FiresNear>,
    mut every: Local<f32>,
    space: pbd_app::SimSpace,
) {
    if !save.mode().is_survival() {
        return;
    }
    *every -= time.delta_secs();
    if *every > 0.0 {
        return;
    }
    *every = 0.5;
    fires.0.clear();
    let (Some(fine), Ok(feet)) = (fine, walkers.single()) else {
        return;
    };
    let campfires: Vec<_> = save
        .edits
        .all()
        .into_iter()
        .filter(|e| e.material == Material::Campfire)
        .collect();
    if campfires.is_empty() {
        return;
    }
    let toward = space.origin().up_at(feet.0);
    for record in fine.set.finest_records() {
        if !campfires.iter().any(|e| e.cell == record.key()) {
            continue;
        }
        let distance = Vec3::from_slice(&record.direction_height[..3])
            .normalize_or(Vec3::Y)
            .distance(toward)
            * pbd_app::planet::PLANET_RADIUS;
        if distance < 6.0 {
            fires.0.push(FireNear {
                source: "block:campfire".into(),
                distance_m: distance,
            });
        }
    }
}

// ------------------------------------------------------------ wear (4.8)

/// A finished action by the tool in hand spends its uses (design 11): read
/// off what the missions count, so every dig, fell, till and catch is
/// already there. A tool that breaks says so once.
fn wear_from_deeds(
    mut deeds: MessageReader<pbd_app::missions::Did>,
    mut save: ResMut<WorldSave>,
    tools: Res<pbd_app::fish::ToolSlot>,
    mut survival: ResMut<Survival>,
    (sfx, listener): (
        Option<ResMut<pbd_app::audio::Sfx>>,
        Option<Res<pbd_app::audio::Listener>>,
    ),
) {
    let mut sfx = sfx;
    let deeds: Vec<_> = deeds.read().cloned().collect();
    if !save.mode().is_survival() {
        return;
    }
    let Some(tool) = tools.0.held() else { return };
    let Some(tier) = tools.0.tier(tool) else {
        return;
    };
    for deed in deeds {
        let h = &deed.0;
        let action = match (h.kind, tool) {
            (pbd_core::missions::Kind::Gather, Tool::Pickaxe | Tool::Shovel | Tool::Axe)
                if h.what.starts_with("block:") =>
            {
                Action::BlockBroken { block_tier: None }
            }
            (pbd_core::missions::Kind::Fell, Tool::Axe) => Action::TreeFelled,
            (pbd_core::missions::Kind::Till, _) => Action::Tilled,
            (pbd_core::missions::Kind::Catch, Tool::Rod) => Action::FishLanded,
            _ => continue,
        };
        if play::WearBook::of(&save).broken(tool, tier) {
            continue;
        }
        if play::spend_wear(&mut save, tool, tier, action) == Spend::Broke {
            survival.say(pbd_core::locale::tr_with(
                "survival.broke",
                &[("tool", &pbd_core::inventory::tool_title(tool, tier))],
            ));
            info!("SURVIVAL the {tool:?} broke");
            // The crack, in the hand (design 11).
            if let (Some(sfx), Some(listener)) = (sfx.as_deref_mut(), listener.as_deref()) {
                sfx.play(
                    "tool_break",
                    pbd_app::audio::Where::At(listener.eye + listener.forward * 0.6),
                );
            }
        }
    }
}

/// The wear bar along the bottom of the tool slot (design 11), Survival
/// only, once the tool is below full; red below a tenth.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn paint_wear(
    mut commands: Commands,
    save: Res<WorldSave>,
    tools: Res<pbd_app::fish::ToolSlot>,
    cells: Query<(Entity, &Children), With<super::equipment::ToolIcon>>,
    icon_cells: Query<Entity, (With<super::equipment::ToolIcon>, Without<Children>)>,
    mut bars: Query<(&mut Node, &mut Visibility), (With<WearBar>, Without<WearFill>)>,
    mut fills: Query<(&mut Node, &mut BackgroundColor), (With<WearFill>, Without<WearBar>)>,
    mut icons: Query<&mut ImageNode, With<super::equipment::ToolIcon>>,
    has_bar: Query<(), With<WearBar>>,
    mut held_broken: ResMut<play::HeldBroken>,
) {
    // Give the tool icon its bar once.
    for entity in &icon_cells {
        commands.entity(entity).with_children(|cell| {
            cell.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(1),
                    right: px(1),
                    bottom: px(-2),
                    height: px(2),
                    ..default()
                },
                BackgroundColor(ui_kit::WELL),
                Visibility::Hidden,
                WearBar,
            ))
            .with_children(|bar| {
                bar.spawn((
                    Node {
                        width: percent(100),
                        height: percent(100),
                        ..default()
                    },
                    BackgroundColor(ui_kit::RUNE),
                    WearFill,
                ));
            });
        });
    }
    for (entity, children) in &cells {
        if !children.iter().any(|c| has_bar.contains(c)) {
            commands.entity(entity).with_children(|cell| {
                cell.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(1),
                        right: px(1),
                        bottom: px(-2),
                        height: px(2),
                        ..default()
                    },
                    BackgroundColor(ui_kit::WELL),
                    Visibility::Hidden,
                    WearBar,
                ))
                .with_children(|bar| {
                    bar.spawn((
                        Node {
                            width: percent(100),
                            height: percent(100),
                            ..default()
                        },
                        BackgroundColor(ui_kit::RUNE),
                        WearFill,
                    ));
                });
            });
        }
    }
    let cfg = play::rules();
    let held = tools.0.held().and_then(|t| Some((t, tools.0.tier(t)?)));
    let state = held
        .filter(|_| save.mode().is_survival())
        .map(|(tool, tier)| {
            let wear = play::WearBook::of(&save).wear(tool, tier);
            let wearer = pbd_core::survival::wear::Wearer::Tool(tool, tier);
            (wear.bar(cfg, &wearer), wear.is_broken())
        });
    for (_, mut vis) in &mut bars {
        let want = if state.is_some_and(|(bar, _)| bar.is_some()) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *vis != want {
            *vis = want;
        }
    }
    if let Some((Some((share, low)), _)) = state {
        for (mut node, mut colour) in &mut fills {
            node.width = percent(share.clamp(0.0, 1.0) * 100.0);
            colour.0 = if low { hp_colour() } else { ui_kit::RUNE };
        }
    }
    // A broken tool's icon is dimmed and greyed, as the mockup's, and it
    // digs as bare hands.
    let broken = state.is_some_and(|(_, b)| b);
    if held_broken.0 != broken {
        held_broken.0 = broken;
    }
    for mut image in &mut icons {
        let want = if broken {
            Color::srgb(0.45, 0.45, 0.45)
        } else {
            Color::WHITE
        };
        if image.color != want {
            image.color = want;
        }
    }
}

/// The open world's recipe book offers Survival's recipes (the spears, the
/// bow, the tanning rack, the still, the bedroll, the stew) only in a
/// Survival world (`survival-mode` task 4.10's `RecipeBook.survival`).
fn recipes_for_mode(save: Res<WorldSave>, recipes: Option<ResMut<pbd_app::config::Recipes>>) {
    let Some(mut recipes) = recipes else { return };
    let survival = save.mode().is_survival();
    if recipes.0.survival != survival {
        recipes.0.survival = survival;
        info!("recipes: survival {survival}");
    }
}

// --------------------------------------------- a capture instrument

/// MEASUREMENT (capture instrument, `survival-mode` task 4.6): with
/// `PBD_SURVIVAL_STATE=<state>` a capture's world is made a Survival one
/// and its bars set to a HUD state of the approved mockup, so each state's
/// `--capture` shot shows it: `full`, `thirsty50`, `thirsty25`, `winded`,
/// `hot` (the heat stroke warning), `under` (breath), `dead` (the pack
/// dropped), `out` (passed out from heat stroke), `hit` (the hit's red
/// edge), `worn` (a worn and then a broken pickaxe), `bag` (the pack
/// dropped a step ahead with the player alive, for its look and its map
/// marker), `bedroll` (a bedroll laid a step ahead), `still` (every still
/// placed by `--place-row` given four sips of sea water and three minutes
/// of boiling, so three are fresh). Once, at the start.
fn instrument(
    mut commands: Commands,
    mut save: ResMut<WorldSave>,
    mut done: Local<u32>,
    time: Res<Time>,
) {
    let Ok(state) = std::env::var("PBD_SURVIVAL_STATE") else {
        return;
    };
    if *done == 0 {
        save.identity.mode = Some(pbd_core::survival::WorldMode::Survival);
        *done = 1;
        return;
    }
    // A moment in, once the server has the player; then held there each
    // frame, so a state the rules would move (a winded bar refills) is the
    // one the shot shows. A death and a broken tool happen once.
    // A still is placed by `--place-row` a few seconds in: fill it after.
    let after_s = if state == "still" { 7.0 } else { 1.0 };
    if time.elapsed_secs() < after_s {
        return;
    }
    let once = matches!(state.as_str(), "dead" | "bag" | "bedroll" | "still" | "out");
    if once && *done >= 2 {
        return;
    }
    let first = *done == 1;
    *done = 2;
    commands.queue(move |world: &mut World| set_state(world, &state, first));
}

/// The capture's bag or bedroll, a step ahead of the walker.
fn place_ahead(world: &mut World, state: &str) {
    let space_origin = pbd_app::sim_origin::origin_of(world);
    let Some(position) = world
        .query_filtered::<&avian3d::prelude::Position, With<Walker>>()
        .iter(world)
        .next()
        .map(|p| p.0)
    else {
        return;
    };
    let up = space_origin.up_at(position);
    let heading = world.resource::<WalkingState>().view().0;
    let ground = {
        let contact = world.resource::<pbd_app::planet::PlanetContact>();
        let mut down = 0.0;
        let start = position - up * HALF_HEIGHT + heading * 2.6 + up * 1.2;
        let mut found = None;
        while down < 3.0 {
            let p = start - up * down;
            if play::solid_at(contact, space_origin.query_point(p)) {
                found = Some(p + up * 0.04);
                break;
            }
            down += 0.02;
        }
        found
    };
    let Some(ground) = ground else {
        warn!("PBD_SURVIVAL_STATE {state}: no ground ahead");
        return;
    };
    let at = space_origin.to_world(ground);
    let mut hotbar = world.resource::<Hotbar>().0.clone();
    let mut save = world.resource_mut::<WorldSave>();
    if state == "bedroll" {
        hotbar.give(
            pbd_core::inventory::Item::Good(pbd_core::goods::Good::BEDROLL),
            1,
        );
        let at_bedroll = (0..pbd_core::inventory::CARRIED).find(|&i| {
            hotbar.get(i).map(|s| s.item)
                == Some(pbd_core::inventory::Item::Good(
                    pbd_core::goods::Good::BEDROLL,
                ))
        });
        if let Some(i) = at_bedroll {
            hotbar.select(i);
        }
        play::places::lay_bedroll(&mut save, &mut hotbar, at, heading);
    } else {
        play::drop_pack(&mut save, &mut hotbar, at);
    }
    world.resource_mut::<Hotbar>().0 = hotbar;
}

fn set_state(world: &mut World, state: &str, first: bool) {
    let cfg = play::rules();
    let mut v = pbd_core::survival::Vitals::new(cfg);
    let mut line = None;
    let mut die = false;
    let mut hit = false;
    match state {
        "full" => {}
        "thirsty50" => {
            v.water = 50.0;
            line = Some(pbd_core::survival::Note::Thirsty);
        }
        "thirsty25" => {
            v.water = 25.0;
            v.food_fp = 42.0;
            v.health = 88.0;
            line = Some(pbd_core::survival::Note::VeryThirsty);
        }
        "winded" => {
            v.stamina.pts = 0.0;
            v.stamina.strain = 60.0;
            v.stamina.winded = true;
            v.health = 90.0;
        }
        "hot" => {
            v.body_c = 39.7;
            v.water = 4.0;
            v.health = 71.0;
            v.stamina.pts = 84.0;
        }
        "cold" => {
            v.body_c = 34.8;
            v.health = 80.0;
            v.food_fp = 30.0;
        }
        "under" => {
            v.breath.breath_s = 12.0;
            v.stamina.pts = 70.0;
        }
        "dead" => die = true,
        // Passed out from heat stroke, held there.
        "out" => {
            v.body_c = 40.1;
            v.water = 3.0;
            v.health = 64.0;
            v.consciousness = pbd_core::survival::passout::Consciousness::pass_out(cfg);
        }
        // Just hit: the red edge, held there.
        "hit" => {
            v.health = 72.0;
            hit = true;
        }
        "salt" => line = Some(pbd_core::survival::Note::SaltWater),
        "bag" | "bedroll" => place_ahead(world, state),
        "still" => {
            let mut save = world.resource_mut::<WorldSave>();
            let stills: Vec<_> = save
                .edits
                .all()
                .into_iter()
                .filter(|e| e.material == Material::Still)
                .map(|e| (e.cell, e.layer))
                .collect();
            for at in stills {
                let mut hands = pbd_core::inventory::Slots::new();
                hands.give(
                    pbd_core::inventory::Item::Good(pbd_core::goods::Good::SEA_BUCKET),
                    1,
                );
                play::places::use_still(&mut save, &mut hands, at, 50.0);
                // Three minutes of boiling beside the fire, so the shot
                // shows its water in the cup.
                let mut still = play::places::still_at(&save, at.0, at.1);
                play::places::step_still(&mut save, at, &mut still, true, false, 180.0);
            }
        }
        "worn" => {
            // Each tool at the tier the slot holds (once the launch has
            // given it): the pickaxe to a quarter of its uses, the axe to
            // breaking.
            let tools = world.resource::<pbd_app::fish::ToolSlot>().0;
            let mut save = world.resource_mut::<WorldSave>();
            for (tool, target) in [(Tool::Pickaxe, 0.25), (Tool::Axe, 0.0)] {
                let Some(tier) = tools.tier(tool) else {
                    continue;
                };
                let wearer = pbd_core::survival::wear::Wearer::Tool(tool, tier);
                while play::WearBook::of(&save)
                    .wear(tool, tier)
                    .share(cfg, &wearer)
                    > target
                    && !play::WearBook::of(&save).broken(tool, tier)
                {
                    if play::spend_wear(
                        &mut save,
                        tool,
                        tier,
                        Action::BlockBroken { block_tier: None },
                    ) == Spend::None
                    {
                        break;
                    }
                }
            }
        }
        _ => warn!("PBD_SURVIVAL_STATE {state}: no such state"),
    }
    {
        let mut save = world.resource_mut::<WorldSave>();
        if state != "worn" || first {
            save.put_vitals(v);
        }
        if die {
            save.survival_act(Act::Hit(pbd_core::survival::health::Cause::Fall, 200.0));
        }
    }
    let vitals = world.resource::<WorldSave>().vitals();
    let mut survival = world.resource_mut::<Survival>();
    survival.vitals = vitals;
    if let Some(note) = line {
        survival.say(note.text());
    }
    if die {
        survival.dead = true;
        survival.died_now = true;
    }
    if hit {
        survival.hit_s = play::HIT_S;
    }
    if first {
        info!("SURVIVAL_STATE {state}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn warning_shown(body_c: Option<f32>) -> bool {
        let mut app = App::new();
        let vitals = body_c.map(|c| {
            let mut v = pbd_core::survival::Vitals::new(play::rules());
            v.body_c = c;
            v
        });
        app.insert_resource(Screen::Playing)
            .insert_resource(Survival {
                vitals,
                ..default()
            })
            .add_systems(Update, paint_overlay);
        app.world_mut().spawn((Node::default(), HeatWarning));
        app.update();
        app.world_mut()
            .query_filtered::<&Node, With<HeatWarning>>()
            .single(app.world())
            .unwrap()
            .display
            == Display::Flex
    }

    #[test]
    fn the_heat_warning_shows_past_39_5_c_and_goes_below_it() {
        assert!(!warning_shown(Some(37.0)));
        assert!(warning_shown(Some(39.6)));
        assert!(warning_shown(Some(40.2)), "and through heat stroke");
        assert!(!warning_shown(Some(39.4)));
        assert!(!warning_shown(None), "Explorer shows none");
    }

    #[test]
    fn every_hud_word_is_in_the_locale_file() {
        for key in [
            "survival.hud.heat_stroke",
            "survival.hud.you_died",
            "survival.hud.wake_up",
            "survival.water",
        ] {
            assert_ne!(tr(key).to_string(), key, "{key}");
        }
    }
}
