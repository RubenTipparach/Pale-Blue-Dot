// Survival mockup, Stamina tab: survival-mode design 12 run in the page, the same model as
// tools/survival/stamina_model.py (10 ms steps), every number on a slider. The graph shows the bar,
// the strain owed and the food spent against the food a body standing about spends.
(function () {
  "use strict";
  const DEF = [
    ["regen", "Refill, pts/s", 2, 30, 0.5, 12],
    ["delay", "Refill delay, s", 0, 4, 0.1, 1.0],
    ["c_low", "Food a point, at or below half", 0, 0.1, 0.005, 0.02],
    ["c_high", "Food a point, at full", 0, 0.15, 0.005, 0.05],
    ["strain", "Strain: refill cost x", 1, 3, 0.05, 1.5],
    ["sprint_d", "Sprint, pts/s", 0.2, 4, 0.1, 1.0],
    ["sprint_f", "Sprint, food/s", 0, 0.1, 0.005, 0.03],
    ["swim_d", "Swim, pts/s", 0.2, 4, 0.1, 0.8],
    ["swim_f", "Swim, food/s", 0, 0.1, 0.005, 0.025],
    ["jump", "Jump, pts", 0, 10, 0.5, 2],
    ["climb", "Climb, pts a metre", 0, 6, 0.25, 1.5],
    ["base_min", "Food bar, minutes standing about", 30, 150, 5, 75],
  ];
  const P = {};
  const side = document.getElementById("sliders");
  DEF.forEach(([k, name, lo, hi, step, v]) => {
    P[k] = v;
    const l = document.createElement("label");
    l.className = "sl";
    l.innerHTML = `${name} <b id="v_${k}">${v}</b><input type="range" min="${lo}" max="${hi}" step="${step}" value="${v}" data-k="${k}">`;
    side.appendChild(l);
  });
  const reset = document.createElement("button");
  reset.className = "btn"; reset.textContent = "Design defaults";
  side.appendChild(reset);

  const cost = (s) => (s <= 50 ? P.c_low : P.c_low + (P.c_high - P.c_low) * (s - 50) / 50);
  // One body under design 12, stepped in 10 ms steps: the graph's runs and the animation share it.
  function makeBody(food0) {
    const b = { s: 100, strain: 0, spent: 0, effort: 0, since: 99 };
    b.base = 100 / (P.base_min * 60);
    b.food = () => food0 - b.spent;
    b.step = function (mode, dt) {
      b.spent += b.base * dt;
      if (mode) {
        const d = P[mode + "_d"], f = P[mode + "_f"];
        b.s = Math.max(0, b.s - d * dt); b.strain = Math.min(100 - b.s, b.strain + d * dt);
        b.spent += f * dt; b.effort += f * dt; b.since = 0;
      } else {
        b.since += dt;
        if (b.since >= P.delay && b.s < 100) {
          const food = food0 - b.spent;
          const rate = P.regen * (food >= 50 ? 1 : 0.4 + 0.6 * Math.max(food, 0) / 50);
          const r = Math.min(rate * dt, 100 - b.s);
          const c = cost(b.s + r / 2) * r * (b.strain > 0 ? P.strain : 1);
          b.spent += c; b.effort += c; b.s += r; b.strain = Math.max(0, b.strain - r);
        }
      }
    };
    b.spend = (p) => { b.s = Math.max(0, b.s - p); b.since = 0; };
    return b;
  }
  function simulate(kind) {
    const food0 = kind === "day" ? 100 : 80;
    const b = makeBody(food0), base = b.base;
    const out = [];
    let t = 0;
    const dt = 0.01;
    function step(mode) { b.step(mode, dt); t += dt; }
    let every = kind === "day" ? 400 : 20, n = 0;
    const rec = () => { if (n++ % every === 0) out.push([t, b.s, b.strain, b.spent, base * t]); };
    const run = (secs, mode) => { for (let i = 0; i < Math.round(secs / dt); i++) { step(mode); rec(); } };
    const spend = (p) => b.spend(p);
    if (kind === "sprint") { run(60, "sprint"); run(10); }
    else if (kind === "swim") { run(50, "swim"); run(6); }
    else {
      let used = 0;
      for (let i = 0; i < 18; i++) { run(20, "sprint"); run(20); used += 40; }
      for (let i = 0; i < 2; i++) { run(60, "swim"); run(10); used += 70; }
      for (let i = 0; i < 40; i++) { spend(P.jump); run(5); used += 5; }
      for (let i = 0; i < 30; i++) { spend(P.climb); run(3); used += 3; }
      run(2880 - used);
    }
    out.push([t, b.s, b.strain, b.spent, base * t]);
    return { out, b, t, base };
  }

  function drawMain(res, kind) {
    const c = document.getElementById("gMain");
    const r = c.getBoundingClientRect();
    c.width = Math.max(300, Math.round(r.width)); c.height = Math.max(200, Math.round(r.height));
    const g = c.getContext("2d"), W = c.width, H = c.height, L = 54, R = 64, T = 14, B = 34;
    g.fillStyle = "#06121d"; g.fillRect(0, 0, W, H);
    const T1 = res.t, fmax = Math.max(1, Math.ceil(res.b.spent / (kind === "day" ? 20 : 2)) * (kind === "day" ? 20 : 2));
    const X = (t) => L + (W - L - R) * t / T1, Y = (v) => T + (H - T - B) * (1 - v / 100), YF = (v) => T + (H - T - B) * (1 - v / fmax);
    g.font = "13px IBM Plex Sans Condensed, sans-serif"; g.fillStyle = "#92a3b2"; g.strokeStyle = "rgba(62,88,110,.35)";
    for (let v = 0; v <= 100; v += 25) { g.beginPath(); g.moveTo(L, Y(v)); g.lineTo(W - R, Y(v)); g.stroke(); g.fillText(v, L - 30, Y(v) + 4); }
    for (let k = 0; k <= 4; k++) { const v = fmax * k / 4; g.fillText(v.toFixed(kind === "day" ? 0 : 1), W - R + 8, YF(v) + 4); }
    const tick = kind === "day" ? 480 : 10;
    for (let t = 0; t <= T1; t += tick) g.fillText(kind === "day" ? (t / 60).toFixed(0) + " min" : t + " s", X(t) - 12, H - 12);
    g.fillText("stamina", L + 6, T + 16); g.textAlign = "right"; g.fillText("food spent", W - R - 6, T + 16); g.textAlign = "left";
    // strain owed, as a band under the stamina line
    g.fillStyle = "rgba(138,106,42,.55)"; g.beginPath(); g.moveTo(X(0), Y(100));
    res.out.forEach((p) => g.lineTo(X(p[0]), Y(Math.min(100, p[1] + p[2]))));
    for (let i = res.out.length - 1; i >= 0; i--) g.lineTo(X(res.out[i][0]), Y(res.out[i][1]));
    g.fill();
    const line = (col, f, w, dash) => { g.strokeStyle = col; g.lineWidth = w; g.setLineDash(dash || []); g.beginPath(); res.out.forEach((p, i) => (i ? g.lineTo : g.moveTo).call(g, X(p[0]), f(p))); g.stroke(); g.setLineDash([]); };
    line("#5d7387", (p) => YF(p[4]), 2, [6, 5]);
    line("#e0893a", (p) => YF(p[3]), 2.5);
    line("#c9d24a", (p) => Y(p[1]), 2.5);
  }
  function drawCost() {
    const c = document.getElementById("gCost");
    const r = c.getBoundingClientRect();
    c.width = Math.max(300, Math.round(r.width)); c.height = 150;
    const g = c.getContext("2d"), W = c.width, H = c.height, L = 54, R = 64, T = 12, B = 30;
    g.fillStyle = "#06121d"; g.fillRect(0, 0, W, H);
    const top = Math.max(P.c_high, P.c_low) * P.strain * 1.1 || 0.01;
    const X = (s) => L + (W - L - R) * s / 100, Y = (v) => T + (H - T - B) * (1 - v / top);
    g.font = "13px IBM Plex Sans Condensed, sans-serif"; g.fillStyle = "#92a3b2";
    g.fillText("food a point refilled, by the bar's height", L, T + 4);
    [0, 50, 100].forEach((s) => g.fillText(s, X(s) - 6, H - 10));
    g.fillText(top.toFixed(3), 4, Y(top) + 10); g.fillText("0", 30, Y(0));
    const plot = (k, col, dash) => { g.strokeStyle = col; g.lineWidth = 2.5; g.setLineDash(dash || []); g.beginPath(); for (let s = 0; s <= 100; s++) (s ? g.lineTo : g.moveTo).call(g, X(s), Y(cost(s) * k)); g.stroke(); g.setLineDash([]); };
    plot(P.strain, "#8a6a2a", [6, 4]); plot(1, "#c9d24a");
    g.fillStyle = "#c9d24a"; g.fillText("plain", X(100) + 4, Y(cost(100)) + 4);
    g.fillStyle = "#c79a4a"; g.fillText("strain", X(100) + 4, Y(cost(100) * P.strain) + 4);
  }
  function redraw() {
    const kind = document.getElementById("case").value;
    const res = simulate(kind);
    drawMain(res, kind); drawCost();
    const idleMin = res.b.effort / (100 / P.base_min);
    document.getElementById("read").innerHTML =
      `<span>Food spent <b>${res.b.spent.toFixed(2)}</b></span><span>of it the effort <b>${res.b.effort.toFixed(2)}</b></span>` +
      `<span>standing about would be <b>${res.base * res.t > 0 ? (res.base * res.t).toFixed(2) : 0}</b></span>` +
      `<span>the effort = <b>${idleMin.toFixed(1)} min</b> of standing about</span>` +
      (kind === "day" ? `<span>a full bar lasts <b>${(48 * 100 / res.b.spent).toFixed(1)} min</b> of this play</span>` : "");
  }
  side.addEventListener("input", (e) => {
    const k = e.target.dataset.k; if (!k) return;
    P[k] = +e.target.value; document.getElementById("v_" + k).textContent = e.target.value; redraw();
  });
  document.getElementById("case").addEventListener("change", redraw);
  reset.addEventListener("click", () => { DEF.forEach(([k, , , , , v]) => { P[k] = v; side.querySelector(`[data-k="${k}"]`).value = v; document.getElementById("v_" + k).textContent = v; }); redraw(); });
  window.addEventListener("resize", () => { if (document.getElementById("stamina").classList.contains("on")) redraw(); });
  // The model in action (owner, 2026-11-26, survey G6: "Some sort of js animation would be nice to
  // see this in action. Add an expand button to lazy load"): the second half of this file, whose
  // scene, canvas and loop are made only when Expand is pressed, and which stands still until Play.
  window.staminaModel = { P, makeBody };
  window.staminaTab = {
    start() { requestAnimationFrame(redraw); },
    stop() { if (window.staminaAnim) window.staminaAnim.pause(); },
    setCase(k) { document.getElementById("case").value = k; redraw(); },
    redraw,
  };
})();


// ---- The model in action: a side view of a player sprinting across a field, swimming a lake,
// jumping up rocks, climbing a ledge and resting by a campfire, with the stamina bar (strain hatched)
// and the food bar moving live by design 12 and the sliders above. Made on Expand; still until Play.
// Painted at 640 x 180 (20 px a metre), lit per pixel: by day the sun, at night every torch and the
// campfire light what is near them, wherever the camera is, under a dim moon.
(function () {
  "use strict";
  const { P, makeBody } = window.staminaModel;
  const W = 640, H = 180, PX = 20, GY = 150, Z = PX / 10; // 20 px a metre; the ground line on screen
  const FOOD0 = 80, DT = 0.01;
  // the route, in model seconds and metres
  const SEG = [
    { k: "sprint", t0: 0, t1: 25, x0: 0, x1: 350, label: "Sprint" },
    { k: "walk", t0: 25, t1: 26, x0: 350, x1: 352, label: "Walk" },
    { k: "swim", t0: 26, t1: 46, x0: 352, x1: 432, label: "Swim" },
    { k: "jump", t0: 46, t1: 49.6, x0: 432, x1: 436, label: "Jump", n: 4 },
    { k: "walk", t0: 49.6, t1: 50.6, x0: 436, x1: 438, label: "Walk" },
    { k: "climb", t0: 50.6, t1: 54.6, x0: 438, x1: 438, label: "Climb", h: 4 },
    { k: "walk", t0: 54.6, t1: 58.6, x0: 438.4, x1: 445, label: "Walk" },
    { k: "rest", t0: 58.6, t1: 72, x0: 445, x1: 445, label: "Rest" },
  ];
  const END = SEG[SEG.length - 1].t1;
  const segAt = (t) => SEG.find((g) => t < g.t1) || SEG[SEG.length - 1];
  // the ground: a field to 350 m, a lake to 432 m, four rock steps of half a metre, a 4 m ledge at 438 m
  function ground(x) {
    if (x < 350) return 0;
    if (x < 432) return x < 353 ? -(x - 350) * 0.73 : x > 429 ? -2.2 + (x - 429) * 0.73 : -2.2;
    if (x < 436) return 0.5 * (Math.floor(x - 432) + 1);
    if (x < 438) return 2;
    return 6;
  }
  const isLake = (x) => x >= 350 && x < 432;
  const TORCHES = [12, 36, 60, 84, 108, 132, 156, 180, 204, 228, 252, 276, 300, 324, 346, 360, 371, 382, 393, 404, 415, 426, 434.5];
  const FIRE = 448;
  function hash(x, y, s) {
    let h = (x * 374761393 + y * 668265263 + s * 2246822519) | 0;
    h = Math.imul(h ^ (h >>> 13), 1274126177);
    return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
  }
  // where the player is, and how they stand, at model time t
  function where(t) {
    const g = segAt(t), f = Math.max(0, Math.min(1, (t - g.t0) / (g.t1 - g.t0)));
    let x = g.x0 + (g.x1 - g.x0) * f, y = ground(x), pose = g.k;
    if (g.k === "swim" || (g.k === "walk" && x > 350 && x < 353)) { y = -0.12; pose = g.k === "swim" ? "swim" : "walk"; }
    if (g.k === "jump") {
      const per = (g.t1 - g.t0) / g.n, i = Math.min(g.n - 1, Math.floor((t - g.t0) / per)), q = (t - g.t0 - i * per) / per;
      x = g.x0 + i + q; const from = 0.5 * i, to = 0.5 * (i + 1);
      y = from + (to - from) * q + 0.9 * Math.sin(Math.PI * q);
      if (q > 0.85) pose = "walk";
    }
    if (g.k === "climb") { x = 437.75; y = 2 + g.h * f; }
    if (g.k === "rest") y = 6;
    return { x, y, pose, seg: g };
  }
  // ---- the run: the model stepped in 10 ms steps, with the jumps' and the climb's spends
  let body, mt, playing = false, speed = 4, night = false, raf = 0, last = 0, built = false;
  let climbed = 0, jumped = 0;
  function reset() { body = makeBody(FOOD0); mt = 0; climbed = 0; jumped = 0; }
  function advance(to) {
    while (mt < to - 1e-9 && mt < END) {
      const g = segAt(mt);
      if (g.k === "jump") {
        const per = (g.t1 - g.t0) / g.n, i = Math.floor((mt - g.t0) / per + 1e-6);
        if (i >= jumped && i < g.n) { body.spend(P.jump); jumped = i + 1; }
      }
      if (g.k === "climb") {
        const m = Math.floor((mt - g.t0) / (g.t1 - g.t0) * g.h + 1e-6);
        if (m >= climbed && m < g.h) { body.spend(P.climb); climbed = m + 1; }
      }
      body.step(g.k === "sprint" ? "sprint" : g.k === "swim" ? "swim" : null, DT);
      mt += DT;
    }
  }
  // ---- painting
  let cv, g, al, ag, em, eg, fl, flg; // the view; albedo (lit); sky (emissive); flames (emissive, on top)
  function figure(c, sx, sy, pose, ph) {
    // one silhouette: limbs, torso and head drawn as one shape in the scene's own lit colours
    const S = PX, lw = (v) => (c.lineWidth = v * S);
    c.lineCap = "round"; c.lineJoin = "round";
    const P2 = (a, l, o) => [o[0] + Math.sin(a) * l * S, o[1] + Math.cos(a) * l * S];
    let lean = 0, hip, legA = [0, 0], kneeA = [0, 0], armA = [0, 0], elbA = [0, 0];
    const sw = Math.sin(ph);
    if (pose === "sprint") { lean = 0.32; legA = [0.95 * sw, -0.95 * sw]; kneeA = [-0.9 * Math.max(0, -sw) - 0.2, -0.9 * Math.max(0, sw) - 0.2]; armA = [-1.0 * sw, 1.0 * sw]; elbA = [1.4, 1.4]; }
    else if (pose === "walk") { lean = 0.06; legA = [0.45 * sw, -0.45 * sw]; kneeA = [-0.3 * Math.max(0, -sw), -0.3 * Math.max(0, sw)]; armA = [-0.4 * sw, 0.4 * sw]; elbA = [0.3, 0.3]; }
    else if (pose === "jump") { lean = 0.2; legA = [1.3, 0.9]; kneeA = [-1.9, -1.6]; armA = [2.6, 2.3]; elbA = [0.4, 0.4]; }
    else if (pose === "climb") { lean = -0.05; legA = [0.5 + 0.3 * sw, 0.5 - 0.3 * sw]; kneeA = [-1.4, -1.4]; armA = [Math.PI - 0.15 + 0.35 * sw, Math.PI - 0.15 - 0.35 * sw]; elbA = [-0.4, -0.4]; }
    else if (pose === "rest") { lean = -0.1; legA = [1.45, 1.35]; kneeA = [-1.3, -1.2]; armA = [0.7, 0.5]; elbA = [0.9, 0.9]; }
    if (pose === "swim") {
      // flat on the surface, head out, arms in a crawl and legs fluttering
      const hipP = [sx - 0.5 * S, sy], sh = [sx + 0.05 * S, sy - 0.05 * S];
      c.strokeStyle = "#5a4a3a"; lw(0.16);
      [0, 1].forEach((k) => { const a = -Math.PI / 2 - 0.15 + 0.25 * Math.sin(ph * 2 + k * Math.PI); const kn = P2(a, 0.42, hipP); c.beginPath(); c.moveTo(...hipP); c.lineTo(...kn); c.lineTo(...P2(a - 0.2, 0.42, kn)); c.stroke(); });
      c.strokeStyle = "#7a5636"; lw(0.26); c.beginPath(); c.moveTo(...hipP); c.lineTo(...sh); c.stroke();
      c.strokeStyle = "#c99a74"; lw(0.12);
      [0, 1].forEach((k) => { const a = ph + k * Math.PI; const el = [sh[0] + Math.cos(a) * 0.32 * S, sh[1] + Math.sin(a) * 0.3 * S]; c.beginPath(); c.moveTo(...sh); c.lineTo(...el); c.lineTo(el[0] + Math.cos(a + 0.5) * 0.28 * S, el[1] + Math.sin(a + 0.5) * 0.28 * S); c.stroke(); });
      c.fillStyle = "#c99a74"; c.beginPath(); c.arc(sx + 0.24 * S, sy - 0.2 * S, 0.13 * S, 0, 7); c.fill();
      c.fillStyle = "#4a3224"; c.beginPath(); c.arc(sx + 0.2 * S, sy - 0.25 * S, 0.12 * S, Math.PI * 0.95, Math.PI * 2.05); c.fill();
      return;
    }
    hip = pose === "rest" ? [sx, sy - 0.12 * S] : [sx, sy - 0.88 * S];
    const shoulder = P2(Math.PI + lean, 0.55, hip);
    // legs (the far one darker)
    [1, 0].forEach((k) => {
      c.strokeStyle = k ? "#4e4234" : "#5e5040"; lw(0.15);
      const kn = P2(legA[k], 0.45, hip), ft = P2(legA[k] + kneeA[k], 0.45, kn);
      c.beginPath(); c.moveTo(...hip); c.lineTo(...kn); c.lineTo(...ft); c.stroke();
    });
    c.strokeStyle = "#7a5636"; lw(0.27); c.beginPath(); c.moveTo(...hip); c.lineTo(...shoulder); c.stroke();
    [1, 0].forEach((k) => {
      c.strokeStyle = k ? "#6a4a30" : "#8a6440"; lw(0.12);
      const el = P2(armA[k] + lean, 0.3, shoulder), hd = P2(armA[k] + lean + elbA[k], 0.28, el);
      c.beginPath(); c.moveTo(...shoulder); c.lineTo(...el); c.lineTo(...hd); c.stroke();
    });
    const head = P2(Math.PI + lean, 0.2, shoulder);
    c.fillStyle = "#c99a74"; c.beginPath(); c.arc(head[0], head[1], 0.13 * S, 0, 7); c.fill();
    c.fillStyle = "#4a3224"; c.beginPath(); c.arc(head[0] - 0.02 * S, head[1] - 0.03 * S, 0.13 * S, Math.PI * 0.9, Math.PI * 2.1); c.fill();
  }
  function paint() {
    const w = where(mt), cx = w.x - 10;
    const cy = Math.max(0, Math.min(6, w.y)) * 0.8; // the camera rises with the player
    const X = (x) => (x - cx) * PX, Y = (y) => GY - (y - cy) * PX;
    const ph = mt * (w.pose === "sprint" ? 9 : w.pose === "swim" ? 3.2 : w.pose === "climb" ? 4 : 5);
    eg.clearRect(0, 0, W, H); ag.clearRect(0, 0, W, H); flg.clearRect(0, 0, W, H);
    // sky and moon or sun (emissive)
    for (let y = 0; y < H; y++) {
      const k = y / GY;
      eg.fillStyle = night ? `rgb(${6 + 10 * k},${10 + 16 * k},${26 + 26 * k})` : `rgb(${110 + 70 * k},${160 + 50 * k},${220 + 20 * k})`;
      eg.fillRect(0, y, W, 1);
    }
    if (night) {
      for (let i = 0; i < 70; i++) { const sx = ((hash(i, 1, 3) * 900 - cx * 0.6) % 900 + 900) % 900 - 130, sy = hash(i, 2, 3) * 90; eg.fillStyle = "rgba(220,220,230,.9)"; eg.fillRect(Math.round(sx), Math.round(sy), 1, 1); }
      eg.fillStyle = "#b9bfcd"; eg.beginPath(); eg.arc(560, 30, 9, 0, 7); eg.fill(); eg.fillStyle = "#0c1428"; eg.beginPath(); eg.arc(565, 27, 8, 0, 7); eg.fill();
    } else { eg.fillStyle = "#fff4c8"; eg.beginPath(); eg.arc(560, 30, 11, 0, 7); eg.fill(); }
    // far hills (lit, parallax)
    ag.fillStyle = night ? "#3d5a66" : "#6e8f8a";
    ag.beginPath(); ag.moveTo(0, H);
    for (let sx = 0; sx <= W; sx += 4) { const wx = sx / PX + cx * 0.3; ag.lineTo(sx, GY + cy * PX * 0.3 - 34 - 14 * Math.sin(wx * 0.05) - 8 * Math.sin(wx * 0.13 + 1)); }
    ag.lineTo(W, H); ag.fill();
    ag.fillStyle = night ? "#4c6a4e" : "#5f8a55";
    ag.beginPath(); ag.moveTo(0, H);
    for (let sx = 0; sx <= W; sx += 4) { const wx = sx / PX + cx * 0.6; ag.lineTo(sx, GY + cy * PX * 0.6 - 14 - 8 * Math.max(0, Math.sin(wx * 0.07 + 2))); }
    ag.lineTo(W, H); ag.fill();
    // trees in the field, each one shaped crown on its trunk, in the ground's palette
    for (let tx = Math.floor(cx / 18) * 18 - 18; tx < cx + W / PX + 18; tx += 18) {
      if (tx < 0 || tx > 335) continue;
      const ox = tx + 6 * hash(tx, 1, 5), s = (0.8 + 0.5 * hash(tx, 2, 5)) * Z, bx = X(ox);
      ag.fillStyle = "#5c4028"; ag.fillRect(bx - 2 * s, Y(0) - 26 * s, 4 * s, 26 * s);
      ag.fillStyle = "#3e6a36"; ag.beginPath();
      for (let a = 0; a <= 24; a++) { const th = a / 24 * 6.283, r = 13 * s * (0.85 + 0.25 * hash(tx, a, 9)); ag.lineTo(bx + Math.cos(th) * r, Y(0) - 32 * s + Math.sin(th) * r * 0.85); }
      ag.fill();
      ag.fillStyle = "#4f7d40"; ag.beginPath(); ag.arc(bx - 4 * s, Y(0) - 36 * s, 7 * s, 0, 7); ag.fill();
    }
    // the ground, a column at a time: grass texels, the lake bed, rock steps and the ledge
    for (let sx = 0; sx < W; sx++) {
      const wx = cx + sx / PX, gy = ground(wx), top = Math.round(Y(gy));
      for (let y = top; y < H; y++) {
        const u = Math.floor(wx * 4), v = Math.floor((y - top) / 2.5);
        let c;
        if (wx >= 432) c = hash(u, v, 13) > 0.5 ? [120, 116, 108] : [104, 100, 94];
        else if (isLake(wx)) c = [96, 86, 66];
        else c = y - top < 3 ? (hash(u, 0, 11) > 0.4 ? [86, 132, 58] : [72, 116, 50]) : (hash(u, v, 12) > 0.5 ? [112, 84, 56] : [98, 72, 48]);
        if (wx >= 438 && y - top < 3) c = [86, 132, 58];
        ag.fillStyle = `rgb(${c[0]},${c[1]},${c[2]})`; ag.fillRect(sx, y, 1, 1);
      }
    }
    // torches on posts and the campfire: the posts lit, the flames emissive
    const flames = [];
    TORCHES.forEach((tx) => { const bx = X(tx), by = Y(Math.max(0, ground(tx))); if (bx < -200 || bx > W + 200) return; ag.fillStyle = "#5c4028"; ag.fillRect(bx - Z, by - 18 * Z, 3 * Z, 18 * Z + (isLake(tx) ? 40 : 0)); ag.fillStyle = "#3e2c1c"; ag.fillRect(bx - 2 * Z, by - 21 * Z, 5 * Z, 4 * Z); flames.push([bx, by - 22 * Z, 0.7]); });
    { const bx = X(FIRE), by = Y(6); ag.fillStyle = "#7a7468"; for (let a = 0; a < 7; a++) { ag.beginPath(); ag.arc(bx + (-9 + a * 3) * Z, by - Z, 2.2 * Z, 0, 7); ag.fill(); } ag.fillStyle = "#5c4028"; ag.fillRect(bx - 7 * Z, by - 4 * Z, 14 * Z, 2 * Z); flames.push([bx, by - 4 * Z, 1.2]); }
    // the player
    figure(ag, X(w.x), Y(w.y), w.pose, ph);
    // the lake's water over the swimmer, lit like everything else
    ag.fillStyle = night ? "rgba(40,90,120,.72)" : "rgba(60,130,170,.66)";
    const l0 = Math.max(0, X(350)), l1 = Math.min(W, X(432));
    if (l1 > l0) { ag.fillRect(l0, Y(0) + 1, l1 - l0, H - Y(0)); ag.fillStyle = night ? "rgba(140,190,220,.5)" : "rgba(200,235,255,.7)"; for (let sx = Math.ceil(l0); sx < l1; sx += 2) if (hash(Math.floor((sx / PX + cx) * 3), Math.floor(mt * 2), 4) > 0.6) ag.fillRect(sx, Y(0) + 1, 2, 1); }
    // flames (emissive), flickering only while the clock runs
    flames.forEach(([fx, fy, s]) => {
      for (let k = 0; k < 9; k++) {
        const hgt = (7 + 4 * hash(k, Math.floor(mt * 6), 2)) * s * Z, wd = (3.2 - k * 0.3) * s * Z;
        flg.fillStyle = k < 4 ? "#fff0a0" : "#f08a2c";
        flg.beginPath(); flg.ellipse(fx + (hash(k, 3, 1) - 0.5) * 2 * s * Z, fy - hgt * 0.4 - k * 0.5 * s * Z, Math.max(0.6, wd), hgt * 0.5, 0, 0, 7); flg.fill();
      }
    });
    // light: albedo times (ambient + every light near), plus emission
    const A = ag.getImageData(0, 0, W, H), E = eg.getImageData(0, 0, W, H), O = g.createImageData(W, H);
    const amb = night ? [0.14, 0.16, 0.26] : [1, 1, 1];
    const lights = night ? flames.map(([fx, fy, s]) => ({ x: fx, y: fy, r: (s > 1 ? 13 : 9) * PX, k: s > 1 ? 1.8 : 1.3 })) : [];
    for (let y = 0; y < H; y++) for (let x = 0; x < W; x++) {
      const i = (y * W + x) * 4, a = A.data[i + 3] / 255;
      let r = amb[0], gg = amb[1], b = amb[2];
      for (const L of lights) { const d = Math.hypot(x - L.x, y - L.y) / L.r; if (d < 1) { const f = L.k * (1 - d) * (1 - d); r += f; gg += 0.62 * f; b += 0.3 * f; } }
      for (let c = 0; c < 3; c++) {
        const li = c === 0 ? r : c === 1 ? gg : b;
        O.data[i + c] = Math.min(255, E.data[i + c] * (1 - a) + A.data[i + c] * a * li);
      }
      O.data[i + 3] = 255;
    }
    g.putImageData(O, 0, 0);
    // flames over everything (they are their own light)
    g.drawImage(fl, 0, 0);
    hud(w);
  }
  function hud(w) {
    const st = document.getElementById("aSt"), fd = document.getElementById("aFd");
    st.querySelector("i.s").style.width = Math.min(100, body.s + body.strain) + "%";
    st.querySelector("i:not(.s)").style.width = body.s + "%";
    st.parentElement.classList.toggle("flash", body.s < 1);
    fd.querySelector("i").style.width = Math.max(0, body.food()) + "%";
    document.getElementById("aRead").innerHTML =
      `<span>${w.seg.label}</span><span>${mt.toFixed(1)} s</span><span>Stamina <b>${body.s.toFixed(0)}</b></span><span>Strain <b>${body.strain.toFixed(0)}</b></span><span>Food <b>${body.food().toFixed(2)}</b></span><span>Spent <b>${body.spent.toFixed(2)}</b></span><span>${speed}x</span>`;
  }
  function frame(now) {
    if (!playing) return;
    const dt = Math.min(0.1, (now - last) / 1000); last = now;
    advance(mt + dt * speed);
    paint();
    if (mt >= END) { playing = false; setPlay(); return; }
    raf = requestAnimationFrame(frame);
  }
  function setPlay() { const b = document.getElementById("aPlay"); if (b) { b.textContent = playing ? "Pause" : mt >= END ? "Replay" : "Play"; b.setAttribute("aria-pressed", String(playing)); } }
  function play() { if (mt >= END) reset(); playing = true; last = performance.now(); setPlay(); raf = requestAnimationFrame(frame); }
  function pause() { playing = false; cancelAnimationFrame(raf); setPlay(); }
  function build() {
    if (built) return;
    built = true;
    const host = document.getElementById("animHost");
    host.innerHTML = `<div class="abar"><button class="btn" id="aPlay" aria-pressed="false">Play</button><button class="btn" id="aRestart">Restart</button>
      <span class="lbl">SPEED</span><button class="btn" data-sp="1">1x</button><button class="btn" data-sp="4">4x</button><button class="btn" data-sp="10">10x</button>
      <span class="lbl">TIME</span><button class="btn" data-an="day">Day</button><button class="btn" data-an="night">Night</button>
      <div class="srow" style="--c:var(--st)"><img src="mission-tree/items/stamina.png" alt=""><div class="sbar" id="aSt"><i class="s"></i><i></i></div></div>
      <div class="srow" style="--c:var(--fd)"><img src="mission-tree/items/roast-meat.png" alt=""><div class="sbar" id="aFd"><i></i></div></div></div>
      <canvas id="aCv" width="${W}" height="${H}"></canvas><div class="read" id="aRead"></div>`;
    cv = document.getElementById("aCv"); g = cv.getContext("2d");
    al = document.createElement("canvas"); al.width = W; al.height = H; ag = al.getContext("2d");
    em = document.createElement("canvas"); em.width = W; em.height = H; eg = em.getContext("2d");
    fl = document.createElement("canvas"); fl.width = W; fl.height = H; flg = fl.getContext("2d");
    document.getElementById("aPlay").addEventListener("click", () => (playing ? pause() : play()));
    document.getElementById("aRestart").addEventListener("click", () => { pause(); reset(); paint(); setPlay(); });
    host.querySelectorAll("[data-sp]").forEach((b) => b.addEventListener("click", () => { speed = +b.dataset.sp; marks(); if (!playing) paint(); }));
    host.querySelectorAll("[data-an]").forEach((b) => b.addEventListener("click", () => { night = b.dataset.an === "night"; marks(); if (!playing) paint(); }));
    reset(); marks(); paint();
  }
  function marks() {
    document.querySelectorAll("[data-sp]").forEach((b) => b.setAttribute("aria-pressed", String(+b.dataset.sp === speed)));
    document.querySelectorAll("[data-an]").forEach((b) => b.setAttribute("aria-pressed", String((b.dataset.an === "night") === night)));
  }
  const exp = document.getElementById("animExpand");
  exp.addEventListener("click", () => {
    const host = document.getElementById("animHost"), open = host.hidden;
    if (open) { host.hidden = false; build(); exp.textContent = "Collapse"; }
    else { pause(); host.hidden = true; exp.textContent = "Expand"; }
    exp.setAttribute("aria-expanded", String(open));
    window.staminaTab.redraw();
  });
  window.staminaAnim = {
    pause,
    expand() { if (document.getElementById("animHost").hidden) exp.click(); },
    // a still at a model time, for the screenshots: the run replayed from the start to t
    at(o) { this.expand(); pause(); if (o.night !== undefined) night = o.night; reset(); advance(o.t); marks(); paint(); setPlay(); return { t: mt, s: body.s, strain: body.strain, food: body.food() }; },
  };
})();
