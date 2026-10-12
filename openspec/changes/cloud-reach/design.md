# Design: cloud-reach

## 1. The march's tail

`cloud_march` (`clouds.wgsl`) keeps its loop and its step rule:
`step = clamp(t * CLOUD_STEP_GROWTH, cloud_step_m, CLOUD_STEP_MAX_M)`, two
steps' length over empty cover, the gap under the base jumped. The loop runs
up to `cloud_max_steps + CLOUD_TAIL_STEPS` iterations. From iteration
`cloud_max_steps` on, the step is at least what is left of the span, not
counting the gap under the base still ahead, divided by the iterations left:

```text
left = (far - t) - |[t, far] n [gap.x, gap.y]|
step = max(step, left / (cloud_max_steps + CLOUD_TAIL_STEPS - i))
```

- **The near steps are unchanged.** A ray that reached its span's end, or
  went opaque, within the budget is drawn exactly as before. A step's length
  depends on its distance from the eye alone until the budget is spent. That
  keeps what `cloud-budget` learned: a step count spread over the span made
  the step jump wherever the span did, and drew the seam.
- **Continuous across the base's horizon.** Just above the base's horizon a
  ray ends at the layer's top, 3-4 km out. Just below it, the ray jumps a gap
  that shrinks to nothing at the horizon and ends at the same top. `far` and
  the gap are continuous there, so the tail is too.
- **Where the budget ends varies** with the cover and the per-frame jitter,
  pixel to pixel and frame to frame. The history averages it, so the switch
  to long steps does not draw a line.
- **Detail.** A tail step is hundreds of metres long. Its footprint, half a
  step (`CLOUD_FOOTPRINT_STEPS`), fades the fine octaves as it does at any
  step, so the tail draws the coarse shape: the far deck as the rays under
  the base already draw it with 100-160 m steps.
- **Light** is marched on every second cloudy sample, as for any sample.

## 2. `CLOUD_TAIL_STEPS`

This is a shader constant, beside `CLOUD_STEP_GROWTH` and `CLOUD_STEP_MAX_M`
(the march's constants are there, the per-machine budget in `weather.ron`).
It is chosen from stills against the 256-step reference: the fewest steps
that draw the far deck with no arc and no visible stepping. The candidates
were 8 (about 330 m a step for the 2.6 km left above the base's horizon) and
16 (about 170 m, near `CLOUD_STEP_MAX_M`); it is 16 (below).

## Cost

Only rays that spend the whole budget and stay translucent take tail steps:
from inside the layer, rays near the horizontal through broken cloud. From
under or over the layer, a ray starts at the layer's edge, already at long
steps, and 48 of them cover its chord. The tail costs at most
`CLOUD_TAIL_STEPS / 48` of the march on those rays. The perf suite prices it,
old against new, on the cloud scenarios.

## Verification

- Stills at the owner's frame (`--route clouds --frames 1300`), before and
  after, beside the 256-step reference. In the columns across the old arc, the
  largest step in brightness from one row to the next is no bigger than the
  reference's.
- Stills from 350 m and 650 m inside the layer (`--view column --height`),
  from under the base and from over the tops. Nothing changes where the
  budget already reached the span's end.
- The `--turn` check (`cloud-ghosting`): the tail's history does not ghost.
- The perf suite (`clouds`, `cloud-hop`, `storm`), old against new,
  interleaved.

## Limits

There is no GPU test of the march itself; the shader tests compile it. The
proof is the stills and the brightness measure above.

## What building it found

- **16 tail steps, not 8.** The owner's frame at `--frames 1300`
  (`docs/screenshots/cloud-reach-ring.png`: as shipped, 16 tail steps, 256
  steps): both remove the arc, and both match the 256-step reference below it. Zoomed, 8 steps leave the interleaved-gradient
  jitter's crosshatch over the far deck above the old arc, where each step is
  about 330 m. At 16 it is much fainter, and a step is about 170 m, close to
  the 160 m the march itself takes that far out.
- **The arc, measured.** In the columns across it (x 820-1120, rows
  150-250), the largest brightening over 6 rows downward is 13 grey levels
  as shipped, 2 with 8 or 16 tail steps, and 1.5 at 256 steps. Below the old
  arc the column is identical in every variant: only the stretch past the
  budget changed.
- **Around the layer** (`--view column --height 200/400/600/1000 --pitch -8`,
  `output/captures/reach/grid.png`): under the base and over the tops the
  picture is unchanged. From inside the layer, far clouds that ended
  short now run on to the horizon.
- **The turn check** (`--tool shovel --pitch 14 --time 11 --rain 0.25
  --frames 150`): still, before against after, 0.001%, which is the floor
  (two runs of one build also differ by 0.001%); turning at 90 degrees/s, 0.000%.
