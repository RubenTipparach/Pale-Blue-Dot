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

use bevy::prelude::*;

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
}
