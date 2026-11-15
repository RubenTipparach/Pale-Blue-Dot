# lightning-fire: a strike lands on something, and sets it alight for a few seconds

## Why

The owner, 2026-11-14:
- "rainy stuff need like lightning would be nice to see lighning strike and
  srtart a tree or ground on fire";
- "fire would be localized because its raining and would go out in a few
  seconds";
- "seeing lightning strike would be cool".

Lightning exists today:
- The atmosphere makes strikes (`pbd_core::atmosphere::Strike`).
- `weather::strike_lightning` flickers the nearest one within 3 km for
  `lightning_flash_s` (0.7 s), lighting the clouds, the rain and the open
  ground.
- `push_bolt` draws a jagged ribbon from the cloud base down to the ground
  under the strike.

But the bolt ends nowhere in particular: it comes down at the centre of the
atmosphere's cell, on whatever the height field says is there. Nothing
happens where it lands, and a strike near the player is rare enough that it is
hard to see one at all.

## What Changes

- **A strike lands on the tallest thing near it.** Within 24 m of where the
  atmosphere put it, the bolt goes to the highest solid point in the column
  tier: a tree's crown if there is one, a rooftop, a hilltop, otherwise the
  ground. It never lands in water.
- **A burst at the point of impact:** a flash of light there (not only in the
  cloud), sparks and a puff of steam and smoke.
- **A short fire where it lands.** A tree's crown or the ground catches. In
  rain the fire is small and goes out in a few seconds: 3 to 6 s in heavy
  rain, up to 12 s in light rain. It lights what is near it, smokes, and
  steams as it goes out.
- **Nothing is saved or changed.** The tree is not burned away, no block is
  edited, and no scorch mark is stored. The fire is weather, like the rain.
- **Seeing one:** a strike is more likely to come down in sight while a
  storm is over the player. A `--strike-at LAT LON` instrument makes one at
  a known place for captures and videos.
- **Sound:** the crack, the thunder and the crackle are designed in the
  separate `audio-and-ambience` change and hooked to the events here.

## Capabilities

### Modified Capabilities
- `world/weather`: lightning strikes land on the tallest point near them,
  burst there, and start a short fire that the rain puts out.

## Impact

- `crates/pbd-app/src/weather.rs`:
  - where the bolt lands;
  - the impact burst;
  - a `Fires` resource of short-lived fires.
- A light for the fire and the impact flash, through the terrain's existing
  lightning term, extended from one light at the cloud to the ground point.
- `assets/shaders/planet_surface.wgsl` and the rain's lighting.
- `assets/config/weather.ron`: new tuning values, with units.
- No generation, save or terrain change.
