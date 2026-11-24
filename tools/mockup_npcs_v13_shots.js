// M5 version 13's screenshots (npcs.html, the Talk tab): the trade columns and the job board in their
// windows of seven rows with the stone scroll bar (design 38b), scrolled by the wheel and by dragging the
// thumb, and a column filtered by shelf (design 38c). Chromium on SwiftShader (software GL), so the frames
// show the look, never the frame rate.
// Usage: THREE_JS=<three.min.js r128> NODE_PATH=$(npm root -g) node tools/mockup_npcs_v13_shots.js <mockups dir> <out dir>
const fs = require("fs"); const { chromium } = require(process.env.PW || "playwright");
(async () => {
  const [dir, out] = process.argv.slice(2);
  fs.mkdirSync(out, { recursive: true });
  const browser = await chromium.launch({ executablePath: "/opt/pw-browsers/chromium", args: ["--use-gl=swiftshader", "--enable-unsafe-swiftshader", "--allow-file-access-from-files"] });
  const body = fs.readFileSync(process.env.THREE_JS); const errors = [];
  async function page(url, vw = 1440, vh = 1400) {
    const p = await browser.newPage({ viewport: { width: vw, height: vh } });
    p.on("pageerror", e => errors.push(url + ": " + String(e)));
    p.on("console", m => { if (m.type() === "error") errors.push(url + " console: " + m.text()); });
    await p.route("**/three.min.js", r => r.fulfill({ body, contentType: "text/javascript" }));
    await p.route("https://fonts.googleapis.com/**", r => r.fulfill({ body: "", contentType: "text/css" }));
    await p.goto(`file://${dir}/npcs.html${url}`);
    for (let k = 0; k < 180; k++) { if (await p.evaluate(() => !!(window.__talk && document.querySelector("#tkFit .kn")))) break; await p.waitForTimeout(500); }
    await p.waitForTimeout(1800);
    return p;
  }
  const shot = async (p, name) => { await p.waitForTimeout(400); await (await p.$("#tkStage")).screenshot({ path: `${out}/${name}.jpg`, type: "jpeg", quality: 90 }); };
  const state = p => p.evaluate(() => JSON.stringify(window.__talk.TK.scroll));
  const wheelOver = async (p, list, notches) => {
    for (let k = 0; k < Math.abs(notches); k++) {
      const box = await (await p.$(`#tkFit [data-list="${list}"]`)).boundingBox();
      await p.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
      await p.mouse.wheel(0, Math.sign(notches) * 100); await p.waitForTimeout(150);
    }
  };
  const log = {};
  for (const [tod, h] of [["day", 12.5], ["night", 22.5]]) {
    const p = await page(`?tpanel=trade&thour=${h}#talk`);
    await p.mouse.move(5, 5);
    await shot(p, `v13-trade-top-${tod}`);
    log[`top-${tod}`] = await state(p);
    if (tod === "night") {
      await wheelOver(p, "buy", 2); await wheelOver(p, "sell", 1); await p.mouse.move(5, 5);
      log.wheeled = await state(p);
      await shot(p, "v13-trade-wheeled-night");
      // drag the BUY thumb to the bottom of its track
      const th = await (await p.$('#tkFit [data-track="buy"] .kthumb')).boundingBox();
      const tr = await (await p.$('#tkFit [data-track="buy"]')).boundingBox();
      await p.mouse.move(th.x + th.width / 2, th.y + 2); await p.mouse.down();
      await p.mouse.move(th.x + th.width / 2, tr.y + tr.height + 20, { steps: 8 }); await p.mouse.up(); await p.mouse.move(5, 5);
      log.dragged = await state(p);
      await shot(p, "v13-trade-dragged-night");
      // the BUY column filtered to food, then SELL to organics
      await p.click('#tkFit .ktab[aria-label="Food"] >> nth=0'); await p.mouse.move(5, 5);
      await p.click('#tkFit .ktab[aria-label="Organics"] >> nth=0'); await p.mouse.move(5, 5);
      log.filtered = await state(p);
      await shot(p, "v13-trade-filter-night");
    }
    await p.close();
  }
  const b = await page(`?tpanel=board&thour=22.5#talk`);
  await b.mouse.move(5, 5);
  await shot(b, "v13-board-top-night");
  await wheelOver(b, "board", 2); await b.mouse.move(5, 5);
  log.board = await state(b);
  await shot(b, "v13-board-wheeled-night");
  await b.close();
  const ph = await page(`?tpanel=trade&thour=22.5#talk`, 390, 844);
  await ph.screenshot({ path: `${out}/v13-trade-phone.jpg`, type: "jpeg", quality: 85, fullPage: false });
  await ph.close();
  console.log(JSON.stringify(log, null, 1));
  console.log(errors.length ? errors.join("\n") : "no page errors");
  await browser.close();
})();
