// player.js: the PLAYER tab of docs/mockups/multiplayer.html (openspec/changes/multiplayer,
// task 5.2, design 14.5): another player on a patch of the fields, by day and lit at night,
// named when near and not when far. One 3D view; its renderer is made when the tab first opens.
//
// - Ground, trees and lamp posts as scene-kit.js builds them (the game's cell, atlas tiles,
//   settlement timber and iron), every lamp lighting what is near it per pixel.
// - The other player is M5's townsperson (figure-kit.js), the same figure the game draws for its
//   NPCs and, in the game, for another player (multiplayer design 24).
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
  // M5's townsperson, the figure the game draws for its NPCs and for another player
  // (figure-kit.js, copied from npcs.html), rolled from the Fields folk's palette for player 1.
  const spec = FK.genSpec(1, 1, FK_FIELDS);
  spec.mood = "happy";
  const F = FK.friend(spec); FK.setExprOn(F, spec.mood, true); FK.liveFace(F, 0);
  F.g.traverse(o => { if (o.isMesh) { o.castShadow = true; o.receiveShadow = true; } });
  const fig = new THREE.Group(); fig.add(F.g); scene.add(fig);
  const [legL, legR] = F.legs, [armL, armR] = F.arms;

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
    FK.liveFace(F, dt);
    if (!moving && !dirty && t - shownAt > 3000) return;   // the first seconds draw while the textures load
    dirty = false;
    placeCamera(); renderer.render(scene, camera); placeTag();
  }
  tagEl.innerHTML = '<span class="pt">' + window.ptSvg(NAME) + "</span>";

  window.PLAYER3D = {
    // the lamp uniforms are the page's (scene-kit.js), shared with the CHARACTER tab: take them back
    show() { if (!renderer) makeRenderer(); PBD.setLights(LIGHTS); PBD.LU.uLamp.value = ST.night ? 0.85 : 0; if (!running) { running = true; dirty = true; last = shownAt = performance.now(); raf = requestAnimationFrame(frame); } },
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
