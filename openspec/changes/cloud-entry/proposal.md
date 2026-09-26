# Proposal: flying into a cloud is a smooth transition with no artifacts

## Why

The owner (2026-09-25): "Make sure flying into clouds has no artifacts and is
a smooth transition." Priority 3 in `CLAUDE.md`.

`cloud-edges` removed the salt that filled the frame at entry and the bright
rim on limb clouds. What is left, on the cloud-hop recording on that build
(145 m/s, 1440x900), from 18.3 s to 19.2 s:

- **stacked sheets**: wavy bands of cloud while the camera passes through a
  thin cloud, for about 0.3 s;
- **speckle** along the fringes of distant clouds and the sea horizon;
- **a curtain**: a vertical smear at a cloud's side for about 0.6 s.

## What was found (read off the code and the frames)

1. **Sheets: the history is reprojected at one depth and never checked
   against this frame.** Inside a thin cloud the cloud along a ray runs from
   the eye to hundreds of metres; the near 12 m steps move across the screen
   many times faster than the coverage-weighted mean depth the history is
   reprojected by. The history keeps 94% and nothing compares it with what
   this frame sees, so each frame's copy survives, displaced once more: a
   stack of sharp-edged copies. `cloud_texel_sees_to` only tests depth order
   against the scene, so it cannot catch it. The resolve pass built on
   `68e98bd` (`cloud-close-up` design section 2) clipped exactly these
   "offset copies" to the neighbourhood.
2. **Curtain: cloud that has left a texel only decays.** A texel whose march
   finds no cloud keeps 94% of its history, and since `cloud-edges` it keeps
   the history's distance and reads it again there, so it passes again; the
   6% decay (about 38 frames to 10%) is the curtain as a cloud's side sweeps
   by.
3. **Speckle: raw samples too noisy to rest on.** The jitter is white noise;
   the steps outgrow the detail they sample (the 38 m third shape octave is
   never faded, the fine octaves' footprint floor is a quarter of a step, and
   at 1-3 km a step is 40-120 m), so a sample at a fringe is all or nothing.
   Where every history tap fails, that sample is what shows. On the sea
   horizon the march takes the sea's hit on the centre ray only while the
   scene depth is the farthest of four corners, so limb texels flip between
   "stopped at the sea" and "sky" as the eye moves and fail the history test.

## What changes

1. **The resolve pass writes distances too.** The clip itself landed as
   `cloud-history-clip` (the other machine's, `bef7d58`): the march writes
   this frame's cloud alone and a resolve clips the reprojected history to
   this frame's 3x3 neighbourhood. This change has the resolve write the
   history's distances as well, so a texel whose cloud is the history's alone
   keeps one (`cloud-edges` section 1), and one the clip empties has none.
2. **Less raw noise**: interleaved gradient noise for the jitter, offset each
   frame (every 3x3 block covers the range, which the clip needs); the fine
   octaves' footprint floor raised to half a step, and the third shape
   octave faded to its mean by the same rule; the sea's hit taken over the
   same four corners as the scene depth.
3. **The near field is the cloud's own fog.** The first `2 x cloud_step_m`
   (24 m) of each ray inside the layer is taken by two fixed samples of the
   coarse density, lit once, at full resolution in the composite and never in
   the history: no noise and no parallax there. The fog thickens continuously
   as the eye crosses a cloud's edge, which is the transition.

## Out of scope

Condensation on the lens (`lens-weather`), a full-resolution near-field
march of the detail noise.
