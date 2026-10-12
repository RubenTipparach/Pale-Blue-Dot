# Proposal: no ring at the cloud base's horizon

## Why

The owner (2026-09-26), of a frame a few hundred metres up on the cloud-hop
route: "There's this ring here at a certain altitude. I think it's cause by
clouds. Can u fix it". The frame shows a flat grey band across the sky just
over the planet's limb, with a sharp top edge that runs parallel to the limb
and clear sky above it. It is one case of priority 4 in `CLAUDE.md`: distant
cloud "not stand as a flat grey shell with a hard edge".

## What was measured

Stills from `pbd-app --route clouds --time 10 --capture <png> --frames 1300`:
the eye is 421 m over the ground, inside the cloud layer, which runs 300-750 m
over the sea. RTX 3060 Laptop, 1440x900, release build. Each variant was a
temporary change, restored afterwards (`output/captures/ring/`):

| Variant | The band |
| --- | --- |
| As shipped | a grey film from the limb up to a hard arc |
| `PBD_PASS_OFF=clouds` | gone: it is cloud |
| `cloud_haze: 0.0` | still there, brighter and opaque: the haze only dims it |
| grazing cap at 10 layer thicknesses, not 40 | barely changed: not the cap |
| `cloud_max_steps: 256`, not 48 | **the arc is gone**: the far deck runs on above it and fills the lower sky |

## What causes it

The march runs out of steps inside the layer. It steps by length: from
`2 x cloud_step_m` (24 m, where the near field ends), each step is 4% of its
distance from the eye, at least 12 m, and there are at most 48. Steps only
lengthen (to two steps' length) where the cover map is empty. Where the map
has cover, even between the clouds, 48 steps reach **about 800 m**. From
inside the layer, though, a ray runs 1.5-2 km horizontally before it leaves
through the top, and 3-4 km just above the base's horizon. Everything past
800 m is never sampled, so the sky past the near clouds shows clear.

A ray that dips under the cloud base does not pay for that stretch. The march
jumps the gap under the base in one step (`cloud-budget`) and spends its whole
budget on the deck beyond it, 2.5-4 km out. So below the base's horizon the
far deck is drawn, and just above it the same deck is not. The hard arc is
the cloud base's horizon, 8-21 degrees below the horizontal from 350-650 m.

## What changes

The march's last few steps stretch to reach the end of the span. Once the
48-step budget is spent short of the span's end, with the cloud still
translucent, a few more steps (`CLOUD_TAIL_STEPS`) divide what is left of the
span between them. The steps nearer the eye are not touched, so no cloud is
sampled differently from today. Only the stretch the march used to drop is
now drawn, at the coarse detail its long steps resolve. The far deck then
runs on continuously above the base's horizon, as it does with a 256-step
budget, for a fraction of that cost.

## Out of scope

- The colour of distant cloud: the far deck is still grey, not the colour of
  the air in front of it. That is the rest of priority 4 (the haze
  in-scatters nothing today).
- The step budget and the growth rule themselves.
