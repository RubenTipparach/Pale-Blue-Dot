# Design: bigger biomes

## Context

See `proposal.md` for why. What the design has to work with, observed on
`main` (2026-09-27):

- **The classifier.** `planet_gen::biome_at(cfg, direction, surface_m)`
  applies these rules in order:
  - below sea level less 1 m is Ocean;
  - below sea level plus the 2 m beach band is Beach;
  - past latitude `|y| > 0.875` is Tundra;
  - above 105 m is Mountains;
  - otherwise `moisture(cfg, dir)` splits the rest: below `desert_below` 0.36
    is Desert, above `wet_above` 0.64 is Swamp under 5 m and Jungle over it,
    and everything else is Fields.
- **The moisture field** is `noise01` with 4 octaves, a feature size of
  `moisture_m` 188 m, and its own seed salt. It does not touch the altitude,
  so the land, the sea, the rivers and the mountains do not move when it
  changes.
- **Every threshold is a config field**, not a literal. So a generator
  version can be a set of values rather than a second function.
- **The measured shares** come from `distribution_report`, an ignored test,
  and `docs/tenebris-comparison.md`. Of the land, Fields is about 59%.
  Desert, Jungle and Swamp are each a few percent of the sphere.
- **The tests that constrain it:**
  - `a_kilometre_of_land_crosses_more_than_one_biome` (1 km walks outside the
    cold band; at most one in four stays in one biome);
  - "Every biome occurs";
  - "A beach is sand".
- **Trees read the biome:** jungle scatters 115 in 256, swamp 34, fields 13
  and tundra 2.
- **World identity.** `GENERATOR_VERSION` (4) lives in `terrain.rs`.
  `WorldFile` stores the seed and not the version. One app call site,
  `planet_terrain.rs`, chooses the config the running world uses.

## Goals / Non-Goals

**Goals:**
- Biomes big enough to be a place, at a scale the owner picks on the map.
- No temperate biome as the majority of the land.
- An existing world does not change under its player without being asked.

**Non-Goals:**
- **New biomes** (savanna, taiga, a forest distinct from fields). They could
  follow, and the map would show where they would go, but the owner asked for
  bigger, not more.
- **Biomes from the simulated climate** (a desert where the air is dry). The
  atmosphere is tuned often. Tying the land to it would change the ground
  every time the weather is retuned, and it would make the biome depend on a
  simulation rather than on one pure function of direction.
- **Moving the cold band or the mountain line.** Tundra and Mountains stay as
  they are.
- **Moving an old world to the new biomes.** An old save keeps version 4. The
  owner starts new worlds to get version 5 (survey B3).

## Decisions

**1. Scale and shares are chosen on the map, from the real generator.**
- The `world-map` instrument renders the planet's biomes for each candidate:
  - today (188 m, 0.36 / 0.64);
  - twice the width (about 375 m, four times the area);
  - four times the width (about 750 m, sixteen times the area).
- Each candidate is shown with thresholds retuned to the target shares.
- The owner picks one.
- *Alternative:* take "4x" as one reading and ship it. Rejected, because the
  two readings differ by a factor of four in area.

**2. The shares come from the thresholds, tuned against the measured
distribution.**
- A 4-octave noise is clustered about 0.5, so 0.36 and 0.64 sit far out in
  its tails.
- The instrument measures the moisture's quantiles over the temperate land
  and reports the thresholds that give the target shares. Those two numbers
  go into the config.
- The starting target is about a third each for fields, desert, and jungle
  with swamp. The spec's floor is a fifth each and its ceiling half.
- *Alternative:* remap moisture through its measured cumulative distribution,
  so a threshold would be a share directly. Rejected: it is more machinery for
  the same two numbers, and the remap table would itself need versioning.

**3. A generator version is a config, and the save records it.**
- `TerrainConfig::for_version(v)` answers `Some(config)` for 4 and 5, and
  `None` for anything else.
- Version 4 is today's values, kept verbatim as `TENEBRIS_V4`. Version 5 is
  the new moisture scale and thresholds.
- The version is recorded in the world's identity file, which
  `world-persistence` introduces. An old save's identity reads as 4, and a
  new world writes `GENERATOR_VERSION`. This change only adds version 5 to
  the table the identity is checked against.
- The app refuses a save with an unknown version and does not generate
  version 5 under it. That is the spec's "not silently treated as the same
  world".
- *Alternative:* bump the version and let old worlds regenerate. Rejected: it
  is exactly the silent change the existing requirement forbids. It is also
  the case in which a player's desert town comes back as a jungle.

**4. The edit log is unaffected.**
- The land's shape does not change between version 4 and 5; only the
  classification of dry land does.
- An edit's cell and material mean the same thing under either version.

**5. The tests follow the scale.**
- ~~The 1 km walk becomes a 4 km walk: at least three in four cross a
  biome.~~
- ~~A 1 km walk is added in the other direction: at least a third stay in
  one biome. This is what "bigger" means as a test.~~
- A shares test runs on the shipped seed and four others. It fails naming the
  biome and its share.
- If the owner picks the 375 m scale, the walks become 2 km and 500 m.

**5a. What the walk tests found, and what "bigger" is tested as (measured
2026-09-28, building version 5).** The two walk tests above were written
before anything was measured, and version 5 failed both. Measured on
temperate land, each field with its thresholds retuned to thirds, over 88
kilometre walks that stay on it:

| moisture field | biome edges a kilometre | kilometre walks in one biome |
| --- | ---: | ---: |
| 188 m (version 4's width) | 13.6 | 0% |
| **750 m (version 5, survey B1)** | **3.3** | **3%** |
| 750 m, two octaves | 2.5 | 6% |
| 1,500 m | 1.5 | 22% |
| 3,000 m | 1.0 | 33% |
| 3,000 m, two octaves | 0.8 | 40% |

- `moisture_m` is the noise's wavelength, not a patch's width. Cut into
  thirds, a patch is a fraction of it, and the finer octaves fray its edges.
  A walk that stays in one biome for a kilometre a third of the time takes a
  3,000 m field, four times the owner's choice. Four-kilometre walks could not
  be judged at all: at this body's size almost none stay on temperate land
  that long.
- So the owner's 750 m does what was asked. It is four times wider, and a
  kilometre crosses about a quarter of the edges it did, 3.3 against 13.6.
  The tests say that, not the walk counts the owner never asked for:
  - `the_biomes_are_about_four_times_wider`: a kilometre of temperate land
    crosses at most a third as many edges as the same generator with the old
    188 m field, at the same thresholds. Version 4 fails it (4.1 against 4.1).
  - `a_kilometre_of_land_crosses_more_than_one_biome`, the main spec's
    existing test, still holds.
  - `grass_is_not_the_majority_on_five_seeds`.
- Whether 750 m is big enough on foot is the owner's to judge, walking it.
  That is survey B4. A wider field later is a new generator version, as this
  one was, and worlds made on version 5 keep it.

**6. Three things the plan above missed (found 2026-09-28, reading the code
before building it).**

- **The climate reads the biome.** `Atmosphere::new` gives every land cell a
  wetness and an albedo from `biome_at`: desert 0.1 and 1.4 times the land
  albedo, fields 0.6 and 1.0, jungle 0.9 and 0.8. Version 5 turns about a
  fifth of the land from fields to desert and a fifth to jungle. From the
  measured shares and those coefficients (arithmetic, not a run), the land's
  mean albedo rises by about 0.01 and its mean wetness falls by about 0.04.
  The thermostat trims the sun for the albedo, but the settled states that a
  new world opens on (`climate-balance` decision 8) were made on version 4.
  A version-5 world opened on them would start off balance, which is what
  survey K6 asked to be rid of.
  - So the settled states are re-made on version 5's terrain, levels 3 and 5.
    That is two to three hours of settle, as the level-5 one was.
  - The shipped `.ron` records the generator it was made on. `shipped_settled`
    falls back to the spin-up, with a warning, where the world's generator is
    not the state's, as it already does for other settings.
- **The version number also salts the planar sampler.** `terrain.rs`'s
  `TerrainGenerator`, the planar hex-patch reference that `headless.rs` runs,
  mixes `GENERATOR_VERSION` into every hash. Bumping it to 5 would move that
  sampler's every voxel for a change that is only the spherical generator's
  moisture. Its salt becomes its own constant, fixed at 4, the version it was
  written under.
- **The planet is built once, at launch, from one constant.** `TERRAIN` in
  `planet_terrain.rs` is a `const` read at 47 places. `create_planet` runs
  once at `Startup`. Loading a world from the saves screen swaps its edits
  and rebuilds the fine tier, but never the planet. That is also why a save
  made on another seed is refused rather than opened. So "build the terrain
  from the save's version" has two parts:
  - The config becomes the world's, chosen once when the app starts from the
    world it opens. The 47 reads go through one accessor rather than a
    constant.
  - A world of the other version, chosen from the saves screen, needs the
    planet rebuilt. Rebuilding it in place means tearing down and remaking
    every planet resource (the base records and their GPU buffers, the fine
    and contact tiers, the atmosphere and the map cache), which the code has
    never done.
  - **Recommendation, provisional until the owner answers:** the game
    restarts itself into the other version, the same as quitting and
    launching with `--world <id>`, and says so on screen. The reload costs a
    launch (about 12 s in the cloud, less on the owner's machine). The owner
    starts new worlds for the new biomes (survey B3), so it happens when an
    old world is opened, and once when the first new world is made.
  - *Alternative:* rebuild the planet in place. It is right eventually, since
    visiting another planet will need it. It is the larger change, and not
    this one.

As built (2026-09-28):
- The accessor is `planet::terrain_config()`, with `generator_version()`
  beside it. `choose_generator` sets it once, in `desktop::run`, from the
  identity of the world the launch opens (4 for a slot with none). Anything
  that reads it first, a test included, gets the current generator, and a
  second choice of another version is refused.
- The restart is built as recommended, since the owner has not answered
  (now survey B5).
  The saves page's Load, and New while an old world is open, launch the game
  again with `--world <name>` and close this one, whose save drains on the way
  out as any quit's does. The saves page says so: "<name> is made with the
  older biomes: restarting the game into it".
- The settled states are `settled-g<generator>-l<level>`. Version 4's were
  renamed, not remade. Version 5's level 3 landed on its second true year at
  14.98 °C, and its new world's first 60 days read 14.87 to 15.25 °C. Its
  level 5 ran as version 4's did (`climate-balance` finding 10): the first
  true year 15.97 °C, the sea shifted down 0.97 K, the second 14.82 °C, and
  a new world's 30 days 14.76 to 14.98 °C.
- The map's cached base is named for the generator too, so a world of one
  version never shows another's picture.
- Version 5 does not carry survey G1 (the desert's east-west rock stripes
  scattered into patches), which is unanswered. If the owner takes it, it is
  version 6, with B4 if the biomes are to be wider still.

## Measured: the candidates (2026-09-27)

`world-map`'s instrument (`examples/world_map.rs`, task 1.1) classified a
2,048 x 1,024 equirectangular raster of the shipped planet through
`pbd_core::map::base_texel`, each pixel weighted by its share of the sphere.
Land is 40% of the surface.

**Today, as shipped (188 m, 0.36 / 0.64), 91% of the temperate land is
fields**: the owner's "the majority is just grass", measured. Of all the land:

| beach | fields | desert | jungle | swamp | mountains | tundra |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 10% | 59% | 3% | 3% | 0.2% | 2% | 23% |

**The moisture's quantiles over the temperate land, and the thresholds that
give a third each** (task 1.1):

| `moisture_m` | q10 | q33 | q50 | q67 | q90 | `desert_below` | `wet_above` | fields | desert | jungle | swamp |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 188 | 0.392 | 0.463 | 0.500 | 0.537 | 0.607 | 0.463 | 0.537 | 33% | 33% | 32% | 2% |
| 375 | 0.392 | 0.464 | 0.501 | 0.537 | 0.607 | 0.464 | 0.537 | 33% | 33% | 32% | 2% |
| 750 | 0.394 | 0.468 | 0.506 | 0.544 | 0.614 | 0.468 | 0.544 | 33% | 33% | 32% | 2% |

What it shows:
- **The thresholds, not the scale, are what made it all grass.** The noise
  is clustered about 0.5 at every scale, so 0.36 and 0.64 sit out past the
  10th and 90th percentiles. The scale changes how big a region is, and the
  thresholds how much of each there is.
- **The shares barely move with the scale**, so the thresholds found at one
  scale serve all three. At 750 m (B1) they are 0.468 and 0.544.
- **Swamp stays small (2%)**: it is the wet share below `swamp_max_elev_m`
  (5 m), which little temperate land is. Jungle and swamp together are the
  third that B2 asks for.
- Of all the land, 750 m retuned gives beach 10%, fields 22%, desert 22%,
  jungle 21%, swamp 1%, mountains 2% and tundra 23%.
- The rasters are `docs/mockups/world-map/biomes-{today,188,375,750}.png`.

## Risks / Trade-offs

- [More jungle means more trees drawn] → Jungle goes from 3% of the land to
  21% (measured above). That can move frame time. It cannot
  be measured in a cloud session (CLAUDE.md). The owner runs
  `tools/perf_suite.py` with the old build against the new, and the report
  goes in `docs/benchmarks/`. If it regresses, the tree scatter's jungle
  density is argued in this design.
- [A big field varies more from seed to seed, so one world could come out
  half desert] → The shares test runs five seeds, not one.
- [The spawn is in jungle today at 87 m, and the captures frame from it]
  → The spawn is a fixed direction (`spawn_direction` in `flight_view.rs`).
  The altitude does not change, so it stays on dry land, but its biome may
  change. The `surface`, `seam` and `coast` captures are re-checked and
  re-framed if they change.
- [An old build opening a new save] → It ignores `generator` and generates
  version 4. That is the usual one-way limit, and it is noted in the save
  format's comment.

## Migration Plan

- There is no save field of its own. The version rides the identity that
  `world-persistence` adds, and an old save's identity reads as 4.
- Old saves keep their biomes. New worlds get version 5.
- Rollback is the previous build. A version-5 save opened by it shows version
  4's biomes on the same land. Nothing is lost, and nothing is corrupted.

## Decided by the owner (survey, 2026-09-27)

- **B1 (recommendation accepted):** four times the width, about 750 m, which
  is sixteen times the area. The walk tests stay at 4 km and 1 km.
- **B2 (recommendation accepted):** fields, desert, and jungle with swamp
  about a third each of the temperate land, and none over half.
- **B3, "I'll start worlds over to pick up the new biomes.":** no
  "update biomes" option is built. Old worlds keep version 4, and the owner
  starts new worlds on version 5.
