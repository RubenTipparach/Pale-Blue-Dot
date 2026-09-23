// The cloud slab: its density field, its self-shadow and its march. ONE copy,
// imported by the sky shell (`sky_atmosphere.wgsl`) and by the sea
// (`water.wgsl`), so a cloud seen against the sky and the same cloud seen
// against the ocean are the same cloud. The sea is drawn after the sky and the
// shell writes no depth, so before this module the sheet painted straight over
// every cloud in front of it and the ocean from orbit was cloudless.
//
// Everything is in the BODY-LOCAL frame: a point is metres from the planet's
// centre.
#define_import_path pbd::clouds

struct CloudLayer {
    // Radius of the slab's base, the clear-sky density threshold, extinction
    // per metre in fair weather, and the night floor.
    clouds: vec4<f32>,
    // Slab thickness (m), the cover over the player, drift seconds, and how
    // dark a cloud's shadowed underside goes in fair weather.
    slab: vec4<f32>,
    // The threshold at full cover, the underside darkness there, the
    // extinction per metre there, and how brightly lightning lights it.
    storm: vec4<f32>,
    // A lightning strike: xyz where it is, body-local metres; w how bright it
    // is right now, zero between strikes.
    flash: vec4<f32>,
}

fn cloud_sphere_hit(ro: vec3<f32>, rd: vec3<f32>, radius: f32) -> vec2<f32> {
    let b = dot(ro, rd);
    let d = b*b - dot(ro,ro) + radius*radius;
    if (d < 0.0) { return vec2<f32>(-1.0); }
    let root = sqrt(max(d, 0.0));
    return vec2<f32>(-b-root, -b+root);
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

// The span of a ray from `camera` along `direction` that lies inside the slab,
// clipped to [start, end]; x >= y when it misses.
fn cloud_span(camera: vec3<f32>, direction: vec3<f32>, start: f32, end: f32, layer: CloudLayer) -> vec2<f32> {
    let inner = layer.clouds.x;
    let outer = inner+layer.slab.x;
    let hit_in = cloud_sphere_hit(camera,direction,inner);
    let hit_out = cloud_sphere_hit(camera,direction,outer);
    if (hit_out.y <= 0.0) { return vec2<f32>(1.0,0.0); }
    // Standing under the slab the near root is behind us, so the march starts
    // at the camera; from orbit it starts at the outer shell.
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
    // the horizon saturates and the whole sky closes over - which is exactly
    // what the first cut did.
    slab_far = min(slab_far,slab_near+layer.slab.x*80.0);
    return vec2<f32>(slab_near,slab_far);
}

// How brightly a lightning strike lights a point: the flash's brightness over
// a soft inverse square of the distance, in metres, to where it struck.
fn cloud_flash_at(p: vec3<f32>, layer: CloudLayer) -> f32 {
    if (layer.flash.w <= 0.0) { return 0.0; }
    let d = distance(p,layer.flash.xyz);
    return layer.flash.w*layer.storm.w*400.0*400.0/(d*d+400.0*400.0);
}

// March the slab between `near` and `far` along the ray. Returns the cloud's
// light, PREMULTIPLIED by its coverage, and the coverage in w: composite as
// `under * (1 - w) + rgb`.
fn cloud_march(camera: vec3<f32>, direction: vec3<f32>, near: f32, far: f32, layer: CloudLayer, sun: vec3<f32>) -> vec4<f32> {
    if (far <= near) { return vec4<f32>(0.0); }
    let inner = layer.clouds.x;
    let outer = inner+layer.slab.x;
    // The field's cover over the player opens and closes the sky: at nothing
    // the threshold is high and only the densest noise shows, at full cover it
    // is low and the slab closes over.
    let cover = clamp(layer.slab.y,0.0,1.0);
    let threshold = mix(layer.clouds.y,layer.storm.x,cover);
    // A storm's base goes slate: the self-shadowed underside darkens with the
    // cover as well as thickening with it.
    let base_dark = mix(layer.slab.w,layer.storm.y,cover);
    // And a storm is DENSER cloud, not only more of it: at the fair weather
    // extinction a raining sky let a fifth of the starlight through its thin
    // middle third.
    let extinction = mix(layer.clouds.z,layer.storm.z,cover);
    let drift = layer.slab.z;
    let steps = 12.0;
    let step_size = (far-near)/steps;
    // Offset every ray's samples by its own fraction of a step. Twelve samples
    // on a fixed grid put their step boundaries at the same depth for
    // neighbouring pixels, which draws the march as bands across a cloud;
    // jittered, the same error is noise nobody reads as a pattern. Hashed off
    // the direction, so it is stable frame to frame rather than a shimmer.
    let jitter = cloud_hash(direction*911.0);
    var transmittance = 1.0;
    var luminance = vec3<f32>(0.0);
    for (var i=0.0; i<steps; i+=1.0) {
        let p = camera+direction*(near+step_size*(i+jitter));
        let density = cloud_density(p,inner,outer,threshold,drift);
        if (density <= 0.0) { continue; }
        // Light this sample by a short march toward the sun through the same
        // field. That difference between a lit top and a shadowed underside IS
        // the thickness; a single sample cannot have one.
        let shadow = cloud_shadow(p,sun,inner,outer,threshold,drift);
        let day = max(dot(normalize(p),sun),0.0);
        let lit = mix(base_dark,1.0,shadow)*(layer.clouds.w+day*0.90);
        let tint = mix(vec3<f32>(0.46,0.62,0.85),vec3<f32>(1.0,0.96,0.86),day);
        // Lightning lights the cloud from inside, cool white, whatever the sun
        // is doing: a strike at night is the whole point.
        let flash = vec3<f32>(0.80,0.85,1.0)*cloud_flash_at(p,layer);
        let absorbed = density*step_size*extinction;
        let fade = exp(-absorbed);
        luminance += (tint*lit+flash)*transmittance*(1.0-fade);
        transmittance *= fade;
        if (transmittance < 0.01) { break; }
    }
    return vec4<f32>(luminance,clamp(1.0-transmittance,0.0,1.0));
}
