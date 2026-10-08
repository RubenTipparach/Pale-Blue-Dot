//! The weather maps on the GPU: two cube textures the render world owns and
//! rewrites in place every frame with the weather the view is showing.
//!
//! The sky's clouds, the sea, the ground's cloud shadows and the overlays all
//! sample these, through one bind group layout, so every one of them sees the
//! same weather at the same place. The textures are created once, so a bind
//! group made with their views stays valid for the life of the app: nothing
//! downstream has to notice that the weather changed.
//!
//! What is written into them is the shown pair's mix (`smooth-weather`
//! decision 4): the two published states the view runs between are uploaded
//! once each into `from` and `to` textures, and a compute pass
//! (`weather_blend.wgsl`) writes their mix into the cubes every frame. Until
//! that pipeline is ready, `to` is written straight in, as every state used
//! to be.

use crate::atmosphere::{Air, MAP_SIZE, WeatherMaps};
use bevy::{
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup, RenderSystems,
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_resource::{
            binding_types::{
                sampler, texture_2d_array, texture_cube, texture_storage_2d_array, uniform_buffer,
            },
            *,
        },
        renderer::{RenderDevice, RenderQueue},
    },
};
use std::borrow::Cow;
use std::sync::Arc;

/// What the main world hands the render world each frame: the shown pair's
/// maps, the pair's generation so an unchanged pair is not uploaded again,
/// and how far the view has come between them.
#[derive(Resource, Clone, Default, ExtractResource)]
pub struct WeatherMapsNow {
    pub generation: u64,
    /// The state the blend runs from.
    pub from: Arc<WeatherMaps>,
    /// The state it runs to.
    pub maps: Arc<WeatherMaps>,
    /// How far from `from` to `maps`, 0..1.
    pub t: f32,
    /// The overlay's map, when one is showing: see `crate::overlay`.
    pub overlay: Option<Arc<Vec<[f32; 4]>>>,
    pub overlay_generation: u64,
    /// Which overlay the map on the GPU holds, and so which one may be drawn:
    /// `None` until the map for the one asked for has been built.
    pub overlay_kind: Option<pbd_core::overlay::Overlay>,
}

fn publish(air: Option<Res<Air>>, mut now: ResMut<WeatherMapsNow>) {
    let Some(air) = air else {
        return;
    };
    if air.shown.pair != now.generation {
        let (from, to) = air.shown.maps();
        now.generation = air.shown.pair;
        now.from = from.clone();
        now.maps = to.clone();
    }
    if now.t != air.shown.t {
        now.t = air.shown.t;
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

/// The blend pass's bind group layout: how far, then each map's `from`,
/// `to` and the cube it writes, as a 2D array of its six faces.
fn blend_layout() -> BindGroupLayoutDescriptor {
    let read = || texture_2d_array(TextureSampleType::Float { filterable: false });
    let write =
        || texture_storage_2d_array(TextureFormat::Rgba16Float, StorageTextureAccess::WriteOnly);
    BindGroupLayoutDescriptor::new(
        "weather maps blend",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::COMPUTE,
            (
                uniform_buffer::<Vec4>(false),
                read(),
                read(),
                write(),
                read(),
                read(),
                write(),
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
    /// The shown pair, as uploaded.
    cloud_from: Texture,
    cloud_to: Texture,
    wind_from: Texture,
    wind_to: Texture,
    /// The blend pass: how far, its bind group and its pipeline.
    blend: Buffer,
    blend_group: BindGroup,
    pipeline: CachedComputePipelineId,
}

fn cube(device: &RenderDevice, label: &'static str) -> Texture {
    texture(
        device,
        label,
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING,
    )
}

/// A texture of the weather maps' size and format, six layers deep.
fn texture(device: &RenderDevice, label: &'static str, usage: TextureUsages) -> Texture {
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
        // Every adapter can also write them from a compute pass.
        format: TextureFormat::Rgba16Float,
        usage,
        view_formats: &[],
    })
}

/// A texture's six faces as a 2D array, for the blend pass.
fn layers_view(texture: &Texture) -> TextureView {
    texture.create_view(&TextureViewDescriptor {
        dimension: Some(TextureViewDimension::D2Array),
        ..default()
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
    assets: Res<AssetServer>,
) {
    let cloud = cube(&device, "weather map: cloud");
    let wind = cube(&device, "weather map: wind");
    let overlay = cube(&device, "weather map: overlay");
    let pair = |label| {
        texture(
            &device,
            label,
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        )
    };
    let cloud_from = pair("weather map: cloud, from");
    let cloud_to = pair("weather map: cloud, to");
    let wind_from = pair("weather map: wind, from");
    let wind_to = pair("weather map: wind, to");
    let blank = vec![[0.0f32; 4]; 6 * MAP_SIZE * MAP_SIZE];
    for texture in [
        &cloud,
        &wind,
        &overlay,
        &cloud_from,
        &cloud_to,
        &wind_from,
        &wind_to,
    ] {
        write(&queue, texture, &blank);
    }
    let blend = device.create_buffer(&BufferDescriptor {
        label: Some("weather maps blend"),
        size: 16,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let blend_layout = blend_layout();
    let blend_group = device.create_bind_group(
        Some("weather maps blend"),
        &cache.get_bind_group_layout(&blend_layout),
        &BindGroupEntries::sequential((
            blend.as_entire_binding(),
            &layers_view(&cloud_from),
            &layers_view(&cloud_to),
            &layers_view(&cloud),
            &layers_view(&wind_from),
            &layers_view(&wind_to),
            &layers_view(&wind),
        )),
    );
    let pipeline = cache.queue_compute_pipeline(ComputePipelineDescriptor {
        label: Some(Cow::Borrowed("weather maps blend")),
        layout: vec![blend_layout],
        shader: assets.load("shaders/weather_blend.wgsl"),
        entry_point: Some(Cow::Borrowed("blend_maps")),
        ..default()
    });
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
        cloud_from,
        cloud_to,
        wind_from,
        wind_to,
        blend,
        blend_group,
        pipeline,
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
        // The pair, once; and `to` straight into the cubes, which is what
        // shows until the blend pass is ready to write the mix there.
        let from = if now.from.cloud.is_empty() {
            &now.maps
        } else {
            &now.from
        };
        write(&queue, &gpu.cloud_from, &from.cloud);
        write(&queue, &gpu.wind_from, &from.wind);
        write(&queue, &gpu.cloud_to, &now.maps.cloud);
        write(&queue, &gpu.wind_to, &now.maps.wind);
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

/// Write the shown pair's mix into the cubes, every frame, before the frame's
/// passes read them (`smooth-weather` decision 4). Submitted on its own, after
/// the uploads, which the queue runs first.
fn blend(
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    cache: Res<PipelineCache>,
    now: Res<WeatherMapsNow>,
    gpu: Option<Res<WeatherMapGpu>>,
) {
    let Some(gpu) = gpu else {
        return;
    };
    if gpu.generation == 0 {
        return;
    }
    let Some(pipeline) = cache.get_compute_pipeline(gpu.pipeline) else {
        return;
    };
    let t = [now.t.clamp(0.0, 1.0), 0.0, 0.0, 0.0];
    let bytes: Vec<u8> = t.iter().flat_map(|v| v.to_le_bytes()).collect();
    queue.write_buffer(&gpu.blend, 0, &bytes);
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("weather maps blend"),
    });
    {
        let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor {
            label: Some("weather maps blend"),
            ..default()
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &gpu.blend_group, &[]);
        let groups = (MAP_SIZE as u32).div_ceil(8);
        pass.dispatch_workgroups(groups, groups, 6);
    }
    queue.submit([encoder.finish()]);
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
        .add_systems(
            Render,
            (upload, blend)
                .chain()
                .in_set(RenderSystems::PrepareResources),
        );
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
