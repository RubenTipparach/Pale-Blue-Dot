// Integrated Bevy 0.18 material. Adapted from the validated atmosphere.wgsl
// and Tenebris's 8-view/4-sun single-scatter shader (see shader-port.md).
// The integrated material uses 16 view steps and a soft planetary penumbra
// to remove visible shadow bands found in the native orbital captures.
// Unlike the standalone asset, this consumes Bevy view/mesh bindings.
//
// Two draws use this file (`sun-in-space`): the dome (`SkyMaterial`), and,
// with `SUN_DISC` defined, the sun seen from outside the shell
// (`SunDiscMaterial`), a quad of its own because from outside the dome has
// fragments only where its shell is. One file, so the air, its extinction and
// the sun's disc are written once.
#import bevy_pbr::mesh_view_bindings::view
#import pbd::clouds::cloud_sphere_hit
#ifndef SUN_DISC
#import bevy_pbr::forward_io::VertexOutput
#endif

struct SkyParameters {
    center_radius: vec4<f32>,
    atmosphere: vec4<f32>,
    sun: vec4<f32>,
    scatter: vec4<f32>,
}
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> sky: SkyParameters;

fn air_density(p: vec3<f32>) -> f32 {
    let shell = max(sky.atmosphere.x - sky.center_radius.w, 1.0);
    let altitude = max(length(p)-sky.center_radius.w, 0.0) / shell;
    // The geometry encloses mountain summits; density, rather than a hard
    // sphere edge, makes the visible limb thin. Taper the last outer band.
    let outer_taper = 1.0-smoothstep(0.72,1.0,altitude);
    return exp(-altitude/max(sky.atmosphere.y, 0.01))*outer_taper;
}

fn sun_depth(p: vec3<f32>, direction: vec3<f32>) -> f32 {
    let ray = cloud_sphere_hit(p, direction, sky.atmosphere.x);
    let distance = max(ray.y, 0.0);
    let step_size = distance * 0.25;
    let shell = max(sky.atmosphere.x-sky.center_radius.w, 1.0);
    var depth = 0.0;
    for (var i=0u; i<4u; i++) {
        depth += air_density(p+direction*(step_size*(f32(i)+0.5)))*step_size/shell;
    }
    return depth;
}

// The sun disc: 0.6 degrees of angular radius, cosines of its edge and of a
// softened rim inside it, and its colour before the radiance scale.
const SUN_COS_OUTER: f32 = 0.999945;
const SUN_COS_INNER: f32 = 0.999975;
const SUN_COLOUR: vec3<f32> = vec3<f32>(1.0,0.93,0.80);

// Over the ground from above the shell (`high-haze`): the Rayleigh phase's
// 1 / (4 pi), the opacity from which the gain rises back to the sky's own,
// and the climb past the shell's top over which the column fades in (the
// owner: "transition to it over 200 m").
const GROUND_RAYLEIGH_GAIN: f32 = 0.0795775;
const GROUND_GAIN_FROM: f32 = 0.3;
const GROUND_HANDOVER_M: f32 = 200.0;
// Farther out the tint over the ground thins again (the owner, from orbit:
// "it should read more as blueish green still from a distance"): from the
// full tint FAR_FROM_M past the shell to ORBIT_GROUND_GAIN of it by FAR_TO_M.
// The limb keeps the sky's own light.
const ORBIT_GROUND_GAIN: f32 = 0.25;
const FAR_FROM_M: f32 = 1000.0;
const FAR_TO_M: f32 = 6000.0;
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

// The sun's own light along a view direction at `cosine` from it: a disc
// with a darkened limb and a glow that reaches a few radii, hidden by the
// planet's shadow over the eye at `camera`. There was a sunset here once and
// no sun; the scattering brightened toward a direction nothing was drawn
// along. Tenebris draws its disc as an angular-size-correct billboard inside
// the far plane. The radius is a few times the real sun's 0.27 degrees, for
// legibility. The light, and in `a` the disc's cover (how much of what is
// behind it the disc replaces).
fn sun_disc_light(cosine: f32, camera: vec3<f32>, sun: vec3<f32>) -> vec4<f32> {
    let disc = smoothstep(SUN_COS_OUTER,SUN_COS_INNER,cosine);
    let limb = mix(0.62,1.0,smoothstep(SUN_COS_OUTER,1.0,cosine));
    let glow = pow(max(cosine,0.0),1800.0)*0.55;
    let shadowed = sun_visibility(camera,sun);
    return vec4<f32>(SUN_COLOUR*sky.sun.w*(disc*limb+glow)*shadowed,disc*shadowed);
}

#ifdef SUN_DISC
// The sun from outside the shell (`sun-in-space`). From inside, the dome's
// back faces surround the eye and it draws the sun; from outside, the dome
// has a fragment only where a ray meets the shell, and a sun above the
// shell's rim had none: it was cut flat along the rim, or gone.
//
// A quad across the sun's direction, SUN_REACH_TAN each way (6.9 degrees:
// the glow is 2e-6 of the sun there), SUN_DISTANCE_M out: in front of the
// stars (180 km), behind the moon and the planet, which the depth test hides
// it behind. Built here from the mesh's four corners; its transform is not
// read (the entity stands at the planet's centre with the dome only so that
// the transparent sort draws it just after the dome).
const SUN_REACH_TAN: f32 = 0.1206;
const SUN_DISTANCE_M: f32 = 150000.0;

struct SunVertex {
    @location(0) position: vec3<f32>,
}

struct SunVaryings {
    @builtin(position) clip_position: vec4<f32>,
    // The ray toward this corner in the world's axes, unnormalised and kept
    // relative to the eye, so f32 keeps the disc's edge.
    @location(0) ray: vec3<f32>,
}

@vertex
fn vertex(v: SunVertex) -> SunVaryings {
    let sun = normalize(sky.sun.xyz);
    let helper = select(vec3<f32>(0.0,1.0,0.0),vec3<f32>(1.0,0.0,0.0),abs(sun.y) > 0.9);
    let right = normalize(cross(sun,helper));
    let up = cross(right,sun);
    let ray = sun+(right*v.position.x+up*v.position.y)*SUN_REACH_TAN;
    var out: SunVaryings;
    out.ray = ray;
    let in_view = (view.view_from_world*vec4<f32>(ray*SUN_DISTANCE_M,0.0)).xyz;
    out.clip_position = view.clip_from_view*vec4<f32>(in_view,1.0);
    // From inside the shell the dome draws the sun: every corner at one
    // point, no area, no fragments.
    let camera = view.world_position - sky.center_radius.xyz;
    if (length(camera) < sky.atmosphere.x) {
        out.clip_position = vec4<f32>(0.0,0.0,0.0,1.0);
    }
    return out;
}

// The sun's light added over what the dome left (`AlphaMode::Add`), through
// the air on its ray: the dome's own extinction, per channel, so a sun seen
// through the limb's air is dimmer and redder. It comes in over the
// hand-over past the shell's top, so leaving the shell changes nothing in a
// frame (the dome's sun, inside, is not attenuated).
@fragment
fn fragment(in: SunVaryings) -> @location(0) vec4<f32> {
    let camera = view.world_position - sky.center_radius.xyz;
    let direction = normalize(in.ray);
    let sun = normalize(sky.sun.xyz);
    // Behind the planet the terrain's depth hides it; the sea sphere makes
    // sure.
    let ground = cloud_sphere_hit(camera,direction,sky.center_radius.w);
    if (ground.x > 0.0) { discard; }
    let light = sun_disc_light(clamp(dot(direction,sun),-1.0,1.0),camera,sun);
    let shell = max(sky.atmosphere.x-sky.center_radius.w, 1.0);
    let air_hit = cloud_sphere_hit(camera,direction,sky.atmosphere.x);
    let start = max(air_hit.x,0.0);
    let end = air_hit.y;
    var depth = 0.0;
    if (end > start) {
        let step_size = (end-start)/16.0;
        for (var i=0u; i<16u; i++) {
            depth += air_density(camera+direction*(start+step_size*(f32(i)+0.5)))*step_size/shell;
        }
    }
    let beta = sky.scatter.xyz*sky.atmosphere.z+vec3<f32>(sky.atmosphere.w);
    let altitude = max(length(camera)-sky.center_radius.w,0.0);
    let handover = smoothstep(0.0,GROUND_HANDOVER_M,max(altitude-shell,0.0));
    let through = mix(vec3<f32>(1.0),exp(-beta*depth),handover);
    return vec4<f32>(light.rgb*through,0.0);
}
#else
@fragment
fn fragment(in: VertexOutput, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    let camera = view.world_position - sky.center_radius.xyz;
    let inside = length(camera) < sky.atmosphere.x;
    if (inside == front) { discard; }
    let direction = normalize(in.world_position.xyz - view.world_position);
    let sun = normalize(sky.sun.xyz);
    let shell = max(sky.atmosphere.x-sky.center_radius.w, 1.0);
    let air_hit = cloud_sphere_hit(camera,direction,sky.atmosphere.x);
    let start = max(air_hit.x,0.0);
    var end = air_hit.y;
    let ground = cloud_sphere_hit(camera,direction,sky.center_radius.w);
    if (ground.x > 0.0) { end = min(end, ground.x); }
    if (end <= start) { discard; }

    let step_size = (end-start)/16.0;
    // One twilight for the sky and the ground (`dusk-sky`). On a planet this
    // small, air a few hundred metres up stays in sunlight until the sun is
    // 20-34 degrees under (acos(R / (R + h))), so the dome, lit sample by
    // sample through the planet's shadow, glowed for hours over a ground the
    // SH (`pbd_core::sky_light`) already lit as night, and the shadow's edge
    // crossed the sky as a hard line. Near the ground each sample's sunlight
    // is the camera's twilight instead, over the ground's own range (gone at
    // sine -0.12, 6.9 degrees under); from high up, the band the daylight
    // veil uses, the dome keeps its shadow and the terminator from orbit.
    let up = normalize(camera);
    let sun_height = dot(up,sun);
    let altitude = max(length(camera)-sky.center_radius.w,0.0);
    let near_ground = 1.0-smoothstep(shell*0.3,shell,altitude);
    let twilight = smoothstep(-0.12,0.0,sun_height);
    let beta_r = sky.scatter.xyz * sky.atmosphere.z;
    let beta_m = sky.atmosphere.w;
    var view_depth = 0.0;
    var integrated = vec3<f32>(0.0);
    // Stable midpoint integration; no frame noise swimming on pixel terrain.
    for (var i=0u; i<16u; i++) {
        let p = camera+direction*(start+step_size*(f32(i)+0.5));
        let segment = air_density(p)*step_size/shell;
        view_depth += segment;
        let visible_sun = mix(sun_visibility(p,sun),twilight,near_ground);
        if (visible_sun <= 0.0) { continue; }
        let light_depth = sun_depth(p,sun);
        let transmission = exp(-(beta_r+vec3<f32>(beta_m))*(view_depth+light_depth));
        integrated += transmission*segment*visible_sun;
    }
    let cosine = clamp(dot(direction,sun),-1.0,1.0);
    let rayleigh_phase = 0.75*(1.0+cosine*cosine);
    let g = clamp(sky.scatter.w,-0.95,0.95);
    let mie_phase = (1.0-g*g)/(12.5663706*pow(max(1.0+g*g-2.0*g*cosine,0.001),1.5));
    // The column over the ground, seen from above the shell (`high-haze`).
    // From inside, the back face over the ground lies beyond it and fails the
    // depth test, so the ground's own haze (`finish`) is all the air there;
    // from outside, the front face lays this column over everything. Laid on
    // whole, it switched on in the one frame the eye crossed the shell (960 m
    // up) as a blue-white veil.
    let over_ground = !inside && ground.x > 0.0;
    let average_beta = dot(beta_r,vec3<f32>(1.0/3.0))+beta_m;
    let column_opacity = clamp(1.0-exp(-view_depth*average_beta),0.0,0.94);
    // The Rayleigh phase here is not normalised by 4 pi (the Mie one is): the
    // sky's brightness was tuned with it so. Over the ground the haze tends
    // to the sky's colour instead, its normalised phase, and meets the sky's
    // own light toward the limb, where the path is long and the opacity nears
    // its cap.
    // The hand-over: from the shell's top the column fades in over
    // GROUND_HANDOVER_M, and with it the haze's colour goes from the sky's to
    // the dome's own blue tint, the look the owner kept.
    let past_shell = max(altitude-shell,0.0);
    let handover = smoothstep(0.0,GROUND_HANDOVER_M,past_shell);
    let limb_light = smoothstep(GROUND_GAIN_FROM,0.94,column_opacity);
    let tint = mix(1.0,ORBIT_GROUND_GAIN,smoothstep(FAR_FROM_M,FAR_TO_M,past_shell));
    let rayleigh_gain = select(1.0,
        mix(mix(GROUND_RAYLEIGH_GAIN,1.0,limb_light),mix(tint,1.0,limb_light),handover),
        over_ground);
    var scattered = integrated*(beta_r*rayleigh_phase*rayleigh_gain+vec3<f32>(beta_m*mie_phase))*sky.sun.w;

    // The dusk tint, and near the ground over the ground's twilight band
    // (sine 0 to -0.2, to 11.5 degrees under) rather than to 16, where the
    // ground had long been night. The horizon's glow is on the sun's side: a
    // ring of it all round the horizon read as an orange arch.
    let dusk_end = mix(-0.28,-0.2,near_ground);
    let dusk = (1.0-smoothstep(0.0,0.2,sun_height))*smoothstep(dusk_end,-0.06*(1.0-near_ground),sun_height);
    let horizon = pow(clamp(1.0-abs(dot(direction,up)),0.0,1.0),5.0);
    let low_camera = 1.0-smoothstep(80.0,700.0,altitude);
    let across = direction-up*dot(direction,up);
    let toward = sun-up*sun_height;
    let sun_side = smoothstep(-0.3,1.0,dot(across,toward)*inverseSqrt(max(dot(across,across)*dot(toward,toward),1e-8)));
    scattered = mix(scattered,scattered*vec3<f32>(2.1,0.65,0.24),dusk*0.65);
    scattered += vec3<f32>(0.8,0.24,0.055)*dusk*horizon*low_camera*0.18*mix(1.0,sun_side,near_ground);
    var opacity = column_opacity;
    // Art-directed daylight veil keeps bright stars out of the surface sky.
    // It fades out with altitude, revealing the star field continuously.
    let veil_altitude = max(length(camera)-sky.center_radius.w,0.0);
    let surface_veil = 1.0-smoothstep(shell*0.3,shell,veil_altitude);
    let day_veil = smoothstep(-0.12,0.20,sun_height)*surface_veil*0.985;
    opacity = max(opacity,day_veil);

    // The clouds are not drawn here: the after-scene clouds pass
    // (`water.wgsl`, `pbd::clouds`) draws them over everything against the
    // scene's depth, so a cloud in front of a hill is in front of it and the
    // sun disc below is covered wherever a cloud crosses it.
    // The sun itself (`sun_disc_light`), from inside the shell, where the
    // dome surrounds the eye and is the far surface every direction meets.
    // From outside, the dome covers only the shell's disc, and the sun is the
    // `SUN_DISC` quad instead (`sun-in-space`); adding it here too would count
    // the glow twice along the rim. The clouds pass draws over both, so a
    // cloud hides it.
    if (inside) {
        let sun_light = sun_disc_light(cosine,camera,sun);
        scattered += sun_light.rgb;
        opacity = max(opacity,sun_light.a);
    }
    // Over the ground the column comes in with the hand-over, from nothing
    // at the shell's top to the whole of it 200 m on: nothing switches on in
    // a frame.
    if (over_ground) {
        scattered *= handover;
        opacity *= handover;
    }
    return vec4<f32>(max(scattered,vec3<f32>(0.0)),opacity);
}
#endif
