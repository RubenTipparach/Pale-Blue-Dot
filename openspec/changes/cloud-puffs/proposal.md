# Proposal: cloud-puffs

## Why

Owner, 2026-10-28: "we have some optimizations and new clouds in the crab-breeze
repo I want you to port over. It seems raymarched clouds are too expensive so I
want you to grab a cheaper alternative and start bench marking with that".

The clouds pass is the dearest thing in the frame. On the owner's RTX 3070 at
1440 x 900 it is 4.1 to 4.9 ms of GPU inside cloud even after `cloud-budget`
and `cloud-reach` (`docs/benchmarks/2026-09-26-cloud-reach`), against a 120 fps
budget of 8.3 ms for the whole frame; before `cloud-budget` it was up to 40 ms.
Every ray marches up to 48 + 16 density samples, each with a light march, and a
temporal history with reprojection, clipping and an edge-aware upsample exists
only to hide the noise of doing that at 0.4 resolution.

Crab-breeze faced the same budget (owner there, 2026-10-24: "we're on a tight
budget, I just want good looking clouds with correct lighting and dont cost
much to render") and settled on **one alpha-blended billboard per puff**, lit
as a sphere by the scene's sun and sky, its edge eaten by noise, sorted back to
front and drawn in one draw (Crab-breeze `assets/shaders/cloud.wesl`, changes
`cloud-puffs` and `cloud-weather`, origin/main `d663b86c`). Its design weighed
ray-marched volumes at "1-3 ms even at quarter resolution with reprojection"
and turned them down.

## What changes

- **A second cloud renderer, `puffs`, beside the march.** `weather.ron` gains
  `cloud_renderer: Puffs | March`. Puffs is the default for this change's
  benchmark; March stays, unchanged, so the two are compared on the same flight
  in the same sitting and the owner chooses on screenshots.
- **Where puffs are is the atmosphere's.** A fixed set of puff sites covers the
  sphere (a jittered Fibonacci lattice, ~250 m apart, stable by index). Each
  frame a site samples the shown weather maps (cover, cloud top, optical depth),
  and its puff's size, height and opacity follow them; a site whose cover falls
  under its own threshold fades out rather than vanishing (no pop-in, priority 1).
  The shading is Crab-breeze's, with "up" the planet's radial direction.
- **Drawn where the march was drawn.** The puffs are rasterised into the cloud
  history targets the composite already reads (premultiplied colour, distance),
  at `cloud_render_scale`; the march and the history resolve do not run. The
  composite's haze (`cloud_air`), silhouette-aware upsample and near fog are
  unchanged, so a puff sinks into the same haze the march's cloud did.
- **Benchmark both on this Mac.** `tools/perf_suite.py` runs on macOS (a
  measurement instrument: process checks with `pgrep`, the exe without `.exe`).
  March against puffs, clouds off as the floor, on `clouds`, `storm` and
  `cloud-hop`.

## Not in this change

- Rain shafts and cirrus as billboards (Crab-breeze has both); PBD's rain
  volume and streaks stay.
- Cloud shadows and the lens mist follow the weather maps, not the puffs: the
  shadow of a lone puff is the map's, softer than the puff.
- Crab-breeze's other optimisations (traffic stepping, debris caps, tree
  rings, the 2048 px texture cap) are about systems PBD does not have; none is
  ported.
- Removing the march. That waits for the owner's choice on screenshots and the
  benchmark.
