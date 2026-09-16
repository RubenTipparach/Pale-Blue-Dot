# Desktop performance validation

This is the flight-only performance baseline, before the walking and direct
camera revision. See [walking-validation.md](walking-validation.md) for the
updated executable's controls, captures, and measurements.

Measured on 2026-09-16: the current planet-flight prototype has substantial
headroom against a **16.67 ms / 60 FPS frame budget** on the tested machine.
All twelve final foreground runs passed, including three inclined and three
polar circumnavigations. The largest per-run p95 was **2.62 ms** and p99 was
**3.12 ms**. These are uncapped application wall-frame intervals, not GPU
timestamps or a guarantee for other hardware or the future full voxel game.

## Conditions and results

- Windows 10 Home, i7-9700F, 32 GB RAM, RTX 3070, NVIDIA 595.97, Vulkan.
- Release executable, Bevy 0.18.1 / Avian 0.6.1; 1440 x 900, HDR, 4x MSAA.
- Seed `0x5eed2026`; 4 km radius, 655,362 columns, existing terrain, trees,
  ocean, atmosphere and clouds. Visual quality settings stayed unchanged.
- One game process at a time. The harness checked foreground ownership and
  recorded executable/shader hashes. It did not run builds during timed runs.
- Three rounds per scene. Static scenes rendered 1,200 frames and tours 3,600.
  Each run excluded its first 60 frames and the screenshot request/readback.
  The final series contains 28,068 measured wall-frame intervals.
- These historical captures requested no VSync and disabled the then-existing
  interactive frame pacer. The later walking revision removes that extra pacer;
  normal play uses VSync with a requested maximum frame latency of one.

The median column is the median of three **per-run medians**; the p95 range
and maximum p99 retain the variation between runs. These are not percentiles
pooled from rounded per-run summaries.

| Scene | Median frame (ms) | p95 range (ms) | Largest p99 (ms) |
| --- | ---: | ---: | ---: |
| Low-altitude surface | 1.66 | 2.06-2.10 | 2.41 |
| Whole planet from orbit | 1.93 | 2.42-2.62 | 3.12 |
| Inclined physical circuit | 1.67 | 2.08-2.24 | 2.60 |
| Polar physical circuit | 1.66 | 2.08-2.12 | 2.40 |

The maximum OS-reported peak process working set was **591.75 MiB**; maximum
sampled private memory was **896.99 MiB**. Those are separate CPU process
counters, including startup and driver allocations; neither measures VRAM.
The terrain's fixed GPU topology remains about 80 MiB, with about 2.5 MiB of
visible IDs per view and a 112-byte per-view uniform update each frame. Those
are implementation sizes, not measured total GPU memory or transfer counters.

The earlier polar p95 of 16.77 ms in [validation.md](validation.md) did not
recur in the controlled baseline or final series. The historical cause was
not established; this result does not label that spike as a proven code bug.

A further 36,000-frame polar run passed with p50/p95/p99 of
**1.66 / 2.10 / 2.36 ms** over 35,939 measured intervals. Its peak working set
was 590.63 MiB and sampled peak private memory was 782.94 MiB, comparable to
the shorter runs. This exercised approximately ten minutes of fixed-step
simulation in 62.76 seconds of wall time; it is not a ten-minute real-time
endurance test. The tour controller continues flying after the first circuit,
while the completion report intentionally retains its first-lap statistics.

## Changes and comparison

The live visibility shader now evaluates the same horizon angle with
angle-addition arithmetic, removing two per-cell `acos` calls and one `cos`.
A small downward tolerance keeps boundary rounding conservative. Its GPU
bindings, persistent geometry, image resolution and material quality are
unchanged. The measured frame-time difference from the baseline is small and
within run variation; **no substantial FPS speedup is claimed**.

| Scene | Baseline median (ms) | Final median (ms) |
| --- | ---: | ---: |
| Surface | 1.65 | 1.66 |
| Orbit | 1.95 | 1.93 |
| Inclined circuit | 1.66 | 1.67 |
| Polar circuit | 1.67 | 1.66 |

A separate conservative frustum-culling trial increased the orbital median
from 1.95 to 2.05 ms and was removed. Its records are retained for comparison;
the shipped renderer continues to use horizon compaction. Future frustum/LOD
work should use bounds prepared with the terrain, rather than rescanning all
corner rays every frame, and must be measured again.

Naga and the native GPU pipeline accepted the final shader. A float32 numerical
probe covered 1,110,916 input pairs, including boundary values, with no modeled
false rejection.
All twelve final world captures were
**pixel-identical to their corresponding baseline images** outside the HUD:
947,520 world pixels compared per image. The HUD was excluded because its FPS
text changes between runs. Both physical circuits completed with zero emergency
terrain corrections. All 35 workspace tests and three validator tests passed.

## Reproduce and inspect

Use [tools/benchmark.ps1](../tools/benchmark.ps1); see its
[usage and measurement limits](performance-harness.md). It preserves every
round's JSON, CSV, logs and native screenshot in a unique output directory.
It fails invalid captures, GPU/log errors, incomplete tours and lost focus.
Exceeding the frame budget produces an explicit warning.

The harness writes `results.json`, `summary.csv`, `image-comparison.json`, the
per-round stdout/stderr and the visibility shader snapshot into its own run
directory under `output/captures/performance/<run-id>/`. That output is not
tracked: a run is one machine's driver banner and belongs where it was
produced. The numbers quoted above are the record; rerun the harness on the
hardware you care about rather than reading someone else's log.
Original local PNG paths remain in the JSON; the native
captures remain under the ignored `output/captures` directories.

GPU pass timestamps, dedicated VRAM residency, browser performance, additional
adapters and a production world with editable chunks remain unmeasured here.
