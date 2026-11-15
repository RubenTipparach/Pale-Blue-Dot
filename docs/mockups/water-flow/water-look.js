// The water look shared by the mockups that draw hex water: M25 (ocean.html: the sea, every fresh-water cap and fall,
// and the "Water sim" tab) and M28 (water-flow.html). One copy, loaded by both pages, as water-flow/liquid.js is for
// the rule, so the sim's water in the two can never look different. Needs three.js r128 (THREE) before it; the sim
// builder (simWater) needs water-flow/liquid.js (PBDLiquid) too.
//
// What a page gives it: a scene object W = {scene, U, opts, seas}. W.U holds the uniforms the water shares with the
// page's own lit materials, spread into each water material:
//   uLampPos[n] (vec3), uLampCol[n] (vec4: linear rgb times intensity, w its range in metres), uLampCount: the lamps lit
//     now, every one lighting the water within its range wherever the camera is (the shader's loop is sized to n);
//   uWL, uWLR: the baked water-level map (r, g the level, b set where there is water, a the depth to the bed times 4)
//     and the x0, z0, x1, z1 it covers; outside it the water reads 40 m deep;
//   uK, uKs, uWCol, uCamWL: absorption and scattering per metre, the water's own colour and the water level over the
//     eye, for the view from under water; uHaze, uFogR: the air's haze colour and the distances it fades in over.
// W.opts.dim, W.skyCol and W.sunK may darken the sky; W.seas collects each material's controller, whose setTime(T) and
// frame(t, eye) the page calls (T = {sky, haze, sun, moon, hemi, sunPos}). PBDWater.uniforms(n) makes a W.U.
(function (root) {
"use strict";
const HW = 2.833, HR = HW / Math.sqrt(3);
const hx = (q, r) => HW * (q + r / 2), hz = r => 1.5 * HR * r;
const CORNER = Array.from({length: 6}, (_, i) => { const a = Math.PI / 180 * (60 * i - 30); return [HR * Math.cos(a), HR * Math.sin(a)]; });
const DIRS = [[1, 0], [0, 1], [-1, 1], [-1, 0], [0, -1], [1, -1]];   // side i faces 60 i degrees
const sstep = (a, b, x) => { const t = Math.max(0, Math.min(1, (x - a) / (b - a))); return t * t * (3 - 2 * t); };
function uniforms(n = 40) {
  return {uLampPos: {value: Array.from({length: n}, () => new THREE.Vector3())}, uLampCol: {value: Array.from({length: n}, () => new THREE.Vector4())}, uLampCount: {value: 0},
    uWL: {value: null}, uWLR: {value: new THREE.Vector4(-1, -1, 1, 1)}, uK: {value: new THREE.Vector3(0.4, 0.1, 0.16)}, uKs: {value: 0.12}, uWCol: {value: new THREE.Color(0.1, 0.17, 0.12)},
    uHaze: {value: new THREE.Color()}, uFogR: {value: new THREE.Vector2(150, 900)}, uCamWL: {value: -1000}};
}
// ======================= the water surface =======================
// After drydock.html's sea (M23), itself after the game's sea.wgsl: deep-water sines (omega = sqrt(g k), the game's
// g of 25 m/s^2) scaled by the sea state, the same sum in JS so floating things ride the water they are drawn on.
// Each wave fades out of the geometry between 3 and 6 wavelengths from the eye, and the normals and foam are worked
// out per pixel, never interpolated across a long thin triangle: no radial-mesh aliasing at a distance.
// One material draws the sea (a polar mesh that follows the eye) and every fresh-water cap and fall (hex caps at their
// own level, carrying a flow vector the streaks advect along: the game's flow hook, zero today).
const WAVES = [[0.42, 58, 0.30, 0.95, 0.0], [0.28, 34, -0.55, 0.83, 1.7], [0.18, 21, 0.82, 0.57, 3.1], [0.10, 13, -0.2, -0.98, 0.6], [0.05, 7.5, 0.6, -0.5, 2.2]].map(([a, lam, dx, dz, ph]) => { const k = 2 * Math.PI / lam, l = Math.hypot(dx, dz); return {a, k, lam, dx: dx / l, dz: dz / l, w: Math.sqrt(25 * k), ph}; });
function seaH(x, z, t, amp, eye) { let h = 0; for (const W1 of WAVES) { const fade = eye ? 1 - sstep(3 * W1.lam, 6 * W1.lam, Math.hypot(x - eye.x, z - eye.z)) : 1; h += W1.a * amp * fade * Math.sin(W1.k * (W1.dx * x + W1.dz * z) - W1.w * t + W1.ph); } return h; }
const WAVE_GLSL = `float etaAt(vec2 p, float t, float amp) {
  float eta = 0.0; float d = distance(p, cameraPosition.xz);
  ${WAVES.map(w1 => `eta += ${w1.a.toFixed(3)} * amp * (1.0 - smoothstep(${(3 * w1.lam).toFixed(1)}, ${(6 * w1.lam).toFixed(1)}, d)) * sin(${w1.k.toFixed(4)} * dot(vec2(${w1.dx.toFixed(4)}, ${w1.dz.toFixed(4)}), p) - ${w1.w.toFixed(4)} * t + ${w1.ph.toFixed(2)});`).join("\n  ")}
  return eta;
}
vec2 gradAt(vec2 p, float t, float amp) {
  vec2 g = vec2(0.0); float d = distance(p, cameraPosition.xz);
  ${WAVES.map(w1 => `g += ${w1.a.toFixed(3)} * amp * (1.0 - smoothstep(${(3 * w1.lam).toFixed(1)}, ${(6 * w1.lam).toFixed(1)}, d)) * ${w1.k.toFixed(4)} * vec2(${w1.dx.toFixed(4)}, ${w1.dz.toFixed(4)}) * cos(${w1.k.toFixed(4)} * dot(vec2(${w1.dx.toFixed(4)}, ${w1.dz.toFixed(4)}), p) - ${w1.w.toFixed(4)} * t + ${w1.ph.toFixed(2)});`).join("\n  ")}
  return g;
}`;
// ======================= the flow texture =======================
// Tenebris's water tile (owner, 2026-11-15: "Add UV scrolling and water texture or shaders to show the flow map. This
// is how tenebris c does it"). The 16 x 16 pixels of tenebris-rs/assets/textures/water.png (RubenTipparach/tenebris at
// ef9865166349c7a3c4ae09acdd20640daf85b4fd; the same file planets/tenebris/water.png copies byte-identical), as the
// index of each pixel's colour, darkest first: 0 the body (29,43,83), 1 a ripple (28,94,172), 2 a glint (88,138,192).
// The shader draws the three as three tones of the water it is on (its own colour, a lighter ripple, a glint lit like
// the foam), so the pattern is palette-true at any hour and lit by the lamps at night. Sampled nearest, never filtered.
// The scroll is Tenebris C's (src/shaders/water.glsl, water_fs): each cap pans its pattern along its own flow vector,
// one constant vector per cell (hex_mesher.c bake_water_flow_uv), so a single phase never stretches and needs no
// two-phase blend; a falling face pans straight down (u_flow_speed_falling). Faster flow pans faster; still water
// stands still. design.md 7c.
const TEX_IDX = "0000011000111100100000000000000000010121000000001000000000001012001000000100000000001001000000000000000000110010010000000000000000000101201000001000000000000122000000000000000010112221010001000000000000000000001000000002100020001001100000010000000000000000";
const TEX_M = 2.0;      // metres one 16-pixel tile covers: 12.5 cm a pixel, an eighth of a layer
const PAN_K = 1.6;      // metres a second a cap's pattern pans per unit of its flow vector (the streaks' own pace)
const FALL_PAN = 2.4;   // metres a second a falling sheet's pattern pans down
let flowTex = null;
function flowTexture() {
  if (flowTex) return flowTex;
  const d = new Uint8Array(16 * 16 * 4);
  for (let i = 0; i < 256; i++) { const v = +TEX_IDX[i] * 127; d[4 * i] = d[4 * i + 1] = d[4 * i + 2] = v; d[4 * i + 3] = 255; }
  // the PNG's first row is the top: flip so v runs up the tile as the image reads
  const f = new Uint8Array(d.length); for (let y = 0; y < 16; y++) f.set(d.subarray((15 - y) * 64, (16 - y) * 64), y * 64);
  flowTex = new THREE.DataTexture(f, 16, 16, THREE.RGBAFormat);
  flowTex.magFilter = flowTex.minFilter = THREE.NearestFilter; flowTex.wrapS = flowTex.wrapT = THREE.RepeatWrapping; flowTex.generateMipmaps = false; flowTex.needsUpdate = true;
  return flowTex;
}
function waterMat(W, o = {}) {
  const MAXL = W.U.uLampPos.value.length;   // the page's lamp array: the shader's loop is sized to it
  const U = Object.assign({}, W.U, {uT: {value: 0}, uAmp: {value: o.amp ?? 0.4}, uCenter: {value: new THREE.Vector2()}, uLevel: {value: 0}, uFollow: {value: o.follow ? 1 : 0},
    uSky: {value: new THREE.Color()}, uHor: {value: new THREE.Color()}, uSun: {value: new THREE.Vector3(-0.4, 0.8, 0.3).normalize()}, uSunI: {value: 1}, uMoonI: {value: 0}, uAmb: {value: 0.5},
    uFoamK: {value: o.foam ?? 1}, uMud: {value: o.mud ?? 0}, uSeaOnly: {value: o.seaOnly ? 1 : 0}, uMinA: {value: o.minA ?? 0.55},
    uFlowTex: {value: flowTexture()}, uTexK: {value: o.tex ?? 0}});
  const mat = new THREE.ShaderMaterial({uniforms: U, transparent: true, depthWrite: false, side: THREE.DoubleSide,
    vertexShader: `uniform float uT; uniform float uAmp; uniform vec2 uCenter; uniform float uLevel; uniform float uFollow;
attribute vec2 aFlow; attribute float aFall; attribute float aTop;
varying vec3 vW; varying vec2 vFlow; varying float vFall; varying float vTop;
${WAVE_GLSL}
void main() {
  vec3 p = position; if (uFollow > 0.5) { p.xz += uCenter; p.y = uLevel; }
  float eta = (1.0 - step(0.5, abs(aFall))) * etaAt(p.xz, uT, uAmp);   // only a cap rides the waves; a fall or a cut stands
  vW = vec3(p.x, p.y + eta, p.z); vFlow = aFlow; vFall = aFall; vTop = aTop;
  gl_Position = projectionMatrix * viewMatrix * vec4(vW, 1.0);
}`,
    fragmentShader: `uniform vec3 uSky; uniform vec3 uHor; uniform vec3 uSun; uniform float uSunI; uniform float uMoonI; uniform float uAmb; uniform float uT; uniform float uAmp; uniform float uFoamK; uniform float uMud; uniform float uSeaOnly; uniform float uMinA;
uniform vec3 uLampPos[${MAXL}]; uniform vec4 uLampCol[${MAXL}]; uniform int uLampCount;
uniform sampler2D uWL; uniform vec4 uWLR; uniform vec3 uK; uniform float uKs; uniform vec3 uWCol; uniform vec3 uHaze; uniform vec2 uFogR; uniform float uCamWL;
uniform sampler2D uFlowTex; uniform float uTexK;
varying vec3 vW; varying vec2 vFlow; varying float vFall; varying float vTop;
float hash(vec2 q) { return fract(sin(dot(q, vec2(127.1, 311.7))) * 43758.5453); }
${WAVE_GLSL}
void main() {
  float depth = 40.0;
  vec2 uv = (vW.xz - uWLR.xy) / (uWLR.zw - uWLR.xy);
  if (uv.x >= 0.0 && uv.y >= 0.0 && uv.x <= 1.0 && uv.y <= 1.0) {
    vec4 t = texture2D(uWL, uv);
    float lvl = (floor(t.r * 255.0 + 0.5) * 256.0 + floor(t.g * 255.0 + 0.5)) / 100.0 - 200.0;
    if (uSeaOnly > 0.5 && (t.b < 0.5 || abs(lvl - vW.y) > 0.35 + uAmp)) discard;
    depth = t.a * 255.0 / 4.0;
  }
  // vFall: 0 a cap, 1 a falling sheet, -1 a cut (the side of the water where the patch ends: still, standing on its bed)
  bool cut = vFall < -0.5, side = abs(vFall) > 0.5;
  if (cut) depth = max(0.4, vTop - vW.y + 1.0);
  vec2 gr = side ? vec2(0.0) : gradAt(vW.xz, uT, uAmp);
  vec3 n = normalize(vec3(-gr.x, 1.0, -gr.y)); if (vFall > 0.5) n = normalize(vec3(vFlow.x, 0.2, vFlow.y)); if (cut) n = vec3(vFlow.x, 0.0, vFlow.y);
  vec3 C = cameraPosition, v = normalize(C - vW); float dist = length(C - vW);
  vec3 col; float alpha;
  if (C.y < vW.y - 0.02 && !side) {
    // from below: Snell's window overhead, the water's own colour mirrored outside it
    vec3 d = -v; float up = d.y;
    float win = smoothstep(0.64, 0.72, up);
    vec3 through = mix(uHor, uSky, clamp(up, 0.0, 1.0)) * (0.6 + 0.6 * uSunI) + vec3(1.0, 0.95, 0.8) * pow(max(dot(d, uSun), 0.0), 60.0) * 2.0 * uSunI;
    col = mix(uWCol * 1.3, through, win);
    float path = dist;
    col = col * exp(-uK * path) + uWCol * (1.0 - exp(-uKs * path));
    alpha = 1.0;
  } else {
    float vCrest = etaAt(vW.xz, uT, uAmp) / max(0.01, 0.75 * uAmp);
    vec3 deep = vec3(0.012, 0.06, 0.12), shallow = vec3(0.07, 0.32, 0.34);
    deep = mix(deep, vec3(0.05, 0.05, 0.02), uMud); shallow = mix(shallow, vec3(0.16, 0.15, 0.07), uMud);
    vec3 base = mix(shallow, deep, smoothstep(0.5, 14.0, depth));
    col = base * (uAmb + 0.55 * uSunI * max(dot(n, uSun), 0.0));
    vec3 r = reflect(-v, n); vec3 sky = mix(uHor, uSky, clamp(r.y * 2.5, 0.0, 1.0));
    float fres = 0.02 + 0.98 * pow(1.0 - max(dot(n, v), 0.0), 5.0);
    col = mix(col, sky, fres * 0.75);
    col += vec3(1.0, 0.95, 0.85) * pow(max(dot(reflect(-uSun, n), v), 0.0), 180.0) * 1.4 * uSunI;
    col += vec3(0.75, 0.82, 1.0) * pow(max(dot(reflect(normalize(vec3(0.4, 0.5, -0.6)), n), v), 0.0), 40.0) * 0.3 * uMoonI;
    vec3 lampSum = vec3(0.0);
    for (int i = 0; i < ${MAXL}; i++) {
      if (i >= uLampCount) break;
      vec3 Lv = uLampPos[i] - vW; float d = length(Lv), rr = uLampCol[i].w * 1.3; if (d > rr) continue;
      float f = 1.0 - d / rr; f *= f; vec3 L = Lv / max(d, 1e-3);
      col += (base * 0.9 * max(dot(n, L), 0.0) + pow(max(dot(reflect(-L, n), v), 0.0), 24.0) * 0.6) * uLampCol[i].rgb * f;
      lampSum += uLampCol[i].rgb * f * (0.3 + 0.7 * max(dot(n, L), 0.0));
    }
    // foam in 0.5 m pixels: crests by the sea state, the shore, falls and fast flow (streaks advected downstream)
    vec2 adv = vW.xz - vFlow * uT * 1.6; vec2 px = floor(adv * 2.0); float nse = hash(px);
    float brk = hash(floor(adv * 1.3) + 17.0);   // whitecaps break in small patches, never along a whole swell crest
    float foam = smoothstep(1.02 - 0.2 * uFoamK, 1.35, vCrest) * step(0.8 - 0.3 * uFoamK, brk) * step(0.35, nse) * (0.25 + 0.4 * uFoamK);
    // Surf (owner, 2026-11-15: "depth foam on coast should be a bit farther off the coast"): waves break
    // offshore, where the water is one to two wave heights deep, not at the waterline. A break band
    // a few metres out, lines of white water rolling in from it towards the beach, and only a thin
    // swash at the sand itself.
    float brkD = clamp(1.8 + uAmp * 8.0, 1.6, 4.0);
    float surfBand = smoothstep(brkD - 1.1, brkD - 0.3, depth) * (1.0 - smoothstep(brkD + 0.2, brkD + 1.2, depth));
    // the white water rolls in on the main swell's own crests (its phase, smooth in space and time), not on the
    // depth, which steps a metre at every hex column and made whole cells blink: depth only says where the surf zone is
    float swellPh = sin(${WAVES[2].k.toFixed(4)} * dot(vec2(${WAVES[2].dx.toFixed(4)}, ${WAVES[2].dz.toFixed(4)}), vW.xz) - ${WAVES[2].w.toFixed(4)} * uT + ${WAVES[2].ph.toFixed(4)});
    float rollIn = smoothstep(0.55, 0.9, swellPh) * step(0.25, depth) * (1.0 - smoothstep(brkD - 0.4, brkD + 0.4, depth));
    float swash = 1.0 - smoothstep(0.0, 0.3, depth);
    foam += (surfBand * (0.45 + 0.45 * nse) + rollIn * (0.25 + 0.35 * nse) + swash * 0.35 * nse) * step(0.15, uAmp);
    float spd = side ? 0.0 : length(vFlow);
    // the flow texture (Tenebris's tile): a cap's pattern pans along its flow vector, its streaks lying along the flow;
    // a falling sheet's pans straight down, its streaks standing; a cut's stands still
    vec2 fd = spd > 1e-4 ? vFlow / spd : vec2(1.0, 0.0);
    vec2 tuv = side ? vec2(vW.y + (vFall > 0.5 ? uT * ${FALL_PAN.toFixed(2)} : 0.0), dot(vW.xz, vec2(-vFlow.y, vFlow.x)))
                    : vec2(dot(vW.xz, fd) - spd * ${PAN_K.toFixed(2)} * uT, dot(vW.xz, vec2(-fd.y, fd.x)));
    float ti = texture2D(uFlowTex, tuv / ${TEX_M.toFixed(2)}).r * 2.0;
    float rip = step(0.5, ti), glint = step(1.5, ti);
    float tk = uTexK * (1.0 - smoothstep(60.0, 140.0, dist));   // a 12 cm pixel would shimmer far off
    foam += step(0.82 - 0.22 * clamp(spd, 0.0, 1.5), nse) * clamp(spd, 0.0, 1.0) * (0.55 - 0.3 * uTexK);
    if (vFall > 0.5) { vec2 fp = floor(vec2(vW.x + vW.z, vW.y * 3.0 + uT * 6.0) * 2.0); foam = mix(0.35 + 0.55 * step(0.45, hash(fp)), 0.32 + 0.3 * rip + 0.3 * glint, uTexK); }
    if (cut) foam = 0.0;
    foam = clamp(foam * (0.75 + 0.25 * nse), 0.0, 0.9);
    vec3 lit = vec3(uAmb + 0.7 * uSunI + 0.08 * uMoonI);
    vec3 foamC = vec3(0.86, 0.92, 0.94) * lit + lampSum * 0.9;
    // the tile's three tones on this water: the body as it is, a ripple lifted toward the sky it mirrors, a glint as foam
    vec3 ripC = mix(col, mix(uHor, uSky, 0.5) * lit + lampSum * 0.5, 0.35) * 1.25;
    col = mix(col, ripC, rip * tk * (1.0 - glint));
    col = mix(col, foamC, glint * tk * 0.7);
    col = mix(col, foamC, foam);
    alpha = mix(uMinA, 0.97, smoothstep(1.0, 18.0, depth / max(v.y, 0.25)));
    alpha = max(alpha, max(foam, fres));
    if (vFall > 0.5) alpha = 0.92;
    if (cut) alpha = 0.86;
    if (C.y < uCamWL) { float path = dist; col = col * exp(-uK * path) + uWCol * (1.0 - exp(-uKs * path)); }
    else col = mix(col, uHaze, smoothstep(uFogR.x, uFogR.y, dist));
  }
  gl_FragColor = vec4(col, alpha);
  #include <encodings_fragment>
}`});
  mat.extensions = {derivatives: false};
  return mat;
}

// the controller of one water material: the sky and sun it mirrors, and its clock
function waterCtl(W, mat, mesh, o) {
  const U = mat.uniforms;
  const S = {mesh, mat, U, amp: o.amp ?? 0.4, level: 0,
    setTime(T) { const sky = new THREE.Color(T.sky), hor = new THREE.Color(T.haze); U.uSky.value.copy(W.skyCol || sky).convertSRGBToLinear(); U.uHor.value.copy(hor).convertSRGBToLinear(); U.uSunI.value = T.sun * (W.opts.dim || 1) * (W.sunK ?? 1); U.uMoonI.value = T.moon * 25; U.uAmb.value = Math.max(0.012, T.hemi * 0.85 * (W.opts.dim || 1)); U.uSun.value.set(...T.sunPos).normalize(); },
    frame(t, cp) { U.uT.value = t; U.uAmp.value = S.amp; U.uLevel.value = S.level; if (S.follow) U.uCenter.value.set(Math.round(cp.x / 4) * 4, Math.round(cp.z / 4) * 4); },
    h: (x, z, t, eye) => S.level + seaH(x, z, t, S.amp, eye)};
  W.seas.push(S); return S;
}
// fresh water: a hex cap at each water cell's level, and a falling face down every step to a lower cell
const FALL_OFF = 0.08;
const PRE_ORDER = 3.5;
// the depth-only twin of a water material: the same shaders and uniforms (so the same waves and the same discards),
// no colour, depth written a hair behind so the colour pass's LessEqual test passes on the nearest surface only
function depthMat(mat) {
  if (mat.userData.depthMat) return mat.userData.depthMat;
  const d = new THREE.ShaderMaterial({uniforms: mat.uniforms, vertexShader: mat.vertexShader, fragmentShader: mat.fragmentShader, transparent: true, depthWrite: true, colorWrite: false,
    side: THREE.DoubleSide, polygonOffset: true, polygonOffsetFactor: 1, polygonOffsetUnits: 2});
  return (mat.userData.depthMat = d);
}   // metres a falling sheet stands off the cliff it falls down
function makeCaps(W, cells, o = {}) {
  const pos = [], flow = [], fall = [], top = [], push = (p, f, fa, tp = 0) => { pos.push(...p); flow.push(...f); fall.push(fa); top.push(tp); };
  for (const [key, c] of cells) {
    const [q, r] = key.split(",").map(Number), X = hx(q, r), Z = hz(r), f = c.flow || [0, 0];
    for (let i = 0; i < 6; i++) { const [ax, az] = CORNER[i], [bx, bz] = CORNER[(i + 1) % 6]; push([X, c.wl, Z], f, 0); push([X + bx, c.wl, Z + bz], f, 0); push([X + ax, c.wl, Z + az], f, 0); }
    for (let i = 0; i < 6; i++) {
      const nk = (q + DIRS[i][0]) + "," + (r + DIRS[i][1]), nb = cells.get(nk) || (o.lower && o.lower.get(nk)); if (o.noFalls || !nb || nb.wl >= c.wl - (o.minStep ?? 0.05)) continue;   // o.lower: water drawn by another mesh (the sea), so the last step still falls into it (owner, 2026-11-15: "last part of this water fall is missing")
      // The falling sheet stands FALL_OFF in front of the cell's edge, toward the lower cell: the cliff face (the column's
      // side) is on the edge itself, and a sheet on the same plane z-fights with it. The sheet runs from the lip of the
      // upper cap down a hair into the pool below, so no seam opens at either end.
      const [ax, az] = CORNER[i], [bx, bz] = CORNER[(i + 1) % 6], a = Math.PI / 3 * i, fd = [Math.cos(a), Math.sin(a)], ox = fd[0] * FALL_OFF, oz = fd[1] * FALL_OFF;
      const el = Math.hypot(bx - ax, bz - az), ex = (bx - ax) / el * FALL_OFF * 0.6, ez = (bz - az) / el * FALL_OFF * 0.6;   // reach past each corner, to meet the next sheet round a bend
      const P = [[X + ax + ox - ex, c.wl, Z + az + oz - ez], [X + bx + ox + ex, c.wl, Z + bz + oz + ez], [X + bx + ox + ex, nb.wl - 0.04, Z + bz + oz + ez], [X + ax + ox - ex, nb.wl - 0.04, Z + az + oz - ez]];
      for (const k of [0, 1, 2, 0, 2, 3]) push(P[k], fd, 1);
      // and a lip: the strip of cap between the edge and the sheet, so the water runs over the edge onto the sheet
      const L = [[X + ax, c.wl, Z + az], [X + bx, c.wl, Z + bz], [X + bx + ox, c.wl, Z + bz + oz], [X + ax + ox, c.wl, Z + az + oz]];
      for (const k of [0, 1, 2, 0, 2, 3]) push(L[k], f, 0);
    }
    // c.cuts: [side, bed] for each side where the patch ends under this water. The water stands there as a still wall
    // down to its bed (the ground's top), so the patch's edge shows the water's side above the ground's own wall
    // (owner, 2026-11-15: "The very front part of the cell doesn't have any geometry").
    for (const [i, bed] of c.cuts || []) {
      if (bed >= c.wl - 0.01) continue;
      const [ax, az] = CORNER[i], [bx, bz] = CORNER[(i + 1) % 6], a = Math.PI / 3 * i, fd = [Math.cos(a), Math.sin(a)];
      const P = [[X + ax, c.wl, Z + az], [X + bx, c.wl, Z + bz], [X + bx, bed, Z + bz], [X + ax, bed, Z + az]];
      for (const k of [0, 1, 2, 0, 2, 3]) push(P[k], fd, -1, c.wl);
    }
  }
  const g = new THREE.BufferGeometry(); g.setAttribute("position", new THREE.Float32BufferAttribute(pos, 3)); g.setAttribute("aFlow", new THREE.Float32BufferAttribute(flow, 2)); g.setAttribute("aFall", new THREE.Float32BufferAttribute(fall, 1)); g.setAttribute("aTop", new THREE.Float32BufferAttribute(top, 1));
  const mat = o.mat || waterMat(W, {amp: o.amp ?? 0.02, foam: 0.2, mud: o.mud, minA: o.minA ?? 0.74, tex: o.tex ?? 1});
  const mesh = new THREE.Mesh(g, mat); mesh.renderOrder = 4; W.scene.add(mesh);
  // Sorting (owner, 2026-11-15: "some zdepth issues with the waterfall/transparency sorting"): caps, lips, falls and cuts
  // are one transparent mesh, and with no depth written its triangles blended in whatever order they were built, so a
  // fall behind a cap could paint over it. A depth-only pass of the same mesh draws first (renderOrder 3.5, after every
  // opaque thing and before any water's colour), then the colour pass keeps only the nearest water at each pixel: water
  // behind water is hidden, as the game's single water layer is, from any angle and at any hour. The arrows (6) test
  // against that depth too, so a fall in front hides the arrow behind it.
  const pre = new THREE.Mesh(g, depthMat(mat)); pre.renderOrder = PRE_ORDER; pre.frustumCulled = mesh.frustumCulled; mesh.add(pre);
  if (!o.mat) { const S = waterCtl(W, mat, mesh, o); S.amp = o.amp ?? 0.02; mesh.userData.ctl = S; }
  return mesh;
}

// ======================= the sim's water: top surfaces, flow vectors, caps and arrows =======================
// The liquid rule's water drawn as the river's is (owner, 2026-11-15: "can we have the frothy walls and edges like the
// river demo you had and show flow direction?"): one cap per column at its top water surface, through makeCaps, so
// every step down to a lower column gets the river's foaming fall sheet and lip, and each cap carries a flow vector the
// streaks advect along: towards the neighbours whose water stands lower, by how much lower (Luanti's levels falling
// outward). A column falling straight down carries none. A flat chevron on each flowing cap points downstream, longer
// the faster it runs (the Flow arrows toggle).
// The water's top in a cell: a source or a falling cell stands nearly full, a flowing one at (level + 1) / 8 of its
// layer, and a cell with water above it is full to its top. So a source (0.94) is drawn 6 cm below the level-7 water
// beside it (1.0), though the water runs out of the source: the flow is worked out on the head, where a source stands
// an eighth above level 7, and no fall is drawn for a step under an eighth (minStep), so a pool of sources neither
// grows a 6 cm froth ring nor shows arrows running into it.
function surfOf(LQ) {
  const {AIR, SRC, FLOW} = root.PBDLiquid, {kind, lvl, down, idx, inside, unidx} = LQ;
  return i => { const [q, r, y] = unidx(i), up = inside(q, r, y + 1) ? kind[idx(q, r, y + 1)] : AIR; if (up === SRC || up === FLOW) return y + 1; return kind[i] === SRC || down[i] ? y + 0.94 : y + (lvl[i] + 1) / 8; };
}
const ARROW_MAX = 2400;
function simWater(W, LQ, o = {}) {
  const {SRC, FLOW, SOLID} = root.PBDLiquid, {kind, down, unidx, idx} = LQ, surf = surfOf(LQ);
  const SW = {mesh: null, mat: null, arrows: null, cells: null, showArrows: o.arrows !== false, surf};
  const M4 = new THREE.Matrix4(), Q = new THREE.Quaternion(), V = new THREE.Vector3(), SC = new THREE.Vector3(), UPA = new THREE.Vector3(0, 1, 0);
  SW.rebuild = () => {
    const tops = new Map();
    for (let i = 0; i < LQ.NCELL; i++) { const k = kind[i]; if (k !== SRC && k !== FLOW) continue; const [q, r, y] = unidx(i), key = q + "," + r, h = surf(i);
      const t = tops.get(key); if (!t || h > t.wl) tops.set(key, {q, r, y, wl: h, head: k === SRC && h < y + 1 ? y + 9 / 8 : h, fall: k === FLOW && !!down[i]}); }
    const cells = new Map();
    for (const [key, t] of tops) {
      const X = hx(t.q, t.r), Z = hz(t.r); let fx = 0, fz = 0;
      if (!t.fall) for (const [dq, dr] of DIRS) { const n = tops.get((t.q + dq) + "," + (t.r + dr)); if (!n || n.head >= t.head - 0.02) continue;
        const dx = hx(t.q + dq, t.r + dr) - X, dz = hz(t.r + dr) - Z, l = Math.hypot(dx, dz) || 1, w = Math.min(1, t.head - n.head); fx += dx / l * w; fz += dz / l * w; }
      const m = Math.hypot(fx, fz), sp = Math.min(0.9, m * 0.9);
      // where the patch ends beside this water, its side stands down to the bed (the first solid cell under it)
      let cuts = null;
      for (let i = 0; i < 6; i++) { if (LQ.inside(t.q + DIRS[i][0], t.r + DIRS[i][1], t.y)) continue;
        let y = t.y; while (y > LQ.Y0 && kind[idx(t.q, t.r, y - 1)] !== SOLID) y--; (cuts = cuts || []).push([i, y]); }
      cells.set(key, {wl: t.wl, y: t.y, head: t.head, flow: m > 1e-4 ? [fx / m * sp, fz / m * sp] : [0, 0], cuts});
    }
    if (SW.mesh) { W.scene.remove(SW.mesh); SW.mesh.geometry.dispose(); }
    SW.mesh = makeCaps(W, cells, Object.assign(SW.mat ? {mat: SW.mat} : Object.assign({amp: 0.01, mud: 0, minA: 0.78}, o.caps), {lower: o.lower, minStep: 0.07}));
    if (!SW.mat) SW.mat = SW.mesh.material;
    if (!SW.arrows) {
      const g = new THREE.BufferGeometry(); g.setAttribute("position", new THREE.Float32BufferAttribute([0.55, 0, 0, -0.35, 0, 0.42, -0.12, 0, 0, 0.55, 0, 0, -0.12, 0, 0, -0.35, 0, -0.42], 3));
      SW.arrows = new THREE.InstancedMesh(g, new THREE.MeshBasicMaterial({color: 0xf4fbff, transparent: true, opacity: 0.85, depthWrite: false, side: THREE.DoubleSide}), ARROW_MAX);
      SW.arrows.renderOrder = 6; SW.arrows.frustumCulled = false; W.scene.add(SW.arrows);
    }
    let na = 0;
    for (const [key, c] of cells) { const sp = Math.hypot(c.flow[0], c.flow[1]); if (sp < 0.05 || na >= ARROW_MAX) continue; const [q, r] = key.split(",").map(Number);
      Q.setFromAxisAngle(UPA, -Math.atan2(c.flow[1], c.flow[0])); V.set(hx(q, r), c.wl + 0.04, hz(r)); const k = 0.9 + 0.9 * sp; SC.set(k, 1, 1.2);
      M4.compose(V, Q, SC); SW.arrows.setMatrixAt(na++, M4); }
    SW.arrows.count = na; SW.arrows.instanceMatrix.needsUpdate = true; SW.arrows.visible = SW.showArrows;
    SW.cells = cells; return SW.mesh;
  };
  SW.setArrows = v => { SW.showArrows = !!v; if (SW.arrows) SW.arrows.visible = SW.showArrows; };
  return SW;
}

root.PBDWater = {WAVES, seaH, WAVE_GLSL, waterMat, waterCtl, makeCaps, FALL_OFF, uniforms, surfOf, simWater, hx, hz, CORNER, DIRS};
})(typeof window !== "undefined" ? window : globalThis);
