// character.js: the CHARACTER tab of docs/mockups/multiplayer.html (openspec/changes/player-character,
// task 1.1): the player's own character screen, a tab of the Options menu so it can be changed at any
// time (owner, 2026-11-27), with every choice on screen, and the same figure seen in the third-person views
// of a rowing boat, a hand cart, a horse, a small car and a small plane, by day and lit at night.
//
// - The figure is M5's townsperson (figure-kit.js, FK.friend), the figure the game draws for its
//   NPCs (owner, 2026-11-27: "humans should be consistent with the NPCs in the game"). No other body
//   is built: every choice on the screen is a field of FK's spec, and every colour a swatch from the
//   peoples' palettes (peoples.js, copied from npcs.html), across every people (design 3).
// - Random rolls a look as FK.genSpec rolls a townsperson, with a people's hat as npcs.html's
//   townSpec picks it (look::generate in the game).
// - The craft are shaped, smooth-shaded forms lofted along curves, textured with the game's own
//   settlement set (plank, boards, timber, iron, hide, rope, linen, still water) and the herd's horse
//   hide; the car's paint and the tyres' rubber are 32 px textures painted here in the same palette.
// - The seat poses: each craft names a seat and where the hands go; the arms are aimed at those
//   points from the shoulders, and the oars, shafts, reins, wheel and stick are then laid to the
//   hands the arms reach, so the hands are on the controls for every build and height.
// - One 3D view, made when the tab first opens and stopped when it is hidden. Nothing turns on its
//   own: drag (or the arrow keys) turns the view about the figure, the wheel or a pinch zooms.
"use strict";
(function () {
  const $ = id => document.getElementById(id);
  const v = (x, y, z) => new THREE.Vector3(x, y, z);
  const lin = hex => new THREE.Color(hex).convertSRGBToLinear();
  const clamp = (x, a, b) => Math.max(a, Math.min(b, x));
  const W = PBD.W, R = PBD.R, ROW = PBD.ROW;
  const { PEOPLES, ROLE_HAT, SHOES } = window.FK_PEOPLES;

  // ================================================================ the look and its choices
  const uniq = a => [...new Set(a)];
  const luma = c => 0.3 * (c >> 16 & 255) + 0.59 * (c >> 8 & 255) + 0.11 * (c & 255);
  const every = k => uniq(Object.values(PEOPLES).flatMap(p => p[k]));
  const byLight = a => a.slice().sort((x, y) => luma(y) - luma(x));
  const PAL = {
    skin: byLight(every("skin")), hair: byLight(every("hair")), cloth: every("cloth"), shoe: SHOES,
    hat: uniq([...every("cloth"), ...Object.values(ROLE_HAT).map(h => h[1]).filter(Boolean)]),
  };
  const OPT = {
    build: [["round", "Round"], ["pear", "Pear"], ["tall", "Tall"], ["stout", "Stout"]],
    hair: [["crop", "Short"], ["bun", "Bun"], ["braid", "Braid"], ["bald", "Bald"]],
    beard: [["none", "None"], ["goatee", "Goatee"], ["full", "Full"]],
    hat: [["none", "None"], ["brim", "Brim hat"], ["cap", "Cap"], ["hood", "Hood"], ["turban", "Wrap"], ["helmet", "Helmet"], ["headset", "Headset"]],
    mood: [["happy", "Happy"], ["neutral", "Calm"], ["thinking", "Thinking"], ["sleepy", "Sleepy"]],
  };
  // genSpec's height range for each build
  const HRANGE = { tall: [1.6, 1.7], stout: [1.45, 1.53], round: [1.48, 1.64], pear: [1.48, 1.64] };
  const HAT_C = { brim: 0xd8b868, helmet: 0x8a9098, headset: 0x606878 };

  function roll(seed) {
    const keys = Object.keys(PEOPLES), rn = (() => { let s = (seed >>> 0) || 1; return () => { s ^= s << 13; s >>>= 0; s ^= s >>> 17; s ^= s << 5; s >>>= 0; return s / 4294967296; }; })();
    const P = PEOPLES[keys[Math.floor(rn() * keys.length) % keys.length]], S = FK.genSpec(seed, 0, P);
    const H = ROLE_HAT[P.hat[Math.floor(rn() * P.hat.length) % P.hat.length]];
    return {
      build: S.build, h: Math.round(S.h * 100) / 100, head: S.head, eye: S.eye, gap: S.gap, nose: S.nose, ears: S.ears,
      skin: S.skin, hairStyle: S.hair === "hat" ? "crop" : S.hair, hairC: S.hairC, beard: S.beard,
      shirt: S.shirt, pants: S.pants, shoe: S.shoe,
      hat: H ? H[0] : "none", hatC: H && H[1] ? H[1] : S.hatC,
      mood: OPT.mood.some(m => m[0] === S.mood) ? S.mood : "happy",
    };
  }
  // the look as FK's spec: a hat replaces the hair (FK draws the hair's back under it), and a hood or
  // a headset hides the ears, as townSpec does
  const toSpec = L => ({
    build: L.build, h: L.h, head: L.head, eye: L.eye, gap: L.gap, nose: L.nose,
    ears: L.ears && L.hat !== "hood" && L.hat !== "headset",
    hair: L.hat !== "none" ? "hat" : L.hairStyle, hatKind: L.hat, hatC: L.hatC, beard: L.beard,
    skin: L.skin, hairC: L.hairC, shirt: L.shirt, pants: L.pants, shoe: L.shoe, mood: L.mood,
  });
  let LOOK = roll(20261127);

  // ================================================================ the character screen (the UI kit)
  const panelEl = $("charPanel");
  const hexs = c => "#" + c.toString(16).padStart(6, "0");
  function small(label, aria, on) { const b = btn(label, on); b.setAttribute("aria-label", aria); return b; }
  function stepRow(label, key, opts) {
    const i = Math.max(0, opts.findIndex(o => o[0] === LOOK[key]));
    const go = d => { set({ [key]: opts[(i + d + opts.length) % opts.length][0] }); };
    return h("div", { class: "k-row" }, P(label), h("div", { class: "k-value", "aria-live": "polite" }, opts[i][1]),
      h("span", { class: "k-step" }, small("<", "Previous " + label.toLowerCase(), () => go(-1)), small(">", "Next " + label.toLowerCase(), () => go(1))));
  }
  function heightRow() {
    const [a, b] = HRANGE[LOOK.build], go = d => set({ h: Math.round(clamp(LOOK.h + d, a, b) * 100) / 100 });
    return h("div", { class: "k-row" }, P("Height"), h("div", { class: "k-value", "aria-live": "polite" }, LOOK.h.toFixed(2) + " m"),
      h("span", { class: "k-step" }, small("<", "Shorter", () => go(-0.01)), small(">", "Taller", () => go(0.01))));
  }
  function swatchRow(label, key, list) {
    return h("div", { class: "k-row sw" }, P(label), h("div", { class: "k-sws", role: "radiogroup", "aria-label": label },
      ...list.map(c => h("button", { class: "k-sw", type: "button", role: "radio", "aria-checked": String(c === LOOK[key]), "aria-label": label + " " + hexs(c), title: hexs(c), style: "background:" + hexs(c),
        onclick: () => set({ [key]: c }) }))));
  }
  // the Options menu (the settings screen, renamed on the title menu) with its tabs; CHARACTER is
  // the new one, and every choice of it is on screen at once
  const OPT_TABS = ["Picture", "Sound", "Input", "Movement", "Weather", "World", "Character"];
  function renderPanel() {
    const tab = label => { const on = label === "Character", b = btn(label, on ? () => {} : null, { cls: on ? "focus" : "" }); b.setAttribute("role", "tab"); b.setAttribute("aria-selected", String(on)); return b; };
    const hatC = swatchRow("Hat colour", "hatC", PAL.hat);
    if (LOOK.hat === "none") { hatC.classList.add("off"); hatC.querySelectorAll("button").forEach(b => { b.disabled = true; }); }
    const panel = frame(360, ...lintel("Options", () => {}),
      h("div", { class: "k-otabs", role: "tablist", "aria-label": "Options" }, ...OPT_TABS.map(tab)),
      h("div", { class: "k-cols" },
        h("div", { class: "k-rows" }, stepRow("Build", "build", OPT.build), heightRow(), swatchRow("Skin", "skin", PAL.skin),
          stepRow("Hair", "hairStyle", OPT.hair), swatchRow("Hair colour", "hairC", PAL.hair), stepRow("Beard", "beard", OPT.beard), stepRow("Face", "mood", OPT.mood)),
        h("div", { class: "k-rows" }, swatchRow("Shirt", "shirt", PAL.cloth), swatchRow("Trousers", "pants", PAL.cloth), swatchRow("Shoes", "shoe", PAL.shoe),
          stepRow("Hat", "hat", OPT.hat), hatC)),
      h("div", { class: "k-foot" }, btn("Random", () => { LOOK = roll(1 + Math.floor(Math.random() * 2e9)); rebuild(); renderPanel(); }), btn("Back", () => {}, { cls: "focus" })));
    const keep = document.activeElement && panelEl.contains(document.activeElement) ? document.activeElement.getAttribute("aria-label") : null;
    panelEl.replaceChildren(panel);
    if (keep) { const el = [...panelEl.querySelectorAll("[aria-label]")].find(e => e.getAttribute("aria-label") === keep); if (el) el.focus({ preventScroll: true }); }
  }
  function set(ch) {
    Object.assign(LOOK, ch);
    if (ch.build) { const [a, b] = HRANGE[LOOK.build]; LOOK.h = Math.round(clamp(LOOK.h, a, b) * 100) / 100; }
    if (ch.hat && ch.hat !== "none" && HAT_C[ch.hat] && !PAL.hat.includes(LOOK.hatC)) LOOK.hatC = HAT_C[ch.hat];
    rebuild(); renderPanel();
  }

  // ================================================================ textures and materials
  function prep(t) { t.encoding = THREE.sRGBEncoding; t.wrapS = t.wrapT = THREE.RepeatWrapping; t.magFilter = t.minFilter = THREE.NearestFilter; t.generateMipmaps = false; return t; }
  const game = n => PBD.game(n);
  // a 32 px tile cut from a sheet (the herd's hides: a row a creature, the horse's row 0)
  const SHEETS = {};
  function tileTex(file, tx, ty) {
    const cv = document.createElement("canvas"); cv.width = cv.height = 32; const t = prep(new THREE.CanvasTexture(cv));
    const draw = img => { cv.getContext("2d").drawImage(img, tx * 32, ty * 32, 32, 32, 0, 0, 32, 32); t.needsUpdate = true; };
    let s = SHEETS[file];
    if (!s) { s = SHEETS[file] = { img: new Image(), wait: [] }; s.img.onload = () => { s.ok = true; for (const f of s.wait.splice(0)) f(s.img); }; s.img.src = PBD.base + "game/" + file; }
    if (s.ok) draw(s.img); else s.wait.push(draw);
    return t;
  }
  // 32 px textures painted here, in the ground's palette (the car's paint, the tyres' rubber)
  function painted(seed, fn) {
    const cv = document.createElement("canvas"); cv.width = cv.height = 32; const g = cv.getContext("2d"), img = g.createImageData(32, 32);
    let s = seed; const rnd = () => ((s = (s * 16807) % 2147483647) / 2147483647);
    const px = (x, y, c, f = 1) => { const i = (((y & 31) * 32) + (x & 31)) * 4; img.data[i] = clamp((c >> 16 & 255) * f, 0, 255); img.data[i + 1] = clamp((c >> 8 & 255) * f, 0, 255); img.data[i + 2] = clamp((c & 255) * f, 0, 255); img.data[i + 3] = 255; };
    fn(px, rnd); g.putImageData(img, 0, 0); return prep(new THREE.CanvasTexture(cv));
  }
  const PAINT = {
    // dull green coach paint, a little worn, a darker panel seam every half metre
    paint: painted(11, (px, rnd) => { for (let y = 0; y < 32; y++) for (let x = 0; x < 32; x++) px(x, y, 0x3e5a3a, (x === 0 || x === 16 ? 0.72 : 1) * (1 + (rnd() - 0.5) * 0.07)); for (let i = 0; i < 14; i++) px(rnd() * 32 | 0, rnd() * 32 | 0, 0x4e6a48); for (let i = 0; i < 6; i++) px(rnd() * 32 | 0, rnd() * 32 | 0, 0x6a6a58); }),
    rubber: painted(23, (px, rnd) => { for (let y = 0; y < 32; y++) for (let x = 0; x < 32; x++) px(x, y, 0x262422, (y % 4 === 0 ? 0.75 : 1) * (1 + (rnd() - 0.5) * 0.12)); }),
  };
  const MT = {};
  // every craft material is double sided, so a lofted shell reads the same from either side
  function mat(key, map, tint = 0xffffff, extra = {}) { return MT[key] || (MT[key] = PBD.litMaterial(map, lin(tint), { side: THREE.DoubleSide, ...extra })); }
  const M = {
    plank: () => mat("plank", game("plank")), boards: () => mat("boards", game("boards")), timber: () => mat("timber", game("timber")),
    iron: () => mat("iron", game("iron")), hide: () => mat("hide", game("hide")), rope: () => mat("rope", game("rope")),
    sack: () => mat("sack", game("linen"), 0xd8c0a0), fabric: () => mat("fabric", game("linen"), 0xa85a44), brass: () => mat("brass", game("iron"), 0xe0b060),
    paint: () => mat("paint", PAINT.paint), rubber: () => mat("rubber", PAINT.rubber),
    coat: () => mat("coat", tileTex("herd_hide.png", 0, 0)), legs: () => mat("legs", tileTex("herd_hide.png", 1, 0)), head: () => mat("headh", tileTex("herd_hide.png", 2, 0)),
    mane: () => mat("mane", tileTex("herd_hide.png", 5, 0)), hoof: () => mat("hoof", tileTex("herd_hide.png", 4, 0), 0xb0a090),
    water: () => MT.water || (MT.water = PBD.litMaterial(game("still_water"), lin(0x86a4b4), { roughness: 0.32 })),
    glass: () => MT.glass || (MT.glass = new THREE.MeshStandardMaterial({ color: 0x9fb8c8, roughness: 0.1, transparent: true, opacity: 0.22, side: THREE.DoubleSide, depthWrite: false })),
  };
  const glowMat = new THREE.MeshBasicMaterial({ color: 0x3a3832 });
  const navR = new THREE.MeshBasicMaterial({ color: 0x501810 }), navG = new THREE.MeshBasicMaterial({ color: 0x184a20 });

  // ================================================================ smooth lofted forms
  // rings: Vector3 rings of equal length, one after another. Smooth normals across every ring
  // (closed: the seam averaged), UVs in metres / rep (u round the ring, v along, or swapped).
  function gridGeo(rings, o = {}) {
    const n = rings[0].length, m = rings.length, cl = o.closed !== false, cols = cl ? n + 1 : n, rep = o.rep || 1, pos = [], uv = [], idx = [];
    const along = [0];
    for (let j = 1; j < m; j++) { let d = 0; for (let i = 0; i < n; i++) d += rings[j][i].distanceTo(rings[j - 1][i]); along.push(along[j - 1] + d / n); }
    for (let j = 0; j < m; j++) {
      let u = 0;
      for (let i = 0; i < cols; i++) { const p = rings[j][i % n]; if (i) u += p.distanceTo(rings[j][(i - 1) % n]); pos.push(p.x, p.y, p.z); if (o.uAlong) uv.push(along[j] / rep, u / rep); else uv.push(u / rep, along[j] / rep); }
    }
    for (let j = 0; j < m - 1; j++) for (let i = 0; i < cols - 1; i++) { const a = j * cols + i, b = a + 1, c = a + cols, d = c + 1; idx.push(a, c, b, b, c, d); }
    const cap = j => {
      const Rg = rings[j], c = Rg.reduce((s, p) => s.add(p), v(0, 0, 0)).multiplyScalar(1 / n), base = pos.length / 3;
      pos.push(c.x, c.y, c.z); uv.push(0, 0);
      for (const p of Rg) { pos.push(p.x, p.y, p.z); uv.push((p.x - c.x + p.z - c.z) / rep, (p.y - c.y) / rep); }
      for (let i = 0; i < n; i++) idx.push(base, base + 1 + i, base + 1 + (i + 1) % n);
    };
    if (o.cap0) cap(0); if (o.cap1) cap(m - 1);
    const g = new THREE.BufferGeometry();
    g.setAttribute("position", new THREE.Float32BufferAttribute(pos, 3)); g.setAttribute("uv", new THREE.Float32BufferAttribute(uv, 2)); g.setIndex(idx);
    g.computeVertexNormals();
    const N = g.attributes.normal, avg = (a, b) => { const x = N.getX(a) + N.getX(b), y = N.getY(a) + N.getY(b), z = N.getZ(a) + N.getZ(b), l = Math.hypot(x, y, z) || 1; N.setXYZ(a, x / l, y / l, z / l); N.setXYZ(b, x / l, y / l, z / l); };
    if (cl) for (let j = 0; j < m; j++) avg(j * cols, j * cols + n);
    if (o.closedPath) for (let i = 0; i < cols; i++) avg(i, (m - 1) * cols + i);
    return g;
  }
  // a tube along points P with half-sizes Rr (x across `side`, y the other way), elliptical rings
  function tubeGeo(P, Rr, o = {}) {
    const n = P.length, seg = o.seg || 12, loop = !!o.loop, side = o.side;
    const T = P.map((p, k) => { const a = loop ? P[(k - 1 + n) % n] : P[Math.max(0, k - 1)], b = loop ? P[(k + 1) % n] : P[Math.min(n - 1, k + 1)]; return b.clone().sub(a).normalize(); });
    const rings = []; let N = null;
    for (let k = 0; k < n; k++) {
      const t = T[k]; let nn;
      if (side) nn = side.clone().sub(t.clone().multiplyScalar(side.dot(t)));
      else if (!N) { nn = Math.abs(t.y) < 0.9 ? v(0, 1, 0) : v(1, 0, 0); nn.sub(t.clone().multiplyScalar(nn.dot(t))); }
      else nn = N.clone().sub(t.clone().multiplyScalar(N.dot(t)));
      nn.normalize(); N = nn; const B = t.clone().cross(nn), ring = [];
      for (let i = 0; i < seg; i++) { const a = i / seg * Math.PI * 2 + (o.phase || 0); ring.push(P[k].clone().addScaledVector(nn, Rr[k].x * Math.cos(a)).addScaledVector(B, Rr[k].y * Math.sin(a))); }
      rings.push(ring);
    }
    if (loop) rings.push(rings[0]);
    return gridGeo(rings, { closedPath: loop, cap0: !loop && o.cap !== false, cap1: !loop && o.cap !== false, rep: o.rep || 1, uAlong: o.uAlong });
  }
  // sweep: control points [x, y, z, rx, ry?] through a Catmull-Rom curve, so a form tapers smoothly
  function sweep(ctrl, o = {}) {
    const pts = ctrl.map(c => v(c[0], c[1], c[2])), rad = ctrl.map(c => v(c[3], c[4] ?? c[3], 0));
    if (ctrl.length < 3) return tubeGeo(pts, rad, o);
    const cp = new THREE.CatmullRomCurve3(pts, !!o.loop, "catmullrom", 0.5), cr = new THREE.CatmullRomCurve3(rad, !!o.loop, "catmullrom", 0.5);
    const n = o.samples || (ctrl.length - 1) * 5, P = [], Rr = [];
    for (let i = 0; i <= n; i++) { if (o.loop && i === n) break; P.push(cp.getPoint(i / n)); Rr.push(cr.getPoint(i / n)); }
    return tubeGeo(P, Rr, o);
  }
  // a rod from a to b, tapering r0 to r1
  const rodGeo = (a, b, r0, r1 = r0, seg = 8) => tubeGeo([a, b], [v(r0, r0, 0), v(r1, r1, 0)], { seg });
  // a ring (a rim, a tyre, a girth) round `axis` at `c`, radius r, the band rx across and ry deep
  function ringGeo(c, axis, r, rx, ry, n = 32, seg = 8) {
    const a = axis.clone().normalize(), u = Math.abs(a.y) < 0.9 ? v(0, 1, 0).cross(a).normalize() : v(1, 0, 0).cross(a).normalize(), w = a.clone().cross(u), P = [], Rr = [];
    for (let i = 0; i < n; i++) { const t = i / n * Math.PI * 2; P.push(c.clone().addScaledVector(u, r * Math.cos(t)).addScaledVector(w, r * Math.sin(t))); Rr.push(v(rx, ry, 0)); }
    return tubeGeo(P, Rr, { loop: true, side: a, seg });
  }
  // a board: a box with a small bevel (a board's edge is a real crease, so its normals stay hard)
  function slabGeo(w, hh, d, bev = 0.012, rep = 1) {
    const g = new PBD.Geo(), ring = (y, i) => { const a = w / 2 - i, b = d / 2 - i, k = Math.min(bev, a * 0.9, b * 0.9); return [[a, b - k], [a - k, b], [-a + k, b], [-a, b - k], [-a, -b + k], [-a + k, -b], [a - k, -b], [a, -b + k]].map(([x, z]) => [x, y, z]); };
    const Rg = [ring(-hh / 2, bev), ring(-hh / 2 + bev, 0), ring(hh / 2 - bev, 0), ring(hh / 2, bev)];
    for (let i = 0; i < 3; i++) for (let k = 0; k < 8; k++) { const a0 = Rg[i][k], a1 = Rg[i][(k + 1) % 8], b0 = Rg[i + 1][k], b1 = Rg[i + 1][(k + 1) % 8]; g.face([a0, a1, b1, b0], [(a0[0] + a1[0]) / 2, 0, (a0[2] + a1[2]) / 2]); }
    g.face(Rg[0], [0, -1, 0]); g.face(Rg[3], [0, 1, 0]);
    return g.build(rep);
  }
  function mk(geo, m, parent, x = 0, y = 0, z = 0, cast = true) { const o = new THREE.Mesh(geo, m); o.position.set(x, y, z); o.castShadow = cast; o.receiveShadow = true; parent.add(o); return o; }
  const slab = (parent, m, w, hh, d, x, y, z, ry = 0, rep = 1) => { const o = mk(slabGeo(w, hh, d, 0.012, rep), m, parent, x, y, z); o.rotation.y = ry; return o; };
  // a lantern: an iron cap and foot round a glass that glows at night; returns the glass's centre
  function lantern(parent, x, y, z, s = 1) {
    const g = new THREE.Group(); g.position.set(x, y, z); g.scale.setScalar(s); parent.add(g);
    mk(sweep([[0, 0.11, 0, 0.1], [0, 0.15, 0, 0.085], [0, 0.19, 0, 0.035], [0, 0.23, 0, 0.012]], { side: v(1, 0, 0) }), M.iron(), g);
    mk(sweep([[0, -0.15, 0, 0.03], [0, -0.14, 0, 0.095], [0, -0.11, 0, 0.1]], { side: v(1, 0, 0) }), M.iron(), g);
    mk(sweep([[0, -0.11, 0, 0.078], [0, 0, 0, 0.088], [0, 0.11, 0, 0.078]], { side: v(1, 0, 0), seg: 10 }), glowMat, g, 0, 0, 0, false);
    for (let k = 0; k < 4; k++) { const a = k * Math.PI / 2 + Math.PI / 4; mk(rodGeo(v(Math.cos(a) * 0.09, -0.11, Math.sin(a) * 0.09), v(Math.cos(a) * 0.09, 0.12, Math.sin(a) * 0.09), 0.009), M.iron(), g); }
    mk(ringGeo(v(0, 0.25, 0), v(1, 0, 0), 0.035, 0.008, 0.008, 12, 6), M.iron(), g);
    return g;
  }

  // ================================================================ scene
  const view = $("cview");
  const scene = new THREE.Scene(); scene.fog = new THREE.Fog(0x9fc6e2, 55, 170);
  const camera = new THREE.PerspectiveCamera(50, 1, 0.05, 500);
  const sun = new THREE.DirectionalLight(0xffffff, 1); sun.castShadow = true; sun.shadow.mapSize.set(2048, 2048);
  Object.assign(sun.shadow.camera, { left: -12, right: 12, top: 12, bottom: -12, near: 1, far: 160 }); sun.shadow.bias = -0.0004; sun.shadow.normalBias = 0.02;
  scene.add(sun, sun.target);
  const hemi = new THREE.HemisphereLight(0xffffff, 0x1a1a22, 0.2); scene.add(hemi);

  // ---- the ground: hex columns (2.833 m, 1 m layers), a path, a pond, the game's trees
  const C0 = -10, C1 = 10, R0 = -16, R1 = 4;
  const cx = (c, r) => W * (c + 0.5 * PBD.par(r)), cz = (c, r) => ROW * r;
  const corner = (c, r, k) => { const a = (60 * k - 30) * Math.PI / 180; return [cx(c, r) + R * Math.cos(a), cz(c, r) + R * Math.sin(a)]; };
  const hexPts = (c, r) => [0, 1, 2, 3, 4, 5].map(k => corner(c, r, k));
  const inMap = (c, r) => c >= C0 && c <= C1 && r >= R0 && r <= R1;
  const key = (c, r) => c + "," + r;
  const WATER_Y = -0.3;
  const TOP = new Map(), KIND = new Map();
  for (let r = R0; r <= R1; r++) for (let c = C0; c <= C1; c++) {
    const x = cx(c, r), z = cz(c, r);
    let hh = 0, kind = "grass";
    if (Math.abs(x) <= W * 0.55) kind = "path";
    else if (x < -6 && x > -27 && z < -7 && z > -33) { kind = "pond"; hh = -1; }
    else if (Math.abs(c) >= 8 && PBD.hash2(c * 5 + 3, r * 11 + 1) > 0.45) hh = 1;
    TOP.set(key(c, r), hh); KIND.set(key(c, r), kind);
  }
  const GS = new PBD.GeoSet(), WG = new PBD.Geo();
  for (let r = R0; r <= R1; r++) for (let c = C0; c <= C1; c++) {
    const hh = TOP.get(key(c, r)), kind = KIND.get(key(c, r)), pts = hexPts(c, r), x = cx(c, r), z = cz(c, r);
    GS.g(kind === "path" ? "a:path" : kind === "pond" ? "a:dirt" : "a:grass").face(pts.map(p => [p[0], hh, p[1]]), [0, 1, 0], PBD.cellUV(x, z));
    if (kind === "pond") WG.face(pts.map(p => [p[0], WATER_Y, p[1]]), [0, 1, 0], p => [p[0] / 2, p[2] / 2]);
    for (let d = 0; d < 6; d++) {
      const [c2, r2] = PBD.nb(c, r, d), h2 = inMap(c2, r2) ? TOP.get(key(c2, r2)) : hh - 3; if (h2 >= hh) continue;
      const A = corner(c, r, d), B = corner(c, r, d + 1), [nx, nz] = PBD.dvec(d);
      for (let y = hh; y > h2 + 1e-3; y -= 1) {
        const y0 = Math.max(h2, y - 1), sm = kind === "grass" && y === hh ? "a:grassside" : "a:dirt";
        GS.g(sm).face([[A[0], y0, A[1]], [B[0], y0, B[1]], [B[0], y, B[1]], [A[0], y, A[1]]], [nx, 0, nz], PBD.sideUV(A, B, y));
      }
    }
  }
  { const g = GS.g("a:grass"), S = 10;
    for (let x = -300; x < 300; x += S) for (let z = -330; z < 270; z += S) {
      if (x >= -30 && x + S <= 30 && z >= -40 && z + S <= 10) continue;
      g.face([[x, -0.03, z], [x + S, -0.03, z], [x + S, -0.03, z + S], [x, -0.03, z + S]], [0, 1, 0], p => [p[0] / 2.78, p[2] / 2.78]);
    } }
  // the places each view stands at
  const AT = { stand: v(0.6, 0, 0.4), cart: v(0, 0, -6.4), car: v(-0.05, 0, -14.2), horse: v(8, 0, -7.4), boat: v(-13, WATER_Y, -19.5), plane: v(4, 17, -22) };
  // lamp posts by the path and the field, the lantern hung toward the place it lights
  const LAMPS = [], GLOW = [];
  function lampPost(x, y, z, s) {
    const p = []; for (let k = 0; k < 6; k++) { const a = k * Math.PI / 3; p.push([x + 0.09 * Math.cos(a), z + 0.09 * Math.sin(a)]); }
    GS.prism("timber", "timber", p, y, y + 2.6); GS.box("timber", x + s * 0.25, y + 2.55, z, 0.5, 0.06, 0.06);
    GS.box("iron", x + s * 0.45, y + 2.07, z, 0.26, 0.05, 0.26); GS.box("iron", x + s * 0.45, y + 2.45, z, 0.26, 0.05, 0.26);
    const m = new THREE.Mesh(new THREE.BoxGeometry(0.2, 0.33, 0.2), glowMat); m.position.set(x + s * 0.45, y + 2.27, z); scene.add(m); GLOW.push(m);
    LAMPS.push({ x: x + s * 0.45, y: y + 2.2, z, R: 5.6, c: lin(0xffd27a), p: 1.6 });
  }
  const LAMPCELL = new Set();
  for (const [c, r, s] of [[1, 0, -1], [1, -2, -1], [-1, -6, 1], [2, -4, 1], [-1, 2, 1], [1, -10, -1], [-1, -14, 1]]) { LAMPCELL.add(key(c, r)); lampPost(cx(c, r), TOP.get(key(c, r)), cz(c, r), s); }
  for (let r = R0; r <= R1; r++) for (let c = C0; c <= C1; c++) {
    const k = key(c, r), x = cx(c, r), z = cz(c, r);
    if (KIND.get(k) !== "grass" || Math.abs(x) < W * 2.2 || LAMPCELL.has(k) || PBD.hash2(c * 3 + 11, r * 7 + 5) > 0.16) continue;
    if (Math.hypot(x - AT.horse.x, z - AT.horse.z) < 5 || (x < -4 && z < -4 && z > -36)) continue;
    PBD.tree(GS, hexPts(c, r), x, z, TOP.get(k), c * 97 + r * 13, "a:bark", "a:leaves");
  }
  PBD.addMeshes(GS, scene, { cast: k => k !== "a:grass" && k !== "a:path" && k !== "a:dirt" });
  { const w = new THREE.Mesh(WG.build(1), M.water()); w.receiveShadow = true; scene.add(w); }
  const stars = (() => { const n = 700, p = new Float32Array(n * 3); let s = 9; const rn = () => ((s = (s * 16807) % 2147483647) / 2147483647);
    for (let i = 0; i < n; i++) { const a = rn() * 6.283, e = 0.06 + rn() * 1.4; p[3 * i] = Math.cos(a) * Math.cos(e) * 320; p[3 * i + 1] = Math.sin(e) * 320; p[3 * i + 2] = Math.sin(a) * Math.cos(e) * 320; }
    const g = new THREE.BufferGeometry(); g.setAttribute("position", new THREE.BufferAttribute(p, 3));
    const m = new THREE.Points(g, new THREE.PointsMaterial({ color: 0xdfe6ff, size: 1.3, sizeAttenuation: false, fog: false, transparent: true, opacity: 0 })); scene.add(m); return m; })();

  // ================================================================ the craft (each faces +z)
  const CRAFT = {};
  function craft(name) { const g = new THREE.Group(), ctl = new THREE.Group(); g.add(ctl); g.position.copy(AT[name]); g.visible = false; scene.add(g); return (CRAFT[name] = { g, ctl, lights: [] }); }
  const lightAt = (C, obj, R, p, c = 0xffd27a) => C.lights.push({ obj, R, p, c: lin(c) });

  // ---- the rowing boat: a lofted clinker hull of planks, two thwarts, oarlocks, a stern lantern
  const BOAT = (() => {
    const C = craft("boat"), g = C.g, L = 3.4;
    const hb = t => Math.max(0.025, 0.62 * Math.pow(Math.sin(Math.PI * (0.17 + 0.83 * t)), 0.62));
    const ys = t => 0.5 + 0.1 * (2 * t - 1) ** 2 + 0.1 * t ** 4, yk = t => 0.05 + 0.3 * t ** 6, zt = t => -L / 2 + L * t;
    const sect = (t, ins) => { const b = hb(t) - ins, top = ys(t), bot = yk(t) + ins * 0.9, out = [];
      for (let i = 0; i <= 10; i++) { const a = Math.PI * i / 10, sx = Math.cos(a), sy = Math.sin(a); out.push(v(-b * Math.sign(sx) * Math.pow(Math.abs(sx), 0.8), top - (top - bot) * Math.pow(sy, 0.55), zt(t))); }
      return out; };
    const rings = []; for (let k = 0; k <= 28; k++) { const t = k / 28, o = sect(t, 0), i = sect(t, Math.min(0.035, hb(t) * 0.6)).reverse(); rings.push([...o, ...i]); }
    mk(gridGeo(rings, { cap0: true, cap1: true, rep: 2, uAlong: true }), M.plank(), g);
    // the rubbing strake: one rounded rail along each gunwale
    for (const s of [-1, 1]) { const P = [], Rr = []; for (let k = 1; k <= 26; k++) { const t = k / 27; P.push(v(s * (hb(t) + 0.01), ys(t) - 0.03, zt(t))); Rr.push(v(0.022, 0.03, 0)); } mk(tubeGeo(P, Rr, { seg: 8 }), M.timber(), g); }
    const seatT = 0.53, seatZ = zt(seatT), seatY = 0.38;
    slab(g, M.boards(), 2 * hb(seatT) - 0.06, 0.05, 0.26, 0, seatY - 0.025, seatZ, Math.PI / 2, 2);
    slab(g, M.boards(), 2 * hb(0.18) - 0.06, 0.05, 0.3, 0, 0.42, zt(0.18), Math.PI / 2, 2);
    slab(g, M.boards(), 0.9, 0.04, 1.6, 0, 0.11, -0.1, 0, 2);   // bottom boards, where the feet go
    const lockZ = seatZ - 0.3, lockT = (lockZ + L / 2) / L, lockX = hb(lockT) + 0.02, lockY = ys(lockT) + 0.04;
    for (const s of [-1, 1]) { mk(sweep([[s * lockX, lockY - 0.06, lockZ, 0.022], [s * lockX, lockY, lockZ, 0.02], [s * lockX, lockY + 0.03, lockZ, 0.03]], { side: v(1, 0, 0) }), M.iron(), g); mk(ringGeo(v(s * lockX, lockY + 0.04, lockZ), v(0, 0, 1), 0.04, 0.008, 0.008, 16, 6), M.iron(), g); }
    // a short post at the stern with the boat's lantern, toward the rower's face
    mk(rodGeo(v(0, 0.42, -1.38), v(0, 1.05, -1.42), 0.025, 0.02), M.timber(), g);
    mk(rodGeo(v(0, 1.04, -1.42), v(0, 1.04, -1.22), 0.016), M.timber(), g);
    const lan = lantern(g, 0, 0.86, -1.22, 0.85); lightAt(C, lan, 3.6, 1.5);
    return Object.assign(C, { seatZ, seatY, lockX, lockY, lockZ, keelY: 0.05 });
  })();
  CRAFT.boat.g.position.y = WATER_Y - 0.24;   // she floats with the water a little up her sides

  // a spoked wheel on axis x: a timber rim with an iron tyre, tapered spokes, an iron hub
  function spokedWheel(parent, x, y, z, r, n = 10) {
    const g = new THREE.Group(); g.position.set(x, y, z); parent.add(g);
    mk(ringGeo(v(0, 0, 0), v(1, 0, 0), r - 0.035, 0.035, 0.04, 36, 8), M.timber(), g);
    mk(ringGeo(v(0, 0, 0), v(1, 0, 0), r - 0.004, 0.04, 0.012, 36, 6), M.iron(), g);
    mk(sweep([[-0.09, 0, 0, 0.045], [-0.06, 0, 0, 0.07], [0.06, 0, 0, 0.07], [0.09, 0, 0, 0.045]], { side: v(0, 1, 0), seg: 12 }), M.iron(), g);
    for (let k = 0; k < n; k++) { const a = k / n * Math.PI * 2, d = v(0, Math.cos(a), Math.sin(a)); mk(rodGeo(d.clone().multiplyScalar(0.06), d.clone().multiplyScalar(r - 0.06), 0.022, 0.015, 6), M.timber(), g); }
    return g;
  }
  // ---- the hand cart: two wheels, a plank bed with board sides, a sack and a barrel
  const CART = (() => {
    const C = craft("cart"), g = C.g, r = 0.45;
    for (const s of [-1, 1]) spokedWheel(g, s * 0.62, r, -1.0, r);
    mk(sweep([[-0.7, r, -1.0, 0.03], [0.7, r, -1.0, 0.03]], { side: v(0, 1, 0) }), M.iron(), g);
    slab(g, M.plank(), 1.0, 0.06, 1.3, 0, 0.63, -1.1, Math.PI / 2, 2);
    for (const s of [-1, 1]) slab(g, M.boards(), 1.3, 0.26, 0.04, s * 0.5, 0.79, -1.1, Math.PI / 2, 2);
    slab(g, M.boards(), 1.04, 0.26, 0.04, 0, 0.79, -1.77, 0, 2); slab(g, M.boards(), 1.04, 0.2, 0.04, 0, 0.76, -0.43, 0, 2);
    // a sack: one soft lofted form, tied at the neck
    mk(sweep([[-0.05, 0.68, -1.42, 0.02, 0.02], [-0.05, 0.72, -1.38, 0.17, 0.12], [-0.05, 0.78, -1.2, 0.22, 0.17], [-0.05, 0.8, -1.02, 0.19, 0.15], [-0.05, 0.84, -0.92, 0.06, 0.05], [-0.05, 0.88, -0.86, 0.035, 0.035], [-0.05, 0.92, -0.8, 0.06, 0.05]], { side: v(1, 0, 0), seg: 14 }), M.sack(), g);
    mk(sweep([[0.27, 0.66, -1.5, 0.15], [0.27, 0.8, -1.5, 0.18], [0.27, 0.96, -1.5, 0.15]], { side: v(1, 0, 0), seg: 16, uAlong: true }), M.plank(), g);
    for (const y of [0.71, 0.91]) mk(ringGeo(v(0.27, y, -1.5), v(0, 1, 0), 0.165, 0.008, 0.012, 24, 6), M.iron(), g);
    // the cart's own lantern on a short post at its front corner, behind the one pulling
    mk(rodGeo(v(-0.47, 0.66, -0.47), v(-0.47, 1.52, -0.47), 0.022, 0.018), M.timber(), g);
    mk(rodGeo(v(-0.47, 1.5, -0.47), v(-0.47, 1.5, -0.3), 0.014), M.timber(), g);
    const lan = lantern(g, -0.47, 1.3, -0.3, 0.75); lightAt(C, lan, 3.4, 1.4);
    return C;
  })();

  // ---- the horse: one lofted barrel, neck and head, tapered legs, a mane, a tail, a saddle
  const HORSE = (() => {
    const C = craft("horse"), g = C.g, X = v(1, 0, 0);
    mk(sweep([[0, 1.17, -0.96, 0.03, 0.05], [0, 1.19, -0.9, 0.15, 0.2], [0, 1.2, -0.74, 0.23, 0.3], [0, 1.19, -0.45, 0.265, 0.33], [0, 1.16, -0.05, 0.27, 0.34], [0, 1.18, 0.32, 0.26, 0.35], [0, 1.24, 0.62, 0.22, 0.33], [0, 1.3, 0.8, 0.14, 0.24], [0, 1.33, 0.88, 0.04, 0.07]], { side: X, seg: 16 }), M.coat(), g);
    const NECK = [[0, 1.3, 0.55, 0.18, 0.28], [0, 1.6, 0.84, 0.135, 0.21], [0, 1.85, 1.0, 0.11, 0.155], [0, 1.95, 1.1, 0.1, 0.13], [0, 1.87, 1.22, 0.1, 0.13], [0, 1.68, 1.35, 0.085, 0.11], [0, 1.52, 1.45, 0.075, 0.085], [0, 1.47, 1.49, 0.03, 0.04]];
    mk(sweep(NECK, { side: X, seg: 14 }), M.coat(), g);
    // the mane follows the neck's crest, tapering in at the withers and the poll
    { const P = [], Rr = []; const crest = [[1.36, 0.46, 0.01, 0.02], [1.6, 0.7, 0.032, 0.06], [1.82, 0.88, 0.034, 0.065], [1.99, 1.0, 0.03, 0.05], [2.06, 1.08, 0.008, 0.012]];
      const cur = new THREE.CatmullRomCurve3(crest.map(c => v(0, c[0], c[1]))), cr = new THREE.CatmullRomCurve3(crest.map(c => v(c[2], c[3], 0)));
      for (let i = 0; i <= 20; i++) { P.push(cur.getPoint(i / 20)); Rr.push(cr.getPoint(i / 20)); }
      mk(tubeGeo(P, Rr, { side: X, seg: 10 }), M.mane(), g); }
    for (const s of [-1, 1]) mk(sweep([[s * 0.055, 1.97, 1.06, 0.03, 0.04], [s * 0.065, 2.08, 1.04, 0.022, 0.03], [s * 0.07, 2.16, 1.03, 0.004, 0.006]], { side: v(0, 0, 1), seg: 8 }), M.head(), g);
    mk(sweep([[0, 1.36, -0.88, 0.05, 0.06], [0, 1.3, -1.0, 0.065, 0.075], [0, 1.0, -1.1, 0.07, 0.08], [0, 0.7, -1.1, 0.055, 0.06], [0, 0.52, -1.06, 0.012, 0.012]], { side: X, seg: 10 }), M.mane(), g);
    for (const s of [-1, 1]) {
      const fx = s * 0.15, fz = 0.55, hx = s * 0.16, hz = -0.6;
      mk(sweep([[fx, 1.08, fz, 0.11, 0.13], [fx, 0.8, fz + 0.02, 0.08, 0.09], [fx, 0.55, fz + 0.03, 0.05, 0.06], [fx, 0.32, fz, 0.04, 0.045], [fx, 0.15, fz + 0.01, 0.045, 0.05], [fx, 0.08, fz + 0.035, 0.05, 0.055]], { side: X, seg: 10 }), M.legs(), g);
      mk(sweep([[hx, 1.12, hz, 0.13, 0.17], [hx, 0.82, hz - 0.05, 0.1, 0.12], [hx, 0.55, hz - 0.12, 0.05, 0.07], [hx, 0.32, hz - 0.08, 0.04, 0.045], [hx, 0.15, hz - 0.06, 0.045, 0.05], [hx, 0.08, hz - 0.035, 0.05, 0.055]], { side: X, seg: 10 }), M.legs(), g);
      for (const [x, z] of [[fx, fz + 0.04], [hx, hz - 0.03]]) mk(sweep([[x, 0.1, z, 0.05, 0.055], [x, 0.05, z + 0.005, 0.06, 0.065], [x, 0.0, z + 0.01, 0.066, 0.07]], { side: X, seg: 10 }), M.hoof(), g);
    }
    // the saddle: one leather shell over the back, its pommel and cantle raised, and a girth
    { const rings = []; const prof = z => ({ cy: 1.17, rx: 0.272, ry: 0.342 });
      for (let k = 0; k <= 12; k++) { const t = k / 12, z = -0.32 + 0.5 * t, P0 = prof(z), lift = 0.05 + 0.09 * Math.pow(Math.abs(2 * t - 1), 3) * (t > 0.5 ? 0.8 : 1), out = [], inn = [];
        for (let i = 0; i <= 12; i++) { const a = Math.PI * (0.16 + 0.68 * i / 12), up = Math.sin(a), dz = Math.pow(up, 6);
          out.push(v(Math.cos(a) * (P0.rx + 0.035), P0.cy + up * (P0.ry + 0.015) + lift * dz, z)); inn.push(v(Math.cos(a) * (P0.rx + 0.005), P0.cy + up * P0.ry - 0.002, z)); }
        rings.push([...out, ...inn.reverse()]); }
      mk(gridGeo(rings, { cap0: true, cap1: true, rep: 0.6 }), M.hide(), g); }
    mk(ringGeo(v(0, 1.17, 0.02), v(0, 0, 1), 1, 0.012, 0.03, 40, 6).scale(0.29, 0.355, 1).translate(0, 1.17 * (1 - 0.355), 0), M.hide(), g);
    // the bridle: a noseband round the muzzle and a cheek strap each side
    mk(ringGeo(v(0, 0, 0), v(0, -0.62, 0.78), 1, 0.01, 0.01, 24, 6).scale(0.082, 0.1, 0.1).translate(0, 1.6, 1.38), M.hide(), g);
    for (const s of [-1, 1]) mk(rodGeo(v(s * 0.08, 1.59, 1.37), v(s * 0.095, 1.93, 1.09), 0.01), M.hide(), g);
    return Object.assign(C, { seatY: 1.565, seatZ: -0.06, bit: [v(-0.085, 1.56, 1.39), v(0.085, 1.56, 1.39)] });
  })();

  // superellipse ring (rounded box) in x/y at depth z: half width hw, from yb to yt, p the squareness
  function superPts(hw, yb, yt, z, p, a0, a1, n) {
    const yc = (yb + yt) / 2, hh = (yt - yb) / 2, out = [];
    for (let i = 0; i <= n; i++) { const a = a0 + (a1 - a0) * i / n, c = Math.cos(a), s = Math.sin(a); out.push(v(hw * Math.sign(c) * Math.pow(Math.abs(c), 2 / p), yc + hh * Math.sign(s) * Math.pow(Math.abs(s), 2 / p), z)); }
    return out;
  }
  const superRing = (hw, yb, yt, z, p, n = 24) => superPts(hw, yb, yt, z, p, 0, 2 * Math.PI, n).slice(0, n);
  // an open tub: the superellipse cut at the rim, an inner skin `t` in, closed over the rim
  function tubRing(hw, yb, yt, rim, z, p, t, n = 12) {
    const cut = (hw2, yb2, yt2) => { const yc = (yb2 + yt2) / 2, hh = (yt2 - yb2) / 2, q = clamp((rim - yc) / hh, -0.999, 0.999), a = Math.asin(Math.sign(q) * Math.pow(Math.abs(q), p / 2)); return superPts(hw2, yb2, yt2, z, p, a, Math.PI - a - 2 * Math.PI, n); };
    return [...cut(hw, yb, yt), ...cut(hw - t, yb + t, yt - t).reverse()];
  }
  // lofts a list of sections (functions of z) into one form
  const loftZ = (zs, f, o) => gridGeo(zs.map(f), o);
  const lerpTab = (tab, z) => { for (let i = 0; i + 1 < tab.length; i++) { const a = tab[i], b = tab[i + 1]; if (z >= Math.min(a[0], b[0]) && z <= Math.max(a[0], b[0])) { const t = (z - a[0]) / (b[0] - a[0]), s = t * t * (3 - 2 * t); return a.map((x, k) => x + (b[k] - x) * s); } } return tab[z > tab[0][0] ? 0 : tab.length - 1]; };
  const steps = (z0, z1, n) => Array.from({ length: n + 1 }, (_, i) => z0 + (z1 - z0) * i / n);

  // ---- the small car: an open two-seater, a painted body lofted in three parts round the cockpit
  const CAR = (() => {
    const C = craft("car"), g = C.g, Pq = 4.2, rim = 0.8;
    const HOOD = [[1.62, 0.36, 0.42, 0.72], [1.5, 0.52, 0.34, 0.86], [1.1, 0.6, 0.3, 0.93], [0.3, 0.64, 0.3, 0.96]];
    const TAIL = [[-0.7, 0.64, 0.3, 0.96], [-1.0, 0.64, 0.3, 1.0], [-1.35, 0.58, 0.32, 0.92], [-1.6, 0.42, 0.38, 0.7]];
    mk(loftZ(steps(0.3, 1.62, 14), z => { const [, hw, yb, yt] = lerpTab(HOOD, z); return superRing(hw, yb, yt, z, Pq); }, { cap0: true, cap1: true }), M.paint(), g);
    mk(loftZ(steps(-0.7, 0.3, 6), z => tubRing(0.64, 0.3, 0.96, rim, z, Pq, 0.04), {}), M.paint(), g);
    mk(loftZ(steps(-1.6, -0.7, 10), z => { const [, hw, yb, yt] = lerpTab(TAIL, z); return superRing(hw, yb, yt, z, Pq); }, { cap0: true, cap1: true }), M.paint(), g);
    // the cockpit's floor and a leather bench with its back
    slab(g, M.boards(), 1.14, 0.03, 0.95, 0, 0.36, -0.2, 0, 2);
    slab(g, M.hide(), 1.12, 0.12, 0.5, 0, 0.47, -0.43, 0, 0.6);
    mk(sweep([[-0.5, 0.75, -0.66, 0.06, 0.18], [0, 0.76, -0.66, 0.07, 0.19], [0.5, 0.75, -0.66, 0.06, 0.18]], { side: v(0, 0, 1), seg: 12 }), M.hide(), g);
    // wheels with mudguards over them
    for (const z of [-1.0, 1.0]) for (const s of [-1, 1]) {
      const c = v(s * 0.66, 0.33, z);
      mk(ringGeo(c, v(1, 0, 0), 0.26, 0.075, 0.075, 32, 10), M.rubber(), g);
      mk(sweep([[s * 0.66 - 0.06, 0.33, z, 0.19], [s * 0.66, 0.33, z, 0.2], [s * 0.66 + 0.06, 0.33, z, 0.19]], { side: v(0, 1, 0), seg: 16 }), M.iron(), g);
      const P = [], Rr = []; for (let k = 0; k <= 14; k++) { const a = Math.PI * (0.06 + 0.88 * k / 14); P.push(v(s * 0.68, 0.33 + Math.sin(a) * 0.42, z + Math.cos(a) * 0.42)); Rr.push(v(0.14, 0.018, 0)); }
      mk(tubeGeo(P, Rr, { side: v(1, 0, 0), seg: 10 }), M.paint(), g);
    }
    for (const s of [-1, 1]) { const P = [], Rr = []; for (let k = 0; k <= 10; k++) { const z = -0.6 + 1.2 * k / 10; P.push(v(s * 0.72, 0.36, z)); Rr.push(v(0.1, 0.016, 0)); } mk(tubeGeo(P, Rr, { side: v(1, 0, 0), seg: 8 }), M.boards(), g); }
    // headlamps: brass bowls with their glass; they light the road ahead at night
    for (const s of [-1, 1]) {
      mk(sweep([[s * 0.4, 0.98, 1.36, 0.02], [s * 0.4, 0.98, 1.42, 0.09], [s * 0.4, 0.98, 1.52, 0.1]], { side: v(1, 0, 0), seg: 14 }), M.brass(), g);
      const gl = mk(new THREE.CircleGeometry(0.092, 14), glowMat, g, s * 0.4, 0.98, 1.521, false);
      lightAt(C, gl, 6, 0.9, 0xfff0c8);
      mk(rodGeo(v(s * 0.4, 0.9, 1.42), v(s * 0.4, 0.86, 1.3), 0.02), M.iron(), g);
    }
    // the windscreen: a glass in an iron frame on the scuttle
    mk(new THREE.PlaneGeometry(1.1, 0.34), M.glass(), g, 0, 1.15, 0.3, false).rotation.x = -0.25;
    { const P = [v(-0.56, 0.96, 0.33), v(-0.56, 1.32, 0.25), v(0.56, 1.32, 0.25), v(0.56, 0.96, 0.33)]; for (let i = 0; i < 3; i++) mk(rodGeo(P[i], P[i + 1], 0.016), M.iron(), g); }
    return Object.assign(C, { seatY: 0.53, seatZ: -0.43, seatX: -0.27, dashZ: 0.3 });
  })();

  // an airfoil section in z/y about (x, y0, z0), chord c, thickness tk; on a vertical fin, x and y swap
  function foil(x, y0, z0, c, tk, n = 16, fin = false) {
    const out = [];
    for (let i = 0; i < n; i++) { const a = i / n * Math.PI * 2, ca = Math.cos(a), sa = Math.sin(a), th = c * (sa > 0 ? tk : tk * 0.4) * sa * Math.sqrt((1 + ca) / 2) * 1.35, zz = z0 + c * 0.5 * ca;
      out.push(fin ? v(th, x, zz) : v(x, y0 + th, zz)); }
    return out;
  }
  // a wing lofted span-wise with a rounded tip; f(t) gives [chord, z of the chord's middle, y]
  function wingGeo(span, f, tk, sgn = 1, fin = false) {
    const st = [0, 0.15, 0.35, 0.55, 0.72, 0.84, 0.92, 0.965, 0.99, 1];
    return gridGeo(st.map(t => { const tip = t > 0.82 ? Math.sqrt(Math.max(0.0004, 1 - ((t - 0.82) / 0.18) ** 2)) : 1, [c, z, y] = f(t); return foil(sgn * span * t, y, z + c * (1 - tip) * 0.18, Math.max(0.03, c * tip), tk * (0.5 + 0.5 * tip), 16, fin); }), { cap0: true, cap1: true });
  }
  // ---- the small plane: an open-cockpit monoplane in doped linen, an iron cowl, a wooden propeller
  const PLANE = (() => {
    const C = craft("plane"), g = C.g, Pq = 2.6, rim = 1.3;
    const FUS = [[2.1, 0.24, 0.84, 1.3], [1.85, 0.42, 0.62, 1.42], [1.2, 0.48, 0.55, 1.48], [0.55, 0.48, 0.55, 1.48]];
    const AFT = [[-0.45, 0.48, 0.55, 1.48], [-1.5, 0.32, 0.72, 1.38], [-2.6, 0.14, 0.95, 1.28], [-3.15, 0.04, 1.1, 1.22]];
    mk(loftZ(steps(1.5, 2.1, 6), z => { const [, hw, yb, yt] = lerpTab(FUS, z); return superRing(hw, yb, yt, z, Pq); }, { cap1: true }), M.iron(), g);
    mk(loftZ(steps(0.55, 1.5, 8), z => { const [, hw, yb, yt] = lerpTab(FUS, z); return superRing(hw, yb, yt, z, Pq); }, { cap0: true }), M.fabric(), g);
    mk(loftZ(steps(-0.45, 0.55, 6), z => tubRing(0.48, 0.55, 1.48, rim, z, Pq, 0.035), {}), M.fabric(), g);
    mk(loftZ(steps(-3.15, -0.45, 16), z => { const [, hw, yb, yt] = lerpTab(AFT, z); return superRing(hw, yb, yt, z, Pq); }, { cap0: true, cap1: true }), M.fabric(), g);
    // the spinner and a two-bladed wooden propeller, standing still
    mk(sweep([[0, 1.07, 2.08, 0.15], [0, 1.07, 2.2, 0.13], [0, 1.07, 2.32, 0.07], [0, 1.07, 2.38, 0.01]], { side: v(1, 0, 0), seg: 14 }), M.iron(), g);
    for (const s of [-1, 1]) mk(sweep([[0, 1.07 + s * 0.08, 2.17, 0.06, 0.02], [0, 1.07 + s * 0.4, 2.17, 0.075, 0.016], [0, 1.07 + s * 0.78, 2.17, 0.06, 0.012], [0, 1.07 + s * 0.9, 2.17, 0.02, 0.006]], { side: v(1, 0, 0.35 * s), seg: 10 }), M.timber(), g);
    // a low wing with a little dihedral, the tailplane and the fin
    for (const s of [-1, 1]) {
      mk(wingGeo(3.7, t => [1.45 - 0.4 * t, 0.32 - 0.1 * t, 0.66 + 0.25 * t], 0.11, s), M.fabric(), g);
      mk(wingGeo(1.35, t => [0.72 - 0.25 * t, -2.82 + 0.06 * t, 1.17], 0.09, s), M.fabric(), g);
    }
    mk(wingGeo(0.9, t => [0.9 - 0.45 * t, -2.85 + 0.15 * t, 0], 0.09, 1, true).translate(0, 1.2, 0), M.fabric(), g);
    // the undercarriage: struts to two wheels, and a tail skid
    for (const s of [-1, 1]) {
      mk(rodGeo(v(s * 0.75, 0.66, 0.7), v(s * 0.92, 0.18, 0.82), 0.03, 0.025), M.iron(), g);
      mk(rodGeo(v(s * 0.3, 0.6, 0.55), v(s * 0.9, 0.18, 0.82), 0.022), M.iron(), g);
      mk(ringGeo(v(s * 0.96, 0.18, 0.82), v(1, 0, 0), 0.17, 0.055, 0.055, 28, 10), M.rubber(), g);
      mk(sweep([[s * 0.93, 0.18, 0.82, 0.1], [s * 0.96, 0.18, 0.82, 0.12], [s * 0.99, 0.18, 0.82, 0.1]], { side: v(0, 1, 0), seg: 12 }), M.iron(), g);
    }
    mk(rodGeo(v(0, 1.12, -2.9), v(0, 0.9, -3.15), 0.02, 0.012), M.iron(), g);
    // the seat, the windscreen, and the panel lamp that lights the pilot at night
    slab(g, M.hide(), 0.62, 0.1, 0.48, 0, 0.72, -0.22, 0, 0.6);
    mk(sweep([[-0.3, 1.05, -0.47, 0.04, 0.2], [0, 1.06, -0.47, 0.05, 0.22], [0.3, 1.05, -0.47, 0.04, 0.2]], { side: v(0, 0, 1), seg: 12 }), M.hide(), g);
    mk(new THREE.PlaneGeometry(0.62, 0.26), M.glass(), g, 0, 1.44, 0.66, false).rotation.x = -0.45;
    { const P = [v(-0.31, 1.32, 0.71), v(-0.31, 1.56, 0.6), v(0.31, 1.56, 0.6), v(0.31, 1.32, 0.71)]; for (let i = 0; i < 3; i++) mk(rodGeo(P[i], P[i + 1], 0.012), M.iron(), g); }
    const lamp = mk(new THREE.CircleGeometry(0.035, 10), glowMat, g, 0.22, 1.2, 0.549, false); lamp.rotation.y = Math.PI;
    mk(sweep([[0.22, 1.2, 0.54, 0.04], [0.22, 1.2, 0.5, 0.045], [0.22, 1.2, 0.49, 0.02]], { side: v(1, 0, 0), seg: 10 }), M.brass(), g);
    lightAt(C, lamp, 2.8, 2.0);
    // the navigation lights at the wing tips, red to port and green to starboard
    for (const s of [-1, 1]) { const nm = s < 0 ? navR : navG, o = mk(sweep([[s * 3.62, 0.9, 0.2, 0.012], [s * 3.67, 0.9, 0.2, 0.045], [s * 3.72, 0.9, 0.2, 0.012]], { side: v(0, 1, 0), seg: 10 }), nm, g, 0, 0, 0, false); lightAt(C, o, 2.4, 1.2, s < 0 ? 0xff5040 : 0x50ff70); }
    return Object.assign(C, { seatY: 0.775, seatZ: -0.22 });
  })();

  // ================================================================ the figure, and its seats
  let F = null;
  const armLen = () => Math.max(0.24, F.torsoH * 0.56);
  function resetPose() {
    F.legs.forEach(l => { l.quaternion.identity(); });
    F.arms.forEach((a, i) => { a.quaternion.identity(); a.rotation.set(0, 0, (i ? 1 : -1) * 0.12); });
    F.body.rotation.set(0, 0, 0); F.head.rotation.set(0, 0, 0);
  }
  // aim an arm from its shoulder toward a point in the world; returns where the hand is
  function aimArm(i, target) {
    const piv = F.arms[i]; piv.quaternion.identity(); piv.parent.updateMatrixWorld(true);
    const local = piv.parent.worldToLocal(target.clone()).sub(piv.position).normalize();
    piv.quaternion.setFromUnitVectors(v(0, -1, 0), local); piv.updateMatrixWorld(true);
    return piv.localToWorld(v(0, -armLen() - 0.045, 0));
  }
  const legs = (rx, rz = 0) => F.legs.forEach((l, i) => l.rotation.set(rx, 0, (i ? 1 : -1) * rz));
  function seatOn(C, x, y, z, yaw, seated) {
    C.g.add(F.g); F.g.position.set(x, seated ? y - F.legLen : y, z); F.g.rotation.set(0, yaw, 0); C.g.updateMatrixWorld(true);
  }
  const W2 = (C, x, y, z) => C.g.localToWorld(v(x, y, z));
  const toCraft = (C, p) => C.g.worldToLocal(p.clone());
  function clearCtl(C) { for (const o of C.ctl.children.slice()) { C.ctl.remove(o); o.geometry.dispose(); } }
  const STAND = { g: new THREE.Group(), ctl: new THREE.Group(), lights: [] }; STAND.g.position.copy(AT.stand); STAND.g.rotation.y = -0.25; scene.add(STAND.g);

  const POSE = {
    stand() { seatOn(STAND, 0, 0, 0, 0, false); },
    boat() {
      const B = BOAT; seatOn(B, 0, B.seatY, B.seatZ, Math.PI, true); legs(-1.25, 0.14); F.body.rotation.x = 0.1;
      B.g.updateMatrixWorld(true); clearCtl(B);
      for (const [i, s] of [[0, 1], [1, -1]]) {
        // the rower faces the stern, so his left hand (arm 0) takes the starboard oar (+x)
        const H = toCraft(B, aimArm(i, W2(B, s * 0.24, B.lockY + 0.12, B.seatZ - 0.45))), O = v(s * B.lockX, B.lockY + 0.04, B.lockZ), d = O.clone().sub(H).normalize();
        const end = H.clone().addScaledVector(d, -0.1), loom = O.clone().addScaledVector(d, 1.35), tip = O.clone().addScaledVector(d, 2.05);
        mk(sweep([[end.x, end.y, end.z, 0.022], [H.x, H.y, H.z, 0.024], [O.x, O.y, O.z, 0.032], [loom.x, loom.y, loom.z, 0.026]], { seg: 8 }), M.timber(), B.ctl);
        // the blade: one lofted form flattening from the loom
        const side = d.clone().cross(v(0, 1, 0)).normalize().cross(d).normalize(), mid = loom.clone().lerp(tip, 0.55);
        mk(tubeGeo([loom, loom.clone().lerp(tip, 0.2), mid, tip.clone().addScaledVector(d, -0.03), tip], [v(0.026, 0.026, 0), v(0.05, 0.016, 0), v(0.075, 0.012, 0), v(0.07, 0.01, 0), v(0.02, 0.006, 0)], { side, seg: 10 }), M.timber(), B.ctl);
      }
      return v(0, 0.95, B.seatZ);
    },
    cart() {
      const Cc = CART; seatOn(Cc, 0, 0, 0.32, 0, false); legs(0, 0);
      F.legs[0].rotation.x = 0.28; F.legs[1].rotation.x = -0.22; F.body.rotation.x = 0.08;
      Cc.g.updateMatrixWorld(true); clearCtl(Cc);
      const shX = Math.abs(F.arms[1].position.x) + 0.06, handY = F.legLen + 0.16;
      for (const [i, s] of [[0, -1], [1, 1]]) {
        const H = toCraft(Cc, aimArm(i, W2(Cc, s * shX, handY, 0.24))), A = v(s * 0.36, 0.72, -0.48), d = H.clone().sub(A).normalize(), tipP = H.clone().addScaledVector(d, 0.14);
        mk(sweep([[A.x, A.y - 0.02, A.z - 0.25, 0.035, 0.04], [A.x, A.y, A.z, 0.034, 0.038], [H.x, H.y, H.z, 0.026], [tipP.x, tipP.y, tipP.z, 0.022]], { seg: 8 }), M.timber(), Cc.ctl);
      }
      return v(0, 1.0, -0.3);
    },
    horse() {
      const Hh = HORSE; seatOn(Hh, 0, Hh.seatY, Hh.seatZ, 0, true); legs(-0.32, 0.78);
      Hh.g.updateMatrixWorld(true); clearCtl(Hh);
      const hy = F.legLen + F.torsoH * 0.42;
      for (const [i, s] of [[0, -1], [1, 1]]) {
        const H = toCraft(Hh, aimArm(i, W2(Hh, s * 0.12, Hh.seatY - F.legLen + hy, Hh.seatZ + 0.42))), B = Hh.bit[i], M1 = H.clone().lerp(B, 0.5); M1.y -= 0.09;
        mk(sweep([[H.x, H.y, H.z, 0.011], [M1.x, M1.y, M1.z, 0.011], [B.x, B.y, B.z, 0.011]], { seg: 6, samples: 14 }), M.hide(), Hh.ctl);
      }
      return v(0, 2.0, 0);
    },
    car() {
      const Cr = CAR; seatOn(Cr, Cr.seatX, Cr.seatY, Cr.seatZ, 0, true); legs(-1.42, 0.06);
      Cr.g.updateMatrixWorld(true); clearCtl(Cr);
      const shY = Cr.seatY + F.torsoH * 0.84, hands = [0, 1].map(i => toCraft(Cr, aimArm(i, W2(Cr, Cr.seatX + (i ? 1 : -1) * 0.17, shY - 0.1, Cr.seatZ + 0.6))));
      const c = hands[0].clone().add(hands[1]).multiplyScalar(0.5), rr = clamp(hands[0].distanceTo(hands[1]) / 2, 0.12, 0.2), chest = v(Cr.seatX, shY, Cr.seatZ), n = chest.clone().sub(c).normalize();
      mk(ringGeo(c, n, rr, 0.016, 0.016, 32, 8), M.hide(), Cr.ctl);
      const u = n.clone().cross(v(1, 0, 0)).normalize(), w = n.clone().cross(u);
      for (const a of [0, 2.2, 4.1]) mk(rodGeo(c, c.clone().addScaledVector(u, Math.cos(a) * rr).addScaledVector(w, Math.sin(a) * rr), 0.01), M.iron(), Cr.ctl);
      mk(sweep([[c.x, c.y, c.z, 0.035], [c.x, c.y, c.z, 0.035]].map((p, k) => k ? [c.x - n.x * 0.05, c.y - n.y * 0.05, c.z - n.z * 0.05, 0.03] : p), {}), M.iron(), Cr.ctl);
      mk(rodGeo(c.clone().addScaledVector(n, -0.04), v(c.x, 0.7, Cr.dashZ + 0.05), 0.022), M.iron(), Cr.ctl);
      return v(Cr.seatX * 0.6, 1.2, Cr.seatZ);
    },
    plane() {
      const Pl = PLANE; seatOn(Pl, 0, Pl.seatY, Pl.seatZ, 0, true); legs(-1.38, 0.1);
      Pl.g.updateMatrixWorld(true); clearCtl(Pl);
      const gy = Pl.seatY + F.torsoH * 0.32, gz = Pl.seatZ + 0.42;
      const R0 = toCraft(Pl, aimArm(1, W2(Pl, 0.02, gy + 0.06, gz))), L0 = toCraft(Pl, aimArm(0, W2(Pl, -0.02, gy - 0.04, gz)));
      const top = R0.clone().lerp(L0, 0.5); top.y = Math.max(R0.y, L0.y) + 0.05;
      mk(rodGeo(v(top.x, 0.6, top.z + 0.03), top, 0.018, 0.016), M.iron(), Pl.ctl);
      mk(sweep([[top.x, top.y - 0.16, top.z, 0.026], [top.x, top.y - 0.06, top.z, 0.03], [top.x, top.y + 0.02, top.z, 0.028]], {}), M.hide(), Pl.ctl);
      mk(sweep([[0, 0.6, top.z + 0.03, 0.08], [0, 0.66, top.z + 0.03, 0.05]], {}), M.hide(), Pl.ctl);
      return v(0, 1.45, Pl.seatZ);
    },
  };

  // ================================================================ view state
  const CAM = {
    stand: { yaw: 0.35, pitch: 0.1, dist: 3.4 }, boat: { yaw: Math.PI - 0.55, pitch: 0.32, dist: 5.4 }, cart: { yaw: Math.PI + 0.62, pitch: 0.28, dist: 5.6 },
    horse: { yaw: Math.PI - 0.62, pitch: 0.22, dist: 5.8 }, car: { yaw: Math.PI + 0.55, pitch: 0.34, dist: 5.6 }, plane: { yaw: Math.PI + 0.5, pitch: 0.28, dist: 8.5 },
  };
  const ST = { mode: "stand", night: false, target: v(0, 1, 0) };
  let renderer = null, running = false, raf = 0, last = 0, dirty = true, shownAt = 0, wasBlink = false;

  function setLights() {
    const list = LAMPS.slice(), C = CRAFT[ST.mode];
    if (C) { C.g.updateMatrixWorld(true); for (const L of C.lights) { const p = L.obj.getWorldPosition(v(0, 0, 0)); list.push({ x: p.x, y: p.y, z: p.z, R: L.R, c: L.c, p: L.p }); } }
    PBD.setLights(list); PBD.LU.uLamp.value = ST.night ? 0.85 : 0;
  }
  function rebuild() {
    if (F) { F.g.parent && F.g.parent.remove(F.g); F.g.traverse(o => { if (o.isMesh && o.geometry) o.geometry.dispose(); }); }
    const spec = toSpec(LOOK);
    F = FK.friend(spec); FK.setExprOn(F, spec.mood, true); FK.liveFace(F, 0);
    F.g.traverse(o => { if (o.isMesh) { o.castShadow = true; o.receiveShadow = true; } });
    place(); dirty = true;
  }
  function place() {
    for (const [k, C] of Object.entries(CRAFT)) C.g.visible = k === ST.mode;
    resetPose();
    const tl = POSE[ST.mode]();
    const host = ST.mode === "stand" ? STAND : CRAFT[ST.mode];
    host.g.updateMatrixWorld(true);
    ST.target = ST.mode === "stand" ? STAND.g.localToWorld(v(0, LOOK.h * 0.58, 0)) : host.g.localToWorld(tl);
    sun.target.position.copy(ST.target); setSun();
    setLights(); dirty = true;
  }
  function setMode(m) { ST.mode = m; $("cSeen").value = m; place(); }
  function setSun() {
    const d = ST.night ? v(-0.4, 0.7, 0.55) : v(0.55, 0.78, 0.32);
    sun.position.copy(d.normalize()).multiplyScalar(60).add(ST.target); sun.target.updateMatrixWorld();
  }
  function setNight(n) {
    ST.night = n;
    const cA = new THREE.Color(n ? 0x0e1424 : 0x9fc6e2);
    scene.background = cA.clone(); scene.fog.color.copy(cA);
    if (!n) { sun.color.setHSL(0.09, 0.5, 0.85); sun.intensity = 1.8; } else { sun.color.set(0x8fa6d8); sun.intensity = 0.035; }   // the moon, dim enough that the lamps read
    setSun();
    hemi.intensity = n ? 0.008 : 0.57; hemi.color.copy(cA).lerp(new THREE.Color(0xffffff), 0.35); hemi.groundColor.set(n ? 0x1a1a22 : 0x5a5040);
    glowMat.color.set(n ? 0xffd9a0 : 0x3a3832); navR.color.set(n ? 0xff5a46 : 0x501810); navG.color.set(n ? 0x5aff78 : 0x184a20);
    stars.material.opacity = n ? 1 : 0;
    if (running) setLights(); dirty = true;
  }

  // ---- input: drag to turn about the figure, wheel or pinch to zoom; no easing, nothing on its own
  function makeRenderer() {
    renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setPixelRatio(Math.min(2, devicePixelRatio)); renderer.outputEncoding = THREE.sRGBEncoding;
    renderer.shadowMap.enabled = true; renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    const cv = renderer.domElement; cv.tabIndex = 0; cv.setAttribute("aria-label", "Your character, standing or at the controls of a craft. Drag to turn the view.");
    view.prepend(cv);
    const pts = new Map(); let pinch = 0;
    cv.addEventListener("pointerdown", e => { cv.setPointerCapture(e.pointerId); pts.set(e.pointerId, [e.clientX, e.clientY]); if (pts.size === 2) { const [a, b] = [...pts.values()]; pinch = Math.hypot(a[0] - b[0], a[1] - b[1]); } });
    cv.addEventListener("pointermove", e => {
      const p = pts.get(e.pointerId); if (!p) return; const O = CAM[ST.mode];
      if (pts.size === 2) { p[0] = e.clientX; p[1] = e.clientY; const [a, b] = [...pts.values()], d = Math.hypot(a[0] - b[0], a[1] - b[1]); if (pinch) O.dist = clamp(O.dist * pinch / d, 1.6, 16); pinch = d; dirty = true; return; }
      const dx = e.clientX - p[0], dy = e.clientY - p[1]; p[0] = e.clientX; p[1] = e.clientY;
      O.yaw -= dx * 0.008; O.pitch = clamp(O.pitch + dy * 0.006, -0.25, 1.3); dirty = true;
    });
    const up = e => { pts.delete(e.pointerId); pinch = 0; };
    cv.addEventListener("pointerup", up); cv.addEventListener("pointercancel", up);
    cv.addEventListener("wheel", e => { e.preventDefault(); const O = CAM[ST.mode]; O.dist = clamp(O.dist * (e.deltaY > 0 ? 1.12 : 0.89), 1.6, 16); dirty = true; }, { passive: false });
    cv.addEventListener("keydown", e => {
      const O = CAM[ST.mode], k = e.key, st = 0.06;
      if (k === "ArrowLeft") O.yaw += st; else if (k === "ArrowRight") O.yaw -= st; else if (k === "ArrowUp") O.pitch = clamp(O.pitch + st, -0.25, 1.3); else if (k === "ArrowDown") O.pitch = clamp(O.pitch - st, -0.25, 1.3);
      else if (k === "+" || k === "=") O.dist = clamp(O.dist * 0.89, 1.6, 16); else if (k === "-") O.dist = clamp(O.dist * 1.12, 1.6, 16); else return;
      e.preventDefault(); dirty = true;
    });
    new ResizeObserver(() => { dirty = true; resize(); }).observe(view);
    resize();
  }
  function resize() { if (!renderer) return; const w = view.clientWidth, hh = view.clientHeight; renderer.setSize(w, hh, false); camera.aspect = w / Math.max(1, hh); camera.updateProjectionMatrix(); }
  function placeCamera() {
    const O = CAM[ST.mode], t = ST.target, cp = Math.cos(O.pitch);
    camera.position.set(t.x + Math.sin(O.yaw) * cp * O.dist, t.y + Math.sin(O.pitch) * O.dist, t.z + Math.cos(O.yaw) * cp * O.dist);
    if (ST.mode !== "plane") camera.position.y = Math.max(camera.position.y, (ST.mode === "boat" ? WATER_Y : 0) + 0.3);
    camera.lookAt(t);
  }
  function tick(t) {
    if (!running) return;
    raf = requestAnimationFrame(tick);
    const dt = Math.min(0.05, (t - last) / 1000 || 0); last = t;
    FK.liveFace(F, dt);
    const blink = F.blinkT >= 0;
    if (!dirty && !blink && !wasBlink && t - shownAt > 3000) return;   // the first seconds draw while the textures load
    wasBlink = blink; dirty = false;
    placeCamera(); renderer.render(scene, camera);
  }
  $("cSeen").addEventListener("change", e => setMode(e.target.value));

  window.CHAR3D = {
    show() { if (!renderer) makeRenderer(); setLights(); if (!running) { running = true; dirty = true; last = shownAt = performance.now(); raf = requestAnimationFrame(tick); } },
    hide() { running = false; cancelAnimationFrame(raf); },
    setNight, setMode,
    // for the page's checks
    _orbit(yaw, pitch, dist) { Object.assign(CAM[ST.mode], { yaw, pitch, dist }); dirty = true; },
    _look() { return JSON.parse(JSON.stringify(LOOK)); }, _set: set, _roll(seed) { LOOK = roll(seed); rebuild(); renderPanel(); },
    ST,
  };
  renderPanel(); rebuild(); setNight(false);
})();
