// River Craft (M34): the physics. Kept simple and stable: a fixed step (1/120 s), semi-implicit Euler, a few points
// per body.
//   Hull   one body on the water: x, z and heading free in the plane, heave, pitch and roll as springs toward the
//          water under its float points. Each float point feels the drag of the water moving past it (along and across
//          the hull), so a hull drifts with the current and spins where the current shears. The water's slope pushes
//          it downhill (the fall). Cells standing above its bottom (banks, rocks) push it back, with a bump.
//   Line   a tow line or an anchor's rode for the forces: slack until taut, then a spring with damping. The same
//          rule as the game's `Mooring::moor` (pbd-core/src/vehicle/mod.rs): k = min(40 m, 4000) N/m and a damping of
//          1.6 sqrt(k m), pulling only.
//   Rope   the same line drawn: a chain of points (position-based, never longer than its length) that sags, floats
//          on the water or sinks to the bed, and is dragged by the current.
//   Net    a cloth of points: a cork line that floats, a lead line that sinks, twine between, dragged by the water;
//          what the water does to it is passed to the boat it hangs from.
(function () {
"use strict";
const RC = window.RC, clamp = RC.clamp;
const G = 9.81, OUT = [0, 0];

RC.Hull = function (o) {
  Object.assign(this, {x: 0, z: 0, yaw: 0, vx: 0, vz: 0, w: 0, y: 0, vy: 0, pitch: 0, vp: 0, roll: 0, vr: 0, fx: 0, fz: 0, tq: 0,
    grounded: 0, bumped: 0, extra: 0});
  this.L = o.L; this.B = o.B; this.m = o.m; this.draft = o.draft; this.cA = o.cA; this.cS = o.cS;
  this.I = o.m * (o.L * o.L + o.B * o.B) / 12;
  this.floats = [];
  const nx = o.nx || 3, nz = o.nz || 2;
  for (let i = 0; i < nx; i++) for (let j = 0; j < nz; j++) this.floats.push([(-0.5 + (i + 0.5) / nx) * o.L * 0.9, nz === 1 ? 0 : (-0.5 + (j + 0.5) / nz) * o.B * 0.9]);
  const hl = o.L / 2, hb = o.B / 2;
  this.outline = o.outline || (o.pointed ? [[hl, 0], [hl * 0.45, -hb], [hl * 0.45, hb], [0, -hb], [0, hb], [-hl, -hb], [-hl, hb]]
    : [[hl, -hb], [hl, hb], [0, -hb], [0, hb], [-hl, -hb], [-hl, hb], [hl, 0], [-hl, 0]]);
  this.R = o.R || hb; this.half = Math.max(0, hl - this.R * (o.pointed ? 0.6 : 0.4));
};
const HP = RC.Hull.prototype;
HP.wx = function (lx, lz) { return this.x + Math.cos(this.yaw) * lx - Math.sin(this.yaw) * lz; };
HP.wz = function (lx, lz) { return this.z + Math.sin(this.yaw) * lx + Math.cos(this.yaw) * lz; };
HP.pv = function (px, pz, o) { o[0] = this.vx - this.w * (pz - this.z); o[1] = this.vz + this.w * (px - this.x); return o; };
HP.force = function (px, pz, fx, fz) { this.fx += fx; this.fz += fz; this.tq += (px - this.x) * fz - (pz - this.z) * fx; };
HP.thrust = function (lx, lz, along, side) {   // a push in the hull's own frame at a point on it
  const c = Math.cos(this.yaw), s = Math.sin(this.yaw);
  this.force(this.wx(lx, lz), this.wz(lx, lz), along * c - side * s, along * s + side * c);
};
HP.mass = function () { return this.m + this.extra; };
const PV = [0, 0];
// The water on the hull: drag toward the local flow at each float point, the surface's slope, the bed.
HP.water = function (river) {
  const c = Math.cos(this.yaw), s = Math.sin(this.yaw), n = this.floats.length, m = this.mass();
  const bottom = this.y - this.draft; this.grounded = 0;
  for (const [lx, lz] of this.floats) {
    const px = this.wx(lx, lz), pz = this.wz(lx, lz);
    river.flow(px, pz, OUT); this.pv(px, pz, PV);
    const rx = OUT[0] - PV[0], rz = OUT[1] - PV[1];
    const ra = rx * c + rz * s, rs = -rx * s + rz * c;
    const Fa = this.cA / n * ra * (Math.abs(ra) + 0.25), Fs = this.cS / n * rs * (Math.abs(rs) + 0.25);
    this.force(px, pz, Fa * c - Fs * s, Fa * s + Fs * c);
    if (river.top(px, pz) > bottom - 0.03) {   // on the bed: it scrapes
      this.grounded++;
      this.force(px, pz, -PV[0] * m / n * 2.5, -PV[1] * m / n * 2.5);
    }
  }
  const dl = (river.level(this.x + 0.4) - river.level(this.x - 0.4)) / 0.8;
  this.force(this.x, this.z, -m * G * clamp(dl, -0.7, 0.7) * 0.85, 0);
};
const SURF = [];
HP.integrate = function (h, river) {
  const m = this.mass(), I = this.I * m / this.m;
  this.vx += this.fx / m * h; this.vz += this.fz / m * h; this.w += this.tq / I * h;
  this.x += this.vx * h; this.z += this.vz * h; this.yaw += this.w * h;
  this.fx = this.fz = this.tq = 0;
  // heave, pitch and roll toward the water under the float points (a least-squares plane)
  let sum = 0, sx = 0, sxx = 0, sz = 0, szz = 0, bedMax = -99;
  for (const [lx, lz] of this.floats) {
    const px = this.wx(lx, lz), pz = this.wz(lx, lz), hs = river.surf(px, pz);
    sum += hs; sx += lx * hs; sxx += lx * lx; sz += lz * hs; szz += lz * lz; bedMax = Math.max(bedMax, river.top(px, pz));
  }
  const n = this.floats.length, mean = sum / n;
  let yT = mean - 0.1 * this.extra / Math.max(1, this.m) * this.draft;
  yT = Math.max(yT, bedMax + this.draft - 0.04);
  const pT = Math.atan(sxx > 0 ? (sx - mean * 0) / sxx : 0), rT = -Math.atan(szz > 0 ? sz / szz : 0);
  this.vy += (60 * (yT - this.y) - 10 * this.vy) * h; this.y += this.vy * h;
  this.vp += (45 * (clamp(pT, -0.5, 0.5) - this.pitch) - 9 * this.vp) * h; this.pitch += this.vp * h;
  this.vr += (45 * (clamp(rT, -0.4, 0.4) - this.roll) - 7 * this.vr) * h; this.roll += this.vr * h;
};
HP.settle = function (river) {   // put it on the water at rest
  this.y = river.surf(this.x, this.z); this.vy = this.vp = this.vr = this.pitch = this.roll = 0;
};
// A contact: push A (and B, or the world if B is null) apart along n (from B into A) at (px, pz).
RC.contact = function (A, B, px, pz, nx, nz, e, mu) {
  const rAx = px - A.x, rAz = pz - A.z, mA = A.mass(), IA = A.I * mA / A.m;
  let vx = A.vx - A.w * rAz, vz = A.vz + A.w * rAx, k, rnB = 0, rBx = 0, rBz = 0, mB = 0, IB = 0;
  if (B) { rBx = px - B.x; rBz = pz - B.z; mB = B.mass(); IB = B.I * mB / B.m; vx -= B.vx - B.w * rBz; vz -= B.vz + B.w * rBx; }
  const vn = vx * nx + vz * nz; if (vn >= 0) return 0;
  const rnA = rAx * nz - rAz * nx; rnB = rBx * nz - rBz * nx;
  k = 1 / mA + rnA * rnA / IA + (B ? 1 / mB + rnB * rnB / IB : 0);
  const j = -(1 + e) * vn / k;
  const tx = -nz, tz = nx, vt = vx * tx + vz * tz, rtA = rAx * tz - rAz * tx, rtB = rBx * tz - rBz * tx;
  const kt = 1 / mA + rtA * rtA / IA + (B ? 1 / mB + rtB * rtB / IB : 0);
  const jt = clamp(-vt / kt, -mu * j, mu * j);
  const Jx = nx * j + tx * jt, Jz = nz * j + tz * jt;
  A.vx += Jx / mA; A.vz += Jz / mA; A.w += (rAx * Jz - rAz * Jx) / IA;
  if (B) { B.vx -= Jx / mB; B.vz -= Jz / mB; B.w -= (rBx * Jz - rBz * Jx) / IB; }
  return j;
};
// Banks and rocks: each outline point inside a cell that stands above the hull's bottom is pushed out.
HP.terrain = function (river) {
  const bottom = this.y - this.draft * 0.6;
  let hit = 0;
  for (const [lx, lz] of this.outline) {
    const px = this.wx(lx, lz), pz = this.wz(lx, lz), r = river.solid(px, pz, bottom);
    if (!r) continue;
    const [nx, nz, d] = r;
    this.x += nx * d * 0.5; this.z += nz * d * 0.5;
    const j = RC.contact(this, null, px, pz, nx, nz, 0.25, 0.35);
    if (j > 0) { hit = Math.max(hit, j / this.mass()); this.vr += (lz > 0 ? 1 : -1) * Math.min(0.5, j / this.mass()) * 0.6; this.vp += 0.15 * Math.min(0.5, j / this.mass()); }
  }
  if (hit > 0.15) this.bumped = 0.6;
  return hit;
};
// Closest points between two segments in the plane (Ericson): into o = [s, t].
function segClosest(p1x, p1z, q1x, q1z, p2x, p2z, q2x, q2z, o) {
  const d1x = q1x - p1x, d1z = q1z - p1z, d2x = q2x - p2x, d2z = q2z - p2z, rx = p1x - p2x, rz = p1z - p2z;
  const a = d1x * d1x + d1z * d1z, e = d2x * d2x + d2z * d2z, f = d2x * rx + d2z * rz;
  let s = 0, t = 0;
  if (a <= 1e-9 && e <= 1e-9) { s = t = 0; }
  else if (a <= 1e-9) { s = 0; t = clamp(f / e, 0, 1); }
  else {
    const c = d1x * rx + d1z * rz;
    if (e <= 1e-9) { t = 0; s = clamp(-c / a, 0, 1); }
    else {
      const b = d1x * d2x + d1z * d2z, den = a * e - b * b;
      s = den !== 0 ? clamp((b * f - c * e) / den, 0, 1) : 0;
      t = (b * s + f) / e;
      if (t < 0) { t = 0; s = clamp(-c / a, 0, 1); } else if (t > 1) { t = 1; s = clamp((b - c) / a, 0, 1); }
    }
  }
  o[0] = s; o[1] = t; return o;
}
RC.segClosest = segClosest;
const ST = [0, 0];
HP.seg = function () {
  const c = Math.cos(this.yaw) * this.half, s = Math.sin(this.yaw) * this.half;
  return [this.x - c, this.z - s, this.x + c, this.z + s];
};
// Two hulls as capsules: pushed apart, with a bump.
RC.collide = function (A, B, e) {
  const a = A.seg(), b = B.seg();
  segClosest(a[0], a[1], a[2], a[3], b[0], b[1], b[2], b[3], ST);
  const ax = a[0] + (a[2] - a[0]) * ST[0], az = a[1] + (a[3] - a[1]) * ST[0];
  const bx = b[0] + (b[2] - b[0]) * ST[1], bz = b[1] + (b[3] - b[1]) * ST[1];
  let dx = ax - bx, dz = az - bz; const d = Math.hypot(dx, dz), R = A.R + B.R;
  if (d >= R) return 0;
  if (d < 1e-6) { dx = 1; dz = 0; } else { dx /= d; dz /= d; }
  const pen = R - d, wa = 1 / A.mass(), wb = 1 / B.mass(), ws = wa + wb;
  A.x += dx * pen * wa / ws * 0.8; A.z += dz * pen * wa / ws * 0.8; B.x -= dx * pen * wb / ws * 0.8; B.z -= dz * pen * wb / ws * 0.8;
  return RC.contact(A, B, (ax + bx) / 2, (az + bz) / 2, dx, dz, e === undefined ? 0.15 : e, 0.4);
};

// ---------------- Line: the force of a tow line or a rode ----------------
// A at local point la, B a hull at lb, or a fixed point (B null, at = [x, z]). Slack until taut.
RC.Line = function (A, la, B, lb, length) { Object.assign(this, {A, la, B, lb, length, tension: 0, at: null, on: true}); };
RC.Line.prototype.ends = function () {
  const A = this.A, B = this.B;
  const ax = A.wx(this.la[0], this.la[1]), az = A.wz(this.la[0], this.la[1]);
  const bx = B ? B.wx(this.lb[0], this.lb[1]) : this.at[0], bz = B ? B.wz(this.lb[0], this.lb[1]) : this.at[1];
  return [ax, az, bx, bz];
};
RC.Line.prototype.step = function () {
  this.tension = 0; if (!this.on) return 0;
  const A = this.A, B = this.B, [ax, az, bx, bz] = this.ends();
  let dx = bx - ax, dz = bz - az; const d = Math.hypot(dx, dz);
  if (d <= this.length || d < 1e-6) return 0;
  dx /= d; dz /= d;
  const mA = A.mass(), mS = B ? mA * B.mass() / (mA + B.mass()) : mA;
  const k = Math.min(40 * mS, 4000);
  A.pv(ax, az, PV); let vx = PV[0], vz = PV[1];
  if (B) { B.pv(bx, bz, PV); vx -= PV[0]; vz -= PV[1]; }
  const closing = vx * dx + vz * dz;
  const pull = Math.max(0, k * (d - this.length) - 1.6 * Math.sqrt(k * mS) * closing);
  A.force(ax, az, dx * pull, dz * pull);
  if (B) B.force(bx, bz, -dx * pull, -dz * pull);
  this.tension = pull;
  return pull;
};

// ---------------- Rope: the line drawn ----------------
// n points, position-based: gravity, the water (floats or sinks, dragged by the flow), the bed, a length it never
// exceeds. Ends pinned to a, b ([x, y, z]); b may be null (a free end).
RC.Rope = function (n, sink) {
  this.n = n; this.sink = !!sink; this.p = new Float32Array(n * 3); this.v = new Float32Array(n * 3); this.q = new Float32Array(n * 3);
};
RC.Rope.prototype.lay = function (a, b) {
  for (let i = 0; i < this.n; i++) { const t = i / (this.n - 1); for (let k = 0; k < 3; k++) { this.p[i * 3 + k] = a[k] + (b[k] - a[k]) * t; this.v[i * 3 + k] = 0; } }
};
RC.Rope.prototype.step = function (h, river, a, b, length) {
  const n = this.n, p = this.p, v = this.v, q = this.q, seg = length / (n - 1);
  for (let i = 0; i < n; i++) {
    const j = i * 3; let x = p[j], y = p[j + 1], z = p[j + 2];
    v[j + 1] -= G * h;
    const sy = river.surf(x, z);
    if (y < sy) {
      if (!this.sink) v[j + 1] += (sy - y) * 60 * h + G * h * 0.9;
      else v[j + 1] += G * h * 0.6;
      river.flow(x, z, OUT);
      const kd = Math.min(1, 4 * h);
      v[j] += (OUT[0] - v[j]) * kd; v[j + 2] += (OUT[1] - v[j + 2]) * kd; v[j + 1] *= 1 - Math.min(1, 6 * h);
    }
    q[j] = x + v[j] * h; q[j + 1] = y + v[j + 1] * h; q[j + 2] = z + v[j + 2] * h;
    const bed = river.top(q[j], q[j + 2]) + 0.05; if (q[j + 1] < bed) q[j + 1] = bed;
  }
  for (let it = 0; it < 12; it++) {
    q[0] = a[0]; q[1] = a[1]; q[2] = a[2];
    if (b) { const j = (n - 1) * 3; q[j] = b[0]; q[j + 1] = b[1]; q[j + 2] = b[2]; }
    for (let i = 0; i < n - 1; i++) {
      const j = i * 3, k = j + 3;
      let dx = q[k] - q[j], dy = q[k + 1] - q[j + 1], dz = q[k + 2] - q[j + 2];
      const d = Math.hypot(dx, dy, dz); if (d <= seg || d < 1e-6) continue;
      const c = (d - seg) / d;
      const wj = i === 0 ? 0 : 1, wk = (i + 1 === n - 1 && b) ? 0 : 1, ws = wj + wk; if (!ws) continue;
      dx *= c; dy *= c; dz *= c;
      q[j] += dx * wj / ws; q[j + 1] += dy * wj / ws; q[j + 2] += dz * wj / ws;
      q[k] -= dx * wk / ws; q[k + 1] -= dy * wk / ws; q[k + 2] -= dz * wk / ws;
    }
  }
  for (let i = 0; i < n * 3; i++) { v[i] = (q[i] - p[i]) / h; p[i] = q[i]; }
};

// ---------------- Net: a cloth of points ----------------
// cols x rows points; row 0 the cork line (floats), the last row the lead line (sinks). The two end columns are
// made fast to the boat's spars. Each point is dragged toward the flow; the sum of that drag is what the net pulls
// on the boat with. `fish` caught add their weight to the bag.
RC.Net = function (cols, rows, gap) {
  Object.assign(this, {cols, rows, gap, haul: 0, fish: 0, pull: [0, 0]});
  this.N = cols * rows; this.p = new Float32Array(this.N * 3); this.v = new Float32Array(this.N * 3); this.q = new Float32Array(this.N * 3);
};
const NP = RC.Net.prototype;
NP.idx = function (c, r) { return (r * this.cols + c) * 3; };
NP.lay = function (L, R, back) {   // L, R: the spar ends [x, y, z]; back: [dx, dz] the way the net streams
  for (let r = 0; r < this.rows; r++) for (let c = 0; c < this.cols; c++) {
    const t = c / (this.cols - 1), j = this.idx(c, r), bag = Math.sin(Math.PI * t) * 1.0;
    this.p[j] = L[0] + (R[0] - L[0]) * t + back[0] * bag; this.p[j + 1] = L[1] - r * this.gap * 0.5; this.p[j + 2] = L[2] + (R[2] - L[2]) * t + back[1] * bag;
    this.v[j] = this.v[j + 1] = this.v[j + 2] = 0;
  }
};
NP.step = function (h, river, L, R) {
  const p = this.p, v = this.v, q = this.q, C = this.cols, Rw = this.rows;
  const gap = this.gap * (1 - 0.85 * this.haul);
  let fx = 0, fz = 0;
  const mPt = 1.2, kd = 4.0;
  for (let r = 0; r < Rw; r++) for (let c = 0; c < C; c++) {
    const j = this.idx(c, r); const x = p[j], y = p[j + 1], z = p[j + 2];
    v[j + 1] -= G * h;
    const sy = river.surf(x, z);
    if (y < sy) {
      if (r === 0) v[j + 1] += (sy - y) * 80 * h + G * h * 1.05;    // corks
      else if (r === Rw - 1) v[j + 1] += G * h * 0.55;               // lead line: sinks slowly
      else v[j + 1] += G * h * 0.9;
      river.flow(x, z, OUT);
      let k = kd * (1 + (r === Rw - 1 && Math.abs(c - (C - 1) / 2) < 2 ? this.fish * 0.6 : 0));
      const ax = (OUT[0] - v[j]) * k, az = (OUT[1] - v[j + 2]) * k;
      v[j] += ax * Math.min(h, 0.2 / k); v[j + 2] += az * Math.min(h, 0.2 / k); v[j + 1] *= 1 - Math.min(1, 5 * h);
      fx += ax * mPt; fz += az * mPt;
    }
    q[j] = x + v[j] * h; q[j + 1] = y + v[j + 1] * h; q[j + 2] = z + v[j + 2] * h;
    const bed = river.top(q[j], q[j + 2]) + 0.05; if (q[j + 1] < bed) q[j + 1] = bed;
  }
  const pin = () => {
    for (let r = 0; r < Rw; r++) {
      const a = this.idx(0, r), b = this.idx(C - 1, r), dy = r * gap;
      q[a] = L[0]; q[a + 1] = L[1] - dy; q[a + 2] = L[2]; q[b] = R[0]; q[b + 1] = R[1] - dy; q[b + 2] = R[2];
    }
  };
  const link = (j, k, len, eq) => {
    let dx = q[k] - q[j], dy = q[k + 1] - q[j + 1], dz = q[k + 2] - q[j + 2]; const d = Math.hypot(dx, dy, dz);
    if (d < 1e-6 || (!eq && d <= len)) return;
    const cc = (d - len) / d * 0.5; dx *= cc; dy *= cc; dz *= cc;
    q[j] += dx; q[j + 1] += dy; q[j + 2] += dz; q[k] -= dx; q[k + 1] -= dy; q[k + 2] -= dz;
  };
  for (let it = 0; it < 8; it++) {
    pin();
    for (let r = 0; r < Rw; r++) for (let c = 0; c < C - 1; c++) link(this.idx(c, r), this.idx(c + 1, r), gap, false);
    for (let r = 0; r < Rw - 1; r++) for (let c = 0; c < C; c++) link(this.idx(c, r), this.idx(c, r + 1), gap * 0.75, true);
  }
  pin();
  for (let i = 0; i < this.N * 3; i++) { v[i] = (q[i] - p[i]) / h; p[i] = q[i]; }
  // what the water does to the net, it does to the boat
  this.pull[0] = fx * (1 + this.fish * 0.12); this.pull[1] = fz * (1 + this.fish * 0.12);
};
// The cork line as a closed polygon with the boat's end (for what the net has swept round).
NP.inside = function (x, z) {
  const C = this.cols; let inside = false;
  for (let c = 0, d = C - 1; c < C; d = c++) {
    const a = this.idx(c, 0), b = this.idx(d, 0);
    const xi = this.p[a], zi = this.p[a + 2], xj = this.p[b], zj = this.p[b + 2];
    if ((zi > z) !== (zj > z) && x < (xj - xi) * (z - zi) / (zj - zi) + xi) inside = !inside;
  }
  return inside;
};
// A fish swimming into the twine is gilled: any point below the corks within r of it.
NP.touches = function (x, z, r) {
  for (let i = this.cols * 3; i < this.N * 3; i += 3) { const dx = this.p[i] - x, dz = this.p[i + 2] - z; if (dx * dx + dz * dz < r * r) return true; }
  return false;
};
NP.area = function () {
  const C = this.cols; let A = 0;
  for (let c = 0, d = C - 1; c < C; d = c++) { const a = this.idx(c, 0), b = this.idx(d, 0); A += this.p[b] * this.p[a + 2] - this.p[a] * this.p[b + 2]; }
  return Math.abs(A) / 2;
};
// The net drawn: the twine as a two-sided textured grid (holes in the texture), corks along the top.
NP.mesh = function (U) {
  const C = this.cols, Rw = this.rows, idx = [], uv = new Float32Array(this.N * 2);
  for (let r = 0; r < Rw; r++) for (let c = 0; c < C; c++) { const i = r * C + c; uv[i * 2] = c * 0.9; uv[i * 2 + 1] = r * 0.9; }
  for (let r = 0; r < Rw - 1; r++) for (let c = 0; c < C - 1; c++) { const a = r * C + c, b = a + 1, d = a + C, e = d + 1; idx.push(a, d, b, b, d, e); }
  const g = new THREE.BufferGeometry();
  g.setAttribute("position", new THREE.BufferAttribute(new Float32Array(this.N * 3), 3));
  g.setAttribute("normal", new THREE.BufferAttribute(new Float32Array(this.N * 3).map((_, i) => i % 3 === 1 ? 1 : 0), 3));
  g.setAttribute("uv", new THREE.BufferAttribute(uv, 2)); RC.tiled(g, "net"); g.setIndex(idx);
  const m = new THREE.Mesh(g, RC.litMaterial(U, {side: THREE.DoubleSide})); m.frustumCulled = false;
  const corkGeo = RC.box(0.16, 0.1, 0.16, "logEnd"), corks = [];
  for (let c = 0; c < C; c++) { const k = new THREE.Mesh(corkGeo, RC.litMaterial(U, {tint: 0xe0b070})); m.add(k); corks.push(k); }
  m.userData.corks = corks;
  return m;
};
NP.draw = function (mesh) {
  const P = mesh.geometry.attributes.position; P.array.set(this.p); P.needsUpdate = true;
  mesh.userData.corks.forEach((k, c) => { const j = this.idx(c, 0); k.position.set(this.p[j], this.p[j + 1] + 0.03, this.p[j + 2]); });
};
})();
