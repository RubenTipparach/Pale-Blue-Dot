// Rain splashes where rain lands (`weather-rain` 6.4, the approved M32
// mockup): one instanced draw of small square droplets, every droplet's whole
// life computed here from its lane index and the clock. Nothing about a
// splash is stored and nothing is uploaded per frame but the view's own
// uniform: the near column list (`planet_splash.rs`) changes only when the
// camera crosses a cell or the fine set changes.
//
// An instance is one droplet: lane = (column * S + slot) * K + droplet. A
// slot fires once a cycle of `period` seconds when a hash of its cycle falls
// under `rain * keep`; its impact point is hashed anew inside the column's
// top each cycle, and the puddle mask there (the terrain's own) picks a crown
// on standing water, a lower splash on wet bare ground, or a softer spray on
// grass. A droplet flies the arc `p = impact + v t - g t^2 / 2` and is gone
// when it falls back to the top. A dead droplet is a degenerate quad.
//
// Group 0 is the terrain's own (`planet.rs`, `draw_layout`): the planet's
// params and the atlas. The items copied from `planet_surface.wgsl` below are
// held to that file word for word by `planet_splash.rs`'s tests.

struct Params {
    clip_from_body: mat4x4<f32>, camera: vec4<f32>, sun: vec4<f32>, settings: vec4<f32>,
    // `water_absorption` rgb is the DOWNWELLING per metre (`ocean-environments`
    // B2), the daylight arriving at a depth; w the sheet's radius.
    water_absorption: vec4<f32>, water_deep: vec4<f32>, weather: vec4<f32>,
    rain: array<vec4<f32>,6>, // wet knobs; overcast sun, fill, fog per cover, fog in rain; sky blue cut, sky dim
    lod_offsets: vec4<u32>, // x base count, y fine-region capacity, z the current set's half (`two-set-dissolve`)
    lod_counts: vec4<u32>,  // live records per fine level of the current set, coarsest first
    lod: vec4<f32>,         // xyz player direction, w base level
    bands: vec4<f32>,       // cos(band radius / R) per fine level, coarsest first
    clutter: vec4<f32>,        // reach m, fade m, blades, base shade
    clutter_chance: vec4<f32>, // grass, flower, rock, bush
    clutter_size: vec4<f32>,   // blade height, blade half-width, rock, bush
    clutter_more: vec4<f32>,   // flower height, shrub chance, shrub size, spare
    column: vec4<f32>,         // tier reach m, cave dark floor, cave dark depth m, cos(2 x reach / R)
    ground: vec4<f32>,         // sod depth m, soil depth m, snow tileset slot, spare
    tilesets: array<vec4<u32>,2>, // atlas slot per biome, in Biome order
    fade: vec4<f32>,           // `detail-fade`: tree fade m, cross-fade progress 0..1, one while it runs; w one for a sun cascade (`sun-shadows`)
    lod_prev: vec4<f32>,       // the partition the cross-fade leaves: xyz its anchor
    bands_prev: vec4<f32>,     // and its band cosines
    lod_counts_prev: vec4<u32>, // the dissolve's old set's counts (`two-set-dissolve`);
    lod_spare: vec4<f32>,       // caustics (`ocean-environments` B5): strength, the web's cell m, gone by m; the visibility pass reads none of it
    anchor: vec4<f32>,         // `distance-lod-fade`: the fine set's anchor (`lod` is the fade centre),
    bands_in: vec4<f32>,       // the bands' cross-fade rings' inner edges, cosines,
    bands_prev_in: vec4<f32>,  // and those of the partition a landing dissolves from
    light: vec4<f32>,          // `lighting-model`: x one for the new model, y a lamp's glow (HDR), z the sun's radius as a tangent (L3), w its widening under a full deck
    sky_sh: array<vec4<f32>,18>, // L2: the sky's SH, clear then a full deck, local frame
    wet: vec4<f32>,            // the new wet film: roughness wet and in a puddle, lamp highlights, one when the rings follow the rain
    lamps: array<vec4<f32>,8>, // the lights nearest the eye, planet frame; w the level 0..1 (zero: none)
    reflect: vec4<f32>,        // L8: x one while the reflected ray walks the columns, y its reach m; L4: z the bounce's strength now; w spare
    lamp_bake: vec4<f32>,      // L7: x the newest bake's fade-in 0..1, y the older half holds this set, z the newest does, w the newest half
    ssr_clip: mat4x4<f32>,     // L8 (10.2): the screen trace history's clip_from_body
    ssr_eye: vec4<f32>,        // and its eye; w one once there is a history
    rain_near: vec4<f32>,      // `wet-near-detail`: x the reach of rain's fine detail, m; y the fade band before it, m
    light_ease: vec4<f32>,     // `baked-light-ease` 7.3: x the light field's newest half blended in 0..1, y that half, z one while the other is blended from
    flora: vec4<f32>,          // `flora-v2`: x tufts per cell (zero: blades), y tuft width m, z tuft height m, w the year 0..1
    flora_more: vec4<f32>,     // x mushroom chance, y fern chance, zw spare
    flora_biome: array<vec4<f32>,8>, // per biome: fern weight, mushroom weight, flower bits, deciduous
    tuft_ramps: array<vec4<u32>,8>,  // per biome: the tuft's three tones 0xRRGGBB, w red caps
    plant_means: array<vec4<u32>,8>, // `flora-v2` 9.2, 9.6: per biome the leaves, bark, stone tiles' means 0xRRGGBB, w fruit bits (low 16), tree table (high 16)
    flora_fall: vec4<f32>,     // 9.4, 9.6: x leaf fall, y gathered under trees, z leaf size, w fruit chance
    tree_fade: array<vec4<f32>,2>, // `flora-v2` 13.3: the trees' cross-fade by time
    night: vec4<f32>,          // `night-lights-from-space`: x how many far lights `night_lights` holds
    night_lights: array<vec4<f32>,128>, // two a light: direction, cos(reach); radius m, strength now, seed, points' spacing m
    felled: array<vec4<u32>,16>,  // `flora-v2` 12.5: felled tree ids near the camera, four a lane, zero empty
    picked: array<vec4<u32>,8>,   // and the trees picked bare
}
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(3) var atlas: texture_2d<f32>;

struct SplashColumn {
    // xyz: the centre of the column's topmost up-face, body-local metres;
    // w: the radius of the circle inside that face, metres.
    top: vec4<f32>,
    // x: the face's render code in the low byte and the biome above it, as
    // the record packs them; y: the class (0 grass, 1 bare ground); z: the
    // column's slot key (a stable hash of its cell); w: spare.
    info: vec4<u32>,
}
struct Splash {
    // x slots per column (S), y droplets per slot (K), z columns, w spare.
    shape: vec4<u32>,
    // x the cycle, s; y the reach, m; z the fade before it, m; w the rain, 0..1.
    timing: vec4<f32>,
    // Droplet sizes, m: crown, wet ground, grass spray, the crown's jet.
    size_m: vec4<f32>,
    // Launch speed up, m/s, at the low end of each kind's spread.
    up_mps: vec4<f32>,
    // Launch speed out, m/s, at the low end of each kind's spread.
    out_mps: vec4<f32>,
    // Share of a slot's cycles that fire at full rain per kind; w the alpha.
    keep: vec4<f32>,
    // x the view's height in pixels, y its width, z the projection's focal
    // length in pixels (clip w to pixels per metre), w spare.
    view: vec4<f32>,
    // xyz: the eye, body-local; w spare.
    eye: vec4<f32>,
}
@group(1) @binding(0) var<uniform> splash: Splash;
@group(1) @binding(1) var<storage, read> columns: array<SplashColumn>;

const GRAVITY: f32 = 9.8;
const KIND_GRASS: u32 = 0u;
const KIND_GROUND: u32 = 1u;
const KIND_CROWN: u32 = 2u;

fn normalized(v: vec3<f32>) -> vec3<f32> { return v*inverseSqrt(max(dot(v,v),1e-12)); }
fn rr_hash12(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3(p.x,p.y,p.x)*0.1031);
    p3 += dot(p3, p3.yzx+19.19);
    return fract((p3.x+p3.y)*p3.z);
}
fn rr_hash22(p: vec2<f32>) -> vec2<f32> {
    var p3 = fract(vec3(p.x,p.y,p.x)*vec3(0.1031,0.1030,0.0973));
    p3 += dot(p3, p3.yzx+19.19);
    return fract((p3.xx+p3.yz)*p3.zy);
}
fn puddle_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let w = f*f*(3.-2.*f);
    let a = mix(rr_hash12(i),rr_hash12(i+vec2(1.,0.)),w.x);
    let b = mix(rr_hash12(i+vec2(0.,1.)),rr_hash12(i+vec2(1.,1.)),w.x);
    return mix(a,b,w.y);
}
const PUDDLE_MASK_MEAN: f32 = 0.5003;
const PUDDLE_MASK_SPREAD: f32 = 0.1587;
fn ground_sky(sun_elevation: f32, cover: f32) -> vec3<f32> {
    let clear = mix(vec3(0.10,0.20,0.29),vec3(0.32,0.49,0.57),max(sun_elevation,0.));
    let grey = vec3(dot(clear,vec3(0.2126,0.7152,0.0722)));
    return mix(clear,grey,cover*params.rain[5].x)*(1.-cover*params.rain[5].y);
}
const SKY_TAU = vec3<f32>(0.040, 0.100, 0.240);
const SKY_TAU_M = 0.07;
const SUN_NOW = vec3<f32>(1.12, 1.03, 0.87);
const SUN_NORM = vec3<f32>(0.880787, 0.821865, 0.699257);
fn airmass(e: f32) -> f32 {
    let h = asin(clamp(e, 0.0, 1.0));
    return 1.0/(sin(h) + 0.50572*pow(h*57.29578 + 6.07995, -1.6364));
}
fn sun_air(e: f32) -> vec3<f32> {
    let t = exp(-(SKY_TAU + SKY_TAU_M)*airmass(e));
    return t/SUN_NORM*SUN_NOW*smoothstep(-0.02, 0.01, e);
}
fn sh_irradiance(n: vec3<f32>, deck: f32) -> vec3<f32> {
    let x = n.x; let y = n.y; let z = n.z;
    let b = array<f32, 9>(0.282095, 0.488603*y, 0.488603*z, 0.488603*x,
        1.092548*x*y, 1.092548*y*z, 0.315392*(3.0*z*z - 1.0), 1.092548*x*z, 0.546274*(x*x - y*y));
    var e = vec3<f32>(0.0);
    for (var k = 0u; k < 9u; k++) {
        e += mix(params.sky_sh[k].xyz, params.sky_sh[k + 9u].xyz, deck)*b[k];
    }
    return e;
}
fn lamp_share(sky: f32, daylight: f32) -> f32 {
    return 1.0 - clamp(daylight, 0.0, 1.0) * clamp(sky, 0.0, 1.0);
}
fn tileset_slot(biome: u32) -> u32 {
    let row = params.tilesets[biome/4u];
    if biome%4u == 0u { return row.x; }
    if biome%4u == 1u { return row.y; }
    if biome%4u == 2u { return row.z; }
    return row.w;
}
const TORCH_TINT: vec3<f32> = vec3<f32>(1.00, 0.70, 0.30);
const TORCH_GAIN: f32 = 1.25;
const FIELDS_BIOME = 2u;
const MOUNTAINS_BIOME = 6u;
const TILLED_CODE = 14u;
const FORGE_CODE = 15u;
const DIRT_CODE = 10u;

// The atlas texel a droplet takes its colour from: the top's own tile at a
// hashed pixel, the sheet and the picture chosen as the terrain chooses them
// for a cap (`shade_terrain`'s table).
fn ground_texel(code: u32, biome: u32, pick: vec2<f32>) -> vec3<f32> {
    var slot = tileset_slot(biome);
    var tile = vec2(0., 0.);
    if code == 5u { tile = vec2(3., 0.); }
    if code == 6u { tile = vec2(2., 2.); slot = u32(params.ground.z); }
    if code == 8u { tile = vec2(2., 1.); }
    if code == 9u { tile = vec2(0., 2.); }
    if code == DIRT_CODE { tile = vec2(2., 0.); }
    if code == TILLED_CODE { tile = vec2(2., 3.); slot = tileset_slot(FIELDS_BIOME); }
    if code == FORGE_CODE { tile = vec2(1., 3.); slot = tileset_slot(MOUNTAINS_BIOME); }
    let pixel = (floor(pick*32.) + 0.5)/32.;
    let sheet = vec2(f32(slot % 4u), f32(slot / 4u))*0.25;
    let uv = sheet + (tile + pixel)*(1./16.);
    return textureLoad(atlas, vec2<i32>(uv*vec2<f32>(textureDimensions(atlas))), 0).rgb;
}

// The terrain's puddle at a point on an up-face whose normal is `up`: its
// triplanar uv on fixed body axes, its two octaves and its own quantile edge
// at the current soak (`shade_terrain`, the wet block).
fn puddle_at(p: vec3<f32>, up: vec3<f32>) -> f32 {
    let an = abs(up);
    var uv = p.xz;
    if an.x >= an.y && an.x >= an.z { uv = p.zy; }
    else if an.z > an.y { uv = p.xy; }
    let scale = max(params.rain[3].y, 0.01);
    let mask = puddle_noise(uv/scale)*0.65 + puddle_noise(uv/(scale*0.37) + vec2(17., 5.))*0.35;
    let soak = clamp(params.weather.x, 0., 1.);
    let share = clamp(params.rain[3].z*soak, 1e-4, 0.9999);
    let edge = PUDDLE_MASK_MEAN + PUDDLE_MASK_SPREAD*log((1. - share)/share)/1.702;
    return smoothstep(edge - 0.03, edge + 0.03, mask);
}

struct SplashOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) color: vec4<f32>,
}

fn dead() -> SplashOut {
    var out: SplashOut;
    out.clip = vec4(2., 2., 2., 1.);
    out.color = vec4(0.);
    return out;
}

@vertex
fn vertex(@builtin(vertex_index) corner: u32, @builtin(instance_index) lane: u32) -> SplashOut {
    let slots = max(splash.shape.x, 1u);
    let drops = max(splash.shape.y, 1u);
    let k = lane % drops;
    let slot_lane = lane / drops;
    let c = slot_lane / slots;
    if c >= splash.shape.z { return dead(); }
    let column = columns[c];
    let s = slot_lane % slots;
    // The slot's own key: the column's stable hash and the slot, so a list
    // rebuilt in another order keeps every slot's rhythm.
    let key = f32(column.info.z*64u + s);
    let period = max(splash.timing.x, 0.05);
    let ph = params.settings.z/period + rr_hash12(vec2(key*0.613, 7.17));
    let cycle = floor(ph);
    let age = fract(ph)*period;
    let sd = vec2(key, (cycle % 4096.)*1.618);
    // Fire first: most lanes are dark at any moment, and leave here.
    let cls = column.info.y;
    let rain = clamp(splash.timing.w, 0., 1.);
    let fire_hash = rr_hash12(sd*vec2(1.37, 0.71) + vec2(17., 3.));
    if fire_hash >= rain*max(max(splash.keep.x, splash.keep.y), splash.keep.z) { return dead(); }
    // Where it lands: a hashed point inside the top's inner circle.
    let up = normalized(column.top.xyz);
    let ref_axis = select(vec3(1., 0., 0.), vec3(0., 1., 0.), abs(up.y) < 0.9);
    let t1 = normalized(cross(ref_axis, up));
    let t2 = cross(up, t1);
    let r2 = rr_hash22(sd*0.731 + vec2(3.3, 9.1));
    let ang = r2.x*6.28318;
    let impact = column.top.xyz + (t1*cos(ang) + t2*sin(ang))*(sqrt(r2.y)*column.top.w);
    // The reach, measured along the ground from the eye.
    let flat = impact - splash.eye.xyz;
    let across = length(flat - up*dot(flat, up));
    let reach = splash.timing.y;
    let fade = 1. - smoothstep(reach - splash.timing.z, reach, across);
    if fade <= 0.001 { return dead(); }
    // Which splash: a crown where the ground stands in a puddle, as the
    // terrain draws it; grass sprays; other bare ground splashes lower.
    var kind = KIND_GRASS;
    if cls == 1u {
        kind = select(KIND_GROUND, KIND_CROWN, puddle_at(impact, up) > 0.5);
    }
    var keep = splash.keep.z;
    var count = 4u;
    if kind == KIND_GROUND { keep = splash.keep.y; count = 5u; }
    if kind == KIND_CROWN { keep = splash.keep.x; count = 8u; }
    if fire_hash >= rain*keep || k >= count { return dead(); }
    // The droplet's launch, as the mockup's crown, splash and spray.
    let hk = rr_hash12(sd + vec2(f32(k)*3.1, 11.));
    let hk2 = rr_hash12(sd + vec2(f32(k)*5.7, 29.));
    let base = rr_hash12(sd + vec2(51., 2.))*6.28318;
    let wet = mix(0.55, 1., clamp(params.weather.x, 0., 1.));
    var a = base;
    var vh = 0.;
    var vv = 0.;
    var delay = 0.;
    var size = 0.;
    var alpha = splash.keep.w;
    if kind == KIND_CROWN {
        if k < 6u {
            a = base + 6.28318*(f32(k) + 0.35*hk)/6.;
            vh = splash.out_mps.x*(1. + 0.667*hk2);
            vv = splash.up_mps.x*(1. + 0.364*hk);
            size = splash.size_m.x;
            alpha *= 0.85;
        } else if k == 6u {
            vh = 0.03;
            vv = splash.up_mps.w*(1. + 0.18*hk);
            delay = 0.07;
            size = splash.size_m.w;
            alpha *= 0.9;
        } else {
            a = base + 6.28318*hk;
            vh = splash.out_mps.x*0.5;
            vv = splash.up_mps.x*(0.64 + 0.18*hk2);
            size = splash.size_m.x*0.75;
            alpha *= 0.75;
        }
        vv *= wet;
        vh *= wet;
    } else if kind == KIND_GROUND {
        a = base + 6.28318*(f32(k) + 0.5*hk)/5.;
        vh = splash.out_mps.y*(1. + 0.75*hk2);
        vv = splash.up_mps.y*(1. + 0.4*hk);
        size = splash.size_m.y;
        alpha *= 0.7;
    } else {
        a = base + 6.28318*(f32(k) + 0.6*hk)/4.;
        vh = splash.out_mps.z*(1. + hk2);
        vv = splash.up_mps.z*(1. + 0.6*hk);
        size = splash.size_m.z;
        alpha *= 0.65;
    }
    let t = age - delay;
    let rise = vv*t - 0.5*GRAVITY*t*t;
    if t < 0. || rise < 0. { return dead(); }
    let dir = t1*cos(a) + t2*sin(a);
    let p = impact + dir*(vh*t) + up*(rise + 0.01);

    // Lit as the water it came from: the wet ground's own colour mixed toward
    // the sky it mirrors, under the sky, the sun and the lamps the ground takes.
    let sun = params.sun.xyz;
    let e = dot(up, sun);
    let daylight = smoothstep(-0.13, 0.20, e);
    let cover = clamp(params.weather.z, 0., 1.);
    let biome = (column.info.x >> 8u) & 0xffu;
    let code = column.info.x & 0xffu;
    let ground = ground_texel(code, biome, rr_hash22(sd + vec2(f32(k)*1.9, 71.)))*(0.72*0.85);
    let sky = ground_sky(e, cover)*mix(0.12, 1., daylight);
    var water = mix(ground, sky, 0.55);
    if kind == KIND_CROWN { water = mix(ground, sky, 0.6); }
    if kind == KIND_GROUND { water = mix(ground, sky, 0.45); }
    let amb = max(sh_irradiance(vec3(0., 1., 0.), cover), vec3(0.006, 0.008, 0.011));
    let sun_dim = 1. - cover*params.rain[4].x;
    var lamps = vec3(0.);
    for (var i = 0u; i < 8u; i++) {
        let lamp_at = params.lamps[i];
        if lamp_at.w <= 0. { continue; }
        let d = distance(lamp_at.xyz, p);
        if d < 16. {
            lamps += TORCH_TINT*TORCH_GAIN*lamp_at.w*11.6/(4.84 + d*d)*(1. - d/16.);
        }
    }
    lamps *= lamp_share(1., daylight);
    let light = amb + sun_air(e)*max(e, 0.)*sun_dim*0.5 + lamps*0.8;
    let color = water*light + lamps*0.2 + sky*(select(0.45, 0.6, kind == KIND_CROWN));

    // A square of whole pixels, set on the pixel grid: one pixel at the
    // reach, a few at arm's length.
    let clip = params.clip_from_body*vec4(p, 1.);
    if clip.w <= 0.05 { return dead(); }
    let px = floor(max(1., size*splash.view.z/clip.w) + 0.5);
    let viewport = vec2(splash.view.y, splash.view.x);
    let centre = (clip.xy/clip.w*0.5 + 0.5)*viewport;
    let odd = (px % 2.) > 0.5;
    let snapped = select(round(centre), floor(centre) + 0.5, odd);
    let offset = vec2(f32(corner & 1u), f32(corner >> 1u)) - 0.5;
    let at = snapped + offset*px;
    var out: SplashOut;
    out.clip = vec4((at/viewport*2. - 1.)*clip.w, clip.z, clip.w);
    out.color = vec4(max(color, vec3(0.)), clamp(alpha*fade, 0., 1.));
    return out;
}

@fragment
fn fragment(input: SplashOut) -> @location(0) vec4<f32> {
    return input.color;
}
