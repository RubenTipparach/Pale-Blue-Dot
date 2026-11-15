// M32: rain splashes. Every droplet and every streak is placed by its vertex
// shader from a lane number and the time; JS sets uniforms each frame and
// uploads nothing else (owner: "the more gpu native rain we can have the better").
"use strict";
(() => {
const W = 2.833, RC = W / Math.sqrt(3), NR = 11, M = 2 * NR + 1;
const SLOTS = 48, DROPS = 8, STREAKS = 1700, MAXL = 4;
const FIELDS = 4; // atlas sheet slot, planet_terrain.rs TILESETS order
const TILE = { grass: [FIELDS, 0, 0], grassSide: [FIELDS, 1, 0], dirt: [FIELDS, 2, 0], path: [FIELDS, 3, 2], boards: [FIELDS, 1, 3], bark: [FIELDS, 2, 1] };

// ------------------------------------------------------------------ the patch
const center = (q, r) => [W * (q + r / 2), W * 0.8660254 * r];
const cells = new Map();
const key = (q, r) => q + "," + r;
for (let q = -NR; q <= NR; q++) for (let r = -NR; r <= NR; r++) {
  if (Math.abs(q + r) > NR) continue;
  const [x, z] = center(q, r);
  const s = x * 0.8 + z * 0.6;
  let h = s > 7 ? 1 : 0; if (s > 15) h = 2;
  if (x < -14 && z > 8) h = 1;
  let mat = "grass";
  const dPath = Math.abs(z - (0.32 * x - 2.0));
  if (dPath < 1.9 && h === 0) mat = "path";
  if (Math.hypot(x + 10, z - 4) < 4.6 && h === 0) mat = "dirt";
  if (Math.hypot(x - 9, z - 2) < 2.0 && h === 1) mat = "dirt";
  cells.set(key(q, r), { q, r, x, z, h, mat, roof: false });
}
const ROOF = [[-4, 3], [-5, 3], [-4, 2]];
for (const [q, r] of ROOF) { const c = cells.get(key(q, r)); c.roof = true; c.mat = "dirt"; }
const ROOF_UP = 3; // the roof cell is the layer [h+3, h+4]

// ------------------------------------------------------------------ geometry
const P = [], Nn = [], UV = [], T = [], C = [];  // C: class (0 grass top, 1 floor top, 2 side), sky
function tri(a, b, c, n, ua, ub, uc, tile, cls, sky) {
  for (const [p, u] of [[a, ua], [b, ub], [c, uc]]) { P.push(...p); Nn.push(...n); UV.push(...u); T.push(...tile); C.push(cls, sky); }
}
function quad(a, b, c, d, n, ua, ub, uc, ud, tile, cls, sky) { tri(a, b, c, n, ua, ub, uc, tile, cls, sky); tri(a, c, d, n, ua, uc, ud, tile, cls, sky); }
const corner = (x, z, i) => { const a = Math.PI / 180 * (30 + 60 * i); return [x + RC * Math.cos(a), z + RC * Math.sin(a)]; };
// a top's uv as planet_surface.wgsl makes it: (corner - centre) / (1.5 * tile) + 0.5, so a hex top spans about 1.3 tiles
const tuv = (x, z, px, pz) => [(px - x) / (1.5 * RC) + 0.5, (pz - z) / (1.5 * RC) + 0.5];
function hexTop(x, z, y, tile, cls, sky, down) {
  for (let i = 0; i < 6; i++) {
    const [ax, az] = corner(x, z, i), [bx, bz] = corner(x, z, i + 1);
    const uc = tuv(x, z, x, z), ua = tuv(x, z, ax, az), ub = tuv(x, z, bx, bz);
    if (down) tri([x, y, z], [ax, y, az], [bx, y, bz], [0, -1, 0], uc, ua, ub, tile, 2, sky);
    else tri([x, y, z], [bx, y, bz], [ax, y, az], [0, 1, 0], uc, ub, ua, tile, cls, sky);
  }
}
function hexSide(x, z, i, y0, y1, topTile, bodyTile, firstBand) {
  const [ax, az] = corner(x, z, i), [bx, bz] = corner(x, z, i + 1);
  const n = [(ax + bx) / 2 - x, 0, (az + bz) / 2 - z]; const l = Math.hypot(n[0], n[2]); n[0] /= l; n[2] /= l;
  for (let top = y1; top > y0 + 1e-4; top -= 1) {
    const bot = Math.max(y0, top - 1);
    const tile = (firstBand && top === y1) ? topTile : bodyTile;
    quad([ax, top, az], [bx, top, bz], [bx, bot, bz], [ax, bot, az], n, [0, 0], [0.999, 0], [0.999, top - bot], [0, top - bot], tile, 2, 1);
  }
}
for (const c of cells.values()) {
  const topTile = TILE[c.mat];
  hexTop(c.x, c.z, c.h, topTile, c.mat === "grass" ? 0 : 1, c.roof ? 0 : 1);
  for (let i = 0; i < 6; i++) {
    // the edge from corner i (30+60i deg) to corner i+1 faces 60+60i deg
    const d = [[0, 1], [-1, 1], [-1, 0], [0, -1], [1, -1], [1, 0]][i];
    const nb = cells.get(key(c.q + d[0], c.r + d[1]));
    const nh = nb ? nb.h : 0;
    if (nh < c.h) hexSide(c.x, c.z, i, nh, c.h, c.mat === "grass" ? TILE.grassSide : TILE.dirt, TILE.dirt, true);
    if (c.roof) {
      const nr = nb && nb.roof;
      if (!nr) hexSide(c.x, c.z, i, c.h + ROOF_UP, c.h + ROOF_UP + 1, TILE.boards, TILE.boards, true);
    }
  }
  if (c.roof) { hexTop(c.x, c.z, c.h + ROOF_UP + 1, TILE.boards, 1, 1); hexTop(c.x, c.z, c.h + ROOF_UP, TILE.boards, 1, 0, true); }
}
// posts under the roof: thin square timbers
function post(x, z, y0, y1, w) {
  const s = w / 2, pts = [[x - s, z - s], [x + s, z - s], [x + s, z + s], [x - s, z + s]];
  for (let i = 0; i < 4; i++) {
    const [ax, az] = pts[i], [bx, bz] = pts[(i + 1) % 4];
    const n = [bz - az, 0, -(bx - ax)]; const l = Math.hypot(n[0], n[2]); n[0] /= l; n[2] /= l;
    for (let top = y1; top > y0 + 1e-4; top -= 1) { const bot = Math.max(y0, top - 1); quad([ax, top, az], [bx, top, bz], [bx, bot, bz], [ax, bot, az], n, [0, 0], [w, 0], [w, top - bot], [0, top - bot], TILE.bark, 2, 1); }
  }
}
const roofCells = ROOF.map(([q, r]) => cells.get(key(q, r)));
for (const c of roofCells) { const [px, pz] = corner(c.x, c.z, 4); post(px * 0.9 + c.x * 0.1, pz * 0.9 + c.z * 0.1, c.h, c.h + ROOF_UP, 0.22); }
// the far ground: one grass plane, under the patch and out to the fog
{ const S = 260, u = S / (1.5 * RC); quad([-S, -0.02, -S], [-S, -0.02, S], [S, -0.02, S], [S, -0.02, -S], [0, 1, 0], [-u, -u], [-u, u], [u, u], [u, -u], TILE.grass, 0, 1); }

// ------------------------------------------------------------------ lamps
const LAMPS = [
  { x: 4.2, z: 0.6, y: 2.3 }, { x: -6.8, z: -4.2, y: 2.3 }, { x: -12.6, z: 7.6, y: 2.3 },
  { x: roofCells[0].x + 0.4, z: roofCells[0].z - 0.6, y: roofCells[0].h + ROOF_UP - 0.45, hung: true },
];
for (const L of LAMPS) if (!L.hung) post(L.x, L.z, 0, 2.1, 0.14);

// ------------------------------------------------------------------ renderer
const canvas = document.getElementById("view");
const renderer = new THREE.WebGLRenderer({ canvas, antialias: false, powerPreference: "high-performance", preserveDrawingBuffer: true });
renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 1.25));
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(60, 16 / 10, 0.05, 400);

const atlas = new THREE.TextureLoader().load("rain-splashes/atlas.png", () => { computeMeans(); });
atlas.magFilter = atlas.minFilter = THREE.NearestFilter; atlas.generateMipmaps = false;

// the column tops, for the streaks' landing: (q+NR, r+NR) -> highest up-face
const tops = new Float32Array(M * M * 4);
for (const c of cells.values()) { const i = ((c.r + NR) * M + (c.q + NR)) * 4; tops[i] = c.roof ? c.h + ROOF_UP + 1 : c.h; tops[i + 1] = 1; }
const topTex = new THREE.DataTexture(tops, M, M, THREE.RGBAFormat, THREE.FloatType);
topTex.minFilter = topTex.magFilter = THREE.NearestFilter; topTex.generateMipmaps = false; topTex.needsUpdate = true;

const U = {
  uTime: { value: 0 }, uRain: { value: 0.7 }, uWet: { value: 1 },
  uSunDir: { value: new THREE.Vector3(0.35, 0.85, 0.25).normalize() }, uSunCol: { value: new THREE.Color() },
  uSky: { value: new THREE.Color() }, uGround: { value: new THREE.Color() }, uFogCol: { value: new THREE.Color() }, uFogFar: { value: 120 },
  uSkyRefl: { value: new THREE.Color() }, uStreakLight: { value: 1 },
  uLampPos: { value: LAMPS.map(L => new THREE.Vector3(L.x, L.y, L.z)) }, uLampCol: { value: LAMPS.map(() => new THREE.Vector3()) }, uLampReach: { value: 8.5 },
  uAtlas: { value: atlas }, uTops: { value: topTex },
  uMeans: { value: [new THREE.Vector3(0.36, 0.42, 0.18), new THREE.Vector3(0.45, 0.36, 0.26), new THREE.Vector3(0.5, 0.38, 0.24)] },
  uDensity: { value: 1 }, uSize: { value: 1 }, uPx: { value: 600 }, uReach: { value: 20 }, uEye: { value: new THREE.Vector3() },
};

// shared GLSL: the game's hashes (rr_hash12/22), puddle noise, lighting
const COMMON = `
uniform float uTime, uRain, uWet, uFogFar, uLampReach;
uniform vec3 uSunDir, uSunCol, uSky, uGround, uFogCol, uSkyRefl;
uniform vec3 uLampPos[${MAXL}]; uniform vec3 uLampCol[${MAXL}];
float rr_hash12(vec2 p){ vec3 p3 = fract(vec3(p.x,p.y,p.x)*0.1031); p3 += dot(p3, p3.yzx+19.19); return fract((p3.x+p3.y)*p3.z); }
vec2 rr_hash22(vec2 p){ vec3 p3 = fract(vec3(p.x,p.y,p.x)*vec3(0.1031,0.1030,0.0973)); p3 += dot(p3, p3.yzx+19.19); return fract((p3.xx+p3.yz)*p3.zy); }
float puddle_noise(vec2 p){ vec2 i = floor(p), f = fract(p), w = f*f*(3.-2.*f);
  float a = mix(rr_hash12(i), rr_hash12(i+vec2(1.,0.)), w.x), b = mix(rr_hash12(i+vec2(0.,1.)), rr_hash12(i+vec2(1.,1.)), w.x); return mix(a,b,w.y); }
// planet_surface.wgsl's puddle: share 0.35 of a soaked floor, the mask's own quantile
float puddleMask(vec2 uv, float soak){
  float mask = puddle_noise(uv/0.4)*0.65 + puddle_noise(uv/(0.4*0.37)+vec2(17.,5.))*0.35;
  float share = clamp(0.35*soak, 1e-4, 0.9999);
  float edge = 0.5003 + 0.1587*log((1.-share)/share)/1.702;
  return smoothstep(edge-0.03, edge+0.03, mask);
}
vec3 lampLight(vec3 p, vec3 n, float wrap){
  vec3 l = vec3(0.);
  for (int i = 0; i < ${MAXL}; i++) { vec3 d = uLampPos[i]-p; float r = length(d); float f = clamp(1.-r/uLampReach,0.,1.); f *= f;
    l += uLampCol[i]*f*mix(max(dot(n, d/max(r,1e-3)),0.), 1., wrap); }
  return l;
}
vec3 fogged(vec3 c, vec3 p){ float f = smoothstep(uFogFar*0.2, uFogFar, length(p-cameraPosition)); return mix(c, uFogCol, f); }
`;

// ---- the ground: atlas, the wet block, puddles, rings, mirror, lamp highlights
const groundMat = new THREE.ShaderMaterial({
  uniforms: U, extensions: { derivatives: true }, side: THREE.DoubleSide,
  vertexShader: `
    attribute vec3 aTile; attribute vec2 aCls;
    varying vec3 vW; varying vec3 vN; varying vec2 vUv; varying vec3 vTile; varying vec2 vCls;
    void main(){ vUv = uv; vTile = aTile; vCls = aCls; vec4 w = modelMatrix*vec4(position,1.); vW = w.xyz; vN = normal; gl_Position = projectionMatrix*viewMatrix*w; }`,
  fragmentShader: COMMON + `
    uniform sampler2D uAtlas;
    varying vec3 vW; varying vec3 vN; varying vec2 vUv; varying vec3 vTile; varying vec2 vCls;
    vec3 atlasTile(vec2 uv, vec3 t0){ vec3 t = floor(t0+0.5); vec2 px = (floor(fract(uv)*32.)+0.5)/32.;
      vec2 sheet = vec2(mod(t.x,4.), floor(t.x/4.))*0.25; vec2 s = sheet+(t.yz+px)/16.; return texture2D(uAtlas, vec2(s.x, 1.-s.y)).rgb; }
    // rain_ripple_layer / rain_rings, as planet_surface.wgsl has them
    vec2 ringLayer(vec2 uv, float t, float keep, float salt){
      vec2 p0 = floor(uv); vec2 circles = vec2(0.);
      for (int j=-1;j<=1;j++) for (int i=-1;i<=1;i++){
        vec2 pi = p0+vec2(float(i),float(j)); vec2 p = pi+rr_hash22(pi+salt);
        float ph = 0.3*t+rr_hash12(pi+salt); float tt = fract(ph);
        if (rr_hash12(pi*1.37+floor(ph)*7.13+salt) >= keep) continue;
        vec2 v = p-uv; float len = length(v)+1e-6; float d = len-2.0*tt; float h = 0.012;
        float d1 = d-h, d2 = d+h;
        float q1 = sin(31.0*d1)*smoothstep(-0.6,-0.3,d1)*smoothstep(0.0,-0.3,d1);
        float q2 = sin(31.0*d2)*smoothstep(-0.6,-0.3,d2)*smoothstep(0.0,-0.3,d2);
        circles += 0.5*(v/len)*((q2-q1)/(2.0*h)*(1.0-tt)*(1.0-tt));
      }
      return circles/9.0;
    }
    vec2 rings(vec2 uv, float t, float rain){ vec2 g = ringLayer(uv,t,clamp(rain*2.,0.,1.),0.); if (rain > 0.5) g += ringLayer(uv+vec2(0.5,0.37),t*1.07,rain*2.-1.,41.); return g; }
    float ggx(vec3 n, vec3 v, vec3 l, float a){ float nl = max(dot(n,l),0.); if (nl<=0.) return 0.; vec3 h = normalize(v+l); float nv = max(dot(n,v),1e-3);
      float k = a*0.5; float vis = 0.25/((nl*(1.-k)+k)*(nv*(1.-k)+k)); float f = 0.02+0.98*pow(1.-max(dot(v,h),0.),5.);
      float a2 = a*a; float nh = max(dot(n,h),0.); float d = nh*nh*(a2-1.)+1.; return a2/(3.14159*d*d)*vis*f*nl; }
    void main(){
      vec3 color = atlasTile(vUv, vTile);
      vec3 n = normalize(vN); float sky = vCls.y; float cls = vCls.x;
      vec3 V = normalize(cameraPosition-vW); float dist = length(cameraPosition-vW);
      float wet_amt = uWet*sky;
      float face = n.y; float top_w = smoothstep(0.35,0.85,face);
      float side_w = (1.-smoothstep(0.1,0.5,face))*smoothstep(-0.5,-0.1,face);
      bool floor_like = cls > 0.5 && cls < 1.5;
      float flat_ = smoothstep(0.90,0.97,face);
      float puddle = floor_like ? flat_*puddleMask(vW.xz, uWet) : 0.;
      float rw = top_w*max(puddle, floor_like ? 1. : 0.5);
      float near_detail = 1.-smoothstep(16.,20.,dist);
      vec2 pg = vec2(0.);
      if (rw > 0.001 && near_detail > 0.001 && wet_amt > 0.001) {
        vec2 uv = vW.xz;
        pg += vec2(cos(uv.x*7.+uTime*1.4)+0.7*cos((uv.x+uv.y)*4.2-uTime*1.12), cos(uv.y*7.-uTime*1.26)+0.7*cos((uv.x-uv.y)*4.2+uTime*0.98))*(0.6*puddle);
        pg += rings(uv*5., uTime, uRain)*(2.0*rw);
        float foot = length(fwidth(vW.xz));
        pg /= 1.+pow(foot*15.,2.);
        pg *= near_detail;
      }
      vec3 wet_n = normalize(n+vec3(pg.x,0.,pg.y));
      float wet = wet_amt*max(top_w,side_w);
      color *= mix(1., 0.72*0.85, wet)*mix(1., 0.7, puddle*wet_amt);
      float grey = dot(color, vec3(0.2126,0.7152,0.0722));
      color = max(mix(vec3(grey), color, 1.+0.25*wet), vec3(0.));
      vec3 light = mix(uGround, uSky, n.y*0.5+0.5)*mix(0.35,1.,sky) + uSunCol*max(dot(n,uSunDir),0.)*mix(0.3,1.,sky) + lampLight(vW+n*0.05, n, 0.3);
      vec3 col = color*light;
      // the sky, mirrored, and the lamps' highlights, at the wet or puddle roughness
      float rough0 = mix(0.18, 0.04, puddle); float turn = dot(pg,pg)*0.4; float rough = clamp(sqrt(rough0*rough0+2.*turn), rough0, 0.6); float a = rough*rough;
      float cosv = clamp(dot(wet_n,V),0.,1.); float fres = 0.02+0.98*pow(1.-cosv,5.);
      vec3 r = reflect(-V, wet_n); vec3 env = uSkyRefl*(r.y < 0. ? 0.35 : 1.)*mix(0.4,1.,sky);
      float fr = fres*(1.-rough)*mix(0.45,1.,puddle)*wet;
      col = mix(col, env, clamp(fr,0.,1.));
      vec3 spec = vec3(0.);
      for (int i = 0; i < ${MAXL}; i++) { vec3 d = uLampPos[i]-vW; float rr = length(d); float f = clamp(1.-rr/(uLampReach*1.6),0.,1.); spec += uLampCol[i]*f*f*ggx(wet_n, V, d/rr, a); }
      col += spec*wet*mix(0.5,1.,puddle);
      gl_FragColor = vec4(fogged(col, vW), 1.);
    }`,
});
{
  const g = new THREE.BufferGeometry();
  g.setAttribute("position", new THREE.Float32BufferAttribute(P, 3));
  g.setAttribute("normal", new THREE.Float32BufferAttribute(Nn, 3));
  g.setAttribute("uv", new THREE.Float32BufferAttribute(UV, 2));
  g.setAttribute("aTile", new THREE.Float32BufferAttribute(T, 3));
  g.setAttribute("aCls", new THREE.Float32BufferAttribute(C, 2));
  scene.add(new THREE.Mesh(g, groundMat));
}
// lanterns: glowing boxes, the lamps' own light
const lanternMat = new THREE.MeshBasicMaterial({ color: 0x40382c });
for (const L of LAMPS) { const m = new THREE.Mesh(new THREE.BoxGeometry(0.26, 0.34, 0.26), lanternMat); m.position.set(L.x, L.y, L.z); scene.add(m); }

// ---- the splashes: one point per droplet, its whole life computed here
const colList = [];
for (const c of cells.values()) {
  // the column's topmost up-face, gated by its sky light: under the roof it is the roof's top
  const top = c.roof ? c.h + ROOF_UP + 1 : c.h, sky = 1, cls = c.roof ? 2 : (c.mat === "grass" ? 0 : 1);
  if (sky > 0) colList.push([c.x, c.z, top, cls]);
}
const nLanes = colList.length * SLOTS * DROPS;
const aCol = new Float32Array(nLanes * 4), aLane = new Float32Array(nLanes * 2), dummy = new Float32Array(nLanes * 3);
{ let i = 0; colList.forEach((c, ci) => { for (let s = 0; s < SLOTS; s++) for (let k = 0; k < DROPS; k++, i++) { aCol.set(c, i * 4); aLane[i * 2] = ci * SLOTS + s; aLane[i * 2 + 1] = k; } }); }
const splashMat = new THREE.ShaderMaterial({
  uniforms: U, transparent: true, depthWrite: false,
  vertexShader: COMMON + `
    uniform float uDensity, uSize, uPx, uReach; uniform vec3 uMeans[3]; uniform vec3 uEye;
    attribute vec4 aCol; attribute vec2 aLane;
    varying vec4 vC;
    float h1(vec2 p){ return rr_hash12(p); }
    void main(){
      float slot = aLane.x, k = aLane.y;
      float period = 0.5/uDensity;
      float ph = uTime/period + h1(vec2(slot*0.613, 7.17));
      float cyc = floor(ph), age = fract(ph)*period;
      vec2 sd = vec2(slot, mod(cyc, 4096.)*1.618);
      vec2 r2 = rr_hash22(sd*0.731+vec2(3.3,9.1));
      float ang = r2.x*6.28318, rad = sqrt(r2.y)*${(W / 2 * 0.97).toFixed(4)};
      vec2 xz = aCol.xy + vec2(cos(ang), sin(ang))*rad;
      float top = aCol.z, cls = aCol.w;
      float soak = uWet;
      float pud = cls > 0.5 ? puddleMask(xz, soak) : 0.;
      float kind = cls < 0.5 ? 0. : (pud > 0.5 ? 2. : 1.);           // 0 grass spray, 1 wet ground, 2 crown
      float keepCls = kind > 1.5 ? 1.0 : (kind > 0.5 ? 0.8 : 0.5);
      float fire = step(h1(sd*vec2(1.37,0.71)+vec2(17.,3.)), clamp(uRain,0.,1.)*keepCls);
      float nDrops = kind > 1.5 ? 8. : (kind > 0.5 ? 5. : 4.);
      float hk = h1(sd+vec2(k*3.1, 11.)), hk2 = h1(sd+vec2(k*5.7, 29.));
      float sz = uSize, wetS = mix(0.55, 1., clamp(soak,0.,1.));
      float base = h1(sd+vec2(51.,2.))*6.28318;
      vec2 dir = vec2(cos(base), sin(base));
      float vh, vv, delay = 0., dsize, alpha;
      if (kind > 1.5) {
        if (k < 5.5) { float a = base + 6.28318*(k + 0.35*hk)/6.; dir = vec2(cos(a), sin(a)); vh = (0.3+0.2*hk2); vv = (1.1+0.4*hk); dsize = 0.016; alpha = 0.85; }
        else if (k < 6.5) { vh = 0.03; vv = 1.4+0.25*hk; delay = 0.07; dsize = 0.018; alpha = 0.9; }
        else { float a = base + 6.28318*(hk); dir = vec2(cos(a), sin(a)); vh = 0.15; vv = 0.7+0.2*hk2; dsize = 0.012; alpha = 0.75; }
        vv *= wetS; vh *= wetS;
      } else if (kind > 0.5) {
        float a = base + 6.28318*(k + 0.5*hk)/5.; dir = vec2(cos(a), sin(a)); vh = 0.2+0.15*hk2; vv = 0.75+0.3*hk; dsize = 0.013; alpha = 0.7;
      } else {
        float a = base + 6.28318*(k + 0.6*hk)/4.; dir = vec2(cos(a), sin(a)); vh = 0.15+0.15*hk2; vv = 0.5+0.3*hk; dsize = 0.011; alpha = 0.65;
      }
      vv *= sqrt(sz); vh *= sqrt(sz);
      float t = age - delay;
      float y = vv*t - 4.9*t*t;
      vec3 p = vec3(xz.x + dir.x*vh*t, top + max(y,0.) + 0.01, xz.y + dir.y*vh*t);
      float dist = length(p - cameraPosition);
      float alive = fire * step(k+0.5, nDrops) * step(0., t) * step(0., y) * step(t, 2.*vv/9.8);
      alive *= 1. - smoothstep(uReach-4., uReach, length(xz - uEye.xz));
      if (alive < 0.001) { gl_Position = vec4(2., 2., 2., 1.); gl_PointSize = 0.; vC = vec4(0.); return; }
      // lit as the water it came from: the wet ground's colour toward the sky it mirrors
      vec3 g = uMeans[int(cls+0.5)]*0.72*0.85;
      vec3 water = mix(g, uSkyRefl, kind > 1.5 ? 0.6 : (kind > 0.5 ? 0.45 : 0.55));
      vec3 light = mix(uGround, uSky, 0.7) + uSunCol*0.5 + lampLight(p, vec3(0.,1.,0.), 1.)*1.6;
      vec3 c = water*light + lampLight(p, vec3(0.,1.,0.), 1.)*0.6 + uSkyRefl*(kind > 1.5 ? 0.3 : 0.22);
      vC = vec4(fogged(c, p), alpha*alive);
      vec4 mv = viewMatrix*vec4(p,1.);
      gl_Position = projectionMatrix*mv;
      gl_PointSize = floor(max(1., dsize*sz*uPx/max(-mv.z,0.05)) + 0.5);
    }`,
  fragmentShader: `varying vec4 vC; void main(){ gl_FragColor = vC; }`,
});
const splashGeo = new THREE.BufferGeometry();
splashGeo.setAttribute("position", new THREE.BufferAttribute(dummy, 3));
splashGeo.setAttribute("aCol", new THREE.BufferAttribute(aCol, 4));
splashGeo.setAttribute("aLane", new THREE.BufferAttribute(aLane, 2));
const splashes = new THREE.Points(splashGeo, splashMat);
splashes.frustumCulled = false; splashes.renderOrder = 2;
scene.add(splashes);

// ---- today's near shower, from the game's own seed layout (streak_seed), drawn the GPU way
function streakSeed(index) {
  const hash = salt => { let n = (Math.imul(index, 0x9e3779b9) ^ Math.imul(salt, 0x85ebca6b)) >>> 0; n = (n ^ (n >>> 15)) >>> 0; n = Math.imul(n, 0x2c1b3c6d) >>> 0; n = (n ^ (n >>> 12)) >>> 0; return (n & 0xffffff) / 16777216; };
  return [hash(1) * Math.PI * 2, Math.sqrt(hash(2)), hash(3)];
}
const sSeed = new Float32Array(STREAKS * 2 * 4), sPos = new Float32Array(STREAKS * 2 * 3);
for (let i = 0; i < STREAKS; i++) { const [a, s, p] = streakSeed(i); for (let e = 0; e < 2; e++) sSeed.set([a, s, p, i * 2 + e], (i * 2 + e) * 4); }
const streakMat = new THREE.ShaderMaterial({
  uniforms: U, transparent: true, depthWrite: false,
  vertexShader: COMMON + `
    uniform sampler2D uTops; uniform vec3 uEye; uniform float uStreakLight;
    attribute vec4 aSeed; varying vec4 vC;
    vec2 axial(vec2 p){ float r = p.y/(${W}*0.8660254); float q = p.x/${W} - r*0.5;
      float x = q, z = r, y = -x-z; float rx = floor(x+.5), ry = floor(y+.5), rz = floor(z+.5);
      float dx = abs(rx-x), dy = abs(ry-y), dz = abs(rz-z);
      if (dx > dy && dx > dz) rx = -ry-rz; else if (dy > dz) ry = -rx-rz; else rz = -rx-ry; return vec2(rx, rz); }
    float topAt(vec2 p){ vec2 a = axial(p)+${NR}.; if (a.x < 0. || a.y < 0. || a.x > ${M - 1}. || a.y > ${M - 1}.) return 0.; return texture2D(uTops, (a+0.5)/${M}.).r; }
    void main(){
      float idx = floor(aSeed.w*0.5), end = mod(aSeed.w, 2.);
      float on = step(idx, ${STREAKS}.*clamp(uRain,0.,1.) - 1.);
      vec2 mark = uEye.xz + vec2(cos(aSeed.x), sin(aSeed.x))*aSeed.y*18.;
      float base = topAt(mark);
      float fallen = fract(uTime*70./46. + aSeed.z);
      float topY = base + 46.*(1.-fallen);
      float y = max(topY - end*1.5, base);
      vec3 p = vec3(mark.x, y, mark.y);
      float fade = clamp(fallen/0.15,0.,1.)*clamp((1.-fallen)/0.15,0.,1.);
      float alpha = 0.78*fade*(1. - aSeed.y*(1. - 0.10/0.78));
      alpha *= clamp((length(p-cameraPosition)-1.)/3., 0., 1.)*on;
      vec3 c = vec3(0.52,0.62,0.90)*uStreakLight + lampLight(p, vec3(0.,1.,0.), 1.)*0.5;
      vC = vec4(fogged(c, p), alpha);
      gl_Position = on > 0.5 ? projectionMatrix*viewMatrix*vec4(p,1.) : vec4(2.,2.,2.,1.);
    }`,
  fragmentShader: `varying vec4 vC; void main(){ gl_FragColor = vC; }`,
});
const streakGeo = new THREE.BufferGeometry();
streakGeo.setAttribute("position", new THREE.BufferAttribute(sPos, 3));
streakGeo.setAttribute("aSeed", new THREE.BufferAttribute(sSeed, 4));
const streaks = new THREE.LineSegments(streakGeo, streakMat);
streaks.frustumCulled = false; streaks.renderOrder = 3;
scene.add(streaks);

// the ground colours a droplet starts from: the mean of each class's tile
function computeMeans() {
  try {
    const img = atlas.image, cv = document.createElement("canvas"); cv.width = 32; cv.height = 32;
    const g = cv.getContext("2d");
    [TILE.grass, TILE.path, TILE.boards].forEach((t, i) => {
      g.drawImage(img, (t[0] % 4) * 128 + t[1] * 32, Math.floor(t[0] / 4) * 128 + t[2] * 32, 32, 32, 0, 0, 32, 32);
      const d = g.getImageData(0, 0, 32, 32).data; let r = 0, gg = 0, b = 0;
      for (let j = 0; j < d.length; j += 4) { r += d[j]; gg += d[j + 1]; b += d[j + 2]; }
      const n = d.length / 4 * 255; U.uMeans.value[i].set(r / n, gg / n, b / n);
    });
  } catch (e) { /* file:// may refuse the pixels; the stand-in means stay */ }
}

// ------------------------------------------------------------------ state, light
const RAIN = { light: [0.3, 0.6], heavy: [0.7, 1.0], storm: [1.0, 1.0] };
const S = { mode: "splash", rain: "heavy", tod: "day", view: "orbit", slow: false, pause: false };
function applyLight() {
  const [rain, wet] = RAIN[S.rain]; U.uRain.value = rain; U.uWet.value = wet;
  const day = S.tod === "day", storm = S.rain === "storm";
  if (day) {
    const k = storm ? 0.72 : (S.rain === "light" ? 1.08 : 0.9);
    U.uSky.value.setRGB(0.62 * k, 0.66 * k, 0.72 * k); U.uGround.value.setRGB(0.26 * k, 0.26 * k, 0.24 * k);
    U.uSunCol.value.setRGB(0.42 * k, 0.41 * k, 0.38 * k); U.uFogCol.value.setRGB(0.5 * k, 0.54 * k, 0.58 * k);
    U.uSkyRefl.value.setRGB(0.62 * k, 0.67 * k, 0.73 * k); U.uFogFar.value = storm ? 80 : 130; U.uStreakLight.value = 0.85 * k;
    U.uSunDir.value.set(0.35, 0.85, 0.25).normalize();
    LAMPS.forEach((L, i) => U.uLampCol.value[i].set(0, 0, 0)); lanternMat.color.setRGB(0.25, 0.22, 0.17);
  } else {
    U.uSky.value.setRGB(0.03, 0.038, 0.06); U.uGround.value.setRGB(0.008, 0.009, 0.012);
    U.uSunCol.value.setRGB(0.035, 0.042, 0.06); U.uFogCol.value.setRGB(0.018, 0.022, 0.032);
    U.uSkyRefl.value.setRGB(0.04, 0.05, 0.075); U.uFogFar.value = storm ? 55 : 75; U.uStreakLight.value = 0.1;
    U.uSunDir.value.set(-0.4, 0.75, 0.3).normalize();
    LAMPS.forEach((L, i) => U.uLampCol.value[i].set(1.0, 0.6, 0.27).multiplyScalar(2.3)); lanternMat.color.setRGB(1.0, 0.78, 0.45);
  }
  scene.background = U.uFogCol.value.clone();
  splashes.visible = S.mode === "splash";
  document.getElementById("vtitle").textContent = (S.mode === "splash" ? "With splashes" : "Today") + " · " + S.rain + " · " + S.tod;
}

// ------------------------------------------------------------------ camera: drag to orbit, presets
const VIEWS = {
  orbit: { t: [2, 0.6, -1], yaw: 3.75, pitch: 0.5, dist: 15 },
  stand: { t: [1.8, 0.0, -1.3], yaw: -2.08, pitch: 0.27, dist: 6.0 },
  close: { t: [-1.0, 0.0, -2.3], yaw: -2.08, pitch: 0.42, dist: 2.3 },
  roof: { t: [roofCells[0].x, roofCells[0].h + 2.0, roofCells[0].z], yaw: -0.9, pitch: 0.62, dist: 9.0 },
};
const cam = { t: new THREE.Vector3(), yaw: 0, pitch: 0, dist: 10 };
function setView(v) { const o = VIEWS[v]; cam.t.set(...o.t); cam.yaw = o.yaw; cam.pitch = o.pitch; cam.dist = o.dist; }
function placeCamera() {
  const cp = Math.cos(cam.pitch);
  camera.position.set(cam.t.x + Math.sin(cam.yaw) * cp * cam.dist, cam.t.y + Math.sin(cam.pitch) * cam.dist, cam.t.z + Math.cos(cam.yaw) * cp * cam.dist);
  camera.lookAt(cam.t);
  U.uEye.value.copy(camera.position);
}
const ptrs = new Map(); let pinch = 0;
canvas.addEventListener("pointerdown", e => { canvas.setPointerCapture(e.pointerId); ptrs.set(e.pointerId, [e.clientX, e.clientY]); canvas.style.cursor = "grabbing"; });
canvas.addEventListener("pointermove", e => {
  if (!ptrs.has(e.pointerId)) return;
  const [px, py] = ptrs.get(e.pointerId); ptrs.set(e.pointerId, [e.clientX, e.clientY]);
  if (ptrs.size === 1) { cam.yaw -= (e.clientX - px) * 0.006; cam.pitch = Math.min(1.45, Math.max(0.03, cam.pitch + (e.clientY - py) * 0.005)); }
  else if (ptrs.size === 2) { const [a, b] = [...ptrs.values()]; const d = Math.hypot(a[0] - b[0], a[1] - b[1]); if (pinch) cam.dist = Math.min(80, Math.max(0.8, cam.dist * pinch / d)); pinch = d; }
});
const up = e => { ptrs.delete(e.pointerId); pinch = 0; if (!ptrs.size) canvas.style.cursor = "grab"; };
canvas.addEventListener("pointerup", up); canvas.addEventListener("pointercancel", up);
canvas.addEventListener("wheel", e => { e.preventDefault(); cam.dist = Math.min(80, Math.max(0.8, cam.dist * Math.exp(e.deltaY * 0.001))); }, { passive: false });

// ------------------------------------------------------------------ UI
function seg(id, attr, onPick) {
  const root = document.getElementById(id);
  root.addEventListener("click", e => { const b = e.target.closest("button"); if (!b) return;
    root.querySelectorAll("button").forEach(x => x.setAttribute("aria-pressed", String(x === b))); onPick(b.dataset[attr]); });
}
seg("mode", "m", v => { S.mode = v; applyLight(); });
seg("rain", "r", v => { S.rain = v; applyLight(); });
seg("tod", "tod", v => { S.tod = v; applyLight(); });
seg("views", "view", v => { S.view = v; setView(v); });
document.getElementById("slow").addEventListener("change", e => { S.slow = e.target.checked; });
document.getElementById("pause").addEventListener("change", e => { S.pause = e.target.checked; });
for (const [id, u] of [["density", "uDensity"], ["size", "uSize"]]) {
  const el = document.getElementById(id), out = document.getElementById(id + "Out");
  const f = () => { U[u].value = +el.value; out.textContent = (+el.value).toFixed(2) + "x"; };
  el.addEventListener("input", f); f();
}
// info modals: the cross, Esc or a click outside closes them
const modal = document.getElementById("modal"), mbody = document.getElementById("modalBody");
let lastFocus = null;
document.querySelectorAll("[data-modal]").forEach(b => b.addEventListener("click", () => {
  lastFocus = b; document.getElementById("modalTitle").textContent = b.dataset.title; mbody.innerHTML = "";
  b.dataset.modal.split(" ").forEach(id => mbody.appendChild(document.getElementById(id).content.cloneNode(true)));
  modal.hidden = false; document.getElementById("modalClose").focus();
}));
const closeModal = () => { modal.hidden = true; if (lastFocus) lastFocus.focus(); };
document.getElementById("modalClose").addEventListener("click", closeModal);
modal.addEventListener("click", e => { if (e.target === modal) closeModal(); });
document.addEventListener("keydown", e => { if (e.key === "Escape" && !modal.hidden) closeModal(); });

// ------------------------------------------------------------------ loop
function resize() {
  const w = canvas.clientWidth, h = canvas.clientHeight;
  if (canvas.width !== Math.floor(w * renderer.getPixelRatio()) || canvas.height !== Math.floor(h * renderer.getPixelRatio())) {
    renderer.setSize(w, h, false); camera.aspect = w / h; camera.updateProjectionMatrix();
  }
  U.uPx.value = canvas.height / (2 * Math.tan(camera.fov * Math.PI / 360));
}
let simT = 20, last = performance.now(), running = true;
function frame(now) {
  if (!running) return;
  const dt = Math.min(0.1, (now - last) / 1000); last = now;
  if (!S.pause) simT += dt * (S.slow ? 0.1 : 1);
  U.uTime.value = simT;
  resize(); placeCamera(); renderer.render(scene, camera);
  requestAnimationFrame(frame);
}
document.addEventListener("visibilitychange", () => { running = !document.hidden; if (running) { last = performance.now(); requestAnimationFrame(frame); } });
setView("orbit"); applyLight(); requestAnimationFrame(frame);
// for stills: a fixed time and a paused clock
window.__rain = { setTime(t) { simT = t; }, setCam(t, yaw, pitch, dist) { cam.t.set(...t); cam.yaw = yaw; cam.pitch = pitch; cam.dist = dist; }, render() { U.uTime.value = simT; resize(); placeCamera(); renderer.render(scene, camera); }, lanes: nLanes };
})();
