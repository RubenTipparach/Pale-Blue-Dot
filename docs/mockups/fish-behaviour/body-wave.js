// Fish behaviour (M37), the Body wave tab: the game's rigid body and tail sway (TODAY) beside a vertex-shader
// body wave (PROPOSED), on the game's own perch, eel and ray meshes.
// TODAY is crates/pbd-app/src/fish/bodies.rs `sway`, constant for constant (Sway::Fish, Sway::Eel, Sway::Ray), applied
// on the CPU to the model before it is placed, as `emit` does; normals are flat, from the swayed triangles, as
// `compute_flat_normals` makes them.
// PROPOSED is the vertex shader below. It is written in GLSL that ports line for line to WGSL (no three.js helpers
// in the maths): the per-vertex inputs are `aU` (place along the body, 0 nose to 1 tail), `aSide` (the vertex's
// distance off the spine, model units) and `aSwim` (per fish: wave phase, amplitude, turn bend), exactly what the
// game's school mesh would carry per vertex. The species profile is uniforms here; in the game it is a fourth
// attribute or a small table indexed by species.
// Registered for fish-behaviour.js as window.__fishWave(lib); that file owns the tabs, the shared controls and the modals.
"use strict";
window.__fishWave = function(lib){
const { environment, loadTextures, getAtlas, state, MODELS } = lib;
const $ = id => document.getElementById(id);
const clamp = (x, a, b) => Math.min(b, Math.max(a, x));
const lerp = (a, b, t) => a + (b - a) * t;
const TAU = Math.PI * 2, DEG = Math.PI / 180;
const V = (x = 0, y = 0, z = 0) => new THREE.Vector3(x, y, z);

// ------------------------------------------------------------------ species: the game's numbers (fauna.ron) and today's sway kind
const SPECIES = {
  perch: { model: "perch", name: "Banded perch", length: 0.38, speed: [0.8, 2.4], sway: "Fish", kind: 0, nose: 0.5, spine: 1.049, span: 0.11, track: 1.1 },
  eel:   { model: "eel",   name: "Eel",          length: 0.9,  speed: [0.7, 1.9], sway: "Eel",  kind: 0, nose: 0.5, spine: 1.03,  span: 0.067, track: 1.7 },
  ray:   { model: "ray",   name: "Ray",          length: 0.6,  speed: [0.5, 1.2], sway: "Ray",  kind: 1, nose: 0.5, spine: 1.0,   span: 0.443, track: 1.6 },
};

// ------------------------------------------------------------------ proposed tuning, per species: [key, label, min, max, step, unit]
const WGROUPS = [
  ["Wave frequency", [
    ["f0", "Beat at rest", 0, 3, 0.05, "Hz"],
    ["fk", "Beat per body length/s", 0, 3, 0.05, "Hz"],
    ["fHover", "Scull when hovering", 0, 4, 0.05, "Hz"],
    ["fMax", "Top beat", 1, 14, 0.1, "Hz"],
  ]],
  ["Amplitude envelope", [
    ["aCruise", "Tail amplitude cruising", 0, 0.25, 0.005, "len"],
    ["aBurst", "Tail amplitude bursting", 0, 0.3, 0.005, "len"],
    ["aHover", "Tail amplitude hovering", 0, 0.15, 0.005, "len"],
    ["aGlide", "Tail amplitude gliding", 0, 0.1, 0.002, "len"],
    ["head", "Amplitude at the nose", 0, 1, 0.01, "x tail"],
    ["expo", "Envelope exponent", 0.2, 4, 0.05, "power"],
    ["aTau", "Amplitude easing", 0.02, 1, 0.01, "s"],
  ]],
  ["Wavelength", [
    ["waves", "Waves along the body", 0.2, 3, 0.05, "per body"],
  ]],
  ["Head recoil", [
    ["recoil", "Head recoil", 0, 1, 0.01, "x tail"],
    ["recoilU", "Recoil reach from the nose", 0.05, 1, 0.01, "body"],
  ]],
  ["Turn bend", [
    ["bendGain", "Bend at 90 deg/s", 0, 0.3, 0.005, "len"],
    ["bendMax", "Most bend", 0, 0.4, 0.005, "len"],
    ["bendTau", "Bend easing", 0.02, 1, 0.01, "s"],
  ]],
  ["Glide and hover", [
    ["glideAcc", "Glide threshold (slowing)", 0, 4, 0.05, "len/s2"],
    ["hoverBelow", "Hover below", 0, 2, 0.05, "len/s"],
  ]],
  ["Ray wings", [
    ["wingExpo", "Wing span exponent", 0.3, 4, 0.05, "power"],
    ["wingWaves", "Waves along the wing", 0, 2, 0.05, "per chord"],
    ["flap", "Undulate (0) to flap (1)", 0, 1, 0.01, ""],
    ["whip", "Tail whip amplitude", 0, 0.15, 0.005, "len"],
  ], "ray"],
];
const DEFAULTS = {
  perch: { f0: 0.5, fk: 1.0, fHover: 1.2, fMax: 8, aCruise: 0.075, aBurst: 0.11, aHover: 0.025, aGlide: 0.004, head: 0.04, expo: 2.0, aTau: 0.18,
    waves: 0.95, recoil: 0.35, recoilU: 0.4, bendGain: 0.06, bendMax: 0.14, bendTau: 0.15, glideAcc: 0.35, hoverBelow: 0.4,
    wingExpo: 1.6, wingWaves: 0.7, flap: 0, whip: 0.05 },
  eel: { f0: 0.4, fk: 1.0, fHover: 0.7, fMax: 4, aCruise: 0.07, aBurst: 0.09, aHover: 0.03, aGlide: 0.01, head: 0.3, expo: 0.8, aTau: 0.25,
    waves: 1.25, recoil: 0.0, recoilU: 0.3, bendGain: 0.1, bendMax: 0.22, bendTau: 0.2, glideAcc: 0.3, hoverBelow: 0.3,
    wingExpo: 1.6, wingWaves: 0.7, flap: 0, whip: 0.05 },
  ray: { f0: 0.4, fk: 0.6, fHover: 0.6, fMax: 3, aCruise: 0.08, aBurst: 0.12, aHover: 0.03, aGlide: 0.006, head: 0.0, expo: 1.0, aTau: 0.25,
    waves: 1.0, recoil: 0.0, recoilU: 0.3, bendGain: 0.04, bendMax: 0.1, bendTau: 0.2, glideAcc: 0.3, hoverBelow: 0.3,
    wingExpo: 1.6, wingWaves: 0.7, flap: 0, whip: 0.05 },
};
const W = {}; for (const k in DEFAULTS) W[k] = { ...DEFAULTS[k] };
const ws = { species: "perch", view: "school", force: "auto", debug: false };

// ------------------------------------------------------------------ TODAY: bodies.rs `sway`, per vertex, model units
function todaySway(kind, x, z, t, phase){
  if (kind === "Fish"){ const w = clamp((0.3 - x) / 0.8, 0, 1); return [0, Math.sin(t * 7 + phase - x * 6) * 0.075 * w * w]; }
  if (kind === "Eel"){ const w = clamp((0.55 - x) / 1.0, 0.15, 1); return [0, Math.sin(t * 3.5 + phase - x * 13) * 0.06 * w]; }
  const ph = phase + t * 1.6, s = 0.5 - x, zz = Math.abs(z) / 0.44;
  const flap = 0.07 * Math.sin(ph - s * 2.5) * zz * zz;
  const whip = s > 1 ? 0.05 * Math.sin(ph * 0.7 - s * 3) * (s - 1) : 0;
  return [flap, whip];
}

// ------------------------------------------------------------------ PROPOSED: the vertex shader (WGSL-portable GLSL)
const SWIM_GLSL = /* glsl */`
attribute float aU;      // place along the body: 0 at the nose, 1 at the tail (beyond 1: a ray's whip)
attribute float aSide;   // the vertex's distance off the spine across the body, model units (signed)
attribute vec4 aSwim;    // per fish: x wave phase (rad), y amplitude (body lengths), z turn bend (body lengths), w unused
uniform float uWaves, uExpo, uHead, uRecoil, uRecoilU, uSpine, uKind, uSpan, uWingExpo, uWingWaves, uFlap, uWhip;
const float SWIM_TAU = 6.28318530718;
// The lateral offset of the spine at u, and its slope d/du.
vec2 swimLateral(float u, float phase, float amp, float bend){
  float uc = clamp(u, 0.0, 1.0);
  float e = uHead + (1.0 - uHead) * pow(max(uc, 1e-3), uExpo);
  float de = (1.0 - uHead) * uExpo * pow(max(uc, 1e-3), uExpo - 1.0);
  float a = SWIM_TAU * uWaves * uc - phase;
  float tail = sin(SWIM_TAU * uWaves - phase);           // where the tail is now
  float r = max(1.0 - uc / uRecoilU, 0.0);
  float rec = -uRecoil * r * r * tail;                    // the head yaws against the tail
  float drec = 2.0 * uRecoil * r / uRecoilU * tail;
  float q = (uc - 0.35) / 0.65;                           // C-shape about the body's pivot
  float f = amp * (e * sin(a) + rec) + bend * q * q;
  float df = amp * (de * sin(a) + e * SWIM_TAU * uWaves * cos(a) + drec) + bend * 2.0 * q / 0.65;
  return vec2(f, df);
}
`;
const SWIM_BODY = /* glsl */`
  vec3 swimPos = position;
  vec3 swimN = objectNormal;
  float phase = aSwim.x, amp = aSwim.y, bend = aSwim.z;
  if (uKind < 0.5){
    // A swimmer: the spine bends sideways (Z); each cross-section turns with the spine's slope,
    // so the body keeps its thickness through a bend and its normals turn with it.
    vec2 fl = swimLateral(aU, phase, amp, bend);
    float slope = -fl.y / uSpine;                         // dz/dx: x runs nose (+) to tail (-)
    float c = inversesqrt(1.0 + slope * slope), s = slope * c;
    swimPos.x = position.x - aSide * s;
    swimPos.z = fl.x + aSide * c;
    swimN = vec3(objectNormal.x * c - objectNormal.z * s, objectNormal.y, objectNormal.x * s + objectNormal.z * c);
  } else {
    // A ray: the wings undulate in a wave running nose to tail along the chord (flap = 1 lifts them together),
    // growing out along the span; the whip behind the disc swings sideways.
    float sp = clamp(abs(aSide) / uSpan, 0.0, 1.0);
    float sg = aSide < 0.0 ? -1.0 : 1.0;
    float k = SWIM_TAU * uWingWaves * (1.0 - uFlap);
    float a = k * clamp(aU, 0.0, 1.0) - phase;
    float inner = 1.0 - clamp(bend * sg * 6.0, -0.6, 0.6); // the wing inside a turn beats less
    float g = amp * inner * pow(max(sp, 1e-3), uWingExpo) * sin(a);
    float gz = amp * inner * uWingExpo * pow(max(sp, 1e-3), uWingExpo - 1.0) / uSpan * sg * sin(a);
    float gx = amp * inner * pow(max(sp, 1e-3), uWingExpo) * cos(a) * k * (-1.0 / uSpine);
    swimPos.y = position.y + g;
    float w = max(aU - 1.0, 0.0);
    float wl = uWhip * w * sin(SWIM_TAU * 1.2 * aU - 0.7 * phase) + bend * aU * aU * 0.5;
    swimPos.z = position.z + wl;
    swimN = normalize(vec3(objectNormal.x - gx * objectNormal.y, objectNormal.y, objectNormal.z - gz * objectNormal.y));
  }
  objectNormal = swimN;
`;
// The same maths in JS for the spine overlay only (the shader is the one that draws).
function lateralJS(u, phase, amp, bend, p){
  const uc = clamp(u, 0, 1), e = p.head + (1 - p.head) * Math.pow(Math.max(uc, 1e-3), p.expo);
  const tail = Math.sin(TAU * p.waves - phase), r = Math.max(1 - uc / p.recoilU, 0), q = (uc - 0.35) / 0.65;
  return amp * (e * Math.sin(TAU * p.waves * uc - phase) - p.recoil * r * r * tail) + bend * q * q;
}

// ------------------------------------------------------------------ meshes
function baseGeometry(sp){
  const m = MODELS[sp.model], g = new THREE.BufferGeometry(), n = m.pos.length / 3;
  g.setAttribute("position", new THREE.Float32BufferAttribute(m.pos.slice(), 3));
  g.setAttribute("uv", new THREE.Float32BufferAttribute(m.uv, 2));
  g.setAttribute("color", new THREE.Float32BufferAttribute(m.col.slice(0, n * 3), 3));
  return g;
}
function todayFish(sp){
  const g = baseGeometry(sp), base = Float32Array.from(g.attributes.position.array);
  const mat = new THREE.MeshStandardMaterial({ map: getAtlas(), vertexColors: true, roughness: 0.8, side: THREE.DoubleSide, flatShading: true });
  const mesh = new THREE.Mesh(g, mat); mesh.frustumCulled = false; mesh.matrixAutoUpdate = false;
  return {
    mesh, mat,
    pose(t, phase){
      const a = g.attributes.position.array;
      for (let i = 0; i < base.length; i += 3){
        const [dy, dz] = todaySway(sp.sway, base[i], base[i + 2], t, phase);
        a[i] = base[i]; a[i + 1] = base[i + 1] + dy; a[i + 2] = base[i + 2] + dz;
      }
      g.attributes.position.needsUpdate = true;
    },
    spineAt(u, t, phase){ const x = sp.nose - u * sp.spine; return todaySway(sp.sway, x, 0, t, phase)[1]; },
  };
}
function proposedFish(sp, uniforms){
  const g = baseGeometry(sp), pos = g.attributes.position.array, n = pos.length / 3;
  g.computeVertexNormals();                 // non-indexed: one normal a face, the game's flat normals
  const au = new Float32Array(n), as = new Float32Array(n), sw = new Float32Array(n * 4);
  for (let i = 0; i < n; i++){ au[i] = (sp.nose - pos[i * 3]) / sp.spine; as[i] = pos[i * 3 + 2]; }
  g.setAttribute("aU", new THREE.BufferAttribute(au, 1));
  g.setAttribute("aSide", new THREE.BufferAttribute(as, 1));
  const swim = new THREE.BufferAttribute(sw, 4); swim.setUsage(THREE.DynamicDrawUsage); g.setAttribute("aSwim", swim);
  const mat = new THREE.MeshStandardMaterial({ map: getAtlas(), vertexColors: true, roughness: 0.8, side: THREE.DoubleSide });
  mat.onBeforeCompile = sh => {
    Object.assign(sh.uniforms, uniforms);
    sh.vertexShader = SWIM_GLSL + sh.vertexShader
      .replace("#include <beginnormal_vertex>", "#include <beginnormal_vertex>\n" + SWIM_BODY)
      .replace("#include <begin_vertex>", "vec3 transformed = swimPos;");
  };
  const mesh = new THREE.Mesh(g, mat); mesh.frustumCulled = false; mesh.matrixAutoUpdate = false;
  return {
    mesh, mat,
    pose(phase, amp, bend){
      for (let i = 0; i < n; i++){ sw[i * 4] = phase; sw[i * 4 + 1] = amp; sw[i * 4 + 2] = bend; }
      swim.needsUpdate = true;
    },
  };
}
function spineLine(colour){
  const N = 33, g = new THREE.BufferGeometry(); g.setAttribute("position", new THREE.BufferAttribute(new Float32Array(N * 3), 3));
  const line = new THREE.Line(g, new THREE.LineBasicMaterial({ color: colour, depthTest: false, transparent: true }));
  line.renderOrder = 10; line.frustumCulled = false; line.matrixAutoUpdate = false;
  const dots = new THREE.Points(g, new THREE.PointsMaterial({ color: colour, size: 4, sizeAttenuation: false, depthTest: false, transparent: true }));
  dots.renderOrder = 10; dots.frustumCulled = false; dots.matrixAutoUpdate = false;
  return { line, dots, N, set(fn, sp, matrix){ const a = g.attributes.position.array;
    for (let i = 0; i < N; i++){ const u = i / (N - 1); a[i * 3] = sp.nose - u * sp.spine; a[i * 3 + 1] = 0; a[i * 3 + 2] = fn(u); }
    g.attributes.position.needsUpdate = true; line.matrix.copy(matrix); dots.matrix.copy(matrix); } };
}

// ------------------------------------------------------------------ the swim: a figure of eight, the same motion on both sides
// A speed programme cycles burst, glide, cruise, hover; the draw side reads only speed, its change and the yaw rate,
// as the game's drawing would from the school's velocities.
const PROGRAMME = [["burst", 0.9], ["glide", 1.8], ["cruise", 2.6], ["burst", 0.6], ["glide", 1.4], ["hover", 2.4], ["cruise", 2.2]];
const PROG_T = PROGRAMME.reduce((s, p) => s + p[1], 0);
function programmeAt(t){ let x = ((t % PROG_T) + PROG_T) % PROG_T; for (const [s, d] of PROGRAMME){ if (x < d) return s; x -= d; } return "cruise"; }
class Track {
  constructor(centre, R, tilt){
    this.pts = []; this.cum = [0];
    const N = 720;
    for (let i = 0; i <= N; i++){ const th = i / N * TAU; this.pts.push(V(centre.x + R * Math.sin(th), centre.y + tilt * Math.sin(th * 2 + 0.6), centre.z + R * 0.75 * Math.sin(th) * Math.cos(th) * 1.6)); }
    for (let i = 1; i <= N; i++) this.cum.push(this.cum[i - 1] + this.pts[i].distanceTo(this.pts[i - 1]));
    this.len = this.cum[N];
  }
  at(s){
    s = ((s % this.len) + this.len) % this.len;
    let lo = 0, hi = this.cum.length - 1; while (hi - lo > 1){ const m = (lo + hi) >> 1; if (this.cum[m] <= s) lo = m; else hi = m; }
    const k = (s - this.cum[lo]) / Math.max(this.cum[hi] - this.cum[lo], 1e-6);
    return { p: this.pts[lo].clone().lerp(this.pts[hi], k), d: this.pts[hi].clone().sub(this.pts[lo]).normalize() };
  }
}
class Swimmer {
  constructor(sp, track, s0, t0, phase){ this.sp = sp; this.track = track; this.s = s0; this.t = t0; this.v = sp.speed[0]; this.vPrev = this.v;
    this.acc = 0; this.yaw = null; this.yawRate = 0; this.phase0 = phase; this.wave = phase; this.amp = 0.05; this.bend = 0; this.mode = "cruise";
    this.pos = V(); this.fwd = V(1, 0, 0); }
  // The motion (the same for TODAY and PROPOSED): speed eased toward the programme's target, carried along the track.
  move(dt, force){
    this.t += dt;
    const want = force !== "auto" ? (force === "turn" ? "cruise" : force) : programmeAt(this.t);
    const [lo, hi] = this.sp.speed;
    const target = { burst: hi, cruise: lo, glide: lo * 0.35, hover: 0.04 }[want];
    const tau = { burst: 0.3, cruise: 0.6, glide: 1.6, hover: 0.8 }[want];
    if (force !== this.forced){ this.forced = force; if (force !== "auto") this.v = target; }
    this.vPrev = this.v; this.v += (target - this.v) * (1 - Math.exp(-dt / tau));
    this.s += this.v * dt;
    const { p, d } = this.track.at(this.s); this.pos.copy(p); this.fwd.copy(d);
    const yaw = Math.atan2(-d.z, d.x);
    if (this.yaw !== null && dt > 0){ let dy = yaw - this.yaw; dy = Math.atan2(Math.sin(dy), Math.cos(dy)); this.yawRate += (dy / dt - this.yawRate) * Math.min(1, dt / 0.1); }
    this.yaw = yaw;
    this.acc += ((this.v - this.vPrev) / Math.max(dt, 1e-4) - this.acc) * Math.min(1, dt / 0.15);
  }
  // PROPOSED, draw side: the wave's phase, amplitude and bend from speed, its change and the yaw rate.
  swim(dt, p, yawRateOverride){
    const L = this.sp.length, vBL = this.v / L, aBL = this.acc / L;
    const cruiseBL = this.sp.speed[0] / L, burstBL = this.sp.speed[1] / L;
    let amp, hz;
    if (vBL < p.hoverBelow){ this.mode = "hover"; amp = p.aHover; hz = p.fHover; }
    else if (aBL < -p.glideAcc){ this.mode = "glide"; amp = p.aGlide; hz = Math.min(p.f0 + p.fk * vBL, p.fMax); }
    else { const k = clamp((vBL - cruiseBL) / Math.max(burstBL - cruiseBL, 1e-3), 0, 1); this.mode = aBL > 0.3 || k > 0.6 ? "burst" : "cruise";
      amp = lerp(p.aCruise, p.aBurst, k); hz = Math.min(p.f0 + p.fk * vBL, p.fMax); }
    this.amp += (amp - this.amp) * (1 - Math.exp(-dt / Math.max(p.aTau, 1e-3)));
    this.hz = hz; this.wave = (this.wave + TAU * hz * dt) % (TAU * 1000);
    const yr = yawRateOverride !== undefined ? yawRateOverride : this.yawRate;
    // A left turn (positive yaw about up) bows the body to the left: model -Z.
    const bendWant = clamp(-p.bendGain * (yr / DEG) / 90, -p.bendMax, p.bendMax);
    this.bend += (bendWant - this.bend) * (1 - Math.exp(-dt / Math.max(p.bendTau, 1e-3)));
  }
}

// ------------------------------------------------------------------ one world: the scene, three swimmers and the close-up fish
const CENTRE = V(-0.7, -1.6, -0.55), CLOSE = V(-0.95, -2.0, -0.75);
class WaveWorld {
  constructor(kind, spId){
    this.kind = kind; this.proposed = kind === "proposed"; this.sp = SPECIES[spId]; this.p = W[spId];
    this.scene = new THREE.Scene(); this.env = environment(this.scene); this.clock = 0;
    const sp = this.sp, p = this.p;
    this.uniforms = { uWaves: { value: 0 }, uExpo: { value: 0 }, uHead: { value: 0 }, uRecoil: { value: 0 }, uRecoilU: { value: 0 },
      uSpine: { value: sp.spine }, uKind: { value: sp.kind }, uSpan: { value: sp.span }, uWingExpo: { value: 0 }, uWingWaves: { value: 0 },
      uFlap: { value: 0 }, uWhip: { value: 0 } };
    this.swimmers = []; this.fish = []; this.spines = [];
    const R = sp.track, offs = [[0, 0, 1.0, 0.0], [0.35, -0.22, 0.86, 0.33], [0.7, 0.2, 1.12, 0.66]];
    const make = () => this.proposed ? proposedFish(sp, this.uniforms) : todayFish(sp);
    offs.forEach(([s0, dy, k, t0], i) => {
      const tr = new Track(V(CENTRE.x, CENTRE.y + dy, CENTRE.z), R * k, 0.12);
      this.swimmers.push(new Swimmer(sp, tr, s0 * tr.len, t0 * PROG_T + i * 0.7, i * 2.1));
      const f = make(); this.scene.add(f.mesh); this.fish.push(f);
      const sl = spineLine(this.proposed ? "#ffd27a" : "#9fe6ff"); this.scene.add(sl.line, sl.dots); this.spines.push(sl);
    });
    // The close-up: fish 0's swim, on a treadmill (it stays put, facing +X; its bend still follows the turn).
    this.close = make(); this.scene.add(this.close.mesh);
    this.closeSpine = spineLine(this.proposed ? "#ffd27a" : "#9fe6ff"); this.scene.add(this.closeSpine.line, this.closeSpine.dots);
    this.closeSwim = new Swimmer(sp, this.swimmers[0].track, 0, 0, 0.4);
    this.m = new THREE.Matrix4();
  }
  syncUniforms(){ const u = this.uniforms, p = this.p;
    u.uWaves.value = p.waves; u.uExpo.value = p.expo; u.uHead.value = p.head; u.uRecoil.value = p.recoil; u.uRecoilU.value = p.recoilU;
    u.uWingExpo.value = p.wingExpo; u.uWingWaves.value = p.wingWaves; u.uFlap.value = p.flap; u.uWhip.value = p.whip; }
  place(f, pos, fwd, len){
    const top0 = V(0, 1, 0), side = fwd.clone().cross(top0).normalize(), top = side.clone().cross(fwd).normalize();
    this.m.makeBasis(fwd.clone().multiplyScalar(len), top.multiplyScalar(len), side.multiplyScalar(len)).setPosition(pos);
    f.mesh.matrix.copy(this.m); return this.m;
  }
  update(dt){
    this.clock += dt; this.syncUniforms();
    const school = ws.view === "school", sp = this.sp, p = this.p;
    this.swimmers.forEach((s, i) => {
      s.move(dt, ws.force); const f = this.fish[i], sl = this.spines[i];
      f.mesh.visible = school; sl.line.visible = sl.dots.visible = school && ws.debug;
      if (!school) return;
      const m = this.place(f, s.pos, s.fwd, sp.length);
      if (this.proposed){ s.swim(dt, p); f.pose(s.wave, s.amp, s.bend); sl.set(u => sp.kind ? 0 : lateralJS(u, s.wave, s.amp, s.bend, p), sp, m); }
      else { f.pose(this.clock, s.phase0); sl.set(u => f.spineAt(u, this.clock, s.phase0), sp, m); }
    });
    const c = this.closeSwim; c.move(dt, ws.force);
    this.close.mesh.visible = !school; this.closeSpine.line.visible = this.closeSpine.dots.visible = !school && ws.debug;
    if (!school){
      const m = this.place(this.close, CLOSE, V(1, 0, 0), sp.length);
      if (this.proposed){ c.swim(dt, p, ws.force === "turn" ? 120 * DEG : undefined); this.close.pose(c.wave, c.amp, c.bend);
        this.closeSpine.set(u => sp.kind ? 0 : lateralJS(u, c.wave, c.amp, c.bend, p), sp, m); }
      else { this.close.pose(this.clock, c.phase0); this.closeSpine.set(u => this.close.spineAt(u, this.clock, c.phase0), sp, m); }
    }
    this.env.waves(this.clock);
  }
  debug(on){ [...this.fish, this.close].forEach(f => { f.mat.wireframe = on; }); }
  label(){
    const c = ws.view === "school" ? this.swimmers[0] : this.closeSwim, L = this.sp.length;
    const v = (c.v / L).toFixed(1) + " len/s";
    if (!this.proposed){ const hz = { Fish: 7, Eel: 3.5, Ray: 1.6 }[this.sp.sway] / TAU; return `TODAY · ${v} · beat ${hz.toFixed(2)} Hz, fixed`; }
    const turn = ws.view !== "school" && ws.force === "turn" ? " · TURN 120 deg/s" : "";
    return `PROPOSED · ${c.mode.toUpperCase()} · ${v} · ${(c.hz || 0).toFixed(1)} Hz · A ${c.amp.toFixed(3)}${turn}`;
  }
}

// ------------------------------------------------------------------ the view: one renderer; Split draws both halves of the one canvas
const canvas = $("cv-wave"), hud = $("hud-wave");
let renderer = null, raf = 0, worlds = null, last = 0;
const cams = { school: { yaw: -0.45, pitch: 0.33, dist: 2.8, target: CENTRE.clone() }, close: { yaw: 0.35, pitch: 0.95, dist: 0.85, target: CLOSE.clone() } };
const closeDist = { perch: 0.85, eel: 1.6, ray: 1.3 }, schoolDist = { perch: 2.8, eel: 4.2, ray: 3.8 };
const cam = new THREE.PerspectiveCamera(46, 16 / 10, 0.02, 120);
const cv = () => cams[ws.view === "school" ? "school" : "close"];
let drag = null; const touches = new Map(); let pinch0 = 0, dist0 = 0;
canvas.addEventListener("pointerdown", e => { touches.set(e.pointerId, [e.clientX, e.clientY]); canvas.setPointerCapture(e.pointerId); const c = cv();
  if (touches.size === 1) drag = [e.clientX, e.clientY, c.yaw, c.pitch]; else { const [a, b] = [...touches.values()]; pinch0 = Math.hypot(a[0] - b[0], a[1] - b[1]); dist0 = c.dist; drag = null; } });
canvas.addEventListener("pointermove", e => { if (!touches.has(e.pointerId)) return; touches.set(e.pointerId, [e.clientX, e.clientY]); const c = cv();
  if (touches.size >= 2){ const [a, b] = [...touches.values()]; const d = Math.hypot(a[0] - b[0], a[1] - b[1]); if (pinch0 > 0) c.dist = clamp(dist0 * pinch0 / d, 0.3, 20); return; }
  if (!drag) return; c.yaw = drag[2] - (e.clientX - drag[0]) * 0.008; c.pitch = clamp(drag[3] + (e.clientY - drag[1]) * 0.006, -1.4, 1.4); });
const up = e => { touches.delete(e.pointerId); drag = null; if (touches.size === 1){ const [q] = [...touches.values()]; const c = cv(); drag = [q[0], q[1], c.yaw, c.pitch]; } };
canvas.addEventListener("pointerup", up); canvas.addEventListener("pointercancel", up);
canvas.addEventListener("wheel", e => { e.preventDefault(); const c = cv(); c.dist = clamp(c.dist * (1 + Math.sign(e.deltaY) * 0.1), 0.3, 20); }, { passive: false });

function build(){
  worlds = { today: new WaveWorld("today", ws.species), proposed: new WaveWorld("proposed", ws.species) };
  for (const w of Object.values(worlds)){ w.env.night(state.night); w.debug(ws.debug); }
}
function frame(now){
  raf = requestAnimationFrame(frame);
  const dtReal = Math.min(0.05, last ? (now - last) / 1000 : 0); last = now;
  const dt = dtReal * state.speed;
  const r = canvas.getBoundingClientRect(); if (!r.width) return;
  const pr = renderer.getPixelRatio();
  if (canvas.width !== Math.round(r.width * pr) || canvas.height !== Math.round(r.height * pr)) renderer.setSize(r.width, r.height, false);
  if (dt > 0){ worlds.today.update(dt); worlds.proposed.update(dt); }
  const c = cv(), t = c.target;
  cam.position.set(t.x + Math.sin(c.yaw) * Math.cos(c.pitch) * c.dist, t.y + Math.sin(c.pitch) * c.dist, t.z + Math.cos(c.yaw) * Math.cos(c.pitch) * c.dist); cam.lookAt(t);
  const Wd = r.width, H = r.height;
  renderer.setScissorTest(state.mode === "split");
  if (state.mode === "split"){
    cam.aspect = (Wd / 2) / H; cam.updateProjectionMatrix();
    renderer.setViewport(0, 0, Wd / 2, H); renderer.setScissor(0, 0, Wd / 2, H); renderer.render(worlds.today.scene, cam);
    renderer.setViewport(Wd / 2, 0, Wd / 2, H); renderer.setScissor(Wd / 2, 0, Wd / 2, H); renderer.render(worlds.proposed.scene, cam);
  } else { cam.aspect = Wd / H; cam.updateProjectionMatrix(); renderer.setViewport(0, 0, Wd, H); renderer.render(worlds[state.mode].scene, cam); }
  hud.querySelector(".l").textContent = state.mode === "proposed" ? "" : worlds.today.label();
  hud.querySelector(".r").textContent = state.mode === "today" ? "" : worlds.proposed.label();
  hud.classList.toggle("one", state.mode !== "split");
}

// ------------------------------------------------------------------ the tab's own controls and sliders
function seg(id, key, values, after){
  const host = $(id);
  host.querySelectorAll("button").forEach((b, i) => b.onclick = () => {
    ws[key] = values[i]; host.querySelectorAll("button").forEach((x, j) => x.setAttribute("aria-pressed", j === i)); if (after) after();
  });
}
const inputs = {};
function showSliders(){
  const p = W[ws.species];
  for (const k in inputs){ const { input, out, unit } = inputs[k]; input.value = p[k]; const v = +p[k]; out.textContent = (Math.abs(v) >= 100 ? v.toFixed(0) : +v.toFixed(3)) + (unit ? " " + unit : ""); }
  document.querySelectorAll("#wave-tuning .grp").forEach(d => { d.hidden = d.dataset.only && d.dataset.only !== ws.species; });
  document.querySelectorAll("#wave-tuning .bodyonly").forEach(d => { d.hidden = ws.species === "ray"; });
}
(function sliders(){
  const host = $("wave-tuning");
  for (const [name, list, only] of WGROUPS){
    const d = document.createElement("details"); d.className = "grp"; if (only) d.dataset.only = only;
    if (name === "Head recoil" || name === "Wavelength") d.classList.add("bodyonly");
    const s = document.createElement("summary"); s.textContent = name; d.appendChild(s);
    const box = document.createElement("div"); box.className = "sliders"; d.appendChild(box);
    for (const [key, label, min, max, step, unit] of list){
      const row = document.createElement("div"); row.className = "sl"; const id = "w-" + key;
      row.innerHTML = `<label for="${id}">${label}</label><input id="${id}" type="range" min="${min}" max="${max}" step="${step}"><output for="${id}"></output>`;
      const input = row.querySelector("input"), out = row.querySelector("output");
      inputs[key] = { input, out, unit };
      input.oninput = () => { W[ws.species][key] = +input.value; showSliders(); };
      box.appendChild(row);
    }
    host.appendChild(d);
  }
  $("bt-wave-reset").onclick = () => { W[ws.species] = { ...DEFAULTS[ws.species] }; if (worlds){ worlds.today.p = worlds.proposed.p = W[ws.species]; } showSliders(); };
  showSliders();
})();
seg("sg-wave-species", "species", ["perch", "eel", "ray"], () => { cams.close.dist = closeDist[ws.species]; cams.close.pitch = ws.species === "ray" ? 0.3 : 0.95; cams.school.dist = schoolDist[ws.species]; showSliders(); if (renderer) build(); });
seg("sg-wave-view", "view", ["school", "close"]);
seg("sg-wave-force", "force", ["auto", "burst", "cruise", "glide", "hover", "turn"]);
$("ck-wave-debug").onchange = e => { ws.debug = e.target.checked; if (worlds) for (const w of Object.values(worlds)) w.debug(ws.debug); };

return {
  show(){
    if (!renderer){
      loadTextures();
      renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
      renderer.outputEncoding = THREE.sRGBEncoding; renderer.setPixelRatio(Math.min(2, devicePixelRatio));
      build();
    }
    last = 0; if (!raf) raf = requestAnimationFrame(frame);
  },
  hide(){ cancelAnimationFrame(raf); raf = 0; },
  replay(){ if (renderer) build(); },
  strike(){},
  night(){ if (worlds) for (const w of Object.values(worlds)) w.env.night(state.night); },
  rebuild(){},
  _w: () => worlds, ws, W,
};
};
