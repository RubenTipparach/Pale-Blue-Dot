// Bakes the fish mockup's species sheet (docs/mockups/fish.html, the "Species sheet" tab) to PNGs, so the sheet shows
// images and opens no WebGL context. The page renders every thumbnail itself: opened with ?bake#sheet it draws each
// individual live, with the same genome, camera, size (320 x 180) and rare morphs the sheet shows, and hands the
// canvases back as data URLs on window.__fishBake. This script only serves the folder, opens the page and saves them.
//
//   node tools/mockups/bake_fish_sheet.js          (from the repository root; needs Node and Playwright)
//
// Writes docs/mockups/fish/sheet/<species>-<seed>.png, and <species>-<seed>-<morph>.png for a rare morph, and removes
// stale PNGs there that the page no longer asks for. After a re-bake that changes the images, bump SHEET_V in fish.html.
// Render on the GPU where there is one (CLAUDE.md, "Render on the GPU when there is one"): pass --gpu on the desktop.
// Without it Chromium picks its own backend, which in a cloud container with no GPU is SwiftShader (software); for
// these still thumbnails that changes nothing but speed.
const { chromium } = require('playwright');
const http = require('http');
const fs = require('fs');
const path = require('path');

const ROOT = path.resolve(__dirname, '..', '..', 'docs', 'mockups');
const OUT = path.join(ROOT, 'fish', 'sheet');
const GPU = process.argv.includes('--gpu');
const TYPES = { '.html': 'text/html; charset=utf-8', '.png': 'image/png', '.js': 'text/javascript', '.css': 'text/css', '.json': 'application/json' };

function serve() {
  const server = http.createServer((req, res) => {
    const rel = decodeURIComponent(req.url.split('?')[0].split('#')[0]);
    const file = path.join(ROOT, rel);
    if (!file.startsWith(ROOT + path.sep)) { res.writeHead(403); return res.end(); }
    fs.readFile(file, (err, data) => {
      if (err) { res.writeHead(404); return res.end(); }
      res.writeHead(200, { 'Content-Type': TYPES[path.extname(file)] || 'application/octet-stream' });
      res.end(data);
    });
  });
  return new Promise(r => server.listen(0, '127.0.0.1', () => r(server)));
}

(async () => {
  const server = await serve();
  const url = `http://127.0.0.1:${server.address().port}/fish.html?bake#sheet`;
  const args = GPU ? ['--use-angle=d3d11', '--enable-gpu', '--ignore-gpu-blocklist'] : [];
  const browser = await chromium.launch({ args });
  const page = await browser.newPage({ viewport: { width: 1280, height: 900 }, deviceScaleFactor: 1 });
  const errs = [];
  page.on('console', m => { if (m.type() === 'error') errs.push(m.text()); });
  page.on('pageerror', e => errs.push(e.message));
  await page.goto(url, { waitUntil: 'load', timeout: 120000 });
  await page.waitForFunction(() => window.__fishBake, null, { timeout: 60000 });
  const shots = await page.evaluate(() => window.__fishBake);
  const renderer = await page.evaluate(() => {
    const gl = document.createElement('canvas').getContext('webgl'); const d = gl && gl.getExtension('WEBGL_debug_renderer_info');
    return d ? gl.getParameter(d.UNMASKED_RENDERER_WEBGL) : 'unknown';
  });
  await browser.close();
  server.close();
  if (errs.length) { console.error('page errors:\n  ' + errs.join('\n  ')); process.exit(1); }

  fs.mkdirSync(OUT, { recursive: true });
  const keep = new Set();
  let bytes = 0;
  for (const { file, png } of shots) {
    const buf = Buffer.from(png.replace(/^data:image\/png;base64,/, ''), 'base64');
    fs.writeFileSync(path.join(OUT, file), buf);
    keep.add(file); bytes += buf.length;
  }
  for (const f of fs.readdirSync(OUT)) if (f.endsWith('.png') && !keep.has(f)) { fs.unlinkSync(path.join(OUT, f)); console.log(`removed stale ${f}`); }
  console.log(`baked ${shots.length} thumbnails, ${(bytes / 1024).toFixed(0)} KiB, into ${path.relative(process.cwd(), OUT)}`);
  console.log(`renderer: ${renderer}${/swiftshader/i.test(renderer) ? ' (software; no GPU here)' : ''}`);
})().catch(e => { console.error(e); process.exit(1); });
