// `lighting-model` L6: the display transform, after Bevy's bloom on the HDR
// image. Exposure (metered on the CPU, `tone.rs`), the grade (a warm white
// balance at golden hour, saturation that eases at night and under cloud),
// Khronos PBR Neutral (the owner's pick; `pbd_core::sky_light::pbr_neutral`),
// and a dither so a dusk gradient does not band in eight bits.
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct ToneGrade {
    // x exposure (a multiplier), y saturation, z dither amount, w frame
    look: vec4<f32>,
    // rgb the white balance
    balance: vec4<f32>,
}
@group(0) @binding(0) var screen: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;
@group(0) @binding(2) var<uniform> grade: ToneGrade;

const NEUTRAL_START: f32 = 0.76;
const NEUTRAL_DESAT: f32 = 0.15;

fn pbr_neutral(color: vec3<f32>) -> vec3<f32> {
    let x = min(color.r, min(color.g, color.b));
    let offset = select(0.04, x - 6.25*x*x, x < 0.08);
    var c = color - offset;
    let peak = max(c.r, max(c.g, c.b));
    if peak < NEUTRAL_START { return c; }
    let d = 1.0 - NEUTRAL_START;
    let new_peak = 1.0 - d*d/(peak + d - NEUTRAL_START);
    c *= new_peak/peak;
    let g = 1.0 - 1.0/(NEUTRAL_DESAT*(peak - new_peak) + 1.0);
    return mix(c, vec3(new_peak), g);
}

fn to_srgb(c: vec3<f32>) -> vec3<f32> {
    return select(1.055*pow(c, vec3(1.0/2.4)) - 0.055, c*12.92, c <= vec3(0.0031308));
}
fn from_srgb(c: vec3<f32>) -> vec3<f32> {
    return select(pow((c + 0.055)/1.055, vec3(2.4)), c/12.92, c <= vec3(0.04045));
}
// Interleaved gradient noise, turned each frame: a fine even grain, half a
// step of eight bits either way.
fn ign(p: vec2<f32>) -> f32 {
    return fract(52.9829189*fract(dot(p, vec2(0.06711056, 0.00583715))));
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let src = textureSample(screen, screen_sampler, in.uv);
    var c = max(src.rgb, vec3(0.0))*grade.look.x*grade.balance.rgb;
    let l = dot(c, vec3(0.2126, 0.7152, 0.0722));
    c = max(vec3(l) + grade.look.y*(c - vec3(l)), vec3(0.0));
    c = clamp(pbr_neutral(c), vec3(0.0), vec3(1.0));
    let n = ign(in.position.xy + vec2(grade.look.w*5.588238, grade.look.w*3.31)) - 0.5;
    let s = clamp(to_srgb(c) + n*grade.look.z/255.0, vec3(0.0), vec3(1.0));
    return vec4(from_srgb(s), src.a);
}
