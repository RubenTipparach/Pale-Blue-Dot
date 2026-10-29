# Design: rain's fine detail only near the eye

## The cut

In the wet branch, after the puddle mask:

```wgsl
let near_detail = 1. - smoothstep(reach - fade, reach, distance_to_camera);
```

- `rings > 0.001 && near_detail > 0.001` gates the sheet and the rings; both
  are scaled by `near_detail`.
- `side_w > 0.001 && near_detail > 0.001` gates the rivulets, scaled the same.
- The branch is dynamic, so a far pixel skips the work, not only its weight.
  The test is per pixel; the fade band keeps the 2x2 quads that straddle it
  smooth.

`pg` and `pert` are zero past the reach, so `wet_n` is the face normal and the
specular-AA widening (`turn`) is zero there: the far film is the smooth wet
film at the wet or puddle roughness. That is what the footprint filter already
left of the rings at that range, so the far look barely moves.

## Data

`WeatherSettings` gains, validated (finite, `reach > 0`, `0 <= fade <= reach`):

| Field | Default | Unit |
| --- | ---: | --- |
| `rain_detail_reach_m` | 20.0 | metres from the eye |
| `rain_detail_fade_m` | 4.0 | metres, the fade band inside the reach |

They ride in `PlanetParams.wet` (a spare lane, or one more vec4); the shader
and the Rust layout are checked by the existing uniform size test.

## Measuring it

- A `rain-stroll` scenario in `tools/perf_suite.py`: Holbrook's lane in a
  forced rain. The storm flight sees wet ground only from far off; the stroll
  sees it near and far, on a script that never digs.
- Before against after, interleaved, 2 repeats, on `rain-stroll` and `storm`,
  with the GPU apps closed (Steam, Chrome, Spotify, VS Code).
- Stills: the same rainy street before and after, at the reach and well past
  it, in `docs/screenshots/wet-near-detail/`.

Recommendation taken (ask only with screenshots): 20 m as the owner said, a
4 m fade, and the reflections left for their own step.

## As built (2026-10-29)

Measured in `docs/benchmarks/2026-10-29-cull-wet`: no change at p50 on the
rain stroll (19.27 -> 19.15 ms, inside its 0.3 ms spread). The cut is what
the owner asked for, rings to 20 m and a wet film past it, at no cost; it is
not a measurable saving on this Mac.
