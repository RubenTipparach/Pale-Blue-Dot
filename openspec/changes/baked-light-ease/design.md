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

Sections 2 to 6 measure each. The runs are fixed-step captures
(`--capture`, 60 Hz steps) in a 480 x 300 point window (`PBD_WINDOW`, 960 x
600 pixels) with `PBD_CAPTURE_RUN` saving every frame; "luma" is the mean
Rec. 709 luma (0..255, of the sRGB pixels) of a patch of the picture. The
fresh world, place and command of each are in
`docs/screenshots/baked-light-ease/README.md`. The bakes' own costs, from the
`BAKED_LIGHT ... took` lines: a bounce bake of 3,193-3,595 columns 3.6-9.8 ms
(29.6 ms once, under a rebuild), a lamp bake 3.4-6.2 ms, on a worker.

## 2. The lamp bake at a landing (measured: the largest)

A night walk through a row of lights in the desert near Tahal (`--time
22.5 --lamps 60 --walk-distance 300`). Sets land every 175-218 frames
(about 3 s at a sprint), each round a new anchor.

| Frame | What | Lit ground (lower half), luma |
| --- | --- | ---: |
| 372 | the old set and its lamp bake | 105.9 |
| 373 | the new set lands; neither half of the lamp bake holds its slots, so the flood is drawn | **130.8 (+24.9 in one frame)** |
| 375 | the new set's lamp bake lands (2 frames later), fading in over 0.5 s from the flood | 130.9 |
| 384 | the fade nearly done | 108.4 |

So every landing near a lamp at night jumps the lamp-lit ground by about a
quarter, for two frames, and walks it back over half a second: a flash. The
flood (no direction, no shadow) is much brighter than the bake beside a lamp.

## 3. The sun's bounce at a landing (measured), and its 2-degree steps (measured: none)

**At a landing.** The same walk by day (`--time 18.5`). On the landing frame
`bounce_gain` drops to zero (the bounce held is for the old anchor) and the
new bake lands two frames later, fading in over 0.5 s:

| Landing frame | Sand terrace walls in shade, luma | Red clay faces, luma |
| --- | ---: | ---: |
| 377 to 378 | 106.0 to 95.0 (**-11.0**) | 60.9 to 52.5 (**-8.4**) |

The whole frame: 19% of pixels darker by more than 6 levels in that frame
(`-2.7` mean). Each later landing of the walk darkens by 4.1 and 3.2 luma
over the lower half. So every landing by day dims every shaded face in one
frame and brings it back over half a second.

**The 2-degree re-bakes.** The sun's bounce is baked again every 2 degrees
(every 16 s at the clock's own pace, every 262 frames at `PBD_CLOCK_RUNS=4`)
and shown at once. On the shaded walls of a sand pit at 18:30 (the bounce's
best case: sand's albedo, a low sun) the re-bake frames 426 and 688 show no
step at all: the walls brighten 0.03-0.04 luma a frame on either side of
them as the sun moves, and the re-bake adds nothing measurable. On a meadow
at 19:00 the same. **Not a pop; left as it is** (decision 7.5).

## 4. Dusk and dawn (measured)

At Holbrook, facing a lantern post and the town (`--time 20.45`, the clock at
its own pace, lamps lit at hour 20.51):

| Frame | What | Luma |
| --- | --- | --- |
| 582 | `switch_dusk_lamps` relights the field (`SPENT 18.4 ms` on the main thread) | lantern glass and the town's windows light in this frame; the open grass does not move (-0.04): at the switch the sun is still 3 degrees up and `lamp_share` keeps a lamp's light off sunlit ground |
| 582 | 54-89 field-lit meshes' corners read the new field | corners move by up to 0.73 of a lamp level in one frame |
| 584 | the lamp bake for the relit field lands | the grass beside the post brightens +2.0 a frame for 30 frames (the bake's linear 0.5 s fade: eased, but fast) |
| 583-601 | the towns' outsides finish their sliced bakes, one mesh at a time | a sandstone wall +5.8 and a town wall +5.3 in one frame (591); in the alley view, a stone tower **+12.4** in one frame (481) |

So at dusk the town's walls light up in steps, one mesh at a time, 1 to 19
frames after the ground beside them. The flood's own step on open ground is
nil at the switch hour, but under an eave or in a cave, where `lamp_share`
is one, the flood steps from dark to lit in the relight frame (from code:
nothing under the old bake's reach covers a dusk-lit lantern, so the flood
is drawn there until the new bake lands).

## 5. Towns while walking at night (measured from the log)

A night walk from Holbrook toward Linenleigh (`--time 22.5 --yaw 180
--walk-distance 700`). Every landing whose tier reaches a standing town
re-bakes it and swaps each mesh in whole. The largest change of any vertex's
lamp level at a swap, per mesh: 0.60, 0.43, 0.54, 0.64, 0.60 (one landing),
0.81, 0.59, 0.67, 0.81, 0.82 (the next), and 0.13-0.17 when the tier left
the town and it went back to open sky. Those are changes between two fields
that both reach the town: a lamp near the edge of the old tier lights less,
because the flood treats what is off the tier as rock (section 6).

## 6. Found, and not this change's

**The flood is cut at the tier's edge.** `light::bake` treats a cell off the
tier as solid, so a lamp within its reach of the edge lights less in a set
that ends near it than in one that does not. Section 5's town changes are
this. In the terrain it is the lamp-lit ground near the tier's edge
brightening when the next set lands. The slots differ between the two sets,
so the shader cannot blend across it; the fix is in the bake (a margin ring
lit but not drawn) and belongs to the light field's own change. Recorded,
not fixed here.

**The dusk relight costs one long frame**: 15.3-18.4 ms on the main thread
(`SPENT ... dusk lamps: re-baking the field`), twice a day. A frame-budget
item.

**A lamp's own glass and flame** switch on with the record's state bit in the
relight frame, as a lamp placed does. The light it casts is what this change
eases.

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

The GPU's light buffer gets two halves. `switch_dusk_lamps` names the version
it made (`DuskRelight`); `upload_fine` writes that version's field into the
other half and starts the blend, and `light_at` mixes the two halves while it
runs. Any other upload (an edit, a landing) writes the newest half and ends a
blend in progress at once: the other half no longer matches the columns.
Memory: the light buffer goes from 7.9 to 15.8 MiB at capacity; the second
read happens only in the second a blend runs, twice a day.

### 7.4 Towns keep their last bake

A town's outside carries its bake in `UV_1` and, from this change, the bake
before it in the vertex colour (`COLOR`, sky and lamp in `xy`). The vertex
colour is given at spawn, so the mesh's layout never changes and no pipeline
is specialised mid-flight. `field_lit.wgsl` reads `COLOR` as light, never as
a tint, on a row marked as a town's outside (`fade.z`), and blends by the
row's `fade.y`, which `light_from_the_field` writes from the time of the
swap. A swap during a blend starts from what was on screen: the CPU mixes the
two by the blend's progress into `COLOR` before writing the new `UV_1`.
Memory: 16 bytes a vertex of a town's outside (2.5 MB for a walled town).
The bake itself is unchanged: 2 ms slices on the main thread.

### 7.5 The bounce's 2-degree re-bakes are left

Section 3 measured nothing at them, so the bounce keeps one buffer and its
re-bakes still show at once. Only its landing changes (7.2).

### 7.6 Field-lit corners ease

When a new field (a landing, a relight) moves the corners of a mesh that has
not moved, its row blends from the corners it showed to the new ones by the
same rule. A mesh that moves takes its new corners at once, as now: its
light follows it continuously.

### 7.7 The lamp bake's fade is the rule's

The lamp bake's cross-fade becomes `BAKE_EASE_S`'s smoothstep instead of
0.5 s linear: section 4's +2.0 luma a frame for 30 frames becomes at most
1.5 luma a frame over 60.

### 7.8 Tests

- The rule: zero at the swap, one at `BAKE_EASE_S` and after, rising, and
  no 60 Hz frame moves more than a twentieth of the step (`pbd_core::light`).
- A landing publishes its set's bounce and lamp bake with it: the bounce's
  gain is whole on the landing frame and the lamp bake shows at once
  (`planet_bounce`, `planet`).
- A dusk relight blends the light field's halves; an edit does not
  (`planet`).
- A town's swap keeps the bake before it, and a swap mid-blend starts from
  what was shown (`field_light`).
- Field-lit corners ease under a new field and snap when the mesh moves
  (`field_light`).
- The shaders carry the blends (shader-text tests beside the existing ones).
