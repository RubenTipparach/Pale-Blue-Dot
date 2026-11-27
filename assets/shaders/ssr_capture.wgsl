// `lighting-model` L8 (10.2): what the screen trace marches through. After the
// main pass and the water's composite, the frame's colour and how far each
// pixel is from the eye go into a small texture (`planet_ssr.rs`); the next
// frame's wet ground reprojects its reflected ray into it with this frame's
// matrix. No second draw of the scene.
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct SsrCapture {
    // The inverse of the view's projection: clip to view space.
    view_from_clip: mat4x4<f32>,
}
@group(0) @binding(0) var scene: texture_2d<f32>;
@group(0) @binding(1) var scene_sampler: sampler;
#ifdef MULTISAMPLED
@group(0) @binding(2) var scene_depth: texture_depth_multisampled_2d;
#else
@group(0) @binding(2) var scene_depth: texture_depth_2d;
#endif
@group(0) @binding(3) var<uniform> capture: SsrCapture;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let size = vec2<f32>(textureDimensions(scene_depth));
    let coord = clamp(vec2<i32>(in.uv*size), vec2<i32>(0), vec2<i32>(size) - 1);
    let depth = textureLoad(scene_depth, coord, 0);
    // Colour from the same whole pixel as the distance (`ocean-environments`
    // B26): filtered, a texel took a thin bright thing's colour (a fishing
    // line at night) with the distance of what lay behind it, and every
    // reflected ray that met that distance came back the line's colour, a
    // scatter of glints in the water round it.
    let csize = vec2<f32>(textureDimensions(scene));
    let ccoord = clamp(vec2<i32>(in.uv*csize), vec2<i32>(0), vec2<i32>(csize) - 1);
    let colour = textureLoad(scene, ccoord, 0).rgb;
    // Reverse Z: nothing drawn (the sky) is zero, and stands for "too far to
    // hit", never a surface.
    if depth <= 1e-7 {
        return vec4(colour, 1e6);
    }
    let v = capture.view_from_clip*vec4(in.uv.x*2.0 - 1.0, 1.0 - in.uv.y*2.0, depth, 1.0);
    return vec4(colour, length(v.xyz/v.w));
}
