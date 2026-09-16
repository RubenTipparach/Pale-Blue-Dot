"""Validate generated PNGs and build an offline, source-atlas review gallery.

This tool reads images unchanged. It does not resample, repaint, or certify
seamlessness. No external Python dependencies are required.
"""

import hashlib
import html
import json
from pathlib import Path
import struct
import zlib

ROOT = Path(__file__).resolve().parents[1]
ASSETS = ROOT / "assets" / "tilesets"


def png_info(path):
    data = path.read_bytes()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError(f"Not a PNG: {path}")
    offset, dimensions, complete = 8, None, False
    while offset < len(data):
        size = struct.unpack_from(">I", data, offset)[0]
        kind = data[offset + 4:offset + 8]
        payload = data[offset + 8:offset + 8 + size]
        crc = struct.unpack_from(">I", data, offset + 8 + size)[0]
        if zlib.crc32(kind + payload) != crc:
            raise ValueError(f"Corrupt PNG chunk {kind!r}: {path}")
        if kind == b"IHDR":
            dimensions = struct.unpack_from(">II", payload)
        if kind == b"IEND":
            complete = True
            break
        offset += size + 12
    if not dimensions or not complete:
        raise ValueError(f"Incomplete PNG: {path}")
    return {
        "width": dimensions[0], "height": dimensions[1],
        "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest(),
        "exact_four_by_four_pixel_grid": all(n % 4 == 0 for n in dimensions),
        "production_approved": False,
    }


def main():
    catalog = json.loads((ASSETS / "biomes.json").read_text(encoding="utf-8"))
    biomes = catalog["biomes"]
    if len({b["id"] for b in biomes}) != len(biomes):
        raise ValueError("Duplicate biome ID")
    manifest = {}
    for biome in biomes:
        if len(biome["tiles"]) != 16:
            raise ValueError(f"Expected 16 material slots: {biome['id']}")
        manifest[biome["id"]] = png_info(ASSETS / biome["image"])
    (ASSETS / "generated-manifest.json").write_text(
        json.dumps({"tool": "built-in image_gen", "images": manifest}, indent=2) + "\n",
        encoding="utf-8",
    )
    template = (ROOT / "tools" / "biome-gallery.template.html").read_text(encoding="utf-8")
    data = json.dumps(biomes, ensure_ascii=True).replace("</", "<\\/")
    options = "".join(
        f'<option value="{i}">{html.escape(b["planet"])} / {html.escape(b["name"])}</option>'
        for i, b in enumerate(biomes)
    )
    result = template.replace("__BIOME_DATA__", data).replace("__OPTIONS__", options)
    (ROOT / "docs" / "biome-atlas.html").write_text(result, encoding="utf-8")
    irregular = sum(not m["exact_four_by_four_pixel_grid"] for m in manifest.values())
    print(f"Validated {len(biomes)} PNGs and {len(biomes) * 16} material slots; gallery built.")
    print(f"{irregular} sheets need pixel-grid normalization at production import; originals preserved.")


if __name__ == "__main__":
    main()
