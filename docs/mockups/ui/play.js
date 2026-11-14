// play.js: the Play tab of docs/mockups/ui.html. A small hillside of Tenebris in three.js r128 that
// you walk around in, with the whole Stone and Timber UI working over it (ui/game-ui.js). Hex columns
// at the game's cell (2.833 m flat to flat, 1 m layers, building/scene-kit.js), the game's atlas and
// settlement textures sampled nearest, the game's tree. One WebGL renderer, made when the tab first
// opens and stopped when another tab opens. Nothing moves on its own except what the world moves:
// the stream and the wheel it drives, fire, smoke, clouds in the wind, machines while they run.
'use strict';
(function () {
  const { GameUI, h, P, BLOCKS, ITEMS } = window.PBDUI;
  const PBD = window.PBD;
  const { W, R, ROW, par, nb, dvec, hash2 } = PBD;
  const NC = 30, NR = 34, WATER_Y = 0.82;
  const cx = (c, r) => W * (c + 0.5 * par(r)), cz = (c, r) => ROW * r;
  const corner = (c, r, k) => { const a = (60 * k - 30) * Math.PI / 180; return [cx(c, r) + R * Math.cos(a), cz(c, r) + R * Math.sin(a)]; };
  const inMap = (c, r) => c >= 0 && r >= 0 && c < NC && r < NR;
  const idx = (c, r) => c + r * NC;
  const smooth = (a, b, x) => { const t = Math.max(0, Math.min(1, (x - a) / (b - a))); return t * t * (3 - 2 * t); };
  const vnoise = (x, z) => { const xi = Math.floor(x), zi = Math.floor(z), fx = x - xi, fz = z - zi, u = fx * fx * (3 - 2 * fx), v = fz * fz * (3 - 2 * fz);
    const a = hash2(xi, zi), b = hash2(xi + 1, zi), c = hash2(xi, zi + 1), d = hash2(xi + 1, zi + 1); return (a * (1 - u) + b * u) * (1 - v) + (c * (1 - u) + d * u) * v; };

  let S = null; // everything the demo holds, made once

  // ------------------------------------------------------------------ the land
  const streamX = (z) => 22 + 6 * Math.sin(z / 13) + 3 * Math.sin(z / 5.3);
  const YARD = [50, 36], POND = [16, 70];
  function makeLand() {
    const TOP = new Int16Array(NC * NR), MAT = new Array(NC * NR).fill('grass'), WET = new Uint8Array(NC * NR);
    for (let r = 0; r < NR; r++) for (let c = 0; c < NC; c++) {
      const x = cx(c, r), z = cz(c, r), i = idx(c, r);
      const t = smooth(80, -4, z);
      let hh = 2 + 6.5 * Math.pow(t, 1.25) + 2.2 * (vnoise(x / 15, z / 15) - 0.5) + 1.2 * smooth(60, 85, x) * t;
      const dy = Math.hypot(x - YARD[0], z - YARD[1]);
      if (dy < 13) hh = hh * smooth(9, 13, dy) + 4 * (1 - smooth(9, 13, dy));
      const ds = Math.abs(x - streamX(z)), dp = Math.hypot(x - POND[0], z - POND[1]);
      if (ds < 3.3 || dp < 9) { hh = 0; WET[i] = 1; } else if (ds < 6 || dp < 12) { hh = Math.min(hh, 1 + Math.floor((Math.min(ds - 3.3, dp - 9)) / 1.6)); if (hh <= 1) MAT[i] = 'sand'; }
      TOP[i] = Math.max(0, Math.round(hh));
      if (TOP[i] >= 9) MAT[i] = 'stone';
    }
    // a path from the spawn by the pond, through the yard, up to the lookout
    const path = [[30, 72], [38, 58], [46, 46], [50, 36], [56, 22], [60, 10]];
    for (let r = 0; r < NR; r++) for (let c = 0; c < NC; c++) {
      const x = cx(c, r), z = cz(c, r);
      for (let k = 0; k + 1 < path.length; k++) {
        const [ax, az] = path[k], [bx, bz] = path[k + 1], vx = bx - ax, vz = bz - az, t = Math.max(0, Math.min(1, ((x - ax) * vx + (z - az) * vz) / (vx * vx + vz * vz)));
        if (Math.hypot(x - ax - t * vx, z - az - t * vz) < 1.8 && !WET[idx(c, r)]) MAT[idx(c, r)] = 'path';
      }
    }
    // the stream's ford: two stepping columns
    return { TOP, MAT, WET };
  }
  function cellAt(x, z) {
    const r0 = Math.round(z / ROW); let best = null, bd = 1e9;
    for (let r = r0 - 1; r <= r0 + 1; r++) { const c0 = Math.round(x / W - 0.5 * par(r)); for (let c = c0 - 1; c <= c0 + 1; c++) { if (!inMap(c, r)) continue; const d = Math.hypot(x - cx(c, r), z - cz(c, r)); if (d < bd) { bd = d; best = [c, r]; } } }
    return best;
  }

  // ------------------------------------------------------------------ materials
  const ATILE = { grass: [0, 0], grassside: [1, 0], dirt: [2, 0], stone: [3, 0], path: [3, 2], bark: [2, 1], leaves: [0, 2] };
  const TOPTEX = { grass: 'a:grass', path: 'a:path', sand: 'sand', stone: 'a:stone', dirt: 'a:dirt', cobble: 'cobble', plank: 'plank', brick: 'brick', log: 'a:bark' };
  const SIDETEX = { grass: 'a:grassside', path: 'a:dirt', sand: 'sand', stone: 'a:stone', dirt: 'a:dirt', cobble: 'cobble', plank: 'plank', brick: 'brick', log: 'a:bark' };
  const BLOCK_TOP = { grass: 'grass', dirt: 'dirt', cobble: 'cobble', sand: 'sand', planks: 'plank', log: 'log', brick: 'brick', snow: 'stone', stone: 'stone' };
  const TOP_ITEM = { grass: 'grass', path: 'dirt', sand: 'sand', stone: 'stone', dirt: 'dirt', cobble: 'cobble', plank: 'planks', brick: 'brick', log: 'log' };
  // PBD.patch for instanced meshes too: every lamp lights what is near it, grass included.
  function patchLit(m, inst) {
    const LU = PBD.LU, MAXL = PBD.MAXL;
    m.onBeforeCompile = (sh) => {
      Object.assign(sh.uniforms, LU);
      sh.vertexShader = sh.vertexShader.replace('#include <common>', '#include <common>\nvarying vec3 vWP;')
        .replace('#include <project_vertex>', '#include <project_vertex>\n' + (inst ? 'vWP = (modelMatrix * instanceMatrix * vec4(transformed, 1.0)).xyz;' : 'vWP = (modelMatrix * vec4(transformed, 1.0)).xyz;'));
      sh.fragmentShader = sh.fragmentShader.replace('#include <common>', `#include <common>
varying vec3 vWP; uniform vec4 uLP[${MAXL}]; uniform vec4 uLC[${MAXL}]; uniform int uLN; uniform float uLamp;`)
        .replace('#include <aomap_fragment>', `#include <aomap_fragment>
{ vec3 wn = normalize((vec4(normal, 0.0) * viewMatrix).xyz); vec3 blk = vec3(0.0);
  for (int i = 0; i < ${MAXL}; i++) { if (i >= uLN) break;
    vec3 d = uLP[i].xyz - vWP; float d2 = dot(d, d), R2 = uLP[i].w * uLP[i].w; if (d2 >= R2) continue;
    float nd = dot(wn, d * inversesqrt(max(d2, 1e-4))); if (nd < -0.1) continue;
    float f = 1.0 - d2 / R2; f *= f; blk += uLC[i].rgb * uLC[i].w * f * (0.3 + 0.7 * max(nd, 0.0)); }
  reflectedLight.indirectDiffuse += blk * uLamp * material.diffuseColor; }`);
    };
    m.customProgramCacheKey = () => 'pbd-lit-' + (inst ? 'i' : 'n') + (m.vertexColors ? 'c' : '');
    return m;
  }
  const MATS = {};
  function mat(name, o = {}) {
    const key = name + (o.vc ? ':vc' : '') + (o.inst ? ':i' : '');
    if (MATS[key]) return MATS[key];
    const map = name.startsWith('a:') ? PBD.atlas(PBD.SHEET.fields, ...ATILE[name.slice(2)]) : PBD.game(name);
    const m = new THREE.MeshStandardMaterial({ map, color: 0xffffff, roughness: 0.95, metalness: 0, vertexColors: !!o.vc, side: o.side || THREE.FrontSide, alphaTest: o.alphaTest || 0, transparent: false });
    return (MATS[key] = patchLit(m, o.inst));
  }

  // ------------------------------------------------------------------ the terrain mesh (vertex AO)
  function buildTerrain() {
    const L = S.land, B = {};
    const buf = (m) => B[m] || (B[m] = { p: [], n: [], u: [], c: [] });
    const quad = (m, pts, n, uvs, cols) => {
      // wind it to face along n (front faces only are drawn)
      const e1 = [pts[1][0] - pts[0][0], pts[1][1] - pts[0][1], pts[1][2] - pts[0][2]], e2 = [pts[2][0] - pts[0][0], pts[2][1] - pts[0][1], pts[2][2] - pts[0][2]];
      const cr = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
      if (cr[0] * n[0] + cr[1] * n[1] + cr[2] * n[2] < 0) { pts = pts.slice().reverse(); uvs = uvs.slice().reverse(); cols = cols.slice().reverse(); }
      const b = buf(m); for (const k of [0, 1, 2, 0, 2, 3]) { b.p.push(...pts[k]); b.n.push(...n); b.u.push(...uvs[k]); b.c.push(cols[k], cols[k], cols[k]); } };
    const fan = (m, pts, n, uvs, cols) => { const b = buf(m); for (let i = 1; i + 1 < pts.length; i++) for (const k of [0, i + 1, i]) { b.p.push(...pts[k]); b.n.push(...n); b.u.push(...uvs[k]); b.c.push(cols[k], cols[k], cols[k]); } };
    for (let r = 0; r < NR; r++) for (let c = 0; c < NC; c++) {
      const i = idx(c, r), top = L.TOP[i], x = cx(c, r), z = cz(c, r), mt = L.MAT[i];
      const hN = []; for (let d = 0; d < 6; d++) { const [c2, r2] = nb(c, r, d); hN.push(inMap(c2, r2) ? L.TOP[idx(c2, r2)] : top - 4); }
      // top: each corner darkened by the higher neighbours that share it (cheap ambient occlusion)
      const pts = [], uvs = [], cols = [], tex = TOPTEX[mt] || 'a:grass', uvf = tex.startsWith('a:') ? PBD.cellUV(x, z) : (p) => [p[0] / 2, p[2] / 2];
      for (let k = 0; k < 6; k++) {
        const [px, pz] = corner(c, r, k), p = [px, top, pz]; pts.push(p); uvs.push(uvf(p));
        const occ = (hN[(k + 5) % 6] > top ? 1 : 0) + (hN[k] > top ? 1 : 0); cols.push(1 - 0.22 * occ);
      }
      fan(tex, pts, [0, 1, 0], uvs, cols);
      for (let d = 0; d < 6; d++) {
        const h2 = Math.max(hN[d], -1); if (h2 >= top) continue;
        const A = corner(c, r, d), B2 = corner(c, r, d + 1), [nx, nz] = dvec(d);
        for (let y = top; y > h2 + 1e-3; y -= 1) {
          const y0 = Math.max(h2, y - 1), sm = y === top ? (SIDETEX[mt] || 'a:dirt') : (top - y >= 3 ? 'a:stone' : 'a:dirt'), uf = PBD.sideUV(A, B2, y);
          const P4 = [[A[0], y0, A[1]], [B2[0], y0, B2[1]], [B2[0], y, B2[1]], [A[0], y, A[1]]];
          const ao = (yy) => 0.55 + 0.45 * Math.min(1, (yy - h2) / 1.5);
          const uv4 = sm.startsWith('a:') ? P4.map(uf) : P4.map((p) => [(p[0] * -nz + p[2] * nx) / 2, p[1] / 2]);
          quad(sm, P4, [nx, 0, nz], uv4, P4.map((p) => ao(p[1])));
        }
      }
    }
    if (S.terrain) { S.terrain.forEach((m) => { S.scene.remove(m); m.geometry.dispose(); }); }
    S.terrain = [];
    for (const [m, b] of Object.entries(B)) {
      const g = new THREE.BufferGeometry();
      g.setAttribute('position', new THREE.Float32BufferAttribute(b.p, 3)); g.setAttribute('normal', new THREE.Float32BufferAttribute(b.n, 3));
      g.setAttribute('uv', new THREE.Float32BufferAttribute(b.u, 2)); g.setAttribute('color', new THREE.Float32BufferAttribute(b.c, 3));
      const mesh = new THREE.Mesh(g, mat(m, { vc: true })); mesh.castShadow = true; mesh.receiveShadow = true; mesh.userData.terrain = true;
      S.scene.add(mesh); S.terrain.push(mesh);
    }
  }

  // ------------------------------------------------------------------ water (the world moves it)
  function buildWater() {
    const p = [];
    for (let r = 0; r < NR; r++) for (let c = 0; c < NC; c++) {
      if (!S.land.WET[idx(c, r)]) continue;
      const q = []; for (let k = 0; k < 6; k++) { const a = (60 * k - 30) * Math.PI / 180; q.push([cx(c, r) + (R + 0.02) * Math.cos(a), WATER_Y, cz(c, r) + (R + 0.02) * Math.sin(a)]); }
      for (let i = 1; i < 5; i++) for (const k of [0, i + 1, i]) p.push(...q[k]);
    }
    const g = new THREE.BufferGeometry(); g.setAttribute('position', new THREE.Float32BufferAttribute(p, 3));
    S.waterU = { uT: { value: 0 }, uSun: { value: new THREE.Vector3() }, uSunC: { value: new THREE.Color() }, uSky: { value: new THREE.Color() }, uHor: { value: new THREE.Color() },
      uDeep: { value: new THREE.Color(0x0b2a3a) }, uFog: { value: new THREE.Color() }, uFogD: { value: new THREE.Vector2(60, 220) }, uLampW: { value: 0 } };
    const m = new THREE.ShaderMaterial({
      uniforms: S.waterU, transparent: true, depthWrite: false,
      vertexShader: 'varying vec3 vW; void main(){ vec4 w = modelMatrix * vec4(position,1.0); vW = w.xyz; gl_Position = projectionMatrix * viewMatrix * w; }',
      fragmentShader: `varying vec3 vW; uniform float uT; uniform vec3 uSun; uniform vec3 uSunC; uniform vec3 uSky; uniform vec3 uHor; uniform vec3 uDeep; uniform vec3 uFog; uniform vec2 uFogD; uniform float uLampW;
      void main(){
        vec2 p = vW.xz; float t = uT;
        vec3 n = normalize(vec3(0.06*sin(p.x*1.7 + t*1.3) + 0.04*sin(p.x*0.6 - p.y*1.1 + t*0.9) + 0.03*sin(p.y*2.9 - t*2.1), 1.0,
                                0.05*cos(p.y*1.5 - t*1.1) + 0.04*cos(p.x*1.3 + p.y*0.7 + t*1.7) + 0.02*cos(p.x*3.1 + t*2.3)));
        vec3 v = normalize(cameraPosition - vW); float fres = 0.04 + 0.96 * pow(1.0 - max(dot(n, v), 0.0), 5.0);
        vec3 r = reflect(-v, n); vec3 sky = mix(uHor, uSky, clamp(r.y * 2.0, 0.0, 1.0));
        float spec = pow(max(dot(r, normalize(uSun)), 0.0), 220.0) * step(0.0, uSun.y);
        vec3 col = mix(uDeep, sky, fres * 0.7) + uSunC * spec * 3.0 + vec3(1.0, 0.7, 0.4) * uLampW * 0.04 * (0.5 + 0.5*sin(p.x*3.0+t));
        float f = smoothstep(uFogD.x, uFogD.y, length(cameraPosition - vW));
        gl_FragColor = vec4(mix(col, uFog, f), mix(0.86, 1.0, fres));
        #include <tonemapping_fragment>
        #include <encodings_fragment>
      }`,
    });
    S.water = new THREE.Mesh(g, m); S.water.renderOrder = 2; S.scene.add(S.water);
  }

  // ------------------------------------------------------------------ sky, sun, moon, stars, clouds
  function buildSky() {
    S.skyU = { uTop: { value: new THREE.Color() }, uHor: { value: new THREE.Color() }, uSun: { value: new THREE.Vector3() }, uSunC: { value: new THREE.Color() }, uMoon: { value: new THREE.Vector3() }, uNight: { value: 0 } };
    const sky = new THREE.Mesh(new THREE.SphereGeometry(400, 32, 16), new THREE.ShaderMaterial({
      uniforms: S.skyU, side: THREE.BackSide, depthWrite: false, fog: false,
      vertexShader: 'varying vec3 vD; void main(){ vD = normalize(position); vec4 p = projectionMatrix * modelViewMatrix * vec4(position,1.0); gl_Position = p.xyww; }',
      fragmentShader: `varying vec3 vD; uniform vec3 uTop; uniform vec3 uHor; uniform vec3 uSun; uniform vec3 uSunC; uniform vec3 uMoon; uniform float uNight;
      void main(){ vec3 d = normalize(vD); float y = max(d.y, 0.0);
        vec3 col = mix(uHor, uTop, pow(y, 0.55));
        if (d.y < 0.0) col = mix(uHor, uHor * 0.6, clamp(-d.y * 4.0, 0.0, 1.0));
        float sd = dot(d, normalize(uSun)); col += uSunC * (smoothstep(0.9993, 0.9997, sd) * 6.0 + pow(max(sd, 0.0), 12.0) * 0.35 + pow(max(sd, 0.0), 3.0) * 0.08);
        float md = dot(d, normalize(uMoon)); col += vec3(0.75, 0.8, 0.9) * smoothstep(0.9994, 0.9997, md) * uNight * 1.5;
        gl_FragColor = vec4(col, 1.0);
        #include <tonemapping_fragment>
        #include <encodings_fragment>
      }`,
    }));
    sky.renderOrder = -2; S.scene.add(sky); S.sky = sky;
    // stars
    const sp = []; for (let i = 0; i < 900; i++) { const u = hash2(i, 3), v = hash2(i, 9) * 0.95 + 0.05, th = u * Math.PI * 2, ph = Math.asin(v); sp.push(380 * Math.cos(ph) * Math.cos(th), 380 * Math.sin(ph), 380 * Math.cos(ph) * Math.sin(th)); }
    const sg = new THREE.BufferGeometry(); sg.setAttribute('position', new THREE.Float32BufferAttribute(sp, 3));
    S.stars = new THREE.Points(sg, new THREE.PointsMaterial({ color: 0xdfe8ff, size: 1.6, sizeAttenuation: false, transparent: true, opacity: 0, depthWrite: false, fog: false }));
    S.scene.add(S.stars);
    // clouds: soft billboards drifting with the wind
    const cv = document.createElement('canvas'); cv.width = cv.height = 128; const g = cv.getContext('2d');
    for (let i = 0; i < 14; i++) { const x = 30 + hash2(i, 1) * 68, y = 50 + hash2(i, 2) * 28, rr = 18 + hash2(i, 3) * 22; const gr = g.createRadialGradient(x, y, 0, x, y, rr); gr.addColorStop(0, 'rgba(255,255,255,0.55)'); gr.addColorStop(1, 'rgba(255,255,255,0)'); g.fillStyle = gr; g.fillRect(0, 0, 128, 128); }
    const ct = new THREE.CanvasTexture(cv);
    S.clouds = [];
    for (let i = 0; i < 9; i++) {
      const m = new THREE.SpriteMaterial({ map: ct, transparent: true, depthWrite: false, fog: false, color: 0xffffff });
      const s = new THREE.Sprite(m); const a = hash2(i, 5) * Math.PI * 2, d = 160 + hash2(i, 6) * 120;
      s.position.set(42 + Math.cos(a) * d, 55 + hash2(i, 7) * 35, 45 + Math.sin(a) * d); s.scale.set(90 + hash2(i, 8) * 70, 34 + hash2(i, 9) * 14, 1);
      S.scene.add(s); S.clouds.push(s);
    }
  }

  // ------------------------------------------------------------------ things in the world
  const GS = () => new PBD.GeoSet();
  function addGS(gs, cast = true) { return PBD.addMeshes(gs, S.scene, { mat: (k) => mat(k), cast: () => cast }); }
  const topAt = (x, z) => { const c = cellAt(x, z); return c ? S.land.TOP[idx(...c)] : 0; };
  function hexPts(x, z, s) { const p = []; for (let k = 0; k < 6; k++) { const a = (60 * k - 30) * Math.PI / 180; p.push([x + R * s * Math.cos(a), z + R * s * Math.sin(a)]); } return p; }
  function glowMat(col, e = 1) { return new THREE.MeshBasicMaterial({ color: new THREE.Color(col).multiplyScalar(e), fog: true }); }
  function buildWorld() {
    const L = S.land, gs = GS();
    S.solids = []; S.machines = []; S.lamps = []; S.anim = {};
    // trees on grass, away from the yard, path and water
    let id = 0;
    for (let r = 1; r < NR - 1; r++) for (let c = 1; c < NC - 1; c++) {
      const i = idx(c, r), x = cx(c, r), z = cz(c, r);
      if (L.MAT[i] !== 'grass' || L.WET[i] || Math.hypot(x - YARD[0], z - YARD[1]) < 15 || hash2(c * 3, r * 7) > 0.085) continue;
      if (Math.hypot(x - 30, z - 72) < 7) continue;
      PBD.tree(gs, hexPts(x, z, 1), x, z, L.TOP[i], id++, 'a:bark', 'a:leaves'); S.solids.push({ x, z, r: 0.35, y0: L.TOP[i], y1: L.TOP[i] + 6 });
    }
    const y = 4; // the yard's level
    // the workshop: fieldstone walls, a door to the south, a lit window east, a shingle gable
    const [wx, wz] = [YARD[0] + 4, YARD[1] - 7], WX = 8, WZ = 5.6, WH = 3, t = 0.5;
    const wall = (x0, z0, x1, z1, gaps) => {
      const L2 = Math.hypot(x1 - x0, z1 - z0), ang = Math.atan2(z1 - z0, x1 - x0), tx = (x1 - x0) / L2, tz = (z1 - z0) / L2;
      let a = 0; const segs = []; for (const [g0, g1, yb, ye] of gaps) { segs.push([a, g0, 0, WH]); segs.push([g0, g1, 0, yb]); if (ye < WH) segs.push([g0, g1, ye, WH]); a = g1; } segs.push([a, L2, 0, WH]);
      for (const [s0, s1, y0, y1] of segs) if (s1 - s0 > 0.01 && y1 - y0 > 0.01) gs.box('fieldstone', x0 + tx * (s0 + s1) / 2, y + y0, z0 + tz * (s0 + s1) / 2, s1 - s0, y1 - y0, t, ang);
    };
    const xa = wx - WX / 2, xb = wx + WX / 2, za = wz - WZ / 2, zb = wz + WZ / 2;
    wall(xa, zb, xb, zb, [[3.4, 4.6, 0, 2.2]]); wall(xa, za, xb, za, [[2.2, 3.2, 1.2, 2.2]]); wall(xa, za, xa, zb, []); wall(xb, za, xb, zb, [[2.2, 3.2, 1.1, 2.1]]);
    gs.box('plank', wx, y - 0.02, wz, WX, 0.1, WZ);
    for (const s of [-1, 1]) { // gable roof
      const ze = wz + s * (WZ / 2 + 0.5), pitch = 0.85, hw = WZ / 2 + 0.5, yr = y + WH + hw * pitch;
      gs.g('shingle').face([[xa - 0.4, y + WH - 0.2, ze], [xb + 0.4, y + WH - 0.2, ze], [xb + 0.4, yr, wz], [xa - 0.4, yr, wz]], [0, 1, s * 0.4], (p) => [p[0] / 2, Math.abs(p[2] - wz) / 1.5]);
      gs.g('plank').face([[xa - 0.4, y + WH - 0.2, ze], [xb + 0.4, y + WH - 0.2, ze], [xb + 0.4, yr, wz], [xa - 0.4, yr, wz]], [0, -1, 0]);
    }
    for (const xe of [xa, xb]) gs.g('fieldstone').face([[xe, y + WH, za], [xe, y + WH, zb], [xe, y + WH + WZ / 2 * 0.85, wz]], [xe < wx ? -1 : 1, 0, 0]);
    gs.box('stone', xa + 1.2, y + 2, za + 0.9, 0.8, 4.4, 0.8); S.chimney = [xa + 1.2, y + 6.5, za + 0.9];
    S.solids.push({ box: [xa - 0.3, za - 0.3, xb + 0.3, zb + 0.3], y0: y, y1: y + 6, door: [wx - 0.4, zb, 1.2] });
    S.windows = [{ x: xb + 0.27, y: y + 1.6, z: za + 2.7, nx: 1, nz: 0, w: 1, h: 1 }, { x: xa + 2.7, y: y + 1.7, z: za - 0.27, nx: 0, nz: -1, w: 1, h: 1 }];
    // a straw hut by the pond, its window lit
    const [hx, hz] = [POND[0] + 15, POND[1] - 9], hy = topAt(hx, hz);
    gs.prism(null, 'reed', hexPts(hx, hz, 1.05), hy, hy + 2.2);
    { const apex = [hx, hy + 4.6, hz], pts = hexPts(hx, hz, 1.35); for (let k = 0; k < 6; k++) { const a = pts[k], b = pts[(k + 1) % 6]; gs.g('thatch').face([[a[0], hy + 2.0, a[1]], [b[0], hy + 2.0, b[1]], apex], [(a[0] + b[0]) / 2 - hx, 0.6, (a[1] + b[1]) / 2 - hz], (p) => [(p[0] + p[2]) / 2, p[1] / 2]); } }
    S.solids.push({ x: hx, z: hz, r: R * 1.05, y0: hy, y1: hy + 4 });
    S.windows.push({ x: hx + R * 1.05 * Math.cos(Math.PI / 6) + 0.03, y: hy + 1.3, z: hz + 0.2, nx: 1, nz: 0, w: 0.7, h: 0.6, rot: Math.PI / 6 });
    // the lookout on the hilltop: four posts, a platform, a rail
    const [lx, lz] = [60, 8], ly = topAt(lx, lz);
    for (const [ox, oz] of [[-1, -1], [1, -1], [-1, 1], [1, 1]]) gs.box('timber', lx + ox * 1.1, ly, lz + oz * 1.1, 0.22, 4.6, 0.22);
    gs.box('plank', lx, ly + 3.4, lz, 2.8, 0.15, 2.8); for (const s of [-1, 1]) { gs.box('timber', lx, ly + 4.4, lz + s * 1.35, 2.8, 0.1, 0.1); gs.box('timber', lx + s * 1.35, ly + 4.4, lz, 0.1, 0.1, 2.8); }
    S.solids.push({ x: lx, z: lz, r: 1.6, y0: ly, y1: ly + 5 });
    // street lamps along the path
    for (const [px, pz] of [[34, 64], [44, 49], [57, 27], [40, 40]]) { const py = topAt(px, pz); PBD.lampPost(gs, px, py, pz); S.lamps.push({ x: px + 0.45, y: py + 2.25, z: pz, R: 6, c: [1.0, 0.72, 0.38], p: 2.6 }); S.solids.push({ x: px, z: pz, r: 0.25, y0: py, y1: py + 3 }); }
    addGS(gs);
    // window panes: warm glass, bright at night
    S.paneMat = new THREE.MeshBasicMaterial({ color: 0x2a2018 });
    for (const w of S.windows) {
      const m = new THREE.Mesh(new THREE.PlaneGeometry(w.w, w.h), S.paneMat); m.position.set(w.x, w.y, w.z); m.lookAt(w.x + (w.nx || 0), w.y, w.z + (w.nz || 0)); if (w.rot) m.rotateY(0); S.scene.add(m);
      S.lamps.push({ x: w.x + (w.nx || 0) * 0.8, y: w.y, z: w.z + (w.nz || 0) * 0.8, R: 5.5, c: [1.0, 0.68, 0.36], p: 2.6, window: true });
    }
    // lantern glass on the posts
    S.lanternMat = new THREE.MeshBasicMaterial({ color: 0x3a2a18 });
    for (const l of S.lamps.filter((q) => !q.window)) { const m = new THREE.Mesh(new THREE.BoxGeometry(0.2, 0.32, 0.2), S.lanternMat); m.position.set(l.x, l.y + 0.03, l.z); S.scene.add(m); }
    buildMachines(y);
  }
  function machine(name, kind, x, z, r, y) { const m = { name, kind, x, z, r, y }; S.machines.push(m); S.solids.push({ x, z, r: r * 0.7, y0: y, y1: y + 2 }); return m; }
  function buildMachines(y) {
    const [ox, oz] = YARD, gs = GS();
    // the bench (M3): a plank top on timber legs
    const bx = ox - 6, bz = oz + 2;
    gs.box('plank', bx, y + 0.9, bz, 2.2, 0.12, 0.9); for (const [a, b] of [[-1, -1], [1, -1], [-1, 1], [1, 1]]) gs.box('timber', bx + a * 0.95, y, bz + b * 0.35, 0.12, 0.9, 0.12);
    gs.box('iron', bx + 0.5, y + 1.02, bz, 0.3, 0.12, 0.2); machine('Bench', 'bench', bx, bz, 1.4, y);
    // the forge and its bellows
    const fx = ox + 1, fz = oz + 3;
    gs.box('fieldstone', fx, y, fz, 2.0, 1.1, 1.6); gs.box('fieldstone', fx, y + 1.1, fz - 0.55, 2.0, 0.6, 0.5); gs.box('stone', fx, y + 1.7, fz - 0.55, 0.7, 2.2, 0.5);
    S.forgeGlow = new THREE.Mesh(new THREE.BoxGeometry(1.2, 0.08, 0.9), glowMat(0xff6a1a, 1.2)); S.forgeGlow.position.set(fx, y + 1.13, fz + 0.15); S.scene.add(S.forgeGlow);
    S.bellows = new THREE.Group(); S.bellows.position.set(fx + 1.55, y + 0.55, fz + 0.2); S.scene.add(S.bellows);
    const bel = new THREE.Mesh(new THREE.BoxGeometry(0.9, 0.5, 0.7), mat('fig_leather' in {} ? 'boards' : 'boards')); bel.castShadow = true; S.bellows.add(bel);
    S.anim.bellows = bel;
    const fm = machine('Forge', 'forge', fx, fz, 1.6, y); fm.light = { x: fx, y: y + 1.6, z: fz + 0.6, R: 8, c: [1.0, 0.45, 0.15], p: 0.9 };
    // the cooking hearth (M27): a stone ring, a pot on an iron hook
    const hx = ox - 1, hz = oz - 5.2;
    gs.prism('cobble', 'cobble', hexPts(hx, hz, 0.45), y, y + 0.45);
    gs.box('iron', hx - 0.55, y, hz, 0.08, 1.6, 0.08); gs.box('iron', hx + 0.55, y, hz, 0.08, 1.6, 0.08); gs.box('iron', hx, y + 1.55, hz, 1.2, 0.06, 0.06);
    const pot = new THREE.Mesh(new THREE.CylinderGeometry(0.32, 0.26, 0.45, 8), mat('iron')); pot.position.set(hx, y + 1.0, hz); pot.castShadow = true; S.scene.add(pot);
    const hm = machine('Hearth', 'hearth', hx, hz, 1.2, y); hm.light = { x: hx, y: y + 0.8, z: hz, R: 6, c: [1.0, 0.55, 0.2], p: 0.7 }; S.hearth = [hx, y + 0.5, hz];
    // the campfire in the yard: stones, logs, fire
    const cxp = ox - 4, czp = oz - 1;
    for (let k = 0; k < 8; k++) { const a = k * Math.PI / 4; gs.box('cobble', cxp + Math.cos(a) * 0.65, y, czp + Math.sin(a) * 0.65, 0.3, 0.22, 0.3, a); }
    gs.box('a:bark', cxp, y, czp, 1.0, 0.16, 0.18, 0.5); gs.box('a:bark', cxp, y + 0.12, czp, 1.0, 0.16, 0.18, -0.6);
    S.fire = []; for (let k = 0; k < 3; k++) { const f = new THREE.Mesh(new THREE.ConeGeometry(0.22 - k * 0.05, 0.7 - k * 0.15, 6), glowMat(k ? 0xffc040 : 0xff6a20, 1.3)); f.position.set(cxp + (k - 1) * 0.12, y + 0.45, czp + (k % 2) * 0.1); S.scene.add(f); S.fire.push(f); }
    S.camp = { x: cxp, y: y + 0.6, z: czp, R: 7, c: [1.0, 0.55, 0.22], p: 3.0 }; S.solids.push({ x: cxp, z: czp, r: 0.7, y0: y, y1: y + 1 });
    // the press, its conveyor and the chest it feeds (M20)
    const px = ox + 6, pz = oz + 4;
    gs.box('timber', px - 0.55, y, pz, 0.18, 2.4, 0.18); gs.box('timber', px + 0.55, y, pz, 0.18, 2.4, 0.18); gs.box('timber', px, y + 2.3, pz, 1.3, 0.2, 0.3); gs.box('cobble', px, y, pz, 1.0, 0.5, 0.8);
    S.plunger = new THREE.Mesh(new THREE.BoxGeometry(0.7, 0.9, 0.5), mat('iron')); S.plunger.position.set(px, y + 1.75, pz); S.plunger.castShadow = true; S.scene.add(S.plunger);
    machine('Brick press', 'press', px, pz, 1.3, y);
    gs.box('boards', px + 2.2, y + 0.45, pz, 3.0, 0.1, 0.7); for (const a of [-1, 1]) gs.box('timber', px + 2.2 + a * 1.3, y, pz, 0.12, 0.45, 0.6);
    S.belt = { x0: px + 0.9, x1: px + 3.6, y: y + 0.62, z: pz, items: [] };
    for (let k = 0; k < 4; k++) { const b = new THREE.Mesh(new THREE.BoxGeometry(0.28, 0.16, 0.2), mat('brick')); b.visible = false; S.scene.add(b); S.belt.items.push(b); }
    const chx = px + 4.4; gs.box('plank', chx, y, pz, 1.0, 0.65, 0.7); gs.box('iron', chx, y + 0.2, pz, 1.04, 0.06, 0.74); gs.box('iron', chx, y + 0.5, pz, 1.04, 0.06, 0.74);
    machine('Chest', 'chest', chx, pz, 1.0, y);
    // the water mill on the stream (grain to flour); the stream turns its wheel
    const mz = 44, mx = streamX(mz) + 5.2, my = topAt(mx, mz);
    gs.box('fieldstone', mx, my, mz, 3.0, 2.6, 3.0); gs.box('thatch', mx, my + 2.6, mz, 3.4, 0.3, 3.4); gs.box('boards', mx, my + 2.9, mz, 2.4, 1.0, 2.4);
    S.wheel = new THREE.Group(); S.wheel.position.set(streamX(mz) + 1.6, WATER_Y + 1.0, mz); // in the stream, the water drives it
    gs.box('timber', (streamX(mz) + 1.6 + mx - 1.5) / 2, WATER_Y + 0.95, mz, mx - 1.5 - streamX(mz) - 1.6, 0.14, 0.14); S.scene.add(S.wheel);
    const axle = new THREE.Mesh(new THREE.CylinderGeometry(0.12, 0.12, 1.4, 8), mat('timber')); axle.rotation.x = Math.PI / 2; S.wheel.add(axle);
    for (const s of [-0.45, 0.45]) { const rim = new THREE.Mesh(new THREE.TorusGeometry(1.6, 0.08, 6, 20), mat('timber')); rim.position.z = s; S.wheel.add(rim); }
    for (let k = 0; k < 10; k++) { const a = k * Math.PI / 5, pad = new THREE.Mesh(new THREE.BoxGeometry(0.1, 0.6, 1.0), mat('plank')); pad.position.set(Math.cos(a) * 1.55, Math.sin(a) * 1.55, 0); pad.rotation.z = a; pad.castShadow = true; S.wheel.add(pad);
      const sp = new THREE.Mesh(new THREE.BoxGeometry(1.6, 0.07, 0.07), mat('timber')); sp.position.set(Math.cos(a) * 0.8, Math.sin(a) * 0.8, 0); sp.rotation.z = a; S.wheel.add(sp); }
    S.wheel.traverse((o) => { if (o.isMesh) o.castShadow = true; });
    machine('Water mill', 'mill', mx - 1.2, mz, 2.0, my);
    S.solids.push({ box: [mx - 1.6, mz - 1.6, mx + 1.6, mz + 1.6], y0: my, y1: my + 4 });
    addGS(gs);
    // Wren the smith, by the forge
    const fig = PBD.figure({ h: 1.0, b: 1.05, skin: 0xc89070, hair: 0x3a2a1a, tunic: 0x6a4a3a, legs: 0x3a3a40, apron: 0x5a3a22, hat: 'none', prop: 'hammer' });
    fig.g.position.set(ox + 3.2, y, oz + 1.2); fig.g.rotation.y = -2.3; S.scene.add(fig.g); S.smith = fig;
    machine('Wren the smith', 'talk', ox + 3.2, oz + 1.2, 1.0, y);
  }

  // ------------------------------------------------------------------ grass and flowers (instanced)
  function tuftTex(flower) {
    const cv = document.createElement('canvas'); cv.width = cv.height = 16; const g = cv.getContext('2d');
    const greens = ['#05582d', '#559642', '#9bb675'];
    for (let i = 0; i < 9; i++) { const x = 1 + Math.floor(hash2(i, flower ? 4 : 2) * 14), hgt = 5 + Math.floor(hash2(i, 5) * 9); for (let y = 0; y < hgt; y++) { g.fillStyle = greens[Math.min(2, Math.floor(y / 4))]; g.fillRect(x + (y > 6 && i % 2 ? 1 : 0), 15 - y, 1, 1); } }
    if (flower) for (let i = 0; i < 3; i++) { const x = 3 + i * 4, y = 3 + (i % 2) * 2; g.fillStyle = ['#faaf5c', '#fffefe', '#c34988'][i]; g.fillRect(x, y, 2, 2); g.fillStyle = '#fb902f'; g.fillRect(x, y + 1, 1, 1); }
    const t = new THREE.CanvasTexture(cv); t.magFilter = t.minFilter = THREE.NearestFilter; t.generateMipmaps = false; t.encoding = THREE.sRGBEncoding; return t;
  }
  function buildGrass() {
    const geo = new THREE.BufferGeometry(), p = [], u = [], n = [];
    for (const a of [0, Math.PI / 2]) { const c = Math.cos(a) * 0.4, s = Math.sin(a) * 0.4; const q = [[-c, 0, -s], [c, 0, s], [c, 0.6, s], [-c, 0.6, -s]], uv = [[0, 0], [1, 0], [1, 1], [0, 1]]; for (const k of [0, 1, 2, 0, 2, 3]) { p.push(...q[k]); u.push(...uv[k]); n.push(0, 1, 0); } }
    geo.setAttribute('position', new THREE.Float32BufferAttribute(p, 3)); geo.setAttribute('uv', new THREE.Float32BufferAttribute(u, 2)); geo.setAttribute('normal', new THREE.Float32BufferAttribute(n, 3));
    for (const flower of [false, true]) {
      const m = patchLit(new THREE.MeshStandardMaterial({ map: tuftTex(flower), alphaTest: 0.5, side: THREE.DoubleSide, roughness: 1 }), true);
      const spots = [];
      for (let r = 0; r < NR; r++) for (let c = 0; c < NC; c++) {
        const i = idx(c, r); if (S.land.MAT[i] !== 'grass' || S.land.WET[i]) continue;
        const k = flower ? (hash2(c, r * 3) < 0.25 ? 1 : 0) : (S.touch ? 2 : 4);
        for (let j = 0; j < k; j++) spots.push([cx(c, r) + (hash2(c * 7 + j, r) - 0.5) * 2.2, S.land.TOP[i], cz(c, r) + (hash2(c, r * 5 + j) - 0.5) * 2.2, hash2(c + j, r + j) * 6.28]);
      }
      const im = new THREE.InstancedMesh(geo, m, spots.length), M4 = new THREE.Matrix4(), q = new THREE.Quaternion(), sc = new THREE.Vector3();
      spots.forEach(([x, y, z, a], i) => { q.setFromAxisAngle(new THREE.Vector3(0, 1, 0), a); const k = 0.55 + hash2(i, 1) * 0.45; sc.set(k, k, k); M4.compose(new THREE.Vector3(x, y, z), q, sc); im.setMatrixAt(i, M4); });
      im.receiveShadow = true; S.scene.add(im); (S.grass = S.grass || []).push(im);
    }
  }

  // ------------------------------------------------------------------ particles: sparks, smoke, motes
  function softTex() { const cv = document.createElement('canvas'); cv.width = cv.height = 32; const g = cv.getContext('2d'); const gr = g.createRadialGradient(16, 16, 0, 16, 16, 16); gr.addColorStop(0, 'rgba(255,255,255,1)'); gr.addColorStop(1, 'rgba(255,255,255,0)'); g.fillStyle = gr; g.fillRect(0, 0, 32, 32); return new THREE.CanvasTexture(cv); }
  function buildParticles() {
    const N = 60, sp = new Float32Array(N * 3);
    S.sparks = { geo: new THREE.BufferGeometry(), life: new Float32Array(N), vel: new Float32Array(N * 3), N };
    S.sparks.geo.setAttribute('position', new THREE.BufferAttribute(sp, 3));
    S.scene.add(new THREE.Points(S.sparks.geo, new THREE.PointsMaterial({ color: 0xffa040, size: 0.09, transparent: true, blending: THREE.AdditiveBlending, depthWrite: false })));
    const st = softTex();
    S.smoke = [];
    for (let i = 0; i < 26; i++) { const m = new THREE.SpriteMaterial({ map: st, color: 0xb8b4ac, transparent: true, opacity: 0, depthWrite: false }); const s = new THREE.Sprite(m); s.userData = { life: hash2(i, 4) * 6, src: i % 2 }; S.scene.add(s); S.smoke.push(s); }
    const NM = 160, mp = new Float32Array(NM * 3);
    for (let i = 0; i < NM; i++) { mp[i * 3] = YARD[0] + (hash2(i, 1) - 0.5) * 40; mp[i * 3 + 1] = 4 + hash2(i, 2) * 6; mp[i * 3 + 2] = YARD[1] + (hash2(i, 3) - 0.5) * 40; }
    S.motes = new THREE.Points(new THREE.BufferGeometry().setAttribute('position', new THREE.BufferAttribute(mp, 3)), new THREE.PointsMaterial({ color: 0xfff0c0, size: 0.03, transparent: true, opacity: 0.5, depthWrite: false, blending: THREE.AdditiveBlending }));
    S.scene.add(S.motes);
  }

  // ------------------------------------------------------------------ time of day
  // colours are picked in sRGB and lit in linear light (the renderer writes sRGB)
  const lerpC = (a, b, t) => new THREE.Color(a).convertSRGBToLinear().lerp(new THREE.Color(b).convertSRGBToLinear(), t);
  function setTime(hours) {
    S.time = hours;
    const day = (hours - 6) / 12 * Math.PI, el = Math.sin(day), az = day; // east at 6, south at noon, west at 18
    const sun = new THREE.Vector3(Math.cos(az), Math.max(-1, el) * 0.95, 0.35).normalize();
    const e = Math.max(0, el), gold = 1 - smooth(0.0, 0.45, e), night = 1 - smooth(-0.18, 0.05, el);
    const top = lerpC(lerpC(0x3f7fd0, 0x3d6aa6, gold), 0x050a18, night), hor = lerpC(lerpC(0xbcd6ec, 0xf0c896, gold), 0x101a2c, night);
    S.skyU.uTop.value.copy(top); S.skyU.uHor.value.copy(hor); S.skyU.uSun.value.copy(sun); S.skyU.uSunC.value.copy(lerpC(0xfff2d8, 0xffb060, gold)).multiplyScalar(1 - night);
    const moon = sun.clone().multiplyScalar(-1); moon.y = Math.abs(moon.y) * 0.8 + 0.25; moon.normalize(); S.skyU.uMoon.value.copy(moon); S.skyU.uNight.value = night;
    S.stars.material.opacity = night * 0.9;
    // one shadow-casting light: the sun by day, the moon (dim, cold) by night
    const lit = night > 0.5 ? moon : sun;
    S.sunL.position.copy(lit).multiplyScalar(80).add(S.focus); S.sunL.target.position.copy(S.focus);
    S.sunL.color.copy(night > 0.5 ? new THREE.Color(0x8ea4d8).convertSRGBToLinear() : lerpC(0xfff4e0, 0xffc080, gold));
    S.sunL.intensity = night > 0.5 ? 0.05 : 2.6 * smooth(-0.02, 0.12, el);
    S.hemi.color.copy(lerpC(lerpC(0x9cc0ee, 0xa8b4d0, gold), 0x2a3a60, night)); S.hemi.groundColor.copy(lerpC(0x4a5a30, 0x05070c, night)); S.hemi.intensity = 0.9 - 0.82 * night;
    S.scene.fog.color.copy(hor); S.renderer.toneMappingExposure = 1.0 - 0.1 * night;
    for (const c of S.clouds) c.material.color.copy(lerpC(lerpC(0xffffff, 0xffc8a0, gold), 0x30384c, night));
    S.lampOn = smooth(0.25, -0.02, el); PBD.LU.uLamp.value = S.lampOn;
    S.paneMat.color.setRGB(0.2 + 0.8 * S.lampOn, 0.14 + 0.5 * S.lampOn, 0.08 + 0.2 * S.lampOn);
    S.lanternMat.color.setRGB(0.25 + 0.75 * S.lampOn, 0.18 + 0.55 * S.lampOn, 0.1 + 0.25 * S.lampOn);
    Object.assign(S.waterU, {}); S.waterU.uSun.value.copy(sun); S.waterU.uSunC.value.copy(S.skyU.uSunC.value); S.waterU.uSky.value.copy(top); S.waterU.uHor.value.copy(hor); S.waterU.uFog.value.copy(hor);
    S.waterU.uDeep.value.copy(lerpC(0x0d3346, 0x02070e, night)); S.waterU.uLampW.value = S.lampOn;
    S.motes.material.opacity = 0.45 * (1 - night) * (0.4 + gold);
    const v = document.getElementById('todVal'); if (v) v.textContent = `${String(Math.floor(hours)).padStart(2, '0')}:${String(Math.round((hours % 1) * 60) % 60).padStart(2, '0')}`;
  }

  // ------------------------------------------------------------------ the player
  const EYE = 1.62, RAD = 0.3;
  function blocked(x, z, feet) {
    for (let k = 0; k < 8; k++) {
      const a = k * Math.PI / 4, px = x + Math.cos(a) * RAD, pz = z + Math.sin(a) * RAD, c = cellAt(px, pz);
      if (!c) return true;
      const i = idx(...c);
      if (S.land.TOP[i] > feet + 0.05) return true;
    }
    for (const s of S.solids) {
      if (feet >= s.y1 || feet + 1.7 <= s.y0) continue;
      if (s.box) { const [x0, z0, x1, z1] = s.box; if (x > x0 - RAD && x < x1 + RAD && z > z0 - RAD && z < z1 + RAD) { if (s.door && Math.abs(x - s.door[0]) < s.door[2] / 2 && Math.abs(z - s.door[1]) < 0.8) continue; if (x > x0 + 0.6 && x < x1 - 0.6 && z > z0 + 0.6 && z < z1 - 0.6) continue; return true; } }
      else if (Math.hypot(x - s.x, z - s.z) < s.r + RAD) return true;
    }
    return false;
  }
  function groundAt(x, z) { const c = cellAt(x, z); return c ? S.land.TOP[idx(...c)] : 0; }
  function step(dt) {
    const pl = S.player, k = S.keys, ui = S.ui;
    if (!ui.menuOpen) {
      const f = (k.KeyW ? 1 : 0) - (k.KeyS ? 1 : 0) - S.joy.y, s = (k.KeyD ? 1 : 0) - (k.KeyA ? 1 : 0) + S.joy.x, sp = k.ShiftLeft ? 6.2 : 4.0;
      const fx = -Math.sin(pl.yaw), fz = -Math.cos(pl.yaw), rx = Math.cos(pl.yaw), rz = -Math.sin(pl.yaw);
      let vx = (fx * f + rx * s), vz = (fz * f + rz * s); const l = Math.hypot(vx, vz); if (l) { const m = Math.min(1, l); vx = vx / l * sp * m; vz = vz / l * sp * m; }
      const nx = pl.x + vx * dt, nz = pl.z + vz * dt;
      if (!blocked(nx, pl.z, pl.y)) pl.x = nx; if (!blocked(pl.x, nz, pl.y)) pl.z = nz;
      if (k.Space && pl.ground) { pl.vy = 7.2; pl.ground = false; }
    }
    pl.vy -= 22 * dt; pl.y += pl.vy * dt;
    const g = Math.max(groundAt(pl.x, pl.z), S.land.WET[idx(...(cellAt(pl.x, pl.z) || [0, 0]))] ? 0.2 : 0);
    if (pl.y <= g) { pl.y = g; pl.vy = 0; pl.ground = true; }
    S.camera.position.set(pl.x, pl.y + EYE, pl.z);
    S.camera.rotation.set(pl.pitch, pl.yaw, 0, 'YXZ');
  }

  // ------------------------------------------------------------------ dig and place
  const ray = new THREE.Raycaster();
  function aim() { S.camera.updateMatrixWorld(); ray.setFromCamera({ x: 0, y: 0 }, S.camera); ray.far = 7; const hit = ray.intersectObjects(S.terrain, false)[0]; return hit && hit.face ? hit : null; }
  function dig() {
    const hit = aim(); if (!hit) return;
    const n = hit.face.normal, p = hit.point.clone().addScaledVector(n, -0.15), c = cellAt(p.x, p.z); if (!c) return;
    const i = idx(...c); if (S.land.TOP[i] <= 0) return;
    const item = TOP_ITEM[S.land.MAT[i]] || 'dirt';
    S.land.TOP[i] -= 1; if (S.land.MAT[i] === 'grass') S.land.MAT[i] = 'dirt';
    S.ui.inv.add(item, 1); S.ui.toast(`+1 ${ITEMS[item].name}`); buildTerrain(); refreshGrass();
    S.stats.digs++;
  }
  function place() {
    const held = S.ui.inv.held(); if (!held || !BLOCKS.has(held[0])) { S.ui.toast('Hold a block to place it'); return; }
    const hit = aim(); if (!hit) return;
    const n = hit.face.normal; let c;
    if (n.y > 0.5) c = cellAt(hit.point.x - n.x * 0.1, hit.point.z - n.z * 0.1); else c = cellAt(hit.point.x + n.x * 0.6, hit.point.z + n.z * 0.6);
    if (!c) return;
    const i = idx(...c), me = cellAt(S.player.x, S.player.z);
    if (me && me[0] === c[0] && me[1] === c[1]) return;
    if (n.y <= 0.5 && S.land.TOP[i] >= hit.point.y) return;
    S.land.TOP[i] += 1; S.land.MAT[i] = BLOCK_TOP[held[0]] || 'dirt'; S.land.WET[i] = 0;
    held[1] -= 1; if (!held[1]) S.ui.inv.hot[S.ui.inv.sel] = null;
    buildTerrain(); S.ui.render();
  }
  function refreshGrass() { /* grass stays where it grew; a dug cell keeps its tufts until it is rebuilt */ }

  // ------------------------------------------------------------------ the loop
  function nearMachine() {
    const pl = S.player, fx = -Math.sin(pl.yaw), fz = -Math.cos(pl.yaw); let best = null, bd = 1e9;
    for (const m of S.machines) { const dx = m.x - pl.x, dz = m.z - pl.z, d = Math.hypot(dx, dz); if (d > m.r + 2.2) continue; if ((dx * fx + dz * fz) / (d || 1) < 0.35) continue; if (d < bd) { bd = d; best = m; } }
    return best;
  }
  function use(m) {
    if (!m) return;
    if (m.kind === 'talk') S.ui.go('talk', { talk: 'start' });
    else if (m.kind === 'chest') S.ui.go('chest');
    else S.ui.go('station', { station: m.kind, recipe: S.ui.st.station === m.kind ? S.ui.st.recipe : 0 });
    S.stats.used.push(m.kind);
  }
  function lightsList(t) {
    const L = [];
    for (const l of S.lamps) L.push({ x: l.x, y: l.y, z: l.z, R: l.R, c: new THREE.Color(...l.c), p: l.p });
    const fl = 0.85 + 0.15 * Math.sin(t * 13.1) * Math.sin(t * 7.3);
    L.push({ x: S.camp.x, y: S.camp.y, z: S.camp.z, R: S.camp.R, c: new THREE.Color(...S.camp.c), p: S.camp.p * fl / Math.max(0.15, S.lampOn) * Math.max(0.35, S.lampOn) });
    for (const m of S.machines) if (m.light) { const run = S.running[m.kind] ? 1 : 0; L.push({ x: m.light.x, y: m.light.y, z: m.light.z, R: m.light.R + run * 3, c: new THREE.Color(...m.light.c), p: (m.light.p + run * 1.6) * fl / Math.max(0.15, S.lampOn) * Math.max(0.35, S.lampOn) }); }
    return L;
  }
  let last = 0;
  function frame(now) {
    if (!S.on) return;
    S.raf = requestAnimationFrame(frame);
    const dt = Math.min(0.05, (now - (last || now)) / 1000); last = now; S.t += dt;
    tick(dt);
    S.renderer.render(S.scene, S.camera);
    S.frames++;
  }
  function tick(dt) {
    const t = S.t;
    step(dt);
    // the world moves these: water, the wheel the water drives, fire, smoke, clouds, machines at work
    S.waterU.uT.value = t;
    S.wheel.rotation.z -= dt * 0.9;
    S.fire.forEach((f, i) => { f.scale.y = 0.85 + 0.25 * Math.sin(t * (9 + i * 3) + i); f.scale.x = f.scale.z = 0.9 + 0.12 * Math.sin(t * 11 + i); });
    const fr = S.running.forge ? 1 : 0, gl = 0.75 + fr * 0.9; S.forgeGlow.material.color.setRGB(1.0 * gl, 0.42 * gl, 0.1 * gl);
    S.anim.bellows.scale.y = fr ? 0.55 + 0.45 * Math.abs(Math.sin(t * 3.2)) : 1;
    S.plunger.position.y = S.running.press ? 4 + 1.0 + 0.6 * Math.abs(Math.sin(t * 4)) + 0.15 : 4 + 1.75;
    S.belt.items.forEach((b, i) => { b.visible = !!S.running.press || S.beltT > 0; const u = ((t * 0.5 + i / 4) % 1); b.position.set(S.belt.x0 + (S.belt.x1 - S.belt.x0) * u, S.belt.y, S.belt.z); });
    if (!S.running.press) S.beltT = Math.max(0, (S.beltT || 0) - dt);
    for (const c of S.clouds) { c.position.x += dt * 0.8; if (c.position.x > 340) c.position.x -= 600; }
    // sparks from the fire (and the forge while it runs)
    const sg = S.sparks, pa = sg.geo.attributes.position.array;
    for (let i = 0; i < sg.N; i++) {
      sg.life[i] -= dt;
      if (sg.life[i] <= 0) { const atForge = S.running.forge && i % 2; const o = atForge ? [S.forgeGlow.position.x, S.forgeGlow.position.y + 0.1, S.forgeGlow.position.z] : [S.camp.x, S.camp.y, S.camp.z];
        pa[i * 3] = o[0] + (Math.random() - 0.5) * 0.4; pa[i * 3 + 1] = o[1]; pa[i * 3 + 2] = o[2] + (Math.random() - 0.5) * 0.4; sg.vel[i * 3] = (Math.random() - 0.5) * 0.6; sg.vel[i * 3 + 1] = 1.2 + Math.random() * 1.6; sg.vel[i * 3 + 2] = (Math.random() - 0.5) * 0.6; sg.life[i] = 0.6 + Math.random() * 1.1; }
      pa[i * 3] += sg.vel[i * 3] * dt; pa[i * 3 + 1] += sg.vel[i * 3 + 1] * dt; pa[i * 3 + 2] += sg.vel[i * 3 + 2] * dt;
    }
    sg.geo.attributes.position.needsUpdate = true;
    for (const s of S.smoke) {
      const u = s.userData; u.life += dt; const src = u.src === 0 ? S.chimney : [S.hearth[0], S.hearth[1] + 1.4, S.hearth[2]]; const L = 6;
      if (u.life > L) u.life -= L; const a = u.life / L;
      s.position.set(src[0] + a * 2.2 + Math.sin(u.life * 1.3 + u.src) * 0.2, src[1] + a * 5, src[2] + a * 0.8); s.scale.setScalar(0.6 + a * 2.6);
      s.material.opacity = (u.src === 1 && !S.running.hearth ? 0.12 : 0.32) * Math.sin(a * Math.PI) * (1 - S.lampOn * 0.6);
    }
    PBD.setLights(lightsList(t));
    // UI: the compass turns with the view, the prompt names what is in front
    const deg = ((-S.player.yaw * 180 / Math.PI) % 360 + 360) % 360;
    S.heading = deg;
    if (Math.abs(((deg - (S.lastDeg ?? -99)) + 540) % 360 - 180) > 0.4) {
      const dx = S.town[0] - S.player.x, dz = S.town[1] - S.player.z; S.ui.st.towns = [((Math.atan2(dx, -dz) * 180 / Math.PI) + 360) % 360];
      S.ui.setHeading(deg); S.lastDeg = deg;
    }
    const m = S.ui.menuOpen ? null : nearMachine(); S.near = m;
    S.ui.setPrompt(m ? `E: ${m.kind === 'talk' ? 'talk to' : 'use'} ${m.name}` : '');
  }

  // ------------------------------------------------------------------ build and input
  function build() {
    const box = document.getElementById('playScenes');
    const touch = !!window.PBDUI.COARSE;
    const k = !touch && window.innerWidth - 40 >= 1440 ? 1 : 0.5;
    const sc = h('div', { class: 'scene', tabindex: '0', 'aria-label': 'The Play demo: click to walk', style: `width:${1440 * k}px;height:${900 * k}px;--k:${k};--s:${Math.round(2 * k)};cursor:crosshair` });
    const canvas = h('canvas', { class: 'gl' }); const ui = h('div', { class: 'ui' });
    sc.append(canvas, ui, h('div', { class: 'hint', id: 'playHint' }, touch ? 'left: walk; drag right: look; buttons: jump, use, dig, place' : 'click to walk: WASD, Space, mouse; E use, Esc pause, I pack, M map'));
    // The view scales as one picture to fit the page, or the whole screen in fullscreen.
    const inner = h('div', { class: 'scenebox', style: `width:${1440 * k}px;max-width:none;overflow:hidden;transform-origin:0 0` }, sc);
    // wrap is what goes fullscreen; stage holds the view and its controls, and turns sideways when a
    // portrait phone cannot be locked to landscape (the browser's own fullscreen styles forbid
    // transforming the fullscreen element itself)
    const stage = h('div', { class: 'stage' }, inner);
    const wrap = h('div', { class: 'playwrap' }, stage);
    const fsBtn = h('button', { class: 'fsbtn icon', type: 'button', 'aria-label': 'Fullscreen', title: 'Fullscreen', 'data-act': 'fs' }, '⤢');
    const start = h('div', { class: 'playstart' }, h('button', { type: 'button', 'data-act': 'play-fs' }, '▶ Play fullscreen'), h('span', {}, touch ? 'or tap to watch here' : 'or click to play here'));
    stage.append(fsBtn, start);
    box.append(wrap);
    const isFull = () => wrap.classList.contains('fs') || document.fullscreenElement === wrap;
    const fit = () => {
      const full = isFull(), rot = full && window.innerHeight > window.innerWidth;
      stage.classList.toggle('rot', rot); S && (S.rot = rot);
      const vw = rot ? window.innerHeight : window.innerWidth, vh = rot ? window.innerWidth : window.innerHeight;
      const aw = full ? vw : Math.min(window.PBDUI.mainWidth(), box.clientWidth || 1e9), ah = full ? vh : Infinity;
      const f = full ? Math.min(aw / (1440 * k), ah / (900 * k)) : Math.min(1, aw / (1440 * k));
      inner.style.transform = f === 1 ? '' : `scale(${f})`;
      if (!full) { wrap.style.width = Math.floor(1440 * k * f) + 'px'; wrap.style.height = Math.ceil(900 * k * f) + 'px'; inner.style.position = ''; stage.classList.remove('full'); }
      else { wrap.style.width = ''; wrap.style.height = ''; stage.classList.add('full'); inner.style.position = 'absolute'; inner.style.left = Math.round((aw - 1440 * k * f) / 2) + 'px'; inner.style.top = Math.round((ah - 900 * k * f) / 2) + 'px'; }
      fsBtn.textContent = full ? '✕' : '⤢'; fsBtn.setAttribute('aria-label', full ? 'Exit fullscreen' : 'Fullscreen');
    };
    const goFull = () => {
      const after = () => { const o = screen.orientation; if (o && o.lock) o.lock('landscape').catch(() => {}).finally(fit); fit(); };
      const fallback = () => { wrap.classList.add('fs'); after(); };
      if (wrap.requestFullscreen) wrap.requestFullscreen().then(after).catch(fallback); else fallback();
    };
    const leaveFull = () => { if (document.fullscreenElement) document.exitFullscreen().catch(() => {}); wrap.classList.remove('fs'); fit(); };
    fsBtn.addEventListener('click', (e) => { e.stopPropagation(); start.remove(); if (isFull()) leaveFull(); else goFull(); });
    start.addEventListener('click', (e) => { e.stopPropagation(); start.remove(); if (e.target.closest('button')) goFull(); sc.focus({ preventScroll: true }); });
    document.addEventListener('fullscreenchange', () => { if (!document.fullscreenElement) wrap.classList.remove('fs'); fit(); });
    window.addEventListener('resize', () => fit());
    window.addEventListener('orientationchange', () => setTimeout(fit, 200));
    const renderer = new THREE.WebGLRenderer({ canvas, antialias: false, powerPreference: 'high-performance' });
    renderer.setPixelRatio(1); renderer.setSize(1440 * k, 900 * k, false); // pixel ratio 1 everywhere, so a phone's GPU draws 720 x 450
    renderer.outputEncoding = THREE.sRGBEncoding; renderer.toneMapping = THREE.ACESFilmicToneMapping; renderer.toneMappingExposure = 1.0;
    renderer.shadowMap.enabled = true; renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    const scene = new THREE.Scene(); scene.fog = new THREE.Fog(0xb8d4ec, 55, 210);
    const camera = new THREE.PerspectiveCamera(70, 1440 / 900, 0.1, 900);
    S = { touch, wrap, stage, fit, goFull, renderer, scene, camera, sc, canvas, keys: {}, joy: { x: 0, y: 0 }, t: 0, frames: 0, running: {}, stats: { digs: 0, used: [] }, focus: new THREE.Vector3(48, 3, 44), town: [380, -920] };
    S.land = makeLand();
    S.sunL = new THREE.DirectionalLight(0xffffff, 2.5); S.sunL.castShadow = true; S.sunL.shadow.mapSize.set(touch ? 1024 : 2048, touch ? 1024 : 2048);
    Object.assign(S.sunL.shadow.camera, { left: -48, right: 48, top: 48, bottom: -48, near: 1, far: 220 }); S.sunL.shadow.bias = -0.0006; S.sunL.shadow.normalBias = 0.03;
    scene.add(S.sunL, S.sunL.target);
    S.hemi = new THREE.HemisphereLight(0x9cc0ee, 0x4a5a30, 0.8); scene.add(S.hemi);
    buildSky(); buildTerrain(); buildWater(); buildWorld(); buildGrass(); buildParticles();
    // spawn on the path by the pond, looking up the hill at the yard
    S.player = { x: 42, z: 47, y: 0, vy: 0, yaw: -0.6, pitch: -0.1, ground: false };
    S.player.y = groundAt(S.player.x, S.player.z);
    // the UI over it
    S.ui = new GameUI(ui, {
      box: sc, mode: 'play', screen: 'hud', town: 'Dunmouth 1 km', mapBg: 'ui/game/map-before.jpg', captured: () => S.locked || document.activeElement === sc,
      onRun: (kind, on) => { S.running[kind] = on; if (kind === 'press' && !on) S.beltT = 2.5; },
      onScreen: (scr) => { if (S.pad) S.pad.classList.toggle('menu', scr !== 'hud'); if (scr !== 'hud') { S.keys = {}; if (document.pointerLockElement) { S.menuRelease = true; document.exitPointerLock(); } } },
    });
    const keydown = (e) => {
      if (e.defaultPrevented) return;
      if (S.ui.menuOpen) return;
      if (['KeyW', 'KeyA', 'KeyS', 'KeyD', 'Space', 'ShiftLeft'].includes(e.code)) { S.keys[e.code] = true; e.preventDefault(); }
      if (e.code === 'KeyE') { e.preventDefault(); use(S.near || nearMachine()); }
    };
    sc.addEventListener('keydown', keydown);
    sc.addEventListener('keyup', (e) => { S.keys[e.code] = false; });
    sc.addEventListener('blur', () => { S.keys = {}; });
    // mouse: pointer lock on click; drag to look where the lock is refused
    let drag = null;
    canvas.addEventListener('mousedown', (e) => {
      sc.focus({ preventScroll: true });
      if (S.touch) return; // touch plays with the on-screen controls
      if (S.ui.menuOpen) return;
      if (S.locked) { if (e.button === 0) dig(); if (e.button === 2) place(); return; }
      drag = { x: e.clientX, y: e.clientY, moved: 0, b: e.button };
    });
    window.addEventListener('mousemove', (e) => {
      if (S.locked) { S.player.yaw -= e.movementX * 0.0025; S.player.pitch = Math.max(-1.45, Math.min(1.45, S.player.pitch - e.movementY * 0.0025)); return; }
      if (!drag) return;
      const dx = e.clientX - drag.x, dy = e.clientY - drag.y; drag.moved += Math.abs(dx) + Math.abs(dy); drag.x = e.clientX; drag.y = e.clientY;
      S.player.yaw -= dx * 0.005; S.player.pitch = Math.max(-1.45, Math.min(1.45, S.player.pitch - dy * 0.005));
    });
    window.addEventListener('mouseup', (e) => {
      if (!drag) return; const d = drag; drag = null;
      if (d.moved < 4) {
        if (d.b === 0 && !S.triedLock && canvas.requestPointerLock) { S.triedLock = true; try { const p = canvas.requestPointerLock(); if (p && p.catch) p.catch(() => {}); } catch (err) { /* refused: drag to look */ } return; }
        if (d.b === 0) dig(); if (d.b === 2) place();
      }
    });
    canvas.addEventListener('contextmenu', (e) => e.preventDefault());
    document.addEventListener('pointerlockchange', () => {
      const was = S.locked; S.locked = document.pointerLockElement === canvas;
      if (was && !S.locked && !S.menuRelease && !S.ui.menuOpen) S.ui.go('pause'); // Esc while walking: the browser takes it to free the mouse
      S.menuRelease = false;
    });
    // time of day controls
    const seg = document.getElementById('todSeg'), rng = document.getElementById('todRange');
    seg.addEventListener('click', (e) => { const b = e.target.closest('button'); if (!b) return; rng.value = b.dataset.t; setTime(+b.dataset.t); seg.querySelectorAll('button').forEach((x) => x.setAttribute('aria-pressed', String(x === b))); });
    rng.addEventListener('input', () => { setTime(+rng.value); seg.querySelectorAll('button').forEach((x) => x.setAttribute('aria-pressed', 'false')); });
    setTime(+rng.value);
    if (touch) touchControls(stage);
    fit();
    step(0);
  }
  // On a touch screen: a stick on the left to walk, a drag on the right to look, and buttons.
  function touchControls(wrap) {
    const stick = h('div', { class: 'stick', 'aria-label': 'walk' }, h('i', {})), look = h('div', { class: 'look', 'aria-label': 'look' });
    const B = (lab, fn) => h('button', { type: 'button', 'data-t': lab.toLowerCase(), onclick: (e) => { e.stopPropagation(); fn(); } }, lab);
    const pad = h('div', { class: 'touchpad' }, look, stick, h('div', { class: 'tbtns' },
      B('Jump', () => { S.keys.Space = true; setTimeout(() => { S.keys.Space = false; }, 150); }), B('Use', () => use(S.near || nearMachine())),
      B('Dig', () => dig()), B('Place', () => place()), B('Pack', () => S.ui.go(S.ui.st.screen === 'pack' ? 'hud' : 'pack')),
      B('Map', () => S.ui.go(S.ui.st.screen === 'map' ? 'hud' : 'map')), B('Pause', () => S.ui.go(S.ui.menuOpen ? 'hud' : 'pause'))));
    wrap.append(pad); S.pad = pad;
    let sid = null, c0 = null;
    stick.addEventListener('pointerdown', (e) => { sid = e.pointerId; stick.setPointerCapture(e.pointerId); const r = stick.getBoundingClientRect(); c0 = [r.left + r.width / 2, r.top + r.height / 2]; move(e); });
    const move = (e) => { if (e.pointerId !== sid) return; let dx = (e.clientX - c0[0]) / 50, dy = (e.clientY - c0[1]) / 50; if (S.rot) [dx, dy] = [dy, -dx]; const l = Math.max(1, Math.hypot(dx, dy)); S.joy.x = dx / l; S.joy.y = dy / l; stick.firstChild.style.transform = `translate(${S.joy.x * 40}px, ${S.joy.y * 40}px)`; };
    stick.addEventListener('pointermove', move);
    const end = (e) => { if (e.pointerId !== sid) return; sid = null; S.joy.x = S.joy.y = 0; stick.firstChild.style.transform = ''; };
    stick.addEventListener('pointerup', end); stick.addEventListener('pointercancel', end);
    let lid = null, lp = null;
    look.addEventListener('pointerdown', (e) => { lid = e.pointerId; look.setPointerCapture(e.pointerId); lp = [e.clientX, e.clientY]; });
    look.addEventListener('pointermove', (e) => { if (e.pointerId !== lid) return; let dx = e.clientX - lp[0], dy = e.clientY - lp[1]; if (S.rot) [dx, dy] = [dy, -dx]; S.player.yaw -= dx * 0.006; S.player.pitch = Math.max(-1.45, Math.min(1.45, S.player.pitch - dy * 0.006)); lp = [e.clientX, e.clientY]; });
    look.addEventListener('pointerup', () => { lid = null; }); look.addEventListener('pointercancel', () => { lid = null; });
  }
  window.PBDPlay = {
    open() { if (!S) build(); if (S.on) return; S.on = true; last = 0; S.raf = requestAnimationFrame(frame); },
    close() { if (!S) return; S.on = false; cancelAnimationFrame(S.raf); if (document.pointerLockElement) { S.menuRelease = true; document.exitPointerLock(); } },
    // for the checks: the state, a fixed step, the view
    get S() { return S; },
    step(n = 1, dt = 1 / 30) { for (let i = 0; i < n; i++) { S.t += dt; tick(dt); } S.renderer.render(S.scene, S.camera); },
    look(yaw, pitch) { S.player.yaw = yaw; if (pitch != null) S.player.pitch = pitch; step(0); },
    goTo(x, z, yaw) { S.player.x = x; S.player.z = z; S.player.y = groundAt(x, z); if (yaw != null) S.player.yaw = yaw; step(0); },
    dig, place, use: (kind) => use(S.machines.find((m) => m.kind === kind)), setTime: (t) => setTime(t),
    hideUI(on) { S.sc.querySelector('.ui').style.display = on ? 'none' : ''; S.sc.querySelector('.hint').style.display = on ? 'none' : ''; },
  };
})();
