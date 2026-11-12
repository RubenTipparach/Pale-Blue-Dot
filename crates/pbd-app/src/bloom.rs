//! `lighting-model` 3.3: a cheaper bloom than Bevy's (`bloom_lite.wgsl`).
//!
//! Bevy's `Bloom` cost about 2.4 ms on the cloud flight on an M4 Max, the same
//! at mip sizes 512, 256 and 128, so its cost was not the blur. This one reads
//! the HDR image once, straight to a quarter of its size, runs the dual filter
//! (five taps down, eight up) over five small levels, and adds the result onto
//! the image in one more pass: ten small passes and one full-size read and
//! write. It runs where Bevy's would, before the display transform
//! (`tone.rs`), on cameras that carry [`LiteBloom`].

use bevy::{
    core_pipeline::{
        FullscreenShader,
        core_3d::graph::{Core3d, Node3d},
    },
    ecs::query::QueryItem,
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup, RenderSystems,
        camera::ExtractedCamera,
        diagnostic::RecordDiagnostics,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_graph::{
            NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{sampler, texture_2d, uniform_buffer},
            *,
        },
        renderer::{RenderContext, RenderDevice},
        texture::{CachedTexture, TextureCache},
        view::ViewTarget,
    },
};

/// The bloom on a camera: how strongly it adds, and the HDR level it starts
/// at (with a soft knee half as wide).
#[derive(Component, ExtractComponent, Clone, Copy, Debug, PartialEq)]
pub struct LiteBloom {
    pub intensity: f32,
    pub threshold: f32,
}

/// How many levels the chain has, the first a quarter of the screen.
const LEVELS: u32 = 5;
const FORMAT: TextureFormat = TextureFormat::Rg11b10Ufloat;

#[derive(ShaderType, Clone, Copy)]
struct LiteBloomUniform {
    look: Vec4,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct LiteBloomLabel;

pub struct LiteBloomPlugin;

impl Plugin for LiteBloomPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<LiteBloom>::default());
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };
        render_app
            .add_systems(RenderStartup, init_pipelines)
            .add_systems(
                Render,
                prepare_textures.in_set(RenderSystems::PrepareResources),
            )
            .add_render_graph_node::<ViewNodeRunner<LiteBloomNode>>(Core3d, LiteBloomLabel)
            .add_render_graph_edges(
                Core3d,
                (
                    Node3d::StartMainPassPostProcessing,
                    LiteBloomLabel,
                    Node3d::Tonemapping,
                ),
            );
    }
}

#[derive(Resource)]
struct LiteBloomPipelines {
    layout: BindGroupLayoutDescriptor,
    sampler: Sampler,
    down_first: CachedRenderPipelineId,
    down: CachedRenderPipelineId,
    up: CachedRenderPipelineId,
    composite: CachedRenderPipelineId,
}

fn init_pipelines(
    mut commands: Commands,
    assets: Res<AssetServer>,
    device: Res<RenderDevice>,
    cache: Res<PipelineCache>,
    fullscreen: Res<FullscreenShader>,
) {
    let layout = BindGroupLayoutDescriptor::new(
        "lite bloom",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d(TextureSampleType::Float { filterable: true }),
                sampler(SamplerBindingType::Filtering),
                uniform_buffer::<LiteBloomUniform>(false),
            ),
        ),
    );
    let shader: Handle<Shader> = assets.load("shaders/bloom_lite.wgsl");
    let add = BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::One,
        operation: BlendOperation::Add,
    };
    // The last pass adds at the bloom's strength, given as the blend constant.
    let add_scaled = BlendComponent {
        src_factor: BlendFactor::Constant,
        dst_factor: BlendFactor::One,
        operation: BlendOperation::Add,
    };
    let keep_alpha = BlendComponent {
        src_factor: BlendFactor::Zero,
        dst_factor: BlendFactor::One,
        operation: BlendOperation::Add,
    };
    let pipeline = |label: &'static str, entry: &'static str, format, blend| {
        cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some(label.into()),
            layout: vec![layout.clone()],
            vertex: fullscreen.to_vertex_state(),
            fragment: Some(FragmentState {
                shader: shader.clone(),
                shader_defs: vec![],
                entry_point: Some(entry.into()),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            ..default()
        })
    };
    let down_first = pipeline("lite bloom: first down", "down_first", FORMAT, None);
    let down = pipeline("lite bloom: down", "down", FORMAT, None);
    let up = pipeline(
        "lite bloom: up",
        "up",
        FORMAT,
        Some(BlendState {
            color: add,
            alpha: add,
        }),
    );
    let composite = pipeline(
        "lite bloom: onto the image",
        "up",
        ViewTarget::TEXTURE_FORMAT_HDR,
        Some(BlendState {
            color: add_scaled,
            alpha: keep_alpha,
        }),
    );
    commands.insert_resource(LiteBloomPipelines {
        layout,
        sampler: device.create_sampler(&SamplerDescriptor {
            label: Some("lite bloom sampler"),
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            ..default()
        }),
        down_first,
        down,
        up,
        composite,
    });
}

/// The chain's levels, one texture with a mip each.
#[derive(Component)]
struct LiteBloomTexture {
    texture: CachedTexture,
    size: UVec2,
}

impl LiteBloomTexture {
    fn view(&self, level: u32) -> TextureView {
        self.texture.texture.create_view(&TextureViewDescriptor {
            base_mip_level: level,
            mip_level_count: Some(1),
            ..default()
        })
    }

    fn level_size(&self, level: u32) -> UVec2 {
        (self.size >> level).max(UVec2::ONE)
    }
}

fn prepare_textures(
    mut commands: Commands,
    mut cache: ResMut<TextureCache>,
    device: Res<RenderDevice>,
    views: Query<(Entity, &ExtractedCamera), With<LiteBloom>>,
) {
    for (entity, camera) in &views {
        let Some(viewport) = camera.physical_viewport_size else {
            continue;
        };
        let size = (viewport / 4).max(UVec2::ONE);
        let texture = cache.get(
            &device,
            TextureDescriptor {
                label: Some("lite bloom levels"),
                size: Extent3d {
                    width: size.x,
                    height: size.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: LEVELS,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: FORMAT,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
        );
        commands
            .entity(entity)
            .insert(LiteBloomTexture { texture, size });
    }
}

#[derive(Default)]
struct LiteBloomNode;

impl ViewNode for LiteBloomNode {
    type ViewQuery = (
        &'static ExtractedCamera,
        &'static ViewTarget,
        &'static LiteBloomTexture,
        &'static LiteBloom,
    );

    fn run<'w>(
        &self,
        _: &mut RenderGraphContext,
        ctx: &mut RenderContext<'w>,
        (camera, target, levels, bloom): QueryItem<'w, '_, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        if bloom.intensity <= 0.0 {
            return Ok(());
        }
        let Some(pipelines) = world.get_resource::<LiteBloomPipelines>() else {
            return Ok(());
        };
        let cache = world.resource::<PipelineCache>();
        let (Some(down_first), Some(down), Some(up), Some(composite)) = (
            cache.get_render_pipeline(pipelines.down_first),
            cache.get_render_pipeline(pipelines.down),
            cache.get_render_pipeline(pipelines.up),
            cache.get_render_pipeline(pipelines.composite),
        ) else {
            return Ok(());
        };
        let Some(viewport) = camera.physical_viewport_size else {
            return Ok(());
        };
        let device = ctx.render_device().clone();
        let layout = cache.get_bind_group_layout(&pipelines.layout);
        // One bind group per pass: what it reads, and one texel of it.
        let group = |source: &TextureView, texel: Vec2| {
            let mut bytes = encase::UniformBuffer::new(Vec::<u8>::new());
            bytes
                .write(&LiteBloomUniform {
                    look: Vec4::new(bloom.threshold, bloom.threshold * 0.5, texel.x, texel.y),
                })
                .expect("the bloom's uniform encodes");
            let uniform = device.create_buffer_with_data(&BufferInitDescriptor {
                label: Some("lite bloom uniform"),
                contents: bytes.as_ref(),
                usage: BufferUsages::UNIFORM,
            });
            device.create_bind_group(
                Some("lite bloom"),
                &layout,
                &BindGroupEntries::sequential((
                    source,
                    &pipelines.sampler,
                    uniform.as_entire_binding(),
                )),
            )
        };
        let texel = |size: UVec2| Vec2::ONE / size.as_vec2();
        let views: Vec<TextureView> = (0..LEVELS).map(|l| levels.view(l)).collect();
        let diagnostics = ctx.diagnostic_recorder();
        let encoder = ctx.command_encoder();
        let span = diagnostics.time_span(encoder, "lite_bloom");
        let run = |encoder: &mut CommandEncoder,
                   pipeline: &RenderPipeline,
                   group: &BindGroup,
                   into: &TextureView,
                   clear: bool| {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("lite bloom"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: into,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: if clear {
                            LoadOp::Clear(LinearRgba::NONE.into())
                        } else {
                            LoadOp::Load
                        },
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, group, &[]);
            pass.draw(0..3, 0..1);
        };
        // Down: the image to a quarter, then each level to the next.
        let first = group(target.main_texture_view(), texel(viewport));
        run(encoder, down_first, &first, &views[0], true);
        for level in 1..LEVELS {
            let g = group(
                &views[level as usize - 1],
                texel(levels.level_size(level - 1)),
            );
            run(encoder, down, &g, &views[level as usize], true);
        }
        // Up: each level added onto the one above it.
        for level in (1..LEVELS).rev() {
            let g = group(&views[level as usize], texel(levels.level_size(level)));
            run(encoder, up, &g, &views[level as usize - 1], false);
        }
        // Onto the image, at the bloom's strength.
        let last = group(&views[0], texel(levels.level_size(0)));
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("lite bloom: onto the image"),
                color_attachments: &[Some(target.get_unsampled_color_attachment())],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(composite);
            pass.set_bind_group(0, &last, &[]);
            pass.set_blend_constant(LinearRgba::gray(bloom.intensity).into());
            pass.draw(0..3, 0..1);
        }
        span.end(encoder);
        Ok(())
    }
}
