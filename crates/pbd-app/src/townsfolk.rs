//! The townspeople, in the world (`townsfolk` decisions 19 to 27): each
//! standing town's people derived by `pbd_core::townsfolk`, drawn as M5
//! version 9's Faces-tab figures, solid to the walker and lit by the field.
//!
//! - **Derived, never saved.** When a town stands, its people are laid from
//!   its template on its chart ([`on_stand`]); when it goes, they go.
//! - **Every person stands and is solid; the nearest are drawn.** Each frame
//!   every person of a standing town is placed (at their post, along their
//!   path, or where a caller drives them) and their circle is put in
//!   [`TownsfolkSolids`] for the walker. At most [`MAX_DRAWN`] within
//!   [`DRAW_M`] of the eye have a figure, faded in and out over [`FADE_S`].
//! - **One texture, two materials.** A figure's colours are swatches of one
//!   nearest-sampled atlas that also holds the mouth cells, so every figure
//!   is drawn with one opaque material and its mouth and happy eyes with one
//!   masked one (decision 21). Every mesh of a figure is a direct child of
//!   its root, which carries `LitByField`, `LitLikeTerrain` and `Faded`.
//! - **Drawn only.** The face and the body's motion read what happens to a
//!   person; nothing reads them.
//!
//! What other parts may use is [`Townsfolk`]'s `set_expression`, `play`,
//! `drive`, `release`, `placed` and `people`, and the [`TownPerson`] on
//! every drawn figure's root (decision 27).

#[cfg(test)]
mod tests;

use crate::field_light::{Faded, LitByField, LitLikeTerrain};
use crate::planet::PlanetRenderFrame;
use bevy::asset::RenderAssetUsages;
use bevy::math::{Affine3A, DQuat, DVec3};
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use pbd_core::townsfolk::face::{self, Act, Chosen, Expression, Face, FacePose, Mood, Mouth, Seen};
use pbd_core::townsfolk::figure::{self, Figure, Part};
use pbd_core::townsfolk::motion::{Body, Doing, Pose};
use pbd_core::townsfolk::{Person, PersonId, Placed, Work, look, people_of};
use pbd_core::world_frame::WorldPos;
use std::collections::HashMap;

pub use pbd_core::townsfolk::face::{Act as TownAct, Expression as TownExpression};

/// At most this many people have a figure at once, the nearest first.
pub const MAX_DRAWN: usize = 32;
/// Figures are drawn within this of the eye, m.
pub const DRAW_M: f32 = 60.0;
/// How long a figure takes to fade in or out, s.
pub const FADE_S: f32 = 0.8;
/// How fast a person turns to the way they go, s (decision 23).
const TURN_S: f64 = 0.12;

/// A drawn person, on their figure's root.
#[derive(Component, Clone, Debug, PartialEq)]
pub struct TownPerson {
    pub id: PersonId,
    pub role: String,
}

/// Every standing town's people's circles, for the walker (decision 25):
/// each one's feet, a planet point in f64.
#[derive(Resource, Clone, Debug, Default)]
pub struct TownsfolkSolids(pub Vec<DVec3>);

impl TownsfolkSolids {
    /// Whether a body at `centre` overlaps anyone.
    pub fn holds(&self, centre: DVec3, half_height: f32, radius: f32) -> bool {
        self.0
            .iter()
            .any(|f| pbd_core::townsfolk::overlaps(centre, half_height, radius, *f).is_some())
    }

    /// The way out of the nearest person it overlaps, a planet direction.
    pub fn push_normal(&self, centre: DVec3, half_height: f32, radius: f32) -> Option<Vec3> {
        self.0
            .iter()
            .filter_map(|f| {
                pbd_core::townsfolk::overlaps(centre, half_height, radius, *f)
                    .map(|n| ((centre - *f).length_squared(), n))
            })
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, n)| n.as_vec3())
    }
}

/// A person's state while their town stands.
struct Live {
    person: Person,
    mood: Mood,
    face: Face,
    body: Body,
    /// Where they are this frame, and the way they face (turned toward the
    /// way they go over [`TURN_S`]).
    placed: Placed,
    forward: DVec3,
    /// Where a caller stands them instead, and whether they are walking.
    driven: Option<Placed>,
    /// How they rest where they are driven: on a seat or a bed
    /// (`furniture` design 7).
    posture: Option<Posture>,
    chosen: Chosen,
    talking: bool,
}

/// How a person rests on a piece of furniture (`furniture` design 7): their
/// feet stood under a seat's place, or laid along a bed from its pillow.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Posture {
    /// On a seat this high over the floor, m.
    Sit { seat_m: f32 },
    /// On a bed whose mattress is this high, m; laid from where they are
    /// driven (the pillow) along their forward.
    Lie { top_m: f32 },
}

/// One standing town's people.
struct Folk {
    site: u32,
    live: Vec<Live>,
}

/// The townspeople of the standing towns, and their drawn figures.
#[derive(Resource, Default)]
pub struct Townsfolk {
    towns: Vec<Folk>,
    slots: Vec<Slot>,
    near: Vec<(f32, usize, usize)>,
    clock: f64,
}

impl Townsfolk {
    fn live_mut(&mut self, id: PersonId) -> Option<&mut Live> {
        self.towns
            .iter_mut()
            .find(|t| t.site == id.site)?
            .live
            .get_mut(id.index as usize)
    }

    fn live(&self, id: PersonId) -> Option<&Live> {
        self.towns
            .iter()
            .find(|t| t.site == id.site)?
            .live
            .get(id.index as usize)
    }

    /// A standing town's people, in its template's order.
    pub fn people(&self, site: u32) -> impl Iterator<Item = &Person> {
        self.towns
            .iter()
            .filter(move |t| t.site == site)
            .flat_map(|t| t.live.iter().map(|l| &l.person))
    }

    /// Every person of every standing town.
    pub fn everyone(&self) -> impl Iterator<Item = &Person> {
        self.towns
            .iter()
            .flat_map(|t| t.live.iter().map(|l| &l.person))
    }

    /// Where a person is this frame: their feet in the planet's frame (f64)
    /// and the way they face.
    pub fn placed(&self, id: PersonId) -> Option<Placed> {
        self.live(id).map(|l| Placed {
            forward: l.forward,
            ..l.placed
        })
    }

    /// Hold an expression on a person for `seconds`, over what their mood
    /// would pick (a trader's smile on a trade); an expression with an act
    /// (delighted's hop) plays it. Whether the person stands.
    pub fn set_expression(&mut self, id: PersonId, expression: Expression, seconds: f32) -> bool {
        let Some(l) = self.live_mut(id) else {
            return false;
        };
        l.mood.hold(expression, seconds);
        if let Some(act) = expression.params().act {
            l.body.play(act);
        }
        true
    }

    /// Play an act: a wave, a hop, a shrug or a tilt of the head.
    pub fn play(&mut self, id: PersonId, act: Act) -> bool {
        let Some(l) = self.live_mut(id) else {
            return false;
        };
        l.body.play(act);
        true
    }

    /// Whether a person talks (their mouth cycles its open shapes).
    pub fn set_talking(&mut self, id: PersonId, talking: bool) -> bool {
        let Some(l) = self.live_mut(id) else {
            return false;
        };
        l.talking = talking;
        true
    }

    /// Stand a person where the caller says instead of their post or path,
    /// until [`Townsfolk::release`]: `feet` a planet point (f64), `forward`
    /// a tangent there, `walking` whether they walk (a lost villager
    /// following the player). Their figure, face and circle follow.
    pub fn drive(&mut self, id: PersonId, feet: DVec3, forward: DVec3, walking: bool) -> bool {
        let Some(l) = self.live_mut(id) else {
            return false;
        };
        l.driven = Some(Placed {
            feet,
            forward,
            walking,
        });
        true
    }

    /// Rest a person on a piece of furniture (`furniture` design 7): their
    /// feet at `feet` (a seat's place on the floor, or a bed's pillow on the
    /// floor), facing `forward`, until [`Townsfolk::release`].
    pub fn rest(&mut self, id: PersonId, feet: DVec3, forward: DVec3, posture: Posture) -> bool {
        let Some(l) = self.live_mut(id) else {
            return false;
        };
        l.driven = Some(Placed {
            feet,
            forward,
            walking: false,
        });
        l.posture = Some(posture);
        true
    }

    /// Whether a person rests on furniture now.
    pub fn resting(&self, id: PersonId) -> Option<Posture> {
        self.live(id).and_then(|l| l.posture)
    }

    /// Back to their post or path.
    pub fn release(&mut self, id: PersonId) -> bool {
        let Some(l) = self.live_mut(id) else {
            return false;
        };
        l.driven = None;
        l.posture = None;
        true
    }

    /// How many figures are drawn now.
    pub fn drawn(&self) -> usize {
        self.slots.iter().filter(|s| s.who.is_some()).count()
    }
}

/// A town has stood: lay its people (called from `towns::stand`).
pub fn on_stand(
    world: &mut World,
    site: u32,
    town: &pbd_core::settlement::record::Town,
    laid: &crate::towns::Laid,
) {
    let Some(template) = world
        .get_resource::<crate::towns::TownAssets>()
        .and_then(|a| a.template_named(&town.template))
    else {
        return;
    };
    let people = people_of(town, template, &laid.chart, &laid.patch);
    let radius = f64::from(crate::planet::terrain_config().radius_m);
    let mut folk = world.get_resource_or_insert_with(Townsfolk::default);
    let t = folk.clock;
    let live: Vec<Live> = people
        .into_iter()
        .map(|person| {
            let placed = person.at(t, radius);
            let resting = person.look.resting;
            Live {
                mood: Mood::new(resting),
                face: Face::new(person.seed, resting),
                body: Body::new(person.seed),
                forward: placed.forward,
                placed,
                driven: None,
                posture: None,
                chosen: Chosen {
                    expression: resting,
                    focus: false,
                },
                talking: false,
                person,
            }
        })
        .collect();
    let mut live = live;
    if let Some(expression) = forced_expression() {
        for l in &mut live {
            l.mood.hold(expression, f32::MAX);
        }
        info!("townsfolk: every face held {}", expression.id());
    }
    info!("{}: {} townspeople", laid.name, live.len());
    folk.towns.retain(|f| f.site != site);
    folk.towns.push(Folk { site, live });
}

/// `PBD_TOWNSFOLK_EXPRESSION=<id>`, a capture instrument (decision 22):
/// every townsperson holds that expression (`happy`, `sleepy`, ...), so a
/// fixed-step capture shows a chosen face. Unset, faces follow what happens.
/// It changes nothing a player meets.
fn forced_expression() -> Option<Expression> {
    static FORCED: std::sync::OnceLock<Option<Expression>> = std::sync::OnceLock::new();
    *FORCED.get_or_init(|| {
        std::env::var("PBD_TOWNSFOLK_EXPRESSION")
            .ok()
            .and_then(|v| Expression::of_id(&v))
    })
}

/// A town has gone: its people go with it, their figures faded.
pub fn on_drop(world: &mut World, site: u32) {
    if let Some(mut folk) = world.get_resource_mut::<Townsfolk>() {
        folk.towns.retain(|f| f.site != site);
    }
}

/// Every town taken down (a new world).
pub fn on_replace(world: &mut World) {
    if let Some(mut folk) = world.get_resource_mut::<Townsfolk>() {
        folk.towns.clear();
    }
    world.insert_resource(TownsfolkSolids::default());
}

// ---------------------------------------------------------------- the kit

/// The atlas: the mouth cells at its top left, the swatches under them,
/// two texels each.
const ATLAS_W: usize = 128;
const ATLAS_H: usize = 64;
const SWATCH_Y: usize = 40;

/// What every figure is drawn with.
#[derive(Resource)]
struct Kit {
    /// Each colour's swatch's middle, as a UV.
    swatch: HashMap<u32, [f32; 2]>,
    solid: Handle<StandardMaterial>,
    masked: Handle<StandardMaterial>,
    /// The face's shared pieces.
    white: Handle<Mesh>,
    pupil: Handle<Mesh>,
    glint: Handle<Mesh>,
    lash: Handle<Mesh>,
    arc: Handle<Mesh>,
    mouths: HashMap<Mouth, Handle<Mesh>>,
}

impl Kit {
    fn uv(&self, colour: u32) -> [f32; 2] {
        self.swatch.get(&colour).copied().unwrap_or_else(|| {
            warn!("townsfolk: no swatch for {colour:06x}");
            [
                0.5 / ATLAS_W as f32,
                (SWATCH_Y as f32 + 0.5) / ATLAS_H as f32,
            ]
        })
    }

    /// A part as a mesh, each corner's UV at its colour's swatch.
    fn mesh(&self, part: &Part) -> Mesh {
        let uvs: Vec<[f32; 2]> = part.colours.iter().map(|c| self.uv(*c)).collect();
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, part.positions.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, part.normals.clone())
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    }
}

/// The atlas's pixels and each swatch's UV: the mouth cells as the
/// mockup's canvas draws them, and every colour a person can wear.
pub fn atlas_pixels(colours: &[u32]) -> (Vec<u8>, HashMap<u32, [f32; 2]>) {
    let mut px = vec![0u8; ATLAS_W * ATLAS_H * 4];
    let mouths = face::mouth_atlas();
    for y in 0..face::ATLAS_H {
        for x in 0..face::ATLAS_W {
            let at = (y * ATLAS_W + x) * 4;
            px[at..at + 4].copy_from_slice(&mouths[y * face::ATLAS_W + x]);
        }
    }
    let per_row = ATLAS_W / 2;
    let mut swatch = HashMap::new();
    for (i, &c) in colours.iter().enumerate() {
        let (sx, sy) = ((i % per_row) * 2, SWATCH_Y + (i / per_row) * 2);
        assert!(
            sy + 2 <= ATLAS_H,
            "the townsfolk atlas has no room for {} swatches",
            colours.len()
        );
        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
            let at = ((sy + dy) * ATLAS_W + sx + dx) * 4;
            px[at..at + 4].copy_from_slice(&[(c >> 16) as u8, (c >> 8) as u8, c as u8, 255]);
        }
        swatch.insert(
            c,
            [
                (sx as f32 + 1.0) / ATLAS_W as f32,
                (sy as f32 + 1.0) / ATLAS_H as f32,
            ],
        );
    }
    (px, swatch)
}

/// A quad of the atlas: a cell's pixels on a `w` x `h` quad centred at its
/// middle (the mouth) or with its top at 0 (none here).
fn cell_mesh(w: f32, h: f32, cell: Mouth) -> Mesh {
    let part = figure::quad(w, h, -h / 2.0, 0);
    let c = cell.cell();
    let (ox, oy) = ((c % 4) * face::CELL_W, (c / 4) * face::CELL_H);
    let (u0, u1) = (
        ox as f32 / ATLAS_W as f32,
        (ox + face::CELL_W) as f32 / ATLAS_W as f32,
    );
    let (v0, v1) = (
        oy as f32 / ATLAS_H as f32,
        (oy + face::CELL_H) as f32 / ATLAS_H as f32,
    );
    let uvs: Vec<[f32; 2]> = part
        .positions
        .iter()
        .map(|p| {
            [
                u0 + (p[0] / w + 0.5) * (u1 - u0),
                v1 - (p[1] / h + 0.5) * (v1 - v0),
            ]
        })
        .collect();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, part.positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, part.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

fn setup(
    mut commands: Commands,
    assets: Option<Res<crate::towns::TownAssets>>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let tunics: Vec<u32> = assets
        .as_deref()
        .map(|a| {
            [
                &a.village, &a.walled, &a.harbour, &a.desert, &a.tundra, &a.jungle,
            ]
            .into_iter()
            .flat_map(|t| t.people.iter().map(|p| p.tunic))
            .collect()
        })
        .unwrap_or_default();
    let colours = look::swatches(tunics);
    let (px, swatch) = atlas_pixels(&colours);
    let mut image = Image::new(
        Extent3d {
            width: ATLAS_W as u32,
            height: ATLAS_H as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        px,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = bevy::image::ImageSampler::nearest();
    let atlas = images.add(image);
    let base = |alpha_mode| StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(atlas.clone()),
        perceptual_roughness: 0.9,
        reflectance: 0.2,
        alpha_mode,
        ..default()
    };
    let solid = materials.add(base(AlphaMode::Opaque));
    let masked = materials.add(base(AlphaMode::Mask(0.5)));
    let mut kit = Kit {
        swatch,
        solid,
        masked,
        white: Handle::default(),
        pupil: Handle::default(),
        glint: Handle::default(),
        lash: Handle::default(),
        arc: Handle::default(),
        mouths: HashMap::new(),
    };
    let (wr, pr) = (figure::WR, figure::PR);
    kit.white = meshes.add(kit.mesh(&figure::disc(wr, figure::WHITE)));
    kit.pupil = meshes.add(kit.mesh(&figure::disc(pr, figure::PUPIL)));
    kit.glint = meshes.add(kit.mesh(&figure::disc(figure::GR, figure::GLINT)));
    kit.lash = meshes.add(kit.mesh(&figure::quad(2.04 * wr, 0.01, -0.005, figure::LASH)));
    kit.arc = meshes.add(cell_mesh(2.0 * wr, wr, Mouth::Happy));
    for m in Mouth::ALL {
        let [w, h] = figure::MOUTH_WH;
        kit.mouths.insert(m, meshes.add(cell_mesh(w, h, m)));
    }
    info!("townsfolk: an atlas of {} swatches", colours.len());
    commands.insert_resource(kit);
}

// ---------------------------------------------------------------- a figure

/// Each mesh entity of a figure, every one a direct child of its root.
#[derive(Clone, Copy)]
struct Parts {
    body: Entity,
    legs: [Entity; 2],
    arms: [Entity; 2],
    head: Entity,
    /// Per eye: the white, the pupil, the catch-light, the lid, the lash
    /// line and the happy arc.
    eyes: [[Entity; 6]; 2],
    brows: [Entity; 2],
    /// Per side, a blush per level.
    blush: [[Entity; 3]; 2],
    mouth: Entity,
}

/// One drawn figure.
struct Slot {
    root: Entity,
    parts: Parts,
    figure: Figure,
    who: Option<PersonId>,
    shown: f32,
    mouth: Mouth,
}

/// A figure: its root at `at`, its parts built from `look`.
fn spawn_figure(
    commands: &mut Commands,
    kit: &Kit,
    meshes: &mut Assets<Mesh>,
    person: &Person,
    at: Transform,
    proxy: bool,
) -> (Entity, Parts, Figure) {
    let mut figure = Figure::of(&person.look);
    if person.spear {
        figure.with_spear();
    }
    let root = commands
        .spawn((
            Name::new(format!(
                "Townsperson {}:{}",
                person.id.site, person.id.index
            )),
            TownPerson {
                id: person.id,
                role: person.role.clone(),
            },
            LitByField,
            LitLikeTerrain,
            Faded(if proxy { 0.0 } else { 1.0 }),
            at,
            if proxy {
                Visibility::Visible
            } else {
                Visibility::Hidden
            },
        ))
        .id();
    if proxy {
        commands.entity(root).insert(crate::loading::PrewarmProxy);
    } else {
        // The interact key talks to them (`townsfolk` design 5 to 12).
        commands.entity(root).insert(crate::economy::Talker {
            site: person.id.site,
            index: person.id.index,
            role: person.role.clone(),
        });
    }
    let mut part = |mesh: Handle<Mesh>, masked: bool| {
        commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(if masked {
                    kit.masked.clone()
                } else {
                    kit.solid.clone()
                }),
                bevy::light::NotShadowCaster,
                Transform::default(),
                Visibility::Inherited,
                ChildOf(root),
            ))
            .id()
    };
    let own = |meshes: &mut Assets<Mesh>, p: &Part| meshes.add(kit.mesh(p));
    let body = part(own(meshes, &figure.body), false);
    let legs = [0, 1].map(|i| part(own(meshes, &figure.legs[i]), false));
    let arms = [0, 1].map(|i| part(own(meshes, &figure.arms[i]), false));
    let head = part(own(meshes, &figure.head), false);
    let wr = figure::WR;
    let lid = own(meshes, &figure::quad(2.19 * wr, 1.0, -1.0, figure.skin));
    let brow = own(meshes, &figure::quad(0.1, 0.026, -0.013, figure.brow));
    let blushes = figure.blush.map(|c| own(meshes, &figure::disc(1.0, c)));
    let eyes = [0, 1].map(|_| {
        [
            part(kit.white.clone(), false),
            part(kit.pupil.clone(), false),
            part(kit.glint.clone(), false),
            part(lid.clone(), false),
            part(kit.lash.clone(), false),
            part(kit.arc.clone(), true),
        ]
    });
    let brows = [0, 1].map(|_| part(brow.clone(), false));
    let blush = [0, 1].map(|_| blushes.clone().map(|b| part(b, false)));
    let mouth = part(kit.mouths[&Mouth::Smile].clone(), true);
    (
        root,
        Parts {
            body,
            legs,
            arms,
            head,
            eyes,
            brows,
            blush,
            mouth,
        },
        figure,
    )
}

/// Where each part goes this frame, in the root's space: the mockup's
/// groups composed (decisions 21 and 23).
fn pose_parts(
    f: &Figure,
    pose: &Pose,
    face: &FacePose,
    look: &look::Look,
) -> Vec<(usize, Option<Affine3A>)> {
    let t = Affine3A::from_translation;
    let g = t(Vec3::new(0.0, pose.hop, 0.0));
    let body = g
        * t(Vec3::new(pose.body_x, f.leg_len + pose.body_y, 0.0))
        * Affine3A::from_rotation_z(pose.roll)
        * Affine3A::from_scale(Vec3::new(pose.scale[0], pose.scale[1], pose.scale[0]));
    let mut out = Vec::with_capacity(30);
    out.push((0, Some(body)));
    for i in 0..2 {
        out.push((
            1 + i,
            Some(body * t(f.leg_at[i]) * Affine3A::from_rotation_x(pose.legs[i])),
        ));
    }
    for i in 0..2 {
        let [x, z] = pose.arms[i];
        let q = Quat::from_euler(EulerRot::XYZ, x, 0.0, z);
        out.push((3 + i, Some(body * t(f.arm_at[i]) * Affine3A::from_quat(q))));
    }
    let hq = Quat::from_euler(
        EulerRot::YXZ,
        face.head_yaw,
        face.head_pitch + pose.head_nod,
        pose.head_roll + 0.26 * face.tilt,
    );
    let head = body
        * t(Vec3::new(f.head_at.x, f.head_at.y - pose.head_drop, 0.0))
        * Affine3A::from_quat(hq)
        * Affine3A::from_scale(Vec3::splat(f.head_scale));
    out.push((5, Some(head)));
    let fc = head * t(Vec3::new(0.0, 0.0, figure::HA));
    let (wr, pr) = (figure::WR, figure::PR);
    let lh = 1.83 * wr * face.lid;
    let ly = 0.913 * wr - lh + 0.0008;
    let lash_x = ((2.0 * (wr - ly.abs() / 1.732) + 0.01) / (2.04 * wr)).max(0.05);
    let es = face.eye * look.eye;
    for (i, s) in [(0usize, -1.0f32), (1, 1.0)] {
        let eye = fc
            * t(Vec3::new(s * look.gap, figure::EYE_Y, 0.0))
            * Affine3A::from_scale(Vec3::new(es, es, 1.0));
        let pg = eye
            * t(Vec3::new(
                face.pupil[0] * (0.98 * wr - pr * face.pup),
                face.pupil[1] * (0.846 * wr - 0.87 * pr * face.pup),
                0.003,
            ));
        let open = !face.arcs;
        let base = 6 + i * 6;
        out.push((base, open.then(|| eye * t(Vec3::new(0.0, 0.0, 0.002)))));
        out.push((
            base + 1,
            open.then(|| pg * Affine3A::from_scale(Vec3::splat(face.pup))),
        ));
        out.push((
            base + 2,
            open.then(|| pg * t(Vec3::new(-0.017, 0.018, 0.001))),
        ));
        out.push((
            base + 3,
            (open && lh > 0.003).then(|| {
                eye * t(Vec3::new(0.0, 0.913 * wr, 0.005))
                    * Affine3A::from_scale(Vec3::new(1.0, lh, 1.0))
            }),
        ));
        out.push((
            base + 4,
            open.then(|| {
                eye * t(Vec3::new(0.0, ly, 0.006))
                    * Affine3A::from_scale(Vec3::new(lash_x, 1.0, 1.0))
            }),
        ));
        out.push((
            base + 5,
            face.arcs
                .then(|| eye * t(Vec3::new(0.0, 0.077 * wr, 0.003))),
        ));
        let by = figure::BROW_Y + face.brow_y[i] + (es - 1.0) * 0.065;
        out.push((
            18 + i,
            Some(
                fc * t(Vec3::new(s * (look.gap + 0.01), by, 0.003))
                    * Affine3A::from_rotation_z(face.brow_tilt[i]),
            ),
        ));
        // The blush at the level nearest the face's.
        let level = (face.blush > 0.04).then(|| {
            look::BLUSH_LEVELS
                .iter()
                .enumerate()
                .min_by(|a, b| {
                    (a.1 - face.blush)
                        .abs()
                        .total_cmp(&(b.1 - face.blush).abs())
                })
                .map_or(0, |(k, _)| k)
        });
        for k in 0..3 {
            out.push((
                20 + i * 3 + k,
                (level == Some(k)).then(|| {
                    fc * t(Vec3::new(s * 0.155, 0.178, 0.0015))
                        * Affine3A::from_scale(Vec3::new(0.052, 0.029, 1.0))
                }),
            ));
        }
    }
    out.push((
        26,
        Some(fc * t(Vec3::new(face.mouth_x, figure::MOUTH_Y, 0.002))),
    ));
    out
}

impl Parts {
    /// The entity of part `k` in [`pose_parts`]'s numbering.
    fn get(&self, k: usize) -> Entity {
        match k {
            0 => self.body,
            1 | 2 => self.legs[k - 1],
            3 | 4 => self.arms[k - 3],
            5 => self.head,
            6..=17 => self.eyes[(k - 6) / 6][(k - 6) % 6],
            18 | 19 => self.brows[k - 18],
            20..=25 => self.blush[(k - 20) / 3][(k - 20) % 3],
            _ => self.mouth,
        }
    }
}

/// The frame at a person's feet: x across, y up, z the way they face.
fn basis(feet: DVec3, forward: DVec3) -> Quat {
    let up = up_of(feet);
    let f = (forward - up * forward.dot(up))
        .try_normalize()
        .unwrap_or_else(|| {
            let side = if up.x.abs() < 0.9 { DVec3::X } else { DVec3::Y };
            (side - up * side.dot(up)).normalize()
        });
    let x = up.cross(f);
    Quat::from_mat3(&Mat3::from_cols(x.as_vec3(), up.as_vec3(), f.as_vec3()))
}

/// Up at a planet point (f64, the planet's frame).
fn up_of(planet: DVec3) -> DVec3 {
    planet.normalize_or(DVec3::Y)
}

/// Turn `from` toward `to` about `up` by the share `k`.
fn turn_toward(from: DVec3, to: DVec3, up: DVec3, k: f64) -> DVec3 {
    let a = from.cross(to).dot(up).atan2(from.dot(to));
    (DQuat::from_axis_angle(up, a * k) * from).normalize()
}

// ---------------------------------------------------------------- each frame

/// A field-lit figure is a pipeline of its own: one shown behind the
/// loading screen, tiny, in front of the eye, so its two materials are
/// built before `READY` (`loading` decision 6).
fn prewarm(
    mut commands: Commands,
    kit: Option<Res<Kit>>,
    warming: Res<crate::loading::Warming>,
    mut meshes: ResMut<Assets<Mesh>>,
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
    let person = proxy_person();
    let at = Transform::from_translation(eye).with_scale(Vec3::splat(1e-4));
    let (_, parts, _) = spawn_figure(&mut commands, &kit, &mut meshes, &person, at, true);
    // Every piece shown, the arcs and blush too, so both materials draw.
    for e in parts
        .eyes
        .iter()
        .flatten()
        .chain(parts.blush.iter().flatten())
    {
        commands.entity(*e).insert(Visibility::Inherited);
    }
    *done = true;
}

fn proxy_person() -> Person {
    use pbd_core::townsfolk::Activity;
    Person {
        id: PersonId { site: 0, index: 0 },
        role: String::new(),
        spear: true,
        at: Vec3::Y,
        altitude_m: 0.0,
        facing: Vec3::Z,
        activity: Activity::Post { work: Work::Idle },
        look: look::generate(look::FIELDS, 1, 0, look::FIELDS.cloth[0]),
        seed: 1,
    }
}

/// Every standing town's people placed, their circles given to the walker,
/// the nearest drawn, and each drawn figure posed.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn step(
    mut commands: Commands,
    folk: Option<ResMut<Townsfolk>>,
    mut solids: ResMut<TownsfolkSolids>,
    kit: Option<Res<Kit>>,
    frame: Res<PlanetRenderFrame>,
    sun: Option<Res<crate::sky::Sun>>,
    time: Res<Time>,
    cameras: Query<
        (&Camera, &GlobalTransform),
        Or<(
            With<crate::walking::WalkingCamera>,
            With<crate::vehicles::VehicleCamera>,
        )>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut transforms: Query<&mut Transform>,
    mut visibility: Query<&mut Visibility>,
    mut faded: Query<&mut Faded>,
    mut mesh3d: Query<&mut Mesh3d>,
) {
    let (Some(mut folk), Some(kit)) = (folk, kit) else {
        return;
    };
    let folk = &mut *folk;
    let dt = time.delta_secs().min(0.1);
    folk.clock += f64::from(dt);
    let radius = f64::from(crate::planet::terrain_config().radius_m);
    let o = crate::sim_origin::from_render_frame(&frame);
    let eye = cameras
        .iter()
        .filter(|(c, _)| c.is_active)
        .max_by_key(|(c, _)| c.order)
        .map(|(_, at)| {
            o.to_world(crate::sim_origin::local_from_render(
                &frame,
                at.translation(),
            ))
            .pos
        });
    // Everyone placed, and their circles for the walker.
    solids.0.clear();
    let k_turn = 1.0 - (-f64::from(dt) / TURN_S).exp();
    for town in &mut folk.towns {
        for l in &mut town.live {
            l.placed = l.driven.unwrap_or_else(|| l.person.at(folk.clock, radius));
            let up = up_of(l.placed.feet);
            l.forward = turn_toward(l.forward, l.placed.forward, up, k_turn);
            solids.0.push(l.placed.feet);
            // The mood for everyone: a meeting is not missed off screen.
            let player_m = eye.map(|e| (e - l.placed.feet).length() as f32);
            let seen = Seen {
                player_m: player_m.filter(|d| *d < 2.0 * face::PART_M),
                night: sun
                    .as_deref()
                    .is_some_and(|s| s.clock.daylight(up.as_vec3()) < 0.12),
                hammering: l.person.work() == Some(Work::Hammer),
            };
            let was = l.chosen.expression;
            l.chosen = l.mood.step(dt, seen);
            if l.chosen.expression != was
                && let Some(act) = l.chosen.expression.params().act
                && act != Act::Tilt
            {
                l.body.play(act);
            }
        }
    }
    // Which are drawn: the nearest within reach.
    let Some(eye) = eye else {
        return;
    };
    folk.near.clear();
    for (t, town) in folk.towns.iter().enumerate() {
        for (i, l) in town.live.iter().enumerate() {
            let d = (l.placed.feet - eye).length() as f32;
            if d < DRAW_M {
                folk.near.push((d, t, i));
            }
        }
    }
    folk.near.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
    folk.near.truncate(MAX_DRAWN);
    let wanted: Vec<PersonId> = folk
        .near
        .iter()
        .map(|&(_, t, i)| folk.towns[t].live[i].person.id)
        .collect();
    for slot in &mut folk.slots {
        if slot.who.is_some_and(|id| !wanted.contains(&id)) {
            slot.who = None;
        }
    }
    for id in &wanted {
        if folk.slots.iter().any(|s| s.who == Some(*id)) {
            continue;
        }
        // A figure is built for its person: a free slot that has faded out
        // is taken down and built again, or a new one made.
        let free = folk
            .slots
            .iter()
            .position(|s| s.who.is_none() && s.shown <= 0.0);
        if free.is_none() && folk.slots.len() >= MAX_DRAWN {
            continue;
        }
        let Some(l) = folk.live(*id) else { continue };
        let (root, parts, figure) = spawn_figure(
            &mut commands,
            &kit,
            &mut meshes,
            &l.person,
            Transform::default(),
            false,
        );
        let slot = Slot {
            root,
            parts,
            figure,
            who: Some(*id),
            shown: 0.0,
            mouth: Mouth::Smile,
        };
        match free {
            Some(f) => {
                let old = std::mem::replace(&mut folk.slots[f], slot);
                commands.entity(old.root).despawn();
            }
            None => folk.slots.push(slot),
        }
    }
    // Each figure faded, posed and placed.
    let Townsfolk { towns, slots, .. } = folk;
    for slot in slots.iter_mut() {
        let live = slot.who.and_then(|id| {
            towns
                .iter_mut()
                .find(|t| t.site == id.site)?
                .live
                .get_mut(id.index as usize)
        });
        let to = if live.is_some() { 1.0 } else { 0.0 };
        slot.shown = if to > slot.shown {
            (slot.shown + dt / FADE_S).min(1.0)
        } else {
            (slot.shown - dt / FADE_S).max(0.0)
        };
        if let Ok(mut v) = visibility.get_mut(slot.root) {
            let want = if slot.shown > 0.0 {
                Visibility::Visible
            } else {
                Visibility::Hidden
            };
            if *v != want {
                *v = want;
            }
        }
        if let Ok(mut f) = faded.get_mut(slot.root) {
            let fade = 1.0 - slot.shown;
            if f.0 != fade {
                f.0 = fade;
            }
        }
        let Some(l) = live else {
            continue;
        };
        let up = up_of(l.placed.feet);
        let rot = basis(l.placed.feet, l.forward);
        // The player's eye as yaw and pitch from the body's forward, while
        // they are met.
        let head = l.placed.feet
            + up * f64::from(
                slot.figure.leg_len
                    + slot.figure.head_at.y
                    + figure::EYE_Y * slot.figure.head_scale,
            );
        let to_eye = eye - head;
        let near = to_eye.length() < f64::from(face::MEET_M);
        let look_at = near.then(|| {
            let v = rot.inverse() * to_eye.as_vec3();
            [
                v.x.atan2(v.z.max(0.05)),
                v.y.atan2(Vec2::new(v.x, v.z).length()),
            ]
        });
        let face_pose = l.face.step(dt, l.chosen, look_at, l.talking);
        let doing = match l.posture {
            Some(Posture::Sit { seat_m }) => {
                Doing::Sitting((slot.figure.leg_len - seat_m).max(0.0))
            }
            Some(Posture::Lie { .. }) => Doing::Lying,
            None if l.placed.walking => Doing::Walking(l.person.pace().unwrap_or(1.0)),
            None => match l.person.work() {
                Some(Work::Hammer) => Doing::Hammering,
                Some(Work::Wipe) => Doing::Wiping,
                _ => Doing::Standing,
            },
        };
        let pose = l.body.step(dt, doing, face_pose.tilt);
        // Lying: the figure laid on its back along the bed, its head on the
        // pillow where it is driven, its feet toward the bed's foot.
        let (feet, rot) = match l.posture {
            Some(Posture::Lie { top_m }) => {
                let up = up_of(l.placed.feet);
                let along = (l.placed.forward - up * l.placed.forward.dot(up))
                    .try_normalize()
                    .unwrap_or(DVec3::X);
                let head = -along;
                let x = head.cross(up);
                let lay =
                    Quat::from_mat3(&Mat3::from_cols(x.as_vec3(), head.as_vec3(), up.as_vec3()));
                let height = f64::from(slot.figure.height_m());
                let feet = l.placed.feet + along * (height - 0.12) + up * f64::from(top_m + 0.1);
                (feet, lay)
            }
            _ => (l.placed.feet, rot),
        };
        let local = o.to_local(WorldPos::new(o.frame, feet));
        if let Ok(mut t) = transforms.get_mut(slot.root) {
            t.translation = crate::render_origin::render_from_physics(&frame, local);
            t.rotation = rot;
            t.scale = Vec3::ONE;
        }
        for (k, at) in pose_parts(&slot.figure, &pose, &face_pose, &l.person.look) {
            let e = slot.parts.get(k);
            if let Ok(mut v) = visibility.get_mut(e) {
                let want = if at.is_some() {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
                if *v != want {
                    *v = want;
                }
            }
            if let Some(at) = at
                && let Ok(mut t) = transforms.get_mut(e)
            {
                *t = Transform::from_matrix(Mat4::from(at));
            }
        }
        if face_pose.mouth != slot.mouth
            && let Ok(mut m) = mesh3d.get_mut(slot.parts.mouth)
        {
            m.0 = kit.mouths[&face_pose.mouth].clone();
            slot.mouth = face_pose.mouth;
        }
    }
}

/// The townspeople.
pub struct TownsfolkPlugin;

impl Plugin for TownsfolkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Townsfolk>()
            .init_resource::<TownsfolkSolids>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    prewarm.run_if(resource_exists::<crate::loading::Warming>),
                    step.after(crate::towns::stand_in_range),
                ),
            );
    }
}
