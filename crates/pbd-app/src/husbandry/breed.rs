//! Eggs, young and breeding in the app (`animal-husbandry` design 13): the
//! farm's own births and conceptions proposed under the world's author, the
//! fertile eggs collected and set (under a broody hen or in an incubator by a
//! lamp), the incubator piece, and the breeding screen's choices.
//!
//! Every rule is the core's (`pbd_core::husbandry::breeding`, `young`). This
//! reads the world for it: how an animal fares in its pen now (S7), whether a
//! lamp burns beside an incubator, and the hands.

use super::*;
use pbd_core::husbandry::breeding::{self, BroodBody, Fitness, IncubatorBody, Place};
use pbd_core::husbandry::{Birth, Keeping};
use pbd_core::locale::{tr, tr_with};
use pbd_core::records::Proposal;

/// What the breeding screen has chosen: the species tab, the sire and the
/// dam, and whose pedigree is shown. Kept here so a capture's script can set
/// it through the same resource the screen presses write.
#[derive(Resource, Default, Clone, Debug, PartialEq)]
pub struct BreedingPick {
    pub species: u16,
    pub sire: Option<u64>,
    pub dam: Option<u64>,
    /// The animal last pressed: its pedigree is drawn.
    pub pedigree: Option<u64>,
    /// The screen's one-line answer to a press of PAIR.
    pub said: String,
}

/// Note the breeding kinds this world now holds, once (`identity.ron`).
pub(super) fn note_kinds(save: &mut WorldSave) {
    save.note_record_kinds(&breeding::KINDS);
}

/// The species' keeping, by roster index.
pub(super) fn keeping(world: &World, sp: u16) -> Option<Keeping> {
    world.roster().get(sp as usize)?.keep.clone()
}

/// The day a kept animal may give products from (E1, E4): adulthood, and
/// for a hen the hatching of the last clutch she sat on.
pub(super) fn productive_from(
    records: &pbd_core::records::Records,
    id: u64,
    body: &AnimalBody,
    keep: &Keeping,
) -> f64 {
    let adult = body.born + f64::from(keep.young.adult_at());
    let sitting = breeding::sitting_until(records, id, &keep.young).unwrap_or(f64::NEG_INFINITY);
    adult.max(sitting)
}

/// Whether a hen of the record's is sitting now: a clutch under her that has
/// not hatched.
pub(super) fn sitting(records: &pbd_core::records::Records, id: u64) -> bool {
    breeding::eggs_at(records, Place::Hen(id)) > 0
}

/// Whether a lit lamp stands within `lamp_m` of an incubator on `cell`: any
/// emitting cell of the resident tier's last bake (`Material::is_lamp`).
pub(super) fn lamp_near(world: &World, cell: u32) -> bool {
    let Some(fine) = world.fine.as_deref() else {
        return false;
    };
    let Some(d) = pbd_core::cell_key::key_direction(cell) else {
        return false;
    };
    let at = d * (world.ground(d) + 0.3);
    let reach = world.settings().breeding.lamp_m;
    fine.set
        .columns
        .lamps()
        .iter()
        .any(|l| (l.direction * (PLANET_RADIUS + l.altitude)).distance(at) <= reach)
}

/// How each kept animal near the player fares now (S7), by id. One far away
/// (beyond the herds' reach) is left out: nothing pairs where no one sees it.
fn fitness_now(world: &World, yard: &mut Farmyard, save: &WorldSave) -> BTreeMap<u64, Fitness> {
    let Some(feet_point) = world.feet_point() else {
        return BTreeMap::new();
    };
    let up = feet_point.normalize_or(Vec3::Y);
    let near = world.settings().herds.near_m;
    let n = world.settings().needs;
    let mut out = BTreeMap::new();
    for (id, body) in record::animals(&save.records) {
        if body.state != record::Life::Alive || body.body != world.body.0 {
            continue;
        }
        let Some(home) = pbd_core::cell_key::key_direction(body.home) else {
            continue;
        };
        if home.angle_between(up) * PLANET_RADIUS > near {
            continue;
        }
        let (state, c) = farm_and_conditions(world, yard, save, id, &body);
        out.insert(
            id,
            Fitness {
                mood: state.mood(c, world.day(), &n),
                health: state.health,
            },
        );
    }
    out
}

/// The farm's own events up to now, proposed under the world's author (E5,
/// E6): births and conceptions. Run with the yard's sync, twice a second
/// while the player is near, and on opening the world.
pub(super) fn realise(world: &World, yard: &mut Farmyard, save: &mut WorldSave) -> usize {
    if record::animals(&save.records).is_empty() {
        return 0;
    }
    let fit = fitness_now(world, yard, save);
    // A young born during a long absence fares as its dam does (S7).
    let fares = |id: u64, a: &AnimalBody| -> Fitness {
        fit.get(&id)
            .or_else(|| a.dam.and_then(|d| fit.get(&d)))
            .copied()
            .unwrap_or(Fitness {
                mood: 0.0,
                health: 0.0,
            })
    };
    let lit: BTreeMap<u32, bool> = breeding::incubators(&save.records)
        .into_iter()
        .map(|cell| (cell, lamp_near(world, cell)))
        .collect();
    let lamp = |cell: u32| lit.get(&cell).copied().unwrap_or(false);
    let events = breeding::farm_events(
        &save.records,
        world.roster(),
        &world.settings().breeding,
        world.day(),
        &fares,
        &lamp,
    );
    let mut born = 0;
    for event in events {
        let author = Author::World {
            process: "husbandry".into(),
            kind: event.author.0.into(),
            id: event.author.1,
        };
        let young = event
            .records
            .iter()
            .filter(|r| r.kind == record::ANIMAL)
            .count();
        let at = event.day;
        match save.propose(
            &author,
            Proposal {
                edits: Vec::new(),
                records: event.records,
            },
        ) {
            Ok(_) => {
                note_kinds(save);
                super::note_kinds(save);
                born += young;
                if young > 0 {
                    yard.say(
                        if young == 1 {
                            tr("breed.young_one_was_born").to_string()
                        } else {
                            tr_with("breed.young_were_born", &[("young", &young)]).to_string()
                        },
                        3.0,
                    );
                }
                info!("husbandry: {} on day {at:.2}: {young} born", author.token());
            }
            Err(why) => warn!("husbandry: {} refused: {why:?}", author.token()),
        }
    }
    born
}

/// The hen's eggs collected: fertile with an adult cockerel in her pen. The
/// parentage is a carried `brood`, stored as the player's ahead of the hands
/// that hold the eggs (E3).
pub(super) fn collected_eggs(
    world: &World,
    save: &mut WorldSave,
    hen_id: u64,
    hen: &AnimalBody,
    keep: &Keeping,
    count: u32,
) -> Good {
    let day = world.day();
    let Some(sire) = breeding::cockerel_for(&save.records, hen, keep, day) else {
        return Good::EGG;
    };
    if !matches!(keep.young.birth, Birth::Eggs { .. }) {
        return Good::EGG;
    }
    let brood = BroodBody {
        body: hen.body.clone(),
        sp: hen.sp,
        dam: hen_id,
        sire,
        eggs: count.min(255) as u8,
        laid: day,
        set: None,
        place: Place::Carried,
    };
    let id = breeding::next_brood_id(&save.records);
    note_kinds(save);
    if save
        .store(&Author::Player(0), vec![brood.record(id)])
        .is_none()
    {
        return Good::EGG;
    }
    Good::FERTILE_EGG
}

/// Set up to `room` of the fertile eggs in hand at `place`: parentage from
/// the oldest warm carried clutches, the carried ones written down by what
/// they gave, and the eggs taken from the hand. The slots go with the last
/// record. How many were set, or why none were.
fn set_eggs(
    world: &World,
    hotbar: &mut Slots,
    save: &mut WorldSave,
    place: Place,
    room: u32,
    last: Option<pbd_core::records::Record>,
) -> Result<u32, &'static str> {
    let held = hotbar
        .held()
        .filter(|s| s.item == Item::Good(Good::FERTILE_EGG))
        .map_or(0, |s| u32::from(s.count));
    if held == 0 {
        return Err("hold fertile eggs");
    }
    let want = held.min(room);
    if want == 0 {
        return Err("there is no room for more eggs");
    }
    let day = world.day();
    let (parts, left) =
        breeding::take_parentage(&save.records, want, day, &world.settings().breeding);
    let set = want - left;
    if set == 0 {
        return Err("these eggs have gone cold");
    }
    let first = breeding::next_brood_id(&save.records);
    let mut records = Vec::new();
    for (next, (id, carried, give)) in (first..).zip(parts) {
        let mut rest = carried.clone();
        rest.eggs -= give;
        records.push(rest.record(id));
        let brood = BroodBody {
            eggs: give,
            set: Some(day),
            place,
            ..carried
        };
        records.push(brood.record(next));
    }
    let at = hotbar.selected();
    hotbar.take(at, set as u16);
    note_kinds(save);
    let with_hands = match last {
        Some(r) => r,
        None => records.pop().expect("a brood was set"),
    };
    if !records.is_empty() && save.store(&Author::Player(0), records).is_none() {
        return Err("the save refused it");
    }
    let schema = if with_hands.kind == record::ANIMAL {
        record::ANIMAL_SCHEMA
    } else {
        breeding::BROOD_SCHEMA
    };
    if !save.store_with_hands(with_hands, schema, hotbar) {
        return Err("the save refused it");
    }
    Ok(set)
}

/// F on a hen of yours with fertile eggs in hand: she goes broody on up to
/// a clutch (E4). Whether the press was hers.
pub(super) fn set_under_hen(
    world: &World,
    yard: &mut Farmyard,
    hotbar: &mut Slots,
    save: &mut WorldSave,
    id: u64,
    body: &mut AnimalBody,
) -> bool {
    let Some(keep) = keeping(world, body.sp) else {
        return false;
    };
    let Birth::Eggs { clutch_max, .. } = keep.young.birth else {
        return false;
    };
    let day = world.day();
    if body.sex != Sex::F {
        yard.say(tr("breed.only_hen_sits_eggs"), 2.0);
        return true;
    }
    if day < body.born + f64::from(keep.young.adult_at()) {
        yard.say(tr("breed.she_too_young_sit"), 2.0);
        return true;
    }
    if sitting(&save.records, id) {
        yard.say(tr("breed.she_already_sitting"), 2.0);
        return true;
    }
    // Her tally re-based up to now, so the clutch costs only what she would
    // lay while she sits.
    let now = farm_now(world, yard, save, id, body);
    body.set_farm(&now);
    let hen = body.record(id);
    match set_eggs(
        world,
        hotbar,
        save,
        Place::Hen(id),
        u32::from(clutch_max),
        Some(hen),
    ) {
        Ok(n) => {
            let days = match keep.young.birth {
                Birth::Eggs {
                    incubation_days, ..
                } => incubation_days,
                Birth::Live { .. } => 0.0,
            };
            yard.say(
                tr_with(
                    "breed.broody",
                    &[("n", &n), ("days", &format!("{days:.0}"))],
                ),
                3.0,
            );
        }
        Err(why) => yard.say(why, 2.0),
    }
    true
}

/// The incubator standing on the aimed cell, if any.
pub(super) fn incubator_at(world: &World, save: &WorldSave, point: Vec3) -> Option<u32> {
    let (_, cell) = world.cell(point)?;
    breeding::incubators(&save.records)
        .contains(&cell)
        .then_some(cell)
}

/// Right click with an incubator in hand: stand it on the aimed cell.
pub(super) fn place_incubator(
    world: &World,
    yard: &mut Farmyard,
    hotbar: &mut Slots,
    save: &mut WorldSave,
    point: Vec3,
) -> bool {
    let Some(feet) = world.feet_point() else {
        return false;
    };
    if point.distance(feet) > world.settings().taming.reach_m * 2.5 {
        yard.say(tr("breed.too_far_set"), 1.5);
        return false;
    }
    let Some((_, cell)) = world.cell(point) else {
        return false;
    };
    if breeding::incubators(&save.records).contains(&cell) {
        yard.say(tr("breed.incubator_already_stands_there"), 1.5);
        return false;
    }
    let piece = IncubatorBody {
        state: Standing::Placed,
    };
    if !spend_one(hotbar)
        || !save.store_with_hands(piece.record(cell), breeding::INCUBATOR_SCHEMA, hotbar)
    {
        return false;
    }
    yard.fences_drawn = None;
    yard.say(
        if lamp_near(world, cell) {
            tr("breed.incubator_set_beside_lamp")
        } else {
            tr("breed.incubator_set_needs_lamp_beside")
        },
        2.0,
    );
    true
}

/// F on an incubator: set fertile eggs in it, or with empty hands take it
/// up while it holds none. Whether there was one.
pub(super) fn use_incubator(
    world: &World,
    yard: &mut Farmyard,
    hotbar: &mut Slots,
    save: &mut WorldSave,
    point: Vec3,
) -> bool {
    let Some(cell) = incubator_at(world, save, point) else {
        return false;
    };
    let holds = breeding::eggs_at(&save.records, Place::Incubator(cell));
    let fertile = hotbar
        .held()
        .is_some_and(|s| s.item == Item::Good(Good::FERTILE_EGG));
    if fertile {
        if !lamp_near(world, cell) {
            yard.say(tr("breed.needs_lamp_beside"), 2.0);
            return true;
        }
        let room = u32::from(world.settings().breeding.incubator_eggs).saturating_sub(holds);
        match set_eggs(world, hotbar, save, Place::Incubator(cell), room, None) {
            Ok(n) => yard.say(tr_with("breed.eggs_set_incubator", &[("n", &n)]), 2.5),
            Err(why) => yard.say(why, 2.0),
        }
        return true;
    }
    if holds > 0 {
        yard.say(
            tr_with("breed.incubator_holds_eggs", &[("holds", &holds)]),
            2.0,
        );
        return true;
    }
    if hotbar.held().is_some() {
        return false;
    }
    if hotbar.give(Item::Good(Good::INCUBATOR), 1) > 0 {
        yard.say(tr("breed.no_room_carry"), 1.5);
        return true;
    }
    let piece = IncubatorBody {
        state: Standing::Removed,
    };
    save.store_with_hands(piece.record(cell), breeding::INCUBATOR_SCHEMA, hotbar);
    yard.fences_drawn = None;
    yard.say(tr("breed.incubator_taken_up"), 1.0);
    true
}

/// What the HUD says of an aimed incubator.
pub(super) fn describe_incubator(world: &World, save: &WorldSave, point: Vec3) -> Option<String> {
    let cell = incubator_at(world, save, point)?;
    let holds = breeding::eggs_at(&save.records, Place::Incubator(cell));
    let lamp = lamp_near(world, cell);
    let young = breeding::broods(&save.records);
    let due = young
        .values()
        .filter(|b| b.place == Place::Incubator(cell))
        .filter_map(|b| {
            let keep = keeping(world, b.sp)?;
            b.due(&keep.young)
        })
        .filter(|d| *d > world.day())
        .reduce(f64::min);
    Some(match (holds, lamp, due) {
        (0, false, _) => tr("breed.incubator_needs_lamp_beside").to_string(),
        (0, true, _) => tr("breed.incubator_set_fertile_eggs").to_string(),
        (n, false, _) => tr_with("breed.incubator_eggs_lamp_out", &[("n", &n)]).to_string(),
        (n, true, Some(d)) => tr_with(
            "breed.incubator_hatching",
            &[("n", &n), ("days", &format!("{:.1}", d - world.day()))],
        )
        .to_string(),
        (n, true, None) => tr_with("breed.incubator_eggs", &[("n", &n)]).to_string(),
    })
}

/// The incubators, drawn as small crates of plank on timber legs with a
/// flat glass lid (design 14), into the pen's meshes.
pub(super) fn draw_incubators(
    world: &World,
    save: &WorldSave,
    pen: &mut PenMeshes,
    origin: &mut Option<Vec3>,
) -> usize {
    let mut n = 0;
    for cell in breeding::incubators(&save.records) {
        let Some(d) = pbd_core::cell_key::key_direction(cell) else {
            continue;
        };
        let at = d * world.ground(d);
        let o = *origin.get_or_insert(at);
        let up = d;
        let side = Vec3::Y.cross(up).normalize_or(Vec3::X);
        let fwd = up.cross(side);
        let base = at - o;
        pen.plank.textured_cuboid(
            base + up * 0.18,
            [side * 0.30, up * 0.18, fwd * 0.22],
            pen.repeat[3],
        );
        pen.glass.cuboid(
            base + up * 0.39,
            [side * 0.28, up * 0.03, fwd * 0.20],
            [0.66, 0.85, 0.91],
        );
        for (x, z) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
            pen.timber.textured_cuboid(
                base + up * 0.2 + side * (0.29 * x) + fwd * (0.21 * z),
                [side * 0.025, up * 0.2, fwd * 0.025],
                pen.repeat[0],
            );
        }
        n += 1;
    }
    n
}

/// The young's stage name and size against the adult's, now.
pub(super) fn stage_now(world: &World, sp: u16, sex: Sex, born: f64) -> Option<(String, f32)> {
    let keep = keeping(world, sp)?;
    let (name, size) = keep.young.stage(sex, born, world.day());
    Some((name.to_string(), size))
}

/// PAIR on the breeding screen: the chosen sire written on the dam as hers
/// (`mate`, the player's field, E6). What the screen says.
pub fn pair(save: &mut WorldSave, dam: u64, sire: u64) -> String {
    let Some(mut body) = save
        .records
        .get(record::ANIMAL, dam)
        .and_then(|r| r.read::<AnimalBody>())
    else {
        return tr("breed.no_such_dam").into();
    };
    if body.mate == Some(sire) {
        return tr("breed.already_paired").into();
    }
    body.mate = Some(sire);
    match save.store(&Author::Player(0), vec![body.record(dam)]) {
        Some(_) => tr("breed.paired_they_will_mate_next").into(),
        None => tr("breed.save_refused").into(),
    }
}

/// A capture's liberty (`--husbandry eggs`): every kept animal and brood
/// moved `days` back in world time, as if kept that much longer. A
/// measurement instrument, as `collect`'s re-base is.
pub(super) fn rebase(save: &mut WorldSave, days: f64) {
    let mut records = Vec::new();
    for (id, mut body) in record::animals(&save.records) {
        body.as_of -= days;
        body.fed_until -= days;
        body.born -= days;
        records.push(body.record(id));
    }
    for (id, mut brood) in breeding::broods(&save.records) {
        brood.laid -= days;
        brood.set = brood.set.map(|s| s - days);
        records.push(brood.record(id));
    }
    save.store(&Author::Player(0), records);
}

/// A capture's liberty (`--husbandry breed`): the farm moved `days` back and
/// its own events run up to now as if every need were met, through the same
/// rule and the same proposals as play. How many were born.
pub(super) fn run_content(
    world: &World,
    yard: &mut Farmyard,
    save: &mut WorldSave,
    days: f64,
) -> usize {
    rebase(save, days);
    let content = |_: u64, _: &AnimalBody| Fitness {
        mood: 100.0,
        health: 100.0,
    };
    let events = breeding::farm_events(
        &save.records,
        world.roster(),
        &world.settings().breeding,
        world.day(),
        &content,
        &|_| true,
    );
    let mut born = 0;
    for event in events {
        let author = Author::World {
            process: "husbandry".into(),
            kind: event.author.0.into(),
            id: event.author.1,
        };
        born += event
            .records
            .iter()
            .filter(|r| r.kind == record::ANIMAL)
            .count();
        if let Err(why) = save.propose(
            &author,
            Proposal {
                edits: Vec::new(),
                records: event.records,
            },
        ) {
            warn!("husbandry: {} refused: {why:?}", author.token());
        }
    }
    note_kinds(save);
    super::note_kinds(save);
    yard.say(tr_with("breed.born", &[("born", &born)]), 1.0);
    born
}

/// A capture's pick for the breeding screen: a ram and a ewe of the herd who
/// share a parent (so the screen shows close kin), else the first of each.
pub(super) fn pick_kin(save: &WorldSave, sp: u16) -> BreedingPick {
    let all: Vec<(u64, AnimalBody)> = record::animals(&save.records)
        .into_iter()
        .filter(|(_, b)| b.sp == sp && b.state == record::Life::Alive)
        .collect();
    let kin = all
        .iter()
        .filter(|(_, b)| b.sex == Sex::M)
        .find_map(|(m, mb)| {
            all.iter()
                .filter(|(_, b)| b.sex == Sex::F)
                .find(|(_, fb)| {
                    mb.sire.is_some()
                        && (fb.sire == mb.sire || fb.dam.is_some() && fb.dam == mb.dam)
                })
                .map(|(f, _)| (*m, *f))
        });
    let first = |sex: Sex| all.iter().find(|(_, b)| b.sex == sex).map(|(id, _)| *id);
    let (sire, dam) = kin.map_or((first(Sex::M), first(Sex::F)), |(m, f)| (Some(m), Some(f)));
    BreedingPick {
        species: sp,
        sire,
        dam,
        pedigree: dam,
        said: String::new(),
    }
}
