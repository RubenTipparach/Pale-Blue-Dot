// The water system's GPU side: Tenebris's water.vs/fs.glsl and composite.fs.glsl
// in WGSL, term for term, with the cap pulled from the planet's persistent Cell
// record instead of a water vertex stream. Provenance and the deliberate
// differences are in docs/shader-port.md.
//
// Reverse-Z depth (near=1, sky=0), linear scene colour, the resolved
// single-sample scene textures. Positions are body-local; `planet_center` is
// where that frame sits in the render frame (zero today).
struct Cell {
    direction_height: vec4<f32>,
    corners: array<vec4<f32>,6>,
    metadata: vec4<u32>,
    owner_a: vec4<f32>,
    owner_b: vec4<f32>,
    floors: vec4<f32>,
    spare: vec4<f32>,
}
struct WaterView {
    clip_from_local: mat4x4<f32>,
    local_from_clip: mat4x4<f32>,
    camera_time: vec4<f32>,   // xyz camera in the local frame, w seconds * time_scale
    planet_center: vec4<f32>, // xyz body centre in the local frame, w sea radius m
    sun: vec4<f32>,           // xyz toward the sun, w specular intensity
    waves: vec4<f32>,         // swell amplitude m, swell frequency, swell speed, wave steepness
    ripple: vec4<f32>,        // ripple scale cells/m, ripple speed, rain ripple cells/m, rain ripple strength
    refraction: vec4<f32>,    // strength, max uv offset, slope cap, max path m
    absorption: vec4<f32>,    // rgb per metre, w night floor
    deep_color: vec4<f32>,    // rgb, w falling-face flow speed
    horizon_color: vec4<f32>, // rgb, w horizon (grazing) reflection floor
    zenith_color: vec4<f32>,
    night_sky: vec4<f32>,      // the sky the sheet mirrors at night  // rgb, w spare
    foam_color: vec4<f32>,    // rgb, w foam intensity
    foam_crest: vec4<f32>,    // lo, hi, weight, spare
    foam_slope: vec4<f32>,    // lo, hi, weight, spare
    sun_tint: vec4<f32>,      // rgb, w specular power
    fog_night: vec4<f32>,     // rgb sky colour at night, w fog density per metre
    fog_day: vec4<f32>,       // rgb sky colour by day, w fog height m
    limits: vec4<f32>,        // fog ceiling, terminator lo, terminator hi, fog mix
    fx: vec4<f32>,            // underwater distortion, submersion 0/0.5/1, rain, emerge
    lens: vec4<f32>,          // droplet density, refraction, speed, size
    screen: vec4<f32>,        // aspect, surface band m, wet blur, detail fade
    lod: vec4<f32>,           // xyz player direction, w base level
    bands: vec4<f32>,         // cos(band radius / R) per fine level, coarsest first
}
@group(0) @binding(0) var<uniform> view: WaterView;
@group(0) @binding(1) var<storage,read> cells: array<Cell>;
@group(0) @binding(2) var<storage,read> water: array<u32>;
@group(0) @binding(3) var<storage,read> flow: array<vec2<f32>>;
@group(1) @binding(0) var scene_color: texture_2d<f32>;
#ifdef MULTISAMPLED
@group(1) @binding(1) var scene_depth: texture_depth_multisampled_2d;
#else
@group(1) @binding(1) var scene_depth: texture_depth_2d;
#endif
@group(1) @binding(2) var scene_sampler: sampler;

struct VertexOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) local_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) sky_light: f32,
    @location(3) body_position: vec3<f32>,
    @location(4) flow_uv: vec2<f32>,
    @location(5) @interpolate(flat) level: u32,
    @location(6) @interpolate(flat) owner_a: vec3<f32>,
    @location(7) @interpolate(flat) owner_b: vec3<f32>,
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
    let id = water[instance];
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
    let p = ray*sea;
    // Tenebris's three-sine swell on every vertex, in body-local metres so a
    // floating-origin shift cannot shift the phase.
    let t = view.camera_time.w*view.waves.z;
    let scale = view.waves.y;
    let wave = (sin(t*0.9+p.x*1.4*scale+p.z*0.6*scale)*0.18
        +sin(t*1.3-p.x*0.7*scale+p.z*1.2*scale)*0.12
        +sin(t*1.7+p.x*2.3*scale-p.z*1.9*scale)*0.06)*view.waves.x;
    let displaced = p + safe_normal(p)*wave;
    var out: VertexOut;
    out.body_position = displaced;
    out.local_position = view.planet_center.xyz + displaced;
    out.clip = view.clip_from_local*vec4<f32>(out.local_position,1.0);
    out.normal = axis;
    // The sheet is open sky by construction in a heightfield; the cell's
    // baked occlusion is the seabed's and would tile the sea by column.
    out.sky_light = 1.0;
    out.flow_uv = vec2<f32>(0.0);
    if (id < arrayLength(&flow)) { out.flow_uv = flow[id]; }
    out.level = level;
    out.owner_a = cell.owner_a.xyz;
    out.owner_b = cell.owner_b.xyz;
    return out;
}
fn water_band_cos(level: u32) -> f32 {
    let k = level - u32(view.lod.w) - 1u;
    if (k == 0u) { return view.bands.x; }
    if (k == 1u) { return view.bands.y; }
    if (k == 2u) { return view.bands.z; }
    return view.bands.w;
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
// The terrain's own haze, so the sheet and the ground fog out together. The
// terrain shader still carries these as literals; lifting them into one
// uniform is task 2 of preview-scale-and-shader-parity, and until then the
// values here are the same numbers fed from Rust.
fn distance_fog(local_position: vec3<f32>, radial: vec3<f32>, sun: vec3<f32>) -> vec4<f32> {
    let camera_body = view.camera_time.xyz - view.planet_center.xyz;
    let altitude = max(length(camera_body) - view.planet_center.w, 0.0);
    let air = exp(-altitude/max(view.fog_day.w,1.0));
    let sun_elevation = dot(radial,sun);
    let daylight = smoothstep(view.limits.y,max(view.limits.z,view.limits.y+1e-4),sun_elevation);
    let d = length(view.camera_time.xyz - local_position);
    let amount = (1.0-exp(-d*max(view.fog_night.w,0.0)))*air*daylight*view.limits.w;
    let sky = mix(view.fog_night.rgb,view.fog_day.rgb,max(sun_elevation,0.0));
    return vec4<f32>(sky, min(amount, clamp(view.limits.x,0.0,0.99)));
}

@fragment
fn fragment(in: VertexOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    let radial = safe_normal(in.body_position);
    // The same partition the terrain applies: a midpoint sheet split between
    // a fine and a coarse owner draws only the fine half.
    if (in.level > u32(view.lod.w)) {
        let a_fine = dot(in.owner_a, view.lod.xyz) > water_band_cos(in.level);
        let b_fine = dot(in.owner_b, view.lod.xyz) > water_band_cos(in.level);
        if (a_fine != b_fine) {
            let nearer_a = dot(radial, in.owner_a) >= dot(radial, in.owner_b);
            if ((nearer_a && !a_fine) || (!nearer_a && !b_fine)) { discard; }
        }
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
    let advected = in.body_position
        + radial*(time*view.deep_color.w*(1.0-horizontal))
        - (tangent_u*in.flow_uv.x + tangent_v*in.flow_uv.y)*time*0.35*horizontal;
    let p = advected*view.ripple.x;
    let t = time*view.ripple.y;
    // Not Tenebris's: once the fbm's features fall under a pixel, its
    // point-sampled height and gradient are noise, and Fresnel and specular
    // turn that noise into white sparkle across the whole far sea. Fade both
    // by the per-pixel footprint of the noise coordinate; `detail_fade` 0 is
    // the unfiltered original.
    let detail = 1.0/(1.0+length(fwidth(p))*max(view.screen.w,0.0));
    let height = fbm3(p,t)*detail;
    var gradient = vec3<f32>(fbm3(p+vec3<f32>(0.08,0.,0.),t)-fbm3(p,t),
        fbm3(p+vec3<f32>(0.,0.08,0.),t)-fbm3(p,t),
        fbm3(p+vec3<f32>(0.,0.,0.08),t)-fbm3(p,t))*12.5*detail;
    gradient -= radial*dot(gradient,radial);
    if (view.fx.z > 0.001) {
        let uv = vec2<f32>(dot(in.body_position,tangent_u),dot(in.body_position,tangent_v))*view.ripple.z;
        let g = rain_ripple_grad(uv,time)*(view.fx.z*view.ripple.w);
        gradient += tangent_u*g.x + tangent_v*g.y;
    }
    // Slope cap BEFORE the gradient bends the normal: the rain kernel spikes it
    // to ~8x the waves and, uncapped, flipped the sheet into a flat mirror.
    // Foam still reads the raw gradient so storm foam keeps its bite.
    let raw_slope = length(gradient);
    var capped = gradient*min(1.0,max(view.refraction.z,0.0)/max(raw_slope,1e-6));
    let normal = safe_normal(radial - capped*view.waves.w);
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
    if (refracted_depth > 1e-7) {
        let background = reconstruct_local(refracted_uv,refracted_depth);
        path_length = min(path_length,max(length(background-view.camera_time.xyz)-camera_distance,0.0));
    }
    let absorption = max(view.absorption.rgb,vec3<f32>(0.));
    // Seen from below: decided by where the camera is, not by the polygon's
    // winding. The swell tilts a 2.8 m cap by several degrees, more than the
    // grazing angle at the horizon, so winding alone could flip a cap seen
    // from above onto the underwater path. The pipeline culls nothing, so
    // `front` is only a tie-break for a camera exactly on the sheet.
    let camera_body = view.camera_time.xyz - view.planet_center.xyz;
    let camera_radius = length(camera_body);
    let sheet_radius = length(in.body_position);
    let below = camera_radius < sheet_radius || (camera_radius == sheet_radius && !front);
    if (below) {
        // Seen from below: the deep colour by camera distance, and Snell's window.
        let underwater = mix(view.deep_color.rgb,scene,exp(-absorption*camera_distance));
        return vec4<f32>(mix(view.deep_color.rgb,underwater,smoothstep(0.55,0.75,dot(-look,normal))),1.);
    }
    let reflection_height = clamp(dot(reflect(-look,normal),radial),0.,1.);
    let fresnel = (0.02+0.98*pow(1.0-max(dot(look,normal),0.0),5.0))
        * mix(clamp(view.horizon_color.w,0.,1.),1.,reflection_height);
    // A reflection is of the SKY, so it is the sky's colour, not an authored
    // daytime gradient under an ambient floor. Left that way the sea was a lit
    // blue sheet under a black sky, brighter than the land beside it. It takes
    // the same day/night sky the distance fog already mixes, which is one
    // colour written down once and used by both.
    let sun_elevation = dot(radial,sun_direction);
    let daylight = smoothstep(view.limits.y,max(view.limits.z,view.limits.y+1e-4),sun_elevation);
    let day_sky = mix(view.horizon_color.rgb,view.zenith_color.rgb,reflection_height);
    let reflected = mix(view.night_sky.rgb,day_sky,daylight);
    let transmitted = mix(view.deep_color.rgb,scene,exp(-absorption*path_length));
    let crest = smoothstep(view.foam_crest.x,max(view.foam_crest.y,view.foam_crest.x+1e-4),height)*view.foam_crest.z;
    let slope = smoothstep(view.foam_slope.x,max(view.foam_slope.y,view.foam_slope.x+1e-4),raw_slope)*view.foam_slope.z;
    let foam = clamp(max(crest,slope)*view.foam_color.w,0.,1.);
    // The night floor is an AMBIENT level: it belongs to the light that reaches
    // the water body and the foam on it, and not to the reflection, which
    // carries the sky's own level already. Applied to both, as it was, the sea
    // at night was dimmed twice and went black at the horizon, where a mirror
    // should be closest to the sky it mirrors. At day it is one, so nothing
    // above the terminator moves.
    let lit = mix(clamp(view.absorption.w,0.,1.),1.,daylight*clamp(in.sky_light,0.,1.));
    var color = mix(mix(transmitted*lit,reflected,fresnel),view.foam_color.rgb*lit,foam);
    let half_vector = safe_normal(look+sun_direction);
    color += view.sun_tint.rgb
        *pow(max(dot(normal,half_vector),0.),max(view.sun_tint.w,1.))
        *view.sun.w*lit;
    let fog = distance_fog(in.local_position,radial,sun_direction);
    return vec4<f32>(mix(color,fog.rgb,fog.a),1.);
}

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
fn view_ray(uv: vec2<f32>) -> Ray {
    // A point part way down the frustum gives the direction; sky pixels have
    // no depth to reconstruct at, so the direction never comes from one.
    let ahead = reconstruct_local(uv,0.5);
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
    let sea = view.planet_center.w;
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
    let mask = select(1.0-smoothstep(-band,band,pixel_alt),1.0,sky);
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
    }
    let absorption = max(view.absorption.rgb,vec3<f32>(0.));
    color = mix(color, mix(view.deep_color.rgb,color,exp(-absorption*travel)), wet);
    // Depth blur while the lens is wet: the distance softens, the foreground stays crisp.
    let dz_amt = max(view.fx.z,view.fx.w);
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
fn rd_drop_layer(uv_in: vec2<f32>, t: f32) -> vec2<f32> {
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
    let m = main_drop+droplets*r*trail_front;
    return vec2<f32>(m,trail)*(keep*fade);
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
fn rd_drops(uv: vec2<f32>, t: f32, l0: f32, l1: f32, l2: f32) -> vec2<f32> {
    let s = rd_static_drops(uv,t)*l0;
    let m1 = rd_drop_layer(uv,t)*l1;
    let m2 = rd_drop_layer(uv*1.85,t)*l2;
    var c = s+m1.x+m2.x;
    // Density above one stacks same-size layers rather than shrinking drops.
    let extra = clamp(view.lens.x-1.0,0.0,2.0);
    if (extra > 0.001) {
        let m3 = rd_drop_layer(uv+vec2<f32>(0.50,0.37),t);
        let m4 = rd_drop_layer(uv*1.85+vec2<f32>(0.27,0.61),t);
        c += (m3.x*l1+m4.x*l2)*min(extra,1.0);
    }
    c = smoothstep(0.3,1.0,c);
    return vec2<f32>(c,max(m1.y*l0,m2.y*l1));
}
fn refract_through_drops(uv: vec2<f32>, drop_uv: vec2<f32>, t: f32, l0: f32, l1: f32, l2: f32) -> vec4<f32> {
    let c = rd_drops(drop_uv,t,l0,l1,l2);
    let e = vec2<f32>(0.001,0.0);
    let cx = rd_drops(drop_uv+e,t,l0,l1,l2).x;
    let cy = rd_drops(drop_uv+e.yx,t,l0,l1,l2).x;
    let n = vec2<f32>(cx-c.x,cy-c.x);
    let refracted = textureSampleLevel(scene_color,scene_sampler,clamp(uv-n*view.lens.y,vec2(0.),vec2(1.)),0.0).rgb;
    return vec4<f32>(refracted, clamp(c.x,0.0,1.0));
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
    var drop_uv = uv-0.5;
    drop_uv.x *= max(view.screen.x,0.1);
    let t = view.camera_time.w*0.2*view.lens.z;
    let rain = clamp(view.fx.z,0.0,1.0);
    if (rain > 0.001) {
        let s = smoothstep(-0.5,1.0,rain)*2.0;
        let l1 = smoothstep(0.25,0.75,rain);
        let l2 = smoothstep(0.0,0.5,rain);
        let r = refract_through_drops(uv,drop_uv,t,s,l1,l2);
        color = mix(color,r.rgb,r.a*rain*above_water);
    }
    let emerge = clamp(view.fx.w,0.0,1.0);
    if (emerge > 0.001) {
        let r = refract_through_drops(uv,drop_uv,t,0.0,1.0,0.6);
        color = mix(color,r.rgb,r.a*emerge*above_water);
    }
    return vec4<f32>(color,1.0);
}
