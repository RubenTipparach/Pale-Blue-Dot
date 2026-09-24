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
const BLADE_HEIGHT_M: f32 = 0.45;

fn paddle_blade(area_m2: f32) -> Mesh {
    Cuboid::new(0.02, BLADE_HEIGHT_M, area_m2 / BLADE_HEIGHT_M).into()
}

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
    Surface(usize),
    Boom,
    Tiller,
    Paddle,
    /// The figure aboard, drawn only from the chase view.
    Crew,
}

impl Part {
    pub(super) fn named(owner: Entity, craft: Kind, name: &str) -> Option<Self> {
        let kind = match name {
            "nacelle_left" => Moving::Nacelle(-1.0),
            "nacelle_right" => Moving::Nacelle(1.0),
            "rotor_left" | "rotor_right" => Moving::Rotor,
            "flaperon_left" => Moving::Surface(0),
            "flaperon_right" => Moving::Surface(1),
            "elevator" => Moving::Surface(2),
            "rudder" if craft == Kind::Kestrel => Moving::Surface(3),
            "rudder" | "tiller" => Moving::Tiller,
            "boom" => Moving::Boom,
            "paddle" => Moving::Paddle,
            "crew" => Moving::Crew,
            _ => return None,
        };
        Some(Self {
            owner,
            kind,
            spin: 0.0,
        })
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
    if super::model::AUTHORED.contains(&craft.kind) {
        super::model::build(world, entity, craft)
            .expect("vehicle model: regenerate and export from Blender");
        return;
    }
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
        Kind::Kestrel => unreachable!("authored Kestrel loaded above"),
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
            b.piece(
                paddle,
                paddle_blade(s.paddle.blade_m2),
                &trim,
                Transform::IDENTITY,
            );
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
            (Moving::Surface(index), CraftState::Kestrel(_)) => {
                if let pbd_core::vehicle::Telemetry::Kestrel(t) = &craft.telemetry {
                    transform.rotation = Quat::from_rotation_x(t.surface_deflections[index] as f32);
                }
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
    if let pbd_core::vehicle::Telemetry::Loon(t) = &craft.telemetry
        && let Some(at) = t.rudder_at
    {
        return Transform::from_translation(at.as_vec3());
    }
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
            let CraftState::Loon(state) = &mut craft.state else {
                unreachable!()
            };
            // Recovery must not hide the working rudder either.
            state.stroke = Some(pbd_core::vehicle::Stroke {
                side,
                direction: 1.0,
                phase: 1.5,
            });
            let CraftState::Loon(state) = &craft.state else {
                unreachable!()
            };
            assert_eq!(paddle(&craft, state).translation, at.as_vec3());
            assert_eq!(craft.body.velocity, pbd_core::DVec3::ZERO);
        }
    }
}
