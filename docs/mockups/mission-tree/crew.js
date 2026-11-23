// Mission Tree mockup, CREW tab: the crew missions of openspec/changes/coop-missions (design 1 to 7),
// drawn as the game's Missions screen would show them to a crew. "Raise a house together" (H1 to H7)
// with each player's share, the crew rules ("hands", "each", the step for two), and the four
// drafted chains. Nothing moves until the viewer steps or presses Play.
(function () {
  "use strict";
  const IMG = "mission-tree/items/";
  const UI = "mission-tree/ui/";
  const $ = (s) => document.querySelector(s);

  // The crew: whoever is in the world. Colours from the kit's palette, one per player.
  const CREW = [
    { id: 0, name: "Ana", col: "#faaf5c" },
    { id: 1, name: "Bo", col: "#84e3b5" },
    { id: 2, name: "Cy", col: "#8fb8ff" },
  ];

  // An objective: s = the line, n = its count, hands / each = the crew rules (design 3), tog = the
  // step for two (design 4).
  const o = (s, n, rule) => Object.assign({ s, n }, rule || {});
  const HOUSE = [
    { id: "H1", title: "Clear the site", icon: "stone-shovel", obj: [o("Level the site", 7, { hands: 2 })],
      help: "Dig the site flat. Two of you." },
    { id: "H2", title: "Gather timber", icon: "log", obj: [o("Fell trees", 6), o("Bring logs", 24, { each: 4 })],
      help: "Everyone brings at least 4 logs." },
    { id: "H3", title: "Lay the floor", icon: "planks", obj: [o("Floor pieces", 7, { hands: 2 })],
      help: "Seven floor pieces on the site." },
    { id: "H4", title: "Raise the walls", icon: "workbench", obj: [o("Walls", 6, { hands: 2, each: 1 }), o("Door", 1), o("Window", 1)],
      help: "Everyone puts up a wall." },
    { id: "H5", title: "Set the ridge beam", icon: "log", obj: [o("Ridge beam", 1, { tog: true })],
      help: "Two of you, one at each end, at the same time." },
    { id: "H6", title: "Roof it", icon: "planks", obj: [o("Roof", 1)], help: "A roof over the floor." },
    { id: "H7", title: "Light the hearth", icon: "campfire", obj: [o("Fire inside", 1)],
      help: "A campfire or brazier inside. The house is yours, all of you." },
  ];
  const DRAFTED = [
    { id: "D1", title: "Log drive", icon: "log", line: "Walk a log down a river, one at each end" },
    { id: "D2", title: "Bucket chain", icon: "water-bucket", line: "Fill a cistern, hand to hand" },
    { id: "D3", title: "Fish and fry", icon: "grilled-fish", line: "One catches, one cooks at the hearth" },
    { id: "D4", title: "The town's order", icon: "planks", line: "60 planks and 20 bricks by nightfall" },
  ];

  // The crew bot run: who does what, in order. Each step adds one to an objective for one player
  // (p), or sets the beam with two (pair).
  const RUN = [];
  const add = (m, i, p, k) => { for (let n = 0; n < (k || 1); n++) RUN.push({ m, i, p }); };
  add(0, 0, 0, 4); add(0, 0, 1, 3);
  add(1, 0, 0, 2); add(1, 0, 1, 2); add(1, 0, 2, 2);
  add(1, 1, 0, 10); add(1, 1, 1, 9); add(1, 1, 2, 5);
  add(2, 0, 1, 4); add(2, 0, 2, 3);
  add(3, 0, 0, 2); add(3, 0, 1, 3); add(3, 0, 2, 1); add(3, 1, 2); add(3, 2, 0);
  RUN.push({ m: 4, i: 0, pair: [0, 1] });
  add(5, 0, 2); add(6, 0, 1);

  // state: per mission, per objective, per player
  let k = 0, sel = null, timer = null;
  const counts = () => {
    const c = HOUSE.map((m) => m.obj.map(() => CREW.map(() => 0)));
    for (const s of RUN.slice(0, k)) {
      if (s.pair) s.pair.forEach((p) => { c[s.m][s.i][p] = 1; });
      else c[s.m][s.i][s.p] += 1;
    }
    return c;
  };
  const total = (row, ob) => (ob.tog ? (row.filter((x) => x > 0).length >= 2 ? 1 : 0) : row.reduce((a, b) => a + b, 0));
  // What an objective still lacks, in the game's plain words (design 3): null when met.
  function lack(row, ob) {
    const t = total(row, ob);
    if (ob.tog) return t >= 1 ? null : "Needs two of you at once";
    if (t < ob.n) return null;
    if (ob.hands && row.filter((x) => x > 0).length < ob.hands) return "Needs another pair of hands";
    if (ob.each && row.some((x) => x < ob.each)) {
      const who = CREW.filter((p, i) => row[i] < ob.each).map((p) => p.name).join(", ");
      return `Waiting on ${who}`;
    }
    return null;
  }
  const met = (row, ob) => total(row, ob) >= (ob.tog ? 1 : ob.n) && !lack(row, ob);
  function status(c) {
    let now = -1;
    return HOUSE.map((m, mi) => {
      const done = m.obj.every((ob, i) => met(c[mi][i], ob));
      if (done) return "done";
      if (now < 0) { now = mi; return "now"; }
      return "locked";
    });
  }

  const root = $("#crew");
  root.innerHTML = `
    <div class="crewbar">
      <div class="roster" aria-label="The crew">${CREW.map((p) => `<span class="who" style="--pc:${p.col}"><i></i>${p.name}</span>`).join("")}</div>
      <div class="grp"><span class="lab">CREW RUN</span>
        <button class="btn oak" id="cReset">Reset</button>
        <button class="btn oak" id="cStep">Step</button>
        <button class="btn oak" id="cPlay" aria-pressed="false">Play</button>
        <span class="stepno" id="cNo"></span>
      </div>
    </div>
    <div class="crewboard" id="cBoard">
      <div class="lane clane">RAISE A HOUSE TOGETHER</div>
      <div class="chain" id="cChain"></div>
      <div class="lane clane">DRAFTED</div>
      <div class="chain drafted" id="cDraft"></div>
    </div>
    <div class="hud" id="cHud"></div>
    <aside class="panel" id="cPanel" aria-label="Crew mission">
      <div class="phead"><div class="ptabs"><button class="ptab" aria-selected="true">CREW</button></div>
        <button class="btn xbtn" id="cClose" aria-label="Close">✕</button></div>
      <div class="pbody" id="cBody"></div>
    </aside>`;

  const slot = (icon, cls) => `<span class="slot ${cls || ""}"><img src="${IMG}${icon}.png" alt=""></span>`;
  const check = (on) => `<img class="ck" src="${UI}stone-check-${on ? "on" : "off"}.png" alt="">`;
  // One player's share of an objective: a short bar in their colour.
  const shares = (row, ob) => `<span class="shares">${CREW.map((p, i) => {
    const w = ob.tog ? (row[i] ? 100 : 0) : Math.min(100, (row[i] / ob.n) * 100 * CREW.length);
    return `<i title="${p.name} ${row[i]}" style="--pc:${p.col};--w:${w}%"></i>`;
  }).join("")}</span>`;

  function render() {
    const c = counts(), st = status(c);
    $("#cChain").innerHTML = HOUSE.map((m, mi) => {
      const s = st[mi];
      const lines = m.obj.map((ob, i) => {
        const row = c[mi][i], t = total(row, ob), ok = met(row, ob), why = s === "locked" ? null : lack(row, ob);
        const n = ob.tog ? `${t}/1` : `${Math.min(t, ob.n)}/${ob.n}`;
        return `<li class="${ok ? "full" : ""}">${check(ok)}<span>${ob.s}</span><em>${n}</em></li>` +
          (s === "locked" ? "" : `<li class="sh">${shares(row, ob)}</li>`) +
          (why ? `<li class="why">${why}</li>` : "");
      }).join("");
      const badge = s === "done" ? "DONE" : s === "now" ? "NOW" : "";
      return `${mi ? '<span class="link" aria-hidden="true"></span>' : ""}<div class="card st-${s}${sel === mi ? " sel" : ""}" data-i="${mi}" tabindex="0">
        ${badge ? `<span class="badge">${badge}</span>` : ""}${s === "done" ? `<img class="ck big" src="${UI}stone-check-on.png" alt="">` : ""}
        <div class="hd">${slot(m.icon, s === "now" ? "on" : s === "locked" ? "off" : "")}<div class="tt"><b>${m.title}</b><small>${m.id}</small></div></div>
        <ul class="obj">${lines}</ul></div>`;
    }).join("");
    $("#cDraft").innerHTML = DRAFTED.map((d) =>
      `<div class="card st-locked dcard"><div class="hd">${slot(d.icon, "off")}<div class="tt"><b>${d.title}</b><small>${d.line}</small></div></div></div>`).join("");
    const nowi = st.indexOf("now");
    const hud = $("#cHud");
    if (nowi < 0) { hud.className = "hud done"; hud.textContent = "Done: Raise a house together"; }
    else {
      const m = HOUSE[nowi], i = m.obj.findIndex((ob, j) => !met(c[nowi][j], ob)), ob = m.obj[i];
      hud.className = "hud";
      hud.textContent = `${m.title} ${ob.tog ? total(c[nowi][i], ob) : Math.min(total(c[nowi][i], ob), ob.n)}/${ob.tog ? 1 : ob.n}`;
    }
    $("#cNo").textContent = `${k} / ${RUN.length}`;
    renderPanel(c, st);
    root.classList.toggle("popen", sel !== null);
    relink();
  }
  // A link that ends a wrapped row joins nothing: hide it. Laid out only while the tab is shown.
  function relink() {
    const links = [...root.querySelectorAll(".chain .link")];
    links.forEach((l) => l.classList.remove("wrap"));
    for (const l of links) {
      const a = l.previousElementSibling, b = l.nextElementSibling;
      if (a && b && b.offsetTop > a.offsetTop + 8) l.classList.add("wrap");
    }
  }
  window.addEventListener("resize", relink);

  function renderPanel(c, st) {
    const panel = $("#cPanel");
    if (sel === null) { panel.classList.remove("open"); return; }
    const m = HOUSE[sel], s = st[sel];
    panel.classList.add("open");
    $("#cBody").innerHTML = `<div class="who">${slot(m.icon, s === "now" ? "on" : "")}<div><h2>${m.title}</h2><div class="sub">${s === "done" ? "Done" : s === "now" ? "Now" : "Locked"}</div></div></div>
      <h3>OBJECTIVES</h3><ul class="pobj">${m.obj.map((ob, i) => {
        const row = c[sel][i], t = total(row, ob), n = ob.tog ? 1 : ob.n, why = lack(row, ob);
        return `<li>${check(met(row, ob))}<span>${ob.s}</span><em>${Math.min(t, n)}/${n}</em>
          <span class="pb"><i style="width:${Math.min(100, (t / n) * 100)}%"></i></span>
          <span class="crewrows">${CREW.map((p, j) => `<span class="cr" style="--pc:${p.col}"><i></i>${p.name}<em>${row[j]}</em></span>`).join("")}</span>
          ${why ? `<span class="why">${why}</span>` : ""}</li>`;
      }).join("")}</ul>
      <h3>HOW</h3><p class="help">${m.help}</p>`;
  }

  root.addEventListener("click", (e) => {
    const card = e.target.closest(".card[data-i]");
    if (!card) return;
    sel = +card.dataset.i === sel ? null : +card.dataset.i;
    render();
  });
  root.addEventListener("keydown", (e) => {
    if (e.key === "Enter" && e.target.matches(".card[data-i]")) { sel = +e.target.dataset.i; render(); }
  });
  $("#cClose").addEventListener("click", () => { sel = null; render(); });

  const playBtn = $("#cPlay");
  function stop() { clearInterval(timer); timer = null; playBtn.textContent = "Play"; playBtn.setAttribute("aria-pressed", "false"); }
  playBtn.addEventListener("click", () => {
    if (timer) return stop();
    if (k >= RUN.length) k = 0;
    timer = setInterval(() => { if (k >= RUN.length) return stop(); k += 1; render(); }, 260);
    playBtn.textContent = "Pause"; playBtn.setAttribute("aria-pressed", "true");
  });
  $("#cStep").addEventListener("click", () => { stop(); k = Math.min(RUN.length, k + 1); render(); });
  $("#cReset").addEventListener("click", () => { stop(); k = 0; sel = null; render(); });

  // A crew partway through: the walls going up, five of six, Cy's share still to come.
  k = RUN.findIndex((s) => s.m === 3 && s.i === 0 && s.p === 2);
  sel = 3;
  render();
  window.crewTab = { relink, steps: RUN.length, at: (n) => { stop(); k = n; render(); }, select: (i) => { sel = i; render(); }, stop };
})();
