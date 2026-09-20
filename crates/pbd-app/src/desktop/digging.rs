//! Digging a block out and putting one back.
//!
//! The rules are the core's: [`pbd_core::aim::march`] finds what the eye ray
//! meets, [`pbd_core::edits::Edits`] is what the player changed, and
//! [`pbd_core::column::generate_edited`] is how a column carries it. What is
//! here is the three things the core cannot know - which cell a world point is
//! in, how to put the changed column back on the GPU, and where the save goes.
//!
//! The order of a frame's work is the one the rules demand: accept the edit,
//! write it durably, THEN show it. A block that vanished on screen and never
//! reached the disk is the failure `CLAUDE.md` names.

use bevy::prelude::*;
use pbd_app::planet::PLANET_RADIUS;
use pbd_app::planet::{PlanetContact, PlanetFine};
use pbd_app::saves::WorldSave;
use pbd_core::aim::{self, Sample};
use pbd_core::column::{self, LAYERS};
use pbd_core::edits::Edit;
use pbd_core::inventory::Item;
use pbd_core::terrain::Material;
use std::sync::Arc;

/// What the player's ray met, kept for the HUD and for the click that follows.
#[derive(Resource, Default)]
pub struct Aim {
    pub target: Option<aim::Target>,
}

/// Which cell and layer a world point is in, and whether it is solid.
///
/// The walk is `PlanetContact`'s, which steps cell to cell across whichever
/// edge the ray falls outside of, so a march's samples cost a step from the
/// last rather than a search.
fn sample_at(fine: &PlanetFine, contact: &PlanetContact, point: Vec3) -> Option<Sample> {
    let direction = point.try_normalize()?;
    let record = contact.finest_cell(direction)?;
    let altitude = point.length() - PLANET_RADIUS;
    let layer = column::layer_at(altitude)?;
    let tier = &fine.set.columns;
    let slot = *tier.slots.get(record)?;
    let column = tier.columns.get(slot)?;
    Some(Sample {
        cell: fine.set.finest_records()[record].metadata[3],
        layer,
        solid: column.solid(layer),
    })
}

/// The record index of a cell by its stable ID. The march answers in stable
/// IDs because that is what an edit is keyed by; applying one needs the slot.
fn record_of(fine: &PlanetFine, cell: u32) -> Option<usize> {
    fine.set
        .finest_records()
        .iter()
        .position(|record| record.metadata[3] == cell)
}

/// What the player's hands do as part of an edit.
///
/// The hands are IN the transaction rather than beside it, and that is the
/// whole reason this enum exists. The log line carries the hotbar after the
/// edit, so the move has to be settled before the record is written: a dig
/// recorded without the block it yielded is a world where the hole is durable
/// and the block was never picked up.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Hands {
    /// A dig: take what was there.
    Take,
    /// A place: spend one of the selected stack.
    Spend,
    /// A scripted edit, which has no player and no hands.
    Empty,
}

/// Everything one edit touches, which is what says these four travel
/// together: the geometry, what the walker stands on, the save, and the hands.
/// A shorter argument list is the symptom; the reason is that an edit is a
/// transaction over exactly these.
pub struct Edited<'a> {
    pub fine: &'a mut PlanetFine,
    pub contact: &'a mut PlanetContact,
    pub save: &'a mut WorldSave,
    pub slots: &'a mut super::slots::Hotbar,
}

/// Accept a dig or a place, and put the world back together.
///
/// Returns the material taken, if any. Everything it touches is what the
/// design named: the column, its own record, its neighbours' records (their
/// flanks are clipped against this column's air), the contact the walker
/// stands on, and the hotbar - which moves only if the save took the record,
/// so the world, the player and the disk agree or none of them moves.
pub fn apply_edit(
    world: &mut Edited,
    hands: Hands,
    cell: u32,
    layer: usize,
    material: Material,
) -> Option<Material> {
    let Edited {
        fine,
        contact,
        save,
        slots,
    } = world;
    if layer == 0 || layer >= LAYERS {
        return None;
    }
    let record = record_of(fine, cell)?;
    let was = {
        let tier = &fine.set.columns;
        let slot = *tier.slots.get(record)?;
        tier.columns.get(slot)?.material(layer)
    };
    if was == material {
        return None;
    }
    // The move is made on a COPY first. What the log records is the hotbar
    // after the edit, and what the player keeps is that same hotbar only if
    // the record was taken.
    let mut moved = slots.0.clone();
    match hands {
        Hands::Take => {
            moved.give(Item::Block(was), 1);
        }
        Hands::Spend => {
            let index = moved.selected();
            if moved.take(index, 1) != 1 {
                return None;
            }
        }
        Hands::Empty => {}
    }
    if !save.accept(
        Edit {
            cell,
            layer: layer as u16,
            material,
        },
        &moved,
    ) {
        return None;
    }
    slots.0 = moved;
    let mut set = (*fine.set).clone();
    set.columns.set_layer(record, layer, material);
    set.columns.repack(record);
    for &neighbor in set.finest_neighbors[record].iter() {
        if neighbor != u32::MAX {
            set.columns.repack(neighbor as usize);
        }
    }
    let set = Arc::new(set);
    contact.set_fine(&set);
    fine.set = set;
    fine.version += 1;
    Some(was)
}

/// The eye ray, the click, and the two verbs.
#[allow(clippy::too_many_arguments)]
pub fn dig_and_place(
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut fine: ResMut<PlanetFine>,
    mut contact: ResMut<PlanetContact>,
    mut edits: ResMut<WorldSave>,
    mut slots: ResMut<super::slots::Hotbar>,
    mut aimed: ResMut<Aim>,
    walking: Option<Res<pbd_app::walking::WalkingReadout>>,
) {
    let Some((transform, _)) = cameras.iter().find(|(_, camera)| camera.is_active) else {
        return;
    };
    // Only on foot, and only while the walker has the pointer. A ship's guns
    // are not a shovel, and a click that is really "give me the mouse back"
    // is not a dig: Bevy's UI does not consume the raw button, so without the
    // capture test a click on a menu row swings the shovel through the button
    // at whatever is behind it.
    if !walking.is_some_and(|readout| readout.active && readout.captured) {
        aimed.target = None;
        return;
    }
    let eye = transform.translation();
    let look = transform.forward().as_vec3();
    let target = aim::march(eye, look, |point| sample_at(&fine, &contact, point));
    aimed.target = target;
    let Some(target) = target else {
        return;
    };

    if buttons.just_pressed(MouseButton::Left) {
        if let Some(taken) = apply_edit(
            &mut Edited {
                fine: &mut fine,
                contact: &mut contact,
                save: &mut edits,
                slots: &mut slots,
            },
            Hands::Take,
            target.dig.cell,
            target.dig.layer,
            Material::Air,
        ) {
            info!(
                "dug {taken:?} from cell {} layer {}",
                target.dig.cell, target.dig.layer
            );
        }
        return;
    }

    if buttons.just_pressed(MouseButton::Right) {
        let Some(place) = target.place else {
            return;
        };
        let Some(Item::Block(material)) = slots.held().map(|stack| stack.item) else {
            return;
        };
        // A block may not be placed into the player. Sealing yourself into
        // rock is a world you cannot move in, and the walker's own feet and
        // eye are what say where you are.
        if occupies(eye, place) {
            return;
        }
        if apply_edit(
            &mut Edited {
                fine: &mut fine,
                contact: &mut contact,
                save: &mut edits,
                slots: &mut slots,
            },
            Hands::Spend,
            place.cell,
            place.layer,
            material,
        )
        .is_some()
        {
            info!(
                "placed {material:?} in cell {} layer {}",
                place.cell, place.layer
            );
        }
    }
}

/// Whether the player's own body stands in this cell and layer.
///
/// The eye is the camera; the body is the two layers under it, which is the
/// walker's capsule rounded to the lattice. A block placed in either is a
/// block placed in the player.
fn occupies(eye: Vec3, place: Sample) -> bool {
    let altitude = eye.length() - PLANET_RADIUS;
    let Some(head) = column::layer_at(altitude) else {
        return false;
    };
    let feet = head.saturating_sub(1);
    (feet..=head).contains(&place.layer)
}

/// The scripted dig: a headless run has no mouse, and a picture of a hole is
/// the only thing that says the verb works end to end.
///
/// It digs straight DOWN from the camera rather than along its look, because a
/// capture's camera is aimed at whatever its view frames and the one thing
/// certainly in reach of a standing player is the ground under them. Then
/// `--place` puts one block back on top of the hole, so a frame shows both
/// verbs: a pit, and a block standing in it.
pub fn scripted_dig(
    launch: Res<super::Launch>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    mut fine: ResMut<PlanetFine>,
    mut contact: ResMut<PlanetContact>,
    mut edits: ResMut<WorldSave>,
    mut slots: ResMut<super::slots::Hotbar>,
    mut done: Local<bool>,
) {
    if *done || launch.dig == 0 || launch.capture.is_none() {
        return;
    }
    let Some((transform, _)) = cameras.iter().find(|(_, camera)| camera.is_active) else {
        return;
    };
    let eye = transform.translation();
    let down = -eye.normalize_or(Vec3::Y);
    let mut dug = 0;
    let mut last: Option<Sample> = None;
    for _ in 0..launch.dig {
        let Some(target) = aim::march(eye, down, |point| sample_at(&fine, &contact, point)) else {
            break;
        };
        if apply_edit(
            &mut Edited {
                fine: &mut fine,
                contact: &mut contact,
                save: &mut edits,
                slots: &mut slots,
            },
            Hands::Take,
            target.dig.cell,
            target.dig.layer,
            Material::Air,
        )
        .is_none()
        {
            break;
        }
        last = Some(target.dig);
        dug += 1;
    }
    if let Some(bottom) = last.filter(|_| launch.place) {
        apply_edit(
            &mut Edited {
                fine: &mut fine,
                contact: &mut contact,
                save: &mut edits,
                slots: &mut slots,
            },
            // The harness has no player: the block it puts back comes from
            // nowhere, as it did before the hands were part of the edit.
            Hands::Empty,
            bottom.cell,
            bottom.layer + 1,
            Material::Stone,
        );
    }
    if dug == 0 {
        // Say why nothing happened rather than failing silently: a scripted
        // dig that finds no ground is either out of the tier or aimed wrong,
        // and a capture with no hole in it cannot tell those apart.
        let altitude = eye.length() - PLANET_RADIUS;
        let cell = contact.finest_cell(eye.normalize_or(Vec3::Y));
        warn!("scripted dig found nothing: eye at {altitude:.1} m, cell {cell:?}");
    }
    if dug > 0 {
        *done = true;
        info!("scripted dig: {dug} layers taken under the camera");
    }
}
