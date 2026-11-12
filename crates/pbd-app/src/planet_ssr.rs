//! `lighting-model` L8 (10.2): the screen trace's history. After the water's
//! composite (so the sea and the clouds are in it), the frame's colour and
//! each pixel's distance from the eye go into `PlanetGpu::ssr`, a small
//! texture of fixed size. The next frame's wet ground reprojects its reflected
//! ray into it with this frame's matrix (`PlanetParams::ssr_clip`): no second
//! draw of the scene, and nothing to rebind when the window changes size.
//! Where the walk over the columns finds a block, this is not read at all.

use super::{PlanetGpu, PlanetViewGpu};
use bevy::{
    core_pipeline::{
        FullscreenShader,
        core_3d::graph::{Core3d, Node3d},
    },
    ecs::query::QueryItem,
    prelude::*,
    render::{
        RenderStartup,
        render_graph::{
            NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{
                sampler, texture_2d, texture_depth_2d, texture_depth_2d_multisampled,
                uniform_buffer,
            },
            *,
        },
        renderer::{RenderContext, RenderDevice},
        view::{ExtractedView, Msaa, ViewDepthTexture, ViewTarget},
    },
    shader::ShaderDefVal,
};

/// The history's size and format: half of 1920 x 1200, whatever the window.
pub(super) const SIZE: UVec2 = UVec2::new(960, 600);
pub(super) const FORMAT: TextureFormat = TextureFormat::Rgba16Float;

#[derive(ShaderType, Clone, Copy)]
struct SsrCaptureUniform {
    view_from_clip: Mat4,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct SsrCaptureLabel;

#[derive(Resource)]
struct SsrPipelines {
    layouts: [BindGroupLayoutDescriptor; 2],
    pipelines: [CachedRenderPipelineId; 2],
    sampler: Sampler,
}

pub(super) fn build(render_app: &mut SubApp) {
    render_app
        .add_systems(RenderStartup, init_pipelines)
        .add_render_graph_node::<ViewNodeRunner<SsrCaptureNode>>(Core3d, SsrCaptureLabel)
        .add_render_graph_edges(
            Core3d,
            (
                super::water::WaterCompositeLabel,
                SsrCaptureLabel,
                Node3d::StartMainPassPostProcessing,
            ),
        );
}

/// The history, created with the planet's other persistent resources.
pub(super) fn create_texture(device: &RenderDevice) -> (Texture, TextureView) {
    let texture = device.create_texture(&TextureDescriptor {
        label: Some("screen trace history"),
        size: Extent3d {
            width: SIZE.x,
            height: SIZE.y,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: FORMAT,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&TextureViewDescriptor::default());
    (texture, view)
}

fn init_pipelines(
    mut commands: Commands,
    assets: Res<AssetServer>,
    device: Res<RenderDevice>,
    cache: Res<PipelineCache>,
    fullscreen: Res<FullscreenShader>,
) {
    let shader: Handle<Shader> = assets.load("shaders/ssr_capture.wgsl");
    let layout = |multisampled: bool| {
        BindGroupLayoutDescriptor::new(
            "screen trace capture",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    if multisampled {
                        texture_depth_2d_multisampled()
                    } else {
                        texture_depth_2d()
                    },
                    uniform_buffer::<SsrCaptureUniform>(false),
                ),
            ),
        )
    };
    let layouts = [layout(false), layout(true)];
    let pipelines = [false, true].map(|multisampled| {
        cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some("screen trace capture".into()),
            layout: vec![layouts[multisampled as usize].clone()],
            vertex: fullscreen.to_vertex_state(),
            fragment: Some(FragmentState {
                shader: shader.clone(),
                shader_defs: if multisampled {
                    vec![ShaderDefVal::from("MULTISAMPLED")]
                } else {
                    vec![]
                },
                entry_point: Some("fragment".into()),
                targets: vec![Some(ColorTargetState {
                    format: FORMAT,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            ..default()
        })
    });
    commands.insert_resource(SsrPipelines {
        layouts,
        pipelines,
        sampler: device.create_sampler(&SamplerDescriptor {
            label: Some("screen trace capture sampler"),
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..default()
        }),
    });
}

#[derive(Default)]
struct SsrCaptureNode;

impl ViewNode for SsrCaptureNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static ViewDepthTexture,
        &'static ExtractedView,
        &'static Msaa,
        &'static PlanetViewGpu,
    );

    fn run<'w>(
        &self,
        _: &mut RenderGraphContext,
        ctx: &mut RenderContext<'w>,
        (target, depth, view, msaa, _): QueryItem<'w, '_, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let on = world
            .get_resource::<crate::config::LightingSettings>()
            .is_some_and(|l| l.reflections_on());
        let (Some(pipelines), Some(planet)) = (
            world.get_resource::<SsrPipelines>(),
            world.get_resource::<PlanetGpu>(),
        ) else {
            return Ok(());
        };
        if !on {
            return Ok(());
        }
        let cache = world.resource::<PipelineCache>();
        let multisampled = (msaa.samples() > 1) as usize;
        let Some(pipeline) = cache.get_render_pipeline(pipelines.pipelines[multisampled]) else {
            return Ok(());
        };
        let device = ctx.render_device().clone();
        let mut bytes = encase::UniformBuffer::new(Vec::<u8>::new());
        bytes
            .write(&SsrCaptureUniform {
                view_from_clip: view.clip_from_view.inverse(),
            })
            .expect("the capture's uniform encodes");
        let uniform = device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("screen trace capture uniform"),
            contents: bytes.as_ref(),
            usage: BufferUsages::UNIFORM,
        });
        let group = device.create_bind_group(
            Some("screen trace capture"),
            &cache.get_bind_group_layout(&pipelines.layouts[multisampled]),
            &BindGroupEntries::sequential((
                target.main_texture_view(),
                &pipelines.sampler,
                depth.view(),
                uniform.as_entire_binding(),
            )),
        );
        let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("screen trace capture"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &planet.ssr_view,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(LinearRgba::NONE.into()),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_render_pipeline(pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.draw(0..3, 0..1);
        Ok(())
    }
}
