# Design: farming and smelting

A first playable slice of M8 (farming) and of M3's bottom tier (smelting),
built on two general foundations that cooking and animal husbandry will use:
**goods** and **stations**. Everything open is a recommendation taken (ask
only with screenshots), marked so; nothing here is the owner's own word
except the quotes in the proposal.

## Context

- Items today: `Item::Block(Material)`, `Item::Tool(Tool)`, `Item::Fish(u16)`.
  A slot saves as `b<code>,<n>`, `t<code>,<n>` or `f<index>,<n>`
  (`saves/format.rs`). A slot field an older build does not know makes its
  whole line unreadable, and the replay skips it as damaged
  (`format::slots_of` returns `None`).
- Records (`pbd-core/src/records.rs`): a `rec` line carries a kind, an id, a
  schema and a RON body kept as text. A kind a build does not know is held as
  its text and written back, so records are the forward-compatible store.
- World time: `sun.clock.seconds`, saved in `world.ron`, advancing only while
  the world is played. A day is `DAY_S` = 2880 s (48 minutes), as M8 says.
- The ground's materials are packed four bits a layer for the GPU
  (`planet_column.rs`, `render_code`). Codes 0-13 are taken; 14 and 15 are
  free.

## Decisions

### 1. Goods are one append-only table, saved by short code

`pbd-core/src/goods.rs` holds `GOODS`, a table of rows: a short code (a
lower-case word with hyphens: `wheat`, `seed-wheat`, `coal`, `ingot-copper`),
a display name, a stack limit and an icon path. `Item::Good(Good)` carries
the row's index **in memory only**. A slot saves a good as
`g<code>,<count>`, so the index never reaches a disk.

- *Why codes, not indices:* animal husbandry adds its own rows on another
  branch at the same time. Two branches that both append index 12 would make
  every save written by one misread by the other. Two branches that both add
  a row with a different code merge as two rows, in either order.
- *Why one table rather than a variant a kind:* food (with freshness later),
  eggs, milk, wool, feed, ingots are all "a thing that stacks and has an
  icon". A new one is a row, and no save format, enum or match arm changes.
  A good that needs per-stack state later (a dish's freshness) gets it by a
  new slot field kind then, not by a new item variant now.
- The table is code rather than a RON file because rules in the core name
  goods (a crop's produce, a coal seam's yield), and a rule that names a row
  of a file it might not find is a rule that can fail at runtime. **The rows
  are marked "append here" in one block**, which is where a merge meets.
- A good whose code a build does not have is not loaded: its line is skipped
  as damaged by that build, exactly as any other unknown slot field is
  (decision 12).

### 2. Tilled soil is a material; the shovel tills with `E`

`Material::Tilled` (save code 18) is a solid block drawn with the fields
sheet's *tilled soil* tile (`biomes.json` tile 14, (2,3)) on its top, the
earth tile on its sides. It is render code 14, drawn from the fields sheet
whatever biome it sits in (M8 notes the tile exists only there).

- **`E` with the shovel in hand** on a grass, dry grass, jungle grass, dirt,
  soil or sand block with air over it turns it to tilled soil. Snow, stone,
  rock and ore do not till (M8: "rock, ice and regolith cannot be tilled").
  *M8 draws a hoe. Recommendation taken (ask only with screenshots): the
  shovel tills*, because a fifth tool widens the tool slot's saved bit mask
  (`hand` line), which an older build refuses, for a verb the shovel already
  has the shape for.
- Tilling is an ordinary edit on the durable path (`apply_edit`), whose line
  carries the seed it turned up as a drop (decision 4).
- Digging tilled soil gives a block of dirt. Tilled soil never reverts on its
  own in this slice (M8's ten untended days are deferred: it is a change the
  world makes, which wants the `world-persistence` world process).

### 3. Four crops, growth derived from the time planted

The roster is `farming::CROPS`, append-only, each naming its seed good and
its produce good:

| Crop | Climate outdoors | Days to ripe | Produce | Seeds back |
| --- | --- | ---: | ---: | ---: |
| Wheat | temperate | 6 | 3 | 2 |
| Turnip | temperate, cold | 4 | 2 | 2 |
| Frost kale | cold, temperate | 5 | 2 | 2 |
| Chili | hot | 5 | 4 | 2 |

Days, produce and seeds back are tunables in `assets/config/farming.ron`
(units in the file); which crops exist and their climates are the roster.
Rice, sun melon and glowcap wait for irrigation, thirst and greenhouses.

- **Stage** is `farming::stage(crop, planted_s, now_s)`: the fraction
  `(now - planted) / (days * DAY_S)` against M8's own thresholds (0.25, 0.6,
  1.0) gives stage 0 (sown), 1 (sprout), 2 (growing) or 3 (ripe). Nothing is
  written while a crop grows; a world opened later has simply grown, and a
  closed game grows nothing because world time does not pass.
- **Climate** is the biome's: fields, swamp and beach are temperate; desert
  and jungle hot; tundra and mountains cold; ocean none. Planting a crop
  whose climates do not include the cell's is refused with a message. This
  is the one M8 condition built; seasons, frost, water, wilting, compost and
  greenhouses are deferred (non-goals below).
- **Seeds come from tilling grass** (recommendation taken): a grassy top
  tilled turns up one seed of a crop that suits its climate, picked by the
  cell's key so the same cell always gives the same crop. Harvest gives the
  seeds back and more. The kit deals a few to start (decision 9).

### 4. A crop is a stored record; planting and harvest carry the hands

A planted cell is a record of kind `plot`, id the cell's key, schema 1, body
`(layer: L, crop: Some("wheat"), planted_s: S)`; a harvest writes the same
record with `crop: None`. The crop is named by the roster's code, not its
index, for decision 1's reason.

- **Planting and harvest move items**, so the record and the slots must reach
  the disk in one line or neither (the hotbar-rides-the-log rule of
  `inventory-grid`). That line is new: `with s0 .. s39 rec @p plot ...`, the
  forty slots after the change and then a whole record line. Its replay sets
  both. An older build reads `with` as a damaged line and skips both halves
  together.
- A plot is **valid** only while its layer is tilled soil with air over it.
  Digging the soil out from under a crop uproots it: the dig's own line and
  then a `crop: None` record (a second line; if a crash falls between them,
  the plot is invalid anyway and is cleared the next time it is tilled and
  planted).
- Placing a block into the air over a growing crop is refused.

### 5. Crops are drawn as crossed sprites, lit like the ground, faded

Each planted cell near the player draws three clumps of two crossed quads
each (M8: "a pair of crossed sprites, the way the game's ground clutter is
drawn"), 1.1 m tall, from the crop's stage sprite: nearest-sampled, cut out by
an alpha test, never blended. The meshes are `LitByField` and
`LitLikeTerrain`, so the field, the sun's cascades and every lamp light them
as they light the ground, and a farm at night is lit by its lanterns
(`lamps-and-lanterns`). A crop that appears, changes stage or goes **fades**
through the terrain's own screen-door mask (`Faded`), never in one frame
(priority 1, no pop-in). Crops are drawn within 64 m.

### 6. Stations are one generic table; the forge is its first row

`pbd-core/src/station.rs` holds `STATIONS`: a code (`forge`), a display
name, the material that is its block, and the most jobs it queues (4).
A **recipe** is data, `assets/config/recipes.ron`: a code, the station it
belongs to, inputs and outputs as `(item code, count)`, and its time in
world seconds. Item codes there are the save's own (`gcoal`, `b5` for mixed
ore), validated against the goods table and the materials at load.

- **A job** is a recipe queued at a world time. Jobs run one after another:
  each starts when it was queued or when the one before it finished,
  whichever is later, and is finished `secs` after it starts. Like a crop's
  stage, a job's progress is derived from world time and its start, and
  nothing is written while it runs.
- **Fuel is an ingredient** (M3's recommendation, taken): coal is an input
  of the recipe, consumed when the job is queued. There is no burn bar.
- A cooking pot, an oven or a campfire is a new row and new recipes; a
  station whose block is not a block of bricks (a campfire) is the cooking
  change's to draw (decision 8).

### 7. A station's queue is a stored record; queuing and collecting carry the hands

A station's state is a record of kind `station`, id `cell << 16 | layer`,
body `(station: "forge", jobs: [(recipe: "mixed-ore", queued_s: S), ..])`.

- **Queuing** takes the recipe's inputs from the slots (hotbar first, then
  the pack) and appends the job: one `with` line.
- **Collecting** happens while the panel is open: each finished job at the
  head of the queue whose outputs fit in the slots is removed and its outputs
  given by the give order: one `with` line. A job whose outputs do not fit
  waits, finished, in the queue; nothing is ever dropped.
- Breaking a station block with jobs in it is refused with a message
  ("empty the forge first"), so nothing in a queue can be lost to a pick.
- A record left on a cell whose block is no longer a station is ignored, and
  overwritten by the next station placed there.

### 8. The forge is a block of slate bricks with a chimney

`Material::Forge` (save code 19) is a solid block, render code 15, drawn with
the mountains sheet's *slate bricks* tile all round. Over it stands an iron
chimney (a small field-lit box), and while a job runs, a flame card in the
chimney's mouth, after M3's forge icon. It is broken with the pickaxe
(stone's row of the dig matrix) and gives itself back.

- *Render code 15 is the last of the four-bit space.* Every later station
  block shares code 15 and is told apart by what stands on it, or the
  material word widens; that is the cooking change's choice, recorded here so
  it is not a surprise.
- M3 says a working forge is a light. The lamp list the GPU knows is fixed at
  six kinds, so the forge's light is deferred to the change that widens it.

### 9. The starting kit, version 3

`KIT_GRANTS` gains a third grant, so every world, old or new, is dealt it
once through the existing kit path: **one forge, 8 coal, 4 wheat seeds and 4
turnip seeds**. The grant table widens from materials to items to do it. A
new world's ten hotbar slots are full, so the grant lands in the pack.

### 10. Ore: mixed ore and coal seams, derived at the dig

A dug ore cell yields **mixed ore**, the `Ore` block it always has (M3: an
existing world's ore is mixed ore, smelted at a fixed split), **except that
one generated ore cell in three is a coal seam** and yields coal, picked by a
hash of the cell's key and layer. An ore block a player placed (an edit
holds it) always yields mixed ore again, so placing and digging cannot farm
coal. No generation changes: which cells are ore is untouched.

The forge's recipes (M3's numbers where it has them):

| Recipe | Inputs | Outputs | Time |
| --- | --- | --- | ---: |
| `mixed-ore` | 4 mixed ore, 2 coal | 2 copper ingots, 1 iron ingot, 1 tin ore | 8 s |
| `bronze` | 1 copper ingot, 1 tin ore, 1 coal | 2 bronze ingots | 5 s |

The split is M3's "2 copper : 1 tin : 1 iron per 4", with copper and iron
coming out as ingots and tin as ore for bronze.

### 11. The panel is the stone-and-wood kit's

`Screen::Forge`, opened with `E` on a forge and closed with Escape or `E`:
the loading screen's rock panel, the oak lintel ("FORGE"), one stone plaque a
recipe naming it, a row of input and output icons with counts beside each,
the kit's progress bar for the job at the head of the queue, and a line
saying how many jobs wait. A plaque whose inputs the slots do not hold, or
whose queue is full, is drawn disabled. The hotbar stays drawn under it.

### 12. Save compatibility

- **Old saves open unchanged.** Nothing is migrated: no line, code or record
  an old save holds changes meaning. The new kit grant is dealt once by the
  existing kit path, which is how every earlier grant reached old worlds.
- **New material codes** 18 (tilled) and 19 (forge), **the `g` slot prefix**,
  **the `with` line**, and records of kind `plot` and `station` are all
  additions. An older build opening a newer save: a `rec` line of an unknown
  kind is held as text and kept (records' own rule); any line holding an
  unknown material, an unknown slot prefix or a `with` head is skipped as
  damaged, so it loses those edits and that inventory, and nothing else.
  That is what every earlier addition (lamps, fish, drops) did.
- The identity file notes the two record kinds at schema 1 the first time
  each is stored (`note_record_kinds`).

### 13. What is derived and what is stored

| Thing | Stored | Derived |
| --- | --- | --- |
| Tilled soil | the edit | |
| A crop | its plot record: code, layer, time planted | its stage, from world time |
| A seed turned up by tilling | the drop on the till's line | which crop, from the cell's key and climate |
| A station | its block (edit) and its queue record | each job's progress and whether it is done |
| Coal from ore | the drop on the dig's line | coal or ore, from the cell's key |

## Non-goals (deferred, and why)

- Seasons, frost, watering, wilting, irrigation, compost, greenhouses, rice,
  sun melon, glowcap, untended soil reverting: each is a rule M8 states that
  needs state written over time or a world process. The plot record leaves
  room (it is a RON body with a schema).
- Towns farming on their own; crops per planet (M8's roster per body): one
  body has crops today.
- The workbench, kiln, loom and every recipe past the forge's two; tool
  tiers; crafting a forge (it is dealt). The station table and recipe file
  take them as rows.
- The forge as a light (decision 8).
- Ore veins by kind: a generator version for new worlds (M3).

## Risks

- Render code 15 is the last: decision 8.
- `E` also opens doors in towns. The farm's `E` acts only on a tillable,
  tilled or forge cell the eye ray meets, which a door is not.
- A crop's stage jumps with the `L` debug key (noon or midnight), as the sky
  does. That is the debug key's nature.
