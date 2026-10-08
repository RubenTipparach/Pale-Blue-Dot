// The world map's base and its finer tiles (`world-map` decision 11): the
// raster as it was built, or greyed out beneath an overlay as the approved
// mockup greys it (survey M3). The mockup did it on a canvas, in sRGB: a
// "saturation" blend with grey, which leaves each pixel's luminosity
// 0.3 R + 0.59 G + 0.11 B, then 45% of (10, 16, 20) laid over the lot. This
// does the same to each texel, converting to sRGB and back because the
// texture is sampled in linear light. `the_map_greys_as_the_mockup_does`
// (`map_gpu_tests.rs`) runs the shared part on a GPU adapter against the
// mockup's arithmetic.

#import bevy_ui::ui_vertex_output::UiVertexOutput

@group(1) @binding(0) var image: texture_2d<f32>;
@group(1) @binding(1) var image_sampler: sampler;
// x 1 to grey the image out.
@group(1) @binding(2) var<uniform> grey: vec4<f32>;

// map_image:shared begin: what `map_gpu_tests` runs on a GPU adapter against
// the mockup's canvas arithmetic, so it must stand alone.
const LUMA: vec3<f32> = vec3<f32>(0.3, 0.59, 0.11);
const UNDER: vec3<f32> = vec3<f32>(10.0, 16.0, 20.0) / 255.0;
const UNDER_ALPHA: f32 = 0.45;

fn srgb_from_linear(c: vec3<f32>) -> vec3<f32> {
    let low = c * 12.92;
    let high = 1.055 * pow(max(c, vec3<f32>(0.0)), vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(high, low, c <= vec3<f32>(0.0031308));
}

fn linear_from_srgb(c: vec3<f32>) -> vec3<f32> {
    let low = c / 12.92;
    let high = pow((max(c, vec3<f32>(0.0)) + 0.055) / 1.055, vec3<f32>(2.4));
    return select(high, low, c <= vec3<f32>(0.04045));
}

// A linear colour greyed out as the mockup greys the planet under an overlay.
fn greyed(linear: vec3<f32>) -> vec3<f32> {
    let luminosity = dot(srgb_from_linear(linear), LUMA);
    let out = vec3<f32>(luminosity) * (1.0 - UNDER_ALPHA) + UNDER * UNDER_ALPHA;
    return linear_from_srgb(out);
}
// map_image:shared end

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    // A raster's first row is the frame's +Y pole, which a compass calls
    // south, and the map is drawn compass-north up (`compass-bar` decision
    // 7): each image is drawn turned top to bottom.
    let colour = textureSample(image, image_sampler, vec2<f32>(in.uv.x, 1.0 - in.uv.y));
    if (grey.x > 0.5) {
        return vec4<f32>(greyed(colour.rgb), colour.a);
    }
    return colour;
}
