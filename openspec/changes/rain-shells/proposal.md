# Proposal: rain-shells

## Why

Rain is the largest single cost near the ground on the Mac (`frame-budget`
2e and the near-ground pricing sitting, 2026-10-28, M4 Max, `clouds`, 2
repeats). Below 200 m, with landings left out:

| Variant | p50 ms | p95 ms | Frames over 16.7 ms |
| --- | --- | --- | --- |
| everything on | 17.3 | 33.7 | 60% |
| rain volume off | 13.8 | 25.4 | 28% |
| rain, lens and shower off | 11.7 | 21.1 | 17% |

The rain volume (`water.wgsl` `rain`) marches 16 steps per pixel at full
Retina resolution, with two `gnoise3` and a bilinear rain-map read per step.

The owner, 2026-10-28: "Rain fx were used in crab breeze. We could do
something with scrolling textures around a cylinder mesh. With the top and
bottom blended to match the env". Crab-breeze draws its distant rain as
shafts: crossed quads with falling streaks (`cloud.wesl` `shaft`). A cylinder
round the eye is the same trick turned inside out: the eye is in the shaft.

## What changes

- The rain volume is replaced by a few nested cylinders centred on the eye
  (shells). Each is textured with a baked, tiling streak texture that scrolls
  down at the fall speed. Nothing is marched.
- Each shell's top fades into the cloud base and takes the base's light. Its
  bottom fades softly where it meets the ground or the sea. It takes the air's
  haze with distance, as the clouds do.
- Each shell reads the rain map where it stands, so a storm 1 km away shows on
  the far shell only, in its own direction.
- Snow uses the same shells, with a flake texture and a slower, swaying fall.
- `rain_renderer: Shells | Volume`, with `PBD_RAIN_RENDERER` as an override,
  as `cloud_renderer` does, so the two can be compared in one sitting. The
  default turns to Shells only after the benchmark and the captures.
- The CPU shower and the shaft streaks stay in this change. They are priced
  again with the shells on; replacing them is a later step.

## What does not change

- Saved games and generators: this is drawing only.
- The rain field, the rain map and every rain number that drives the ground,
  the water and the lens.
