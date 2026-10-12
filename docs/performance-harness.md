# Repeatable desktop performance captures

See [recorded performance results](performance.md) for the measured native
baseline, final scene timings, memory and visual comparisons.

Build once with `cargo build --release --locked -p pbd-app`, close existing game
windows, then run from the repository root in Windows PowerShell 5.1 or later:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File tools/benchmark.ps1
```

The harness runs walking spawn, surface, orbit, inclined tour, and polar tour sequentially,
three rounds each. Static views use 1,200 frames; tours use 3,600 frames and must
report a completed physical circuit. Each child has a 60-second wall timeout.
The harness focuses only its own game window and restores the previous window
afterward. Keep the game visible and avoid using other windows while it runs;
failure to acquire focus or losing it makes a run invalid. Existing game
processes cause an error and are never closed. On timeout, only the child
started by the harness is terminated.

```powershell
./tools/benchmark.ps1 -Scenes surface,polar -Rounds 1
./tools/benchmark.ps1 -StaticFrames 2400 -TourFrames 7200 -TimeoutSeconds 120
./tools/benchmark.ps1 -Executable ./output/captures/baseline/pbd-app.exe -OutputBase ./output/captures/baseline-results
```

Each invocation creates a timestamped, unique directory under
`output/captures/performance/`, retaining screenshots, stdout/stderr, `results.json`,
and `summary.csv`. JSON records executable and current shader SHA-256 hashes,
hardware, scene settings, timing methodology, and memory methodology. CSV
contains one row per run. Preserve a baseline executable **and its runtime
assets** before changing code or shaders: the executable uses the asset path
embedded when it was compiled, so copying the executable alone does not freeze
shader or texture contents.

The reported p50/p95/p99 values are application wall-frame intervals, with the
first 60 frames and screenshot frames excluded. Capture mode requests no VSync
instead of interactive VSync. These are not GPU timestamps or a
guarantee of performance on other hardware. Windows composition, driver queues,
focus changes, and other GPU work can affect the results. The default scene is
1440 x 900, seed `0x5eed2026`; confirm actual screenshot dimensions when Windows
display scaling changes. The default 16.67 ms p95 comparison is a 60 Hz frame
budget; exceeding it produces a warning, while invalid captures, log errors,
incomplete tours, lost focus, or a nonzero process exit fail the harness.

Process working set and private memory are sampled about every 100 ms across
startup and rendering. The OS peak working set is recorded separately. These
figures include CPU-side application/driver allocations and do not measure GPU
VRAM. The current terrain renderer's 80 MiB topology allocation, two 2.5 MiB
ID lists per view (terrain and foliage), and 112-byte steady-state uniform update are
implementation sizes; this harness does not measure GPU allocation totals or
transfer counters. Compare all rounds and tail latency rather than choosing
the fastest run.

For the body-frame rendering regression, static captures accept
`--render-offset x y z` in metres. For example, compare otherwise identical
`--capture origin.png --view surface --frames 240 --fixed-dt` and
`--capture offset.png --view surface --frames 240 --fixed-dt --render-offset 16384 -8192 32768`
runs. The instrument translates the local frame, camera and backdrop together;
the authoritative planet stays at the same system coordinates. Compare the
scene region between HUD bars (rows 94 through 751 at 1440 by 900), allowing
small rasterization differences from local f32 coordinates. This option is
restricted to static captures and is not a player movement control.
