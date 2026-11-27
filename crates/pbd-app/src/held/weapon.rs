//! A weapon in hand (`survival-mode` design 16, task 4.10): the spears, the
//! bow, an arrow, the hunting axes and the swords, drawn in the approved
//! hand when one is in the selected hotbar slot, as the approved weapons
//! mockup builds them (`crate::weapon_models`).
//!
//! It is held with the held items' pose: the pickaxe's grip turned to the
//! hold in `held.ron`'s `weapon` (the torch's hold, a little lower and
//! further out), the fist closed on the weapon where it is gripped (a
//! spear near its balance, a sword at its grip). It is in the right hand
//! when the tool slot is empty, and mirrored into the left beside a tool.
//! Its swings are not here: they wait on their mockup (task 1.6), so the
//! weapon rests in the hand with the breath of the held items' sway.
//!
//! Each part is unlit with a light fixed in its own frame baked in, and the
//! field's light at the eye laid over it (`light_held`), as the tools and
//! the torch are: in a cave or at night it is as dark as the ground.
//! Drawn only: no rule reads it.

use super::light::{hold_rest, item_frame};
use super::{
    HandShading, Held, HeldConfig, HeldMaterials, Texture, hand_colours, hand_image, hand_mesh,
    models, seated_afloat,
};
use crate::carried_light::HeldItem;
use crate::fish::ToolSlot;
use crate::walking::{WalkingCamera, WalkingReadout};
use bevy::light::NotShadowCaster;
use bevy::prelude::*;
use pbd_core::goods::Good;
use pbd_core::inventory::{Item, Tool};
use serde::{Deserialize, Serialize};

/// How a weapon is held (`held.ron`'s `weapon`): where the fist closes, eye
/// space m, which way the weapon runs up from it, and the breath at rest, m.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct WeaponHold {
    pub hold: super::light::Hold,
    /// A spear's way up from the same grip: leant forward, its head ahead
    /// of the eye, as a spear is carried at the ready.
    pub spear_up: [f32; 3],
    pub breathe_m: f32,
}

impl Default for WeaponHold {
    fn default() -> Self {
        Self {
            hold: super::light::Hold {
                grip: [0.26, -0.3, -0.56],
                up: [-0.18, 1.0, -0.42],
            },
            spear_up: [-0.22, 0.62, -1.0],
            breathe_m: 0.003,
        }
    }
}

impl WeaponHold {
    pub fn validate(&self) -> Result<(), String> {
        if Vec3::from(self.hold.up).length() < 1e-3
            || Vec3::from(self.spear_up).length() < 1e-3
            || !Vec3::from(self.hold.grip).is_finite()
            || !(0.0..0.05).contains(&self.breathe_m)
        {
            return Err("held.weapon: a grip, a way up and a breath of 0..0.05 m".into());
        }
        Ok(())
    }
}

/// A weapon in hand: its good, its rest pose and its fist.
#[derive(Component)]
pub struct HeldWeaponView {
    good: Good,
    rest: Transform,
    fist: Vec3,
}

/// The weapon a slot's item is drawn as, if it is one.
pub fn weapon_of(item: Item) -> Option<Good> {
    match item {
        Item::Good(good) if crate::weapon_models::WEAPONS.contains(&good) => Some(good),
        _ => None,
    }
}

/// Build every weapon and its hand once the tools are built, hidden.
#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_weapons(
    mut commands: Commands,
    settings: Res<HeldConfig>,
    assets: Option<Res<AssetServer>>,
    cameras: Query<Entity, With<WalkingCamera>>,
    existing: Query<(), With<HeldWeaponView>>,
    held_materials: Option<ResMut<HeldMaterials>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let (Some(mut held_materials), Some(camera)) = (held_materials, cameras.iter().next()) else {
        return;
    };
    if !existing.is_empty() {
        return;
    }
    let s = &settings.0;
    let unlit = |texture: Option<Handle<Image>>, two_sided: bool| StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: texture,
        unlit: true,
        fog_enabled: false,
        // Mirrored into the left hand, its faces turn: drawn from both
        // sides, as the held lights are.
        cull_mode: None,
        double_sided: two_sided,
        ..default()
    };
    let mut lit = Vec::new();
    let hand_materials: Vec<Handle<StandardMaterial>> = Texture::ALL
        .iter()
        .map(|&t| {
            let m = materials.add(unlit(Some(images.add(hand_image(t))), false));
            lit.push(m.clone());
            m
        })
        .collect();
    let mut by_texture: std::collections::BTreeMap<&'static str, Handle<StandardMaterial>> =
        Default::default();
    let all = models();
    let pick = all.get(Tool::Pickaxe);
    let built = super::meshes(pick, all.spacing, &all.hand, s.stroke.sleeve_extra);
    let fist_tool = pick
        .rest()
        .compute_affine()
        .inverse()
        .transform_point3(pick.fist.into());
    for good in crate::weapon_models::WEAPONS {
        let spear = [Good::STONE_SPEAR, Good::BRONZE_SPEAR, Good::IRON_SPEAR].contains(&good);
        let hold = if spear {
            super::light::Hold {
                up: s.weapon.spear_up,
                ..s.weapon.hold
            }
        } else {
            s.weapon.hold
        };
        let (rest, fist) = hold_rest(pick, hold);
        let Some((parts, grip)) = crate::weapon_models::model(good) else {
            continue;
        };
        let arm = commands
            .spawn((Name::new("arm"), Transform::IDENTITY, Visibility::Inherited))
            .id();
        let holder = commands
            .spawn((
                Name::new(format!("{} in hand", good.code())),
                Held {
                    tool: Tool::Pickaxe,
                    rest,
                    fist,
                    wrist: built.wrist,
                    arm,
                    dirt: None,
                    bare: None,
                },
                HeldWeaponView { good, rest, fist },
                rest,
                Visibility::Hidden,
            ))
            .id();
        // The item's frame on the fist; the weapon moved down so its grip
        // is in the fist.
        let frame = item_frame(fist_tool, pick.scale, None);
        let item = commands
            .spawn((Name::new("item"), frame, Visibility::Inherited))
            .id();
        for part in parts {
            let part = part.clone();
            let material = by_texture
                .entry(part.texture)
                .or_insert_with(|| {
                    let texture = assets
                        .as_deref()
                        .map(|a| crate::towns::texture(a, part.texture));
                    let m = materials.add(unlit(texture, true));
                    lit.push(m.clone());
                    m
                })
                .clone();
            let mesh = meshes.add(part.held_mesh());
            let child = commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(material),
                    // Turned so its flat (a blade's, a head's) faces the
                    // eye; the bow keeps its curve across the view.
                    Transform::from_rotation(Quat::from_rotation_y(if good == Good::BOW {
                        0.0
                    } else {
                        std::f32::consts::FRAC_PI_2
                    }))
                    .with_translation(Vec3::NEG_Y * grip),
                    Visibility::Inherited,
                    NotShadowCaster,
                ))
                .id();
            commands.entity(item).add_child(child);
        }
        commands.entity(holder).add_children(&[item, arm]);
        let light = s.hand_light;
        for (on_arm, set) in [(false, &built.hand), (true, &built.arm)] {
            for texture in Texture::ALL {
                let part = &set[texture.index()];
                if part.positions.is_empty() {
                    continue;
                }
                let normals: Vec<Vec3> = part.normals.iter().map(|n| Vec3::from(*n)).collect();
                let tint: Vec<[f32; 3]> = part.flush.iter().map(|f| all.hand.tint(*f)).collect();
                let colours = hand_colours(&normals, &part.shade, &tint, rest.rotation, &light);
                let child = commands
                    .spawn((
                        Name::new(if on_arm { "arm part" } else { "hand part" }),
                        Mesh3d(meshes.add(hand_mesh(part, colours))),
                        MeshMaterial3d(hand_materials[texture.index()].clone()),
                        Transform::IDENTITY,
                        Visibility::Inherited,
                        NotShadowCaster,
                        HandShading {
                            holder,
                            on_arm,
                            normals,
                            shade: part.shade.clone(),
                            tint,
                            lit_at: None,
                        },
                    ))
                    .id();
                commands
                    .entity(if on_arm { arm } else { holder })
                    .add_child(child);
            }
        }
        commands.entity(camera).add_child(holder);
    }
    held_materials.0.extend(lit);
}

/// Show the weapon in the selected slot, at rest in the hand with a breath.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(super) fn hold_weapons(
    settings: Res<HeldConfig>,
    item: Res<HeldItem>,
    tools: Res<ToolSlot>,
    walking: Option<Res<WalkingReadout>>,
    aboard: Option<Res<crate::vehicles::Aboard>>,
    view: Option<Res<crate::vehicles::view::VehicleView>>,
    vehicles: Query<&crate::vehicles::Vehicle>,
    time: Res<Time>,
    mut holders: Query<(&HeldWeaponView, &mut Transform, &mut Visibility)>,
) {
    let on_foot = walking.is_some_and(|w| w.active)
        || seated_afloat(aboard.as_deref(), view.as_deref(), &vehicles);
    let good = item.0.and_then(weapon_of).filter(|_| on_foot);
    let left = tools.held().is_some();
    let t = time.elapsed_secs();
    let breath = settings.0.weapon.breathe_m * (t * 1.7).sin();
    for (view, mut transform, mut visibility) in &mut holders {
        let shown = good == Some(view.good);
        let want = if shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if *visibility != want {
            *visibility = want;
        }
        if !shown {
            continue;
        }
        let pose = super::HeldPose {
            shift: Vec3::Y * breath,
            ..default()
        };
        let mut posed = pose.apply(view.rest, view.fist);
        if left {
            let q = posed.rotation;
            posed.translation.x = -posed.translation.x;
            posed.rotation = Quat::from_xyzw(q.x, -q.y, -q.z, q.w);
            posed.scale.x = -posed.scale.x;
        }
        if *transform != posed {
            *transform = posed;
        }
    }
}
