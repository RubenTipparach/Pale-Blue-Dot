// The mockup's water rule as a test fixture for `pbd_core::liquid` (openspec/changes/water-flow, task 2.1/2.2).
//
// Runs `docs/mockups/water-flow/liquid.js` (the LGPL port the mockup draws with) on M25's Water sim patches with the
// presets of `docs/mockups/ocean.html` (SIM_PRESETS), settles each, and writes what the rule left: the patch, its
// ground, the edits, the ticks to settle and every water cell. The Rust test builds the same patch from the same
// ground, makes the same edits and must leave the same field, cell for cell. Nothing of liquid.js is copied into the
// game: the fixture is data the mockup produced.
//
//   node tools/water_flow_fixture.mjs > crates/pbd-core/tests/fixtures/water_flow_presets.txt
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const here = fileURLToPath(new URL(".", import.meta.url));
const ctx = { globalThis: {} };
vm.createContext(ctx);
vm.runInContext(readFileSync(here + "../docs/mockups/water-flow/liquid.js", "utf8"), ctx);
const L = ctx.globalThis.PBDLiquid;
const { AIR, SOLID, FLOW, SRC, DIRS } = L;

// ocean.html's own geometry and grounds, copied as the page has them
const HW = 2.833, HR = HW / Math.sqrt(3);
const hx = (q, r) => HW * (q + r / 2), hz = r => 1.5 * HR * r;
const hexDistA = (q, r) => Math.max(Math.abs(q), Math.abs(r), Math.abs(q + r));
const m28Ground = (q, r) => { const x = hx(q, r), z = hz(r); if (x < -17) return -5; if (x < -9) return Math.min(0, Math.round(-5 + (x + 17) / 8 * 5.6)); if (hexDistA(q - 1, r - 1) <= 2) return -2; if (x > 17 && z > -6) return 9; if (x > 9) return 6; return 0; };
const PATCH = { m28: { N: 12, Y0: -6, Y1: 13 }, m25: { N: 9, Y0: 0, Y1: 15 } };
const PRESETS = {
  spring: { patch: "m28", run: S => S.pour(7, -2, S.topOf(7, -2)) },
  channel: { patch: "m28", run: S => { for (let q = -9; q <= 0; q++) S.dig(q, 1, -1); } },
  basin: { patch: "m28", run: S => { for (const k of [0, 2, 4]) S.pour(1 + DIRS[k][0], 1 + DIRS[k][1], -2); } },
  pair: { patch: "m28", renew: 3, run: S => { S.pour(-2, -5, 0); S.pour(-1, -5, 0); } },
  runaway: { patch: "m28", renew: 2, run: S => { S.pour(-2, -5, 0); S.pour(-1, -5, 0); } },
  recede: { patch: "m28", run: S => { const y = S.topOf(7, -2); S.pour(7, -2, y); S.settle(400); S.dig(7, -2, y); } },
  pond: { patch: "m25", ground: (q, r) => hexDistA(q, r) <= 3 ? 1 : 3, run: S => { for (const k of [0, 2, 4]) S.pour(DIRS[k][0], DIRS[k][1], 1); } },
  fall: { patch: "m25", ground: (q, r) => { const x = hx(q, r); return x < -14 ? 11 : x < -8 ? 9 : x < -3 ? 7 : x < 3 ? 5 : x < 9 ? 3 : 1; }, run: S => S.pour(-7, 0, 11) },
};

const out = [];
for (const [name, Pr] of Object.entries(PRESETS)) {
  const o = PATCH[Pr.patch], LQ = L.create(o), { kind, lvl, down, idx, inside, SIM } = LQ;
  const ground = Pr.patch === "m28" ? m28Ground : Pr.ground, beach = Pr.patch === "m28" ? (q, r) => hx(q, r) < -6 : null;
  const edits = [];
  for (let q = -o.N; q <= o.N; q++) for (let r = -o.N; r <= o.N; r++) {
    if (hexDistA(q, r) > o.N) continue;
    const h = ground(q, r);
    for (let y = o.Y0; y < h; y++) kind[idx(q, r, y)] = SOLID;
    if (beach && beach(q, r)) for (let y = h; y < 0; y++) kind[idx(q, r, y)] = SRC;
  }
  SIM.renew = Pr.renew ?? 3; SIM.range = 8;
  const setCell = (q, r, y, k, tag) => { if (!inside(q, r, y)) return false; const i = idx(q, r, y); kind[i] = k; lvl[i] = 0; down[i] = 0; LQ.touch(q, r, y); edits.push(`${tag} ${q} ${r} ${y} ${SIM.tick}`); return true; };
  let pre = 0;
  const S = {
    pour: (q, r, y) => inside(q, r, y) && kind[idx(q, r, y)] !== SOLID && setCell(q, r, y, SRC, "pour"),
    dig: (q, r, y) => inside(q, r, y) && kind[idx(q, r, y)] !== AIR && y > o.Y0 && setCell(q, r, y, AIR, "dig"),
    topOf: (q, r) => { let y = o.Y1; while (y > o.Y0 && kind[idx(q, r, y)] !== SOLID) y--; return y + 1; },
    settle: (max = 2000) => { let n = 0; while (SIM.queue.length && n < max) { LQ.tick(); n++; } pre += n; return n; },
  };
  Pr.run(S);
  let ticks = 0; while (SIM.queue.length && ticks < 5000) { LQ.tick(); ticks++; }
  let s = 0, f = 0, d = 0; const cells = [];
  for (let i = 0; i < LQ.NCELL; i++) {
    if (kind[i] === SRC) s++; else if (kind[i] === FLOW) { f++; if (down[i]) d++; } else continue;
    const [q, r, y] = LQ.unidx(i);
    cells.push(`${q} ${r} ${y} ${kind[i] === SRC ? "s" : (down[i] ? "d" : "f") + lvl[i]}`);
  }
  out.push(`preset ${name} ${Pr.patch} ${o.N} ${o.Y0} ${o.Y1} renew ${SIM.renew} ticks ${ticks} sources ${s} flowing ${f} falling ${d}`);
  const g = [];
  for (let q = -o.N; q <= o.N; q++) for (let r = -o.N; r <= o.N; r++) if (hexDistA(q, r) <= o.N) g.push(`${q},${r},${ground(q, r)}${beach && beach(q, r) ? "s" : ""}`);
  out.push(`ground ${g.join(" ")}`);
  for (const e of edits) out.push(`edit ${e}`);
  for (const c of cells) out.push(`cell ${c}`);
  out.push("end");
}
console.log(out.join("\n"));
