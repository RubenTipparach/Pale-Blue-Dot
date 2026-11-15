#!/usr/bin/env python3
"""Range maps for the fish of Pale Blue Dot, drawn off the measured world.

Reads the fields `cargo run --profile fast -p pbd-core --example fish_ranges`
writes (depth, rivers, and the water temperature over a simulated year) and
applies the habitat rules of `assets/config/fauna.ron`
(`openspec/changes/fishing-and-equipment/design.md` section 7) to draw one
greyscale range map per species, in the manner of a field guide:

- land is mid grey and water pale grey, shaded by depth;
- water frozen all year is hatched;
- where a species lives all year is solid red, and where it lives for only
  part of the year is pale orange.

It also draws the water classes, the water temperature and the species count,
and prints the table the design quotes.

    python3 tools/fish_ranges.py OUT_DIR [--page TEMPLATE.html] \
        ID:LABEL:FIELDS.bin[:RUN.log] ...

One scenario per field file (a run of the instrument). Every simulated year of
it is drawn into OUT_DIR/ID/y1, OUT_DIR/ID/y2, ...; the run's own log, when
given, supplies the sea temperature over time. With a page template,
OUT_DIR/index.html is written with the measured numbers baked in.

The rules are the game's: the water limits and every species' classes and
temperature window are read from `assets/config/fauna.ron`, so the atlas is
drawn from the roster the spawner uses.
"""

import base64
import json
import os
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from ron_read import load as _load_ron  # noqa: E402

FAUNA = os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
                     "assets", "config", "fauna.ron")


def roster(path=FAUNA, body="home"):
    """The rules, read from the game's own roster (`fauna.ron`): the water
    limits and, per species in roster order, the water classes it lives in and
    its temperature window. A species' `env` tags narrow where it spawns
    further (reefs, kelp, vents ...); the range maps draw the class and
    temperature envelope those places sit inside, and list the tags."""
    fauna = _load_ron(path)
    water = fauna["water"]
    species = [
        dict(
            id=sp["id"],
            name=sp["name"],
            water=[w.lower() for w in sp["water"]],
            temp=tuple(sp["temp_c"]),
            kind=sp.get("class", "Fish"),
            env=list(sp.get("env") or []),
            origin="Tenebris" if isinstance(sp["from"], dict) else "new",
            bed=bool(sp["bed"]),
            icon=f"{sp['icon']}.png",
        )
        for sp in fauna["bodies"][body]
    ]
    return water, species


_WATER, SPECIES = roster()
# Water temperature at which the sea surface freezes, deg C.
FREEZE_C = _WATER["freezes_c"]
# Water classes by depth, m. Shallows are what a cast from the shore reaches.
SHALLOW_M = _WATER["shallows_max_m"]
SHELF_M = _WATER["shelf_max_m"]
# The fish, the species a rod takes: the world map packs these alone.
FISH = [sp for sp in SPECIES if sp["kind"] == "Fish"]

# Greys for the ground, and the two range colours.
LAND = np.array([178, 178, 178])
COAST = np.array([96, 96, 96])
WATER = {  # by class
    "river": np.array([236, 236, 236]),
    "shallows": np.array([244, 244, 244]),
    "shelf": np.array([232, 232, 232]),
    "deep": np.array([216, 216, 216]),
}
ICE = np.array([250, 250, 250])
ICE_HATCH = np.array([200, 200, 200])
GRATICULE = np.array([150, 150, 150])
RESIDENT = np.array([215, 48, 31])
SEASONAL = np.array([253, 174, 107])


def load(path):
    raw = open(path, "rb").read()
    if raw[:8] != b"PBDFISH1":
        raise SystemExit(f"{path} is not a fish_ranges field file")
    width, height, planes = np.frombuffer(raw[8:20], "<u4")
    data = np.frombuffer(raw[20:], "<f4").reshape(planes, height, width)
    return data


def classes(alt, alt_no_rivers):
    water = alt < 0.0
    river = water & (alt_no_rivers >= 0.0)
    depth = -alt
    sea = water & ~river
    return {
        "river": river,
        "shallows": sea & (depth <= SHALLOW_M),
        "shelf": sea & (depth > SHALLOW_M) & (depth <= SHELF_M),
        "deep": sea & (depth > SHELF_M),
    }, water


def area_weights(height, width):
    lat = (0.5 - (np.arange(height) + 0.5) / height) * np.pi
    return np.cos(lat)[:, None] * np.ones((1, width))


def base_map(alt, water, cls, frozen):
    h, w = alt.shape
    img = np.zeros((h, w, 3))
    img[~water] = LAND
    # A little relief on the land, lit from the north-west, so the continents
    # read as ground rather than as a flat stencil.
    gy, gx = np.gradient(np.where(water, 0.0, alt))
    shade = np.clip(1.0 - 0.012 * (gx - gy), 0.85, 1.12)
    # Three steps of shade, not a ramp: a map, not a photograph, and a
    # quarter of the file.
    shade = np.round((shade - 1.0) / 0.08) * 0.08 + 1.0
    img[~water] = (img[~water] * shade[~water][:, None]).clip(0, 255)
    for name, mask in cls.items():
        img[mask] = WATER[name]
    rows, cols = np.indices((h, w))
    hatch = ((rows + cols) % 6) == 0
    img[frozen] = ICE
    img[frozen & hatch] = ICE_HATCH
    # Coast: a water pixel with land beside it.
    land = ~water
    edge = np.zeros_like(water)
    for dy, dx in [(0, 1), (0, -1), (1, 0), (-1, 0)]:
        edge |= np.roll(np.roll(land, dy, 0), dx, 1)
    img[water & edge & ~cls["river"]] = COAST
    # Graticule every 30 degrees.
    for k in range(1, 6):
        img[int(h * k / 6), ::3] = GRATICULE
    for k in range(1, 12):
        img[::3, int(w * k / 12)] = GRATICULE
    return img


def thicken(mask):
    """One pixel wider, so a range along a river channel can be seen."""
    out = mask.copy()
    for dy, dx in [(0, 1), (0, -1), (1, 0), (-1, 0)]:
        out |= np.roll(np.roll(mask, dy, 0), dx, 1)
    return out


def save(img, path, scale=1):
    im = Image.fromarray(img.astype(np.uint8), "RGB")
    if scale != 1:
        im = im.resize((im.width // scale, im.height // scale), Image.NEAREST)
    im.quantize(colors=32, method=Image.Quantize.MEDIANCUT).save(path, optimize=True)


def sea_series(log_path):
    """The instrument's own progress lines: (day, mean sea surface C)."""
    series = []
    for line in open(log_path):
        if line.startswith("day ") and "mean sea surface" in line:
            day = int(line.split(":")[0].split()[1])
            series.append((day, float(line.split("mean sea surface")[1].split()[0])))
    return series


def main():
    args = sys.argv[1:]
    if not args:
        raise SystemExit(__doc__)
    root, page = args[0], None
    rest = args[1:]
    if rest[:1] == ["--page"]:
        page, rest = rest[1], rest[2:]
    scenarios = []
    for spec in rest:
        parts = spec.split(":")
        sid, label, fields = parts[0], parts[1], parts[2]
        drawn = []
        # A run with no field file (stopped early) still has a log: it is
        # charted and not mapped.
        if fields:
            data = load(fields)
            years = (data.shape[0] - 2) // 3
            print(f"== {sid}: {label}")
            drawn = [draw_year(data, f"{root}/{sid}/y{y}", y, years) for y in range(1, years + 1)]
        scenarios.append(dict(id=sid, label=label, years=drawn,
                              sea=sea_series(parts[3]) if len(parts) > 3 else []))
    json.dump(scenarios, open(f"{root}/summary.json", "w"), indent=1)
    # The page draws each species' own 16 px icon, read from the game's items.
    icons = {}
    items = os.path.join(os.path.dirname(FAUNA), "..", "items")
    for sp in SPECIES:
        with open(os.path.join(items, sp["icon"]), "rb") as f:
            icons[sp["id"]] = base64.b64encode(f.read()).decode()
    if page:
        text = open(page).read()
        marker = "/*SUMMARY*/null"
        if marker not in text:
            raise SystemExit(f"{page} has no {marker} to fill")
        text = text.replace(marker, json.dumps(scenarios))
        text = text.replace("/*ICONS*/null", json.dumps(icons))
        open(f"{root}/index.html", "w").write(text)


def draw_year(data, out, year, years):
    os.makedirs(out, exist_ok=True)
    alt, alt0 = data[0], data[1]
    t_mean, t_cold, t_warm = data[2 + 3 * (year - 1) : 5 + 3 * (year - 1)]
    cls, water = classes(alt, alt0)
    frozen = water & (t_warm < FREEZE_C)
    open_part = water & (t_cold < FREEZE_C) & ~frozen
    area = area_weights(*alt.shape)
    water_area = (area * water).sum()

    base = base_map(alt, water, cls, frozen)
    save(base, f"{out}/base.png")

    summary = {"year": year, "years": years, "species": [], "classes": {}}
    for name, mask in cls.items():
        summary["classes"][name] = float((area * mask).sum() / water_area * 100)
    summary["frozen_all_year"] = float((area * frozen).sum() / water_area * 100)
    summary["frozen_part_year"] = float((area * open_part).sum() / water_area * 100)

    richness = np.zeros(alt.shape, int)
    for sp in SPECIES:
        lo, hi = sp["temp"]
        home = np.zeros_like(water)
        for name in sp["water"]:
            home |= cls[name]
        home &= ~frozen
        # Resident: the whole year's daily means sit inside the window, and
        # the water never freezes. Seasonal: some of the year does.
        resident = home & (t_cold >= lo) & (t_warm <= hi) & (t_cold >= FREEZE_C)
        seasonal = home & (t_warm >= lo) & (t_cold <= hi) & ~resident
        richness += resident | seasonal
        img = base.copy()
        river_only = sp["water"] == ["river"]
        s_draw, r_draw = seasonal, resident
        if "river" in sp["water"]:
            s_draw = seasonal | (thicken(seasonal & cls["river"]) & ~water)
            r_draw = resident | (thicken(resident & cls["river"]) & ~water)
        img[s_draw] = SEASONAL
        img[r_draw] = RESIDENT
        save(img, f"{out}/{sp['id']}.png")
        res_share = float((area * resident).sum() / water_area * 100)
        sea_share = float((area * seasonal).sum() / water_area * 100)
        lat = (0.5 - (np.arange(alt.shape[0]) + 0.5) / alt.shape[0]) * 180
        present = (resident | seasonal).any(axis=1)
        band = (float(lat[present].min()), float(lat[present].max())) if present.any() else None
        summary["species"].append(
            dict(sp, resident=res_share, seasonal=sea_share, latitudes=band, river_only=river_only)
        )

    # Species count, in greys darkening with the count, on the same ground.
    img = base.copy()
    ramp = [None, (254, 229, 217), (252, 187, 161), (252, 146, 114), (251, 106, 74),
            (222, 45, 38), (165, 15, 21), (103, 0, 13), (60, 0, 8)]
    for n in range(1, len(SPECIES) + 1):
        img[(richness == n)] = ramp[min(n, len(ramp) - 1)]
    save(img, f"{out}/richness.png")
    summary["richness_max"] = int(richness.max())

    # Water classes, drawn in four greys plus rivers in black so they show.
    img = base.copy()
    img[cls["shallows"] & ~frozen] = (205, 205, 205)
    img[cls["shelf"] & ~frozen] = (230, 230, 230)
    img[cls["deep"] & ~frozen] = (250, 250, 250)
    img[cls["river"] & ~frozen] = (80, 80, 80)
    save(img, f"{out}/classes.png")

    # Water temperature, the year's mean, in a cold-to-hot ramp on the water.
    img = base.copy()
    t = np.clip((t_mean + 20.0) / 50.0, 0.0, 1.0)
    cold = np.array([49, 54, 149]); mid = np.array([245, 245, 245]); hot = np.array([165, 0, 38])
    rgb = np.where(t[..., None] < 0.5,
                   cold + (mid - cold) * (t[..., None] / 0.5),
                   mid + (hot - mid) * ((t[..., None] - 0.5) / 0.5))
    img[water] = rgb[water]
    img[frozen & ((np.indices(alt.shape).sum(0) % 6) == 0)] = ICE_HATCH
    save(img, f"{out}/temperature.png")
    swing = t_warm - t_cold
    summary["swing_c"] = [float(v) for v in np.percentile(swing[water & ~frozen], [10, 50, 90])] if (water & ~frozen).any() else [0.0, 0.0, 0.0]
    summary["mean_c"] = [float(v) for v in np.percentile(t_mean[water & ~frozen], [5, 50, 95])] if (water & ~frozen).any() else None
    summary["water_mean_c"] = float((area * t_mean)[water].sum() / water_area)

    print(f"year {year} of {years}; water classes, share of water:",
          ", ".join(f"{k} {v:.1f}%" for k, v in summary["classes"].items()),
          f"; frozen all year {summary['frozen_all_year']:.1f}%, part of it {summary['frozen_part_year']:.1f}%")
    print("seasonal swing of the daily-mean water temperature, 10/50/90th pct:",
          " / ".join(f"{v:.2f}" for v in summary["swing_c"]), "C")
    print("| Species | Water | Window, C | Resident, % of water | Part of the year, % | Latitudes |")
    print("| --- | --- | --- | ---: | ---: | --- |")
    for s in summary["species"]:
        band = s["latitudes"]
        lat = f"{band[0]:+.0f} to {band[1]:+.0f}" if band else "none"
        print(f"| {s['name']} | {', '.join(s['water'])} | {s['temp'][0]} to {s['temp'][1]} | "
              f"{s['resident']:.1f} | {s['seasonal']:.1f} | {lat} |")
    return summary


if __name__ == "__main__":
    main()
