# Proposal: rain's fine detail only near the eye

## Why

The owner (2026-10-29), after the lighting-model perf sitting: "The wet
surface can also be simplified 20m from the camera. We don't need [rain]
drops [further] out than that."

Today every wet pixel to the horizon runs the full wet branch of
`planet_surface.wgsl`: the rippled water sheet (four cosines), the raindrop
rings (`rain_rings`, several hashed ring layers), and on walls the rivulets
(three `rivulets` evaluations for a finite-difference normal). Past a few tens
of metres a ring is smaller than a pixel: the branch already fades the rings
there by the pixel's footprint (`pg /= 1 + (foot*scale*3)^2`), so it pays for
detail it then throws away.

## What changes

- Past a configured reach from the eye (`rain_detail_reach_m`, 20 m), the wet
  ground skips the sheet's waves, the rings and the rivulets. It stays wet:
  darkened, puddled under the same mask, mirroring the sky and the scene, with
  the sun's glint and the lamps' highlights.
- The detail fades out over the last `rain_detail_fade_m` (4 m) before the
  reach, so nothing pops (priority 1, "No pop-in").
- Today's lighting model (`PBD_LIGHTING=today`) is left as it is.

## Not in this change

- The reflection traces (L8, the measured 1.6 ms) are not touched here. Limiting
  the voxel walk by distance changes what far puddles mirror; it is its own
  measured step, written up when it is taken.
