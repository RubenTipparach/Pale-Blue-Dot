//! Shut screens out of the UI's per-frame passes (`memory-leak`, towns at
//! 60 fps, cut 5).
//!
//! Bevy lays out, clips, stacks and hit-tests every UI node under every root
//! each frame, shown or not, and the game builds its big screens (settings,
//! the mission tree, the map, the stations) at startup and shuts them with
//! `Display::None`: about 6,500 of the UI's 8,500 nodes sit in shut screens.
//! A root shut for [`SHUT_FRAMES`] frames, with [`MIN_NODES`] nodes or more
//! under it, is made a child of one hidden entity that is not a UI node, so
//! it is no root and no UI pass reaches it. The game's systems still find
//! and fill it by its components. The frame anything sets its display back,
//! it is unparked before that frame's layout, so it opens when asked, as
//! before. Nothing drawn changes: a shut screen draws nothing either way.
//! `PBD_UI_PARK_OFF` leaves every root where it is, for an A/B.

use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};

/// Frames a root stays shut before it is parked: a panel that blinks shut
/// for a frame or two is never moved.
const SHUT_FRAMES: u32 = 30;

/// Nodes under a root (itself included) worth parking it for.
const MIN_NODES: usize = 64;

pub struct UiParkPlugin;

impl Plugin for UiParkPlugin {
    fn build(&self, app: &mut App) {
        if std::env::var_os("PBD_UI_PARK_OFF").is_some() {
            return;
        }
        app.add_systems(Startup, spawn_park).add_systems(
            PostUpdate,
            park_shut_screens.before(bevy::ui::UiSystems::Prepare),
        );
    }
}

/// The one entity shut screens are parked under: no `Node`, and hidden, so
/// what is under it is neither laid out nor drawn.
#[derive(Component)]
pub struct UiPark;

fn spawn_park(mut commands: Commands) {
    commands.spawn((UiPark, Visibility::Hidden));
}

#[allow(clippy::type_complexity)]
fn park_shut_screens(
    mut commands: Commands,
    park: Single<(Entity, Option<&Children>), With<UiPark>>,
    roots: Query<(Entity, &Node), Without<ChildOf>>,
    nodes: Query<&Node>,
    mut visibility: Query<&mut Visibility>,
    children: Query<&Children>,
    mut shut: Local<HashMap<Entity, u32>>,
    mut small: Local<HashSet<Entity>>,
) {
    let (park, parked) = *park;
    // Opened: back to a root before this frame's layout. Its visibility is
    // touched so it is propagated again from its own, not the park's.
    for &screen in parked.into_iter().flatten() {
        if nodes.get(screen).is_ok_and(|n| n.display != Display::None) {
            commands.entity(screen).remove::<ChildOf>();
            if let Ok(mut v) = visibility.get_mut(screen) {
                v.set_changed();
            }
        }
    }
    shut.retain(|e, _| roots.contains(*e));
    for (root, node) in &roots {
        if node.display != Display::None {
            shut.remove(&root);
            continue;
        }
        if small.contains(&root) {
            continue;
        }
        let frames = shut.entry(root).or_default();
        *frames += 1;
        if *frames < SHUT_FRAMES {
            continue;
        }
        shut.remove(&root);
        // Its size, counted once, up to what is worth it.
        let mut n = 0usize;
        let mut stack = vec![root];
        while let Some(e) = stack.pop() {
            n += 1;
            if n >= MIN_NODES {
                break;
            }
            if let Ok(c) = children.get(e) {
                stack.extend(c.iter());
            }
        }
        if n >= MIN_NODES {
            commands.entity(root).insert(ChildOf(park));
        } else {
            small.insert(root);
        }
    }
}
