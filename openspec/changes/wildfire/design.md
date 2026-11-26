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

## Goals / Non-Goals

**Goals:**
- Fires that start only when and where it is dry, read before they are
  dangerous, and are stopped by weather, water and the player's work.
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

### 2. How dry it is: the ground's wetness and the fire danger

- **`ground_wet`**, 0..1, one value per atmosphere cell, a new state of the
  atmosphere, stepped with it:
  - rain adds 0.1 for each millimetre that falls (10 mm soaks it);
  - snow lying keeps it at 1;
  - it dries by 0.04 an in-game hour (120 s of play) x (0.5 + sunlight /
    1000 W/m^2) x (1 - humidity) x clamp((air_c + 5) / 30, 0, 1).
  So soaked ground under a dry summer sun needs about a world day to dry
  enough to burn, and a damp, cool place may never.
  - The sea's cells stay 1.
  - Saving: an additive field of `Atmosphere::to_bytes`. A save from before
    restores with `ground_wet` at its spun-up value (the biome's wetness),
    so no world changes under a player.
- **The fire danger**, `danger` 0..1, at a place:
  `danger = (1 - ground_wet) x dry(humidity) x warm(air_c) x biome`, where
  - `dry(h)` is 1 at humidity 0.35 or less, falling smoothly to 0 at 0.85;
  - `warm(t)` is 0 at 0 C or less, rising smoothly to 1 at 25 C;
  - `biome` is fields 1.0, desert 1.0 (its fuel is sparse), mountains 0.8,
    tundra 0.4, jungle 0.3, swamp 0.1, beach 0.6, ocean 0.
- **"The dry season"** is wherever `danger` is 0.35 or more. It comes with
  the season where the flora's straw is high and the rain stops, and goes
  with the rain: it is the weather's own, so it moves with the climate and
  differs from valley to valley, which a calendar date would not.
- **Shown**, in Survival only, as a small flame beside the HUD's
  thermometer while the player stands where `danger` is 0.35 or more, and as
  a layer of the world map's weather overlays (`weather-overlays`). No text.

### 3. Fuel: what each cell has to burn

Derived from the world's own generator version and records, never saved:

| Fuel | Where | Spread, per second to each neighbour, at danger 1 | Burns | Then |
| --- | --- | --- | --- | --- |
| Grass | a grass, dry grass or jungle grass top | 0.25, x (0.4 + 0.6 x straw) | 12 s | ash |
| Shrub | ground cover with a bush (`scatter`) | 0.12 | 30 s | ash |
| Tree | `grows_tree`, not felled | 0.05 (a conifer 0.07) | 120 s: 60 s crown, 60 s embers | a charred snag |
| Litter | a deciduous tree's cell in the season's litter (`SEASON_LITTER`) | x 1.3 on grass | - | - |
| None | water, sand, rock, snow or ice, soil without grass, a road, a cactus | never | - | a break |

Spread is a hazard per second; in a one-second tick a neighbour catches with
`p = 1 - exp(-lambda)`. On grass at danger 1 with no wind that is a front of
about 1 m/s; downwind in 5 m/s about 2.5 m/s; through a forest about a
fifth of that, faster when the crowns burn in the wind. All starting numbers
for the owner to tune (`assets/config/fire.ron`, design 11).

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

- **A blaze** is a record, kind `blaze`, id n, schema 1:
  `(started_s, origin: cell key, lit_by: Player(n) | Lightning,
  state: Burning | Out(day))`. Its cells are its lines: each tick's newly
  lit and newly burnt-out cells are one durable line authored
  `@wfire:blaze/<n>`, so the fire is saved as it goes, never on exit.
- **The tick** is 1 s of world time. For each burning cell, in cell-key
  order, for each of its 5 or 6 neighbours (the topology's explicit tables,
  pentagons included) with fuel, not burning, not burnt and not in the
  yield set, a hazard `lambda = spread(fuel) x danger(neighbour) x wind x
  slope x rain` is tried against `hash(world seed, blaze id, cell, tick)`.
  No hash map's order and no GPU result decides anything.
  - `wind` = 1 + 0.25 x speed x max(0, cos) - 0.15 x speed x max(0, -cos),
    at least 0.2, `cos` the angle between the wind and the step;
  - `slope` = 1 + 0.15 per layer the neighbour stands higher, 0.7 lower;
  - `rain` = exp(-rain_mm_h / 0.5).
- **Weather is sampled** from the server's atmosphere at each burning cell
  once a tick. The fuel comes from the generator functions, so a blaze far
  from every player runs without loaded terrain, and only while the world
  is played (world time).
- **Limits**: 4 live blazes in a world, 20,000 cells (about 14 ha) and 1,500
  burning at once in each. At a limit the front stops spreading and burns
  out, and the line says so. A fifth ignition while four burn is a short
  local fire (design 6).

### 6. Starting one

In a Survival world, where `danger` >= 0.35:
- **A lit torch** used (hold use, 1 s) on a grass, shrub or tree cell lights
  it. It is the player's act, in the same durable line as the record's
  creation.
- **Lightning**: a `lightning-fire` strike whose cell has less than 0.5
  mm/h of rain and `danger` >= 0.5 starts a blaze lit by `Lightning`. A wet
  strike keeps `lightning-fire`'s short fire and saves nothing.
- **Never** from a placed campfire, brazier, torch or lantern: a placed fire
  is contained (recommendation taken), so the player's own camp never lights
  the dry grass it stands in.
- Below the threshold, a torch on grass makes a small flame that dies in 3
  s and saves nothing; no note says why (CLAUDE.md, "Game text is plain").

### 7. Stopping one

- **Rain**: the `rain` term slows the spread; and every burning cell's time
  left shrinks by `rain_mm_h x 0.5` s a second, so 4 mm/h puts grass out in
  about 3 s and a tree in about 20 s. At 2 mm/h or more no cell catches.
- **Water**: a water cell, and a cell beside fresh or sea water, does not
  burn. A water bucket poured (`water-field`'s `pour`) on a burning cell puts
  out it and its neighbours, and leaves them wet for a world day.
- **Breaks**: bare ground. Digging a line of grass to soil, a road, a path,
  a paved cell, sand or rock stops a grass fire; a tree fire in wind can
  throw embers a cell over (a second chance, at a fifth of the hazard, to a
  cell two away downwind).
- **It ends** when no cell burns. The record becomes `Out(day)`.

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

```ron
// Fire (pbd_core::fire; openspec/changes/wildfire). Seconds of world time,
// metres, mm/h of rain, deg C, world days of 48 minutes. Survival worlds only.
(
    tick_s: 1.0,
    ground_wet: (per_mm: 0.1, dry_per_hour: 0.04, sun_ref_wm2: 1000.0),
    danger: (humid_dry: 0.35, humid_wet: 0.85, warm_from_c: 0.0, warm_full_c: 25.0,
             ignite: 0.35, lightning: 0.5, lightning_rain_below_mmh: 0.5,
             biome: {"fields": 1.0, "desert": 1.0, "mountains": 0.8, "tundra": 0.4,
                     "jungle": 0.3, "swamp": 0.1, "beach": 0.6, "ocean": 0.0}),
    fuel: {
        "grass": (spread_per_s: 0.25, burn_s: 12.0, straw_floor: 0.4),
        "shrub": (spread_per_s: 0.12, burn_s: 30.0),
        "tree": (spread_per_s: 0.05, conifer_per_s: 0.07, crown_s: 60.0, ember_s: 60.0),
        "litter_mult": 1.3,
    },
    wind: (down_per_mps: 0.25, up_per_mps: 0.15, floor: 0.2),
    slope: (up_per_layer: 0.15, down: 0.7),
    rain: (spread_scale_mmh: 0.5, douse_s_per_mmh: 0.5, no_catch_mmh: 2.0),
    ember_throw: (cells: 2, share: 0.2),
    limits: (live_blazes: 4, cells: 20000, burning: 1500),
    regrow_days: (shoots: 2.0, grass: 4.0, grass_whole: 8.0, shrub: 10.0,
                  sapling: 20.0, young: 40.0, tree: 60.0),
    torch: (placed_out_s: 20.0, held_out_s: 10.0, rain_seen: 0.3, sky_layers: 8,
            relight_s: 1.0, damp_flame_s: 3.0),
    harm: (in_cell_per_s: 4.0, beside_per_s: 1.0, feel_c: 20.0, feel_cells: 2,
           flee_m: 40.0),
    burnt_table: 8192,
)
```

## Questions for the owner (with screenshots), 2026-11-26

| Id | Question | Options | Recommendation | Screenshots |
| --- | --- | --- | --- | --- |
| G9 | How fast a dry-season fire runs, and how it looks at night | as drawn (grass front about 1 m/s calm, 2.5 m/s in a 5 m/s wind; a tree burns 2 minutes); half as fast; twice as fast | as drawn: fast enough to fear, slow enough to outrun on foot, and the rain still ends it | `docs/screenshots/survival-mode/mockup/fire-night-front.png`, `fire-night-close.png`, `fire-night-rain.png`, `fire-day-after.png` |

**Recommendations taken (ask only with screenshots)**, not the owner's own
words: fire is its own change; the player's things never burn (house fires
a follow-up, to be asked with screenshots then); a placed campfire, brazier,
torch or lantern never starts a fire; danger from the ground's wetness in
the atmosphere rather than a calendar; every number in design 2 to 11; a
placed torch going out kept as the world's record, not an edit of the
player's block; Explorer has no forest fires and its torches never go out.

## Risks / Trade-offs

- **A fire could take a forest the player loved.** It heals in 60 world
  days, never touches the player's own things, starts only in the dry season
  and is seen from far off; the rain ends it.
- **Cost.** 1,500 burning cells a second is small CPU work in the core, and
  no frame time was measured in this write-up (a cloud session, no GPU). The
  flames and their light are the frame cost: measured with `perf_suite.py`
  on the owner's hardware before a number is claimed, and the light's
  cluster size chosen from that measurement.
- **Multiplayer bandwidth.** A blaze's lines are its cells (a key each) once
  a second; at the limit a few kilobytes a second to clients near it.
- **The atmosphere's save grows** by one float per cell; it is additive and
  an older save restores without it.
- **`lightning-fire` is not built.** The dry strike waits for it; torches
  start fires without it.

## Verification

- Unit tests in `pbd_core::fire`: the danger's terms; a grass front's speed
  at danger 1 with no wind and with 5 m/s; rain stopping a front; a bucket;
  a dug line holding; the yield set holding; the same blaze from the same
  seed giving the same cells on two runs and on a reload mid-fire; regrowth
  by day; a pentagon cell's five neighbours.
- `ground_wet`: a soaked cell dries in about a world day of summer sun; an
  old atmosphere save restores.
- App: a `--capture` night shot of a blaze at its front, its scar the next
  day and at day 30; a placed torch going out in forced rain and relit.
- Multiplayer: two clients see the same burnt cells (`multiplayer`'s region
  checksums include the scar).
