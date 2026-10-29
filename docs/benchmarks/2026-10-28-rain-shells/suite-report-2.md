# Performance suite, 2026-10-28 16:14

- revision: `db25fe2` (uncommitted changes)
- adapter: Apple M4 Max (Metal); window 1440 x 900, uncapped (`--no-vsync`)
- clock pinned at 10.0 h; first 10.0 s of each run thrown away; 2 run(s) per variant, interleaved
- budgets: p95 within 8.3 ms (120 fps), p99 within 16.7 ms (60 fps); **over** marks a miss. Each cell is the median over the valid runs.

## clouds

| variant | valid runs | fps | p50 | p95 | p99 | p99.9 | max | GPU total p50 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| volume | 2/2 | 78.51 | 11.08 | 25.50 **over** | 37.32 **over** | 42.40 | 48.32 | - | 6.66 / 7.89 |
| shells | 2/2 | 83.20 | 11.02 | 23.08 **over** | 30.76 **over** | 37.15 | 44.38 | - | 6.55 / 7.90 |
| rain-off | 2/2 | 84.50 | 10.95 | 21.88 **over** | 29.19 **over** | 34.69 | 39.48 | - | 6.60 / 8.07 |

On the cloud leg only (the stretch the plan flies through cloud):

| variant | fps | p50 | p95 | p99 | max | GPU total p50 / p95 | GPU clouds p50 / p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| volume | 96.10 | 10.21 | 18.08 **over** | 22.03 **over** | 30.09 | - / - | 6.69 / 7.45 |
| shells | 95.60 | 10.35 | 19.32 **over** | 22.59 **over** | 30.87 | - / - | 6.66 / 7.45 |
| rain-off | 94.89 | 10.33 | 18.45 **over** | 22.34 **over** | 29.91 | - / - | 6.75 / 7.62 |

![clouds](clouds.png)

