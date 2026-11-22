#!/usr/bin/env python3
"""Write docs/mockups/npcs/talk-assets.js for M5's Talk tab (version 13).

The tab draws the game's own talk, trade and job board panels, so it takes
the game's own art and words, never redrawn or retyped:
- the stone kit's pieces (assets/ui/stone/*.png, rendered at 8 px a native
  pixel, `slices.json`), reduced to native size by taking every 8th pixel;
- the kit font (assets/fonts/kit-font.png and its glyph table);
- every item icon a row shows (assets/items/**.png, 16 px) and the coin;
- the words, by key, from assets/locale/locale_en.yaml;
- the prices, worked out by the rules of pbd_core::economy::trade and
  ::jobs from assets/config/values.ron, trade.ron and jobs.ron;
- design 37's derived values (a basket deal), from recipes.ron and
  cooking.ron.

All embedded as data URIs. Run from the repository root:
    python3 tools/mockup_talk_assets.py
"""
import base64
import io
import json
import math
import re
from pathlib import Path

from PIL import Image
import yaml

ROOT = Path(__file__).resolve().parent.parent
A = ROOT / "assets"
OUT = ROOT / "docs/mockups/npcs/talk-assets.js"


def uri(img: Image.Image) -> str:
    buf = io.BytesIO()
    img.save(buf, "PNG", optimize=True)
    return "data:image/png;base64," + base64.b64encode(buf.getvalue()).decode()


def native(name: str) -> Image.Image:
    slices = json.loads((A / "ui/slices.json").read_text())
    k = slices["render_px_per_native_px"]
    img = Image.open(A / f"ui/stone/{name}.png").convert("RGBA")
    p = slices["pieces"][name]
    small = img.resize((p["w"], p["h"]), Image.NEAREST)
    # NEAREST at an exact 1/k takes one pixel of each k x k block.
    assert img.size == (p["w"] * k, p["h"] * k), name
    return small


def pieces():
    slices = json.loads((A / "ui/slices.json").read_text())["pieces"]
    out = {}
    for n in ["frame", "lintel", "slot-idle", "slot-hover", "slot-selected", "slider-track",
              "button-idle", "button-hover", "button-pressed"]:
        s = slices[n].get("slice", 0)
        if isinstance(s, int):
            s = [s, s, s, s]
        out[n] = {"uri": uri(native(n)), "w": slices[n]["w"], "h": slices[n]["h"], "slice": s}
    return out


def font():
    table = json.loads((A / "fonts/kit-font.json").read_text())
    glyphs = {c: [g["x"], g["w"]] for c, g in table["glyphs"].items()}
    return {
        "uri": uri(Image.open(A / "fonts/kit-font.png").convert("RGBA")),
        "height": table["height"],
        "spacing": table["spacing"],
        "space": table["space"],
        "glyphs": glyphs,
    }


def ron_values():
    rows = re.findall(r'\("([^"]+)",\s*(\d+),\s*(\w+)\)', (A / "config/values.ron").read_text())
    return [(c, int(v), g) for c, v, g in rows]


def ron_trade():
    t = (A / "config/trade.ron").read_text()
    t = re.sub(r"//.*", "", t)
    num = lambda k: float(re.search(rf"\b{k}:\s*([\d.]+)", t).group(1))
    lst = lambda k: re.findall(r'"([^"]+)"', re.search(rf"\b{k}:\s*\[([^\]]*)\]", t).group(1))
    needs = {}
    for town, body in re.findall(r"(\w+): \{([^}]*)\}", re.search(r"needs: \{(.*?)\n    \}", t, re.S).group(1)):
        needs[town] = {g: float(v) for g, v in re.findall(r"(\w+): ([\d.]+)", body)}
    size = {k: float(v) for k, v in re.findall(r"(\w+): ([\d.]+)", re.search(r"size: \{(.*?)\}", t, re.S).group(1))}
    return {
        "buy_share": num("buy_share"),
        "makes_below": num("makes_below"),
        "stock": num("stock"),
        "want": num("want"),
        "staples": lst("staples"),
        "unsold": lst("unsold"),
        "unbought": lst("unbought"),
        "untraded": re.findall(r"(\w+)", re.search(r"untraded: \[([^\]]*)\]", t).group(1)),
        "needs": needs,
        "size": size,
    }


def locale():
    return yaml.safe_load((A / "locale/locale_en.yaml").read_text())


def key(L, k):
    v = L
    for part in k.split("."):
        v = v[part]
    return v


def icon_path(code):
    if code.startswith("fish:"):
        return A / f"items/fish/{code[5:]}.png"
    if code.startswith("tool:"):
        return A / f"items/tools/{code[5:]}.png"
    return A / f"items/goods/{code}.png"


def name_of(L, code):
    if code.startswith("fish:"):
        return key(L, f"species.{code[5:]}.name") if "species" in L else code
    if code.startswith("tool:"):
        return key(L, f"tool.{code[5:]}.name")
    return key(L, f"item.{code}.name")


# The trade filters' shelves (design 38c), as pbd_core::economy::values::
# Shelf::of maps the groups, in tab order, with the good each tab shows.
SHELF_OF = {"crop": "food", "fruit": "food", "fish": "food", "dish": "food",
            "seed": "organics", "herd": "organics", "wood": "organics",
            "ore": "raw", "stone": "raw", "tool": "tools", "goods": "goods", "light": "goods"}
HERD_FOOD = {"egg", "milk"}  # values::HERD_FOOD: eaten, so Food
SHELVES = ["food", "organics", "raw", "tools", "goods"]
SHELF_PICTURE = {"food": "grilled-fish", "organics": "log", "raw": "iron-ingot",
                 "tools": "tool:pickaxe", "goods": "rope"}


def shelf_of(code):
    return "food" if code in HERD_FOOD else SHELF_OF[ron_values_by_code()[code]]


def ron_values_by_code():
    return {c: g for c, _, g in ron_values()}


def main():
    L = locale()
    values = ron_values()
    V = {c: (v, g) for c, v, g in values}
    T = ron_trade()
    kind = "Desert"  # a caravan master keeps the trade and gives the jobs
    need = lambda g: T["needs"].get(kind, {}).get(g, 1.0)
    scaled = lambda n: max(1, round(n * T["size"][kind]))

    def sells(c):
        v, g = V[c]
        if g in T["untraded"] or c in T["unsold"]:
            return False
        return c in T["staples"] or need(g) <= T["makes_below"]

    def buys(c):
        return c not in T["unbought"] and V[c][1] not in T["untraded"]

    def quote(c):
        v, g = V[c]
        raw = v * need(g)
        sell = max(1, math.ceil(raw - 1e-6))
        pays = min(math.floor(raw * T["buy_share"] + 1e-6), sell - 1)
        return sell, pays

    # The trade panel's lists, as paint_trade makes them: everything it
    # sells in values order (since version 13 no longer cut at nine: the
    # column scrolls, design 38b; block codes have no pack icon file and are
    # drawn from the terrain atlas in the game, so the list is goods), and
    # what the pack holds that it buys: a full pack, so both columns
    # overflow their seven rows and show the scroll bar.
    buy_rows = []
    for c, _, _ in values:
        if sells(c) and not c.startswith("block:"):
            s, _ = quote(c)
            buy_rows.append({"code": c, "price": s, "count": scaled(T["stock"]), "shelf": shelf_of(c)})
    pack = [("fish:perch", 3), ("fish:silverfin", 2), ("log", 4), ("planks", 8), ("wheat", 6),
            ("wool", 3), ("egg", 5), ("copper-ingot", 2), ("rope", 4), ("fibre", 9)]
    sell_rows = []
    for c, n in pack:
        if buys(c):
            _, p = quote(c)
            if p > 0:
                sell_rows.append({"code": c, "price": p, "count": n, "shelf": shelf_of(c)})

    # The board: posted jobs and one of the player's, rewards by jobs.rs
    # (count x unit value x pay_share, rounded).
    jobs_ron = (A / "config/jobs.ron").read_text()
    pay_share = float(re.search(r"pay_share:\s*([\d.]+)", jobs_ron).group(1))
    unit = {k: int(v) for k, v in re.findall(r"(\w+): (\d+)", re.search(r"unit_value: \{(.*?)\}", jobs_ron, re.S).group(1))}
    rnd = lambda x: int(math.floor(x + 0.5))
    town = "Sandmere"
    job_rows = [
        {"kind": "deliver", "count": 1, "have": 0, "town": "Cobblereach", "icon": "parcel", "reward": rnd(1 * unit["deliver"] * pay_share), "list": "posted"},
        {"kind": "cook", "count": 2, "have": 0, "thing": "grilled-fish", "icon": "grilled-fish", "reward": rnd(2 * V["grilled-fish"][0] * pay_share), "list": "posted"},
        {"kind": "gather", "count": 6, "have": 0, "thing": "log", "icon": "log", "reward": rnd(6 * V["log"][0] * pay_share), "list": "posted"},
        {"kind": "lost", "count": 1, "have": 0, "icon": None, "reward": rnd(1 * unit["lost"] * pay_share), "list": "posted"},
        # Two more, so the board's list overflows its seven rows (design 38b).
        {"kind": "cook", "count": 3, "have": 0, "thing": "flatbread", "icon": "flatbread", "reward": rnd(3 * V["flatbread"][0] * pay_share), "list": "posted"},
        {"kind": "gather", "count": 8, "have": 0, "thing": "wheat", "icon": "wheat", "reward": rnd(8 * V["wheat"][0] * pay_share), "list": "posted"},
        {"kind": "repair", "count": 1, "have": 1, "icon": "planks", "reward": rnd(1 * unit["repair"] * pay_share), "list": "yours"},
    ]

    # Design 37's basket: a made good is worth its inputs plus labour, times
    # the craft margin; a raw material keeps its low base value.
    MARGIN, RATE = 1.25, 0.1  # coins a second of work (mockup's assumption)
    recipes = {}
    for text in [(A / "config/recipes.ron").read_text(), (A / "config/cooking.ron").read_text()]:
        for body in re.findall(r"\(\s*code: \"[^\"]+\",(.*?)outputs: \[([^\]]*)\]", text, re.S):
            secs = re.search(r"secs: ([\d.]+)", body[0])
            mins = None
            ins = re.findall(r'\("([^"]+)", (\d+)\)', re.search(r"inputs: \[([^\]]*)\]", body[0]).group(1)) if "inputs" in body[0] else []
            outs = re.findall(r'\("([^"]+)", (\d+)\)', body[1])
            for o, n in outs:
                recipes.setdefault(o, ([(i, int(k)) for i, k in ins], float(secs.group(1)) if secs else 4.0, int(n)))
    derived = {}

    def value(c, depth=0):
        if c in derived:
            return derived[c]
        if c in recipes and depth < 8:
            ins, secs, n = recipes[c]
            v = (sum(value(i, depth + 1) * k for i, k in ins) + secs * RATE) * MARGIN / n
        elif c == "fish":
            # a recipe's "any fish": the mean of the fish the trade lists
            fish = [v for k, (v, g) in V.items() if k.startswith("fish:") and k not in T["unsold"]]
            v = sum(fish) / len(fish)
        else:
            v = float(V.get(c, (1, ""))[0])
        derived[c] = round(v, 2)
        return derived[c]

    basket_codes = list(dict.fromkeys([r["code"] for r in buy_rows] + [c for c, _ in pack]
                                      + ["wheat", "fibre", "rope", "bucket", "grilled-fish", "bronze-ingot"]))
    basket = {}
    for c in basket_codes:
        v = value(c)
        g = V[c][1]
        basket[c] = {"value": v, "raw": c not in recipes, "group": g, "need": need(g),
                     "inputs": recipes[c][0] if c in recipes else [], "secs": recipes[c][1] if c in recipes else 0,
                     "makes": recipes[c][2] if c in recipes else 0}

    codes = sorted({r["code"] for r in buy_rows + sell_rows} | set(SHELF_PICTURE.values()) | {j["icon"] for j in job_rows if j["icon"]} | set(basket_codes)
                   | {i for c in basket_codes for i, _ in basket[c]["inputs"]})
    items = {}
    for c in codes:
        p = icon_path(c)
        if not p.exists():
            continue
        items[c] = {"uri": uri(Image.open(p).convert("RGBA")), "name": name_of(L, c)}
    words = {k: key(L, k) for k in [
        "talk.trade", "talk.work", "talk.bye", "talk.back", "talk.line.trader", "talk.line.ready", "talk.line.giver",
        "talk.role.caravan_master", "trade.title", "trade.buy", "trade.sell", "trade.buy_one", "trade.sell_one",
        "trade.purse", "trade.nothing_to_sell", "trade.bought", "trade.sold", "trade.short", "trade.sold_out", "trade.coins", "trade.count",
        "trade.shelf.all", "trade.shelf.food", "trade.shelf.organics", "trade.shelf.raw", "trade.shelf.tools",
        "trade.shelf.goods",
        "jobs.title", "jobs.posted", "jobs.yours", "jobs.take", "jobs.hand_in", "jobs.taken_line", "jobs.paid",
        "jobs.kind.deliver", "jobs.kind.cook", "jobs.kind.gather", "jobs.kind.lost", "jobs.kind.repair"]}
    data = {
        "source": "tools/mockup_talk_assets.py",
        "pieces": pieces(),
        "font": font(),
        "coin": uri(Image.open(A / "items/ui/coin.png").convert("RGBA")),
        "items": items,
        "words": words,
        "colours": {"TEXT": "#eab798", "TEXT_DIM": "#cfaa81", "TITLE": "#faaf5c", "LABEL": "#fffefe",
                    "INK": "#010000", "WELL": "#011c34", "OAK_LABEL": "#010000"},
        "town": town, "kind": kind,
        "buy": buy_rows, "sell": sell_rows, "shelves": SHELVES, "shelf_picture": SHELF_PICTURE, "jobs": job_rows, "purse": 37,
        "basket": basket, "today": {c: V[c][0] for c in basket_codes}, "margin": MARGIN, "rate": RATE, "buy_share": T["buy_share"],
    }
    OUT.write_text("// Generated by tools/mockup_talk_assets.py from the game's own assets; do not edit.\n"
                   "window.TALK_ASSETS = " + json.dumps(data, separators=(",", ":")) + ";\n")
    print(OUT, OUT.stat().st_size, "bytes;", len(items), "icons; buy", [(r["code"], r["price"]) for r in buy_rows],
          "sell", [(r["code"], r["price"]) for r in sell_rows], "jobs", [(j["kind"], j["reward"]) for j in job_rows])
    print({c: basket[c]["value"] for c in basket})


if __name__ == "__main__":
    main()
