# Design: the performance rig

## Measurements

| Column | Source | Meaning |
| --- | --- | --- |
| `wall_ms` | `FrameStats` (existing) | frame-to-frame wall time on the main thread |
| `gpu_clouds_ms` | render-diagnostic spans `cloud_march` + `clouds` | GPU time of the two cloud passes |
| `gpu_total_ms` | every `render/*/elapsed_gpu` span in the store | the GPU time the render graph's instrumented passes took |
| `clearance_m`, `speed_mps` | `FlightReadout` (existing) | where the flight is |

GPU spans come from `RenderDiagnosticsPlugin`, which uses timestamp queries
(Vulkan and DX12; the reference machine is Vulkan on an RTX 3070). A span's
value arrives a few frames late, so a GPU column describes the frame a few
rows above it; statistics over a scenario are unaffected. The plugin is only
added when `--frame-log` is, so a normal run pays nothing for it.

## Scenarios

| Name | Command | What it stresses |
| --- | --- | --- |
| `clouds` | `--route scenic` | lift-off, a climb into the thickest reachable cloud, a leg inside it, then low over the land |
| `far-side` | `--route far-side` | ground to 3 km, the fine-set handover, and back down |
| `walk` | `--walk-distance 1000` | ground streaming on foot |

Every run: a fresh world (`--world perf-<scenario>`, deleted before and after),
the clock pinned by `--time` (daylight, so the scenic route has its clouds),
`--no-vsync`, the window at its default 1440 x 900, and the first 10 s thrown
away. A `clouds` run whose log has no `scenic route: into N m of cloud` line,
or whose route never completed, is marked invalid rather than reported.

## Comparisons

Variants are a build (`--exe`) and an environment (`--env PBD_NO_CLOUDS=1`),
and runs are interleaved A, B, A, B, because the same build measured on
another day, or after the machine warmed up, moves by more than most changes
do (godot-sandbox recorded 2.4 ms of such drift).

## Budgets

The owner's targets: 120 fps (8.3 ms) wanted, 60 fps (16.7 ms) the minimum.
The report marks a scenario's p95 against 8.3 ms and its p99 against 16.7 ms.

## A route through several clouds (`--route clouds`)

The owner, of the first fly-through recording: "make sure you fly through
several clouds too ... so I can report visually what I'm seeing on cloud
flythroughs". The scenic route flies the longest unbroken run of cover >= 0.55
at 40% of the layer's thickness: in the recording that was one 2.1 km
whiteout, one cloud entered once.

`--route clouds` is the scenic route with a different cloud score and height:

- **Score:** the longest run of BROKEN cover along the first 80% of the path:
  cover between 0.3 and 0.75, where the shader's shape noise (a wavelength of
  about 230 m, `clouds.wgsl` `cloud_density`) leaves separate puffs with clear
  air between them. Above 0.75 the puffs merge into a deck. The atmosphere's
  cover field is far coarser than a puff, so counting patches in it was tried
  first and found one patch on the best heading; the puffs exist only in the
  shader. Weighted as the scenic route weights its run.
- **Height:** from 700 m before the first patch to 400 m past the last, the
  ship holds 25% of the layer's thickness above the cloud base (scaled by the
  tops, as the scenic route does), where broken cloud is patchy, instead of
  the scenic route's 40% inside solid cover.
- **Log:** the plan line keeps the scenic route's shape (`into N m of cloud
  ... from M m`, the span from the first patch to the last) and adds how many
  patches it crosses, so the suite's cloud-leg window still reads it.

It is a new scenario, `cloud-hop`, in the suite, and the route the next
fly-through recording uses.
