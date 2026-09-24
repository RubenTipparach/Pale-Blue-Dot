# Proposal: clouds that are smooth, and that drift rather than race

## Why

The owner, after the storm and cloud work landed on main: "the clouds look
pixelated, fix that, and they update TOOO FAST". Both are measured below on
the owner's desktop (i7-9700F, the release build, 1440 x 900).

### Pixelated

A native-resolution capture (`--view surface --pitch 20 --time 11`) shows every
cloud hatched in a regular diagonal halftone two pixels across. That pattern is
the clouds pass's per-pixel jitter (`water.wgsl`, `jitter` fed to
`cloud_march`): interleaved gradient noise, a pattern designed to be averaged
away by temporal anti-aliasing over several frames. This renderer has no
temporal accumulation, so the pattern is drawn as it is. It shows at full
strength because a ray takes only sixteen steps through the cloud layer, so
neighbouring pixels, offset by up to a whole step, sample the cloud at points
tens to hundreds of metres apart.

### Too fast

`atmosphere::tests::cloud_pace`, an instrument added for this change, reads the
shipped atmosphere after its spin-up:

| | Speed | Overhead at the 300 m cloud base |
| --- | ---: | ---: |
| The wind the GPU drifts cloud detail with (`upper`, the jet) | 33.9 m/s mean, 45.3 p90 | 6.5 deg/s, 8.7 p90 |
| The steering wind the simulation carries cloud with | 22.2 m/s mean | 4.2 deg/s |
| Surface wind | 1.3 m/s | |

A cloud crosses the whole sky in under half a minute. Cumulus at a real 1-2 km
base in a 10 m/s wind drifts at 0.3-0.6 deg/s: the world is small (the base
is 300 m up) and the winds are real, so what is seen is roughly ten times too
fast. The map of cover also churns: 13.5% of texels change by more than 0.05
in five seconds, 30% in thirty.

And the two winds disagree: the detail is drifted by the full upper wind
while the mass it sits on is carried by the steering blend, so the texture
slides across its own cloud at half as fast again.

## What

1. **The jitter becomes unstructured, and the march fine enough that it has
   little to hide.** The step count follows the length of the span through
   the layer, within a floor and a cap, so a ray's step stays short where the
   cloud is near; the per-pixel offset becomes a hash of the pixel (white
   noise) rather than interleaved gradient noise, so what error remains is a
   fine grain and never a pattern. Judged by native-resolution captures of the
   same view before and after, and priced in frame time with the capture's own
   `FRAME_WALL_MS`.
2. **A cloud pace.** `AtmosphereSettings::cloud_pace`, a fraction, validated,
   scales the steering wind that carries CLOUD in the simulation's `carry`
   stage, and only cloud: vapour, heat, charge and the wind itself move as they
   did, so the circulation, the storms' pressure and the rain physics are the
   same model. Cloud is made where air rises and now lingers there longer,
   which is what cumulus does.
3. **One wind for the cloud.** The wind map the GPU drifts detail with becomes
   the wind that actually carries the cloud: the steering blend times the
   pace. The texture then moves with its mass.

The pace's default is chosen from the instrument: a mean overhead drift near
0.8 deg/s, a cloud crossing the sky in minutes rather than seconds.

## What this is not

- No temporal anti-aliasing: that is the real cure for a noisy march and a
  much larger change to the render graph. The grain left after this change is
  its argument, if it is needed.
- The world clock, the day, the atmosphere's step and its physics other than
  cloud transport are unchanged.
