// The clouds: where they are, what shape, and how they are lit. ONE copy,
// imported by the after-scene clouds pass (`water.wgsl`), which draws them over
// the sky, the sea and the land alike against the scene's depth.
//
// WHERE is the simulated atmosphere's: the weather map (`planet_weather.rs`)
// gives each place its cover, how tall its cloud stands and how thick it is,
// and the wind aloft carries the cloud's detail. SHAPE is noise under that:
// flat bases, billowing tops, detail finer than a cell. LIGHT is the standard
// real-time model (Schneider, Horizon Zero Dawn, 2015; Hillaire, Frostbite,
// 2016): one extinction for the view and the light, a march toward the sun,
// three octaves of multiple scattering so a thin cloud is white through and a
// thick one darkens with depth, a two-lobe phase for the silver lining, and
// sky and ground light dimmed by the cloud above and below a sample. So a
// cloud's top is white and its base goes darker the more cloud stands over
// it: that darkness is an output of the density, not a knob.
//
// Everything is in the BODY-LOCAL frame: a point is metres from the planet's
// centre.
#define_import_path pbd::clouds

struct CloudLayer {
    // Radius of the cloud base, sky-light strength, extinction per metre in
    // fair weather, and the night floor.
    clouds: vec4<f32>,
    // The tallest a cloud stands above its base (m), ground-bounce strength,
    // drift seconds, and the forward phase lobe's g.
    slab: vec4<f32>,
    // The back lobe's g, the lobes' blend, extinction per metre at full
    // cover, and how brightly lightning lights the cloud.
    storm: vec4<f32>,
    // A lightning strike: xyz where it is, body-local metres; w how bright it
    // is right now, zero between strikes.
    flash: vec4<f32>,
    // The sun's strength on a cloud, and the multiple-scattering octaves'
    // falloffs of extinction, energy and phase.
    light: vec4<f32>,
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

// Three octaves: a mass has a shape at more than one scale.
fn cloud_shape(q: vec3<f32>) -> f32 {
    return cloud_noise(q)*0.55 + cloud_noise(q*2.7+vec3<f32>(7.0))*0.30
        + cloud_noise(q*6.1+vec3<f32>(19.0))*0.15;
}

// Where a point reads the weather maps: its own direction pushed about by a
// little noise, about a map texel's worth. The maps are the atmosphere's cells
// interpolated, and a cloud's edge is where the cover crosses a threshold, so
// read straight a cloud's outline traces the texel grid; warped, the same edge
// is ragged the way a cloud's is.
fn cloud_lookup(p: vec3<f32>) -> vec3<f32> {
    let radial = normalize(p);
    let q = radial*44.0;
    let warp = vec3<f32>(cloud_noise(q), cloud_noise(q+vec3<f32>(31.7)), cloud_noise(q+vec3<f32>(-17.3)))
        - vec3<f32>(0.5);
    return normalize(radial + warp*0.035);
}

// A weather map read with a cubic B-spline rather than the sampler's
// bilinear. Bilinear is continuous but its slope jumps at every texel line,
// and a cloud's edge is a steep threshold on the cover, so a bilinear read
// draws every cloud outlined in texel-sized stairs (measured: the steps from
// orbit were the map's 118 m texels exactly). The B-spline is smooth through
// its slope, so the same threshold draws a curve. Four bilinear taps make it
// (Sigg and Hadwiger, GPU Gems 2 ch. 20). The taps are placed on the face the
// direction points through, and one that falls past the face's edge is still
// a direction the sampler resolves on the neighbouring face, so the cube's
// seams need nothing. Shared by the clouds and the ground's cloud shadows so
// the two can never disagree about where a cloud is.
fn cloud_cubic_weights(f: f32) -> vec4<f32> {
    let n = vec4<f32>(1.0, 2.0, 3.0, 4.0) - vec4<f32>(f);
    let c = n*n*n;
    let x = c.x;
    let y = c.y - 4.0*c.x;
    let z = c.z - 4.0*c.y + 6.0*c.x;
    return vec4<f32>(x, y, z, 6.0 - x - y - z)/6.0;
}

fn cloud_map_smooth(map: texture_cube<f32>, map_sampler: sampler, direction: vec3<f32>) -> vec4<f32> {
    let size = f32(textureDimensions(map).x);
    let a = abs(direction);
    // The face's axis and its two in-face coordinates, each in -1..1. Which
    // way round the in-face axes run does not matter: the texel lattice on a
    // face is symmetric about its centre.
    var axis: vec3<f32>;
    var u_axis: vec3<f32>;
    var v_axis: vec3<f32>;
    var major: f32;
    if (a.x >= a.y && a.x >= a.z) {
        major = a.x; axis = vec3<f32>(sign(direction.x), 0.0, 0.0);
        u_axis = vec3<f32>(0.0, 0.0, 1.0); v_axis = vec3<f32>(0.0, 1.0, 0.0);
    } else if (a.y >= a.z) {
        major = a.y; axis = vec3<f32>(0.0, sign(direction.y), 0.0);
        u_axis = vec3<f32>(1.0, 0.0, 0.0); v_axis = vec3<f32>(0.0, 0.0, 1.0);
    } else {
        major = a.z; axis = vec3<f32>(0.0, 0.0, sign(direction.z));
        u_axis = vec3<f32>(1.0, 0.0, 0.0); v_axis = vec3<f32>(0.0, 1.0, 0.0);
    }
    let uv = vec2<f32>(dot(direction, u_axis), dot(direction, v_axis))/max(major, 1e-6);
    // Texel space: texel i's centre at i.
    let texel = (uv*0.5 + 0.5)*size - 0.5;
    let f = fract(texel);
    let base = texel - f;
    let wx = cloud_cubic_weights(f.x);
    let wy = cloud_cubic_weights(f.y);
    let gx = vec2<f32>(wx.x + wx.y, wx.z + wx.w);
    let gy = vec2<f32>(wy.x + wy.y, wy.z + wy.w);
    let ox = base.x + vec2<f32>(-0.5, 1.5) + vec2<f32>(wx.y, wx.w)/gx;
    let oy = base.y + vec2<f32>(-0.5, 1.5) + vec2<f32>(wy.y, wy.w)/gy;
    // Back from texel space to a direction through the face.
    let px = ((ox + 0.5)/size)*2.0 - 1.0;
    let py = ((oy + 0.5)/size)*2.0 - 1.0;
    let s00 = textureSampleLevel(map, map_sampler, axis + u_axis*px.x + v_axis*py.x, 0.0);
    let s10 = textureSampleLevel(map, map_sampler, axis + u_axis*px.y + v_axis*py.x, 0.0);
    let s01 = textureSampleLevel(map, map_sampler, axis + u_axis*px.x + v_axis*py.y, 0.0);
    let s11 = textureSampleLevel(map, map_sampler, axis + u_axis*px.y + v_axis*py.y, 0.0);
    let sx = gx.x/(gx.x + gx.y);
    let sy = gy.x/(gy.x + gy.y);
    return mix(mix(s11, s01, sx), mix(s10, s00, sx), sy);
}

// The weather at a point: cover, cloud top 0..1, rain mm/h, optical depth.
fn cloud_local(at: vec3<f32>, cloud_map: texture_cube<f32>, map_sampler: sampler) -> vec4<f32> {
    return cloud_map_smooth(cloud_map, map_sampler, at);
}

// The wind aloft at a point, m/s, body frame.
fn cloud_wind(at: vec3<f32>, wind_map: texture_cube<f32>, map_sampler: sampler) -> vec3<f32> {
    return textureSampleLevel(wind_map, map_sampler, at, 0.0).xyz;
}

// Density of cloud at a point, 0..1, given the weather there. The cover
// REMAPS the noise (Horizon Zero Dawn): where the atmosphere says clear there
// is nothing, where it says overcast the noise's every lump is cloud. The
// profile is flat at the base and billows to the column's own top, so a storm
// towers and a stratus deck lies flat. The noise drifts with the wind aloft
// by a two-phase flow map, so the detail moves the way the air does without
// ever stretching.
fn cloud_density(p: vec3<f32>, layer: CloudLayer, local: vec4<f32>, wind: vec3<f32>) -> f32 {
    let cover = clamp(local.x, 0.0, 1.0);
    if (cover <= 0.01) { return 0.0; }
    let inner = layer.clouds.x;
    let radius = length(p);
    let top = inner + layer.slab.x * clamp(local.y, 0.2, 1.0);
    if (radius < inner || radius > top) { return 0.0; }
    let height = (radius-inner)/max(top-inner, 1.0);
    let profile = smoothstep(0.0, 0.08, height) * (1.0-smoothstep(0.45, 1.0, height));
    let radial = p/radius;
    let per_metre = 22.0/inner;
    let period = 120.0;
    let seconds = layer.slab.z;
    let phase_a = fract(seconds/period);
    let phase_b = fract(seconds/period + 0.5);
    let drift = wind*(per_metre*period);
    let q = radial*22.0 + vec3<f32>(height*1.7);
    let shape = mix(cloud_shape(q - drift*phase_a), cloud_shape(q - drift*phase_b),
        abs(phase_a*2.0-1.0));
    let d = clamp((shape*profile - (1.0-cover))/max(cover, 0.08), 0.0, 1.0);
    return d*(2.0-d);
}

// Optical depth from a point along a direction, through the same field: six
// steps growing by half each time, out to the tallest cloud. The weather is
// read once at the start, which is the cell the light is crossing.
fn cloud_light_depth(p: vec3<f32>, toward: vec3<f32>, layer: CloudLayer, local: vec4<f32>,
        wind: vec3<f32>, extinction: f32, steps: u32) -> f32 {
    var length_sum = 0.0;
    var step = 1.0;
    for (var i = 0u; i < steps; i++) { length_sum += step; step *= 1.5; }
    var size = layer.slab.x/length_sum;
    var t = 0.0;
    var depth = 0.0;
    for (var i = 0u; i < steps; i++) {
        let at = p + toward*(t + size*0.5);
        depth += cloud_density(at, layer, local, wind)*size;
        t += size;
        size *= 1.5;
    }
    return depth*extinction;
}

// Henyey-Greenstein, scaled so an isotropic lobe is one.
fn cloud_hg(g: f32, cos_theta: f32) -> f32 {
    let gg = g*g;
    return (1.0-gg)/pow(max(1.0+gg-2.0*g*cos_theta, 1e-4), 1.5);
}

fn cloud_phase(layer: CloudLayer, g_scale: f32, cos_theta: f32) -> f32 {
    let forward = cloud_hg(layer.slab.w*g_scale, cos_theta);
    let back = cloud_hg(layer.storm.x*g_scale, cos_theta);
    return min(mix(forward, back, layer.storm.y), 8.0);
}

// The span of a ray from `camera` along `direction` that lies inside the
// cloud layer, clipped to [start, end]; x >= y when it misses.
fn cloud_span(camera: vec3<f32>, direction: vec3<f32>, start: f32, end: f32, layer: CloudLayer) -> vec2<f32> {
    let inner = layer.clouds.x;
    let outer = inner+layer.slab.x;
    let hit_in = cloud_sphere_hit(camera,direction,inner);
    let hit_out = cloud_sphere_hit(camera,direction,outer);
    if (hit_out.y <= 0.0) { return vec2<f32>(1.0,0.0); }
    var slab_near = max(hit_out.x,0.0);
    var slab_far = hit_out.y;
    if (hit_in.y > 0.0) {
        if (length(camera) < inner) { slab_near = max(slab_near,hit_in.y); }
        else if (hit_in.x > 0.0) { slab_far = min(slab_far,hit_in.x); }
    }
    slab_near = max(slab_near,start);
    slab_far = min(slab_far,end);
    // A grazing ray crosses more cloud than a vertical one, not unboundedly
    // more: at the horizon the chord through the layer is tens of kilometres.
    slab_far = min(slab_far,slab_near+layer.slab.x*40.0);
    return vec2<f32>(slab_near,slab_far);
}

// How brightly a lightning strike lights a point.
fn cloud_flash_at(p: vec3<f32>, layer: CloudLayer) -> f32 {
    if (layer.flash.w <= 0.0) { return 0.0; }
    let d = distance(p,layer.flash.xyz);
    return layer.flash.w*layer.storm.w*400.0*400.0/(d*d+400.0*400.0);
}

// March the layer between `near` and `far` along the ray. Returns the cloud's
// light, PREMULTIPLIED by its coverage, and the coverage in w: composite as
// `under * (1 - w) + rgb`.
fn cloud_march(camera: vec3<f32>, direction: vec3<f32>, near: f32, far: f32, layer: CloudLayer,
        sun: vec3<f32>, cloud_map: texture_cube<f32>, wind_map: texture_cube<f32>,
        map_sampler: sampler) -> vec4<f32> {
    if (far <= near) { return vec4<f32>(0.0); }
    let steps = 16.0;
    let step_size = (far-near)/steps;
    // Offset each ray's samples by its own fraction of a step, hashed off the
    // direction so it is stable frame to frame: banding becomes grain.
    let jitter = cloud_hash(direction*911.0);
    let cos_theta = dot(direction, sun);
    var transmittance = 1.0;
    var luminance = vec3<f32>(0.0);
    for (var i=0.0; i<steps; i+=1.0) {
        let p = camera+direction*(near+step_size*(i+jitter));
        let at = cloud_lookup(p);
        let local = cloud_local(at, cloud_map, map_sampler);
        if (local.x <= 0.01) { continue; }
        let wind = cloud_wind(at, wind_map, map_sampler);
        let density = cloud_density(p, layer, local, wind);
        if (density <= 0.0) { continue; }
        let radial = normalize(p);
        let extinction = mix(layer.clouds.z, layer.storm.z, clamp(local.x, 0.0, 1.0));
        // Sunlight: the sun's elevation over this sample (a cloud on the night
        // side is not lit by it), through the cloud toward it, scattered in
        // three octaves (Wrenninge 2013) so light reaches into a thick cloud.
        let elevation = dot(radial, sun);
        let day = smoothstep(-0.10, 0.15, elevation);
        var sunlit = 0.0;
        if (day > 0.0) {
            let tau_sun = cloud_light_depth(p, sun, layer, local, wind, extinction, 6u);
            var a = 1.0;
            var b = 1.0;
            var c = 1.0;
            for (var o = 0u; o < 3u; o++) {
                sunlit += b*cloud_phase(layer, c, cos_theta)*exp(-a*tau_sun);
                a *= layer.light.y;
                b *= layer.light.z;
                c *= layer.light.w;
            }
        }
        let low_sun = smoothstep(0.0, 0.35, elevation);
        let sun_colour = mix(vec3<f32>(1.0,0.62,0.36), vec3<f32>(1.0,0.97,0.92), low_sun);
        // Sky light from above through the cloud over this sample, and light
        // bounced off the ground from below: the cloud standing over a base is
        // what makes it dark, so a thick storm goes slate and a thin cumulus
        // stays grey.
        let tau_up = cloud_light_depth(p, radial, layer, local, wind, extinction, 3u);
        let sky = mix(vec3<f32>(0.03,0.04,0.07), vec3<f32>(0.55,0.68,0.92), day);
        let top = layer.clouds.x + layer.slab.x*clamp(local.y, 0.2, 1.0);
        let height = clamp((length(p)-layer.clouds.x)/max(top-layer.clouds.x, 1.0), 0.0, 1.0);
        let ground = vec3<f32>(0.30,0.32,0.28)*day*(1.0-height);
        let lit = sun_colour*sunlit*layer.light.x*day
            + sky*layer.clouds.y*exp(-tau_up)
            + ground*layer.slab.y
            + vec3<f32>(layer.clouds.w)
            + vec3<f32>(0.80,0.85,1.0)*cloud_flash_at(p,layer);
        let fade = exp(-density*step_size*extinction);
        luminance += lit*transmittance*(1.0-fade);
        transmittance *= fade;
        if (transmittance < 0.01) { break; }
    }
    return vec4<f32>(luminance, clamp(1.0-transmittance,0.0,1.0));
}
