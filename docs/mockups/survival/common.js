// Survival mockup: what the two 3D tabs share. Pixel textures drawn on canvases and sampled nearest,
// smooth-shaded geometry helpers, a drag-to-orbit camera that never moves on its own, and the fire
// light: every fire light lights every surface near it, wherever the camera is (CLAUDE.md, mockups
// lit at night), by a loop over up to 48 lights added to the Phong material's own lighting.
(function () {
  "use strict";
  const S = {};
  S.rng = function (seed) {
    let a = seed >>> 0;
    return function () {
      a = (a + 0x6d2b79f5) >>> 0;
      let t = a;
      t = Math.imul(t ^ (t >>> 15), t | 1);
      t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
      return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
    };
  };
  S.hash = function (x, y, s) {
    let h = (x * 374761393 + y * 668265263 + (s || 0) * 2246822519) | 0;
    h = Math.imul(h ^ (h >>> 13), 1274126177);
    return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
  };
  // A pixel texture: fn(x, y) -> [r, g, b] (0..255) at size n, sampled nearest and repeated.
  S.tex = function (n, fn, repeat) {
    const c = document.createElement("canvas");
    c.width = c.height = n;
    const g = c.getContext("2d"), img = g.createImageData(n, n);
    for (let y = 0; y < n; y++) for (let x = 0; x < n; x++) {
      const v = fn(x, y), i = (y * n + x) * 4;
      img.data[i] = v[0]; img.data[i + 1] = v[1]; img.data[i + 2] = v[2]; img.data[i + 3] = 255;
    }
    g.putImageData(img, 0, 0);
    const t = new THREE.CanvasTexture(c);
    t.magFilter = THREE.NearestFilter; t.minFilter = THREE.NearestFilter; t.generateMipmaps = false;
    t.wrapS = t.wrapT = THREE.RepeatWrapping;
    if (repeat) t.repeat.set(repeat[0], repeat[1]);
    t.encoding = THREE.sRGBEncoding;
    return t;
  };
  // Weld a non-indexed geometry's coincident vertices, so computeVertexNormals is smooth.
  S.weld = function (geo) {
    const pos = geo.attributes.position, uv = geo.attributes.uv;
    const map = new Map(), P = [], U = [], idx = [];
    for (let i = 0; i < pos.count; i++) {
      const k = [pos.getX(i), pos.getY(i), pos.getZ(i)].map((v) => Math.round(v * 1e4)).join(",");
      let j = map.get(k);
      if (j === undefined) { j = P.length / 3; map.set(k, j); P.push(pos.getX(i), pos.getY(i), pos.getZ(i)); if (uv) U.push(uv.getX(i), uv.getY(i)); }
      idx.push(j);
    }
    const g = new THREE.BufferGeometry();
    g.setAttribute("position", new THREE.Float32BufferAttribute(P, 3));
    if (uv) g.setAttribute("uv", new THREE.Float32BufferAttribute(U, 2));
    g.setIndex(idx);
    return g;
  };
  // Shared fire-light uniforms, positions in view space, filled each frame.
  S.fireUniforms = { fireL: { value: Array.from({ length: 48 }, () => new THREE.Vector4()) }, fireN: { value: 0 }, fireC: { value: new THREE.Color(1.0, 0.55, 0.22) } };
  S.firelit = function (mat) {
    mat.onBeforeCompile = (sh) => {
      Object.assign(sh.uniforms, S.fireUniforms);
      sh.fragmentShader = "uniform vec4 fireL[48];\nuniform vec3 fireC;\nuniform int fireN;\n" + sh.fragmentShader.replace(
        "#include <lights_fragment_end>",
        `#include <lights_fragment_end>
        for (int fi = 0; fi < 48; fi++) {
          if (fi >= fireN) break;
          vec3 lv = fireL[fi].xyz + vViewPosition;
          float d = length(lv);
          float att = fireL[fi].w / (1.0 + d * d * 0.06) * clamp(1.0 - d / 34.0, 0.0, 1.0);
          float nl = max(dot(normal, lv / max(d, 0.001)), 0.0) * 0.8 + 0.2;
          reflectedLight.directDiffuse += BRDF_Diffuse_Lambert(material.diffuseColor) * fireC * att * nl;
        }`);
    };
    return mat;
  };
  // Drag to orbit, wheel or pinch to zoom. Moves only when the viewer moves it.
  S.orbit = function (cam, el, st, onchange) {
    let drag = null, pinch = null;
    const apply = () => {
      st.pol = Math.max(0.15, Math.min(1.5, st.pol));
      st.r = Math.max(st.min || 2, Math.min(st.max || 300, st.r));
      cam.position.set(st.t.x + st.r * Math.sin(st.pol) * Math.sin(st.az), st.t.y + st.r * Math.cos(st.pol), st.t.z + st.r * Math.sin(st.pol) * Math.cos(st.az));
      cam.lookAt(st.t);
      if (onchange) onchange();
    };
    el.addEventListener("pointerdown", (e) => { drag = { x: e.clientX, y: e.clientY, moved: 0 }; el.setPointerCapture(e.pointerId); });
    el.addEventListener("pointermove", (e) => {
      if (!drag || pinch) return;
      const dx = e.clientX - drag.x, dy = e.clientY - drag.y;
      drag.moved += Math.abs(dx) + Math.abs(dy);
      drag.x = e.clientX; drag.y = e.clientY;
      st.az -= dx * 0.006; st.pol -= dy * 0.006; apply();
    });
    el.addEventListener("pointerup", (e) => { if (drag && drag.moved < 5 && st.click) st.click(e); drag = null; });
    el.addEventListener("wheel", (e) => { e.preventDefault(); st.r *= Math.exp(e.deltaY * 0.001); apply(); }, { passive: false });
    el.addEventListener("touchmove", (e) => {
      if (e.touches.length !== 2) return;
      const d = Math.hypot(e.touches[0].clientX - e.touches[1].clientX, e.touches[0].clientY - e.touches[1].clientY);
      if (pinch) { st.r *= pinch / d; apply(); }
      pinch = d;
    });
    el.addEventListener("touchend", () => { pinch = null; });
    apply();
    return apply;
  };
  window.SurvivalCommon = S;
})();
