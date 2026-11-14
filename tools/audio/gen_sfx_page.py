#!/usr/bin/env python3
"""Writes the Pale Blue Dot Sounds page (docs/mockups/sounds.html) from output/sfx/sfx.json.

Ported from Fleet Command (tools/audio/gen_sfx_page.py at 08374c4, itself after Crab Breeze's at
0e569e3). Every sound tools/audio/render_sfx.sh makes, grouped by where the player hears it (on
foot, digging, in the water, fishing, in a craft, weather, night, menus, and the ambience beds),
each with what it is, where it came from and a play button, and the link back to the mockup tracker
(CLAUDE.md, "The mockup tracker"). The page's look and player are tools/audio/audio_page.py. The
MP3s sit beside the page (docs/mockups/sounds/) and docs/mockups/sounds.files.json is the publish's
`files` map.

Usage: python3 tools/audio/gen_sfx_page.py   (after tools/audio/render_sfx.sh)
"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import audio_page  # noqa: E402  (the page's one look and player)

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
TRACKER = "https://claude.ai/artifact/EwNcNzHQU3B7C3jyi629Wk"

# page sections, in order: (key, heading)
SECTIONS = {
    "foot": "On foot (placed at the feet)",
    "dig": "Digging and building (placed at the block)",
    "water": "In the water (placed)",
    "fishing": "Fishing (placed at the rod or the float)",
    "craft": "Craft (placed at the craft's parts)",
    "weather": "Weather: rain, thunder and fire (placed)",
    "night": "Night (beds and scatters)",
    "menus": "Menus and HUD (not placed)",
    "beds_air": "Beds: wind, rain and the sea (stereo, not placed)",
    "beds_day": "Beds: day, by biome (stereo, not placed)",
    "beds_place": "Beds: towns, caves and under water (stereo, not placed)",
}

GROUP_SECTION = {"Steps": "foot", "Ground": "dig", "Water": "water", "Fishing": "fishing", "Craft": "craft",
                 "Weather": "weather", "Fauna": "beds_day", "Menu": "menus"}


def section(s: dict) -> str:
    """Where a sound goes on the page: by its group, except that night's beds and scatters have their
    own section, and the beds are split by what drives them (design section 5)."""
    cue = s.get("cue", s["name"])
    if "night" in cue or cue in ("owl",):
        return "night"
    if s["group"] == "Ambience":
        if cue.startswith(("amb_wind", "amb_rain", "amb_surf", "amb_open_water")):
            return "beds_air"
        if cue.startswith(("amb_town", "amb_cave", "amb_underwater")):
            return "beds_place"
        return "beds_day"
    return GROUP_SECTION.get(s["group"], "menus")


BODY = """<div class="wrap">
  <header class="mast">
    <div class="stripe" aria-hidden="true"></div>
    <a class="master" href="__TRACKER__" target="_blank" rel="noopener">All Pale Blue Dot mockups (the tracker)</a>
    <span class="eyebrow">Pale Blue Dot sounds · first set · __DATE__</span>
    <h1>Pale Blue Dot Sounds</h1>
    <p>The planned sounds of the planet, before any of them is in the game: __COUNT__ files for __CUES__ cues and beds. Footsteps and digs for every ground the player can stand on, swimming, fishing, the craft, rain, thunder near and far, fire, the menus, and the ambience beds that tell a meadow at noon from a jungle at night, a beach, a town, a cave or the sea from under it. __REC__ are cut from real recordings (CC0 and public domain only: OpenGameArt, Kenney, Freesound's CC0 sounds), slowed, layered and filtered to fit; __SYN__ are synthesised where nothing recorded serves (menu tones, the stall warning). Placed sounds are mono and will be heard from where they happen; beds are stereo and play under everything, quiet and far.</p>
    <div class="facts"><span>Files <b>__COUNT__</b></span><span>Recorded <b>__REC__</b></span><span>Synthesised <b>__SYN__</b></span><span>Beds <b>__BEDS__</b></span><span>Game format <b>48 kHz OGG Vorbis</b></span><span>Here <b>MP3 copies</b></span><span>Licence <b>CC0, public domain and ours</b></span></div>
    <p class="status">A mockup: nothing here plays in the game yet (openspec/changes/audio-and-ambience, task 2.3). Listen and keep or send back each take; a cue with several takes (step_grass_1 to _6) is played one take at a time in the game, never the same twice running. The levels between cues are set by the game's data, not here: every short cue is written at full level, so a menu tone or a bed here is as loud as it will ever be against its neighbours on the page, not in play. Claude cannot listen; these were chosen by measurement and a spectrogram sheet, and only your ears approve them.</p>
    <nav class="toc" aria-label="Sections">__TOC__</nav>
  </header>
  <div id="groups" style="display:grid;gap:44px"></div>
  <footer>
    <span>Cuts: tools/audio/cut_cc0.py to tools/audio/cc0_cues.json · Synthesis: tools/audio/gen_sfx.py · Encode: tools/audio/render_sfx.sh · Game copies: assets/audio/sfx/*.ogg, assets/audio/ambience/*.ogg · Plan: openspec/changes/audio-and-ambience</span>
    <span>Credits: __CREDITS__</span>
    <span>Every pack and its licence: AUDIO_SOURCES.md · Each cue's files and edits: assets/audio/sfx/SOURCES.md, assets/audio/ambience/SOURCES.md · Method: the sound-effects skill</span>
  </footer>
</div>
"""


def main() -> int:
    path = os.path.join(REPO, "output", "sfx", "sfx.json")
    data = json.load(open(path)) if os.path.exists(path) else []
    if not data:
        print("no sounds yet: the Sounds page is not written")
        return 0
    page = os.path.join(REPO, "docs", "mockups", "sounds.html")
    for s in data:
        s["src"] = f"sounds/{s['name']}.mp3"
        if not os.path.exists(os.path.join(os.path.dirname(page), s["src"])):
            raise SystemExit(f"docs/mockups/{s['src']} is missing: run tools/audio/render_sfx.sh")
        s["group"] = section(s)
    order = list(SECTIONS)
    data.sort(key=lambda s: (order.index(s["group"]), s.get("_order", 0)))
    rec = sum(1 for s in data if s.get("recorded"))
    beds = sum(1 for s in data if s["name"].startswith("amb_"))
    cues = len({s.get("cue", s["name"]) for s in data})
    used_sections = [k for k in SECTIONS if any(s["group"] == k for s in data)]
    toc = " ".join(f'<a href="#g-{k}">{SECTIONS[k].split(" (")[0]}</a>' for k in used_sections)
    import datetime
    body = (BODY.replace("__TRACKER__", TRACKER).replace("__COUNT__", str(len(data))).replace("__REC__", str(rec))
            .replace("__SYN__", str(len(data) - rec)).replace("__BEDS__", str(beds)).replace("__CUES__", str(cues))
            .replace("__TOC__", toc).replace("__DATE__", datetime.date.today().isoformat()))
    # the credit any non-CC0 pack in use asks for (tools/audio/licence_exceptions.json), read from the records
    lic = json.load(open(os.path.join(REPO, "tools", "audio", "cc0_licences.json")))
    cc0 = json.load(open(os.path.join(REPO, "tools", "audio", "cc0_cues.json")))
    used = {l["src"].split(":")[0] for k, cue in cc0.items() if not k.startswith("_")
            for l in (list(cue["layers"]) + [p for grp in cue.get("scatter", []) for p in grp["pick"]]
                      + [x for t in cue.get("takes", []) for x in t.get("layers", [])]) if "src" in l}
    credits = [f"{lic[k]['attribution']} ({', '.join(lic[k]['licences'])}): {k.split('/', 1)[1].replace('-', ' ')}"
               for k in sorted(used) if lic.get(k, {}).get("attribution")]
    body = body.replace("__CREDITS__", (" · ".join(credits)) if credits else "none needed: every recording is CC0 or public domain")
    audio_page.write(page, "Pale Blue Dot Sounds", body, SECTIONS, data)
    print(f"wrote {os.path.relpath(page, REPO)}: {len(data)} sounds; files map docs/mockups/sounds.files.json")
    return 0


if __name__ == "__main__":
    sys.exit(main())
