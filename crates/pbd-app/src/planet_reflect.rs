//! `lighting-perf` 2.8: the terrain's own pass, and L8's traces at half size.
//!
//! The terrain draws here, between Bevy's transmissive and transparent passes
//! (where it drew before as the transparent phase's first item). While
//! `reflect_render_scale` is under one (and the view is MSAA), its fragment
//! shader (`fragment_split`) leaves each wet pixel's traces undone: it draws
//! the miss and writes beside its colour the request (the column slot and the
//! reflected ray) and what a hit would replace. Then:
//!
//! - `reflect_half` traces one pixel of each block, at the scale, with the
//!   terrain's own `reflect_trace` and `ssr_trace` (`reflect_gather`);
//! - `reflect_compose` adds the hits back over the view, each pixel taking
//!   the texels round it by distance and ray, as the inline trace would have
//!   added them.
//!
//! The plan is the design's "L8's traces at half size: the pass layout".

use super::{PlanetDraw, PlanetViewGpu, shadow, weather_maps};
use bevy::{
    core_pipeline::{
        FullscreenShader,
        core_3d::graph::{Core3d, Node3d},
    },
    ecs::query::QueryItem,
    math::DMat4,
    prelude::*,
    render::{
        Render, RenderStartup, RenderSystems,
        diagnostic::RecordDiagnostics,
        render_graph::{
            NodeRunError, RenderGraphContext, RenderGraphExt, RenderLabel, ViewNode, ViewNodeRunner,
        },
        render_resource::{
            binding_types::{
                texture_2d, texture_2d_multisampled, texture_depth_2d_multisampled, uniform_buffer,
            },
            *,
        },
        renderer::{RenderContext, RenderDevice},
        view::{ExtractedView, ViewDepthTexture, ViewTarget},
    },
};

/// A wet pixel's request: the slot in two halves (the top bit of the upper
/// one says "asked") and the reflected ray, octahedral.
pub(super) const REQUEST_FORMAT: TextureFormat = TextureFormat::Rgba16Uint;
/// What a hit replaces, through the haze: `k*B` and `k*fr`.
pub(super) const REPLACED_FORMAT: TextureFormat = TextureFormat::Rgba16Float;
/// The half-size hit (colour, weight) and the ray and distance it was for.
const HALF_FORMAT: TextureFormat = TextureFormat::Rgba16Float;
/// The camera's MSAA (`desktop.rs`); the passes are built for it.
pub(super) const SAMPLES: u32 = 4;

#[derive(ShaderType, Clone, Copy)]
struct ReflectViewUniform {
    body_from_clip: Mat4,
    eye: Vec4,
    size: Vec4,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct PlanetTerrainLabel;

#[derive(Resource)]
struct ReflectPipelines {
    gather_layout: BindGroupLayoutDescriptor,
    compose_layout: BindGroupLayoutDescriptor,
    gather: CachedRenderPipelineId,
    compose: CachedRenderPipelineId,
}

/// A view's targets for the split, made at its size and remade when that
/// or the scale changes.
#[derive(Component)]
struct ReflectTargets {
    size: UVec2,
    scale: f32,
    half: UVec2,
    request: TextureView,
    replaced_ms: TextureView,
    replaced: TextureView,
    hit: TextureView,
    ray: TextureView,
}

pub(super) fn build(render_app: &mut SubApp) {
    render_app
        .add_systems(RenderStartup, init_pipelines)
        .add_systems(
            Render,
            prepare_targets.in_set(RenderSystems::PrepareResources),
        )
        .add_render_graph_node::<ViewNodeRunner<PlanetTerrainNode>>(Core3d, PlanetTerrainLabel)
        .add_render_graph_edges(
            Core3d,
            (
                Node3d::MainTransmissivePass,
                PlanetTerrainLabel,
                Node3d::MainTransparentPass,
            ),
        );
}

fn init_pipelines(
    mut commands: Commands,
    assets: Res<AssetServer>,
    cache: Res<PipelineCache>,
    fullscreen: Res<FullscreenShader>,
) {
    let gather_layout = BindGroupLayoutDescriptor::new(
        "reflect gather",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d_multisampled(TextureSampleType::Uint),
                texture_depth_2d_multisampled(),
                uniform_buffer::<ReflectViewUniform>(false),
            ),
        ),
    );
    let unfiltered = TextureSampleType::Float { filterable: false };
    let compose_layout = BindGroupLayoutDescriptor::new(
        "reflect compose",
        &BindGroupLayoutEntries::sequential(
            ShaderStages::FRAGMENT,
            (
                texture_2d_multisampled(TextureSampleType::Uint),
                texture_depth_2d_multisampled(),
                texture_2d(unfiltered),
                texture_2d(unfiltered),
                texture_2d(unfiltered),
                uniform_buffer::<ReflectViewUniform>(false),
            ),
        ),
    );
    let surface: Handle<Shader> = assets.load("shaders/planet_surface.wgsl");
    let half_target = Some(ColorTargetState {
        format: HALF_FORMAT,
        blend: None,
        write_mask: ColorWrites::ALL,
    });
    let gather = cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("reflect gather".into()),
        layout: vec![
            super::draw_layout(),
            weather_maps::layout(),
            shadow::receiver_layout(),
            gather_layout.clone(),
        ],
        vertex: fullscreen.to_vertex_state(),
        fragment: Some(FragmentState {
            shader: surface,
            entry_point: Some("reflect_gather".into()),
            targets: vec![half_target.clone(), half_target],
            ..default()
        }),
        ..default()
    });
    let compose = cache.queue_render_pipeline(RenderPipelineDescriptor {
        label: Some("reflect compose".into()),
        layout: vec![compose_layout.clone()],
        vertex: fullscreen.to_vertex_state(),
        fragment: Some(FragmentState {
            shader: assets.load("shaders/reflect_compose.wgsl"),
            entry_point: Some("fragment".into()),
            targets: vec![Some(ColorTargetState {
                format: ViewTarget::TEXTURE_FORMAT_HDR,
                // Added to what the terrain drew, every sample alike, so the
                // resolve (a mean) takes the mean of what each sample asked.
                blend: Some(BlendState {
                    color: BlendComponent {
                        src_factor: BlendFactor::One,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,
                    },
                    alpha: BlendComponent {
                        src_factor: BlendFactor::Zero,
                        dst_factor: BlendFactor::One,
                        operation: BlendOperation::Add,
                    },
                }),
                write_mask: ColorWrites::COLOR,
            })],
            ..default()
        }),
        multisample: MultisampleState {
            count: SAMPLES,
            ..default()
        },
        ..default()
    });
    commands.insert_resource(ReflectPipelines {
        gather_layout,
        compose_layout,
        gather,
        compose,
    });
}

fn target(
    device: &RenderDevice,
    label: &'static str,
    size: UVec2,
    samples: u32,
    format: TextureFormat,
    usage: TextureUsages,
) -> TextureView {
    device
        .create_texture(&TextureDescriptor {
            label: Some(label),
            size: Extent3d {
                width: size.x.max(1),
                height: size.y.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: samples,
            dimension: TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        })
        .create_view(&TextureViewDescriptor::default())
}

fn prepare_targets(
    mut commands: Commands,
    device: Res<RenderDevice>,
    lighting: Option<Res<crate::config::LightingSettings>>,
    views: Query<(Entity, &ExtractedView, &PlanetDraw, Option<&ReflectTargets>)>,
) {
    let scale = lighting.map_or(1.0, |l| l.reflect_scale());
    for (entity, view, draw, existing) in &views {
        if !draw.split {
            if existing.is_some() {
                commands.entity(entity).remove::<ReflectTargets>();
            }
            continue;
        }
        let size = view.viewport.zw();
        if existing.is_some_and(|t| t.size == size && t.scale == scale) {
            continue;
        }
        let half = (size.as_vec2() * scale).ceil().as_uvec2().max(UVec2::ONE);
        let sampled = TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING;
        commands.entity(entity).insert(ReflectTargets {
            size,
            scale,
            half,
            request: target(
                &device,
                "reflect request",
                size,
                SAMPLES,
                REQUEST_FORMAT,
                sampled,
            ),
            replaced_ms: target(
                &device,
                "reflect replaced (MSAA)",
                size,
                SAMPLES,
                REPLACED_FORMAT,
                TextureUsages::RENDER_ATTACHMENT,
            ),
            replaced: target(
                &device,
                "reflect replaced",
                size,
                1,
                REPLACED_FORMAT,
                sampled,
            ),
            hit: target(&device, "reflect hit", half, 1, HALF_FORMAT, sampled),
            ray: target(&device, "reflect ray", half, 1, HALF_FORMAT, sampled),
        });
    }
}

#[derive(Default)]
struct PlanetTerrainNode;

impl ViewNode for PlanetTerrainNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static ViewDepthTexture,
        &'static PlanetViewGpu,
        &'static PlanetDraw,
        Option<&'static ReflectTargets>,
    );

    fn run<'w>(
        &self,
        _: &mut RenderGraphContext,
        ctx: &mut RenderContext<'w>,
        (view_target, depth, view, draw, targets): QueryItem<'w, '_, Self::ViewQuery>,
        world: &'w World,
    ) -> Result<(), NodeRunError> {
        let cache = world.resource::<PipelineCache>();
        let (Some(pipeline), Some(maps), Some(shadows)) = (
            cache.get_render_pipeline(draw.pipeline),
            world.get_resource::<weather_maps::WeatherMapGpu>(),
            world.get_resource::<shadow::ShadowState>(),
        ) else {
            return Ok(());
        };
        let Some(receiver) = shadows.receiver.as_ref() else {
            return Ok(());
        };
        let split = match (draw.split, targets) {
            (false, _) => None,
            (true, Some(targets)) => Some(targets),
            // The targets are made in this frame's prepare; a split pipeline
            // without them has nowhere to write.
            (true, None) => return Ok(()),
        };
        let diagnostics = ctx.diagnostic_recorder();
        let clear = |view: &'w TextureView, resolve: Option<&'w TextureView>| {
            Some(RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: resolve.map(|r| &**r),
                ops: Operations {
                    load: LoadOp::Clear(LinearRgba::NONE.into()),
                    store: StoreOp::Store,
                },
            })
        };
        {
            let mut colors = vec![Some(view_target.get_color_attachment())];
            if let Some(t) = split {
                colors.push(clear(&t.request, None));
                colors.push(clear(&t.replaced_ms, Some(&t.replaced)));
            }
            let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("planet terrain"),
                color_attachments: &colors,
                depth_stencil_attachment: Some(depth.get_attachment(StoreOp::Store)),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            let span = diagnostics.pass_span(&mut pass, "planet_terrain");
            pass.set_render_pipeline(pipeline);
            super::draw_terrain(&mut pass, view, maps, receiver);
            span.end(&mut pass);
        }
        let Some(t) = split else {
            return Ok(());
        };
        let Some(pipelines) = world.get_resource::<ReflectPipelines>() else {
            return Ok(());
        };
        let (Some(gather), Some(compose)) = (
            cache.get_render_pipeline(pipelines.gather),
            cache.get_render_pipeline(pipelines.compose),
        ) else {
            return Ok(());
        };
        // Clip back to the body frame, less the eye: inverted in f64 from the
        // very matrix the terrain was drawn with, so a pixel lands where its
        // fragment stood, and kept small, so f32 holds it.
        let body_from_clip = DMat4::from_translation(-view.last_camera.as_dvec3())
            * view.last_clip.as_dmat4().inverse();
        let mut bytes = encase::UniformBuffer::new(Vec::<u8>::new());
        bytes
            .write(&ReflectViewUniform {
                body_from_clip: body_from_clip.as_mat4(),
                eye: view.last_camera.extend(t.scale),
                size: Vec4::new(
                    t.size.x as f32,
                    t.size.y as f32,
                    t.half.x as f32,
                    t.half.y as f32,
                ),
            })
            .expect("the reflections' uniform encodes");
        let device = ctx.render_device().clone();
        let uniform = device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("reflect view uniform"),
            contents: bytes.as_ref(),
            usage: BufferUsages::UNIFORM,
        });
        let gather_group = device.create_bind_group(
            Some("reflect gather"),
            &cache.get_bind_group_layout(&pipelines.gather_layout),
            &BindGroupEntries::sequential((&t.request, depth.view(), uniform.as_entire_binding())),
        );
        let compose_group = device.create_bind_group(
            Some("reflect compose"),
            &cache.get_bind_group_layout(&pipelines.compose_layout),
            &BindGroupEntries::sequential((
                &t.request,
                depth.view(),
                &t.replaced,
                &t.hit,
                &t.ray,
                uniform.as_entire_binding(),
            )),
        );
        {
            let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
                label: Some("reflect half"),
                color_attachments: &[clear(&t.hit, None), clear(&t.ray, None)],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            let span = diagnostics.pass_span(&mut pass, "reflect_half");
            pass.set_render_pipeline(gather);
            pass.set_bind_group(0, &view.draw_bind_group, &[]);
            pass.set_bind_group(1, &maps.bind_group, &[]);
            pass.set_bind_group(2, receiver, &[]);
            pass.set_bind_group(3, &gather_group, &[]);
            pass.draw(0..3, 0..1);
            span.end(&mut pass);
        }
        let mut pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("reflect compose"),
            color_attachments: &[Some(view_target.get_color_attachment())],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        let span = diagnostics.pass_span(&mut pass, "reflect_compose");
        pass.set_render_pipeline(compose);
        pass.set_bind_group(0, &compose_group, &[]);
        pass.draw(0..3, 0..1);
        span.end(&mut pass);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The uniform both half-size passes read is laid out as each shader
    /// declares it, and the two shaders agree on what marks a request.
    #[test]
    fn the_passes_read_the_uniform_and_the_request_as_written() {
        let rust = ReflectViewUniform::min_size().get() as u32;
        let surface = crate::shader_tests::planet_surface_source();
        let compose = crate::shader_tests::reflect_compose_source();
        for (label, source) in [
            ("planet_surface.wgsl", surface.as_str()),
            ("reflect_compose.wgsl", compose.as_str()),
        ] {
            assert_eq!(
                crate::shader_tests::wgsl_struct_size(label, source, "ReflectView"),
                rust,
                "{label}'s ReflectView"
            );
            assert!(
                source.contains("const REQUEST_FLAG: u32 = 0x8000u;"),
                "{label} marks a request with the top bit of the slot's upper half"
            );
        }
    }

    /// The scale is validated, and one keeps today's inline traces.
    #[test]
    fn a_full_scale_traces_inline() {
        use crate::config::Validated;
        let mut lighting = crate::config::LightingSettings::default();
        assert_eq!(lighting.reflect_render_scale, 0.5);
        lighting.reflect_render_scale = 1.0;
        assert!(lighting.validate().is_ok());
        assert!(!lighting.reflect_split_on());
        lighting.reflect_render_scale = 0.1;
        assert!(lighting.validate().is_err());
    }
}
