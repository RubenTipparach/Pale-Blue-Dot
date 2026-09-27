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

    python3 tools/world_map.py WORLD_MAP.bin OUT_DIR

It writes the same bytes from the same input, so a regenerated map diffs to
nothing.
"""

import json
import os
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


def main():
    source, out = sys.argv[1], sys.argv[2]
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
    with open(os.path.join(out, "summary.json"), "w") as f:
        json.dump(summary, f, indent=1, sort_keys=True)
        f.write("\n")
    print(json.dumps(summary, indent=1, sort_keys=True))


if __name__ == "__main__":
    main()
