// River Craft (M34): the Rowing tab (river-craft design 9; the owner, 2026-11-19: "mockup better rowing on a raft").
// The raft on a slow, wide reach, rowed the game's way today (the pole leaning on the held key, the push 1.35 m out
// on alternate sides) or the proposed way: two oars on a rowing frame, each stroke in five stages (reach, catch,
// drive, release, recovery) on one clock for both oars, the push the blade's own drag through the water, the water
// answering with a splash, wash, drips and the oar's swirls. No pole is proposed (the owner, 2026-11-19: "no pole just
// oar is best"). First person with the approved hand of M36 Tools in Hand (hand.js), or third person.
// RC.Oarsman (the raft, its frame, oars and rower, and the stroke) is shared with the Raft tab.
// Needs engine.js, physics.js, hand.js.
(function () {
"use strict";
const RC = window.RC, clamp = RC.clamp, HAND = RC.HAND;
const D = Math.PI / 180, RHO = 1000, G = 9.81;
const OUT = [0, 0], PV = [0, 0];
const lerp = (a, b, t) => a + (b - a) * t;
const c01 = u => clamp(u, 0, 1);
const smooth = u => { u = c01(u); return u * u * (3 - 2 * u); };
const halfCos = u => (1 - Math.cos(Math.PI * c01(u))) / 2;
const easeOut = u => { u = c01(u); return 1 - (1 - u) ** 3; };
const easeIn = u => { u = c01(u); return u * u * u; };
const V3 = (x, y, z) => new THREE.Vector3(x, y, z);

// ---------------- the tunables (design 9.7): one source, the sliders edit it, the RON is printed from it ----------
// Version 4: the seat 0.26 m nearer the pins and the sweep 38 degrees, so the arms reach the handles through the
// whole stroke (straight at the release, the handles at the chest at the catch); a forearm length of its own.
const DEF = {
  oars: {pin: [0.0, 0.62, 1.0], length_m: 3.0, inboard_m: 0.95, blade_m2: 0.14, blade_cd: 1.3, catch_deg: 22, release_deg: -16,
    coil_deg: 3, follow_deg: 3, blade_depth_m: 0.12, blade_clear_m: 0.22, feather_deg: 80, turn_share: 0.35, bed_clear_m: 0.1},
  stroke: {reach_s: 0.16, catch_s: 0.12, drive_s: 0.62, release_s: 0.14, recovery_s: 0.66, exaggerate: 1.0},
  rower: {seat: [-0.52, 0.62, 0.0], shoulder_m: 0.5, shoulder_half_m: 0.27, upper_arm_m: 0.3, forearm_m: 0.37, lean_catch_deg: -6, lean_release_deg: 22,
    body_lead: 0.25, head_follow: 0.6, hand_scale_m: 0.03},
  // the raft's own lantern (the owner, 2026-11-19: "Rafts should also have a light that comes with it at night"):
  // fitted to the bow post, lit from dusk to dawn, a lamp in the light field's lamp term; the page lights it with
  // its own lamp falloff (engine.js lampFall, range 15 m) and colour (LAMP_RGB), so these print for the RON only
  lamp: {at: [1.95, 1.45, -0.95], range_m: 15.0, colour: [1.0, 0.62, 0.3], intensity: 2.4},
  water: {splash_drops: 16, splash_speed_mps: 2.4, drips: 5, puddle_m: 0.24, puddle_s: 4.0, puddle_spin_rps: 0.6},
};
const T = JSON.parse(JSON.stringify(DEF));
const GROUPS = ["oars", "stroke", "rower", "lamp", "water"];
// The sliders: [group, key, min, max, step]
const SLIDERS = [
  ["stroke", "reach_s", 0.05, 0.5, 0.01], ["stroke", "catch_s", 0.04, 0.4, 0.01], ["stroke", "drive_s", 0.3, 1.4, 0.02],
  ["stroke", "release_s", 0.05, 0.4, 0.01], ["stroke", "recovery_s", 0.3, 1.4, 0.02], ["stroke", "exaggerate", 0, 2, 0.05],
  ["oars", "catch_deg", 10, 34, 1], ["oars", "release_deg", -28, 0, 1], ["oars", "coil_deg", 0, 8, 0.5], ["oars", "follow_deg", 0, 8, 0.5],
  ["oars", "blade_depth_m", 0, 0.3, 0.01], ["oars", "blade_clear_m", 0.05, 0.45, 0.01], ["oars", "feather_deg", 0, 90, 1],
  ["oars", "blade_m2", 0.05, 0.3, 0.01], ["oars", "turn_share", 0, 1, 0.05],
  ["rower", "lean_catch_deg", -20, 10, 1], ["rower", "lean_release_deg", 0, 30, 1], ["rower", "body_lead", 0, 0.6, 0.05],
  ["rower", "head_follow", 0, 1, 0.05], ["rower", "hand_scale_m", 0.016, 0.034, 0.001],
  ["water", "splash_drops", 0, 40, 1], ["water", "splash_speed_mps", 0.5, 4, 0.1], ["water", "drips", 0, 12, 1],
  ["water", "puddle_m", 0.1, 0.8, 0.02], ["water", "puddle_s", 1, 8, 0.25], ["water", "puddle_spin_rps", 0, 2, 0.05],
];
const fmt = v => Array.isArray(v) ? "(" + v.map(fmt).join(", ") + ")" : Number.isInteger(v) && Math.abs(v) < 1000 ? v.toFixed(1) : String(+v.toFixed(4));
function ron() {
  const grp = g => g + ": (\n" + Object.entries(T[g]).map(([k, v]) => "    " + k + ": " + fmt(v) + ",").join("\n") + "\n),";
  return GROUPS.map(grp).join("\n");
}

// ---------------- the stroke schedule: one function, read by the force and by the drawing ----------------
// Returns the oar's pose at time t into a stroke: the sweep theta (rad, + toward the bow), dip (0 clear .. 1 the
// blade buried), feather (0 square .. 1 flat), body (0 the catch's lean .. 1 the release's), and the stage.
const STAGES = ["Reach", "Catch", "Drive", "Release", "Recovery"];
function period() { const s = T.stroke; return s.reach_s + s.catch_s + s.drive_s + s.release_s + s.recovery_s; }
function oarPose(t, dir, range, o) {
  const s = T.stroke, a = T.oars, E = s.exaggerate;
  const tc = a.catch_deg * D, tr = a.release_deg * D, mid = (tc + tr) / 2;
  const C = mid + (tc - mid) * range, Rl = mid + (tr - mid) * range;
  const coil = a.coil_deg * D * E, foll = a.follow_deg * D * E, lead = clamp(T.rower.body_lead, 0, 0.9);
  let k, th, dip, fe, body, st;
  const e1 = s.reach_s, e2 = e1 + s.catch_s, e3 = e2 + s.drive_s, e4 = e3 + s.release_s;
  // The handle's path is one loop: the catch's lift and the release's press each overlap the sweep a little (the
  // blade starts moving aft as it buries and keeps moving as it lifts), so the hands go round a rounded D, never a
  // square corner.
  if (t < e1) { k = t / s.reach_s; st = 0; th = C + coil * easeOut(k); dip = 0; fe = 1 - smooth(k); body = -0.08 * E * easeOut(k); }
  else if (t < e2) { k = (t - e1) / s.catch_s; st = 1; th = C + coil * (1 - smooth(k)) - (C - Rl) * 0.04 * k * k; dip = easeIn(k) * 0.75 + 0.25 * k; fe = 0; body = -0.08 * E * (1 - k); }
  else if (t < e3) { k = (t - e2) / s.drive_s; st = 2; th = lerp(C - (C - Rl) * 0.04, Rl, halfCos(k)); dip = 1; fe = 0; body = halfCos(k / (1 - lead)); }
  else if (t < e4) { k = (t - e3) / s.release_s; st = 3; th = Rl - foll * easeOut(k); dip = (1 - k) * (1 - k); fe = easeOut(k); body = 1; }
  else { k = (t - e4) / s.recovery_s; st = 4; th = lerp(Rl - foll, C, halfCos(k)); dip = 0; fe = 1; body = 1 - halfCos(k); }
  if (dir < 0) { th = C + Rl - th; body = 1 - body; }
  o.theta = th; o.dip = dip; o.feather = fe; o.body = body; o.stage = st; o.k = k;
  return o;
}
// The oar in the raft's frame from a pose: the pin, the loom's direction, the handle end, the grip and the blade.
// floorY: the bed under the blade, in the raft's frame (the blade comes to rest on it and pushes nothing off it).
function oarGeom(side, p, g, floorY) {
  const a = T.oars, rb = a.length_m - a.inboard_m - 0.3;
  let yb = lerp(a.blade_clear_m, -a.blade_depth_m, p.dip);
  g.onBed = floorY !== undefined && yb < floorY;
  if (g.onBed) yb = floorY;
  const phi = Math.asin(clamp((a.pin[1] - yb) / rb, -0.95, 0.95));
  const ct = Math.cos(p.theta), stt = Math.sin(p.theta), cp = Math.cos(phi), sp = Math.sin(phi);
  g.pin = V3(a.pin[0], a.pin[1], side * a.pin[2]);
  g.dir = V3(stt * cp, -sp, side * ct * cp);                     // outboard along the loom
  g.handle = g.pin.clone().addScaledVector(g.dir, -a.inboard_m);
  g.grip = g.pin.clone().addScaledVector(g.dir, -(a.inboard_m - 0.12));
  g.blade = g.pin.clone().addScaledVector(g.dir, rb);
  g.yb = yb; g.phi = phi;
  g.feather = p.feather * a.feather_deg * D;
  return g;
}

// ---------------- the arm: two bones, the elbow out and down ----------------
function elbowOf(S, Hd, la, lb, pole) {
  const d = Hd.clone().sub(S), L = clamp(d.length(), 1e-4, la + lb - 1e-4); d.normalize();
  const x = (la * la - lb * lb + L * L) / (2 * L), h = Math.sqrt(Math.max(0, la * la - x * x));
  const pv = pole.clone().addScaledVector(d, -pole.dot(d)); if (pv.lengthSq() < 1e-8) pv.set(0, -1, 0); pv.normalize();
  return S.clone().addScaledVector(d, x).addScaledVector(pv, h);
}

// ---------------- the hand: M36's approved hand, lit as the terrain is ----------------
function handKit(U) {
  const tool = HAND.TOOL, v3 = HAND.v3;
  const geo = HAND.proposedHandGeometry(tool, 0, undefined);
  const q = new THREE.Quaternion(...tool.rotation).normalize();
  const fistT = v3(tool.fist).sub(v3(tool.butt)).applyQuaternion(q.clone().invert()).multiplyScalar(1 / tool.scale);
  const pc = i => v3(tool.hand[i].at);
  const h = pc(10).sub(pc(1)).normalize();
  // face normals and the shade baked at build (the AO and the fingertips' flush) as attributes
  const n = geo.userData.N, K = geo.userData.k, FL = geo.userData.flush, look = HAND.HAND_LOOK, cnt = geo.getAttribute("position").count;
  const col = new Float32Array(cnt * 3);
  for (let i = 0; i < cnt; i++) {
    const t = FL && FL[i] ? look.tip_to.map((c, j) => 1 + FL[i] * (c / look.skin[j] - 1)) : [1, 1, 1];
    col[i * 3] = K[i] * t[0]; col[i * 3 + 1] = K[i] * t[1]; col[i * 3 + 2] = K[i] * t[2];
  }
  geo.setAttribute("normal", new THREE.Float32BufferAttribute(n, 3));
  geo.setAttribute("aCol", new THREE.BufferAttribute(col, 3));
  // the arm's way from the grip: the sleeve's vertices (group 2, cloth), square to the handle; its far end the elbow
  const P = geo.getAttribute("position"), grp = geo.groups[HAND.HAND_PARTS.cloth], arm = new THREE.Vector3(), tmp = new THREE.Vector3();
  for (let i = grp.start; i < grp.start + grp.count; i++) arm.add(tmp.fromBufferAttribute(P, i).sub(fistT));
  arm.addScaledVector(h, -arm.dot(h)).normalize();
  let reach = 0; for (let i = grp.start; i < grp.start + grp.count; i++) reach = Math.max(reach, tmp.fromBufferAttribute(P, i).sub(fistT).dot(arm));
  const c = h.clone().cross(arm);
  const toolBasis = new THREE.Matrix4().makeBasis(h, arm, c);
  const tex = Object.assign(HAND.handTextures(), {palm: HAND.palmTexture(), fold: HAND.foldTexture()});
  const mats = ["skin", "nail", "cloth", "cuff", "palm", "fold"].map(k => {
    const t = new THREE.CanvasTexture(tex[k]); t.magFilter = t.minFilter = THREE.NearestFilter; t.generateMipmaps = false; t.wrapS = t.wrapT = THREE.RepeatWrapping;
    return new THREE.ShaderMaterial({
      uniforms: Object.assign({uMap: {value: t}}, U), side: THREE.DoubleSide,
      vertexShader: `attribute vec3 aCol; varying vec2 vUv; varying vec3 vC; varying vec3 vN; varying vec3 vP;
        void main() { vUv = uv; vC = aCol; vec4 w = modelMatrix*vec4(position, 1.0); vP = w.xyz; vN = normalize(mat3(modelMatrix)*normal);
          gl_Position = projectionMatrix*viewMatrix*w; }`,
      fragmentShader: `uniform sampler2D uMap; varying vec2 vUv; varying vec3 vC; varying vec3 vN; varying vec3 vP;
        ${RC.LIGHT_GLSL}
        void main() {
          vec3 alb = fromSrgb(texture2D(uMap, vUv).rgb)*vC;
          vec3 n = normalize(vN);
          vec3 L = uAmb*(0.55 + 0.45*n.y) + uSunCol*max(dot(n, uSunDir), 0.0) + lampLight(vP, n);
          gl_FragColor = vec4(toSrgb(fogged(alb*L, vP)), 1.0);
        }`,
    });
  });
  const mirror = new THREE.Matrix4().makeScale(1, 1, -1);
  const KEEP = new THREE.Vector3(-1, -0.35, 0).normalize(), ELBOW_SHARE = 0.35;
  const kit = {reach, fistT};
  // A hand on a handle: grip point G, handle axis Hw (the index-to-little-finger way), elbow E; side -1 is the left
  // hand, built as the right hand on the mirrored handle and mirrored back.
  kit.make = () => { const m = new THREE.Mesh(geo, mats); m.matrixAutoUpdate = false; m.frustumCulled = false; return m; };
  const R = new THREE.Matrix4(), M = new THREE.Matrix4(), S = new THREE.Matrix4(), Tm = new THREE.Matrix4();
  kit.place = (mesh, side, G, Hw, E) => {
    const g = G.clone(), hw = Hw.clone(), e = E.clone();
    if (side < 0) { g.z = -g.z; hw.z = -hw.z; e.z = -e.z; }
    hw.normalize();
    // the forearm's way: toward the elbow, held toward back-and-down so the knuckles stay up through the stroke
    // (a wrist that flexes, as a rower's does); `elbow_share` of it toward the solved elbow
    const a = e.sub(g); a.addScaledVector(hw, -a.dot(hw)); if (a.lengthSq() < 1e-8) a.set(0, -1, 0); a.normalize();
    const keep = KEEP.clone().addScaledVector(hw, -KEEP.dot(hw)).normalize();
    a.multiplyScalar(ELBOW_SHARE).addScaledVector(keep, 1 - ELBOW_SHARE).normalize();
    const cc = hw.clone().cross(a);
    R.makeBasis(hw, a, cc).multiply(M.copy(toolBasis).transpose());
    const sc = T.rower.hand_scale_m;
    mesh.matrix.makeTranslation(g.x, g.y, g.z).multiply(R).multiply(S.makeScale(sc, sc, sc)).multiply(Tm.makeTranslation(-fistT.x, -fistT.y, -fistT.z));
    if (side < 0) mesh.matrix.premultiply(mirror);
    mesh.matrixWorldNeedsUpdate = true;
  };
  return kit;
}

// ---------------- the water's answer: drops (points) and the oar's swirls ----------------
function Drops(U, n) {
  const pos = new Float32Array(n * 3), alpha = new Float32Array(n), size = new Float32Array(n);
  const g = new THREE.BufferGeometry();
  g.setAttribute("position", new THREE.BufferAttribute(pos, 3)); g.setAttribute("aA", new THREE.BufferAttribute(alpha, 1)); g.setAttribute("aS", new THREE.BufferAttribute(size, 1));
  const mat = new THREE.ShaderMaterial({uniforms: Object.assign({uPx: {value: 1}}, U), transparent: true, depthWrite: false,
    vertexShader: `uniform float uPx; attribute float aA; attribute float aS; varying float vA; varying vec3 vC;
      ${RC.LIGHT_GLSL}
      void main() { vec4 w = modelMatrix*vec4(position, 1.0); vec4 mv = viewMatrix*w; gl_Position = projectionMatrix*mv;
        gl_PointSize = max(1.5, aS*uPx*420.0/max(0.3, -mv.z)); vA = aA;
        vec3 L = uAmb*1.1 + uSunCol*max(uSunDir.y, 0.0)*0.9 + lampLight(w.xyz, normalize(cameraPosition - w.xyz))*1.3;
        vC = toSrgb(fogged(vec3(0.80, 0.90, 0.94)*L, w.xyz)); }`,
    fragmentShader: `varying float vA; varying vec3 vC; void main() { if (vA <= 0.01) discard; gl_FragColor = vec4(vC, vA); }`});
  const pts = new THREE.Points(g, mat); pts.frustumCulled = false; pts.renderOrder = 4;
  const P = [];
  for (let i = 0; i < n; i++) P.push({x: 0, y: -99, z: 0, vx: 0, vy: 0, vz: 0, life: 0, max: 1, s: 0.02, float: 0});
  let next = 0;
  return {
    mesh: pts, mat,
    spawn(x, y, z, vx, vy, vz, life, s, float) { const p = P[next]; next = (next + 1) % n; Object.assign(p, {x, y, z, vx, vy, vz, life, max: life, s, float: float || 0}); },
    clear() { for (const p of P) p.life = 0; },
    step(h, R) {
      for (const p of P) {
        if (p.life <= 0) continue;
        p.life -= h;
        if (p.float) { R.flow(p.x, p.z, OUT); p.x += OUT[0] * h; p.z += OUT[1] * h; p.y = R.surf(p.x, p.z) + 0.02; continue; }
        p.vy -= G * h; p.x += p.vx * h; p.y += p.vy * h; p.z += p.vz * h;
        const s = R.surf(p.x, p.z); if (p.y < s) { p.life = Math.min(p.life, 0); }
      }
    },
    sync() {
      for (let i = 0; i < n; i++) {
        const p = P[i], on = p.life > 0;
        pos[i * 3] = p.x; pos[i * 3 + 1] = on ? p.y : -99; pos[i * 3 + 2] = p.z;
        alpha[i] = on ? (p.float ? 0.75 * c01(p.life / p.max * 2) : 0.9 * c01(p.life / p.max * 3)) : 0; size[i] = p.s;
      }
      g.attributes.position.needsUpdate = true; g.attributes.aA.needsUpdate = true; g.attributes.aS.needsUpdate = true;
    },
  };
}
function Swirls(U, n, scene) {
  const geo = new THREE.PlaneGeometry(2, 2); geo.rotateX(-Math.PI / 2);
  const list = [];
  for (let i = 0; i < n; i++) {
    const mat = new THREE.ShaderMaterial({transparent: true, depthWrite: false,
      uniforms: Object.assign({uFade: {value: 0}, uSpin: {value: 0}, uSign: {value: 1}}, U),
      vertexShader: `varying vec2 vQ; varying vec3 vP; void main() { vQ = position.xz; vec4 w = modelMatrix*vec4(position, 1.0); vP = w.xyz; gl_Position = projectionMatrix*viewMatrix*w; }`,
      fragmentShader: `uniform float uFade; uniform float uSpin; uniform float uSign; varying vec2 vQ; varying vec3 vP;
        ${RC.LIGHT_GLSL}
        void main() {
          vec2 q = (floor(vQ*9.0) + 0.5)/9.0;              // pixel steps, as the game's textures
          float r = length(q); if (r > 1.0) discard;
          float a = atan(q.y, q.x)*uSign + uSpin;
          float arm = 0.5 + 0.5*sin(2.0*a + r*7.0);          // two spiral arms of foam
          float ring = smoothstep(0.30, 0.55, r)*(1.0 - smoothstep(0.75, 1.0, r));
          vec3 L = uAmb + uSunCol*max(uSunDir.y, 0.0)*0.8 + lampLight(vP, vec3(0.0, 1.0, 0.0));
          float foam = ring*step(0.45, arm);
          float dim = (1.0 - smoothstep(0.0, 0.45, r));       // the dimple: the swirl's darker eye
          vec3 c = mix(vec3(0.01, 0.04, 0.05)*L, vec3(0.86, 0.93, 0.96)*L*0.9, foam);
          float al = (foam*0.45 + dim*0.35)*uFade;
          if (al < 0.02) discard;
          gl_FragColor = vec4(toSrgb(fogged(c, vP)), al);
        }`});
    const m = new THREE.Mesh(geo, mat); m.visible = false; m.renderOrder = 3; m.frustumCulled = false; scene.add(m);
    list.push({m, x: 0, z: 0, age: 0, life: 1, r: 0.3, spin: 0, sign: 1, rate: 1});
  }
  let next = 0;
  return {
    spawn(x, z, r, sign, life, rate) { const s = list[next]; next = (next + 1) % n; Object.assign(s, {x, z, r, sign, age: 0, life, spin: Math.random() * 6, rate}); s.m.visible = true; },
    clear() { for (const s of list) { s.age = s.life; s.m.visible = false; } },
    step(h, R) {
      for (const s of list) {
        if (s.age >= s.life) continue;
        s.age += h; R.flow(s.x, s.z, OUT); s.x += OUT[0] * h; s.z += OUT[1] * h;
        s.spin += s.rate * Math.PI * 2 * h * (1 - s.age / s.life);
      }
    },
    sync(R) {
      for (const s of list) {
        if (s.age >= s.life) { s.m.visible = false; continue; }
        const u = s.age / s.life, grow = 0.7 + 0.5 * easeOut(u * 2);
        s.m.position.set(s.x, R.surf(s.x, s.z) + 0.035, s.z); s.m.scale.setScalar(s.r * grow);
        s.m.material.uniforms.uFade.value = Math.pow(1 - u, 1.4) * smooth(u * 12); s.m.material.uniforms.uSpin.value = s.spin; s.m.material.uniforms.uSign.value = s.sign;
      }
    },
  };
}

// ---------------- the raft, its rowing frame, the oars, the rower and today's poler ----------------
const mat = (U, tint) => RC.litMaterial(U, {tint});
const mk = (geo, m, x, y, z) => { const o = new THREE.Mesh(geo, m); o.position.set(x || 0, y || 0, z || 0); return o; };
// a block laid from A to B (a limb, a pole)
const UNIT = RC.box(1, 1, 1, "canvas");
function limb(o, A, B, w) {
  const d = B.clone().sub(A), L = d.length();
  o.position.copy(A).addScaledVector(d, 0.5);
  o.quaternion.setFromUnitVectors(new THREE.Vector3(0, 1, 0), d.multiplyScalar(1 / Math.max(L, 1e-6)));
  o.scale.set(w, L, w);
}
// The raft's own parts, in its frame (+x the bow, +z the right, y up from the waterline). One list, read by the
// drawing and by the clearance check (`RC.Oarsman.clear`), so the two cannot disagree.
const LOG_TOP = 0.21, LOG_EDGE = 1.21;
function deckBoxes() {
  const a = DEF.oars, s = DEF.rower.seat, top = a.pin[1] - 0.08;
  return [
    {name: "plank aft", c: [-1.5, 0.24, 0], h: [0.15, 0.04, 1.25]}, {name: "plank mid", c: [0, 0.24, 0], h: [0.15, 0.04, 1.25]}, {name: "plank fore", c: [1.5, 0.24, 0], h: [0.15, 0.04, 1.25]},
    {name: "crate", c: [1.2, 0.53, 0.5], h: [0.35, 0.25, 0.3]}, {name: "lantern post", c: [1.95, 0.8, -0.95], h: [0.05, 0.55, 0.05]},
    {name: "beam", c: [a.pin[0], 0.36, 0], h: [0.08, 0.06, 1.3]},
    {name: "post left", c: [a.pin[0], (0.42 + top) / 2, -a.pin[2]], h: [0.07, (top - 0.42) / 2, 0.07]},
    {name: "post right", c: [a.pin[0], (0.42 + top) / 2, a.pin[2]], h: [0.07, (top - 0.42) / 2, 0.07]},
    {name: "bench", c: [s[0], 0.43, 0], h: [0.18, 0.15, 0.45]},
  ];
}
function raftKit(U, withToday) {
  const g = new THREE.Group(), m = mat(U), lg = RC.logGeo(4.4, 0.25);
  for (let i = 0; i < 5; i++) g.add(mk(lg, m, 0, -0.04, -0.96 + i * 0.48));
  const tiles = {"plank aft": "planks", "plank mid": "planks", "plank fore": "planks", crate: "crate", "lantern post": "darkPlanks", beam: "darkPlanks", "post left": "darkPlanks", "post right": "darkPlanks", bench: "planks"};
  const frame = new THREE.Group(); g.add(frame);
  for (const b of deckBoxes()) {
    const o = mk(RC.box(b.h[0] * 2, b.h[1] * 2, b.h[2] * 2, tiles[b.name]), m, ...b.c);
    (["beam", "post left", "post right", "bench"].includes(b.name) ? frame : g).add(o);
  }
  const lan = RC.lantern(U, 0.9); lan.position.set(1.95, 1.45, -0.95); g.add(lan);
  // the oars: a loom and a blade on a pivot at the pin; a swivel oarlock (two iron horns either side of the loom,
  // turning with its sweep) on each post, so the loom rides in the lock above the post, never through it
  const iron = mat(U, 0x9a9a9a);
  const oars = [-1, 1].map(s => {
    const a = DEF.oars;
    const lock = new THREE.Group(); lock.position.set(a.pin[0], a.pin[1], s * a.pin[2]); frame.add(lock);
    lock.add(mk(RC.box(0.04, 0.05, 0.04, "iron"), iron, 0, -0.06, 0));
    lock.add(mk(RC.box(0.03, 0.12, 0.022, "iron"), iron, 0, 0.0, 0.048));
    lock.add(mk(RC.box(0.03, 0.12, 0.022, "iron"), iron, 0, 0.0, -0.048));
    lock.add(mk(RC.box(0.03, 0.02, 0.118, "iron"), iron, 0, -0.04, 0));
    const p = new THREE.Group(); frame.add(p);
    const loom = mk(RC.box(a.length_m - 0.55 - a.inboard_m, 0.06, 0.06, "planks"), m, 0, 0, 0); p.add(loom);
    const handle = mk(RC.box(a.inboard_m, 0.034, 0.034, "planks"), m, 0, 0, 0); p.add(handle);
    const bladeG = new THREE.Group(); p.add(bladeG);
    bladeG.add(mk(RC.box(0.62, 0.2, 0.035, "planks"), m, 0, 0, 0));
    const grip = mk(RC.box(0.02, 0.044, 0.044, "darkPlanks"), m, 0, 0, 0); p.add(grip);
    return {p, lock, loom, handle, bladeG, grip, side: s};
  });
  // the seated rower: hips on the bench, feet braced on the frame's beam, the body a pivot at the hip. He faces the
  // bow (the push stroke): his cap's peak and his nose point the way the raft goes, the back of his head is hair.
  const rower = new THREE.Group(); g.add(rower);
  const cloth = mat(U, 0x8a5a3a), skin = mat(U, 0xd9a77c), dark = mat(U, 0x4a5368), hat = mat(U, 0x6b5638), hair = mat(U, 0x5a4030);
  const legs = new THREE.Group(); rower.add(legs);
  const thighs = [], shins = [], feet = [];
  for (const s of [-1, 1]) { thighs.push(mk(UNIT, dark)); shins.push(mk(UNIT, dark)); feet.push(mk(RC.box(0.2, 0.08, 0.12, "canvas"), mat(U, 0x3a3026))); }
  legs.add(...thighs, ...shins, ...feet);
  const hip = new THREE.Group(); rower.add(hip);
  const torso = mk(RC.box(0.26, 0.58, 0.42, "canvas"), cloth, 0, 0.32, 0); hip.add(torso);
  const head = new THREE.Group(); head.position.set(0, 0.78, 0); hip.add(head);
  head.add(mk(RC.box(0.26, 0.28, 0.26, "canvas"), skin, 0, 0, 0));
  head.add(mk(RC.box(0.05, 0.22, 0.27, "canvas"), hair, -0.11, 0.02, 0));
  head.add(mk(RC.box(0.04, 0.05, 0.05, "canvas"), skin, 0.145, -0.02, 0));
  head.add(mk(RC.box(0.3, 0.07, 0.3, "canvas"), hat, -0.01, 0.17, 0));
  head.add(mk(RC.box(0.14, 0.025, 0.26, "canvas"), hat, 0.2, 0.14, 0));
  const arms = [0, 1].map(() => ({up: mk(UNIT, cloth), fore: mk(UNIT, cloth), hand: mk(RC.box(0.09, 0.09, 0.1, "canvas"), skin)}));
  for (const a of arms) rower.add(a.up, a.fore, a.hand);
  // today's poler (the game's, draw.rs): standing aft on the right, the pole hanging from his hand at the raft's edge,
  // leaning on the held key. Version 4 stands him at the edge and hangs the pole just outboard of the logs, so it
  // no longer runs down through the deck and his body; the lean and tilt are the game's.
  let today = null, man = null, pivot = null;
  if (withToday) {
    today = new THREE.Group(); g.add(today);
    man = RC.person(U, 0x7a6a3a); man.position.set(-0.7, LOG_TOP, 0.92); today.add(man);
    pivot = new THREE.Group(); pivot.position.set(-0.55, 1.45, 1.3); today.add(pivot);
    pivot.add(mk(RC.box(0.07, 4.6, 0.07, "planks"), m, 0, -1.4, 0));
  }
  g.userData = {lantern: lan, frame, oars, rower, legs, thighs, shins, feet, hip, torso, head, arms, today, man, pivot};
  return g;
}

// ======================================================================================================================
// RC.Oarsman: the raft kit and its stroke. opt.fp builds the first-person hands; opt.today the game's poler.
RC.Oarsman = function (U, opt) {
  opt = opt || {};
  const model = raftKit(U, opt.today), ud = model.userData;
  const hands = opt.fp ? handKit(U) : null, handL = hands && hands.make(), handR = hands && hands.make();
  const clock = {t: 0, running: false, dir: 1, strokes: []};
  const mkOar = () => ({rowing: false, rest: {}, range: 1, pose: {}, last: null, g: {}, floor: undefined});
  const oar = [mkOar(), mkOar()];   // [left, right]
  const me = {model, hands, handL, handR, clock, oar, body: 0, stage: "Resting", lean: 0, onBed: false};
  let bodyV = 0;
  function restPose(o) { o.theta = T.oars.catch_deg * D; o.dip = 0; o.feather = 1; o.body = 0; o.stage = -1; return o; }
  me.reset = () => {
    clock.t = 0; clock.running = false; clock.strokes = [];
    for (let i = 0; i < 2; i++) { const o = oar[i]; o.rowing = false; restPose(o.rest); Object.assign(o.pose, o.rest); o.last = null; o.range = 1; o.floor = undefined; oarGeom(i ? 1 : -1, o.pose, o.g); }
    me.body = 0; bodyV = 0; me.onBed = false;
  };
  // which oars the keys ask for: W both, S back both, A the right oar (turn left), D the left (turn right), Q E one oar
  me.wanted = (inp, Q, E) => {
    const W = inp.up, Sb = inp.down, A = inp.left, Dd = inp.right;
    return {l: W || Sb || Dd || !!Q, r: W || Sb || A || !!E, dir: Sb && !W ? -1 : 1,
      rangeL: W && A ? 1 - T.oars.turn_share : 1, rangeR: W && Dd ? 1 - T.oars.turn_share : 1};
  };
  // One physics step. fx (optional) draws the water's answer: fx.catch(g, side), fx.release(g, side), fx.wash(g), fx.drip(g).
  me.step = (h, raft, R, want, fx, simT) => {
    const P = period();
    if (!clock.running && (want.l || want.r)) { clock.running = true; clock.t = 0; clock.dir = want.dir; oar[0].rowing = want.l; oar[1].rowing = want.r; oar[0].range = want.rangeL; oar[1].range = want.rangeR; clock.strokes.push(simT); }
    if (clock.running) {
      const t0 = clock.t; clock.t += h;
      const s = T.stroke, eCatch = s.reach_s + s.catch_s, eRel = eCatch + s.drive_s + s.release_s * 0.45;
      for (let i = 0; i < 2; i++) {
        const o = oar[i]; if (!o.rowing || !fx) continue;
        const side = i ? 1 : -1;
        if (t0 < eCatch && clock.t >= eCatch && !o.g.onBed) fx.catch(o.g, side);
        if (t0 < eRel && clock.t >= eRel) fx.release(o.g, side);
        if (clock.t > eCatch && clock.t < eCatch + s.drive_s && Math.random() < h * 22) fx.wash(o.g);
        if (clock.t > eRel && Math.random() < h * T.water.drips / Math.max(0.2, s.recovery_s)) fx.drip(o.g);
      }
      if (clock.t >= P) {
        clock.t -= P;
        if (want.l || want.r) { clock.dir = want.dir; clock.strokes.push(simT); }
        for (let i = 0; i < 2; i++) {
          const o = oar[i], w = i ? want.r : want.l;
          if (o.rowing && !w) Object.assign(o.rest, o.pose);   // it finishes and rests where it ended: no snap
          o.rowing = w; o.range = i ? want.rangeR : want.rangeL;
        }
        if (!want.l && !want.r) { clock.running = false; clock.t = 0; }
      }
    }
    me.onBed = false;
    for (let i = 0; i < 2; i++) {
      const o = oar[i], side = i ? 1 : -1;
      if (o.rowing && clock.running) {
        oarPose(clock.t, clock.dir, o.range, o.pose);
        if (clock.t < T.stroke.reach_s) { const w = smooth(clock.t / T.stroke.reach_s); for (const k of ["theta", "dip", "feather"]) o.pose[k] = lerp(o.rest[k], o.pose[k], w); }
        Object.assign(o.rest, {theta: o.pose.theta, dip: o.pose.dip, feather: o.pose.feather});
      } else {
        const tgt = restPose({}), r = o.rest, k = 1 - Math.exp(-h * 5);
        for (const key of ["theta", "dip", "feather"]) r[key] += (tgt[key] - r[key]) * k;
        Object.assign(o.pose, r); o.pose.stage = -1; o.pose.body = 0;
      }
      // the bed under the blade (from last step's blade): in shallows the blade comes to rest on it, a bump, and
      // pushes nothing off it; the push is only ever the blade's drag through the water above the bed
      if (o.g.blade) {
        const bx = raft.wx(o.g.blade.x, o.g.blade.z), bz = raft.wz(o.g.blade.x, o.g.blade.z);
        o.floor = R.top(bx, bz) - raft.y + T.oars.bed_clear_m;
      }
      const prev = o.last ? o.g.blade.clone() : null;
      oarGeom(side, o.pose, o.g, o.floor);
      if (o.g.onBed && o.pose.dip > 0.3) me.onBed = true;
      // the blade's drag through the water, at the blade (loon.rs's law against the water)
      if (prev) {
        const yb = o.g.blade.y, share = c01((0.12 - yb) / 0.24) * c01((o.g.yb - (o.floor - T.oars.bed_clear_m)) / 0.2 + 0.0);
        if (share > 0) {
          const lvx = (o.g.blade.x - prev.x) / h, lvz = (o.g.blade.z - prev.z) / h, c = Math.cos(raft.yaw), s = Math.sin(raft.yaw);
          const px = raft.wx(o.g.blade.x, o.g.blade.z), pz = raft.wz(o.g.blade.x, o.g.blade.z);
          raft.pv(px, pz, PV); R.flow(px, pz, OUT);
          const rx = PV[0] + lvx * c - lvz * s - OUT[0], rz = PV[1] + lvx * s + lvz * c - OUT[1], rl = Math.hypot(rx, rz);
          const k = -0.5 * RHO * T.oars.blade_cd * T.oars.blade_m2 * share * rl;
          raft.force(px, pz, k * rx, k * rz);
        }
      }
      o.last = true;
    }
    // the body: the rowing oars' body share, else back to upright
    const rowingPose = oar.find(o => o.rowing && clock.running);
    const target = rowingPose ? rowingPose.pose.body : 0;
    if (rowingPose) { me.body = target; bodyV = 0; } else { bodyV += ((target - me.body) * 60 - bodyV * 14) * h; me.body += bodyV * h; }
    const st = rowingPose ? rowingPose.pose.stage : -1;
    me.stage = st >= 0 ? STAGES[st] : "Resting";
    if (rowingPose && st === 1) raft.vp -= 0.02 * h * 60 * (oar[0].rowing && oar[1].rowing ? 1 : 0.5);
  };
  // Show poses: freeze both oars on keyed pose i (0 the coiled reach .. 4 mid-recovery)
  me.KEYS = ["Reach (coiled)", "Catch", "Mid-drive", "Release", "Mid-recovery"];
  me.posed = i => {
    const s = T.stroke, keys = [s.reach_s * 0.99, s.reach_s + s.catch_s * 0.99, s.reach_s + s.catch_s + s.drive_s * 0.5,
      s.reach_s + s.catch_s + s.drive_s + s.release_s * 0.99, period() - s.recovery_s * 0.5];
    for (let j = 0; j < 2; j++) { oarPose(keys[i], 1, 1, oar[j].pose); oarGeom(j ? 1 : -1, oar[j].pose, oar[j].g, oar[j].floor); }
    me.body = oar[0].pose.body; me.stage = me.KEYS[i];
  };
  me.leanOf = body => T.stroke.exaggerate * lerp(T.rower.lean_catch_deg, T.rower.lean_release_deg, body) * D;
  // The drawing. fp: first person (the M36 hands on the handles, no body); otherwise the blocky rower's own arms.
  // Every part is drawn in exactly one view: the hands only in first person, the body and its arms only in third.
  me.sync = (fp, showRower) => {
    for (let i = 0; i < 2; i++) { if (!oar[i].g.pin) oarGeom(i ? 1 : -1, oar[i].pose, oar[i].g, oar[i].floor); placeOar(ud.oars[i], oar[i].g); }
    const r = T.rower, lean = me.leanOf(me.body);
    me.lean = lean;
    ud.rower.visible = showRower !== false;
    const bodyOn = ud.rower.visible && !fp;
    for (const k of ["legs", "hip"]) ud[k].visible = bodyOn;
    ud.hip.position.set(r.seat[0], r.seat[1], 0); ud.hip.rotation.set(0, 0, -lean);
    // legs: hip to knee to the foot braced on the beam's aft face
    for (let i = 0; i < 2; i++) {
      const s = i ? 1 : -1, H = V3(r.seat[0] + 0.04, r.seat[1] + 0.06, s * 0.11), K = V3(r.seat[0] + 0.33, r.seat[1] + 0.05, s * 0.12);
      const F = V3(DEF.oars.pin[0] - 0.08 - 0.11, 0.33, s * 0.12);
      limb(ud.thighs[i], H, K, 0.15); limb(ud.shins[i], K, F, 0.13); ud.feet[i].position.set(F.x, F.y, F.z);
    }
    for (let i = 0; i < 2; i++) {
      const side = i ? 1 : -1, g = oar[i].g, Sh = shoulders(lean, side);
      const El = elbowOf(Sh, g.grip, r.upper_arm_m, r.forearm_m, V3(-0.2, -1, side * 0.9));
      const a = ud.arms[i];
      for (const k of ["up", "fore", "hand"]) a[k].visible = bodyOn;
      limb(a.up, Sh, El, 0.11); limb(a.fore, El, g.grip, 0.1);
      // the block hand wraps the handle at the grip, along the loom
      a.hand.position.copy(g.grip); a.hand.quaternion.setFromUnitVectors(V3(0, 0, 1), V3(g.dir.x, g.dir.y, g.dir.z).multiplyScalar(side));
      if (hands) hands.place(i ? handR : handL, side, g.grip, g.dir.clone(), El);
    }
    if (hands) handL.visible = handR.visible = !!fp && ud.rower.visible;
    return lean;
  };
  // after the model's world matrix is current: the hands ride the raft (their matrices are in its frame)
  me.after = () => { if (hands && handL.visible) for (const m of [handL, handR]) { m.matrix.premultiply(model.matrixWorld); m.matrixWorldNeedsUpdate = true; } };
  me.eye = () => {
    const r = T.rower, lean = me.lean, base = V3(r.seat[0], r.seat[1] + 0.74, 0), c = Math.cos(lean), s = Math.sin(lean);
    const full = V3(r.seat[0] + s * 0.74, r.seat[1] + c * 0.74, 0);
    return base.lerp(full, r.head_follow * T.stroke.exaggerate).add(V3(0.1, 0, 0));
  };
  me.reset();
  return me;
};
const _Y = new THREE.Vector3(0, 1, 0), _Z = new THREE.Vector3(0, 0, 1);
function placeOar(o, g) {
  const u = o.p;
  u.position.copy(g.pin);
  // the loom along +x of the pivot: yaw about y, then droop
  const dirH = Math.hypot(g.dir.x, g.dir.z), yaw = Math.atan2(-g.dir.z, g.dir.x);
  u.quaternion.setFromAxisAngle(_Y, yaw);
  u.quaternion.multiply(new THREE.Quaternion().setFromAxisAngle(_Z, Math.atan2(g.dir.y, dirH)));
  o.lock.quaternion.setFromAxisAngle(_Y, yaw);
  const a = T.oars, rb = a.length_m - a.inboard_m - 0.3;
  o.loom.position.set((a.length_m - 0.55 - a.inboard_m) / 2, 0, 0);
  o.handle.position.set(-a.inboard_m / 2, 0, 0);
  o.bladeG.position.set(rb, 0, 0); o.bladeG.rotation.set(o.side * g.feather, 0, 0);
  o.grip.position.set(-a.inboard_m + 0.01, 0, 0);
}
function shoulders(lean, side) {
  const r = T.rower, s = r.seat, c = Math.cos(lean), sn = Math.sin(lean);
  return V3(s[0] + sn * r.shoulder_m, s[1] + c * r.shoulder_m, side * r.shoulder_half_m);
}
// for the clearance check (a measurement, run from the screenshot script)
RC.ROW = {T, DEF, oarPose, oarGeom, period, shoulders, elbowOf, deckBoxes, LOG_TOP, LOG_EDGE};

// ======================================================================================================================
RC.scenes.rowing = function () {
  const half = () => 11;
  const river = {x0: -100, x1: 210, z0: -40, z1: 40, center: x => 6 * Math.sin(x / 40), half,
    depth: () => 2.7, U: () => 0.45, level0: 0.55, slope: 0.0005, steps: [],
    rocks: [[30, 7.4], [95, -7.8], [150, 7.2]], bed: (x, z) => (Math.sin(x * 0.07) > 0.3 ? "gravel" : "sand")};
  let ctx, R, raft, crew, model, drops, swirls, graph;
  const inp = {up: false, down: false, left: false, right: false};
  const S = {mode: "proposed", view: "first", posing: false};
  const look = {yaw: 0, pitch: -0.72};
  // today's stroke (loon.rs): one at a time, alternating sides on W
  const today = {stroke: null, lastSide: 1, strokes: []};
  let stageName = "", speedHist = [];
  // the boat's menu (boatmenu.js): the raft's hold of ten slots (the game's) and its lantern, on or out
  const raftHold = [{item: "log", n: 12}, {item: "planks", n: 6}, {item: "rope", n: 2}, null, null, null, null, null, null, null], lantern = {lit: true};
  const acts = {};
  const keyHeld = id => acts[id] && acts[id].down;

  function resetState() {
    crew.reset(); today.stroke = null; today.strokes = []; speedHist = [];
    if (drops) drops.clear(); if (swirls) swirls.clear();
  }

  // ---- the water's answer to a blade, in the world
  function bladeWorld(g) { return [raft.wx(g.blade.x, g.blade.z), raft.y + g.blade.y, raft.wz(g.blade.x, g.blade.z)]; }
  function splash(x, z, n, sp, side) {
    const y = R.surf(x, z);
    const ox = -Math.sin(raft.yaw) * side, oz = Math.cos(raft.yaw) * side;   // outboard
    for (let i = 0; i < n; i++) {
      const a = Math.random() * Math.PI * 2, u = 0.35 + 0.65 * Math.random();
      drops.spawn(x + Math.cos(a) * 0.12, y + 0.02, z + Math.sin(a) * 0.12,
        (Math.cos(a) * 0.5 + ox * 0.4) * sp * u, sp * (0.55 + 0.6 * Math.random()), (Math.sin(a) * 0.5 + oz * 0.4) * sp * u, 1.2, 0.028 + 0.02 * Math.random());
    }
    for (let i = 0; i < 6; i++) drops.spawn(x + (Math.random() - 0.5) * 0.4, y, z + (Math.random() - 0.5) * 0.4, 0, 0, 0, 0.8 + Math.random() * 0.6, 0.035, 1);
  }
  const fx = {
    catch(g, side) { const [x, , z] = bladeWorld(g); splash(x, z, T.water.splash_drops, T.water.splash_speed_mps, side); },
    release(g, side) {
      const w = T.water, [x, , z] = bladeWorld(g);
      // the blade's two edges, along the loom: one swirl each, turning opposite ways
      const ex = g.dir.x, ez = g.dir.z, c = Math.cos(raft.yaw), s = Math.sin(raft.yaw);
      const wx = ex * c - ez * s, wz = ex * s + ez * c, Lh = Math.hypot(wx, wz) || 1;
      for (const k of [-1, 1]) swirls.spawn(x + wx / Lh * 0.32 * k, z + wz / Lh * 0.32 * k, w.puddle_m, k * side, w.puddle_s, w.puddle_spin_rps);
    },
    wash(g) { const [x, , z] = bladeWorld(g); drops.spawn(x, R.surf(x, z) + 0.02, z, (Math.random() - 0.5) * 0.6, 0.5 + Math.random() * 0.6, (Math.random() - 0.5) * 0.6, 0.6, 0.026); },
    drip(g) { const [x, y, z] = bladeWorld(g); drops.spawn(x, y - 0.1, z, 0, -0.2, 0, 1.2, 0.02); },
  };

  function stepToday(h) {
    // loon.rs: the stroke begun when the last has recovered and a key is held
    const p = {reach: 1.35, catchZ: -0.2, len: 1.4, power: 0.8, recovery: 0.7, area: 0.7, cd: 1.25};
    if (today.stroke) { today.stroke.phase += h / (today.stroke.phase < 1 ? p.power : p.recovery); if (today.stroke.phase >= 2) today.stroke = null; }
    if (!today.stroke) {
      let s = null;
      if (inp.left) s = {side: 1, dir: 1}; else if (inp.right) s = {side: -1, dir: 1};
      else if (inp.up) s = {side: -today.lastSide, dir: 1}; else if (inp.down) s = {side: -today.lastSide, dir: -1};
      if (s) { today.lastSide = s.side; s.phase = 0; today.stroke = s; today.strokes.push(ctx.S.simT); }
    }
    stageName = today.stroke ? (today.stroke.phase < 1 ? "Power" : "Recovery") : "Resting";
    if (today.stroke && today.stroke.phase < 1) {
      const u = today.stroke.phase, travel = p.len * (1 - Math.cos(Math.PI * u)) / 2;
      // the game's z is aft; here +x is the bow
      const lx = today.stroke.dir > 0 ? -(p.catchZ + travel) : -(p.catchZ + p.len - travel), lz = today.stroke.side * p.reach;
      const speed = today.stroke.dir * p.len / p.power * Math.PI / 2 * Math.sin(Math.PI * u);
      const px = raft.wx(lx, lz), pz = raft.wz(lx, lz); raft.pv(px, pz, PV);
      const c = Math.cos(raft.yaw), s = Math.sin(raft.yaw);
      const rx = PV[0] - speed * c, rz = PV[1] - speed * s, rl = Math.hypot(rx, rz);   // a pole: against the ground
      const k = -0.5 * RHO * p.cd * p.area * rl; raft.force(px, pz, k * rx, k * rz);
    }
  }

  const def = {
    river, posts: [[-62, -1], [-24, 1], [14, -1], [52, 1], [90, -1]], followDefault: true,
    home: {yaw: 0.9, pitch: -0.36, dist: 10},
    hold: inp,
    build(c) {
      ctx = c; R = c.river;
      raft = new RC.Hull({L: 4.4, B: 2.4, m: 1280, draft: 0.3, cA: 420, cS: 1100, nx: 3, nz: 3});
      crew = RC.Oarsman(c.U, {fp: true, today: true}); model = crew.model; c.scene.add(model, crew.handL, crew.handR);
      drops = Drops(c.U, 260); c.scene.add(drops.mesh);
      swirls = Swirls(c.U, 28, c.scene);
      graph = document.getElementById("surge");
      def.reset();
    },
    reset() {
      raft.x = -70; raft.z = R.cfg.center(-70) - 1.5; raft.yaw = 0.12; raft.vx = raft.vz = raft.w = 0; raft.settle(R);
      resetState();
    },
    step(h) {
      if (S.posing) return;
      raft.water(R);
      if (S.mode === "today") stepToday(h);
      else { crew.step(h, raft, R, crew.wanted(inp, keyHeld("qoar"), keyHeld("eoar")), fx, ctx.S.simT); stageName = crew.stage; }
      raft.integrate(h, R); raft.terrain(R);
      drops.step(h, R); swirls.step(h, R);
      if (raft.x > R.cfg.x1 - 8) def.reset();
      const c = Math.cos(raft.yaw), s = Math.sin(raft.yaw);
      R.flow(raft.x, raft.z, OUT);
      if (Math.round(ctx.S.simT * 120) % 6 === 0) { speedHist.push((raft.vx - OUT[0]) * c + (raft.vz - OUT[1]) * s); if (speedHist.length > 120) speedHist.shift(); }
    },
    sync() {
      const ud = model.userData, fp = S.view === "first", proposed = S.mode === "proposed";
      // Show poses: freeze on each keyed pose in turn (both oars, forward)
      if (S.posing && proposed) { crew.posed(Math.floor(performance.now() / 1300) % 5); stageName = crew.stage; }
      RC.placeHull(model, raft);
      ud.frame.visible = proposed;
      for (const o of ud.oars) o.p.visible = proposed;
      ud.today.visible = !proposed; ud.man.visible = !proposed && !fp;
      def._lean = crew.sync(fp, proposed);
      if (!proposed) {
        // today's pole (draw.rs `pole`): set from the held keys, in one frame. The side tilt is drawn outboard for
        // either key: tilted inboard (as the game does on one of them) it runs through the deck.
        const f = (inp.up ? 1 : 0) - (inp.down ? 1 : 0), t = (inp.right ? 1 : 0) - (inp.left ? 1 : 0);
        ud.pivot.rotation.set(-Math.abs(t) * 0.35, 0, f ? -0.5 * f : 0.12);
      }
      ud.lantern.userData.out = !lantern.lit;
      model.updateMatrixWorld(true);
      crew.after();
      if (!proposed) { const u = ud.lantern.userData; u.glass.color.set(ctx.S.night ? 0xffd28a : 0x3a3226); u.glow.visible = ctx.S.night; }
      drops.sync(); swirls.sync(R);
      drawGraph();
    },
    camera(cam) {
      if (S.view !== "first") { if (cam.fov !== 50) { cam.fov = 50; cam.near = 0.1; cam.updateProjectionMatrix(); } return false; }
      if (cam.fov !== 72) { cam.fov = 72; cam.near = 0.04; cam.updateProjectionMatrix(); }
      const eye = S.mode === "today" ? V3(-0.7, 1.75, 0.92) : crew.eye();
      cam.position.copy(eye.applyMatrix4(model.matrixWorld));
      const q = new THREE.Quaternion().setFromRotationMatrix(new THREE.Matrix4().extractRotation(model.matrixWorld));
      cam.quaternion.copy(q).multiply(new THREE.Quaternion().setFromEuler(new THREE.Euler(look.pitch, -Math.PI / 2 + look.yaw, 0, "YXZ")));
      return true;
    },
    look(dx, dy) {
      if (S.view !== "first") return false;
      look.yaw = clamp(look.yaw - dx * 0.005, -2.6, 2.6); look.pitch = clamp(look.pitch - dy * 0.005, -1.3, 0.9);
      return true;
    },
    // Proposed: the raft's lantern is a lamp and lights the deck, the water, the oars, the splash and the hands.
    // Today: the game's lantern flame shows at dusk (draw.rs, Moving::Flame) but lights nothing.
    lanterns: () => S.mode === "proposed" ? [model.userData.lantern] : [],
    menu: {title: "Raft", condition: () => "Dry", hold: raftHold, lantern: {get: () => lantern.lit, set: v => { lantern.lit = v; }}},
    follow: () => [raft.x, raft.y + 0.8, raft.z],
    hud() {
      R.flow(raft.x, raft.z, OUT);
      const c = Math.cos(raft.yaw), s = Math.sin(raft.yaw), thru = (raft.vx - OUT[0]) * c + (raft.vz - OUT[1]) * s;
      const list = S.mode === "today" ? today.strokes : crew.clock.strokes;
      const now = ctx.S.simT; while (list.length && now - list[0] > 8) list.shift();
      const rate = list.length > 1 && now - list[list.length - 1] < 3 ? Math.round(60 * (list.length - 1) / (list[list.length - 1] - list[0])) : 0;
      const depth = R.surf(raft.x, raft.z) - R.top(raft.x, raft.z);
      return [["", S.mode === "today" ? "Today: pole" : "Proposed: oars", "on"], ["Stage", stageName], ["Strokes", rate + "/min"], ["Through water", thru.toFixed(2) + " m/s"],
        ["Turning", (raft.w * 180 / Math.PI).toFixed(0) + "°/s"], ["Depth", depth.toFixed(1) + " m"],
        ...(S.mode === "proposed" && crew.onBed ? [["", "Oar on the bed", "warn"]] : [])];
    },
    keys: "W both · S back · A D one side · Q E one oar · drag to look",
    choices: [
      {label: "Rowing", options: [["today", "Today"], ["proposed", "Proposed"]], get: () => S.mode, set: v => { S.mode = v; resetStroke(); }},
      {label: "View", options: [["first", "First person"], ["third", "Third person"]], get: () => S.view, set: v => { S.view = v; }},
    ],
    actions: [
      {id: "qoar", label: "Left oar", key: "Q", hold: true, run() {}},
      {id: "eoar", label: "Right oar", key: "E", hold: true, run() {}},
      {id: "poses", label: "Show poses", key: "P", run() { if (S.mode === "proposed") S.posing = !S.posing; }, label2: () => S.posing ? "Stop poses" : "Show poses"},
    ],
    tune(box) {
      for (const [gname, key, lo, hi, st] of SLIDERS) {
        const lab = document.createElement("label"); lab.className = "sl";
        const name = document.createElement("span"); name.textContent = key; lab.appendChild(name);
        const inpEl = document.createElement("input"); inpEl.type = "range"; inpEl.min = lo; inpEl.max = hi; inpEl.step = st; inpEl.value = T[gname][key];
        const out = document.createElement("output"); out.textContent = fmt(T[gname][key]);
        inpEl.addEventListener("input", () => { T[gname][key] = +inpEl.value; out.textContent = fmt(T[gname][key]); ronBox.value = ron(); });
        lab.appendChild(inpEl); lab.appendChild(out); box.querySelector(".sliders").appendChild(lab);
      }
      const ronBox = box.querySelector("textarea"); ronBox.value = ron();
      box.querySelector("button").addEventListener("click", () => {
        for (const g of GROUPS) Object.assign(T[g], JSON.parse(JSON.stringify(DEF[g])));
        const els = [...box.querySelectorAll("input[type=range]")];
        els.forEach((el, i) => { const [gname, key] = SLIDERS[i]; el.value = T[gname][key]; el.nextSibling.textContent = fmt(T[gname][key]); });
        ronBox.value = ron();
      });
    },
    // for the screenshots: set the clock to a time in the stroke with both oars rowing
    _at(t, dir) { const c = crew.clock; c.running = true; c.t = t; c.dir = dir || 1; for (const o of crew.oar) { o.rowing = true; o.range = 1; o.last = null; Object.assign(o.rest, oarPose(t, c.dir, 1, {})); } },
    _S: S, _look: look, _raft: () => raft, _crew: () => crew,
  };
  for (const a of def.actions) acts[a.id] = a;
  function resetStroke() { resetState(); S.posing = false; }
  // the surge: speed through the water over the last 12 s
  function drawGraph() {
    if (!graph) return;
    const g = graph.getContext("2d"), w = graph.width, hgt = graph.height;
    g.clearRect(0, 0, w, hgt); g.fillStyle = "rgba(1,14,26,0.72)"; g.fillRect(0, 0, w, hgt);
    g.strokeStyle = "#3e586e"; g.beginPath(); for (const v of [0, 1, 2]) { const y = hgt - 6 - v / 2.2 * (hgt - 18); g.moveTo(0, y); g.lineTo(w, y); } g.stroke();
    g.strokeStyle = "#84e3b5"; g.lineWidth = 2; g.beginPath();
    speedHist.forEach((v, i) => { const x = w - (speedHist.length - 1 - i) * (w / 120), y = hgt - 6 - clamp(v, -0.4, 2.2) / 2.2 * (hgt - 18); if (i) g.lineTo(x, y); else g.moveTo(x, y); });
    g.stroke(); g.lineWidth = 1;
    g.fillStyle = "#cfaa81"; g.font = "600 12px 'IBM Plex Sans Condensed', sans-serif"; g.fillText("Surge, m/s", 6, 13);
  }
  return def;
};
RC.placeHull = function (obj, b) { obj.position.set(b.x, b.y, b.z); obj.rotation.set(b.roll, -b.yaw, b.pitch, "YZX"); };
})();
