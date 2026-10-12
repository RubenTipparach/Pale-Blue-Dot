//! The field guide: every species of the body's roster, its thumbnail, what
//! it is, the numbers the simulation fishes it by, and what this world has
//! caught of it.
//!
//! J opens it, on the fish in the selected hotbar slot when there is one and
//! on the page last read otherwise, and J or Escape closes it. It is a
//! `menu::Screen`, so it holds the pointer and the keyboard exactly as the
//! pause menu does and every input reader already stands down for it.
//!
//! Nothing on an entry is written twice: the text is the species record's
//! guide, the numbers are `fish::guide_facts` off the same record the schools
//! and the hook read, and the catches are the save's.

use super::menu::{self, EDGE, INK, MINT, MenuAction, PANEL_FILL, Panel, Screen};
use super::slots::ItemIcons;
use bevy::prelude::*;
use pbd_app::config::FaunaConfig;
use pbd_app::fish::{Body, guide_facts};
use pbd_app::hotbar::Hotbar;
use pbd_app::saves::WorldSave;
use pbd_core::fauna::Origin;
use pbd_core::inventory::Item;

/// Which entry the guide is on.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct GuidePage {
    pub species: u16,
}

/// The part of the page rebuilt when the entry changes.
#[derive(Component)]
pub struct GuideEntry;

/// A list row's catch count, by roster index.
#[derive(Component)]
pub struct GuideCount(pub u16);

const DIM: Color = Color::srgb(0.60, 0.70, 0.70);
const AMBER: Color = Color::srgb(0.96, 0.72, 0.30);

fn text(value: impl Into<String>, size: f32, colour: Color) -> impl Bundle {
    (
        Text::new(value.into()),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(colour),
    )
}

/// Build the guide's panel, hidden. `PostStartup`, after the icons.
pub fn spawn(
    mut commands: Commands,
    icons: Res<ItemIcons>,
    fauna: Res<FaunaConfig>,
    body: Res<Body>,
) {
    let roster = fauna.0.roster(&body.0);
    commands
        .spawn((menu::layer(), GlobalZIndex(10), Panel(Screen::Guide)))
        .with_children(|layer| {
            layer
                .spawn((
                    menu::panel_node(820.0),
                    BorderColor::all(EDGE),
                    BackgroundColor(PANEL_FILL),
                ))
                .with_children(|panel| {
                    panel.spawn(menu::title("FIELD GUIDE"));
                    panel
                        .spawn(Node {
                            column_gap: px(18),
                            ..default()
                        })
                        .with_children(|columns| {
                            columns
                                .spawn(Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(4),
                                    width: px(230),
                                    flex_shrink: 0.0,
                                    ..default()
                                })
                                .with_children(|list| {
                                    if roster.is_empty() {
                                        list.spawn(text(
                                            "Nothing lives in the water here.",
                                            12.0,
                                            DIM,
                                        ));
                                    }
                                    for (index, species) in roster.iter().enumerate() {
                                        row(list, index as u16, &species.name, &icons);
                                    }
                                });
                            columns.spawn((
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(8),
                                    flex_grow: 1.0,
                                    ..default()
                                },
                                GuideEntry,
                            ));
                        });
                    menu::button(panel, "CLOSE  (J)", MenuAction::Back);
                });
        });
}

/// One row of the species list: a pressable thumbnail, name and count.
fn row(list: &mut ChildSpawnerCommands, index: u16, name: &str, icons: &ItemIcons) {
    list.spawn((
        Button,
        Node {
            align_items: AlignItems::Center,
            column_gap: px(8),
            padding: UiRect::axes(px(8), px(4)),
            border: UiRect::all(px(1)),
            ..default()
        },
        BorderColor::all(EDGE),
        BackgroundColor(Color::srgba(0.06, 0.13, 0.15, 0.9)),
        MenuAction::Species(index),
    ))
    .with_children(|row| {
        if let Some(icon) = icons.fish.get(index as usize) {
            row.spawn((
                ImageNode::new(icon.clone()),
                Node {
                    width: px(28),
                    height: px(28),
                    ..default()
                },
            ));
        }
        row.spawn((
            text(name, 13.0, INK),
            Node {
                flex_grow: 1.0,
                ..default()
            },
        ));
        row.spawn((text("", 11.0, DIM), GuideCount(index)));
    });
}

/// J opens the guide from the world and closes it again. `PreUpdate` after
/// `menu::toggle`, before the world's readers, like every key a menu owns.
pub fn open(
    keys: Res<ButtonInput<KeyCode>>,
    mut screen: ResMut<Screen>,
    mut page: ResMut<GuidePage>,
    hotbar: Option<Res<Hotbar>>,
) {
    if !keys.just_pressed(KeyCode::KeyJ) {
        return;
    }
    match *screen {
        Screen::Playing => {
            if let Some(Item::Fish(species)) = hotbar.and_then(|h| h.held()).map(|s| s.item) {
                page.species = species;
            }
            *screen = Screen::Guide;
        }
        Screen::Guide => *screen = Screen::Playing,
        // The other pages hold the keyboard for themselves (the name field
        // types a J), so J means nothing there.
        _ => {}
    }
}

/// A species row pressed: turn to its entry.
pub fn press(
    rows: Query<(&Interaction, &MenuAction), Changed<Interaction>>,
    mut page: ResMut<GuidePage>,
) {
    for (interaction, action) in &rows {
        if let (Interaction::Pressed, MenuAction::Species(index)) = (interaction, action) {
            page.species = *index;
        }
    }
}

/// Fill the open page: the counts down the list and the entry beside it.
#[allow(clippy::too_many_arguments)]
pub fn paint(
    mut commands: Commands,
    screen: Res<Screen>,
    page: Res<GuidePage>,
    fauna: Res<FaunaConfig>,
    body: Res<Body>,
    icons: Res<ItemIcons>,
    save: Option<Res<WorldSave>>,
    entry: Query<Entity, With<GuideEntry>>,
    mut counts: Query<(&GuideCount, &mut Text)>,
) {
    if *screen != Screen::Guide {
        return;
    }
    let saved = save.as_ref().is_some_and(|s| s.is_changed());
    if !screen.is_changed() && !page.is_changed() && !saved {
        return;
    }
    let catches = save.as_ref().map(|s| &s.catches);
    for (count, mut label) in &mut counts {
        let n = catches
            .and_then(|c| c.get(&count.0))
            .map_or(0, |record| record.count);
        label.0 = if n == 0 { "-".into() } else { format!("x{n}") };
    }
    let roster = fauna.0.roster(&body.0);
    let Some(species) = roster.get(page.species as usize) else {
        return;
    };
    let Ok(entry) = entry.single() else {
        return;
    };
    let record = catches.and_then(|c| c.get(&page.species)).copied();
    let facts = guide_facts(species, &fauna.0.fishing);
    commands.entity(entry).despawn_children();
    commands.entity(entry).with_children(|entry| {
        entry
            .spawn(Node {
                column_gap: px(14),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|head| {
                if let Some(icon) = icons.fish.get(page.species as usize) {
                    head.spawn((
                        ImageNode::new(icon.clone()),
                        Node {
                            width: px(96),
                            height: px(96),
                            flex_shrink: 0.0,
                            ..default()
                        },
                    ));
                }
                head.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(4),
                    ..default()
                })
                .with_children(|words| {
                    words.spawn(text(species.name.to_uppercase(), 18.0, MINT));
                    let from = match &species.from {
                        Origin::Tenebris(kind) => format!("from Tenebris ({kind})"),
                        Origin::New => "new to Pale Blue Dot".into(),
                    };
                    words.spawn(text(from, 11.0, AMBER));
                    let caught = match record {
                        Some(r) if r.count > 0 => {
                            format!("caught {}, best {} cm", r.count, r.best_cm)
                        }
                        _ => "not caught in this world yet".into(),
                    };
                    words.spawn(text(caught, 12.0, INK));
                });
            });
        entry.spawn((
            text(species.guide.entry.clone(), 13.0, INK),
            Node {
                max_width: px(520),
                ..default()
            },
        ));
        entry
            .spawn(Node {
                display: Display::Grid,
                grid_template_columns: vec![GridTrack::px(120.0), GridTrack::flex(1.0)],
                row_gap: px(3),
                column_gap: px(10),
                ..default()
            })
            .with_children(|grid| {
                for (key, value) in facts {
                    grid.spawn(text(key, 10.0, DIM));
                    grid.spawn(text(value, 12.0, INK));
                }
            });
        entry.spawn((
            text(format!("TIP  {}", species.guide.tip), 12.0, AMBER),
            Node {
                max_width: px(520),
                ..default()
            },
        ));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbd_core::inventory::SLOTS;

    /// J from the world opens the guide on the fish in the selected slot, and
    /// J again closes it.
    #[test]
    fn j_opens_the_guide_on_the_fish_in_hand() {
        let mut hotbar = Hotbar::default();
        assert_eq!(hotbar.give(Item::Fish(3), 1), 0);
        let slot = (0..SLOTS)
            .find(|i| hotbar.get(*i).is_some_and(|s| s.item == Item::Fish(3)))
            .expect("the fish went in");
        hotbar.select(slot);
        let mut app = App::new();
        app.init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(Screen::Playing)
            .init_resource::<GuidePage>()
            .insert_resource(hotbar)
            .add_systems(Update, open);
        let tap_j = |app: &mut App| {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.clear();
            keys.release(KeyCode::KeyJ);
            keys.clear();
            keys.press(KeyCode::KeyJ);
            app.update();
        };
        tap_j(&mut app);
        assert_eq!(*app.world().resource::<Screen>(), Screen::Guide);
        assert_eq!(app.world().resource::<GuidePage>().species, 3);
        tap_j(&mut app);
        assert_eq!(*app.world().resource::<Screen>(), Screen::Playing);
    }
}
