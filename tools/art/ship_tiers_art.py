"""Part icons and part textures for the ship-tiers mockup (docs/mockups/ship-tiers.html).

Pixel art drawn in code with the balatro-pixel-art kit (`.claude/skills/balatro-pixel-art`), the
solitaire collection's style, every colour from Sweaft-64 (the kit's save() refuses anything else).
It reuses the ship-crafting mockup's ramps and plate helper, so a hex-stack part and a grid-ship
module wear the same hull.

- part icons: 32 x 32, transparent, 1 px outline, light from the top left, shown only at whole
  multiples of 32 px with `image-rendering: pixelated`;
- part textures: 32 x 32 tiles that repeat seamlessly, opaque, sampled NearestFilter with no
  mipmaps by the page's three.js parts.

It also copies the game's own files the page uses, byte for byte, into
docs/mockups/ship-tiers/game/, and repackages each vehicle .glb as .gltf.json (the same model with
its binary buffer as base64) because the artifact host does not serve .glb.

    python tools/art/ship_tiers_art.py
    python tools/art/ship_tiers_art.py --sheet OUT.png
"""

import base64
import glob
import importlib.util
import json
import os
import shutil
import struct
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
SKILL = os.path.join(ROOT, ".claude", "skills", "balatro-pixel-art")
sys.path.insert(0, os.path.join(SKILL, "scripts"))
from pixelkit import Sprite, save, sheet  # noqa: E402

_spec = importlib.util.spec_from_file_location("ship_crafting_art", os.path.join(HERE, "ship-crafting_art.py"))
SC = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(SC)
R, plates, glint_star, XS, YS, SPECK = SC.R, SC.plates, SC.glint_star, SC.XS, SC.YS, SC.SPECK
R = dict(R)
R.update({
    "magenta": ["#2f1525", "#9e3ba2", "#c34988", "#dd576d", "#fffefe"],
    "warp": ["#230c44", "#4f1d83", "#7d2dbd", "#9e3ba2", "#d2f8ff"],
    "fusion": ["#15578a", "#1096c3", "#74effe", "#d2f8ff", "#fffefe"],
    "yellow": ["#8d4526", "#c56d25", "#fb902f", "#faaf5c"],
    "green": ["#003227", "#297537", "#559642", "#9bb675"],
})
GLINT = "#fffefe"
OUT = os.path.join(ROOT, "docs", "mockups", "ship-tiers")
GAME = os.path.join(OUT, "game")
N = 32

# ---------------------------------------------------------------- part icons


def pod(sp):
    """The command pod: a blunt cone, a window, a heat shield under it."""
    body = sp.poly([(11, 4), (20, 4), (28, 24), (3, 24)])
    sp.part(body, R["white"], "cyl_v", base=3)
    sp.part(sp.rect(13, 1, 18, 4), R["dark"], "rim", base=2)
    sp.part(sp.poly([(13, 10), (18, 10), (20, 15), (11, 15)]), R["cyan"], "rim", base=2)
    sp.part(sp.rect(3, 25, 28, 28), R["rust"], "cyl_v", base=1)
    sp.plots([(14, 11), (13, 12)], GLINT)
    sp.outline()


def tank(sp):
    """A hex-cell propellant tank: orange foam, a white band."""
    body = sp.poly([(8, 3), (23, 3), (28, 8), (28, 25), (23, 29), (8, 29), (3, 25), (3, 8)])
    sp.part(body, R["orange"], "cyl_v", base=2)
    sp.part(sp.rect(3, 13, 28, 18), R["white"], "cyl_v", base=3)
    for x in (10, 21):
        sp.fill(sp.rect(x, 4, x, 12) | sp.rect(x, 19, x, 28), R["orange"][1])
    sp.plots([(7, 6), (6, 7)], GLINT)
    sp.outline()


def engine_chem(sp):
    """The chemical bell: a white mount, a dark bell, an orange flame."""
    sp.part(sp.rect(9, 2, 22, 7), R["white"], "rim", base=3)
    sp.part(sp.poly([(11, 8), (20, 8), (26, 21), (5, 21)]), R["dark"], "cyl_v", base=3)
    sp.part(sp.rect(5, 19, 26, 21), R["steel"], "rim", base=2)
    sp.plots([(10, 3), (11, 3)], GLINT)
    sp.outline()
    sp.part(sp.poly([(7, 22), (24, 22), (19, 30), (15, 31), (12, 30)]), R["flame"], "inset", glow=True)


def engine_ntr(sp):
    """Nuclear thermal: a long reactor drum with a yellow hazard band, a pale blue plume."""
    sp.part(sp.rect(9, 1, 22, 13), R["dark"], "cyl_v", base=2)
    band = sp.rect(9, 5, 22, 9)
    sp.part(band, R["yellow"], "cyl_v", base=2)
    sp.fill(band & (((XS + YS) // 2) % 2 == 0), R["ink"][1])
    sp.part(sp.poly([(12, 14), (19, 14), (24, 23), (7, 23)]), R["metal"], "cyl_v", base=2)
    sp.plots([(10, 2), (11, 2)], GLINT)
    sp.outline()
    sp.part(sp.poly([(9, 24), (22, 24), (18, 31), (13, 31)]), R["fusion"], "inset", glow=True)


def decoupler(sp):
    """A thin separation ring in hazard stripes."""
    ring = sp.rect(2, 10, 29, 21)
    sp.part(ring, R["hazard"], "cyl_v", base=2)
    sp.fill(ring & (((XS - YS) // 3) % 2 == 0), R["ink"][1])
    sp.part(sp.rect(6, 7, 25, 9), R["steel"], "rim", base=2)
    sp.part(sp.rect(6, 22, 25, 24), R["steel"], "rim", base=2)
    sp.plots([(4, 13), (5, 13)], GLINT)
    sp.outline()


def booster(sp):
    """A radial solid booster: tall, white with black roll marks, a nose cone."""
    body = sp.rect(11, 7, 20, 25)
    sp.part(sp.poly([(15, 1), (16, 1), (20, 7), (11, 7)]), R["red"], "cyl_v", base=2)
    sp.part(body, R["white"], "cyl_v", base=3)
    sp.fill(body & (((YS // 6) + (XS >= 16)) % 2 == 0), R["dark"][1])
    sp.part(sp.poly([(11, 26), (20, 26), (22, 29), (9, 29)]), R["dark"], "rim", base=2)
    sp.plots([(12, 8), (12, 9)], GLINT)
    sp.outline()
    sp.part(sp.poly([(11, 30), (20, 30), (16, 31)]), R["flame"], "inset", glow=True)


def fin(sp):
    """A swept fin, red, on a white root."""
    sp.part(sp.rect(4, 4, 9, 28), R["white"], "rim", base=3)
    sp.part(sp.poly([(10, 6), (27, 22), (27, 28), (10, 28)]), R["red"], "rim", base=2)
    sp.plots([(5, 5), (5, 6)], GLINT)
    sp.outline()


def legs(sp):
    """A lander leg: strut, piston, a wide foot."""
    sp.part(sp.rect(6, 3, 12, 9), R["white"], "rim", base=3)
    sp.part(sp.line([(10, 8), (22, 24)], 3), R["metal"], "rim", base=2)
    sp.part(sp.line([(9, 9), (14, 24)], 2), R["steel"], "rim", base=2)
    sp.part(sp.rect(16, 25, 29, 28), R["dark"], "rim", base=3)
    sp.plots([(7, 4), (8, 4)], GLINT)
    sp.outline()


def fusion(sp):
    """A fusion drive: a torus of magnets round a cyan plasma, a magnetic nozzle."""
    sp.part(sp.rect(6, 3, 25, 18), R["white"], "cyl_v", base=3)
    for y in (6, 13):
        sp.part(sp.rect(4, y, 27, y + 2), R["dark"], "cyl_v", base=2)
    sp.part(sp.poly([(10, 19), (21, 19), (27, 26), (4, 26)]), R["metal"], "cyl_v", base=2)
    sp.plots([(8, 4), (9, 4)], GLINT)
    sp.outline()
    sp.part(sp.rect(12, 9, 19, 11), R["fusion"], "inset", glow=True)
    sp.part(sp.taper(16, 27, 16, 31, 7, 2), R["fusion"], "inset", glow=True)


def antimatter(sp):
    """An antimatter core: a magnetic bottle with a magenta spark, gold coils."""
    sp.part(sp.circle(16, 15, 12), R["dark"], "sphere", base=2)
    for y in (7, 22):
        sp.part(sp.rect(5, y, 26, y + 2), R["gold"], "cyl_v", base=2)
    sp.part(sp.rect(10, 27, 21, 30), R["metal"], "rim", base=2)
    sp.outline()
    sp.part(sp.circle(16, 15, 5), R["magenta"], "inset", glow=True)
    glint_star(sp, 16, 15, R["magenta"])


def warp(sp):
    """A warp ring: a violet field in a ring of plates."""
    ring = sp.circle(16, 16, 14) & ~sp.circle(16, 16, 9)
    sp.part(ring, R["metal"], "sphere", base=3)
    for k in range(6):
        import math
        a = k * math.pi / 3
        sp.fill(sp.circle(round(16 + 11.5 * math.cos(a)), round(16 + 11.5 * math.sin(a)), 1) & ring, R["warp"][1])
    sp.outline()
    sp.part(sp.circle(16, 16, 8), R["warp"], "inset", glow=True)
    sp.plots([(16, 16), (15, 16), (17, 16)], GLINT, glow=True)


def navcomp(sp):
    """The nav computer: a screen with a plotted course."""
    sp.part(sp.rect(2, 4, 29, 24), R["white"], "rim", base=3)
    screen = sp.rect(5, 7, 26, 21)
    sp.part(screen, R["night"], "flat", base=1)
    sp.part(sp.rect(10, 25, 21, 28), R["dark"], "rim", base=2)
    sp.outline()
    sp.fill(sp.circle(9, 17, 2), R["cyan"][3], glow=True)
    sp.fill(sp.circle(22, 10, 2), R["green"][3], glow=True)
    sp.fill(sp.line([(10, 16), (21, 11)], 1), R["gold"][3], glow=True)
    sp.plot(15, 14, GLINT, glow=True)


def freighter(sp):
    """An automated freighter: a spine of orange cargo pods, a drive, a lamp."""
    sp.part(sp.rect(2, 14, 29, 17), R["dark"], "cyl_h", base=2)
    for x in (5, 13, 21):
        c = sp.rect(x, 7, x + 6, 24)
        sp.part(c, R["orange"], "rim", base=2)
    sp.part(sp.poly([(27, 9), (30, 12), (30, 19), (27, 22)]), R["white"], "rim", base=3)
    sp.plots([(6, 8), (7, 8)], GLINT)
    sp.outline()
    sp.part(sp.taper(1, 15, 0, 15, 2, 1), R["fusion"], "inset", glow=True)
    sp.plot(29, 15, "#faaf5c", glow=True)


def lander(sp):
    """A lander: a cabin on four splayed legs."""
    sp.part(sp.poly([(10, 4), (21, 4), (25, 17), (6, 17)]), R["white"], "cyl_v", base=3)
    sp.part(sp.rect(12, 8, 19, 11), R["cyan"], "rim", base=2)
    sp.part(sp.line([(8, 16), (3, 27)], 2) | sp.line([(23, 16), (28, 27)], 2), R["metal"], "rim", base=2)
    sp.part(sp.rect(1, 27, 6, 29) | sp.rect(25, 27, 30, 29), R["dark"], "rim", base=2)
    sp.part(sp.rect(13, 18, 18, 21), R["dark"], "rim", base=2)
    sp.plots([(13, 9), (12, 5)], GLINT)
    sp.outline()
    sp.part(sp.poly([(13, 22), (18, 22), (16, 27), (15, 27)]), R["fusion"], "inset", glow=True)


def starship(sp):
    """A starship: a long hull, a warp ring amidships, an antimatter glow aft."""
    sp.part(sp.poly([(2, 11), (21, 10), (30, 14), (30, 17), (21, 21), (2, 20)]), R["white"], "cyl_h", base=3)
    ring = sp.rect(11, 4, 15, 27)
    sp.part(ring, R["metal"], "rim", base=3)
    sp.part(sp.rect(24, 14, 27, 16), R["cyan"], "rim", base=2)
    sp.plots([(4, 13), (5, 13)], GLINT)
    sp.outline()
    sp.fill(sp.rect(12, 6, 14, 9) | sp.rect(12, 22, 14, 25), R["warp"][2], glow=True)
    sp.part(sp.taper(1, 15, 0, 15, 5, 3), R["magenta"], "inset", glow=True)


ICONS = {
    "pod": pod, "tank": tank, "engine-chem": engine_chem, "engine-ntr": engine_ntr, "decoupler": decoupler,
    "booster": booster, "fin": fin, "legs": legs, "fusion": fusion, "antimatter": antimatter, "warp": warp,
    "navcomp": navcomp, "freighter": freighter, "lander": lander, "starship": starship,
}

# ---------------------------------------------------------------- part textures (32 x 32, seamless)


def tex_pod(sp):
    plates(sp, R["white"])


def tex_tank(sp):
    o = R["orange"]
    sp.fill(sp.rect(0, 0, 31, 31), o[2])
    sp.fill(SPECK, o[1])
    sp.fill((YS % 8) == 0, o[1])
    sp.fill((XS % 16) == 0, o[0])
    sp.fill((XS % 16) == 1, o[3])
    band = (YS >= 12) & (YS <= 19)
    sp.fill(band, R["white"][3])
    sp.fill(YS == 12, R["white"][4])
    sp.fill((YS == 11) | (YS == 20), o[0])


def tex_engine(sp):
    d = R["dark"]
    v = YS % 4
    sp.fill(v == 0, d[0])
    sp.fill(v == 1, d[4])
    sp.fill(v == 2, d[3])
    sp.fill(v == 3, d[2])
    sp.fill((XS % 16 == 0) & (v != 0), d[1])


def tex_hazard(sp):
    sp.fill(sp.rect(0, 0, 31, 31), R["hazard"][2])
    sp.fill(((XS + YS) // 4) % 2 == 0, R["ink"][1])
    sp.fill((YS % 16) == 0, R["steel"][2])


def tex_fin(sp):
    plates(sp, R["red"], rivets=False)


def tex_booster(sp):
    w = R["white"]
    plates(sp, w)
    sp.fill(((YS // 16) + (XS // 16)) % 2 == 0, R["dark"][1])
    sp.fill((((YS // 16) + (XS // 16)) % 2 == 0) & SPECK, R["dark"][2])


def tex_ntr(sp):
    d = R["dark"]
    sp.fill(sp.rect(0, 0, 31, 31), d[2])
    sp.fill(SPECK, d[1])
    sp.fill((XS % 8) == 0, d[1])
    band = (YS >= 10) & (YS <= 21)
    sp.fill(band, R["yellow"][3])
    sp.fill(band & (((XS + YS) // 4) % 2 == 0), R["ink"][1])
    sp.fill((YS == 9) | (YS == 22), d[0])


def tex_fusion(sp):
    plates(sp, R["white"], cell=16)
    rib = ((YS % 16) >= 6) & ((YS % 16) <= 9)
    sp.fill(rib, R["dark"][2])
    sp.fill(rib & ((XS % 8) >= 2) & ((XS % 8) <= 5), R["fusion"][2])
    sp.fill(rib & ((XS % 8) >= 3) & ((XS % 8) <= 4) & ((YS % 16) == 7), R["fusion"][3])


def tex_am(sp):
    plates(sp, R["dark"], cell=16)
    core = ((XS % 16) >= 5) & ((XS % 16) <= 10) & ((YS % 16) >= 5) & ((YS % 16) <= 10)
    sp.fill(core, R["magenta"][1])
    sp.fill(core & ((XS % 16) >= 6) & ((XS % 16) <= 9) & ((YS % 16) >= 6) & ((YS % 16) <= 9), R["magenta"][2])
    sp.fill(((XS % 16) == 7) & ((YS % 16) == 7), R["magenta"][3])


def tex_warp(sp):
    edges = SC.hex_edges()
    sp.fill(sp.rect(0, 0, 31, 31), R["warp"][1])
    sp.fill(SPECK, R["warp"][2])
    sp.fill(edges, R["warp"][3])


TEXTURES = {
    "tex-pod": tex_pod, "tex-tank": tex_tank, "tex-engine": tex_engine, "tex-hazard": tex_hazard,
    "tex-fin": tex_fin, "tex-booster": tex_booster, "tex-ntr": tex_ntr, "tex-fusion": tex_fusion,
    "tex-am": tex_am, "tex-warp": tex_warp,
}

# ---------------------------------------------------------------- the game's own files

GAME_FILES = {
    "atlas.png": "assets/tilesets/atlas.png",
    "iron.png": "assets/textures/settlement/iron.png",
    "granite.png": "assets/textures/settlement/granite.png",
    "stone.png": "assets/textures/settlement/stone.png",
    "slate.png": "assets/textures/settlement/slate.png",
    "plank.png": "assets/textures/settlement/plank.png",
    "hand.png": "assets/items/tools/hand.png",
    "pickaxe.png": "assets/items/tools/pickaxe.png",
    "lantern_post.png": "assets/items/lights/lantern_post.png",
    "torch.png": "assets/items/lights/torch.png",
    "kestrel.glb": "assets/models/vehicles/kestrel/kestrel.glb",
    "tern.glb": "assets/models/vehicles/tern/tern.glb",
    "loon.glb": "assets/models/vehicles/loon/loon.glb",
}


def glb_to_gltf_json(src, dst):
    """The same model as glTF JSON, its one binary buffer embedded as a base64 data URI."""
    data = open(src, "rb").read()
    magic, _version, _length = struct.unpack_from("<III", data, 0)
    assert magic == 0x46546C67, src + " is not a GLB"
    off, js, binary = 12, None, None
    while off < len(data):
        clen, ctype = struct.unpack_from("<II", data, off)
        chunk = data[off + 8: off + 8 + clen]
        if ctype == 0x4E4F534A:
            js = json.loads(chunk.decode("utf-8"))
        elif ctype == 0x004E4942:
            binary = chunk
        off += 8 + clen
    js["buffers"][0]["uri"] = "data:application/octet-stream;base64," + base64.b64encode(binary).decode("ascii")
    with open(dst, "w", encoding="utf-8") as f:
        json.dump(js, f, separators=(",", ":"))


def copy_game():
    os.makedirs(GAME, exist_ok=True)
    for name, src in GAME_FILES.items():
        shutil.copyfile(os.path.join(ROOT, src), os.path.join(GAME, name))
        if name.endswith(".glb"):
            glb_to_gltf_json(os.path.join(GAME, name), os.path.join(GAME, name[:-4] + ".gltf.json"))
    print("copied", len(GAME_FILES), "game files to", GAME)


def main():
    os.makedirs(OUT, exist_ok=True)
    copy_game()
    icons, texes = [], []
    for name, draw in ICONS.items():
        sp = Sprite(N, N)
        draw(sp)
        img = sp.image()
        save(img, os.path.join(OUT, name + ".png"), source="balatro-pixel-art pixelkit")
        icons.append((name, img))
    for name, draw in TEXTURES.items():
        sp = Sprite(N, N)
        draw(sp)
        img = sp.image()
        assert img.getchannel("A").getextrema() == (255, 255), name + " has holes"
        save(img.convert("RGB"), os.path.join(OUT, name + ".png"), source="balatro-pixel-art pixelkit")
        texes.append((name, img))
    print("wrote", len(icons), "icons and", len(texes), "textures to", OUT)
    if "--sheet" in sys.argv:
        from PIL import Image
        path = sys.argv[sys.argv.index("--sheet") + 1]
        refs = sorted(p for p in glob.glob(os.path.join(SKILL, "references", "*.png")) if not p.endswith("_contact.png"))
        refs = [(os.path.basename(p)[:-4], Image.open(p).convert("RGBA")) for p in refs]
        refs = [r for r in refs if r[1].size == (56, 56)][-6:]
        tiled = []
        for name, img in texes:
            t = Image.new("RGBA", (64, 64))
            for ox in (0, 32):
                for oy in (0, 32):
                    t.paste(img, (ox, oy))
            tiled.append((name, t))
        sheet(refs + icons + tiled, scale=3, cols=8).save(path)
        print("sheet", path)


if __name__ == "__main__":
    main()
