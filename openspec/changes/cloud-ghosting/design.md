# Design: the clouds' history never crosses a silhouette

## What the history does today

The cloud march runs at `cloud_render_scale` of the view. Each texel marches
one jittered ray and blends it into the previous frame's texel:
`history = mix(previous, current, CLOUD_BLEND)`, with `CLOUD_BLEND` at 0.06.
It finds `previous` by projecting this frame's cloud point with last frame's
`clip_from_local` and reading the history there **bilinearly**, whatever that
texel held (`water.wgsl`, `cloud_march_pass`). A texel with nothing to march
(terrain, the sea, a held tool or a craft in front) writes an empty history
(`calm-clouds`).

Keeping 94% a frame is a half-life of `ln 0.5 / ln 0.94` = **11.2 frames**,
0.19 s at 60 fps, and 38 frames (0.6 s) to get within 10%.

## Why it ghosts

Nothing checks that the previous texel looked at the same thing. A silhouette
is where that check fails, and it fails three ways:

1. **Revealed sky fades in (the trail).** When a near object moves off a patch
   of sky, the texels it covered last frame were written EMPTY. This frame
   reprojects the sky's cloud onto them and keeps 94% of empty, so the cloud
   comes back over 0.2 to 0.6 s. What is left is a cloudless print of the
   object trailing behind it. The empty-history rule was written to stop cloud
   ghosting over a hill. It stops that, and makes this instead.
2. **Cloud carried over a new occluder (the smear).** The empty rule fires only
   when a ray cannot reach the layer at all. If a hill or a craft moves in front
   of the far part of a cloud but the ray still reaches the layer, the march
   runs to the occluder. The history it reads came from a ray that reached
   further, so the cloud beyond the occluder is carried in front of it.
3. **The bilinear read straddles the edge.** A read at a silhouette averages
   an empty texel with a cloudy one. That half-strength value goes into a
   history that holds it for sixteen frames, so the edge picks up a soft dark
   band that follows the object.

The held tool and a boarded craft are the worst case: they are fixed on
screen, so **any** turn of the view moves every pixel of their silhouette over
new sky every frame. That is the ghosting "around objects in front of them".

The composite's depth-aware upsample (`cloud-budget`) already applies the right
test inside one frame: drop a texel whose cloud lies beyond this pixel's ground,
or whose march stopped short of where this pixel's ray goes. The history has
never had that test across frames.

## The fix: a history tap is kept only when it saw this cloud

**Keep last frame's distances.** The march already writes, per texel, how far
along the ray its cloud is and where the march stopped (`cloud_depth`, `Rg16Float`).
It becomes a pair that flips with the history. This frame writes one and reads
the other, as the colour does.

**Keep last frame's eye.** A new `WaterView` lane, `cloud_prev_eye`: those
distances were measured from last frame's eye, so this frame's point is measured
from there too before it is compared.

**Four taps, each tested.** The bilinear read becomes the four texels around
the reprojected point, weighted bilinearly. Each tap is **kept** only when both
hold:

- its march reached as far as this frame's does: last frame's stop `>=` the
  distance from last frame's eye to this frame's own stop point (nothing stood
  in front of it then: this rules out fault 1);
- its cloud is not beyond what this frame can see: last frame's cloud distance
  `<=` that same distance (nothing stands in front of it now: this rules out
  fault 2).

This is exactly the upsample's test, with this frame's stop measured from last
frame's eye.

The comparisons use the upsample's own tolerance (3% and 10 m), through one
function the upsample calls too, so the two tests cannot drift apart. The kept
taps are renormalised (fault 3: an empty tap on the far side of an edge no
longer weighs in). When no tap is kept, the texel **stands alone** this frame
and blends nothing.

A texel that stands alone is one jittered sample, so a revealed strip is
grainy for the few frames it takes to converge. That grain lasts the same time
the ghost did, but it is the right cloud from the first frame, which is the
priority's requirement: the history must never carry cloud across a silhouette.

## What it costs

- **Memory:** one more `Rg16Float` target at the march's size (4 bytes a
  texel; at the default scale 0.5 and 1440x900, 1.3 MB).
- **Bandwidth:** four `textureLoad`s of colour and four of distances replace
  one bilinear sample. That is against a march of dozens of noise samples a
  texel.

The frame cost is measured after, with `--frame-log` in a windowed run.

## What this does not do

- It does not clamp the history to the current frame's neighbourhood. A
  lightning flash still lags by the blend, and that stays the `calm-clouds`
  knob.
- It does not change `CLOUD_BLEND` or the jump rule (`CLOUD_HISTORY_JUMP_M`).

## How it is measured

`--turn DEG_PER_S` turns the walker's view at a steady rate during a capture.
It is a measurement instrument: it changes nothing unless it is given. Two
scenes on the build from before and the build from after, each compared with
the other:

- **still** (`--turn 0`): the fix must change nothing at rest. Two runs of one
  binary give the floor, and before-versus-after must sit on it;
- **turning** (`--turn 90`, the shovel held, sky in view): the ghost is the
  difference, and it must lie along silhouettes.
