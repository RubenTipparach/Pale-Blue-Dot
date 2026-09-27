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
- **Moving an old world to the new biomes.** An old save keeps version 4
  unless the owner asks otherwise (see Open Questions).

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
- The 1 km walk becomes a 4 km walk: at least three in four cross a biome.
- A 1 km walk is added in the other direction: at least a third stay in one
  biome. This is what "bigger" means as a test.
- A shares test runs on the shipped seed and four others. It fails naming the
  biome and its share.
- If the owner picks the 375 m scale, the walks become 2 km and 500 m.

## Risks / Trade-offs

- [More jungle means more trees drawn] → Jungle goes from about 2% of the
  sphere to perhaps a sixth of the land. That can move frame time. It cannot
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

## Open Questions

For the owner, at the map mockup:
- The scale: twice or four times the width.
- The target shares, if a third each is not right.
- Whether the current test world should move to version 5 (a menu choice,
  "update this world's biomes"), or new worlds only.
