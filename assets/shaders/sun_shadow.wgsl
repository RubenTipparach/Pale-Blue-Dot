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
// a texel and a half soft. `spacing` is in texels (one: today's).
fn sun_shadow_taps(map: texture_depth_2d_array, cmp: sampler_comparison,
                   uv: vec2<f32>, layer: u32, depth: f32, spacing: f32) -> f32 {
    let texel = spacing / vec2<f32>(textureDimensions(map));
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
    return vec2<f32>(sun_shadow_taps(map, cmp, uv, i, depth, 1.0), edge);
}

// `lighting-model` L3: the disc's taps, a Poisson set in the unit circle. The
// set is not turned per pixel or per cell: a turn per screen pixel swims as the
// camera moves, and one per texel cell printed blotches in the mockup.
const SUN_POISSON = array<vec2<f32>, 12>(
    vec2(-0.326, -0.406), vec2(-0.840, -0.074), vec2(-0.696, 0.457),
    vec2(-0.203, 0.621), vec2(0.962, -0.195), vec2(0.473, -0.480),
    vec2(0.519, 0.767), vec2(0.185, -0.893), vec2(0.507, 0.064),
    vec2(0.896, 0.412), vec2(-0.322, -0.933), vec2(-0.792, -0.598));
const SUN_SEARCH = array<vec2<f32>, 8>(
    vec2(0.0, 0.0), vec2(0.71, 0.0), vec2(-0.71, 0.0), vec2(0.0, 0.71),
    vec2(0.0, -0.71), vec2(0.62, 0.62), vec2(-0.62, -0.62), vec2(-0.62, 0.62));

// The same question as `sun_shadow_in`, answered with a shadow as wide as
// the sun makes it (PCSS lite): an 8-tap search finds how far above the
// receiver its blockers stand, the penumbra is that height times `spread`
// (the tangent of the sun's apparent radius), and 12 comparison taps filter
// a disc that wide, never under half a texel. Each tap's bias grows with its
// distance and with the light's slope on the face (`tan_l`), so a wide disc
// on a face turned from the sun does not shadow the face itself (acne).
fn sun_shadow_soft_in(map: texture_depth_2d_array, cmp: sampler_comparison, settings: vec4<f32>,
                      i: u32, clip_from_body: mat4x4<f32>, texel_m: f32, depth_m: f32,
                      position: vec3<f32>, normal: vec3<f32>, spread: f32, tan_l: f32) -> vec2<f32> {
    let lifted = position + normal * (settings.y * texel_m);
    let clip = clip_from_body * vec4<f32>(lifted, 1.0);
    let edge = max(abs(clip.x), abs(clip.y));
    if edge >= 1.0 || clip.z < 0.0 || clip.z > 1.0 {
        return vec2<f32>(1.0, 2.0);
    }
    let size = vec2<f32>(textureDimensions(map));
    let uv = vec2<f32>(clip.x * 0.5 + 0.5, 0.5 - clip.y * 0.5);
    let depth = clip.z - settings.z * texel_m / depth_m;
    // Blockers up to 16 m over the receiver are searched for, the disc they
    // would make, in texels: between two and twenty-four.
    let search = clamp(16.0 * spread / texel_m, 2.0, 24.0);
    var blocker = 0.0;
    var found = 0.0;
    for (var k = 0u; k < 8u; k++) {
        let at = vec2<i32>(clamp(uv * size + SUN_SEARCH[k] * search, vec2(0.0), size - 1.0));
        let d = textureLoad(map, at, i32(i), 0);
        if d < depth {
            blocker += d;
            found += 1.0;
        }
    }
    if found < 0.5 {
        return vec2<f32>(1.0, edge);
    }
    let height_m = max(depth - blocker / found, 0.0) * depth_m;
    let radius = clamp(height_m * spread / texel_m, 0.5, search);
    var lit = 0.0;
    for (var k = 0u; k < 12u; k++) {
        let o = SUN_POISSON[k] * radius;
        let bias = length(o) * texel_m * min(tan_l, 8.0) * 1.2 / depth_m;
        lit += textureSampleCompareLevel(map, cmp, uv + o / size, i, depth - bias);
    }
    return vec2<f32>(lit / 12.0, edge);
}

// One cascade's answer: soft in the two finest while `spread` is above zero,
// the 3 x 3 (a three-quarter texel apart) beyond them.
fn sun_shadow_pick(map: texture_depth_2d_array, cmp: sampler_comparison, c: SunCascades,
                   i: u32, position: vec3<f32>, normal: vec3<f32>, spread: f32, tan_l: f32) -> vec2<f32> {
    if spread > 0.0 && i < 2u {
        return sun_shadow_soft_in(map, cmp, c.settings, i, c.clip_from_body[i], c.texel_m[i],
            c.depth_m[i], position, normal, spread, tan_l);
    }
    if spread > 0.0 {
        let lifted = position + normal * (c.settings.y * c.texel_m[i]);
        let clip = c.clip_from_body[i] * vec4<f32>(lifted, 1.0);
        let edge = max(abs(clip.x), abs(clip.y));
        if edge >= 1.0 || clip.z < 0.0 || clip.z > 1.0 {
            return vec2<f32>(1.0, 2.0);
        }
        let uv = vec2<f32>(clip.x * 0.5 + 0.5, 0.5 - clip.y * 0.5);
        let depth = clip.z - c.settings.z * c.texel_m[i] / c.depth_m[i];
        return vec2<f32>(sun_shadow_taps(map, cmp, uv, i, depth, 0.75), edge);
    }
    return sun_shadow_in(map, cmp, c.settings, i, c.clip_from_body[i], c.texel_m[i],
        c.depth_m[i], position, normal);
}

// `sun_shadow` with the sun as wide as `spread` says (zero: today's filter),
// for a face whose normal makes `n_dot_l` with the sun.
fn sun_shadow_soft(map: texture_depth_2d_array, cmp: sampler_comparison, cascades: SunCascades,
                   position: vec3<f32>, normal: vec3<f32>, spread: f32, n_dot_l: f32) -> f32 {
    var c = cascades;
    if c.settings.x < 0.5 {
        return 1.0;
    }
    let cl = clamp(n_dot_l, 0.05, 1.0);
    let tan_l = sqrt(1.0 - cl * cl) / cl;
    let band = max(c.settings.w, 1e-3);
    for (var i = 0u; i < 4u; i++) {
        let here = sun_shadow_pick(map, cmp, c, i, position, normal, spread, tan_l);
        if here.y >= 1.0 {
            continue;
        }
        let t = clamp((here.y - (1.0 - band)) / band, 0.0, 1.0);
        if t <= 0.0 {
            return here.x;
        }
        var next = 1.0;
        if i < 3u {
            let there = sun_shadow_pick(map, cmp, c, i + 1u, position, normal, spread, tan_l);
            if there.y >= 1.0 {
                return here.x;
            }
            next = there.x;
        }
        return mix(here.x, next, t);
    }
    return 1.0;
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
