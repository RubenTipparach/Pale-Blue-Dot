# Design: cloud-edges

**Landed on top of `cloud-history-clip`** (`bef7d58`, the other machine's),
whose resolve pass now reads the history. Sections 2 and 3 live in that
resolve; section 1's first half (the resolve keeping a distance for a texel
whose cloud is the history's alone) needs the resolve to write distances,
which `cloud-entry` adds; until then the composite's fallback to the layer
entry is what hazes such a texel, and on the limb stills it removes the rim.

## 1. A distance for every texel that holds cloud

`cloud_previous` also returns the kept texels' cloud distance, weighted by the
cloud each holds, as the composite's upsample weights its haze distance. The
march pass writes `depth.x`:

- this frame's cloud distance where this frame's march found cloud (as now);
- otherwise, where the blended cloud still holds cloud, the distance along
  this ray at which the history was read (section 3), which is the history's
  own cloud distance, measured from the previous eye (the eye moves a few
  metres a frame against cloud distances of hundreds to thousands);
- zero only where the texel holds no cloud.

The composite's `clouds` pass measures the haze to the kept texels' distance
as now; where none has one, to `span.x`, this pixel's own entry into the
layer (zero from inside the layer, which is right: no air lies between).

## 2. The history test reads the cloud's reach, not the ground's

`cloud_texel_sees(texel, far)` stays the composite's test. The history's
becomes `cloud_texel_sees_to(texel, far, reach)`:

```text
near_enough = texel.x <= far*1.03 + 10        far:   this ray's stop, from the previous eye
far_enough  = texel.y >= min(reach, far, CAP)*0.97 - 10
                                              reach: the point read, from the previous eye
```

`near_enough` is unchanged: a previous cloud beyond this ray's occluder is
still refused. `far_enough` asks that the previous march reached the cloud
being read rather than this ray's ground, so it still refuses history where
something stood in front of the cloud last frame (the held tool, a hill
sliding in front), and no longer refuses it for parallax against the ground
behind. With `reach >= far` the test is the old one.

## 3. Read the history where its cloud is

Where this frame's march found cloud, the history is read at this frame's
cloud point (as now). Where it found none, the span's middle is only a guess:
the history is looked up there once, and if the texel there holds a cloud
distance, the point is moved to that distance along this ray (clamped to the
span) and the history is read again at it. One extra texture read, only for
texels without cloud this frame.

## Verification

- `--route far-side --time 10 --capture --frames 3000`, before and after: the
  limb clouds' rim (the before is in `output/captures/cloudfix/`).
- A real-time recording of cloud-hop for the owner: the entry (18 s) and the
  descent (28-29 s).
- `cloud-ghosting`'s check: `--turn` with the held tool against cloud, no
  print of the tool.
- The perf suite on `clouds` and `cloud-hop`: one extra read, for cloudless
  texels only.
- The GPU tests (the composite and march pipelines build).

## What it measured

- **The rim** (`--route far-side --time 10 --capture --frames 3000`, the
  release binary with the old and new `water.wgsl`): before, both limb
  clouds carry a stair-stepped bright rim and a dotted line along the limb;
  after, their edges are soft and hazed like their insides
  (`output/captures/cloudfix/limb-before-after.png`).
- **Ghosting kept** (`cloud-ghosting`'s check: `--tool shovel --pitch 14
  --time 11 --rain 0.25 --frames 150`, shovel against overcast): turning at
  90 degrees a second, before against after, 0.000% of pixels differ by more
  than 8 of 255; still, 0.003% (the floor, two runs of one binary, 0.000%).
- **Entering a cloud**: judged on a real-time recording; capture mode's
  weather drifts differently, so the entry does not fall on a known frame.
