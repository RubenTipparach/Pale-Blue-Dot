# smaa-option

Rendered in a Claude Code cloud container with no GPU, on lavapipe (Mesa's
software Vulkan) under Xvfb. These show what the frame looks like, not how
fast it runs: frame time was not measured (task 2.2 is the owner's, on the
desktop).

- `smaa-on-underwater-rod.png`: ANTIALIASING on, under water with the rod.
  `target/fast/pbd-app --walk --at -4.125 -11.125 --tool rod --pitch -10
  --time 11 --frames 360 --antialiasing on --capture <png>`. The held rod
  has no bright edge under the water.

## No late pipelines (task 1.4)

A `--real-time` run with ANTIALIASING off at launch, switched on and off
every 20 frames after READY (`PBD_CYCLE_ANTIALIASING=20 ... --antialiasing
off --real-time --frames 120`): READY at 102 pipelines, three switches on,
and no `LATE_PIPELINE` line at all. The loading screen keeps SMAA on the
cameras while it warms the world, so its three pipelines (edge detection,
blending weights, neighbourhood blending) are made before READY.

A fixed-step `--capture` does not warm (it lifts the screen on its first
frame), so there every pipeline, SMAA's three among them, is logged late.
That run is not a test of the prewarm.
