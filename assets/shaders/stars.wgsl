// The stars, sized in screen pixels (`steady-stars`). Each star is six
// vertices at its centre; UV_0 is the corner in pixels (-h..h, h three times
// the star's Gaussian width), COLOR its tint times its intensity. A quad
// sized in metres was a third of a pixel for most stars and winked on and
// off as the sky turned; sized in pixels it covers the same pixels at any
// resolution or render scale, and its light is carried in the intensity.
#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_world}
#import bevy_pbr::view_transformations::position_world_to_clip
#import bevy_pbr::mesh_view_bindings::view

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) corner: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    // The corner in pixels from the star's centre, and its Gaussian width.
    @location(0) at: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) sigma: f32,
}

@vertex
fn vertex(v: Vertex) -> VertexOutput {
    let world_from_local = get_world_from_local(v.instance_index);
    let world = mesh_position_local_to_world(world_from_local, vec4<f32>(v.position, 1.0));
    var clip = position_world_to_clip(world.xyz);
    clip = vec4<f32>(clip.xy + v.corner*2.0/view.viewport.zw*clip.w, clip.zw);
    var out: VertexOutput;
    out.clip_position = clip;
    out.at = v.corner;
    out.sigma = max(abs(v.corner.x)/3.0, 1e-3);
    out.color = v.color;
    return out;
}

// A Gaussian at least 0.8 px wide: sampled at pixel centres it sums to the
// same light wherever between them the star falls, so a star moving by a
// fraction of a pixel a frame keeps its brightness.
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let fall = exp(-dot(in.at, in.at)/(2.0*in.sigma*in.sigma));
    return vec4<f32>(in.color.rgb*fall, 1.0);
}
