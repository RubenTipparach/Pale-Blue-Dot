// A lit thing the terrain pass does not draw - a town, the ship, a fish
// school, the float - taking the terrain's light field (`lamps-and-lanterns`
// decision 10) and the sun's shadow (`sun-shadows` decision 4).
//
// Two looks, by `field.look.x`:
// - **As the terrain is lit** (a town's pieces, `sun-shadows` decision 7):
//   the albedo in the terrain's own fill and sun, with the terrain's numbers,
//   so a house wall and the cliff beside it are one light. The fill is the
//   sky's share where the thing stands: the field's sky times `look.y`, one
//   outdoors and a room's own share indoors.
// - **Bevy's PBR picture** (a craft, a fish), kept where the sun reaches and
//   giving way to the albedo in the terrain's fill where it does not.
//
// Either way the sun reaches only where it is up, the sky reaches, and the
// cascades see it, and the lamps' warm light is added, both read off the
// field at the eight corners of the mesh's bounds and blended across them.
// The constants are the terrain shader's, and `the_field_lit_shader_carries_
// the_terrain_light_constants` holds both copies to `pbd_core::light`.

#import bevy_pbr::{
    pbr_functions::alpha_discard,
    pbr_fragment::pbr_input_from_standard_material,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#import pbd::sun_shadow::{SunCascades, sun_shadow}
#endif

struct Field {
    // The mesh's bounds in the render frame.
    low: vec4<f32>,
    high: vec4<f32>,
    // The planet's centre in the render frame, and the way to the sun.
    centre: vec4<f32>,
    sun: vec4<f32>,
    // The field at the bounds' eight corners, corner `x + 2y + 4z`: sky in
    // `sky[i / 4][i % 4]`, block in `block[i / 4][i % 4]`, each 0..1.
    sky: array<vec4<f32>, 2>,
    block: array<vec4<f32>, 2>,
    // x one where it is lit as the terrain is; y the share of the sky that
    // reaches it (one outdoors); z the sun bounced round a room; w how many
    // room lights.
    look: vec4<f32>,
    // A building's own lights (`cities-in-the-world` decision 7a): where, in
    // the planet's frame, and how far; colour times power, w one for a
    // candle; the band of height round each that it lights.
    lights: array<vec4<f32>, 24>,
    light_colour: array<vec4<f32>, 24>,
    light_span: array<vec4<f32>, 24>,
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<uniform> field: Field;
#ifndef PREPASS_PIPELINE
@group(#{MATERIAL_BIND_GROUP}) @binding(101) var sun_map: texture_depth_2d_array;
@group(#{MATERIAL_BIND_GROUP}) @binding(102) var sun_cmp: sampler_comparison;
@group(#{MATERIAL_BIND_GROUP}) @binding(103) var<storage, read> sun_cascades: SunCascades;
#endif

const AMBIENT_FLOOR: f32 = 0.05;
const NIGHT_FILL: f32 = 0.12;
const SKY_FILL: vec3<f32> = vec3<f32>(0.16, 0.21, 0.27);
const TORCH_TINT: vec3<f32> = vec3<f32>(1.00, 0.70, 0.30);
const TORCH_GAIN: f32 = 1.25;
// The terrain's sun, and the paler fill it gives a wall (`planet_surface.wgsl`,
// kinds 1 and 4): a wall faces the whole sky's edge, a cap only its top.
const SUN_TINT: vec3<f32> = vec3<f32>(1.12, 1.03, 0.87);
const WALL_FILL: vec3<f32> = vec3<f32>(0.30, 0.32, 0.34);
const WALL_NIGHT: f32 = 0.20;
const WALL_GAIN: f32 = 0.95;
// A room's own light: the sun that came in by its door and windows and
// bounced round it off the floor and the walls, warm, as much of it as the
// room's share of the sky lets in and the sun's height puts on the ground
// (`sun-shadows` decision 7). Times `look.z`.
const ROOM_BOUNCE: vec3<f32> = vec3<f32>(0.30, 0.24, 0.16);
// How a building's own light weighs against the sky's fill: the mockup's
// own, as its block light is added to its hemisphere's. At half, a room at
// 22:30 stayed in the deep orange under the tonemapper's shoulder where the
// mockup's is a warm tan (`docs/screenshots/sun-shadows`).
const ROOM_LIGHT_GAIN: f32 = 1.0;

// What a building's own fires and candles lay on a room face at `body`
// facing `n` (`cities-in-the-world` decision 7a, the mockup's `blockLighter`):
// each within its reach and its storey, facing it, the fires burning all day
// and brighter at night, the candles only by night.
fn room_lights(body: vec3<f32>, n: vec3<f32>, night: f32) -> vec3<f32> {
    let fire = 0.9 + 0.3 * night;
    let candle = 1.8 * clamp((night - 0.25) / 0.35, 0.0, 1.0);
    var sum = vec3<f32>(0.0);
    let count = min(u32(field.look.w), 24u);
    for (var i = 0u; i < count; i++) {
        let at = field.lights[i];
        let to = at.xyz - body;
        let d = length(to);
        if d >= at.w {
            continue;
        }
        let rise = dot(body - at.xyz, normalize(at.xyz));
        let span = field.light_span[i];
        if rise < span.x || rise > span.y {
            continue;
        }
        let facing = dot(to, n) / max(d, 1e-3);
        if facing < -0.15 {
            continue;
        }
        let q = 1.0 - (d / at.w) * (d / at.w);
        let f = q * q * (0.3 + 0.7 * max(facing, 0.0)) * d * d / (d * d + 0.36);
        let c = field.light_colour[i];
        sum += c.rgb * (f * select(fire, candle, c.w > 0.5));
    }
    return sum * ROOM_LIGHT_GAIN;
}

fn lamp_strength(level: f32) -> f32 {
    let f = clamp(level, 0.0, 1.0);
    let g = f * (2.0 - f);
    return g * g;
}

// One channel at a point in the bounds, blended across the eight corners.
fn blend(corners: array<vec4<f32>, 2>, t: vec3<f32>) -> f32 {
    let x00 = mix(corners[0].x, corners[0].y, t.x);
    let x10 = mix(corners[0].z, corners[0].w, t.x);
    let x01 = mix(corners[1].x, corners[1].y, t.x);
    let x11 = mix(corners[1].z, corners[1].w, t.x);
    return mix(mix(x00, x10, t.y), mix(x01, x11, t.y), t.z);
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) is_front: bool) -> FragmentOutput {
    var pbr_input = pbr_input_from_standard_material(in, is_front);
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);
#ifdef PREPASS_PIPELINE
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    let span = max(field.high.xyz - field.low.xyz, vec3<f32>(1e-3));
    let t = clamp((in.world_position.xyz - field.low.xyz) / span, vec3<f32>(0.0), vec3<f32>(1.0));
    let sky = blend(field.sky, t);
    let block = blend(field.block, t);
    let body = in.world_position.xyz - field.centre.xyz;
    let up = normalize(body);
    let sun = field.sun.xyz;
    let elevation = dot(up, sun);
    let daylight = smoothstep(-0.13, 0.20, elevation);
    // The sun ends at the horizon (`sun-shadows` decision 5): a shadow cast
    // by a sun under it would say something false.
    let sunlight = smoothstep(-0.0145, 0.02, elevation);
    let n = pbr_input.N;
    let facing = max(dot(n, sun), 0.0);
    var shadow = 1.0;
    if facing > 0.0 && sunlight > 0.0 {
        shadow = sun_shadow(sun_map, sun_cmp, sun_cascades, body, pbr_input.world_normal);
    }
    // The sun where it is up, the sky reaches and the cascades see it.
    let sun_up = sunlight * sky * shadow;
    let base = pbr_input.material.base_color.rgb;
    let lamp = base * TORCH_TINT * (lamp_strength(block) * TORCH_GAIN);
    if field.look.x > 0.5 {
        // As the terrain is lit: a cap's cool fill on what faces up, a wall's
        // paler one on what faces across or down.
        let upness = clamp(dot(n, up), 0.0, 1.0);
        let fill = mix(WALL_FILL, SKY_FILL, upness);
        let night = mix(WALL_NIGHT, NIGHT_FILL, upness);
        let gain = mix(WALL_GAIN, 1.0, upness);
        let open = sky * field.look.y;
        let bounce = ROOM_BOUNCE * (field.look.z * open * sunlight * max(elevation, 0.0));
        let lit = base * (fill * max(AMBIENT_FLOOR, mix(night, 1.0, daylight) * open)
            + SUN_TINT * facing * sun_up + bounce) * gain;
        let own = base * room_lights(body, n, 1.0 - daylight);
        out.color = vec4<f32>(lit + lamp + own, pbr_input.material.base_color.a);
    } else {
        out.color = apply_pbr_lighting(pbr_input);
        // Bevy's sun has no shadows and lights whatever faces it, from under
        // the horizon at night and through the rock over a cave. So its
        // picture is kept only as far as the sun reaches, as the terrain's
        // direct term is, and the rest is the albedo in the terrain's fill,
        // whose floor keeps a cave legible (`lamps-and-lanterns` decision 14).
        let ambient = base * SKY_FILL * max(AMBIENT_FLOOR, mix(NIGHT_FILL, 1.0, daylight) * sky);
        out.color = vec4<f32>(out.color.rgb * sun_up + ambient * (1.0 - sun_up) + lamp, out.color.a);
    }
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif
    return out;
}
