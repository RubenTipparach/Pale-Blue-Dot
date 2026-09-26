#!/usr/bin/env python3
"""Range maps for the fish of Pale Blue Dot, drawn off the measured world.

Reads the fields `cargo run --release -p pbd-core --example fish_ranges`
writes (depth, rivers, and the water temperature over a simulated year) and
applies the habitat rules proposed in
`openspec/changes/fishing-and-equipment/design.md` section 7 to draw one
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

The rules live HERE only for the proposal. When the change is built they move
to `assets/config/fauna.ron`, and this script reads them from there.
"""

import json
import os
import sys

import numpy as np
from PIL import Image

# Water temperature at which the sea surface freezes, deg C.
FREEZE_C = -1.8
# Water classes by depth, m. Shallows are what a cast from the shore reaches.
SHALLOW_M = 6.0
SHELF_M = 40.0

# The proposed rules: the water a species lives in and the temperature window
# its water has to be inside, deg C. "river" is a channel the carve cut below
# the sea; the rest are sea by depth.
SPECIES = [
    dict(id="minnow", name="Minnow", water=["river", "shallows"], temp=(10, 30)),
    dict(id="silverfin", name="Silverfin", water=["shallows", "shelf"], temp=(-1.8, 17)),
    dict(id="perch", name="Banded perch", water=["river"], temp=(-1.8, 24)),
    dict(id="ray", name="Ray", water=["shallows", "shelf"], temp=(16, 32)),
    dict(id="eel", name="Eel", water=["river", "shallows"], temp=(6, 28)),
    dict(id="reef", name="Reef fish", water=["shallows"], temp=(23, 32)),
    dict(id="deepback", name="Deepback", water=["shelf", "deep"], temp=(-1.5, 9)),
    dict(id="serpent", name="Sea serpent", water=["deep"], temp=(-1.8, 32)),
]

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
    if page:
        text = open(page).read()
        marker = "/*SUMMARY*/null"
        if marker not in text:
            raise SystemExit(f"{page} has no {marker} to fill")
        open(f"{root}/index.html", "w").write(text.replace(marker, json.dumps(scenarios)))


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
