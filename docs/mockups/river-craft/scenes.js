// River Craft (M34): the five scenes. Each is a river (a flow field over a channel of hex cells), the craft on it,
// and what the player does: def = {river, posts, home, build(ctx), reset(), step(h), sync(), lanterns(), follow(),
// hud(), actions, choice, hold}. Needs engine.js and physics.js before it.
(function () {
"use strict";
const RC = window.RC, clamp = RC.clamp, TAU = Math.PI * 2;
const OUT = [0, 0];
RC.scenes = {};
const wrap = a => { while (a > Math.PI) a -= TAU; while (a < -Math.PI) a += TAU; return a; };
// Put a mesh where a hull is: heading, then pitch, then roll.
function place(obj, b) { obj.position.set(b.x, b.y, b.z); obj.rotation.set(b.roll, -b.yaw, b.pitch, "YZX"); }
// A point on a hull, in the world, at height ly above its waterline.
const p3 = (b, lx, lz, ly) => [b.wx(lx, lz), b.y + ly, b.wz(lx, lz)];
const speedOf = b => Math.hypot(b.vx, b.vz);
const fmtN = t => t >= 1000 ? (t / 1000).toFixed(1) + " kN" : Math.round(t) + " N";
const mat = (U, tint) => RC.litMaterial(U, {tint});
const mk = (geo, m, x, y, z) => { const o = new THREE.Mesh(geo, m); o.position.set(x || 0, y || 0, z || 0); return o; };
const hold = () => ({up: false, down: false, left: false, right: false});

// ---------------- craft models ----------------
function raftModel(U) {
  const g = new THREE.Group(), lg = RC.logGeo(4.4, 0.25), m = mat(U);
  for (let i = 0; i < 5; i++) g.add(mk(lg, m, 0, -0.04, -0.96 + i * 0.48));
  const plank = RC.box(0.3, 0.08, 2.5, "planks");
  for (const x of [-1.5, 0, 1.5]) g.add(mk(plank, m, x, 0.24, 0));
  g.add(mk(RC.box(0.7, 0.5, 0.6, "crate"), m, 1.2, 0.53, 0.5));
  const post = mk(RC.box(0.1, 1.1, 0.1, "darkPlanks"), m, 1.95, 0.8, -0.95); g.add(post);
  const lan = RC.lantern(U, 0.9); lan.position.set(1.95, 1.45, -0.95); g.add(lan);
  const man = RC.person(U, 0x7a6a3a); man.position.set(-0.7, 0.28, 0.35); g.add(man);
  const pivot = new THREE.Group(); pivot.position.set(-0.55, 1.45, 0.62); g.add(pivot);
  pivot.add(mk(RC.box(0.07, 4.6, 0.07, "planks"), m, 0, -1.4, 0));
  g.userData = {lantern: lan, pivot};
  return g;
}
function tugModel(U, L, B) {
  const g = new THREE.Group(), m = mat(U);
  g.add(new THREE.Mesh(RC.hullGeo(L, B, 0.8, 0.75, L * 0.3, "darkPlanks"), RC.litMaterial(U, {side: THREE.DoubleSide})));
  g.add(mk(RC.box(L * 0.32, 1.1, B * 0.7, "planks"), m, L * 0.05, 1.3, 0));
  g.add(mk(RC.box(L * 0.36, 0.1, B * 0.8, "darkPlanks"), m, L * 0.05, 1.9, 0));
  const stack = new THREE.Mesh(RC.tiled(new THREE.CylinderGeometry(0.22, 0.24, 1.3, 6), "iron", 1.4, 1.3), m); stack.position.set(-L * 0.12, 1.95, 0); g.add(stack);
  g.add(mk(RC.box(0.3, 0.35, 0.3, "iron"), m, -L / 2 + 0.4, 0.92, 0));     // the towing bitt
  const mast = mk(RC.box(0.08, 1.2, 0.08, "darkPlanks"), m, L * 0.15, 2.5, 0); g.add(mast);
  const lan = RC.lantern(U, 0.9); lan.position.set(L * 0.15, 3.15, 0); g.add(lan);
  const man = RC.person(U, 0x3f5a78); man.position.set(L * 0.05, 0.75, B * 0.25); man.scale.setScalar(0.95); g.add(man);
  g.userData = {lantern: lan, stack};
  return g;
}
function bargeModel(U, L, B, load) {
  const g = new THREE.Group(), m = mat(U);
  const hull = mk(RC.box(L, 1.3, B, "darkPlanks"), m, 0, -0.1, 0); g.add(hull);
  g.add(mk(RC.box(L - 0.2, 0.06, B - 0.2, "planks"), m, 0, 0.58, 0));
  if (load === "crates") { for (let i = 0; i < 3; i++) for (let j = 0; j < 2; j++) g.add(mk(RC.box(1.1, 0.9, 1.1, "crate"), m, -2.2 + i * 2.2, 1.06, -0.65 + j * 1.3)); }
  else { const lg = RC.logGeo(L - 1.2, 0.24); for (let i = 0; i < 5; i++) g.add(mk(lg, m, 0, 0.86, -1.0 + i * 0.5)); for (let i = 0; i < 4; i++) g.add(mk(lg, m, 0, 1.28, -0.75 + i * 0.5)); }
  g.add(mk(RC.box(0.3, 0.35, 0.3, "iron"), m, L / 2 - 0.3, 0.78, 0));
  g.add(mk(RC.box(0.3, 0.35, 0.3, "iron"), m, -L / 2 + 0.3, 0.78, 0));
  const lan = RC.lantern(U, 0.9); lan.position.set(-L / 2 + 0.4, 1.35, 0); g.add(lan);
  g.add(mk(RC.box(0.08, 0.55, 0.08, "darkPlanks"), m, -L / 2 + 0.4, 0.95, 0));
  g.userData = {lantern: lan};
  return g;
}
function skiffModel(U, L, B) {
  const g = new THREE.Group(), m = mat(U);
  g.add(new THREE.Mesh(RC.hullGeo(L, B, 0.35, 0.42, L * 0.32, "planks"), RC.litMaterial(U, {side: THREE.DoubleSide})));
  for (const x of [-L * 0.3, 0, L * 0.22]) g.add(mk(RC.box(0.25, 0.06, B * 0.85, "darkPlanks"), m, x, 0.3, 0));
  const man = RC.person(U, 0x8a4a3a); man.position.set(-0.1, 0.1, 0); man.scale.setScalar(0.9); g.add(man);
  const oars = [];
  for (const s of [-1, 1]) {
    const p = new THREE.Group(); p.position.set(0.25, 0.5, s * B * 0.5); g.add(p);
    p.add(mk(RC.box(0.08, 0.06, 2.4, "planks"), m, 0, 0, s * 0.9));
    p.add(mk(RC.box(0.08, 0.3, 0.55, "planks"), m, 0, 0, s * 2.05));
    oars.push(p);
  }
  const mast = mk(RC.box(0.07, 1.4, 0.07, "darkPlanks"), m, L * 0.3, 1.05, 0); g.add(mast);
  const lan = RC.lantern(U, 0.8); lan.position.set(L * 0.3, 1.85, 0); g.add(lan);
  g.userData = {lantern: lan, oars, man};
  return g;
}
function animateOars(model, b, stroke, t) {
  const [po, so] = model.userData.oars;
  const ph = stroke ? Math.sin(t * 5) : 0;
  po.rotation.set(-0.25 - 0.15 * (stroke ? Math.cos(t * 5) : 0), ph * 0.6 * (stroke > 0 ? 1 : -1) + 0.0, 0);
  so.rotation.set(0.25 + 0.15 * (stroke ? Math.cos(t * 5) : 0), -ph * 0.6 * (stroke > 0 ? 1 : -1), 0);
}
function anchorModel(U) {
  const g = new THREE.Group(), m = mat(U);
  g.add(mk(RC.box(0.1, 0.9, 0.1, "iron"), m, 0, 0.45, 0));
  g.add(mk(RC.box(0.08, 0.08, 0.7, "iron"), m, 0, 0.82, 0));
  const a1 = mk(RC.box(0.08, 0.45, 0.1, "iron"), m, 0.17, 0.12, 0); a1.rotation.z = -0.9; g.add(a1);
  const a2 = mk(RC.box(0.08, 0.45, 0.1, "iron"), m, -0.17, 0.12, 0); a2.rotation.z = 0.9; g.add(a2);
  return g;
}

// ---------------- driving a powered or rowed craft ----------------
// tug: throttle notches (-2 to 4) a tap at a time, rudder held. rowed: thrust while held, turn while held.
function tugDrive(t, inp, h) {
  const b = t.hull, target = (inp.left ? 1 : 0) - (inp.right ? 1 : 0);
  t.rud += clamp(target * 0.6 - t.rud, -2 * h, 2 * h);
  b.thrust(-b.L / 2, 0, t.thr / 4 * t.maxT, 0);
  const lx = -b.L / 2, px = b.wx(lx, 0), pz = b.wz(lx, 0);
  t.river.flow(px, pz, OUT); const vx = b.vx - b.w * (pz - b.z), vz = b.vz + b.w * (px - b.x);
  const ua = -((OUT[0] - vx) * Math.cos(b.yaw) + (OUT[1] - vz) * Math.sin(b.yaw));
  const flowR = Math.abs(ua) + 2.6 * Math.max(0, t.thr) / 4;
  b.thrust(lx, 0, 0, t.rudK * t.rud * flowR * flowR * (ua >= -0.05 || t.thr > 0 ? 1 : -1));
}
function rowDrive(b, inp, F) {
  const fwd = (inp.up ? 1 : 0) - (inp.down ? 0.65 : 0), turn = (inp.right ? 1 : 0) - (inp.left ? 1 : 0);
  if (fwd) b.thrust(0, 0, fwd * F.thrust, 0);
  if (turn) { b.thrust(b.L * 0.3, 0, 0, turn * F.turn / b.L); b.thrust(-b.L * 0.3, 0, 0, -turn * F.turn / b.L); }
  return fwd || turn ? (fwd >= 0 ? 1 : -1) : 0;
}

// ======================================================================================================================
// 1. Raft: a log raft on the current, poled where the bed is in reach, paddled where it is not.
RC.scenes.raft = function () {
  const half = x => 8.6 - 2.6 * Math.exp(-(((x - 36) / 13) ** 2));
  const river = {x0: -72, x1: 100, z0: -34, z1: 36, center: x => 4 * Math.sin(x / 26), half,
    depth: x => x > 45 ? 2.9 : 1.6, U: x => 1.0 * 8.6 / half(x), level0: 0.55, slope: 0.002, steps: [{x: 45, drop: 1}],
    rocks: [[2, 2.6], [15, -3.2], [25, 2.4], [62, -2.8]], bed: (x, z) => (x > 28 && x < 62) ? "gravel" : "sand"};
  let ctx, R, raft, model, mode = "Pole", bumpT = 0, wasGround = 0;
  const inp = hold();
  const def = {
    river, posts: [[-38, -1], [40, 1]], followDefault: true,
    home: {yaw: 0.5, pitch: -0.42, dist: 17},
    hold: inp,
    build(c) {
      ctx = c; R = c.river;
      raft = new RC.Hull({L: 4.4, B: 2.4, m: 720, draft: 0.3, cA: 420, cS: 520, nx: 3, nz: 3});
      model = raftModel(c.U); c.scene.add(model);
      def.reset();
    },
    reset() { raft.x = -52; raft.z = R.cfg.center(-52); raft.yaw = 0.15; raft.vx = raft.vz = raft.w = 0; raft.settle(R); bumpT = 0; },
    step(h) {
      raft.water(R);
      const depth = R.surf(raft.x, raft.z) - R.top(raft.x, raft.z);
      mode = depth < 2.7 ? "Pole" : "Paddle";
      const F = mode === "Pole" ? {thrust: 520, turn: 1500} : {thrust: 210, turn: 650};
      rowDrive(raft, inp, F);
      raft.integrate(h, R);
      raft.terrain(R);
      if (raft.bumped > 0) { bumpT = 1.2; raft.bumped = 0; }
      bumpT = Math.max(0, bumpT - h);
      wasGround = raft.grounded;
      if (raft.x > R.cfg.x1 - 6) def.reset();
    },
    sync() {
      place(model, raft);
      const pv = model.userData.pivot, f = (inp.up ? 1 : 0) - (inp.down ? 1 : 0), t = (inp.right ? 1 : 0) - (inp.left ? 1 : 0);
      pv.rotation.set(t * 0.35, 0, f ? -0.5 * f : 0.12);
      pv.scale.y = mode === "Pole" ? 1 : 0.55;
    },
    lanterns: () => [model.userData.lantern],
    follow: () => [raft.x, raft.y, raft.z],
    hud() {
      const depth = R.surf(raft.x, raft.z) - R.top(raft.x, raft.z);
      R.flow(raft.x, raft.z, OUT);
      return [["", mode, "on"], ["Speed", speedOf(raft).toFixed(1) + " m/s"], ["Current", Math.hypot(OUT[0], OUT[1]).toFixed(1) + " m/s"], ["Depth", depth.toFixed(1) + " m"],
        ...(bumpT > 0 ? [["", "Bump", "warn"]] : []), ...(wasGround ? [["", "Aground", "warn"]] : [])];
    },
    keys: "W S pole or paddle · A D turn",
    actions: [],
  };
  return def;
};

// ======================================================================================================================
// 2. Tug: a steam tug with two barges on tow lines that go slack and taut.
RC.scenes.tug = function () {
  const river = {x0: -95, x1: 110, z0: -40, z1: 42, center: x => 6 * Math.sin(x / 45), half: () => 13, depth: () => 2.3, U: () => 0.55,
    level0: 0.55, slope: 0.001, steps: [], rocks: [[55, -7.5]], bed: (x, z) => ((Math.floor(x / 9) + Math.floor(z / 7)) & 3) ? "sand" : "gravel"};
  let ctx, R, tug, b1, b2, mT, m1, m2, L1, L2, r1, r2, t1, t2, lenMode = 0;
  const LEN = [[10, 6], [16, 9]];
  const inp = hold();
  const T = {thr: 0, rud: 0, maxT: 4200, rudK: 55};
  const def = {
    river, posts: [[-30, 1], [30, -1]], followDefault: true,
    home: {yaw: 0.85, pitch: -0.5, dist: 34},
    hold: inp,
    press(d) { if (d === "up") T.thr = Math.min(4, T.thr + 1); if (d === "down") T.thr = Math.max(-2, T.thr - 1); },
    build(c) {
      ctx = c; R = c.river; T.river = R;
      tug = new RC.Hull({L: 6.2, B: 2.3, m: 2600, draft: 0.8, cA: 300, cS: 2200, pointed: true});
      b1 = new RC.Hull({L: 7.5, B: 3.2, m: 4200, draft: 0.7, cA: 600, cS: 3600});
      b2 = new RC.Hull({L: 7.5, B: 3.2, m: 4200, draft: 0.7, cA: 600, cS: 3600});
      T.hull = tug;
      mT = tugModel(c.U, 6.2, 2.3); m1 = bargeModel(c.U, 7.5, 3.2, "crates"); m2 = bargeModel(c.U, 7.5, 3.2, "logs");
      c.scene.add(mT, m1, m2);
      L1 = new RC.Line(tug, [-3.1, 0], b1, [3.75, 0], 10); L2 = new RC.Line(b1, [-3.75, 0], b2, [3.75, 0], 6);
      r1 = new RC.Rope(16); r2 = new RC.Rope(12);
      const rm = mat(c.U); t1 = new RC.Tube(16, 0.045, rm); t2 = new RC.Tube(12, 0.045, rm); c.scene.add(t1.mesh, t2.mesh);
      def.reset();
    },
    reset() {
      const zc = R.cfg.center;
      Object.assign(tug, {x: -40, z: zc(-40), yaw: 0.05, vx: 0, vz: 0, w: 0});
      Object.assign(b1, {x: -54.9, z: zc(-54.9) + 0.6, yaw: 0.1, vx: 0, vz: 0, w: 0});
      Object.assign(b2, {x: -67.5, z: zc(-67.5) - 0.8, yaw: -0.1, vx: 0, vz: 0, w: 0});
      for (const b of [tug, b1, b2]) b.settle(R);
      T.thr = 0; T.rud = 0; L1.on = L2.on = true; L1.length = LEN[lenMode][0]; L2.length = LEN[lenMode][1];
      r1.lay(p3(tug, -3.1, 0, 0.7), p3(b1, 3.75, 0, 0.75)); r2.lay(p3(b1, -3.75, 0, 0.75), p3(b2, 3.75, 0, 0.75));
    },
    step(h) {
      for (const b of [tug, b1, b2]) b.water(R);
      tugDrive(T, inp, h);
      L1.step(); L2.step();
      for (const b of [tug, b1, b2]) b.integrate(h, R);
      for (const b of [tug, b1, b2]) b.terrain(R);
      RC.collide(tug, b1); RC.collide(tug, b2); RC.collide(b1, b2);
      if (L1.on) r1.step(h, R, p3(tug, -3.1, 0, 0.7), p3(b1, 3.75, 0, 0.75), L1.length);
      else r1.step(h, R, p3(tug, -3.1, 0, 0.7), null, L1.length);
      r2.step(h, R, p3(b1, -3.75, 0, 0.75), p3(b2, 3.75, 0, 0.75), L2.length);
    },
    sync() {
      place(mT, tug); place(m1, b1); place(m2, b2);
      t1.set(r1.p); t2.set(r2.p);
    },
    lanterns: () => [mT.userData.lantern, m1.userData.lantern, m2.userData.lantern],
    follow: () => [(tug.x + b1.x) / 2, tug.y, (tug.z + b1.z) / 2],
    hud() {
      const st = l => !l.on ? ["Line", "cast off", ""] : l.tension > 1 ? ["Line", "taut " + fmtN(l.tension), "on"] : ["Line", "slack", ""];
      const a = st(L1), b = st(L2); a[0] = "Tug line"; b[0] = "Barge line";
      return [["Throttle", (T.thr > 0 ? "ahead " : T.thr < 0 ? "astern " : "stop ") + Math.abs(T.thr) + "/" + (T.thr < 0 ? 2 : 4)], ["Rudder", T.rud > 0.05 ? "port" : T.rud < -0.05 ? "starboard" : "amidships"],
        ["Speed", speedOf(tug).toFixed(1) + " m/s"], a, b];
    },
    keys: "W S throttle (a notch a tap) · A D rudder",
    actions: [
      {id: "cast", label: "Cast off", key: "C", run() {
        if (L1.on) { L1.on = false; return; }
        const [ax, az, bx, bz] = L1.ends();
        if (Math.hypot(bx - ax, bz - az) < L1.length + 3) L1.on = true;
      }, label2: () => L1.on ? "Cast off" : "Make fast"},
      {id: "len", label: "Long line", key: "L", run() { lenMode = 1 - lenMode; L1.length = LEN[lenMode][0]; L2.length = LEN[lenMode][1]; }, label2: () => lenMode ? "Short line" : "Long line"},
    ],
  };
  return def;
};

// ======================================================================================================================
// 3. Log drive: logs rolled into the river float down, jam at a narrows, are caught by a boom, bundled and hauled.
RC.scenes.logs = function () {
  const half = x => 9.5 - 5.2 * Math.exp(-(((x + 22) / 6.5) ** 2));
  const river = {x0: -90, x1: 100, z0: -36, z1: 38, center: x => 3 * Math.sin(x / 30), half, depth: () => 1.9, U: x => 1.1 * 9.5 / half(x),
    level0: 0.55, slope: 0.0015, steps: [], rocks: [[-22, 0]], bed: () => "gravel"};
  let ctx, R, U, tug, mT, tow = null, rope, tube, bundle = null, mB = null;
  const logs = [], meshes = [], inp = hold();
  const T = {thr: 0, rud: 0, maxT: 3200, rudK: 45};
  const NL = 14;
  let released = 0, relT = 0, jamT = 0, jammed = false, pryCool = 0, boomOpen = false;
  // the boom: floating logs chained bank to bank, at a slant so the drive slides to its pocket by the right bank
  const BN = 10; const boom = {p: new Float32Array(BN * 2), v: new Float32Array(BN * 2), seg: 0, A: [0, 0], B: [0, 0]};
  let boomMeshes = [];
  const pile = [];   // where each log waits on the rollway
  function boomInit() {
    const c = river.center, xa = 6, xb = 22;
    boom.A = [xa, c(xa) - half(xa) - 1.2]; boom.B = [xb, c(xb) + half(xb) + 1.2];
    const d = Math.hypot(boom.B[0] - boom.A[0], boom.B[1] - boom.A[1]); boom.seg = d * 1.06 / (BN - 1);
    for (let i = 0; i < BN; i++) { const t = i / (BN - 1); boom.p[i * 2] = boom.A[0] + (boom.B[0] - boom.A[0]) * t + Math.sin(Math.PI * t) * 1.5; boom.p[i * 2 + 1] = boom.A[1] + (boom.B[1] - boom.A[1]) * t; boom.v[i * 2] = boom.v[i * 2 + 1] = 0; }
    boomOpen = false;
  }
  function boomStep(h) {
    const p = boom.p, v = boom.v;
    for (let i = 0; i < BN; i++) {
      R.flow(p[i * 2], p[i * 2 + 1], OUT);
      v[i * 2] += (OUT[0] - v[i * 2]) * Math.min(1, 1.5 * h); v[i * 2 + 1] += (OUT[1] - v[i * 2 + 1]) * Math.min(1, 1.5 * h);
      p[i * 2] += v[i * 2] * h; p[i * 2 + 1] += v[i * 2 + 1] * h;
    }
    for (let it = 0; it < 10; it++) {
      p[0] = boom.A[0]; p[1] = boom.A[1]; if (!boomOpen) { p[(BN - 1) * 2] = boom.B[0]; p[(BN - 1) * 2 + 1] = boom.B[1]; }
      for (let i = 0; i < BN - 1; i++) {
        const j = i * 2, k = j + 2; let dx = p[k] - p[j], dz = p[k + 1] - p[j + 1]; const d = Math.hypot(dx, dz);
        if (d <= boom.seg) continue; const cc = (d - boom.seg) / d;
        const wj = i === 0 ? 0 : 1, wk = (i + 1 === BN - 1 && !boomOpen) ? 0 : 1, ws = wj + wk; if (!ws) continue;
        p[j] += dx * cc * wj / ws; p[j + 1] += dz * cc * wj / ws; p[k] -= dx * cc * wk / ws; p[k + 1] -= dz * cc * wk / ws;
      }
    }
    // the banks hold the boom's logs out of the cells
    for (let i = 1; i < BN; i++) { const r = R.solid(p[i * 2], p[i * 2 + 1], R.level(p[i * 2]) - 0.2); if (r && (i < BN - 1 || boomOpen)) { p[i * 2] += r[0] * r[2]; p[i * 2 + 1] += r[1] * r[2]; } }
  }
  const SC = [0, 0];
  function boomHit(b) {   // a hull against the boom's logs
    const s = b.seg(), p = boom.p;
    for (let i = 0; i < BN - 1; i++) {
      const j = i * 2;
      RC.segClosest(s[0], s[1], s[2], s[3], p[j], p[j + 1], p[j + 2], p[j + 3], SC);
      const ax = s[0] + (s[2] - s[0]) * SC[0], az = s[1] + (s[3] - s[1]) * SC[0];
      const bx = p[j] + (p[j + 2] - p[j]) * SC[1], bz = p[j + 1] + (p[j + 3] - p[j + 1]) * SC[1];
      let dx = ax - bx, dz = az - bz; const d = Math.hypot(dx, dz), Rr = b.R + 0.26;
      if (d >= Rr || d < 1e-6) continue;
      dx /= d; dz /= d; const pen = Rr - d;
      b.x += dx * pen * 0.6; b.z += dz * pen * 0.6;
      const wA = (i === 0 ? 0 : 1) * (1 - SC[1]), wB = ((i + 1 === BN - 1 && !boomOpen) ? 0 : 1) * SC[1];
      p[j] -= dx * pen * 0.4 * wA; p[j + 1] -= dz * pen * 0.4 * wA; p[j + 2] -= dx * pen * 0.4 * wB; p[j + 3] -= dz * pen * 0.4 * wB;
      RC.contact(b, null, bx, bz, dx, dz, 0.05, 0.15);
    }
  }
  function nearBoom(b) {
    let best = 99; const p = boom.p;
    for (let i = 0; i < BN - 1; i++) {
      const j = i * 2; RC.segClosest(b.x, b.z, b.x, b.z, p[j], p[j + 1], p[j + 2], p[j + 3], SC);
      best = Math.min(best, Math.hypot(b.x - (p[j] + (p[j + 2] - p[j]) * SC[1]), b.z - (p[j + 1] + (p[j + 3] - p[j + 1]) * SC[1])));
    }
    return best;
  }
  const held = () => logs.filter(l => l.on && nearBoom(l) < 6 && speedOf(l) < 0.35 && l.x < 28);
  const inNarrows = l => l.on && l.x > -28 && l.x < -16;
  function makeLog(i) {
    const l = new RC.Hull({L: 5, B: 0.5, m: 420, draft: 0.26, cA: 40, cS: 450, nx: 3, nz: 1, R: 0.27,
      outline: [[2.5, 0.22], [2.5, -0.22], [-2.5, 0.22], [-2.5, -0.22], [0, 0.25], [0, -0.25], [1.25, 0], [-1.25, 0]]});
    l.on = false; l.i = i; return l;
  }
  function bundleMesh(n) {
    const g = new THREE.Group(), m = mat(U), lg = RC.logGeo(5, 0.25);
    for (let i = 0; i < n; i++) g.add(mk(lg, m, 0, -0.04, -(n - 1) * 0.25 + i * 0.5));
    for (const x of [-1.6, 1.6]) g.add(mk(RC.box(0.12, 0.08, n * 0.5 + 0.1, "iron"), m, x, 0.24, 0));
    return g;
  }
  const def = {
    river, posts: [[-28, -1], [4, -1]], followDefault: false,
    home: {tx: -12, ty: 0, tz: -2, yaw: -1.3, pitch: -0.72, dist: 56},
    hold: inp,
    press(d) { if (d === "up") T.thr = Math.min(4, T.thr + 1); if (d === "down") T.thr = Math.max(-2, T.thr - 1); },
    build(c) {
      ctx = c; R = c.river; U = c.U; T.river = R;
      for (let i = 0; i < NL; i++) { logs.push(makeLog(i)); const m = new THREE.Mesh(RC.logGeo(5, 0.25), mat(c.U)); meshes.push(m); c.scene.add(m); }
      tug = new RC.Hull({L: 4.8, B: 2.0, m: 1500, draft: 0.6, cA: 220, cS: 1600, pointed: true}); T.hull = tug;
      mT = tugModel(c.U, 4.8, 2.0); mT.scale.setScalar(0.92); c.scene.add(mT);
      rope = new RC.Rope(14); tube = new RC.Tube(14, 0.045, mat(c.U)); tube.mesh.visible = false; c.scene.add(tube.mesh);
      const lg = RC.logGeo(boomLen(), 0.26);
      for (let i = 0; i < BN - 1; i++) { const m = new THREE.Mesh(lg, mat(c.U, 0xb8a080)); boomMeshes.push(m); c.scene.add(m); }
      // the rollway: a stack on the left bank, upstream
      for (let i = 0; i < NL; i++) {
        const row = Math.floor(i / 5);
        const zb = river.center(-52) - half(-52) - 3.2;
        pile.push({x: -52, z: zb - (i % 5) * 0.52 - row * 0.26, y: 0, row});
      }
      def.reset();
    },
    reset() {
      boomInit();
      released = 0; relT = 0.4; jamT = 0; jammed = false; pryCool = 0;
      logs.forEach((l, i) => { l.on = false; l.taken = false; const p = pile[i]; l.x = p.x; l.z = p.z; l.yaw = Math.PI / 2 * 0 + 0; l.y = R.top(p.x, p.z) + 0.25 + p.row * 0.44; l.vx = l.vz = l.w = 0; l.pitch = l.roll = 0; });
      for (const m of meshes) m.visible = true;
      if (mB) { ctx.scene.remove(mB); mB = null; } bundle = null;
      Object.assign(tug, {x: 38, z: river.center(38) + 4, yaw: Math.PI - 0.2, vx: 0, vz: 0, w: 0}); tug.settle(R);
      T.thr = 0; T.rud = 0; tow = null; tube.mesh.visible = false;
    },
    step(h) {
      // the crew rolls a log in every second and a half
      relT -= h;
      if (released < NL && relT <= 0) {
        const l = logs[released++]; relT = 1.5;
        const x = -50 + (released % 3) * 2.2, zc = river.center(x);
        l.x = x; l.z = zc - half(x) + 3.4; l.yaw = (released % 2 ? 0.35 : -0.3); l.vx = 0.4; l.vz = 1.1; l.w = 0; l.on = true; l.settle(R);
      }
      const live = logs.filter(l => l.on);
      const bodies = bundle ? [...live, bundle, tug] : [...live, tug];
      for (const b of bodies) b.water(R);
      tugDrive(T, inp, h);
      if (tow) tow.step();
      for (const b of bodies) b.integrate(h, R);
      for (const b of bodies) b.terrain(R);
      for (let i = 0; i < bodies.length; i++) for (let j = i + 1; j < bodies.length; j++) RC.collide(bodies[i], bodies[j], 0.05);
      boomStep(h);
      for (const b of bodies) boomHit(b);
      // a jam: logs stalled in the narrows
      const stalled = live.filter(l => inNarrows(l) && speedOf(l) < 0.12).length;
      jamT = stalled >= 2 ? jamT + h : Math.max(0, jamT - 2 * h);
      jammed = jamT > 2.5;
      pryCool = Math.max(0, pryCool - h);
      for (const l of live) if (l.x > river.x1 - 4) { l.on = false; l.taken = true; }
      if (tow) { const [ax, az, bx, bz] = tow.ends(); rope.step(h, R, [ax, tug.y + 0.6, az], [bx, bundle.y + 0.35, bz], tow.length); }
    },
    sync() {
      logs.forEach((l, i) => { const m = meshes[i]; m.visible = !l.taken; if (m.visible) place(m, l); });
      place(mT, tug);
      if (mB && bundle) place(mB, bundle);
      const p = boom.p;
      for (let i = 0; i < BN - 1; i++) {
        const m = boomMeshes[i], x0 = p[i * 2], z0 = p[i * 2 + 1], x1 = p[i * 2 + 2], z1 = p[i * 2 + 3];
        const mx = (x0 + x1) / 2, mz = (z0 + z1) / 2;
        m.position.set(mx, R.surf(mx, mz) - 0.04, mz); m.rotation.set(0, -Math.atan2(z1 - z0, x1 - x0), 0);
      }
      if (tow) tube.set(rope.p);
    },
    lanterns: () => [mT.userData.lantern],
    follow: () => [tug.x, tug.y, tug.z],
    hud() {
      const onBoom = held().length, afloat = logs.filter(l => l.on).length;
      return [["Rollway", (NL - released) + " left"], ["Afloat", String(afloat)], ["At the boom", String(onBoom)],
        ...(jammed ? [["", "Jam at the narrows", "warn"]] : []),
        ...(bundle ? [["Bundle", bundle.n + " logs"]] : []),
        ["Throttle", (T.thr > 0 ? "ahead " : T.thr < 0 ? "astern " : "stop ") + Math.abs(T.thr)],
        ...(tow ? [["Tow", tow.tension > 1 ? "taut " + fmtN(tow.tension) : "slack", tow.tension > 1 ? "on" : ""]] : [])];
    },
    keys: "W S throttle · A D rudder",
    probe: () => ({logs, boom, tug, bundle}),
    actions: [
      {id: "pry", label: "Break jam", key: "P", run() {
        if (pryCool > 0) return;
        const st = logs.filter(l => l.on && l.x > -36 && l.x < -14 && speedOf(l) < 0.3).sort((a, b) => b.x - a.x);
        if (!st.length) return;
        st.slice(0, 3).forEach((k, n) => {
          R.flow(k.x, k.z, OUT); const fa = Math.atan2(OUT[1], OUT[0]);
          let d = wrap(fa - k.yaw); if (Math.abs(d) > Math.PI / 2) d = wrap(d + Math.PI);
          k.yaw += d * (n ? 0.5 : 0.85); k.w = 0; const push = n ? 0.9 : 1.8;
          k.vx += push * Math.cos(fa); k.vz += push * Math.sin(fa) + Math.sign(R.cfg.center(k.x) - k.z) * 0.5; k.vy += n ? 0.6 : 1.5;
        });
        pryCool = 1.2; jamT = 0;
      }},
      {id: "bundle", label: "Bundle", key: "B", run() {
        if (bundle) return;
        const hs = held().slice(0, 9); if (hs.length < 4) return;
        let x = 0, z = 0; for (const l of hs) { x += l.x; z += l.z; l.on = false; l.taken = true; } x /= hs.length; z /= hs.length;
        R.flow(x, z, OUT);
        bundle = new RC.Hull({L: 5.2, B: hs.length * 0.5, m: hs.length * 420, draft: 0.28, cA: 60 * hs.length, cS: 260 * hs.length, nx: 3, nz: 2});
        bundle.n = hs.length; bundle.x = x; bundle.z = z; bundle.yaw = Math.atan2(OUT[1], OUT[0]); bundle.settle(R);
        mB = bundleMesh(hs.length); ctx.scene.add(mB);
      }},
      {id: "hitch", label: "Hitch", key: "H", run() {
        if (!bundle || tow) { if (tow) { tow = null; tube.mesh.visible = false; } return; }
        const st = [tug.wx(-2.4, 0), tug.wz(-2.4, 0)];
        const ends = [[2.6, 0], [-2.6, 0]].map(e => [e, Math.hypot(bundle.wx(e[0], 0) - st[0], bundle.wz(e[0], 0) - st[1])]).sort((a, b) => a[1] - b[1]);
        if (ends[0][1] > 12) return;
        tow = new RC.Line(tug, [-2.4, 0], bundle, ends[0][0], 8);
        rope.lay([st[0], tug.y + 0.6, st[1]], [bundle.wx(ends[0][0][0], 0), bundle.y + 0.35, bundle.wz(ends[0][0][0], 0)]); tube.mesh.visible = true;
      }, label2: () => tow ? "Unhitch" : "Hitch"},
      {id: "boom", label: "Open boom", key: "O", run() { boomOpen = !boomOpen; }, label2: () => boomOpen ? "Close boom" : "Open boom"},
    ],
  };
  function boomLen() { const c = river.center, xa = 6, xb = 22; return Math.hypot(xb - xa, (c(xb) + half(xb) + 1.2) - (c(xa) - half(xa) - 1.2)) * 1.06 / (BN - 1) + 0.15; }
  return def;
};

// ======================================================================================================================
// 4. Nets: a skiff drags a net from two spars at its stern. Fish schools hold in the eddy and the pool; what the net
// sweeps round is caught, the full net drags harder, and hauling it in lands the catch.
RC.scenes.nets = function () {
  const river = {x0: -95, x1: 70, z0: -38, z1: 40, center: x => 5 * Math.sin(x / 35), half: () => 12, depth: () => 2.2, U: () => 0.65,
    level0: 0.55, slope: 0.001, steps: [], rocks: [[-44, -4]], bed: (x, z) => ((Math.floor(x / 8) * 3 + Math.floor(z / 6)) % 5) ? "sand" : "gravel"};
  let ctx, R, boat, model, net, netMesh, state = "stowed", landed = 0, stroke = 0, simT = 0;
  const inp = hold(), fish = [], fishMeshes = [];
  const SCHOOLS = [[-40, -4.3], [-56, 3.5], [-70, -2]];
  const spars = () => [p3(boat, -2.5, -1.7, 0.0), p3(boat, -2.5, 1.7, 0.0)];
  const def = {
    river, posts: [[-30, 1], [-52, -1]], followDefault: true,
    home: {yaw: -0.6, pitch: -0.55, dist: 20},
    hold: inp,
    build(c) {
      ctx = c; R = c.river;
      boat = new RC.Hull({L: 4.8, B: 1.6, m: 420, draft: 0.3, cA: 70, cS: 1100, pointed: true});
      model = skiffModel(c.U, 4.8, 1.6); c.scene.add(model);
      for (const s of [-1, 1]) model.add(mk(RC.box(0.6, 0.07, 0.07, "darkPlanks"), mat(c.U), -2.35, 0.32, s * 1.25));
      for (const s of [-1, 1]) { const sp = mk(RC.box(0.07, 0.07, 1.0, "darkPlanks"), mat(c.U), -2.5, 0.32, s * 1.25); model.add(sp); }
      net = new RC.Net(13, 3, 0.8); netMesh = net.mesh(c.U); netMesh.visible = false; c.scene.add(netMesh);
      const fg = RC.box(0.42, 0.14, 0.12, "canvas"), tg = RC.box(0.12, 0.16, 0.04, "canvas");
      for (let s = 0; s < SCHOOLS.length; s++) for (let i = 0; i < 7; i++) {
        const f = {s, i, x: 0, y: 0, z: 0, vx: 0, vz: 0, ph: Math.random() * 6, caught: false, landed: false};
        fish.push(f);
        const g = new THREE.Group(); g.add(mk(fg, mat(c.U, 0x9fb0a0))); g.add(mk(tg, mat(c.U, 0x8a9a8c), -0.25, 0, 0));
        fishMeshes.push(g); c.scene.add(g);
      }
      def.reset();
    },
    reset() {
      Object.assign(boat, {x: -22, z: river.center(-22) - 4.2, yaw: Math.PI, vx: 0, vz: 0, w: 0, extra: 0}); boat.settle(R);
      state = "stowed"; net.haul = 0; net.fish = 0; landed = 0; netMesh.visible = false;
      fish.forEach((f, k) => {
        const [sx, sz] = SCHOOLS[f.s], a = f.i / 7 * TAU;
        f.x = sx + Math.cos(a) * 1.4; f.z = river.center(sx) + sz + Math.sin(a) * 1.2; f.vx = f.vz = 0; f.caught = f.landed = false;
        f.y = R.surf(f.x, f.z) - 0.7;
      });
    },
    step(h) {
      simT += h;
      boat.water(R);
      stroke = rowDrive(boat, inp, {thrust: 340, turn: 900});
      if (state === "out" || state === "hauling") {
        const [L, Rr] = spars();
        net.step(h, R, L, Rr);
        boat.force(boat.wx(-2.5, 0), boat.wz(-2.5, 0), net.pull[0], net.pull[1]);
        if (state === "hauling") {
          net.haul = Math.min(1, net.haul + h / 5);
          if (net.haul >= 1) { state = "stowed"; netMesh.visible = false; landed += net.fish; boat.extra += net.fish * 1.5; for (const f of fish) if (f.caught) { f.caught = false; f.landed = true; } net.fish = 0; net.haul = 0; }
        }
      }
      boat.integrate(h, R); boat.terrain(R);
      // the fish: hold station by their school, wander, dart from the hull; caught if the net sweeps round them
      const open = state === "out" && net.area() > 3;
      for (const f of fish) {
        if (f.landed) continue;
        if (f.caught) continue;
        const [sx, sz] = SCHOOLS[f.s], hx = sx + Math.cos(simT * 0.3 + f.ph) * 2.4, hz = river.center(sx) + sz + Math.sin(simT * 0.4 + f.ph * 1.3) * 2.2;
        let ax = (hx - f.x) * 0.6 - f.vx * 0.8, az = (hz - f.z) * 0.6 - f.vz * 0.8;
        const dx = f.x - boat.x, dz = f.z - boat.z, d = Math.hypot(dx, dz);
        if (d < 1.9) { ax += dx / d * 4; az += dz / d * 4; }
        f.vx += ax * h; f.vz += az * h; const sp = Math.hypot(f.vx, f.vz); if (sp > 2.2) { f.vx *= 2.2 / sp; f.vz *= 2.2 / sp; }
        f.x += f.vx * h; f.z += f.vz * h; f.y = R.surf(f.x, f.z) - 0.6 - 0.2 * Math.sin(simT + f.ph);
        if (open && (net.inside(f.x, f.z) || net.touches(f.x, f.z, 0.6))) { f.caught = true; net.fish++; }
      }
      // caught fish ride in the bag
      let k = 0; const C = net.cols;
      for (const f of fish) if (f.caught) {
        const c = Math.round((C - 1) / 2) + ((k % 5) - 2), j = net.idx(clamp(c, 1, C - 2), net.rows - 1);
        f.x = net.p[j] + ((k * 37) % 5 - 2) * 0.07; f.y = net.p[j + 1] + 0.12 + Math.floor(k / 5) * 0.1; f.z = net.p[j + 2] + ((k * 53) % 5 - 2) * 0.07; k++;
      }
    },
    sync() {
      place(model, boat); animateOars(model, boat, stroke, simT);
      if (netMesh.visible) net.draw(netMesh);
      fish.forEach((f, i) => { const m = fishMeshes[i]; m.visible = !f.landed; m.position.set(f.x, f.y, f.z); if (!f.caught) m.rotation.y = -Math.atan2(f.vz, f.vx || 1e-3); });
    },
    lanterns: () => [model.userData.lantern],
    follow: () => [boat.x - 2 * Math.cos(boat.yaw), boat.y, boat.z - 2 * Math.sin(boat.yaw)],
    hud() {
      const pull = Math.hypot(net.pull[0], net.pull[1]);
      return [["Net", state === "out" ? "out" : state === "hauling" ? "hauling " + Math.round(net.haul * 100) + "%" : "stowed", state === "stowed" ? "" : "on"],
        ["In the net", String(net.fish)], ["Landed", String(landed)], ["Speed", speedOf(boat).toFixed(1) + " m/s"],
        ...(state !== "stowed" ? [["Net drag", fmtN(pull)]] : [])];
    },
    keys: "W S row · A D turn",
    probe: () => ({boat, net, fish}),
    actions: [
      {id: "cast", label: "Cast net", key: "C", run() {
        if (state !== "stowed") return;
        const [L, Rr] = spars(); net.lay(L, Rr, [-Math.cos(boat.yaw) * 1.2, -Math.sin(boat.yaw) * 1.2]);
        state = "out"; net.haul = 0; netMesh.visible = true;
      }},
      {id: "haul", label: "Haul in", key: "H", run() { if (state === "out") state = "hauling"; }},
    ],
  };
  return def;
};

// ======================================================================================================================
// 5. Anchor: a skiff drops a bow anchor (and a stern one). The rode pays out, the anchor drags until it bites (or
// ploughs on in sand), the boat swings to the current on it, and hauling the rode short breaks it out to weigh it.
RC.scenes.anchor = function () {
  const BEDS = {
    sand: {tile: "sand", hold: 600, bite: 4.0, cap: 0.25},
    mud: {tile: "mud", hold: 2600, bite: 1.2, cap: 1.0},
    rock: {tile: "stone", hold: 4200, bite: 0, cap: 1.0},
  };
  let bed = "sand";
  const mkRiver = () => ({x0: -70, x1: 80, z0: -36, z1: 38, center: x => 4 * Math.sin(x / 30), half: () => 11, depth: () => 2.0, U: () => 1.45,
    level0: 0.55, slope: 0.002, steps: [], rocks: [], bed: () => BEDS[bed].tile});
  let ctx, R, U, boat, model, stroke = 0, simT = 0;
  const inp = hold();
  const anchors = [];
  function makeAnchor(lx, name) {
    const a = {lx, name, state: "stowed", x: 0, y: 0, z: 0, rode: 0, set: 0, hooked: false, drag: 0, dragV: 0, line: null, rope: new RC.Rope(18, true), tube: null, mesh: null, trail: [], marks: [], target: 0};
    a.line = new RC.Line(boat, [lx, 0], null, null, 1); a.line.at = [0, 0];
    a.tube = new RC.Tube(18, 0.04, mat(U, 0x8a8f96)); a.tube.mesh.visible = false; ctx.scene.add(a.tube.mesh);
    a.mesh = anchorModel(U); ctx.scene.add(a.mesh);
    for (let i = 0; i < 60; i++) { const m = new THREE.Mesh(RC.box(0.32, 0.03, 0.22, "mud"), mat(U, 0x7a6a58)); m.visible = false; ctx.scene.add(m); a.marks.push(m); }
    a.mark = 0; a.lastMark = null;
    return a;
  }
  const bow = a => p3(boat, a.lx, 0, 0.45);
  function anchorStep(a, h) {
    const B = BEDS[bed], b = bow(a);
    if (a.state === "stowed") return;
    if (a.state === "drop") {
      a.y -= 2.4 * h; R.flow(a.x, a.z, OUT); a.x += OUT[0] * 0.15 * h; a.z += OUT[1] * 0.15 * h;
      const floor = R.top(a.x, a.z);
      a.rode = Math.max(a.rode, Math.hypot(b[0] - a.x, b[1] - a.y, b[2] - a.z) + 0.3);
      if (a.y <= floor) { a.y = floor; a.state = "down"; a.set = 0; a.target = Math.max(a.rode, (b[1] - floor) * 5); }
    } else if (a.state === "down") {
      if (a.rode < a.target) a.rode = Math.min(a.target, a.rode + 1.5 * h);     // veering to the scope
      const dy = Math.max(0.2, b[1] - a.y), Lh = Math.sqrt(Math.max(0.04, a.rode * a.rode - dy * dy));
      a.line.length = Lh; a.line.at[0] = a.x; a.line.at[1] = a.z;
      const T = a.line.step();
      // the pull's angle at the anchor: a short rode lifts the shank and breaks it out
      const ang = T > 0 ? Math.atan2(dy, Lh) : 0, fa = clamp(1 - (ang / 0.5) ** 2, 0, 1);
      const setF = bed === "rock" ? (a.hooked ? 1 : 0.02) : a.set;
      const H = B.hold * setF * fa;
      a.dragV = 0;
      if (T > H) {
        const dx = b[0] - a.x, dz = b[2] - a.z, d = Math.hypot(dx, dz) || 1, v = Math.min(1.2, (T - H) / 320);
        a.x += dx / d * v * h; a.z += dz / d * v * h; a.y = R.top(a.x, a.z); a.dragV = v;
        const dist = v * h; a.drag += dist;
        if (bed === "rock") { if (!a.hooked && Math.random() < dist * 0.5) a.hooked = true; }
        else a.set = Math.min(B.cap, a.set + dist / B.bite);
        if (!a.lastMark || Math.hypot(a.x - a.lastMark[0], a.z - a.lastMark[1]) > 0.35) {
          const m = a.marks[a.mark++ % a.marks.length]; m.visible = true; m.position.set(a.x, a.y + 0.02, a.z); m.rotation.y = -Math.atan2(dz, dx); a.lastMark = [a.x, a.z];
        }
      }
      const k = Math.min(1, 1.5 * h); a.T = (a.T || 0) + (T - (a.T || 0)) * k; a.dv = (a.dv || 0) + (a.dragV - (a.dv || 0)) * k;
      if (a.weigh) {
        a.rode = Math.max(dy + 0.2, a.rode - 1.0 * h); a.target = a.rode;
        if (a.rode <= dy + 0.45 && fa < 0.3) { a.state = "up"; }
      }
    } else if (a.state === "up") {
      a.x += (b[0] - a.x) * Math.min(1, 3 * h); a.z += (b[2] - a.z) * Math.min(1, 3 * h); a.y += 1.2 * h;
      a.rode = Math.max(0.3, Math.hypot(b[0] - a.x, b[1] - a.y, b[2] - a.z));
      if (a.y >= b[1] - 0.3) { a.state = "stowed"; a.weigh = false; a.tube.mesh.visible = false; }
    }
    if (a.state !== "stowed") a.rope.step(h, R, b, [a.x, a.y + 0.75, a.z], a.rode);
  }
  function drop(a) {
    if (a.state !== "stowed") return;
    const b = bow(a); a.x = b[0]; a.y = b[1] - 0.3; a.z = b[2]; a.rode = 0.5; a.state = "drop"; a.set = 0; a.hooked = false; a.drag = 0; a.weigh = false;
    a.rope.lay(b, [a.x, a.y, a.z]); a.tube.mesh.visible = true;
  }
  function anchorLabel(a) {
    if (a.state === "stowed") return [a.name, "stowed", ""];
    if (a.state === "drop") return [a.name, "dropping", "on"];
    if (a.state === "up") return [a.name, "aweigh", "on"];
    if (a.weigh && a.dv > 0.02) return [a.name, "breaking out", "on"];
    if (a.dv > 0.02) return [a.name, "dragging " + a.dv.toFixed(2) + " m/s", "warn"];
    if (a.T < 5) return [a.name, "on the bed", ""];
    return [a.name, "holding " + fmtN(a.T), "on"];
  }
  const def = {
    river: mkRiver(), posts: [[-12, -1], [26, 1]], followDefault: true,
    home: {yaw: 2.35, pitch: -0.5, dist: 19},
    hold: inp,
    choice: {id: "bed", label: "Bed", options: [["sand", "Sand"], ["mud", "Mud"], ["rock", "Rock"]], get: () => bed,
      set(v) { bed = v; ctx.setRiver(mkRiver()); R = ctx.river; def.reset(); }},
    build(c) {
      ctx = c; R = c.river; U = c.U;
      boat = new RC.Hull({L: 5, B: 1.8, m: 520, draft: 0.34, cA: 90, cS: 1300, pointed: true});
      model = skiffModel(c.U, 5, 1.8); c.scene.add(model);
      model.userData.lantern.userData.warm = 0;   // an anchor light shows white
      anchors.push(makeAnchor(2.3, "Bow"), makeAnchor(-2.4, "Stern"));
      def.reset();
    },
    reset() {
      Object.assign(boat, {x: -4, z: R.cfg.center(-4) + 1.5, yaw: Math.PI + 0.55, vx: 0, vz: 0, w: 0}); boat.settle(R);
      for (const a of anchors) { a.state = "stowed"; a.weigh = false; a.tube.mesh.visible = false; a.T = 0; a.dv = 0; a.dragV = 0; for (const m of a.marks) m.visible = false; a.lastMark = null; }
    },
    step(h) {
      simT += h;
      boat.water(R);
      stroke = rowDrive(boat, inp, {thrust: 380, turn: 1000});
      for (const a of anchors) anchorStep(a, h);
      boat.integrate(h, R); boat.terrain(R);
      if (boat.x > R.cfg.x1 - 6 || boat.x < R.cfg.x0 + 4) def.reset();
    },
    sync() {
      place(model, boat); animateOars(model, boat, stroke, simT);
      for (const a of anchors) {
        if (a.state === "stowed") { const b = p3(boat, a.lx * 0.92, 0, 0.1); a.mesh.position.set(b[0], b[1], b[2]); a.mesh.rotation.set(0, -boat.yaw, 0); }
        else {
          a.mesh.position.set(a.x, a.y, a.z);
          const b = bow(a), lay = a.state === "down" ? 1.2 : 0;
          a.mesh.rotation.set(0, -Math.atan2(b[2] - a.z, b[0] - a.x), -lay, "YZX");
          a.tube.set(a.rope.p);
        }
      }
    },
    lanterns: () => [model.userData.lantern],
    follow: () => [boat.x, boat.y, boat.z],
    hud() {
      const a = anchors[0], s = anchors[1], b = bow(a);
      const out = [["Bed", BEDS[bed] && bed[0].toUpperCase() + bed.slice(1)], anchorLabel(a), anchorLabel(s)];
      if (a.state === "down") out.push(["Rode", a.rode.toFixed(1) + " m, scope " + (a.rode / Math.max(0.5, b[1] - a.y)).toFixed(1) + ":1"]);
      out.push(["Speed", speedOf(boat).toFixed(2) + " m/s"]);
      return out;
    },
    keys: "W S row · A D turn · R F pay out or shorten the bow rode",
    actions: [
      {id: "bow", label: "Drop bow", key: "Z", run() { const a = anchors[0]; if (a.state === "stowed") drop(a); else if (a.state === "down") a.weigh = !a.weigh; },
        label2: () => { const a = anchors[0]; return a.state === "stowed" ? "Drop bow" : a.weigh ? "Stop hauling" : a.state === "down" ? "Weigh bow" : "Bow anchor"; }},
      {id: "stern", label: "Drop stern", key: "X", run() { const a = anchors[1]; if (a.state === "stowed") drop(a); else if (a.state === "down") a.weigh = !a.weigh; },
        label2: () => { const a = anchors[1]; return a.state === "stowed" ? "Drop stern" : a.weigh ? "Stop hauling" : a.state === "down" ? "Weigh stern" : "Stern anchor"; }},
      {id: "payout", label: "Pay out", key: "R", hold: true, run(dt) { const a = anchors[0]; if (a.state === "down") { a.weigh = false; a.target = a.rode = Math.min(30, a.rode + 1.2 * dt); } }},
      {id: "shorten", label: "Shorten", key: "F", hold: true, run(dt) { const a = anchors[0]; if (a.state === "down") { const dy = bow(a)[1] - a.y; a.target = a.rode = Math.max(dy + 0.3, a.rode - 1.0 * dt); } }},
    ],
  };
  return def;
};
})();
