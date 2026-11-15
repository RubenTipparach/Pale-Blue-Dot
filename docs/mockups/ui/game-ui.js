// game-ui.js: the Stone and Timber UI kit as a working mock of the game's screens
// (openspec/changes/stone-and-wood-ui). Used by every tab of docs/mockups/ui.html and by the Play
// demo (ui/play.js). Three looks of the same kit, one font, one inventory model, one set of menus.
'use strict';
(function () {
  const root = document.documentElement;

  // ------------------------------------------------------------------ looks
  // Every kit image is a custom property; a look sets them all. Drawn: ui/ (ui_art.py). Rendered:
  // ui/rendered/ (ui_panels_blend.py, pixelated). Smooth: ui/rendered/hires/ (the 8x Cycles render,
  // so every 9-slice margin is the drawn one times --m = 8, and --ir turns nearest sampling off).
  const STONE = ['rock', 'frame', 'lintel', 'slot-idle', 'slot-hover', 'slot-selected', 'slot-disabled', 'slot-uncommon', 'slot-rare', 'slot-epic', 'slot-legendary',
    'button-idle', 'button-hover', 'button-pressed', 'button-disabled', 'button-focus', 'tab-open', 'tab-shut', 'scroll-track', 'scroll-thumb', 'slider-track', 'slider-knob',
    'check-off', 'check-on', 'tooltip', 'bar', 'fill-health', 'fill-stamina', 'fill-breath', 'fill-progress', 'compass', 'compass-pointer', 'compass-town', 'map-frame',
    'portrait', 'emblem', 'icon-close', 'icon-back', 'icon-gear', 'icon-map', 'icon-pack'];
  const OAK = ['frame', 'button-idle', 'button-hover', 'button-pressed', 'button-disabled', 'button-focus', 'tab-open', 'tab-shut', 'scroll-thumb', 'slider-knob', 'tooltip',
    'portrait', 'icon-close', 'icon-back', 'icon-gear', 'icon-map', 'icon-pack'];
  const LOOKS = {
    drawn: { dir: 'ui/', oak: 'ui/oak/', m: 1, ir: 'pixelated' },
    rendered: { dir: 'ui/rendered/', oak: 'ui/rendered/oak/', m: 1, ir: 'pixelated' },
    smooth: { dir: 'ui/rendered/hires/', oak: 'ui/rendered/hires/oak/', m: 8, ir: 'auto' },
  };
  let ROCKS = {}; // panel name -> [w, h] native px of its baked rock frame (rendered/slices.json)
  let css = '';
  for (const [k, L] of Object.entries(LOOKS)) {
    const sel = k === 'drawn' ? ':root, [data-look="drawn"]' : `:root.look-${k}, [data-look="${k}"]`;
    css += `${sel} { --m: ${L.m}; --ir: ${L.ir};\n` + STONE.map((n) => `  --i-${n}: url(${L.dir}${n}.png);`).join('\n') + '\n' +
      OAK.map((n) => `  --o-${n}: url(${L.oak}${n}.png);`).join('\n') + '\n}\n';
  }
  document.getElementById('lookvars').textContent = css;
  let LOOK = 'smooth'; // the owner's decision, 2026-11-14: "Smooth is the best"
  try { const v = localStorage.getItem('pbd-ui-look'); if (LOOKS[v]) LOOK = v; } catch (e) { /* storage blocked: default look */ }
  function setLook(l, remember) {
    if (!LOOKS[l]) return;
    LOOK = l;
    root.classList.remove('look-rendered', 'look-smooth');
    if (l !== 'drawn') root.classList.add('look-' + l);
    document.querySelectorAll('#lookSeg button').forEach((b) => b.setAttribute('aria-pressed', String(b.dataset.look === l)));
    if (remember) { try { localStorage.setItem('pbd-ui-look', l); } catch (e) { /* not remembered */ } }
    rockify(document);
    document.dispatchEvent(new CustomEvent('pbd-look', { detail: l }));
    if (remember) {
      const t = document.getElementById('lookToast');
      if (t) { t.textContent = 'Look: ' + { drawn: 'Drawn', rendered: 'Rendered', smooth: 'Smooth' }[l]; t.classList.add('on'); clearTimeout(t._h); t._h = setTimeout(() => t.classList.remove('on'), 1600); }
    }
  }
  setLook(LOOK, false);
  document.getElementById('lookSeg').addEventListener('click', (e) => { const b = e.target.closest('button'); if (b) setLook(b.dataset.look, true); });

  // ------------------------------------------------------------------ the kit's font
  let FONT = null;
  const PT = new Map();
  function ptSvg(text) {
    const t = String(text).toUpperCase();
    if (PT.has(t)) return PT.get(t);
    const G = FONT.glyphs; let x = 0, d = '';
    for (const ch of t) {
      const g = G[ch] || G['?'];
      g.rows.forEach((row, y) => {
        let i = 0;
        while (i < row.length) {
          if (row[i] !== '#') { i++; continue; }
          let j = i; while (j < row.length && row[j] === '#') j++;
          d += `M${x + i} ${y}h${j - i}v1h${i - j}z`; i = j;
        }
      });
      x += g.w + FONT.spacing;
    }
    const w = Math.max(1, x - FONT.spacing);
    const svg = `<svg viewBox="0 0 ${w} 8" width="${w}" height="8" shape-rendering="crispEdges" aria-hidden="true"><path class="sh" transform="translate(0 1)" d="${d}"/><path class="fg" d="${d}"/></svg>`;
    PT.set(t, svg);
    return svg;
  }
  // A label in the kit's font: whole pixels at the UI scale, coloured by CSS (currentColor).
  function P(text, cls) {
    const s = document.createElement('span');
    s.className = 'pt' + (cls ? ' ' + cls : '');
    s.setAttribute('role', 'img');
    s.setAttribute('aria-label', text);
    if (FONT) s.innerHTML = ptSvg(text);
    else { s.textContent = text; s.style.font = 'calc(9px * var(--s))/1 var(--body)'; }
    return s;
  }

  // ------------------------------------------------------------------ DOM helpers
  function h(tag, attrs, ...kids) {
    const e = document.createElement(tag);
    for (const [k, v] of Object.entries(attrs || {})) {
      if (v == null || v === false) continue;
      if (k === 'style') e.style.cssText = v; else if (k === 'class') e.className = v;
      else if (k.startsWith('on')) e.addEventListener(k.slice(2), v); else e.setAttribute(k, v === true ? '' : v);
    }
    for (const k of kids.flat()) if (k != null && k !== false) e.append(k);
    return e;
  }
  const ki = (piece, w, hh, extra) => h('i', { class: 'ki', style: `--k:var(--${piece});width:calc(${w}px * var(--s));height:calc(${hh}px * var(--s));${extra || ''}` });
  function btn(label, o = {}) {
    const b = h('button', { class: 'btn' + (o.oak ? ' oak' : '') + (o.cls ? ' ' + o.cls : ''), type: 'button', 'aria-label': label, disabled: o.disabled, 'aria-pressed': o.pressed == null ? null : String(o.pressed), 'data-act': o.act || null }, P(label));
    if (o.on) b.addEventListener('click', (e) => { e.stopPropagation(); o.on(e); });
    if (o.style) b.style.cssText += o.style;
    return b;
  }
  const lintel = (title, big) => h('div', { class: 'lintel-row' }, h('div', { class: 'p-lintel' }, P(title, big ? 'big' : '')));
  // The rough-rock frame of an in-play panel. In the rendered and smooth looks it is the approved
  // base rock rendered at exactly this panel's native size (rendered/slices.json "rock_panels"), so
  // nothing in its rim or lip shadow tiles; in the drawn look it is the drawn slab as a 9-slice.
  // (ROCKS is declared at the top: setLook rockifies the page before this point)
  function rockPanel(name, attrs, ...kids) {
    const [w, hh] = ROCKS[name] || [200, 120];
    const a = Object.assign({}, attrs, { class: 'nine p-rock face', 'data-rock': name });
    a.style = `width:calc(${w}px * var(--s));height:calc(${hh}px * var(--s));` + (attrs.style || '');
    return h('div', a, ...kids);
  }
  function rockify(scope) {
    for (const el of (scope || document).querySelectorAll('[data-rock]')) {
      const [w, hh] = ROCKS[el.dataset.rock] || [0, 0];
      const lk = (el.closest('[data-look]') || {}).dataset?.look || LOOK;
      if (lk === 'drawn' || !w) { el.style.borderImageSource = ''; el.style.borderColor = ''; el.style.background = ''; continue; }
      el.style.borderImageSource = 'none'; el.style.borderColor = 'transparent';
      el.style.background = `url(ui/rendered/${lk === 'smooth' ? 'hires/' : ''}rock/rock-${w}x${hh}.png) 0 0 / 100% 100% no-repeat border-box`;
    }
  }

  // ------------------------------------------------------------------ items
  const T = (c, r, name, v) => ({ tile: [c, r], name, v });
  const I = (src, name, v) => ({ img: src, name, v });
  const G = (n, name, v) => I('ui/game/' + n + '.png', name, v);
  const M = (n, name, v) => I('ui/items/' + n + '.png', name, v);
  const ITEMS = {
    grass: T(0, 4, 'Grass', 1), dirt: T(1, 5, 'Dirt', 1), cobble: T(3, 4, 'Cobblestone', 1), sand: T(8, 0, 'Sand', 1), planks: T(1, 7, 'Planks', 2),
    log: T(2, 5, 'Log', 2), brick: T(1, 11, 'Brick', 3), snow: T(4, 4, 'Snow', 1), stone: T(1, 0, 'Stone', 1), ore: T(4, 1, 'Iron ore', 4), coal: T(0, 0, 'Coal', 3),
    hand: G('hand', 'Hand', 0), pickaxe: G('pickaxe', 'Pickaxe', 30), axe: G('axe', 'Axe', 24), shovel: G('shovel', 'Shovel', 20), rod: G('rod', 'Rod', 18),
    torch: G('torch', 'Torch', 2), lantern: G('lantern_post', 'Lantern, post', 16), wall: G('lantern_wall', 'Lantern, wall', 12), candle: G('candle', 'Candle', 3),
    brazier: G('brazier', 'Brazier', 20), perch: G('perch', 'Perch', 4), minnow: G('minnow', 'Minnow', 2), eel: G('eel', 'Eel', 6), silverfin: G('silverfin', 'Silverfin', 8),
    cooked: G('perch', 'Grilled perch', 7), ingot: M('ingot', 'Iron ingot', 10), grain: M('grain', 'Grain', 1), flour: M('flour', 'Flour', 3), bread: M('bread', 'Bread', 6),
  };
  const BLOCKS = new Set(['grass', 'dirt', 'cobble', 'sand', 'planks', 'log', 'brick', 'snow', 'stone']);
  function icon(id) {
    const it = ITEMS[id]; if (!it) return null;
    const ic = h('div', { class: 'ic' + (it.tile ? ' tile' : ''), role: 'img', 'aria-label': it.name });
    if (it.tile) ic.style.backgroundPosition = `calc(${-it.tile[0] * 16}px * var(--s)) calc(${-it.tile[1] * 16}px * var(--s))`;
    if (it.tile) { ic.style.setProperty('--tx', it.tile[0]); ic.style.setProperty('--ty', it.tile[1]); }
    else ic.style.background = `url(${it.img}) 0 0 / 100% 100%`;
    return ic;
  }
  function slotEl(cell, cls) {
    const s = h('div', { class: 'slot' + (cls ? ' ' + cls : '') });
    if (cell && cell[0]) {
      s.append(icon(cell[0]));
      if (cell[1] > 1) s.append(h('span', { class: 'n' }, P(String(cell[1]))));
    }
    return s;
  }
  // The pack and hotbar: [id, count] or null. One model for the static tabs and the demo.
  function newInv() {
    const pack = Array(30).fill(null), hot = Array(10).fill(null);
    [[0, 'planks', 40], [1, 'log', 12], [2, 'brick', 24], [3, 'ore', 7], [4, 'coal', 5], [5, 'candle', 6], [6, 'brazier', 1], [7, 'grain', 6], [8, 'flour', 2],
      [10, 'perch', 3], [11, 'minnow', 7], [12, 'eel', 1], [13, 'silverfin', 2], [20, 'shovel', 1], [21, 'wall', 2], [22, 'ingot', 1]].forEach(([i, id, n]) => { pack[i] = [id, n]; });
    [['grass', 64], ['dirt', 64], ['cobble', 48], ['sand', 32], ['torch', 16], ['lantern', 4], ['pickaxe', 1], ['axe', 1], ['rod', 1], null].forEach((c, i) => { hot[i] = c; });
    return {
      pack, hot, sel: 0,
      all() { return this.pack.concat(this.hot); },
      count(id) { return this.all().reduce((a, c) => a + (c && c[0] === id ? c[1] : 0), 0); },
      add(id, n) {
        for (const arr of [this.hot, this.pack]) for (const c of arr) if (c && c[0] === id && c[1] < 64) { const k = Math.min(n, 64 - c[1]); c[1] += k; n -= k; if (!n) return 0; }
        for (const arr of [this.pack, this.hot]) for (let i = 0; i < arr.length; i++) if (!arr[i]) { const k = Math.min(n, 64); arr[i] = [id, k]; n -= k; if (!n) return 0; }
        return n;
      },
      take(id, n) {
        if (this.count(id) < n) return false;
        for (const arr of [this.pack, this.hot]) for (let i = 0; i < arr.length; i++) { const c = arr[i]; if (c && c[0] === id && n) { const k = Math.min(n, c[1]); c[1] -= k; n -= k; if (!c[1]) arr[i] = null; } }
        return true;
      },
      held() { return this.hot[this.sel]; },
    };
  }

  // ------------------------------------------------------------------ settings and bindings
  // The rows are the game's (crates/pbd-app/src/controls.rs BINDINGS, regrouped by context as the
  // owner asked: "organize the input menu better with tabs"). `p: true` marks a proposed row the
  // game does not have.
  const BIND_TABS = [
    ['foot', 'On foot', [['Move', ['W', 'A', 'S', 'D']], ['Jump / swim up', ['Space']], ['Sprint', ['Shift']], ['Walk or fly', ['R']], ['Back to the spawn', ['H']], ['Crouch / sink', ['Ctrl'], true]]],
    ['hands', 'Hands', [['Dig / use tool', ['LMB']], ['Place block', ['RMB']], ['Interact', ['E']], ['Select a slot', ['1', '0']], ['Next slot', ['Wheel']], ['Tools wheel (hold)', ['G']], ['Drop stack', ['Q'], true]]],
    ['fly', 'Flying', [['Move', ['W', 'A', 'S', 'D']], ['Up / down', ['Space', 'Ctrl']], ['Roll', ['Q', 'E']], ['Cruise', ['Shift']], ['Dampeners', ['X']], ['Brake', ['B']],
      ['Kestrel: pitch', ['W', 'S']], ['Kestrel: roll', ['A', 'D']], ['Kestrel: yaw', ['Q', 'E']], ['Kestrel: power', ['Space', 'Ctrl']], ['Kestrel: nacelles', ['Z', 'C']], ['Kestrel: assist', ['X']], ['Kestrel: wheel brake', ['B']]]],
    ['ships', 'Vehicles', [['Board, step off', ['F']], ['Seat or chase view', ['V']], ['Anchor, weigh', ['T']], ['Tern: tiller', ['A', 'D']], ['Tern: sheet in, ease', ['W', 'S']], ['Tern: hike', ['Q', 'E']], ['Bail', ['B']],
      ['Loon: paddle', ['W', 'S']], ['Loon: steer', ['A', 'D']], ['Loon: blade rudder', ['Q', 'E']], ['Rowboat: row', ['W', 'S']], ['Rowboat: steer', ['A', 'D']], ['Rowboat: trail an oar', ['Q', 'E']],
      ['Cog: tiller', ['A', 'D']], ['Cog: brace the yard', ['W', 'S']]]],
    ['view', 'Menus', [['The pack', ['I']], ['The map', ['M']], ['The field guide', ['J']], ['Menu', ['Esc']], ['Screenshot', ['F12']], ['Frame graph', ['F3']], ['Cycle the weather', ['P']], ['Noon or midnight', ['L']]]],
  ];
  const freshBinds = () => Object.fromEntries(BIND_TABS.map(([id, , rows]) => [id, rows.map((r) => r[1].slice())]));
  const keyName = (e) => {
    if (e.code.startsWith('Key')) return e.code.slice(3);
    if (e.code.startsWith('Digit')) return e.code.slice(5);
    return ({ Space: 'Space', ShiftLeft: 'Shift', ShiftRight: 'Shift', ControlLeft: 'Ctrl', ControlRight: 'Ctrl', AltLeft: 'Alt', Tab: 'Tab', Enter: 'Enter', Backquote: '`', Minus: '-', Equal: '=' })[e.code] || e.key.toUpperCase().slice(0, 5);
  };
  const SCALES = [50, 67, 75, 85, 100];

  // ------------------------------------------------------------------ recipes
  const STATIONS = {
    forge: { title: 'Forge', verb: 'Smelt', fuel: 'coal', recipes: [['ingot', 1, [['ore', 2]]], ['lantern', 1, [['ingot', 1], ['torch', 1]]], ['pickaxe', 1, [['ingot', 2], ['log', 1]]]] },
    hearth: { title: 'Hearth and pot', verb: 'Cook', fuel: 'log', recipes: [['bread', 1, [['flour', 2]]], ['cooked', 1, [['perch', 1]]]] },
    bench: { title: 'Bench', verb: 'Make', recipes: [['planks', 4, [['log', 1]]], ['torch', 4, [['planks', 1], ['coal', 1]]], ['wall', 1, [['ingot', 1], ['candle', 1]]]] },
    mill: { title: 'Water mill', verb: 'Grind', recipes: [['flour', 1, [['grain', 2]]]] },
    press: { title: 'Brick press', verb: 'Press', recipes: [['brick', 2, [['cobble', 3]]]] },
  };
  // Made in the pack, by hand, with no station (owner, 2026-11-14: "the pack should also include some
  // kind of basic crafting system, with a drop down list, and a button to look at crafting
  // trees/recipes"). Mockup recipes: the game has no crafting yet.
  const HAND = { title: 'By hand', verb: 'Craft', recipes: [['planks', 4, [['log', 1]]], ['torch', 4, [['planks', 1], ['coal', 1]]], ['shovel', 1, [['planks', 2], ['cobble', 1]]],
    ['axe', 1, [['planks', 2], ['cobble', 3]]], ['rod', 1, [['planks', 3]]]] };
  // Where each item is made: the first place that makes it (the tree follows these).
  const MADE = (() => { const m = {}; for (const [k, S] of [['hand', HAND], ...Object.entries(STATIONS)]) for (const [out, n, ins] of S.recipes) if (!m[out]) m[out] = { at: k, n, ins, fuel: S.fuel }; return m; })();
  const madeAt = (k) => (k === 'hand' ? HAND : STATIONS[k]).title;

  // ------------------------------------------------------------------ compass
  function compass(el, heading, towns) {
    let inner = el.querySelector('.inner');
    if (!inner) { inner = h('div', { class: 'inner' }); el.append(inner, h('i', { class: 'ki ptr' })); }
    inner.innerHTML = '';
    const s = parseFloat(getComputedStyle(el).getPropertyValue('--s')) || 2;
    const W = el.clientWidth || 560, names = ['N', 'NE', 'E', 'SE', 'S', 'SW', 'W', 'NW'], pxPerDeg = W / 200;
    for (let a = -180; a <= 540; a += 15) {
      const off = ((a - heading + 540) % 360 + 360) % 360 - 180;
      if (Math.abs(off) > 95 || a < 0 || a >= 360) continue;
      const x = W / 2 + off * pxPerDeg + 6 * s;
      if (a % 45 === 0) {
        const nm = names[a / 45], m = h('div', { class: 'mark' + (nm === 'N' ? ' n' : '') }, P(nm));
        inner.append(m);
        const w = m.offsetWidth || (nm.length * 6 - 1) * s;
        m.style.left = Math.round((x - w / 2) / s) * s + 'px';
      } else inner.append(h('div', { class: 'tick', style: `left:${Math.round(x / s) * s}px` }));
    }
    for (const t of towns || []) {
      const off = ((t - heading + 540) % 360 + 360) % 360 - 180;
      if (Math.abs(off) > 95) continue;
      inner.append(h('i', { class: 'ki town', style: `left:${Math.round((W / 2 + off * pxPerDeg + 6 * s - 2.5 * s) / s) * s}px` }));
    }
  }

  // ------------------------------------------------------------------ the UI over one game screen
  const MAP_OVERLAYS = ['The planet', 'Biomes', 'Wind', 'Jet', 'Currents', 'Cloud', 'Rain', 'Humidity', 'Sunlight', 'Heat', 'Night side', 'Clouds'];
  const SAVES = [['Hillside', 'Day 3, 41 min'], ['Dunmouth coast', 'Day 12, 6 h'], ['Tahal crossing', 'Day 1, 9 min']];
  const TALK = {
    start: ['Ore for the gate hinges. Bring six iron and the gate is hung by nightfall.', [['I will', 'accept'], ['Trade', '#trade'], ['What turns that wheel?', 'wheel'], ['Later', '#close']]],
    accept: ['Good. The forge is past the bench; ore smelts with a little coal. Work the bellows and it runs hot.', [['Trade', '#trade'], ['Goodbye', '#close']]],
    wheel: ['The stream. It turns the mill stones day and night; bring grain and it grinds you flour.', [['Back', 'start'], ['Goodbye', '#close']]],
  };
  const STOCK = () => [['pickaxe', 1], ['axe', 1], ['lantern', 2], ['brazier', 1], ['torch', 16], ['bread', 3], ['coal', 8], null];

  class GameUI {
    constructor(host, o = {}) {
      this.host = host; this.o = o; this.box = o.box || host; this.box.__ui = this;
      this.inv = o.inv || newInv();
      this.st = {
        screen: o.screen || 'hud', back: null, settingsTab: o.settingsTab || 'picture', inputTab: 'foot',
        set: { bloom: true, frame: 60, scale: 85, rain: false, step: false }, binds: freshBinds(), rebind: null, changed: new Set(),
        drawer: o.drawer ?? false, overlay: 1, talk: 'start', station: o.station || 'forge', recipe: 0, run: null, out: null, craft: 0, craftOpen: false, tree: 'lantern',
        offer: [], ask: [], stock: STOCK(), chest: [['cobble', 20], ['ore', 4], ['grain', 8], ['coal', 6], null, null, null, null, null, null, null, null, null, null, null, null],
        heading: o.heading || 0, towns: o.towns || [-22, 38], townName: o.town || '', prompt: '', toast: '', quit: false, saveAsk: -1, load: 0, loadRun: false,
      };
      this.held = null;
      this.box.addEventListener('keydown', (e) => this.key(e));
      this.box.addEventListener('wheel', (e) => this.wheel(e), { passive: false });
      this.host.addEventListener('contextmenu', (e) => { if (this.st.screen !== 'hud') e.preventDefault(); });
      this.host.addEventListener('mousemove', (e) => this.moveHeld(e));
      this.render();
    }
    get menuOpen() { return this.st.screen !== 'hud'; }
    go(screen, extra) {
      const prev = this.st.screen;
      Object.assign(this.st, { screen, quit: false, rebind: null }, extra || {});
      if (screen === 'hud' && this.held) { this.inv.add(this.held[0], this.held[1]); this.held = null; }
      this.render();
      if (this.o.onScreen) this.o.onScreen(screen, prev);
    }
    // keys: only while this game screen has focus (the page still scrolls otherwise)
    key(e) {
      const st = this.st;
      if (st.rebind) {
        e.preventDefault(); e.stopPropagation();
        if (e.code !== 'Escape') { const [tab, i] = st.rebind; st.binds[tab][i] = [keyName(e)]; st.changed.add(tab + ':' + i); }
        st.rebind = null; this.render(); return true;
      }
      const k = e.code;
      if (k === 'Escape') { e.preventDefault(); this.back(); return true; }
      if ((k === 'KeyI' || k === 'Tab') && ['hud', 'pack'].includes(st.screen)) { e.preventDefault(); this.go(st.screen === 'pack' ? 'hud' : 'pack'); return true; }
      if (k === 'KeyM' && ['hud', 'map'].includes(st.screen)) { e.preventDefault(); this.go(st.screen === 'map' ? 'hud' : 'map'); return true; }
      if (k.startsWith('Digit') && ['hud', 'pack'].includes(st.screen)) { this.inv.sel = (parseInt(k.slice(5), 10) + 9) % 10; this.render(); return true; }
      // E closes whatever is open, a step back as Esc does: in fullscreen the browser takes Esc to leave
      // fullscreen (owner, 2026-11-14: "esc also closes out of big screen ... add e as exit instead").
      // On the HUD E is still "use".
      if (k === 'KeyE' && !['hud', 'title', 'loading'].includes(st.screen) && !(e.target && /^(INPUT|TEXTAREA)$/.test(e.target.tagName))) { e.preventDefault(); this.back(); return true; }
      return false;
    }
    wheel(e) {
      if (!['hud', 'pack'].includes(this.st.screen)) return;
      if (this.o.mode === 'play' && !this.o.captured()) return;
      e.preventDefault();
      this.inv.sel = (this.inv.sel + (e.deltaY > 0 ? 1 : 9)) % 10; this.render();
    }
    setHeading(deg) {
      this.st.heading = deg;
      const c = this.host.querySelector('.hud-compass'); if (c) compass(c, deg, this.st.towns);
    }
    setPrompt(text) {
      if (this.st.prompt === text) return;
      this.st.prompt = text;
      const p = this.host.querySelector('.prompt'); if (p) p.remove();
      if (text && this.st.screen === 'hud') this.host.append(h('div', { class: 'prompt nine p-tip face' }, P(text)));
    }
    toast(text) {
      this.st.toast = text; clearTimeout(this._tt);
      this.render(); this._tt = setTimeout(() => { this.st.toast = ''; this.render(); }, 1600);
    }
    moveHeld(e) {
      const hd = this.host.querySelector('.held'); if (!hd) return;
      const r = this.host.getBoundingClientRect(), f = r.width / (this.host.offsetWidth || r.width);
      hd.style.left = ((e.clientX - r.left) / f + 6) + 'px'; hd.style.top = ((e.clientY - r.top) / f + 6) + 'px';
    }

    // ---------------------------------------------------------------- rendering
    // What Esc does, for the visible close button too (the owner, on a phone, 2026-11-14: "There's no
    // button to close or exit use modal").
    back() {
      const st = this.st;
      if (st.screen === 'pack' && st.craftOpen) { st.craftOpen = false; this.render(); return; }
      const back = { recipes: 'pack', hud: 'pause', pause: 'hud', settings: this.o.title ? 'title' : 'pause', saves: this.o.title ? 'title' : 'pause', title: 'title' }[st.screen] || 'hud';
      if (st.screen === 'settings' && st.back) this.go(st.back); else this.go(back);
    }
    render() {
      const st = this.st, u = this.host; u.innerHTML = '';
      const sc = st.screen;
      const timber = ['pause', 'settings', 'saves', 'title'].includes(sc);
      if (timber) u.append(h('div', { class: 'dimmer' }));
      if (sc === 'hud') u.append(h('div', { class: 'cross' }));
      if (!['title', 'loading', 'map'].includes(sc)) {
        const c = h('div', { class: 'hud-compass nine p-compass' }, h('div', { class: 'inner' }), h('i', { class: 'ki ptr' }));
        u.append(c);
        if (st.townName) u.append(h('div', { class: 'hud-name' }, P(st.townName)));
        requestAnimationFrame(() => compass(c, st.heading, st.towns));
        u.append(this.hotbar());
      }
      const panel = ({ pause: () => this.pause(), settings: () => this.settings(), saves: () => this.saves(), title: () => this.title(), pack: () => this.pack(), map: () => this.map(),
        station: () => this.station(), chest: () => this.chestPanel(), talk: () => this.talk(), trade: () => this.trade(), loading: () => this.loading(), recipes: () => this.recipes() })[sc];
      if (panel) {
        const el = panel(); u.append(el);
        // Every open panel can be shut without a keyboard (the owner, on a phone, 2026-11-14:
        // "There's no button to close or exit use modal"): an X in the panel's own top-right corner,
        // unless the panel already carries one. It does what Esc does.
        const has = el.querySelector('[aria-label^="close"], [aria-label^="Close"]');
        // (touch screens only: on a desktop Esc shuts it, and the game itself has no X; owner, 2026-11-14)
        if (COARSE && !['title', 'loading'].includes(sc) && !has) {
          if (getComputedStyle(el).position === 'static') el.style.position = 'relative';
          el.append(h('button', { type: 'button', class: 'ui-x', 'aria-label': 'Close', title: 'Close (Esc)',
            style: 'position:absolute;top:calc(-6px*var(--s));right:calc(-6px*var(--s));z-index:40;width:calc(20px*var(--s));height:calc(20px*var(--s));min-width:36px;min-height:36px;padding:0;border:2px solid #92a3b2;border-radius:6px;background:#011c34;color:#eab798;font:700 calc(14px*var(--s))/1 VT323,monospace;box-shadow:0 2px 6px #000a;cursor:pointer;touch-action:manipulation',
            onclick: (e) => { e.stopPropagation(); this.back(); } }, '\u2715'));
        }
      }
      if (st.prompt && sc === 'hud') u.append(h('div', { class: 'prompt nine p-tip face' }, P(st.prompt)));
      if (st.toast) u.append(h('div', { class: 'toast nine p-tip face' }, P(st.toast)));
      if (this.held) { const hd = h('div', { class: 'held' }, slotEl(this.held, '')); hd.firstChild.style.background = 'none'; u.append(hd); }
      rockify(u);
      if (this.o.onRender) this.o.onRender(sc);
    }
    hotbar(cls) {
      const d = h('div', { class: 'slots' + (cls === false ? '' : ' hud-hotbar') });
      d.append(slotEl(['hand', 1], ''));
      d.firstChild.style.marginRight = 'calc(4px * var(--s))';
      this.inv.hot.forEach((c, i) => {
        const s = slotEl(c, i === this.inv.sel ? 'sel' : '');
        if (this.st.screen === 'pack') { s.classList.add('click'); this.slotEvents(s, this.inv.hot, i); }
        else if (this.st.screen === 'hud') s.addEventListener('click', (e) => { e.stopPropagation(); this.inv.sel = i; this.render(); });
        d.append(s);
      });
      return d;
    }
    slotEvents(s, arr, i) {
      s.tabIndex = 0;
      s.addEventListener('click', (e) => {
        e.stopPropagation(); this.clickSlot(arr, i, false);
        if (COARSE) { const c = (this.held || arr[i]); if (c) { this.toast(`${ITEMS[c[0]].name} \u00b7 ${c[1]}`); } }
      });
      s.addEventListener('contextmenu', (e) => { e.preventDefault(); e.stopPropagation(); this.clickSlot(arr, i, true); });
      s.addEventListener('keydown', (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); this.clickSlot(arr, i, false); } });
      s.addEventListener('mouseenter', () => {
        const c = arr[i]; if (!c || this.held) return;
        const tip = h('div', { class: 'tipfloat nine p-tip face' }, P(`${ITEMS[c[0]].name} · ${c[1]}`));
        const r = s.getBoundingClientRect(), hr = this.host.getBoundingClientRect();
        const f = hr.width / (this.host.offsetWidth || hr.width);
        tip.style.left = ((r.right - hr.left) / f + 4) + 'px'; tip.style.top = ((r.top - hr.top) / f - 4) + 'px';
        tip.dataset.tip = '1'; this.host.append(tip);
      });
      // touch: a long press is the right click (take half); a tap shows the tooltip as well
      let lp = null;
      s.addEventListener('pointerdown', (e) => { if (e.pointerType !== 'touch') return; lp = setTimeout(() => { lp = 'done'; this.clickSlot(arr, i, true); }, 480); });
      s.addEventListener('pointerup', (e) => { if (e.pointerType !== 'touch') return; if (lp && lp !== 'done') clearTimeout(lp); });
      s.addEventListener('click', (e) => { if (lp === 'done') { lp = null; e.stopImmediatePropagation(); } }, true);
      s.addEventListener('mouseleave', () => this.host.querySelectorAll('[data-tip]').forEach((t) => t.remove()));
    }
    clickSlot(arr, i, right) {
      const c = arr[i];
      if (!this.held) {
        if (!c) return;
        if (right && c[1] > 1) { const k = Math.ceil(c[1] / 2); this.held = [c[0], k]; c[1] -= k; }
        else { this.held = c; arr[i] = null; }
      } else if (right) {
        if (!c) { arr[i] = [this.held[0], 1]; this.held[1]--; } else if (c[0] === this.held[0] && c[1] < 64) { c[1]++; this.held[1]--; }
        if (!this.held[1]) this.held = null;
      } else if (c && c[0] === this.held[0]) {
        const k = Math.min(64 - c[1], this.held[1]); c[1] += k; this.held[1] -= k; if (!this.held[1]) this.held = null;
      } else { arr[i] = this.held; this.held = c; }
      this.render();
    }

    // ---- timber (out of play): pause, settings, saves, title
    pause() {
      const st = this.st;
      const body = st.quit
        ? h('div', { style: 'display:grid;gap:calc(3px*var(--s));justify-items:center' }, h('div', { class: 'face', style: 'text-align:center;max-width:calc(80px*var(--s))' }, 'Quit to the title? The world is saved as you play.'),
          h('div', { style: 'display:flex;gap:calc(3px*var(--s))' }, btn('Quit', { oak: true, act: 'quit-yes', on: () => this.go('title') }), btn('Stay', { oak: true, cls: 'is-focus', act: 'quit-no', on: () => { st.quit = false; this.render(); } })))
        : h('div', { style: 'display:grid;gap:calc(3px*var(--s))' },
          btn('Resume', { oak: true, cls: 'is-focus', act: 'resume', on: () => this.go('hud') }), btn('Saves', { oak: true, act: 'saves', on: () => this.go('saves') }),
          btn('Settings', { oak: true, act: 'settings', on: () => this.go('settings', { back: 'pause' }) }), btn('Quit', { oak: true, act: 'quit', on: () => { st.quit = true; this.render(); } }));
      return h('div', { class: 'centre oakset' }, h('div', { class: 'nine p-oak face', style: 'width:calc(100px*var(--s))', 'data-panel': 'pause' }, lintel('Paused'), body));
    }
    title() {
      const st = this.st;
      const body = st.quit
        ? h('div', { style: 'display:grid;gap:calc(3px*var(--s));justify-items:center' }, h('div', { class: 'face' }, 'The game closes here.'), btn('Back', { oak: true, on: () => { st.quit = false; this.render(); } }))
        : h('div', { style: 'display:grid;gap:calc(3px*var(--s))' },
          btn('Continue', { oak: true, cls: 'is-focus', act: 'continue', on: () => (this.o.onPlay ? this.o.onPlay() : this.go('loading')) }),
          h('div', { class: 'dim', style: 'text-align:center;margin-top:calc(-2px*var(--s))' }, SAVES[0][0] + ', ' + SAVES[0][1]),
          btn('New world', { oak: true, act: 'new', on: () => (this.o.onPlay ? this.o.onPlay() : this.go('loading')) }), btn('Load', { oak: true, act: 'load', on: () => this.go('saves') }),
          btn('Settings', { oak: true, act: 'settings', on: () => this.go('settings', { back: 'title' }) }), btn('Quit', { oak: true, act: 'quit', on: () => { st.quit = true; this.render(); } }));
      return h('div', { class: 'centre oakset', style: 'top:56%' }, h('div', { class: 'nine p-oak face', style: 'width:calc(120px*var(--s))', 'data-panel': 'title' }, lintel('Pale Blue Dot', true), h('div', { style: 'height:calc(8px*var(--s))' }), body));
    }
    saves() {
      const st = this.st;
      const rows = SAVES.map(([n, d], i) => h('div', { style: 'display:grid;grid-template-columns:1fr auto auto;gap:calc(3px*var(--s));align-items:center;min-height:calc(24px*var(--s))' },
        h('div', {}, h('div', { class: 'carved', style: '--pt-shadow:var(--ink)' }, P(n)), h('div', { class: 'dim' }, d)),
        st.saveAsk === i ? btn('Delete forever', { oak: true, on: () => { SAVES.splice(i, 1); st.saveAsk = -1; this.render(); } }) : btn(i === 0 && !this.o.title ? 'Continue' : 'Load', { oak: true, act: 'load-' + i, on: () => (this.o.onPlay ? this.o.onPlay() : this.go(this.o.title ? 'loading' : 'hud')) }),
        st.saveAsk === i ? btn('Cancel', { oak: true, on: () => { st.saveAsk = -1; this.render(); } }) : btn('Delete', { oak: true, on: () => { st.saveAsk = i; this.render(); } })));
      return h('div', { class: 'centre oakset' }, h('div', { class: 'nine p-oak face', style: 'width:calc(190px*var(--s))', 'data-panel': 'saves' }, lintel('Saves'), ...rows,
        h('div', { style: 'display:flex;justify-content:space-between;margin-top:calc(4px*var(--s))' }, btn('New world', { oak: true, on: () => (this.o.onPlay ? this.o.onPlay() : this.go('loading')) }), btn('Back', { oak: true, act: 'back', on: () => this.go(this.o.title ? 'title' : 'pause') }))));
    }
    settings() {
      const st = this.st, S = st.set;
      const tabs = h('div', { class: 'ptabs', role: 'tablist' }, ...[['picture', 'Picture'], ['weather', 'Weather'], ['move', 'Movement'], ['input', 'Input']].map(([id, lab]) =>
        h('button', { class: 'ptab oak', role: 'tab', type: 'button', 'aria-selected': String(st.settingsTab === id), 'data-tab': id, onclick: () => { st.settingsTab = id; this.render(); } }, P(lab))));
      const row = (label, ctl, note) => h('div', { class: 'bindrow', style: 'grid-template-columns:1fr auto' }, h('span', {}, label, note ? h('span', { class: 'dim' }, ' ' + note) : null), ctl);
      const check = (key, label) => h('button', { class: 'ki', type: 'button', role: 'checkbox', 'aria-checked': String(S[key]), 'aria-label': label, 'data-check': key,
        style: `--k:var(--i-check-${S[key] ? 'on' : 'off'});width:calc(12px*var(--s));height:calc(12px*var(--s));border:0;padding:0;cursor:pointer`,
        onclick: () => { S[key] = !S[key]; this.render(); } });
      let body;
      if (st.settingsTab === 'picture') {
        body = [row('Bloom', check('bloom', 'Bloom')), row('Frame limit', this.slider('frame', [0, 60, 120], (v) => (v ? v + ' fps' : 'Off'))),
          row('Render scale', this.slider('scale', SCALES, (v) => v + '%'))];
      } else if (st.settingsTab === 'weather') {
        body = [row('Rain shadow', check('rain', 'Rain shadow'), '(the rain fog and streaks; off by default)'), h('div', { class: 'dim', style: 'margin-top:calc(3px*var(--s))' }, 'The near shower, the wet ground and the drops on the lens still draw.')];
      } else if (st.settingsTab === 'move') {
        body = [row('Auto step', check('step', 'Auto step'), '(walk up one layer without a key)')];
      } else body = this.inputTab();
      return h('div', { class: 'centre oakset' }, h('div', { class: 'nine p-oak face', style: 'width:calc(300px*var(--s))', 'data-panel': 'settings' },
        lintel('Settings'), tabs, h('div', { style: 'display:grid;gap:calc(2px*var(--s));min-height:calc(202px*var(--s));align-content:start' }, ...body),
        h('div', { style: 'margin-top:calc(4px*var(--s));display:flex;justify-content:flex-end' }, btn('Back', { oak: true, act: 'back', on: () => this.go(st.back || 'pause') }))));
    }
    slider(key, values, fmt) {
      const S = this.st.set, idx = Math.max(0, values.indexOf(S[key]));
      const pct = (i) => (i / (values.length - 1)) * 100;
      const fill = h('div', { style: `position:absolute;left:0;top:0;height:calc(2px*var(--s));width:${pct(idx)}%;background:var(--rune-dim)` });
      const knob = h('i', { class: 'ki', style: `--k:var(--o-slider-knob);position:absolute;left:calc(${pct(idx)}% - 4.5px*var(--s));top:calc(-5px*var(--s));width:calc(9px*var(--s));height:calc(13px*var(--s));cursor:grab` });
      const val = h('span', { style: 'display:inline-block;width:calc(30px*var(--s));text-align:right', 'data-val': key }, fmt(values[idx]));
      const track = h('div', { class: 'nine p-track-h', role: 'slider', tabindex: '0', 'aria-label': key, 'aria-valuetext': fmt(values[idx]), 'data-slider': key, style: 'width:calc(70px*var(--s));height:calc(8px*var(--s));position:relative;cursor:pointer;touch-action:none' }, fill, knob);
      const set = (i) => {
        i = Math.max(0, Math.min(values.length - 1, i)); S[key] = values[i];
        fill.style.width = pct(i) + '%'; knob.style.left = `calc(${pct(i)}% - 4.5px*var(--s))`; val.textContent = fmt(values[i]); track.setAttribute('aria-valuetext', fmt(values[i]));
      };
      const at = (e) => { const r = track.getBoundingClientRect(); return Math.round(((e.clientX - r.left) / r.width) * (values.length - 1)); };
      track.addEventListener('pointerdown', (e) => { track.setPointerCapture(e.pointerId); track.dataset.drag = '1'; knob.style.cursor = 'grabbing'; set(at(e)); });
      track.addEventListener('pointermove', (e) => { if (track.dataset.drag) set(at(e)); });
      track.addEventListener('pointerup', () => { delete track.dataset.drag; knob.style.cursor = 'grab'; });
      track.addEventListener('keydown', (e) => { const i = values.indexOf(S[key]); if (e.key === 'ArrowRight') { set(i + 1); e.preventDefault(); } if (e.key === 'ArrowLeft') { set(i - 1); e.preventDefault(); } });
      return h('span', { style: 'display:inline-flex;align-items:center;gap:calc(4px*var(--s))' }, track, val);
    }
    inputTab() {
      const st = this.st, tab = BIND_TABS.find((t) => t[0] === st.inputTab), keys = st.binds[tab[0]];
      // sub-tabs: the same oak tab piece, sized to each label, one row
      const sub = h('div', { class: 'ptabs subtabs', role: 'tablist', style: 'flex-wrap:nowrap;margin-bottom:calc(2px*var(--s))' },
        ...BIND_TABS.map(([id, lab]) => h('button', { class: 'ptab oak', type: 'button', role: 'tab', 'aria-selected': String(st.inputTab === id), 'data-act': 'sub-' + id, style: 'padding:0 0 calc(2px*var(--s))',
          onclick: () => { st.inputTab = id; st.rebind = null; this.render(); } }, P(lab))));
      const used = {};
      keys.forEach((ks, i) => ks.forEach((k) => (used[k] = used[k] || []).push(i)));
      const rows = tab[2].map(([label, , proposed], i) => {
        const waiting = st.rebind && st.rebind[0] === tab[0] && st.rebind[1] === i;
        // a key the player just bound that another row of this tab already uses (the game's own
        // defaults share keys between contexts on purpose: W is move on foot and pitch in the Kestrel)
        const clash = st.changed.has(tab[0] + ':' + i) ? keys[i].filter((k) => used[k].length > 1) : [];
        const caps = h('span', { style: 'display:inline-flex;gap:calc(1px*var(--s))' }, ...(waiting ? [h('span', { class: 'keycap' }, P('press a key'))] : keys[i].map((k) => h('span', { class: 'keycap' }, P(k)))));
        const also = clash.length ? h('span', { class: 'warn' }, 'also: ' + used[clash[0]].filter((j) => j !== i).map((j) => tab[2][j][0]).join(', ')) : null;
        return h('div', { class: 'bindrow', 'data-bind': i },
          h('span', {}, label, proposed ? h('span', { class: 'dim' }, ' (proposed)') : null),
          h('span', { style: 'display:inline-flex;flex-direction:column;align-items:flex-end' }, caps, also),
          btn(waiting ? 'Cancel' : 'Change', { oak: true, cls: 'sm', act: 'change-' + i, on: () => { st.rebind = waiting ? null : [tab[0], i]; this.render(); this.box.focus(); } }));
      });
      // seven rows show at once; a longer tab (flying, vehicles) scrolls, and says so
      const H = 7 * 22, more = rows.length > 7;
      const list = h('div', { class: 'listbox', style: `height:calc(${H}px*var(--s))`, 'data-list': '1' }, h('div', {}, ...rows));
      const thumb = h('div', { class: 'nine p-thumb', style: `position:absolute;left:calc(-4px*var(--s));top:calc(-4px*var(--s));width:calc(8px*var(--s));height:calc(${Math.round(H * 7 / Math.max(7, rows.length))}px*var(--s))` });
      const track = h('div', { class: 'nine p-track-v', style: `width:calc(8px*var(--s));height:calc(${H}px*var(--s));position:relative;flex:none;${more ? '' : 'visibility:hidden'}` }, thumb);
      let off = 0;
      list.addEventListener('wheel', (e) => {
        e.preventDefault(); e.stopPropagation();
        const inner = list.firstChild, max = Math.max(0, inner.scrollHeight - list.clientHeight), s = parseFloat(getComputedStyle(list).getPropertyValue('--s')) || 2;
        off = Math.max(0, Math.min(max, off + Math.sign(e.deltaY) * 19 * s)); inner.style.transform = `translateY(${-off}px)`;
        thumb.style.top = `calc(-4px*var(--s) + ${max ? (off / max) * (track.clientHeight - thumb.clientHeight) : 0}px)`;
      }, { passive: false });
      return [sub, h('div', { style: 'display:flex;gap:calc(3px*var(--s))' }, h('div', { style: 'flex:1' }, list), track),
        h('div', { style: 'display:flex;justify-content:space-between;align-items:center' }, more ? h('span', { class: 'dim' }, `${rows.length} rows: the wheel scrolls`) : h('span', {}), btn('Reset tab', { oak: true, act: 'reset', on: () => { st.binds[tab[0]] = tab[2].map((r) => r[1].slice()); tab[2].forEach((_, j) => st.changed.delete(tab[0] + ':' + j)); this.render(); } }))];
    }

    // ---- rock (in play): pack, map, stations, chest, talk, trade, loading
    pack() {
      const st = this.st;
      const grid = h('div', { class: 'grid10' }, ...this.inv.pack.map((c, i) => { const s = slotEl(c, 'click'); this.slotEvents(s, this.inv.pack, i); return s; }));
      const hot = h('div', { class: 'grid10', style: 'margin-top:calc(3px*var(--s))' }, ...this.inv.hot.map((c, i) => { const s = slotEl(c, 'click' + (i === this.inv.sel ? ' sel' : '')); this.slotEvents(s, this.inv.hot, i); return s; }));
      return h('div', { class: 'centre' }, rockPanel('pack-craft', { 'data-panel': 'pack' }, lintel('Pack'),
        h('div', { style: 'display:flex;justify-content:flex-end;margin:calc(-14px*var(--s)) 0 calc(3px*var(--s))' },
          h('button', { class: 'ki', type: 'button', 'aria-label': 'close', style: '--k:var(--i-icon-close);width:calc(11px*var(--s));height:calc(11px*var(--s));border:0;padding:0;cursor:pointer', onclick: () => this.go('hud') })),
        grid, h('div', { class: 'carved', style: 'margin-top:calc(5px*var(--s));color:var(--text-dim)' }, P('Hotbar')), hot, this.craftRow()));
    }
    // The pack's own crafting: a drop-down of what the hands can make, what it takes against what is
    // carried, the result, Craft, and Recipes for the trees.
    craftRow() {
      const st = this.st, R = HAND.recipes[st.craft], [out, n, ins] = R;
      const can = ins.every(([id, k]) => this.inv.count(id) >= k);
      const canOf = (r) => r[2].every(([id, k]) => this.inv.count(id) >= k);
      const dd = h('button', { class: 'btn dd', type: 'button', 'data-act': 'craft-pick', 'aria-haspopup': 'listbox', 'aria-expanded': String(st.craftOpen),
        style: 'width:calc(92px*var(--s));height:calc(24px*var(--s));border-width:calc(5px*var(--s));display:flex;align-items:center;gap:calc(3px*var(--s));justify-content:flex-start;padding:0' },
        h('span', { class: 'ddi' }, icon(out)), P(ITEMS[out].name + (n > 1 ? ' x' + n : '')), h('span', { class: 'ddcaret', style: 'margin-left:auto' }));
      dd.addEventListener('click', (e) => { e.stopPropagation(); st.craftOpen = !st.craftOpen; this.render(); });
      const kids = [dd];
      if (st.craftOpen) {
        kids.push(h('div', { class: 'nine p-frame face', role: 'listbox', 'data-panel': 'craft-list', style: 'position:absolute;left:0;bottom:calc(100% + 2px*var(--s));z-index:30;display:grid;gap:calc(1px*var(--s));min-width:calc(120px*var(--s))' },
          ...HAND.recipes.map((r, i) => { const ok = canOf(r);
            const row = h('div', { role: 'option', 'aria-selected': String(i === st.craft), 'data-craft': i, tabindex: '0', style: `display:flex;align-items:center;gap:calc(3px*var(--s));cursor:pointer;${i === st.craft ? 'color:var(--title)' : ''}${ok ? '' : ';opacity:.55'}` },
              h('span', { class: 'ddi' }, icon(r[0])), P(ITEMS[r[0]].name + (r[1] > 1 ? ' x' + r[1] : '')), h('span', { class: 'dim', style: 'margin-left:auto' }, P(ok ? '' : 'short')));
            row.addEventListener('click', (e) => { e.stopPropagation(); st.craft = i; st.craftOpen = false; this.render(); });
            return row; })));
      }
      const need = ins.map(([id, k]) => { const have = this.inv.count(id); const s = slotEl([id, k], have >= k ? 'hov' : 'off'); s.title = `${ITEMS[id].name}: need ${k}, have ${have}`; s.dataset.need = have >= k ? 'ok' : 'short'; return s; });
      while (need.length < 2) need.push(slotEl(null, 'off'));
      const craft = btn('Craft', { act: 'craft', disabled: !can, on: () => {
        if (!ins.every(([id, k]) => this.inv.count(id) >= k)) return;
        ins.forEach(([id, k]) => this.inv.take(id, k)); this.inv.add(out, n); this.toast(`+${n} ${ITEMS[out].name}`); } });
      craft.style.cssText += ';height:calc(24px*var(--s));min-width:calc(44px*var(--s))';
      const tree = btn('Recipes', { act: 'recipes', on: () => this.go('recipes', { tree: out, craftOpen: false }) });
      tree.style.cssText += ';height:calc(14px*var(--s));border-width:calc(5px*var(--s));min-width:0';
      return h('div', { 'data-panel': 'craft', style: 'margin-top:calc(5px*var(--s))' },
        h('div', { style: 'display:flex;justify-content:space-between;align-items:center;height:calc(14px*var(--s))' }, h('div', { class: 'carved', style: 'color:var(--text-dim)' }, P('Craft')), tree),
        h('div', { style: 'position:relative;display:flex;align-items:center;gap:calc(3px*var(--s));margin-top:calc(3px*var(--s))' },
          ...kids, h('div', { class: 'slots' }, ...need), P('\u2192'), slotEl([out, n], can ? 'r-uncommon' : ''), h('span', { style: 'margin-left:auto' }, craft)));
    }
    // Recipes: every made thing, and the tree under the one picked, down to what is dug or caught.
    recipes() {
      const st = this.st, all = Object.keys(MADE);
      const pick = h('div', { class: 'grid4', style: 'align-content:start' }, ...all.map((id) => { const s = slotEl([id, 1], 'click' + (id === st.tree ? ' sel' : '')); s.title = ITEMS[id].name; s.dataset.tree = id;
        s.addEventListener('click', (e) => { e.stopPropagation(); st.tree = id; this.render(); }); return s; }));
      const rows = [];
      const walk = (id, k, depth, seen) => {
        const m = MADE[id], have = this.inv.count(id);
        rows.push(h('div', { style: `display:flex;align-items:center;gap:calc(3px*var(--s));padding-left:calc(${depth * 10}px*var(--s));height:calc(14px*var(--s))`, 'data-node': id },
          depth ? h('span', { class: 'elbow' }) : null, h('span', { class: 'ddi' }, icon(id)), P(`${ITEMS[id].name} x${k}`, depth ? '' : 'carved'),
          h('span', { class: 'dim', style: 'margin-left:auto;white-space:nowrap' }, P(m ? (m.at === 'hand' ? 'by hand' : 'at the ' + madeAt(m.at).toLowerCase()) + (m.fuel ? ', ' + ITEMS[m.fuel].name.toLowerCase() : '') : `found${have ? ', have ' + have : ''}`))));
        if (!m || seen.has(id) || depth > 3) return;
        const runs = Math.ceil(k / m.n);
        for (const [ins, kk] of m.ins) walk(ins, kk * runs, depth + 1, new Set([...seen, id]));
      };
      walk(st.tree, MADE[st.tree] ? MADE[st.tree].n : 1, 0, new Set());
      const handIdx = HAND.recipes.findIndex((r) => r[0] === st.tree);
      const foot = h('div', { style: 'display:flex;justify-content:flex-end;gap:calc(4px*var(--s));margin-top:calc(4px*var(--s))' },
        handIdx >= 0 ? btn('Craft in the pack', { act: 'tree-craft', on: () => this.go('pack', { craft: handIdx }) }) : null,
        btn('Back', { act: 'tree-back', on: () => this.go('pack') }));
      return h('div', { class: 'centre' }, rockPanel('chest', { 'data-panel': 'recipes' }, lintel('Recipes'),
        h('div', { style: 'display:flex;gap:calc(8px*var(--s))' }, pick, h('div', { style: 'flex:1;min-width:0;display:grid;align-content:start;gap:calc(1px*var(--s))', 'data-panel': 'tree' }, ...rows)), foot));
    }
    map() {
      const st = this.st, u = [];
      u.push(h('div', { class: 'nine p-map', style: 'position:absolute;inset:0;pointer-events:none' }));
      if (this.o.mapBg) u.unshift(h('div', { style: `position:absolute;inset:0;background:#000 url(${this.o.mapBg}) center/cover` }));
      const close = h('button', { class: 'ki', type: 'button', 'aria-label': 'close the drawer', 'data-act': 'drawer-close', style: '--k:var(--i-icon-close);width:calc(11px*var(--s));height:calc(11px*var(--s));border:0;padding:0;cursor:pointer', onclick: () => { st.drawer = false; this.render(); } });
      if (st.drawer) {
        u.push(h('div', { class: 'nine p-frame face', style: 'position:absolute;left:calc(22px*var(--s));top:calc(22px*var(--s));width:calc(144px*var(--s))', 'data-panel': 'drawer' },
          h('div', { style: 'display:flex;justify-content:space-between;align-items:center' }, h('div', { class: 'carved' }, P('Overlays')), close),
          h('div', { style: 'display:grid;grid-template-columns:1fr 1fr;gap:calc(2px*var(--s));margin-top:calc(4px*var(--s))' },
            ...MAP_OVERLAYS.map((t, i) => btn(t, { pressed: i === st.overlay, act: 'ov-' + i, on: () => { st.overlay = i; this.render(); } })))));
      } else {
        u.push(h('div', { style: 'position:absolute;left:calc(24px*var(--s));top:calc(24px*var(--s))' }, btn('Overlays', { act: 'drawer-open', on: () => { st.drawer = true; this.render(); } })));
      }
      u.push(h('div', { class: 'nine p-tip face', style: 'position:absolute;right:calc(22px*var(--s) + 4px);bottom:calc(22px*var(--s) + 4px)' }, P(MAP_OVERLAYS[st.overlay] + ' · 28.6N 0.0W')));
      return h('div', { style: 'position:absolute;inset:0', 'data-panel': 'map' }, ...u);
    }
    station() {
      const st = this.st, S = STATIONS[st.station], R = S.recipes[Math.min(st.recipe, S.recipes.length - 1)];
      const can = R[2].every(([id, n]) => this.inv.count(id) >= n) && (!S.fuel || this.inv.count(S.fuel) >= 1);
      const list = h('div', { style: 'display:grid;gap:calc(1px*var(--s));align-content:start;flex:1;height:calc(104px*var(--s));overflow:hidden' },
        ...S.recipes.map(([out, n, ins], i) => h('div', { class: 'recipe', 'data-recipe': i, tabindex: '0', style: `display:flex;align-items:center;gap:calc(4px*var(--s));cursor:pointer;${i === st.recipe ? 'color:var(--title)' : ''}`,
          onclick: () => { if (!st.run) { st.recipe = i; this.render(); } } }, slotEl([out, n], i === st.recipe ? 'sel' : ''), h('span', {}, ITEMS[out].name + (n > 1 ? ' x' + n : '')))));
      const ins = R[2].map(([id, n]) => { const have = this.inv.count(id); const s = slotEl([id, n], have >= n ? 'hov' : 'off'); s.title = `${ITEMS[id].name}: need ${n}, have ${have}`; s.dataset.need = have >= n ? 'ok' : 'short'; return s; });
      const pct = st.run ? st.run.t : (st.out ? 100 : 0);
      const bar = h('div', { class: 'nine p-bar', style: 'width:calc(56px*var(--s));height:calc(10px*var(--s))' }, h('div', { class: 'prog', 'data-prog': '1', style: `--k:var(--i-fill-${st.station === 'forge' ? 'stamina' : 'progress'});width:${pct}%` }));
      const outSlot = slotEl(st.out, st.out ? 'r-uncommon click' : '');
      outSlot.dataset.out = '1';
      if (st.out) outSlot.addEventListener('click', () => this.collect());
      const work = h('div', { style: 'display:grid;gap:calc(4px*var(--s));justify-items:center;align-content:start' },
        h('div', { class: 'slots' }, ...ins),
        S.fuel ? h('div', { style: 'display:flex;align-items:center;gap:calc(4px*var(--s))' }, h('span', { class: 'dim' }, 'Fuel'), slotEl([S.fuel, this.inv.count(S.fuel)], this.inv.count(S.fuel) ? 'hov' : 'off')) : h('div', { class: 'dim' }, st.station === 'mill' ? 'The stream turns it' : 'By hand'),
        bar, outSlot,
        btn(st.run ? S.verb + '...' : S.verb, { act: 'make', disabled: !can || !!st.run || !!st.out, cls: st.run ? 'is-pressed' : '', on: () => this.make() }));
      return h('div', { class: 'centre' }, rockPanel('station', { 'data-panel': 'station' }, lintel(S.title),
        h('div', { style: 'display:flex;gap:calc(5px*var(--s))' }, list,
          h('div', { class: 'nine p-track-v', style: 'width:calc(8px*var(--s));height:calc(104px*var(--s));position:relative;flex:none' }, h('div', { class: 'nine p-thumb', style: 'position:absolute;left:calc(-4px*var(--s));top:calc(-4px*var(--s));width:calc(8px*var(--s));height:calc(40px*var(--s))' })),
          work)));
    }
    make() {
      const st = this.st, S = STATIONS[st.station], R = S.recipes[st.recipe];
      if (st.run || st.out || !R[2].every(([id, n]) => this.inv.take(id, n))) return;
      if (S.fuel) this.inv.take(S.fuel, 1);
      st.run = { t: 0, start: performance.now(), dur: this.o.runMs || 2400 };
      if (this.o.onRun) this.o.onRun(st.station, true);
      this.render();
      const step = () => {
        if (!st.run) return;
        st.run.t = Math.min(100, ((performance.now() - st.run.start) / st.run.dur) * 100);
        const p = this.host.querySelector('[data-prog]'); if (p) p.style.width = st.run.t + '%';
        if (st.run.t < 100) { requestAnimationFrame(step); return; }
        st.run = null; st.out = [R[0], R[1]];
        if (this.o.onRun) this.o.onRun(st.station, false);
        this.render();
        setTimeout(() => this.collect(), 900);
      };
      requestAnimationFrame(step);
    }
    collect() {
      const st = this.st; if (!st.out) return;
      const [id, n] = st.out; st.out = null; this.inv.add(id, n);
      if (st.screen === 'station') this.render();
      this.toast(`+${n} ${ITEMS[id].name}`);
      if (this.o.onMade) this.o.onMade(id, n, st.station);
    }
    chestPanel() {
      const st = this.st;
      const move = (from, i, to) => { const c = from[i]; if (!c) return; const j = to.findIndex((x) => x && x[0] === c[0] && x[1] < 64); if (j >= 0) { to[j][1] += c[1]; from[i] = null; } else { const e = to.indexOf(null); if (e < 0) return; to[e] = c; from[i] = null; } this.render(); };
      const g = (arr, other, cols) => h('div', { class: cols }, ...arr.map((c, i) => { const s = slotEl(c, 'click'); s.addEventListener('click', () => move(arr, i, other)); return s; }));
      return h('div', { class: 'centre' }, rockPanel('chest', { 'data-panel': 'chest' }, lintel('Chest'),
        h('div', { style: 'display:flex;gap:calc(8px*var(--s));align-items:flex-start' },
          h('div', {}, h('div', { class: 'dim' }, 'The chest (fed by the press)'), g(st.chest, this.inv.pack, 'grid4')),
          h('div', {}, h('div', { class: 'dim' }, 'Your pack'), g(this.inv.pack, st.chest, 'grid10'))),
        h('div', { class: 'dim', style: 'margin-top:calc(3px*var(--s))' }, 'Click a stack to move it across.')));
    }
    talk() {
      const st = this.st, [line, choices] = TALK[st.talk];
      return h('div', { style: 'position:absolute;left:50%;transform:translateX(-50%);bottom:calc(20px*var(--k) + 32px*var(--s));display:flex;gap:calc(4px*var(--s));align-items:flex-end', 'data-panel': 'talk' },
        h('div', { class: 'nine p-portrait', style: 'width:calc(64px*var(--s));height:calc(64px*var(--s));overflow:hidden;flex:none' }, h('img', { src: 'npcs/smith.png', alt: 'Wren, the smith', style: 'width:calc(48px*var(--s));height:calc(48px*var(--s));image-rendering:pixelated' })),
        rockPanel('talk', {},
          h('div', { class: 'carved', style: 'margin-bottom:calc(3px*var(--s))' }, P('Wren, the smith')), h('div', { 'data-line': '1' }, line),
          h('div', { style: 'display:flex;gap:calc(3px*var(--s));margin-top:calc(5px*var(--s));flex-wrap:wrap' },
            ...choices.map(([lab, to], i) => btn(lab, { cls: i === 0 ? 'is-focus' : '', act: 'say-' + i, on: () => { if (to === '#close') this.go('hud'); else if (to === '#trade') this.go('trade'); else { st.talk = to; this.render(); } } })))));
    }
    trade() {
      const st = this.st, val = (list) => list.reduce((a, [id, n]) => a + ITEMS[id].v * n, 0);
      const offerV = val(st.offer), askV = val(st.ask);
      const addTo = (list, id) => { const e = list.find((x) => x[0] === id); if (e) e[1]++; else list.push([id, 1]); };
      const left = (list, id, have) => have - ((list.find((x) => x[0] === id) || [0, 0])[1]);
      const mine = this.inv.pack.filter(Boolean).slice(0, 8);
      const gMine = h('div', { class: 'grid4' }, ...Array.from({ length: 8 }, (_, i) => { const c = mine[i]; const n = c ? left(st.offer, c[0], c[1]) : 0; const s = slotEl(c && n ? [c[0], n] : null, c && n ? 'click' : ''); if (c && n) s.addEventListener('click', () => { addTo(st.offer, c[0]); this.render(); }); s.dataset.mine = i; return s; }));
      const gHers = h('div', { class: 'grid4' }, ...st.stock.map((c, i) => { const n = c ? left(st.ask, c[0], c[1]) : 0; const s = slotEl(c && n ? [c[0], n] : null, c && n ? 'click' : ''); if (c && n) s.addEventListener('click', () => { addTo(st.ask, c[0]); this.render(); }); s.dataset.hers = i; return s; }));
      const table = (list, which) => h('div', { class: 'slots', style: 'min-height:calc(24px*var(--s));flex-wrap:wrap;max-width:calc(72px*var(--s))' }, ...(list.length ? list.map(([id, n], i) => { const s = slotEl([id, n], 'click'); s.addEventListener('click', () => { list[i][1]--; if (!list[i][1]) list.splice(i, 1); this.render(); }); return s; }) : [slotEl(null, 'off')]));
      const fair = offerV >= askV && askV > 0;
      return h('div', { class: 'centre' }, rockPanel('trade', { 'data-panel': 'trade' }, lintel('Barter with Ysolde'),
        h('div', { style: 'display:flex;gap:calc(8px*var(--s));align-items:flex-start' },
          h('div', {}, h('div', { class: 'dim' }, 'Yours'), gMine),
          h('div', { style: 'display:grid;gap:calc(2px*var(--s));justify-items:center' }, h('div', { class: 'dim' }, 'You give'), table(st.offer), h('div', { class: 'dim' }, 'You get'), table(st.ask)),
          h('div', {}, h('div', { class: 'dim' }, 'Hers'), gHers)),
        h('div', { style: 'display:flex;justify-content:space-between;align-items:center;gap:calc(4px*var(--s));margin-top:calc(5px*var(--s))' },
          h('div', { 'data-total': '1', style: `color:${fair ? 'var(--text)' : 'var(--text-dim)'}` }, `Give ${offerV} · get ${askV}${askV ? (fair ? ': fair' : ': she wants more') : ''}`),
          h('div', { style: 'display:flex;gap:calc(3px*var(--s))' }, btn('Back', { act: 'back', on: () => this.go('talk') }),
            btn('Trade', { act: 'trade', disabled: !fair, on: () => {
              st.offer.forEach(([id, n]) => this.inv.take(id, n));
              st.ask.forEach(([id, n]) => { this.inv.add(id, n); const c = st.stock.find((x) => x && x[0] === id); c[1] -= n; if (!c[1]) st.stock[st.stock.indexOf(c)] = null; });
              const got = st.ask.map(([id, n]) => `${n} ${ITEMS[id].name}`).join(', '); st.offer = []; st.ask = []; this.toast('Traded for ' + got);
            } })))));
    }
    loading() {
      const st = this.st;
      const steps = ['Building the planet', 'Growing the trees', 'Warming the shaders', 'Ready'];
      const stepTxt = steps[Math.min(3, Math.floor(st.load / 34))];
      const go = () => {
        if (st.loadRun) return; st.loadRun = true; st.load = 0; const t0 = performance.now();
        const f = () => { st.load = Math.min(100, (performance.now() - t0) / 25); this.render(); if (st.load < 100) requestAnimationFrame(f); else { st.loadRun = false; if (this.o.onLoaded) this.o.onLoaded(); else if (this.o.screen !== 'loading') this.go('hud'); } };
        requestAnimationFrame(f);
      };
      return h('div', { style: 'position:absolute;inset:0;background:#010e1a' }, h('div', { class: 'centre' }, rockPanel('loading', { 'data-panel': 'loading', style: 'text-align:center' },
        ki('i-emblem', 48, 48), h('div', { class: 'carved', style: 'margin:calc(3px*var(--s)) 0' }, P('Pale Blue Dot')),
        h('div', { class: 'dim', style: 'margin-bottom:calc(4px*var(--s))', 'data-step': '1' }, stepTxt),
        h('div', { class: 'nine p-bar', style: 'width:100%;height:calc(10px*var(--s))' }, h('div', { class: 'prog', style: `--k:var(--i-fill-progress);width:${st.load}%` })),
        h('div', { style: 'margin-top:calc(4px*var(--s))' }, btn(st.load >= 100 ? 'Replay' : 'Play', { act: 'load-play', disabled: st.loadRun, on: go })))));
    }
  }

  // ------------------------------------------------------------------ scenes
  const BASE_W = 1440, BASE_H = 900;
  const COARSE = window.matchMedia && matchMedia('(pointer: coarse)').matches;
  const mainWidth = () => document.querySelector('main').clientWidth - 2;
  function sceneScale() { const w = document.querySelector('main').clientWidth - 2; return w >= BASE_W ? 1 : 0.5; }
  function scene(bg, label, o = {}) {
    const k = o.k || sceneScale(), s = Math.round(2 * k);
    const sc = h('div', { class: 'scene', tabindex: '0', 'aria-label': label + ' (focus to use the keys)', style: `width:${BASE_W * k}px;height:${BASE_H * k}px;${bg ? `background-image:url(${bg});` : ''}--k:${k};--s:${s}` });
    const ui = h('div', { class: 'ui' });
    sc.append(ui, h('div', { class: 'tag' }, label));
    if (o.hint !== false) sc.append(h('div', { class: 'hint' }, o.hint || 'click to focus: Esc, I, M, 1-0, wheel'));
    sc.addEventListener('mousedown', () => sc.focus({ preventScroll: true }));
    let el = h('div', { class: 'scenebox' }, sc);
    if (o.crop) {
      const [cw, ch, cx, cy] = o.crop;
      sc.style.position = 'absolute'; sc.style.left = -cx + 'px'; sc.style.top = -cy + 'px';
      el = h('div', { class: 'scenebox', style: `position:relative;width:${cw}px;max-width:100%;height:${ch}px;overflow:hidden` }, sc);
    }
    if (o.look) el.dataset.look = o.look;
    // A page narrower than the scene (a phone): the whole scene is scaled down to fit, as one picture
    // (the game itself is desktop only; this is the mockup page). Pixel art stays pixelated.
    const w0 = o.crop ? o.crop[0] : BASE_W * k, h0 = o.crop ? o.crop[1] : BASE_H * k, avail = mainWidth() - (o.pad || 0);
    if (avail < w0) {
      const f = avail / w0;
      el.style.cssText += `;width:${w0}px;max-width:none;overflow:hidden;transform:scale(${f});transform-origin:0 0`;
      el = h('div', { class: 'fitbox', style: `width:${Math.floor(avail)}px;height:${Math.ceil(h0 * f)}px;overflow:hidden` }, el);
      if (o.look) el.dataset.look = o.look;
    }
    // Touch: the keys a scene answers to, as buttons under it (slots are tapped directly)
    if (COARSE && o.keys !== false) {
      const key = (code) => () => { if (sc.__ui) sc.__ui.key({ code, preventDefault() {}, stopPropagation() {} }); };
      el = h('div', { class: 'withkeys' }, el, h('div', { class: 'keybar' },
        h('button', { type: 'button', onclick: key('Escape') }, 'Esc / Back'), h('button', { type: 'button', onclick: key('KeyI') }, 'I  Pack'), h('button', { type: 'button', onclick: key('KeyM') }, 'M  Map')));
      if (o.look) el.dataset.look = o.look;
    }
    return { el, sc, ui, k, s };
  }

  // ------------------------------------------------------------------ tabs
  const state = { hud: 'after', pack: 'after', menu: 'after', map: 'after', station: 'forge', talk: 'talk', loading: 'after' };
  const UIS = {};
  const R = {
    hud() {
      const box = document.getElementById('hudScenes'); box.innerHTML = '';
      for (const [bg, lab, sel, head, town] of [['ui/game/day-desert.jpg', 'Noon, the desert', 6, 0, 'Tahal  2.2 km'], ['ui/game/night-dunmouth.jpg', 'Night, Dunmouth', 4, 120, 'Dunmouth  80 m']]) {
        const sc = scene(bg, (state.hud === 'before' ? 'Before: ' : 'After: ') + lab, { hint: state.hud === 'before' ? false : undefined });
        if (state.hud === 'after') { const u = new GameUI(sc.ui, { box: sc.sc, screen: 'hud', heading: head, town, mapBg: 'ui/game/map-before.jpg' }); u.inv.sel = sel; u.render(); (UIS.hud = UIS.hud || []).push(u); }
        box.append(sc.el);
      }
    },
    pack() {
      const box = document.getElementById('packScenes'); box.innerHTML = '';
      const before = state.pack === 'before';
      const sc = scene(before ? 'ui/game/pack-before.jpg' : 'ui/game/day-meadow.jpg', before ? 'Before: the pack today' : 'After: the pack', { hint: before ? false : 'right click takes half; I or Esc shuts' });
      if (!before) UIS.pack = new GameUI(sc.ui, { box: sc.sc, screen: 'pack', mapBg: 'ui/game/map-before.jpg' });
      box.append(sc.el);
    },
    menu() {
      const box = document.getElementById('menuScenes'); box.innerHTML = '';
      if (state.menu === 'before') { box.append(scene('ui/game/pause-before.jpg', 'Before: paused', { hint: false }).el, scene('ui/game/settings-before.jpg', 'Before: settings', { hint: false }).el); return; }
      const a = scene('ui/game/day-meadow.jpg', 'After: paused'), b = scene('ui/game/day-meadow.jpg', 'After: settings, Input');
      UIS.pause = new GameUI(a.ui, { box: a.sc, screen: 'pause' });
      UIS.settings = new GameUI(b.ui, { box: b.sc, screen: 'settings', settingsTab: 'input', back: 'pause' });
      box.append(a.el, b.el);
    },
    map() {
      const box = document.getElementById('mapScenes'); box.innerHTML = '';
      const sc = scene('ui/game/map-before.jpg', state.map === 'before' ? 'Before: the map today' : 'After: framed, the overlays in a drawer', { hint: state.map === 'before' ? false : undefined });
      if (state.map === 'after') UIS.map = new GameUI(sc.ui, { box: sc.sc, screen: 'map', drawer: true });
      box.append(sc.el);
    },
    station() {
      const box = document.getElementById('stationScenes'); box.innerHTML = '';
      const night = state.station === 'hearth';
      const sc = scene(night ? 'ui/game/night-dunmouth.jpg' : 'ui/game/day-meadow.jpg', STATIONS[state.station] ? STATIONS[state.station].title : 'Chest', { hint: 'E or Esc shuts' });
      UIS.station = new GameUI(sc.ui, { box: sc.sc, screen: state.station === 'chest' ? 'chest' : 'station', station: state.station === 'chest' ? 'forge' : state.station });
      box.append(sc.el);
    },
    talk() {
      const box = document.getElementById('talkScenes'); box.innerHTML = '';
      const sc = scene('ui/game/night-dunmouth.jpg', state.talk === 'talk' ? 'Talk, at night' : 'Trade, at night');
      UIS.talk = new GameUI(sc.ui, { box: sc.sc, screen: state.talk, town: 'Dunmouth' });
      box.append(sc.el);
    },
    loading() {
      const box = document.getElementById('loadingScenes'); box.innerHTML = '';
      const sc = scene('ui/game/loading-before.jpg', state.loading === 'before' ? 'Before: the loading screen today' : 'After', { hint: false });
      if (state.loading === 'after') UIS.loading = new GameUI(sc.ui, { box: sc.sc, screen: 'loading' });
      box.append(sc.el);
    },
    title() {
      const box = document.getElementById('titleScenes'); box.innerHTML = '';
      const sc = scene('ui/game/title-backdrop.jpg', 'The main menu', { hint: 'Esc steps back' });
      UIS.title = new GameUI(sc.ui, { box: sc.sc, screen: 'title', title: true, onLoaded: () => { location.hash = '#play'; } });
      UIS.title.o.onPlay = null;
      box.append(sc.el);
    },
    play() { if (window.PBDPlay) window.PBDPlay.open(); else loadPlay(); },
    kit() { kitTab(); },
  };
  function loadPlay() {
    const box = document.getElementById('playScenes');
    if (box.dataset.loading) return; box.dataset.loading = '1';
    box.append(h('div', { class: 'note', id: 'playLoading' }, 'Loading three.js and the scene...'));
    const add = (src) => new Promise((res, rej) => { const s = document.createElement('script'); s.src = src; s.onload = res; s.onerror = () => rej(new Error('failed ' + src)); document.body.append(s); });
    add('https://cdnjs.cloudflare.com/ajax/libs/three.js/r128/three.min.js').then(() => add('building/scene-kit.js')).then(() => add('ui/play.js'))
      .then(() => { document.getElementById('playLoading')?.remove(); if (location.hash === '#play' || !location.hash) window.PBDPlay.open(); })
      .catch((e) => { const n = document.getElementById('playLoading'); if (n) n.textContent = 'The demo could not load: ' + e.message; });
  }

  // ------------------------------------------------------------------ the kit tab
  let kitMade = false;
  function kitTab() {
    if (kitMade) return; kitMade = true;
    // Q1: which rock, on in-play screens, three looks side by side
    const q = document.getElementById('q1Rows');
    for (const [screen, crop, lab] of [['hud', [600, 150, 420, 750], 'HUD (hotbar)'], ['pack', [600, 430, 420, 236], 'Pack']]) {
      const row = h('div', { class: 'row', 'data-q1': screen });
      for (const look of ['drawn', 'rendered', 'smooth']) {
        const sc = scene('ui/game/day-meadow.jpg', look, { k: 1, crop, look, hint: false, pad: 28 });
        const u = new GameUI(sc.ui, { box: sc.sc, screen, heading: 20 }); u.inv.sel = 4; u.render();
        row.append(h('div', { class: 'card' }, sc.el, h('div', { class: 'cap' }, `${lab}: ${look === 'drawn' ? 'drawn rock' : look === 'rendered' ? 'rendered rock (pixelated)' : 'smooth rock (Cycles)'}`)));
      }
      q.append(row);
    }
    // timber out of play, rock in play
    const split = document.getElementById('splitRow');
    for (const [screen, crop, cap] of [['pause', [380, 300, 530, 300], 'Out of play: pause in carved oak'], ['station', [700, 380, 370, 260], 'In play: a machine in rock and stone']]) {
      const sc = scene('ui/game/day-meadow.jpg', cap, { k: 1, crop, hint: false, pad: 28 });
      new GameUI(sc.ui, { box: sc.sc, screen, station: 'forge' });
      split.append(h('div', { class: 'card' }, sc.el, h('div', { class: 'cap' }, cap)));
    }
    // font
    const fr = document.getElementById('fontRow');
    fr.append(h('div', { class: 'card' }, h('div', { class: 'scrollx' }, h('img', { src: 'ui/font/kit-font.png', alt: 'the glyph sheet', style: 'height:28px;max-width:none;image-rendering:pixelated' })), h('div', { class: 'cap' }, 'the glyph sheet, 4x')),
      h('div', { class: 'card', style: 'display:grid;gap:8px' }, h('div', { class: 'p-lintel' }, P('Pack')), btn('Settings'), btn('Settings', { oak: true }),
        h('div', { style: 'background:var(--face);padding:6px;color:var(--label)' }, P('ABCDEFGHIJKLM NOPQRSTUVWXYZ')), h('div', { style: 'background:var(--face);padding:6px;color:var(--label)' }, P('0123456789 O0 I1 E3 C G S')),
        h('div', { class: 'cap' }, 'carved in oak, inlaid in stone, 2x')));
    // rock variants (fixed images)
    const rv = document.getElementById('rockVariants');
    rv.append(h('div', { class: 'card' }, h('div', { class: 'nine p-rock', style: 'width:192px;height:96px' }), h('div', { class: 'cap' }, 'drawn (9-slice of the 48 px slab)')));
    for (const [url, cap, rend] of [['ui/rendered/rock/rock-192x96.png', 'the base rock, pixelated (no outline)', 'pixelated'], ['ui/rendered/hires/rock/rock-192x96.png', 'the base rock, smooth: the approved sample', 'auto'],
      ['ui/rendered/hires/rock-classic-original.png', 'the original smooth render, kept as it was (lines and all)', 'auto']])
      rv.append(h('div', { class: 'card' }, h('div', { style: `width:${url.includes('original') ? 192 : 384}px;height:${url.includes('original') ? 192 : 192}px;max-width:100%;background:url(${url}) 0 0/100% 100% no-repeat;image-rendering:${rend}` }), h('div', { class: 'cap' }, cap)));
    // sizes
    const sz = document.getElementById('sizes');
    for (const n of ['kit-48', 'kit-128', 'kit-200']) sz.append(rockPanel(n, {}));
    for (const cls of ['p-frame', 'p-oak']) for (const [w, hh] of [[96, 96], [256, 128], [400, 200]]) sz.append(h('div', { class: 'nine ' + cls, style: `width:${w}px;height:${hh}px` }));
    for (const w of [80, 160, 320]) sz.append(h('div', { class: 'p-lintel', style: `width:${w}px` }));
    rockify(document.getElementById('kit'));
    // every piece, at 2x, in the current look
    const pc = document.getElementById('pieces');
    const meta = { rock: [48, 48], frame: [48, 48], lintel: [40, 16], 'map-frame': [64, 64], portrait: [32, 32], emblem: [48, 48], compass: [80, 18], 'compass-pointer': [7, 6], 'compass-town': [5, 5], tooltip: [24, 18], bar: [40, 10],
      'scroll-track': [8, 24], 'scroll-thumb': [8, 16], 'slider-track': [32, 8], 'slider-knob': [9, 13], 'check-off': [12, 12], 'check-on': [12, 12], 'tab-open': [40, 18], 'tab-shut': [40, 18] };
    const dims = (n) => meta[n] || (n.startsWith('slot') ? [24, 24] : n.startsWith('button') ? [48, 22] : n.startsWith('fill') ? [2, 4] : n.startsWith('icon') ? [11, 11] : [16, 16]);
    for (const n of STONE) { const [w, hh] = dims(n); pc.append(h('div', { class: 'card', style: 'text-align:center' }, ki('i-' + n, n.startsWith('fill') ? 8 : w, n.startsWith('fill') ? 8 : hh), h('div', { class: 'cap' }, n))); }
    for (const n of OAK) { const [w, hh] = dims(n); pc.append(h('div', { class: 'card', style: 'text-align:center' }, ki('o-' + n, w, hh), h('div', { class: 'cap' }, 'oak/' + n))); }
    // contrast, three looks
    const byId = (id) => document.getElementById(id) || Object.values(INFO).map((i) => i.box.querySelector('#' + id)).find(Boolean); // the tables live in the Kit's info modal
    const ct = byId('contrast');
    Promise.all([fetch('ui/slices.json').then((r) => r.json()), fetch('ui/rendered/slices.json').then((r) => r.json())]).then(([d, r]) => {
      ct.append(h('tr', {}, h('th', {}, 'Drawn pair (tokens)'), h('th', {}, 'Ratio')));
      for (const [k, v] of Object.entries(d.contrast)) ct.append(h('tr', {}, h('td', {}, k), h('td', {}, v + ':1')));
      ct.append(h('tr', {}, h('th', {}, 'Rendered and smooth piece: token'), h('th', {}, 'Rendered / smooth')));
      for (const [k, v] of Object.entries(r.contrast)) for (const [t, x] of Object.entries(v)) ct.append(h('tr', {}, h('td', {}, `${k}: ${t}`), h('td', {}, `${x}:1 / ${r.contrast_smooth[k][t]}:1`)));
    }).catch(() => ct.append(h('tr', {}, h('td', {}, 'contrast tables load from ui/slices.json and ui/rendered/slices.json'))));
    const tok = byId('tokens');
    for (const [n, cs] of [['STONE 0..4 (+ STONE_MID)', '#011c34 #1f3b52 #3e586e #5d778d #92a3b2 #bfcad7'], ['ROCK 0..4', '#010e1a #1f3b52 #3e586e #5d778d #92a3b2'], ['OAK 0..4 (timber menus, accents)', '#2f1525 #41322c #725244 #ab8465 #cfaa81'],
      ['IRON', '#010000 #161612 #2c2c25 #434137 #a09f91'], ['BRASS', '#733126 #c56d25 #fb902f #faaf5c'], ['RUNE (focus, selection)', '#204f63 #53998b #84e3b5 #c1f1d8'], ['TEXT, TEXT_DIM, TITLE, LABEL, LABEL_DIM, TAB_DIM', '#eab798 #cfaa81 #faaf5c #fffefe #92a3b2 #bfcad7']])
      tok.append(h('span', { class: 'tok' }, ...cs.split(' ').map((c) => h('i', { style: `background:${c}`, title: c })), n));
  }

  // ------------------------------------------------------------------ tabs: one open, linkable
  const made = new Set();
  function openTab() {
    const id = (location.hash || '#play').slice(1);
    const ok = document.getElementById(id) && document.getElementById(id).classList.contains('tab') ? id : 'play';
    document.querySelectorAll('section.tab').forEach((s) => s.classList.toggle('open', s.id === ok));
    document.querySelectorAll('nav.tabs a').forEach((a) => a.setAttribute('aria-selected', String(a.getAttribute('href') === '#' + ok)));
    if (window.PBDPlay && ok !== 'play') window.PBDPlay.close();
    R[ok]();
    made.add(ok);
  }
  document.querySelectorAll('.seg[data-for]').forEach((seg) => seg.addEventListener('click', (e) => {
    const b = e.target.closest('button'); if (!b) return;
    const key = seg.dataset.for; state[key] = b.dataset.v;
    seg.querySelectorAll('button').forEach((x) => x.setAttribute('aria-pressed', String(x === b)));
    R[key]();
  }));
  window.addEventListener('hashchange', openTab);
  let rz = 0, lastK = 0;
  let lastW = 0;
  window.addEventListener('resize', () => { clearTimeout(rz); rz = setTimeout(() => { const k = sceneScale(), w = mainWidth(); if (k !== lastK || (w !== lastW && w < 1440)) { lastK = k; lastW = w; const id = (location.hash || '#play').slice(1); if (id !== 'play' && id !== 'kit' && R[id]) R[id](); } }, 150); });

  // ------------------------------------------------------------------ descriptive text in modals
  // CLAUDE.md, "Descriptive text goes in modals" (owner, 2026-11-14: "Too much text on screen.
  // Descriptive Text on all mockups belong in modals"). The screen keeps the scenes, the controls, a
  // short title per tab and the tracker link; every note, decision, caption and table moves into the
  // tab's info modal, opened from the small i button by the title.
  const dlg = h('dialog', { class: 'infomodal', 'aria-label': 'About this' });
  const dTitle = h('b', {}), dBody = h('div', { class: 'mbody' });
  dlg.append(h('div', { class: 'mhead' }, dTitle, h('button', { type: 'button', 'aria-label': 'Close', onclick: () => dlg.close() }, '\u2715')), dBody);
  dlg.addEventListener('click', (e) => { if (e.target === dlg) dlg.close(); });
  document.body.append(dlg);
  const INFO = {};
  function infoButton(key, title, nodes) {
    const box = h('div', {}); nodes.forEach((n) => box.append(n));
    INFO[key] = { title, box };
    return h('button', { class: 'infobtn', type: 'button', 'aria-label': 'About: ' + title, title: 'About this', 'data-info': key, onclick: () => openInfo(key) }, 'i');
  }
  function openInfo(key) {
    const it = INFO[key]; dTitle.textContent = it.title; dBody.innerHTML = ''; dBody.append(it.box);
    if (dlg.showModal) dlg.showModal(); else dlg.setAttribute('open', '');
  }
  (function moveText() {
    const head = document.querySelector('header.top'), h1 = head.querySelector('h1');
    const hn = [...head.querySelectorAll('.info')];
    const row = h('div', { class: 'titlerow' }); h1.replaceWith(row); row.append(h1, infoButton('page', 'Stone and Timber', hn));
    for (const sec of document.querySelectorAll('section.tab')) {
      const h2 = sec.querySelector('h2'); if (!h2) continue;
      const nodes = [...sec.querySelectorAll('p.note, .ask, .info, .scrollx, .tok')].filter((n) => !n.parentElement.closest('p.note, .ask, .info, .scrollx'));
      // a kit subsection title whose only content moved goes with it
      const tr = h('div', { class: 'titlerow' }); h2.replaceWith(tr); tr.append(h2, infoButton(sec.id, h2.textContent, nodes));
    }
  })();

  window.PBDUI = { openInfo, COARSE, mainWidth, GameUI, P, h, btn, slotEl, ITEMS, BLOCKS, newInv, STATIONS, scene, compass, setLook, get look() { return LOOK; }, UIS };
  // The font and the rock panel sizes are inlined (copies of ui/font/kit-font.json and the
  // "rock_panels" of ui/rendered/slices.json), so the page draws at once instead of waiting on two
  // fetches (the owner, 2026-11-14: "there's some weird delay in loading ui").
  FONT = {"source":"tools/art/ui_art.py","height":7,"spacing":1,"space":3,"case":"upper (lower case is drawn as upper)","glyphs":{"A":{"x":0,"w":5,"rows":[".###.","#...#","#...#","#####","#...#","#...#","#...#"]},"B":{"x":6,"w":5,"rows":["####.","#...#","#...#","####.","#...#","#...#","####."]},"C":{"x":12,"w":5,"rows":[".####","#....","#....","#....","#....","#....",".####"]},"D":{"x":18,"w":5,"rows":["####.","#...#","#...#","#...#","#...#","#...#","####."]},"E":{"x":24,"w":5,"rows":["#####","#....","#....","####.","#....","#....","#####"]},"F":{"x":30,"w":5,"rows":["#####","#....","#....","####.","#....","#....","#...."]},"G":{"x":36,"w":5,"rows":[".####","#....","#....","#.###","#...#","#...#",".###."]},"H":{"x":42,"w":5,"rows":["#...#","#...#","#...#","#####","#...#","#...#","#...#"]},"I":{"x":48,"w":3,"rows":["###",".#.",".#.",".#.",".#.",".#.","###"]},"J":{"x":52,"w":5,"rows":["..###","...#.","...#.","...#.","#..#.","#..#.",".##.."]},"K":{"x":58,"w":5,"rows":["#...#","#..#.","#.#..","##...","#.#..","#..#.","#...#"]},"L":{"x":64,"w":5,"rows":["#....","#....","#....","#....","#....","#....","#####"]},"M":{"x":70,"w":5,"rows":["#...#","##.##","#.#.#","#.#.#","#...#","#...#","#...#"]},"N":{"x":76,"w":5,"rows":["#...#","##..#","#.#.#","#..##","#...#","#...#","#...#"]},"O":{"x":82,"w":5,"rows":[".###.","#...#","#...#","#...#","#...#","#...#",".###."]},"P":{"x":88,"w":5,"rows":["####.","#...#","#...#","####.","#....","#....","#...."]},"Q":{"x":94,"w":5,"rows":[".###.","#...#","#...#","#...#","#.#.#","#..#.",".##.#"]},"R":{"x":100,"w":5,"rows":["####.","#...#","#...#","####.","#.#..","#..#.","#...#"]},"S":{"x":106,"w":5,"rows":[".####","#....","#....",".###.","....#","....#","####."]},"T":{"x":112,"w":5,"rows":["#####","..#..","..#..","..#..","..#..","..#..","..#.."]},"U":{"x":118,"w":5,"rows":["#...#","#...#","#...#","#...#","#...#","#...#",".###."]},"V":{"x":124,"w":5,"rows":["#...#","#...#","#...#","#...#",".#.#.",".#.#.","..#.."]},"W":{"x":130,"w":5,"rows":["#...#","#...#","#...#","#.#.#","#.#.#","##.##","#...#"]},"X":{"x":136,"w":5,"rows":["#...#",".#.#.",".#.#.","..#..",".#.#.",".#.#.","#...#"]},"Y":{"x":142,"w":5,"rows":["#...#","#...#",".#.#.","..#..","..#..","..#..","..#.."]},"Z":{"x":148,"w":5,"rows":["#####","....#","...#.","..#..",".#...","#....","#####"]},"0":{"x":154,"w":5,"rows":[".###.","#...#","#..##","#.#.#","##..#","#...#",".###."]},"1":{"x":160,"w":3,"rows":[".#.","##.",".#.",".#.",".#.",".#.","###"]},"2":{"x":164,"w":5,"rows":[".###.","#...#","....#","...#.","..#..",".#...","#####"]},"3":{"x":170,"w":5,"rows":["####.","....#","....#",".###.","....#","....#","####."]},"4":{"x":176,"w":5,"rows":["...#.","..##.",".#.#.","#..#.","#####","...#.","...#."]},"5":{"x":182,"w":5,"rows":["#####","#....","####.","....#","....#","#...#",".###."]},"6":{"x":188,"w":5,"rows":[".###.","#....","#....","####.","#...#","#...#",".###."]},"7":{"x":194,"w":5,"rows":["#####","....#","...#.","..#..","..#..","..#..","..#.."]},"8":{"x":200,"w":5,"rows":[".###.","#...#","#...#",".###.","#...#","#...#",".###."]},"9":{"x":206,"w":5,"rows":[".###.","#...#","#...#",".####","....#","....#",".###."]}," ":{"x":212,"w":3,"rows":["...","...","...","...","...","...","..."]},".":{"x":216,"w":1,"rows":[".",".",".",".",".",".","#"]},",":{"x":218,"w":2,"rows":["..","..","..","..","..",".#","#."]},":":{"x":221,"w":1,"rows":[".",".","#",".",".","#","."]},";":{"x":223,"w":2,"rows":["..","..",".#","..","..",".#","#."]},"!":{"x":226,"w":1,"rows":["#","#","#","#","#",".","#"]},"?":{"x":228,"w":5,"rows":[".###.","#...#","....#","...#.","..#..",".....","..#.."]},"'":{"x":234,"w":1,"rows":["#","#",".",".",".",".","."]},"\"":{"x":236,"w":3,"rows":["#.#","#.#","...","...","...","...","..."]},"-":{"x":240,"w":3,"rows":["...","...","...","###","...","...","..."]},"+":{"x":244,"w":5,"rows":[".....","..#..","..#..","#####","..#..","..#..","....."]},"=":{"x":250,"w":4,"rows":["....","....","####","....","####","....","...."]},"/":{"x":255,"w":5,"rows":["....#","....#","...#.","..#..",".#...","#....","#...."]},"(":{"x":261,"w":2,"rows":[".#","#.","#.","#.","#.","#.",".#"]},")":{"x":264,"w":2,"rows":["#.",".#",".#",".#",".#",".#","#."]},"&":{"x":267,"w":5,"rows":[".##..","#..#.","#.#..",".#...","#.#.#","#..#.",".##.#"]},"%":{"x":273,"w":5,"rows":["##..#","##..#","...#.","..#..",".#...","#..##","#..##"]},"#":{"x":279,"w":5,"rows":[".#.#.",".#.#.","#####",".#.#.","#####",".#.#.",".#.#."]},"<":{"x":285,"w":4,"rows":["...#","..#.",".#..","#...",".#..","..#.","...#"]},">":{"x":290,"w":4,"rows":["#...",".#..","..#.","...#","..#.",".#..","#..."]},"_":{"x":295,"w":4,"rows":["....","....","....","....","....","....","####"]},"\u00b7":{"x":300,"w":1,"rows":[".",".",".","#",".",".","."]},"\u2191":{"x":302,"w":5,"rows":["..#..",".###.","#.#.#","..#..","..#..","..#..","..#.."]},"\u2193":{"x":308,"w":5,"rows":["..#..","..#..","..#..","..#..","#.#.#",".###.","..#.."]},"\u2190":{"x":314,"w":7,"rows":[".......","..#....",".#.....","#######",".#.....","..#....","......."]},"\u2192":{"x":322,"w":7,"rows":[".......","....#..",".....#.","#######",".....#.","....#..","......."]}}};
  ROCKS = {"pack":[268,152],"pack-craft":[268,204],"station":[230,156],"chest":[372,168],"talk":[270,124],"trade":[308,148],"loading":[150,148],"sample":[192,96],"kit-48":[48,48],"kit-128":[128,64],"kit-200":[200,100]};
  // Fetch every kit piece and rock frame of the open look now, and the other looks when idle, so a
  // panel never appears before its art.
  const KEEP = [];
  function preload(l) {
    const L = LOOKS[l]; if (!L) return;
    const src = STONE.map((n) => L.dir + n + '.png').concat(OAK.map((n) => L.oak + n + '.png'));
    if (l !== 'drawn') for (const [w, hh] of Object.values(ROCKS)) src.push(`ui/rendered/${l === 'smooth' ? 'hires/' : ''}rock/rock-${w}x${hh}.png`);
    for (const u of src) { const i = new Image(); i.decoding = 'async'; i.src = u; KEEP.push(i); }
  }
  rockify(document);
  preload(LOOK);
  document.addEventListener('pbd-look', (e) => preload(e.detail));
  setTimeout(() => Object.keys(LOOKS).filter((l) => l !== LOOK).forEach(preload), 1500);
  lastK = sceneScale(); openTab();
})();
