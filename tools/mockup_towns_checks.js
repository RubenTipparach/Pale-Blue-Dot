// Checks the towns mockup (docs/mockups/towns.html) settlement by settlement,
// for the faults the owner found on the walk-through of 2026-09-29
// (openspec/changes/tenebris-towns/design.md, sections 11 and 12):
//   - faces dropped for having no area (and none left with a zero normal);
//   - holes in street steps: a ray down onto every step cell must land on a
//     top face that faces up, no lower than the step's foot;
//   - faces of one plane that overlap by more than 30 cm2, facing up and
//     facing down (a chimney's bottom on the soffit was a downward one);
//   - doors turned outward because their leaf would sweep through furniture;
//   - stairwell sconces, window panes, and hulls that mask the water.
//
// Usage: THREE_JS=<three.min.js r128> NODE_PATH=$(npm root -g) \
//        node tools/mockup_towns_checks.js [page.html] > report.json
const fs = require("fs");
const path = require("path");
const { chromium } = require("playwright");

const page_path = path.resolve(process.argv[2] || path.join(__dirname, "../docs/mockups/towns.html"));

function inPage() {
  const has = (name) => { try { return eval(`typeof ${name} !== "undefined"`); } catch (e) { return false; } };
  const out = {};
  for (const key of Object.keys(SCENES)) {
    loadScene(key);
    const r = { scene: key };
    // Faces: dropped for no area, and any left with a zero normal.
    let zero = 0, faces = 0;
    for (const b of Object.values(BUF)) for (const f of b.faces) { faces++; const s = f[3]; if (Math.hypot(b.n[3 * s], b.n[3 * s + 1], b.n[3 * s + 2]) < 0.5) zero++; }
    r.faces = faces; r.zeroNormal = zero; r.dropped = has("FACE_STATS") ? FACE_STATS.dropped : null;
    // Step holes: rays straight down onto every step cell.
    const ray = new THREE.Raycaster(); let rays = 0, holes = 0;
    for (const i of STEPCELL) {
      const c = i % NC, rr = (i - c) / NC, x0 = cx(c, rr), z0 = cz(c, rr), hex = hexPts(c, rr);
      for (let a = -4; a <= 4; a++) for (let b = -4; b <= 4; b++) {
        const px = x0 + a * 0.3, pz = z0 + b * 0.3; if (!inPoly(hex, px, pz)) continue;
        rays++;
        ray.set(new THREE.Vector3(px, TOP[i] + 6, pz), new THREE.Vector3(0, -1, 0));
        const hit = ray.intersectObjects(STATIC, false)[0];
        if (!hit || hit.point.y < TOP[i] - 0.02 || !hit.face || hit.face.normal.y < 0.3) holes++;
      }
    }
    r.stepRays = rays; r.stepHoles = holes;
    // Coplanar overlaps, up and down.
    const groups = new Map();
    for (const [m, b] of Object.entries(BUF)) for (const f of b.faces) {
      const s = f[3], cnt = f[4], ny = b.n[3 * s + 1]; if (Math.abs(ny) < 0.99) continue;
      const v = k => [b.p[3 * k], b.p[3 * k + 2]], poly = [v(s), v(s + 1), v(s + 2)];
      for (let t = 1; t < cnt / 3; t++) poly.push(v(s + 3 * t + 2));
      const y = b.p[3 * s + 1], key = (ny > 0 ? "u" : "d") + Math.round(y * 500);
      let minx = 1e9, maxx = -1e9, minz = 1e9, maxz = -1e9; for (const p of poly) { minx = Math.min(minx, p[0]); maxx = Math.max(maxx, p[0]); minz = Math.min(minz, p[1]); maxz = Math.max(maxz, p[1]); }
      if (!groups.has(key)) groups.set(key, []);
      groups.get(key).push({ m, poly, minx, maxx, minz, maxz });
    }
    const area = p => { let a = 0; for (let i = 0; i < p.length; i++) { const q = p[(i + 1) % p.length]; a += p[i][0] * q[1] - q[0] * p[i][1]; } return a / 2; };
    const ccw = p => area(p) < 0 ? p.slice().reverse() : p;
    const clip = (subject, clipper) => {   // Sutherland-Hodgman, both convex and counter-clockwise
      let out = subject;
      for (let i = 0; i < clipper.length && out.length; i++) {
        const a = clipper[i], b = clipper[(i + 1) % clipper.length], inside = p => (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]) >= -1e-9;
        const inp = out; out = [];
        for (let j = 0; j < inp.length; j++) {
          const p = inp[j], q = inp[(j + 1) % inp.length], pi = inside(p), qi = inside(q);
          if (pi) out.push(p);
          if (pi !== qi) { const d1 = (b[0] - a[0]) * (p[1] - a[1]) - (b[1] - a[1]) * (p[0] - a[0]), d2 = (b[0] - a[0]) * (q[1] - a[1]) - (b[1] - a[1]) * (q[0] - a[0]), t = d1 / (d1 - d2); out.push([p[0] + (q[0] - p[0]) * t, p[1] + (q[1] - p[1]) * t]); }
        }
      }
      return out;
    };
    const overlaps = { u: 0, d: 0 }, pairs = { u: {}, d: {} };
    for (const [key, list] of groups) {
      const dir = key[0];
      for (let i = 0; i < list.length; i++) for (let j = i + 1; j < list.length; j++) {
        const A = list[i], B = list[j];
        if (A.maxx <= B.minx || B.maxx <= A.minx || A.maxz <= B.minz || B.maxz <= A.minz) continue;
        const inter = clip(ccw(A.poly), ccw(B.poly));
        if (inter.length >= 3 && Math.abs(area(inter)) > 0.003) { overlaps[dir]++; const k = [A.m, B.m].sort().join("/"); pairs[dir][k] = (pairs[dir][k] || 0) + 1; }
      }
    }
    r.overlapUp = overlaps.u; r.overlapDown = overlaps.d;
    r.overlapDownPairs = Object.entries(pairs.d).sort((a, b) => b[1] - a[1]).slice(0, 6);
    // Doors, sconces, glass, masks.
    r.doors = DOORS.length;
    r.doorsOutward = has("DOOR_STATS") ? DOOR_STATS.outward : null;
    r.doorsStillBlocked = has("DOOR_STATS") ? DOOR_STATS.stillBlocked : null;
    r.sconces = has("SCONCES") ? SCONCES.count : null;
    r.windows = WINDOWS.length;
    r.panes = has("GLASS") ? GLASS.panes : null;
    r.masks = has("MASKS") ? MASKS.count : null;
    r.roofClashes = ROOF_CLASHES.length;
    // The owner's second notes (2026-09-30, design section 12b).
    // Rope bridges: ends shared by two bridges, and rails over a deck.
    const decks = SOLIDS.filter(s => s.type === "surf" && s.label === "Rope bridge");
    const ends = decks.map(s => { const p = s.pts, m = (a, b) => [(a[0] + b[0]) / 2, (a[1] + b[1]) / 2], short = Math.hypot(p[1][0] - p[0][0], p[1][1] - p[0][1]) < Math.hypot(p[2][0] - p[1][0], p[2][1] - p[1][1]); return short ? [m(p[0], p[1]), m(p[2], p[3])] : [m(p[1], p[2]), m(p[3], p[0])]; });
    let shared = 0;
    for (let i = 0; i < ends.length; i++) for (let j = i + 1; j < ends.length; j++) for (const a of ends[i]) for (const b of ends[j]) if (Math.hypot(a[0] - b[0], a[1] - b[1]) < 0.3) shared++;
    const sat = (A, B) => { for (const P of [A, B]) for (let i = 0; i < P.length; i++) { const a = P[i], b = P[(i + 1) % P.length], nx = b[1] - a[1], nz = a[0] - b[0]; let a0 = 1e9, a1 = -1e9, b0 = 1e9, b1 = -1e9; for (const q of A) { const d = q[0] * nx + q[1] * nz; a0 = Math.min(a0, d); a1 = Math.max(a1, d); } for (const q of B) { const d = q[0] * nx + q[1] * nz; b0 = Math.min(b0, d); b1 = Math.max(b1, d); } if (a1 <= b0 || b1 <= a0) return false; } return true; };
    const shrink = (P, m) => { const cxm = P.reduce((s, q) => s + q[0], 0) / P.length, czm = P.reduce((s, q) => s + q[1], 0) / P.length; return P.map(q => { const dx = q[0] - cxm, dz = q[1] - czm, l = Math.hypot(dx, dz); return [q[0] - dx / l * m, q[1] - dz / l * m]; }); };
    let over = 0;
    for (const d of decks) { const D = shrink(d.pts, 0.05); for (const s of SOLIDS) if (s.label === "rail" && s.y1 > d.y0 && s.y0 < d.y1 + 0.5 && sat(D, s.pts)) over++; }
    r.bridgeEndsShared = shared; r.railsOverBridgeDecks = over;
    // Hulls: water left showing inside each hull where its mask falls short
    // of the hull's own waterline (the mesh cut by the water plane), with the
    // boat at rest and at the ends of its bob, pitch and roll, seen at 45
    // degrees from four sides: the mask's shape is carried down onto the
    // water along the line of sight. cm2 in all, and the widest sliver in cm.
    const inTri = (P, x, z) => { let c = false; for (let i = 0, j = P.length - 1; i < P.length; j = i++) if ((P[i][1] > z) !== (P[j][1] > z) && x < (P[j][0] - P[i][0]) * (z - P[i][1]) / (P[j][1] - P[i][1]) + P[i][0]) c = !c; return c; };
    const segDist = (x, z, a, b) => { const dx = b[0] - a[0], dz = b[1] - a[1], l2 = dx * dx + dz * dz || 1, t = Math.max(0, Math.min(1, ((x - a[0]) * dx + (z - a[1]) * dz) / l2)); return Math.hypot(x - a[0] - dx * t, z - a[1] - dz * t); };
    let waterCm2 = 0, widest = 0, masked = 0;
    const maskMeshes = []; scene.traverse(o => { if (o.isMesh && o.material && o.material.stencilWrite && o.material.colorWrite === false) maskMeshes.push(o); });
    const vw = new THREE.Vector3();
    for (const o of maskMeshes) {
      const hull = o.userData.hull || o.parent.children.find(c => c.isMesh && c !== o && Object.values(HULLS).includes(c.geometry)); if (!hull) continue;
      masked++;
      const grp = hull.parent, bob = BOBBERS.find(b => b.obj === grp), fresh = o.userData.hull && typeof cutMask === "function" ? MASKS.list.find(k => k.mesh === o) : null;
      const pose = (sy, sp, sr) => { if (bob) { grp.position.y = bob.y + sy * bob.amp; grp.rotation.x = sp * bob.amp * 0.5; grp.rotation.z = sr * bob.amp * 0.8; } grp.updateMatrixWorld(true); if (fresh) cutMask(fresh); o.updateMatrixWorld(true); };
      pose(0, 0, 0);
      const mp = o.geometry.attributes.position; vw.fromBufferAttribute(mp, 0).applyMatrix4(o.matrixWorld); const wl = vw.y - (fresh ? 0.002 : 0);
      for (const [sy, sp, sr] of bob ? [[0, 0, 0], [1, 1, 1], [-1, -1, -1], [-1, 1, -1], [1, -1, 1]] : [[0, 0, 0]]) {
        pose(sy, sp, sr);
        // the hull's waterline in the world, as cut segments
        const g = hull.geometry, hp = g.attributes.position, hi = g.index.array, Wv = [];
        for (let i = 0; i < hp.count; i++) { vw.fromBufferAttribute(hp, i).applyMatrix4(hull.matrixWorld); Wv.push([vw.x, vw.y, vw.z]); }
        const segs = [];
        for (let k = 0; k < hi.length; k += 3) { const cut = []; for (let j = 0; j < 3; j++) { const a = Wv[hi[k + j]], b = Wv[hi[k + (j + 1) % 3]]; if ((a[1] - wl) * (b[1] - wl) < 0) { const t = (wl - a[1]) / (b[1] - a[1]); cut.push([a[0] + (b[0] - a[0]) * t, a[2] + (b[2] - a[2]) * t]); } } if (cut.length === 2) segs.push(cut); }
        const mi = o.geometry.index ? o.geometry.index.array : [...Array(o.geometry.drawRange.count === Infinity ? mp.count : Math.min(mp.count, o.geometry.drawRange.count)).keys()];
        const mw = []; for (const i of mi) { vw.fromBufferAttribute(mp, i).applyMatrix4(o.matrixWorld); mw.push([vw.x, vw.y, vw.z]); }
        for (const az of [0, 90, 180, 270]) {
          const a = az * Math.PI / 180, e = Math.PI / 4, dir = [Math.cos(a) * Math.cos(e), -Math.sin(e), Math.sin(a) * Math.cos(e)];
          const tris = []; for (let k = 0; k + 2 < mw.length; k += 3) tris.push([0, 1, 2].map(j => { const q = mw[k + j], t = (wl - q[1]) / dir[1]; return [q[0] + dir[0] * t, q[2] + dir[2] * t]; }));
          for (const [p0, p1] of segs) {
            const L = Math.hypot(p1[0] - p0[0], p1[1] - p0[1]), n = Math.max(1, Math.ceil(L / 0.01));
            for (let k = 0; k < n; k++) {
              const x = p0[0] + (p1[0] - p0[0]) * (k + 0.5) / n, z = p0[1] + (p1[1] - p0[1]) * (k + 0.5) / n;
              if (tris.some(t => inTri(t, x, z))) continue;
              let d = 1e9; for (const t of tris) for (let j = 0; j < 3; j++) d = Math.min(d, segDist(x, z, t[j], t[(j + 1) % 3]));
              if (d > 2) continue; // not this hull's mask at all
              waterCm2 += d * 100 * (L / n) * 100; widest = Math.max(widest, d * 100);
            }
          }
        }
      }
      pose(0, 0, 0);
    }
    r.hullsMasked = masked; r.waterInHullsCm2 = Math.round(waterCm2); r.widestSliverCm = +widest.toFixed(1);
    // The igloo: rays from inside that leave it other than by the tunnel's
    // mouth, through a gap where the tunnel meets the dome.
    if (GOTO.igloo) {
      const g = GOTO.igloo, ox = g.look[0], oz = g.look[1], base = g.y, l = Math.hypot(g.x - ox, g.z - oz), ux = (g.x - ox) / l, uz = (g.z - oz) / l;
      const b = BUF.snowblock, near = [];
      for (let k = 0; k < b.p.length; k += 9) { const cxm = (b.p[k] + b.p[k + 3] + b.p[k + 6]) / 3, czm = (b.p[k + 2] + b.p[k + 5] + b.p[k + 8]) / 3; if (Math.hypot(cxm - ox, czm - oz) < 5) near.push(...b.p.slice(k, k + 9)); }
      const geo = new THREE.BufferGeometry(); geo.setAttribute("position", new THREE.Float32BufferAttribute(near, 3));
      const mesh = new THREE.Mesh(geo, new THREE.MeshBasicMaterial({ side: THREE.DoubleSide })); mesh.updateMatrixWorld();
      const rc = new THREE.Raycaster(); let rays = 0, leaks = 0;
      const mouth = (px, py, pz) => { const u = (px - ox) * ux + (pz - oz) * uz, v = Math.abs(-(px - ox) * uz + (pz - oz) * ux), y = py - base; return u > 3.5 && v < 0.72 && y < 1.0 + 0.95 * Math.sqrt(Math.max(0, 1 - (v / 0.72) ** 2)); };
      for (const h of [0.7, 1.3, 1.9]) for (let az = -70; az <= 70; az += 1.5) for (let el = -5; el <= 80; el += 1.5) {
        // a leak must leak on rays a twentieth of a degree either side too: a
        // ray exactly along an edge between two faces can miss both
        const cast = (daz, del) => { const a = Math.atan2(uz, ux) + (az + daz) * Math.PI / 180, e = (el + del) * Math.PI / 180, d = new THREE.Vector3(Math.cos(a) * Math.cos(e), Math.sin(e), Math.sin(a) * Math.cos(e)); rc.set(new THREE.Vector3(ox, base + h, oz), d); rc.far = 6; return rc.intersectObject(mesh, false).length ? null : d; };
        rays++;
        const dir = cast(0, 0);
        if (!dir || !cast(0.05, 0.05) || !cast(-0.05, -0.05)) continue;
        // no snow block within 6 m: out through the mouth, or through a gap
        const t = 3.9 / Math.max(1e-6, dir.x * ux + dir.z * uz), px = ox + dir.x * t, py = base + h + dir.y * t, pz = oz + dir.z * t;
        if (dir.x * ux + dir.z * uz > 0 && mouth(px, py, pz)) continue;
        leaks++;
      }
      r.iglooRays = rays; r.iglooLeaks = leaks;
    }
    out[key] = r;
  }
  return out;
}

(async () => {
  const browser = await chromium.launch({ executablePath: "/opt/pw-browsers/chromium", args: ["--use-gl=swiftshader", "--enable-unsafe-swiftshader"] });
  const page = await browser.newPage({ viewport: { width: 800, height: 500 } });
  const errors = [];
  page.on("pageerror", (e) => errors.push(String(e)));
  const body = fs.readFileSync(process.env.THREE_JS);
  await page.route("**/three.min.js", (r) => r.fulfill({ body, contentType: "text/javascript" }));
  await page.goto(`file://${page_path}#village`);
  await page.waitForFunction(() => typeof SCENE !== "undefined" && document.getElementById("loading").hidden, null, { timeout: 90000 });
  const report = await page.evaluate(`(${inPage.toString()})()`);
  report.errors = errors;
  console.log(JSON.stringify(report, null, 1));
  await browser.close();
})();
