//! What is on screen while the player is in the world: the crosshair, and
//! nothing else.
//!
//! It used to be four blocks of floating text on two full-width scrims - a
//! mode and frame-rate line, a speed and altitude block, a one-line binding
//! strip, and the full binding wall behind `H` - which together dimmed 27% of
//! the window to make a readout legible. None of it was a control. The owner's
//! instruction was that all of it is excess, and the scrims went with it: they
//! were only ever there to keep small text readable over snow and sea glint,
//! so deleting the text deletes their reason to exist.
//!
//! The crosshair stays because it is a CONTROL: it is where a dig lands. The
//! slot row stays for the same reason and lives in [`super::slots`]. The
//! bindings live in the settings page, off one table, in [`super::menu`].
//!
//! One line of text came back, at the owner's request, as an INSTRUMENT: the
//! level drawing the ground underfoot, whether that cell has a column, how
//! far the resident set's anchor is and whether a rebuild is in flight. It is
//! what says "you have outrun the streaming" while the near-field-streaming
//! change is built, and it goes when that change lands.

use bevy::prelude::*;
use pbd_app::planet::NearField;

/// The near-field readout's text node.
#[derive(Component)]
pub struct NearFieldText;

pub fn setup(mut commands: Commands) {
    // Three pixels, pale, in the middle. Small enough to disappear while
    // walking and exact enough to aim a block with, which is the whole job.
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: percent(50),
            width: px(3),
            height: px(3),
            ..default()
        },
        BackgroundColor(Color::srgba(0.88, 0.94, 0.91, 0.75)),
    ));
    commands.spawn((
        Text::new(NearField::default().line()),
        TextFont {
            font_size: 11.0,
            ..default()
        },
        TextColor(Color::srgb(0.92, 0.97, 0.95)),
        TextShadow::default(),
        Node {
            position_type: PositionType::Absolute,
            left: px(8),
            top: px(6),
            ..default()
        },
        NearFieldText,
    ));
}

/// Keep the readout current; the resource changes only when a number does.
pub fn near_field(near: Res<NearField>, mut text: Query<&mut Text, With<NearFieldText>>) {
    if !near.is_changed() {
        return;
    }
    for mut text in &mut text {
        text.0 = near.line();
    }
}
