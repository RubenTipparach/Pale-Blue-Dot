# Proposal: cull cloud puffs outside the view

## Why

The owner (2026-10-29), after asking whether the clouds are culled: "Cloud
puff frustum culling then. Make sure not to cull stuff whose bounding box
intersects with frustum."

Today the puffs (`cloud-puffs`) are culled only past the planet's horizon
(`cloud_puffs::over_horizon`, pinned by
`a_puff_past_the_horizon_is_hidden_and_one_near_it_is_not`). Every puff on
the near side of the horizon is built, sorted far to near and drawn,
including those behind the camera. The GPU clips their billboards before
shading, so the cost is mostly CPU: the per-frame sort, the instance upload
and the vertex work, for puffs nobody sees.

## What changes

- After the horizon cull, a puff is kept unless its bounding box lies wholly
  outside the view frustum. A box that touches or crosses the frustum is
  always kept: the test is conservative, never exact-or-less.
- The box bounds the billboard in every orientation it can take (it turns to
  face the eye), so a puff is never cut while any corner of it can reach the
  screen.
- The frame log gains the puff counts (built, past the horizon cull, past the
  view cull) and the CPU time of the build, so the cull's effect is measured
  rather than assumed.

## Not in this change

- Occlusion culling (puffs hidden behind terrain). Hidden pixels are still
  discarded in the fragment shader as today.
