//! The beavers and otters, drawn (`ocean-environments` design B28, B29;
//! tasks 20.3 and 8.4): M25 version 5's sculpted, skinned, furred bodies and
//! their motion, on the fauna's own creatures.
//!
//! - **The creatures are the fauna's.** The spawner puts a beaver, a river
//!   otter or a sea otter in the water, the floater mover moves it and the
//!   spawner drops it. This module draws the nearest of them (at most
//!   [`MAX_DRAWN`], within [`DRAW_M`]) with a skinned body and an
//!   [`actor::Actor`] that poses it; the schools' mesh leaves them out.
//! - **Drawn only.** The actor reads where its creature is and whether it is
//!   startled; nothing reads the actor. On the bank the drawn animal leaves
//!   its creature's place for a few metres and comes back to it.
//! - **One system, nothing allocated a frame.** Every drawn animal's pose is
//!   plain numbers; its twenty bones are written in place and Bevy skins on
//!   the GPU. Rigs are kept and reused, never spawned per frame.
//! - **Lit as the terrain is.** Each root carries `LitByField` and
//!   `LitLikeTerrain`; under water its fur takes the water's dimming at its
//!   depth (B23), and wet fur is darker.

pub mod actor;
pub mod motion;
pub mod rig;

#[cfg(test)]
mod tests;

use crate::field_light::{Faded, FieldLitMaterial, LitByField, LitLikeTerrain};
use crate::fish::Fishery;
use crate::planet::{PlanetContact, PlanetRenderFrame};
use actor::{Actor, Ground, Held, Kind};
use bevy::asset::RenderAssetUsages;
use bevy::math::{DMat3, DVec3};
use bevy::mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes};
use bevy::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use pbd_core::world_frame::WorldPos;
use rig::{Animal, BONES, Spec};

/// At most this many animals are drawn at once, the nearest first.
pub const MAX_DRAWN: usize = 8;
/// They are drawn within this of the eye, m.
pub const DRAW_M: f32 = 70.0;
/// How long a drawn animal takes to fade in or out, s: the schools' fade.
const FADE_S: f32 = 0.8;
/// How far the drawn animal may wander from its frame's middle before the
/// frame is moved under it, m.
const RECENTRE_M: f32 = 12.0;

/// `PBD_CRITTER_PLAN=<plan>`, a capture instrument (B29): every animal
/// drawn begins with that plan instead of its first seeded draw (`bank`,
/// `sit`, `slap`, `haul`, `porpoise`, `groom`, `look`, `crack`, `dive`,
/// `sleep`; see [`actor::Actor::force`]), so a fixed-step capture shows a
/// chosen action. Unset, each picks its own. It changes nothing a player
/// meets.
fn forced_plan() -> Option<&'static str> {
    static PLAN: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    PLAN.get_or_init(|| std::env::var("PBD_CRITTER_PLAN").ok())
        .as_deref()
}

/// The motion's tunables, from `creature_motion.ron`.
#[derive(Resource, Clone, Debug, PartialEq, Default)]
pub struct CreatureMotionConfig(pub motion::CreatureMotion);

pub struct CrittersPlugin;

impl Plugin for CrittersPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Drawn>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    prewarm.run_if(resource_exists::<crate::loading::Warming>),
                    draw.after(crate::fish::FishSet),
                ),
            );
    }
}

// ---------------------------------------------------------------- the bodies

/// A body as `tools/art/critter_sculpt.py --game` writes it (`PBDCRIT1`).
#[derive(Clone, Debug)]
pub struct Body {
    pub rest: [Vec3; 20],
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub joints: Vec<[u16; 4]>,
    pub weights: Vec<[f32; 4]>,
    pub indices: Vec<u16>,
}

fn bytes_of(animal: Animal) -> (&'static [u8], &'static [u8]) {
    match animal {
        Animal::Beaver => (
            include_bytes!("../../../assets/models/critters/beaver.bin"),
            include_bytes!("../../../assets/textures/critters/fur-beaver.png"),
        ),
        Animal::RiverOtter => (
            include_bytes!("../../../assets/models/critters/river_otter.bin"),
            include_bytes!("../../../assets/textures/critters/fur-river_otter.png"),
        ),
        Animal::SeaOtter => (
            include_bytes!("../../../assets/models/critters/sea_otter.bin"),
            include_bytes!("../../../assets/textures/critters/fur-sea_otter.png"),
        ),
    }
}

/// Read a `PBDCRIT1` body.
pub fn parse(bytes: &[u8]) -> Result<Body, String> {
    let mut at = 0usize;
    let mut take = |n: usize| -> Result<&[u8], String> {
        let s = bytes
            .get(at..at + n)
            .ok_or_else(|| format!("a critter body ends at {} of {}", at, bytes.len()))?;
        at += n;
        Ok(s)
    };
    if take(8)? != b"PBDCRIT1" {
        return Err("not a PBDCRIT1 body".into());
    }
    let mut u32s = [0u32; 4];
    for v in &mut u32s {
        *v = u32::from_le_bytes(take(4)?.try_into().unwrap());
    }
    let [version, bones, n, tris] = u32s;
    if version != 1 || bones as usize != BONES.len() {
        return Err(format!("version {version} with {bones} bones"));
    }
    let (n, tris) = (n as usize, tris as usize);
    let f32s = |b: &[u8]| -> Vec<f32> {
        b.chunks_exact(4)
            .map(|c| f32::from_le_bytes(c.try_into().unwrap()))
            .collect()
    };
    let rest_f = f32s(take(12 * 20)?);
    let mut rest = [Vec3::ZERO; 20];
    for (k, r) in rest.iter_mut().enumerate() {
        *r = Vec3::new(rest_f[3 * k], rest_f[3 * k + 1], rest_f[3 * k + 2]);
    }
    let p = f32s(take(12 * n)?);
    let nr = f32s(take(12 * n)?);
    let uv = f32s(take(8 * n)?);
    let si = take(4 * n)?.to_vec();
    let sw = take(4 * n)?.to_vec();
    let idx: Vec<u16> = take(6 * tris)?
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    if idx.iter().any(|i| *i as usize >= n) {
        return Err("a triangle names a vertex past the end".into());
    }
    let positions = p.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect();
    let normals = nr.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect();
    let uvs = uv.chunks_exact(2).map(|c| [c[0], c[1]]).collect();
    let mut joints = Vec::with_capacity(n);
    let mut weights = Vec::with_capacity(n);
    for v in 0..n {
        let j = &si[4 * v..4 * v + 4];
        if j.iter().any(|b| *b as usize >= BONES.len()) {
            return Err("a vertex names a bone past the twentieth".into());
        }
        joints.push([j[0] as u16, j[1] as u16, j[2] as u16, j[3] as u16]);
        let w = &sw[4 * v..4 * v + 4];
        let sum: f32 = w.iter().map(|b| *b as f32).sum();
        let s = if sum > 0.0 { 1.0 / sum } else { 0.0 };
        let mut ww = [
            w[0] as f32 * s,
            w[1] as f32 * s,
            w[2] as f32 * s,
            w[3] as f32 * s,
        ];
        if sum <= 0.0 {
            ww = [1.0, 0.0, 0.0, 0.0];
        }
        weights.push(ww);
    }
    Ok(Body {
        rest,
        positions,
        normals,
        uvs,
        joints,
        weights,
        indices: idx,
    })
}

pub fn body_of(animal: Animal) -> Body {
    parse(bytes_of(animal).0).unwrap_or_else(|e| panic!("{} body: {e}", animal.name()))
}

fn mesh_of(body: Body) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, body.positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, body.normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, body.uvs);
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_JOINT_INDEX,
        VertexAttributeValues::Uint16x4(body.joints),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, body.weights);
    mesh.insert_indices(Indices::U16(body.indices));
    mesh
}

fn fur_image(animal: Animal) -> Image {
    let pixels = image::load_from_memory(bytes_of(animal).1)
        .expect("assets/textures/critters/fur-*.png")
        .to_rgba8();
    let mut image = Image::from_dynamic(
        image::DynamicImage::ImageRgba8(pixels),
        true,
        RenderAssetUsages::default(),
    );
    image.sampler = bevy::image::ImageSampler::nearest();
    image
}

/// Each animal's mesh, fur and bind pose, and the props' meshes.
#[derive(Resource)]
struct Kit {
    mesh: [Handle<Mesh>; 3],
    fur: [Handle<Image>; 3],
    bind: [Handle<SkinnedMeshInverseBindposes>; 3],
    stick: Handle<Mesh>,
    urchin: Handle<Mesh>,
    stone: Handle<Mesh>,
    stick_mat: Handle<StandardMaterial>,
    urchin_mat: Handle<StandardMaterial>,
    stone_mat: Handle<StandardMaterial>,
}

fn index(animal: Animal) -> usize {
    match animal {
        Animal::Beaver => 0,
        Animal::RiverOtter => 1,
        Animal::SeaOtter => 2,
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut binds: ResMut<Assets<SkinnedMeshInverseBindposes>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = Animal::ALL.map(|a| meshes.add(mesh_of(body_of(a))));
    let fur = Animal::ALL.map(|a| images.add(fur_image(a)));
    let bind = Animal::ALL.map(|a| {
        let rest = body_of(a).rest;
        binds.add(SkinnedMeshInverseBindposes::from(
            rest.iter()
                .map(|r| Mat4::from_translation(-*r))
                .collect::<Vec<_>>(),
        ))
    });
    let prop = |colour: Color| StandardMaterial {
        base_color: colour,
        perceptual_roughness: 0.9,
        reflectance: 0.2,
        ..default()
    };
    // The stick: the bark of the beaver's own sticks cell, a thin rod along
    // x from its end; the urchin and the stone the mockup's colours.
    let stick = meshes.add(
        Cuboid::new(0.6, 0.03, 0.03)
            .mesh()
            .build()
            .translated_by(Vec3::new(-0.3, 0.0, 0.0)),
    );
    let urchin = meshes.add(Sphere::new(0.045).mesh().ico(1).expect("an urchin"));
    let stone = meshes.add(Sphere::new(0.045).mesh().ico(0).expect("a stone"));
    commands.insert_resource(Kit {
        mesh,
        fur,
        bind,
        stick,
        urchin,
        stone,
        stick_mat: materials.add(prop(Color::srgb(0.45, 0.33, 0.22))),
        urchin_mat: materials.add(prop(Color::srgb(0.31, 0.11, 0.51))),
        stone_mat: materials.add(prop(Color::srgb(0.48, 0.46, 0.42))),
    });
}

// ---------------------------------------------------------------- the frame

/// A small flat frame round an animal (x east, y up, z south, metres from
/// a point on the sphere), turned to and from planet points in f64.
#[derive(Clone, Copy, Debug)]
pub struct Tangent {
    base: DVec3,
    r0: f64,
    basis: DMat3,
}

impl Tangent {
    /// The frame at a planet point, its height reckoned from that radius.
    pub fn at(planet: DVec3) -> Self {
        let up = planet.try_normalize().unwrap_or(DVec3::Y);
        let east = DVec3::Y.cross(up).try_normalize().unwrap_or(DVec3::X);
        let south = east.cross(up);
        Self {
            base: planet,
            r0: planet.length(),
            basis: DMat3::from_cols(east, up, south),
        }
    }

    /// The planet direction under a point of the frame.
    pub fn dir(&self, x: f32, z: f32) -> DVec3 {
        (self.base + self.basis.x_axis * f64::from(x) + self.basis.z_axis * f64::from(z))
            .try_normalize()
            .unwrap_or(DVec3::Y)
    }

    /// A point of the frame on the planet: its height along its own up.
    pub fn to_planet(&self, p: Vec3) -> DVec3 {
        self.dir(p.x, p.z) * (self.r0 + f64::from(p.y))
    }

    pub fn from_planet(&self, planet: DVec3) -> Vec3 {
        let d = planet - self.base;
        Vec3::new(
            d.dot(self.basis.x_axis) as f32,
            (planet.length() - self.r0) as f32,
            d.dot(self.basis.z_axis) as f32,
        )
    }

    pub fn height(&self, radius: f64) -> f32 {
        (radius - self.r0) as f32
    }

    fn rotation(&self) -> Quat {
        Quat::from_mat3(&self.basis.as_mat3())
    }
}

/// The ground and the water under a drawn animal, from the column tier.
struct PlanetGround<'a> {
    contact: Option<&'a PlanetContact>,
    frame: Tangent,
    sea_radius: Option<f64>,
}

impl PlanetGround<'_> {
    fn floor(&self, dir: DVec3) -> f64 {
        crate::vehicles::ground_under(self.contact, dir * self.frame.r0)
    }
}

impl Ground for PlanetGround<'_> {
    fn ground(&self, x: f32, z: f32) -> f32 {
        self.frame.height(self.floor(self.frame.dir(x, z)))
    }

    fn level(&self, x: f32, z: f32) -> Option<f32> {
        let dir = self.frame.dir(x, z);
        let floor = self.floor(dir);
        if let Some(contact) = self.contact
            && let Some(top) = crate::fish::fresh_top(contact, dir.as_vec3(), floor as f32)
        {
            return Some(self.frame.height(f64::from(top)));
        }
        let sea = self.sea_radius?;
        (floor < sea).then(|| self.frame.height(sea))
    }
}

// ---------------------------------------------------------------- drawing

/// One drawn animal: its rig's entities, its actor and its frame.
struct Slot {
    animal: Animal,
    root: Entity,
    bones: [Entity; 20],
    skin: Entity,
    stick: Entity,
    urchin: Entity,
    stone: Entity,
    /// The creature it draws: its school's seed and its id.
    who: Option<(u64, u32)>,
    actor: Actor,
    frame: Tangent,
    /// 0 gone to 1 whole.
    shown: f32,
    /// The colour last given its fur, and how wet.
    tint: [f32; 4],
    /// A plan to begin with, on its first frame (`PBD_CRITTER_PLAN`).
    plan: Option<&'static str>,
}

/// The drawn animals, and the candidates a frame looks at (kept, so a frame
/// allocates nothing once warm).
#[derive(Resource, Default)]
pub struct Drawn {
    slots: Vec<Slot>,
    near: Vec<(f32, usize, usize)>,
}

impl Drawn {
    /// How many animals are drawn now.
    pub fn count(&self) -> usize {
        self.slots.iter().filter(|s| s.who.is_some()).count()
    }
}

/// A rig: the root, its twenty bones in the mockup's hierarchy, the skin
/// under it and the props, all hidden until it is given a creature.
#[allow(clippy::too_many_arguments)]
fn spawn_rig(
    commands: &mut Commands,
    kit: &Kit,
    materials: &mut Assets<StandardMaterial>,
    animal: Animal,
    at: Transform,
    visible: bool,
    proxy: bool,
) -> (Entity, [Entity; 20], Entity, [Entity; 3]) {
    let spec = animal.spec();
    let offsets = spec.offsets();
    let vis = if visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    let root = commands
        .spawn((
            Name::new(format!("{} (drawn)", animal.name())),
            LitByField,
            LitLikeTerrain,
            Faded(if visible { 0.0 } else { 1.0 }),
            at,
            vis,
        ))
        .id();
    if proxy {
        commands.entity(root).insert(crate::loading::PrewarmProxy);
    }
    let mut bones = [Entity::PLACEHOLDER; 20];
    for k in 0..20 {
        let parent = rig::PARENT[k].map_or(root, |p| bones[p]);
        bones[k] = commands
            .spawn((
                Transform::from_translation(offsets[k]),
                Visibility::Inherited,
                ChildOf(parent),
            ))
            .id();
    }
    let fur = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(kit.fur[index(animal)].clone()),
        perceptual_roughness: 0.88,
        reflectance: 0.3,
        // The tufts are one face each.
        cull_mode: None,
        double_sided: true,
        ..default()
    });
    let skin = commands
        .spawn((
            Mesh3d(kit.mesh[index(animal)].clone()),
            MeshMaterial3d(fur),
            SkinnedMesh {
                inverse_bindposes: kit.bind[index(animal)].clone(),
                joints: bones.to_vec(),
            },
            bevy::light::NotShadowCaster,
            bevy::camera::visibility::NoFrustumCulling,
            Transform::default(),
            Visibility::Inherited,
            ChildOf(root),
        ))
        .id();
    let prop = |commands: &mut Commands, mesh: &Handle<Mesh>, mat: &Handle<StandardMaterial>| {
        commands
            .spawn((
                Mesh3d(mesh.clone()),
                MeshMaterial3d(mat.clone()),
                bevy::light::NotShadowCaster,
                Transform::default(),
                if proxy {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                },
                ChildOf(root),
            ))
            .id()
    };
    let props = [
        prop(commands, &kit.stick, &kit.stick_mat),
        prop(commands, &kit.urchin, &kit.urchin_mat),
        prop(commands, &kit.stone, &kit.stone_mat),
    ];
    if proxy {
        commands.entity(skin).insert(crate::loading::PrewarmProxy);
        for p in props {
            commands.entity(p).insert(crate::loading::PrewarmProxy);
        }
    }
    (root, bones, skin, props)
}

/// A skinned field-lit mesh is a pipeline of its own: each animal and its
/// props shown once, tiny, in front of the eye behind the loading screen, so
/// it is built before `READY` (`loading` decision 6). The proxies go at
/// `READY`.
fn prewarm(
    mut commands: Commands,
    kit: Option<Res<Kit>>,
    warming: Res<crate::loading::Warming>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cameras: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    mut done: Local<bool>,
) {
    if *done || warming.immediate() {
        return;
    }
    let Some(kit) = kit else {
        return;
    };
    let Some(eye) = cameras
        .iter()
        .find(|(camera, _)| camera.is_active)
        .map(|(_, at)| at.translation() + at.forward() * 2.0)
    else {
        return;
    };
    for animal in Animal::ALL {
        let at = Transform::from_translation(eye).with_scale(Vec3::splat(1e-4));
        spawn_rig(&mut commands, &kit, &mut materials, animal, at, true, true);
    }
    *done = true;
}

/// Every drawn animal for a frame: which creatures are drawn, each actor
/// stepped, its bones written and its root placed.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn draw(
    mut commands: Commands,
    mut drawn: ResMut<Drawn>,
    kit: Option<Res<Kit>>,
    fishery: Res<Fishery>,
    (fauna, body, motion): (
        Res<crate::config::FaunaConfig>,
        Res<crate::fish::Body>,
        Option<Res<CreatureMotionConfig>>,
    ),
    (frame, contact, sea, water): (
        Res<PlanetRenderFrame>,
        Option<Res<PlanetContact>>,
        Option<Res<crate::sea::Sea>>,
        Option<Res<crate::config::WaterSettings>>,
    ),
    time: Res<Time>,
    cameras: Query<&GlobalTransform, With<Camera3d>>,
    mut transforms: Query<&mut Transform>,
    mut visibility: Query<&mut Visibility>,
    mut faded: Query<&mut Faded>,
    lit: Query<&MeshMaterial3d<FieldLitMaterial>>,
    mut lit_materials: ResMut<Assets<FieldLitMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(kit) = kit else {
        return;
    };
    let defaults;
    let m = match motion.as_deref() {
        Some(m) => &m.0,
        None => {
            defaults = motion::CreatureMotion::default();
            &defaults
        }
    };
    let dt = time.delta_secs().min(0.1);
    let roster = fauna.0.roster(&body.0);
    let o = crate::sim_origin::from_render_frame(&frame);
    let Some(eye) = cameras.iter().next() else {
        return;
    };
    let eye = o
        .to_world(crate::sim_origin::local_from_render(
            &frame,
            eye.translation(),
        ))
        .pos;
    let alpha = fishery.step_alpha(fauna.0.spawn.step_hz);
    let drawn = &mut *drawn;
    // The candidates, nearest first: (distance, school, creature).
    drawn.near.clear();
    for (s, school) in fishery.schools.iter().enumerate() {
        let Some(species) = roster.get(school.species as usize) else {
            continue;
        };
        if Animal::of_species(&species.id).is_none() {
            continue;
        }
        for i in 0..school.len() {
            let p = o.to_world(school.drawn(i, alpha)).pos;
            let d = p.distance(eye) as f32;
            if d < DRAW_M {
                drawn.near.push((d, s, i));
            }
        }
    }
    drawn.near.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
    drawn.near.truncate(MAX_DRAWN);
    // Slots whose creature is no longer near let it go (and fade out).
    for slot in &mut drawn.slots {
        if let Some((seed, id)) = slot.who {
            let kept = drawn.near.iter().any(|(_, s, i)| {
                let school = &fishery.schools[*s];
                school.seed == seed && school.ids.get(*i) == Some(&id)
            });
            if !kept {
                slot.who = None;
            }
        }
    }
    // Creatures not drawn yet take a free slot of their animal, or a new one.
    for k in 0..drawn.near.len() {
        let (_, s, i) = drawn.near[k];
        let school = &fishery.schools[s];
        let id = school.ids.get(i).copied().unwrap_or(i as u32);
        if drawn
            .slots
            .iter()
            .any(|slot| slot.who == Some((school.seed, id)))
        {
            continue;
        }
        let Some(animal) = roster
            .get(school.species as usize)
            .and_then(|sp| Animal::of_species(&sp.id))
        else {
            continue;
        };
        let planet = o.to_world(school.drawn(i, alpha)).pos;
        let frame_here = Tangent::at(planet);
        let free = drawn
            .slots
            .iter()
            .position(|slot| slot.who.is_none() && slot.animal == animal && slot.shown <= 0.0);
        let slot = match free {
            Some(f) => f,
            None if drawn.slots.len() < MAX_DRAWN => {
                let (root, bones, skin, props) = spawn_rig(
                    &mut commands,
                    &kit,
                    &mut materials,
                    animal,
                    Transform::default(),
                    false,
                    false,
                );
                drawn.slots.push(Slot {
                    animal,
                    root,
                    bones,
                    skin,
                    stick: props[0],
                    urchin: props[1],
                    stone: props[2],
                    who: None,
                    actor: Actor::new(animal, Vec3::ZERO, 0.0, 0),
                    frame: frame_here,
                    shown: 0.0,
                    tint: [1.0; 4],
                    plan: None,
                });
                drawn.slots.len() - 1
            }
            None => continue,
        };
        let slot = &mut drawn.slots[slot];
        slot.who = Some((school.seed, id));
        slot.frame = frame_here;
        let seed = (school.seed as u32) ^ id.wrapping_mul(0x9E37_79B9);
        slot.actor = Actor::new(animal, Vec3::ZERO, (seed % 628) as f32 / 100.0, seed);
        slot.plan = forced_plan();
        info!(
            "critters: a {} drawn {:.0} m off",
            animal.name(),
            drawn.near[k].0
        );
    }
    let under = crate::fish::places::UnderWater::of(water.as_deref());
    for slot in &mut drawn.slots {
        // Its creature this frame, if it still has one.
        let creature = slot.who.and_then(|(seed, id)| {
            let school = fishery.schools.iter().find(|s| s.seed == seed)?;
            let i = school.ids.iter().position(|x| *x == id)?;
            Some((school, i))
        });
        let fade_to = if creature.is_some() { 1.0 } else { 0.0 };
        slot.shown = if fade_to > slot.shown {
            (slot.shown + dt / FADE_S).min(1.0)
        } else {
            (slot.shown - dt / FADE_S).max(0.0)
        };
        if slot.shown <= 0.0 && creature.is_none() {
            if let Ok(mut v) = visibility.get_mut(slot.root)
                && *v != Visibility::Hidden
            {
                *v = Visibility::Hidden;
            }
            continue;
        }
        // Shown (a rig spawned this frame is shown from its first frame in
        // the world).
        if let Ok(mut v) = visibility.get_mut(slot.root)
            && *v != Visibility::Visible
        {
            *v = Visibility::Visible;
        }
        let ground = PlanetGround {
            contact: contact.as_deref(),
            frame: slot.frame,
            sea_radius: sea.as_deref().map(|s| f64::from(s.radius)),
        };
        if let Some((school, i)) = creature {
            let planet = o.to_world(school.drawn(i, alpha)).pos;
            slot.actor.anchor = slot.frame.from_planet(planet);
            slot.actor.spooked = school.spook.get(i).is_some_and(|s| *s > 0.0);
        }
        if let Some(plan) = slot.plan.take() {
            slot.actor.force(&ground, plan);
            info!("critters: {} begins with {plan}", slot.animal.name());
        }
        slot.actor.update(&ground, m, dt);
        // The frame follows the animal, so its flat ground stays flat.
        let off = Vec2::new(slot.actor.pos.x, slot.actor.pos.z);
        if off.length_squared() > RECENTRE_M * RECENTRE_M {
            let planet = slot
                .frame
                .to_planet(Vec3::new(slot.actor.pos.x, 0.0, slot.actor.pos.z));
            let moved = Tangent::at(planet.normalize() * slot.frame.r0);
            let before = slot.actor.pos;
            let after = moved.from_planet(slot.frame.to_planet(before));
            slot.frame = moved;
            slot.actor.shift(after - before);
        }
        place(slot, &o, &frame, &mut transforms, &mut visibility);
        let school_fade = creature.map_or(0.0, |(school, _)| fishery.fade_of(school));
        let fade = school_fade.max(1.0 - slot.shown);
        if let Ok(mut f) = faded.get_mut(slot.root)
            && f.0 != fade
        {
            f.0 = fade;
        }
        // Under water its fur takes the water's share at its depth; wet fur
        // is darker.
        let a = &slot.actor;
        let level = ground.level(a.pos.x, a.pos.z);
        let depth = level.map_or(0.0, |l| l - (a.pos.y + a.out.y));
        let [r, g, b] = under.albedo([1.0; 3], depth.max(0.0));
        let wet = lerp(1.0, 0.68, a.wet);
        let tint = [r * wet, g * wet, b * wet, a.wet];
        if tint
            .iter()
            .zip(slot.tint)
            .any(|(x, y)| (x - y).abs() > 0.02)
            && let Ok(handle) = lit.get(slot.skin)
            && let Some(material) = lit_materials.get_mut(&handle.0)
        {
            material.base.base_color = Color::linear_rgb(tint[0], tint[1], tint[2]);
            material.base.perceptual_roughness = lerp(0.88, 0.5, a.wet);
            slot.tint = tint;
        }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// The root at the animal's place, its bones posed, its props held.
fn place(
    slot: &Slot,
    o: &pbd_core::world_frame::SimOrigin,
    frame: &PlanetRenderFrame,
    transforms: &mut Query<&mut Transform>,
    visibility: &mut Query<&mut Visibility>,
) {
    let a = &slot.actor;
    let spec: &Spec = a.animal.spec();
    let planet = slot.frame.to_planet(a.pos);
    let local = o.to_local(WorldPos::new(o.frame, planet));
    let yaw = Quat::from_rotation_y(a.yaw);
    if let Ok(mut t) = transforms.get_mut(slot.root) {
        t.translation = crate::render_origin::render_from_physics(frame, local);
        t.rotation = slot.frame.rotation() * yaw;
        t.scale = Vec3::ONE;
    }
    let rot = rig::rotations(&a.out);
    let tr = rig::translations(spec, &a.out);
    for k in 0..20 {
        if let Ok(mut t) = transforms.get_mut(slot.bones[k]) {
            t.translation = tr[k];
            t.rotation = rot[k];
        }
    }
    // The props: the stick in its teeth or on the ground under its paws, the
    // urchin between the sea otter's paws, the stone on its chest.
    let need_fk = a.held == Held::Stick || a.urchin.is_some() || a.stick_ground.is_some();
    let fk = need_fk.then(|| rig::fk(spec, &a.out));
    let show = |visibility: &mut Query<&mut Visibility>, e: Entity, on: bool| {
        if let Ok(mut v) = visibility.get_mut(e) {
            let want = if on {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            if *v != want {
                *v = want;
            }
        }
    };
    let step = a.step.map(|s| s.kind);
    // the stick
    let stick_at = match (a.held, a.stick_ground, &fk) {
        (Held::Stick, _, Some(fk)) => {
            let jaw = fk[rig::JAW].translation.into();
            Some(Transform::from_translation(jaw).with_rotation(Quat::from_rotation_z(-0.35)))
        }
        (_, Some(g), _) => {
            let inv = yaw.inverse();
            let at = inv * (g - a.pos);
            Some(
                Transform::from_translation(at + Vec3::new(0.0, 0.015, 0.0))
                    .with_rotation(Quat::from_rotation_y(1.2)),
            )
        }
        _ => None,
    };
    if let Some(t) = stick_at
        && let Ok(mut st) = transforms.get_mut(slot.stick)
    {
        *st = t;
    }
    show(visibility, slot.stick, stick_at.is_some());
    // the urchin while it forages and eats, the stone while it cracks
    let eating = matches!(
        step,
        Some(Kind::Crack { .. } | Kind::EatUrchin | Kind::SeaDive { up: true, .. })
    );
    let urchin_at = match (a.urchin, eating, &fk) {
        (Some(left), true, Some(fk)) => {
            let mid: Vec3 =
                ((fk[rig::foot(0)].translation + fk[rig::foot(1)].translation) * 0.5).into();
            Some(Transform::from_translation(mid).with_scale(Vec3::splat(left.max(0.2))))
        }
        _ => None,
    };
    if let Some(t) = urchin_at
        && let Ok(mut ut) = transforms.get_mut(slot.urchin)
    {
        *ut = t;
    }
    show(visibility, slot.urchin, urchin_at.is_some());
    let cracking = matches!(step, Some(Kind::Crack { .. }));
    let stone_at = match (cracking, &fk) {
        (true, Some(fk)) => Some(Transform::from_translation(
            fk[rig::CHEST].transform_point3(Vec3::new(0.17, -0.155, 0.0)),
        )),
        _ => None,
    };
    if let Some(t) = stone_at
        && let Ok(mut s) = transforms.get_mut(slot.stone)
    {
        *s = t;
    }
    show(visibility, slot.stone, stone_at.is_some());
}
