//! Capture instruments for `survival-mode` task 4.10 (measurement only:
//! nothing a player meets). `--survival-kit` puts one of each survival good
//! and both new stations in the pack, as a player who had made them would
//! carry them, so a `--menu pack` capture shows their icons. `--lay-weapons
//! <m>` lays one of each weapon as a drop in a row that far ahead of the
//! eye, as a dropped or thrown one lies. A capture's save is memory-only.

use super::slots::Hotbar;
use bevy::prelude::*;
use pbd_app::drops::Drops;
use pbd_core::drops::ItemDrop;
use pbd_core::goods::Good;
use pbd_core::inventory::Item;
use pbd_core::terrain::Material;

/// The goods `--survival-kit` gives, in the pack's order.
pub const KIT: [&str; 20] = [
    "raw-meat",
    "hide",
    "bone",
    "feather",
    "straw",
    "stone-spear",
    "bronze-spear",
    "iron-spear",
    "bow",
    "stone-arrow",
    "bone-arrow",
    "bronze-arrow",
    "bronze-hunting-axe",
    "iron-hunting-axe",
    "bronze-sword",
    "iron-sword",
    "straw-hat",
    "wool-cloak",
    "bedroll",
    "meat-stew",
];

/// `--survival-kit`: give the kit once.
pub fn give_kit(launch: Res<super::Launch>, hotbar: Option<ResMut<Hotbar>>, mut done: Local<bool>) {
    let Some(mut hotbar) = hotbar else { return };
    if !launch.survival_kit || *done {
        return;
    }
    *done = true;
    let mut slots = hotbar.0.clone();
    for code in KIT {
        let Some(good) = Good::by_code(code) else {
            warn!("--survival-kit: no good {code}");
            continue;
        };
        let n = good.stack_limit().min(4);
        slots.give(Item::Good(good), n);
    }
    for m in [Material::TanningRack, Material::Still] {
        slots.give(Item::Block(m), 1);
    }
    hotbar.0 = slots;
    info!("SURVIVAL_KIT given");
}

/// `--lay-weapons <m>`: lay each weapon as a drop, in a row across the
/// view that far ahead, once the eye has found the ground.
pub fn lay_weapons(
    launch: Res<super::Launch>,
    drops: Option<ResMut<Drops>>,
    origin: Option<Res<pbd_app::sim_origin::SimFrame>>,
    walker: Query<&Transform, With<pbd_app::walking::Walker>>,
    cameras: Query<&GlobalTransform, With<pbd_app::walking::WalkingCamera>>,
    time: Option<Res<pbd_app::sky::Sun>>,
    mut frames: Local<u32>,
) {
    let (Some(ahead_m), Some(mut drops), Some(origin)) = (launch.lay_weapons, drops, origin) else {
        return;
    };
    *frames += 1;
    if *frames != 90 {
        return;
    }
    let (Ok(feet), Some(eye)) = (walker.single(), cameras.iter().next()) else {
        return;
    };
    let origin = origin.0;
    let up = origin.up_at(feet.translation);
    let look = eye.forward().as_vec3();
    let ahead = (look - up * look.dot(up)).normalize_or(up.any_orthonormal_vector());
    let right = ahead.cross(up);
    let now = time.map_or(0.0, |s| s.clock.seconds);
    let base = feet.translation + ahead * ahead_m;
    let weapons = pbd_app::weapon_models::WEAPONS;
    for (k, good) in weapons.iter().take(11).enumerate() {
        let across = (k as f32 - 5.0) * 0.6;
        let at = base + right * across + ahead * ((k % 2) as f32 * 0.6)
            - up * (pbd_app::walking::HALF_HEIGHT - 0.3);
        let world = origin.to_world(at);
        drops.live.push(ItemDrop {
            id: 9_000_000 + k as u64,
            item: Item::Good(*good),
            count: 1,
            position: world.pos,
            made_s: now,
        });
    }
    info!("LAY_WEAPONS {} ahead {ahead_m} m", weapons.len().min(11));
}
