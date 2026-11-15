// Low-poly 3D creatures for "Seas, Rivers and Marshes" (ocean.html, M25).
// The owner, 2026-11-15: "ducks should be 3d, birds in general are 3d" and "all fish should also be 3d, unless they are
// caught, and a player drops them as 2d sprites". So every animal alive in a scene is a small mesh built here, flat
// shaded, one colour per face, each colour taken from the creature's own sprite (the Sweaft-64 tones read off
// ocean/*.png, m7/*.png, m9/*.png and the game's fish icons). The fish body follows fish.html's buildFish (M9): a
// six-sided body in rings with tail, dorsal, anal and pectoral fins; here it is coloured per face instead of painted.
// A model is one unit long, head along +x, back along +y. Swimmers sway from the tail in the vertex shader (one
// uniform clock, a phase from where the body is), so a school of hundreds is one InstancedMesh and one draw.
// Exposes C3.make(W, id, size), C3.geo(id), C3.mat(W, id), C3.Kit (the face-coloured geometry kit the reef reuses).
(function (root) {
  "use strict";
  const T3 = THREE;
  const CC = {};
  const lin = h => CC[h] || (CC[h] = new T3.Color(h).convertSRGBToLinear());
  const hh = (a, b) => { let h = (a * 374761393 + b * 668265263) | 0; h = Math.imul(h ^ (h >>> 13), 1274126177); return ((h ^ (h >>> 16)) >>> 0) / 4294967296; };
  const _m = new T3.Matrix4(), _q = new T3.Quaternion(), _e = new T3.Euler(), _p = new T3.Vector3(), _s = new T3.Vector3();
  const tfm = tf => { if (!tf) return null; if (tf.isMatrix4) return tf; _e.set(...(tf.r || [0, 0, 0]), tf.o || "XYZ"); _q.setFromEuler(_e); _p.set(...(tf.p || [0, 0, 0])); const s = tf.s; _s.set(...(s == null ? [1, 1, 1] : typeof s === "number" ? [s, s, s] : s)); return _m.compose(_p, _q, _s).clone(); };

  // ---- the kit: geometry with one colour per triangle, a colour being a hex or a function of the face's centre
  function Kit() {
    const P = [], C = [];
    const k = {
      add(geo, color, tf) {
        let g = geo.index ? geo.toNonIndexed() : geo.clone(); const M = tfm(tf); if (M) g.applyMatrix4(M);
        const a = g.attributes.position;
        for (let i = 0; i + 2 < a.count; i += 3) {
          const cx = (a.getX(i) + a.getX(i + 1) + a.getX(i + 2)) / 3, cy = (a.getY(i) + a.getY(i + 1) + a.getY(i + 2)) / 3, cz = (a.getZ(i) + a.getZ(i + 1) + a.getZ(i + 2)) / 3;
          const c = lin(typeof color === "function" ? color(cx, cy, cz, i / 3) : color);
          for (let j = 0; j < 3; j++) { P.push(a.getX(i + j), a.getY(i + j), a.getZ(i + j)); C.push(c.r, c.g, c.b); }
        }
        g.dispose(); return k;
      },
      count: () => P.length / 9,
      build() { const g = new T3.BufferGeometry(); g.setAttribute("position", new T3.Float32BufferAttribute(P, 3)); g.setAttribute("color", new T3.Float32BufferAttribute(C, 3)); g.computeVertexNormals(); g.computeBoundingSphere(); return g; },
    };
    return k;
  }
  // primitives
  const ell = (rx, ry, rz, ws = 6, hs = 4) => new T3.SphereGeometry(1, ws, hs).scale(rx, ry, rz);
  const bx = (w, h, d) => new T3.BoxGeometry(w, h, d);
  const cyl = (rt, rb, h, n = 6) => new T3.CylinderGeometry(rt, rb, h, n, 1);
  const shape = pts => { const s = new T3.Shape(); s.moveTo(pts[0][0], pts[0][1]); for (const p of pts.slice(1)) s.lineTo(p[0], p[1]); return new T3.ShapeGeometry(s); };
  // a body of rings along x: prof = [[x, halfHeight, halfWidth, yOffset?], ...] from the front back; nv sides
  function lathe(prof, nv = 6, rot = Math.PI / 6) {
    const pos = [];
    const ring = R => { const out = []; for (let j = 0; j < nv; j++) { const th = 2 * Math.PI * j / nv + rot; out.push([R[0], (R[3] || 0) + R[1] * Math.cos(th), R[2] * Math.sin(th)]); } return out; };
    const rs = prof.map(ring);
    for (let i = 0; i + 1 < rs.length; i++) for (let j = 0; j < nv; j++) {
      const a = rs[i][j], b = rs[i][(j + 1) % nv], c = rs[i + 1][j], d = rs[i + 1][(j + 1) % nv];
      pos.push(...a, ...c, ...b, ...b, ...c, ...d);
    }
    const g = new T3.BufferGeometry(); g.setAttribute("position", new T3.Float32BufferAttribute(pos, 3)); return g;
  }

  // ======================= fish (after fish.html's buildFish) =======================
  // p: back, side, belly, fin, h (depth over length), w (width over length), tail, dorsal [u0, u1, height], anal,
  // bands [[u0, u1, hex]], lateral (a stripe along the side), spots (hex, how many in ten), eel, wings, lure, beak
  function fishGeo(p) {
    const K = Kit(), eel = !!p.eel, NU = eel ? 16 : 9, BL = eel ? 0.98 : 0.8, H = p.h / 2, Wd = p.w / 2;
    const xAt = u => 0.5 - u * BL, uAt = x => (0.5 - x) / BL;
    const prof = u => eel ? (u < 0.07 ? Math.pow(Math.sin(u / 0.07 * Math.PI / 2), 0.8) : 1 - 0.8 * Math.pow((u - 0.07) / 0.93, 1.6))
      : (u < (p.peak ?? 0.32) ? Math.pow(Math.sin(u / (p.peak ?? 0.32) * Math.PI / 2), 0.75) : 0.17 + 0.83 * Math.pow(1 - (u - (p.peak ?? 0.32)) / (1 - (p.peak ?? 0.32)), 1.25));
    const R = []; for (let i = 0; i <= NU; i++) { const u = i / NU, f = i === 0 ? 0 : prof(u); R.push([xAt(u), H * f, Wd * f]); }
    R.push([xAt(1) - 0.004, 0, 0]);
    const side = p.side || p.back, belly = p.belly || side;
    K.add(lathe(R, 6), (x, y, z, n) => {
      const u = clamp01(uAt(x)), hy = Math.max(1e-4, H * prof(u)), yn = y / hy;
      if (p.bands) for (const [u0, u1, c] of p.bands) if (u >= u0 && u <= u1) return c;
      if (p.spots && hh(n, 7) < p.spots / 10 && yn > -0.2) return p.spotC || p.fin;
      if (p.lateral && Math.abs(yn) < 0.22 && u > 0.15) return p.lateral;
      if (p.head && u < 0.22) return p.head;
      return yn > 0.3 ? p.back : yn < -0.35 ? belly : side;
    });
    const fin = p.fin || p.back, xt = xAt(1), hp = Math.max(H * prof(1), 0.012), TL = eel ? 0.06 : xt + 0.52;
    const tails = {
      fork: [[xt + .02, hp], [xt - TL, hp + TL * .8], [xt - TL * .55, 0], [xt - TL, -hp - TL * .8], [xt + .02, -hp]],
      deep: [[xt + .02, hp], [xt - TL * 1.05, hp + TL], [xt - TL * .8, hp + TL * .7], [xt - TL * .45, 0], [xt - TL * .8, -hp - TL * .7], [xt - TL * 1.05, -hp - TL], [xt + .02, -hp]],
      lunate: [[xt + .02, hp], [xt - TL, hp + TL * 1.05], [xt - TL * .55, hp * .4], [xt - TL * .5, 0], [xt - TL * .55, -hp * .4], [xt - TL, -hp - TL * 1.05], [xt + .02, -hp]],
      trunc: [[xt + .02, hp], [xt - TL, hp + TL * .55], [xt - TL * 1.04, 0], [xt - TL, -hp - TL * .55], [xt + .02, -hp]],
      round: [[xt + .02, hp], [xt - TL * .6, hp + TL * .45], [xt - TL, 0], [xt - TL * .6, -hp - TL * .45], [xt + .02, -hp]],
    };
    if (!eel && p.tail !== "none") K.add(shape(tails[p.tail || "fork"]), p.tailC || fin);
    const fins = (list, sgn) => { for (const [u0, u1, Hh] of list || []) { const base = []; for (let i = 0; i <= 6; i++) { const u = u0 + (u1 - u0) * i / 6; base.push([xAt(u), sgn * H * prof(u) * 0.86]); } const ap = [[base[1][0] + Hh * 0.1, base[1][1] + sgn * Hh], [base[6][0] - Hh * 0.45, base[6][1] + sgn * Hh * 0.3]]; K.add(shape([base[0], ...ap, base[6], ...base.slice(1, 6).reverse()]), fin); } };
    fins(p.dorsal || (eel ? [[0.12, 0.97, 0.03]] : [[0.28, 0.62, H * 0.9]]), 1);
    fins(p.anal || (eel ? [[0.4, 0.97, 0.025]] : [[0.6, 0.82, H * 0.5]]), -1);
    if (!eel) for (const s of [1, -1]) {
      const pw = p.wings ? 0.38 : H * 0.55, pc = p.wings ? p.wingC || fin : fin;
      const g = shape(p.wings ? [[0, 0], [-0.32, 0.03], [-0.36, -0.02], [-0.1, -0.05]] : [[0, 0], [-pw, 0.03], [-pw * 0.8, -pw * 0.45]]);
      K.add(g, pc, {p: [xAt(0.27), -H * 0.25, s * Wd * 0.75], r: [s * (p.wings ? 1.45 : 0.9), 0, 0]});
    }
    // eyes: a dark cube each side, a light ring for big-eyed deep fish
    const ex = xAt(eel ? 0.04 : 0.13), ew = Wd * prof(eel ? 0.04 : 0.13);
    for (const s of [1, -1]) { if (p.eyeRing) K.add(bx(0.07, 0.07, 0.02), p.eyeRing, {p: [ex, H * 0.25, s * (ew + 0.004)]}); K.add(bx(0.045, 0.045, 0.03), p.eye || 0x161612, {p: [ex, H * 0.25, s * (ew + 0.008)]}); }
    if (p.beak) K.add(cyl(0.0, Math.max(H, Wd) * 0.35, 0.08, 4), p.beak, {p: [0.5, -H * 0.05, 0], r: [0, 0, -Math.PI / 2]});
    if (p.lure) { K.add(bx(0.18, 0.012, 0.012), p.back, {p: [0.48, H * 1.05, 0], r: [0, 0, 0.6]}); }
    return K.build();
  }
  const clamp01 = x => x < 0 ? 0 : x > 1 ? 1 : x;

  // ======================= crawlers =======================
  function crabGeo(p) {   // carapace wider than long; eight legs; two claws (one big for the fiddler)
    const K = Kit();
    K.add(ell(0.28, 0.12, 0.36, 8, 4), (x, y) => y > 0.05 ? p.top : p.under, {p: [0, 0.16, 0]});
    for (const s of [1, -1]) for (let i = 0; i < 4; i++) { const x = 0.12 - i * 0.1; K.add(bx(0.04, 0.04, 0.3), p.leg, {p: [x, 0.12, s * 0.42], r: [s * 0.55, (i - 1.5) * 0.25 * s, 0]}); K.add(bx(0.035, 0.18, 0.035), p.leg, {p: [x - 0.01, 0.06, s * 0.56]}); }
    for (const s of [1, -1]) { const big = p.fiddler && s === 1 ? 1.9 : 1; if (p.fiddler && s === 1) continue; K.add(bx(0.2 * big, 0.08 * big, 0.08 * big), p.claw, {p: [0.32, 0.17, s * 0.22], r: [0, s * 0.5, 0]}); K.add(bx(0.06, 0.06, 0.06), p.claw, {p: [0.22, 0.17, s * 0.22]}); }
    for (const s of [1, -1]) K.add(cyl(0.012, 0.012, 0.12, 4), 0x161612, {p: [0.24, 0.29, s * 0.07]});
    for (const s of [1, -1]) K.add(bx(0.04, 0.04, 0.04), 0x161612, {p: [0.24, 0.36, s * 0.07]});
    return K.build();
  }
  function lobsterGeo(p) {   // carapace, segmented tail and fan, two claws on arms, antennae
    const K = Kit();
    K.add(lathe([[0.42, 0.0, 0.0], [0.36, 0.06, 0.07], [0.2, 0.09, 0.09], [0.05, 0.09, 0.085], [-0.02, 0.07, 0.07]], 6), (x, y) => y > 0.02 ? p.top : p.under, {p: [0, 0.1, 0]});
    for (let i = 0; i < 5; i++) { const x = -0.04 - i * 0.075, r = 0.065 - i * 0.007; K.add(lathe([[x + 0.035, r, r * 1.1], [x - 0.035, r * 0.95, r * 1.05]], 6), i % 2 ? p.top : p.band || p.top, {p: [0, 0.1, 0]}); }
    K.add(shape([[0, 0.0], [-0.12, 0.09], [-0.15, 0], [-0.12, -0.09]]), p.top, {p: [-0.41, 0.09, 0], r: [Math.PI / 2, 0, 0]});
    for (const s of [1, -1]) {
      K.add(bx(0.18, 0.03, 0.03), p.top, {p: [0.42, 0.11, s * 0.1], r: [0, s * 0.5, 0]});
      K.add(ell(0.13, 0.05, 0.065, 6, 3), p.claw, {p: [0.6, 0.11, s * 0.2], r: [0, s * 0.3, 0]});
      K.add(bx(0.7, 0.012, 0.012), p.ant || p.top, {p: [0.75, 0.16, s * 0.18], r: [0, s * 0.35, 0.12]});
      for (let i = 0; i < 4; i++) K.add(bx(0.025, 0.12, 0.025), p.under, {p: [0.25 - i * 0.07, 0.05, s * 0.1], r: [s * 0.6, 0, 0]});
    }
    for (const s of [1, -1]) K.add(bx(0.03, 0.03, 0.03), 0x161612, {p: [0.4, 0.16, s * 0.04]});
    return K.build();
  }
  function hermitGeo(p) {
    const K = Kit();
    for (let i = 0; i < 4; i++) K.add(ell(0.22 - i * 0.045, 0.2 - i * 0.04, 0.2 - i * 0.04, 7, 4), i % 2 ? p.shell : p.shell2, {p: [-0.12 - i * 0.04, 0.2 + i * 0.07, 0.02 * i], r: [0, i * 0.9, 0.5]});
    K.add(ell(0.12, 0.09, 0.12, 6, 3), p.body, {p: [0.12, 0.1, 0]});
    for (const s of [1, -1]) { K.add(bx(0.14, 0.07, 0.07), p.claw, {p: [0.24, 0.08, s * 0.08]}); K.add(bx(0.18, 0.025, 0.025), p.body, {p: [0.1, 0.03, s * 0.16], r: [0, s * 0.6, 0]}); K.add(bx(0.03, 0.08, 0.03), 0x161612, {p: [0.22, 0.2, s * 0.04]}); }
    return K.build();
  }
  function starGeo(p) {
    const K = Kit();
    K.add(cyl(0.13, 0.15, 0.06, 5), p.top, {p: [0, 0.035, 0]});
    for (let i = 0; i < 5; i++) { const a = i * Math.PI * 2 / 5; K.add(cyl(0.02, 0.07, 0.42, 4).rotateZ(-Math.PI / 2).translate(0.28, 0.0, 0), (x, y) => y > 0 ? p.top : p.edge, {p: [0, 0.035, 0], r: [0, a, 0], s: [1, 0.55, 1]}); K.add(bx(0.03, 0.02, 0.03), p.dot, {p: [Math.cos(a) * 0.2, 0.07, -Math.sin(a) * 0.2]}); }
    return K.build();
  }
  function urchinGeo(p) {
    const K = Kit(), b = new T3.IcosahedronGeometry(0.2, 0);
    K.add(b, (x, y, z, n) => n % 3 ? p.body : p.body2, {p: [0, 0.18, 0]});
    const v = b.attributes.position, seen = new Set();
    for (let i = 0; i < v.count; i++) { const d = new T3.Vector3(v.getX(i), v.getY(i), v.getZ(i)).normalize(), key = d.toArray().map(x => x.toFixed(2)).join(); if (seen.has(key) || d.y < -0.5) continue; seen.add(key);
      const q = new T3.Quaternion().setFromUnitVectors(new T3.Vector3(0, 1, 0), d); K.add(cyl(0.0, 0.025, 0.42, 4).translate(0, 0.21, 0), p.spine, new T3.Matrix4().compose(d.clone().multiplyScalar(0.16).add(new T3.Vector3(0, 0.18, 0)), q, new T3.Vector3(1, 1, 1))); }
    return K.build();
  }
  function shellGeo(p) {   // a pair of shells, a bivalve lying on its side (mussel) or a cup (abalone)
    const K = Kit();
    if (p.cup) { K.add(ell(0.42, 0.14, 0.32, 8, 4), (x, y, z, n) => y > 0.12 ? p.top : hh(n, 3) < 0.25 ? p.dark : p.edge, {p: [0, 0.06, 0]}); for (let i = 0; i < 5; i++) K.add(bx(0.04, 0.02, 0.04), p.dark, {p: [0.25 - i * 0.1, 0.19 - Math.abs(i - 2) * 0.01, 0.14]}); K.add(bx(0.5, 0.04, 0.38), p.inner, {p: [0, 0.0, 0]}); return K.build(); }
    for (const s of [1, -1]) K.add(ell(0.45, 0.18, 0.1, 8, 4), (x, y) => y > 0.15 ? p.top : p.edge, {p: [0, 0.16, s * 0.08], r: [s * 0.25, 0, 0]});
    K.add(bx(0.6, 0.03, 0.05), p.inner, {p: [0.05, 0.3, 0]});
    return K.build();
  }
  function cucumberGeo(p) {
    const K = Kit(), pr = []; for (let i = 0; i <= 8; i++) { const u = i / 8, r = Math.sin(Math.PI * (0.06 + 0.88 * u)) * 0.12; pr.push([0.5 - u, r, r * 1.1]); }
    K.add(lathe(pr, 6), (x, y, z, n) => y < 0.02 ? p.under : hh(n, 5) < 0.3 ? p.bump : p.top, {p: [0, 0.12, 0]});
    for (let i = 0; i < 7; i++) K.add(cyl(0.0, 0.025, 0.06, 4), p.bump, {p: [0.35 - i * 0.11, 0.24, (hh(i, 2) - 0.5) * 0.14]});
    return K.build();
  }
  function shrimpGeo(p) {   // a curved body of segments, a tail fan, legs, long feelers
    const K = Kit(), pr = []; for (let i = 0; i <= 8; i++) { const u = i / 8, r = 0.08 * (u < 0.2 ? 0.6 + 2 * u : 1 - 0.7 * (u - 0.2) / 0.8); pr.push([0.4 - u * 0.75, r, r * 0.8, -0.18 * u * u]); }
    K.add(lathe(pr, 6), (x, y, z, n) => y > -0.06 && n % 4 ? p.top : p.band || p.top, {p: [0, 0.18, 0]});
    K.add(shape([[0, 0], [-0.14, 0.07], [-0.14, -0.07]]), p.top, {p: [-0.34, 0.0, 0], r: [Math.PI / 2, 0, -0.5]});
    for (const s of [1, -1]) { K.add(bx(0.7, 0.01, 0.01), p.ant || p.top, {p: [0.72, 0.3, s * 0.05], r: [0, s * 0.18, 0.25]}); K.add(bx(0.035, 0.035, 0.035), 0x161612, {p: [0.38, 0.24, s * 0.05]}); for (let i = 0; i < 4; i++) K.add(bx(0.015, 0.12, 0.015), p.band || p.top, {p: [0.2 - i * 0.08, 0.08, s * 0.05]}); }
    return K.build();
  }
  function squidGeo(p) {   // mantle trailing back with its fins, head, arms forward
    const K = Kit();
    K.add(lathe([[0.1, 0.09, 0.09], [-0.05, 0.1, 0.1], [-0.25, 0.08, 0.08], [-0.42, 0.03, 0.03], [-0.5, 0, 0]], 6), (x, y, z, n) => hh(n, 4) < 0.25 ? p.spot : p.body, {p: [0, 0, 0]});
    K.add(shape([[-0.25, 0], [-0.5, 0.0], [-0.42, 0.16]]), p.fin, {r: [Math.PI / 2, 0, 0]}); K.add(shape([[-0.25, 0], [-0.5, 0.0], [-0.42, -0.16]]), p.fin, {r: [Math.PI / 2, 0, 0]});
    K.add(ell(0.08, 0.075, 0.08, 6, 4), p.body, {p: [0.16, 0, 0]});
    for (const s of [1, -1]) K.add(bx(0.05, 0.05, 0.02), p.eye || 0x161612, {p: [0.17, 0.03, s * 0.08]});
    for (let i = 0; i < 8; i++) { const a = i * Math.PI / 4; K.add(cyl(0.004, 0.02, 0.3, 4).rotateZ(-Math.PI / 2).translate(0.15, 0, 0), p.arm || p.body, {p: [0.22, Math.cos(a) * 0.04, Math.sin(a) * 0.04], r: [a, 0, 0]}); }
    return K.build();
  }
  function rayGeo(p) {   // the body; the wings are parts
    const K = Kit();
    K.add(lathe([[0.42, 0, 0], [0.3, 0.04, 0.14], [0.05, 0.06, 0.2], [-0.15, 0.05, 0.14], [-0.3, 0.02, 0.05], [-0.33, 0, 0]], 6), (x, y) => y > 0 ? p.top : p.under);
    K.add(cyl(0.004, 0.018, 0.5, 4).rotateZ(Math.PI / 2), p.top, {p: [-0.55, 0.0, 0]});
    for (const s of [1, -1]) K.add(bx(0.04, 0.03, 0.03), 0x161612, {p: [0.2, 0.05, s * 0.07]});
    return K.build();
  }
  const rayWing = (p, s) => { const K = Kit(); K.add(shape([[0.32, 0], [-0.25, 0], [-0.02, s * 0.5]]), (x, y, z, n) => hh(n, 9) < 0.2 ? p.spot || p.top : p.top, {r: [Math.PI / 2, 0, 0]}); return K.build(); };
  function flatfishGeo(p) {
    const K = Kit();
    K.add(lathe([[0.45, 0, 0], [0.35, 0.03, 0.17], [0.05, 0.04, 0.28], [-0.25, 0.03, 0.18], [-0.38, 0.01, 0.05]], 6), (x, y, z, n) => y < 0 ? p.under : hh(n, 3) < 0.22 ? p.spot : p.top, {p: [0, 0.04, 0]});
    K.add(shape([[-0.36, 0], [-0.52, 0.11], [-0.52, -0.11]]), p.spot, {p: [0, 0.04, 0], r: [Math.PI / 2, 0, 0]});
    K.add(bx(0.04, 0.05, 0.04), 0x161612, {p: [0.26, 0.09, 0.04]}); K.add(bx(0.04, 0.05, 0.04), 0x161612, {p: [0.22, 0.09, -0.05]});
    return K.build();
  }
  function frogGeo(p) {   // squat body, head, folded hind legs, front legs
    const K = Kit();
    K.add(ell(0.26, 0.15, 0.22, 7, 4), (x, y) => y > 0.17 ? p.top : p.under, {p: [-0.02, 0.17, 0], r: [0, 0, 0.25]});
    K.add(ell(0.15, 0.1, 0.17, 7, 4), (x, y) => y > 0.24 ? p.top : p.under, {p: [0.22, 0.25, 0]});
    for (const s of [1, -1]) {
      K.add(bx(0.06, 0.06, 0.06), p.eyeC || 0x161612, {p: [0.28, 0.34, s * 0.09]});
      K.add(bx(0.3, 0.06, 0.08), p.top, {p: [-0.16, 0.08, s * 0.2], r: [0, s * 0.3, 0.3]}); K.add(bx(0.28, 0.04, 0.07), p.leg || p.top, {p: [-0.04, 0.03, s * 0.27], r: [0, -s * 0.4, 0]});
      K.add(bx(0.05, 0.16, 0.05), p.leg || p.top, {p: [0.2, 0.08, s * 0.15]});
      for (let i = 0; i < 3; i++) K.add(bx(0.03, 0.02, 0.03), p.spot || p.top, {p: [-0.1 + i * 0.1, 0.3, s * 0.06 * i]});
    }
    return K.build();
  }
  function lurkerGeo(p) {   // a long low swamp hunter: body, a long head, four splayed legs, a thick tail; pale eyes
    const K = Kit();
    K.add(lathe([[0.5, 0, 0], [0.42, 0.035, 0.06], [0.25, 0.05, 0.07], [0.18, 0.07, 0.11], [0.0, 0.09, 0.15], [-0.18, 0.08, 0.12], [-0.3, 0.05, 0.07], [-0.5, 0.01, 0.015]], 6), (x, y, z, n) => y > 0.03 ? (hh(n, 2) < 0.3 ? p.spot : p.top) : p.under, {p: [0, 0.08, 0]});
    for (let i = 0; i < 7; i++) K.add(cyl(0, 0.025, 0.05, 4), p.spot, {p: [0.08 - i * 0.07, 0.18 - i * 0.006, 0]});
    for (const s of [1, -1]) { K.add(bx(0.03, 0.03, 0.03), p.eye, {p: [0.24, 0.155, s * 0.05]}); for (const x of [0.12, -0.15]) K.add(bx(0.06, 0.04, 0.14), p.top, {p: [x, 0.03, s * 0.17], r: [0, 0, 0]}); }
    return K.build();
  }
  function turtleGeo(p) {   // shell dome and plastron; head; the flippers are parts
    const K = Kit();
    K.add(ell(0.36, 0.13, 0.3, 8, 4), (x, y, z, n) => y < 0.02 ? p.under : (n % 5 === 0 ? p.scute : p.shell), {p: [0, 0.02, 0]});
    K.add(ell(0.36, 0.03, 0.3, 8, 2), p.under, {p: [0, -0.01, 0]});
    K.add(ell(0.1, 0.07, 0.07, 6, 4), p.skin, {p: [0.42, 0.02, 0]});
    for (const s of [1, -1]) K.add(bx(0.03, 0.03, 0.02), 0x161612, {p: [0.46, 0.05, s * 0.055]});
    K.add(cyl(0.0, 0.04, 0.12, 4).rotateZ(Math.PI / 2), p.skin, {p: [-0.38, 0, 0]});
    return K.build();
  }
  const turtleFlipper = (p, front) => { const K = Kit(); K.add(bx(front ? 0.14 : 0.1, 0.02, front ? 0.36 : 0.16).translate(0, 0, front ? 0.18 : 0.08), p.skin); return K.build(); };
  function jellyGeo(p) {   // tentacles (the bell is a part, it pulses)
    const K = Kit();
    for (let i = 0; i < 8; i++) { const a = i * Math.PI / 4; K.add(bx(0.012, 0.6, 0.012), i % 2 ? p.tent : p.tent2, {p: [Math.cos(a) * 0.14, -0.32, Math.sin(a) * 0.14], r: [0.08 * Math.sin(a), 0, 0.08 * Math.cos(a)]}); }
    for (let i = 0; i < 4; i++) { const a = i * Math.PI / 2 + 0.4; K.add(bx(0.04, 0.4, 0.02), p.arm, {p: [Math.cos(a) * 0.04, -0.22, Math.sin(a) * 0.04], r: [0, a, 0]}); }
    return K.build();
  }
  const jellyBell = p => { const K = Kit(); K.add(new T3.SphereGeometry(0.28, 8, 4, 0, Math.PI * 2, 0, Math.PI / 2).scale(1, 0.8, 1), (x, y, z, n) => y > 0.16 ? p.bell2 : p.bell); K.add(cyl(0.29, 0.29, 0.03, 8), p.rim, {p: [0, 0.0, 0]}); return K.build(); };
  function octoGeo(p) {   // the mantle (the arms are parts)
    const K = Kit();
    K.add(ell(0.22, 0.2, 0.2, 7, 5), (x, y, z, n) => hh(n, 6) < 0.25 ? p.spot : p.body, {p: [-0.12, 0.26, 0], r: [0, 0, 0.4]});
    K.add(ell(0.13, 0.11, 0.15, 7, 4), p.body, {p: [0.06, 0.12, 0]});
    for (const s of [1, -1]) { K.add(bx(0.06, 0.05, 0.05), p.eye, {p: [0.12, 0.19, s * 0.1]}); K.add(bx(0.07, 0.015, 0.015), 0x161612, {p: [0.13, 0.19, s * 0.125]}); }
    return K.build();
  }
  const octoArm = p => { const K = Kit(); K.add(cyl(0.012, 0.045, 0.5, 4).rotateZ(-Math.PI / 2).translate(0.25, 0, 0), (x, y) => y < 0 ? p.sucker : p.body); return K.build(); };

  // ======================= mammals =======================
  function sealGeo(p) {
    const K = Kit();
    K.add(lathe([[0.5, 0, 0], [0.44, 0.08, 0.08], [0.36, 0.1, 0.1], [0.3, 0.09, 0.09], [0.15, 0.13, 0.14], [-0.05, 0.14, 0.15], [-0.25, 0.1, 0.11], [-0.38, 0.05, 0.05], [-0.42, 0, 0]], 6), (x, y, z, n) => y > 0.06 ? (hh(n, 4) < 0.2 ? p.spot : p.back) : y < -0.05 ? p.belly : p.side, {p: [0, 0.13, 0]});
    for (const s of [1, -1]) { K.add(bx(0.035, 0.035, 0.03), 0x161612, {p: [0.43, 0.2, s * 0.06]}); K.add(bx(0.18, 0.025, 0.08), p.back, {p: [0.15, 0.04, s * 0.14], r: [0, s * 0.6, 0]}); K.add(shape([[0, 0], [-0.16, 0.0], [-0.18, s * 0.12]]), p.back, {p: [-0.38, 0.13, 0], r: [Math.PI / 2, 0, 0]}); }
    K.add(bx(0.03, 0.02, 0.03), 0x161612, {p: [0.5, 0.15, 0]});
    return K.build();
  }
  function otterGeo(p) {   // rafting: on its back, belly and face up, paws on its chest holding an urchin
    const K = Kit(), up = !!p.back2up;
    K.add(lathe([[0.5, 0, 0], [0.45, 0.06, 0.06], [0.36, 0.075, 0.075], [0.3, 0.065, 0.065], [0.12, 0.1, 0.11], [-0.12, 0.1, 0.1], [-0.26, 0.07, 0.07], [-0.3, 0, 0]], 6), (x, y) => (up ? y < 0.02 : y > 0.02) ? p.back : p.belly, {p: [0, 0.03, 0]});
    K.add(cyl(0.01, 0.045, 0.3, 5).rotateZ(Math.PI / 2), p.back, {p: [-0.42, 0.03, 0]});
    for (const s of [1, -1]) { K.add(bx(0.03, 0.03, 0.03), 0x161612, {p: [0.42, up ? 0.075 : 0.07, s * 0.04], r: [0, 0, 0]}); K.add(bx(0.06, 0.03, 0.04), p.back, {p: [0.36, up ? 0.0 : 0.09, s * 0.06]}); }
    K.add(bx(0.03, 0.03, 0.04), 0x161612, {p: [0.5, up ? 0.05 : 0.04, 0]});
    if (up) { for (const s of [1, -1]) K.add(bx(0.12, 0.03, 0.03), p.back, {p: [0.2, 0.13, s * 0.05], r: [0, s * 0.3, 0.3]}); K.add(new T3.IcosahedronGeometry(0.05, 0), 0x7d2dbd, {p: [0.24, 0.16, 0]}); for (const s of [1, -1]) K.add(bx(0.12, 0.03, 0.04), p.back, {p: [-0.24, 0.08, s * 0.06], r: [0, 0, -0.4]}); }
    else for (const s of [1, -1]) for (const x of [0.15, -0.15]) K.add(bx(0.04, 0.1, 0.04), p.back, {p: [x, -0.07, s * 0.08]});
    return K.build();
  }
  function beaverGeo(p) {   // a chunky body, head and ears, orange teeth, a flat scaled paddle tail; a stick when carrying
    const K = Kit();
    K.add(lathe([[0.5, 0, 0], [0.45, 0.08, 0.07], [0.36, 0.1, 0.1], [0.28, 0.1, 0.1], [0.12, 0.17, 0.17], [-0.08, 0.18, 0.18], [-0.22, 0.13, 0.13], [-0.27, 0, 0]], 6), (x, y) => y > -0.06 ? p.back : p.belly, {p: [0, 0.16, 0]});
    K.add(bx(0.3, 0.035, 0.15), (x, y, z, n) => n % 2 ? p.tail : p.tail2, {p: [-0.4, 0.06, 0], r: [0, 0, 0.12]});
    for (const s of [1, -1]) { K.add(bx(0.035, 0.035, 0.03), 0x161612, {p: [0.4, 0.23, s * 0.07]}); K.add(bx(0.04, 0.04, 0.03), p.back, {p: [0.32, 0.29, s * 0.07]}); for (const x of [0.2, -0.1]) K.add(bx(0.06, 0.1, 0.05), p.belly, {p: [x, 0.03, s * 0.12]}); }
    K.add(bx(0.02, 0.05, 0.04), p.teeth, {p: [0.49, 0.12, 0]}); K.add(bx(0.03, 0.03, 0.04), 0x161612, {p: [0.5, 0.18, 0]});
    if (p.stick) { K.add(cyl(0.018, 0.022, 0.9, 5).rotateX(Math.PI / 2), p.stick, {p: [0.48, 0.12, 0], r: [0, 0.25, 0]}); for (let i = 0; i < 3; i++) K.add(bx(0.08, 0.02, 0.05), p.leaf, {p: [0.5 + (i - 1) * 0.03, 0.13, 0.25 + i * 0.12]}); }
    return K.build();
  }

  // ======================= birds =======================
  // body and tail are the static mesh; head and neck, both wings and both feet are parts on pivots, so a duck can
  // paddle, tip up, tuck its head to sleep and fly; a heron can strike.
  function birdBody(p) {
    const K = Kit(), L = p.len || 1;
    const pr = (p.prof || [[0.36, 0.0, 0.0], [0.32, 0.1, 0.1], [0.2, 0.15, 0.15], [0.0, 0.15, 0.16], [-0.2, 0.12, 0.12], [-0.34, 0.07, 0.06], [-0.4, 0.0, 0.0]]).map(r => [r[0] * L, r[1], r[2], r[3]]);
    K.add(lathe(pr, 7), (x, y, z) => {
      if (p.breast && x > 0.12 * L && y > -0.08) return p.breast;
      if (p.flankC && Math.abs(z) > 0.09 && y < 0.07 && y > -0.08 && x < 0.15) return p.flankC;
      return y > 0.06 ? p.back : y < -0.06 ? p.belly : p.side || p.back;
    }, {p: [0, p.bodyY ?? 0.12, 0], r: [0, 0, p.tilt || 0]});
    K.add(shape([[0, 0.03], [-(p.tailLen || 0.16), 0.08], [-(p.tailLen || 0.16), -0.05], [0, -0.03]]), p.tailC || p.back, {p: [-0.38 * L, (p.bodyY ?? 0.12) + 0.03, 0], r: [Math.PI / 2, 0, 0.25]});
    if (p.curl) K.add(bx(0.04, 0.06, 0.02), p.curl, {p: [-0.42 * L, (p.bodyY ?? 0.12) + 0.12, 0]});
    if (p.legs) for (const s of [1, -1]) { K.add(bx(0.02, p.legs, 0.02), p.legC, {p: [0.0, p.legs / 2, s * 0.05]}); K.add(bx(0.12, 0.01, 0.06), p.legC, {p: [0.03, 0.005, s * 0.05]}); }
    return K.build();
  }
  function birdHead(p) {   // pivot at the neck's base
    const K = Kit(), nl = p.neck || 0.14, hr = p.headR || 0.1;
    K.add(cyl(0.05, 0.065, nl, 6), p.neckC || p.headC, {p: [0, nl / 2, 0], r: [0, 0, p.neckLean ?? -0.25]});
    const hx = Math.sin(-(p.neckLean ?? -0.25)) * nl, hy = Math.cos(p.neckLean ?? -0.25) * nl;
    K.add(ell(hr * 1.1, hr * 0.9, hr * 0.85, 7, 4), p.headC, {p: [hx + 0.02, hy + 0.02, 0]});
    if (p.ring) K.add(cyl(0.068, 0.068, 0.025, 6), p.ring, {p: [hx * 0.2, nl * 0.2, 0]});
    const bl = p.bill || 0.12; K.add(bx(bl, p.billH || 0.03, p.billW || 0.06), p.billC, {p: [hx + 0.02 + hr + bl / 2 - 0.02, hy + 0.0, 0], r: [0, 0, p.billTilt || -0.1]});
    for (const s of [1, -1]) K.add(bx(0.025, 0.025, 0.02), 0x161612, {p: [hx + 0.02 + hr * 0.45, hy + 0.04, s * hr * 0.8]});
    if (p.crest) K.add(bx(0.18, 0.015, 0.015), p.crest, {p: [hx - 0.1, hy + 0.05, 0], r: [0, 0, 0.2]});
    if (p.cheek) for (const s of [1, -1]) K.add(bx(0.05, 0.04, 0.01), p.cheek, {p: [hx + 0.03, hy - 0.01, s * hr * 0.85]});
    return K.build();
  }
  function birdWing(p, s) {   // span along s * z from the shoulder, chord along -x
    const K = Kit(), sp = p.span || 0.34, ch = p.chord || 0.2;
    K.add(shape([[0.04, 0], [-ch, 0], [-ch * 1.1, sp * 0.6], [-ch * 0.7, sp], [0.0, sp * 0.55]]), (x, y, z) => Math.abs(z) > sp * 0.65 && p.tipC ? p.tipC : (x < -ch * 0.55 && Math.abs(z) < sp * 0.5 && p.spec) ? p.spec : p.wingC || p.back, {r: [s * Math.PI / 2, 0, 0]});
    return K.build();
  }
  const footGeo = p => { const K = Kit(); K.add(bx(0.02, 0.08, 0.02).translate(0, -0.04, 0), p.legC || 0xfb902f); K.add(bx(0.08, 0.012, 0.06).translate(0.02, -0.08, 0), p.legC || 0xfb902f); return K.build(); };
  const dflyWing = s => { const K = Kit(); K.add(shape([[0, 0], [0.06, s * 0.45], [-0.06, s * 0.45]]), 0x84e3b5, {r: [Math.PI / 2, 0, 0]}); return K.build(); };
  function dflyGeo(p) { const K = Kit(); K.add(lathe([[0.32, 0, 0], [0.28, 0.06, 0.06], [0.18, 0.05, 0.05], [0.1, 0.04, 0.04], [-0.55, 0.018, 0.018], [-0.6, 0, 0]], 5), (x, y, z, n) => x < 0 && Math.floor(x * 20) % 2 ? p.band : p.body); for (const s of [1, -1]) K.add(bx(0.06, 0.06, 0.06), p.eye, {p: [0.28, 0.03, s * 0.05]}); return K.build(); }

  // ======================= the species =======================
  // f: a fish's params; t: its type; sway: how it swims ("fish", "eel", none); glow: an emissive colour; scale: model
  // length over the old card's size (a card's 24 px framed the creature with a margin)
  const F = (o, extra) => Object.assign({t: "fish", sway: o.eel ? "eel" : "fish"}, extra || {}, {f: o});
  const SPEC = {
    "game/fish-silverfin": F({back: 0x9fb9c2, side: 0xcfe3ea, belly: 0xf0f6f8, fin: 0x9fb9c2, h: 0.22, w: 0.11, tail: "deep"}),
    "game/fish-deepback": F({back: 0x4f6f8f, side: 0x6f8faf, belly: 0x9fb6c9, fin: 0x4f6f8f, h: 0.3, w: 0.14, tail: "trunc", dorsal: [[0.25, 0.7, 0.16]]}),
    "game/fish-serpent": F({back: 0x1e5042, side: 0x307866, belly: 0xb6d0b2, fin: 0x307866, h: 0.06, w: 0.05, eel: true, dorsal: [[0.08, 0.98, 0.04]]}, {scale: 1}),
    "game/fish-eel": F({back: 0x4c562e, side: 0x788248, belly: 0xa8b070, fin: 0x4c562e, h: 0.07, w: 0.06, eel: true}, {scale: 1}),
    "game/fish-minnow": F({back: 0x54697e, side: 0x7896b4, belly: 0xbcd0e4, fin: 0x7896b4, h: 0.2, w: 0.1, lateral: 0x3c4c5c}),
    "game/fish-perch": F({back: 0x6b4a18, side: 0xd8a13a, belly: 0xe8c878, fin: 0xc8612b, h: 0.3, w: 0.13, tail: "trunc", bands: [[0.22, 0.28, 0x6b4a18], [0.4, 0.46, 0x6b4a18], [0.58, 0.64, 0x6b4a18]], dorsal: [[0.22, 0.45, 0.13], [0.5, 0.7, 0.09]]}),
    "game/fish-reef": F({back: 0x246064, side: 0x3c9696, belly: 0x96d6d6, fin: 0x246064, h: 0.38, w: 0.1, tail: "lunate"}),
    "game/fish-ray": {t: "ray", p: {top: 0x8c7a58, under: 0xd8cfb0, spot: 0xb9a57d}, scale: 0.95},
    darter: F({back: 0xc56d25, side: 0xfb902f, belly: 0xfaaf5c, fin: 0xfb902f, tailC: 0x8d4526, h: 0.36, w: 0.14, tail: "round", bands: [[0.13, 0.19, 0xfffefe], [0.42, 0.5, 0xfffefe], [0.74, 0.8, 0xfffefe]], dorsal: [[0.22, 0.7, 0.12]]}),
    beakfish: F({back: 0x204f63, side: 0x53998b, belly: 0x84e3b5, fin: 0xc34988, h: 0.34, w: 0.15, tail: "lunate", beak: 0xeab798, head: 0x53998b, spots: 1.5, spotC: 0xc34988}),
    kelpfin: F({back: 0x8d4526, side: 0xfb902f, belly: 0xfaaf5c, fin: 0xc56d25, h: 0.38, w: 0.13, tail: "fork"}),
    trout: F({back: 0x297537, side: 0x9bb675, belly: 0xeab798, fin: 0x297537, h: 0.24, w: 0.12, tail: "trunc", lateral: 0x561c27, spots: 2, spotC: 0x003227}),
    redrun: F({back: 0x4a051e, side: 0xd12d41, belly: 0xfa3a64, fin: 0x4a051e, head: 0x92a3b2, h: 0.25, w: 0.12, tail: "trunc"}),
    glider: F({back: 0x15578a, side: 0x1096c3, belly: 0xfffefe, fin: 0x15578a, wingC: 0x74effe, h: 0.18, w: 0.12, tail: "deep", wings: true}),
    "m9/bloatfin": F({back: 0xc56d25, side: 0xfaaf5c, belly: 0xeab798, fin: 0xc56d25, h: 0.62, w: 0.48, tail: "round", spots: 2, spotC: 0x8d4526, peak: 0.45}),
    "m9/emberback": F({back: 0x161612, side: 0x2c2c25, belly: 0x434137, fin: 0x2e0d06, h: 0.3, w: 0.14, tail: "trunc", spots: 2.5, spotC: 0xfe6420}, {glow: 0xfe6420}),
    "m9/lanternjaw": F({back: 0x2b1f35, side: 0x3e3551, belly: 0x69608b, fin: 0x2b1f35, h: 0.42, w: 0.22, tail: "round", eyeRing: 0xd2f8ff, lure: true, peak: 0.25}, {lure: true}),
    "m9/glassback": F({back: 0x5d778d, side: 0x92a3b2, belly: 0xd2f8ff, fin: 0xbfcad7, h: 0.28, w: 0.1, tail: "fork"}),
    "m9/sunplate": F({back: 0xc56d25, side: 0xfb902f, belly: 0xfaaf5c, fin: 0x1096c3, h: 0.72, w: 0.09, tail: "trunc", peak: 0.45}),
    anthias: F({back: 0xdd576d, side: 0xfb902f, belly: 0xfaaf5c, fin: 0xfb902f, tailC: 0xdd576d, h: 0.3, w: 0.1, tail: "lunate", lateral: 0xfaaf5c}),
    damsel: F({back: 0x161612, side: 0x2c2c25, belly: 0x434137, fin: 0x161612, h: 0.48, w: 0.12, tail: "fork", spots: 0.6, spotC: 0xfffefe, peak: 0.38}),
    flounder: {t: "flat", p: {top: 0xcfaa81, under: 0xeab798, spot: 0xab8465}},
    crab: {t: "crab", p: {top: 0xd4462b, under: 0xe69d69, leg: 0xe69d69, claw: 0x841c1d}},
    "m7/shore_crab": {t: "crab", p: {top: 0xa61e1f, under: 0xd4462b, leg: 0xfe6420, claw: 0x4a051e}},
    ventcrab: {t: "crab", p: {top: 0xbfcad7, under: 0x92a3b2, leg: 0xfffefe, claw: 0x92a3b2}},
    fiddler: {t: "crab", p: {top: 0xc56d25, under: 0xfb902f, leg: 0xfaaf5c, claw: 0xfaaf5c, fiddler: true}, parts: "fiddler"},
    lobster: {t: "lobster", p: {top: 0xa8633e, under: 0xe69d69, band: 0x733126, claw: 0x733126, ant: 0x2e0d06}},
    crayfish: {t: "lobster", p: {top: 0x725244, under: 0xab8465, band: 0x41322c, claw: 0x2b1f35, ant: 0x41322c}},
    hermit: {t: "hermit", p: {shell: 0xcfaa81, shell2: 0x725244, body: 0xd12d41, claw: 0xa61e1f}},
    starfish: {t: "star", p: {top: 0xfb902f, edge: 0xc56d25, dot: 0xfaaf5c}},
    urchin: {t: "urchin", p: {body: 0x4f1d83, body2: 0x230c44, spine: 0x7d2dbd}},
    cucumber: {t: "cucumber", p: {top: 0xa8633e, under: 0xe69d69, bump: 0x733126}},
    abalone: {t: "shell", p: {cup: true, top: 0x725244, edge: 0xab8465, dark: 0x2e0d06, inner: 0x84e3b5}},
    rivermussel: {t: "shell", p: {top: 0x41322c, edge: 0x725244, inner: 0xab8465}},
    ventmussel: {t: "shell", p: {top: 0x1f3b52, edge: 0x3e586e, inner: 0x84e3b5}},
    shrimp: {t: "shrimp", p: {top: 0xe69d69, band: 0xd4462b, ant: 0x841c1d}, sway: "fish"},
    ventshrimp: {t: "shrimp", p: {top: 0xbfcad7, band: 0x92a3b2, ant: 0xfffefe}, sway: "fish"},
    krill: {t: "shrimp", p: {top: 0xfb902f, band: 0xc56d25, ant: 0x8d4526}, sway: "fish"},
    squid: {t: "squid", p: {body: 0xc34988, spot: 0x561c27, fin: 0xdd576d, arm: 0xeab798}, sway: "fish"},
    jelly: {t: "jelly", p: {bell: 0x9e3ba2, bell2: 0xdd576d, rim: 0xc1f1d8, tent: 0x84e3b5, tent2: 0x53998b, arm: 0xc1f1d8}},
    octopus: {t: "octo", p: {body: 0xd4462b, spot: 0x841c1d, eye: 0xeab798, sucker: 0xe69d69}},
    turtle: {t: "turtle", p: {shell: 0x725244, scute: 0xab8465, under: 0xcfaa81, skin: 0x297537}},
    seal: {t: "seal", p: {back: 0x3e586e, side: 0x5d778d, belly: 0x92a3b2, spot: 0x1f3b52}, sway: "fish"},
    otter: {t: "otter", p: {back: 0x522b2f, belly: 0xcfaa81, back2up: true}},
    riverotter: {t: "otter", p: {back: 0x522b2f, belly: 0x7d4636}, sway: "fish"},
    beaver: {t: "beaver", p: {back: 0x7d4636, belly: 0x522b2f, tail: 0x2f1525, tail2: 0x3e3551, teeth: 0xfb902f}},
    beaver_branch: {t: "beaver", p: {back: 0x7d4636, belly: 0x522b2f, tail: 0x2f1525, tail2: 0x3e3551, teeth: 0xfb902f, stick: 0xab8465, leaf: 0x559642}},
    frog: {t: "frog", p: {top: 0x559642, under: 0xc1f1d8, leg: 0x9bb675, spot: 0x05582d}},
    "m7/bog_toad": {t: "frog", p: {top: 0x297537, under: 0x9bb675, leg: 0x05582d, spot: 0x84e3b5}, glow: 0x96ffd2},
    "m7/mire_lurker": {t: "lurker", p: {top: 0x297537, under: 0x559642, spot: 0x003227, eye: 0xfffefe}},
    dragonfly: {t: "dfly", p: {body: 0x53998b, band: 0x0c2f44, eye: 0x74effe}},
    dabbler_drake: {t: "bird", p: {back: 0x434137, side: 0xbfcad7, belly: 0xa09f91, breast: 0xab8465, flankC: 0xbfcad7, tailC: 0x2c2c25, curl: 0x2c2c25, headC: 0x297537, ring: 0xfffefe, neckC: 0x297537, billC: 0xfaaf5c, wingC: 0x434137, spec: 0x1096c3, legC: 0xfb902f}},
    dabbler_hen: {t: "bird", p: {back: 0x725244, side: 0xab8465, belly: 0xcfaa81, tailC: 0x725244, headC: 0xab8465, neckC: 0xab8465, cheek: 0x2e0d06, billC: 0xfb902f, wingC: 0x725244, spec: 0x1096c3, legC: 0xfb902f}},
    duckling: {t: "bird", p: {back: 0xc56d25, side: 0xfaaf5c, belly: 0xfffefe, tailC: 0xc56d25, headC: 0xfaaf5c, neckC: 0xfaaf5c, cheek: 0x725244, billC: 0x725244, wingC: 0xc56d25, legC: 0xc56d25, span: 0.12, chord: 0.1, headR: 0.13, neck: 0.08, bill: 0.07}},
    diver: {t: "bird", p: {back: 0x2c2c25, side: 0xbfcad7, belly: 0xfffefe, flankC: 0xfffefe, tailC: 0x161612, headC: 0x434137, neckC: 0x2c2c25, billC: 0x92a3b2, wingC: 0x2c2c25, spec: 0xfffefe, legC: 0x92a3b2, crest: 0x161612}},
    "m7/heron": {t: "bird", heron: true, p: {back: 0x92a3b2, side: 0xbfcad7, belly: 0xfffefe, tailC: 0x5d778d, headC: 0xfffefe, neckC: 0xbfcad7, crest: 0x1f3b52, billC: 0xfaaf5c, bill: 0.24, billW: 0.035, billH: 0.035, neck: 0.42, neckLean: -0.15, headR: 0.07, wingC: 0x5d778d, tipC: 0x1f3b52, span: 0.6, chord: 0.26, legs: 0.48, legC: 0x5d778d, bodyY: 0.6, len: 0.8, tilt: 0.35}},
    kingfisher: {t: "bird", p: {back: 0x1096c3, side: 0x1096c3, belly: 0xfb902f, tailC: 0x15578a, headC: 0x1096c3, neckC: 0x1096c3, cheek: 0xfb902f, billC: 0x191954, bill: 0.2, billW: 0.035, headR: 0.12, neck: 0.06, wingC: 0x15578a, tipC: 0x191954, span: 0.3, legC: 0xd12d41}},
    "m7/gull": {t: "bird", p: {back: 0x92a3b2, side: 0xfffefe, belly: 0xfffefe, tailC: 0xfffefe, headC: 0xfffefe, neckC: 0xfffefe, billC: 0xfb902f, wingC: 0x92a3b2, tipC: 0x1f3b52, span: 0.7, chord: 0.2, legC: 0xfb902f}},
  };
  SPEC["game/fish-eel"].scale = 1; SPEC["game/fish-serpent"].scale = 1;

  // ---- building, cached per species (one unit long; the group scales it)
  const BUILT = {};
  function built(id) {
    if (BUILT[id]) return BUILT[id];
    const S = SPEC[id] || SPEC["game/fish-minnow"], p = S.p || S.f, parts = [];
    let geo;
    switch (S.t) {
      case "fish": geo = fishGeo(p); if (S.lure) parts.push({name: "lure", geo: (() => { const K = Kit(); K.add(bx(0.06, 0.06, 0.06), 0x74effe); return K.build(); })(), at: [0.58, p.h / 2 + 0.11, 0], glowTip: true}); break;
      case "ray": geo = rayGeo(p); parts.push({name: "wingR", geo: rayWing(p, 1), at: [0, 0, 0.1]}, {name: "wingL", geo: rayWing(p, -1), at: [0, 0, -0.1]}); break;
      case "flat": geo = flatfishGeo(p); break;
      case "crab": geo = crabGeo(p); if (p.fiddler) parts.push({name: "claw", geo: (() => { const K = Kit(); K.add(bx(0.36, 0.15, 0.13).translate(0.16, 0, 0), p.claw); K.add(bx(0.1, 0.08, 0.08), p.top); return K.build(); })(), at: [0.26, 0.18, 0.24]}); break;
      case "lobster": geo = lobsterGeo(p); break;
      case "hermit": geo = hermitGeo(p); break;
      case "star": geo = starGeo(p); break;
      case "urchin": geo = urchinGeo(p); break;
      case "cucumber": geo = cucumberGeo(p); break;
      case "shell": geo = shellGeo(p); break;
      case "shrimp": geo = shrimpGeo(p); break;
      case "squid": geo = squidGeo(p); break;
      case "jelly": geo = jellyGeo(p); parts.push({name: "bell", geo: jellyBell(p), at: [0, 0, 0]}); break;
      case "octo": geo = octoGeo(p); for (let i = 0; i < 8; i++) parts.push({name: "arm" + i, geo: octoArm(p), at: [0.05, 0.08, 0], yaw: i * Math.PI / 4 + Math.PI / 8}); break;
      case "turtle": geo = turtleGeo(p); for (const [n, x, s, fr] of [["fr", 0.18, 1, 1], ["fl", 0.18, -1, 1], ["br", -0.22, 1, 0], ["bl", -0.22, -1, 0]]) parts.push({name: n, geo: turtleFlipper(p, fr).scale(1, 1, s), at: [x, 0.0, s * 0.22]}); break;
      case "seal": geo = sealGeo(p); break;
      case "otter": geo = otterGeo(p); break;
      case "beaver": geo = beaverGeo(p); break;
      case "frog": geo = frogGeo(p); break;
      case "lurker": geo = lurkerGeo(p); break;
      case "dfly": geo = dflyGeo(p); for (const [n, x, s] of [["w1", 0.12, 1], ["w2", 0.12, -1], ["w3", 0.02, 1], ["w4", 0.02, -1]]) parts.push({name: n, geo: dflyWing(s), at: [x, 0.05, 0]}); break;
      case "bird": {
        geo = birdBody(p); const L = p.len || 1, by = p.bodyY ?? 0.12;
        parts.push({name: "head", geo: birdHead(p), at: [0.28 * L, by + 0.06, 0]});
        parts.push({name: "wingR", geo: birdWing(p, 1), at: [0.12 * L, by + 0.1, 0.12]}, {name: "wingL", geo: birdWing(p, -1), at: [0.12 * L, by + 0.1, -0.12]});
        if (!p.legs) parts.push({name: "footR", geo: footGeo(p), at: [-0.04, by - 0.05, 0.06]}, {name: "footL", geo: footGeo(p), at: [-0.04, by - 0.05, -0.06]});
        break;
      }
      default: geo = fishGeo(SPEC["game/fish-minnow"].f);
    }
    return (BUILT[id] = {S, geo, parts});
  }
  // the whole creature as one geometry in its rest pose, for an InstancedMesh
  const STATIC = {};
  function geo(id) {
    if (STATIC[id]) return STATIC[id];
    const B = built(id); if (!B.parts.length) return (STATIC[id] = B.geo);
    const K = Kit(), add = (g, M) => { const gg = g.clone(); gg.applyMatrix4(M); const a = gg.attributes.position, c = gg.attributes.color; const G = new T3.BufferGeometry(); G.setAttribute("position", a); G.setAttribute("color", c); return G; };
    const geos = [B.geo, ...B.parts.map(pt => add(pt.geo, new T3.Matrix4().makeTranslation(...pt.at)))];
    const P = [], C = []; for (const g of geos) { P.push(...g.attributes.position.array); C.push(...g.attributes.color.array); }
    const G = new T3.BufferGeometry(); G.setAttribute("position", new T3.Float32BufferAttribute(P, 3)); G.setAttribute("color", new T3.Float32BufferAttribute(C, 3)); G.computeVertexNormals(); G.computeBoundingSphere();
    return (STATIC[id] = G);
  }
  // a material per world and species kind: vertex colours, flat shaded, both sides, the page's lighting; glowing
  // creatures carry an emissive the page's night hook brightens
  function mat(W, id, o = {}) {
    const S = (SPEC[id] || {}), sway = o.sway !== undefined ? o.sway : S.sway || null, glow = o.glow || S.glow || null, key = (sway || "-") + "|" + (glow || "-") + (o.own ? "|" + Math.random() : "");
    const cache = W._c3m || (W._c3m = {}); if (cache[key]) return cache[key];
    const m = W.lit(new T3.MeshStandardMaterial({vertexColors: true, flatShading: true, side: T3.DoubleSide, roughness: 0.85, metalness: 0, emissive: glow ? new T3.Color(glow) : 0x000000, emissiveIntensity: 0}), false, {sway});
    if (glow) W.glow.push(m);
    return (cache[key] = m);
  }
  let LURE = null;
  function make(W, id, size, o = {}) {
    const B = built(id), S = B.S, m = mat(W, id, o);
    const g = new T3.Group(), body = new T3.Mesh(B.geo, m); g.add(body);
    const parts = {};
    for (const pt of B.parts) {
      const piv = new T3.Group(); piv.position.set(...pt.at); if (pt.yaw) piv.rotation.y = pt.yaw;
      const pm = pt.glowTip ? (LURE || (LURE = new T3.MeshBasicMaterial({vertexColors: true}))) : m;
      piv.add(new T3.Mesh(pt.geo, pm)); g.add(piv); parts[pt.name] = piv;
    }
    g.scale.setScalar(size * (S.scale || 0.85));
    g.userData.c3 = {id, S, parts, pose: o.pose || "swim", body, mat: m, last: null, speed: 0, strike: 0, t0: Math.random() * 10};
    (o.parent || W.scene).add(g);
    return g;
  }
  // a creature's own motion: wings, feet, arms, bells. t is the scene's clock; v its speed (m/s) if known
  function animate(g, t) {
    const c = g.userData.c3; if (!c) return; const P = c.parts, ph = c.t0, S = c.S;
    switch (S.t) {
      case "bird": {
        const fly = c.pose === "fly" || (c.pose === "auto" && c.speed > 1.2), sleep = c.pose === "sleep";
        const f = Math.sin(t * (S.p.span > 0.5 ? 6 : 13) + ph);
        if (fly) { P.wingR.rotation.set(-f * 0.75 + 0.15, 0, 0); P.wingL.rotation.set(f * 0.75 - 0.15, 0, 0); }
        else { P.wingR.rotation.set(0.25, -1.42, 0); P.wingL.rotation.set(-0.25, 1.42, 0); }
        const H = P.head;
        if (sleep) H.rotation.set(0, 2.6, 0.9); else if (S.heron) H.rotation.set(0, 0, -c.strike * 1.2); else H.rotation.set(0, Math.sin(t * 0.7 + ph) * 0.25 * (fly ? 0 : 1), c.pose === "upend" ? -0.6 : 0);
        if (P.footR) { const pad = fly || sleep ? 0 : Math.sin(t * 5 + ph) * 0.6; P.footR.rotation.z = pad; P.footL.rotation.z = -pad; P.footR.visible = P.footL.visible = !fly; }
        break;
      }
      case "ray": { const f = Math.sin(t * 1.6 + ph) * 0.35; P.wingR.rotation.x = f; P.wingL.rotation.x = -f; break; }
      case "turtle": { const f = Math.sin(t * 1.4 + ph); P.fr.rotation.x = f * 0.5; P.fl.rotation.x = -f * 0.5; P.br.rotation.x = -f * 0.25; P.bl.rotation.x = f * 0.25; P.fr.rotation.y = P.fl.rotation.y = 0.2 * f; break; }
      case "jelly": { const k = 1 + Math.max(0, Math.sin(t * 1.8 + ph)) * 0.22; P.bell.scale.set(1 / Math.sqrt(k), k, 1 / Math.sqrt(k)); break; }
      case "octo": { for (let i = 0; i < 8; i++) { const a = P["arm" + i]; a.rotation.z = -0.25 + Math.sin(t * 2 + i * 0.9 + ph) * 0.18 * (c.pose === "jet" ? 0.2 : 1); a.rotation.x = Math.sin(t * 1.3 + i) * 0.15; } break; }
      case "dfly": { const f = Math.sin(t * 40 + ph) * 0.4; P.w1.rotation.x = f; P.w2.rotation.x = -f; P.w3.rotation.x = -f; P.w4.rotation.x = f; break; }
      case "crab": if (P.claw) P.claw.rotation.z = Math.max(0, Math.sin(t * 3 + ph)) * 0.9; break;
    }
  }
  // turn a creature along its heading, upright (it pitches with the heading, never rolls); keeps its speed for
  // the pose. Used through the page's faceAlong for anything that was a card turned toward the eye.
  const _x = new T3.Vector3(), _y = new T3.Vector3(), _z = new T3.Vector3(), _B = new T3.Matrix4(), UP = new T3.Vector3(0, 1, 0);
  function orient(g, pos, head, t, dt) {
    const c = g.userData.c3;
    if (c.last && dt > 0) { const v = c.last.distanceTo(pos) / dt; c.speed += (v - c.speed) * Math.min(1, dt * 4); }
    (c.last || (c.last = new T3.Vector3())).copy(pos);
    g.position.copy(pos);
    _x.copy(head); if (_x.lengthSq() < 1e-10) _x.set(1, 0, 0); _x.normalize();
    _y.copy(UP).addScaledVector(_x, -_x.dot(UP)); if (_y.lengthSq() < 1e-6) _y.set(-_x.y, _x.x, 0); _y.normalize();
    _z.crossVectors(_x, _y); _B.makeBasis(_x, _y, _z); g.quaternion.setFromRotationMatrix(_B);
    animate(g, t);
  }
  root.C3 = {make, geo, mat, animate, orient, Kit, lathe, ell, bx, cyl, shape, SPEC, lin, hh};
})(window);
