# Design

## Context

- **The owner, 2026-11-26:** "Survival: you can start foresr fires during dry
  seasons. Torches can light up stuff or can be out out by rain."
- **What fire is today.** Placed light blocks (`Material::Torch`, the post
  and wall lanterns, the hanging lantern, the brazier, the candle, the
  campfire: `Material::emission`) light the cell light field from
  `bake_lamps` and burn for ever. A placed torch is drawn as the held one
  (`held/placed.rs`, `playtest-fixes-2026-11-25` design 4). A held torch
  lights the world round the hand (`glowing-mushrooms` design 14).
- **What the weather knows.** `pbd_core::atmosphere::Atmosphere::sample`
  gives, at any direction on the body, `air_c`, `humidity`, `wind`,
  `rain_rate`, `rain_seen`, `snow` and `sunlight`. The model is CPU, in the
  core, deterministic, saved (`to_bytes`/`restore`) and run by the server.
  Its `surface.wetness` is the biome's fixed wetness (0.1 desert, 1 sea),
  not a state: nothing tracks how dry the ground has become.
- **What the flora knows.** `flora::season_at(year_fraction, latitude)` dries
  grass to straw (`straw`, 0..1) in its season, strongest away from the
  equator; a year is 100 world days (`daylight::YEAR_DAYS`) of 48 minutes.
  `flora::grows_tree` and `tree_kit::grows_cactus` say, on the CPU, which
  cells stand a tree or a cactus; biomes are `planet_gen::Biome`.
- **What the world may change.** A world process proposes changes through
  the durable path, authored `@w<process>:<kind>/<id>`, checked against what
  the player has touched (`records::YieldSet`): the beavers do (`beaver.rs`).
  A felled tree is a `tree` record; the tree draw leaves out the nearest 64
  felled trees (`trees_taken.rs`, `FELLED`).
- **`lightning-fire`** (designed, not built) lands a strike on the tallest
  point near it and burns a few seconds in the rain, saving nothing.
- **The owner, on the first draft, 2026-11-26:** "Let's not spread fire too
  much for now , consider lots of factors like dryness of plants" (decision
  12), with Our World in Data's *Wildfires* page as "some ideas".

## Goals / Non-Goals

**Goals:**
- Fires that start only when and where the plants are dry, stay small, read
  before they are dangerous, and are stopped by weather, water and the player's work.
- One rule, run by the server, giving the same fire on every machine and on
  every replay.
- Everything a fire did is in the save, and it heals over time.
- Nothing a player built, planted or placed is ever burned by the world.

**Non-Goals:**
- Houses, crops and placed blocks catching fire (design 4; a follow-up for
  the owner).
- Smoke that chokes, firefighting gear, fire spreading through caves.
- Any of this in Explorer.

## Decisions

### 1. Its own change, turned on by Survival

Recommendation taken (ask only with screenshots). A forest fire is the world
changing itself: records, a world process, multiplayer authority, its own
look in the ground and the trees, and a weather field. None of that is about
the player's body, and `survival-mode`'s rules (`player/survival`) would be
the wrong place to read it. `survival-mode` says when fire is on: only in
Survival worlds. The torches are here too, because a torch going out is the
same rain-on-fire rule as a forest going out.

### 2. How dry it is: the ground's wetness, the fire weather, and each plant's dryness

Revised 2026-11-26 for decision 12 ("consider lots of factors like dryness
of plants"). What burns is decided per cell by its plant's dryness
(`plant_dry`, below); the place's weather is one of its factors.

- **`ground_wet`**, 0..1, one value per atmosphere cell, a new state of the
  atmosphere, stepped with it (unchanged):
  - rain adds 0.1 for each millimetre that falls (10 mm soaks it);
  - snow lying keeps it at 1;
  - it dries by 0.04 an in-game hour (120 s of play) x (0.5 + sunlight /
    1000 W/m^2) x (1 - humidity) x clamp((air_c + 5) / 30, 0, 1).
  So soaked ground under a dry summer sun needs about a world day to dry
  enough to burn, and a damp, cool place may never. It is "days since rain"
  without a counter: every shower resets it, every dry day lowers it.
  - The sea's cells stay 1.
  - Saving: an additive field of `Atmosphere::to_bytes`. A save from before
    restores with `ground_wet` at its spun-up value (the biome's wetness,
    `surface.wetness`: 0.1 desert, 0.6 fields, 0.9 jungle, 1 swamp), so no
    world changes under a player.
- **The fire weather**, `weather` 0..1, at a place:
  `weather = (1 - ground_wet) x air(humidity) x warm(air_c)`, where
  - `air(h) = 1 - 0.6 x smooth(0.35, 0.85, h)`: dry air leaves a plant as it
    is, humid air keeps it 40% wetter;
  - `warm(t) = 0.4 + 0.6 x smooth(0, 25, t)`: cool air dries slowly, never
    not at all (late-winter grass can still burn where it is dry);
  - altitude acts through these: the atmosphere's cells carry
    `surface.elevation`, so high ground is cooler (`air_c`) and wetter
    (`humidity`, the rain on its slopes), and above the generator's snow
    line (`TerrainConfig::snow_line_m`, `mountain_snowcap_elev_m`) the snow
    term below is 0.
  The biome multiplier of the first draft is gone: a biome's fire comes
  from what grows there (its plants' kinds) and its own `ground_wet`.
- **`plant_dry`**, 0..1, per cell, **derived and never saved**: a pure
  function of the world's generator version, the cell, the clock and the
  atmosphere's sample there, the same on every machine:

  ```text
  plant_dry = clamp(cure(kind, season) x weather x shade x water x hollow x patch x grazed, 0, 1)
            = 0 where snow lies, where the cell is flooded, where the plant is a cactus or there is none
  ```

  | Factor | Values | Source in the code |
  | --- | --- | --- |
  | `kind` | the cell's plant: grass (green to straw), a shrub, a deciduous tree, a conifer, a palm or jungle tree, a cactus (never), none | grass tops: the top block (`flora` grass codes); a tree: `flora::grows_tree` and its species, `tree_kit::pick` / `BIOME_TREES` (oak, birch, willow deciduous by `tree_kit::look(..).deciduous`; pine and krummholz conifers; kapok and palm evergreen broadleaf); a cactus: `tree_kit::grows_cactus`; a shrub: ground cover with a bush (`scatter`) |
  | `season` | `cure`, 0..1, how cured the plant is in its own hemisphere's year (below) | `flora::season_at` (its `own` year is half a year on across the equator, `SEASON_TROPIC` fades seasons out near the equator), `SEASON_STRAW`, `SEASON_LITTER` |
  | `weather` | above | `atmosphere::Sample`'s `humidity`, `air_c`, `rain_rate`; `ground_wet` (new) |
  | `shade` | grass and shrubs only: 1 - 0.07 for each tree among the cell's 5 or 6 neighbours, x 0.85 under its own tree (a closed stand's floor stays green) | `grows_tree` on the neighbours, the topology's neighbour tables |
  | `water` | riparian: 0.35 on a cell beside water, 0.675 two cells away, 1 from three cells on (`0.35 + 0.65 x smooth(1, 3, cells to water)`): a bank is never dry enough to burn | `planet_gen::in_river`, `map::river_water`, the sea (`surface_altitude < sea_level_m`), the flora's `FLOODED_BIT` (a river, a pool, the sea) |
  | `hollow` | `1 + 0.06 x clamp(layers above the neighbours' mean, -2, 2)`: a hollow holds water and stays greener (0.88), a rise dries first (1.12) | the column heights the generator gives the cell and its neighbours |
  | `patch` | `0.62 + 0.5 x n`, `n` a smooth 0..1 field about 60 m across, times `0.92 + 0.16 x roll` per cell: the soil's own moisture, so the same field has green and cured patches | `flora::shader_hash` / `tree_kit::roll` with a new salt, at the cell key (derived, never saved) |
  | `grazed` | grass only: 0.5 on a pen's cells and within 2 cells of one (grazed short), else 1 | `husbandry::farm::Conditions::grazing`, the pens' records |
  | `snow` | 0 where snow lies, else 1 | `atmosphere::Sample::snow`, `surface.snowy`, the generator's snow line and snow tops |

  **The cure** by kind, from the season's `c = max(dry(local), 0.5 x straw)`,
  where `dry(local)` is a new bump on the plant's own year, one at local
  0.10 (late summer, the year starting at the place's midsummer) and nought
  0.20 either side, faded toward the equator by `SEASON_TROPIC` as every
  season is; and `straw` is the flora's (`SEASON_STRAW`, which peaks a
  little before midwinter):

  | Kind | `cure` | Spring (c 0) | Late summer (c 1) |
  | --- | --- | ---: | ---: |
  | Grass | 0.2 + 0.8 c | 0.20 | 1.00 |
  | Shrub | 0.25 + 0.6 c | 0.25 | 0.85 |
  | Deciduous tree (oak, birch, willow) | 0.15 + 0.45 c + 0.15 x litter | 0.15 | 0.60 |
  | Conifer (pine, krummholz) | 0.3 + 0.6 c | 0.30 | 0.90 |
  | Evergreen broadleaf (kapok, palm) | 0.1 + 0.3 c | 0.10 | 0.40 |
  | Cactus | 0 | 0 | 0 |

  **The fire season follows each hemisphere's summer** (OWID, below): the
  fire's `dry` bump peaks in late summer of the place's own year, half a
  year apart across the equator, as Europe burns June to August and the
  south in its own summer. The flora's drawn straw (`SEASON_STRAW`) peaks
  a little before midwinter instead; it counts at half (`0.5 x straw`), so
  a cured winter field can burn only where its ground has dried, which in
  winter it seldom does. That the grass looks straw-coloured in winter and
  burns best in late summer is a look question, not a rule one: moving the
  drawn straw is the owner's (to be asked with screenshots, not here).

- **Examples** (fields, an open cell, no neighbour trees, level ground,
  `patch` 1; worked by hand, and by the mockup's own code, which runs the
  same lines):

  | Place and time | `ground_wet`, humidity, air | `weather` | Plant | `cure` | Other | `plant_dry` | Burns? |
  | --- | --- | ---: | --- | ---: | --- | ---: | --- |
  | Spring meadow | 0.5, 0.6, 16 C | 0.29 | grass | 0.20 | - | 0.06 | no |
  | Midsummer field | 0.25, 0.45, 26 C | 0.70 | grass | 0.60 | - | 0.42 | no |
  | Late-summer field, two dry weeks | 0.12, 0.32, 29 C | 0.88 | grass | 1.00 | - | 0.88 | yes |
  | The same, on the river bank | | 0.88 | grass | 1.00 | water 0.35 | 0.31 | no |
  | The same, two cells from the river | | 0.88 | grass | 1.00 | water 0.675 | 0.59 | barely |
  | The same, under an oak wood (4 trees round) | | 0.88 | grass | 1.00 | shade 0.72 x 0.85 | 0.54 | barely |
  | The same, in a hollow, a green patch | | 0.88 | grass | 1.00 | hollow 0.88, patch 0.7 | 0.54 | barely |
  | The same, beside a pen | | 0.88 | grass | 1.00 | grazed 0.5 | 0.44 | no |
  | Oak in that field | | 0.88 | deciduous | 0.60 | - | 0.53 | barely |
  | Mountain pine, late summer | 0.2, 0.3, 22 C | 0.78 | conifer | 0.90 | - | 0.70 | yes, slowly |
  | Desert cactus | 0.1, 0.15, 35 C | 0.90 | cactus | 0 | - | 0 | never |
  | Jungle floor | 0.9, 0.85, 28 C | 0.04 | grass | 1.00 | shade | 0.03 | no |
  | Tundra, autumn | 0.6, 0.6, 4 C | 0.12 | grass | 0.32 | - | 0.04 | no |
  | Anything under snow | - | - | - | - | snow 0 | 0 | never |

- **Green plants do not burn.** A cell whose `plant_dry` is below
  **0.5** (`dry_min`) never catches, from a torch, a neighbour or
  lightning. Above it the hazard grows steeply, as the square of the
  margin: `burn(d) = ((d - 0.5) / 0.5)^2`, so 0.6 is 0.04 of a dry cell's
  hazard, 0.75 a quarter, 0.9 0.64.
- **"The dry season"** at a place is when its open grass would read
  `plant_dry` >= 0.5 there. It moves with the weather and the hemisphere,
  and differs from valley to valley.
- **Shown**, in Survival only, as a small flame beside the HUD's
  thermometer while the player stands in the dry season, and as a layer of
  the world map's weather overlays (`weather-overlays`). No text.

### 3. Fuel: what each cell has to burn

Revised 2026-11-26 (decision 12). Derived from the world's own generator
version and records, never saved. Spread is a hazard per second to each
neighbour, multiplied by `burn(plant_dry)` of the neighbour (design 2):

| Fuel | Where | Spread, per second to each neighbour, at `burn` 1 | Burns | Then |
| --- | --- | --- | --- | --- |
| Grass | a grass, dry grass or jungle grass top | 0.20 | 15 s | ash |
| Shrub | ground cover with a bush (`scatter`) | 0.12 | 40 s | ash |
| Deciduous tree | `grows_tree`, a deciduous species, not felled | 0.02 | 90 s, a ground fire round the trunk | a scorched tree |
| Conifer | `grows_tree`, a pine or krummholz, not felled | 0.04; **x 3 as a crown fire** where `plant_dry` >= 0.85 and the wind is 6 m/s or more | 90 s; a crown fire's crown burns 30 s of it | a charred snag |
| Evergreen broadleaf | kapok, palm | 0.02 | 90 s | a scorched tree |
| None | water, sand, rock, snow or ice, soil without grass, a road, a path, a paved or tilled cell, a cactus | never | - | a break |

Grasslands and savannas burn far more than forests (OWID, decision 12), so
the rule is built that way: grass carries a fire, a shrub less, a tree ten
times slower than grass and only where it is itself dry. A crown fire needs
a very dry conifer and a strong wind together; any other tree burns at its
foot and stands scorched.

What a tree's fire does to the tree's look: a crown fire leaves a snag
(trunk, no crown); a ground fire leaves the tree standing with a charred
foot and browned leaves until its regrowth (design 8).

Spread is a hazard per second; in a one-second tick a neighbour catches with
`p = 1 - exp(-lambda)`. On late-summer grass (`plant_dry` about 0.85) with
no wind the front advances about 0.1 to 0.2 m/s; in a 3 m/s wind about
0.2 m/s downwind and half that upwind; in 8 m/s about 0.25 m/s, under
1 m/s in any wind the atmosphere makes. Through trees it creeps. Worked
examples in decision 12. All starting numbers for the owner to tune
(`assets/config/fire.ron`, design 11).

### 4. The player's things never burn

- **A change the world makes never overwrites the player's** (CLAUDE.md).
  Every cell the player has edited, built, tilled or planted, every placed
  block, chest, fence and furniture, and every kept animal's pen, is in the
  blaze's `YieldSet`: a break the fire stops at, as the beavers' dams stop
  at a player's cells.
- The ignition is the player's when a player lights it, and named in the
  blaze record; the spread is the world's, authored `@wfire:blaze/<n>`.
- Recommendation taken (ask only with screenshots): this also keeps a
  Survival base safe from a fire it did not start. Houses and crops that
  catch are a follow-up the owner can ask for, with screenshots then.

### 5. The spread rule, run by the server

Revised 2026-11-26 (decision 12): the hazard reads each neighbour's own
dryness, green cells never catch, there is no ember throw, and the limits
are tight.

- **A blaze** is a record, kind `blaze`, id n, schema 1:
  `(started_s, origin: cell key, lit_by: Player(n) | Lightning,
  state: Burning | Out(day))`. Its cells are its lines: each tick's newly
  lit and newly burnt-out cells are one durable line authored
  `@wfire:blaze/<n>`, so the fire is saved as it goes, never on exit.
- **The tick** is 1 s of world time. For each burning cell, in cell-key
  order, for each of its 5 or 6 neighbours (the topology's explicit tables,
  pentagons included) with fuel, `plant_dry` >= 0.5, not burning, not burnt
  and not in the yield set, a hazard
  `lambda = spread(fuel) x burn(plant_dry) x wind x slope x rain` is tried
  against `hash(world seed, blaze id, cell, tick)`. No hash map's order and
  no GPU result decides anything.
  - `burn(d) = ((d - 0.5) / 0.5)^2` (design 2);
  - `wind` = 1 + 0.15 x speed x max(0, cos) - 0.10 x speed x max(0, -cos),
    at least 0.2, `cos` the angle between the wind and the step;
  - `slope` = 1 + 0.10 per layer the neighbour stands higher, 0.8 lower;
  - `rain` = exp(-rain_mm_h / 0.5), and none at 1 mm/h or more.
- **No ember throw**, for now (recommendation taken, ask only with
  screenshots): a fire crosses only to a touching cell, so a break one cell
  wide holds every fire.
- **It burns itself out.** Nothing keeps a fire going but dry neighbours:
  a front that meets green ground (a bank, a hollow, a shaded floor, a
  grazed pasture, a green patch, all below 0.5) has nothing to take, and
  its cells burn down and go out.
- **Weather is sampled** from the server's atmosphere at each burning cell
  once a tick, and `plant_dry` with it. The fuel comes from the generator
  functions, so a blaze far from every player runs without loaded terrain,
  and only while the world is played (world time).
- **Limits**: 2 live blazes in a world, 300 cells (about 0.2 ha, a patch
  about 50 m across) and 40 burning at once in each. At a limit the front
  stops spreading and burns out, and the line says so. A third ignition
  while two burn is a short local fire (design 6).

### 6. Starting one

Revised 2026-11-26 (decision 12). In a Survival world, on a cell whose
`plant_dry` is 0.5 or more:
- **A lit torch** used (hold use, 1 s) on a grass, shrub or tree cell lights
  that one cell. It is the player's act, in the same durable line as the
  record's creation. Whether it then spreads is the rule's: on marginal
  ground it often burns its one cell and goes out.
- **Lightning**: a `lightning-fire` strike whose cell has less than 0.5
  mm/h of rain and `plant_dry` of 0.7 or more starts a blaze lit by
  `Lightning`, and at most one in 10 world days within 2 km of the last
  (`lightning_quiet_days`, `lightning_quiet_m`), so a dry region sees a
  few small scars a season, never a landscape lost. A wet strike keeps
  `lightning-fire`'s short fire and saves nothing.
- **Never** from a placed campfire, brazier, torch or lantern: a placed fire
  is contained (recommendation taken), so the player's own camp never lights
  the dry grass it stands in.
- **Below 0.5**, a torch on a plant makes a small flame that dies in 3 s
  and saves nothing; no note says why (CLAUDE.md, "Game text is plain").
  In spring nothing catches anywhere.

### 7. Stopping one

- **Rain**: the `rain` term slows the spread; and every burning cell's time
  left shrinks by `rain_mm_h x 0.5` s a second, so 4 mm/h puts grass out in
  about 5 s and a tree in about 30 s. At 1 mm/h or more no cell catches.
- **Water**: a water cell, and a cell beside fresh or sea water, does not
  burn. A water bucket poured (`water-field`'s `pour`) on a burning cell puts
  out it and its neighbours, and leaves them wet for a world day.
- **Breaks**: bare ground and green ground. Digging a line of grass to
  soil, a road, a path, a paved or tilled cell, sand or rock stops any fire,
  and so does any cell below `plant_dry` 0.5: a river's green bank, a
  grazed pasture, a shaded wood floor. There is no ember throw for now
  (decision 12), so a break one cell wide holds.
- **It ends** when no cell burns, which on a front that reaches green or
  wetter ground is soon (design 5). The record becomes `Out(day)`.

### 8. What it leaves, and regrowth

- **The scar** is the blaze's burnt cells and the day it went out, kept in
  the record (its lines). Derived from it, by days since:
  - ground: ash grey at once, darker where trees stood; green shoots from
    day 2; grass back from day 4, patchy, whole by day 8;
  - shrubs back from day 10;
  - trees: a charred snag (trunk, no crown, a charred bark texture) until
    day 20, then a sapling, a young tree at day 40 and the full tree at day
    60, the tree's own species and size from its cell as before;
  - a felled snag gives 1 `charcoal`, a forge fuel (burns as `coal` does),
    and its stump regrows as a felled tree does.
- **Drawn**: the ground and tree draws read a burnt-cell table near the
  camera, uploaded only when it changes, as `TreesTaken` is for felled trees
  but sized for a forest (8,192 cells, a hash table in a storage buffer,
  nearest first). A cell past it draws as unburnt only where it is too far
  to read; the table's reach is set from the tree draw's own reach with a
  test, as `FELLED` is.
- **A burnt cell has nothing to burn** until its grass is whole again
  (day 8; a tree's cell until its sapling, day 20): its `plant_dry` reads 0,
  so ground that burned this season cannot burn again in it, and a
  region's burnt share stays small (decision 12).
- **Never a generation change.** The ground's blocks are not edited: the
  ash is a look over the world's own top, and the world reads the same
  cells as before.

### 9. Torches and the rain (Survival)

- **Placed torch.** A placed `Material::Torch` whose cell has open sky above
  (no solid cell within 8 layers) and `rain_seen` of 0.3 or more for 20 s
  goes out. That is a `lamp_out` record keyed by the cell, authored
  `@wfire:lamp/<cell>`, and the block stays the player's untouched. An out
  torch stops lighting the field and draws its head charred, with no flame.
  Snow does not put one out; a roof or an overhang keeps one lit.
- **Held torch.** Lit in hand under open sky in that rain for 10 s, it goes
  out: no light, a charred head. Under a roof it stays lit.
- **Relighting**: use a held lit torch on an out torch, or hold an out
  torch to any fire (a campfire, a brazier, a lit torch, a burning cell),
  1 s. Relighting deletes the `lamp_out` record (the player's act).
- **A lit torch lights** an out torch, a brazier, a candle or a lantern that
  is out, and, where it is dry (design 6), growth.
- **Explorer**: torches never go out and light nothing but the world.

### 10. Danger to the living

- Standing in a burning cell: 4 health a second; beside one: 1 a second,
  and the body's `feel` + 20 C within two cells (`survival-mode` design 13).
- Animals flee from 40 m of a burning cell, at their run speed, and a herd
  runs together; predators too.
- A fire is seen and heard first: smoke from 500 m, the glow at night from
  further, the crackle from 60 m (`audio-and-ambience`).

### 11. `assets/config/fire.ron`, the proposed shape

Revised 2026-11-26 (decision 12): the dryness table, the slower spread, no
ember throw, the tight limits.

```ron
// Fire (pbd_core::fire; openspec/changes/wildfire). Seconds of world time,
// metres, mm/h of rain, deg C, world days of 48 minutes. Survival worlds only.
(
    tick_s: 1.0,
    ground_wet: (per_mm: 0.1, dry_per_hour: 0.04, sun_ref_wm2: 1000.0),
    weather: (humid_dry: 0.35, humid_wet: 0.85, humid_keep: 0.6,
              warm_from_c: 0.0, warm_full_c: 25.0, warm_floor: 0.4),
    dryness: (
        dry_min: 0.5,                  // below it nothing catches
        burn_power: 2.0,               // burn(d) = ((d - dry_min) / (1 - dry_min))^power
        season: (dry_peak: 0.10, dry_half: 0.20, straw_share: 0.5),
        cure: {                        // cure = base + per_cure x c (+ per_litter x litter)
            "grass": (base: 0.2, per_cure: 0.8),
            "shrub": (base: 0.25, per_cure: 0.6),
            "deciduous": (base: 0.15, per_cure: 0.45, per_litter: 0.15),
            "conifer": (base: 0.3, per_cure: 0.6),
            "evergreen": (base: 0.1, per_cure: 0.3),
            "cactus": (base: 0.0, per_cure: 0.0),
        },
        shade: (per_tree: 0.07, own_tree: 0.85),
        water: (bank: 0.35, from_cells: 1.0, to_cells: 3.0),
        hollow: (per_layer: 0.06, clamp_layers: 2.0),
        patch: (base: 0.62, span: 0.5, field_m: 60.0, cell_base: 0.92, cell_span: 0.16),
        grazed: (mult: 0.5, cells: 2),
    ),
    fuel: {
        "grass": (spread_per_s: 0.20, burn_s: 15.0),
        "shrub": (spread_per_s: 0.12, burn_s: 40.0),
        "deciduous": (spread_per_s: 0.02, burn_s: 90.0),
        "conifer": (spread_per_s: 0.04, burn_s: 90.0, crown_mult: 3.0,
                    crown_dry: 0.85, crown_wind_mps: 6.0, crown_s: 30.0),
        "evergreen": (spread_per_s: 0.02, burn_s: 90.0),
    },
    wind: (down_per_mps: 0.15, up_per_mps: 0.10, floor: 0.2),
    slope: (up_per_layer: 0.10, down: 0.8),
    rain: (spread_scale_mmh: 0.5, douse_s_per_mmh: 0.5, no_catch_mmh: 1.0),
    ember_throw: None,                 // none for now (decision 12)
    ignite: (lightning_dry: 0.7, lightning_rain_below_mmh: 0.5,
             lightning_quiet_days: 10.0, lightning_quiet_m: 2000.0),
    limits: (live_blazes: 2, cells: 300, burning: 40),
    regrow_days: (shoots: 2.0, grass: 4.0, grass_whole: 8.0, shrub: 10.0,
                  sapling: 20.0, young: 40.0, tree: 60.0),
    torch: (placed_out_s: 20.0, held_out_s: 10.0, rain_seen: 0.3, sky_layers: 8,
            relight_s: 1.0, damp_flame_s: 3.0),
    harm: (in_cell_per_s: 4.0, beside_per_s: 1.0, feel_c: 20.0, feel_cells: 2,
           flee_m: 40.0),
    burnt_table: 8192,
)
```

### 12. Fires stay small, and plants burn by how dry they are (owner, 2026-11-26)

**The owner's direction**, on the first draft's fire and its mockup tab:
"Let's not spread fire too much for now , consider lots of factors like
dryness of plants". The first draft spread about 1 m/s on grass at danger 1
and 2.5 m/s downwind, threw embers two cells, and let a blaze reach 20,000
cells with 1,500 burning: too much. The direction is the owner's; **every
number below is a recommendation taken (ask only with screenshots)**.

And the owner's "some ideas", 2026-11-26: Our World in Data, *Wildfires*
(https://ourworldindata.org/wildfires), which this decision follows:
- grasslands and savannas burn far more area each year than forests, and
  forest fires are rarer: the fuel table favours cured grass and shrub,
  trees burn ten times slower, and a crown fire needs a very dry conifer
  in a strong wind (design 3);
- how a fire spreads depends on the amount and type of dry vegetation, the
  wind's speed and direction, moisture and heat, and some vegetation burns
  more easily than others: that is `plant_dry`'s factors (design 2);
- fire seasons are summer (Europe June to August, the south in its own
  spring and summer): the fire's season follows each hemisphere's own
  late summer, not a global calendar (design 2);
- even the most fire-prone region, Africa, burns only about 6 to 8% of its
  land in a year, and a burn rate is not cumulative: a region of the game
  sees only a small share burn, with no runaway loss (the limits below,
  lightning's quiet spell in design 6, and a burnt cell's fuel gone until
  its grass is whole, design 8);
- ignitions are people (an unattended fire, a burn out of control) or
  lightning, and containment matters: the torch and lightning ignitions,
  and the breaks of design 7;
- global burned area has fallen about a quarter in recent decades, mostly in
  grassland, where farming broke the fuel up: tilled fields, paths, roads
  and grazed pasture are fuel breaks (design 2's `grazed`, design 4 and 7).

What changes:
- **A per-cell plant dryness**, `plant_dry` 0..1, derived from the plant's
  kind, its season, the days since rain (`ground_wet`), the air's humidity
  and warmth, shade under trees, water nearby, the ground's hollows and
  rises, the soil's own patches, grazing, and snow (design 2, with its
  table of examples and each factor's source in the code).
- **Green plants do not burn**: below 0.5 a cell never catches, and above it
  the hazard is the square of the margin (design 2).
- **Slower spread** (design 3): grass 0.20 a second at full dryness (it was
  0.25 at danger 1 before the dryness, which now scales it to a few
  hundredths on most cells), trees 0.02 to 0.04, weaker wind and slope
  terms (design 5).
- **No ember throw** for now (design 5, 7).
- **Crown fire** only in conifers at `plant_dry` 0.85 or more in 6 m/s or
  more of wind (design 3).
- **It burns itself out** when its front reaches green or wetter cells
  (design 5).
- **Tight limits**: 2 live blazes, 300 cells and 40 burning in each
  (design 5).

**Worked examples**, run by the rule itself (a seeded script of design 2, 3
and 5 on a patch of fields 90 cells across: grass with one tree in eight,
one cell in twelve a shrub, a river, the soil patches and hollows of design
2; 24 torches lit at random cells near the middle):

| Case | What happens |
| --- | --- |
| **A typical late-summer grass fire**, fields after two dry weeks (`ground_wet` 0.12, humidity 0.32, 29 C), a 3 m/s wind | Open grass reads `plant_dry` 0.54 to 0.89 (the 10th to 90th percentile), green round the river, the trees and the hollows. Of 24 torches: 7 burn their cell and at most one more and go out within 30 s; 2 take 10 to 30 cells and go out in about a minute; 4 run 200 to 260 cells and go out on their own at green ground in 2 to 3 minutes; 11 reach the 300-cell limit in 2.5 to 4 minutes and burn out there. Its front runs about 0.2 m/s downwind (45 m) and half that upwind (30 m); at most about 40 cells burn at once. A full-size fire is 300 cells, about 2,100 m^2 (0.2 ha), a scar about 50 m across, out within 5 minutes of world time. |
| The same field, calm | About the same size, a rounder scar, fronts 0.1 to 0.2 m/s. |
| The same field, 8 m/s wind | Fronts about 0.25 m/s downwind and almost none upwind; still 300 cells at most. |
| **The same torch on spring-green grass** (`ground_wet` 0.5, humidity 0.6, 16 C) | Every grass cell reads `plant_dry` about 0.06 (0.07 at the driest): nothing catches. The torch makes its 3-second flame and nothing is saved. |
| Midsummer, the same field | Open grass about 0.42 and the driest rises just under 0.5: nothing catches. |

Without the cell limit the same 24 late-summer torches would have burned
up to about 1,100 cells (0.8 ha) at the most; the limit, not the weather,
keeps the biggest small.

### 13. What the design left open, decided for the build (2026-11-27)

Every item here is a recommendation taken (ask only with screenshots); none
is the owner's own words.

- **Where `ground_wet`'s rates live.** In `fire.ron` (`ground_wet`), the one
  source, held by the atmosphere beside its settings rather than inside
  them: the shipped settled climates are matched against
  `AtmosphereSettings` field for field, and a new field there would throw
  every one away. The field is stepped in every world (the weather is the
  world's, the same on every machine) and read only in Survival. Saved as
  `PBDATM05`, the wetness after the thermostat; an older state, and the
  shipped settled climate, restore with it at the biome's wetness.
- **Where a cell's fuel is read.** From the fine tier the player has loaded
  (each record's material word, its tree, its scatter, its water and the
  edits), through a `fire::Ground` the core asks; a cell off the tier reads
  as a break. A fire is lit by a hand or a strike in sight and is at most
  about 50 m across, so its cells are on the tier. A blaze whose origin is
  off the tier does not tick: it holds, and goes on when it is loaded again,
  so it is the same blaze whatever the player did meanwhile. (Design 5
  wanted the generator alone; a cell's tree, scatter and water are the
  record's word, which only the tier carries today.)
- **How a fire is saved.** Two kinds of record, both authored
  `@wfire:blaze/<n>`:
  - `blaze` n: its start, origin, who lit it, its state, the ticks it has
    run and the cells burning now with their time left (at most 40), so a
    reload resumes on the same tick with the same cells;
  - `burnt`, one per cell, its id the cell's key: the blaze, the second it
    burnt out, its fuel and whether its crown burned. Regrowth runs from
    each cell's own second (design 8's "the day it went out", taken per
    cell, so the cells lit first heal first).
  A player's torch writes an `ignition` record (the player's act, its own
  line); the world's blaze follows it. A record a player's line wrote is
  the player's whole, so the world could never update a blaze the player
  had written.
- **A placed torch going out and lit again** are two records, `lamp_out`
  (the world's, `@wfire:lamp/<cell>`) and `lamp_lit` (the player's), each
  counting; a torch is out while the world's count is ahead. Each author
  writes only its own, so the yield set never stops the rain putting a
  torch out a second time.
- **Being near a fire is warm** through `survival.ron`'s own fire table
  (`body.fires`, a `wildfire` row: + 20 C within 5.7 m, two cells), which
  is where every fire's warmth lives; `fire.ron`'s `harm` keeps the damage
  and the flight. Its damage is a new cause, `Cause::Fire`.
- **The burnt-cell table is the tier's own records.** In place of a new
  8,192-cell buffer: each burnt cell's record in the fine tier carries it
  in its material word, as a road does. The road bit (no tufts, no
  clutter) and the cleared bit (no generated tree) while it is bare, and a
  new ash bit (31) the ground's cap greys by. It reaches as far as the
  fine tier, which is well past a 50 m scar, and costs no new upload: the
  words go up with the records when they change. A cell's grass comes back
  between day 4 and 8 by its own roll. A burnt tree is drawn as a charred
  snag (trunk and stubs, the bark's own texture darkened, field-lit) until
  day 20, then a sapling of its own species grown to full size by day 60,
  when the generator's tree stands again.
- **A shrub** is a cell whose ground cover grows a bush (`scatter::Kind::Bush`).
- **Grazing.** A pen's cells come from the pens the husbandry finds
  (`husbandry::pen`); a cell within 2 of one is grazed.
- **Spread draws.** Each try is `hash(world seed, blaze id, from cell, to
  cell, tick)`: a cell two burning neighbours reach is tried twice, by two
  draws.
- **Torch ignition** is the held lit torch used (the use key held 1 s) on
  the plant the crosshair is on, within reach, in Survival.

## As built: the core rules (2026-11-27)

Tasks 2.1, 2.2 and 3.1 to 3.4 are built: `pbd_core::fire` (`dryness`,
`blaze`, `scar`, `torch`), `Atmosphere::ground_wet` and
`assets/config/fire.ron`. No requirement has moved to `openspec/specs/`
yet; each moves with the game code that makes it true (task 5.2).

- **The API.** `fire::FireConfig::shipped()`; `dryness::{weather, cured,
  cure, plant_dry, burn, can_catch, shade, water, hollow, patch, grazed}`
  on a `Factors`; `blaze::World { cfg, seed, ground, records, yields,
  now_s, year_fraction }` over the app's `blaze::Ground` (a cell's `Land`
  and its `Air`), with `World::{factors, plant_dry, hazard}`,
  `blaze::{ignite, tick, run, douse, lightning_starts, next_id, live,
  burnt}`; `scar::{ground_look, tree_look, burns_again, snag_gives,
  healed}`; `torch::{Soak, Exposure, open_sky, is_out, out_record,
  lit_record, out_lamps}`; `fire::harm_per_s`.
- **`fire.ron`'s shape** follows design 11 with named fields for the plant
  and fuel rows (`cure.grass`, `fuel.grass`) in place of string maps, the
  crown's terms in a `crown` row, `ground_wet`'s temperature terms
  (`dry_from_c`, `dry_span_c`) named, the snag's good, and the bucket's
  wet days. `harm` has no warmth (design 13).
- **Fronts on the test patch.** On a patch of fields with no river, hollow
  or shade (the test's), fronts ran 0.3 to 0.45 m/s calm (the farthest
  cell from the start, any way) and 0.16 to 0.37 m/s downwind in 3 m/s,
  0.3 to 0.36 in 8 m/s: under a metre a second always, but up to twice
  decision 12's 0.1 to 0.25, because the patch has none of the green
  breaks decision 12's had. Every fire on it reached the 300-cell limit and
  burnt out there within ten minutes; at most 40 burned at once. Nothing
  was retuned: the numbers are decision 12's, and the size is the limit's.

## As built: in the game (2026-11-27)

Tasks 4.1, 4.3 and the player's half of 4.4 are built (`pbd_app::wildfire`,
`desktop/fire_play.rs`); 4.2's flames, smoke, embers, light and scar are
drawn, shown in `docs/screenshots/wildfire/` (lavapipe, no GPU; no frame
time was measured, and `perf_suite.py` was not run: a cloud session).
Recommendations taken (ask only with screenshots):

- **F, not a held use.** A lit torch in hand and F on a plant lights it,
  on an out placed torch lights it again, and with the torch in hand out,
  near any fire (a burning cell, a campfire, a brazier, a lit placed
  torch), lights it again: one press does one thing, as every F does
  (`interact`'s rule), and the prompt names it ("Light grass").
- **The weather a tick reads** is the atmosphere's at the blaze's origin,
  once a tick: its cells are kilometres across, and a blaze 50 m.
- **The torch in hand going out is not saved**: it is lit again by a fire
  near, and a reload finds it lit. A placed torch out is the world's
  record and stays out across a reload.
- **An out torch** gives no light to the field and is drawn with no flame;
  its head is drawn as it was (the charred head waits on its texture).
- **The damp flame** on a plant too green to burn is drawn for
  `damp_flame_s` and saved nowhere.
- **The scar's ground** is ash until the cell's grass is back (day 4 to 8
  by its roll); the green shoots of day 2 are not drawn apart from it.
- **A burnt tree** is the tree's own model (`tree_kit`): a crown fire's
  bark alone in its own bark burnt near black; a ground fire's whole tree
  with its foot black and its leaves browned; from day 20 the tree's own
  model again, scaled from a fifth to whole by day 60, when the
  generator's tree stands in its place.
- **Smoke** is a new 4-frame pixel puff sheet (`tools/art/fire_art.py`),
  lit by the field; **flames** are the held torch's own sheet; **embers**
  are bright pixels rising and drifting downwind.
- **Danger**: the player's harm is `harm_at` (in a burning cell, within
  0.6 of a cell; beside it, within 1.5 cells), taken a whole point at a
  time through the survival server (`Cause::Fire`), and the warmth is
  `survival.ron`'s `wildfire` row. Every animal, wild, kept or hunting,
  within `flee_m` of a burning cell runs 12 m on away from the nearest
  flame at its run (`husbandry::fire_goal`); a herd's members, standing
  together, run the same way, and a kept animal stays in its pen.
- **Charcoal from a snag**: a burnt tree standing charred (`Snag` or
  `Scorched`) is a trunk the axe meets; its one stroke gives
  `snag.count` of a new good, `charcoal` (worth what coal is; as a forge
  fuel it waits on the forge's own change), and fells the tree in the
  same line (`wildfire::take_snag`), so its stump regrows as a felled
  tree's does and the snag leaves the draw. A sapling gives none.
- **The settled climate opens as before.** A shipped settled climate (a
  `PBDATM04` state) restores every field it carries to the byte; a state
  saved now writes the newer header and the wetness after it
  (`atmosphere::settled_tests::a_new_world_opens_on_the_settled_climate`).
- **Capture instrument**: `PBD_FIRE=front[:ticks] | tree[:ticks] |
  scar:<days> | tree-snag:<days> | torch-out` (`wildfire::capture_scene`),
  `tools/capture_wildfire.sh`. The shots stand on a dry mountain meadow
  found by the ignored `wildfire::tests::find_dry_grassland`: the fields
  biome is wooded everywhere (about a third of its cells stand a tree), so
  its floor sits near the 0.5 line and its fires stay one or two cells,
  as decision 12 means them to.

## Questions for the owner (with screenshots), 2026-11-26

| Id | Question | Options | Recommendation | Screenshots |
| --- | --- | --- | --- | --- |
| G9 | How far and fast a dry-season fire runs, and how it looks at night | as drawn now (decision 12: green plants never catch; a late-summer grass front about 0.2 m/s; at most 300 cells, 40 burning, out within about 5 minutes; no embers; a tree burns 90 s at its foot); smaller still (150 cells); larger (600 cells) | as drawn: the owner asked for little spread, and a fire this size is a scare and a scar, never a lost valley | `docs/screenshots/survival-mode/mockup/fire-night-front.png`, `fire-night-close.png`, `fire-night-rain.png`, `fire-day-after.png`, `fire-dryness.png`, `fire-spring-green.png` |

**Recommendations taken (ask only with screenshots)**, not the owner's own
words: fire is its own change; the player's things never burn (house fires
a follow-up, to be asked with screenshots then); a placed campfire, brazier,
torch or lantern never starts a fire; dryness from the ground's wetness in
the atmosphere, the plant and its own hemisphere's season rather than a
calendar; every number in design 2 to 12 (decision 12's direction, "Let's
not spread fire too much for now , consider lots of factors like dryness of
plants", is the owner's; its numbers are not); no ember throw for now; a
placed torch going out kept as the world's record, not an edit of the
player's block; Explorer has no forest fires and its torches never go out.

## Risks / Trade-offs

- **A fire could take a wood the player loved.** It takes at most 300
  cells (about 50 m across), seldom a wood (trees burn ten times slower
  than grass, and their shaded floor is green), heals in 60 world days, never touches the player's own things, starts only in the dry season
  and is seen from far off; the rain ends it.
- **Cost.** 40 burning cells a blaze, two blazes, is small CPU work in the
  core (`plant_dry` is a few multiplications per neighbour), and no frame time was measured in this write-up (a cloud session, no GPU). The
  flames and their light are the frame cost: measured with `perf_suite.py`
  on the owner's hardware before a number is claimed, and the light's
  cluster size chosen from that measurement.
- **Multiplayer bandwidth.** A blaze's lines are its cells (a key each) once
  a second; at the limit (40 burning) well under a kilobyte a second to clients near it.
- **The atmosphere's save grows** by one float per cell; it is additive and
  an older save restores without it.
- **`lightning-fire` is not built.** The dry strike waits for it; torches
  start fires without it.

## Verification

- Unit tests in `pbd_core::fire`: the weather's terms; `plant_dry` against
  design 2's table of examples, row by row; a cell below 0.5 never
  catching, from a torch or a neighbour; spring-green grass never catching
  anywhere on a test patch; a late-summer grass front's speed calm and in
  3 m/s and 8 m/s inside design 3's ranges; a blaze stopping at the cell and
  burning limits; a front going out at a green bank; rain stopping a front; a bucket;
  a dug line holding; the yield set holding; the same blaze from the same
  seed giving the same cells on two runs and on a reload mid-fire; regrowth
  by day; a pentagon cell's five neighbours.
- `ground_wet`: a soaked cell dries in about a world day of summer sun; an
  old atmosphere save restores.
- App: a `--capture` night shot of a blaze at its front, its scar the next
  day and at day 30; a placed torch going out in forced rain and relit.
- Multiplayer: two clients see the same burnt cells (`multiplayer`'s region
  checksums include the scar).
