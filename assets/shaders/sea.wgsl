// The sea's surface: `pbd_core::sea::LocalSea` written out in WGSL.
//
// The Rust is the reference and this is its transcription: the same fixed
// table, uploaded from the same `SeaTable::gpu`, evaluated by the same steps
// in the same order. `pbd-app`'s parity test runs this file on a GPU adapter
// and compares it with the Rust. Change the Rust first.
//
// The one addition is the vertex filter: `spacing` is the distance between the
// caller's vertices, and a band shorter than `filter spacings` of them fades
// out instead of being drawn as its alias. The physics passes zero, which
// filters nothing.
#define_import_path pbd::sea

const SEA_DIRECTIONS: u32 = 12u;
const SEA_BANDS: u32 = 5u;

struct SeaView {
    // The twelve fixed wave directions, unit, w spare.
    directions: array<vec4<f32>, 12>,
    // The five wind-sea bands then the swell: wavelength m, k rad/m, omega
    // rad/s, the band's amplitude summed over directions m.
    bands: array<vec4<f32>, 6>,
    // Every component's phase now, rad, four to a lane; index band * 12 +
    // direction. The time is folded in on the CPU in f64.
    phase: array<vec4<f32>, 18>,
    // xyz where the wind sea runs (unit tangent, or zero), w the steepness cap.
    heading: vec4<f32>,
    // Tangent fade lo and hi, filter spacings, depth limit.
    limits: vec4<f32>,
}

struct SeaSurface {
    // Height over the sea radius, m.
    height: f32,
    // The surface's slope along the sphere: subtract it from the normal.
    gradient: vec3<f32>,
}

fn sea_smoothstep(lo: f32, hi: f32, x: f32) -> f32 {
    let t = clamp((x - lo) / (hi - lo), 0.0, 1.0);
    return t * t * (3.0 - 2.0 * t);
}

// The surface over `p` (a point on the sea sphere, body-local metres), with
// unit `normal` there, water `depth` m deep and the caller's vertex `spacing`
// in metres (zero: no filter).
fn sea_surface(view: SeaView, p: vec3<f32>, normal: vec3<f32>, depth: f32, spacing: f32) -> SeaSurface {
    let lo = view.limits.x;
    let hi = view.limits.y;
    var fade: array<f32, 12>;
    var along: array<f32, 12>;
    let heading = view.heading.xyz;
    let has_heading = dot(heading, heading) > 0.0;
    var wind_norm = 0.0;
    var fade_norm = 0.0;
    for (var i = 0u; i < SEA_DIRECTIONS; i = i + 1u) {
        let d = view.directions[i].xyz;
        let tangent = d - normal * dot(normal, d);
        let share = length(tangent);
        fade[i] = sea_smoothstep(lo, hi, share);
        along[i] = 1.0;
        if (has_heading && share > 1e-4) {
            let c = dot(tangent / share, heading);
            along[i] = select(0.0, c * c, c > 0.0);
        }
        wind_norm = wind_norm + fade[i] * along[i];
        fade_norm = fade_norm + fade[i];
    }
    // No direction lies with the wind: share it by the fade alone.
    let by_fade = wind_norm <= 1e-4;
    if (by_fade) { wind_norm = fade_norm; }
    wind_norm = max(wind_norm, 1e-4);
    let swell_norm = max(fade_norm, 1e-4);
    var out: SeaSurface;
    out.height = 0.0;
    out.gradient = vec3<f32>(0.0);
    var total = 0.0;
    for (var b = 0u; b <= SEA_BANDS; b = b + 1u) {
        let band = view.bands[b];
        let shoal = sea_smoothstep(0.0, band.x * 0.5, depth);
        var resolve = 1.0;
        if (spacing > 0.0) {
            let shortest = view.limits.z * spacing;
            resolve = sea_smoothstep(shortest, shortest * 1.6, band.x);
        }
        for (var i = 0u; i < SEA_DIRECTIONS; i = i + 1u) {
            var share = fade[i] / swell_norm;
            if (b < SEA_BANDS) {
                var weight = fade[i] * along[i];
                if (by_fade) { weight = fade[i]; }
                share = weight / wind_norm;
            }
            let a = min(band.w * sqrt(share) * shoal, view.heading.w / band.y);
            if (a < 1e-4) { continue; }
            let d = view.directions[i].xyz;
            let k = d * band.y;
            let index = b * SEA_DIRECTIONS + i;
            let phase = view.phase[index / 4u][index % 4u];
            let on_sea = dot(k, p) + phase;
            let drawn = a * resolve;
            out.height = out.height + drawn * sin(on_sea);
            out.gradient = out.gradient + (k - normal * dot(normal, k)) * (drawn * cos(on_sea));
            total = total + a;
        }
    }
    // Never higher than the water is deep.
    let cap = view.limits.w * max(depth, 0.0);
    if (total > cap) {
        let scale = cap / total;
        out.height = out.height * scale;
        out.gradient = out.gradient * scale;
    }
    return out;
}
