# Proposal: fishing anywhere there is water

## Why

The owner (2026-11-17), casting into a river: "also fishing not detecting
water in rivers", then "fishing should work anywhere thers water, just that
smaller bodiues have smaller fish".

Today the line and the fish know only the sea:
- **The float's water is the sea's surface.** `fish.rs`,
  `PlanetWater::surface` returns the sea radius plus the swell, everywhere.
  A river above the sea's level is not water to it, so a float cast into
  one falls through to the riverbed and nothing can bite.
- **Schools spawn where the generator says a river is, at the sea's
  radius.** `try_spawn` classes the water by `fauna::water_class` (the
  generator's river carve) and anchors the school at `direction *
  sea.radius`. For any river above sea level that is under the ground.
- **A pond, a lake or poured water is not water to fishing at all.**

## What changes

1. **The water fishing reads is the water the world holds.** The column
   tier's water under a point: a river, a lake, a pond or water a player
   poured, at its own top. The sea keeps its drawn sheet with its swell and
   tide.
2. **Schools spawn in that water**, at its surface. Fresh water standing
   above the sea is river water for the species that live there.
3. **Smaller water, smaller fish.** A school spawned in fresh water takes a
   size from the body of water it is in: how far the water reaches round it
   and how deep it is. Its fish are that much shorter, down to a third of
   their species' length in a puddle. The sea's fish are full size.

## Saves

None: schools are not saved, and a catch's length is saved as caught.
