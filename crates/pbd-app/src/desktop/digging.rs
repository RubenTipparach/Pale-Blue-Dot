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
use pbd_app::drops::Drops;
use pbd_app::planet::NearField;
use pbd_app::planet::surface_height;
use pbd_app::planet::{PLANET_RADIUS, PlanetContact, PlanetFine};
use pbd_app::saves::WorldSave;
use pbd_core::aim::{self, Sample};
use pbd_core::column::{self, LAYERS};
use pbd_core::dig::{Digger, Refusal, Step};
use pbd_core::edits::Edit;
use pbd_core::goods::Good;
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

/// The layer a tree being worked is named by in a [`Block`]: no layer, so
/// the cracks leave it alone (`tool-crafting` decision 3).
pub const TREE_LAYER: usize = usize::MAX;

/// The block being broken and how far along it is (`pbd_core::dig`), kept
/// between frames because breaking is a hold. The crack overlay reads
/// `progress`.
#[derive(Resource, Default)]
pub struct Mining {
    breaking: pbd_core::dig::Breaking<Block>,
    /// The last break time looked up: for which block, tool and fine set
    /// version. A lookup finds the record and may build its column, so it is
    /// done once per target rather than every frame of a hold.
    lookup: Option<(Block, Digger, u64, Result<f32, Refusal>)>,
}

impl Mining {
    /// The block being broken and its progress, 0 to 1.
    pub fn progress(&self) -> Option<(Block, f32)> {
        self.breaking.progress()
    }

    /// How long `block` takes for `digger`, or why it does not break,
    /// remembered until either changes.
    fn secs(
        &mut self,
        block: Block,
        digger: Digger,
        fine: &PlanetFine,
        save: &WorldSave,
        dig: &pbd_core::dig::DigSettings,
    ) -> Result<f32, Refusal> {
        if let Some((b, d, v, secs)) = self.lookup
            && b == block
            && d == digger
            && v == fine.version
        {
            return secs;
        }
        let secs = material_of(fine, save, block)
            .ok_or(Refusal::NotABlock)
            .and_then(|material| dig.secs_by(material, digger));
        self.lookup = Some((block, digger, fine.version, secs));
        secs
    }
}

/// What a block is made of: its column's answer, or the column the edit path
/// would adopt for it off the tier's edge.
pub(super) fn material_of(
    fine: &PlanetFine,
    save: &WorldSave,
    (cell, layer): Block,
) -> Option<Material> {
    let record = record_of(fine, cell)?;
    match fine.set.columns.column(record) {
        Some(column) => Some(column.material(layer)),
        None => fine
            .set
            .adoptable(record, &save.edits)
            .map(|column| column.material(layer)),
    }
}

/// Whether a tree has been felled: the axe has taken all its logs.
pub(super) fn tree_felled(taken: u16, axe_logs: u16) -> bool {
    taken >= axe_logs
}

/// Whether `block` is the ground a standing tree grows from: its cell grows a
/// tree the axe has not felled, and the block is the cell's top (solid, with
/// air over it). That ground is not dug or tilled until the tree is cut down
/// (`tool-crafting` decision 3a).
pub(super) fn under_standing_tree(
    fine: &PlanetFine,
    save: &WorldSave,
    axe_logs: u16,
    (cell, layer): Block,
) -> bool {
    use pbd_core::gather::{TREE_KIND, TreeTaken};
    let Some(record) = record_of(fine, cell) else {
        return false;
    };
    let r = &fine.set.finest_records()[record];
    if !pbd_core::flora::grows_tree(
        &pbd_app::planet::terrain_config().trees,
        r.spare[0],
        r.metadata[1],
    ) {
        return false;
    }
    let taken = save
        .records
        .get(TREE_KIND, u64::from(cell))
        .and_then(|r| r.read::<TreeTaken>())
        .map_or(0, |t| t.logs);
    if tree_felled(taken, axe_logs) {
        return false;
    }
    material_of(fine, save, (cell, layer)).is_some_and(|m| m != Material::Air && !m.is_water())
        && material_of(fine, save, (cell, layer + 1)) == Some(Material::Air)
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
pub(super) fn sample_at(fine: &PlanetFine, contact: &PlanetContact, point: Vec3) -> Option<Sample> {
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
        cell: cell.key(),
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

/// The record index of a cell by its key. The march answers in keys because
/// that is what an edit is saved by; applying one needs the slot.
pub(super) fn record_of(fine: &PlanetFine, cell: u32) -> Option<usize> {
    fine.set
        .finest_records()
        .iter()
        .position(|record| record.key() == cell)
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
    /// A till (`farming-and-smelting` decision 2): the hands take nothing,
    /// but the soil may turn up a seed, which floats out as a dig's block
    /// does, on the edit's own line.
    Till(Option<Item>),
    /// Trade one of the selected stack for this item, on the same line: a
    /// bucket filled or emptied (`water-flow` E11).
    Swap(Item),
    /// A dig that also turns up finds into the hands, on the same line: a
    /// seed out of dug grass (`farming-and-smelting` decision 14), fibre out
    /// of grass and loose stone out of earth (`tool-crafting` decision 2),
    /// bait out of wet ground at the shore (`starter-missions` G1).
    TakeFinds([Option<Item>; 3]),
}

/// What a dug block yields (`farming-and-smelting` decisions 2 and 10):
/// tilled soil gives dirt; a generated ore cell gives mixed ore or, in a coal
/// seam, coal; a placed one gives ore; everything else gives itself.
pub(super) fn yield_of(save: &WorldSave, was: Material, cell: u32, layer: usize) -> Item {
    match was {
        Material::Tilled => Item::Block(Material::Dirt),
        Material::Ore => {
            let placed = save
                .edits
                .for_cell(cell)
                .iter()
                .any(|&(l, m)| usize::from(l) == layer && m == Material::Ore);
            pbd_core::station::ore_yield(cell, layer as u16, placed)
        }
        other => Item::Block(other),
    }
}

/// Everything one edit touches, which is what says these travel together: the
/// geometry, what the walker stands on, the save, the hands, and the world a
/// dug block floats in (`inventory-grid` decision 4).
/// A shorter argument list is the symptom; the reason is that an edit is a
/// transaction over exactly these.
pub struct Edited<'a> {
    pub fine: &'a mut PlanetFine,
    pub contact: &'a mut PlanetContact,
    pub save: &'a mut WorldSave,
    pub slots: &'a mut super::slots::Hotbar,
    pub drops: &'a mut Drops,
    /// The world's time, s: when a drop is made.
    pub now_s: f64,
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
        drops,
        now_s,
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
    // One lamp to a column. The record carries ONE lamp layer, because a
    // lamp is not in a run and the runs are all the shader reads, so a second
    // one would light a cell nothing was drawn in. Refusing is better than
    // drawing the wrong one.
    if material.is_lamp()
        && let Some(column) = adopting
            .as_ref()
            .or_else(|| fine.set.columns.column(record))
        && pbd_app::planet::column::has_lamp(column)
    {
        info!("edit refused: cell {cell} already carries a lamp");
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
    let mut dropped = None;
    match hands {
        // The dug block floats where it was cut, at the cell's centre, and
        // the magnet brings it in (`inventory-grid` decision 4). The hands do
        // not change here; the pickup is its own line.
        Hands::Take | Hands::TakeFinds(_) | Hands::Till(Some(_)) => {
            // What does not fit is not given: a find is not a block the
            // player is owed.
            if let Hands::TakeFinds(finds) = hands {
                for find in finds.into_iter().flatten() {
                    moved.give(find, 1);
                }
            }
            let item = match hands {
                Hands::Till(Some(item)) => item,
                _ => yield_of(save, was, cell, layer),
            };
            let direction = fine
                .set
                .finest_records()
                .get(record)
                .map(|cell| Vec3::from_slice(&cell.direction_height[..3]))
                .and_then(Vec3::try_normalize);
            if let Some(direction) = direction {
                // A till's seed comes up out of the soil, over the cell.
                let lift = if matches!(hands, Hands::Till(_)) {
                    1.3
                } else {
                    0.5
                };
                // The body's frame in f64 (`world-frames` 10.4).
                let centre = direction.as_dvec3().normalize()
                    * (f64::from(PLANET_RADIUS) + f64::from(column::layer_altitude(layer)) + lift);
                dropped = Some(Drops::make(save, item, centre, *now_s));
            }
        }
        Hands::Spend => {
            let index = moved.selected();
            if moved.take(index, 1) != 1 {
                info!("edit refused: nothing in the selected slot to place");
                return None;
            }
        }
        Hands::Swap(item) => {
            let index = moved.selected();
            if moved.take(index, 1) != 1 {
                info!("edit refused: nothing in the selected slot to use");
                return None;
            }
            moved.give(item, 1);
        }
        Hands::Empty | Hands::Till(None) => {}
    }
    // Whether what this replaces is something a player put there: its
    // removal is logged by author (the owner, 2026-11-16: "if an object a
    // user placed despawns, i need logs").
    let was_placed = !matches!(was, Material::Air)
        && !was.is_water()
        && save
            .edits
            .for_cell(cell)
            .iter()
            .any(|&(l, m)| usize::from(l) == layer && m == was);
    if !save.accept(
        Edit {
            cell,
            layer: layer as u16,
            material,
        },
        dropped.as_ref(),
        &moved,
    ) {
        // The durable path refused it, so the world must not change either:
        // an edit that shows and does not save is the worse of the two.
        error!("edit BLOCKED: the save did not accept cell {cell} layer {layer} {material:?}");
        return None;
    }
    if was_placed {
        info!("REMOVED {was:?} at cell {cell} layer {layer} by the player, now {material:?}");
    }
    slots.0 = moved;
    drops.live.extend(dropped);
    let started = std::time::Instant::now();
    let mut set = (*fine.set).clone();
    let cloned = started.elapsed();
    if let Some(column) = adopting {
        // Checked before the save took the edit, so this cannot refuse.
        let adopted = set.adopt(record, column);
        debug_assert!(adopted, "adoption was checked before the save");
        info!("adopted cell {cell} into the tier ahead of the streaming");
    }
    // The column, its record and its neighbours', the water and the surface
    // (`FineSet::write_layer`, the one path a landing's replay takes too).
    // The surface matters as much as the runs: repacking the runs and leaving
    // the record's cap alone is the bug the owner reported as broken geometry,
    // "a meadow over the hole and a wall across it".
    set.write_layer(record, layer, material, &save.edits);
    // The whole tier, at the measured six milliseconds. A block changes what
    // light reaches every cell it can be seen from, which is not a region this
    // code can name: a dug shaft lets daylight forty metres down, and a torch
    // lights round a corner. The reference runs a bounded incremental pass
    // because a full one costs it a second; ours costs six milliseconds, so it
    // buys the exactness instead.
    let repacked = started.elapsed();
    set.columns.relight();
    // The sea's sheet over the tier, its shade from the light just baked.
    set.mark_sea();
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
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn dig_and_place(
    (cameras, frame): (
        Query<(&GlobalTransform, &Camera), With<Camera3d>>,
        Option<Res<pbd_app::planet::PlanetRenderFrame>>,
    ),
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
    (mut mining, dig, time, swinging, mut drops, sun, mut ground): (
        ResMut<Mining>,
        Res<pbd_app::config::DigConfig>,
        Res<Time>,
        Option<ResMut<pbd_app::held::Swinging>>,
        ResMut<Drops>,
        Res<pbd_app::sky::Sun>,
        // Absent in a still capture, which has no husbandry.
        Option<ResMut<pbd_app::husbandry::AimedGround>>,
    ),
    (mut note, farm, mut gained, mut planted, gather, effects, mut did): (
        ResMut<super::farm::FarmNote>,
        Res<pbd_app::config::FarmConfig>,
        MessageWriter<pbd_app::hotbar::Gained>,
        MessageWriter<pbd_app::hotbar::Planted>,
        Res<pbd_app::config::GatherConfig>,
        Option<Res<pbd_app::kitchen::MealEffects>>,
        MessageWriter<pbd_app::missions::Did>,
    ),
) {
    if let Some(ground) = ground.as_mut() {
        ground.0 = None;
    }
    let dt = time.delta_secs();
    let between = dig.0.between_s;
    // The tool in hand strokes on the dig's beat (`held-swing` decision 5):
    // last frame's answer, a frame behind, which nobody can see. While a
    // block breaks, its time and how far in; in the pause after it, on
    // past the break.
    if let Some(mut swinging) = swinging {
        let was = swinging.0;
        let beat = match mining.breaking.timing() {
            Some((break_s, elapsed_s)) => pbd_app::held::stroke::DigBeat {
                working: true,
                break_s,
                between_s: between,
                local_s: elapsed_s,
            },
            None if was.working && mining.breaking.pause_left() > 0.0 => {
                pbd_app::held::stroke::DigBeat {
                    local_s: was.break_s + (between - mining.breaking.pause_left()).max(0.0),
                    ..was
                }
            }
            None => pbd_app::held::stroke::DigBeat::default(),
        };
        if was != beat {
            swinging.0 = beat;
        }
    }
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
    let eye = pbd_app::render_origin::local_of(frame.as_deref(), transform.translation());
    let look = transform.forward().as_vec3();
    // The march reads "nothing" as air. Remember the first point it read that
    // way INSIDE the ground, because a click that lands there is a click the
    // world should have answered.
    let mut unsampled = None;
    let mut solid_m: Option<f32> = None;
    let target = aim::march(eye, look, |point| {
        let sample = sample_at(&fine, &contact, point);
        if sample.is_none() && unsampled.is_none() {
            unsampled = why_unsampled(&contact, point);
        }
        if solid_m.is_none() && sample.is_some_and(|s| s.solid) {
            solid_m = Some(point.distance(eye));
        }
        sample
    });
    aimed.target = target;
    // What breaks: bare hands dig soft ground (`tool-crafting` decision 1),
    // a tool at its tier; the rod nothing.
    let digger = if tools.bare_handed() {
        Some(Digger::Hands)
    } else {
        tools.digging().map(|(tool, tier)| Digger::Tool(tool, tier))
    };
    // A tree's trunk nearer than the ground is worked for a log (decision 3).
    if buttons.pressed(MouseButton::Left)
        && digger.is_some()
        && let Some(tree) = tree_on_ray(&fine, eye, look, solid_m.unwrap_or(aim::REACH_M))
    {
        let by_axe = matches!(digger, Some(Digger::Tool(Tool::Axe, _)));
        let wood = dig.0.wood.axe;
        let secs = match digger {
            Some(Digger::Tool(Tool::Axe, tier)) => {
                wood / dig.0.tier_speeds[usize::from(tier.digit())]
            }
            _ => wood / dig.0.hand_speed,
        };
        let step = mining
            .breaking
            .step(dt, true, Some(((tree, TREE_LAYER), secs)), between);
        if let Step::Broken(_) = step {
            take_log(
                &mut edits,
                &mut slots,
                &gather,
                tree,
                by_axe,
                &mut note,
                time.elapsed_secs(),
                &mut gained,
                &mut did,
            );
        }
        return;
    }
    // Where the ray met the ground, for a fence piece's edge
    // (`animal-husbandry` S12).
    if let Some(ground) = ground.as_mut() {
        ground.0 = target.map(|t| eye + look * t.distance);
    }
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
    // has that button; bare hands break nothing (`inventory-grid` decision
    // 8); every other tool digs, the right one fastest.
    let block = (target.dig.cell, target.dig.layer);
    let rooted = under_standing_tree(&fine, &edits, gather.0.axe_logs, block);
    if rooted && buttons.just_pressed(MouseButton::Left) && digger.is_some() {
        note.say(time.elapsed_secs(), "Cut down the tree first");
    }
    let held = buttons.pressed(MouseButton::Left) && digger.is_some() && !rooted;
    let secs = match digger {
        // Keen (`cooking` design S5): a break takes its time over the
        // effect's strength while it runs.
        Some(digger) if held => mining
            .secs(block, digger, &fine, &edits, &dig.0)
            .map(|secs| secs / effects.as_deref().map_or(1.0, |e| e.keen)),
        _ => Err(Refusal::NotABlock),
    };
    // Say once, on the press, why a block will not break.
    if buttons.just_pressed(MouseButton::Left)
        && let Err(why) = secs
    {
        let wall_s = time.elapsed_secs();
        match why {
            Refusal::NotByHand => note.say(wall_s, "Too hard to dig by hand"),
            Refusal::TierTooLow(tier) => {
                note.say(wall_s, format!("Needs a {} pickaxe", tier.name()))
            }
            Refusal::NotADiggingTool | Refusal::NotABlock => {}
        }
    }
    let secs = secs.ok();
    let step = mining
        .breaking
        .step(dt, held, secs.map(|secs| (block, secs)), between);
    if let Step::Broken((cell, layer)) = step {
        let wall_s = time.elapsed_secs();
        let was = material_of(&fine, &edits, (cell, layer));
        // A station with work in it is not broken (decision 7): nothing in a
        // queue is lost to a pick.
        if let Some(row) = was.and_then(pbd_core::station::station_of)
            && super::farm::station_state(&edits, cell, layer as u16, row).busy()
        {
            note.say(wall_s, format!("Empty the {} first", row.name));
            return;
        }
        // Dug grass turns up a seed, one cell in a few (decision 14).
        let seed = was
            .filter(|m| pbd_core::farming::sprouts_seed(*m))
            .and_then(|_| {
                farm.0
                    .grass_seed(cell, super::farm::climate_at(&fine, cell, layer))
            });
        // And tier 0 (`tool-crafting` decision 2): fibre from grass, loose
        // stone from earth, never from a block a player placed.
        let placed = was.is_some_and(|m| {
            edits
                .edits
                .for_cell(cell)
                .iter()
                .any(|&(l, e)| usize::from(l) == layer && e == m)
        });
        let find = was.and_then(|m| gather.0.find(m, cell, layer as u16, placed));
        // Bait worms out of wet ground at the shore (`starter-missions` G1).
        let bait = was.and_then(|m| {
            gather
                .0
                .bait(m, cell, layer as u16, placed, column::layer_altitude(layer))
        });
        let hands = if seed.is_none() && find.is_none() && bait.is_none() {
            Hands::Take
        } else {
            Hands::TakeFinds([
                seed.map(Item::Good),
                find.map(Item::Good),
                bait.map(Item::Good),
            ])
        };
        if let Some(taken) = apply_edit(
            &mut Edited {
                fine: &mut fine,
                contact: &mut contact,
                save: &mut edits,
                slots: &mut slots,
                drops: &mut drops,
                now_s: sun.clock.seconds,
            },
            hands,
            cell,
            layer,
            Material::Air,
        ) {
            for good in [seed, find, bait].into_iter().flatten() {
                gained.write(pbd_app::hotbar::Gained {
                    item: Item::Good(good),
                    count: 1,
                    how: pbd_app::hotbar::How::Found,
                });
            }
            if taken == Material::Tilled {
                super::farm::uproot(&mut edits, &slots.0, cell);
            }
            // What the starter missions count: a block dug (`gather`).
            did.write(pbd_app::missions::Did::block(
                pbd_core::missions::Kind::Gather,
                taken,
            ));
            info!(
                "dug {taken:?} from cell {cell} layer {layer} with the {} in {:.2} s, \
                 fine set version {}",
                match digger {
                    Some(Digger::Tool(tool, tier)) => pbd_core::inventory::tool_title(tool, tier),
                    _ => "bare hands".to_string(),
                },
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
            info!("PLACE refused: no air along the ray to place into");
            return;
        };
        let wall_s = time.elapsed_secs();
        // A seed is planted on the tilled soil the ray meets
        // (`farming-and-smelting` decision 3).
        if let Some(Item::Good(seed)) = slots.held().map(|stack| stack.item)
            && pbd_core::farming::Crop::of_seed(seed).is_some()
        {
            super::farm::plant(
                &mut Edited {
                    fine: &mut fine,
                    contact: &mut contact,
                    save: &mut edits,
                    slots: &mut slots,
                    drops: &mut drops,
                    now_s: sun.clock.seconds,
                },
                target.dig.cell,
                target.dig.layer,
                seed,
                &mut note,
                wall_s,
                &mut planted,
            );
            return;
        }
        // The wooden bucket (`water-flow` E11): an empty one takes the
        // source the eye meets, a full one pours where a block would go.
        if let Some(Item::Good(good)) = slots.held().map(|stack| stack.item)
            && (good == Good::BUCKET || good == Good::WATER_BUCKET)
        {
            let full = good == Good::WATER_BUCKET;
            let at = if full {
                Some((place.cell, place.layer))
            } else {
                // The ray stops at water for a bucket, as it does not for a
                // hand: the first source along it.
                aim::march(eye, look, |point| {
                    sample_at(&fine, &contact, point).map(|mut s| {
                        if material_of(&fine, &edits, (s.cell, s.layer)) == Some(Material::Water) {
                            s.solid = true;
                        }
                        s
                    })
                })
                .map(|t| (t.dig.cell, t.dig.layer))
            };
            let Some((cell, layer)) = at else {
                info!("PLACE refused: the bucket found no water or cell along the ray");
                return;
            };
            let Some(now) = material_of(&fine, &edits, (cell, layer))
                .and_then(|was| pbd_core::liquid::bucket(full, was))
            else {
                info!("PLACE refused: the bucket has nothing to take or nowhere to pour");
                return;
            };
            let swapped = if full {
                Good::BUCKET
            } else {
                Good::WATER_BUCKET
            };
            let poured = full;
            if apply_edit(
                &mut Edited {
                    fine: &mut fine,
                    contact: &mut contact,
                    save: &mut edits,
                    slots: &mut slots,
                    drops: &mut drops,
                    now_s: sun.clock.seconds,
                },
                Hands::Swap(Item::Good(swapped)),
                cell,
                layer,
                now,
            )
            .is_some()
            {
                info!(
                    "{} cell {cell} layer {layer} with the bucket",
                    if full {
                        "poured into"
                    } else {
                        "took water from"
                    }
                );
                if poured {
                    did.write(pbd_app::missions::Did::new(
                        pbd_core::missions::Kind::Pour,
                        "water",
                        1,
                    ));
                }
            }
            return;
        }
        let Some(Item::Block(material)) = slots.held().map(|stack| stack.item) else {
            info!("PLACE refused: the selected slot holds no block");
            return;
        };
        if super::farm::crop_in_the_way(&fine, &edits, place.cell, place.layer) {
            info!(
                "PLACE refused: a crop grows in cell {} layer {}",
                place.cell, place.layer
            );
            note.say(wall_s, "Something is planted here");
            return;
        }
        // A block may not be placed into the player. Sealing yourself into
        // rock is a world you cannot move in, and the walker's own feet and
        // eye are what say where you are.
        if occupies(&fine, &contact, eye, place) {
            info!(
                "PLACE refused: {material:?} into cell {} layer {} is where the player stands",
                place.cell, place.layer
            );
            return;
        }
        if apply_edit(
            &mut Edited {
                fine: &mut fine,
                contact: &mut contact,
                save: &mut edits,
                slots: &mut slots,
                drops: &mut drops,
                now_s: sun.clock.seconds,
            },
            Hands::Spend,
            place.cell,
            place.layer,
            material,
        )
        .is_some()
        {
            info!(
                "PLACE ok: {material:?} in cell {} layer {}, fine set version {}",
                place.cell, place.layer, fine.version
            );
            did.write(pbd_app::missions::Did::block(
                pbd_core::missions::Kind::Place,
                material,
            ));
        } else {
            // The reason is the `edit refused` / `edit BLOCKED` line just above.
            info!(
                "PLACE refused: {material:?} into cell {} layer {} (reason in the line above)",
                place.cell, place.layer
            );
        }
    }
}

/// The tree whose trunk the eye ray meets nearer than `before_m` and within
/// reach, by its cell's key: a vertical column at the centre of a cell that
/// grows a tree (`pbd_core::flora::grows_tree`, the foliage pass's own rule),
/// the trunk's width low down and the canopy's higher up.
pub fn tree_on_ray(fine: &PlanetFine, eye: Vec3, look: Vec3, before_m: f32) -> Option<u32> {
    let reach = before_m.min(aim::REACH_M);
    let toward = eye.normalize_or(Vec3::Y);
    let near = ((aim::REACH_M + 3.0) / PLANET_RADIUS).cos();
    let mut best: Option<(f32, u32)> = None;
    for record in fine.set.finest_records() {
        let up = Vec3::from_slice(&record.direction_height[..3]);
        let Some(up) = up.try_normalize() else {
            continue;
        };
        if up.dot(toward) < near
            || !pbd_core::flora::grows_tree(
                &pbd_app::planet::terrain_config().trees,
                record.spare[0],
                record.metadata[1],
            )
        {
            continue;
        }
        let base = up * (PLANET_RADIUS + record.direction_height[3]);
        // The ray in the trunk's frame: across (perpendicular to up) and up.
        let rel = eye - base;
        let d_across = look - up * look.dot(up);
        let p_across = rel - up * rel.dot(up);
        let a = d_across.length_squared();
        if a < 1e-6 {
            continue;
        }
        for (radius, low, high) in [(0.45_f32, 0.0_f32, 3.0_f32), (1.0, 3.0, 6.5)] {
            let b = 2.0 * p_across.dot(d_across);
            let c = p_across.length_squared() - radius * radius;
            let disc = b * b - 4.0 * a * c;
            if disc < 0.0 {
                continue;
            }
            let t = (-b - disc.sqrt()) / (2.0 * a);
            let t = if t < 0.0 {
                (-b + disc.sqrt()) / (2.0 * a)
            } else {
                t
            };
            if !(0.0..=reach).contains(&t) {
                continue;
            }
            let height = (rel + look * t).dot(up);
            if (low..=high).contains(&height) && best.is_none_or(|(bt, _)| t < bt) {
                best = Some((t, record.key()));
            }
        }
    }
    best.map(|(_, key)| key)
}

/// A log from a worked tree into the slots, and what the tree has given, in
/// one saved line (`tool-crafting` decision 3).
#[allow(clippy::too_many_arguments)]
fn take_log(
    save: &mut WorldSave,
    slots: &mut super::slots::Hotbar,
    gather: &pbd_app::config::GatherConfig,
    tree: u32,
    by_axe: bool,
    note: &mut super::farm::FarmNote,
    wall_s: f32,
    gained: &mut MessageWriter<pbd_app::hotbar::Gained>,
    did: &mut MessageWriter<pbd_app::missions::Did>,
) {
    use pbd_core::gather::{TREE_KIND, TREE_SCHEMA, TreeSpent, TreeTaken};
    let taken: TreeTaken = save
        .records
        .get(TREE_KIND, u64::from(tree))
        .and_then(|r| r.read())
        .unwrap_or_default();
    match gather.0.tree_gives(taken.logs, by_axe) {
        // Nothing to say (owner, 2026-11-17: the line "sounds like AI writing").
        Err(TreeSpent::BranchTaken) => return,
        Err(TreeSpent::Felled) => {
            note.say(wall_s, "No more wood");
            return;
        }
        Ok(()) => {}
    }
    let log = Item::Good(pbd_core::goods::Good::LOG);
    let mut moved = slots.0.clone();
    if moved.give(log, 1) > 0 {
        note.say(wall_s, "No room in the pack");
        return;
    }
    let record = pbd_core::records::Record::of(
        TREE_KIND,
        u64::from(tree),
        TREE_SCHEMA,
        &TreeTaken {
            logs: taken.logs + 1,
        },
    );
    if !save.store_with_hands(record, TREE_SCHEMA, &moved) {
        error!("log BLOCKED: the save did not accept it");
        return;
    }
    slots.0 = moved;
    gained.write(pbd_app::hotbar::Gained {
        item: log,
        count: 1,
        how: pbd_app::hotbar::How::Found,
    });
    // The toast says it (`item-toasts` decision 6).
    info!("took log {} from the tree at cell {tree}", taken.logs + 1);
    // The axe's last log fells the tree (`starter-missions` H.1).
    if by_axe && gather.0.tree_gives(taken.logs + 1, true).is_err() {
        did.write(pbd_app::missions::Did::new(
            pbd_core::missions::Kind::Fell,
            "tree",
            1,
        ));
        info!("felled the tree at cell {tree}");
    }
}

/// Whether the player's own body stands in this cell and layer.
///
/// The eye is the camera; the body is the two layers under it, which is the
/// walker's capsule rounded to the lattice, in the cells the capsule's
/// footprint touches. A block placed in either is a block placed in the
/// player. Only those cells: the layer alone refused every block set on
/// level ground beside the player, a workbench or a campfire included
/// (`step-videos` 5.2, found by the showcase's bot).
fn occupies(fine: &PlanetFine, contact: &PlanetContact, eye: Vec3, place: Sample) -> bool {
    let altitude = eye.length() - PLANET_RADIUS;
    let Some(head) = column::layer_at(altitude) else {
        return false;
    };
    let feet = head.saturating_sub(1);
    if !(feet..=head).contains(&place.layer) {
        return false;
    }
    // The capsule's footprint: its axis and four points on its rim.
    let up = eye.normalize_or(Vec3::Y);
    let east = Vec3::Y.cross(up).normalize_or(Vec3::X);
    let north = up.cross(east);
    let rim = 0.35;
    // Keys from the contact tier's own set: a key is the same in every set,
    // so this holds while the tier lags the newest set after an edit. Asking
    // `contact.serves(&fine.set)` here refused every place at foot level in
    // that window, and the window followed every edit (owner, 2026-11-16:
    // "now suddenly I cant place anything").
    let _ = fine;
    let keys: Vec<u32> = [Vec3::ZERO, east, -east, north, -north]
        .into_iter()
        .filter_map(|offset| contact.finest_key((eye + offset * rim).normalize()))
        .collect();
    // No contact tier at all (the first frames): the body may be anywhere,
    // so refuse, as before.
    keys.is_empty() || keys.contains(&place.cell)
}

/// `--lamps`: every light in a row across the view, one cell apart, each on
/// the ground of its own cell (`lamps-and-lanterns` task 5.2). The wall
/// lantern gets a two-block pillar in the cell beyond it to hang on, and the
/// hanging lantern a block over it to hang from. Returns how many lights were
/// placed.
/// The cell `metres` ahead of the eye along the ground and `across` to its
/// right, as its key and the first layer over its ground: where a capture's
/// row of lamps or blocks stands each one.
fn row_spot(
    world: &Edited,
    eye: Vec3,
    forward: Vec3,
    metres: f32,
    across: f32,
) -> Option<(u32, usize)> {
    let up = eye.normalize_or(Vec3::Y);
    let ahead = (forward - up * forward.dot(up)).normalize_or(Vec3::X);
    let right = ahead.cross(up).normalize_or(Vec3::Z);
    let offset = ahead * metres + right * across;
    let angle = offset.length() / PLANET_RADIUS;
    let direction = (up * angle.cos() + offset.normalize_or(ahead) * angle.sin()).normalize();
    let record = world.contact.finest_cell(direction)?;
    let key = world.fine.set.finest_records().get(record)?.key();
    let top = world.fine.set.columns.column(record)?.surface()?;
    Some((key, top + 1))
}

/// `--place-row` (`placed-blocks-keep-material`): each block on the ground
/// in a row across the view, `row` metres ahead, one cell apart, each in a
/// cell of its own. Nought before the fine set is under the camera.
fn block_row(world: &mut Edited, eye: Vec3, forward: Vec3, row: f32, blocks: &[Material]) -> usize {
    if row_spot(world, eye, forward, row, 0.0).is_none() {
        return 0;
    }
    let mut used = Vec::new();
    let mut across = -0.5 * LAMP_SPACING_M * (blocks.len() as f32 - 1.0);
    let mut placed = 0;
    for &block in blocks {
        let mut found = None;
        for _ in 0..16 {
            match row_spot(world, eye, forward, row, across) {
                Some((cell, layer)) if !used.contains(&cell) => {
                    found = Some((cell, layer));
                    break;
                }
                _ => across += 0.5,
            }
        }
        if let Some((cell, layer)) = found {
            used.push(cell);
            if apply_edit(world, Hands::Empty, cell, layer, block).is_some() {
                info!("scripted {block:?} placed in cell {cell} layer {layer}");
                placed += 1;
            }
        }
        across += LAMP_SPACING_M;
    }
    placed
}

fn lamp_row(world: &mut Edited, eye: Vec3, forward: Vec3, row: f32, aside: f32) -> usize {
    let spot =
        |world: &Edited, metres: f32, across: f32| row_spot(world, eye, forward, metres, across);
    // Before the fine set is under the camera there is no cell to put a lamp
    // in, and the caller tries again next frame.
    if spot(world, row, 0.0).is_none() {
        return 0;
    }
    let mut placed = 0;
    // One row across the view, so a single frame shows every light. Each
    // light walks right from the last until it is in a cell nobody has used:
    // a fixed step across a hex grid sometimes lands twice in one cell.
    let mut used = Vec::new();
    let mut across = aside - 2.5 * LAMP_SPACING_M;
    for &lamp in Material::LAMPS.iter() {
        let mut found = None;
        for _ in 0..16 {
            match spot(world, row, across) {
                Some((cell, layer)) if !used.contains(&cell) => {
                    found = Some((cell, layer));
                    break;
                }
                _ => across += 0.5,
            }
        }
        let Some((cell, layer)) = found else {
            warn!("scripted lamps: no free cell {across:.1} m across for {lamp:?}");
            continue;
        };
        used.push(cell);
        // A wall lantern needs a wall: two stones in the next cell back.
        if lamp == Material::LanternWall
            && let Some((wall, base)) = spot(world, row + LAMP_SPACING_M, across)
        {
            used.push(wall);
            for step in 0..2 {
                apply_edit(world, Hands::Empty, wall, base + step, Material::Stone);
            }
        }
        // A hanging lantern needs something to hang from: a stone over it.
        let layer = if lamp == Material::LanternHanging {
            apply_edit(world, Hands::Empty, cell, layer + 2, Material::Stone);
            layer + 1
        } else {
            layer
        };
        if apply_edit(world, Hands::Empty, cell, layer, lamp).is_some() {
            info!("scripted {lamp:?} in cell {cell} layer {layer}, {across:.1} m across");
            placed += 1;
        }
        across += LAMP_SPACING_M;
    }
    placed
}

/// How far apart `--lamps` puts its lights: about one cell, so each stands in
/// a cell of its own.
const LAMP_SPACING_M: f32 = 2.9;

/// The scripted dig: a headless run has no mouse, and a picture of a hole is
/// the only thing that says the verb works end to end.
///
/// It digs straight DOWN from the camera rather than along its look, because a
/// capture's camera is aimed at whatever its view frames and the one thing
/// certainly in reach of a standing player is the ground under them. Then
/// `--place` puts one block back on top of the hole, so a frame shows both
/// verbs: a pit, and a block standing in it.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn scripted_dig(
    launch: Res<super::Launch>,
    (cameras, frame): (
        Query<(&GlobalTransform, &Camera), With<Camera3d>>,
        Option<Res<pbd_app::planet::PlanetRenderFrame>>,
    ),
    mut fine: ResMut<PlanetFine>,
    mut contact: ResMut<PlanetContact>,
    mut edits: ResMut<WorldSave>,
    mut slots: ResMut<super::slots::Hotbar>,
    (mut drops, sun): (ResMut<Drops>, Res<pbd_app::sky::Sun>),
    mut done: Local<bool>,
) {
    if *done
        || launch.capture.is_none()
        || (launch.dig == 0 && !launch.torch && !launch.lamps && launch.place_row.is_empty())
    {
        return;
    }
    let Some((transform, _)) = cameras.iter().find(|(_, camera)| camera.is_active) else {
        return;
    };
    let eye = pbd_app::render_origin::local_of(frame.as_deref(), transform.translation());
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
                drops: &mut drops,
                now_s: sun.clock.seconds,
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
                    drops: &mut drops,
                    now_s: sun.clock.seconds,
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
                drops: &mut drops,
                now_s: sun.clock.seconds,
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
    if launch.lamps {
        let forward = transform.forward().as_vec3();
        let placed = lamp_row(
            &mut Edited {
                fine: &mut fine,
                contact: &mut contact,
                save: &mut edits,
                slots: &mut slots,
                drops: &mut drops,
                now_s: sun.clock.seconds,
            },
            eye,
            forward,
            launch.lamps_at,
            launch.lamps_across,
        );
        // Before the fine set is under the camera there is no cell to put a
        // lamp in; the next frame tries again, as the scripted dig does.
        if placed > 0 {
            info!("scripted lamps: {placed} placed");
            // And a stack of each in the hotbar, in place of the kit's
            // blocks, so the same frame shows every light's own icon (task
            // 5.3). A capture's save is memory-only, so nothing is kept.
            let mut carried = pbd_core::inventory::Slots::new();
            for light in Material::LAMPS {
                carried.give(pbd_core::inventory::Item::Block(light), 8);
            }
            **slots = carried;
            *done = true;
        }
    }
    if !launch.place_row.is_empty() {
        let forward = transform.forward().as_vec3();
        let placed = block_row(
            &mut Edited {
                fine: &mut fine,
                contact: &mut contact,
                save: &mut edits,
                slots: &mut slots,
                drops: &mut drops,
                now_s: sun.clock.seconds,
            },
            eye,
            forward,
            launch.lamps_at,
            &launch.place_row,
        );
        // Before the fine set is under the camera, the next frame tries again.
        if placed > 0 {
            info!("scripted blocks: {placed} placed");
            *done = true;
        }
        return;
    }
    if dug == 0 && !launch.torch && !launch.lamps {
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

#[cfg(test)]
mod tree_tests {
    /// A tree stands until the axe has taken all its logs, and its ground
    /// holds until then (`tool-crafting` decision 3a).
    #[test]
    fn a_tree_stands_until_its_last_log() {
        assert!(!super::tree_felled(0, 4));
        assert!(!super::tree_felled(3, 4));
        assert!(super::tree_felled(4, 4));
    }
}

/// `placed-blocks-keep-material`: what a placed block gives back when it is
/// dug, wherever it was put down.
#[cfg(test)]
mod placed_tests {
    use super::*;
    use pbd_app::planet::terrain_config;
    use pbd_core::edits::Edit;
    use pbd_core::inventory::{Item, Slots};
    use pbd_core::planet_gen::{self, Biome};

    /// A block placed on desert sand is dug as that block, never as the
    /// sand under it: the column the tier builds from the save is what the
    /// dig reads (`material_of`), and that is what it yields.
    #[test]
    fn a_placed_block_on_desert_sand_digs_as_itself() {
        let cfg = terrain_config();
        let desert = (0..60_000)
            .map(|i| {
                let y = 1.0 - 2.0 * (i as f32 + 0.5) / 60_000.0;
                let r = (1.0 - y * y).sqrt();
                let phi = i as f32 * 2.399_963;
                Vec3::new(r * phi.cos(), y, r * phi.sin())
            })
            .find(|&d| {
                let surface = column::surface_m(cfg, d);
                planet_gen::biome_at(cfg, d, surface) == Biome::Desert
                    && planet_gen::top_material(cfg, d, surface) == Material::Sand
                    && surface > cfg.sea_level_m
            })
            .expect("the shipped world has a desert");
        let under = column::generate_solid(cfg, desert).surface().unwrap();
        for (cell, material) in Material::ALL
            .into_iter()
            .filter(|&m| column::home_biome(m).is_some() && !m.is_water())
            .enumerate()
        {
            let cell = cell as u32 + 1;
            let mut save = WorldSave::memory_only();
            assert!(save.accept(
                Edit {
                    cell,
                    layer: under as u16 + 1,
                    material,
                },
                None,
                &Slots::new()
            ));
            let built = column::generate_edited_solid(cfg, desert, save.edits.for_cell(cell));
            let was = built.material(under + 1);
            assert_eq!(was, material, "{material:?}: stored");
            let expected = match material {
                // Turned soil is dirt again (`farming-and-smelting` decision 2).
                Material::Tilled => Item::Block(Material::Dirt),
                Material::Ore => pbd_core::station::ore_yield(cell, under as u16 + 1, true),
                other => Item::Block(other),
            };
            assert_eq!(
                yield_of(&save, was, cell, under + 1),
                expected,
                "{material:?}"
            );
            // And the sand under it is still the desert's sand.
            assert_eq!(built.material(under), Material::Sand);
        }
    }
}
