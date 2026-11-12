# Lighting model, step 1 (L1 + L2 + L6): today against new

## Findings

- **The new shading costs about 0.6 ms.** Cloud flight p50: today 10.5 ms,
  new without bloom or the tone pass 11.1 ms. That covers the sun after the
  air and the sky's nine coefficients per fragment, in the terrain and in
  the field-lit meshes.
- **Bloom costs about 2.4 ms, and its size does not change that.** Bevy's
  `Bloom` measured the same at max mip 512 (14.2 ms), 256 (14.1) and 128
  (13.9), against 11.8 without it, in one sitting. So the cost is not the
  blur. A cheaper bloom of our own is task 3.3.
- **The tone pass and the CPU meter cost nothing measurable.** New without
  bloom (11.2) matches new without the tone pass (11.1).
- **The first build cost +6.6 ms, and that is fixed.** The sky's light was
  projected per call, and in flight the sun's elevation at the camera
  changes every frame. Its single-entry cache missed on two threads. It is
  now a 513-elevation table built once and interpolated: new went from
  17.1 to 14.1 ms.
- **Walk** (first sitting, before the table fix): p50 9.2 to 11.3 ms. p95
  and p99 were lower with new (14.2 and 20.7 against 17.3 and 27.7), inside
  the spread of two runs.

## Limits

- Apple M4 Max (Metal), window 1440 x 900, `--no-vsync`, release build,
  2 runs per variant, interleaved. Only the cloud passes have GPU timers on
  a Mac, so a pass's own GPU time is inferred from variants.
- Variants come from one build: `PBD_LIGHTING=today` draws today's model,
  `PBD_NO_TONE=1` drops bloom and the tone pass, and `PBD_NO_BLOOM=1` drops
  bloom.
- Sittings: `output/perf/2026-10-28-lighting-l126` (first build),
  `-split` (this page's tables), `-split2` and `-bloom` (bloom).
## clouds

| variant | valid runs | fps | p50 | p95 | p99 | p99.9 | max | GPU total p50 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| today | 2/2 | 82.61 | 10.52 | 21.03 **over** | 33.53 **over** | 37.24 | 40.23 | - | 6.57 / 7.69 |
| new | 2/2 | 63.25 | 14.05 | 26.88 **over** | 40.04 **over** | 47.02 | 58.84 | - | 7.12 / 8.36 |
| new-no-tone | 2/2 | 78.38 | 11.13 | 21.25 **over** | 33.98 **over** | 38.16 | 46.02 | - | 7.18 / 8.44 |

On the cloud leg only (the stretch the plan flies through cloud):

| variant | fps | p50 | p95 | p99 | max | GPU total p50 / p95 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| today | 100.27 | 9.97 | 12.27 **over** | 19.76 **over** | 28.27 | - / - | 6.67 / 7.18 |
| new | 74.03 | 13.26 | 21.31 **over** | 26.75 **over** | 50.32 | - / - | 7.19 / 7.63 |
| new-no-tone | 94.85 | 10.51 | 12.85 **over** | 20.42 **over** | 26.16 | - / - | 7.27 / 7.82 |

![clouds](clouds.png)

