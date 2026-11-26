// Survival mockup, HUD tab (survival-mode design 9, 12 to 14): the same first-person view twice,
// Explorer and Survival, painted at 320 x 180 and lit per pixel by every light in it (the sun or the
// moon, the campfire, the placed torch), so the HUD reads by day and at night. Nothing moves.
(function () {
  "use strict";
  const IMG = "mission-tree/items/";
  const W = 320, H = 180, HOR = 76;
  let time = "night", state = "fine";

  const STATES = {
    fine: { biome: "fields", hp: 92, st: 100, strain: 0, fd: 68, wt: 74, t: 37.0, tr: "", cause: null },
    hot: { biome: "desert", hp: 96, st: 84, strain: 12, fd: 60, wt: 41, t: 39.1, tr: "up", cause: "sun", danger: true },
    stroke: { biome: "desert", hp: 38, st: 70, strain: 0, fd: 55, wt: 0, t: 40.2, tr: "up", cause: "sun", danger: true, fx: "stroke", flash: "wt" },
    cold: { biome: "tundra", hp: 80, st: 62, strain: 20, fd: 30, wt: 70, t: 34.8, tr: "down", cause: "snowflake", fx: "cold" },
    winded: { biome: "fields", hp: 90, st: 0, strain: 60, fd: 52, wt: 66, t: 37.6, tr: "up", cause: null, flash: "st" },
    under: { biome: "under", hp: 85, st: 70, strain: 20, fd: 64, wt: 80, t: 36.6, tr: "down", cause: "water-drop", breath: 4 },
    worn: { biome: "fields", hp: 92, st: 100, strain: 0, fd: 68, wt: 74, t: 37.0, tr: "", cause: null, worn: true },
    dead: { biome: "fields", hp: 0, st: 0, strain: 0, fd: 50, wt: 50, t: 37.0, tr: "", cause: null, fx: "dead" },
  };

  // ---- the painting: albedo, emission and a depth per pixel
  function hash(x, y, s) {
    let h = (x * 374761393 + y * 668265263 + s * 2246822519) | 0;
    h = Math.imul(h ^ (h >>> 13), 1274126177);
    return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
  }
  const PAL = {
    fields: { g: [[86, 132, 58], [72, 116, 50], [100, 146, 66]], hill: [62, 98, 70], far: [88, 120, 120] },
    desert: { g: [[214, 176, 116], [198, 160, 104], [226, 190, 130]], hill: [176, 120, 84], far: [196, 150, 120] },
    tundra: { g: [[224, 232, 240], [204, 214, 226], [236, 240, 246]], hill: [150, 164, 182], far: [176, 190, 206] },
    under: { g: [[96, 120, 92], [82, 104, 82], [110, 132, 98]], hill: [60, 90, 96], far: [60, 100, 120] },
  };
  function paint(biome, night) {
    const alb = new Float32Array(W * H * 3), emi = new Float32Array(W * H * 3), dep = new Float32Array(W * H);
    const P = PAL[biome];
    const put = (x, y, c, d, e) => {
      x |= 0; y |= 0;
      if (x < 0 || y < 0 || x >= W || y >= H) return;
      const i = y * W + x;
      if (d !== undefined && dep[i] && dep[i] < d) return;
      alb[i * 3] = c[0]; alb[i * 3 + 1] = c[1]; alb[i * 3 + 2] = c[2];
      if (d !== undefined) dep[i] = d;
      const E = e || [0, 0, 0];
      emi[i * 3] = E[0]; emi[i * 3 + 1] = E[1]; emi[i * 3 + 2] = E[2];
    };
    // sky
    for (let y = 0; y < HOR + 30; y++) for (let x = 0; x < W; x++) {
      const k = y / HOR;
      let c;
      if (biome === "under") c = [30 + 20 * k, 80 + 30 * k, 110 + 20 * k];
      else if (night) c = [6 + 10 * k, 10 + 16 * k, 26 + 26 * k];
      else if (biome === "desert") c = [150 + 60 * k, 190 + 40 * k, 230 + 10 * k];
      else c = [110 + 70 * k, 160 + 50 * k, 220 + 20 * k];
      const i = y * W + x; emi[i * 3] = c[0]; emi[i * 3 + 1] = c[1]; emi[i * 3 + 2] = c[2]; dep[i] = 1e9;
      if (night && biome !== "under" && hash(x, y, 7) > 0.996) { emi[i * 3] = emi[i * 3 + 1] = emi[i * 3 + 2] = 200; }
    }
    if (night && biome !== "under") for (let y = 14; y < 28; y++) for (let x = 250; x < 264; x++) {
      const dx = x - 257, dy = y - 21, ddx = x - 261, ddy = y - 19;
      if (dx * dx + dy * dy < 42 && ddx * ddx + ddy * ddy > 30) { const i = y * W + x; emi[i * 3] = 170; emi[i * 3 + 1] = 176; emi[i * 3 + 2] = 190; }
    }
    if (!night && biome === "desert") for (let y = 8; y < 26; y++) for (let x = 60; x < 78; x++) {
      const dx = x - 69, dy = y - 17; if (dx * dx + dy * dy < 64) { const i = y * W + x; emi[i * 3] = 255; emi[i * 3 + 1] = 244; emi[i * 3 + 2] = 200; }
    }
    // far ridge and near hills
    for (let x = 0; x < W; x++) {
      const far = HOR - 10 - 8 * Math.sin(x * 0.03) - 5 * Math.sin(x * 0.081 + 1) - 3 * hash(x >> 3, 1, 3);
      const near = HOR - 3 - 6 * Math.max(0, Math.sin(x * 0.02 + 2)) - 2 * hash(x >> 2, 2, 5);
      for (let y = Math.floor(far); y < HOR; y++) put(x, y, y >= near ? P.hill : P.far, 5000);
    }
    // ground: blocky texels in perspective, a texel a quarter metre
    for (let y = HOR; y < H; y++) {
      const z = 900 / (y - HOR + 2);
      for (let x = 0; x < W; x++) {
        const u = Math.floor(((x - 160) * z) / 170), v = Math.floor(z / 1.3);
        const c = P.g[Math.floor(hash(u, v, 11) * 3)];
        const shade = 0.92 + 0.12 * hash(u >> 2, v >> 2, 13);
        put(x, y, [c[0] * shade, c[1] * shade, c[2] * shade], z);
      }
    }
    // props: trees, cacti or pines, as one shaped form each (no outline, the ground's palette)
    const props = [];
    for (let k = 0; k < 14; k++) props.push({ x: 10 + hash(k, 1, 21) * 300, y: HOR + 4 + hash(k, 2, 21) * 30 });
    props.sort((a, b) => a.y - b.y);
    props.forEach((p, k) => {
      const s = (p.y - HOR + 6) / 22, d = 900 / (p.y - HOR + 2);
      if (p.x > 170 && p.x < 240 && p.y > HOR + 22) return;
      if (biome === "fields" || biome === "under") {
        const th = 14 * s, cr = 9 * s;
        for (let yy = 0; yy < th; yy++) for (let xx = -1.2 * s; xx < 1.2 * s; xx++) put(p.x + xx, p.y - yy, [92, 64, 40], d);
        for (let yy = -cr * 1.1; yy < cr; yy++) for (let xx = -cr; xx < cr; xx++) {
          const r = Math.hypot(xx, yy * 1.1) + 2.2 * s * (hash(Math.floor(p.x + xx), Math.floor(p.y + yy), 31) - 0.5);
          if (r < cr) { const l = 0.8 + 0.35 * (-xx - yy) / cr * 0.5; put(p.x + xx, p.y - th - cr * 0.4 + yy, [52 * l, 96 * l, 46 * l], d); }
        }
      } else if (biome === "desert") {
        if (k % 2) return;
        const th = 18 * s, w = 2.2 * s;
        for (let yy = 0; yy < th; yy++) for (let xx = -w; xx < w; xx++) put(p.x + xx, p.y - yy, xx < 0 ? [96, 150, 70] : [70, 120, 56], d);
        for (let yy = 0; yy < th * 0.45; yy++) for (let xx = -w * 0.8; xx < w * 0.8; xx++) { put(p.x - 4 * s + xx, p.y - th * 0.4 - yy, [86, 140, 64], d); put(p.x + 4 * s + xx, p.y - th * 0.55 - yy, [70, 120, 56], d); }
        for (let xx = -4 * s; xx < 4 * s; xx++) for (let yy = 0; yy < w; yy++) put(p.x + xx, p.y - th * 0.4 + yy, [78, 130, 60], d);
      } else {
        const th = 22 * s;
        for (let yy = 0; yy < th; yy++) {
          const half = (th - yy) * 0.32 + (yy % 6 < 2 ? 1.2 * s : 0);
          for (let xx = -half; xx < half; xx++) put(p.x + xx, p.y - yy, (yy % 6 < 2 && xx > -half + 1) ? [226, 234, 242] : (xx < 0 ? [44, 74, 66] : [34, 60, 54]), d);
        }
      }
    });
    // the campfire and a placed torch: stones, logs and flames (emissive)
    const lights = [];
    // no fire under water: there the view is lit only by the water's own light
    if (biome !== "under") {
    const fx = 205, fy = 136, fd = 900 / (fy - HOR + 2);
    for (let a = 0; a < 10; a++) {
      const sx = fx + Math.cos(a / 10 * 6.283) * 14, sy = fy + Math.sin(a / 10 * 6.283) * 4;
      for (let yy = -3; yy < 2; yy++) for (let xx = -3; xx < 3; xx++) if (xx * xx + yy * yy * 1.6 < 9) put(sx + xx, sy + yy, [120, 116, 110], fd - 0.1);
    }
    for (let xx = -10; xx < 10; xx++) for (let yy = -2; yy < 1; yy++) { put(fx + xx, fy - 1 + yy + xx * 0.15, [92, 60, 34], fd - 0.2); put(fx + xx, fy - 1 + yy - xx * 0.15, [80, 52, 30], fd - 0.2); }
    for (let yy = 0; yy < 18; yy++) {
      const half = (18 - yy) * 0.42 * (0.8 + 0.4 * hash(yy, 1, 41));
      for (let xx = -half; xx < half; xx++) {
        const core = Math.abs(xx) < half * 0.45 && yy < 10;
        put(fx + xx + Math.sin(yy * 0.7) * 1.5, fy - 3 - yy, [0, 0, 0], fd - 0.3, core ? [255, 236, 150] : [250, 140 - yy * 3, 40]);
      }
    }
    lights.push({ x: fx, y: fy - 6, r: 95, c: [1.0, 0.62, 0.3], k: 1.6 });
    const tx = 92, ty = 110, td = 900 / (ty - HOR + 2);
    for (let yy = 0; yy < 18; yy++) for (let xx = -1; xx < 1; xx++) put(tx + xx, ty - yy, [96, 66, 40], td);
    for (let yy = 0; yy < 4; yy++) for (let xx = -2; xx < 2; xx++) put(tx + xx, ty - 18 - yy, [70, 50, 34], td);
    for (let yy = 0; yy < 7; yy++) for (let xx = -2 + yy * 0.25; xx < 2 - yy * 0.25; xx++) put(tx + xx, ty - 22 - yy, [0, 0, 0], td - 0.1, yy < 3 ? [255, 230, 150] : [250, 130, 40]);
    lights.push({ x: tx, y: ty - 24, r: 55, c: [1.0, 0.66, 0.34], k: 1.2 });
    }
    return { alb, emi, dep, lights };
  }

  function render(canvas, biome, night) {
    const P = paint(biome, night);
    const ctx = canvas.getContext("2d");
    const img = ctx.createImageData(W, H);
    const amb = biome === "under" ? (night ? [0.22, 0.3, 0.4] : [0.45, 0.62, 0.72]) : night ? [0.09, 0.11, 0.18] : (biome === "desert" ? [1.08, 1.02, 0.94] : [0.98, 0.98, 1.0]);
    for (let y = 0; y < H; y++) for (let x = 0; x < W; x++) {
      const i = y * W + x;
      let r = amb[0], g = amb[1], b = amb[2];
      if (night || biome === "under") for (const L of P.lights) {
        const dx = x - L.x, dy = (y - L.y) * 1.8, d = Math.hypot(dx, dy) / L.r;
        if (d < 1) { const f = L.k * (1 - d) * (1 - d); r += L.c[0] * f; g += L.c[1] * f; b += L.c[2] * f; }
      }
      let R = P.alb[i * 3] * r + P.emi[i * 3], G = P.alb[i * 3 + 1] * g + P.emi[i * 3 + 1], B = P.alb[i * 3 + 2] * b + P.emi[i * 3 + 2];
      if (biome === "under") { const f = 0.55; R = R * f + 20; G = G * f + 60; B = B * f + 80; }
      img.data[i * 4] = Math.min(255, R); img.data[i * 4 + 1] = Math.min(255, G); img.data[i * 4 + 2] = Math.min(255, B); img.data[i * 4 + 3] = 255;
    }
    ctx.putImageData(img, 0, 0);
  }

  // ---- the HUD over each view
  const icon = (n) => `<img src="${IMG}${n}.png" alt="">`;
  function hotbar(items, worn) {
    return `<div class="hotbar">` + items.map((n, k) => {
      let extra = "", style = "";
      if (worn && n === "stone-pickaxe") extra = `<span class="wear"><i style="width:8%;background:#d8443a"></i></span>`;
      if (worn && n === "stone-shovel") extra = `<span class="wear"><i style="width:55%"></i></span>`;
      if (worn && n === "stone-axe") style = ` style="filter:grayscale(.8) brightness(.6)"`;
      const crack = worn && n === "stone-axe" ? `<svg viewBox="0 0 16 16" style="position:absolute;left:7px;top:7px;width:30px;height:30px" shape-rendering="crispEdges"><path d="M3 2h1v2h1v2h2v1h1v3h2v2h1v2h1" stroke="#010000" fill="none"/></svg>` : "";
      return `<div class="slot${k === 0 ? " sel" : ""}"><img src="${IMG}${n}.png" alt=""${style}>${crack}${extra}</div>`;
    }).join("") + `</div>`;
  }
  const compass = `<div class="compass"><span>W</span><span>·</span><span>N</span><span>·</span><span>E</span></div>`;
  function sbar(cls, v, c, ic, strain) {
    return `<div class="sbar${cls}" style="--c:${c}">${strain ? `<i class="s" style="width:${Math.min(100, v + strain)}%"></i>` : ""}<i style="width:${v}%"></i>${icon(ic)}</div>`;
  }
  function overlay(s, survival) {
    let h = compass;
    if (!survival) return h + hotbar(["stone-axe", "stone-pickaxe", "stone-shovel", "rod", "torch", "planks", "log", "grilled-fish", "bucket"]);
    const items = ["stone-axe", "stone-pickaxe", "stone-shovel", "rod", "torch", "stone-spear", "water-bucket", "roast-meat", "straw"];
    let vig = "";
    if (s.fx === "stroke") vig = `<div class="vign" style="background:radial-gradient(ellipse at center, rgba(40,10,0,.25) 30%, rgba(170,60,10,.75) 100%)"></div><div class="vign" style="background:rgba(0,0,0,.55)"></div>`;
    if (s.t >= 38.5 && s.fx !== "stroke") vig = `<div class="vign" style="background:radial-gradient(ellipse at center, transparent 55%, rgba(220,110,30,.35) 100%)"></div>`;
    if (s.fx === "cold") vig = `<div class="vign" style="background:radial-gradient(ellipse at center, transparent 50%, rgba(200,230,255,.28) 100%)"></div>`;
    if (s.fx === "dead") vig = `<div class="vign" style="background:rgba(40,0,0,.6)"></div><div class="cap">You died<br><span class="btn" style="display:inline-block;margin-top:12px;font-size:17px;pointer-events:auto">Wake up</span></div>`;
    const tcol = s.t >= 39.5 ? "#e0502a" : s.t >= 38 ? "#e0893a" : s.t <= 35.5 ? "#7fc4ff" : "#c9d24a";
    const tfill = Math.max(8, Math.min(100, (s.t - 33) / 8 * 100));
    const arrow = s.tr === "up" ? "▲" : s.tr === "down" ? "▼" : "";
    h += vig;
    h += `<div class="bars">` +
      sbar(s.hp < 30 ? " flash" : "", s.hp, "var(--hp)", "heart") +
      sbar(s.flash === "st" ? " flash" : "", s.st, "var(--st)", "stamina", s.strain) +
      sbar("", s.fd, "var(--fd)", "roast-meat") +
      sbar(s.flash === "wt" ? " flash" : "", s.wt, "var(--wt)", "water-drop") +
      `<div class="therm"><span class="tube"><i style="height:${tfill}%;background:${tcol}"></i></span>${s.t.toFixed(1)}°<span style="color:${tcol}">${arrow}</span>${s.cause ? icon(s.cause) : ""}${s.danger ? `<img src="${IMG}campfire.png" alt="" style="width:18px;height:18px;margin-left:4px">` : ""}</div>` +
      `</div>`;
    if (s.breath !== undefined) h += `<div class="breath">` + Array.from({ length: 10 }, (_, k) => `<img src="${IMG}breath.png" alt="" style="opacity:${k < s.breath ? 1 : 0.18}">`).join("") + `</div>`;
    return h + hotbar(items, s.worn);
  }

  function draw() {
    const s = STATES[state];
    const night = time === "night";
    ["vEx", "vSv"].forEach((id, k) => {
      const v = document.getElementById(id);
      render(v.querySelector("canvas"), s.biome, night);
      v.querySelector(".ov").innerHTML = overlay(s, k === 1);
    });
    document.querySelectorAll("[data-time]").forEach((b) => b.setAttribute("aria-pressed", String(b.dataset.time === time)));
  }
  document.querySelectorAll("[data-time]").forEach((b) => b.addEventListener("click", () => { time = b.dataset.time; draw(); }));
  document.getElementById("hudState").addEventListener("change", (e) => { state = e.target.value; draw(); });
  window.hudTab = {
    start() { draw(); },
    stop() {},
    set(t, st) { time = t; state = st; document.getElementById("hudState").value = st; draw(); },
  };
})();
