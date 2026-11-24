// M5 version 12's screenshots (npcs.html): the Talk tab's panels by day and at night, the Faces tab
// hero's ears close up, and the soft-shaded beards. Chromium on SwiftShader (software GL), so the
// frames show the look, never the frame rate.
// Usage: THREE_JS=<three.min.js r128> NODE_PATH=$(npm root -g) node tools/mockup_npcs_v12_shots.js <mockups dir> <out dir> [talk,ears,beard]
const fs = require("fs"); const { chromium } = require(process.env.PW || "playwright");
(async () => {
  const [dir, out, which] = process.argv.slice(2);
  const parts = (which || "talk,ears,beard").split(",");
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
    return p;
  }
  // wait for a condition by asking every half second (waitForFunction stalls under SwiftShader's load)
  async function until(p, fn) {
    for (let k = 0; k < 180; k++) { if (await p.evaluate(fn)) return; await p.waitForTimeout(500); }
    console.log(errors.join("\n"), await p.evaluate(() => [location.href, !!window.__talk, document.getElementById("err").textContent]));
    throw new Error("timed out: " + fn);
  }
  if (parts.includes("talk")) {
    const shots = [];
    for (const [tod, h] of [["day", 12.5], ["night", 22.5]]) {
      shots.push([`v12-talk-talk-${tod}`, `?tpanel=talk&thour=${h}#talk`]);
      shots.push([`v12-talk-trade-${tod}`, `?tpanel=trade&thour=${h}#talk`]);
      shots.push([`v12-talk-board-${tod}`, `?tpanel=board&thour=${h}#talk`]);
      shots.push([`v12-talk-basket-${tod}`, `?tpanel=trade&tmode=basket&tbasket=take:flatbread,take:flatbread,take:mango,give:fish:perch,give:log,give:log&thour=${h}#talk`]);
    }
    for (const [name, url] of shots) {
      const p = await page(url);
      await until(p, () => !!(window.__talk && document.querySelector("#tkFit .kn")));
      await p.waitForTimeout(1800);
      await (await p.$("#tkStage")).screenshot({ path: `${out}/${name}.jpg`, type: "jpeg", quality: 90 });
      await p.close();
    }
    // the clicks: talk, then TRADE, BUY, BACK, WORK, HAND IN, BYE
    const p = await page(`?tpanel=talk&thour=22.5#talk`);
    await until(p, () => !!(window.__talk && document.querySelector("#tkFit .kplaque")));
    await p.waitForTimeout(1200);
    const press = async label => { await p.click(`#tkFit .kplaque[aria-label="${label}"] >> nth=0`); await p.waitForTimeout(250); };
    await press("TRADE"); await press("BUY"); const afterBuy = await p.evaluate(() => [window.__talk.TK.panel, window.__talk.ST().purse, window.__talk.TK.said]);
    await press("SELL"); const afterSell = await p.evaluate(() => [window.__talk.ST().purse, window.__talk.TK.said]);
    await press("BACK"); await press("WORK"); await press("HAND IN"); const afterHand = await p.evaluate(() => [window.__talk.TK.panel, window.__talk.ST().purse, window.__talk.TK.said]);
    await press("TAKE"); await p.waitForTimeout(600);
    await (await p.$("#tkStage")).screenshot({ path: `${out}/v12-talk-clicked-board-night.jpg`, type: "jpeg", quality: 90 });
    await press("BACK"); await press("BYE"); const afterBye = await p.evaluate(() => [window.__talk.TK.panel, window.__talk.who.talkE]);
    console.log("clicks:", JSON.stringify({ afterBuy, afterSell, afterHand, afterBye }));
    await p.close();
    const ph = await page(`?tpanel=trade&thour=22.5#talk`, 390, 844);
    await until(ph, () => !!(window.__talk && document.querySelector("#tkFit .kn")));
    await ph.waitForTimeout(1500);
    await ph.screenshot({ path: `${out}/v12-talk-trade-phone.jpg`, type: "jpeg", quality: 85, fullPage: false });
    await ph.close();
  }
  const faceShot = async (name, q, view, hour) => {
    const p = await page(`?fbeard=${q.beard}&fexpr=${q.expr || "neutral"}&fhour=${hour}&fview=face#faces`, 900, 700);
    await until(p, () => !!(window.__faces && window.__faces.hero()));
    await p.waitForTimeout(1200);
    await p.evaluate(([v]) => { const F = window.__faces; F.setView("face"); Object.assign(F.FO, v); }, [view]);
    await p.waitForTimeout(q.talk ? 900 : 1500);
    await (await p.$("#fc")).screenshot({ path: `${out}/${name}.jpg`, type: "jpeg", quality: 90 });
    await p.close();
  };
  if (parts.includes("ears")) {
    const V = {
      front: { yaw: 0, pitch: 0.05, dist: 2.4 },
      "three-quarter": { yaw: 0.85, pitch: 0.1, dist: 1.2 },
      side: { yaw: 1.57, pitch: 0.05, dist: 1.05 },
      "side-close": { yaw: 1.45, pitch: 0.08, dist: 0.62 },
    };
    for (const [v, view] of Object.entries(V)) await faceShot(`v12-ears-${v}`, { beard: "none" }, view, 13);
    await faceShot(`v12-ears-three-quarter-night`, { beard: "none" }, V["three-quarter"], 22.5);
  }
  if (parts.includes("beard")) {
    const V = { front: { yaw: 0, pitch: 0.05, dist: 1.6 }, "three-quarter": { yaw: 0.75, pitch: 0.12, dist: 1.65 }, side: { yaw: 1.5, pitch: 0.05, dist: 1.7 }, below: { yaw: 0.45, pitch: -0.15, dist: 1.65 } };
    for (const b of ["goatee", "full"]) {
      const bn = b === "full" ? "full-front" : b;
      for (const [v, view] of Object.entries(V)) await faceShot(`v12-beard-${bn}-${v}`, { beard: b }, view, 13);
      await faceShot(`v12-beard-${bn}-talking`, { beard: b, expr: "talking", talk: true }, V.front, 13);
      await faceShot(`v12-beard-${bn}-night`, { beard: b, expr: "happy" }, V["three-quarter"], 22.5);
    }
  }
  console.log("errors:", errors.length ? "\n" + errors.join("\n") : "none");
  await browser.close();
})();
