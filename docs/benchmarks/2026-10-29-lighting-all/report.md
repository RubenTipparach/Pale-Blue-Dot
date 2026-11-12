# lighting-model, every part: one pass of the cloud flight (2026-10-29)

An M4 Max Mac, release build, windowed, `--route scenic`, the clock at
10:00, one run of each variant in one sitting. The sitting was stopped after
the first pass, at the owner's word ("All is good with frame time. We'll do
more perf tuning"), so there are no repeats and no spread: these
differences are a first read, not results.

| Variant | Env | p50 ms | p95 ms | p99 ms | GPU clouds p50 ms |
| --- | --- | ---: | ---: | ---: | ---: |
| today | `PBD_LIGHTING=today` | 15.81 | 30.12 | 42.13 | 11.28 |
| new (all parts, no bloom: a Mac's default) | | 19.77 | 36.00 | 54.69 | 15.04 |
| new without L3 | `PBD_SOFT_SHADOWS=0` | 19.21 | 35.48 | 53.99 | 14.88 |
| new without L8 | `PBD_REFLECT=0` | 18.09 | 35.78 | 48.50 | 13.78 |
| new without L4 and L7 | `PBD_BOUNCE=0;PBD_LAMP_BAKE=0` | 19.22 | 35.04 | 53.31 | 14.91 |
| new, our bloom | `PBD_BLOOM=lite` | 21.35 | 38.39 | 57.73 | 14.86 |
| new, Bevy's bloom | `PBD_BLOOM=bevy` | 22.19 | 40.83 | 59.62 | 14.72 |

## Findings (first read)

- The whole sitting ran slower than 2026-10-28's: today's model was 15.8 ms
  at p50 here against 10.5 ms then. That is the machine, not this change;
  compare within the table only.
- All parts together: about +4 ms at p50 over today's on this flight. The
  owner has accepted the cost and will tune it ("Adding more frame time is
  not a problem").
- L8 (the walk and the screen trace, with its capture pass) is the largest
  single part, about 1.7 ms. L3 and L4 + L7 are about 0.5 ms each.
- Our bloom costs about 1.6 ms against Bevy's 2.4 here.
- The storm scenario (the wet film and the lit rain fog) was not reached.

The CSVs and logs are local (`output/perf/2026-10-29-lighting-all`).
