//! What a craft looks like: procedural meshes built from the same numbers its
//! physics is (the hull's own loft, the blade's own area), and the parts
//! that move with its state - the Loon's paddle, the raft's oars and rower,
//! the tube rider's kicks - placed from that state every frame. Nothing here decides
//! anything.

use super::{Vehicle, view::VehicleView};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use pbd_core::vehicle::oars::{RowingSpec, elbow, elbow_pole};
use pbd_core::vehicle::{Craft, CraftState, Kind};
use std::f32::consts::FRAC_PI_2;

/// How far a recovering paddle blade is lifted clear of the water, m.
const RECOVERY_LIFT_M: f32 = 0.45;
const BLADE_HEIGHT_M: f32 = 0.45;

fn paddle_blade(area_m2: f32) -> Mesh {
    Cuboid::new(0.02, BLADE_HEIGHT_M, area_m2 / BLADE_HEIGHT_M).into()
}

/// A moving part and the craft it belongs to.
#[derive(Component)]
pub struct Part {
    owner: Entity,
    kind: Moving,
}

#[derive(Clone, Copy, PartialEq)]
enum Moving {
    Paddle,
    /// What lies in a boat until someone aboard takes it up: the canoe's
    /// paddle.
    Laid,
    /// A lantern's flame, burning while the dusk lamps do.
    Flame,
    /// The figure aboard, drawn only from the chase view.
    Crew,
    /// The tube rider's body from the hips, sitting up to paddle.
    TubeBody,
    /// A tube rider's arm on a side: 0 the upper arm, 1 the forearm, 2 the
    /// hand, paddling beside the ring while a stroke runs on its side.
    TubeArm(i8, u8),
    /// The raft's oar on a side, a pivot at its pin turned along the loom
    /// (`river-craft` design 9).
    Oar(i8),
    /// The oar's blade, feathered about the loom.
    Blade(i8),
    /// The swivel oarlock on a side, turning with the sweep.
    Lock(i8),
    /// The seated rower, drawn only from the chase view.
    Rower,
    /// The rower's body from the hip, leaning with the stroke.
    Hip,
    /// A rower's limb on a side: 0 the upper arm, 1 the forearm, 2 the
    /// block hand round the handle. A pivot scaled along the bone.
    Arm(i8, u8),
    /// The first-person hand on a side, closed on its oar's handle: drawn
    /// only in the seat view (design 9.6).
    Hand(i8),
    /// The boat's one anchor (`river-craft` design 4): on the deck while
    /// stowed (where the boat has a place for it), else where it is.
    Anchor,
    /// Its rode from the bow while it is out: a straight line.
    Rode,
}

impl Part {
    pub(super) fn named(owner: Entity, name: &str) -> Option<Self> {
        let kind = match name {
            "paddle" => Moving::Paddle,
            "crew" => Moving::Crew,
            _ => return None,
        };
        Some(Self { owner, kind })
    }
}

fn mesh(positions: Vec<Vec3>, indices: Vec<u32>) -> Mesh {
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_POSITION,
        positions.iter().map(|p| p.to_array()).collect::<Vec<_>>(),
    )
    .with_inserted_indices(Indices::U32(indices));
    mesh.duplicate_vertices();
    mesh.compute_flat_normals();
    mesh
}

/// A piece of a torus round +y: `major` to the middle of the tube, `minor`
/// round, over `arc` radians from +x, in `along` x `round` quads (M34's
/// pitched seam on the tube).
fn torus_arc(major: f32, minor: f32, arc: f32, along: u32, round: u32) -> Mesh {
    let mut positions = Vec::new();
    for i in 0..=along {
        let a = arc * i as f32 / along as f32;
        let (sa, ca) = a.sin_cos();
        for j in 0..round {
            let t = std::f32::consts::TAU * j as f32 / round as f32;
            let r = major + minor * t.cos();
            positions.push(Vec3::new(r * ca, minor * t.sin(), -r * sa));
        }
    }
    let mut indices = Vec::new();
    for i in 0..along {
        for j in 0..round {
            let k = |i: u32, j: u32| i * round + j % round;
            let (a, b, c, d) = (k(i, j), k(i + 1, j), k(i + 1, j + 1), k(i, j + 1));
            indices.extend([a, c, b, a, d, c]);
        }
    }
    mesh(positions, indices)
}

fn paint(world: &mut World, color: Color, double: bool) -> Handle<StandardMaterial> {
    world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.7,
            double_sided: double,
            cull_mode: if double {
                None
            } else {
                Some(bevy::render::render_resource::Face::Back)
            },
            ..default()
        })
}

struct Builder<'w> {
    world: &'w mut World,
    owner: Entity,
}

impl Builder<'_> {
    /// A fixed piece under `parent`.
    fn piece(
        &mut self,
        parent: Entity,
        mesh: Mesh,
        material: &Handle<StandardMaterial>,
        at: Transform,
    ) -> Entity {
        let handle = self.world.resource_mut::<Assets<Mesh>>().add(mesh);
        let child = self
            .world
            .spawn((Mesh3d(handle), MeshMaterial3d(material.clone()), at))
            .id();
        self.world.entity_mut(parent).add_child(child);
        child
    }

    /// A moving part: a pivot the state turns, under `parent`.
    fn pivot(&mut self, parent: Entity, kind: Moving, at: Transform) -> Entity {
        let owner = self.owner;
        let child = self
            .world
            .spawn((at, Visibility::default(), Part { owner, kind }))
            .id();
        self.world.entity_mut(parent).add_child(child);
        child
    }

    fn cuboid(&mut self, parent: Entity, size: Vec3, at: Vec3, paint: &Handle<StandardMaterial>) {
        self.piece(
            parent,
            Cuboid::from_size(size).into(),
            paint,
            Transform::from_translation(at),
        );
    }

    /// A limb from joint `a` to joint `b`: a block `size` thick, laid
    /// along the line between them.
    fn limb(
        &mut self,
        parent: Entity,
        a: Vec3,
        b: Vec3,
        size: Vec2,
        paint: &Handle<StandardMaterial>,
    ) {
        let along = b - a;
        let length = along.length().max(1e-3);
        let turn = Quat::from_rotation_arc(Vec3::Y, along / length);
        self.piece(
            parent,
            Cuboid::new(size.x, length, size.y).into(),
            paint,
            Transform::from_translation((a + b) / 2.0).with_rotation(turn),
        );
    }

    /// M34's lantern (`river-craft`): an iron cage top and bottom round
    /// dark glass, scaled; the glass glows while the dusk lamps burn.
    fn lantern(&mut self, parent: Entity, at: Vec3, scale: f32) {
        let iron = paint(self.world, Color::srgb_u8(0x4a, 0x4d, 0x52), false);
        let glass = paint(self.world, Color::srgb_u8(0x3a, 0x32, 0x26), false);
        let s = scale;
        for y in [0.17, -0.17] {
            self.cuboid(
                parent,
                Vec3::new(0.26 * s, 0.05 * s, 0.26 * s),
                at + Vec3::Y * y * s,
                &iron,
            );
        }
        self.cuboid(parent, Vec3::new(0.2 * s, 0.3 * s, 0.2 * s), at, &glass);
        let warm = self
            .world
            .resource_mut::<Assets<StandardMaterial>>()
            .add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.8, 0.5),
                emissive: LinearRgba::rgb(4.0, 2.4, 1.0),
                // A flame draws its own light: unlit, so the field does not
                // take it over and darken it with the night (`field_light`,
                // `take_the_field`).
                unlit: true,
                ..default()
            });
        let flame = self.pivot(parent, Moving::Flame, Transform::from_translation(at));
        self.cuboid(
            flame,
            Vec3::new(0.205 * s, 0.305 * s, 0.205 * s),
            Vec3::ZERO,
            &warm,
        );
    }

    /// A pivot holding a unit cube: posed each frame as a bone (`Arm`).
    fn bone(&mut self, parent: Entity, kind: Moving, paint: &Handle<StandardMaterial>) {
        let pivot = self.pivot(parent, kind, Transform::IDENTITY);
        self.cuboid(pivot, Vec3::ONE, Vec3::ZERO, paint);
    }

    /// M34's rowing frame (`river-craft` design 9.10, `deckBoxes` and
    /// `raftKit` in `rowing.js`), in the game's frame: a beam across the
    /// planks at the pins, a post each side with a swivel oarlock, the
    /// bench the rower sits on, the two oars, the rower (third person) and
    /// the first-person hands.
    fn rowing_frame(
        &mut self,
        parent: Entity,
        spec: &RowingSpec,
        planks: &Handle<StandardMaterial>,
        dark: &Handle<StandardMaterial>,
    ) {
        let a = spec.oars;
        let pin = Vec3::from(a.pin);
        let seat = Vec3::from(spec.rower.seat);
        let top = pin.y - 0.08;
        self.cuboid(
            parent,
            Vec3::new(2.6, 0.12, 0.16),
            Vec3::new(0.0, 0.36, pin.z),
            dark,
        );
        self.cuboid(
            parent,
            Vec3::new(0.9, 0.3, 0.36),
            Vec3::new(0.0, 0.43, seat.z),
            planks,
        );
        let iron = paint(self.world, Color::srgb_u8(0x9a, 0x9a, 0x9a), false);
        let rb = spec.blade_reach() as f32;
        let loom = a.length_m - 0.55 - a.inboard_m;
        for side in [-1_i8, 1] {
            let s = f32::from(side);
            let at = Vec3::new(s * pin.x, pin.y, pin.z);
            self.cuboid(
                parent,
                Vec3::new(0.14, top - 0.42, 0.14),
                Vec3::new(at.x, (0.42 + top) / 2.0, at.z),
                dark,
            );
            // The oarlock: two iron horns either side of the loom on a
            // swivel, so the loom rides in the lock above the post.
            let lock = self.pivot(parent, Moving::Lock(side), Transform::from_translation(at));
            self.cuboid(lock, Vec3::new(0.04, 0.05, 0.04), Vec3::Y * -0.06, &iron);
            for z in [0.048_f32, -0.048] {
                self.cuboid(
                    lock,
                    Vec3::new(0.022, 0.12, 0.03),
                    Vec3::new(0.0, 0.0, z),
                    &iron,
                );
            }
            self.cuboid(lock, Vec3::new(0.118, 0.02, 0.03), Vec3::Y * -0.04, &iron);
            // The oar along its pivot's +x: loom, handle, grip and blade.
            let oar = self.pivot(parent, Moving::Oar(side), Transform::from_translation(at));
            self.cuboid(
                oar,
                Vec3::new(loom, 0.06, 0.06),
                Vec3::X * loom / 2.0,
                planks,
            );
            self.cuboid(
                oar,
                Vec3::new(a.inboard_m, 0.034, 0.034),
                Vec3::X * -a.inboard_m / 2.0,
                planks,
            );
            self.cuboid(
                oar,
                Vec3::new(0.02, 0.044, 0.044),
                Vec3::X * (0.01 - a.inboard_m),
                dark,
            );
            let blade = self.pivot(
                oar,
                Moving::Blade(side),
                Transform::from_translation(Vec3::X * rb),
            );
            self.cuboid(blade, Vec3::new(0.62, 0.2, 0.035), Vec3::ZERO, planks);
        }
        // The rower, facing the bow (the push stroke): a cap's peak and a
        // nose ahead, hair at the back, feet braced on the beam.
        let cloth = paint(self.world, Color::srgb_u8(0x8a, 0x5a, 0x3a), false);
        let skin = paint(self.world, Color::srgb_u8(0xd9, 0xa7, 0x7c), false);
        let trousers = paint(self.world, Color::srgb_u8(0x4a, 0x53, 0x68), false);
        let hat = paint(self.world, Color::srgb_u8(0x6b, 0x56, 0x38), false);
        let hair = paint(self.world, Color::srgb_u8(0x5a, 0x40, 0x30), false);
        let boots = paint(self.world, Color::srgb_u8(0x3a, 0x30, 0x26), false);
        let rower = self.pivot(parent, Moving::Rower, Transform::IDENTITY);
        for side in [-1.0_f32, 1.0] {
            let hip = Vec3::new(side * 0.11, seat.y + 0.06, seat.z - 0.04);
            let knee = Vec3::new(side * 0.12, seat.y + 0.05, seat.z - 0.33);
            let foot = Vec3::new(side * 0.12, 0.33, pin.z + 0.19);
            self.limb(rower, hip, knee, Vec2::splat(0.15), &trousers);
            self.limb(rower, knee, foot, Vec2::splat(0.13), &trousers);
            self.cuboid(rower, Vec3::new(0.12, 0.08, 0.2), foot, &boots);
        }
        let body = self.pivot(rower, Moving::Hip, Transform::from_translation(seat));
        self.cuboid(body, Vec3::new(0.42, 0.58, 0.26), Vec3::Y * 0.32, &cloth);
        let head = Vec3::Y * 0.78;
        self.cuboid(body, Vec3::splat(0.26), head, &skin);
        self.cuboid(
            body,
            Vec3::new(0.27, 0.22, 0.05),
            head + Vec3::new(0.0, 0.02, 0.11),
            &hair,
        );
        self.cuboid(
            body,
            Vec3::new(0.05, 0.05, 0.04),
            head + Vec3::new(0.0, -0.02, -0.145),
            &skin,
        );
        self.cuboid(
            body,
            Vec3::new(0.3, 0.07, 0.3),
            head + Vec3::new(0.0, 0.17, 0.01),
            &hat,
        );
        self.cuboid(
            body,
            Vec3::new(0.26, 0.025, 0.14),
            head + Vec3::new(0.0, 0.14, -0.2),
            &hat,
        );
        for side in [-1_i8, 1] {
            self.bone(rower, Moving::Arm(side, 0), &cloth);
            self.bone(rower, Moving::Arm(side, 1), &cloth);
            self.bone(rower, Moving::Arm(side, 2), &skin);
        }
        // The first-person hands: M36's approved hand (the axe's grip, as
        // the mockup's), lit as the raft is.
        let kit = super::oar_hand::Kit::new(self.world);
        for side in [-1_i8, 1] {
            let hand = self.pivot(parent, Moving::Hand(side), Transform::IDENTITY);
            kit.spawn(self.world, hand);
        }
    }

    /// M34's anchor (`anchorModel`): an iron shank, its stock across the
    /// top and its crown and flukes at the foot, about 0.6 m long; and its
    /// rode, a hemp line.
    fn anchor_gear(&mut self, parent: Entity) {
        let iron = paint(self.world, Color::srgb_u8(0x4a, 0x4d, 0x52), false);
        let hemp = paint(self.world, Color::srgb_u8(0x9a, 0x86, 0x5c), false);
        let anchor = self.pivot(parent, Moving::Anchor, Transform::IDENTITY);
        self.cuboid(anchor, Vec3::new(0.05, 0.6, 0.05), Vec3::Y * 0.3, &iron);
        self.cuboid(anchor, Vec3::new(0.04, 0.04, 0.42), Vec3::Y * 0.56, &iron);
        self.cuboid(anchor, Vec3::new(0.4, 0.06, 0.06), Vec3::Y * 0.03, &iron);
        for side in [-1.0_f32, 1.0] {
            self.cuboid(
                anchor,
                Vec3::new(0.07, 0.16, 0.08),
                Vec3::new(side * 0.2, 0.1, 0.0),
                &iron,
            );
        }
        self.bone(parent, Moving::Rode, &hemp);
    }

    fn crew(&mut self, parent: Entity, eye: [f32; 3], paint: &Handle<StandardMaterial>) {
        let eye = Vec3::from(eye);
        let figure = self.pivot(parent, Moving::Crew, Transform::from_translation(eye));
        self.piece(
            figure,
            Capsule3d::new(0.2, 0.7).into(),
            paint,
            Transform::from_xyz(0.0, -0.55, 0.0),
        );
    }
}

/// Hang a craft's meshes under its entity.
pub fn build(world: &mut World, entity: Entity, craft: &Craft) {
    // Lit by the field wherever it stands (`lamps-and-lanterns` decision 10),
    // and as the terrain is: its own sun and shadows, not Bevy's
    // (`craft-lighting`; the owner: "can you properly light the raft?").
    world.entity_mut(entity).insert((
        crate::field_light::LitByField,
        crate::field_light::LitLikeTerrain,
    ));
    if super::model::AUTHORED.contains(&craft.kind) {
        super::model::build(world, entity, craft)
            .expect("vehicle model: regenerate and export from Blender");
        return;
    }
    let dark = paint(world, Color::srgb(0.18, 0.2, 0.22), false);
    let trim = paint(world, Color::srgb(0.75, 0.32, 0.18), false);
    let crew = paint(world, Color::srgb(0.9, 0.55, 0.15), false);
    let mut b = Builder {
        world,
        owner: entity,
    };
    match craft.kind {
        Kind::Loon => {
            // The towns mockup's canoe (`cities-in-the-world` task 4.2b), in
            // driftwood, at the craft's own hull and unpainted.
            use pbd_core::settlement::pieces::dressing as boats;
            let s = *craft.loon_spec();
            let h = s.hull;
            let repeat = crate::towns::repeat_in(b.world);
            let size = Vec3::new(h.length_m, h.beam_m, h.depth_m);
            let floor = craft.floor().expect("an open boat has floorboards");
            let cut = boats::small_boat(&repeat, "canoe", size, h.sheer_m, floor);
            crate::towns::spawn_meshes(b.world, entity, &cut);
            // Its paddle lies in it until someone takes it up.
            let laid = b.pivot(entity, Moving::Laid, Transform::IDENTITY);
            crate::towns::spawn_meshes(
                b.world,
                laid,
                &boats::laid_paddle(&repeat, size, h.sheer_m),
            );
            let paddle = b.pivot(entity, Moving::Paddle, Transform::IDENTITY);
            b.cuboid(paddle, Vec3::new(0.04, 1.3, 0.04), Vec3::Y * 0.65, &dark);
            b.piece(
                paddle,
                paddle_blade(s.paddle.blade_m2),
                &trim,
                Transform::IDENTITY,
            );
            b.crew(entity, s.seat.eye, &crew);
            b.anchor_gear(entity);
        }
        Kind::Tube => {
            // M34's inner tube (`river-craft`, the Tube tab; `small-boats`):
            // a felted-wool ring sealed with pine pitch, 0.52 m to the middle
            // of the tube and 0.22 m thick, a darker pitched seam over a
            // quarter of it, and a lantern on a short pole tied to the ring.
            // Colours are the mockup's tiles under its tints, taken down to
            // read as it does under the game's brighter noon.
            let ring = paint(b.world, Color::srgb_u8(0x52, 0x40, 0x28), false);
            let seam = paint(b.world, Color::srgb_u8(0x38, 0x2c, 0x1c), false);
            b.piece(
                entity,
                Torus::new(0.52 - 0.22, 0.52 + 0.22)
                    .mesh()
                    .minor_resolution(8)
                    .major_resolution(14)
                    .into(),
                &ring,
                Transform::from_xyz(0.0, 0.05, 0.0),
            );
            b.piece(
                entity,
                torus_arc(0.52, 0.225, std::f32::consts::PI * 0.25, 4, 4),
                &seam,
                Transform::from_xyz(0.0, 0.05, 0.0),
            );
            // No lantern (owner, 2026-11-19: "no light for tube guy"). The
            // rider sits back in the ring, legs over its front with the feet
            // in the water, and paddles with the hands beside it, sitting up
            // to reach (M34 version 4: "fix arms/body"): the hand on the
            // stroke's side goes in at the front, pulls back through the
            // water and comes back low over the rim.
            let eye = Vec3::from(craft.loon_spec().seat.eye);
            let at = b.pivot(entity, Moving::Crew, Transform::from_translation(eye));
            let shirt = paint(b.world, Color::srgb_u8(0x6b, 0x87, 0x66), false);
            let trousers = paint(b.world, Color::srgb_u8(0x73, 0x70, 0x5f), false);
            let skin = paint(b.world, Color::srgb_u8(0xbb, 0x9c, 0x6e), false);
            let hat = paint(b.world, Color::srgb_u8(0x87, 0x73, 0x4a), false);
            let hair = paint(b.world, Color::srgb_u8(0x5a, 0x40, 0x30), false);
            // Joints in the tube's frame (+z aft, the ring's back), then
            // under the eye's pivot.
            let lp = |p: Vec3| p - eye;
            let body = b.pivot(
                at,
                Moving::TubeBody,
                Transform::from_translation(lp(TUBE_HIPS)),
            );
            let from_hips = |p: Vec3| p - TUBE_HIPS;
            b.limb(
                body,
                Vec3::ZERO,
                from_hips(Vec3::new(0.0, 0.3, 0.46)),
                Vec2::new(0.44, 0.26),
                &shirt,
            );
            let head = from_hips(Vec3::new(0.0, 0.48, 0.6));
            b.cuboid(body, Vec3::splat(0.28), head, &skin);
            b.cuboid(
                body,
                Vec3::new(0.29, 0.22, 0.05),
                head + Vec3::new(0.0, 0.02, 0.12),
                &hair,
            );
            b.cuboid(
                body,
                Vec3::new(0.05, 0.05, 0.04),
                head + Vec3::new(0.0, -0.02, -0.155),
                &skin,
            );
            b.cuboid(
                body,
                Vec3::new(0.36, 0.08, 0.36),
                head + Vec3::new(0.0, 0.16, 0.04),
                &hat,
            );
            for side in [-1.0_f32, 1.0] {
                let x = side * 0.12;
                let knee = Vec3::new(x, 0.32, -0.5);
                let foot = Vec3::new(x, -0.02, -0.74);
                b.limb(
                    at,
                    lp(TUBE_HIPS + Vec3::X * x),
                    lp(knee),
                    Vec2::splat(0.16),
                    &trousers,
                );
                b.limb(at, lp(knee), lp(foot), Vec2::splat(0.14), &trousers);
                let s = side as i8;
                b.bone(at, Moving::TubeArm(s, 0), &shirt);
                b.bone(at, Moving::TubeArm(s, 1), &shirt);
                b.bone(at, Moving::TubeArm(s, 2), &skin);
            }
        }
        Kind::Raft => {
            // M34's raft (`river-craft`, the Raft tab; `small-boats`): five
            // logs 4.4 m long and 0.25 m round (six-sided, bark round and
            // rings on the ends) side by side, three planks across them, a
            // crate lashed aft of the bow, a lantern on a post at the bow's
            // corner, and the rowing frame with its two oars and its rower
            // (version 4 of M34's Rowing tab; no pole).
            let bark = paint(b.world, Color::srgb_u8(0x4a, 0x33, 0x1f), false);
            let rings = paint(b.world, Color::srgb_u8(0x9a, 0x7a, 0x50), false);
            let planks = paint(b.world, Color::srgb_u8(0x74, 0x55, 0x32), false);
            let dark_planks = paint(b.world, Color::srgb_u8(0x6e, 0x4f, 0x30), false);
            let crate_wood = paint(b.world, Color::srgb_u8(0x6d, 0x51, 0x30), false);
            const LOG_M: f32 = 4.4;
            const LOG_R: f32 = 0.25;
            for i in 0..5 {
                let x = -0.96 + i as f32 * 0.48;
                let along = Quat::from_rotation_x(FRAC_PI_2);
                b.piece(
                    entity,
                    Cylinder::new(LOG_R, LOG_M).mesh().resolution(6).into(),
                    &bark,
                    Transform::from_xyz(x, -0.04, 0.0).with_rotation(along),
                );
                for end in [-1.0_f32, 1.0] {
                    b.piece(
                        entity,
                        RegularPolygon::new(LOG_R * 0.98, 6).into(),
                        &rings,
                        Transform::from_xyz(x, -0.04, end * (LOG_M / 2.0 + 0.002)).with_rotation(
                            Quat::from_rotation_y(if end > 0.0 {
                                0.0
                            } else {
                                std::f32::consts::PI
                            }) * Quat::from_rotation_z(FRAC_PI_2),
                        ),
                    );
                }
            }
            for z in [-1.5_f32, 0.0, 1.5] {
                b.cuboid(
                    entity,
                    Vec3::new(2.5, 0.08, 0.3),
                    Vec3::new(0.0, 0.24, z),
                    &planks,
                );
            }
            b.cuboid(
                entity,
                Vec3::new(0.6, 0.5, 0.7),
                Vec3::new(0.5, 0.53, -1.2),
                &crate_wood,
            );
            b.cuboid(
                entity,
                Vec3::new(0.1, 1.1, 0.1),
                Vec3::new(-0.95, 0.8, -1.95),
                &dark_planks,
            );
            b.lantern(entity, Vec3::new(-0.95, 1.45, -1.95), 0.9);
            if let Some(rowing) = craft.loon_spec().rowing {
                b.rowing_frame(entity, &rowing, &planks, &dark_planks);
            }
            b.anchor_gear(entity);
        }
    }
}

/// Put every craft where the physics says it is and move its parts.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn place(
    frame: Res<crate::planet::PlanetRenderFrame>,
    view: Res<VehicleView>,
    aboard: Res<super::Aboard>,
    dusk: Option<Res<crate::planet::lod::DuskLamps>>,
    tools: Option<Res<crate::fish::ToolSlot>>,
    mut crafts: Query<(Entity, &Vehicle, &mut Transform), Without<Part>>,
    mut parts: Query<(&mut Part, &mut Transform, &mut Visibility)>,
) {
    let lamps_lit = dusk.is_some_and(|d| d.lit);
    for (_, vehicle, mut transform) in &mut crafts {
        let craft = &vehicle.craft;
        transform.translation = (frame.center + craft.reference_position()).as_vec3();
        transform.rotation = craft.body.orientation.as_quat();
    }
    for (part, mut transform, mut visibility) in &mut parts {
        let Ok((entity, vehicle, _)) = crafts.get(part.owner) else {
            continue;
        };
        let craft = &vehicle.craft;
        match (part.kind, &craft.state) {
            (Moving::Flame, _) => {
                *visibility = visible(lamps_lit && !craft.lantern_out);
            }
            (Moving::Laid, _) => {
                *visibility = visible(!craft.occupied);
            }
            (Moving::Paddle, CraftState::Loon(s)) => {
                *transform = paddle(craft, s);
                *visibility = visible(craft.occupied);
            }
            (Moving::TubeBody | Moving::TubeArm(..), CraftState::Loon(s)) => {
                *transform = tube_pose(craft, s, part.kind);
            }
            (Moving::Crew, _) => {
                let chase = !(view.seat && aboard.0 == Some(entity));
                *visibility = visible(craft.occupied && chase);
                transform.translation = Vec3::from(craft.seat().eye);
            }
            (Moving::Rower, _) => {
                // Each part in exactly one view (the owner: "why is fps arm
                // in third person"): the body from the chase view only.
                let chase = !(view.seat && aboard.0 == Some(entity));
                *visibility = visible(craft.occupied && chase);
            }
            (Moving::Hand(side), _) => {
                // The hands on the handles in the seat view, unless a tool
                // is in hand while the oars rest (the rod, `small-boats`
                // design 6).
                let seat = view.seat && aboard.0 == Some(entity);
                let rowing = craft.rowing().is_some_and(|(_, r)| r.running);
                let free = tools.as_deref().is_none_or(|t| t.0.held().is_none());
                let shown = seat && (rowing || free);
                *visibility = visible(shown);
                if shown && let Some(pose) = rowing_pose(craft, Moving::Hand(side)) {
                    *transform = pose;
                }
            }
            (Moving::Anchor, _) => {
                let (pose, shown) = anchor_pose(craft);
                *transform = pose;
                *visibility = visible(shown);
            }
            (Moving::Rode, _) => {
                let rode = rode_pose(craft);
                *visibility = visible(rode.is_some());
                if let Some(pose) = rode {
                    *transform = pose;
                }
            }
            (kind, _) => {
                if let Some(pose) = rowing_pose(craft, kind) {
                    *transform = pose;
                }
            }
        }
    }
}

fn visible(shown: bool) -> Visibility {
    if shown {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    }
}

/// The paddle's blade: in the water through the power phase, sweeping along
/// the stroke, and lifted forward again through the recovery.
fn paddle(craft: &Craft, state: &pbd_core::vehicle::LoonState) -> Transform {
    let p = craft.loon_spec().paddle;
    if let pbd_core::vehicle::Telemetry::Loon(t) = &craft.telemetry
        && let Some(at) = t.rudder_at
    {
        return Transform::from_translation(at.as_vec3());
    }
    let Some(stroke) = state.stroke else {
        // The raft's pole stands in its poler's hand, as M34's: the hand at
        // 1.45 m, a third of the way down its 4.6 m (the pole is drawn 0.35 m
        // over its pivot).
        if craft.kind == Kind::Raft {
            return Transform::from_xyz(0.62, 1.45 - 1.4 - 0.35, 0.55)
                .with_rotation(Quat::from_rotation_x(-0.12));
        }
        // Resting across the gunwales.
        return Transform::from_xyz(0.0, 0.35, 0.5).with_rotation(Quat::from_rotation_z(FRAC_PI_2));
    };
    let u = stroke.phase as f32;
    let travel = |u: f32| p.stroke_m * (1.0 - (std::f32::consts::PI * u).cos()) / 2.0;
    let (along, lift) = if u < 1.0 {
        (travel(u), 0.0)
    } else {
        (p.stroke_m - travel(u - 1.0), RECOVERY_LIFT_M)
    };
    let z = if stroke.direction > 0.0 {
        p.catch_z + along
    } else {
        p.catch_z + p.stroke_m - along
    };
    let x = stroke.side as f32 * p.reach_m;
    Transform::from_xyz(x, p.depth_m + lift, z)
        .with_rotation(Quat::from_rotation_z(-x.signum() * 0.35))
}

/// The tube rider's hips, the tube's frame.
const TUBE_HIPS: Vec3 = Vec3::new(0.0, -0.12, 0.12);
/// How far the rider sits up to paddle, rad (M34: 40 degrees back at rest,
/// 20 paddling).
const TUBE_SIT_UP: f32 = 0.35;

/// The tube rider's hand on a side and how far it is into paddling (0 at
/// rest on the rim, 1 on its arc), from the stroke: the Loon's stroke the
/// tube is paddled on (`loon.rs`), its power phase the hand in the water.
fn tube_hand(state: &pbd_core::vehicle::LoonState, side: f32) -> (Vec3, f32, f32) {
    use std::f32::consts::PI;
    let rest = Vec3::new(side * 0.68, 0.31, -0.16);
    let Some(stroke) = state.stroke.filter(|k| k.side.signum() as f32 == side) else {
        return (rest, 0.0, 0.0);
    };
    let u = stroke.phase as f32;
    // In at the front, back through the water, out behind and low over the
    // rim to the front again; backing goes the other way round.
    let ph = if stroke.direction > 0.0 {
        2.0 * PI + PI * u
    } else {
        3.0 * PI - PI * u
    };
    let wet = ph.sin().max(0.0);
    let arc = Vec3::new(
        side * (0.6 + 0.16 * wet),
        0.08 - 0.16 * ph.sin(),
        0.08 - 0.2 * ph.cos(),
    );
    let ease = |t: f32| {
        let t = t.clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    };
    let engage = ease(u / 0.15) * ease((2.0 - u) / 0.15);
    (rest.lerp(arc, engage), engage, wet)
}

/// The tube rider posed from the stroke, under the eye's pivot: the body
/// sitting up as a hand paddles, each arm from its shoulder through an
/// elbow solved out and down (up while the hand is in the water) to the
/// hand.
fn tube_pose(craft: &Craft, state: &pbd_core::vehicle::LoonState, kind: Moving) -> Transform {
    let eye = Vec3::from(craft.loon_spec().seat.eye);
    let hands = [tube_hand(state, -1.0), tube_hand(state, 1.0)];
    let sit = TUBE_SIT_UP * hands[0].1.max(hands[1].1);
    let turn = Quat::from_rotation_x(-sit);
    match kind {
        Moving::TubeBody => Transform::from_translation(TUBE_HIPS - eye).with_rotation(turn),
        Moving::TubeArm(side, bone) => {
            let s = f32::from(side);
            let (hand, _, wet) = hands[usize::from(side > 0)];
            let shoulder = TUBE_HIPS + turn * (Vec3::new(s * 0.24, 0.26, 0.42) - TUBE_HIPS);
            let pole = pbd_core::DVec3::new(f64::from(s), f64::from(-1.0 + 1.8 * wet), 0.2);
            let e = elbow(shoulder.as_dvec3(), hand.as_dvec3(), 0.38, 0.38, pole).as_vec3();
            match bone {
                0 => bone_between(shoulder - eye, e - eye, 0.12),
                1 => bone_between(e - eye, hand - eye, 0.11),
                _ => {
                    Transform::from_translation(hand - eye).with_scale(Vec3::new(0.12, 0.09, 0.13))
                }
            }
        }
        _ => Transform::IDENTITY,
    }
}

/// Where the anchor lies, in the craft's frame, planet-frame point `at`.
fn local_of(craft: &Craft, at: pbd_core::DVec3) -> Vec3 {
    (craft.body.orientation.inverse() * (at - craft.reference_position())).as_vec3()
}

/// Where the anchor is out of the boat, planet frame: its work, or a
/// mooring read from a save or made by a harbour.
fn anchor_at(craft: &Craft) -> Option<pbd_core::DVec3> {
    craft
        .anchor
        .map(|a| a.at)
        .or_else(|| craft.mooring.filter(|m| m.anchored).map(|m| m.at))
}

/// Where a raft keeps its anchor stowed: on the deck by the bow, laid down.
const STOWED: Vec3 = Vec3::new(0.25, 0.3, -1.85);

/// The anchor's place and whether it is drawn: out of the boat, upright on
/// its crown, the stock across; stowed, laid on a raft's deck (a canoe
/// keeps it out of sight).
fn anchor_pose(craft: &Craft) -> (Transform, bool) {
    if let Some(at) = anchor_at(craft) {
        let local = local_of(craft, at);
        let up = local_of(craft, at + at.normalize()) - local;
        return (
            Transform::from_translation(local)
                .with_rotation(Quat::from_rotation_arc(Vec3::Y, up.normalize_or(Vec3::Y))),
            true,
        );
    }
    let laid = Transform::from_translation(STOWED).with_rotation(Quat::from_rotation_x(-FRAC_PI_2));
    (laid, craft.kind == Kind::Raft)
}

/// The rode from the bow to the anchor's stock while it is out.
fn rode_pose(craft: &Craft) -> Option<Transform> {
    let at = anchor_at(craft)?;
    let top = local_of(craft, at + at.normalize() * 0.56);
    Some(bone_between(Vec3::from(craft.loon_spec().bow), top, 0.03))
}

/// A part of the rowing frame posed from the oars' stroke (`river-craft`
/// design 9): the oars, their locks and blades from each oar's place, the
/// rower's lean and his arms from the shoulders to the grips, the
/// first-person hands closed on the handles. Read off the state the force
/// was worked from; nothing here decides anything.
fn rowing_pose(craft: &Craft, kind: Moving) -> Option<Transform> {
    let (spec, state) = craft.rowing()?;
    let oar = |side: i8| &state.oars[usize::from(side > 0)].geom;
    let lean = state.lean(spec);
    let arm_of = |side: i8| {
        let s = f64::from(side);
        let g = oar(side);
        let shoulder = spec.shoulder(lean, s);
        let r = &spec.rower;
        let e = elbow(
            shoulder,
            g.grip,
            f64::from(r.upper_arm_m),
            f64::from(r.forearm_m),
            elbow_pole(s),
        );
        (shoulder.as_vec3(), e.as_vec3(), g.grip.as_vec3())
    };
    Some(match kind {
        Moving::Oar(side) => {
            let g = oar(side);
            Transform::from_translation(g.pin.as_vec3()).with_rotation(along(g.dir.as_vec3()))
        }
        Moving::Lock(side) => {
            let g = oar(side);
            Transform::from_translation(g.pin.as_vec3())
                .with_rotation(Quat::from_rotation_y(yaw(g.dir.as_vec3())))
        }
        Moving::Blade(side) => Transform::from_translation(Vec3::X * spec.blade_reach() as f32)
            .with_rotation(Quat::from_rotation_x(
                f32::from(side) * oar(side).feather as f32,
            )),
        Moving::Hip => Transform::from_translation(Vec3::from(spec.rower.seat))
            .with_rotation(Quat::from_rotation_x(-lean as f32)),
        Moving::Arm(side, bone) => {
            let (shoulder, e, grip) = arm_of(side);
            match bone {
                0 => bone_between(shoulder, e, 0.11),
                1 => bone_between(e, grip, 0.1),
                _ => {
                    let dir = oar(side).dir.as_vec3() * f32::from(side);
                    Transform::from_translation(grip)
                        .with_rotation(Quat::from_rotation_arc(Vec3::Z, dir.normalize()))
                        .with_scale(Vec3::new(0.09, 0.09, 0.1))
                }
            }
        }
        Moving::Hand(side) => {
            let (_, e, grip) = arm_of(side);
            super::oar_hand::place(
                f32::from(side),
                grip,
                oar(side).dir.as_vec3(),
                e,
                spec.rower.hand_scale_m,
            )
        }
        _ => return None,
    })
}

/// The heading of a direction in the craft's plane, about +y from +x.
fn yaw(dir: Vec3) -> f32 {
    (-dir.z).atan2(dir.x)
}

/// A turn that lays +x along `dir`, by heading then pitch, so the blade's
/// square face stays upright.
fn along(dir: Vec3) -> Quat {
    let pitch = dir.y.atan2(Vec2::new(dir.x, dir.z).length());
    Quat::from_rotation_y(yaw(dir)) * Quat::from_rotation_z(pitch)
}

/// A unit cube posed as a bone from `a` to `b`, `width` thick.
fn bone_between(a: Vec3, b: Vec3, width: f32) -> Transform {
    let d = b - a;
    let length = d.length().max(1e-4);
    Transform::from_translation((a + b) / 2.0)
        .with_rotation(Quat::from_rotation_arc(Vec3::Y, d / length))
        .with_scale(Vec3::new(width, length, width))
}

#[cfg(test)]
mod tests {
    use super::*;
    use pbd_core::vehicle::{Hulls, LoonTelemetry, Telemetry, spec::VehicleSpecs};
    use std::sync::Arc;

    fn dimensions(mesh: &Mesh) -> Vec3 {
        let bevy::mesh::VertexAttributeValues::Float32x3(points) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap()
        else {
            panic!("positions")
        };
        let lo = points
            .iter()
            .map(|p| Vec3::from(*p))
            .fold(Vec3::splat(f32::INFINITY), Vec3::min);
        let hi = points
            .iter()
            .map(|p| Vec3::from(*p))
            .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max);
        hi - lo
    }

    #[test]
    fn paddle_mesh_and_motion_follow_area_and_working_rudder_without_force() {
        let specs = Arc::new(VehicleSpecs::default());
        let mut craft = Craft::new(
            Kind::Loon,
            1,
            specs.clone(),
            Hulls::new(&specs),
            pbd_core::DVec3::Y * 4800.0,
            pbd_core::DQuat::IDENTITY,
        );
        let size = dimensions(&paddle_blade(specs.loon.paddle.blade_m2));
        assert!((size.y * size.z - specs.loon.paddle.blade_m2).abs() < 1e-6);
        for side in [-1.0, 1.0] {
            let at = pbd_core::DVec3::new(
                side * specs.loon.paddle.rudder_at[0] as f64,
                specs.loon.paddle.depth_m as f64,
                specs.loon.paddle.rudder_at[1] as f64,
            );
            craft.telemetry = Telemetry::Loon(LoonTelemetry {
                rudder_at: Some(at),
                ..default()
            });
            let CraftState::Loon(state) = &mut craft.state;
            // Recovery must not hide the working rudder either.
            state.stroke = Some(pbd_core::vehicle::Stroke {
                side,
                direction: 1.0,
                phase: 1.5,
            });
            let CraftState::Loon(state) = &craft.state;
            assert_eq!(paddle(&craft, state).translation, at.as_vec3());
            assert_eq!(craft.body.velocity, pbd_core::DVec3::ZERO);
        }
    }
}
