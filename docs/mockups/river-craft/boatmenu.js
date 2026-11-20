// River Craft (M34), version 6: the boat's menu (the owner, 2026-11-19: "rope length can be controlled in the boat
// menu, yes boats have menus for inventory and various stuff"; "also only really need one anchor per boat").
// An overlay panel over the river in the game's stone-and-carved-wood kit (assets/ui, copied into river-craft/ui):
// the boat's name and condition on an oak lintel, its hold beside a row of the pack (a tap sends a whole stack
// across, as a shift-click does in the game's pack), its one anchor (drop or weigh, and the rope's length: "To the
// bed" by default, or set in metres up to the rode's maximum), and its lantern. A scene gives what its boat has:
//   def.menu = {title, condition(), hold: [10 slots], anchor?: {state(), button(), toggle(), rope(), setRope(m|null),
//               max, rode()}, lantern?: {get(), set(on)}}
// Opened with C or the Boat button; closed with C, Esc or its X. Labels only: explanations are in the info modal.
(function () {
"use strict";
const RC = window.RC;
const NAMES = {log: "Log", rope: "Rope", bait: "Bait", "grilled-fish": "Grilled fish", planks: "Planks", "pine-cone": "Pine cone", fibre: "Fibre", stone: "Stone"};
const STACK = 99;
// the player's pack, shared by every tab: the first ten of its slots
RC.pack = [{item: "log", n: 20}, {item: "planks", n: 8}, {item: "pine-cone", n: 5}, {item: "fibre", n: 9}, {item: "stone", n: 14}, null, null, null, null, null];

// The game's send (vehicle/hold.rs `send`): the whole stack goes to the other side, merging into stacks of the same
// item first, then into empty slots; as much as fits goes and the rest stays.
function send(from, to, i) {
  const s = from[i]; if (!s) return;
  for (let j = 0; j < to.length && s.n > 0; j++) if (to[j] && to[j].item === s.item && to[j].n < STACK) { const k = Math.min(s.n, STACK - to[j].n); to[j].n += k; s.n -= k; }
  for (let j = 0; j < to.length && s.n > 0; j++) if (!to[j]) { to[j] = {item: s.item, n: s.n}; s.n = 0; }
  if (s.n <= 0) from[i] = null;
}

RC.BoatMenu = function (stage, onAct) {
  const el = document.createElement("div");
  el.className = "boatmenu"; el.hidden = true;
  el.setAttribute("role", "dialog"); el.setAttribute("aria-label", "Boat");
  stage.appendChild(el);
  let menu = null, dragging = false;
  const q = s => el.querySelector(s);

  function slots(box, mine, other) {
    box.innerHTML = "";
    mine.forEach((s, i) => {
      const b = document.createElement("button"); b.className = "bm-slot"; b.type = "button";
      b.setAttribute("aria-label", s ? `${NAMES[s.item] || s.item}, ${s.n}` : "Empty");
      if (s) {
        b.title = NAMES[s.item] || s.item;
        const img = document.createElement("img"); img.src = `river-craft/ui/items/${s.item}.png`; img.alt = ""; b.appendChild(img);
        const n = document.createElement("span"); n.className = "bm-n"; n.textContent = s.n; b.appendChild(n);
      }
      b.addEventListener("click", () => { send(mine, other, i); paintSlots(); });
      box.appendChild(b);
    });
  }
  function paintSlots() { slots(q(".bm-hold"), menu.hold, RC.pack); slots(q(".bm-pack"), RC.pack, menu.hold); }

  function ropeText(v) { return v === null ? "To the bed" : (v % 1 ? v.toFixed(1) : v.toFixed(0)) + " m"; }
  function build() {
    const A = menu.anchor, Lt = menu.lantern;
    el.innerHTML = `
      <div class="bm-head"><span class="bm-lintel"></span><span class="bm-cond"></span>${Lt ? `<button class="bm-check" type="button">Lantern</button>` : ""}<button class="btn bm-x" type="button" aria-label="Close">&#x2715;</button></div>
      ${menu.hold ? `<h3 class="bm-h">Hold</h3><div class="bm-grid bm-hold" role="group" aria-label="Hold"></div>
      <h3 class="bm-h">Pack</h3><div class="bm-grid bm-pack" role="group" aria-label="Pack"></div>` : ""}
      ${A ? `<h3 class="bm-h">Anchor</h3>
      <div class="bm-row"><span class="bm-state"></span><button class="btn oak bm-drop" type="button"></button></div>
      <div class="bm-row"><span class="bm-lab">Rope</span><button class="btn bm-less" type="button" aria-label="Shorter rope">&minus;</button><output class="bm-rope"></output><button class="btn bm-more" type="button" aria-label="Longer rope">+</button><button class="btn bm-bed" type="button">To the bed</button></div>
      <input class="bm-range" type="range" min="0" max="${A.max}" step="0.5" aria-label="Rope length">
      <div class="bm-row bm-out"><span class="bm-lab">Out</span><span class="bm-outv"></span></div>` : ""}`;
    q(".bm-lintel").textContent = menu.title.toUpperCase();
    q(".bm-x").addEventListener("click", close);
    if (menu.hold) paintSlots();
    if (A) {
      q(".bm-drop").addEventListener("click", () => { A.toggle(); onAct(); refresh(); });
      const step = d => {
        const v = A.rope(), now = v === null ? (A.rode() ? Math.ceil(A.rode()) : 2) : v;
        const next = now + d;
        A.setRope(next < 2 ? null : Math.min(A.max, next)); onAct(); refresh();
      };
      q(".bm-less").addEventListener("click", () => step(-1));
      q(".bm-more").addEventListener("click", () => step(+1));
      q(".bm-bed").addEventListener("click", () => { A.setRope(null); onAct(); refresh(); });
      const r = q(".bm-range");
      r.addEventListener("input", () => { dragging = true; const v = +r.value; A.setRope(v < 1 ? null : v); q(".bm-rope").textContent = ropeText(A.rope()); onAct(); });
      r.addEventListener("change", () => { dragging = false; refresh(); });
    }
    if (Lt) q(".bm-check").addEventListener("click", () => { Lt.set(!Lt.get()); refresh(); });
    refresh();
  }
  function refresh() {
    if (!menu || el.hidden) return;
    q(".bm-cond").textContent = menu.condition ? menu.condition() : "";
    const A = menu.anchor;
    if (A) {
      q(".bm-state").textContent = A.state();
      const b = A.button(), d = q(".bm-drop");
      d.hidden = !b; if (b && d.textContent !== b) d.textContent = b;
      const v = A.rope();
      q(".bm-rope").textContent = ropeText(v);
      q(".bm-bed").disabled = v === null;
      q(".bm-more").disabled = v !== null && v >= A.max;
      if (!dragging) q(".bm-range").value = v === null ? 0 : v;
      const out = A.rode();
      q(".bm-out").hidden = out === null; if (out !== null) q(".bm-outv").textContent = out.toFixed(1) + " m";
    }
    const Lt = menu.lantern;
    if (Lt) q(".bm-check").setAttribute("aria-pressed", String(Lt.get()));
  }
  function open(m) { menu = m; el.hidden = false; stage.classList.add("menu-open"); build(); }
  function close() { el.hidden = true; stage.classList.remove("menu-open"); menu = null; }
  function toggle(m) { if (!el.hidden && menu === m) close(); else open(m); }
  return {open, close, toggle, refresh, get isOpen() { return !el.hidden; }};
};
})();
