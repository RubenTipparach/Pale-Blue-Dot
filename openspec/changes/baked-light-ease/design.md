# Design: baked-light-ease

## 0. Method

"Measured" means read off a capture or a log of the fast build
(`cargo build --profile fast -p pbd-app`, Apple M4 Max, Metal). "From code"
means read off the source and not isolated by a run.

A pop is a change of a surface's brightness between two consecutive frames
that no motion of the camera, the sun or the weather explains. It is measured
on the screen: a run that holds the camera still (or walks it) saves every
frame across the swap, and a fixed patch of the picture is averaged frame by
frame. The swap's frame is found by a log line written when the bake reaches
the GPU.

### Instruments (measurement only; no behaviour changes)

Each is a switch read from the environment, off by default, and changes
nothing a player sees:

- **`PBD_CLOCK_RUNS=<rate>`**: the clock keeps running in a capture or a run
  with `--time`, at `rate` times its own pace. Without it a capture pins the
  hour, and neither the sun's 2-degree re-bakes nor dusk ever happen in one.
- **`PBD_CAPTURE_RUN=<dir>,<first>,<count>`**: save every frame from `first`
  for `count` frames as PNGs in `dir`, beside the run's usual `--capture`.
  With `PBD_WINDOW` (frame-budget 2.9) the frames are small.
- **`BAKED_LIGHT` log lines** at each swap, with the frame number: a bounce
  landing (its version, whether the anchor moved, whether it fades), a lamp
  bake going up, a dusk relight going up, a town's bake swapped in (vertices,
  mean and largest change of sky and lamp), and a field-lit mesh whose corners
  were read again under a new field (how far they moved).

## 1. What swaps (from code)

| Bake | Where | When it changes | How it reaches the screen today |
| --- | --- | --- | --- |
| L4 sun bounce | `planet_bounce.rs::update_bounce`, `upload_fine` | sun moved 2 degrees; a new set | written whole; shown at once unless the anchor moved; zero from the landing frame until the new bake lands, then a 0.5 s fade |
| L7 lamp bake | `update_lamp_bake`, `upload_fine`, `lamp_bake_state` | a new set; a relight | two halves, 0.5 s cross-fade; but at a new anchor both halves are invalid and the flood is drawn until the new bake lands |
| Voxel light field (sky, lamp) | `ColumnTier::relight`, `upload_fine` | dusk and dawn (`switch_dusk_lamps`), an edit, a landing | written whole with the records; no fade |
| Town outside | `field_light.rs::bake_town_outsides` | a set reaching the town lands, a relight; the tier stops reaching it | sliced at 2 ms a frame, `UV_1` swapped whole; open sky swapped whole at once |
| Field-lit corners | `light_from_the_field` | the set or its light version changes | the row rewritten that frame |

Sections 2 to 6 measure each (to be filled from the runs).

## 7. Decisions

### 7.1 One easing rule

A bake that replaces another is shown as `mix(old, new, e(t))`, with
`t = (now - swapped) / BAKE_EASE_S` clamped to 0..1 and `e` the smoothstep.
The CPU computes `e(t)` and hands it to the shader (a uniform lane or a
mesh's row); the shader never decides when a fade starts. `BAKE_EASE_S` is
1 s: long enough that a lamp coming on reads as coming on, short enough that
a light is never visibly stale. Smoothstep, not linear, so the change has no
kink at either end. At 60 Hz the largest change in one frame is 2.5% of the
step (`1.5 / 60`), against 100% today.

### 7.2 Where the old bake cannot be read

A bake is laid out by the column tier's slots, and a set round a new anchor
has other slots. The old bake cannot be read against the new set, so it
cannot be faded from. Instead the new set's bounce and lamp bake are made in
the same pool task as the set, after it is built, and land in the same frame
as its records. Cell for cell they hold the light the old set's bakes held
(the same ground, lamps and sun), so the change at the landing is the
geometry's own, which `detail-fade` handles.

### 7.3 Dusk, not edits

The light field is eased only across a dusk or dawn relight: the columns and
slots are the same before and after, only the lamps changed. An edit changes
what is solid, and a dug face's old light is the rock's darkness: faded from,
it would flash dark. The light spec requires an edit's light at once.

### 7.4 Towns keep their last bake

A town's outside carries its bake in `UV_1` and, from this change, the bake
before it in the vertex colour (`COLOR`, sky and lamp in `xy`). The vertex
colour is given at spawn, so the mesh's layout never changes and no pipeline
is specialised mid-flight. `field_lit.wgsl` reads `COLOR` as light, never as
a tint, on a row marked as a town's outside.
