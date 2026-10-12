---
name: perf-measure
description: Measure Pale Blue Dot's frame time the repository's way and report it - fixed scripted scenarios, old build against new in one sitting, GPU time per pass, a checked-in report and a published HTML page. Use for any performance question or change that can move frame time ("is it faster", "did this regress", "what do the clouds cost", "measure the fps", "benchmark", "performance report", "why is it laggy").
metadata:
  author: Pale Blue Dot (Claude Code)
  version: "1.0"
---

# Measure performance

The rig is `tools/perf_suite.py` (the `perf-rig` change). The rules below are
in `CLAUDE.md` because each one was learned by getting a number wrong.

## The rules

1. **Measure the release build** (`cargo build --release -p pbd-app`, or
   `run.bat --full`). The `fast` profile (no LTO) is for iterating, not numbers.
2. **Nothing else runs while it samples.** No build, no second game, no
   recording. A release build compiling beside a walk made it read 37 ms at
   p50 against 4.6 ms alone. The suite refuses to start while `cargo`,
   `rustc` or another `pbd-app` runs, and warns about OBS.
3. **Compare old against new in the same sitting, interleaved** (A, B, B, A),
   never against a number from another day. Keep the old exe
   (`output/perf/baseline-exe/pbd-app.exe` is the 2026-09-25 baseline; copy a
   new one there when a baseline is adopted).
4. **An old exe needs its own assets.** The game reads `assets/` (shaders,
   `assets/config/*.ron`) from the checkout at run time, so an old exe would
   run the new shaders, or refuse a config field it does not know. Give the
   old variant the git revision its assets come from; the suite lays them over
   the checkout for its runs only and restores them after each run.
5. **Real time, windowed.** `--capture` steps the simulation in fixed steps
   and is for pictures, never for frame time.
6. **A run that did not do the hard part is invalid, not measured.** The
   suite marks a cloud run that found no cloud, a route that never landed or a
   walk that never finished, and leaves it out.
7. **Say what was measured**: machine, window size, build, repeats, and the
   spread between repeats. A difference smaller than that spread is not a
   result.

## Run it

```bash
# The current build on every scenario, clouds on and off:
python tools/perf_suite.py --repeats 2 --out output/perf/<date>-<name> \
  --variant "clouds-on" --variant "clouds-off||PBD_NO_CLOUDS=1"

# Old against new (a variant is name|exe|ENV=1;ENV2=2|git-rev-of-assets):
python tools/perf_suite.py --scenarios clouds,storm,walk --repeats 2 \
  --out output/perf/<date>-<name> \
  --variant "before|output/perf/baseline-exe/pbd-app.exe||<rev the baseline was built from>" \
  --variant "after"
```

Scenarios, each from a fresh world with the clock pinned at 10:00 and the
first 10 s thrown away:

| Scenario | Command | Stresses |
| --- | --- | --- |
| `clouds` | `--route scenic` | a climb into the thickest cloud in reach, then low over land |
| `storm` | `--route scenic --rain 1` | the same through a forced storm's deck |
| `cloud-hop` | `--route clouds` | low through broken cloud (not in builds before 2026-09-25) |
| `far-side` | `--route far-side` | ground to 3 km and down on the far side |
| `walk` | `--walk-distance 1000` | ground streaming on foot |

Each run writes `<scenario>_<variant>_<n>.{csv,log,json}`; the sitting writes
`suite.json`, `report.md` and a PNG per scenario. A run takes about a minute
(the walk about three); 2 repeats x 2 variants x 5 scenarios is about 25 min.

## Read it

- **Budgets** (the owner's): p95 within 8.3 ms (120 fps), p99 within 16.7 ms
  (60 fps). The report marks misses **over**.
- **Wall against GPU**: `gpu_total_ms` well under `wall_ms` means the frame
  is CPU-bound; clouds are `gpu_clouds_ms` (the `cloud_march` and `clouds`
  passes).
- **Spikes**: single frames of 40-100 ms show in `max` and `p99.9`; find what
  caused one with `python tools/frame_graph.py <csv> --log <log>`, which lines
  each slow frame up with the game's log (fine-set landings, `SPENT` lines).

## Find what costs it

- `PBD_NO_CLOUDS=1` skips the cloud passes; `PBD_PASS_OFF=rain,clouds,overlay,lens`
  skips any of the composite's passes. As variants, they price a pass on the
  same flight.
- The frame log alone: `pbd-app.exe --route scenic --no-vsync --frame-log out.csv`
  (the route quits when it lands). Columns: `wall_ms`, `gpu_clouds_ms`,
  `gpu_total_ms`, `route_m`, `clearance_m`, `rebuild_s`, `fine_version`.
- In game, `F3` shows the frame graph (`--frame-graph` starts with it shown).

## Report it

1. Write the findings on top of the suite's `report.md` (`## Findings`,
   `## Limits`, then the tables), in `docs/benchmarks/<date>-<name>/`, with
   `suite.json` (drop the `csv`/`log` paths) and the PNGs.
2. Make the page: `python tools/perf_report_html.py output/perf/<run>
   --notes docs/benchmarks/<date>-<name>/report.md --out
   docs/benchmarks/<date>-<name>/report.html --title "<name>" --eyebrow
   "Performance suite · A/B" --headline after`.
3. Publish it as an artifact, or republish the existing page's URL when it is
   the same report; link it from `CLAUDE.md`'s performance section.
4. Check it in with the change that moved the numbers.

## Gotchas

- `output/` is git-ignored: the CSVs and logs stay local; the report carries
  the numbers.
- The suite's windows must not be covered or minimised; a hidden window can
  be paced by the compositor.
- The first landing after startup and some fast-flight landings log
  `LOD_FADE skipped`; that is expected, not a fault.
