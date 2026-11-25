//! The rower's first-person hands on the oars (`river-craft` design 9.6):
//! M36's approved hand, the one `held` draws round a tool (`hand-model`),
//! built from the axe's grip as M34's Rowing tab does (`handKit` in
//! `docs/mockups/river-craft/rowing.js`), closed on each oar's handle and
//! turned toward the elbow the arm is solved to. The right hand is built and
//! the left is its mirror.
//!
//! Unlike the tool in hand, these hang on the raft, not the camera: they are
//! part of the craft, lit as it is (`LitByField`, `LitLikeTerrain` on its
//! root), so at night they are as dark as the deck except where a lamp
//! reaches.

use bevy::prelude::*;
use pbd_core::hand::{self, Texture};
use pbd_core::inventory::Tool;
use std::sync::OnceLock;

/// The built hand and how it sits on its handle, hand units.
struct Model {
    hand: hand::Hand,
    /// The fist's centre on the handle.
    fist: Vec3,
    /// The hand's frame: along the handle, toward the forearm, and their
    /// cross.
    basis: Mat3,
}

/// The forearm's way at rest, the game's frame: back toward the stern and
/// down, so the knuckles stay up through the stroke (`KEEP`).
const KEEP: Vec3 = Vec3::new(0.0, -0.35, 1.0);
/// How much of the forearm's way follows the solved elbow (`ELBOW_SHARE`).
const ELBOW_SHARE: f32 = 0.35;

fn model() -> &'static Model {
    static MODEL: OnceLock<Model> = OnceLock::new();
    MODEL.get_or_init(|| {
        let all = crate::held::models();
        let tool = all.get(Tool::Axe);
        let fist = tool
            .rest()
            .compute_affine()
            .inverse()
            .transform_point3(tool.fist.into());
        let built = hand::build(&tool.bones, fist, &all.hand, 0.0);
        let h = hand::handle_dir(&tool.bones);
        // The arm's way from the fist: the sleeve's points, square to the
        // handle.
        let cloth = &built.arm[Texture::Cloth.index()];
        let cloth = if cloth.positions.is_empty() {
            &built.hand[Texture::Cloth.index()]
        } else {
            cloth
        };
        let mut arm = cloth
            .positions
            .iter()
            .map(|p| Vec3::from(*p) - fist)
            .sum::<Vec3>();
        arm -= h * arm.dot(h);
        let arm = arm.normalize_or(Vec3::NEG_Y);
        Model {
            hand: built,
            fist,
            basis: Mat3::from_cols(h, arm, h.cross(arm)),
        }
    })
}

/// The hand's meshes and materials, made once a raft.
pub struct Kit {
    parts: Vec<(Handle<Mesh>, Handle<StandardMaterial>)>,
}

impl Kit {
    pub fn new(world: &mut World) -> Self {
        let m = model();
        let look = &crate::held::models().hand;
        let mut parts = Vec::new();
        for set in [&m.hand.hand, &m.hand.arm] {
            for texture in Texture::ALL {
                let part = &set[texture.index()];
                if part.positions.is_empty() {
                    continue;
                }
                // The shade baked into the hand (its creases and occlusion)
                // and the fingertips' flush; the light is the field's.
                let colours = part
                    .shade
                    .iter()
                    .zip(&part.flush)
                    .map(|(k, f)| {
                        let t = look.tint(*f);
                        [k * t[0], k * t[1], k * t[2], 1.0]
                    })
                    .collect();
                let mesh = world
                    .resource_mut::<Assets<Mesh>>()
                    .add(crate::held::hand_mesh(part, colours));
                let image = world
                    .resource_mut::<Assets<Image>>()
                    .add(crate::held::hand_image(texture));
                let material =
                    world
                        .resource_mut::<Assets<StandardMaterial>>()
                        .add(StandardMaterial {
                            base_color_texture: Some(image),
                            perceptual_roughness: 0.9,
                            double_sided: true,
                            cull_mode: None,
                            ..default()
                        });
                parts.push((mesh, material));
            }
        }
        Self { parts }
    }

    /// Hang one hand under `pivot`, which [`place`] poses.
    pub fn spawn(&self, world: &mut World, pivot: Entity) {
        let offset = world
            .spawn((
                Transform::from_translation(-model().fist),
                Visibility::Inherited,
            ))
            .id();
        world.entity_mut(pivot).add_child(offset);
        for (mesh, material) in &self.parts {
            let child = world
                .spawn((
                    Mesh3d(mesh.clone()),
                    MeshMaterial3d(material.clone()),
                    Transform::IDENTITY,
                    Visibility::Inherited,
                    bevy::light::NotShadowCaster,
                ))
                .id();
            world.entity_mut(offset).add_child(child);
        }
    }
}

/// The hand on a side closed on its handle at `grip`, the handle along
/// `handle` (outboard), its arm toward `elbow`; the craft's frame, `scale`
/// metres a hand unit (`kit.place`). The left hand is the right one on the
/// mirrored handle, mirrored back.
pub fn place(side: f32, grip: Vec3, handle: Vec3, elbow: Vec3, scale: f32) -> Transform {
    let mirror = Vec3::new(-1.0, 1.0, 1.0);
    let (g, hw, e) = if side < 0.0 {
        (grip * mirror, handle * mirror, elbow * mirror)
    } else {
        (grip, handle, elbow)
    };
    let hw = hw.normalize_or(Vec3::X);
    let mut a = e - g;
    a -= hw * a.dot(hw);
    let a = a.normalize_or(Vec3::NEG_Y);
    let keep = KEEP.normalize();
    let keep = (keep - hw * keep.dot(hw)).normalize_or(Vec3::NEG_Y);
    let a = (a * ELBOW_SHARE + keep * (1.0 - ELBOW_SHARE)).normalize_or(Vec3::NEG_Y);
    let world = Mat3::from_cols(hw, a, hw.cross(a));
    let turn = world * model().basis.transpose();
    let mut matrix =
        Mat4::from_translation(g) * Mat4::from_mat3(turn * Mat3::from_diagonal(Vec3::splat(scale)));
    if side < 0.0 {
        matrix = Mat4::from_scale(mirror) * matrix;
    }
    Transform::from_matrix(matrix)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fist lands on the grip and the handle runs through it, on both
    /// sides; the left is the right mirrored.
    #[test]
    fn the_fist_closes_on_the_grip() {
        let m = model();
        for side in [-1.0_f32, 1.0] {
            let grip = Vec3::new(side * 0.3, 0.9, -0.1);
            let handle = Vec3::new(side, -0.3, -0.2).normalize();
            let pose = place(side, grip, handle, grip + Vec3::new(0.0, -0.2, 0.3), 0.03);
            // A hand-unit point as drawn: the meshes hang `-fist` under the
            // posed pivot.
            let drawn = |p: Vec3| pose.transform_point(p - m.fist);
            let at = drawn(m.fist);
            assert!(at.distance(grip) < 1e-4, "{side}: {at} for {grip}");
            let along = drawn(m.fist + m.basis.col(0)) - at;
            assert!(along.normalize().dot(handle) > 0.999, "{side}: {along}");
        }
    }
}
