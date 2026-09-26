# Proposal: clouds take the colour of the air, and the sky has a horizon

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
from the ground.

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

## What changes

1. **One air.** A single function in one shared WGSL module gives the air
   between the eye and a point: its optical depth along the path (the
   clouds' five-point Simpson rule, one density and one scale height), and
   its colour. The colour is the day/night haze blue by the sun's elevation,
   greyed by cover, and tinted at dusk with the sky's own tint.
2. **Clouds take the air's colour.** A cloud's light is dimmed by the air in
   front of it, and that air adds its own colour over the cloud's coverage:
   `scene * (1 - a) + T * cloud + a * (1 - T) * air`. At present the whole
   cloud is scaled by `T`, so it turns transparent instead of hazy. A far
   deck then reads as pale haze, not as dark sky through a grey film.
3. **The sky has a horizon.** The sky shader lays the same air over its own
   colour along its ray, out to the shell or the ground. Where a ray runs
   through deep air (low down, near the horizon), the sky becomes the haze
   colour. So a hazed cloud or ridge meets a sky of its own colour, from
   the ground, from inside the layer and from above it.
4. **Terrain and water take the same function**, in place of their
   eye-altitude-only formula. The ground, the sheet, the clouds and the sky
   then haze alike at the same distance. The terrain's literals move into
   the uniform as this does, which is task 2 of
   `preview-scale-and-shader-parity`.

## Decisions for the owner

- **Haze strength.** With the air adding its colour, a hazed cloud goes pale
  instead of transparent. The reason for `cloud_haze` 3 (at 1 the deck stood
  as a grey band) may then be gone, and 1 would haze clouds exactly like the
  ground. The handoff says to ask before changing it; stills at 1, 2 and 3
  come with the change.
- **How high the air reaches.** With the haze's 1050 m scale height, the
  horizon still glows 2-3 km up. Today the sky is space-dark by about 1 km.
  That moves the ground-to-space transition, which is priority 5, so it is
  the owner's call. It can be held where it is by thinning the air in the
  sky pass above the shell, at the cost of the horizon glow over the tops.

## Out of scope

- The sky's single-scattering model and its constants.
- Lighting the clouds themselves (`cloud_light`).
- Night: the haze colour at night is today's `FOG_NIGHT_SKY`.
