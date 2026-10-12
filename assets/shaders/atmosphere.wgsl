// Tenebris 8-view / 4-sun single-scattering atmosphere, ported to WGSL.
// Camera basis and camera_body are PLANET-LOCAL; no global f32 coordinates.
// This is a sky-only pass: stencil/depth test EQUAL reverse-Z clear depth 0.
// Terrain fog is separate; do not alpha-blend this over opaque terrain.
struct AtmosphereView {
    right_tan_half: vec4<f32>,
    up_aspect: vec4<f32>,
    forward: vec4<f32>,
    camera_body: vec4<f32>,
    sun: vec4<f32>,
    radii: vec4<f32>,          // planet radius, outer radius, normalized scale height, spare
    scatter: vec4<f32>,        // Rayleigh scale, Mie scale, g, sun intensity
    wavelengths: vec4<f32>,    // RGB 1/lambda^4 ratios, sunset strength
    sunset_tint: vec4<f32>,
    sunset_glow: vec4<f32>,
    sunset_horizon: vec4<f32>,
}
@group(0) @binding(0) var<uniform> view: AtmosphereView;
struct VertexOut { @builtin(position) clip: vec4<f32>, @location(0) direction: vec3<f32> }
fn safe_normal(v: vec3<f32>) -> vec3<f32> { return v*inverseSqrt(max(dot(v,v),1e-12)); }
@vertex
fn vertex(@builtin(vertex_index) id: u32) -> VertexOut {
    let p = array<vec2<f32>,3>(vec2<f32>(-1.,-1.),vec2<f32>(3.,-1.),vec2<f32>(-1.,3.))[id];
    var out: VertexOut;
    out.clip = vec4<f32>(p,0.,1.);
    out.direction = view.forward.xyz + view.right_tan_half.xyz*(p.x*view.right_tan_half.w*view.up_aspect.w)
        +view.up_aspect.xyz*(p.y*view.right_tan_half.w);
    return out;
}
fn ray_sphere(origin: vec3<f32>, direction: vec3<f32>, radius: f32) -> vec2<f32> {
    let b = dot(origin,direction);
    let c = dot(origin,origin)-radius*radius;
    let discriminant = b*b-c;
    if (discriminant < 0.) { return vec2<f32>(-1.); }
    let root = sqrt(max(discriminant,0.));
    return vec2<f32>(-b-root,-b+root);
}
fn density(altitude: f32) -> f32 { return exp(-clamp(altitude,0.,1.)/max(view.radii.z,1e-4)); }
fn optical_depth(origin: vec3<f32>, direction: vec3<f32>, distance: f32) -> f32 {
    let thickness = max(view.radii.y-view.radii.x,1e-3);
    let step_length = max(distance,0.)*0.25;
    var depth = 0.;
    for (var i=0u; i<4u; i++) {
        let altitude = (length(origin+direction*(step_length*(f32(i)+0.5)))-view.radii.x)/thickness;
        depth += density(altitude)*step_length/thickness;
    }
    return depth;
}
fn rayleigh_phase(cosine: f32) -> f32 { return 0.75*(1.+cosine*cosine); }
fn mie_phase(cosine: f32, anisotropy: f32) -> f32 {
    let g = clamp(anisotropy,-0.99,0.99);
    return (1.-g*g)/(12.5663706*pow(max(1.+g*g-2.*g*cosine,1e-4),1.5));
}
@fragment
fn fragment(in: VertexOut) -> @location(0) vec4<f32> {
    if (view.radii.x <= 0. || view.radii.y <= view.radii.x) { return vec4<f32>(0.); }
    let direction = safe_normal(in.direction);
    let sun = safe_normal(view.sun.xyz);
    let camera = view.camera_body.xyz;
    let hit = ray_sphere(camera,direction,view.radii.y);
    if (hit.y < 0.) { return vec4<f32>(0.); }
    let start = max(hit.x,0.);
    var end = hit.y;
    let ground = ray_sphere(camera,direction,view.radii.x).x;
    if (ground > 0.) { end = min(end,ground); }
    if (start >= end) { return vec4<f32>(0.); }
    let step_length = (end-start)*0.125;
    let normalized_step = step_length/max(view.radii.y-view.radii.x,1e-3);
    let jitter = fract(sin(dot(in.clip.xy,vec2<f32>(127.1,311.7)))*43758.5453)*0.5;
    let rayleigh = max(view.wavelengths.rgb,vec3<f32>(0.))*max(view.scatter.x,0.);
    let mie = max(view.scatter.y,0.);
    var integrated = vec3<f32>(0.);
    var view_depth = 0.;
    for (var i=0u; i<8u; i++) {
        let sample_position = camera + direction*(start+step_length*(f32(i)+jitter));
        let radius = length(sample_position);
        if (radius < view.radii.x) { continue; }
        let altitude = (radius-view.radii.x)/max(view.radii.y-view.radii.x,1e-3);
        let segment = density(altitude)*normalized_step;
        view_depth += segment;
        // Shadowed samples still contribute extinction, as in Tenebris.
        if (ray_sphere(sample_position,sun,view.radii.x).x > 0.) { continue; }
        let sun_length = max(ray_sphere(sample_position,sun,view.radii.y).y,0.);
        let sun_depth = optical_depth(sample_position,sun,sun_length);
        integrated += exp(-(rayleigh+vec3<f32>(mie))*(view_depth+sun_depth))*segment;
    }
    let cosine = clamp(dot(direction,sun),-1.,1.);
    let up = safe_normal(camera);
    let sun_height = dot(sun,up);
    // Original reversed smoothstep edges rewritten into defined WGSL domains.
    let sunset = (1.-smoothstep(0.,0.18,sun_height))*smoothstep(-0.32,-0.1,sun_height);
    let intensity = max(view.scatter.w,0.);
    var scattered = integrated*(rayleigh*rayleigh_phase(cosine)+vec3<f32>(mie*mie_phase(cosine,view.scatter.z)))*intensity;
    scattered *= mix(vec3<f32>(1.),view.sunset_tint.rgb,clamp(sunset*view.wavelengths.w,0.,1.));
    let glow = view.sunset_glow.rgb*pow(max(cosine,0.),3.)*sunset*intensity*0.22;
    let band = view.sunset_horizon.rgb*pow(clamp(1.-abs(dot(direction,up)),0.,1.),3.)*sunset*intensity*0.11;
    // Linear HDR output for Bevy's final tonemapper: source's 1-exp(-x) is
    // deliberately not applied twice. Premultiplied alpha composition.
    let color = max(scattered+glow+band,vec3<f32>(0.));
    let opacity = clamp(1.-exp(-view_depth*(dot(rayleigh,vec3<f32>(1./3.))+mie)),0.,1.);
    return vec4<f32>(color,opacity);
}
