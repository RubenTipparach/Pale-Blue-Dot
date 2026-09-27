// A lit thing the terrain pass does not draw - the ship, a fish school, the
// float - taking the terrain's light field (`lamps-and-lanterns` decision 10).
//
// Bevy's own PBR lighting runs first and is kept: by day in the open the
// colour is Bevy's, unchanged. It is then scaled by the sky's fill where the
// fragment is, and the lamps' warm light is added, both read off the field
// at the eight corners of the mesh's bounds and blended across them, so a
// ship half out of a cave mouth is lit at its bow and dark at its stern. The
// constants are the terrain shader's, and `the_field_lit_shader_carries_the_
// terrain_light_constants` holds both copies to `pbd_core::light`.

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
}

@group(#{MATERIAL_BIND_GROUP}) @binding(100) var<uniform> field: Field;

const AMBIENT_FLOOR: f32 = 0.05;
const NIGHT_FILL: f32 = 0.12;
const TORCH_TINT: vec3<f32> = vec3<f32>(1.00, 0.70, 0.30);
const TORCH_GAIN: f32 = 1.25;

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
    out.color = apply_pbr_lighting(pbr_input);
    let span = max(field.high.xyz - field.low.xyz, vec3<f32>(1e-3));
    let t = clamp((in.world_position.xyz - field.low.xyz) / span, vec3<f32>(0.0), vec3<f32>(1.0));
    let sky = blend(field.sky, t);
    let block = blend(field.block, t);
    let up = normalize(in.world_position.xyz - field.centre.xyz);
    let daylight = smoothstep(-0.13, 0.20, dot(up, field.sun.xyz));
    let fill = max(AMBIENT_FLOOR, mix(NIGHT_FILL, 1.0, daylight) * sky);
    let lamp = pbr_input.material.base_color.rgb * TORCH_TINT * (lamp_strength(block) * TORCH_GAIN);
    out.color = vec4<f32>(out.color.rgb * fill + lamp, out.color.a);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif
    return out;
}
