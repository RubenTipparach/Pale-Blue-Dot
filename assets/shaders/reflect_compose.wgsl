// `lighting-perf` 2.8: L8's half-size hits laid back over the terrain.
//
// The terrain (`fragment_split` in `planet_surface.wgsl`) drew each wet pixel
// as if its reflected ray hit nothing, and wrote beside it what a hit would
// replace: `k*B` and `k*fr`, through the haze. `reflect_gather` traced one
// pixel in each 2 x 2. Here, per pixel that asked, the four half-size texels
// round it are weighed bilinearly, by how near their distance is to the
// pixel's (as the rain's compose) and by how near their ray is to the
// pixel's, so a wall's hit does not cross to the floor at its foot; with
// `hH` and `h` the weighed means of `hit.rgb*hit.a` and `hit.a`, the pass
// ADDS `k*fr*hH - h*k*B` to the view (additive blend, every sample), which is
// what the inline trace added over the miss, term for term.

#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

struct ReflectView {
    body_from_clip: mat4x4<f32>,
    eye: vec4<f32>,
    size: vec4<f32>,
}
@group(0) @binding(0) var reflect_request: texture_multisampled_2d<u32>;
@group(0) @binding(1) var reflect_depth: texture_depth_multisampled_2d;
@group(0) @binding(2) var replaced: texture_2d<f32>;
@group(0) @binding(3) var hit_tex: texture_2d<f32>;
@group(0) @binding(4) var ray_tex: texture_2d<f32>;
@group(0) @binding(5) var<uniform> reflect_view: ReflectView;

const REQUEST_FLAG: u32 = 0x8000u;

fn oct_decode(e: vec2<u32>) -> vec3<f32> {
    let p = vec2<f32>(e)/65535.0*2.0 - 1.0;
    var v = vec3(p, 1.0 - abs(p.x) - abs(p.y));
    if v.z < 0.0 {
        v = vec3((1.0 - abs(v.yx))*select(vec2(-1.0), vec2(1.0), v.xy >= vec2(0.0)), v.z);
    }
    return normalize(v);
}

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let px = vec2<i32>(in.position.xy);
    let req = textureLoad(reflect_request, px, 0);
    if (req.y & REQUEST_FLAG) == 0u { discard; }
    let r = oct_decode(req.zw);
    let depth = textureLoad(reflect_depth, px, 0);
    let ndc = vec2((f32(px.x) + 0.5)/reflect_view.size.x*2.0 - 1.0,
                   1.0 - (f32(px.y) + 0.5)/reflect_view.size.y*2.0);
    let h4 = reflect_view.body_from_clip*vec4(ndc, depth, 1.0);
    let d = length(h4.xyz/h4.w);
    let half_size = vec2<i32>(reflect_view.size.zw);
    let at = in.position.xy*reflect_view.eye.w - 0.5;
    let i0 = vec2<i32>(floor(at));
    let f = at - floor(at);
    var sum_w = 0.0;
    var sum_h = 0.0;
    var sum_hh = vec3(0.0);
    for (var k = 0; k < 4; k++) {
        let o = vec2<i32>(k & 1, k >> 1u);
        let t = clamp(i0 + o, vec2<i32>(0), half_size - 1);
        let ray = textureLoad(ray_tex, t, 0);
        if ray.w <= 0.0 { continue; }
        let wb = select(1.0 - f.x, f.x, o.x == 1)*select(1.0 - f.y, f.y, o.y == 1);
        let wd = exp(-abs(ray.w - d)/(0.02*d + 0.1));
        let wr = smoothstep(0.6, 0.95, dot(ray.xyz, r));
        let w = max(wb, 0.01)*wd*wr;
        if w < 1e-4 { continue; }
        let hit = textureLoad(hit_tex, t, 0);
        sum_w += w;
        sum_h += w*hit.a;
        sum_hh += w*hit.rgb*hit.a;
    }
    if sum_w <= 0.0 { discard; }
    let rep = textureLoad(replaced, px, 0);
    let delta = rep.a*sum_hh/sum_w - (sum_h/sum_w)*rep.rgb;
    return vec4(delta, 0.0);
}
