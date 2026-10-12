# Proposal: a world map (step 2a of the cities plan)

## Accepted

**The owner (2026-09-28), on the in-game map's screenshots beside the
mockup: "map looks good".** The map is accepted on the screenshots, as
CLAUDE.md lets them stand in. The gate video stays open for the owner's batch
(task 6.1), and the sites on the map wait for `city-sites` (task 5.4).

## Why

**The owner (2026-09-27): "we'll first need to designate cities on a map ...
we need an in game map to show where fish spawn as well. And a live map. All
of these I expect a 2d map I can view. So map with cities first mocked up,
then I will approve."**

The game has no map of any kind: no map screen, no minimap, no compass. The
closest thing is the M key, which cycles weather overlays (wind, jet stream,
currents, cloud, rain, humidity, sunlight, temperature) painted onto the 3D
globe. Cities, the climate and the fish all need to be shown somewhere before
they can be designed, placed or approved. Everything a map would draw can
already be read on the CPU:
- the biome, as a pure function of direction;
- the live atmosphere and its cloud and wind maps;
- the clock's sun;
- the player's and the ship's latitude and longitude.

## What Changes

- **A map mockup first, for the owner's approval**, before any code: one HTML
  page showing the map with every layer the plan needs:
  - the base map, with the biomes at today's size and at the bigger sizes
    `bigger-biomes` proposes, side by side;
  - the city sites `city-sites` proposes;
  - the climate and fish layers `climate-and-fish-maps` proposes;
  - the live layer.

  The projection, the zoom levels and the controls are settled on the mockup.
- **A map screen in the game.** M opens and closes it. It is a flat 2D map of
  the whole planet, with pan and zoom from the whole globe down to a few cells
  a pixel.
- **The base map:** the planet as it looks, with each pixel the colour of its
  ground's top block, shaded by relief, and the towns drawn on it. It is drawn
  from the same generator functions as the terrain, so the map cannot
  disagree with the ground. Biomes and the other layers are overlays that
  grey the base map out while they are shown (the owner, survey M3).
- **The live layer:** the player, with their heading; the ship, and every
  parked vehicle; the night side of the terminator; and the clouds and rain,
  moving as the weather moves.
- **Layers the later changes plug into** (city sites, climate, fish), each
  toggled from a legend.
- **The globe overlays become map layers.** The eight weather overlays M
  cycles today are shown on the map. They can still be painted on the 3D
  globe from the map's legend, and M's old cycling goes away.

## Capabilities

### New Capabilities
- `player/map`: the map screen. It covers:
  - opening and closing it;
  - the projection;
  - pan and zoom;
  - the base map;
  - the live layer;
  - the layer legend;
  - the rule that the map is drawn from the same functions as the world.

### Modified Capabilities
- None. The weather overlays have no main-spec requirement, so moving them
  into the map changes no spec.

## Impact

- **`pbd-core`:** a `geo` module holding latitude and longitude of a direction
  and back, north and east at a point, and the map's projection. Today these
  are written inline wherever they are needed.
- **`pbd-app`:**
  - a `map` module that builds the base raster on a background task, caches it
    per seed and generator version, and holds the layer registry;
  - `bevy_ui` nodes for the screen, the legend and the markers;
  - `overlay.rs` becomes a map layer source;
  - M is added to `BINDINGS` in `controls.rs`.
- **Docs:** `docs/mockups/world-map.html` (the mockup, lit at night as
  CLAUDE.md requires of every mockup).
- **Performance:** the base raster is built once per world, off the frame. The
  live layer redraws a few markers and one cloud texture while the map is
  open, and nothing while it is closed.
- **No save change.**
