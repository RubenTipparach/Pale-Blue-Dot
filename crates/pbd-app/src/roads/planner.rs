//! The road planner in the game (`townsfolk` design 42 to 42l): with survey
//! stakes in hand, a click on the ground starts a road, the live ghost
//! follows the aim from it with its red reasons, and the next click ends it,
//! staked every `max_segment` cells; then the planner is idle. A click on a
//! stake of any road starts a new road from it; shift and a click picks a
//! stake up to move it; the eraser takes a whole road. X held opens the
//! planner menu (width, surface, lamps, the eraser).
//!
//! Every edit goes through [`super::paving::commit`]: the plan records and
//! the pack in one durable line.

use super::paving::{self, EditRefused, PlannerSettings};
use super::{RoadNet, RoadSay, RoadsSettings, RoadsStanding};
use crate::saves::WorldSave;
use bevy::prelude::*;
use pbd_core::goods::Good;
use pbd_core::inventory::Item;
use pbd_core::locale::{Localized, tr, tr_with};
use pbd_core::missions::Kind;
use pbd_core::roads::planner::{self, Bill, Clash, How, LaidCell, PlanRoad};
use pbd_core::roads::{self, Surface};

/// The planner's state: the start of a road with no end yet, a stake picked
/// up, the eraser, and how the next road is built.
#[derive(Resource, Default, Clone, Debug, PartialEq)]
pub struct Planner {
    pub start: Option<u32>,
    pub grabbed: Option<u32>,
    pub eraser: bool,
    pub how: How,
}

impl Planner {
    /// Drop a road with no end yet and a stake picked up (design 42k).
    pub fn drop_open(&mut self) {
        self.start = None;
        self.grabbed = None;
    }
}

/// The planner menu's rows (design 42c, 42l).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Row {
    Width(u8),
    Surface(Surface),
    Lamps,
    Eraser,
}

/// The rows in order.
pub const ROWS: [Row; 7] = [
    Row::Width(1),
    Row::Width(2),
    Row::Width(3),
    Row::Surface(Surface::Dirt),
    Row::Surface(Surface::Gravel),
    Row::Lamps,
    Row::Eraser,
];

impl Row {
    /// Its label.
    pub fn label(self) -> Localized {
        match self {
            Row::Width(1) => tr("roads.menu.width_1"),
            Row::Width(2) => tr("roads.menu.width_2"),
            Row::Width(_) => tr("roads.menu.width_3"),
            Row::Surface(Surface::Dirt) => tr("roads.menu.dirt"),
            Row::Surface(Surface::Gravel) => tr("roads.menu.gravel"),
            Row::Lamps => tr("roads.menu.lamps"),
            Row::Eraser => tr("roads.menu.eraser"),
        }
    }

    /// Whether it is the planner's current choice.
    pub fn chosen(self, p: &Planner) -> bool {
        match self {
            Row::Width(w) => p.how.width == w,
            Row::Surface(s) => p.how.surface == s,
            Row::Lamps => p.how.lamps,
            Row::Eraser => p.eraser,
        }
    }

    /// Take it (letting X go on it).
    pub fn take(self, p: &mut Planner) {
        match self {
            Row::Width(w) => p.how.width = w,
            Row::Surface(s) => p.how.surface = s,
            Row::Lamps => p.how.lamps = !p.how.lamps,
            Row::Eraser => {
                p.eraser = !p.eraser;
                p.drop_open();
            }
        }
    }
}

/// The planner menu while X is held: open, its highlight, and the wheel's
/// steps the hotbar hands it.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct PlannerMenu {
    pub open: bool,
    pub highlight: usize,
    pub step: i32,
}

/// What the aim shows: the road the next click would make (or the roads a
/// picked-up stake drags), its first clash, and the bill.
#[derive(Resource, Default, Clone, Debug)]
pub struct PlannerAim {
    pub showing: bool,
    pub cells: Vec<LaidCell>,
    /// Lamp posts the aim's road would stand and that are not standing yet:
    /// the post's cell and its ground's top, m (design 42o).
    pub lamps: Vec<(u32, i32)>,
    pub clash: Option<Clash>,
    pub bill: Bill,
    /// What the pack holds of the bill's goods: stakes, gravel, planks,
    /// logs and street lanterns.
    pub have: [u32; 5],
    /// Bumped whenever the cells change, for the ghost.
    pub generation: u64,
    /// What it was made for: the aimed cell, the planner, the save's serial.
    made_for: Option<(Option<u32>, Planner, u64)>,
}

fn stakes_held(hotbar: &crate::hotbar::Hotbar) -> bool {
    hotbar.0.held().map(|s| s.item) == Some(Item::Good(Good::SURVEY_STAKES))
}

/// Whether a tree still stands on cell `key` (design 42b): its finest record
/// grows one and the axe has not felled it.
pub fn tree_on(
    fine: &crate::planet::PlanetFine,
    contact: &crate::planet::PlanetContact,
    save: &WorldSave,
    axe_logs: u16,
    key: u32,
) -> bool {
    use pbd_core::gather::{TREE_KIND, TreeTaken};
    let Some(c) = roads::centre(key) else {
        return false;
    };
    let Some(record) = contact.finest_cell(c) else {
        return false;
    };
    let Some(r) = fine.set.finest_records().get(record) else {
        return false;
    };
    if r.key() != key
        || !pbd_core::flora::grows_tree(
            &crate::planet::terrain_config().trees,
            r.spare[0],
            r.metadata[1],
        )
    {
        return false;
    }
    let taken = save
        .records
        .get(TREE_KIND, u64::from(key))
        .and_then(|r| r.read::<TreeTaken>())
        .map_or(0, |t| t.logs);
    taken < axe_logs
}

/// X held with stakes in hand opens the planner menu; the wheel moves its
/// highlight, and letting X go takes the highlighted row.
pub fn planner_menu(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    hotbar: Option<Res<crate::hotbar::Hotbar>>,
    walking: Option<Res<crate::walking::WalkingReadout>>,
    menu_open: Option<Res<crate::controls::MenuOpen>>,
    mut menu: ResMut<PlannerMenu>,
    mut planner: ResMut<Planner>,
) {
    let held = hotbar.as_deref().is_some_and(stakes_held);
    if !held {
        if *planner
            != (Planner {
                how: planner.how,
                ..default()
            })
        {
            planner.drop_open();
            planner.eraser = false;
        }
        menu.open = false;
        return;
    }
    // Esc opens the game's menu, and drops a road with no end (42l).
    if menu_open.is_some_and(|m| m.0) {
        planner.drop_open();
        menu.open = false;
        return;
    }
    let on_foot = walking.is_some_and(|w| w.active);
    let x = keys.is_some_and(|k| k.pressed(KeyCode::KeyX)) && on_foot;
    if x && !menu.open {
        menu.open = true;
        menu.step = 0;
        menu.highlight = ROWS
            .iter()
            .position(|r| matches!(r, Row::Width(w) if *w == planner.how.width))
            .unwrap_or(0);
    }
    if menu.open && x {
        let n = ROWS.len() as i32;
        menu.highlight = (menu.highlight as i32 + menu.step).rem_euclid(n) as usize;
        menu.step = 0;
    }
    if menu.open && !x {
        menu.open = false;
        ROWS[menu.highlight].take(&mut planner);
        info!("ROAD_PLANNER menu: {:?}", ROWS[menu.highlight]);
    }
}

/// The aim's ghost and the bill, recut when the aimed cell, the planner or
/// the save changes (design 42a, 42d).
#[allow(clippy::too_many_arguments)]
pub fn planner_aim(
    aimed: Option<Res<crate::husbandry::AimedGround>>,
    hotbar: Option<Res<crate::hotbar::Hotbar>>,
    save: Option<Res<WorldSave>>,
    (fine, contact): (
        Option<Res<crate::planet::PlanetFine>>,
        Option<Res<crate::planet::PlanetContact>>,
    ),
    gather: Option<Res<crate::config::GatherConfig>>,
    settings: Res<RoadsSettings>,
    pcfg: Res<PlannerSettings>,
    planner: Res<Planner>,
    mut aim: ResMut<PlannerAim>,
) {
    let (Some(hotbar), Some(save)) = (hotbar, save) else {
        return;
    };
    if !stakes_held(&hotbar) {
        if aim.showing {
            *aim = PlannerAim {
                generation: aim.generation + 1,
                ..default()
            };
        }
        return;
    }
    let key = aimed.and_then(|a| a.0).and_then(paving::key_of);
    let stamp = (key, planner.clone(), save.serial());
    if aim.made_for.as_ref() == Some(&stamp) {
        return;
    }
    let config = crate::planet::terrain_config();
    let ford = settings.0.ford_depth_m;
    let axe_logs = gather.map_or(4, |g| g.0.axe_logs);
    let plans = planner::plans(&save.records);
    let ground = paving::stake_ground(config, &save.edits, &save.yields);
    let tree = |k: u32| match (&fine, &contact) {
        (Some(f), Some(c)) => tree_on(f, c, &save, axe_logs, k),
        _ => false,
    };
    // The road the next click makes, or the roads a picked-up stake drags.
    let mut roads_aimed: Vec<PlanRoad> = Vec::new();
    if let (Some(start), Some(k)) = (planner.start, key)
        && !planner.eraser
    {
        roads_aimed.push(PlanRoad::new(
            planner::stake_line(start, k, pcfg.0.max_segment),
            planner.how,
        ));
    } else if let (Some(from), Some(k)) = (planner.grabbed, key) {
        if let Ok(e) = planner::move_stake(&plans, from, k, &ground, &|_| false, &pcfg.0, ford) {
            roads_aimed = e
                .plans
                .values()
                .flat_map(|p| p.roads.iter())
                .filter(|r| r.stakes.contains(&k))
                .cloned()
                .collect();
        }
    } else if let Some(k) = key
        && !planner.eraser
    {
        // Idle: the cell a click would start on.
        roads_aimed.push(PlanRoad::new(vec![k], planner.how));
    }
    let lit = |k: u32| super::paving::lamp_standing(&save, k);
    let mut cells = Vec::new();
    let mut lamps = Vec::new();
    for road in &roads_aimed {
        let laid = planner::lay(road, &ground, &tree, &pcfg.0, ford);
        lamps.extend(ghost_lamps(
            planner::lamp_posts(road, &laid, &ground, &pcfg.0)
                .into_iter()
                .map(|(_, post)| post),
            &lit,
            &ground,
        ));
        cells.extend(laid);
    }
    let clash = cells.iter().find_map(|c| c.clash);
    let built = roads::built(&save.records);
    let bill = planner::bill(
        &plans,
        roads_aimed.first().filter(|_| planner.start.is_some()),
        &built,
        &lit,
        &ground,
        &pcfg.0,
        ford,
    );
    // What the hand holds now: the hotbar is the pack the bill is read
    // against.
    let have = [
        hotbar.0.count(Item::Good(Good::SURVEY_STAKES)),
        hotbar.0.count(Item::Good(Good::GRAVEL)),
        hotbar.0.count(Item::Good(Good::PLANKS)),
        hotbar.0.count(Item::Good(Good::LOG)),
        hotbar
            .0
            .count(Item::Block(pbd_core::terrain::Material::LanternPost)),
    ];
    *aim = PlannerAim {
        showing: true,
        cells,
        lamps,
        clash,
        bill,
        have,
        generation: aim.generation + 1,
        made_for: Some(stamp),
    };
}

/// The lamp posts the ghost stands (design 42o): each of `posts` not
/// holding its lamp yet, with its ground's top, m.
pub fn ghost_lamps(
    posts: impl IntoIterator<Item = u32>,
    lit: &dyn Fn(u32) -> bool,
    ground: &roads::GroundFn,
) -> Vec<(u32, i32)> {
    posts
        .into_iter()
        .filter(|&post| !lit(post))
        .filter_map(|post| ground(post).map(|g| (post, g.top_m)))
        .collect()
}

/// The planner's clicks (design 42k): the place button starts and ends a
/// road, starts one from a stake, sets a picked-up stake down, or with the
/// eraser takes a road; shift and the place button picks a stake up.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn planner_hands(
    (buttons, keys, aimed, walking, menu_open): (
        Option<Res<ButtonInput<MouseButton>>>,
        Option<Res<ButtonInput<KeyCode>>>,
        Option<Res<crate::husbandry::AimedGround>>,
        Option<Res<crate::walking::WalkingReadout>>,
        Option<Res<crate::controls::MenuOpen>>,
    ),
    (hotbar, save, net, mut standing): (
        Option<ResMut<crate::hotbar::Hotbar>>,
        Option<ResMut<WorldSave>>,
        Option<ResMut<RoadNet>>,
        ResMut<RoadsStanding>,
    ),
    (settings, pcfg, mut planner, menu, aim, mut say): (
        Res<RoadsSettings>,
        Res<PlannerSettings>,
        ResMut<Planner>,
        Res<PlannerMenu>,
        Res<PlannerAim>,
        ResMut<RoadSay>,
    ),
    mut work: MessageWriter<paving::RoadWork>,
    mut did: MessageWriter<crate::missions::Did>,
) {
    let (Some(buttons), Some(aimed), Some(mut hotbar), Some(mut save)) =
        (buttons, aimed, hotbar, save)
    else {
        return;
    };
    if !stakes_held(&hotbar)
        || menu.open
        || menu_open.is_some_and(|m| m.0)
        || !walking.is_some_and(|w| w.active && w.captured)
        || !buttons.just_pressed(MouseButton::Right)
    {
        return;
    }
    let Some(key) = aimed.0.and_then(paving::key_of) else {
        return;
    };
    let config = *crate::planet::terrain_config();
    let rcfg = settings.0.clone();
    let shift =
        keys.is_some_and(|k| k.pressed(KeyCode::ShiftLeft) || k.pressed(KeyCode::ShiftRight));
    let plans = planner::plans(&save.records);
    let staked = planner::stake_at(&plans, key).is_some();
    let say_why = |say: &mut RoadSay, why: EditRefused| {
        info!("ROAD_PLANNER refused: {why:?}");
        if let Some(line) = why.line() {
            say.0 = Some(line.into());
        }
    };
    let mut changed = false;
    if planner.eraser {
        match paving::erase_road(&mut save, &mut hotbar.0, key, &config, &rcfg, &pcfg.0) {
            Ok(true) => changed = true,
            Ok(false) => {}
            Err(why) => say_why(&mut say, why),
        }
    } else if let Some(from) = planner.grabbed {
        if from == key {
            planner.grabbed = None;
        } else {
            let tree = |k: u32| {
                aim.cells
                    .iter()
                    .any(|c| c.key == k && c.clash == Some(Clash::Tree))
            };
            match paving::move_stake(
                &mut save,
                &mut hotbar.0,
                from,
                key,
                &tree,
                &config,
                &rcfg,
                &pcfg.0,
            ) {
                Ok(()) => {
                    planner.grabbed = None;
                    changed = true;
                }
                Err(why) => say_why(&mut say, why),
            }
        }
    } else if shift && staked {
        planner.grabbed = Some(key);
        planner.start = None;
    } else if shift && planner.start.is_none() {
        // A stake driven on a leg and picked up (design 42f, 42m).
        match paving::insert_stake(&mut save, &mut hotbar.0, key, &config, &rcfg, &pcfg.0) {
            Ok(true) => {
                planner.grabbed = Some(key);
                changed = true;
                work.write(paving::RoadWork::Stake);
                did.write(crate::missions::Did::new(Kind::Stake, "stake", 1));
            }
            Ok(false) => {}
            Err(why) => say_why(&mut say, why),
        }
    } else if let Some(start) = planner.start {
        if start == key {
            planner.start = None;
        } else if let Some(why) = aim.clash.filter(|_| aim.made_for_key() == Some(key)) {
            say_why(&mut say, EditRefused::Clash(why));
        } else {
            let tree = |k: u32| {
                aim.cells
                    .iter()
                    .any(|c| c.key == k && c.clash == Some(Clash::Tree))
            };
            match paving::end_road(
                &mut save,
                &mut hotbar.0,
                start,
                key,
                planner.how,
                &tree,
                &config,
                &rcfg,
                &pcfg.0,
            ) {
                Ok((stakes, cells)) => {
                    planner.start = None;
                    changed = true;
                    for _ in 0..stakes {
                        work.write(paving::RoadWork::Stake);
                    }
                    if stakes > 0 {
                        did.write(crate::missions::Did::new(
                            Kind::Stake,
                            "stake",
                            stakes as u32,
                        ));
                    }
                    if cells > 0 {
                        work.write(paving::RoadWork::Mark(cells as u32));
                        did.write(crate::missions::Did::new(Kind::Mark, "cell", cells as u32));
                    }
                }
                Err(why) => say_why(&mut say, why),
            }
        }
    } else if staked {
        planner.start = Some(key);
    } else if let Some(why) = aim.clash.filter(|_| aim.made_for_key() == Some(key)) {
        say_why(&mut say, EditRefused::Clash(why));
    } else if hotbar.0.count(Item::Good(Good::SURVEY_STAKES)) == 0 {
        say_why(&mut say, EditRefused::NoStakes);
    } else {
        planner.start = Some(key);
    }
    if changed && let Some(mut net) = net {
        paving::restand(&save, &mut net, &mut standing);
    }
}

impl PlannerAim {
    /// The aimed cell it was made for.
    pub fn made_for_key(&self) -> Option<u32> {
        self.made_for.as_ref().and_then(|m| m.0)
    }
}

// ---- The lifted view (design 42n, the owner's survey P2) ----------------------

/// How far the planner's eye is lifted: `amount` from 0 (first person) to
/// 1 (the whole lift), eased over `aerial_ease_s`; `height_m` the lift.
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq)]
pub struct PlannerLift {
    pub amount: f32,
    pub height_m: f32,
}

/// How far behind the player the lifted eye stands, as a share of its
/// height, and how far ahead of them the ground the view centres on is
/// when they look level, m.
pub const LIFT_BACK: f32 = 0.6;
pub const LIFT_LOOK_AHEAD_M: f32 = 10.0;
/// How far the aim reaches from the lifted eye, m: past the ground its
/// ghost shows (`ghost_reach_m`, 60 m) from 30 m up.
pub const LIFTED_REACH_M: f32 = 90.0;

impl PlannerLift {
    /// The lift as the eye takes it: eased in and out, never a jump.
    pub fn eased(&self) -> f32 {
        let t = self.amount.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    /// Whether the eye is off the walker's at all.
    pub fn lifted(&self) -> bool {
        self.amount > 0.0
    }

    /// The eye's offset from the walker's (along `up` and back along the
    /// level `forward`), and the tilt down added to the player's own look,
    /// radians.
    pub fn offset(&self, up: Vec3, forward: Vec3) -> (Vec3, f32) {
        let k = self.eased();
        let h = self.height_m * k;
        let back = h * LIFT_BACK;
        let flat = (forward - up * forward.dot(up)).normalize_or_zero();
        let tilt = (h).atan2(back + LIFT_LOOK_AHEAD_M);
        (up * h - flat * back, tilt)
    }
}

/// Ease the planner's lifted view in while survey stakes are in hand on
/// foot, and out when they leave it (design 42n).
pub fn ease_lift(
    hotbar: Option<Res<crate::hotbar::Hotbar>>,
    walking: Option<Res<crate::walking::WalkingReadout>>,
    pcfg: Res<PlannerSettings>,
    time: Res<Time>,
    mut lift: ResMut<PlannerLift>,
) {
    let on = hotbar.as_deref().is_some_and(stakes_held) && walking.is_some_and(|w| w.active);
    let ease = pcfg.0.aerial_ease_s;
    let step = if ease > 0.0 {
        time.delta_secs() / ease
    } else {
        1.0
    };
    let target = if on { 1.0 } else { 0.0 };
    let amount = if lift.amount < target {
        (lift.amount + step).min(target)
    } else {
        (lift.amount - step).max(target)
    };
    let next = PlannerLift {
        amount,
        height_m: pcfg.0.aerial_m,
    };
    if *lift != next {
        if (lift.amount == 0.0) != (amount == 0.0) || (lift.amount == 1.0) != (amount == 1.0) {
            info!(
                "ROAD_PLANNER view lifted {amount:.2} of {} m",
                next.height_m
            );
        }
        *lift = next;
    }
}

// ---- The ghost ---------------------------------------------------------------

/// The ghost's root, meshes and materials.
#[derive(Resource, Default)]
pub struct Ghost {
    root: Option<Entity>,
    meshes: [Option<Handle<Mesh>>; 2],
    drawn: u64,
}

/// The ghost's materials: the plan's tint and the refusal's red, in the
/// gravel's own texture, blended.
#[derive(Resource)]
pub struct GhostLooks {
    pub tint: Handle<StandardMaterial>,
    pub red: Handle<StandardMaterial>,
}

pub fn make_ghost_looks(
    mut commands: Commands,
    assets: Res<AssetServer>,
    pcfg: Res<PlannerSettings>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let texture = crate::towns::texture(&assets, "gravel");
    let mut make = |rgb: [f32; 3]| {
        materials.add(StandardMaterial {
            base_color: Color::linear_rgba(rgb[0], rgb[1], rgb[2], pcfg.0.ghost_opacity),
            base_color_texture: Some(texture.clone()),
            uv_transform: bevy::math::Affine2::from_scale(Vec2::splat(0.5)),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.95,
            ..default()
        })
    };
    let looks = GhostLooks {
        tint: make(pcfg.0.ghost_tint),
        red: make(pcfg.0.refuse_tint),
    };
    // Shown behind the loading screen so its pipeline is built before play.
    let cap = meshes.add(Cuboid::new(0.01, 0.01, 0.01));
    commands.spawn((
        Mesh3d(cap),
        MeshMaterial3d(looks.tint.clone()),
        crate::field_light::LitByField,
        crate::field_light::LitLikeTerrain,
        Transform::default(),
        Visibility::Visible,
        crate::loading::PrewarmProxy,
        Name::new("Road ghost prewarm"),
    ));
    commands.insert_resource(looks);
}

/// How far the ghost stands over what it shows, m.
const GHOST_LIFT_M: f32 = 0.06;

/// The mesh of `cells`' road tops, lifted a little over them: each cell a
/// fan from its centre to its corners, on the plane its slope cuts (design
/// 10), smooth-normalled on the sphere, its gravel laid in metres. A bridge
/// cell not refused is the bridge piece paving builds, cut by the same
/// `roads::piece::cut` (its deck, posts and rails, railed where no cell of
/// `road` is beside it), and each of `lamps` (a post's cell and its
/// ground's top, m) stands the street lantern's ghost (design 42o).
pub fn ghost_mesh(
    cells: &[&LaidCell],
    road: &std::collections::BTreeSet<u32>,
    lamps: &[(u32, i32)],
    radius_m: f32,
    centre: Vec3,
) -> Mesh {
    let (mut positions, mut normals, mut uvs) = (Vec::new(), Vec::new(), Vec::new());
    let (east, north) = {
        let up = centre.normalize_or(Vec3::Y);
        let e = Vec3::Y.cross(up).normalize_or(Vec3::X);
        (e, up.cross(e))
    };
    for cell in cells {
        let Some(c) = roads::centre(cell.key) else {
            continue;
        };
        let drawn = cell.drawn();
        if cell.deck.is_some() && cell.clash.is_none() {
            let beside: Vec<u32> = roads::neighbours(cell.key)
                .into_iter()
                .filter(|n| road.contains(n))
                .collect();
            if let Some(piece) =
                roads::piece::cut(&drawn, Surface::Gravel, &beside, radius_m, &|_| 2.0)
            {
                let lift = GHOST_LIFT_M - roads::piece::LIFT_M;
                for buf in piece.meshes.into_values() {
                    for p in buf.positions {
                        let p = Vec3::from_array(p);
                        positions.push((p + p.normalize_or(c) * lift - centre).to_array());
                    }
                    normals.extend(buf.normals);
                    uvs.extend(buf.uvs);
                }
                continue;
            }
        }
        let point = |d: Vec3| {
            let h = if cell.clash.is_some() {
                cell.base_m as f32
            } else {
                drawn.top_at(d)
            };
            d * (radius_m + h + GHOST_LIFT_M)
        };
        let corners = roads::corners(cell.key);
        let k = corners.len();
        for s in 0..k {
            for d in [c, corners[s], corners[(s + 1) % k]] {
                let p = point(d);
                positions.push((p - centre).to_array());
                normals.push(d.to_array());
                uvs.push([p.dot(east), p.dot(north)]);
            }
        }
    }
    let lamp = super::models::ghost_lamp();
    for &(key, top_m) in lamps {
        let Some(frame) = roads::piece::frame(key, top_m as f32, radius_m) else {
            continue;
        };
        for &i in &lamp.indices {
            let i = i as usize;
            let p = frame.world(lamp.positions[i]);
            let n = lamp.normals[i];
            positions.push((p - centre).to_array());
            normals.push((frame.x * n.x + frame.y * n.y + frame.z * n.z).to_array());
            uvs.push(lamp.uvs[i].to_array());
        }
    }
    Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

/// Rebuild the ghost when the aim's cells change; it stands at the planet's
/// render frame, lit as the terrain is.
pub fn draw_ghost(
    mut commands: Commands,
    aim: Res<PlannerAim>,
    looks: Option<Res<GhostLooks>>,
    frame: Option<Res<crate::planet::PlanetRenderFrame>>,
    mut ghost: ResMut<Ghost>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let Some(looks) = looks else {
        return;
    };
    let centre = frame.as_ref().map_or(Vec3::ZERO, |f| f.center.as_vec3());
    let root = match ghost.root {
        Some(e) => e,
        None => {
            let e = commands
                .spawn((
                    Name::new("Road ghost"),
                    crate::field_light::LitByField,
                    crate::field_light::LitLikeTerrain,
                    Transform::from_translation(centre),
                    Visibility::default(),
                ))
                .id();
            ghost.root = Some(e);
            e
        }
    };
    if ghost.drawn == aim.generation {
        return;
    }
    ghost.drawn = aim.generation;
    let radius = crate::planet::terrain_config().radius_m;
    let at = aim
        .cells
        .first()
        .and_then(|c| roads::centre(c.key))
        .map_or(Vec3::Y * radius, |c| c * radius);
    let road: std::collections::BTreeSet<u32> = aim.cells.iter().map(|c| c.key).collect();
    for (n, red) in [false, true].into_iter().enumerate() {
        let cells: Vec<&LaidCell> = aim
            .cells
            .iter()
            .filter(|c| c.clash.is_some() == red)
            .collect();
        // The lamp posts stand in the plan's tint, never the red.
        let lamps: &[(u32, i32)] = if red { &[] } else { &aim.lamps };
        let mesh = ghost_mesh(&cells, &road, lamps, radius, at);
        match &ghost.meshes[n] {
            Some(handle) => {
                if let Some(m) = meshes.get_mut(handle) {
                    *m = mesh;
                }
            }
            None => {
                let handle = meshes.add(mesh);
                let material = if red {
                    looks.red.clone()
                } else {
                    looks.tint.clone()
                };
                let child = commands
                    .spawn((
                        Mesh3d(handle.clone()),
                        MeshMaterial3d(material),
                        Transform::default(),
                        Name::new("Road ghost cells"),
                    ))
                    .id();
                commands.entity(root).add_child(child);
                ghost.meshes[n] = Some(handle);
            }
        }
    }
    // The cells are laid relative to the aim's first cell.
    commands.entity(root).insert(GhostOffset(at));
}

/// Where the ghost's cells are laid from, planet-centred: the root stands
/// at the planet's render frame plus this.
#[derive(Component, Clone, Copy)]
pub struct GhostOffset(pub Vec3);

/// Keep the ghost's root at the planet's render frame plus its offset.
pub fn follow_ghost(
    frame: Option<Res<crate::planet::PlanetRenderFrame>>,
    mut roots: Query<(&GhostOffset, &mut Transform)>,
) {
    let centre = frame.as_ref().map_or(Vec3::ZERO, |f| f.center.as_vec3());
    for (offset, mut t) in &mut roots {
        t.translation = centre + offset.0;
    }
}

// ---- The HUD and the menu -----------------------------------------------------

/// The planner's line under the crosshair.
#[derive(Component)]
pub struct PlannerLine;

/// The planner menu's panel and its rows.
#[derive(Component)]
pub struct MenuPanel;
#[derive(Component)]
pub struct MenuRow(pub usize);

pub fn spawn_hud(mut commands: Commands) {
    commands.spawn((
        crate::ui_kit::KitText::new(Localized::empty(), crate::ui_kit::LABEL)
            .outline(crate::ui_kit::INK),
        Node {
            position_type: PositionType::Absolute,
            left: percent(50),
            top: percent(66),
            margin: UiRect::left(px(-200)),
            width: px(400),
            justify_content: JustifyContent::Center,
            ..default()
        },
        PlannerLine,
    ));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                top: percent(30),
                margin: UiRect::left(px(40)),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(6)),
                row_gap: px(2),
                ..default()
            },
            BackgroundColor(crate::ui_kit::DIMMER),
            Visibility::Hidden,
            MenuPanel,
        ))
        .with_children(|panel| {
            for (n, row) in ROWS.iter().enumerate() {
                panel.spawn((
                    crate::ui_kit::KitText::new(row.label(), crate::ui_kit::TEXT)
                        .outline(crate::ui_kit::INK),
                    Node::default(),
                    MenuRow(n),
                ));
            }
        });
}

/// The HUD line: the red reason while the aim is refused, else the bill;
/// and the menu with its highlight and the current choices.
#[allow(clippy::type_complexity)]
pub fn show_hud(
    aim: Res<PlannerAim>,
    planner: Res<Planner>,
    menu: Res<PlannerMenu>,
    mut line: Query<&mut crate::ui_kit::KitText, (With<PlannerLine>, Without<MenuRow>)>,
    mut panel: Query<&mut Visibility, With<MenuPanel>>,
    mut rows: Query<(&MenuRow, &mut crate::ui_kit::KitText), Without<PlannerLine>>,
) {
    let (text, colour) = hud_line(&aim, &planner);
    for mut t in &mut line {
        if t.text != text || t.colour != colour {
            t.text = text.clone();
            t.colour = colour;
        }
    }
    for mut v in &mut panel {
        *v = if menu.open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (row, mut t) in &mut rows {
        let r = ROWS[row.0];
        let colour = if row.0 == menu.highlight {
            crate::ui_kit::TITLE
        } else if r.chosen(&planner) {
            crate::ui_kit::RUNE
        } else {
            crate::ui_kit::TEXT_DIM
        };
        if t.colour != colour {
            t.colour = colour;
        }
    }
}

/// What the HUD line says, and its colour (design 42d, 42l).
pub fn hud_line(aim: &PlannerAim, planner: &Planner) -> (Localized, Color) {
    if !aim.showing {
        return (Localized::empty(), crate::ui_kit::LABEL);
    }
    if planner.eraser {
        return (tr("roads.menu.eraser"), crate::ui_kit::LABEL);
    }
    if let Some(why) = aim.clash {
        return (why.line(), crate::ui_kit::WAYPOINT);
    }
    let b = &aim.bill;
    let wanted = [b.stakes, b.gravel, b.planks, b.logs, b.lanterns];
    let short = wanted.iter().zip(aim.have).any(|(w, h)| *w > h);
    let text = if b.planks + b.logs + b.lanterns > 0 {
        tr_with(
            "roads.bill.line_more",
            &[
                ("stakes", &b.stakes),
                ("gravel", &b.gravel),
                ("planks", &b.planks),
                ("logs", &b.logs),
                ("lanterns", &b.lanterns),
                ("left", &b.left),
                ("cells", &b.cells),
                ("turns", &b.turns),
            ],
        )
    } else {
        tr_with(
            "roads.bill.line",
            &[
                ("stakes", &b.stakes),
                ("gravel", &b.gravel),
                ("left", &b.left),
                ("cells", &b.cells),
                ("turns", &b.turns),
            ],
        )
    };
    let colour = if short {
        crate::ui_kit::WAYPOINT
    } else {
        crate::ui_kit::LABEL
    };
    (text, colour)
}
