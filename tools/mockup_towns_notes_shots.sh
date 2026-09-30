#!/bin/bash
# The owner's eight walk-through notes (tenebris-towns design section 12), one view each,
# and the views of the second look (section 12b), as 12b-*:
# tools/mockup_towns_notes_shots.sh <out dir> [page]. Needs THREE_JS and Playwright (NODE_PATH=$(npm root -g)).
T=$(cd "$(dirname "$0")" && pwd)
OUT=$1; PAGE=${2:-$T/../docs/mockups/towns.html}; mkdir -p $OUT
run() { PAGE=$PAGE THREE_JS=${THREE_JS:?set THREE_JS to three.min.js r128} NODE_PATH=$(npm root -g) timeout 300 node $T/mockup_probe.js "$OUT" "$@" 2>&1 | tail -4; }

run coast \
  1-boats '(() => { Object.assign(ORB, { tx: cx(22,15) - 6, tz: cz(0,9), ty: 0.5, dist: 11, yaw: -1.2, pitch: 0.8 }); setView("overview"); })()' \
  2-steps '(() => { Object.assign(ORB, { tx: cx(22,21) - 0.3, tz: cz(22,21), ty: 1.3, dist: 4, yaw: -1.2, pitch: 0.75 }); setView("overview"); })()'
run village \
  3-windows-day '(() => { teleport({ x: cx(14,17), z: cz(14,17), y: 1, look: [cx(11,15), cz(11,15)] }); P.pitch = 0.2; setView("walk"); })()' \
  3-windows-night '(() => { document.getElementById("tod").value = 22.5; setTime(22.5); assignLights(); })()' \
  4-chimney '(() => { document.getElementById("tod").value = 11; setTime(11); assignLights(); const hm = { c: 12, r: 16 }; teleport({ x: cx(hm.c, hm.r) - 1.2, z: cz(hm.c, hm.r) - 0.6, y: 4, look: [cx(hm.c, hm.r), cz(hm.c, hm.r)] }); P.pitch = 1.1; setView("walk"); })()' \
  7-newel '(() => { const ex = cx(10,15), ez = cz(10,15), sx = ex - 0.548, sz = ez + 0.950; teleport({ x: ex - (sx - ex) * 0.9, z: ez - (sz - ez) * 0.9, y: 1.6, look: [sx, sz] }); P.pitch = 0.3; setView("walk"); })()' \
  7-flight '(() => { teleport({ x: 44.94, z: 37.21, y: GOTO.house.y, look: [48.24, 37.41] }); P.pitch = 0.3; setView("walk"); })()'
run mounds \
  5-door '(() => { for (const d of DOORS) { d.open = true; d.solid.off = true; } const h = GOTO.smial, ux = (h.x - h.look[0]), uz = (h.z - h.look[1]), l = Math.hypot(ux, uz); teleport({ x: h.look[0] - ux / l * 0.6, z: h.look[1] - uz / l * 0.6, y: h.y, look: [h.look[0] + ux / l * 3, h.look[1] + uz / l * 3] }); P.yaw -= 0.5; setView("walk"); })()'
run jungle \
  6-bridge '(() => { const b = GOTO.bridge, ax = (b.x - 0.1 * b.look[0]) / 0.9, az = (b.z - 0.1 * b.look[1]) / 0.9, a = Math.atan2(b.look[1] - az, b.look[0] - ax); Object.assign(ORB, { tx: ax, tz: az, ty: 9.3, dist: 5, yaw: a - 1.1, pitch: 0.9 }); setView("overview"); })()'
run tundra \
  8-igloo '(() => { const g = GOTO.igloo, dx = g.x - g.look[0], dz = g.z - g.look[1], l = Math.hypot(dx, dz); Object.assign(ORB, { tx: g.look[0] + dx / l * 2.3, tz: g.look[1] + dz / l * 2.3, ty: 1.4, dist: 7.5, yaw: Math.atan2(-dz, -dx) + 0.55, pitch: 0.35 }); setView("overview"); })()'

# Section 12b: standing on each rope bridge looking back at its entrance, the
# harbour's rowboats at two moments of their bob, the igloo's tunnel joint.
ON='(ex, ez, bx, bz) => { const L = Math.hypot(bx - ex, bz - ez), ux = (bx - ex) / L, uz = (bz - ez) / L, t = 2.6 / L, y = 9 - 4 * 0.8 * t * (1 - t); teleport({ x: ex + ux * 2.6, z: ez + uz * 2.6, y, look: [ex - ux * 3, ez - uz * 3] }); P.pitch = -0.25; setView("walk"); }'
ENDS='(k, e) => { const out = []; for (const s of SOLIDS) if (s.type === "surf" && s.label === "Rope bridge") { const p = s.pts, m = (a, b) => [(a[0] + b[0]) / 2, (a[1] + b[1]) / 2], sh = Math.hypot(p[1][0] - p[0][0], p[1][1] - p[0][1]) < Math.hypot(p[2][0] - p[1][0], p[2][1] - p[1][1]); out.push(sh ? [m(p[0], p[1]), m(p[2], p[3])] : [m(p[1], p[2]), m(p[3], p[0])]); } const [a, b] = out[k]; return e ? [b[0], b[1], a[0], a[1]] : [a[0], a[1], b[0], b[1]]; }'
run jungle \
  12b-bridge-1 "($ON)(...($ENDS)(0, 1))" 12b-bridge-2 "($ON)(...($ENDS)(2, 1))" 12b-bridge-3 "($ON)(...($ENDS)(0, 0))" 12b-bridge-4 "($ON)(...($ENDS)(1, 1))"
BO='(d, p, yaw) => { Object.assign(ORB, { tx: cx(22,15) - 6, tz: cz(0,9), ty: 0.5, dist: d, yaw: yaw, pitch: p }); setView("overview"); }'
run coast \
  12b-boats-1 "($BO)(6, 0.75, -1.2)" 12b-boats-2 '(() => new Promise(r => setTimeout(r, 1700)))()'
IG='(a, d, p, ty) => { const g = GOTO.igloo, dx = g.x - g.look[0], dz = g.z - g.look[1], l = Math.hypot(dx, dz), ux = dx / l, uz = dz / l; Object.assign(ORB, { tx: g.look[0] + ux * 2.4, tz: g.look[1] + uz * 2.4, ty: ty, dist: d, yaw: Math.atan2(-uz, -ux) + a, pitch: p }); setView("overview"); }'
run tundra \
  12b-igloo-outside "($IG)(0.9, 4.5, 0.25, 1.2)" 12b-igloo-inside "($IG)(Math.PI, 3.2, 0.1, 1.0)" 12b-igloo-mouth '(() => { const g = GOTO.igloo; Object.assign(ORB, { tx: g.look[0], tz: g.look[1], ty: 1.3, dist: 6, yaw: Math.atan2(g.z - g.look[1], g.x - g.look[0]) + Math.PI, pitch: 0.3 }); setView("overview"); })()'
