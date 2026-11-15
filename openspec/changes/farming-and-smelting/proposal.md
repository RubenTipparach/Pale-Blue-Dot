## Why

The owner, 2026-11-14: "start building the farming systems into the game, as
well as the basic smelting stuff", and in the same session: "oh and cooking
and domestication of animals too". Farming is mockup M8 ("Furrows and
Greenhouses", `docs/mockups/farming.html`) and smelting is the first tier of
mockup M3 ("Forge and Fabricator", `docs/mockups/crafting.html`). Neither is
in the game: the pack holds blocks, tools and fish, and nothing a player digs
can be made into anything.

Cooking (`openspec/changes/cooking`) and animal husbandry
(`openspec/changes/animal-husbandry`, built in parallel on its own branch)
come next. They need the same two things this slice needs: items that are not
blocks, and a station that turns inputs into outputs over world time. So this
change builds those two foundations generally, and farming and the forge are
their first users.

## What Changes

- **Goods: one table of non-block items, saved by short name.** A new item
  kind, `Item::Good`, names a row of one append-only table
  (`pbd-core/src/goods.rs`): seeds, produce, coal, ore and ingots now; food,
  eggs, milk and wool later, as rows, with no further save-format change. A
  slot saves a good as `g<code>,<count>` with the row's short code
  (`gwheat,3`), never its index, so two branches adding rows cannot collide
  in a save.
- **Tilled soil.** With the shovel in hand, `E` on a grass, dirt, soil or sand
  top turns it to tilled soil (a new material drawn with the fields sheet's
  own *tilled soil* tile). Tilling grass turns up a seed of a crop that suits
  the climate there.
- **Four crops** from M8's base set: wheat, turnip, frost kale and chili. A
  seed in hand and the right button plant it on tilled soil where the climate
  suits it. It grows through M8's four stages on world time, and its stage is
  derived from when it was planted, so a farm costs no writes while it grows.
  `E` on a ripe crop harvests its produce and seeds. Crops are drawn as
  crossed pixel-art sprites, lit like the ground, faded in and out.
- **The forge.** A station block placed from the pack (the starting kit deals
  one, with some coal and seeds). `E` on it opens its panel in the
  stone-and-wood kit: pick a recipe, its inputs (fuel is an ingredient, M3's
  recommendation) leave the pack, and its outputs land in the pack when its
  time has passed in world time. The queue is a stored record on the forge.
- **Ore and coal.** A dug ore cell stays *mixed ore*, as M3 recommends for
  every existing world; one ore cell in three is a coal seam and yields coal.
  The forge smelts mixed ore into copper and iron ingots and tin ore, and
  copper and tin into bronze.
- **One generic station.** A station is a row of a table (its kind, its
  material, its recipes); recipes are data (`assets/config/recipes.ron`).
  A cooking pot or an oven is a new row, new recipes and new art.

## Capabilities

### New Capabilities
- `player/farming`: tilling, planting, growth on world time, harvest, and how
  a crop is drawn and saved.
- `player/smelting`: stations, recipes on world time, the forge, its panel,
  and how a station's queue is saved.

### Modified Capabilities
- `player/inventory`: a slot can hold a good; goods save by short code.

## Impact

- Code: `pbd-core` (`goods`, `farming`, `station`; `terrain` gains `Tilled`
  and `Forge`; `inventory` gains `Item::Good`; `dig` classes them),
  `pbd-app` (`saves/format.rs` codes and the `with` line,
  `planet_terrain.rs` render codes, `planet_surface.wgsl` two tiles,
  `desktop/digging.rs` drops, new `desktop/farm.rs` and `desktop/forge.rs`,
  `hotbar.rs` kit v3, `slots.rs` icons).
- Art: crop stage sprites and produce, seed, coal, ore and ingot icons from
  the mockups' own pixel art (`docs/mockups/farming/`,
  `docs/mockups/crafting/`), copied with provenance.
- Saves: every old save opens unchanged. New lines and codes are ones an older
  build skips as damaged (design, decision 12).
- Generation: untouched. No generator version moves.
- Frame cost: a few dozen small meshes near a farm. Not measured; the perf
  suite is the owner's.
