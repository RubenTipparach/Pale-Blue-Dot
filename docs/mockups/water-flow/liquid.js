// The liquid rule shared by the mockups that move water on the hex grid: M28 (water-flow.html) and M25's
// "Water sim" tab (ocean.html#sim). One copy, loaded by both pages, so the two can never disagree.
//
// A port of Luanti's ServerMap::transformLiquidsLocal and get_max_liquid_level (src/servermap.cpp at
// a10dc8dbff3b0af8295ae55df44f65b1d9ee92c7), adapted to the hex grid.
//   Luanti: Copyright (C) 2010-2024 celeron55, Perttu Ahola <celeron55@gmail.com> and contributors.
//   SPDX-License-Identifier: LGPL-2.1-or-later. This port is offered under the same licence;
//   the licence text is published beside this file as water-flow/COPYING.LESSER.
// What changed for hex (openspec/changes/water-flow, design 1 to 4): eight neighbours (up, six sides,
// down) for Luanti's six; the renew threshold is a setting, 3 on hex (Luanti's 2 runs away there); one
// liquid kind, so the liquid_kind bookkeeping falls away; cells outside the patch are solid, not
// CONTENT_IGNORE. Viscosity is water's, 1, so it is left out.
//
// PBDLiquid.create({N, Y0, Y1}) makes one hex patch of radius N cells and layers Y0..Y1, axial (q, r).
(function (root) {
  "use strict";
  const LIQUID_LEVEL_MAX = 7, LIQUID_LEVEL_SOURCE = 8, WATER_DROP_BOOST = 4;
  const AIR = 0, SOLID = 1, FLOW = 2, SRC = 3;
  const UPPER = 0, SAME = 1, LOWER = 2;
  const DIRS = [[1, 0], [0, 1], [-1, 1], [-1, 0], [0, -1], [1, -1]];
  const hexDist = (q, r) => Math.max(Math.abs(q), Math.abs(r), Math.abs(q + r));
  // Luanti's order: upper before same level before lower
  const NB8 = [[0, 0, 1, UPPER], ...DIRS.map(([a, b]) => [a, b, 0, SAME]), [0, 0, -1, LOWER]];

  function get_max_liquid_level(nbLvl, nbDown, t, cur) {
    let max = cur;
    if (t === UPPER) {
      if (nbLvl + WATER_DROP_BOOST > cur) { max = LIQUID_LEVEL_MAX; if (nbLvl + WATER_DROP_BOOST < LIQUID_LEVEL_MAX) max = nbLvl + WATER_DROP_BOOST; }
      else if (nbLvl > cur) max = nbLvl;
    } else if (t === SAME) {
      if (!nbDown && nbLvl > 0 && nbLvl - 1 > max) max = nbLvl - 1;
    }
    return max;
  }

  function create(o) {
    const N = o.N, Y0 = o.Y0, Y1 = o.Y1, NY = Y1 - Y0 + 1, SIDE = 2 * N + 1, NCELL = SIDE * SIDE * NY;
    const kind = new Uint8Array(NCELL), lvl = new Uint8Array(NCELL), down = new Uint8Array(NCELL);
    const idx = (q, r, y) => ((q + N) * SIDE + (r + N)) * NY + (y - Y0);
    const inside = (q, r, y) => hexDist(q, r) <= N && y >= Y0 && y <= Y1;
    const unidx = i => { const y = i % NY + Y0, t = (i / NY) | 0; return [((t / SIDE) | 0) - N, t % SIDE - N, y]; };
    const SIM = { range: o.range ?? 8, renew: o.renew ?? 3, queue: [], inQ: new Uint8Array(NCELL), tick: 0, changed: 0, lastChanged: 0, LOOP_MAX: 100000 };
    // Luanti's UniqueQueue: a push of a cell already waiting does nothing
    function qpush(i) { if (!SIM.inQ[i]) { SIM.inQ[i] = 1; SIM.queue.push(i); } }
    function qpop() { const i = SIM.queue.shift(); SIM.inQ[i] = 0; return i; }
    const flows = [], airs = [];
    function transformOne(p0) {
      const k0 = kind[p0];
      if (k0 === SOLID) return false;                          // LIQUID_NONE and not floodable
      const [q, r, y] = unidx(p0);
      let num_sources = 0, flowing_down = false; flows.length = 0; airs.length = 0;
      for (const [dq, dr, dy, t] of NB8) {
        const nq = q + dq, nr = r + dr, ny = y + dy;
        if (!inside(nq, nr, ny)) continue;                     // outside the patch: a neutral
        const n = idx(nq, nr, ny), kn = kind[n];
        if (kn === AIR) {                                      // floodable
          airs.push([n, t]);
          if (t !== UPPER && k0 !== AIR) qpush(n);
          if (t === LOWER) flowing_down = true;
        } else if (kn === SRC) {
          if (t !== LOWER) num_sources++;                      // do not count the source below
        } else if (kn === FLOW) {
          flows.push([n, t]);
          if (t === LOWER) flowing_down = true;
        }                                                      // SOLID: a neutral
      }
      // decide on the type (and possibly level) of the current cell
      const range = Math.min(SIM.range, LIQUID_LEVEL_MAX + 1);
      let newK, newL = -1;
      if (num_sources >= SIM.renew || k0 === SRC) newK = SRC;
      else if (num_sources >= 1) { newL = LIQUID_LEVEL_MAX; newK = newL >= LIQUID_LEVEL_MAX + 1 - range ? FLOW : AIR; }
      else {
        let max = -1; for (const [n, t] of flows) max = get_max_liquid_level(lvl[n], down[n], t, max);
        newL = max; newK = max >= LIQUID_LEVEL_MAX + 1 - range ? FLOW : AIR;
      }
      // has anything changed?
      if (newK === k0 && (k0 !== FLOW || (lvl[p0] === newL && !!down[p0] === flowing_down))) return false;
      kind[p0] = newK;
      if (newK === FLOW) { lvl[p0] = newL & 7; down[p0] = flowing_down ? 1 : 0; } else { lvl[p0] = 0; down[p0] = 0; }
      // enqueue neighbours
      if (newK === SRC || newK === FLOW) { for (const [n, t] of flows) if (t !== UPPER) qpush(n); for (const [n, t] of airs) if (t !== UPPER) qpush(n); }
      else for (const [n] of flows) qpush(n);                 // this flow has turned to air; neighbouring flows may follow
      return true;
    }
    // one tick: the whole queue at most once, as transformLiquids rate-limits it
    function tick() {
      const max = Math.min(SIM.queue.length, SIM.LOOP_MAX); let changed = 0;
      for (let c = 0; c < max && SIM.queue.length; c++) if (transformOne(qpop())) changed++;
      SIM.tick++; SIM.lastChanged = changed; return changed;
    }
    // an edit queues the cell and its eight neighbours
    function touch(q, r, y) { for (const [dq, dr, dy] of [[0, 0, 0], ...NB8]) if (inside(q + dq, r + dr, y + dy)) qpush(idx(q + dq, r + dr, y + dy)); }
    function clear() { kind.fill(AIR); lvl.fill(0); down.fill(0); SIM.queue.length = 0; SIM.inQ.fill(0); SIM.tick = 0; SIM.lastChanged = 0; }
    return { N, Y0, Y1, NY, SIDE, NCELL, kind, lvl, down, idx, inside, unidx, SIM, qpush, qpop, transformOne, tick, touch, clear };
  }

  root.PBDLiquid = { create, get_max_liquid_level, hexDist, DIRS, NB8, LIQUID_LEVEL_MAX, LIQUID_LEVEL_SOURCE, WATER_DROP_BOOST, AIR, SOLID, FLOW, SRC, UPPER, SAME, LOWER };
})(typeof window !== "undefined" ? window : globalThis);
