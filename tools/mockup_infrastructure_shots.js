// M33's INFRASTRUCTURE tab (`infrastructure-missions` task 1.4, M33 version 12): the whole tree
// fitted, each branch focused, missions selected with their MISSION and RECIPES panels, the TOWN tab
// without its roads, and the old #power link landing on #infrastructure. Every other tab is loaded and
// its page errors counted. Headless Chromium renders on the CPU (SwiftShader): looks, not speed.
// Usage: NODE_PATH=$(npm root -g) node tools/mockup_infrastructure_shots.js <mission-tree.html or its http URL> <out dir>
// (serve docs/mockups over http, as the published page is, so the MILLS tab can load its textures)
const fs = require("fs");
const { chromium } = require("playwright");
(async () => {
  const [html, out] = process.argv.slice(2);
  fs.mkdirSync(out, { recursive: true });
  const browser = await chromium.launch();
  const errors = [];
  const open = async (hash, vp) => {
    const page = await browser.newPage({ viewport: vp || { width: 1440, height: 900 }, deviceScaleFactor: 2 });
    page.on("pageerror", (e) => errors.push(`${hash}: ${e}`));
    page.on("console", (m) => { if (m.type() === "error") errors.push(`${hash}: ${m.text()}`); });
    await page.goto(`${/^https?:/.test(html) ? html : "file://" + html}#${hash}`);
    await page.waitForTimeout(1200);
    return page;
  };
  const shot = (page, name) => page.screenshot({ path: `${out}/${name}.png` });

  // the old link
  {
    const page = await open("power");
    const h = await page.evaluate(() => [location.hash, window.missionTree.tree, window.missionTree.missions]);
    console.log("#power lands on", h.join(" "));
    await page.close();
  }

  const page = await open("infrastructure");
  console.log("steps", await page.evaluate(() => window.missionTree.steps));
  await shot(page, "infra-open");
  await page.click("#fitBtn");
  await page.waitForTimeout(400);
  await shot(page, "infra-fit");
  // the bot run to the end, fitted
  await page.evaluate(() => { window.missionTree.setStep(window.missionTree.steps); window.missionTree.fit(); });
  await page.waitForTimeout(300);
  await shot(page, "infra-done-fit");
  await page.evaluate(() => window.missionTree.setStep(window.missionTree.sample));
  for (const key of ["stone", "roads", "civil", "town", "mill", "wind", "water", "lines", "appl"]) {
    await page.evaluate((k) => window.missionTree.focusPath(k), key);
    await page.waitForTimeout(700);
    await shot(page, `infra-branch-${key}`);
  }
  for (const id of ["stone-bridge", "gears", "race", "two-towns", "grinder"]) {
    await page.evaluate((m) => { window.missionTree.select(m); window.missionTree.focus(m, 7); }, id);
    await page.waitForTimeout(400);
    await shot(page, `infra-selected-${id}`);
  }
  await page.click("#tabR");
  await page.waitForTimeout(300);
  await shot(page, "infra-recipes");
  await page.evaluate(() => { const b = document.querySelector("#pbody"); b.scrollTop = b.scrollHeight; });
  await page.waitForTimeout(200);
  await shot(page, "infra-recipes-end");
  await page.close();

  // phone width
  {
    const p = await open("infrastructure", { width: 390, height: 844 });
    await p.click("#fitBtn");
    await p.waitForTimeout(400);
    await shot(p, "infra-fit-phone");
    await p.evaluate(() => window.missionTree.select("stone-bridge"));
    await p.waitForTimeout(300);
    await shot(p, "infra-selected-phone");
    await p.close();
  }

  // the Town tab without roads, and every other tab loading
  for (const tab of ["town", "starter", "survival", "crew", "mills"]) {
    const p = await open(tab);
    if (tab === "town") {
      await p.click("#fitBtn");
      await p.waitForTimeout(400);
      const roads = await p.evaluate(() => ["grinder", "mark", "pave"].filter((id) => document.querySelector(`[data-id="${id}"]`)));
      console.log("town tab road cards:", roads.length ? roads.join(",") : "none");
    }
    if (tab === "mills") await p.waitForTimeout(2500);
    await shot(p, `tab-${tab}`);
    await p.close();
  }
  console.log("errors:", errors.length ? errors.join("; ") : "none");
  await browser.close();
})();
