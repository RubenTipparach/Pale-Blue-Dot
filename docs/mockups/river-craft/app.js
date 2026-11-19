// River Craft (M34): the page. Tabs (one 3D view at a time: a tab's renderer is made the first time it opens and
// the hidden ones stop drawing), the controls, the keys, the on-screen pad, the HUD and the info modal.
(function () {
"use strict";
const RC = window.RC;
const TABS = ["raft", "tube", "tug", "logs", "nets", "anchor", "rowing"];
const stages = {}, defs = {};
const shared = {night: false, arrows: false, speed: 1};
let cur = null;
const $ = id => document.getElementById(id);

function make(name) {
  const def = RC.scenes[name]();
  // held actions (Pay out, Shorten) run every physics step while they are held
  const step = def.step;
  def.step = h => { for (const a of def.actions) if (a.hold && a.down) a.run(h); step(h); };
  defs[name] = def;
  const canvas = document.querySelector(`canvas[data-tab="${name}"]`);
  const st = RC.Stage(canvas, def);
  st.S.follow = !!def.followDefault;
  let n = 0; st.ctx.onFrame = () => { if (++n % 6 === 0 && cur === name) hud(); };
  stages[name] = st;
  return st;
}
function wake() { const st = stages[cur]; if (st && !st.S.playing) { st.S.playing = true; syncTime(); } }
function syncTime() {
  const st = stages[cur]; if (!st) return;
  $("play").setAttribute("aria-pressed", String(st.S.playing));
  $("play").firstChild.textContent = st.S.playing ? "Pause " : "Play ";
  $("paused").hidden = st.S.playing;
  $("follow").setAttribute("aria-pressed", String(st.S.follow));
}
function hud() {
  const def = defs[cur]; if (!def) return;
  const box = $("hud"); box.innerHTML = "";
  for (const [k, v, tone] of def.hud()) {
    const c = document.createElement("span"); c.className = "chip" + (tone ? " " + tone : "");
    if (k) { const b = document.createElement("b"); b.textContent = k; c.appendChild(b); }
    c.appendChild(document.createTextNode(v)); box.appendChild(c);
  }
  for (const a of def.actions) if (a.label2 && a.btn) { const t = a.label2(); if (a.btn.firstChild.textContent !== t + " ") a.btn.firstChild.textContent = t + " "; }
}
function buildActions() {
  const box = $("acts"), def = defs[cur]; box.innerHTML = "";
  const keys = document.createElement("span"); keys.className = "lab"; keys.textContent = def.keys; keys.style.textTransform = "none";
  // one choice (def.choice) or several (def.choices: the Rowing tab's Today / Proposed, gear and view)
  for (const ch of def.choices || (def.choice ? [def.choice] : [])) {
    if (ch.visible && !ch.visible()) continue;
    const g = document.createElement("span"); g.className = "grp"; g.setAttribute("role", "group"); g.setAttribute("aria-label", ch.label);
    const l = document.createElement("span"); l.className = "lab"; l.textContent = ch.label; g.appendChild(l);
    for (const [v, t] of ch.options) {
      const b = document.createElement("button"); b.className = "btn"; b.textContent = t; b.setAttribute("aria-pressed", String(ch.get() === v));
      b.addEventListener("click", () => { ch.set(v); if (def.choices) buildActions(); else for (const x of g.querySelectorAll("button")) x.setAttribute("aria-pressed", String(x === b)); stages[cur].draw(); hud(); });
      g.appendChild(b);
    }
    box.appendChild(g);
  }
  const g = document.createElement("span"); g.className = "grp";
  for (const a of def.actions) {
    const b = document.createElement("button"); b.className = "btn oak"; b.id = "act-" + a.id;
    b.appendChild(document.createTextNode((a.label2 ? a.label2() : a.label) + " "));
    const k = document.createElement("kbd"); k.textContent = a.key; b.appendChild(k);
    if (a.hold) {
      const on = e => { e.preventDefault(); a.down = true; b.classList.add("held"); wake(); };
      const off = () => { a.down = false; b.classList.remove("held"); };
      b.addEventListener("pointerdown", on); b.addEventListener("pointerup", off); b.addEventListener("pointerleave", off); b.addEventListener("pointercancel", off);
    } else b.addEventListener("click", () => { a.run(); wake(); hud(); });
    a.btn = b; g.appendChild(b);
  }
  box.appendChild(g); box.appendChild(keys);
}
function show(name) {
  if (!TABS.includes(name)) name = "raft";
  if (cur && stages[cur]) stages[cur].stop();
  for (const a of (defs[cur] && defs[cur].actions) || []) a.down = false;
  cur = name;
  for (const b of document.querySelectorAll(".tab")) b.setAttribute("aria-selected", String(b.dataset.tab === name));
  for (const c of document.querySelectorAll("#stage canvas[data-tab]")) c.hidden = c.dataset.tab !== name;
  for (const el of document.querySelectorAll("[data-only]")) el.hidden = el.dataset.only !== name;
  const st = stages[name] || make(name);
  Object.assign(st.S, {night: shared.night, arrows: shared.arrows, speed: shared.speed});
  st.apply(); st.resize(); st.start();
  buildActions(); syncTime(); hud();
  // the Rowing tab's tunables: sliders and the RON they make, under the controls
  const tune = $("tune");
  if (tune) { tune.hidden = !defs[name].tune; if (defs[name].tune && !tune.dataset.built) { defs[name].tune($("tunebody")); tune.dataset.built = "1"; } }
  if (location.hash !== "#" + name) history.replaceState(null, "", "#" + name);
}

// ---- tabs and the anchor in the link
document.querySelector("nav.tabs").addEventListener("click", e => { const b = e.target.closest(".tab"); if (b) show(b.dataset.tab); });
document.querySelector("nav.tabs").addEventListener("keydown", e => {
  if (e.key !== "ArrowRight" && e.key !== "ArrowLeft") return;
  const i = TABS.indexOf(cur) + (e.key === "ArrowRight" ? 1 : -1); show(TABS[(i + TABS.length) % TABS.length]);
  document.querySelector(`.tab[data-tab="${cur}"]`).focus();
});
window.addEventListener("hashchange", () => { const h = location.hash.slice(1); if (h !== cur && TABS.includes(h)) show(h); });

// ---- time and view
$("play").addEventListener("click", () => { const st = stages[cur]; st.S.playing = !st.S.playing; syncTime(); });
$("step").addEventListener("click", () => { const st = stages[cur]; st.S.playing = false; st.advance(0.1); syncTime(); st.draw(); hud(); });
$("speed").addEventListener("change", e => { shared.speed = +e.target.value; stages[cur].S.speed = shared.speed; });
function tod(night) {
  shared.night = night; $("day").setAttribute("aria-pressed", String(!night)); $("night").setAttribute("aria-pressed", String(night));
  const st = stages[cur]; st.S.night = night; st.apply(); st.draw();
}
$("day").addEventListener("click", () => tod(false)); $("night").addEventListener("click", () => tod(true));
$("arrows").addEventListener("click", () => {
  shared.arrows = !shared.arrows; $("arrows").setAttribute("aria-pressed", String(shared.arrows));
  const st = stages[cur]; st.S.arrows = shared.arrows; st.apply(); st.draw();
});
$("follow").addEventListener("click", () => { const st = stages[cur]; st.S.follow = !st.S.follow; syncTime(); st.draw(); });
$("reset").addEventListener("click", () => { const st = stages[cur]; st.S.playing = false; defs[cur].reset(); st.home(); syncTime(); st.draw(); hud(); });

// ---- steering: keys and the on-screen pad (hold to act; the tug's throttle moves a notch a tap)
const KEYMAP = {KeyW: "up", ArrowUp: "up", KeyS: "down", ArrowDown: "down", KeyA: "left", ArrowLeft: "left", KeyD: "right", ArrowRight: "right"};
function press(dir, down) {
  const def = defs[cur]; if (!def) return;
  if (down && def.press) def.press(dir);
  def.hold[dir] = down;
  if (down) { wake(); hud(); }
}
document.addEventListener("keydown", e => {
  if ($("modal").open || e.target.closest("select, input")) return;
  if (e.target.closest(".tabs") && (e.key === "ArrowLeft" || e.key === "ArrowRight")) return;
  if (e.code === "Space") { e.preventDefault(); $("play").click(); return; }
  const dir = KEYMAP[e.code];
  if (dir) { e.preventDefault(); if (!e.repeat) press(dir, true); return; }
  const def = defs[cur]; if (!def || e.repeat) return;
  for (const a of def.actions) if (e.key.toUpperCase() === a.key) {
    if (a.hold) { a.down = true; a.btn && a.btn.classList.add("held"); wake(); } else { a.run(); wake(); hud(); }
  }
});
document.addEventListener("keyup", e => {
  const dir = KEYMAP[e.code]; if (dir) press(dir, false);
  const def = defs[cur]; if (!def) return;
  for (const a of def.actions) if (a.hold && e.key.toUpperCase() === a.key) { a.down = false; a.btn && a.btn.classList.remove("held"); }
});
for (const b of document.querySelectorAll(".pad .btn")) {
  const dir = b.dataset.dir;
  b.addEventListener("pointerdown", e => { e.preventDefault(); b.setPointerCapture(e.pointerId); b.classList.add("held"); press(dir, true); });
  const up = () => { b.classList.remove("held"); press(dir, false); };
  b.addEventListener("pointerup", up); b.addEventListener("pointercancel", up); b.addEventListener("lostpointercapture", up);
}
window.addEventListener("blur", () => { const d = defs[cur]; if (d) for (const k in d.hold) d.hold[k] = false; });

// ---- the info modal: everything that explains the page; closes with its X, Esc or a tap outside
const modal = $("modal");
$("info").addEventListener("click", () => {
  modal.showModal();
  const t = document.getElementById("m-" + cur); if (t) t.scrollIntoView({block: "start"});
  else $("mbody").scrollTop = 0;
});
$("mclose").addEventListener("click", () => modal.close());
modal.addEventListener("click", e => { if (e.target === modal) modal.close(); });
modal.querySelector(".mnav").addEventListener("click", e => {
  const a = e.target.closest("a"); if (!a) return; e.preventDefault();
  const t = document.querySelector(a.getAttribute("href")); if (t) t.scrollIntoView({block: "start"});
});

window.RCApp = {stages, defs, show, get cur() { return cur; }};
show(location.hash.slice(1));
})();
