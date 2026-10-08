# Proposal: weather that changes smoothly

## Why

The owner, 2026-10-08: "the clouds and weather transitions dont seem smooth
please fix this".

The weather is a simulation stepped once a second (`atmosphere.ron`,
`dt_s: 1.0`). It is stepped on a copy, off the main thread, and published
whole (`crate::atmosphere::Air`). Everything that shows weather reads the
latest published state as it stands. Nothing shows the time between two
states. `atmosphere::tests::weather_steps`, an instrument added for this
change, measures on the shipped settled climate what that looks like.

### The clouds tick, and lurch when a step is held

The GPU's cloud map is rewritten whole at every publish. Stepping also waits
while the fine terrain set is rebuilt (`far-side-flight`), then catches up in
one go, up to 30 steps.

| One publish of | Cover moves by at most | 99th percentile | Texels moving more than 0.01 | more than 0.05 |
| --- | ---: | ---: | ---: | ---: |
| 1 step | 0.060-0.273 | 0.015 | 4.0% | 0.02-0.06% |
| 3 steps (a held step) | 0.204 | 0.049 | 13.2% | 0.9% |
| 30 steps (a long hold) | 0.688 | 0.395 | 31.1% | 18.0% |

At one step the change is small, but it lands in one frame, once a second, so
cloud edges tick rather than flow. In a flight that holds the weather while
terrain loads, a fifth of the sky can jump at once.

### Rain switches on and off, at full strength, every half minute

The rain a player sees (`pbd_core::weather::rain_at`) is the cover where the
simulation's rain rate is over `raining_rate`, and nothing where it is under:

```rust
if cell.raining { cell.cover } else { 0.0 }
```

The simulation's rain is convective. A column's rain rate leaps past the
threshold in a single step: the median time from half the threshold to the
threshold is 0 s. It rains in bursts. Over ten minutes at 3,000 places:

| | Measured |
| --- | ---: |
| Places that start or stop raining at least once | 1,692 (56%) |
| Starts and stops at each of those places | 8.4 in 10 minutes |
| What the rain jumps by when it starts or stops | median 1.00, 10th percentile 0.79 |
| How long a rain lasts | median 25 s, 10th percentile 7 s |
| How long it is dry between two rains | median 23 s, 10th percentile 6 s |

At a raining place, then, the streaks, the lens drops, the ripples, the rain
shafts and the rain volume all switch to full and back to nothing about every
25 seconds. That is the transition the owner sees.

The rain rate cannot simply be averaged instead. A downpour's rate starts at
up to seventy times the threshold, so a thirty-second mean of it still
crosses from dry to full in one step about once in a hundred.

## What

1. **The rain seen builds in and dies away.**
   - The atmosphere carries one more per-cell value: the rain as a person
     there sees it, `rain_seen`.
   - It follows the rain as it is shown now (the cover where it rains,
     nothing where it does not):
     - rising with a 5 s time constant;
     - falling with a 20 s one.
   - It is derived, never saved and never read by the physics. A restored or
     settled state starts it at the rain as it stands.
   - `rain_at`, `cloud_cell`, the precipitation map and the rain lattice all
     read it. So the player's rain, the shafts and the rain volume still agree
     everywhere: one number, as the weather spec requires.
   - Measured on the same 3,000 places:
     - it crosses on or off 2.3 times a raining place in ten minutes, against
       8.4 hard switches;
     - it never moves more than 0.18 in a second;
     - it shows 15% more rain on average (0.273 against 0.236), because a
       shower dies away rather than stopping.
2. **What is shown runs between two states.**
   - `Air` keeps the pair of published states it is showing and how far it
     is between them.
   - A state published while a blend runs waits for that blend to finish, so
     what is shown never jumps.
   - A blend lasts the weather time it covers, at most 4 s. A long hold
     therefore plays out over 4 s, never in one frame.
   - The player's cover and rain, the precipitation map and the rain shafts
     mix the two states on the CPU.
   - The cloud and wind maps are mixed on the GPU each frame by a small
     compute pass, so every shader that reads them sees the mix and none of
     them changes.
   - A clock that jumps (a world loaded, the time slider) snaps to the newest
     state, and so does a capture, whose picture must be a function of its
     flags.

Not changed:
- the atmosphere's physics and its saved state;
- the settled climates and the weather maps' format;
- the cloud shaders.

## Impact

- `pbd-core`:
  - `atmosphere` gains `rain_seen`, its update after each step, its reset on
    restore, and two settings (`rain_rise_s`, `rain_fall_s`), validated;
  - `weather` reads it;
  - tests pin the follower, the reset, and that the saved bytes are
    unchanged.
- `pbd-app`:
  - `atmosphere::Air` gains the shown pair and its blend;
  - `weather` mixes the pair;
  - `planet_weather` gains the blend textures and the compute pass
    (`weather_blend.wgsl`), with a GPU test of the pass on an adapter.
- Frame cost:
  - The compute pass touches 2 x 24,576 texels a frame.
  - The CPU does at most two extra point samples per rain-lattice cell, and
    4,096 lerps for the rain map.
  - Not measured in this cloud session (no GPU). The owner runs
    `perf_suite.py`.
