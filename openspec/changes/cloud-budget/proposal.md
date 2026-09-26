# Proposal: clouds inside a frame budget, and rain that falls wherever you stand

## Why

The owner, flying up through a storm: "at a certain altitude clouds stopped
working, rain fall animation in the distance not falling correctly. If I stop
moving they don't move, and if I do move in one direction they fall down, in
another they fall up." And: "measure how cloud rendering impacts performance
and figure out a good setting for my pc ... at least 120 fps, 60 fps at the
VERY MINIMUM." Then, of a capture from inside the layer: "why is it showing
planet curvature through opaque clouds?"

Measured on the owner's desktop: RTX 3070 (driver 595.97, Vulkan), i7-9700F,
release build, 1440 x 900, `--view column --pitch -25 --rain 1 --frames 400
--fixed-dt`, two runs per row, `FRAME_WALL_MS` (first 60 frames excluded).

### The cost is the clouds pass, and inside the layer it is 40 ms

| Camera height | Shipped p50 / p95 | Clouds pass off | Rain volume off |
| ---: | ---: | ---: | ---: |
| 200 m (under the base) | 10.4 / 12.5 ms | 3.8 / 5.3 | 9.6 / 16.1 |
| 450 m (in the layer) | 29.6 / 54.5 | 2.4 / 3.0 | 29.4 / 55.2 |
| 650 m (in the layer) | **40.8 / 72.8** | 2.2 / 2.8 | 39.6 / 71.7 |
| 900 m (over the tops) | 19.6 / 36.3 | 2.2 / 2.7 | 19.6 / 35.8 |

So the whole scene without clouds runs at 2-4 ms, and the clouds pass is
between 70% and 95% of every frame at altitude. Inside the layer the owner
is at 25 fps median and 14 fps at the 95th percentile. The rain volume costs
nothing measurable.

Where the clouds pass spends it (650 m, p50):

| Variant | p50 | What it says |
| --- | ---: | --- |
| Shipped | 40.8 ms | |
| Cellular (Worley) noise off everywhere | 10.9 | **Worley is ~75% of the pass** |
| Worley only on view samples, not in the light march | 18.2 | the light march calls density 9 times per lit sample, and each call ran 54 Worley cells |
| No light march (flat light) | 12.3 | light is the rest |
| 8 steps instead of 16 | 22.7 | per-pixel cost is in the samples, not the loop |

Each Worley evaluation is 27 cells x 3 hashes and an `exp`, done twice (the
two flow phases), in every `cloud_density` call. It runs for every view sample
and every light-march sample.

### The rain streaks are anchored to the camera, not the world

`water.wgsl`'s `rain` pass draws a streak with noise whose coordinate is the
sample's position divided by `width = 1.5 + t*0.004`, where `t` is the distance
**from the eye**. Walk toward a shower and `t` shrinks for every point in it, so
the whole noise field is rescaled around the planet's centre. The body-local
coordinates are ~5 km, so a width change of a percent moves the pattern by
many streaks. At 100 m/s the rescale moves the pattern about ten times faster
than the 70 m/s fall scrolls it. So the streaks run down as you approach,
up as you back away, and when you stop they fall at only a tenth of that
rate.

### The curve through solid cloud is the step length jumping

From inside the layer a ray below the cloud base's horizon leaves the layer
through the base after a few hundred metres. A ray just above it runs sideways
through the layer for up to `40 * CLOUD_THICKNESS` = 18 km. Both get the same 16
steps (`CLOUD_MIN_STEPS = CLOUD_MAX_STEPS = 16`), so the step is a few tens of
metres on one side and 1.1 km on the other, against cloud features ~230 m
across. Long rays average into flat fog, and short ones keep their structure.
The seam between them is the base sphere's horizon, a hard arc across
cloud that should read as solid. The `cloud-detail` descent wrote this up
("hard curved seams", "fur at grazing views") and planned the cure: march by
length, not by count. It was never built.

### The history smears when the eye moves

The `calm-clouds` accumulation reprojects each pixel's history by the middle
of the ray's span through the layer. Inside the layer that middle can be 9 km
out while the cloud the pixel shows is 50 m away. So any movement reprojects
from the wrong place, and with a blend of 0.06 (sixteen frames) the error
smears. That is the ghosted grey mass in the owner's first capture.

## What

1. **Rain streaks fixed in the world.** Two streak scales, fine near and broad
   far, each with a constant width in metres and coordinates relative to the
   precipitation map's anchor. Distance chooses how much of each is drawn,
   never where it is. Only the fall scrolls them.
2. **Cellular noise baked once.** A tileable 3D texture of the same smooth-F1
   cellular noise, built on the CPU at startup. The shader reads one trilinear
   tap per phase instead of evaluating 27 cells.
3. **The light march takes the coarse shape only.** No cellular texture in the
   light march (HZD's cheap samples, as the code already says of the fine
   octaves).
4. **March by length, not count.** The step grows with distance from the eye
   (a few metres near, capped far), under a total step cap and the existing
   early exit. Neighbouring rays then sample at matching intervals, which
   removes the seam and the fur together, and the grazing span's 18 km cap
   drops to a reach set by the step cap.
5. **Reproject by where the cloud is.** The march returns the
   coverage-weighted distance of the cloud it drew, and the history is
   reprojected by that point, not the span's midpoint.
6. **The march at reduced resolution.** The clouds pass splits in two. A march
   at `cloud_render_scale` of the view writes the history. A full-resolution
   composite upsamples it over the scene, and drops it wherever the pixel's own
   ray does not reach the cloud layer (a hill in front), so hill silhouettes
   stay sharp.
7. **Quality knobs in `weather.ron`**, validated and with units:
   `cloud_render_scale`, `cloud_step_m`, `cloud_max_steps`. Defaults are chosen
   from captures on this machine to hold 120 fps (8.3 ms p95) at the dearest
   height.

## Measured after

The same bench, the same machine, `cloud_render_scale: 0.4`,
`cloud_step_m: 12`, `cloud_max_steps: 48` (the shipped defaults), two runs
each, p50 / p95 ms:

| Camera height | Before | After |
| ---: | ---: | ---: |
| 2 m (in the rain) | 3.3 / 4.5 | 3.7 / 5.2 |
| 200 m (under the base) | 10.4 / 12.5 | 5.1 / 7.9 |
| 450 m (in the layer) | 29.6 / 54.5 | 4.4 / 5.0 |
| 650 m (in the layer) | **40.8 / 72.8** | **6.0 / 7.0** |
| 900 m (over the tops) | 19.6 / 36.3 | 3.0 / 3.7 |
| 2500 m | | 2.4 / 3.0 |
| Ground, day, fair weather (`--view surface --pitch 20 --time 11`) | | 3.3 / 4.1 |

Every view holds 120 fps at the 95th percentile at 1440 x 900. The steps
there, 650 m p50:

| Step | p50 |
| --- | ---: |
| Shipped | 40.8 |
| + cells baked, off the light march, march by length, scale 1.0 | 43.6 |
| ... at scale 0.5 | 10.9 |
| + light march on the coarse shape, 4 + 2 steps | 8.1 |
| ... at scale 0.4 | 6.0 |

At full resolution the march by length costs what the old one did: it takes
more, shorter steps where the cloud is near, and those are what remove the
seam and the fur. Resolution is the lever. Clouds cost scales with the
pixel count, so a 1920 x 1080 window is about 1.6 times the clouds' share of
these numbers. 0.35 is the next step down if a larger window needs it.

Two more defects turned up in the captures and are fixed here:

- **Cloud past the base's horizon was never marched.** From above the base,
  `cloud_span` ended every ray that dipped under the base sphere at that
  point. A ray that passes under the base and comes back up into the layer
  further out lost all the cloud there, which was cut off along the base's
  horizon as a hard curve. That is the owner's "planet curvature through
  opaque clouds", from above. The span now keeps the whole chord unless the
  ground ends the ray first, and the march steps over the gap.
- **The upsample stair-stepped along the limb.** The composite's depth-aware
  upsample also compares where each march texel stopped with where the pixel's
  own ray goes.

## Not in this change

- Any change to where cloud is or how it is lit (the atmosphere, the lighting
  model's terms, the cover remap).
- A settings-menu page for graphics quality. The knobs are data now, and a
  menu can expose them later.
- Neighbourhood clamping of the history (see `calm-clouds` design).
