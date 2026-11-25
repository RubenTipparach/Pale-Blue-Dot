// River Craft (M34): the approved first-person hand (M36 Tools in Hand, the Hand tab; hand-model design 5,
// approved for the game 2026-11-17, "tools in hand approved, implment"; "You know what that hand looks good").
// GENERATED, do not edit: copied verbatim from docs/mockups/tool-motion.html at f100b308 (its proposed hand
// builder, from "The proposed hand" to "Paint the proposed hand"), with the axe's grip from that page's HELD data
// (assets/models/held_tools.ron). River Craft lights it with the terrain's own lighting (sun, sky and every lamp
// near it) instead of tool-motion's baked view light: see handMesh in rowing.js.
(function () {
"use strict";
const RC = window.RC;
const v3 = (a) => new THREE.Vector3(a[0], a[1], a[2]);
const lin8 = (c) => { c /= 255; return c <= 0.04045 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4); };
function rng(seed) { let s = seed >>> 0; return () => ((s = (s * 1664525 + 1013904223) >>> 0) / 4294967296); }
const TOOL = {"id":"axe","butt":[0.299066,-0.481313,-0.534672],"rotation":[-0.546237,-0.449027,0.567979,0.421189],"scale":0.0374227,"fist":[0.29,-0.3,-0.58],"hand":[{"axis":[0.0853,0.8729,0.4804],"at":[4.6159,-1.1732,-1.7209],"radius":0.6955,"length":4.9745,"across":[0.9964,-0.0747,-0.0411],"width":1.9739,"colour":[227,176,137]},{"axis":[0.1287,-0.0922,0.9874],"at":[6.3541,0.6165,0.003],"radius":0.4827,"length":2.0066,"colour":[227,176,137]},{"axis":[-0.0904,-0.9832,-0.1584],"at":[6.4204,-0.036,0.8214],"radius":0.4827,"length":1.3181,"colour":[195,151,118]},{"axis":[-0.1786,-0.5352,-0.8257],"at":[6.285,-0.8532,0.3445],"radius":0.4345,"length":1.0972,"colour":[227,176,137]},{"axis":[0.1759,-0.1779,0.9682],"at":[5.4803,0.6639,0.1872],"radius":0.5073,"length":2.1907,"colour":[227,176,137]},{"axis":[-0.0192,-0.9431,-0.332],"at":[5.646,-0.1375,0.9496],"radius":0.5073,"length":1.4805,"colour":[195,151,118]},{"axis":[-0.1094,-0.4909,-0.8643],"at":[5.5825,-0.9865,0.3297],"radius":0.4565,"length":1.0923,"colour":[227,176,137]},{"axis":[0.2313,-0.2889,0.929],"at":[4.5263,0.588,0.2277],"radius":0.4786,"length":1.9972,"colour":[227,176,137]},{"axis":[0.0483,-0.9263,-0.3738],"at":[4.7697,-0.2794,0.8364],"radius":0.4786,"length":1.4645,"colour":[195,151,118]},{"axis":[-0.1192,-0.2775,-0.9533],"at":[4.7558,-0.9871,0.2314],"radius":0.4308,"length":0.9225,"colour":[227,176,137]},{"axis":[0.2927,-0.4388,0.8496],"at":[3.544,0.486,0.2342],"radius":0.4173,"length":1.3042,"colour":[227,176,137]},{"axis":[0.1508,-0.8521,-0.5012],"at":[3.7805,-0.1579,0.4872],"radius":0.4173,"length":1.0878,"colour":[195,151,118]},{"axis":[-0.08,-0.2912,-0.9533],"at":[3.8259,-0.641,-0.0374],"radius":0.3755,"length":0.7781,"colour":[227,176,137]},{"axis":[0.3218,0.446,0.8352],"at":[5.9769,-2.1244,-0.9781],"radius":0.6075,"length":1.8712,"colour":[227,176,137]},{"axis":[0.1516,0.6103,0.7775],"at":[6.3553,-1.3261,0.2667],"radius":0.5245,"length":1.532,"colour":[195,151,118]},{"axis":[-0.0239,0.6924,0.7211],"at":[6.4486,-0.614,1.1055],"radius":0.4692,"length":1.0141,"colour":[227,176,137]},{"axis":[-0.6428,-0.6717,-0.3683],"at":[3.9646,-3.6108,-3.0614],"radius":1.0227,"length":1.4318,"colour":[227,176,137]},{"axis":[-0.6428,-0.6717,-0.3683],"at":[3.2679,-4.339,-3.4606],"radius":1.35,"length":0.7364,"colour":[58,82,110]},{"axis":[-0.6428,-0.6717,-0.3683],"at":[-0.913,-8.7082,-5.8559],"radius":1.2682,"length":12.2727,"colour":[77,106,140]}]};
// The proposed hand (rod-motion, the owner's "improved hand model, better
// normals, better shading, more uniform colors, hell throw in some
// textures"). Built from the same bones as the game's hand (the 19 prisms of
// held_tools.ron, so every grip still closes on its handle at the fist), but
// each bone is a chamfered block: an octagonal section with bevelled ends, so
// the silhouette reads rounder while every face stays flat (one normal per
// face, the chamfer bands carrying the in-between normals). One skin tone
// for the whole hand, nail plates on the five tips, a hem ring on the sleeve,
// and small nearest-sampled pixel textures drawn in code.
// Parts in held_tools.ron: 0 back of the hand, 1-12 four fingers of three
// segments (proximal, middle, distal), 13-15 the thumb, 16 the wrist, 17 the
// cuff, 18 the sleeve.
const SKINS = [["Game", [227, 176, 137]], ["Fair", [240, 196, 158]], ["Tan", [201, 154, 116]], ["Brown", [156, 107, 74]], ["Dark", [122, 80, 56]]];
const HAND_SHAPE = { thumb_len: 0.72, grip_turn: 45 * Math.PI / 180, thumb_sink: 0.4 };
const HAND_LOOK = { skin: SKINS[0][1], cloth: [77, 106, 140], cuff: [58, 82, 110], textured: true,
  // Palm and back (hand-model design 5; owner, 2026-11-17: "usually back of
  // the hand and fingers a little darker than palm, and palm has kind of that
  // palm texture"): the back is the skin a little darker, the palm the skin
  // lifted toward a pale pink, with crease rows. two_tone off is version 8.
  two_tone: true, back_k: [0.86, 0.82, 0.80], palm_mix: 0.45, palm_to: [252, 214, 204], ao: 0.6,
  // The fingertips flush redder (the owner's reference, Sketchfab "Female hands"): the last segment of each finger and the thumb.
  tip_flush: 0.22, tip_to: [226, 140, 128] };
const backTone = () => HAND_LOOK.two_tone ? HAND_LOOK.skin.map((c, i) => c * HAND_LOOK.back_k[i]) : HAND_LOOK.skin;
const palmTone = () => HAND_LOOK.two_tone ? HAND_LOOK.skin.map((c, i) => c + (HAND_LOOK.palm_to[i] - c) * HAND_LOOK.palm_mix) : HAND_LOOK.skin;
const TEX_UNIT = 0.24;   // tool units per texel (~9 mm on the pickaxe's hand)
const HAND_PARTS = { skin: 0, nail: 1, cloth: 2, cuff: 3, palm: 4, fold: 5 };
function texCanvas(draw) { const c = document.createElement("canvas"); c.width = c.height = 16; draw(c.getContext("2d")); return c; }
const hexOf = c => "#" + c.map(x => Math.max(0, Math.min(255, Math.round(x))).toString(16).padStart(2, "0")).join("");
const tone = (c, k) => c.map(x => x * k);
function handTextures() {
  const r = rng(91), skin = backTone(), cloth = HAND_LOOK.cloth, cuff = HAND_LOOK.cuff, palm = palmTone();
  const px = (x, X, Y, col) => { x.fillStyle = hexOf(col); x.fillRect(X, Y, 1, 1); };
  return {
    // Skin: the tone with a light mottle, a few warmer and a few paler pixels.
    skin: texCanvas(x => { for (let Y = 0; Y < 16; Y++) for (let X = 0; X < 16; X++) { const u = r(); px(x, X, Y, u < 0.07 ? [skin[0] * 1.0, skin[1] * 0.95, skin[2] * 0.93] : u < 0.13 ? tone(skin, 1.03) : u < 0.19 ? tone(skin, 0.975) : skin); } }),
    // Palm: the paler tone, its own mottle, and darker crease rows across it
    // (the palm's lines and the finger joints' folds), a pixel wide, broken
    // and wandering a row up or down. Drawn after the others so their
    // pixels are as before.
    palm: null,
    // Nails: paler than the skin, a lighter row at the tip.
    nail: texCanvas(x => { const n = [skin[0] * 0.55 + 240 * 0.45, skin[1] * 0.55 + 214 * 0.45, skin[2] * 0.55 + 200 * 0.45]; for (let Y = 0; Y < 16; Y++) for (let X = 0; X < 16; X++) px(x, X, Y, Y < 3 ? tone(n, 1.07) : r() < 0.15 ? tone(n, 0.97) : n); }),
    // Cloth: a two-tone weave and an odd darker thread.
    cloth: texCanvas(x => { for (let Y = 0; Y < 16; Y++) for (let X = 0; X < 16; X++) { const w = ((X >> 1) + (Y >> 1)) & 1; px(x, X, Y, r() < 0.05 ? tone(cloth, 0.88) : tone(cloth, w ? 1.03 : 0.96)); } }),
    // Cuff: the band's colour with a row of stitches near each edge.
    cuff: texCanvas(x => { for (let Y = 0; Y < 16; Y++) for (let X = 0; X < 16; X++) { const st = (Y === 3 || Y === 12) && (X % 3) !== 2; px(x, X, Y, st ? tone(cuff, 1.32) : r() < 0.1 ? tone(cuff, 0.95) : cuff); } }),
  };
}
// The palm (hand-model design 5.1): 32 x 32, drawn for the palm's own UV
// island: u from the thumb's side (left) to the little finger's (right),
// v from the wrist (bottom row) to the knuckles (top row).
function palmTexture() {
  const r = rng(173), palm = palmTone(), N = 32;
  const c = document.createElement("canvas"); c.width = c.height = N; const x = c.getContext("2d");
  const px = (X, Y, col) => { x.fillStyle = hexOf(col); x.fillRect(X, Y, 1, 1); };
  for (let Y = 0; Y < N; Y++) for (let X = 0; X < N; X++) { const u = r(); px(X, Y, u < 0.07 ? [palm[0], palm[1] * 0.96, palm[2] * 0.95] : u < 0.14 ? tone(palm, 1.02) : palm); }
  if (!HAND_LOOK.two_tone) return c;
  const line = (pts, k) => { for (let q = 0; q + 1 < pts.length; q++) { const [x0, y0] = pts[q], [x1, y1] = pts[q + 1], n = Math.max(Math.abs(x1 - x0), Math.abs(y1 - y0)); for (let t = 0; t <= n; t++) px(Math.round(x0 + (x1 - x0) * t / n), Math.round(y0 + (y1 - y0) * t / n), tone(palm, k)); } };
  line([[31, 9], [24, 9], [17, 8], [12, 6], [9, 4]], 0.8);        // the heart line
  line([[3, 12], [10, 13], [18, 15], [26, 18]], 0.8);             // the head line
  line([[3, 11], [6, 15], [8, 20], [9, 25], [9, 31]], 0.8);       // the life line, round the thumb
  for (let X = 6; X < N; X += 7) line([[X, 1], [X + 4, 1]], 0.86); // the folds at the fingers' base
  return c;
}
// The inside of a finger, the thumb or the wrist: 16 x 16, one island per
// segment (u across, v along the bone), a fold at each joint.
function foldTexture() {
  const r = rng(211), palm = palmTone();
  return texCanvas(x => {
    for (let Y = 0; Y < 16; Y++) for (let X = 0; X < 16; X++) {
      const fold = HAND_LOOK.two_tone && (Y === 1 || Y === 14) && X > 2 && X < 13;
      x.fillStyle = hexOf(fold ? tone(palm, 0.82) : r() < 0.1 ? tone(palm, 0.975) : palm); x.fillRect(X, Y, 1, 1);
    }
  });
}
function canvasTex(c) {
  const t = new THREE.CanvasTexture(c);
  t.magFilter = THREE.NearestFilter; t.minFilter = THREE.NearestFilter; t.generateMipmaps = false;
  t.wrapS = t.wrapT = THREE.RepeatWrapping; t.encoding = THREE.sRGBEncoding;
  return t;
}
// The hand's shared materials: one per texture, plus a plain one (no map)
// for the diagnostics. All unlit; the shade is baked into vertex colours.
const HAND_MATS = (() => {
  const mk = () => new THREE.MeshBasicMaterial({ vertexColors: true });
  const m = { tex: [mk(), mk(), mk(), mk(), mk(), mk()], plain: mk(), canvases: null };
  m.refresh = () => {
    m.canvases = handTextures(); m.canvases.palm = palmTexture(); m.canvases.fold = foldTexture();
    ["skin", "nail", "cloth", "cuff", "palm", "fold"].forEach((k, i) => { if (m.tex[i].map) m.tex[i].map.dispose(); m.tex[i].map = canvasTex(m.canvases[k]); m.tex[i].needsUpdate = true; });
  };
  m.refresh();
  return m;
})();
// A chamfered block along `axis`: half sizes w (along u) and h (along v),
// length len, chamfer c. Adds its faces to the builder `B`.
function chamferBlock(B, o) {
  const { at, axis, u, w, h, len, c, group, colour, crease, uvAlong } = o;
  const palm = o.palm ? Object.assign({ at, axis, len, E: Math.max(w, h), pu: axis.clone().cross(o.palm.dir).normalize() }, o.palm) : null;
  const v = axis.clone().cross(u);
  // A taper (the palm's): the width at the near end (z0) is taper[0] of w,
  // at the far end taper[1], straight between.
  const taperAt = z => o.taper ? o.taper[0] + (o.taper[1] - o.taper[0]) * (z / len + 0.5) : 1;
  const sec = (s, z) => {   // the octagon, scaled inward by s at the bevels
    const W = w * taperAt(z) - s, Hh = h - s, C = Math.max(0.0001, c - s * 0.4);
    return [[W - C, -Hh], [W, -Hh + C], [W, Hh - C], [W - C, Hh], [-W + C, Hh], [-W, Hh - C], [-W, -Hh + C], [-W + C, -Hh]];
  };
  const bev = Math.min(c, len * 0.3);
  const ringAt = (z, s) => sec(s, z).map(([a, b]) => at.clone().addScaledVector(u, a).addScaledVector(v, b).addScaledVector(axis, z));
  const z0 = -len / 2, z1 = len / 2;
  // A round tip (the thumb's): a longer bevel at the far end in two steps,
  // so the end reads as a rounded pad, not a squared finger end.
  const tb = o.round ? Math.min(Math.min(w, h) * 0.75, len * 0.45) : bev;
  const r0 = ringAt(z0, bev), r1 = ringAt(z0 + bev, 0), r2 = ringAt(z1 - tb, 0), r3 = ringAt(z1, tb * (o.round ? 0.95 : 1));
  const rM = o.round ? ringAt(z1 - tb * 0.35, tb * 0.32) : null;
  const cBot = at.clone().addScaledVector(axis, z0), cTop = at.clone().addScaledVector(axis, z1);
  const quad = (a, b, cc, d, k, along) => B.face([a, b, cc, d], at, group, colour, k, along ? { axis, z0, len } : null, palm);
  for (let i = 0; i < 8; i++) {
    const j = (i + 1) % 8;
    quad(r1[i], r1[j], r2[j], r2[i], 1, uvAlong);
    quad(r0[i], r0[j], r1[j], r1[i], crease ? crease[0] : 1, false);
    if (rM) { quad(r2[i], r2[j], rM[j], rM[i], 1, false); quad(rM[i], rM[j], r3[j], r3[i], 1, false); }
    else quad(r2[i], r2[j], r3[j], r3[i], crease ? crease[1] : 1, false);
    B.face([cBot, r0[j], r0[i]], at, group, colour, crease ? crease[0] : 1, null, palm);
    B.face([cTop, r3[i], r3[j]], at, group, colour, crease ? crease[1] : 1, null, palm);
  }
}
// The builder: flat faces, each wound outward from its part's centre, a
// planar UV per face at TEX_UNIT tool units a texel (or along the axis for
// the cuff, so its stitch rows sit at its edges).
function HandBuilder() {
  const P = [], N = [], UV = [], COL = [], K = [], PART = [], FL = [], groups = [[], [], [], [], [], []];
  let part = -1, flush = 0;
  const e1 = new THREE.Vector3(), e2 = new THREE.Vector3(), n = new THREE.Vector3(), mid = new THREE.Vector3();
  function tri(a, b, c, centre, group, colour, k, along, palm) {
    e1.subVectors(b, a); e2.subVectors(c, a); n.crossVectors(e1, e2);
    if (n.lengthSq() < 1e-14) return;
    n.normalize();
    mid.copy(a).add(b).add(c).multiplyScalar(1 / 3).sub(centre);
    if (n.dot(mid) < 0) { [b, c] = [c, b]; n.negate(); }
    // A skin face turned toward the handle is the palm's side, and takes the
    // block's palm island: projected on the palm's plane in the block's
    // frame (hand-model design 5.1).
    let island = null;
    if (palm && group === HAND_PARTS.skin && n.dot(palm.dir) > 0.35) { group = palm.group; colour = palmTone(); island = palm; }
    const T = e1.subVectors(b, a).normalize(), Bt = new THREE.Vector3().crossVectors(n, T);
    for (const p of [a, b, c]) {
      groups[group].push(P.length / 3);
      P.push(p.x, p.y, p.z); N.push(n.x, n.y, n.z); COL.push(colour); K.push(k); PART.push(part); FL.push(flush);
      if (island) { const d = p.clone().sub(island.at); let uu = d.dot(island.pu) / (2 * island.E) + 0.5, vv = d.dot(island.axis) / island.len + 0.5; UV.push(island.fu ? 1 - uu : uu, island.fv ? 1 - vv : vv); }
      else if (along) { const z = p.clone().sub(along.at).dot(along.axis), Ta = new THREE.Vector3().crossVectors(n, along.axis).normalize(); UV.push(p.dot(Ta) / TEX_UNIT / 16, (z - along.z0) / along.len); }
      else UV.push(p.dot(T) / TEX_UNIT / 16, p.dot(Bt) / TEX_UNIT / 16);
    }
  }
  return {
    setPart(i) { part = i; },
    setFlush(f) { flush = f; },
    // Baked ambient occlusion (owner, 2026-11-17: "Bake some ao"): each
    // vertex is darkened by the other bones near it and in front of it, each
    // bone a few spheres along its length: sum (r/d)^2 * max(0, n.d), then
    // k *= 1 - strength * min(1, sum). Once, at build; the hand is rigid.
    bakeAO(spheres, strength) {
      const p = new THREE.Vector3(), nn = new THREE.Vector3(), d = new THREE.Vector3();
      for (let i = 0; i < K.length; i++) {
        p.set(P[i * 3], P[i * 3 + 1], P[i * 3 + 2]); nn.set(N[i * 3], N[i * 3 + 1], N[i * 3 + 2]);
        let sum = 0;
        for (const sp of spheres) {
          if (sp.part === PART[i]) continue;
          d.subVectors(sp.c, p); const dd = d.length(); if (dd < 1e-6) continue;
          const cos = nn.dot(d) / dd; if (cos <= 0) continue;
          const rr = Math.min(1, (sp.r * sp.r) / (dd * dd)); sum += rr * cos;
        }
        K[i] *= 1 - strength * Math.min(1, sum);
      }
    },
    face(pts, centre, group, colour, k, along, palm) {
      if (along) along.at = centre;
      tri(pts[0], pts[1], pts[2], centre, group, colour, k, along, palm);
      if (pts.length === 4) tri(pts[0], pts[2], pts[3], centre, group, colour, k, along, palm);
    },
    geometry() {
      // Order the vertices by group so the four textured materials draw as
      // geometry groups.
      const g = new THREE.BufferGeometry(), order = groups.flat();
      const pick = (src, w) => { const out = new Float32Array(order.length * w); order.forEach((i, j) => { for (let q = 0; q < w; q++) out[j * w + q] = src[i * w + q]; }); return out; };
      g.setAttribute("position", new THREE.BufferAttribute(pick(P, 3), 3));
      g.setAttribute("uv", new THREE.BufferAttribute(pick(UV, 2), 2));
      g.setAttribute("color", new THREE.BufferAttribute(new Float32Array(order.length * 3), 3));
      let start = 0; groups.forEach((gr, i) => { g.addGroup(start, gr.length, i); start += gr.length; });
      g.userData = { N: Array.from(pick(N, 3)), cols: order.map(i => COL[i]), k: order.map(i => K[i]), flush: order.map(i => FL[i]), proposedHand: true };
      return g;
    },
  };
}
function proposedHandGeometry(tool, sleeveExtra, only) {
  const B = HandBuilder(), parts = tool.hand;
  const q = new THREE.Quaternion(...tool.rotation).normalize();
  const fistT = v3(tool.fist).sub(v3(tool.butt)).applyQuaternion(q.clone().invert()).multiplyScalar(1 / tool.scale);
  const pc = i => v3(parts[i].at);
  const handleDir = pc(10).sub(pc(1)).normalize();   // index to little finger: along the handle
  // The thumb is shortened from its root out (owner, 2026-11-17: "thumb is
  // too long"): the game's three segments, each HAND_SHAPE.thumb_len of its
  // length, chained from the root. Then two segments, not three (owner:
  // "Two thumbs segments not 3"), from the root to the tip, so the tip stays
  // where it closes on the handle.
  const thumb = {}; let thumbRoot = null;
  { let start = null; const pts = [];
    for (let i = 13; i <= 15; i++) {
      const p = parts[i], ax = v3(p.axis).normalize(), L = p.length * HAND_SHAPE.thumb_len;
      if (start === null) { start = v3(p.at).addScaledVector(ax, -p.length / 2); pts.push(start.clone()); }
      start = start.clone().addScaledVector(ax, L); pts.push(start.clone());
    }
    const seg = (a, b) => ({ at: a.clone().add(b).multiplyScalar(0.5), len: a.distanceTo(b), axis: b.clone().sub(a).normalize() });
    // The two halves are equal (owner: "should be 50/50 for the thumb"): the
    // joint sits half way along the chain from the root to the tip.
    // The root sinks into the palm (owner, 2026-11-18: "root or bottom of
    // thumb is floating above the palm making it look like a big finger"):
    // the first segment starts thumb_sink of the way toward the palm's
    // centre, so it grows out of the hand's mass.
    thumbRoot = pts[0].clone();
    pts[0].lerp(pc(0), HAND_SHAPE.thumb_sink);
    const L = [0, 1, 2].map(k => pts[k].distanceTo(pts[k + 1])), half = (L[0] + L[1] + L[2]) / 2;
    let mid = pts[3].clone(), run = 0;
    for (let k = 0; k < 3; k++) { if (run + L[k] >= half) { mid = pts[k].clone().lerp(pts[k + 1], (half - run) / L[k]); break; } run += L[k]; }
    thumb[13] = seg(pts[0], mid); thumb[14] = { skip: true }; thumb[15] = seg(mid, pts[3]); }
  // The palm is one shaped piece, not a box (owner, 2026-11-17: "I REPEAT A
  // RECTANGLE IS NOT A GOOD PALM SHAPE"; "Thumb needs to be more connected
  // to the palm"). Its outline is traced off the bones in the palm's plane:
  // the four knuckles across the top, the little finger's side narrowing to
  // the wrist, the thumb's side swelling out (the mound at the thumb's base)
  // and running up to the thumb's root, then the web back to the index
  // knuckle. It is extruded through the hand's thickness with bevelled
  // edges, the back a little domed.
  function palmMesh(B, o) {
    const { at, axis, h, palm, colour } = o;
    const nd = palm ? palm.dir.clone() : new THREE.Vector3().crossVectors(axis, handleDir).normalize();
    const pu = axis.clone().cross(nd).normalize();
    const to2 = q => { const d = q.clone().sub(at); return [d.dot(pu), d.dot(axis)]; };
    const base = k => { const p = parts[k], ax = v3(p.axis).normalize(), a = v3(p.at), e1 = a.clone().addScaledVector(ax, p.length / 2), e2 = a.clone().addScaledVector(ax, -p.length / 2); return e1.distanceTo(at) < e2.distanceTo(at) ? e1 : e2; };
    const K = [1, 4, 7, 10].map(k => to2(base(k)));          // index .. little knuckle
    const wr = to2(base(16)), ww = (parts[16].width || parts[16].radius) * 0.95;
    const T = thumbRoot ? to2(thumbRoot) : [K[0][0], (K[0][1] + wr[1]) / 2];
    const up = Math.sign(K[1][1] - wr[1]) || 1, ts = Math.sign(K[0][0] - K[3][0]) || 1;   // toward the knuckles; toward the thumb
    const fr = parts[1].radius * 0.75;
    // The outline, going round: the knuckles (pushed out a little past each
    // base), the little finger's side, the wrist, the thumb's mound, the
    // thumb's root, the web.
    const O = [
      [K[0][0] + ts * fr * 0.4, K[0][1] + up * fr * 0.3],
      [K[1][0], K[1][1] + up * fr * 0.55],
      [K[2][0], K[2][1] + up * fr * 0.5],
      [K[3][0] - ts * fr * 0.2, K[3][1] + up * fr * 0.2],
      [K[3][0] - ts * fr * 0.9, K[3][1] - up * fr * 0.9],
      [K[3][0] * 0.55 + wr[0] * 0.45 - ts * fr * 1.0, (K[3][1] + wr[1]) / 2],
      [wr[0] - ts * ww, wr[1] + up * fr * 0.4],
      [wr[0] + ts * ww * 0.9, wr[1] + up * fr * 0.3],
      [(wr[0] + T[0]) / 2 + ts * fr * 1.1, (wr[1] + T[1]) / 2],     // the mound
      [T[0] + ts * fr * 0.5, T[1]],
      [T[0] + ts * fr * 0.1, T[1] + up * fr * 0.9],                 // the thumb's root, its far side
      [(T[0] + K[0][0]) / 2 + ts * fr * 0.2, (T[1] + K[0][1]) / 2 + up * fr * 0.4],   // the web
    ];
    // Keep it counter-clockwise in (pu, axis).
    let area = 0; for (let k = 0; k < O.length; k++) { const [x0, y0] = O[k], [x1, y1] = O[(k + 1) % O.length]; area += x0 * y1 - x1 * y0; }
    if (area < 0) O.reverse();
    const cx = O.reduce((a, q) => a + q[0], 0) / O.length, cy = O.reduce((a, q) => a + q[1], 0) / O.length;
    const ext = Math.max(...O.map(q => Math.abs(q[0] - cx))), eyy = Math.max(...O.map(q => Math.abs(q[1] - cy)));
    const thick = y => h * (0.82 + 0.18 * Math.min(1, Math.max(0, up * (y - wr[1]) / Math.max(1e-6, up * (K[1][1] - wr[1])))));
    const P3 = (x, y, z) => at.clone().addScaledVector(pu, x).addScaledVector(axis, y).addScaledVector(nd, z);
    const c = h * 0.45;
    const inset = (q, d) => { const dx = cx - q[0], dy = cy - q[1], L = Math.hypot(dx, dy) || 1; return [q[0] + dx / L * d, q[1] + dy / L * d]; };
    const ring = (side, s) => O.map(q => { const r = inset(q, s); return P3(r[0], r[1], side * thick(q[1]) * (s > 0 ? 1 : 1 - 0.45 * c / h)); });
    // Back (-nd) and palm (+nd): an inset ring on the face, a full ring at the bevel.
    const bk0 = ring(-1, c), bk1 = ring(-1, 0), pm1 = ring(1, 0), pm0 = ring(1, c);
    const isl = palm ? { dir: nd, group: HAND_PARTS.palm, at: P3(cx, cy, 0), axis, pu, E: ext, len: 2 * eyy, fu: thumbRoot ? (to2(thumbRoot)[0] - cx) > 0 : false, fv: up < 0 } : null;
    const n = O.length;
    for (let k = 0; k < n; k++) {
      const m = (k + 1) % n, ex = O[m][0] - O[k][0], ey = O[m][1] - O[k][1];
      const outC = P3((O[k][0] + O[m][0]) / 2 - ey * -4, (O[k][1] + O[m][1]) / 2 - -ex * -4, 0);   // a point inside, for winding
      const inside = P3((O[k][0] + O[m][0]) / 2 + (-ey), (O[k][1] + O[m][1]) / 2 + ex, 0);
      B.face([bk0[k], bk0[m], bk1[m], bk1[k]], inside, HAND_PARTS.skin, colour, 0.92, null, isl);
      B.face([bk1[k], bk1[m], pm1[m], pm1[k]], inside, HAND_PARTS.skin, colour, 1, null, isl);
      B.face([pm1[k], pm1[m], pm0[m], pm0[k]], inside, HAND_PARTS.skin, colour, 1, null, isl);
      // The back, a little domed; the palm, flat.
      const cb = P3(cx, cy, -thick(cy) * 1.12), cp = P3(cx, cy, thick(cy) * 0.98);
      B.face([cb, bk0[m], bk0[k]], P3(cx, cy, 0), HAND_PARTS.skin, colour, 1, null, isl);
      B.face([cp, pm0[k], pm0[m]], P3(cx, cy, 0), HAND_PARTS.skin, colour, 1, null, isl);
    }
  }
  parts.forEach((p, i) => {
    if (only === "hand" && i >= 16 || only === "arm" && i < 16) return;
    B.setPart(i);
    let axis = v3(p.axis).normalize(), at = v3(p.at), len = p.length;
    if (thumb[i] && thumb[i].skip) return;
    if (thumb[i]) { at = thumb[i].at; len = thumb[i].len; axis = thumb[i].axis; }
    const first = p.across ? v3(p.across) : Math.abs(axis.x) < 0.9 ? new THREE.Vector3(1, 0, 0) : new THREE.Vector3(0, 1, 0);
    let u = first.clone().addScaledVector(axis, -first.dot(axis)).normalize();
    const finger = i >= 1 && i <= 15, distal = finger && (i % 3 === 0), seg = finger ? (i - 1) % 3 : -1;
    // Turn a finger block so a flat face looks away from the handle (the
    // back of the finger), so the nail and the knuckles sit on a face.
    let out = null;
    if (finger) {
      const rel = at.clone().sub(fistT); rel.addScaledVector(handleDir, -rel.dot(handleDir)).addScaledVector(axis, -rel.dot(axis));
      if (rel.lengthSq() > 1e-8) { out = rel.normalize(); u = axis.clone().cross(out).normalize(); }
    }
    let w = (p.width || p.radius), h = p.radius, group = HAND_PARTS.skin, colour = backTone(), uvAlong = false, crease = null;
    const flush = HAND_LOOK.two_tone && distal ? HAND_LOOK.tip_flush : 0;
    // The palm's side: toward the handle. A finger's is opposite its back;
    // the back of the hand's and the wrist's point from the block to the
    // fist, square to the handle.
    let palm = null;
    if (HAND_LOOK.two_tone && i <= 16) {
      if (out) palm = { dir: out.clone().negate(), group: HAND_PARTS.fold };
      else if (i === 0 || i === 16) {
        // The palm's side is where the fingertips curl to (into the palm in
        // every grip), square to the bone (the owner's own fist, 2026-11-17:
        // "Use my hand as reference").
        const tips = new THREE.Vector3(); for (const t of [3, 6, 9, 12]) tips.add(pc(t)); tips.multiplyScalar(0.25);
        const d = tips.sub(at); d.addScaledVector(axis, -d.dot(axis));
        if (d.lengthSq() > 1e-8) {
          palm = { dir: d.normalize(), group: i === 0 ? HAND_PARTS.palm : HAND_PARTS.fold };
          if (i === 0) {
            // Lay the island out from the thumb's side (u 0) and the wrist (v 0).
            const pu = axis.clone().cross(palm.dir).normalize();
            palm.fu = thumbRoot.clone().sub(at).dot(pu) > 0;
            palm.fv = pc(16).sub(at).dot(axis) > 0;
          }
        }
      }
    }
    if (i === 17) { group = HAND_PARTS.cuff; colour = HAND_LOOK.cuff; uvAlong = true; }
    if (i === 18) {
      group = HAND_PARTS.cloth; colour = HAND_LOOK.cloth;
      // Lengthen the sleeve at its far end (away from the cuff).
      const far = at.clone().addScaledVector(axis, len / 2).distanceTo(pc(17)) > at.clone().addScaledVector(axis, -len / 2).distanceTo(pc(17)) ? 1 : -1;
      at.addScaledVector(axis, far * sleeveExtra / 2); len += sleeveExtra;
    }
    // Slimmer fingers with flatter sides, each segment tapering toward the
    // tip (owner, 2026-11-17: "fingers look like sausges").
    let ftaper = null;
    if (finger) { w *= 0.8; h *= 0.84; crease = seg < 2 ? [1, 0.88] : [1, 1]; ftaper = seg === 2 ? [0.95, 0.8] : [1.0, 0.92]; }   // a darker bevel at each knuckle
    // The thumb is thicker than a finger, its tip a rounded pad (owner,
    // 2026-11-17: "Tip of thumb could be rounder. It doesnt look much like a
    // thumb").
    const isThumb = i >= 13 && i <= 15;
    if (isThumb) { w *= 1.08; h *= 1.04; }   // a little thicker than a finger ("thumb is really big")
    if (i === 16) { w *= 1.05; h *= 0.92; len *= 1.15; }   // the wrist runs into the cuff
    const c = Math.min(w, h) * (finger ? 0.26 : 0.3);
    B.setFlush(flush);
    if (i === 0) palmMesh(B, { at, axis, h: h * 1.05, palm, colour });
    else chamferBlock(B, { at, axis, u, w, h, len, c, group, colour, crease, uvAlong, palm, round: isThumb && distal, taper: ftaper });
    B.setFlush(0);
    // The hem: a ring of the sleeve's cloth where it meets the cuff.
    if (i === 18) {
      const near = v3(p.at).addScaledVector(axis, (v3(p.at).addScaledVector(axis, p.length / 2).distanceTo(pc(17)) < v3(p.at).addScaledVector(axis, -p.length / 2).distanceTo(pc(17)) ? 1 : -1) * (p.length / 2 - 0.3));
      chamferBlock(B, { at: near, axis, u, w: w * 1.06, h: h * 1.06, len: 0.5, c: c * 1.06, group, colour: tone(HAND_LOOK.cloth, 0.86), crease: null });
    }
    // A nail plate on each tip, on the back face near the end.
    if (distal && out) {
      const nat = at.clone().addScaledVector(axis, len * 0.14).addScaledVector(out, h * 0.93);
      const nw = isThumb ? 0.7 : 0.62, nl = isThumb ? 0.5 : 0.56, nz = isThumb ? 0.08 : 0.14;
      chamferBlock(B, { at: at.clone().addScaledVector(axis, len * (nz - 0.14)).add(nat.clone().sub(at)), axis, u, w: w * nw, h: h * 0.14, len: len * nl, c: h * 0.08, group: HAND_PARTS.nail, colour: tone(HAND_LOOK.skin, 1.06), crease: null });
    }
  });
  // The occluders: every bone of the hand and arm (whichever half is being
  // built), three spheres along each, and the tool's handle through the fist.
  if (HAND_LOOK.ao > 0) {
    const spheres = [];
    parts.forEach((p, i) => {
      if (thumb[i] && thumb[i].skip) return;
      const ax = thumb[i] ? thumb[i].axis : v3(p.axis).normalize(), at = thumb[i] ? thumb[i].at : v3(p.at), L = thumb[i] ? thumb[i].len : p.length;
      const r = Math.min(p.width || p.radius, p.radius);
      for (const t of [-0.33, 0, 0.33]) spheres.push({ c: at.clone().addScaledVector(ax, t * L), r, part: i });
    });
    for (let t = -2; t <= 2; t++) spheres.push({ c: fistT.clone().addScaledVector(handleDir, t * 0.8), r: 0.55, part: -2 });
    B.bakeAO(spheres, HAND_LOOK.ao);
  }
  const g = B.geometry();
  // The grip turned about the handle (through the fist), so the fist stays
  // closed on it: grip_turn, rad (the owner's fist: the back of the hand and
  // the thumb on top toward the eye).
  if (HAND_SHAPE.grip_turn) {
    const qq = new THREE.Quaternion().setFromAxisAngle(handleDir, HAND_SHAPE.grip_turn), P = g.getAttribute("position"), t = new THREE.Vector3(), N = g.userData.N;
    for (let i = 0; i < P.count; i++) { t.fromBufferAttribute(P, i).sub(fistT).applyQuaternion(qq).add(fistT); P.setXYZ(i, t.x, t.y, t.z); t.set(N[i * 3], N[i * 3 + 1], N[i * 3 + 2]).applyQuaternion(qq); N[i * 3] = t.x; N[i * 3 + 1] = t.y; N[i * 3 + 2] = t.z; }
    P.needsUpdate = true;
  }
  return g;
}
RC.HAND = {TOOL, HAND_LOOK, HAND_PARTS, proposedHandGeometry, handTextures, palmTexture, foldTexture, v3, lin8};
})();
