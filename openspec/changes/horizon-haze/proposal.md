# Proposal: the sky behind distant cloud is lit air

## Why

Priority 4 in `CLAUDE.md`: "Distant cloud must fade into the sky's haze at
the horizon, taking the colour of the air between (the owner's reference
photos: an ocean horizon, cumulus from an airliner, a sunset over a deck), not
stand as a flat grey shell with a hard edge."

`cloud-reach` removed the hard edge (the owner's "ring"). The shell is still
there. From inside the cloud layer the far deck now fills the lower sky, and
it is a dark grey sheet (`docs/screenshots/cloud-reach-ring.png`, middle),
not a pale horizon. The handoff (`docs/handoff-2026-09-25.md`, open item 4)
names the cause: "the sky behind the clouds has little haze of its own, so a
faded cloud reveals dark blue rather than a bright horizon."

## What was measured

Stills, release build, RTX 3060 Laptop, 1440x900, `--time 10` unless noted
(`output/captures/p4/`, `output/captures/ring/`).

### The sky is space-dark by the top of the cloud layer

The sky (`sky_atmosphere.wgsl`, a shell to `ATMOSPHERE_RADIUS` = 1.2 R, 960 m
up) has air that thins with a normalised scale height of 0.22: **211 m**. Its
"day veil", which keeps the stars out of the surface sky, fades out between
0.3 and 1.0 of the shell: **288 m to 960 m**. The cloud layer is 300-750 m. So
the whole layer sits in the sky's own transition to space.

The sky just above the limb, sRGB, at the same column:

| Eye height | Just above the limb | Higher up |
| ---: | --- | --- |
| 2 m (ground) | (111, 162, 209) | (29, 55, 87) at the top |
| 421 m (in the layer, the owner's ring frame, clouds off) | **(40, 70, 104)** | (16, 33, 53) |
| 1500 m (over the tops) | (100, 150, 198), the limb's glow band below the eye | black |

From inside the layer, the sky just over the horizon is a third as bright as
from the ground. **Corrected below ("Measured again"): these two stills face
different ways under different suns; at one heading the horizon is as bright
from the layer as from the ground.**

### The haze assumes air five times deeper, and has no colour for a cloud

The ground, the water sheet and the clouds haze with another model: density
0.00036 per metre at the sea, falling off with a **1050 m** scale height
(`FOG_DENSITY_PER_M`, `FOG_HEIGHT_M`), five times the sky's. Three different
formulas use it:

| What | How it hazes | Toward |
| --- | --- | --- |
| Terrain (`planet_surface.wgsl`) | the eye's altitude only, `1 - exp(-d * density)`, at most 0.55 | `ground_sky`: day/night blue, greyed by cover |
| Water sheet (`distance_fog`) | the same, without cover | the same blue |
| Clouds (`cloud_air`) | optical depth along the path (Simpson, five points), times `cloud_haze` = 3 | **nothing: the cloud turns transparent** and the scene behind shows |

A cloud 3 km out from 421 m has an optical depth of 1.7-2.5, depending on
how far the ray dips (0.6-0.8 at a `cloud_haze` of 1), so 82-91% of it turns
transparent. Behind it is that dark
sky, so what is left is a grey film over dark blue. `cloud-close-up` chose to
fade clouds into what is behind them "without a second, disagreeing sky
colour". That holds only where the sky behind is the colour the air in front
would be. Inside and above the layer it is not.

### At dusk the haze stays blue

At dusk the sky shader tints itself orange-red near the horizon. The
terrain's `ground_sky` and the water's fog colour have no dusk term, so the
haze and the sky disagree exactly when the owner's "sunset over a deck" would
show it.

## What was tried (2026-09-26): a haze colour laid over the sky fails

The first plan (the write-up at `5b21118`) gave clouds and the sky one air
model: a density, a scale height and a flat colour, the terrain's haze blue
greyed by cover. Clouds took that colour in front of them rather than
turning transparent, and the sky laid it over its own scattering. The
owner decided: judge the haze strength from stills, and keep space at about
1 km. The air ended at the shell's top.

Built and captured at the same frames
(`docs/screenshots/horizon-haze-experiments.png`, columns: today, A, B):

- **Haze over the sky, with clouds taking its colour: worse everywhere.**
  From orbit a thick grey halo rings the planet. At 1500 m the limb's blue
  glow turns to a grey band. From the ground the whole sky goes a dull
  grey-blue. The flat colour is not the sky's own: it is the "second,
  disagreeing sky colour" `cloud-close-up` warned about.
- **A: clouds take the colour, the sky untouched: worse.** Far and limb
  clouds become flat grey patches: from inside the layer the far deck turns
  into an opaque grey sheet, and at 1500 m clouds near the limb go grey. A
  cloud fading into what is behind it, `cloud-close-up`'s rule, is right.
  What is behind it is the problem.
- **B: the sky's own air deeper inside its shell (normalised scale height
  0.5, not 0.22), clouds as today: better everywhere.** From inside the layer
  the far deck fades into a bluer sky. The horizon glows at 1500 m. From
  orbit the halo is thicker and blue, not grey. The shell and its taper are
  unchanged, so space still starts at 960 m, as the owner decided.

B, measured (sRGB, same pixels):

| View | Today | B |
| --- | --- | --- |
| 421 m, clouds off, just over the limb | (40, 70, 104) | (56, 93, 133) |
| 421 m, clouds off, higher | (19, 37, 58) | (40, 68, 101) |
| Ground, just over the horizon | (111, 162, 209) | (132, 181, 218) |
| Ground, near the zenith | (31, 58, 90) | (68, 111, 158) |
| 1500 m, the limb's glow | (100, 150, 198) | (128, 177, 214) |

The cost is the ground's zenith: with the same scattering, twice the air
overhead (0.410 against 0.215 in shell units) also pales the sky straight up
from the ground.

## Measured again: the horizon is not dark from the layer

The table under "The sky is space-dark by the top of the cloud layer"
compares stills that face different ways under different suns: the owner's
ring frame is kilometres along the cloud-hop route, and the ground still is
at the spawn. Held to one heading (`--view column --height H --pitch` at the
limb's dip, clouds off, `--time 10`, `output/captures/heights/`), the sky
over the limb is bright at every height:

| Eye height | 6 px over the limb | 80 px over | 200 px over |
| ---: | --- | --- | --- |
| 450 m | (120, 170, 216) | (84, 133, 184) | (51, 86, 129) |
| 600 m | (123, 174, 218) | (82, 131, 181) | (44, 77, 117) |
| 1200 m | (128, 177, 215) | (71, 115, 165) | (13, 30, 50) |

What darkens with height is the sky well above the horizon, which is the
day veil fading into space, and that is as the owner decided it should be.
The ring frame's sky is 2.5 times darker than this at the same height and
the same distance above the limb, so its darkness belongs to its place and
its heading, not to the layer's height.

And with the zenith held (the scattering divided by the extra air overhead,
design section 2), a deeper air barely changes the ring frame: over its limb
it is (41, 71, 106) at 0.35, 0.5 and 0.7 alike, against (40, 70, 104) today;
higher up it goes from (19, 37, 58) to (28, 50, 77) at 0.5. The limb's glow at
1500 m thickens, and so does the halo from orbit (the stills are in
`output/captures/depth/`). Experiment B looked better because it was a
brighter sky everywhere, the ground's included.

## Where this leaves priority 4

- The hard edge, which was what the owner reported, was the march's reach:
  `cloud-reach`, landed.
- The grey far deck in the ring frame is cloud lit and seen at that place's
  light, fading into that place's sky. Neither measured lever (a haze colour,
  a deeper air) fixes it without greying or brightening the whole sky.
- **Next**: the owner looks at the cloud-hop flight on the `cloud-reach`
  build. The deck is judged there, in motion and across the route's changing
  light, before anything else changes. If it still reads as a shell, the
  candidates are a brighter sky (B, without holding the zenith) and the
  clouds' own light at a low sun (`cloud_light`), which this change did not
  measure.

## The plan as it stood before the second measurement

## What changes (revised)

1. **The sky's air is deeper inside its shell.** Its normalised scale height
   goes from 0.22 (211 m) to about 0.5 (480 m), chosen from stills of 0.35,
   0.5 and 0.7. The shell (1.2 R, 960 m up) and its top taper are unchanged,
   so the air still ends there. From 421 m the air overhead is 4.6 times
   today's, and along the horizon more.
2. **The zenith from the ground holds.** The clear sky's Rayleigh and Mie
   scales (`CLEAR_RAYLEIGH`, `CLEAR_MIE`) come down by the ratio of the air
   overhead from the sea (1.9 at 0.5), so the ground's sky straight up stays
   today's blue. The horizon, and the sky seen from the cloud layer and above
   it, keep what the deeper air gives them.
3. **Clouds keep fading into what is behind them** (`cloud_air`,
   unchanged): the sky behind is now lit air.
4. `cloud_haze` is judged again on stills of 1, 2 and 3 once 1-2 land, as
   the owner asked.

## Decisions for the owner

- **B's look, and its depth.** Stills at 0.35, 0.5 and 0.7 with the zenith
  held, in the views above, before one is chosen. From orbit the halo
  thickens with it.

## Out of scope

- The terrain and water haze (1050 m scale height, eye-altitude form): they
  now differ from the sky's air in depth as well as form. That is the
  ground-to-space transition, priority 5.
- Lighting the clouds themselves (`cloud_light`).
- A dusk tint in the ground's haze.
