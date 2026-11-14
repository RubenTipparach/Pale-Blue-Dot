// A player-like test of the hangar mockup's flight (docs/mockups/hangar.html, decision 2a). The stick is the
// player's alone: nothing rolls, pitches or yaws the craft for them (owner, 2026-11-13).
// Serve docs/mockups (python3 -m http.server 8763 from that folder), then: node tools/mockups/hangar_flight_test.js
// The page's ?test hook steps the same simulation the frame loop steps, 120 steps a second, with the keys a
// player holds put in the same key set the keyboard fills; one short real-keyboard check proves the wiring.
const { chromium } = require('playwright');
const URL = process.env.HANGAR_URL || 'http://localhost:8763/hangar.html?test&still';
const fail = [];
const check = (ok, what) => { console.log(`${ok ? 'PASS' : 'FAIL'}  ${what}`); if (!ok) fail.push(what); };
(async () => {
  const browser = await chromium.launch({args: ['--use-angle=swiftshader', '--enable-unsafe-swiftshader', '--ignore-gpu-blocklist', '--js-flags=--expose-gc', '--enable-precise-memory-info']});
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

  // 2. the circuit: Sharky off the runway, climbing on S, turning by rolling with A and pulling with S, rolling level with D
  console.log('\n-- circuit: Sharky, flown by hand, 60 s --');
  const script = [[0, 2, ['ShiftLeft']], [3, 6, ['KeyS']], [15, 15.5, ['KeyA']], [15.5, 20, ['KeyS']], [20, 20.9, ['KeyD']], [30, 30.5, ['KeyA']], [30.5, 35, ['KeyS']], [35, 35.9, ['KeyD']]];
  const c = await page.evaluate(script => { const H = window.__hangar; H.fly('sharky'); return H.run(60, script); }, script);
  for (const l of c.log.filter((_, i) => i % 5 === 0)) console.log(`  t ${String(l.t).padStart(2)} s  x ${String(l.x).padStart(5)} z ${String(l.z).padStart(5)}  ${String(l.agl).padStart(4)} m up  ${l.v} m/s  bank ${l.bank} pitch ${l.pitch} hdg ${l.hdg}  power ${l.power}%  ${l.stall ? 'STALL ' : ''}${l.keys}`);
  console.log(`  takeoff after ${c.takeoff && c.takeoff.roll} m at ${c.takeoff && c.takeoff.speed} m/s; lowest after climb-out ${c.minAgl} m; max bank ${c.maxBank} deg; stall warning ${c.stallSec} s; crashes ${c.crashes.length}`);
  check(c.takeoff !== null, 'Sharky took off');
  check(c.crashes.length === 0, 'no crash in the circuit');
  check(c.minAgl > 5, `never stalled into the ground (lowest ${c.minAgl} m after climb-out)`);
  check(c.pen === 0, `no point inside the ground or a solid at any step (${c.pen})`);

  // 3. into the hangar wall at speed: a crash, reported, and no point ever inside the box
  console.log('\n-- crash: Sharky flown into the hangar\'s west wall --');
  const h = await page.evaluate(() => { const H = window.__hangar; H.fly('sharky'); H.put([-110, 6, -35], 0, 30); const r = H.run(4.8, [[0, 4.8, ['ShiftLeft']]]); return r; });
  console.log(`  crashes ${JSON.stringify(h.crashes)}; points ever inside: ${h.pen} (deepest ${h.penMax} m); reset to ${JSON.stringify(h.end.p)}`);
  check(h.crashes.length === 1 && /hangar/.test(h.crashes[0].what), 'a crash into the hangar is reported');
  check(h.pen === 0, 'no point of the plane ever inside the hangar box');
  check(Math.abs(h.end.p[2] - 35) < 1 && h.end.p[0] < -100 && !h.end.crashed, `2 s after the crash it is back at the runway's end (${h.end.p})`);

  // 4. into the ground: a dive is a crash, and nothing passes under the cells
  const g = await page.evaluate(() => { const H = window.__hangar; H.fly('sharky'); H.put([400, 120, 35], 0, 30); return H.run(8, [[0, 8, ['KeyW', 'ShiftLeft']]]); });
  console.log(`\n-- crash: a full-power dive at the hills -- crashes ${JSON.stringify(g.crashes)}, points under the ground: ${g.pen}`);
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
  check(l.crashes.length === 0 && l.end.p[1] < 3, 'a gentle touchdown is a landing, not a crash');

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

  // 8. the flight effects (owner, 2026-11-13): airflow lines and wingtip trails read the flight and never write it.
  //    Above a speed they appear, a hard pull thickens the trails, parked there are none; a rotor in the hover
  //    blows dust; an airship has none; the Effects button turns them off; a frame of them makes no garbage.
  console.log('\n-- effects: airflow lines, wingtip trails, prop swirl, rotor dust --');
  for (const id of ['sharky', 'gull', 'dart']) {
    const e = await page.evaluate(id => {
      const H = window.__hangar, o = {};
      H.fly(id); o.parked = (H.run(1.5, [[0, 1.5, ['ControlLeft']]]), H.fx());
      H.put([300, 90, -160], 0.3, 13); o.slow = (H.run(0.3, []), H.fx());
      H.put([300, 90, -160], 0.3, 40); o.cruise = (H.run(3, [[0, 3, ['ShiftLeft']]]), H.fx());
      H.run(0.45, [[0, 0.45, ['KeyA']]]); o.turn = (H.run(1.2, [[0, 1.2, ['KeyS', 'ShiftLeft']]]), H.fx());
      return o;
    }, id);
    console.log(`${id.padEnd(7)} parked: ${e.parked.streaks} lines, trail ${e.parked.trail} | 13 m/s: ${e.slow.streaks} lines | cruise: ${e.cruise.streaks} lines, trail ${e.cruise.trail} x ${e.cruise.width} m, ${e.cruise.g} g | hard turn: trail ${e.turn.trail} x ${e.turn.width} m, ${e.turn.g} g, ${e.turn.aoa} deg | ${e.cruise.tips} tips, ${e.cruise.props} prop${e.cruise.props === 1 ? '' : 's'}`);
    check(e.parked.streaks === 0 && e.parked.trail === 0, `${id}: nothing on the ground at a standstill`);
    check(e.slow.streaks === 0, `${id}: no airflow lines below the speed they start at`);
    check(e.cruise.streaks >= 8 && e.cruise.trail > 0 && e.cruise.tips >= 2, `${id}: lines and faint trails in cruise (${e.cruise.streaks} lines, trail ${e.cruise.trail})`);
    check(e.turn.trail > e.cruise.trail * 2 && e.turn.width > e.cruise.width * 1.5, `${id}: the trails grow in a hard pull (${e.cruise.trail} -> ${e.turn.trail})`);
  }
  const hv = await page.evaluate(() => { const H = window.__hangar, o = {}; H.fly('bumble'); o.bumble = (H.run(2.5, [[0, 2.5, ['ShiftLeft']]]), H.fx()); H.fly('puffin'); o.puffin = (H.run(2.5, [[0, 2.5, ['ShiftLeft']]]), H.fx()); H.fly('puff'); o.puff = (H.run(3, [[0, 3, ['KeyW']]]), H.fx()); return o; });
  console.log(`bumble hover: ${hv.bumble.dust} dust motes, ${hv.bumble.streaks} lines | puffin hover: ${hv.puffin.dust} dust | Puff (airship): ${hv.puff.streaks} lines, ${hv.puff.dust} dust, trail ${hv.puff.trail}`);
  check(hv.bumble.dust > 10 && hv.bumble.streaks === 0 && hv.puffin.dust > 10, 'a rotor in a low hover blows dust, and makes no airflow lines');
  check(!hv.puff.ready && hv.puff.streaks === 0 && hv.puff.dust === 0, 'an airship has no effects');
  const bd = await page.evaluate(() => { grid = PRESETS.gull.rows.map(r => r.split('')); YARD.flyDesign(); return window.__hangar.fx(); });
  check(bd.ready && bd.tips === 4 && bd.props === 1, `a design flown from the builder finds its own wingtips, tailplane tips and prop (${bd.tips} tips, ${bd.props} prop)`);
  const tg = await page.evaluate(() => { const H = window.__hangar, b = document.getElementById('fxBtn'); b.click(); const off = {on: H.fx().on, pressed: b.getAttribute('aria-pressed'), box: document.getElementById('fxChk').checked}; b.click(); return {off, on: H.fx().on}; });
  check(!tg.off.on && tg.off.pressed === 'false' && !tg.off.box && tg.on, 'the Effects button (and the box in Controls) turns them off and on');
  const al = await page.evaluate(() => {
    const H = window.__hangar; H.fly('sharky'); H.put([300, 90, -160], 0.3, 40); H.run(2, [[0, 2, ['ShiftLeft']]]);
    H.fxBench(3000);   // warmed up: the first frames run before the engine has compiled the code, and box their numbers
    let grew = 1e12, ms = 0;
    for (let k = 0; k < 3; k++) { gc(); const m0 = performance.memory.usedJSHeapSize; ms = H.fxBench(3000); grew = Math.min(grew, performance.memory.usedJSHeapSize - m0); }
    gc(); const e0 = performance.memory.usedJSHeapSize; for (let i = 0; i < 3000; i++) performance.now(); const empty = performance.memory.usedJSHeapSize - e0;
    return {grew, empty, us: Math.round(ms / 3000 * 1000)};
  });
  console.log(`3000 effect frames (a step and a layout each), warmed up: heap grew ${al.grew} bytes (an empty loop of 3000: ${al.empty}), ${al.us} us a frame (software rendering; not the game's speed)`);
  check(al.grew < 200 * 1024, `an effects frame allocates nothing to speak of (${al.grew} bytes over 3000 frames, ${Math.round(al.grew / 3000)} a frame)`);

  check(errs.length === 0, `no console errors${errs.length ? ': ' + errs.slice(0, 3).join(' | ') : ''}`);
  console.log(`\n${fail.length ? `${fail.length} FAILED` : 'ALL PASSED'}`);
  await browser.close(); process.exit(fail.length ? 1 : 0);
})();
