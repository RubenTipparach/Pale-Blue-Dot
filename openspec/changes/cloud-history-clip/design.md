# Design: cloud-history-clip

## Passes and targets

| Target (march resolution) | Written by | Read by |
| --- | --- | --- |
| `current` (RGBA16F, new) | march | resolve |
| `cloud_depth[2]` (RG16F, `cloud-ghosting`'s pair) | march writes `[w]` | resolve reads `[w]` (this frame) and `[r]` (the previous, for `cloud_previous`); composite reads `[w]` |
| `history[2]` (RGBA16F) | **resolve** writes `[w]` (was the march) | resolve reads `[r]`; composite reads `[w]` |

The resolve's fourth bind group: `0` the previous history and `1` its
sampler, `4` the previous distances (the bindings `cloud_previous` already
reads), `5` this frame's march, `6` this frame's distances.

## The resolve, per texel

```text
current   = this frame's march at the texel
if the history is not usable (a jump, a resize): current
box       = mean +/- 1.25 sigma of current's 3 x 3, widened to include current
point     = the ray to this texel's cloud (its distance, or where its march
            stopped), projected by the previous camera -> was
stop      = the ray's own stop (its march far, capped)
previous  = cloud_previous(was, stop)       # cloud-ghosting, unchanged
if nothing kept: current
else: mix(clamp(previous, box), current, CLOUD_BLEND)
```

The reprojection point is the one `cloud-ghosting`'s march used: the cloud's
own point, with the march's stop where there is no cloud.

## Why the clip does not undo the smoothing

A single march sample is noisy (one jittered ray a frame), so this frame's
neighbourhood is wide wherever there is cloud, and a converged history sits
inside it: the clip leaves it alone. It bites where the history is far outside
what any neighbour shows: cloud carried in by a bad reprojection, or a cloud
that has changed. The blend stays at 0.06, so still cloud is as calm as now.

## Verification

- `cloud-ghosting`'s own captures, before (`main` at `964ddf4`) against after:
  **still** (`--turn 0`) must sit on the floor (two runs of one binary);
  **turning** (`--turn 90`, shovel held) must not bring the ghost back.
- **Into a cloud**: the `--route clouds` capture at the frames where it enters
  and is inside cloud, before against after.
- Frame time: the perf suite, `clouds`, `storm`, `cloud-hop`, before against
  after, same sitting.

## What it measured

Captures (`--capture`, 1440 x 900, RTX 3070), `main` at `964ddf4` on its own
assets against this change; the share of pixels moved by more than 8 and 24
of 255, and two runs of the before exe for the floor:

| Scene | Floor | Before vs after |
| --- | ---: | ---: |
| Still (`cloud-ghosting`'s: `--walk --tool shovel --pitch 14 --time 11 --rain 0.25 --turn 0 --frames 150`) | 0.000% / 0.000% | 0.000% / 0.000% |
| Turning (the same, `--turn 90`) | 0.000% / 0.000% | 0.000% / 0.000% |
| `--route clouds`, frame 1300 | 0.004% / 0.003% | 2.400% / 0.448% |
| `--route clouds`, frame 1500 | 0.000% / 0.000% | 4.443% / 1.640% |

- **At rest and turning, nothing changes**: the clip leaves a converged
  history alone, and `cloud-ghosting`'s silhouette test is untouched.
- **Flying past cloud, the difference is the smear.** Before, frame 1500
  shows white sheets and straight streaks over the ground on the left and
  faint lines under the clouds, all pointing along the flight: history
  reprojected a little wrong every frame and piled up. After, they are gone
  and a light haze remains
  (`docs/screenshots/cloud-history-clip-before-after.png`, before on top).
