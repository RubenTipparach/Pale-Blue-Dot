# Tenebris's own textures, as planets.html uses them

Copied byte-identical on 2026-11-14 from the owner's Tenebris repository
(`RubenTipparach/tenebris`, the Rust port's `tenebris-rs/assets/textures/`),
cloned read-only at commit `ef9865166349c7a3c4ae09acdd20640daf85b4fd`
("Merge pull request #89"), for the star-systems change's ported worlds
(`openspec/changes/star-systems`). Mockup only: no game code reads them.
Re-copy from the source when it changes; never edit these copies.

None of these was a Git LFS stub: every file is a real PNG, so the page draws
no stand-ins for Tenebris art. (The clone has no `.gitattributes`.)

| File | Source (in `tenebris-rs/`) | Bytes | Tenebris uses it for | The mockup uses it for |
| --- | --- | ---: | --- | --- |
| `planet_seq_grass.png` | `assets/textures/planet_seq/planet_seq_grass.png` | 475 | SEQ_GRASS top (Sequoia's tan sod; "Sequoia doesnt have green grass") | Sequoia's redwood-grove tops |
| `planet_seq_grass_dirt.png` | `assets/textures/planet_seq/planet_seq_grass_dirt.png` | 604 | SEQ_DIRT_GRASS, the sod's side | First metre of Sequoia's walls |
| `planet_seq_dirt.png` | `assets/textures/planet_seq/planet_seq_dirt.png` | 564 | SEQ_DIRT | Sequoia's earth; mycelium sides |
| `planet_seq_sand.png` | `assets/textures/planet_seq/planet_seq_sand.png` | 510 | SEQ_SAND | River banks |
| `planet_seq_snow.png` | `assets/textures/planet_seq/planet_seq_snow.png` | 418 | SEQ_SNOW | (copied; the peaks are off the vignette) |
| `planet_seq_snow_dirt.png` | `assets/textures/planet_seq/planet_seq_snow_dirt.png` | 604 | SEQ_DIRT_SNOW | (copied, as above) |
| `planet_seq_side.png` | `assets/textures/planet_seq/planet_seq_side.png` | 278 | Sequoia wood (TS tileset `wood`) | Redwood bark |
| `planet_seq_top.png` | `assets/textures/planet_seq/planet_seq_top.png` | 262 | Sequoia wood top | (copied; shown on the page) |
| `planet_seq_leaves.png` | `assets/textures/planet_seq/planet_seq_leaves.png` | 579 | SEQ_LEAVES, rust-red foliage (`tools/gen-sequoia-textures.py`) | Redwood crowns |
| `planet_seq_mycelium.png` | `assets/textures/planet_seq/planet_seq_mycelium.png` | 546 | SEQ_MYCELIUM, the Sporewood's ground | Fungal-forest tops |
| `planet_seq_shroom_cap.png` | `assets/textures/planet_seq/planet_seq_shroom_cap.png` | 718 | SEQ_SHROOM_CAP | Giant mushroom caps |
| `grass.png` | `assets/textures/grass.png` | 318 | GRASS; LEAVES tinted (0.65, 1.05, 0.50) | Tenebris's tree crowns, with that tint |
| `dirt_grass.png` | `assets/textures/dirt_grass.png` | 489 | DIRT_GRASS | (copied) |
| `dirt.png` | `assets/textures/dirt.png` | 563 | DIRT | Tenebris's earth |
| `sand.png` | `assets/textures/sand.png` | 461 | SAND | Tenebris's beach and desert |
| `rocks.png` | `assets/textures/rocks.png` | 464 | STONE; ASTEROID_ROCK tinted (0.70, 0.68, 0.65) | Stone under every column; the belt |
| `snow.png` | `assets/textures/snow.png` | 459 | SNOW | Tenebris's snow line; Frostbite's crust |
| `dirt_snow.png` | `assets/textures/dirt_snow.png` | 489 | DIRT_SNOW | Snow-capped walls |
| `wood.png` | `assets/textures/wood.png` | 443 | WOOD (mushroom stems are Wood) | Tenebris trunks; pale-tinted mushroom stems |
| `ice.png` | `assets/textures/ice.png` | 447 | FROST_ICE tinted (0.55, 0.80, 1.25) | Frostbite's and Rime's glacier |
| `moon.png` | `assets/textures/moon.png` | 222 | MOON; CRAG_ROCK tinted (1.30, 0.55, 0.35) | Crag, Luna, Selene, Verdant, Sprout, Quartz |
| `water.png` | `assets/textures/water.png` | 431 | WATER | (copied; the vignettes tint water flat) |
| `jungle_grass.png`, `jungle_dirt_grass.png` | `assets/textures/` | 193, 290 | JUNGLE_GRASS top and side | (copied) |
| `dry_grass.png`, `dry_dirt_grass.png` | `assets/textures/` | 198, 293 | DRY_GRASS, Tenebris's yellow-green pasture | Tenebris's fields |
| `cactus.png` | `assets/textures/cactus.png` | 504 | CACTUS | Tenebris's desert cactus |
| `animals.png` | `assets/textures/animals.png` | 34,885 | The animal atlas: 24 columns of 32 px, one per `AnimalKind` wire id (`fauna.rs`), body, head and leg bands (`animals.rs` `part_uv`) | The box-built animals: critter 0, stalker 1, sheep 3, chicken 4, cow 5, serpent 15, raptor 16, crane 17, sparkit 18, shellkin 19 |

Tints are Tenebris's `ATLAS_TINTS` (`crates/tenebris-client/src/renderer.rs`),
applied by the page as a per-channel multiply.

Mockup-only art for these worlds (portraits, maps, species icons, and
`../species/fauna_tex.png`, box textures for the species Tenebris did not
build) is drawn by `tools/art/star_systems_art.py`.
