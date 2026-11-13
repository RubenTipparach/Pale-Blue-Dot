// A player-like test of the hangar mockup's flight (docs/mockups/hangar.html, decision 2a).
// Serve docs/mockups (python3 -m http.server 8763 from that folder), then: node tools/mockups/hangar_flight_test.js
// The page's ?test hook steps the same simulation the frame loop steps, 120 steps a second, with the keys a
// player holds put in the same key set the keyboard fills; one short real-keyboard check proves the wiring.
const { chromium } = require('playwright');
const URL = process.env.HANGAR_URL || 'http://localhost:8763/hangar.html?test&still';
const fail = [];
const check = (ok, what) => { console.log(`${ok ? 'PASS' : 'FAIL'}  ${what}`); if (!ok) fail.push(what); };
(async () => {
  const browser = await chromium.launch({args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist']});
  const page = await browser.newPage({viewport: {width: 1000, height: 700}});
  const errs = [];
  page.on('console', m => { if (m.type() === 'error') errs.push(m.text()); });
  page.on('pageerror', e => errs.push(e.message));
  await page.goto(URL, {waitUntil: 'load', timeout: 120000});
  await page.waitForFunction(() => window.__hangar && !document.getElementById('loading'), null, {timeout: 600000});

  // 1. every preset: its stall speed, and how far it rolls before it unsticks (left at the 70% it starts at,
  //    and with Shift held to full power); helicopters and the tiltrotor lift off on Shift
  console.log('\n-- presets: stall speed and take-off roll --');
  for (const id of ['sharky', 'gull', 'dart', 'puffin', 'bumble']) {
    const r = await page.evaluate(id => {
      const H = window.__hangar, m = H.model(id), lift = s => s.takeoff ? s.takeoff : null;
      H.fly(id); const idle = lift(H.run(20, []));
      H.fly(id); const full = lift(H.run(20, [[0, 20, ['ShiftLeft']]]));
      H.fly(id); const climb = H.run(30, [[0, 2, ['ShiftLeft']]]);
      return {id, kind: m.kind, stall: m.stall ? +m.stall.toFixed(1) : null, idle, full, climbAgl: climb.log[29] && climb.log[29].agl, crashes: climb.crashes.length};
    }, id);
    console.log(`${id.padEnd(7)} ${r.kind.padEnd(5)} stall ${r.stall ?? '-'} m/s | at 70%: ${r.idle ? `unsticks after ${r.idle.roll} m at ${r.idle.speed} m/s` : 'stays down'} | Shift held: ${r.full ? `${r.full.roll} m at ${r.full.speed} m/s` : 'stays down'} | height after 30 s: ${r.climbAgl} m`);
    if (r.kind === 'plane') { check(r.stall >= 18 && r.stall <= 24, `${id} stalls at 18-24 m/s (${r.stall})`); check(r.full && r.full.roll < 120, `${id} unsticks in a short roll with Shift (${r.full && r.full.roll} m)`); }
    check(r.crashes === 0 && r.climbAgl > 30, `${id} climbs away with margin (${r.climbAgl} m after 30 s, no crash)`);
  }

  // 2. the circuit: Sharky off the runway with only W, Shift, A and D, 60 sim-seconds
  console.log('\n-- circuit: Sharky, keys W/Shift/A/D only, 60 s --');
  const script = [[0, 2, ['ShiftLeft']], [16, 26, ['KeyA']], [34, 44, ['KeyA']], [47, 49, ['KeyD']], [50, 56, ['KeyW']]];
  const c = await page.evaluate(script => { const H = window.__hangar; H.fly('sharky'); return H.run(60, script); }, script);
  for (const l of c.log.filter((_, i) => i % 5 === 0)) console.log(`  t ${String(l.t).padStart(2)} s  x ${String(l.x).padStart(5)} z ${String(l.z).padStart(5)}  ${String(l.agl).padStart(4)} m up  ${l.v} m/s  bank ${l.bank} pitch ${l.pitch} hdg ${l.hdg}  power ${l.power}%  ${l.stall ? 'STALL ' : ''}${l.keys}`);
  console.log(`  takeoff after ${c.takeoff && c.takeoff.roll} m at ${c.takeoff && c.takeoff.speed} m/s; lowest after climb-out ${c.minAgl} m; max bank ${c.maxBank} deg; stall warning ${c.stallSec} s; crashes ${c.crashes.length}`);
  check(c.takeoff !== null, 'Sharky took off');
  check(c.crashes.length === 0, 'no crash in the circuit');
  check(c.minAgl > 5, `never stalled into the ground (lowest ${c.minAgl} m after climb-out)`);
  check(c.maxBank <= 60, `bank stayed inside the 60 deg limit (${c.maxBank})`);
  check(c.pen === 0, `no point inside the ground or a solid at any step (${c.pen})`);

  // 3. into the hangar wall at speed: a crash, reported, and no point ever inside the box
  console.log('\n-- crash: Sharky flown into the hangar\'s west wall --');
  const h = await page.evaluate(() => { const H = window.__hangar; H.fly('sharky'); H.put([-110, 6, -35], 0, 30); const r = H.run(4.8, [[0, 4.8, ['ShiftLeft']]]); return r; });
  console.log(`  crashes ${JSON.stringify(h.crashes)}; points ever inside: ${h.pen} (deepest ${h.penMax} m); reset to ${JSON.stringify(h.end.p)}`);
  check(h.crashes.length === 1 && /hangar/.test(h.crashes[0].what), 'a crash into the hangar is reported');
  check(h.pen === 0, 'no point of the plane ever inside the hangar box');
  check(Math.abs(h.end.p[2] - 35) < 1 && h.end.p[0] < -100 && !h.end.crashed, `2 s after the crash it is back at the runway's end (${h.end.p})`);

  // 4. into the ground: a dive is a crash, and nothing passes under the cells
  await page.click('#expertBtn');   // assists off: W is the elevator, and a held W is a dive
  const g = await page.evaluate(() => { const H = window.__hangar; H.fly('sharky'); H.put([400, 120, 35], 0, 30); return H.run(8, [[0, 8, ['KeyW', 'ShiftLeft']]]); });
  await page.click('#expertBtn');
  console.log(`\n-- crash: Expert, a full-power dive at the hills -- crashes ${JSON.stringify(g.crashes)}, points under the ground: ${g.pen}`);
  check(g.crashes.length >= 1 && g.pen === 0, 'a dive into the ground is a crash and never clips');

  // 4b. a house in Wendle, and a parked craft on the apron
  const w = await page.evaluate(() => { const H = window.__hangar, t = H.towns()[0]; H.fly('sharky'); H.put([t.x - 160, t.h + 6, t.z - 14], 0, 30); return H.run(6, []); });
  console.log(`-- crash: along Wendle's street at roof height -- crashes ${JSON.stringify(w.crashes)}, points inside: ${w.pen}`);
  check(w.crashes.length >= 1 && /Wendle/.test(w.crashes[0].what) && w.pen === 0, 'a house in town is solid');
  const pc = await page.evaluate(() => { const H = window.__hangar; H.fly('sharky'); H.put([-70, 3, -6], 0, 26); return H.run(3, []); });
  console.log(`-- crash: low across the apron -- crashes ${JSON.stringify(pc.crashes)}, points inside: ${pc.pen}`);
  check(pc.crashes.length >= 1 && pc.pen === 0, 'a parked craft is solid');

  // 5. gentle on the gear is a landing: 2 m over the runway at 25 m/s, throttle back
  const l = await page.evaluate(() => { const H = window.__hangar; H.fly('sharky'); H.put([-60, 3.2, 35], 0, 25); return H.run(6, [[0, 6, ['ControlLeft']]]); });
  console.log(`\n-- landing: settle onto the runway -- crashes ${JSON.stringify(l.crashes)}, ends at ${JSON.stringify(l.end.p)}`);
  check(l.crashes.length === 0 && l.end.ground, 'a gentle touchdown is a landing, not a crash');

  // 6. an airship into the shed's wall
  const a = await page.evaluate(() => { const H = window.__hangar; H.fly('puff'); return H.run(60, [[15, 60, ['KeyW']], [30, 45, ['KeyA']]]); });
  console.log(`\n-- airship: Puff slipped from its mast, rising on its free lift, then engines and rudder, 60 s -- crashes ${JSON.stringify(a.crashes)}, height ${a.end.p[1]} m, points inside: ${a.pen}`);
  check(a.crashes.length === 0 && a.pen === 0 && a.end.p[1] > 15, 'Puff leaves its mast and flies without touching anything');
  const b = await page.evaluate(() => { const H = window.__hangar; H.fly('puff'); H.put([62, 3, -36], 0, 8); return H.run(6, [[0, 6, ['KeyW']]]); });
  console.log(`-- airship: Puff driven at the shed's west wall -- crashes ${JSON.stringify(b.crashes)}, points inside: ${b.pen}`);
  check(b.crashes.length === 1 && /shed/.test(b.crashes[0].what) && b.pen === 0, 'an airship into the shed is a crash and never passes into it');

  // 7. the real keyboard: Shift held for a second in real time raises the throttle
  await page.evaluate(() => window.__hangar.fly('sharky'));
  await page.keyboard.down('ShiftLeft'); await page.waitForTimeout(1500); await page.keyboard.up('ShiftLeft');
  const pw = await page.evaluate(() => document.getElementById('flyHud').innerText.match(/Throttle\s+(\d+)%/));
  console.log(`\n-- keyboard: Shift held 1.5 s in real time -> ${pw && pw[0]}`);
  check(pw && +pw[1] > 70, 'the keys reach the flight model');

  check(errs.length === 0, `no console errors${errs.length ? ': ' + errs.slice(0, 3).join(' | ') : ''}`);
  console.log(`\n${fail.length ? `${fail.length} FAILED` : 'ALL PASSED'}`);
  await browser.close(); process.exit(fail.length ? 1 : 0);
})();
