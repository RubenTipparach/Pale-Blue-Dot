// The waterfall mist and bubbles mockup (`water-flow` 6.5; CLAUDE.md, "Waterfalls are designed in a mockup first").
// The owner, 2026-11-16, on the splash drawn in front of the water: "design better mist instead of just a gradient
// circle puff, use blender/krita or whatever, look at crab breeze smoke puffs as a model", and "in the water there
// should be bubbles particles, this should give it that teal color".
// Built on waterfall/waterfall.js (M31): its world, its port of the game's water fragment (water.wgsl at dc0c5cb)
// and its lights, copied here with pools 3.35 m deep and the view from under the water added. Since the owner's
// "show me a comparison of different water fall heights" (2026-11-16) the scene is five falls side by side (a
// riffle and falls of 1, 2, 4 and 9 layers), and the New tab carries the proposed mist rule (every fall, however
// short, mists) and the fall's scroll speeding up down its height.
// The splash is a port of the game's `animate_splash` (water_flow.rs): the same drops and mist puffs, the same
// 32-texel dot. Needs three.js r128 (THREE) before it.
(function () {
"use strict";
const HW = 2.833, HR = HW / Math.sqrt(3);
const DIRS = [[1, 0], [0, 1], [-1, 1], [-1, 0], [0, -1], [1, -1]];     // side i faces 60 i degrees
const cx = (q, r) => HW * (q + r / 2), cz = r => 1.5 * HR * r;
const cornerXZ = (q, r, i) => { const a = Math.PI / 180 * (60 * i - 30); return [cx(q, r) + HR * Math.cos(a), cz(r) + HR * Math.sin(a)]; };
const sideN = i => [Math.cos(Math.PI / 3 * i), Math.sin(Math.PI / 3 * i)];
const key = (q, r) => q + "," + r;
const hexDist = (q, r) => (Math.abs(q) + Math.abs(r) + Math.abs(q + r)) / 2;

// ---------------- the world: cells and their tops ----------------
// The height comparison (owner, 2026-11-16: "show me a comparison of different water fall heights"): five falls side
// by side along z, each off the same plateau's lip (ground 0, x >= 1.5 cells), so one view frames them all.
//   riffle  a river stepping down one layer: the source spills into a falling cell over a channel one layer lower,
//           whose shallow water (0.7 m) runs on toward the eye;
//   f1..f9  a fall of N layers into a pool 3.35 m deep, dug into a shore N - 1 layers under the plateau (open to the
//           front, so the side view sees every fall's whole face). f9 is the scene the owner tuned on (a pit two
//           rings wide); the others are one ring wide.
// N is the game's h: the falling layers over the foot. rc is the fall's row (z); its basin is centred on x = 0.
const SPECS = [
  {name: "riffle", label: "Riffle", N: 1, rc: -10, rad: 0},
  {name: "f1", label: "1 layer", N: 1, rc: -6, rad: 1},
  {name: "f2", label: "2 layers", N: 2, rc: -2, rad: 1},
  {name: "f4", label: "4 layers", N: 4, rc: 2, rad: 1},
  {name: "f9", label: "9 layers", N: 9, rc: 8, rad: 2},
];
const GRID = 22, GRID_LOOK = GRID, DEEP_M = -16;
const X0 = -14 * HW, X1 = 4.6 * HW, Z0 = cz(-13.2), Z1 = cz(12.2), PLATEAU_X = 1.5 * HW - 1e-6;
const BASIN = new Map();                       // a basin or channel cell -> its fall
// S the source's cell (on the plateau), R the falling cell over the drop.
const FALLS = SPECS.map(sp => {
  const cq = -sp.rc / 2, k = Math.max(sp.rad, 1), riffle = sp.rad === 0;
  const F = {name: sp.name, label: sp.label, N: sp.N, riffle, rad: sp.rad, c: [cq, sp.rc],
    S: [cq + k + 1, sp.rc], R: [cq + k, sp.rc], y0: 0, levelS: 0.9, hR: 0.75,
    floor: riffle ? -1 : -sp.N - 3, poolY: riffle ? -0.3 : -sp.N + 0.35, shore: riffle ? 0 : 1 - sp.N, pool: []};
  if (riffle) for (let j = 1; j >= -4; j--) F.pool.push([cq + j, sp.rc]);
  else for (let q = -sp.rad; q <= sp.rad; q++) for (let r = -sp.rad; r <= sp.rad; r++) if (hexDist(q, r) <= sp.rad) F.pool.push([cq + q, sp.rc + r]);
  for (const [q, r] of F.pool) BASIN.set(key(q, r), F);
  return F;
});
// The fall whose row is nearest: its shore is the lower ground in front of the plateau there.
const segOf = z => FALLS.reduce((a, b) => Math.abs(cz(b.c[1]) - z) < Math.abs(cz(a.c[1]) - z) ? b : a);
const topLook = (q, r) => top(q, r);
function top(q, r) {
  const x = cx(q, r), z = cz(r);
  if (x < X0 || x > X1 || z < Z0 || z > Z1) return null;
  const B = BASIN.get(key(q, r)); if (B) return B.floor;
  if (x >= PLATEAU_X) return 0;
  return segOf(z).shore;
}
// The ground's rim: the height a cell's own sky is open from (its fall's shore in a basin, else its top).
function rimOf(q, r) { const B = BASIN.get(key(q, r)); return B ? B.shore : top(q, r); }
for (const F of FALLS) {
  const [sq, sr] = F.S, [rq, rr] = F.R;
  F.iSR = DIRS.findIndex(([dq, dr]) => sq + dq === rq && sr + dr === rr);
  F.iRS = (F.iSR + 3) % 6;
  F.flow = sideN(F.iSR);                      // the way the water goes over: from S into R
  F.cR = [cx(rq, rr), cz(rr)];
  F.cS = [cx(sq, sr), cz(sr)];
  F.H = F.hR - F.poolY;                       // the fall's height, metres
  F.drop = F.y0 - F.poolY;                    // from the lip's ground to the pool
  F.poolSet = new Set(F.pool.map(([q, r]) => key(q, r)));
  F.cC = [cx(F.c[0], F.c[1]), cz(F.c[1])];
}
const poolAt = (q, r) => { const B = BASIN.get(key(q, r)); return B ? B.poolY : null; };
// The hex a point is over (cube rounding), for the eye's pool.
function hexAt(x, z) {
  const r = z / (1.5 * HR), q = x / HW - r / 2, s = -q - r;
  let rq = Math.round(q), rr = Math.round(r); const rs = Math.round(s);
  const dq = Math.abs(rq - q), dr = Math.abs(rr - r), ds = Math.abs(rs - s);
  if (dq > dr && dq > ds) rq = -rr - rs; else if (dr > ds) rr = -rq - rs;
  return [rq, rr];
}
// The lamps (night): one on the plateau by each fall's lip, one on its shore in front, left of the pool.
const LAMP_SPOTS = [];
for (const F of FALLS) {
  const [cq, cr] = F.c, k = Math.max(F.rad, 1);
  for (const [q, r] of [[F.S[0], F.S[1] - 1], [cq - k, cr - 1]]) LAMP_SPOTS.push([cx(q, r), top(q, r), cz(r)]);
}
const NL = LAMP_SPOTS.length, NF = FALLS.length;

// ---------------- GLSL: the game's noise, ported ----------------
const NOISE_GLSL = `
vec3 gn_fade(vec3 t) { return t*t*t*(t*(t*6.0-15.0)+10.0); }
uint gn_hash(ivec3 p) {
  uvec3 q = uvec3(p) + uvec3(1073741824u);
  uint h = (q.x*374761393u) ^ (q.y*668265263u) ^ (q.z*1274126177u);
  h = (h ^ (h>>13u))*1103515245u;
  return h ^ (h>>16u);
}
float gn_grad(uint hash, vec3 p) {
  uint h = hash & 15u;
  float u = h < 8u ? p.x : p.y;
  float v = h < 4u ? p.y : ((h == 12u || h == 14u) ? p.x : p.z);
  return ((h & 1u) != 0u ? -u : u) + ((h & 2u) != 0u ? -v : v);
}
float gnoise3(vec3 p) {
  ivec3 lattice = ivec3(floor(p));
  vec3 f = fract(p);
  vec3 s = gn_fade(f);
  float v[8];
  for (int i = 0; i < 8; i++) {
    ivec3 corner = ivec3(i & 1, (i >> 1) & 1, (i >> 2) & 1);
    v[i] = gn_grad(gn_hash(lattice + corner), f - vec3(corner));
  }
  return mix(mix(mix(v[0], v[1], s.x), mix(v[2], v[3], s.x), s.y),
             mix(mix(v[4], v[5], s.x), mix(v[6], v[7], s.x), s.y), s.z);
}
float fbm3(vec3 p, float t) {
  vec3 q = p*0.9 + vec3(0.35, 0.18, -0.42)*t;
  float h = gnoise3(q)*0.5;
  q = q*2.1 + vec3(-0.22, 0.33, 0.17)*t;
  h += gnoise3(q)*0.28;
  q = q*2.3 + vec3(0.19, -0.27, 0.11)*t;
  return h + gnoise3(q)*0.15;
}`;

// Lights shared by the ground and the water: a sun (day), a moon, the sky's ambient and the lamps (night).
const LIGHT_GLSL = `
uniform vec3 uSunDir; uniform vec3 uSunCol; uniform vec3 uAmb;
uniform vec3 uLampPos[${NL}]; uniform vec3 uLampCol[${NL}]; uniform float uLampOn;
vec3 lampLight(vec3 p, vec3 n, float wrap) {
  vec3 e = vec3(0.0);
  for (int i = 0; i < ${NL}; i++) {
    vec3 l = uLampPos[i] - p; float d = length(l); l /= max(d, 1e-4);
    float fall = pow(clamp(1.0 - pow(d/16.0, 4.0), 0.0, 1.0), 2.0)/(1.0 + 0.35*d*d);
    e += uLampCol[i]*fall*clamp((dot(n, l) + wrap)/(1.0 + wrap), 0.0, 1.0);
  }
  return e*uLampOn;
}
vec3 skyRad(vec3 d, float night) {
  float h = clamp(d.y, -0.2, 1.0);
  vec3 day = mix(vec3(0.55, 0.66, 0.78), vec3(0.20, 0.38, 0.72), smoothstep(0.0, 0.7, h));
  vec3 nite = mix(vec3(0.010, 0.014, 0.026), vec3(0.003, 0.005, 0.014), smoothstep(0.0, 0.7, h));
  return mix(day, nite, night);
}
vec3 toSrgb(vec3 c) { c = c/(1.0 + 0.08*c); return pow(clamp(c, 0.0, 1.0), vec3(1.0/2.2)); }
vec3 fromSrgb(vec3 c) { c = pow(c, vec3(2.2)); return c/max(1.0 - 0.08*c, 0.05); }`;


// Under the water (the game's compose and the water's 'below' path): the murk, and the extinction per metre.
const UNDER_GLSL = `uniform float uEyeUnder; uniform vec3 uMurk; uniform vec3 uExt;`;

// ---------------- the ground ----------------
function sandTexture() {
  // 16 x 16, the game's look: a warm sand with darker orange squares, sampled nearest.
  const c = document.createElement("canvas"); c.width = c.height = 16; const g = c.getContext("2d");
  g.fillStyle = "#d6a44a"; g.fillRect(0, 0, 16, 16);
  let s = 7; const rnd = () => (s = (s * 16807) % 2147483647) / 2147483647;
  for (let i = 0; i < 26; i++) { g.fillStyle = rnd() < 0.5 ? "#c98a34" : "#dcb05a"; const x = Math.floor(rnd() * 16), y = Math.floor(rnd() * 16); g.fillRect(x, y, 1 + (rnd() < 0.3), 1 + (rnd() < 0.3)); }
  for (let i = 0; i < 5; i++) { g.fillStyle = "#c27e2c"; g.fillRect(Math.floor(rnd() * 14), Math.floor(rnd() * 14), 2, 2); }
  const t = new THREE.CanvasTexture(c); t.magFilter = t.minFilter = THREE.NearestFilter; t.wrapS = t.wrapT = THREE.RepeatWrapping; t.generateMipmaps = false;
  return t;
}
// o (optional, the Pour tab's): {top(q, r), grid, deep}; by default the Look tab's pit scene.
function groundMesh(U, o) {
  const top = (o && o.top) || topLook, GRID = (o && o.grid) || GRID_LOOK, DEEP = (o && o.deep) || DEEP_M;
  const pos = [], nor = [], uvw = [], rim = [];
  let rimNow = 0;
  const tri = (a, b, c, n, uv) => { pos.push(...a, ...b, ...c); for (let k = 0; k < 3; k++) { nor.push(...n); rim.push(rimNow); } uvw.push(...uv); };
  for (let q = -GRID; q <= GRID; q++) for (let r = -GRID; r <= GRID; r++) {
    const t = top(q, r); if (t === null) continue;
    const c = [cx(q, r), t, cz(r)];
    rimNow = rimOf(q, r);
    for (let i = 0; i < 6; i++) {
      const a = cornerXZ(q, r, i), b = cornerXZ(q, r, (i + 1) % 6);
      tri(c, [b[0], t, b[1]], [a[0], t, a[1]], [0, 1, 0], [c[0], c[2], 0, b[0], b[1], 0, a[0], a[1], 0]);
      const [nq, nr] = [q + DIRS[i][0], r + DIRS[i][1]];
      let nt = top(nq, nr); if (nt === null) nt = DEEP;
      if (nt < t) {
        const n = sideN(i), A = [a[0], t, a[1]], B = [b[0], t, b[1]], A0 = [a[0], nt, a[1]], B0 = [b[0], nt, b[1]];
        const ua = a[0] * -n[1] + a[1] * n[0], ub = b[0] * -n[1] + b[1] * n[0];
        tri(A0, B0, B, [n[0], 0, n[1]], [ua, nt, 1, ub, nt, 1, ub, t, 1]);
        tri(A0, B, A, [n[0], 0, n[1]], [ua, nt, 1, ub, t, 1, ua, t, 1]);
      }
    }
  }
  const g = new THREE.BufferGeometry();
  g.setAttribute("position", new THREE.Float32BufferAttribute(pos, 3));
  g.setAttribute("normal", new THREE.Float32BufferAttribute(nor, 3));
  g.setAttribute("aUV", new THREE.Float32BufferAttribute(uvw, 3));
  g.setAttribute("aRim", new THREE.Float32BufferAttribute(rim, 1));
  // The cells' tops as a texture, for the sun's shadow: a march toward the sun over the hex height field.
  const N = 2 * GRID + 1, tops = new Float32Array(N * N * 4);
  for (let q = -GRID; q <= GRID; q++) for (let r = -GRID; r <= GRID; r++) { const t = top(q, r); tops[4 * ((r + GRID) * N + (q + GRID))] = t === null ? DEEP : t; }
  const topTex = new THREE.DataTexture(tops, N, N, THREE.RGBAFormat, THREE.FloatType);
  topTex.minFilter = topTex.magFilter = THREE.NearestFilter; topTex.generateMipmaps = false; topTex.needsUpdate = true;
  const mat = new THREE.ShaderMaterial({uniforms: Object.assign({uSand: {value: sandTexture()}, uTops: {value: topTex}}, U), side: THREE.DoubleSide,
    vertexShader: `attribute vec3 aUV; attribute float aRim; varying vec3 vUV; varying vec3 vN; varying vec3 vP; varying float vRim;
      void main() { vUV = aUV; vN = normal; vP = position; vRim = aRim; gl_Position = projectionMatrix*viewMatrix*vec4(position, 1.0); }`,
    fragmentShader: `uniform sampler2D uSand; uniform sampler2D uTops; varying vec3 vUV; varying vec3 vN; varying vec3 vP; varying float vRim; ${LIGHT_GLSL} ${UNDER_GLSL}
      float topAt(vec2 xz) {
        float r = xz.y/${(1.5 * HR).toFixed(5)}, q = xz.x/${HW.toFixed(4)} - r*0.5, s = -q - r;
        float rq = floor(q + 0.5), rr = floor(r + 0.5), rs = floor(s + 0.5);
        float dq = abs(rq - q), dr = abs(rr - r), ds = abs(rs - s);
        if (dq > dr && dq > ds) rq = -rr - rs; else if (dr > ds) rr = -rq - rs;
        if (abs(rq) > ${GRID}.0 || abs(rr) > ${GRID}.0) return ${DEEP.toFixed(1)};
        return texture2D(uTops, (vec2(rq, rr) + ${GRID}.5)/${N}.0).r;
      }
      float sunShadow(vec3 p, vec3 n) {
        vec3 o = p + n*0.03 + uSunDir*0.03;
        for (int i = 1; i <= 48; i++) {
          vec3 q = o + uSunDir*(float(i)*0.28);
          if (q.y > 1.0) return 1.0;
          if (q.y < topAt(q.xz) - 0.001) return 0.0;
        }
        return 1.0;
      }
      void main() {
        vec3 tex = texture2D(uSand, vUV.xy/2.0).rgb;
        vec3 alb = pow(tex, vec3(2.2));
        // Deeper than 2.5 m under the rim the walls are dirt, the layers marked as the game's blocks are.
        if (vUV.z > 0.5 && vP.y < vRim - 2.5) alb = pow(tex*vec3(0.62, 0.50, 0.42), vec3(2.2));
        // The sky's share: less down a pit or under a cliff, by the depth under the cell's own rim.
        float sky = vP.y < vRim - 0.01 ? mix(0.42, 1.0, smoothstep(vRim - 9.0, vRim, vP.y)) : 1.0;
        vec3 n = normalize(vN);
        float lit = max(dot(n, uSunDir), 0.0);
        if (lit > 0.0) lit *= sunShadow(vP, n);
        vec3 light = uSunCol*lit + uAmb*sky + lampLight(vP, n, 0.0);
        vec3 col = alb*light;
        if (uEyeUnder > 0.5) col = mix(uMurk, col, exp(-uExt*length(vP - cameraPosition)));
        gl_FragColor = vec4(toSrgb(col), 1.0);
      }`});
  return new THREE.Mesh(g, mat);
}

// ---------------- the water: caps, pools and the falling faces ----------------
// Attributes, beside position (bent for the lean):
//   aRest   the rest position (where the mesher puts the vertex); the froth is evaluated there, so C/D's lean
//           bends the same pattern B draws, never slicing a new one;
//   aFaceN  the face's own normal (the mesher's): the tangent frame, rain's "faces up" test and the back-face rule;
//   aKind   0 a cap or pool, 2 a falling face, 3 a still face (the game's kinds);
//   aFroth  the mesher's froth weight on a falling face (flow_uv.x: 1 at the lip and foot layers, 0.55 between);
//           on a cap, how squarely the flow crosses the falling cell (D's lip froth fades to 0 at grazing rays);
//   aPath   (x, z, s): s metres along the flow from the lip, down a face (> 0) or still on the top (< 0), and
//           x, z where that path goes over (the face itself, for a face): D's noise runs on it, so the froth on
//           the top and on the face are one pattern across the edge;
//   aTop    the lip's height; aFlowN the fall's main face normal (D's one frame per fall); aFlow a cap's flow, m/s;
//   aSky    how open to the sky (the pit's walls shade its foot).
// Rounded lip (New tab, Lip: Glassy; owner, 2026-11-16: "still looks kinda sharp there"): the falling cell's top
// edge, on every side but the one it is fed from, is a quarter-round of this radius. The top bends down over the
// arc (its vertices within LIP_R of the rim are moved onto the arc, with the arc's normal), and the sides start
// LIP_R below the top, where the arc ends. 0 is the game's square edge.
let LIP_R = 0;
function waterGeometry(lean) {
  const A = {position: [], aRest: [], aFaceN: [], aKind: [], aFroth: [], aPath: [], aTop: [], aFlowN: [], aFlow: [], aSky: [], aDepth: []};
  let depthNow = 0.35;
  const prof = u => { const e = 0.05; return (Math.sqrt(Math.max(u, 0) + e) - Math.sqrt(e)) / (Math.sqrt(1 + e) - Math.sqrt(e)); };
  function vert(F, rest, faceN, kind, froth, path, sky, flow) {
    let p = rest;
    if (F && kind === 2 && rest[1] < F.y0 - 1e-4 && lean > 0) {
      const off = lean * prof((F.y0 - rest[1]) / F.drop);
      p = [rest[0] + F.flow[0] * off, rest[1], rest[2] + F.flow[1] * off];
    }
    A.position.push(...p); A.aRest.push(...rest); A.aFaceN.push(...faceN); A.aKind.push(kind); A.aFroth.push(froth);
    A.aPath.push(...path); A.aTop.push(F ? F.hR : 0); A.aFlowN.push(F ? F.flow[0] : 1, 0, F ? F.flow[1] : 0);
    A.aFlow.push(...(flow || [0, 0, 0])); A.aSky.push(sky); A.aDepth.push(depthNow);
  }
  // D's path on a top: the ray from the source's centre through the point, to where it leaves the falling cell.
  function lipPath(F, x, z) {
    const [px, pz] = F.cS; let dx = x - px, dz = z - pz; const L = Math.hypot(dx, dz);
    if (L < 1e-4) { dx = F.flow[0]; dz = F.flow[1]; } else { dx /= L; dz /= L; }
    let tIn = -1e9, tOut = 1e9;
    for (let i = 0; i < 6; i++) {
      const n = sideN(i), den = dx * n[0] + dz * n[1], o = (px - F.cR[0]) * n[0] + (pz - F.cR[1]) * n[1];
      const t = (HW / 2 - o) / (Math.abs(den) < 1e-6 ? 1e-6 : den);
      if (den < 0) tIn = Math.max(tIn, t); else tOut = Math.min(tOut, t);
    }
    if (tIn > tOut || dx * F.flow[0] + dz * F.flow[1] < 0) return {path: [x, z, -99], fade: 0};
    return {path: [px + dx * tOut, pz + dz * tOut, Math.min(L - tOut, 0)], fade: Math.min(1, Math.max(0, (tOut - tIn - 0.1) / 0.6))};
  }
  // A hex cap, its six triangles each cut into n x n, heights from corner heights (flat when one height).
  function cap(F, q, r, hs, n, kind, sky, flow, lip, round) {
    const c = [cx(q, r), cz(r)], hc = hs.reduce((a, b) => a + b, 0) / 6;
    for (let i = 0; i < 6; i++) {
      const a = cornerXZ(q, r, i), b = cornerXZ(q, r, (i + 1) % 6), ha = hs[i], hb = hs[(i + 1) % 6];
      const P = (u, v) => { const w = 1 - u - v; return [c[0] * w + a[0] * u + b[0] * v, hc * w + ha * u + hb * v, c[1] * w + a[1] * u + b[1] * v]; };
      const emit = p0 => {
        let p = p0, nrm = [0, 1, 0];
        if (round && LIP_R > 0) {
          // Distance in from the rim (the nearest side but the fed one), and that side's normal.
          let rim = 1e9, ni = 0;
          for (let s2 = 0; s2 < 6; s2++) { if (s2 === F.iRS) continue; const n2 = sideN(s2), d2 = HW / 2 - ((p0[0] - c[0]) * n2[0] + (p0[2] - c[1]) * n2[1]); if (d2 < rim) { rim = d2; ni = s2; } }
          if (rim < LIP_R) {
            const th = (1 - Math.max(rim, 0) / LIP_R) * Math.PI / 2, inward = LIP_R * (1 - Math.sin(th));
            const n2 = sideN(ni), push = rim - inward;   // move out along the side's normal by rim - inward
            p = [p0[0] + n2[0] * push, p0[1] - LIP_R * (1 - Math.cos(th)), p0[2] + n2[1] * push];
            nrm = [n2[0] * Math.sin(th), Math.cos(th), n2[1] * Math.sin(th)];
          }
        }
        const L = lip ? lipPath(F, p0[0], p0[2]) : {path: [p0[0], p0[2], -99], fade: 0}; vert(F, p, nrm, kind, L.fade, L.path, sky, flow);
      };
      for (let j = 0; j < n; j++) for (let k = 0; k < n - j; k++) {
        const p00 = P(j / n, k / n), p10 = P((j + 1) / n, k / n), p01 = P(j / n, (k + 1) / n);
        emit(p00); emit(p01); emit(p10);
        if (k < n - j - 1) { const p11 = P((j + 1) / n, (k + 1) / n); emit(p10); emit(p01); emit(p11); }
      }
    }
  }
  // A vertical face between two corners, from y1 down to y0, in rows (so the lean bends it smoothly).
  function face(F, a, b, n, yTop, yBot, kind, froth, sky) {
    const rows = Math.max(1, Math.ceil((yTop - yBot) / 0.2));
    for (let k = 0; k < rows; k++) {
      const ya = yTop - (yTop - yBot) * k / rows, yb = yTop - (yTop - yBot) * (k + 1) / rows;
      const V = (p, y) => { const rest = [p[0], y, p[1]]; const s = kind === 2 ? F.hR - y : -99; vert(F, rest, [n[0], 0, n[1]], kind, froth, [p[0], p[1], s], sky(y)); };
      V(a, yb); V(b, yb); V(b, ya); V(a, yb); V(b, ya); V(a, ya);
    }
  }
  for (const F of FALLS) {
    const skyAt = y => y < F.y0 ? mix(0.62, 1.0, smooth(F.floor, Math.min(F.y0, F.shore + 1), y)) : 1.0;
    const [sq, sr] = F.S, [rq, rr] = F.R;
    const flowW = [F.flow[0] * 0.8, 0, F.flow[1] * 0.8];
    // The source's top, sloping down to the falling cell's.
    const hsS = Array.from({length: 6}, (_, i) => (i === F.iSR || i === (F.iSR + 1) % 6) ? F.hR : F.levelS);
    cap(F, sq, sr, hsS, 10, 0, 1, flowW, true);
    // The falling cell's top: the lip; it shares its rim's vertices with the faces under it.
    cap(F, rq, rr, Array(6).fill(F.hR), LIP_R > 0 ? 16 : 10, 0, 1, flowW, true, true);
    // The source's still sides over the ground (kind 3).
    for (let i = 0; i < 6; i++) {
      if (i === F.iSR) continue;
      const a = cornerXZ(sq, sr, i), b = cornerXZ(sq, sr, (i + 1) % 6);
      face(F, a, b, sideN(i), Math.min(hsS[i], hsS[(i + 1) % 6]), F.y0, 3, 0.5, () => 1);
    }
    // The falling cell's sides: over the ground they stand from it to the lip; over the drop they fall to the pool,
    // a quad per layer as the mesher makes them (froth 1 on the lip's layer and the foot's, 0.55 between).
    for (let i = 0; i < 6; i++) {
      if (i === F.iRS) continue;
      const nq = rq + DIRS[i][0], nr = rr + DIRS[i][1];
      let nt = top(nq, nr); if (nt === null) nt = DEEP_M;
      const pl = poolAt(nq, nr); const bottom = pl !== null ? Math.max(nt, pl) : nt;
      const a = cornerXZ(rq, rr, i), b = cornerXZ(rq, rr, (i + 1) % 6), n = sideN(i);
      if (bottom >= F.y0 - 0.01) { face(F, a, b, n, F.hR - LIP_R, F.y0, 2, 1.0, skyAt); continue; }
      const lipLayer = Math.floor(F.y0), footLayer = Math.floor(bottom + 1e-3);
      for (let L = lipLayer; L >= footLayer; L--) {
        const y1 = Math.min(L + 1, F.hR - LIP_R), y0 = Math.max(L, bottom);
        if (y1 - y0 < 1e-3) continue;
        face(F, a, b, n, y1, y0, 2, (L === lipLayer || L === footLayer) ? 1.0 : 0.55, skyAt);
      }
    }
    // The pool at the foot.
    for (const [q, r] of F.pool) {
      depthNow = F.poolY - top(q, r);
      // A riffle's lower water runs on, away from the step (its waves stream as a flowing top's do).
      cap(null, q, r, Array(6).fill(F.poolY), 8, 0, F.riffle ? 0.9 : 0.8, F.riffle ? flowW : null, false);
      depthNow = 0.35;
      for (let i = 0; i < 6; i++) {
        const nq = q + DIRS[i][0], nr = r + DIRS[i][1];
        if (F.poolSet.has(key(nq, nr))) continue;
        const nt = top(nq, nr);
        if (nt === null || nt >= F.poolY) continue;
        face(null, cornerXZ(q, r, i), cornerXZ(q, r, (i + 1) % 6), sideN(i), F.poolY, nt, 3, 0.5, () => 0.9);
      }
    }
  }
  const g = new THREE.BufferGeometry();
  g.setAttribute("position", new THREE.Float32BufferAttribute(A.position, 3));
  for (const [k, n] of [["aRest", 3], ["aFaceN", 3], ["aKind", 1], ["aFroth", 1], ["aPath", 3], ["aTop", 1], ["aFlowN", 3], ["aFlow", 3], ["aSky", 1], ["aDepth", 1]])
    g.setAttribute(k, new THREE.Float32BufferAttribute(A[k], n));
  return g;
}
function mix(a, b, t) { return a + (b - a) * t; }
function smooth(a, b, x) { const t = Math.min(1, Math.max(0, (x - a) / (b - a))); return t * t * (3 - 2 * t); }

// The game's water fragment, for the game's own water (the cap and its kind): water.wgsl dc0c5cb `fragment`, cut to
// what a fall and a pool use. The scene behind is the opaque pass copied before the water draws, as the game's
// scene_color is. Body frame: uM turns the page's local frame (y up) into a planet's at the capture's latitude.
function waterMaterial(U, vertexShader) {
  const m = new THREE.ShaderMaterial({
    uniforms: Object.assign({
      uM: {value: new THREE.Matrix3()}, uRp: {value: 2400}, uTime: {value: 0}, uScene: {value: null}, uRes: {value: new THREE.Vector2(1, 1)},
      uStretch: {value: 1}, uSpeed: {value: 1}, uAccel: {value: 0}, uLip: {value: 0}, uFallFrame: {value: 0},
      uOpen: {value: new THREE.Vector3(1, 1, 1)}, uNight: {value: 0}, uRain: {value: 0}, uSunI: {value: 1},
      uFeet: {value: FALLS.map(() => new THREE.Vector3())}, uFeetH: {value: FALLS.map(() => 9)}, uAer: {value: 0}, uAerAmt: {value: 1}, uRip: {value: 0}, uOut: {value: 0}, uGlass: {value: 0},
      uG: {value: 0}, uVmax: {value: 8}}, U),
    side: THREE.DoubleSide,
    vertexShader: `attribute vec3 aRest; attribute vec3 aFaceN; attribute float aKind; attribute float aFroth; attribute vec3 aPath;
      attribute float aTop; attribute vec3 aFlowN; attribute vec3 aFlow; attribute float aSky; attribute float aDepth; varying float vDepth;
      varying vec3 vRest; varying vec3 vFaceN; varying float vKind; varying float vFroth; varying vec3 vPath; varying float vTop;
      varying vec3 vFlowN; varying vec3 vFlow; varying float vSky; varying vec3 vP;
      void main() { vRest = aRest; vFaceN = aFaceN; vKind = aKind; vFroth = aFroth; vPath = aPath; vTop = aTop; vFlowN = aFlowN;
        vFlow = aFlow; vSky = aSky; vDepth = aDepth; vP = position; gl_Position = projectionMatrix*viewMatrix*vec4(position, 1.0); }`,
    // (the Pour tab passes its own vertex shader: the same varyings, eased between two ticks)
    fragmentShader: `uniform mat3 uM; uniform float uRp; uniform float uTime; uniform sampler2D uScene; uniform vec2 uRes;
      uniform float uStretch; uniform float uSpeed; uniform float uAccel; uniform float uLip; uniform float uFallFrame;
      uniform vec3 uOpen; uniform float uNight; uniform float uRain; uniform float uSunI;
      varying vec3 vRest; varying vec3 vFaceN; varying float vKind; varying float vFroth; varying vec3 vPath; varying float vTop;
      varying vec3 vFlowN; varying vec3 vFlow; varying float vSky; varying vec3 vP; varying float vDepth;
      uniform vec3 uFeet[${NF}]; uniform float uFeetH[${NF}]; uniform float uAer; uniform float uAerAmt; uniform float uRip; uniform float uOut; uniform float uGlass;
      uniform float uG; uniform float uVmax;
      ${NOISE_GLSL}
      ${LIGHT_GLSL}
      ${UNDER_GLSL}
      // The game's constants (assets/config/water.ron at dc0c5cb).
      const float FALL_SPEED = 1.0;            // flow_uv_speed_falling, m/s (view.deep_color.w)
      const float RIPPLE_SCALE = 1.5, RIPPLE_SPEED = 1.0, WAVE_STEEP = 0.45, SLOPE_MAX = 0.7, DETAIL_FADE = 4.0;
      const vec3 DEEP = vec3(0.004, 0.018, 0.040), SHALLOW = vec3(0.020, 0.090, 0.075);
      const float SKY_HORIZON = 0.7, SHALLOW_DEPTH = 14.0, FOAM_INT = 0.10, SPEC_I = 0.06, SPEC_P = 140.0;
      const vec3 ABSORB = vec3(0.32, 0.035, 0.012), FOAM_COL = vec3(0.95, 0.97, 1.00), SUN_TINT = vec3(1.10, 1.15, 1.20);
      const float TURBID = 0.07;               // the coast's turbidity, view.absorption.w
      const vec3 AER_COL = vec3(0.16, 0.50, 0.46);  // mockup-only: aerated water, white bubbles under the teal
      vec3 tangentU(vec3 face) { vec3 t = abs(face.x) > 0.9 ? vec3(0.0, 1.0, 0.0) : vec3(1.0, 0.0, 0.0); return normalize(t - face*dot(t, face)); }
      // The path coordinate, metres: identity unless D's speed-up warps it. Down the face the water speeds up from
      // v0 = 1 m/s under g = 3 m/s^2 (gentle, not free fall): c is v0 times the time it takes to fall s metres, so
      // the pattern's features stretch as the water speeds up. On the top it slows away from the lip.
      float pathC(float s) {
        if (s >= 0.0) { float g = 3.0; float T = (sqrt(1.0 + 2.0*g*s) - 1.0)/g; return mix(s, T, uAccel); }
        float k = 1.2*uAccel; float u = max(s, -3.0);
        return u - k*u*u*0.5 + (s - u)*(1.0 + 3.0*k);
      }
      // The water's speed down a fall (New tab; owner, 2026-11-16: "can you make the side scrolling texture increase
      // with the height of the fall?"): s metres under the lip it is v(s) = min(sqrt(v0^2 + 2 g s), vmax). A pattern
      // scrolled at v0 and read at c(s) = v0 tau(s), tau(s) the time the water takes to fall s metres, has its
      // features move at v(s): one continuous pattern, steady in time, stretched as the water speeds up. g 0 is
      // c(s) = s, the game's constant scroll. (Reading the pattern at s - t v(s) instead would shear: neighbouring
      // heights slide apart a little more every second.)
      float fallC(float s, float v0) {
        if (uG <= 0.0 || s <= 0.0) return s;
        float vm = max(uVmax, v0 + 1e-3), sc = (vm*vm - v0*v0)/(2.0*uG);
        float tau = s <= sc ? (sqrt(v0*v0 + 2.0*uG*s) - v0)/uG : (vm - v0)/uG + (s - sc)/vm;
        return v0*tau;
      }
      vec2 rings(vec2 uv, float t) {
        vec2 g = vec2(0.0); vec2 cell = floor(uv);
        for (int i = -1; i <= 1; i++) for (int j = -1; j <= 1; j++) {
          vec2 c = cell + vec2(float(i), float(j));
          vec2 h = fract(sin(vec2(dot(c, vec2(127.1, 311.7)), dot(c, vec2(269.5, 183.3))))*43758.5453);
          float ph = fract(t*0.9 + h.x); vec2 d = uv - (c + 0.2 + 0.6*h); float r = length(d); float R = ph*0.85;
          float w = exp(-pow((r - R)/0.07, 2.0))*(1.0 - ph);
          g += d/max(r, 1e-4)*w*sin((r - R)*40.0);
        }
        return g;
      }
      void main() {
        vec3 radial = uM*vec3(0.0, 1.0, 0.0);
        vec3 face = normalize(uM*vFaceN);
        float horizontal = abs(dot(face, radial));
        vec3 camB = uM*cameraPosition, posB = uM*vP;
        // The game's back-face rule: a wall's back seen from dry air is not drawn.
        if (vKind > 0.5 && dot(uM*(cameraPosition - vP), face) < 0.0) discard;
        vec3 tu = tangentU(face), tv = cross(face, tu);
        float falls = vKind > 2.5 ? 0.0 : 1.0;
        bool fallFace = vKind > 1.5 && vKind < 2.5 && horizontal < 0.5;
        // The waves on a falling face, scrolled at the game's 1 m/s at the lip; with the speed-up, read at c(s).
        vec3 rest = vRest;
        if (fallFace && uG > 0.0) rest.y = vTop - fallC(vPath.z, FALL_SPEED*uSpeed);
        vec3 body = uM*rest + radial*uRp;
        vec3 flowB = uM*vFlow;
        vec3 advected = body + radial*(uTime*FALL_SPEED*(1.0 - horizontal)*falls) - flowB*uTime*0.35*horizontal;
        vec3 p = advected*RIPPLE_SCALE; float t = uTime*RIPPLE_SPEED;
        float detail = 1.0/(1.0 + length(fwidth(p))*DETAIL_FADE);
        float f0 = fbm3(p, t);
        float height = f0*detail;
        vec3 gradient = vec3(fbm3(p + vec3(0.08, 0.0, 0.0), t) - f0, fbm3(p + vec3(0.0, 0.08, 0.0), t) - f0, fbm3(p + vec3(0.0, 0.0, 0.08), t) - f0)*12.5*detail;
        // Pool flow away from the fall (New tab, mockup; owner, 2026-11-16: "set some flow direction away from the
        // source too"): the pool's waves and foam drift outward from the column, fastest at it (1.2 m/s under a
        // 9-layer fall) and slowing over a few metres. A spreading flow can't be scrolled for ever without
        // stretching, so it is the usual flow-map trick: two copies of the waves, each dragged along the flow for
        // 2 s and reset, cross-faded so neither reset shows.
        if (uOut > 0.0 && vKind < 0.5) {
          vec2 fl = vec2(0.0);
          for (int i = 0; i < ${NF}; i++) {
            if (abs(vP.y - uFeet[i].y) > 0.3) continue;
            vec2 off = vP.xz - uFeet[i].xz; float d = length(off), x = d - 1.25;
            if (x <= -0.5 || x > 10.0) continue;
            fl += off/max(d, 1e-3)*(0.5 + 0.07*min(uFeetH[i], 10.0))*exp(-max(x, 0.0)/(2.5 + 0.3*uFeetH[i]))*smoothstep(-0.5, 0.3, x);
          }
          fl *= uOut;
          if (dot(fl, fl) > 1e-6) {
            const float T = 2.0;
            float a0 = fract(uTime/0.75/T), a1 = fract(uTime/0.75/T + 0.5);
            vec3 flB = uM*vec3(fl.x, 0.0, fl.y)*T*RIPPLE_SCALE;
            vec3 p0 = p - flB*a0, p1 = p - flB*a1 + vec3(0.37, 0.0, 0.61);
            float w0 = 1.0 - abs(2.0*a0 - 1.0), w1 = 1.0 - w0;
            float g0 = fbm3(p0, t), g1 = fbm3(p1, t);
            vec3 gr0 = vec3(fbm3(p0 + vec3(0.08, 0.0, 0.0), t) - g0, fbm3(p0 + vec3(0.0, 0.08, 0.0), t) - g0, fbm3(p0 + vec3(0.0, 0.0, 0.08), t) - g0);
            vec3 gr1 = vec3(fbm3(p1 + vec3(0.08, 0.0, 0.0), t) - g1, fbm3(p1 + vec3(0.0, 0.08, 0.0), t) - g1, fbm3(p1 + vec3(0.0, 0.0, 0.08), t) - g1);
            float m = clamp(length(fl)/0.3, 0.0, 1.0);
            height = mix(height, (g0*w0 + g1*w1)*detail, m);
            gradient = mix(gradient, (gr0*w0 + gr1*w1)*12.5*detail, m);
          }
        }
        gradient -= radial*dot(gradient, radial);
        if (uRain > 0.5) {
          vec2 g = rings(vec2(dot(body, tu), dot(body, tv))*3.0, uTime)*6.0;
          // Only on water that faces up (owner, 2026-11-15: "when raining - no droplets on vertical water").
          g *= smoothstep(0.7, 0.95, horizontal);
          gradient += tu*g.x + tv*g.y;
        }
        // Foot ripples (New tab, mockup; owner, 2026-11-16: "is there some rippling you can do here too? to show
        // water being pushed outward"): rings run out from the falling column across the pool, 0.9 m apart at
        // 1.3 m/s, strongest at the column and dying over a few metres, larger under a taller fall. They tilt the
        // normal (so the reflection and the fresnel ring) and whiten a little on their crests near the column.
        float ripFoam = 0.0;
        if (uRip > 0.0 && vKind < 0.5) {
          for (int i = 0; i < ${NF}; i++) {
            if (abs(vP.y - uFeet[i].y) > 0.3) continue;
            vec2 off = vP.xz - uFeet[i].xz; float d = length(off), x = d - 1.25;
            if (x <= 0.0 || x > 9.0) continue;
            float k = 6.2831853/0.9, ph = k*x - uTime/0.75*1.3*k;
            float reach = 2.0 + 0.35*uFeetH[i];
            float env = exp(-x/reach)*smoothstep(0.0, 0.25, x)*(0.6 + 0.06*min(uFeetH[i], 10.0))*uRip;
            vec2 dir = off/max(d, 1e-3);
            float slope = cos(ph)*env*0.55;
            gradient += uM*vec3(dir.x*slope, 0.0, dir.y*slope)*12.5*0.25;
            ripFoam = max(ripFoam, smoothstep(0.55, 0.95, sin(ph))*env*exp(-x/1.2)*0.6);
          }
        }
        // Glassy lip (New tab, mockup; owner, 2026-11-16, a photo of travertine falls: "check out the transition
        // between smooth water and water fall, can we do something like that?"): the water speeds up and its
        // surface pulls smooth over the last 2 m before the lip, so the top goes glassy (its small waves die away
        // and the sky's reflection runs unbroken), and stays glassy over the first half-metre of the drop.
        float glass = 0.0;
        if (uGlass > 0.5) {
          if (vKind < 0.5 && vPath.z > -50.0 && vPath.z <= 0.0) glass = smoothstep(-2.0, -0.3, vPath.z);
          if (fallFace) glass = 1.0 - smoothstep(0.55, 1.3, vPath.z);
          gradient *= 1.0 - 0.9*glass;
        }
        float raw_slope = length(gradient);
        vec3 capped = gradient*min(1.0, SLOPE_MAX/max(raw_slope, 1e-6));
        vec3 normal = normalize((vKind < 0.5 ? face : radial) - capped*WAVE_STEEP);
        vec3 look = normalize(camB - posB);
        vec3 refl = reflect(-look, normal);
        if (vKind > 0.5 && horizontal < 0.5) refl = reflect(-look, radial);
        float rh = clamp(dot(refl, radial), 0.0, 1.0);
        float fresnel = (0.02 + 0.98*pow(1.0 - max(dot(look, normal), 0.0), 5.0))*mix(SKY_HORIZON, 1.0, rh);
        vec3 reflLocal = transpose(uM)*refl;
        vec3 reflected = skyRad(reflLocal, uNight);
        vec3 nLocal = vKind > 0.5 ? vFaceN : vec3(0.0, 1.0, 0.0);
        vec3 water_lit = uOpen*vSky + lampLight(vP, nLocal, 0.6);
        // Seen from below (the game's 'below' path): the murk by camera distance, and Snell's window.
        if (uEyeUnder > 0.5 && vKind < 0.5 && vDepth > 0.5 && cameraPosition.y < vP.y) {
          float camD = length(cameraPosition - vP);
          vec3 sceneB = fromSrgb(texture2D(uScene, gl_FragCoord.xy/uRes).rgb);
          vec3 underwater = mix(uMurk, sceneB, exp(-uExt*camD));
          vec3 seenB = mix(uMurk, underwater, smoothstep(0.55, 0.75, dot(-look, normal)));
          gl_FragColor = vec4(toSrgb(seenB), 1.0); return;
        }
        float depth_under = vKind > 1.5 ? 1.0 : vDepth;
        float path_len = vKind > 1.5 ? 2.0 : vDepth/max(dot(look, radial), 0.15);
        // The churn over the fall's foot (variant C): the bubbles under the surface scatter the light back up, so the
        // water there reads a lighter, aerated teal and the eye sees less deep into it.
        float aer = 0.0;
        if (uAer > 0.5 && vKind < 0.5) {
          float churn = gnoise3(vec3(vP.x*1.3, uTime*0.9, vP.z*1.3))*0.5 + 0.5;
          for (int i = 0; i < ${NF}; i++) {
            if (abs(vP.y - uFeet[i].y) > 0.3) continue;
            float R = 2.2 + 0.15*uFeetH[i], d = length(vP.xz - uFeet[i].xz);
            aer = max(aer, (1.0 - smoothstep(0.8, R, d))*(0.7 + 0.3*churn));
          }
        }
        vec3 scene = fromSrgb(texture2D(uScene, gl_FragCoord.xy/uRes).rgb);
        vec3 albedo = mix(SHALLOW, DEEP, smoothstep(0.0, SHALLOW_DEPTH, depth_under));
        vec3 transmitted = mix(albedo*water_lit, scene, exp(-(ABSORB + TURBID)*path_len));
        transmitted = mix(transmitted, AER_COL*water_lit, clamp(aer*0.85*uAerAmt, 0.0, 1.0));
        // Glassy lip: the smooth sheet over the edge is a thick body of water, so it shows the water's own green,
        // not the wall behind it.
        if (fallFace && glass > 0.0) transmitted = mix(transmitted, mix(SHALLOW, DEEP, 0.35)*water_lit*1.3, 0.7*glass);
        float crest = smoothstep(0.35, 0.60, height)*0.55;
        float slope = smoothstep(0.50, 1.20, raw_slope)*0.26;
        float foam = clamp(max(crest, slope)*FOAM_INT + ripFoam, 0.0, 1.0);
        // ---- the fall's froth: water.wgsl dc0c5cb, line 806 on ----
        bool lipTop = vKind < 0.5 && vPath.z > -50.0 && uLip > 0.0;
        if (fallFace || lipTop) {
          // Where the noise is read: the rest point, its height measured down the path from the lip and divided by
          // the stretch (B), the path warped by the speed-up (D). With stretch 1 and no speed-up this is the rest
          // point itself, exactly the game's.
          vec3 wN = vec3(vPath.x, vTop - pathC(vPath.z)/uStretch, vPath.y);
          // The froth's own streaks move at (0.35 + 0.6)/0.35 = 2.71 m/s (the scroll and the 0.6 drift on the
          // stretched noise, below): with the speed-up they start at that at the lip and speed up from it.
          if (fallFace && uG > 0.0) wN.y = vTop - fallC(vPath.z, uSpeed*(FALL_SPEED + 0.6/0.35));
          vec3 fr = uFallFrame > 0.5 ? normalize(uM*vFlowN) : face;
          vec3 fu = tangentU(fr), fv = cross(fr, fu);
          vec3 down_p = uM*wN + radial*uRp + radial*(uTime*uSpeed*FALL_SPEED);
          vec3 along = vec3(dot(down_p, fu), dot(down_p, radial), dot(down_p, fv));
          vec3 streak_p = vec3(along.x*5.0, along.y*0.35, along.z*5.0) + vec3(0.0, uTime*uSpeed*0.6, 0.0);
          float n = gnoise3(streak_p)*0.5 + gnoise3(streak_p*2.3)*0.3 + 0.5;
          float froth = smoothstep(0.42, 0.7, n)*0.75 + 0.15;
          if (uGlass > 0.5) {
            // Glassy lip: no whitening on the top; on the face, thin strands first (narrow, long streaks with clear
            // water between), filling in to the full froth over the first 2.5 m down.
            if (!fallFace) froth = 0.0;
            else {
              vec3 sp = vec3(along.x*16.0, along.y*0.12, along.z*16.0) + vec3(0.0, uTime*uSpeed*0.9, 0.0);
              float strands = smoothstep(0.55, 0.8, gnoise3(sp)*0.5 + gnoise3(sp*2.1)*0.25 + 0.5);
              float s = vPath.z, fill = smoothstep(1.0, 2.9, s);
              froth = mix(strands*0.85*smoothstep(0.6, 1.1, s), froth, fill);
            }
          }
          if (fallFace) {
            float weight = clamp(vFroth, 0.35, 1.0);
            foam = max(foam, clamp(froth*weight + 0.12*(1.0 - glass), 0.0, 0.92));
          } else {
            // D: the top whitens into the edge over its last 1.5 m.
            float ramp = smoothstep(-1.5, 0.0, vPath.z)*vFroth*uLip*(1.0 - uGlass);
            foam = max(foam, clamp((froth + 0.12)*ramp, 0.0, 0.92));
          }
        }
        vec3 color = mix(mix(transmitted, reflected, fresnel), FOAM_COL*water_lit, foam);
        vec3 h = normalize(look + uM*uSunDir);
        color += SUN_TINT*pow(max(dot(normal, h), 0.0), SPEC_P)*SPEC_I*uSunI*vSky*4.0;
        gl_FragColor = vec4(toSrgb(color), 1.0);
      }`
  });
  if (vertexShader) m.vertexShader = vertexShader;
  return m;
}

// ---------------- the splash: a port of the game's animate_splash (water_flow.rs) ----------------
// Drops thrown up and out and falling back, and mist puffs, rebuilt every frame as camera-facing quads, nearest foot
// first. The quads carry the dot (cell -1, the game's 32-texel soft dot) or a cell of the new puff sheet (0..15).
const SPLASH_EDGE_M = 1.42, SPLASH_CLEAR_M = 0.05, SPLASH_BUDGET = 4000, SPLASH_REACH_NEW_M = 140;
function hash01(a, b) {
  let h = (Math.imul(a, 0x9E3779B9) ^ Math.imul(b, 0x85EBCA6B)) >>> 0;
  h = (h ^ (h >>> 15)) >>> 0; h = Math.imul(h, 0x2C1B3C6D) >>> 0; h = (h ^ (h >>> 12)) >>> 0;
  return (h & 0xFFFF) / 65535;
}
// foot {at: Vector3 (the landing cell's surface), h: layers fallen, D: metres from the lip's water to the water it
// lands in, sky}; light the game's (0.12 + 0.88 day) * sky.
// tune (New tab only): {mist: puff count x, mistSize: puff size x, rule: "new" | "game"}; 1, 1, "game" is the game's.
// The proposed rule (New tab; owner, 2026-11-16: "anything that exposes this even a tiny bit of the water falling
// animation should have some mist"): every fall is measured in metres it drops, D, not in whole falling layers, and
// has a floor, so a riffle's metre reads as misty:
//   puffs  clamp(round(7 + 0.75 D), 8, 24)            (the game's: clamp((h - 1) 1.5, 1, 24), so 1 for h = 1 or 2)
//   alpha  0.45, every fall                           (the game's: 0.12 + 0.33 min(h / 8, 1), 0.45 from 8 layers)
//   size   max(0.35, min(0.25 + 0.19 D, 5))           (the game's: min(0.5 + 0.18 h, 5))
//   rise   0.15 + 0.05 D m/s                          (the game's: 0.25 + 0.04 h), so a short fall's mist hugs its foot
//   drops  max(12, min(3 + 6 D, 160))                 (the game's: min(3 + 6 h, 160))
//   fall droplets  max(6, min(160, 9 D))
//   where  the puffs rise on the fall's open side, within 110 degrees of the way the water goes over (the game's
//          ring all round the foot puts half of a short fall's few puffs on the ground behind its lip)
// The tall end is the game's (a 9-layer fall's 9.4 m gives 14 puffs at 0.45 and 2.04 m, the game's 12 at 0.45 and
// 2.12 m).
const RULES = {
  game: h => ({m: Math.max(h.h, 1), puffs: Math.min(24, Math.max(1, (Math.max(h.h, 1) - 1) * 1.5)) | 0,
    alpha: m => 0.12 + 0.33 * Math.min(m / 8, 1), size: m => Math.min(0.5 + 0.18 * m, 5), rise: m => 0.25 + 0.04 * m, drops: m => Math.min(3 + 6 * m, 160) | 0,
    veil: m => Math.min(160, 9 * m)}),
  new: h => ({m: h.D, puffs: Math.min(24, Math.max(8, Math.round(7 + 0.75 * h.D))), open: true,
    alpha: () => 0.45, size: m => Math.max(0.35, Math.min(0.25 + 0.19 * m, 5)), rise: m => 0.15 + 0.05 * m,
    drops: m => Math.max(12, Math.min(3 + 6 * m, 160)) | 0, veil: m => Math.max(6, Math.min(160, 9 * m))}),
};
function splashQuads(feet, t, eye, right, upCam, day, newPuffs, out, tune) {
  const mistN = newPuffs && tune ? tune.mist : 1, mistS = newPuffs && tune ? tune.mistSize : 1;
  const veilN = newPuffs && tune ? tune.veil : 0;
  const rule = RULES[newPuffs && tune && tune.rule === "new" ? "new" : "game"];
  out.n = 0;
  const quad = (p, size, rgba, cell, flip) => {
    if (out.n >= SPLASH_BUDGET) return;
    const i = out.n++;
    out.c.set([p.x, p.y, p.z, size], i * 4); out.col.set(rgba, i * 4); out.cell[i] = cell; out.flip[i] = flip;
  };
  const near = feet.slice().sort((a, b) => a.at.distanceTo(eye) - b.at.distanceTo(eye));
  for (const f of near) {
    const R = rule(f), foot = f.at, h = R.m;
    // The game's reach: drawn within 70 m, faded over the last 15 m by distance, so a zoom out pops it away.
    // New tab (proposed; owner, 2026-11-16: "myst popping happens when I zoom out ... and then it disappears"):
    // reach 140 m, and crossing it starts a 0.6 s fade in time (CLAUDE.md, "a fade is a moment in time"), so a
    // still camera never holds a half-faded fall.
    let fade;
    if (newPuffs) {
      const want = foot.distanceTo(eye) < SPLASH_REACH_NEW_M ? 1 : 0, dt = Math.max(0, Math.min(0.1, t - (f._t ?? t)));
      f._t = t; f._vis = f._vis === undefined ? want : f._vis + Math.sign(want - f._vis) * Math.min(Math.abs(want - f._vis), dt / 0.6);
      fade = f._vis; if (fade <= 0) continue;
    } else fade = 1 - Math.min(1, Math.max(0, (foot.distanceTo(eye) - 55) / 15));
    const light = (0.12 + 0.88 * day) * Math.max(f.sky, 0.15);
    const seed = ((Math.trunc(foot.x * 7) >>> 0) ^ (((Math.trunc(foot.z * 13) >>> 0) << 11) | ((Math.trunc(foot.z * 13) >>> 0) >>> 21))) >>> 0;
    const drops = R.drops(h);
    for (let i = 0; i < drops; i++) {
      const r = k => hash01(seed, i * 8 + k);
      const period = 0.7 + 0.5 * r(0), phase = (t / period + r(1)) % 1, s = phase * period;
      const rise = Math.min(1.5 + 0.35 * h, 9) * (0.5 + 0.5 * r(2)), outR = Math.min(0.6 + 0.15 * h, 4) * (0.3 + 0.7 * r(3));
      const a = r(4) * Math.PI * 2, lift = rise * s - 0.5 * 15 * s * s, size = 0.08 + 0.1 * r(6);
      if (lift < SPLASH_CLEAR_M + size) continue;
      const d = SPLASH_EDGE_M + outR * s + 0.4 * r(5);
      quad({x: foot.x + Math.cos(a) * d, y: foot.y + lift, z: foot.z + Math.sin(a) * d}, size,
        [light, light, light * 1.02, 0.75 * (1 - phase) * (newPuffs ? Math.min(1, phase / 0.15) : 1) * fade], -1, 0);
    }
    const puffs = Math.round(R.puffs * mistN);
    for (let i = 0; i < puffs; i++) {
      const r = k => hash01((seed ^ 0xA5A5) >>> 0, i * 8 + k);
      const period = 2 + 1.5 * r(0), phase = (t / period + r(1)) % 1, s = phase * period;
      const a = R.open && f.flow ? Math.atan2(f.flow[1], f.flow[0]) + (r(2) * 2 - 1) * 110 * Math.PI / 180 : r(2) * Math.PI * 2;
      const spread = Math.min(0.5 + 0.12 * h, 5);
      const size = R.size(h) * (0.6 + 0.8 * phase) * mistS;
      const d = SPLASH_EDGE_M + spread * (0.3 + phase) * r(3);
      // New tab: the puff's centre sits low, so its body reaches into the pool and the wall blend (which now reads
      // the water's depth too) fades it out at the waterline; a puff no longer floats clear of the water (owner,
      // 2026-11-16: "mist seems to not be connecting with the bottom"). The game's: its full size above the foot.
      const y = foot.y + R.rise(h) * s + (newPuffs ? 0.2 * size : size + SPLASH_CLEAR_M);
      // New tab: each puff fades in over the first fifth of its life instead of being born at full alpha, which
      // read as puffs flickering in and out on the tall fall's big puffs (owner, 2026-11-16).
      const born = newPuffs ? Math.min(1, phase / 0.2) : 1;
      const alpha = R.alpha(h) * (1 - phase) * born * fade;
      // The new puffs: a variant by the puff's own hash, its frame by its age.
      // A continuous frame (variant*4 + 0..3); the shader cross-fades the two frames either side, so a big puff
      // never changes shape in one frame (owner, 2026-11-16: "see the popping?").
      const cell = newPuffs ? Math.floor(r(5) * 4) * 4 + Math.min(3, phase * 4) : -1;
      quad({x: foot.x + Math.cos(a) * d, y, z: foot.z + Math.sin(a) * d}, size, [light, light, light, alpha], cell, r(6) < 0.5 ? 1 : 0);
    }
    // Fall droplets (New tab only, mockup; owner: "fall mist shouldnt be mist, should be like water droplets fx").
    // Drops shaken off the falling sheet: each leaves the sheet's face at a height along the drop, already moving
    // down at the fall's speed there, flung a little outward, and falls under gravity until it reaches the pool.
    // More of them low down, where the water has fallen furthest and broken up most. Drawn as hard 8-texel
    // droplets (cell -2), not soft puffs.
    const veil = Math.round(R.veil(h) * veilN);
    const G = 15;  // the splash's own gravity, m/s^2
    const STREAK_S = 0.035 * (tune && tune.streak !== undefined ? tune.streak : 1);  // a streak's length in seconds of travel
    for (let i = 0; i < veil; i++) {
      const r = k => hash01((seed ^ 0x5C3E) >>> 0, i * 8 + k);
      const along = Math.pow(r(2), 0.7);                          // 0 at the lip .. 1 at the foot
      const fallen = h * along, y0 = foot.y + h - fallen;
      const vDown = Math.sqrt(2 * G * Math.max(fallen, 0.2));      // the sheet's speed where it leaves
      const vOut = (0.6 + 1.6 * r(4)) * (0.5 + 0.5 * along);       // flung out, m/s
      const tLand = (-vDown + Math.sqrt(vDown * vDown + 2 * G * Math.max(y0 - foot.y, 0.05))) / G;
      const period = tLand + 0.15 + 0.4 * r(0), s = ((t / period + r(1)) % 1) * period;
      if (s > tLand) continue;                                      // gone into the pool
      const a = r(3) * Math.PI * 2;
      const size = (0.035 + 0.05 * r(6)) * mistS;
      // A streak (owner: "make droplets more like streaks"): the drop and a trail of droplets along the path it
      // just took, packed a little closer than their own size so they join into one line, fading towards the tail.
      // Its length follows its speed, so a drop low on a tall fall streaks longest.
      const at = s2 => ({d: SPLASH_EDGE_M * 0.95 + vOut * s2, y: y0 - vDown * s2 - 0.5 * G * s2 * s2});
      const speed = Math.hypot(vOut, vDown + G * s);
      const TRAIL = STREAK_S <= 0 ? 1 : Math.max(3, Math.min(14, Math.round(STREAK_S * speed / (0.8 * size))));
      const head = 0.9 * Math.min(1, s / 0.05) * fade;
      for (let k = 0; k < TRAIL; k++) {
        const s2 = s - k * 0.8 * size / speed;
        if (s2 < 0) break;
        const p = at(s2);
        quad({x: foot.x + Math.cos(a) * p.d, y: p.y, z: foot.z + Math.sin(a) * p.d}, size * (1 - 0.35 * k / TRAIL),
          [light, light, light * 1.03, head * (1 - k / TRAIL)], -2, 0);
      }
    }
  }
}
function splashMesh(U, sheet) {
  const N = SPLASH_BUDGET, g = new THREE.BufferGeometry();
  const corner = new Float32Array(N * 6 * 2), c = new Float32Array(N * 6 * 4), col = new Float32Array(N * 6 * 4), cell = new Float32Array(N * 6 * 2);
  const CORN = [[-1, -1], [1, -1], [1, 1], [-1, -1], [1, 1], [-1, 1]];
  for (let i = 0; i < N; i++) for (let k = 0; k < 6; k++) corner.set(CORN[k], (i * 6 + k) * 2);
  g.setAttribute("position", new THREE.BufferAttribute(new Float32Array(N * 6 * 3), 3));
  g.setAttribute("aCorner", new THREE.BufferAttribute(corner, 2));
  g.setAttribute("aC", new THREE.BufferAttribute(c, 4));
  g.setAttribute("aCol", new THREE.BufferAttribute(col, 4));
  g.setAttribute("aCell", new THREE.BufferAttribute(cell, 2));
  const mat = new THREE.ShaderMaterial({transparent: true, depthWrite: false, depthTest: true,
    uniforms: Object.assign({uRight: {value: new THREE.Vector3()}, uUpCam: {value: new THREE.Vector3()}, uSheet: {value: sheet},
      uGain: {value: 1}, uLamps: {value: 0}, uPix: {value: 1},
      uDepthTex: {value: null}, uDRes: {value: new THREE.Vector2(1, 1)}, uNear: {value: 0.05}, uFar: {value: 600}, uSoft: {value: 0}}, U),
    vertexShader: `attribute vec2 aCorner; attribute vec4 aC; attribute vec4 aCol; attribute vec2 aCell; uniform vec3 uRight; uniform vec3 uUpCam;
      varying vec2 vUV; varying vec4 vCol; varying vec2 vCell; varying vec3 vP; varying float vViewZ;
      void main() { vec3 p = aC.xyz + (uRight*aCorner.x + uUpCam*aCorner.y)*aC.w; vP = p; vViewZ = -(viewMatrix*vec4(p, 1.0)).z;
        vUV = vec2(0.5*(aCorner.x + 1.0), 0.5*(1.0 - aCorner.y)); if (aCell.y > 0.5) vUV.x = 1.0 - vUV.x;
        vCol = aCol; vCell = aCell; gl_Position = projectionMatrix*viewMatrix*vec4(p, 1.0); }`,
    fragmentShader: `uniform sampler2D uSheet; uniform float uGain; uniform float uLamps; uniform float uPix; varying vec2 vUV; varying vec4 vCol; varying vec2 vCell; varying vec3 vP; varying float vViewZ;
      uniform sampler2D uDepthTex; uniform vec2 uDRes; uniform float uNear; uniform float uFar; uniform float uSoft;
      ${LIGHT_GLSL} ${UNDER_GLSL}
      void main() {
        vec3 rgb; float a;
        if (vCell.x < -1.5) {
          // A fall droplet: 8 texels, sampled nearest; a bright core, a lit rim, a glint, no outline.
          vec2 tx = clamp(floor(vUV*8.0), 0.0, 7.0); float r = length((tx + 0.5)/4.0 - 1.0);
          if (uPix < 0.5) {
            // Not pixelated (mockup toggle): a smooth round drop with a soft rim and a soft glint.
            r = length(vUV*2.0 - 1.0); if (r > 1.0) discard;
            a = mix(0.95, 0.0, smoothstep(0.55, 1.0, r)); rgb = mix(vec3(0.86, 0.95, 1.0), vec3(1.25), 1.0 - smoothstep(0.0, 0.3, length(vUV - 0.3)));
          } else {
          if (r > 0.95) discard;
          a = r < 0.55 ? 0.95 : 0.7; rgb = vec3(0.86, 0.95, 1.0);
          if (tx.x == 2.0 && tx.y == 2.0) rgb = vec3(1.25);
          }
        } else if (vCell.x < -0.5) {
          // The game's dot: alpha (1 - r)^2 at the texel's centre, a byte, sampled nearest.
          vec2 texel = clamp(floor(vUV*32.0), 0.0, 31.0);
          float r = length((texel + 0.5)/32.0*2.0 - 1.0), k = clamp(1.0 - r, 0.0, 1.0);
          a = floor(k*k*255.0)/255.0; rgb = vec3(1.0);
          if (uPix < 0.5) { k = clamp(1.0 - length(vUV*2.0 - 1.0), 0.0, 1.0); a = k*k; }
        } else {
          float vr = floor(vCell.x/4.0 + 1e-4), fr = clamp(vCell.x - vr*4.0, 0.0, 3.0);
          float f0 = floor(fr), f1 = min(f0 + 1.0, 3.0), w = fr - f0;
          vec2 tx = uPix > 0.5 ? clamp(floor(vUV*32.0), 0.0, 31.0) + 0.5 : clamp(vUV*32.0, 0.75, 31.25);
          vec2 uv0 = (vec2(f0, vr)*32.0 + tx)/128.0, uv1 = (vec2(f1, vr)*32.0 + tx)/128.0;
          vec4 tex = mix(texture2D(uSheet, vec2(uv0.x, 1.0 - uv0.y)), texture2D(uSheet, vec2(uv1.x, 1.0 - uv1.y)), w);
          rgb = pow(tex.rgb, vec3(2.2)); a = tex.a*uGain;
        }
        vec3 c = rgb*vCol.rgb + rgb*lampLight(vP, vec3(0.0, 1.0, 0.0), 1.0)*uLamps;
        if (uEyeUnder > 0.5) c = mix(uMurk, c, exp(-uExt*length(vP - cameraPosition)));
        // Wall blend (New tab, mockup): a soft particle. The puff fades out as it nears the ground or wall behind it,
        // by the view-space distance between them, so a quad cutting into a wall leaves no hard edge. The game has
        // the same depth to read (the main pass's depth), so this ports as one texture read.
        if (uSoft > 0.0) {
          float d = texture2D(uDepthTex, gl_FragCoord.xy/uDRes).x;
          float sceneZ = 2.0*uNear*uFar/(uFar + uNear - (2.0*d - 1.0)*(uFar - uNear));
          a *= smoothstep(0.0, 1.0, clamp((sceneZ - vViewZ)/uSoft, 0.0, 1.0));
        }
        gl_FragColor = vec4(toSrgb(c), vCol.a*a);
      }`});
  const m = new THREE.Mesh(g, mat); m.frustumCulled = false;
  const Q = {n: 0, c: new Float32Array(N * 4), col: new Float32Array(N * 4), cell: new Float32Array(N), flip: new Float32Array(N)};
  m.userData.Q = Q;
  m.userData.upload = () => {
    for (let i = 0; i < Q.n; i++) for (let k = 0; k < 6; k++) {
      const v = i * 6 + k;
      c[v * 4] = Q.c[i * 4]; c[v * 4 + 1] = Q.c[i * 4 + 1]; c[v * 4 + 2] = Q.c[i * 4 + 2]; c[v * 4 + 3] = Q.c[i * 4 + 3];
      col.set(Q.col.subarray(i * 4, i * 4 + 4), v * 4); cell[v * 2] = Q.cell[i]; cell[v * 2 + 1] = Q.flip[i];
    }
    for (const k of ["aC", "aCol", "aCell"]) g.attributes[k].needsUpdate = true;
    g.setDrawRange(0, Q.n * 6);
  };
  return m;
}

// ---------------- bubbles under the water at the foot (variant C) ----------------
// Small bubbles rising through the pool where the fall lands: born under the falling column and round it, deeper
// and more of them the further the water fell, rising and wobbling, gone at the surface. All on the GPU: a point
// per bubble, its place a function of the time. Drawn before the water, so from above the water's own transmission
// tints them (the game's scene_color), and from under the eye sees them through the murk.
function bubblePoints(F, U) {
  // Twice the default count is made; the Bubbles slider (uAmt, 0-2) shows the first uAmt/2 of them by rank.
  const N = 2 * Math.round(60 + 14 * F.H), seed = new Float32Array(N * 4), pos = new Float32Array(N * 3), rank = new Float32Array(N);
  let s = 5; const rnd = () => (s = (s * 16807) % 2147483647) / 2147483647;
  for (let i = 0; i < N; i++) { seed.set([rnd(), rnd(), rnd(), rnd()], i * 4); rank[i] = (i + 0.5) / N; }
  const g = new THREE.BufferGeometry();
  g.setAttribute("position", new THREE.BufferAttribute(pos, 3));
  g.setAttribute("aSeed", new THREE.BufferAttribute(seed, 4));
  g.setAttribute("aRank", new THREE.BufferAttribute(rank, 1));
  const depth = F.poolY - F.floor;
  const mat = new THREE.ShaderMaterial({transparent: true, depthWrite: false,
    uniforms: Object.assign({uFoot: {value: new THREE.Vector3()}, uH: {value: F.H}, uDepth: {value: depth}, uTime: {value: 0},
      uOpen: {value: new THREE.Vector3(1, 1, 1)}, uPx: {value: 600}, uAmt: {value: 1}, uPix: {value: 1}}, U),
    vertexShader: `attribute vec4 aSeed; attribute float aRank; uniform float uAmt; uniform vec3 uFoot; uniform float uH; uniform float uDepth; uniform float uTime; uniform float uPx;
      varying float vA; varying vec3 vP;
      void main() {
        float sq = sqrt(uH);
        float d0 = mix(0.25, min(uDepth - 0.15, 0.6 + 0.22*uH), aSeed.y);   // how deep it is born
        float speed = 0.35 + 0.35*aSeed.z;                                   // m/s, smaller rise slower
        float life = d0/speed + 0.4;
        float tl = fract(uTime/life + aSeed.x)*life;
        float ang = aSeed.w*6.2831853, rad = pow(fract(aSeed.x*7.31 + aSeed.y*3.7), 0.7)*(1.2 + 0.09*uH);
        float y = uFoot.y - d0 + speed*tl;
        vec3 p = vec3(uFoot.x + cos(ang)*rad + 0.08*sin(tl*7.0 + aSeed.z*20.0), min(y, uFoot.y - 0.04), uFoot.z + sin(ang)*rad + 0.08*cos(tl*6.0 + aSeed.w*20.0));
        vA = smoothstep(0.0, 0.25, tl)*(y < uFoot.y - 0.04 ? 1.0 : 0.0)*(aRank < 0.5*uAmt ? 1.0 : 0.0);
        vP = p;
        vec4 mv = viewMatrix*vec4(p, 1.0);
        float size = 0.05 + 0.09*aSeed.z;
        gl_PointSize = vA > 0.0 ? max(2.0, size*uPx/max(-mv.z, 0.1)) : 0.0;
        gl_Position = projectionMatrix*mv;
      }`,
    fragmentShader: `uniform vec3 uOpen; uniform vec3 uFoot; uniform float uPix; varying float vA; varying vec3 vP; ${LIGHT_GLSL} ${UNDER_GLSL}
      void main() {
        // An 8-texel bubble, sampled nearest: a bright rim, a clear middle, a glint up and to the left. No outline:
        // the rim is the bubble.
        vec2 tx = uPix > 0.5 ? floor(gl_PointCoord*8.0) : gl_PointCoord*8.0 - 0.5; vec2 d = (tx + 0.5)/4.0 - 1.0; float r = length(d);
        if (r > 1.0) discard;
        float rim = smoothstep(0.55, 0.8, r);
        float a = mix(0.28, 0.9, rim);
        vec3 c = vec3(0.86, 0.97, 1.0);
        if (uPix > 0.5 && tx.x == 2.0 && tx.y == 2.0) { c = vec3(1.2); a = 1.0; }
        if (uPix < 0.5) { a *= 1.0 - smoothstep(0.85, 1.0, r); float g = 1.0 - smoothstep(0.0, 0.3, length(d + 0.4)); c = mix(c, vec3(1.2), g); a = max(a, g); }
        vec3 lit = uOpen*0.9 + lampLight(vP, vec3(0.0, 1.0, 0.0), 1.0);
        vec3 col = c*lit;
        // Seen from above, a bubble is under the water: it takes the aerated water's teal and fades by its own
        // depth below the surface (the water's transmission reads the pool's floor depth, not the bubble's, so
        // without this a white bubble came through nearly white; owner, 2026-11-16: "they dont seem affected by
        // water transparency").
        float below = max(uFoot.y - vP.y, 0.0), k = exp(-1.6*below);
        if (uEyeUnder < 0.5) { col = mix(vec3(0.16, 0.50, 0.46)*lit, col, 0.25 + 0.5*k); a *= 0.25 + 0.55*k; }
        if (uEyeUnder > 0.5) col = mix(uMurk, col, exp(-uExt*length(vP - cameraPosition)));
        gl_FragColor = vec4(toSrgb(col), a*vA);
      }`});
  const pts = new THREE.Points(g, mat); pts.frustumCulled = false;
  return pts;
}

// ---------------- the page's scene ----------------
function lampMesh(x, y, z) {
  const grp = new THREE.Group();
  const post = new THREE.Mesh(new THREE.BoxGeometry(0.14, 2.4, 0.14), new THREE.MeshBasicMaterial({color: 0x3a2c22}));
  post.position.set(x, y + 1.2, z); grp.add(post);
  const bulbMat = new THREE.MeshBasicMaterial({color: 0x443322});
  const bulb = new THREE.Mesh(new THREE.BoxGeometry(0.34, 0.34, 0.34), bulbMat); bulb.position.set(x, y + 2.55, z); grp.add(bulb);
  grp.userData.bulb = bulbMat; return grp;
}

// opts: {labels: an element over the canvas for each fall's one-line label}.
window.PBDMist = function (canvas, sheetUrl, opts) {
  const renderer = new THREE.WebGLRenderer({canvas, antialias: false, preserveDrawingBuffer: true});
  renderer.autoClear = false;
  const LAMPS = LAMP_SPOTS;
  const U = {uSunDir: {value: new THREE.Vector3()}, uSunCol: {value: new THREE.Vector3()}, uAmb: {value: new THREE.Vector3()},
    uLampPos: {value: LAMPS.map(([x, y, z]) => new THREE.Vector3(x, y + 2.55, z))}, uLampCol: {value: LAMPS.map(() => new THREE.Vector3(1.0, 0.62, 0.30).multiplyScalar(1.7))}, uLampOn: {value: 0},
    uEyeUnder: {value: 0}, uMurk: {value: new THREE.Vector3()}, uExt: {value: new THREE.Vector3(0.39, 0.105, 0.082)}};
  const opaque = new THREE.Scene(), waterScene = new THREE.Scene(), fx = new THREE.Scene();
  const skyMat = new THREE.ShaderMaterial({side: THREE.BackSide, depthWrite: false, uniforms: Object.assign({uNight: {value: 0}}, U),
    vertexShader: `varying vec3 vD; void main() { vD = position; vec4 p = projectionMatrix*viewMatrix*vec4(position + cameraPosition, 1.0); gl_Position = p.xyww; }`,
    fragmentShader: `uniform float uNight; varying vec3 vD; ${LIGHT_GLSL} ${UNDER_GLSL} void main() { vec3 d = normalize(vD);
      vec3 c = skyRad(d, uNight); float s = max(dot(d, uSunDir), 0.0); c += uSunCol*pow(s, 400.0)*4.0*(1.0 - uNight);
      if (uEyeUnder > 0.5) c = uMurk;
      gl_FragColor = vec4(toSrgb(c), 1.0); }`});
  const sky = new THREE.Mesh(new THREE.SphereGeometry(300, 24, 12), skyMat); sky.frustumCulled = false; sky.renderOrder = -1; opaque.add(sky);
  opaque.add(groundMesh(U));
  const lamps = LAMPS.map(([x, y, z]) => { const m = lampMesh(x, y, z); opaque.add(m); return m; });
  const wmat = waterMaterial(U);
  const lat = 24.117 * Math.PI / 180, lon = -2.964 * Math.PI / 180;
  const up = new THREE.Vector3(Math.cos(lat) * Math.cos(lon), Math.sin(lat), Math.cos(lat) * Math.sin(lon));
  const east = new THREE.Vector3(0, 1, 0).cross(up).normalize(), north = up.clone().cross(east);
  wmat.uniforms.uM.value.set(north.x, up.x, east.x, north.y, up.y, east.y, north.z, up.z, east.z);
  const water = new THREE.Mesh(waterGeometry(0), wmat); water.frustumCulled = false; waterScene.add(water);
  // Every fall's foot: where it lands, the game's h (its falling layers) and D (metres from the lip's water to the
  // water it lands in, the proposed rule's measure).
  const feet = FALLS.map((F, i) => {
    const at = new THREE.Vector3(F.cR[0], F.poolY, F.cR[1]);
    wmat.uniforms.uFeet.value[i].copy(at); wmat.uniforms.uFeetH.value[i] = F.drop;
    return {at, h: F.N, D: F.H, sky: 0.9, flow: F.flow};
  });
  // Flow arrows (mockup toggle): the pool's outward flow round each foot, the same field the water shader drifts
  // its waves along, drawn on top. Arrow length is the speed, 1 m per m/s.
  const arrows = new THREE.Group(); arrows.visible = false; arrows.renderOrder = 10; fx.add(arrows);
  feet.forEach((f, i) => {
    const H = FALLS[i].drop;
    for (const rr of [1.8, 3.2, 4.8, 6.6]) for (let k = 0; k < 10; k++) {
      const a = k / 10 * Math.PI * 2 + rr, x = f.at.x + Math.cos(a) * rr, z = f.at.z + Math.sin(a) * rr;
      const [q, r] = hexAt(x, z), pl = poolAt(q, r);
      if (pl === null || Math.abs(pl - f.at.y) > 0.3) continue;
      const sp = (0.5 + 0.07 * Math.min(H, 10)) * Math.exp(-Math.max(rr - 1.25, 0) / (2.5 + 0.3 * H));
      const arr = new THREE.ArrowHelper(new THREE.Vector3(Math.cos(a), 0, Math.sin(a)), new THREE.Vector3(x, f.at.y + 0.06, z), Math.max(0.35, sp * 1.6), 0xff6a2a, 0.22, 0.14);
      arr.traverse(o => { if (o.material) { o.material.depthTest = false; o.material.transparent = true; o.renderOrder = 10; } });
      arr.userData.sp = sp; arrows.add(arr);
    }
  });
  // The new sheet, sampled nearest with no mipmaps.
  const sheet = new THREE.TextureLoader().load(sheetUrl);
  sheet.magFilter = sheet.minFilter = THREE.NearestFilter; sheet.generateMipmaps = false;
  // The splash, twice: in the opaque pass (A, the game today: the water reads it as what is behind it and paints over
  // it) and after the water (B and C, the order fix).
  const leak = splashMesh(U, sheet); leak.renderOrder = 5; opaque.add(leak);
  const front = splashMesh(U, sheet); fx.add(front);
  const bubbles = FALLS.map((F, i) => { const b = bubblePoints(F, U); b.material.uniforms.uFoot.value.copy(feet[i].at); opaque.add(b); return b; });
  const camera = new THREE.PerspectiveCamera(50, 1, 0.05, 600);
  let copyTex = null, copyW = 0, copyH = 0, depthRT = null;
  const S = {variant: "new", night: false, running: true, gain: 2, bubbles: 1, teal: 1, mist: 1, mistSize: 1.65, veil: 1, soft: 1.2, streak: 0,
    rule: "new", accel: 2, pixel: false, rip: 1, out: 1, arrows: false, glass: true};
  // The eye's pool (the cell under it, if it is a basin's), for the view from under the water.
  let eyePool = null;
  function murk(open) {
    // The game's murk: the deep's albedo lit by the light on open water, dimmed by the water above the eye.
    const eyeDepth = eyePool === null ? 0 : Math.max(0, eyePool - camera.position.y);
    const k = Math.exp(-0.12 * eyeDepth);
    return new THREE.Vector3(0.020, 0.090, 0.075).lerp(new THREE.Vector3(0.004, 0.018, 0.040), 0.35).multiply(open).multiplyScalar(2.2 * k);
  }
  let open = new THREE.Vector3(1, 1, 1);
  function lighting() {
    if (!S.night) {
      U.uSunDir.value.set(0.5, 0.72, -0.48).normalize(); U.uSunCol.value.set(1.0, 0.96, 0.9); U.uAmb.value.set(0.30, 0.34, 0.40); U.uLampOn.value = 0;
    } else {
      U.uSunDir.value.set(0.3, 0.75, -0.4).normalize(); U.uSunCol.value.set(0.020, 0.026, 0.050); U.uAmb.value.set(0.010, 0.013, 0.026); U.uLampOn.value = 1;
    }
    open = U.uSunCol.value.clone().multiplyScalar(Math.max(U.uSunDir.value.y, 0)).add(U.uAmb.value).multiplyScalar(S.night ? 1 : 0.82);
    wmat.uniforms.uOpen.value.copy(open); wmat.uniforms.uNight.value = S.night ? 1 : 0; wmat.uniforms.uSunI.value = S.night ? 0 : 1;
    skyMat.uniforms.uNight.value = S.night ? 1 : 0;
    for (const b of bubbles) b.material.uniforms.uOpen.value.copy(open).multiplyScalar(0.8);
    for (const l of lamps) l.userData.bulb.color.set(S.night ? 0xffd28a : 0x443322);
  }
  // The speed-up down a fall: g = 3 m/s^2 at strength 1, to at most 8 m/s (New tab only; the others keep the game's
  // constant scroll).
  const ACCEL_G = 3, ACCEL_VMAX = 8;
  function apply() {
    const v = S.variant;
    leak.visible = v === "today"; front.visible = v !== "today";
    for (const b of bubbles) { b.visible = v === "new"; b.material.uniforms.uAmt.value = S.bubbles; }
    wmat.uniforms.uAer.value = v === "new" ? 1 : 0;
    wmat.uniforms.uAerAmt.value = S.teal; wmat.uniforms.uRip.value = v === "new" ? S.rip : 0; wmat.uniforms.uOut.value = v === "new" ? S.out : 0; wmat.uniforms.uGlass.value = v === "new" && S.glass ? 1 : 0;
    const wantR = v === "new" && S.glass ? 0.45 : 0;
    if (wantR !== LIP_R) { LIP_R = wantR; water.geometry.dispose(); water.geometry = waterGeometry(0); }
    if (arrows) arrows.visible = v === "new" && S.arrows;
    wmat.uniforms.uG.value = v === "new" ? ACCEL_G * S.accel : 0; wmat.uniforms.uVmax.value = ACCEL_VMAX;
    front.material.uniforms.uGain.value = S.gain; front.material.uniforms.uLamps.value = v === "new" ? 1 : 0;
    front.material.uniforms.uSoft.value = v === "new" ? S.soft : 0;
    // Pixelated FX (mockup toggle): off samples the puff sheet smoothly and draws the drops and bubbles round.
    const pix = v !== "new" || S.pixel;
    front.material.uniforms.uPix.value = pix ? 1 : 0; for (const b of bubbles) b.material.uniforms.uPix.value = pix ? 1 : 0;
    const filt = pix ? THREE.NearestFilter : THREE.LinearFilter;
    if (sheet.magFilter !== filt) { sheet.magFilter = sheet.minFilter = filt; if (sheet.image) sheet.needsUpdate = true; }
    lighting();
  }
  function resize() {
    const w = canvas.clientWidth, h = canvas.clientHeight; if (!w || !h) return;
    renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2)); renderer.setSize(w, h, false);
    camera.aspect = w / h; camera.updateProjectionMatrix();
    const sz = renderer.getDrawingBufferSize(new THREE.Vector2());
    if (sz.x !== copyW || sz.y !== copyH) {
      if (copyTex) copyTex.dispose();
      copyW = sz.x; copyH = sz.y;
      copyTex = new THREE.DataTexture(new Uint8Array(copyW * copyH * 3), copyW, copyH, THREE.RGBFormat);
      copyTex.minFilter = copyTex.magFilter = THREE.NearestFilter; copyTex.generateMipmaps = false; copyTex.needsUpdate = true;
      wmat.uniforms.uScene.value = copyTex; wmat.uniforms.uRes.value.set(copyW, copyH);
      if (depthRT) depthRT.dispose();
      depthRT = new THREE.WebGLRenderTarget(copyW, copyH);
      depthRT.depthTexture = new THREE.DepthTexture(copyW, copyH); depthRT.depthTexture.type = THREE.UnsignedIntType;
      depthRT.depthTexture.minFilter = depthRT.depthTexture.magFilter = THREE.NearestFilter;
      front.material.uniforms.uDepthTex.value = depthRT.depthTexture; front.material.uniforms.uDRes.value.set(copyW, copyH);
    }
    for (const b of bubbles) b.material.uniforms.uPx.value = sz.y / (2 * Math.tan(camera.fov * Math.PI / 360));
  }
  // ---- the view: drag to turn, wheel or pinch to come closer. Nothing moves on its own.
  const V = {target: new THREE.Vector3(), yaw: 0, pitch: -0.7, dist: 10};
  function place() {
    const d = new THREE.Vector3(Math.cos(V.pitch) * Math.cos(V.yaw), Math.sin(V.pitch), Math.cos(V.pitch) * Math.sin(V.yaw));
    camera.position.copy(V.target).addScaledVector(d, -V.dist); camera.lookAt(V.target);
  }
  // Looking at a point from yaw, pitch (degrees) and a distance; or from an eye.
  function frame(target, yawDeg, pitchDeg, dist) {
    V.yaw = yawDeg * Math.PI / 180; V.pitch = pitchDeg * Math.PI / 180; V.dist = dist; V.target.set(...target); place();
  }
  function view(eye, yawDeg, pitchDeg, dist) {
    V.yaw = yawDeg * Math.PI / 180; V.pitch = pitchDeg * Math.PI / 180; V.dist = dist;
    const d = new THREE.Vector3(Math.cos(V.pitch) * Math.cos(V.yaw), Math.sin(V.pitch), Math.cos(V.pitch) * Math.sin(V.yaw));
    V.target.set(...eye).addScaledVector(d, dist); place();
  }
  // The views: of every fall at once ("all"), or of one, framed by its height. Under is always one fall's pool (the
  // 9-layer one for "all").
  const zMid = (cz(FALLS[0].c[1]) + cz(FALLS[NF - 1].c[1])) / 2;
  function close(F, which) {
    const [Rx, Rz] = F.cR, [Cx, Cz] = F.cC;
    if (which === "above") return frame([Rx - 0.5, F.poolY + 0.45 * F.H, Rz + 0.4], -4, -36, 6 + 0.85 * F.H);
    if (which === "side") return frame([Rx - 0.6, F.poolY + 0.4 * F.H + 0.3, Rz + 0.4], -5, -9, 5 + 0.55 * F.H);
    return view([Cx - 0.5, Math.max(F.poolY - 1.6, F.floor + 0.3), Cz + 0.4], -4, 16, 6);
  }
  let FOCUS = "all", VIEW = "above";
  function look(viewName, focus) {
    if (viewName) VIEW = viewName; if (focus !== undefined) FOCUS = focus;
    const F = FOCUS === "all" ? null : FALLS.find(f => f.name === FOCUS);
    if (F) return close(F, VIEW);
    if (VIEW === "above") return frame([HW * 0.6, -3.2, zMid], 0, -34, 46);
    if (VIEW === "side") return frame([HW, -2.6, zMid + 1], 0, -9, 36);
    return close(FALLS[NF - 1], "under");
  }
  const VIEWS = {above: () => look("above"), side: () => look("side"), under: () => look("under")};
  const ptrs = new Map(); let pinch0 = 0, panning = false;
  // Middle-button drag pans: the point looked at slides with the mouse, at the distance looked from.
  function pan(dx, dy) {
    const k = 2 * V.dist * Math.tan(camera.fov * Math.PI / 360) / Math.max(1, canvas.clientHeight);
    camera.updateMatrixWorld(); const right = new THREE.Vector3().setFromMatrixColumn(camera.matrixWorld, 0);
    const up = new THREE.Vector3().setFromMatrixColumn(camera.matrixWorld, 1);
    V.target.addScaledVector(right, -dx * k).addScaledVector(up, dy * k); place();
  }
  canvas.addEventListener("mousedown", e => { if (e.button === 1) e.preventDefault(); });
  canvas.addEventListener("auxclick", e => { if (e.button === 1) e.preventDefault(); });
  canvas.addEventListener("pointerdown", e => { if (e.button === 1) { e.preventDefault(); panning = true; } canvas.setPointerCapture(e.pointerId); ptrs.set(e.pointerId, [e.clientX, e.clientY]); if (ptrs.size === 2) { const [a, b] = [...ptrs.values()]; pinch0 = Math.hypot(a[0] - b[0], a[1] - b[1]); } });
  canvas.addEventListener("pointermove", e => {
    if (!ptrs.has(e.pointerId)) return; const [px, py] = ptrs.get(e.pointerId); ptrs.set(e.pointerId, [e.clientX, e.clientY]);
    if (panning) pan(e.clientX - px, e.clientY - py);
    else if (ptrs.size === 1) { V.yaw += (e.clientX - px) * 0.006; V.pitch = Math.max(-1.45, Math.min(1.2, V.pitch - (e.clientY - py) * 0.006)); place(); }
    else if (ptrs.size === 2) { const [a, b] = [...ptrs.values()]; const d = Math.hypot(a[0] - b[0], a[1] - b[1]); if (pinch0 > 0) { V.dist = Math.max(1, Math.min(90, V.dist * pinch0 / d)); place(); } pinch0 = d; }
  });
  const up_ = e => { ptrs.delete(e.pointerId); pinch0 = 0; if (!ptrs.size) panning = false; };
  canvas.addEventListener("pointerup", up_); canvas.addEventListener("pointercancel", up_);
  canvas.addEventListener("wheel", e => { e.preventDefault(); V.dist = Math.max(1, Math.min(90, V.dist * Math.exp(e.deltaY * 0.001))); place(); }, {passive: false});
  // ---- each fall's label, over its lip
  const labelHost = opts && opts.labels;
  const labels = labelHost ? FALLS.map(F => { const d = document.createElement("span"); d.textContent = F.label; labelHost.appendChild(d); return d; }) : [];
  const lp = new THREE.Vector3();
  function placeLabels() {
    if (!labels.length) return;
    const w = canvas.clientWidth, h = canvas.clientHeight;
    FALLS.forEach((F, i) => {
      lp.set(F.cS[0], 1.9, F.cS[1]).project(camera);
      const shown = lp.z > -1 && lp.z < 1 && Math.abs(lp.x) < 0.98 && Math.abs(lp.y) < 0.98 && U.uEyeUnder.value < 0.5;
      labels[i].hidden = !shown;
      if (shown) labels[i].style.transform = `translate(${((lp.x + 1) / 2 * w).toFixed(1)}px, ${((1 - lp.y) / 2 * h).toFixed(1)}px) translate(-50%, -100%)`;
    });
  }
  // ---- the frame
  let simT = 0, raf = 0, last = performance.now();
  function frameLoop(now) {
    raf = requestAnimationFrame(frameLoop);
    const dt = Math.min(0.1, (now - last) / 1000); last = now;
    if (S.running) simT += dt;
    draw();
  }
  function draw() {
    wmat.uniforms.uTime.value = simT * 0.75;
    for (const b of bubbles) b.material.uniforms.uTime.value = simT;
    // The eye under a pool: the game's compose fogs everything by its distance, and the water draws from below.
    const [hq, hr] = hexAt(camera.position.x, camera.position.z);
    eyePool = poolAt(hq, hr);
    const under = eyePool !== null && camera.position.y < eyePool;
    U.uEyeUnder.value = under ? 1 : 0; U.uMurk.value.copy(murk(open));
    const right = new THREE.Vector3().setFromMatrixColumn(camera.matrixWorld, 0), upCam = new THREE.Vector3().setFromMatrixColumn(camera.matrixWorld, 1);
    const day = S.night ? 0 : 1;
    for (const m of [leak, front]) {
      if (!m.visible) continue;
      splashQuads(feet, simT, camera.position, right, upCam, day, S.variant === "new", m.userData.Q, {mist: S.mist, mistSize: S.mistSize, veil: S.veil, streak: S.streak, rule: S.rule});
      m.material.uniforms.uRight.value.copy(right); m.material.uniforms.uUpCam.value.copy(upCam);
      m.userData.upload();
    }
    if (depthRT && front.visible && front.material.uniforms.uSoft.value > 0) {
      // The ground and walls' depth, for the wall blend: the opaque scene once more into a depth target.
      front.material.uniforms.uNear.value = camera.near; front.material.uniforms.uFar.value = camera.far;
      renderer.setRenderTarget(depthRT); renderer.clear(); renderer.render(opaque, camera); renderer.render(waterScene, camera); renderer.setRenderTarget(null);
    }
    renderer.clear();
    renderer.render(opaque, camera);
    renderer.copyFramebufferToTexture(new THREE.Vector2(0, 0), copyTex);
    renderer.render(waterScene, camera);
    renderer.render(fx, camera);
    placeLabels();
  }
  resize(); look("above", "all"); apply();
  window.addEventListener("resize", resize);
  return {
    S, apply, resize, views: VIEWS, look, falls: FALLS.map(F => ({name: F.name, label: F.label, N: F.N, D: F.H})),
    start() { if (!raf) { last = performance.now(); raf = requestAnimationFrame(frameLoop); } },
    stop() { cancelAnimationFrame(raf); raf = 0; },
    setTime(sec) { simT = sec; draw(); },
    info() { const gl = renderer.getContext(); const ext = gl.getExtension("WEBGL_debug_renderer_info"); return ext ? gl.getParameter(ext.UNMASKED_RENDERER_WEBGL) : gl.getParameter(gl.RENDERER); },
  };
};
})();
