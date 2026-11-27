#!/usr/bin/env python3
"""Draw a bridge's two cut-out cards (`townsfolk` design 42p, the owner's
survey P7, 2026-11-26: "You should use a texture with alpha cutout instead
of adding geom"):

- assets/textures/settlement/bridgerail.png: a rail along one side of a
  bridge cell, a metre high: a post at each end, a hand rail on top, a mid
  rail and a bottom rail, clear between. Mapped once across the side.
- assets/textures/settlement/bridgetrestle.png: the legs under the deck,
  down to the bed or the bank: a leg at each end, a waling and an X of
  braces, clear between. Repeats a metre down, seamless top to bottom.

64 x 32 each. Every colour is the timber's or the plank's (each pixel is
drawn as a tone and snapped to the nearest colour of theirs), the grain and
the pegs are painted in, there is no outline, and a clear texel is alpha 0
(the manifest cuts both). `roads::tests::the_bridge_cards_are_drawn_in_the_timber_s_palette`
(pbd-app) holds the palette, the holes and the manifest rows; `--check`
holds the files to this script.

    python3 tools/art/bridge_cards.py            # write the PNGs
    python3 tools/art/bridge_cards.py --check    # fail if they differ

Deterministic: a fixed seed, integer maths, and the sources' colours sorted.
"""

import random
import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[2] / "assets" / "textures" / "settlement"
SOURCES = ("timber", "plank")
W, H = 64, 32
POST = 5  # a post's or a leg's width, px (about 0.13 m across a side)


def palette():
    colours = set()
    for name in SOURCES:
        image = Image.open(ROOT / f"{name}.png").convert("RGB")
        colours.update(image.getdata())
    # No outline: the plank's black is not a colour of the wood.
    return sorted(c for c in colours if sum(c) > 120)


def nearest(colours, rgb):
    return min(colours, key=lambda c: sum((a - b) ** 2 for a, b in zip(c, rgb)))


class Card:
    """A tone map: (x, y) -> rgb, the rest clear."""

    def __init__(self, seed):
        self.rng = random.Random(seed)
        self.tone = {}

    def beam(self, cells, base, along):
        """Timber over `cells`, its grain running `along` ('x' or 'y'), lit
        from above: the top edge light, the bottom edge in shade."""
        cells = sorted(set(cells))
        ys = {}
        for x, y in cells:
            ys.setdefault(x if along == "y" else y, []).append((x, y))
        # A streak of grain per row (or column) of the beam.
        streak = {}
        for x, y in cells:
            key = y if along == "x" else x
            if key not in streak:
                streak[key] = self.rng.choice((0, 0, -6, 5, -10))
        top = min(y for _, y in cells)
        bottom = max(y for _, y in cells)
        left = min(x for x, _ in cells)
        for x, y in cells:
            lift = streak[y if along == "x" else x]
            if along == "x":
                if y == top:
                    lift += 16
                elif y == bottom:
                    lift -= 18
            else:
                if x == left:
                    lift += 12
                elif x == max(xx for xx, yy in cells if yy == y):
                    lift -= 16
            # The grain's flecks, a shade off here and there.
            lift += self.rng.choice((0, 0, 0, 0, 0, -7, 6))
            self.tone[(x, y)] = tuple(max(0, min(255, c + lift)) for c in base)

    def peg(self, x, y):
        """A wooden peg where a rail meets a post: a dark pixel and a lit one."""
        if (x, y) in self.tone:
            r, g, b = self.tone[(x, y)]
            self.tone[(x, y)] = (r - 26, g - 20, b - 14)
        if (x, y - 1) in self.tone:
            r, g, b = self.tone[(x, y - 1)]
            self.tone[(x, y - 1)] = (r + 10, g + 8, b + 6)

    def image(self, colours):
        out = Image.new("RGBA", (W, H), (0, 0, 0, 0))
        cache = {}
        for (x, y), rgb in self.tone.items():
            if not (0 <= x < W and 0 <= y < H):
                continue
            if rgb not in cache:
                cache[rgb] = nearest(colours, rgb)
            out.putpixel((x, y), cache[rgb] + (255,))
        return out


def rect(x0, y0, x1, y1):
    return [(x, y) for x in range(x0, x1) for y in range(y0, y1)]


def rail():
    card = Card(4207)
    post = (98, 64, 38)
    bar = (124, 88, 52)
    # The rails first, then the posts over their ends.
    card.beam(rect(0, 0, W, 4), bar, "x")  # the hand rail
    card.beam(rect(0, 14, W, 17), bar, "x")  # the mid rail
    card.beam(rect(0, 27, W, 30), bar, "x")  # the bottom rail
    for x0 in (0, W - POST):
        card.beam(rect(x0, 0, x0 + POST, H), post, "y")
        for y in (2, 15, 28):
            card.peg(x0 + POST // 2, y)
    # A post halfway along, so a long side is not a single span.
    mid = W // 2 - 2
    card.beam(rect(mid, 3, mid + 4, 28), post, "y")
    for y in (15,):
        card.peg(mid + 2, y)
    return card


def trestle():
    card = Card(5113)
    leg = (92, 60, 35)
    brace = (112, 78, 46)
    # The X of braces, three px thick, corner to corner of the span between
    # the legs: on the next repeat down it starts again at the legs.
    cells = []
    span = W - 2 * POST
    for y in range(H):
        t = y / (H - 1)
        for d in (-1, 0, 1):
            a = POST + round(t * (span - 1)) + d
            b = W - POST - 1 - round(t * (span - 1)) + d
            cells += [(a, y), (b, y)]
    card.beam([(x, y) for x, y in cells if POST <= x < W - POST], brace, "y")
    # The waling across the top of each repeat, under the deck.
    card.beam(rect(0, 0, W, 4), brace, "x")
    for x0 in (0, W - POST):
        card.beam(rect(x0, 0, x0 + POST, H), leg, "y")
        card.peg(x0 + POST // 2, 2)
    return card


def main():
    colours = palette()
    images = {
        "bridgerail": rail().image(colours),
        "bridgetrestle": trestle().image(colours),
    }
    check = "--check" in sys.argv[1:]
    stale = False
    for name, image in images.items():
        path = ROOT / f"{name}.png"
        if check:
            have = Image.open(path).convert("RGBA")
            if list(have.getdata()) != list(image.getdata()):
                print(f"stale: {path}")
                stale = True
        else:
            image.save(path)
            print(f"wrote {path}")
    if check:
        if stale:
            sys.exit(1)
        print("bridge cards are current")


if __name__ == "__main__":
    main()
