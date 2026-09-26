# Tasks

- [x] 1. GPU spans on the water composite node's passes; `RenderDiagnosticsPlugin` with `--frame-log`
- [x] 2. Frame log columns `gpu_clouds_ms`, `gpu_total_ms`
- [x] 3. `--route` with `--frame-log` quits at `ROUTE_COMPLETE`
- [x] 4. `PBD_NO_CLOUDS=1` skips the cloud passes
- [x] 5. `tools/perf_suite.py`: scenarios, interleaved variants, JSON per run, Markdown and PNG report
- [x] 6. First report: the three scenarios, clouds on and off, on the reference machine
- [x] 7. `--route clouds`: the scenic route through several separate clouds; suite scenario `cloud-hop`; record it

First report: `docs/benchmarks/2026-09-25-baseline/report.md`. Its exe is kept
at `output/perf/baseline-exe/pbd-app.exe` (not committed) for A/B runs.
