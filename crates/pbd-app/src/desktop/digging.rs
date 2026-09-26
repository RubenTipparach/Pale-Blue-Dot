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
use pbd_app::planet::NearField;
use pbd_app::planet::surface_height;
use pbd_app::planet::{PLANET_RADIUS, PlanetContact, PlanetFine};
use pbd_app::saves::WorldSave;
use pbd_core::aim::{self, Sample};
use pbd_core::column::{self, LAYERS};
use pbd_core::dig::Step;
use pbd_core::edits::Edit;
use pbd_core::inventory::{Item, Tool};
use pbd_core::terrain::Material;
use std::sync::Arc;

/// What the player's ray met, kept for the HUD and for the click that follows.
#[derive(Resource, Default)]
pub struct Aim {
    pub target: Option<aim::Target>,
}

/// A block, as the digging rule names it: its cell's stable ID and its layer.
pub type Block = (u32, usize);

/// The block being broken and how far along it is (`pbd_core::dig`), kept
/// between frames because breaking is a hold. The crack overlay reads
/// `progress`.
#[derive(Resource, Default)]
pub struct Mining {
    breaking: pbd_core::dig::Breaking<Block>,
    /// The last break time looked up: for which block, tool and fine set
    /// version. A lookup finds the record and may build its column, so it is
    /// done once per target rather than every frame of a hold.
    lookup: Option<(Block, Tool, u64, Option<f32>)>,
}

impl Mining {
    /// The block being broken and its progress, 0 to 1.
    pub fn progress(&self) -> Option<(Block, f32)> {
        self.breaking.progress()
    }

    /// How long `block` takes with `tool`, remembered until either changes.
    fn secs(
        &mut self,
        block: Block,
        tool: Tool,
        fine: &PlanetFine,
        save: &WorldSave,
        dig: &pbd_core::dig::DigSettings,
    ) -> Option<f32> {
        if let Some((b, t, v, secs)) = self.lookup
            && b == block
            && t == tool
            && v == fine.version
        {
            return secs;
        }
        let secs = material_of(fine, save, block).and_then(|material| dig.secs(material, tool));
        self.lookup = Some((block, tool, fine.version, secs));
        secs
    }
}

/// What a block is made of: its column's answer, or the column the edit path
/// would adopt for it off the tier's edge.
fn material_of(fine: &PlanetFine, save: &WorldSave, (cell, layer): Block) -> Option<Material> {
    let record = record_of(fine, cell)?;
    match fine.set.columns.column(record) {
        Some(column) => Some(column.material(layer)),
        None => fine
            .set
            .adoptable(record, &save.edits)
            .map(|column| column.material(layer)),
    }
}

/// Which cell and layer a world point is in, and whether it is solid.
///
/// The walk is `PlanetContact`'s, which steps cell to cell across whichever
/// edge the ray falls outside of, so a march's samples cost a step from the
/// last rather than a search.
///
/// A finest record with no column answers from its own cap: solid below it.
/// That is the ground the heightfield draws there and exactly what the tier
/// would generate for it as rim, so the ray stops on the cap the player sees
/// and the edit adopts the column (`apply_edit`) instead of the click passing
/// into the hill as if it were sky.
fn sample_at(fine: &PlanetFine, contact: &PlanetContact, point: Vec3) -> Option<Sample> {
    let direction = point.try_normalize()?;
    // Only while the contact and `PlanetFine` hold the same set: for the frame
    // between a set landing in one and the other, the index is the wrong set's.
    if !contact.serves(&fine.set) {
        return None;
    }
    let record = contact.finest_cell(direction)?;
    let altitude = point.length() - PLANET_RADIUS;
    let layer = column::layer_at(altitude)?;
    let cell = fine.set.finest_records().get(record)?;
    let solid = match fine.set.columns.column(record) {
        Some(column) => column.solid(layer),
        None => column::layer_altitude(layer) + 0.5 < cell.direction_height[3],
    };
    Some(Sample {
        cell: cell.metadata[3],
        layer,
        solid,
    })
}

/// Why a point along the eye ray could not be sampled although it is inside
/// the ground: the finest level is not resident there at all. The march reads
/// air where the sampler answers nothing, so without this a click into a hill
/// the fine set has not reached looks exactly like a click at the sky. A
/// record with no column is not in this list: the sampler answers it from the
/// cap and the edit adopts the column.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Unsampled {
    /// The point is under the heightfield surface but no finest record covers
    /// its direction: outside the resident fine set.
    OffTheFineSet { depth_m: f32 },
}

/// What kept `sample_at` from answering at `point`, if the point is inside
/// the ground and so should have had an answer.
fn why_unsampled(contact: &PlanetContact, point: Vec3) -> Option<Unsampled> {
    let direction = point.try_normalize()?;
    let depth_m = surface_height(direction) - (point.length() - PLANET_RADIUS);
    if depth_m <= 0.0 {
        return None;
    }
    contact
        .finest_cell(direction)
        .is_none()
        .then_some(Unsampled::OffTheFineSet { depth_m })
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
        info!("edit refused: layer {layer} is bedrock or above the world");
        return None;
    }
    // The world is fully editable on foot, so an edit that finds no column
    // under a cell the march already resolved is a streaming failure and is
    // logged as one. The march only answers cells it sampled through a column,
    // so either of these is the set having changed under the click.
    let Some(record) = record_of(fine, cell) else {
        error!(
            "edit BLOCKED: cell {cell} is not in the resident fine set (version {}, {} columns)",
            fine.version,
            fine.set.columns.columns.len()
        );
        return None;
    };
    // A record past the tier's edge is given its column NOW rather than when
    // the next rebuild reaches it: generated solid with the save's edits, as
    // the rim is, and adopted below once the save has taken the edit.
    let adopting = match fine.set.columns.column(record) {
        Some(_) => None,
        None => fine.set.adoptable(record, &save.edits),
    };
    let Some(was) = adopting
        .as_ref()
        .or_else(|| fine.set.columns.column(record))
        .map(|column| column.material(layer))
    else {
        error!(
            "edit BLOCKED: cell {cell} has no column and none could be made (version {}, {} columns)",
            fine.version,
            fine.set.columns.columns.len()
        );
        return None;
    };
    if was == material {
        info!("edit refused: cell {cell} layer {layer} is already {material:?}");
        return None;
    }
    // The move is made on a COPY first. What the log records is the hotbar
    // after the edit, and what the player keeps is that same hotbar only if
    // the record was taken.
    // One lamp to a column. The record carries ONE torch layer, because a
    // torch is not in a run and the runs are all the shader reads, so a second
    // one would light a cell nothing was drawn in. Refusing is better than
    // drawing the wrong one.
    if material == Material::Torch
        && let Some(column) = adopting
            .as_ref()
            .or_else(|| fine.set.columns.column(record))
        && pbd_app::planet::column::has_torch(column)
    {
        info!("edit refused: cell {cell} already carries a torch");
        return None;
    }
    // Adopting takes a slot, so a full tier refuses here, BEFORE the save:
    // an edit that saved and could not be shown is the worse failure.
    if adopting.is_some()
        && fine.set.columns.columns.len() >= pbd_app::planet::column::COLUMN_CAPACITY as usize
    {
        error!("edit BLOCKED: cell {cell} needs a column and the tier is full");
        return None;
    }
    let mut moved = slots.0.clone();
    match hands {
        Hands::Take => {
            moved.give(Item::Block(was), 1);
        }
        Hands::Spend => {
            let index = moved.selected();
            if moved.take(index, 1) != 1 {
                info!("edit refused: nothing in the selected slot to place");
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
        // The durable path refused it, so the world must not change either:
        // an edit that shows and does not save is the worse of the two.
        error!("edit BLOCKED: the save did not accept cell {cell} layer {layer} {material:?}");
        return None;
    }
    slots.0 = moved;
    let started = std::time::Instant::now();
    let mut set = (*fine.set).clone();
    let cloned = started.elapsed();
    if let Some(column) = adopting {
        // Checked before the save took the edit, so this cannot refuse.
        let adopted = set.adopt(record, column);
        debug_assert!(adopted, "adoption was checked before the save");
        info!("adopted cell {cell} into the tier ahead of the streaming");
    }
    set.columns.set_layer(record, layer, material);
    set.columns.repack(record);
    for &neighbor in set.finest_neighbors[record].iter() {
        if neighbor != u32::MAX {
            set.columns.repack(neighbor as usize);
        }
    }
    // And the SURFACE, which is a second representation of where the ground
    // is: the terrain pass draws its cap from the cell record and every
    // neighbour draws its wall down to it. Repacking the runs and leaving that
    // alone is the bug the owner reported as broken geometry - the tier's own
    // build pass says what it looks like in as many words, "a meadow over the
    // hole and a wall across it", and that pass ran only at build.
    set.reconcile(record);
    // The whole tier, at the measured six milliseconds. A block changes what
    // light reaches every cell it can be seen from, which is not a region this
    // code can name: a dug shaft lets daylight forty metres down, and a torch
    // lights round a corner. The reference runs a bounded incremental pass
    // because a full one costs it a second; ours costs six milliseconds, so it
    // buys the exactness instead.
    let repacked = started.elapsed();
    set.columns.relight();
    let relit = started.elapsed();
    let set = Arc::new(set);
    contact.set_fine(&set);
    let contacted = started.elapsed();
    // A `debug!` rather than an `info!`: it is one line per block edit, which
    // is a wall of text while a player holds the button, and it is exactly
    // what you want the moment an edit feels slow.
    debug!(
        "edit cost: clone {:.2} ms, repack+reconcile {:.2} ms, relight {:.2} ms, \
         contact {:.2} ms, total {:.2} ms",
        cloned.as_secs_f64() * 1000.,
        (repacked - cloned).as_secs_f64() * 1000.,
        (relit - repacked).as_secs_f64() * 1000.,
        (contacted - relit).as_secs_f64() * 1000.,
        contacted.as_secs_f64() * 1000.,
    );
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
    near: Res<NearField>,
    tools: Res<pbd_app::fish::ToolSlot>,
    fishery: Option<Res<pbd_app::fish::Fishery>>,
    (mut mining, dig, time, swinging): (
        ResMut<Mining>,
        Res<pbd_app::config::DigConfig>,
        Res<Time>,
        Option<ResMut<pbd_app::held::Swinging>>,
    ),
) {
    // The tool in hand chops while a block is being broken: last frame's
    // answer, a frame behind, which nobody can see.
    let breaking = mining.progress().is_some();
    if let Some(mut swinging) = swinging
        && swinging.0 != breaking
    {
        swinging.0 = breaking;
    }
    let dt = time.delta_secs();
    let between = dig.0.between_s;
    let Some((transform, _)) = cameras.iter().find(|(_, camera)| camera.is_active) else {
        mining.breaking.step(dt, false, None, between);
        return;
    };
    // Only on foot, and only while the walker has the pointer. A ship's guns
    // are not a shovel, and a click that is really "give me the mouse back"
    // is not a dig: Bevy's UI does not consume the raw button, so without the
    // capture test a click on a menu row swings the shovel through the button
    // at whatever is behind it.
    if !walking.is_some_and(|readout| readout.active && readout.captured) {
        aimed.target = None;
        mining.breaking.step(dt, false, None, between);
        return;
    }
    let eye = transform.translation();
    let look = transform.forward().as_vec3();
    // The march reads "nothing" as air. Remember the first point it read that
    // way INSIDE the ground, because a click that lands there is a click the
    // world should have answered.
    let mut unsampled = None;
    let target = aim::march(eye, look, |point| {
        let sample = sample_at(&fine, &contact, point);
        if sample.is_none() && unsampled.is_none() {
            unsampled = why_unsampled(&contact, point);
        }
        sample
    });
    aimed.target = target;
    let clicked =
        buttons.just_pressed(MouseButton::Left) || buttons.just_pressed(MouseButton::Right);
    let Some(target) = target else {
        mining.breaking.step(dt, false, None, between);
        if clicked && let Some(why) = unsampled {
            error!(
                "edit BLOCKED: the eye ray entered ground the world cannot answer for: {why:?}; \
                 {}",
                near.line()
            );
        }
        return;
    };

    // The left button is the tool in hand's, and breaking is a HOLD: the
    // block goes when the button has been down on it for its break time
    // (`pbd_core::dig`). A rod casts rather than digs, and the fishing system
    // has that button; every other tool digs, the right one fastest.
    let tool = tools.held();
    let block = (target.dig.cell, target.dig.layer);
    let held = buttons.pressed(MouseButton::Left) && tool.digs();
    let secs = if held {
        mining.secs(block, tool, &fine, &edits, &dig.0)
    } else {
        None
    };
    let step = mining
        .breaking
        .step(dt, held, secs.map(|secs| (block, secs)), between);
    if let Step::Broken((cell, layer)) = step {
        if let Some(taken) = apply_edit(
            &mut Edited {
                fine: &mut fine,
                contact: &mut contact,
                save: &mut edits,
                slots: &mut slots,
            },
            Hands::Take,
            cell,
            layer,
            Material::Air,
        ) {
            info!(
                "dug {taken:?} from cell {cell} layer {layer} with the {} in {:.2} s, \
                 fine set version {}",
                tool.name(),
                secs.unwrap_or(0.0),
                fine.version
            );
        }
        return;
    }
    if held {
        return;
    }

    // With a line out, the right button winds it in and places nothing. This
    // runs before the fishing system, so it sees the line as it was when the
    // button went down.
    let line_out = fishery.is_some_and(|f| f.line.phase != pbd_core::fishing::Phase::Ready);
    if buttons.just_pressed(MouseButton::Right) && !line_out {
        let Some(place) = target.place else {
            info!("edit refused: no air along the ray to place into");
            return;
        };
        let Some(Item::Block(material)) = slots.held().map(|stack| stack.item) else {
            info!("edit refused: the selected slot holds no block");
            return;
        };
        // A block may not be placed into the player. Sealing yourself into
        // rock is a world you cannot move in, and the walker's own feet and
        // eye are what say where you are.
        if occupies(eye, place) {
            info!("edit refused: that cell is where the player stands");
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
                "placed {material:?} in cell {} layer {}, fine set version {}",
                place.cell, place.layer, fine.version
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
    if *done || launch.capture.is_none() || (launch.dig == 0 && !launch.torch) {
        return;
    }
    let Some((transform, _)) = cameras.iter().find(|(_, camera)| camera.is_active) else {
        return;
    };
    let eye = transform.translation();
    // Down, or along the look where the capture asked for it.
    let down = if launch.dig_ahead {
        transform.forward().as_vec3()
    } else {
        -eye.normalize_or(Vec3::Y)
    };
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
        // Every cell and layer taken, and what it is lit to once the edit's
        // own re-bake has run: a picture of a pit cannot say which column a
        // slanted dig landed in, nor whether the field under it is fresh.
        let lit = record_of(&fine, target.dig.cell)
            .and_then(|record| fine.set.columns.slots.get(record).copied())
            .map(|slot| fine.set.columns.sky(slot, target.dig.layer));
        info!(
            "scripted dig {}: cell {} layer {} now lit to {lit:?}",
            dug + 1,
            target.dig.cell,
            target.dig.layer
        );
        last = Some(target.dig);
        dug += 1;
    }
    if let Some(bottom) = last.filter(|_| launch.place > 0) {
        // A tower: N stones stacked on the hole. The harness has no player,
        // so the blocks come from nowhere, as they did before the hands were
        // part of the edit.
        for step in 1..=launch.place as usize {
            apply_edit(
                &mut Edited {
                    fine: &mut fine,
                    contact: &mut contact,
                    save: &mut edits,
                    slots: &mut slots,
                },
                Hands::Empty,
                bottom.cell,
                bottom.layer + step,
                Material::Stone,
            );
        }
    }
    // A torch on the ground under the camera, which is what `--torch` is for:
    // the place ray is the same one a dig uses, so the lamp lands in the air
    // the ground opens onto rather than inside it.
    if launch.torch
        && let Some(target) = aim::march(eye, down, |point| sample_at(&fine, &contact, point))
        && let Some(place) = target.place
    {
        let lit = apply_edit(
            &mut Edited {
                fine: &mut fine,
                contact: &mut contact,
                save: &mut edits,
                slots: &mut slots,
            },
            Hands::Empty,
            place.cell,
            place.layer,
            Material::Torch,
        );
        match lit {
            Some(_) => info!(
                "scripted torch in cell {} layer {}",
                place.cell, place.layer
            ),
            None => warn!("scripted torch refused at cell {}", place.cell),
        }
        *done = true;
    }
    if dug == 0 && !launch.torch {
        // Say why nothing happened rather than failing silently: a scripted
        // dig that finds no ground is either out of the tier or aimed wrong,
        // and a capture with no hole in it cannot tell those apart.
        let altitude = eye.length() - PLANET_RADIUS;
        let cell = contact.finest_cell(eye.normalize_or(Vec3::Y));
        warn!("scripted dig found nothing: eye at {altitude:.1} m, cell {cell:?}");
    }
    if dug > 0 {
        *done = true;
        // What the dug cells are LIT to, which is the one thing a picture of a
        // hole cannot tell you: a stale field and a fresh one draw the same
        // geometry, and the stale one draws it black. Rock has a sky level of
        // zero because rock holds no light, so a cell dug out of it stays at
        // zero until something re-bakes - which is exactly the bug this
        // reports, and why it reports a NUMBER.
        if let Some(bottom) = last {
            let record = record_of(&fine, bottom.cell);
            let tier = &fine.set.columns;
            let sky = record
                .and_then(|record| tier.slots.get(record).copied())
                .map(|slot| tier.sky(slot, bottom.layer))
                .unwrap_or(0);
            info!(
                "scripted dig: {dug} layers taken under the camera;                  the hole's floor cell is lit to {sky} of {}",
                pbd_core::light::MAX
            );
        }
    }
}
