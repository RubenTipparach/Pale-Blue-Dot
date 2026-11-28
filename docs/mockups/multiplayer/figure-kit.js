// figure-kit.js: M5's townsperson kit, the friendly figure the game draws (townsfolk design 13a,
// 21, 36), copied verbatim from docs/mockups/npcs.html (the window.FK block) so another player
// here is the same figure as the game's NPCs (owner, 2026-11-27: "humans should be consistent with
// the NPCs in the game"). FK_FIELDS is that page's Fields folk palette, also copied.
window.FK = (function () {
const lin = h => new THREE.Color(h).convertSRGBToLinear();
const clamp = (v, a, b) => v < a ? a : v > b ? b : v;
function rng(seed) { let s = (seed >>> 0) || 1; return () => { s ^= s << 13; s >>>= 0; s ^= s >>> 17; s ^= s << 5; s >>>= 0; return s / 4294967296; }; }
const LAMP_GLOW = new THREE.MeshBasicMaterial({ color: 0xfff0b0 });
// beards (design 36a, version 11): one shell per beard laid over the head's front and front bevels, never
// boxes. BEARD_U are the grid's column edges either side of the middle (0.215 the bevel's start, 0.2645 its end at
// the side), BEARD_Y its row edges (0 the head's bottom, 0.058 and 0.148 the mouth's lowest and highest pixel). A
// stencil per style, top row first, middle column first, mirrored: "#" beard, "+" stubble lying flush in a
// colour between the hair and the skin. The game's figure::beard_shell builds the same shell from the same data.
const BEARD_U = [0, 0.035, 0.07, 0.108, 0.14, 0.175, 0.215, 0.24, 0.215 + 0.035 * Math.SQRT2];
const BEARD_Y = [-0.11, -0.075, -0.04, 0, 0.035, 0.058, 0.103, 0.148, 0.163, 0.178, 0.208, 0.238, 0.268];
const BEARDS = {
  none: null,
  goatee: ["........", "........", "........", "+##.....", "+####...", "...##...", "...#....", "+###+...", "###+....", "##......", "#.......", "........"],
  full:   ["......+#", "......##", "......##", "+###.###", "+#######", "...#####", "...#####", "+#######", "#######+", "#####...", "###.....", "#......."],
};
const BEARD_SINK = 0.005, BEARD_EDGE = 0.004, BEARD_STUBBLE = 0.0015, BEARD_STUBBLE_MIX = 0.45;
const beardCore = y => y < 0.058 ? 0.034 : y < 0.148 ? 0.026 : y < 0.178 ? 0.03 : 0.018;
// the shell: { beard, stubble } geometries in the head's space (front sheet, walls round its edge, an underside
// where it hangs below the chin), each cell two triangles on an alternating diagonal
function beardShell(style) {
  const M = BEARDS[style]; if (!M) return null;
  const S2 = Math.SQRT1_2, FH = HW / 2 - HC, nc = 16, nr = BEARD_Y.length - 1;
  const U = [...BEARD_U.slice(1).reverse().map(u => -u), ...BEARD_U];
  const cell = (c, r) => c < 0 || c >= nc || r < 0 || r >= nr ? "." : M[nr - 1 - r][c >= 8 ? c - 8 : 7 - c];
  const isB = (c, r) => cell(c, r) === "#", isF = (c, r) => cell(c, r) !== ".";
  const norm = v => { const l = Math.hypot(...v); return v.map(k => k / l); };
  const base = (u, inset) => {
    const a = Math.abs(u), s = u < 0 ? -1 : 1, fh = FH - inset;
    if (a <= FH) return { x: u * fh / FH, z: HA - inset, n: Math.abs(a - FH) < 1e-6 ? norm([s * S2, 0, 1 + S2]) : [0, 0, 1] };
    const q = (a - FH) * S2; return { x: s * (fh + q), z: HA - inset - q, n: [s * S2, 0, S2] };
  };
  const nb = (i, j) => [isB(i - 1, j - 1), isB(i, j - 1), isB(i - 1, j), isB(i, j)].filter(Boolean).length;
  const edge = []; for (let i = 0; i <= nc; i++) for (let j = 0; j <= nr; j++) { const k = nb(i, j); if (k > 0 && k < 4) edge.push([U[i], BEARD_Y[j]]); }
  const F = [], Bk = [];
  for (let i = 0; i <= nc; i++) { F.push([]); Bk.push([]); for (let j = 0; j <= nr; j++) {
    const u = U[i], y = BEARD_Y[j], k = nb(i, j), core = beardCore(y);
    let t;
    if (k === 0) t = BEARD_STUBBLE;
    else if (k < 4) t = BEARD_EDGE + (core - BEARD_EDGE) * (k - 1) / 3 * 0.65;
    else { let d = 1; for (const [eu, ey] of edge) d = Math.min(d, Math.hypot(eu - u, ey - y)); t = core * (0.7 + 0.3 * Math.min(1, d / 0.06)); }
    const f = base(u, 0), fr = [f.x + f.n[0] * t, y, f.z + f.n[2] * t];
    if (y < 0) fr[2] += y * 0.15;
    const inset = y >= HC ? 0 : y >= 0 ? HC - y : HC, b = base(u, inset), bk = [b.x - b.n[0] * BEARD_SINK, y, b.z - b.n[2] * BEARD_SINK];
    if (y < 0) bk[2] = Math.min(HA - HC - BEARD_SINK - y * 0.5, fr[2] - BEARD_EDGE);
    F[i].push(fr); Bk[i].push(bk);
  } }
  const G = { beard: new PBD.Geo(), stubble: new PBD.Geo() };
  const sub = (a, b) => [a[0] - b[0], a[1] - b[1], a[2] - b[2]], mid = (...p) => [0, 1, 2].map(k => p.reduce((s, q) => s + q[k], 0) / p.length);
  for (let c = 0; c < nc; c++) for (let r = 0; r < nr; r++) {
    if (!isF(c, r)) continue;
    const g = isB(c, r) ? G.beard : G.stubble, a = F[c][r], b = F[c + 1][r], cc = F[c + 1][r + 1], d = F[c][r + 1], out = mid(a, b, cc, d);
    const want = [0, 0, 1];
    if ((c + r) % 2) { g.face([a, b, cc], want); g.face([a, cc, d], want); } else { g.face([a, b, d], want); g.face([b, cc, d], want); }
    if (BEARD_Y[r] < 0) g.face([Bk[c][r], Bk[c + 1][r], Bk[c + 1][r + 1], Bk[c][r + 1]], [0, -0.7, -1]);
    // a wall on each side that meets no cell, from the front sheet into the head
    for (const [dc, dr, i0, j0, i1, j1] of [[0, -1, c, r, c + 1, r], [1, 0, c + 1, r, c + 1, r + 1], [0, 1, c + 1, r + 1, c, r + 1], [-1, 0, c, r + 1, c, r]]) {
      if (isF(c + dc, r + dr)) continue;
      g.face([F[i0][j0], F[i1][j1], Bk[i1][j1], Bk[i0][j0]], sub(mid(F[i0][j0], F[i1][j1]), out));
    }
  }
  smoothGeos([G.beard, G.stubble], SMOOTH_CREASE);
  return { beard: G.beard.build(1), stubble: G.stubble.build(1) };
}
// soft shading (design 36a and 38, owner 2026-11-22: "soft shade the beard, not flat shade"): each corner's normal
// becomes the mean of the face normals of the corners at its exact point whose face is within the crease of its
// own (cos 0.5, 60 degrees); sharper turns stay crisp. The game's Part::smooth_from does the same.
const SMOOTH_CREASE = 0.5;
function smoothGeos(geos, crease) {
  const at = new Map(), all = [];
  for (const g of geos) for (let i = 0; i < g.p.length / 3; i++) {
    const k = g.p[3 * i] + "," + g.p[3 * i + 1] + "," + g.p[3 * i + 2], f = [g.n[3 * i], g.n[3 * i + 1], g.n[3 * i + 2]];
    all.push([g, i, k, f]); if (!at.has(k)) at.set(k, []); at.get(k).push(f);
  }
  for (const [g, i, k, f] of all) {
    let x = 0, y = 0, z = 0; for (const o of at.get(k)) if (f[0] * o[0] + f[1] * o[1] + f[2] * o[2] >= crease) { x += o[0]; y += o[1]; z += o[2]; }
    const l = Math.hypot(x, y, z); if (l > 1e-9) { g.n[3 * i] = x / l; g.n[3 * i + 1] = y / l; g.n[3 * i + 2] = z / l; }
  }
}
// ears (design 38, version 12): one tapered, slightly cupped form grown from the head's side face, never a box.
// Three rings of an eight-point outline (root 1.3x sunk 2 mm so the foot flares into the head like a bevel, the
// outline 14 mm out, a rim 0.86x 42 mm out leaning back 10 mm at its top) and a bowl inside the rim falling to
// 26 mm out; the outside in the skin, the cup in the skin's shade; soft shaded. The game's figure::ear.
const EAR_AT = [-0.005, 0.26], EAR_LEAN = 0.01, EAR_CUP = [0.026, -0.004];
const EAR_OUTLINE = [[0, 0.078], [0.04, 0.058], [0.05, 0.01], [0.03, -0.045], [0.006, -0.075], [-0.03, -0.06], [-0.05, -0.005], [-0.042, 0.05]];
const EAR_RINGS = [[1.3, -0.002], [1.0, 0.014], [0.86, 0.042]];
function earGeo(s) {
  const x0 = HW / 2, G = { skin: new PBD.Geo(), inner: new PBD.Geo() };
  const R = EAR_RINGS.map(([sc, out], k) => EAR_OUTLINE.map(([dz, dy]) => [s * (x0 + out), EAR_AT[1] + dy * sc, EAR_AT[0] + dz * sc + (k === 2 ? -EAR_LEAN * dy / 0.078 : 0)]));
  for (let k = 0; k < 2; k++) for (let i = 0; i < 8; i++) {
    const a = R[k][i], b = R[k][(i + 1) % 8], c = R[k + 1][(i + 1) % 8], d = R[k + 1][i];
    const rad = [0, (a[1] + b[1]) / 2 - EAR_AT[1], (a[2] + b[2]) / 2 - EAR_AT[0]], rl = Math.hypot(rad[1], rad[2]);
    G.skin.face([a, b, c, d], [s * 0.5 * rl, rad[1], rad[2]]);
  }
  const cup = [s * (x0 + EAR_CUP[0]), EAR_AT[1], EAR_AT[0] + EAR_CUP[1]];
  for (let i = 0; i < 8; i++) G.inner.face([R[2][i], R[2][(i + 1) % 8], cup], [s, 0, 0]);
  smoothGeos([G.skin, G.inner], SMOOTH_CREASE);
  return { skin: G.skin.build(1), inner: G.inner.build(1) };
}
const beardOf = spec => spec.beard === true ? "goatee" : BEARDS[spec.beard] ? spec.beard : "none";
// ---- materials: lit like the ground (scene-kit.js), one cache by colour and face layer -------------
const MC = {};
const shade = (hex, k) => { const c = new THREE.Color(hex); return new THREE.Color(c.r * k, c.g * k, c.b * k).getHex(); };
const mixHex = (a, b, t) => new THREE.Color(a).lerp(new THREE.Color(b), t).getHex();
function mat(hex, layer = 0) {
  const k = hex + ":" + layer; if (MC[k]) return MC[k];
  const extra = layer ? { polygonOffset: true, polygonOffsetFactor: -1, polygonOffsetUnits: -2 * layer } : {};
  return (MC[k] = PBD.litMaterial(null, lin(hex), extra));
}

// ---- the face atlas: pixel art drawn in a canvas, 4 x 3 cells of 24 x 12 px, sampled nearest --------
// The "happy" upturned eye arcs follow the High Frontier fan game's character kit
// (rubentipparach/high-frontier-fan-game @ 6cbe5b3, scripts/rig2d/cutout.mjs faceShapes 'happy'),
// and the dark pupil with a white catch-light its rover mascot (scratchpad/mascot-buggy2.html);
// both are redrawn here as pixels and discs, not copied.
const AC = document.createElement("canvas"); AC.width = 96; AC.height = 36;
const ax = AC.getContext("2d");
const CELL = { smile: 0, grin: 1, o: 2, flat: 3, frown: 4, wobble: 5, A: 6, O: 7, M: 8, happy: 9, closed: 10 };
const LIP = "#5c2228", IN = "#3e1219", TNG = "#d8707a", TTH = "#f4efe6", INK = "#23203a";
function px(cell, x, y, col) { if (x < 0 || x > 23 || y < 0 || y > 11) return; ax.fillStyle = col; ax.fillRect((cell % 4) * 24 + x, Math.floor(cell / 4) * 12 + y, 1, 1); }
function curve(cell, x0, x1, yEnd, yMid, th, col) { const c = (x0 + x1) / 2, h = (x1 - x0) / 2; for (let x = x0; x <= x1; x++) { const t = (x - c) / h, y = Math.round(yEnd + (yMid - yEnd) * (1 - t * t)); for (let k = 0; k < th; k++) px(cell, x, y + k, col); } }
function oval(cell, cx, cy, rx, ry, teeth, tongue) {
  for (let y = 0; y < 12; y++) for (let x = 0; x < 24; x++) {
    const d = ((x + 0.5 - cx) / rx) ** 2 + ((y + 0.5 - cy) / ry) ** 2; if (d > 1) continue;
    let c = d > 0.62 ? LIP : IN;
    if (teeth && c === IN && y + 0.5 < cy - ry * 0.45) c = TTH;
    if (tongue && c === IN && y + 0.5 > cy + ry * 0.25 && Math.abs(x + 0.5 - cx) < rx * 0.55) c = TNG;
    px(cell, x, y, c);
  }
}
curve(CELL.smile, 6, 17, 4, 7, 2, LIP);
(function grin() { const c = CELL.grin; for (let x = 5; x <= 18; x++) { const t = (x - 11.5) / 6.5, bot = Math.round(4 + 6 * (1 - t * t)); for (let y = 3; y <= bot; y++) { let col = (y === 3 || y === bot) ? LIP : IN; if (col === IN && y === 4 && Math.abs(t) < 0.8) col = TTH; if (col === IN && y >= bot - 2 && Math.abs(t) < 0.45) col = TNG; px(c, x, y, col); } } })();
oval(CELL.o, 12, 6.5, 3.0, 4.0, false, true);
for (let x = 8; x <= 15; x++) { px(CELL.flat, x, 6, LIP); px(CELL.flat, x, 7, LIP); }
curve(CELL.frown, 7, 16, 8, 5, 2, LIP);
for (let x = 6; x <= 17; x++) { const y = 6 + Math.round(Math.sin((x - 6) * Math.PI / 2.6) * 1.1); px(CELL.wobble, x, y, LIP); px(CELL.wobble, x, y + 1, LIP); }
oval(CELL.A, 12, 6, 4.6, 4.2, true, true);
oval(CELL.O, 12, 6, 2.8, 3.0, false, true);
for (let x = 7; x <= 16; x++) { px(CELL.M, x, 5, LIP); px(CELL.M, x, 6, "#8a3a42"); }
curve(CELL.happy, 3, 20, 9, 2, 3, INK);
curve(CELL.closed, 4, 19, 3, 7, 2, INK);
const atlasTex = new THREE.CanvasTexture(AC); atlasTex.magFilter = atlasTex.minFilter = THREE.NearestFilter; atlasTex.generateMipmaps = false; atlasTex.encoding = THREE.sRGBEncoding;
const atlasMat = {};
const aMat = layer => atlasMat[layer] || (atlasMat[layer] = PBD.litMaterial(atlasTex, 0xffffff, { alphaTest: 0.5, polygonOffset: true, polygonOffsetFactor: -1, polygonOffsetUnits: -2 * layer }));
function cellQuad(w, h) { return new THREE.PlaneGeometry(w, h); }
function setCell(geo, name) {
  const c = CELL[name], u0 = (c % 4) / 4, u1 = u0 + 0.25, r = Math.floor(c / 4), v1 = 1 - r / 3, v0 = 1 - (r + 1) / 3, uv = geo.attributes.uv;
  uv.setXY(0, u0, v1); uv.setXY(1, u1, v1); uv.setXY(2, u0, v0); uv.setXY(3, u1, v0); uv.needsUpdate = true;
}

// ---- six-sided geometry: lathes and prisms, flat side forward (+z), flat normals ------------------
const SIX = [0, 1, 2, 3, 4, 5].map(k => k * Math.PI / 3);
// profile: [[y, r]] bottom to top, r the circumradius; o: { sx, sz, cz, open0, open1 }
function lathe(profile, o = {}) {
  const g = new PBD.Geo(), sx = o.sx || 1, sz = o.sz || 1, cz = o.cz || 0;
  const ring = ([y, r]) => SIX.map(a => [r * Math.cos(a) * sx, y, r * Math.sin(a) * sz + cz]);
  for (let i = 0; i + 1 < profile.length; i++) {
    const A = ring(profile[i]), B = ring(profile[i + 1]);
    for (let k = 0; k < 6; k++) {
      const m = (k + 0.5) * Math.PI / 3, want = [Math.cos(m), 0, Math.sin(m)], a0 = A[k], a1 = A[(k + 1) % 6], b0 = B[k], b1 = B[(k + 1) % 6];
      if (profile[i][1] === 0) g.face([a0, b1, b0], want);
      else if (profile[i + 1][1] === 0) g.face([a0, a1, b0], want);
      else g.face([a0, a1, b1, b0], want);
    }
  }
  const f = profile[0], l = profile[profile.length - 1];
  if (f[1] > 0 && !o.open0) g.face(ring(f), [0, -1, 0]);
  if (l[1] > 0 && !o.open1) g.face(ring(l), [0, 1, 0]);
  return g.build(1);
}
const disc = r => new THREE.CircleGeometry(r, 6);
// a block (version 9, design 13a): a w x h x d box standing on y0, its four upright edges cut by c and its
// top and bottom edges by cTop and cBot (a small bevel), flat normals; o: { x, z, cTop, cBot }
function block(w, h, d, c, y0 = 0, o = {}) {
  const g = new PBD.Geo(), cx = o.x || 0, cz = o.z || 0, ct = o.cTop ?? c, cb = o.cBot ?? c;
  const ring = (y, i) => {
    const a = w / 2 - i, b = d / 2 - i, k = Math.min(c, a * 0.9, b * 0.9);
    const p = k > 1e-6 ? [[a, b - k], [a - k, b], [-a + k, b], [-a, b - k], [-a, -b + k], [-a + k, -b], [a - k, -b], [a, -b + k]] : [[a, b], [-a, b], [-a, -b], [a, -b]];
    return p.map(([x, z]) => [x + cx, y, z + cz]);
  };
  const prof = [];
  if (cb > 0) prof.push([y0, cb]); prof.push([y0 + cb, 0], [y0 + h - ct, 0]); if (ct > 0) prof.push([y0 + h, ct]);
  const R = prof.map(([y, i]) => ring(y, i));
  for (let i = 0; i + 1 < R.length; i++) {
    const A = R[i], B = R[i + 1], n = A.length;
    for (let k = 0; k < n; k++) { const a0 = A[k], a1 = A[(k + 1) % n], b0 = B[k], b1 = B[(k + 1) % n]; g.face([a0, a1, b1, b0], [(a0[0] + a1[0]) / 2 - cx, 0, (a0[2] + a1[2]) / 2 - cz]); }
  }
  g.face(R[0], [0, -1, 0]); g.face(R[R.length - 1], [0, 1, 0]);
  return g.build(1);
}

// ---- the proposed townsperson -----------------------------------------------------------------------
// Version 9 (design 13a): a block head, 0.5 m wide, 0.6 m tall and 0.45 m deep with a small bevel, on the
// short rounded body version 8 approved; the face fills the block's flat front.
const HW = 0.5, HH = 0.6, HD = 0.45, HC = 0.035;
const HA = HD / 2;   // the head's flat front: the face sits on it
const EYE_Y = 0.285, MOUTH_Y = 0.108, BROW_Y = 0.38, HAIR_Y = 0.48;   // HAIR_Y: no hair or hat comes below it at the front
// face pieces about 1.55 times version 8's (white r 0.052), the mouth twice as wide (0.12 m)
const WR = 0.08, PR = 0.048, GR = 0.016, FSC = 1.6;
const BUILDS = {
  round: { w: 1.0, legs: 0.40, prof: [[0, .17], [.12, .21], [.45, .25], [.78, .22], [.95, .15], [1, .10]] },
  pear:  { w: 1.0, legs: 0.39, prof: [[0, .21], [.15, .25], [.45, .24], [.75, .18], [.95, .13], [1, .09]] },
  tall:  { w: 0.92, legs: 0.46, prof: [[0, .15], [.15, .18], [.5, .19], [.8, .18], [.95, .13], [1, .09]] },
  stout: { w: 1.08, legs: 0.36, prof: [[0, .20], [.12, .25], [.45, .27], [.78, .25], [.95, .17], [1, .11]] },
};
function profAt(prof, t) { for (let i = 0; i + 1 < prof.length; i++) { const [a, ra] = prof[i], [b, rb] = prof[i + 1]; if (t <= b) return ra + (rb - ra) * (t - a) / (b - a); } return prof[prof.length - 1][1]; }
function cutProf(prof, t0, t1, H, w) {
  const out = [[t0, profAt(prof, t0)]]; for (const p of prof) if (p[0] > t0 && p[0] < t1) out.push(p); out.push([t1, profAt(prof, t1)]);
  return out.map(([t, r]) => [t * H, r * w]);
}
const EXPR = {
  neutral:   { eye: 1.00, pup: 1.00, lid: 0.12, by: 0.000, bt: 0.00, mouth: "flat",   blush: 0 },
  happy:     { eye: 1.04, pup: 1.06, lid: 0.05, by: 0.008, bt: 0.06, mouth: "smile",  blush: 0.7 },
  delighted: { eye: 1.05, pup: 1.00, lid: 0.00, by: 0.020, bt: 0.08, mouth: "grin",   blush: 1, arcs: true, act: "hop" },
  surprised: { eye: 1.15, pup: 0.70, lid: 0.00, by: 0.026, bt: 0.03, mouth: "o",      blush: 0 },
  worried:   { eye: 1.02, pup: 0.95, lid: 0.10, by: 0.012, bt: 0.34, mouth: "wobble", blush: 0 },
  sad:       { eye: 0.98, pup: 1.08, lid: 0.16, by: 0.002, bt: 0.38, mouth: "frown",  blush: 0, look: [0.1, -0.6] },
  cross:     { eye: 0.95, pup: 0.90, lid: 0.30, by: -0.014, bt: -0.36, mouth: "frown", blush: 0 },
  sleepy:    { eye: 1.00, pup: 1.00, lid: 0.62, by: -0.006, bt: -0.03, mouth: "M",    blush: 0, look: [0, -0.4] },
  thinking:  { eye: 1.00, pup: 1.00, lid: 0.14, by: 0.004, bt: 0.00, mouth: "flat",   blush: 0, asym: 0.022, mx: 0.022, look: [0.75, 0.85], act: "tilt" },
  talking:   { eye: 1.03, pup: 1.05, lid: 0.05, by: 0.010, bt: 0.06, mouth: "smile",  blush: 0.35, talk: true },
};
const EXPR_KEYS = Object.keys(EXPR);
const NUMK = ["eye", "pup", "lid", "by", "bt", "blush", "asym", "mx"];

function friend(spec) {
  const B = BUILDS[spec.build], hs = spec.head, headH = HH * hs, rest = spec.h - headH + 0.03;
  const legLen = rest * B.legs, torsoH = rest - legLen, w = B.w;
  const skin = spec.skin, skinS = shade(skin, 0.84), shirtS = shade(spec.shirt, 0.74), pantsS = shade(spec.pants, 0.72);
  const browC = shade(spec.hairC, 0.62), cast = [];
  const g = new THREE.Group(), body = new THREE.Group(); g.add(body); body.position.y = legLen;
  const mesh = (geo, m, parent, x = 0, y = 0, z = 0, shadow = true) => { const o = new THREE.Mesh(geo, m); o.position.set(x, y, z); o.castShadow = shadow; o.receiveShadow = true; parent.add(o); return o; };
  // torso: trousers below, the shirt above, a belt and a collar in their own shade (two tones a material)
  mesh(lathe(cutProf(B.prof, 0, 0.22, torsoH, w)), mat(spec.pants), body);
  mesh(lathe(cutProf(B.prof, 0.22, 1, torsoH, w)), mat(spec.shirt), body);
  mesh(lathe(cutProf(B.prof, 0.19, 0.26, torsoH, w * 1.045), { open0: true, open1: true }), mat(pantsS), body);
  mesh(lathe(cutProf(B.prof, 0.9, 1.0, torsoH, w * 1.06), { open0: true }), mat(shirtS), body);
  // legs and boots
  const legs = [-1, 1].map(s => {
    const p = new THREE.Group(); p.position.set(s * 0.085 * w, 0.03, 0); body.add(p);
    mesh(lathe([[-(legLen - 0.05), 0.068], [0, 0.068]]), mat(spec.pants), p);
    mesh(lathe([[-legLen - 0.03, 0.085], [-legLen + 0.06, 0.085]], { sz: 1.3, cz: 0.03 }), mat(spec.shoe), p);
    return p;
  });
  // stubby arms: a sleeve, a cuff in the shirt's shade, a round six-sided hand
  const shY = torsoH * 0.84, shX = profAt(B.prof, 0.84) * w + 0.035, armL = Math.max(0.24, torsoH * 0.56);
  const arms = [-1, 1].map(s => {
    const p = new THREE.Group(); p.position.set(s * shX, shY, 0); body.add(p);
    mesh(lathe([[-armL + 0.03, 0.05], [-0.02, 0.055], [0.03, 0.045]]), mat(spec.shirt), p);
    mesh(lathe([[-armL, 0.057], [-armL + 0.05, 0.057]]), mat(shirtS), p);
    mesh(lathe([[-armL - 0.1, 0], [-armL - 0.075, 0.056], [-armL - 0.02, 0.056], [-armL + 0.005, 0.03]]), mat(skin), p);
    p.rotation.z = s * 0.12; return p;
  });
  // the head, and the face on its flat front
  const head = new THREE.Group(); head.position.set(0, torsoH - 0.035, 0); head.scale.setScalar(hs); body.add(head);
  mesh(block(HW, HH, HD, HC, 0), mat(skin), head);
  if (spec.ears) for (const s of [-1, 1]) { const E = earGeo(s); mesh(E.skin, mat(skin), head); mesh(E.inner, mat(skinS), head); }
  if (spec.nose === "button") mesh(lathe([[0.14, 0], [0.152, 0.032], [0.188, 0.032], [0.2, 0]], { cz: HA + 0.012 }), mat(skinS), head, 0, 0, 0, false);
  if (spec.nose === "long") mesh(lathe([[0.138, 0], [0.152, 0.028], [0.192, 0.028], [0.206, 0]], { sz: 2.6, cz: HA + 0.046 }), mat(skinS), head, 0, 0, 0, false);
  const hair = mat(spec.hairC), hairS = mat(shade(spec.hairC, 0.78));
  // hair is blocks sized to the head block: a cap whose front edge stops at HAIR_Y (above the highest brow),
  // a slab down the back, and short sideburns above the ears
  const back = () => mesh(block(HW + 0.03, 0.4, 0.07, 0.02, 0.12, { z: -HD / 2 - 0.012 }), hair, head);
  const crop = () => { mesh(block(HW + 0.03, HH + 0.04 - HAIR_Y, HD + 0.03, 0.04, HAIR_Y, { cBot: 0 }), hair, head); back(); for (const s of [-1, 1]) mesh(block(0.025, 0.16, 0.2, 0, 0.34, { x: s * (HW / 2 + 0.01), z: -0.08 }), hair, head); };
  if (spec.hair === "crop") crop();
  if (spec.hair === "bun") { crop(); mesh(block(0.18, 0.13, 0.16, 0.035, HH + 0.02, { z: -0.07 }), hairS, head); }
  if (spec.hair === "braid") { crop(); for (let i = 0; i < 3; i++) mesh(block(0.085, 0.1, 0.075, 0.02, 0.1 - i * 0.1, { z: -HD / 2 - 0.04 - i * 0.008 }), i % 2 ? hairS : hair, head); }
  if (spec.hair === "bald") for (const s of [-1, 1]) mesh(block(0.03, 0.12, 0.17, 0.01, 0.35, { x: s * (HW / 2 + 0.01), z: -0.1 }), hair, head);
  if (spec.hair === "hat") {
    const hk = spec.hatKind || "brim", hc = mat(spec.hatC), hcs = mat(shade(spec.hatC, 0.74));
    if (hk === "brim") {
      mesh(block(HW + 0.24, 0.03, HD + 0.24, 0.08, HAIR_Y + 0.03, { cTop: 0, cBot: 0 }), hcs, head);
      mesh(block(HW + 0.05, 0.045, HD + 0.05, 0.05, HAIR_Y + 0.06, { cTop: 0, cBot: 0 }), hcs, head);
      mesh(block(HW + 0.03, 0.2, HD + 0.03, 0.05, HAIR_Y + 0.06, { cBot: 0 }), hc, head); back();
    } else if (hk === "cap") {
      mesh(block(HW + 0.04, HH + 0.06 - HAIR_Y, HD + 0.04, 0.05, HAIR_Y, { cBot: 0 }), hc, head); back();
      mesh(block(HW - 0.06, 0.025, 0.16, 0.02, HAIR_Y + 0.005, { z: HD / 2 + 0.07, cTop: 0, cBot: 0 }), hcs, head);
    } else if (hk === "helmet" || hk === "miner") {
      mesh(block(HW + 0.06, HH + 0.09 - HAIR_Y, HD + 0.06, 0.07, HAIR_Y, { cBot: 0 }), hc, head);
      mesh(block(HW + 0.11, 0.03, HD + 0.11, 0.06, HAIR_Y, { cTop: 0, cBot: 0 }), hcs, head); back();
      if (hk === "miner") { const lamp = new THREE.Mesh(new THREE.BoxGeometry(0.1, 0.07, 0.03), LAMP_GLOW); lamp.position.set(0, HAIR_Y + 0.1, HD / 2 + 0.045); head.add(lamp); }
    } else if (hk === "turban") {
      mesh(block(HW + 0.07, 0.13, HD + 0.07, 0.06, HAIR_Y, {}), hcs, head);
      mesh(block(HW + 0.04, 0.15, HD + 0.04, 0.09, HAIR_Y + 0.11, {}), hc, head); back();
    } else if (hk === "hood") {
      mesh(block(HW + 0.08, HH + 0.07 - HAIR_Y, HD + 0.06, 0.06, HAIR_Y, { cBot: 0, z: -0.01 }), hc, head);
      for (const s of [-1, 1]) mesh(block(0.04, HAIR_Y + 0.02, HD + 0.04, 0.012, -0.02, { x: s * (HW / 2 + 0.035), z: -0.01 }), hc, head);
      mesh(block(HW + 0.08, HH + 0.08, 0.06, 0.02, -0.02, { z: -HD / 2 - 0.035 }), hcs, head);
    } else if (hk === "headset") {
      crop();
      mesh(block(HW + 0.1, 0.03, 0.05, 0.01, HH + 0.035, {}), hcs, head);
      for (const s of [-1, 1]) mesh(block(0.045, 0.14, 0.12, 0.015, 0.19, { x: s * (HW / 2 + 0.05) }), hc, head);
      mesh(block(0.02, 0.02, 0.16, 0, 0.16, { x: -(HW / 2 + 0.06), z: 0.1 }), hcs, head);
    }
  }

  // a beard is one shell on the face's front and front bevels (design 36a), its edge in stubble
  const shell = beardShell(beardOf(spec));
  if (shell) { mesh(shell.beard, hair, head); mesh(shell.stubble, mat(mixHex(spec.hairC, skin, BEARD_STUBBLE_MIX)), head, 0, 0, 0, false); }
  // the role's dress (version 10): an apron or a tabard on the torso's front, and one held prop in the right hand
  if (spec.apron) mesh(frontStrip(cutProf(B.prof, 0.06, 0.8, torsoH, w * 1.05)), mat(spec.apron), body);
  if (spec.tabard) mesh(frontStrip(cutProf(B.prof, 0.12, 0.97, torsoH, w * 1.075)), mat(spec.tabard), body);
  if (spec.prop && spec.prop !== "none") prop(spec.prop, arms[1], -armL - 0.05, mesh);
  const face = new THREE.Group(); face.position.z = HA; head.add(face);
  const eyes = [-1, 1].map(s => {
    const e = new THREE.Group(); e.position.set(s * spec.gap, EYE_Y, 0); face.add(e);
    const white = mesh(disc(WR), mat(0xf6f3ea, 1), e, 0, 0, 0.002, false);
    const pg = new THREE.Group(); pg.position.z = 0.003; e.add(pg);
    const pupil = mesh(disc(PR), mat(0x23203a, 2), pg, 0, 0, 0, false);
    const glint = mesh(disc(GR), mat(0xffffff, 3), pg, -0.017, 0.018, 0.001, false);
    const lg = new THREE.PlaneGeometry(2.19 * WR, 1); lg.translate(0, -0.5, 0);
    const lid = mesh(lg, mat(skin, 4), e, 0, 0.913 * WR, 0.005, false);
    const lash = mesh(new THREE.PlaneGeometry(2.04 * WR, 0.01), mat(0x3a2a26, 5), e, 0, 0.846 * WR, 0.006, false);
    const ag = cellQuad(2 * WR, WR); setCell(ag, "happy");
    const arc = mesh(ag, aMat(3), e, 0, 0.077 * WR, 0.003, false); arc.visible = false;
    const brow = mesh(new THREE.PlaneGeometry(0.1, 0.026), mat(browC, 2), face, s * (spec.gap + 0.01), BROW_Y, 0.003, false);
    const blushM = PBD.litMaterial(null, lin(skin), { polygonOffset: true, polygonOffsetFactor: -1, polygonOffsetUnits: -2 });
    const blush = mesh(disc(1), blushM, face, s * 0.155, 0.178, 0.0015, false); blush.scale.set(0.052, 0.029, 1);
    return { s, e, white, pg, pupil, glint, lid, lash, arc, brow, blush, blushM };
  });
  const mg = cellQuad(0.24, 0.12); setCell(mg, "smile");
  const mouth = mesh(mg, aMat(1), face, 0, MOUTH_Y, 0.002, false);
  const F = { spec, g, body, head, face, legs, arms, eyes, mouth, mg, legLen, torsoH, hs, ph: Math.random() * 6,
    cur: {}, tgt: null, exprName: "neutral", mouthCell: "smile", arcs: false,
    nextBlink: 1 + Math.random() * 3, blinkT: -1, dbl: false, gazeHold: 0, gazeW: new THREE.Vector3(), pupil: [0, 0], hy: 0, hp: 0, bp: 0,
    talkT: 0, talkCell: "A", act: null, walk: { on: false, th: Math.PI / 2, phase: 0, yaw: 0 }, home: null };
  for (const k of NUMK) F.cur[k] = 0; F.cur.eye = 1; F.cur.pup = 1;
  setExprOn(F, "neutral", true);
  return F;
}
function setExprOn(F, name, snap) {
  const E = EXPR[name]; F.exprName = name; F.tgt = {}; for (const k of NUMK) F.tgt[k] = E[k] || 0; F.tgt.eye = E.eye; F.tgt.pup = E.pup;
  if (snap) Object.assign(F.cur, F.tgt);
  F.arcs = !!E.arcs; F.mouthBase = E.mouth; F.look = E.look || null; F.talkE = !!E.talk;
  for (const e of F.eyes) { e.arc.visible = F.arcs; e.white.visible = e.pg.visible = e.lid.visible = e.lash.visible = !F.arcs; }
}

// the front face of a six-sided lathe only (the k = 1 side, facing +z), for an apron or a tabard
function frontStrip(profile) {
  const g = new PBD.Geo(), a0 = Math.PI / 3, a1 = 2 * Math.PI / 3, P = (y, r, a) => [r * Math.cos(a), y, r * Math.sin(a)];
  for (let i = 0; i + 1 < profile.length; i++) { const [y0, r0] = profile[i], [y1, r1] = profile[i + 1]; g.face([P(y0, r0, a0), P(y0, r0, a1), P(y1, r1, a1), P(y1, r1, a0)], [0, 0, 1]); }
  return g.build(1);
}
// a role's held prop on the arm's pivot, the hand at y = hy: fork, spear, hammer, pick, fish, pouch, pad, crate
const WOOD = 0x8a6a42, IRON = 0x9aa0a8;
function prop(P, arm, hy, mesh) {
  const pole = (y0, y1, r, c) => mesh(lathe([[y0, r], [y1, r]]), mat(c), arm);
  if (P === "fork") { pole(hy - 0.55, hy + 0.8, 0.02, WOOD); mesh(block(0.13, 0.02, 0.02, 0, hy + 0.79, {}), mat(IRON), arm); for (const k of [-1, 0, 1]) mesh(block(0.018, 0.16, 0.018, 0, hy + 0.8, { x: k * 0.055 }), mat(IRON), arm); }
  if (P === "spear") { pole(hy - 0.5, hy + 1.8, 0.024, 0x6b4a2e); mesh(lathe([[hy + 1.8, 0.04], [hy + 2.0, 0]]), mat(IRON), arm); }
  if (P === "hammer") { pole(hy - 0.3, hy + 0.04, 0.02, WOOD); mesh(block(0.17, 0.075, 0.075, 0.01, hy - 0.37, {}), mat(IRON), arm); }
  if (P === "pick") { pole(hy - 0.5, hy + 0.04, 0.02, WOOD); mesh(block(0.42, 0.045, 0.045, 0.012, hy - 0.55, {}), mat(IRON), arm); }
  if (P === "fish") { mesh(lathe([[hy - 0.38, 0], [hy - 0.3, 0.05], [hy - 0.13, 0.06], [hy - 0.05, 0.016]], { sx: 0.45 }), mat(0x5ab0d0), arm); mesh(block(0.03, 0.06, 0.09, 0.01, hy - 0.44, {}), mat(shade(0x5ab0d0, 0.78)), arm); }
  if (P === "pouch") mesh(block(0.13, 0.14, 0.1, 0.03, hy - 0.19, {}), mat(0xa07a4a), arm);
  if (P === "pad") mesh(block(0.15, 0.2, 0.025, 0.01, hy - 0.13, { z: 0.07 }), mat(0x506080), arm);
  if (P === "crate") mesh(block(0.32, 0.25, 0.26, 0.02, hy - 0.24, { x: 0.12 }), mat(WOOD), arm);
}
function genSpec(seed, i, P) {
  const rn = rng(seed * 9973 + i * 131 + 7), pick = a => a[Math.floor(rn() * a.length) % a.length];
  const build = pick(["round", "pear", "tall", "stout"]);
  const h = build === "tall" ? 1.6 + rn() * 0.1 : build === "stout" ? 1.45 + rn() * 0.08 : 1.48 + rn() * 0.16;   // version 9: the 0.1 m taller head on the same bodies
  const eye = 0.88 + rn() * 0.2, gap = clamp(0.093 + rn() * 0.014, WR * eye + 0.008, HW / 2 - HC - 0.008 - WR * eye * 1.15);
  const hairS = pick(["bun", "crop", "braid", "bald", "hat", "crop"]), shirt = pick(P.cloth);
  const S = { build, h, head: 0.9 + rn() * 0.12, eye, gap, nose: pick(["none", "button", "button", "long"]), hair: hairS,
    beard: hairS !== "bun" && hairS !== "braid" && rn() < 0.35, ears: rn() < 0.6, skin: pick(P.skin), hairC: pick(P.hair),
    shirt, pants: pick(P.cloth.filter(c => c !== shirt)), shoe: pick([0x4a3428, 0x3a2e26, 0x6a4a30]), hatC: pick(P.cloth),
    mood: pick(["happy", "neutral", "happy", "thinking", "sleepy", "delighted", "surprised", "talking"]),
    beardStyle: rn() };   // design 36: appended after every other roll, so nobody else's look moves
  S.beard = S.beard ? (S.beardStyle < 0.5 ? "goatee" : "full") : "none";
  return S;
}

// the face of a person off the Faces tab: their resting expression, eyes ahead, blinking every 2 to 6 s
function liveFace(F, dt) {
  let blink = 0;
  if (F.blinkT < 0) { F.nextBlink -= dt; if (F.nextBlink <= 0) F.blinkT = 0; }
  else { F.blinkT += dt; const t = F.blinkT; blink = t < 0.06 ? t / 0.06 : t < 0.09 ? 1 : Math.max(0, 1 - (t - 0.09) / 0.12); if (t > 0.21) { F.blinkT = -1; F.nextBlink = 2 + Math.random() * 4; } }
  if (F.talkE) { F.talkT -= dt; if (F.talkT <= 0) { F.talkT = 0.08 + Math.random() * 0.09; const c = ["A", "O", "M", "A", "smile", "O", "M"]; F.talkCell = c[Math.floor(Math.random() * c.length)]; } }
  const cell = F.talkE ? F.talkCell : F.mouthBase; if (cell !== F.mouthCell) { setCell(F.mg, cell); F.mouthCell = cell; }
  F.mouth.position.x = F.cur.mx * FSC;
  const look = F.look || [0, 0], lid = Math.min(1, Math.max(blink, F.cur.lid + Math.max(0, -look[1]) * 0.3));
  for (const e of F.eyes) {
    const es = F.cur.eye * F.spec.eye; e.e.scale.set(es, es, 1);
    e.pupil.scale.setScalar(F.cur.pup); e.pg.position.set(look[0] * (0.98 * WR - PR * F.cur.pup), look[1] * (0.846 * WR - 0.87 * PR * F.cur.pup), 0.003);
    const lh = 1.83 * WR * lid; e.lid.scale.y = Math.max(lh, 1e-4); e.lid.visible = !F.arcs && lh > 0.003;
    const ly = 0.913 * WR - lh + 0.0008; e.lash.position.y = ly; e.lash.scale.x = Math.max(0.05, (2 * (WR - Math.abs(ly) / 1.732) + 0.01) / (2.04 * WR));
    e.brow.position.y = BROW_Y + FSC * (F.cur.by + (e.s > 0 ? F.cur.asym : 0)) + (es - 1) * 0.065; e.brow.rotation.z = -e.s * F.cur.bt + e.s * 0.06;
    const b = F.cur.blush; e.blush.visible = b > 0.04; if (e.blush.visible) e.blushM.color.copy(lin(mixHex(F.spec.skin, 0xf27a7a, 0.55 * b)));
  }
}
return { MC, shade, mixHex, mat, AC, CELL, atlasTex, aMat, cellQuad, setCell, SIX, lathe, disc, block, HW, HH, HD, HC, HA, EYE_Y, MOUTH_Y, BROW_Y, HAIR_Y, WR, PR, GR, FSC, BUILDS, profAt, cutProf, EXPR, EXPR_KEYS, NUMK, friend, setExprOn, genSpec, liveFace, BEARDS, BEARD_SINK, BEARD_U, BEARD_Y, beardShell, beardOf, frontStrip, prop, earGeo, smoothGeos };
})();

window.FK_FIELDS = { name: "Fields folk", world: "Tenebris \u00b7 fields, village, walled town", skin: [0xe2ac80, 0xc99a74, 0x9c6b4a, 0xf0c49e, 0x7a5038], cloth: [0x5c9a46, 0x7a5a3a, 0xb0884a, 0x4a6e9a, 0xa04a3a, 0xd8c8a0], hair: [0x3b2a1e, 0x8a5a2a, 0xd8b060, 0x1e1a18, 0xa04a2a], hat: ["none", "none", "straw", "cap"], text: "Farmers, millers, smiths and guards in oak and limestone towns. Undyed wool, olive and madder red." };
