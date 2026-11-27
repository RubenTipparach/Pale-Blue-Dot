// Survival mockup, Hunt tab (survival-mode design 6, combat C2, task 1.2): a predator's hunt played out
// by the core rule's states and numbers (survival.ron's predators and hunt): it wanders, notices the
// player, stalks low, warns (a crouch and a growl, held at least 1.5 s), lunges, bites, backs off and
// warns again; the player's spear thrust lands on the swing's hit moment (combat C1), the struck
// predator fights back, and below a third of its health it flees. The red fox on grass, the dust
// hound on desert sand, in the herd's blocky family (M26), textured, by day and at night lit by a
// campfire and lanterns that light what is near them. Built when the tab first opens; nothing moves
// until Play.
(function () {
  "use strict";
  const S = () => window.SurvivalCommon;
  const W = () => window.SurvivalWeapons;
  const H = (x, y, s) => S().hash(x, y, s);

  // ---- the beasts: the herd's blocky forms, bevelled, one fur texture tinted per part
  function bevelBox(w, h, d, b) {
    const s = new THREE.Shape(), x = w / 2 - b, y = h / 2 - b;
    s.moveTo(-x, -h / 2); s.lineTo(x, -h / 2); s.quadraticCurveTo(w / 2, -h / 2, w / 2, -y); s.lineTo(w / 2, y);
    s.quadraticCurveTo(w / 2, h / 2, x, h / 2); s.lineTo(-x, h / 2); s.quadraticCurveTo(-w / 2, h / 2, -w / 2, y);
    s.lineTo(-w / 2, -y); s.quadraticCurveTo(-w / 2, -h / 2, -x, -h / 2);
    const g = new THREE.ExtrudeGeometry(s, { depth: d - 2 * b, bevelEnabled: true, bevelThickness: b, bevelSize: 0, bevelSegments: 2, curveSegments: 3 });
    g.translate(0, 0, -(d - 2 * b) / 2);
    const uv = g.attributes.uv; for (let i = 0; i < uv.count; i++) uv.setXY(i, uv.getX(i) * 3, uv.getY(i) * 3);
    g.computeVertexNormals();
    return g;
  }
  const SPECIES = {
    "red-fox": { len: 0.9, h: 0.42, coat: [194, 97, 41], belly: [235, 219, 194], legs: [46, 33, 26], tip: [240, 232, 220], walk: 1.2, run: 7.0 },
    "dust-hound": { len: 1.0, h: 0.55, coat: [199, 163, 107], belly: [230, 209, 168], legs: [117, 92, 61], tip: [120, 95, 62], walk: 1.4, run: 7.0 },
  };
  function fur(c, seed, belly) {
    return S().tex(16, (x, y) => {
      const n = 0.9 + 0.16 * H(x >> 1, y, seed) - ((x * 3 + y) % 9 === 0 ? 0.08 : 0);
      const b = belly && y > 11 ? belly : c;
      return [b[0] * n, b[1] * n, b[2] * n];
    });
  }
  function make(species) {
    const sp = SPECIES[species], L = sp.len, Hh = sp.h, mat = (map) => W().mat({ map, shininess: 4, specular: 0x080808 });
    const g = new THREE.Group(), body = new THREE.Group();
    const coat = mat(fur(sp.coat, 41, sp.belly)), legs = mat(fur(sp.legs, 42)), tipM = mat(fur(sp.tip, 43));
    const torso = new THREE.Mesh(bevelBox(0.62 * L, 0.3 * Hh / 0.42 * 0.42, 0.26 * L, 0.03), coat);
    torso.rotation.y = Math.PI / 2; body.add(torso);
    const head = new THREE.Group();
    const skull = new THREE.Mesh(bevelBox(0.2 * L, 0.18 * L, 0.2 * L, 0.025), coat); head.add(skull);
    const snout = new THREE.Mesh(bevelBox(0.09 * L, 0.08 * L, 0.12 * L, 0.018), tipM); snout.position.set(0, -0.035 * L, 0.14 * L); head.add(snout);
    for (const s of [-1, 1]) { const ear = new THREE.Mesh(bevelBox(0.05 * L, 0.09 * L, 0.03 * L, 0.01), coat); ear.position.set(s * 0.06 * L, 0.12 * L, -0.02 * L); ear.rotation.z = -s * 0.15; head.add(ear); }
    head.position.set(0, 0.12 * Hh / 0.42, 0.36 * L); body.add(head);
    const tail = new THREE.Group();
    const tm = new THREE.Mesh(bevelBox(0.08 * L, 0.08 * L, 0.36 * L, 0.025), coat); tm.position.z = -0.18 * L; tail.add(tm);
    const tt = new THREE.Mesh(bevelBox(0.07 * L, 0.07 * L, 0.08 * L, 0.02), tipM); tt.position.z = -0.38 * L; tail.add(tt);
    tail.position.set(0, 0.05, -0.31 * L); tail.rotation.x = -0.5; body.add(tail);
    const legsG = [];
    for (const [x, z] of [[-1, 1], [1, 1], [-1, -1], [1, -1]]) {
      const hip = new THREE.Group(), leg = new THREE.Mesh(bevelBox(0.07 * L, Hh * 0.55, 0.07 * L, 0.015), legs);
      leg.position.y = -Hh * 0.27; hip.add(leg); hip.position.set(x * 0.08 * L, -0.06, z * 0.22 * L); body.add(hip); legsG.push(hip);
    }
    body.position.y = Hh * 0.62; g.add(body);
    return {
      group: g,
      // crouch 0..1, flinch 0..1, phase rad, speed m/s, lunge -1 gather .. 1 spring, head low 0..1
      pose(o) {
        const c = o.crouch || 0, f = o.flinch || 0, sw = Math.min(0.7, (o.speed || 0) / sp.walk * 0.45);
        legsG.forEach((hip, i) => { const s = (i === 0 || i === 3 ? 1 : -1); hip.rotation.x = Math.sin(o.phase || 0) * sw * s - c * 0.5 * (i < 2 ? -1 : 1); });
        const lunge = o.lunge || 0;
        body.position.y = Hh * 0.62 * (1 - 0.32 * c) + Math.abs(Math.sin(o.phase || 0)) * 0.02 * sw + Math.max(0, lunge) * 0.12;
        // squash on the gather, stretch on the spring
        body.scale.set(1 + 0.06 * Math.max(0, -lunge), 1 - 0.1 * Math.max(0, -lunge) + 0.04 * Math.max(0, lunge), 1 + 0.12 * Math.max(0, lunge));
        body.rotation.x = -0.12 * c + 0.18 * f - 0.15 * Math.max(0, lunge);
        body.position.z = -0.12 * f;
        head.rotation.x = (o.headLow || 0) * 0.45 - 0.5 * f;
        head.position.y = 0.12 * Hh / 0.42 - (o.headLow || 0) * 0.08;
        tail.rotation.x = o.tailDown ? 0.25 : -0.5 + 0.08 * Math.sin((o.t || 0) * 2.2) - 0.25 * c;
      },
    };
  }
  function lantern(at) {
    const g = new THREE.Group(), w = W();
    const post = new THREE.Mesh(new THREE.LatheGeometry([[0, 0], [0.05, 0], [0.045, 0.05], [0.04, 1.7], [0, 1.72]].map(([r, y]) => new THREE.Vector2(r, y)), 8), w.mat({ map: w.barkT ? w.barkT() : w.woodT() }));
    const arm = new THREE.Mesh(w.taperTube(new THREE.CatmullRomCurve3([new THREE.Vector3(0, 1.6, 0), new THREE.Vector3(0.12, 1.72, 0), new THREE.Vector3(0.28, 1.68, 0)]), 8, 0.02, () => 1, 1), w.mat({ map: w.woodT() }));
    const lamp = new THREE.Mesh(new THREE.LatheGeometry([[0, 1.36], [0.06, 1.37], [0.08, 1.45], [0.07, 1.56], [0.03, 1.6], [0, 1.61]].map(([r, y]) => new THREE.Vector2(r, y)), 8), new THREE.MeshBasicMaterial({ color: 0xffc070 }));
    lamp.position.x = 0.28; lamp.position.y = -0.0;
    g.add(post, arm, lamp); g.position.set(at[0], at[1], at[2]);
    return g;
  }
  window.SurvivalFox = { make, lantern, SPECIES };

  // ---- the hunt, scripted on the rule's numbers: [t0, t1, state]
  const RULES = {
    "red-fox": { notice: 14, stalk: 1.4, warn_m: 5.0, warn_s: 1.8, lunge_m: 3.5, lunge: 8, bite: 1.3, dmg: 6, health: 30 },
    "dust-hound": { notice: 18, stalk: 1.8, warn_m: 6.0, warn_s: 1.5, lunge_m: 4.0, lunge: 9, bite: 1.4, dmg: 10, health: 50 },
  };
  // The player at the origin, facing +x. Each species' script: where the beast is along +x (and a
  // little off it), its state, and the player's presses.
  function script(species) {
    const R = RULES[species];
    const start = 12, warnAt = R.warn_m, lungeTo = 1.0;
    const tN = 1.5, tW = tN + (start - warnAt) / R.stalk, tA = tW + R.warn_s, tB = tA + 0.6, tR = tB + 0.7, tA2 = tR + 0.8;
    // the player's presses: the spear's hit lands 0.315 s after each (0.45 of 0.7 s)
    const hit = 0.7 * 0.45, p1 = tA2 + 0.12 - hit + 0.18, p2 = p1 + 0.7;
    const h1 = p1 + hit, h2 = p2 + hit;
    const hits = R.health > 30 ? [h1, h2, h2 + 0.7] : [h1, h2];
    const presses = hits.map((h) => h - hit);
    const fleeAt = hits[hits.length - 1] + 0.05;
    return { R, start, warnAt, lungeTo, tN, tW, tA, tB, tR, tA2, presses, hits, fleeAt, end: fleeAt + 3.5 };
  }
  function stateAt(sc, t) {
    const { tN, tW, tA, tB, tR, tA2, fleeAt } = sc;
    if (t < tN) return "wander";
    if (t < tW) return "stalk";
    if (t < tA) return "warn";
    if (t < tB) return "attack";
    if (t < tA2) return "warn";
    if (t < fleeAt) return "attack";
    return "flee";
  }
  // where the beast stands at t (x along the player's facing), its speed and its lunge
  function beastAt(sc, t) {
    const { R, start, warnAt, lungeTo, tN, tW, tA, tB, tR, tA2, hits, fleeAt } = sc;
    const smooth = (u) => u * u * (3 - 2 * u);
    let x, sp = 0, lunge = 0, z = 0;
    if (t < tN) { x = start + 0.6 * Math.sin(t * 1.3); z = 1.0 - 0.4 * t; sp = 0.6; }
    else if (t < tW) { x = start - (t - tN) * R.stalk; z = 1.0 - 0.6 - (t - tN) * 0.04; sp = R.stalk; }
    else if (t < tA) { x = warnAt; z = 0.4 - (tW - tN) * 0.04; }
    else if (t < tB) { const u = (t - tA) / (tB - tA); x = warnAt + (lungeTo - warnAt) * Math.pow(u, 1.6); lunge = u < 0.25 ? -u / 0.25 : u < 0.8 ? 1 : (1 - u) / 0.2; sp = R.lunge; }
    else if (t < tR) { const u = (t - tB) / (tR - tB); x = lungeTo + (warnAt - lungeTo) * smooth(u); sp = 2.5; }
    else if (t < tA2) { x = warnAt; }
    else if (t < fleeAt) {
      // lunges again; each spear hit knocks it back a step, and struck it comes on again
      const u = Math.min(1, (t - tA2) / 0.6);
      x = warnAt + (lungeTo + 0.6 - warnAt) * Math.pow(u, 1.6);
      for (const h of hits) if (t > h) x += 0.5 * Math.min(1, (t - h) / 0.12) * Math.max(0, 1 - (t - h - 0.12) / 0.5);
      lunge = u < 0.25 ? -u / 0.25 : 1; sp = R.lunge;
    } else { const e = t - fleeAt; x = lungeTo + 1.1 + e * 7.0 * Math.min(1, e / 0.4); z = -e * 1.2; sp = 7.0; }
    if (t >= tW && t < fleeAt) z = 0.4 - (tW - tN) * 0.04;
    return { x, z, sp, lunge };
  }

  // ---- the scene
  let built = false, running = false, playing = false, slow = false, night = false, species = "red-fox";
  let renderer, scene, cam, st, apply, stage, beasts = {}, fig, spear, groundG, groundD, lights, last = 0, t = 0, ft = 0, raf = 0, sc;
  const LAMPS = [[-2.6, 0, -2.2], [4.5, 0, 3.2], [9.5, 0, -2.6], [14.5, 0, 2.8]];
  function build() {
    stage = document.getElementById("hStage");
    renderer = new THREE.WebGLRenderer({ antialias: true, preserveDrawingBuffer: true });
    renderer.setPixelRatio(Math.min(2, devicePixelRatio)); renderer.outputEncoding = THREE.sRGBEncoding;
    stage.insertBefore(renderer.domElement, stage.firstChild);
    scene = new THREE.Scene();
    cam = new THREE.PerspectiveCamera(42, 1, 0.05, 300);
    const sun = new THREE.DirectionalLight(0xfff1d8, 0.95); sun.position.set(-4, 8, 5); scene.add(sun);
    const moon = new THREE.DirectionalLight(0x9fb4e0, 0.08); moon.position.set(4, 7, -3); scene.add(moon);
    const hemi = new THREE.HemisphereLight(0xd8e8ff, 0x6a5a40, 0.55); scene.add(hemi);
    lights = { sun, moon, hemi };
    // grass for the fox, sand for the hound: the same hex patch, two textures
    const mk = (col) => { const m = W().groundHex(); m.material.map = S().tex(16, (x, y) => { const h = H(x, y, 51); const v = h > 0.82 ? 1.15 : h < 0.2 ? 0.85 : 1; return [col[0] * v, col[1] * v, col[2] * v]; }); m.scale.set(2.2, 1, 2.2); m.position.set(3.6, 0, 0); return m; };
    groundG = mk([86, 132, 58]); groundD = mk([196, 168, 116]); scene.add(groundG, groundD);
    const fire = W().campfire(); fire.position.set(-2.4, 0, 1.6); scene.add(fire);
    LAMPS.forEach((p) => scene.add(lantern(p)));
    for (const k of ["red-fox", "dust-hound"]) { beasts[k] = make(k); scene.add(beasts[k].group); }
    fig = W().figure(); fig.rotation.y = Math.PI / 2; scene.add(fig);
    spear = new THREE.Group(); const sm = W().spear(); sm.position.y = -0.95; spear.add(sm); scene.add(spear);
    st = Object.assign({ min: 1.5, max: 60 }, { t: new THREE.Vector3(3.4, 0.45, 0.2), r: 6.5, pol: 1.2, az: -0.8 });
    apply = S().orbit(cam, renderer.domElement, st);
    built = true;
    setSpecies(species); setNight(night); resize();
  }
  function setSpecies(k) {
    species = k; sc = script(k);
    document.querySelectorAll("[data-hspecies]").forEach((b) => b.setAttribute("aria-pressed", String(b.dataset.hspecies === k)));
    const scrub = document.getElementById("hScrub"); if (scrub) scrub.max = sc.end.toFixed(2);
    if (!built) return;
    for (const b in beasts) beasts[b].group.visible = b === k;
    groundG.visible = k === "red-fox"; groundD.visible = k !== "red-fox";
  }
  function setNight(n) {
    night = n;
    document.querySelectorAll("[data-htime]").forEach((b) => b.setAttribute("aria-pressed", String((b.dataset.htime === "night") === n)));
    if (!built) return;
    scene.background = new THREE.Color(n ? 0x050a14 : 0x9cc4e4);
    lights.sun.intensity = n ? 0 : 0.95; lights.hemi.intensity = n ? 0.05 : 0.55; lights.moon.intensity = n ? 0.06 : 0;
    lights.hemi.color.set(n ? 0x6070a0 : 0xd8e8ff); lights.hemi.groundColor.set(n ? 0x101418 : 0x6a5a40);
  }
  function resize() {
    if (!built) return;
    const r = stage.getBoundingClientRect();
    renderer.setSize(r.width, r.height, false);
    renderer.domElement.style.width = r.width + "px"; renderer.domElement.style.height = r.height + "px";
    cam.aspect = r.width / Math.max(1, r.height); cam.updateProjectionMatrix();
  }
  const LABEL = { wander: "WANDER", stalk: "STALK", warn: "WARN", attack: "ATTACK", flee: "FLEE" };
  function draw() {
    const tt = Math.min(t, sc.end);
    const state = stateAt(sc, tt), b = beastAt(sc, tt), beast = beasts[species];
    const g = beast.group;
    g.position.set(b.x, 0, b.z);
    const dirX = state === "flee" ? 1 : -1;
    g.rotation.y = dirX > 0 ? Math.PI / 2 - 0.15 : -Math.PI / 2 + (state === "wander" ? 0.6 * Math.sin(tt) : 0);
    const flinch = sc.hits.reduce((f, h) => (tt > h && tt < h + 0.25 ? 1 - (tt - h) / 0.25 : f), 0);
    const crouch = state === "warn" ? 1 : state === "stalk" ? 0.45 : 0;
    beast.pose({ crouch, flinch, phase: tt * (b.sp / (SPECIES[species].len * 0.55)) * Math.PI, speed: b.sp, lunge: state === "attack" ? b.lunge : 0, headLow: state === "stalk" || state === "warn" ? 1 : 0, tailDown: state === "flee", t: tt });
    // the player: steps in during the warning, the spear thrusts on each press (the In hand swing)
    const step = Math.max(0, Math.min(1, (tt - (sc.tW + 0.3)) / 1.0)) * 1.6;
    fig.position.set(step, 0, 0);
    let pull = 0, pitch = 0;
    for (const p of sc.presses) { if (tt >= p) { const r = window.SurvivalSwing.poseAt("spear", tt - p, null); if (r.stage !== "rest") { pull = r.pose.pull; pitch = r.pose.pitch; } } }
    const bitten = tt > sc.tB && tt < sc.tB + 0.3 && Math.abs(b.x - step) < sc.R.bite + 0.6;
    fig.rotation.z = bitten ? -0.06 : 0;
    spear.position.set(step + 0.15 + pull * 1.6, 1.08, -0.24);
    spear.rotation.set(0, 0, -Math.PI / 2 - pitch * 0.2 - 0.06);
    // lit at night by the campfire and the lanterns, each lighting what is near it
    const U = S().fireUniforms;
    cam.updateMatrixWorld();
    if (night) {
      const L = [[-2.4, 0.7, 1.6, 3.2 * (0.9 + 0.1 * Math.sin(ft * 8) * Math.sin(ft * 5.1))]].concat(LAMPS.map((p) => [p[0] + 0.28, 1.5, p[2], 1.8]));
      U.fireN.value = L.length;
      L.forEach((l, i) => { const v = new THREE.Vector3(l[0], l[1], l[2]).applyMatrix4(cam.matrixWorldInverse); U.fireL.value[i].set(v.x, v.y, v.z, l[3]); });
    } else U.fireN.value = 0;
    renderer.render(scene, cam);
    const lab = document.getElementById("hState"); if (lab) lab.textContent = LABEL[state];
    const scrub = document.getElementById("hScrub"); if (scrub && document.activeElement !== scrub) scrub.value = tt.toFixed(2);
    return { state, t: tt, x: b.x };
  }
  function frame(now) {
    if (!running) return;
    const dt = Math.min(0.1, (now - last) / 1000 || 0); last = now; ft += dt;
    if (playing) { t += dt * (slow ? 0.25 : 1); if (t > sc.end + 1) t = 0; }
    draw();
    raf = requestAnimationFrame(frame);
  }
  function setPlay(p) { playing = p; const b = document.getElementById("hPlay"); if (b) { b.textContent = p ? "Pause" : "Play"; b.setAttribute("aria-pressed", String(p)); } }
  document.querySelectorAll("[data-htime]").forEach((b) => b.addEventListener("click", () => setNight(b.dataset.htime === "night")));
  document.querySelectorAll("[data-hspecies]").forEach((b) => b.addEventListener("click", () => { setSpecies(b.dataset.hspecies); t = 0; }));
  const bind = (id, f) => { const el = document.getElementById(id); if (el) f(el); };
  bind("hPlay", (el) => el.addEventListener("click", () => setPlay(!playing)));
  bind("hSlow", (el) => el.addEventListener("click", () => { slow = !slow; el.setAttribute("aria-pressed", String(slow)); }));
  bind("hScrub", (el) => el.addEventListener("input", () => { t = parseFloat(el.value); setPlay(false); }));
  window.addEventListener("resize", resize);
  sc = script(species);
  window.huntTab = {
    start() { if (!built) build(); running = true; resize(); last = performance.now(); raf = requestAnimationFrame(frame); },
    stop() { running = false; cancelAnimationFrame(raf); },
    still(o) {
      running = false; cancelAnimationFrame(raf); if (!built) build();
      if (o.species) setSpecies(o.species);
      if (o.night !== undefined) setNight(o.night);
      if (o.state) { const at = { wander: 0.8, stalk: (sc.tN + sc.tW) / 2, warn: sc.tW + 1.0, attack: sc.tA + 0.42, hit: sc.hits[0] + 0.02, flee: sc.fleeAt + 0.9 }; t = at[o.state]; }
      if (o.t !== undefined) t = o.t;
      if (o.view) { Object.assign(st, o.view, { t: new THREE.Vector3(...o.view.t) }); apply(); }
      if (o.ft !== undefined) ft = o.ft;
      resize();
      return draw();
    },
    script: () => sc,
  };
})();
