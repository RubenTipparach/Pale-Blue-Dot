# Design: cloud-entry

## 1. Passes and targets

All at the march's resolution except the composite:

| Pass | Reads | Writes |
| --- | --- | --- |
| march | scene depth, weather, cells | `current` (RGBA16F), `current_depth` (RG16F): this frame only |
| `cloud_resolve` (new) | `current` 3x3, `current_depth`, `history[read]`, `cloud_depth[read]` | `history[write]`, `cloud_depth[write]` |
| composite (full) | `history[write]`, `cloud_depth[write]` | the scene |

The ping-pong is unchanged: the resolve reads `[read]` and writes
`[1 - read]`, and the composite reads what the resolve wrote. The march's
targets become two single textures, `current` and `current_depth`, recreated
on resize with the history.

## 2. The resolve, per texel

1. Load `c`, `d` from `current`, `current_depth`. With the history off, write
   them as they are.
2. The box: the 3x3 moments of `current`, `mean +/- 1.25 sigma`, widened to
   include `c` (as `68e98bd`).
3. The point and `was`: the march's code for them moves here unchanged,
   `cloud-edges`' held-distance lookup reading `cloud_depth[read]`.
4. `cloud_previous(was, stop, point)`, unchanged: `cloud-ghosting`'s four
   tested taps, renormalised.
5. `out = mix(clamp(previous.cloud, lo, hi), c, CLOUD_BLEND)`.
6. `cloud-edges`' distance rule, after the clamp: a ghost clipped to nothing
   gets no distance and is not read again.

Where the neighbourhood holds no cloud the box is zero, and a curtain dies in
one frame; where the noise toggles at a real fringe, the box is wide and the
history stays.

## 3. Less raw noise

- **Jitter**: interleaved gradient noise, offset each frame,
  `q = pixel + 5.588238 * (frame % 64)`,
  `jitter = fract(52.9829189 * fract(dot(q, (0.06711056, 0.00583715))))`.
- **Footprint floor** half a step, not a quarter (a named constant): the 9 m
  octave is off at the 12 m near step.
- **The third shape octave** fades to its mean (0.075) by the same footprint
  weight, the value `coarse` already uses.
- **The sea's hit** in `cloud_far` over the same four corners as the scene
  depth, the farthest winning.

## 4. The near field

`NEAR_M = 2 * cloud_step_m` (24 m).

- The march: two fixed, unjittered samples at a quarter and three quarters of
  `[span.x, min(NEAR_M, far)]`, the first 24 m from the EYE (empty unless the
  layer starts within 24 m, so a distant cloud's edge is never drawn from it), of the coarse density (footprint 0),
  lit once; written to a third target, `near` (RGBA16F: lit colour,
  extinction per metre). The jittered march then starts at the end of that
  slab.
- The composite, per pixel with its own `far`:
  `len = clamp(min(far, NEAR_M) - span.x, 0, NEAR_M)`,
  `n = 1 - exp(-near.a * len)`, and the pixel is the far cloud over the scene,
  seen through the near slab: `mix(scene * (1 - c.w) + c.rgb, near.rgb, n)`.
- Those 24 m hold a quarter to a half of the opacity inside a cloud, never
  enter the history (no noise, no parallax), and thicken continuously as the
  eye crosses a cloud's edge.

## Verification

- `tools/check_all.ps1` (the GPU tests build the march, resolve and composite
  pipelines).
- The limb frame (`--route far-side --time 10 --frames 3000`) against
  `cloud-edges`' after.
- The `cloud-ghosting` turn check (`--tool shovel --pitch 14 --time 11 --rain
  0.25 --turn 90 --frames 150`): 0.000% change turning, the still within its
  floor.
- A real-time recording of cloud-hop through the entry (17.5-20 s) and of the
  scenic route, for the owner.
- The perf suite on `clouds`, `storm`, `cloud-hop`, reading `gpu_clouds_ms`.
