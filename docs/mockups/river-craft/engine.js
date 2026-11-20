// River Craft (M34): the shared engine. A river as a 2D flow field over a channel of hex cells (the game's cells:
// 2.833 m flat to flat, 1 m layers), the blocky pixel-art ground, the river's water, the lights (sun, a dim moon and
// every lamp near what it lights), the orbit camera and the fixed-step clock. Needs three.js r128 (THREE) before it.
// The water shader is a cut-down stand-in for the game's (assets/shaders/water.wgsl): its colour terms, a flow-mapped
// ripple that moves with the current, foam where the water runs fast. See waterfall/waterfall.js for the full port.
(function () {
"use strict";
const RC = window.RC = window.RC || {};
const HW = 2.833, HR = HW / Math.sqrt(3);
const DIRS = [[1, 0], [0, 1], [-1, 1], [-1, 0], [0, -1], [1, -1]];   // side i faces 60 i degrees
const cx = (q, r) => HW * (q + r / 2), cz = r => 1.5 * HR * r;
const key = (q, r) => q + "," + r;
const clamp = (v, a, b) => v < a ? a : v > b ? b : v;
const smooth = (a, b, x) => { const t = clamp((x - a) / (b - a), 0, 1); return t * t * (3 - 2 * t); };
function hexAt(x, z) {
  const r = z / (1.5 * HR), q = x / HW - r / 2, s = -q - r;
  let rq = Math.round(q), rr = Math.round(r); const rs = Math.round(s);
  const dq = Math.abs(rq - q), dr = Math.abs(rr - r), ds = Math.abs(rs - s);
  if (dq > dr && dq > ds) rq = -rr - rs; else if (dr > ds) rr = -rq - rs;
  return [rq, rr];
}
Object.assign(RC, {HW, HR, DIRS, cx, cz, key, clamp, smooth, hexAt});

// ---------------- the river: channel, level, flow ----------------
// cfg: x0, x1, z0, z1 (the patch), center(x), half(x) (half width, m), depth(x) (centre depth, m), U(x) (centre speed,
// m/s), level0 and slope (the water's level), steps [{x, drop}], rocks [[x, zOff]] (snapped to cells), bed(x, z) the
// bed's tile name, lamps [[x, zSide]] (posts on the bank).
RC.River = function (cfg) {
  this.cfg = cfg;
  this.rocks = [];
  this.cells = new Map();
  for (const [x, dz] of cfg.rocks || []) {
    const [q, r] = hexAt(x, cfg.center(x) + dz);
    if (!this.rocks.some(k => k.q === q && k.r === r)) this.rocks.push({q, r, x: cx(q, r), z: cz(r), R: 1.55});
  }
  this.build();
};
const RP = RC.River.prototype;
RP.level = function (x) {
  const c = this.cfg; let y = c.level0 - c.slope * x;
  for (const s of c.steps || []) y -= s.drop * smooth(s.x - 0.75, s.x + 0.75, x);
  return y;
};
// The water's surface: the level plus the standing waves below a step (they stand still in a rapid).
RP.surf = function (x, z) {
  let y = this.level(x);
  for (const s of this.cfg.steps || []) {
    const d = x - s.x; if (d > 0.5 && d < 14) y += 0.11 * Math.sin(d * 2.3) * Math.exp(-(((d - 4) / 4) ** 2)) * (0.7 + 0.3 * Math.cos(z * 0.9));
  }
  return y;
};
RP.speedAt = function (x) {
  let u = this.cfg.U(x);
  for (const s of this.cfg.steps || []) u *= 1 + 1.3 * Math.exp(-(((x - s.x) / 3.2) ** 2));
  return u;
};
// The flow at (x, z), into out = [vx, vz]: along the channel's centre line, fastest mid-stream and slow at the
// banks, bent round each rock (a cylinder's potential flow) with an eddy in its lee that runs back upstream.
RP.flow = function (x, z, out) {
  const c = this.cfg, cxl = c.center(x), dc = (c.center(x + 0.5) - c.center(x - 0.5));
  const il = 1 / Math.hypot(1, dc), tx = il, tz = dc * il;
  const w = c.half(x), n = (z - cxl) * tx, s = Math.abs(n) / w;
  if (s >= 1) { out[0] = 0; out[1] = 0; return out; }
  const U = this.speedAt(x) * Math.sqrt(Math.max(0, 1 - s * s * s * s));
  let ua = U, us = 0;
  for (const k of this.rocks) {
    const dx = (x - k.x) * tx + (z - k.z) * tz, dz = -(x - k.x) * tz + (z - k.z) * tx;
    const r2 = dx * dx + dz * dz, R2 = k.R * k.R;
    if (r2 > 100) continue;
    if (r2 < R2) { out[0] = 0; out[1] = 0; return out; }
    const f = Math.exp(-r2 / 30), r4 = r2 * r2;
    ua -= U * f * R2 * (dx * dx - dz * dz) / r4;
    us -= U * f * 2 * R2 * dx * dz / r4;
    if (dx > 0) {
      const wk = Math.exp(-((dz / (1.25 * k.R)) ** 2)) * Math.exp(-dx / (3.2 * k.R)) * smooth(0, k.R * 1.2, dx);
      ua -= U * 1.45 * wk;                                            // the eddy: back upstream behind the rock
      us -= U * 0.35 * wk * Math.sign(dz) * smooth(k.R * 2, k.R * 4, dx);   // and in toward the wake's middle
    }
  }
  out[0] = ua * tx - us * tz; out[1] = ua * tz + us * tx;
  return out;
};
// The cells: bank (grass above the water), bed (under it) and rock (a stone cell standing out of the river).
RP.build = function () {
  const c = this.cfg; this.cells.clear();
  const [qa] = hexAt(c.x0, 0), [qb] = hexAt(c.x1, 0);
  const ra = Math.floor(c.z0 / (1.5 * HR)) - 1, rb = Math.ceil(c.z1 / (1.5 * HR)) + 1;
  for (let r = ra; r <= rb; r++) for (let q = qa - Math.abs(r); q <= qb + Math.abs(r); q++) {
    const x = cx(q, r), z = cz(r);
    if (x < c.x0 || x > c.x1 || z < c.z0 || z > c.z1) continue;
    const lv = this.level(x), w = c.half(x), s = Math.abs(z - c.center(x)) / w;
    let top, kind, tile;
    if (this.rocks.some(k => k.q === q && k.r === r)) { top = Math.floor(lv) + 1 + (((q * 7 + r * 3) & 1)); kind = "rock"; tile = "stone"; }
    else if (s < 1) {
      const d = c.depth(x) * (1 - s * s) + 0.35;
      top = Math.floor(lv - d); kind = "bed"; tile = c.bed(x, z);
    } else {
      const far = (s - 1) * w, bump = Math.sin(x * 0.11 + z * 0.07) + Math.sin(x * 0.05 - z * 0.13);
      top = Math.floor(lv) + 1 + (far > 6 && bump > 0.6 ? 1 : 0) + (far > 12 && bump > 1.2 ? 1 : 0);
      kind = "bank"; tile = "grass";
    }
    this.cells.set(key(q, r), {q, r, x, z, top, kind, tile, lv});
  }
};
RP.cell = function (x, z) { const [q, r] = hexAt(x, z); return this.cells.get(key(q, r)); };
RP.top = function (x, z) { const k = this.cell(x, z); return k ? k.top : 50; };
// Push a point out of a cell that stands above `bottom`, through the nearest side whose neighbour is open.
// Returns null when the point is free, else [nx, nz, depth].
RP.solid = function (x, z, bottom) {
  const [q, r] = hexAt(x, z), k = this.cells.get(key(q, r));
  if (!k) return [-(x - (this.cfg.x0 + this.cfg.x1) / 2) / 100, 0, 0.05];
  if (k.top <= bottom) return null;
  let best = null;
  for (let i = 0; i < 6; i++) {
    const [dq, dr] = DIRS[i], nb = this.cells.get(key(q + dq, r + dr));
    if (!nb || nb.top > bottom) continue;
    const nx = Math.cos(Math.PI / 3 * i), nz = Math.sin(Math.PI / 3 * i);
    const d = HW / 2 - ((x - k.x) * nx + (z - k.z) * nz);
    if (!best || d < best[2]) best = [nx, nz, d];
  }
  if (!best) {   // walled in: push back toward the channel's middle
    const cz0 = this.cfg.center(x), n = z - cz0; return [0, -Math.sign(n) || 1, 0.2];
  }
  return best;
};

// ---------------- pixel-art textures: one atlas, 16 px a tile, sampled nearest ----------------
const TILES = ["grass", "grassSide", "dirt", "sand", "gravel", "mud", "stone", "bark", "logEnd", "planks", "darkPlanks", "iron", "rope", "net", "canvas", "crate"];
RC.TILE = {}; TILES.forEach((n, i) => { RC.TILE[n] = i; });
function atlasCanvas() {
  const c = document.createElement("canvas"); c.width = 128; c.height = 32; const g = c.getContext("2d");
  let seed = 11; const rnd = () => (seed = (seed * 16807) % 2147483647) / 2147483647;
  const tile = (i, base, specks) => {
    const ox = (i % 8) * 16, oy = Math.floor(i / 8) * 16;
    g.fillStyle = base; g.fillRect(ox, oy, 16, 16);
    for (const [col, n, w, h] of specks) for (let k = 0; k < n; k++) { g.fillStyle = col; g.fillRect(ox + Math.floor(rnd() * 16), oy + Math.floor(rnd() * 16), w || 1, h || 1); }
    return [ox, oy];
  };
  tile(0, "#5d8a3a", [["#6f9c44", 40], ["#4c7631", 34], ["#7aa84c", 8, 1, 2]]);
  const [gx, gy] = tile(1, "#7a5534", [["#6a4a2e", 30], ["#8a6440", 20]]);
  g.fillStyle = "#5d8a3a"; g.fillRect(gx, gy, 16, 4); g.fillStyle = "#4c7631";
  for (let i = 0; i < 16; i++) if (rnd() < 0.6) g.fillRect(gx + i, gy + 4, 1, 1 + Math.floor(rnd() * 2));
  tile(2, "#7a5534", [["#6a4a2e", 34], ["#8a6440", 22], ["#5c3f27", 8, 2, 1]]);
  tile(3, "#c9a865", [["#b8955a", 36], ["#d6b878", 26]]);
  tile(4, "#8a8274", [["#6e675c", 30, 2, 2], ["#a39a8a", 22, 2, 1], ["#5e584f", 12]]);
  tile(5, "#5a4632", [["#4a3a2a", 40], ["#665140", 20, 2, 1], ["#3f3124", 10]]);
  const [sx, sy] = tile(6, "#7d8288", [["#6b7076", 30, 2, 2], ["#8f949a", 26], ["#5d6166", 10, 3, 1]]);
  g.fillStyle = "#62676c"; g.fillRect(sx, sy + 7, 9, 1); g.fillRect(sx + 9, sy + 12, 7, 1);
  // bark: vertical furrows; log end: rings
  const [bx, by] = tile(7, "#6b4a2c", [["#5a3d24", 18, 1, 3], ["#7c5735", 14, 1, 2]]);
  g.fillStyle = "#4e3520"; for (let i = 0; i < 16; i += 3) g.fillRect(bx + i, by, 1, 16);
  const [lx, ly] = tile(8, "#c8a06a", []);
  g.fillStyle = "#a9814f"; g.fillRect(lx + 2, ly + 2, 12, 12); g.fillStyle = "#c8a06a"; g.fillRect(lx + 4, ly + 4, 8, 8);
  g.fillStyle = "#a9814f"; g.fillRect(lx + 6, ly + 6, 4, 4); g.fillStyle = "#6b4a2c"; g.fillRect(lx, ly, 16, 1); g.fillRect(lx, ly + 15, 16, 1); g.fillRect(lx, ly, 1, 16); g.fillRect(lx + 15, ly, 1, 16);
  const [px, py] = tile(9, "#a57a48", [["#946b3c", 26, 3, 1], ["#b68a55", 16, 2, 1]]);
  g.fillStyle = "#7c5833"; for (let i = 0; i < 16; i += 4) g.fillRect(px, py + i, 16, 1);
  const [dx, dy] = tile(10, "#6e4f30", [["#5f4329", 26, 3, 1], ["#7d5b39", 14, 2, 1]]);
  g.fillStyle = "#4f3820"; for (let i = 0; i < 16; i += 4) g.fillRect(dx, dy + i, 16, 1);
  tile(11, "#4a4d52", [["#3c3f43", 26], ["#5a5e63", 16]]);
  const [rx, ry] = tile(12, "#b59a68", []);
  g.fillStyle = "#9a7f52"; for (let i = 0; i < 16; i += 2) g.fillRect(rx + ((i * 3) % 16), ry + i, 3, 1);
  // net: twine squares with open holes (alpha)
  const [nx, ny] = [(13 % 8) * 16, 16];
  g.clearRect(nx, ny, 16, 16); g.fillStyle = "#8c8a72"; for (let i = 0; i < 16; i += 4) { g.fillRect(nx + i, ny, 1, 16); g.fillRect(nx, ny + i, 16, 1); }
  tile(14, "#c9bc98", [["#b8ab88", 26], ["#d6caa8", 12]]);
  const [kx, ky] = tile(15, "#9c7444", [["#8a6438", 20, 2, 1]]);
  g.fillStyle = "#6e4f2c"; g.fillRect(kx, ky, 16, 2); g.fillRect(kx, ky + 14, 16, 2); g.fillRect(kx, ky, 2, 16); g.fillRect(kx + 14, ky, 2, 16); g.fillRect(kx + 7, ky, 2, 16);
  return c;
}
RC.atlas = function () {
  const t = new THREE.CanvasTexture(atlasCanvas());
  t.magFilter = t.minFilter = THREE.NearestFilter; t.generateMipmaps = false; t.flipY = false;
  return t;
};

// ---------------- the lights, shared by every material ----------------
RC.LAMPS = 8;
RC.uniforms = function () {
  return {
    uSunDir: {value: new THREE.Vector3(0.5, 0.75, -0.4).normalize()}, uSunCol: {value: new THREE.Vector3(1, 0.96, 0.9)},
    uAmb: {value: new THREE.Vector3(0.3, 0.34, 0.4)}, uNight: {value: 0}, uFog: {value: new THREE.Vector3(0.6, 0.7, 0.8)},
    uLampPos: {value: Array.from({length: RC.LAMPS}, () => new THREE.Vector3(0, -999, 0))},
    uLampCol: {value: Array.from({length: RC.LAMPS}, () => new THREE.Vector3())},
    uTime: {value: 0}, uAtlas: {value: null},
  };
};
const LIGHT_GLSL = `
uniform vec3 uSunDir; uniform vec3 uSunCol; uniform vec3 uAmb; uniform float uNight; uniform vec3 uFog;
uniform vec3 uLampPos[${RC.LAMPS}]; uniform vec3 uLampCol[${RC.LAMPS}];
float lampFall(float d, float range) { return pow(clamp(1.0 - pow(d/range, 4.0), 0.0, 1.0), 2.0)/(1.0 + 0.30*d*d); }
vec3 lampLight(vec3 p, vec3 n) {
  vec3 e = vec3(0.0);
  for (int i = 0; i < ${RC.LAMPS}; i++) {
    vec3 l = uLampPos[i] - p; float d = length(l); l /= max(d, 1e-4);
    e += uLampCol[i]*lampFall(d, 15.0)*clamp((dot(n, l) + 0.35)/1.35, 0.0, 1.0);
  }
  return e;
}
vec3 skyRad(vec3 d) {
  float h = clamp(d.y, -0.2, 1.0);
  vec3 day = mix(vec3(0.62, 0.72, 0.82), vec3(0.24, 0.42, 0.74), smoothstep(0.0, 0.7, h));
  vec3 nite = mix(vec3(0.012, 0.016, 0.030), vec3(0.003, 0.005, 0.014), smoothstep(0.0, 0.7, h));
  return mix(day, nite, uNight);
}
vec3 toSrgb(vec3 c) { c = c/(1.0 + 0.08*c); return pow(clamp(c, 0.0, 1.0), vec3(1.0/2.2)); }
vec3 fromSrgb(vec3 c) { return pow(c, vec3(2.2)); }
vec3 fogged(vec3 c, vec3 p) { float d = length(cameraPosition - p); return mix(c, uFog, 1.0 - exp(-d*mix(0.0035, 0.012, uNight))); }`;
RC.LIGHT_GLSL = LIGHT_GLSL;

// The lit material: the atlas tile at the vertex (aTile), world-space or face UVs, the sun, the sky, the lamps.
// Underwater fragments (below aWY, the water's level over them) fade toward the river's own colour.
RC.litMaterial = function (U, o) {
  o = o || {};
  return new THREE.ShaderMaterial({
    uniforms: Object.assign({uTint: {value: new THREE.Color(o.tint || 0xffffff)}, uEmit: {value: 0}}, U),
    side: o.side || THREE.FrontSide, transparent: false,
    vertexShader: `attribute float aTile; attribute float aWY; varying vec2 vUv; varying float vTile; varying vec3 vN; varying vec3 vP; varying float vWY;
      void main() { vUv = uv; vTile = aTile; vWY = aWY; vec4 w = modelMatrix*vec4(position, 1.0); vP = w.xyz;
        vN = normalize(mat3(modelMatrix)*normal); gl_Position = projectionMatrix*viewMatrix*w; }`,
    fragmentShader: `uniform sampler2D uAtlas; uniform vec3 uTint; uniform float uEmit; varying vec2 vUv; varying float vTile; varying vec3 vN; varying vec3 vP; varying float vWY;
      ${LIGHT_GLSL}
      void main() {
        vec2 t = vec2(mod(vTile, 8.0), floor(vTile/8.0));
        vec2 f = clamp(fract(vUv), 0.002, 0.998);
        vec4 tx = texture2D(uAtlas, (t + f)/vec2(8.0, 2.0));
        if (tx.a < 0.5) discard;
        vec3 alb = fromSrgb(tx.rgb)*uTint;
        vec3 n = normalize(gl_FrontFacing ? vN : -vN);
        vec3 L = uAmb*(0.55 + 0.45*n.y) + uSunCol*max(dot(n, uSunDir), 0.0) + lampLight(vP, n);
        vec3 c = alb*L + alb*uEmit;
        if (vP.y < vWY) {
          float d = vWY - vP.y;
          vec3 wl = uAmb + uSunCol*max(uSunDir.y, 0.0)*0.6 + lampLight(vec3(vP.x, vWY, vP.z), vec3(0.0, 1.0, 0.0));
          c = mix(vec3(0.02, 0.07, 0.065)*wl, c, exp(-d*vec3(0.55, 0.22, 0.16)));
        }
        gl_FragColor = vec4(toSrgb(fogged(c, vP)), 1.0);
      }`,
  });
};

// ---------------- geometry helpers ----------------
// Give a three.js geometry the lit material's attributes: one tile everywhere, no water over it, its UVs scaled
// so a tile covers a metre (box faces get their own sizes).
RC.tiled = function (geo, tile, sx, sy) {
  const n = geo.attributes.position.count;
  geo.setAttribute("aTile", new THREE.Float32BufferAttribute(new Float32Array(n).fill(RC.TILE[tile] !== undefined ? RC.TILE[tile] : tile), 1));
  geo.setAttribute("aWY", new THREE.Float32BufferAttribute(new Float32Array(n).fill(-99), 1));
  if (sx || sy) { const uv = geo.attributes.uv; for (let i = 0; i < uv.count; i++) uv.setXY(i, uv.getX(i) * (sx || 1), uv.getY(i) * (sy || 1)); }
  return geo;
};
RC.box = function (w, h, d, tile) {
  const g = new THREE.BoxGeometry(w, h, d), uv = g.attributes.uv;
  const sz = [[d, h], [d, h], [w, d], [w, d], [w, h], [w, h]];
  for (let f = 0; f < 6; f++) for (let k = 0; k < 4; k++) { const i = f * 4 + k; uv.setXY(i, uv.getX(i) * sz[f][0], uv.getY(i) * sz[f][1]); }
  return RC.tiled(g, tile);
};
// A log: a six-sided prism along x (bark round, rings on the ends).
RC.logGeo = function (len, rad) {
  const g = new THREE.CylinderGeometry(rad, rad, len, 6, 1, false);
  g.rotateZ(Math.PI / 2);
  const n = g.attributes.position.count, tiles = new Float32Array(n), uv = g.attributes.uv, nor = g.attributes.normal;
  for (let i = 0; i < n; i++) {
    const cap = Math.abs(nor.getX(i)) > 0.9;
    tiles[i] = cap ? RC.TILE.logEnd : RC.TILE.bark;
    if (!cap) uv.setXY(i, uv.getX(i) * 6 * rad * 1.1, uv.getY(i) * len);
    else uv.setXY(i, uv.getX(i) * 0.999, uv.getY(i) * 0.999);
  }
  g.setAttribute("aTile", new THREE.Float32BufferAttribute(tiles, 1));
  g.setAttribute("aWY", new THREE.Float32BufferAttribute(new Float32Array(n).fill(-99), 1));
  return g;
};
// A boat's hull: a pointed bow, a square stern, sides that close in toward the keel. Length along x (bow at +x),
// the waterline at y = 0, `free` its freeboard and `draft` below.
RC.hullGeo = function (L, B, draft, free, bowLen, tile) {
  const pos = [], uvs = [];
  const sec = []; const N = 8;
  for (let i = 0; i <= N; i++) {
    const x = -L / 2 + L * i / N, bx = L / 2 - bowLen;
    const half = x <= bx ? B / 2 : B / 2 * Math.max(0.02, 1 - ((x - bx) / bowLen) ** 1.6);
    sec.push([x, half]);
  }
  const yT = free, yB = -draft;
  const quad = (a, b, c, d, ua, ub) => { pos.push(...a, ...b, ...c, ...a, ...c, ...d); uvs.push(ua[0], ua[1], ub[0], ua[1], ub[0], ub[1], ua[0], ua[1], ub[0], ub[1], ua[0], ub[1]); };
  for (let i = 0; i < N; i++) {
    const [x0, h0] = sec[i], [x1, h1] = sec[i + 1];
    quad([x0, yB, h0 * 0.6], [x1, yB, h1 * 0.6], [x1, yT, h1], [x0, yT, h0], [x0, yB], [x1, yT]);          // starboard
    quad([x1, yB, -h1 * 0.6], [x0, yB, -h0 * 0.6], [x0, yT, -h0], [x1, yT, -h1], [x1, yB], [x0, yT]);      // port
    quad([x0, yB, -h0 * 0.6], [x1, yB, -h1 * 0.6], [x1, yB, h1 * 0.6], [x0, yB, h0 * 0.6], [x0, -h0], [x1, h0]);   // bottom
    quad([x0, yT, h0], [x1, yT, h1], [x1, yT, -h1], [x0, yT, -h0], [x0, h0], [x1, -h0]);                     // deck
  }
  const [xs, hs] = sec[0];
  quad([xs, yB, -hs * 0.6], [xs, yB, hs * 0.6], [xs, yT, hs], [xs, yT, -hs], [-hs, yB], [hs, yT]);         // transom
  const g = new THREE.BufferGeometry();
  g.setAttribute("position", new THREE.Float32BufferAttribute(pos, 3));
  g.setAttribute("uv", new THREE.Float32BufferAttribute(uvs, 2));
  g.computeVertexNormals();
  return RC.tiled(g, tile || "planks");
};
// A rope or chain drawn through points: a four-sided tube rebuilt in place each frame.
RC.Tube = function (n, rad, mat) {
  this.n = n; this.rad = rad;
  const pos = new Float32Array(n * 4 * 3), idx = [];
  for (let i = 0; i < n - 1; i++) for (let k = 0; k < 4; k++) {
    const a = i * 4 + k, b = i * 4 + (k + 1) % 4, c = a + 4, d = b + 4; idx.push(a, b, c, b, d, c);
  }
  const g = new THREE.BufferGeometry();
  g.setAttribute("position", new THREE.BufferAttribute(pos, 3));
  g.setAttribute("normal", new THREE.BufferAttribute(new Float32Array(n * 12), 3));
  const uv = new Float32Array(n * 8); for (let i = 0; i < n; i++) for (let k = 0; k < 4; k++) { uv[(i * 4 + k) * 2] = k * 0.25; uv[(i * 4 + k) * 2 + 1] = i * 0.3; }
  g.setAttribute("uv", new THREE.BufferAttribute(uv, 2));
  RC.tiled(g, "rope"); g.setIndex(idx);
  this.mesh = new THREE.Mesh(g, mat); this.mesh.frustumCulled = false;
};
RC.Tube.prototype.set = function (pts) {   // pts: flat [x, y, z, ...] of n points
  const P = this.mesh.geometry.attributes.position, Nn = this.mesh.geometry.attributes.normal, n = this.n, r = this.rad;
  for (let i = 0; i < n; i++) {
    const a = Math.max(0, i - 1), b = Math.min(n - 1, i + 1);
    let tx = pts[b * 3] - pts[a * 3], ty = pts[b * 3 + 1] - pts[a * 3 + 1], tz = pts[b * 3 + 2] - pts[a * 3 + 2];
    const tl = Math.hypot(tx, ty, tz) || 1; tx /= tl; ty /= tl; tz /= tl;
    let ux = -tz, uy = 0, uz = tx; let ul = Math.hypot(ux, uz); if (ul < 1e-3) { ux = 1; uz = 0; ul = 1; } ux /= ul; uz /= ul;
    const vx = ty * uz - tz * uy, vy = tz * ux - tx * uz, vz = tx * uy - ty * ux;
    for (let k = 0; k < 4; k++) {
      const an = k * Math.PI / 2, c = Math.cos(an), s = Math.sin(an);
      const nx = ux * c + vx * s, ny = uy * c + vy * s, nz = uz * c + vz * s, j = i * 4 + k;
      P.setXYZ(j, pts[i * 3] + nx * r, pts[i * 3 + 1] + ny * r, pts[i * 3 + 2] + nz * r); Nn.setXYZ(j, nx, ny, nz);
    }
  }
  P.needsUpdate = true; Nn.needsUpdate = true;
};

// ---------------- the ground: hex columns ----------------
RC.groundMesh = function (river, U) {
  const pos = [], nor = [], uvs = [], til = [], wy = [];
  const push = (p, n, uv, t, w) => { pos.push(...p); nor.push(...n); uvs.push(...uv); til.push(t); wy.push(w); };
  const BOT = -7;
  for (const k of river.cells.values()) {
    const w = k.kind === "bed" ? k.lv : -99;
    const T = RC.TILE[k.tile];
    const corners = []; for (let i = 0; i < 6; i++) { const a = Math.PI / 180 * (60 * i - 30); corners.push([k.x + HR * Math.cos(a), k.z + HR * Math.sin(a)]); }
    for (let i = 0; i < 6; i++) {
      const a = corners[i], b = corners[(i + 1) % 6];
      push([k.x, k.top, k.z], [0, 1, 0], [k.x * 0.7, k.z * 0.7], T, w);
      push([b[0], k.top, b[1]], [0, 1, 0], [b[0] * 0.7, b[1] * 0.7], T, w);
      push([a[0], k.top, a[1]], [0, 1, 0], [a[0] * 0.7, a[1] * 0.7], T, w);
    }
    for (let i = 0; i < 6; i++) {
      const [dq, dr] = DIRS[i], nb = river.cells.get(key(k.q + dq, k.r + dr));
      const nt = nb ? nb.top : BOT; if (nt >= k.top) continue;
      // side i lies between corners i and i+1 (the corner at 60 i - 30 and 60 i + 30)
      const a = corners[i], b = corners[(i + 1) % 6], n = [Math.cos(Math.PI / 3 * i), 0, Math.sin(Math.PI / 3 * i)];
      for (let y = nt; y < k.top; y++) {
        let t = k.kind === "bank" ? (y === k.top - 1 ? RC.TILE.grassSide : RC.TILE.dirt) : T;
        if (k.kind === "bank" && y < k.lv - 1.2) t = RC.TILE.stone;
        const ww = nb && nb.kind === "bed" ? nb.lv : w;
        push([a[0], y, a[1]], n, [0, 0], t, ww); push([b[0], y + 1, b[1]], n, [HR * 0.62, 0.999], t, ww); push([b[0], y, b[1]], n, [HR * 0.62, 0], t, ww);
        push([a[0], y, a[1]], n, [0, 0], t, ww); push([a[0], y + 1, a[1]], n, [0, 0.999], t, ww); push([b[0], y + 1, b[1]], n, [HR * 0.62, 0.999], t, ww);
      }
    }
  }
  const g = new THREE.BufferGeometry();
  g.setAttribute("position", new THREE.Float32BufferAttribute(pos, 3));
  g.setAttribute("normal", new THREE.Float32BufferAttribute(nor, 3));
  g.setAttribute("uv", new THREE.Float32BufferAttribute(uvs, 2));
  g.setAttribute("aTile", new THREE.Float32BufferAttribute(til, 1));
  g.setAttribute("aWY", new THREE.Float32BufferAttribute(wy, 1));
  const m = new THREE.Mesh(g, RC.litMaterial(U)); m.frustumCulled = false;
  return m;
};

// ---------------- the river's water ----------------
// A ribbon that follows the channel. Each vertex carries the flow (so the ripples move with the current), the
// depth to the bed (how much of the bed shows) and the foam (fast water: the rapid, the fall, round rocks).
RC.waterMesh = function (river, U) {
  const c = river.cfg, pos = [], flow = [], dep = [], foam = [], idx = [], out = [0, 0];
  const NZ = 26; let rows = 0;
  for (let x = c.x0; x <= c.x1 + 1e-6; x += 0.7) {
    const w = c.half(x) + 2.2, zc = c.center(x);
    for (let j = 0; j <= NZ; j++) {
      const z = zc - w + 2 * w * j / NZ, y = river.surf(x, z);
      pos.push(x, y, z); river.flow(x, z, out); flow.push(out[0], out[1]);
      dep.push(Math.max(0, y - river.top(x, z)));
      let f = clamp((Math.hypot(out[0], out[1]) - 1.6) / 1.4, 0, 1);
      for (const s of c.steps || []) f = Math.max(f, Math.exp(-(((x - s.x - 1.5) / 2.8) ** 2)) * 0.95);
      for (const k of river.rocks) { const d = Math.hypot(x - k.x, z - k.z); f = Math.max(f, 0.6 * clamp(1 - (d - k.R) / 1.4, 0, 1)); }
      foam.push(f);
    }
    rows++;
  }
  for (let i = 0; i < rows - 1; i++) for (let j = 0; j < NZ; j++) {
    const a = i * (NZ + 1) + j, b = a + 1, d = a + NZ + 1, e = d + 1; idx.push(a, b, d, b, e, d);
  }
  const g = new THREE.BufferGeometry();
  g.setAttribute("position", new THREE.Float32BufferAttribute(pos, 3));
  g.setAttribute("aFlow", new THREE.Float32BufferAttribute(flow, 2));
  g.setAttribute("aDepth", new THREE.Float32BufferAttribute(dep, 1));
  g.setAttribute("aFoam", new THREE.Float32BufferAttribute(foam, 1));
  g.setIndex(idx);
  const mat = new THREE.ShaderMaterial({
    uniforms: U, transparent: true, depthWrite: false, side: THREE.DoubleSide,
    vertexShader: `attribute vec2 aFlow; attribute float aDepth; attribute float aFoam; varying vec2 vFlow; varying float vDepth; varying float vFoam; varying vec3 vP;
      void main() { vFlow = aFlow; vDepth = aDepth; vFoam = aFoam; vec4 w = modelMatrix*vec4(position, 1.0); vP = w.xyz; gl_Position = projectionMatrix*viewMatrix*w; }`,
    fragmentShader: `uniform float uTime; varying vec2 vFlow; varying float vDepth; varying float vFoam; varying vec3 vP;
      ${LIGHT_GLSL}
      // The game's water colours (assets/config/water.ron): deep and shallow, the foam's white.
      const vec3 DEEP = vec3(0.004, 0.018, 0.040), SHALLOW = vec3(0.020, 0.090, 0.075), FOAM = vec3(0.95, 0.97, 1.0);
      float hash(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7)))*43758.5453); }
      float vnoise(vec2 p) { vec2 i = floor(p), f = fract(p); f = f*f*(3.0 - 2.0*f);
        return mix(mix(hash(i), hash(i + vec2(1.0, 0.0)), f.x), mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), f.x), f.y); }
      float waves(vec2 p) { return vnoise(p*1.3)*0.62 + vnoise(p*3.1 + 7.0)*0.3; }
      vec2 grad(vec2 p) { float h = waves(p); return vec2(waves(p + vec2(0.07, 0.0)) - h, waves(p + vec2(0.0, 0.07)) - h)/0.07; }
      void main() {
        // flow-mapped ripples: two copies of the pattern carried along the current, each faded out as it restarts
        float T = 2.4, ph0 = fract(uTime/T), ph1 = fract(uTime/T + 0.5);
        vec2 p = vP.xz*0.8;
        vec2 g0 = grad(p - vFlow*ph0*T*0.8), g1 = grad(p + vec2(3.7, 1.3) - vFlow*ph1*T*0.8);
        float w0 = 1.0 - abs(2.0*ph0 - 1.0);
        vec2 g = g0*w0 + g1*(1.0 - w0);
        float speed = length(vFlow);
        float k = 0.07 + 0.12*clamp(speed/2.0, 0.0, 1.0);
        vec3 n = normalize(vec3(-g.x*k, 1.0, -g.y*k));
        vec3 v = normalize(cameraPosition - vP);
        float fres = 0.02 + 0.98*pow(1.0 - max(dot(v, n), 0.0), 5.0);
        vec3 refl = skyRad(reflect(-v, n));
        vec3 lit = uAmb + uSunCol*max(uSunDir.y, 0.0) + lampLight(vP, vec3(0.0, 1.0, 0.0));
        vec3 body = mix(SHALLOW, DEEP, 1.0 - exp(-vDepth*0.55))*lit;
        float fo = vFoam*(0.25 + 0.75*smoothstep(0.45, 0.75, waves(vP.xz*1.6 - vFlow*uTime*0.9)));
        vec3 c = mix(body, refl, fres);
        // glints: every lamp on the water near it, and the sun by day
        for (int i = 0; i < ${RC.LAMPS}; i++) {
          vec3 l = uLampPos[i] - vP; float d = length(l); l /= max(d, 1e-4);
          c += uLampCol[i]*pow(max(dot(n, normalize(l + v)), 0.0), 90.0)*lampFall(d, 26.0)*6.0;
        }
        c += uSunCol*pow(max(dot(n, normalize(uSunDir + v)), 0.0), 320.0)*0.22*(1.0 - uNight);
        c = mix(c, FOAM*lit*0.85, fo);
        float a = clamp(0.38 + 0.42*(1.0 - exp(-vDepth*0.9)) + fres*0.3 + fo, 0.0, 0.93);
        gl_FragColor = vec4(toSrgb(fogged(c, vP)), a);
      }`,
  });
  const m = new THREE.Mesh(g, mat); m.frustumCulled = false; m.renderOrder = 2;
  return m;
};

// Flow arrows: a chevron every 3 m in the channel, pointing with the flow, longer where it is faster.
RC.arrowsMesh = function (river) {
  const c = river.cfg, pos = [], out = [0, 0];
  for (let x = c.x0 + 2; x < c.x1 - 2; x += 3) {
    const w = c.half(x);
    for (let s = -w + 1.2; s <= w - 1.2; s += 3) {
      const z = c.center(x) + s; if (river.top(x, z) > river.level(x) - 0.2) continue;
      river.flow(x, z, out); const sp = Math.hypot(out[0], out[1]); if (sp < 0.04) continue;
      const fx = out[0] / sp, fz = out[1] / sp, L = 0.5 + Math.min(sp, 3) * 0.55, y = river.surf(x, z) + 0.06, wd = 0.28;
      pos.push(x + fx * L * 0.5, y, z + fz * L * 0.5, x - fx * L * 0.5 - fz * wd, y, z - fz * L * 0.5 + fx * wd, x - fx * L * 0.25, y, z - fz * L * 0.25);
      pos.push(x + fx * L * 0.5, y, z + fz * L * 0.5, x - fx * L * 0.25, y, z - fz * L * 0.25, x - fx * L * 0.5 + fz * wd, y, z - fz * L * 0.5 - fx * wd);
    }
  }
  const g = new THREE.BufferGeometry(); g.setAttribute("position", new THREE.Float32BufferAttribute(pos, 3));
  const m = new THREE.Mesh(g, new THREE.MeshBasicMaterial({color: 0xd8f0f4, transparent: true, opacity: 0.62, depthWrite: false, side: THREE.DoubleSide}));
  m.renderOrder = 3; m.visible = false; m.frustumCulled = false;
  return m;
};

// ---------------- lamps: a bank post, a boat's lantern ----------------
const LAMP_RGB = [1.0, 0.62, 0.30];
RC.glowTex = null;
function glowTexture() {
  if (RC.glowTex) return RC.glowTex;
  const c = document.createElement("canvas"); c.width = c.height = 64; const g = c.getContext("2d");
  const gr = g.createRadialGradient(32, 32, 0, 32, 32, 32); gr.addColorStop(0, "rgba(255,210,150,0.9)"); gr.addColorStop(0.25, "rgba(255,170,90,0.35)"); gr.addColorStop(1, "rgba(255,140,60,0)");
  g.fillStyle = gr; g.fillRect(0, 0, 64, 64); RC.glowTex = new THREE.CanvasTexture(c); return RC.glowTex;
}
// A lantern: an iron cage round a glass that shines at night. Returns a group whose userData.light is the point it lights from.
RC.lantern = function (U, scale) {
  const s = scale || 1, grp = new THREE.Group();
  const cage = new THREE.Mesh(RC.box(0.26 * s, 0.05 * s, 0.26 * s, "iron"), RC.litMaterial(U)); cage.position.y = 0.17 * s; grp.add(cage);
  const base = cage.clone(); base.position.y = -0.17 * s; grp.add(base);
  const glassMat = new THREE.MeshBasicMaterial({color: 0x3a3226});
  const glass = new THREE.Mesh(new THREE.BoxGeometry(0.2 * s, 0.3 * s, 0.2 * s), glassMat); grp.add(glass);
  const glow = new THREE.Sprite(new THREE.SpriteMaterial({map: glowTexture(), blending: THREE.AdditiveBlending, depthWrite: false, transparent: true}));
  glow.scale.set(1.6 * s, 1.6 * s, 1); glow.visible = false; glow.renderOrder = 5; grp.add(glow);
  grp.userData = {glass: glassMat, glow, warm: 1};
  return grp;
};
RC.lampPost = function (U, x, y, z) {
  const grp = new THREE.Group();
  const post = new THREE.Mesh(RC.box(0.16, 2.5, 0.16, "darkPlanks"), RC.litMaterial(U)); post.position.set(0, 1.25, 0); grp.add(post);
  const arm = new THREE.Mesh(RC.box(0.6, 0.1, 0.1, "darkPlanks"), RC.litMaterial(U)); arm.position.set(0.25, 2.45, 0); grp.add(arm);
  const l = RC.lantern(U); l.position.set(0.5, 2.15, 0); grp.add(l);
  grp.position.set(x, y, z); grp.userData.lantern = l;
  return grp;
};
// A person: blocky, coloured like a villager, a hand where the pole or oar goes.
RC.person = function (U, shirt) {
  const g = new THREE.Group(), m = c => RC.litMaterial(U, {tint: c});
  const legs = new THREE.Mesh(RC.box(0.24, 0.7, 0.34, "canvas"), m(0x4a5368)); legs.position.y = 0.35; g.add(legs);
  const body = new THREE.Mesh(RC.box(0.28, 0.62, 0.44, "canvas"), m(shirt || 0x9a5b3c)); body.position.y = 1.0; g.add(body);
  const head = new THREE.Mesh(RC.box(0.28, 0.28, 0.28, "canvas"), m(0xd9a77c)); head.position.y = 1.47; g.add(head);
  const hat = new THREE.Mesh(RC.box(0.36, 0.08, 0.36, "canvas"), m(0x6b5638)); hat.position.y = 1.64; g.add(hat);
  return g;
};

// ---------------- the stage: one renderer, the scene, the orbit camera and the clock ----------------
// def: {river (cfg), build(ctx), step(h), sync(), lamps() -> [[x, y, z, warmth]], follow() -> [x, y, z], home}
RC.Stage = function (canvas, def) {
  const renderer = new THREE.WebGLRenderer({canvas, antialias: false, preserveDrawingBuffer: true});
  const U = RC.uniforms(); U.uAtlas.value = RC.atlas();
  const scene = new THREE.Scene(), camera = new THREE.PerspectiveCamera(50, 1, 0.1, 500);
  const skyMat = new THREE.ShaderMaterial({side: THREE.BackSide, depthWrite: false, uniforms: Object.assign({uMoon: {value: new THREE.Vector3(-0.4, 0.5, 0.6).normalize()}}, U),
    vertexShader: `varying vec3 vD; void main() { vD = position; vec4 p = projectionMatrix*viewMatrix*vec4(position + cameraPosition, 1.0); gl_Position = p.xyww; }`,
    fragmentShader: `uniform vec3 uMoon; varying vec3 vD; ${LIGHT_GLSL}
      float h2(vec3 p) { return fract(sin(dot(p, vec3(12.9898, 78.233, 37.719)))*43758.5453); }
      void main() { vec3 d = normalize(vD); vec3 c = skyRad(d);
        c += uSunCol*pow(max(dot(d, uSunDir), 0.0), 500.0)*3.0*(1.0 - uNight);
        float m = smoothstep(0.9985, 0.9990, dot(d, uMoon)); c += vec3(0.30, 0.32, 0.36)*m*uNight;
        vec3 q = floor(d*180.0); float s = step(0.9975, h2(q))*smoothstep(0.05, 0.3, d.y); c += vec3(0.20)*s*uNight;
        gl_FragColor = vec4(toSrgb(c), 1.0); }`});
  const sky = new THREE.Mesh(new THREE.SphereGeometry(400, 24, 12), skyMat); sky.frustumCulled = false; sky.renderOrder = -1; scene.add(sky);
  const S = {night: false, arrows: false, follow: !!def.followDefault, playing: false, speed: 1, simT: 0, waterT: 0};
  const ctx = {U, scene, S, camera, stage: null};
  let river = null, ground = null, water = null, arrows = null;
  function setRiver(cfg) {
    for (const m of [ground, water, arrows]) if (m) { scene.remove(m); m.geometry.dispose(); }
    river = new RC.River(cfg); ctx.river = river;
    ground = RC.groundMesh(river, U); scene.add(ground);
    water = RC.waterMesh(river, U); scene.add(water);
    arrows = RC.arrowsMesh(river); arrows.visible = S.arrows; scene.add(arrows);
  }
  ctx.setRiver = setRiver;
  setRiver(def.river);
  const posts = (def.posts || []).map(([x, s]) => {
    const zc = river.cfg.center(x), z = zc + s * (river.cfg.half(x) + 1.4), t = river.top(x, z);
    const p = RC.lampPost(U, x, t, z); p.rotation.y = s > 0 ? Math.PI / 2 : -Math.PI / 2; scene.add(p); return p;
  });
  def.build(ctx);
  // ---- lights
  function lighting() {
    if (!S.night) { U.uSunDir.value.set(0.5, 0.75, -0.4).normalize(); U.uSunCol.value.set(1.0, 0.95, 0.88); U.uAmb.value.set(0.34, 0.38, 0.44); U.uFog.value.set(0.62, 0.72, 0.82); }
    else { U.uSunDir.value.set(-0.4, 0.5, 0.6).normalize(); U.uSunCol.value.set(0.016, 0.020, 0.036); U.uAmb.value.set(0.007, 0.009, 0.018); U.uFog.value.set(0.012, 0.016, 0.030); }
    U.uNight.value = S.night ? 1 : 0;
  }
  const _v = new THREE.Vector3();
  function lamps() {
    const L = [];
    for (const p of posts) { p.userData.lantern.getWorldPosition(_v); L.push([_v.x, _v.y, _v.z, 1, p.userData.lantern]); }
    for (const l of def.lanterns ? def.lanterns() : []) {
      if (!l.visible && l.parent) continue;
      // a lantern put out in the boat's menu: dark glass, no glow, lights nothing
      if (l.userData.out) { l.userData.glass.color.set(0x3a3226); l.userData.glow.visible = false; continue; }
      l.getWorldPosition(_v); L.push([_v.x, _v.y, _v.z, l.userData.warm, l]); }
    for (let i = 0; i < RC.LAMPS; i++) {
      const e = L[i];
      if (e && S.night) { U.uLampPos.value[i].set(e[0], e[1], e[2]); const w = e[3]; U.uLampCol.value[i].set(LAMP_RGB[0] * 2.4, (LAMP_RGB[1] + 0.3 * (1 - w)) * 2.4, (LAMP_RGB[2] + 0.55 * (1 - w)) * 2.4); }
      else { U.uLampPos.value[i].set(0, -999, 0); U.uLampCol.value[i].set(0, 0, 0); }
    }
    for (const e of L) { const u = e[4].userData; u.glass.color.set(S.night ? (u.warm < 1 ? 0xfff0d8 : 0xffd28a) : 0x3a3226); u.glow.visible = S.night; }
  }
  // ---- the camera: orbit about a target; drag to turn, wheel or pinch to come closer. Never turns on its own.
  const V = Object.assign({tx: 0, ty: 0, tz: 0, yaw: 0, pitch: -0.5, dist: 20}, def.home);
  function place() {
    if (def.camera && def.camera(camera, V)) return;   // a scene's own camera (the Rowing tab's first person)
    if (S.follow && def.follow) { const f = def.follow(); V.tx = f[0]; V.ty = f[1]; V.tz = f[2]; }
    const cp = Math.cos(V.pitch);
    camera.position.set(V.tx - cp * Math.cos(V.yaw) * V.dist, V.ty - Math.sin(V.pitch) * V.dist, V.tz - cp * Math.sin(V.yaw) * V.dist);
    camera.lookAt(V.tx, V.ty, V.tz);
  }
  const ptrs = new Map(); let pinch0 = 0;
  canvas.addEventListener("pointerdown", e => { canvas.setPointerCapture(e.pointerId); ptrs.set(e.pointerId, [e.clientX, e.clientY]); if (ptrs.size === 2) { const [a, b] = [...ptrs.values()]; pinch0 = Math.hypot(a[0] - b[0], a[1] - b[1]); } });
  canvas.addEventListener("pointermove", e => {
    if (!ptrs.has(e.pointerId)) return; const [px, py] = ptrs.get(e.pointerId); ptrs.set(e.pointerId, [e.clientX, e.clientY]);
    if (ptrs.size === 1 && def.look && def.look(e.clientX - px, e.clientY - py)) { /* the scene turned its own view */ }
    else if (ptrs.size === 1) { V.yaw += (e.clientX - px) * 0.006; V.pitch = clamp(V.pitch - (e.clientY - py) * 0.006, -1.45, -0.08); }
    else if (ptrs.size === 2) { const [a, b] = [...ptrs.values()]; const d = Math.hypot(a[0] - b[0], a[1] - b[1]); if (pinch0 > 0) V.dist = clamp(V.dist * pinch0 / d, 4, 90); pinch0 = d; }
    place(); dirty = true;
  });
  const up = e => { ptrs.delete(e.pointerId); pinch0 = 0; };
  canvas.addEventListener("pointerup", up); canvas.addEventListener("pointercancel", up);
  canvas.addEventListener("wheel", e => { e.preventDefault(); V.dist = clamp(V.dist * Math.exp(e.deltaY * 0.001), 4, 90); place(); }, {passive: false});
  // ---- the clock: a fixed physics step, semi-implicit Euler, run only while playing
  const H = 1 / 120;
  let acc = 0, raf = 0, last = 0, dirty = true;
  function advance(sec) { let n = Math.round(sec / H); while (n-- > 0) { def.step(H); S.simT += H; } }
  function frame(now) {
    raf = requestAnimationFrame(frame);
    const dt = Math.min(0.1, (now - last) / 1000); last = now;
    S.waterT += dt;
    if (S.playing) { acc += dt * S.speed; let n = 0; while (acc >= H && n < 24) { def.step(H); S.simT += H; acc -= H; n++; } if (n >= 24) acc = 0; }
    draw();
  }
  function draw() {
    def.sync(); place(); lamps();
    U.uTime.value = S.waterT;
    renderer.render(scene, camera);
    if (ctx.onFrame) ctx.onFrame();
  }
  function resize() {
    const w = canvas.clientWidth, h = canvas.clientHeight; if (!w || !h) return;
    renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 1.5)); renderer.setSize(w, h, false);
    camera.aspect = w / h; camera.updateProjectionMatrix();
  }
  lighting(); resize(); place();
  window.addEventListener("resize", resize);
  const st = {
    S, ctx, V, resize, draw, advance,
    apply() { lighting(); arrows.visible = S.arrows; },
    start() { if (!raf) { last = performance.now(); raf = requestAnimationFrame(frame); } },
    stop() { cancelAnimationFrame(raf); raf = 0; },
    home() { Object.assign(V, def.home); place(); },
    river: () => river, setRiver,
  };
  ctx.stage = st;
  return st;
};
})();
