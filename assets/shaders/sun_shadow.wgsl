// The sun's cascaded shadows (`sun-shadows` decisions 2 and 4), read by the
// terrain (`planet_surface.wgsl`) and by what the field lights
// (`field_lit.wgsl`) through this one function, so a house and the lane it
// shades take the same shadow from the same cascades.
//
// The cascades are fitted in `pbd_core::shadow` and drawn by
// `planet_shadow.rs`. This module declares no binding: each shader binds the
// map, its comparison sampler and the cascades in its own group and hands
// them in.
#define_import_path pbd::sun_shadow

// `planet_shadow::GpuCascades`, in the planet's frame.
struct SunCascades {
    // Planet frame to each cascade's clip space: x and y -1..1 across its box,
    // depth 0 at the near plane (toward the sun) to 1 at the far.
    clip_from_body: array<mat4x4<f32>, 4>,
    // Per cascade: a texel's width on the ground, metres.
    texel_m: vec4<f32>,
    // Per cascade: metres from its near plane to its far, one unit of depth.
    depth_m: vec4<f32>,
    // x one while the cascades are drawn (zero: no shadows, all lit); y the
    // normal offset and z the depth bias, both in texels of the cascade in
    // use; w the share of a box across which it blends into the next.
    settings: vec4<f32>,
}

// A 3 x 3 kernel of comparison taps round `uv`, each a bilinear 2 x 2: an edge
// a texel and a half soft.
fn sun_shadow_taps(map: texture_depth_2d_array, cmp: sampler_comparison,
                   uv: vec2<f32>, layer: u32, depth: f32) -> f32 {
    let texel = 1.0 / vec2<f32>(textureDimensions(map));
    var lit = 0.0;
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            lit += textureSampleCompareLevel(map, cmp, uv + vec2<f32>(f32(x), f32(y)) * texel,
                layer, depth);
        }
    }
    return lit / 9.0;
}

// How much of the sun cascade `i` lets reach `position`, 0..1, and where the
// point stands in its box, the larger of |x| and |y| (over one: outside it).
// `clip_from_body`, `texel_m` and `depth_m` are the cascade's.
fn sun_shadow_in(map: texture_depth_2d_array, cmp: sampler_comparison, settings: vec4<f32>,
                 i: u32, clip_from_body: mat4x4<f32>, texel_m: f32, depth_m: f32,
                 position: vec3<f32>, normal: vec3<f32>) -> vec2<f32> {
    let lifted = position + normal * (settings.y * texel_m);
    let clip = clip_from_body * vec4<f32>(lifted, 1.0);
    let edge = max(abs(clip.x), abs(clip.y));
    if edge >= 1.0 || clip.z < 0.0 || clip.z > 1.0 {
        return vec2<f32>(1.0, 2.0);
    }
    let uv = vec2<f32>(clip.x * 0.5 + 0.5, 0.5 - clip.y * 0.5);
    let depth = clip.z - settings.z * texel_m / depth_m;
    return vec2<f32>(sun_shadow_taps(map, cmp, uv, i, depth), edge);
}

// How much of the sun reaches `position` (the planet's frame) on a face
// turned to `normal`: one in full sun, zero in shadow. The finest cascade
// whose box holds the point answers, blended into the next over the last
// `settings.w` of its box, and into full sun past the last one's.
fn sun_shadow(map: texture_depth_2d_array, cmp: sampler_comparison, cascades: SunCascades,
              position: vec3<f32>, normal: vec3<f32>) -> f32 {
    // A copy in function memory, so a cascade can be picked by a runtime index.
    var c = cascades;
    if c.settings.x < 0.5 {
        return 1.0;
    }
    let band = max(c.settings.w, 1e-3);
    for (var i = 0u; i < 4u; i++) {
        let here = sun_shadow_in(map, cmp, c.settings, i, c.clip_from_body[i], c.texel_m[i],
            c.depth_m[i], position, normal);
        if here.y >= 1.0 {
            continue;
        }
        let t = clamp((here.y - (1.0 - band)) / band, 0.0, 1.0);
        if t <= 0.0 {
            return here.x;
        }
        var next = 1.0;
        if i < 3u {
            let j = i + 1u;
            let there = sun_shadow_in(map, cmp, c.settings, j, c.clip_from_body[j], c.texel_m[j],
                c.depth_m[j], position, normal);
            if there.y >= 1.0 {
                return here.x;
            }
            next = there.x;
        }
        return mix(here.x, next, t);
    }
    return 1.0;
}
