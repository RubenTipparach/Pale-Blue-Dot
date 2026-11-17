// Fish behaviour (M37): how a fish comes to the lure, swims and fights, today and proposed.
// TODAY is a port of the game's rules and drawing, constant for constant:
//   crates/pbd-core/src/fauna/school.rs (School::step, acceleration, drawn),
//   crates/pbd-core/src/fishing.rs (Line::step, Floating -> Nibble -> Bite -> Hooked),
//   crates/pbd-app/src/fish.rs draw::build (heading from the last step's velocity),
//   crates/pbd-app/src/fish/bodies.rs sway (Sway::Fish, the tail beat),
//   assets/config/fauna.ron (flock, fishing, perch and minnow).
// PROPOSED keeps every fishing rule (the phases, the nibble and bite timers, the dips, the hook
// window, the fight's tension) and changes only how the fish moves and is drawn.
"use strict";
(() => {
const $ = id => document.getElementById(id);
const clamp = (x, a, b) => Math.min(b, Math.max(a, x));
const lerp = (a, b, t) => a + (b - a) * t;
const ease = t => { t = clamp(t, 0, 1); return t * t * (3 - 2 * t); };
const DEG = Math.PI / 180, TAU = Math.PI * 2;
const V = (x = 0, y = 0, z = 0) => new THREE.Vector3(x, y, z);
const UP = V(0, 1, 0);
function rng(seed){
  let a = seed >>> 0;
  const unit = () => { a |= 0; a = a + 0x6D2B79F5 | 0; let t = Math.imul(a ^ a >>> 15, 1 | a); t = t + Math.imul(t ^ t >>> 7, 61 | t) ^ t; return ((t ^ t >>> 14) >>> 0) / 4294967296; };
  return { unit, range: (lo, hi) => lo + (hi - lo) * unit() };
}
// glam's normalize_or and any_orthonormal_vector, so TODAY falls back exactly as the game does.
function normOr(v, fb){ const l = v.length(); return (l > 0 && isFinite(1 / l)) ? v.divideScalar(l) : fb.clone(); }
function anyOrtho(v){ const s = v.z >= 0 ? 1 : -1, a = -1 / (s + v.z), b = v.x * v.y * a; return V(b, s + v.y * v.y * a, -v.y); }
const flat = v => V(v.x, 0, v.z);

// ------------------------------------------------------------------ the game's numbers (fauna.ron)
const FLOCK = { separation: 2.6, alignment: 1.0, cohesion: 0.7, goal: 0.45, lured: 1.4, lure: 3.0, wall: 6.0,
  separation_m: 0.55, neighbour_m: 2.2, goal_every_s: [6, 12], splash_m: 4, splash_push: 12, spook_s: 2 };
const FISHING = { splash_s: 1.2, scent_every_s: 0.5, scent_reach: 3, nibble_m: 0.45, nibbles: [1, 3], nibble_s: [0.6, 1.4],
  hook_window_s: 0.9, hook_step: 0.13, hook_floor_s: 0.25, bite_dip_m: 0.14, nibble_dip_m: 0.035, reel_mps: 1.8,
  tension_base: 0.35, tension_pull: 0.8, tension_fall: 0.7, stamina_drain: 0.08, stamina_floor: 0.15, land_line_m: 1.3 };
const STEP_HZ = 30, SKIN = 0.25, SURFACE = 0, BED = -3.2;
const SPECIES = {
  perch:  { id: "perch",  name: "Banded perch", length: 0.38, speed: [0.8, 2.4], depth: [0.5, 2.0], count: 10, pull: 0.85, strength: 2 },
  minnow: { id: "minnow", name: "Minnow",       length: 0.14, speed: [1.0, 2.6], depth: [0.3, 1.8], count: 22, pull: 0.4,  strength: 1 },
};
const hookWindow = s => Math.max(FISHING.hook_window_s * (1 - (Math.max(1, s) - 1) * FISHING.hook_step), FISHING.hook_floor_s);

// ------------------------------------------------------------------ proposed tuning (the sliders)
// [key, label, min, max, step, default, unit]. Ranges leave headroom past the default both ways.
const GROUPS = [
  ["Turning and banking", [
    ["turnMax", "Max turn rate", 30, 900, 10, 260, "deg/s"],
    ["headTau", "Heading smoothing", 0.02, 0.8, 0.01, 0.12, "s"],
    ["maxPitch", "Max pitch", 5, 85, 1, 35, "deg"],
    ["bankAt90", "Bank at 90 deg/s turn", 0, 60, 1, 14, "deg"],
    ["maxBank", "Max bank", 0, 80, 1, 28, "deg"],
    ["rollTau", "Roll smoothing", 0.02, 0.8, 0.01, 0.18, "s"],
  ]],
  ["Speed: glide and burst", [
    ["simTurn", "Swim turn limit (sim)", 30, 900, 10, 220, "deg/s"],
    ["burstPeriod", "Burst-glide cycle", 0.3, 6, 0.1, 1.8, "s"],
    ["burstFrac", "Burst share of cycle", 0.05, 0.95, 0.01, 0.35, ""],
    ["glideDrag", "Glide drag", 0, 5, 0.05, 0.9, "1/s"],
    ["depthDrift", "Depth wander", 0, 1, 0.01, 0.18, "m"],
  ]],
  ["Tail beat", [
    ["beatRest", "Beat at rest", 0, 5, 0.05, 1.3, "Hz"],
    ["beatPerBL", "Beat per body length/s", 0, 5, 0.05, 1.0, "Hz"],
    ["ampHover", "Amplitude hovering", 0, 0.2, 0.005, 0.025, "len"],
    ["ampRest", "Amplitude cruising", 0, 0.25, 0.005, 0.05, "len"],
    ["ampBurst", "Amplitude bursting", 0, 0.3, 0.005, 0.11, "len"],
    ["ampGlide", "Amplitude gliding", 0, 0.2, 0.005, 0.012, "len"],
  ]],
  ["Approach and hover", [
    ["approachK", "Approach speed", 0.1, 2, 0.05, 0.75, "x cruise"],
    ["slowRadius", "Slow-down radius", 0.1, 4, 0.05, 1.3, "m"],
    ["standoff", "Hover stand-off (mouth)", 2, 24, 1, 15, "cm"],
    ["hoverDrift", "Hover drift", 0, 15, 0.5, 3, "cm"],
    ["driftPeriod", "Drift period", 0.5, 8, 0.1, 2.6, "s"],
  ]],
  ["Pecks (each nibble)", [
    ["dartTime", "Dart in", 0.04, 0.6, 0.01, 0.14, "s"],
    ["backOff", "Back off", 0.1, 1.2, 0.01, 0.35, "s"],
    ["shakeDeg", "Head shake", 0, 45, 1, 12, "deg"],
    ["shakeHz", "Head shake rate", 1, 16, 0.5, 7, "Hz"],
    ["tugCm", "Tug on the lure", 0, 10, 0.5, 2, "cm"],
    ["circleChance", "Turn away and circle", 0, 1, 0.05, 0.6, "chance"],
    ["circleR", "Circle radius", 0.05, 1.2, 0.01, 0.2, "m"],
  ]],
  ["The take and the bolt", [
    ["takeDart", "Take dart", 0.03, 0.4, 0.01, 0.08, "s"],
    ["takeTurn", "Take turn rate", 60, 1080, 10, 520, "deg/s"],
    ["takeSpeed", "Take run speed", 0.2, 4, 0.05, 1.4, "m/s"],
    ["runDown", "Take dives", 0, 60, 1, 22, "deg"],
    ["boltK", "Bolt speed", 0.5, 3, 0.05, 1.6, "x top"],
  ]],
  ["The fight", [
    ["fightTurn", "Turn rate on the line", 30, 900, 10, 300, "deg/s"],
    ["fightTau", "Follow the line", 0.02, 1, 0.01, 0.22, "s"],
    ["fightShake", "Head shake when reeled", 0, 50, 1, 18, "deg"],
    ["fightShakeHz", "Head shake rate", 0.5, 10, 0.1, 3.5, "Hz"],
    ["flashRoll", "Roll (flash) when reeled", 0, 80, 1, 30, "deg"],
    ["runDive", "Dive on a run", 0, 45, 1, 14, "deg"],
  ]],
];
const P = {};
for (const [, list] of GROUPS) for (const s of list) P[s[0]] = s[5];

// ------------------------------------------------------------------ the school (School::step, faithful; options are PROPOSED's sim-side changes)
class School {
  constructor(sp, anchor, seed, count, reach, proposed){
    this.sp = sp; this.anchor = anchor.clone(); this.rng = rng(seed); this.reach = reach; this.proposed = proposed;
    this.positions = []; this.before = []; this.velocities = []; this.velBefore = []; this.spook = []; this.ph = [];
    this.goal = anchor.clone(); this.goalT = 0; this.lured = false; this.lure = null; this.held = null; this.owned = -1; this.clock = 0;
    for (let i = 0; i < count; i++){
      const p = V(anchor.x + this.rng.range(-1.5, 1.5), this.depthFor(), anchor.z + this.rng.range(-1.5, 1.5));
      const h = normOr(V(this.rng.range(-1, 1), 0, this.rng.range(-1, 1)), V(1, 0, 0));
      this.positions.push(p); this.before.push(p.clone());
      this.velocities.push(h.multiplyScalar(sp.speed[0])); this.velBefore.push(this.velocities[i].clone());
      this.spook.push(0); this.ph.push(this.rng.unit());
    }
    this.goal = this.pickGoal();
  }
  get length(){ return this.positions.length; }
  depthFor(){
    const deepest = Math.max(SURFACE - BED - SKIN - 0.05, SKIN);
    const lo = Math.min(this.sp.depth[0], deepest), hi = Math.max(Math.min(this.sp.depth[1], deepest), lo);
    return SURFACE - this.rng.range(lo, hi);
  }
  pickGoal(){ return V(this.anchor.x + this.rng.range(-this.reach, this.reach), this.depthFor(), this.anchor.z + this.rng.range(-this.reach, this.reach)); }
  centre(){ const c = V(); this.positions.forEach(p => c.add(p)); return c.divideScalar(Math.max(1, this.length)); }
  nearestCalm(point){ let best = -1, bd = 1e9; this.positions.forEach((p, i) => { const d = p.distanceToSquared(point); if (this.spook[i] <= 0 && d < bd){ bd = d; best = i; } }); return best; }
  scent(lure, hold){ this.goal = V(lure.x, this.depthFor(), lure.z); this.goalT = hold; this.lured = true; }
  frighten(i, s){ this.spook[i] = Math.max(this.spook[i], s); }
  acceleration(i, splash){
    const p = this.positions[i];
    const sep = V(), head = V(), mid = V(); let n = 0;
    this.positions.forEach((q, j) => {
      if (j === i) return;
      const d = p.distanceTo(q);
      if (d < FLOCK.separation_m && d > 1e-4) sep.add(p.clone().sub(q).divideScalar(d * d));
      if (d < FLOCK.neighbour_m){ head.add(this.velocities[j]); mid.add(q); n++; }
    });
    const acc = sep.multiplyScalar(FLOCK.separation);
    if (n > 0){
      acc.add(head.divideScalar(n).sub(this.velocities[i]).multiplyScalar(FLOCK.alignment));
      acc.add(mid.divideScalar(n).sub(p).multiplyScalar(FLOCK.cohesion));
    }
    const lure = this.lure && this.lure[0] === i ? this.lure[1] : null;
    let target = this.goal, weight = this.lured ? FLOCK.lured : FLOCK.goal;
    if (lure){ target = lure; weight = FLOCK.lure; }
    if (this.proposed && !lure){
      // PROPOSED: each fish wanders its own little way in depth, so a school is not a flat sheet.
      target = target.clone(); target.y += Math.sin(this.clock * 0.7 + this.ph[i] * TAU) * P.depthDrift;
    }
    const want = target.clone().sub(p);
    acc.add(normOr(want.clone(), V()).multiplyScalar(weight * Math.min(want.length(), 1)));
    const r = p.y, top = SURFACE - SKIN, bottom = BED + SKIN;
    if (r > top - 0.3) acc.y -= FLOCK.wall * (r - (top - 0.3) + 0.1);
    if (r < bottom + 0.3) acc.y += FLOCK.wall * ((bottom + 0.3) - r + 0.1);
    if (splash){
      const d = p.distanceTo(splash);
      if (d < FLOCK.splash_m) acc.add(normOr(p.clone().sub(splash), V()).multiplyScalar(FLOCK.splash_push * (1 - d / FLOCK.splash_m)));
    }
    return acc;
  }
  step(dt, splash){
    this.clock += dt;
    this.before = this.positions.map(p => p.clone()); this.velBefore = this.velocities.map(v => v.clone());
    this.goalT -= dt;
    if (this.goalT <= 0){ this.goal = this.pickGoal(); this.goalT = this.rng.range(FLOCK.goal_every_s[0], FLOCK.goal_every_s[1]); this.lured = false; }
    const accs = this.positions.map((_, i) => this.acceleration(i, splash));
    for (let i = 0; i < this.length; i++){
      if (this.owned === i) continue;   // PROPOSED: the fish at the lure is moved by its choreography
      if (this.held && this.held[0] === i){
        this.velocities[i] = this.held[1].clone().sub(this.positions[i]).divideScalar(dt);
        this.positions[i] = this.held[1].clone();
        continue;
      }
      if (splash && splash.distanceTo(this.positions[i]) < FLOCK.splash_m) this.spook[i] = Math.max(this.spook[i], 1);
      this.spook[i] = Math.max(this.spook[i] - dt, 0);
      let v = this.velocities[i].clone().addScaledVector(accs[i], dt);
      const fright = this.spook[i] > 0 ? 2 : 1;
      const slow = this.sp.speed[0] * 0.5, fast = this.sp.speed[1] * fright;
      if (this.proposed){
        // PROPOSED, sim side: a fish turns at a limited rate, and swims in bursts and glides.
        const was = this.velocities[i], ang = was.angleTo(v), lim = P.simTurn * DEG * dt;
        if (ang > lim && was.lengthSq() > 1e-8){
          const axis = normOr(was.clone().cross(v), UP);
          v = was.clone().normalize().applyAxisAngle(axis, lim).multiplyScalar(v.length());
        }
        const phase = (this.clock / P.burstPeriod + this.ph[i]) % 1;
        let s = v.length();
        if (phase < P.burstFrac) s += (fast * 0.8 - s) * Math.min(1, 2.5 * dt);
        else s *= Math.exp(-P.glideDrag * dt);
        v.setLength(Math.max(s, 1e-4));
      }
      const speed = v.length();
      if (speed > fast) v.multiplyScalar(fast / speed);
      else if (speed < slow) v = normOr(v, V(1, 0, 0)).multiplyScalar(slow);
      const p = this.positions[i].clone().addScaledVector(v, dt);
      p.y = clamp(p.y, BED + SKIN, SURFACE - SKIN);
      this.positions[i] = p; this.velocities[i] = v;
    }
  }
  drawn(i, alpha){
    if (this.held && this.held[0] === i) return this.held[1].clone();
    return this.before[i].clone().lerp(this.positions[i], clamp(alpha, 0, 1));
  }
}

// ------------------------------------------------------------------ the line (Line::step from Floating on, faithful)
class Line {
  constructor(seed, feet){
    this.rng = rng(seed); this.feet = feet.clone();
    this.phase = "floating"; this.float = V(0, SURFACE, 0); this.floatBefore = this.float.clone(); this.dip = 0;
    this.splash_s = FISHING.splash_s; this.check_s = FISHING.scent_every_s; this.timer = 0; this.nibbles = 0; this.nibbleOf = 0;
    this.target = -1; this.tension = 0; this.stamina = 1; this.run = 0; this.run_s = 0; this.line_m = 0; this.heading = V(1, 0, 0);
    this.clock = 0; this.checks = 0; this.events = [];
  }
  lure(){ return this.float.clone().addScaledVector(UP, -0.35); }
  splash(){ return this.splash_s > 0 ? this.float.clone() : null; }
  release(school){ if (this.target >= 0){ school.frighten(this.target, FLOCK.spook_s); } this.target = -1; school.lure = null; school.held = null; }
  hook(school){
    this.phase = "hooked"; this.tension = 0.2; this.stamina = 1; this.run = school.sp.pull; this.run_s = this.rng.range(0.8, 1.6);
    const out = flat(this.float.clone().sub(this.feet)); this.line_m = out.length(); this.heading = normOr(out, V(1, 0, 0));
    this.events.push("hooked");
  }
  step(dt, controls, school){
    this.events.length = 0; this.floatBefore.copy(this.float);
    this.clock += dt; this.splash_s = Math.max(this.splash_s - dt, 0);
    if (this.target >= 0 && this.target >= school.length) this.target = -1;
    school.lure = null; school.held = null;
    switch (this.phase){
      case "floating":
        this.check_s -= dt;
        if (this.check_s <= 0 && this.splash_s <= 0){
          this.check_s = FISHING.scent_every_s;
          if (this.target < 0){
            const lure = this.lure(), d = flat(school.centre().sub(lure)).length();
            if (d <= FISHING.scent_reach * 9){
              if (d > 2.5) school.scent(lure, 3);
              // The game rolls bite * bite_factor * bite_chance each check; the replay takes the second
              // check inside sense range so nobody waits.
              if (d <= 9 && ++this.checks >= 2){ const f = school.nearestCalm(lure); if (f >= 0){ this.target = f; this.events.push("interest"); } }
            }
          }
        }
        if (this.target >= 0){
          const lure = this.lure();
          school.lure = [this.target, lure];
          if (school.positions[this.target].distanceTo(lure) < FISHING.nibble_m){
            this.phase = "nibble"; this.nibbles = this.forceNibbles || Math.round(this.rng.range(FISHING.nibbles[0], FISHING.nibbles[1]));
            this.nibbleOf = this.nibbles; this.timer = this.rng.range(FISHING.nibble_s[0], FISHING.nibble_s[1]); this.events.push("nibble");
          }
        }
        break;
      case "nibble":
        if (this.target < 0){ this.phase = "floating"; break; }
        school.held = [this.target, this.lure()];
        if (controls.pressed){ this.release(school); this.phase = "floating"; this.check_s = 2; this.checks = -1e9; this.events.push("tooEarly"); }
        else {
          this.timer -= dt;
          if (this.timer <= 0){
            this.nibbles = Math.max(this.nibbles - 1, 0);
            if (this.nibbles === 0){ this.timer = hookWindow(school.sp.strength); this.window = this.timer; this.phase = "bite"; this.events.push("bite"); }
            else { this.timer = this.rng.range(FISHING.nibble_s[0], FISHING.nibble_s[1]); this.events.push("nibble"); }
          }
        }
        break;
      case "bite":
        if (this.target < 0){ this.phase = "floating"; break; }
        school.held = [this.target, this.lure()];
        if (controls.pressed) this.hook(school);
        else { this.timer -= dt; if (this.timer <= 0){ this.release(school); this.phase = "floating"; this.check_s = 1.5; this.checks = -1e9; this.events.push("missed"); } }
        break;
      case "hooked": {
        if (this.target < 0){ this.phase = "ready"; break; }
        const pull = school.sp.pull;
        this.run_s -= dt;
        if (this.run_s <= 0){ this.run = this.run > 0 ? 0 : pull; this.run_s = this.run > 0 ? this.rng.range(0.8, 2.0) : this.rng.range(1.2, 3.0); }
        if (controls.held){
          this.line_m -= FISHING.reel_mps * dt * (1 - 0.6 * Math.min(this.run / 1.5, 1));
          this.tension += (FISHING.tension_base + FISHING.tension_pull * this.run * this.stamina) * dt;
        } else { this.tension -= FISHING.tension_fall * dt; this.line_m += this.run * 1.2 * this.stamina * dt; }
        if (this.run > 0 && this.tension > 0.25) this.stamina = Math.max(this.stamina - FISHING.stamina_drain * dt, FISHING.stamina_floor);
        this.tension = Math.max(this.tension, 0); this.line_m = clamp(this.line_m, 0, 45);
        const swing = Math.sin(this.clock * 1.7) * 0.12 * this.run;
        const away = this.heading.clone().applyAxisAngle(UP, swing);
        const point = this.feet.clone().addScaledVector(away, this.line_m);
        this.float = V(point.x, SURFACE - this.dip, point.z);
        school.held = [this.target, this.lure()];
        if (this.tension >= 1){ this.release(school); this.phase = "ready"; this.events.push("snapped"); }
        else if (this.line_m <= FISHING.land_line_m){ this.phase = "ready"; this.events.push("caught"); }
        break;
      }
    }
    if (this.phase === "floating" || this.phase === "nibble" || this.phase === "bite"){
      const want = this.phase === "bite" ? FISHING.bite_dip_m : (this.phase === "nibble" && this.timer < 0.12 ? FISHING.nibble_dip_m : 0);
      this.dip += (want - this.dip) * Math.min(dt * 14, 1);
      this.float.y = SURFACE - this.dip;
    }
  }
}

// ------------------------------------------------------------------ the game's fish meshes, with the tail's sway applied per vertex
const MODELS = Object.fromEntries((window.FISH_MODELS || { models: [] }).models.map(m => [m.id, m]));
let atlas = null, sand = null, timber = null;
function loadTextures(){
  if (atlas) return;
  const L = new THREE.TextureLoader(), px = (t, rep) => { t.magFilter = t.minFilter = THREE.NearestFilter; t.generateMipmaps = false; t.encoding = THREE.sRGBEncoding; if (rep){ t.wrapS = t.wrapT = THREE.RepeatWrapping; } return t; };
  atlas = px(L.load("fish-behaviour/game/water_life_atlas.png")); atlas.flipY = false;
  sand = px(L.load("fish-behaviour/game/sand.png"), true); sand.repeat.set(20, 20);   // repeat_m 2.0 on a 40 m bed
  timber = px(L.load("fish-behaviour/game/timber.png"), true);
}
function fishMesh(id){
  const m = MODELS[id], g = new THREE.BufferGeometry(), n = m.pos.length / 3, col = new Float32Array(n * 3);
  for (let i = 0; i < n * 3; i++) col[i] = m.col[i];
  g.setAttribute("position", new THREE.Float32BufferAttribute(m.pos.slice(), 3));
  g.setAttribute("uv", new THREE.Float32BufferAttribute(m.uv, 2));
  g.setAttribute("color", new THREE.Float32BufferAttribute(col, 3));
  const mesh = new THREE.Mesh(g, new THREE.MeshStandardMaterial({ map: atlas, vertexColors: true, roughness: 0.8, side: THREE.DoubleSide, flatShading: true }));
  mesh.matrixAutoUpdate = false; mesh.frustumCulled = false;
  return { mesh, base: new Float32Array(m.pos), attr: g.attributes.position };
}
const _m = new THREE.Matrix4();
function placeFish(fm, origin, fwd, top, scale, sway){
  const a = fm.attr.array, b = fm.base;
  for (let i = 0; i < b.length; i += 3){ a[i] = b[i]; a[i + 1] = b[i + 1]; a[i + 2] = b[i + 2] + sway(b[i]); }
  fm.attr.needsUpdate = true;
  const side = fwd.clone().cross(top);
  _m.makeBasis(fwd.clone().multiplyScalar(scale), top.clone().multiplyScalar(scale), side.multiplyScalar(scale)).setPosition(origin);
  fm.mesh.matrix.copy(_m);
}
const fishSway = (t, phase) => x => { const w = clamp((0.3 - x) / 0.8, 0, 1); return Math.sin(t * 7 + phase - x * 6) * 0.075 * w * w; };

// ------------------------------------------------------------------ PROPOSED body: heading that turns, banks, and a tail tied to speed
class Body {
  constructor(pos, vel, len, phase){
    this.pos = pos.clone(); this.fwd = normOr(flat(vel.clone()), V(1, 0, 0)); this.len = len;
    this.roll = 0; this.yawRate = 0; this.speed = vel.length(); this.prevSpeed = this.speed; this.accel = 0;
    this.tail = phase * TAU; this.amp = P.ampRest; this.shake = 0; this.extraRoll = 0;
  }
  // Turn by yaw and pitch, each eased and rate-limited, so a half turn goes round level and never
  // through nose-down; pitch is held inside the max pitch.
  turnToward(dir, rateDeg, tau, dt){
    if (dir.lengthSq() < 1e-10 || dt <= 0) return;
    const d = dir.clone().normalize(), f = this.fwd;
    const yaw0 = Math.atan2(-f.z, f.x), pit0 = Math.asin(clamp(f.y, -1, 1));
    const yawT = Math.atan2(-d.z, d.x), lim = P.maxPitch * DEG, pitT = clamp(Math.asin(clamp(d.y, -1, 1)), -lim, lim);
    const k = 1 - Math.exp(-dt / Math.max(tau, 1e-3)), rate = rateDeg * DEG * dt;
    let dy = yawT - yaw0; dy = Math.atan2(Math.sin(dy), Math.cos(dy));
    if (Math.abs(d.x) + Math.abs(d.z) < 1e-6) dy = 0;
    const sy = Math.sign(dy) * Math.min(Math.abs(dy) * k, rate);
    const dp = pitT - pit0, sp = Math.sign(dp) * Math.min(Math.abs(dp) * k, rate * 0.6);
    const yaw = yaw0 + sy, pit = clamp(pit0 + sp, -lim, lim);
    f.set(Math.cos(pit) * Math.cos(yaw), Math.sin(pit), -Math.cos(pit) * Math.sin(yaw));
    this.yawRate += (sy / dt - this.yawRate) * Math.min(1, dt / 0.08);
  }
  // Tail: frequency from speed in body lengths a second; amplitude from what the fish is doing.
  beat(dt, mode){
    this.accel += ((this.speed - this.prevSpeed) / Math.max(dt, 1e-4) - this.accel) * Math.min(1, dt / 0.15); this.prevSpeed = this.speed;
    let want;
    if (mode === "hover") want = P.ampHover;
    else if (mode === "burst" || this.accel > 0.25) want = P.ampBurst;
    else if (this.accel < -0.15 && this.speed > 0.2) want = P.ampGlide;
    else want = lerp(P.ampRest, P.ampBurst, clamp(this.speed / 2.5 - 0.2, 0, 1));
    this.amp += (want - this.amp) * Math.min(1, dt / 0.15);
    const hz = P.beatRest + P.beatPerBL * this.speed / Math.max(this.len, 0.05) * (mode === "glide" ? 0.3 : 1);
    this.tail += TAU * Math.min(hz, 14) * dt;
    const bankWant = clamp(-(this.yawRate / DEG) / 90 * P.bankAt90, -P.maxBank, P.maxBank) * DEG;
    this.roll += (bankWant - this.roll) * Math.min(1, dt / Math.max(P.rollTau, 1e-3));
  }
  frame(){
    let fwd = this.fwd.clone();
    const side0 = normOr(fwd.clone().cross(UP), V(0, 0, 1)), top0 = side0.clone().cross(fwd).normalize();
    const r = this.roll + this.extraRoll;
    let top = top0.clone().multiplyScalar(Math.cos(r)).addScaledVector(side0, Math.sin(r)).normalize();
    if (this.shake){ fwd.applyAxisAngle(top, this.shake); }
    return { fwd, top };
  }
  sway(){ const amp = this.amp, ph = this.tail; return x => { const w = clamp((0.3 - x) / 0.8, 0, 1); return Math.sin(ph - x * 6) * amp * w * w; }; }
  mouth(){ return this.pos.clone().addScaledVector(this.fwd, 0.5 * this.len); }
}

// ------------------------------------------------------------------ PROPOSED choreography of the fish at the lure, read off the line's phase and timer
class Choreo {
  constructor(world){ this.w = world; this.i = -1; this.mode = "none"; this.r = rng(world.seed ^ 0x51ed); }
  get body(){ return this.w.bodies[this.i]; }
  begin(i){
    const w = this.w; this.i = i; w.school.owned = i; this.mode = "approach";
    const b = this.body, lure = w.lureNow();
    this.side = normOr(flat(b.pos.clone().sub(lure)), V(1, 0, 0));   // which side of the lure it comes up on
    this.lastTimer = 1e9; this.t = 0;
  }
  station(lure){ const b = this.body; return lure.clone().addScaledVector(this.side, P.standoff / 100 + 0.5 * b.len).add(V(0, -0.03, 0)); }
  end(bolt){
    const w = this.w, b = this.body;
    if (bolt){ this.mode = "bolt"; this.t = 0; this.boltDir = normOr(flat(b.pos.clone().sub(w.lureNow())), V(1, 0, 0)).add(V(0, -0.25, 0)).normalize(); return; }
    w.school.owned = -1; this.i = -1; this.mode = "none";
  }
  update(dt, line){
    if (this.i < 0) return;
    const w = this.w, b = this.body, sp = w.sp, lure = w.lureNow();
    const cruise = (sp.speed[0] + sp.speed[1]) * 0.5;
    this.t += dt;
    const kept = w.lureOffset.clone().multiplyScalar(Math.exp(-dt * 3));
    w.lureOffset.set(0, 0, 0); b.shake = 0; b.extraRoll = 0;
    let mode = "cruise";
    if (this.mode === "approach" && line.phase === "nibble"){ this.mode = "nibble"; this.backFrom = null; this.circle = false; }
    if ((this.mode === "approach" || this.mode === "nibble") && line.phase === "bite"){ this.mode = "bite"; this.t = 0; this.from = b.pos.clone();
      this.runDir = normOr(flat(lure.clone().sub(line.feet)), V(1, 0, 0)).applyAxisAngle(UP, (this.r.unit() < 0.5 ? -1 : 1) * 0.6);
      this.runDir.y = -Math.tan(P.runDown * DEG); this.runDir.normalize(); }
    if (this.mode === "bite" && line.phase === "hooked"){ this.mode = "hooked"; }
    if ((this.mode === "nibble" || this.mode === "bite" || this.mode === "approach") && line.target < 0 && line.phase === "floating"){ this.end(true); }
    if (this.mode === "approach"){
      const st = this.station(lure), to = st.clone().sub(b.pos), d = to.length();
      const want = cruise * P.approachK * clamp(d / P.slowRadius, 0.04, 1);
      b.speed += clamp(want - b.speed, -3 * dt, 2 * dt);
      const face = d > 0.3 ? to.clone() : to.clone().normalize().lerp(lure.clone().sub(b.pos).normalize(), 1 - d / 0.3);
      b.turnToward(face, P.turnMax, P.headTau, dt);
      b.pos.addScaledVector(b.fwd, b.speed * dt);
      if (d < 0.35) b.pos.add(to.multiplyScalar(1 - Math.exp(-dt * 2)));
      mode = d < 0.3 ? "hover" : "cruise";
    } else if (this.mode === "nibble"){
      // Each nibble: hover a body length off, dart in so the mouth meets the lure as the float dips
      // (the game dips it in the last 0.12 s of each nibble timer), shake and tug, back off; maybe circle.
      const rem = line.timer - w.alpha / STEP_HZ;
      if (line.timer > this.lastTimer + 1e-3){ // a new nibble began
        this.T = line.timer; this.t = 0; this.backFrom = b.pos.clone();
        const loop = this.T - P.dartTime - 0.12 - 0.12;
        this.circle = loop > 0.45 && this.r.unit() < P.circleChance; this.loopDur = loop; this.loopDir = this.r.unit() < 0.5 ? -1 : 1;
      }
      this.lastTimer = line.timer;
      const st = this.station(lure).add(V(Math.sin(this.t * TAU / P.driftPeriod) * P.hoverDrift / 100, Math.sin(this.t * TAU / P.driftPeriod * 0.7 + 1) * P.hoverDrift / 200, 0));
      const faceDir = normOr(lure.clone().sub(st), this.side.clone().negate());
      const contact = lure.clone().addScaledVector(faceDir, -0.5 * b.len - 0.01);
      const prev = b.pos.clone();
      if (rem <= 0.12){
        const k = 1 - rem / 0.12;
        b.pos.lerp(contact, 1 - Math.exp(-dt * 30));
        b.turnToward(faceDir, 900, 0.03, dt);
        b.shake = P.shakeDeg * DEG * Math.sin(TAU * P.shakeHz * (0.12 - rem)) * (1 - 0.5 * k);
        w.lureOffset.copy(faceDir).multiplyScalar(-P.tugCm / 100 * Math.sin(Math.PI * clamp(k, 0, 1)));
        mode = "burst"; this.dartFrom = null;
      } else if (rem <= 0.12 + P.dartTime){
        if (!this.dartFrom) this.dartFrom = b.pos.clone();
        const k = ease(1 - (rem - 0.12) / P.dartTime);
        b.pos.copy(this.dartFrom).lerp(contact, k);
        b.turnToward(faceDir, 900, 0.04, dt); mode = "burst";
      } else if (this.backFrom && !this.circle && this.t < P.backOff){
        b.pos.copy(this.backFrom).lerp(st, ease(this.t / P.backOff));
        b.turnToward(lure.clone().sub(b.pos), P.turnMax, P.headTau, dt); mode = "hover";
      } else if (this.circle && this.t < this.loopDur){
        // Turn away and swim a small loop, coming back to the stand-off point facing the lure.
        const u = ease(this.t / this.loopDur), th = u * TAU * this.loopDir;
        const side = faceDir.clone().cross(UP).normalize();
        const c = st.clone().addScaledVector(faceDir, -P.circleR);
        const at = c.clone().addScaledVector(faceDir, P.circleR * Math.cos(th)).addScaledVector(side, P.circleR * Math.sin(th) * 0.9);
        at.y += Math.sin(u * Math.PI) * 0.08;
        if (this.backFrom) at.addScaledVector(this.backFrom.clone().sub(st), 1 - ease(u / 0.3));   // leave from where the peck ended
        b.pos.copy(at);
        const tan = at.clone().sub(prev);
        if (tan.lengthSq() > 1e-10) b.turnToward(tan, P.turnMax, P.headTau * 0.6, dt);
        mode = "cruise";
      } else {
        b.pos.lerp(st, 1 - Math.exp(-dt * 3));
        b.turnToward(lure.clone().sub(b.pos), P.turnMax, P.headTau * 2, dt); mode = "hover";
      }
      b.speed = prev.distanceTo(b.pos) / Math.max(dt, 1e-4);
    } else if (this.mode === "bite"){
      // The take: a quick dart onto the lure, then a hard turn and a run down and away; the lure goes
      // with the mouth and drags the float under.
      const prev = b.pos.clone();
      if (this.t < P.takeDart){
        const faceDir = normOr(lure.clone().sub(b.pos), this.side.clone().negate());
        const contact = lure.clone().addScaledVector(faceDir, -0.5 * b.len);
        b.pos.copy(this.from).lerp(contact, ease(this.t / P.takeDart));
        b.turnToward(faceDir, 900, 0.03, dt); this.hold = null;
      } else {
        b.turnToward(this.runDir, P.takeTurn, 0.05, dt);
        const s = P.takeSpeed * Math.exp(-(this.t - P.takeDart) * 1.5);
        if (!this.hold) this.hold = lure.clone();
        this.hold.addScaledVector(b.fwd, s * dt);
        const off = this.hold.clone().sub(lure); const max = 0.3;
        if (off.length() > max) this.hold.copy(lure).add(off.setLength(max));
        w.lureOffset.copy(this.hold).sub(lure);
        b.pos.copy(this.hold).addScaledVector(b.fwd, -0.5 * b.len);
      }
      b.speed = prev.distanceTo(b.pos) / Math.max(dt, 1e-4); mode = "burst";
    } else if (this.mode === "hooked"){
      w.lureOffset.copy(kept);   // a lure the take pulled aside eases back onto the line
      mode = fight(b, line, lure.clone().add(kept), dt);
    } else if (this.mode === "bolt"){
      const s = sp.speed[1] * P.boltK * Math.exp(-this.t * 1.2);
      b.turnToward(this.boltDir, P.takeTurn, 0.05, dt);
      b.speed = s; b.pos.addScaledVector(b.fwd, s * dt);
      b.pos.y = clamp(b.pos.y, BED + SKIN, SURFACE - SKIN);
      mode = "burst";
      if (this.t > 1.2){ const i = this.i; w.school.positions[i].copy(b.pos); w.school.before[i].copy(b.pos); w.school.velocities[i] = b.fwd.clone().multiplyScalar(Math.max(s, sp.speed[0])); this.end(false); }
    }
    b.beat(dt, mode);
    if (this.i >= 0){ const i = this.i; w.school.positions[i].copy(b.pos); w.school.before[i].copy(b.pos); w.school.velocities[i] = b.fwd.clone().multiplyScalar(b.speed); }
  }
}
// PROPOSED fight: the mouth stays on the lure and the body swings round it, along the line.
// Running: head away from the rod, diving a little, across with the line's swing.
// Being reeled: the hook pulls the head toward the rod; the fish shakes its head and rolls, flashing.
function fight(b, line, lure, dt){
  const away = normOr(flat(lure.clone().sub(line.feet)), V(1, 0, 0));
  const prev = b.pos.clone();
  let mode;
  if (line.run > 0){
    const want = away.clone(); want.y = -Math.tan(P.runDive * DEG); want.normalize();
    b.turnToward(want, P.fightTurn, P.fightTau, dt); b.shake = 0; b.extraRoll = 0; mode = "burst";
  } else {
    const want = away.clone().negate(); want.y = 0.12;
    b.turnToward(want, P.fightTurn, P.fightTau, dt);
    b.shake = P.fightShake * DEG * Math.sin(line.clock * TAU * P.fightShakeHz);
    b.extraRoll = P.flashRoll * DEG * Math.sin(line.clock * TAU * P.fightShakeHz * 0.5 + 0.7);
    mode = "cruise";
  }
  b.pos.copy(lure).addScaledVector(b.fwd, -0.5 * b.len);
  b.speed = Math.max(prev.distanceTo(b.pos) / Math.max(dt, 1e-4), line.run > 0 ? 1.2 : 0.4);
  return mode;
}

// ------------------------------------------------------------------ the scene: water seen from under, the bed, a dock with a lantern, the float and line
function environment(scene){
  const hemi = new THREE.HemisphereLight("#bfefff", "#0a3040", 0.75); scene.add(hemi);
  const sun = new THREE.DirectionalLight("#e8fbff", 0.9); sun.position.set(-3, 10, 4); scene.add(sun);
  const lantern = new THREE.PointLight("#ffb55e", 0, 12, 2); lantern.position.set(-2.95, 1.05, -1.65); scene.add(lantern);
  const under = new THREE.PointLight("#ffb55e", 0, 6, 2); under.position.set(-1.3, -0.9, -0.9); scene.add(under);   // the lantern's pool, through the surface
  const surfMat = new THREE.MeshStandardMaterial({ color: "#3c8ea0", roughness: 0.3, metalness: 0.0, transparent: true, opacity: 0.82, side: THREE.DoubleSide, flatShading: true, depthWrite: false });
  const sg = new THREE.PlaneGeometry(60, 60, 60, 60); sg.rotateX(-Math.PI / 2);
  const surf = new THREE.Mesh(sg, surfMat); scene.add(surf);
  const base = Float32Array.from(sg.attributes.position.array);
  const bed = new THREE.Mesh(new THREE.PlaneGeometry(40, 40), new THREE.MeshStandardMaterial({ map: sand, roughness: 1 }));
  bed.rotation.x = -Math.PI / 2; bed.position.y = BED; scene.add(bed);
  // The dock: timber piles and a deck, the lantern on a post.
  const wood = new THREE.MeshStandardMaterial({ map: timber, roughness: 0.95 });
  const dock = new THREE.Group(); scene.add(dock);
  [[-3.6, -3.0], [-3.6, -1.6], [-2.9, -3.0], [-2.9, -1.6]].forEach(([x, z]) => { const p = new THREE.Mesh(new THREE.BoxGeometry(0.22, 4.3, 0.22), wood); p.position.set(x, BED + 2.15, z); dock.add(p); });
  const deck = new THREE.Mesh(new THREE.BoxGeometry(1.0, 0.12, 1.8), wood); deck.position.set(-3.25, 0.55, -2.3); dock.add(deck);
  const post = new THREE.Mesh(new THREE.BoxGeometry(0.1, 0.5, 0.1), wood); post.position.set(-2.95, 0.8, -1.65); dock.add(post);
  const lampMat = new THREE.MeshStandardMaterial({ color: "#3a2a18", emissive: "#ffb050", emissiveIntensity: 0.2 });
  const lamp = new THREE.Mesh(new THREE.BoxGeometry(0.14, 0.18, 0.14), lampMat); lamp.position.set(-2.95, 1.12, -1.65); dock.add(lamp);
  return {
    night(n){
      scene.background = new THREE.Color(n ? "#03090d" : "#0f4656");
      scene.fog = new THREE.FogExp2(scene.background, n ? 0.30 : 0.16);
      hemi.intensity = n ? 0.035 : 0.75; sun.intensity = n ? 0.03 : 0.9;
      lantern.intensity = n ? 2.4 : 0; under.intensity = n ? 1.3 : 0; lampMat.emissiveIntensity = n ? 2.2 : 0.2;
      surfMat.color.set(n ? "#0b2530" : "#3c8ea0");
    },
    waves(t){
      const a = sg.attributes.position.array;
      for (let i = 0; i < a.length; i += 3){ const x = base[i], z = base[i + 2]; a[i + 1] = 0.03 * Math.sin(x * 1.3 + t * 1.1) + 0.02 * Math.sin(z * 1.7 - t * 0.9); }
      sg.attributes.position.needsUpdate = true;
    },
  };
}
function tackle(scene, proposed){
  const g = new THREE.Group(); scene.add(g);
  const red = new THREE.Mesh(new THREE.SphereGeometry(0.06, 12, 8), new THREE.MeshStandardMaterial({ color: new THREE.Color().setRGB(0.85, 0.2, 0.15).convertSRGBToLinear(), roughness: 0.5 }));
  red.position.y = 0.03;
  const white = new THREE.Mesh(new THREE.SphereGeometry(0.055, 12, 8), new THREE.MeshStandardMaterial({ color: new THREE.Color().setRGB(0.93, 0.93, 0.9).convertSRGBToLinear(), roughness: 0.6 }));
  white.position.y = -0.02;
  g.add(red, white);
  const lineGeo = new THREE.BufferGeometry().setFromPoints([V(), V()]);
  const line = new THREE.Line(lineGeo, new THREE.LineBasicMaterial({ color: "#dfe8ea", transparent: true, opacity: 0.7 })); line.frustumCulled = false; scene.add(line);
  let leader = null, hook = null;
  if (proposed){
    const lg = new THREE.BufferGeometry().setFromPoints([V(), V()]);
    leader = new THREE.Line(lg, new THREE.LineBasicMaterial({ color: "#c9d6d8", transparent: true, opacity: 0.55 })); leader.frustumCulled = false; scene.add(leader);
    hook = new THREE.Group();
    const bait = new THREE.Mesh(new THREE.BoxGeometry(0.018, 0.035, 0.018), new THREE.MeshStandardMaterial({ color: "#b0605a", roughness: 0.9 }));
    const shank = new THREE.Mesh(new THREE.BoxGeometry(0.004, 0.03, 0.004), new THREE.MeshStandardMaterial({ color: "#5a5f63", metalness: 0.6, roughness: 0.4 }));
    shank.position.y = 0.02; hook.add(bait, shank); scene.add(hook);
  }
  const TIP = V(-8.2, 2.4, 0);
  return {
    set(float, lure){
      g.position.copy(float);
      lineGeo.attributes.position.setXYZ(0, float.x, float.y + 0.05, float.z); lineGeo.attributes.position.setXYZ(1, TIP.x, TIP.y, TIP.z); lineGeo.attributes.position.needsUpdate = true;
      if (leader){
        leader.geometry.attributes.position.setXYZ(0, float.x, float.y - 0.07, float.z); leader.geometry.attributes.position.setXYZ(1, lure.x, lure.y + 0.03, lure.z); leader.geometry.attributes.position.needsUpdate = true;
        hook.position.copy(lure);
      }
    },
    visible(v){ g.visible = v; line.visible = v; if (leader){ leader.visible = v; hook.visible = v; } },
  };
}

// ------------------------------------------------------------------ one world: a school, maybe a line, drawn TODAY or PROPOSED
const FEET = V(-9, 0.3, 0.6);
class World {
  constructor(kind, tab, spId, seed){
    this.kind = kind; this.tab = tab; this.sp = SPECIES[spId]; this.seed = seed; this.proposed = kind === "proposed";
    this.scene = new THREE.Scene(); this.env = environment(this.scene);
    this.clock = 0; this.acc = 0; this.alpha = 0; this.lureOffset = V(); this.done = 0;
    const hooked = tab === "hooked";
    const count = tab === "swim" ? this.sp.count + 4 : hooked ? 1 : this.sp.count;
    const anchor = tab === "swim" ? V(0, -1.2, 0) : hooked ? V(0, -0.35, 0) : V(4.2, -1.2, -2.6);
    this.school = new School(this.sp, anchor, seed, count, tab === "swim" ? 5 : 3, this.proposed);
    this.fish = this.school.positions.map((_, i) => { const f = fishMesh(this.sp.id); this.scene.add(f.mesh); return f; });
    this.lens = this.school.positions.map(() => this.sp.length * (0.85 + 0.4 * this.school.rng.unit()));
    this.phases = this.school.positions.map(() => this.school.rng.unit() * TAU);
    this.bodies = this.school.positions.map((p, i) => new Body(p, this.school.velocities[i], this.lens[i], this.phases[i] / TAU));
    this.line = null; this.tackle = tackle(this.scene, this.proposed && tab !== "swim");
    if (tab !== "swim"){
      this.line = new Line(seed * 7 + 3, FEET);
      if (tab === "lure") this.line.forceNibbles = 3;   // perch: "Nibbles come in threes"
      if (hooked){
        this.line.target = 0; this.lens[0] = this.sp.length; this.bodies[0].len = this.sp.length;
        this.line.float.set(0, SURFACE - FISHING.bite_dip_m, 0); this.line.dip = FISHING.bite_dip_m; this.line.floatBefore.copy(this.line.float);
        this.school.positions[0] = this.line.lure(); this.school.before[0] = this.line.lure();
        this.line.hook(this.school);
        this.bodies[0].pos.copy(this.line.lure()); this.bodies[0].fwd.set(1, 0, 0);
      }
    }
    this.tackle.visible(!!this.line);
    this.choreo = this.proposed && this.line ? new Choreo(this) : null;
    if (this.choreo && hooked){ this.choreo.begin(0); this.choreo.mode = "hooked"; }
    this.strikeAt = -1; this.hookedFor = 0; this.pressNext = false;
  }
  lureNow(){
    if (!this.line) return V();
    const f = this.proposed ? this.line.floatBefore.clone().lerp(this.line.float, this.alpha) : this.line.float.clone();
    return f.addScaledVector(UP, -0.35);
  }
  bot(){
    // The angler: strikes 0.3 s into the bite, reels while the fish rests, lets it run.
    const l = this.line, c = { pressed: false, held: false };
    if (!l) return c;
    if (this.pressNext){ c.pressed = true; this.pressNext = false; }
    if (l.phase === "bite"){ if (this.strikeAt < 0) this.strikeAt = l.clock + 0.3; if (l.clock >= this.strikeAt) c.pressed = true; }
    if (l.phase === "hooked") c.held = l.run === 0 ? l.tension < 0.85 : l.tension < 0.35;
    return c;
  }
  step(dt){
    const l = this.line;
    if (l){
      l.step(dt, this.bot(), this.school);
      if (l.phase === "nibble" || l.phase === "bite" || l.phase === "hooked") this.hookedFor += l.phase === "hooked" ? dt : 0;
      if (this.choreo){
        if (l.events.includes("interest")) this.choreo.begin(l.target);
      }
      if (["caught", "snapped", "tooEarly", "missed"].some(e => l.events.includes(e))) this.done = 1;
    }
    this.school.step(dt, l ? l.splash() : null);
  }
  update(dt){
    this.clock += dt; this.acc += dt;
    let n = 0;
    while (this.acc >= 1 / STEP_HZ && n++ < 8){ this.acc -= 1 / STEP_HZ; this.step(1 / STEP_HZ); }
    this.alpha = clamp(this.acc * STEP_HZ, 0, 1);
    if (this.choreo) this.choreo.update(dt, this.line);
    this.draw(dt);
  }
  draw(dt){
    const s = this.school;
    for (let i = 0; i < s.length; i++){
      const fm = this.fish[i], len = this.lens[i];
      if (!this.proposed){
        // TODAY (fish.rs draw::build): drawn between steps, but turned along the LAST step's velocity, unsmoothed.
        const p = s.drawn(i, this.alpha), v = s.velocities[i];
        const fwd = normOr(v.clone().addScaledVector(UP, -v.dot(UP) * 0.5), anyOrtho(UP));
        const side = normOr(fwd.clone().cross(UP), anyOrtho(UP)), top = side.clone().cross(fwd);
        placeFish(fm, p, fwd, top, len, fishSway(this.clock, this.phases[i]));
      } else {
        const b = this.bodies[i];
        if (!this.choreo || this.choreo.i !== i){
          const p = s.drawn(i, this.alpha), v = s.velBefore[i].clone().lerp(s.velocities[i], this.alpha);
          b.pos.copy(p); b.speed = v.length(); b.shake = 0; b.extraRoll = 0;
          const want = v.clone(); want.y *= 0.5;
          b.turnToward(want, P.turnMax, P.headTau, dt);
          const phase = (s.clock / P.burstPeriod + s.ph[i]) % 1;
          b.beat(dt, phase < P.burstFrac ? "burst" : "glide");
        }
        const { fwd, top } = b.frame();
        placeFish(fm, b.pos, fwd, top, len, b.sway());
      }
    }
    if (this.line){
      const l = this.line;
      let float, lure;
      if (this.proposed){
        float = l.floatBefore.clone().lerp(l.float, this.alpha);
        lure = float.clone().addScaledVector(UP, -0.35).add(this.lureOffset);
        // The float leans toward a lure that is pulled aside.
        float.x += this.lureOffset.x * 0.45; float.z += this.lureOffset.z * 0.45; float.y -= Math.max(0, -this.lureOffset.y) * 0.3;
      } else { float = l.float.clone(); lure = l.lure(); }
      this.tackle.set(float, lure);
    }
    this.env.waves(this.clock);
  }
  phaseLabel(){
    const l = this.line; if (!l) return "";
    if (l.phase === "floating") return l.target >= 0 ? "INTEREST" : (l.splash_s > 0 ? "SPLASH" : "FLOATING");
    if (l.phase === "nibble") return `NIBBLE ${l.nibbleOf - l.nibbles + 1}/${l.nibbleOf}`;
    if (l.phase === "bite") return "BITE";
    if (l.phase === "hooked") return l.run > 0 ? "HOOKED: RUN" : "HOOKED: REEL";
    return l.phase.toUpperCase();
  }
}

// ------------------------------------------------------------------ a tab's view: one renderer, one or two worlds (Split draws both halves of the one canvas)
const state = { mode: "split", speed: 1, night: false, species: "perch", follow: true, loop: true };
function makeView(tab, cam0){
  const canvas = $("cv-" + tab), hud = $("hud-" + tab);
  let renderer = null, raf = 0, worlds = null, last = 0, seed = 11, restartAt = -1;
  const cam = new THREE.PerspectiveCamera(48, 16 / 10, 0.02, 120);
  let { yaw, pitch, dist } = cam0; const target = cam0.target.clone(), targets = {};
  let drag = null; const touches = new Map(); let pinch0 = 0, dist0 = 0;
  canvas.addEventListener("pointerdown", e => { touches.set(e.pointerId, [e.clientX, e.clientY]); canvas.setPointerCapture(e.pointerId);
    if (touches.size === 1) drag = [e.clientX, e.clientY, yaw, pitch]; else { const [a, b] = [...touches.values()]; pinch0 = Math.hypot(a[0] - b[0], a[1] - b[1]); dist0 = dist; drag = null; } });
  canvas.addEventListener("pointermove", e => { if (!touches.has(e.pointerId)) return; touches.set(e.pointerId, [e.clientX, e.clientY]);
    if (touches.size >= 2){ const [a, b] = [...touches.values()]; const d = Math.hypot(a[0] - b[0], a[1] - b[1]); if (pinch0 > 0) dist = clamp(dist0 * pinch0 / d, 0.5, 30); return; }
    if (!drag) return; yaw = drag[2] - (e.clientX - drag[0]) * 0.008; pitch = clamp(drag[3] + (e.clientY - drag[1]) * 0.006, -1.4, 1.4); });
  const up = e => { touches.delete(e.pointerId); drag = null; if (touches.size === 1){ const [p] = [...touches.values()]; drag = [p[0], p[1], yaw, pitch]; } };
  canvas.addEventListener("pointerup", up); canvas.addEventListener("pointercancel", up);
  canvas.addEventListener("wheel", e => { e.preventDefault(); dist = clamp(dist * (1 + Math.sign(e.deltaY) * 0.1), 0.5, 30); }, { passive: false });
  function build(){
    worlds = { today: new World("today", tab, state.species, seed), proposed: new World("proposed", tab, state.species, seed) };
    worlds.today.env.night(state.night); worlds.proposed.env.night(state.night); restartAt = -1;
  }
  function frame(now){
    raf = requestAnimationFrame(frame);
    const dtReal = Math.min(0.05, last ? (now - last) / 1000 : 0); last = now;
    const dt = dtReal * state.speed;
    const r = canvas.getBoundingClientRect(); if (!r.width) return;
    const pr = renderer.getPixelRatio();
    if (canvas.width !== Math.round(r.width * pr) || canvas.height !== Math.round(r.height * pr)) renderer.setSize(r.width, r.height, false);
    if (dt > 0){ worlds.today.update(dt); worlds.proposed.update(dt); }
    // End of a run: the lure tab loops after the first run of the take, the fight tab after the catch.
    const w = worlds.proposed, l = w.line;
    if (l && restartAt < 0 && ((tab === "lure" && w.hookedFor > 2.2) || w.done || worlds.today.done || (tab === "lure" && w.clock > 40))) restartAt = w.clock + 2.0;
    if (restartAt >= 0 && w.clock > restartAt && state.loop){ seed++; build(); }
    // Follow: each side's camera keeps its own fish in view (the float on the line, the school's middle).
    const aim = k => { const ww = worlds[k]; if (tab === "hooked"){ const L = ww.lureNow(); return V(L.x, -0.5, L.z); } const c = ww.school.centre(); return V(c.x, c.y, c.z); };
    for (const k of ["today", "proposed"]){
      if (!targets[k]) targets[k] = target.clone();
      if (tab !== "lure" && state.follow) targets[k].lerp(aim(k), Math.min(1, dtReal * 1.5)); else targets[k].copy(target);
    }
    const place = k => { const t = targets[k]; cam.position.set(t.x + Math.sin(yaw) * Math.cos(pitch) * dist, t.y + Math.sin(pitch) * dist, t.z + Math.cos(yaw) * Math.cos(pitch) * dist); cam.lookAt(t); };
    const W = r.width, H = r.height;
    renderer.setScissorTest(state.mode === "split");
    if (state.mode === "split"){
      cam.aspect = (W / 2) / H; cam.updateProjectionMatrix();
      renderer.setViewport(0, 0, W / 2, H); renderer.setScissor(0, 0, W / 2, H); place("today"); renderer.render(worlds.today.scene, cam);
      renderer.setViewport(W / 2, 0, W / 2, H); renderer.setScissor(W / 2, 0, W / 2, H); place("proposed"); renderer.render(worlds.proposed.scene, cam);
    } else {
      cam.aspect = W / H; cam.updateProjectionMatrix(); renderer.setViewport(0, 0, W, H); place(state.mode);
      renderer.render(worlds[state.mode].scene, cam);
    }
    if (hud){
      const a = worlds.today.phaseLabel(), b = worlds.proposed.phaseLabel();
      hud.querySelector(".l").textContent = state.mode === "proposed" ? "" : "TODAY" + (a ? " · " + a : "");
      hud.querySelector(".r").textContent = state.mode === "today" ? "" : "PROPOSED" + (b ? " · " + b : "");
      hud.classList.toggle("one", state.mode !== "split");
    }
  }
  return {
    show(){
      if (!renderer){
        loadTextures();
        renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
        renderer.outputEncoding = THREE.sRGBEncoding; renderer.setPixelRatio(Math.min(2, devicePixelRatio));
        build();
      }
      last = 0; if (!raf) raf = requestAnimationFrame(frame);
    },
    hide(){ cancelAnimationFrame(raf); raf = 0; },
    replay(){ if (renderer) build(); },
    strike(){ if (worlds){ worlds.today.pressNext = true; worlds.proposed.pressNext = true; } },
    night(){ if (worlds){ worlds.today.env.night(state.night); worlds.proposed.env.night(state.night); } },
    rebuild(){ if (renderer) build(); },
    _w: () => worlds,
  };
}
const VIEWS = {
  lure:   makeView("lure",   { yaw: 1.05, pitch: -0.1, dist: 1.9, target: V(0.15, -0.42, 0) }),
  swim:   makeView("swim",   { yaw: 0.6, pitch: -0.06, dist: 4.2, target: V(0, -1.3, 0) }),
  hooked: makeView("hooked", { yaw: 1.4, pitch: -0.05, dist: 2.6, target: V(0, -0.5, 0) }),
};

// ------------------------------------------------------------------ the page: tabs (one 3D view at a time, #anchor), shared controls, sliders, modals
const TABS = ["lure", "swim", "hooked"];
let current = null;
function setTab(t, push){
  if (!TABS.includes(t)) t = "lure";
  if (current === t) return;
  if (current) VIEWS[current].hide();
  current = t;
  TABS.forEach(x => { $("tab-" + x).hidden = x !== t; $("tb-" + x).setAttribute("aria-pressed", x === t); });
  document.querySelectorAll(".lureonly").forEach(e => e.hidden = t !== "lure");
  document.querySelectorAll(".hookonly").forEach(e => e.hidden = t === "lure");
  // Each tab's canvas sits in the one shared stage.
  VIEWS[t].show();
  if (push !== false && location.hash !== "#" + t) history.replaceState(null, "", "#" + t);
}
TABS.forEach(t => $("tb-" + t).onclick = () => setTab(t));
addEventListener("hashchange", () => setTab(location.hash.slice(1), false));
function seg(id, key, values, after){
  const host = $(id);
  host.querySelectorAll("button").forEach((b, i) => b.onclick = () => {
    state[key] = values[i]; host.querySelectorAll("button").forEach((x, j) => x.setAttribute("aria-pressed", j === i)); if (after) after();
  });
}
seg("sg-mode", "mode", ["today", "proposed", "split"]);
seg("sg-speed", "speed", [1, 0.5, 0.25]);
seg("sg-time", "night", [false, true], () => Object.values(VIEWS).forEach(v => v.night()));
seg("sg-species", "species", ["perch", "minnow"], () => Object.values(VIEWS).forEach(v => v.rebuild()));
$("bt-replay").onclick = () => VIEWS[current].replay();
$("bt-strike").onclick = () => VIEWS[current].strike();
$("ck-follow").onchange = e => state.follow = e.target.checked;
$("ck-loop").onchange = e => state.loop = e.target.checked;

// Sliders, grouped and collapsible.
(function sliders(){
  const host = $("tuning");
  for (const [name, list] of GROUPS){
    const d = document.createElement("details"); d.className = "grp";
    const s = document.createElement("summary"); s.textContent = name; d.appendChild(s);
    const box = document.createElement("div"); box.className = "sliders"; d.appendChild(box);
    for (const [key, label, min, max, step, def, unit] of list){
      const row = document.createElement("div"); row.className = "sl";
      const id = "p-" + key;
      row.innerHTML = `<label for="${id}">${label}</label><input id="${id}" type="range" min="${min}" max="${max}" step="${step}" value="${def}"><output for="${id}"></output>`;
      const input = row.querySelector("input"), out = row.querySelector("output");
      const show = () => { const v = +input.value; out.textContent = (Math.abs(v) >= 100 ? v.toFixed(0) : +v.toFixed(3)) + (unit ? " " + unit : ""); };
      input.oninput = () => { P[key] = +input.value; show(); };
      show(); box.appendChild(row);
    }
    host.appendChild(d);
  }
  $("bt-reset").onclick = () => { for (const [, list] of GROUPS) for (const s of list){ const i = $("p-" + s[0]); i.value = s[5]; P[s[0]] = s[5]; i.oninput(); } };
})();

// Modals: the page's notes and each tab's, behind the (i) buttons.
(function modals(){
  const modal = $("modal"), body = $("mbody"), title = $("mtitle");
  const close = () => { modal.hidden = true; };
  $("mclose").onclick = close; modal.addEventListener("click", e => { if (e.target === modal) close(); });
  addEventListener("keydown", e => { if (e.key === "Escape" && !modal.hidden) close(); });
  document.querySelectorAll("[data-info]").forEach(b => b.onclick = () => {
    const src = $(b.dataset.info); title.textContent = src.dataset.title; body.innerHTML = src.innerHTML; modal.hidden = false; $("mclose").focus();
  });
})();

setTab(location.hash.slice(1) || "lure", false);
window.__fishBehaviour = { VIEWS, state, P };
})();
