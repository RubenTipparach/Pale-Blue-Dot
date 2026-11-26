// Survival mockup, Fire tab: wildfire design 3, 5 and 7 on a patch of hex cells (2.83 m apart, 1 m
// layers) at night in the dry season. Grass, broadleaf and conifer trees, a stream, a dug break and a
// player's tilled field (the fire never takes the player's). Every second of fire time each burning
// cell tries its six neighbours. Shown at four seconds of fire a real second. Built when the tab first
// opens; its loop stops when the tab closes.
(function () {
  "use strict";
  const S = () => window.SurvivalCommon;
  const CELL = 2.83, RAD = 19, SPEED = 4;
  let built = false, running = false, playing = true, raf = 0, night = true;
  let renderer, scene, cam, orbitSt, applyOrbit, ground, groundCol, cells = [], byKey = new Map();
  let trunks, crownsB, crownsC, flames, smoke, rain, water, moon, hemi, sun, stage, stat;
  let simT = 0, acc = 0, ft = 0, rng, last = 0;
  const DIRS = [[1, 0], [1, -1], [0, -1], [-1, 0], [-1, 1], [0, 1]];
  const P = { danger: 0.8, wind: 4, rain: 0 };

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
      else if (H(q, r, 5) < 0.42) c.tree = { kind: c.h >= 2 || H(q, r, 6) < 0.35 ? "conifer" : "broad", s: 0.8 + 0.45 * H(q, r, 7), rot: H(q, r, 8) * 6.28, ox: (H(q, r, 9) - 0.5) * 0.8, oz: (H(q, r, 10) - 0.5) * 0.8 };
      byKey.set(q + "," + r, c);
      cells.push(c);
    }
  }
  const nbrs = (c) => DIRS.map(([dq, dr]) => byKey.get((c.q + dq) + "," + (c.r + dr))).filter(Boolean);
  const burnable = (c) => c.fuel === "grass";

  // ---- colours a cell's top takes
  function topColour(c) {
    if (c.fuel === "water") return [0.16, 0.3, 0.42];
    if (c.fuel === "soil") return [0.5, 0.36, 0.24];
    if (c.fuel === "field") return [0.44, 0.3, 0.18];
    if (c.state === 1) return [0.16, 0.12, 0.1];
    if (c.state === 2) return c.tree ? [0.2, 0.19, 0.18] : [0.36, 0.35, 0.33];
    const d = P.danger;
    return [0.55 + 0.25 * d, 0.62 + 0.06 * d, 0.32 - 0.04 * d];
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
    const snag = c.state === 2 ? 0.72 : 1;
    m4.compose(pos, q4, new THREE.Vector3(trunkS, trunkS * (t.kind === "conifer" ? 1.3 : 1) * snag, trunkS));
    trunks.setMatrixAt(t.ti, m4);
    const ch = c.state === 2 ? 0.2 : c.state === 1 && c.left < 60 ? 0.35 : 1;
    trunks.instanceColor.setXYZ(t.ti, ch, ch, ch);
    const crown = t.kind === "broad" ? crownsB : crownsC;
    const gone = c.state === 2 || (c.state === 1 && c.left < 60);
    m4.compose(pos, q4, gone ? new THREE.Vector3(0, 0, 0) : new THREE.Vector3(s, s, s));
    crown.setMatrixAt(t.ci, m4);
    if (c.state === 1) crown.instanceColor.setXYZ(t.ci, 0.5, 0.3, 0.18); else crown.instanceColor.setXYZ(t.ci, 1, 1, 1);
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
      const crown = c.tree && c.state === 1 && c.left >= 60;
      const ember = c.state === 2;
      const per = ember ? 2 : crown ? 16 : 7;
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
          y = c.h + (ember ? 0.15 : 0.35 + ph * 2.0); size = ember ? 0.6 : 1.7 * (0.45 + 0.75 * life);
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
      const w = c.state === 2 ? 0.25 : c.tree && c.left >= 60 ? 3 : 1;
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
    stat.textContent = `burning ${cells.filter((c) => c.state === 1).length} · burnt ${cells.filter((c) => c.state === 2).length} cells · fire time ${Math.round(simT)} s (x${SPEED})`;
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

  // ---- the rule (wildfire design 5 and 7)
  function ignite(c) {
    if (!c || !burnable(c) || c.state !== 0) return;
    c.state = 1; c.left = c.tree ? 120 : 12; c.damp = P.danger < 0.35;
    if (c.damp) c.left = 3;
    paintCell(c); if (c.tree) setTree(c);
  }
  function tick() {
    simT += 1;
    const wind = P.wind, rain = P.rain;
    const lit = [];
    const burning = cells.filter((c) => c.state === 1);
    burning.forEach((c) => {
      if (!c.damp && rain < 2 && P.danger >= 0.2) nbrs(c).forEach((n) => {
        if (n.state !== 0 || !burnable(n)) return;
        let lam = (n.tree ? (n.tree.kind === "conifer" ? 0.07 : 0.05) : 0.25) * P.danger;
        const dx = n.x - c.x, dz = n.z - c.z, cos = dx / Math.hypot(dx, dz);
        lam *= Math.max(0.2, 1 + 0.25 * wind * Math.max(0, cos) - 0.15 * wind * Math.max(0, -cos));
        lam *= n.h > c.h ? 1 + 0.15 * (n.h - c.h) : n.h < c.h ? 0.7 : 1;
        lam *= Math.exp(-rain / 0.5);
        if (rng() < 1 - Math.exp(-lam)) lit.push(n);
      });
      c.left -= 1 + rain * 0.5;
      const was = c.left;
      if (c.tree && was < 60 && was + 1 + rain * 0.5 >= 60) setTree(c);
      if (c.left <= 0) { c.state = 2; c.burnt = simT; paintCell(c); if (c.tree) setTree(c); }
    });
    lit.forEach(ignite);
  }
  function reset() {
    rng = S().rng(20261126); simT = 0; acc = 0;
    cells.forEach((c) => { c.state = 0; c.left = 0; c.burnt = 0; paintCell(c); if (c.tree) setTree(c); });
    const start = cells.reduce((b, c) => (burnable(c) && Math.hypot(c.x + 34, c.z + 4) < Math.hypot(b.x + 34, b.z + 4) ? c : b), cells.find(burnable));
    ignite(start); nbrs(start).forEach(ignite);
    for (let i = 0; i < 15; i++) tick();
  }

  function setNight(n) {
    night = n;
    document.querySelectorAll("[data-ftime]").forEach((b) => b.setAttribute("aria-pressed", String((b.dataset.ftime === "night") === n)));
    if (!built) return;
    scene.background = new THREE.Color(n ? 0x050a14 : 0x9cc4e4);
    scene.fog = new THREE.Fog(n ? 0x050a14 : 0x9cc4e4, 70, 190);
    moon.intensity = n ? 0.16 : 0; hemi.intensity = n ? 0.16 : 0.55; sun.intensity = n ? 0 : 0.95;
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
      ignite(best);
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
  const sync = () => { $("vDanger").textContent = P.danger.toFixed(2); $("vWind").textContent = P.wind + " m/s"; $("vRain").textContent = P.rain + " mm/h"; };
  $("fDanger").addEventListener("input", (e) => { P.danger = +e.target.value; sync(); if (built) cells.forEach((c) => c.state === 0 && paintCell(c)); });
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
      if (o.danger !== undefined) { P.danger = o.danger; $("fDanger").value = o.danger; }
      if (o.wind !== undefined) { P.wind = o.wind; $("fWind").value = o.wind; }
      if (o.rain !== undefined) { P.rain = o.rain; $("fRain").value = o.rain; }
      sync();
      if (o.reset) reset();
      for (let i = 0; i < (o.ticks || 0); i++) tick();
      if (o.night !== undefined) setNight(o.night);
      if (o.view) { Object.assign(orbitSt, o.view); if (o.view.t) orbitSt.t = new THREE.Vector3(...o.view.t); applyOrbit(); }
      if (o.ft !== undefined) ft = o.ft;
      if (o.rainY) updateRain(0.5);
      playing = false; $("fPlay").textContent = "Play";
      cam.updateMatrixWorld(); updateParticles(); renderer.render(scene, cam);
      return { burning: cells.filter((c) => c.state === 1).length, burnt: cells.filter((c) => c.state === 2).length, t: simT };
    },
  };
})();
