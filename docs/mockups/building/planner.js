// planner.js: M6's Planner and Site tabs (openspec/changes/building-planner).
//
// - A plan is in its own hex coordinates (design 1): rooms painted per storey, whose
//   outside edges are walls by themselves, inner walls, doors and windows, stairs (their
//   two cells cut from the floor above as the well), lanterns, furniture slots, the roof.
// - The planner's checks are build mode's rules on the plan (design 2), in M6's words.
// - The Site tab charts the plan onto a plot at an origin and a turn in sixths, checks it
//   whole against the world (design 5), sets it out with stakes and lime lines, builds it
//   piece by piece from a pack (design 6, with scaffolding), and changes it mid-build by a
//   diff of piece ids (design 7). Built is derived: a piece is built when the built map holds
//   its id with the same body.
// - One 3D view at a time: the site's renderer is made when its tab first opens, and its
//   loop runs only while the tab is open. Nothing moves until a button asks.
"use strict";
var TAB = "";
(function () {
  const $ = id => document.getElementById(id);
  const W = PBD.W, R = PBD.R, ROW = PBD.ROW, SH = 3;
  const DIRS = [[1, 0], [0, 1], [-1, 1], [-1, 0], [0, -1], [1, -1]];
  const K = (q, r) => q + "," + r, unK = k => k.split(",").map(Number);
  const hx = (q, r) => W * (q + r / 2), hz = (q, r) => ROW * r;
  const corner = (q, r, k) => { const a = (60 * k - 30) * Math.PI / 180; return [hx(q, r) + R * Math.cos(a), hz(q, r) + R * Math.sin(a)]; };
  const hexPts = (q, r, s = 1) => [0, 1, 2, 3, 4, 5].map(k => { const c = corner(q, r, k), x = hx(q, r), z = hz(q, r); return [x + (c[0] - x) * s, z + (c[1] - z) * s]; });
  const hexDist = (q, r) => Math.max(Math.abs(q), Math.abs(r), Math.abs(q + r));
  const nbr = (q, r, d) => [q + DIRS[((d % 6) + 6) % 6][0], r + DIRS[((d % 6) + 6) % 6][1]];
  const dvec = d => [Math.cos(d * Math.PI / 3), Math.sin(d * Math.PI / 3)];
  function axRound(q, r) { const s = -q - r; let rq = Math.round(q), rr = Math.round(r); const rs = Math.round(s), dq = Math.abs(rq - q), dr = Math.abs(rr - r), ds = Math.abs(rs - s); if (dq > dr && dq > ds) rq = -rr - rs; else if (dr > ds) rr = -rq - rs; return [rq, rr]; }
  const cellAt = (x, z) => { const r = z / ROW; return axRound(x / W - r / 2, r); };
  const clamp = (v, a, b) => v < a ? a : v > b ? b : v;
  const rot1 = ([q, r]) => [-r, q + r];
  const rotN = (c, t) => { for (let i = 0; i < ((t % 6) + 6) % 6; i++) c = rot1(c); return c; };
  const lin = h => new THREE.Color(h).convertSRGBToLinear();

  // ================================================================= tunables (design 12)
  const TUNE = { ghostOpacity: 0.35, waitOpacity: 0.15, tint: "cyan", strokeScale: 1, beat: 0.5, rotStep: 1, scaffold: true, removal: "mark", refund: 1, costScale: 1, maxCells: 24, maxRadius: 4, maxStoreys: 3, maxFooting: 2, world: "planned" };
  const TINTS = { cyan: 0x6fd0e8, white: 0xe6eef2, amber: 0xf0b43c };
  const STROKES = { floor: 2, wall: 3, stair: 4, lantern: 1, roof: 2, deck: 2 };

  // ================================================================= kits and costs (design 8)
  const PKITS = {
    straw: { label: "Straw", wall: "reed", inside: "reed", edge: "timber", t: 0.3, floor: "dirt", roof: "thatch", gable: "reed", stair: "plank" },
    timber: { label: "Timber", wall: "boards", inside: "boards", edge: "timber", t: 0.25, floor: "plank", roof: "shingle", gable: "boards", stair: "plank" },
    stone: { label: "Fieldstone", wall: "fieldstone", inside: "plaster", edge: "fieldstone", t: 0.5, floor: "flag", roof: "thatch", gable: "fieldstone", stair: "flag" },
    halftimber: { label: "Half-timbered", wall: "halftimber", lower: "stone", inside: "plaster", edge: "timber", t: 0.3, floor: "flag", roof: "thatch", gable: "halftimber", stair: "plank" },
    brick: { label: "Red brick", locked: "Needs bricks" },
  };
  const COST = {
    straw: { wall: { fibre: 3, log: 1 }, door: { fibre: 2, log: 1, planks: 2, rope: 1 }, window: { fibre: 2, log: 1, planks: 1 }, floor: {}, stair: { planks: 4 }, roof: { fibre: 3, log: 1 } },
    timber: { wall: { planks: 4 }, door: { planks: 5, rope: 1 }, window: { planks: 4 }, floor: { planks: 2 }, stair: { planks: 4 }, roof: { planks: 3 } },
    stone: { wall: { stone: 5, planks: 1 }, door: { stone: 4, planks: 3, rope: 1 }, window: { stone: 4, planks: 2 }, floor: { stone: 3 }, stair: { stone: 4 }, roof: { fibre: 3, log: 1 } },
    halftimber: { wall: { stone: 3, planks: 2 }, door: { stone: 2, planks: 4, rope: 1 }, window: { stone: 2, planks: 3 }, floor: { stone: 3 }, stair: { planks: 4 }, roof: { fibre: 3, log: 1 } },
  };
  const GOODS = { planks: ["Planks", "planks"], stone: ["Stone", "stone"], log: ["Logs", "log"], fibre: ["Fibre", "fibre"], rope: ["Rope", "rope"], lantern: ["Lanterns", "lantern_post"] };
  const goodIcon = g => `building/game/items/${GOODS[g][1]}.png`;
  const BAY = { planks: 2, ropeEvery: 3 };
  const bayCost = bays => ({ planks: BAY.planks * bays, rope: Math.ceil(bays / BAY.ropeEvery) });
  function costOf(p, footing = 0) {
    const base = p.cls === "wall" ? COST[p.kit][p.open === "door" ? "door" : p.open === "window" ? "window" : "wall"]
      : p.cls === "floor" ? COST[p.kit].floor : p.cls === "stair" ? COST[p.kit].stair
      : p.cls === "lantern" ? { lantern: 1 } : p.cls === "roof" ? COST[p.kit].roof : p.cls === "deck" ? { planks: 2 } : {};
    const n = p.cls === "roof" || p.cls === "deck" ? p.cells.length : 1, out = {};
    for (const [g, v] of Object.entries(base)) out[g] = Math.max(1, Math.ceil(v * n * TUNE.costScale));
    if (footing > 0) out.stone = (out.stone || 0) + Math.ceil(2 * footing * TUNE.costScale);
    return out;
  }
  const addTo = (a, b, k = 1) => { for (const [g, v] of Object.entries(b)) a[g] = (a[g] || 0) + v * k; return a; };
  const goodsText = o => Object.entries(o).filter(([, v]) => v > 0).map(([g, v]) => `${v} ${GOODS[g][0].toLowerCase()}`).join(", ");
  const strokesOf = p => Math.max(1, Math.round((p.cls === "roof" || p.cls === "deck" ? STROKES[p.cls] * p.cells.length : STROKES[p.cls] || 1) * TUNE.strokeScale));

  // ================================================================= the plan (design 1)
  function newPlan(kit = "timber") { return { name: "New plan", preset: null, size: 1, kit, roof: "gable", st: [new Set()], edges: {}, stairs: [], lanterns: [], slots: [] }; }
  function clonePlan(p) { return { ...p, st: p.st.map(s => new Set(s)), edges: { ...p.edges }, stairs: p.stairs.map(x => ({ ...x })), lanterns: p.lanterns.map(x => ({ ...x })), slots: p.slots.map(x => ({ ...x })) }; }
  function edgeKey(s, q, r, d) { const [q2, r2] = nbr(q, r, d); return (q < q2 || (q === q2 && r < r2)) ? `${s}|${q},${r}|${d}` : `${s}|${q2},${r2}|${(d + 3) % 6}`; }
  const wells = (p, s) => { const w = new Set(); if (s > 0) for (const t of p.stairs) if (t.s === s - 1) { w.add(K(t.q, t.r)); w.add(K(...nbr(t.q, t.r, t.d))); } return w; };
  const stairCells = t => [K(t.q, t.r), K(...nbr(t.q, t.r, t.d))];
  // The plan's pieces, in plan coordinates, each with an id (its slot and class) and a body signature.
  function piecesOf(p) {
    const out = [];
    p.st.forEach((cells, s) => {
      const wl = wells(p, s);
      for (const k of cells) { const [q, r] = unK(k); if (!wl.has(k)) out.push({ id: `f|${s}|${k}`, cls: "floor", s, q, r, kit: p.kit }); }
      const seen = new Set();
      for (const k of cells) {
        const [q, r] = unK(k);
        for (let d = 0; d < 6; d++) {
          const ek = edgeKey(s, q, r, d); if (seen.has(ek)) continue; seen.add(ek);
          const outer = !cells.has(K(...nbr(q, r, d))), set = p.edges[ek];
          const type = outer ? (set === "door" || set === "window" ? set : "wall") : (set === "wall" || set === "door" ? set : null);
          if (!type) continue;
          out.push({ id: `w|${ek}`, cls: "wall", s, q, r, d, outer, open: type === "wall" ? "none" : type, kit: p.kit });
        }
      }
    });
    for (const t of p.stairs) out.push({ id: `s|${t.s}|${K(t.q, t.r)}|${t.d}`, cls: "stair", s: t.s, q: t.q, r: t.r, d: t.d, kit: p.kit });
    for (const l of p.lanterns) out.push({ id: `l|${l.s}|${K(l.q, l.r)}`, cls: "lantern", s: l.s, q: l.q, r: l.r, kit: p.kit });
    for (const o of p.slots) out.push({ id: `o|${o.s}|${K(o.q, o.r)}`, cls: "slot", s: o.s, q: o.q, r: o.r, d: o.d, kind: o.kind, kit: p.kit });
    const T = p.st.length - 1;
    p.st.forEach((cells, s) => {
      const exp = [...cells].filter(k => s === T || !p.st[s + 1].has(k)).sort();
      if (!exp.length) return;
      if (s === T) { const shape = p.roof === "cone" && exp.length > 1 ? "gable" : p.roof; out.push({ id: `r|${s}`, cls: "roof", s, cells: exp.map(unK), shape, kit: p.kit }); }
      else out.push({ id: `k|${s}`, cls: "deck", s, cells: exp.map(unK), shape: "flat", kit: p.kit });
    });
    for (const x of out) x.sig = [x.kit, x.open || "", x.shape || "", x.cells ? x.cells.map(c => c.join(",")).join(";") : "", x.kind || "", x.d ?? ""].join("|");
    return out;
  }
  const footprint = p => { const u = new Set(); for (const c of p.st) for (const k of c) u.add(k); return u; };

  // ================================================================= the planner's checks (design 2)
  const REASON = {
    held_up: "Nothing holds it up: put a room under it",
    plan_size: "Too big to plan",
    storeys: "Too many storeys",
    landing: "No landing at the top of the stair",
    stair_cut: "The wall would cut through a stair",
    stair_floor: "A stair needs a floor at its foot and head",
    headroom: "No headroom: a stair over a stair",
    slot: "Something is already here",
    opening: "A lantern cannot stand on a stair",
    way_in: "No way in",
    no_stair: "No stair up to this floor",
  };
  function checkPlan(p) {
    const out = [], all = footprint(p);
    p.st.forEach((cells, s) => { if (s === 0) return; const bad = [...cells].filter(k => !p.st[s - 1].has(k)); if (bad.length) out.push({ lvl: "bad", rule: "held_up", s, cells: bad }); });
    const far = [...all].filter(k => hexDist(...unK(k)) > TUNE.maxRadius);
    if (all.size > TUNE.maxCells || far.length) out.push({ lvl: "bad", rule: "plan_size", s: -1, cells: far, note: `${all.size} of ${TUNE.maxCells} cells` });
    if (p.st.length > TUNE.maxStoreys) out.push({ lvl: "bad", rule: "storeys", s: p.st.length - 1, cells: [] });
    const used = new Map();
    for (const t of p.stairs) {
      const [f, h] = stairCells(t), wl = wells(p, t.s), cells = p.st[t.s] || new Set();
      if (!cells.has(f) || !cells.has(h)) out.push({ lvl: "bad", rule: "stair_floor", s: t.s, cells: [f, h] });
      else if (wl.has(f) || wl.has(h)) out.push({ lvl: "bad", rule: "headroom", s: t.s, cells: [f, h] });
      const land = K(...nbr(...unK(h), t.d)), up = p.st[t.s + 1];
      const landWall = up && p.edges[edgeKey(t.s + 1, ...unK(h), t.d)] === "wall";
      if (!up || !up.has(land) || landWall) out.push({ lvl: "bad", rule: "landing", s: t.s, cells: [f, h] });
      const cut = p.edges[edgeKey(t.s, ...unK(f), t.d)];
      if (cut === "wall" || cut === "door") out.push({ lvl: "bad", rule: "stair_cut", s: t.s, cells: [f, h] });
      for (const c of [f, h]) { const key = t.s + "|" + c; if (used.has(key)) out.push({ lvl: "bad", rule: "slot", s: t.s, cells: [c] }); used.set(key, "stair"); }
    }
    for (const l of p.lanterns) { const key = l.s + "|" + K(l.q, l.r); if (used.get(key) === "stair") out.push({ lvl: "bad", rule: "opening", s: l.s, cells: [K(l.q, l.r)] }); else if (wells(p, l.s).has(K(l.q, l.r))) out.push({ lvl: "bad", rule: "held_up", s: l.s, cells: [K(l.q, l.r)] }); }
    for (const o of p.slots) { const key = o.s + "|" + K(o.q, o.r); if (used.has(key) || wells(p, o.s).has(K(o.q, o.r))) out.push({ lvl: "bad", rule: "slot", s: o.s, cells: [K(o.q, o.r)] }); used.set(key, "slot"); }
    if (!piecesOf(p).some(x => x.cls === "wall" && x.s === 0 && x.open === "door")) out.push({ lvl: "warn", rule: "way_in", s: 0, cells: [] });
    for (let s = 1; s < p.st.length; s++) if (!p.stairs.some(t => t.s === s - 1)) out.push({ lvl: "warn", rule: "no_stair", s, cells: [] });
    return out;
  }
  const planOK = p => p.st[0].size > 0 && !checkPlan(p).some(i => i.lvl === "bad");

  // ================================================================= presets (design 4)
  function centred(cells) {
    let mx = 0, mz = 0; for (const [q, r] of cells) { mx += hx(q, r); mz += hz(q, r); } mx /= cells.length; mz /= cells.length;
    const [dq, dr] = cellAt(mx, mz); return cells.map(([q, r]) => [q - dq, r - dr]);
  }
  const rect = (w, h) => { const c = []; for (let r = 0; r < h; r++) for (let q = 0; q < w; q++) c.push([q - Math.floor(r / 2), r]); return centred(c); };
  function blob(n) { const c = [[0, 0]]; for (let rad = 1; c.length < n; rad++) { let [q, r] = [DIRS[4][0] * rad, DIRS[4][1] * rad]; for (let d = 0; d < 6 && c.length < n; d++) for (let i = 0; i < rad && c.length < n; i++) { c.push([q, r]); [q, r] = nbr(q, r, d); } } return c; }
  function autoStairs(p) {
    p.stairs = [];
    for (let s = 0; s + 1 < p.st.length; s++) {
      const wl = wells(p, s), cells = [...p.st[s]].sort();
      let done = false;
      for (const k of cells) { if (done) break; const [q, r] = unK(k); for (const d of [0, 3, 1, 4, 2, 5]) {
        const h = K(...nbr(q, r, d)), land = K(...nbr(...nbr(q, r, d), d));
        if (wl.has(k) || wl.has(h) || !p.st[s].has(h) || !p.st[s + 1].has(land) || !p.st[s + 1].has(k) || !p.st[s + 1].has(h)) continue;
        p.stairs.push({ q, r, d, s }); done = true; break; } }
    }
  }
  function outerEdges(p, s) {
    const e = [], cells = p.st[s]; let cx = 0, cz = 0; for (const k of cells) { const [q, r] = unK(k); cx += hx(q, r); cz += hz(q, r); } cx /= cells.size || 1; cz /= cells.size || 1;
    for (const k of cells) { const [q, r] = unK(k); for (let d = 0; d < 6; d++) if (!cells.has(K(...nbr(q, r, d)))) { const a = corner(q, r, d), b = corner(q, r, d + 1); e.push({ q, r, d, key: edgeKey(s, q, r, d), ang: Math.atan2((a[1] + b[1]) / 2 - cz, (a[0] + b[0]) / 2 - cx), x: (a[0] + b[0]) / 2, z: (a[1] + b[1]) / 2 }); } }
    return e.sort((a, b) => a.ang - b.ang);
  }
  // A preset: footprint and storeys by size, then its doors, windows, lanterns and slots laid out by rule.
  const PRESETS = [
    { id: "hut", label: "Straw hut", kit: "straw", roof: "cone", cells: z => [blob(1), blob(1), rect(2, 1)][z], storeys: () => 1, doors: 1, win: 0, slots: ["bed"] },
    { id: "cottage", label: "Cottage", kit: "timber", roof: "gable", cells: z => [rect(2, 1), rect(3, 1), rect(3, 2)][z], storeys: () => 1, doors: 1, win: 3, slots: ["table", "bed", "chair"] },
    { id: "house", label: "Two-storey house", kit: "stone", roof: "gable", cells: z => [rect(3, 1), rect(3, 2), rect(4, 2)][z], storeys: () => 2, doors: 1, win: 3, slots: ["table", "chair", "bed", "chest"] },
    { id: "townhouse", label: "Townhouse", kit: "halftimber", roof: "gable", cells: z => [rect(3, 2), rect(3, 2), rect(4, 2)][z], storeys: z => [2, 3, 3][z], doors: 1, win: 2, slots: ["table", "bed", "bed"] },
    { id: "workshop", label: "Workshop", kit: "timber", roof: "gable", cells: z => [rect(2, 2), rect(3, 2), rect(4, 2)][z], storeys: () => 1, doors: 2, win: 3, slots: ["bench", "chest"] },
    { id: "shop", label: "Shop", kit: "halftimber", roof: "gable", cells: z => [rect(3, 1), rect(3, 2), rect(4, 2)][z], storeys: () => 1, doors: 1, win: 2, slots: ["table", "chest"] },
    { id: "barn", label: "Barn", kit: "timber", roof: "gable", cells: z => [rect(3, 2), rect(4, 2), rect(4, 3)][z], storeys: () => 1, doors: 2, win: 5, slots: ["chest"] },
    { id: "stable", label: "Stable", kit: "timber", roof: "gable", cells: z => [rect(3, 1), rect(4, 1), rect(4, 2)][z], storeys: () => 1, doors: 2, win: 4, slots: ["bench"] },
    { id: "warehouse", label: "Warehouse", kit: "stone", roof: "gable", cells: z => [rect(3, 2), rect(4, 3), rect(5, 3)][z], storeys: () => 1, doors: 2, win: 6, slots: ["chest", "chest", "chest"] },
    { id: "tower", label: "Tower", kit: "stone", roof: "flat", cells: z => [blob(7), rect(3, 3), rect(4, 3)][z], storeys: z => [2, 3, 3][z], doors: 1, win: 2, slots: ["bed"] },
  ];
  function makePreset(pr, z) {
    const p = newPlan(pr.kit); p.name = pr.label + ["", "", ""][z]; p.preset = pr.id; p.size = z; p.roof = pr.roof;
    const cells = pr.cells(z), n = pr.storeys(z); p.st = [];
    for (let s = 0; s < n; s++) p.st.push(new Set(cells.map(c => K(...c))));
    autoStairs(p);
    // doors on the front (the edges facing +z), the middle one, two side by side for a wide door
    const oe = outerEdges(p, 0), front = oe.filter(e => e.d === 1 || e.d === 2).sort((a, b) => a.x - b.x);
    const mid = Math.floor((front.length - 1) / 2), doors = front.slice(mid, mid + pr.doors);
    const doorKeys = new Set(doors.map(e => e.key)); for (const k of doorKeys) p.edges[k] = "door";
    for (let s = 0; s < n; s++) {
      const es = outerEdges(p, s).filter(e => !doorKeys.has(e.key)); if (!pr.win) continue;
      const step = Math.max(1, Math.floor(es.length / pr.win));
      for (let i = Math.floor(step / 2), c = 0; i < es.length && c < pr.win; i += step, c++) p.edges[es[i].key] = "window";
    }
    // lanterns and slots on free floor cells, the slots facing the room's middle
    for (let s = 0; s < n; s++) {
      const taken = new Set([...wells(p, s)]); for (const t of p.stairs) if (t.s === s) for (const c of stairCells(t)) taken.add(c);
      const free = [...p.st[s]].filter(k => !taken.has(k)).sort((a, b) => hexDist(...unK(a)) - hexDist(...unK(b)) || (a < b ? -1 : 1));
      if (free.length) { const [q, r] = unK(free[0]); p.lanterns.push({ q, r, s }); taken.add(free[0]); }
      const want = pr.slots.filter((k, i) => n === 1 || (k === "bed" ? s === n - 1 : s === 0) || (k === "bed" && i > 0 && s > 0));
      const spots = free.filter(k => !taken.has(k)).reverse();
      want.forEach((kind, i) => { const k = spots[i]; if (!k) return; const [q, r] = unK(k); p.slots.push({ q, r, s, kind, d: (Math.round(Math.atan2(-hz(q, r), -hx(q, r)) / (Math.PI / 3)) + 6) % 6 }); });
    }
    return p;
  }
  // the barn's and the warehouse's slots must not pile on one cell: keep only one per cell
  function tidy(p) { const seen = new Set(); p.slots = p.slots.filter(o => { const k = o.s + "|" + K(o.q, o.r); if (seen.has(k)) return false; seen.add(k); return true; }); return p; }

  // ================================================================= the planner screen
  const PL = { plan: tidy(makePreset(PRESETS[1], 1)), storey: 0, tool: "room", slotKind: "bed", undo: [], edit: null, presetSize: 1, paint: null };
  let MINE = [];
  try { const j = JSON.parse(localStorage.getItem("pbd-m6-blueprints") || "[]"); MINE = j.map(o => ({ name: o.name, plan: fromJSON(o.plan) })); } catch (e) { MINE = []; }
  function toJSON(p) { return { ...p, st: p.st.map(s => [...s]) }; }
  function fromJSON(o) { return { ...o, st: o.st.map(s => new Set(s)) }; }
  function saveMine() { try { localStorage.setItem("pbd-m6-blueprints", JSON.stringify(MINE.map(m => ({ name: m.name, plan: toJSON(m.plan) })))); } catch (e) { /* private window: kept for this visit only */ } }

  const TOOLS = [["room", "Room", "building/icon_floor.png"], ["wall", "Wall", "building/icon_wall.png"], ["door", "Door", "building/icon_door.png"], ["window", "Window", "building/icon_window.png"], ["stair", "Stair", "building/icon_stairs.png"], ["lantern", "Lantern", "building/game/items/lantern_post.png"], ["slot", "Slot", "building/icon_post.png"], ["erase", "Erase", "building/game/items/pickaxe.png"]];
  const SLOTKINDS = ["bed", "table", "chest", "bench", "chair"];
  const pcv = $("pcv"), pctx = pcv.getContext("2d");
  let PV = { s: 30, cx: 0, cy: 0 };
  function pfit() {
    const w = pcv.clientWidth, h = pcv.clientHeight, dpr = Math.min(2, devicePixelRatio || 1);
    pcv.width = Math.round(w * dpr); pcv.height = Math.round(h * dpr); pctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    let ext = 2.6; for (const k of footprint(PL.plan)) ext = Math.max(ext, hexDist(...unK(k)) + 1.6); ext = Math.min(ext, TUNE.maxRadius + 1.2);
    const top = (document.querySelector(".ptop").offsetHeight || 40) + 16, bot = 96, availH = Math.max(120, h - top - bot);
    PV.s = Math.min(w / (2 * ext * W + W), availH / (2 * ext * ROW + 2 * R)); PV.cx = w / 2; PV.cy = top + availH / 2;
  }
  const P2 = (x, z) => [PV.cx + x * PV.s, PV.cy + z * PV.s];
  const fromScreen = (sx, sy) => [(sx - PV.cx) / PV.s, (sy - PV.cy) / PV.s];
  function hexPath(q, r, sc = 1) { pctx.beginPath(); hexPts(q, r, sc).forEach((p, i) => { const [x, y] = P2(...p); i ? pctx.lineTo(x, y) : pctx.moveTo(x, y); }); pctx.closePath(); }
  function edgeLine(q, r, d) { const a = P2(...corner(q, r, d)), b = P2(...corner(q, r, d + 1)); return [a, b]; }
  let HOVER = null;
  function pdraw() {
    const w = pcv.clientWidth, h = pcv.clientHeight, p = PL.plan, s = PL.storey, issues = checkPlan(p);
    pctx.clearRect(0, 0, w, h);
    // drafting paper
    pctx.fillStyle = "#11293a"; pctx.fillRect(0, 0, w, h);
    const lim = TUNE.maxRadius;
    for (let q = -lim - 2; q <= lim + 2; q++) for (let r = -lim - 2; r <= lim + 2; r++) {
      const dd = hexDist(q, r); if (dd > lim + 1) continue;
      hexPath(q, r, 0.97); pctx.strokeStyle = dd > lim ? "rgba(255,107,87,.18)" : "rgba(143,195,216,.16)"; pctx.lineWidth = 1; pctx.stroke();
    }
    // the storey below, faint
    if (s > 0) for (const k of p.st[s - 1]) { hexPath(...unK(k), 0.94); pctx.fillStyle = "rgba(143,195,216,.10)"; pctx.fill(); }
    const wl = wells(p, s);
    for (const k of p.st[s] || []) {
      const [q, r] = unK(k); hexPath(q, r, 0.98);
      pctx.fillStyle = wl.has(k) ? "rgba(20,40,55,.9)" : "rgba(196,228,240,.26)"; pctx.fill();
      if (wl.has(k)) { pctx.save(); hexPath(q, r, 0.9); pctx.clip(); pctx.strokeStyle = "rgba(143,195,216,.35)"; pctx.lineWidth = 1; const [x, y] = P2(hx(q, r), hz(q, r)); for (let i = -4; i <= 4; i++) { pctx.beginPath(); pctx.moveTo(x - 40 + i * 6, y - 40); pctx.lineTo(x + 40 + i * 6, y + 40); pctx.stroke(); } pctx.restore(); }
    }
    // issues on cells
    for (const is of issues) if (is.s === s || is.s === -1) for (const k of is.cells) { hexPath(...unK(k), 0.9); pctx.fillStyle = is.lvl === "bad" ? "rgba(255,107,87,.38)" : "rgba(240,180,60,.3)"; pctx.fill(); }
    // walls of this storey
    const kit = PKITS[p.kit], wt = Math.max(3, (kit.t || 0.3) * PV.s);
    for (const pc of piecesOf(p)) {
      if (pc.cls !== "wall" || pc.s !== s) continue;
      const [a, b] = edgeLine(pc.q, pc.r, pc.d);
      pctx.lineCap = "round";
      if (pc.open === "none") { pctx.strokeStyle = pc.outer ? "#e2d6b6" : "#bfb08c"; pctx.lineWidth = pc.outer ? wt : wt * 0.7; pctx.beginPath(); pctx.moveTo(...a); pctx.lineTo(...b); pctx.stroke(); }
      else {
        const L = Math.hypot(b[0] - a[0], b[1] - a[1]), ux = (b[0] - a[0]) / L, uy = (b[1] - a[1]) / L, gap = (pc.open === "door" ? 1.0 : 0.8) / 1.636 * L;
        const g0 = [a[0] + ux * (L - gap) / 2, a[1] + uy * (L - gap) / 2], g1 = [a[0] + ux * (L + gap) / 2, a[1] + uy * (L + gap) / 2];
        pctx.strokeStyle = "#e2d6b6"; pctx.lineWidth = pc.outer ? wt : wt * 0.7; pctx.beginPath(); pctx.moveTo(...a); pctx.lineTo(...g0); pctx.moveTo(...g1); pctx.lineTo(...b); pctx.stroke();
        if (pc.open === "window") { pctx.strokeStyle = "#8fc3d8"; pctx.lineWidth = 2; pctx.beginPath(); pctx.moveTo(...g0); pctx.lineTo(...g1); pctx.stroke(); }
        else { const [nx, ny] = dvec(pc.d), a0 = Math.atan2(uy, ux), a1 = Math.atan2(-ny, -nx), dd = ((a1 - a0 + 3 * Math.PI) % (2 * Math.PI)) - Math.PI; pctx.strokeStyle = "#f0b43c"; pctx.lineWidth = 1.5; pctx.beginPath(); pctx.moveTo(...g0); pctx.arc(g0[0], g0[1], gap, a0, a0 + dd, dd < 0); pctx.closePath(); pctx.stroke(); }
      }
    }
    // stairs, lanterns, slots
    for (const t of p.stairs) if (t.s === s) {
      const [q2, r2] = nbr(t.q, t.r, t.d), a = P2(hx(t.q, t.r), hz(t.q, t.r)), b = P2(hx(q2, r2), hz(q2, r2)), [ux, uy] = dvec(t.d), px = -uy, py = ux;
      const bad = issues.some(i => i.lvl === "bad" && i.s === s && i.cells.includes(K(t.q, t.r)) && ["landing", "stair_cut", "stair_floor", "headroom"].includes(i.rule));
      pctx.strokeStyle = bad ? "#ff6b57" : "#ece4d2"; pctx.lineWidth = 1.5; const hw = 0.55 * PV.s;
      for (let i = 0; i <= 10; i++) { const f = -0.35 + i * 0.17, x = a[0] + (b[0] - a[0]) * f, y = a[1] + (b[1] - a[1]) * f; pctx.beginPath(); pctx.moveTo(x + px * hw, y + py * hw); pctx.lineTo(x - px * hw, y - py * hw); pctx.stroke(); }
      pctx.lineWidth = 2.5; pctx.beginPath(); pctx.moveTo(a[0] - ux * 0.3 * PV.s, a[1] - uy * 0.3 * PV.s); pctx.lineTo(b[0] + ux * 0.6 * PV.s, b[1] + uy * 0.6 * PV.s); pctx.stroke();
      const tip = [b[0] + ux * 0.9 * PV.s, b[1] + uy * 0.9 * PV.s]; pctx.fillStyle = pctx.strokeStyle; pctx.beginPath(); pctx.moveTo(...tip); pctx.lineTo(tip[0] - ux * 9 + px * 6, tip[1] - uy * 9 + py * 6); pctx.lineTo(tip[0] - ux * 9 - px * 6, tip[1] - uy * 9 - py * 6); pctx.fill();
    }
    if (s > 0) for (const t of p.stairs) if (t.s === s - 1) { const [q2, r2] = nbr(t.q, t.r, t.d), b = P2(hx(q2, r2), hz(q2, r2)); pctx.fillStyle = "rgba(236,228,210,.6)"; pctx.font = "600 12px Barlow Condensed, sans-serif"; pctx.textAlign = "center"; pctx.fillText("STAIR UP", b[0], b[1] - PV.s * 0.3); }
    for (const l of p.lanterns) if (l.s === s) { const [x, y] = P2(hx(l.q, l.r), hz(l.q, l.r)); const g = pctx.createRadialGradient(x, y, 0, x, y, PV.s * 1.1); g.addColorStop(0, "rgba(255,200,120,.75)"); g.addColorStop(1, "rgba(255,200,120,0)"); pctx.fillStyle = g; pctx.beginPath(); pctx.arc(x, y, PV.s * 1.1, 0, 7); pctx.fill(); pctx.fillStyle = "#ffd690"; pctx.beginPath(); pctx.arc(x, y, Math.max(3, PV.s * 0.12), 0, 7); pctx.fill(); }
    for (const o of p.slots) if (o.s === s) {
      const [x, y] = P2(hx(o.q, o.r), hz(o.q, o.r)), bad = issues.some(i => i.s === s && i.rule === "slot" && i.cells.includes(K(o.q, o.r)));
      const lab = o.kind.toUpperCase(); pctx.font = "600 12px Barlow Condensed, sans-serif"; const tw = pctx.measureText(lab).width + 10;
      pctx.fillStyle = bad ? "rgba(255,107,87,.85)" : "rgba(240,180,60,.85)"; pctx.fillRect(x - tw / 2, y + PV.s * 0.25, tw, 16); pctx.fillStyle = "#17150f"; pctx.textAlign = "center"; pctx.fillText(lab, x, y + PV.s * 0.25 + 12);
      const [fx, fy] = dvec(o.d); pctx.strokeStyle = bad ? "#ff6b57" : "#f0b43c"; pctx.lineWidth = 2; pctx.beginPath(); pctx.moveTo(x, y); pctx.lineTo(x + fx * PV.s * 0.45, y + fy * PV.s * 0.45); pctx.stroke();
    }
    // hover
    if (HOVER) {
      pctx.strokeStyle = "#f0b43c"; pctx.lineWidth = 2;
      if (HOVER.edge != null) { const [a, b] = edgeLine(HOVER.q, HOVER.r, HOVER.edge); pctx.lineWidth = 5; pctx.beginPath(); pctx.moveTo(...a); pctx.lineTo(...b); pctx.stroke(); }
      else { hexPath(HOVER.q, HOVER.r, 0.92); pctx.stroke(); if (PL.tool === "stair") { const a = P2(hx(HOVER.q, HOVER.r), hz(HOVER.q, HOVER.r)), [ux, uy] = dvec(HOVER.d); pctx.beginPath(); pctx.moveTo(...a); pctx.lineTo(a[0] + ux * W * 1.5 * PV.s, a[1] + uy * W * 1.5 * PV.s); pctx.stroke(); } }
    }
    // the compass of the plan: which way is the front
    pctx.fillStyle = "rgba(236,228,210,.55)"; pctx.font = "600 12px Barlow Condensed, sans-serif"; pctx.textAlign = "center"; pctx.fillText("FRONT", PV.cx, Math.min(h - 84, PV.cy + (TUNE.maxRadius + 1.1) * ROW * PV.s));
  }
  function target(ev) {
    const rc = pcv.getBoundingClientRect(), [x, z] = fromScreen(ev.clientX - rc.left, ev.clientY - rc.top), [q, r] = cellAt(x, z);
    const dx = x - hx(q, r), dz = z - hz(q, r), ang = Math.atan2(dz, dx), d = ((Math.round(ang / (Math.PI / 3)) % 6) + 6) % 6, rad = Math.hypot(dx, dz);
    const edgeTool = ["wall", "door", "window"].includes(PL.tool) || (PL.tool === "erase" && rad > R * 0.62);
    return { q, r, d, edge: edgeTool ? d : null };
  }
  function markEdited() { if (PL.plan.preset && !PL.edit) { PL.plan.preset = null; PL.plan.name = PL.plan.name + " (changed)"; } }
  function pushUndo() { PL.undo.push(clonePlan(PL.plan)); if (PL.undo.length > 60) PL.undo.shift(); }
  function prune(p) {
    p.stairs = p.stairs.filter(t => p.st[t.s]); p.lanterns = p.lanterns.filter(l => p.st[l.s] && p.st[l.s].has(K(l.q, l.r))); p.slots = p.slots.filter(o => p.st[o.s] && p.st[o.s].has(K(o.q, o.r)));
  }
  function applyTool(t, drag) {
    const p = PL.plan, s = PL.storey, k = K(t.q, t.r), cells = p.st[s];
    if (PL.tool === "room") {
      if (!drag) PL.paint = cells.has(k) ? "erase" : "add";
      if (PL.paint === "add") cells.add(k); else cells.delete(k);
      prune(p); return;
    }
    if (PL.tool === "wall" || PL.tool === "door" || PL.tool === "window") {
      if (!cells.has(k)) return; const ek = edgeKey(s, t.q, t.r, t.edge), outer = !cells.has(K(...nbr(t.q, t.r, t.edge))), cur = p.edges[ek];
      if (PL.tool === "wall") { if (outer) delete p.edges[ek]; else p.edges[ek] = cur === "wall" ? undefined : "wall"; }
      else if (PL.tool === "door") p.edges[ek] = cur === "door" ? (outer ? undefined : "wall") : "door";
      else if (outer) p.edges[ek] = cur === "window" ? undefined : "window";
      if (p.edges[ek] === undefined) delete p.edges[ek];
      return;
    }
    if (PL.tool === "stair") { const i = p.stairs.findIndex(x => x.s === s && K(x.q, x.r) === k); if (i >= 0 && p.stairs[i].d === t.d) p.stairs.splice(i, 1); else { if (i >= 0) p.stairs.splice(i, 1); p.stairs.push({ q: t.q, r: t.r, d: t.d, s }); } return; }
    if (PL.tool === "lantern") { if (!cells.has(k)) return; const i = p.lanterns.findIndex(x => x.s === s && K(x.q, x.r) === k); if (i >= 0) p.lanterns.splice(i, 1); else p.lanterns.push({ q: t.q, r: t.r, s }); return; }
    if (PL.tool === "slot") { if (!cells.has(k)) return; const i = p.slots.findIndex(x => x.s === s && K(x.q, x.r) === k); if (i >= 0) p.slots.splice(i, 1); else p.slots.push({ q: t.q, r: t.r, s, kind: PL.slotKind, d: t.d }); return; }
    if (PL.tool === "erase") {
      if (t.edge != null) { delete p.edges[edgeKey(s, t.q, t.r, t.edge)]; return; }
      const before = p.stairs.length + p.lanterns.length + p.slots.length;
      p.stairs = p.stairs.filter(x => !(x.s === s && K(x.q, x.r) === k)); p.lanterns = p.lanterns.filter(x => !(x.s === s && K(x.q, x.r) === k)); p.slots = p.slots.filter(x => !(x.s === s && K(x.q, x.r) === k));
      if (before === p.stairs.length + p.lanterns.length + p.slots.length) { cells.delete(k); prune(p); }
    }
  }
  function hoverText(t) {
    if (!t) return "Pick a tool, click the plan";
    const p = PL.plan, cells = p.st[PL.storey], has = cells.has(K(t.q, t.r));
    const tool = TOOLS.find(x => x[0] === PL.tool)[1];
    if (PL.tool === "room") return has ? "Click to take this cell out of the room" : (hexDist(t.q, t.r) > TUNE.maxRadius ? REASON.plan_size : "Click or drag to paint the room");
    if (PL.tool === "window" && t.edge != null && cells.has(K(...nbr(t.q, t.r, t.edge)))) return "A window goes in an outside wall";
    if (["wall", "door", "window", "lantern", "slot"].includes(PL.tool) && !has) return "Paint a room here first";
    if (PL.tool === "stair") return "Stair up, the way you point from the cell's middle";
    if (PL.tool === "slot") return `A ${PL.slotKind} slot, facing the way you point`;
    return tool;
  }
  {
    let down = false;
    pcv.addEventListener("pointerdown", e => { pcv.setPointerCapture(e.pointerId); down = true; pushUndo(); markEdited(); applyTool(target(e), false); prefresh(); });
    pcv.addEventListener("pointermove", e => { const t = target(e); HOVER = t; if (down && PL.tool === "room") { applyTool(t, true); prefresh(); } else { $("ppill").textContent = hoverText(t); pdraw(); } });
    pcv.addEventListener("pointerup", () => { down = false; PL.paint = null; });
    pcv.addEventListener("pointerleave", () => { HOVER = null; pdraw(); });
    addEventListener("keydown", e => { if (TAB !== "planner" || (e.target.closest && e.target.closest("input"))) return; if ((e.ctrlKey || e.metaKey) && e.key === "z") { const u = PL.undo.pop(); if (u) { PL.plan = u; prefresh(); } e.preventDefault(); } });
  }
  function thumb(cv, p) {
    const c = cv.getContext("2d"), w = cv.width = 88, h = cv.height = 68; c.fillStyle = "#11293a"; c.fillRect(0, 0, w, h);
    const fp = [...footprint(p)].map(unK); if (!fp.length) return; let x0 = 1e9, x1 = -1e9, z0 = 1e9, z1 = -1e9;
    for (const [q, r] of fp) for (const [x, z] of hexPts(q, r)) { x0 = Math.min(x0, x); x1 = Math.max(x1, x); z0 = Math.min(z0, z); z1 = Math.max(z1, z); }
    const sc = Math.min((w - 10) / (x1 - x0), (h - 10) / (z1 - z0)), ox = w / 2 - (x0 + x1) / 2 * sc, oz = h / 2 - (z0 + z1) / 2 * sc;
    for (const [q, r] of fp) { c.beginPath(); hexPts(q, r, 0.96).forEach(([x, z], i) => i ? c.lineTo(ox + x * sc, oz + z * sc) : c.moveTo(ox + x * sc, oz + z * sc)); c.closePath(); const tall = p.st.filter(s => s.has(K(q, r))).length; c.fillStyle = ["", "rgba(196,228,240,.45)", "rgba(196,228,240,.7)", "rgba(226,240,246,.9)", "#fff"][Math.min(4, tall)]; c.fill(); }
  }
  function renderLib() {
    $("presets").innerHTML = PRESETS.map(pr => { const p = makePreset(pr, PL.presetSize); return `<button data-pr="${pr.id}" aria-pressed="${!PL.edit && PL.plan.preset === pr.id && PL.plan.size === PL.presetSize && !PL.plan.mine}"><canvas></canvas><span>${pr.label}<small>${footprint(p).size} cells &middot; ${p.st.length} storey${p.st.length > 1 ? "s" : ""}</small></span></button>`; }).join("");
    [...$("presets").children].forEach((b, i) => { thumb(b.querySelector("canvas"), makePreset(PRESETS[i], PL.presetSize)); b.addEventListener("click", () => loadPlan(tidy(makePreset(PRESETS[i], PL.presetSize)))); });
    $("mine").innerHTML = MINE.length ? MINE.map((m, i) => `<button data-mi="${i}" aria-pressed="${PL.plan.mine === m.name}"><canvas></canvas><span>${esc(m.name)}<small>${footprint(m.plan).size} cells &middot; ${PKITS[m.plan.kit].label}</small></span></button>`).join("") : `<div class="empty">None yet. Change a preset and save it.</div>`;
    [...$("mine").querySelectorAll("button")].forEach(b => { const m = MINE[+b.dataset.mi]; thumb(b.querySelector("canvas"), m.plan); b.addEventListener("click", () => { const p = clonePlan(m.plan); p.mine = m.name; loadPlan(p); }); });
    for (const b of $("psize").children) b.setAttribute("aria-pressed", +b.dataset.sz === PL.presetSize);
  }
  const esc = t => String(t).replace(/[&<>"]/g, c => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
  function loadPlan(p) { if (PL.edit) { const keep = PL.plan.name; p = clonePlan(p); p.name = keep; } pushUndo(); PL.plan = p; PL.storey = Math.min(PL.storey, p.st.length - 1); prefresh(); }
  function renderTop() {
    const p = PL.plan;
    $("storeys").innerHTML = p.st.map((_, s) => `<button class="btn" data-s="${s}" aria-pressed="${s === PL.storey}">${s === 0 ? "Ground" : "Floor " + s}</button>`).join("") + `<button class="btn" id="addS" aria-label="Add a storey">+</button><button class="btn" id="delS" aria-label="Take off the top storey"${p.st.length < 2 ? " disabled" : ""}>&minus;</button>`;
    for (const b of $("storeys").querySelectorAll("[data-s]")) b.addEventListener("click", () => { PL.storey = +b.dataset.s; prefresh(); });
    $("addS").addEventListener("click", () => { pushUndo(); markEdited(); p.st.push(new Set(p.st[p.st.length - 1])); PL.storey = p.st.length - 1; prefresh(); });
    $("delS").addEventListener("click", () => { if (p.st.length < 2) return; pushUndo(); markEdited(); p.st.pop(); prune(p); p.stairs = p.stairs.filter(t => t.s < p.st.length); PL.storey = Math.min(PL.storey, p.st.length - 1); prefresh(); });
    $("pkits").innerHTML = Object.entries(PKITS).map(([k, v]) => `<button class="btn" data-k="${k}" aria-pressed="${p.kit === k}"${v.locked ? ` disabled title="${v.locked}"` : ""}>${v.label}</button>`).join("");
    for (const b of $("pkits").querySelectorAll("button:not([disabled])")) b.addEventListener("click", () => { pushUndo(); markEdited(); p.kit = b.dataset.k; prefresh(); });
    $("proofs").innerHTML = [["gable", "Gable"], ["cone", "Cone"], ["flat", "Flat"]].map(([k, l]) => `<button class="btn" data-r="${k}" aria-pressed="${p.roof === k}">${l}</button>`).join("");
    for (const b of $("proofs").children) b.addEventListener("click", () => { pushUndo(); markEdited(); p.roof = b.dataset.r; prefresh(); });
  }
  function bomHTML(need, have) {
    const rows = Object.entries(need).filter(([, v]) => v > 0).map(([g, v]) => { const hv = have ? have[g] || 0 : null; return `<div class="row"><img src="${goodIcon(g)}" alt=""><span>${GOODS[g][0]}</span><span class="n${hv != null && hv < v ? " short" : ""}">${v}</span>${hv != null ? `<span class="have">${hv} in pack</span>` : "<span></span>"}</div>`; });
    return rows.join("") || `<div class="sum">Nothing to pay</div>`;
  }
  function planBill(p) { const need = {}; let strokes = 0, n = 0; for (const pc of piecesOf(p)) { if (pc.cls === "slot") continue; addTo(need, costOf(pc)); strokes += strokesOf(pc); n++; } return { need, strokes, n }; }
  function prefresh() {
    const p = PL.plan, issues = checkPlan(p);
    renderTop(); renderLib(); pfit(); pdraw();
    $("checks").innerHTML = (issues.length ? issues.map(i => `<div class="${i.lvl}">${i.s >= 0 ? (i.s === 0 ? "Ground: " : `Floor ${i.s}: `) : ""}${REASON[i.rule]}${i.note ? ` (${i.note})` : ""}</div>`).join("") : "") + (!issues.some(i => i.lvl === "bad") ? `<div class="good">${p.st[0].size ? "Every check passes" : "Paint a room to start"}</div>` : "");
    const b = planBill(p);
    $("bom").innerHTML = bomHTML(b.need) + `<div class="sum">${b.n} pieces &middot; ${b.strokes} mallet strokes, about ${Math.max(1, Math.round(b.strokes * TUNE.beat / 60))} min of building &middot; ${p.slots.length} furniture slot${p.slots.length === 1 ? "" : "s"}</div>`;
    $("setout").hidden = !!PL.edit; $("setout").disabled = !planOK(p);
    $("changeF").hidden = !PL.edit; $("editbar").hidden = !PL.edit;
    if (PL.edit) { const d = diffPlan(SITE, p); $("change").innerHTML = changeHTML(d); $("editbar").textContent = `Changing the site: ${SITE.plan.name}, ${builtCount(SITE)} of ${piecesOf(SITE.plan).filter(x => x.cls !== "slot").length} built`; $("csave").disabled = !planOK(p); }
    $("ptools").innerHTML = TOOLS.map(([k, l, ic], i) => `<button data-t="${k}" aria-pressed="${PL.tool === k}" title="${l}" aria-label="${l}"><span class="k">${i + 1}</span><img src="${ic}" width="32" height="32" alt=""></button>`).join("") + (PL.tool === "slot" ? `<button id="slotk" title="Slot kind" aria-label="Slot kind" style="flex-basis:64px;width:64px;font:600 12px var(--display);color:#f0b43c">${PL.slotKind.toUpperCase()}</button>` : "");
    for (const bt of $("ptools").querySelectorAll("[data-t]")) bt.addEventListener("click", () => { PL.tool = bt.dataset.t; prefresh(); });
    if ($("slotk")) $("slotk").addEventListener("click", () => { PL.slotKind = SLOTKINDS[(SLOTKINDS.indexOf(PL.slotKind) + 1) % SLOTKINDS.length]; prefresh(); });
    $("ppill").textContent = HOVER ? hoverText(HOVER) : "Pick a tool, click the plan";
    $("pname").placeholder = PL.plan.mine || "Name";
  }
  for (const b of $("psize").children) b.addEventListener("click", () => { PL.presetSize = +b.dataset.sz; const pr = PRESETS.find(x => x.id === PL.plan.preset); if (pr && !PL.plan.mine) loadPlan(tidy(makePreset(pr, PL.presetSize))); else prefresh(); });
  $("psave").addEventListener("click", () => { const name = ($("pname").value || "").trim() || (PL.plan.mine ? PL.plan.mine : `My ${PL.plan.name.toLowerCase()}`); const p = clonePlan(PL.plan); p.name = name; p.mine = name; const i = MINE.findIndex(m => m.name === name); if (i >= 0) MINE[i].plan = p; else MINE.push({ name, plan: p }); PL.plan.mine = name; PL.plan.name = name; $("pname").value = ""; saveMine(); prefresh(); refreshPlanSelect(); });
  $("setout").addEventListener("click", () => { newSite(clonePlan(PL.plan)); openTab("site"); });
  $("csave").addEventListener("click", () => { const res = applyPlan(SITE, clonePlan(PL.plan)); if (!res.ok) { $("ppill").textContent = res.reason; return; } PL.edit = null; openTab("site"); });
  $("ccancel").addEventListener("click", () => { PL.edit = null; openTab("site"); });
  addEventListener("keydown", e => { if (TAB !== "planner" || (e.target.closest && e.target.closest("input"))) return; const n = "12345678".indexOf(e.key); if (n >= 0 && !e.ctrlKey) { PL.tool = TOOLS[n][0]; prefresh(); } });

  // ================================================================= the world the site is in
  const WR = 15;
  function groundH(q, r) { const x = hx(q, r), z = hz(q, r); let h = 0; if (x > 8) h = 1; if (x > 12.5) h = 2; if (x > 17) h = 3; if (z > 9 && x < -2) h -= 1; return h; }
  const TREES = new Set(["-3,4", "4,-5", "6,-1", "-9,6", "9,3", "-2,-6", "11,-7", "-12,4", "3,7", "-6,9", "13,-2", "1,-9", "-10,-2", "8,6", "-14,9", "14,-9", "5,9", "-5,-8"].filter(k => true));
  // the neighbour's finished house, the village lane and its lamps
  const NB = { plan: tidy(makePreset(PRESETS[2], 0)), origin: [-7, 1], turn: 0 };
  const LANE = []; for (let q = -12; q <= 12; q++) LANE.push(K(q, -4 - (q < -4 ? 0 : 0)));
  const LANESET = new Set(LANE);
  const VLAMPS = [[-4, -3], [3, -3], [-9, -3]];
  const WORKLAMP = [3, 1];
  function worldCells(plan, origin, turn) { const m = new Map(); for (const k of footprint(plan)) { const [q, r] = rotN(unK(k), turn); m.set(k, [q + origin[0], r + origin[1]]); } return m; }
  const NBCELLS = new Set([...worldCells(NB.plan, NB.origin, NB.turn).values()].map(c => K(...c)));
  function checkWorld(plan, origin, turn) {
    const cm = worldCells(plan, origin, turn), bad = []; let reason = null;
    const g0 = [...plan.st[0]].map(k => cm.get(k)); const hs = g0.map(c => groundH(...c)), base = Math.max(...hs), low = Math.min(...hs);
    for (const c of cm.values()) { const k = K(...c); if (hexDist(...c) > WR - 2) { bad.push(k); reason = reason || "Something is in the way"; } if (NBCELLS.has(k)) { bad.push(k); reason = reason || "Something is already here"; } }
    for (const c of cm.values()) { const k = K(...c); if (TREES.has(k) || LANESET.has(k) || VLAMPS.some(v => K(...v) === k) || K(...WORKLAMP) === k) { bad.push(k); reason = reason || "Something is in the way"; } }
    if (!reason && base - low > TUNE.maxFooting) { reason = "Too steep here"; g0.forEach((c, i) => { if (base - hs[i] > TUNE.maxFooting) bad.push(K(...c)); }); }
    if (!reason && !planOK(plan)) reason = REASON[checkPlan(plan).find(i => i.lvl === "bad").rule];
    return { ok: !reason, reason, bad, base };
  }

  // ================================================================= the site (design 5 to 9)
  let SITE = null;
  function newSite(plan) { SITE = { plan, origin: [0, 0], turn: 0, phase: "placing", base: 0, built: new Map(), marked: new Set(), scaf: new Set(), last: null, striking: null }; siteDirty = true; refreshPlanSelect(); }
  const builtCount = S => piecesOf(S.plan).filter(p => p.cls !== "slot" && S.built.get(p.id) === p.sig).length;
  function wpiece(S, p) {
    const T = (q, r) => { const [a, b] = rotN([q, r], S.turn); return [a + S.origin[0], b + S.origin[1]]; };
    const o = { ...p, plan: p };
    if (p.q !== undefined) [o.q, o.r] = T(p.q, p.r);
    if (p.d !== undefined) o.d = (p.d + S.turn) % 6;
    if (p.cells) o.cells = p.cells.map(c => T(...c));
    return o;
  }
  function isBuilt(S, p) { return S.built.get(p.id) === p.sig; }
  function stateOf(S) {
    const P = piecesOf(S.plan), byId = new Map(P.map(p => [p.id, p]));
    const builtSet = new Set(P.filter(p => isBuilt(S, p)).map(p => p.id));
    const floorAt = (s, k) => builtSet.has(`f|${s}|${k}`);
    const wallsDone = s => P.filter(p => p.cls === "wall" && p.s === s).every(p => builtSet.has(p.id));
    const wl = s => wells(S.plan, s);
    const can = p => {
      if (builtSet.has(p.id) || p.cls === "slot") return false;
      if (S.marked.has(p.id)) return false;
      if (p.cls === "floor") return p.s === 0 || (wallsDone(p.s - 1) && floorAt(p.s - 1, K(p.q, p.r)) || (wallsDone(p.s - 1) && wl(p.s - 1).has(K(p.q, p.r))));
      if (p.cls === "wall") { const ks = [K(p.q, p.r), K(...nbr(p.q, p.r, p.d))].filter(k => S.plan.st[p.s].has(k)); return ks.some(k => floorAt(p.s, k) || (wl(p.s).has(k) && (p.s === 0 || wallsDone(p.s - 1)))); }
      if (p.cls === "stair") return floorAt(p.s, K(p.q, p.r)) && floorAt(p.s, K(...nbr(p.q, p.r, p.d)));
      if (p.cls === "lantern") return floorAt(p.s, K(p.q, p.r));
      if (p.cls === "roof" || p.cls === "deck") return wallsDone(p.s);
      return false;
    };
    // a marked piece waits on the marked pieces that rest on it (top first)
    const order = { roof: 0, deck: 0, lantern: 1, stair: 2, wall: 3, floor: 4 };
    const markedList = [...S.marked].map(id => S.mpieces.get(id)).filter(Boolean).sort((a, b) => b.s - a.s || order[a.cls] - order[b.cls]);
    return { P, byId, builtSet, can, markedList, wallsDone };
  }
  const BUILDORDER = { floor: 0, wall: 1, stair: 2, lantern: 3, deck: 4, roof: 5 };
  function nextPiece(S) { const st = stateOf(S); if (st.markedList.length) return { down: st.markedList[0] }; const c = st.P.filter(st.can).sort((a, b) => a.s - b.s || BUILDORDER[a.cls] - BUILDORDER[b.cls] || (a.id < b.id ? -1 : 1)); return c.length ? { up: c[0] } : null; }
  function footingOf(S, p) { if (p.cls !== "floor" || p.s !== 0) return 0; const [q, r] = rotN([p.q, p.r], S.turn); return Math.max(0, S.base - groundH(q + S.origin[0], r + S.origin[1])); }
  const PACK0 = { planks: 120, stone: 160, log: 12, rope: 12, fibre: 30, lantern: 4 }, PACK = { ...PACK0 };
  const canPay = c => Object.entries(c).every(([g, v]) => (PACK[g] || 0) >= v);
  const pay = (c, k = 1) => { for (const [g, v] of Object.entries(c)) PACK[g] = (PACK[g] || 0) - v * k; };
  function refundOf(c) { const o = {}; for (const [g, v] of Object.entries(c)) { const n = Math.floor(v * TUNE.refund); if (n > 0) o[g] = n; } return o; }
  function say(t) { $("spill").textContent = t; }
  function buildOne(S, p) {
    const c = TUNE.world === "free" ? {} : costOf(p, footingOf(S, p));
    if (!canPay(c)) { const miss = Object.entries(c).find(([g, v]) => (PACK[g] || 0) < v); say(`Needs ${miss[1]} ${GOODS[miss[0]][0].toLowerCase()}`); return false; }
    pay(c); S.built.set(p.id, p.sig); say(`${label(p)} built`); return true;
  }
  function takeDown(S, p) { S.built.delete(p.id); S.marked.delete(p.id); const back = refundOf(costOf(p, footingOf(S, p))); pay(back, -1); S.last = S.last || {}; say(`${label(p)} taken down: ${goodsText(back) || "nothing back"}`); return back; }
  const label = p => ({ floor: "Floor", wall: p.open === "door" ? "Door" : p.open === "window" ? "Window" : "Wall", stair: "Stair", lantern: "Lantern", roof: "Roof", deck: "Flat roof", slot: "Slot" }[p.cls]);
  function scaffoldStep(S) {
    if (S.phase !== "building") return;
    const st = stateOf(S);
    for (let s = 1; s < S.plan.st.length; s++) {
      const need = TUNE.scaffold && st.wallsDone(s - 1) && !st.wallsDone(s);
      if (need && !S.scaf.has(s)) { const bays = outerEdges(S.plan, s).length, c = bayCost(bays); if (TUNE.world === "free" || canPay(c)) { if (TUNE.world !== "free") pay(c); S.scaf.add(s); say(`Scaffolding up round floor ${s}: ${goodsText(c)}`); } else say(`Needs ${c.planks} planks and ${c.rope} rope for the scaffolding`); }
      if (!need && S.scaf.has(s)) { const bays = outerEdges(S.plan, s).length, c = bayCost(bays); if (TUNE.world !== "free") pay(c, -1); S.scaf.delete(s); say(`Scaffolding down from floor ${s}: ${goodsText(c)} back`); }
    }
  }
  function diffPlan(S, plan) {
    const oldP = new Map(piecesOf(S.plan).map(p => [p.id, p])), newP = new Map(piecesOf(plan).map(p => [p.id, p]));
    const add = [], gone = [], down = [], back = {};
    for (const [id, p] of newP) { if (p.cls === "slot") continue; const o = oldP.get(id); if (!o || o.sig !== p.sig) add.push(p); }
    for (const [id, o] of oldP) {
      if (o.cls === "slot") continue; const n = newP.get(id), built = S.built.get(id) === o.sig;
      if (!n || n.sig !== o.sig) { if (built) { down.push(o); addTo(back, refundOf(costOf(o, footingOf(S, o)))); } else gone.push(o); }
    }
    for (const id of S.marked) if (!newP.has(id) || newP.get(id).sig !== S.built.get(id)) { /* still to take down */ }
    return { add, gone, down, back };
  }
  function changeHTML(d) {
    const cnt = list => { const m = {}; for (const p of list) m[label(p)] = (m[label(p)] || 0) + 1; return Object.entries(m).map(([k, v]) => `${v} ${k.toLowerCase()}${v > 1 ? (k === "Stair" ? "s" : "s") : ""}`).join(", "); };
    const rows = [];
    if (d.add.length) rows.push(`<div class="add">New: ${cnt(d.add)}</div>`);
    if (d.gone.length) rows.push(`<div>Not built, dropped: ${cnt(d.gone)}</div>`);
    if (d.down.length) rows.push(`<div class="down">${TUNE.removal === "mark" ? "Built, to take down" : "Built, coming down now"}: ${cnt(d.down)}</div>`);
    if (Object.keys(d.back).length) rows.push(`<div class="back">Comes back: ${goodsText(d.back)}</div>`);
    return rows.join("") || `<div>No change yet</div>`;
  }
  function applyPlan(S, plan) {
    if (S.phase === "placing") { S.plan = plan; siteDirty = true; return { ok: true }; }
    const w = checkWorld(plan, S.origin, S.turn);
    // the site's own built pieces count as its own: only the neighbour, trees, the lane and the slope can refuse it
    if (!w.ok) { say(w.reason); return w; }
    const d = diffPlan(S, plan);
    S.mpieces = S.mpieces || new Map();
    for (const p of d.down) { if (TUNE.removal === "mark") { S.marked.add(p.id); S.mpieces.set(p.id, p); } else takeDown(S, p); }
    if (TUNE.removal !== "mark") for (const p of d.down) S.built.delete(p.id);
    // a kit or opening swap: the old body stays built until it is down
    const keep = new Map(piecesOf(plan).map(p => [p.id, p.sig]));
    for (const id of [...S.marked]) if (keep.get(id) === S.built.get(id)) S.marked.delete(id);
    S.plan = plan; S.last = d; siteDirty = true;
    if (S.phase === "done" && (d.add.length || S.marked.size)) S.phase = "building";
    scaffoldStep(S);
    $("schange").innerHTML = changeHTML(d);
    say(d.down.length && TUNE.removal === "mark" ? "Marked pieces come down first" : "Plan changed");
    refreshPlanSelect();
    return { ok: true };
  }
  function strike(S) {
    if (!S || S.phase !== "building") return false;
    const n = nextPiece(S);
    if (!n) { S.phase = "done"; say("Finished: the stakes and lime lines are gone"); siteDirty = true; return false; }
    if (n.down) { takeDown(S, n.down); siteDirty = true; return true; }
    const ok = buildOne(S, n.up); if (ok) { scaffoldStep(S); siteDirty = true; if (!nextPiece(S)) { S.phase = "done"; say("Finished: the stakes and lime lines are gone"); } } return ok;
  }

  // ================================================================= the site's 3D view
  let S3 = null, siteDirty = true;
  function initSite() {
    const canvas = $("scv");
    const renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
    renderer.setPixelRatio(Math.min(2, devicePixelRatio)); renderer.outputEncoding = THREE.sRGBEncoding; renderer.shadowMap.enabled = true; renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    const scene = new THREE.Scene(); scene.fog = new THREE.Fog(0x0e1424, 60, 150);
    const camera = new THREE.PerspectiveCamera(46, 1, 0.3, 600);
    const sun = new THREE.DirectionalLight(0xffffff, 1); sun.castShadow = true; sun.shadow.mapSize.set(2048, 2048); Object.assign(sun.shadow.camera, { left: -30, right: 30, top: 30, bottom: -30, near: 1, far: 220 }); sun.shadow.bias = -0.0005; scene.add(sun, sun.target);
    const hemi = new THREE.HemisphereLight(0xffffff, 0x1a1a22, 0.2); scene.add(hemi);
    const stars = (() => { const n = 900, p = new Float32Array(n * 3); let sd = 9; const rn = () => ((sd = (sd * 16807) % 2147483647) / 2147483647); for (let i = 0; i < n; i++) { const a = rn() * 6.283, e = 0.05 + rn() * 1.45; p[3 * i] = Math.cos(a) * Math.cos(e) * 400; p[3 * i + 1] = Math.sin(e) * 400; p[3 * i + 2] = Math.sin(a) * Math.cos(e) * 400; } const g = new THREE.BufferGeometry(); g.setAttribute("position", new THREE.BufferAttribute(p, 3)); const m = new THREE.Points(g, new THREE.PointsMaterial({ color: 0xdfe6ff, size: 1.3, sizeAttenuation: false, fog: false, transparent: true })); scene.add(m); return m; })();
    const world = new THREE.Group(), siteG = new THREE.Group(), nbG = new THREE.Group(); scene.add(world, siteG, nbG);
    PBD.sheet = PBD.SHEET.fields;
    // ground: hex columns with the terrain atlas, the lane in packed path
    const GS = new PBD.GeoSet(), cellsAll = [];
    for (let q = -WR; q <= WR; q++) for (let r = -WR; r <= WR; r++) if (hexDist(q, r) <= WR) cellsAll.push([q, r]);
    for (const [q, r] of cellsAll) {
      const h = groundH(q, r), pts = hexPts(q, r), x = hx(q, r), z = hz(q, r), top = LANESET.has(K(q, r)) ? "a:path" : "a:grass";
      GS.g(top).face(pts.map(p => [p[0], h, p[1]]), [0, 1, 0], PBD.cellUV(x, z));
      for (let d = 0; d < 6; d++) {
        const [q2, r2] = nbr(q, r, d), h2 = hexDist(q2, r2) <= WR ? groundH(q2, r2) : h - 4; if (h2 >= h) continue;
        const A = corner(q, r, d), B = corner(q, r, d + 1), [nx, nz] = dvec(d);
        for (let y = h; y > h2 + 1e-3; y -= 1) GS.g(top === "a:grass" && y === h ? "a:grassside" : "a:dirt").face([[A[0], y - 1, A[1]], [B[0], y - 1, B[1]], [B[0], y, B[1]], [A[0], y, A[1]]], [nx, 0, nz], PBD.sideUV(A, B, y));
      }
    }
    for (const k of TREES) { const [q, r] = unK(k); if (hexDist(q, r) > WR) continue; PBD.tree(GS, hexPts(q, r), hx(q, r), hz(q, r), groundH(q, r), (q * 31 + r * 17) >>> 0, "a:bark", "a:leaves"); }
    for (const [q, r] of [...VLAMPS, WORKLAMP]) PBD.lampPost(GS, hx(q, r), groundH(q, r), hz(q, r));
    PBD.addMeshes(GS, world);
    // the far ground beyond the plot, so it does not float
    const farG = new THREE.RingGeometry(WR * W * 0.9, 260, 48, 1), fp = farG.attributes.position, fu = farG.attributes.uv; for (let i = 0; i < fp.count; i++) fu.setXY(i, fp.getX(i) / 2, fp.getY(i) / 2);
    const far = new THREE.Mesh(farG, PBD.mat("a:grass")); far.rotation.x = -Math.PI / 2; far.position.y = -0.02; far.receiveShadow = true; world.add(far);
    // lamp heads (the light is per pixel, PBD.setLights)
    const LAMPMAT = new THREE.MeshBasicMaterial({ color: 0xffd9a0 });
    for (const [q, r] of [...VLAMPS, WORKLAMP]) { const m = new THREE.Mesh(new THREE.CylinderGeometry(0.1, 0.1, 0.32, 6), LAMPMAT); m.position.set(hx(q, r) + 0.45, groundH(q, r) + 2.26, hz(q, r)); world.add(m); }
    S3 = { renderer, scene, camera, sun, hemi, stars, world, siteG, nbG, LAMPMAT, canvas, ORB: { tx: 0, ty: 2.5, tz: 0.5, dist: 25, yaw: 0.5, pitch: 0.56 }, on: false, last: performance.now(), acc: 0 };
    // the neighbour's house, built whole
    const nbS = { plan: NB.plan, origin: NB.origin, turn: NB.turn, base: Math.max(...[...NB.plan.st[0]].map(k => groundH(...rotN(unK(k), 0).map((v, i) => v + NB.origin[i])))), built: new Map(piecesOf(NB.plan).map(p => [p.id, p.sig])), marked: new Set(), scaf: new Set(), phase: "done" };
    S3.nb = nbS; drawSite(nbS, nbG, true);
    // orbit: drag to turn, right-drag or two fingers to slide, wheel or pinch to come closer
    const pts = new Map(); let pinch = 0, moved = 0;
    canvas.addEventListener("pointerdown", e => { canvas.setPointerCapture(e.pointerId); pts.set(e.pointerId, [e.clientX, e.clientY, e.button, e.shiftKey]); moved = 0; if (pts.size === 2) { const [a, b] = [...pts.values()]; pinch = Math.hypot(a[0] - b[0], a[1] - b[1]); } });
    canvas.addEventListener("pointermove", e => {
      const p = pts.get(e.pointerId); if (!p) return; const dx = e.clientX - p[0], dy = e.clientY - p[1]; moved += Math.abs(dx) + Math.abs(dy); const O = S3.ORB;
      if (pts.size === 2) { const [a, b] = [...pts.values()]; const d = Math.hypot(a[0] - b[0], a[1] - b[1]); if (pinch) O.dist = clamp(O.dist * pinch / Math.max(1, d), 8, 90); pinch = d; }
      else if (p[2] === 2 || p[3]) { const c = Math.cos(O.yaw), s = Math.sin(O.yaw), k = O.dist * 0.0016; O.tx -= (dx * c - dy * s) * k; O.tz -= (-dx * s - dy * c) * k * -1; }
      else { O.yaw -= dx * 0.006; O.pitch = clamp(O.pitch + dy * 0.005, 0.12, 1.45); }
      p[0] = e.clientX; p[1] = e.clientY;
    });
    canvas.addEventListener("pointerup", e => { const p = pts.get(e.pointerId); pts.delete(e.pointerId); if (pts.size < 2) pinch = 0; if (p && moved < 6 && p[2] === 0) siteClick(e); });
    canvas.addEventListener("pointercancel", e => pts.delete(e.pointerId));
    canvas.addEventListener("contextmenu", e => e.preventDefault());
    canvas.addEventListener("wheel", e => { e.preventDefault(); S3.ORB.dist = clamp(S3.ORB.dist * Math.exp(e.deltaY * 0.0012), 8, 90); }, { passive: false });
    addEventListener("resize", () => { if (TAB === "site") sresize(); });
  }
  function sresize() { const { renderer, camera, canvas } = S3, w = canvas.clientWidth, h = canvas.clientHeight; renderer.setSize(w, h, false); camera.aspect = w / Math.max(1, h); camera.updateProjectionMatrix(); }
  const ray = new THREE.Raycaster(), mv = new THREE.Vector2();
  function siteClick(e) {
    const rc = S3.canvas.getBoundingClientRect(); mv.set((e.clientX - rc.left) / rc.width * 2 - 1, -(e.clientY - rc.top) / rc.height * 2 + 1); ray.setFromCamera(mv, S3.camera);
    if (SITE.phase === "building") {
      const hit = ray.intersectObjects(S3.siteG.children, true).find(h => h.object.userData.pid);
      if (hit) {
        const id = hit.object.userData.pid, st = stateOf(SITE);
        if (SITE.marked.has(id)) { const top = st.markedList[0]; if (top && top.id !== id) { say(`Take the ${label(top).toLowerCase()} above it down first`); return; } takeDown(SITE, SITE.mpieces.get(id)); siteDirty = true; return; }
        const p = st.byId.get(id); if (!p) return; if (p.cls === "slot") { say(`A ${p.kind} goes here: place the furniture`); return; }
        if (st.markedList.length) { say("Marked pieces come down first"); return; }
        if (!st.can(p)) { say(`${label(p)}: waiting on what holds it up`); return; }
        if (buildOne(SITE, p)) { scaffoldStep(SITE); if (!nextPiece(SITE)) { SITE.phase = "done"; say("Finished: the stakes and lime lines are gone"); } siteDirty = true; }
        return;
      }
    }
    if (SITE.phase !== "placing") return;
    const g = ray.intersectObjects(S3.world.children, false)[0]; if (!g) return;
    const [q, r] = cellAt(g.point.x, g.point.z); if (hexDist(q, r) > WR) return;
    SITE.origin = [q, r]; siteDirty = true;
  }

  // ---- geometry of one piece, in world cells -------------------------------------------------
  function taper(GS, m, x, z, y0, y1, r0, r1, n = 6, a0 = 0) {
    const ring = (rr, y) => { const o = []; for (let i = 0; i < n; i++) { const a = a0 + i * 2 * Math.PI / n; o.push([x + rr * Math.cos(a), y, z + rr * Math.sin(a)]); } return o; };
    const A = ring(r0, y0), B = ring(r1, y1);
    for (let i = 0; i < n; i++) { const j = (i + 1) % n, mx = (A[i][0] + A[j][0]) / 2 - x, mz = (A[i][2] + A[j][2]) / 2 - z; GS.g(m).face([A[i], A[j], B[j], B[i]], [mx, 0, mz], p => [Math.atan2(p[2] - z, p[0] - x) / (2 * Math.PI) * 0.6, p[1]]); }
    GS.g(m).face(B, [0, 1, 0]); GS.g(m).face(A, [0, -1, 0]);
  }
  function pieceGeo(S, p, GS, kitName) {
    const kit = PKITS[kitName || p.kit], y0 = S.base + p.s * SH, slabTop = y0 + 0.12;
    if (p.cls === "floor") {
      const pts = hexPts(p.q, p.r);
      GS.prism(kit.floor, p.s === 0 ? kit.edge : kit.edge, pts, p.s === 0 ? S.base - 0.1 : slabTop - 0.24, slabTop, { uvTop: PBD.cellUV(hx(p.q, p.r), hz(p.q, p.r)), bottom: p.s > 0 ? "plank" : null });
      if (p.s === 0) { const g = groundH(p.q, p.r); if (g < S.base) GS.prism(null, "fieldstone", hexPts(p.q, p.r, 1.0), g - 0.2, S.base - 0.1); }
      return;
    }
    if (p.cls === "wall") {
      const A = corner(p.q, p.r, p.d), B = corner(p.q, p.r, p.d + 1), L = Math.hypot(B[0] - A[0], B[1] - A[1]), tx = (B[0] - A[0]) / L, tz = (B[1] - A[1]) / L;
      const [nx, nz] = dvec(p.d), mx = (A[0] + B[0]) / 2, mz = (A[1] + B[1]) / 2, ang = Math.atan2(tz, tx), t = kit.t, len = L + t * 0.58, top = y0 + SH;
      const outM = kit.lower && p.s === 0 ? kit.lower : kit.wall;
      const pick = n => { const s2 = n[0] * nx + n[2] * nz; if (p.outer && s2 > 0.5) return outM; if (!p.outer && Math.abs(s2) > 0.5) return kit.inside; if (Math.abs(n[1]) > 0.5) return kit.edge; return p.outer && s2 < -0.5 ? kit.inside : kit.edge; };
      const uv = (n, m) => m === "halftimber" ? q => [((q[0] - A[0]) * tx + (q[2] - A[1]) * tz) / L, (q[1] - y0) / SH] : null;
      const seg = (along, l, a, b) => { if (b - a < 0.005 || l < 0.005) return; GS.box(pick, mx + tx * along, a, mz + tz * along, l, b - a, t, ang, uv); };
      const yb = p.s === 0 ? S.base - 0.1 : y0;
      if (p.open === "none") { seg(0, len, yb, top); return; }
      const op = p.open === "door" ? { b: slabTop, e: slabTop + 2.2, w: 1.0 } : { b: y0 + 1.0, e: y0 + 2.0, w: 0.8 };
      const ww = Math.min(op.w, L - 0.2), jl = (len - ww) / 2;
      seg(0, len, yb, op.b); seg(-(ww / 2 + jl / 2), jl, op.b, op.e); seg(ww / 2 + jl / 2, jl, op.b, op.e); seg(0, len, op.e, top);
      if (p.open === "door") GS.box("timber", mx - nx * 0.03, op.b, mz - nz * 0.03, ww, op.e - op.b, 0.07, ang);
      else { GS.box("timber", mx + nx * 0.02, op.b - 0.05, mz + nz * 0.02, ww + 0.12, 0.07, t + 0.12, ang); GS.box("timber", mx, op.b, mz, 0.06, op.e - op.b, t * 0.5, ang); GS.box("timber", mx, (op.b + op.e) / 2 - 0.03, mz, ww, 0.06, t * 0.5, ang); }
      return;
    }
    if (p.cls === "stair") {
      const [q2, r2] = nbr(p.q, p.r, p.d), [ux, uz] = dvec(p.d), ax = hx(p.q, p.r) - ux * W * 0.42, az = hz(p.q, p.r) - uz * W * 0.42, run = W * 1.84, N = 15, rise = SH / N, dep = run / N, ang = Math.atan2(uz, ux);
      for (let i = 0; i < N; i++) { const f = (i + 0.5) * dep; GS.box(kit.stair, ax + ux * f, slabTop - 0.1, az + uz * f, dep + 0.01, rise * (i + 1) + 0.1, 1.15, ang); }
      return;
    }
    if (p.cls === "lantern") {
      const x = hx(p.q, p.r) + 0.55, z = hz(p.q, p.r) + 0.3;
      taper(GS, "timber", x, z, slabTop, slabTop + 1.25, 0.06, 0.045);
      taper(GS, "iron", x, z, slabTop + 1.25, slabTop + 1.31, 0.13, 0.13); taper(GS, "iron", x, z, slabTop + 1.58, slabTop + 1.7, 0.14, 0.05);
      return;
    }
    if (p.cls === "roof" || p.cls === "deck") {
      const top = y0 + SH, cells = p.cells; let minx = 1e9, maxx = -1e9, minz = 1e9, maxz = -1e9;
      for (const [q, r] of cells) for (const c of hexPts(q, r)) { minx = Math.min(minx, c[0]); maxx = Math.max(maxx, c[0]); minz = Math.min(minz, c[1]); maxz = Math.max(maxz, c[1]); }
      if (p.shape === "flat") {
        for (const [q, r] of cells) GS.prism("plank", null, hexPts(q, r), top, top + 0.18, { uvTop: PBD.cellUV(hx(q, r), hz(q, r)), bottom: "plank" });
        const set = new Set(cells.map(c => K(...c)));
        for (const [q, r] of cells) for (let d = 0; d < 6; d++) { if (set.has(K(...nbr(q, r, d)))) continue; const A = corner(q, r, d), B = corner(q, r, d + 1), L = Math.hypot(B[0] - A[0], B[1] - A[1]); GS.box(n => Math.abs(n[1]) > 0.5 ? kit.edge : kit.wall, (A[0] + B[0]) / 2, top, (A[1] + B[1]) / 2, L + kit.t * 0.58, 0.55, kit.t, Math.atan2(B[1] - A[1], B[0] - A[0])); }
        return;
      }
      if (p.shape === "cone" && cells.length === 1) {
        const [q, r] = cells[0], x = hx(q, r), z = hz(q, r), h = 2.6, oh = 0.5, apex = [x, top + h, z], rr = R + oh / Math.cos(Math.PI / 6), slope = h / (W / 2), pts = [];
        for (let k = 0; k < 6; k++) { const a = (60 * k - 30) * Math.PI / 180; pts.push([x + rr * Math.cos(a), top - oh * slope * 0.6, z + rr * Math.sin(a)]); }
        for (let k = 0; k < 6; k++) { const a = pts[k], b = pts[(k + 1) % 6]; GS.g(kit.roof).face([a, b, apex], [(a[0] + b[0]) / 2 - x, 0.5, (a[2] + b[2]) / 2 - z], q2 => [(q2[0] + q2[2]) / 2, q2[1] / 2]); GS.g(kit.roof).face([a, b, apex], [0, -1, 0], q2 => [(q2[0] + q2[2]) / 2, q2[1] / 2]); }
        return;
      }
      // a gable is fitted along the plan's own long axis, so it turns with the plan
      const ra = (S.turn || 0) * Math.PI / 3, rc = Math.cos(ra), rs = Math.sin(ra), ox = hx(...S.origin), oz = hz(...S.origin);
      const toW = v => [ox + v[0] * rc - v[2] * rs, v[1], oz + v[0] * rs + v[2] * rc], dirW = v => [v[0] * rc - v[2] * rs, v[1], v[0] * rs + v[2] * rc];
      const toL = w => { const dx = w[0] - ox, dz = w[2] - oz; return [dx * rc + dz * rs, w[1], -dx * rs + dz * rc]; };
      const LG = { g: mm => ({ face: (pts, want, uvf) => GS.g(mm).face(pts.map(toW), want ? dirW(want) : want, uvf ? (w, n) => uvf(toL(w), n) : uvf) }) };
      minx = 1e9; maxx = -1e9; minz = 1e9; maxz = -1e9;
      for (const [q, r] of cells) for (const c of hexPts(q, r)) { const l = toL([c[0], 0, c[1]]); minx = Math.min(minx, l[0]); maxx = Math.max(maxx, l[0]); minz = Math.min(minz, l[2]); maxz = Math.max(maxz, l[2]); }
      const oh = 0.45, pitch = 0.85, x0 = minx, x1 = maxx, z0 = minz, z1 = maxz, zc = (z0 + z1) / 2, hw = (z1 - z0) / 2, yR = top + hw * pitch, eY = top - oh * pitch, th = 0.18, m = kit.roof;
      for (const s of [-1, 1]) {
        const ze = zc + s * (hw + oh), uv = q2 => [q2[0] / 2, (Math.abs(q2[2] - zc) * Math.sqrt(1 + pitch * pitch)) / 2];
        LG.g(m).face([[x0 - oh, eY + th, ze], [x1 + oh, eY + th, ze], [x1 + oh, yR + th, zc], [x0 - oh, yR + th, zc]], [0, 1, s * 0.01], uv);
        LG.g("plank").face([[x0 - oh, eY, ze], [x1 + oh, eY, ze], [x1 + oh, yR, zc], [x0 - oh, yR, zc]], [0, -1, 0]);
        LG.g(m).face([[x0 - oh, eY, ze], [x1 + oh, eY, ze], [x1 + oh, eY + th, ze], [x0 - oh, eY + th, ze]], [0, 0, s]);
        for (const xe of [x0 - oh, x1 + oh]) LG.g(m).face([[xe, eY, ze], [xe, eY + th, ze], [xe, yR + th, zc], [xe, yR, zc]], [xe < x0 ? -1 : 1, 0, 0]);
      }
      for (const xe of [x0 + 0.05, x1 - 0.05]) LG.g(kit.gable).face([[xe, top, z0], [xe, top, z1], [xe, yR, zc]], [xe < (x0 + x1) / 2 ? -1 : 1, 0, 0], kit.gable === "halftimber" ? q2 => [(q2[2] - z0) / (z1 - z0) * 2, (q2[1] - top) / 3] : null);
      LG.g("plank").face([[x0, top, z0], [x1, top, z0], [x1, top, z1], [x0, top, z1]], [0, -1, 0]);
      { const c = toW([(x0 + x1) / 2, 0, zc]); GS.box("timber", c[0], yR + th - 0.05, c[2], x1 - x0 + 2 * oh + 0.1, 0.16, 0.22, ra); }
      return;
    }
    if (p.cls === "slot") {
      const x = hx(p.q, p.r), z = hz(p.q, p.r), ang = p.d * Math.PI / 3, c = Math.cos(ang), s = Math.sin(ang), f = (a, b) => [x + c * a - s * b, z + s * a + c * b], m = "slot";
      const at = (a, b, y, sx, sy, sz) => { const [px, pz] = f(a, b); GS.box(m, px, slabTop + y, pz, sx, sy, sz, ang); };
      if (p.kind === "bed") { at(0, 0, 0, 1.9, 0.42, 0.95); at(-0.92, 0, 0.42, 0.08, 0.45, 0.95); }
      else if (p.kind === "table") { at(0, 0, 0.7, 1.1, 0.07, 0.75); at(0, 0, 0, 0.1, 0.7, 0.1); }
      else if (p.kind === "chest") { at(0, 0, 0, 0.5, 0.45, 0.85); }
      else if (p.kind === "bench") { at(0, 0, 0, 0.7, 0.85, 1.6); }
      else { at(0, 0, 0, 0.48, 0.46, 0.48); at(-0.22, 0, 0.46, 0.05, 0.5, 0.48); }
    }
  }
  function limeTex() {
    if (limeTex.t) return limeTex.t;
    const cv = document.createElement("canvas"); cv.width = 64; cv.height = 16; const c = cv.getContext("2d"), img = c.createImageData(64, 16); let sd = 7; const rn = () => ((sd = (sd * 16807) % 2147483647) / 2147483647);
    for (let y = 0; y < 16; y++) for (let x = 0; x < 64; x++) { const e = Math.abs(y - 7.5) / 8, edge = 0.55 + 0.35 * Math.sin(x * 0.7) * Math.sin(x * 0.23 + 1) + rn() * 0.25, a = e < edge ? (0.55 + rn() * 0.45) : 0; const i = (y * 64 + x) * 4; img.data[i] = 226 + rn() * 20; img.data[i + 1] = 224 + rn() * 18; img.data[i + 2] = 208 + rn() * 16; img.data[i + 3] = a * 255; }
    c.putImageData(img, 0, 0); const t = new THREE.CanvasTexture(cv); t.magFilter = t.minFilter = THREE.NearestFilter; t.generateMipmaps = false; t.encoding = THREE.sRGBEncoding; t.wrapS = THREE.RepeatWrapping;
    return (limeTex.t = t);
  }
  const GHOST = {}; function ghostMat(kind) {
    const col = kind === "bad" ? 0xff6b57 : kind === "slot" ? 0xf0b43c : kind === "mark" ? 0xf4c9b8 : TINTS[TUNE.tint];
    const op = kind === "wait" ? TUNE.waitOpacity : kind === "slot" ? Math.max(0.12, TUNE.waitOpacity + 0.08) : kind === "mark" ? 0.42 : TUNE.ghostOpacity;
    const key = kind + col + op; return GHOST[key] || (GHOST[key] = new THREE.MeshBasicMaterial({ color: col, transparent: true, opacity: op, depthWrite: false, side: THREE.DoubleSide, polygonOffset: true, polygonOffsetFactor: -1, polygonOffsetUnits: -1 }));
  }
  const GLASS = { lit: new THREE.MeshBasicMaterial({ color: 0xffc870 }), dark: new THREE.MeshBasicMaterial({ color: 0x1a1a22 }) };
  const SLOTMAT = PBD.litMaterial(null, 0xffffff);
  function drawSite(S, grp, nb) {
    for (const o of grp.children) o.traverse(m => { if (m.geometry) m.geometry.dispose(); });
    grp.clear();
    if (!S) return [];
    if (S.phase === "placing") { const w = checkWorld(S.plan, S.origin, S.turn); S.base = w.base; S.check = w; }
    const st = nb ? null : stateOf(S), P = piecesOf(S.plan), lights = [], lanternBuilt = new Set();
    for (const pp of P) {
      const p = wpiece(S, pp), built = isBuilt(S, pp), GS = new PBD.GeoSet();
      if (pp.cls === "lantern" && built) lanternBuilt.add(pp.s);
      let state = built ? "built" : pp.cls === "slot" ? "slot" : S.phase === "placing" ? (S.check.ok ? "ghost" : "bad") : (st.can(pp) ? "ghost" : "wait");
      if (S.marked.has(pp.id) || (pp.cls === "slot" && (built || nb))) continue;
      pieceGeo(S, p, GS);
      const g = new THREE.Group(); g.userData.pid = pp.id;
      const meshes = PBD.addMeshes(GS, g, state === "built" ? {} : { mat: () => ghostMat(state), cast: () => false });
      for (const m of meshes) m.userData.pid = pp.id;
      if (state !== "built") for (const m of meshes) { m.receiveShadow = false; m.renderOrder = 2; }
      grp.add(g);
      if (pp.cls === "lantern" && built) { const x = hx(p.q, p.r) + 0.55, z = hz(p.q, p.r) + 0.3, y = S.base + p.s * SH + 0.12 + 1.45; const gl = new THREE.Mesh(new THREE.CylinderGeometry(0.1, 0.1, 0.27, 6), GLASS.lit); gl.position.set(x, y, z); g.add(gl); lights.push({ x, y, z, c: lin(0xffc878), p: 1.4, R: 5.2 }); }
    }
    // pieces marked for removal: still built, washed in chalk; and a swapped body still standing
    for (const id of S.marked) {
      const pp = S.mpieces.get(id); if (!pp) continue; const p = wpiece(S, pp), GS = new PBD.GeoSet(); pieceGeo(S, p, GS, pp.kit);
      const g = new THREE.Group(); g.userData.pid = id; PBD.addMeshes(GS, g).forEach(m => m.userData.pid = id);
      const GS2 = new PBD.GeoSet(); pieceGeo(S, p, GS2, pp.kit); PBD.addMeshes(GS2, g, { mat: () => ghostMat("mark"), cast: () => false }).forEach(m => { m.userData.pid = id; m.renderOrder = 3; });
      grp.add(g);
    }
    // windows glow at night where the storey has a lantern built
    for (const pp of P) if (pp.cls === "wall" && pp.open === "window" && isBuilt(S, pp)) {
      const p = wpiece(S, pp), A = corner(p.q, p.r, p.d), B = corner(p.q, p.r, p.d + 1), y = S.base + p.s * SH + 1.5, lit = lanternBuilt.has(pp.s) && NIGHT, [nx, nz] = dvec(p.d);
      const pane = new THREE.Mesh(new THREE.PlaneGeometry(0.78, 0.98), lit ? GLASS.lit : GLASS.dark); pane.position.set((A[0] + B[0]) / 2, y, (A[1] + B[1]) / 2); pane.lookAt(pane.position.x + nx, y, pane.position.z + nz); grp.add(pane);
      const pane2 = pane.clone(); pane2.lookAt(pane.position.x - nx, y, pane.position.z - nz); grp.add(pane2);
      if (lit) lights.push({ x: (A[0] + B[0]) / 2 + nx * 0.6, y, z: (A[1] + B[1]) / 2 + nz * 0.6, c: lin(0xffb860), p: 0.4, R: 2.4 });
    }
    if (nb) return lights;
    // set-out marks: stakes at the outline's corners, lime along the outer walls (not once finished)
    if (S.phase === "building") {
      const GS = new PBD.GeoSet(), lime = [], cm = new Set(); const g0 = S.plan.st[0], ck = c => c.map(v => v.toFixed(2)).join(","), cnt = new Map();
      for (const k of g0) { const w = wpiece(S, { q: unK(k)[0], r: unK(k)[1], d: 0 }); for (let j = 0; j < 6; j++) { const kk = ck(corner(w.q, w.r, j)); cnt.set(kk, (cnt.get(kk) || 0) + 1); } }
      for (const k of g0) { const [pq, pr] = unK(k); for (let d = 0; d < 6; d++) { if (g0.has(K(...nbr(pq, pr, d)))) continue; const p = wpiece(S, { q: pq, r: pr, d }); lime.push(p); for (const kk of [p.d, p.d + 1]) { const c = corner(p.q, p.r, kk), key = ck(c); if (cm.has(key) || cnt.get(key) !== 1) continue; cm.add(key); const gh = groundH(p.q, p.r), [nx, nz] = [c[0] - hx(p.q, p.r), c[1] - hz(p.q, p.r)], l = Math.hypot(nx, nz), sx = c[0] + nx / l * 0.45, sz = c[1] + nz / l * 0.45; taper(GS, "timber", sx, sz, gh - 0.25, gh + 0.95, 0.055, 0.03, 6, 0.4); GS.box("cloth", sx + 0.05, gh + 0.62, sz, 0.02, 0.2, 0.16, 0.3); } } }
      // only every other corner keeps a stake once there are many
      PBD.addMeshes(GS, grp);
      const LM = PBD.litMaterial(limeTex(), 0xffffff, { transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 });
      for (const p of lime) { const A = corner(p.q, p.r, p.d), B = corner(p.q, p.r, p.d + 1), L = Math.hypot(B[0] - A[0], B[1] - A[1]), gh = groundH(p.q, p.r) + 0.01; const m = new THREE.Mesh(new THREE.PlaneGeometry(L + 0.3, 0.55), LM); m.rotation.x = -Math.PI / 2; m.rotation.z = -Math.atan2(B[1] - A[1], B[0] - A[0]); const [nx, nz] = dvec(p.d); m.position.set((A[0] + B[0]) / 2 + nx * 0.42, gh, (A[1] + B[1]) / 2 + nz * 0.42); m.receiveShadow = true; m.renderOrder = 1; grp.add(m); }
      // scaffolding round a storey being built
      const SG = new PBD.GeoSet(), poles = new Map(), off = 0.95;
      for (const s of S.scaf) {
        const y = S.base + s * SH + 0.1;
        for (const e of outerEdges(S.plan, s)) {
          const p = wpiece(S, { q: e.q, r: e.r, d: e.d }), A = corner(p.q, p.r, p.d), B = corner(p.q, p.r, p.d + 1), [nx, nz] = dvec(p.d), L = Math.hypot(B[0] - A[0], B[1] - A[1]), ang = Math.atan2(B[1] - A[1], B[0] - A[0]);
          const ox = nx * off, oz = nz * off, mx = (A[0] + B[0]) / 2 + ox, mz = (A[1] + B[1]) / 2 + oz;
          for (const c of [A, B]) { const key = c[0].toFixed(2) + "," + c[1].toFixed(2), o = poles.get(key) || { c, nx: 0, nz: 0 }; o.nx += nx; o.nz += nz; poles.set(key, o); }
          SG.box("plank", mx, y, mz, L + 0.6, 0.06, 0.62, ang); SG.box("timber", mx + nx * 0.36, y + 1.0, mz + nz * 0.36, L + 0.6, 0.07, 0.07, ang);
        }
        for (const o of poles.values()) { const l = Math.hypot(o.nx, o.nz) || 1, px = o.c[0] + o.nx / l * off * 1.05, pz = o.c[1] + o.nz / l * off * 1.05; const g = groundH(...cellAt(px, pz)); taper(SG, "timber", px, pz, g - 0.1, S.base + s * SH + 1.9, 0.075, 0.055, 6, 0.3); taper(SG, "cloth2", px, pz, y - 0.06, y + 0.12, 0.09, 0.09, 6, 0.3); }
        poles.clear();
      }
      PBD.addMeshes(SG, grp);
    }
    return lights;
  }
  PBD.REP.cloth = PBD.REP.cloth || 1; PBD.REP.cloth2 = PBD.REP.cloth2 || 1;
  let NIGHT = true;
  function applyTime() {
    const { scene, sun, hemi, stars, LAMPMAT } = S3, c = new THREE.Color(NIGHT ? 0x0e1424 : 0x9fc6e2);
    scene.background = c; scene.fog.color.copy(c);
    if (NIGHT) { sun.color.set(0x8fa6d8); sun.intensity = 0.035; sun.position.set(-30, 50, 40); hemi.intensity = 0.008; hemi.color.set(0x34405a); hemi.groundColor.set(0x1a1a22); }
    else { sun.color.set(0xfff0d8); sun.intensity = 1.4; sun.position.set(50, 45, 30); hemi.intensity = 0.45; hemi.color.set(0xc8dcea); hemi.groundColor.set(0x5a5040); }
    PBD.LU.uLamp.value = NIGHT ? 1.15 : 0; LAMPMAT.color.set(NIGHT ? 0xffd9a0 : 0x6a655a); GLASS.lit.color.set(NIGHT ? 0xffc870 : 0x6a5a40); stars.material.opacity = NIGHT ? 1 : 0;
  }
  function srebuild() {
    if (!S3) return;
    const L = [];
    for (const [q, r] of [...VLAMPS, WORKLAMP]) L.push({ x: hx(q, r) + 0.45, y: groundH(q, r) + 2.25, z: hz(q, r), c: lin(0xffc878), p: 1.4, R: 5.2 });
    S3.nb.marked = new Set(); S3.nb.mpieces = new Map();
    L.push(...drawSite(S3.nb, S3.nbG, true));
    SITE.mpieces = SITE.mpieces || new Map();
    L.push(...drawSite(SITE, S3.siteG, false));
    PBD.setLights(L); applyTime(); shud(); siteDirty = false;
  }
  function shud() {
    const S = SITE, P = piecesOf(S.plan).filter(p => p.cls !== "slot"), done = P.filter(p => isBuilt(S, p)).length, left = P.length - done;
    $("stop").textContent = S.phase === "placing" ? `${S.plan.name}  \u00b7  ${P.length} pieces  \u00b7  turned ${S.turn * 60}\u00b0` : S.phase === "done" ? `${S.plan.name}  \u00b7  finished` : `${S.plan.name}  \u00b7  ${left} of ${P.length} left${S.marked.size ? `  \u00b7  ${S.marked.size} to take down` : ""}`;
    if (S.phase === "placing") say(S.check.ok ? "Turn it, then set it out" : S.check.reason);
    const need = {}; for (const p of P) if (!isBuilt(S, p)) addTo(need, costOf(p, footingOf(S, p)));
    $("sbom").innerHTML = TUNE.world === "free" ? "" : Object.entries(need).filter(([, v]) => v > 0).map(([g, v]) => `<div class="chip${(PACK[g] || 0) < v ? " short" : ""}"><img src="${goodIcon(g)}" alt="">${PACK[g] || 0} / ${v}</div>`).join("");
    $("spack").innerHTML = Object.keys(GOODS).map((g, i) => `<button title="${GOODS[g][0]}" aria-label="${GOODS[g][0]}: ${PACK[g] || 0}"><span class="k">${i + 1}</span><img src="${goodIcon(g)}" width="32" height="32" alt=""><span class="n">${PACK[g] || 0}</span></button>`).join("");
    $("placeF").hidden = S.phase !== "placing"; $("buildF").hidden = S.phase === "placing"; $("editF").hidden = S.phase === "placing";
    $("sout").disabled = !S.check || !S.check.ok; $("bnext").disabled = S.phase !== "building";
    $("bauto").setAttribute("aria-pressed", AUTO);
  }
  let AUTO = false;
  function sframe(now) {
    if (TAB !== "site") { S3.on = false; return; }
    const dt = Math.min(0.1, (now - S3.last) / 1000); S3.last = now;
    if (AUTO && SITE.phase === "building") {
      const n = nextPiece(SITE); const strokes = n ? (n.down ? 1 : strokesOf(n.up)) : 1;
      S3.acc += dt; const k = SITE.striking === (n && (n.down || n.up).id) ? SITE.k || 0 : 0;
      if (S3.acc >= TUNE.beat) { S3.acc = 0; SITE.striking = n && (n.down || n.up).id; SITE.k = k + 1; if (n) say(`${label(n.down || n.up)}: stroke ${SITE.k} of ${strokes}`); if (SITE.k >= strokes) { SITE.k = 0; SITE.striking = null; if (!strike(SITE)) { AUTO = false; } } }
    }
    if (siteDirty) srebuild();
    const O = S3.ORB, cp = Math.cos(O.pitch); S3.camera.position.set(O.tx + Math.sin(O.yaw) * cp * O.dist, O.ty + Math.sin(O.pitch) * O.dist, O.tz + Math.cos(O.yaw) * cp * O.dist); S3.camera.lookAt(O.tx, O.ty, O.tz);

    S3.renderer.render(S3.scene, S3.camera); requestAnimationFrame(sframe);
  }
  function showSite() { if (!S3) initSite(); sresize(); siteDirty = true; if (!S3.on) { S3.on = true; S3.last = performance.now(); requestAnimationFrame(sframe); } }
  // site controls
  function refreshPlanSelect() {
    const opts = PRESETS.map(pr => `<option value="p:${pr.id}">${pr.label}</option>`).concat(MINE.map((m, i) => `<option value="m:${i}">${esc(m.name)}</option>`));
    if (SITE && !SITE.plan.preset && !SITE.plan.mine) opts.unshift(`<option value="cur">${esc(SITE.plan.name)}</option>`);
    $("splan").innerHTML = opts.join("");
    if (SITE) { const v = SITE.plan.mine ? `m:${MINE.findIndex(m => m.name === SITE.plan.mine)}` : SITE.plan.preset ? `p:${SITE.plan.preset}` : "cur"; $("splan").value = v; for (const b of $("ssize").children) b.setAttribute("aria-pressed", +b.dataset.sz === SITE.plan.size); }
  }
  function planFromSelect(v, size) { if (v.startsWith("p:")) return tidy(makePreset(PRESETS.find(p => p.id === v.slice(2)), size)); if (v.startsWith("m:")) { const m = MINE[+v.slice(2)]; const p = clonePlan(m.plan); p.mine = m.name; return p; } return clonePlan(SITE.plan); }
  $("splan").addEventListener("change", () => { const r = applyPlan(SITE, planFromSelect($("splan").value, SITE.plan.size ?? 1)); if (!r.ok) refreshPlanSelect(); });
  for (const b of $("ssize").children) b.addEventListener("click", () => { const r = applyPlan(SITE, planFromSelect($("splan").value, +b.dataset.sz)); if (r.ok) refreshPlanSelect(); });
  $("sedit").addEventListener("click", () => { PL.plan = clonePlan(SITE.plan); PL.storey = 0; PL.edit = SITE.phase === "placing" ? null : SITE; if (!PL.edit) { /* still a ghost: the planner just edits the plan */ } openTab("planner"); });
  $("tl").addEventListener("click", () => { SITE.turn = (SITE.turn + 6 - TUNE.rotStep) % 6; siteDirty = true; });
  $("tr").addEventListener("click", () => { SITE.turn = (SITE.turn + TUNE.rotStep) % 6; siteDirty = true; });
  $("sout").addEventListener("click", () => {
    const w = checkWorld(SITE.plan, SITE.origin, SITE.turn); if (!w.ok) { say(w.reason); return; }
    SITE.base = w.base; SITE.phase = "building"; SITE.mpieces = new Map();
    if (TUNE.world === "free") { for (const p of piecesOf(SITE.plan)) if (p.cls !== "slot") SITE.built.set(p.id, p.sig); SITE.phase = "done"; say("Free building: filled whole"); }
    else say("Set out. Strike a ghost, or Strike next");
    siteDirty = true;
  });
  $("bnext").addEventListener("click", () => { strike(SITE); });
  $("bauto").addEventListener("click", () => { AUTO = !AUTO; S3.acc = 0; shud(); });
  $("bfill").addEventListener("click", () => { addTo(PACK, { planks: 40, stone: 30, log: 10, rope: 6, fibre: 20, lantern: 3 }); siteDirty = true; say("Pack filled"); });
  $("bclear").addEventListener("click", () => { AUTO = false; if (SITE) newSite(clonePlan(SITE.plan)); Object.assign(PACK, PACK0); });
  // the quick changes mid-build
  const EDITS = [
    ["room", "Another room", p => { const fp = p.st[0]; let best = null; for (const k of fp) { const [q, r] = unK(k); for (const d of [0, 5, 1]) { const n = K(...nbr(q, r, d)); if (!fp.has(n) && hexDist(...unK(n)) <= TUNE.maxRadius && (!best || hx(...unK(n)) > hx(...unK(best)))) best = n; } } if (best) fp.add(best); }],
    ["drop", "Drop a room", p => { const fp = [...p.st[0]].sort((a, b) => hx(...unK(b)) - hx(...unK(a)) || (a < b ? -1 : 1)); if (fp.length < 2) return; for (const k of fp) { const landing = p.stairs.some(t => K(...nbr(...nbr(t.q, t.r, t.d), t.d)) === k); if (p.stairs.some(t => stairCells(t).includes(k)) || landing) continue; for (const c of p.st) c.delete(k); prune(p); return; } }],
    ["storey", "A storey more", p => { if (p.st.length >= TUNE.maxStoreys) return; p.st.push(new Set(p.st[p.st.length - 1])); autoStairs(p); }],
    ["kit", "Stone instead", p => { p.kit = p.kit === "stone" ? "timber" : "stone"; }],
  ];
  $("edits").innerHTML = EDITS.map(([k, l]) => `<button class="btn" data-e="${k}">${l}</button>`).join("");
  for (const b of $("edits").children) b.addEventListener("click", () => { const p = clonePlan(SITE.plan); EDITS.find(e => e[0] === b.dataset.e)[2](p); p.preset = SITE.plan.preset ? null : p.preset; if (SITE.plan.preset) { p.name = SITE.plan.name.replace(/ \(changed\)$/, "") + " (changed)"; } applyPlan(SITE, p); refreshPlanSelect(); });
  const segPair = (a, b, f) => { $(a).addEventListener("click", () => { f(true); $(a).setAttribute("aria-pressed", true); $(b).setAttribute("aria-pressed", false); }); $(b).addEventListener("click", () => { f(false); $(a).setAttribute("aria-pressed", false); $(b).setAttribute("aria-pressed", true); }); };
  segPair("wPlan", "wFree", v => { TUNE.world = v ? "planned" : "free"; siteDirty = true; });
  segPair("sNight", "sDay", v => { NIGHT = v; siteDirty = true; });
  segPair("tScafOn", "tScafOff", v => { TUNE.scaffold = v; if (SITE) scaffoldStep(SITE); siteDirty = true; });
  segPair("tMark", "tReturn", v => { TUNE.removal = v ? "mark" : "return"; });
  for (const b of $("ttint").children) { b.setAttribute("aria-pressed", b.dataset.v === TUNE.tint); b.addEventListener("click", () => { TUNE.tint = b.dataset.v; for (const c of $("ttint").children) c.setAttribute("aria-pressed", c === b); siteDirty = true; }); }
  const FMT = { ghostOpacity: v => v.toFixed(2), waitOpacity: v => v.toFixed(2), strokeScale: v => "x" + v, beat: v => v.toFixed(2) + " s", rotStep: v => v * 60 + "\u00b0", maxFooting: v => v + (v === 1 ? " layer" : " layers"), costScale: v => "x" + v, refund: v => Math.round(v * 100) + "%", maxCells: v => v, maxStoreys: v => v };
  for (const k of Object.keys(FMT)) { const el = $("t-" + k); if (!el) continue; el.value = TUNE[k]; $("o-" + k).textContent = FMT[k](TUNE[k]); el.addEventListener("input", () => { TUNE[k] = +el.value; $("o-" + k).textContent = FMT[k](TUNE[k]); siteDirty = true; if (TAB === "planner") prefresh(); }); }

  // ================================================================= tabs and the info modal
  const PANES = ["planner", "site", "builder"];
  function openTab(name, fromHash) {
    if (!PANES.includes(name)) name = "planner";
    TAB = name;
    for (const b of document.querySelectorAll(".tab")) b.setAttribute("aria-selected", b.dataset.tab === name);
    for (const p of document.querySelectorAll(".pane")) p.hidden = p.dataset.pane !== name;
    if (!fromHash && location.hash !== "#" + name && !(name === "builder" && BUILDER_HASH[location.hash])) history.replaceState(null, "", "#" + name);
    if (name === "planner") { requestAnimationFrame(() => prefresh()); }
    if (name === "site") { if (!SITE) newSite(clonePlan(PL.plan)); refreshPlanSelect(); showSite(); }
    if (name === "builder") { if (!window.__builderShow) initBuilder(); window.__builderShow(); }
  }
  window.openTab = openTab;
  for (const b of document.querySelectorAll(".tab")) b.addEventListener("click", () => { if (b.dataset.tab === "planner" && PL.edit && TAB !== "planner") PL.edit = PL.edit; openTab(b.dataset.tab); });
  const fromHash = () => { const h = location.hash; if (BUILDER_HASH[h]) return "builder"; return h.slice(1); };
  addEventListener("hashchange", () => openTab(fromHash(), true));
  addEventListener("resize", () => { if (TAB === "planner") prefresh(); });
  const modal = $("modal");
  $("info").addEventListener("click", () => modal.showModal());
  $("mclose").addEventListener("click", () => modal.close());
  modal.addEventListener("click", e => { if (e.target === modal) modal.close(); });
  for (const a of modal.querySelectorAll(".mnav a")) a.addEventListener("click", e => { e.preventDefault(); const t = modal.querySelector(a.getAttribute("href")); if (t) t.scrollIntoView({ block: "start" }); });

  window.__planner = { PL, PRESETS, makePreset, checkPlan, piecesOf, planBill, TUNE, get SITE() { return SITE; }, strike, applyPlan, newSite, PACK, checkWorld, EDITS, clonePlan, nextPiece, setNight: v => { NIGHT = v; siteDirty = true; }, orb: () => S3 && S3.ORB };
  { const Q = new URLSearchParams(location.search); if (Q.get("t") === "day") { NIGHT = false; $("sNight").setAttribute("aria-pressed", false); $("sDay").setAttribute("aria-pressed", true); } }
  openTab(fromHash(), true);
})();
