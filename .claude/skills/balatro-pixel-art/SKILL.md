---
name: balatro-pixel-art
description: Draw true pixel art in code in the style of Balatro's jokers: a tiny native canvas, one bold subject filling it, flat 3 to 5 tone ramps lit from the top left, a dark seam between parts, a 1 px outline, single-pixel glints, no anti-aliasing, every colour from one palette (Sweaft-64 by default). A small PIL and numpy kit (scripts/pixelkit.py) draws masks, shades them, outlines, dithers, checks the palette and makes nearest-neighbour contact sheets. Use whenever sprites, card art, icons, emblems or item art are wanted "in pixel art", "like Balatro", "pixel style", "in the palette", or existing art has to be redrawn as pixel art ("redraw the ships as pixel art", "make an icon for the new trick", "pixel art for the shop item", "match the jokers").
metadata:
  author: Claude Code, for the owner's projects (first in Tactical Airsuite)
  version: "1.0"
---

> **In this repository (Pale Blue Dot).** Copied from `RubenTipparach/rubens-solitare-collection`
> (`.claude/skills/balatro-pixel-art`, commit cdb3b24) on 2026-11-12, at the owner's word: "the card
> game rubens solitaire collection uses a different art style, can you adopt that art style instead?"
> It replaces the crayon kit (`hand-drawn-illustration`) for every painted icon, portrait and
> mockup-only texture in `docs/mockups/` (scripts in `tools/art/*_art.py`). `references/` holds 16 of
> that game's sprites (items, tricks, bosses; `_contact.png` sheets them) to judge against. Where the
> game already ships a texture (`assets/tilesets/atlas.png`, `assets/textures/settlement/`), use the
> game's, never a redrawn one. The Tactical Airsuite notes below are that repository's history.

> **In this repository (Tactical Airsuite).** Written here on 2026-10-21 at the owner's word:
> "Rewrite skill based on balatro pixel art and redraw everything in this style" (with a tier list of
> Balatro's jokers), after "Here is a nice color palletes for you
> https://lospec.com/palette-list/sweaft-64" and "It would be nice if the background was more pixel
> art. Same with cards and card art". It draws every sprite the cards load: `tools/art/ship_art.py`
> (the ships, 56 x 40, and the two command ships, 96 x 64), `tools/art/trick_icons.py` (trick emblems,
> 56 x 56) and `tools/art/shop_icons.py` (ammunition and refits, 56 x 56), the last two through
> `tools/art/emblems.py`; `tools/art/contact_sheet.py --ref <tier list>` writes
> `docs/screenshots/pixel-art-contact.png`. The game's palette is `src/data/palette.js`; the kit's
> `SWEAFT_64` is the same list. The card face (72 x 100) draws these sprites pixel exact, never scaled
> by a fraction.

# Balatro-style pixel art, drawn in code

Pixel art is not a picture made small. Every pixel is placed on purpose at the size it is shown, from a
handful of colours, and only ever scaled up by whole numbers with nearest neighbour. This skill draws it
from code so it can be changed and redrawn like any other source file.

## The look, from Balatro's jokers

Study of the jokers (Vampire, Cavendish, Rocket, Seltzer, Bloodstone, Canio, Campfire and the rest):

1. **Tiny native canvas.** A joker is 71 x 95 px with its frame; the subject sits in about 50 x 60 of
   it. Draw at that size. Never draw big and shrink: shrinking is what makes mush.
2. **One bold subject filling the frame.** A banana, a rocket, a face, a gem: centred, nearly touching
   the edges, nothing small and fussy around it. If it needs a caption to be read, the silhouette is
   wrong.
3. **A chunky silhouette that reads at 1x.** Thick limbs, fat shapes, exaggerated features (a rocket is
   all fins and plume). Anything one pixel thin disappears behind its outline; make it 2 px or drop it.
4. **A 1 px dark outline round everything.** Near-black or the darkest tone of the subject's ramp. Four
   connected (no diagonal corner pixels), so the edge reads rounded and clean.
5. **Dark seams between parts.** Where one part lies over another (a sleeve over a body, a wing over a
   fuselage) a 1 px line in the darker part's deepest tone separates them. This is what keeps busy
   subjects readable.
6. **Flat colour in short ramps.** Each material has 2 to 4 tones (shadow, body, light, sometimes a
   deep shadow), 5 at most. No gradients. Light comes from the top left: the top and left edges one
   tone up, the bottom and right edges one or two tones down, a cylinder banded along its length, a
   sphere lighter towards its upper left.
7. **A few single highlight pixels.** One white or palest pixel on a glossy spot (an eye, a lens, a
   gem facet, the top of a dome). Sparks are a white pixel in a plus of the palest warm tone.
8. **No anti-aliasing, no soft edges, no half transparency.** Alpha is 0 or 255. A gradient, where one
   is wanted (a glow, a sky, a shimmer), is an ordered dither between two palette tones, or a
   patterned fill.
9. **Saturated colour that pops on its card.** The subject is brighter and more saturated than its
   background; light (fire, plumes, holograms) is the brightest thing on the card and is never
   outlined: it glows over the outline.
10. **Every colour from the palette.** No colour is invented; ramps are picked from it. The kit's
    `save()` refuses a sprite with one pixel off it.

## The kit: `scripts/pixelkit.py`

PIL and numpy only. Import it by path (`sys.path.insert(0, '<skill>/scripts')`).

```python
from pixelkit import Sprite, RAMPS, INK, save, sheet, upscale

sp = Sprite(56, 40, flip=False)            # flip=True mirrors every shape and plot (a ship facing left)
hull = sp.ellipse(4, 10, 50, 24)           # a mask: a numpy bool array; | & ~ combine masks
sp.part(hull, RAMPS['sand'], 'cyl_h')      # shaded fill, seamed where it lies over earlier parts
sp.part(sp.rect(20, 6, 30, 10), RAMPS['brass'])        # 'rim' is the default shading
sp.plot(22, 7, '#fffefe')                  # a glint
sp.outline()                               # 1 px INK round the silhouette (glow excluded)
sp.part(sp.taper(10, 25, 10, 33, 4), RAMPS['flame'], 'inset', glow=True)   # light, after the outline
save(sp.image(), 'out.png')                # checks palette and alpha, then writes the native-size PNG
```

- **Masks, never anti-aliased**: `rect`, `ellipse`, `circle`, `poly`, `line(pts, width)`,
  `taper(x0, y0, x1, y1, w0, w1)` (plumes, blades, booms), `star(cx, cy, r_out, r_in, points)`,
  `empty()`. Coordinates are pixel coordinates, inclusive.
- **Shading** (`part(mask, ramp, mode, base=None, depth=1, seam=True, glow=False, seam_color=None)`):
  `rim` (the body tone, lit top and left edge, `depth` px of shadow on the bottom and right), `cyl_h`
  (a cylinder lying along x), `cyl_v` (standing up), `sphere`, `inset` (by distance from the edge:
  plumes, gems, glows; the edge darkest), `flat`. `base` is the ramp index of the body tone (default
  the second lightest). `tones()` returns the tone index per pixel for custom shading.
- **Dither**: `dither(mask, c1, c2, amount)` fills with a 4x4 Bayer pattern, `amount` of it c2.
- **Outline**: `outline(color=INK, diagonal=False)` after the solid parts, before the light.
- **Ramps** (`RAMPS`, dark to light, every entry from Sweaft-64): `ink`, `metal`, `steel`, `soot`,
  `sand`, `olive`, `khaki`, `brass`, `gold`, `rust`, `wood`, `bone`, `paper`, `glass`, `amber`,
  `cyan`, `violet`, `red`, `green`, `blue`, `flame`, `night`. Add a ramp by picking 3 to 5 palette
  colours of one hue from dark to light, with a visible step between each.
- **Out**: `check(img)`, `save(img, path)`, `upscale(img, n)`, `sheet([(label, img)], scale=4, cols=8)`.
- **Another palette**: `use_palette(list_of_hex)`; then remap the ramps to it (pick each ramp again
  from the new palette by hand: snapping a ramp blindly can merge two of its tones).

## How to draw a sprite

1. **Pick the size the game shows it at**, and draw at exactly that size. Leave a pixel of margin for
   the outline.
2. **Block the silhouette** with masks, back to front (far wing, body, near wing). Check it at 1x
   filled in one colour: if it does not read as the thing, change the shapes before shading anything.
3. **Shade each part as it is drawn** with `part()`: one ramp per material, the far side of a thing a
   ramp step darker (`[ramp[0]] + ramp[:-1]`). Let the seams separate parts; do not draw extra lines.
4. **Add the identity colour** (a team or suit colour on fins, tips, stripes, flags) as its own part,
   seamed, so it reads as a painted panel.
5. **Glints**: one to three single pixels, not more.
6. **Outline**, then **light** (plumes, flashes, bursts, holograms, sparks) with `glow=True`.
7. **Save and sheet it**: look at it at 4x and at 1x beside its neighbours. Fix what reads badly and
   look again. Two rounds at least.

## Pitfalls

- A 1 px detail beside an outline vanishes; 2 px is the minimum for anything that must read.
- Too many tones on a small part turns it to noise: on anything under 6 px use `flat` or `rim` with a
  3-tone ramp.
- Do not light the mirrored sprite from the other side: the kit mirrors the masks, then shades, so the
  light stays top left for both facings.
- Symbols on a flag or shield need contrast with the field, not just a different hue: cream on gold
  fails, rust on gold reads.
- Never resample a finished sprite to another size. If it is needed bigger, draw it again bigger (the
  kit's shapes take a scale, as `cruiser(d, k=1.68)` does in this repository).
