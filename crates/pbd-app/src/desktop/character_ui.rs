//! The Character page of the Options menu (`player-character` design 7):
//! a row per part and colour of the player's look, each stepped with the
//! kit's `<` and `>`, and RANDOM. Every change is kept in `character.ron`
//! and sent to the host at once (`net_play::send_look` reads the resource).
//!
//! The preview is the player's own figure, drawn by the townsfolk's code
//! in the world in front of the camera, so it is lit as the ground there is.

use super::menu::{MenuAction, Screen, SettingsTab};
use bevy::prelude::*;
use pbd_app::character::PlayerLook;
use pbd_app::ui_kit::{self, KitPiece, KitText, Plaque, Set, UiKit, plaque_label};
use pbd_core::locale::{Localized, tr, tr_with};
use pbd_core::townsfolk::character::{self, Field};
use pbd_core::townsfolk::look::Look;

/// The settings tab this page is.
pub const TAB: &str = "character";

/// The visitor number the preview stands as: no host gives it to a player.
pub const PREVIEW: u32 = u32::MAX - 1;

/// How far in front of the camera the preview stands, m, and how far to the
/// right of the panel.
const AHEAD_M: f64 = 4.5;
const RIGHT_M: f64 = 3.4;

/// A row's value as words (a part, a face, the height), by field index.
#[derive(Component)]
pub struct LookValue(pub u8);

/// A row's value as its swatch (a colour), by field index.
#[derive(Component)]
pub struct LookSwatch(pub u8);

/// A field's label.
fn label(field: Field) -> Localized {
    match field {
        Field::Build => tr("character.build"),
        Field::Height => tr("character.height"),
        Field::Skin => tr("character.skin"),
        Field::Hair => tr("character.hair"),
        Field::HairColour => tr("character.hair_colour"),
        Field::Beard => tr("character.beard"),
        Field::Face => tr("character.face"),
        Field::Nose => tr("character.nose"),
        Field::Ears => tr("character.ears"),
        Field::Shirt => tr("character.shirt"),
        Field::Trousers => tr("character.trousers"),
        Field::Shoes => tr("character.shoes"),
        Field::HatColour => tr("character.hat_colour"),
    }
}

/// A part's or a face's name, by its id.
fn value(id: &str) -> Localized {
    match id {
        "round" => tr("character.value.round"),
        "pear" => tr("character.value.pear"),
        "tall" => tr("character.value.tall"),
        "stout" => tr("character.value.stout"),
        "bun" => tr("character.value.bun"),
        "crop" => tr("character.value.crop"),
        "braid" => tr("character.value.braid"),
        "bald" => tr("character.value.bald"),
        "hat" => tr("character.value.hat"),
        "goatee" => tr("character.value.goatee"),
        "full" => tr("character.value.full"),
        "button" => tr("character.value.button"),
        "long" => tr("character.value.long"),
        "happy" => tr("character.value.happy"),
        "neutral" => tr("character.value.neutral"),
        "thinking" => tr("character.value.thinking"),
        "sleepy" => tr("character.value.sleepy"),
        "yes" => tr("character.value.yes"),
        "no" => tr("character.value.no"),
        _ => tr("character.value.none"),
    }
}

/// What a words row shows for `look`.
fn shown(look: &Look, field: Field) -> Localized {
    match character::value_id(look, field) {
        Some(id) => value(id),
        None => tr_with(
            "character.metres",
            &[("metres", &format!("{:.2}", look.height_m))],
        ),
    }
}

fn swatch(colour: u32) -> Color {
    Color::srgb_u8((colour >> 16) as u8, (colour >> 8) as u8, colour as u8)
}

/// A `<` or `>` plaque: a step of one row.
fn stepper(parent: &mut ChildSpawnerCommands, kit: &UiKit, text: Localized, action: MenuAction) {
    // The kit's plaque, narrower than a row's: 6 px a side.
    let piece = (
        Button,
        kit.nine(Set::Oak, "button-idle"),
        KitPiece(Set::Oak.key("button-idle")),
        Plaque::new(Set::Oak),
        Node {
            min_width: px(18),
            height: px(22),
            padding: UiRect::new(px(6), px(6), px(8), px(0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::FlexStart,
            flex_shrink: 0.0,
            ..default()
        },
    );
    parent.spawn((piece, action)).with_children(|p| {
        p.spawn(plaque_label(Set::Oak, text));
    });
}

fn row(rows: &mut ChildSpawnerCommands, kit: &UiKit, i: usize) {
    let field = Field::ALL[i];
    rows.spawn(Node {
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        min_height: px(23),
        column_gap: px(3),
        ..default()
    })
    .with_children(|row| {
        row.spawn(super::menu::body(label(field), 10.0, ui_kit::TEXT));
        row.spawn(Node {
            align_items: AlignItems::Center,
            column_gap: px(3),
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|control| {
            stepper(
                control,
                kit,
                tr("character.previous"),
                MenuAction::Look(i as u8, -1),
            );
            if field.is_colour() {
                control
                    .spawn(Node {
                        width: px(50),
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_children(|well| {
                        well.spawn((
                            Node {
                                width: px(24),
                                height: px(11),
                                border: UiRect::all(px(1)),
                                ..default()
                            },
                            BorderColor::all(ui_kit::INK),
                            BackgroundColor(Color::NONE),
                            LookSwatch(i as u8),
                        ));
                    });
            } else {
                control.spawn((
                    KitText::new(Localized::empty(), ui_kit::LABEL).shadow(ui_kit::INK),
                    Node {
                        width: px(50),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    LookValue(i as u8),
                ));
            }
            stepper(
                control,
                kit,
                tr("character.next"),
                MenuAction::Look(i as u8, 1),
            );
        });
    });
}

/// The page, laid into its settings tab: two columns of rows, then RANDOM.
pub fn spawn(page: &mut ChildSpawnerCommands, kit: &UiKit) {
    page.spawn(Node {
        column_gap: px(10),
        ..default()
    })
    .with_children(|cols| {
        for half in [0..7, 7..Field::ALL.len()] {
            cols.spawn(Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(1),
                width: px(186),
                ..default()
            })
            .with_children(|rows| {
                for i in half {
                    row(rows, kit, i);
                }
            });
        }
    });
    page.spawn(Node {
        margin: UiRect::top(px(3)),
        ..default()
    })
    .with_children(|foot| {
        super::menu::small(foot, kit, tr("character.random"), MenuAction::LookRandom);
    });
}

/// A step or RANDOM changes the look, keeps it and shows it.
pub fn press(
    rows: Query<(&Interaction, &MenuAction), Changed<Interaction>>,
    mut look: ResMut<PlayerLook>,
) {
    for (interaction, action) in &rows {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let next = match *action {
            MenuAction::Look(i, by) => match Field::ALL.get(usize::from(i)) {
                Some(field) => character::step(&look.0, *field, i32::from(by)),
                None => continue,
            },
            MenuAction::LookRandom => {
                let mut key = [0u8; 16];
                let seed = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(1, |d| d.as_nanos());
                key.copy_from_slice(&seed.to_le_bytes());
                character::rolled(&key)
            }
            _ => continue,
        };
        if next != look.0 {
            if !PlayerLook::save(&next) {
                warn!("character: the look could not be kept");
            }
            look.0 = next;
        }
    }
}

/// Every row shows the look as it is.
pub fn show(
    look: Res<PlayerLook>,
    mut words: Query<(&LookValue, &mut KitText)>,
    mut swatches: Query<(&LookSwatch, &mut BackgroundColor)>,
    added: Query<(), Added<LookValue>>,
) {
    if !look.is_changed() && added.is_empty() {
        return;
    }
    for (row, mut text) in &mut words {
        if let Some(field) = Field::ALL.get(usize::from(row.0)) {
            let want = shown(&look.0, *field);
            if text.text != want {
                text.text = want;
            }
        }
    }
    for (row, mut fill) in &mut swatches {
        if let Some(c) = Field::ALL
            .get(usize::from(row.0))
            .and_then(|f| character::colour(&look.0, *f))
        {
            fill.0 = swatch(c);
        }
    }
}

/// While the page is open, the player's own figure stands in front of the
/// camera, to the right of the panel, facing it.
#[allow(clippy::too_many_arguments)]
pub fn preview(
    screen: Res<Screen>,
    tab: Res<SettingsTab>,
    look: Res<PlayerLook>,
    folk: Option<ResMut<pbd_app::townsfolk::Townsfolk>>,
    space: pbd_app::SimSpace,
    walkers: Query<&Transform, With<pbd_app::walking::Walker>>,
    cameras: Query<&GlobalTransform, With<pbd_app::walking::WalkingCamera>>,
    mut standing: Local<bool>,
) {
    let Some(mut folk) = folk else { return };
    let open = *screen == Screen::Settings
        && super::menu::SETTINGS_TABS.get(usize::from(tab.0)) == Some(&TAB);
    let eye = walkers.iter().next().zip(cameras.iter().next());
    let Some((walker, camera)) = eye.filter(|_| open) else {
        if std::mem::take(&mut *standing) {
            folk.unvisit(PREVIEW);
        }
        return;
    };
    let pose = pbd_app::net_play::own_pose(&space.origin(), walker.translation, *camera.forward());
    let feet = pose.at.pos;
    let up = feet.normalize();
    let forward = pbd_core::net::protocol::forward_of(pose.yaw, up);
    let right = forward.cross(up);
    let at = (feet + forward * AHEAD_M + right * RIGHT_M).normalize() * feet.length();
    let facing = {
        let to = feet - at;
        let flat = to - up * to.dot(up);
        flat.normalize_or(-forward)
    };
    folk.visit(PREVIEW, at, facing, false, Some(look.0), look.0.shirt);
    *standing = true;
}

/// Offered once (design 7): the first time a world is entered from the
/// worlds page with no look kept, the page opens before play. The rolled
/// look is kept at once, so closing the page keeps it and it is not offered
/// again.
pub fn offer(
    mut screen: ResMut<Screen>,
    mut tab: ResMut<SettingsTab>,
    look: Res<PlayerLook>,
    mut last: Local<Option<Screen>>,
) {
    let before = last.replace(*screen);
    if before != Some(Screen::Saves) || *screen != Screen::Playing || PlayerLook::kept() {
        return;
    }
    if !PlayerLook::save(&look.0) {
        return;
    }
    if let Some(i) = super::menu::SETTINGS_TABS.iter().position(|t| *t == TAB) {
        tab.0 = i as u8;
        *screen = Screen::Settings;
    }
}
