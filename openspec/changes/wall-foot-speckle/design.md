# Design: wall-foot-speckle

On `claude/perf-tuning` at `d43d018`, the fast build, an M4 Max Mac, Metal,
2880 x 1800, fixed-step captures of 200 frames, a new world each.

## 1. Where it shows

`--walk --at 30 177.5 --pitch -35 --time 22 --rain 0 --weather-at 7200
--torch`: a sandy coast with 1 m sand and grass steps (the sun is up at this
longitude at 22:00 UTC; the torch is in hand). Cropped at full resolution
(x 1080-1330, y 1100-1300, the shaded sand at a wall's foot), the sand is
covered in fine grey-blue dots, single pixels, densest near the wall and
gone about a metre out. Holbrook's lane (`--at 29.69673 1.19287 --yaw
179.4 --time 22`) shows the owner's second screenshot's setting.

## 2. The bisection

Each switch alone, the same spot and flags:

| Switch | Speckle |
| --- | --- |
| none | yes |
| `PBD_BOUNCE=0` (no sun bounce) | yes |
| `PBD_SOFT_SHADOWS=0` (3 x 3 filter, no PCSS) | yes |
| `PBD_MSAA=1` (one sample a pixel) | yes, unchanged |
| `--no-shadows` | no |
| `PBD_LAMP_BAKE=0` (the flood instead of the bake) | no |

The owner's z-fighting question: the geometry is identical in every row and
a single sample per pixel changes nothing, while switching one lighting term
removes it; a depth fight would also crawl with the camera and does not
depend on a light. `--no-shadows` removes it because it lights the sand in
full sun, which swamps the lamp term; the lamp bake is the term.

## 3. Why

`lamp_near(half, position, n, slot)`:

```wgsl
let q = position + n*0.5;
let pair = column_pair(q, slot);
let alt = length(q) - params.settings.x - COLUMN_BASE_M - 0.5;
let l0 = u32(floor(alt));
... lamp_column(half, s, l0, fract(alt), ...)
```

and `lamp_column` reads the cells `l0` and `l0 + 1`, returning the other
when one is rock. For a floor at whole altitude `h`, `q` is at `h + 0.5`
and `alt` is `h`, give or take the f32 step of `length(q)` at 4,800 m
(2^-11 m = 0.49 mm). So per pixel:

- `l0 = h - 1`: cell `h - 1` is rock, so the answer is cell `h`.
- `l0 = h`: cell `h` with cell `h + 1` at a weight of about zero.

In the floor's own column both are cell `h`, the air over the floor. In the
wall's column (blended in by `column_pair` at up to a half on the border,
fading to none 0.8 m in), cell `h` is rock: the bake leaves a rock cell
empty (`pbd_core::lamp_bake::bake` skips solid cells: no lamps, no bounce,
not in reach), so the first case blends black in and the second reads cell
`h + 1`, the lit air on top of the wall. Pixels flip between the two: dark
dots on the lit sand in a band 0.8 m wide along every wall foot.
`bounce_near` has the same `floor(alt)` and the same flip (its
`bounce_column` falls through to the zero rock cell in the first case); it
is hidden under the lamp term in these shots.

## 4. The fix

```wgsl
const LAYER_SNAP_M: f32 = 0.01;
fn sample_layer(alt: f32) -> f32 {
    let whole = round(alt);
    return select(alt, whole, abs(alt - whole) < LAYER_SNAP_M);
}
```

`lamp_near` and `bounce_near` take `alt = sample_layer(...)`. A floor (or a
ceiling, whose `q` sits at `h - 0.5`) then reads `l0 = h` with `f = 0`
exactly at every pixel: the air cell it faces, which is the answer most of
its pixels already gave. A wall's sample moves continuously with height and
is not near a whole number except along one 2 cm line, where the two layers
it blends are within 1 % of each other anyway. 1 cm is about twenty f32
steps at this radius, ample for interpolation error, and still a small
fraction of a layer.

## 5. What else was looked at

- The column blend itself (`column_pair`) already uses the plane test that
  `bounce`'s earlier speckle fix introduced, and its weight is smooth.
- The PCSS soft edge shows a fine regular stipple from its Poisson taps at
  the shadow's edge; it is a pattern, not the reported dots, and removing
  PCSS did not remove the dots.
