// Survival mockup, Weapons tab (survival-mode design 16): the stone spear, the bow and an arrow, the
// bronze hunting axe and the bronze sword standing at true size beside a 1.8 m figure on 2.83 m hex
// cells, by day and at night by a campfire. Each is one fitted, smooth-shaded form with its own pixel
// texture; a binding or a wrap is painted in, not stuck on. Drag to turn; nothing turns on its own.
(function () {
  "use strict";
  const S = () => window.SurvivalCommon;
  let built = false, running = false, raf = 0, night = false, last = 0, ft = 0;
  let renderer, scene, cam, st, apply, stage, sun, hemi, moon, flames, labels = [];
  const LOOKS = {
    all: { t: [1.7, 0.9, 0], r: 5.2, pol: 1.25, az: 0.25 },
    spear: { t: [1.1, 1.75, 0], r: 0.9, pol: 1.3, az: 0.5 },
    bow: { t: [1.85, 0.8, 0], r: 2.0, pol: 1.4, az: 0.2 },
    axe: { t: [2.75, 0.4, 0], r: 0.8, pol: 1.2, az: 0.6 },
    sword: { t: [3.45, 0.55, 0], r: 1.15, pol: 1.25, az: 0.45 },
  };
  const tx = (n, fn, rep) => S().tex(n, fn, rep);
  const H = (x, y, s) => S().hash(x, y, s);
  const mat = (o) => S().firelit(new THREE.MeshPhongMaterial(Object.assign({ shininess: 8, specular: 0x111111 }, o)));
  const lathe = (pts, seg) => new THREE.LatheGeometry(pts.map(([r, y]) => new THREE.Vector2(r, y)), seg || 16);

  // ---- textures, 16 or 32 px, in the game's palette
  const woodT = () => tx(32, (x, y) => { const g = (x * 3 + Math.floor(y / 4)) % 9 < 2 ? 0.78 : 1; const v = (150 + 30 * H(x, y >> 2, 1)) * g; return [v, v * 0.7, v * 0.44]; });
  const shaftT = () => tx(32, (x, y) => { // a spear shaft along v: fibre wrap near the top
    if (y < 4) { const b = (x + y) % 4 < 2 ? 1 : 0.8; return [206 * b, 180 * b, 112 * b]; }
    const g = x % 6 < 1 ? 0.8 : 1; const v = (156 + 26 * H(x, y >> 3, 2)) * g; return [v, v * 0.7, v * 0.45]; });
  const stoneT = () => tx(16, (x, y) => { const f = H(x >> 1, y >> 1, 3); const v = 120 + 70 * f + (H(x, y, 4) > 0.85 ? 30 : 0); return [v * 0.95, v * 0.95, v]; });
  const bronzeT = () => tx(16, (x, y) => { const f = H(x >> 2, y >> 2, 5); const d = H(x, y, 6) > 0.92 ? 0.9 : 1; const v = (0.94 + 0.08 * f) * d; return [226 * v, 160 * v, 82 * v]; });
  const leatherT = () => tx(16, (x, y) => { const band = (x + y * 2) % 8 < 2 ? 0.7 : 1; const v = (0.9 + 0.2 * H(x, y, 7)) * band; return [120 * v, 76 * v, 44 * v]; });
  const bowT = () => tx(32, (x, y) => { // u along the bow: a leather grip in the middle
    if (x >= 14 && x <= 18) { const b = (y + x) % 3 === 0 ? 0.75 : 1; return [118 * b, 74 * b, 42 * b]; }
    const g = (y * 5 + Math.floor(x / 3)) % 7 < 1 ? 0.8 : 1; const v = (170 + 30 * H(x >> 2, y, 8)) * g; return [v, v * 0.72, v * 0.46]; });
  const featherT = () => tx(16, (x, y) => { const v = x === 8 ? 0.7 : 0.95 + 0.1 * H(x, y >> 1, 9); return [236 * v, 230 * v, 216 * v]; });
  const clothT = () => tx(16, (x, y) => { const v = 0.88 + 0.2 * H(x, y, 10) - ((x + y) % 5 === 0 ? 0.06 : 0); return [255 * v, 255 * v, 255 * v]; });
  const grassT = () => tx(16, (x, y) => { const h = H(x, y, 11); const v = h > 0.82 ? 1.2 : h < 0.2 ? 0.82 : 1; return [86 * v, 132 * v, 58 * v]; });
  const barkT = () => tx(16, (x, y) => { const s = (x * 5 + (y >> 2)) % 4 === 0 ? 0.7 : 1; const v = (100 + 40 * H(x, y >> 1, 12)) * s; return [v, v * 0.7, v * 0.48]; });

  // taper a tube's rings by f(t), and flatten across by k
  function taperTube(curve, n, r, f, k) {
    const g = new THREE.TubeGeometry(curve, n, r, 10, false);
    const p = g.attributes.position, v = new THREE.Vector3();
    for (let i = 0; i <= n; i++) {
      const c = curve.getPointAt(i / n), s = f(i / n);
      for (let j = 0; j <= 10; j++) {
        const idx = i * 11 + j;
        v.fromBufferAttribute(p, idx).sub(c).multiplyScalar(s); v.z *= k;
        p.setXYZ(idx, c.x + v.x, c.y + v.y, c.z + v.z);
      }
    }
    g.computeVertexNormals();
    return g;
  }
  function extrude(shape, depth, bevel, size, segs, rep) {
    const g = new THREE.ExtrudeGeometry(shape, { depth, bevelEnabled: true, bevelThickness: bevel, bevelSize: size, bevelSegments: segs, curveSegments: 18 });
    g.translate(0, 0, -depth / 2);
    const uv = g.attributes.uv; for (let i = 0; i < uv.count; i++) uv.setXY(i, uv.getX(i) * rep, uv.getY(i) * rep);
    const w = S().weld(g); w.computeVertexNormals(); return w;
  }

  function spear() {
    const g = new THREE.Group();
    const sp = []; for (let i = 0; i <= 18; i++) sp.push([0.017 - 0.002 * i / 18, 1.72 * i / 18]);
    const shaft = new THREE.Mesh(lathe([[0, 0], ...sp, [0, 1.74]], 12), mat({ map: shaftT() }));
    shaft.material.map.repeat.set(2, 1); shaft.material.map.offset.set(0, 0.0);
    const head = new THREE.Mesh(lathe([[0, 1.62], [0.016, 1.63], [0.026, 1.69], [0.037, 1.78], [0.032, 1.87], [0.018, 1.95], [0.004, 1.995], [0, 2.0]], 6), mat({ map: stoneT(), shininess: 20 }));
    head.scale.z = 0.38;
    g.add(shaft, head);
    return g;
  }
  function bow() {
    const g = new THREE.Group();
    const pts = []; for (let i = 0; i <= 20; i++) { const t = i / 20, y = 1.5 * t; let x = -0.13 * Math.sin(Math.PI * t); if (t < 0.08) x += 0.04 * (0.08 - t) / 0.08; if (t > 0.92) x += 0.04 * (t - 0.92) / 0.08; pts.push(new THREE.Vector3(x, y, 0)); }
    const curve = new THREE.CatmullRomCurve3(pts);
    const limb = new THREE.Mesh(taperTube(curve, 80, 0.02, (t) => 0.35 + 0.65 * Math.pow(Math.sin(Math.PI * t), 0.6), 0.55), mat({ map: bowT() }));
    const string = new THREE.Mesh(taperTube(new THREE.LineCurve3(pts[0].clone().add(new THREE.Vector3(0.004, 0.012, 0)), pts[20].clone().add(new THREE.Vector3(0.004, -0.012, 0))), 4, 0.0022, () => 1, 1), mat({ color: 0xe6dcc0 }));
    g.add(limb, string);
    // an arrow standing beside it
    const a = new THREE.Group();
    const shaft = new THREE.Mesh(lathe([[0, 0], [0.0055, 0.005], [0.0055, 0.7], [0, 0.71]], 8), mat({ map: woodT() }));
    const head = new THREE.Mesh(lathe([[0, 0.69], [0.007, 0.695], [0.013, 0.715], [0.009, 0.735], [0, 0.755]], 6), mat({ map: stoneT() }));
    head.scale.z = 0.4;
    a.add(shaft, head);
    const fs = new THREE.Shape(); fs.moveTo(0, 0); fs.quadraticCurveTo(0.022, 0.03, 0.02, 0.1); fs.quadraticCurveTo(0.01, 0.11, 0, 0.12); fs.lineTo(0, 0);
    const fg = new THREE.ShapeGeometry(fs, 8); const uv = fg.attributes.uv; for (let i = 0; i < uv.count; i++) uv.setXY(i, uv.getX(i) * 8, uv.getY(i) * 8);
    for (let k = 0; k < 3; k++) { const f = new THREE.Mesh(fg, mat({ map: featherT(), side: THREE.DoubleSide })); f.position.y = 0.04; f.rotation.y = k * 2.094; a.add(f); }
    a.position.set(0.35, 0, 0.05);
    g.add(a);
    return g;
  }
  function huntingAxe() {
    const g = new THREE.Group();
    const curve = new THREE.CatmullRomCurve3([new THREE.Vector3(0, 0, 0), new THREE.Vector3(0.012, 0.2, 0), new THREE.Vector3(-0.006, 0.42, 0), new THREE.Vector3(0, 0.6, 0)]);
    const handle = new THREE.Mesh(taperTube(curve, 30, 0.017, (t) => (t < 0.05 ? 1.15 : 0.85 + 0.15 * t), 0.75), mat({ map: tx(32, (x, y) => (x < 7 ? [[112, 72, 42], [96, 60, 36]][(x + y) % 3 === 0 ? 1 : 0] : [160 + 20 * H(x >> 2, y, 13), (160 + 20 * H(x >> 2, y, 13)) * 0.7, (160 + 20 * H(x >> 2, y, 13)) * 0.45])) }));
    const s = new THREE.Shape();
    s.moveTo(-0.03, 0.6); s.lineTo(-0.03, 0.51); s.lineTo(0.02, 0.5);
    s.quadraticCurveTo(0.07, 0.48, 0.1, 0.43); s.quadraticCurveTo(0.14, 0.47, 0.15, 0.55);
    s.quadraticCurveTo(0.155, 0.6, 0.14, 0.64); s.quadraticCurveTo(0.08, 0.62, 0.03, 0.62); s.lineTo(-0.03, 0.6);
    const head = new THREE.Mesh(extrude(s, 0.016, 0.008, 0.005, 3, 3), mat({ map: bronzeT(), shininess: 50, specular: 0x554422 }));
    head.scale.z = 1;
    g.add(handle, head);
    return g;
  }
  function sword() {
    const g = new THREE.Group();
    const pommel = new THREE.Mesh(lathe([[0, 0], [0.016, 0.004], [0.024, 0.018], [0.02, 0.034], [0.012, 0.042], [0.011, 0.046], [0, 0.046]], 14), mat({ map: bronzeT(), shininess: 50, specular: 0x554422 }));
    pommel.scale.z = 0.7;
    const grip = new THREE.Mesh(lathe([[0, 0.04], [0.013, 0.042], [0.015, 0.1], [0.013, 0.165], [0, 0.17]], 12), mat({ map: leatherT() }));
    const gs = new THREE.Shape();
    gs.moveTo(-0.095, 0.172); gs.quadraticCurveTo(-0.05, 0.165, 0, 0.165); gs.quadraticCurveTo(0.05, 0.165, 0.095, 0.172);
    gs.quadraticCurveTo(0.1, 0.18, 0.093, 0.19); gs.quadraticCurveTo(0.05, 0.185, 0, 0.188); gs.quadraticCurveTo(-0.05, 0.185, -0.093, 0.19); gs.quadraticCurveTo(-0.1, 0.18, -0.095, 0.172);
    const guard = new THREE.Mesh(extrude(gs, 0.018, 0.006, 0.004, 3, 6), mat({ map: bronzeT(), shininess: 50, specular: 0x554422 }));
    const bs = new THREE.Shape();
    bs.moveTo(-0.023, 0.186); bs.lineTo(0.023, 0.186); bs.quadraticCurveTo(0.02, 0.5, 0.017, 0.74); bs.quadraticCurveTo(0.012, 0.85, 0, 0.9);
    bs.quadraticCurveTo(-0.012, 0.85, -0.017, 0.74); bs.quadraticCurveTo(-0.02, 0.5, -0.023, 0.186);
    const blade = new THREE.Mesh(extrude(bs, 0.003, 0.0035, 0.006, 3, 2.2), mat({ map: tx(16, (x, y) => { const f = x === 7 || x === 8 ? 0.84 : x < 2 || x > 13 ? 1.12 : 1; const v = (0.95 + 0.06 * H(x >> 2, y >> 2, 14)) * f; return [236 * v, 180 * v, 100 * v]; }), shininess: 70, specular: 0x665533 }));
    g.add(pommel, grip, guard, blade);
    return g;
  }
  function figure() {
    const g = new THREE.Group();
    const cloth = clothT();
    const colour = (geo, fn) => {
      const p = geo.attributes.position, c = [], v = new THREE.Vector3();
      for (let i = 0; i < p.count; i++) { v.fromBufferAttribute(p, i); c.push(...fn(v)); }
      geo.setAttribute("color", new THREE.Float32BufferAttribute(c, 3));
      return geo;
    };
    const skin = [0.86, 0.66, 0.5], tunic = [0.3, 0.46, 0.5], belt = [0.36, 0.24, 0.14], trousers = [0.42, 0.33, 0.24], boots = [0.24, 0.17, 0.12], hair = [0.3, 0.2, 0.12];
    const body = lathe([[0, 0.84], [0.15, 0.86], [0.172, 0.95], [0.162, 1.05], [0.155, 1.15], [0.172, 1.3], [0.19, 1.41], [0.17, 1.47], [0.09, 1.51], [0.056, 1.55], [0.06, 1.6], [0.088, 1.64], [0.1, 1.7], [0.094, 1.75], [0.066, 1.79], [0, 1.805]], 20);
    body.scale(1, 1, 0.72);
    colour(body, (v) => (v.y > 1.52 ? (v.y > 1.73 || (v.z < -0.03 && v.y > 1.62) ? hair : skin) : v.y > 0.95 && v.y < 1.0 ? belt : v.y > 0.88 ? tunic : trousers));
    const mesh = (geo) => new THREE.Mesh(geo, mat({ map: cloth, vertexColors: true }));
    g.add(mesh(body));
    [-1, 1].forEach((sd) => {
      const leg = lathe([[0, 0], [0.058, 0.0], [0.064, 0.05], [0.052, 0.12], [0.055, 0.45], [0.07, 0.62], [0.088, 0.82], [0.075, 0.93], [0, 0.95]], 12);
      leg.scale(1, 1, 0.9); leg.translate(sd * 0.085, 0, 0);
      colour(leg, (v) => (v.y < 0.11 ? boots : trousers));
      g.add(mesh(leg));
      const arm = lathe([[0, -0.02], [0.03, 0.0], [0.04, 0.06], [0.034, 0.12], [0.036, 0.33], [0.043, 0.52], [0.05, 0.62], [0.042, 0.68], [0, 0.71]], 12);
      colour(arm, (v) => (v.y < 0.13 ? skin : tunic));
      const am = mesh(arm); am.position.set(sd * 0.205, 0.76, 0); am.rotation.z = sd * 0.1;
      g.add(am);
    });
    return g;
  }
  function campfire() {
    const g = new THREE.Group();
    for (let k = 0; k < 9; k++) {
      const geo = S().weld(new THREE.IcosahedronGeometry(0.11, 1)), p = geo.attributes.position, v = new THREE.Vector3();
      for (let i = 0; i < p.count; i++) { v.fromBufferAttribute(p, i); const n = v.clone().normalize(); v.multiplyScalar(1 + 0.22 * Math.sin(n.x * 4 + k) * Math.cos(n.z * 3 - k)); v.y *= 0.7; p.setXYZ(i, v.x, v.y, v.z); }
      geo.computeVertexNormals();
      const m = new THREE.Mesh(geo, mat({ map: stoneT() }));
      const a = k / 9 * 6.283; m.position.set(Math.cos(a) * 0.38, 0.05, Math.sin(a) * 0.38); m.rotation.y = a * 3;
      g.add(m);
    }
    for (let k = 0; k < 3; k++) {
      const log = new THREE.Mesh(lathe([[0, -0.36], [0.05, -0.35], [0.06, -0.2], [0.055, 0.1], [0.05, 0.3], [0.035, 0.36], [0, 0.37]], 10), mat({ map: barkT() }));
      log.rotation.z = 1.25; log.rotation.y = k * 2.09; log.position.y = 0.14;
      g.add(log);
    }
    return g;
  }
  function groundHex() {
    const pos = [], uv = [], col = [], idx = [], R = 2.83 / Math.sqrt(3);
    for (let q = -3; q <= 3; q++) for (let r = -3; r <= 3; r++) {
      if (Math.abs(q + r) > 3) continue;
      const x = 2.83 * (q + r / 2) + 1.6, z = 2.83 * 0.866 * r, b = pos.length / 3, sh = 0.84 + 0.22 * H(q, r, 15);
      pos.push(x, 0, z); uv.push(x / 2.83 * 4, z / 2.83 * 4); col.push(sh, sh, sh);
      for (let k = 0; k < 6; k++) { const a = Math.PI / 6 + k * Math.PI / 3, X = x + R * Math.cos(a), Z = z + R * Math.sin(a); pos.push(X, 0, Z); uv.push(X / 2.83 * 4, Z / 2.83 * 4); col.push(sh * 0.9, sh * 0.9, sh * 0.9); }
      for (let k = 0; k < 6; k++) idx.push(b, b + 1 + ((k + 1) % 6), b + 1 + k);
      for (let k = 0; k < 6; k++) { const a0 = b + 1 + k, a1 = b + 1 + ((k + 1) % 6), s0 = pos.length / 3;
        pos.push(pos[a0 * 3], 0, pos[a0 * 3 + 2], pos[a1 * 3], 0, pos[a1 * 3 + 2], pos[a1 * 3], -1, pos[a1 * 3 + 2], pos[a0 * 3], -1, pos[a0 * 3 + 2]);
        uv.push(0, 0, 1, 0, 1, -1, 0, -1); for (let m = 0; m < 4; m++) col.push(0.62, 0.45, 0.33); idx.push(s0, s0 + 2, s0 + 1, s0, s0 + 3, s0 + 2); }
    }
    const geo = new THREE.BufferGeometry();
    geo.setAttribute("position", new THREE.Float32BufferAttribute(pos, 3)); geo.setAttribute("uv", new THREE.Float32BufferAttribute(uv, 2)); geo.setAttribute("color", new THREE.Float32BufferAttribute(col, 3));
    geo.setIndex(idx); geo.computeVertexNormals();
    return new THREE.Mesh(geo, mat({ map: grassT(), vertexColors: true, shininess: 0, specular: 0 }));
  }

  function setNight(n) {
    night = n;
    document.querySelectorAll("[data-wtime]").forEach((b) => b.setAttribute("aria-pressed", String((b.dataset.wtime === "night") === n)));
    if (!built) return;
    scene.background = new THREE.Color(n ? 0x050a14 : 0x9cc4e4);
    sun.intensity = n ? 0 : 0.95; hemi.intensity = n ? 0.12 : 0.55; moon.intensity = n ? 0.12 : 0;
    hemi.color.set(n ? 0x6070a0 : 0xd8e8ff); hemi.groundColor.set(n ? 0x101418 : 0x6a5a40);
    flames.visible = n;
  }
  function build() {
    stage = document.getElementById("wStage");
    renderer = new THREE.WebGLRenderer({ antialias: true, preserveDrawingBuffer: true });
    renderer.setPixelRatio(Math.min(2, devicePixelRatio)); renderer.outputEncoding = THREE.sRGBEncoding;
    stage.appendChild(renderer.domElement);
    scene = new THREE.Scene();
    cam = new THREE.PerspectiveCamera(40, 1, 0.02, 200);
    sun = new THREE.DirectionalLight(0xfff1d8, 0.95); sun.position.set(-3, 6, 4); scene.add(sun);
    moon = new THREE.DirectionalLight(0x9fb4e0, 0.12); moon.position.set(3, 6, -2); scene.add(moon);
    hemi = new THREE.HemisphereLight(0xd8e8ff, 0x6a5a40, 0.55); scene.add(hemi);
    scene.add(groundHex());
    const fig = figure(); scene.add(fig);
    const items = [["spear", spear(), 1.1, "Stone spear · 2.0 m", 2.05], ["bow", bow(), 1.8, "Bow 1.5 m, arrow 0.75 m", 1.58], ["axe", huntingAxe(), 2.75, "Bronze hunting axe · 0.6 m", 0.7], ["sword", sword(), 3.45, "Bronze sword · 0.9 m", 0.96]];
    items.forEach(([k, o, x, text, top]) => { o.position.x = x; o.rotation.y = 0.35; scene.add(o); labels.push(mkLabel(text, new THREE.Vector3(x, top, 0))); });
    labels.push(mkLabel("Figure · 1.8 m", new THREE.Vector3(0, 1.86, 0)));
    const fire = campfire(); fire.position.set(1.8, 0, 1.5); scene.add(fire);
    // flames: a few soft additive cards rising from the logs
    const fg = new THREE.BufferGeometry(), N = 60;
    fg.setAttribute("position", new THREE.Float32BufferAttribute(new Float32Array(N * 3), 3));
    fg.setAttribute("size", new THREE.Float32BufferAttribute(new Float32Array(N), 1));
    fg.setAttribute("col", new THREE.Float32BufferAttribute(new Float32Array(N * 4), 4));
    flames = new THREE.Points(fg, new THREE.ShaderMaterial({
      uniforms: { scale: { value: 600 } },
      vertexShader: "attribute float size; attribute vec4 col; varying vec4 vC; uniform float scale; void main(){ vC = col; vec4 mv = modelViewMatrix * vec4(position,1.0); gl_PointSize = size * scale / -mv.z; gl_Position = projectionMatrix * mv; }",
      fragmentShader: "varying vec4 vC; void main(){ vec2 d = gl_PointCoord - vec2(0.5, 0.55); float w = mix(0.18, 0.5, gl_PointCoord.y); float r = length(vec2(d.x * 0.5 / w, d.y)) * 2.0; if (r > 1.0) discard; float a = vC.a * (1.0 - r * r); vec3 c = mix(vC.rgb, vec3(1.0, 0.95, 0.7), (1.0 - r) * 0.6); gl_FragColor = vec4(c * a, a); }",
      transparent: true, depthWrite: false, blending: THREE.AdditiveBlending,
    }));
    flames.frustumCulled = false; scene.add(flames);
    st = Object.assign({ min: 0.4, max: 20 }, LOOKS.all, { t: new THREE.Vector3(...LOOKS.all.t) });
    apply = S().orbit(cam, renderer.domElement, st);
    built = true;
    setNight(night);
    resize();
  }
  function mkLabel(text, at) { const el = document.createElement("div"); el.className = "wlabel"; el.textContent = text; stage.appendChild(el); return { el, at }; }
  function look(k) { const L = LOOKS[k]; Object.assign(st, L, { t: new THREE.Vector3(...L.t) }); apply(); document.querySelectorAll("[data-wlook]").forEach((b) => b.setAttribute("aria-pressed", String(b.dataset.wlook === k))); }
  function resize() {
    if (!built) return;
    const r = stage.getBoundingClientRect();
    renderer.setSize(r.width, r.height, false);
    renderer.domElement.style.width = r.width + "px"; renderer.domElement.style.height = r.height + "px";
    cam.aspect = r.width / Math.max(1, r.height); cam.updateProjectionMatrix();
    flames.material.uniforms.scale.value = r.height * 0.9;
  }
  function draw() {
    cam.updateMatrixWorld();
    const U = S().fireUniforms;
    if (night) {
      const v = new THREE.Vector3(1.8, 0.7, 1.5).applyMatrix4(cam.matrixWorldInverse);
      U.fireL.value[0].set(v.x, v.y, v.z, 4.2 * (0.9 + 0.1 * Math.sin(ft * 8) * Math.sin(ft * 5.1)));
      U.fireN.value = 1; U.fireC.value.setRGB(1.0, 0.55, 0.22);
      const p = flames.geometry.attributes, n = p.size.count;
      for (let i = 0; i < n; i++) {
        const ph = (ft * (0.8 + 0.4 * H(i, 1, 20)) + H(i, 2, 20)) % 1, life = 1 - ph;
        p.position.setXYZ(i, 1.8 + (H(i, 3, 20) - 0.5) * 0.4 * life, 0.12 + ph * 0.7, 1.5 + (H(i, 4, 20) - 0.5) * 0.4 * life);
        p.size.setX(i, 0.16 + 0.22 * life);
        p.col.setXYZW(i, 1, 0.3 + 0.4 * life, 0.06 + 0.12 * life * life, 0.9 * life);
      }
      p.position.needsUpdate = p.size.needsUpdate = p.col.needsUpdate = true;
    } else U.fireN.value = 0;
    renderer.render(scene, cam);
    const r = stage.getBoundingClientRect(), v = new THREE.Vector3();
    labels.forEach((L) => {
      v.copy(L.at).project(cam);
      const on = v.z < 1 && Math.abs(v.x) < 1.1 && Math.abs(v.y) < 1.1;
      L.el.style.display = on ? "" : "none";
      L.el.style.left = ((v.x + 1) / 2 * r.width) + "px"; L.el.style.top = ((1 - v.y) / 2 * r.height) + "px";
    });
  }
  function frame(now) {
    if (!running) return;
    const dt = Math.min(0.1, (now - last) / 1000 || 0); last = now; ft += dt;
    draw();
    raf = requestAnimationFrame(frame);
  }
  document.querySelectorAll("[data-wtime]").forEach((b) => b.addEventListener("click", () => setNight(b.dataset.wtime === "night")));
  document.querySelectorAll("[data-wlook]").forEach((b) => b.addEventListener("click", () => built && look(b.dataset.wlook)));
  window.addEventListener("resize", resize);
  window.weaponsTab = {
    start() { if (!built) build(); running = true; resize(); last = performance.now(); raf = requestAnimationFrame(frame); },
    stop() { running = false; cancelAnimationFrame(raf); },
    still(o) { running = false; cancelAnimationFrame(raf); if (o.night !== undefined) setNight(o.night); if (o.look) look(o.look); if (o.ft !== undefined) ft = o.ft; resize(); draw(); },
  };
})();
