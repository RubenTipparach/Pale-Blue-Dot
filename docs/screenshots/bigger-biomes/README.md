# Bigger biomes: screenshots

For `openspec/changes/bigger-biomes` task 4.1: an old world (generator
version 4) beside a new one (version 5), and each of version 5's biomes seen
from inside it. Captured 2026-09-28 in a cloud container on lavapipe at
1440 x 900, with no GPU, each a fresh run of the fast build (`--profile
fast`) at `--time 12` on day 0. They show what the ground looks like, not how
smoothly it runs; frame cost was not measured (CLAUDE.md).

- **Version 4** is a slot with its identity taken away, as a slot from before
  identities has none, opened with `--world "Old biomes"`. The launch records
  generator 4 and builds the planet from it, and the world opens on version
  4's settled climate (`settled-g4-l5`).
- **Version 5** is a new, memory-only world on version 5's settled climate
  (`settled-g5-l5`).

Both worlds open on the same land: the heights are version 4's to the bit.
What moves is which biome the land is.

## Version 4 beside version 5

| files | flags | what it shows |
| --- | --- | --- |
| `map-planet-v4.jpg`, `map-planet-v5.jpg` | `--walk --menu map --map-layer biomes --map-mpp 40` | The whole planet's biome layer, with each biome's share of the land in the key. Version 4: fields 63%, desert 3%, jungle 3%, swamp 0%. Version 5: fields 23%, desert 23%, jungle 22%, swamp 1%. Beach (5%), mountains (2%) and tundra (24%) are the same in both, being set by height and latitude. The version 5 map matches the 750 m mockup the owner chose (`docs/screenshots/world-map/biomes-750.jpg`), within a point on each share. |
| `map-v4.jpg`, `map-v5.jpg` | the same at `--map-mpp 12` | The spawn's continent, the player's marker in the middle. Version 4's fields hold scattered specks of desert and jungle a few hundred metres across; version 5's biomes are regions kilometres across, desert inland and jungle round the coasts. |
| `orbit-v4.jpg`, `orbit-v5.jpg` | `--view orbit` | The planet from 7,800 m. At noon on day 0 both settled climates put one large cloud mass over the spawn's continent; the land at the rim shows version 5's desert as orange ground where version 4's is green. |
| `surface-v4.jpg`, `surface-v5.jpg` | `--view surface` | 90 m over the spawn, looking east. Version 4: fields and scattered trees up to grey stone. Version 5: a desert runs from the spawn's field to the same grey stone, with the east-west rock bands of survey G1 across it. |
| `spawn-v4.jpg`, `spawn-v5.jpg` | `--walk` | Eye level at the spawn: the same field (73 m up, dry grass, `tests::an_old_worlds_spawn_column_is_the_ground_it_was`), with desert on the hills behind it in version 5. |

## Version 5's biomes from inside

Each from `--view column --spawn <biome> --height 40 --pitch -18` (swamp and
beach 25 m up, looking further down, since they are strips): a camera over
the nearest place inside that biome, looking east. `--spawn <biome>` and the
column view standing over the spawn are capture instruments added for these
(`bigger-biomes` task 4.1).

| file | nearest | what it shows |
| --- | ---: | --- |
| `biome-desert.jpg` | 128 m | Sand with the rock bands of survey G1 running east-west, bare stone beyond. |
| `biome-jungle.jpg` | 853 m | Dense jungle trees to the horizon, desert past its edge. |
| `biome-fields.jpg` | 108 m | Grass and scattered trees, desert on the rise behind. |
| `biome-swamp.jpg` | 1,288 m | Low wet ground under jungle trees, with swamp water. |
| `biome-mountains.jpg` | 557 m | Grey stone, the ranges as they are today: barely above the land round them (`openspec/changes/taller-mountains`). |
| `biome-tundra.jpg` | 3,078 m | The northern coast under a clear sky: snow, pines and a beach. |
| `biome-beach.jpg` | 1,282 m | The sand rim of a lake, the jungle stepping up behind. |

## What they show that the tests do not

- **A new version-5 world opens in rain.** Version 5's settled climate, at
  noon on day 0 where a new world's clock opens, has full cover and rain over
  the whole spawn continent ("Weather here: cover 1.00, rain 1.00"); version
  4's has full cover and no rain. The weather replays from the shipped state,
  so every new world opens the same way. It is not a fault, but it is the
  first thing a player sees; the owner is told.
- **The desert's rock stripes show far more** now that deserts are regions:
  the bands run straight east-west across the whole desert east of the spawn.
  Version 5 does not change them; survey G1 asks whether they go.
- **The straight line where the tundra begins** is the cold band, a line of
  latitude. It was there in version 4 and in the approved mockup.
- **Beach reads 5% of the land in the game and 10% in the mockup**, in both
  versions, so it is the two tools counting differently, not version 5.
