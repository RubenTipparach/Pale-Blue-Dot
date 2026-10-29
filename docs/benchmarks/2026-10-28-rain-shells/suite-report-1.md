# Performance suite, 2026-10-28 15:39

- revision: `db25fe2` (uncommitted changes)
- adapter: Apple M4 Max (Metal); window 1440 x 900, uncapped (`--no-vsync`)
- clock pinned at 10.0 h; first 10.0 s of each run thrown away; 2 run(s) per variant, interleaved
- budgets: p95 within 8.3 ms (120 fps), p99 within 16.7 ms (60 fps); **over** marks a miss. Each cell is the median over the valid runs.

## clouds

| variant | valid runs | fps | p50 | p95 | p99 | p99.9 | max | GPU total p50 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| volume | 2/2 | 81.03 | 10.51 | 23.83 **over** | 35.61 **over** | 40.12 | 46.35 | - | 6.56 / 7.71 |
| shells | 2/2 | 85.54 | 10.70 | 21.85 **over** | 29.31 **over** | 34.59 | 39.53 | - | 6.47 / 7.67 |
| rain-off | 2/2 | 87.79 | 10.90 | 20.83 **over** | 26.14 **over** | 30.56 | 32.45 | - | 6.48 / 7.64 |

On the cloud leg only (the stretch the plan flies through cloud):

| variant | fps | p50 | p95 | p99 | max | GPU total p50 / p95 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| volume | 99.53 | 9.92 | 13.99 **over** | 20.35 **over** | 29.76 | - / - | 6.60 / 7.16 |
| shells | 98.33 | 10.05 | 17.18 **over** | 20.45 **over** | 24.76 | - / - | 6.57 / 7.14 |
| rain-off | 98.01 | 10.07 | 17.47 **over** | 21.12 **over** | 30.18 | - / - | 6.62 / 7.22 |

![clouds](clouds.png)

## walk

| variant | valid runs | fps | p50 | p95 | p99 | p99.9 | max | GPU total p50 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| volume | 2/2 | 96.40 | 9.20 | 17.12 **over** | 27.93 **over** | 32.71 | 40.92 | - | 2.15 / 4.07 |
| shells | 2/2 | 106.10 | 8.71 | 13.84 **over** | 22.77 **over** | 28.74 | 303.86 | - | 2.13 / 4.10 |
| rain-off | 2/2 | 118.81 | 8.27 | 12.29 **over** | 18.40 **over** | 21.89 | 32.92 | - | 2.10 / 2.93 |

![walk](walk.png)

