"""The logo's key (owner, 2026-11-16: "can you also explain all the glyphs? get me an image with
text annotate"): the render with every etched scene numbered, and a legend under it that tells the
story they make, from the comet to the starship.

    python3 tools/art/logo/logo_key.py <render.png> <glyphs.json> <out.jpg>
"""
import json
import sys

from PIL import Image, ImageDraw, ImageFont

FONT = "/mnt/skills/examples/canvas-design/canvas-fonts/YoungSerif-Regular.ttf"
BODY = "/mnt/skills/examples/canvas-design/canvas-fonts/IBMPlexSerif-Regular.ttf"

STORY = {
    "comet": ("Comet", "The story begins: a comet breaks apart and rains pieces of alien knowledge over the world."),
    "hunting": ("Hunting", "A hunter draws a bow on a stag: the first tools."),
    "fishing": ("Fishing", "A fisher in a canoe, the line down to a fish."),
    "farming": ("Farming", "Under the sun, a farmer at the hoe beside the wheat."),
    "rafting": ("Rafting", "Poling a log raft down the river."),
    "sailing": ("Sailing", "A ship under sail on the waves, a bird above."),
    "mining": ("Mining", "A miner with a pick at the mine's mouth, ore in the rock."),
    "shipbuilding": ("Ship building", "A hull's ribs on the slipway, a builder with a hammer."),
    "windmill": ("Windmills", "The first power from the wind."),
    "watermill": ("Water mills", "A wheel turned by the river beside its mill."),
    "flying": ("Flying", "A biplane over the clouds."),
    "nuclear": ("Nuclear power", "The atom: a nucleus and its orbits."),
    "rocketry": ("Rocketry", "A rocket rising on its flame toward the moon and stars."),
    "computer": ("Supercomputers", "Racks of a great computer and the one who tends it."),
    "solar": ("Solar arrays", "Rows of panels turned to the sun."),
    "fusion": ("Fusion", "A star held in a ring."),
    "station": ("Space stations", "A ring station: hub, spokes and a panel."),
    "colony": ("Colonies", "Domes on the curve of a new world."),
    "dyson": ("Dyson swarm", "A sun ringed by a swarm of collectors."),
    "antimatter": ("Antimatter", "Matter meets antimatter, + and -, in a burst."),
    "starship": ("Starship", "The story's end: a starship, built from all of it, leaving for the stars."),
    "elevator": ("Space elevator", "A tether from the planet down to the moon, climbers on it: the world reaching out."),
}


def main(render, placed_json, out):
    im = Image.open(render).convert("RGBA")
    W, H = im.size
    placed = json.load(open(placed_json))
    bg = Image.new("RGBA", (W, H), (8, 14, 24, 255))
    bg.alpha_composite(im)
    legend_h = 1150
    page = Image.new("RGB", (W, H + legend_h), (8, 14, 24))
    page.paste(bg.convert("RGB"), (0, 0))
    d = ImageDraw.Draw(page)
    fb = ImageFont.truetype(FONT, 46)
    ft = ImageFont.truetype(FONT, 40)
    fx = ImageFont.truetype(BODY, 30)
    gold = (250, 175, 92)
    # badges, set off the slab along a leader, so no carving is covered
    for i, g in enumerate(placed, 1):
        x, y = g["x"], g["y"]
        if g["name"] == "elevator":
            bx, by = x + 140, y
        elif y < 1000:
            bx, by = x, y - 150
        else:
            bx, by = x, y + 150
        d.line([(x, y + (60 if by > y and g["name"] != "elevator" else -60 if g["name"] != "elevator" else 0)), (bx, by)], fill=gold, width=4)
        d.ellipse([bx - 34, by - 34, bx + 34, by + 34], fill=(20, 30, 44), outline=gold, width=5)
        d.text((bx, by + 2), str(i), font=fb, fill=gold, anchor="mm")
    # the legend: two columns, the story in order
    d.text((W / 2, H + 60), "The story etched in the stone", font=ImageFont.truetype(FONT, 64), fill=gold, anchor="mm")
    col_w = W / 2
    per = (len(placed) + 1) // 2
    for i, g in enumerate(placed, 1):
        title, line = STORY[g["name"]]
        c, r = (i - 1) // per, (i - 1) % per
        x0, y0 = 90 + c * col_w, H + 140 + r * 88
        d.text((x0, y0), f"{i}.", font=ft, fill=gold)
        d.text((x0 + 70, y0), title, font=ft, fill=(234, 220, 200))
        d.text((x0 + 70, y0 + 44), line, font=fx, fill=(170, 182, 196))
    page.save(out, quality=90)
    print("key", page.size)


if __name__ == "__main__":
    main(*sys.argv[1:4])
