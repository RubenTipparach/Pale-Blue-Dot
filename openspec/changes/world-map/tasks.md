# Tasks

Step 2a of the owner's plan. It starts after the owner approves the lights
(`lamps-and-lanterns`, group 7). Group 1 is the mockup, and it is the owner's
gate for everything after it, in this change and in `bigger-biomes`,
`city-sites` and `climate-and-fish-maps`.

## 1. The map mockup, for the owner's approval

- [x] 1.1 (2.6 s for the whole raster on one thread; the test compares with the floored altitude the column and the terrain's surface code use, not `surface_altitude` raw, which differs at a threshold.) `pbd_core::map::base_texel(cfg, direction)` (altitude, top block, biome, land or sea; the base map is coloured by the top block, survey M3) and the instrument `examples/world_map.rs`. It writes a 2,048 × 1,024 equirectangular raster of the shipped generator, and one for each biome scale `bigger-biomes` proposes, and prints how long each took. Verify: a core test that `base_texel` and `biome_at(surface_altitude(...))` agree on 10,000 seeded directions, and the build time is recorded in this design's risk note.
- [x] 1.2 The rasters exported as PNGs into `docs/mockups/world-map/`, with a small script next to `tools/fish_ranges.py` (`tools/world_map.py`). Verify: the PNGs are committed, and the script regenerates them byte for byte.
- [x] 1.3 (Layers from the level-5, 200-day balanced run and a day of `map_weather` frames; the only console error is the Google Fonts request the sandbox cannot reach. An overlay hides the live clouds and fades the night side, since a year's mean has no hour. Screenshots in `docs/screenshots/world-map/`.) `docs/mockups/world-map.html`. It has:
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
- [x] 1.3b (`map_weather` writes `PBDWTHR2`; `tools/world_map.py` reads the ramps from `pbd_app::overlay::RAMPS` and the fade from the shader; the currents leave land clear. The night side and live clouds are off by default, as the owner asked.) The eight weather overlays in the mockup (decision 8): wind, jet, currents, cloud, rain, humidity, sunlight and temperature, through a day, in the game's ramps, the flows as moving streaks. Verify: a screenshot of each in `docs/screenshots/world-map/`, and the mockup republished.
- [x] 1.3c (Every layer is drawn filtered; `tools/world_map.py tiles` cuts the 4096- and 8192-wide levels, 7.4 m and 3.7 m a pixel, into 512-pixel tiles in the base palette, loaded for the view. `close.jpg` is before, `close-smooth.jpg` and `closest-detail.jpg` after; the stripes on sand are the ground's own: the desert's rock is a latitude-only dither in `top_material`, measured on the game's map at 0.7 m a pixel, and asked of the owner (survey G1). Republished as version 3 of the mockup.) Linear filtering at every zoom, and the base map's finer levels as tiles loaded in view (decision 9, survey M5). Verify: screenshots at the finest zoom before and after, and the mockup republished; then the gate is asked again (M5).
- [ ] 1.3a A walkthrough video of the mockup (`step-videos`' `mockup_video.js`): the whole planet, zooming to the player, each layer in turn, the biome scales side by side, the sites, and the night side with the towns' lights. Verify: it is on the gate page.
- [x] 1.4 (https://claude.ai/artifact/G7QY9BYXYMH8tgaEv8E7az, linked from the roadmap page and the PR.) Publish the mockup as an Artifact and link it from the roadmap page and the PR. Verify: the link opens.
- [x] 1.5 (Approved on the mockup, survey M6: "good approve". The video, 1.3a, waits for the owner's batch.) **Gate:** the owner watches the video, tries the mockup, and approves the map. Record their words and their answers to the open questions in `design.md`, and change the decisions they overrule before any code below starts. Verify: the quote is in `design.md`.

## 2. Latitude and longitude in one place

- [x] 2.1 (`crates/pbd-core/src/geo.rs`, four tests. Latitude is `atan2` of the height over the distance from the axis, since `asin` of an `f32` loses the last hundredth of a degree by the poles; `examples/world_map.rs` projects through it too.) `pbd_core::geo`: `lat_lon`, `direction`, `north_east`, and the equirectangular projection with its inverse (survey M2: flat at every zoom). Verify: core round-trip tests at the poles, on the antimeridian and at the equator, and on an offset planet (CLAUDE.md).
- [x] 2.2 (The readout tests pass unchanged; the grep finds nothing.) The inline copies in `walking.rs` and `flight_view.rs` call `geo`. Verify: the readout tests are unchanged, and `git grep "atan2(direction.x)\|atan2(up.x)"` finds nothing outside `geo`.

## 3. The map screen

- [x] 3.1 (`map_live.wgsl`, one `UiMaterial` over the base. At `--time 12` on day 0 the night crosses the equator at -60.6 and 116.2 degrees east in the capture, against -59.5 and 116.1 from `Clock::daylight` (`examples/terminator.rs`); a pixel is 0.25 degrees there.) Spike: a `UiMaterial` that draws the night side from the sun's direction on an equirectangular quad. Verify: a `--capture` at a pinned `--time` whose terminator crosses the equator at the longitudes the clock gives.
- [x] 3.2 (`world_map::{build_base, save_base, load_base}`: texels, not colours, as a PNG named by the seed and `GENERATOR_VERSION` (decision 10); `the_cache_is_keyed_by_seed_and_generator` reads a written cache back, reads nothing for another seed, and deletes and ignores another generator's. Built on the pool in play and in place in a capture.) The base raster built on the async pool, cached in the save folder under the seed and generator version, and loaded on the next start. Verify: an app test that a changed generator version discards the cache, and that a second start does not rebuild.
- [x] 3.3 (Levels 2,664 / 5,328 / 10,656 of 333-pixel tiles, decision 10. `the_finest_level_is_a_cell_a_pixel`: 2.830 m against 2.833 m; `tiles_meet_without_a_seam`, the antimeridian included; `the_lru_drops_the_least_recently_drawn`.) The quadtree of tiles and the LRU for close zoom. Verify: a test that the tile under the player at the finest zoom is 2.833 m a pixel, give or take the geodesic spread CLAUDE.md gives.
- [x] 3.4 (`map_gpu_tests.rs` runs the shader's `unproject` and `daylight` on a Vulkan adapter against `geo::unproject` and `Clock::daylight` at 320 points; it fails when the shader's longitude or its dusk band is changed, both tried.) The shader's inverse projection, checked against `geo` on the same sample points. Verify: the check is in the shader-constant test's family, and it fails when either copy is changed.
- [x] 3.5 (`m_opens_and_closes_the_map_and_the_world_runs_behind_it`: M opens it centred on the player and closes it, `MenuOpen` is held while it is open, which is what the walker and the pilot take no input by, and the clock runs.) M, Escape, the wheel, dragging and Home. `MenuOpen` holds the pointer, and the world keeps running. Verify: app tests that M opens and closes, that walking input is ignored while the map is open, and that the clock advances while it is open.

## 4. The live layer

- [x] 4.1 (`a_parked_ships_marker_stays_put_while_the_player_walks_away`. The player is the walker on foot and the ship at the controls, read off their positions rather than the readouts, which start at zero.) The player's marker and heading, and a marker for each owned vehicle, placed with `geo`. Verify: an app test that a parked ship's marker stays put while the player walks away.
- [x] 4.2 (The map's own cloud cube, filled from `Air`'s weather maps and sampled in `map_live.wgsl`; `game-cloud-beside-globe.jpg`. Found on the way: the map's "Show on globe" sync wrote over the launch's `--overlay` on its first frame; it now writes only when its own switch or choice changes, `the_map_leaves_the_globes_overlay_alone_until_asked`.) Clouds and rain sampled from `Air`'s cube maps in the map's shader. Verify: a capture of the map beside the globe's cloud overlay at the same time, in `docs/screenshots/world-map/`.

## 5. Layers and the legend

- [x] 5.1 (`world_map::MapLayers` of `RasterLayer`s painted from the base's texels; the biomes are the first, added through it. `a_registered_layer_appears_in_the_legend`.) The layer registry (raster or markers, legend entry, default visibility). Verify: a test layer added from outside the `map` module appears in the legend.
- [x] 5.2 (The map colours an overlay from `overlay_texels` with `overlay_rgba`, whose fade `the_map_and_the_globe_fade_an_overlay_alike` pins to the shader's. `the_legend_sets_the_globes_overlay`; `cycle_overlay` is gone.) The eight weather overlays as one radio group in the legend, with "show on globe" setting `OverlayMode`. M's cycling is removed. Verify: `the_legend_and_the_shader_share_one_ramp_table` still passes, and an app test that the legend's switch sets `OverlayMode`.
- [x] 5.3 (In the WORLD group, "the map"; `map_screen.rs` is one of the files the test reads.) M added to `BINDINGS`. Verify: the controls-list test finds it.

- [ ] 5.4 (Waits for `city-sites`: the game has no sites, so no places to list. Decision 10.) A site's named places on the map, as the towns mockup's Places list has them (the inn, the keep, the market). Choosing one shows it on the map, and in a debug build moves the player there. Verify: an app test that choosing a place in a debug build puts the walker at its door.

## 6. The owner's check

- [ ] 6.1 (Screenshots stand in until the owner's batch: the gate page https://claude.ai/artifact/2Vs2e3h3rQS1EPKQSKA1gF puts each game view beside the approved mockup's.) The gate video (showcase `map`):
  - M opened on foot;
  - a zoom from the whole planet down to the player;
  - a walk away from a parked ship with its marker staying put;
  - the night side moving through dusk;
  - clouds and rain beside the globe's overlay;
  - the legend's layers;
  - a pole.

  It is published with stills beside the approved mockup, and a note that frame cost was not measured in the cloud session. Verify: the page is linked from the PR.
- [ ] 6.2 (Accepted on the screenshots, the owner 2026-09-28: "map looks good", quoted in `proposal.md`. Open for the video in the batch; the sync and the archive follow it and 5.4.) The owner watches the video and accepts the map. Verify: the quote is in `proposal.md`. Then sync `player/map` into `openspec/specs` with each requirement's test named, and archive.
