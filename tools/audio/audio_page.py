#!/usr/bin/env python3
"""The audition page every Pale Blue Dot audio set is heard on: the one implementation of its look.

Ported from Fleet Command (tools/audio/audio_page.py at 08374c4; the look and the player are Crab
Breeze's tools/audio/gen_sfx_page.py at 0e569e3). A works sheet: a masthead (written by the caller),
then one run sheet per group, each row a sound with its description, where it came from, its
length, a waveform strip from its peaks and a play button (loops play on repeat). Each sound plays
from its MP3 share copy, kept beside the page (docs/mockups/sounds/<cue>.mp3, committed) and
published beside it at the same relative path (files_map below): a page with megabytes of audio
inlined never loaded in the artifact viewer (Fleet Command, 2026-11-05). MP3, not the game's OGG,
because every browser plays it. Changed here: an item's src is relative to the page, so the page
plays from the repository and from the artifact alike, and the files map names each MP3's
repository path; each row prints its sources.
"""
import json
import subprocess

import numpy as np

HEAD = """<meta charset="utf-8">
<title>__TITLE__</title>
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="claude:viewport-fit-sides" content="cover">
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Big+Shoulders+Display:wght@600;800&family=Barlow:wght@400;500;600&family=IBM+Plex+Mono:wght@400;500&display=swap">
<style>
/* Layout: the works sheet for the game's sounds. A masthead plate, then one run sheet per group, each row a cue with its waveform strip. */
:root {
  --paper: #edf0ee; --plate: #f8faf9; --ink: #1a2026; --steel: #56626c; --rule: #c6cdd0;
  --amber: #b97700; --amber-ink: #1a2026; --wave: #2f6f86;
  --f-display: "Big Shoulders Display", "Arial Narrow", Impact, sans-serif;
  --f-body: "Barlow", "Segoe UI", Roboto, sans-serif;
  --f-mono: "IBM Plex Mono", ui-monospace, Menlo, Consolas, monospace;
}
@media (prefers-color-scheme: dark) { :root:not([data-theme="light"]) {
  --paper: #111518; --plate: #181d21; --ink: #e2e6e7; --steel: #929ea7; --rule: #2b3339;
  --amber: #f0a62a; --amber-ink: #111518; --wave: #5fb0cc; color-scheme: dark } }
:root[data-theme="dark"] {
  --paper: #111518; --plate: #181d21; --ink: #e2e6e7; --steel: #929ea7; --rule: #2b3339;
  --amber: #f0a62a; --amber-ink: #111518; --wave: #5fb0cc; color-scheme: dark }
* { box-sizing: border-box }
body { background: var(--paper); color: var(--ink); font: 400 16px/1.55 var(--f-body);
  padding-left: max(16px, env(safe-area-inset-left, 0px)); padding-right: max(16px, env(safe-area-inset-right, 0px)); padding-block: 28px 64px }
.wrap { max-width: 980px; margin: 0 auto; display: grid; gap: 44px }
h1, h2 { font-family: var(--f-display); font-weight: 800; line-height: 1; margin: 0; text-wrap: balance; text-transform: uppercase; letter-spacing: .01em }
h1 { font-size: clamp(56px, 11vw, 112px) }
h2 { font-size: clamp(30px, 5vw, 42px) }
p { margin: 0; max-width: 66ch }
.eyebrow { font: 500 12px/1.2 var(--f-mono); letter-spacing: .12em; text-transform: uppercase; color: var(--steel) }
.mast { display: grid; gap: 18px; border-top: 6px solid var(--ink); padding-top: 18px }
.mast .stripe { height: 10px; background: repeating-linear-gradient(-45deg, var(--amber) 0 12px, transparent 12px 24px); max-width: 280px }
.facts { display: flex; flex-wrap: wrap; gap: 10px 28px; font: 500 13px/1.3 var(--f-mono); color: var(--steel) }
.facts b { color: var(--ink); font-weight: 500 }
.status { border-left: 3px solid var(--amber); padding: 2px 0 2px 14px; color: var(--steel) }
section { display: grid; gap: 14px }
.head { display: flex; flex-wrap: wrap; align-items: baseline; justify-content: space-between; gap: 8px 20px; border-bottom: 2px solid var(--ink); padding-bottom: 8px }
.rows { display: grid; border-top: 1px solid var(--rule) }
.row { display: grid; grid-template-columns: auto minmax(0, 1fr) 200px; gap: 6px 16px; align-items: center; padding: 12px 0; border-bottom: 1px solid var(--rule) }
.row .txt { display: grid; gap: 2px; min-width: 0 }
.row .name { font: 500 14px/1.2 var(--f-mono) }
.rec { color: var(--steel) !important }
.row .name em { font-style: normal; color: var(--amber); margin-left: 8px; font-size: 11px; letter-spacing: .1em; text-transform: uppercase }
.row .what { color: var(--steel); font-size: 14.5px }
.row .from { color: var(--steel); font: 400 12px/1.4 var(--f-mono); overflow-wrap: anywhere }
.toc { display: flex; flex-wrap: wrap; gap: 8px 18px; font: 500 13px/1.3 var(--f-mono) }
.toc a { color: var(--wave) }
.row canvas { width: 200px; height: 40px; display: block; cursor: pointer }
button.play { appearance: none; border: 1.5px solid var(--ink); background: transparent; color: var(--ink); font: 600 13px/1 var(--f-mono);
  letter-spacing: .08em; text-transform: uppercase; padding: 9px 10px; min-width: 76px; border-radius: 2px; cursor: pointer }
button.play[aria-pressed="true"] { background: var(--amber); border-color: var(--amber); color: var(--amber-ink) }
button.play:focus-visible { outline: 2px solid var(--amber); outline-offset: 2px }
.master { font: 500 13px/1.3 var(--f-mono); color: var(--wave); text-decoration: underline; text-underline-offset: 3px; justify-self: start }
footer { color: var(--steel); font-size: 14px; display: grid; gap: 6px; border-top: 1px solid var(--rule); padding-top: 14px }
@media (max-width: 620px) { .row { grid-template-columns: auto minmax(0, 1fr) } .row canvas { grid-column: 1 / 3; width: 100% } }
</style>
"""

SCRIPT = """<script>
const SOUNDS = __DATA__;
const GROUPS = __GROUPS__;
const fmt = (s) => s.toFixed(1) + ' s';
let current = null;
const stop = () => { if (current) { current.audio.pause(); current.audio.currentTime = 0; current.btn.setAttribute('aria-pressed', 'false'); current.btn.textContent = 'Play'; current = null; } };
const canvases = [];
const root = document.getElementById('groups');
for (const [g, title] of Object.entries(GROUPS)) {
  const list = SOUNDS.filter((s) => s.group === g);
  if (!list.length) continue;
  const sec = document.createElement('section');
  sec.id = 'g-' + g;
  sec.innerHTML = `<div class="head"><h2>${title}</h2><span class="eyebrow">${list.length} ${list.length === 1 ? 'cue' : 'cues'}</span></div><div class="rows"></div>`;
  root.appendChild(sec);
  const rows = sec.querySelector('.rows');
  for (const s of list) {
    const row = document.createElement('div');
    row.className = 'row';
    row.innerHTML = `<button class="play" type="button" id="play-${s.name}" aria-pressed="false" aria-label="Play ${s.name}">Play</button>
      <div class="txt"><span class="name">${s.name}${s.recorded ? '<em class="rec">rec</em>' : '<em class="rec">synth</em>'}${s.loop ? '<em>loop</em>' : ''}${s.stereo ? '<em>stereo</em>' : ''} · ${fmt(s.seconds)}</span><span class="what">${s.what}</span>${(s.sources || []).length ? `<span class="from">From: ${s.sources.join('; ')}</span>` : ''}</div>
      <canvas aria-hidden="true"></canvas>`;
    rows.appendChild(row);
    const audio = new Audio();
    audio.preload = 'auto';
    audio.src = s.src;
    audio.loop = s.loop;
    const btn = row.querySelector('button');
    const me = { audio, btn };
    const play = () => { if (current === me) { stop(); return; } stop(); current = me; audio.currentTime = 0; audio.play().catch(() => {}); btn.setAttribute('aria-pressed', 'true'); btn.textContent = s.loop ? 'Stop' : 'Playing'; };
    btn.addEventListener('click', play);
    row.querySelector('canvas').addEventListener('click', play);
    audio.addEventListener('ended', () => { if (current === me) stop(); });
    canvases.push({ s, cv: row.querySelector('canvas') });
  }
}
function draw() {
  const cs = getComputedStyle(document.documentElement);
  const wave = cs.getPropertyValue('--wave').trim(), rule = cs.getPropertyValue('--rule').trim();
  for (const { s, cv } of canvases) {
    const w = cv.clientWidth, h = cv.clientHeight, dpr = window.devicePixelRatio || 1;
    cv.width = Math.round(w * dpr); cv.height = Math.round(h * dpr);
    const g = cv.getContext('2d'); g.setTransform(dpr, 0, 0, dpr, 0, 0); g.clearRect(0, 0, w, h);
    g.fillStyle = rule; g.fillRect(0, h / 2, w, 1);
    g.fillStyle = wave;
    const bw = w / s.peaks.length;
    s.peaks.forEach((p, i) => { const ph = Math.max(1, p * (h - 4)); g.fillRect(i * bw, (h - ph) / 2, Math.max(1, bw - 0.6), ph); });
  }
}
draw();
addEventListener('resize', draw);
try { matchMedia('(prefers-color-scheme: dark)').addEventListener('change', draw); } catch (e) {}
new MutationObserver(draw).observe(document.documentElement, { attributes: true, attributeFilter: ['data-theme'] });
</script>
"""


def files_map(items: list, page_dir: str, repo: str) -> dict:
    """The Artifact publish `files` map for a page's sounds: each published path is the path the page
    names (relative to the page, without its fingerprint), and its source the file's repository path."""
    import os
    return {i["src"].split("?")[0]: os.path.relpath(os.path.join(page_dir, i["src"].split("?")[0]), repo)
            for i in items}


def fingerprint(items: list, root: str) -> None:
    """Adds ?v=<first 10 hex of the file's SHA-256> to each item's src, so a sound republished at the same path
    is fetched afresh rather than played from the viewer's cache (2026-11-07: the bridge bed was changed six times
    at one path)."""
    import hashlib
    import os
    for i in items:
        path = i["src"].split("?")[0]
        with open(os.path.join(root, path), "rb") as f:
            i["src"] = path + "?v=" + hashlib.sha256(f.read()).hexdigest()[:10]


def peaks(path: str, points: int = 120) -> list:
    """A peak outline of any audio file for the waveform strip, scaled to its loudest point."""
    raw = subprocess.run(["ffmpeg", "-v", "error", "-i", path, "-ac", "1", "-ar", "8000", "-f", "f32le", "-"],
                         capture_output=True, check=True).stdout
    x = np.abs(np.frombuffer(raw, np.float32))
    x = x[: len(x) // points * points].reshape(points, -1).max(axis=1)
    return [round(float(p), 3) for p in x / (x.max() or 1.0)]


def write(out: str, title: str, body: str, groups: dict, items: list) -> None:
    """The page at `out`: `body` is the masthead and footer HTML (with an empty #groups div),
    `groups` maps each item's group to its heading, in order, and each item has name, group, what,
    seconds, loop, peaks and src (a path relative to the page), and optionally recorded, stereo
    and sources.
    Writes the publish files map beside the page (<page>.files.json)."""
    import os
    page_dir = os.path.dirname(os.path.abspath(out))
    repo = os.path.dirname(os.path.dirname(page_dir))
    fingerprint(items, page_dir)
    page = (HEAD.replace("__TITLE__", title) + body
            + SCRIPT.replace("__GROUPS__", json.dumps(groups)).replace("__DATA__", json.dumps(items)))
    with open(out, "w") as f:
        f.write(page)
    with open(out[:-5] + ".files.json", "w") as f:
        json.dump(files_map(items, page_dir, repo), f, indent=1)
        f.write("\n")
