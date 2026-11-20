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

use super::menu::{self, MenuAction, Panel, Screen};
use super::slots::ItemIcons;
use bevy::prelude::*;
use pbd_app::config::FaunaConfig;
use pbd_app::fish::{Body, guide_facts};
use pbd_app::hotbar::Hotbar;
use pbd_app::saves::WorldSave;
use pbd_app::ui_kit::Localized;
use pbd_app::ui_kit::{self, KitText, Set, UiKit, plaque};
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

const DIM: Color = ui_kit::TEXT_DIM;
const AMBER: Color = ui_kit::TITLE;
const INK: Color = ui_kit::TEXT;

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
    kit: Res<UiKit>,
    icons: Res<ItemIcons>,
    fauna: Res<FaunaConfig>,
    body: Res<Body>,
) {
    let roster = fauna.0.roster(&body.0);
    commands
        .spawn((menu::layer(), GlobalZIndex(10), Panel(Screen::Guide)))
        .with_children(|layer| {
            layer
                // In play, so stone: the dressed-stone moulding, which
                // stretches with nothing to repeat (no rock is baked at the
                // guide's size).
                .spawn(menu::framed(&kit, Set::Stone, 420.0))
                .with_children(|panel| {
                    menu::lintel(panel, &kit, "FIELD GUIDE");
                    panel
                        .spawn(Node {
                            column_gap: px(9),
                            ..default()
                        })
                        .with_children(|columns| {
                            columns
                                .spawn(Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(1),
                                    width: px(120),
                                    flex_shrink: 0.0,
                                    ..default()
                                })
                                .with_children(|list| {
                                    if roster.is_empty() {
                                        list.spawn(text(
                                            "Nothing lives in the water here.",
                                            10.0,
                                            DIM,
                                        ));
                                    }
                                    for (index, species) in roster.iter().enumerate() {
                                        row(list, &kit, index as u16, &species.name, &icons);
                                    }
                                });
                            columns.spawn((
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: px(4),
                                    flex_grow: 1.0,
                                    ..default()
                                },
                                GuideEntry,
                            ));
                        });
                });
        });
}

/// One row of the species list: a pressable thumbnail, name and count.
fn row(list: &mut ChildSpawnerCommands, kit: &UiKit, index: u16, name: &str, icons: &ItemIcons) {
    list.spawn((plaque(kit, Set::Stone, 48.0), MenuAction::Species(index)))
        .insert(Node {
            align_items: AlignItems::Center,
            column_gap: px(4),
            height: px(22),
            padding: UiRect::axes(px(6), px(3)),
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|row| {
            if let Some(icon) = icons.fish.get(index as usize) {
                row.spawn((
                    ImageNode::new(icon.clone()),
                    Node {
                        width: px(16),
                        height: px(16),
                        ..default()
                    },
                ));
            }
            row.spawn((
                text(name, 10.0, ui_kit::LABEL),
                Node {
                    flex_grow: 1.0,
                    ..default()
                },
            ));
            row.spawn((text("", 9.0, ui_kit::TAB_DIM), GuideCount(index)));
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
                column_gap: px(7),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|head| {
                if let Some(icon) = icons.fish.get(page.species as usize) {
                    head.spawn((
                        ImageNode::new(icon.clone()),
                        Node {
                            width: px(48),
                            height: px(48),
                            flex_shrink: 0.0,
                            ..default()
                        },
                    ));
                }
                head.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(2),
                    ..default()
                })
                .with_children(|words| {
                    words.spawn(
                        KitText::new(
                            Localized::legacy(species.name.to_uppercase()),
                            ui_kit::TITLE,
                        )
                        .shadow(ui_kit::INK)
                        .size(2),
                    );
                    let from = match &species.from {
                        Origin::Tenebris(kind) => format!("from Tenebris ({kind})"),
                        Origin::New => "new to Pale Blue Dot".into(),
                    };
                    words.spawn(text(from, 9.0, AMBER));
                    let caught = match record {
                        Some(r) if r.count > 0 => {
                            format!("caught {}, best {} cm", r.count, r.best_cm)
                        }
                        _ => "not caught in this world yet".into(),
                    };
                    words.spawn(text(caught, 10.0, INK));
                });
            });
        entry.spawn((
            text(species.guide.entry.clone(), 10.0, INK),
            Node {
                max_width: px(260),
                ..default()
            },
        ));
        entry
            .spawn(Node {
                display: Display::Grid,
                grid_template_columns: vec![GridTrack::px(60.0), GridTrack::flex(1.0)],
                row_gap: px(1),
                column_gap: px(5),
                ..default()
            })
            .with_children(|grid| {
                for (key, value) in facts {
                    grid.spawn(text(key, 9.0, DIM));
                    grid.spawn(text(value, 10.0, INK));
                }
            });
        entry.spawn((
            text(format!("TIP  {}", species.guide.tip), 10.0, AMBER),
            Node {
                max_width: px(260),
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
