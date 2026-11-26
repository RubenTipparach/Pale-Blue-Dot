// Survival mockup, Fire tab: wildfire design 2, 3, 5, 7 and 12 on a patch of hex cells (2.83 m apart,
// 1 m layers). Each cell's plant dryness (plant_dry, design 2) comes from its plant's kind, the season,
// the ground's wetness, the air, shade under trees, water nearby, hollows, the soil's patches and
// grazing; below 0.5 nothing catches. Grass, shrubs, broadleaf and conifer trees, a stream with its
// green banks, a dug break, a pen's grazed pasture and a player's tilled field (the fire never takes
// the player's). Every second of fire time each burning cell tries its six neighbours. Shown at six
// seconds of fire a real second. Built when the tab first opens; its loop stops when the tab closes.
(function () {
  "use strict";
  const S = () => window.SurvivalCommon;
  const CELL = 2.83, RAD = 19, SPEED = 6;
  let built = false, running = false, playing = true, raf = 0, night = true, overlay = false;
  let renderer, scene, cam, orbitSt, applyOrbit, ground, groundCol, cells = [], byKey = new Map();
  let trunks, crownsB, crownsC, flames, smoke, rain, water, moon, hemi, sun, stage, stat;
  let simT = 0, acc = 0, ft = 0, rng, last = 0;
  const DIRS = [[1, 0], [1, -1], [0, -1], [-1, 0], [-1, 1], [0, 1]];
  const P = { season: "dry", wind: 3, rain: 0 };
  // design 2's seasons: the place's own year (0 its midsummer), ground_wet, humidity, air in C
  const SEASONS = { spring: { y: 0.75, gw: 0.5, h: 0.6, t: 16 }, summer: { y: 0.0, gw: 0.25, h: 0.45, t: 26 }, dry: { y: 0.1, gw: 0.12, h: 0.32, t: 29 } };
  // design 3 and 11 (decision 12): spread per second at burn 1, burn seconds; limits
  const SPREAD = { grass: 0.2, shrub: 0.12, broad: 0.02, conifer: 0.04 }, BURN = { grass: 15, shrub: 40, broad: 90, conifer: 90 };
  const LIMIT = { cells: 300, burning: 40 }, DRY_MIN = 0.5;
  const smooth = (lo, hi, x) => { const t = Math.min(1, Math.max(0, (x - lo) / (hi - lo))); return t * t * (3 - 2 * t); };
  const bump = (x, at) => { const d = Math.abs(x - at[0]); return 1 - smooth(0, at[1], Math.min(d, 1 - d)); };
  const burnOf = (d) => (d < DRY_MIN ? 0 : ((d - DRY_MIN) / (1 - DRY_MIN)) ** 2);
  let litCount = 0;

  function cellXZ(q, r) { return [CELL * (q + r / 2), CELL * 0.866 * r]; }
  function makeCells() {
    cells = []; byKey.clear();
    const H = S().hash;
    for (let q = -RAD; q <= RAD; q++) for (let r = -RAD; r <= RAD; r++) {
      if (Math.abs(q + r) > RAD) continue;
      const [x, z] = cellXZ(q, r);
      let h = Math.round(1.6 * Math.sin(x * 0.05 + 1) + 1.2 * Math.sin(z * 0.07) + x * 0.03);
      const c = { q, r, x, z, h, fuel: "grass", tree: null, state: 0, left: 0, burnt: 0, i: cells.length };
      const sx = 22 + 5 * Math.sin(z / 11);
      if (Math.abs(x - sx) < 2.6) { c.fuel = "water"; c.h = Math.round(sx * 0.03) - 2; }
      else if (Math.abs(x + 8) < 1.6 && z > -26 && z < 18) c.fuel = "soil";
      else if (x > 2 && x < 15 && z > 14 && z < 30) c.fuel = "field";
      else if (x > -30 && x < -18 && z > 14 && z < 26) c.fuel = "pen";
      else if (H(q, r, 5) < (x > -4 && x < 18 ? 0.5 : 0.1)) c.tree = { kind: c.h >= 2 || H(q, r, 6) < 0.35 ? "conifer" : "broad", s: 0.8 + 0.45 * H(q, r, 7), rot: H(q, r, 8) * 6.28, ox: (H(q, r, 9) - 0.5) * 0.8, oz: (H(q, r, 10) - 0.5) * 0.8 };
      else if (H(q, r, 11) < 0.08) c.fuel = "shrub";
      byKey.set(q + "," + r, c);
      cells.push(c);
    }
    // cells to the nearest water, by the neighbour tables (design 2's water term)
    let ring = cells.filter((c) => c.fuel === "water");
    cells.forEach((c) => (c.dw = c.fuel === "water" ? 0 : 99));
    for (let d = 1; ring.length && d < 6; d++) { const next = []; ring.forEach((c) => nbrs(c).forEach((n) => { if (n.dw > d) { n.dw = d; next.push(n); } })); ring = next; }
    // the pen's grazed pasture: its own cells and two round them
    cells.forEach((c) => (c.grazed = cells.some((p) => p.fuel === "pen" && Math.hypot(p.x - c.x, p.z - c.z) < 2.2 * CELL)));
    cells.forEach((c) => { const n = nbrs(c); c.trees = n.filter((m) => m.tree).length; c.mh = n.reduce((s, m) => s + m.h, 0) / n.length; });
    setDryness();
  }
  const kindOf = (c) => (c.tree ? c.tree.kind : c.fuel);
  // design 2: plant_dry = cure(kind, season) x weather x shade x water x hollow x patch x grazed
  function setDryness() {
    const se = SEASONS[P.season];
    const cs = Math.max(bump(se.y, [0.1, 0.2]), 0.5 * bump(se.y, [0.47, 0.22]));
    const weather = (1 - se.gw) * (1 - 0.6 * smooth(0.35, 0.85, se.h)) * (0.4 + 0.6 * smooth(0, 25, se.t));
    const H = S().hash;
    cells.forEach((c) => {
      const k = kindOf(c);
      const cure = { grass: 0.2 + 0.8 * cs, shrub: 0.25 + 0.6 * cs, broad: 0.15 + 0.45 * cs, conifer: 0.3 + 0.6 * cs }[k];
      if (cure === undefined) { c.dry = 0; return; }
      const shade = k === "grass" || k === "shrub" ? 1 - 0.07 * c.trees : 1;
      const water = 0.35 + 0.65 * smooth(1, 3, c.dw);
      const hollow = 1 + 0.06 * Math.max(-2, Math.min(2, c.h - c.mh));
      const n = 0.5 + 0.25 * Math.sin(c.x / 9 + 1.3) + 0.25 * Math.sin(c.z / 11 + 0.4 + c.x / 23);
      const patch = (0.62 + 0.5 * n) * (0.92 + 0.16 * H(c.q, c.r, 21));
      const grazed = c.grazed && k === "grass" ? 0.5 : 1;
      c.dry = Math.min(1, cure * weather * shade * water * hollow * patch * grazed);
    });
  }
  const nbrs = (c) => DIRS.map(([dq, dr]) => byKey.get((c.q + dq) + "," + (c.r + dr))).filter(Boolean);
  const burnable = (c) => c.fuel === "grass" || c.fuel === "shrub";

  // ---- colours a cell's top takes
  function topColour(c) {
    if (c.fuel === "water") return [0.16, 0.3, 0.42];
    if (c.fuel === "soil") return [0.5, 0.36, 0.24];
    if (c.fuel === "field") return [0.44, 0.3, 0.18];
    if (c.fuel === "pen") return [0.42, 0.5, 0.24];
    if (c.state === 1) return [0.16, 0.12, 0.1];
    if (c.state === 2) return c.tree ? [0.2, 0.19, 0.18] : [0.36, 0.35, 0.33];
    const d = c.dry;
    if (overlay) {
      // dryness overlay: greener is wetter; from 0.5 (catches) yellow to burnt orange
      if (d < DRY_MIN) { const t = d / DRY_MIN; return [0.12 + 0.3 * t, 0.42 + 0.16 * t, 0.22 - 0.06 * t]; }
      const t = (d - DRY_MIN) / (1 - DRY_MIN); return [0.82 + 0.1 * t, 0.7 - 0.36 * t, 0.2 - 0.08 * t];
    }
    // the grass's own look: lush green when wet, cured straw when dry
    const t = smooth(0.15, 0.95, d), k = c.fuel === "shrub" ? 0.82 : 1;
    return [(0.3 + 0.48 * t) * k, (0.5 + 0.16 * t) * k, (0.2 + 0.14 * t) * k];
  }

  function buildGround() {
    const pos = [], col = [], uv = [], idx = [];
    const R = CELL / Math.sqrt(3);
    cells.forEach((c) => {
      c.v0 = pos.length / 3;
      const y = c.h, cols = topColour(c);
      const corners = [];
      for (let k = 0; k < 6; k++) { const a = Math.PI / 6 + k * Math.PI / 3; corners.push([c.x + R * Math.cos(a), c.z + R * Math.sin(a)]); }
      pos.push(c.x, y, c.z); col.push(...cols); uv.push(c.x / 2.83, c.z / 2.83);
      corners.forEach(([x, z]) => { pos.push(x, y, z); col.push(...cols); uv.push(x / 2.83, z / 2.83); });
      for (let k = 0; k < 6; k++) idx.push(c.v0, c.v0 + 1 + ((k + 1) % 6), c.v0 + 1 + k);
      c.nTop = 7;
      // sides down to the lowest layer, soil coloured
      for (let k = 0; k < 6; k++) {
        const [ax, az] = corners[k], [bx, bz] = corners[(k + 1) % 6], b0 = pos.length / 3;
        const s = [0.36, 0.27, 0.19];
        pos.push(ax, y, az, bx, y, bz, bx, -6, bz, ax, -6, az);
        for (let m = 0; m < 4; m++) col.push(...s);
        uv.push(0, y, 1, y, 1, -6, 0, -6);
        idx.push(b0, b0 + 2, b0 + 1, b0, b0 + 3, b0 + 2);
      }
    });
    const g = new THREE.BufferGeometry();
    g.setAttribute("position", new THREE.Float32BufferAttribute(pos, 3));
    groundCol = new THREE.Float32BufferAttribute(col, 3);
    g.setAttribute("color", groundCol);
    g.setAttribute("uv", new THREE.Float32BufferAttribute(uv, 2));
    g.setIndex(idx); g.computeVertexNormals();
    const tex = S().tex(16, (x, y) => { const h = S().hash(x, y, 3), v = 200 + 50 * h - ((x + y * 3) % 7 === 0 ? 30 : 0); return [v, v, v * 0.96]; });
    ground = new THREE.Mesh(g, S().firelit(new THREE.MeshPhongMaterial({ map: tex, vertexColors: true, shininess: 0, specular: 0x000000 })));
    scene.add(ground);
    // the stream's water surface, a little below the banks
    const wpos = [], widx = [];
    cells.filter((c) => c.fuel === "water").forEach((c) => {
      const b = wpos.length / 3; wpos.push(c.x, c.h + 0.7, c.z);
      for (let k = 0; k < 6; k++) { const a = Math.PI / 6 + k * Math.PI / 3; wpos.push(c.x + R * 1.02 * Math.cos(a), c.h + 0.7, c.z + R * 1.02 * Math.sin(a)); }
      for (let k = 0; k < 6; k++) widx.push(b, b + 1 + ((k + 1) % 6), b + 1 + k);
    });
    const wg = new THREE.BufferGeometry(); wg.setAttribute("position", new THREE.Float32BufferAttribute(wpos, 3)); wg.setIndex(widx); wg.computeVertexNormals();
    water = new THREE.Mesh(wg, S().firelit(new THREE.MeshPhongMaterial({ color: 0x2a5878, transparent: true, opacity: 0.8, shininess: 60, specular: 0x334455 })));
    scene.add(water);
  }
  function paintCell(c) {
    const cols = topColour(c);
    for (let k = 0; k < c.nTop; k++) groundCol.setXYZ(c.v0 + k, cols[0], cols[1], cols[2]);
    groundCol.needsUpdate = true;
  }

  // ---- trees: one fitted trunk (flared foot, tapering), a lumpy broadleaf crown or a tiered conifer
  function trunkGeo() {
    const pts = [[0.34, 0], [0.26, 0.25], [0.19, 0.9], [0.16, 2.4], [0.12, 3.8], [0.085, 4.5], [0.05, 4.56], [0, 4.6]].map(([r, y]) => new THREE.Vector2(r, y));
    return new THREE.LatheGeometry(pts, 9);
  }
  function broadGeo() {
    const g = S().weld(new THREE.IcosahedronGeometry(2.3, 2));
    const p = g.attributes.position, v = new THREE.Vector3();
    for (let i = 0; i < p.count; i++) {
      v.fromBufferAttribute(p, i);
      const n = v.clone().normalize();
      const k = 1 + 0.16 * Math.sin(n.x * 5.1 + n.y * 3.3) + 0.12 * Math.sin(n.z * 6.7 - n.y * 2.1) - (n.y < -0.4 ? 0.25 * (-n.y - 0.4) : 0);
      v.multiplyScalar(k); v.y *= 0.82;
      p.setXYZ(i, v.x, v.y + 4.6, v.z);
      g.attributes.uv && g.attributes.uv.setXY(i, n.x * 2 + 2, n.y * 2 + 2);
    }
    g.computeVertexNormals();
    return g;
  }
  function coniferGeo() {
    const prof = [[0.0, 1.0], [1.9, 1.3], [1.5, 2.3], [0.8, 2.6], [1.55, 2.75], [1.1, 3.9], [0.55, 4.2], [1.15, 4.35], [0.7, 5.4], [0.3, 5.7], [0.6, 5.8], [0.0, 7.2]];
    return new THREE.LatheGeometry(prof.map(([r, y]) => new THREE.Vector2(r, y)), 11);
  }
  function buildTrees() {
    const treeCells = cells.filter((c) => c.tree);
    const bark = S().tex(16, (x, y) => { const s = (x * 7 + Math.floor(y / 3) * 3) % 5 === 0 ? 0.7 : 1; const v = (110 + 40 * S().hash(x, y >> 1, 4)) * s; return [v, v * 0.72, v * 0.5]; }, [2, 2]);
    const leaf = S().tex(16, (x, y) => { const h = S().hash(x, y, 12); const v = h > 0.8 ? 1.25 : h < 0.2 ? 0.75 : 1; return [70 * v, 118 * v, 52 * v]; }, [3, 3]);
    const needle = S().tex(16, (x, y) => { const h = S().hash(x, y, 13); const v = (y % 4 === 0 ? 0.75 : 1) * (0.9 + 0.25 * h); return [38 * v, 84 * v, 60 * v]; }, [4, 3]);
    const mk = (geo, map, n) => { const m = new THREE.InstancedMesh(geo, S().firelit(new THREE.MeshPhongMaterial({ map, shininess: 0, specular: 0x000000 })), n); m.instanceColor = new THREE.InstancedBufferAttribute(new Float32Array(n * 3).fill(1), 3); scene.add(m); return m; };
    trunks = mk(trunkGeo(), bark, treeCells.length);
    const nB = treeCells.filter((c) => c.tree.kind === "broad").length;
    crownsB = mk(broadGeo(), leaf, nB);
    crownsC = mk(coniferGeo(), needle, treeCells.length - nB);
    let b = 0, k = 0;
    treeCells.forEach((c, i) => { c.tree.ti = i; if (c.tree.kind === "broad") c.tree.ci = b++; else c.tree.ci = k++; });
    treeCells.forEach(setTree);
  }
  const m4 = new THREE.Matrix4(), q4 = new THREE.Quaternion(), e4 = new THREE.Euler();
  function setTree(c) {
    const t = c.tree, s = t.s;
    const pos = new THREE.Vector3(c.x + t.ox, c.h, c.z + t.oz);
    q4.setFromEuler(e4.set(0, t.rot, 0));
    const trunkS = t.kind === "conifer" ? s * 0.9 : s;
    const crowned = c.crown, snag = c.state === 2 && crowned ? 0.72 : 1;
    m4.compose(pos, q4, new THREE.Vector3(trunkS, trunkS * (t.kind === "conifer" ? 1.3 : 1) * snag, trunkS));
    trunks.setMatrixAt(t.ti, m4);
    const ch = c.state === 2 ? (crowned ? 0.2 : 0.45) : c.state === 1 ? 0.6 : 1;
    trunks.instanceColor.setXYZ(t.ti, ch, ch, ch);
    const crown = t.kind === "broad" ? crownsB : crownsC;
    const gone = crowned && (c.state === 2 || (c.state === 1 && c.left < BURN[t.kind] - 30));
    m4.compose(pos, q4, gone ? new THREE.Vector3(0, 0, 0) : new THREE.Vector3(s, s, s));
    crown.setMatrixAt(t.ci, m4);
    // a ground fire browns the leaves; a dry tree is a little yellower than a wet one
    const dt = smooth(0.3, 0.9, c.dry);
    if (c.state >= 1) crown.instanceColor.setXYZ(t.ci, 0.62, 0.42, 0.24);
    else crown.instanceColor.setXYZ(t.ci, 0.92 + 0.22 * dt, 1.02 - 0.02 * dt, 0.95 - 0.3 * dt);
    trunks.instanceMatrix.needsUpdate = trunks.instanceColor.needsUpdate = true;
    crown.instanceMatrix.needsUpdate = crown.instanceColor.needsUpdate = true;
  }

  // ---- particles: flames (additive), smoke (soft dark), and rain streaks
  function pointsMat(additive, tint) {
    return new THREE.ShaderMaterial({
      uniforms: { scale: { value: 600 } },
      vertexShader: `attribute float size; attribute vec4 col; varying vec4 vC; uniform float scale;
        void main(){ vC = col; vec4 mv = modelViewMatrix * vec4(position,1.0); gl_PointSize = size * scale / -mv.z; gl_Position = projectionMatrix * mv; }`,
      // a flame is a tongue, narrow at the top; smoke a soft round puff
      fragmentShader: `varying vec4 vC; void main(){ vec2 d = gl_PointCoord - vec2(0.5, 0.55);
        float w = ${tint ? "0.5" : "mix(0.18, 0.5, gl_PointCoord.y)"};
        float r = length(vec2(d.x * 0.5 / w, d.y)) * 2.0; if (r > 1.0) discard;
        float a = vC.a * (1.0 - r * r); vec3 c = mix(vC.rgb, vec3(1.0, 0.85, 0.5), ${tint ? "0.0" : "(1.0 - r) * 0.35"});
        gl_FragColor = vec4(c * ${tint ? "1.0" : "a"}, a); }`,
      transparent: true, depthWrite: false, blending: additive ? THREE.AdditiveBlending : THREE.NormalBlending,
    });
  }
  function makePoints(n, additive) {
    const g = new THREE.BufferGeometry();
    g.setAttribute("position", new THREE.Float32BufferAttribute(new Float32Array(n * 3), 3));
    g.setAttribute("size", new THREE.Float32BufferAttribute(new Float32Array(n), 1));
    g.setAttribute("col", new THREE.Float32BufferAttribute(new Float32Array(n * 4), 4));
    const p = new THREE.Points(g, pointsMat(additive, !additive));
    p.frustumCulled = false; scene.add(p); return p;
  }
  function updateParticles() {
    const burning = cells.filter((c) => c.state === 1 || (c.state === 2 && c.tree && simT - c.burnt < 40));
    const fp = flames.geometry.attributes, sp = smoke.geometry.attributes;
    let nf = 0, ns = 0;
    const NF = fp.size.count, NS = sp.size.count;
    burning.forEach((c) => {
      const crown = c.tree && c.crown && c.state === 1 && c.left >= BURN[c.tree.kind] - 30;
      const ember = c.state === 2;
      const per = ember ? 2 : crown ? 16 : c.damp ? 3 : c.tree ? 9 : 7;
      for (let k = 0; k < per && nf < NF; k++) {
        const ph = (ft * (0.9 + 0.3 * S().hash(c.i, k, 1)) + S().hash(c.i, k, 2)) % 1, life = 1 - ph;
        let x, y, z, size;
        if (crown) {
          // tongues round and over the burning crown, leaning downwind
          const a = S().hash(c.i, k, 3) * 6.283, rr = (1.6 + 1.1 * S().hash(c.i, k, 4)) * c.tree.s;
          const top = c.tree.kind === "conifer" ? 7 : 6.2;
          const h0 = (2 + (top - 2) * S().hash(c.i, k, 5)) * c.tree.s;
          const pull = 1 - (h0 / (top * c.tree.s)) * 0.6;
          x = c.x + c.tree.ox + Math.cos(a) * rr * pull + P.wind * 0.25 * ph; z = c.z + c.tree.oz + Math.sin(a) * rr * pull;
          y = c.h + h0 + ph * 3.2; size = 3.0 * (0.45 + 0.75 * life);
        } else {
          x = c.x + (S().hash(c.i, k, 3) - 0.5) * 2.4 + P.wind * 0.12 * ph; z = c.z + (S().hash(c.i, k, 4) - 0.5) * 2.4;
          y = c.h + (ember ? 0.15 : 0.35 + ph * 2.0); size = ember ? 0.6 : (c.tree ? 2.4 : 2.1) * (0.45 + 0.75 * life);
        }
        fp.position.setXYZ(nf, x, y, z);
        fp.size.setX(nf, size);
        fp.col.setXYZW(nf, 1.0, 0.22 + 0.32 * life, 0.05 + 0.08 * life * life, ember ? 0.6 : 0.95 * life);
        nf++;
      }
      if (!ember) for (let k = 0; k < 3 && ns < NS; k++) {
        const ph = (ft * 0.18 + S().hash(c.i, k, 6)) % 1;
        sp.position.setXYZ(ns, c.x + P.wind * 2.5 * ph + (S().hash(c.i, k, 7) - 0.5) * 3, c.h + 2 + ph * 22, c.z + (S().hash(c.i, k, 8) - 0.5) * 3);
        sp.size.setX(ns, 3 + 8 * ph);
        const g = night ? 0.09 : 0.42;
        sp.col.setXYZW(ns, g, g * 0.95, g * 0.9, 0.35 * (1 - ph));
        ns++;
      }
    });
    for (let i = nf; i < NF; i++) fp.size.setX(i, 0);
    for (let i = ns; i < NS; i++) sp.size.setX(i, 0);
    fp.position.needsUpdate = fp.size.needsUpdate = fp.col.needsUpdate = true;
    sp.position.needsUpdate = sp.size.needsUpdate = sp.col.needsUpdate = true;
    // the fire's light: burning cells grouped into bins of about 8.5 m, the brightest 48
    const bins = new Map();
    burning.forEach((c) => {
      const k = Math.floor(c.x / 8.5) + "," + Math.floor(c.z / 8.5);
      const w = c.state === 2 ? 0.25 : c.tree && c.crown ? 3 : c.damp ? 0.4 : 1;
      const b = bins.get(k) || { x: 0, y: 0, z: 0, w: 0 };
      b.x += c.x * w; b.y += (c.h + 2.5) * w; b.z += c.z * w; b.w += w; bins.set(k, b);
    });
    const list = [...bins.values()].sort((a, b) => b.w - a.w).slice(0, 48);
    rain.visible = P.rain > 0;
    const U = S().fireUniforms, v = new THREE.Vector3();
    list.forEach((b, i) => {
      v.set(b.x / b.w, b.y / b.w, b.z / b.w).applyMatrix4(cam.matrixWorldInverse);
      const flick = 0.85 + 0.15 * Math.sin(ft * 9 + i * 1.7) * Math.sin(ft * 5.3 + i);
      U.fireL.value[i].set(v.x, v.y, v.z, Math.min(9, 2.2 * Math.sqrt(b.w)) * flick);
    });
    U.fireN.value = list.length;
    stat.textContent = `burning ${cells.filter((c) => c.state === 1).length} / ${LIMIT.burning} · burnt ${litCount} / ${LIMIT.cells} cells · ${Math.round(simT)} s (x${SPEED})`;
  }
  function updateRain(dt) {
    rain.visible = P.rain > 0;
    if (!dt) return;
    if (!rain.visible) return;
    const p = rain.geometry.attributes.position, n = p.count / 2;
    for (let i = 0; i < n; i++) {
      let y = p.getY(i * 2) - dt * 22;
      if (y < -2) y += 40;
      const x = p.getX(i * 2), z = p.getZ(i * 2);
      p.setXYZ(i * 2, x, y, z); p.setXYZ(i * 2 + 1, x - 0.15, y - 1.1, z);
    }
    p.needsUpdate = true;
    rain.material.opacity = Math.min(0.55, 0.12 + P.rain * 0.08);
  }

  // ---- the rule (wildfire design 5, 7 and 12)
  function ignite(c, torch) {
    if (!c || !burnable(c) || c.state !== 0) return;
    if (c.dry < DRY_MIN) {
      // too green: a torch makes a 3 s flame that saves nothing (design 6); a neighbour never lights it
      if (!torch) return;
      c.state = 1; c.left = 3; c.damp = true; paintCell(c); return;
    }
    c.state = 1; c.damp = false; litCount++;
    const k = kindOf(c);
    c.crown = k === "conifer" && c.dry >= 0.85 && P.wind >= 6;
    c.left = BURN[k];
    paintCell(c); if (c.tree) setTree(c);
  }
  function tick() {
    simT += 1;
    const wind = P.wind, rain = P.rain;
    const lit = [];
    const burning = cells.filter((c) => c.state === 1);
    const atLimit = litCount >= LIMIT.cells || burning.filter((c) => !c.damp).length >= LIMIT.burning;
    burning.forEach((c) => {
      if (!c.damp && !atLimit && rain < 1) nbrs(c).forEach((n) => {
        if (n.state !== 0 || !burnable(n) || n.dry < DRY_MIN) return;
        let lam = SPREAD[kindOf(n)] * burnOf(n.dry);
        if (kindOf(n) === "conifer" && n.dry >= 0.85 && wind >= 6) lam *= 3;
        const dx = n.x - c.x, dz = n.z - c.z, cos = dx / Math.hypot(dx, dz);
        lam *= Math.max(0.2, 1 + 0.15 * wind * Math.max(0, cos) - 0.1 * wind * Math.max(0, -cos));
        lam *= n.h > c.h ? 1 + 0.1 * (n.h - c.h) : n.h < c.h ? 0.8 : 1;
        lam *= Math.exp(-rain / 0.5);
        if (rng() < 1 - Math.exp(-lam)) lit.push(n);
      });
      const before = c.left;
      c.left -= 1 + rain * 0.5;
      if (c.tree && c.crown && before >= BURN[c.tree.kind] - 30 && c.left < BURN[c.tree.kind] - 30) setTree(c);
      if (c.left <= 0) {
        if (c.damp) { c.state = 0; c.damp = false; paintCell(c); return; }
        c.state = 2; c.burnt = simT; paintCell(c); if (c.tree) setTree(c);
      }
    });
    lit.forEach((n) => litCount < LIMIT.cells && ignite(n));
  }
  // the page opens on a fire that takes: the first seed whose torch carries (a third of dry-season
  // torches fizzle on their own cell, decision 12); the rule is the same for every seed
  let seed0 = 0;
  function reset(trySeeds) {
    if (trySeeds !== false && P.season === "dry" && !reset.busy) {
      reset.busy = true;
      for (let k = 0; k < 24; k++) { seed0 = k; reset(false); for (let i = 0; i < 60; i++) tick(); if (litCount >= 40) break; }
      reset.busy = false;
    }
    rng = S().rng(20261126 + seed0 * 7919); simT = 0; acc = 0; litCount = 0;
    setDryness();
    cells.forEach((c) => { c.state = 0; c.left = 0; c.burnt = 0; c.crown = false; c.damp = false; paintCell(c); if (c.tree) setTree(c); });
    // the torch: the driest grass cell west of the dug break, upwind
    const near = cells.filter((c) => c.fuel === "grass" && !c.tree && c.x < -16 && c.x > -34 && Math.abs(c.z) < 12 && !c.grazed);
    const start = near.reduce((b, c) => (c.dry > b.dry ? c : b), near[0]);
    ignite(start, true);
    reset.start = { dry: start.dry, x: start.x, z: start.z, nb: nbrs(start).map((n) => [kindOf(n), +n.dry.toFixed(2), n.state]) };
    for (let i = 0; i < 15; i++) tick();
  }
  function setSeason(k) {
    P.season = k;
    document.querySelectorAll("[data-fseason]").forEach((b) => b.setAttribute("aria-pressed", String(b.dataset.fseason === k)));
    if (built) reset();
  }
  function setOverlay(o) {
    overlay = o; $("fDry").setAttribute("aria-pressed", String(o));
    if (built) { cells.forEach(paintCell); }
  }

  function setNight(n) {
    night = n;
    document.querySelectorAll("[data-ftime]").forEach((b) => b.setAttribute("aria-pressed", String((b.dataset.ftime === "night") === n)));
    if (!built) return;
    scene.background = new THREE.Color(n ? 0x050a14 : 0x9cc4e4);
    scene.fog = new THREE.Fog(n ? 0x050a14 : 0x9cc4e4, 70, 190);
    moon.intensity = n ? 0.07 : 0; hemi.intensity = n ? 0.05 : 0.55; sun.intensity = n ? 0 : 0.95;
    hemi.color.set(n ? 0x6070a0 : 0xd8e8ff); hemi.groundColor.set(n ? 0x101418 : 0x6a5a40);
    S().fireUniforms.fireC.value.setRGB(n ? 1.0 : 0.6, n ? 0.55 : 0.33, n ? 0.22 : 0.13);
  }
  function build() {
    stage = document.getElementById("fStage"); stat = document.getElementById("fStat");
    renderer = new THREE.WebGLRenderer({ antialias: true, preserveDrawingBuffer: true });
    renderer.setPixelRatio(Math.min(2, devicePixelRatio)); renderer.outputEncoding = THREE.sRGBEncoding;
    stage.appendChild(renderer.domElement);
    scene = new THREE.Scene();
    cam = new THREE.PerspectiveCamera(50, 1, 0.5, 600);
    moon = new THREE.DirectionalLight(0x9fb4e0, 0.16); moon.position.set(-40, 80, 30); scene.add(moon);
    sun = new THREE.DirectionalLight(0xfff1d8, 0); sun.position.set(50, 90, 40); scene.add(sun);
    hemi = new THREE.HemisphereLight(0x6070a0, 0x101418, 0.16); scene.add(hemi);
    makeCells(); buildGround(); buildTrees();
    flames = makePoints(6000, true); smoke = makePoints(1400, false);
    const rp = [];
    for (let i = 0; i < 1600; i++) { const x = (Math.random() - 0.5) * 110, z = (Math.random() - 0.5) * 110, y = Math.random() * 40; rp.push(x, y, z, x, y - 1.1, z); }
    const rg = new THREE.BufferGeometry(); rg.setAttribute("position", new THREE.Float32BufferAttribute(rp, 3));
    rain = new THREE.LineSegments(rg, new THREE.LineBasicMaterial({ color: 0x9fb4cc, transparent: true, opacity: 0.3 })); scene.add(rain);
    orbitSt = { t: new THREE.Vector3(-4, 0, 0), r: 92, pol: 0.95, az: 0.55, min: 10, max: 220 };
    const ray = new THREE.Raycaster(), m = new THREE.Vector2();
    orbitSt.click = (e) => {
      const r = renderer.domElement.getBoundingClientRect();
      m.set(((e.clientX - r.left) / r.width) * 2 - 1, -((e.clientY - r.top) / r.height) * 2 + 1);
      ray.setFromCamera(m, cam);
      const hit = ray.intersectObject(ground)[0];
      if (!hit) return;
      let best = null, bd = 1e9;
      cells.forEach((c) => { const d = Math.hypot(c.x - hit.point.x, c.z - hit.point.z); if (d < bd) { bd = d; best = c; } });
      ignite(best, true);
    };
    applyOrbit = S().orbit(cam, renderer.domElement, orbitSt);
    built = true;
    setNight(night);
    reset();
    resize();
  }
  function resize() {
    if (!built) return;
    const r = stage.getBoundingClientRect();
    renderer.setSize(r.width, r.height, false);
    renderer.domElement.style.width = r.width + "px"; renderer.domElement.style.height = r.height + "px";
    cam.aspect = r.width / Math.max(1, r.height); cam.updateProjectionMatrix();
    flames.material.uniforms.scale.value = smoke.material.uniforms.scale.value = r.height * 0.9;
  }
  function frame(now) {
    if (!running) return;
    const dt = Math.min(0.1, (now - last) / 1000 || 0); last = now;
    if (playing) {
      ft += dt; acc += dt * SPEED;
      while (acc >= 1) { acc -= 1; tick(); }
      updateRain(dt);
    }
    cam.updateMatrixWorld();
    updateParticles();
    renderer.render(scene, cam);
    raf = requestAnimationFrame(frame);
  }
  // controls
  const $ = (id) => document.getElementById(id);
  const sync = () => { $("vWind").textContent = P.wind + " m/s"; $("vRain").textContent = P.rain + " mm/h"; };
  document.querySelectorAll("[data-fseason]").forEach((b) => b.addEventListener("click", () => setSeason(b.dataset.fseason)));
  $("fDry").addEventListener("click", () => setOverlay(!overlay));
  $("fWind").addEventListener("input", (e) => { P.wind = +e.target.value; sync(); });
  $("fRain").addEventListener("input", (e) => { P.rain = +e.target.value; sync(); });
  $("fPlay").addEventListener("click", () => { playing = !playing; $("fPlay").textContent = playing ? "Pause" : "Play"; $("fPlay").setAttribute("aria-pressed", String(playing)); });
  $("fReset").addEventListener("click", () => built && reset());
  document.querySelectorAll("[data-ftime]").forEach((b) => b.addEventListener("click", () => {
    const n = b.dataset.ftime === "night";
    if (!n && built) { let k = 0; while (cells.some((c) => c.state === 1) && k++ < 600) tick(); }
    setNight(n);
  }));
  sync();
  window.addEventListener("resize", resize);
  window.fireTab = {
    get scene() { return scene; },
    start() { if (!built) build(); running = true; resize(); last = performance.now(); raf = requestAnimationFrame(frame); },
    stop() { running = false; cancelAnimationFrame(raf); },
    // for the stills: run the fire on to a time, set the sliders, the view and the hour
    still(o) {
      running = false; cancelAnimationFrame(raf);
      if (o.season !== undefined) { P.season = o.season; document.querySelectorAll("[data-fseason]").forEach((b) => b.setAttribute("aria-pressed", String(b.dataset.fseason === o.season))); }
      if (o.overlay !== undefined) setOverlay(o.overlay);
      if (o.wind !== undefined) { P.wind = o.wind; $("fWind").value = o.wind; }
      if (o.rain !== undefined) { P.rain = o.rain; $("fRain").value = o.rain; }
      sync();
      if (o.reset) reset();
      if (o.torch) { const c = cells.filter((c) => c.fuel === "grass" && !c.tree).reduce((b, c) => (Math.hypot(c.x - o.torch[0], c.z - o.torch[1]) < Math.hypot(b.x - o.torch[0], b.z - o.torch[1]) ? c : b)); ignite(c, true); }
      for (let i = 0; i < (o.ticks || 0); i++) tick();
      if (o.night !== undefined) setNight(o.night);
      if (o.view) { Object.assign(orbitSt, o.view); if (o.view.t) orbitSt.t = new THREE.Vector3(...o.view.t); applyOrbit(); }
      if (o.ft !== undefined) ft = o.ft;
      if (o.rainY) updateRain(0.5);
      playing = false; $("fPlay").textContent = "Play";
      cam.updateMatrixWorld(); updateParticles(); renderer.render(scene, cam);
      const g = cells.filter((c) => c.fuel === "grass" && !c.tree).map((c) => c.dry);
      return { burning: cells.filter((c) => c.state === 1).length, burnt: cells.filter((c) => c.state === 2).length, lit: litCount, t: simT,
        season: P.season, start: reset.start, grassDryMax: +Math.max(...g).toFixed(2), grassCatch: +(g.filter((d) => d >= DRY_MIN).length / g.length).toFixed(2) };
    },
  };
})();
