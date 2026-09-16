// Tenebris water GLSL -> WGSL; mathematical provenance in docs/shader-port.md.
// Reverse-Z depth (near=1, sky=0), linear scene color, resolved single-sample
// scene textures. All positions/matrices use ONE rebased local frame.
struct WaterView {
    clip_from_local: mat4x4<f32>,
    local_from_clip: mat4x4<f32>,
    camera_time: vec4<f32>,   // xyz local camera, w seconds (wrap time on host)
    planet_center: vec4<f32>, // xyz same local frame, w sea radius
    sun: vec4<f32>,           // xyz toward sun, w specular intensity
    waves: vec4<f32>,         // amplitude m, spatial scale, speed, normal strength
    refraction: vec4<f32>,    // strength, max UV offset, slope cap, max path length m
    absorption: vec4<f32>,    // RGB absorption/m; w dark-side floor
    deep_color: vec4<f32>,
    horizon_color: vec4<f32>, // rgb sky horizon; w minimum reflection
    zenith_color: vec4<f32>,
    foam_color: vec4<f32>,
    foam: vec4<f32>,          // height low/high, slope low/high
    lighting: vec4<f32>,      // foam strength, specular power, block gain, spare
    fog: vec4<f32>,           // rgb; w density/m
    limits: vec4<f32>,        // x fog ceiling (<1), y terminator low, z high, w spare
}
@group(0) @binding(0) var<uniform> view: WaterView;
@group(1) @binding(0) var scene_color: texture_2d<f32>;
@group(1) @binding(1) var scene_depth: texture_depth_2d;
@group(1) @binding(2) var scene_sampler: sampler;

struct VertexIn {
    @location(0) body_position: vec3<f32>,
    @location(1) face_normal: vec3<f32>,
    @location(2) baked_rgb_sky: vec4<f32>,
}
struct VertexOut {
    @builtin(position) clip: vec4<f32>,
    @location(0) local_position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) baked_rgb_sky: vec4<f32>,
    @location(3) body_position: vec3<f32>,
}
fn safe_normal(v: vec3<f32>) -> vec3<f32> { return v * inverseSqrt(max(dot(v,v),1e-12)); }
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
@vertex
fn vertex(in: VertexIn) -> VertexOut {
    let t = view.camera_time.w*view.waves.z;
    let p = in.body_position;
    let scale = view.waves.y;
    let wave = (sin(t*0.9+p.x*1.4*scale+p.z*0.6*scale)*0.18
        +sin(t*1.3-p.x*0.7*scale+p.z*1.2*scale)*0.12
        +sin(t*1.7+p.x*2.3*scale-p.z*1.9*scale)*0.06)*view.waves.x;
    let radial = safe_normal(p);
    let top = smoothstep(0.35,0.9,dot(safe_normal(in.face_normal),radial));
    let displaced = p + radial * wave * top;
    var out: VertexOut;
    out.body_position = displaced;
    out.local_position = view.planet_center.xyz + displaced;
    out.clip = view.clip_from_local*vec4<f32>(out.local_position,1.0);
    out.normal = in.face_normal;
    out.baked_rgb_sky = in.baked_rgb_sky;
    return out;
}
fn load_depth(uv: vec2<f32>) -> f32 {
    let size = textureDimensions(scene_depth);
    let coord = clamp(vec2<i32>(uv*vec2<f32>(size)),vec2<i32>(0),vec2<i32>(size)-1);
    return textureLoad(scene_depth,coord,0);
}
fn reconstruct_local(uv: vec2<f32>, depth: f32) -> vec3<f32> {
    let homogeneous = view.local_from_clip * vec4<f32>(uv.x*2.0-1.0,1.0-uv.y*2.0,depth,1.0);
    // Caller avoids depth=0 infinite-far reconstruction.
    return homogeneous.xyz / max(abs(homogeneous.w),1e-8) * sign(homogeneous.w);
}
@fragment
fn fragment(in: VertexOut, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    let radial = safe_normal(in.body_position);
    let face = safe_normal(in.normal);
    let p = in.body_position * view.waves.y;
    let t = view.camera_time.w * view.waves.z;
    let height = fbm3(p,t);
    var gradient = vec3<f32>(fbm3(p+vec3<f32>(0.08,0.,0.),t)-height,
        fbm3(p+vec3<f32>(0.,0.08,0.),t)-height,
        fbm3(p+vec3<f32>(0.,0.,0.08),t)-height)*12.5;
    gradient -= face * dot(gradient,face);
    let raw_slope = length(gradient);
    gradient *= min(1.0,max(view.refraction.z,0.0)/max(raw_slope,1e-6));
    let normal = safe_normal(face - gradient*view.waves.w);
    let uv = in.clip.xy / vec2<f32>(textureDimensions(scene_depth));
    let own_depth = load_depth(uv);
    // Bevy/WebGPU reversed depth: larger depth is nearer.
    if (own_depth > in.clip.z + 1e-6) { discard; }
    let screen_gradient = (view.clip_from_local*vec4<f32>(gradient,0.0)).xy;
    var offset = screen_gradient * vec2<f32>(1.,-1.) * view.refraction.x;
    offset *= min(1.0,max(view.refraction.y,0.0)/max(length(offset),1e-6));
    var refracted_uv = clamp(uv+offset,vec2<f32>(0.),vec2<f32>(1.));
    var refracted_depth = load_depth(refracted_uv);
    let own_sky = own_depth <= 1e-7;
    let sampled_sky = refracted_depth <= 1e-7;
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
    if (!front) {
        let underwater = mix(view.deep_color.rgb,scene,exp(-absorption*camera_distance));
        return vec4<f32>(mix(view.deep_color.rgb,underwater,smoothstep(0.55,0.75,dot(-look,normal))),1.);
    }
    let reflection_height = clamp(dot(reflect(-look,normal),radial),0.,1.);
    let fresnel = (0.02+0.98*pow(1.0-max(dot(look,normal),0.0),5.0))
        * mix(clamp(view.horizon_color.w,0.,1.),1.,reflection_height);
    let reflected = mix(view.horizon_color.rgb,view.zenith_color.rgb,reflection_height);
    let transmitted = mix(view.deep_color.rgb,scene,exp(-absorption*path_length));
    let foam = clamp(max(smoothstep(view.foam.x,max(view.foam.y,view.foam.x+1e-4),height),
        smoothstep(view.foam.z,max(view.foam.w,view.foam.z+1e-4),raw_slope))*view.lighting.x,0.,1.);
    var color = mix(mix(transmitted,reflected,fresnel),view.foam_color.rgb,foam);
    let sun = safe_normal(view.sun.xyz);
    let half_vector = safe_normal(look+sun);
    color += vec3<f32>(pow(max(dot(normal,half_vector),0.),max(view.lighting.y,1.))*view.sun.w);
    let daylight = smoothstep(view.limits.y,max(view.limits.z,view.limits.y+1e-4),dot(radial,sun));
    color *= mix(clamp(view.absorption.w,0.,1.),1.,daylight*clamp(in.baked_rgb_sky.w,0.,1.));
    color += in.baked_rgb_sky.rgb*view.lighting.z*(fresnel+foam);
    let fog = min(1.0-exp(-camera_distance*max(view.fog.w,0.)),clamp(view.limits.x,0.,0.99));
    return vec4<f32>(mix(color,view.fog.rgb,fog),1.);
}
