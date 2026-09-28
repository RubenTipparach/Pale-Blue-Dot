// The world map's live layers (`world-map` decisions 4 and 10): the chosen
// weather overlay, the clouds and rain, and the night side, drawn over the
// base map's image nodes by one UI material.
//
// Each pixel is turned back into a body-local direction by the map's own
// projection, `unproject` below, which is `pbd_core::geo::unproject` written
// again because WGSL cannot call Rust; `the_map_shader_projects_as_geo_does`
// (`map_gpu_tests.rs`) runs this one on a GPU adapter and holds the two
// together, and the night's `daylight` with `Clock::daylight` the same way. The layers are
// then looked up at that direction, so they move because their inputs move
// and nothing is rebuilt while the map is open.

#import bevy_ui::ui_vertex_output::UiVertexOutput

struct MapLive {
    // The view: the map position at the node's centre (u, v), and how much of
    // the map the node spans (u across, v down).
    view: vec4<f32>,
    // The sun's body-local direction; w 1 to draw the night side.
    sun: vec4<f32>,
    // x 1 to draw the clouds and rain, y 1 to draw the overlay.
    layers: vec4<f32>,
};

@group(1) @binding(0) var<uniform> map: MapLive;
// Cover in r, rain in g, snow in b, 0..1 each: the weather maps the globe
// draws, sampled onto the map's own cube.
@group(1) @binding(1) var cloud_map: texture_cube<f32>;
@group(1) @binding(2) var cloud_sampler: sampler;
// The overlay already in its ramp's colour, and how much of it to draw in a.
@group(1) @binding(3) var overlay_map: texture_cube<f32>;
@group(1) @binding(4) var overlay_sampler: sampler;

// map_live:shared begin: what `map_gpu_tests` runs on a GPU adapter against
// the Rust, so it must stand alone, with no binding in it.
const PI: f32 = 3.14159265358979;
const TAU: f32 = 6.28318530717959;

// The direction at an equirectangular map position: `geo::unproject`. `u`
// wraps round the antimeridian and `v` is held to the poles.
fn unproject(uv: vec2<f32>) -> vec3<f32> {
    let u = uv.x - floor(uv.x);
    let v = clamp(uv.y, 0.0, 1.0);
    let lat = (0.5 - v) * PI;
    let lon = (u - 0.5) * TAU;
    return vec3<f32>(cos(lat) * cos(lon), sin(lat), cos(lat) * sin(lon));
}

// How much it is day at a place: `Clock::daylight`, the terrain shader's
// `smoothstep(-0.13, 0.20, sun_elevation)`, so the map's dusk is the ground's.
const DAY_LOW: f32 = -0.13;
const DAY_HIGH: f32 = 0.20;

fn daylight(sun: vec3<f32>, up: vec3<f32>) -> f32 {
    return smoothstep(DAY_LOW, DAY_HIGH, dot(sun, up));
}
// map_live:shared end

// The night: a deep blue-black laid over the land, dark enough that the
// map still reads through it. The layers blend in linear light, where 0.72
// left the night at 55% of the day's brightness to the eye (measured on the
// first capture); 0.86 leaves it near a third.
const NIGHT: vec3<f32> = vec3<f32>(0.004, 0.007, 0.03);
const NIGHT_ALPHA: f32 = 0.86;
// Cloud white, rain blue and snow's pale violet, each at most this opaque.
const CLOUD: vec3<f32> = vec3<f32>(0.85, 0.87, 0.9);
const CLOUD_ALPHA: f32 = 0.8;
const RAIN: vec3<f32> = vec3<f32>(0.05, 0.16, 0.8);
const SNOW: vec3<f32> = vec3<f32>(0.7, 0.62, 1.0);
const FALL_ALPHA: f32 = 0.55;

// Layers go on bottom to top over whatever the base shows: after each, the
// picture is `premultiplied + keep * base`.
struct Stack {
    premultiplied: vec3<f32>,
    keep: f32,
};

fn over(stack: Stack, colour: vec3<f32>, alpha: f32) -> Stack {
    let a = clamp(alpha, 0.0, 1.0);
    return Stack(colour * a + stack.premultiplied * (1.0 - a), stack.keep * (1.0 - a));
}

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let uv = map.view.xy + (in.uv - vec2<f32>(0.5)) * map.view.zw;
    if (uv.y < 0.0 || uv.y > 1.0) {
        return vec4<f32>(0.0);
    }
    let up = unproject(uv);
    var stack = Stack(vec3<f32>(0.0), 1.0);
    if (map.layers.y > 0.5) {
        let overlay = textureSampleLevel(overlay_map, overlay_sampler, up, 0.0);
        stack = over(stack, overlay.rgb, overlay.a);
    }
    if (map.layers.x > 0.5) {
        let weather = textureSampleLevel(cloud_map, cloud_sampler, up, 0.0);
        stack = over(stack, CLOUD, CLOUD_ALPHA * smoothstep(0.1, 0.9, weather.r));
        stack = over(stack, RAIN, FALL_ALPHA * weather.g);
        stack = over(stack, SNOW, FALL_ALPHA * weather.b);
    }
    if (map.sun.w > 0.5) {
        stack = over(stack, NIGHT, NIGHT_ALPHA * (1.0 - daylight(map.sun.xyz, up)));
    }
    let alpha = 1.0 - stack.keep;
    if (alpha < 1e-4) {
        return vec4<f32>(0.0);
    }
    return vec4<f32>(stack.premultiplied / alpha, alpha);
}
