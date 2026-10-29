# Design: cloud-puffs

## 1. Options against "cheap, good looking, lit right"

| Option | Cost on the 3070 | Kept? |
| --- | --- | --- |
| The march (shipped): 48 + 16 steps a ray at 0.4 resolution, light march, history, clip, upsample | 4.1-4.9 ms GPU in cloud (`2026-09-26-cloud-reach`) | stays as `March`, for the A/B |
| Crab-breeze `cloud-puffs` v1: lumpy icosphere meshes, ~500 triangles a puff | one blended layer a blob; their owner later chose billboards | no: from orbit PBD shows a hemisphere of cloud, ~2,000 puffs, a million triangles |
| **Crab-breeze `cloud-weather`: one billboard a puff, lit as a sphere, noise-eroded edge, sorted, one draw** | ~1 blended layer a puff; 2-3 where a cluster overlaps | **yes** |
| Impostor atlases | cheap, but a bake pipeline | no |

Source: Crab-breeze origin/main `d663b86c`, `assets/shaders/cloud.wesl` (`puff`,
`facing`, `fbm2`, `gn`) and `crates/cb-core/src/world/clouds.rs` (`Puff`).
Copied with provenance; Crab-breeze is not a dependency and is not modified.

## 2. Where the puffs are

Crab-breeze lays puffs out per weather system on a flat map. PBD's weather is a
simulated atmosphere on a sphere, already published as cube maps (`WeatherMaps`:
cover, cloud top 0..1 of `CLOUD_THICKNESS`, precipitation, optical depth) that
the march reads. The puffs read the same maps, so a puff stands where the march
drew cloud and rains where the rain falls.

- **Sites.** `pbd_core::cloud_puffs::sites(radius_m, spacing_m)`: a Fibonacci
  lattice of `round(4 pi R^2 / s^2)` directions, each nudged by a hash of its
  index by up to a third of `s`, with three stable hashes (`threshold`, `size`,
  `seed`). `s` = `cloud_puff_spacing_m` (250 m): 4,632 sites on the 4.8 km body.
  Stable by index, so the same weather lays the same puffs every run.
- **Each frame**, for every site, the shown maps' mix at its direction
  (the `smooth-weather` pair, mixed by `t` as the GPU blend does) gives cover
  `c`, top `h` and optical depth `od`:
  - a low deck keeps lanes, as the march's `cloud_deck_floor` opens them: the
    cover a site sees is `c * (1 - deck_floor * (1 - convective))`, with
    `convective = smoothstep(0.35, 0.7, h)`. A full deck at the shown maps'
    usual top of 0.3 leaves 45 % of its sites open; a tower leaves none;
  - presence `a = smoothstep(threshold - 0.06, threshold + 0.06, c') *
    smoothstep(0, 0.12, c')`: about a share `c'` of the sites carry a puff, one
    fades over a change in cover of 0.12 instead of appearing in one frame, and
    no cover carries nothing however low a site's threshold;
  - across radius `r = s * (0.5 + 0.35 c) * (0.8 + 0.4 size)`: about two puffs
    over a point at full cover. The first try, `0.55 + 0.65 c`, put four or five
    over a point and stacked them into an opaque sheet whatever each one's
    opacity (stills in section 7);
  - height `H = max(80 m, 0.9 h * CLOUD_THICKNESS)`, base on `CLOUD_RADIUS`,
    kept a tenth under the layer's top so the composite's shell never cuts it;
    a puff taller than `1.4 r` is stacked: up to three billboards up the
    column, each `H / n` tall, so a tower is a tower and not one tall ellipse;
  - darkness `dark = smoothstep(25, 90, od)`: storm grey from the column's
    optical depth. Measured on the shown maps (2026-10-28, all 4,632 sites):
    median 7, 75th percentile 19, 90th 52, 99th 130; the first guess, 4-18,
    turned ordinary cloud grey;
  - opacity `a * (1 - exp(-od / cloud_puff_optical_depth)) * cloud_puff_opacity`,
    `cloud_puff_optical_depth` 15: thin cloud is drawn thin, as the march draws
    it.
- Puffs with `a < 0.01` are dropped; the rest are sorted far to near from the
  eye and uploaded as instances (48 bytes each). The sort and the map reads are
  the CPU's: ~4,600 cube-map reads and a sort of at most a few thousand floats,
  measured in the benchmark's CPU wall time.
- **Wind.** The maps move as the atmosphere carries its cloud, so the puffs
  come and go with it; within a puff the billows scroll with the frame clock.
  A puff does not glide between sites: stated limitation, judged on video later.

## 3. Drawing

- **Into the march's targets.** With `cloud_renderer: Puffs`, the water node
  runs, in place of the march and the resolve:
  1. the march pass with its march skipped (`cloud_cells.w = 1`), so the near
     field's fog (`cloud_near_fog`, `cloud-entry`) is still made: flying into a
     cloud keeps the fog it has today;
  2. a raster pass of instanced quads into the history's colour (premultiplied,
     blend `One, OneMinusSrcAlpha`) and distance targets (blend `Min` on both
     channels, cleared to the far cap), at `cloud_render_scale`.
  The composite (`clouds` in `water.wgsl`) reads them as it reads the march's:
  its silhouette-aware upsample, its haze over the cloud's distance and its near
  fog are not changed, except that a distance at the cap means no cloud.
- **Depth.** A puff fragment computes the scene's stop as the march does
  (`cloud_far`: the depth buffer, the sea's sphere) and fades over
  `cloud_puff_soft_m` in front of it, so a puff behind a hill or the planet is
  cut by the full-resolution composite and softly where it meets the ground.
- **Facing.** Crab-breeze's `facing` with the planet's radial direction for up:
  a puff faces the eye's position and stays upright on the sphere, so turning
  the head never rolls it.
- **The eye inside a puff.** A billboard faces the eye through the puff's
  middle, so with the eye inside it one flat sheet lies over the whole view.
  It fades as the eye's distance from the centre, in the puff's own radii,
  falls from 1.6 to 0.8, and the near fog carries the cloud from there.
  (The grey that fills the cloud-hop stills from inside the layer is not
  this: it is there with the march and with the clouds off, and is the
  existing in-cloud fog and lens mist.)

## 3a. Inside a cloud, and the near fog's cost

Measured on the first build (section 6 and `-coast.jpg`):

- **Inside cloud the puffs were wrong.** The near fog (`cloud_near_fog`, two
  samples of the map's density, `cloud-entry`) reaches `cloud_near_m`, 24 m.
  Under the march, the march carries the cloud past that; under the puffs
  nothing does, so from inside the layer the view was faded billboards and
  clear air: a light grey with a dark smear where the march drew an even fog.
  **Change:** under the puffs the near fog reaches `cloud_puff_near_m`
  (120 m), the distance the puffs fade in over. The fog carries the cloud up to
  where the puffs take over, as the near fog hands over to the march.
- **The near-fog pass ran every frame** under the puffs, wherever the eye was:
  a full-screen pass at 0.4 scale with four depth reads a texel, most of what
  the puffs still cost. The composite reads the near fog only where the layer
  is within its reach of the eye. **Change:** with the puffs, the pass is
  skipped while the eye is more than that reach from the layer's shell (below
  the base or above the top), and the composite's own test keeps the stale
  target unread.

## 3b. Distant puffs do not need a near puff's detail

Owner, 2026-10-28: "Distant puffs. Do they need the same resolution?" No. On
the first build every puff fragment paid the same, however far:

- **Hidden fragments paid in full.** The test against the scene's depth is the
  shader's own (the puffs draw into the march's target, with no depth
  attachment), and it ran after the noise and the light. A puff behind a hill
  or past the planet's limb was shaded and then faded to nothing.
  **Change:** the depth and sea test first; a fragment behind the scene is
  discarded before any noise.
- **Puffs past the horizon were drawn.** From low down most of the sphere's
  puffs are behind the planet. **Change:** the CPU drops a puff whose top lies
  below the eye's horizon on the sea's sphere (the tangent cone from the eye,
  widened by the puff's own height and radius), before sorting.
- **A small puff ran every octave.** Three octaves of gradient noise for the
  billows and one for the edge, eight hashed corners each, on a puff a few
  dozen texels across. **Change:** detail by the puff's size on screen,
  `texels = 2 r / (distance * pixel_angle)`: under 48 texels the third octave
  fades out, under 24 the second and the edge noise, so a far puff is one
  octave. The edge softness is unchanged, so its outline still reads round.

A separate, coarser target for far puffs would save more fill but needs a
second input to the composite; it is left until these are measured.

## 4. Light (Crab-breeze `puff`, per pixel, body-local)

- the sun wrapped round a sphere normal bent by the billows (`wrap` 0.6);
- darker toward its system's base: `body = mix(1 - base_shade, 1, smoothstep(0, 0.7, h))`;
- its own underside shadowed, and the last 350 m above the base;
- a silver lining where the eye looks toward the sun through a thin edge;
- sky ambient from PBD's cloud layer (`cloud_ambient_sky`), dimmed toward the
  base, and a shade floor so a storm base is grey, never black;
- the night floor (`cloud_night_floor`) and a lightning flash (`cloud_flash`)
  as the march has them;
- haze toward the eye is the composite's `cloud_air`, as for the march.

New `weather.ron` values, validated: `cloud_renderer`, `cloud_puff_spacing_m`,
`cloud_puff_opacity`, `cloud_puff_wrap`, `cloud_puff_base_shade`,
`cloud_puff_silver`, `cloud_puff_erosion`, `cloud_puff_soft_m`,
`cloud_puff_near_m` (fade-in from the eye, where the near fog takes over).

## 5. Cost (expected; to be measured)

| Item | Value |
| --- | --- |
| Sites | 4,632 on the 4.8 km body |
| Instances drawn | ~cover x sites x (1-3), at most ~10,000 under full cover |
| Vertex work | 6 vertices an instance: nothing |
| Fragment work | one blended layer a puff, ~2-3 where a deck overlaps, at 0.4 x 0.4 of the view, ~3 noise reads a pixel against the march's 48-64 density samples and their light |
| CPU | ~4,600 map reads, a sort, a 0.5 MB upload at most |

## 6. Benchmark

On this Mac (MacBook Pro, Apple M4 Max, 40-core GPU, Metal; the owner's
desktop was busy with other tests), release, windowed 1440 x 900 logical,
which is 2880 x 1800 pixels on its Retina display: four times the pixels of
the desktop's 1440 x 900, so its numbers are not the desktop's.
`tools/perf_suite.py`, interleaved, repeats 2:

- `march` (`PBD_CLOUD_RENDERER=march`), `puffs` (default), `off`
  (`PBD_NO_CLOUDS=1`) on `clouds`, `storm`, `cloud-hop`.
- Report to `docs/benchmarks/2026-10-28-cloud-puffs/`, published as an HTML page.

The suite was written for the Windows desktop (`tasklist`, `pbd-app.exe`); it
gains a macOS path (`pgrep`, no `.exe`). That is a measurement instrument and
carries no behaviour change.

**GPU time on Metal** (`planet_cloud_timer.rs`, an instrument). Bevy's pass
spans write timestamps inside a pass, which Apple GPUs do not support, so on
the Mac `gpu_clouds_ms` read nought. The cloud passes now also carry
`timestamp_writes` at their own beginning and end, the one form Metal has (the
per-pass counterpart of `MTLCommandBuffer.GPUStartTime`/`GPUEndTime`), resolved
and read back three frames late into the frame log where the spans read
nothing. An Apple GPU overlaps consecutive passes, so one pass's pair brackets
its neighbours' work too and the pairs do not add up (measured: the march pass
and the resolve after it each read ~13.7 ms, the composite nought). What is
logged is the span from the first cloud pass's start to the last one's end. On
the desktop the spans read as before and this timer is not used.

## 7. Screenshots

Headless `--capture` from the stills the cloud changes already use (from under
the base, in the layer, over the tops, from orbit), March above and Puffs below,
into `docs/screenshots/2026-10-28-cloud-puffs-*.png`.

## Open questions

| Id | Question | Recommendation | State |
| --- | --- | --- | --- |
| P1 | Which renderer ships | the cheaper one if it holds up on the screenshots | owner's, with screenshots and the benchmark |
| P2 | Rain shafts and cirrus as billboards too | later, if the puffs are chosen | recommendation taken (ask only with screenshots) |
