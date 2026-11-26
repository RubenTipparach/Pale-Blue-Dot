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
  function simulate(kind) {
    const b = { s: 100, strain: 0, spent: 0, effort: 0, since: 99 };
    const food0 = kind === "day" ? 100 : 80;
    const base = 100 / (P.base_min * 60);
    const out = [];
    let t = 0;
    const dt = 0.01;
    function step(mode) {
      b.spent += base * dt;
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
      t += dt;
    }
    let every = kind === "day" ? 400 : 20, n = 0;
    const rec = () => { if (n++ % every === 0) out.push([t, b.s, b.strain, b.spent, base * t]); };
    const run = (secs, mode) => { for (let i = 0; i < Math.round(secs / dt); i++) { step(mode); rec(); } };
    const spend = (p) => { b.s = Math.max(0, b.s - p); b.since = 0; };
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
  window.staminaTab = { start() { requestAnimationFrame(redraw); }, stop() {}, setCase(k) { document.getElementById("case").value = k; redraw(); } };
})();
