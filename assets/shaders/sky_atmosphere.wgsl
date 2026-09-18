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
    // Slab thickness (m), the cover the field says is overhead, drift seconds,
    // and how dark a cloud's underside goes.
    cloud_slab: vec4<f32>,
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

// Density of the cloud field at a point in the slab, in [0, 1]. Three octaves,
// so a mass has a shape at two scales rather than the two the flat shell used,
// and a vertical profile that tapers to nothing at both faces: a slab with hard
// edges reads as a ceiling, not as cloud.
fn cloud_density(p: vec3<f32>, inner: f32, outer: f32, threshold: f32, drift: f32) -> f32 {
    let radius = length(p);
    if (radius < inner || radius > outer) { return 0.0; }
    let radial = p/radius;
    // The layer drifts about the spin axis, which is what makes weather move
    // across the sky rather than hang in place.
    let flow = vec3<f32>(drift*0.004,0.0,drift*0.0026);
    // 22 rather than 12: at a twelfth of a radian a "cloud" is four hundred
    // metres of gradient, which reads as weather-coloured haze. Smaller and
    // more numerous is what makes a sky somebody can count clouds in.
    let q = radial*22.0+flow;
    let shape = cloud_noise(q)*0.62+cloud_noise(q*2.7+vec3<f32>(7.0))*0.26
        +cloud_noise(q*6.1+vec3<f32>(19.0))*0.12;

    // Tapered at top and bottom: full weight in the middle third.
    let height = (radius-inner)/max(outer-inner,1.0);
    let profile = smoothstep(0.0,0.30,height)*(1.0-smoothstep(0.62,1.0,height));
    let d = max(shape*profile-threshold,0.0)/max(1.0-threshold,1e-3);
    // Rise FAST above the threshold rather than ramping linearly: a linear ramp
    // off smooth noise is haze, and a cloud needs a boundary somebody can point
    // at with solid cloud just inside it. Squaring the shape BEFORE the
    // threshold was the first attempt and did the opposite - it cut the density
    // faster than a lower threshold could raise it, so a full overcast came out
    // as thin as a clear sky and the storm could never close over.
    return d*(2.0-d);
}

// How much sunlight reaches a point inside the slab: four samples toward the
// sun through the same field, Beer's law over them. Cheap on purpose - what it
// has to deliver is a dark underside and a bright top, not a shadow anyone
// could trace.
fn cloud_shadow(p: vec3<f32>, sun: vec3<f32>, inner: f32, outer: f32, threshold: f32, drift: f32) -> f32 {
    let step_size = (outer-inner)*0.32;
    var depth = 0.0;
    for (var i=0u; i<4u; i++) {
        depth += cloud_density(p+sun*(step_size*(f32(i)+0.5)),inner,outer,threshold,drift);
    }
    return exp(-depth*step_size*0.05);
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

    // The clouds are a SLAB, not a shell. A single ray-sphere hit gives one
    // sample at one depth, which is a stencil painted on a sphere: nothing in
    // it to be lit from one side and no silhouette from below. Marching a shell
    // of real thickness is what makes a mass. A grazing ray crosses more of it
    // than a vertical one by construction, which is why the horizon builds up
    // while the zenith stays open, and no extra term is needed for that.
    let inner = sky.clouds.x;
    let outer = inner+sky.cloud_slab.x;
    let hit_in = sphere_hit(camera,direction,inner);
    let hit_out = sphere_hit(camera,direction,outer);
    if (hit_out.y > 0.0) {
        // The span of this ray that is inside the slab. Standing under it the
        // near root is behind us, so the march starts at the camera; from orbit
        // it starts at the outer shell.
        var slab_near = max(hit_out.x,0.0);
        var slab_far = hit_out.y;
        // Below the inner shell the slab begins where the ray leaves it; above
        // it, the inner sphere cuts the span short.
        if (hit_in.y > 0.0) {
            if (length(camera) < inner) { slab_near = max(slab_near,hit_in.y); }
            else if (hit_in.x > 0.0) { slab_far = min(slab_far,hit_in.x); }
        }
        slab_near = max(slab_near,start);
        slab_far = min(slab_far,end);
        // A grazing ray crossing MORE slab than a vertical one is the effect;
        // crossing unboundedly more is a wall. At the horizon the chord through
        // a 260 m shell is tens of kilometres, so without a cap every ray near
        // the horizon saturates and the whole sky closes over - which is
        // exactly what the first cut did. Eight thicknesses keeps the horizon
        // visibly denser than the zenith and stops there.
        slab_far = min(slab_far,slab_near+sky.cloud_slab.x*80.0);
        if (slab_far > slab_near) {
            // The field's cover over the player opens and closes the sky: at
            // nothing the threshold is high and only the densest noise shows,
            // at full cover it is low and the slab closes over.
            let cover = clamp(sky.cloud_slab.y,0.0,1.0);
            let threshold = mix(sky.clouds.y,sky.clouds.y*0.28,cover);
            let drift = sky.cloud_slab.z;
            let span = slab_far-slab_near;
            let steps = 12.0;
            let step_size = span/steps;
            // Offset every ray's samples by its own fraction of a step. Twelve
            // samples on a fixed grid put their step boundaries at the same
            // depth for neighbouring pixels, which draws the march as bands
            // across a cloud; jittered, the same error is noise nobody reads as
            // a pattern. The offset is hashed off the direction, so it is
            // stable frame to frame rather than a shimmer.
            let jitter = cloud_hash(direction*911.0);
            var transmittance = 1.0;
            var luminance = vec3<f32>(0.0);
            for (var i=0.0; i<steps; i+=1.0) {
                let p = camera+direction*(slab_near+step_size*(i+jitter));
                let density = cloud_density(p,inner,outer,threshold,drift);
                if (density <= 0.0) { continue; }
                // Light this sample by a short march toward the sun through the
                // same field. That difference between a lit top and a shadowed
                // underside IS the thickness; a single sample cannot have one.
                let shadow = cloud_shadow(p,sun,inner,outer,threshold,drift);
                let day = max(dot(normalize(p),sun),0.0);
                let lit = mix(sky.cloud_slab.w,1.0,shadow)*(sky.clouds.w+day*0.90);
                let tint = mix(vec3<f32>(0.46,0.62,0.85),vec3<f32>(1.0,0.96,0.86),day);
                // Tuned so a full-density VERTICAL crossing of the slab comes
                // out mostly opaque and a thin one stays translucent: 260 m at
                // this coefficient is an optical depth of about 1.6.
                // `clouds.z` was the flat shell's peak OPACITY, and left there
                // it capped the slab at 0.52 however thick the cloud got - a
                // full overcast that could never close. The slab computes its
                // own opacity from transmittance, so the knob is an extinction
                // coefficient now: how much a metre of full-density cloud
                // absorbs. 260 m at the shipped value is an optical depth of
                // about 1.6, so a vertical crossing of solid cloud is mostly
                // opaque and a thin one stays translucent.
                let absorbed = density*step_size*sky.clouds.z*0.012;
                let fade = exp(-absorbed);
                luminance += tint*lit*transmittance*(1.0-fade);
                transmittance *= fade;
                if (transmittance < 0.01) { break; }
            }
            let cloud_alpha = clamp(1.0-transmittance,0.0,1.0);
            if (cloud_alpha > 0.0) {
                scattered = scattered*(1.0-cloud_alpha)+luminance/max(1.0-transmittance,1e-4)*cloud_alpha;
                opacity = opacity+(1.0-opacity)*cloud_alpha;
            }
        }
    }
    return vec4<f32>(max(scattered,vec3<f32>(0.0)),opacity);
}
