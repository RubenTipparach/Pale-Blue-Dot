// Survival mockup, Weapons tab's In hand view (survival-mode combat C1, task 1.6): the held weapons'
// swings by the animation skill, beside today's tool stroke, in the weapons tab's one renderer.
// Each swing is four stages, windup, anticipation, hit and recoil, then the ease home: keyed poses on
// the fist's arc about a virtual shoulder (eye space: x right, y up, -z ahead), the weapon's own pitch
// and roll on top, blended by curves (windup eased out, anticipation a slow ease into a held pose, the
// hit eased in with t cubed so it is fastest at contact, a held frame, a quick recoil with an
// overshoot). The rule owns when the hit lands (survival.ron: hit_share of the weapon's time_s); the
// stage times are fitted to it, the hit never shortened. A thrown spear and the bow are held: raised
// or drawn while the button is down, released on letting go. Nothing moves until Play.
(function () {
  "use strict";
  const S = () => window.SurvivalCommon;
  const W = () => window.SurvivalWeapons;
  const RULE = { hit_share: 0.45, tool_swing_s: 0.55, time_s: { spear: 0.7, axe: 0.6, sword: 0.5 }, draw_s: 0.8 };
  const HOLD = { grip: [0.26, -0.3, -0.56], up: [-0.18, 1.0, -0.42], spear_up: [-0.22, 0.62, -1.0] };
  const SHOULDER = [0.2, -0.25, 0.05];
  const K = (phi, yaw, pull, pitch, roll) => ({ phi, yaw, pull, pitch, roll: roll || 0 });
  // The proposed swings (held.ron's weapon_swing). Times s, angles rad, pull m along the aim.
  const DEF = {
    spear: { kind: "melee", windup_s: 0.14, anticipate_s: 0.10, strike_s: 0.07, impact_hold_s: 0.05, recoil_s: 0.08, recover_s: 0.26,
      top: K(0.04, 0.06, -0.22, -0.58), coil: K(0.06, 0.08, -0.28, -0.62), hit: K(-0.05, -0.08, 0.4, -0.74), recoil: K(-0.03, -0.05, 0.3, -0.7) },
    axe: { kind: "melee", windup_s: 0.11, anticipate_s: 0.07, strike_s: 0.07, impact_hold_s: 0.05, recoil_s: 0.07, recover_s: 0.23,
      top: K(0.55, -0.1, -0.08, 0.95, 0.1), coil: K(0.62, -0.12, -0.1, 1.08, 0.12), hit: K(-0.3, 0.14, 0.14, -1.45, -0.05), recoil: K(-0.2, 0.1, 0.1, -1.15) },
    sword: { kind: "melee", windup_s: 0.09, anticipate_s: 0.06, strike_s: 0.06, impact_hold_s: 0.03, recoil_s: 0.08, recover_s: 0.18,
      top: K(0.4, -0.5, -0.06, 0.55, -0.85), coil: K(0.46, -0.6, -0.08, 0.65, -0.95), hit: K(-0.25, 0.7, 0.12, -1.1, 0.7), recoil: K(-0.32, 0.85, 0.08, -1.2, 0.8) },
    throw: { kind: "draw", windup_s: 0.25, anticipate_s: 0.15, strike_s: 0.08, impact_hold_s: 0.0, recoil_s: 0.1, recover_s: 0.3,
      top: K(0.42, -0.08, -0.2, -0.5), coil: K(0.46, -0.1, -0.28, -0.52), hit: K(0.12, 0.2, 0.35, -0.72), recoil: K(-0.2, 0.28, 0.2, -0.9) },
    bow: { kind: "draw", windup_s: 0.15, anticipate_s: 0.8, strike_s: 0.04, impact_hold_s: 0.02, recoil_s: 0.1, recover_s: 0.3,
      top: K(0.12, 0.36, 0.08, 0.32, -0.12), coil: K(0.14, 0.36, 0.06, 0.32, -0.12), hit: K(0.15, 0.36, 0.12, 0.34, -0.12), recoil: K(0.08, 0.34, 0.06, 0.32, -0.1) },
    // Today: the tools' stroke (held.ron's stroke, held-swing decisions 5 to 8) on the stone axe, in this
    // page's pose terms: the raise about the shoulder, the head's pitch, the reach and the sweep across.
    today: { kind: "melee", windup_s: 0.1, anticipate_s: 0.07, strike_s: 0.06, impact_hold_s: 0.05, recoil_s: 0.06, recover_s: 0.16,
      top: K(0.42, 0.0, 0.0, 0.35), coil: K(0.46, 0.0, 0.0, 0.43), hit: K(-0.05, 0.25, 0.12, -0.55), recoil: K(0.0, 0.2, 0.08, -0.4) },
  };
  const P = JSON.parse(JSON.stringify(DEF));
  let exaggerate = 1.0;

  // ---- the curves (the skill's): ease out, slow in and out, t cubed, a kick with an overshoot
  const easeOut = (t) => 1 - (1 - t) * (1 - t);
  const smooth = (t) => t * t * (3 - 2 * t);
  const backOut = (t) => { const c = 1.7; const u = t - 1; return 1 + (c + 1) * u * u * u + c * u * u; };
  const lerpK = (a, b, u) => { const o = {}; for (const k of ["phi", "yaw", "pull", "pitch", "roll"]) o[k] = a[k] + (b[k] - a[k]) * u; return o; };
  const REST = K(0, 0, 0, 0, 0);
  function scaled(sw, k) { if (k === 1) return sw; const o = {}; for (const n of ["phi", "yaw", "pull", "pitch", "roll"]) o[n] = sw[n] * k; return o; }

  // The stage times for one swing: a melee swing's fitted to the rule's period and hit, a held one's
  // as written with the release where the button is let go.
  function times(name, p, release) {
    const t = { w: p.windup_s, a: p.anticipate_s, s: p.strike_s, h: p.impact_hold_s, r: p.recoil_s, v: p.recover_s };
    if (p.kind === "melee") {
      const period = name === "today" ? RULE.tool_swing_s : RULE.time_s[name];
      const hit = period * RULE.hit_share;
      const k1 = Math.max(0.1, (hit - t.s) / (t.w + t.a)), post = t.h + t.r + t.v, k2 = Math.max(0.1, (period - hit) / post);
      t.w *= k1; t.a *= k1; t.h *= k2; t.r *= k2; t.v *= k2;
      t.rel = t.w + t.a; t.period = period; t.hit = hit;
    } else {
      t.rel = release; t.hit = release == null ? null : release + t.s;
      t.period = release == null ? Infinity : release + t.s + t.h + t.r + t.v;
    }
    return t;
  }
  // The pose at `t` seconds into a swing, and its stage.
  function poseAt(name, t, release) {
    const p = P[name], T = times(name, p, release), x = exaggerate;
    const top = scaled(p.top, x), coil = scaled(p.coil, x), hit = p.hit, rec = scaled(p.recoil, x);
    if (t < 0 || t >= T.period) return { pose: REST, stage: "rest", T };
    if (t < T.w) return { pose: lerpK(REST, top, easeOut(t / T.w)), stage: "windup", T };
    const rel = T.rel == null ? Infinity : T.rel;
    if (t < rel) {
      const u = Math.min(1, (t - T.w) / Math.max(1e-3, T.a));
      return { pose: lerpK(top, coil, smooth(u)), stage: "anticipation", draw: u, T };
    }
    let e = t - rel;
    if (e < T.s) return { pose: lerpK(coil, hit, Math.pow(e / T.s, 3)), stage: "hit", T };
    e -= T.s;
    if (e < T.h) return { pose: hit, stage: "hit", held: true, T };
    e -= T.h;
    if (e < T.r) return { pose: lerpK(hit, rec, backOut(e / T.r)), stage: "recoil", T };
    e -= T.r;
    return { pose: lerpK(rec, REST, smooth(Math.min(1, e / Math.max(1e-3, T.v)))), stage: "recover", T };
  }

  // ---- the scene: eye space, the camera at the origin looking down -z
  let scene, cam, built = false, ground, fist, sleeve, items = {}, target, flying = null, night = false, lights = [];
  let cur = "spear", motion = "proposed", playing = false, slow = false, posesOn = false, clock = 0, fireT = 0;
  const v3 = (a) => new THREE.Vector3(a[0], a[1], a[2]);
  function weaponOf(name) {
    if (name === "today") return "stoneAxe";
    if (name === "throw") return "spear";
    return name;
  }
  function stoneAxe() {
    const g = new THREE.Group(), w = W();
    const curve = new THREE.CatmullRomCurve3([v3([0, 0, 0]), v3([0.01, 0.22, 0]), v3([-0.005, 0.45, 0]), v3([0, 0.6, 0])]);
    const handle = new THREE.Mesh(w.taperTube(curve, 24, 0.018, (t) => 0.85 + 0.15 * t, 0.8), w.mat({ map: w.woodT() }));
    const s = new THREE.Shape();
    s.moveTo(-0.025, 0.47); s.quadraticCurveTo(0.03, 0.44, 0.09, 0.45); s.quadraticCurveTo(0.13, 0.5, 0.125, 0.56);
    s.quadraticCurveTo(0.09, 0.63, 0.03, 0.62); s.quadraticCurveTo(-0.03, 0.6, -0.035, 0.55); s.quadraticCurveTo(-0.04, 0.5, -0.025, 0.47);
    const head = new THREE.Mesh(w.extrude(s, 0.03, 0.012, 0.008, 3, 3), w.mat({ map: w.stoneT(), shininess: 14 }));
    g.add(handle, head);
    return g;
  }
  // Where each weapon is gripped, m up its own length from its foot.
  const GRIP = { spear: 0.95, bow: 0.75, axe: 0.08, sword: 0.1, stoneAxe: 0.08 };
  function fistMesh() {
    const geo = S().weld(new THREE.IcosahedronGeometry(0.045, 2)), p = geo.attributes.position, v = new THREE.Vector3();
    for (let i = 0; i < p.count; i++) { v.fromBufferAttribute(p, i); v.x *= 0.9; v.y *= 1.15; v.z *= 1.05; const k = 1 + 0.08 * Math.sin(v.y * 60) * (v.x > 0 ? 1 : 0.4); v.multiplyScalar(k); p.setXYZ(i, v.x, v.y, v.z); }
    geo.computeVertexNormals();
    return new THREE.Mesh(geo, W().mat({ map: S().tex(16, (x, y) => { const s = 0.92 + 0.12 * S().hash(x, y, 31) - ((x + 2 * y) % 7 === 0 ? 0.08 : 0); return [220 * s, 168 * s, 128 * s]; }) }));
  }
  function sleeveMesh() {
    // A tapered sleeve from the wrist back past the screen's edge: rebuilt as the fist moves.
    return new THREE.Mesh(new THREE.BufferGeometry(), W().mat({ map: S().tex(16, (x, y) => { const s = 0.85 + 0.2 * S().hash(x, y >> 1, 33) - (y % 6 === 0 ? 0.1 : 0); return [80 * s, 118 * s, 128 * s]; }) }));
  }
  function setSleeve(wrist) {
    const sh = v3(SHOULDER).add(new THREE.Vector3(0.08, -0.12, 0.1));
    const mid = wrist.clone().lerp(sh, 0.5).add(new THREE.Vector3(0.03, -0.03, 0));
    const curve = new THREE.CatmullRomCurve3([wrist.clone().add(wrist.clone().sub(sh).normalize().multiplyScalar(-0.02)), mid, sh, sh.clone().add(sh.clone().sub(mid).normalize().multiplyScalar(0.5))]);
    const g = W().taperTube(curve, 20, 0.04, (t) => 0.75 + 0.45 * t, 0.9);
    sleeve.geometry.dispose(); sleeve.geometry = g;
  }
  function build(renderer) {
    scene = new THREE.Scene();
    cam = new THREE.PerspectiveCamera(70, 1, 0.01, 80);
    cam.position.set(0, 0, 0); cam.lookAt(0, -0.18, -1);
    const sun = new THREE.DirectionalLight(0xfff1d8, 0.95); sun.position.set(-3, 6, 2); scene.add(sun);
    const hemi = new THREE.HemisphereLight(0xd8e8ff, 0x6a5a40, 0.55); scene.add(hemi);
    const moon = new THREE.DirectionalLight(0x9fb4e0, 0.1); moon.position.set(2, 5, -3); scene.add(moon);
    lights = { sun, hemi, moon };
    ground = W().groundHex(); ground.position.set(-1.6, -1.6, -3); scene.add(ground);
    // the target: a red fox, the game's own blocky form, a stride ahead
    target = window.SurvivalFox.make("red-fox"); target.group.position.set(0.05, -1.6, -2.1); target.group.rotation.y = Math.PI * 0.55; scene.add(target.group);
    const fire = W().campfire(); fire.position.set(-1.6, -1.6, -3.4); scene.add(fire);
    scene.add(window.SurvivalFox.lantern([1.6, -1.6, -3.2]));
    for (const k of ["spear", "bow", "axe", "sword", "stoneAxe"]) {
      const m = k === "spear" ? W().spear() : k === "bow" ? W().bow() : k === "axe" ? W().huntingAxe() : k === "sword" ? W().sword() : stoneAxe();
      // the bow's belly faces the target and its string the player; the arrow is drawn on its own
      if (k === "bow") { m.children[2].visible = false; m.rotation.y = -Math.PI / 2; }
      const pivot = new THREE.Group(); m.position.y = -GRIP[k]; pivot.add(m); pivot.visible = false; scene.add(pivot);
      items[k] = pivot;
    }
    // a nocked arrow, and a thrown spear's flight
    const ar = W().bow().children[2]; ar.position.set(0, 0, 0); const arrow = new THREE.Group(); ar.rotation.x = 0; arrow.add(ar); arrow.visible = false; scene.add(arrow); items.arrow = arrow;
    const fl = W().spear(); const flyG = new THREE.Group(); fl.position.y = -GRIP.spear; flyG.add(fl); flyG.visible = false; scene.add(flyG); items.flight = flyG;
    fist = fistMesh(); scene.add(fist);
    sleeve = sleeveMesh(); scene.add(sleeve);
    built = true;
  }
  // The rest orientation of a weapon: its foot-to-tip axis along the hold's way up.
  function restQuat(name) {
    const up = v3(name === "spear" || name === "throw" ? HOLD.spear_up : HOLD.up).normalize();
    return new THREE.Quaternion().setFromUnitVectors(new THREE.Vector3(0, 1, 0), up);
  }
  function place(name, pose) {
    const sh = v3(SHOULDER), rest = v3(HOLD.grip);
    const arm = rest.clone().sub(sh);
    const q = new THREE.Quaternion().setFromEuler(new THREE.Euler(pose.phi, pose.yaw, 0, "YXZ"));
    const aim = new THREE.Vector3(0, -0.18, -1).normalize();
    const fistAt = sh.clone().add(arm.applyQuaternion(q)).add(aim.multiplyScalar(pose.pull));
    const turn = new THREE.Quaternion().setFromEuler(new THREE.Euler(pose.pitch, pose.yaw, pose.roll, "YXZ"));
    return { fist: fistAt, rot: turn.multiply(restQuat(name)) };
  }
  // Play: a melee swing loops one tap's swing every 1.6 s; a held one raises or draws for 0.9 s,
  // releases, and the spear or arrow flies at the fox.
  function scriptT(name) { return P[name].kind === "draw" ? 0.9 : null; }
  function frame(dt, renderer) {
    if (!built) build(renderer);
    const name = motion === "today" ? "today" : cur;
    if (playing) clock += dt * (slow ? 0.25 : 1);
    fireT += dt;
    const loop = 1.8;
    let t = clock % loop;
    let r = poseAt(name, t, scriptT(name));
    if (posesOn) {
      // freeze on each keyed pose in turn, a second each
      const keys = ["rest", "top", "coil", "hit", "recoil"], k = keys[Math.floor(clock) % keys.length];
      r = { pose: k === "rest" ? REST : P[name][k], stage: k, T: r.T };
    }
    const wk = weaponOf(name);
    for (const k in items) if (k !== "arrow" && k !== "flight") items[k].visible = false;
    const pl = place(name, r.pose);
    const flown = name === "throw" && r.T.rel != null && t >= r.T.rel;
    const it = items[wk];
    it.visible = !flown || posesOn; it.position.copy(pl.fist); it.quaternion.copy(pl.rot);
    fist.position.copy(pl.fist); fist.quaternion.copy(pl.rot);
    setSleeve(pl.fist);
    // the arrow on the string while drawn, then away
    items.arrow.visible = name === "bow" && (r.stage === "anticipation" || r.stage === "windup");
    if (items.arrow.visible) {
      const d = r.draw || 0;
      const fwd = new THREE.Vector3(0, -0.18, -1).normalize();
      const across = new THREE.Vector3(1, 0, 0).applyQuaternion(pl.rot);
      items.arrow.position.copy(pl.fist).add(across.multiplyScalar(-0.02)).add(fwd.clone().multiplyScalar(-0.1 - 0.45 * d));
      items.arrow.quaternion.setFromUnitVectors(new THREE.Vector3(0, 1, 0), fwd);
    }
    // the thrown spear or the loosed arrow's flight at the fox, under gravity
    items.flight.visible = false;
    if ((name === "throw" || name === "bow") && r.T.rel != null && t >= r.T.rel && !posesOn) {
      const e = t - r.T.rel, sp = name === "throw" ? 18 : 45, from = pl.fist.clone();
      const to = target.group.position.clone().add(new THREE.Vector3(0, 0.35, 0));
      const dir = to.clone().sub(v3(HOLD.grip)).normalize();
      const reach = v3(HOLD.grip).distanceTo(to), tf = reach / sp;
      if (e < tf) {
        const g = name === "throw" ? items.flight : items.arrow;
        const p = v3(HOLD.grip).add(dir.clone().multiplyScalar(sp * e)); p.y += (tf - e) * e * 0.5 * 9.8 * 0.6;
        g.visible = true; g.position.copy(name === "throw" ? p : p); g.quaternion.setFromUnitVectors(new THREE.Vector3(0, 1, 0), dir);
        if (name === "bow") items.arrow.visible = true;
        from.copy(p);
      }
    }
    // the fox flinches on the hit's frame
    const hitAt = r.T.hit;
    const since = hitAt == null ? 99 : t - hitAt - (name === "throw" || name === "bow" ? 0.1 : 0);
    target.pose({ crouch: 0.3, flinch: since >= 0 && since < 0.25 ? 1 - since / 0.25 : 0, t: fireT, phase: 0, speed: 0 });
    // the light: a day sun, or at night the campfire and a lantern lighting what is near them
    lights.sun.intensity = night ? 0 : 0.95; lights.hemi.intensity = night ? 0.05 : 0.55; lights.moon.intensity = night ? 0.06 : 0;
    scene.background = new THREE.Color(night ? 0x050a14 : 0x9cc4e4);
    cam.updateMatrixWorld();
    const U = S().fireUniforms;
    if (night) {
      const L = [[-1.6, -1.0, -3.4, 5.0 * (0.9 + 0.1 * Math.sin(fireT * 8))], [1.88, -0.1, -3.2, 3.4]];
      U.fireN.value = L.length;
      L.forEach((l, i) => { const v = new THREE.Vector3(l[0], l[1], l[2]).applyMatrix4(cam.matrixWorldInverse); U.fireL.value[i].set(v.x, v.y, v.z, l[3]); });
    } else U.fireN.value = 0;
    return { scene, cam, stage: r.stage, t };
  }
  function ron(name) {
    const p = P[name], f = (n) => Number(n).toFixed(3).replace(/0+$/, "").replace(/\.$/, ".0");
    const key = (k) => `(phi: ${f(k.phi)}, yaw: ${f(k.yaw)}, pull_m: ${f(k.pull)}, pitch: ${f(k.pitch)}, roll: ${f(k.roll)})`;
    return `${name}: (\n    windup_s: ${f(p.windup_s)}, anticipate_s: ${f(p.anticipate_s)}, strike_s: ${f(p.strike_s)},\n    impact_hold_s: ${f(p.impact_hold_s)}, recoil_s: ${f(p.recoil_s)}, recover_s: ${f(p.recover_s)},\n    top: ${key(p.top)},\n    coil: ${key(p.coil)},\n    hit: ${key(p.hit)},\n    recoil: ${key(p.recoil)},\n),`;
  }
  window.SurvivalSwing = {
    DEF, P, RULE, poseAt, times, ron, frame,
    set(o) {
      if (o.weapon) cur = o.weapon;
      if (o.motion) motion = o.motion;
      if (o.playing !== undefined) playing = o.playing;
      if (o.slow !== undefined) slow = o.slow;
      if (o.poses !== undefined) posesOn = o.poses;
      if (o.clock !== undefined) clock = o.clock;
      if (o.night !== undefined) night = o.night;
      if (o.exaggerate !== undefined) exaggerate = o.exaggerate;
      return { weapon: cur, motion, playing, slow, poses: posesOn, clock, night };
    },
    get exaggerate() { return exaggerate; },
    current: () => (motion === "today" ? "today" : cur),
  };
})();
