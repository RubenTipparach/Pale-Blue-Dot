# Design: lens-drop-refraction

## The drop space

The lens works in BigWings' frame (`drop_uv`): the screen centred, y up, in
screen heights, x times the aspect, all times `rain_lens_scale` (S, 2). The
drop field has three kinds of drop, and a layer's drop has trail beads as
well. Each one is a smoothstep bump. Its radius `R` (where its raw field
reaches zero) follows from its cell and from `rain_lens_size` (0.7):

| Kind | Cell | Bump | R, drop space | R at 1440x900 | Visible across |
| --- | --- | --- | ---: | ---: | ---: |
| static | 1/(40·size) square | `ss(0.3,0,d)` | 0.3/(40·size) = 0.0107 | 4.8 px | about 6 px |
| near layer, drop | 1/(12·size) by 1/(2·size), y stretched 6 | `ss(0.4,0,d)` | 0.4/(12·size) = 0.0476 | 21 px | about 27 px |
| near layer, bead | the same cell, a bead every 0.1 down | `ss(0.3,0,dd)` | 0.3/(12·size) = 0.0357 across (0.03 down) | 16 px | about 20 px |
| far layer | the near cell / 1.85 | the same | the near's / 1.85 | 11.6 px (drop) | about 15 px |

A drop's distance is the same across and down in drop space. A bead's is
not quite: it is 0.3 of a cell across but 0.03 of drop space down, whatever
the size. A pixel shows a drop where the field summed reaches 0.3, which is
`c = smoothstep(0.3, 1, static + near + far)`. That is the drop's alpha.

## What the lens does today, with numbers

```text
n      = (c(p + e·x) - c(p), -(c(p + e·y) - c(p)))     e = 0.001 drop space; y flipped to the uv's down
sample = uv - n * rain_lens_refract / S                 rain_lens_refract = 1.7 (Tenebris)
```

- **Scale.** `n` is a difference over 0.001 of drop space, so it is about
  0.001 times the slope. On a near drop the steepest `c` is about 55 per
  unit of drop space, so `n` is about 0.055 and the offset 0.047 of the
  screen: 42 px down or 67 px across at 1440x900, against a 13.7 px visible
  radius. A far drop is 1.85 times steeper: 78 px and 125 px. A static drop's
  amplitude is up to 2 (`l0`) and the `smoothstep(0.3,1)` clips it to a
  1-2 px rim, so its rim steps `c` by up to about 0.6 per difference. That
  sends the rim hundreds of pixels away.
- **Measured** over the whole frame, with a CPU mirror of the drop field at
  rain 1 and the shipped knobs, at three times (t = 0.7, 3.7, 11.3), for the
  pixels in a drop (`c > 0.05`, about 14.5% of the frame):

  | | median | p90 | p99 | max | sample lands where no drop is | sample over 20 px below the pixel |
  | --- | ---: | ---: | ---: | ---: | ---: | ---: |
  | today | 69-71 px | 219-225 px | 447-452 px | 1224 px | 67-69% | 29% |

- **Sign.** `n` is the gradient of `c`, so it points at the drop's centre,
  and `uv - n` samples away from it. The top half of a drop shows what is
  above the drop and the bottom half what is below, each many radii off. In
  the beach captures (`docs/screenshots/lens-drop-refraction/`), a drop over
  the sky next to the treeline has a dark crescent along its lower rim
  (it is sampling the trees), and a drop on the trees has a sky-white
  crescent along its upper rim. BigWings' Heartfelt samples `UV + n`, and a
  water drop on glass is a convex lens that shows the scene inverted:
  against a bright sky it is dark on top and bright underneath. Tenebris's
  port has `v_uv - n*1.7`, which is the mirror image, and that is what reads
  "upside down".
- **Aspect.** The slope is per screen height in both axes, but `n.x` is
  added to `uv.x`, a share of the width. Across the frame the sideways
  offsets average 1.54-1.58 times the vertical ones (1440/900 = 1.6).

## The new offset

```text
field  = (coverage, h, trail)                          at p, p + e·x, p + e·y (the same three evaluations as today)
h      = R_s² · static/max(l0,1) + Σ_layers (R_d² · drop + R_b² · beads)    R = (0.3/40, 0.4/12, 0.3/12) / size, the far layer's / 1.85
o      = (bend / 6) · (h(p+e·x) - h, h(p+e·y) - h) / e    drop space, y up
o      = o · min(1, bend·R_d / |o|)                    the seams' cap
sample = uv + (o.x / aspect, -o.y) / S
```

For a bump `f(u) = smoothstep(1, 0, u)` with `u = ρ/R`, `R²·∇f` has length
`R·|f'(u)| = R·6u(1-u)`. So the offset is `bend · (1-u) · ρ`, pointing toward
the centre. A pixel at distance ρ from the centre samples at
`ρ·(1 - bend·(1-u))`, whatever the drop's size:

- `bend < 1`: an upright magnifier, `1/(1-bend)` times at the centre and
  none at the rim;
- `1 <= bend <= 2`: the middle inverts, still within the outline;
- `bend <= 2`: `|1 - bend·(1-u)|·u <= 1`, so the sample never leaves the
  drop's field.

Where two drops overlap their slopes add, as two lenses would.

Each kind is weighted by its own radius, so none is over-bent:

- **The beads.** `rd_drop_layer` added a layer's trail beads into its drop.
  Weighted by the drop's radius, a bead (0.75 of it) bent about 1.8 times
  the share, and its middle inverted. The layer now returns the drop and
  its beads apart.
- **The static drops.** A static drop's height reaches 2 at full rain
  (`l0`). Its lens part is divided by `max(l0, 1)`, so its middle bends by
  the share, not twice it.

The alpha `c` is the field summed exactly as before.

**The cap.** Cells hold different drops (`keep`, `fade`, the column's
offset), so the field steps at a cell's edge where a drop crosses it. A
difference taken across that seam has no meaning. No pixel inside a drop
bends further than `bend` times the largest drop's radius, so the offset is
clamped there: 10.7 px at 1440x900 for `bend` 0.5. In the mirror,
0.07-0.15% of drop pixels are clamped.

**Measured** with the same mirror, frame and pixels as the table above. The
inverted share counts the pixels that move over 1 px and whose sample lies
past their drop's centre:

| `bend` | median | p90 | p99 | max | sample lands where no drop is | inverted | across / down |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| today | 69-71 px | 219-225 px | 447-452 px | 1224 px | 67-69% | - | 1.54-1.58 |
| **0.5** | 1.2 px | 2.6-2.7 px | 4.2-4.5 px | 10.7 px | 0.09-0.11% | 0.9-1.4% | 0.97-1.01 |
| 0.8 | 1.9 px | 4.2-4.3 px | 6.8-7.2 px | 17.1 px | 0.24-0.28% | 4.6-6.2% | 0.97-1.01 |

Before the beads were split out, 0.8 inverted 20% and 0.5 inverted 4.5%.
What is left of "lands where no drop is" is the anti-aliased rim, where a
pixel is 5% drop and its sample sits just past the 0.3 sum.

## The edge

A drop that shows only what is behind it disappears against a flat sky. The
first after-capture at the beach had no visible drop over the sky, and over
the trees only faint swellings. What shows a real drop against an overcast
sky is its edge. There the surface is steep, so the light through it comes
from far off, and more of it is reflected away, and the rim reads darker
than the sky inside it. The lens draws that edge as a share of the light
lost where the drop's own alpha `c` passes a half:

```text
refracted *= 1 - rain_lens_rim * 4c(1-c)
```

This is zero at the drop's middle (`c` = 1) and outside it (`c` = 0), and it
is mixed over the scene by `c` like the rest of the drop, so the darkest
point is `0.59 * rain_lens_rim` at `c` = 2/3. The term needs no other
information: it is the drop's own outline, at every size. A drop over the
sky becomes a faint darker ring around sky, and a drop over the ground a
magnified patch with a soft dark rim. Neither shows anything that is not
behind the drop.

## The light at the lower edge

Over the ground the edge alone is not enough. The second captures had faint
rings on the sky but almost nothing on the wet soil and sand. There, the old
lens's drops read as water only because their upper rims sampled the bright
sky hundreds of pixels above: bright arcs on top of each drop. That is the
other half of "upside down". A drop on glass is a lens, and the light from
the sky above it gathers at its **lower** edge, while its upper edge is dark.

The lens draws that light without sampling anything outside the drop. It
uses the sky's mean radiance where the eye is (`rain_sky.rgb`, the zeroth
spherical harmonic the rain volume is already lit by: grey under overcast,
dim at night). It adds that light along the part of the edge band that
faces down:

```text
lower     = max(0, ĝ.y)                     ĝ the lens slope's direction in drop space (y up): +1 at the bottom of a drop
refracted = refracted * (1 - rim * 4c(1-c)) + glint * 4c(1-c) * lower * rain_sky.rgb
```

So a drop over the sky has a darker top and a lighter bottom, a subtle
lighter/darker distortion of the sky. A drop over dark ground has a soft
bright crescent under a dark rim. The crescent is the sky's own light, so
it dims with the sky at dusk and at night. At midnight in the night shot,
the drops are dim smudges on the dark sky.

## The knobs

| Knob | Lane | Default | Valid | Meaning |
| --- | --- | ---: | --- | --- |
| `rain_lens_bend` | `lens.y` | 0.5 | finite, 0..=2 | the share of a pixel's distance from its drop's centre the drop bends it by, at the middle; replaces `rain_lens_refract` |
| `rain_lens_rim` | `lens_look.x` | 0.2 | finite, 0..=1 | the share of light lost at a drop's edge |
| `rain_lens_glint` | `lens_look.y` | 1.0 | finite, not negative | the sky's mean light gathered on a drop's lower edge; a lens gathers light, so it may pass 1 |

`lens_look` is a new lane after `mist_look`; zw are spare, and the uniform's
size test counts it. A zero turns each term off. The `.ron` stays equal to
the code defaults.

The defaults were picked from the beach captures (recommendation taken: ask
only with screenshots):

- **bend:** at this size the 0.5 and 0.8 shots looked alike, and 0.5
  inverts four times fewer pixels;
- **rim:** 0.3 made the drops on the sky busy, and 0.2 is subtle;
- **glint:** 0.5 was barely visible on the wet soil, and 1.0 reads as water
  there without lighting the sky drops past a faint underside.

## What does not change

- The drop field: where drops are, their size, count, fall, trails and fade.
- The alpha (`c`), the trail channel, and the wipe through the mist.
- The emerge drips call the same function, so they get the fix too.
- The cost: the same three evaluations of the drop field per pixel. The
  field comes back as a `vec3` (coverage, lens height, trail), and the lens
  adds one texture-free term. The frame cost was not measured (no perf
  suite for this change).

## Verification

- **A GPU test** (`lens_gpu_tests`, Metal or Vulkan) runs the shipped
  `water.wgsl` drop section over a 720x450 lens in full rain at the
  default knobs, at three drop times. In each case:
  - every drop pixel looks within the cap;
  - at least 97% of moving drop pixels look up their drop's lens height,
    toward its centre (measured 99.84-99.88%);
  - at least 99% look at a point a drop covers (measured 99.88-99.95%);
  - the light is on the lower half of every drop (100%);
  - the same drops at aspects 1 and 2 bend each pixel by the same number of
    pixels across and down.

  With the offset's sign flipped back to outward, the uphill share is 0.1%
  and the test fails. It is skipped, loudly, where there is no adapter.
- **A source test** checks that the drop section compiles on its own,
  reading no lane but `lens`, `screen` and `mist_look`, and that the lens
  samples at `uv + lens.offset`.
- **Config:** the shipped `.ron` equals the defaults, each knob's range is
  enforced (NaN refused), and `rain_lens_refract` is an unknown field.
- **Captures** before and after at the owner's beach (`--at 12.948 8.135`,
  rain 1), four directions at noon and one at midnight, with side-by-sides
  and 1:1 crops, in `docs/screenshots/lens-drop-refraction/`.
