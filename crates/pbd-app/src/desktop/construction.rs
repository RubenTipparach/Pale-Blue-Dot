//! Construction sites in play (`building-planner` design 5 to 7): a plan set
//! out from build mode's Blueprint slot, its ghost drawn by the pieces still
//! to build, each struck with the mallet on F from the pack's goods, the
//! scaffolding put up and struck as the storeys rise, the HUD's count and
//! bill, the compass tick on the next piece, and a site's plan changed in
//! the planner, its dropped pieces marked and taken down.
//!
//! Every change goes through the save first (`pbd_app::construction`,
//! `pbd_app::builds`); the rules are the core's.

use super::build_mode::{
    BuildMode, GroundCache, PALETTE, Tool as PaletteTool, ground_top, reference,
};
use super::farm::FarmNote;
use super::menu::{self, Screen};
use super::planner::{PlanName, Planner, open as open_planner, site_display_name};
use super::slots::{Hotbar, ItemIcons, paint_icon};
use bevy::prelude::*;
use pbd_app::builds::{Builds, PATCH_REACH_M, Scaffolds, patch_round};
use pbd_app::construction::{
    PlannerData, SiteShown, Sites, chart_site, give_goods, good_name, set_out, store_site,
    store_site_with_hands, take_goods,
};
use pbd_app::interact::{Act, Interact, Offer, Reader};
use pbd_app::planet::{PLANET_RADIUS, PlanetContact, PlanetFine, PlanetRenderFrame};
use pbd_app::saves::WorldSave;
use pbd_app::ui_kit::{self, KitText, Localized, Set, UiKit, tr, tr_with};
use pbd_core::building::cut::cut;
use pbd_core::building::plan::{Plan, PlanCells, PlanPiece, adopt, chart_plan};
use pbd_core::building::rules::{can_place, dependant};
use pbd_core::building::scaffold::{Bay, scaffold};
use pbd_core::building::site::{
    BuildingSetting, Refusal, SiteBody, SiteChart, Size, World, check_site, diff, is_built,
    progress, scaffold_bays, scaffold_cost, still_needed,
};
use pbd_core::building::{Cells, Piece, STOREY, Shape, Site};
use pbd_core::inventory::{Item, Stack};
use pbd_core::settlement::pieces::MeshBuf;
use pbd_core::terrain::Material;
use std::collections::{BTreeMap, HashMap};

/// A plan ready to be set out: the planner's, or one chosen in build mode.
#[derive(Clone, Debug, PartialEq)]
pub struct InHand {
    pub plan: Plan,
    /// The player's words, or a preset's or a town's locale key.
    pub name: String,
    pub preset: Option<(String, Size)>,
}

/// The plan the planner sent out to be set out (design 5).
#[derive(Resource, Default)]
pub struct PlanInHand(pub Option<InHand>);

/// What build mode's Blueprint slot steps through with the wheel: the plan
/// in hand first, then the presets and the player's own blueprints.
#[derive(Resource, Default)]
pub struct PlanChoices(pub Vec<InHand>, (usize, usize));

/// A change saved in the planner, for [`apply_change`] (design 7).
#[derive(Resource, Default)]
pub struct ChangeAsked(pub Option<(u64, Plan)>);

/// A house outside any site, read as a plan to change (design 7): saved in
/// the planner, it becomes a site.
#[derive(Resource, Default)]
pub struct Adopting(pub Option<SiteBody>);

/// The next piece to build, for the compass's tick (design 6).
#[derive(Resource, Default)]
pub struct SiteTick(pub Option<Vec3>);

/// What F would do to a site's piece now, and how far it has gone.
#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    pub site: u64,
    pub piece: Piece,
    pub act: Act,
}

/// The mallet's work: the piece being struck and its strokes so far.
#[derive(Resource, Default, Debug)]
pub struct Striking {
    pub target: Option<Target>,
    pub strokes: u32,
    since_s: f32,
}

/// The ghost's three meshes: buildable now, waiting on others, and marked
/// for removal (the chalk wash).
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum SiteGhost {
    Ready,
    Waiting,
    Marked,
}

/// The HUD's site lines.
#[derive(Component)]
pub struct SiteHud;
#[derive(Component)]
pub struct SiteHudLines;

/// Keep the sites in step with the save, and the planner's data loaded.
pub fn follow(save: Option<Res<WorldSave>>, mut sites: ResMut<Sites>) {
    if let Some(save) = save {
        sites.follow(&save);
    }
}

/// The plans build mode's Blueprint slot steps through.
pub fn choices(
    data: Res<PlannerData>,
    in_hand: Res<PlanInHand>,
    save: Option<Res<WorldSave>>,
    mut choices: ResMut<PlanChoices>,
    mut mode: ResMut<BuildMode>,
) {
    let mine = save
        .as_deref()
        .map(|s| pbd_core::building::site::blueprints(&s.records))
        .unwrap_or_default();
    let stamp = (usize::from(in_hand.0.is_some()), mine.len());
    if !in_hand.is_changed() && choices.1 == stamp && !choices.0.is_empty() {
        return;
    }
    let mut list: Vec<InHand> = in_hand.0.iter().cloned().collect();
    for preset in &data.presets.presets {
        list.push(InHand {
            plan: preset.plan(Size::Standard),
            name: format!("blueprint.preset.{}", preset.id),
            preset: Some((preset.id.clone(), Size::Standard)),
        });
    }
    for body in mine.into_values() {
        if let Some(plan) = body.plan {
            list.push(InHand {
                plan,
                name: body.name,
                preset: body.preset,
            });
        }
    }
    *choices = PlanChoices(list, stamp);
    if in_hand.is_changed() && in_hand.0.is_some() {
        // The planner's plan is in hand: build mode opens on it.
        mode.on = true;
        mode.slot = PALETTE
            .iter()
            .position(|t| *t == PaletteTool::Blueprint)
            .unwrap_or(0);
        mode.choice = 0;
        mode.turn = 0;
    }
}

/// The world's building setting.
pub fn setting(save: &WorldSave) -> BuildingSetting {
    save.identity.building()
}

/// What the world holds round a site, for the whole-site check: whether a
/// block is taken by a solid block or a standing tree (design 5).
pub fn blocked(
    fine: &PlanetFine,
    save: &WorldSave,
    cell: u32,
    layer: u16,
    ground: Option<u16>,
) -> bool {
    let block = (cell, usize::from(layer));
    if super::digging::material_of(fine, save, block)
        .is_some_and(|m| m != Material::Air && !m.is_water())
    {
        return true;
    }
    // A tree stands in the block over its ground, a storey of it.
    ground.is_some_and(|g| {
        layer >= g
            && layer < g + STOREY
            && super::digging::under_standing_tree(
                fine,
                save,
                super::farm::axe_logs(None),
                (cell, usize::from(g.saturating_sub(1))),
            )
    })
}

/// A plan charted where it stands: the patch, the origin's index, the base
/// layer and the pieces.
pub type ChartedHere = (
    std::sync::Arc<pbd_core::settlement::chart::Patch>,
    usize,
    u16,
    Vec<(PlanPiece, Piece)>,
);

/// Chart `plan` at `cell` with turn `turn` on the ground under it: the
/// patch, the origin's index, the base layer and the charted pieces.
pub fn chart_here(
    fine: &PlanetFine,
    save: &WorldSave,
    cache: &mut GroundCache,
    plan: &Plan,
    cell: u32,
    turn: usize,
) -> Option<ChartedHere> {
    let direction = pbd_core::cell_key::key_direction(cell)?;
    let patch = std::sync::Arc::new(patch_round(direction, PATCH_REACH_M));
    let at = patch.keys.iter().position(|&k| k == cell)?;
    let first = chart_plan(&patch, plan, at, turn, 0).ok()?;
    let ground: Vec<Option<u16>> = first
        .iter()
        .filter_map(|(pp, p)| match (pp, &p.shape) {
            (PlanPiece::Floor { storey: 0, .. }, Shape::Floor { cell, .. }) => Some(*cell),
            _ => None,
        })
        .map(|c| ground_top(fine, save, cache, c))
        .collect();
    let base = ground
        .iter()
        .copied()
        .try_fold(0u16, |m, g| Some(m.max(g?)))?;
    let charted = chart_plan(&patch, plan, at, turn, base).ok()?;
    Some((patch, at, base, charted))
}

/// The whole site checked against the world (design 5).
#[allow(clippy::too_many_arguments)]
pub fn check_here(
    fine: &PlanetFine,
    save: &WorldSave,
    cache: &mut GroundCache,
    patch: &pbd_core::settlement::chart::Patch,
    charted: &[(PlanPiece, Piece)],
    base: u16,
    near: &[Piece],
    data: &PlannerData,
    plan: &Plan,
) -> Result<(), (Refusal, Vec<usize>)> {
    let cells = Cells::new(patch);
    let mut tops: HashMap<u32, Option<u16>> = HashMap::new();
    for (_, p) in charted {
        for c in p.shape.cells() {
            tops.entry(c)
                .or_insert_with(|| ground_top(fine, save, cache, c));
        }
    }
    let ground = |c: u32| tops.get(&c).copied().flatten();
    let site = Site {
        cells: &cells,
        ground: &ground,
    };
    let taken = |c: u32, l: u16| blocked(fine, save, c, l, tops.get(&c).copied().flatten());
    let world = World {
        site: &site,
        blocked: &taken,
        pieces: near,
    };
    let charted: Vec<(PlanPiece, Piece)> = charted.to_vec();
    check_site(&world, &charted, base, &data.tun, plan)
}

/// The ground's first air layer round a site's pieces, for the rules.
fn tops_of(
    fine: &PlanetFine,
    save: &WorldSave,
    cache: &mut GroundCache,
    pieces: impl Iterator<Item = u32>,
) -> HashMap<u32, Option<u16>> {
    let mut tops = HashMap::new();
    for c in pieces {
        tops.entry(c)
            .or_insert_with(|| ground_top(fine, save, cache, c));
    }
    tops
}

/// Whether a site's piece can be struck now: held up by what stands, by
/// build mode's own rules.
fn buildable(site: &Site, standing: &[Piece], piece: &Piece) -> bool {
    !matches!(piece.shape, Shape::Furniture { .. }) && can_place(site, standing, piece).is_ok()
}

/// Draw the ghosts of every site near the player (design 5): the pieces
/// left to build in the ghost's tint, those buildable now stronger than
/// those waiting, and the pieces marked for removal in a chalk wash.
#[allow(clippy::too_many_arguments)]
pub fn draw_ghosts(
    sites: Res<Sites>,
    builds: Res<Builds>,
    data: Res<PlannerData>,
    assets: Res<pbd_app::towns::TownAssets>,
    (fine, save): (Option<Res<PlanetFine>>, Option<Res<WorldSave>>),
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    frame: Option<Res<PlanetRenderFrame>>,
    mut ghosts: Query<(&SiteGhost, &Mesh3d, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cache: Local<GroundCache>,
    mut drawn: Local<Option<(u64, u64, Vec<u64>)>>,
) {
    let (Some(fine), Some(save)) = (fine, save) else {
        return;
    };
    let Some((eye, _)) = cameras.iter().find(|(_, c)| c.is_active) else {
        return;
    };
    let eye = pbd_app::render_origin::local_of(frame.as_deref(), eye.translation());
    let reach = data.tun.ghost_reach_m;
    let near: Vec<(&u64, &SiteShown)> = sites
        .by_id
        .iter()
        .filter(|(_, s)| {
            pbd_core::cell_key::key_direction(s.body.chart.origin)
                .is_some_and(|d| (d * eye.length()).distance(eye) <= reach + 12.0)
        })
        .collect();
    let ids: Vec<u64> = near.iter().map(|(id, _)| **id).collect();
    let key = (sites.revision, builds.revision, ids.clone());
    if drawn.as_ref() == Some(&key) {
        return;
    }
    *drawn = Some(key);
    let mut bufs: BTreeMap<u8, MeshBuf> = BTreeMap::new();
    let repeat = |_: &str| 2.0;
    for (_, site) in &near {
        let cells = Cells::new(&site.patch);
        let standing: Vec<Piece> = builds.pieces.values().cloned().collect();
        let tops = tops_of(
            &fine,
            &save,
            &mut cache,
            site.charted.iter().flat_map(|(_, p)| p.shape.cells()),
        );
        let ground = |c: u32| tops.get(&c).copied().flatten();
        let rules = Site {
            cells: &cells,
            ground: &ground,
        };
        let mut groups: BTreeMap<u8, Vec<&Piece>> = BTreeMap::new();
        for (_, piece) in &site.charted {
            if is_built(&builds.pieces, piece) {
                continue;
            }
            let ready = buildable(&rules, &standing, piece);
            groups
                .entry(if ready { 0 } else { 1 })
                .or_default()
                .push(piece);
        }
        for piece in site.body.marked.values() {
            if is_built(&builds.pieces, piece) {
                groups.entry(2).or_default().push(piece);
            }
        }
        let anchor = site.body.chart.origin;
        for (group, pieces) in groups {
            let Some(c) = cut(
                &cells,
                anchor,
                &pieces,
                &assets.kits,
                &repeat,
                PLANET_RADIUS,
            ) else {
                continue;
            };
            let buf = bufs.entry(group).or_default();
            for b in c.meshes.into_values() {
                buf.positions.extend(&b.positions);
                buf.normals.extend(&b.normals);
                buf.uvs.extend(&b.uvs);
            }
        }
    }
    info!(
        "SITE ghosts: {} sites near, {:?} vertices by group",
        near.len(),
        bufs.iter()
            .map(|(g, b)| (*g, b.positions.len()))
            .collect::<Vec<_>>()
    );
    for (ghost, mesh, mut v) in &mut ghosts {
        let group = match ghost {
            SiteGhost::Ready => 0,
            SiteGhost::Waiting => 1,
            SiteGhost::Marked => 2,
        };
        match bufs.get_mut(&group).filter(|b| !b.positions.is_empty()) {
            Some(buf) => {
                // A hair out along each face, so a marked piece's wash is
                // seen over the piece.
                for (p, n) in buf.positions.iter_mut().zip(&buf.normals) {
                    for k in 0..3 {
                        p[k] += n[k] * 0.02;
                    }
                }
                if let Some(m) = meshes.get_mut(&mesh.0) {
                    *m = ghost_mesh(buf);
                }
                *v = Visibility::Visible;
            }
            None => *v = Visibility::Hidden,
        }
    }
}

fn ghost_mesh(buf: &MeshBuf) -> Mesh {
    let n = buf.positions.len() as u32;
    Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, buf.positions.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, buf.normals.clone())
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, buf.uvs.clone())
    .with_inserted_indices(bevy::mesh::Indices::U32((0..n).collect()))
}

/// The ghost's meshes, made at startup hidden so their pipelines warm with
/// the world's: a tint of the piece's own cut, never an edge drawn round it.
pub fn spawn_ghosts(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    data: Res<PlannerData>,
) {
    let [r, g, b] = data.tun.ghost_tint;
    for (ghost, colour) in [
        (
            SiteGhost::Ready,
            Color::linear_rgba(r, g, b, data.tun.ghost_opacity),
        ),
        (
            SiteGhost::Waiting,
            Color::linear_rgba(r, g, b, data.tun.ghost_waiting_opacity),
        ),
        // The chalk wash on a piece to take down.
        (SiteGhost::Marked, Color::srgba(0.93, 0.92, 0.86, 0.45)),
    ] {
        let material = materials.add(StandardMaterial {
            base_color: colour,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        });
        let mesh = meshes.add(ghost_mesh(&MeshBuf {
            positions: vec![[0.0; 3]; 3],
            normals: vec![[0.0, 1.0, 0.0]; 3],
            uvs: vec![[0.0; 2]; 3],
        }));
        commands.spawn((
            Name::new("Site ghost"),
            ghost,
            pbd_app::towns::TownRoot,
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::default(),
            Visibility::Hidden,
            bevy::light::NotShadowCaster,
            bevy::light::NotShadowReceiver,
        ));
    }
}

/// The piece of a site F would work on: the one nearest the look within
/// the mallet's reach, buildable now, or marked to take down, or a slot
/// whose furniture is carried.
#[allow(clippy::too_many_arguments)]
fn aimed_target(
    sites: &Sites,
    builds: &Builds,
    data: &PlannerData,
    fine: &PlanetFine,
    save: &WorldSave,
    cache: &mut GroundCache,
    carried: &pbd_core::inventory::Slots,
    eye: Vec3,
    look: Vec3,
) -> Option<Target> {
    let mut best: Option<(f32, Target)> = None;
    let standing: Vec<Piece> = builds.pieces.values().cloned().collect();
    for (id, site) in &sites.by_id {
        let Some(d) = pbd_core::cell_key::key_direction(site.body.chart.origin) else {
            continue;
        };
        if (d * eye.length()).distance(eye) > data.tun.ghost_reach_m {
            continue;
        }
        let cells = Cells::new(&site.patch);
        let tops = tops_of(
            fine,
            save,
            cache,
            site.charted.iter().flat_map(|(_, p)| p.shape.cells()),
        );
        let ground = |c: u32| tops.get(&c).copied().flatten();
        let rules = Site {
            cells: &cells,
            ground: &ground,
        };
        let mut consider = |piece: &Piece, act: Act| {
            let Some(at) = reference(piece) else {
                return;
            };
            let to = at - eye;
            let dist = to.length();
            if dist > data.tun.reach_m + 1.0 {
                return;
            }
            let off = 1.0 - to.normalize_or_zero().dot(look);
            if off > 0.08 {
                return;
            }
            let score = off * 10.0 + dist * 0.05;
            if best.as_ref().is_none_or(|(s, _)| score < *s) {
                best = Some((
                    score,
                    Target {
                        site: *id,
                        piece: piece.clone(),
                        act,
                    },
                ));
            }
        };
        // Marked pieces come down top first.
        if let Some(next) = pbd_core::building::site::next_down(&site.body.marked, &standing)
            && is_built(&builds.pieces, next)
        {
            consider(next, Act::TakeDown);
        }
        if !site.body.marked.is_empty() {
            continue;
        }
        for (_, piece) in &site.charted {
            if is_built(&builds.pieces, piece) {
                continue;
            }
            match piece.shape {
                Shape::Furniture { kind, .. } => {
                    if carried.count(Item::Good(kind.good())) > 0
                        && can_place(&rules, &standing, piece).is_ok()
                    {
                        consider(piece, Act::Fill);
                    }
                }
                _ => {
                    if buildable(&rules, &standing, piece) {
                        consider(piece, Act::Build);
                    }
                }
            }
        }
    }
    best.map(|(_, t)| t)
}

/// A piece's name as the prompt and the HUD say it.
fn piece_name(piece: &Piece) -> Localized {
    match piece.shape {
        Shape::Furniture { kind, .. } => kind.name(),
        ref shape => tr(&format!(
            "build.piece.{}",
            pbd_app::town_missions::build_name(shape)
        )),
    }
}

/// The mallet on F (design 6): offer what F would do to the aimed piece,
/// strike it on the mallet's beat while F is held, and on the last stroke
/// take its goods from the pack and place it, take a marked piece down and
/// give its goods back, or set a carried piece of furniture in its slot.
#[allow(clippy::too_many_arguments)]
pub fn strike(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    screen: Res<Screen>,
    walking: Option<Res<pbd_app::walking::WalkingReadout>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    frame: Option<Res<PlanetRenderFrame>>,
    (fine, mut save): (Option<Res<PlanetFine>>, Option<ResMut<WorldSave>>),
    (sites, mut builds, data): (Res<Sites>, ResMut<Builds>, Res<PlannerData>),
    mut hotbar: ResMut<Hotbar>,
    mut interact: ResMut<Interact>,
    mut striking: ResMut<Striking>,
    mut note: ResMut<FarmNote>,
    mut tick: ResMut<SiteTick>,
    mut did: MessageWriter<pbd_app::missions::Did>,
    mut cache: Local<GroundCache>,
) {
    let (Some(fine), Some(save)) = (fine, save.as_deref_mut()) else {
        return;
    };
    let on_foot = walking.is_some_and(|w| w.active && w.captured) && *screen == Screen::Playing;
    let Some((camera, _)) = cameras.iter().find(|(_, c)| c.is_active) else {
        return;
    };
    let eye = pbd_app::render_origin::local_of(frame.as_deref(), camera.translation());
    let look = camera.forward().as_vec3();
    let target = if on_foot {
        aimed_target(
            &sites, &builds, &data, &fine, save, &mut cache, &hotbar.0, eye, look,
        )
    } else {
        None
    };
    // The compass's tick: the aimed piece, or the nearest buildable one.
    tick.0 = target
        .as_ref()
        .and_then(|t| reference(&t.piece))
        .or_else(|| next_piece_near(&sites, &builds, eye, data.tun.ghost_reach_m));
    if target.as_ref().map(|t| (t.site, t.piece.id(), t.act))
        != striking
            .target
            .as_ref()
            .map(|t| (t.site, t.piece.id(), t.act))
    {
        striking.target = target.clone();
        striking.strokes = 0;
        striking.since_s = 0.0;
    }
    let Some(target) = target else {
        return;
    };
    interact.offer(Offer::new(target.act, piece_name(&target.piece)));
    if !keys.pressed(KeyCode::KeyF) || !pbd_app::interact::may(Some(&interact), Reader::Site) {
        striking.strokes = 0;
        striking.since_s = 0.0;
        return;
    }
    let now = time.elapsed_secs();
    let planned = setting(save) == BuildingSetting::Planned;
    // Building needs the mallet; furniture is set in its slot by hand.
    if target.act != Act::Fill && hotbar.0.count(Item::Good(pbd_core::goods::Good::MALLET)) == 0 {
        if keys.just_pressed(KeyCode::KeyF) {
            note.say(now, tr("site.needs_mallet"));
        }
        return;
    }
    let need = if target.act == Act::Fill {
        1
    } else {
        data.tun.strokes_of(&target.piece)
    };
    striking.since_s += time.delta_secs();
    if keys.just_pressed(KeyCode::KeyF) || striking.since_s >= data.tun.stroke_s {
        striking.since_s = 0.0;
        striking.strokes += 1;
        if striking.strokes < need {
            note.say_for(
                now,
                tr_with("site.stroke", &[("n", &striking.strokes), ("of", &need)]),
                data.tun.stroke_s + 0.2,
            );
            return;
        }
    } else {
        return;
    }
    striking.strokes = 0;
    let piece = target.piece.clone();
    match target.act {
        Act::Build => {
            let goods = if planned {
                data.costs
                    .cost_of(&piece, data.tun.cost_scale)
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            let after = match take_goods(&hotbar.0, &goods) {
                Ok(after) => after,
                Err((good, count)) => {
                    note.say(
                        now,
                        tr_with(
                            "site.needs",
                            &[("count", &count), ("good", &good_name(&good))],
                        ),
                    );
                    return;
                }
            };
            if builds.place_with_hands(save, piece.clone(), &after) {
                hotbar.0 = after;
                info!("SITE {} built {:?}", target.site, piece.shape.class());
                did.write(pbd_app::missions::Did::new(
                    pbd_core::missions::Kind::Build,
                    pbd_app::town_missions::build_name(&piece.shape),
                    1,
                ));
            }
        }
        Act::Fill => {
            let Shape::Furniture { kind, .. } = piece.shape else {
                return;
            };
            let mut after = hotbar.0.clone();
            if after.take_item(Item::Good(kind.good()), 1)
                && builds.place_with_hands(save, piece.clone(), &after)
            {
                hotbar.0 = after;
                info!("SITE {} set a {} in its slot", target.site, kind.code());
            }
        }
        Act::TakeDown => {
            let standing: Vec<Piece> = builds.pieces.values().cloned().collect();
            if let Some(d) = dependant(&standing, &piece) {
                let thing = piece_name(d);
                note.say(now, tr_with("build.needed_by", &[("thing", &thing)]));
                return;
            }
            let back = if planned {
                pbd_core::building::costs::refund(
                    &data
                        .costs
                        .cost_of(&piece, data.tun.cost_scale)
                        .unwrap_or_default(),
                    data.tun.refund_share,
                )
            } else {
                Vec::new()
            };
            let Some(after) = give_goods(&hotbar.0, &back) else {
                note.say(now, tr("furniture.full"));
                return;
            };
            if builds.remove_with_hands(save, piece.id(), &after) {
                hotbar.0 = after;
                if let Some(site) = sites.by_id.get(&target.site) {
                    let mut body = site.body.clone();
                    body.marked.remove(&piece.id());
                    store_site(save, target.site, &body);
                }
                info!("SITE {} took down {:?}", target.site, piece.shape.class());
            }
        }
        _ => {}
    }
}

/// The nearest piece left to build within `reach_m`, for the compass.
fn next_piece_near(sites: &Sites, builds: &Builds, eye: Vec3, reach_m: f32) -> Option<Vec3> {
    sites
        .by_id
        .values()
        .flat_map(|s| s.charted.iter().map(|(_, p)| p))
        .filter(|p| !matches!(p.shape, Shape::Furniture { .. }) && !is_built(&builds.pieces, p))
        .filter_map(reference)
        .filter(|at| at.distance(eye) <= reach_m)
        .min_by(|a, b| a.distance(eye).total_cmp(&b.distance(eye)))
}

/// Put up and strike each site's scaffolding (design 6): round a storey
/// above the ground once the walls under it are built and until its own
/// are, paid for in a planned world and given back when it comes down; and
/// cut what stands for the walker and the draw.
#[allow(clippy::too_many_arguments)]
pub fn scaffolding(
    mut save: Option<ResMut<WorldSave>>,
    sites: Res<Sites>,
    mut builds: ResMut<Builds>,
    data: Res<PlannerData>,
    mut hotbar: ResMut<Hotbar>,
    mut scaffolds: ResMut<Scaffolds>,
    mut note: ResMut<FarmNote>,
    time: Res<Time>,
    mut cut_for: Local<Option<(u64, u64)>>,
) {
    let Some(save) = save.as_deref_mut() else {
        return;
    };
    let planned = setting(save) == BuildingSetting::Planned;
    for (id, site) in &sites.by_id {
        for s in 1..site.body.plan.storeys.len() as u8 {
            let want = pbd_core::building::site::wants_scaffold(
                &site.charted,
                s,
                &builds.pieces,
                &data.tun,
            );
            if want == site.body.scaffold.contains(&s) {
                continue;
            }
            let cost = scaffold_cost(scaffold_bays(&site.body.plan, s).len(), &data.tun);
            let mut body = site.body.clone();
            let after = if !planned {
                Some(hotbar.0.clone())
            } else if want {
                take_goods(&hotbar.0, &cost).ok()
            } else {
                give_goods(&hotbar.0, &cost)
            };
            let Some(after) = after else {
                // Not enough to put it up: said once a while, tried again.
                if want && (time.elapsed_secs() % 6.0) < time.delta_secs() {
                    note.say(time.elapsed_secs(), tr("site.scaffold_needs"));
                }
                continue;
            };
            if want {
                body.scaffold.insert(s);
            } else {
                body.scaffold.remove(&s);
            }
            if store_site_with_hands(save, *id, &body, &after) {
                hotbar.0 = after;
                info!(
                    "SITE {id} scaffolding {} round storey {s}",
                    if want { "up" } else { "down" }
                );
            }
        }
    }
    let key = (sites.revision, 0);
    if *cut_for == Some(key) {
        return;
    }
    *cut_for = Some(key);
    let mut stands = Vec::new();
    for site in sites.by_id.values() {
        let cells = Cells::new(&site.patch);
        let Some(at) = site
            .patch
            .keys
            .iter()
            .position(|&k| k == site.body.chart.origin)
        else {
            continue;
        };
        let pc = PlanCells::new(&site.patch, at, usize::from(site.body.chart.turn), 6);
        for &s in &site.body.scaffold {
            let bays: Vec<Bay> = scaffold_bays(&site.body.plan, s)
                .into_iter()
                .filter_map(|(h, d)| {
                    Some(Bay {
                        inside: pc.key(h).ok()?,
                        outside: pc.across(h, d).ok()?,
                    })
                })
                .collect();
            let floor = site.body.chart.base + u16::from(s) * STOREY;
            if let Some(stand) = scaffold(
                &cells,
                site.body.chart.origin,
                &bays,
                site.body.chart.base,
                floor,
                PLANET_RADIUS,
            ) {
                stands.push(stand);
            }
        }
    }
    if !(stands.is_empty() && scaffolds.0.is_empty()) {
        scaffolds.0 = stands;
        builds.revision += 1;
    }
}

/// Apply a change saved in the planner (design 7): the new plan charted
/// where the site stands, checked whole against the world with the site's
/// own pieces as its own, then the site written with what to take down.
#[allow(clippy::too_many_arguments)]
pub fn apply_change(
    mut change: ResMut<ChangeAsked>,
    mut adopting: ResMut<Adopting>,
    (fine, mut save): (Option<Res<PlanetFine>>, Option<ResMut<WorldSave>>),
    sites: Res<Sites>,
    mut builds: ResMut<Builds>,
    data: Res<PlannerData>,
    mut hotbar: ResMut<Hotbar>,
    mut screen: ResMut<Screen>,
    mut planner: ResMut<Planner>,
    mut note: ResMut<FarmNote>,
    time: Res<Time>,
    mut cache: Local<GroundCache>,
) {
    let Some((id, plan)) = change.0.take() else {
        return;
    };
    let (Some(fine), Some(save)) = (fine, save.as_deref_mut()) else {
        return;
    };
    let now = time.elapsed_secs();
    // The site as it was, or the house being adopted.
    let old = if id == 0 {
        adopting.0.clone()
    } else {
        sites.by_id.get(&id).map(|s| s.body.clone())
    };
    let Some(old) = old else {
        return;
    };
    let Some((patch, old_charted)) = chart_site(&old) else {
        return;
    };
    let mut new_body = old.clone();
    new_body.plan = plan;
    let Some((_, new_charted)) = chart_site(&new_body) else {
        planner.hint = Some("site.refused.pentagon".into());
        planner.revision += 1;
        return;
    };
    let near = builds.near(
        pbd_core::cell_key::key_direction(old.chart.origin).unwrap_or(Vec3::Y),
        PATCH_REACH_M,
    );
    if let Err((why, _)) = check_here(
        &fine,
        save,
        &mut cache,
        &patch,
        &new_charted,
        old.chart.base,
        &near,
        &data,
        &new_body.plan,
    ) {
        // Its own pieces dropped by the change are not in the way.
        let mine: Vec<u64> = old_charted.iter().map(|(_, p)| p.id()).collect();
        let others: Vec<Piece> = near
            .iter()
            .filter(|p| !mine.contains(&p.id()))
            .cloned()
            .collect();
        if check_here(
            &fine,
            save,
            &mut cache,
            &patch,
            &new_charted,
            old.chart.base,
            &others,
            &data,
            &new_body.plan,
        )
        .is_err()
        {
            planner.hint = Some(why.key());
            planner.revision += 1;
            note.say(now, tr(&why.key()));
            return;
        }
    }
    let old_pieces: Vec<Piece> = old_charted.iter().map(|(_, p)| p.clone()).collect();
    let new_pieces: Vec<Piece> = new_charted.iter().map(|(_, p)| p.clone()).collect();
    let d = diff(
        &old_pieces,
        &new_pieces,
        &builds.pieces,
        &data.costs,
        &data.tun,
    );
    for p in &d.marked {
        new_body.marked.insert(p.id(), p.clone());
    }
    // A marked piece the new plan wants back as it stands is kept.
    new_body
        .marked
        .retain(|pid, p| !new_pieces.iter().any(|n| n.id() == *pid && n == p));
    let planned = setting(save) == BuildingSetting::Planned;
    if data.tun.removal == pbd_core::building::planner::Removal::Return {
        // Down at once, top first, the goods to the pack.
        let mut order: Vec<Piece> = new_body.marked.values().cloned().collect();
        order.sort_by_key(|p| std::cmp::Reverse(p.shape.layer()));
        for p in order {
            let back = if planned {
                pbd_core::building::costs::refund(
                    &data
                        .costs
                        .cost_of(&p, data.tun.cost_scale)
                        .unwrap_or_default(),
                    data.tun.refund_share,
                )
            } else {
                Vec::new()
            };
            if let Some(after) = give_goods(&hotbar.0, &back)
                && builds.remove_with_hands(save, p.id(), &after)
            {
                hotbar.0 = after;
                new_body.marked.remove(&p.id());
            }
        }
    }
    let written = if id == 0 {
        set_out(save, &new_body).is_some()
    } else {
        store_site(save, id, &new_body)
    };
    if written {
        info!(
            "SITE {id} changed: {} new, {} dropped, {} to take down",
            d.new.len(),
            d.gone.len(),
            d.marked.len()
        );
        adopting.0 = None;
        planner.editing = None;
        *screen = Screen::Playing;
    }
}

/// O in build mode (design 3 and 7): aimed at a site, its plan in the
/// planner to change; aimed at a house outside any site, the house read as
/// a plan; anywhere else the planner as it was left.
#[allow(clippy::too_many_arguments)]
pub fn open_from_build_mode(
    mut mode: ResMut<BuildMode>,
    mut screen: ResMut<Screen>,
    mut planner: ResMut<Planner>,
    sites: Res<Sites>,
    builds: Res<Builds>,
    contact: Option<Res<PlanetContact>>,
    mut adopting: ResMut<Adopting>,
    (fine, save): (Option<Res<PlanetFine>>, Option<Res<WorldSave>>),
    data: Res<PlannerData>,
    mut cache: Local<GroundCache>,
) {
    if !std::mem::take(&mut mode.plan_asked) {
        return;
    }
    let cell = mode
        .aim
        .zip(contact.as_deref())
        .and_then(|(p, c)| c.finest_key(p.normalize_or(Vec3::Y)));
    // A site with a piece in the aimed cell.
    let site = cell.and_then(|cell| {
        sites.by_id.iter().find(|(_, s)| {
            s.charted
                .iter()
                .any(|(_, p)| p.shape.cells().contains(&cell))
        })
    });
    if let Some((id, s)) = site {
        planner.editing = Some(*id);
        let name = if s.body.name.starts_with("blueprint.") || s.body.name.starts_with("planner.") {
            PlanName::Key(s.body.name.clone())
        } else {
            PlanName::Mine(s.body.name.clone())
        };
        open_planner(
            &mut planner,
            &mut screen,
            Some((s.body.plan.clone(), name, s.body.preset.clone())),
        );
        info!("PLANNER opened on site {id}");
        return;
    }
    // A house outside any site: adopted.
    let house = cell.and_then(|cell| {
        builds
            .pieces
            .values()
            .find(|p| p.shape.cells().contains(&cell))
            .cloned()
    });
    if let (Some(piece), Some(fine), Some(save)) = (house, fine.as_deref(), save.as_deref()) {
        let origin = piece.shape.cells().into_iter().min().unwrap_or(0);
        let direction = pbd_core::cell_key::key_direction(origin).unwrap_or(Vec3::Y);
        let near = builds.near(direction, pbd_core::building::PLOT_M);
        let base = near.iter().map(|p| p.shape.layer()).min().unwrap_or(0);
        let patch = patch_round(direction, PATCH_REACH_M);
        if let Some(at) = patch.keys.iter().position(|&k| k == origin) {
            let plan = adopt(&patch, at, base, &near);
            let _ = ground_top(fine, save, &mut cache, origin);
            if !plan.is_empty() {
                adopting.0 = Some(SiteBody {
                    name: "planner.adopted".into(),
                    preset: None,
                    plan: plan.clone(),
                    chart: SiteChart {
                        origin,
                        turn: 0,
                        base,
                    },
                    marked: BTreeMap::new(),
                    scaffold: Default::default(),
                    cleared: false,
                });
                planner.editing = Some(0);
                open_planner(
                    &mut planner,
                    &mut screen,
                    Some((plan, PlanName::Key("planner.adopted".into()), None)),
                );
                info!("PLANNER adopted the house at cell {origin}");
                return;
            }
        }
    }
    let first = planner.plan.is_empty().then(|| {
        let preset = &data.presets.presets[1.min(data.presets.presets.len() - 1)];
        (
            preset.plan(Size::Standard),
            PlanName::Key(format!("blueprint.preset.{}", preset.id)),
            Some((preset.id.clone(), Size::Standard)),
        )
    });
    open_planner(&mut planner, &mut screen, first);
}

/// The HUD's site panel (design 6): the nearest site's name, the count
/// left and the goods still needed against the pack, each by its icon.
pub fn spawn_hud(mut commands: Commands, kit: Res<UiKit>) {
    commands
        .spawn((
            kit.nine(Set::Stone, "frame"),
            Node {
                position_type: PositionType::Absolute,
                top: px(130),
                right: px(8),
                padding: UiRect::all(px(8)),
                flex_direction: FlexDirection::Column,
                row_gap: px(2),
                min_width: px(120),
                display: Display::None,
                ..default()
            },
            SiteHud,
            menu::InPlayHud,
        ))
        .with_children(|panel| {
            panel.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(2),
                    ..default()
                },
                SiteHudLines,
            ));
        });
}

/// Fill the HUD's site panel while the player is near an unfinished site.
#[allow(clippy::too_many_arguments)]
pub fn paint_hud(
    mut commands: Commands,
    sites: Res<Sites>,
    builds: Res<Builds>,
    data: Res<PlannerData>,
    hotbar: Res<Hotbar>,
    icons: Res<ItemIcons>,
    images: Res<Assets<Image>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    frame: Option<Res<PlanetRenderFrame>>,
    mut hud: Query<&mut Node, With<SiteHud>>,
    lines: Query<(Entity, Option<&Children>), With<SiteHudLines>>,
    mut shown: Local<Option<(u64, u64, u64, pbd_core::inventory::Slots)>>,
) {
    let Some((eye, _)) = cameras.iter().find(|(_, c)| c.is_active) else {
        return;
    };
    let eye = pbd_app::render_origin::local_of(frame.as_deref(), eye.translation());
    let nearest = sites
        .by_id
        .iter()
        .filter_map(|(id, s)| {
            let d = pbd_core::cell_key::key_direction(s.body.chart.origin)?;
            let dist = (d * eye.length()).distance(eye);
            (dist <= data.tun.ghost_reach_m).then_some((dist, *id, s))
        })
        .min_by(|a, b| a.0.total_cmp(&b.0));
    let key = (
        nearest.map_or(0, |n| n.1),
        builds.revision,
        sites.revision,
        hotbar.0.clone(),
    );
    if shown.as_ref() == Some(&key) {
        return;
    }
    *shown = Some(key);
    let Ok(mut node) = hud.single_mut() else {
        return;
    };
    let Some((_, _, site)) = nearest else {
        node.display = Display::None;
        return;
    };
    let p = progress(&site.charted, site.body.marked.len(), &builds.pieces);
    if p.finished {
        node.display = Display::None;
        return;
    }
    node.display = Display::Flex;
    let Ok((lines, children)) = lines.single() else {
        return;
    };
    if let Some(children) = children {
        for c in children.iter() {
            commands.entity(c).despawn();
        }
    }
    let need =
        still_needed(&site.charted, &builds.pieces, &data.costs, &data.tun).unwrap_or_default();
    commands.entity(lines).with_children(|col| {
        col.spawn(
            KitText::new(site_display_name(&site.body.name), ui_kit::TITLE).shadow(ui_kit::INK),
        );
        col.spawn(
            KitText::new(
                tr_with(
                    "site.left",
                    &[("left", &(p.total - p.built)), ("total", &p.total)],
                ),
                ui_kit::TEXT,
            )
            .shadow(ui_kit::INK),
        );
        if !site.body.marked.is_empty() {
            col.spawn(
                KitText::new(
                    tr_with("site.to_take_down", &[("count", &site.body.marked.len())]),
                    ui_kit::TEXT,
                )
                .shadow(ui_kit::INK),
            );
        }
        for (good, n) in &need {
            let item = pbd_core::station::item_named(good);
            let have = item.map_or(0, |i| hotbar.0.count(i));
            col.spawn(Node {
                column_gap: px(3),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|row| {
                let mut icon = ImageNode::default();
                if let Some(item) = item {
                    paint_icon(&mut icon, Some(Stack::new(item, 1)), &icons, &images);
                }
                row.spawn((
                    icon,
                    Node {
                        width: px(16),
                        height: px(16),
                        ..default()
                    },
                ));
                let colour = if have < *n {
                    Color::srgb(1.0, 0.45, 0.36)
                } else {
                    ui_kit::TEXT
                };
                row.spawn(
                    KitText::new(
                        tr_with("site.have", &[("have", &have), ("count", n)]),
                        colour,
                    )
                    .shadow(ui_kit::INK),
                );
            });
        }
    });
}

/// Set out the plan in the Blueprint slot where build mode aims (design 5):
/// charted on the ground there, checked whole, then the site's record
/// written; nothing is taken. Called by build mode's click.
#[allow(clippy::too_many_arguments)]
pub fn set_out_here(
    save: &mut WorldSave,
    hand: &InHand,
    chart: SiteChart,
    in_hand: &mut PlanInHand,
) -> Option<u64> {
    let body = SiteBody {
        name: hand.name.clone(),
        preset: hand.preset.clone(),
        plan: hand.plan.clone(),
        chart,
        marked: BTreeMap::new(),
        scaffold: Default::default(),
        cleared: false,
    };
    let id = set_out(save, &body)?;
    if in_hand.0.as_ref() == Some(hand) {
        in_hand.0 = None;
    }
    info!(
        "SITE {id} set out at cell {} turn {}",
        chart.origin, chart.turn
    );
    Some(id)
}

/// The construction systems.
pub fn plugin(app: &mut App) {
    app.init_resource::<PlannerData>()
        .init_resource::<Sites>()
        .init_resource::<PlanInHand>()
        .init_resource::<PlanChoices>()
        .init_resource::<ChangeAsked>()
        .init_resource::<Adopting>()
        .init_resource::<SiteTick>()
        .init_resource::<Striking>()
        .init_resource::<Scaffolds>()
        .init_resource::<Planner>()
        .init_resource::<super::planner::TownPlans>()
        .add_systems(Startup, spawn_ghosts)
        .add_systems(
            PostStartup,
            (super::planner::spawn, spawn_hud).after(super::slots::load_icons),
        )
        .add_systems(
            Update,
            (
                follow,
                choices,
                super::planner::load_town_plans,
                open_from_build_mode,
                super::planner::use_table
                    .in_set(pbd_app::interact::InteractSet::Offer)
                    .run_if(resource_exists::<PlanetFine>),
                super::planner::press,
                super::planner::input,
                super::planner::refresh,
                super::planner::dress_icons,
                super::planner::paint,
                super::planner::forget,
                apply_change.run_if(resource_exists::<PlanetFine>),
            )
                .chain(),
        )
        .add_systems(
            Update,
            (
                strike
                    .in_set(pbd_app::interact::InteractSet::Offer)
                    .run_if(resource_exists::<PlanetFine>),
                scaffolding,
                draw_ghosts
                    .run_if(resource_exists::<pbd_app::towns::TownAssets>)
                    .before(pbd_app::builds::sync),
                paint_hud,
            )
                .chain()
                .after(follow),
        )
        // `PBD_PLAN_SHOT`, a capture instrument (task 3.7).
        .add_systems(
            Update,
            scripted
                .run_if(resource_exists::<super::CaptureState>.and(resource_exists::<PlanetFine>))
                .before(follow),
        );
}

/// `PBD_PLAN_SHOT`: a capture instrument (`building-planner` task 3.7),
/// never read in play. A comma list, done once at its frame (`at:<frame>`,
/// default 40), each step through the same paths as play:
/// - `give`: goods, a mallet and lanterns into the pack (logged);
/// - `table`: a drafting table set down `ahead` metres along the look;
/// - `planner:<preset>`, `size:<0-2>`, `storey:<n>`, `iso`, `tool:<n>`,
///   `hover:<q>:<r>:<side>`, `upstairs`: the planner open on a preset (a
///   storey added with no stair for `upstairs`, to show a check);
/// - `site:<preset>`, `turn:<n>`, `ahead:<m>`: that preset set out ahead,
///   checked whole as build mode's click is;
/// - `build:<n>`: a frame later, the first `n` buildable pieces of the site
///   struck, each with its goods from the pack, as the mallet would;
/// - `change`: then the site's plan with its last ground cell taken out,
///   saved as the planner's Save change is;
/// - `view`, `dist:<m>`, `tilt:<degrees>`, `yaw:<degrees>`: architecture
///   mode's camera over the site.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn scripted(
    state: Res<super::CaptureState>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    frame: Option<Res<PlanetRenderFrame>>,
    (mut fine, mut contact, mut save, mut drops): (
        ResMut<PlanetFine>,
        ResMut<PlanetContact>,
        ResMut<WorldSave>,
        ResMut<pbd_app::drops::Drops>,
    ),
    (mut hotbar, mut builds, data, mut sites): (
        ResMut<Hotbar>,
        ResMut<Builds>,
        Res<PlannerData>,
        ResMut<Sites>,
    ),
    (mut planner, mut screen, mut change, mut view, mut mode): (
        ResMut<Planner>,
        ResMut<Screen>,
        ResMut<ChangeAsked>,
        ResMut<pbd_app::build_view::BuildView>,
        ResMut<BuildMode>,
    ),
    mut cache: Local<GroundCache>,
    mut done: Local<u8>,
) {
    let Ok(plan) = std::env::var("PBD_PLAN_SHOT") else {
        return;
    };
    let value = |key: &str| {
        plan.split(',')
            .find_map(|step| step.strip_prefix(&format!("{key}:")))
            .map(str::to_string)
    };
    let has = |key: &str| plan.split(',').any(|s| s.split(':').next() == Some(key));
    let num = |key: &str, or: f32| value(key).and_then(|v| v.parse::<f32>().ok()).unwrap_or(or);
    let at = num("at", 40.0) as u32;
    if *done >= 2 || state.frame < at + u32::from(*done) * 4 {
        return;
    }
    let Some((transform, _)) = cameras.iter().find(|(_, c)| c.is_active) else {
        return;
    };
    let eye = pbd_app::render_origin::local_of(frame.as_deref(), transform.translation());
    let up = eye.normalize_or(Vec3::Y);
    let look = transform.forward().as_vec3();
    let level = (look - up * look.dot(up)).normalize_or(up.any_orthonormal_vector());
    let ahead = num("ahead", 12.0);
    let target = eye + level * ahead;
    let found = contact
        .finest_cell(target.normalize())
        .and_then(|r| fine.set.finest_records().get(r).map(|c| c.key()))
        .or_else(|| contact.finest_key(target.normalize()));
    // The fine set is not round the eye yet: asked again next frame. The
    // planner alone needs no cell.
    let needs_cell = ["table", "site", "view", "build", "change"]
        .iter()
        .any(|k| has(k));
    let cell = match found {
        Some(c) => c,
        None if needs_cell => return,
        None => 0,
    };
    let size = Size::ALL[(num("size", 1.0) as usize).min(2)];
    if *done == 0 {
        *done = 1;
        if has("give") {
            use pbd_core::goods::Good;
            for (good, n) in [
                (Good::PLANKS, 99),
                (Good::PLANKS, 99),
                (Good::PLANKS, 99),
                (Good::LOOSE_STONE, 99),
                (Good::LOOSE_STONE, 99),
                (Good::FIBRE, 60),
                (Good::LOG, 30),
                (Good::ROPE, 30),
                (Good::MALLET, 1),
            ] {
                hotbar.0.give(Item::Good(good), n);
            }
            hotbar.0.give(Item::Block(Material::LanternPost), 6);
            info!("PBD_PLAN_SHOT: goods, a mallet and lanterns put in the pack");
        }
        if has("table")
            && let Some(g) = ground_top(&fine, &save, &mut cache, cell)
        {
            super::digging::apply_edit(
                &mut super::digging::Edited {
                    fine: &mut fine,
                    contact: &mut contact,
                    save: &mut save,
                    slots: &mut hotbar,
                    drops: &mut drops,
                    now_s: 0.0,
                },
                pbd_core::accept::Hands::Empty,
                cell,
                usize::from(g),
                Material::DraftingTable,
            );
            info!("PBD_PLAN_SHOT: a drafting table at cell {cell} layer {g}");
        }
        if let Some(id) = value("planner")
            && let Some(preset) = data.presets.get(&id).cloned()
        {
            let mut p = preset.plan(size);
            if has("upstairs") {
                p.add_storey();
                p.storeys[1].insert((3, 0));
            }
            planner.size = size;
            open_planner(
                &mut planner,
                &mut screen,
                Some((
                    p,
                    PlanName::Key(format!("blueprint.preset.{id}")),
                    Some((id, size)),
                )),
            );
            planner.iso = has("iso");
            planner.storey = (num("storey", 0.0) as u8).min(planner.plan.storeys.len() as u8 - 1);
            planner.tool = (num("tool", 0.0) as usize).min(super::planner::TOOLS.len() - 1);
            if let Some(h) = value("hover") {
                let n: Vec<i32> = h.split(':').filter_map(|v| v.parse().ok()).collect();
                if let [q, r, side] = n[..] {
                    planner.hover = Some(pbd_core::building::plan::Aim {
                        cell: (q, r),
                        side: side as u8,
                        at_edge: true,
                    });
                }
            }
            info!("PBD_PLAN_SHOT: the planner open");
        }
        if let Some(id) = value("site")
            && let Some(preset) = data.presets.get(&id).cloned()
        {
            let hand = InHand {
                plan: preset.plan(size),
                name: format!("blueprint.preset.{id}"),
                preset: Some((id.clone(), size)),
            };
            let turn = num("turn", 0.0) as usize % 6;
            match chart_here(&fine, &save, &mut cache, &hand.plan, cell, turn) {
                Some((patch, _, base, charted)) => {
                    for (_, p) in &charted {
                        if let Shape::Floor { cell: c, layer } = p.shape
                            && layer == base
                        {
                            let g = ground_top(&fine, &save, &mut cache, c);
                            let tree = g.is_some_and(|g| {
                                super::digging::under_standing_tree(
                                    &fine,
                                    &save,
                                    super::farm::axe_logs(None),
                                    (c, usize::from(g.saturating_sub(1))),
                                )
                            });
                            info!(
                                "PBD_PLAN_SHOT: floor cell {c} ground {g:?} base {base} tree {tree}"
                            );
                        }
                    }
                    let near = builds.near(target, PATCH_REACH_M);
                    match check_here(
                        &fine, &save, &mut cache, &patch, &charted, base, &near, &data, &hand.plan,
                    ) {
                        Ok(()) => {
                            let chart = SiteChart {
                                origin: cell,
                                turn: turn as u8,
                                base,
                            };
                            let mut none = PlanInHand::default();
                            set_out_here(&mut save, &hand, chart, &mut none);
                        }
                        Err((why, hit)) => warn!(
                            "PBD_PLAN_SHOT: refused here: {} on {} pieces",
                            why.key(),
                            hit.len()
                        ),
                    }
                }
                None => warn!("PBD_PLAN_SHOT: the plan does not chart here"),
            }
        }
        if has("view") {
            let layer = ground_top(&fine, &save, &mut cache, cell).unwrap_or(0);
            let r = PLANET_RADIUS + pbd_core::column::layer_altitude(usize::from(layer));
            mode.on = true;
            mode.quiet = true;
            view.enter(
                (target.normalize() * r).as_dvec3(),
                layer,
                level.as_dvec3(),
                eye.as_dvec3(),
                target.as_dvec3(),
            );
            if let Some(d) = value("dist").and_then(|v| v.parse().ok()) {
                view.distance = d;
            }
            if let Some(t) = value("tilt").and_then(|v| v.parse::<f32>().ok()) {
                view.tilt = t.to_radians();
            }
            if let Some(y) = value("yaw").and_then(|v| v.parse::<f32>().ok()) {
                view.orbit(y.to_radians(), 0.0);
            }
        }
        return;
    }
    // Frames later, once the site is read back: build and change it.
    *done = 2;
    sites.follow(&save);
    let Some((&id, site)) = sites.by_id.iter().next_back() else {
        return;
    };
    let site = site.clone();
    let n = num("build", 0.0) as usize;
    let cells = Cells::new(&site.patch);
    let mut built = 0;
    'passes: for _ in 0..64 {
        let mut placed_any = false;
        for (_, piece) in &site.charted {
            if built >= n {
                break 'passes;
            }
            if is_built(&builds.pieces, piece) || matches!(piece.shape, Shape::Furniture { .. }) {
                continue;
            }
            let standing: Vec<Piece> = builds.pieces.values().cloned().collect();
            let tops = tops_of(&fine, &save, &mut cache, piece.shape.cells().into_iter());
            let ground = |c: u32| tops.get(&c).copied().flatten();
            let rules = Site {
                cells: &cells,
                ground: &ground,
            };
            if !buildable(&rules, &standing, piece) {
                continue;
            }
            let goods = data
                .costs
                .cost_of(piece, data.tun.cost_scale)
                .unwrap_or_default();
            let Ok(after) = take_goods(&hotbar.0, &goods) else {
                warn!("PBD_PLAN_SHOT: the pack ran short");
                break 'passes;
            };
            if builds.place_with_hands(&mut save, piece.clone(), &after) {
                hotbar.0 = after;
                built += 1;
                placed_any = true;
            }
        }
        if !placed_any {
            break;
        }
    }
    info!("PBD_PLAN_SHOT: struck {built} pieces of site {id}");
    if has("change") {
        let mut p = site.body.plan.clone();
        if let Some(&last) = p.storeys[0].iter().next_back() {
            for s in &mut p.storeys {
                s.remove(&last);
            }
            p.prune();
        }
        change.0 = Some((id, p));
        info!("PBD_PLAN_SHOT: the site's plan changed");
    }
}
