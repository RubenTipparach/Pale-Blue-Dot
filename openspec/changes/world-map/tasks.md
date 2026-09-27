# Tasks

Step 2a of the owner's plan. It starts after the owner approves the lights
(`lamps-and-lanterns`, group 7). Group 1 is the mockup, and it is the owner's
gate for everything after it, in this change and in `bigger-biomes`,
`city-sites` and `climate-and-fish-maps`.

## 1. The map mockup, for the owner's approval

- [ ] 1.1 `pbd_core::map::base_texel(cfg, direction)` (altitude, top block, biome, land or sea; the base map is coloured by the top block, survey M3) and the instrument `examples/world_map.rs`. It writes a 2,048 × 1,024 equirectangular raster of the shipped generator, and one for each biome scale `bigger-biomes` proposes, and prints how long each took. Verify: a core test that `base_texel` and `biome_at(surface_altitude(...))` agree on 10,000 seeded directions, and the build time is recorded in this design's risk note.
- [ ] 1.2 The rasters exported as PNGs into `docs/mockups/world-map/`, with a small script next to `tools/fish_ranges.py`. Verify: the PNGs are committed, and the script regenerates them byte for byte.
- [ ] 1.3 `docs/mockups/world-map.html`. It has:
  - the planet flat and zoomable;
  - the flat, equirectangular projection at every zoom, as the fish range maps have (survey M2);
  - the base map as the planet looks, with its towns, and each overlay greying it out (survey M3);
  - a button per overlay in the legend, and M to open and close (survey M4);
  - the biome layer, today's and the bigger-biomes proposals, switchable side by side;
  - placeholder site markers where `city-sites` would put them, drawn from its rules run in the page;
  - a climate layer and a fish layer from the `fish_ranges` fields;
  - the live layer with a player, a parked ship, a moving night side and clouds;
  - a day/night toggle, with the lit settlements glowing on the night side (CLAUDE.md: mockups are lit at night).

  Verify: the page loads with no console errors in a headless browser, and a screenshot of each layer is checked in.
- [ ] 1.3a A walkthrough video of the mockup (`step-videos`' `mockup_video.js`): the whole planet, zooming to the player, each layer in turn, the biome scales side by side, the sites, and the night side with the towns' lights. Verify: it is on the gate page.
- [ ] 1.4 Publish the mockup as an Artifact and link it from the roadmap page and the PR. Verify: the link opens.
- [ ] 1.5 **Gate:** the owner watches the video, tries the mockup, and approves the map. Record their words and their answers to the open questions in `design.md`, and change the decisions they overrule before any code below starts. Verify: the quote is in `design.md`.

## 2. Latitude and longitude in one place

- [ ] 2.1 `pbd_core::geo`: `lat_lon`, `direction`, `north_east`, and the equirectangular projection with its inverse (survey M2: flat at every zoom). Verify: core round-trip tests at the poles, on the antimeridian and at the equator, and on an offset planet (CLAUDE.md).
- [ ] 2.2 The inline copies in `walking.rs` and `flight_view.rs` call `geo`. Verify: the readout tests are unchanged, and `git grep "atan2(direction.x)\|atan2(up.x)"` finds nothing outside `geo`.

## 3. The map screen

- [ ] 3.1 Spike: a `UiMaterial` that draws the night side from the sun's direction on an equirectangular quad. Verify: a `--capture` at a pinned `--time` whose terminator crosses the equator at the longitudes the clock gives.
- [ ] 3.2 The base raster built on the async pool, cached in the save folder under the seed and generator version, and loaded on the next start. Verify: an app test that a changed generator version discards the cache, and that a second start does not rebuild.
- [ ] 3.3 The quadtree of tiles and the LRU for close zoom. Verify: a test that the tile under the player at the finest zoom is 2.833 m a pixel, give or take the geodesic spread CLAUDE.md gives.
- [ ] 3.4 The shader's inverse projection, checked against `geo` on the same sample points. Verify: the check is in the shader-constant test's family, and it fails when either copy is changed.
- [ ] 3.5 M, Escape, the wheel, dragging and Home. `MenuOpen` holds the pointer, and the world keeps running. Verify: app tests that M opens and closes, that walking input is ignored while the map is open, and that the clock advances while it is open.

## 4. The live layer

- [ ] 4.1 The player's marker and heading, and a marker for each owned vehicle, placed with `geo`. Verify: an app test that a parked ship's marker stays put while the player walks away.
- [ ] 4.2 Clouds and rain sampled from `Air`'s cube maps in the map's shader. Verify: a capture of the map beside the globe's cloud overlay at the same time, in `docs/screenshots/world-map/`.

## 5. Layers and the legend

- [ ] 5.1 The layer registry (raster or markers, legend entry, default visibility). Verify: a test layer added from outside the `map` module appears in the legend.
- [ ] 5.2 The eight weather overlays as one radio group in the legend, with "show on globe" setting `OverlayMode`. M's cycling is removed. Verify: `the_legend_and_the_shader_share_one_ramp_table` still passes, and an app test that the legend's switch sets `OverlayMode`.
- [ ] 5.3 M added to `BINDINGS`. Verify: the controls-list test finds it.

- [ ] 5.4 A site's named places on the map, as the towns mockup's Places list has them (the inn, the keep, the market). Choosing one shows it on the map, and in a debug build moves the player there. Verify: an app test that choosing a place in a debug build puts the walker at its door.

## 6. The owner's check

- [ ] 6.1 The gate video (showcase `map`):
  - M opened on foot;
  - a zoom from the whole planet down to the player;
  - a walk away from a parked ship with its marker staying put;
  - the night side moving through dusk;
  - clouds and rain beside the globe's overlay;
  - the legend's layers;
  - a pole.

  It is published with stills beside the approved mockup, and a note that frame cost was not measured in the cloud session. Verify: the page is linked from the PR.
- [ ] 6.2 The owner watches the video and accepts the map. Verify: the quote is in `proposal.md`. Then sync `player/map` into `openspec/specs` with each requirement's test named, and archive.
