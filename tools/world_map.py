#!/usr/bin/env python3
"""The world map mockup's images, drawn from the game's own rasters.

Reads what `cargo run --release -p pbd-core --example world_map` writes (the
altitude, the top block and the biome at each scale, through
`pbd_core::map::base_texel`) and draws the PNGs `docs/mockups/world-map.html`
loads (`openspec/changes/world-map` task 1.2):

- base.png: the planet as it looks (survey M3). Each pixel takes the colour
  of its top block's tile in its biome's tileset, shaded by relief; the sea
  is shaded by depth.
- biomes-today.png, biomes-<m>.png: the biome as shipped, and at each
  moisture scale with its thresholds retuned to a third each, clear over the
  sea, for the overlay that greys the base map out.
- height.png: the altitude and two biomes packed for the page's own use (the
  site rules and the readout). Red and green are the altitude plus 32,768 m,
  high byte and low byte. Blue is today's biome in the low nibble and the
  four-times biome in the high one.

    python3 tools/world_map.py WORLD_MAP.bin OUT_DIR \
        [--fish FISH_FIELDS.bin] [--weather MAP_WEATHER.bin]
    python3 tools/world_map.py tiles FINER.bin OUT_DIR LEVEL

With `--fish` (what `examples/fish_ranges.rs` writes) it also draws the
climate and fish overlays of the last simulated year, by the rules in
`tools/fish_ranges.py`, imported rather than copied:

- temperature.png: the year's mean surface temperature, land and sea, in the
  fish maps' cold-to-hot ramp;
- sea-ice.png: water frozen all year, and water frozen for part of it;
- fish.png: each species' range packed two bits a species (0 none, 1 part
  of the year, 2 all year), species 0 to 3 in red and 4 to 7 in green, and
  the count of species in blue, for the page to unpack.

With `--weather` (what `examples/map_weather.rs` writes) it draws the live
layer's frames, clouds-NN.png: white cloud with its cover as alpha, tinted
blue by how hard it rains. From that instrument's second format it also
draws every weather overlay the globe can show (`world-map` decision 8):
weather/<name>-NN.png, each overlay's value in the game's own ramp
(`pbd_app::overlay::RAMPS`, read from its source) as a palette image, cloud
and rain fading to clear as the globe's shader fades them; and for wind, the
jet and the currents, weather/<name>-flow-NN.png, the flow at half the
raster, eastward in red and northward in green about 128.

It writes the same bytes from the same input, so a regenerated map diffs to
nothing.
"""

import json
import os
import re
import struct
import sys

import numpy as np
from PIL import Image

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..")
TILESETS = os.path.join(ROOT, "assets", "tilesets")

# `pbd_core::planet_gen::Biome`, in discriminant order.
BIOMES = ["ocean", "beach", "fields", "desert", "jungle", "swamp", "mountains", "tundra"]
# `pbd_core::terrain::Material` codes the top block can be.
STONE, GRASS, WATER, ORE, SAND, DRY_GRASS, JUNGLE_GRASS, SNOW, ROCK, DIRT = (
    1, 3, 4, 5, 6, 7, 8, 9, 10, 11)
# The moisture scales the instrument writes, after today's biome plane.
SCALES = [188, 375, 750]

# The biome overlay's colours: one flat colour a biome, picked apart.
OVERLAY = {
    "beach": (232, 214, 150),
    "fields": (150, 190, 80),
    "desert": (222, 150, 60),
    "jungle": (30, 120, 60),
    "swamp": (90, 130, 110),
    "mountains": (140, 130, 125),
    "tundra": (225, 235, 245),
}


def read(path):
    with open(path, "rb") as f:
        data = f.read()
    assert data[:8] == b"PBDMAP01", "not a world_map raster"
    width, height, planes = struct.unpack_from("<III", data, 8)
    body = np.frombuffer(data, dtype="<f4", offset=20)
    return width, height, body.reshape(planes, height, width)


def tile_colour(sheet, col, row):
    """The mean colour of one tile of a biome's 4 x 4 sheet."""
    image = Image.open(os.path.join(TILESETS, sheet + ".png")).convert("RGB")
    w, h = image.size
    box = (col * w // 4, row * h // 4, (col + 1) * w // 4, (row + 1) * h // 4)
    return np.asarray(image.crop(box), dtype=np.float64).reshape(-1, 3).mean(axis=0)


def ground_colours(altitude, top, biome):
    """Each pixel's top block, coloured from its biome's tiles as the
    terrain draws it: the ground tile (0,0), the earth (2,0), the stone
    (3,0); sand from the beach or the desert, snow from the tundra."""
    colour = np.zeros(altitude.shape + (3,))
    for b, name in enumerate(BIOMES):
        here = biome == b
        if not here.any():
            continue
        ground = tile_colour(name, 0, 0)
        earth = tile_colour(name, 2, 0)
        stone = tile_colour("mountains" if name == "mountains" else name, 3, 0)
        colour[here] = ground
        colour[here & (top == DIRT)] = earth
        colour[here & np.isin(top, [STONE, ROCK, ORE])] = stone
    colour[np.isin(top, [SAND])] = tile_colour("beach", 0, 0)
    colour[(top == SAND) & (biome == BIOMES.index("desert"))] = tile_colour("desert", 0, 0)
    colour[top == SNOW] = tile_colour("tundra", 0, 0)
    colour[top == WATER] = (70, 110, 120)
    return colour


def relief(altitude, width):
    """A hillshade from the north-west, with the east-west spacing narrowed by
    the latitude so a slope reads the same at every latitude."""
    height = altitude.shape[0]
    spacing = 2 * np.pi * 4800.0 / width
    lat = (0.5 - (np.arange(height) + 0.5) / height) * np.pi
    east = np.maximum(np.cos(lat), 0.05)[:, None] * spacing
    ground = np.maximum(altitude, 0.0)
    dx = (np.roll(ground, -1, axis=1) - np.roll(ground, 1, axis=1)) / (2 * east)
    dy = np.zeros_like(ground)
    dy[1:-1] = (ground[:-2] - ground[2:]) / (2 * spacing)
    # Exaggerated three times: a planet 4.8 km across has gentle hills at
    # 15 m a pixel, and flat shading would hide them.
    nx, ny, nz = -3 * dx, -3 * dy, np.ones_like(ground)
    norm = np.sqrt(nx * nx + ny * ny + nz * nz)
    light = np.array([-1.0, 1.0, 1.4])
    light /= np.linalg.norm(light)
    shade = (nx * light[0] + ny * light[1] + nz * light[2]) / norm
    return np.clip(0.55 + 0.6 * (shade - light[2]) + 0.45, 0.45, 1.25)


def sea_colours(depth):
    """Shallows pale, the shelf mid blue, the deep dark (the fish classes'
    6 m and 40 m)."""
    stops = [(0.0, (120, 190, 200)), (6.0, (70, 150, 185)), (40.0, (35, 90, 150)),
             (150.0, (18, 45, 95)), (400.0, (10, 25, 60))]
    out = np.zeros(depth.shape + (3,))
    for (d0, c0), (d1, c1) in zip(stops, stops[1:]):
        t = np.clip((depth - d0) / (d1 - d0), 0.0, 1.0)[..., None]
        band = (depth >= d0) & (depth < d1)
        out[band] = (np.array(c0) * (1 - t) + np.array(c1) * t)[band]
    out[depth >= stops[-1][0]] = stops[-1][1]
    return out


def save(image, path):
    # Quantised to one palette, so the files stay small and the same input
    # always gives the same bytes.
    image.save(path, optimize=False)


def ramp(t):
    """The fish maps' temperature ramp, t in 0..1: blue, white, red."""
    cold = np.array([49, 54, 149]); mid = np.array([245, 245, 245]); hot = np.array([165, 0, 38])
    t = t[..., None]
    return np.where(t < 0.5, cold + (mid - cold) * (t / 0.5), mid + (hot - mid) * ((t - 0.5) / 0.5))


def climate_and_fish(fields, out):
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    import fish_ranges as fr
    data = fr.load(fields)
    years = (data.shape[0] - 2) // 3
    alt, alt0 = data[0], data[1]
    t_mean, t_cold, t_warm = data[2 + 3 * (years - 1): 5 + 3 * (years - 1)]
    cls, water = fr.classes(alt, alt0)
    h, w = alt.shape

    rgba = np.zeros((h, w, 4), dtype=np.uint8)
    rgba[..., :3] = ramp(np.clip((t_mean + 20.0) / 50.0, 0.0, 1.0)).astype(np.uint8)
    rgba[..., 3] = 255
    Image.fromarray(rgba, "RGBA").save(os.path.join(out, "temperature.png"))

    frozen = water & (t_warm < fr.FREEZE_C)
    part = water & (t_cold < fr.FREEZE_C) & ~frozen
    ice = np.zeros((h, w, 4), dtype=np.uint8)
    ice[part] = (170, 215, 240, 255)
    ice[frozen] = (245, 250, 255, 255)
    Image.fromarray(ice, "RGBA").save(os.path.join(out, "sea-ice.png"))

    packed = np.zeros((h, w, 3), dtype=np.uint8)
    for k, sp in enumerate(fr.SPECIES):
        lo, hi = sp["temp"]
        home = np.zeros_like(water)
        for name in sp["water"]:
            home |= cls[name]
        home &= ~frozen
        resident = home & (t_cold >= lo) & (t_warm <= hi) & (t_cold >= fr.FREEZE_C)
        seasonal = home & (t_warm >= lo) & (t_cold <= hi) & ~resident
        code = np.where(resident, 2, np.where(seasonal, 1, 0)).astype(np.uint8)
        channel, shift = (0, 2 * k) if k < 4 else (1, 2 * (k - 4))
        packed[..., channel] |= code << shift
        packed[..., 2] += (code > 0).astype(np.uint8)
    Image.fromarray(packed, "RGB").save(os.path.join(out, "fish.png"))

    area = np.cos((0.5 - (np.arange(h) + 0.5) / h) * np.pi)[:, None] * np.ones((1, w))
    return {
        "climate_year": years,
        "species": [dict(id=sp["id"], name=sp["name"], water=sp["water"], temp=sp["temp"]) for sp in fr.SPECIES],
        "surface_mean_c": round(float((area * t_mean).sum() / area.sum()), 2),
        "water_mean_c": round(float((area * t_mean)[water].sum() / (area * water).sum()), 2),
        "frozen_all_year_share_of_water": round(float((area * frozen).sum() / (area * water).sum()), 4),
    }


ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def game_ramps():
    """The overlay ramps as the game's legend and shader draw them, read from
    `pbd_app::overlay::RAMPS` itself, and the fade the shader gives cloud and
    rain (`OVERLAY_FADE_FULL` in water.wgsl): read, never restated, as the
    game's own test reads the shader's copy of the table."""
    with open(os.path.join(ROOT, "crates/pbd-app/src/overlay.rs")) as f:
        rust = f.read()
    table = rust[rust.index("pub const RAMPS"):]
    table = table[: table.index("];\n")]
    numbers = [float(n) for n in re.findall(r"-?\d+\.\d+", table.split("=", 1)[1])]
    stops = np.array(numbers).reshape(-1, 5, 3)
    with open(os.path.join(ROOT, "assets/shaders/water.wgsl")) as f:
        fade = float(re.search(r"const OVERLAY_FADE_FULL: f32 = ([\d.]+);", f.read()).group(1))
    return stops, fade


def ramp_rgb(stops, t):
    """A ramp's colour at t in 0..1, between its stops as the shader does."""
    x = np.clip(t, 0.0, 1.0) * (len(stops) - 1)
    i = np.minimum(np.floor(x).astype(int), len(stops) - 2)
    f = (x - i)[..., None]
    return stops[i] + (stops[i + 1] - stops[i]) * f


def cloud_frame(cover, rain_kg, out, k):
    """The live cloud layer: white cloud with its cover as alpha, tinted blue
    by how hard it rains: nothing below the atmosphere's raining rate
    (2e-4 kg/m^2/s), full at four times it."""
    height, width = cover.shape
    white, rainy = np.array([250.0, 250.0, 250.0]), np.array([105.0, 145.0, 212.0])
    wet = np.clip((rain_kg - 2.0e-4) / 6.0e-4, 0.0, 1.0)[..., None]
    rgba = np.zeros((height, width, 4), dtype=np.uint8)
    rgba[..., :3] = (white * (1 - wet) + rainy * wet).astype(np.uint8)
    rgba[..., 3] = (np.clip(cover, 0, 1) * 215).astype(np.uint8)
    Image.fromarray(rgba, "RGBA").save(os.path.join(out, f"clouds-{k:02}.png"))


def weather_frames(path, out, altitude=None):
    with open(path, "rb") as f:
        data = f.read()
    if data[:8] == b"PBDWTHR1":
        width, height, frames = struct.unpack_from("<III", data, 8)
        body = np.frombuffer(data, dtype="<f4", offset=20).reshape(frames, 3, height, width)
        for k in range(frames):
            cloud_frame(body[k, 0], body[k, 1], out, k)
        return {"weather_frames": int(frames), "weather_frame_hours": 24.0 / frames}
    assert data[:8] == b"PBDWTHR2", "not a map_weather raster"
    width, height, frames, count = struct.unpack_from("<IIII", data, 8)
    at = 24
    overlays = []
    for _ in range(count):
        name = data[at:at + 16].rstrip(b"\0").decode()
        unit = data[at + 16:at + 24].rstrip(b"\0").decode()
        lo, hi = struct.unpack_from("<ff", data, at + 24)
        ramp_row, flags = struct.unpack_from("<II", data, at + 32)
        overlays.append({"name": name, "unit": unit, "lo": lo, "hi": hi, "ramp": ramp_row,
                         "flows": bool(flags & 1), "fades": bool(flags & 2)})
        at += 40
    stops, fade = game_ramps()
    # The current is the sea's and reads zero on land (`Sample::current`), so
    # land is left clear under it rather than drawn as still water: palette
    # entry 255 is "no data", and every value takes the other 255.
    land = None
    if altitude is not None:
        rows = ((np.arange(height) + 0.5) / height * altitude.shape[0]).astype(int)
        cols = ((np.arange(width) + 0.5) / width * altitude.shape[1]).astype(int)
        land = altitude[rows][:, cols] >= 0.0
    planes = np.frombuffer(data, dtype="<f4", offset=at)
    per_frame = sum(3 if o["flows"] else 1 for o in overlays)
    planes = planes.reshape(frames, per_frame, height, width)
    weather = os.path.join(out, "weather")
    os.makedirs(weather, exist_ok=True)
    for o in overlays:
        # One palette per overlay: 256 steps of its ramp, and for cloud and
        # rain an alpha that fades toward nothing as the globe's does.
        t = np.minimum(np.arange(256) / 254.0, 1.0)
        rgb = np.clip(ramp_rgb(stops[o["ramp"]], t) * 255, 0, 255).astype(np.uint8)
        value = o["lo"] + t * (o["hi"] - o["lo"])
        alpha = np.full(256, 255, dtype=np.uint8)
        if o["fades"]:
            alpha = (np.clip(np.abs(value) / (o["hi"] * fade), 0, 1) * 255).astype(np.uint8)
        alpha[255] = 0
        o["palette"] = rgb.reshape(-1).tolist()
        o["alpha"] = alpha.tolist()
        o["stops"] = np.round(stops[o["ramp"]] * 255).astype(int).tolist()
    for k in range(frames):
        plane = 0
        for o in overlays:
            scalar = planes[k, plane]
            key = o["name"].lower()
            if key == "cloud":
                cover = scalar / 100.0
            if key == "rain":
                rain_kg = np.maximum(scalar, 0.0) / 3600.0
            t = (scalar - o["lo"]) / (o["hi"] - o["lo"])
            index = np.clip(np.round(t * 254), 0, 254).astype(np.uint8)
            if key == "currents" and land is not None:
                index[land] = 255
            image = Image.fromarray(index, "P")
            image.putpalette(o["palette"])
            image.info["transparency"] = bytes(o["alpha"])
            image.save(os.path.join(weather, f"{key}-{k:02}.png"), optimize=True,
                       transparency=bytes(o["alpha"]))
            plane += 1
            if o["flows"]:
                # The flow at half the raster, eastward in red and northward
                # in green, 128 for still and the ends for the range's top.
                east, north = planes[k, plane], planes[k, plane + 1]
                half = lambda a: a.reshape(height // 2, 2, width // 2, 2).mean(axis=(1, 3))
                code = lambda v: np.clip(np.round(128 + v / o["hi"] * 127), 1, 255).astype(np.uint8)
                rgb = np.stack([code(half(east)), code(half(north)),
                                np.full((height // 2, width // 2), 128, np.uint8)], axis=-1)
                Image.fromarray(rgb, "RGB").save(os.path.join(weather, f"{key}-flow-{k:02}.png"),
                                                 optimize=True)
                plane += 2
        cloud_frame(cover, rain_kg, out, k)
    for o in overlays:
        del o["palette"], o["alpha"]
    return {"weather_frames": int(frames), "weather_frame_hours": 24.0 / frames,
            "weather_overlays": overlays}


def tiles(source, out, level, tile=512):
    """The base map's finer levels (`world-map` decision 9): a finer raster
    from `world_map ... <width> base`, coloured as base.png is, cut into
    512-pixel tiles, tiles/<level>/<row>-<col>.png. Every tile takes
    base.png's own palette, so no seam shows between tiles or levels. The
    level's size goes into summary.json for the page."""
    width, height, planes = read(source)
    altitude, top, biome = planes[0], planes[1].astype(int), planes[2].astype(int)
    sea = altitude < 0.0
    colour = ground_colours(altitude, top, biome) * relief(altitude, width)[..., None]
    colour[sea] = sea_colours(-altitude)[sea]
    rgb = Image.fromarray(np.clip(colour, 0, 255).astype(np.uint8), "RGB")
    del colour
    palette = Image.open(os.path.join(out, "base.png"))
    folder = os.path.join(out, "tiles", str(level))
    os.makedirs(folder, exist_ok=True)
    for row in range(height // tile):
        for col in range(width // tile):
            piece = rgb.crop((col * tile, row * tile, (col + 1) * tile, (row + 1) * tile))
            save(piece.quantize(palette=palette, dither=Image.Dither.NONE),
                 os.path.join(folder, f"{row}-{col}.png"))
    path = os.path.join(out, "summary.json")
    with open(path) as f:
        summary = json.load(f)
    levels = [entry for entry in summary.get("tile_levels", []) if entry["level"] != level]
    levels.append({"level": level, "width": width, "height": height, "tile": tile})
    summary["tile_levels"] = sorted(levels, key=lambda entry: entry["level"])
    with open(path, "w") as f:
        json.dump(summary, f, indent=1, sort_keys=True)
        f.write("\n")
    print(f"level {level}: {width} x {height} in {(width // tile) * (height // tile)} tiles")


def main():
    if sys.argv[1] == "tiles":
        tiles(sys.argv[2], sys.argv[3], int(sys.argv[4]))
        return
    source, out = sys.argv[1], sys.argv[2]
    options = dict(zip(sys.argv[3::2], sys.argv[4::2]))
    os.makedirs(out, exist_ok=True)
    width, height, planes = read(source)
    altitude, top, today = planes[0], planes[1].astype(int), planes[2].astype(int)
    scales = {m: planes[3 + i].astype(int) for i, m in enumerate(SCALES)}

    sea = altitude < 0.0
    colour = ground_colours(altitude, top, today) * relief(altitude, width)[..., None]
    colour[sea] = sea_colours(-altitude)[sea]
    base = Image.fromarray(np.clip(colour, 0, 255).astype(np.uint8), "RGB")
    save(base.quantize(colors=128, method=Image.Quantize.MEDIANCUT, dither=Image.Dither.NONE),
         os.path.join(out, "base.png"))

    shares = {}
    lat = (0.5 - (np.arange(height) + 0.5) / height) * np.pi
    area = np.repeat(np.cos(lat)[:, None], width, axis=1)
    for m, biome in [("today", today)] + list(scales.items()):
        rgba = np.zeros((height, width, 4), dtype=np.uint8)
        for b, name in enumerate(BIOMES):
            if name in OVERLAY:
                rgba[biome == b] = OVERLAY[name] + (255,)
        Image.fromarray(rgba, "RGBA").save(os.path.join(out, f"biomes-{m}.png"))
        land = area[biome != 0].sum()
        shares[str(m)] = {name: round(float(area[biome == b].sum() / land), 4)
                          for b, name in enumerate(BIOMES) if b != 0}

    packed = np.clip(np.round(altitude) + 32768, 0, 65535).astype(np.uint32)
    four = scales[750]
    height_png = np.stack([packed >> 8, packed & 255, (four << 4) | today], axis=-1).astype(np.uint8)
    Image.fromarray(height_png, "RGB").save(os.path.join(out, "height.png"))

    summary = {
        "width": width,
        "height": height,
        "land_share": round(float(area[~sea].sum() / area.sum()), 4),
        "altitude_m": [float(altitude.min()), float(altitude.max())],
        "biome_shares_of_land": shares,
    }
    if "--fish" in options:
        summary.update(climate_and_fish(options["--fish"], out))
    if "--weather" in options:
        summary.update(weather_frames(options["--weather"], out, altitude))
    try:
        with open(os.path.join(out, "summary.json")) as f:
            kept = json.load(f).get("tile_levels")
        if kept:
            summary["tile_levels"] = kept
    except (OSError, ValueError):
        pass
    with open(os.path.join(out, "summary.json"), "w") as f:
        json.dump(summary, f, indent=1, sort_keys=True)
        f.write("\n")
    print(json.dumps(summary, indent=1, sort_keys=True))


if __name__ == "__main__":
    main()
