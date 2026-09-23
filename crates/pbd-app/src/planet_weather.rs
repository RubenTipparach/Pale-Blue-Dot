//! The weather maps on the GPU: two cube textures the render world owns and
//! rewrites in place whenever the atmosphere publishes a new state.
//!
//! The sky's clouds, the sea, the ground's cloud shadows and the overlays all
//! sample these, through one bind group layout, so every one of them sees the
//! same weather at the same place. The textures are created once and written
//! with `write_texture`, so a bind group made with their views stays valid for
//! the life of the app: nothing downstream has to notice that the weather
//! changed.

use crate::atmosphere::{Air, MAP_SIZE, WeatherMaps};
use bevy::{
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup, RenderSystems,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_resource::{
            binding_types::{sampler, texture_cube},
            *,
        },
        renderer::{RenderDevice, RenderQueue},
    },
};
use std::sync::Arc;

/// What the main world hands the render world each frame: the published
/// maps, and their generation so an unchanged state is not uploaded again.
#[derive(Resource, Clone, Default, ExtractResource)]
pub struct WeatherMapsNow {
    pub generation: u64,
    pub maps: Arc<WeatherMaps>,
    /// The overlay's map, when one is showing: see `crate::overlay`.
    pub overlay: Option<Arc<Vec<[f32; 4]>>>,
    pub overlay_generation: u64,
}

fn publish(air: Option<Res<Air>>, mut now: ResMut<WeatherMapsNow>) {
    if let Some(air) = air
        && air.generation != now.generation
    {
        now.generation = air.generation;
        now.maps = air.maps.clone();
    }
}

/// The weather maps' bind group layout: the cloud cube, the wind cube and the
/// overlay cube, all filtered, and their sampler.
pub fn layout() -> BindGroupLayoutDescriptor {
    BindGroupLayoutDescriptor::new(
        "weather maps",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::VERTEX_FRAGMENT,
            (
                texture_cube(TextureSampleType::Float { filterable: true }),
                texture_cube(TextureSampleType::Float { filterable: true }),
                texture_cube(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
            ),
        ),
    )
}

/// The render world's textures and the one bind group every consumer sets.
#[derive(Resource)]
pub struct WeatherMapGpu {
    cloud: Texture,
    wind: Texture,
    overlay: Texture,
    pub bind_group: BindGroup,
    generation: u64,
    overlay_generation: u64,
}

fn cube(device: &RenderDevice, label: &'static str) -> Texture {
    device.create_texture(&TextureDescriptor {
        label: Some(label),
        size: Extent3d {
            width: MAP_SIZE as u32,
            height: MAP_SIZE as u32,
            depth_or_array_layers: 6,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        // Half floats: filterable on every adapter, where 32-bit floats need
        // a feature, and a cover or a wind speed needs nothing like their range.
        format: TextureFormat::Rgba16Float,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

fn cube_view(texture: &Texture) -> TextureView {
    texture.create_view(&TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    })
}

fn create(
    mut commands: Commands,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    cache: Res<PipelineCache>,
) {
    let cloud = cube(&device, "weather map: cloud");
    let wind = cube(&device, "weather map: wind");
    let overlay = cube(&device, "weather map: overlay");
    let blank = vec![[0.0f32; 4]; 6 * MAP_SIZE * MAP_SIZE];
    for texture in [&cloud, &wind, &overlay] {
        write(&queue, texture, &blank);
    }
    let sampler = device.create_sampler(&SamplerDescriptor {
        label: Some("weather maps"),
        mag_filter: FilterMode::Linear,
        min_filter: FilterMode::Linear,
        address_mode_u: AddressMode::ClampToEdge,
        address_mode_v: AddressMode::ClampToEdge,
        address_mode_w: AddressMode::ClampToEdge,
        ..default()
    });
    let bind_group = device.create_bind_group(
        Some("weather maps"),
        &cache.get_bind_group_layout(&layout()),
        &BindGroupEntries::sequential((
            &cube_view(&cloud),
            &cube_view(&wind),
            &cube_view(&overlay),
            &sampler,
        )),
    );
    commands.insert_resource(WeatherMapGpu {
        cloud,
        wind,
        overlay,
        bind_group,
        generation: 0,
        overlay_generation: 0,
    });
}

/// Upload a cube's six faces from `[f32; 4]` texels as half floats.
fn write(queue: &RenderQueue, texture: &Texture, texels: &[[f32; 4]]) {
    let size = MAP_SIZE as u32;
    if texels.len() != 6 * MAP_SIZE * MAP_SIZE {
        return;
    }
    let mut bytes = Vec::with_capacity(texels.len() * 8);
    for texel in texels {
        for value in texel {
            bytes.extend_from_slice(&half_bits(*value).to_le_bytes());
        }
    }
    queue.write_texture(
        TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        &bytes,
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(size * 8),
            rows_per_image: Some(size),
        },
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 6,
        },
    );
}

fn upload(queue: Res<RenderQueue>, now: Res<WeatherMapsNow>, gpu: Option<ResMut<WeatherMapGpu>>) {
    let Some(mut gpu) = gpu else {
        return;
    };
    if now.generation != gpu.generation && !now.maps.cloud.is_empty() {
        write(&queue, &gpu.cloud, &now.maps.cloud);
        write(&queue, &gpu.wind, &now.maps.wind);
        gpu.generation = now.generation;
    }
    if now.overlay_generation != gpu.overlay_generation
        && let Some(overlay) = &now.overlay
    {
        write(&queue, &gpu.overlay, overlay);
        gpu.overlay_generation = now.overlay_generation;
    }
}

/// An `f32` as IEEE half-float bits, rounded to nearest, with infinities and
/// NaN folded to the largest finite half: nothing the shader reads may be
/// non-finite.
pub fn half_bits(value: f32) -> u16 {
    let value = if value.is_finite() {
        value.clamp(-65504.0, 65504.0)
    } else {
        0.0
    };
    let bits = value.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exponent = ((bits >> 23) & 0xff) as i32;
    let mantissa = bits & 0x007f_ffff;
    if exponent == 0 {
        return sign;
    }
    let half_exponent = exponent - 127 + 15;
    if half_exponent >= 31 {
        return sign | 0x7bff;
    }
    if half_exponent <= 0 {
        // Subnormal half: shift the implicit one in.
        if half_exponent < -10 {
            return sign;
        }
        let m = mantissa | 0x0080_0000;
        let shift = (14 - half_exponent) as u32;
        let rounded = (m + (1 << (shift - 1))) >> shift;
        return sign | rounded as u16;
    }
    let rounded = mantissa + 0x0000_1000;
    if rounded & 0x0080_0000 != 0 {
        let e = half_exponent + 1;
        if e >= 31 {
            return sign | 0x7bff;
        }
        return sign | ((e as u16) << 10);
    }
    sign | ((half_exponent as u16) << 10) | ((rounded >> 13) as u16)
}

pub(super) fn build(app: &mut App) {
    app.init_resource::<WeatherMapsNow>()
        .add_plugins(ExtractResourcePlugin::<WeatherMapsNow>::default())
        .add_systems(PostUpdate, publish);
    app.sub_app_mut(RenderApp)
        .add_systems(RenderStartup, create)
        .add_systems(Render, upload.in_set(RenderSystems::PrepareResources));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Half-float bits for values whose halves are exact, and a round trip
    /// through a reference decoder for the rest.
    #[test]
    fn half_floats_round_to_the_nearest() {
        assert_eq!(half_bits(0.0), 0x0000);
        assert_eq!(half_bits(1.0), 0x3c00);
        assert_eq!(half_bits(-2.0), 0xc000);
        assert_eq!(half_bits(0.5), 0x3800);
        assert_eq!(half_bits(65504.0), 0x7bff);
        assert_eq!(half_bits(1.0e9), 0x7bff);
        assert_eq!(half_bits(f32::NAN), 0x0000);
        let decode = |h: u16| {
            let sign = if h & 0x8000 != 0 { -1.0 } else { 1.0 };
            let e = ((h >> 10) & 0x1f) as i32;
            let m = (h & 0x3ff) as f32;
            sign * if e == 0 {
                m / 1024.0 * 2f32.powi(-14)
            } else {
                (1.0 + m / 1024.0) * 2f32.powi(e - 15)
            }
        };
        for x in [0.3f32, 12.75, -41.2, 0.0007, 3.1e-5, 0.999] {
            let back = decode(half_bits(x));
            assert!(((back - x) / x).abs() < 1e-3, "{x} -> {back}");
        }
    }
}
