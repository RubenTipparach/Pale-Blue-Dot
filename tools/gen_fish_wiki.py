#!/usr/bin/env python3
"""Write the field guide's wiki page, `docs/wiki/fish.md`, from `fauna.ron`.

One section per species on every body, in roster order, with the species'
16 px icon, the entry and the angler's tip, and the same numbers the in-game
field guide shows (`pbd_app::fish::guide_facts`), computed the way the game
computes them: in single precision, and the hook window by the one formula
(`FishingSettings::hook_window`).

    python3 tools/gen_fish_wiki.py            # write the page
    python3 tools/gen_fish_wiki.py --check    # fail if the page is stale

`pbd-app`'s `the_wiki_page_holds_to_the_roster` test reads the committed page
against the roster the game loads, so a change to `fauna.ron` that is not
regenerated here fails the build's tests as well as `--check`.
"""

import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ron_read import load  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FAUNA = os.path.join(ROOT, "assets", "config", "fauna.ron")
# A species' name, entry and tip are in the locale file by its id
# (`openspec/changes/localization`): `species.<id>.name`, `.entry`, `.tip`.
LOCALE = os.path.join(ROOT, "assets", "locale", "locale_en.yaml")


def with_words(fauna):
    """The roster with each species' words from the English locale file."""
    import yaml  # PyYAML

    with open(LOCALE, encoding="utf-8") as f:
        words = yaml.safe_load(f)["species"]
    for roster in fauna["bodies"].values():
        for sp in roster:
            w = words[sp["id"]]
            sp["name"] = w["name"]
            sp["guide"] = {"entry": w["entry"], "tip": w["tip"]}
    return fauna
OUT = os.path.join(ROOT, "docs", "wiki", "fish.md")

# Display names of the enums, as the game spells them.
WATER = {"River": "rivers", "Shallows": "shallows", "Shelf": "shelf", "Deep": "deep"}
CLASS = {
    "Mammal": "mammal",
    "Bird": "bird",
    "ReptileAmphibian": "reptile",
    "Fish": "fish",
    "Arthropod": "crustacean",
    "MolluscSoft": "mollusc or soft-bodied",
}
PLACE = {
    "Open": "open water",
    "Coast": "coasts",
    "Reef": "reefs",
    "Kelp": "kelp",
    "Vent": "deep vents",
    "UnderIce": "under the ice",
    "Pool": "rock pools",
    "Riffle": "riffles",
    "Marsh": "marsh",
    "Estuary": "estuaries",
    "Mudflat": "mudflats",
    "Trench": "trenches",
}
BODY = {"home": "Pale Blue Dot"}


def f32(x):
    """The value as the game holds it: rounded to single precision."""
    return struct.unpack("<f", struct.pack("<f", float(x)))[0]


def fmt(x, places):
    return f"{f32(x):.{places}f}"


def hook_window(fishing, strength):
    """`FishingSettings::hook_window`, step by step in f32."""
    steps = f32(max(strength, 1)) - 1.0
    window = f32(fishing["hook_window_s"]) * f32(1.0 - f32(steps * f32(fishing["hook_step"])))
    return max(f32(window), f32(fishing["hook_floor_s"]))


def facts(sp, fishing):
    """`pbd_app::fish::guide_facts`, line for line."""
    lo, hi = sp["depth_m"]
    depth = (f"on the bed, {fmt(lo, 0)}-{fmt(hi, 0)} m" if sp["bed"]
             else f"{fmt(lo, 0)}-{fmt(hi, 0)} m down")
    strength = sp["strength"]
    pips = "#" * strength + "-" * (5 - min(strength, 5))
    return [
        ("WATER", ", ".join(WATER[w] for w in sp["water"])),
        ("TEMPERATURE", f"{fmt(sp['temp_c'][0], 0)} to {fmt(sp['temp_c'][1], 0)} C"),
        ("DEPTH", depth),
        ("FOUND", f"schools of {sp['school'][0]}-{sp['school'][1]}"),
        ("LENGTH", f"about {f32(f32(sp['length_m']) * f32(100.0)):.0f} cm"),
        ("STRENGTH", f"{pips} {strength}/5"),
        ("HOOK WINDOW", f"{hook_window(fishing, strength):.2f} s"),
        ("SPEED", f"{fmt(sp['speed_mps'][0], 1)}-{fmt(sp['speed_mps'][1], 1)} m/s"),
    ]


def can_bite(sp):
    """`Species::can_bite`: a schooling fish, not protected, that bites."""
    return (sp.get("class", "Fish") == "Fish" and sp.get("status", "WildOnly") != "Protected"
            and sp.get("mover", "School") == "School" and sp["bite"] > 0.0)


def chips(sp):
    out = []
    origin = sp["from"]
    if isinstance(origin, dict) and origin.get("$variant") == "Tenebris":
        out.append(f"from Tenebris ({origin['$fields'][0]})")
    else:
        out.append("new")
    out.append(CLASS.get(sp.get("class", "Fish"), sp.get("class", "Fish")))
    if sp["bed"]:
        out.append("bottom dweller")
    if sp.get("status") == "Protected":
        out.append("protected: it may not be caught")
    elif not can_bite(sp):
        out.append("not taken on a line")
    return out


def render(fauna):
    fishing = fauna["fishing"]
    lines = [
        "# The field guide",
        "",
        "<!-- Written by tools/gen_fish_wiki.py from assets/config/fauna.ron."
        " Do not edit: change the roster and run the generator. -->",
        "",
        "Everything that lives in the water, one entry per species, from the same",
        "record the schools, the hook and the in-game field guide (J) read. The",
        "numbers are the guide's own. Where each species lives on the planet is",
        "drawn in the [Fish Range Atlas](fish-ranges/index.html).",
        "",
    ]
    for body, roster in fauna["bodies"].items():
        name = BODY.get(body, body)
        lines += [f"## {name}", ""]
        lines += ["| | Species | Kind | Water | Temperature |", "| --- | --- | --- | --- | --- |"]
        for sp in roster:
            fa = dict(facts(sp, fishing))
            lines.append(
                f"| ![](../../assets/items/{sp['icon']}.png) | [{sp['name']}](#{sp['id'].replace('_', '-')}) "
                f"| {CLASS.get(sp.get('class', 'Fish'))} | {fa['WATER']} | {fa['TEMPERATURE']} |"
            )
        lines.append("")
        for sp in roster:
            lines += [
                f'<a id="{sp["id"].replace("_", "-")}"></a>',
                "",
                f"### ![](../../assets/items/{sp['icon']}.png) {sp['name']}",
                "",
                " · ".join(chips(sp)),
                "",
                sp["guide"]["entry"],
                "",
                "| | |",
                "| --- | --- |",
            ]
            for label, value in facts(sp, fishing):
                lines.append(f"| {label} | {value.replace('|', '/')} |")
            places = sp.get("env") or []
            if places:
                lines.append(f"| PLACES | {', '.join(PLACE[p] for p in places)} |")
            lines += ["", f"*Tip:* {sp['guide']['tip']}", ""]
    return "\n".join(lines).rstrip() + "\n"


def main():
    text = render(with_words(load(FAUNA)))
    if "--check" in sys.argv[1:]:
        current = open(OUT, encoding="utf-8").read() if os.path.exists(OUT) else ""
        if current != text:
            raise SystemExit(f"{os.path.relpath(OUT, ROOT)} is stale: run tools/gen_fish_wiki.py")
        print("fish wiki matches fauna.ron")
        return
    os.makedirs(os.path.dirname(OUT), exist_ok=True)
    with open(OUT, "w", encoding="utf-8", newline="\n") as f:
        f.write(text)
    print(f"wrote {os.path.relpath(OUT, ROOT)}")


if __name__ == "__main__":
    main()
