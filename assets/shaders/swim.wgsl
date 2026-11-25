// A swimmer's body wave (`fish-behaviour` design 8.2, M37's Body wave tab):
// the spine of a fish or an eel bends in a wave that runs from the head to
// the tail, its cross-sections turning with the spine and its normals with
// them; a ray's wings undulate nose to tail. The fish's school mesh is laid
// rigid on the CPU; this bends it in the vertex stage (`swim_vertex.wgsl`).
//
// Units are body lengths: `side` and `up` are the creature's axes scaled to
// metres a body length, `u` runs 0 at the nose to 1 at the tail (beyond 1 a
// ray's whip), `d` is the vertex's distance off the spine, and the wave's
// amplitude and bend are body lengths. `pbd_core::fauna::swim::lateral` is
// `swim_lateral` on the CPU, for the tests and the drawn mouth.

#define_import_path pbd::swim

const SWIM_TAU: f32 = 6.28318530718;

// One body's row of the table: waves, exponent, head, recoil; recoil reach,
// kind (0 a swimmer, 1 a ray), span, wing exponent; wing waves, flap, whip.
struct SwimRow {
    a: vec4<f32>,
    b: vec4<f32>,
    c: vec4<f32>,
}

// The spine's lateral offset at `u`, and its slope d/du.
fn swim_lateral(u_in: f32, phase: f32, amp: f32, bend: f32, row: SwimRow) -> vec2<f32> {
    let waves = row.a.x;
    let expo = row.a.y;
    let head = row.a.z;
    let recoil = row.a.w;
    let reach = row.b.x;
    let u = clamp(u_in, 0.0, 1.0);
    let um = max(u, 1e-3);
    let e = head + (1.0 - head) * pow(um, expo);
    let de = (1.0 - head) * expo * pow(um, expo - 1.0);
    let a = SWIM_TAU * waves * u - phase;
    // Where the tail is now: the head yaws against it.
    let tail = sin(SWIM_TAU * waves - phase);
    let r = max(1.0 - u / reach, 0.0);
    // The C-shape of a turn, about the body's pivot.
    let q = (u - 0.35) / 0.65;
    let f = amp * (e * sin(a) - recoil * r * r * tail) + bend * q * q;
    let df = amp * (de * sin(a) + e * SWIM_TAU * waves * cos(a) + 2.0 * recoil * r / reach * tail)
        + bend * 2.0 * q / 0.65;
    return vec2<f32>(f, df);
}

struct Swum {
    position: vec3<f32>,
    normal: vec3<f32>,
}

// The vertex at `p` with normal `n` (mesh space, laid rigid) bent by the
// wave. `side` and `up` are the creature's axes, metres a body length.
fn swim_body(
    p: vec3<f32>,
    n: vec3<f32>,
    side: vec3<f32>,
    up: vec3<f32>,
    u: f32,
    d: f32,
    phase: f32,
    amp: f32,
    bend: f32,
    row: SwimRow,
) -> Swum {
    var out: Swum;
    let metres = length(side);
    let side_n = side / max(metres, 1e-6);
    let up_n = up / max(length(up), 1e-6);
    let fwd_n = cross(up_n, side_n);
    let nf = dot(n, fwd_n);
    let nu = dot(n, up_n);
    let ns = dot(n, side_n);
    if (row.b.y < 0.5) {
        // A swimmer: the spine bends sideways, each cross-section turning
        // with its slope, so the body keeps its thickness through a bend.
        let fl = swim_lateral(u, phase, amp, bend, row);
        // dz/dx: x runs nose (+) to tail (-).
        let slope = -fl.y;
        let c = inverseSqrt(1.0 + slope * slope);
        let sn = slope * c;
        out.position = p + side_n * metres * (fl.x + d * (c - 1.0)) - fwd_n * metres * (d * sn);
        let n2 = n + fwd_n * (nf * (c - 1.0) - ns * sn) + side_n * (nf * sn + ns * (c - 1.0));
        out.normal = normalize(n2);
    } else {
        // A ray: the wings undulate in a wave running nose to tail along the
        // chord (flap 1 lifts them together), growing out along the span;
        // the wing inside a turn beats less; the whip swings sideways.
        let span = row.b.z;
        let wing_expo = row.b.w;
        let wing_waves = row.c.x;
        let flap = row.c.y;
        let whip = row.c.z;
        let sp = clamp(abs(d) / span, 0.0, 1.0);
        let sg = select(1.0, -1.0, d < 0.0);
        let k = SWIM_TAU * wing_waves * (1.0 - flap);
        let a = k * clamp(u, 0.0, 1.0) - phase;
        let inner = 1.0 - clamp(bend * sg * 6.0, -0.6, 0.6);
        let reach = pow(max(sp, 1e-3), wing_expo);
        let g = amp * inner * reach * sin(a);
        let gz = amp * inner * wing_expo * pow(max(sp, 1e-3), wing_expo - 1.0) / span * sg * sin(a);
        let gx = -amp * inner * reach * cos(a) * k;
        let w = max(u - 1.0, 0.0);
        let lateral = whip * w * sin(SWIM_TAU * 1.2 * u - 0.7 * phase) + bend * u * u * 0.5;
        out.position = p + up_n * metres * g + side_n * metres * lateral;
        out.normal = normalize(fwd_n * (nf - gx * nu) + up_n * nu + side_n * (ns - gz * nu));
    }
    return out;
}
