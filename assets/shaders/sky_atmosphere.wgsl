// Integrated Bevy 0.18 material. Adapted from the validated atmosphere.wgsl
// and Tenebris's 8-view/4-sun single-scatter shader (see shader-port.md).
// The integrated material uses 16 view steps and a soft planetary penumbra
// to remove visible shadow bands found in the native orbital captures.
// Unlike the standalone asset, this consumes Bevy view/mesh bindings.
#import bevy_pbr::forward_io::VertexOutput
#import bevy_pbr::mesh_view_bindings::view

struct SkyParameters {
    center_radius: vec4<f32>,
    atmosphere: vec4<f32>,
    sun: vec4<f32>,
    scatter: vec4<f32>,
    clouds: vec4<f32>,
}
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> sky: SkyParameters;

fn sphere_hit(ro: vec3<f32>, rd: vec3<f32>, radius: f32) -> vec2<f32> {
    let b = dot(ro, rd);
    let d = b*b - dot(ro,ro) + radius*radius;
    if (d < 0.0) { return vec2<f32>(-1.0); }
    let root = sqrt(max(d, 0.0));
    return vec2<f32>(-b-root, -b+root);
}

fn air_density(p: vec3<f32>) -> f32 {
    let shell = max(sky.atmosphere.x - sky.center_radius.w, 1.0);
    let altitude = max(length(p)-sky.center_radius.w, 0.0) / shell;
    // The geometry encloses mountain summits; density, rather than a hard
    // sphere edge, makes the visible limb thin. Taper the last outer band.
    let outer_taper = 1.0-smoothstep(0.72,1.0,altitude);
    return exp(-altitude/max(sky.atmosphere.y, 0.01))*outer_taper;
}

fn sun_depth(p: vec3<f32>, direction: vec3<f32>) -> f32 {
    let ray = sphere_hit(p, direction, sky.atmosphere.x);
    let distance = max(ray.y, 0.0);
    let step_size = distance * 0.25;
    let shell = max(sky.atmosphere.x-sky.center_radius.w, 1.0);
    var depth = 0.0;
    for (var i=0u; i<4u; i++) {
        depth += air_density(p+direction*(step_size*(f32(i)+0.5)))*step_size/shell;
    }
    return depth;
}

fn sun_visibility(p: vec3<f32>, sun: vec3<f32>) -> f32 {
    let along_sun = dot(p,sun);
    if (along_sun >= 0.0) { return 1.0; }
    // For night-side samples, the nearest point on the forward sun ray is
    // its perpendicular distance to the body axis. Smooth that silhouette
    // instead of dropping whole view samples at a binary sphere intersection.
    let closest_radius = length(p-sun*along_sun);
    let penumbra = 36.0 + min(-along_sun,sky.center_radius.w)*0.006;
    return smoothstep(-penumbra,penumbra,closest_radius-sky.center_radius.w);
}

fn cloud_hash(p: vec3<f32>) -> f32 {
    var q = fract(p * vec3<f32>(0.1031, 0.11369, 0.13787));
    q += dot(q, q.yzx + 19.19);
    return fract((q.x+q.y)*q.z);
}

fn cloud_noise(p: vec3<f32>) -> f32 {
    let cell = floor(p);
    let f = fract(p);
    let w = f*f*(3.0-2.0*f);
    return mix(
        mix(mix(cloud_hash(cell), cloud_hash(cell+vec3<f32>(1.,0.,0.)),w.x),
            mix(cloud_hash(cell+vec3<f32>(0.,1.,0.)), cloud_hash(cell+vec3<f32>(1.,1.,0.)),w.x),w.y),
        mix(mix(cloud_hash(cell+vec3<f32>(0.,0.,1.)),cloud_hash(cell+vec3<f32>(1.,0.,1.)),w.x),
            mix(cloud_hash(cell+vec3<f32>(0.,1.,1.)),cloud_hash(cell+vec3<f32>(1.,1.,1.)),w.x),w.y),w.z);
}

@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    let camera = view.world_position - sky.center_radius.xyz;
    let inside = length(camera) < sky.atmosphere.x;
    if (inside == front) { discard; }
    let direction = normalize(in.world_position.xyz - view.world_position);
    let sun = normalize(sky.sun.xyz);
    let shell = max(sky.atmosphere.x-sky.center_radius.w, 1.0);
    let air_hit = sphere_hit(camera,direction,sky.atmosphere.x);
    let start = max(air_hit.x,0.0);
    var end = air_hit.y;
    let ground = sphere_hit(camera,direction,sky.center_radius.w);
    if (ground.x > 0.0) { end = min(end, ground.x); }
    if (end <= start) { discard; }

    let step_size = (end-start)/16.0;
    let beta_r = sky.scatter.xyz * sky.atmosphere.z;
    let beta_m = sky.atmosphere.w;
    var view_depth = 0.0;
    var integrated = vec3<f32>(0.0);
    // Stable midpoint integration; no frame noise swimming on pixel terrain.
    for (var i=0u; i<16u; i++) {
        let p = camera+direction*(start+step_size*(f32(i)+0.5));
        let segment = air_density(p)*step_size/shell;
        view_depth += segment;
        let visible_sun = sun_visibility(p,sun);
        if (visible_sun <= 0.0) { continue; }
        let light_depth = sun_depth(p,sun);
        let transmission = exp(-(beta_r+vec3<f32>(beta_m))*(view_depth+light_depth));
        integrated += transmission*segment*visible_sun;
    }
    let cosine = clamp(dot(direction,sun),-1.0,1.0);
    let rayleigh_phase = 0.75*(1.0+cosine*cosine);
    let g = clamp(sky.scatter.w,-0.95,0.95);
    let mie_phase = (1.0-g*g)/(12.5663706*pow(max(1.0+g*g-2.0*g*cosine,0.001),1.5));
    var scattered = integrated*(beta_r*rayleigh_phase+vec3<f32>(beta_m*mie_phase))*sky.sun.w;

    let up = normalize(camera);
    let sun_height = dot(up,sun);
    let dusk = (1.0-smoothstep(0.0,0.2,sun_height))*smoothstep(-0.28,-0.06,sun_height);
    let horizon = pow(clamp(1.0-abs(dot(direction,up)),0.0,1.0),5.0);
    let low_camera = 1.0-smoothstep(80.0,700.0,length(camera)-sky.center_radius.w);
    scattered = mix(scattered,scattered*vec3<f32>(2.1,0.65,0.24),dusk*0.65);
    scattered += vec3<f32>(0.8,0.24,0.055)*dusk*horizon*low_camera*0.18;
    let average_beta = dot(beta_r,vec3<f32>(1.0/3.0))+beta_m;
    var opacity = clamp(1.0-exp(-view_depth*average_beta),0.0,0.94);
    // Art-directed daylight veil keeps bright stars out of the surface sky.
    // It fades out with altitude, revealing the star field continuously.
    let veil_altitude = max(length(camera)-sky.center_radius.w,0.0);
    let surface_veil = 1.0-smoothstep(shell*0.3,shell,veil_altitude);
    let day_veil = smoothstep(-0.12,0.20,sun_height)*surface_veil*0.985;
    opacity = max(opacity,day_veil);

    // Sparse cloud patches sit at a fixed physical radius. Shape samples are
    // quantized so silhouettes keep a quiet pixel-art rhythm from orbit.
    let cloud_hit = sphere_hit(camera,direction,sky.clouds.x);
    let cloud_distance = select(cloud_hit.x,cloud_hit.y,cloud_hit.x <= 0.0);
    if (cloud_distance > start && cloud_distance < end) {
        let cloud_position = camera+direction*cloud_distance;
        let radial = normalize(cloud_position);
        let q = floor(radial*480.0)/480.0;
        let shape = cloud_noise(q*12.0)*0.68+cloud_noise(q*29.0+vec3<f32>(7.0))*0.32;
        let cloud_alpha = smoothstep(sky.clouds.y,sky.clouds.y+0.16,shape)*sky.clouds.z;
        let day = max(dot(radial,sun),0.0);
        let light = sky.clouds.w+day*0.90;
        // Cloud radiance follows sunlight, retaining only a blue night floor.
        // A constant bright base made dark-side patches look self-illuminated.
        let cloud_color = mix(vec3<f32>(0.46,0.62,0.85),vec3<f32>(1.0,0.96,0.86),day)*light;
        scattered = scattered*(1.0-cloud_alpha)+cloud_color*cloud_alpha;
        opacity = opacity+(1.0-opacity)*cloud_alpha;
    }
    return vec4<f32>(max(scattered,vec3<f32>(0.0)),opacity);
}
