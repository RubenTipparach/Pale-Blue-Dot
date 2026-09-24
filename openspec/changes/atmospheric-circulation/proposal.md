# Proposal: an atmosphere that moves, on the planet's own cells

## Why

The owner:

> density across planet should not be so uniform, we're trying to model a
> realistic planet here so we should have some flow simulations. Air flow
> should kind of follow water current flows similar to how jet streams work.
> Where we have cells across the planet, and motion vectors that change and
> equalize. solar energy adds to the system in various ways driving the
> weather, and so does thunder and lightning

Today's weather is `pbd_core::weather`, a stateless field ported from
Tenebris: a drifting noise of "warm pockets" times a noise of "moisture". It
has no wind, no pressure, no heat, and nothing flows. I measured it over
40,000 directions and five times of day (a scratch instrument that calls
`cloud_cell`):

| What the field does | Measured |
| --- | --- |
| Cover by 15 deg latitude band, equator to pole | 0.18, 0.18, 0.16, 0.16, 0.21, 0.24 |
| Cover over ocean vs land | 0.19 vs 0.16 |
| Rain over ocean vs land | 5.9% vs 6.1% |
| How the pattern moves | One translation of the noise along a fixed body-frame axis. Up to **37 m/s**; **zero** at two opposite points, 15 deg N and 15 deg S; unrelated to the spin axis |

So there are no latitude bands: no equatorial rain belt, no desert belt, no
storm tracks. The ocean is not where the water comes from; it rains as often
over land as over sea. And nothing about the motion is wind. Wherever one fixed
vector happens to be tangent, the whole planet's weather slides along it, and it
stands still where that vector points straight up.

**The rendering flattens it further.** The cloud shader thresholds the whole
planet's noise with one number: the cover over the player
(`sky.cloud_slab.y = cover`). From orbit, the entire globe has whatever sky the
player happens to be standing under. `storm` recorded this as a held item.

## What

A small, deterministic **atmosphere simulation** in `pbd-core`, on the
planet's own hexagonal cells, replacing the stateless field as the one source
of cloud, rain, snow, wind and lightning.

- **Cells.** The Goldberg dual the terrain is built on, at a coarse level: 5
  (10,242 cells, 181 m apart) by default. It has explicit five- and
  six-neighbour tables and twelve pentagons. The topology moves from `pbd-app`
  into the core, so there is one implementation.
- **The sun drives it.**
  - Sunlight heats the surface by the real sun direction off the clock. Day
    and night sweep round with the 48-minute day.
  - Oceans store heat slowly and land quickly: sea breezes, and afternoon
    storms over land.
  - Clouds and snow reflect sunlight back out, which is the feedback that stops
    a runaway.
- **Air flows and equalizes.** Warm air makes low pressure, and air flows from
  high pressure to low. Pressure differences spread out and even out, with drag
  at the surface. It is a shallow-water model driven by heating: the
  Matsuno-Gill model, the standard teaching model of how heating drives the
  tropics.
- **The spin turns it.**
  - A Coriolis force turns the flow: storms spin anticlockwise in the north and
    clockwise in the south, with trade winds and westerlies.
  - A **jet stream** at cloud height comes out of thermal-wind balance: fast
    wind forms where temperature changes most steeply between equator and pole.
  - The clouds ride that wind, so the jet reads as streaming cloud.
- **Water cycles.**
  - Evaporation from the sea, and less from wet land.
  - The air carries its vapour. Where air converges and rises, the vapour
    condenses into cloud, and the latent heat released makes the air rise
    harder. That feedback is what grows a storm.
  - Cloud rains out past a threshold, as snow where the surface is below
    freezing. Snow then follows temperature, not the biome.
- **Lightning is part of the system.**
  - Strong rising, condensing air builds charge. Past a threshold, a cell
    strikes.
  - A strike marks the storm's collapse. The rain dumps, a cold, dense
    downdraft spreads out from under it, and its leading edge lifts the air
    around it, which can start the next storm.
  - The strike positions, and so the flash the player sees, come from the
    simulation, not from a clock hash.
- **Clouds per place.**
  - The simulation is resampled into a small weather map on the GPU: cover,
    cloud-top height, precipitation, and cloud-level wind.
  - The cloud march reads its cover and height where it is, not the player's.
    Storms tower, stratus lies flat, and from orbit the planet shows its bands
    and storm systems.
  - The cloud noise is carried along by the local wind, so the motion on screen
    is the simulated wind.
- **The weather slider** becomes a forcing at the player: it feeds heat and
  moisture into the cells round them until a storm forms there. It no longer
  saturates the whole planet at once.

## Decided by the owner

- **The weather is saved state: yes.** The requirement that weather is "a
  deterministic field, not a stored state" is removed. The step is
  deterministic (fixed step, fixed order, basic arithmetic only), and the state
  is saved with the world. A future server owns it and sends snapshots.
- **The spin the air feels is a tuning knob.** "No real physics here: the spin
  the atmosphere perceives can differ from the day/night spin." At the real spin,
  the Held-Hou estimate puts the Hadley cell's edge at 80-90 deg: one cell from
  equator to pole, like Venus, with no jets. `coriolis_scale` is set by
  measurement so the planet gets Earth's three cells a hemisphere and a jet.
  The day length is untouched.
- **Seasons: yes.** The planet orbits its sun. That is its own change,
  `orbit-and-seasons`: a 100-day year, with the sun's latitude swinging
  +/-23.45 deg. The simulation reads the sun from the clock, so seasons reach it
  without any code here.
- **Ocean currents: yes, in this change.** The ocean is a second fluid layer on
  the same cells, using the same operators. It is driven by wind stress, turned
  by the same Coriolis force, walled by the coasts, and it carries the sea's heat.
  Its surface current feeds the water shader's flow hook (`water-flow`), which
  has been zero everywhere until now.
- **Visual overlays** for wind, currents, cloud, rain, humidity, sunlight and
  temperature are their own change, `weather-overlays`.

## The picture to aim at

The owner's reference is an ISS photograph of a hurricane over the sea:
- a spiral of dense cloud tens of cells across, with an eye;
- feeder bands curling into it;
- ragged cumulus streets over open ocean;
- clear sea between systems.

Dense cloud is **opaque and white on top**. Earth's wind and current maps are
the reference for motion:
- counter-rotating storm spirals in the westerlies;
- a meandering western-boundary current shedding eddies.

This change is judged against those pictures, from orbit. `cloud-lighting`
owns the white tops.

## Out of scope

- Seasons, which are `orbit-and-seasons`.
- Overlays, which are `weather-overlays`.
- Thunder audio: there is no audio in the engine.
- Wind on grass, rain slant, and sailing. The surface wind is exposed from the
  core for those later consumers, but not wired.
- `cloud-lighting` owns how a cloud is lit; this change owns where it is.
