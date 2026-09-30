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
