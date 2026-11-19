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
// 1. Raft: a log raft on the current, rowed with two oars on a rowing frame (the Rowing tab's, RC.Oarsman in
// rowing.js: one stroke clock, the push each blade's drag through the water). No pole (the owner, 2026-11-19: "no pole
// just oar is best"): in the shallows the blades come to rest on the bed and push nothing off it.
RC.scenes.raft = function () {
  const half = x => 8.6 - 2.6 * Math.exp(-(((x - 36) / 13) ** 2));
  const river = {x0: -72, x1: 100, z0: -34, z1: 36, center: x => 4 * Math.sin(x / 26), half,
    depth: x => x > 45 ? 2.9 : 1.6, U: x => 1.0 * 8.6 / half(x), level0: 0.55, slope: 0.002, steps: [{x: 45, drop: 1}],
    rocks: [[2, 2.6], [15, -3.2], [25, 2.4], [62, -2.8]], bed: (x, z) => (x > 28 && x < 62) ? "gravel" : "sand"};
  let ctx, R, raft, crew, model, bumpT = 0, wasGround = 0;
  const inp = hold();
  const raftHold = [{item: "log", n: 12}, {item: "rope", n: 2}, null, null, null, null, null, null, null, null], lantern = {lit: true};
  const def = {
    river, posts: [[-38, -1], [40, 1]], followDefault: true,
    home: {yaw: 0.5, pitch: -0.42, dist: 17},
    hold: inp,
    build(c) {
      ctx = c; R = c.river;
      raft = new RC.Hull({L: 4.4, B: 2.4, m: 720, draft: 0.3, cA: 420, cS: 520, nx: 3, nz: 3});
      crew = RC.Oarsman(c.U); model = crew.model; c.scene.add(model);
      def.reset();
    },
    reset() { raft.x = -52; raft.z = R.cfg.center(-52); raft.yaw = 0.15; raft.vx = raft.vz = raft.w = 0; raft.settle(R); bumpT = 0; crew.reset(); },
    step(h) {
      raft.water(R);
      crew.step(h, raft, R, crew.wanted(inp), null, ctx.S.simT);
      raft.integrate(h, R);
      raft.terrain(R);
      if (raft.bumped > 0) { bumpT = 1.2; raft.bumped = 0; }
      bumpT = Math.max(0, bumpT - h);
      wasGround = raft.grounded;
      if (raft.x > R.cfg.x1 - 6) def.reset();
    },
    sync() { place(model, raft); crew.sync(false); model.userData.lantern.userData.out = !lantern.lit; model.updateMatrixWorld(true); },
    lanterns: () => [model.userData.lantern],
    follow: () => [raft.x, raft.y, raft.z],
    // the boat's menu (boatmenu.js): the raft's hold of ten slots (the game's, vehicle/hold.rs) and its lantern
    menu: {title: "Raft", condition: () => "Dry", hold: raftHold, lantern: {get: () => lantern.lit, set: v => { lantern.lit = v; }}},
    hud() {
      const depth = R.surf(raft.x, raft.z) - R.top(raft.x, raft.z);
      R.flow(raft.x, raft.z, OUT);
      return [["", crew.stage === "Resting" ? "Drifting" : "Rowing", "on"], ["Speed", speedOf(raft).toFixed(1) + " m/s"], ["Current", Math.hypot(OUT[0], OUT[1]).toFixed(1) + " m/s"], ["Depth", depth.toFixed(1) + " m"],
        ...(crew.onBed ? [["", "Oar on the bed", "warn"]] : []), ...(bumpT > 0 ? [["", "Bump", "warn"]] : []), ...(wasGround ? [["", "Aground", "warn"]] : [])];
    },
    keys: "W S row ahead or back · A D one oar to turn",
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
    home: {tx: -14, ty: 0, tz: -2, yaw: -1.0, pitch: -0.72, dist: 40},
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
    home: {yaw: 2.55, pitch: -0.68, dist: 17},
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
    follow: () => [boat.x - 4 * Math.cos(boat.yaw), boat.y, boat.z - 4 * Math.sin(boat.yaw)],
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
// 5. Anchor: a skiff with one anchor at its bow (the owner, 2026-11-19: "also only really need one anchor per boat").
// The rode pays out only until the anchor touches the bed (the owner, 2026-11-19: "by default anchor rope should stop
// extending as soon as it touches ground"); a longer rope is set in the boat's menu (the owner, 2026-11-19: "rope
// length can be controlled in the boat menu, yes boats have menus for inventory and various stuff"). The anchor sets
// on the bed in under two seconds (sand slowest, mud fastest); once set it holds the boat (the owner, 2026-11-19:
// "anchor in mockup should stop you from moving"): rowing and the current move the boat only as far as its swing on
// the one rode. Weighing is the only way to move again.
RC.scenes.anchor = function () {
  // the bed decides only how fast the anchor sets (seconds from touching the bed to holding); while it sets it can
  // slide a little toward the boat, leaving a furrow, and then it holds whatever pulls on it
  const BEDS = {
    sand: {tile: "sand", set_s: 1.8, slide: 600},
    mud: {tile: "mud", set_s: 0.8, slide: 1200},
    rock: {tile: "stone", set_s: 1.3, slide: 900},
  };
  // the rope: null is "to the bed" (it stops paying out the moment the anchor touches); otherwise metres, up to the
  // rode's maximum. A setting of the boat's, kept while the anchor is stowed and used on the next drop.
  const ROPE_MAX = 30, VEER = 1.5, HAUL = 1.0;
  let bed = "sand", rope = null;
  const mkRiver = () => ({x0: -70, x1: 80, z0: -36, z1: 38, center: x => 4 * Math.sin(x / 30), half: () => 11, depth: () => 2.0, U: () => 1.45,
    level0: 0.55, slope: 0.002, steps: [], rocks: [], bed: () => BEDS[bed].tile});
  let ctx, R, U, boat, model, stroke = 0, simT = 0, a = null;
  const inp = hold();
  // the skiff's hold: ten slots, as the game's raft has (vehicle/hold.rs); a few things already in it
  const hold10 = [{item: "rope", n: 4}, {item: "bait", n: 12}, {item: "grilled-fish", n: 3}, null, null, null, null, null, null, null];
  const lantern = {lit: true};
  function makeAnchor(lx) {
    const a = {lx, state: "stowed", x: 0, y: 0, z: 0, rode: 0, set: 0, dragV: 0, line: null, rope: new RC.Rope(18, true), tube: null, mesh: null, marks: [], target: 0};
    a.line = new RC.Line(boat, [lx, 0], null, null, 1); a.line.at = [0, 0];
    a.tube = new RC.Tube(18, 0.09, mat(U, 0xc8ccd2)); a.tube.mesh.visible = false; ctx.scene.add(a.tube.mesh);
    a.mesh = anchorModel(U); a.mesh.scale.setScalar(1.35); ctx.scene.add(a.mesh);
    for (let i = 0; i < 60; i++) { const m = new THREE.Mesh(RC.box(0.5, 0.04, 0.34, "sand"), mat(U, bed === "mud" ? 0x6a5a48 : 0xe8dcc0)); m.visible = false; ctx.scene.add(m); a.marks.push(m); }
    a.mark = 0; a.lastMark = null;
    return a;
  }
  const bow = () => p3(boat, a.lx, 0, 0.45);
  // the rode the setting asks for, given the anchor's height under the bow: never shorter than straight up and down
  const plumb = () => Math.max(0.2, bow()[1] - a.y) + 0.3;
  const wanted = () => rope === null ? plumb() : clamp(rope, plumb(), ROPE_MAX);
  function anchorStep(h) {
    const B = BEDS[bed], b = bow();
    if (a.state === "stowed") return;
    if (a.state === "drop") {
      a.y -= 2.4 * h; R.flow(a.x, a.z, OUT); a.x += OUT[0] * 0.15 * h; a.z += OUT[1] * 0.15 * h;
      const floor = R.top(a.x, a.z);
      a.rode = Math.max(a.rode, Math.hypot(b[0] - a.x, b[1] - a.y, b[2] - a.z) + 0.3);
      // on the bed the rode stops paying out (owner, 2026-11-19), unless the boat's menu has a longer rope set
      if (a.y <= floor) { a.y = floor; a.state = "down"; a.set = 0; a.target = rope === null ? a.rode : Math.max(a.rode, wanted()); }
    } else if (a.state === "down") {
      // veering to a longer setting, or hauling in to a shorter one: the boat swings out or is pulled up
      if (a.rode < a.target) a.rode = Math.min(a.target, a.rode + VEER * h);
      else if (a.rode > a.target) a.rode = Math.max(a.target, a.rode - HAUL * h);
      const dy = Math.max(0.2, b[1] - a.y), Lh = Math.sqrt(Math.max(0.04, a.rode * a.rode - dy * dy));
      a.line.length = Lh; a.line.at[0] = a.x; a.line.at[1] = a.z;
      const T = a.line.step();
      // setting: the flukes dig in over set_s; until then a hard pull slides it a little toward the boat
      a.set = Math.min(1, a.set + h / B.set_s);
      a.dragV = 0;
      if (a.set < 1 && T > B.slide) {
        const dx = b[0] - a.x, dz = b[2] - a.z, d = Math.hypot(dx, dz) || 1, v = Math.min(0.6, (T - B.slide) / 320);
        a.x += dx / d * v * h; a.z += dz / d * v * h; a.y = R.top(a.x, a.z); a.dragV = v;
        if (!a.lastMark || Math.hypot(a.x - a.lastMark[0], a.z - a.lastMark[1]) > 0.35) {
          const m = a.marks[a.mark++ % a.marks.length]; m.visible = true; m.position.set(a.x, a.y + 0.02, a.z); m.rotation.y = -Math.atan2(dz, dx); a.lastMark = [a.x, a.z];
        }
      }
      const k = Math.min(1, 1.5 * h); a.T = (a.T || 0) + (T - (a.T || 0)) * k;
      // weighing: the rode hauled in to straight up and down, then the anchor broken out and lifted. The only way
      // the anchor lets go.
      if (a.weigh) {
        a.rode = Math.max(dy + 0.2, a.rode - HAUL * h); a.target = a.rode;
        if (a.rode <= dy + 0.45) a.state = "up";
      }
    } else if (a.state === "up") {
      a.x += (b[0] - a.x) * Math.min(1, 3 * h); a.z += (b[2] - a.z) * Math.min(1, 3 * h); a.y += 1.2 * h;
      a.rode = Math.max(0.3, Math.hypot(b[0] - a.x, b[1] - a.y, b[2] - a.z));
      if (a.y >= b[1] - 0.3) { a.state = "stowed"; a.weigh = false; a.tube.mesh.visible = false; }
    }
    if (a.state !== "stowed") a.rope.step(h, R, b, [a.x, a.y + 0.75, a.z], a.rode);
  }
  function drop() {
    if (a.state !== "stowed") return;
    const b = bow(); a.x = b[0]; a.y = b[1] - 0.3; a.z = b[2]; a.rode = 0.5; a.state = "drop"; a.set = 0; a.weigh = false;
    a.rope.lay(b, [a.x, a.y, a.z]); a.tube.mesh.visible = true;
  }
  function dropOrWeigh() { if (a.state === "stowed") drop(); else if (a.state === "down") a.weigh = !a.weigh; }
  // the anchor's state in plain words, for the HUD and the boat's menu
  function anchorWord() {
    if (a.state === "stowed") return ["Stowed", ""];
    if (a.state === "drop") return ["Dropping", "on"];
    if (a.state === "up") return ["Coming aboard", "on"];
    if (a.weigh) return ["Weighing", "on"];
    if (a.set < 1) return ["Setting", "on"];
    if (a.rode < a.target - 0.05) return ["Paying out", "on"];
    if (a.rode > a.target + 0.05) return ["Hauling in", "on"];
    return ["Holding", "on"];
  }
  function setRope(v) {
    rope = v === null ? null : clamp(Math.round(v * 2) / 2, 1, ROPE_MAX);
    if (a.state === "down" && !a.weigh) a.target = wanted();
  }
  const def = {
    river: mkRiver(), posts: [[-12, -1], [26, 1]], followDefault: true,
    home: {yaw: 2.75, pitch: -0.62, dist: 21},
    hold: inp,
    choice: {id: "bed", label: "Bed", options: [["sand", "Sand"], ["mud", "Mud"], ["rock", "Rock"]], get: () => bed,
      set(v) { bed = v; ctx.setRiver(mkRiver()); R = ctx.river; def.reset(); }},
    build(c) {
      ctx = c; R = c.river; U = c.U;
      boat = new RC.Hull({L: 5, B: 1.8, m: 520, draft: 0.34, cA: 90, cS: 1300, pointed: true});
      model = skiffModel(c.U, 5, 1.8); c.scene.add(model);
      model.userData.lantern.userData.warm = 0;   // an anchor light shows white
      a = makeAnchor(2.3);
      def.reset();
    },
    reset() {
      Object.assign(boat, {x: -4, z: R.cfg.center(-4) + 1.5, yaw: Math.PI + 0.55, vx: 0, vz: 0, w: 0}); boat.settle(R);
      a.state = "stowed"; a.weigh = false; a.tube.mesh.visible = false; a.T = 0; a.dragV = 0; for (const m of a.marks) m.visible = false; a.lastMark = null;
    },
    step(h) {
      simT += h;
      boat.water(R);
      stroke = rowDrive(boat, inp, {thrust: 380, turn: 1000});
      anchorStep(h);
      boat.integrate(h, R); boat.terrain(R);
      if (boat.x > R.cfg.x1 - 6 || boat.x < R.cfg.x0 + 4) def.reset();
    },
    sync() {
      place(model, boat); animateOars(model, boat, stroke, simT);
      model.userData.lantern.userData.out = !lantern.lit;
      if (a.state === "stowed") { const b = p3(boat, a.lx * 0.92, 0, 0.1); a.mesh.position.set(b[0], b[1], b[2]); a.mesh.rotation.set(0, -boat.yaw, 0); }
      else {
        a.mesh.position.set(a.x, a.y, a.z);
        const b = bow(), lay = a.state === "down" ? 1.2 : 0;
        a.mesh.rotation.set(0, -Math.atan2(b[2] - a.z, b[0] - a.x), -lay, "YZX");
        a.tube.set(a.rope.p);
      }
    },
    lanterns: () => [model.userData.lantern],
    follow: () => a.state === "down" ? [(boat.x * 2 + a.x) / 3, boat.y - 0.5, (boat.z * 2 + a.z) / 3] : [boat.x, boat.y, boat.z],
    hud() {
      const anchored = a.state === "down" && a.set >= 1 && !a.weigh;
      const [w, tone] = anchorWord();
      const out = [...(anchored ? [["", "Anchored", "on"]] : []), ["Bed", bed[0].toUpperCase() + bed.slice(1)], ["Anchor", w, tone]];
      if (a.state === "down") out.push(["Rope", a.rode.toFixed(1) + " m"]);
      if (anchored && a.T >= 5) out.push(["Pull", fmtN(a.T)]);
      out.push(["Speed", speedOf(boat).toFixed(2) + " m/s"]);
      return out;
    },
    keys: "W S row · A D turn · T drop or weigh · C the boat",
    probe: () => ({boat, anchor: a, rope: () => rope}),
    // the boat's menu (boatmenu.js): the hold, the anchor and its rope, the lantern
    menu: {
      title: "Skiff",
      condition: () => "Dry",
      hold: hold10,
      anchor: {
        state: () => anchorWord()[0],
        button: () => a.state === "stowed" ? "Drop" : a.weigh ? "Stop" : a.state === "down" ? "Weigh" : null,
        toggle: () => dropOrWeigh(),
        rope: () => rope, setRope, max: ROPE_MAX,
        rode: () => a.state === "stowed" ? null : a.rode,
      },
      lantern: {get: () => lantern.lit, set: v => { lantern.lit = v; }},
    },
    actions: [
      {id: "anchor", label: "Drop anchor", key: "T", run: dropOrWeigh,
        label2: () => a.state === "stowed" ? "Drop anchor" : a.weigh ? "Stop hauling" : a.state === "down" ? "Weigh anchor" : "Anchor"},
    ],
  };
  return def;
};

// ======================================================================================================================
// 6. Tube: one person in an inner tube, carried by the current and turning slowly, steered a little with the hands.
// The tube is a felted-wool ring sealed with pine pitch and blown up (owner, 2026-11-16: "also what about innertubes?").
// The rider sits back in the ring as a person does in a tube: bottom in the hole, back on the far side of the ring,
// legs over the front with the knees bent and the feet in the water, arms on the rim. Paddling, the hands reach
// down into the water beside the ring and pull back in arcs. Built from the raft rower's blocks (the same sizes and
// colours, lit like the terrain). No lantern (the owner, 2026-11-19: "no light for tube guy"): at night the tube is
// lit only by the bank lamps and the moon.
const TUBE = {ring: 0.45, rim: 0.17, ringY: 0.05, hip: [0.0, 0.04], shoulder_m: 0.5, shoulder_half_m: 0.27, upper_arm_m: 0.3, forearm_m: 0.37,
  lean_idle_deg: 40, lean_paddle_deg: 20, knee: [0.48, 0.46], foot: [0.8, -0.08], stroke_hz: 0.95};
function tubeModel(U) {
  const g = new THREE.Group();
  const ring = new THREE.Mesh(RC.tiled(new THREE.TorusGeometry(TUBE.ring, TUBE.rim, 8, 14), "canvas", 3, 1), RC.litMaterial(U, {tint: 0x3a2a1c}));
  ring.rotation.x = Math.PI / 2; ring.position.y = TUBE.ringY; g.add(ring);
  // the pitched seam: a darker band round the ring
  const seam = new THREE.Mesh(new THREE.TorusGeometry(TUBE.ring, TUBE.rim + 0.005, 4, 14, Math.PI * 0.25), RC.litMaterial(U, {tint: 0x1e1610}));
  seam.rotation.x = Math.PI / 2; seam.position.y = TUBE.ringY; g.add(seam);
  const UNIT = RC.box(1, 1, 1, "canvas");
  const cloth = RC.litMaterial(U, {tint: 0x3f7a6a}), skin = RC.litMaterial(U, {tint: 0xd9a77c}), dark = RC.litMaterial(U, {tint: 0x4a5368}),
    hat = RC.litMaterial(U, {tint: 0x6b5638}), hair = RC.litMaterial(U, {tint: 0x5a4030}), boot = RC.litMaterial(U, {tint: 0x3a3026});
  const hip = new THREE.Group(); hip.position.set(TUBE.hip[0], TUBE.hip[1], 0); g.add(hip);
  hip.add(mk(RC.box(0.26, 0.58, 0.42, "canvas"), cloth, 0, 0.32, 0));
  const head = new THREE.Group(); head.position.set(0, 0.78, 0); hip.add(head);
  head.add(mk(RC.box(0.26, 0.28, 0.26, "canvas"), skin, 0, 0, 0));
  head.add(mk(RC.box(0.05, 0.22, 0.27, "canvas"), hair, -0.11, 0.02, 0));
  head.add(mk(RC.box(0.04, 0.05, 0.05, "canvas"), skin, 0.145, -0.02, 0));
  head.add(mk(RC.box(0.3, 0.07, 0.3, "canvas"), hat, -0.01, 0.17, 0));
  head.add(mk(RC.box(0.14, 0.025, 0.26, "canvas"), hat, 0.2, 0.14, 0));
  const legs = [-1, 1].map(s => {
    const th = mk(UNIT, dark), sh = mk(UNIT, dark), ft = mk(RC.box(0.2, 0.08, 0.12, "canvas"), boot); g.add(th, sh, ft);
    const H = new THREE.Vector3(TUBE.hip[0] + 0.04, TUBE.hip[1] + 0.05, s * 0.11), K = new THREE.Vector3(TUBE.knee[0], TUBE.knee[1], s * 0.12), F = new THREE.Vector3(TUBE.foot[0], TUBE.foot[1], s * 0.13);
    lay(th, H, K, 0.15); lay(sh, K, F, 0.13); ft.position.copy(F); ft.rotation.z = -0.5;
    return {th, sh, ft};
  });
  const arms = [-1, 1].map(() => { const a = {up: mk(UNIT, cloth), fore: mk(UNIT, cloth), hand: mk(RC.box(0.09, 0.09, 0.1, "canvas"), skin)}; g.add(a.up, a.fore, a.hand); return a; });
  g.userData = {hip, arms, legs};
  return g;
}
// a block laid from A to B (a limb)
function lay(o, A, B, w) {
  const d = B.clone().sub(A), L = d.length();
  o.position.copy(A).addScaledVector(d, 0.5);
  o.quaternion.setFromUnitVectors(new THREE.Vector3(0, 1, 0), d.multiplyScalar(1 / Math.max(L, 1e-6)));
  o.scale.set(w, L, w);
}
// Each hand's target, on the rim at rest or round its paddling arc. ph: the hand's phase in its stroke; engage:
// 0 resting .. 1 paddling. A forward stroke goes in at the front, pulls back through the water and lifts out behind;
// a back stroke goes the other way round.
function tubeHand(side, ph, engage) {
  const V = (x, y, z) => new THREE.Vector3(x, y, z);
  const rest = V(0.1, TUBE.ringY + TUBE.rim + 0.07, side * (TUBE.ring + 0.04));
  // out of the water the hand comes back in over the rim, low, never out to the side at shoulder height
  const wet = Math.max(0, Math.sin(ph));
  const arc = V(-0.08 + 0.2 * Math.cos(ph), 0.1 - 0.14 * Math.sin(ph), side * (TUBE.ring + 0.14 + (TUBE.rim + 0.02) * wet));
  return rest.lerp(arc, engage);
}

RC.scenes.tube = function () {
  const half = x => 9.5 - 2.4 * Math.exp(-(((x - 30) / 12) ** 2));
  const river = {x0: -80, x1: 95, z0: -34, z1: 36, center: x => 5 * Math.sin(x / 24), half,
    depth: x => x > 22 && x < 40 ? 1.1 : 1.7, U: x => 0.8 * 9.5 / half(x), level0: 0.55, slope: 0.0025, steps: [],
    rocks: [[-12, 3.0], [8, -3.4], [33, 2.0], [55, -2.6]], bed: (x, z) => (x > 20 && x < 42) ? "gravel" : "sand"};
  let ctx, R, tube, model, carried, onboard = true, bumpT = 0, stroke = 0, bank = {x: 0, z: 0, side: 1};
  // each hand's stroke: its phase, its way round (+1 forward, -1 back) and how far it is into paddling
  const handS = [{ph: 0, dir: 1, engage: 0}, {ph: 0, dir: 1, engage: 0}];
  let lean = TUBE.lean_idle_deg * Math.PI / 180;
  const inp = hold();
  const man = () => model.userData.man;
  function landAt(x, side) {   // stand on the bank beside the river at x
    const c = R.cfg.center(x), hw = R.cfg.half(x);
    bank = {x, z: c + side * (hw + 1.6), side};
  }
  const def = {
    river, posts: [[-40, 1], [48, -1]], followDefault: true,
    home: {yaw: 0.55, pitch: -0.4, dist: 11},
    hold: inp,
    build(c) {
      ctx = c; R = c.river;
      tube = new RC.Hull({L: 1.3, B: 1.3, m: 115, draft: 0.2, cA: 150, cS: 150, nx: 3, nz: 3, R: 0.68});
      model = tubeModel(c.U); c.scene.add(model);
      // the tube as carried on the bank: the same ring, upright on the walker's back
      carried = RC.person(c.U, 0x3f7a6a);
      const ring = new THREE.Mesh(RC.tiled(new THREE.TorusGeometry(TUBE.ring, TUBE.rim, 8, 14), "canvas", 3, 1), RC.litMaterial(c.U, {tint: 0x3a2a1c}));
      ring.position.set(-0.32, 1.05, 0); ring.rotation.y = Math.PI / 2; carried.add(ring);
      carried.visible = false; c.scene.add(carried);
      def.reset();
    },
    reset() {
      tube.x = -62; tube.z = R.cfg.center(-62) - 1; tube.yaw = 0.3; tube.vx = tube.vz = tube.w = 0; tube.settle(R);
      onboard = true; bumpT = 0; carried.visible = false; model.visible = true;
      for (const hs of handS) { hs.ph = 0; hs.engage = 0; hs.dir = 1; }
    },
    step(h) {
      if (!onboard) return;
      tube.water(R);
      // the current's shear across the ring turns it: faster water on one side, a slow spin
      const c = Math.cos(tube.yaw), s = Math.sin(tube.yaw);
      const l = [0, 0], r = [0, 0];
      R.flow(tube.x - s * 0.6, tube.z + c * 0.6, l); R.flow(tube.x + s * 0.6, tube.z - c * 0.6, r);
      const ul = l[0] * c + l[1] * s, ur = r[0] * c + r[1] * s;
      tube.tq += (ul - ur) * 55 - tube.w * 18;
      // hands: a little thrust and a little turn, no more
      stroke = rowDrive(tube, inp, {thrust: 75, turn: 38});
      // the hands: both together for ahead or back; for a turn the outer hand pulls ahead and the inner backs,
      // half a stroke apart (A turns left: the right hand pulls)
      const f = (inp.up ? 1 : 0) - (inp.down ? 1 : 0), turn = (inp.right ? 1 : 0) - (inp.left ? 1 : 0);
      for (let i = 0; i < 2; i++) {
        const side = i ? 1 : -1, hs = handS[i];
        const dir = f ? f : turn ? (side === -turn ? 1 : -1) : 0, want = dir !== 0 ? 1 : 0;
        if (want && hs.engage < 0.05) hs.ph = Math.PI * 1.5 + (f ? 0 : (i ? Math.PI : 0));   // start at the top of the arc
        if (want) hs.dir = dir;
        hs.engage += (want - hs.engage) * Math.min(1, h * 5);
        hs.ph += hs.dir * TUBE.stroke_hz * Math.PI * 2 * h * (want ? 1 : 0.35);
      }
      lean = (TUBE.lean_idle_deg + (TUBE.lean_paddle_deg - TUBE.lean_idle_deg) * Math.max(handS[0].engage, handS[1].engage)) * Math.PI / 180;
      tube.integrate(h, R);
      tube.terrain(R);
      if (tube.bumped > 0) { bumpT = 1.2; tube.bumped = 0; }
      bumpT = Math.max(0, bumpT - h);
      if (tube.x > R.cfg.x1 - 6) def.reset();
    },
    sync() {
      place(model, tube);
      const ud = model.userData;
      const V = (x, y, z) => new THREE.Vector3(x, y, z);
      ud.hip.rotation.set(0, 0, lean);
      for (let i = 0; i < 2; i++) {
        const side = i ? 1 : -1, hs = handS[i], a = ud.arms[i];
        const c = Math.cos(lean), sn = Math.sin(lean);
        const Sh = V(TUBE.hip[0] - sn * TUBE.shoulder_m, TUBE.hip[1] + c * TUBE.shoulder_m, side * TUBE.shoulder_half_m);
        let Hd = tubeHand(side, hs.ph, hs.engage);
        // a hand never goes further than the arm reaches: it stops at the arm's end
        const reach = TUBE.upper_arm_m + TUBE.forearm_m - 0.01, d = Hd.clone().sub(Sh);
        if (d.length() > reach) Hd = Sh.clone().addScaledVector(d.normalize(), reach);
        // the elbow out and down, and up while the hand is in the water, so the arm goes over the rim and not through it
        const wet = hs.engage * Math.max(0, Math.sin(hs.ph));
        const El = RC.ROW.elbowOf(Sh, Hd, TUBE.upper_arm_m, TUBE.forearm_m, V(-0.9, -0.7 + 1.3 * wet, side * 0.45));
        lay(a.up, Sh, El, 0.11); lay(a.fore, El, Hd, 0.1); a.hand.position.copy(Hd);
        a.hand.quaternion.setFromUnitVectors(V(0, 1, 0), Hd.clone().sub(El).normalize());
      }
      if (!onboard) {
        const y = R.top(bank.x, bank.z);
        carried.position.set(bank.x, y, bank.z); carried.rotation.y = bank.side > 0 ? -Math.PI / 2 : Math.PI / 2;
      }
    },
    follow: () => onboard ? [tube.x, tube.y, tube.z] : [bank.x, R.top(bank.x, bank.z) + 1, bank.z],
    hud() {
      if (!onboard) return [["", "On the bank, tube carried", "on"]];
      const depth = R.surf(tube.x, tube.z) - R.top(tube.x, tube.z);
      R.flow(tube.x, tube.z, OUT);
      return [["", "Floating", "on"], ["Speed", speedOf(tube).toFixed(1) + " m/s"], ["Current", Math.hypot(OUT[0], OUT[1]).toFixed(1) + " m/s"],
        ["Turning", Math.abs(tube.w * 180 / Math.PI).toFixed(0) + "°/s"], ["Depth", depth.toFixed(1) + " m"],
        ...(bumpT > 0 ? [["", "Bump", "warn"]] : []), ...(tube.grounded ? [["", "Bottom", "warn"]] : [])];
    },
    keys: "W S paddle · A D turn · E get on or off",
    actions: [
      {id: "onoff", label: "Get off", key: "E", run() {
        if (onboard) {
          // off: wade or swim to the nearer bank and pick the tube up
          const side = tube.z >= R.cfg.center(tube.x) ? 1 : -1;
          landAt(tube.x, side); onboard = false; model.visible = false; carried.visible = true;
        } else {
          // on: walk in where the water is waist deep and sit down in the ring
          const c = R.cfg.center(bank.x), hw = R.cfg.half(bank.x);
          tube.x = bank.x; tube.z = c + bank.side * (hw - 1.8); tube.yaw = 0; tube.vx = tube.vz = tube.w = 0; tube.settle(R);
          onboard = true; model.visible = true; carried.visible = false;
        }
      }, label2: () => onboard ? "Get off" : "Get on"},
    ],
  };
  return def;
};

})();
