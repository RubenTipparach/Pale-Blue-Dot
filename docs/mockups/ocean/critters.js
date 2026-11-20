// Beavers and otters (ocean.html's #critters tab; openspec/changes/ocean-environments design B28).
// The owner, 2026-11-20: "can you improve on beaver and otter animations?", then "make me a mockup with those two
// animls running around interacting with env". A valley stream with a beaver pond, its dam and lodge, a muddy bank
// with an otter slide, aspens and willows, a riffle with trout, and the stream's mouth onto a kelp-edged sea. Three
// animals live in it, each a rig of pivots (body, hind and chest segments, neck and head, four two-bone legs, a
// three-part tail), each choosing what to do next by a seeded draw. Every action is windup, anticipation, hit and
// recoil (the animation skill); a gait's phase runs on the distance moved, so no foot slides; the tail and head lag
// the body. Uses the page's helpers (setupStage, buildColumns, makeCaps, makeSea, W.lit, C3) and its one renderer.
"use strict";
function sceneCritters(stage) {
  // ======================= tunables (design B28): the sliders print them as RON =======================
  const TUNE_SPEC = [
    ["beaver.walk", "stride_m", 0.36, 0.2, 0.7, 0.01], ["beaver.walk", "stance", 0.62, 0.4, 0.8, 0.01], ["beaver.walk", "speed_mps", 0.5, 0.15, 1.0, 0.01], ["beaver.walk", "foot_lift_m", 0.05, 0, 0.12, 0.005],
    ["beaver.walk", "waddle_roll_rad", 0.10, 0, 0.3, 0.01], ["beaver.walk", "hip_bob_m", 0.02, 0, 0.06, 0.002], ["beaver.walk", "head_lag_s", 0.10, 0.02, 0.4, 0.01], ["beaver.walk", "tail_lag_s", 0.18, 0.02, 0.5, 0.01],
    ["beaver.gnaw", "beat_s", 0.42, 0.2, 1.0, 0.01], ["beaver.gnaw", "pull_rad", 0.35, 0.1, 0.7, 0.01], ["beaver.gnaw", "chips_per_bite", 4, 0, 10, 1], ["beaver.gnaw", "bites_to_fell", 12, 4, 30, 1],
    ["beaver.fell", "creak_s", 1.0, 0.3, 2.5, 0.05], ["beaver.fell", "lean_back_rad", 0.05, 0, 0.15, 0.005], ["beaver.fell", "fall_s", 1.7, 0.8, 3.5, 0.05], ["beaver.fell", "bounce_rad", 0.10, 0, 0.25, 0.01], ["beaver.fell", "settle_s", 0.9, 0.3, 2, 0.05],
    ["beaver.swim", "speed_mps", 0.75, 0.2, 1.4, 0.01], ["beaver.swim", "scull_hz", 1.3, 0.4, 3, 0.05], ["beaver.swim", "scull_rad", 0.35, 0, 0.8, 0.01], ["beaver.swim", "head_above_m", 0.07, 0, 0.2, 0.005],
    ["beaver.slap", "windup_s", 0.30, 0.1, 0.8, 0.01], ["beaver.slap", "anticipate_s", 0.22, 0, 0.6, 0.01], ["beaver.slap", "strike_s", 0.08, 0.03, 0.3, 0.005], ["beaver.slap", "recoil_s", 0.35, 0.1, 0.8, 0.01], ["beaver.slap", "raise_rad", 1.15, 0.4, 1.6, 0.01],
    ["beaver.shake", "hz", 7.0, 2, 12, 0.1], ["beaver.shake", "time_s", 1.0, 0.4, 2.5, 0.05], ["beaver.shake", "roll_rad", 0.45, 0.1, 0.9, 0.01], ["beaver.shake", "dry_s", 20, 4, 60, 0.5],
    ["river_otter.bound", "stride_m", 0.85, 0.4, 1.4, 0.01], ["river_otter.bound", "stance", 0.38, 0.25, 0.6, 0.01], ["river_otter.bound", "speed_mps", 1.5, 0.5, 3, 0.05], ["river_otter.bound", "arch_rad", 0.42, 0, 0.8, 0.01], ["river_otter.bound", "lift_m", 0.07, 0, 0.18, 0.005],
    ["river_otter.slide", "accel_mps2", 3.0, 0.5, 6, 0.1], ["river_otter.slide", "max_mps", 4.5, 1, 8, 0.1],
    ["river_otter.porpoise", "arc_m", 0.45, 0.1, 0.9, 0.01], ["river_otter.eat", "beat_s", 0.36, 0.15, 0.8, 0.01], ["river_otter.look", "hold_s", 0.9, 0.2, 2.5, 0.05],
    ["sea_otter.crack", "windup_s", 0.32, 0.1, 0.8, 0.01], ["sea_otter.crack", "anticipate_s", 0.16, 0, 0.5, 0.01], ["sea_otter.crack", "strike_s", 0.07, 0.03, 0.25, 0.005], ["sea_otter.crack", "recoil_s", 0.26, 0.1, 0.7, 0.01],
    ["sea_otter.crack", "raise_m", 0.16, 0.05, 0.3, 0.005], ["sea_otter.crack", "cracks", 4, 1, 8, 1],
    ["sea_otter.roll", "roll_s", 0.7, 0.3, 1.5, 0.05], ["sea_otter.dive", "dive_s", 5.0, 2, 10, 0.1], ["sea_otter.sleep", "kelp_wrap_s", 2.4, 0.8, 5, 0.1],
    // version 4 (the owner, 2026-11-20): "the arms on beaver and otter should animate when they move or stand up", and on
    // the tufts "the fur shaking is cool, but a little over the top, I would want more subtle"
    ["shared.forelegs", "anticipate_s", 0.18, 0, 0.5, 0.01], ["shared.forelegs", "rise_s", 0.42, 0.15, 1.0, 0.01], ["shared.forelegs", "lower_s", 0.55, 0.2, 1.2, 0.01],
    ["shared.forelegs", "overshoot", 0.12, 0, 0.4, 0.01], ["shared.forelegs", "swing_curl_rad", 0.8, 0, 1.5, 0.05], ["shared.forelegs", "paddle_rad", 0.7, 0, 1.2, 0.05],
    ["shared.fur", "spring_hz", 3.6, 1, 8, 0.1], ["shared.fur", "damping", 0.75, 0.1, 1.2, 0.01], ["shared.fur", "sway", 0.4, 0, 1.5, 0.05], ["shared.fur", "max_m", 0.012, 0.002, 0.04, 0.001],
    ["shared.fur", "wet_spike", 0.4, 0, 1, 0.05], ["shared.fur", "shake", 0.45, 0, 1.5, 0.05],
    ["all", "exaggerate", 1.0, 0.3, 2.0, 0.05],
  ];
  const TU = {}; for (const [b, k, v] of TUNE_SPEC) TU[b + "." + k] = v;
  const tn = key => TU[key], EX = () => TU["all.exaggerate"];

  // ======================= small helpers =======================
  const V3 = (x = 0, y = 0, z = 0) => new THREE.Vector3(x, y, z);
  const c01 = x => Math.max(0, Math.min(1, x)), frac = x => x - Math.floor(x);
  const easeOut = t => 1 - (1 - t) * (1 - t), easeIn3 = t => t * t * t, smooth = t => t * t * (3 - 2 * t);
  const angWrap = a => { while (a > Math.PI) a -= 2 * Math.PI; while (a < -Math.PI) a += 2 * Math.PI; return a; };
  const mulberry = seed => () => { seed |= 0; seed = seed + 0x6D2B79F5 | 0; let t = Math.imul(seed ^ seed >>> 15, 1 | seed); t = t + Math.imul(t ^ t >>> 7, 61 | t) ^ t; return ((t ^ t >>> 14) >>> 0) / 4294967296; };
  const SEED = +(Q.get("seed") || 7);
  // one strike in four stages: windup to `top`, anticipation easing to `coil` and held, the hit to `contact` (t cubed:
  // fastest at contact), the recoil kicking past rest by `kick` and easing back to 0. d = [windup, anticipate, hit, recoil]
  function strike(t, d, top, coil, contact, kick) {
    const [w, a, h, r] = d;
    if (t < w) return {v: top * easeOut(t / w), st: "windup"};
    t -= w; if (t < a) return {v: lerp(top, coil, smooth(t / a)), st: "anticipation"};
    t -= a; if (t < h) return {v: lerp(coil, contact, easeIn3(t / h)), st: "hit"};
    t -= h; const s = c01(t / r); return {v: lerp(contact, 0, easeOut(s)) + kick * Math.sin(Math.PI * Math.min(1, s * 1.7)) * (1 - s), st: s < 1 ? "recoil" : "rest"};
  }
  const strikeKeys = (d, off = 0) => [["windup", off + d[0]], ["anticipation", off + d[0] + d[1]], ["hit", off + d[0] + d[1] + d[2]], ["recoil", off + d[0] + d[1] + d[2] + d[3] * 0.3]];

  // ======================= the valley =======================
  const W = setupStage("critters", stage, {marine: true, water: "river", far: 900,
    extra: `<div class="lab2"><span>Time of day</span><b id="crHourV">13:00</b></div><input type="range" id="crHour" min="0" max="24" step="0.25" value="${Q.get("hour") ?? 13}" aria-label="Time of day"><div class="lab2"><span>midnight</span><span>dawn</span><span>noon</span><span>dusk</span><span>midnight</span></div>`});
  W.water = WATERS.river;
  const M = commonMats(W), B = SHEET;
  const P_LVL = 2.55;                                     // the pond's surface, held by the dam
  const zc = x => x > 0 ? 2.5 * Math.sin(x / 8) : 0;       // the stream's line below the dam
  const inPond = (x, z) => x < -2.2 && ((x + 12.5) / 11.5) ** 2 + (z / 7.4) ** 2 < 1;
  const cells = rectRegion(-46, 100, -28, 28, (q, r, x, z) => {
    const d = Math.abs(z - zc(x));
    if (x >= -2.2 && x < 0.6 && Math.abs(z) < 7.5) return {h: r === 0 ? 2 : 3, m: "sticks", dam: true, spill: r === 0};
    if (inPond(x, z)) { const e = ((x + 12.5) / 11.5) ** 2 + (z / 7.4) ** 2; return {h: e < 0.45 ? 0 : 1, s: B.swamp, top: [0, 3], pond: true}; }
    if (x < -23 && Math.abs(z) < 1.6) return {h: 3, s: B.beach, top: [0, 3], inflow: true};
    if (x >= 0.6 && x < 27 && d < 1.9) { const rock = x > 13 && hsh(q * 7, r * 3) > 0.72; return rock ? {h: 1, s: B.mountains, top: [0, 3]} : {h: 0, s: B.beach, top: [0, 3], stream: true, riffle: x > 13}; }
    if (x >= 27 && d < 2.6 && x < 38) return {h: -1, s: B.ocean, top: [0, 0], sea: true};
    if (x >= 38) { const h = Math.max(-8, Math.round(-1 - (x - 38) * 0.3 + (fbm(x / 6, z / 6) - 0.5) * 1.5)); return {h, s: B.ocean, top: h > -3 ? [3, 2] : [0, 0], sea: true}; }
    if (x >= 30) return {h: 1, s: B.beach, top: [0, 0]};
    if (x > 0.6) { const n = fbm(x / 9, z / 9); return {h: x > 13 ? (d < 5 ? 1 : 2) : (d < 5 ? 2 : 3) + (Math.abs(z) > 16 ? 1 : 0) + (n > 0.68 ? 1 : 0), s: B.fields, top: [0, 0]}; }
    if (x < -23) return {h: 4 + (Math.abs(z) > 12 ? 1 : 0), s: B.fields, top: [0, 0]};
    if (z < 0) return {h: z > -9.8 ? 3 : z > -16 ? 4 : 5, s: B.fields, top: [0, 0]};
    return {h: z > 19 ? 4 : 3, s: B.fields, top: [0, 0]};
  });
  // the lodge: a dome of stick cells in the pond
  const LODGE = V3(-18, 0, 2.2); { const [q0, r0] = cellOf(LODGE.x, LODGE.z); cells.set(q0 + "," + r0, {h: 4, m: "sticks", lodge: true}); for (const [dq, dr] of DIRS) cells.set((q0 + dq) + "," + (r0 + dr), {h: 3, m: "sticks", lodge: true}); }
  // ramps of mud: the beaver's way in and out on the south bank, the otter's path up to the ridge, the otter slide
  const RAMPS = [
    {id: "beaver", a: V3(-12, 3.0, 9.6), b: V3(-12, 2.12, 6.4), w: 1.7},
    {id: "path", a: V3(-26.0, 4.0, -10.2), b: V3(-22.0, 2.12, -3.0), w: 1.4},
    {id: "slide", a: V3(-14.6, 4.0, -10.6), b: V3(-13.6, 2.05, -5.6), w: 1.3},
  ];
  const rampAt = (R, x, z) => { const ax = R.b.x - R.a.x, az = R.b.z - R.a.z, L2 = ax * ax + az * az, t = ((x - R.a.x) * ax + (z - R.a.z) * az) / L2; if (t < -0.02 || t > 1.02) return null; const lat = Math.abs((x - R.a.x) * az - (z - R.a.z) * ax) / Math.sqrt(L2); if (lat > R.w / 2) return null; return lerp(R.a.y, R.b.y, c01(t)); };
  for (const [k, c] of cells) { if (c.m) continue; const [q, r] = k.split(",").map(Number), x = hx(q, r), z = hz(r); for (const R of RAMPS) { const y = rampAt(Object.assign({}, R, {w: R.w + HR * 1.2}), x, z); if (y != null && c.h > y - 0.2) c.h = Math.floor(y - 0.25); } }
  buildColumns(W, cells, M);
  const colTop = (x, z) => { const k = cellOf(x, z), c = cells.get(k[0] + "," + k[1]); return c ? c.h : -8; };
  const mudMat = W.std({map: tex("game/peat", 2), color: 0x9a7a62, roughness: 0.35});
  for (const R of RAMPS) { const L = Math.hypot(R.b.x - R.a.x, R.b.z - R.a.z), dy = R.b.y - R.a.y, len = Math.hypot(L, dy); const m = new THREE.Mesh(new THREE.BoxGeometry(len, 0.9, R.w), mudMat); boxUV(m.geometry, len, 0.9, R.w);
    m.position.set((R.a.x + R.b.x) / 2, (R.a.y + R.b.y) / 2 - 0.45 * Math.cos(Math.atan2(dy, L)), (R.a.z + R.b.z) / 2); m.rotation.order = "YZX"; m.rotation.y = Math.atan2(-(R.b.z - R.a.z), R.b.x - R.a.x); m.rotation.z = Math.atan2(dy, L); W.scene.add(m); }
  // the otter's rock in the pond
  const ROCK = V3(-6.6, 0, -4.0), ROCK_R = 0.62, ROCK_TOP = P_LVL + 0.36;
  { const g = new THREE.DodecahedronGeometry(1, 0); g.scale(ROCK_R * 1.15, 1.0, ROCK_R); const m = new THREE.Mesh(g, W.std({map: tex("game/granite", 1), roughness: 0.9})); m.position.set(ROCK.x, ROCK_TOP - 1.0, ROCK.z); W.scene.add(m); }
  const groundAt = (x, z) => { let y = colTop(x, z); for (const R of RAMPS) { const r = rampAt(R, x, z); if (r != null) y = Math.max(y, r); } const dr = Math.hypot(x - ROCK.x, z - ROCK.z); if (dr < ROCK_R) y = Math.max(y, ROCK_TOP - dr * dr * 0.5); return y; };
  // the water: the pond and the stream as caps on their cells (the page's fresh-water look), the sea as the wave surface
  const caps = new Map(), seaCells = new Map();
  for (const [k, c] of cells) {
    const [q, r] = k.split(",").map(Number), x = hx(q, r), z = hz(r), fl = [1, 2.5 / 8 * Math.cos(x / 8)];
    if (c.pond || c.spill) caps.set(k, {wl: P_LVL, flow: c.spill ? [1, 0] : [0, 0]});
    else if (c.inflow) caps.set(k, {wl: 3.32, flow: [1, 0]});
    else if (c.stream) caps.set(k, {wl: c.riffle ? 0.42 : 1.0, flow: fl});
    else if (c.sea) seaCells.set(k, {wl: 0});
  }
  makeCaps(W, caps, {lower: seaCells, amp: 0.012, mud: 0.25});
  const sea = makeSea(W, {amp: 0.16, seaOnly: true});
  const levelAt = (x, z) => { const k = cellOf(x, z), key = k[0] + "," + k[1], c = caps.get(key); if (c) return c.wl; if (seaCells.has(key) || x > 99) return sea.h(x, z, W.t, W.camera.position); return -1000; };
  bakeWater(W, (x, z) => { const k = cellOf(x, z), key = k[0] + "," + k[1], c = caps.get(key); return c ? c.wl : seaCells.has(key) ? 0 : -1000; }, -60, -40, 140, 40, 320, (x, z) => colTop(x, z));
  W.U.uWLdef.value = 0;
  // kelp along the sea's edge, swaying in the surge (the sea moves it)
  const kelpMat = W.std({map: spriteTex("card-kelp"), alphaTest: 0.5, side: THREE.DoubleSide, roughness: 1}), fronds = [];
  for (const [k, c] of cells) { const [q, r] = k.split(",").map(Number), x = hx(q, r), z = hz(r); if (!c.sea || x < 46 || x > 72 || c.h > -3 || hsh(q, r * 5) > 0.45) continue;
    const g = new THREE.Group(); g.position.set(x + (hsh(r, q) - 0.5), c.h, z); W.scene.add(g); const L = -c.h, n = Math.ceil(L / 4);
    for (let i = 0; i < n; i++) for (let j = 0; j < 2; j++) { const m = new THREE.Mesh(new THREE.PlaneGeometry(1.1, 4).translate(0, 2, 0), kelpMat); m.position.y = i * 4; m.rotation.y = j * Math.PI / 2 + hsh(q, r) * 3; g.add(m); }
    g.scale.y = (L + 0.1) / (n * 4); const can = new THREE.Mesh(new THREE.PlaneGeometry(1.1, 3.4).rotateX(-Math.PI / 2), kelpMat); can.position.set(x + 1.2, -0.04, z); can.rotation.y = hsh(q + 1, r) * 6; W.scene.add(can); fronds.push({g, ph: hsh(q, r) * 6}); }
  // trees: aspens on the south meadow (three the beaver fells), willows by the water, firs on the ridge
  const aspenBark = W.std({map: tex("tex-aspen", 0.5), roughness: 0.9}), aspenLeaf = W.std({map: tex("game/leaves", 2), color: 0xe3ecaa});
  const treeMats = {bark: M.bark, leaves: M.leaves};
  for (const [k, c] of cells) { if (c.m || c.pond || c.stream || c.sea || c.inflow) continue; const [q, r] = k.split(",").map(Number), x = hx(q, r), z = hz(r), p = hsh(q * 5, r * 3);
    if (x > -30 && x < -2 && z > 15 && p < 0.28) aspen(x, c.h, z, 0.85 + p);
    else if (x > 0.6 && x < 27 && Math.abs(z - zc(x)) < 7 && Math.abs(z - zc(x)) > 3.5 && p < 0.22) tree(W, treeMats, x, c.h, z, 0.9, {willow: true});
    else if (z < -14 && x < 6 && x > -40 && p < 0.25) tree(W, treeMats, x, c.h, z, 0.95);
    else if (x < -26 && Math.abs(z) > 4 && p < 0.14) tree(W, treeMats, x, c.h, z, 0.9, {willow: Math.abs(z) < 9}); }
  function aspen(x, y, z, s) { const g = new THREE.Group(); g.position.set(x, y, z); W.scene.add(g); prism(W, aspenBark, 0.15 * s, 4.4 * s, 0, 0, 0, g); for (const [rr, h, yy] of [[1.2, 1.4, 2.6], [0.95, 1.2, 3.6], [0.55, 1.0, 4.5]]) prism(W, aspenLeaf, rr * s, h * s, 0, yy * s, 0, g); return g; }
  // the reeds and the lilies of the pond
  for (const [k, c] of cells) { const [q, r] = k.split(",").map(Number), x = hx(q, r), z = hz(r), p = hsh(q * 3, r * 11);
    if (c.pond && c.h === 1 && p < 0.16 && Math.hypot(x + 12, z - 6) > 3) { const m = card(W, spriteTex("card-lily"), 1.3, 1.3, {flat: true}); m.position.set(x + 0.3, P_LVL + 0.02, z); m.rotation.y = p * 40; }
    else if (!c.m && !c.pond && c.h === 3 && x > -24 && x < -2 && Math.abs(z) < 9.5 && p < 0.3 && Math.hypot(x + 12, z - 9) > 2.5 && Math.hypot(x + 14, z + 9) > 2.5) crossed(W, "card-reed", 1.8, 1.8, x, c.h, z, {n: 2, rot: p * 5}); }

  // ======================= night lights: a lantern by the beavers' bank, the fishing camp, a boat, fireflies ==========
  { const px = -8.6, pz = 9.6; box(W, M.iron, 0.12, 2.2, 0.12, px, 3 + 1.1, pz); W.lamp(px, 3 + 2.3, pz, 0xffb35a, 2.4, 15, {size: 0.3}); }
  { const cx = 32, cz = -8.5; box(W, M.timber, 3.2, 2.2, 2.6, cx, 1 + 1.1, cz); box(W, M.slate, 3.8, 0.35, 3.2, cx, 1 + 2.35, cz); W.lamp(cx - 1.75, 1 + 1.7, cz + 0.4, 0xffb35a, 2.2, 13);
    for (const a of [0, 1.1, 2.2]) box(W, M.driftwood, 0.9, 0.12, 0.12, cx + 1.2 + Math.cos(a) * 0.3, 1.07, cz + 3.2 + Math.sin(a) * 0.3).rotation.y = a; W.lamp(cx + 1.2, 1.35, cz + 3.2, 0xff8a3a, 2.3, 10, {size: 0.35}); }
  const boat = new THREE.Group(); W.scene.add(boat); boat.position.set(50, 0, 6.8); box(W, M.plank, 3.2, 0.5, 1.3, 0, 0.1, 0, boat); box(W, M.iron, 0.07, 1.2, 0.07, 1.2, 0.8, 0, boat); W.lamp(1.2, 1.45, 0, 0xffb35a, 2.2, 14, {parent: boat, size: 0.25});
  const ff = fireflies(W, 120, V3(-14, 3.9, 13), [26, 2, 10], 0xc8ff5a, 3), ff2 = fireflies(W, 60, V3(16, 2.4, 6), [14, 1.6, 6], 0xc8ff5a, 2);

  // ======================= effects: chips, drops, shell bits; rings on the water =======================
  // lit like the ground (W.std): small cubes, one instanced draw, so they are as dark as the bank at night
  const BITS = (() => {
    const N = 700, im = new THREE.InstancedMesh(new THREE.BoxGeometry(1, 1, 1), W.std({color: 0xffffff, roughness: 0.55}), N); im.frustumCulled = false; W.scene.add(im);
    const L = [], m4 = new THREE.Matrix4(), q = new THREE.Quaternion(), e = new THREE.Euler(), sc = V3(), col = new THREE.Color();
    for (let i = 0; i < N; i++) { L.push({life: 0}); im.setColorAt(i, col.set(0xffffff)); m4.makeScale(0, 0, 0); im.setMatrixAt(i, m4); }
    let next = 0;
    return {spawn(p, v, color, size, life, o = {}) { const bi = next, b = L[next]; next = (next + 1) % N; Object.assign(b, {p: p.clone(), v: v.clone(), size, life, age: 0, rest: false, stick: !!o.stick, water: o.water !== false, spin: V3(Math.random() * 9, Math.random() * 9, Math.random() * 9), rot: V3()}); im.setColorAt(bi, col.set(color)); im.instanceColor.needsUpdate = true; },
      update(dt) { for (let i = 0; i < N; i++) { const b = L[i]; if (b.life <= 0) continue; b.age += dt; if (b.age > b.life) { b.life = 0; m4.makeScale(0, 0, 0); im.setMatrixAt(i, m4); continue; }
          if (!b.rest) { b.v.y -= 9.8 * dt; b.v.multiplyScalar(1 - 0.6 * dt); b.p.addScaledVector(b.v, dt); b.rot.addScaledVector(b.spin, dt);
            const g = groundAt(b.p.x, b.p.z), l = levelAt(b.p.x, b.p.z);
            if (b.water && l > g && b.p.y < l && b.v.y < 0) { if (b.stick) { b.p.y = l + 0.005; b.rest = true; b.v.set(0, 0, 0); } else { b.life = 0; m4.makeScale(0, 0, 0); im.setMatrixAt(i, m4); continue; } }
            else if (b.p.y < g + b.size * 0.5) { b.p.y = g + b.size * 0.5; if (b.stick) { b.rest = true; b.v.set(0, 0, 0); } else { b.life = Math.min(b.life, b.age + 0.05); } } }
          const k = b.age > b.life - 0.6 ? c01((b.life - b.age) / 0.6) : 1; e.set(b.rot.x, b.rot.y, b.rot.z); q.setFromEuler(e); sc.set(b.size, b.size * (b.stick ? 0.45 : 1), b.size).multiplyScalar(k); m4.compose(b.p, q, sc); im.setMatrixAt(i, m4); }
        im.instanceMatrix.needsUpdate = true; }};
  })();
  const RINGS = (() => {
    const L = []; for (let i = 0; i < 24; i++) { const mat = W.lit(new THREE.MeshStandardMaterial({color: 0xeef6f6, transparent: true, opacity: 0, roughness: 0.25, depthWrite: false, side: THREE.DoubleSide}), false); const m = new THREE.Mesh(new THREE.RingGeometry(0.86, 1, 28).rotateX(-Math.PI / 2), mat); m.renderOrder = 7; m.visible = false; W.scene.add(m); L.push({m, mat, age: 0, life: 0, R: 1}); }
    let next = 0;
    return {spawn(x, y, z, R = 1.2, life = 1.6, o = 0.6) { const r = L[next]; next = (next + 1) % L.length; r.m.position.set(x, y + 0.012, z); r.age = 0; r.life = life; r.R = R; r.o = o; r.m.visible = true; },
      update(dt) { for (const r of L) { if (!r.m.visible) continue; r.age += dt; const s = r.age / r.life; if (s >= 1) { r.m.visible = false; continue; } const rr = 0.08 + r.R * easeOut(s); r.m.scale.set(rr, 1, rr); r.mat.opacity = r.o * (1 - s); } }};
  })();
  const DROP = 0xdbe9ee;
  function splash(p, n = 22, power = 1, ring = 1.3) { for (let i = 0; i < n; i++) { const a = Math.random() * 6.28, s = (0.4 + Math.random() * 1.4) * power; BITS.spawn(p, V3(Math.cos(a) * s * 0.6, (1.4 + Math.random() * 2.2) * power, Math.sin(a) * s * 0.6), DROP, 0.025 + Math.random() * 0.02, 1.4); } RINGS.spawn(p.x, p.y, p.z, ring, 1.8, 0.7); RINGS.spawn(p.x, p.y, p.z, ring * 0.55, 1.3, 0.5); }

  // ======================= the rigs =======================
  // A rig is a root on the ground (turned to the heading), a body pivot (hips' height, pitch, roll) with a hind and a
  // chest segment that bend at the spine, a neck and head, four two-bone legs with feet and a three-part tail: bones.
  // Version 3 (the owner, 2026-11-20: "Some weird geometry poking out from middle of them. Also need textures for
  // both. Some fur tuffs would be nice too", then "See mining mike how we did that for the wolf"): each animal is
  // one skin sculpted as a signed distance field round these joints (tools/art/critter_sculpt.py, after Mining Mike's
  // Ice Wolf), meshed, decimated and skinned to the bones, so a leg grows out of the body and nothing pokes out of it
  // or floats beside it; painted as pixel-art fur on its own UVs (ocean/fur-*.png, nearest, square texels, Sweaft-64
  // tones); with fur tufts on the nape, cheeks, back and the base of the tail that ride their bones, lag and settle
  // (overlapping action) and clump into wet spikes while the coat is wet. Lit by W.lit like the ground; no outline.
  // The joints below are the sculpt's (critter_sculpt.py SPECS): buildRig checks the mesh's rest joints against them.
  const SPEC = {
    beaver: {mesh: "beaver", H: 0.17, float: 0.13, legLen: 0.17,
      neck: [0.27, 0.035], jaw: [0.16, -0.045, 0],
      legs: [{at: [0.16, -0.08, 0.085], a: 0.07, b: 0.07, rt: 0.03, front: true, foot: [0.05, 0.04]}, {at: [0.16, -0.08, -0.085], a: 0.07, b: 0.07, rt: 0.03, front: true, foot: [0.05, 0.04]},
        {at: [-0.2, -0.07, 0.1], a: 0.09, b: 0.085, rt: 0.045, foot: [0.11, 0.075]}, {at: [-0.2, -0.07, -0.1], a: 0.09, b: 0.085, rt: 0.045, foot: [0.11, 0.075]}],
      tailAt: [-0.31, -0.01], tail: [{L: 0.07, r: 0.05}, {L: 0.06, r: 0.045}, {L: 0.27, paddle: [0.14, 0.026]}],
      gait: "walk"},
    otter: {mesh: "otter", H: 0.14, float: 0.07, legLen: 0.13,
      neck: [0.245, 0.02], jaw: [0.12, -0.03, 0],
      legs: [{at: [0.17, -0.045, 0.055], a: 0.055, b: 0.05, rt: 0.022, front: true, foot: [0.045, 0.035]}, {at: [0.17, -0.045, -0.055], a: 0.055, b: 0.05, rt: 0.022, front: true, foot: [0.045, 0.035]},
        {at: [-0.22, -0.04, 0.062], a: 0.065, b: 0.06, rt: 0.028, foot: [0.07, 0.045]}, {at: [-0.22, -0.04, -0.062], a: 0.065, b: 0.06, rt: 0.028, foot: [0.07, 0.045]}],
      tailAt: [-0.3, 0.0], tail: [{L: 0.13, r: 0.042}, {L: 0.12, r: 0.03}, {L: 0.11, r: 0.018}],
      gait: "bound"},
    sea: {mesh: "sea", H: 0.0, float: 0.0, legLen: 0.12,
      neck: [0.29, 0.0], jaw: [0.14, -0.04, 0],
      legs: [{at: [0.2, -0.06, 0.085], a: 0.065, b: 0.06, rt: 0.026, front: true, foot: [0.045, 0.04]}, {at: [0.2, -0.06, -0.085], a: 0.065, b: 0.06, rt: 0.026, front: true, foot: [0.045, 0.04]},
        {at: [-0.3, -0.05, 0.08], a: 0.07, b: 0.06, rt: 0.032, foot: [0.13, 0.085]}, {at: [-0.3, -0.05, -0.08], a: 0.07, b: 0.06, rt: 0.032, foot: [0.13, 0.085]}],
      tailAt: [-0.37, 0.0], tail: [{L: 0.09, paddle: [0.075, 0.03]}, {L: 0.09, paddle: [0.07, 0.026]}, {L: 0.09, paddle: [0.055, 0.022]}],
      gait: "walk"},
  };
  const b64 = (s, T) => { const b = atob(s), u = new Uint8Array(b.length); for (let i = 0; i < b.length; i++) u[i] = b.charCodeAt(i); return new T(u.buffer); };
  function skinGeo(B, extra) {
    const g = new THREE.BufferGeometry();
    g.setAttribute("position", new THREE.BufferAttribute(b64(B.pos, Float32Array), 3)); g.setAttribute("normal", new THREE.BufferAttribute(b64(B.nrm, Float32Array), 3));
    g.setAttribute("uv", new THREE.BufferAttribute(b64(B.uv, Float32Array), 2)); g.setIndex(new THREE.BufferAttribute(b64(B.idx, Uint16Array), 1));
    g.setAttribute("skinIndex", new THREE.Uint16BufferAttribute(Array.from(b64(B.si, Uint8Array)), 4)); g.setAttribute("skinWeight", new THREE.Float32BufferAttribute(Array.from(b64(B.sw, Uint8Array), v => v / 255), 4));
    if (extra) { g.setAttribute("wpos", new THREE.BufferAttribute(b64(B.wpos, Float32Array), 3)); g.setAttribute("lk", new THREE.BufferAttribute(b64(B.lk, Float32Array), 4)); }
    return g;
  }
  function buildRig(sp) {
    const D = window.CRITTER_MESH[sp.mesh], map = tex(D.tex, 1); map.wrapS = map.wrapT = THREE.ClampToEdgeWrapping; map.repeat.set(1, 1);
    const mat = W.lit(new THREE.MeshStandardMaterial({map, skinning: true, roughness: 0.88, metalness: 0}), false);
    // the tufts: the same map, both faces, the tip a little lighter than the root (it catches the light), blended to
    // their wet shape (uWet) and swung by the body's and the head's lag (uLag, uLagH: springs, so they overshoot and
    // settle), in the root's frame, after skinning, more toward the tip
    const TU = {uWet: {value: 0}, uLag: {value: V3()}, uLagH: {value: V3()}};
    const tmat = W.lit(new THREE.MeshStandardMaterial({map, skinning: true, roughness: 0.88, metalness: 0, side: THREE.DoubleSide, flatShading: true}), false);
    const lit = tmat.onBeforeCompile;
    tmat.onBeforeCompile = sh => { lit(sh); Object.assign(sh.uniforms, TU);
      sh.vertexShader = sh.vertexShader.replace("#include <common>", "#include <common>\nattribute vec3 wpos; attribute vec4 lk; uniform float uWet; uniform vec3 uLag; uniform vec3 uLagH; varying float vTip;")
        .replace("#include <morphtarget_vertex>", "transformed = mix(transformed, wpos, uWet); vTip = lk.x;\n#include <morphtarget_vertex>")
        .replace("#include <skinning_vertex>", "#include <skinning_vertex>\ntransformed += mix(uLag, uLagH, lk.w) * lk.x * lk.x * (1.0 - 0.6 * uWet);");
      sh.fragmentShader = sh.fragmentShader.replace("#include <common>", "#include <common>\nvarying float vTip;").replace("#include <map_fragment>", "#include <map_fragment>\ndiffuseColor.rgb *= 0.8 + 0.38 * vTip;"); };
    tmat.customProgramCacheKey = () => "pbd-tuft";
    const root = new THREE.Group(); W.scene.add(root);
    const B = {}, bone = (name, parent, p, order) => { const b = new THREE.Bone(); b.name = name; if (p) b.position.set(...p); if (order) b.rotation.order = order; (parent || root).add(b); B[name] = b; return b; };
    const body = bone("body", null, [0, sp.H, 0], "ZYX"), hind = bone("hind", body), chest = bone("chest", body);
    const neck = bone("neck", chest, [sp.neck[0], sp.neck[1], 0], "YZX"), jaw = bone("jaw", neck, sp.jaw);
    const legs = sp.legs.map((L, i) => { const hip = bone("hip" + i, L.front ? chest : hind, L.at), knee = bone("knee" + i, hip, [0, -L.a, 0]), foot = bone("foot" + i, knee, [0, -L.b, 0]); return {L, hip, knee, foot, front: !!L.front}; });
    let parent = hind, at = [sp.tailAt[0], sp.tailAt[1], 0];
    const tail = sp.tail.map((T, i) => { const g = bone("tail" + i, parent, at, "YZX"); parent = g; at = [-T.L, 0, 0]; return g; });
    root.updateMatrixWorld(true);
    // the sculpt was skinned round these joints: a joint that moved since is a bug in one of the two copies
    for (const n of D.bones) { const w = B[n].getWorldPosition(V3()), r = D.rest[n]; if (w.distanceTo(V3(...r)) > 0.001) console.error(`critters: ${sp.mesh} joint ${n} is at ${w.toArray()} on the page, ${r} in critters_mesh.js`); }
    const skel = new THREE.Skeleton(D.bones.map(n => B[n]));
    const skin = new THREE.SkinnedMesh(skinGeo(D.body), mat), tufts = new THREE.SkinnedMesh(skinGeo(D.tufts, true), tmat);
    for (const m of [skin, tufts]) { m.frustumCulled = false; root.add(m); m.bind(skel, new THREE.Matrix4()); }
    return {sp, mat, tmat, TU, root, body, hind, chest, neck, jaw, legs, tail, skin, tufts};
  }
  // ---- poses: plain numbers, blended between actions so a change never pops
  const PK = ["y", "pitch", "roll", "A", "hp", "hy", "hr"];
  const newPose = () => ({y: 0, pitch: 0, roll: 0, A: 0, hp: 0, hy: 0, hr: 0, la: [0, 0, 0, 0], lk: [0, 0, 0, 0], lf: [0, 0, 0, 0], lt: [null, null, null, null], ty: [0, 0, 0], tp: [0, 0, 0]});
  function lerpPose(o, a, b, t) { for (const k of PK) o[k] = lerp(a[k], b[k], t); for (const k of ["la", "lk", "lf"]) for (let i = 0; i < 4; i++) o[k][i] = lerp(a[k][i], b[k][i], t); for (const k of ["ty", "tp"]) for (let i = 0; i < 3; i++) o[k][i] = lerp(a[k][i], b[k][i], t); return o; }
  // two-bone legs to a target: "root" (in the root's frame, the ground at y = 0) or "seg" (from the hip, in its segment)
  // A target may carry c, a curl of the paw (rad) added to the foot's angle: the wrist flexing as the paw lifts and
  // reaching forward as it lands.
  function solveLegs(rig, P) { for (let i = 0; i < 4; i++) solveLeg(rig, P, i); }
  function solveLeg(rig, P, i) {
    { const t = P.lt[i]; if (!t) return; const leg = rig.legs[i], L = leg.L, segP = leg.front ? -P.A : P.A;
      let hx = 0, hy = 0, par = 0;
      if (t.m === "root") { const c1 = Math.cos(segP), s1 = Math.sin(segP), lx = L.at[0] * c1 - L.at[1] * s1, ly = L.at[0] * s1 + L.at[1] * c1, c2 = Math.cos(P.pitch), s2 = Math.sin(P.pitch); hx = lx * c2 - ly * s2; hy = lx * s2 + ly * c2 + P.y; par = P.pitch + segP; }
      const dx = t.x - hx, dy = t.y - hy, a = L.a, b = L.b, d = clamp(Math.hypot(dx, dy), Math.abs(a - b) + 1e-3, (a + b) * 0.999);
      const base = Math.atan2(dx, -dy), cA = Math.acos(clamp((a * a + d * d - b * b) / (2 * a * d), -1, 1)), g = Math.acos(clamp((a * a + b * b - d * d) / (2 * a * b), -1, 1)), s = leg.front ? -1 : 1;
      const aw = base + s * cA, k = -s * (Math.PI - g);
      P.la[i] = aw - par; P.lk[i] = k; P.lf[i] = (t.m === "root" ? -(aw + k) : (t.f ?? -(aw + k) * 0.6)) + (t.c || 0); }
  }
  // ---- the forelegs (version 4). The owner, 2026-11-20: "the arms on beaver and otter should animate when they move or
  // stand up". Version 3 set the forepaws' targets per action and switched them in one frame half way up a rear-up, so
  // the arms jumped, then hung still. These put every arm move on a schedule of its own.
  // a point in a foreleg's segment frame (the chest's, from the shoulder), as a point in the root's frame, for this pose
  function segToRoot(A, P, i, x, y) {
    const L = A.rig.legs[i].L, segP = -P.A, c1 = Math.cos(segP), s1 = Math.sin(segP), lx = L.at[0] * c1 - L.at[1] * s1, ly = L.at[0] * s1 + L.at[1] * c1;
    const c2 = Math.cos(P.pitch), s2 = Math.sin(P.pitch), hx = lx * c2 - ly * s2, hy = lx * s2 + ly * c2 + P.y, par = P.pitch + segP;
    return {x: hx + x * Math.cos(par) - y * Math.sin(par), y: hy + x * Math.sin(par) + y * Math.cos(par)};
  }
  // a rear-up and the sit back down on one clock (dur: when the forepaws are down again; Infinity to stay up):
  // - crouch: the anticipation, the body sinking and the paws pressing before the push;
  // - body: 0 on all fours to 1 reared;
  // - arm: the forepaws leave the ground a beat after the body (they trail, the wrists hanging), catch up on an arc,
  //   carry a little past their mark and settle (overlapping action); going down they lead the body, reaching for the
  //   ground before it arrives;
  // - give: the landing, the elbows bending as the weight comes onto them;
  // - curl: the wrist, flexed as the paw lifts, reaching forward as it comes down.
  // o.from = "up" starts it reared (a step that only sits back down).
  function rear(t, dur, o = {}) {
    const ex = EX(), up0 = o.from === "up", ant = up0 ? 0 : (o.ant ?? tn("shared.forelegs.anticipate_s")), up = up0 ? 1 : (o.up ?? tn("shared.forelegs.rise_s")), dn = o.down ?? tn("shared.forelegs.lower_s"), g = Math.min(0.14, dn * 0.3);
    const t1 = dur - dn, lag = up * 0.3;
    const crouch = up0 ? 0 : (t < ant ? smooth(t / Math.max(ant, 1e-3)) : 1 - smooth(c01((t - ant) / (up * 0.4))));
    const rise = up0 ? 1 : smooth(c01((t - ant) / up)), fall = smooth(c01((t - t1) / (dn - g)));
    const a = up0 ? 1 : c01((t - ant - lag) / (up * 0.9)), s = t - ant - lag - up * 0.9;
    const settle = !up0 && s > 0 ? tn("shared.forelegs.overshoot") * ex * Math.exp(-s * 6) * Math.sin(s * 13) : 0;
    const d = c01((t - t1 + 0.05) / ((dn - g) * 0.75)), arm = (smooth(a) + settle) * (1 - smooth(d));
    const give = dur < 1e8 ? Math.sin(Math.PI * c01((t - (dur - g)) / g)) : 0;
    const curl = (up0 ? 0 : -0.9 * ex * Math.sin(Math.PI * a)) * (1 - d) + 0.5 * ex * Math.sin(Math.PI * d);
    return {crouch, body: rise * (1 - fall), arm, give, curl, d};
  }
  // a forepaw from the ground under its shoulder (where the stand puts it) toward u, a point in the root's frame, k of
  // the way, on an arc that bulges up and forward (never a straight line); c: the curl
  function armTo(A, P, i, u, k, c = 0, bulge = 0.04) {
    const L = A.rig.legs[i].L, gx = L.at[0] + 0.01, gy = A.groundRel(gx, L.at[2]), kk = clamp(k, -0.2, 1.3), b = Math.sin(Math.PI * c01(kk)) * bulge;
    P.lt[i] = {m: "root", x: lerp(gx, u.x, kk) + b * 0.6, y: lerp(gy, u.y, kk) + b, c};
  }
  // blend one leg from whatever P now asks of it toward target tg by g (both solved, the angles mixed: no pop between)
  function blendLeg(A, P, i, tg, g) {
    if (g <= 0) return; if (P.lt[i]) solveLeg(A.rig, P, i); const a = [P.la[i], P.lk[i], P.lf[i]];
    P.lt[i] = tg; solveLeg(A.rig, P, i); P.lt[i] = null; P.la[i] = lerp(a[0], P.la[i], g); P.lk[i] = lerp(a[1], P.lk[i], g); P.lf[i] = lerp(a[2], P.lf[i], g);
  }
  function applyPose(rig, P) {
    rig.body.position.y = P.y; rig.body.rotation.set(P.roll, 0, P.pitch); rig.hind.rotation.z = P.A; rig.chest.rotation.z = -P.A;
    rig.neck.rotation.set(P.hr, P.hy, P.hp);
    rig.legs.forEach((l, i) => { l.hip.rotation.z = P.la[i]; l.knee.rotation.z = P.lk[i]; l.foot.rotation.z = P.lf[i]; });
    rig.tail.forEach((g, i) => g.rotation.set(0, P.ty[i], -P.tp[i]));
  }

  // ======================= an animal: its place, its state machine, its pose =======================
  function Actor(id, rig, o) {
    const A = {id, rig, sp: rig.sp, pos: o.pos.clone(), yaw: o.yaw || 0, v: 0, vy: 0, phi: 0, swimPh: 0, step: null, queue: [], rng: mulberry(SEED * 977 + o.seedK), label: "", stage: "", wet: 0,
      snap: newPose(), cur: newPose(), bt: 1, lag: {hr: 0, ty: [0, 0, 0], tp: [0, 0, 0], yawRate: 0, lastYaw: o.yaw || 0}, where: o.where, held: null, plans: o.plans, brain: o.brain, keyI: 0};
    A.fwd = () => V3(Math.cos(A.yaw), 0, -Math.sin(A.yaw));
    A.toWorld = (lx, ly, lz) => V3(A.pos.x + lx * Math.cos(A.yaw) + lz * Math.sin(A.yaw), A.pos.y + ly, A.pos.z - lx * Math.sin(A.yaw) + lz * Math.cos(A.yaw));
    A.groundRel = (lx, lz) => { const p = A.toWorld(lx, 0, lz); return groundAt(p.x, p.z) - A.pos.y; };
    A.waterAt = () => { const g = groundAt(A.pos.x, A.pos.z), l = levelAt(A.pos.x, A.pos.z); return c01((l - g - A.sp.legLen * 0.55) / 0.22); };
    A.go = (name, now) => { const steps = A.plans[name] && A.plans[name](A); if (!steps) return false; if (now) { if (A.drop) A.drop(A); A.queue = steps; A.setStep(A.queue.shift()); } else A.queue.push(...steps); A.goal = name; return true; };
    A.setStep = s => { A.step = s; if (s) { s.t = 0; if (s.init) s.init(A); } A.snap = JSON.parse(JSON.stringify(A.cur)); A.bt = 0; };
    A.next = () => { if (!A.queue.length) { let g = A.brain(A); if (g === A.lastGoal) g = A.brain(A); A.lastGoal = g; A.go(g); A.goal = g; } A.setStep(A.queue.shift() || Hold(1, "rest")); };
    A.update = (dt, t) => {
      if (!A.step) A.next();
      const s = A.step; let done = false;
      if (SHOWPOSES) { const ks = s.keys ? s.keys(A) : [["held", s.t]]; const k = ks[Math.floor(POSE_CLOCK / 1.2) % ks.length]; A.poseKey = k; }
      else { A.poseKey = null; s.t += dt; done = s.update(A, dt, t); }
      const P = newPose(); const saveT = s.t, savePhi = A.phi, saveSw = A.swimPh;
      if (A.poseKey) { if (A.poseKey[1] != null) s.t = A.poseKey[1]; if (A.poseKey[2] != null) { A.phi = A.poseKey[2]; A.swimPh = A.poseKey[2] * 6.283; } }
      s.pose(A, P, t); solveLegs(A.rig, P);
      A.stage = A.poseKey ? A.poseKey[0] : (s.stage ? s.stage(A) : "");
      s.t = saveT; A.phi = savePhi; A.swimPh = saveSw;
      A.bt = Math.min(1, A.bt + dt / 0.28); if (SHOWPOSES) A.bt = 1;
      lerpPose(A.cur, A.snap, P, smooth(A.bt));
      // overlapping action: the head counters the body's roll a beat late; each tail part follows the hips' roll and
      // the turn a beat behind the one before it
      const out = JSON.parse(JSON.stringify(A.cur)), L = A.lag, hl = A.id === "beaver" ? tn("beaver.walk.head_lag_s") : 0.08, tl = A.id === "beaver" ? tn("beaver.walk.tail_lag_s") : 0.12;
      const yr = angWrap(A.yaw - L.lastYaw) / Math.max(dt, 1e-3); L.lastYaw = A.yaw; L.yawRate += (yr - L.yawRate) * c01(dt * 6);
      const k = c01(dt / hl); L.hr += (-A.cur.roll * 0.6 * (Math.abs(A.cur.roll) < 1.2 ? 1 : 0) - L.hr) * k; out.hr += L.hr;
      let drive = A.cur.roll * (Math.abs(A.cur.roll) < 1.2 ? 1.1 : 0) - L.yawRate * 0.22, driveP = -A.vy * 0.4;
      for (let i = 0; i < 3; i++) { const kk = c01(dt / tl); L.ty[i] += (drive - L.ty[i]) * kk; L.tp[i] += (driveP - L.tp[i]) * kk; out.ty[i] += L.ty[i]; out.tp[i] += L.tp[i]; drive = L.ty[i]; driveP = L.tp[i]; }
      applyPose(A.rig, out);
      A.rig.root.position.copy(A.pos); A.rig.root.rotation.set(0, A.yaw, 0);
      // a wet coat: darker and shinier while it drips, drying on land
      const inW = levelAt(A.pos.x, A.pos.z) > A.pos.y + (A.cur.y || 0) - 0.05; if (inW) A.wet = 1; else A.wet = Math.max(0, A.wet - dt / (A.id === "beaver" ? tn("beaver.shake.dry_s") : 14));
      for (const m of [A.rig.mat, A.rig.tmat]) { m.roughness = lerp(0.88, 0.5, A.wet); m.color.setScalar(lerp(1, 0.68, A.wet)); }
      // wet fur clumps into spikes until the shake (which leaves the coat at 0.45) or until it dries
      A.rig.TU.uWet.value = smooth(c01((A.wet - 0.5) / 0.35)) * tn("shared.fur.wet_spike");
      furLag(A, dt);
      if (done && !SHOWPOSES) A.next();
    };
    return A;
  }
  // the world position of a rig part's point (after the pose has been applied)
  const worldOf = (obj, x = 0, y = 0, z = 0) => { obj.updateWorldMatrix(true, false); return V3(x, y, z).applyMatrix4(obj.matrixWorld); };
  // the tufts' overlapping action: a damped spring follows a point on the back and one on the head; the spring's lag
  // behind the point, turned into the root's frame, swings the tufts' tips, so they trail a move, overshoot when it stops
  // and settle (the shake flings them). Version 4 (the owner, 2026-11-20: "the fur shaking is cool, but a little over the
  // top, I would want more subtle"): version 3's spring was 2.4 Hz at a third of critical damping, its whole lag (up to
  // 3 cm) on the tips; now shared.fur's stiffer, better-damped spring (3.6 Hz, 0.75 of critical: one small overshoot,
  // settled in about a third of a second), 0.4 of the lag on the tips, at most 1.2 cm, and less again in the shake
  function furLag(A, dt) {
    const R = A.rig, pts = [worldOf(R.body, -0.05, 0.12, 0), worldOf(R.neck, 0.0, 0.06, 0)], outs = [R.TU.uLag.value, R.TU.uLagH.value];
    if (!R.spr) R.spr = pts.map(p => ({p: p.clone(), v: V3()}));
    pts.forEach((p, i) => { const s = R.spr[i];
      const w = 2 * Math.PI * tn("shared.fur.spring_hz"), kS = w * w, cS = 2 * tn("shared.fur.damping") * w;
      if (dt > 0) { const n = Math.max(1, Math.ceil(dt / 0.005)), h = dt / n; for (let k = 0; k < n; k++) { const a = p.clone().sub(s.p).multiplyScalar(kS).addScaledVector(s.v, -cS); s.v.addScaledVector(a, h); s.p.addScaledVector(s.v, h); } }
      if (s.p.distanceTo(p) > 0.5) { s.p.copy(p); s.v.set(0, 0, 0); }
      const shaking = A.step && A.step.name === "shake off", lag = s.p.clone().sub(p).multiplyScalar(tn("shared.fur.sway") * (shaking ? tn("shared.fur.shake") : 1)), mx = tn("shared.fur.max_m"); if (lag.length() > mx) lag.setLength(mx);
      const c = Math.cos(A.yaw), sn = Math.sin(A.yaw); outs[i].set(lag.x * c - lag.z * sn, lag.y, lag.x * sn + lag.z * c); });
  }

  // ---- shared poses
  // a gait: feet in stance stay put (their target moves back under the body at the body's speed), in swing they lift on
  // an arc and land half a reach ahead; the ground under each foot is sampled
  function gaitPose(A, P, G) {
    const reach = G.stance * G.stride, ex = EX();
    for (let i = 0; i < 4; i++) { const L = A.rig.legs[i].L, u = frac(A.phi + G.offs[i]); let x, y, c = 0;
      if (u < G.stance) { const s = u / G.stance; x = reach / 2 - reach * s; y = 0; } else { const s = (u - G.stance) / (1 - G.stance); x = -reach / 2 + reach * smooth(s); y = G.lift * Math.sin(Math.PI * s);
        // a forepaw in its swing (version 4): the wrist flexes as it peels off the ground, then opens to reach for the
        // next contact, the elbow bending with it (the leg is solved to the arc)
        if (L.front) c = -tn("shared.forelegs.swing_curl_rad") * ex * Math.sin(Math.PI * s) * (1.2 - s); }
      const fx = L.at[0] + x; P.lt[i] = {m: "root", x: fx, y: A.groundRel(fx, L.at[2]) + y, c}; }
    const slope = Math.atan2(A.groundRel(0.25, 0) - A.groundRel(-0.25, 0), 0.5);
    P.pitch = slope; P.y = A.sp.H;
    if (G.kind === "walk") { P.y += G.bob * ex * Math.cos(4 * Math.PI * A.phi); P.roll = G.roll * ex * Math.sin(2 * Math.PI * A.phi); P.hp = -slope * 0.6 + 0.04 * Math.sin(4 * Math.PI * A.phi - 1); }
    else { const arch = G.arch * ex * (0.55 * Math.cos(2 * Math.PI * (A.phi - 0.95)) + 0.25); P.A = arch; P.y += G.lift * ex * Math.max(0, Math.cos(4 * Math.PI * (A.phi - 0.44))); P.pitch += 0.07 * ex * Math.sin(2 * Math.PI * (A.phi - 0.2)); P.hp = -P.pitch * 0.8 - 0.05; }
    for (let i = 0; i < 3; i++) P.tp[i] = G.kind === "walk" ? TAIL_DRAG[i] : 0.12 - i * 0.05;
  }
  const GAIT = {
    beaver: () => ({kind: "walk", stride: tn("beaver.walk.stride_m"), stance: tn("beaver.walk.stance"), lift: tn("beaver.walk.foot_lift_m"), offs: [-0.25, -0.75, 0, -0.5], bob: tn("beaver.walk.hip_bob_m"), roll: tn("beaver.walk.waddle_roll_rad")}),
    otter: () => ({kind: "bound", stride: tn("river_otter.bound.stride_m"), stance: tn("river_otter.bound.stance"), lift: tn("river_otter.bound.lift_m"), offs: [-0.5, -0.56, 0, -0.06], arch: tn("river_otter.bound.arch_rad")}),
  };
  const TAIL_DRAG = [-0.95, 0.6, 0.32];   // the beaver's tail: down from the hips, the paddle flat on the ground
  const WALK_KEYS = [["contact", null, 0], ["down", null, 0.12], ["passing", null, 0.25], ["up", null, 0.37]];
  // standing still: each foot under its hip
  function standPose(A, P) { for (let i = 0; i < 4; i++) { const L = A.rig.legs[i].L; P.lt[i] = {m: "root", x: L.at[0], y: A.groundRel(L.at[0], L.at[2])}; } P.pitch = Math.atan2(A.groundRel(0.25, 0) - A.groundRel(-0.25, 0), 0.5); P.y = A.sp.H; for (let i = 0; i < 3; i++) P.tp[i] = A.id === "beaver" ? TAIL_DRAG[i] : 0.12 - i * 0.05; }
  // swimming at the surface: legs paddling (hind) and tucked (fore), the tail sculling, each part a beat behind
  function swimPose(A, P, deep = 0) {
    const ph = A.swimPh, ex = EX(), beaver = A.id === "beaver";
    P.y = -A.sp.float; P.pitch = (beaver ? 0.07 : 0.04) + Math.atan2(A.vy, Math.max(0.25, A.v)) * 0.9;
    P.hp = deep ? -0.05 : (beaver ? 0.32 + tn("beaver.swim.head_above_m") * 2 : 0.38);
    for (let i = 0; i < 4; i++) { const fr = i < 2, side = i % 2 ? Math.PI : 0;
      // the forelegs (version 4): a river otter dog-paddles with them while it swims slowly or treads water (alternate
      // strokes: reach forward, the paw open, pull back, the elbow folding on the recovery) and lays them back along
      // its body when it swims fast or dives; a beaver (and a sea otter belly down) swims on its hind feet alone, its
      // forepaws held loosely to its chest, nodding with each kick
      if (fr && A.id === "otter") { const pa = tn("shared.forelegs.paddle_rad") * ex, fast = deep ? 1 : c01((A.v - 0.5) / 0.8), q = ph + side;
        P.la[i] = lerp(-0.3 + pa * Math.sin(q), -1.3, fast); P.lk[i] = lerp(0.5 + 0.8 * Math.max(0, Math.cos(q)), 1.15, fast); P.lf[i] = lerp(-0.15 + 0.55 * Math.cos(q), -0.45, fast); }
      else if (fr) { const bob = 0.1 * ex * Math.sin(ph + 0.8 + i * 0.3); P.la[i] = -1.15 + bob; P.lk[i] = 1.3 - bob * 0.8; P.lf[i] = -0.3 - 0.25 * Math.sin(ph + 1.6); }
      else { P.la[i] = -0.55 + 0.55 * Math.sin(ph * 1.0 + side); P.lk[i] = -0.6 - 0.45 * Math.sin(ph + side + 1.2); P.lf[i] = 0.4 + 0.5 * Math.sin(ph + side + 2.0); } }
    const amp = (beaver ? tn("beaver.swim.scull_rad") : 0.28) * ex * c01(A.v / 0.3 + 0.25);
    for (let i = 0; i < 3; i++) { if (beaver) { P.ty[i] = amp * Math.sin(ph - i * 0.9) * (i === 2 ? 0.6 : 1); P.tp[i] = 0.02; } else { P.tp[i] = amp * 0.6 * Math.sin(ph - i * 0.9); P.ty[i] = 0.1 * Math.sin(ph * 0.5 - i); } }
    if (!beaver) P.A = 0.12 * ex * Math.sin(ph);
  }
  // ---- steps: each has update (moves the animal, returns true when done), pose (fills P from its time t and the
  // animal's phase), keys (the keyed poses "Show poses" freezes on) and stage (its name in the HUD)
  // a move to a point: walks or bounds on land, swims in water (under it, `depth` metres), turning on the way
  function Move(target, o = {}) {
    return {name: o.name || "move",
      update(A, dt) {
        const tg = typeof target === "function" ? target(A) : target, dx = tg.x - A.pos.x, dz = tg.z - A.pos.z, dist = Math.hypot(dx, dz);
        const want = Math.atan2(-dz, dx), err = angWrap(want - A.yaw), turn = (A.id === "otter" ? 5 : 3) * dt; A.yaw += clamp(err, -turn, turn);
        const sw = A.waterAt(), walk = A.id === "beaver" ? tn("beaver.walk.speed_mps") : A.id === "otter" ? tn("river_otter.bound.speed_mps") : 0.5, swim = A.id === "beaver" ? tn("beaver.swim.speed_mps") : 1.1;
        const vmax = lerp(walk, swim, sw) * (o.speed || 1), vw = vmax * clamp(dist / 0.5, 0.3, 1) * clamp(Math.cos(err), 0.12, 1);
        A.v += (vw - A.v) * c01(dt * 3);
        const stepL = Math.min(A.v * dt, dist); A.pos.addScaledVector(A.fwd(), stepL);
        const G = A.id === "beaver" ? GAIT.beaver() : GAIT.otter(); A.phi += stepL / G.stride * (1 - sw);
        A.swimPh += dt * 2 * Math.PI * (A.id === "beaver" ? tn("beaver.swim.scull_hz") : 1.8) * (0.4 + 0.6 * sw);
        const g = groundAt(A.pos.x, A.pos.z), l = levelAt(A.pos.x, A.pos.z), yW = l - (o.depth || 0), yT = sw > 0 ? lerp(g, yW, sw) : g;
        const y0 = A.pos.y; A.pos.y = sw > 0.01 ? A.pos.y + (yT - A.pos.y) * c01(dt * (o.depth ? 2 : 5)) : yT; if (A.pos.y < g) A.pos.y = g; A.vy += ((A.pos.y - y0) / Math.max(dt, 1e-3) - A.vy) * c01(dt * 5);
        this.sw = sw; if (o.each) o.each(A, dt);
        return dist < (o.tol || 0.1);
      },
      pose(A, P) { const sw = this.sw ?? A.waterAt(); const a = newPose(), b = newPose();
        if (sw < 1) { gaitPose(A, a, A.id === "beaver" ? GAIT.beaver() : GAIT.otter()); solveLegs(A.rig, a); }
        if (sw > 0) swimPose(A, b, o.depth);
        lerpPose(P, a, b, sw); P.lt = [null, null, null, null];
        if (o.posePlus) o.posePlus(A, P); },
      keys: () => WALK_KEYS, stage(A) { return o.name || ((this.sw ?? 0) > 0.5 ? "swim" : A.id === "otter" ? "bound" : "walk"); }};
  }
  // a timed action: pose(A, P, t) of its own time, events at their times
  function Act(name, dur, poseFn, o = {}) {
    return {name, ev: (o.events || []).map(e => ({at: e[0], fn: e[1], done: false})),
      init(A) { this.dur = typeof dur === "function" ? dur(A) : dur; this.ev.forEach(e => e.done = false); if (o.init) o.init(A, this); },
      update(A, dt) { if (o.move) o.move(A, dt, this.t, this); for (const e of this.ev) { const at = typeof e.at === "function" ? e.at(A) : e.at; if (!e.done && this.t >= at) { e.done = true; e.fn(A, this); } } return this.t >= this.dur; },
      pose(A, P) { poseFn(A, P, Math.min(this.t, this.dur ?? 1e9), this); }, keys: o.keys ? (A => o.keys(A)) : null, stage(A) { return o.stage ? o.stage(A, this.t) : name; }};
  }
  // patch Idle into an Act (simpler: an Idle is an Act that holds a pose)
  const Hold = (dur, name, poseFn) => Act(name, dur, (A, P, t) => { if (A.waterAt() > 0.5) { A.swimPh += 0.02; swimPose(A, P); } else standPose(A, P); if (poseFn) poseFn(A, P, t); });
  function Face(pt) { return {name: "turn", update(A, dt) { const want = Math.atan2(-(pt.z - A.pos.z), pt.x - A.pos.x), err = angWrap(want - A.yaw); const tr = 2.5 * dt; A.yaw += clamp(err, -tr, tr); A.phi += Math.abs(clamp(err, -tr, tr)) * 0.12 / 0.36; return Math.abs(err) < 0.04; },
    pose(A, P) { if (A.waterAt() > 0.5) swimPose(A, P); else { const G = A.id === "beaver" ? GAIT.beaver() : GAIT.otter(); gaitPose(A, P, Object.assign(G, {lift: G.lift * 0.6})); } }, keys: () => WALK_KEYS, stage: () => "turn"}; }

  // ======================= the beaver =======================
  // aspens it fells: a stump, a waist that narrows as it is gnawed, and the trunk on a pivot that tips toward the pond
  const WOOD = W.std({color: 0xe8d2a0, roughness: 0.9});   // gnawed wood: the notch and the stump's top
  const FELL = [V3(-14.6, 3, 12.0), V3(-8.8, 3, 12.6), V3(-19.6, 3, 12.4)].map((p, i) => {
    const dir = V3(i === 1 ? 0.35 : i === 2 ? -0.35 : 0.1, 0, -1).normalize(), g = new THREE.Group(); g.position.copy(p); W.scene.add(g);
    const R = 0.17, stump = prism(W, aspenBark, R, 0.3, 0, 0, 0, g), waist = prism(W, aspenBark, R, 0.2, 0, 0.3, 0, g), cut = prism(W, WOOD, R * 0.95, 0.02, 0, 0.3, 0, g); cut.visible = false;
    const pivot = new THREE.Group(); pivot.position.y = 0.5; g.add(pivot); prism(W, aspenBark, R, 4.2, 0, 0, 0, pivot);
    const crown = new THREE.Group(); pivot.add(crown); for (const [rr, h, yy] of [[1.15, 1.3, 2.6], [0.9, 1.15, 3.55], [0.5, 1.0, 4.4]]) prism(W, aspenLeaf, rr, h, 0, yy, 0, crown);
    const axis = V3(0, 1, 0).cross(dir).normalize(), side = V3().crossVectors(dir, V3(0, 1, 0)).normalize();
    const T = {i, p, dir, axis, side, g, pivot, waist, cut, crown, stumpTop: 0.3, R, state: "standing", bites: 0, ang: 0, ft: 0, branches: [], hitDone: false};
    T.makeBranches = () => { T.branches.forEach(b => { if (b.on === "tree") { pivot.remove(b.g); } }); T.branches = [0, 1, 2].map(j => { const b = makeBranch(); b.on = "tree"; pivot.add(b.g); const sgn = j % 2 ? 1 : -1; b.g.position.set(0, 2.3 + j * 0.55, 0); b.g.quaternion.setFromUnitVectors(V3(1, 0, 0), side.clone().multiplyScalar(sgn * 0.8).addScaledVector(dir, 0.45).add(V3(0, 0.2, 0)).normalize()); return b; }); };
    T.makeBranches();
    return T;
  });
  function makeBranch() { const K = C3.Kit(), L = 1.15; K.add(C3.cyl(0.014, 0.024, L, 5).rotateZ(-Math.PI / 2).translate(L / 2, 0, 0), 0xab8465); for (const [x, a] of [[0.45, 0.6], [0.7, -0.7], [0.9, 0.5]]) K.add(C3.cyl(0.006, 0.01, 0.3, 4).rotateZ(-Math.PI / 2).translate(0.15, 0, 0), 0x8d6a4a, {p: [x, 0, 0], r: [0, a, 0.2]});
    for (let i = 0; i < 9; i++) K.add(C3.bx(0.1, 0.03, 0.08), i % 3 ? 0x9bb675 : 0x559642, {p: [0.55 + i * 0.07, 0.03 + (i % 2) * 0.04, (hsh(i, 3) - 0.5) * 0.3], r: [0.3 * i, i, 0]});
    const g = new THREE.Mesh(K.build(), W.lit(new THREE.MeshStandardMaterial({vertexColors: true, flatShading: true, side: THREE.DoubleSide, roughness: 0.9}), false)); return {g, L}; }
  // the tree's own clock once the bites are done: creak (it leans back over the cut and shudders), fall (an angle that
  // grows as t squared: slow at first, fastest at the hit), the hit (the crown shakes, leaves fly), bounce, settle
  const FALL_TO = 1.5;
  function treeUpdate(T, dt) {
    const cs = tn("beaver.fell.creak_s"), fs = tn("beaver.fell.fall_s"), ss = tn("beaver.fell.settle_s"), ex = EX();
    const nk = 1 - 0.72 * c01(T.bites / tn("beaver.gnaw.bites_to_fell")); T.waist.scale.set(nk, 1, nk); T.waist.material = T.bites > 0 && T.state !== "regrow" ? WOOD : aspenBark;
    if (T.state === "falling") { T.ft += dt; const t = T.ft;
      if (t < cs) T.ang = -tn("beaver.fell.lean_back_rad") * ex * smooth(t / cs) + 0.008 * Math.sin(t * 90) * (t / cs);
      else if (t < cs + fs) { const s = (t - cs) / fs; T.ang = lerp(-tn("beaver.fell.lean_back_rad") * ex, FALL_TO, s * s); }
      else { if (!T.hitDone) { T.hitDone = true; const tip = worldOf(T.pivot, 0, 3.4, 0); for (let i = 0; i < 26; i++) BITS.spawn(tip.clone().add(V3((Math.random() - 0.5) * 1.6, 0.3, (Math.random() - 0.5) * 1.6)), V3((Math.random() - 0.5) * 2, 1 + Math.random() * 2.2, (Math.random() - 0.5) * 2), Math.random() < 0.5 ? 0x9bb675 : 0xe3ecaa, 0.05, 6, {stick: true}); if (levelAt(tip.x, tip.z) > -100) splash(V3(tip.x, P_LVL, tip.z), 16, 0.8, 1.6); }
        const s = (t - cs - fs) / ss; T.ang = FALL_TO - tn("beaver.fell.bounce_rad") * ex * Math.abs(Math.sin(Math.PI * 1.5 * Math.min(1, s))) * (1 - c01(s)) ** 2;
        T.crown.rotation.set(0.05 * Math.sin(t * 40) * (1 - c01(s)), 0, 0.04 * Math.cos(t * 33) * (1 - c01(s)));
        if (s >= 1) { T.state = "down"; T.ang = FALL_TO; T.crown.rotation.set(0, 0, 0); } } }
    else if (T.state === "regrow") { T.ft += dt; const s = c01(T.ft / 3); T.pivot.scale.setScalar(Math.max(0.01, smooth(s))); T.waist.scale.set(1, 1, 1); if (s >= 1) T.state = "standing"; }
    T.pivot.quaternion.setFromAxisAngle(T.axis, T.ang); T.waist.visible = T.state === "standing" || T.state === "regrow" || (T.state === "falling" && T.ft < cs); T.cut.visible = !T.waist.visible;
  }
  const DAM_SLOTS = [];
  for (const [k, c] of cells) if (c.dam && !c.spill) { const [q, r] = k.split(",").map(Number), x = hx(q, r), z = hz(r); const w = cells.get((q - 1) + "," + r); if (w && w.pond) DAM_SLOTS.push({x, z, n: 0, spot: V3(x - HW / 2 - 0.42, 0, z)}); }
  const placed = [];
  const RAMP_TOP = V3(-12, 0, 10.2), RAMP_BOT = V3(-12, 0, 6.3), WATER_IN = V3(-12, 0, 5.0), POND_MID = V3(-9.5, 0, 0.5);
  const beaverRig = buildRig(SPEC.beaver);
  const beaver = Actor("beaver", beaverRig, {pos: V3(-12.5, 3, 13.5), yaw: 0.4, seedK: 1, where: "land", plans: {}, brain: null});
  const branchWorld = (A, b) => { // the held branch hangs from the jaw and trails behind, its far end on the ground or the water
    const j = worldOf(A.rig.jaw), back = A.fwd().negate(), lv = levelAt(j.x, j.z), far0 = j.clone().addScaledVector(back, b.L * 0.9), g = Math.max(groundAt(far0.x, far0.z), lv > -100 ? lv : -99) + 0.04;
    const dy = clamp(j.y - g, -b.L * 0.95, b.L * 0.95), hd = Math.sqrt(b.L * b.L - dy * dy), F = j.clone().addScaledVector(back, hd); F.y = j.y - dy;
    F.add(A.rig.root.localToWorld(V3(0, 0, 0)).sub(A.pos).multiplyScalar(0)); const side = V3(-back.z, 0, back.x).multiplyScalar(Math.sin(A.phi * 6.283) * 0.08 + (lv > j.y - 0.3 ? 0.45 : 0)); F.add(side);
    b.g.position.copy(j); b.g.quaternion.setFromUnitVectors(V3(1, 0, 0), F.sub(j).normalize()); };
  // gnawing: reared on the hind legs and the tail (a tripod), forepaws on the trunk; each bite four stages on one beat
  const gnawD = () => { const b = tn("beaver.gnaw.beat_s"); return [b * 0.3, b * 0.2, b * 0.12, b * 0.38]; };
  // version 4: the rear-up before the first bite (gnawRise seconds): the crouch, the push up onto the hind legs and the
  // tail, the forepaws trailing up onto the trunk and settling there; on each bite the paws grip and pull in a little
  const gnawRise = () => tn("shared.forelegs.anticipate_s") + tn("shared.forelegs.rise_s") * 1.25;
  const GNAW_UP = [0.95, 0.12, -0.05];
  function gnawPose(A, P, t, low, r) {
    const d = gnawD(), beat = d.reduce((a, b) => a + b), pull = tn("beaver.gnaw.pull_rad") * EX(), R0 = low ? 0 : gnawRise(), tb = t - R0;
    const S = tb >= 0 ? strike(tb % beat, d, pull, pull * 1.2, -0.14, 0.1) : {v: 0, st: "rear up"};
    standPose(A, P);
    if (!low) { r = r || rear(t, Infinity); const u = r.body, grip = 0.014 * c01(S.v / pull);
      P.pitch = lerp(P.pitch, 0.92, u) - 0.08 * r.crouch; P.y = A.sp.H + 0.06 * u - 0.02 * r.crouch - 0.012 * r.give; P.hp = lerp(-0.15, -1.02 + S.v, u) + 0.15 * r.crouch; P.hr = 0.35 * c01(S.v / pull) * u;
      const hx = lerp(-0.2, -0.06, u); P.lt[2] = {m: "root", x: hx, y: A.groundRel(hx, 0.1)}; P.lt[3] = {m: "root", x: hx, y: A.groundRel(hx, -0.1)};
      armTo(A, P, 0, {x: 0.27 - grip, y: 0.36 + 0.015 * Math.sin(t * 3)}, r.arm, r.curl, 0.06); armTo(A, P, 1, {x: 0.27 - grip * 0.7, y: 0.33 - 0.006 * c01(S.v / pull)}, r.arm, r.curl, 0.06);
      for (let i = 0; i < 3; i++) P.tp[i] = lerp(TAIL_DRAG[i], GNAW_UP[i], u); }
    else { P.pitch = -0.12; P.y = A.sp.H - 0.02; P.hp = -0.55 + S.v * 0.8; P.hr = 0.3 * c01(S.v / pull);
      // gnawing a branch on the ground: the forepaws hold it, pressing on each bite
      for (const i of [0, 1]) { const L = A.rig.legs[i].L, x = L.at[0] + 0.05 + 0.012 * c01(-S.v / 0.14); P.lt[i] = {m: "root", x, y: A.groundRel(x, L.at[2]) + 0.012, c: -0.3 - 0.25 * c01(S.v / pull)}; } }
    return S;
  }
  function Gnaw(T, n, low, onHit) {
    const R0 = () => low ? 0 : gnawRise();
    return Act(low ? "gnaw a branch" : "gnaw", () => { const d = gnawD(); return R0() + d.reduce((a, b) => a + b) * n; }, (A, P, t) => { gnawPose(A, P, t, low); }, {
      init(A, s) { s.lastBeat = -1; },
      move(A, dt, t0, s) { const t = t0 - R0(); if (t < 0) return; const d = gnawD(), beat = d.reduce((a, b) => a + b), k = Math.floor(t / beat), into = t - k * beat; if (into >= d[0] + d[1] + d[2] && s.lastBeat < k) { s.lastBeat = k; onHit(A, k); } },
      keys: () => low ? strikeKeys(gnawD()) : [["anticipation", tn("shared.forelegs.anticipate_s")], ["rear up", tn("shared.forelegs.anticipate_s") + tn("shared.forelegs.rise_s") * 0.55], ["paws on the trunk", R0()], ...strikeKeys(gnawD(), R0())],
      stage: (A, t) => { const d = gnawD(); if (t < R0()) return t < tn("shared.forelegs.anticipate_s") ? "gnaw: anticipation (crouch)" : "gnaw: rear up"; return (low ? "gnaw a branch: " : "gnaw: ") + strike((t - R0()) % d.reduce((a, b) => a + b), d, 1, 1, 0, 0).st; }});
  }
  const chips = (p, toward) => { for (let i = 0; i < tn("beaver.gnaw.chips_per_bite"); i++) { const v = toward.clone().multiplyScalar(0.6 + Math.random() * 0.8).add(V3((Math.random() - 0.5) * 1.2, 0.8 + Math.random() * 1.2, (Math.random() - 0.5) * 1.2)); BITS.spawn(p, v, Math.random() < 0.7 ? 0xe8d2a0 : 0xcfaa81, 0.035 + Math.random() * 0.025, 9, {stick: true}); } };
  beaver.plans = {
    fell(A) { const T = FELL.find(T => T.state === "standing"); if (!T) return null; A.target = T;
      const spot = T.p.clone().addScaledVector(T.side, 0.45), back = T.p.clone().addScaledVector(T.side, 1.7).addScaledVector(T.dir, -0.9);
      const steps = [];
      if (A.waterAt() > 0.3) steps.push(...climbOut());
      steps.push(Move(spot.clone().addScaledVector(T.side, 0.5), {tol: 0.12}), Move(spot, {tol: 0.05, speed: 0.5}), Face(T.p),
        Gnaw(T, Math.max(1, tn("beaver.gnaw.bites_to_fell") - T.bites), false, (A2, k) => { T.bites++; chips(worldOf(T.g, 0, 0.4, 0).addScaledVector(T.side, T.R), T.side.clone()); }),
        Act("the tree creaks: drop to all fours", () => tn("shared.forelegs.lower_s") + 0.05, (A2, P, t) => { const dur = tn("shared.forelegs.lower_s") + 0.05, r = rear(t, dur, {from: "up"}); gnawPose(A2, P, gnawRise(), false, r); P.hp = lerp(-0.15, -0.7, r.body); P.hy = 0.25 * Math.sin(t * 8) * r.body; },
          {init() { T.state = "falling"; T.ft = 0; T.hitDone = false; }, keys: () => [["paws reach down", tn("shared.forelegs.lower_s") * 0.4], ["down", tn("shared.forelegs.lower_s") + 0.05]]}),
        Move(back, {name: "scramble clear", speed: 2.0, tol: 0.15}),
        Hold(2.6, "watch it fall", (A2, P, t) => { const tp = T.p.clone().addScaledVector(T.dir, 2.5); const want = Math.atan2(-(tp.z - A2.pos.z), tp.x - A2.pos.x), r = rear(t, 2.6); P.hy = clamp(angWrap(want - A2.yaw), -0.9, 0.9) * c01(t * 2);
          P.pitch = lerp(P.pitch, 0.35, r.body) - 0.05 * r.crouch; P.y = A2.sp.H + 0.03 * r.body - 0.015 * r.crouch - 0.01 * r.give; armTo(A2, P, 0, {x: 0.15, y: 0.13}, r.arm, r.curl); armTo(A2, P, 1, {x: 0.15, y: 0.11}, r.arm, r.curl); P.hp = -0.2 * r.body; }));
      return steps; },
    fetch(A) { const T = FELL.find(T => T.state === "down" && T.branches.some(b => b.on === "tree")); if (!T) return null; const b = T.branches.find(b => b.on === "tree");
      let base = worldOf(b.g, 0.1, 0, 0), bd = 9; for (let x = 0.05; x < b.L * 0.7; x += 0.05) { const p = worldOf(b.g, x, 0, 0), e = Math.abs(p.y - groundAt(p.x, p.z) - 0.2); if (e < bd) { bd = e; base = p; } }
      const along = worldOf(b.g, 0.9, 0, 0).sub(worldOf(b.g, 0.1, 0, 0)).setY(0).normalize(), perp = V3(-along.z, 0, along.x), spot = base.clone().addScaledVector(perp, base.clone().addScaledVector(perp, 0.42).distanceTo(T.p) > base.clone().addScaledVector(perp, -0.42).distanceTo(T.p) ? 0.42 : -0.42);
      const slot = DAM_SLOTS[Math.floor(A.rng() * DAM_SLOTS.length)];
      const steps = [];
      if (A.waterAt() > 0.3) steps.push(...climbOut());
      steps.push(Move(spot, {tol: 0.08}), Face(base),
        Gnaw(T, 3, true, (A2) => chips(base.clone(), spot.clone().sub(base).setY(0).normalize())),
        Act("take the branch", 0.45, (A2, P, t) => { standPose(A2, P); P.hp = -0.6 + 0.5 * smooth(c01(t / 0.45)); P.pitch = -0.1 * (1 - t / 0.45); }, {init(A2) { T.pivot.remove(b.g); W.scene.add(b.g); b.on = "mouth"; A2.held = b; }}),
        Move(RAMP_TOP, {name: "drag the branch"}), Move(RAMP_BOT, {name: "into the water", speed: 1.1}), Move(WATER_IN, {name: "swim with the branch"}),
        Move(slot.spot, {name: "swim to the dam", tol: 0.12}), Face(V3(slot.x, 0, slot.z)),
        Act("add it to the dam", () => tn("beaver.slap.windup_s") + 0.25 + 0.12 + 0.45, (A2, P, t) => { swimPose(A2, P); const d = [tn("beaver.slap.windup_s"), 0.25, 0.12, 0.45], S = strike(t, d, -0.25, -0.32, 0.75, -0.1); P.pitch = 0.1 + Math.max(0, S.v); P.y = -A2.sp.float + Math.max(0, S.v) * 0.12; P.hp = 0.2 - S.v * 0.35; P.la[0] = P.la[1] = -0.2 + S.v * 1.4; P.lk[0] = P.lk[1] = 0.6; },
          {events: [[() => tn("beaver.slap.windup_s") + 0.25 + 0.06, (A2) => { const k = slot.n++; const bb = A2.held; A2.held = null; bb.on = "dam"; bb.g.position.set(slot.x + (A2.rng() - 0.5) * 0.6, 3.04 + (k % 4) * 0.05, slot.z + (A2.rng() - 0.5) * 0.5); bb.g.rotation.set(0, Math.PI / 2 + (A2.rng() - 0.5) * 0.9, 0.04); placed.push(bb); if (placed.length > 9) { const old = placed.shift(); W.scene.remove(old.g); } splash(V3(A2.pos.x, P_LVL, A2.pos.z), 10, 0.5, 0.8); }]],
            keys: () => strikeKeys([tn("beaver.slap.windup_s"), 0.25, 0.12, 0.45]), stage: (A2, t) => "push: " + strike(t, [tn("beaver.slap.windup_s"), 0.25, 0.12, 0.45], 1, 1, 0, 0).st}));
      A.where = "water"; return steps; },
    slap(A) { const steps = []; if (A.waterAt() < 0.5) steps.push(Move(RAMP_TOP), Move(RAMP_BOT, {speed: 1.1}), Move(WATER_IN)); steps.push(Move(POND_MID.clone().add(V3((A.rng() - 0.5) * 3, 0, (A.rng() - 0.5) * 2)), {name: "swim out"}), slapAct(), Move(V3(-12, 0, 3.0), {name: "under water", depth: 0.75}), Move(WATER_IN, {name: "surface"}), ...climbOut(), shakeAct()); A.where = "land"; return steps; },
    shake(A) { const steps = []; if (A.waterAt() > 0.3) steps.push(...climbOut()); steps.push(shakeAct()); A.where = "land"; return steps; },
    sit(A) { const steps = []; if (A.waterAt() > 0.3) steps.push(...climbOut()); steps.push(sitAct()); return steps; },
    regrow() { FELL.forEach(T => { if (T.state === "down") { T.state = "regrow"; T.ft = 0; T.ang = 0; T.bites = 0; T.makeBranches(); } }); return [Hold(3, "the aspens grow back (mockup only)")]; },
  };
  function climbOut() { return [Move(WATER_IN, {name: "swim to the bank"}), Move(RAMP_BOT, {name: "climb out", speed: 1.1}), Move(RAMP_TOP, {name: "climb out"})]; }
  // the tail slap: lift (windup), held high (anticipation), down flat (the hit: splash and rings), bounce (recoil) as
  // the head goes under; the body pitches nose down and the tail goes last
  function slapD() { return [tn("beaver.slap.windup_s"), tn("beaver.slap.anticipate_s"), tn("beaver.slap.strike_s"), tn("beaver.slap.recoil_s")]; }
  function slapAct() { return Act("tail slap", () => slapD().reduce((a, b) => a + b) + 0.5, (A, P, t) => {
      swimPose(A, P); const d = slapD(), r = tn("beaver.slap.raise_rad") * EX(), S = strike(t, d, r, r * 1.12, -0.25, 0.3), after = c01((t - d[0] - d[1] - d[2]) / (d[3] + 0.5));
      P.tp = [S.v * 0.55, S.v * 0.2, S.v * 0.06]; P.ty = [0, 0, 0]; P.pitch = -0.12 * c01(S.v / r) - 0.85 * smooth(after); P.y = -A.sp.float - 0.03 * c01(S.v / r) - 0.35 * after; P.hp = 0.3 - 0.6 * after; },
    {events: [[() => { const d = slapD(); return d[0] + d[1] + d[2]; }, (A) => { const p = worldOf(A.rig.tail[2], -0.2, 0, 0); p.y = P_LVL; splash(p, 34, 1.3, 1.8); }]], keys: () => strikeKeys(slapD()), stage: (A, t) => "tail slap: " + strike(t, slapD(), 1, 1, 0, 0).st}); }
  // shaking off: a short crouch, then a fast roll on a decaying wave, the head a beat late and the tail last; drops fly
  function shakeAct() { return Act("shake off", () => 0.2 + tn("beaver.shake.time_s"), (A, P, t) => { standPose(A, P); const T0 = 0.2, T = tn("beaver.shake.time_s");
      if (t < T0) { P.y = A.sp.H - 0.03 * smooth(t / T0); return; } const s = (t - T0) / T, env = Math.sin(Math.PI * Math.min(1, s)) ** 0.6; P.roll = tn("beaver.shake.roll_rad") * EX() * env * Math.sin(2 * Math.PI * tn("beaver.shake.hz") * (t - T0)); P.y = A.sp.H - 0.015; P.hp = 0.1; },
    {move(A, dt, t) { if (t > 0.2 && t < 0.2 + tn("beaver.shake.time_s")) { const c = worldOf(A.rig.body); for (let i = 0; i < 4; i++) { const a = Math.random() * 6.28; BITS.spawn(c.clone().add(V3(Math.cos(a) * 0.15, 0.08, Math.sin(a) * 0.15)), V3(Math.cos(a) * 2.2, 0.8 + Math.random() * 1.5, Math.sin(a) * 2.2), DROP, 0.022, 1.2); } A.wet = Math.min(A.wet, 0.45); } },
      keys: () => [["crouch", 0.2], ["shake left", 0.2 + 0.25 / tn("beaver.shake.hz")], ["shake right", 0.2 + 0.75 / tn("beaver.shake.hz")], ["settle", 0.2 + tn("beaver.shake.time_s")]], stage: (A, t) => t < 0.2 ? "shake: crouch" : "shake off"}); }
  // sitting up on the tail to groom: forepaws rubbing the face
  // version 4: the rear-up's schedule (rear) moves the arms: the paws trail up to the face, settle, rub it in circles,
  // then reach for the ground ahead of the body as it sits back down
  function sitAct() { const D = 3.6; return Act("sit up and groom", D, (A, P, t) => { standPose(A, P); const r = rear(t, D), up = r.body, g = smooth(c01((t - 0.95) / 0.3)) * (1 - smooth(r.d));
      P.pitch = lerp(P.pitch, 1.05, up) - 0.08 * r.crouch; P.y = A.sp.H + 0.07 * up - 0.02 * r.crouch - 0.012 * r.give; P.tp = [lerp(TAIL_DRAG[0], 0.95, up), lerp(TAIL_DRAG[1], 0.1, up), lerp(TAIL_DRAG[2], 0, up)]; P.hp = -0.8 * up + 0.15 * Math.sin(t * 5) * g + 0.15 * r.crouch;
      const hx = -0.08 * up - 0.2 * (1 - up); P.lt[2] = {m: "root", x: hx, y: A.groundRel(-0.1, 0.1)}; P.lt[3] = {m: "root", x: hx, y: A.groundRel(-0.1, -0.1)};
      for (const i of [0, 1]) { const a = t * 7 + i * 3; armTo(A, P, i, segToRoot(A, P, i, 0.06 + 0.03 * Math.cos(a) * g, 0.02 + 0.03 * Math.sin(a) * g), r.arm, r.curl, 0.05); } },
    {keys: () => { const a = tn("shared.forelegs.anticipate_s"), u = tn("shared.forelegs.rise_s"), l = tn("shared.forelegs.lower_s"); return [["anticipation", a], ["rear up: paws trail", a + u * 0.55], ["paws up", a + u * 1.3], ["groom", 1.6], ["groom", 1.8], ["paws reach down", D - l * 0.55], ["down", D]]; }, stage: (A, t) => t < tn("shared.forelegs.anticipate_s") ? "sit up: anticipation" : t > D - tn("shared.forelegs.lower_s") ? "sit back down" : t < 0.95 ? "sit up" : "groom"}); }
  beaver.drop = A => { if (A.held) { A.held.on = "ground"; A.held = null; } };
  beaver.brain = A => {
    const down = FELL.some(T => T.state === "down" && T.branches.some(b => b.on === "tree")), standing = FELL.some(T => T.state === "standing"), r = A.rng();
    if (A.waterAt() > 0.5) return r < 0.5 ? "slap" : "shake";
    if (!down && !standing) return "regrow";
    if (down) return r < 0.62 ? "fetch" : r < 0.8 || !standing ? "sit" : "fell";
    return r < 0.8 ? "fell" : "sit";
  };

  // ======================= the river otter =======================
  const otterRig = buildRig(SPEC.otter);
  const SLIDE = RAMPS[2], PATH = RAMPS[1];
  const PATH_IN = V3(-21.2, 0, -1.6), SLIDE_PRE = V3(-15.4, 0, -11.6), RIDGE_W = V3(-26.4, 0, -10.9), ROCK_IN = V3(-7.5, 0, -2.9);
  const otter = Actor("otter", otterRig, {pos: V3(-30, 4, -11.2), yaw: 0, seedK: 2, where: "ridge", plans: {}, brain: null});
  // pond fish: trout circling at depth; the first is the otter's prey
  const FISH = [0, 1, 2].map(i => { const m = C3.make(W, "trout", 0.42); return {m, i, c: V3(-11 + i * 2.5, 0, -1 + i * 1.5), r: 2.2 + i * 0.6, w: 0.35 + i * 0.07, ph: i * 2, caught: false, hide: 0}; });
  const fishPos = (f, t) => V3(f.c.x + Math.cos(t * f.w + f.ph) * f.r, P_LVL - 0.75 - 0.2 * Math.sin(t * 0.4 + f.i), f.c.z + Math.sin(t * f.w + f.ph) * f.r * 0.7);
  function slideAct() {
    return Act("slide", 99, (A, P, t, s) => {
      const crouch = 0.35;
      if (t < crouch) { standPose(A, P); const k = smooth(t / crouch); P.y = A.sp.H - 0.05 * k; P.pitch = 0.12 * k; P.A = 0.25 * k; P.hp = -0.2; return; }
      P.y = 0.075; P.pitch = Math.atan2(SLIDE.b.y - SLIDE.a.y, Math.hypot(SLIDE.b.x - SLIDE.a.x, SLIDE.b.z - SLIDE.a.z)) * (s.inW ? 0.2 : 1); P.A = -0.08; P.hp = 0.35;
      P.la = [-1.45, -1.45, -1.55, -1.55]; P.lk = [0.15, 0.15, -0.1, -0.1]; P.lf = [0.3, 0.3, 1.2, 1.2]; P.lt = [null, null, null, null]; P.tp = [-0.05, 0, 0]; if (s.inW) { swimPose(A, P); P.hp = 0.3; } },
      {init(A, s) { s.d = 0; s.v = 0; s.inW = false; A.yaw = Math.atan2(-(SLIDE.b.z - SLIDE.a.z), SLIDE.b.x - SLIDE.a.x); },
        move(A, dt, t, s) { if (t < 0.35) return; const L = Math.hypot(SLIDE.b.x - SLIDE.a.x, SLIDE.b.z - SLIDE.a.z);
          if (!s.inW) { s.v = Math.min(tn("river_otter.slide.max_mps"), s.v + tn("river_otter.slide.accel_mps2") * dt + (t < 0.5 ? 2.2 * dt : 0)); s.d += s.v * dt; const k = c01(s.d / L);
            A.pos.set(lerp(SLIDE.a.x, SLIDE.b.x, k), groundAt(lerp(SLIDE.a.x, SLIDE.b.x, k), lerp(SLIDE.a.z, SLIDE.b.z, k)), lerp(SLIDE.a.z, SLIDE.b.z, k));
            const lv = levelAt(A.pos.x, A.pos.z); if (lv > A.pos.y + 0.02 && lv > -100) { s.inW = true; splash(V3(A.pos.x, P_LVL, A.pos.z), 30, 1.1, 1.6); } }
          else { s.v = Math.max(0, s.v - 2.2 * dt); A.pos.addScaledVector(A.fwd(), s.v * dt); A.pos.y += (P_LVL - A.pos.y) * c01(dt * 4); A.swimPh += dt * 8; if (s.v < 0.25) s.t = 99; }
          A.v = s.v; },
        keys: () => [["crouch", 0.35], ["push off", 0.5], ["slide", 0.9], ["splash", 1.4]], stage: (A, t) => t < 0.35 ? "slide: crouch (anticipation)" : "slide"});
  }
  function hopAct(to, name) { return Act(name || "hop", 0.95, (A, P, t, s) => { const k = c01((t - 0.3) / 0.45);
      if (t < 0.3) { if (A.waterAt() > 0.5) swimPose(A, P); else standPose(A, P); P.y -= 0.04 * smooth(t / 0.3); P.pitch += 0.25 * smooth(t / 0.3); return; }
      if (k < 1) { standPose(A, P); P.lt = [null, null, null, null]; P.la = [0.9, 0.9, -1.2, -1.2]; P.lk = [-0.4, -0.4, 0.3, 0.3]; P.lf = [0, 0, 0.8, 0.8]; P.pitch = 0.5 - k * 0.9; P.A = -0.2; return; }
      standPose(A, P); const l = c01((t - 0.75) / 0.2); P.y = A.sp.H - 0.05 * Math.sin(Math.PI * l); },
    {init(A, s) { s.from = A.pos.clone(); s.to = to.clone(); s.to.y = groundAt(to.x, to.z); if (levelAt(to.x, to.z) > s.to.y) s.to.y = levelAt(to.x, to.z); A.yaw = Math.atan2(-(to.z - A.pos.z), to.x - A.pos.x); },
      move(A, dt, t, s) { const k = c01((t - 0.3) / 0.45); if (t >= 0.3) { A.pos.lerpVectors(s.from, s.to, k); A.pos.y += 0.38 * Math.sin(Math.PI * k); if (k >= 1 && !s.splashed && levelAt(s.to.x, s.to.z) >= s.to.y - 0.01) { s.splashed = true; splash(s.to.clone(), 20, 0.9, 1.2); } } },
      keys: () => [["crouch", 0.3], ["leap", 0.5], ["land", 0.8]], stage: (A, t) => t < 0.3 ? "hop: crouch" : t < 0.75 ? "hop" : "hop: land"}); }
  // where the otter is now: on its rock, in the pond, up on the ridge, or on a bank between
  const whereOf = A => Math.hypot(A.pos.x - ROCK.x, A.pos.z - ROCK.z) < 0.9 && A.pos.y > P_LVL + 0.15 ? "rock" : A.waterAt() > 0.5 ? "water" : A.pos.y > 3.6 ? "ridge" : "bank";
  const ridgeToSlide = () => [Move(SLIDE_PRE, {name: "bound along the bank"}), Move(V3(SLIDE.a.x - 0.25, 0, SLIDE.a.z - 0.5), {name: "to the top of the slide", speed: 0.5, tol: 0.08}), slideAct()];
  const toWater = A => { const w = whereOf(A); return w === "rock" ? [hopAct(ROCK_IN, "into the water")] : w === "ridge" ? ridgeToSlide() : w === "bank" ? [Move(PATH.b, {name: "down to the water"}), Move(PATH_IN, {name: "into the water"})] : []; };
  otter.drop = A => { if (A.held && A.held.fish) { const f = A.held.fish; f.caught = false; f.bite = 0; f.hide = 3; } A.held = null; };
  const PORP = {c: V3(-9.6, 0, -1.2), R: 3.4};   // the porpoising loop, clear of the lodge and the rock
  otter.plans = {
    slide(A) { const w = whereOf(A), st = w === "rock" ? [hopAct(ROCK_IN, "into the water")] : [];
      if (w === "rock" || w === "water") st.push(Move(PATH_IN, {name: "swim to the path"}), Move(PATH.b, {name: "out of the water"}));
      if (w !== "ridge") st.push(Move(RIDGE_W, {name: "up the path"}));
      st.push(...ridgeToSlide()); return st; },
    porpoise(A) { const st = toWater(A); st.push(Move(A2 => { const c = PORP.c, a = Math.atan2((A2.pos.z - c.z) / 0.75, A2.pos.x - c.x); return V3(c.x + Math.cos(a) * PORP.R, 0, c.z + Math.sin(a) * PORP.R * 0.75); }, {name: "swim out"}));
      st.push(Act("porpoise", 14, (A2, P, t, s) => { swimPose(A2, P); const dy = s.dy || 0; P.pitch = Math.atan2(dy, 1.4) * 1.1; P.A = -0.18 * Math.sin(s.ph || 0); P.la[0] = P.la[1] = -1.4; P.hp = 0.1; },
        {init(A2, s) { s.c = PORP.c; s.a0 = Math.atan2((A2.pos.z - s.c.z) / 0.75, A2.pos.x - s.c.x); s.d = 0; s.lastAbove = true; },
          move(A2, dt, t, s) { const R = PORP.R, sp = 1.4; s.d += sp * dt; const a = s.a0 - s.d / R, len = 1.8; s.ph = 2 * Math.PI * s.d / len;
            const x = s.c.x + Math.cos(a) * R, z = s.c.z + Math.sin(a) * R * 0.75, y = P_LVL - 0.22 + tn("river_otter.porpoise.arc_m") * EX() * Math.sin(s.ph);
            const ny = P_LVL - 0.22 + tn("river_otter.porpoise.arc_m") * EX() * Math.sin(s.ph + 0.1); s.dy = (ny - y) / (0.1 * len / (2 * Math.PI)) * 1.4 / 1.4;
            A2.yaw = Math.atan2(-(z - A2.pos.z), x - A2.pos.x); A2.pos.set(x, y, z); A2.v = sp; A2.swimPh += dt * 9;
            const above = y > P_LVL; if (above !== s.lastAbove) { s.lastAbove = above; splash(V3(x, P_LVL, z), above ? 10 : 16, 0.6, 1.0); } },
          keys: () => [["porpoise", null, null]], stage: () => "porpoise"})); return st; },
    fish(A) { const st = toWater(A);
      const f = FISH[0];
      st.push(Act("dive for a fish", 12, (A2, P, t) => { swimPose(A2, P, 1); P.A = 0.15 * Math.sin(A2.swimPh); },
        {init(A2, s) { f.hide = 0; f.m.visible = true; },
          move(A2, dt, t, s) { const fp = f.caught ? null : fishPos(f, W.t); if (!fp) { s.t = 99; return; } const d = fp.clone().sub(A2.pos), L = d.length(); A2.yaw += clamp(angWrap(Math.atan2(-d.z, d.x) - A2.yaw), -5 * dt, 5 * dt);
            const sp = Math.min(L, 1.7 * dt); A2.pos.addScaledVector(d.normalize(), sp); A2.vy = d.y * 1.7; A2.v = 1.7; A2.swimPh += dt * 11; if (L < 0.28) { f.caught = true; A2.held = {fish: f}; s.t = 99; } },
          stage: () => "dive for a fish"}),
        Move(ROCK_IN, {name: "carry it to the rock"}), hopAct(V3(ROCK.x, 0, ROCK.z), "onto the rock"),
        Act("eat the fish", () => EAT0() + tn("river_otter.eat.beat_s") * 6.5 + tn("shared.forelegs.lower_s"), (A2, P, t) => { const b = tn("river_otter.eat.beat_s"), D = EAT0() + b * 6.5 + tn("shared.forelegs.lower_s"), tb = t - EAT0(), d = [b * 0.3, b * 0.18, b * 0.12, b * 0.4];
            const S = tb > 0 && tb < b * 6.5 ? strike(tb % b, d, 0.35, 0.42, -0.55, 0.12) : {v: 0}, lift = c01(-S.v / 0.55), r = rear(t, D);
            sitPose(A2, P, r, i => [0.045 + 0.012 * lift, -0.03 + 0.03 * lift - (i ? 0.006 : 0)]); P.hp = lerp(0, -0.75 + S.v, r.body) + 0.12 * r.crouch; },
          {move(A2, dt, t0, s) { const b = tn("river_otter.eat.beat_s"), t = t0 - EAT0(), k = Math.floor((t - b * 0.6) / b); if (t < b * 6.5 && k > (s.k ?? -1) && t > b * 0.6) { s.k = k; f.bite = (f.bite || 0) + 1; const p = worldOf(A2.rig.jaw); for (let i = 0; i < 3; i++) BITS.spawn(p, V3((Math.random() - 0.5) * 0.8, 0.6 + Math.random(), (Math.random() - 0.5) * 0.8), i ? 0xdd576d : 0xbfcad7, 0.02, 5, {stick: true}); } },
            init(A2, s) { A2.where = "rock"; }, keys: () => { const b = tn("river_otter.eat.beat_s"); return [["sit up: paws trail", tn("shared.forelegs.anticipate_s") + 0.2], ...strikeKeys([b * 0.3, b * 0.18, b * 0.12, b * 0.4], EAT0())]; },
            stage: (A2, t) => { const b = tn("river_otter.eat.beat_s"), tb = t - EAT0(); return tb < 0 ? "sit up to eat" : tb > b * 6.5 ? "sit back down" : "eat: " + strike(tb % b, [b * 0.3, b * 0.18, b * 0.12, b * 0.4], 1, 1, 0, 0).st; }}),
        Act("done eating", 0.1, (A2, P) => standPose(A2, P), {init(A2) { A2.held = null; f.caught = false; f.bite = 0; f.hide = 6; f.m.visible = false; }}));
      return st; },
    groom(A) { return [Act("roll and groom", 3.6, (A2, P, t) => { const inW = A2.waterAt() > 0.5; if (inW) swimPose(A2, P); else standPose(A2, P);
        const r = t < 0.25 ? -0.2 * smooth(t / 0.25) : t < 0.8 ? lerp(-0.2, Math.PI, smooth((t - 0.25) / 0.55)) : t < 2.9 ? Math.PI + 0.25 * Math.sin((t - 0.8) * 3) : lerp(Math.PI, 2 * Math.PI, smooth(c01((t - 2.9) / 0.6)));
        // version 4: the paws come in to the belly as it rolls (blended from wherever they were, never switched), rub
        // and squeeze the fur in alternate circles, and go back out as it rolls back
        P.roll = r; const g = smooth(c01((t - 0.45) / 0.35)) * smooth(c01((3.15 - t) / 0.35));
        if (g > 0) { const a = t * 8; blendLeg(A2, P, 0, {m: "seg", x: 0.06 + 0.025 * Math.cos(a), y: -0.03 + 0.025 * Math.sin(a), f: 0.3 * Math.sin(a)}, g); blendLeg(A2, P, 1, {m: "seg", x: 0.06 + 0.025 * Math.cos(a + 2.5), y: -0.03 + 0.025 * Math.sin(a + 2.5), f: 0.3 * Math.sin(a + 2.5)}, g);
          for (const i of [2, 3]) { if (P.lt[i]) solveLeg(A2.rig, P, i); P.lt[i] = null; P.la[i] = lerp(P.la[i], 0.6 + 0.3 * Math.sin(t * 5), g); P.lk[i] = lerp(P.lk[i], -0.8, g); }
          P.hp = -0.35 * g; P.hy = 0.3 * Math.sin(t * 2.2) * g; P.y = lerp(P.y, inW ? 0 : A2.sp.H * 0.55, g); } },
      {keys: () => [["anticipation", 0.25], ["roll", 0.6], ["groom", 1.5], ["groom", 1.9], ["roll back", 3.2]], stage: (A2, t) => t < 0.25 ? "roll: anticipation" : t < 0.8 ? "roll" : t < 2.9 ? "groom" : "roll back"})]; },
    // version 4: on land the rear-up's schedule moves the arms (the crouch, the paws trailing up to the chest, wrists
    // hanging, a small overshoot and settle, then reaching down ahead of the body); in the water it treads, the forepaws
    // paddling small circles in front of its chest
    look(A) { return [Act("look around", () => 1.3 + tn("river_otter.look.hold_s") * 3, (A2, P, t) => { const inW = A2.waterAt() > 0.5; const hold = tn("river_otter.look.hold_s"), D = 1.3 + hold * 3, r = rear(t, D, {up: 0.36}), up = r.body;
        if (inW) { swimPose(A2, P); P.pitch = 1.25 * up; P.y = -A2.sp.float + 0.12 * up; for (const i of [0, 1]) { const a = t * 9 + i * 3; blendLeg(A2, P, i, {m: "seg", x: 0.045 + 0.02 * Math.cos(a), y: -0.05 + 0.02 * Math.sin(a), f: -0.3 + 0.3 * Math.sin(a)}, up); } }
        else { standPose(A2, P); P.pitch = lerp(P.pitch, 1.22, up) - 0.08 * r.crouch; P.y = A2.sp.H + 0.1 * up - 0.02 * r.crouch - 0.01 * r.give; const hx = -0.08 * up - 0.22 * (1 - up); P.lt[2] = {m: "root", x: hx, y: A2.groundRel(-0.1, 0.06)}; P.lt[3] = {m: "root", x: hx, y: A2.groundRel(-0.1, -0.06)};
          for (const i of [0, 1]) armTo(A2, P, i, segToRoot(A2, P, i, 0.035, -0.035 + 0.006 * Math.sin(t * 2.4 + i)), r.arm, r.curl - 0.7 * clamp(r.arm, 0, 1), 0.04); P.tp = [lerp(0.12, 0.9, up), lerp(0.07, 0.1, up), lerp(0.02, 0, up)]; }
        P.hp = -1.15 * up + 0.12 * r.crouch; const segT = (t - 0.35 - tn("shared.forelegs.anticipate_s")) / hold; const yaws = [0, 0.8, 0.8, -0.8, -0.8, 0, 0]; const i = clamp(Math.floor(segT * 2), 0, 5); P.hy = lerp(yaws[i], yaws[i + 1], smooth(c01(segT * 2 - i))) * up; },
      {keys: () => { const h = tn("river_otter.look.hold_s"), a = tn("shared.forelegs.anticipate_s"), D = 1.3 + h * 3; return [["anticipation", a], ["rear up: paws trail", a + 0.2], ["paws to the chest", a + 0.5], ["look left", 0.35 + a + h * 0.75], ["look right", 0.35 + a + h * 1.75], ["paws reach down", D - tn("shared.forelegs.lower_s") * 0.55], ["down", D]]; },
        stage: (A2, t) => { const D = 1.3 + tn("river_otter.look.hold_s") * 3; return t < tn("shared.forelegs.anticipate_s") ? "look: anticipation" : t > D - tn("shared.forelegs.lower_s") ? "look: back down" : "look around"; }})]; },
  };
  // sitting up on the haunches (r: the rear-up's schedule, reared by default); hand(i): where each forepaw is held, in
  // its segment's frame
  const EAT0 = () => tn("shared.forelegs.anticipate_s") + tn("shared.forelegs.rise_s") * 1.2;
  function sitPose(A, P, r = {body: 1, arm: 1, crouch: 0, give: 0, curl: 0}, hand = null) { standPose(A, P); const u = r.body;
    P.pitch = lerp(P.pitch, 0.9, u) - 0.08 * r.crouch; P.y = A.sp.H + 0.03 * u - 0.02 * r.crouch - 0.01 * r.give; const hx = lerp(-0.22, -0.1, u);
    P.lt[2] = {m: "root", x: hx, y: A.groundRel(-0.1, 0.06)}; P.lt[3] = {m: "root", x: hx, y: A.groundRel(-0.1, -0.06)};
    for (const i of [0, 1]) { const h = hand ? hand(i) : [0.04, i ? -0.03 : -0.02]; armTo(A, P, i, segToRoot(A, P, i, h[0], h[1]), r.arm, r.curl, 0.04); }
    P.tp = [lerp(0.12, 0.85, u), lerp(0.07, 0.15, u), lerp(0.02, 0.05, u)]; }
  otter.brain = A => { const r = A.rng(); A.where = whereOf(A);
    if (A.where === "rock") return r < 0.3 ? "groom" : r < 0.5 ? "look" : r < 0.75 ? "porpoise" : "slide";
    if (A.where === "ridge") return r < 0.85 ? "slide" : "look";
    return r < 0.38 ? "fish" : r < 0.62 ? "porpoise" : r < 0.9 ? "slide" : "look"; };

  // ======================= the sea otter =======================
  const seaRig = buildRig(SPEC.sea);
  const HOME = V3(55, 0, 1.2);
  const seaO = Actor("sea", seaRig, {pos: HOME.clone(), yaw: 2.3, seedK: 3, where: "water", plans: {}, brain: null});
  const stone = new THREE.Mesh(new THREE.DodecahedronGeometry(0.045, 0), W.std({map: tex("game/granite", 1), roughness: 0.8})); W.scene.add(stone);
  const urchinMesh = (() => { const K = C3.Kit(), b = new THREE.IcosahedronGeometry(0.045, 0); K.add(b, (x, y, z, n) => n % 3 ? 0x4f1d83 : 0x230c44); const v = b.attributes.position; for (let i = 0; i < v.count; i += 2) { const d = V3(v.getX(i), v.getY(i), v.getZ(i)).normalize(), q = new THREE.Quaternion().setFromUnitVectors(V3(0, 1, 0), d); K.add(C3.cyl(0, 0.008, 0.06, 4).translate(0, 0.03, 0), 0x7d2dbd, new THREE.Matrix4().compose(d.clone().multiplyScalar(0.04), q, V3(1, 1, 1))); }
    const m = new THREE.Mesh(K.build(), W.lit(new THREE.MeshStandardMaterial({vertexColors: true, flatShading: true, roughness: 0.8}), false)); W.scene.add(m); return m; })();
  seaO.urchin = 1;   // 1 whole, 0..1 eaten, null none (dive for one)
  // the kelp it wraps itself in to sleep: a ribbon wound round its body, grown along its length
  const ribbon = (() => { const N = 64, pos = new Float32Array(N * 2 * 3), idx = []; for (let i = 0; i < N - 1; i++) { const a = i * 2, b = a + 1, c = a + 2, d = a + 3; idx.push(a, b, c, b, d, c); }
    const g = new THREE.BufferGeometry(); g.setAttribute("position", new THREE.BufferAttribute(pos, 3)); g.setIndex(idx);
    for (let i = 0; i < N; i++) { const u = i / (N - 1), a = u * 2.4 * 2 * Math.PI, x = lerp(0.42, -0.5, u), r = 0.16 - 0.03 * Math.abs(u - 0.4); for (let s = 0; s < 2; s++) { const xx = x + (s ? 0.06 : -0.06); pos.set([xx, r * Math.cos(a), r * Math.sin(a)], (i * 2 + s) * 3); } }
    g.computeVertexNormals(); const m = new THREE.Mesh(g, W.std({color: 0x8d6a26, side: THREE.DoubleSide, roughness: 0.6})); m.visible = false; seaRig.root.add(m); m.position.y = 0.0; return {m, g, N}; })();
  const crackD = () => [tn("sea_otter.crack.windup_s"), tn("sea_otter.crack.anticipate_s"), tn("sea_otter.crack.strike_s"), tn("sea_otter.crack.recoil_s")];
  // on its back: the body rolled over, the head up looking at its chest, the hind flippers up, the tail flat
  function raftPose(A, P, t) { P.roll = Math.PI; P.pitch = 0.16; P.y = 0.02; P.hp = -0.55 + 0.05 * Math.sin(t * 0.7); P.A = -0.06;
    P.la[2] = P.la[3] = -1.1 + 0.08 * Math.sin(t * 1.3); P.lk[2] = P.lk[3] = 0.4; P.lf[2] = P.lf[3] = -2.1 + 0.2 * Math.sin(t * 1.3 + 1);
    P.lt[0] = {m: "seg", x: -0.05, y: -0.08 - 0.006 * Math.sin(t * 1.4), f: 0.4}; P.lt[1] = {m: "seg", x: -0.05, y: -0.08 - 0.006 * Math.sin(t * 1.4 + 0.4), f: 0.4}; P.tp = [-0.12, 0.05, 0.05]; }
  const drift = (A, dt) => { const t = W.t; A.pos.x += (HOME.x + Math.sin(t * 0.05) * 1.2 - A.pos.x) * c01(dt * 0.3); A.pos.z += (HOME.z + Math.cos(t * 0.04) * 1.0 - A.pos.z) * c01(dt * 0.3); A.pos.y = sea.h(A.pos.x, A.pos.z, W.t, W.camera.position) - 0.02; A.yaw += Math.sin(t * 0.13) * 0.05 * dt; };
  seaO.plans = {
    crack(A) { if (A.urchin == null) return seaO.plans.dive(A); return [Act("crack an urchin", () => crackD().reduce((a, b) => a + b) * tn("sea_otter.crack.cracks") + 0.4, (A2, P, t) => { raftPose(A2, P, W.t); const d = crackD(), beat = d.reduce((a, b) => a + b), n = tn("sea_otter.crack.cracks");
        if (t > beat * n) return; const r = tn("sea_otter.crack.raise_m") * EX(), S = strike(t % beat, d, r, r * 1.15, -0.035, 0.03); P.lt[0] = {m: "seg", x: -0.03, y: -0.22 - S.v, f: 0.5}; P.lt[1] = {m: "seg", x: -0.03, y: -0.22 - S.v, f: 0.5}; P.hp = -0.7 + S.v * 0.6; P.A = -0.06 - S.v * 0.4; },
      {move(A2, dt, t, s) { drift(A2, dt); const d = crackD(), beat = d.reduce((a, b) => a + b), k = Math.floor(t / beat); if (t % beat >= d[0] + d[1] + d[2] && k > (s.k ?? -1) && k < tn("sea_otter.crack.cracks")) { s.k = k; const p = stone.getWorldPosition(V3()); for (let i = 0; i < 6; i++) BITS.spawn(p.clone().add(V3(0, 0.05, 0)), V3((Math.random() - 0.5) * 1.6, 0.8 + Math.random() * 1.6, (Math.random() - 0.5) * 1.6), i % 2 ? 0x7d2dbd : 0x4f1d83, 0.02, 4); BITS.spawn(p, V3(0, 1.5, 0), DROP, 0.02, 1); RINGS.spawn(A2.pos.x, A2.pos.y + 0.02, A2.pos.z, 0.5, 1, 0.4); } },
        keys: () => strikeKeys(crackD()), stage: (A2, t) => { const d = crackD(), beat = d.reduce((a, b) => a + b); return "crack: " + strike(t % beat, d, 1, 1, 0, 0).st; }}),
      Act("eat the urchin", 3.2, (A2, P, t) => { raftPose(A2, P, W.t); const S = strike(t % 0.8, [0.25, 0.1, 0.1, 0.35], 0.2, 0.25, -0.35, 0.08), lift = c01(-S.v / 0.35); P.lt[0] = P.lt[1] = {m: "seg", x: 0.02 + 0.035 * lift, y: -0.1 - 0.015 * lift, f: 0.5}; P.hp = -0.9 + S.v; },
        {move(A2, dt, t) { drift(A2, dt); A2.urchin = Math.max(0.05, 1 - t / 3.2); }, init(A2) {}, keys: () => strikeKeys([0.25, 0.1, 0.1, 0.35]), stage: () => "eat"}),
      // it lets go of the shell: the bits drift down off its chest, and the stone goes back under its foreleg
      Act("done", 0.1, (A2, P) => raftPose(A2, P, W.t), {init(A2) { A2.urchin = null; const p = urchinMesh.getWorldPosition(V3()); for (let i = 0; i < 7; i++) BITS.spawn(p, V3((Math.random() - 0.5) * 0.7, 0.2 + Math.random() * 0.4, (Math.random() - 0.5) * 0.7), i % 2 ? 0x7d2dbd : 0x4f1d83, 0.018, 3); }})]; },
    // roll over belly down (anticipation: a little further onto its back first), a flick of the tail and down; under
    // for dive_s, up nose first with a new urchin, then roll onto its back again
    dive(A) { const R = () => tn("sea_otter.roll.roll_s"), U = () => 1.6 + tn("sea_otter.dive.dive_s");
      const dep = u => 2.2 * smooth(c01(u / 1.0)) * (1 - smooth(c01((u - (U() - 1.3)) / 1.2)));
      return [Act("roll over and dive", () => 0.25 + 2 * R() + U() + 0.2, (A2, P, t) => { const r = R(), u = t - 0.25 - r, D = U();
          if (t < 0.25) { raftPose(A2, P, W.t); P.roll = Math.PI + 0.25 * smooth(t / 0.25); return; }
          if (u < 0) { raftPose(A2, P, W.t); P.roll = lerp(Math.PI + 0.25, 2 * Math.PI, smooth((t - 0.25) / r)); P.lt = [null, null, null, null]; P.la = [-1.2, -1.2, -0.6, -0.6]; P.lk = [1.2, 1.2, -0.4, -0.4]; return; }
          if (u < D) { swimPose(A2, P, 1); P.roll = 0; P.A = 0.2 * Math.sin(u * 7); for (let i = 0; i < 3; i++) P.tp[i] = 0.35 * Math.sin(u * 7 - i * 0.9);
            if (u < 0.45) P.tp = [0.9 * Math.sin(Math.PI * u / 0.45), 0.4, 0.2];
            P.pitch = -1.1 * smooth(c01(u / 0.45)) * (1 - smooth(c01((u - 0.45) / 0.8))) + 0.9 * smooth(c01((u - (D - 1.3)) / 0.6)) * (1 - smooth(c01((u - (D - 0.35)) / 0.35))); return; }
          raftPose(A2, P, W.t); P.roll = lerp(0, Math.PI, smooth(c01((u - D) / r))); },
        {init(A2, s) { s.went = false; s.up = false; },
          move(A2, dt, t, s) { const r = R(), u = t - 0.25 - r, D = U(), surf = sea.h(A2.pos.x, A2.pos.z, W.t, W.camera.position);
            if (u < 0) { drift(A2, dt); return; }
            if (!s.went) { s.went = true; splash(A2.pos.clone().setY(surf), 18, 0.8, 1.2); A2.urchin = null; }
            A2.pos.y = surf - 0.02 - (u < D ? dep(u) : 0); A2.pos.addScaledVector(A2.fwd(), (u < D ? 0.7 : 0.1) * dt); if (u > 1.0 && u < D - 1.0) A2.yaw += 0.55 * dt; A2.swimPh += dt * 7; A2.v = u < D ? 0.7 : 0;
            if (u > D - 0.25 && !s.up) { s.up = true; A2.urchin = 1; splash(A2.pos.clone().setY(surf), 14, 0.7, 1.0); } },
          keys: () => [["anticipation", 0.25], ["roll", 0.25 + R() * 0.6], ["tail flick", 0.25 + R() + 0.25], ["dive", 0.25 + R() + 1.0]],
          stage: (A2, t) => { const u = t - 0.25 - R(); return t < 0.25 ? "roll: anticipation" : u < 0 ? "roll over" : u < 0.45 ? "tail flick" : u < U() ? "dive" : "roll onto its back"; }})]; },
    // grooming with empty paws (version 4): they work along its belly and chest, rubbing in circles and squeezing the
    // fur between them, rolling it from side to side
    groom(A) { return [Act("groom", 4.2, (A2, P, t) => { raftPose(A2, P, W.t); P.roll = Math.PI + 0.55 * Math.sin(t * 2.3) * Math.sin(Math.PI * t / 4.2); const g = smooth(c01(t / 0.4)) * smooth(c01((4.2 - t) / 0.4)), a = t * 7, along = 0.05 * Math.sin(t * 1.3), sq = 0.02 * Math.max(0, Math.sin(t * 3.5));
        for (const i of [0, 1]) blendLeg(A2, P, i, {m: "seg", x: -0.03 + along + 0.035 * Math.cos(a + i * 3), y: -0.1 + 0.025 * Math.sin(a + i * 3), f: 0.4 + 0.4 * Math.sin(a + i * 3)}, g);
        if (!P.lt[0]) { P.la[0] += sq * 3; P.la[1] -= sq * 3; } P.hy = 0.35 * Math.sin(t * 2.3); },
      {move(A2, dt) { drift(A2, dt); }, keys: () => [["roll left", 0.68], ["roll right", 2.05], ["rub", 3.0]], stage: () => "groom"})]; },
    // asleep with empty paws (version 4): they rest on its chest while it wraps, come up toward its face and its head curls down onto them once it is
    // asleep, and go back to its chest as it wakes
    sleep(A) { const w = () => tn("sea_otter.sleep.kelp_wrap_s"); return [Act("wrap in kelp and sleep", () => w() + 7 + 1.5, (A2, P, t) => { raftPose(A2, P, W.t); const W0 = w(), g = smooth(c01((t - W0 - 0.3) / 0.9)) * (1 - smooth(c01((t - W0 - 6.2) / 0.9)));
        blendLeg(A2, P, 0, {m: "seg", x: 0.12, y: -0.13, f: 0.2}, g); blendLeg(A2, P, 1, {m: "seg", x: 0.12, y: -0.12, f: 0.2}, g); P.hp = -0.25 - 0.6 * g; P.y = 0.02 + 0.01 * Math.sin(t * 1.4); },
      {move(A2, dt, t) { drift(A2, dt); const W0 = w(), k = t < W0 ? smooth(t / W0) : t < W0 + 7 ? 1 : 1 - smooth(c01((t - W0 - 7) / 1.5)); ribbon.m.visible = k > 0.01; ribbon.g.setDrawRange(0, Math.floor(k * (ribbon.N - 1)) * 6); },
        keys: () => [["wrap", w() * 0.5], ["asleep", w() + 2]], stage: (A2, t) => t < w() ? "wrap in kelp" : t < w() + 7 ? "asleep" : "wake"})]; },
  };
  // an urchin brought up is cracked and eaten before anything else, so it grooms and sleeps with empty paws
  seaO.brain = A => { const r = A.rng(); if (A.urchin != null) return "crack"; return r < 0.55 ? "dive" : r < 0.78 ? "groom" : "sleep"; };
  // Idle steps used above (Hold) need the sea otter's drift when they hold it in the water
  const ACTORS = [beaver, otter, seaO];

  // ======================= today's motion, as the page moves these animals now =======================
  // (design B28's table): the beaver a rigid mesh shuttling on a sine and swapping models, the gnawer bobbing in place,
  // the river otter swaying like a fish on an ellipse, the sea otter rigid on its back drifting
  const TODAY = {b: C3.make(W, "beaver", 1.2), bb: C3.make(W, "beaver_branch", 1.2), g: C3.make(W, "beaver", 1.1), ro: C3.make(W, "riverotter", 1.1), so: C3.make(W, "otter", 1.4)};
  Object.values(TODAY).forEach(m => m.visible = false);
  function todayUpdate(t) {
    const a = t * 0.12, k = (Math.sin(a) + 1) / 2, from = V3(-12, 0, 11), to = V3(-3.2, 0, 1.5), p = from.clone().lerp(to, k), lv = levelAt(p.x, p.z); p.y = (lv > -100 ? lv : groundAt(p.x, p.z)) - 0.1;
    const carry = Math.cos(a) > 0, h = V3(Math.cos(a) * (carry ? 1 : -1), 0, 0.01).add(to.clone().sub(from).normalize().multiplyScalar(carry ? 1 : -1)); TODAY.b.visible = !carry; TODAY.bb.visible = carry; faceAlong(carry ? TODAY.bb : TODAY.b, p, h, W.camera.position);
    const T = FELL[0]; TODAY.g.visible = true; faceAlong(TODAY.g, V3(T.p.x + 0.8, T.p.y + Math.abs(Math.sin(t * 6)) * 0.03, T.p.z), V3(-1, 0, 0), W.camera.position);
    TODAY.ro.visible = true; const ao = t * 0.3, po = V3(-9 + Math.cos(ao) * 3.5, P_LVL - 0.15 - Math.max(0, Math.sin(t * 0.5)) * 1.8, -2 + Math.sin(ao) * 1.8); faceAlong(TODAY.ro, po, V3(-Math.sin(ao), 0, Math.cos(ao)), W.camera.position);
    TODAY.so.visible = true; const x = HOME.x + Math.sin(t * 0.05) * 1.5, z = HOME.z + Math.cos(t * 0.04) * 1.5; faceAlong(TODAY.so, V3(x, sea.h(x, z, t, W.camera.position) - 0.02, z), V3(1, 0, 0.3), W.camera.position);
  }

  // ======================= the side panel: actions, modes, tunables =======================
  let MODE = Q.get("mode") === "today" ? "today" : "prop", SLOW = Q.has("slow"), SHOWPOSES = Q.has("poses"), POSE_CLOCK = 0, FOLLOW = Q.get("follow") || "none";
  const side = document.getElementById("critSide");
  const btns = (id, label, list) => `<div class="lab2"><span>${label}</span><b id="${id}St"></b></div><div class="seg" role="group" aria-label="${label}" id="${id}">${list.map(([k, n]) => `<button data-a="${k}">${n}</button>`).join("")}</div>`;
  side.innerHTML = `${btns("crB", "Beaver", [["fell", "Fell a tree"], ["fetch", "Fetch a branch"], ["slap", "Tail slap and dive"], ["shake", "Shake off"], ["sit", "Sit up"]])}
    ${btns("crO", "River otter", [["slide", "Slide"], ["fish", "Catch a fish"], ["porpoise", "Porpoise"], ["groom", "Roll and groom"], ["look", "Look around"]])}
    ${btns("crS", "Sea otter", [["crack", "Crack an urchin"], ["dive", "Dive"], ["groom", "Groom"], ["sleep", "Sleep in kelp"]])}
    <div class="seg" role="group" aria-label="Motion"><button id="crToday" aria-pressed="false">Today</button><button id="crProp" aria-pressed="true">Proposed</button><button id="crSlow" class="tog" aria-pressed="false">Slow &times;0.25</button><button id="crPoses" class="tog" aria-pressed="false">Show poses</button></div>
    <div class="lab2"><span>Follow</span></div><div class="seg" role="group" aria-label="Follow" id="crFollow">${[["none", "Off"], ["beaver", "Beaver"], ["otter", "River otter"], ["sea", "Sea otter"]].map(([k, n]) => `<button data-f="${k}" aria-pressed="false">${n}</button>`).join("")}</div>
    <details class="tune"><summary>Tune</summary><div class="tunes" id="crTunes"></div><textarea class="ron" id="crRon" readonly aria-label="creature_motion.ron"></textarea></details>`;
  const ACT = {crB: beaver, crO: otter, crS: seaO};
  for (const [id, A] of Object.entries(ACT)) side.querySelectorAll(`#${id} button`).forEach(b => b.addEventListener("click", () => { if (MODE === "today") setMode("prop"); const ok = A.go(b.dataset.a, true); if (!ok) { const st = side.querySelector(`#${id}St`); st.textContent = "not now"; } }));
  const setMode = m => { MODE = m; side.querySelector("#crToday").setAttribute("aria-pressed", m === "today"); side.querySelector("#crProp").setAttribute("aria-pressed", m === "prop"); };
  side.querySelector("#crToday").addEventListener("click", () => setMode("today")); side.querySelector("#crProp").addEventListener("click", () => setMode("prop"));
  const tog = (id, get, set) => { const b = side.querySelector(id); const show = () => b.setAttribute("aria-pressed", String(get())); b.addEventListener("click", () => { set(!get()); show(); }); show(); };
  tog("#crSlow", () => SLOW, v => SLOW = v); tog("#crPoses", () => SHOWPOSES, v => { SHOWPOSES = v; POSE_CLOCK = 0; });
  const showFollow = () => side.querySelectorAll("#crFollow button").forEach(b => b.setAttribute("aria-pressed", b.dataset.f === FOLLOW));
  side.querySelectorAll("#crFollow button").forEach(b => b.addEventListener("click", () => { FOLLOW = b.dataset.f; showFollow(); })); showFollow(); setMode(MODE);
  const tunes = side.querySelector("#crTunes"), ronEl = side.querySelector("#crRon");
  let lastBlock = "";
  tunes.innerHTML = TUNE_SPEC.map(([b, k, v, lo, hi, st]) => { const head = b !== lastBlock ? `<div class="tblock">${b}</div>` : ""; lastBlock = b; return `${head}<label><span>${k}</span><b data-v="${b}.${k}">${v}</b><input type="range" min="${lo}" max="${hi}" step="${st}" value="${v}" data-k="${b}.${k}" aria-label="${b}.${k}"></label>`; }).join("");
  const INTS = new Set(TUNE_SPEC.filter(r => r[5] === 1).map(r => r[1])), fmt = (v, k) => INTS.has(k) ? String(Math.round(v)) : (Number.isInteger(+(+v).toFixed(3)) ? (+v).toFixed(1) : String(+(+v).toFixed(3)));
  function printRon() {
    const tree = {}; for (const [b, k] of TUNE_SPEC) { const [a, c] = b.split("."); if (!c) { tree[a] = TU[b + "." + k]; continue; } ((tree[a] = tree[a] || {})[c] = tree[a][c] || {})[k] = TU[b + "." + k]; }
    const out = ["// assets/config/creature_motion.ron (proposed: ocean-environments design B28)", "("];
    for (const [a, v] of Object.entries(tree)) { if (typeof v !== "object") { out.push(`    ${k2(a)}: ${fmt(v, a)},`); continue; } out.push(`    ${a}: (`); for (const [c, o] of Object.entries(v)) out.push(`        ${c}: (${Object.entries(o).map(([k, x]) => `${k}: ${fmt(x, k)}`).join(", ")}),`); out.push("    ),"); }
    out.push(")"); ronEl.value = out.join("\n");
  }
  const k2 = a => a === "all" ? "exaggerate" : a;
  tunes.querySelectorAll("input").forEach(i => i.addEventListener("input", () => { TU[i.dataset.k] = +i.value; tunes.querySelector(`[data-v="${i.dataset.k}"]`).textContent = i.value; printRon(); }));
  printRon();
  // the time of day: the sun's height sets the sky, the light and the lamps; at night the lamps light what is near them
  const hourEl = stage.querySelector("#crHour"), hourV = stage.querySelector("#crHourV");
  const tsel = stage.querySelector(".tsel"); tsel.innerHTML = [["Day", 13], ["Dusk", 17.5], ["Night", 22.5]].map(([n, h]) => `<button data-h="${h}" aria-pressed="false">${n}</button>`).join("");
  tsel.querySelectorAll("button").forEach(b => b.addEventListener("click", () => { hourEl.value = b.dataset.h; applyHour(); }));
  const C = (a, b, t) => new THREE.Color(a).lerp(new THREE.Color(b), t).getHex();
  function applyHour() {
    const h = +hourEl.value; hourV.textContent = `${String(Math.floor(h) % 24).padStart(2, "0")}:${String(Math.round((h % 1) * 60)).padStart(2, "0")}`;
    const ang = (h - 6) / 12 * Math.PI, e = Math.sin(ang), day = sstep(-0.15, 0.4, e), warm = sstep(0.45, 0.05, Math.abs(e)) * sstep(-0.22, 0.02, e);
    const T = {sky: C(C(0x05080e, 0x8fbfe0, day), 0xc0907a, warm * 0.7), haze: C(C(0x070a10, 0xa9c6d6, day), 0xd0a088, warm * 0.6), hemi: lerp(0.012, 0.62, day), sun: Math.max(0, sstep(-0.05, 0.18, e)), sunCol: C(0xfff1dc, 0xffa060, warm),
      sunPos: [-Math.cos(ang) * 150, Math.max(0.06, e) * 150 + 8, 50], moon: 0.02 * (1 - day), lamps: 1 - sstep(-0.04, 0.22, e), stars: 0.9 * (1 - sstep(-0.15, 0.15, e)), day};
    W.T = T; W.hemi.intensity = T.hemi; W.sun.intensity = T.sun; W.sun.color.setHex(T.sunCol); W.sun.position.set(...T.sunPos); W.moon.intensity = T.moon; W.stars.material.opacity = T.stars; W.moonDisc.visible = T.stars > 0.1;
    W.U.uHaze.value.setHex(T.haze).convertSRGBToLinear(); W.U.uDay.value = T.day; W.skyCol = new THREE.Color(T.sky);
    W.seas.forEach(s => s.setTime(T)); W.timeHooks.forEach(f => f(day > 0.5 ? "day" : "night", T));
    tsel.querySelectorAll("button").forEach(b => b.setAttribute("aria-pressed", Math.abs(+b.dataset.h - h) < 0.01));
  }
  hourEl.addEventListener("input", applyHour);

  // ======================= the frame =======================
  W.hudExtra = () => MODE === "today" ? "today's motion: rigid models sliding and swaying (design B28)" : ACTORS.map(A => `${A.id === "sea" ? "sea otter" : A.id === "otter" ? "river otter" : "beaver"}: ${A.stage}`).join("\n") + (SHOWPOSES ? "\nshowing the keyed poses" : "");
  let fishHideT = 0;
  let FROZEN = false;   // a capture holds the clock still between its fixed steps
  W.update = (t0, dt0) => {
    if (FROZEN) dt0 = 0;
    const dt = dt0 * (SLOW ? 0.25 : 1); W.simT = (W.simT || 0) + dt; const t = W.simT; POSE_CLOCK += dt0;
    for (const f of fronds) { f.g.rotation.z = Math.sin(t * 0.55 + f.ph) * 0.06; f.g.rotation.x = Math.cos(t * 0.45 + f.ph) * 0.05; }
    boat.position.y = sea.h(50, 6.8, W.t, W.camera.position) - 0.1; boat.rotation.z = Math.sin(W.t * 0.8) * 0.04;
    for (const f of FISH) { if (f.caught) continue; if (f.hide > 0) { f.hide -= dt; f.m.visible = false; continue; } f.m.visible = true; const p = fishPos(f, W.t), q = fishPos(f, W.t + 0.2); faceAlong(f.m, p, q.sub(p), W.camera.position); }
    RIFFLE.forEach(r => r.update(t, dt));
    if (MODE === "today") { todayUpdate(t); ACTORS.forEach(A => A.rig.root.visible = false); FELL.forEach(T => { T.pivot.quaternion.identity(); T.waist.visible = true; T.cut.visible = false; }); stone.visible = urchinMesh.visible = false; ribbon.m.visible = false; }
    else {
      Object.values(TODAY).forEach(m => m.visible = false); ACTORS.forEach(A => A.rig.root.visible = true);
      FELL.forEach(T => treeUpdate(T, SHOWPOSES ? 0 : dt));
      for (const A of ACTORS) A.update(SHOWPOSES ? 0 : dt, t);
      // what each one holds: the branch trails from the beaver's jaw; the fish in the otter's jaw, then its paws
      if (beaver.held) branchWorld(beaver, beaver.held);
      const of = otter.held && otter.held.fish; if (of) { of.m.visible = true; const eating = otter.step && otter.step.name === "eat the fish";
        if (eating) { const a = worldOf(otter.rig.legs[0].foot), b = worldOf(otter.rig.legs[1].foot), c = a.add(b).multiplyScalar(0.5).add(V3(0, 0.03, 0)); of.m.position.copy(c); of.m.quaternion.setFromUnitVectors(V3(1, 0, 0), V3(-Math.sin(otter.yaw), 0.6, -Math.cos(otter.yaw)).normalize()); of.m.scale.set(0.42 * 0.85 * Math.max(0.25, 1 - (of.bite || 0) * 0.13), 0.42 * 0.85, 0.42 * 0.85); }
        else { const j = worldOf(otter.rig.jaw); of.m.position.copy(j); of.m.quaternion.setFromUnitVectors(V3(1, 0, 0), V3(-Math.sin(otter.yaw), 0, -Math.cos(otter.yaw))); of.m.scale.setScalar(0.42 * 0.85); } }
      else FISH[0].m.scale.setScalar(0.42 * 0.85);
      const onBack = Math.abs(Math.cos(seaO.cur.roll)) > 0.5 && Math.cos(seaO.cur.roll) < 0;
      // the urchin is in its paws only while it forages and eats (up from the dive, cracked, eaten), and the stone is on its
      // chest only while it cracks; between uses the stone is tucked in the loose skin under its foreleg, out of sight
      const sn = seaO.step && seaO.step.name, eating = sn === "crack an urchin" || sn === "eat the urchin", foraging = sn === "roll over and dive" && seaO.step.up;
      stone.visible = onBack && sn === "crack an urchin"; stone.position.copy(worldOf(seaO.rig.chest, 0.17, -0.155, 0));
      urchinMesh.visible = seaO.urchin != null && (eating || foraging); if (urchinMesh.visible) { const a = worldOf(seaO.rig.legs[0].foot), b = worldOf(seaO.rig.legs[1].foot); urchinMesh.position.copy(a.add(b).multiplyScalar(0.5)); urchinMesh.scale.setScalar(Math.max(0.2, seaO.urchin)); }
      if (!(seaO.step && seaO.step.name === "wrap in kelp and sleep")) ribbon.m.visible = false;
      // a wake and rings behind a swimmer at the surface
      for (const A of ACTORS) { if (A.waterAt() < 0.5 || A.v < 0.2 || A.pos.y < levelAt(A.pos.x, A.pos.z) - 0.25) continue; A.wakeT = (A.wakeT || 0) + dt; if (A.wakeT > 0.45) { A.wakeT = 0; const p = worldOf(A.rig.neck); RINGS.spawn(p.x, levelAt(p.x, p.z), p.z, 0.7, 1.6, 0.35); } }
    }
    BITS.update(dt); RINGS.update(dt); ff.update(W.t); ff2.update(W.t);
    if (FOLLOW !== "none" && MODE === "prop") { const A = FOLLOW === "beaver" ? beaver : FOLLOW === "otter" ? otter : seaO; W.cam.target.lerp(A.pos.clone().add(V3(0, 0.3, 0)), c01(dt0 * 3)); }
    const st = {crB: beaver, crO: otter, crS: seaO}; for (const [id, A] of Object.entries(st)) { const el = side.querySelector(`#${id}St`); if (el && el.textContent !== A.stage) el.textContent = MODE === "today" ? "" : A.stage; }
  };
  // the riffle's trout: holding nose upstream in the current, now and then one leaps
  const RIFFLE = [0, 1, 2, 3].map(i => { const m = C3.make(W, "trout", 0.4); const x = 15 + i * 2.6, base = V3(x, 0.2, zc(x) + (i % 2 ? 0.5 : -0.4)); return {m, base, leapAt: 3 + i * 4.3, leapT: -1, update(t, dt) {
    let p = base.clone().add(V3(Math.sin(t * 0.9 + i) * 0.12, 0, Math.sin(t * 1.3 + i * 2) * 0.1)), h = V3(-1, 0, Math.sin(t * 1.7 + i) * 0.15);
    if (this.leapT < 0 && (t % 17) > this.leapAt && (t % 17) < this.leapAt + 0.05) { this.leapT = 0; splash(V3(p.x, 0.42, p.z), 12, 0.6, 0.8); }
    if (this.leapT >= 0) { this.leapT += dt; const k = this.leapT / 0.9; if (k >= 1) { this.leapT = -1; splash(V3(p.x - 0.7, 0.42, p.z), 14, 0.6, 0.9); } else { p.x -= 0.7 * k; p.y = 0.2 + 0.75 * Math.sin(Math.PI * k); h = V3(-1, 2.6 * Math.cos(Math.PI * k), 0); } }
    faceAlong(m, p, h, W.camera.position); }}; });
  // the opening plan: the beaver fells, the river otter slides, the sea otter cracks its urchin
  beaver.go(Q.get("beaver") || "fell", true); otter.go(Q.get("otter") || "slide", true); seaO.go(Q.get("sea") || "crack", true);
  W.setTime = () => applyHour();
  W.setViews({"Pond": [[-11, 2.6, 2], 0.45, 0.42, 24], "Felling": [[-14.4, 3.4, 11.5], 0.75, 0.2, 4.4], "Dam": [[-2.6, 2.9, 0.2], -1.25, 0.42, 7.5], "Slide": [[-14.2, 3.2, -8.2], 1.05, 0.3, 7.5], "Rock": [[-6.6, 3.0, -4.0], 0.9, 0.35, 4], "Riffle": [[19, 0.6, zc(19)], 0.35, 0.5, 11], "Sea otter": [[55, 0.15, 1.2], 0.6, 0.38, 4.5], "Valley": [[6, 1, 0], 0.5, 0.78, 92]});
  // a test and capture hook: run the scene in fixed steps without drawing, start an action, read the state
  window.__critters = {W, beaver, otter, sea: seaO, FELL, go: (a, n) => ({beaver, otter, sea: seaO})[a].go(n, true), freeze(v) { FROZEN = v; }, run(s, step = 1 / 30) { const f = FROZEN; FROZEN = false; for (let i = 0; i < s / step; i++) { W.t += step; ANIM.t = W.t; ANIM.dt = step; W.update(W.t, step); } FROZEN = f; },
    until(f, max = 150) { for (let i = 0; i < max * 30; i++) { if (f(window.__critters)) return true; this.run(1 / 30); } return false; },
    cam(A, yo = 0, pitch = 0.25, dist = 1.6, dy = 0.15) { W.cam.set([A.pos.x, A.pos.y + dy, A.pos.z], A.yaw + yo, pitch, dist); },
    hour: h => { hourEl.value = h; applyHour(); }, mode: setMode, poses: v => { SHOWPOSES = v; POSE_CLOCK = 0; }, state: () => ACTORS.map(A => ({id: A.id, stage: A.stage, pos: A.pos.toArray().map(v => +v.toFixed(2)), goal: A.goal}))};
  finish(W, "Pond"); applyHour();
  return W;
}
