// `lighting-model` 3.3: a cheap bloom (`bloom.rs`). The HDR image is taken
// straight to a quarter of its size through a soft threshold (Karis-weighted,
// so one hot pixel does not flicker), then down and back up a short chain with
// the dual filter (Marius Bjorge's "dual Kawase": five taps down, eight up),
// and added onto the image at the bloom's strength (the blend constant).
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct LiteBloom {
    // x threshold, y its soft knee, z one texel of the source in u, w in v
    look: vec4<f32>,
}
@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> bloom: LiteBloom;

fn luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3(0.2126, 0.7152, 0.0722));
}

fn read(uv: vec2<f32>) -> vec3<f32> {
    return max(textureSampleLevel(source, source_sampler, uv, 0.0).rgb, vec3(0.0));
}

// What of a colour is over the threshold, with a quadratic knee.
fn bright(c: vec3<f32>) -> vec3<f32> {
    let l = luma(c);
    let knee = max(bloom.look.y, 1e-4);
    let soft = clamp(l - bloom.look.x + knee, 0.0, 2.0*knee);
    let over = max(soft*soft/(4.0*knee), l - bloom.look.x);
    return c*over/max(l, 1e-4);
}

// Full size to a quarter: four bilinear taps, each the mean of a 2 x 2,
// cover the 4 x 4 a quarter texel stands for; each weighed by 1/(1 + luma).
@fragment
fn down_first(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let t = bloom.look.zw;
    var sum = vec3(0.0);
    var weight = 0.0;
    for (var k = 0u; k < 4u; k++) {
        let o = vec2(select(-1.0, 1.0, (k & 1u) == 1u), select(-1.0, 1.0, k >= 2u));
        let c = bright(read(in.uv + o*t));
        let w = 1.0/(1.0 + luma(c));
        sum += c*w;
        weight += w;
    }
    return vec4(sum/max(weight, 1e-4), 1.0);
}

@fragment
fn down(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let t = bloom.look.zw*0.5;
    var c = read(in.uv)*4.0;
    c += read(in.uv + vec2(-t.x, -t.y));
    c += read(in.uv + vec2(t.x, -t.y));
    c += read(in.uv + vec2(-t.x, t.y));
    c += read(in.uv + vec2(t.x, t.y));
    return vec4(c/8.0, 1.0);
}

@fragment
fn up(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let t = bloom.look.zw;
    var c = read(in.uv + vec2(-2.0*t.x, 0.0));
    c += read(in.uv + vec2(2.0*t.x, 0.0));
    c += read(in.uv + vec2(0.0, -2.0*t.y));
    c += read(in.uv + vec2(0.0, 2.0*t.y));
    c += read(in.uv + vec2(-t.x, -t.y))*2.0;
    c += read(in.uv + vec2(t.x, -t.y))*2.0;
    c += read(in.uv + vec2(-t.x, t.y))*2.0;
    c += read(in.uv + vec2(t.x, t.y))*2.0;
    return vec4(c/12.0, 1.0);
}
