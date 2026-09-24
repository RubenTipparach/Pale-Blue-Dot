# Design: clouds inside a frame budget

## Rain streaks

`rain()` in `water.wgsl` keeps its march, its map and its density. Only the
streak term changes:

```text
rel     = p - anchor * sea_radius          // metres from the map's anchor
across  = (rel . u, rel . v) / W
down    = (|p| - sea_radius + seconds * fall) / (W * stretch)
streak  = mix(noise(fine W), noise(broad W), smoothstep(near, far, t))
```

`W` is a constant per scale (`RAIN_STREAK_FINE_M`, `RAIN_STREAK_BROAD_M`), so a
point's noise coordinate depends on the point and the clock only. Height above
the sea replaces the radius so the coordinate stays small and exact in `f32`.
The fall direction is unchanged: the pattern moves toward the centre as
`seconds` grows. The distance blend only weighs the two scales. Neither one
moves when the eye does.

## Cellular noise texture

`planet_cloud_noise.rs` (pbd-app, derived render state) builds an `R8Unorm` 3D texture:
`CLOUD_CELLS_SIZE`^3 texels spanning `CLOUD_CELLS_PERIOD` lattice cells, with
periodic hashing so the texture tiles. Each texel holds the shader's own
function, `clamp(1 - smoothF1, 0, 1)` with smoothF1 the log-sum-exp minimum
over the 27 neighbouring cells at sharpness 8. It is built on every core at
startup, measured and logged. A test pins that it tiles (the face texels
continue across the wrap) and that its range is in 0..1.

`cloud_cells(q)` in `clouds.wgsl` becomes one `textureSampleLevel` with a
repeating trilinear sampler at `q / CLOUD_CELLS_PERIOD`. The layout and the
lattice are the texture's; the shader holds no second copy of the function.

## The march

`cloud_march` returns `CloudSample { cloud: vec4, depth: f32 }`.

- **Steps by length.** Sample `i` sits at `t_i + jitter * step_i`. The step is
  `clamp(t * CLOUD_STEP_GROWTH, cloud_step_m, CLOUD_STEP_MAX_M)` at the eye's
  distance `t`. A sample with no cover takes two steps' length, because the
  cover is smooth at map scale. The loop ends at the span's end, at
  `cloud_max_steps`, or when transmittance drops under 1%. The span keeps its
  grazing cap (40 layer thicknesses), which the probes for the cloud top read
  along.
- **The gap under the base.** From over the base, a ray that dips under it and
  does not meet the ground comes back up into the layer beyond the base's
  horizon. `cloud_span` keeps that whole chord, not only the part before the
  dip, and the march jumps the stretch under the base.
- **LOD from the larger footprint.** The fine octaves fade by
  `max(pixel footprint, step / 4)`, not by the footprint alone, so detail far
  finer than the march can resolve is not drawn. A quarter, not the whole
  step: the per-frame jitter and the history average along the ray, and fading
  by the whole step removed the fine octaves everywhere past a few metres.
- **Light.** The light march reads `cloud_density` with a zero footprint, which
  now skips the cellular texture and the shape's third octave (its mean
  stands in), in 4 steps toward the sun and 2 up, down from 6 and 3.
- **Depth.** `sum(t * dA) / sum(dA)` over the coverage each sample adds, or the
  span's middle when nothing is drawn.

## Reduced-resolution march

`Pass::Clouds` becomes two pipelines:

- `Pass::CloudMarch` draws a fullscreen triangle into the history target
  (`Rgba16Float`, sized `ceil(view * cloud_render_scale)`), reading the previous
  history. Its depth read takes the FARTHEST of the four corners of its
  footprint in the full-resolution depth, so a march texel on a silhouette
  holds the cloud the sky side sees.
- `Pass::Clouds` (composite) draws at full resolution. It reads the scene, and
  the new history bilinearly. It first runs the pixel's own `cloud_span`
  against its own depth. Where the span is empty it writes the scene
  untouched, so a hill in front of a cloud keeps a full-resolution edge.

Group 3 of the march: the previous history, a filtering clamp sampler, the 3D
cells texture and a repeating sampler. Group 3 of the composite: this frame's
history, the sampler, and the distances at binding 4.

The history pair is recreated when the view's size or the scale changes. The
validity rules (`clouds_ran`, resize, `CLOUD_HISTORY_JUMP_M`) are unchanged.

## Knobs

In `WeatherSettings` / `weather.ron`:

| Knob | Unit | Range | Meaning |
| --- | --- | --- | --- |
| `cloud_render_scale` | fraction | 0.25..1 | march resolution per axis |
| `cloud_step_m` | m | > 0 | the nearest step |
| `cloud_max_steps` | count | 8..256 | cap per ray |

They reach the shader in `CloudLayer.cells.yz` (spare today). The scale is
read on the CPU only.

## Verification

- The bench from the proposal, before and after, at 200, 450, 650 and 900 m.
- Captures at the same heights, compared by eye for the seam and the fur.
- `cargo test` (WGSL parse and validate of the shipped shaders, the
  uniform-size test, the noise tiling test), `cargo fmt --check`, `cargo
  clippy`, `openspec validate --all`.
- The rain fix is judged with a moving capture and not a still one. It is
  argued from the coordinate: every term of the noise input is a function of
  the point and the clock.
