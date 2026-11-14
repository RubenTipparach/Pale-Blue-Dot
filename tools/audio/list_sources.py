#!/usr/bin/env python3
"""Writes AUDIO_SOURCES.md at the repository root: every recording the game's sounds are cut from.

One row per pack: its page, who recorded or uploaded it, its licence as the page stated it when
tools/audio/fetch_cc0.py read it, and the cues cut from it. Built from tools/audio/cc0_cues.json
(which pack each cue uses) and tools/audio/cc0_licences.json (the fetcher's licence report, copied
from <dest>/licences.json), so the list cannot drift from what is actually used.

Ported from Fleet Command (tools/audio/list_sources.py at 08374c4, itself after Crab Breeze's at
0e569e3). Changed here: Freesound sounds are listed (one row per sound, by its title), a take's layers
count, and there is no music line yet (no music change has landed).

Usage: python3 tools/audio/list_sources.py
"""
import json
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import gen_sfx as g  # noqa: E402  (the synthesised cues)

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
DOC = os.path.join(REPO, "AUDIO_SOURCES.md")


def main() -> int:
    cues = json.load(open(os.path.join(HERE, "cc0_cues.json")))
    lic = json.load(open(os.path.join(HERE, "cc0_licences.json")))
    used: dict[str, set] = {}
    synth = set()
    for name, cue in cues.items():
        if name.startswith("_"):
            continue
        for layer in (list(cue["layers"]) + [p for grp in cue.get("scatter", []) for p in grp["pick"]]
                      + [l for t in cue.get("takes", []) for l in t.get("layers", [])]):
            if "synth" in layer:
                synth.add(name)
            else:
                used.setdefault(layer["src"].split(":")[0], set()).add(name)
    rows = []
    for pack in sorted(used, key=lambda p: (p.split("/")[0], p.split("/", 1)[1])):
        info = lic.get(pack, {})
        site = {"oga": "OpenGameArt", "kenney": "Kenney", "archive": "Internet Archive",
                "freesound": "Freesound"}[pack.split("/")[0]]
        title = (info.get("title") or pack.split("/", 1)[1].replace("-", " ")).replace("|", "/")
        lics = info.get("licences", ["?"])
        licence = "public domain" if any("publicdomain/mark" in s for s in lics) else (
            "CC0" if "CC0" in lics or any("/zero/" in s for s in lics) else ", ".join(lics))
        if info.get("attribution"):
            licence += f" (credit: {info['attribution']}; asked for by the owner)"
        rows.append(f"| [{title}]({info.get('url', '')}) | {site} | {info.get('author') or '-'} | {licence} | "
                    + ", ".join(f"`{c}`" for c in sorted(used[pack])) + " |")
    body = "\n".join([
        "# Audio sources",
        "",
        "Every recording Pale Blue Dot's sounds are cut from. "
        f"{len(used)} packs, every one CC0 or public domain by its own page when `tools/audio/fetch_cc0.py` read "
        "it (a pack whose page does not say so is refused), except the few the owner asked for under another "
        "licence (`tools/audio/licence_exceptions.json`), each printed with the credit its page asks for, which "
        "the game's credits must carry. CC0 asks for no credit; the people who recorded them are named anyway.",
        "",
        "This file is written by `python3 tools/audio/list_sources.py` (run by `tools/audio/render_sfx.sh`) "
        "from `tools/audio/cc0_cues.json` and `tools/audio/cc0_licences.json`: edit those, not this. Each "
        "cue's exact files and edits are in `assets/audio/sfx/SOURCES.md` and "
        "`assets/audio/ambience/SOURCES.md`. Freesound sounds are taken from the high-quality preview "
        "Freesound serves without an account (an MP3), under the same licence as the sound.",
        "",
        "| Pack | Site | Recorded or uploaded by | Licence | Cues cut from it |",
        "| --- | --- | --- | --- | --- |",
        *rows,
        "",
        "## Not recorded",
        "",
        "- Layered with our own synthesis (`tools/audio/gen_sfx.py`): " + ", ".join(f"`{c}`" for c in sorted(synth)) + ".",
        "- Synthesised outright: " + ", ".join(f"`{n}`" for n in sorted(n for n in g.SOUNDS if not g.is_layer(n) and n not in cues)) + ".",
        "- Music: none yet (a music change, `music-authoring` skill).",
        "",
    ])
    open(DOC, "w").write(body)
    print(f"{len(used)} packs listed in {os.path.relpath(DOC, REPO)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
