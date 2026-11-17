# Design: far trees lit by their own relief

## 1. The finding

**How a far tree is drawn.** `flora-v2` design 10.3 bakes each variant's
near model on the CPU (`pbd_core::tree_bake`), from 8 bearings, into 48 x 72
px views. Inside 0.85 x `tree_near_m` (37 m) a tree is its near model, inside
0.85 x `tree_mid_m` (76 m) four crossed mid cards, past that a billboard
turned to the eye (`planet_surface.wgsl`, the `vertex < CLUTTER_FIRST_VERTEX`
branch).

**How it is lit today.** Both impostors go through the terrain's own
fragment (`shade_terrain`), the same as the near trees and the ground:

- the sky (its spherical harmonics) and the sun's colour at that elevation;
- the sun's cascades (`sun_shadow_soft`), which reach about 900 m on foot,
  so they do fall on far trees;
- the field's sky light, the lamps, the haze.

What differs is what goes in:

| | near model | mid cards and billboard (before) |
| --- | --- | --- |
| normal | per vertex, the model's own: a leaf card's normal leans 60% out from the crown's centre (`leaf_card`'s `n_out`), turned up | **one per tree**: toward the eye, laid level, lifted by 0.35 (`IMPOSTOR_LIFT`, `flora-v2` 13.4) |
| shade | per vertex, root-to-tip AO, `mix(0.45, 1, ao)` | **one constant**, `NEAR_CROWN_SHADE` 0.87 (the near crown's mean), folded into the albedo match |
| sun shadow | the cascades, including the crown shading its own lower leaves | the cascades on a flat card: shadow from other things only |
| bake's own light | none | a fixed 1.15 key on leaves (taken out by the mean match), and 0.85 to 1.15 on bark |

So every pixel of a far crown takes the same light. It is an ambient-only
look in practice: the sun term is the same on the lit and the shaded side
of the crown, and the bake carries no light of its own.

**The numbers.** Fixed-step `--capture` stills on the M4 Max Mac (Metal),
2880 x 1800, a fresh world each shot, the jungle at `--at 28.706 3.031`. The
crop is the lower canopy; "luma" is Rec. 709 over its green pixels, sRGB
0..255.

The hand-off: one view (`--fly --height 30 --yaw 270 --pitch -28`) with
`scatter.ron`'s rings forcing every tree to one representation.

| 10:00 | luma mean | luma std | p90 |
| --- | ---: | ---: | ---: |
| all near models | 43.0 | 20.1 | 71.0 |
| all mid cards | 33.9 (-21%) | 12.2 | 46.9 |
| all billboards | 38.0 (-12%) | 16.8 | 52.2 |

The mid ring is darker than the near model it replaces at 37 m, and flatter.
That is a lighting step at the hand-off as well as the owner's flat look.

The owner's view (`--height 40 --pitch -15`, noon): luma std 19.9 over the
canopy. Into a morning sun (`--yaw 180 --time 8`): std **5.8**, the whole
hillside one tone.

## 2. The options

| | what | frame cost | memory | matches the near tree? |
| --- | --- | --- | --- | --- |
| A | **Baked relief**: a normal-and-AO strip beside each colour strip, rasterised from the near model in the same pass | one more 4-byte storage read per impostor pixel, a cross and a normalise | +4.6 MiB of kit (6.0 to 10.6 MiB); bake +45 ms on its own thread behind the loading screen | yes: it is the near model's own normals and AO |
| B | **Spheroid normal**: a normal from the pixel's place in the card, as if the crown were an ellipsoid, darker toward the bottom and away from the sun | a few ALU ops | none | roughly: a smooth ball, not the near crown's lumps |
| C | **Per-tree tint** | none | none | already there: the near and far trees share `cell_variation` (0.94 to 1.06 by seed) |
| D | **Cascades on the card** | none more | none | already there (design 1); a flat card cannot shadow itself |

B is cheaper but invents a shape the near model does not have, so a tree
changes look at 37 m. A costs one read and is the near model's data, which
is what keeps the hand-off quiet. C and D are in the game already.

## 3. Decision: A, the baked relief

Recommendation taken (ask only with screenshots); the owner judges the
result on the before-and-after stills.

**The bake** (`tree_bake::bake_with_relief`). The rasteriser already finds,
per 3x pixel, the triangle in front. It now also keeps that point's
interpolated normal and AO. The normal is stored in the view's **level**
frame: across (the view's x), up (the tree's up), toward the view laid
level. When a 3 x 3 block folds to one pixel, the relief is the mean over
the block's drawn sub-pixels of the pass that gave the colour (the crown, or
the bark laid over it). Packed as three snorm bytes and the AO as a byte.
Clear pixels are zero.

**The kit** (`tree_kit::build_kit`). Each strip, leafy and bare, is followed
by its relief strip, `IMPOSTOR_RELIEF` (27648) words on. No header word
changes; the shader finds a pixel's relief at its colour's index plus that.

**The vertex.** A mid card shows view k (or k + 4 from behind), and the
billboard the view nearest the eye's bearing. Its normal output is now that
view's direction laid level, in the tree's turned frame. That is the frame
the relief is kept in.

**The fragment** (`impostor_relief`). Up is the radial; across is
`cross(up, level)`. The pixel's normal is rebuilt from the relief, and it
replaces the card's normal for everything after it: the sky, the sun, the
cascades' bias, the bounce. The pixel's AO goes in as the near pass's
`mix(0.45, 1, ao)`. On a crown pixel it goes over 0.87, because the crown's
albedo was already met to the near crown's under that mean.

**The knob.** `tree_impostor_relief` (`scatter.ron`, 0..1, validated) mixes
the baked normal with 13.4's standing normal and the pixel's shade with the
constant. Zero is the old card exactly. It rides in `light_ease.w`, a spare
lane of the uniform, so no struct layout changes.

**Art style.** Nothing is added to the colour: the bake's palette pixels
are unchanged, and the light is the scene's own (the sky, the sun, the
cascades), applied once by the plant's own normal (`flora-v2` 13.5). There
are no outlines and no new colours.

## 4. Result

The same frames, before and after, `docs/screenshots/tree-impostor-depth/`.

The hand-off, against all near models (43.0, std 20.1):

| | before | after |
| --- | --- | --- |
| all mid cards, 10:00 | 33.9 (-21%), std 12.2 | **45.1 (+5%)**, std 21.2 |
| all billboards, 10:00 | 38.0 (-12%), std 16.8 | 50.2 (+17%), std 22.3 |
| all mid cards, 07:30 (near 36.8) | 36.0 (-2%), std 10.9 | 38.6 (+5%), std 11.3 |
| all billboards, 07:30 | 35.9 (-2%), std 12.7 | 37.8 (+3%), std 13.1 |

At noon the near-to-mid step, the one a walker crosses at 37 m, goes from
-21% to +5%. The mid-to-far step at 76 m stays about where it was (+12%
before, +11% after). The billboards end up a little brighter than the near
models because a card has no self-shadow. The near crown's lower and inner
leaves sit in its own cascade shadow, and the baked AO gives back only part
of that. If the owner finds the far band too bright, the next step is a
baked sun-occlusion term. It is not built.

The full views, over the canopy:

| view | luma before | luma after | std before | std after |
| --- | ---: | ---: | ---: | ---: |
| owner-like, 40 m, noon | 52.4 | 64.2 | 19.9 | 20.2 |
| into a morning sun, 40 m | 37.3 | 43.9 | **5.8** | **12.9** |
| 120 m up, noon | 79.6 | 90.0 | 37.1 | 31.3 |
| morning, sun behind | 39.7 | 42.7 | 10.2 | 10.1 |
| evening, sun behind | 52.7 | 60.0 | 19.7 | 21.8 |
| night (02:00) | 14.1 | 16.0 | 3.5 | 3.3 |
| fields, 40 m | 81.5 | 83.7 | 35.7 | 32.9 |

Into a low sun the spread more than doubles. The crowns show lit rims and
shaded hollows where they were one tone. With the sun behind the camera,
every crown faces the light either way, so there is little to add. At night
the far band stays as dim as 13.4 left it (+2 luma).

**Frame cost.** Not measured on this sitting: another agent was building
and testing on the same Mac (`cargo test -p pbd-app`), and the perf rule
forbids a sitting beside a build. Owed:

```sh
# the base, a256f35a (claude/env-v2), built in a worktree of its own:
cargo build --release -p pbd-app        # copy to output/perf/env-v2-exe/pbd-app
# this branch:
cargo build --release -p pbd-app
python3 tools/perf_suite.py --scenarios clouds,walk,far-side --repeats 3 \
  --variant "before|output/perf/env-v2-exe/pbd-app||a256f35a" \
  --variant "after" --out docs/benchmarks/<date>-tree-impostor-depth
```

Expected cost: one storage-buffer read, a cross product and a normalise per
impostor fragment, and nothing anywhere else.

## 5. With the crossed cards (merge, 2026-11-17)

`crossed-tree-cards` landed beside this change: the far billboard became
two fixed cards (0 and 2) and a crown cap showing a ninth, top-down view.
The two fit together, and nothing of either is given up:

- **The strip is nine views.** `IMPOSTOR_RELIEF` is one strip's pixels,
  `STRIP_W * VIEW_H`, so it follows the strip from 27648 to **31104**
  words; the shader's constant moved with it and `planet_trees`' test holds
  the two equal. The kit grows by the relief of the top view as well.
- **The top view is relieved too.** `tree_bake::draw` keeps its normals in
  the tree's own frame (across x, up, z), which is bearing 0's level frame,
  so the shader's `level = t.x*sin(th) + t.z*cos(th)` at view 8
  (th = 2 pi) rebuilds them with no special case. The bake test checks the
  top view's crown normals lean up.
- **Every shown card and the cap carry their view's level direction** as
  the vertex normal, as the mid cards and billboard did here.
- **The standing normal is the crossed cards'**: at
  `tree_impostor_relief` below one the fragment mixes toward
  `crossed-tree-cards` 5's normal (toward the eye across the ground, lifted
  by `IMPOSTOR_LIFT`), rebuilt per pixel from the eye rather than per tree
  from its base. At the distances a card is drawn the two differ by well
  under a degree. At one (the default) every plane of a tree is lit by the
  same baked model normals, so the X still never shows one light arm and
  one dark.

The section 4 numbers were taken before the merge, on the billboard.
