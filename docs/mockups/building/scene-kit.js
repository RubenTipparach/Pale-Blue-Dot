// scene-kit.js: what the npcs and building mockups share so they look like the game.
//
// - The game's cell: 2.833 m flat to flat, 1 m layers, offset (c, r) coordinates with
//   odd rows half a cell east, as docs/mockups/towns.html and assets/settlements/v1 use.
// - The game's textures, nearest and unmipped: the terrain atlas (assets/tilesets/atlas.png,
//   4x4 sheets of 4x4 32 px tiles; the fields sheet is #4, the lunar regolith #7) and the
//   settlement set (assets/textures/settlement, repeat_m from its manifest).
// - A settlement template built the way towns.html builds it (and the engine cuts it):
//   walls on every outside edge per storey with their doors and windows, a gable or cone
//   roof over the footprint, its chimney, its street lamps.
// - The game's tree (planet_surface.wgsl): a hex-prism trunk at 0.20 of the cell, 3 or 4 m,
//   and two 1 m leaf layers at 0.65 + 0.35 * hash of the cell.
// - Block light per pixel: every lamp lights what is near it wherever the camera is, with
//   towns.html's falloff (1 - (d/R)^2)^2, so each one throws a pool on the ground.
// - The townsfolk figure (openspec/changes/townsfolk design 2): hexagonal prisms, flat
//   side forward, textured with pixel-art cloth, face and hair tiles drawn by the
//   balatro-pixel-art kit (tools/art/npcs_art.py), tinted per person.
"use strict";
(function () {
  const PBD = {};
  const W = 2.833, R = W / Math.sqrt(3), ROW = 1.5 * R, STOREY = 3, SLAB = 0.2;
  Object.assign(PBD, { W, R, ROW, STOREY });
  const par = r => ((r % 2) + 2) % 2;
  const toAx = (c, r) => [c - (r - par(r)) / 2, r];
  const toOff = (q, r) => [q + (r - par(r)) / 2, r];
  const DIRS = [[1, 0], [0, 1], [-1, 1], [-1, 0], [0, -1], [1, -1]];
  const nb = (c, r, d) => { const [q] = toAx(c, r); return toOff(q + DIRS[d][0], r + DIRS[d][1]); };
  const dvec = d => [Math.cos(d * Math.PI / 3), Math.sin(d * Math.PI / 3)];
  const hash2 = (x, y) => { let h = (x * 374761393 + y * 668265263) | 0; h = (h ^ (h >>> 13)) * 1274126177 | 0; return ((h ^ (h >>> 16)) >>> 0) / 4294967296; };
  Object.assign(PBD, { par, toAx, toOff, nb, dvec, hash2 });

  // ---- textures --------------------------------------------------------------------------
  const base = (document.currentScript && document.currentScript.src) ? document.currentScript.src.replace(/scene-kit\.js.*$/, "") : "building/";
  PBD.base = base;
  const TEX = {}, loader = new THREE.TextureLoader();
  function prep(t) { t.encoding = THREE.sRGBEncoding; t.wrapS = t.wrapT = THREE.RepeatWrapping; t.magFilter = t.minFilter = THREE.NearestFilter; t.generateMipmaps = false; return t; }
  PBD.tex = url => TEX[url] || (TEX[url] = prep(loader.load(url)));
  PBD.game = name => PBD.tex(base + "game/" + name + ".png?v=2");
  // The atlas: one image, each tile cropped into its own repeating canvas.
  const ATLAS = new Image(); let atlasReady = false; const atlasWait = [];
  ATLAS.onload = () => { atlasReady = true; for (const f of atlasWait.splice(0)) f(); };
  ATLAS.src = base + "game/atlas.png";
  PBD.SHEET = { fields: 4, lunar: 7 };
  const atlasXY = (sheet, tx, ty) => [(sheet % 4) * 128 + tx * 32, Math.floor(sheet / 4) * 128 + ty * 32];
  function whenAtlas(f) { if (atlasReady) f(); else atlasWait.push(f); }
  PBD.atlas = (sheet, tx, ty) => {
    const k = "atlas:" + sheet + ":" + tx + ":" + ty; if (TEX[k]) return TEX[k];
    const cv = document.createElement("canvas"); cv.width = cv.height = 32; const t = prep(new THREE.CanvasTexture(cv));
    whenAtlas(() => { const [x, y] = atlasXY(sheet, tx, ty); cv.getContext("2d").drawImage(ATLAS, x, y, 32, 32, 0, 0, 32, 32); t.needsUpdate = true; });
    return (TEX[k] = t);
  };
  PBD.atlasURL = (sheet, tx, ty) => new Promise(res => whenAtlas(() => { const cv = document.createElement("canvas"); cv.width = cv.height = 32; const [x, y] = atlasXY(sheet, tx, ty); cv.getContext("2d").drawImage(ATLAS, x, y, 32, 32, 0, 0, 32, 32); res(cv.toDataURL()); }));

  // ---- block light, per pixel ------------------------------------------------------------
  const MAXL = 128;
  const LU = { uLP: { value: [] }, uLC: { value: [] }, uLN: { value: 0 }, uLamp: { value: 1 } };
  for (let i = 0; i < MAXL; i++) { LU.uLP.value.push(new THREE.Vector4()); LU.uLC.value.push(new THREE.Vector4()); }
  PBD.LU = LU; PBD.MAXL = MAXL;
  // lights: [{x, y, z, R, c: THREE.Color (linear), p}]; the strongest first if there are too many
  PBD.setLights = list => {
    const l = list.length > MAXL ? list.slice().sort((a, b) => b.p * b.R - a.p * a.R).slice(0, MAXL) : list;
    l.forEach((L, i) => { LU.uLP.value[i].set(L.x, L.y, L.z, L.R); LU.uLC.value[i].set(L.c.r, L.c.g, L.c.b, L.p); });
    LU.uLN.value = l.length;
  };
  PBD.patch = (m, sky) => {
    m.onBeforeCompile = sh => {
      Object.assign(sh.uniforms, LU);
      sh.vertexShader = sh.vertexShader.replace("#include <common>", "#include <common>\nvarying vec3 vWP;")
        .replace("#include <project_vertex>", "#include <project_vertex>\nvWP = (modelMatrix * vec4(transformed, 1.0)).xyz;");
      sh.fragmentShader = sh.fragmentShader.replace("#include <common>", `#include <common>
varying vec3 vWP; uniform vec4 uLP[${MAXL}]; uniform vec4 uLC[${MAXL}]; uniform int uLN; uniform float uLamp;`)
        .replace("#include <aomap_fragment>", `#include <aomap_fragment>
{ vec3 wn = normalize((vec4(normal, 0.0) * viewMatrix).xyz); vec3 blk = vec3(0.0);
  for (int i = 0; i < ${MAXL}; i++) { if (i >= uLN) break;
    vec3 d = uLP[i].xyz - vWP; float d2 = dot(d, d), R2 = uLP[i].w * uLP[i].w; if (d2 >= R2) continue;
    float nd = dot(wn, d * inversesqrt(max(d2, 1e-4))); if (nd < -0.1) continue;
    float f = 1.0 - d2 / R2; f *= f; blk += uLC[i].rgb * uLC[i].w * f * (0.3 + 0.7 * max(nd, 0.0)); }
  reflectedLight.indirectDiffuse += blk * uLamp * material.diffuseColor; }`);
    };
    m.customProgramCacheKey = () => "pbd-lit";
    return m;
  };
  PBD.litMaterial = (map, color = 0xffffff, extra = {}) => PBD.patch(new THREE.MeshStandardMaterial({ map: map || null, color, roughness: 0.95, metalness: 0, ...extra }));

  // ---- geometry: one buffer per material ------------------------------------------------
  const V = {
    add: (a, b) => [a[0] + b[0], a[1] + b[1], a[2] + b[2]], sub: (a, b) => [a[0] - b[0], a[1] - b[1], a[2] - b[2]],
    mul: (a, s) => [a[0] * s, a[1] * s, a[2] * s], dot: (a, b) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2],
  };
  function newell(pts) { let x = 0, y = 0, z = 0; for (let i = 0; i < pts.length; i++) { const p = pts[i], q = pts[(i + 1) % pts.length]; x += (p[1] - q[1]) * (p[2] + q[2]); y += (p[2] - q[2]) * (p[0] + q[0]); z += (p[0] - q[0]) * (p[1] + q[1]); } return [x, y, z]; }
  class Geo {
    constructor() { this.p = []; this.n = []; this.u = []; }
    face(pts, want, uvf) {
      const nw = newell(pts), a = Math.hypot(nw[0], nw[1], nw[2]); if (a < 1e-7) return;
      let n = V.mul(nw, 1 / a); if (want && V.dot(n, want) < 0) { pts = pts.slice().reverse(); n = V.mul(n, -1); }
      for (let i = 1; i + 1 < pts.length; i++) for (const k of [0, i, i + 1]) { const p = pts[k]; this.p.push(p[0], p[1], p[2]); this.n.push(n[0], n[1], n[2]); const uv = uvf ? uvf(p, n) : null; this.u.push(uv ? uv[0] : NaN, uv ? uv[1] : NaN); }
    }
    build(rep = 2) {
      const g = new THREE.BufferGeometry(), P = new Float32Array(this.p), N = new Float32Array(this.n), nv = P.length / 3, uv = new Float32Array(nv * 2);
      for (let i = 0; i < nv; i++) {
        if (!Number.isNaN(this.u[2 * i])) { uv[2 * i] = this.u[2 * i]; uv[2 * i + 1] = this.u[2 * i + 1]; continue; }
        const x = P[3 * i], y = P[3 * i + 1], z = P[3 * i + 2], nx = N[3 * i], ny = N[3 * i + 1], nz = N[3 * i + 2];
        if (Math.abs(ny) > 0.7) { uv[2 * i] = x / rep; uv[2 * i + 1] = z / rep; }
        else { const l = Math.hypot(nx, nz) || 1; uv[2 * i] = (x * -nz + z * nx) / l / rep; uv[2 * i + 1] = y / rep; }
      }
      g.setAttribute("position", new THREE.BufferAttribute(P, 3)); g.setAttribute("normal", new THREE.BufferAttribute(N, 3)); g.setAttribute("uv", new THREE.BufferAttribute(uv, 2));
      return g;
    }
  }
  PBD.Geo = Geo;
  // A set of geometries by material name.
  PBD.GeoSet = class {
    constructor() { this.G = {}; }
    g(m) { return this.G[m] || (this.G[m] = new Geo()); }
    // an oriented box: centre (x, z), floor y0, size sx along the heading, sy up, sz across; mf(n) picks a material
    box(m, x, y0, z, sx, sy, sz, ang = 0, uvf) {
      const ca = Math.cos(ang), sa = Math.sin(ang), ex = [ca, 0, sa], ez = [-sa, 0, ca], c = [x, y0, z], hx = sx / 2, hz = sz / 2;
      const P = (a, b, h) => V.add(V.add(V.add(c, V.mul(ex, a)), V.mul(ez, b)), [0, h, 0]);
      const q = (n, pts) => { const mm = typeof m === "function" ? m(n) : m; if (!mm) return; this.g(mm).face(pts, n, uvf && uvf(n, mm)); };
      q(ex, [P(hx, hz, 0), P(hx, -hz, 0), P(hx, -hz, sy), P(hx, hz, sy)]); q(V.mul(ex, -1), [P(-hx, -hz, 0), P(-hx, hz, 0), P(-hx, hz, sy), P(-hx, -hz, sy)]);
      q(ez, [P(-hx, hz, 0), P(hx, hz, 0), P(hx, hz, sy), P(-hx, hz, sy)]); q(V.mul(ez, -1), [P(hx, -hz, 0), P(-hx, -hz, 0), P(-hx, -hz, sy), P(hx, -hz, sy)]);
      q([0, 1, 0], [P(-hx, -hz, sy), P(hx, -hz, sy), P(hx, hz, sy), P(-hx, hz, sy)]); q([0, -1, 0], [P(-hx, -hz, 0), P(hx, -hz, 0), P(hx, hz, 0), P(-hx, hz, 0)]);
    }
    // a prism over a convex polygon [[x, z]...]
    prism(mTop, mSide, pts, y0, y1, o = {}) {
      let cx = 0, cz = 0; for (const p of pts) { cx += p[0]; cz += p[1]; } cx /= pts.length; cz /= pts.length;
      if (mTop) this.g(mTop).face(pts.map(p => [p[0], y1, p[1]]), [0, 1, 0], o.uvTop);
      if (o.bottom) this.g(o.bottom).face(pts.map(p => [p[0], y0, p[1]]), [0, -1, 0], o.uvTop);
      if (!mSide) return;
      for (let i = 0; i < pts.length; i++) { const a = pts[i], b = pts[(i + 1) % pts.length]; this.g(mSide).face([[a[0], y0, a[1]], [b[0], y0, b[1]], [b[0], y1, b[1]], [a[0], y1, a[1]]], [(a[0] + b[0]) / 2 - cx, 0, (a[1] + b[1]) / 2 - cz], o.uvSide); }
    }
  };
  // UVs the terrain shader uses: a side one tile across its edge and one a metre up
  PBD.sideUV = (A, B, yTop) => (p) => { const L = Math.hypot(B[0] - A[0], B[1] - A[1]); return [Math.hypot(p[0] - A[0], p[2] - A[1]) / L, (p[1] - yTop) + 1]; };
  PBD.cellUV = (cx, cz, s = 1.7 * R) => p => [(p[0] - cx) / s + 0.5, (p[2] - cz) / s + 0.5];

  // ---- the game's tree --------------------------------------------------------------------
  // trunk: wood at 0.20 of the cell, 3 or 4 m; two 1 m leaf layers at 0.65 + 0.35 * hash.
  PBD.tree = (GS, pts, x, z, y, id, mWood, mLeaf) => {
    const shrink = s => pts.map(p => [x + (p[0] - x) * s, z + (p[1] - z) * s]);
    const roll = hash2(id, 7), trunk = 3 + (roll < 0.5 ? 0 : 1);
    const part = (m, s, y0, y1) => {
      const q = shrink(s);
      for (let i = 0; i < 6; i++) { const a = q[i], b = q[(i + 1) % 6]; GS.g(m).face([[a[0], y0, a[1]], [b[0], y0, b[1]], [b[0], y1, b[1]], [a[0], y1, a[1]]], [(a[0] + b[0]) / 2 - x, 0, (a[1] + b[1]) / 2 - z], p => [Math.hypot(p[0] - a[0], p[2] - a[1]) / Math.hypot(b[0] - a[0], b[1] - a[1]), (p[1] - y1)]); }
      const uv = PBD.cellUV(x, z, 1.7 * R * s);
      GS.g(m).face(q.map(p => [p[0], y1, p[1]]), [0, 1, 0], uv); GS.g(m).face(q.map(p => [p[0], y0, p[1]]), [0, -1, 0], uv);
    };
    part(mWood, 0.2, y, y + trunk);
    part(mLeaf, 0.65 + 0.35 * hash2(id, 1), y + trunk, y + trunk + 1);
    part(mLeaf, 0.65 + 0.35 * hash2(id, 2), y + trunk + 1, y + trunk + 2);
  };

  // ---- a settlement template, as towns.html builds it ------------------------------------
  const KITS = {
    stone:      { walls: [{ out: "fieldstone", t: 0.5 }], gable: "fieldstone", floor: "flag", chimney: "fieldstone" },
    halftimber: { walls: [{ out: "stone", t: 0.5 }, { out: "halftimber", t: 0.3, piece: true }], gable: "halftimber", floor: "flag", chimney: "stone" },
    timber:     { walls: [{ out: "boards", t: 0.25 }], gable: "boards", floor: "plank", chimney: "stone" },
    ashlar:     { walls: [{ out: "stone", t: 0.5 }], gable: "stone", floor: "flag", chimney: "stone" },
    brick:      { walls: [{ out: "brick", t: 0.4 }], gable: "brick", floor: "plank", chimney: "brick" },
    straw:      { walls: [{ out: "reed", t: 0.3 }], hut: true, floor: "dirt", door: [0.9, 1.9] },
  };
  PBD.KITS = KITS;
  // tops the template paints, to what draws them: grass is the terrain's own (atlas)
  const TOPS = { grass: "a:grass", dirt: "a:path", flag: "flag", plank: "plank", sand: "sand", cobble: "cobble" };
  // Build `tpl` (assets/settlements/v1 JSON). Coordinates are the template's metres minus (ox, oz).
  PBD.buildTown = (tpl, o = {}) => {
    const ox = o.ox || 0, oz = o.oz || 0, NC = tpl.grid.columns, NR = tpl.grid.rows;
    const cx = (c, r) => W * (c + 0.5 * par(r)) - ox, cz = (c, r) => ROW * r - oz;
    const corner = (c, r, k) => { const a = (60 * k - 30) * Math.PI / 180; return [cx(c, r) + R * Math.cos(a), cz(c, r) + R * Math.sin(a)]; };
    const hexPts = (c, r) => [0, 1, 2, 3, 4, 5].map(k => corner(c, r, k));
    const inMap = (c, r) => c >= 0 && r >= 0 && c < NC && r < NR, idx = (c, r) => c + r * NC;
    const TOP = new Float32Array(NC * NR), MAT = new Array(NC * NR).fill("grass"), AREA = new Array(NC * NR).fill(""), BLD = new Int16Array(NC * NR).fill(-1);
    for (const g of tpl.ground) { TOP[idx(g.c, g.r)] = g.h; MAT[idx(g.c, g.r)] = g.top; AREA[idx(g.c, g.r)] = g.area; }
    const GS = new PBD.GeoSet(), out = { GS, panes: [], lamps: [], doors: [], buildings: [], cx, cz, corner, hexPts, TOP, MAT, AREA, BLD, idx, inMap, NC, NR, trees: [] };
    tpl.buildings.forEach((b, bi) => {
      const kit = KITS[b.kit] || KITS.stone;
      for (const [c, r] of b.cells) { TOP[idx(c, r)] = b.base; MAT[idx(c, r)] = kit.floor; BLD[idx(c, r)] = bi; }
    });
    // ground: hex columns, the terrain's tiles on grass, the town's own on its lanes and floors
    for (let r = 0; r < NR; r++) for (let c = 0; c < NC; c++) {
      const i = idx(c, r), h = TOP[i], pts = hexPts(c, r), m = TOPS[MAT[i]] || MAT[i], x = cx(c, r), z = cz(c, r);
      GS.g(m).face(pts.map(p => [p[0], h, p[1]]), [0, 1, 0], m.startsWith("a:") ? PBD.cellUV(x, z) : null);
      for (let d = 0; d < 6; d++) {
        const [c2, r2] = nb(c, r, d), h2 = inMap(c2, r2) ? TOP[idx(c2, r2)] : h - 3; if (h2 >= h) continue;
        const A = corner(c, r, d), B = corner(c, r, d + 1), [nx, nz] = dvec(d);
        for (let y = h; y > h2 + 1e-3; y -= 1) {
          const y0 = Math.max(h2, y - 1), sm = MAT[i] === "grass" && y === h ? "a:grassside" : "a:dirt";
          GS.g(sm).face([[A[0], y0, A[1]], [B[0], y0, B[1]], [B[0], y, B[1]], [A[0], y, A[1]]], [nx, 0, nz], PBD.sideUV(A, B, y));
        }
      }
      if (MAT[i] === "sand" && h <= 0) out.water = (out.water || []).concat([[c, r]]);
    }
    // buildings
    tpl.buildings.forEach((b, bi) => {
      const kit = KITS[b.kit] || KITS.stone, cells = b.cells, inB = (c, r) => cells.some(([a, q]) => a === c && q === r);
      const storeys = b.storeys || 1, sh = kit.hut ? 2 : STOREY * (b.tall || 1), top = b.base + storeys * sh;
      const [doorW, doorH] = kit.door || [1.0, 2.2];
      let minx = 1e9, maxx = -1e9, minz = 1e9, maxz = -1e9;
      for (const [c, r] of cells) for (const p of hexPts(c, r)) { minx = Math.min(minx, p[0]); maxx = Math.max(maxx, p[0]); minz = Math.min(minz, p[1]); maxz = Math.max(maxz, p[1]); }
      const rec = { name: b.name, kit: b.kit, cells, base: b.base, top, minx, maxx, minz, maxz, door: null };
      for (const [c, r] of cells) for (let d = 0; d < 6; d++) {
        const [c2, r2] = nb(c, r, d); if (inB(c2, r2)) continue;
        for (let s = 0; s < storeys; s++) {
          const y0 = b.base + s * sh, y1 = y0 + sh, w = kit.walls[Math.min(s, kit.walls.length - 1)], t = w.t;
          const door = (b.doors || []).find(e => e[0] === c && e[1] === r && e[2] === d && (e[3] || 0) === s);
          const win = !door && (b.windows || []).some(e => e[0] === c && e[1] === r && e[2] === d && (e[3] || 0) === s);
          const [A, B] = [corner(c, r, d), corner(c, r, d + 1)], L = Math.hypot(B[0] - A[0], B[1] - A[1]), tx = (B[0] - A[0]) / L, tz = (B[1] - A[1]) / L;
          const [nx, nz] = dvec(d), mx = (A[0] + B[0]) / 2, mz = (A[1] + B[1]) / 2, ang = Math.atan2(tz, tx), len = L + t * 0.58;
          const pick = n => { const s2 = n[0] * nx + n[2] * nz; if (s2 > 0.5) return w.out; return w.piece ? "timber" : w.out; };
          const pieceUV = (n, m) => m === "halftimber" ? p => [((p[0] - A[0]) * tx + (p[2] - A[1]) * tz) / L, (p[1] - y0) / STOREY] : null;
          const seg = (along, l, a, bb) => { if (bb - a < 0.005 || l < 0.005) return; GS.box(pick, mx + tx * along, a, mz + tz * along, l, bb - a, t, ang, pieceUV); };
          let op = null;
          if (door) op = { yb: y0, ye: y0 + doorH, w: doorW, door: true };
          else if (win && !kit.hut) op = b.tall > 1 ? { yb: y0 + 1.6, ye: y0 + 4.4, w: 0.9, win: true } : { yb: y0 + 1.0, ye: y0 + 2.0, w: 0.8, win: true, shutters: true };
          if (!op) { seg(0, len, y0, y1); continue; }
          const ww = Math.min(op.w, L - 0.2), jl = (len - ww) / 2;
          seg(0, len, y0, op.yb); seg(-(ww / 2 + jl / 2), jl, op.yb, op.ye); seg(ww / 2 + jl / 2, jl, op.yb, op.ye); seg(0, len, op.ye, y1);
          if (op.door) {
            GS.box("timber", mx - nx * 0.04, op.yb, mz - nz * 0.04, ww, op.ye - op.yb, 0.08, ang);
            const d0 = { x: mx + nx * 1.0, z: mz + nz * 1.0, y: b.base, nx, nz, mx, mz, ang, t, lamp: [mx + nx * (t / 2 + 0.2) + tx * (ww / 2 + 0.35), y0 + 2.3, mz + nz * (t / 2 + 0.2) + tz * (ww / 2 + 0.35)] };
            if (!rec.door) rec.door = d0; out.doors.push(d0);
          }
          if (op.win) {
            out.panes.push({ x: mx, y: (op.yb + op.ye) / 2, z: mz, w: ww, h: op.ye - op.yb, ang, nx, nz, lit: hash2(c * 7 + d, r * 13 + s * 3 + bi) < 0.55, b: 0.6 + 0.4 * hash2(c + d, r * 5 + s) });
            GS.box("timber", mx + nx * 0.02, op.yb - 0.04, mz + nz * 0.02, ww + 0.1, 0.07, t + 0.12, ang);
            if (op.shutters) for (const sg of [-1, 1]) {
              const hx2 = mx + tx * sg * ww / 2 + nx * (t / 2 + 0.03), hz2 = mz + tz * sg * ww / 2 + nz * (t / 2 + 0.03), a2 = ang + sg * 1.9;
              GS.box("timber", hx2 + Math.cos(a2) * ww / 4 * sg, op.yb, hz2 + Math.sin(a2) * ww / 4 * sg, ww / 2, op.ye - op.yb, 0.05, a2);
            }
          }
        }
      }
      // roof
      const oh = kit.hut ? 0.55 : 0.45;
      if (b.roof === "cone" || (kit.hut && cells.length === 1)) {
        const [c, r] = cells[0], x = cx(c, r), z = cz(c, r), h = 2.6, apex = [x, top + h, z], rr = R + oh / Math.cos(Math.PI / 6), slope = h / (W / 2);
        const pts = []; for (let k = 0; k < 6; k++) { const a = (60 * k - 30) * Math.PI / 180; pts.push([x + rr * Math.cos(a), top - oh * slope, z + rr * Math.sin(a)]); }
        for (let k = 0; k < 6; k++) { const a = pts[k], q = pts[(k + 1) % 6]; GS.g("thatch").face([a, q, apex], [(a[0] + q[0]) / 2 - x, 0.5, (a[2] + q[2]) / 2 - z], p => [(p[0] + p[2]) / 2, p[1] / 2]); GS.g("thatch").face([a, q, apex], [0, -1, 0], p => [(p[0] + p[2]) / 2, p[1] / 2]); }
      } else {
        const m = b.roof, pitch = b.pitch ?? 1, x0 = minx, x1 = maxx, z0 = minz, z1 = maxz, zc = (z0 + z1) / 2, hw = (z1 - z0) / 2, yR = top + hw * pitch, eY = top - oh * pitch, th = 0.18;
        for (const s of [-1, 1]) {
          const ze = zc + s * (hw + oh), uv = p => [p[0] / 2, (Math.abs(p[2] - zc) * Math.sqrt(1 + pitch * pitch)) / 2];
          GS.g(m).face([[x0 - oh, eY + th, ze], [x1 + oh, eY + th, ze], [x1 + oh, yR + th, zc], [x0 - oh, yR + th, zc]], [0, 1, s * 0.01], uv);
          GS.g("plank").face([[x0 - oh, eY, ze], [x1 + oh, eY, ze], [x1 + oh, yR, zc], [x0 - oh, yR, zc]], [0, -1, 0]);
          GS.g(m).face([[x0 - oh, eY, ze], [x1 + oh, eY, ze], [x1 + oh, eY + th, ze], [x0 - oh, eY + th, ze]], [0, 0, s]);
          for (const xe of [x0 - oh, x1 + oh]) GS.g(m).face([[xe, eY, ze], [xe, eY + th, ze], [xe, yR + th, zc], [xe, yR, zc]], [xe < x0 ? -1 : 1, 0, 0]);
        }
        for (const xe of [x0 + 0.05, x1 - 0.05]) GS.g(kit.gable).face([[xe, top, z0], [xe, top, z1], [xe, yR, zc]], [xe < (x0 + x1) / 2 ? -1 : 1, 0, 0], kit.gable === "halftimber" ? p => [(p[2] - z0) / (z1 - z0) * 2, (p[1] - top) / 3] : null);
        GS.g("plank").face([[x0, top, z0], [x1, top, z0], [x1, top, z1], [x0, top, z1]], [0, -1, 0]);
        GS.box("timber", (x0 + x1) / 2, yR + th - 0.05, zc, x1 - x0 + 2 * oh + 0.1, 0.16, 0.22);
        if (b.chimney) { const [c, r] = b.chimney; GS.box(kit.chimney || "stone", cx(c, r), top + 0.02, cz(c, r), 0.8, hw + 1.18, 0.8); rec.chimney = [cx(c, r), yR + 1.2, cz(c, r)]; }
      }
      out.buildings.push(rec);
    });
    // street lamps: a timber post, an arm, a lantern
    for (const [c, r] of tpl.lamps || []) {
      const x = cx(c, r) + (BLD[idx(c, r)] >= 0 ? 0 : 0), z = cz(c, r), y = TOP[idx(c, r)];
      if (BLD[idx(c, r)] >= 0) continue;
      PBD.lampPost(GS, x, y, z); out.lamps.push({ x: x + 0.45, y: y + 2.25, z, kind: "post" });
    }
    return out;
  };
  PBD.lampPost = (GS, x, y, z) => {
    const p = []; for (let k = 0; k < 6; k++) { const a = k * Math.PI / 3; p.push([x + 0.09 * Math.cos(a), z + 0.09 * Math.sin(a)]); }
    GS.prism("timber", "timber", p, y, y + 2.6); GS.box("timber", x + 0.25, y + 2.55, z, 0.5, 0.06, 0.06);
    GS.box("iron", x + 0.45, y + 2.07, z, 0.26, 0.05, 0.26); GS.box("iron", x + 0.45, y + 2.45, z, 0.26, 0.05, 0.26);
  };

  // ---- materials by name -----------------------------------------------------------------
  // "a:<tile>" is the terrain atlas (fields sheet unless the page says "lunar"); anything
  // else a settlement texture at its manifest repeat_m.
  const ATILE = { grass: [0, 0], grassside: [1, 0], dirt: [2, 0], stone: [3, 0], bark: [2, 1], leaves: [0, 2], path: [3, 2], boards: [1, 3] };
  PBD.REP = { grass: 2, dirt: 2, sand: 2, cobble: 2, flag: 2.5, stone: 2, plank: 2, timber: 1, plaster: 2, halftimber: 1, thatch: 2, shingle: 2, slate: 2, iron: 1, leaves: 2, bark: 1, brick: 2, reed: 2, boards: 2, fieldstone: 2 };
  PBD.sheet = PBD.SHEET.fields;
  const MATS = {};
  PBD.mat = (name, extra = {}) => {
    if (MATS[name]) return MATS[name];
    const map = name.startsWith("a:") ? PBD.atlas(PBD.sheet, ...ATILE[name.slice(2)]) : PBD.game(name);
    const side = name === "thatch" || name === "a:leaves" ? { side: THREE.DoubleSide } : {};
    return (MATS[name] = PBD.litMaterial(map, 0xffffff, { ...side, ...extra }));
  };
  PBD.rep = name => name.startsWith("a:") ? 2 : (PBD.REP[name] || 2);
  // Every geometry of a GeoSet as meshes in `parent`.
  PBD.addMeshes = (GS, parent, opts = {}) => {
    const meshes = [];
    for (const [k, g] of Object.entries(GS.G)) { const m = new THREE.Mesh(g.build(PBD.rep(k)), opts.mat ? opts.mat(k) : PBD.mat(k)); m.castShadow = opts.cast ? opts.cast(k) : true; m.receiveShadow = true; m.userData.mat = k; parent.add(m); meshes.push(m); }
    return meshes;
  };

  // ---- the townsfolk figure ---------------------------------------------------------------
  // A hex prism, flat side forward (+z): w across, h up, d deep. Groups: 0 sides, 1 front, 2 caps.
  const PRISM = {};
  function prismGeo(w, h, d, tileV) {
    const k = [w, h, d, tileV].join(":"); if (PRISM[k]) return PRISM[k];
    const r = w / 2, sz = d / (r * Math.sqrt(3)), pts = []; for (let i = 0; i < 6; i++) { const a = i * Math.PI / 3; pts.push([r * Math.cos(a), r * Math.sin(a) * sz]); }
    const P = [], N = [], U = [], g = new THREE.BufferGeometry(); let vi = 0; const groups = [[], [], []];
    const push = (grp, tris) => { const start = vi; for (const [p, n, u] of tris) { P.push(...p); N.push(...n); U.push(...u); vi++; } groups[grp].push([start, vi - start]); };
    for (let i = 0; i < 6; i++) {
      const a = pts[i], b = pts[(i + 1) % 6], mx = (a[0] + b[0]) / 2, mz = (a[1] + b[1]) / 2, l = Math.hypot(mx, mz), n = [mx / l, 0, mz / l];
      const sideW = Math.hypot(b[0] - a[0], b[1] - a[1]), front = i === 1, vmax = front || !tileV ? 1 : h / Math.max(sideW, 0.05);
      const A0 = [b[0], -h / 2, b[1]], B0 = [a[0], -h / 2, a[1]], B1 = [a[0], h / 2, a[1]], A1 = [b[0], h / 2, b[1]];
      push(front ? 1 : 0, [[A0, n, [0, 0]], [B0, n, [1, 0]], [B1, n, [1, vmax]], [A0, n, [0, 0]], [B1, n, [1, vmax]], [A1, n, [0, vmax]]]);
    }
    for (const s of [1, -1]) { const tris = []; for (let i = 1; i < 5; i++) { const ids = s > 0 ? [0, i + 1, i] : [0, i, i + 1]; for (const j of ids) tris.push([[pts[j][0], s * h / 2, pts[j][1]], [0, s, 0], [pts[j][0] / w + 0.5, pts[j][1] / w + 0.5]]); } push(2, tris); }
    g.setAttribute("position", new THREE.Float32BufferAttribute(P, 3)); g.setAttribute("normal", new THREE.Float32BufferAttribute(N, 3)); g.setAttribute("uv", new THREE.Float32BufferAttribute(U, 2));
    groups.forEach((list, mi) => list.forEach(([s, c]) => g.addGroup(s, c, mi)));
    return (PRISM[k] = g);
  }
  const FIG = {};
  const figTex = n => PBD.tex(PBD.figBase + n + ".png?v=2");
  PBD.figBase = "npcs/";
  function figMat(tex, col, lit) {
    const k = tex + ":" + col + ":" + lit; if (FIG[k]) return FIG[k];
    const color = new THREE.Color(col);
    const m = lit ? PBD.litMaterial(figTex(tex), color) : new THREE.MeshStandardMaterial({ map: figTex(tex), color, roughness: 0.95 });
    return (FIG[k] = m);
  }
  // spec: { h, b, skin, hair, tunic, legs, accent, hat, prop, apron, tabard, long }
  PBD.figure = (spec, lit = true) => {
    const g = new THREE.Group();
    const part = (w, h, d, tex, col, x, y, z, parent = g, front, tile = true) => {
      const side = figMat(tex, col, lit), m = new THREE.Mesh(prismGeo(w, h, d, tile), [side, front ? figMat(front, col, lit) : side, side]);
      m.position.set(x, y, z); m.castShadow = true; parent.add(m); return m;
    };
    const s = spec.h, b = spec.b;
    const hip = x => { const p = new THREE.Group(); p.position.set(x, 0.8 * s, 0); g.add(p); part(0.18 * b, 0.8 * s, 0.2, "fig_cloth", spec.legs, 0, -0.4 * s, 0, p); part(0.2 * b, 0.12, 0.26, "fig_leather", 0x5a4030, 0, -0.78 * s, 0.02, p); return p; };
    const legL = hip(-0.11 * b), legR = hip(0.11 * b);
    part(0.5 * b, 0.72 * s, 0.28, "fig_cloth", spec.tunic, 0, 1.16 * s, 0);
    if (spec.apron) part(0.42 * b, 0.62 * s, 0.05, "fig_leather", spec.apron, 0, 1.08 * s, 0.14);
    if (spec.tabard) part(0.3 * b, 0.6 * s, 0.05, "fig_cloth", spec.tabard, 0, 1.18 * s, 0.15);
    part(0.52 * b, 0.12, 0.3, "fig_leather", 0x6a4a30, 0, 0.86 * s, 0);
    const headG = new THREE.Group(); headG.position.set(0, 1.54 * s, 0); g.add(headG);
    part(0.32, 0.32, 0.3, "fig_skin", spec.skin, 0, 0.16, 0, headG, "fig_face", false);
    const hat = spec.hat;
    if (hat === "straw") { part(0.62, 0.04, 0.62, "fig_straw", 0xffffff, 0, 0.34, 0, headG); part(0.32, 0.14, 0.3, "fig_straw", 0xffffff, 0, 0.42, 0, headG); }
    else if (hat === "helmet") part(0.38, 0.16, 0.36, "fig_metal", 0xffffff, 0, 0.36, 0, headG);
    else if (hat === "sou") { part(0.48, 0.05, 0.46, "fig_cloth", 0xf0c830, 0, 0.32, -0.03, headG); part(0.34, 0.12, 0.32, "fig_cloth", 0xf0c830, 0, 0.38, 0, headG); }
    else if (hat === "turban") part(0.38, 0.16, 0.36, "fig_cloth", spec.accent || 0xe0b050, 0, 0.38, 0, headG);
    else if (hat === "hood") part(0.4, 0.44, 0.36, "fig_cloth", spec.accent || 0x504870, 0, 0.2, -0.05, headG);
    else if (hat === "miner") { part(0.38, 0.14, 0.36, "fig_cloth", 0xf0b428, 0, 0.36, 0, headG); const lamp = new THREE.Mesh(new THREE.BoxGeometry(0.08, 0.06, 0.03), new THREE.MeshBasicMaterial({ color: 0xfff0b0 })); lamp.position.set(0, 0.38, 0.19); headG.add(lamp); }
    else if (hat === "headset") { part(0.34, 0.08, 0.32, "fig_hair", spec.hair, 0, 0.35, 0, headG); part(0.05, 0.1, 0.06, "fig_metal", 0x606878, -0.18, 0.18, 0, headG); }
    else if (hat === "cap") part(0.35, 0.08, 0.33, "fig_cloth", spec.accent || 0x4a5a3a, 0, 0.35, 0, headG);
    if (hat === "none" || hat === "headset" || hat === "straw") part(0.34, 0.1, 0.32, "fig_hair", spec.hair, 0, 0.33, 0, headG);
    if (spec.long) part(0.34, 0.3, 0.06, "fig_hair", spec.hair, 0, 0.14, -0.16, headG);
    const arm = x => { const p = new THREE.Group(); p.position.set(x * b, 1.48 * s, 0); g.add(p); part(0.14, 0.66 * s, 0.16, "fig_cloth", spec.tunic, 0, -0.3 * s, 0, p); part(0.11, 0.1, 0.12, "fig_skin", spec.skin, 0, -0.66 * s, 0, p); return p; };
    const armL = arm(-0.33), armR = arm(0.33);
    const P = spec.prop, hand = (w, h, d, tex, col, y, z, rx = 0) => { const m = part(w, h, d, tex, col, 0, y, z, armR); m.rotation.x = rx; return m; };
    if (P === "fork") { hand(0.05, 1.6, 0.05, "fig_wood", 0xffffff, -0.4, 0.1); hand(0.18, 0.2, 0.03, "fig_metal", 0xffffff, 0.45, 0.1); }
    if (P === "spear") { hand(0.06, 2.3, 0.06, "fig_wood", 0xffffff, -0.2, 0.1); hand(0.08, 0.22, 0.04, "fig_metal", 0xffffff, 1.02, 0.1); }
    if (P === "hammer") { hand(0.06, 0.5, 0.06, "fig_wood", 0xffffff, -0.75, 0.1); hand(0.2, 0.1, 0.1, "fig_metal", 0xffffff, -0.98, 0.1); }
    if (P === "pick") { hand(0.06, 0.8, 0.06, "fig_wood", 0xffffff, -0.6, 0.12); hand(0.5, 0.06, 0.06, "fig_metal", 0xffffff, -0.98, 0.12); }
    if (P === "fish") hand(0.09, 0.36, 0.14, "fig_cloth", 0x5ab0d0, -0.82, 0.08);
    if (P === "pad") hand(0.22, 0.28, 0.03, "fig_metal", 0x506080, -0.66, 0.16, -0.9);
    if (P === "pouch") hand(0.18, 0.2, 0.14, "fig_leather", 0xa07a4a, -0.78, 0.04);
    if (P === "crate") part(0.42, 0.34, 0.34, "fig_wood", 0xffffff, 0.28, 1.0 * s, 0.12);
    return { g, legL, legR, armL, armR, headG };
  };
  window.PBD = PBD;
})();
