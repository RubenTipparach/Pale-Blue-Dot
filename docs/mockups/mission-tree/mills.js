// Mission Tree mockup, MILLS tab (openspec/changes/early-power, design 12): a windmill on a rise and a
// water mill on a river, joined by a pole line with lamps. One 3D view, made the first time the tab
// opens and stopped when it closes (CLAUDE.md, tabs). Built of the settlement set's textures
// (mission-tree/tex, copied by tools/art/early_power_art.py, which also draws the sails, the wheel,
// the river and the wire), sampled nearest, no outlines. Lit at night by the line's lamps, the
// mill's windows and a lantern at the windmill's door, every light lighting what is near it wherever
// the camera is; the moon stays dim. Nothing moves until Turn is pressed; Sluice shuts the wheel and
// the line goes dark at once, because nothing stores power.
(function () {
  "use strict";
  const THREE_URL = "https://cdnjs.cloudflare.com/ajax/libs/three.js/r128/three.min.js";
  const TEX = "mission-tree/tex/";
  const root = document.getElementById("mills");
  let built = false, S = null;
  const st = { night: true, turn: false, sluice: true };

  root.innerHTML = `
    <div class="crewbar millbar">
      <div class="grp"><span class="lab">TIME</span>
        <button class="btn oak" id="mDay" aria-pressed="false">Day</button>
        <button class="btn oak" id="mNight" aria-pressed="true">Night</button></div>
      <div class="grp"><span class="lab">MILLS</span>
        <button class="btn oak" id="mTurn" aria-pressed="false">Turn</button>
        <button class="btn oak" id="mSluice" aria-pressed="false">Shut sluice</button></div>
      <div class="grp"><span class="stepno" id="mStat"></span></div>
    </div>
    <div class="millview"><canvas id="mCv" tabindex="0" aria-label="A windmill on a rise and a water mill on a river, joined by a pole line. Drag to turn, wheel or pinch to zoom."></canvas>
      <div class="hud" id="mHud"></div></div>`;
  const $ = (s) => document.getElementById(s);

  function load(cb) {
    if (window.THREE) return cb();
    const sc = document.createElement("script");
    sc.src = THREE_URL;
    sc.onload = cb;
    document.head.appendChild(sc);
  }

  // ---- the hex grid: cells 2.833 m flat to flat, layers 1 m (CLAUDE.md, the gold standard)
  const HS = 2.833 / Math.sqrt(3);
  const cellXZ = (q, r) => [HS * 1.5 * q, HS * Math.sqrt(3) * (r + q / 2)];
  const R = 12;
  const riverX = (z) => 13 + 3.2 * Math.sin(z / 9);
  const HQ = -6, HR = 1; // the rise's cell, axial
  const HILL = cellXZ(HQ, HR);

  function heights() {
    const cells = [];
    for (let q = -R; q <= R; q++) for (let r = Math.max(-R, -q - R); r <= Math.min(R, -q + R); r++) {
      const [x, z] = cellXZ(q, r);
      const d = Math.hypot(x - HILL[0], z - HILL[1]);
      // the rise: clean hex terraces, a layer every two rings out from the windmill's cell
      const hd = (Math.abs(q - HQ) + Math.abs(r - HR) + Math.abs(q - HQ + r - HR)) / 2;
      let h = 2 + Math.max(0, 3 - Math.floor((hd - 1) / 2));
      const dr = Math.abs(x - riverX(z));
      if (dr < 2.4) h = 0;
      cells.push({ q, r, x, z, h, d, river: dr < 2.4 });
    }
    // the windmill's cell stands 2 layers above every cell within 3 (early-power design 4)
    const top = cells.reduce((a, c) => (c.d < a.d ? c : a));
    const near = cells.filter((c) => c !== top && Math.hypot(c.x - top.x, c.z - top.z) < 3 * 2.833 + 0.1);
    top.h = Math.max(...near.map((c) => c.h)) + 2;
    return { cells, top };
  }

  function build() {
    const T = window.THREE;
    const cv = $("mCv");
    const ren = new T.WebGLRenderer({ canvas: cv, antialias: true });
    ren.setPixelRatio(Math.min(2, window.devicePixelRatio || 1));
    ren.outputEncoding = T.sRGBEncoding;
    ren.shadowMap.enabled = true;
    ren.shadowMap.type = T.PCFSoftShadowMap;
    const scene = new T.Scene();
    const cam = new T.PerspectiveCamera(45, 1, 0.2, 400);
    const loader = new T.TextureLoader();
    const texCache = {};
    const tex = (n, rep) => {
      const k = n + rep;
      if (texCache[k]) return texCache[k];
      const t = loader.load(TEX + n + ".png", () => draw());
      t.magFilter = T.NearestFilter; t.minFilter = T.NearestFilter; t.generateMipmaps = false;
      t.wrapS = t.wrapT = T.RepeatWrapping; t.encoding = T.sRGBEncoding;
      return (texCache[k] = t);
    };

    // Every lamp lights every surface near it, wherever the camera is: a loop over up to 16 lights
    // added to the Phong material's own lighting (as M39's fire light).
    const LN = 16;
    const U = { lampL: { value: Array.from({ length: LN }, () => new T.Vector4()) }, lampC: { value: Array.from({ length: LN }, () => new T.Vector3()) }, lampN: { value: 0 } };
    const lit = (m) => {
      m.onBeforeCompile = (sh) => {
        Object.assign(sh.uniforms, U);
        sh.fragmentShader = `uniform vec4 lampL[${LN}];\nuniform vec3 lampC[${LN}];\nuniform int lampN;\n` + sh.fragmentShader.replace(
          "#include <lights_fragment_end>",
          `#include <lights_fragment_end>
          for (int li = 0; li < ${LN}; li++) {
            if (li >= lampN) break;
            vec3 lv = lampL[li].xyz + vViewPosition;
            float d = length(lv);
            float att = lampL[li].w / (1.0 + d * d * 0.16) * clamp(1.0 - d / 18.0, 0.0, 1.0);
            float nl = max(dot(normal, lv / max(d, 0.001)), 0.0) * 0.85 + 0.15;
            reflectedLight.directDiffuse += BRDF_Diffuse_Lambert(material.diffuseColor) * lampC[li] * att * nl;
          }`);
      };
      return m;
    };
    const mat = (n, rep, o) => lit(new T.MeshPhongMaterial(Object.assign({ map: tex(n, rep || 1), shininess: 0, specular: 0x000000 }, o || {})));

    // world UVs: a texture laid at its manifest scale (repeat_m metres a tile)
    const worldUV = (g, rep, axis) => {
      const p = g.attributes.position, uv = g.attributes.uv;
      for (let i = 0; i < p.count; i++) {
        if (axis === "xz") uv.setXY(i, p.getX(i) / rep, p.getZ(i) / rep);
      }
      uv.needsUpdate = true;
      return g;
    };

    // ---- ground: hex columns, grass on top, dirt on the sides
    const { cells, top } = heights();
    const Ptop = [], UVtop = [], Pside = [], UVside = [], Nside = [];
    const corner = (x, z, i) => [x + HS * Math.cos((Math.PI / 3) * i), z + HS * Math.sin((Math.PI / 3) * i)];
    const byQR = new Map(cells.map((c) => [c.q + "," + c.r, c]));
    const NB = [[1, 0], [1, -1], [0, -1], [-1, 0], [-1, 1], [0, 1]].map(([dq, dr]) => { const [x, z] = cellXZ(dq, dr); return { dq, dr, x, z }; });
    // the neighbour across an edge: the axial step whose direction is the edge's midpoint's
    const across = (c, mx, mz) => { const o = NB.reduce((a, n) => (n.x * mx + n.z * mz > a.x * mx + a.z * mz ? n : a)); return byQR.get(c.q + o.dq + "," + (c.r + o.dr)); };
    for (const c of cells) {
      const y = c.h;
      for (let i = 0; i < 6; i++) {
        const a = corner(c.x, c.z, i), b = corner(c.x, c.z, i + 1);
        Ptop.push(c.x, y, c.z, b[0], y, b[1], a[0], y, a[1]);
        UVtop.push(c.x / 2, c.z / 2, b[0] / 2, b[1] / 2, a[0] / 2, a[1] / 2);
      }
      // sides down to the lower neighbour (or -3 at the rim)
      for (let i = 0; i < 6; i++) {
        const a = corner(c.x, c.z, i), b = corner(c.x, c.z, i + 1);
        const mx = (a[0] + b[0]) / 2 - c.x, mz = (a[1] + b[1]) / 2 - c.z;
        const nb = across(c, mx, mz);
        const lo = nb ? nb.h : -3;
        if (lo >= y) continue;
        const nx = mx, nz = mz, nl = Math.hypot(nx, nz);
        const ed = Math.hypot(b[0] - a[0], b[1] - a[1]) / 2;
        Pside.push(a[0], y, a[1], b[0], y, b[1], b[0], lo, b[1], a[0], y, a[1], b[0], lo, b[1], a[0], lo, a[1]);
        UVside.push(0, y / 2, ed, y / 2, ed, lo / 2, 0, y / 2, ed, lo / 2, 0, lo / 2);
        for (let k = 0; k < 6; k++) Nside.push(nx / nl, 0, nz / nl);
      }
    }
    const gTop = new T.BufferGeometry();
    gTop.setAttribute("position", new T.Float32BufferAttribute(Ptop, 3));
    gTop.setAttribute("uv", new T.Float32BufferAttribute(UVtop, 2));
    gTop.computeVertexNormals();
    const gSide = new T.BufferGeometry();
    gSide.setAttribute("position", new T.Float32BufferAttribute(Pside, 3));
    gSide.setAttribute("uv", new T.Float32BufferAttribute(UVside, 2));
    gSide.setAttribute("normal", new T.Float32BufferAttribute(Nside, 3));
    const grassM = mat("grass"), dirtM = mat("dirt", 1, { side: T.DoubleSide });
    const groundTop = new T.Mesh(gTop, grassM); groundTop.receiveShadow = true; groundTop.castShadow = true; scene.add(groundTop);
    const groundSide = new T.Mesh(gSide, dirtM); groundSide.receiveShadow = true; groundSide.castShadow = true; scene.add(groundSide);

    // river bed: the river cells' tops are dirt, not grass, under the water
    const bed = [];
    for (const c of cells) if (c.river) for (let i = 0; i < 6; i++) {
      const a = corner(c.x, c.z, i), b = corner(c.x, c.z, i + 1);
      bed.push(c.x, 0.01, c.z, b[0], 0.01, b[1], a[0], 0.01, a[1]);
    }
    const gBed = new T.BufferGeometry(); gBed.setAttribute("position", new T.Float32BufferAttribute(bed, 3));
    gBed.setAttribute("uv", new T.Float32BufferAttribute(bed.length / 3 * 2, 2)); gBed.computeVertexNormals();
    worldUV(gBed, 2, "xz");
    scene.add(new T.Mesh(gBed, mat("stone")));

    // the river: a ribbon down the channel, its texture moving only while Turn is on
    const water = tex("riverwater", 4);
    const wv = [];
    for (const c of cells) if (c.river) for (let i = 0; i < 6; i++) {
      const a = corner(c.x, c.z, i), b = corner(c.x, c.z, i + 1);
      wv.push(c.x, 0.62, c.z, b[0], 0.62, b[1], a[0], 0.62, a[1]);
    }
    const gW = new T.BufferGeometry();
    gW.setAttribute("position", new T.Float32BufferAttribute(wv, 3));
    gW.setAttribute("uv", new T.Float32BufferAttribute(wv.length / 3 * 2, 2)); gW.computeVertexNormals();
    worldUV(gW, 4, "xz");
    const waterM = lit(new T.MeshPhongMaterial({ map: water, transparent: true, opacity: 0.86, shininess: 30, specular: 0x223344 }));
    scene.add(new T.Mesh(gW, waterM));

    const add = (g, m, x, y, z, o) => {
      const me = new T.Mesh(g, m); me.position.set(x, y, z);
      if (o && o.ry) me.rotation.y = o.ry;
      if (o && o.rx) me.rotation.x = o.rx;
      if (o && o.rz) me.rotation.z = o.rz;
      me.castShadow = true; me.receiveShadow = true;
      (o && o.parent ? o.parent : scene).add(me);
      return me;
    };
    // a box whose faces carry the texture at its scale (metres per tile)
    const boxUV = (w, h, d, rep) => {
      const g = new T.BoxGeometry(w, h, d);
      const uv = g.attributes.uv, n = g.attributes.normal, p = g.attributes.position;
      for (let i = 0; i < uv.count; i++) {
        const ax = Math.abs(n.getX(i)) > 0.5 ? "x" : Math.abs(n.getY(i)) > 0.5 ? "y" : "z";
        const X = p.getX(i), Y = p.getY(i), Z = p.getZ(i);
        if (ax === "x") uv.setXY(i, Z / rep, Y / rep); else if (ax === "y") uv.setXY(i, X / rep, Z / rep); else uv.setXY(i, X / rep, Y / rep);
      }
      return g;
    };
    // a turned shape (smooth normals round it): pts = [[radius, y], ...], u round, v up at rep metres a tile
    const lathe = (pts, rep, seg) => {
      const g = new T.LatheGeometry(pts.map((p) => new T.Vector2(p[0], p[1])), seg || 20);
      const uv = g.attributes.uv, p = g.attributes.position;
      const circ = 2 * Math.PI * Math.max(...pts.map((q) => q[0]));
      for (let i = 0; i < uv.count; i++) uv.setXY(i, (uv.getX(i) * circ) / rep, p.getY(i) / rep);
      return g;
    };

    // ---- the windmill on its rise (early-power design 4): a fieldstone foot, a tapered plank tower,
    // a thatched cap, four sails on a timber stock, facing the river
    const wm = new T.Group(); wm.position.set(top.x, top.h, top.z); scene.add(wm);
    wm.rotation.y = 0.9; // the sails face the valley and the river
    add(lathe([[0, -0.6], [1.42, -0.6], [1.42, 0.5], [1.36, 1.0], [1.25, 1.25], [0, 1.25]], 2), mat("fieldstone", 2), 0, 0, 0, { parent: wm });
    add(lathe([[1.24, 0], [1.2, 0.6], [1.05, 4.0], [0.92, 6.4], [0, 6.42]], 2), mat("plank", 2), 0, 1.2, 0, { parent: wm });
    add(lathe([[1.0, 0], [1.16, 0.1], [1.14, 0.55], [1.0, 1.1], [0.76, 1.55], [0.4, 1.82], [0, 1.9]], 2), mat("thatch", 2, { side: T.DoubleSide }), 0, 7.55, 0, { parent: wm });
    // the door at its foot: a timber frame let into the tower's curve, planks inside
    add(boxUV(0.8, 1.5, 0.2, 1), mat("timber", 1), 0, 1.95, 1.16, { parent: wm });
    const shaft = add(new T.CylinderGeometry(0.14, 0.18, 1.3, 12), mat("iron", 1), 0, 8.05, 0.95, { parent: wm, rx: Math.PI / 2 });
    shaft.castShadow = false;
    const sails = new T.Group(); sails.position.set(0, 8.05, 1.6); wm.add(sails);
    add(lathe([[0, -0.3], [0.32, -0.25], [0.36, 0], [0.32, 0.25], [0, 0.3]], 1, 14), mat("iron", 1), 0, 0, 0, { parent: sails, rx: Math.PI / 2 });
    const clothM = mat("sailcloth", 2, { side: T.DoubleSide });
    for (let i = 0; i < 4; i++) {
      const arm = new T.Group(); arm.rotation.z = (Math.PI / 2) * i + 0.35; sails.add(arm);
      add(boxUV(0.18, 4.7, 0.14, 1), mat("timber", 1), 0, 2.4, 0.05, { parent: arm });
      // the sail: a lattice of laths and linen, one board along the stock, narrowing at its root
      const sg = new T.BufferGeometry();
      const L0 = 0.9, L1 = 4.6, w0 = 0.6, w1 = 1.15;
      const P = [0.1, L0, 0, w0, L0, -0.03, w1, L1, -0.06, 0.1, L1, 0];
      sg.setAttribute("position", new T.Float32BufferAttribute(P, 3));
      sg.setAttribute("uv", new T.Float32BufferAttribute([0, L0 / 2, w0 / 2, L0 / 2, w1 / 2, L1 / 2, 0, L1 / 2], 2));
      sg.setIndex([0, 1, 2, 0, 2, 3]); sg.computeVertexNormals();
      add(sg, clothM, 0, 0, 0.12, { parent: arm });
    }

    // ---- the water mill on the bank (early-power design 5): a fieldstone ground storey, a plank
    // upper storey under a thatched gable, the wheel in the race, a sluice gate upstream
    const MZ = 0, MX = riverX(MZ) - 5.4, MY = 2;
    const mill = new T.Group(); mill.position.set(MX, MY, MZ); scene.add(mill);
    add(boxUV(5.0, 1.8, 4.0, 2), mat("fieldstone", 2), 0, 0.9, 0, { parent: mill });
    add(boxUV(4.8, 1.7, 3.8, 2), mat("plank", 2), 0, 2.65, 0, { parent: mill });
    // the gable roof, one piece with its overhang
    {
      const w = 2.55, d = 2.35, h = 1.9, y0 = 3.45;
      const P = [-w, y0, -d, w, y0, -d, w, y0 + h, 0, -w, y0 + h, 0, -w, y0, d, w, y0, d];
      const g = new T.BufferGeometry();
      const tri = [0, 3, 2, 0, 2, 1, 4, 5, 2, 4, 2, 3];
      const pos = [], uv = [];
      tri.forEach((i) => { pos.push(P[i * 3], P[i * 3 + 1], P[i * 3 + 2]); uv.push(P[i * 3] / 2, Math.hypot(P[i * 3 + 1] - y0, Math.abs(P[i * 3 + 2]) - d) / 2); });
      g.setAttribute("position", new T.Float32BufferAttribute(pos, 3)); g.setAttribute("uv", new T.Float32BufferAttribute(uv, 2)); g.computeVertexNormals();
      add(g, mat("thatch", 2, { side: T.DoubleSide }), 0, 0, 0, { parent: mill });
      const gab = new T.BufferGeometry();
      const gp = [-2.4, y0, -1.9, -2.4, y0, 1.9, -2.4, y0 + 1.55, 0, 2.4, y0, 1.9, 2.4, y0, -1.9, 2.4, y0 + 1.55, 0];
      gab.setAttribute("position", new T.Float32BufferAttribute(gp, 3));
      gab.setAttribute("uv", new T.Float32BufferAttribute([gp[2] / 2, gp[1] / 2, gp[5] / 2, gp[4] / 2, 0, gp[7] / 2, gp[11] / 2, gp[10] / 2, gp[14] / 2, gp[13] / 2, 0, gp[16] / 2], 2));
      gab.computeVertexNormals();
      add(gab, mat("plank", 2, { side: T.DoubleSide }), 0, 0, 0, { parent: mill });
    }
    // windows: lamplit panes flush in the wall at night, a door of timber
    const paneM = new T.MeshBasicMaterial({ color: 0xffc070 });
    const panes = [];
    for (const [x, z, ry] of [[-1.2, 1.91, 0], [1.2, 1.91, 0], [-2.41, 0, Math.PI / 2]]) {
      const pn = add(new T.PlaneGeometry(0.62, 0.62), paneM, x, 2.7, z, { parent: mill, ry: ry ? -ry : 0 });
      pn.castShadow = false; panes.push(pn);
    }
    add(boxUV(0.95, 1.6, 0.12, 1), mat("timber", 1), 0, 0.85, 2.0, { parent: mill });
    // the wheel, on the river side, half in the race
    const wheel = new T.Group(); wheel.position.set(MX + 3.75, 2.3, MZ); scene.add(wheel);
    wheel.rotation.y = Math.PI / 2;
    const wheelM = mat("millwheel", 1), ironM = mat("iron", 1);
    for (const s of [-0.5, 0.5]) {
      add(new T.TorusGeometry(1.95, 0.09, 8, 40), wheelM, 0, 0, s, { parent: wheel });
      add(new T.TorusGeometry(0.55, 0.07, 8, 20), ironM, 0, 0, s, { parent: wheel });
    }
    for (let i = 0; i < 12; i++) {
      const a = (i / 12) * Math.PI * 2;
      const pd = add(boxUV(0.08, 0.62, 1.15, 0.5), wheelM, Math.cos(a) * 1.9, Math.sin(a) * 1.9, 0, { parent: wheel, rz: a + Math.PI / 2 });
      pd.rotation.z = a;
      if (i % 2 === 0) for (const s of [-0.5, 0.5]) {
        const sp = add(boxUV(1.4, 0.09, 0.09, 0.5), wheelM, Math.cos(a) * 1.22, Math.sin(a) * 1.22, s, { parent: wheel });
        sp.rotation.z = a;
      }
    }
    add(new T.CylinderGeometry(0.14, 0.14, 2.2, 12), ironM, 0, 0, -0.4, { parent: wheel, rx: Math.PI / 2 });
    // the sluice upstream: two timber posts in the race and a plank gate that drops when shut
    const SZ = MZ - 6.5, SX = riverX(SZ) - 1.2;
    add(boxUV(0.25, 2.6, 0.25, 1), mat("timber", 1), SX - 1.4, 1.0, SZ);
    add(boxUV(0.25, 2.6, 0.25, 1), mat("timber", 1), SX + 1.4, 1.0, SZ);
    add(boxUV(3.1, 0.22, 0.25, 1), mat("timber", 1), SX, 2.3, SZ);
    const gate = add(boxUV(2.6, 1.1, 0.12, 2), mat("plank", 2), SX, 1.6, SZ);

    // ---- the line (design 6): wood poles 4 cells apart at most, copper wire, lamps by the path
    const poleM = mat("poles", 2), armM = mat("timber", 1), wireM = mat("copperwire", 0.25);
    const groundAt = (x, z) => {
      let best = null;
      for (const c of cells) { const d = (c.x - x) ** 2 + (c.z - z) ** 2; if (!best || d < best.d) best = { d, h: c.h }; }
      return best.h;
    };
    const poles = [];
    const LZ = 3.6;
    const xs = [MX - 3.2, MX - 3.2 - 2.833 * 3, MX - 3.2 - 2.833 * 6, MX - 3.2 - 2.833 * 9];
    for (const x of xs) {
      const y = groundAt(x, LZ), H = 5.2;
      add(lathe([[0.15, 0], [0.13, H * 0.5], [0.1, H], [0, H + 0.05]], 2, 10), poleM, x, y, LZ);
      add(boxUV(1.5, 0.13, 0.13, 1), armM, x, y + H - 0.45, LZ, { ry: Math.PI / 2 });
      for (const s of [-0.6, 0.6]) add(new T.CylinderGeometry(0.05, 0.07, 0.16, 8), ironM, x, y + H - 0.3, LZ + s);
      poles.push({ x, y: y + H - 0.22, by: y });
    }
    // the mill's own lead to the first pole
    const lead = { x: MX - 1.5, y: MY + 3.3, z: LZ - 1.75 };
    const wire = (a, b, sag) => {
      const m = new T.Vector3((a.x + b.x) / 2, (a.y + b.y) / 2 - sag, (a.z + b.z) / 2);
      const curve = new T.QuadraticBezierCurve3(new T.Vector3(a.x, a.y, a.z), m, new T.Vector3(b.x, b.y, b.z));
      const w = add(new T.TubeGeometry(curve, 16, 0.025, 5, false), wireM, 0, 0, 0);
      w.castShadow = false;
    };
    for (const s of [-0.6, 0.6]) {
      wire(lead, { x: poles[0].x, y: poles[0].y, z: LZ + s }, 0.25);
      for (let i = 1; i < poles.length; i++) wire({ x: poles[i - 1].x, y: poles[i - 1].y, z: LZ + s }, { x: poles[i].x, y: poles[i].y, z: LZ + s }, 0.55);
    }
    // a lamp on each pole: an iron bracket and hood, its glass lit while the line has power
    const bulbOn = new T.MeshBasicMaterial({ color: 0xfff0c0 }), bulbOff = mat("iron", 1, { color: 0x8a8f99 });
    const lamps = [];
    for (const p of poles) {
      const y = p.by + 3.4;
      add(boxUV(0.08, 0.08, 0.8, 0.5), ironM, p.x, y + 0.35, LZ - 0.45);
      add(lathe([[0, 0.28], [0.12, 0.26], [0.3, 0.05], [0.33, 0], [0.26, 0.02], [0.1, 0.2], [0, 0.22]], 1, 14), ironM, p.x, y - 0.05, LZ - 0.85);
      const b = add(new T.SphereGeometry(0.12, 12, 8), bulbOn, p.x, y - 0.02, LZ - 0.85);
      b.castShadow = false;
      lamps.push({ mesh: b, pos: new T.Vector3(p.x, y - 0.35, LZ - 0.95) });
    }
    // an oil lantern at the windmill's door: not on the line
    wm.updateMatrixWorld(true);
    const doorW = wm.localToWorld(new T.Vector3(0.6, 2.5, 1.35));
    const lantern = add(lathe([[0, 0.34], [0.1, 0.32], [0.16, 0.18], [0.14, 0.02], [0, 0]], 1, 10), new T.MeshBasicMaterial({ color: 0xffb060 }), doorW.x, doorW.y, doorW.z);
    lantern.castShadow = false;

    // ---- light: sun by day, a dim moon by night; the lamps by night
    const hemi = new T.HemisphereLight(0xbfd8ff, 0x5a4a30, 0.5); scene.add(hemi);
    const sun = new T.DirectionalLight(0xfff2dc, 1.0); sun.position.set(-30, 45, 25); sun.castShadow = true;
    sun.shadow.mapSize.set(2048, 2048); Object.assign(sun.shadow.camera, { left: -40, right: 40, top: 40, bottom: -40, near: 1, far: 140 });
    sun.shadow.bias = -0.0008; scene.add(sun);

    const lightList = () => {
      const L = [];
      const line = !st.sluice ? 0 : 1;
      if (st.night) {
        if (line) lamps.forEach((l) => L.push([l.pos, 3.2, [1.0, 0.86, 0.6]]));
        const mw = new T.Vector3(); panes.forEach((p) => { p.getWorldPosition(mw); L.push([mw.clone().add(new T.Vector3(0, -0.2, 0)), 1.2, [1.0, 0.62, 0.28]]); });
        L.push([new T.Vector3(doorW.x, doorW.y - 0.2, doorW.z + 0.3), 1.5, [1.0, 0.6, 0.25]]);
      }
      return L;
    };

    const orbit = { az: -0.5, pol: 1.1, r: 40, t: new T.Vector3(-1, 3, 0) };
    function place() {
      orbit.pol = Math.max(0.2, Math.min(1.45, orbit.pol));
      orbit.r = Math.max(8, Math.min(120, orbit.r));
      cam.position.set(orbit.t.x + orbit.r * Math.sin(orbit.pol) * Math.sin(orbit.az), orbit.t.y + orbit.r * Math.cos(orbit.pol), orbit.t.z + orbit.r * Math.sin(orbit.pol) * Math.cos(orbit.az));
      cam.lookAt(orbit.t);
    }
    let drag = null, pinch = null;
    cv.addEventListener("pointerdown", (e) => { drag = { x: e.clientX, y: e.clientY }; cv.setPointerCapture(e.pointerId); });
    cv.addEventListener("pointermove", (e) => {
      if (!drag || pinch) return;
      orbit.az -= (e.clientX - drag.x) * 0.006; orbit.pol -= (e.clientY - drag.y) * 0.006;
      drag = { x: e.clientX, y: e.clientY }; draw();
    });
    cv.addEventListener("pointerup", () => { drag = null; });
    cv.addEventListener("wheel", (e) => { e.preventDefault(); orbit.r *= Math.exp(e.deltaY * 0.001); draw(); }, { passive: false });
    cv.addEventListener("touchmove", (e) => {
      if (e.touches.length !== 2) return;
      const d = Math.hypot(e.touches[0].clientX - e.touches[1].clientX, e.touches[0].clientY - e.touches[1].clientY);
      if (pinch) { orbit.r *= pinch / d; draw(); }
      pinch = d;
    });
    cv.addEventListener("touchend", () => { pinch = null; });
    cv.addEventListener("keydown", (e) => {
      const k = { ArrowLeft: [0.08, 0, 1], ArrowRight: [-0.08, 0, 1], ArrowUp: [0, -0.06, 1], ArrowDown: [0, 0.06, 1], "+": [0, 0, 0.9], "-": [0, 0, 1.1] }[e.key];
      if (!k) return; orbit.az += k[0]; orbit.pol += k[1]; orbit.r *= k[2]; draw();
    });

    function size() {
      const r = cv.parentElement.getBoundingClientRect();
      ren.setSize(Math.max(1, r.width), Math.max(1, r.height), false);
      cam.aspect = Math.max(1, r.width) / Math.max(1, r.height); cam.updateProjectionMatrix();
    }
    function draw() {
      if (!S || root.offsetParent === null) return;
      size(); place();
      const night = st.night;
      scene.background = new T.Color(night ? 0x0a1424 : 0x9cc4e4);
      scene.fog = new T.Fog(night ? 0x0a1424 : 0x9cc4e4, 70, 170);
      hemi.intensity = night ? 0.07 : 0.55;
      hemi.color.set(night ? 0x8aa0d0 : 0xbfd8ff);
      sun.intensity = night ? 0.1 : 1.0;
      sun.color.set(night ? 0x9fb4e8 : 0xfff2dc);
      sun.position.set(night ? 20 : -38, 30, night ? -25 : 30);
      const on = st.sluice;
      lamps.forEach((l) => { l.mesh.material = on && night ? bulbOn : bulbOff; });
      paneM.color.set(night ? 0xffc070 : 0x0c1218);
      lantern.material.color.set(night ? 0xffb060 : 0x5a4030);
      const L = lightList();
      cam.updateMatrixWorld();
      for (let i = 0; i < LN; i++) {
        if (i < L.length) {
          const v = L[i][0].clone().applyMatrix4(cam.matrixWorldInverse);
          U.lampL.value[i].set(v.x, v.y, v.z, L[i][1]);
          U.lampC.value[i].set(...L[i][2]);
        }
      }
      U.lampN.value = L.length;
      gate.position.y = st.sluice ? 2.35 : 1.15;
      ren.render(scene, cam);
      const kw = st.sluice ? 6.0 : 0;
      const draw_ = st.sluice && night ? lamps.length * 0.1 : 0;
      $("mStat").textContent = `Water mill ${kw.toFixed(1)} kW · Line ${draw_.toFixed(1)} kW · Lamps ${st.sluice && night ? lamps.length : 0}/${lamps.length}`;
      const hud = $("mHud");
      hud.textContent = !st.sluice ? "Sluice shut · No power" : st.turn ? "Wind 9 m/s · Grinding" : "";
      hud.className = "hud";
    }
    // motion only while Turn is on (CLAUDE.md, nothing spins on its own)
    let raf = 0, last = 0;
    function tick(t) {
      raf = 0;
      if (!st.turn || root.offsetParent === null) return;
      const dt = last ? Math.min(0.05, (t - last) / 1000) : 0; last = t;
      sails.rotation.z -= dt * 1.1;
      if (st.sluice) { wheel.rotation.z -= dt * 0.9; water.offset.y -= dt * 0.35; }
      draw();
      raf = requestAnimationFrame(tick);
    }
    function run() { if (st.turn && !raf) { last = 0; raf = requestAnimationFrame(tick); } }
    window.addEventListener("resize", () => draw());
    return { draw, run, stop: () => { if (raf) cancelAnimationFrame(raf); raf = 0; }, orbit, scene, cam, groundSide, groundTop,
      still: (o) => { Object.assign(st, o.st || {}); if (o.az !== undefined) orbit.az = o.az; if (o.pol !== undefined) orbit.pol = o.pol; if (o.r !== undefined) orbit.r = o.r; if (o.t) orbit.t.set(...o.t); if (o.sails !== undefined) sails.rotation.z = o.sails; if (o.wheel !== undefined) wheel.rotation.z = o.wheel; sync(); draw(); } };
  }

  function sync() {
    $("mDay").setAttribute("aria-pressed", String(!st.night));
    $("mNight").setAttribute("aria-pressed", String(st.night));
    $("mTurn").setAttribute("aria-pressed", String(st.turn));
    $("mTurn").textContent = st.turn ? "Stop" : "Turn";
    $("mSluice").setAttribute("aria-pressed", String(!st.sluice));
    $("mSluice").textContent = st.sluice ? "Shut sluice" : "Open sluice";
  }
  const act = (f) => () => { f(); sync(); if (S) { S.draw(); S.run(); } };
  $("mDay").addEventListener("click", act(() => { st.night = false; }));
  $("mNight").addEventListener("click", act(() => { st.night = true; }));
  $("mTurn").addEventListener("click", act(() => { st.turn = !st.turn; if (!st.turn && S) S.stop(); }));
  $("mSluice").addEventListener("click", act(() => { st.sluice = !st.sluice; }));
  sync();

  // shown: build on first open; hidden: stop
  window.millsTab = {
    show() {
      if (!built) { built = true; load(() => { S = build(); window.millsTab.S = S; S.draw(); S.run(); }); }
      else if (S) { S.draw(); S.run(); }
    },
    hide() { if (S) S.stop(); },
    still(o) { if (S) S.still(o); },
    ready: () => !!S,
  };
})();
