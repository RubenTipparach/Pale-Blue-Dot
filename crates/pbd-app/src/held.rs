//! The tool in hand: each tool's own 16 px icon built into a model of hex
//! pixels (`pbd_core::hexel`), held in first person where Tenebris holds its
//! tools, swaying a little and swinging while a block is being broken.
//!
//! The icon is the only source of the shape: the model is built from the
//! same PNG the slot and the picker draw, so the two cannot differ. Where the
//! model sits is data (`assets/config/held.ron`): an eye-space anchor for the
//! grip, the point the head leans toward, and for each tool the two pixels of
//! its icon that are its grip and its head. The rod is placed on the grip and
//! tip the fishing line uses, so the line leaves the rod that is drawn
//! (`openspec/changes/fishing-and-equipment/design.md` section 12).

use crate::config::HeldConfig;
use crate::fish::ToolSlot;
use crate::walking::{WalkingCamera, WalkingReadout};
use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::light::NotShadowCaster;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;
use pbd_core::hexel;
use pbd_core::inventory::Tool;
use serde::{Deserialize, Serialize};

/// Two pixels of a tool's icon, x right and y down as the PNG stores them.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IconAxis {
    /// Where the hand closes on it.
    pub grip: [f32; 2],
    /// The working end: the pickaxe's head, the shovel's blade, the rod's tip.
    pub head: [f32; 2],
}

/// Where the tool in hand sits and how it moves, in `held.ron`.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct HeldSettings {
    /// A digging tool's grip in eye space, m: x right, y up, -z ahead.
    /// Tenebris's viewmodel anchor.
    pub anchor_m: [f32; 3],
    /// From the grip to the head in eye space, m: how long a tool is drawn
    /// and which way it leans.
    pub reach_m: [f32; 3],
    /// The rod's grip and tip in eye space, m (Tenebris's rod), where the
    /// fishing line leaves.
    pub rod_grip_m: [f32; 3],
    pub rod_tip_m: [f32; 3],
    /// How far the icon's plane is turned about the handle from facing the
    /// eye, rad: enough for its thickness to show.
    pub twist_rad: f32,
    /// How deep the model is, in icon pixels. Minecraft's items are one.
    pub depth_px: f32,
    pub rod: IconAxis,
    pub shovel: IconAxis,
    pub pickaxe: IconAxis,
    pub axe: IconAxis,
    /// The idle sway: amplitude right and up, m, and rate, rad/s (Tenebris).
    pub sway_m: [f32; 2],
    pub sway_rate: [f32; 2],
    /// The chop while a block is being broken (Tenebris): the swing's rate,
    /// rad/s, how far the tool drops, m, and how far it pitches, rad.
    pub swing_rate: f32,
    pub swing_drop_m: f32,
    pub swing_pitch_rad: f32,
}

impl Default for HeldSettings {
    fn default() -> Self {
        Self {
            anchor_m: [0.26, -0.26, -0.48],
            reach_m: [-0.06, 0.2, -0.1],
            rod_grip_m: [0.26, -0.26, -0.48],
            rod_tip_m: [0.26, 0.05, -0.95],
            twist_rad: 0.5,
            depth_px: 1.0,
            rod: IconAxis {
                grip: [0.5, 14.5],
                head: [15.5, 0.5],
            },
            shovel: IconAxis {
                grip: [12.5, 1.5],
                head: [2.5, 11.5],
            },
            pickaxe: IconAxis {
                grip: [0.5, 12.5],
                head: [9.5, 2.5],
            },
            axe: IconAxis {
                grip: [0.5, 11.5],
                head: [10.5, 2.5],
            },
            sway_m: [0.006, 0.005],
            sway_rate: [1.3, 1.7],
            swing_rate: 9.0,
            swing_drop_m: 0.06,
            swing_pitch_rad: 0.85,
        }
    }
}

impl HeldSettings {
    pub fn validate(&self) -> Result<(), String> {
        let points = [
            ("anchor_m", self.anchor_m),
            ("reach_m", self.reach_m),
            ("rod_grip_m", self.rod_grip_m),
            ("rod_tip_m", self.rod_tip_m),
        ];
        for (name, p) in points {
            if p.iter().any(|v| !v.is_finite() || v.abs() > 3.0) {
                return Err(format!("held.{name} must be finite and within 3 m: {p:?}"));
            }
        }
        if Vec3::from(self.reach_m).length() < 0.01
            || Vec3::from(self.rod_tip_m).distance(Vec3::from(self.rod_grip_m)) < 0.01
        {
            return Err("held: a tool needs a length".into());
        }
        for (name, axis) in [
            ("rod", self.rod),
            ("shovel", self.shovel),
            ("pickaxe", self.pickaxe),
            ("axe", self.axe),
        ] {
            let ok = axis
                .grip
                .iter()
                .chain(&axis.head)
                .all(|v| (0.0..=16.0).contains(v));
            if !ok || Vec2::from(axis.grip).distance(Vec2::from(axis.head)) < 1.0 {
                return Err(format!(
                    "held.{name}: grip and head are two pixels of the icon"
                ));
            }
        }
        let nonneg = [
            self.depth_px,
            self.sway_m[0],
            self.sway_m[1],
            self.sway_rate[0],
            self.sway_rate[1],
            self.swing_rate,
            self.swing_drop_m,
            self.swing_pitch_rad,
        ];
        if nonneg.iter().any(|v| !v.is_finite() || *v < 0.0) || self.depth_px <= 0.0 {
            return Err("held: depth, sway and swing are non-negative, depth above 0".into());
        }
        if !self.twist_rad.is_finite() || self.twist_rad.abs() > std::f32::consts::PI {
            return Err("held.twist_rad must be within half a turn".into());
        }
        Ok(())
    }

    fn axis(&self, tool: Tool) -> IconAxis {
        match tool {
            Tool::Rod => self.rod,
            Tool::Shovel => self.shovel,
            Tool::Pickaxe => self.pickaxe,
            Tool::Axe => self.axe,
        }
    }

    /// Where a tool's grip and head go, in eye space.
    fn ends(&self, tool: Tool) -> (Vec3, Vec3) {
        match tool {
            Tool::Rod => (self.rod_grip_m.into(), self.rod_tip_m.into()),
            _ => {
                let grip = Vec3::from(self.anchor_m);
                (grip, grip + Vec3::from(self.reach_m))
            }
        }
    }

    /// Where the fishing line leaves the rod, in eye space: the rod model's
    /// own tip.
    pub fn rod_tip(&self) -> Vec3 {
        self.rod_tip_m.into()
    }

    /// The model's resting pose in eye space, for an icon `height` pixels
    /// tall: its grip pixel on the grip, its head pixel on the head, its
    /// front facing the eye turned `twist_rad` about the handle.
    pub fn rest(&self, tool: Tool, height: f32) -> Transform {
        let axis = self.axis(tool);
        let model = |p: [f32; 2]| Vec3::new(p[0], height - p[1], 0.0);
        let (grip_m, head_m) = (model(axis.grip), model(axis.head));
        let (grip, head) = self.ends(tool);
        let scale = (head - grip).length() / (head_m - grip_m).length();
        // The icon's frame: along the handle, across it, and out of the page.
        let a1 = (head_m - grip_m).normalize();
        let a3 = Vec3::Z;
        let a2 = a3.cross(a1);
        // The eye's: along the held handle, and the page turned to face the
        // eye (+z in eye space) as nearly as the handle allows, then twisted.
        let b1 = (head - grip).normalize();
        let facing = (Vec3::Z - b1 * Vec3::Z.dot(b1)).normalize_or(Vec3::X);
        let b3 = Quat::from_axis_angle(b1, self.twist_rad) * facing;
        let b2 = b3.cross(b1);
        let rotation = Quat::from_mat3(
            &(Mat3::from_cols(b1, b2, b3) * Mat3::from_cols(a1, a2, a3).transpose()),
        )
        .normalize();
        let translation = grip - rotation * (grip_m * scale);
        Transform {
            translation,
            rotation,
            scale: Vec3::splat(scale),
        }
    }
}

/// Whether a block is being broken, so the tool in hand chops. Written by
/// whoever runs the digging.
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct Swinging(pub bool);

/// A held tool's model, one per tool, a child of the walking camera.
#[derive(Component)]
pub struct Held {
    tool: Tool,
    rest: Transform,
}

/// Each tool's icon, the same file its slot draws.
fn icon_bytes(tool: Tool) -> &'static [u8] {
    match tool {
        Tool::Rod => include_bytes!("../../../assets/items/tools/rod.png"),
        Tool::Shovel => include_bytes!("../../../assets/items/tools/shovel.png"),
        Tool::Pickaxe => include_bytes!("../../../assets/items/tools/pickaxe.png"),
        Tool::Axe => include_bytes!("../../../assets/items/tools/axe.png"),
    }
}

/// A tool's icon as width, height and RGBA.
pub fn icon(tool: Tool) -> (u32, u32, Vec<u8>) {
    let image = Image::from_buffer(
        icon_bytes(tool),
        ImageType::Extension("png"),
        CompressedImageFormats::NONE,
        true,
        ImageSampler::Default,
        RenderAssetUsages::default(),
    )
    .expect("a tool icon is a PNG");
    let (w, h) = (image.width(), image.height());
    let data = image.data.expect("a decoded icon has pixels");
    assert_eq!(data.len(), (w * h * 4) as usize, "{tool:?}'s icon is RGBA8");
    (w, h, data)
}

/// A tool's hexel mesh, in icon pixels.
pub fn model(tool: Tool, depth_px: f32) -> (hexel::HexelMesh, u32) {
    let (w, h, rgba) = icon(tool);
    (hexel::mesh(&hexel::hexels(w, h, &rgba), h, depth_px), h)
}

pub struct HeldPlugin;

impl Plugin for HeldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Swinging>()
            .add_systems(Update, (spawn, hold).chain());
    }
}

/// Build the four models once the walking camera exists, hidden.
fn spawn(
    mut commands: Commands,
    settings: Res<HeldConfig>,
    cameras: Query<Entity, With<WalkingCamera>>,
    existing: Query<(), With<Held>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !existing.is_empty() {
        return;
    }
    let Some(camera) = cameras.iter().next() else {
        return;
    };
    // Unlit, with the shade baked per face: Tenebris draws its tool unlit so
    // it reads at night and in a cave, and so does this.
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        fog_enabled: false,
        ..default()
    });
    for tool in Tool::ALL {
        let (built, height) = model(tool, settings.0.depth_px);
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, built.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, built.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, built.colours);
        let rest = settings.0.rest(tool, height as f32);
        let child = commands
            .spawn((
                Name::new(format!("{} in hand", tool.name())),
                Held { tool, rest },
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(material.clone()),
                rest,
                Visibility::Hidden,
                NotShadowCaster,
            ))
            .id();
        commands.entity(camera).add_child(child);
    }
}

/// Show the tool in hand, sway it, and chop while a block is being broken.
fn hold(
    settings: Res<HeldConfig>,
    tools: Res<ToolSlot>,
    walking: Option<Res<WalkingReadout>>,
    swinging: Res<Swinging>,
    time: Res<Time>,
    mut phase: Local<f32>,
    mut models: Query<(&Held, &mut Transform, &mut Visibility)>,
) {
    let s = &settings.0;
    let on_foot = walking.is_some_and(|w| w.active);
    // The chop advances while breaking and snaps back when it stops, as
    // Tenebris's does; a tool changed mid-swing does not pop.
    *phase = if swinging.0 {
        *phase + time.delta_secs() * s.swing_rate
    } else {
        0.0
    };
    let chop = if *phase > 0.0 {
        phase.sin().max(-0.2)
    } else {
        0.0
    };
    let t = time.elapsed_secs();
    let sway = Vec3::new(
        (t * s.sway_rate[0]).sin() * s.sway_m[0],
        (t * s.sway_rate[1]).cos() * s.sway_m[1],
        0.0,
    );
    for (held, mut transform, mut visibility) in &mut models {
        let shown = on_foot && tools.held() == held.tool;
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
        let (grip, _) = s.ends(held.tool);
        // The chop pitches the tool about its grip and drops it: the rod
        // does not chop, it casts.
        let chop = if held.tool == Tool::Rod { 0.0 } else { chop };
        let pitch = Quat::from_rotation_x(-chop * s.swing_pitch_rad);
        let mut pose = held.rest;
        pose.translation =
            grip + pitch * (pose.translation - grip) + sway - Vec3::Y * chop * s.swing_drop_m;
        pose.rotation = pitch * pose.rotation;
        *transform = pose;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every tool's model is built from its icon, is one connected piece (a
    /// handle that fell apart would float in the hand), and is closed.
    #[test]
    fn every_tool_is_one_piece_of_hexels() {
        for tool in Tool::ALL {
            let (w, h, rgba) = icon(tool);
            assert_eq!((w, h), (16, 16), "{tool:?}");
            let hexels = hexel::hexels(w, h, &rgba);
            assert!(hexels.len() > 20, "{tool:?}: {} hexels", hexels.len());
            let pieces = hexel::pieces(&hexels);
            assert_eq!(pieces.len(), 1, "{tool:?} is in pieces: {pieces:?}");
            let (mesh, _) = model(tool, 1.0);
            assert_eq!(mesh.positions.len(), mesh.colours.len());
            assert!(mesh.triangles() > hexels.len() * 12);
        }
    }

    /// The grip pixel lands on the grip and the head pixel on the head, and
    /// the icon's front faces the eye.
    #[test]
    fn a_tool_is_held_by_its_grip() {
        let s = HeldSettings::default();
        for tool in Tool::ALL {
            let rest = s.rest(tool, 16.0);
            let axis = s.axis(tool);
            let at = |p: [f32; 2]| rest.transform_point(Vec3::new(p[0], 16.0 - p[1], 0.0));
            let (grip, head) = s.ends(tool);
            assert!(at(axis.grip).distance(grip) < 1e-4, "{tool:?} grip");
            assert!(at(axis.head).distance(head) < 1e-4, "{tool:?} head");
            // The face turns toward the eye as far as the handle allows: the
            // rod points mostly away, so its best is about 0.44 before the
            // twist; what must never happen is the back facing the eye.
            let front = rest.rotation * Vec3::Z;
            let along = (head - grip).normalize();
            assert!(front.z > 0.2, "{tool:?} faces away: {front}");
            assert!(
                front.dot(along).abs() < 1e-4,
                "{tool:?}: face not square to the handle"
            );
        }
    }

    /// The fishing line leaves from the tip of the rod that is drawn.
    #[test]
    fn the_line_leaves_the_rod_model_tip() {
        let s = HeldSettings::default();
        let rest = s.rest(Tool::Rod, 16.0);
        let tip = rest.transform_point(Vec3::new(s.rod.head[0], 16.0 - s.rod.head[1], 0.0));
        assert!(tip.distance(s.rod_tip()) < 1e-4);
    }

    /// With the pickaxe in hand on foot, the pickaxe's model is shown and no
    /// other; flying, none is.
    #[test]
    fn only_the_tool_in_hand_is_shown() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(HeldConfig::default())
            .insert_resource(WalkingReadout {
                active: true,
                ..default()
            })
            .init_resource::<Swinging>()
            .insert_resource({
                let mut slot = ToolSlot::default();
                slot.hold(Tool::Pickaxe);
                slot
            })
            .add_systems(Update, hold);
        let s = HeldSettings::default();
        let models: Vec<Entity> = Tool::ALL
            .into_iter()
            .map(|tool| {
                let rest = s.rest(tool, 16.0);
                app.world_mut()
                    .spawn((Held { tool, rest }, rest, Visibility::Hidden))
                    .id()
            })
            .collect();
        app.update();
        let shown = |app: &App| -> Vec<Tool> {
            models
                .iter()
                .zip(Tool::ALL)
                .filter(|(e, _)| app.world().get::<Visibility>(**e) != Some(&Visibility::Hidden))
                .map(|(_, t)| t)
                .collect()
        };
        assert_eq!(shown(&app), vec![Tool::Pickaxe]);
        app.world_mut().resource_mut::<WalkingReadout>().active = false;
        app.update();
        assert!(shown(&app).is_empty(), "nothing in hand while flying");
    }

    #[test]
    fn the_defaults_validate() {
        HeldSettings::default().validate().unwrap();
        let bad = HeldSettings {
            depth_px: 0.0,
            ..Default::default()
        };
        assert!(bad.validate().is_err());
    }
}
