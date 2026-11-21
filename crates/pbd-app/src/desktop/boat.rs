//! The boat's menu (`river-craft` design 10; the owner, 2026-11-19: "rope
//! length can be controlled in the boat menu, yes boats have menus for
//! inventory and various stuff").
//!
//! C aboard a boat, or on foot within reach of one, opens the pack with the
//! boat's own panel beside its HOLD: the boat's name and whether water is
//! aboard, its one anchor (its state, Drop or Weigh, and the rope's length
//! from "To the bed" up to the rode carried) and its lantern. The hold is
//! the pack's own HOLD panel and clicks (`pack`): one rule, one view. C or
//! Esc closes it. Plain labels only: no notes (CLAUDE.md).

use super::menu::Screen;
use bevy::prelude::*;
use pbd_app::ui_kit::{self, KitText, Localized, Set, UiKit, tr, tr_with};
use pbd_app::vehicles::{AnchorOrder, Fleet, Vehicle};
use pbd_core::vehicle::anchor;

/// Water aboard, kg, over which the boat reads "Water aboard".
const WET_KG: f64 = 20.0;

/// The rope's step for + and -, m.
const ROPE_STEP_M: f64 = 1.0;

/// The boat whose menu is open, if it is.
#[derive(Resource, Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoatMenu(pub Option<Entity>);

/// The boat's panel, beside the HOLD.
#[derive(Component)]
pub struct BoatPanel;

/// What a line of the panel shows.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum BoatText {
    Name,
    Condition,
    Anchor,
    Rope,
    Out,
    Lantern,
}

/// What a button does.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum BoatAct {
    /// Drop the anchor or weigh it.
    Anchor,
    Shorter,
    Longer,
    ToTheBed,
    Lantern,
}

/// The lantern's row, shown only on a boat with one.
#[derive(Component)]
pub struct LanternRow;

/// The lantern's check.
#[derive(Component)]
pub struct LanternCheck;

/// The nearest boat with an anchor in reach of the walker, by the reach
/// that boards it (`interact`).
fn in_reach(
    vehicles: &Query<(Entity, &mut Vehicle)>,
    walker: Option<bevy::math::DVec3>,
) -> Option<Entity> {
    let walker = walker?;
    vehicles
        .iter()
        .filter(|(_, v)| v.craft.has_anchor())
        .map(|(e, v)| (e, v.craft.body.position.distance(walker), v))
        .filter(|(_, d, v)| *d <= f64::from(v.craft.seat().reach_m))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(e, _, _)| e)
}

/// C opens the boat's menu aboard a boat with one, or on foot by one, and
/// shuts it again. `PreUpdate` beside `pack::open`.
#[allow(clippy::too_many_arguments)]
pub fn open(
    keys: Res<ButtonInput<KeyCode>>,
    mut screen: ResMut<Screen>,
    mut menu: ResMut<BoatMenu>,
    walking: Option<Res<pbd_app::walking::WalkingReadout>>,
    aboard: Option<Res<pbd_app::vehicles::Aboard>>,
    vehicles: Query<(Entity, &mut Vehicle)>,
    walkers: Query<&avian3d::prelude::Position, With<pbd_app::walking::Walker>>,
    frame: Option<Res<pbd_app::planet::PlanetRenderFrame>>,
) {
    if *screen != Screen::Pack && menu.0.is_some() {
        menu.0 = None;
    }
    if !keys.just_pressed(KeyCode::KeyC) {
        return;
    }
    if *screen == Screen::Pack && menu.0.is_some() {
        *screen = Screen::Playing;
        menu.0 = None;
        return;
    }
    if *screen != Screen::Playing {
        return;
    }
    let boat = if let Some(entity) = aboard.and_then(|a| a.0) {
        vehicles
            .get(entity)
            .ok()
            .filter(|(_, v)| v.craft.has_anchor())
            .map(|(e, _)| e)
    } else if walking.is_some_and(|w| w.active) {
        let centre = frame
            .as_deref()
            .copied()
            .unwrap_or_default()
            .physics_center();
        let feet = walkers.iter().next().map(|p| p.0.as_dvec3() - centre);
        in_reach(&vehicles, feet)
    } else {
        None
    };
    if let Some(boat) = boat {
        menu.0 = Some(boat);
        *screen = Screen::Pack;
    }
}

/// A row: a label and what follows it.
fn row(
    parent: &mut ChildSpawnerCommands,
    label: Localized,
    rest: impl FnOnce(&mut ChildSpawnerCommands),
) {
    parent
        .spawn(Node {
            align_items: AlignItems::Center,
            column_gap: px(6),
            min_height: px(24),
            ..default()
        })
        .with_children(|line| {
            line.spawn((
                KitText::new(label, ui_kit::TEXT_DIM).shadow(ui_kit::INK),
                Node {
                    width: px(52),
                    ..default()
                },
            ));
            rest(line);
        });
}

fn button(
    line: &mut ChildSpawnerCommands,
    kit: &UiKit,
    act: BoatAct,
    label: Localized,
    width: f32,
) {
    line.spawn((ui_kit::plaque(kit, Set::Stone, width), act))
        .with_children(|b| {
            b.spawn(ui_kit::plaque_label(Set::Stone, label));
        });
}

/// The boat's panel, under the pack beside the HOLD; hidden until C.
pub fn spawn(
    mut commands: Commands,
    kit: Res<UiKit>,
    rows: Query<Entity, With<super::pack::HoldRow>>,
) {
    let kit = &*kit;
    let Some(layer) = rows.iter().next() else {
        return;
    };
    let panel = commands
        .spawn((
            kit.nine(Set::Stone, "frame"),
            Node {
                width: px(250),
                padding: UiRect::new(px(14), px(14), px(14), px(10)),
                flex_direction: FlexDirection::Column,
                row_gap: px(2),
                display: Display::None,
                ..default()
            },
            BoatPanel,
        ))
        .with_children(|panel| {
            super::menu::lintel(panel, kit, tr("boat.boat"));
            panel
                .spawn(Node {
                    justify_content: JustifyContent::Center,
                    margin: UiRect::bottom(px(4)),
                    ..default()
                })
                .with_children(|line| {
                    line.spawn((
                        KitText::new(Localized::empty(), ui_kit::TITLE).shadow(ui_kit::INK),
                        BoatText::Name,
                    ));
                    line.spawn((
                        KitText::new(Localized::empty(), ui_kit::TEXT).shadow(ui_kit::INK),
                        BoatText::Condition,
                        Node {
                            margin: UiRect::left(px(8)),
                            ..default()
                        },
                    ));
                });
            row(panel, tr("boat.anchor"), |line| {
                line.spawn((
                    KitText::new(Localized::empty(), ui_kit::TEXT).shadow(ui_kit::INK),
                    BoatText::Anchor,
                    Node {
                        width: px(84),
                        ..default()
                    },
                ));
                button(line, kit, BoatAct::Anchor, tr("boat.drop"), 64.0);
            });
            row(panel, tr("boat.rope"), |line| {
                button(line, kit, BoatAct::Shorter, Localized::verbatim("-"), 24.0);
                line.spawn((
                    KitText::new(Localized::empty(), ui_kit::LABEL).shadow(ui_kit::INK),
                    BoatText::Rope,
                    Node {
                        min_width: px(66),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                ));
                button(line, kit, BoatAct::Longer, Localized::verbatim("+"), 24.0);
            });
            row(panel, Localized::empty(), |line| {
                button(line, kit, BoatAct::ToTheBed, tr("boat.to_the_bed"), 90.0);
                line.spawn((
                    KitText::new(Localized::empty(), ui_kit::TEXT_DIM).shadow(ui_kit::INK),
                    BoatText::Out,
                ));
            });
            panel
                .spawn((
                    Node {
                        align_items: AlignItems::Center,
                        column_gap: px(6),
                        min_height: px(24),
                        ..default()
                    },
                    LanternRow,
                ))
                .with_children(|line| {
                    line.spawn((
                        KitText::new(tr("boat.lantern"), ui_kit::TEXT_DIM).shadow(ui_kit::INK),
                        Node {
                            width: px(52),
                            ..default()
                        },
                    ));
                    line.spawn((
                        Button,
                        BoatAct::Lantern,
                        LanternCheck,
                        kit.whole(Set::Stone, "check-on"),
                        Node {
                            width: px(16),
                            height: px(16),
                            ..default()
                        },
                    ));
                    line.spawn((
                        KitText::new(Localized::empty(), ui_kit::TEXT).shadow(ui_kit::INK),
                        BoatText::Lantern,
                    ));
                });
        })
        .id();
    // Beside the HOLD, in the row under the pack.
    commands.entity(layer).add_child(panel);
}

/// Show the panel while the menu is open on a boat, and say what it is.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn paint(
    screen: Res<Screen>,
    menu: Res<BoatMenu>,
    kit: Res<UiKit>,
    vehicles: Query<(Entity, &mut Vehicle)>,
    mut panels: Query<&mut Node, (With<BoatPanel>, Without<LanternRow>)>,
    mut lantern_rows: Query<&mut Node, (With<LanternRow>, Without<BoatPanel>)>,
    mut texts: Query<(&BoatText, &mut KitText)>,
    mut buttons: Query<(&BoatAct, &Children, &mut ui_kit::Plaque)>,
    mut labels: Query<&mut KitText, (With<ui_kit::PlaqueLabel>, Without<BoatText>)>,
    mut checks: Query<&mut ImageNode, With<LanternCheck>>,
) {
    let craft = menu
        .0
        .filter(|_| *screen == Screen::Pack)
        .and_then(|e| vehicles.get(e).ok())
        .map(|(_, v)| &v.craft);
    let want = if craft.is_some() {
        Display::Flex
    } else {
        Display::None
    };
    for mut node in &mut panels {
        if node.display != want {
            node.display = want;
        }
    }
    let Some(craft) = craft else {
        return;
    };
    let has_lamp = craft.loon_spec().lamp.is_some();
    for mut node in &mut lantern_rows {
        let want = if has_lamp {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != want {
            node.display = want;
        }
    }
    let out = craft.anchor.is_some() || craft.mooring.is_some_and(|m| m.anchored);
    let rode = craft
        .anchor
        .map(|a| a.rode)
        .or_else(|| craft.mooring.filter(|m| m.anchored).map(|m| m.length));
    for (what, mut text) in &mut texts {
        let words = match what {
            BoatText::Name => craft.kind.title(),
            BoatText::Condition => tr(if craft.bilge_kg > WET_KG {
                "boat.water_aboard"
            } else {
                "boat.dry"
            }),
            BoatText::Anchor => {
                if craft.anchor.is_none() && out {
                    tr("vehicle.anchor.holding")
                } else {
                    anchor::word(craft.anchor.as_ref())
                }
            }
            BoatText::Rope => match craft.rope_m {
                None => tr("boat.to_the_bed"),
                Some(m) => tr_with("boat.rope_m", &[("metres", &format!("{m:.0}"))]),
            },
            BoatText::Out => rode.map_or_else(Localized::empty, |r| {
                tr_with("boat.out_m", &[("metres", &format!("{r:.1}"))])
            }),
            BoatText::Lantern => tr(if craft.lantern_out {
                "boat.lantern_out"
            } else {
                "boat.lantern_on"
            }),
        };
        if text.text != words {
            text.text = words;
        }
    }
    let weighing = matches!(
        craft.anchor.map(|a| a.phase),
        Some(anchor::Phase::Down { weighing: true, .. })
    );
    let max = f64::from(craft.anchor_spec().rode_max_m);
    for (act, children, mut plaque) in &mut buttons {
        let (label, disabled) = match act {
            BoatAct::Anchor => {
                let busy = matches!(
                    craft.anchor.map(|a| a.phase),
                    Some(anchor::Phase::Dropping { .. } | anchor::Phase::Up)
                );
                let label = tr(if weighing {
                    "boat.stop"
                } else if out {
                    "boat.weigh"
                } else {
                    "boat.drop"
                });
                (Some(label), busy)
            }
            BoatAct::Shorter => (None, craft.rope_m.is_none()),
            BoatAct::Longer => (None, craft.rope_m.is_some_and(|m| m >= max)),
            BoatAct::ToTheBed => (None, craft.rope_m.is_none()),
            BoatAct::Lantern => continue,
        };
        if plaque.disabled != disabled {
            plaque.disabled = disabled;
        }
        if let Some(label) = label {
            for child in children.iter() {
                if let Ok(mut text) = labels.get_mut(child)
                    && text.text != label
                {
                    text.text = label.clone();
                }
            }
        }
    }
    let piece = if craft.lantern_out {
        "check-off"
    } else {
        "check-on"
    };
    for mut image in &mut checks {
        let want = kit.image(&Set::Stone.key(piece));
        if image.image != want {
            image.image = want;
        }
    }
}

/// A button pressed: the anchor goes to the vehicles (where the water's
/// depth is known), the rope and the lantern are the boat's own settings
/// and are saved with it at once.
pub fn press(
    buttons: Query<(&Interaction, &BoatAct, Option<&ui_kit::Plaque>), Changed<Interaction>>,
    menu: Res<BoatMenu>,
    mut vehicles: Query<(Entity, &mut Vehicle)>,
    mut order: ResMut<AnchorOrder>,
    mut fleet: ResMut<Fleet>,
) {
    let Some(entity) = menu.0 else {
        return;
    };
    for (interaction, act, plaque) in &buttons {
        if *interaction != Interaction::Pressed || plaque.is_some_and(|p| p.disabled) {
            continue;
        }
        let Ok((_, mut vehicle)) = vehicles.get_mut(entity) else {
            return;
        };
        let craft = &mut vehicle.craft;
        let plumb = || {
            craft
                .anchor
                .map(|a| a.plumb(craft.bow()).max(0.2) + 0.3)
                .unwrap_or(1.0)
        };
        match act {
            BoatAct::Anchor => {
                order.0 = Some(entity);
                continue;
            }
            BoatAct::Shorter => {
                let now = craft.rope_m.unwrap_or_else(plumb);
                let next = now - ROPE_STEP_M;
                let bed = plumb();
                craft.set_rope((next > bed).then_some(next.round()));
            }
            BoatAct::Longer => {
                let now = craft.rope_m.unwrap_or_else(plumb);
                craft.set_rope(Some((now + ROPE_STEP_M).round()));
            }
            BoatAct::ToTheBed => craft.set_rope(None),
            BoatAct::Lantern => craft.lantern_out = !craft.lantern_out,
        }
        fleet.dirty = true;
    }
}
