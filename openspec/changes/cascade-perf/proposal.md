## Why

The owner, 2026-11-17, playing by a long shallow river cascade in a desert
canyon (map 23.2255S 3.012W, 79.6 m; also 20.885S 6.393W): "im getting
serious frame drops near these areas". The river there steps down one layer
at a time, many steps in a row, a fall's foot and its mist at every lip.

Every cut here keeps the picture and the sound (CLAUDE.md, "Performance
improvements need no permission"). Anything a player would see differently
is written up for the owner, not made.

## Where the frame goes at the cascade

Measured on the M4 Max (the owner's Mac), release build of `origin/main`
(`2de3c03a`) plus the instruments below, `--stand 40` at
`--at 23.2255 -3.012 --yaw 0 --pitch -10`, and the same spot turned round
(`--yaw 180`, dry desert, the control). The cascade on a fresh world is the
owner's: the HUD reads `map 23.226S 3.012W | 79.6 m elevation`
(`docs/benchmarks/2026-11-17-cascade-perf/`).

1. **The frame is the GPU's.** A CPU sample of the game standing at the
   cascade (`sample`, 12 s) found the render thread blocked in
   `-[MTLCommandQueue commandBuffer]` (waiting for the GPU to hand a command
   buffer back) for about 40 % of its samples, and the main world waiting on
   the render world. Neither thread is busy: the main world's update is
   4.7-6 ms at p50 (`cpu_main_ms`), the same at the cascade and the control.
2. **The water sim and its mesher cost nothing while standing.** The
   river is settled (`a_freshly_built_tier_s_rivers_are_what_the_rule_keeps`):
   no water tick ran in 40 s at the cascade, so no fine-set clone, no
   remesh, no fine-set upload. The mesher drew 1,720 cells, 575 tops, 4,478
   triangles in 1.5 ms, once, at the landing.
3. **The splash costs the CPU 0.07-0.1 ms a frame.** `animate_splash` at the
   cascade: 131 feet, all inside the 140 m reach, about 1,200 quads (the
   budget is 4,000), 64-97 us a frame (`SPLASH_STATS`).
4. **The water's fragment is the heavy shader on screen,** and the cascade
   is the worst case for how it was drawn:
   - The flowing water is drawn opaque with depth writes and, because the
     fragment can `discard` (the landing dissolve, the far river's dither,
     the scene in front, the back of a face), the GPU cannot reject a
     fragment by depth before running it, and Apple's hidden-surface
     removal is off for it. Every face of every step, front and back, that
     the rasteriser meets runs the whole fragment (waves: 4 fbm, near a foot
     12; the flow-map, ripples and churn loops over 8 feet; the screen trace
     of the reflection), then most lose to a nearer face drawn later.
   - The test against the scene's depth (`own_depth > in.clip.z`), the
     cheapest reason to draw nothing, ran after all of the waves had been
     worked out, so water behind a canyon wall or a step's lip paid in full.
   A cascade is many tops and fall faces stacked one behind the other along
   the line of sight, both sides of each face rasterised (nothing is culled:
   the back faces are the underwater view), in the canyon's walls.
5. **Spikes, standing.** At the cascade the slow frames line up with
   nothing the water does: no tick, no remesh.
6. **Spikes, walking along it** (`--walk-distance 250` from the cascade,
   which digs when it stalls on the bank, as a player does):
   - every fine-set version costs the render thread 8-11 ms in
     `render: fine-set upload`: the four levels' records written whole (3-4.5
     ms), the tier's light worked out (`gpu_light`, 2.2-3.3 ms, 387,828
     words) and its materials (2.8-3.3 ms), every time, for the whole tier
     (`UPLOAD_PARTS`). A landing every 3 s while walking makes one; so does
     every edit and every water tick;
   - **a water tick that moves nothing still made one.** Of 12 ticks after
     three digs by the river, 7 changed no cell (`water tick 3: 0 cells
     changed, 0 queued`), and each still cloned the set, raised its version,
     re-uploaded the whole tier (8-9 ms on the render thread) and remeshed
     the water (1.5-2 ms on the main thread): a 60 Hz miss a second for
     nothing on screen.

What could not be measured in this sitting: GPU time by pass. On a Mac
Bevy's pass spans read nothing (Metal times only at pass boundaries, and an
Apple GPU overlaps consecutive passes, so a boundary pair brackets its
neighbours' work too: tried for the water and the splash, it read the same
11-20 ms for both, at the cascade and the control, and was taken out). And
the wall time was paced by the machine, not the game: the screen was locked
for the whole sitting (macOS throttles a window it cannot show; `acquire_ms`
7-13 ms once Bevy's 60 Hz unfocused cap was lifted), and other sessions ran
up to three more game windows and cargo builds beside it. The perf suite
refuses both. The numbers that stand are the CPU's and the counts above.

## What changes

1. **The water's discards first.** `water_hidden` holds every discard of the
   water's fragment that does not hang on its shading: the landing
   dissolve, the band fades, the far river's dither, the scene in front and
   the back of a face seen from dry air. `fragment` runs it first, so a
   hidden fragment costs none of the waves. The later tests stay where they
   were and never fire. Pure reordering: the same pixels are drawn with the
   same values.
2. **A depth pre-pass for the flowing water.** `flowing_depth` draws the
   flowing water's depth alone, with `water_hidden`'s discards, then the
   shading pass draws it testing against that depth (`GreaterEqual`) and
   writing none. With no depth write in the shading pipeline the GPU tests
   depth before the fragment runs, so `fragment` runs once a pixel, for the
   nearest face, however many faces of the cascade stand behind it. The
   nearest face still wins, and of two at the same depth the last drawn, as
   before. The sea's cap is unchanged.
3. **A water tick that moves nothing changes nothing else.** It keeps the
   queue it worked through but not a new version: no upload, no remesh, and
   the drawing's ease (`WaterEase`) runs on, so a fall that landed last tick
   does not splash again. Measured on the walk: the 7 empty ticks made no
   upload and no remesh (20 water meshes to 13).
4. **Instruments** (CLAUDE.md, the measurement exception):
   - `--stand SECONDS`: hold the view where `--walk --at` put it, log
     `STAND_START`/`STAND_DONE`, quit;
   - a `--frame-log` run is not held to 60 Hz when its window loses focus
     (Bevy's unfocused pacing hid every frame under 16.7 ms);
   - `SPLASH_STATS` at debug level: feet, quads and `animate_splash`'s time;
   - `UPLOAD_PARTS` at debug level: the fine-set upload's parts;
   - two perf-suite scenarios, `cascade` (the held view) and `cascade-walk`
     (250 m along the river), so the cascade is measured from now on.

Verified identical: captures at the cascade (`--capture`, frame 300, two
views) from `origin/main` and from the branch are equal pixel for pixel
(0 of 5,184,000 differ). A further cut, skipping the still waves where the
flow-map's copies replace them wholly, changed up to 200,000 pixels by 1-141
levels (the compiler's arithmetic, not the terms) and was dropped.

## Next, the same picture (planned, not built)

- **The fine-set upload off the render thread.** The light and the
  materials (5-6 ms of the 8-11) are a function of the set: work them out
  where the set is built (the landing's task, the tick, the edit) and keep
  them on it, so the render thread only copies. Then write only the columns
  an edit or a tick changed. Every landing in the game pays this, not only
  the cascade's.

## Not in this change: for the owner (a look change)

- **Mist on one-layer steps.** 131 feet in reach at the cascade, each with 8
  or more puffs, spray and droplets. `claude/fall-size-rules` (water-flow
  6.7, the owner's request) already takes the mist off drops under 1 m and
  thins it on small ones. Its branch (`35a9622e`) at this view: 131 feet,
  quads 1,230 to 113 (-91 %), `animate_splash` 66-69 us to 25-31 us a
  frame. Its GPU share could not be timed here (above).
- **The splash's reach.** 140 m: every foot of the cascade in view splashes.
  A shorter reach for small falls would cut quads and overdraw and would
  show.
- **The back faces of the flowing water.** Drawn for the underwater view;
  culling them above water would change what a swimmer at the surface sees.
