# Where the frame goes: one part off at a time (2026-10-29)

The owner, of the flights' 19 ms: "Then what is the major cost". Apple M4 Max
(Metal), release build at `6f85efe` (the puff cull and the 20 m wet cut in),
window 1440 x 900, uncapped, clock at 10:00, first 10 s thrown away, 2 runs
of each variant interleaved, GPU apps closed. All 30 runs were valid.

## Findings

- **What each part saves at p50, switched off** (clouds flight / rain stroll / 1 km walk; full frame 17.16 / 19.25 / 12.83 ms): rain passes (`PBD_PASS_OFF=rain,overlay,lens,shower`) 0.10 / **6.54** / 1.03 ms; the new lighting (`PBD_LIGHTING=today`) 2.16 / 5.41 / 1.69 ms; reflections, L8 (`PBD_REFLECT=0`) 1.42 / 3.85 / 0.70 ms; clouds (`PBD_NO_CLOUDS=1`) 1.98 / 0.35 / 0.00 ms. Shares overlap and do not add up to the frame.
Each variant switches one part off; what it saves at p50 is that part's
share. Shares overlap (rain lights the wet film the reflections trace), so
they do not add up to the frame.

| Variant | Switch | clouds flight | rain stroll | 1 km walk |
| --- | --- | ---: | ---: | ---: |
| full | | **17.16** | **19.25** | **12.83** |
| rain passes off | `PBD_PASS_OFF=rain,overlay,lens,shower` | -0.10 | **-6.54** | -1.03 |
| today's lighting | `PBD_LIGHTING=today` | -2.16 | -5.41 | -1.69 |
| reflections off (L8) | `PBD_REFLECT=0` | -1.42 | -3.85 | -0.70 |
| clouds off | `PBD_NO_CLOUDS=1` | -1.98 | -0.35 | +0.03 |

Repeats agree within 0.1 ms on the flight and 0.5 ms on the ground, except
the walk's no-reflect pair (12.91 / 11.35).

- **On the ground in rain, the screen rain passes are the biggest single
  cost: 6.5 ms** of the stroll's 19.3 (rain, overlay, lens and shower
  together). Which of the four is not split yet.
- **Next is the new lighting, 5.4 ms on the stroll, and 3.9 ms of it is the
  reflections**: a lane in rain is wet ground near the eye, and each wet pixel
  walks the columns and the screen for its mirror. In flight it is 2.2 ms
  (reflections 1.4).
- **The clouds cost 2.0 ms in flight** and almost nothing on the ground.
- **In flight about 13 ms is none of these**: with the clouds or the new
  lighting off the flight is still 15 ms, and the rain passes barely show
  there at p50 (their p95 drops 5.6 ms). That floor is the terrain, the sky
  and atmosphere, the water, the near fog, the tone pass and the CPU side,
  and it is not broken down yet.

## Next

- Split the rain passes one by one (`PBD_PASS_OFF=rain`, `lens`, `overlay`,
  `shower`) on the rain stroll.
- Price the flight's floor: a Metal System Trace in Instruments of a few
  seconds of the clouds flight gives per-pass GPU time directly, and shows
  whether the frame waits on the GPU or the CPU.
- Reflections: a reach by distance (like the 20 m wet cut) for the voxel walk
  and the screen trace, measured on the rain stroll.

## Limits

- One machine, one sitting; shares overlap and are not additive.
- "GPU clouds" on this Mac spans the GPU work up to the composite and is
  not shown here.
- The CSVs and logs are local (`output/perf/2026-10-29-breakdown`).

## clouds

| variant | valid runs | fps | p50 | p95 | p99 | p99.9 | max | GPU total p50 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| full | 2/2 | 52.43 | 17.19 | 32.86 **over** | 50.39 **over** | 57.11 | 61.61 | - | 13.02 / 15.43 |
| no-clouds | 2/2 | 57.76 | 15.21 | 29.86 **over** | 48.75 **over** | 55.45 | 60.06 | - | - / - |
| no-rain-passes | 2/2 | 57.36 | 17.12 | 27.01 **over** | 37.43 **over** | 43.18 | 48.34 | - | 13.07 / 15.53 |
| today-lighting | 2/2 | 61.21 | 15.01 | 28.45 **over** | 41.05 **over** | 48.03 | 51.64 | - | 10.73 / 12.89 |
| no-reflect | 2/2 | 57.83 | 15.79 | 30.00 **over** | 44.31 **over** | 52.65 | 59.93 | - | 11.46 / 13.70 |

On the cloud leg only (the stretch the plan flies through cloud):

| variant | fps | p50 | p95 | p99 | max | GPU total p50 / p95 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| full | 61.77 | 16.34 | 20.24 **over** | 32.66 **over** | 37.35 | - / - | 13.00 / 13.97 |
| no-clouds | 70.15 | 14.42 | 17.18 **over** | 29.19 **over** | 30.93 | - / - | - / - |
| no-rain-passes | 63.39 | 15.90 | 22.27 **over** | 32.71 **over** | 35.47 | - / - | 13.05 / 14.04 |
| today-lighting | 71.15 | 14.19 | 21.95 **over** | 28.29 **over** | 31.08 | - / - | 10.90 / 11.84 |
| no-reflect | 67.57 | 14.83 | 22.31 **over** | 29.67 **over** | 32.24 | - / - | 11.61 / 12.59 |


## rain-stroll

| variant | valid runs | fps | p50 | p95 | p99 | p99.9 | max | GPU total p50 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| full | 2/2 | 46.78 | 19.51 | 44.71 **over** | 55.12 **over** | 57.56 | 58.43 | - | 7.01 / 16.96 |
| no-clouds | 2/2 | 47.09 | 19.02 | 41.10 **over** | 54.96 **over** | 55.93 | 60.18 | - | - / - |
| no-rain-passes | 2/2 | 65.04 | 12.81 | 24.40 **over** | 43.86 **over** | 45.17 | 46.41 | - | 6.35 / 16.92 |
| today-lighting | 2/2 | 70.83 | 13.87 | 26.03 **over** | 29.62 **over** | 31.27 | 32.42 | - | 1.76 / 3.99 |
| no-reflect | 2/2 | 63.08 | 15.44 | 29.97 **over** | 35.32 **over** | 38.02 | 39.41 | - | 2.90 / 6.54 |


## walk

| variant | valid runs | fps | p50 | p95 | p99 | p99.9 | max | GPU total p50 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| full | 2/2 | 75.96 | 12.91 | 17.41 **over** | 27.10 **over** | 32.17 | 33.93 | - | 5.88 / 7.35 |
| no-clouds | 2/2 | 75.43 | 12.90 | 17.45 **over** | 25.56 **over** | 33.73 | 38.50 | - | - / - |
| no-rain-passes | 2/2 | 82.71 | 11.94 | 14.93 **over** | 20.97 **over** | 27.44 | 29.82 | - | 5.63 / 7.19 |
| today-lighting | 2/2 | 87.96 | 11.15 | 15.20 **over** | 22.03 **over** | 28.15 | 35.75 | - | 3.70 / 5.71 |
| no-reflect | 2/2 | 76.59 | 12.91 | 16.00 **over** | 23.62 **over** | 29.42 | 32.91 | - | 5.47 / 7.20 |
