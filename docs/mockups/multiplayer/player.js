// player.js: the PLAYER tab of docs/mockups/multiplayer.html (openspec/changes/multiplayer,
// task 5.2, design 14.5): another player on a patch of the fields, by day and lit at night,
// named when near and not when far. One 3D view; its renderer is made when the tab first opens.
//
// - Ground, trees and lamp posts as scene-kit.js builds them (the game's cell, atlas tiles,
//   settlement timber and iron), every lamp lighting what is near it per pixel.
// - The other player is one lofted body: superellipse rings from the hips to the crown, the
//   legs and arms tapered tubes grown from the hips and shoulders, normals smooth across every
//   curve. Its face, hair, belt, cuffs and boots are painted into its textures from the
//   townsfolk's 16 px figure tiles (tools/art/npcs_art.py), tinted per player.
// - Nothing moves until asked: a distance button sends the player walking there, then they
//   stop and turn to face you.
"use strict";
(function () {
  const view = document.getElementById("view3d"), tagEl = document.getElementById("nametag"), crossEl = document.getElementById("cross");
  const W = PBD.W, R = PBD.R, ROW = PBD.ROW;
  const lin = hex => new THREE.Color(hex).convertSRGBToLinear();
  const clamp = (v, a, b) => Math.max(a, Math.min(b, v));
  const smooth = (a, b, x) => { const t = clamp((x - a) / (b - a), 0, 1); return t * t * (3 - 2 * t); };
  const NAME = "Wren", NAME_M = 24;   // the name shows within 24 m (recommendation taken)

  // ------------------------------------------------------------------ scene
  const scene = new THREE.Scene(); scene.fog = new THREE.Fog(0x9fc6e2, 55, 170);
  const camera = new THREE.PerspectiveCamera(62, 1, 0.05, 500); camera.rotation.order = "YXZ";
  const sun = new THREE.DirectionalLight(0xffffff, 1); sun.castShadow = true; sun.shadow.mapSize.set(4096, 4096);
  Object.assign(sun.shadow.camera, { left: -40, right: 40, top: 40, bottom: -40, near: 1, far: 260 }); sun.shadow.bias = -0.0004; sun.shadow.normalBias = 0.02;
  scene.add(sun, sun.target);
  const hemi = new THREE.HemisphereLight(0xffffff, 0x1a1a22, 0.2); scene.add(hemi);

  // ------------------------------------------------------------------ the patch: hex columns
  const C0 = -14, C1 = 14, R0 = -30, R1 = 5;
  const cx = (c, r) => W * (c + 0.5 * PBD.par(r)), cz = (c, r) => ROW * r;
  const corner = (c, r, k) => { const a = (60 * k - 30) * Math.PI / 180; return [cx(c, r) + R * Math.cos(a), cz(c, r) + R * Math.sin(a)]; };
  const hexPts = (c, r) => [0, 1, 2, 3, 4, 5].map(k => corner(c, r, k));
  const inMap = (c, r) => c >= C0 && c <= C1 && r >= R0 && r <= R1;
  const key = (c, r) => c + "," + r;
  const pathH = r => (r <= -17 ? 2 : r <= -9 ? 1 : 0);
  const isPath = (c, r) => Math.abs(cx(c, r)) <= W * 0.55;
  const TOP = new Map(), KIND = new Map();
  for (let r = R0; r <= R1; r++) for (let c = C0; c <= C1; c++) {
    const x = cx(c, r), z = cz(c, r), d = Math.abs(x) / W;
    let h = pathH(r), kind = "path";
    if (!isPath(c, r)) {
      const n = 0.5 + 0.5 * Math.sin(x * 0.23 + z * 0.11) * Math.cos(z * 0.19 - x * 0.08);
      const edge = 1 - smooth(9, 14, Math.abs(c)) ;
      h += Math.floor(clamp((d - 1.4) * 0.42 + n * 1.3, 0, 3) * edge); kind = "grass";
    }
    TOP.set(key(c, r), h); KIND.set(key(c, r), kind);
  }
  const GS = new PBD.GeoSet();
  for (let r = R0; r <= R1; r++) for (let c = C0; c <= C1; c++) {
    const h = TOP.get(key(c, r)), kind = KIND.get(key(c, r)), pts = hexPts(c, r), x = cx(c, r), z = cz(c, r);
    GS.g(kind === "path" ? "a:path" : "a:grass").face(pts.map(p => [p[0], h, p[1]]), [0, 1, 0], PBD.cellUV(x, z));
    for (let d = 0; d < 6; d++) {
      const [c2, r2] = PBD.nb(c, r, d), h2 = inMap(c2, r2) ? TOP.get(key(c2, r2)) : h - 3; if (h2 >= h) continue;
      const A = corner(c, r, d), B = corner(c, r, d + 1), [nx, nz] = PBD.dvec(d);
      for (let y = h; y > h2 + 1e-3; y -= 1) {
        const y0 = Math.max(h2, y - 1), sm = kind === "grass" && y === h ? "a:grassside" : "a:dirt";
        GS.g(sm).face([[A[0], y0, A[1]], [B[0], y0, B[1]], [B[0], y, B[1]], [A[0], y, A[1]]], [nx, 0, nz], PBD.sideUV(A, B, y));
      }
    }
  }
  // the far ground past the patch, in 10 m tiles so its fog reads right
  { const g = GS.g("a:grass"), S = 10;
    for (let x = -300; x < 300; x += S) for (let z = -330; z < 270; z += S) {
      if (x >= -44 && x + S <= 44 && z >= -76 && z + S <= 16) continue;
      g.face([[x, -0.03, z], [x + S, -0.03, z], [x + S, -0.03, z + S], [x, -0.03, z + S]], [0, 1, 0], p => [p[0] / 2.78, p[2] / 2.78]);
    } }

  // lamp posts by the path every fourth row, sides alternating, the lantern hung over the path
  const LIGHTS = [], GLOW = [];
  const lampMat = new THREE.MeshBasicMaterial({ color: 0x3a3832 });
  function lampPost(x, y, z, s) {
    const p = []; for (let k = 0; k < 6; k++) { const a = k * Math.PI / 3; p.push([x + 0.09 * Math.cos(a), z + 0.09 * Math.sin(a)]); }
    GS.prism("timber", "timber", p, y, y + 2.6); GS.box("timber", x + s * 0.25, y + 2.55, z, 0.5, 0.06, 0.06);
    GS.box("iron", x + s * 0.45, y + 2.07, z, 0.26, 0.05, 0.26); GS.box("iron", x + s * 0.45, y + 2.45, z, 0.26, 0.05, 0.26);
    const m = new THREE.Mesh(new THREE.BoxGeometry(0.2, 0.33, 0.2), lampMat); m.position.set(x + s * 0.45, y + 2.27, z); scene.add(m); GLOW.push(m);
    LIGHTS.push({ x: x + s * 0.45, y: y + 2.2, z, R: 5.6, c: lin(0xffd27a), p: 1.6 });
  }
  const LAMPROWS = [0, -4, -8, -12, -16, -20, -24, -28];
  const LAMPCELL = new Set();
  LAMPROWS.forEach((r, i) => { const c = i % 2 ? -1 : 1; LAMPCELL.add(key(c, r)); lampPost(cx(c, r), TOP.get(key(c, r)), cz(c, r), -Math.sign(cx(c, r))); });
  // the game's trees (planet_surface.wgsl's), on the fields either side
  for (let r = R0; r <= R1; r++) for (let c = C0; c <= C1; c++) {
    const k = key(c, r), x = cx(c, r);
    if (KIND.get(k) !== "grass" || Math.abs(x) < W * 2.6 || LAMPCELL.has(k) || PBD.hash2(c * 3 + 11, r * 7 + 5) > 0.13) continue;
    PBD.tree(GS, hexPts(c, r), x, cz(c, r), TOP.get(k), c * 97 + r * 13, "a:bark", "a:leaves");
  }
  PBD.addMeshes(GS, scene, { cast: k => k !== "a:grass" && k !== "a:path" });
  PBD.setLights(LIGHTS);

  // stars, for the night
  const stars = (() => { const n = 700, p = new Float32Array(n * 3); let s = 7; const rn = () => ((s = (s * 16807) % 2147483647) / 2147483647);
    for (let i = 0; i < n; i++) { const a = rn() * 6.283, e = 0.06 + rn() * 1.4; p[3 * i] = Math.cos(a) * Math.cos(e) * 320; p[3 * i + 1] = Math.sin(e) * 320; p[3 * i + 2] = Math.sin(a) * Math.cos(e) * 320; }
    const g = new THREE.BufferGeometry(); g.setAttribute("position", new THREE.BufferAttribute(p, 3));
    const m = new THREE.Points(g, new THREE.PointsMaterial({ color: 0xdfe6ff, size: 1.3, sizeAttenuation: false, fog: false, transparent: true, opacity: 0 })); scene.add(m); return m; })();

  // ground height at a point: the column whose hexagon holds it
  function cellOf(x, z) {
    const qf = (Math.sqrt(3) / 3 * x - z / 3) / R, rf = (2 / 3 * z) / R, sf = -qf - rf;
    let q = Math.round(qf), r = Math.round(rf); const s = Math.round(sf), dq = Math.abs(q - qf), dr = Math.abs(r - rf), ds = Math.abs(s - sf);
    if (dq > dr && dq > ds) q = -r - s; else if (dr > ds) r = -q - s;
    return PBD.toOff(q, r);
  }
  const groundY = (x, z) => { const [c, r] = cellOf(x, z); return inMap(c, r) ? TOP.get(key(c, r)) : 0; };

  // ------------------------------------------------------------------ the other player
  // A loft: rings of superellipse sections up an axis, resampled by Catmull-Rom so the silhouette
  // curves; normals from the grid's own tangents, wrapping round the seam, so the body is smooth
  // shaded everywhere. Each ring: y, half width a, half depth b, roundness n, centre x and z.
  function loft(rings0, K, sub) {
    const keys = ["y", "a", "b", "n", "x", "z"], rings = [];
    const cr = (p0, p1, p2, p3, t) => 0.5 * (2 * p1 + (-p0 + p2) * t + (2 * p0 - 5 * p1 + 4 * p2 - p3) * t * t + (-p0 + 3 * p1 - 3 * p2 + p3) * t * t * t);
    for (let i = 0; i < rings0.length - 1; i++) for (let s = 0; s < sub; s++) {
      const t = s / sub, q = {}, r0 = rings0[i - 1] || rings0[i], r1 = rings0[i], r2 = rings0[i + 1], r3 = rings0[i + 2] || rings0[i + 1];
      for (const k of keys) q[k] = cr(r0[k] ?? 0, r1[k] ?? 0, r2[k] ?? 0, r3[k] ?? 0, t);
      q.a = Math.max(0.004, q.a); q.b = Math.max(0.004, q.b); q.n = Math.max(2, q.n); rings.push(q);
    }
    rings.push(Object.assign({ x: 0, z: 0 }, rings0[rings0.length - 1]));
    const sp = (v, e) => Math.sign(v) * Math.pow(Math.abs(v), e);
    const P = rings.map(r => { const row = []; for (let k = 0; k < K; k++) { const t = -Math.PI / 2 + 2 * Math.PI * k / K; row.push(new THREE.Vector3((r.x || 0) + r.a * sp(Math.cos(t), 2 / r.n), r.y, (r.z || 0) + r.b * sp(Math.sin(t), 2 / r.n))); } return row; });
    const NR = P.length, N = P.map(() => []), tu = new THREE.Vector3(), tv = new THREE.Vector3();
    for (let i = 0; i < NR; i++) for (let k = 0; k < K; k++) {
      tu.subVectors(P[i][(k + 1) % K], P[i][(k - 1 + K) % K]); tv.subVectors(P[Math.min(NR - 1, i + 1)][k], P[Math.max(0, i - 1)][k]);
      N[i][k] = new THREE.Vector3().crossVectors(tv, tu).normalize();
    }
    const y0 = rings[0].y, y1 = rings[NR - 1].y, pos = [], nor = [], uv = [];
    const put = (i, k) => { const kk = k % K, p = P[i][kk], n = N[i][kk]; pos.push(p.x, p.y, p.z); nor.push(n.x, n.y, n.z); uv.push(k / K, (rings[i].y - y0) / (y1 - y0)); };
    const up = y1 > y0;
    for (let i = 0; i < NR - 1; i++) for (let k = 0; k < K; k++) {
      const a = [i, k], b = [i, k + 1], c = [i + 1, k + 1], d = [i + 1, k];
      for (const v of up ? [a, c, b, a, d, c] : [a, b, c, a, c, d]) put(v[0], v[1]);
    }
    const g = new THREE.BufferGeometry();
    g.setAttribute("position", new THREE.Float32BufferAttribute(pos, 3)); g.setAttribute("normal", new THREE.Float32BufferAttribute(nor, 3)); g.setAttribute("uv", new THREE.Float32BufferAttribute(uv, 2));
    return { g, y0, y1 };
  }

  // The body, from the crotch to the crown (m, before the figure's 0.9 scale).
  const TRUNK = [
    { y: 0.74, a: 0.02, b: 0.02, n: 2.4 }, { y: 0.765, a: 0.16, b: 0.105, n: 2.6 }, { y: 0.84, a: 0.19, b: 0.12, n: 2.7 }, { y: 0.95, a: 0.196, b: 0.125, n: 2.8 },
    { y: 1.06, a: 0.2, b: 0.126, n: 2.8 }, { y: 1.19, a: 0.214, b: 0.132, n: 2.9 }, { y: 1.29, a: 0.214, b: 0.128, n: 3 }, { y: 1.345, a: 0.155, b: 0.1, n: 2.6 },
    { y: 1.385, a: 0.074, b: 0.07, n: 2 }, { y: 1.43, a: 0.08, b: 0.075, n: 2 }, { y: 1.465, a: 0.17, b: 0.16, n: 3 }, { y: 1.5, a: 0.205, b: 0.19, n: 4.2 },
    { y: 1.62, a: 0.212, b: 0.196, n: 4.6 }, { y: 1.75, a: 0.212, b: 0.196, n: 4.6 }, { y: 1.83, a: 0.204, b: 0.19, n: 4.4, z: -0.004 }, { y: 1.875, a: 0.16, b: 0.15, n: 3.4, z: -0.008 }, { y: 1.895, a: 0.012, b: 0.012, n: 2, z: -0.01 },
  ];
  const LEG = [ // from the hip joint
    { y: 0.07, a: 0.03, b: 0.03, n: 2 }, { y: 0.045, a: 0.085, b: 0.09, n: 2.3 }, { y: -0.1, a: 0.09, b: 0.095, n: 2.4 }, { y: -0.4, a: 0.073, b: 0.08, n: 2.3 },
    { y: -0.68, a: 0.063, b: 0.069, n: 2.2 }, { y: -0.74, a: 0.072, b: 0.078, n: 2.4, z: 0.005 }, { y: -0.81, a: 0.077, b: 0.1, n: 2.6, z: 0.025 },
    { y: -0.865, a: 0.08, b: 0.125, n: 2.8, z: 0.04 }, { y: -0.9, a: 0.076, b: 0.12, n: 2.8, z: 0.04 }, { y: -0.905, a: 0.012, b: 0.012, n: 2, z: 0.04 },
  ].reverse();
  const ARM = [ // from the shoulder joint
    { y: 0.06, a: 0.02, b: 0.02, n: 2 }, { y: 0.04, a: 0.07, b: 0.07, n: 2.3 }, { y: -0.05, a: 0.077, b: 0.075, n: 2.4 }, { y: -0.28, a: 0.064, b: 0.064, n: 2.3 },
    { y: -0.5, a: 0.056, b: 0.056, n: 2.2 }, { y: -0.53, a: 0.059, b: 0.059, n: 2.2 }, { y: -0.565, a: 0.047, b: 0.044, n: 2.2 }, { y: -0.625, a: 0.054, b: 0.04, n: 2.6 },
    { y: -0.685, a: 0.048, b: 0.034, n: 2.6 }, { y: -0.715, a: 0.01, b: 0.01, n: 2 },
  ].reverse();

  // ---- its textures: painted from the townsfolk's figure tiles, tinted per player
  const LOOK = { tunic: "#15578a", trousers: "#725244", leather: "#41322c", skin: "#e69d69", hair: "#522b2f", buckle: "#5d778d" };
  const TILES = {}, tileNames = ["cloth", "skin", "face", "hair", "leather"];
  const hexRGB = h => [parseInt(h.slice(1, 3), 16), parseInt(h.slice(3, 5), 16), parseInt(h.slice(5, 7), 16)];
  const hash = (a, b) => PBD.hash2(a, b);
  function tile(name, x, y) { const t = TILES[name]; if (!t) return [255, 255, 255]; const i = ((((y % 16) + 16) % 16) * 16 + (((x % 16) + 16) % 16)) * 4; return [t[i], t[i + 1], t[i + 2]]; }
  const mul = (c, tint, f = 1) => { const t = hexRGB(tint); return [c[0] * t[0] / 255 * f, c[1] * t[1] / 255 * f, c[2] * t[2] / 255 * f]; };
  function canvasTex(w, hgt, paint) {
    const cv = document.createElement("canvas"); cv.width = w; cv.height = hgt;
    const tex = new THREE.CanvasTexture(cv); tex.encoding = THREE.sRGBEncoding; tex.magFilter = tex.minFilter = THREE.NearestFilter; tex.generateMipmaps = false; tex.wrapS = THREE.RepeatWrapping;
    tex.userData = { draw() { const ctx = cv.getContext("2d"), im = ctx.createImageData(w, hgt); for (let py = 0; py < hgt; py++) for (let px = 0; px < w; px++) { const c = paint(px, py), i = (py * w + px) * 4; im.data[i] = c[0]; im.data[i + 1] = c[1]; im.data[i + 2] = c[2]; im.data[i + 3] = 255; } ctx.putImageData(im, 0, 0); tex.needsUpdate = true; } };
    return tex;
  }
  // the body: 64 round by 72 up, the crotch (0.74 m) at the bottom row and the crown at the top
  const BW = 64, BH = 72, BY0 = 0.74, BY1 = 1.895, bodyY = py => BY0 + (1 - (py + 0.5) / BH) * (BY1 - BY0), PXM = (BY1 - BY0) / BH;
  const bodyTex = canvasTex(BW, BH, (px, py) => {
    const y = bodyY(py), u = (px + 0.5) / BW, d = Math.abs(u - 0.5) * 2;   // d: 0 at the front, 1 at the back
    // the hairline: a fringe over the brow, down past the ears at the sides, to the nape behind,
    // stepping a pixel or two at random so the edge is ragged, not ruled
    const hairline = 1.795 - 0.27 * smooth(0.24, 0.86, d) + (hash(px, 3) < 0.45 ? PXM : 0) + (d < 0.24 && hash(px, 9) < 0.3 ? -PXM : 0);
    if (y >= hairline) return mul(tile("hair", px, py), LOOK.hair, 0.95 + 0.1 * smooth(1.8, 1.89, y));
    if (y >= 1.455) {
      const fx = px - 24, fy = Math.floor((1.812 - y) / PXM);
      if (fx >= 0 && fx < 16 && fy >= 0 && fy < 16) {
        const c = tile("face", fx, fy), l = (c[0] + c[1] + c[2]) / 765;
        if (l < 0.2) return [c[0] + 30, c[1] + 22, c[2] + 18];                              // the eyes
        if (fy >= 4 && fy <= 6 && l > 0.97 && (fx === 3 || fx === 11) && fy === 5) return [255, 250, 240]; // their glint
        if (fy === 3 && l < 0.6) return mul([255, 255, 255], LOOK.hair, 0.9);               // the brows
        if (l < 0.75 && fx > 0 && fx < 15) return mul([255, 255, 255], LOOK.skin, 0.62);    // the mouth
      }
      return mul(tile("skin", px, py), LOOK.skin, 1 - 0.06 * smooth(1.5, 1.46, y));
    }
    if (y >= 1.352) return mul(tile("skin", px, py), LOOK.skin, 0.9);
    if (y >= 0.99) {   // the tunic: a darker collar and hem
      const f = y > 1.33 ? 0.78 : y < 1.006 ? 0.84 : 1;
      return mul(tile("cloth", px, py), LOOK.tunic, f);
    }
    if (y >= 0.93) {   // the belt, its buckle at the front
      if (Math.abs(u - 0.5) < 0.035 && y > 0.94 && y < 0.983) return mul(tile("leather", px, py), LOOK.buckle, Math.abs(u - 0.5) < 0.017 && y > 0.952 && y < 0.97 ? 0.55 : 0.85);
      return mul(tile("leather", px, py), LOOK.leather, 1.25);
    }
    return mul(tile("cloth", px * 2, py), LOOK.trousers, 1);
  });
  const LEG_Y0 = -0.905, LEG_Y1 = 0.07, legTex = canvasTex(32, 48, (px, py) => {
    const y = LEG_Y0 + (1 - (py + 0.5) / 48) * (LEG_Y1 - LEG_Y0);
    if (y < -0.74) return mul(tile("leather", px, py), LOOK.leather, y > -0.76 ? 1.35 : y < -0.885 ? 0.8 : 1.1);
    return mul(tile("cloth", px, py), LOOK.trousers, y < -0.72 ? 0.85 : 1);
  });
  const ARM_Y0 = -0.715, ARM_Y1 = 0.06, armTex = canvasTex(32, 40, (px, py) => {
    const y = ARM_Y0 + (1 - (py + 0.5) / 40) * (ARM_Y1 - ARM_Y0);
    if (y < -0.553) return mul(tile("skin", px, py), LOOK.skin, 0.96);
    if (y < -0.515) return mul(tile("cloth", px, py), LOOK.tunic, 0.78);
    return mul(tile("cloth", px, py), LOOK.tunic, 1);
  });
  const ALLTEX = [bodyTex, legTex, armTex];
  ALLTEX.forEach(t => t.userData.draw());
  let tilesLeft = tileNames.length;
  for (const n of tileNames) {
    const im = new Image();
    im.onload = () => { const cv = document.createElement("canvas"); cv.width = cv.height = 16; const ctx = cv.getContext("2d"); ctx.drawImage(im, 0, 0); TILES[n] = ctx.getImageData(0, 0, 16, 16).data; if (--tilesLeft === 0) { ALLTEX.forEach(t => t.userData.draw()); dirty = true; } };
    im.src = PBD.base + "fig/fig_" + n + ".png?v=1";
  }

  const fig = new THREE.Group(); scene.add(fig);
  const body = new THREE.Group(); body.scale.setScalar(0.9); fig.add(body);
  const mesh = (geo, tex, parent) => { const m = new THREE.Mesh(geo, PBD.litMaterial(tex, 0xffffff)); m.castShadow = true; m.receiveShadow = true; parent.add(m); return m; };
  mesh(loft(TRUNK, 28, 3).g, bodyTex, body);
  const limb = (rings, tex, x, y, rz) => { const p = new THREE.Group(); p.position.set(x, y, 0); p.rotation.z = rz; body.add(p); mesh(loft(rings, 14, 3).g, tex, p); return p; };
  const legL = limb(LEG, legTex, -0.1, 0.9, 0), legR = limb(LEG, legTex, 0.1, 0.9, 0);
  const armL = limb(ARM, armTex, -0.226, 1.3, -0.07), armR = limb(ARM, armTex, 0.226, 1.3, 0.07);

  // ------------------------------------------------------------------ you, and where Wren walks
  const EYE = { x: -0.7, z: cz(0, 2) };            // you stand on the path's near end
  const ST = { dist: 9, target: 9, fx: 0.35, fz: EYE.z - 9, fy: 0, face: Math.PI, walk: 0, night: false, mode: "eyes" };
  ST.fy = groundY(ST.fx, ST.fz);
  const LOOKV = { yaw: 0, pitch: -0.05 }, ORB = { yaw: 0.5, pitch: 0.12, dist: 3.4 };
  function setDist(d) {
    ST.target = clamp(d, 2, 60);
    document.getElementById("pDist").value = Math.round(ST.target); document.getElementById("pDistO").textContent = Math.round(ST.target) + " m";
    document.querySelectorAll("[data-dist]").forEach(b => b.setAttribute("aria-pressed", String(+b.dataset.dist === Math.round(ST.target))));
    dirty = true;
  }
  document.querySelectorAll("[data-dist]").forEach(b => b.addEventListener("click", () => setDist(+b.dataset.dist)));
  document.getElementById("pDist").addEventListener("input", e => setDist(+e.target.value));
  document.querySelectorAll("[data-view]").forEach(b => b.addEventListener("click", () => {
    ST.mode = b.dataset.view; document.querySelectorAll("[data-view]").forEach(q => q.setAttribute("aria-pressed", String(q === b))); crossEl.style.display = ST.mode === "eyes" ? "" : "none"; dirty = true;
  }));

  // ------------------------------------------------------------------ day and night
  function setNight(n) {
    ST.night = n;
    const cA = new THREE.Color(n ? 0x0e1424 : 0x9fc6e2);
    scene.background = cA.clone(); scene.fog.color.copy(cA);
    if (!n) { const s = new THREE.Vector3(0.55, 0.78, 0.32).normalize(); sun.position.copy(s).multiplyScalar(90).add(new THREE.Vector3(0, 0, -28)); sun.color.setHSL(0.09, 0.5, 0.85); sun.intensity = 1.8; }
    else { const s = new THREE.Vector3(-0.4, 0.7, 0.55).normalize(); sun.position.copy(s).multiplyScalar(90).add(new THREE.Vector3(0, 0, -28)); sun.color.set(0x8fa6d8); sun.intensity = 0.035; } // the moon, dim enough that the lamps read
    sun.target.position.set(0, 0, -28);
    hemi.intensity = n ? 0.008 : 0.57; hemi.color.copy(cA).lerp(new THREE.Color(0xffffff), 0.35); hemi.groundColor.set(n ? 0x1a1a22 : 0x5a5040);
    PBD.LU.uLamp.value = n ? 0.85 : 0;
    lampMat.color.set(n ? 0xffd9a0 : 0x3a3832);
    stars.material.opacity = n ? 1 : 0;
    dirty = true;
  }

  // ------------------------------------------------------------------ input: drag to look (no easing)
  let renderer = null, running = false, raf = 0, last = 0, dirty = true, shownAt = 0;
  function makeRenderer() {
    renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setPixelRatio(Math.min(2, devicePixelRatio)); renderer.outputEncoding = THREE.sRGBEncoding;
    renderer.shadowMap.enabled = true; renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    const cv = renderer.domElement; cv.tabIndex = 0; cv.setAttribute("aria-label", "A path through the fields on hex columns, lamp posts beside it, and another player, Wren, standing on it. Drag to look.");
    view.prepend(cv);
    const pts = new Map(); let pinch = 0;
    cv.addEventListener("pointerdown", e => { cv.setPointerCapture(e.pointerId); pts.set(e.pointerId, [e.clientX, e.clientY]); if (pts.size === 2) { const [a, b] = [...pts.values()]; pinch = Math.hypot(a[0] - b[0], a[1] - b[1]); } });
    cv.addEventListener("pointermove", e => {
      const p = pts.get(e.pointerId); if (!p) return;
      if (pts.size === 2) { p[0] = e.clientX; p[1] = e.clientY; const [a, b] = [...pts.values()], d = Math.hypot(a[0] - b[0], a[1] - b[1]); if (pinch) ORB.dist = clamp(ORB.dist * pinch / d, 1.6, 14); pinch = d; dirty = true; return; }
      const dx = e.clientX - p[0], dy = e.clientY - p[1]; p[0] = e.clientX; p[1] = e.clientY;
      if (ST.mode === "eyes") { LOOKV.yaw -= dx * 0.005; LOOKV.pitch = clamp(LOOKV.pitch - dy * 0.005, -1.3, 1.3); }
      else { ORB.yaw -= dx * 0.008; ORB.pitch = clamp(ORB.pitch + dy * 0.006, -0.3, 1.3); }
      dirty = true;
    });
    const up = e => { pts.delete(e.pointerId); pinch = 0; };
    cv.addEventListener("pointerup", up); cv.addEventListener("pointercancel", up);
    cv.addEventListener("wheel", e => { if (ST.mode !== "orbit") return; e.preventDefault(); ORB.dist = clamp(ORB.dist * (e.deltaY > 0 ? 1.12 : 0.89), 1.6, 14); dirty = true; }, { passive: false });
    cv.addEventListener("keydown", e => {
      const k = e.key, st = 0.06;
      if (ST.mode === "eyes") { if (k === "ArrowLeft") LOOKV.yaw += st; else if (k === "ArrowRight") LOOKV.yaw -= st; else if (k === "ArrowUp") LOOKV.pitch = clamp(LOOKV.pitch + st, -1.3, 1.3); else if (k === "ArrowDown") LOOKV.pitch = clamp(LOOKV.pitch - st, -1.3, 1.3); else return; }
      else { if (k === "ArrowLeft") ORB.yaw += st; else if (k === "ArrowRight") ORB.yaw -= st; else if (k === "ArrowUp") ORB.pitch = clamp(ORB.pitch + st, -0.3, 1.3); else if (k === "ArrowDown") ORB.pitch = clamp(ORB.pitch - st, -0.3, 1.3); else if (k === "+" || k === "=") ORB.dist = clamp(ORB.dist * 0.89, 1.6, 14); else if (k === "-") ORB.dist = clamp(ORB.dist * 1.12, 1.6, 14); else return; }
      e.preventDefault(); dirty = true;
    });
    new ResizeObserver(() => { dirty = true; resize(); }).observe(view);
    resize();
  }
  function resize() { if (!renderer) return; const w = view.clientWidth, hh = view.clientHeight; renderer.setSize(w, hh, false); camera.aspect = w / Math.max(1, hh); camera.updateProjectionMatrix(); }

  // ------------------------------------------------------------------ the frame
  const tagV = new THREE.Vector3();
  function step(dt) {
    const want = EYE.z - ST.target, dz = want - ST.fz, speed = 3.2;   // walking.rs's pace, near enough
    let moving = false;
    if (Math.abs(dz) > 0.02) { const s = Math.sign(dz) * Math.min(Math.abs(dz), speed * dt); ST.fz += s; ST.walk += Math.abs(s) * 2.6; moving = true; }
    else if (ST.walk) { const ph = ST.walk % Math.PI; ST.walk = ph < 0.08 || ph > Math.PI - 0.08 ? 0 : ST.walk + dt * 7; moving = ST.walk !== 0; }
    // face the way walked, and when stopped turn to face you
    const faceWant = Math.abs(dz) > 0.02 ? (dz < 0 ? Math.PI : 0) : Math.atan2(EYE.x - ST.fx, EYE.z - ST.fz);
    let df = ((faceWant - ST.face + Math.PI * 3) % (Math.PI * 2)) - Math.PI;
    if (Math.abs(df) > 0.002) { ST.face += Math.sign(df) * Math.min(Math.abs(df), dt * 6); moving = true; }
    const gy = groundY(ST.fx, ST.fz); if (Math.abs(gy - ST.fy) > 0.002) { ST.fy += (gy - ST.fy) * Math.min(1, dt * 9); moving = true; } else ST.fy = gy;
    const sw = Math.sin(ST.walk);
    legL.rotation.x = 0.5 * sw; legR.rotation.x = -0.5 * sw; armL.rotation.x = -0.38 * sw; armR.rotation.x = 0.38 * sw;
    fig.position.set(ST.fx, ST.fy + Math.abs(Math.sin(ST.walk)) * 0.025, ST.fz); fig.rotation.y = ST.face;
    return moving;
  }
  function placeCamera() {
    if (ST.mode === "eyes") { camera.position.set(EYE.x, groundY(EYE.x, EYE.z) + 1.62, EYE.z); camera.rotation.set(LOOKV.pitch, LOOKV.yaw, 0); }
    else {
      const tx = ST.fx, ty = ST.fy + 1.05, tz = ST.fz, cp = Math.cos(ORB.pitch);
      camera.position.set(tx + Math.sin(ORB.yaw) * cp * ORB.dist, ty + Math.sin(ORB.pitch) * ORB.dist, tz + Math.cos(ORB.yaw) * cp * ORB.dist);
      camera.lookAt(tx, ty, tz);
    }
  }
  function placeTag() {
    const d = camera.position.distanceTo(new THREE.Vector3(ST.fx, ST.fy + 1.0, ST.fz));
    tagV.set(ST.fx, ST.fy + 1.84, ST.fz).project(camera);
    const on = d <= NAME_M && tagV.z < 1 && tagV.z > -1;
    tagEl.classList.toggle("off", !on);
    if (on) { const w = view.clientWidth, hh = view.clientHeight; tagEl.style.left = Math.round((tagV.x * 0.5 + 0.5) * w) + "px"; tagEl.style.top = Math.round((-tagV.y * 0.5 + 0.5) * hh) + "px"; }
  }
  function frame(t) {
    if (!running) return;
    raf = requestAnimationFrame(frame);
    const dt = Math.min(0.05, (t - last) / 1000 || 0); last = t;
    const moving = step(dt);
    if (!moving && !dirty && t - shownAt > 3000) return;   // the first seconds draw while the textures load
    dirty = false;
    placeCamera(); renderer.render(scene, camera); placeTag();
  }
  tagEl.innerHTML = '<span class="pt">' + window.ptSvg(NAME) + "</span>";

  window.PLAYER3D = {
    show() { if (!renderer) makeRenderer(); if (!running) { running = true; dirty = true; last = shownAt = performance.now(); raf = requestAnimationFrame(frame); } },
    hide() { running = false; cancelAnimationFrame(raf); },
    setNight,
    // for the page's checks: jump Wren to a distance at once, and draw
    _jump(d) { setDist(d); ST.fz = EYE.z - ST.target; ST.fy = groundY(ST.fx, ST.fz); ST.face = Math.atan2(EYE.x - ST.fx, EYE.z - ST.fz); ST.walk = 0; dirty = true; },
    _orbit(yaw, pitch, dist) { Object.assign(ORB, { yaw, pitch, dist }); dirty = true; },
    _look(yaw, pitch) { Object.assign(LOOKV, { yaw, pitch }); dirty = true; },
    ST,
  };
  setNight(false); setDist(9); ST.fz = EYE.z - 9; ST.fy = groundY(ST.fx, ST.fz); ST.face = Math.atan2(EYE.x - ST.fx, EYE.z - ST.fz);
})();
