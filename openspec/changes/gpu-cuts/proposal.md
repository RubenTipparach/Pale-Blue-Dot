## Why

The owner, 2026-11-17: "with the shadow cascade fix, what else can we save
on gpu?", then "ok proceed with those things". Every cut here keeps the
picture and the sound (CLAUDE.md, "Performance improvements need no
permission"); a cut a player would see is not in this change.

The findings it answers (`docs/benchmarks/2026-11-17-play-budget/`,
`SHADOW_SPANS`, `PBD_TERRAIN_OFF`):

- The shadow pass is bound by vertices. The nearest cascade, a box a few
  metres across, ran about 4 million vertex invocations a draw.
- The column tier's cull bound is a sphere round each column's cap as wide as
  the whole span to bedrock (`COLUMN_BASE_M`, -145 m): 145 m or more for
  every column. Every column of the tier (about 3,900, 936 vertices each,
  about 3.65 M vertices) passed every cascade's test and the camera's,
  whatever they looked at.
- The shadow pass runs the terrain's full vertex shader, light reads included
  (`light_at`, `corner_light`, `wall_light`: the column tier's baked sky and
  lamp field), which a depth map never uses.
- The terrain's lamp lookups (`lamp_near`, two bakes) run on every face,
  though a lamp's light is multiplied by `lamp_share`, nought in the open by
  day.

## What changes

1. **The column is culled as a segment.** From the span's floor to its cap,
   thickened by the cap's own bound (`in_frustum_segment`), in place of the
   sphere. Exact for that shape: no face the sphere kept in view is dropped.
   It applies to the camera's view and to every cascade.
2. **A shadow-caster vertex shader.** The cascades' pipeline compiles the
   terrain shader with `SHADOW_CASTER`, which makes the three light reads
   constants. Positions, cut-outs and fades are untouched, so the depth map is
   the same.
3. **Lamp lookups only where a lamp can show.** Skipped where
   `lamp_share(skylight, daylight)` is nought, where they added nothing.

Measured afterwards on the trunk replay's bench step and the pasture view,
the old build against the new in one sitting.

## Looked at and left

- A cheaper far-ground path and the overdraw check wait for the bench
  replay's fragment counts.

## Also in this change

4. **Far town lights past the horizon are left out** (`night_lights.rs`
   `over_the_horizon`). A town on the planet's night side counts as lit while
   it is day where the player stands, so every ground pixel looped over towns
   no pixel could see: 2.1 ms of the terrain pass at the bench step.
5. **The shadow reach is 200 m at LOW, MEDIUM and HIGH**, on foot and in
   flight (owner, 2026-11-17: "max distance should be 200m? we dont really
   care that much all the way out there", "we already have planet scale
   lighting"). ULTRA keeps the reach it had. `PBD_SHADOW_REACH` overrides it,
   a pricing instrument.
6. **Benchmarks run without vsync** (owner: "to avoid confusion vsync just
   shouldnt run when we benchmark"): a `--frame-log` run presents without
   it unless `--vsync` is given, which `tools/perf_suite.py --vsync` passes.
7. **Exclusive clocks in the frame log** (owner: "these should be exclusive
   times", "we shouldnt measure cpu time when gpu is doing work"):
   `main_between_ms` (the main thread between updates), `submit_ms` and
   `present_ms` (Bevy's own spans, timed by a tracing layer). Each thread's
   rows add up to the frame: main = logic + between; render = its own work +
   acquire + present + submit + idle.

## Measured

The desktop, `fast` build, the trunk replay (`--showcase phase-1-trunk`), LOW.

Cuts 1-4, old build against new, two clean pairs, vsync on:

| | Old | New |
| --- | ---: | ---: |
| GPU, whole replay | 11.29 / 10.70 | 9.45 / 8.97 |
| GPU, bench step | 14.18 / 14.16 | 11.97 / 11.71 |
| Terrain vertex invocations | 6.4 M | 3.9 M |
| Nearest cascade vertex invocations | 4.6 M | 2.0 M |

A third pair was thrown out: its window lost focus.

After the cuts, vsync off, two replays each (exclusive, ms):

| | LOW 200 m | LOW 900 m | ULTRA |
| --- | ---: | ---: | ---: |
| Frame, whole replay | 14.62 | 14.80 | 18.86 |
| Game logic | 13.10 | 13.25 | 14.35 |
| GPU busy | 8.85 | 9.34 | 15.81 |
| Frame, bench step | 12.98 | 13.50 | 22.89 |
| GPU busy, bench step | 11.48 | 11.87 | 21.72 |

At LOW the game logic is now the frame; the GPU is idle about 5.8 ms of it.
The page: https://claude.ai/artifact/JSfGSKZQLv672HrCAWwpnC
(`docs/benchmarks/2026-11-17-play-budget/terrain-breakdown.html`).
