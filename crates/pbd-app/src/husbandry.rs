//! Keeping animals in the app (`animal-husbandry`, the first playable slice,
//! design decision 12): wild herds round the player, taming, fence and gate
//! pieces, pens, kept animals that eat, sleep and give, all drawn and
//! animated.
//!
//! Every rule is the core's (`pbd_core::husbandry`). This adapts them to the
//! world: which cell a point is in and its neighbours (the fine set's
//! five- and six-sided table), what grows on a cell, where the water is, the
//! clock, the hands, and the save. Every change the player makes is a `@p`
//! record line on the frame it is made; nothing here writes on its own.
//!
//! Animals near the player are entities, each one mesh rebuilt from its rig
//! every frame (S11), faded in and out through the terrain's mask. Wild ones
//! are derived from the world seed and never saved (S10).

pub mod breed;
pub mod rig;

pub use breed::BreedingPick;

use crate::config::HusbandryConfig;
use crate::field_light::{Faded, LitByField};
use crate::hotbar::Hotbar;
use crate::planet::{PLANET_RADIUS, PlanetContact, PlanetFine, PlanetRenderFrame};
use crate::saves::WorldSave;
use crate::walking::{HALF_HEIGHT, Walker, WalkingCamera, WalkingReadout};
use avian3d::prelude::Position;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use pbd_core::daylight::DAY_S;
use pbd_core::fauna::{Rng, hash2};
use pbd_core::goods::Good;
use pbd_core::husbandry::farm::{Conditions, FarmState};
use pbd_core::husbandry::record::{self, AnimalBody, FenceBody, Standing};
use pbd_core::husbandry::{
    Edge, Genome, HusbandrySettings, LandSpecies, Pen, Plan, Sex, Trust, herds, taming,
};
use pbd_core::inventory::{Item, Slots};
use pbd_core::records::Author;
use pbd_core::terrain::Material;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

/// Where the player's eye ray met the ground this frame, body-local: the
/// digging system's march, which a fence piece's edge is chosen from.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct AimedGround(pub Option<Vec3>);

/// A capture's scripted farm scene (`--husbandry <scene>`), a measurement
/// instrument: `wild`, `tame`, `pen` or `collect`.
#[derive(Resource, Clone, Debug)]
pub struct HusbandryScript {
    pub scene: String,
}

/// Something the player did with animals, as a message on the frame its
/// durable write was accepted (`starter-missions`: "tame an animal",
/// "build a pen"). What they collect is a [`crate::hotbar::Gained`] with
/// [`crate::hotbar::How::Collected`].
#[derive(Message, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Deed {
    /// A wild animal of this land species (roster index) was tamed.
    Tamed { species: u16 },
    /// A kept animal was let go inside a pen.
    Penned { species: u16 },
    /// A piece closed a pen with a gate: the pen's id and how many cells.
    PenBuilt { pen: u32, cells: usize },
}

/// The use key's reader, so other readers of E (`desktop::farm::use_key`)
/// run after it and do not act on a press it used.
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub struct HusbandryHands;

/// Whose an animal is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Who {
    /// A wild herd's member, by its seed.
    Wild { seed: u64 },
    /// A kept animal, by its record id.
    Kept { id: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Idle,
    Walk,
    Flee,
    Approach,
    Follow,
}

/// Where an animal may go.
#[derive(Clone, Debug)]
enum Range {
    /// About a herd's centre (or a loose animal's home), within this.
    Around(Vec3, f32),
    /// The cells of its pen, by centre direction, and their keys.
    Pen(Arc<Vec<Vec3>>, Arc<BTreeSet<u32>>),
}

/// One animal near the player.
struct Beast {
    who: Who,
    species: u16,
    sex: Sex,
    genome: Genome,
    /// Feet, body-local.
    pos: Vec3,
    /// Unit, along the ground.
    facing: Vec3,
    speed: f32,
    goal: Option<Vec3>,
    mode: Mode,
    mode_s: f32,
    idle_s: f32,
    phase: f32,
    swing: f32,
    head_down: f32,
    head_goal: f32,
    peck_s: f32,
    pecking: f32,
    glance: f32,
    glance_goal: f32,
    glance_s: f32,
    tail: f32,
    tail_s: f32,
    down: f32,
    trust: Trust,
    rng: Rng,
    range: Range,
    shown: f32,
    leaving: bool,
    entity: Option<(Entity, Handle<Mesh>)>,
    clock_s: f32,
    /// A kept animal's birth day, for its life stage (design 13, E1).
    born: Option<f64>,
    /// A hen sitting on a clutch (E4): she rests on it.
    sitting: bool,
}

impl Beast {
    fn new(
        who: Who,
        species: u16,
        sex: Sex,
        genome: Genome,
        pos: Vec3,
        range: Range,
        seed: u64,
    ) -> Self {
        let mut rng = Rng::new(seed);
        let up = pos.normalize_or(Vec3::Y);
        let a = rng.range(0.0, std::f32::consts::TAU);
        let east = Vec3::Y.cross(up).normalize_or(Vec3::X);
        let north = up.cross(east);
        Self {
            who,
            species,
            sex,
            genome,
            pos,
            facing: east * a.cos() + north * a.sin(),
            speed: 0.0,
            goal: None,
            mode: Mode::Idle,
            mode_s: 0.0,
            idle_s: rng.range(0.5, 3.0),
            phase: rng.range(0.0, std::f32::consts::TAU),
            swing: 0.0,
            head_down: 0.0,
            head_goal: 0.0,
            peck_s: rng.range(0.4, 2.0),
            pecking: 0.0,
            glance: 0.0,
            glance_goal: 0.0,
            glance_s: rng.range(0.8, 2.5),
            tail: 0.0,
            tail_s: rng.range(1.0, 3.0),
            down: 0.0,
            trust: Trust::default(),
            rng,
            range,
            shown: 0.0,
            leaving: false,
            entity: None,
            clock_s: 0.0,
            born: None,
            sitting: false,
        }
    }
}

/// The yard: every animal near the player, and what the pens are.
#[derive(Resource)]
pub struct Farmyard {
    beasts: Vec<Beast>,
    sync_s: f32,
    /// A cell's key to its finest record, for the set it was built from.
    index: Option<(usize, HashMap<u32, usize>)>,
    /// What the HUD line says, and how long a one-off message stays.
    status: String,
    hold_s: f32,
    fences_drawn: Option<(BTreeMap<Edge, bool>, Vec<u32>, usize)>,
    material: Option<Handle<StandardMaterial>>,
    /// What the player did this frame, announced as messages.
    deeds: Vec<Deed>,
    gained: Vec<(Item, u16)>,
}

impl Default for Farmyard {
    fn default() -> Self {
        Self {
            beasts: Vec::new(),
            sync_s: 0.0,
            index: None,
            status: String::new(),
            hold_s: 0.0,
            fences_drawn: None,
            material: None,
            deeds: Vec::new(),
            gained: Vec::new(),
        }
    }
}

impl Farmyard {
    fn say(&mut self, text: impl Into<String>, hold_s: f32) {
        self.status = text.into();
        self.hold_s = hold_s;
        info!("husbandry: {}", self.status);
    }
}

/// The fence mesh's root.
#[derive(Component)]
struct FenceRoot;

/// The HUD line.
#[derive(Component)]
struct FarmText;

/// Animals, fences and the hands that keep them.
pub struct HusbandryPlugin;

impl Plugin for HusbandryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Farmyard>()
            .init_resource::<AimedGround>()
            .init_resource::<BreedingPick>()
            .add_message::<Deed>()
            .add_message::<crate::hotbar::Gained>()
            .add_systems(Startup, spawn)
            .add_systems(
                Update,
                (
                    sync,
                    hands.in_set(HusbandryHands),
                    script,
                    announce,
                    step,
                    draw,
                    draw_fences,
                    hud,
                )
                    .chain()
                    .after(crate::fish::FishSet),
            );
    }
}

/// The world as the yard reads it.
#[derive(bevy::ecs::system::SystemParam)]
struct World<'w, 's> {
    config: Res<'w, HusbandryConfig>,
    body: Res<'w, crate::fish::Body>,
    fine: Option<Res<'w, PlanetFine>>,
    contact: Option<Res<'w, PlanetContact>>,
    frame: Res<'w, PlanetRenderFrame>,
    sun: Option<Res<'w, crate::sky::Sun>>,
    sea: Option<Res<'w, crate::sea::Sea>>,
    walking: Option<Res<'w, WalkingReadout>>,
    time: Res<'w, Time>,
    walkers: Query<'w, 's, &'static Position, With<Walker>>,
    cameras: Query<'w, 's, &'static GlobalTransform, With<WalkingCamera>>,
}

impl World<'_, '_> {
    fn settings(&self) -> &HusbandrySettings {
        &self.config.0
    }

    fn roster(&self) -> &[LandSpecies] {
        self.config.0.roster(&self.body.0)
    }

    /// The player's feet, body-local.
    fn feet(&self) -> Option<Vec3> {
        let p = self.walkers.iter().next()?;
        let at = (p.0.as_dvec3() - self.frame.center).as_vec3();
        Some(at - at.normalize_or(Vec3::Y) * HALF_HEIGHT)
    }

    /// The eye and the look, body-local.
    fn eye(&self) -> Option<(Vec3, Vec3)> {
        let t = self.cameras.iter().next()?;
        Some((
            (t.translation().as_dvec3() - self.frame.center).as_vec3(),
            t.forward().as_vec3(),
        ))
    }

    /// World days now.
    fn day(&self) -> f64 {
        self.sun.as_ref().map_or(0.0, |s| s.clock.seconds) / f64::from(DAY_S)
    }

    fn night_at(&self, up: Vec3) -> bool {
        self.sun
            .as_ref()
            .is_some_and(|s| s.clock.daylight(up) < 0.12)
    }

    /// Ground radius under a direction: the column's floor where the fine
    /// tier stands, else the generated surface.
    fn ground(&self, d: Vec3) -> f32 {
        let d = d.normalize_or(Vec3::Y);
        let surface = PLANET_RADIUS + crate::planet::surface_height(d).max(0.0);
        match self.contact.as_deref() {
            Some(c) => c.stand(d * (surface + 1.5)).floor_radius,
            None => surface,
        }
    }

    /// The finest record a direction falls in, and its key.
    fn cell(&self, d: Vec3) -> Option<(usize, u32)> {
        let fine = self.fine.as_deref()?;
        let contact = self.contact.as_deref()?;
        if !contact.serves(&fine.set) {
            return None;
        }
        let i = contact.finest_cell(d.normalize_or(Vec3::Y))?;
        Some((i, fine.set.finest_records().get(i)?.key()))
    }
}

/// The yard's material and the prewarm template: an animal's mesh, so its
/// pipeline (field-lit, vertex colours) is built behind the loading screen.
fn spawn(
    mut commands: Commands,
    mut yard: ResMut<Farmyard>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<HusbandryConfig>,
) {
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.9,
        reflectance: 0.2,
        ..default()
    });
    yard.material = Some(material.clone());
    let mut builder = rig::Builder::default();
    if let Some(sp) = config.0.roster(pbd_core::fauna::HOME_BODY).first() {
        rig::build(
            &mut builder,
            sp,
            &Genome::default(),
            Sex::F,
            &rig::Pose::default(),
            [Vec3::X, Vec3::Y, Vec3::Z],
        );
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    builder.write(&mut mesh);
    commands
        .spawn((
            Name::new("Husbandry prewarm template"),
            LitByField,
            Faded(1.0),
            Transform::from_xyz(0.0, -1.0e5, 0.0),
            Visibility::Hidden,
        ))
        .with_child((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material.clone()),
            Transform::default(),
        ));
    commands.spawn((
        Name::new("Fences"),
        FenceRoot,
        LitByField,
        Transform::default(),
        Visibility::default(),
    ));
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: px(78),
            left: percent(50),
            width: px(320),
            margin: UiRect::left(px(-160)),
            justify_content: JustifyContent::Center,
            ..default()
        },
        Text::new(""),
        TextFont {
            font_size: 11.0,
            ..default()
        },
        TextColor(Color::srgb(0.86, 0.93, 0.74)),
        TextLayout::new_with_justify(Justify::Center),
        crate::ui_kit::text_shadow(),
        FarmText,
    ));
}

/// The pen a cell is in, on the resident fine set.
fn pen_at(
    world: &World,
    yard: &mut Farmyard,
    pieces: &BTreeMap<Edge, bool>,
    cell: u32,
) -> Option<Pen> {
    let fine = world.fine.as_deref()?;
    let set = &fine.set;
    let ptr = Arc::as_ptr(set) as usize;
    if yard.index.as_ref().is_none_or(|(p, _)| *p != ptr) {
        let map = set
            .finest_records()
            .iter()
            .enumerate()
            .map(|(i, r)| (r.key(), i))
            .collect();
        yard.index = Some((ptr, map));
    }
    let (_, index) = yard.index.as_ref()?;
    let neighbours = |key: u32| -> Option<Vec<u32>> {
        let &i = index.get(&key)?;
        let record = set.finest_records().get(i)?;
        let table = set.finest_neighbors.get(i)?;
        table[..record.degree()]
            .iter()
            .map(|&n| set.finest_records().get(n as usize).map(|r| r.key()))
            .collect()
    };
    pbd_core::husbandry::find_pen(cell, neighbours, pieces, 64).ok()
}

/// What a pen gives the animals in it, as the world says now (S8).
fn conditions(world: &World, pen: Option<&Pen>, herd: &[(u16, Sex)]) -> Conditions {
    let roster = world.roster();
    let need = |f: fn(&pbd_core::husbandry::Keeping) -> f32| -> f32 {
        herd.iter()
            .filter_map(|(sp, _)| roster.get(*sp as usize)?.keep.as_ref().map(f))
            .sum()
    };
    let Some(pen) = pen else {
        // Loose in the yard: room enough, nothing to graze, no trough.
        return Conditions {
            space: true,
            ..Conditions::default()
        };
    };
    let Some(fine) = world.fine.as_deref() else {
        return Conditions::default();
    };
    let set = &fine.set;
    let mut grass = 0.0;
    let mut water = false;
    let records = set.finest_records();
    for (i, r) in records.iter().enumerate() {
        if !pen.cells.contains(&r.key()) {
            continue;
        }
        let top = set
            .columns
            .column(i)
            .and_then(|c| c.surface().map(|l| c.material(l)));
        if matches!(
            top,
            Some(Material::Grass | Material::DryGrass | Material::JungleGrass)
        ) {
            grass += 1.0;
        }
        if let (Some(sea), Some(table)) = (world.sea.as_deref(), set.finest_neighbors.get(i)) {
            water |= table[..r.degree()].iter().any(|&n| {
                records.get(n as usize).is_some_and(|nr| {
                    sea.depth_at(Vec3::from_slice(&nr.direction_height[..3])) > 0.05
                })
            });
        }
    }
    Conditions {
        grazing: grass >= need(|k| k.grazing_cells),
        water,
        shelter: false,
        space: pen.cells.len() as f32 >= need(|k| k.space_cells),
    }
}

/// A kept animal's farm state now, from its record and its pen.
fn farm_now(
    world: &World,
    yard: &mut Farmyard,
    save: &WorldSave,
    id: u64,
    body: &AnimalBody,
) -> FarmState {
    farm_and_conditions(world, yard, save, id, body).0
}

/// [`farm_now`], and the conditions its pen gives now.
fn farm_and_conditions(
    world: &World,
    yard: &mut Farmyard,
    save: &WorldSave,
    id: u64,
    body: &AnimalBody,
) -> (FarmState, Conditions) {
    let pieces = record::pieces(&save.records);
    let pen = body
        .pen
        .and_then(|_| pen_at(world, yard, &pieces, body.home));
    let herd: Vec<(u16, Sex)> = match &pen {
        Some(p) => record::animals(&save.records)
            .into_iter()
            .filter(|(_, b)| b.pen.is_some() && p.cells.contains(&b.home))
            .map(|(_, b)| (b.sp, b.sex))
            .collect(),
        None => vec![(body.sp, body.sex)],
    };
    let c = conditions(world, pen.as_ref(), &herd);
    let roster = world.roster();
    let Some(keep) = roster.get(body.sp as usize).and_then(|s| s.keep.as_ref()) else {
        return (body.farm(), c);
    };
    // A young animal gives from adulthood, a hen that sat from the hatching
    // (design 13, E1, E4).
    let from = breed::productive_from(&save.records, id, body, keep);
    let state = body.farm().advance_from(
        world.day(),
        c,
        keep,
        body.sex,
        body.genome().yield_factor(),
        &world.settings().needs,
        from,
    );
    (state, c)
}

/// The centres of a pen's cells, for an animal to wander between.
fn pen_range(world: &World, pen: &Pen) -> Option<Range> {
    let fine = world.fine.as_deref()?;
    let cells: BTreeSet<u32> = pen.cells.iter().copied().collect();
    let centres: Vec<Vec3> = fine
        .set
        .finest_records()
        .iter()
        .filter(|r| cells.contains(&r.key()))
        .map(|r| Vec3::from_slice(&r.direction_height[..3]))
        .collect();
    (!centres.is_empty()).then(|| Range::Pen(Arc::new(centres), Arc::new(cells)))
}

/// Keep the yard's animals the ones near the player: wild herds derived
/// here, kept animals read from the records. Twice a second.
fn sync(world: World, mut save: Option<ResMut<WorldSave>>, mut yard: ResMut<Farmyard>) {
    let dt = world.time.delta_secs().min(0.1);
    yard.sync_s -= dt;
    if yard.sync_s > 0.0 {
        return;
    }
    yard.sync_s = 0.5;
    let (Some(feet), Some(save)) = (world.feet(), save.as_deref_mut()) else {
        return;
    };
    if world.cell(feet).is_none() {
        return;
    }
    // The farm's own births and conceptions up to now (design 13, E5, E6).
    breed::realise(&world, &mut yard, save);
    let s = world.settings().clone();
    let near = s.herds.near_m;
    let up = feet.normalize_or(Vec3::Y);
    let kept = record::animals(&save.records);
    let tamed: BTreeSet<u64> = kept.iter().map(|(_, b)| b.wild).collect();
    let mut wanted: Vec<Who> = Vec::new();
    // Kept animals near.
    let pieces = record::pieces(&save.records);
    for (id, body) in &kept {
        if body.state != record::Life::Alive || body.body != world.body.0 {
            continue;
        }
        let Some(home) = pbd_core::cell_key::key_direction(body.home) else {
            continue;
        };
        if home.angle_between(up) * PLANET_RADIUS > near {
            continue;
        }
        let who = Who::Kept { id: *id };
        wanted.push(who);
        let pen = body
            .pen
            .and_then(|_| pen_at(&world, &mut yard, &pieces, body.home));
        let range = pen
            .as_ref()
            .and_then(|p| pen_range(&world, p))
            .unwrap_or(Range::Around(home, 6.0));
        let sitting = breed::sitting(&save.records, *id);
        if let Some(b) = yard.beasts.iter_mut().find(|b| b.who == who) {
            if b.mode != Mode::Follow {
                b.range = range;
            }
            b.leaving = false;
            b.born = Some(body.born);
            b.sitting = sitting;
            continue;
        }
        // A tamed animal already standing here as wild becomes this one.
        if let Some(b) = yard
            .beasts
            .iter_mut()
            .find(|b| b.who == Who::Wild { seed: body.wild } && body.wild != 0)
        {
            b.who = who;
            b.range = range;
            b.born = Some(body.born);
            continue;
        }
        if world.roster().get(body.sp as usize).is_none() {
            continue;
        }
        let jitter = {
            let mut r = Rng::new(hash2(*id, 0xa11));
            let east = Vec3::Y.cross(home).normalize_or(Vec3::X);
            let north = home.cross(east);
            (east * r.range(-1.0, 1.0) + north * r.range(-1.0, 1.0)) * 0.8
        };
        let d = (home * PLANET_RADIUS + jitter).normalize();
        let pos = d * world.ground(d);
        let mut beast = Beast::new(
            who,
            body.sp,
            body.sex,
            body.genome(),
            pos,
            range,
            hash2(*id, 0xbea57),
        );
        beast.born = Some(body.born);
        beast.sitting = sitting;
        yard.beasts.push(beast);
    }
    // Wild herds near, on dry grass.
    let roster: Vec<LandSpecies> = world.roster().to_vec();
    let seed = crate::planet::terrain_config().seed;
    for square in herds::squares_near(up, near, PLANET_RADIUS, &s.herds) {
        let Some(herd) = herds::herd_in(seed, square, PLANET_RADIUS, &s.herds, &roster) else {
            continue;
        };
        if herd.centre.angle_between(up) * PLANET_RADIUS > near + s.herds.spread_m {
            continue;
        }
        if !grassy_land(&world, herd.centre) {
            continue;
        }
        let east = Vec3::Y.cross(herd.centre).normalize_or(Vec3::X);
        let north = herd.centre.cross(east);
        for m in &herd.members {
            if tamed.contains(&m.seed) {
                continue;
            }
            let who = Who::Wild { seed: m.seed };
            wanted.push(who);
            if let Some(b) = yard.beasts.iter_mut().find(|b| b.who == who) {
                b.leaving = false;
                continue;
            }
            let d =
                (herd.centre * PLANET_RADIUS + east * m.offset.0 + north * m.offset.1).normalize();
            let sp = &roster[herd.species as usize];
            let genome = sp.keep.as_ref().map_or_else(Genome::default, |k| {
                Genome::roll(m.seed, k.coats.len(), k.patterns.len())
            });
            yard.beasts.push(Beast::new(
                who,
                herd.species,
                m.sex,
                genome,
                d * world.ground(d),
                Range::Around(herd.centre, s.herds.spread_m * 1.5),
                m.seed,
            ));
        }
    }
    // Everything else fades out. A follower and a scripted animal stay.
    for b in &mut yard.beasts {
        let scripted = matches!(b.who, Who::Wild { seed } if seed >> 56 == 0x5c);
        if !wanted.contains(&b.who) && b.mode != Mode::Follow && !scripted {
            b.leaving = true;
        }
    }
}

/// Whether a direction is dry land topped with grass, by the generator (a
/// herd's centre may be beyond the resident fine set).
fn grassy_land(world: &World, d: Vec3) -> bool {
    if crate::planet::surface_height(d) <= 0.5 {
        return false;
    }
    if world.sea.as_deref().is_some_and(|s| s.depth_at(d) > 0.0) {
        return false;
    }
    let config = crate::planet::terrain_config();
    matches!(
        pbd_core::planet_gen::top_material(
            config,
            d,
            pbd_core::planet_gen::surface_altitude(config, d)
        ),
        Material::Grass | Material::DryGrass
    )
}

/// The use key and the right button: feed, tame, collect, lead, and place
/// or take back a fence piece.
#[allow(clippy::too_many_arguments)]
fn hands(
    world: World,
    mut keys: Option<ResMut<ButtonInput<KeyCode>>>,
    buttons: Option<Res<ButtonInput<MouseButton>>>,
    menu: Option<Res<crate::controls::MenuOpen>>,
    aimed: Res<AimedGround>,
    mut yard: ResMut<Farmyard>,
    mut hotbar: Option<ResMut<Hotbar>>,
    mut save: Option<ResMut<WorldSave>>,
) {
    let dt = world.time.delta_secs().min(0.1);
    yard.hold_s = (yard.hold_s - dt).max(0.0);
    let (Some(hotbar), Some(save)) = (hotbar.as_deref_mut(), save.as_deref_mut()) else {
        return;
    };
    // A follower whose time is up stays where it is.
    let done: Vec<usize> = yard
        .beasts
        .iter()
        .enumerate()
        .filter(|(_, b)| b.mode == Mode::Follow && b.mode_s <= 0.0)
        .map(|(k, _)| k)
        .collect();
    for k in done {
        settle(&world, &mut yard, save, k);
    }
    let walking = world
        .walking
        .as_ref()
        .is_some_and(|w| w.active && w.captured);
    if !walking || menu.as_ref().is_some_and(|m| m.0) {
        return;
    }
    let used = keys.as_ref().is_some_and(|k| k.just_pressed(KeyCode::KeyE));
    let placed = buttons
        .as_ref()
        .is_some_and(|b| b.just_pressed(MouseButton::Right));
    if placed
        && let Some(Item::Good(good)) = hotbar.held().map(|s| s.item)
        && (good == Good::FENCE || good == Good::GATE)
    {
        if let Some(point) = aimed.0 {
            place_piece(&world, &mut yard, hotbar, save, point, good == Good::GATE);
        }
        return;
    }
    if placed && hotbar.held().map(|s| s.item) == Some(Item::Good(Good::INCUBATOR)) {
        if let Some(point) = aimed.0 {
            breed::place_incubator(&world, &mut yard, hotbar, save, point);
        }
        return;
    }
    if !used {
        return;
    }
    let used_it = if let Some(k) = aimed_beast(&world, &yard) {
        use_on(&world, &mut yard, hotbar, save, k);
        true
    } else if let Some(point) = aimed.0 {
        breed::use_incubator(&world, &mut yard, hotbar, save, point)
            || take_piece(&world, &mut yard, hotbar, save, point)
    } else {
        false
    };
    // A press used on an animal or a piece is not also a door's or a crop's.
    if used_it && let Some(keys) = keys.as_deref_mut() {
        keys.clear_just_pressed(KeyCode::KeyE);
    }
}

/// The animal the player is looking at within reach, if any.
fn aimed_beast(world: &World, yard: &Farmyard) -> Option<usize> {
    let (eye, look) = world.eye()?;
    let reach = world.settings().taming.reach_m;
    let roster = world.roster();
    yard.beasts
        .iter()
        .enumerate()
        .filter(|(_, b)| !b.leaving && b.shown > 0.5)
        .filter_map(|(k, b)| {
            let sp = roster.get(b.species as usize)?;
            let centre = b.pos + b.pos.normalize_or(Vec3::Y) * sp.height_m * 0.5;
            let to = centre - eye;
            let d = to.length();
            let off = (to - look * to.dot(look)).length();
            (to.dot(look) > 0.0 && d < reach + sp.length_m && off < sp.length_m.max(0.5))
                .then_some((k, d))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(k, _)| k)
}

/// Note the record kinds this world now holds, once (`identity.ron`).
fn note_kinds(save: &mut WorldSave) {
    save.note_record_kinds(&[
        (record::ANIMAL, record::ANIMAL_SCHEMA),
        (record::FENCE, record::FENCE_SCHEMA),
    ]);
}

/// Spend one of the selected stack. The caller writes the slots, with the
/// record the spending made where there is one (`store_with_hands`).
fn spend_one(hotbar: &mut Slots) -> bool {
    let at = hotbar.selected();
    hotbar.take(at, 1) > 0
}

/// Store a kept animal's record as the player's, with the slots after the
/// action in the same line where they changed.
fn store_animal(save: &mut WorldSave, body: &AnimalBody, id: u64, hands: Option<&Slots>) -> bool {
    match hands {
        Some(slots) => save.store_with_hands(body.record(id), record::ANIMAL_SCHEMA, slots),
        None => {
            note_kinds(save);
            save.store(&Author::Player(0), vec![body.record(id)])
                .is_some()
        }
    }
}

/// E on an animal.
fn use_on(world: &World, yard: &mut Farmyard, hotbar: &mut Slots, save: &mut WorldSave, k: usize) {
    let roster = world.roster().to_vec();
    let s = world.settings().clone();
    let (who, species) = (yard.beasts[k].who, yard.beasts[k].species);
    let Some(sp) = roster.get(species as usize) else {
        return;
    };
    let name = sp.name.clone();
    let held = hotbar.held().map(|st| st.item);
    match who {
        Who::Wild { .. } => {
            if let Some(why) = sp.refusal() {
                yard.say(format!("{name}: {why}"), 2.5);
                return;
            }
            let keep = sp.keep.as_ref().expect("tameable");
            if !held.is_some_and(|i| keep.eats(i)) {
                yard.say(
                    format!(
                        "{name} eats {}: hold some and stand still",
                        keep.food_name()
                    ),
                    2.5,
                );
                return;
            }
            feed_wild(world, yard, hotbar, save, k);
        }
        Who::Kept { id } => {
            let Some(rec) = save.records.get(record::ANIMAL, id).cloned() else {
                return;
            };
            let Some(mut body) = rec.read::<AnimalBody>() else {
                return;
            };
            // Fertile eggs in hand on a hen: she goes broody (design 13, E4).
            if held == Some(Item::Good(Good::FERTILE_EGG))
                && breed::set_under_hen(world, yard, hotbar, save, id, &mut body)
            {
                return;
            }
            let mut now = farm_now(world, yard, save, id, &body);
            // Collect what is waiting first.
            let mut got = Vec::new();
            for p in sp.keep.iter().flat_map(|k| k.products.iter()) {
                let waiting = now.waiting(&p.good);
                let Some(mut good) = p.good() else { continue };
                if waiting == 0 {
                    continue;
                }
                // A hen's eggs with a cockerel in her pen are fertile, their
                // parentage stored as she gives them (design 13, E3).
                if good == Good::EGG
                    && let Some(keep) = sp.keep.as_ref()
                {
                    good = breed::collected_eggs(world, save, id, &body, keep, waiting);
                }
                let left = hotbar.give(Item::Good(good), waiting as u16);
                let took = waiting - u32::from(left);
                if let Some(t) = now.made.get_mut(&p.good) {
                    *t -= took as f32;
                }
                if took > 0 {
                    yard.gained.push((Item::Good(good), took as u16));
                    got.push(match (p.good.as_str(), took) {
                        ("egg", 1) if good == Good::FERTILE_EGG => "a fertile egg".to_string(),
                        ("egg", n) if good == Good::FERTILE_EGG => format!("{n} fertile eggs"),
                        ("egg", 1) => "an egg".to_string(),
                        ("egg", n) => format!("{n} eggs"),
                        ("milk", 1) => "a pail of milk".to_string(),
                        ("milk", n) => format!("{n} pails of milk"),
                        (good, n) => format!("{n} {good}"),
                    });
                }
            }
            if !got.is_empty() {
                body.set_farm(&now);
                store_animal(save, &body, id, Some(hotbar));
                yard.say(format!("{name}: collected {}", got.join(", ")), 2.5);
                return;
            }
            if held.is_some_and(|i| sp.keep.as_ref().is_some_and(|k| k.eats(i))) {
                if !spend_one(hotbar) {
                    return;
                }
                now.hand_feed(world.day(), &s.needs);
                body.set_farm(&now);
                store_animal(save, &body, id, Some(hotbar));
                yard.say(
                    format!("{name} is fed for {} days", s.needs.hand_feed_days),
                    2.5,
                );
                return;
            }
            let b = &mut yard.beasts[k];
            if b.mode == Mode::Follow {
                settle(world, yard, save, k);
            } else {
                b.mode = Mode::Follow;
                b.mode_s = s.taming.follow_s;
                yard.say(format!("{name} follows you (E to let it stay)"), 2.5);
            }
        }
    }
}

/// One feed of a wild animal: trust, and at full trust, taming.
fn feed_wild(
    world: &World,
    yard: &mut Farmyard,
    hotbar: &mut Slots,
    save: &mut WorldSave,
    k: usize,
) {
    let roster = world.roster();
    let s = world.settings();
    let b = &yard.beasts[k];
    let Some(sp) = roster.get(b.species as usize) else {
        return;
    };
    let Some(keep) = sp.keep.as_ref() else {
        return;
    };
    let Who::Wild { seed } = b.who else {
        return;
    };
    if !spend_one(hotbar) {
        return;
    }
    let factor = b.genome.trust_factor();
    let name = sp.name.clone();
    let b = &mut yard.beasts[k];
    if !b.trust.feed(keep.trust_gain, factor) {
        let pct = b.trust.value.round();
        // A feed that does not tame changes only the hands.
        save.record_pack(hotbar);
        b.mode = Mode::Idle;
        b.head_goal = 1.2;
        yard.say(format!("{name} eats from your hand - trust {pct}%"), 2.0);
        return;
    }
    // Tamed: the record, at once, as the player's (decision 10).
    let day = world.day();
    let home = world.cell(b.pos).map_or(0, |(_, key)| key);
    let mut body = AnimalBody {
        body: world.body.0.clone(),
        sp: b.species,
        sex: b.sex,
        born: day - f64::from(keep.adult_days),
        genes: b.genome.0.to_vec(),
        home,
        wild: seed,
        ..AnimalBody::default()
    };
    let mut farm = FarmState::new(day);
    farm.hand_feed(day, &s.needs);
    body.set_farm(&farm);
    let id = record::next_animal_id(&save.records);
    note_kinds(save);
    if !store_animal(save, &body, id, Some(hotbar)) {
        warn!("husbandry: the save refused the tamed {name}");
        return;
    }
    let b = &mut yard.beasts[k];
    b.who = Who::Kept { id };
    b.mode = Mode::Follow;
    b.mode_s = s.taming.follow_s;
    let species = b.species;
    yard.deeds.push(Deed::Tamed { species });
    yard.say(
        format!("{name} trusts you and follows (E to let it stay)"),
        3.0,
    );
}

/// A follower let go: it lives where it stands, in the pen there if any.
fn settle(world: &World, yard: &mut Farmyard, save: &mut WorldSave, k: usize) {
    let Who::Kept { id } = yard.beasts[k].who else {
        yard.beasts[k].mode = Mode::Idle;
        return;
    };
    let Some(mut body) = save
        .records
        .get(record::ANIMAL, id)
        .and_then(|r| r.read::<AnimalBody>())
    else {
        return;
    };
    let name = world
        .roster()
        .get(body.sp as usize)
        .map_or("It".to_string(), |s| s.name.clone());
    // The farm state up to now under the old home, then the new home.
    let now = farm_now(world, yard, save, id, &body);
    body.set_farm(&now);
    let Some((_, cell)) = world.cell(yard.beasts[k].pos) else {
        return;
    };
    let pieces = record::pieces(&save.records);
    let pen = pen_at(world, yard, &pieces, cell);
    body.home = cell;
    body.pen = pen.as_ref().map(|p| p.id);
    store_animal(save, &body, id, None);
    if pen.is_some() {
        yard.deeds.push(Deed::Penned { species: body.sp });
    }
    let herd: Vec<(u16, Sex)> = match &pen {
        Some(p) => record::animals(&save.records)
            .into_iter()
            .filter(|(_, b)| b.pen.is_some() && p.cells.contains(&b.home))
            .map(|(_, b)| (b.sp, b.sex))
            .collect(),
        None => vec![(body.sp, body.sex)],
    };
    let c = conditions(world, pen.as_ref(), &herd);
    info!(
        "husbandry: {name} {id} home {cell} pen {:?} ({} cells, herd {}): {c:?}",
        body.pen,
        pen.as_ref().map_or(0, |p| p.cells.len()),
        herd.len()
    );
    let b = &mut yard.beasts[k];
    b.mode = Mode::Idle;
    b.goal = None;
    match pen.as_ref().and_then(|p| pen_range(world, p)) {
        Some(range) => {
            b.range = range;
            yard.say(format!("{name} is penned"), 2.5);
        }
        None => {
            b.range = Range::Around(b.pos.normalize(), 6.0);
            yard.say(format!("{name} stays here, loose"), 2.5);
        }
    }
}

/// The edge of the aimed cell nearest `point`, and its two corner
/// directions.
fn edge_at(world: &World, point: Vec3) -> Option<(Edge, Vec3, Vec3)> {
    let fine = world.fine.as_deref()?;
    let (i, key) = world.cell(point)?;
    let set = &fine.set;
    let record = set.finest_records().get(i)?;
    let table = set.finest_neighbors.get(i)?;
    let d = point.normalize();
    let here = Vec3::from_slice(&record.direction_height[..3]);
    let (n, _) = table[..record.degree()]
        .iter()
        .filter_map(|&n| {
            let other = set.finest_records().get(n as usize)?;
            let mid = (here + Vec3::from_slice(&other.direction_height[..3])).normalize();
            Some((n as usize, mid.distance_squared(d)))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))?;
    let other = set.finest_records().get(n)?;
    let (a, b) = edge_corners(record, other)?;
    Some((Edge::new(key, other.key()), a, b))
}

/// The two corners two neighbouring cells share: the corners of the first
/// nearest the midpoint between them.
fn edge_corners(a: &crate::planet::GpuCell, b: &crate::planet::GpuCell) -> Option<(Vec3, Vec3)> {
    let ca = Vec3::from_slice(&a.direction_height[..3]);
    let cb = Vec3::from_slice(&b.direction_height[..3]);
    let mid = (ca + cb).normalize();
    let mut corners: Vec<Vec3> = a.corners[..a.degree()]
        .iter()
        .map(|c| Vec3::new(c[0], c[1], c[2]).normalize_or(ca))
        .collect();
    corners.sort_by(|x, y| x.distance_squared(mid).total_cmp(&y.distance_squared(mid)));
    Some((*corners.first()?, *corners.get(1)?))
}

/// Right click with a piece in hand: stand it on the aimed edge.
fn place_piece(
    world: &World,
    yard: &mut Farmyard,
    hotbar: &mut Slots,
    save: &mut WorldSave,
    point: Vec3,
    gate: bool,
) -> bool {
    let Some(feet) = world.feet() else {
        return false;
    };
    if point.distance(feet) > world.settings().taming.reach_m * 2.5 {
        yard.say("Too far to set a fence", 1.5);
        return false;
    }
    let Some((edge, _, _)) = edge_at(world, point) else {
        return false;
    };
    if record::pieces(&save.records).contains_key(&edge) {
        yard.say("A piece already stands there", 1.5);
        return false;
    }
    note_kinds(save);
    let piece = FenceBody {
        gate,
        state: Standing::Placed,
    };
    if !spend_one(hotbar)
        || !save.store_with_hands(piece.record(edge), record::FENCE_SCHEMA, hotbar)
    {
        return false;
    }
    yard.say(if gate { "Gate set" } else { "Fence set" }, 1.0);
    closed_pen(world, yard, save, edge);
    true
}

/// E on a piece: take it back. Whether there was one to take.
fn take_piece(
    world: &World,
    yard: &mut Farmyard,
    hotbar: &mut Slots,
    save: &mut WorldSave,
    point: Vec3,
) -> bool {
    let Some((edge, _, _)) = edge_at(world, point) else {
        return false;
    };
    let Some(&gate) = record::pieces(&save.records).get(&edge) else {
        return false;
    };
    let good = if gate { Good::GATE } else { Good::FENCE };
    if hotbar.give(Item::Good(good), 1) > 0 {
        yard.say("No room to carry it", 1.5);
        return true;
    }
    let piece = FenceBody {
        gate,
        state: Standing::Removed,
    };
    save.store_with_hands(piece.record(edge), record::FENCE_SCHEMA, hotbar);
    yard.say(
        if gate {
            "Gate taken up"
        } else {
            "Fence taken up"
        },
        1.0,
    );
    true
}

/// Whether a piece just set closed a pen, and if so, say so.
fn closed_pen(world: &World, yard: &mut Farmyard, save: &WorldSave, edge: Edge) {
    let pieces = record::pieces(&save.records);
    let (a, b) = edge.cells();
    for cell in [a, b] {
        if let Some(pen) = pen_at(world, yard, &pieces, cell) {
            yard.deeds.push(Deed::PenBuilt {
                pen: pen.id,
                cells: pen.cells.len(),
            });
            yard.say(
                format!("A pen of {} cells, with a gate", pen.cells.len()),
                2.0,
            );
            return;
        }
    }
}

/// Announce what the hands did this frame.
fn announce(
    mut yard: ResMut<Farmyard>,
    mut deeds: MessageWriter<Deed>,
    mut gained: MessageWriter<crate::hotbar::Gained>,
) {
    for deed in std::mem::take(&mut yard.deeds) {
        deeds.write(deed);
    }
    for (item, count) in std::mem::take(&mut yard.gained) {
        gained.write(crate::hotbar::Gained {
            item,
            count,
            how: crate::hotbar::How::Collected,
        });
    }
}

/// Move and animate every animal.
fn step(world: World, mut yard: ResMut<Farmyard>, hotbar: Option<Res<Hotbar>>) {
    let dt = world.time.delta_secs().min(0.1);
    if dt <= 0.0 {
        return;
    }
    let s = world.settings().clone();
    let roster = world.roster().to_vec();
    let feet = world.feet();
    let moving = world.walking.as_ref().is_some_and(|w| w.speed > 0.6);
    let held = hotbar.as_ref().and_then(|h| h.held()).map(|st| st.item);
    let hours = dt / DAY_S * 24.0;
    let fade = dt / s.herds.fade_s;
    for b in yard.beasts.iter_mut() {
        b.shown = if b.leaving {
            (b.shown - fade).max(0.0)
        } else {
            (b.shown + fade).min(1.0)
        };
        b.clock_s += dt;
        let Some(sp) = roster.get(b.species as usize) else {
            continue;
        };
        let up = b.pos.normalize_or(Vec3::Y);
        let night = world.night_at(up);
        let bird = sp.plan == Plan::Bird;
        // What it wants.
        b.mode_s -= dt;
        if let (Who::Wild { .. }, Some(feet)) = (b.who, feet) {
            b.trust.decay(hours, &s.taming);
            let d = feet.distance(b.pos);
            let food = sp
                .keep
                .as_ref()
                .is_some_and(|kp| held.is_some_and(|i| kp.eats(i)));
            let fear = taming::fear_radius(&s.taming, moving, food);
            if d < fear && !night {
                if b.mode != Mode::Flee {
                    b.trust.spook(&s.taming);
                }
                b.mode = Mode::Flee;
                b.mode_s = 2.5;
                let away = (b.pos - feet).normalize_or(b.facing);
                b.goal = Some(b.pos + away * 8.0);
            } else if food
                && !moving
                && d < s.taming.fear_moving_m
                && d > 1.4
                && !night
                && b.mode != Mode::Flee
            {
                b.mode = Mode::Approach;
                b.goal = Some(feet + (b.pos - feet).normalize_or(b.facing) * 1.2);
            } else if b.mode == Mode::Approach {
                b.mode = Mode::Idle;
                b.goal = None;
                b.idle_s = b.rng.range(1.0, 3.0);
            }
            if b.mode == Mode::Flee && b.mode_s <= 0.0 {
                b.mode = Mode::Idle;
                b.goal = None;
            }
        }
        if b.mode == Mode::Follow
            && let Some(feet) = feet
        {
            let d = feet.distance(b.pos);
            b.goal = (d > 1.8).then(|| feet + (b.pos - feet).normalize_or(b.facing) * 1.4);
        }
        // Sleep at night, where nothing is chasing it.
        let resting = (night || b.sitting) && matches!(b.mode, Mode::Idle | Mode::Walk);
        let target_down = if resting { 1.0 } else { 0.0 };
        b.down += (target_down - b.down).clamp(-dt * 0.6, dt * 0.6);
        if resting {
            b.goal = None;
            b.mode = Mode::Idle;
        }
        // Wander: potter about every few seconds.
        if b.mode == Mode::Idle && !resting && b.down < 0.05 {
            b.idle_s -= dt;
            if b.idle_s <= 0.0 {
                b.goal = wander_goal(b, bird);
                b.mode = Mode::Walk;
                b.idle_s = if bird {
                    b.rng.range(1.5, 4.0)
                } else {
                    b.rng.range(3.0, 8.0)
                };
            }
        }
        // Move toward the goal: turn, then go, at the mode's pace.
        let pace = match b.mode {
            Mode::Flee => sp.run_mps,
            Mode::Follow => {
                let d = feet.map_or(0.0, |f| f.distance(b.pos));
                if d > 5.0 {
                    sp.run_mps * 0.8
                } else {
                    sp.walk_mps * 1.8
                }
            }
            Mode::Approach => sp.walk_mps * 0.8,
            _ => sp.walk_mps,
        };
        let mut want = 0.0;
        if let Some(goal) = b.goal {
            let to = goal - b.pos;
            let flat = to - up * to.dot(up);
            if flat.length() < 0.15 {
                b.goal = None;
                if matches!(b.mode, Mode::Walk | Mode::Approach) {
                    b.mode = Mode::Idle;
                }
            } else {
                let dir = flat.normalize();
                let turn = if bird { 7.0 } else { 2.5 } * dt;
                let angle = b.facing.angle_between(dir);
                if angle > 1e-3 {
                    let axis = b.facing.cross(dir).normalize_or(up);
                    b.facing = Quat::from_axis_angle(axis, angle.min(turn)) * b.facing;
                }
                if b.facing.dot(dir) > 0.4 {
                    want = pace.min(flat.length() * 2.0 + 0.1);
                }
            }
        }
        b.speed += (want - b.speed).clamp(-dt * 4.0, dt * 4.0);
        let moved = b.facing * b.speed * dt;
        let next = b.pos + moved;
        let allowed = match &b.range {
            Range::Pen(_, cells) if b.mode != Mode::Follow => {
                world.cell(next).is_none_or(|(_, key)| cells.contains(&key))
            }
            _ => true,
        };
        let travelled = if allowed && b.speed > 0.0 {
            let d = next.normalize();
            b.pos = d * world.ground(d);
            moved.length()
        } else {
            if !allowed {
                b.goal = None;
                b.mode = Mode::Idle;
                b.speed = 0.0;
            }
            let d = b.pos.normalize();
            b.pos = d * world.ground(d);
            0.0
        };
        let up = b.pos.normalize_or(Vec3::Y);
        b.facing = (b.facing - up * b.facing.dot(up)).normalize_or(up.any_orthonormal_vector());
        // The legs: phase from distance over stride, so no foot slides.
        let stride = sp.stride_m * b.genome.size_factor();
        b.phase = (b.phase + travelled / stride * std::f32::consts::TAU) % std::f32::consts::TAU;
        let swing_goal = (b.speed / sp.walk_mps * 0.45).min(0.75);
        b.swing += (swing_goal - b.swing) * (dt * 8.0).min(1.0);
        // The head, the glance and the tail.
        let standing = b.speed < 0.05 && b.down < 0.05;
        if bird {
            // Pecks: quick dips at random 0.4 to 2 s while standing.
            if standing {
                b.peck_s -= dt;
                if b.peck_s <= 0.0 {
                    b.pecking = 1.0;
                    b.peck_s = b.rng.range(0.4, 2.0);
                }
            }
            b.pecking = (b.pecking - dt / 0.28).max(0.0);
            b.head_down = (b.pecking * std::f32::consts::PI).sin() * 1.15;
        } else {
            // Grazing, head down while it stands; up to walk or look about.
            let grazing = standing && b.glance_s > 0.6 && b.mode != Mode::Follow;
            b.head_goal = if grazing { 1.85 } else { 0.0 };
            let chew = if grazing {
                (b.clock_s * 5.0).sin() * 0.05
            } else {
                0.0
            };
            b.head_down += (b.head_goal + chew - b.head_down) * (dt * 2.5).min(1.0);
        }
        b.glance_s -= dt;
        if b.glance_s <= 0.0 {
            b.glance_goal = if b.rng.unit() < 0.35 {
                0.0
            } else {
                b.rng.range(-0.8, 0.8)
            };
            b.glance_s = if bird {
                b.rng.range(0.8, 2.5)
            } else {
                b.rng.range(2.0, 6.0)
            };
        }
        let rate = if bird { 14.0 } else { 3.0 };
        b.glance += (b.glance_goal - b.glance) * (dt * rate).min(1.0);
        b.tail_s -= dt;
        if b.tail_s <= 0.0 {
            b.tail_s = b.rng.range(1.0, 3.0);
        }
        b.tail = if bird {
            // A flick: up and back in a quarter second.
            let t = (0.25 - b.tail_s.min(0.25)) / 0.25;
            (t * std::f32::consts::PI).sin() * 0.35 * f32::from(b.tail_s < 0.25)
        } else {
            (b.clock_s * 2.2 + b.phase * 0.1).sin() * 0.35
        };
    }
    yard.beasts
        .retain(|b| !(b.leaving && b.shown <= 0.0 && b.entity.is_none()));
}

/// A new place to potter to: a bird a short way, a beast a few metres,
/// always inside its range.
fn wander_goal(b: &mut Beast, bird: bool) -> Option<Vec3> {
    let up = b.pos.normalize_or(Vec3::Y);
    let east = Vec3::Y.cross(up).normalize_or(Vec3::X);
    let north = up.cross(east);
    let r = b.pos.length();
    match &b.range {
        Range::Pen(centres, _) => {
            let c = centres[(b.rng.next_u64() % centres.len() as u64) as usize];
            let jitter = (east * b.rng.range(-0.9, 0.9) + north * b.rng.range(-0.9, 0.9)) * 0.6;
            let goal = c * r + jitter;
            if bird {
                // A hen potters: a short step toward that cell.
                let to = goal - b.pos;
                Some(b.pos + to.clamp_length_max(b.rng.range(0.4, 1.4)))
            } else {
                Some(goal)
            }
        }
        Range::Around(centre, radius) => {
            let a = b.rng.range(0.0, std::f32::consts::TAU);
            let step = if bird {
                b.rng.range(0.4, 1.5)
            } else {
                b.rng.range(1.0, 4.0)
            };
            let goal = b.pos + (east * a.cos() + north * a.sin()) * step;
            let home = *centre * r;
            let off = goal - home;
            Some(home + off.clamp_length_max(*radius))
        }
    }
}

/// One entity per animal, its mesh rebuilt from its rig each frame and
/// faded through the terrain's mask.
fn draw(
    mut commands: Commands,
    world: World,
    mut yard: ResMut<Farmyard>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let roster = world.roster().to_vec();
    let Some(material) = yard.material.clone() else {
        return;
    };
    let centre = world.frame.center;
    let day = world.day();
    for b in &mut yard.beasts {
        let Some(sp) = roster.get(b.species as usize) else {
            continue;
        };
        if b.leaving && b.shown <= 0.0 {
            if let Some((e, _)) = b.entity.take() {
                commands.entity(e).despawn();
            }
            continue;
        }
        let up = b.pos.normalize_or(Vec3::Y);
        let right = b.facing.cross(up).normalize_or(Vec3::Z);
        let pose = rig::Pose {
            phase: b.phase,
            swing: b.swing,
            head_down: b.head_down,
            glance: b.glance,
            bob: if sp.plan == Plan::Bird {
                (b.phase * 2.0).sin() * 0.02 * (b.swing / 0.45).min(1.0)
            } else {
                0.0
            },
            tail: b.tail,
            shift: if sp.plan == Plan::Quadruped {
                (b.clock_s * 0.9 + (b.rng_seed() as f32)).sin() * 0.035
            } else {
                0.0
            },
            down: b.down,
        };
        let mut builder = rig::Builder::default();
        rig::build(
            &mut builder,
            sp,
            &b.genome,
            b.sex,
            &pose,
            [b.facing, up, right],
        );
        let at = (centre + b.pos.as_dvec3()).as_vec3();
        // A young animal is drawn at its stage's size (design 13, E1).
        let size = b
            .born
            .zip(sp.keep.as_ref())
            .map_or(1.0, |(born, k)| k.young.stage(b.sex, born, day).1);
        let place = Transform::from_translation(at).with_scale(Vec3::splat(size));
        match &b.entity {
            Some((e, handle)) => {
                if let Some(mesh) = meshes.get_mut(handle) {
                    builder.write(mesh);
                }
                commands.entity(*e).insert((place, Faded(1.0 - b.shown)));
            }
            None => {
                let mut mesh = Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::default(),
                );
                builder.write(&mut mesh);
                let handle = meshes.add(mesh);
                let e = commands
                    .spawn((
                        Name::new(sp.name.clone()),
                        LitByField,
                        Faded(1.0 - b.shown),
                        place,
                        Visibility::default(),
                    ))
                    .with_child((
                        Mesh3d(handle.clone()),
                        MeshMaterial3d(material.clone()),
                        Transform::default(),
                        bevy::camera::visibility::NoFrustumCulling,
                    ))
                    .id();
                b.entity = Some((e, handle));
            }
        }
    }
}

impl Beast {
    fn rng_seed(&self) -> u32 {
        match self.who {
            Who::Wild { seed } => (seed & 0xff) as u32,
            Who::Kept { id } => (id & 0xff) as u32,
        }
    }
}

/// The fences, one mesh, rebuilt when the pieces change.
fn draw_fences(
    world: World,
    save: Option<Res<WorldSave>>,
    mut yard: ResMut<Farmyard>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut commands: Commands,
    roots: Query<(Entity, Option<&Children>), With<FenceRoot>>,
) {
    let (Some(save), Some(fine)) = (save, world.fine.as_deref()) else {
        return;
    };
    let pieces = record::pieces(&save.records);
    let incubators = pbd_core::husbandry::breeding::incubators(&save.records);
    let version = Arc::as_ptr(&fine.set) as usize;
    if yard
        .fences_drawn
        .as_ref()
        .is_some_and(|(p, i, v)| *p == pieces && *i == incubators && *v == version)
    {
        return;
    }
    let Ok((root, children)) = roots.single() else {
        return;
    };
    let set = &fine.set;
    let records = set.finest_records();
    let by_key: HashMap<u32, usize> = if pieces.is_empty() {
        HashMap::new()
    } else {
        records
            .iter()
            .enumerate()
            .map(|(i, r)| (r.key(), i))
            .collect()
    };
    let mut origin = None;
    let mut builder = rig::Builder::default();
    let wood = [0.42, 0.28, 0.16];
    let light = [0.80, 0.70, 0.52];
    let mut drawn = 0;
    for (edge, &gate) in &pieces {
        let (a, b) = edge.cells();
        let (Some(&ia), Some(&ib)) = (by_key.get(&a), by_key.get(&b)) else {
            continue;
        };
        let Some((ca, cb)) = edge_corners(&records[ia], &records[ib]) else {
            continue;
        };
        let pa = ca * world.ground(ca);
        let pb = cb * world.ground(cb);
        let o = *origin.get_or_insert(pa);
        fence_piece(
            &mut builder,
            pa - o,
            pb - o,
            (ca + cb).normalize_or(Vec3::Y),
            gate,
            if gate { light } else { wood },
        );
        drawn += 1;
    }
    drawn += breed::draw_incubators(&world, &save, &mut builder, &mut origin);
    yard.fences_drawn = Some((pieces, incubators, version));
    if let Some(children) = children {
        for c in children.iter() {
            commands.entity(c).despawn();
        }
    }
    let Some(o) = origin else {
        return;
    };
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    builder.write(&mut mesh);
    let Some(material) = yard.material.clone() else {
        return;
    };
    commands
        .entity(root)
        .insert(Transform::from_translation(
            (world.frame.center + o.as_dvec3()).as_vec3(),
        ))
        .with_child((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material),
            Transform::default(),
        ));
    debug!("husbandry: {drawn} fence pieces drawn");
}

/// A piece between two corners on the ground: two posts and two rails; a
/// gate adds a brace.
fn fence_piece(out: &mut rig::Builder, a: Vec3, b: Vec3, up: Vec3, gate: bool, colour: [f32; 3]) {
    let along = b - a;
    let side = along.cross(up).normalize_or(Vec3::Z);
    let post = |out: &mut rig::Builder, at: Vec3| {
        let fwd = along.normalize_or(Vec3::X);
        out.cuboid(
            at + up * 0.6,
            [fwd * 0.07, up * 0.65, side * 0.07],
            [0.33, 0.22, 0.12],
        );
    };
    post(out, a);
    post(out, b);
    // The posts stand on the corners, the rails run between them.
    let rail = |out: &mut rig::Builder, h: f32| {
        let mid = (a + b) * 0.5 + up * h;
        out.cuboid(mid, [along * 0.5, up * 0.05, side * 0.03], colour);
    };
    rail(out, 0.35);
    rail(out, 0.85);
    if gate {
        // A brace from bottom corner to top corner.
        let from = a + up * 0.35;
        let to = b + up * 0.85;
        let brace = to - from;
        let bup = brace.cross(side).normalize_or(up);
        out.cuboid(
            (from + to) * 0.5,
            [brace * 0.5, bup * 0.045, side * 0.035],
            colour,
        );
        rail(out, 0.6);
    }
}

/// The HUD line: a one-off message, or what the aimed animal is.
fn hud(
    world: World,
    save: Option<Res<WorldSave>>,
    aimed: Res<AimedGround>,
    mut yard: ResMut<Farmyard>,
    mut texts: Query<&mut Text, With<FarmText>>,
) {
    let line = if yard.hold_s > 0.0 {
        yard.status.clone()
    } else {
        describe(&world, save.as_deref(), &mut yard)
            .or_else(|| {
                let walking = world.walking.as_ref().is_some_and(|w| w.active);
                walking
                    .then(|| breed::describe_incubator(&world, save.as_deref()?, aimed.0?))
                    .flatten()
            })
            .unwrap_or_default()
    };
    for mut text in &mut texts {
        if text.0 != line {
            text.0 = line.clone();
        }
    }
}

/// What the player is looking at, in a line.
fn describe(world: &World, save: Option<&WorldSave>, yard: &mut Farmyard) -> Option<String> {
    if !world.walking.as_ref().is_some_and(|w| w.active) {
        return None;
    }
    let k = aimed_beast(world, yard)?;
    let b = &yard.beasts[k];
    let sp = world.roster().get(b.species as usize)?.clone();
    let sex = if b.sex == Sex::M { "male" } else { "female" };
    // A young one is named by its stage: a chick, a lamb (design 13, E1).
    let young = b
        .born
        .and_then(|born| breed::stage_now(world, b.species, b.sex, born))
        .filter(|(_, size)| *size < 1.0)
        .map(|(name, _)| format!(" {name}"))
        .unwrap_or_default();
    let sitting = if b.sitting { ", sitting" } else { "" };
    match b.who {
        Who::Wild { .. } => Some(match (sp.refusal(), sp.keep.as_ref()) {
            (Some(why), _) => format!("{} (wild) - {why}", sp.name),
            (None, Some(k)) => format!(
                "{} (wild, {sex}) - trust {:.0}% - E with {}",
                sp.name,
                b.trust.value,
                k.food_name()
            ),
            _ => sp.name.clone(),
        }),
        Who::Kept { id } => {
            let save = save?;
            let body = save.records.get(record::ANIMAL, id)?.read::<AnimalBody>()?;
            let now = farm_now(world, yard, save, id, &body);
            let n = &world.settings().needs;
            let keep = sp.keep.as_ref()?;
            let waiting: Vec<String> = keep
                .products
                .iter()
                .filter(|p| p.gives(body.sex))
                .filter_map(|p| {
                    let w = now.waiting(&p.good);
                    (w > 0).then(|| format!("{w} {}", p.good))
                })
                .collect();
            let c_mood = if now.sick(n) {
                "sick".to_string()
            } else if now.strayed(n) {
                "strayed: feed it".to_string()
            } else {
                format!("health {:.0}", now.health)
            };
            let pen = if body.pen.is_some() {
                "penned"
            } else {
                "loose"
            };
            Some(if waiting.is_empty() {
                format!("{}{young} ({sex}, {pen}{sitting}) - {c_mood}", sp.name)
            } else {
                format!(
                    "{}{young} ({sex}, {pen}{sitting}) - {c_mood} - {} waiting [E]",
                    sp.name,
                    waiting.join(", ")
                )
            })
        }
    }
}

/// `--husbandry <scene>`: a capture's farm, set up through the same paths a
/// player's hands take (a measurement instrument; `animal-husbandry` S12).
fn script(
    world: World,
    script: Option<Res<HusbandryScript>>,
    mut yard: ResMut<Farmyard>,
    mut hotbar: Option<ResMut<Hotbar>>,
    mut save: Option<ResMut<WorldSave>>,
    mut pick: ResMut<BreedingPick>,
    mut done: Local<u32>,
) {
    let Some(script) = script else {
        return;
    };
    if *done > 1 {
        return;
    }
    let (Some(hotbar), Some(save)) = (hotbar.as_deref_mut(), save.as_deref_mut()) else {
        return;
    };
    let (Some((_, look)), Some(feet)) = (world.eye(), world.feet()) else {
        return;
    };
    if world.cell(feet).is_none() {
        return;
    }
    let up = feet.normalize();
    let ahead = (look - up * look.dot(up)).normalize_or(up.any_orthonormal_vector());
    let right = ahead.cross(up);
    let roster = world.roster().to_vec();
    let index = |id: &str| roster.iter().position(|s| s.id == id).unwrap_or(0) as u16;
    let ground = |at: Vec3| at.normalize() * world.ground(at);
    let wild = |yard: &mut Farmyard, sp: u16, n: u64, at: Vec3| -> usize {
        let seed = (0x5c_u64 << 56) | (u64::from(sp) << 16) | n;
        let s = &roster[sp as usize];
        let genome = s.keep.as_ref().map_or_else(Genome::default, |k| {
            Genome::roll(seed, k.coats.len(), k.patterns.len())
        });
        let sex = if n == 0 { Sex::M } else { Sex::F };
        let pos = ground(at);
        let mut b = Beast::new(
            Who::Wild { seed },
            sp,
            sex,
            genome,
            pos,
            Range::Around(pos.normalize(), 3.0),
            seed,
        );
        b.facing = -ahead;
        yard.beasts.push(b);
        yard.beasts.len() - 1
    };
    *done += 1;
    if *done == 1 {
        // The first frame only finds the ground; the scene is set on the next.
        return;
    }
    let spot = feet + ahead * 7.0;
    // The hands the scene needs, where a capture's `--lamps` took the kit
    // (a capture's save is memory-only, so nothing is kept).
    for (item, count) in [
        (Item::Block(Material::Grass), 64),
        (Item::Good(Good::WHEAT_SEED), 16),
        (Item::Good(Good::WHEAT), 16),
        (Item::Good(Good::TURNIP), 16),
        (Item::Good(Good::FENCE), 24),
        (Item::Good(Good::GATE), 2),
    ] {
        if !hotbar.iter().flatten().any(|s| s.item == item) {
            hotbar.give(item, count);
        }
    }
    match script.scene.as_str() {
        "wild" => {
            for n in 0..5 {
                let a = n as f32 * 1.3;
                wild(
                    &mut yard,
                    index("meadow-hen"),
                    n,
                    spot + right * (a.cos() * 1.5 - 2.0) + ahead * a.sin(),
                );
            }
            for n in 0..3 {
                wild(
                    &mut yard,
                    index("woolback"),
                    n,
                    spot + right * (2.0 + n as f32 * 1.6) + ahead * (n as f32 - 1.0) * 1.5,
                );
            }
        }
        "tame" => {
            let k = wild(&mut yard, index("woolback"), 1, feet + ahead * 3.2);
            select_food(hotbar, &roster[index("woolback") as usize]);
            for _ in 0..12 {
                if matches!(yard.beasts[k].who, Who::Kept { .. }) {
                    break;
                }
                feed_wild(&world, &mut yard, hotbar, save, k);
            }
            for n in 0..3 {
                wild(
                    &mut yard,
                    index("meadow-hen"),
                    n + 5,
                    feet + ahead * 5.0 + right * (n as f32 - 1.0) * 1.2,
                );
            }
        }
        "pen" | "collect" => {
            let Some((_, centre)) = world.cell(spot) else {
                return;
            };
            build_ring(&world, &mut yard, hotbar, save, centre, feet);
            let herd = [
                ("meadow-hen", 0u64, 0.0f32, 0.0f32),
                ("meadow-hen", 1, 0.8, 0.6),
                ("meadow-hen", 2, -0.7, 0.9),
                ("woolback", 3, 2.3, 0.4),
                ("woolback", 4, -2.2, -0.6),
                ("plodder", 5, 0.3, 2.6),
            ];
            for (id, n, x, y) in herd {
                let k = wild(&mut yard, index(id), n + 10, spot + right * x + ahead * y);
                select_food(hotbar, &roster[index(id) as usize]);
                // Hens: no cockerel in the script but the first.
                for _ in 0..12 {
                    if matches!(yard.beasts[k].who, Who::Kept { .. }) {
                        break;
                    }
                    feed_wild(&world, &mut yard, hotbar, save, k);
                }
                settle(&world, &mut yard, save, k);
            }
            if script.scene == "collect" {
                // As if kept three days: every record re-based three world
                // days back (the instrument's one liberty), then the hen in
                // front collected from through the player's own path.
                let back = 3.0;
                for (id, mut body) in record::animals(&save.records) {
                    body.as_of -= back;
                    body.fed_until -= back;
                    body.born -= back;
                    save.store(&Author::Player(0), vec![body.record(id)]);
                }
                if let Some(k) = yard
                    .beasts
                    .iter()
                    .position(|b| b.species == index("meadow-hen") && b.sex == Sex::F)
                {
                    use_on(&world, &mut yard, hotbar, save, k);
                    yard.hold_s = 1.0e6;
                }
            }
        }
        "eggs" | "breed" => {
            let Some((_, centre)) = world.cell(spot) else {
                return;
            };
            build_ring(&world, &mut yard, hotbar, save, centre, feet);
            let (sp, herd): (&str, &[(u64, f32, f32)]) = if script.scene == "eggs" {
                // The cockerel first (the script's n = 0 is male), three hens.
                (
                    "meadow-hen",
                    &[(0, 0.0, 0.0), (1, 0.8, 0.6), (2, -0.7, 0.9), (3, 0.6, -0.8)],
                )
            } else {
                (
                    "woolback",
                    &[(0, 0.0, 0.0), (1, 2.0, 0.6), (2, -2.0, 0.6), (3, 0.4, 2.2)],
                )
            };
            for &(n, x, y) in herd {
                let k = wild(&mut yard, index(sp), n, spot + right * x + ahead * y);
                select_food(hotbar, &roster[index(sp) as usize]);
                for _ in 0..12 {
                    if matches!(yard.beasts[k].who, Who::Kept { .. }) {
                        break;
                    }
                    feed_wild(&world, &mut yard, hotbar, save, k);
                }
                settle(&world, &mut yard, save, k);
            }
            if script.scene == "eggs" {
                // As if kept three days (the instrument's one liberty, as in
                // `collect`), then a hen's eggs collected and set under
                // another through the player's own path, then five more days
                // so the clutch has hatched two days ago.
                breed::rebase(save, 3.0);
                let hens: Vec<usize> = yard
                    .beasts
                    .iter()
                    .enumerate()
                    .filter(|(_, b)| matches!(b.who, Who::Kept { .. }) && b.sex == Sex::F)
                    .map(|(k, _)| k)
                    .collect();
                if let [first, second, ..] = hens[..] {
                    use_on(&world, &mut yard, hotbar, save, first);
                    if let Some(k) = (0..pbd_core::inventory::SLOTS).find(|&k| {
                        hotbar
                            .get(k)
                            .is_some_and(|s| s.item == Item::Good(Good::FERTILE_EGG))
                    }) {
                        hotbar.select(k);
                        use_on(&world, &mut yard, hotbar, save, second);
                    }
                }
                breed::rebase(save, 5.0);
                hotbar.give(Item::Good(Good::INCUBATOR), 1);
                if let Some(k) = (0..pbd_core::inventory::SLOTS).find(|&k| {
                    hotbar
                        .get(k)
                        .is_some_and(|s| s.item == Item::Good(Good::INCUBATOR))
                }) {
                    hotbar.select(k);
                    let at = feet + ahead * 2.5 - right * 1.5;
                    breed::place_incubator(&world, &mut yard, hotbar, save, at);
                }
                let born = breed::realise(&world, &mut yard, save);
                info!("HUSBANDRY eggs: {born} hatched");
            } else {
                // Forty days of the farm run at once as if every need were met
                // (the instrument's liberty: a capture's pen has no water),
                // through the same rule and the same proposals as play.
                let born = breed::run_content(&world, &mut yard, save, 40.0);
                *pick = breed::pick_kin(save, index(sp));
                info!("HUSBANDRY breed: {born} born");
            }
            yard.hold_s = 1.0e6;
        }
        other => warn!("--husbandry {other}: wild, tame, pen, collect, eggs or breed"),
    }
    info!(
        "HUSBANDRY scene {} set: {} animals, {} pieces, {} kept",
        script.scene,
        yard.beasts.len(),
        record::pieces(&save.records).len(),
        record::animals(&save.records).len()
    );
}

/// Put the species' favourite food in hand, from the pack if need be, as a
/// player would.
fn select_food(hotbar: &mut Slots, species: &LandSpecies) {
    let Some(keep) = species.keep.as_ref() else {
        return;
    };
    let Some(k) = (0..pbd_core::inventory::CARRIED)
        .find(|&k| hotbar.get(k).is_some_and(|s| keep.eats(s.item)))
    else {
        return;
    };
    // From the pack, swapped with the last hotbar slot.
    let k = if k >= pbd_core::inventory::SLOTS {
        let last = pbd_core::inventory::SLOTS - 1;
        let (a, b) = (hotbar.get(last), hotbar.get(k));
        hotbar.set(last, b);
        hotbar.set(k, a);
        last
    } else {
        k
    };
    hotbar.select(k);
}

/// A ring of fence round a cell and its neighbours, the gate on the side
/// nearest `toward`: through the same placing path as the player's.
fn build_ring(
    world: &World,
    yard: &mut Farmyard,
    hotbar: &mut Slots,
    save: &mut WorldSave,
    centre: u32,
    toward: Vec3,
) {
    let Some(fine) = world.fine.as_deref() else {
        return;
    };
    let set = &fine.set;
    let records = set.finest_records();
    let by_key: HashMap<u32, usize> = records
        .iter()
        .enumerate()
        .map(|(i, r)| (r.key(), i))
        .collect();
    let Some(&ci) = by_key.get(&centre) else {
        return;
    };
    let inner: BTreeSet<usize> = std::iter::once(ci)
        .chain(
            set.finest_neighbors[ci][..records[ci].degree()]
                .iter()
                .map(|&n| n as usize),
        )
        .collect();
    let mut edges: Vec<(Edge, Vec3)> = Vec::new();
    for &i in &inner {
        for &n in &set.finest_neighbors[i][..records[i].degree()] {
            let n = n as usize;
            if inner.contains(&n) {
                continue;
            }
            let mid = (Vec3::from_slice(&records[i].direction_height[..3])
                + Vec3::from_slice(&records[n].direction_height[..3]))
            .normalize();
            edges.push((Edge::new(records[i].key(), records[n].key()), mid));
        }
    }
    edges.sort_by_key(|e| e.0);
    edges.dedup_by_key(|e| e.0);
    let gate = edges
        .iter()
        .min_by(|a, b| {
            a.1.distance_squared(toward.normalize())
                .total_cmp(&b.1.distance_squared(toward.normalize()))
        })
        .map(|e| e.0);
    for (edge, mid) in edges {
        let is_gate = Some(edge) == gate;
        let good = if is_gate { Good::GATE } else { Good::FENCE };
        if let Some(k) = (0..pbd_core::inventory::CARRIED)
            .find(|&k| hotbar.get(k).is_some_and(|s| s.item == Item::Good(good)))
        {
            // The pack's pieces are moved to the hand, as a player would.
            if k >= pbd_core::inventory::SLOTS {
                hotbar.send(k);
            }
        }
        if let Some(k) = (0..pbd_core::inventory::SLOTS)
            .find(|&k| hotbar.get(k).is_some_and(|s| s.item == Item::Good(good)))
        {
            hotbar.select(k);
        }
        // Aimed a hand's breadth inside the cell, toward the edge's middle.
        let inside = by_key[&edge.cells().0];
        let c = Vec3::from_slice(&records[inside].direction_height[..3]);
        let aim = (mid * 0.97 + c * 0.03).normalize();
        let aim_at = aim * world.ground(aim);
        place_piece_at(world, yard, hotbar, save, aim_at, is_gate, edge);
    }
}

/// [`place_piece`] without the reach check, for the instrument, which is
/// standing outside the ring. The edge it places is checked against the one
/// the aim finds.
#[allow(clippy::too_many_arguments)]
fn place_piece_at(
    world: &World,
    yard: &mut Farmyard,
    hotbar: &mut Slots,
    save: &mut WorldSave,
    point: Vec3,
    gate: bool,
    expected: Edge,
) {
    let Some((edge, _, _)) = edge_at(world, point) else {
        return;
    };
    if edge != expected {
        warn!("husbandry script: aimed {edge:?}, meant {expected:?}");
    }
    note_kinds(save);
    let piece = FenceBody {
        gate,
        state: Standing::Placed,
    };
    if !spend_one(hotbar)
        || !save.store_with_hands(piece.record(edge), record::FENCE_SCHEMA, hotbar)
    {
        return;
    }
    yard.fences_drawn = None;
    closed_pen(world, yard, save, edge);
}

#[cfg(test)]
mod tests;
