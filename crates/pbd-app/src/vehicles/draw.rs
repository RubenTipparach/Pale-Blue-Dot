//! What a craft looks like: procedural meshes built from the same numbers its
//! physics is (the hull's own loft, the wing's own span and chord, the rotor's
//! own hub and radius), and the parts that move with its state - the
//! Kestrel's nacelles and rotors, the Tern's boom and tiller, the Loon's
//! paddle - placed from that state every frame. Nothing here decides anything.

use super::{Vehicle, view::VehicleView};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use pbd_core::vehicle::{Craft, CraftState, Kind};
use std::f32::consts::FRAC_PI_2;

/// Visual rotor speed at full power, rad/s: slow enough to read as blades.
const ROTOR_SPIN: f32 = 18.0;
/// How far a recovering paddle blade is lifted clear of the water, m.
const RECOVERY_LIFT_M: f32 = 0.45;

/// A moving part and the craft it belongs to.
#[derive(Component)]
pub struct Part {
    owner: Entity,
    kind: Moving,
    /// A rotor's own angle, rad.
    spin: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum Moving {
    /// The right (+1) or left (-1) nacelle.
    Nacelle(f32),
    Rotor,
    Boom,
    Tiller,
    Paddle,
    /// The figure aboard, drawn only from the chase view.
    Crew,
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
            .spawn((
                at,
                Visibility::default(),
                Part {
                    owner,
                    kind,
                    spin: 0.0,
                },
            ))
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
    let hull = paint(world, Color::srgb(0.86, 0.84, 0.78), true);
    let dark = paint(world, Color::srgb(0.18, 0.2, 0.22), false);
    let trim = paint(world, Color::srgb(0.75, 0.32, 0.18), false);
    let crew = paint(world, Color::srgb(0.9, 0.55, 0.15), false);
    let mut b = Builder {
        world,
        owner: entity,
    };
    let specs = craft.specs().clone();
    match craft.kind {
        Kind::Kestrel => {
            let s = &specs.kestrel;
            let body = paint(b.world, Color::srgb(0.78, 0.8, 0.82), false);
            let glass = paint(b.world, Color::srgb(0.12, 0.2, 0.28), false);
            b.cuboid(
                entity,
                Vec3::new(1.5, 1.5, 7.0),
                Vec3::new(0.0, -0.15, -0.6),
                &body,
            );
            b.cuboid(
                entity,
                Vec3::new(1.2, 0.7, 1.8),
                Vec3::new(0.0, 0.7, -2.4),
                &glass,
            );
            b.cuboid(
                entity,
                Vec3::new(0.6, 0.6, 3.6),
                Vec3::new(0.0, 0.4, 3.9),
                &body,
            );
            // Wing: two panels, span and chord from their area and aspect.
            let panel_span = (s.wing.panel_area_m2 * s.wing.aspect * 2.0).sqrt() / 2.0;
            let chord = s.wing.panel_area_m2 / panel_span;
            let [px, py, pz] = s.wing.panel_at;
            b.cuboid(
                entity,
                Vec3::new(
                    2.0 * (px + panel_span / 2.0).max(s.rotor.at[0]),
                    0.16,
                    chord,
                ),
                Vec3::new(0.0, py, pz),
                &body,
            );
            let foil = |area: f32, aspect: f32| {
                let span = (area * aspect).sqrt();
                (span, area / span)
            };
            let (span, chord) = foil(s.tail.area_m2, s.tail.aspect);
            b.cuboid(
                entity,
                Vec3::new(span, 0.1, chord),
                Vec3::from(s.tail.at),
                &trim,
            );
            let (height, chord) = foil(s.fin.area_m2, s.fin.aspect);
            b.cuboid(
                entity,
                Vec3::new(0.1, height, chord),
                Vec3::from(s.fin.at),
                &trim,
            );
            for gear in &s.gear {
                let at = Vec3::from(gear.at);
                b.cuboid(
                    entity,
                    Vec3::new(0.12, 0.7, 0.12),
                    at + Vec3::Y * 0.35,
                    &dark,
                );
                b.cuboid(entity, Vec3::new(0.2, 0.3, 0.5), at + Vec3::Y * 0.12, &dark);
            }
            for side in [1.0f32, -1.0] {
                let [rx, ry, rz] = s.rotor.at;
                let nacelle = b.pivot(
                    entity,
                    Moving::Nacelle(side),
                    Transform::from_xyz(side * rx, ry, rz),
                );
                b.piece(
                    nacelle,
                    Cylinder::new(0.38, 2.0).into(),
                    &body,
                    Transform::from_xyz(0.0, -0.3, 0.0),
                );
                let rotor = b.pivot(nacelle, Moving::Rotor, Transform::from_xyz(0.0, 0.8, 0.0));
                for blade in 0..3 {
                    let turn = Quat::from_rotation_y(blade as f32 * std::f32::consts::TAU / 3.0);
                    b.piece(
                        rotor,
                        Cuboid::new(0.22, 0.04, s.rotor.radius_m).into(),
                        &dark,
                        Transform::from_rotation(turn)
                            .with_translation(turn * Vec3::Z * s.rotor.radius_m / 2.0),
                    );
                }
            }
        }
        Kind::Tern => {
            let s = &specs.tern;
            let shape = craft.hull().expect("a boat has a hull").clone();
            let (p, i) = shape.loft(28, 10);
            b.piece(entity, mesh(p, i), &hull, Transform::IDENTITY);
            let (p, i) = shape.deck(28);
            b.piece(entity, mesh(p, i), &trim, Transform::IDENTITY);
            let mast = Vec3::from(s.sail.mast);
            let head = s.sail.head_height_m;
            b.cuboid(
                entity,
                Vec3::new(0.1, head, 0.1),
                mast + Vec3::Y * head / 2.0,
                &dark,
            );
            let keel_h = (s.keel.area_m2 * s.keel.aspect).sqrt();
            b.cuboid(
                entity,
                Vec3::new(0.08, keel_h, s.keel.area_m2 / keel_h),
                Vec3::from(s.keel.at),
                &dark,
            );
            // The boom and the sail swing together about the mast.
            let boom = b.pivot(
                entity,
                Moving::Boom,
                Transform::from_translation(mast + Vec3::Y * s.sail.boom_height_m),
            );
            let length = s.sail.boom_length_m;
            b.cuboid(
                boom,
                Vec3::new(0.08, 0.08, length),
                Vec3::Z * length / 2.0,
                &dark,
            );
            let sail = paint(b.world, Color::srgb(0.95, 0.94, 0.9), true);
            let rise = head - s.sail.boom_height_m;
            b.piece(
                boom,
                mesh(
                    vec![
                        Vec3::ZERO,
                        Vec3::new(0.0, 0.0, length),
                        Vec3::new(0.0, rise, 0.0),
                    ],
                    vec![0, 1, 2],
                ),
                &sail,
                Transform::IDENTITY,
            );
            let rudder_h = (s.rudder.area_m2 * s.rudder.aspect).sqrt();
            let tiller = b.pivot(
                entity,
                Moving::Tiller,
                Transform::from_translation(Vec3::from(s.rudder.at)),
            );
            b.cuboid(
                tiller,
                Vec3::new(0.05, rudder_h, s.rudder.area_m2 / rudder_h),
                Vec3::ZERO,
                &dark,
            );
            b.crew(entity, s.seat.eye, &crew);
        }
        Kind::Loon => {
            let s = &specs.loon;
            let shape = craft.hull().expect("a boat has a hull").clone();
            let (p, i) = shape.loft(28, 8);
            let canoe = paint(b.world, Color::srgb(0.2, 0.42, 0.36), true);
            b.piece(entity, mesh(p, i), &canoe, Transform::IDENTITY);
            let paddle = b.pivot(entity, Moving::Paddle, Transform::IDENTITY);
            b.cuboid(paddle, Vec3::new(0.04, 1.3, 0.04), Vec3::Y * 0.65, &dark);
            b.cuboid(paddle, Vec3::new(0.02, 0.45, 0.2), Vec3::ZERO, &trim);
            b.crew(entity, s.seat.eye, &crew);
        }
    }
}

/// Put every craft where the physics says it is and move its parts.
#[allow(clippy::type_complexity)]
pub fn place(
    time: Res<Time>,
    frame: Res<crate::planet::PlanetRenderFrame>,
    view: Res<VehicleView>,
    aboard: Res<super::Aboard>,
    mut crafts: Query<(Entity, &Vehicle, &mut Transform), Without<Part>>,
    mut parts: Query<(&mut Part, &mut Transform, &mut Visibility)>,
) {
    for (_, vehicle, mut transform) in &mut crafts {
        let craft = &vehicle.craft;
        transform.translation = (frame.center + craft.reference_position()).as_vec3();
        transform.rotation = craft.body.orientation.as_quat();
    }
    let dt = time.delta_secs();
    for (mut part, mut transform, mut visibility) in &mut parts {
        let Ok((entity, vehicle, _)) = crafts.get(part.owner) else {
            continue;
        };
        let craft = &vehicle.craft;
        match (part.kind, &craft.state) {
            (Moving::Nacelle(_), CraftState::Kestrel(s)) => {
                transform.rotation = Quat::from_rotation_x(s.nacelle as f32 - FRAC_PI_2);
            }
            (Moving::Rotor, CraftState::Kestrel(s)) => {
                part.spin =
                    (part.spin + ROTOR_SPIN * s.throttle as f32 * dt) % std::f32::consts::TAU;
                transform.rotation = Quat::from_rotation_y(part.spin);
            }
            (Moving::Boom, CraftState::Tern(s)) => {
                transform.rotation = Quat::from_rotation_y(s.boom as f32);
            }
            (Moving::Tiller, CraftState::Tern(s)) => {
                transform.rotation = Quat::from_rotation_y(s.tiller as f32);
            }
            (Moving::Paddle, CraftState::Loon(s)) => {
                *transform = paddle(craft, s);
                *visibility = visible(craft.occupied);
            }
            (Moving::Crew, state) => {
                let chase = !(view.seat && aboard.0 == Some(entity));
                *visibility = visible(craft.occupied && chase);
                let hike = match state {
                    CraftState::Tern(s) => s.crew as f32,
                    _ => 0.0,
                };
                transform.translation = Vec3::from(craft.seat().eye) + Vec3::X * hike;
            }
            _ => {}
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
    let p = craft.specs().loon.paddle;
    let Some(stroke) = state.stroke else {
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
