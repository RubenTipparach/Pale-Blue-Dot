//! The sea in the app: the one table (`pbd_core::sea`), built once from the
//! water settings and the planet's gravity; the sea state at a place, read off
//! the atmosphere; and what the water pass is handed each frame.
//!
//! The water shader draws the table with the sea state at the active camera,
//! and every hull floats on the same table with the state at the hull. Within
//! a view the two differ only by how much the weather changes over the
//! distance between them, which at the atmosphere's 181 m cells is little.

use crate::atmosphere::Air;
use crate::config::WaterSettings;
use crate::planet::terrain::{PLANET_RADIUS, surface_height};
use bevy::{prelude::*, render::extract_resource::ExtractResource};
use pbd_core::sea::{SeaGpu, SeaState, SeaTable};
use std::sync::Arc;

/// The planet's surface gravity, m/s^2: the table's frequencies follow it.
pub fn surface_gravity() -> f32 {
    pbd_core::gravity::SURFACE_GRAVITY_MPS2_PER_G as f32
}

/// The sea's constants, shared by the drawing and every hull.
#[derive(Resource, Clone)]
pub struct Sea {
    pub table: Arc<SeaTable>,
    /// The radius of the mean sea surface, m.
    pub radius: f32,
    pub depth_offset_m: f32,
}

impl Sea {
    pub fn new(water: &WaterSettings) -> Self {
        Self {
            table: Arc::new(SeaTable::new(water.sea, surface_gravity())),
            radius: PLANET_RADIUS - water.depth_offset_m,
            depth_offset_m: water.depth_offset_m,
        }
    }

    /// The sea state over a body-local direction: the wind the waves have
    /// caught up with and which way the wind blows there. Calm, with only the
    /// swell, when there is no atmosphere.
    pub fn state_at(&self, air: Option<&Air>, direction: Vec3) -> SeaState {
        match air {
            Some(air) => {
                let sample = air.now.sample(direction);
                self.table.state(sample.sea, sample.wind)
            }
            None => self.table.state(0.0, Vec3::ZERO),
        }
    }

    /// How deep the water is over a direction by the height field, m; zero on
    /// land. The column tier can say more; the waves only need the depth.
    pub fn depth_at(&self, direction: Vec3) -> f32 {
        (-surface_height(direction) - self.depth_offset_m).max(0.0)
    }
}

/// What the water pass reads this frame: the table at the camera's sea state,
/// and how high the surface stands under the camera.
#[derive(Resource, Clone, Default, ExtractResource)]
pub struct SeaNow {
    pub gpu: SeaGpu,
    /// The surface's height over the sea radius directly under the camera, m.
    pub camera_height: f32,
}

pub struct SeaPlugin;

impl Plugin for SeaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SeaNow>()
            .add_plugins(bevy::render::extract_resource::ExtractResourcePlugin::<
                SeaNow,
            >::default())
            .add_systems(PreStartup, init_sea)
            .add_systems(Startup, load_sea_module)
            .add_systems(
                PostUpdate,
                publish_sea.after(bevy::transform::TransformSystems::Propagate),
            );
    }
}

/// `shaders/sea.wgsl` is imported (`#import pbd::sea`) by the water pass, so
/// nothing loads it unless this does; held for the app's life.
#[derive(Resource)]
struct SeaModule(#[allow(dead_code)] Handle<Shader>);

fn load_sea_module(mut commands: Commands, assets: Res<AssetServer>) {
    commands.insert_resource(SeaModule(assets.load("shaders/sea.wgsl")));
}

fn init_sea(mut commands: Commands, water: Res<WaterSettings>) {
    commands.insert_resource(Sea::new(&water));
}

/// The sea state at the active camera, as the shader's table.
fn publish_sea(
    cameras: Query<(&GlobalTransform, &Camera), With<Camera3d>>,
    frame: Res<crate::planet::PlanetRenderFrame>,
    air: Option<Res<Air>>,
    sun: Res<crate::sky::Sun>,
    sea: Option<Res<Sea>>,
    mut now: ResMut<SeaNow>,
) {
    let Some(sea) = sea else {
        return;
    };
    let camera = cameras
        .iter()
        .find(|(_, camera)| camera.is_active)
        .map(|(transform, _)| (transform.translation().as_dvec3() - frame.center).as_vec3())
        .unwrap_or(Vec3::Y * sea.radius);
    let direction = camera.normalize_or(Vec3::Y);
    let seconds = sun.clock.seconds;
    let state = sea.state_at(air.as_deref(), direction);
    let local = sea
        .table
        .local(&state, direction, sea.depth_at(direction), seconds);
    now.gpu = sea.table.gpu(&state, seconds);
    now.camera_height = local.height(camera, sea.radius);
}
