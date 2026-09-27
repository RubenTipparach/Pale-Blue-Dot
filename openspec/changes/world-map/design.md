# Design: world map

## Context

See `proposal.md` for why. What the design has to work with, observed on
`main` (2026-09-27):

- **Nothing draws the world flat in the game.** The M key
  (`pbd-app/src/overlay.rs`, `cycle_overlay`) steps `OverlayMode` through the
  eight weather overlays, painted on the 3D globe by the water pass.
  `desktop/overlay_ui.rs` draws their legend from the ramp table the shader
  uses. M is not listed in `controls::BINDINGS`.
- **Flat maps exist outside the game.** `examples/fish_ranges.rs` writes a
  1440 × 720 equirectangular raster, built from `planet_gen::surface_altitude`
  and the atmosphere run forward. `tools/fish_ranges.py` draws field-guide
  range maps from that raster, and `docs/biome-atlas.html` shows the biomes.
  These are the precedent for a map built from the world's own functions.
- **The generator is a pure function of direction.**
  `surface_altitude(cfg, dir)` and `biome_at(cfg, dir, surface_m)` answer any
  point on the planet without the terrain being loaded. That is what lets a
  map show the far side.
- **The weather is on the CPU and on the GPU.**
  - `Air` holds the atmosphere on a level-5 grid of 10,242 cells, about
    181 m apart.
  - It also holds two cube maps of 6 × 64 × 64 texels, which the sky and the
    overlays sample.
- **Where the player is.** `WalkingReadout` and `FlightReadout` carry the
  latitude and longitude in degrees. Each is worked out inline from a body-local
  direction (`walking.rs:961`, `flight_view.rs:459`, `:628`) as
  `asin(y), atan2(z, x)`, with +Y the pole. The planet is pinned and the sky
  turns.
- **UI.** Bevy 0.18 `bevy_ui`, with `ImageNode` for pictures. `MenuOpen` says
  who holds the pointer.
- **The scale.**
  - The planet's radius is 4,800 m, so the equator is 30.2 km round.
  - A cell is 2.833 m across (CLAUDE.md), so one pixel a cell is 10,650
    pixels round the equator.

## Goals / Non-Goals

**Goals:**
- A mockup, built from the real world's rasters, that the owner approves
  before the game gets a map.
- A map screen that shows all of the planet and zooms to one cell a pixel.
- A live layer that is correct without being rebuilt: the players, the ships,
  the night and the weather.
- Room for the layers the later changes bring (sites, climate, fish), without
  those changes touching the map's code.

**Non-Goals:**
- **A minimap or a compass on the HUD.** They could come later from the same
  projection code, but the owner asked for a map to view.
- **Markers the player places, and waypoints.**
- **Fog of war or an explored-area mask.** The whole planet is drawn from the
  start. Whether exploring should reveal it is a question for the owner at the
  mockup, not a default.
- **Maps of other bodies.** The map draws the body the player is on or
  orbiting. Other bodies wait until more than one body has terrain worth
  mapping.
- **Buildings drawn on the map.** A settlement shows as a marker and its name,
  not its streets. Street plans belong to `cities-in-the-world`, if the owner
  asks for them.

## Decisions

**1. The mockup is drawn from rasters the game's own functions made.**
- A measurement instrument, `examples/world_map.rs` in `pbd-core`, writes:
  - an equirectangular raster of the altitude and the biome, for today's
    generator and for each biome scale `bigger-biomes` proposes;
  - the climate fields, the same way `fish_ranges` does.
- `docs/mockups/world-map.html` loads those rasters as PNGs and draws every
  layer from them, so the owner judges the real planet, not a painting.
- The instrument's per-pixel function is `pbd_core::map::base_texel`, the same
  function the game's map calls later. So there is one source for what the map
  shows.
- *Alternative:* a hand-drawn mockup. Rejected, because it would show a planet
  the game does not have, and approval of it would not carry over.

**2. The projection is equirectangular for the whole planet, and azimuthal
close in.** *Overruled by the owner (M2): flat and equirectangular at every
zoom, like the fish maps. See "Decided by the owner" below.*
- Zoomed out, the map is equirectangular:
  - latitude and longitude are straight lines;
  - it matches `fish_ranges` and the atmosphere reports;
  - it is one texture.
- Zoomed in past a threshold, it changes to an azimuthal equidistant
  projection centred on the view. Close to the player it is then true to
  shape and distance at every latitude, poles included. That is what "the
  poles readable" needs, since the equirectangular map stretches a polar cell
  without limit.
- The change of projection is a blend across one zoom step, not a cut.
- The threshold is set on the mockup.
- *Alternative:* an icosahedral net (a Dymaxion map), which matches the
  world's own topology. Rejected for the main view: it cuts the land along 20
  triangle seams and is hard to read. It can go in the mockup as a comparison
  if the owner wants it.

**3. The base map is tiled, built off the frame, and cached on disk.**
- The whole planet is one 2,048 × 1,024 raster, about 14.7 m a pixel at the
  equator (five cells). It is built on the async pool the first time a world
  is opened. It is cached in the save's folder, keyed by the seed and the
  generator version, and deleted when either changes.
- Closer zooms are drawn from 256 × 256 tiles in a quadtree. They are built on
  demand from the same `base_texel` and kept in an in-memory LRU. The finest
  level is 2.833 m a pixel.
- The build cost is measured (task 1.1, 2026-09-27): `base_texel` over the
  whole 2,048 x 1,024 raster took 2.6 s on one thread of the cloud box, a
  release build, 1.24 us a texel. On the async pool that is well under a
  second on four threads, and it runs once a world, before the cache.
- *Alternative:* one full-resolution raster, 10,650 × 5,325 pixels. Rejected:
  it is 57 million texels, and most of them are never looked at.

**4. One UI material draws the map, and the live layers are lookups in it.**
- A `UiMaterial` shader draws the map node from:
  - the base raster (or its tiles);
  - the sun's direction, from `daylight::Clock`;
  - the weather cube maps `Air` already uploads.
- For each pixel it inverts the projection to a body-local direction. From
  that direction it shades:
  - night, with `dot(direction, sun)`, with the same twilight band the sky
    uses;
  - clouds and rain, from the cube maps;
  - the active weather overlay, from the ramp table the globe's overlay uses.
- None of these layers is rebuilt while the map is open; they move because
  their inputs move.
- Markers are `bevy_ui` nodes placed by the same projection on the CPU. They
  cover the player and heading, the owned vehicles, and later the sites.
- *Alternative:* re-rasterise the night and the clouds on the CPU each
  second. Rejected: it duplicates what the GPU already holds and would lag.

**5. `pbd_core::geo` owns latitude and longitude.**
- `lat_lon(dir)`, `direction(lat, lon)`, `north_east(dir)`, and the two
  projections and their inverses, all tested in the core.
- The four inline copies in `walking.rs` and `flight_view.rs` call it.
- The map, the site placer (`city-sites`) and the readouts then agree by
  construction.
- The WGSL inverse projection is checked against the Rust one on the same
  sample points, as CLAUDE.md asks of Rust and WGSL that must agree.

**6. M opens the map. The overlays are chosen in the map's legend.**
- M toggles the map, and Escape closes it. It is added to `BINDINGS` and so to
  the controls list.
- The map takes the pointer through `MenuOpen`:
  - the wheel zooms about the pointer;
  - a drag pans;
  - Home recentres on the player.
- The world keeps running. On foot the player stands still. In a ship the
  controls read as released, so the dampeners hold it, as they do when a menu
  is open now.
- The legend lists each layer with a switch. The weather overlays are one
  group of radio buttons, including off.
- A "show on globe" switch sets `OverlayMode`, so the globe overlay still
  works, chosen from the legend instead of by cycling M.
- *Alternative:* keep M cycling the overlays and put the map on another key.
  Rejected: M is the key a player tries first for a map, and cycling eight
  overlays blind is the weaker of the two uses.

**7. Layers are registered, not hard-coded.**
- A layer is one of two kinds:
  - a raster, either built by a `fn(direction) -> colour` on the pool or
    sampled in the shader;
  - a set of markers.
- Each layer has a legend entry and a default visibility.
- `city-sites` and `climate-and-fish-maps` add layers through that registry,
  and the map's code does not change for them.

**8. The mockup shows the eight weather overlays too (the owner, 2026-09-27,
on the published mockup: "what about all the other overlays like cloud
cover, solar, wind, currents etc").** Decision 6 already makes them map
layers in the game (task 5.2); the mockup showed only the live clouds, so the
owner had nothing to judge them by. They go in as the game draws them, from
the game's own definitions:
- `examples/map_weather.rs` writes each overlay's value with
  `Overlay::texel`, the one reading the globe's overlay uses, over a day in
  twelve frames, and writes each overlay's name, unit, range, ramp and
  whether it flows or fades from `pbd_core::overlay`, so nothing about an
  overlay is restated.
- `tools/world_map.py` colours each frame with the ramp table the game's
  legend and shader share (`pbd_app::overlay::RAMPS`), parsed from its
  source as the shader's copy is checked against it. Cloud and rain fade
  toward nothing as the globe's do.
- The page groups them apart from the year's climate: the weather is at the
  hour on the world clock, and moves with it. Wind, the jet and the
  currents draw moving streaks along the flow, where the globe draws
  streamlines, at a speed that reads on screen rather than to scale.
- They load when first chosen, so the page opens as fast as it did.

**9. Smooth at every zoom, and more detail as you zoom in (the owner, survey
M5, 2026-09-27: "id also like it to have linear interpolation, instead of
blocky pixels, canwe have a more detailed layer whenI zoom in?").**
- Every layer is drawn with linear filtering at every zoom. The mockup kept
  pixels square when zoomed in, as pixel art; the owner asked for smooth. The
  terrain's own nearest-point rule (CLAUDE.md) is for the ground's textures,
  not the map.
- The base map gains two finer levels, drawn by the same `base_texel` and
  the same colouring: 4,096 across (7.4 m a pixel) and 8,192 across (3.7 m,
  about a cell a pixel), cut into 512-pixel tiles. The page draws the finest
  level whose pixels are no bigger than the screen's, loading only the
  tiles in view, over the coarser level while they load. This is decision
  3's quadtree, shown in the mockup at the depth the game will have.
- *Alternative:* one bigger image. 8,192 across is 33 million pixels, too
  much to load at once for a page that shows a few tiles of it.

## Risks / Trade-offs

- [The base raster takes too long to build] → Timed by the instrument
  (task 1.1): 2.6 s on one thread for the whole planet. It runs off the frame
  and is cached. A close-zoom 256 x 256 tile is 65,536 texels, about 80 ms on
  one thread.
- [Two projections confuse the player] → The blend is shown in the mockup, and
  the owner decides whether it stays. The fallback is equirectangular only,
  with the poles drawn in two small azimuthal insets.
- [The cache outlives a generator change] → The cache is keyed by generator
  version, and `bigger-biomes` makes that version a real part of the world's
  identity. A stale cache is deleted, never shown.
- [The UI material and the cube maps in `bevy_ui` on Bevy 0.18] → A spike in
  task 3.1 draws the night side alone before anything else is built on it.
- [Frame cost while the map is open] → One full-screen quad and a few dozen
  markers. It cannot be measured in a cloud session (CLAUDE.md). The owner
  runs `perf_suite.py` on real hardware.

## Migration Plan

- There is no save change. The raster cache is derived and can be deleted at
  any time.
- M's cycling goes, and the controls list gains M, "the map".
- Rollback is the previous build.

## Decided by the owner (survey, 2026-09-27)

- **M1, "all is known":** the whole planet is drawn from the start. No reveal
  by exploring.
- **M2, "the way you did the fish map is good?":** the map is flat and
  equirectangular at every zoom, like the fish range maps. Decision 2's
  azimuthal close-in view is dropped. The poles stretch, as they do on the fish
  maps. If that reads badly on the mockup, two small pole insets are the
  fallback, and the owner is asked again then.
- **M3, "normal planet terrain like what it looks like on the world, and the
  towns. biomes and stuff are different overlays (which grey out the
  basemap)":** the base map is the planet as it looks: each pixel takes the
  colour of the ground's top block as the terrain draws it, shaded by relief,
  with the sea by depth and the towns drawn on it. Biomes, climate, fish and
  weather are overlays. While one is shown, the base map is greyed out beneath
  it.
- **M4, "need to add buttons for these instead of m to cycle. <M opens and
  closes maps":** M opens and closes the map. Each overlay has its own button
  in the legend, and M no longer cycles overlays.
- **On the published mockup (chat, 2026-09-27), "what about all the other
  overlays like cloud cover, solar, wind, currents etc":** the eight weather
  overlays join the mockup (decision 8).
- **The same day, "nigth side, clouds and rain should be off e dfault":** the
  map opens with the night side and the live clouds and rain off, each a
  toggle in the legend. It opens at the spawn's noon. The towns still light
  up on the night side once it is turned on, which is how the mockup keeps
  CLAUDE.md's rule that mockups are lit at night.
- **M5, "id also like it to have linear interpolation, instead of blocky
  pixels, canwe have a more detailed layer whenI zoom in?":** decision 9,
  built in the mockup as task 1.3c.
- **M6, the gate: "good approve".** The owner approves the map on the
  mockup's version 3. Under CLAUDE.md's standing screenshot rule the
  walkthrough video (task 1.3a) waits for the owner's batch, and the code
  below starts.
