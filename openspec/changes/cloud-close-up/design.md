# Design: cloud-close-up

## Evidence

| What | How it was seen |
| --- | --- |
| Columns come from the clouds pass | `output/captures/hop/{base,rain,clouds}.png`: the same route moment with no pass off, the rain pass off, the clouds pass off (`PBD_PASS_OFF`, an instrument added for this) |
| The prism shape | `clouds.wgsl` `cloud_density`: `q = cloud_sheared(radial*22.0, ...) + vec3(height*1.7)` |
| Ghosting and the renormalised fallback | `water.wgsl` `cloud_march_pass` (history blend), `cloud_upsampled` (`sum/max(weight,1e-6)` with rejected texels weighted 1e-3) |
| No haze on clouds | `water.wgsl` `clouds`: `scene*(1-w) + rgb`, no distance term |

## 1. Shape noise in three dimensions

`q = cloud_sheared(p * per_metre, wind, layer)` with `per_metre = 22/base`,
the scale `radial*22` had at the base, so a lump keeps its size across the
ground (about 230 m) and gains the same scale up and down. The flat base and
the billowing top stay the height profile's job, and the convective cells and
fine octaves follow `q` as before.

## 2. The resolve

Targets at the march's resolution (`cloud_render_scale`):

| Target | Written by | Read by |
| --- | --- | --- |
| `current` (RGBA16F) | march | resolve |
| `distances[2]` (RG16F, ping-pong) | march writes `[w]` | resolve reads `[w]` and last frame's `[r]`; composite reads `[w]` |
| `history[2]` (RGBA16F, ping-pong) | resolve writes `[w]` | resolve reads `[r]`; composite reads `[w]` |

The resolve, per texel:

1. This frame's 3 x 3 neighbourhood of `current`: mean and standard deviation
   per channel; the clip box is `mean +/- 1.25 sigma` (Salvi 2016), widened to
   include the centre texel.
2. Reproject: the texel's cloud point (its distance, or the march's far end
   where it has no cloud) projected by last frame's camera.
3. Disocclusion: last frame's march far at that point (`distances[r].g`)
   against where this frame's ray reaches; if last frame's was shorter by
   more than 10% and 20 m, something stood in front then, and the history is
   not used there.
4. The history, clipped toward the box centre, blended with this frame at
   `CLOUD_BLEND` = 0.1 (from 0.06: the clip removes the reason the blend had
   to be slow).

## 3. Haze

The air between the eye and the cloud, as optical depth: Simpson's rule over
five points from the eye to the cloud of `exp(-altitude/FOG_HEIGHT_M)`, times the distance,
times `FOG_DENSITY_PER_M` (the ground haze's own density) times `cloud_haze`
from `weather.ron` (0 turns it off). The default, 3.0, lies between stills
of the same route moment at 2 (the layer still stands at the horizon as a
grey band) and 4 (it is gone and the sky behind shows). The sky itself has
little horizon glow yet, which is priority 4. The cloud's premultiplied
light and coverage are both scaled by `exp(-optical depth)`, so the scene
behind (the sky's own horizon glow, or already-hazed ground) shows through:
the cloud dissolves into whatever the air in front of it would show, without
a second, disagreeing sky colour.

## Verification

- Stills: the three `hop` captures again after the change (columns gone).
- Ghosting: a capture looking past a tree line at cloud while moving, before
  and after.
- Frame time: `tools/perf_suite.py`, old exe (`output/perf/baseline-exe`)
  against new, interleaved, on `clouds`, `storm` and `cloud-hop`; the report
  in `docs/benchmarks/`.
- A new fly-through recording for the owner to judge.
