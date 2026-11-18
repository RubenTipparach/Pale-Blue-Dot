//! Picking the trees' fruit (`flora-v2` design 12.5): F on a fruiting tree
//! takes what hangs on it. Which tree bears what, and how much the season
//! keeps, is the tree draw's own rule (`pbd_core::flora::hanging_fruit`);
//! what was taken is a `fruit` record by the tree's cell, written with the
//! hands in one line, and the draw leaves a picked tree's hanging fruit out
//! until it grows back (`trees_taken`).

use super::farm::FarmNote;
use super::menu::Screen;
use bevy::prelude::*;
use pbd_app::hotbar::{Gained, How};
use pbd_app::interact::{Act, Interact, Offer, Reader};
use pbd_app::planet::{PLANET_RADIUS, PlanetFine};
use pbd_app::saves::WorldSave;
use pbd_core::daylight::DAY_S;
use pbd_core::flora::{
    FRUIT_GOODS, FRUIT_RECORD, FRUIT_REGROW_DAYS, FRUIT_SCHEMA, FruitPicked, hanging_fruit,
};
use pbd_core::gather::{TREE_KIND, TreeTaken};
use pbd_core::goods::Good;
use pbd_core::inventory::Item;
use pbd_core::records::Record;

/// The fruiting tree the eye is on, as it stands.
struct OnTree {
    cell: u32,
    /// Its fruit, an index of `FRUIT_GOODS`.
    kind: usize,
    /// How many hang on it this season.
    hanging: u32,
    felled: bool,
    /// Picked bare: the days until it bears again.
    bare_for: Option<f64>,
    day: f64,
}

impl OnTree {
    /// Whether F takes something from it.
    fn has_fruit(&self) -> bool {
        !self.felled && self.bare_for.is_none() && self.hanging > 0
    }
}

/// The fruiting tree whose hanging fruit the eye ray meets nearest within
/// reach, by its cell (`fruit-aim`): the trunk and the leaves are not the
/// fruit.
fn fruit_on_ray(
    fine: &PlanetFine,
    scatter: &pbd_app::config::ScatterSettings,
    year: f64,
    eye: Vec3,
    look: Vec3,
) -> Option<u32> {
    let reach = pbd_core::aim::REACH_M;
    let toward = eye.normalize_or(Vec3::Y);
    let near = ((reach + 8.0) / PLANET_RADIUS).cos();
    let table = &pbd_app::planet::terrain_config().trees;
    let mut best: Option<(f32, u32)> = None;
    for record in fine.set.finest_records() {
        let Some(up) = Vec3::from_slice(&record.direction_height[..3]).try_normalize() else {
            continue;
        };
        if up.dot(toward) < near
            || !pbd_core::flora::grows_tree(table, record.spare[0], record.metadata[1])
        {
            continue;
        }
        let biome = (record.metadata[1] >> 8) & 0xff;
        let points = pbd_core::flora::hanging_fruit_points(
            record.spare[0],
            biome,
            up.to_array(),
            PLANET_RADIUS,
            record.direction_height[3],
            year,
            scatter.fruit_chance,
        );
        for (centre, r) in points {
            // The ray against the fruit's sphere: the nearest crossing ahead.
            let to = Vec3::from(centre) - eye;
            let along = to.dot(look);
            let miss = to.length_squared() - along * along;
            if along < 0.0 || miss > r * r {
                continue;
            }
            let t = along - (r * r - miss).sqrt();
            if t <= reach && best.is_none_or(|(bt, _)| t < bt) {
                best = Some((t, record.key()));
            }
        }
    }
    best.map(|(_, key)| key)
}

/// The fruiting tree on the eye ray, if one bears anything.
#[allow(clippy::too_many_arguments)]
fn on_tree(
    fine: &PlanetFine,
    save: &WorldSave,
    scatter: &pbd_app::config::ScatterSettings,
    gather: &pbd_app::config::GatherConfig,
    sun: Option<&pbd_app::sky::Sun>,
    eye: Vec3,
    look: Vec3,
) -> Option<OnTree> {
    let year = sun.map_or(0.0, |s| s.clock.year_fraction());
    let cell = fruit_on_ray(fine, scatter, year, eye, look)?;
    let record = fine.set.finest_records().iter().find(|r| r.key() == cell)?;
    let day = sun.map_or(0.0, |s| s.clock.seconds / f64::from(DAY_S));
    let axis = Vec3::from_slice(&record.direction_height[..3]).normalize_or(Vec3::Y);
    let biome = (record.metadata[1] >> 8) & 0xff;
    let (kind, hanging) = hanging_fruit(
        record.spare[0],
        biome,
        axis.to_array(),
        PLANET_RADIUS,
        year,
        scatter.fruit_chance,
    )?;
    let felled = save
        .records
        .get(TREE_KIND, u64::from(cell))
        .and_then(|r| r.read::<TreeTaken>())
        .is_some_and(|t| t.logs >= gather.0.axe_logs);
    let bare_for = save
        .records
        .get(FRUIT_RECORD, u64::from(cell))
        .and_then(|r| r.read::<FruitPicked>())
        .filter(|p| p.bare_at(day))
        .map(|p| (p.day + FRUIT_REGROW_DAYS - day).max(0.0));
    Some(OnTree {
        cell,
        kind,
        hanging,
        felled,
        bare_for,
        day,
    })
}

/// A fruit's name as the prompt says it, many: "apples", "cherries".
pub fn many(name: &str) -> String {
    let name = name.to_lowercase();
    if let Some(stem) = name.strip_suffix('y')
        && !stem.ends_with(['a', 'e', 'i', 'o', 'u'])
    {
        return format!("{stem}ies");
    }
    if name.ends_with('o') {
        return format!("{name}es");
    }
    format!("{name}s")
}

/// Offer the fruit on the tree aimed at to the interact key.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn offer(
    walking: Option<Res<pbd_app::walking::WalkingReadout>>,
    (cameras, frame): (
        Query<(&GlobalTransform, &Camera), With<Camera3d>>,
        Option<Res<pbd_app::planet::PlanetRenderFrame>>,
    ),
    fine: Option<Res<PlanetFine>>,
    save: Option<Res<WorldSave>>,
    sun: Option<Res<pbd_app::sky::Sun>>,
    scatter: Option<Res<pbd_app::config::ScatterSettings>>,
    gather: Option<Res<pbd_app::config::GatherConfig>>,
    time: Res<Time>,
    mut interact: ResMut<Interact>,
    mut last: Local<(pbd_app::interact::Recheck, Option<Offer>)>,
) {
    let (Some(fine), Some(save), Some(scatter), Some(gather)) = (fine, save, scatter, gather)
    else {
        return;
    };
    if !walking.is_some_and(|w| w.active) {
        return;
    }
    let Some((transform, _)) = cameras.iter().find(|(_, c)| c.is_active) else {
        return;
    };
    let (eye, look) = (
        pbd_app::render_origin::local_of(frame.as_deref(), transform.translation()),
        transform.forward().as_vec3(),
    );
    let (recheck, offered) = &mut *last;
    if recheck.due(eye, look, fine.version, time.delta_secs()) {
        *offered = on_tree(&fine, &save, &scatter, &gather, sun.as_deref(), eye, look)
            .filter(OnTree::has_fruit)
            .and_then(|tree| Good::by_code(FRUIT_GOODS[tree.kind]))
            .map(|good| Offer::new(Act::PickFruit, many(good.row().name)));
    }
    if let Some(offer) = offered.clone() {
        interact.offer(offer);
    }
}

/// F on a tree: pick its fruit. After the husbandry hands, which take an F
/// they used, and before the farm's, so an F used here is not a crop's.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn pick(
    mut keys: ResMut<ButtonInput<KeyCode>>,
    screen: Res<Screen>,
    walking: Option<Res<pbd_app::walking::WalkingReadout>>,
    (cameras, frame): (
        Query<(&GlobalTransform, &Camera), With<Camera3d>>,
        Option<Res<pbd_app::planet::PlanetRenderFrame>>,
    ),
    fine: Option<Res<PlanetFine>>,
    sun: Option<Res<pbd_app::sky::Sun>>,
    scatter: Option<Res<pbd_app::config::ScatterSettings>>,
    gather: Option<Res<pbd_app::config::GatherConfig>>,
    time: Res<Time>,
    (mut save, slots, note, mut gained, interact): (
        Option<ResMut<WorldSave>>,
        Option<ResMut<super::slots::Hotbar>>,
        Option<ResMut<FarmNote>>,
        MessageWriter<Gained>,
        Option<Res<Interact>>,
    ),
) {
    // A still capture has no walker, hands or gathering: nothing to pick.
    let (Some(scatter), Some(gather), Some(mut slots), Some(mut note)) =
        (scatter, gather, slots, note)
    else {
        return;
    };
    if !keys.just_pressed(KeyCode::KeyF) || *screen != Screen::Playing {
        return;
    }
    if !walking.is_some_and(|w| w.active && w.captured) {
        return;
    }
    if !pbd_app::interact::may(interact.as_deref(), Reader::Fruit) {
        return;
    }
    let (Some(fine), Some(save)) = (fine, save.as_deref_mut()) else {
        return;
    };
    let Some((transform, _)) = cameras.iter().find(|(_, c)| c.is_active) else {
        return;
    };
    let (eye, look) = (
        pbd_app::render_origin::local_of(frame.as_deref(), transform.translation()),
        transform.forward().as_vec3(),
    );
    let Some(tree) = on_tree(&fine, save, &scatter, &gather, sun.as_deref(), eye, look) else {
        // A tree that bears nothing: the F is someone else's.
        return;
    };
    keys.clear_just_pressed(KeyCode::KeyF);
    if tree.felled {
        return;
    }
    let wall_s = time.elapsed_secs();
    let (cell, kind, hanging, day) = (tree.cell, tree.kind, tree.hanging, tree.day);
    if let Some(left) = tree.bare_for {
        note.say(wall_s, format!("No fruit. Grows back in {left:.0} days"));
        return;
    }
    let Some(good) = Good::by_code(FRUIT_GOODS[kind]) else {
        return;
    };
    if hanging == 0 {
        note.say(wall_s, "No ripe fruit");
        return;
    }
    let item = Item::Good(good);
    let mut moved = slots.0.clone();
    let left = moved.give(item, hanging as u16);
    let took = hanging as u16 - left;
    if took == 0 {
        note.say(wall_s, "No room in the pack");
        return;
    }
    let body = FruitPicked { day, took };
    let line = Record::of(FRUIT_RECORD, u64::from(cell), FRUIT_SCHEMA, &body);
    if !save.store_with_hands(line, FRUIT_SCHEMA, &moved) {
        error!("fruit BLOCKED: the save did not accept it");
        return;
    }
    slots.0 = moved;
    gained.write(Gained {
        item,
        count: took,
        how: How::Picked,
    });
    info!(
        "picked {took} {} from the tree at cell {cell} on day {day:.2}",
        FRUIT_GOODS[kind]
    );
}

/// `--take-trees`: a capture's instrument (`flora-v2` 12.5). Once the fine set
/// stands round the camera, the three nearest fruiting trees in front of it
/// are marked picked today and the next twelve trees felled, as `fruit` and
/// `tree` records stored as the player's (the records a player's hands
/// write; the instrument's liberty is not walking to each). The draw then
/// leaves their fruit and the felled trees out.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn scripted(
    launch: Res<super::Launch>,
    (cameras, frame): (
        Query<(&GlobalTransform, &Camera), With<Camera3d>>,
        Option<Res<pbd_app::planet::PlanetRenderFrame>>,
    ),
    fine: Option<Res<PlanetFine>>,
    mut save: Option<ResMut<WorldSave>>,
    sun: Option<Res<pbd_app::sky::Sun>>,
    scatter: Option<Res<pbd_app::config::ScatterSettings>>,
    gather: Option<Res<pbd_app::config::GatherConfig>>,
    mut done: Local<u32>,
) {
    if !launch.take_trees || *done > 30 {
        return;
    }
    let (Some(scatter), Some(gather)) = (scatter, gather) else {
        return;
    };
    *done += 1;
    if *done < 30 {
        // A few frames for the fine set to land round the camera.
        return;
    }
    let (Some(fine), Some(save)) = (fine, save.as_deref_mut()) else {
        return;
    };
    let Some((transform, _)) = cameras.iter().find(|(_, c)| c.is_active) else {
        return;
    };
    let (eye, look) = (
        pbd_app::render_origin::local_of(frame.as_deref(), transform.translation()),
        transform.forward().as_vec3(),
    );
    let table = pbd_app::planet::terrain_config().trees;
    let day = sun
        .as_ref()
        .map_or(0.0, |s| s.clock.seconds / f64::from(DAY_S));
    let year = sun.as_ref().map_or(0.0, |s| s.clock.year_fraction());
    let mut trees: Vec<(f32, u32, bool)> = fine
        .set
        .finest_records()
        .iter()
        .filter(|r| pbd_core::flora::grows_tree(&table, r.spare[0], r.metadata[1]))
        .filter_map(|r| {
            let d = Vec3::from_slice(&r.direction_height[..3]).normalize_or(Vec3::Y);
            let at = d * (PLANET_RADIUS + r.direction_height[3]);
            let to = at - eye;
            let ahead = to.normalize_or(Vec3::ZERO).dot(look);
            let fruiting = hanging_fruit(
                r.spare[0],
                (r.metadata[1] >> 8) & 0xff,
                d.to_array(),
                PLANET_RADIUS,
                year,
                scatter.fruit_chance,
            )
            .is_some_and(|(_, n)| n > 0);
            (ahead > 0.5 && to.length() < 40.0).then_some((to.length(), r.key(), fruiting))
        })
        .collect();
    trees.sort_by(|a, b| a.0.total_cmp(&b.0));
    let picked: Vec<u32> = trees.iter().filter(|t| t.2).take(3).map(|t| t.1).collect();
    let felled_at: Vec<(f32, u32)> = trees
        .iter()
        .filter(|t| !picked.contains(&t.1))
        .take(12)
        .map(|t| (t.0, t.1))
        .collect();
    let felled: Vec<u32> = felled_at.iter().map(|t| t.1).collect();
    info!(
        "TAKE_TREES felled at {:?} m",
        felled_at.iter().map(|t| t.0.round()).collect::<Vec<_>>()
    );
    let mut records = Vec::new();
    for cell in &picked {
        let body = FruitPicked { day, took: 4 };
        records.push(Record::of(
            FRUIT_RECORD,
            u64::from(*cell),
            FRUIT_SCHEMA,
            &body,
        ));
    }
    for cell in &felled {
        let body = TreeTaken {
            logs: gather.0.axe_logs,
        };
        records.push(Record::of(
            TREE_KIND,
            u64::from(*cell),
            pbd_core::gather::TREE_SCHEMA,
            &body,
        ));
    }
    save.note_record_kinds(&[
        (FRUIT_RECORD, FRUIT_SCHEMA),
        (TREE_KIND, pbd_core::gather::TREE_SCHEMA),
    ]);
    save.store(&pbd_core::records::Author::Player(0), records);
    info!(
        "TAKE_TREES picked {picked:?} felled {felled:?} of {} trees in view",
        trees.len()
    );
}
