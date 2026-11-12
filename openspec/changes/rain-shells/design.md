# Design: rain-shells

## As built (2026-10-28)

Three departures from the plan below, each to keep the cost and the code down:

- **The cylinders are met analytically, not drawn as a mesh.** The eye is on
  every shell's axis, so a ray's hit is one division (`radius / sqrt(1 -
  c^2)`). That is exactly what a cylinder mesh would rasterise, inside the
  volume's own fullscreen pass: no new pipeline, no copy, no instance buffer.
- **The streak "texture" is a hash per column, not a baked image.** One streak
  per column per cycle, of hashed place, length and presence. It is as cheap
  as a texture tap and needs no new binding. 1,536 columns round every shell
  (about 0.004 rad each); cycles of 1, 4, 16, 64 and 256 m along the fall.
- **No sideways U shift.** A shift by the eye's tangential travel stretches the
  pattern on the inner shells. The lean already makes streaks slant and rush
  past in flight. The fall is accumulated on the CPU (`rain_shell_step`) and
  wrapped at 8,192 m, a multiple of every cycle, so a change of speed never
  jumps the streaks.
- **Every stretch of the ray is haze.** Each shell's slab is cut at the ground,
  the sea, the cloud base or the range. A shell behind the ground still adds
  its haze, read midway, so looking down from 150 m darkens as the volume
  does. Only the streaks fade at steep angles and at the ground.

## 1. Shells

- `RAIN_SHELLS` open cylinders, axis along the eye's up (body-local), radii
  6, 20, 70, 250, 700 and 1,500 m (as built; first planned as five, to 900 m:
  one reading for 575 m to 2 km missed a storm 1-2 km off, owner 2026-10-28).
  Each ring is about 3 times the one inside, so
  each shell stands for the slab of air between it and the one inside.
- One mesh: a ring of 48 segments, unit radius and unit height, drawn
  instanced (one instance a shell), as the puffs are.
- Height: from 50 m below the lowest surface in reach (the sea, or the
  ground under the eye) up to the cloud base (`CLOUD_RADIUS`). Under the base
  only, as the volume is.
- The axis leans into the wind the eye feels. The fall is `fall * down` minus
  the eye's velocity, so in flight the streaks rush past and slant. Without
  this the rain would travel with the eye.

## 2. The look

- A baked streak texture, R8, 256 x 512, made on the CPU at startup: thin
  vertical streaks of hashed length, width and brightness. U wraps the
  circumference a whole number of times per shell, so there is no seam. A
  test checks that it tiles.
- V scrolls by `clock * fall / tile_height`. Each shell has its own tile size
  (world-sized: fine near, broad far, so a streak is about a pixel wide at
  every shell) and its own phase.
- Sideways motion: U also shifts by the eye's travel along the shell's
  tangent, divided by the shell's circumference, so inner shells pass faster
  than outer ones. That is parallax.
- Opacity per shell is `1 - exp(-density * rain * slab * (0.3 + 1.4 streak))`.
  `rain` is `rain_map_at` at the fragment's world point, and `slab` is the gap
  to the next shell in. This is the volume's own sum, taken at six depths
  instead of sixteen, so the overall darkening matches it.
- Colour: the volume's tint and `rain_light` and its lightning term. With
  distance it mixes toward the air colour the clouds' composite uses.
- Blending into the surroundings (the owner's "top and bottom blended to match
  the env"):
  - Bottom: the scene's depth is read (`load_depth`). A fragment behind the
    ground or the sea is discarded. In front, it fades over `rain_soft_m`
    (default 4 m) to where it meets them, as the puffs' soft term does.
  - Top: it fades over the last 15% below the cloud base.
  - Looking steeply up or down, a cylinder's streaks crowd into lines. They
    fade by `|dot(view, up)|` past 0.75. The lens drops carry the
    looking-up case, as they do now.
- Snow: a flake texture, `snow_fall_mps`, and a sway in U by
  `sin(clock + phase)`.

## 3. Where it draws

The volume's slot in `WaterCompositeNode`: a `post_process_write`, the scene
copied with a fullscreen pass, then the shells blended "over" in one instanced
draw into the same destination. Disabled exactly where `rain_needed` is false
now. `PBD_PASS_OFF=rain` still prices it.

## 4. Cost (inferred, not measured)

- Volume, per pixel: 16 x (two `gnoise3` with 8 hashed corners each, plus a
  4-tap storage read).
- Shells, per covered pixel: at most 5 x (one texture tap, one 4-tap map
  read, one depth load). Fragments where the rain is zero are discarded
  first.

Expected: most of the volume's 3.5 ms near the ground (17.3 to 13.8 ms at p50
with it off). The benchmark settles it.

## 5. Measure

- `perf_suite` on `clouds` and `walk`: `PBD_RAIN_RENDERER=volume`, `=shells`
  and `PBD_PASS_OFF=rain`, interleaved, 2 repeats, unlocked, with
  `keep_awake.sh`. Report the near-ground band table.
- Captures, volume above and shells below: 2 m on the ground in rain, 150 m
  in rain, a storm seen from 2 km, snow, and looking up.

## 6. Distant rain: curtains under the storms

**The gap.** The rain is drawn out to about 1.6 km only, with either renderer:
the rain map is 64 cells of 50 m centred on the eye, and the volume's range is
2 km. A storm 5 km off is a dark cloud with nothing falling from it. The CPU
shafts reach only 150 m (`rain_detail_range_m`).

**The fix: Crab-breeze's rain shafts, laid out like the puffs.**
- Every puff site whose shown precipitation (the maps' third lane, which the
  puffs already read on the CPU) is over a threshold carries a curtain: one
  upright billboard turned to the eye, from the cloud base to the ground under
  it, as wide as the puff.
- Look: the same column streak pattern as the shells, scrolled at the fall
  speed. Densest under the base and thinning toward the ground. Darker under a
  dark storm. Its top tucks into the puff's flat base.
- Hand-over: curtains fade in from 1.2 to 2 km, where the shells fade out, and
  fade into the air's haze with distance (`cloud_air`), out to the horizon.
- Drawn in the puffs' pass and blended into the same history, sorted with
  them, so a curtain is behind a nearer puff and in front of a farther one.
  The base is the ground radius from the planet's coarse height, read once per
  site.
- Cost: one billboard per raining site in view, a few hundred at most, in the
  instanced draw that exists.
- Snow: the same curtain, paler and softer, with no streaks.

**Measure:** the `storm` scenario and a capture of a storm from 5 km, before
and after.
## Open questions

| Id | Question | Recommendation | State |
| --- | --- | --- | --- |
| R1 | Default renderer | Shells, if they are under budget and the captures hold | owner's, with the captures |
| R2 | Replace the CPU shower and shafts with the shells | Later, after pricing them with the shells on | recommendation taken (ask only with screenshots) |
