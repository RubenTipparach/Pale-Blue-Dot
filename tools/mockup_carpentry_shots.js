// M33's Town tab with the Carpentry branch (`furniture` task 1.3, M33
// version 6): the whole tree fitted, the Carpentry branch's first card
// opened, at desktop width and at phone width.
// Usage: NODE_PATH=$(npm root -g) node tools/mockup_carpentry_shots.js <mission-tree.html> <out dir>
const fs = require("fs"); const { chromium } = require("playwright");
(async () => {
  const [html, out] = process.argv.slice(2);
  fs.mkdirSync(out, { recursive: true });
  const browser = await chromium.launch();
  const errors = [];
  for (const [tag, vp] of [["desktop", { width: 1440, height: 900 }], ["phone", { width: 390, height: 844 }]]) {
    const page = await browser.newPage({ viewport: vp });
    page.on("pageerror", (e) => errors.push(String(e)));
    await page.goto(`file://${html}#town`);
    await page.waitForTimeout(1500);
    await page.click("#fitBtn").catch(() => {});
    await page.waitForTimeout(500);
    await page.screenshot({ path: `${out}/m33-town-carpentry-fit-${tag}.png` });
    // the Carpentry branch's cards, opened one by one
    for (const id of ["carpenter", "furnish"]) {
      const card = await page.$(`[data-id="${id}"]`);
      if (card) { await card.scrollIntoViewIfNeeded().catch(() => {}); await card.click().catch(() => {}); await page.waitForTimeout(500); }
      await page.screenshot({ path: `${out}/m33-town-carpentry-${id}-${tag}.png` });
    }
    await page.close();
  }
  console.log("errors:", errors.length ? errors.join("; ") : "none");
  await browser.close();
})();
