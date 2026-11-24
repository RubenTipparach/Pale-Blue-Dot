// The herd's forms for the Art style tab of the husbandry mockup (M26, `#art`).
//
// A port of the game's `crates/pbd-app/src/husbandry/loft.rs` (the smooth loft and its Catmull-Rom curve) and
// `husbandry/forms.rs` (the horse, the plodder, the woolback and the meadow hen), with the art-style knobs of
// `animal-husbandry` design 17 added. With the TODAY style every number is the game's and the mesh is the game's to
// float precision: `docs/mockups/husbandry/art/today.json` is the game's own mesh, written by the instrument
// `dump_the_herds_forms_for_the_art_mockup` (pbd-app husbandry tests), and `check(today)` compares the two.
//
// Model space is the game's: +X right, +Y up, -Z forward, the feet on the ground at the origin. Colours are sRGB
// (the game converts them to linear vertex colours; the page draws in display space).
(function (root) {
"use strict";
const V = {
  add: (a, b) => [a[0] + b[0], a[1] + b[1], a[2] + b[2]], sub: (a, b) => [a[0] - b[0], a[1] - b[1], a[2] - b[2]],
  mul: (a, s) => [a[0] * s, a[1] * s, a[2] * s], dot: (a, b) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2],
  cross: (a, b) => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]],
  len: a => Math.hypot(a[0], a[1], a[2]), lerp: (a, b, t) => [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t],
  dist: (a, b) => Math.hypot(a[0] - b[0], a[1] - b[1], a[2] - b[2]),
  norm: (a, fb) => { const l = Math.hypot(a[0], a[1], a[2]); return l > 1e-20 && isFinite(1 / l) ? [a[0] / l, a[1] / l, a[2] / l] : fb; },
};
const X = [1, 0, 0], Y = [0, 1, 0], Z = [0, 0, 1];
// Quaternions as [x, y, z, w], glam's conventions.
const Q = {
  x: a => [Math.sin(a / 2), 0, 0, Math.cos(a / 2)], y: a => [0, Math.sin(a / 2), 0, Math.cos(a / 2)], z: a => [0, 0, Math.sin(a / 2), Math.cos(a / 2)],
  axis: (n, a) => { const s = Math.sin(a / 2); return [n[0] * s, n[1] * s, n[2] * s, Math.cos(a / 2)]; },
  mul: (a, b) => [
    a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1], a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
    a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3], a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2]],
  rot: (q, v) => { const u = [q[0], q[1], q[2]], s = q[3]; const t = V.mul(V.cross(u, v), 2); return V.add(V.add(v, V.mul(t, s)), V.cross(u, t)); },
};
const TILE = { hair: 0, mane: 1, hoof: 2, feather: 3, fleece: 4, skin: 5 }, TILES = 6;

// A style: how a form is lofted. TODAY is the game's.
//  sides: points round each part; per: Catmull-Rom rings between authored rings (1 = straight between them);
//  sq: the section's squareness (2 a circle or ellipse, higher a rounded square: a superellipse's exponent);
//  facet: how much each face's own normal is blended into the smooth one (0 smooth, 1 flat); creases: hoof, horn and
//  the jaw drawn hard; neck: "game" (the game's carriage and graze) or "fixed" (design 17's carriage and graze).
const STYLES = {
  today:  { sides: { body: 16, neck: 14, head: 12, ear: 6, horn: 6, tail: 8, leg: 10, hoof: 10, bird: 14, wing: 8, small: 6 }, per: { body: 3, neck: 3, head: 2, leg: 2, bird: 3 }, sq: { body: 2, neck: 2, head: 2, leg: 2 }, facet: 0, creases: false, neck: "game", texM: 0 },
  neck:   { sides: { body: 16, neck: 14, head: 12, ear: 6, horn: 6, tail: 8, leg: 10, hoof: 10, bird: 14, wing: 8, small: 6 }, per: { body: 3, neck: 3, head: 2, leg: 2, bird: 3 }, sq: { body: 2, neck: 2, head: 2, leg: 2 }, facet: 0, creases: false, neck: "fixed", texM: 0 },
  soft:   { sides: { body: 9, neck: 8, head: 8, ear: 4, horn: 5, tail: 5, leg: 6, hoof: 6, bird: 8, wing: 5, small: 4 }, per: { body: 1, neck: 1, head: 1, leg: 1, bird: 1 }, sq: { body: 2.2, neck: 2.2, head: 2.6, leg: 2.4 }, facet: 0.4, creases: true, neck: "fixed", texM: 2, blend: true },
  carved: { sides: { body: 16, neck: 12, head: 12, ear: 4, horn: 5, tail: 6, leg: 8, hoof: 8, bird: 12, wing: 6, small: 4 }, per: { body: 1, neck: 1, head: 1, leg: 1, bird: 1 }, sq: { body: 4.2, neck: 3.4, head: 4.5, leg: 3.6 }, facet: 0.2, creases: true, neck: "fixed", texM: 2, blend: true },
};

function ring(centre, u, v, ru, rv){ return { centre, u, v, ru, rv }; }
const at = (r, x, y) => V.add(V.add(r.centre, V.mul(r.u, x * r.ru)), V.mul(r.v, y * r.rv));
// n sides round from u toward v; sq > 2 squares the section into a superellipse.
function profile(n, sq = 2){
  const out = [];
  for (let k = 0; k < n; k++){
    const a = 2 * Math.PI * k / n, c = Math.cos(a), s = Math.sin(a);
    if (sq === 2) out.push([c, s]);
    else out.push([Math.sign(c) * Math.pow(Math.abs(c), 2 / sq), Math.sign(s) * Math.pow(Math.abs(s), 2 / sq)]);
  }
  return out;
}
function curve(rings, per){
  if (rings.length < 3 || per < 2) return rings.slice();
  const n = rings.length, get = i => rings[Math.max(0, Math.min(n - 1, i))];
  const cr = (a, b, c, d, t) => { const t2 = t * t, t3 = t2 * t; return 0.5 * ((2 * b) + (-a + c) * t + (2 * a - 5 * b + 4 * c - d) * t2 + (-a + 3 * b - 3 * c + d) * t3); };
  const crv = (a, b, c, d, t) => [0, 1, 2].map(j => cr(a[j], b[j], c[j], d[j], t));
  const out = [];
  for (let i = 0; i < n - 1; i++){
    const a = get(i - 1), b = get(i), c = get(i + 1), d = get(i + 2);
    for (let s = 0; s < per; s++){
      const t = s / per;
      out.push(ring(crv(a.centre, b.centre, c.centre, d.centre, t), V.norm(V.lerp(b.u, c.u, t), b.u), V.norm(V.lerp(b.v, c.v, t), b.v),
        Math.max(cr(a.ru, b.ru, c.ru, d.ru, t), Math.min(b.ru, c.ru) * 0.5), Math.max(cr(a.rv, b.rv, c.rv, d.rv, t), Math.min(b.rv, c.rv) * 0.5)));
    }
  }
  out.push(rings[n - 1]);
  return out;
}
// A point's texture coordinate within its tile of the strip (the tile is its own attribute; the page's shader takes the
// fraction and picks the texel nearest). texM 0 is the game's: the tile once round the section, inset half a texel each
// side, and repeated along at square texels. texM > 0 lays it by metres, a tile every texM metres each way.
function uvOf(tile, around, along, perimeter, texM = 0){
  if (texM > 0) return [around * perimeter / texM, along / texM];
  return [(0.5 + Math.min(1, Math.max(0, around)) * 31) / 32, along / Math.max(perimeter, 1e-3)];
}
// The game's own UV for a point (loft.rs `uv_of`), for checking against today.json.
const gameUV = (tile, lu) => (tile * 32 + lu * 32) / (32 * TILES);
let TEXM = 0;   // the style being built's texture scale (uvOf's texM)
function builder(){ return { position: [], normal: [], colour: [], uv: [], tile: [], index: [] }; }
function pushV(out, p, n, c, uv, tile){ out.position.push(p[0], p[1], p[2]); out.normal.push(n[0], n[1], n[2]); out.colour.push(c[0], c[1], c[2]); out.uv.push(uv[0], uv[1]); out.tile.push(tile); return out.position.length / 3 - 1; }
function pushTri(out, p, n, c, uv, tile){
  const face = V.cross(V.sub(p[1], p[0]), V.sub(p[2], p[0]));
  const o = V.dot(face, n) >= 0 ? [0, 1, 2] : [0, 2, 1];
  const base = out.position.length / 3;
  for (const i of o) pushV(out, p[i], n, c, uv[i], tile);
  out.index.push(base, base + 1, base + 2);
}
// The game's smooth loft (loft.rs `smooth`). `o.sq` squares the section; `o.facet` blends each face's own normal in
// (its points then not shared between faces), so planes read on a curved form without it going flat-shaded.
function smooth(out, rings, sides, tile, colour, o = {}){
  if (rings.length < 2 || sides < 3) return;
  const sq = o.sq ?? 2, facet = o.facet ?? 0, tm = o.texM ?? TEXM;
  // circ: a few-sided section's faces set out to the true ellipse (its points on a circumscribed one), so parts grown
  // from inside it do not show through its flats
  if (o.circ){ const k = 1 / Math.cos(Math.PI / sides); rings = rings.map(r => ring(r.centre, r.u, r.v, r.ru * k, r.rv * k)); }
  const prof = profile(sides, sq), rows = rings.length, cols = sides + 1;
  const pts = rings.map(r => { const row = []; for (let k = 0; k < cols; k++){ const [x, y] = prof[k % sides]; row.push(at(r, x, y)); } return row; });
  const normals = Array.from({ length: rows }, () => Array.from({ length: sides }, () => [0, 0, 0]));
  let sign = 0;
  const faceN = [];
  for (let i = 0; i < rows - 1; i++){
    faceN.push([]);
    for (let k = 0; k < sides; k++){
      const a = pts[i][k], b = pts[i][k + 1], c = pts[i + 1][k + 1], d = pts[i + 1][k];
      const n = V.add(V.cross(V.sub(b, a), V.sub(d, a)), V.cross(V.sub(c, b), V.sub(d, b)));
      faceN[i].push(n);
      if (sign === 0 && V.dot(n, n) > 1e-12){
        const mid = V.mul(V.add(rings[i].centre, rings[i + 1].centre), 0.5);
        sign = V.dot(n, V.sub(V.mul(V.add(V.add(a, b), V.add(c, d)), 0.25), mid)) >= 0 ? 1 : -1;
      }
      for (const [ii, kk] of [[i, k], [i, k + 1], [i + 1, k + 1], [i + 1, k]]){ const t = normals[ii][kk % sides]; t[0] += n[0]; t[1] += n[1]; t[2] += n[2]; }
    }
  }
  if (sign === 0) sign = 1;
  const along = [0];
  for (let i = 1; i < rows; i++) along.push(along[i - 1] + V.dist(rings[i].centre, rings[i - 1].centre));
  const total = Math.max(along[rows - 1], 1e-4);
  const perimeter = rings.reduce((s, r) => { const a = Math.abs(r.ru), b = Math.abs(r.rv); return s + Math.PI * (3 * (a + b) - Math.sqrt((3 * a + b) * (a + 3 * b))); }, 0) / rows;
  const nAt = (i, k) => V.norm(V.mul(normals[i][k % sides], sign), V.norm(V.sub(pts[i][k], rings[i].centre), Y));
  if (facet <= 0){
    const base = out.position.length / 3;
    for (let i = 0; i < rows; i++) for (let k = 0; k < cols; k++){
      const n = nAt(i, k), share = k / sides;
      pushV(out, pts[i][k], n, colour(n, along[i] / total, share), uvOf(tile, share, along[i], perimeter, tm), tile);
    }
    for (let i = 0; i < rows - 1; i++) for (let k = 0; k < sides; k++){
      const a = base + i * cols + k, b = a + 1, d = base + (i + 1) * cols + k, c = d + 1;
      if (sign > 0) out.index.push(a, b, c, a, c, d); else out.index.push(a, d, c, a, c, b);
    }
  } else {
    for (let i = 0; i < rows - 1; i++) for (let k = 0; k < sides; k++){
      const fn = V.norm(V.mul(faceN[i][k], sign), Y), idx = [];
      for (const [ii, kk] of [[i, k], [i, k + 1], [i + 1, k + 1], [i + 1, k]]){
        const sn = nAt(ii, kk), n = V.norm(V.lerp(sn, fn, facet), sn), share = kk / sides;
        idx.push(pushV(out, pts[ii][kk], n, colour(sn, along[ii] / total, share), uvOf(tile, share, along[ii], perimeter, tm), tile));
      }
      const [a, b, c, d] = idx;
      if (sign > 0) out.index.push(a, b, c, a, c, d); else out.index.push(a, d, c, a, c, b);
    }
  }
  for (const [i, toward] of [[0, 1], [rows - 1, rows - 2]]){
    const r = rings[i], n = V.norm(V.sub(r.centre, rings[toward].centre), Y);
    if (Math.abs(r.ru) * Math.abs(r.rv) < 1e-8) continue;
    const c = colour(n, along[i] / total, 0.75);
    for (let k = 0; k < sides; k++){
      pushTri(out, [r.centre, pts[i][k], pts[i][(k + 1) % sides]], n, c,
        [uvOf(tile, 0.5, along[i], perimeter, tm), uvOf(tile, k / sides, along[i] + 0.01, perimeter, tm), uvOf(tile, (k + 1) / sides, along[i] + 0.01, perimeter, tm)], tile);
    }
  }
}

const BEASTS = {
  horse: { withers: 1.55, body: [[0.95, 1.20, 0.17, 0.20], [0.82, 1.22, 0.27, 0.29], [0.55, 1.20, 0.31, 0.34], [0.15, 1.17, 0.32, 0.37], [-0.30, 1.19, 0.30, 0.36], [-0.62, 1.24, 0.25, 0.33], [-0.80, 1.30, 0.18, 0.26]],
    bodyTile: TILE.hair, neckRoot: [1.32, -0.62], neckDir: [0.62, -0.5], neck: [[0.0, 0.2, 0.27], [0.35, 0.16, 0.22], [0.7, 0.12, 0.17], [0.86, 0.1, 0.15]], poll: 0.8, neckTile: TILE.hair,
    headDir: [-0.62, -0.78], head: [[-0.04, 0.10, 0.13], [0.18, 0.11, 0.12], [0.38, 0.08, 0.09], [0.5, 0.075, 0.07], [0.54, 0.055, 0.05]], muzzle: 0.38, ear: 0.13, earsOut: false, horn: 0,
    tailRoot: [1.36, 0.92], tailDir: [-0.85, 0.32], tailLen: 0.8, tail: [[0.0, 0.06], [0.3, 0.075], [0.65, 0.065], [1.0, 0.025]], switch: 0,
    legX: 0.17, foreZ: -0.55, hindZ: 0.62, legTop: 1.12, foreJoint: 0.5, hindJoint: 0.58, legW: 1.0, mane: true },
  plodder: { withers: 1.35, body: [[0.86, 0.98, 0.17, 0.20], [0.74, 1.00, 0.31, 0.32], [0.45, 0.97, 0.37, 0.38], [0.05, 0.93, 0.39, 0.43], [-0.35, 0.95, 0.37, 0.41], [-0.62, 1.02, 0.31, 0.35], [-0.77, 1.07, 0.21, 0.27]],
    bodyTile: TILE.hair, neckRoot: [1.05, -0.62], neckDir: [0.3, -0.9], neck: [[0.0, 0.22, 0.28], [0.3, 0.19, 0.25], [0.55, 0.16, 0.21]], poll: 0.5, neckTile: TILE.hair,
    headDir: [-0.78, -0.62], head: [[-0.04, 0.13, 0.15], [0.18, 0.135, 0.13], [0.36, 0.115, 0.1], [0.47, 0.105, 0.08], [0.51, 0.08, 0.055]], muzzle: 0.36, ear: 0.1, earsOut: true, horn: 0.17,
    tailRoot: [1.07, 0.85], tailDir: [-0.95, 0.18], tailLen: 0.82, tail: [[0.0, 0.045], [0.4, 0.03], [0.75, 0.028], [0.85, 0.055], [1.0, 0.02]], switch: 0.78,
    legX: 0.21, foreZ: -0.5, hindZ: 0.6, legTop: 0.82, foreJoint: 0.42, hindJoint: 0.47, legW: 1.2, mane: false },
  woolback: { withers: 0.8, body: [[0.5, 0.57, 0.13, 0.15], [0.42, 0.59, 0.25, 0.25], [0.2, 0.58, 0.3, 0.28], [-0.05, 0.57, 0.31, 0.29], [-0.28, 0.59, 0.28, 0.27], [-0.42, 0.63, 0.21, 0.22], [-0.5, 0.66, 0.12, 0.15]],
    bodyTile: TILE.fleece, neckRoot: [0.68, -0.42], neckDir: [0.6, -0.8], neck: [[0.0, 0.13, 0.15], [0.16, 0.1, 0.12], [0.26, 0.085, 0.095]], poll: 0.24, neckTile: TILE.fleece,
    headDir: [-0.55, -0.83], head: [[-0.03, 0.075, 0.085], [0.1, 0.075, 0.075], [0.2, 0.057, 0.055], [0.27, 0.045, 0.04], [0.29, 0.03, 0.028]], muzzle: 0.22, ear: 0.08, earsOut: true, horn: 0,
    tailRoot: [0.66, 0.48], tailDir: [-0.7, 0.7], tailLen: 0.14, tail: [[0.0, 0.06], [0.5, 0.05], [1.0, 0.025]], switch: 0,
    legX: 0.13, foreZ: -0.3, hindZ: 0.33, legTop: 0.42, foreJoint: 0.22, hindJoint: 0.25, legW: 0.42, mane: false },
};
// Design 17's carriage: the neck's and head's angles above horizontal, degrees, standing (the game's in brackets), and
// the graze: how the head reaches the grass. Standing, a plodder's poll is about level with its withers and its face
// about 50 degrees below level; grazing, the neck bends down along its length (an arch, not a hinge at the chest)
// until the muzzle is at the grass, and the face is never turned past vertical.
const CARRIAGE = {
  // neck and head: degrees above level standing (the game's: horse 51.1 and -38.5, plodder 18.4 and -51.5, woolback
  // 36.9 and -33.5); reach: the neck's length against the game's (the plodder's and woolback's necks are too short to
  // reach the grass without folding, which is how today's graze ends with the face turned back at the chest).
  horse:    { neck: 51, head: -40, reach: 1.0, graze: { face: -80, reachY: 0.05 } },
  plodder:  { neck: 30, head: -42, reach: 1.2, graze: { face: -76, reachY: 0.05 } },
  woolback: { neck: 42, head: -38, reach: 1.5, graze: { face: -78, reachY: 0.04 } },
};
const dirOf = deg => [Math.sin(deg * Math.PI / 180), -Math.cos(deg * Math.PI / 180)];   // (up, z): -z is forward
const mix = (a, b, t) => [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t];
const dark = (c, k) => c.map(x => x * k);

// The neck and head, the game's way: one pitch turns both rigidly about the neck's root.
function neckGame(f, s, pose, neckRoot, down, walk){
  const nod = Math.sin(pose.phase * 2 + 0.6) * 0.06 * walk;
  const pitch = pose.head_down * (1 - 0.6 * down) + nod;
  const turn = Q.mul(Q.y(pose.glance), Q.x(-pitch));
  const neckDir = Q.rot(turn, V.norm([0, f.neckDir[0], f.neckDir[1]], Y));
  const side = V.norm(Q.rot(turn, X), X);
  const crest = V.norm(V.cross(neckDir, side), Y);
  const centres = f.neck.map(([t]) => V.add(neckRoot, V.mul(neckDir, t * s)));
  const dirs = f.neck.map(() => neckDir);
  const poll = V.add(neckRoot, V.mul(neckDir, f.poll * s));
  const headDir = Q.rot(turn, V.norm([0, f.headDir[0], f.headDir[1]], Y));
  return { centres, dirs, side, crest: () => crest, poll, headDir };
}
// Design 17: the neck arches down along its length by `bend` (radians at the poll), and the head turns on to `face`.
function neckFixedAt(f, kind, s, neckRoot, bend, headExtra, glance){
  const C = CARRIAGE[kind], yaw = Q.y(glance);
  const d0 = V.norm([0, ...dirOf(C.neck)], Y);
  // Most of the bend is the neck lowered from the shoulder; the rest an arch along it, all of it by the poll.
  const dirAt = t => Q.rot(Q.mul(yaw, Q.x(-bend * (0.65 + 0.35 * Math.min(1, t / (f.poll * C.reach))))), d0);
  // Integrate the bent neck's centre line in small steps.
  const centreAt = t => { let p = neckRoot.slice(); const n = 24; for (let i = 0; i < n; i++){ const a = t * i / n, b = t * (i + 1) / n; p = V.add(p, V.mul(dirAt((a + b) / 2), (b - a) * s)); } return p; };
  const side = V.norm(Q.rot(yaw, X), X);
  const centres = f.neck.map(([t]) => centreAt(t * C.reach));
  const dirs = f.neck.map(([t]) => dirAt(t * C.reach));
  const poll = centreAt(f.poll * C.reach);
  // The face's angle is its own (degrees above level), never the neck's bend added to it: so a grazing face is never
  // turned past vertical toward the chest, the game's fault.
  const headDir = Q.rot(yaw, V.norm([0, ...dirOf(headExtra)], Y));
  return { centres, dirs, side, crest: i => V.norm(V.cross(dirs[i], side), Y), poll, headDir };
}
function neckFixed(f, kind, s, pose, neckRoot, down, walk, groundY){
  const C = CARRIAGE[kind];
  const nod = Math.sin(pose.phase * 2 + 0.6) * 0.06 * walk;
  const g = Math.max(0, Math.min(1, pose.head_down / 1.85)) * (1 - 0.6 * down);
  const headLen = f.head[f.head.length - 1][0] + 0.04;
  // The full graze: the face at its grazing angle, and the neck bent until the muzzle is at the grass.
  const muzzleY = bend => { const k = neckFixedAt(f, kind, s, neckRoot, bend, C.graze.face, 0); return k.poll[1] + k.headDir[1] * headLen * s; };
  let lo = 0, hi = 2.6;
  for (let i = 0; i < 40; i++){ const m = (lo + hi) / 2; if (muzzleY(m) > groundY + C.graze.reachY * s) lo = m; else hi = m; }
  const bend = g * lo + nod;
  const face = C.head + (C.graze.face - C.head) * g - nod * 180 / Math.PI;
  return neckFixedAt(f, kind, s, neckRoot, bend, face, pose.glance);
}

function beast(out, kind, c, pose, size, st){
  const f = BEASTS[kind], s = size, S = st.sides, P = st.per, SQ = st.sq, facet = st.facet;
  const walk = Math.max(0, Math.min(1, pose.walk)), down = Math.max(0, Math.min(1, pose.down));
  const k = f.legTop / 1.12;
  const bob = Math.cos(pose.phase * 2) * 0.025 * k * walk, drop = f.legTop * 0.72 * down;
  const lift = [0, (bob - drop) * s, 0];
  const rollAt = V.add([0, f.legTop * s, 0], lift), roll = Q.z(pose.shift);
  const rolled = p => V.add(rollAt, Q.rot(roll, V.sub(p, rollAt)));
  const v = (x, y, z) => [x * s, y * s, z * s];
  const blanket = [0.42, 0.16, 0.12];
  const body = f.body.map(([z, y, w, h]) => ring(rolled(V.add(v(0, y, z), lift)), Q.rot(roll, X), Q.rot(roll, Y), w * s, h * s));
  const pattern = c.pattern;
  smooth(out, curve(body, P.body), S.body, f.bodyTile, (n, t) => {
    let col = c.body;
    if (kind !== "woolback" && n[1] < -0.6) col = mix(col, c.pale, 0.6);
    if (pattern > 0 && t >= 0.38 && t < 0.62 && kind !== "horse") col = c.pale;
    if (pose.saddled && n[1] > 0.35 && t >= 0.42 && t < 0.72) col = blanket;
    return col;
  }, { sq: SQ.body, facet, circ: st.blend });
  const bodyEnd = out.position.length / 3;
  const neckRoot = rolled(V.add(v(0, f.neckRoot[0], f.neckRoot[1]), lift));
  const N = st.neck === "fixed" ? neckFixed(f, kind, s, pose, neckRoot, down, walk, 0) : neckGame(f, s, pose, neckRoot, down, walk);
  const side = N.side;
  const neck = f.neck.map(([t, w, h], i) => { const cr = N.crest(i); return ring(N.centres[i], side, V.mul(cr, -1), w * s, h * s); });
  // A blended style flares the neck's root into the chest, so it grows from the body rather than meeting it at a seam.
  if (st.blend) neck.unshift(ring(V.sub(N.centres[0], V.mul(N.dirs[0], 0.1 * s)), side, neck[0].v, neck[0].ru * 1.35, neck[0].rv * 1.3));
  const crest0 = N.crest(0);
  smooth(out, curve(neck, P.neck), S.neck, f.neckTile, (n, t) => {
    if (f.mane && V.dot(n, V.mul(crest0, -1)) > 0.55 && !(st.blend && t < 0.12)) return c.legs;
    if (kind === "woolback") return c.body;
    // blended: the body's colour at the root, the head's at the poll, no seam at either end
    if (st.blend) return t < 0.5 ? mix(c.body, mix(c.body, c.head, 0.5), t * 2) : mix(mix(c.body, c.head, 0.5), c.head, (t - 0.5) * 2);
    return mix(c.body, c.head, 0.5);
  }, { sq: SQ.neck, facet });
  const poll = N.poll, headDir = N.headDir, headUp = V.norm(V.cross(headDir, side), Y);
  out.carriage = { root: neckRoot, poll, neckDir: N.dirs[0], neckEnd: N.dirs[N.dirs.length - 1], headDir, tip: V.add(poll, V.mul(headDir, f.head[f.head.length - 1][0] * s)), bodyEnd };
  const head = f.head.map(([t, w, h]) => ring(V.add(poll, V.mul(headDir, t * s)), side, V.mul(headUp, -1), w * s, h * s));
  const headLen = f.head[f.head.length - 1][0] + 0.04, muzzleT = (f.muzzle + 0.04) / headLen;
  const faceTile = kind === "woolback" ? TILE.skin : TILE.hair;
  // A hard jaw: the head's section squared below (a flat, creased underside), as the style asks.
  smooth(out, curve(head, P.head), S.head, faceTile, (n, t) => {
    const eye = Math.abs(V.dot(n, side)) > 0.8 && V.dot(n, headUp) < 0.1 && t >= 0.22 && t < 0.4;
    if (eye) return [0.05, 0.04, 0.04];
    if (t > muzzleT) return kind === "horse" ? mix(c.head, c.legs, 0.7) : kind === "plodder" ? c.pale : dark(c.head, 0.8);
    return c.head;
  }, { sq: SQ.head, facet: st.creases ? Math.max(facet, 0.45) : facet });
  for (const sgn of [-1, 1]){
    const base = V.sub(V.add(poll, V.mul(side, sgn * 0.06 * s * Math.max(k, 0.6))), V.mul(headUp, 0.04 * s));
    const reach = f.earsOut ? V.norm(V.add(V.sub(V.mul(side, sgn * 0.9), V.mul(headUp, 0.3)), V.mul(headDir, 0.1)), Y)
                            : V.norm(V.sub(V.add(V.mul(headUp, -0.95), V.mul(side, sgn * 0.2)), V.mul(headDir, 0.15)), Y);
    const along = V.norm(V.cross(reach, headDir), side);
    const er = [ring(base, along, headDir, f.ear * 0.24 * s, f.ear * 0.14 * s), ring(V.add(base, V.mul(reach, f.ear * 0.55 * s)), along, headDir, f.ear * 0.22 * s, f.ear * 0.09 * s),
                ring(V.add(base, V.mul(reach, f.ear * s)), along, headDir, f.ear * 0.04 * s, f.ear * 0.03 * s)];
    smooth(out, er, S.ear, faceTile, () => c.head, { facet });
    if (f.horn > 0){
      const root = V.sub(V.add(poll, V.mul(side, sgn * 0.07 * s)), V.mul(headUp, 0.07 * s));
      const pts = [V.sub(root, V.mul(side, sgn * 0.03 * s)), V.sub(V.add(root, V.mul(side, sgn * f.horn * 0.55 * s)), V.mul(headUp, 0.02 * s)),
        V.sub(V.sub(V.add(root, V.mul(side, sgn * f.horn * 0.85 * s)), V.mul(headUp, 0.09 * s)), V.mul(headDir, 0.03 * s)),
        V.sub(V.sub(V.add(root, V.mul(side, sgn * f.horn * 0.95 * s)), V.mul(headUp, 0.17 * s)), V.mul(headDir, 0.05 * s))];
      const radii = [0.03, 0.026, 0.016, 0.004];
      smooth(out, pts.map((p, i) => ring(p, headDir, headUp, radii[i] * s, radii[i] * s)), S.horn, TILE.hoof, (n, t) => mix([0.86, 0.8, 0.66], [0.3, 0.26, 0.22], t * t), { facet: st.creases ? 0.5 : facet });
    }
  }
  const tailRoot = rolled(V.add(v(0, f.tailRoot[0], f.tailRoot[1]), lift));
  const swish = Q.y(pose.tail + Math.sin(pose.phase) * 0.08 * walk);
  const tailDir = Q.rot(swish, V.norm([0, f.tailDir[0], f.tailDir[1]], Y));
  const tail = f.tail.map(([t, r]) => ring(V.add(V.add(tailRoot, V.mul(tailDir, t * f.tailLen * s)), V.mul(Z, 0.08 * t * t * f.tailLen * s)), X, Z, r * s, r * s));
  smooth(out, tail, S.tail, kind === "woolback" ? TILE.fleece : TILE.mane, (n, t) => kind === "horse" ? c.legs : kind === "woolback" ? c.body : t >= f.switch ? dark(c.legs, 0.8) : c.body, { facet });
  for (const [x, z, offset, fore] of [[-1, f.hindZ, 0, false], [-1, f.foreZ, 0.25, true], [1, f.hindZ, 0.5, false], [1, f.foreZ, 0.75, true]]){
    const p = pose.phase + offset * 2 * Math.PI;
    const swing = Math.sin(p) * 0.32 * walk + (fore ? 1.2 : -1.1) * down;
    const liftLeg = Math.max(0, Math.cos(p)) * 0.5 * walk;
    const top = rolled(V.add(v(x * f.legX, f.legTop, z), lift));
    const swingQ = Q.x(swing), jointY = fore ? f.foreJoint : f.hindJoint;
    const bend = fore ? -liftLeg - 2.4 * down : liftLeg * 0.8 + 2.2 * down;
    const joint = V.add(top, Q.rot(swingQ, [0, (jointY - f.legTop) * s, fore ? 0 : 0.08 * k * s]));
    const below = Q.mul(swingQ, Q.x(bend));
    const fetlock = V.add(joint, Q.rot(below, [0, -(jointY - 0.12 * k) * s, 0]));
    const hoof = V.add(fetlock, Q.rot(below, [0, -0.1 * k * s, -0.03 * k * s]));
    const w = f.legW * Math.max(k, 0.55);
    const rg = (centre, rw, rh) => ring(centre, X, Z, rw * w * s, rh * w * s);
    const leg = [rg(V.add(top, [0, 0.14 * k * s, 0]), 0.13, 0.2), rg(V.lerp(top, joint, 0.3), 0.1, fore ? 0.13 : 0.17), rg(V.lerp(top, joint, 0.7), 0.07, fore ? 0.09 : 0.11),
                 rg(joint, 0.058, 0.07), rg(V.lerp(joint, fetlock, 0.5), 0.042, 0.05), rg(fetlock, 0.05, 0.055)];
    const upper = kind === "woolback" ? c.legs : c.body;
    smooth(out, curve(leg, P.leg), S.leg, kind === "woolback" ? TILE.skin : TILE.hair,
      (n, t) => kind === "woolback" ? upper : mix(upper, c.legs, Math.max(0, Math.min(1, (t - 0.45) * 3))), { sq: SQ.leg, facet });
    smooth(out, [rg(fetlock, 0.05, 0.055), rg(hoof, 0.065, 0.075)], S.hoof, TILE.hoof, () => [0.2, 0.17, 0.15], { sq: st.creases ? 3 : 2, facet: st.creases ? 1 : facet });
  }
}

function bird(out, c, male, pose, size, st){
  const s = size, S = st.sides, P = st.per, facet = st.facet;
  const v = (x, y, z) => [x * s, y * s, z * s];
  const down = Math.max(0, Math.min(1, pose.down));
  const sink = [0, -0.1 * down * s, 0];
  const rollAt = V.add(v(0, 0.2, 0), sink), roll = Q.z(pose.shift);
  const rolled = p => V.add(rollAt, Q.rot(roll, V.sub(p, rollAt)));
  const body = [[0.17, 0.23, 0.035, 0.045], [0.13, 0.21, 0.085, 0.085], [0.06, 0.2, 0.115, 0.105], [-0.03, 0.2, 0.12, 0.11], [-0.1, 0.22, 0.1, 0.1], [-0.14, 0.26, 0.06, 0.07]]
    .map(([z, y, w, h]) => ring(rolled(V.add(v(0, y, z), sink)), Q.rot(roll, X), Q.rot(roll, Y), w * s, h * s));
  const speckle = c.pattern > 0;
  smooth(out, curve(body, P.bird), S.bird, TILE.feather, (n, t, share) => {
    const fleck = speckle && (Math.floor(t * 37) + Math.round(share * 14)) % 3 === 0;
    return fleck && Math.abs(n[1]) < 0.7 ? dark(c.body, 0.62) : c.body;
  }, { sq: st.sq.body, facet });
  for (const sgn of [-1, 1]){
    const rings = [[-0.08, 0.235, 0.012, 0.05], [0, 0.235, 0.02, 0.062], [0.09, 0.24, 0.016, 0.05], [0.15, 0.25, 0.006, 0.02]]
      .map(([z, y, w, h]) => ring(rolled(V.add(v(sgn * 0.095, y, z), sink)), Q.rot(roll, X), Q.rot(roll, V.norm([0, 1, 0.25], Y)), w * s, h * s));
    smooth(out, curve(rings, Math.min(2, P.bird === 1 ? 1 : 2)), S.wing, TILE.feather, () => dark(c.body, 0.88), { facet });
  }
  const [tailColour, tailSize] = male ? [[0.12, 0.25, 0.2], 1.4] : [c.body, 1.0];
  const lift = Q.x(-(0.6 + pose.tail));
  const tailRoot = rolled(V.add(v(0, 0.24, 0.12), sink));
  const tail = [[0, 0.05, 0.03], [0.06, 0.055, 0.02], [0.13, 0.03, 0.008]].map(([t, w, h]) => ring(V.add(tailRoot, Q.rot(lift, [0, 0, t * tailSize * s])), X, Q.rot(lift, Y), w * s, h * s));
  smooth(out, tail, S.tail, TILE.feather, () => tailColour, { facet });
  const pivot = rolled(V.add(v(0, 0.27, -0.1 + pose.bob), sink));
  const nq = Q.mul(Q.y(pose.glance), Q.x(-pose.head_down * (1 - down)));
  const atp = p => V.add(pivot, Q.rot(nq, V.mul(p, s)));
  const fwd = Q.rot(nq, [0, 0, -1]), side = Q.rot(nq, X), up = Q.rot(nq, Y);
  const neck = [[[0, -0.03, 0.02], 0.06, 0.065], [[0, 0.05, -0.025], 0.045, 0.045], [[0, 0.11, -0.045], 0.045, 0.048], [[0, 0.135, -0.075], 0.038, 0.036], [[0, 0.13, -0.1], 0.02, 0.018]]
    .map(([p, w, h]) => ring(atp(p), side, fwd, w * s, h * s));
  smooth(out, curve(neck, Math.min(2, P.bird === 1 ? 1 : 2)), S.bird - 2, TILE.feather, (n, t) => {
    if (Math.abs(V.dot(n, side)) > 0.85 && t >= 0.6 && t < 0.8) return [0.04, 0.035, 0.03];
    return t < 0.35 ? mix(c.body, c.head, t / 0.35) : c.head;
  }, { facet });
  const beak = [[[0, 0.125, -0.09], 0.018, 0.015], [[0, 0.118, -0.115], 0.01, 0.008], [[0, 0.112, -0.13], 0.002, 0.002]].map(([p, w, h]) => ring(atp(p), side, up, w * s, h * s));
  smooth(out, beak, S.small, TILE.skin, () => c.legs, { facet: st.creases ? 0.6 : facet });
  const comb = male ? 1.7 : 1.0;
  const crown = [[[0, 0.145, -0.03], 0.006, 0.008], [[0, 0.16, -0.05], 0.009, 0.02 * comb], [[0, 0.165, -0.07], 0.009, 0.022 * comb], [[0, 0.155, -0.09], 0.006, 0.012 * comb]].map(([p, w, h]) => ring(atp(p), side, up, w * s, h * s));
  smooth(out, curve(crown, Math.min(2, P.bird === 1 ? 1 : 2)), S.small, TILE.skin, () => c.pale, { facet });
  const wattle = [[[0, 0.105, -0.085], 0.008, 0.008], [[0, 0.085, -0.09], 0.01, 0.012 * Math.sqrt(comb)], [[0, 0.065, -0.088], 0.004, 0.004]].map(([p, w, h]) => ring(atp(p), side, fwd, w * s, h * s));
  smooth(out, wattle, S.small, TILE.skin, () => c.pale, { facet });
  if (down < 0.9){
    const kk = 1 - down;
    for (const [sgn, ph] of [[-1, 0], [1, Math.PI]]){
      const q = Q.x(Math.sin(pose.phase + ph) * 0.45 * Math.max(0, Math.min(1, pose.walk)));
      const hip = V.add(v(sgn * 0.04, 0.17, 0.01), sink), hock = V.add(hip, Q.rot(q, v(0, -0.1 * kk, 0.01))), foot = V.add(hip, Q.rot(q, v(0, -0.17 * kk, -0.005)));
      const leg = [ring(hip, X, Z, 0.035 * s, 0.04 * s), ring(V.lerp(hip, hock, 0.6), X, Z, 0.02 * s, 0.025 * s), ring(hock, X, Z, 0.009 * s, 0.009 * s), ring(foot, X, Z, 0.008 * s, 0.008 * s)];
      smooth(out, leg, S.small, TILE.skin, (n, t) => t < 0.4 ? c.body : c.legs, { facet });
      for (const toe of [-0.5, 0, 0.5]){
        const dir = V.norm(Q.rot(Q.y(toe), [0, 0, -1]), Z);
        smooth(out, [ring(foot, X, Y, 0.008 * s, 0.006 * s), ring(V.sub(V.add(foot, V.mul(dir, 0.045 * s)), [0, 0.003 * s, 0]), X, Y, 0.003 * s, 0.003 * s)], Math.max(4, S.small - 1), TILE.skin, () => c.legs, { facet });
      }
    }
  }
}

// The species as the game's data has them (assets/config/husbandry.ron): body, pale, legs, head; size 1.
const SPECIES = {
  horse: { colours: [[0.45, 0.28, 0.16], [0.62, 0.46, 0.32], [0.16, 0.11, 0.08], [0.40, 0.25, 0.15]], height: 1.55 },
  plodder: { colours: [[0.52, 0.34, 0.20], [0.92, 0.88, 0.80], [0.30, 0.22, 0.16], [0.46, 0.30, 0.18]], height: 1.35 },
  woolback: { colours: [[0.90, 0.87, 0.78], [0.82, 0.78, 0.68], [0.20, 0.17, 0.15], [0.24, 0.20, 0.18]], height: 0.8 },
  "meadow-hen": { colours: [[0.93, 0.89, 0.80], [0.80, 0.16, 0.12], [0.92, 0.70, 0.22], [0.95, 0.92, 0.85]], height: 0.42 },
};
const STAND = { phase: 0, walk: 0, head_down: 0, glance: 0, bob: 0, tail: 0, shift: 0, down: 0, saddled: false };
function coatOf(id, body){ const c = SPECIES[id].colours; return { body: body || c[0], pale: c[1], legs: c[2], head: c[3], pattern: 0 }; }
function build(id, styleName, pose = STAND, coat = null){
  const out = builder(), st = STYLES[styleName], c = coat || coatOf(id);
  TEXM = st.texM || 0;
  if (id === "meadow-hen") bird(out, c, false, pose, SPECIES[id].height / 0.42, st);
  else beast(out, id, c, pose, SPECIES[id].height / BEASTS[id].withers, st);
  return out;
}
// The plodder's carriage measured off a mesh: the withers (the top of the body at the fore legs), the poll, the
// muzzle (the point furthest forward), the lowest point of the head, and the neck's and face's angles.
function measure(id, styleName, pose){
  const f = BEASTS[id], s = SPECIES[id].height / f.withers;
  const parts = { body: builder(), all: build(id, styleName, pose) };
  const pts = []; for (let i = 0; i < parts.all.position.length; i += 3) pts.push([parts.all.position[i], parts.all.position[i + 1], parts.all.position[i + 2]]);
  let withers = 0; for (const p of pts) if (Math.abs(p[2] - f.foreZ * s) < 0.08 * s && Math.abs(p[0]) < 0.1 * s) withers = Math.max(withers, p[1]);
  const front = pts.reduce((a, b) => b[2] < a[2] ? b : a);
  const lowestAhead = pts.filter(p => p[2] < f.foreZ * s - 0.25 * s).reduce((a, b) => b[1] < a[1] ? b : a, [0, 9, 0]);
  return { withers, muzzle: front, lowestHead: lowestAhead };
}
// Compare a built mesh with the game's (today.json): vertex for vertex, the largest distance in metres.
function check(built, game){
  if (built.position.length !== game.position.length || built.index.length !== game.index.length) return { same: false, why: `counts ${built.position.length / 3}/${game.position.length / 3} vertices, ${built.index.length / 3}/${game.index.length / 3} triangles` };
  let worst = 0; for (let i = 0; i < built.position.length; i++) worst = Math.max(worst, Math.abs(built.position[i] - game.position[i]));
  let idx = 0; for (let i = 0; i < built.index.length; i++) if (built.index[i] !== game.index[i]) idx++;
  let uv = 0; for (let i = 0; i < built.tile.length; i++) uv = Math.max(uv, Math.abs(gameUV(built.tile[i], built.uv[2 * i]) - game.uv[2 * i]), Math.abs(built.uv[2 * i + 1] - game.uv[2 * i + 1]));
  let col = 0; for (let i = 0; i < built.colour.length; i++){ const c = built.colour[i], lin = c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4); col = Math.max(col, Math.abs(lin - game.colour_linear[i])); }
  return { same: worst < 1e-3 && idx === 0 && uv < 1e-3 && col < 1e-3, worst, idx, uv, col };
}
const API = { STYLES, BEASTS, CARRIAGE, SPECIES, STAND, TILE, build, measure, check, coatOf, smooth, curve, ring, builder, profile, V, Q };
if (typeof module !== "undefined" && module.exports) module.exports = API; else root.HerdForms = API;
})(typeof self !== "undefined" ? self : this);
