# Design: the clouds pass accumulates over frames

## Why accumulation

The proposal's table is the argument: a thin cloud point-sampled at random
depths is noisy at any affordable step count (64 steps cost 2.5x the frame and
still grain at the edges), and a fixed depth instead draws contour bands. Every
renderer that marches clouds in real time pays the noise down over time
instead: each frame marches with a different random offset, and the result is
blended into a history that is carried from frame to frame, so N frames of
history are N times the samples for the cost of one (Schneider, Horizon Zero
Dawn, 2015; Hillaire, Frostbite, 2016). The pace makes this safe here: cloud now
moves under a degree a second, so the history is still right a few frames on.

## What changes

**A history pair per view.** `WaterViewGpu` gains two `Rgba16Float` textures
the size of the view and an index saying which was written last. The clouds
pass reads one and writes the other; they swap each frame. A texel holds the
cloud's light premultiplied by its coverage, and the coverage: the pass's own
`vec4` result, before it is composited over the scene.

**The clouds pass writes two targets.** Location 0 is the composite, as
today. Location 1 is the new history texel. The pass's pipeline gains a fourth
bind group (the previous history and a sampler) and the second target; no
other pass changes.

**Reprojection by the cloud's own point.** For each pixel the march's span
gives a representative point, the middle of the part of the ray inside the
layer. The previous frame's `clip_from_local` (a new lane in `WaterView`)
projects that point to where it was on screen last frame. The history is read
there, bilinearly. Off screen, on the first frame, after a resize, or after the
camera jumps (a teleport, a load), there is no history and the frame stands
alone.

**The blend.** `history = mix(previous, current, CLOUD_BLEND)` with
`CLOUD_BLEND` near a tenth: about ten frames of samples. The jitter becomes a
hash of the pixel AND the frame, so each frame samples new depths; that is the
whole point.

**Where there is no cloud to march** (terrain in front, a ray that misses the
layer) the pass writes an empty history texel and composites nothing, so
cloud never ghosts over a hill that moved in front of it. The price is that a
cloud edge at a silhouette fills in over a few frames as the camera turns.

## What it costs

One more render target write and one bilinear read per pixel, against the
march's hundreds of noise samples; measured after, with the same capture.

## What is not settled

- Neighbourhood clamping, the usual guard against ghosting, needs the
  current frame's neighbours, which one fragment does not have. Without it a
  fast change (a lightning flash, a cut) smears for a few frames. The blend
  factor is the knob if it shows.
- The captures are single frames; a still capture after enough frames shows
  the converged picture, which is what the eye sees at rest.
