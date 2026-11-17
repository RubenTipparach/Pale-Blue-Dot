// The water system's GPU side: Tenebris's water.vs/fs.glsl and composite.fs.glsl
// in WGSL, term for term, with the cap pulled from the planet's persistent Cell
// record instead of a water vertex stream. Provenance and the deliberate
// differences are in docs/shader-port.md.
//
// Reverse-Z depth (near=1, sky=0), linear scene colour, the resolved
// single-sample scene textures. Positions are body-local; `planet_center` is
// where that frame sits in the render frame (zero today).
#import pbd::clouds::{CloudLayer, CloudSample, cloud_span, cloud_march, cloud_sphere_hit, cloud_flash_at, cloud_map_smooth, cloud_hash, cloud_near_m, cloud_near_fog, cloud_density, cloud_lookup, cloud_local, cloud_wind, cloud_noise, cloud_sunlight, cloud_sky_light, cloud_day_share}
#import pbd::sea::{SeaView, sea_surface}
#import pbd::sun_shadow::SunCascades
struct Cell {
    direction_height: vec4<f32>,
    corners: array<vec4<f32>,6>,
    metadata: vec4<u32>,
    owner_a: vec4<f32>,
    owner_b: vec4<f32>,
    floors: vec4<f32>,
    spare: vec4<u32>,  // x the tree it stands (`distance-lod-fade`)
}
struct WaterView {
    clip_from_local: mat4x4<f32>,
    local_from_clip: mat4x4<f32>,
    // The previous frame's clip_from_local: where the clouds' history is.
    prev_clip_from_local: mat4x4<f32>,
    camera_time: vec4<f32>,   // xyz camera in the local frame, w seconds * time_scale
    planet_center: vec4<f32>, // xyz body centre in the local frame, w sea radius m
    sun: vec4<f32>,           // xyz toward the sun, w specular intensity
    waves: vec4<f32>,         // xyz the downwelling per metre (`ocean-environments` B2), w the ripples' steepness (the swell is `sea`)
    ripple: vec4<f32>,        // ripple scale cells/m, ripple speed, rain ripple cells/m, rain ripple strength
    refraction: vec4<f32>,    // strength, max uv offset, slope cap, max path m
    absorption: vec4<f32>,    // rgb per metre along the view, w the turbidity at the eye per metre
    deep_color: vec4<f32>,    // rgb the deep's albedo, w falling-face flow speed
    shallow: vec4<f32>,       // rgb the shallows' albedo, w horizon (grazing) reflection floor
    water_light: vec4<f32>,   // rgb the light on open water at the camera, w shallows' depth m
    haze: vec4<f32>,          // the terrain's cloud_fog_add, rain_fog_mult, overcast blue cut, overcast dim
    foam_color: vec4<f32>,    // rgb, w foam intensity
    foam_crest: vec4<f32>,    // lo, hi, weight, w rain's fine detail's reach m (`wet-near-detail`)
    foam_slope: vec4<f32>,    // lo, hi, weight, w the fade band before that reach m
    sun_tint: vec4<f32>,      // rgb, w specular power
    fog_night: vec4<f32>,     // rgb sky colour at night, w fog density per metre
    fog_day: vec4<f32>,       // rgb sky colour by day, w fog height m
    limits: vec4<f32>,        // fog ceiling, terminator lo, terminator hi, fog mix
    fx: vec4<f32>,            // underwater distortion, submersion 0/0.5/1, rain, emerge
    lens: vec4<f32>,          // droplet density, bend (rain_lens_bend), speed, size
    screen: vec4<f32>,        // aspect, surface band m, wet blur, detail fade
    lod: vec4<f32>,           // xyz player direction, w base level
    bands: vec4<f32>,         // cos(band radius / R) per fine level, coarsest first
    bands_in: vec4<f32>,      // the bands' cross-fade rings' inner edges (`distance-lod-fade`)
    lod_prev: vec4<f32>,      // the partition a landing dissolves from: xyz its centre, w the dissolve's progress (`two-set-dissolve`)
    bands_prev: vec4<f32>,    // and its bands
    bands_prev_in: vec4<f32>, // and their rings' inner edges
    rain: vec4<f32>,         // x the rain on the LENS (zero under a roof), y frame s, z airspeed, w one when the rings follow the rain (`lighting-model`)
    // The cloud layer, `pbd::clouds::CloudLayer`'s four lanes as the sky has
    // them this frame, so the sheet can put the same clouds over itself.
    cloud_clouds: vec4<f32>,
    cloud_slab: vec4<f32>,
    cloud_storm: vec4<f32>,
    cloud_flash: vec4<f32>,
    cloud_light: vec4<f32>,
    cloud_shape: vec4<f32>,
    cloud_cells: vec4<f32>,
    // The precipitation map (`weather::RainMap`): the anchor direction and the
    // cell size in metres; the plane's u axis and the map's side in cells; its
    // v axis and the volume's extinction per metre of full rain.
    rain_map: vec4<f32>,
    rain_u: vec4<f32>,
    rain_v: vec4<f32>,
    // Rain fall m/s, streak stretch, how lit the rain is (`rain_light`), and
    // how far the volume is marched (zero draws none).
    rain_look: vec4<f32>,
    // The rain's colour, linear; w the snow's fall speed, m/s.
    rain_tint: vec4<f32>,
    // The snow's colour, linear; w the weather clock, seconds.
    snow_tint: vec4<f32>,
    // The overlay (`overlay.rs`): x its ramp's row plus one, zero when none
    // is showing; y and z the values the ramp's ends stand for; w opacity.
    overlay: vec4<f32>,
    // x one streak step (m), y how many streak lengths a second a flow at
    // the top of the range crawls, z streak brightness, w flags: 1 the
    // overlay flows (draw streaks), 2 it fades out toward zero.
    overlay_flow: vec4<f32>,
    // The clouds' accumulation (`calm-clouds`): x the new frame's share of
    // the blend, y one when the history is usable, z the frame number; w the
    // haze on the clouds (`cloud_haze`, `cloud-close-up`).
    cloud_history: vec4<f32>,
    // xyz the previous frame's eye, body-local: where the previous frame's
    // cloud distances were measured from (`cloud-ghosting`); w spare.
    cloud_prev_eye: vec4<f32>,
    // The sea's table (`pbd::sea`), the same one the hulls float on.
    sea: SeaView,
    // x how far the sheet sits below sea level (`depth_offset_m`); y the
    // surface of the water the eye is in or over, a body radius: an inland
    // water's drawn top, else the sea's tided, swelling surface at the eye
    // (`ocean-environments` B17, B18); z and w how far the drawing is
    // through the water's last tick, smoothstepped (the levels) and linear
    // (the fronts), both one when it is drawn as it is (`water_flow::WaterEase`).
    sea_frame: vec4<f32>,
    // The lens mist (`lens-weather`): seconds to mist over and to clear
    // standing still, how much faster per m/s of airspeed, its strength; then
    // its beads per screen height, its blur in screen heights, how far it lifts
    // the image toward its own brightness, and the drops' scale.
    mist: vec4<f32>,
    mist_look: vec4<f32>,
    // A lens drop's edge (`lens-drop-refraction`): x the light it loses
    // there (`rain_lens_rim`), y the sky's light it gathers on the lower part
    // (`rain_lens_glint`); zw spare.
    lens_look: vec4<f32>,
    // The puffs (`cloud-puffs`): the sun's wrap, a white and a storm column's
    // base shade, the silver lining; then the edge erosion, the fade in front
    // of the ground (m), the fade-in from the eye (m), and one when the puffs
    // draw the clouds (the march then makes only the near fog).
    cloud_puff: vec4<f32>,
    cloud_puff_fade: vec4<f32>,
    // The rain shells (`rain-shells`): xyz the eye's velocity, body-local m/s,
    // w one when the shells draw the rain instead of the volume; then how far
    // the rain and the snow have fallen past the eye, m (wrapped at
    // `RAIN_SHELL_WRAP_M` in Rust), the fade in front of the ground (m), w spare.
    rain_shell: vec4<f32>,
    rain_shell_fall: vec4<f32>,
    // `lighting-model`, the rain fog lit: the sky's mean radiance (w one for
    // the new model), the sun after the air and the cloud (w the forward
    // scatter's g), and the lights nearest the eye (w the level, 0 none).
    rain_sky: vec4<f32>,
    rain_sun: vec4<f32>,
    rain_lamps: array<vec4<f32>, 8>,
    // `lighting-model` L8 (10.3): the screen trace history's clip_from_body
    // and its eye (w one once there is one, and reflections are on).
    ssr_clip: mat4x4<f32>,
    ssr_eye: vec4<f32>,
    // `water-lit`: the cover and the rain over the camera (the haze's); the
    // sky's nine L2 coefficients at the camera, mixed by the cover.
    haze_weather: vec4<f32>,
    sky_sh: array<vec4<f32>, 9>,
    // Under the water (`ocean-environments` B5): the shafts' strength and
    // reach m, the marine snow's strength and the depth it shows from m;
    // then the snow's own glow, the wind at the sea m/s, the spray's start
    // m/s and its strength.
    under: vec4<f32>,
    under_more: vec4<f32>,
    // The column tier (`water-flow` E6, design 13): xyz its anchor, w its
    // reach, m. A far river's sheet and the tier's own water are dithered
    // across its last 25 m, so neither pops in.
    column: vec4<f32>,
    // The height fog in the rain volume's place while RAIN SHADOW is off
    // (`weather-rain` 7): x the extinction per metre at the reference level
    // (zero: none), y the falloff height m, z the reference level's radius
    height_fog: vec4<f32>,
    // A fall's look (`water-flow` 6.6, M35's New tab; `water.ron` `falls`):
    // x the feet told of, y the teal churn, z the ripples' height, w the
    // metres between rings; x the ripples' reach m, y the pool flow, z its
    // reach m, w the speed-up g m/s^2; the glassy lip: x on, y smooth before
    // the edge m, z glassy over it m, w the froth filled by m; x the glassy
    // green, y the edge highlight, z the fastest a face moves m/s, w the
    // bubbles; the splash: x the wall blend m, y the mist's opacity, z one
    // when smooth, w the viewport's height px; rgb the aerated colour, w
    // the water's time scale.
    fall_look: vec4<f32>,
    fall_flow: vec4<f32>,
    fall_lip: vec4<f32>,
    fall_more: vec4<f32>,
    fall_splash: vec4<f32>,
    aer_color: vec4<f32>,
    // The nearest feet: xyz where each lands (body-local), w its drop m;
    // then x the pool's depth under it m, y a seed, z its fade.
    feet: array<vec4<f32>, 8>,
    feet_more: array<vec4<f32>, 8>,
}
@group(0) @binding(0) var<uniform> view: WaterView;
@group(0) @binding(1) var<storage,read> cells: array<Cell>;
@group(0) @binding(2) var<storage,read> water: array<u32>;
// The game's own water that the sheet does not draw (`water-flow` E6): four
// lanes a vertex, triangles end to end. xyz body-local position, w the sky's
// share; xyz the face's normal, w its kind (1 a top, 2 a falling face, 3 a
// still face); xy the flow in the face's tangent frame, m/s over 0.35; xyz
// where the vertex was the tick before, w its ease flags (1 a front, eased
// linearly; 2 a dried piece, hidden once the tick has played).
@group(0) @binding(3) var<storage,read> flowing: array<vec4<f32>>;
@group(0) @binding(4) var<storage,read> rain_map: array<f32>;
@group(1) @binding(0) var scene_color: texture_2d<f32>;
#ifdef MULTISAMPLED
@group(1) @binding(1) var scene_depth: texture_depth_multisampled_2d;
#else
@group(1) @binding(1) var scene_depth: texture_depth_2d;
#endif
@group(1) @binding(2) var scene_sampler: sampler;
// The weather maps (`planet_weather.rs`): cover, cloud top, rain and optical
// depth per place; the wind aloft; the overlay; one sampler.
@group(2) @binding(0) var weather_cloud: texture_cube<f32>;
@group(2) @binding(1) var weather_wind: texture_cube<f32>;
@group(2) @binding(2) var weather_overlay: texture_cube<f32>;
@group(2) @binding(3) var weather_sampler: sampler;
// The clouds passes only: a cloud-march target (the march reads the previous
// frame's history, the composite this frame's), and the baked cellular noise
// with its repeating sampler (`cloud-budget`).
@group(3) @binding(0) var cloud_history: texture_2d<f32>;
@group(3) @binding(1) var cloud_history_sampler: sampler;
@group(3) @binding(2) var cloud_cells_tex: texture_3d<f32>;
@group(3) @binding(3) var cloud_cells_sampler: sampler;
// How far along its ray each march texel's cloud is, and where its march
// stopped: the composite reads this frame's, the resolve the previous frame's
// (`cloud-ghosting`).
@group(3) @binding(4) var cloud_depth: texture_2d<f32>;
// The resolve only (`cloud-history-clip`): this frame's march, and this
// frame's distances.
@group(3) @binding(5) var cloud_current: texture_2d<f32>;
@group(3) @binding(6) var cloud_depth_now: texture_2d<f32>;
// The composite's: the near field's fog, lit colour and extinction per metre.
@group(3) @binding(7) var cloud_near: texture_2d<f32>;
// The composite's, under the puffs (`puff-haze`): their coverage as the air
// leaves it, in r. The history's alpha is then the plain coverage.
@group(3) @binding(16) var cloud_puff_cover: texture_2d<f32>;
// The lens mist (`lens-weather`), one texel: the probe reads the previous
// frame's and writes this frame's, which the lens reads.
@group(3) @binding(8) var lens_mist_prev: texture_2d<f32>;
@group(3) @binding(9) var lens_mist: texture_2d<f32>;
fn cloud_layer() -> CloudLayer {
    return CloudLayer(view.cloud_clouds,view.cloud_slab,view.cloud_storm,view.cloud_flash,view.cloud_light,
        view.cloud_shape,view.cloud_cells);
}

struct VertexOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) local_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) sky_light: f32,
    @location(3) body_position: vec3<f32>,
    @location(4) flow_uv: vec3<f32>,
    @location(5) @interpolate(flat) level: u32,
    @location(6) @interpolate(flat) owner_a: vec3<f32>,
    @location(7) @interpolate(flat) owner_b: vec3<f32>,
    // The swell's slope along the sphere, for the normal.
    @location(8) sea_slope: vec3<f32>,
    // The owners' fades and the finer band's fade at the centre, as the
    // terrain carries them (`distance-lod-fade`).
    @location(9) @interpolate(flat) fade_t: vec3<f32>,
    // Which partition the sheet is drawn for while a landing dissolves, as
    // the terrain marks it (`two-set-dissolve`): 0 both, 1 the new, 2 the old.
    @location(10) @interpolate(flat) part_mark: u32,
    // 0 the sea sheet; else the flowing water's face kind (`vertex_flowing`).
    @location(11) @interpolate(flat) kind: u32,
}
fn safe_normal(v: vec3<f32>) -> vec3<f32> { return v * inverseSqrt(max(dot(v,v),1e-12)); }

// ---- Gradient noise and fbm, hash constants as Tenebris has them ----------
fn gn_fade(t: vec3<f32>) -> vec3<f32> { return t*t*t*(t*(t*6.0-15.0)+10.0); }
fn gn_hash(p: vec3<i32>) -> u32 {
    // Integer wrapping preserves the original lattice hash without signed overflow.
    let q = bitcast<vec3<u32>>(p) + vec3<u32>(1073741824u);
    var h = (q.x*374761393u) ^ (q.y*668265263u) ^ (q.z*1274126177u);
    h = (h ^ (h>>13u))*1103515245u;
    return h ^ (h>>16u);
}
fn gn_grad(hash: u32, p: vec3<f32>) -> f32 {
    let h = hash & 15u;
    let u = select(p.y,p.x,h<8u);
    let v = select(select(p.z,p.x,h==12u || h==14u),p.y,h<4u);
    return select(u,-u,(h&1u)!=0u) + select(v,-v,(h&2u)!=0u);
}
fn gnoise3(p: vec3<f32>) -> f32 {
    let lattice = vec3<i32>(floor(p));
    let f = fract(p);
    let s = gn_fade(f);
    var values: array<f32,8>;
    for (var i = 0u; i < 8u; i++) {
        let corner = vec3<i32>(i32(i&1u),i32((i>>1u)&1u),i32((i>>2u)&1u));
        values[i] = gn_grad(gn_hash(lattice+corner), f-vec3<f32>(corner));
    }
    return mix(mix(mix(values[0],values[1],s.x),mix(values[2],values[3],s.x),s.y),
               mix(mix(values[4],values[5],s.x),mix(values[6],values[7],s.x),s.y),s.z);
}
fn fbm3(p: vec3<f32>, t: f32) -> f32 {
    var q = p*0.9 + vec3<f32>(0.35,0.18,-0.42)*t;
    var h = gnoise3(q)*0.5;
    q = q*2.1 + vec3<f32>(-0.22,0.33,0.17)*t;
    h += gnoise3(q)*0.28;
    q = q*2.3 + vec3<f32>(0.19,-0.27,0.11)*t;
    return h + gnoise3(q)*0.15;
}

// ---- Rain ripples: Zavie / Ctrl-Alt-Test "H - Immersion" raindrop kernel as a
// surface gradient, exactly as water.fs.glsl carries it. The kernel's inner
// constants are the effect's identity and stay inline; the scale and strength
// are `ripple.zw`, gated by the rain intensity `fx.z`.
fn rr_hash12(p: vec2<f32>) -> f32 {
    var p3 = fract(vec3<f32>(p.x,p.y,p.x)*0.1031);
    p3 += dot(p3, p3.yzx + 19.19);
    return fract((p3.x+p3.y)*p3.z);
}
fn rr_hash22(p: vec2<f32>) -> vec2<f32> {
    var p3 = fract(vec3<f32>(p.x,p.y,p.x)*vec3<f32>(0.1031,0.1030,0.0973));
    p3 += dot(p3, p3.yzx + 19.19);
    return fract((p3.xx+p3.yz)*p3.zy);
}
// `lighting-model`: each drop of each cycle kept with a chance `keep`, picked
// anew each cycle, on a grid offset by `salt` (the terrain's own rings).
fn rain_ripple_layer(uv: vec2<f32>, t: f32, keep: f32, salt: f32) -> vec2<f32> {
    let p0 = floor(uv);
    var circles = vec2<f32>(0.0);
    for (var j = -1; j <= 1; j++) {
        for (var i = -1; i <= 1; i++) {
            let pi = p0 + vec2<f32>(f32(i),f32(j));
            let p = pi + rr_hash22(pi + salt);
            let ph = 0.3*t + rr_hash12(pi + salt);
            let tt = fract(ph);
            if rr_hash12(pi*1.37 + floor(ph)*7.13 + salt) >= keep { continue; }
            let v = p - uv;
            let len = length(v) + 1e-6;
            let d = len - 2.0*tt;
            let h = 1e-3;
            let d1 = d - h;
            let d2 = d + h;
            let q1 = sin(31.0*d1)*smoothstep(-0.6,-0.3,d1)*smoothstep(0.0,-0.3,d1);
            let q2 = sin(31.0*d2)*smoothstep(-0.6,-0.3,d2)*smoothstep(0.0,-0.3,d2);
            circles += 0.5*(v/len)*((q2-q1)/(2.0*h)*(1.0-tt)*(1.0-tt));
        }
    }
    return circles/9.0;
}
fn rain_rings(uv: vec2<f32>, t: f32, rain: f32) -> vec2<f32> {
    var g = rain_ripple_layer(uv, t, clamp(rain*2.0, 0.0, 1.0), 0.0);
    if rain > 0.5 { g += rain_ripple_layer(uv + vec2<f32>(0.5, 0.37), t*1.07, rain*2.0 - 1.0, 41.0); }
    return g;
}
fn rain_ripple_grad(uv: vec2<f32>, t: f32) -> vec2<f32> {
    let p0 = floor(uv);
    var circles = vec2<f32>(0.0);
    for (var j = -1; j <= 1; j++) {
        for (var i = -1; i <= 1; i++) {
            let pi = p0 + vec2<f32>(f32(i),f32(j));
            let p = pi + rr_hash22(pi);
            let tt = fract(0.3*t + rr_hash12(pi));
            let v = p - uv;
            let len = length(v) + 1e-6;
            let d = len - 2.0*tt;
            let h = 1e-3;
            let d1 = d - h;
            let d2 = d + h;
            let q1 = sin(31.0*d1)*smoothstep(-0.6,-0.3,d1)*smoothstep(0.0,-0.3,d1);
            let q2 = sin(31.0*d2)*smoothstep(-0.6,-0.3,d2)*smoothstep(0.0,-0.3,d2);
            circles += 0.5*(v/len)*((q2-q1)/(2.0*h)*(1.0-tt)*(1.0-tt));
        }
    }
    return circles/9.0;
}

// ---- The cap, pulled from the Cell record ----------------------------------
// One instance per water cell listed by the visibility pass; 18 vertices fan
// the cap from its centre. A pentagon's sixth triangle collapses to the centre.
@vertex
fn vertex(@builtin(vertex_index) vertex: u32, @builtin(instance_index) instance: u32) -> VertexOut {
    // A cell's index, and in the top two bits the partition the sheet is
    // drawn for, as the terrain's list entries carry it (`two-set-dissolve`).
    let listed = water[instance];
    let mark = listed >> 30u;
    let id = listed & 0x3fffffffu;
    let cell = cells[id];
    let degree = cell.metadata.x & 0xffu;
    let level = cell.metadata.x >> 8u;
    let axis = cell.direction_height.xyz;
    let sea = view.planet_center.w;
    var ray = axis;
    let triangle = vertex/3u;
    let corner = vertex%3u;
    if (triangle < degree && corner != 0u) {
        ray = cell.corners[(triangle+corner-1u)%degree].xyz;
    }
    // A far river (`spare.w`, design 13): a still sheet at its own surface,
    // a hair over the cell's ground where the cell is wider than the river.
    let river = cell.spare.w != 0u;
    var radius = sea;
    if river {
        let surface_m = bitcast<f32>(cell.spare.w);
        radius = view.planet_center.w + view.sea_frame.x
            + max(surface_m, cell.direction_height.w + 0.05);
    }
    let p = ray*radius;
    // The sea the hulls float on (`pbd::sea`), in body-local metres so a
    // floating-origin shift cannot shift the phase. The cell's height is its
    // seabed; the filter drops any wave its own corners are too far apart to
    // draw.
    let depth = -cell.direction_height.w - view.sea_frame.x;
    let spacing = length(cell.corners[0].xyz - axis)*sea;
    var surface = sea_surface(view.sea, p, ray, depth, spacing);
    if river {
        surface.height = 0.0;
        surface.gradient = vec3<f32>(0.0);
    }
    let displaced = p + ray*surface.height;
    var out: VertexOut;
    out.sea_slope = surface.gradient;
    out.body_position = displaced;
    out.local_position = view.planet_center.xyz + displaced;
    out.clip = view.clip_from_local*vec4<f32>(out.local_position,1.0);
    out.normal = axis;
    // The sheet is open sky by construction in a heightfield; the cell's
    // baked occlusion is the seabed's and would tile the sea by column. Under
    // a roof the sea reaches, the record carries how much darker than the
    // open sky its water's top is lit (`SHEET_SHADE_SHIFT`, bits 8-11 of
    // `spare.z`, `ocean-environments` B22), so the sea in a cave is as dark
    // as the cave.
    out.sky_light = 1.0 - f32((cell.spare.z >> 8u) & 15u)/15.0;
    out.flow_uv = vec3<f32>(0.0);
    out.kind = select(0u, 4u, river);
    out.level = level;
    out.owner_a = cell.owner_a.xyz;
    out.owner_b = cell.owner_b.xyz;
    let finest = u32(view.lod.w) + 4u;
    // Split by the partition it is drawn for: the old one's centre and bands
    // for a sheet the old set lists, the current ones otherwise.
    let old = mark == 2u;
    let centre = select(view.lod.xyz, view.lod_prev.xyz, old);
    let bands = select(view.bands, view.bands_prev, old);
    let bands_in = select(view.bands_in, view.bands_prev_in, old);
    out.fade_t = vec3<f32>(water_band_t(cell.owner_a.xyz, level, centre, bands, bands_in),
        water_band_t(cell.owner_b.xyz, level, centre, bands, bands_in),
        select(0.0, water_band_t(cell.direction_height.xyz, level + 1u, centre, bands, bands_in), level < finest));
    out.part_mark = mark;
    return out;
}
// The game's own water (`water-flow` E6): the mesher's triangles, drawn by
// the sea's own fragment so a poured pool and the sea are one water.
@vertex
fn vertex_flowing(@builtin(vertex_index) vertex: u32) -> VertexOut {
    let a = flowing[vertex*4u];
    let b = flowing[vertex*4u + 1u];
    let c = flowing[vertex*4u + 2u];
    let d = flowing[vertex*4u + 3u];
    // Smooth water between ticks (M31's Pour tab): eased from where the
    // vertex was the tick before. Fronts at one speed, levels smoothstepped.
    let flags = u32(d.w + 0.5);
    let t = select(view.sea_frame.z, view.sea_frame.w, (flags & 1u) != 0u);
    let p = mix(d.xyz, a.xyz, t);
    var out: VertexOut;
    out.body_position = p;
    out.local_position = view.planet_center.xyz + p;
    out.clip = view.clip_from_local*vec4<f32>(out.local_position, 1.0);
    // A dried cell, its tick played: gone (outside the clip volume).
    if ((flags & 2u) != 0u && view.sea_frame.w > 0.999) {
        out.clip = vec4<f32>(0.0, 0.0, 2.0, 1.0);
    }
    out.normal = b.xyz;
    out.sky_light = a.w;
    out.flow_uv = c.xyz;
    out.level = 0u;
    out.owner_a = safe_normal(p);
    out.owner_b = out.owner_a;
    out.sea_slope = vec3<f32>(0.0);
    out.fade_t = vec3<f32>(1.0, 1.0, 0.0);
    out.part_mark = 0u;
    out.kind = max(u32(b.w + 0.5), 1u);
    return out;
}

fn water_band_cos(level: u32) -> f32 {
    return water_lane(view.bands, level);
}
fn water_lane(lanes: vec4<f32>, level: u32) -> f32 {
    let k = level - u32(view.lod.w) - 1u;
    if (k == 0u) { return lanes.x; }
    if (k == 1u) { return lanes.y; }
    if (k == 2u) { return lanes.z; }
    return lanes.w;
}
// How far into fine level `level`'s band, across its cross-fade ring, for the
// partition round `centre`: the terrain's `band_t` (`distance-lod-fade`).
fn water_band_t(direction: vec3<f32>, level: u32, centre: vec3<f32>, bands: vec4<f32>,
    bands_in: vec4<f32>) -> f32 {
    if (level <= u32(view.lod.w)) { return 1.0; }
    if (level > u32(view.lod.w) + 4u) { return 0.0; }
    let out_cos = water_lane(bands, level);
    if (out_cos > 1.0) { return 0.0; }
    let in_cos = water_lane(bands_in, level);
    let d = dot(direction, centre);
    let span = in_cos - out_cos;
    if (span <= 1e-9) { return select(0.0, 1.0, d > out_cos); }
    return clamp((d - out_cos)/span, 0.0, 1.0);
}
// The terrain's 4 x 4 screen-door threshold for a pixel, 0..1 (`bayer4` in
// `planet_surface.wgsl`): the sheet takes the same share of the pixels across
// a ring as the ground does.
fn water_bayer4(pixel: vec2<f32>) -> f32 {
    let p = vec2<u32>(pixel) & vec2<u32>(3u);
    let m = array<u32,16>(0u,8u,2u,10u, 12u,4u,14u,6u, 3u,11u,1u,9u, 15u,7u,13u,5u);
    return (f32(m[p.y*4u + p.x]) + 0.5)/16.0;
}

fn load_depth(uv: vec2<f32>) -> f32 {
    let size = textureDimensions(scene_depth);
    let coord = clamp(vec2<i32>(uv*vec2<f32>(size)),vec2<i32>(0),vec2<i32>(size)-1);
#ifdef MULTISAMPLED
    return textureLoad(scene_depth,coord,0);
#else
    return textureLoad(scene_depth,coord,0);
#endif
}
fn reconstruct_local(uv: vec2<f32>, depth: f32) -> vec3<f32> {
    let homogeneous = view.local_from_clip*vec4<f32>(uv.x*2.0-1.0,1.0-uv.y*2.0,depth,1.0);
    // Callers avoid depth=0 infinite-far reconstruction.
    return homogeneous.xyz/max(abs(homogeneous.w),1e-8)*sign(homogeneous.w);
}

// The water's murk at `eye_depth` metres under its surface: the deep's albedo
// lit by the light on open water, dimmed by the water above (`water-lit`).
// Lit, not luminous: it darkens with the land at dusk, under cloud and at
// night. It was a fixed deep colour scaled only by a day/night floor, a
// glowing blue under a black sky.
//
// The light reaching that depth is the DOWNWELLING's (`ocean-environments`
// B2), not the view's absorption: it fades fast enough that blue keeps 15%
// at 40 m and 1.7% at 90 m, so the deep goes dark inside the column.
fn murk(eye_depth: f32, radial: vec3<f32>) -> vec3<f32> {
    return view.deep_color.rgb*view.water_light.rgb*light_left(eye_depth);
}

// The share of daylight left per colour `depth` metres under the surface
// (`pbd_core::ocean::light_left`).
fn light_left(depth: f32) -> vec3<f32> {
    return exp(-max(view.waves.xyz,vec3<f32>(0.0))*max(depth,0.0));
}

// The view's extinction per metre through water: the absorption per colour
// and the turbidity at the eye, which takes every colour toward the murk
// (`ocean-environments` B3). Turbidity rides in the absorption lane's w.
fn view_extinction() -> vec3<f32> {
    return max(view.absorption.rgb,vec3<f32>(0.0)) + vec3<f32>(max(view.absorption.w,0.0));
}

// Light shafts seen from under the water by day (`ocean-environments` B5):
// samples along the view ray, each lit where the line from it toward the
// (refracted) sun leaves the water through a bright patch of a slow pattern
// on the surface, so the shafts are columns that lean with the sun and sway.
// Each sample is as lit as the light left at its depth and dimmed on its way
// back to the eye; brightest looking toward the sun.
// The shafts' pattern's mean over its plane, `pow(max(a b, 0), 6)` of two
// sines: what a sample spanning many of its cells sees (B19).
const SHAFT_MEAN: f32 = 0.045;
fn shafts(origin: vec3<f32>, dir: vec3<f32>, dist: f32, center: vec3<f32>, sea: f32, extinction: vec3<f32>) -> vec3<f32> {
    let strength = view.under.x;
    let reach = min(dist, view.under.y);
    if (strength <= 0.0 || reach <= 0.0) { return vec3<f32>(0.0); }
    let up = safe_normal(origin-center);
    let sun = view.sun.xyz;
    let sun_up = dot(sun, up);
    if (sun_up <= 0.02) { return vec3<f32>(0.0); }
    // Snell: the sun's slant shrinks by the water's index.
    let across = sun - up*sun_up;
    let sin_w = min(length(across)/1.333, 0.99);
    let cos_w = sqrt(1.0-sin_w*sin_w);
    let sun_w = safe_normal(across)*sin_w + up*cos_w;
    // Body-fixed coordinates across the radial's main axis (a cube face), so
    // the shafts stay put as the eye moves.
    let a = abs(up);
    var axes = vec2<u32>(0u, 2u);
    if (a.x >= a.y && a.x >= a.z) { axes = vec2<u32>(1u, 2u); } else if (a.z >= a.y) { axes = vec2<u32>(0u, 1u); }
    let t = view.camera_time.w;
    var gathered = vec3<f32>(0.0);
    let steps = 10;
    // One offset for every ray (B19): a random offset per ray put each
    // pixel's samples on a different part of the pattern, per-pixel speckle
    // wherever the samples stood further apart than the pattern is wide.
    // Neighbouring rays now sample neighbouring points and the shafts are
    // smooth; the packed spacing below keeps the slices from showing.
    let jitter = 0.5;
    // The samples are packed toward the eye (B19): spread evenly over 40 m
    // they stood 4 m apart in a pattern about 4.5 m across, each pixel's
    // jitter landed on a different part of it, and the far end of a river's
    // channel was a band of per-pixel speckle. Each sample now weighs its
    // own stretch of the ray and fades to the pattern's mean where that
    // stretch outgrows the pattern.
    for (var i = 0; i < steps; i++) {
        let u = (f32(i)+jitter)/f32(steps);
        let d = reach*u*u;
        let stretch = 2.0*reach*max(u, 0.5/f32(steps))/f32(steps);
        let q = origin + dir*d;
        let depth = sea - length(q-center);
        if (depth <= 0.0) { continue; }
        let s = q + sun_w*(depth/cos_w);
        let uv = vec2<f32>(s[axes.x], s[axes.y])*1.4;
        let a = sin(uv.x + 1.7*sin(uv.y*0.43 + t*0.21) + t*0.33);
        let b = sin(uv.y*1.13 + 1.7*sin(uv.x*0.37 - t*0.17) - t*0.29);
        let bright = mix(pow(max(a*b, 0.0), 6.0), SHAFT_MEAN, smoothstep(0.8, 2.5, stretch));
        gathered += bright*light_left(depth)*exp(-extinction*d)*stretch;
    }
    let toward = max(dot(dir, sun_w), 0.0);
    let phase = 0.3 + 1.6*pow(toward, 6.0);
    return gathered*0.3*strength*phase*view.rain_sun.rgb;
}

// How far under the surface the composite's waterline mask fades in, m: the
// short soft edge of the underwater tint, wobble and fog. Nothing above the
// surface is ever wet.
const WATERLINE_EDGE_M: f32 = 0.1;

// The sheet nearer the eye than this is behind the near plane (0.2 m), m:
// the composite stands in for it.
const NEAR_SHEET_M: f32 = 0.3;

// Past this distance from the eye the sea's sheet is seen from the side the
// eye is on, not by each fragment's wave height, m (B19).
const SEA_SIDE_NEAR_M: f32 = 3.0;
// The eye's surface is the sea's when it is within this of the sea's mean
// sheet, m: the tide's whole range and some.
const SEA_EYE_RANGE_M: f32 = 3.0;

// The surface of the water the eye is in or over, as a body radius: a
// river's or a pool's drawn top where the eye is in inland water, the sea's
// tided surface at the eye otherwise, the eye in air included
// (`ocean-environments` B17, B18, B21). Everything the underwater view
// measures, depth, the waterline, the sky's exit, is measured from it. The
// mean sphere is only the fallback for a view with no water lanes.
fn eye_water_surface() -> f32 {
    return select(view.planet_center.w, view.sea_frame.y, view.sea_frame.y > 0.0);
}

// What the eye in the water sees added along `dist` metres of its view: the
// shafts, and the marine snow over the twilight and midnight zones, lit by
// what light is left at the eye and a faint glow of its own. Zero above water.
fn under_water_light(origin: vec3<f32>, dir: vec3<f32>, dist: f32) -> vec3<f32> {
    let center = view.planet_center.xyz;
    let sea = eye_water_surface();
    let eye_depth = sea - length(origin-center);
    if (eye_depth <= 0.0) { return vec3<f32>(0.0); }
    var add = shafts(origin, dir, dist, center, sea, view_extinction());
    let snow_on = view.under.z*smoothstep(view.under.w, view.under.w+15.0, eye_depth);
    if (snow_on > 0.0) {
        let speck = marine_snow(origin, dir, dist, safe_normal(origin-center));
        let lit = view.water_light.rgb*light_left(eye_depth)*0.6 + vec3<f32>(view.under_more.x);
        add += vec3<f32>(0.80,0.86,0.88)*lit*(speck*snow_on*2.0);
    }
    return add;
}

// A cheap hash of a cell, three values in 0..1.
fn snow_hash(c: vec3<f32>) -> vec4<f32> {
    var p = vec3<u32>(vec3<i32>(c) + vec3<i32>(1<<20));
    p = p*vec3<u32>(1664525u, 1013904223u, 2654435769u) + p.yzx*vec3<u32>(22695477u, 1103515245u, 1664525u);
    p ^= p >> vec3<u32>(16u);
    p = p*vec3<u32>(2246822519u, 3266489917u, 668265263u) + p.zxy;
    p ^= p >> vec3<u32>(13u);
    let w = (p.x ^ p.y ^ p.z)*747796405u;
    return vec4<f32>(vec4<u32>(p, w) >> vec4<u32>(8u))/16777216.0;
}

// Marine snow (`ocean-environments` B5): specks drifting down round the eye,
// on three shells at 1.2, 2.4 and 4.8 m, from a grid anchored to the body so
// they stay put as the eye turns. How much of a speck this pixel shows, 0..1.
fn marine_snow(origin: vec3<f32>, dir: vec3<f32>, dist: f32, up: vec3<f32>) -> f32 {
    let t = view.camera_time.w;
    var seen = 0.0;
    for (var k = 0; k < 3; k++) {
        let r = 1.2*exp2(f32(k));
        if (r > dist) { break; }
        let cell = 0.03*r;
        let q = (origin + dir*r + up*(t*0.05))/cell;
        let c = floor(q);
        let h = snow_hash(c);
        if (h.w > 0.45) { continue; }
        let at = 0.2 + 0.6*h.xyz;
        let off = length(q - c - at);
        seen += (1.0 - smoothstep(0.08, 0.15, off))*(1.0 - 0.25*f32(k));
    }
    return min(seen, 1.0);
}

// The terrain's sky frame (`sky_frame` in `planet_surface.wgsl`): a
// direction in the local frame the sky's coefficients are in, y up and the
// sun in the x-y plane.
fn water_sky_frame(n: vec3<f32>, up: vec3<f32>, sun: vec3<f32>) -> vec3<f32> {
    let sx = sun - up*dot(sun, up);
    let l = length(sx);
    let ax = select(safe_normal(cross(up, vec3<f32>(0.0, 0.0, 1.0)) + vec3<f32>(1e-4, 0.0, 0.0)), sx/l, l > 1e-4);
    let az = cross(ax, up);
    return vec3<f32>(dot(n, ax), dot(n, up), dot(n, az));
}

// The sky's radiance toward a local direction, out of the camera's nine
// coefficients (`sh_radiance` in `planet_surface.wgsl`, which the wet ground
// mirrors): what the sheet reflects where nothing nearer is found. It greys
// under cloud and darkens at dusk and at night with the sky itself.
fn water_sky_radiance(d: vec3<f32>) -> vec3<f32> {
    let x = d.x; let y = d.y; let z = d.z;
    let b = array<f32, 9>(0.282095, 1.5*0.488603*y, 1.5*0.488603*z, 1.5*0.488603*x,
        4.0*1.092548*x*y, 4.0*1.092548*y*z, 4.0*0.315392*(3.0*z*z - 1.0), 4.0*1.092548*x*z, 4.0*0.546274*(x*x - y*y));
    var e = vec3<f32>(0.0);
    for (var k = 0u; k < 9u; k++) {
        e += view.sky_sh[k].xyz*b[k];
    }
    return max(e, vec3<f32>(0.0));
}

// The terrain's haze colour (`ground_sky` in `planet_surface.wgsl`): the
// clear sky's colour greyed and dimmed by the cover.
fn water_ground_sky(sun_elevation: f32, cover: f32) -> vec3<f32> {
    let clear = mix(vec3<f32>(0.10,0.20,0.29),vec3<f32>(0.32,0.49,0.57),max(sun_elevation,0.));
    let grey = vec3<f32>(dot(clear,vec3<f32>(0.2126,0.7152,0.0722)));
    return mix(clear,grey,cover*view.haze.z)*(1.-cover*view.haze.w);
}

// The terrain's own haze (`finish` in `planet_surface.wgsl`), so the sheet and
// the shore beside it fog to one colour: the same density, thickened by the
// cover and the rain, toward the same overcast-greyed sky. w the share.
fn distance_fog(local_position: vec3<f32>, radial: vec3<f32>, sun: vec3<f32>) -> vec4<f32> {
    let camera_body = view.camera_time.xyz - view.planet_center.xyz;
    let altitude = max(length(camera_body) - view.planet_center.w, 0.0);
    let air = exp(-altitude/1050.0);
    let sun_elevation = dot(radial,sun);
    let daylight = smoothstep(view.limits.y,max(view.limits.z,view.limits.y+1e-4),sun_elevation);
    let cover = clamp(view.haze_weather.x,0.0,1.0);
    let density = 0.00036*(1.0+cover*view.haze.x)*mix(1.0,view.haze.y,clamp(view.haze_weather.y,0.0,1.0));
    let d = length(view.camera_time.xyz - local_position);
    let amount = (1.0-exp(-d*density))*air*daylight*0.55;
    return vec4<f32>(water_ground_sky(sun_elevation,cover), amount);
}

// ---- A fall's look (`water-flow` 6.6; M35, `waterfall-mist/mist.js`) -------
// The most feet the fragment and the bubbles loop over (`FALL_FEET_MAX`).
const FALL_FEET: u32 = 8u;
// The falling column's radius from its foot, m, where the pool's terms start.
const FALL_COLUMN_M: f32 = 1.25;

// The path coordinate down a fall's face with the speed-up (`fallC`): `s`
// metres under the lip the water moves at min(sqrt(v0^2 + 2 g s), vmax); a
// pattern scrolled at `v0` and read at v0 tau(s) moves at that speed, one
// continuous pattern stretched as the water speeds up. g 0 is `s` itself.
fn fall_c(s: f32, v0: f32) -> f32 {
    let g = view.fall_flow.w;
    if (g <= 0.0 || s <= 0.0) { return s; }
    let vm = max(view.fall_more.z, v0 + 1e-3);
    let sc = (vm*vm - v0*v0)/(2.0*g);
    var tau = (vm - v0)/g + (s - sc)/vm;
    if (s <= sc) { tau = (sqrt(v0*v0 + 2.0*g*s) - v0)/g; }
    return v0*tau;
}

// The pool's outward speed, m/s, `x` metres out from the column of a fall
// `h` metres high (`outSpeed`).
fn fall_out_speed(x: f32, h: f32, reach_max: f32) -> f32 {
    return (0.5 + 0.07*min(h, 10.0))*exp(-max(x, 0.0)/min(2.5 + 0.3*h, reach_max))
        *smoothstep(-0.5, 0.3, x)*(1.0 - smoothstep(0.6*reach_max, reach_max, x));
}

// A foot's offset from `p` in the tangent plane, or none (w < 0) where `p`
// is not at the foot's level (0.3 m) or the foot has faded out.
fn fall_foot_offset(i: u32, p: vec3<f32>, radial: vec3<f32>) -> vec4<f32> {
    let foot = view.feet[i];
    let fade = view.feet_more[i].z;
    let rel = p - foot.xyz;
    let up = dot(rel, radial);
    if (abs(up) > 0.3 || fade <= 0.0) { return vec4<f32>(0.0, 0.0, 0.0, -1.0); }
    return vec4<f32>(rel - radial*up, fade);
}

// The lamps near the eye on a point (`lampLight` with a wrap of one), as the
// mist and the bubbles take them.
fn fall_lamps(p: vec3<f32>, n: vec3<f32>) -> vec3<f32> {
    var e = vec3<f32>(0.0);
    for (var i = 0u; i < 8u; i++) {
        let lamp = view.rain_lamps[i];
        if (lamp.w <= 0.0) { continue; }
        let l = lamp.xyz - p;
        let d = length(l);
        if (d > 16.0) { continue; }
        let fall = pow(clamp(1.0 - pow(d/16.0, 4.0), 0.0, 1.0), 2.0)/(1.0 + 0.35*d*d);
        e += vec3<f32>(1.0, 0.62, 0.30)*1.7*lamp.w*fall*clamp((dot(n, l/max(d, 1e-4)) + 1.0)*0.5, 0.0, 1.0);
    }
    return e;
}

@fragment
fn fragment(in: VertexOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    let radial = safe_normal(in.body_position);
    // The same partition the terrain applies, in the same pixel's dither
    // class (`distance-lod-fade`): the sheet is drawn where its owner (the
    // nearer, for a split midpoint cell) is in its band and the finer band
    // has not taken its centre.
    // A landing's dissolve (`two-set-dissolve`): a sheet listed for one
    // partition keeps that partition's half of the pixels, by the terrain's
    // own mask, so the sea and the seabed under it dissolve in the same
    // pixels.
    if (in.part_mark != 0u) {
        let dissolve = fract(52.9829189*fract(dot(in.clip.xy, vec2<f32>(0.06711056, 0.00583715))));
        let new_half = dissolve < view.lod_prev.w;
        if (in.part_mark == 1u && !new_half) { discard; }
        if (in.part_mark == 2u && new_half) { discard; }
    }
    let m = water_bayer4(in.clip.xy);
    if (in.fade_t.z > m) { discard; }
    // The tier's water and the far river's sheet share its last 25 m by the
    // same dither, so the one gives way to the other with no seam (design 13).
    if (in.kind != 0u && view.column.w > 0.0) {
        let from_anchor = acos(clamp(dot(radial, safe_normal(view.column.xyz)), -1.0, 1.0))
            * length(in.body_position);
        let far = smoothstep(view.column.w - 25.0, view.column.w - 5.0, from_anchor);
        if (in.kind == 4u && far <= m) { discard; }
        if (in.kind != 4u && far > m) { discard; }
    }
    if (in.level > u32(view.lod.w)) {
        let owner_t = select(in.fade_t.y, in.fade_t.x, dot(radial, in.owner_a) >= dot(radial, in.owner_b));
        if (owner_t <= m) { discard; }
    }
    let face = safe_normal(in.normal);
    let sun_direction = safe_normal(view.sun.xyz);
    let time = view.camera_time.w;
    // Tangent frame off X or Y, projected off the face, exactly as Tenebris
    // builds it; the flow vector lives in this frame.
    var tangent_u = vec3<f32>(1.0,0.0,0.0);
    if (abs(face.x) > 0.9) { tangent_u = vec3<f32>(0.0,1.0,0.0); }
    tangent_u = safe_normal(tangent_u - face*dot(tangent_u,face));
    let tangent_v = cross(face,tangent_u);
    let horizontal = abs(dot(face,radial));
    // Flow: horizontal faces stream along the per-cell flow, vertical faces
    // scroll radially at the falling speed. A zero field is the unadvected form.
    // A still face of the game's water (kind 3) does not stream down.
    let falls = select(1.0, 0.0, in.kind == 3u);
    // A falling face (`water-flow` 6.6): its waves read down the face at
    // the speed-up's path, `flow_uv.z` its metres under the lip.
    let fall_face = in.kind == 2u && horizontal < 0.5;
    let under_lip = in.flow_uv.z;
    var rest = in.body_position;
    if (fall_face && view.fall_flow.w > 0.0) {
        rest += radial*(under_lip - fall_c(under_lip, view.deep_color.w));
    }
    // A top of the water near a foot takes its churn, ripples and flow.
    let pool_top = in.kind <= 1u && horizontal >= 0.5 && view.fall_look.x > 0.5;
    let feet_n = min(u32(view.fall_look.x), FALL_FEET);
    let advected = rest
        + radial*(time*view.deep_color.w*(1.0-horizontal)*falls)
        - (tangent_u*in.flow_uv.x + tangent_v*in.flow_uv.y)*time*0.35*horizontal;
    let p = advected*view.ripple.x;
    let t = time*view.ripple.y;
    // Not Tenebris's: once the fbm's features fall under a pixel, its
    // point-sampled height and gradient are noise, and Fresnel and specular
    // turn that noise into white sparkle across the whole far sea. Fade both
    // by the per-pixel footprint of the noise coordinate; `detail_fade` 0 is
    // the unfiltered original.
    let detail = 1.0/(1.0+length(fwidth(p))*max(view.screen.w,0.0));
    var height = fbm3(p,t)*detail;
    var gradient = vec3<f32>(fbm3(p+vec3<f32>(0.08,0.,0.),t)-fbm3(p,t),
        fbm3(p+vec3<f32>(0.,0.08,0.),t)-fbm3(p,t),
        fbm3(p+vec3<f32>(0.,0.,0.08),t)-fbm3(p,t))*12.5*detail;
    // The real seconds the mockup's pool terms run on.
    let seconds = time/max(view.aer_color.w, 1e-3);
    // The pool's flow away from a fall (M35, `uOut`): the waves and foam
    // drift outward from the column, by the flow-map method: two copies of
    // the waves, each dragged along the flow for 2 s and reset, cross-faded
    // so neither reset shows. Only within reach of a foot at its level.
    if (pool_top && view.fall_flow.y > 0.0) {
        var fl = vec3<f32>(0.0);
        for (var i = 0u; i < feet_n; i++) {
            let o = fall_foot_offset(i, in.body_position, radial);
            if (o.w < 0.0) { continue; }
            let d = length(o.xyz);
            let x = d - FALL_COLUMN_M;
            if (x <= -0.5 || x > view.fall_flow.z) { continue; }
            fl += o.xyz/max(d, 1e-3)*fall_out_speed(x, view.feet[i].w, view.fall_flow.z)*o.w;
        }
        fl *= view.fall_flow.y;
        if (dot(fl, fl) > 1e-6) {
            let period = 2.0;
            let a0 = fract(seconds/period);
            let a1 = fract(seconds/period + 0.5);
            let drag = fl*period*view.ripple.x;
            let p0 = p - drag*a0;
            let p1 = p - drag*a1 + vec3<f32>(0.37, 0.0, 0.61);
            let w0 = 1.0 - abs(2.0*a0 - 1.0);
            let w1 = 1.0 - w0;
            let g0 = fbm3(p0, t);
            let g1 = fbm3(p1, t);
            let gr0 = vec3<f32>(fbm3(p0 + vec3<f32>(0.08, 0.0, 0.0), t) - g0,
                fbm3(p0 + vec3<f32>(0.0, 0.08, 0.0), t) - g0, fbm3(p0 + vec3<f32>(0.0, 0.0, 0.08), t) - g0);
            let gr1 = vec3<f32>(fbm3(p1 + vec3<f32>(0.08, 0.0, 0.0), t) - g1,
                fbm3(p1 + vec3<f32>(0.0, 0.08, 0.0), t) - g1, fbm3(p1 + vec3<f32>(0.0, 0.0, 0.08), t) - g1);
            let m = clamp(length(fl)/0.3, 0.0, 1.0);
            height = mix(height, (g0*w0 + g1*w1)*detail, m);
            gradient = mix(gradient, (gr0*w0 + gr1*w1)*12.5*detail, m);
        }
    }
    gradient -= radial*dot(gradient,radial);
    // The rain's rings fade with their own footprint (`water-flow` "Rain
    // rings that alias"; owner, 2026-11-16: the river "covered in a dense
    // white speckle"): a ring is `sin(31 d)` in the kernel's uv, so this is
    // how many of its wavelengths one pixel spans. Whole while a ring spans
    // five pixels or more, gone by about two; past that the rings fell under
    // a pixel and point-sampled into per-pixel noise the mirror lit white.
    let ring_cycles = length(fwidth(in.body_position))*view.ripple.z*31.0/6.2831853;
    // And only within rain's fine detail's reach, as on the ground
    // (`wet-near-detail`): past it the water keeps its broad waves.
    let ring_reach = max(view.foam_crest.w, 0.0);
    let ring_near = 1.0 - smoothstep(ring_reach - max(view.foam_slope.w, 0.0), ring_reach,
        length(view.camera_time.xyz - in.local_position));
    let ring_detail = (1.0 - smoothstep(0.2, 0.45, ring_cycles))*ring_near;
    if (view.fx.z > 0.001 && ring_detail > 0.0) {
        let uv = vec2<f32>(dot(in.body_position,tangent_u),dot(in.body_position,tangent_v))*view.ripple.z;
        var g = rain_ripple_grad(uv,time)*(view.fx.z*view.ripple.w);
        if view.rain.w > 0.5 {
            // `lighting-model`: as many drops as it is raining, the terrain's
            // rule, at full strength each.
            g = rain_rings(uv,time,view.fx.z)*view.ripple.w;
        }
        // Only on water that faces up (owner, 2026-11-15: "when raining - no
        // droplets on vertical water"): a fall's or a wall's face takes none.
        g *= smoothstep(0.7, 0.95, horizontal)*ring_detail;
        gradient += tangent_u*g.x + tangent_v*g.y;
    }
    // Foot ripples (M35, `uRip`): rings run out from the falling column
    // across the pool, `fall_look.w` apart at 1.3 m/s, strongest at the
    // column and gone by the reach; they tilt the normal and whiten a little
    // on their crests near the column.
    var rip_foam = 0.0;
    if (pool_top && view.fall_look.z > 0.0) {
        let len = max(view.fall_look.w, 0.05);
        let reach_max = view.fall_flow.x;
        for (var i = 0u; i < feet_n; i++) {
            let o = fall_foot_offset(i, in.body_position, radial);
            if (o.w < 0.0) { continue; }
            let d = length(o.xyz);
            let x = d - FALL_COLUMN_M;
            if (x <= 0.0 || x > reach_max) { continue; }
            let h = view.feet[i].w;
            let k = 6.2831853/len;
            let ph = k*x - seconds*1.3*k;
            let reach = min(2.0 + 0.35*h, reach_max);
            let env = exp(-x/reach)*smoothstep(0.0, 0.25, x)*(1.0 - smoothstep(0.6*reach_max, reach_max, x))
                *(0.6 + 0.06*min(h, 10.0))*view.fall_look.z*o.w;
            let slope = cos(ph)*env*0.55*(0.9/len);
            gradient += o.xyz/max(d, 1e-3)*slope*12.5*0.25;
            rip_foam = max(rip_foam, smoothstep(0.55, 0.95, sin(ph))*env*exp(-x/1.2)*0.6);
        }
    }
    // The glassy lip (M35, `uGlass`): the top's small waves die away over the
    // last metres before the edge and the first part of the drop stays
    // smooth. On a top `flow_uv.z` is how near a lip its corner is, read as
    // metres before it over half a cell; on a face, metres under it.
    var glass = 0.0;
    if (view.fall_lip.x > 0.5) {
        let glass_top = max(view.fall_lip.y, 0.05);
        if (in.kind == 1u && horizontal >= 0.5 && in.flow_uv.z > 0.0) {
            let before = -(1.0 - clamp(in.flow_uv.z, 0.0, 1.0))*1.42;
            glass = smoothstep(-glass_top, -0.15*glass_top, before);
        }
        if (fall_face) {
            glass = 1.0 - smoothstep(0.17*view.fall_lip.z, max(view.fall_lip.z, 0.02), under_lip);
        }
        gradient *= 1.0 - 0.9*glass;
    }
    // Slope cap BEFORE the gradient bends the normal: the rain kernel spikes it
    // to ~8x the waves and, uncapped, flipped the sheet into a flat mirror.
    // Foam still reads the raw gradient so storm foam keeps its bite.
    let raw_slope = length(gradient);
    var capped = gradient*min(1.0,max(view.refraction.z,0.0)/max(raw_slope,1e-6));
    let normal = safe_normal(radial - in.sea_slope - capped*view.waves.w);
    let uv = in.clip.xy/vec2<f32>(textureDimensions(scene_depth));
    let own_depth = load_depth(uv);
    // Bevy/WebGPU reversed depth: larger depth is nearer.
    if (own_depth > in.clip.z + 1e-6) { discard; }
    let screen_gradient = (view.clip_from_local*vec4<f32>(gradient,0.0)).xy;
    var offset = screen_gradient*vec2<f32>(1.,-1.)*view.refraction.x;
    offset *= min(1.0,max(view.refraction.y,0.0)/max(length(offset),1e-6));
    var refracted_uv = clamp(uv+offset,vec2<f32>(0.),vec2<f32>(1.));
    var refracted_depth = load_depth(refracted_uv);
    let own_sky = own_depth <= 1e-7;
    let sampled_sky = refracted_depth <= 1e-7;
    // Reject a sample across the sky boundary or of geometry in front of the sheet.
    if (own_sky != sampled_sky || refracted_depth > in.clip.z + 1e-6) {
        refracted_uv = uv;
        refracted_depth = own_depth;
    }
    let scene = textureSampleLevel(scene_color,scene_sampler,refracted_uv,0.0).rgb;
    let to_camera = view.camera_time.xyz-in.local_position;
    let camera_distance = length(to_camera);
    let look = safe_normal(to_camera);
    var path_length = max(view.refraction.w,0.0);
    // How deep the water is under this pixel: the shallows' colour eases to
    // the deep's over `water_light.w` of it. Open sea seen to the sky is deep.
    var depth_under = 1.0e4;
    if (refracted_depth > 1e-7) {
        let background = reconstruct_local(refracted_uv,refracted_depth);
        path_length = min(path_length,max(length(background-view.camera_time.xyz)-camera_distance,0.0));
        depth_under = max(length(in.body_position)-length(background-view.planet_center.xyz),0.0);
    }
    // A wall of the game's water is a cell or two thick, however far the
    // ground behind it is: the light crosses that much water, not the view.
    if (in.kind >= 2u) { path_length = min(path_length, 2.0); depth_under = min(depth_under, 1.0); }
    let absorption = max(view.absorption.rgb,vec3<f32>(0.));
    // Seen from below: decided by where the camera is, not by the polygon's
    // winding. The swell tilts a 2.8 m cap by several degrees, more than the
    // grazing angle at the horizon, so winding alone could flip a cap seen
    // from above onto the underwater path. The pipeline culls nothing, so
    // `front` is only a tie-break for a camera exactly on the sheet.
    let camera_body = view.camera_time.xyz - view.planet_center.xyz;
    let camera_radius = length(camera_body);
    let sheet_radius = length(in.body_position);
    var below = camera_radius < sheet_radius || (camera_radius == sheet_radius && !front);
    // Past a few metres the sea is seen from the side of it the EYE is on
    // (`ocean-environments` B19): with the eye in the swell's reach of the
    // surface, a far crest over the eye's height was drawn as seen from
    // below, a dark band at the far waterline with the eye above the water.
    // Near the eye the wave under it still decides.
    let sea_eye = view.sea_frame.y > 0.0 && abs(view.sea_frame.y - view.planet_center.w) < SEA_EYE_RANGE_M;
    if (in.kind == 0u && sea_eye && camera_distance > SEA_SIDE_NEAR_M) {
        below = camera_radius < view.sea_frame.y;
    }
    // The game's own water has true faces: the camera is under one when it
    // is behind it.
    if (in.kind != 0u) {
        below = dot(view.camera_time.xyz - in.local_position, face) < 0.0;
        // Its back seen from dry air is the inside of a block of water the
        // eye is not in: the face toward the eye draws it, this one does not.
        // An eye at its own water's surface (the straddle) is under the
        // face it is below, and sees it from below: without it the surface
        // overhead vanished for the frames the eye crossed it (owner,
        // 2026-11-16: "I dont see water surface above the player").
        if (below && view.fx.y < 0.25) { discard; }
    }
    if (below) {
        // Seen from below: Snell's window and the murk (`ocean-environments`
        // B21). Inside the window, what is over the water, refracted, seen
        // through the water between; outside it the water's own murk, the
        // waves and the rain's rings only a gentle shading on it. Nothing of
        // the scene under the water is mirrored in the surface, at any
        // REFLECTIONS level (owner, 2026-11-16: "do we really need SSR from
        // bottom of the water?"): the mirror drew the sea floor upside down
        // over the waterline and streaked under the waves.
        let deep = murk(max(sheet_radius-camera_radius,0.0),safe_normal(camera_body));
        let underwater = mix(deep,scene,exp(-view_extinction()*camera_distance));
        // Only a face that faces up takes the waves' shading: the side face
        // of a step or a fall is where the water leaves, and stays the murk.
        var ceiling = deep;
        if (in.kind == 0u || horizontal >= 0.5) { ceiling = under_ceiling(deep, -look, normal, radial); }
        var seen = mix(ceiling,underwater,smoothstep(0.55,0.75,dot(-look,normal)));
        // The shafts and the snow between the eye and the sheet, as the
        // composite draws them in front of the seabed (`ocean-environments`
        // B5): the sheet is drawn over the composite, so it carries its own.
        seen += under_water_light(view.camera_time.xyz, -look, camera_distance);
        return vec4<f32>(seen,1.);
    }
    var reflected_ray = reflect(-look,normal);
    // A wall of the game's water (a fall, a sheet's edge) mirrors the sky as
    // the surface over it does, not the ground at its foot.
    if (in.kind != 0u && horizontal < 0.5) { reflected_ray = reflect(-look,radial); }
    let reflection_height = clamp(dot(reflected_ray,radial),0.,1.);
    let fresnel = (0.02+0.98*pow(1.0-max(dot(look,normal),0.0),5.0))
        * mix(clamp(view.shallow.w,0.,1.),1.,reflection_height);
    // A reflection is of the SKY, so it is the sky's own radiance along the
    // reflected ray (`water-lit`): grey under cloud, dark at night, warm at
    // dusk. It was two authored day blues and a night colour, saturated and
    // blind to the cover.
    var reflected = water_sky_radiance(water_sky_frame(reflected_ray,radial,sun_direction));
    // What the reflected ray really meets on the screen: the shore, a house,
    // the trees (`lighting-model` L8). Only where the mirror shows.
    if (fresnel > 0.02) {
        let hit = sea_ssr(in.body_position + radial*0.05, reflected_ray);
        reflected = mix(reflected, hit.rgb, hit.a);
    }
    // The water's own colour is an albedo lit by the light on open water, as
    // far as the sky reaches the cell (`water-lit`); what is seen through it
    // is already lit.
    let water_lit = view.water_light.rgb*clamp(in.sky_light,0.,1.);
    let albedo = mix(view.shallow.rgb,view.deep_color.rgb,smoothstep(0.0,max(view.water_light.w,1e-3),depth_under));
    var transmitted = mix(albedo*water_lit,scene,exp(-view_extinction()*path_length));
    // The churn over a fall's foot (M35, `uAer`): the bubbles under the
    // surface scatter the light back up, so the water there reads a lighter,
    // aerated teal and the eye sees less deep into it.
    if (pool_top && view.fall_look.y > 0.0) {
        let churn = gnoise3(vec3<f32>(dot(in.body_position, tangent_u)*1.3, time*0.9,
            dot(in.body_position, tangent_v)*1.3))*0.5 + 0.5;
        var aer = 0.0;
        for (var i = 0u; i < feet_n; i++) {
            let o = fall_foot_offset(i, in.body_position, radial);
            if (o.w < 0.0) { continue; }
            let r = 2.2 + 0.15*view.feet[i].w;
            aer = max(aer, (1.0 - smoothstep(0.8, r, length(o.xyz)))*(0.7 + 0.3*churn)*o.w);
        }
        transmitted = mix(transmitted, view.aer_color.rgb*water_lit, clamp(aer*0.85*view.fall_look.y, 0.0, 1.0));
    }
    // The glassy sheet over the edge is a thick body of water: it shows the
    // water's own green, not the wall behind it.
    if (fall_face && glass > 0.0) {
        transmitted = mix(transmitted, mix(view.shallow.rgb, view.deep_color.rgb, 0.35)*water_lit*1.3,
            clamp(view.fall_more.x*glass, 0.0, 1.0));
    }
    let crest = smoothstep(view.foam_crest.x,max(view.foam_crest.y,view.foam_crest.x+1e-4),height)*view.foam_crest.z;
    let slope = smoothstep(view.foam_slope.x,max(view.foam_slope.y,view.foam_slope.x+1e-4),raw_slope)*view.foam_slope.z;
    var foam = clamp(max(crest,slope)*view.foam_color.w + rip_foam,0.,1.);
    // A falling face of the game's water is white water: froth streaming down
    // its own advected waves (`water-flow` 7a, the mockups' fall froth).
    // White water on a fall (owner, 2026-11-15: "vertical water should have
    // a bit froth to it"): streaks of froth stretched down the face and
    // scrolled down it with the fall's own advection, denser at a fall's lip
    // and foot (the mesher's froth weight, `flow_uv.x` on a falling face).
    if (fall_face) {
        var down_p = advected;
        // With the speed-up the froth's streaks leave the lip at their own
        // (0.35 + 0.6)/0.35 m/s and speed up from it.
        if (view.fall_flow.w > 0.0) {
            down_p = in.body_position
                + radial*(under_lip - fall_c(under_lip, view.deep_color.w + 0.6/0.35))
                + radial*(time*view.deep_color.w);
        }
        let along = vec3<f32>(dot(down_p, tangent_u), dot(down_p, radial), dot(down_p, tangent_v));
        let streak_p = vec3<f32>(along.x*5.0, along.y*0.35, along.z*5.0) + vec3<f32>(0.0, time*0.6, 0.0);
        let n = gnoise3(streak_p)*0.5 + gnoise3(streak_p*2.3)*0.3 + 0.5;
        let weight = clamp(in.flow_uv.x, 0.35, 1.0);
        var froth = smoothstep(0.42, 0.7, n)*0.75 + 0.15;
        // Glassy: thin strands first, narrow long streaks with clear water
        // between, filling in to the full froth down the face.
        if (view.fall_lip.x > 0.5) {
            let sp = vec3<f32>(along.x*16.0, along.y*0.12, along.z*16.0) + vec3<f32>(0.0, time*0.9, 0.0);
            let strands = smoothstep(0.55, 0.8, gnoise3(sp)*0.5 + gnoise3(sp*2.1)*0.25 + 0.5);
            let fill_m = view.fall_lip.w;
            let fill = smoothstep(0.24*fill_m, max(fill_m, 0.05), under_lip);
            froth = mix(strands*0.85*smoothstep(0.22*view.fall_lip.z, 0.78*view.fall_lip.z, under_lip), froth, fill);
        }
        foam = max(foam, clamp(froth*weight + 0.12*(1.0 - glass), 0.0, 0.92));
    }
    // A gale and up (`ocean-environments` 4.3): foam torn into streaks along
    // the wind and spray off the crests, growing from `spray_from_mps` to
    // twice it. Streaks are the noise stretched twenty to one down the wind
    // sea's heading; the spray is the crest foam widened and whitened.
    let gale = clamp((view.under_more.y - view.under_more.z)/max(view.under_more.z,1.0), 0.0, 1.0)*view.under_more.w;
    let heading = view.sea.heading.xyz;
    if (gale > 0.0 && dot(heading, heading) > 0.5 && !below) {
        let along = dot(in.body_position, heading);
        let across = dot(in.body_position, safe_normal(cross(radial, heading)));
        let streak_p = vec3<f32>(along*0.035 - time*0.4, across*0.7, time*0.05);
        let streaks = smoothstep(0.42, 0.62, gnoise3(streak_p)*0.5 + gnoise3(streak_p*2.1)*0.25 + 0.5)*detail;
        let spray = smoothstep(view.foam_crest.x*0.6, view.foam_crest.y, height);
        foam = clamp(foam + gale*(0.55*streaks + 0.6*spray), 0.0, 1.0);
    }
    // Foam is lit as the water is; the glint is the sun as it reaches the
    // water, after the air and the cloud (`rain_sun`), gone at night.
    var color = mix(mix(transmitted,reflected,fresnel),view.foam_color.rgb*water_lit,foam);
    // Glassy's edge highlight (M35, `uEdgeHi`): the light a rounded lip would
    // catch, a soft band either side of the edge. Zero by default.
    if (view.fall_lip.x > 0.5 && view.fall_more.y > 0.0) {
        var e = 0.0;
        if (in.kind == 1u && horizontal >= 0.5 && in.flow_uv.z > 0.0) {
            e = 1.0 - smoothstep(0.0, 0.25, (1.0 - clamp(in.flow_uv.z, 0.0, 1.0))*1.42);
        }
        if (fall_face) { e = 1.0 - smoothstep(0.0, 0.3, under_lip); }
        let sky_up = water_sky_radiance(water_sky_frame(radial, radial, sun_direction));
        color = mix(color, mix(view.foam_color.rgb, sky_up, 0.4)*water_lit*1.15,
            clamp(e*e*0.6*view.fall_more.y, 0.0, 1.0));
    }
    let half_vector = safe_normal(look+sun_direction);
    color += view.sun_tint.rgb
        *pow(max(dot(normal,half_vector),0.),max(view.sun_tint.w,1.))
        *view.sun.w*view.rain_sun.rgb*clamp(in.sky_light,0.,1.);
    let fog = distance_fog(in.local_position,radial,sun_direction);
    let fogged = mix(color,fog.rgb,fog.a*clamp(in.sky_light,0.,1.));
    // The clouds over this water are the clouds pass's, drawn after the sheet
    // over everything against depth.
    return vec4<f32>(fogged,1.);
}

// ---- The sea's screen trace (`lighting-model` L8, 10.3) ----------------------
// The terrain's `ssr_trace`, over the same history: last frame's colour and
// distances, reprojected. The sea's ray is in the body frame.
fn sea_ssr_uv(p: vec3<f32>) -> vec3<f32> {
    let clip = view.ssr_clip*vec4<f32>(p, 1.0);
    if (clip.w <= 1e-4) { return vec3<f32>(-1.0); }
    return vec3<f32>(clip.x/clip.w*0.5 + 0.5, 0.5 - clip.y/clip.w*0.5, 1.0);
}
fn sea_ssr(origin: vec3<f32>, dir: vec3<f32>) -> vec4<f32> {
    if (view.ssr_eye.w < 0.5) { return vec4<f32>(0.0); }
    let size = vec2<f32>(textureDimensions(ssr_history));
    let home = sea_ssr_uv(origin).xy;
    var t = 0.1;
    var behind_before = -1.0;
    let reach = 120.0;
    // The march's last step lands 53.2 m out (t += 0.1 + 0.14 t, 32 times).
    // A straight ray projects to a straight line, so where its far end lands
    // within the three texels every step skips, so does every step: nothing
    // could be hit, and the trace returns its miss without marching
    // (`lighting-perf` 2.2). The far sea seen from low is that case.
    let far = sea_ssr_uv(origin + dir*min(53.3, reach));
    if (far.z > 0.0 && length((far.xy - home)*size) < 3.0) { return vec4<f32>(0.0); }
    for (var i = 0u; i < 32u; i++) {
        t += 0.1 + t*0.14;
        if (t > reach) { break; }
        let p = origin + dir*t;
        let at = sea_ssr_uv(p);
        if (at.z < 0.0 || any(at.xy < vec2<f32>(0.0)) || any(at.xy > vec2<f32>(1.0))) { break; }
        if (length((at.xy - home)*size) < 3.0) { continue; }
        let texel = textureLoad(ssr_history, vec2<i32>(at.xy*size), 0);
        if (texel.w <= 0.0) { break; }
        let behind = distance(p, view.ssr_eye.xyz) - texel.w;
        let thickness = clamp(0.4 + 0.08*t, 0.4, 3.0);
        if (behind > 0.0 && behind_before < 0.0 && behind < thickness) {
            let edge = min(min(at.x, 1.0 - at.x), min(at.y, 1.0 - at.y));
            return vec4<f32>(texel.rgb, smoothstep(0.0, 0.08, edge)*(1.0 - smoothstep(0.75*reach, reach, t)));
        }
        behind_before = behind;
    }
    return vec4<f32>(0.0);
}

// The surface seen from under it, outside Snell's window (`ocean-environments`
// B21): the water's own murk `deep`, shaded gently by the waves and the
// rain's rings. A facet tilted toward the eye faces nearer the window and
// takes a little more of its light, one tilted away a little less; at most
// `CEILING_SHADE` either way. `v` is the ray from the eye to the surface,
// `n` the surface's normal, `up` the radial there.
fn under_ceiling(deep: vec3<f32>, v: vec3<f32>, n: vec3<f32>, up: vec3<f32>) -> vec3<f32> {
    let along = v - up*dot(v, up);
    let toward = -safe_normal(along);
    let tilt = dot(n - up*dot(n, up), toward);
    return deep*(1.0 + clamp(tilt*CEILING_TILT_GAIN, -CEILING_SHADE, CEILING_SHADE));
}

// The most the waves brighten or darken the ceiling outside Snell's window,
// a share of the murk, and how much a radian of tilt moves it (B21).
const CEILING_SHADE: f32 = 0.12;
const CEILING_TILT_GAIN: f32 = 2.0;

// ---- The composite: compose before the cap, lens after it ------------------
// Tenebris's composite.fs.glsl. `fx.y` is the submersion tri-state decided on
// the CPU: 0 dry, 0.5 straddling the surface, 1 fully under.
struct FullscreenVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// 17-tap two-ring blur, radius growing with focus.
fn blur(uv: vec2<f32>, focus: f32) -> vec3<f32> {
    let r = focus*0.0016;
    var s = textureSampleLevel(scene_color,scene_sampler,uv,0.0).rgb;
    var w = 1.0;
    let taps = array<vec2<f32>,8>(vec2(r,0.),vec2(0.,r),vec2(r,r)*0.707,vec2(r,-r)*0.707,
        vec2(r*2.2,0.),vec2(0.,r*2.2),vec2(r*2.2,r*2.2)*0.707,vec2(r*2.2,-r*2.2)*0.707);
    for (var i = 0u; i < 8u; i++) {
        let d = taps[i];
        s += textureSampleLevel(scene_color,scene_sampler,clamp(uv+d,vec2(0.),vec2(1.)),0.0).rgb;
        s += textureSampleLevel(scene_color,scene_sampler,clamp(uv-d,vec2(0.),vec2(1.)),0.0).rgb;
        w += 2.0;
    }
    return s/w;
}

struct Ray { origin: vec3<f32>, direction: vec3<f32> }
// Reverse-Z clip depth of the point a pixel's direction is taken through:
// near/depth metres ahead, a kilometre at Bevy's 0.1 m near plane. The
// direction is that point minus the camera, both in f32 body-local metres, so
// it is only as good as the camera's position over the distance between them.
// At 0.5 (twenty centimetres) the camera's millimetre of rounding at orbital
// range turned every ray by about 0.005 rad, and the clouds, the sea and the
// rain were drawn in screen-aligned blocks seven pixels across.
const VIEW_RAY_DEPTH: f32 = 1.0e-4;

fn view_ray(uv: vec2<f32>) -> Ray {
    // A point far down the frustum gives the direction; sky pixels have no
    // depth to reconstruct at, so the direction never comes from one.
    let ahead = reconstruct_local(uv,VIEW_RAY_DEPTH);
    return Ray(view.camera_time.xyz, safe_normal(ahead-view.camera_time.xyz));
}

@fragment
fn compose(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let band = max(view.screen.y,1e-4);
    let depth = load_depth(uv);
    let sky = depth <= 1e-7;
    let ray = view_ray(uv);
    let center = view.planet_center.xyz;
    // The surface of the water the eye is in: a river's above the sea, or
    // the sea's sphere (`ocean-environments` B17).
    let sea = eye_water_surface();
    let under = view.fx.y > 0.75;
    let straddle = view.fx.y > 0.25 && !under;
    let cam_gap = max(0.0, sea - length(ray.origin-center));
    let ray_d_up = dot(ray.direction, safe_normal(ray.origin-center));
    var view_dist = 0.0;
    var pixel_alt = band;
    if (!sky) {
        let p = reconstruct_local(uv,depth);
        view_dist = length(p-ray.origin);
        pixel_alt = length(p-center)-sea;
    }
    // Wet: a heightfield has no geometry below the sea radius that is not
    // seabed, so the waterline mask alone is exact where Tenebris also needed
    // the terrain's own submerged flag. Sky pixels murk fully under, and while
    // straddling only along the near-level rays that travel metres of water.
    var wet = 1.0;
    if (sky) {
        let straddle_sky = select(0.0, 1.0-smoothstep(0.02,0.12,ray_d_up), cam_gap > 0.0);
        wet = select(select(0.0,straddle_sky,straddle),1.0,under);
    }
    // The waterline: a pixel is wet only below the surface, with a short
    // soft edge under it (owner, 2026-11-16: "underwater ground tint is
    // happening before the water line"). It reached `band` ABOVE the surface,
    // so the ground just over the water took the murk and the wobble; what
    // lies under a crest above the mean surface is behind the sheet, which
    // draws its own.
    //
    // With the eye fully under, EVERYTHING not hidden by a sheet is seen
    // through water (owner, 2026-11-16: "you must be submerged at just the
    // right height"): a river's water stands at a different height cell to
    // cell, so one waterline sphere left the walls under a higher neighbour's
    // water dry and bright, and at the sea's shore the bank between the
    // drawn sheet and the cell's top was seen through the gap past the
    // sheet's edge as a bright strip. What lies above the surface is drawn
    // by the sheet from below, from the scene before this pass.
    let mask = select(1.0-smoothstep(-WATERLINE_EDGE_M,0.0,pixel_alt),1.0,sky || under);
    wet *= mask;
    var color = textureSampleLevel(scene_color,scene_sampler,uv,0.0).rgb;
    if (wet > 0.0 && view.fx.x > 0.0) {
        let time = view.camera_time.w;
        let wobble = vec2<f32>(sin(time*1.3+uv.y*32.0)+0.5*sin(time*0.7+uv.x*47.0),
            cos(time*0.9+uv.x*28.0)+0.5*cos(time*1.6+uv.y*41.0));
        color = textureSampleLevel(scene_color,scene_sampler,clamp(uv+wobble*view.fx.x*wet,vec2(0.),vec2(1.)),0.0).rgb;
    }
    var travel: f32;
    if (under || straddle) {
        // Submerged sky: the exit distance off the mean sea sphere, saturating
        // for rays that never rise so the far water is murk and not a hole.
        travel = select(view_dist, select(100000.0, cam_gap/ray_d_up, ray_d_up > 0.0008), sky);
    } else {
        travel = max(0.0,-pixel_alt);
        // An eye a hand over the surface, looking down: the sheet nearer than
        // the near plane is clipped, and what is under it was seen dry
        // through the hole. There the light crosses the water from the
        // surface to the pixel (the waterline videos, 2026-11-16).
        let eye_over = length(ray.origin-center) - sea;
        if (!sky && pixel_alt < 0.0 && ray_d_up < -1e-4 && eye_over >= 0.0) {
            let t_cross = eye_over/(-ray_d_up);
            if (t_cross < NEAR_SHEET_M) {
                travel = max(travel, view_dist - t_cross);
            }
        }
    }
    let extinction = view_extinction();
    let deep = murk(cam_gap,safe_normal(ray.origin-center));
    color = mix(color, mix(deep,color,exp(-extinction*travel)), wet);
    // The sheet the near plane clips, for an eye under the surface (B19):
    // a ray that crosses the surface nearer than `NEAR_SHEET_M` saw the sky
    // nearly clear and the bank above as a silhouette through the hole,
    // bright bands over the dark ones. Shade it as the sheet would: the
    // murk outside Snell's window, the view through it inside.
    var light_reach = select(view_dist, 1.0e4, sky);
    if (cam_gap > 0.0 && ray_d_up > 1e-4) {
        let t_cross = cam_gap/ray_d_up;
        if (t_cross < NEAR_SHEET_M && (sky || view_dist > t_cross)) {
            let raw = textureSampleLevel(scene_color,scene_sampler,uv,0.0).rgb;
            let through = mix(deep, raw, exp(-extinction*t_cross));
            // Outside the window the murk, as the sheet draws it (B21).
            color = mix(deep, through, smoothstep(0.55,0.75,ray_d_up));
            light_reach = t_cross;
        }
    }
    // From under the water: the sun's shafts and the marine snow
    // (`ocean-environments` B5), both over the murk, both for the eye in it.
    if (under && wet > 0.0) {
        color += under_water_light(ray.origin, ray.direction, light_reach)*wet;
    }
    // Depth blur while the lens is wet: the distance softens, the foreground stays crisp.
    let dz_amt = max(view.rain.x,view.fx.w);
    if (dz_amt > 0.001 && !under) {
        let dz = smoothstep(28.0,190.0,view_dist)*clamp(dz_amt,0.0,1.0);
        if (dz > 0.002) { color = mix(color, blur(uv,dz*3.5), dz*clamp(view.screen.z,0.0,1.0)); }
    }
    return vec4<f32>(color,1.0);
}

// ---- Rain on glass: "Heartfelt" by Martijn Steinrucken (BigWings), 2017, CC
// BY-NC-SA 3.0, as Tenebris carries it. Refraction only, so the distance fog
// between drops is preserved.
fn rd_n13(p: f32) -> vec3<f32> {
    var p3 = fract(vec3<f32>(p)*vec3<f32>(0.1031,0.11369,0.13787));
    p3 += dot(p3,p3.yzx+19.19);
    return fract(vec3<f32>((p3.x+p3.y)*p3.z,(p3.x+p3.z)*p3.y,(p3.y+p3.z)*p3.x));
}
fn rd_n(t: f32) -> f32 { return fract(sin(t*12345.564)*7658.76); }
fn rd_saw(b: f32, t: f32) -> f32 { return smoothstep(0.0,b,t)*smoothstep(1.0,b,t); }
// x the main drop, y its trail's beads (apart for the lens's radii,
// `lens-drop-refraction`), z the trail.
fn rd_drop_layer(uv_in: vec2<f32>, t: f32) -> vec3<f32> {
    var uv = uv_in;
    let whole = uv_in;
    uv.y += t*0.75;
    let a = vec2<f32>(6.0,1.0);
    let grid = a*2.0*max(view.lens.w,0.05);
    var id = floor(uv*grid);
    uv.y += rd_n(id.x);
    id = floor(uv*grid);
    let n = rd_n13(id.x*35.2+id.y*2376.1);
    let keep = step(1.0-clamp(view.lens.x,0.0,1.0), fract(n.x*3.17+n.y*6.91+n.z));
    let st = fract(uv*grid)-vec2<f32>(0.5,0.0);
    var x = n.x-0.5;
    var y = whole.y*20.0;
    let wiggle = sin(y+sin(y));
    x += wiggle*(0.5-abs(x))*(n.z-0.5);
    x *= 0.7;
    let ti = fract(t+n.z);
    // Monotonic fall, faded at both ends so the wrap is never a flick upward.
    y = 0.92-ti*0.88;
    let fade = smoothstep(0.0,0.12,ti)*smoothstep(1.0,0.86,ti);
    let p = vec2<f32>(x,y);
    let d = length((st-p)*a.yx);
    let main_drop = smoothstep(0.4,0.0,d);
    let r = sqrt(smoothstep(1.0,y,st.y));
    let cd = abs(st.x-x);
    var trail = smoothstep(0.23*r,0.15*r*r,cd);
    let trail_front = smoothstep(-0.02,0.02,st.y-y);
    trail *= trail_front*r*r;
    y = whole.y;
    let trail2 = smoothstep(0.2*r,0.0,cd);
    var droplets = max(0.0,(sin(y*(1.0-y)*120.0)-st.y))*trail2*trail_front*n.z;
    y = fract(y*10.0)+(st.y-0.5);
    let dd = length(st-vec2<f32>(x,y));
    droplets = smoothstep(0.3,0.0,dd);
    return vec3<f32>(main_drop,droplets*r*trail_front,trail)*(keep*fade);
}
fn rd_static_drops(uv_in: vec2<f32>, t: f32) -> f32 {
    var uv = uv_in*40.0*max(view.lens.w,0.05);
    let id = floor(uv);
    uv = fract(uv)-0.5;
    let n = rd_n13(id.x*107.45+id.y*3543.654);
    let keep = step(1.0-clamp(view.lens.x,0.0,1.0), fract(n.x*3.17+n.y*6.91+n.z));
    let p = (n.xy-0.5)*0.7;
    let d = length(uv-p);
    let fade = rd_saw(0.025,fract(t+n.z));
    return smoothstep(0.3,0.0,d)*fract(n.z*10.0)*fade*keep;
}
// Each kind of drop's radius in the drop space at `rain_lens_size` 1, where
// its field reaches zero: the static drop's 0.3 of a 1/40 cell; a layer's
// drop's 0.4 of a 1/12 cell (its distance the same across and down), and its
// beads' 0.3 of it across. The far layer's are 1.85 times smaller.
const RD_RADII: vec3<f32> = vec3<f32>(0.3/40.0, 0.4/12.0, 0.3/12.0);
// The drop field at a point (`lens-drop-refraction`): x the coverage the drops
// add up to (a pixel shows a drop where it passes 0.3), y the lens height, z
// the trail. The height weighs each kind of drop by the square of its radius,
// and the static drops by their height over `l0`'s (it reaches 2), so every
// drop's slope is the same share of its own size.
fn rd_drop_parts(uv: vec2<f32>, t: f32, l0: f32, l1: f32, l2: f32) -> vec3<f32> {
    let s = rd_static_drops(uv,t)*l0;
    var near = rd_drop_layer(uv,t)*l1;
    var far = rd_drop_layer(uv*1.85,t)*l2;
    let trail = max(near.z*l0,far.z*l1);
    // Density above one stacks same-size layers rather than shrinking drops.
    let extra = clamp(view.lens.x-1.0,0.0,2.0);
    if (extra > 0.001) {
        near += rd_drop_layer(uv+vec2<f32>(0.50,0.37),t)*(l1*min(extra,1.0));
        far += rd_drop_layer(uv*1.85+vec2<f32>(0.27,0.61),t)*(l2*min(extra,1.0));
    }
    let r = RD_RADII/max(view.lens.w,0.05);
    let w = r*r;
    let h = w.x*s/max(l0,1.0) + dot(near.xy,w.yz) + dot(far.xy,w.yz)/(1.85*1.85);
    return vec3<f32>(s+near.x+near.y+far.x+far.y, h, trail);
}
// The lens's uv (y down) in the reference's frame: centred, y up, in screen
// heights. Tenebris's drop space; the lens multiplies it by `rain_lens_scale`.
fn rd_space(uv: vec2<f32>) -> vec2<f32> {
    return vec2<f32>((uv.x-0.5)*max(view.screen.x,0.1), 0.5-uv.y);
}
// A lens pixel through the drops (`lens-drop-refraction`): where it looks,
// as an offset in the lens's uv; the drop field there (`rd_drop_parts`); and
// how far the drop's surface there faces down, 0..1, from its slope.
struct RdLens {
    offset: vec2<f32>,
    field: vec3<f32>,
    lower: f32,
}
// The slope of the lens height bends a pixel toward its drop's centre: every
// drop, whatever its size, by `rain_lens_bend` times the pixel's distance from
// the centre at its middle, and by none at its rim. Below 1 an upright
// magnifier; up to 2 the sample never leaves the drop. Across a cell's seam
// the field steps; no pixel in a drop bends further than the bend times the
// largest drop's radius, so that caps it.
fn rd_lens(drop_uv: vec2<f32>, t: f32, l0: f32, l1: f32, l2: f32) -> RdLens {
    let e = 0.001;
    let here = rd_drop_parts(drop_uv,t,l0,l1,l2);
    let across = rd_drop_parts(drop_uv+vec2<f32>(e,0.0),t,l0,l1,l2);
    let up = rd_drop_parts(drop_uv+vec2<f32>(0.0,e),t,l0,l1,l2);
    let slope = vec2<f32>(across.y-here.y, up.y-here.y)/e;
    let bend = clamp(view.lens.y,0.0,2.0);
    // A unit bump's R^2 slope is 6 u(1-u) R: a sixth of it is (1-u) times
    // the distance from the centre.
    var o = slope*(bend/6.0);
    let reach = bend*RD_RADII.y/max(view.lens.w,0.05);
    let len = length(o);
    if (len > reach) { o *= reach/len; }
    var lens: RdLens;
    // Back from the drop space to the uv: undo the scale, the aspect and the
    // flip of `rd_space`.
    lens.offset = vec2<f32>(o.x/max(view.screen.x,0.1), -o.y)/max(view.mist_look.w,0.05);
    lens.field = here;
    // The slope points up the drop, toward its middle: up at its lower edge.
    lens.lower = clamp(slope.y*inverseSqrt(max(dot(slope,slope),1e-20)),0.0,1.0);
    return lens;
}
// `wipe` receives how much of the glass the drops and their trails have run
// over here: what clears the mist (`lens-weather`).
fn refract_through_drops(uv: vec2<f32>, drop_uv: vec2<f32>, t: f32, l0: f32, l1: f32, l2: f32,
        wipe: ptr<function, f32>) -> vec4<f32> {
    let lens = rd_lens(drop_uv,t,l0,l1,l2);
    let c = smoothstep(0.3,1.0,lens.field.x);
    // The edge (`lens-drop-refraction`): where the drop's alpha passes a half
    // its surface is steep, and light is lost there; what shows a drop against
    // a flat sky. The sky's light, bent through the drop, gathers on the
    // lower part of that edge: the sky's mean radiance, so it goes with the
    // day. Nothing at a drop's middle, nothing outside it.
    let band = 4.0*c*(1.0-c);
    let rim = clamp(view.lens_look.x,0.0,1.0);
    let glint = max(view.lens_look.y,0.0);
    let behind = textureSampleLevel(scene_color,scene_sampler,clamp(uv+lens.offset,vec2(0.),vec2(1.)),0.0).rgb;
    let refracted = behind*(1.0-rim*band) + view.rain_sky.rgb*(glint*band*lens.lower);
    *wipe = max(*wipe, clamp(max(c, lens.field.z), 0.0, 1.0));
    return vec4<f32>(refracted, clamp(c,0.0,1.0));
}

// The mist's beads: a field of tiny drops, `mist_look.x` per screen height,
// each a round bead where its cell keeps one (`lens-weather`).
fn mist_beads(drop_uv: vec2<f32>) -> f32 {
    let g = drop_uv*max(view.mist_look.x, 1.0);
    let id = floor(g);
    let f = fract(g)-0.5;
    let n = rd_n13(id.x*57.13+id.y*913.71);
    let d = length(f-(n.xy-0.5)*0.6);
    return smoothstep(0.35,0.0,d)*step(0.35,n.z);
}

// The lens mist, per frame, one pixel (`lens-weather`): the one cloud density
// at the eye, the march's own, eased toward over `fog_s` as it rises and
// `clear_s` as it falls, both faster with airspeed. None under water, and none
// after a jump. The lens draws it.
// The cloud densities at which the lens starts to mist and is fully misted.
const LENS_MIST_THIN: vec2<f32> = vec2<f32>(0.01, 0.12);

@fragment
fn lens_mist_pass(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let previous = textureLoad(lens_mist_prev, vec2<i32>(0), 0).r;
    if (view.fx.y > 0.75 || view.cloud_history.y < 0.5) { return vec4<f32>(0.0); }
    let eye = view_ray(vec2<f32>(0.5)).origin-view.planet_center.xyz;
    let layer = cloud_layer();
    let at = cloud_lookup(eye);
    let local = cloud_local(at, weather_cloud, weather_sampler);
    var target_mist = 0.0;
    if (local.x > 0.01) {
        let wind = cloud_wind(at, weather_wind, weather_sampler);
        // Any cloud mists a lens: the target saturates at a thin cloud's
        // density. Taken as the density itself, the lens barely misted in the
        // middle of a storm deck, whose density at the eye was 0.1-0.4.
        let density = cloud_density(eye, layer, local, wind, 0.25*max(layer.cells.y, 0.5),
            cloud_cells_tex, cloud_cells_sampler);
        target_mist = smoothstep(LENS_MIST_THIN.x, LENS_MIST_THIN.y, density);
    }
    let seconds = select(view.mist.y, view.mist.x, target_mist > previous);
    let rate = (1.0 + max(view.rain.z, 0.0)*view.mist.z)/max(seconds, 1e-3);
    let mist = target_mist + (previous-target_mist)*exp(-max(view.rain.y, 0.0)*rate);
    return vec4<f32>(clamp(mist, 0.0, 1.0), 0.0, 0.0, 1.0);
}

@fragment
fn lens(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    var color = textureSampleLevel(scene_color,scene_sampler,uv,0.0).rgb;
    let under = view.fx.y > 0.75;
    if (under) { return vec4<f32>(color,1.0); }
    // Straddling, drops bead on everything at or above the on-screen
    // waterline, split by ray direction rather than per-pixel seabed altitude.
    var above_water = 1.0;
    if (view.fx.y > 0.25) {
        let ray = view_ray(uv);
        above_water = smoothstep(-0.12,-0.02,dot(ray.direction,safe_normal(ray.origin-view.planet_center.xyz)));
    }
    // The drop math is the reference's, whose full-screen v_uv is y-UP ("high
    // y = high on screen"); Bevy's fullscreen uv is y-down, and in it the same
    // monotonic fall climbed. The lens works in the reference's frame.
    let tenebris_uv = rd_space(uv);
    // The drops `rain_lens_scale` times smaller than Tenebris's, trails and
    // beads with them (`lens-weather`).
    let drop_uv = tenebris_uv*max(view.mist_look.w, 0.05);
    // The drops' own clock (`weather-rain` R3): it runs while rain lands on
    // the glass and stops while none does, so a drying lens grows no new
    // drop and the ones on it stay where they are as they shrink and fade.
    let t = view.lens_look.z*0.2*view.lens.z;
    // How wet the lens is, not the rain itself: it builds up over a few
    // seconds in rain and dries off over a few out of it (owner, 2026-11-16:
    // "fade out instead of abruptly cut", "gradually increase as well"). The
    // layers thin with it, the drops shrink with it, and the whole fades.
    let rain = clamp(view.rain.x,0.0,1.0);
    var wipe = 0.0;
    var drops = vec4<f32>(0.0);
    if (rain > 0.001) {
        let s = smoothstep(-0.5,1.0,rain)*2.0;
        let l1 = smoothstep(0.25,0.75,rain);
        let l2 = smoothstep(0.0,0.5,rain);
        drops = refract_through_drops(uv,drop_uv,t,s,l1,l2,&wipe);
        wipe *= rain;
    }
    // Condensation (`lens-weather`): a film of fine beads over a softened,
    // lifted image, where the probe's mist is thick enough; it clears from
    // the screen's edges inward, in patches, and the drops and their trails
    // cut clear runs through it. Under the drops, which draw over it.
    let mist = textureLoad(lens_mist, vec2<i32>(0), 0).r*view.mist.w;
    if (mist > 0.001) {
        let aspect = max(view.screen.x,0.1);
        let centred = vec2<f32>(tenebris_uv.x/aspect, tenebris_uv.y)*2.0;
        let edge = clamp(length(centred)/1.4142, 0.0, 1.0);
        let blotch = cloud_noise(vec3<f32>(tenebris_uv*5.0, 3.7));
        let threshold = 0.08 + 0.55*edge + 0.25*(blotch-0.5);
        let cover = smoothstep(threshold-0.1, threshold+0.1, mist);
        let film = blur(uv, view.mist_look.y/0.0016);
        let bright = dot(film, vec3<f32>(0.2126,0.7152,0.0722));
        let lifted = mix(film, vec3<f32>(bright)*1.15, view.mist_look.z);
        let beaded = lifted*(1.0 + 0.45*mist_beads(tenebris_uv));
        color = mix(color, beaded, cover*min(mist*1.5, 1.0)*above_water*(1.0-wipe));
    }
    if (rain > 0.001) {
        color = mix(color,drops.rgb,drops.a*rain*above_water);
    }
    let emerge = clamp(view.fx.w,0.0,1.0);
    if (emerge > 0.001) {
        var dry = 0.0;
        let r = refract_through_drops(uv,drop_uv,view.camera_time.w*0.2*view.lens.z,0.0,1.0,0.6,&dry);
        color = mix(color,r.rgb,r.a*emerge*above_water);
    }
    return vec4<f32>(color,1.0);
}

// ---- Rain as a volume ---------------------------------------------------------
// Rain seen from a distance, marched per pixel from the eye to the nearer of the
// ground, the sea and the cloud base, off the precipitation map the CPU fills
// from the weather field. Drawn after the sea, so it stands in front of the
// ground and the water; stopped at the cloud base, so from below the clouds it
// is nearer than every cloud it falls from. Streaks come from gradient noise
// stretched along the vertical and scrolled down at the fall speed, which is
// what makes the fog fall. See `openspec/changes/storm`.

// The map at a body-local point: bilinear, zero off its edge, negative where
// it snows. The plane is tangent at the anchor; a point is carried onto it
// gnomonically, exactly as the CPU filled it.
fn rain_map_at(p: vec3<f32>) -> f32 {
    let size = view.rain_u.w;
    if (size < 2.0) { return 0.0; }
    let d = safe_normal(p);
    let c = dot(d,view.rain_map.xyz);
    if (c < 0.5) { return 0.0; }
    let on_plane = d*(view.planet_center.w/c);
    let cell = max(view.rain_map.w,1.0);
    let x = dot(on_plane,view.rain_u.xyz)/cell+size*0.5-0.5;
    let y = dot(on_plane,view.rain_v.xyz)/cell+size*0.5-0.5;
    if (x < 0.0 || y < 0.0 || x > size-1.0 || y > size-1.0) { return 0.0; }
    let n = u32(size);
    let x0 = u32(floor(x)); let y0 = u32(floor(y));
    let x1 = min(x0+1u,n-1u); let y1 = min(y0+1u,n-1u);
    let fx = fract(x); let fy = fract(y);
    let a = mix(rain_map[y0*n+x0],rain_map[y0*n+x1],fx);
    let b = mix(rain_map[y1*n+x0],rain_map[y1*n+x1],fx);
    return mix(a,b,fy);
}

// ---- The clouds, over everything ------------------------------------------
// `pbd::clouds` marched along each pixel's ray from the eye to the nearest of
// the scene's depth and the sea, and composited over what is there: the sky,
// the sea and the land alike. A cloud in front of a hill is in front of it.
// Two passes (`cloud-budget`). The MARCH runs at `cloud_render_scale` of the
// view and writes the cloud alone (light premultiplied by coverage, and the
// coverage), blended with the previous frame's: that target is the history.
// The COMPOSITE runs at full resolution and lays it over the scene.

// The farthest scene depth over a march texel's footprint, as a distance
// along the ray; the sea's hit included. Reverse-Z: the farthest is the
// smallest depth, and zero is the sky. A march texel on a hill's silhouette
// then holds the cloud its sky side sees, and the composite's own span test
// keeps it off the hill.
// The sea's hit is taken over the same four corners, the farthest stop
// winning: taken on the centre ray alone, a texel on the sea's horizon flipped
// between stopping at the sea and seeing the sky as the eye moved, and failed
// the history's test each time it flipped (`cloud-entry`).
fn cloud_far(ray: Ray, uv: vec2<f32>, half_texel: vec2<f32>, eye: vec3<f32>) -> f32 {
    var far = 0.0;
    for (var k = 0u; k < 4u; k++) {
        let corner = vec2<f32>(select(-1.0, 1.0, (k & 1u) == 1u), select(-1.0, 1.0, (k & 2u) == 2u));
        let at = clamp(uv + corner*half_texel, vec2<f32>(0.0), vec2<f32>(1.0));
        let depth = load_depth(at);
        var stop = 1.0e9;
        if (depth > 1e-7) {
            stop = length(reconstruct_local(at,depth)-ray.origin);
        }
        let sea = cloud_sphere_hit(eye,view_ray(at).direction,view.planet_center.w);
        if (sea.x > 0.0) { stop = min(stop,sea.x); }
        far = max(far, stop);
    }
    return far;
}

// Whether a march texel saw what a ray stopping at `far` sees: its cloud lies
// no further than `far`, and its march went at least as far. Both fail across
// a silhouette: a texel on the far side of an edge either stopped on the near
// object or holds cloud beyond it. The composite's upsample asks it within a
// frame, the history read across frames (`cloud-budget`, `cloud-ghosting`).
fn cloud_texel_sees(texel: vec2<f32>, far: f32) -> bool {
    return cloud_texel_sees_to(texel, far, far);
}

// The history's form of the test (`cloud-edges`): its cloud lies no further
// than this ray's stop `far`, and its march reached `reach`, where the cloud
// being read is. Asking it to reach this ray's ground instead refused a near
// cloud's history for the parallax of the ground far behind it: entering a
// cloud, every texel over land and sea fell back to one raw sample. An
// occluder in front of the cloud still stopped the march short of `reach`.
fn cloud_texel_sees_to(texel: vec2<f32>, far: f32, reach: f32) -> bool {
    let near_enough = cloud_texel_distance(texel.x) <= far*1.03 + 10.0;
    let far_enough = texel.y >= min(min(reach, far), CLOUD_FAR_CAP_M)*0.97 - 10.0;
    return near_enough && far_enough;
}

// A texel's cloud distance, zero for none. The puffs' pass clears its
// distances to the far cap and keeps the nearest puff (`cloud-puffs`), so the
// cap is "no cloud" there as zero is in the march's.
fn cloud_texel_distance(x: f32) -> f32 {
    return select(x, 0.0, x >= CLOUD_FAR_CAP_M*0.999);
}

// The previous frame's cloud at `was` on its screen, for a ray this frame
// stopping at `stop`: the four history texels round it, bilinearly, each kept only when
// its march saw what this ray sees, measured from the previous eye. w of the
// result is the kept weight; zero means no texel qualifies and this frame
// stands alone. A plain bilinear read carried cloud across every silhouette:
// sky revealed from behind a hill or the held tool faded its cloud back in
// from the empty texel it had been, and cloud beyond a new occluder was drawn
// over it (`cloud-ghosting`).
struct CloudPrevious {
    cloud: vec4<f32>,
    weight: f32,
    // Where the kept texels' cloud is along their rays, weighted by the cloud
    // each holds; zero where none holds a distance (`cloud-edges`).
    distance: f32,
}

// `point` is where along this ray the history is read (the cloud's point),
// which the previous march must have reached (`cloud_texel_sees_to`).
fn cloud_previous(was: vec2<f32>, stop: vec3<f32>, point: vec3<f32>) -> CloudPrevious {
    var out: CloudPrevious;
    out.cloud = vec4<f32>(0.0);
    out.weight = 0.0;
    out.distance = 0.0;
    let size = vec2<i32>(textureDimensions(cloud_history));
    let at = was*vec2<f32>(size) - 0.5;
    let base = vec2<i32>(floor(at));
    let f = at - floor(at);
    let far = distance(view.cloud_prev_eye.xyz, stop);
    let reach = distance(view.cloud_prev_eye.xyz, point);
    var sum = vec4<f32>(0.0);
    var at_sum = 0.0;
    var at_weight = 0.0;
    for (var k = 0u; k < 4u; k++) {
        let o = vec2<i32>(i32(k & 1u), i32((k >> 1u) & 1u));
        let texel = base + o;
        if (any(texel < vec2<i32>(0)) || any(texel >= size)) { continue; }
        let w = select(1.0-f.x, f.x, o.x == 1)*select(1.0-f.y, f.y, o.y == 1);
        let depth = textureLoad(cloud_depth, texel, 0).rg;
        if (!cloud_texel_sees_to(depth, far, reach)) { continue; }
        let cloud = textureLoad(cloud_history, texel, 0);
        sum += cloud*w;
        out.weight += w;
        let holds = w*cloud.w*select(0.0, 1.0, depth.x > 0.0);
        at_sum += depth.x*holds;
        at_weight += holds;
    }
    if (out.weight > 1e-3) { out.cloud = sum/out.weight; }
    if (at_weight > 1e-6) { out.distance = at_sum/at_weight; }
    return out;
}

// The previous screen position of a point, or a negative x where it was
// behind the previous camera or off its screen.
fn cloud_was(point: vec3<f32>) -> vec2<f32> {
    let before = view.prev_clip_from_local*vec4<f32>(point,1.0);
    if (before.w <= 0.0) { return vec2<f32>(-1.0); }
    let ndc = before.xy/before.w;
    let was = vec2<f32>(ndc.x*0.5+0.5, 0.5-ndc.y*0.5);
    if (any(was < vec2<f32>(0.0)) || any(was > vec2<f32>(1.0))) { return vec2<f32>(-1.0); }
    return was;
}

struct CloudMarchOut {
    // This frame's cloud alone: its light premultiplied by coverage, and
    // coverage. The resolve folds it into the history.
    @location(0) cloud: vec4<f32>,
    // How far along the ray this frame's cloud is, metres, zero for none; and
    // how far the march could see (the scene, the sea), capped to stay a
    // finite half float.
    @location(1) depth: vec2<f32>,
    // The near field's fog: its lit colour, and its extinction per metre
    // (`cloud-entry`).
    @location(2) near: vec4<f32>,
}

@fragment
fn cloud_march_pass(in: FullscreenVertexOutput) -> CloudMarchOut {
    let uv = in.uv;
    // The angle one pixel of THIS target spans, off the ray's own screen
    // derivative: what fades the clouds' finer octaves as they fall under a
    // pixel. Taken before any branch, where a derivative is defined.
    let ray = view_ray(uv);
    let pixel_angle = length(fwidth(ray.direction));
    let half_texel = 0.5*fwidth(uv);
    var out: CloudMarchOut;
    out.cloud = vec4<f32>(0.0);
    out.depth = vec2<f32>(0.0, CLOUD_FAR_CAP_M);
    out.near = vec4<f32>(0.0);
    // Under the sea the surface is the sky; the compose pass drew it.
    if (view.fx.y > 0.75) { return out; }
    let eye = ray.origin-view.planet_center.xyz;
    let layer = cloud_layer();
    let far = cloud_far(ray, uv, half_texel, eye);
    out.depth.y = min(far, CLOUD_FAR_CAP_M);
    let span = cloud_span(eye,ray.direction,0.0,far,layer);
    // No cloud to march: an empty history, so a cloud never ghosts over a
    // hill that has moved in front of it.
    if (span.y <= span.x) { return out; }
    let sun = safe_normal(view.sun.xyz);
    // The first metres from the eye are the cloud's own fog, unjittered and
    // outside the history; the march takes the rest (`cloud-entry`).
    let slab_end = min(cloud_near_reach(layer), span.y);
    if (slab_end > span.x) {
        out.near = cloud_near_fog(eye,ray.direction,span.x,slab_end,layer,sun,
            weather_cloud,weather_wind,weather_sampler,cloud_cells_tex,cloud_cells_sampler);
    }
    // The puffs draw the cloud (`cloud-puffs`): this pass makes only the
    // near field's fog, which carries the eye into a cloud as before.
    if (view.cloud_puff_fade.w > 0.5) { return out; }
    // A different step offset for every pixel AND every frame, so the
    // history averages samples at new depths: interleaved gradient noise,
    // moved each frame. White noise left a 3x3 block short of the range the
    // resolve clips the history to, and the clip then held noise
    // (`cloud-entry`).
    let q = in.position.xy + 5.588238*f32(u32(view.cloud_history.z) % 64u);
    let jitter = fract(52.9829189*fract(dot(q, vec2<f32>(0.06711056, 0.00583715))));
    let marched = cloud_march(eye,ray.direction,max(span.x, slab_end),span.y,layer,sun,
        weather_cloud,weather_wind,weather_sampler,cloud_cells_tex,cloud_cells_sampler,
        jitter,pixel_angle);
    // This frame's cloud alone: the resolve folds it into the history
    // (`cloud-history-clip`).
    out.cloud = marched.cloud;
    out.depth.x = select(0.0, marched.depth, marched.cloud.w > 1e-3);
    return out;
}

// The history's resolve (`cloud-history-clip`): what the march's blend did,
// plus a clip. The previous history is read where this texel's cloud was a
// frame ago through `cloud-ghosting`'s test (`cloud_previous`: only texels
// whose march saw what this ray sees), then held inside the range this
// frame's march shows round the texel: the mean plus or minus 1.25 standard
// deviations of the 3 x 3 neighbourhood (Salvi 2016), widened to include this
// texel. The test catches a silhouette; the clip catches a history that saw
// the same depths and is still wrong: a cloud spread along the whole ray,
// which one depth cannot reproject, or a cloud that has changed. A converged
// history lies inside a single noisy sample's neighbourhood, so still cloud
// is left as calm as before.
const CLOUD_CLIP_SIGMA: f32 = 1.25;
struct CloudResolveOut {
    // The history.
    @location(0) cloud: vec4<f32>,
    // Its distances, as the march writes them; a texel whose cloud is the
    // history's alone keeps the history's (`cloud-entry`, `cloud-edges`).
    @location(1) depth: vec2<f32>,
}
@fragment
fn cloud_resolve(in: FullscreenVertexOutput) -> CloudResolveOut {
    let size = vec2<i32>(textureDimensions(cloud_current));
    let texel = clamp(vec2<i32>(in.position.xy), vec2<i32>(0), size - 1);
    let current = textureLoad(cloud_current, texel, 0);
    let now = textureLoad(cloud_depth_now, texel, 0).rg;
    var out: CloudResolveOut;
    out.cloud = current;
    out.depth = now;
    if (view.cloud_history.y < 0.5) { return out; }
    var m1 = vec4<f32>(0.0);
    var m2 = vec4<f32>(0.0);
    for (var dy = -1; dy <= 1; dy++) {
        for (var dx = -1; dx <= 1; dx++) {
            let s = textureLoad(cloud_current, clamp(texel + vec2<i32>(dx, dy), vec2<i32>(0), size - 1), 0);
            m1 += s;
            m2 += s*s;
        }
    }
    let mean = m1/9.0;
    let sigma = sqrt(max(m2/9.0 - mean*mean, vec4<f32>(0.0)));
    let lo = min(mean - CLOUD_CLIP_SIGMA*sigma, current);
    let hi = max(mean + CLOUD_CLIP_SIGMA*sigma, current);
    // The cloud's own point, or where the march stopped where it has none,
    // projected by the previous camera: where this texel's cloud was.
    let ray = view_ray(in.uv);
    let stop_m = min(now.y, CLOUD_FAR_CAP_M);
    var point = ray.origin + ray.direction*select(stop_m, now.x, now.x > 0.0);
    var was = cloud_was(point);
    // No cloud this frame: the stop is a guess at where the history's cloud
    // is, and read there the history slid with the ground's parallax, not the
    // cloud's. Look up the distance the history holds there and read it again
    // at that distance (`cloud-edges`).
    if (now.x <= 0.0 && was.x >= 0.0) {
        let held_size = vec2<i32>(textureDimensions(cloud_depth));
        let held_at = clamp(vec2<i32>(was*vec2<f32>(held_size)), vec2<i32>(0), held_size - 1);
        let held = textureLoad(cloud_depth, held_at, 0).r;
        if (held > 0.0) {
            point = ray.origin + ray.direction*min(held, stop_m);
            was = cloud_was(point);
        }
    }
    if (was.x < 0.0) { return out; }
    let stop = ray.origin + ray.direction*stop_m;
    let previous = cloud_previous(was, stop, point);
    if (previous.weight <= 1e-3) { return out; }
    out.cloud = mix(clamp(previous.cloud, lo, hi), current, view.cloud_history.x);
    // A texel whose cloud is the history's alone keeps that cloud's distance,
    // so the composite hazes it like its neighbours (`cloud-edges`); one the
    // clip emptied has no cloud and no distance.
    if (now.x <= 0.0 && out.cloud.w > 1e-3) {
        out.depth.x = select(distance(ray.origin, point), previous.distance, previous.distance > 0.0);
    }
    return out;
}

// Where the march stops for a ray that meets nothing, metres: a half float's
// range with room to spare.
const CLOUD_FAR_CAP_M: f32 = 60000.0;

// The march texel's cloud laid over one full-resolution pixel: the four
// texels round it, bilinearly, each dropped when it did not look where this
// pixel looks: its cloud lies beyond this pixel's own ground, sea or limb, or
// its march stopped on ground well short of where this pixel's ray goes. A
// coarse texel straddling a silhouette otherwise carried the cloud across it
// as a staircase of texels (`cloud-budget`).
// `distance` receives where the kept texels' cloud is along the ray,
// weighted by the cloud each holds: what the haze is measured to
// (`cloud-close-up`).
// `own` takes the puffs' own coverage, unhazed (the history's alpha), beside
// the hazed coverage in the result's w (`dusk-clouds`).
fn cloud_upsampled(uv: vec2<f32>, far: f32, distance: ptr<function, f32>, own: ptr<function, f32>) -> vec4<f32> {
    let size = vec2<i32>(textureDimensions(cloud_history));
    let at = uv*vec2<f32>(size) - 0.5;
    let base = vec2<i32>(floor(at));
    let f = at - floor(at);
    var sum = vec4<f32>(0.0);
    var weight = 0.0;
    var at_sum = 0.0;
    var at_weight = 0.0;
    var own_sum = 0.0;
    for (var k = 0u; k < 4u; k++) {
        let o = vec2<i32>(i32(k & 1u), i32((k >> 1u) & 1u));
        let texel = clamp(base + o, vec2<i32>(0), size - 1);
        let w = select(1.0-f.x, f.x, o.x == 1)*select(1.0-f.y, f.y, o.y == 1);
        let depth = textureLoad(cloud_depth, texel, 0).rg;
        let kept = select(0.0, 1.0, cloud_texel_sees(depth, far));
        var cloud = textureLoad(cloud_history, texel, 0);
        own_sum += cloud.w*w*kept;
        // Under the puffs, the coverage the air leaves is kept apart from the
        // light (`puff-haze`).
        if (view.cloud_puff_fade.w > 0.5) { cloud.w = textureLoad(cloud_puff_cover, texel, 0).r; }
        sum += cloud*w*kept;
        weight += w*kept;
        let held_at = cloud_texel_distance(depth.x);
        let holds = w*kept*cloud.w*select(0.0, 1.0, held_at > 0.0);
        at_sum += held_at*holds;
        at_weight += holds;
    }
    *distance = select(0.0, at_sum/max(at_weight, 1e-6), at_weight > 1e-6);
    // Every texel round the pixel looked somewhere else: there is no cloud
    // this pixel can take. Dividing their near-nothing weights back up drew a
    // whole cloud from texels that had been rejected (`cloud-close-up`).
    if (weight < 1e-3) { *own = 0.0; return vec4<f32>(0.0); }
    *own = own_sum/weight;
    return sum/weight;
}

// The sky behind a cloud without its stars (`dusk-clouds`): the dimmest of
// the pixel and four neighbours three pixels off whose depth says sky. A
// star is a point a few pixels across (`steady-stars`); the sky's glow is
// smooth over six.
fn sky_without_stars(uv: vec2<f32>, scene: vec3<f32>) -> vec3<f32> {
    let px = 3.0/vec2<f32>(textureDimensions(scene_color));
    var best = scene;
    var best_l = dot(scene, vec3<f32>(0.2126, 0.7152, 0.0722));
    let offsets = array<vec2<f32>, 4>(vec2<f32>(1.0, 0.0), vec2<f32>(-1.0, 0.0), vec2<f32>(0.0, 1.0), vec2<f32>(0.0, -1.0));
    for (var k = 0u; k < 4u; k++) {
        let at = clamp(uv + offsets[k]*px, vec2<f32>(0.0), vec2<f32>(1.0));
        if (load_depth(at) > 1e-7) { continue; }
        let c = textureSampleLevel(scene_color, scene_sampler, at, 0.0).rgb;
        let l = dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
        if (l < best_l) { best = c; best_l = l; }
    }
    return best;
}

// How much of a cloud at `d` metres along the ray the air in front of it
// leaves (`cloud-close-up`): the optical depth of the same exponential
// atmosphere the ground's haze is (density `fog_night.w` per metre at the sea,
// scale height `fog_day.w`), by Simpson's rule over five points from the eye
// to the cloud, times `cloud_haze`. Near the ground a cloud kilometres out
// dissolves into what is behind it; from orbit the air is thin and a cloud
// keeps its contrast.
fn cloud_air(eye: vec3<f32>, direction: vec3<f32>, d: f32) -> f32 {
    let haze = view.cloud_history.w;
    if (haze <= 0.0 || d <= 0.0) { return 1.0; }
    let radius = view.planet_center.w;
    let height = max(view.fog_day.w, 1.0);
    var sum = 0.0;
    for (var k = 0; k <= 4; k++) {
        let at = eye + direction*(d*f32(k)/4.0);
        let weight = select(select(2.0, 4.0, (k & 1) == 1), 1.0, k == 0 || k == 4);
        sum += weight*exp(-max(length(at) - radius, 0.0)/height);
    }
    let depth = d*sum/12.0*max(view.fog_night.w, 0.0)*haze;
    return exp(-depth);
}

// ---- The height fog with RAIN SHADOW off (`weather-rain` 7) -----------------
// The owner: "if we have rain shadow turned off can we just do normal
// distance fog?", then "or height fog?". With the rain volume off, rain over
// the eye lays an exponential height fog in its place, at the end of the
// clouds' composite (no pass of its own). Its density at a height h over the
// reference level (the ground or the sea under the eye, along the body's own
// up) is sigma0 exp(-h/H): the volume's extinction at the player's feet,
// thicker in the valley below, thinner up the hill and the sky. Integrated
// along the ray in closed form, the height taken linear in distance between
// the ray's two ends, each measured exactly on the sphere. It stops where the
// volume would have: the ground, the sea, the cloud base or the range.
// Coloured as the volume is lit, without its shadows and lamps: the rain's
// tint in the sky's mean light and the sun's forward scatter.
const HEIGHT_FOG_DEEPEST: f32 = 2.5;
fn height_fog(uv: vec2<f32>, color: vec3<f32>) -> vec3<f32> {
    let sigma0 = view.height_fog.x;
    if (sigma0 <= 0.0) { return color; }
    let far = rain_far(uv);
    if (far <= 0.0) { return color; }
    let ray = view_ray(uv);
    let eye = ray.origin-view.planet_center.xyz;
    let fall = max(view.height_fog.y, 1.0);
    let ground = view.height_fog.z;
    let ha = max((length(eye) - ground)/fall, -HEIGHT_FOG_DEEPEST);
    let hb = max((length(eye + ray.direction*far) - ground)/fall, -HEIGHT_FOG_DEEPEST);
    let dh = hb - ha;
    // The mean of exp(-h) over the ray, h linear from ha to hb.
    var mean = exp(-ha)*(1.0 - 0.5*dh);
    if (abs(dh) > 1e-3) { mean = (exp(-ha) - exp(-hb))/dh; }
    let depth = sigma0*far*max(mean, 0.0);
    let kept = exp(-depth);
    // As the renderer it stands in for lights its rain: the volume by
    // `rain_light` (the sun's height and the cover), the shells by the sky's
    // light and the sun's forward scatter.
    var tint = view.rain_tint.rgb*view.rain_look.z;
    if (view.rain_shell.w > 0.5 && view.rain_sky.w > 0.5) {
        let mu = dot(ray.direction, safe_normal(view.sun.xyz));
        tint = view.rain_tint.rgb*(view.rain_sky.rgb
            + view.rain_sun.rgb*rain_phase(mu, view.rain_sun.w)*12.566371);
    }
    return color*kept + tint*(1.0 - kept);
}

@fragment
fn clouds(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let uv = sea_gap_uv(in.uv);
    return vec4<f32>(height_fog(uv, clouds_over(uv)), 1.0);
}

// The sea's horizon line (`horizon-haze` 4). A pixel whose depth says sky
// but whose ray meets the sea's sphere looks into the gap between the sea's
// silhouette and that sphere's tangent: the tide and the troughs take the
// sheet under the sphere, and a few rows of rays under the horizon meet no
// sea. The cloud stops such a ray at the sphere, short of its layer, so
// under cover the gap kept the clear dome's blue under a greyed sky: a dark
// band. Such a pixel is drawn as the one `SEA_GAP_LIFT_PX` over the
// sphere's horizon in the same compass direction, its cloud and fog
// included. Anything else is its own pixel: the sheet is not in the scene's
// depth, so a pixel the sheet drew (its own depth target, `water_depth`) is
// the sea, not the gap.
const SEA_GAP_LIFT_PX: f32 = 2.0;
// Whether `uv` is open to the sky in both the scene's depth and the sheet's:
// a ray there that meets the sea's sphere is in the gap, not on the sea.
fn sea_gap(uv: vec2<f32>) -> bool {
    return load_depth(uv) <= 1e-7
        && water_depth_at(uv*vec2<f32>(textureDimensions(water_depth))) <= 1e-7;
}
fn sea_gap_uv(uv: vec2<f32>) -> vec2<f32> {
    if (!sea_gap(uv)) { return uv; }
    let ray = view_ray(uv);
    let eye = ray.origin - view.planet_center.xyz;
    let radius = view.planet_center.w;
    let r = length(eye);
    if (r <= radius || cloud_sphere_hit(eye, ray.direction, radius).x <= 0.0) { return uv; }
    let up = eye/r;
    let across = ray.direction - up*dot(ray.direction, up);
    if (dot(across, across) < 1e-10) { return uv; }
    let level = radius/r;
    let horizon = safe_normal(safe_normal(across)*level - up*sqrt(max(1.0 - level*level, 0.0)));
    let far = ray.origin + horizon*1.0e4;
    let a = view.clip_from_local*vec4<f32>(far, 1.0);
    let b = view.clip_from_local*vec4<f32>(far + up*10.0, 1.0);
    if (a.w <= 1e-4 || b.w <= 1e-4) { return uv; }
    let at = vec2<f32>(a.x/a.w*0.5 + 0.5, 0.5 - a.y/a.w*0.5);
    let above = vec2<f32>(b.x/b.w*0.5 + 0.5, 0.5 - b.y/b.w*0.5);
    let size = vec2<f32>(textureDimensions(scene_color));
    let step = (above - at)*size;
    if (dot(step, step) < 1e-8) { return uv; }
    return clamp(at + normalize(step)*SEA_GAP_LIFT_PX/size, vec2<f32>(0.0), vec2<f32>(1.0));
}

// The clouds laid over the scene at `uv`.
fn clouds_over(uv: vec2<f32>) -> vec3<f32> {
    let scene = textureSampleLevel(scene_color,scene_sampler,uv,0.0).rgb;
    if (view.fx.y > 0.75) { return scene; }
    // This pixel's own ray against its own depth: where it cannot reach the
    // layer (a hill in front of the cloud) nothing is laid over it, so the
    // hill keeps a full-resolution edge however coarse the march.
    let ray = view_ray(uv);
    let eye = ray.origin-view.planet_center.xyz;
    let far = cloud_far(ray, uv, vec2<f32>(0.0), eye);
    let span = cloud_span(eye,ray.direction,0.0,far,cloud_layer());
    if (span.y <= span.x) { return scene; }
    // The air in front of the cloud fades it into what is behind, its light
    // and its cover alike, measured to where the texels it came from hold
    // their cloud (`cloud-close-up`).
    // Where no texel it keeps holds a distance (cloud from the history alone),
    // this pixel's own entry into the layer: measured to zero, the haze was
    // none, and every such texel at a cloud's edge stood out unhazed, a bright
    // rim round a limb cloud seen through thick air (`cloud-edges`).
    // The puffs come hazed, each to its own distance, from their own pass
    // (`puff-haze`); with the puffs configured the history holds nothing
    // else, so the air here is the march's alone.
    var at = 0.0;
    var own = 0.0;
    let upsampled = cloud_upsampled(uv, far, &at, &own);
    var air = 1.0;
    if (view.cloud_puff_fade.w < 0.5) {
        air = cloud_air(eye, ray.direction, min(select(span.x, at, at > 0.0), far));
    }
    let cloud = upsampled*air;
    var behind = scene*(1.0-cloud.w)+cloud.rgb;
    // Under the puffs, with the sky behind: the cloud hides the scene by its
    // own coverage, and the air in front of it adds the sky's light, never
    // the stars' (`dusk-clouds`). With no star there this is the line above.
    if (view.cloud_puff_fade.w > 0.5 && far >= CLOUD_FAR_CAP_M*0.999 && own > cloud.w) {
        let air_light = sky_without_stars(uv, scene);
        behind = scene*(1.0-own) + cloud.rgb + air_light*(own - cloud.w);
    }
    // The first metres from the eye, to this pixel's own stop, are the
    // cloud's fog seen in front of everything else: it thickens as the eye
    // enters a cloud and thins as it leaves, with nothing to flicker
    // (`cloud-entry`).
    let near_m = cloud_near_reach(cloud_layer());
    let slab = clamp(min(far, near_m) - span.x, 0.0, near_m);
    if (slab <= 0.0) { return behind; }
    let fog = textureSampleLevel(cloud_near, cloud_history_sampler, uv, 0.0);
    return mix(behind, fog.rgb, 1.0 - exp(-max(fog.a, 0.0)*slab));
}

// ---- The puffs (`cloud-puffs`) ---------------------------------------------
// One billboard a puff, laid out by the weather on the CPU
// (`pbd_core::cloud_puffs`), sorted far to near and drawn instanced into the
// clouds' history targets at the march's resolution; the composite above lays
// them over the scene as it lays the march, hazed by the air in front.
// The look is Crab-breeze's (`assets/shaders/cloud.wesl`, `puff` and
// `facing`, origin/main d663b86c) on a sphere: "up" is the puff's own
// direction from the body's centre, and the light is the march's colours.

struct PuffInstance {
    centre_seed: vec4<f32>, // xyz centre, body-local m; w noise seed
    shape: vec4<f32>,       // x radius across, y radius up, z darkness, w opacity
    layer: vec4<f32>,       // x the column's base radius, y its top radius, z the cover over it
}
@group(3) @binding(10) var<storage, read> cloud_puffs: array<PuffInstance>;

struct PuffVaryings {
    @builtin(position) position: vec4<f32>,
    @location(0) body: vec3<f32>,
    @location(1) corner: vec2<f32>,
    @location(2) clip: vec4<f32>,
    @location(3) @interpolate(flat) instance: u32,
}

// A puff's frame: it faces the eye's position and stays upright on the
// sphere, so turning the head never rolls it; only moving past it turns it.
// Columns: right, up, back toward the eye. Straight under or over a puff,
// where upright means nothing, its right is a fixed tangent.
fn puff_facing(c: vec3<f32>, eye: vec3<f32>) -> mat3x3<f32> {
    let radial = safe_normal(c);
    let back = safe_normal(eye - c);
    let level = cross(radial, back);
    var fixed = cross(radial, vec3<f32>(0.0, 0.0, 1.0));
    if (dot(fixed, fixed) < 1e-4) { fixed = cross(radial, vec3<f32>(1.0, 0.0, 0.0)); }
    fixed = fixed - back*dot(fixed, back);
    if (dot(fixed, level) < 0.0) { fixed = -fixed; }
    let right = safe_normal(mix(safe_normal(fixed), level, smoothstep(0.02, 0.2, length(level))));
    let up = cross(back, right);
    return mat3x3<f32>(right, up, back);
}

@vertex
fn cloud_puff_vertex(@builtin(vertex_index) vertex: u32, @builtin(instance_index) instance: u32) -> PuffVaryings {
    let puff = cloud_puffs[instance];
    let corners = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0));
    let corner = corners[vertex % 6u];
    let c = puff.centre_seed.xyz;
    let eye = view.camera_time.xyz - view.planet_center.xyz;
    let f = puff_facing(c, eye);
    // A flat puff seen from above is round: its height on screen grows toward
    // its width as the view turns down onto it.
    let k = abs(dot(f[2], safe_normal(c)));
    let up = mix(puff.shape.y, puff.shape.x, k*k);
    let body = c + f[0]*(corner.x*puff.shape.x) + f[1]*(corner.y*up);
    var out: PuffVaryings;
    out.clip = view.clip_from_local*vec4<f32>(body + view.planet_center.xyz, 1.0);
    out.position = out.clip;
    out.body = body;
    out.corner = corner;
    out.instance = instance;
    return out;
}

// Billows, 0..1: up to three octaves of gradient noise on a plane through
// the lattice, one plane a puff. `detail` weighs the second and third: a puff
// small on screen leaves them out (design 3b), and as noise of mean nought
// they fade to their mean, so its edge keeps its place.
fn puff_billow(p: vec2<f32>, z: f32, detail: vec2<f32>) -> f32 {
    var g = gnoise3(vec3<f32>(p.x, z, p.y))*0.5;
    if (detail.x > 0.0) {
        g += gnoise3(vec3<f32>(p.x*2.03 + 17.0, z + 3.1, p.y*2.03))*0.25*detail.x;
    }
    if (detail.y > 0.0) {
        g += gnoise3(vec3<f32>(p.x*4.07 + 31.0, z + 7.3, p.y*4.07))*0.125*detail.y;
    }
    return clamp(0.5 + 0.75*g, 0.0, 1.0);
}

// How far the near field's fog reaches from the eye: the march's two near
// steps, or with the puffs the distance they fade in over, so the fog
// carries the cloud up to where the puffs take over (design 3a).
fn cloud_near_reach(layer: CloudLayer) -> f32 {
    return select(cloud_near_m(layer), max(view.cloud_puff_fade.z, 1.0), view.cloud_puff_fade.w > 0.5);
}

struct PuffOut {
    // Premultiplied light, as the air in front of the puff leaves it, and
    // the puff's own coverage, blended "over" (`puff-haze`).
    @location(0) cloud: vec4<f32>,
    // The puff's distance and the scene's stop along this ray, both kept at
    // their nearest (blend Min); cleared to the far cap.
    @location(1) depth: vec2<f32>,
    // The coverage the air leaves in r, blended "over" by the puff's own
    // coverage in a: the far puffs are hidden by the near puff, not by its
    // haze (`puff-haze`).
    @location(2) cover: vec4<f32>,
}

@fragment
fn cloud_puff_fragment(in: PuffVaryings) -> PuffOut {
    let puff = cloud_puffs[in.instance];
    let p = in.corner;
    // The puff's width in texels of this target, before any branch, where the
    // derivative is defined: what its detail is chosen by (design 3b).
    let texels = 2.0/max(length(fwidth(p)), 1e-6);
    let detail = vec2<f32>(smoothstep(16.0, 24.0, texels), smoothstep(32.0, 48.0, texels));
    let d = length(p);
    if (d > 1.0) { discard; }
    let eye = view.camera_time.xyz - view.planet_center.xyz;
    let to = in.body - eye;
    let dist = max(length(to), 1e-3);
    let look = to/dist;
    // Where this pixel's ray stops in the scene: the depth buffer, the sea.
    let ndc = in.clip.xy/in.clip.w;
    let uv = vec2<f32>(ndc.x*0.5 + 0.5, 0.5 - ndc.y*0.5);
    var far = CLOUD_FAR_CAP_M;
    let depth = load_depth(uv);
    if (depth > 1e-7) {
        far = min(far, length(reconstruct_local(uv, depth) - view.camera_time.xyz));
    }
    let sea = cloud_sphere_hit(eye, look, view.planet_center.w);
    if (sea.x > 0.0) { far = min(far, sea.x); }
    let soft = smoothstep(0.0, max(view.cloud_puff_fade.y, 1.0), far - dist);
    let near = smoothstep(view.cloud_puff_fade.z*0.3, view.cloud_puff_fade.z, dist);
    // The eye inside the puff: its billboard faces the eye through the puff's
    // middle and would lay one flat sheet over the whole view. It fades as
    // the eye enters, and the near fog carries the cloud from there.
    let rel = eye - puff.centre_seed.xyz;
    let radial_off = dot(rel, safe_normal(puff.centre_seed.xyz));
    let across = rel - safe_normal(puff.centre_seed.xyz)*radial_off;
    let inside = length(vec2<f32>(length(across)/max(puff.shape.x, 1.0), radial_off/max(puff.shape.y, 1.0)));
    let outside = smoothstep(0.8, 1.6, inside);
    // Behind the ground, past the limb, too near or with the eye inside: done
    // before any noise is paid for.
    if (soft*near*outside*puff.shape.w < 0.004) { discard; }
    let seed = puff.centre_seed.w;
    let dark = puff.shape.z;
    // The edge eaten by billows, faded out before the circle whatever the
    // noise says, so no puff shows the arc where its quad is cut.
    let n = puff_billow(p*2.2, seed*13.1 + view.snow_tint.w*0.004, detail);
    var n2 = 0.5;
    if (detail.x > 0.0) {
        n2 = mix(0.5, clamp(0.5 + 0.65*gnoise3(vec3<f32>(p.x*7.0, seed*5.0 + 40.0, p.y*7.0)), 0.0, 1.0), detail.x);
    }
    let dens = smoothstep(1.0, 0.45, d + (n - 0.5)*view.cloud_puff_fade.x + (n2 - 0.5)*0.2)
        * smoothstep(1.0, 0.8, d);
    let f = puff_facing(puff.centre_seed.xyz, eye);
    let z = sqrt(max(0.0, 1.0 - d*d));
    let nrm = safe_normal(f[0]*p.x + f[1]*p.y + f[2]*(z + 0.15) + 0.5*(vec3<f32>(n, n2, n) - 0.5));
    let sun = safe_normal(view.sun.xyz);
    let radial = safe_normal(in.body);
    let wrap = view.cloud_puff.x;
    let diffuse = clamp((dot(nrm, sun) + wrap)/(1.0 + wrap), 0.0, 1.0);
    // Height through its column: the base goes grey, the tops stay white.
    let r = length(in.body);
    let h = clamp((r - puff.layer.x)/max(puff.layer.y - puff.layer.x, 1.0), 0.0, 1.0);
    let base_shade = mix(view.cloud_puff.y, view.cloud_puff.z, dark);
    let under = smoothstep(-0.95, 0.55, p.y + (n - 0.5)*0.4);
    let above_base = smoothstep(0.0, 350.0, r - puff.layer.x);
    let shadow = mix(0.35, 1.0, under)*mix(0.55, 1.0, above_base);
    let body = mix(1.0 - base_shade, 1.0, smoothstep(0.0, 0.7, h))*shadow;
    let silver = pow(max(dot(look, sun), 0.0), 6.0)*(1.0 - z)*(1.0 - z)*view.cloud_puff.w;
    // The march's light (`cloud_light`): the sun's colour by its height, the
    // sky, the ground's bounce, the night floor and a lightning flash.
    let elevation = dot(radial, sun);
    // The dome's sun and the sky's own light (`dusk-clouds`): the sun
    // lifted for this point's height and gone with the dome's twilight, the
    // sky at this puff's sun and cover, the day's blue only while the sun is up.
    let layer = cloud_layer();
    let day = cloud_day_share(elevation);
    let sun_colour = cloud_sunlight(elevation, r - view.planet_center.w);
    let sky = cloud_sky_light(elevation, puff.layer.z);
    let ground = vec3<f32>(0.30, 0.32, 0.28)*day*(1.0 - h);
    let albedo = mix(1.0, 0.55, dark);
    let lit = sun_colour*layer.light.x*1.6*(diffuse*body + silver + 0.25*body)*albedo
        + sky*layer.clouds.y*mix(0.45, 1.0, h)*mix(0.5, 1.0, shadow)*albedo
        + ground*layer.slab.y*(1.0 - under*0.5)
        + vec3<f32>(layer.clouds.w)
        + vec3<f32>(0.80, 0.85, 1.0)*cloud_flash_at(in.body, layer);
    // The flat base: nothing below the column's base, cut on the height of
    // the puff's own round surface here over a band a third of its height
    // deep. From under the base looking up, the base is what the eye sees.
    let surface = puff.centre_seed.xyz + puff.shape.x*f[0]*p.x + puff.shape.y*(f[1]*p.y + f[2]*z);
    let band = max(puff.shape.y*0.35, 20.0);
    let eye_r = length(eye);
    let from_below = smoothstep(puff.layer.x - 30.0, puff.layer.x - 250.0, eye_r)
        * smoothstep(0.15, 0.5, dot(look, safe_normal(eye)));
    let floor_cut = mix(smoothstep(puff.layer.x - band, puff.layer.x + band*0.5, length(surface)), 1.0, from_below);
    let a = clamp(dens*puff.shape.w*near*soft*floor_cut*outside, 0.0, 1.0);
    // The distances are written where the puff is, hazed or not, so the
    // silhouette tests read them as before (`puff-haze`).
    if (a < 0.004) { discard; }
    // Each puff is hazed by the air in front of IT, to its own distance along
    // this ray (`puff-haze`). Hazed once for the whole texel at the nearest
    // puff's distance, a far puff seen through a near one's thin edge took
    // the near one's lighter haze and stepped back to its own at the edge: an
    // outline round every puff in front of another. Measured at the
    // billboard's corners instead, which lie further off than the puff, the
    // air came out a little thick on every puff.
    // It adds its light and coverage as the air leaves them, and hides what
    // is behind it by its own coverage: the air adds its light over the puff,
    // it does not make the puff transparent to the cloud behind. Hidden by
    // the hazed coverage, every stack of puffs came out brighter inside, and
    // a deep one at the horizon would come out opaque.
    // Fading into what is behind stands for the air's light only where the
    // sky is behind. Where the ray stops on the ground or the sea, the
    // surface shows only the share of that air it is itself seen through:
    // the air between the puff and the surface (`space-clouds`). From orbit
    // the sea just under a cloud is seen through the same air the cloud is,
    // and faded into it the cloud went half transparent.
    var air = cloud_air(eye, look, dist);
    if (far < CLOUD_FAR_CAP_M*0.999) {
        let behind = cloud_air(in.body, look, max(far - dist, 0.0));
        air = 1.0 - (1.0 - behind)*(1.0 - air);
    }
    let seen = a*air;
    var out: PuffOut;
    out.cloud = vec4<f32>(lit*seen, a);
    out.depth = vec2<f32>(dist, far);
    out.cover = vec4<f32>(seen, 0.0, 0.0, a);
    return out;
}

// The streaks' two widths, metres, and the distance from the eye over which
// the fine one hands over to the broad.
const RAIN_STREAK_FINE_M: f32 = 1.5;
const RAIN_STREAK_BROAD_M: f32 = 8.0;
const RAIN_STREAK_BLEND_M: vec2<f32> = vec2<f32>(150.0, 700.0);

// ---- Rain as shells round the eye (`rain-shells`) -----------------------------
// The owner's cylinders: streaks on a few cylinders centred on the eye,
// scrolling down at the fall speed, nothing marched. Each cylinder is met
// analytically here (the eye is on its axis, so the hit is one division), which
// draws exactly what a cylinder mesh would, in the volume's own pass. Each shell
// stands for the slab of air between it and the next, so five samples carry the
// volume's sum. The axis leans into the fall the eye feels (the fall minus the
// eye's velocity), so in flight the streaks slant and rush past.

const RAIN_SHELLS: u32 = 6u;
// Each about 3 times the one inside. Two out past half a kilometre, so a
// storm 1-2 km off is read where it is, as the volume's sixteen steps read it.
const RAIN_SHELL_RADII: array<f32, 6> = array<f32, 6>(6.0, 20.0, 70.0, 250.0, 700.0, 1500.0);
// The slab each stands for, between the midpoints to its neighbours, metres
// from the axis; the last runs to the volume's range.
const RAIN_SHELL_EDGES: array<f32, 7> = array<f32, 7>(0.0, 13.0, 45.0, 160.0, 475.0, 1100.0, 1.0e9);
// One cycle of the streak pattern along the axis, metres: a power of two, so
// all of them divide the fall's wrap and the pattern never jumps.
const RAIN_SHELL_CYCLE_M: array<f32, 6> = array<f32, 6>(1.0, 4.0, 16.0, 64.0, 256.0, 512.0);
// Streak columns round every shell: about 0.004 rad each, a few pixels.
const RAIN_SHELL_COLUMNS: f32 = 1536.0;

fn rain_hash(a: vec2<f32>) -> f32 {
    return fract(sin(dot(a, vec2<f32>(127.1, 311.7)))*43758.5453);
}

// The streak pattern, 0..1, at `x` columns round and `y` cycles along the
// fall: one streak a column a cycle, of hashed place, length and presence,
// brightest at its head. Snow is a short flake.
fn rain_shell_streak(x: f32, y: f32, shell: f32, snow: bool) -> f32 {
    let column = floor(x);
    let wrapped = column - RAIN_SHELL_COLUMNS*floor(column/RAIN_SHELL_COLUMNS);
    let h1 = rain_hash(vec2<f32>(wrapped, shell));
    let h2 = rain_hash(vec2<f32>(shell*3.1, wrapped + 17.0));
    let across = clamp(1.0 - abs(fract(x) - (0.3 + 0.4*h1))/0.18, 0.0, 1.0);
    let along = y + h2;
    let present = step(0.4, rain_hash(vec2<f32>(wrapped + shell*7.0, floor(along))));
    let v = fract(along);
    let streak_len = select(0.15 + 0.3*h1, 0.03 + 0.03*h1, snow);
    return across*present*smoothstep(streak_len, 0.0, v);
}

// ---- The rain fog, lit (`lighting-model`) ------------------------------------
// Under the new model a stretch of rain takes the light that is there: the
// sky's from all round (L2), the sun's or the moon's scattered forward
// (Henyey-Greenstein) where the cascades say it reaches, so trees and eaves
// cast shafts through it, and every lamp near the eye by the closed form of a
// point light's in-scatter along the stretch, so a lantern glows in the rain.
@group(3) @binding(14) var ssr_history: texture_2d<f32>;
@group(3) @binding(11) var rain_sun_map: texture_depth_2d_array;
@group(3) @binding(12) var rain_sun_cmp: sampler_comparison;
@group(3) @binding(13) var<storage, read> rain_cascades: SunCascades;

// One comparison tap of the finest cascade holding `p` (body frame): 1 lit.
fn rain_sun_at(p: vec3<f32>) -> f32 {
    if (rain_cascades.settings.x < 0.5) { return 1.0; }
    for (var i = 0u; i < 4u; i++) {
        let clip = rain_cascades.clip_from_body[i]*vec4<f32>(p, 1.0);
        if (max(abs(clip.x), abs(clip.y)) < 1.0 && clip.z > 0.0 && clip.z < 1.0) {
            let uv = vec2<f32>(clip.x*0.5 + 0.5, 0.5 - clip.y*0.5);
            let bias = rain_cascades.settings.z*rain_cascades.texel_m[i]/rain_cascades.depth_m[i];
            return textureSampleCompareLevel(rain_sun_map, rain_sun_cmp, uv, i, clip.z - bias);
        }
    }
    return 1.0;
}

// How much of the stretch `a`..`b` the sun reaches, six taps jittered per
// pixel (twelve over the two nearest shells' sixty metres). A cascade's
// clip space is an affine map of the point, so each cascade's clip is taken
// at the stretch's two ends and mixed for each tap (`lighting-perf` 2.10):
// eight matrix products a stretch instead of up to twenty-four. Each tap
// then picks its cascade and compares exactly as `rain_sun_at` does.
fn rain_sun_seen(eye: vec3<f32>, direction: vec3<f32>, a: f32, b: f32, jitter: f32) -> f32 {
    if (rain_cascades.settings.x < 0.5) { return 1.0; }
    let pa = vec4<f32>(eye + direction*a, 1.0);
    let pb = vec4<f32>(eye + direction*b, 1.0);
    var ca: array<vec4<f32>, 4>;
    var cb: array<vec4<f32>, 4>;
    for (var i = 0u; i < 4u; i++) {
        ca[i] = rain_cascades.clip_from_body[i]*pa;
        cb[i] = rain_cascades.clip_from_body[i]*pb;
    }
    var seen = 0.0;
    for (var k = 0u; k < 6u; k++) {
        let f = (f32(k) + jitter)/6.0;
        var lit = 1.0;
        for (var i = 0u; i < 4u; i++) {
            let clip = mix(ca[i], cb[i], f);
            if (max(abs(clip.x), abs(clip.y)) < 1.0 && clip.z > 0.0 && clip.z < 1.0) {
                let uv = vec2<f32>(clip.x*0.5 + 0.5, 0.5 - clip.y*0.5);
                let bias = rain_cascades.settings.z*rain_cascades.texel_m[i]/rain_cascades.depth_m[i];
                lit = textureSampleCompareLevel(rain_sun_map, rain_sun_cmp, uv, i, clip.z - bias);
                break;
            }
        }
        seen += lit;
    }
    return seen/6.0;
}

fn rain_phase(mu: f32, g: f32) -> f32 {
    return (1.0 - g*g)/(12.566371*pow(1.0 + g*g - 2.0*g*mu, 1.5));
}

// The mean light of every lamp near the eye over the stretch `a`..`b`: the
// integral of 1/(h^2 + s^2) along it, closed form, softened by 2.2 m as the
// wet film's highlights are.
fn rain_lamp_glow(eye: vec3<f32>, direction: vec3<f32>, a: f32, b: f32) -> vec3<f32> {
    var glow = vec3<f32>(0.0);
    for (var i = 0u; i < 8u; i++) {
        let lamp = view.rain_lamps[i];
        if (lamp.w <= 0.0) { continue; }
        let q = lamp.xyz - eye;
        let s0 = dot(q, direction);
        let h2 = max(dot(q, q) - s0*s0, 0.0);
        if (h2 > 256.0) { continue; }
        let h = sqrt(h2 + 4.84);
        let along = (atan((b - s0)/h) - atan((a - s0)/h))/h;
        glow += vec3<f32>(1.00, 0.70, 0.30)*1.25*lamp.w*11.6*along/max(b - a, 1e-3);
    }
    return glow;
}

fn rain_shells(scene: vec3<f32>, eye: vec3<f32>, direction: vec3<f32>, far: f32, range: f32,
               jitter: f32) -> vec4<f32> {
    let gathered = rain_gather(eye, direction, far, range, jitter);
    return vec4<f32>(scene*gathered.a + gathered.rgb, 1.0);
}

// What the shells along this ray gather (`lighting-perf` 2.9): rgb the light,
// a the transmittance. `rain_shells` lays it over the scene at full size;
// `rain_half` keeps it in a small target for `rain_compose`.
fn rain_gather(eye: vec3<f32>, direction: vec3<f32>, far: f32, range: f32,
               jitter: f32) -> vec4<f32> {
    let layer = cloud_layer();
    let base = view.cloud_clouds.x;
    let down = -safe_normal(eye);
    let velocity = view.rain_shell.xyz;
    let seconds = view.snow_tint.w;
    let soft_m = max(view.rain_shell_fall.z, 0.01);
    var transmittance = 1.0;
    var light = vec3<f32>(0.0);
    // `rain_shell_fall.w`: which kinds the rain map holds anywhere, one bit
    // each (rain 1, snow 2). A kind it does not hold could only `continue`
    // at every shell (`lighting-perf` 2.10).
    let kinds = u32(view.rain_shell_fall.w + 0.5);
    for (var kind = 0u; kind < 2u; kind++) {
        if ((kinds & (1u << kind)) == 0u) { continue; }
        let snow = kind == 1u;
        let fall = select(view.rain_look.x, view.rain_tint.w, snow);
        let relative = down*fall - velocity;
        let axis = select(down, relative/max(length(relative), 1e-3), length(relative) > 0.1);
        let fallen = select(view.rain_shell_fall.x, view.rain_shell_fall.y, snow);
        let c = dot(direction, axis);
        // Looking along the axis a cylinder's streaks crowd into lines.
        let steep = 1.0 - smoothstep(0.75, 0.95, abs(c));
        let across = direction - axis*c;
        let reference = view.rain_u.xyz - axis*dot(view.rain_u.xyz, axis);
        let b1 = select(safe_normal(reference), safe_normal(cross(axis, vec3<f32>(0.0, 0.0, 1.0))), length(reference) < 1e-3);
        let b2 = cross(axis, b1);
        let angle = atan2(dot(across, b2), dot(across, b1));
        let oblique = 1.0/sqrt(max(1.0 - c*c, 1e-4));
        let stop = min(far, range);
        for (var i = 0u; i < RAIN_SHELLS; i++) {
            // The stretch of this ray the shell stands for, cut at the ground,
            // the sea, the cloud base or the range: all of it is rain's haze,
            // so looking down at the ground the whole way there is covered.
            let enter = RAIN_SHELL_EDGES[i]*oblique;
            if (enter >= stop) { break; }
            let leave = min(RAIN_SHELL_EDGES[i + 1u]*oblique, stop);
            let t = RAIN_SHELL_RADII[i]*oblique;
            // The streaks are on the shell, when it is in front of what is
            // behind it; otherwise the stretch is haze alone, read midway.
            let on_shell = t < stop;
            let p = eye + direction*select(0.5*(enter + leave), t, on_shell);
            let here = rain_map_at(p);
            // The other kind's shells carry this place.
            if (abs(here) < 1e-3 || (here < 0.0) != snow) { continue; }
            var streak = 0.0;
            if (on_shell) {
                let shell = f32(i);
                var x = angle/6.2831853*RAIN_SHELL_COLUMNS + shell*0.37;
                let y = (t*c - fallen)/RAIN_SHELL_CYCLE_M[i];
                if (snow) { x += 0.3*sin(seconds*1.3 + y*2.0 + shell); }
                // Into the ground or the sea at the bottom, and the steep view.
                streak = rain_shell_streak(x, y, shell, snow)*steep*smoothstep(0.0, soft_m, far - t);
            }
            // The volume's mean darkening (its 0.3 + 1.4 streak averages
            // about one), with the light gathered into the streaks.
            let density = abs(here)*view.rain_v.w*(0.7 + 6.0*streak);
            // Into the cloud base at the top, and the air's haze with distance.
            let blend = smoothstep(base, base - 45.0, length(p))*cloud_air(eye, direction, enter);
            let absorbed = (1.0 - exp(-density*(leave - enter)))*blend;
            var tint = select(view.rain_tint.rgb, view.snow_tint.rgb, snow)*view.rain_look.z
                + vec3<f32>(0.80, 0.85, 1.0)*cloud_flash_at(p, layer)*0.15;
            if (view.rain_sky.w > 0.5) {
                var sun_seen = 1.0;
                if (enter < 60.0) { sun_seen = rain_sun_seen(eye, direction, enter, min(leave, 60.0), jitter); }
                let mu = dot(direction, safe_normal(view.sun.xyz));
                let lit = view.rain_sky.rgb
                    + view.rain_sun.rgb*rain_phase(mu, view.rain_sun.w)*12.566371*sun_seen
                    + rain_lamp_glow(eye, direction, enter, leave);
                tint = select(view.rain_tint.rgb, view.snow_tint.rgb, snow)*lit
                    + vec3<f32>(0.80, 0.85, 1.0)*cloud_flash_at(p, layer)*0.15;
            }
            light += tint*transmittance*absorbed;
            transmittance *= 1.0 - absorbed;
        }
    }
    return vec4<f32>(light, transmittance);
}

// ---- The rain volume at a smaller size (`lighting-perf` 2.9) ------------------
// The shells' light and transmittance are gathered into a small target (the
// view at `rain_render_scale`), and `rain_compose` lays them over the scene
// at full size: the four small texels round a pixel, each weighed bilinearly
// and by how near its own depth is to the pixel's, so a wall's edge in front
// of the rain takes none of the rain gathered past it.
@group(3) @binding(15) var rain_half_tex: texture_2d<f32>;

// The rain is in the air (`rain-under-water`). From a camera under the sea
// every ray starts in the water, and what it sees of the air beyond the
// surface - Snell's window, the murk over it - is the sheet's, drawn before
// this pass. The volume marched from the eye through the water as if it
// were air and laid its fog over the murk, the sheet and the seabed alike.
fn rain_under_water() -> bool { return view.fx.y > 0.75; }

// Where the ray through `uv` stops for the rain: the ground, the sea, the
// cloud base or the range; zero where no rain is drawn.
fn rain_far(uv: vec2<f32>) -> f32 {
    let range = view.rain_look.w;
    if (range <= 0.0 || rain_under_water()) { return 0.0; }
    let ray = view_ray(uv);
    let eye = ray.origin-view.planet_center.xyz;
    let base = view.cloud_clouds.x;
    if (length(eye) >= base) { return 0.0; }
    var far = range;
    let depth = load_depth(uv);
    if (depth > 1e-7) {
        far = min(far,length(reconstruct_local(uv,depth)-ray.origin));
    }
    let sea = cloud_sphere_hit(eye,ray.direction,view.planet_center.w);
    if (sea.x > 0.0 && !sea_gap(uv)) { far = min(far,sea.x); }
    let ceiling = cloud_sphere_hit(eye,ray.direction,base);
    if (ceiling.y > 0.0) { far = min(far,ceiling.y); }
    return select(0.0, far, far > 1.0);
}

@fragment
fn rain_half(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let far = rain_far(in.uv);
    if (far <= 0.0) { return vec4<f32>(0.0, 0.0, 0.0, 1.0); }
    let ray = view_ray(in.uv);
    let eye = ray.origin-view.planet_center.xyz;
    if (view.rain_shell.w < 0.5) { return rain_volume(in.uv, eye, ray.direction, far); }
    let pixel = in.position.xy;
    let jitter = fract(52.9829189*fract(dot(pixel, vec2<f32>(0.06711056, 0.00583715))));
    return rain_gather(eye, ray.direction, far, view.rain_look.w, jitter);
}

// The distance to what a pixel shows, for weighing the small texels by it.
fn rain_depth_m(uv: vec2<f32>) -> f32 {
    let depth = load_depth(uv);
    if (depth <= 1e-7) {
        // The sheet is not in the scene's depth: where it drew, its sphere's
        // hit, so the sky's long rain does not bleed down onto the sea at the
        // horizon (`horizon-haze` 4).
        let ray = view_ray(uv);
        let sea = cloud_sphere_hit(ray.origin-view.planet_center.xyz, ray.direction, view.planet_center.w);
        if (sea.x > 0.0 && !sea_gap(uv)) { return sea.x; }
        return 1.0e6;
    }
    return length(reconstruct_local(uv, depth)-view_ray(uv).origin);
}

@fragment
fn rain_compose(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let scene = textureSampleLevel(scene_color,scene_sampler,uv,0.0).rgb;
    let size = vec2<f32>(textureDimensions(rain_half_tex));
    let at = uv*size - 0.5;
    let corner = floor(at);
    let f = at - corner;
    let here = rain_depth_m(uv);
    var sum = vec4<f32>(0.0);
    var weight = 0.0;
    var nearest = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    var nearest_gap = 1.0e30;
    for (var k = 0u; k < 4u; k++) {
        let o = vec2<f32>(f32(k & 1u), f32(k >> 1u));
        let texel = clamp(corner + o, vec2<f32>(0.0), size - 1.0);
        let gathered = textureLoad(rain_half_tex, vec2<i32>(texel), 0);
        let there = rain_depth_m((texel + 0.5)/size);
        let gap = abs(there - here);
        let bilinear = mix(1.0 - f.x, f.x, o.x)*mix(1.0 - f.y, f.y, o.y);
        let w = bilinear*exp(-gap/(0.02*min(here, 1.0e4) + 0.1));
        sum += gathered*w;
        weight += w;
        if (gap < nearest_gap) { nearest_gap = gap; nearest = gathered; }
    }
    let gathered = select(nearest, sum/max(weight, 1e-6), weight > 1e-3);
    return vec4<f32>(scene*gathered.a + gathered.rgb, 1.0);
}

@fragment
fn rain(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let scene = textureSampleLevel(scene_color,scene_sampler,uv,0.0).rgb;
    let range = view.rain_look.w;
    if (range <= 0.0 || rain_under_water()) { return vec4<f32>(scene,1.0); }
    let ray = view_ray(uv);
    let center = view.planet_center.xyz;
    let eye = ray.origin-center;
    let base = view.cloud_clouds.x;
    // From above the cloud base the clouds are in front of the rain and the
    // storm reads through them; the volume is for looking along and up at it.
    if (length(eye) >= base) { return vec4<f32>(scene,1.0); }
    var far = range;
    let depth = load_depth(uv);
    if (depth > 1e-7) {
        far = min(far,length(reconstruct_local(uv,depth)-ray.origin));
    }
    let sea = cloud_sphere_hit(eye,ray.direction,view.planet_center.w);
    if (sea.x > 0.0 && !sea_gap(uv)) { far = min(far,sea.x); }
    let ceiling = cloud_sphere_hit(eye,ray.direction,base);
    if (ceiling.y > 0.0) { far = min(far,ceiling.y); }
    if (far <= 1.0) { return vec4<f32>(scene,1.0); }
    if (view.rain_shell.w > 0.5) {
        let pixel = in.uv*vec2<f32>(textureDimensions(scene_color));
        let jitter = fract(52.9829189*fract(dot(pixel, vec2<f32>(0.06711056, 0.00583715))));
        return rain_shells(scene,eye,ray.direction,far,range,jitter);
    }
    let gathered = rain_volume(uv, eye, ray.direction, far);
    return vec4<f32>(scene*gathered.a + gathered.rgb, 1.0);
}

// The volume's sixteen jittered steps along the ray through `uv` up to
// `far` (`rain_renderer: Volume`): rgb the light, a the transmittance, for
// `rain` to lay over the scene at full size or `rain_half` to keep small
// (`lighting-perf` 2.9).
fn rain_volume(uv: vec2<f32>, eye: vec3<f32>, direction: vec3<f32>, far: f32) -> vec4<f32> {
    let ray_direction = direction;
    let layer = cloud_layer();
    let steps = 16.0;
    let step_size = far/steps;
    // Jittered per pixel so sixteen steps read as grain, not as bands.
    let jitter = fract(sin(dot(uv,vec2<f32>(12.9898,78.233)))*43758.5453);
    let seconds = view.snow_tint.w;
    var transmittance = 1.0;
    var light = vec3<f32>(0.0);
    for (var i = 0.0; i < steps; i += 1.0) {
        let t = step_size*(i+jitter);
        let p = eye+ray_direction*t;
        let here = rain_map_at(p);
        if (abs(here) < 1e-3) { continue; }
        let snow = here < 0.0;
        let fall = select(view.rain_look.x,view.rain_tint.w,snow);
        // Streaks at two widths fixed in the world, fine near and broad far so
        // each stays about a pixel rather than shimmering. The distance only
        // weighs the two; where a streak IS depends on the point and the clock
        // alone. A width that grew with the distance from the eye rescaled the
        // whole pattern as the eye moved, and it slid up or down faster than
        // it fell (`cloud-budget`).
        let stretch = max(view.rain_look.y,1.0);
        let rel = p-view.rain_map.xyz*view.planet_center.w;
        let plane = vec2<f32>(dot(rel,view.rain_u.xyz),dot(rel,view.rain_v.xyz));
        let drop = length(p)-view.planet_center.w+seconds*fall;
        let fine = gnoise3(vec3<f32>(plane/RAIN_STREAK_FINE_M,drop/(RAIN_STREAK_FINE_M*stretch)));
        let broad = gnoise3(vec3<f32>(plane/RAIN_STREAK_BROAD_M,drop/(RAIN_STREAK_BROAD_M*stretch)));
        let n = mix(fine,broad,smoothstep(RAIN_STREAK_BLEND_M.x,RAIN_STREAK_BLEND_M.y,t));
        let streak = smoothstep(0.1,0.9,n*0.5+0.5);
        let density = abs(here)*view.rain_v.w*(0.3+1.4*streak);
        let absorbed = 1.0-exp(-density*step_size);
        let tint = select(view.rain_tint.rgb,view.snow_tint.rgb,snow)*view.rain_look.z
            + vec3<f32>(0.80,0.85,1.0)*cloud_flash_at(p,layer)*0.15;
        light += tint*transmittance*absorbed;
        transmittance *= 1.0-absorbed;
        if (transmittance < 0.02) { break; }
    }
    return vec4<f32>(light, transmittance);
}

// ---- The overlays ----------------------------------------------------------
// One colour ramp per `pbd_core::overlay::Ramp`, in its order, five stops
// each, as display (sRGB) colours. `overlay.rs` holds the same table for the
// legend's colour bar, and a test reads this one to hold them together.
const OVERLAY_STOPS: u32 = 5u;
const OVERLAY_RAMPS: array<vec3<f32>, 30> = array<vec3<f32>, 30>(
    // Speed: calm blue, teal, green, yellow, red.
    vec3<f32>(0.14, 0.20, 0.55), vec3<f32>(0.10, 0.55, 0.75), vec3<f32>(0.25, 0.75, 0.35),
    vec3<f32>(0.95, 0.85, 0.25), vec3<f32>(0.85, 0.20, 0.15),
    // Cover: clear night blue to white.
    vec3<f32>(0.05, 0.10, 0.20), vec3<f32>(0.25, 0.32, 0.45), vec3<f32>(0.55, 0.60, 0.68),
    vec3<f32>(0.80, 0.83, 0.88), vec3<f32>(1.00, 1.00, 1.00),
    // Rain: heavy snow violet, light snow, dry, light rain, heavy rain blue.
    vec3<f32>(0.55, 0.25, 0.80), vec3<f32>(0.80, 0.70, 0.95), vec3<f32>(0.92, 0.92, 0.92),
    vec3<f32>(0.40, 0.65, 0.95), vec3<f32>(0.08, 0.20, 0.70),
    // Humidity: dry brown to wet teal.
    vec3<f32>(0.55, 0.40, 0.20), vec3<f32>(0.80, 0.70, 0.45), vec3<f32>(0.85, 0.88, 0.75),
    vec3<f32>(0.40, 0.75, 0.70), vec3<f32>(0.10, 0.45, 0.55),
    // Sunlight: dark through orange to pale yellow.
    vec3<f32>(0.05, 0.03, 0.02), vec3<f32>(0.45, 0.15, 0.03), vec3<f32>(0.85, 0.40, 0.05),
    vec3<f32>(0.98, 0.75, 0.25), vec3<f32>(1.00, 0.97, 0.75),
    // Temperature: cold blue through white to hot red.
    vec3<f32>(0.15, 0.25, 0.75), vec3<f32>(0.45, 0.70, 0.95), vec3<f32>(0.95, 0.95, 0.95),
    vec3<f32>(0.98, 0.65, 0.30), vec3<f32>(0.80, 0.12, 0.10),
);

// Steps a streak is walked upstream over, and the share of the lattice cells
// that seed one. A streak's head crawls down from its seed; its tail is
// `OVERLAY_TAIL` of a streak's length.
const OVERLAY_STREAK_STEPS: u32 = 12u;
const OVERLAY_SEED_SHARE: f32 = 0.08;
const OVERLAY_TAIL: f32 = 0.35;
// A seed's lattice cell, as a share of one step: the cell is how WIDE a streak
// is, and narrower than a step so a streak is a line rather than a dash.
const OVERLAY_SEED_CELL: f32 = 0.6;
// Where a fading overlay (cloud, rain) is fully drawn, as a share of its top.
const OVERLAY_FADE_FULL: f32 = 0.15;

fn srgb_to_linear(c: vec3<f32>) -> vec3<f32> {
    return select(pow((c + 0.055)/1.055, vec3<f32>(2.4)), c/12.92, c <= vec3<f32>(0.04045));
}

fn overlay_ramp(ramp: u32, t: f32) -> vec3<f32> {
    var ramps = OVERLAY_RAMPS;
    let x = clamp(t, 0.0, 1.0)*f32(OVERLAY_STOPS - 1u);
    let i = min(u32(floor(x)), OVERLAY_STOPS - 2u);
    let row = min(ramp, 5u)*OVERLAY_STOPS;
    return srgb_to_linear(mix(ramps[row + i], ramps[row + i + 1u], x - f32(i)));
}

// Streamlines by walking the flow: from this point, step upstream along the
// overlay's vector; a lattice cell the walk passes through may be a SEED, and
// a seed sends a streak's head downstream at a rate that grows with the flow's
// speed. The pixel is lit when a head has just passed it. So the streaks lie
// along the flow and crawl the way it goes, as the wind map's do, with no
// particles and nothing stored between frames.
fn overlay_streaks(start: vec3<f32>, top: f32) -> f32 {
    let radius = view.planet_center.w;
    let step_m = view.overlay_flow.x;
    let flow0 = textureSampleLevel(weather_overlay, weather_sampler, start, 0.0).yzw;
    let speed0 = length(flow0);
    if (speed0 < 1e-5 || step_m <= 0.0) { return 0.0; }
    let share = clamp(speed0/max(top, 1e-6), 0.0, 1.5);
    let rate = view.overlay_flow.y*max(share, 0.05);
    let seconds = view.snow_tint.w;
    let cell = step_m*OVERLAY_SEED_CELL;
    var p = start;
    var lit = 0.0;
    for (var i = 0u; i < OVERLAY_STREAK_STEPS; i++) {
        let flow = textureSampleLevel(weather_overlay, weather_sampler, p, 0.0).yzw;
        let along = flow - p*dot(flow, p);
        let len = length(along);
        if (len < 1e-5) { break; }
        p = normalize(p - along/len*(step_m/radius));
        let cell_at = floor(p*radius/cell);
        if (cloud_hash(cell_at) < OVERLAY_SEED_SHARE) {
            let head = fract(cloud_hash(cell_at + vec3<f32>(7.1, 3.3, 5.7)) + seconds*rate);
            let behind = head - f32(i + 1u)/f32(OVERLAY_STREAK_STEPS);
            if (behind >= 0.0 && behind < OVERLAY_TAIL) {
                lit = max(lit, 1.0 - behind/OVERLAY_TAIL);
            }
        }
    }
    return lit*smoothstep(0.0, 0.08, share);
}

@fragment
fn overlay(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let scene = textureSampleLevel(scene_color,scene_sampler,uv,0.0).rgb;
    let row = view.overlay.x;
    if (row < 0.5) { return vec4<f32>(scene,1.0); }
    let ray = view_ray(uv);
    let eye = ray.origin-view.planet_center.xyz;
    // The planet point under the pixel: the ground or the sea, whichever is
    // nearer. The sky has no depth; the moon has, and stands far above the
    // cloud base, which is the test.
    var far = 1.0e9;
    let depth = load_depth(uv);
    if (depth > 1e-7) { far = length(reconstruct_local(uv,depth)-ray.origin); }
    let sea = cloud_sphere_hit(eye,ray.direction,view.planet_center.w);
    if (sea.x > 0.0) { far = min(far,sea.x); }
    if (far >= 1.0e9) { return vec4<f32>(scene,1.0); }
    let point = eye+ray.direction*far;
    if (length(point) > view.cloud_clouds.x) { return vec4<f32>(scene,1.0); }
    let direction = normalize(point);
    let here = cloud_map_smooth(weather_overlay,weather_sampler,direction);
    let low = view.overlay.y;
    let top = view.overlay.z;
    let t = (here.x-low)/max(top-low,1e-6);
    var colour = overlay_ramp(u32(row-0.5),t);
    var alpha = clamp(view.overlay.w,0.0,1.0);
    let flags = u32(view.overlay_flow.w+0.5);
    if ((flags & 2u) != 0u) {
        alpha *= clamp(abs(here.x)/max(top*OVERLAY_FADE_FULL,1e-6),0.0,1.0);
    }
    if ((flags & 1u) != 0u) {
        colour += vec3<f32>(overlay_streaks(direction,top)*view.overlay_flow.z);
    }
    // The planet under a map is grey, so the only colour on it is the data's:
    // the land keeps its relief and the coast its line, in brightness alone.
    let grey = vec3<f32>(dot(scene,vec3<f32>(0.2126,0.7152,0.0722)));
    return vec4<f32>(mix(grey,colour,alpha),1.0);
}

// ---- The fall splash, after the water (`water-flow` 6.1, 6.6) ---------------
// The spray, mist and fall droplets at a fall's foot (`water_flow.rs`
// `animate_splash`, M35's `splashQuads`): camera-facing quads drawn after the
// water in their own pass, so a splash over a pool stands on its surface.
// The fragment tests the main pass's depth (terrain in front) and the water's
// own depth (water in front) itself and fades softly against the nearer of
// the two (M35's wall blend). Lane 0 the camera's right, lane 1 its up, then
// per quad (`SPLASH_LANES`) its body-local centre and half size, its colour,
// and its cell and flip.
@group(3) @binding(17) var<storage, read> splash_quads: array<vec4<f32>>;
// The mist puffs' sheet: 4 x 4 cells of 32 texels, rows four puffs, columns
// four frames of a puff's life (`assets/textures/fx/mist_puffs`), sRGB.
@group(3) @binding(18) var fall_puffs: texture_2d<f32>;
@group(3) @binding(19) var fall_puffs_sampler: sampler;
// The water's own depth, as its pass left it (reversed Z; zero none).
@group(3) @binding(20) var water_depth: texture_depth_2d;

// `water_flow::SPLASH_LANES`.
const SPLASH_LANES: u32 = 3u;

struct SplashOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) colour: vec4<f32>,
    @location(2) body: vec3<f32>,
    @location(3) @interpolate(flat) cell: f32,
    // The radius of the pool's surface the quad's foot stands on.
    @location(4) @interpolate(flat) surface: f32,
}

@vertex
fn splash_vertex(@builtin(vertex_index) vertex: u32) -> SplashOut {
    let quad = vertex/6u;
    // Two triangles, 0 1 2 and 0 2 3, of the corners -r-u, r-u, r+u, u-r.
    let k = vertex%6u;
    let corner = select(select(k, k - 2u, k >= 3u), 0u, k == 3u);
    let sx = select(-1.0, 1.0, corner == 1u || corner == 2u);
    let sy = select(-1.0, 1.0, corner >= 2u);
    let base = 2u + quad*SPLASH_LANES;
    let centre = splash_quads[base];
    let cell = splash_quads[base + 2u];
    let p = centre.xyz + (splash_quads[0].xyz*sx + splash_quads[1].xyz*sy)*centre.w;
    var out: SplashOut;
    out.clip = view.clip_from_local*vec4<f32>(view.planet_center.xyz + p, 1.0);
    out.uv = vec2<f32>(0.5*(sx + 1.0), 0.5*(1.0 - sy));
    if (cell.y > 0.5) { out.uv.x = 1.0 - out.uv.x; }
    out.colour = splash_quads[base + 1u];
    out.body = p;
    out.cell = cell.x;
    out.surface = cell.z;
    return out;
}

// The water's depth at a pixel.
fn water_depth_at(pixel: vec2<f32>) -> f32 {
    let size = vec2<i32>(textureDimensions(water_depth));
    return textureLoad(water_depth, clamp(vec2<i32>(pixel), vec2<i32>(0), size - 1), 0);
}

// How far the nearer of the terrain and the water behind a pixel is from the
// eye, m; a large number where there is neither.
fn splash_behind_m(screen: vec2<f32>, ground: f32, water_d: f32) -> f32 {
    let eye = view.camera_time.xyz;
    var nearest = 1.0e9;
    if (ground > 1e-7) { nearest = length(reconstruct_local(screen, ground) - eye); }
    if (water_d > 1e-7) { nearest = min(nearest, length(reconstruct_local(screen, water_d) - eye)); }
    return nearest;
}

// With the eye fully under, what no sheet hides is seen through water: the
// murk the compose lays over this pixel, by the same terms, over `travel` m.
fn splash_murk(colour: vec3<f32>, travel: f32) -> vec3<f32> {
    let center = view.planet_center.xyz;
    let eye = view.camera_time.xyz;
    let cam_gap = max(0.0, eye_water_surface() - length(eye - center));
    let deep = murk(cam_gap, safe_normal(eye - center));
    return mix(deep, colour, exp(-view_extinction()*travel));
}

@fragment
fn splash_fragment(in: SplashOut) -> @location(0) vec4<f32> {
    // Behind the terrain or the water: either depth is nearer (reversed Z).
    let screen = in.clip.xy/vec2<f32>(textureDimensions(scene_depth));
    let ground = load_depth(screen);
    if (ground > in.clip.z + 1e-6) { discard; }
    let water_d = water_depth_at(in.clip.xy);
    if (water_d > in.clip.z + 1e-6) { discard; }
    // Nothing of the splash under its pool's surface: from above the water
    // hides it, from under it would hang in the pool.
    if (length(in.body) < in.surface - 0.02) { discard; }
    let smooth_fx = view.fall_splash.z > 0.5;
    var rgb = vec3<f32>(1.0);
    var a = 0.0;
    if (in.cell < -1.5) {
        // A fall droplet: a bright core, a soft rim and a glint, no outline.
        if (smooth_fx) {
            let r = length(in.uv*2.0 - 1.0);
            if (r > 1.0) { discard; }
            a = mix(0.95, 0.0, smoothstep(0.55, 1.0, r));
            rgb = mix(vec3<f32>(0.86, 0.95, 1.0), vec3<f32>(1.25), 1.0 - smoothstep(0.0, 0.3, length(in.uv - 0.3)));
        } else {
            let tx = clamp(floor(in.uv*8.0), vec2<f32>(0.0), vec2<f32>(7.0));
            let r = length((tx + 0.5)/4.0 - 1.0);
            if (r > 0.95) { discard; }
            a = select(0.7, 0.95, r < 0.55);
            rgb = vec3<f32>(0.86, 0.95, 1.0);
            if (tx.x == 2.0 && tx.y == 2.0) { rgb = vec3<f32>(1.25); }
        }
    } else if (in.cell < -0.5) {
        // A spray drop: the soft dot, alpha (1 - r)^2, smooth or as the
        // 32-texel byte it was baked as.
        if (smooth_fx) {
            let k = clamp(1.0 - length(in.uv*2.0 - 1.0), 0.0, 1.0);
            a = k*k;
        } else {
            let texel = clamp(floor(in.uv*32.0), vec2<f32>(0.0), vec2<f32>(31.0));
            let r = length((texel + 0.5)/32.0*2.0 - 1.0);
            let k = clamp(1.0 - r, 0.0, 1.0);
            a = floor(k*k*255.0)/255.0;
        }
    } else {
        // A mist puff: its row, and the two frames either side of its age
        // cross-faded, so it never changes shape in one frame.
        let row = floor(in.cell/4.0 + 1e-4);
        let age = clamp(in.cell - row*4.0, 0.0, 3.0);
        let f0 = floor(age);
        let f1 = min(f0 + 1.0, 3.0);
        let w = age - f0;
        var tx = clamp(floor(in.uv*32.0), vec2<f32>(0.0), vec2<f32>(31.0)) + 0.5;
        if (smooth_fx) { tx = clamp(in.uv*32.0, vec2<f32>(0.75), vec2<f32>(31.25)); }
        let uv0 = (vec2<f32>(f0, row)*32.0 + tx)/128.0;
        let uv1 = (vec2<f32>(f1, row)*32.0 + tx)/128.0;
        let tex = mix(textureSampleLevel(fall_puffs, fall_puffs_sampler, uv0, 0.0),
            textureSampleLevel(fall_puffs, fall_puffs_sampler, uv1, 0.0), w);
        rgb = tex.rgb;
        a = tex.a*view.fall_splash.y;
    }
    var colour = rgb*in.colour.rgb + rgb*fall_lamps(in.body, safe_normal(in.body));
    let eye = view.camera_time.xyz;
    let here = length(view.planet_center.xyz + in.body - eye);
    let behind = splash_behind_m(screen, ground, water_d);
    if (view.fx.y > 0.75) {
        colour = splash_murk(colour, select(behind, 100000.0, behind > 1.0e8));
    }
    // The wall blend (M35): a soft particle, faded as it nears the ground,
    // a wall or the water behind it, so a quad cutting into them leaves no
    // hard edge and the mist meets the pool softly.
    if (view.fall_splash.x > 0.0) {
        a *= smoothstep(0.0, 1.0, clamp((behind - here)/view.fall_splash.x, 0.0, 1.0));
    }
    return vec4<f32>(colour, clamp(in.colour.a*a, 0.0, 1.0));
}

// ---- Bubbles under a fall's foot (`water-flow` 6.6; M35's `bubblePoints`) ---
// Small bubbles rising through the pool where the fall lands: born under the
// falling column and round it, deeper and more of them the further the water
// fell, rising and wobbling, gone at the surface. All on the GPU, from the
// feet in the view: a camera-facing quad per bubble, its place a function of
// the time. Drawn after the water, so each takes the water's transmission
// over it itself: the mockup drew them under the water, which mixed them by
// the pool's own colour; the colour here is that mix, so the result is the
// same.

// `planet_water::BUBBLES_MAX`: the most bubbles under one foot.
const BUBBLES_MAX: u32 = 256u;

struct BubbleOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) body: vec3<f32>,
    @location(2) alpha: f32,
    @location(3) @interpolate(flat) foot: u32,
}

fn bubble_hash(n: u32) -> f32 {
    var h = n*747796405u + 2891336453u;
    h = ((h >> ((h >> 28u) + 4u)) ^ h)*277803737u;
    h = (h >> 22u) ^ h;
    return f32(h & 0xffffu)/65536.0;
}

@vertex
fn bubble_vertex(@builtin(vertex_index) vertex: u32) -> BubbleOut {
    var out: BubbleOut;
    // Nothing drawn: outside the clip volume.
    out.clip = vec4<f32>(0.0, 0.0, 2.0, 1.0);
    out.uv = vec2<f32>(0.0);
    out.body = vec3<f32>(0.0);
    out.alpha = 0.0;
    out.foot = 0u;
    let i = vertex/(BUBBLES_MAX*6u);
    let b = (vertex/6u)%BUBBLES_MAX;
    let k = vertex%6u;
    if (i >= min(u32(view.fall_look.x), FALL_FEET)) { return out; }
    let foot = view.feet[i];
    let more = view.feet_more[i];
    let h = max(foot.w, 0.0);
    let count = min(u32(round((60.0 + 14.0*h)*view.fall_more.w)), BUBBLES_MAX);
    if (b >= count || more.z <= 0.0) { return out; }
    let salt = u32(more.y*65535.0)*7919u + b*4u;
    let seed = vec4<f32>(bubble_hash(salt), bubble_hash(salt + 1u), bubble_hash(salt + 2u), bubble_hash(salt + 3u));
    let seconds = view.camera_time.w/max(view.aer_color.w, 1e-3);
    let depth = max(more.x, 0.2);
    let d0 = mix(0.25, max(min(depth - 0.15, 0.6 + 0.22*h), 0.25), seed.y);
    let speed = 0.35 + 0.35*seed.z;
    let life = d0/speed + 0.4;
    let tl = fract(seconds/life + seed.x)*life;
    let ang = seed.w*6.2831853;
    let rad = pow(fract(seed.x*7.31 + seed.y*3.7), 0.7)*(1.2 + 0.09*h);
    let y = -d0 + speed*tl;
    let visible = smoothstep(0.0, 0.25, tl)*select(0.0, 1.0, y < -0.04)*more.z;
    if (visible <= 0.0) { return out; }
    let up = safe_normal(foot.xyz);
    let east = safe_normal(cross(vec3<f32>(0.0, 1.0, 0.0), up));
    let north = cross(up, east);
    let c = foot.xyz
        + east*(cos(ang)*rad + 0.08*sin(tl*7.0 + seed.z*20.0))
        + north*(sin(ang)*rad + 0.08*cos(tl*6.0 + seed.w*20.0))
        + up*min(y, -0.04);
    // The camera's right and up: the view's rows.
    let m = view.clip_from_local;
    let right = safe_normal(vec3<f32>(m[0].x, m[1].x, m[2].x));
    let cam_up = safe_normal(vec3<f32>(m[0].y, m[1].y, m[2].y));
    var half = 0.5*(0.05 + 0.09*seed.z);
    // At least a couple of pixels across, as the mockup's points are.
    let c0 = m*vec4<f32>(view.planet_center.xyz + c, 1.0);
    let c1 = m*vec4<f32>(view.planet_center.xyz + c + cam_up*half, 1.0);
    if (c0.w <= 1e-4 || c1.w <= 1e-4) { return out; }
    let px = abs(c1.y/c1.w - c0.y/c0.w)*0.5*view.fall_splash.w;
    half *= max(1.0, 1.0/max(px, 1e-3));
    let corner = select(select(k, k - 2u, k >= 3u), 0u, k == 3u);
    let sx = select(-1.0, 1.0, corner == 1u || corner == 2u);
    let sy = select(-1.0, 1.0, corner >= 2u);
    let p = c + (right*sx + cam_up*sy)*half;
    out.clip = m*vec4<f32>(view.planet_center.xyz + p, 1.0);
    out.uv = vec2<f32>(0.5*(sx + 1.0), 0.5*(1.0 - sy));
    out.body = c;
    out.alpha = visible;
    out.foot = i;
    return out;
}

@fragment
fn bubble_fragment(in: BubbleOut) -> @location(0) vec4<f32> {
    let screen = in.clip.xy/vec2<f32>(textureDimensions(scene_depth));
    let ground = load_depth(screen);
    if (ground > in.clip.z + 1e-6) { discard; }
    // Hidden by water in front of it, but not by the pool's own surface over
    // it: the fall's column or another pool is.
    let foot = view.feet[in.foot];
    let foot_r = length(foot.xyz);
    let water_d = water_depth_at(in.clip.xy);
    if (water_d > in.clip.z + 1e-6) {
        let w = reconstruct_local(screen, water_d) - view.planet_center.xyz;
        let rel = w - foot.xyz;
        let up = dot(rel, safe_normal(foot.xyz));
        let across = length(rel - safe_normal(foot.xyz)*up);
        if (abs(up) > 0.25 || across > 2.5 + 0.1*max(foot.w, 0.0)) { discard; }
    }
    // An 8-texel bubble: a bright rim, a clear middle, a glint up and to the
    // left. No outline: the rim is the bubble.
    let smooth_fx = view.fall_splash.z > 0.5;
    var tx = in.uv*8.0 - 0.5;
    if (!smooth_fx) { tx = floor(in.uv*8.0); }
    let d = (tx + 0.5)/4.0 - 1.0;
    let r = length(d);
    if (r > 1.0) { discard; }
    let rim = smoothstep(0.55, 0.8, r);
    var a = mix(0.28, 0.9, rim);
    var c = vec3<f32>(0.86, 0.97, 1.0);
    if (!smooth_fx && tx.x == 2.0 && tx.y == 2.0) { c = vec3<f32>(1.2); a = 1.0; }
    if (smooth_fx) {
        a *= 1.0 - smoothstep(0.85, 1.0, r);
        let g = 1.0 - smoothstep(0.0, 0.3, length(d + 0.4));
        c = mix(c, vec3<f32>(1.2), g);
        a = max(a, g);
    }
    let radial = safe_normal(in.body);
    let lit = view.water_light.rgb*0.72 + fall_lamps(in.body, radial);
    var col = c*lit;
    let eye = view.camera_time.xyz;
    let travel = length(view.planet_center.xyz + in.body - eye);
    if (view.fx.y > 0.75) {
        col = splash_murk(col, travel);
    } else {
        // Seen from above, a bubble is under the water: it takes the
        // aerated teal and fades by its own depth below the surface, then
        // the water's transmission over it (the pool's depth along the
        // view, as the water's own fragment measures it) mixes it with the
        // water's lit colour.
        let below = max(foot_r - length(in.body), 0.0);
        let kd = exp(-1.6*below);
        col = mix(view.aer_color.rgb*lit, col, 0.25 + 0.5*kd);
        a *= 0.25 + 0.55*kd;
        let look = safe_normal(eye - view.planet_center.xyz - in.body);
        let path = max(view.feet_more[in.foot].x, 0.0)/max(dot(look, radial), 0.15);
        let through = exp(-view_extinction()*path);
        let depth_under = max(view.feet_more[in.foot].x, 0.0);
        let albedo = mix(view.shallow.rgb, view.deep_color.rgb,
            smoothstep(0.0, max(view.water_light.w, 1e-3), depth_under));
        col = col*through + albedo*view.water_light.rgb*(1.0 - through);
    }
    return vec4<f32>(col, clamp(a*in.alpha, 0.0, 1.0));
}
