# Proposal: clouds white on top and dark underneath, by how thick they are

## Why

The owner: *"clouds should be white on top dark on bottom depending on
density."* The spec already says a mass is brighter on top than underneath
(`world/weather`, "Clouds are a lit slab with a thickness"). That is true, but
the numbers show why it does not read that way.

I measured the cloud march by transcribing it: `cloud_march` and `cloud_shadow`
from `shaders/clouds.wgsl`, run over a flat slab of uniform noise, with the
shipped `weather.ron` values (`cloudlight.py`, see design section 1).

| Slab, sun 60 deg | Top | Base | Base / top |
| --- | ---: | ---: | ---: |
| Fair (cover 0.2) | 0.44 | 0.29 | 0.66 |
| Half (cover 0.6) | 0.47 | 0.20 | 0.43 |
| Storm (cover 1.0) | 0.44 | 0.11 | 0.24 |
| Unshadowed sample, for reference | 0.82 | | |

Three things are wrong:

1. **The top is grey, not white.** Seen from above, a cloud's top comes out at
   **53%** of the light an unshadowed sample gets. `cloud_shadow` uses
   extinction **0.05 per metre**, written as a literal. That is 4.3 times the
   fair-weather extinction the view march uses (0.0115 in `weather.ron`) and
   1.7 times the storm one. Light is absorbed far faster on its way in from the
   sun than on its way out to the eye. One quantity has two numbers, and the
   one nobody can tune is the wrong one.
2. **Base darkness does not depend on density.** At that extinction, the shadow
   term falls from 1.0 at the lit top to **0.02 within 50 m** below it (fair
   cover). So nearly every sample under the top is fully shadowed, and its
   brightness is the knob `cloud_base_dark` times the sun. A 50 m wisp and a
   250 m tower have the same underside: whatever the knob says.
3. **Nothing lights the shadowed body but a floor.** The only ambient term is
   `cloud_night_floor` (0.045), flat. There is no sky light arriving from above
   and no bounce from the ground below. So a base cannot be dark because of the
   cloud over it; it can only be dark because of the knob.

The stateless field adds a fourth problem: the whole planet's clouds are
thresholded by the cover over the player (`sky.cloud_slab.y = cover`). That is
covered in `atmospheric-circulation`, which gives the clouds a cover per place.

## What

The cloud lighting is rebuilt on the standard real-time model (Schneider,
*The Real-Time Volumetric Cloudscapes of Horizon Zero Dawn*, 2015; Hillaire,
*Physically Based Sky, Atmosphere and Cloud Rendering in Frostbite*, 2016).
It stays inside the one `pbd::clouds` module, so the sky and the sea change
together.

- **One extinction.** The light march reads the view march's extinction. The
  0.05 literal goes.
- **A longer, finer light march.** Six samples toward the sun, each step longer
  than the last, instead of four steps of 83 m.
- **Multiple scattering, approximated.** Three octaves, each with less
  extinction and less scattering than the last. Light reaches deeper, so a thin
  cloud is white through and a thick one darkens gradually toward its base.
  This is what makes the base's darkness a function of density.
- **Silver lining.** A two-lobe phase function, forward and back, instead of
  none. Looking toward the sun, cloud edges glow.
- **Ambient from above and below.** Sky light enters from the top, dimmed by
  the cloud above the sample. Ground bounce enters from the bottom. Under a
  thick storm the sky light cannot reach the base, so the base goes slate.
  Under a thin cumulus it does, so the base stays grey.
- **The two darkness knobs go.** `cloud_base_dark` and `cloud_storm_dark` are
  removed; darkness now follows from density. New knobs, with units and
  validation in `weather.ron`: octaves and their three falloffs, the two phase
  lobes and their blend, sky and ground ambient.

## Out of scope

- Where clouds are and how they move. That is `atmospheric-circulation`.
- The dome's own scattering, unchanged since `overcast-and-rain`.

## Measured as

The transcription is the reference: it is run before and after, and the claims
are checked on its numbers and on captures.

- A fair-weather top at sun 60 deg reaches **at least 85%** of an unshadowed
  sample (now 53%).
- Base brightness **falls monotonically** with the column's optical depth: a
  thicker cloud always has a darker base.
- A storm base is **15% of its top or less** (now 24%). A fair cumulus base is
  **35-60%** (now 66%, from the knob alone).
- Cost: density samples per view step go from 5 to about 9. The frame time is
  measured on llvmpipe as an A/B, stated as not a GPU number.
