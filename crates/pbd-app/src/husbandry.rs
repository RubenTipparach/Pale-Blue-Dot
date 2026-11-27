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
pub mod forms;
pub mod hunt;
pub mod loft;
pub mod rig;

pub use breed::BreedingPick;

use crate::config::HusbandryConfig;
use crate::field_light::{Faded, LitByField, LitLikeTerrain};
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
    Edge, Genome, Ground, HusbandrySettings, LandSpecies, Pen, Plan, Sex, Trust, herds, taming,
};
use pbd_core::inventory::{Item, Slots};
use pbd_core::locale::{Localized, tr, tr_with};
use pbd_core::records::Author;
use pbd_core::terrain::Material;
use pbd_core::world_frame::SimOrigin;
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

/// The interact key's reader, so other readers of F (`desktop::farm::use_key`)
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
    /// A predator hunting (`survival-mode` combat C2): its pace is the
    /// hunt's.
    Hunt,
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
    /// Feet, local (physics space, `world-frames` design 10): relative to
    /// the one `SimOrigin`, moved by its rebase.
    pos: Vec3,
    /// Unit, along the ground.
    facing: Vec3,
    speed: f32,
    /// Where it is going, local like `pos`.
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
    /// Held still where a capture set it (`--husbandry close`): it neither
    /// flees nor wanders.
    posed: bool,
    /// Its head held where a capture set it (`--husbandry close-stand` or
    /// `close-graze`): grazing or up, looking ahead. A capture instrument.
    held_graze: Option<bool>,
    /// Its health, hunt and carcass in a Survival world (combat C2).
    game: hunt::Game,
    /// The hunt's pace while in [`Mode::Hunt`], m/s.
    hunt_mps: f32,
}

impl Beast {
    #[allow(clippy::too_many_arguments)]
    fn new(
        who: Who,
        species: u16,
        sex: Sex,
        genome: Genome,
        pos: Vec3,
        up: Vec3,
        range: Range,
        seed: u64,
    ) -> Self {
        let mut rng = Rng::new(seed);
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
            posed: false,
            held_graze: None,
            game: hunt::Game::default(),
            hunt_mps: 0.0,
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
    /// The herd's coat (design 16).
    material: Option<Handle<StandardMaterial>>,
    /// Flat colour: the incubator's glass lid (M26: glass is flat).
    flat: Option<Handle<StandardMaterial>>,
    /// The pen's textured materials in [`PEN_TEXTURES`] order, and each
    /// texture's repeat in metres (design 14).
    pen_materials: Option<([Handle<StandardMaterial>; 4], [f32; 4])>,
    /// What the player did this frame, announced as messages.
    deeds: Vec<Deed>,
    gained: Vec<(Item, u16)>,
    /// Hearts to show over an animal this frame, for a feed
    /// (`taming-hearts`): whose, and how many.
    hearts: Vec<(Who, u8)>,
    /// Predators' bites landed, pts, for the combat path (combat C2).
    bites: Vec<f32>,
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
            flat: None,
            pen_materials: None,
            deeds: Vec::new(),
            gained: Vec::new(),
            hearts: Vec::new(),
            bites: Vec::new(),
        }
    }
}

/// One animal as the player sees it: where it stands, what it is, and
/// whether it is wild, kept or following. Read by the showcase's bot, which
/// looks for an animal the way a player does (`step-videos` 5.1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Seen {
    /// Feet, a planet query point (the bot reasons in planet points).
    pub pos: Vec3,
    /// Roster index on this body.
    pub species: u16,
    pub wild: bool,
    pub following: bool,
    pub female: bool,
}

impl Farmyard {
    /// The animals in view: drawn, and not leaving. Their feet are local;
    /// `origin` gives them as planet points.
    pub fn seen(&self, origin: SimOrigin) -> impl Iterator<Item = Seen> + '_ {
        self.beasts
            .iter()
            .filter(|b| !b.leaving && b.shown > 0.5)
            .map(move |b| Seen {
                pos: origin.query_point(b.pos),
                species: b.species,
                wild: matches!(b.who, Who::Wild { .. }),
                following: b.mode == Mode::Follow,
                female: b.sex == Sex::F,
            })
    }

    /// The kept animals standing in the yard, by record id, with their feet
    /// as planet points: where the map draws a tamed animal that is near
    /// (`world-map` 7.1); one further off is drawn at its home cell.
    pub fn kept(&self, origin: SimOrigin) -> impl Iterator<Item = (u64, Vec3)> + '_ {
        self.beasts
            .iter()
            .filter(|b| !b.leaving)
            .filter_map(move |b| match b.who {
                Who::Kept { id } => Some((id, origin.query_point(b.pos))),
                Who::Wild { .. } => None,
            })
    }

    /// The kept animals of `species` within `within_m` of `near` (local, as
    /// a beast's feet are): their record ids and feet, nearest first. A
    /// horse cart's shafts ask for the horse beside them (`land-transport`
    /// design 11).
    pub fn kept_near(&self, species: u16, near: Vec3, within_m: f32) -> Vec<(u64, Vec3)> {
        let mut found: Vec<(u64, Vec3, f32)> = self
            .beasts
            .iter()
            .filter(|b| b.species == species && !b.leaving)
            .filter_map(|b| match b.who {
                Who::Kept { id } => Some((id, b.pos, b.pos.distance(near))),
                Who::Wild { .. } => None,
            })
            .filter(|(_, _, d)| *d <= within_m)
            .collect();
        found.sort_by(|a, b| a.2.total_cmp(&b.2));
        found.into_iter().map(|(id, p, _)| (id, p)).collect()
    }

    /// Where the kept animal `id` stands (local feet) and faces, if it is
    /// within `within_m` of `near`.
    pub fn kept_pose(&self, id: u64, near: Vec3, within_m: f32) -> Option<(Vec3, Vec3)> {
        self.beasts
            .iter()
            .find(|b| b.who == (Who::Kept { id }) && !b.leaving)
            .filter(|b| b.pos.distance(near) <= within_m)
            .map(|b| (b.pos, b.facing))
    }

    /// A kept animal goes into a cart's shafts: the yard stops drawing it at
    /// once, and its sync leaves it out while a cart names it.
    pub fn hand_over(&mut self, id: u64) {
        for b in &mut self.beasts {
            if b.who == (Who::Kept { id }) {
                b.leaving = true;
                b.shown = 0.0;
                b.mode = Mode::Idle;
            }
        }
    }

    /// A kept animal comes out of a cart's shafts at `pos` (local), facing
    /// `facing`, and follows the player a while, as a newly tamed one does.
    /// It is taken back into the yard at once, so it never blinks out.
    pub fn take_back(
        &mut self,
        id: u64,
        record: &AnimalBody,
        pos: Vec3,
        up: Vec3,
        facing: Vec3,
        follow_s: f32,
    ) {
        self.beasts.retain(|b| b.who != (Who::Kept { id }));
        let mut beast = Beast::new(
            Who::Kept { id },
            record.sp,
            record.sex,
            record.genome(),
            pos,
            up,
            Range::Around(up, 6.0),
            hash2(id, 0xbea57),
        );
        beast.facing = facing;
        beast.born = Some(record.born);
        beast.shown = 1.0;
        beast.mode = Mode::Follow;
        beast.mode_s = follow_s;
        self.beasts.push(beast);
    }

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
        // The yard's feet and goals are local (`world-frames` 10.2).
        crate::sim_origin::add_rebase_hook(app, rebase_yard);
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
                    hearts,
                )
                    .chain()
                    .after(crate::fish::FishSet),
            )
            .add_systems(
                PostUpdate,
                offer_hands
                    .in_set(crate::interact::InteractSet::Offer)
                    .run_if(resource_exists::<crate::interact::Interact>),
            );
    }
}

/// A rebase of the local origin: every animal's feet and goal move with it
/// (`world-frames` 10.2). Its facing is a direction, and the rig is rebuilt
/// every frame, so nothing else is kept in local space.
fn rebase_yard(world: &mut bevy::ecs::world::World, shift: Vec3) {
    if let Some(mut yard) = world.get_resource_mut::<Farmyard>() {
        yard.rebase(shift);
    }
}

impl Farmyard {
    /// Move every local point by `-shift` (a rebase).
    pub fn rebase(&mut self, shift: Vec3) {
        for b in &mut self.beasts {
            b.pos -= shift;
            if let Some(goal) = b.goal.as_mut() {
                *goal -= shift;
            }
        }
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
    space: crate::SimSpace<'w>,
}

impl World<'_, '_> {
    fn settings(&self) -> &HusbandrySettings {
        &self.config.0
    }

    fn roster(&self) -> &[LandSpecies] {
        self.config.0.roster(&self.body.0)
    }

    /// The local origin (`world-frames`): every animal's feet are relative
    /// to it.
    fn origin(&self) -> SimOrigin {
        self.space.origin()
    }

    /// The player's feet, local: what an animal flees, follows and comes to.
    fn feet_local(&self) -> Option<Vec3> {
        let p = self.walkers.iter().next()?;
        let o = self.origin();
        Some(o.with_radius(p.0, o.radius_at(p.0) - f64::from(HALF_HEIGHT)))
    }

    /// The player's feet as a planet query point: for a cell, a herd's
    /// square, a piece's reach.
    fn feet_point(&self) -> Option<Vec3> {
        self.feet_local().map(|f| self.origin().query_point(f))
    }

    /// An animal's local point as a planet query point.
    fn query(&self, local: Vec3) -> Vec3 {
        self.origin().query_point(local)
    }

    /// The eye, local, and the look.
    fn eye(&self) -> Option<(Vec3, Vec3)> {
        let t = self.cameras.iter().next()?;
        Some((
            crate::sim_origin::local_from_render(&self.frame, t.translation()),
            t.forward().as_vec3(),
        ))
    }

    /// The local point standing on the ground along the planet direction
    /// `d`, in f64.
    fn stood(&self, d: Vec3) -> Vec3 {
        let d = d.normalize_or(Vec3::Y);
        self.origin().at(d.as_dvec3(), self.ground_precise(d))
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

    /// [`World::ground`] in f64: the floor an animal is stood on. An f32
    /// radius holds half a millimetre, a stair on a tilted facet.
    fn ground_precise(&self, d: Vec3) -> f64 {
        let d = d.normalize_or(Vec3::Y);
        let surface = PLANET_RADIUS + crate::planet::surface_height(d).max(0.0);
        match self.contact.as_deref() {
            Some(c) => c.stand(d * (surface + 1.5)).floor_precise,
            None => f64::from(surface),
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
    assets: Option<Res<AssetServer>>,
) {
    // The pen's own wood (design 14): the towns' textures, sampled nearest
    // and repeated at the manifest's scale.
    if let Some(assets) = assets.as_deref() {
        let repeats = crate::towns::load_repeats();
        let repeat = PEN_TEXTURES.map(|n| repeats.get(n).copied().unwrap_or(1.0).max(0.25));
        let pen = PEN_TEXTURES.map(|n| {
            materials.add(StandardMaterial {
                base_color_texture: Some(crate::towns::texture(assets, n)),
                perceptual_roughness: 0.93,
                reflectance: 0.2,
                ..default()
            })
        });
        // Its pipeline built behind the loading screen: one hidden post.
        let mut post = rig::Builder::default();
        post.textured_cuboid(
            Vec3::ZERO,
            [Vec3::X * 0.07, Vec3::Y * 0.6, Vec3::Z * 0.07],
            1.0,
        );
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        post.write(&mut mesh);
        commands
            .spawn((
                Name::new("Pen prewarm template"),
                LitByField,
                LitLikeTerrain,
                Faded(1.0),
                Transform::from_xyz(0.0, -1.0e5, 0.0),
                Visibility::Hidden,
            ))
            .with_child((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(pen[0].clone()),
                Transform::default(),
            ));
        yard.pen_materials = Some((pen, repeat));
    }
    // The herd's coats (design 16): the `coat` strip, tinted by each
    // animal's colours, sampled nearest.
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: assets
            .as_deref()
            .map(|a| crate::towns::texture(a, "herd_hide")),
        perceptual_roughness: 0.9,
        reflectance: 0.2,
        ..default()
    });
    yard.material = Some(material.clone());
    yard.flat = Some(materials.add(StandardMaterial {
        base_color: Color::WHITE,
        perceptual_roughness: 0.9,
        reflectance: 0.2,
        ..default()
    }));
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
            LitLikeTerrain,
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
        LitLikeTerrain,
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
    let (Some(feet_point), Some(save)) = (world.feet_point(), save.as_deref_mut()) else {
        return;
    };
    if world.cell(feet_point).is_none() {
        return;
    }
    // The farm's own births and conceptions up to now (design 13, E5, E6).
    breed::realise(&world, &mut yard, save);
    let s = world.settings().clone();
    let near = s.herds.near_m;
    let up = feet_point.normalize_or(Vec3::Y);
    let kept = record::animals(&save.records);
    // A horse in a cart's shafts is the cart's to draw (`land-transport`
    // design 11).
    let hitched = pbd_core::vehicle::cart::hitched(&save.records);
    let tamed: BTreeSet<u64> = kept.iter().map(|(_, b)| b.wild).collect();
    let mut wanted: Vec<Who> = Vec::new();
    // Kept animals near.
    let pieces = record::pieces(&save.records);
    for (id, body) in &kept {
        // An animal ridden is the ride's to draw (design 14).
        if body.state != record::Life::Alive
            || body.body != world.body.0
            || hitched.contains(id)
            || body.ridden
        {
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
        let pos = world.stood(d);
        let mut beast = Beast::new(
            who,
            body.sp,
            body.sex,
            body.genome(),
            pos,
            d,
            range,
            hash2(*id, 0xbea57),
        );
        beast.born = Some(body.born);
        beast.sitting = sitting;
        yard.beasts.push(beast);
    }
    // Wild herds near, on dry grass.
    let roster: Vec<LandSpecies> = world.roster().to_vec();
    // A member killed in a Survival world stays away until its herd comes
    // back (design 17): its `hunted` record.
    let survival = hunt::mode_of(Some(&*save)).is_survival();
    let day = world.day();
    for herd in wild_herds_near(up, near, &s.herds, &roster, world.sea.as_deref()) {
        let east = Vec3::Y.cross(herd.centre).normalize_or(Vec3::X);
        let north = herd.centre.cross(east);
        for (member, m) in herd.members.iter().enumerate() {
            if tamed.contains(&m.seed)
                || (survival && hunt::away(save, crate::survival::rules(), m.seed, day))
            {
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
            let mut beast = Beast::new(
                who,
                herd.species,
                m.sex,
                genome,
                world.stood(d),
                d,
                Range::Around(herd.centre, s.herds.spread_m * 1.5),
                m.seed,
            );
            beast.game.herd = Some((hunt::square_id(herd.square), member as u32));
            yard.beasts.push(beast);
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

/// The wild herds whose centre is within `reach_m` of the direction `up`
/// (and a herd's spread past it), on dry grass: the one rule the yard spawns
/// its herds by (`near_m`) and the world map draws them by (`world-map` 7.1,
/// 1.5 km), so a marker is always where a herd stands. In square order.
pub fn wild_herds_near(
    up: Vec3,
    reach_m: f32,
    settings: &pbd_core::husbandry::HerdSettings,
    roster: &[LandSpecies],
    sea: Option<&crate::sea::Sea>,
) -> Vec<herds::Herd> {
    let seed = crate::planet::terrain_config().seed;
    let up = up.normalize_or(Vec3::Y);
    herds::squares_near(up, reach_m, PLANET_RADIUS, settings)
        .into_iter()
        .flat_map(|square| herds::herds_in(seed, square, PLANET_RADIUS, settings, roster))
        .filter(|herd| herd.centre.angle_between(up) * PLANET_RADIUS <= reach_m + settings.spread_m)
        .filter(|herd| {
            let ground = roster
                .get(herd.species as usize)
                .map_or(Ground::Grass, |sp| sp.ground);
            herd_ground(sea, herd.centre, ground)
        })
        .collect()
}

/// Whether a direction is the ground a herd of this kind stands on: dry
/// grass (the meadows' game and herds), or dry desert sand and rock (the
/// dust hound, `survival-mode` design 17).
pub fn herd_ground(sea: Option<&crate::sea::Sea>, d: Vec3, ground: Ground) -> bool {
    match ground {
        Ground::Grass => grassy_land(sea, d),
        Ground::Desert => desert_land(sea, d),
    }
}

/// Whether a direction is dry land of the desert: its sand or bare rock.
pub fn desert_land(sea: Option<&crate::sea::Sea>, d: Vec3) -> bool {
    if crate::planet::surface_height(d) <= 0.5 {
        return false;
    }
    if sea.is_some_and(|s| s.depth_at(d) > 0.0) {
        return false;
    }
    let config = crate::planet::terrain_config();
    let altitude = pbd_core::planet_gen::surface_altitude(config, d);
    pbd_core::planet_gen::biome_at(config, d, altitude) == pbd_core::planet_gen::Biome::Desert
        && matches!(
            pbd_core::planet_gen::top_material(config, d, altitude),
            Material::Sand | Material::Rock | Material::Stone
        )
}

/// Whether a direction is dry land topped with grass, by the generator (a
/// herd's centre may be beyond the resident fine set).
pub fn grassy_land(sea: Option<&crate::sea::Sea>, d: Vec3) -> bool {
    if crate::planet::surface_height(d) <= 0.5 {
        return false;
    }
    if sea.is_some_and(|s| s.depth_at(d) > 0.0) {
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

/// What F would do with the animal, incubator or piece aimed at: the
/// husbandry hands' rows of `interact::ACTS`, in the order [`hands`] and
/// [`use_on`] try them.
fn offer(
    world: &World,
    yard: &mut Farmyard,
    hotbar: &Slots,
    save: &WorldSave,
    aimed: Option<Vec3>,
) -> Option<crate::interact::Offer> {
    use crate::interact::{Act, Offer};
    let held = hotbar.held().map(|st| st.item);
    if let Some(k) = aimed_beast(world, yard) {
        let (who, species) = (yard.beasts[k].who, yard.beasts[k].species);
        let following = yard.beasts[k].mode == Mode::Follow;
        let sp = world.roster().get(species as usize)?.clone();
        let name = sp.name();
        let keep = sp.keep.as_ref();
        return match who {
            Who::Wild { .. } => (sp.refusal().is_none() && keep.is_some())
                .then(|| Offer::new(Act::Feed, name.clone())),
            Who::Kept { id } => {
                let body = save.records.get(record::ANIMAL, id)?.read::<AnimalBody>()?;
                let keep = keep?;
                if held == Some(Item::Good(Good::FERTILE_EGG))
                    && matches!(keep.young.birth, pbd_core::husbandry::Birth::Eggs { .. })
                {
                    return Some(Offer::new(Act::SetEggs, name.clone()));
                }
                let now = farm_now(world, yard, save, id, &body);
                let waiting: Vec<String> = keep
                    .products
                    .iter()
                    .filter(|p| now.waiting(&p.good) > 0 && p.good().is_some())
                    .map(|p| tr(&format!("husbandry.collect.{}", p.good)).into_string())
                    .collect();
                if !waiting.is_empty() {
                    return Some(Offer::new(
                        Act::Collect,
                        Localized::verbatim(waiting.join(", ")),
                    ));
                }
                if held.is_some_and(|i| keep.eats(i)) {
                    return Some(Offer::new(Act::Feed, name.clone()));
                }
                Some(Offer::new(
                    if following { Act::LetStay } else { Act::Lead },
                    name,
                ))
            }
        };
    }
    let point = aimed?;
    if let Some(cell) = breed::incubator_at(world, save, point) {
        let holds = pbd_core::husbandry::breeding::eggs_at(
            &save.records,
            pbd_core::husbandry::breeding::Place::Incubator(cell),
        );
        if held == Some(Item::Good(Good::FERTILE_EGG)) {
            return Some(Offer::new(
                Act::SetInIncubator,
                tr("husbandry.thing.incubator"),
            ));
        }
        if holds == 0 && held.is_none() {
            return Some(Offer::new(
                Act::TakeIncubator,
                tr("husbandry.thing.incubator"),
            ));
        }
        if holds > 0 {
            return None;
        }
    }
    let (edge, _, _) = edge_at(world, point)?;
    let gate = *record::pieces(&save.records).get(&edge)?;
    Some(Offer::new(
        Act::TakeUp,
        if gate {
            tr("husbandry.thing.gate")
        } else {
            tr("husbandry.thing.fence")
        },
    ))
}

/// Offer the husbandry hands' act to the interact key.
fn offer_hands(
    world: World,
    aimed: Res<AimedGround>,
    mut yard: ResMut<Farmyard>,
    hotbar: Option<Res<Hotbar>>,
    save: Option<Res<WorldSave>>,
    mut interact: ResMut<crate::interact::Interact>,
) {
    let (Some(hotbar), Some(save)) = (hotbar, save) else {
        return;
    };
    if !world.walking.as_ref().is_some_and(|w| w.active) {
        return;
    }
    if let Some(offer) = offer(&world, &mut yard, &hotbar, &save, aimed.0) {
        interact.offer(offer);
    }
}

/// The interact key and the right button: feed, tame, collect, lead, and
/// place or take back a fence piece.
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
    interact: Option<Res<crate::interact::Interact>>,
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
    let used = keys.as_ref().is_some_and(|k| k.just_pressed(KeyCode::KeyF))
        && crate::interact::may(interact.as_deref(), crate::interact::Reader::Husbandry);
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
        keys.clear_just_pressed(KeyCode::KeyF);
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
            let centre = b.pos + world.origin().up_at(b.pos) * sp.height_m * 0.5;
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

/// F on an animal.
fn use_on(world: &World, yard: &mut Farmyard, hotbar: &mut Slots, save: &mut WorldSave, k: usize) {
    let roster = world.roster().to_vec();
    let s = world.settings().clone();
    let (who, species) = (yard.beasts[k].who, yard.beasts[k].species);
    let Some(sp) = roster.get(species as usize) else {
        return;
    };
    let name = sp.name();
    let held = hotbar.held().map(|st| st.item);
    match who {
        Who::Wild { .. } => {
            if let Some(why) = sp.refusal() {
                yard.say(
                    tr_with("husbandry.refused", &[("name", &name), ("why", &why)]),
                    2.5,
                );
                return;
            }
            let keep = sp.keep.as_ref().expect("tameable");
            if !held.is_some_and(|i| keep.eats(i)) {
                yard.say(
                    tr_with(
                        "husbandry.eats",
                        &[("name", &name), ("food", &keep.food_name())],
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
                    let what = if good == Good::FERTILE_EGG {
                        "fertile_egg"
                    } else {
                        p.good.as_str()
                    };
                    got.push(
                        tr_with(&format!("husbandry.collected.{what}"), &[("count", &took)])
                            .into_string(),
                    );
                }
            }
            if !got.is_empty() {
                body.set_farm(&now);
                store_animal(save, &body, id, Some(hotbar));
                yard.say(
                    tr_with(
                        "husbandry.collected_line",
                        &[("name", &name), ("got", &got.join(", "))],
                    ),
                    2.5,
                );
                return;
            }
            if held.is_some_and(|i| sp.keep.as_ref().is_some_and(|k| k.eats(i))) {
                if !spend_one(hotbar) {
                    return;
                }
                now.hand_feed(world.day(), &s.needs);
                body.set_farm(&now);
                store_animal(save, &body, id, Some(hotbar));
                let who = yard.beasts[k].who;
                yard.hearts.push((who, 5));
                yard.say(
                    tr_with(
                        "husbandry.fed",
                        &[("name", &name), ("days", &s.needs.hand_feed_days)],
                    ),
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
                yard.say(tr_with("husbandry.follows", &[("name", &name)]), 2.5);
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
    let name = sp.name();
    let b = &mut yard.beasts[k];
    if !b.trust.feed(keep.trust_gain, factor) {
        // A feed that does not tame changes only the hands. The hearts say
        // how far it trusts, and it follows a while (`taming-hearts`).
        save.record_pack(hotbar);
        b.mode = Mode::Follow;
        b.mode_s = s.taming.wild_follow_s;
        b.head_goal = 1.2;
        let shown = (b.who, pbd_core::husbandry::taming::hearts(b.trust.value));
        yard.hearts.push(shown);
        yard.say(tr_with("husbandry.eats_from_hand", &[("name", &name)]), 2.0);
        return;
    }
    // Tamed: the record, at once, as the player's (decision 10).
    let day = world.day();
    let home = world.cell(world.query(b.pos)).map_or(0, |(_, key)| key);
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
    yard.hearts.push((Who::Kept { id }, 5));
    let b = &mut yard.beasts[k];
    let species = b.species;
    yard.deeds.push(Deed::Tamed { species });
    yard.say(tr_with("husbandry.tamed", &[("name", &name)]), 3.0);
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
        .map_or_else(|| tr("husbandry.it"), |s| s.name());
    // The farm state up to now under the old home, then the new home.
    let now = farm_now(world, yard, save, id, &body);
    body.set_farm(&now);
    let Some((_, cell)) = world.cell(world.query(yard.beasts[k].pos)) else {
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
            yard.say(tr_with("husbandry.penned", &[("name", &name)]), 2.5);
        }
        None => {
            b.range = Range::Around(world.origin().up_at(b.pos), 6.0);
            yard.say(tr_with("husbandry.loose", &[("name", &name)]), 2.5);
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
    let Some(feet) = world.feet_point() else {
        return false;
    };
    if point.distance(feet) > world.settings().taming.reach_m * 2.5 {
        yard.say(tr("husbandry.too_far"), 1.5);
        return false;
    }
    let Some((edge, _, _)) = edge_at(world, point) else {
        return false;
    };
    if record::pieces(&save.records).contains_key(&edge) {
        yard.say(tr("husbandry.piece_there"), 1.5);
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
    yard.say(
        if gate {
            tr("husbandry.gate_set")
        } else {
            tr("husbandry.fence_set")
        },
        1.0,
    );
    closed_pen(world, yard, save, edge);
    true
}

/// F on a piece: take it back. Whether there was one to take.
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
        yard.say(tr("husbandry.no_room"), 1.5);
        return true;
    }
    let piece = FenceBody {
        gate,
        state: Standing::Removed,
    };
    save.store_with_hands(piece.record(edge), record::FENCE_SCHEMA, hotbar);
    yard.say(
        if gate {
            tr("husbandry.gate_taken")
        } else {
            tr("husbandry.fence_taken")
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
                tr_with("husbandry.pen_closed", &[("cells", &pen.cells.len())]),
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

/// MEASUREMENT (`world-frames` 10.7): `PBD_HERD_TRACE=<path.csv>` writes one
/// row per frame per animal that strode: its planet-frame feet before and
/// after the stride (f64, through the origin), the stride it was asked
/// (`facing * speed * dt`), and the origin, so the move can be compared with
/// what was asked along the ground. Changes nothing simulated.
struct HerdTrace(std::io::BufWriter<std::fs::File>, u64);

fn herd_trace() -> Option<HerdTrace> {
    use std::io::Write as _;
    let path = std::env::var_os("PBD_HERD_TRACE")?;
    let mut out = std::io::BufWriter::new(std::fs::File::create(&path).ok()?);
    writeln!(
        out,
        "frame,animal,mode,dt_s,ask_x,ask_y,ask_z,before_x,before_y,before_z,after_x,after_y,after_z,origin_x,origin_y,origin_z"
    )
    .ok()?;
    info!(
        "HERD_TRACE writing {}",
        std::path::Path::new(&path).display()
    );
    Some(HerdTrace(out, 0))
}

/// Move and animate every animal.
fn step(
    world: World,
    mut yard: ResMut<Farmyard>,
    hotbar: Option<Res<Hotbar>>,
    mut trace: Local<Option<Option<HerdTrace>>>,
    (save, survival, fire): (
        Option<Res<WorldSave>>,
        Option<Res<crate::survival::Survival>>,
        (
            Option<Res<crate::wildfire::FireNow>>,
            Option<Res<crate::wildfire::FireRules>>,
        ),
    ),
) {
    let trace = trace.get_or_insert_with(herd_trace);
    if let Some(t) = trace.as_mut() {
        t.1 += 1;
    }
    let dt = world.time.delta_secs().min(0.1);
    if dt <= 0.0 {
        return;
    }
    let s = world.settings().clone();
    let roster = world.roster().to_vec();
    // Everything the step compares is local (`world-frames` 10.2).
    let o = world.origin();
    let ground = |up: Vec3| world.ground_precise(up);
    let feet = world.feet_local();
    let moving = world.walking.as_ref().is_some_and(|w| w.speed > 0.6);
    let held = hotbar.as_ref().and_then(|h| h.held()).map(|st| st.item);
    let hours = dt / DAY_S * 24.0;
    let fade = dt / s.herds.fade_s;
    // The hunt (combat C2): Survival only, never on a downed player.
    let mode = hunt::mode_of(save.as_deref());
    let rules = crate::survival::rules();
    let player_down = survival
        .as_deref()
        .is_some_and(|sv| sv.dead || sv.vitals.as_ref().is_some_and(|v| v.consciousness.is_out()));
    let mut bites = Vec::new();
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
        pbd_core::world_frame::check_local("animal", b.pos);
        let up = o.up_at(b.pos);
        let night = world.night_at(up);
        let bird = sp.plan == Plan::Bird;
        if !b.posed {
            bites.extend(hunt::drive(
                b,
                sp,
                rules,
                mode,
                feet,
                up,
                night,
                player_down,
                dt,
            ));
        }
        // A carcass lies where it fell (design 17).
        if b.game.dead_s.is_some() {
            b.speed = 0.0;
            b.goal = None;
            b.mode = Mode::Idle;
            b.swing = 0.0;
            b.pos = stand_on(&o, b.pos, ground);
            continue;
        }
        // What it wants.
        b.mode_s -= dt;
        if let (Who::Wild { .. }, Some(feet)) = (b.who, feet)
            && !b.posed
            && b.game.hunt.phase == pbd_core::survival::predator::Phase::Wander
            && b.mode != Mode::Hunt
        {
            b.trust.decay(hours, &s.taming);
            let d = feet.distance(b.pos);
            let food = sp
                .keep
                .as_ref()
                .is_some_and(|kp| held.is_some_and(|i| kp.eats(i)));
            let fear = taming::fear_radius(&s.taming, moving, food);
            if b.mode == Mode::Follow {
                // Fed from the hand: it follows a while, unafraid
                // (`taming-hearts`).
            } else if d < fear && !night {
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
        // Fire (`wildfire` design 10): every animal, wild, kept or hunting,
        // runs from a burning cell within the flight's reach, at its run;
        // a herd's members, standing together, run the same way.
        if !b.posed
            && let (Some(now), Some(rules)) = (fire.0.as_deref(), fire.1.as_deref())
        {
            if let Some(goal) = fire_goal(&rules.0, now, &o, b.pos) {
                b.mode = Mode::Flee;
                b.mode_s = 2.5;
                b.goal = Some(goal);
            } else if b.mode == Mode::Flee && b.mode_s <= 0.0 {
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
                b.goal = wander_goal(b, bird, &o);
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
            Mode::Hunt => hunt::hunt_pace(b),
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
            Range::Pen(_, cells) if b.mode != Mode::Follow => world
                .cell(world.query(next))
                .is_none_or(|(_, key)| cells.contains(&key)),
            _ => true,
        };
        let travelled = if allowed && b.speed > 0.0 {
            let before = b.pos;
            b.pos = stride(&o, b.pos, b.facing, b.speed, dt, ground);
            if let Some(t) = trace.as_mut() {
                use std::io::Write as _;
                let (w0, w1) = (o.to_world(before).pos, o.to_world(b.pos).pos);
                let ask = moved.as_dvec3();
                let _ = writeln!(
                    t.0,
                    "{},{},{:?},{:?},{:.9},{:.9},{:.9},{:.9},{:.9},{:.9},{:.9},{:.9},{:.9},{},{},{}",
                    t.1,
                    match b.who {
                        Who::Wild { seed } => seed,
                        Who::Kept { id } => id,
                    },
                    b.mode,
                    dt,
                    ask.x,
                    ask.y,
                    ask.z,
                    w0.x,
                    w0.y,
                    w0.z,
                    w1.x,
                    w1.y,
                    w1.z,
                    o.origin.x,
                    o.origin.y,
                    o.origin.z
                );
            }
            moved.length()
        } else {
            if !allowed {
                b.goal = None;
                b.mode = Mode::Idle;
                b.speed = 0.0;
            }
            b.pos = stand_on(&o, b.pos, ground);
            0.0
        };
        let up = o.up_at(b.pos);
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
            let grazing = b
                .held_graze
                .unwrap_or(standing && b.glance_s > 0.6 && b.mode != Mode::Follow);
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
        if b.held_graze.is_some() {
            b.glance_goal = 0.0;
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
    yard.bites.extend(bites);
    yard.beasts
        .retain(|b| !(b.leaving && b.shown <= 0.0 && b.entity.is_none()));
}

/// `local` stood on the ground under it: moved along its own up to the
/// ground's radius there (`ground(up)`, f64), through the origin, so nothing
/// is rounded at the planet's scale (`world-frames` 10.2).
/// Where an animal at `local` runs to from the fire, if a burning cell is
/// within the flight's reach (`wildfire` design 10): 12 m on, away from
/// the nearest flame along the ground. Worked out in the planet's frame in
/// f64, the step a tangent direction, which is the same in the local frame.
pub(crate) fn fire_goal(
    cfg: &pbd_core::fire::FireConfig,
    now: &crate::wildfire::FireNow,
    origin: &SimOrigin,
    local: Vec3,
) -> Option<Vec3> {
    let away = crate::wildfire::flee_from(cfg, now, origin.to_world(local).pos)?;
    Some(local + away.as_vec3() * 12.0)
}

fn stand_on(origin: &SimOrigin, local: Vec3, ground: impl Fn(Vec3) -> f64) -> Vec3 {
    origin.with_radius(local, ground(origin.up_at(local)))
}

/// One stride of an animal: its feet moved `speed * dt` along `facing` and
/// stood on the ground, all in local space.
fn stride(
    origin: &SimOrigin,
    pos: Vec3,
    facing: Vec3,
    speed: f32,
    dt: f32,
    ground: impl Fn(Vec3) -> f64,
) -> Vec3 {
    stand_on(origin, pos + facing * speed * dt, ground)
}

/// A new place to potter to: a bird a short way, a beast a few metres,
/// always inside its range. Local, laid from the range's directions through
/// the origin.
fn wander_goal(b: &mut Beast, bird: bool, o: &SimOrigin) -> Option<Vec3> {
    let up = o.up_at(b.pos);
    let east = Vec3::Y.cross(up).normalize_or(Vec3::X);
    let north = up.cross(east);
    let r = o.radius_at(b.pos);
    match &b.range {
        Range::Pen(centres, _) => {
            let c = centres[(b.rng.next_u64() % centres.len() as u64) as usize];
            let jitter = (east * b.rng.range(-0.9, 0.9) + north * b.rng.range(-0.9, 0.9)) * 0.6;
            let goal = o.at(c.as_dvec3(), r) + jitter;
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
            let home = o.at(centre.as_dvec3(), r);
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
    let o = world.origin();
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
        let up = o.up_at(b.pos);
        let right = b.facing.cross(up).normalize_or(Vec3::Z);
        let pose = rig::Pose {
            phase: b.phase,
            swing: b.swing,
            // A blow throws the head up (combat C2); drawn only.
            head_down: b.head_down - b.game.flinch * 0.6,
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
            // A carcass lies, a warning predator crouches.
            down: hunt::drawn_down(b),
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
        // Local to render space, in f64 before the cast.
        let at = crate::render_origin::render_from_physics(&world.frame, b.pos);
        // A young animal is drawn at its stage's size (design 13, E1).
        let size = b
            .born
            .zip(sp.keep.as_ref())
            .map_or(1.0, |(born, k)| k.young.stage(b.sex, born, day).1);
        // A lunge's spring and nose (combat C2); drawn only.
        let (lift, pitch, _) = hunt::lunge_draw(b);
        let place = Transform::from_translation(at + up * lift)
            .with_rotation(Quat::from_axis_angle(right, pitch))
            .with_scale(Vec3::splat(size));
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
                        Name::new(sp.id.clone()),
                        LitByField,
                        LitLikeTerrain,
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
    let Some((materials, repeat)) = yard.pen_materials.clone() else {
        return;
    };
    let mut pen = PenMeshes {
        repeat,
        ..PenMeshes::default()
    };
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
            &mut pen,
            pa - o,
            pb - o,
            (ca + cb).normalize_or(Vec3::Y),
            gate,
        );
        drawn += 1;
    }
    drawn += breed::draw_incubators(&world, &save, &mut pen, &mut origin);
    yard.fences_drawn = Some((pieces, incubators, version));
    if let Some(children) = children {
        for c in children.iter() {
            commands.entity(c).despawn();
        }
    }
    let Some(o) = origin else {
        return;
    };
    let Some(flat) = yard.flat.clone() else {
        return;
    };
    commands.entity(root).insert(Transform::from_translation(
        (world.frame.center + o.as_dvec3()).as_vec3(),
    ));
    let parts = [
        (pen.timber, materials[0].clone()),
        (pen.poles, materials[1].clone()),
        (pen.boards, materials[2].clone()),
        (pen.plank, materials[3].clone()),
        (pen.glass, flat),
    ];
    for (builder, material) in parts {
        if builder.positions.is_empty() {
            continue;
        }
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        builder.write(&mut mesh);
        commands.entity(root).with_child((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(material),
            Transform::default(),
        ));
    }
    debug!("husbandry: {drawn} fence pieces drawn");
}

/// A piece between two corners on the ground: two posts and two rails; a
/// gate adds a brace.
fn fence_piece(pen: &mut PenMeshes, a: Vec3, b: Vec3, up: Vec3, gate: bool) {
    let along = b - a;
    let side = along.cross(up).normalize_or(Vec3::Z);
    let fwd = along.normalize_or(Vec3::X);
    // The posts stand on the corners in timber, the rails run between them
    // in poles; a gate is boards (M26's `pen()`).
    for at in [a, b] {
        pen.timber.textured_cuboid(
            at + up * 0.6,
            [fwd * 0.07, up * 0.65, side * 0.07],
            pen.repeat[0],
        );
    }
    let (rails, repeat) = if gate {
        (&mut pen.boards, pen.repeat[2])
    } else {
        (&mut pen.poles, pen.repeat[1])
    };
    let mut rail = |h: f32| {
        let mid = (a + b) * 0.5 + up * h;
        rails.textured_cuboid(mid, [along * 0.5, up * 0.05, side * 0.03], repeat);
    };
    rail(0.35);
    rail(0.85);
    if gate {
        rail(0.6);
        // A brace from bottom corner to top corner.
        let from = a + up * 0.35;
        let to = b + up * 0.85;
        let brace = to - from;
        let bup = brace.cross(side).normalize_or(up);
        pen.boards.textured_cuboid(
            (from + to) * 0.5,
            [brace * 0.5, bup * 0.045, side * 0.035],
            repeat,
        );
    }
}

/// The pen's meshes, one per texture (`animal-husbandry` design 14): posts
/// and legs in timber, rails in poles, gates in boards, an incubator's body
/// in plank, and its glass flat. `repeat` is each texture's metres, in that
/// order.
#[derive(Default)]
pub(crate) struct PenMeshes {
    pub(crate) timber: rig::Builder,
    pub(crate) poles: rig::Builder,
    pub(crate) boards: rig::Builder,
    pub(crate) plank: rig::Builder,
    pub(crate) glass: rig::Builder,
    pub(crate) repeat: [f32; 4],
}

/// The pen's textures by name, in [`PenMeshes`] order.
const PEN_TEXTURES: [&str; 4] = ["timber", "poles", "boards", "plank"];

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
    let sex = if b.sex == Sex::M {
        tr("husbandry.sex.male")
    } else {
        tr("husbandry.sex.female")
    };
    // A young one is named by its stage: a chick, a lamb (design 13, E1).
    let young = b
        .born
        .and_then(|born| breed::stage_now(world, b.species, b.sex, born))
        .filter(|(_, size)| *size < 1.0)
        .map(|(name, _)| tr(&format!("husbandry.stage.{name}")).into_string())
        .unwrap_or_default();
    let sitting = b.sitting;
    let name = sp.name();
    let named = if young.is_empty() {
        name.clone()
    } else {
        tr_with("husbandry.young", &[("name", &name), ("stage", &young)])
    };
    match b.who {
        Who::Wild { .. } => Some(match (sp.refusal(), sp.keep.as_ref()) {
            (Some(why), _) => {
                tr_with("husbandry.wild_refused", &[("name", &name), ("why", &why)]).into_string()
            }
            (None, Some(k)) => tr_with(
                "husbandry.wild",
                &[
                    ("name", &name),
                    ("sex", &sex),
                    ("trust", &format!("{:.0}", b.trust.value)),
                    ("food", &k.food_name()),
                ],
            )
            .into_string(),
            _ => name.into_string(),
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
                    (w > 0).then(|| {
                        tr_with(&format!("husbandry.waiting.{}", p.good), &[("count", &w)])
                            .into_string()
                    })
                })
                .collect();
            let c_mood = if now.sick(n) {
                tr("husbandry.mood.sick")
            } else if now.strayed(n) {
                tr("husbandry.mood.strayed")
            } else {
                tr_with(
                    "husbandry.mood.health",
                    &[("health", &format!("{:.0}", now.health))],
                )
            };
            let state = match (body.pen.is_some(), sitting) {
                (true, false) => tr_with("husbandry.state.penned", &[("sex", &sex)]),
                (false, false) => tr_with("husbandry.state.loose", &[("sex", &sex)]),
                (true, true) => tr_with("husbandry.state.penned_sitting", &[("sex", &sex)]),
                (false, true) => tr_with("husbandry.state.loose_sitting", &[("sex", &sex)]),
            };
            Some(if waiting.is_empty() {
                tr_with(
                    "husbandry.kept",
                    &[("name", &named), ("state", &state), ("mood", &c_mood)],
                )
                .into_string()
            } else {
                tr_with(
                    "husbandry.kept_waiting",
                    &[
                        ("name", &named),
                        ("state", &state),
                        ("mood", &c_mood),
                        ("waiting", &waiting.join(", ")),
                    ],
                )
                .into_string()
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
    let (Some((_, look)), Some(feet_point)) = (world.eye(), world.feet_point()) else {
        return;
    };
    if world.cell(feet_point).is_none() {
        return;
    }
    let up = feet_point.normalize();
    let ahead = (look - up * look.dot(up)).normalize_or(up.any_orthonormal_vector());
    let right = ahead.cross(up);
    let roster = world.roster().to_vec();
    let index = |id: &str| roster.iter().position(|s| s.id == id).unwrap_or(0) as u16;
    let wild = |yard: &mut Farmyard, sp: u16, n: u64, at: Vec3| -> usize {
        let seed = (0x5c_u64 << 56) | (u64::from(sp) << 16) | n;
        let s = &roster[sp as usize];
        let genome = s.keep.as_ref().map_or_else(Genome::default, |k| {
            Genome::roll(seed, k.coats.len(), k.patterns.len())
        });
        let sex = if n == 0 { Sex::M } else { Sex::F };
        let d = at.normalize();
        let mut b = Beast::new(
            Who::Wild { seed },
            sp,
            sex,
            genome,
            world.stood(d),
            d,
            Range::Around(d, 3.0),
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
    let spot = feet_point + ahead * 7.0;
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
        "close" | "close-stand" | "close-graze" | "game" | "game-stand" => {
            // One of each species of the herd broadside a few metres ahead,
            // held where it stands, to look at close up; `close-stand` and
            // `close-graze` hold its head up or down to graze as well.
            // `game` and `game-stand` are the game of `survival-mode` design
            // 17 (task 4.11) the same way: the hare, the ridge goat, the
            // rootling, the red fox and the dust hound.
            let held = match script.scene.as_str() {
                "close-stand" | "game-stand" => Some(false),
                "close-graze" => Some(true),
                _ => None,
            };
            let herd: &[(&str, f32, f32)] = if script.scene.starts_with("game") {
                &[
                    ("hare", -2.0, 2.3),
                    ("red-fox", -0.9, 3.0),
                    ("ridge-goat", 0.6, 3.6),
                    ("rootling", 2.3, 4.3),
                    ("dust-hound", 4.1, 5.0),
                ]
            } else {
                &[
                    ("meadow-hen", -1.9, 2.4),
                    ("woolback", -0.6, 3.4),
                    ("plodder", 1.6, 4.6),
                    ("horse", 4.6, 5.6),
                ]
            };
            for (n, &(id, x, y)) in herd.iter().enumerate() {
                if !roster.iter().any(|s| s.id == id) {
                    continue;
                }
                let k = wild(
                    &mut yard,
                    index(id),
                    n as u64 + 1,
                    feet_point + right * x + ahead * y,
                );
                let d = world.query(yard.beasts[k].pos).normalize();
                yard.beasts[k].range = Range::Around(d, 0.05);
                yard.beasts[k].facing = right;
                yard.beasts[k].idle_s = 1.0e6;
                yard.beasts[k].posed = true;
                yard.beasts[k].held_graze = held;
            }
        }
        "tame" => {
            let k = wild(&mut yard, index("woolback"), 1, feet_point + ahead * 3.2);
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
                    feet_point + ahead * 5.0 + right * (n as f32 - 1.0) * 1.2,
                );
            }
        }
        "pen" | "collect" => {
            let Some((_, centre)) = world.cell(spot) else {
                return;
            };
            build_ring(&world, &mut yard, hotbar, save, centre, feet_point);
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
            build_ring(&world, &mut yard, hotbar, save, centre, feet_point);
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
                    if select_item(hotbar, Item::Good(Good::FERTILE_EGG)) {
                        use_on(&world, &mut yard, hotbar, save, second);
                    }
                }
                breed::rebase(save, 5.0);
                hotbar.give(Item::Good(Good::INCUBATOR), 1);
                if select_item(hotbar, Item::Good(Good::INCUBATOR)) {
                    let at = feet_point + ahead * 2.5 - right * 1.5;
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
    select_where(hotbar, |item| keep.eats(item));
}

/// Put an item in hand, from the pack if need be. Whether there was one.
fn select_item(hotbar: &mut Slots, item: Item) -> bool {
    select_where(hotbar, |i| i == item)
}

/// Put the first carried stack that `want`s in hand, swapping it from the
/// pack into the last hotbar slot if need be. Whether there was one.
fn select_where(hotbar: &mut Slots, want: impl Fn(Item) -> bool) -> bool {
    let Some(k) =
        (0..pbd_core::inventory::CARRIED).find(|&k| hotbar.get(k).is_some_and(|s| want(s.item)))
    else {
        return false;
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
    true
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

/// One row of hearts rising over an animal fed from the hand
/// (`taming-hearts` design 2): a UI emote placed where the head projects.
#[derive(Component)]
struct HeartRow {
    who: Who,
    age: f32,
    width: f32,
}

/// How long a row of hearts shows, s; how far it rises, m; one heart's
/// size on screen, px (the 11 x 10 icon at twice its size).
const HEARTS_S: f32 = 1.5;
const HEARTS_RISE_M: f32 = 0.6;
const HEART_PX: (f32, f32) = (22.0, 20.0);

/// Spawn the rows a feed asked for, then move and fade every row.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn hearts(
    mut commands: Commands,
    world: World,
    time: Res<Time>,
    mut yard: ResMut<Farmyard>,
    assets: Option<Res<AssetServer>>,
    ui_scale: Option<Res<UiScale>>,
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    mut rows: Query<(Entity, &mut HeartRow, &mut Node, &mut Visibility, &Children)>,
    mut images: Query<&mut ImageNode>,
) {
    let fresh = std::mem::take(&mut yard.hearts);
    if let Some(assets) = assets.as_deref() {
        let heart: Handle<Image> = assets.load("ui/heart.png");
        for (who, n) in fresh {
            let width = HEART_PX.0 * f32::from(n) + 2.0 * f32::from(n.saturating_sub(1));
            commands
                .spawn((
                    HeartRow {
                        who,
                        age: 0.0,
                        width,
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        column_gap: Val::Px(2.0),
                        ..default()
                    },
                    Visibility::Hidden,
                    GlobalZIndex(5),
                ))
                .with_children(|row| {
                    for _ in 0..n {
                        row.spawn((
                            ImageNode::new(heart.clone()),
                            Node {
                                width: Val::Px(HEART_PX.0),
                                height: Val::Px(HEART_PX.1),
                                ..default()
                            },
                        ));
                    }
                });
        }
    }
    let camera = cameras.iter().find(|(_, c)| c.is_active);
    let scale = ui_scale.map_or(1.0, |s| s.0);
    let roster = world.roster();
    let o = world.origin();
    let dt = time.delta_secs();
    for (entity, mut row, mut node, mut shown, children) in &mut rows {
        row.age += dt;
        if row.age > HEARTS_S {
            commands.entity(entity).despawn();
            continue;
        }
        let beast = yard.beasts.iter().find(|b| b.who == row.who);
        let spot = beast.zip(camera).and_then(|(b, (at, camera))| {
            let tall = roster.get(b.species as usize).map_or(0.8, |sp| sp.height_m);
            let up = o.up_at(b.pos);
            let head = b.pos + up * (tall + 0.25 + HEARTS_RISE_M * row.age / HEARTS_S);
            let render = crate::render_origin::render_from_physics(&world.frame, head);
            camera.world_to_viewport(at, render).ok()
        });
        let Some(px) = spot else {
            *shown = Visibility::Hidden;
            continue;
        };
        *shown = Visibility::Inherited;
        node.left = Val::Px(px.x / scale - row.width / 2.0);
        node.top = Val::Px(px.y / scale - HEART_PX.1);
        // Fade over the last third.
        let alpha = ((HEARTS_S - row.age) / (HEARTS_S / 3.0)).clamp(0.0, 1.0);
        for child in children.iter() {
            if let Ok(mut image) = images.get_mut(child) {
                image.color = Color::srgba(1.0, 1.0, 1.0, alpha);
            }
        }
    }
}
