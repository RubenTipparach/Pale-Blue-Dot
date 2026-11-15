# Tool crafting: design

## Context

The loop's first step (CLAUDE.md, "The first game loop"). The rules come from
M3 "Forge and Fabricator":
- section 2: resource tiers;
- section 4: recipes;
- section 7: tool tiers.

The mechanism comes from `farming-and-smelting` (built on `claude/phase-1`):
a placed station, timed recipes on world time, a table of goods with short
codes, the `hand` station row and `tool:<name>` recipe outputs it left as
hooks (its decision 6), and the `Gained` message (its decision 15). Recipes
added here are data on that mechanism.

What the farming branch changed under this plan, and what follows:
- **The forge already exists, and is dealt.** Kit v3 gives every world, old
  and new, one forge, coal and seeds. Decision 10 says what happens to it.
- **The station panel exists** (`Screen::Forge`), and was built for the forge
  alone. It becomes the panel of whichever station is open.
- **The four-bit GPU material word is full** (codes 14 and 15 went to tilled
  soil and the forge). The workbench draws without widening it (decision 5).
- **The owner asked for crafting in the pack** the same day (M29 in
  `docs/mockups/ui.html`: "the pack should also include some kind of basic
  crafting system, with a drop down list, and a button to look at crafting
  trees/recipes"). The Craft tab of decision 9 is that row.

## Decisions

The owner's own words are the go-ahead for the loop. Every decision below is a
recommendation taken (ask only with screenshots). Each can go into the survey
with a capture once it is built.

1. **Hands dig soft ground slowly.**
   - Soft ground is soil, dirt, grass (all three), sand, snow and tilled soil:
     the dig matrix's dirt row. Hands break it at a quarter of the stone
     shovel's speed: `hand_speed` 0.25 against the stone tier's 1.0, so dirt
     takes 0.5 / 0.25 = 2.0 s.
   - Hands also take up a placed light (a torch, a lantern) at the same
     quarter speed: a player who stood a torch in the wrong place is not
     stuck with it.
   - Rock, ore and wood need a tool. The workbench is wood.
   - This answers the owner's "can't dig dirt with my hand". It keeps the rule
     in `inventory-grid` decision 8 that hands draw nothing in hand.
2. **Tier 0 drops.** Each is a find on the dig's own line (the dug block
   floats out as before; the find goes straight into the slots), picked by a
   hash of the cell and layer so the same cell always answers the same, and
   never off a block a player placed, so placing and digging cannot farm it:
   - Dug grass gives **fibre** one time in three.
   - Dug dirt, soil and sand give **loose stone** one time in three: the
     pebbles in the earth. "Stone" here is the M3 item (`stone`, "loose
     stone"), not the `Stone` terrain block, which still needs a tool.
     *Changed from the first draft*, which had rock outcrops give it: hands
     break no rock, so a hand could never have had it.
   - Clay and flint are not built: the ground has no clay or flint to dig
     (they are a generation change, for new worlds).
3. **Logs from trees.**
   - A tree's trunk is aimed at like a block, and the dig button held on it
     works it. By hand, a tree gives **one branch log**, once. With an axe it
     gives logs up to `axe_logs` (4) in all, counting the branch.
   - What a tree has given is a stored record, kind `tree`, id the cell's
     key, body `(logs: N)`, saved with the slots in one `with` line, so it
     survives saves and a tree is never stripped twice.
   - **The tree stays standing.** Trees are drawn by the GPU from the cell's
     seed and nothing on the CPU removes one today; felling (the tree going,
     a stump, regrowth) is deferred to the change that can draw it. "Whatever
     a tree gives today is unchanged": it gave nothing, and still stands.
   - Which cells stand a tree is the foliage pass's rule
     (`planet_visibility.wgsl`, `has_nearby_foliage`), mirrored on the CPU in
     `pbd_core::flora::grows_tree`, with a test that reads the shader file
     and holds the two together.
   - Recommendation: one branch per tree. Rejected: an always-free first log,
     which would let the player skip the step.
4. **Recipes by hand.** Made from the pack's Craft row (decision 9), at M3's
   times, with no station. The hands' queue is a `station` record like a
   forge's, with the id `u64::MAX` (no cell's id reaches it), one job at a
   time, collected while the pack is open:
   - planks ×4 from a log, 1 s;
   - rope from fibre ×3, 2 s;
   - a workbench from planks ×6, 4 s.
5. **The workbench.** `Material::Workbench`, save code 20, a solid placed
   station. Its panel opens with the use key, like the forge's.
   - **Drawn without widening the GPU word:** its render code is 8, the
     local biome's tree wood (the trunk's own tile), so it is a block of the
     same timber the trees stand in; a plank top and an iron vise stand on
     it, as the forge's chimney does (field-lit boxes, faded in and out).
   - It is dug like wood: the axe is fastest; hands do not take it.
   - Its recipes, at M3's times:
     - stone pickaxe, shovel and axe from loose stone ×3, planks ×2, rope ×1, 3 s;
     - the rod from planks ×2, rope ×2, fibre ×2, 3 s;
     - a forge from loose stone ×8 and stone blocks ×4, 6 s (M3's
       "stone ×12, clay ×4" without the clay the ground does not have);
     - bronze pickaxe, shovel and axe from bronze ingots ×3, planks ×2, 4 s;
     - iron pickaxe, shovel and axe from iron ingots ×3, planks ×2, 4 s.
6. **Tiers.** A tier is a number on the owned tool, from 0 to 3: stone,
   bronze, iron, steel.
   - Speed by tier: 1.0, 1.6, 2.4, 3.6 (`dig.ron`, `tier_speeds`).
   - Break time = the dig matrix's time for the tool and the material's row,
     over the tier's speed. The matrix is the stone tier, which is what every
     tool was until now, so an old world digs exactly as it did. (M3 writes
     it as hardness over speed; the matrix is the per-tool hardness.)
   - Ore of tier `t` needs a tool of tier `t - 1` or higher (M3's reach:
     stone takes tier 0-1, bronze 2, iron 3, steel 4). The ground's one ore,
     mixed ore, is tier 1, so every tool takes it today; the gate is in place
     for the ore veins of a later generator.
   - This slice crafts bronze and iron only. Steel waits for the steel
     furnace.
   - A crafted tool of a higher tier replaces the one owned. One of the same
     or a lower tier is refused when it is queued, before anything is taken.
     A finished job whose tool is no longer an upgrade (made twice across two
     stations) is collected and given nothing: never stuck in a queue.
7. **Durability:** none in this slice. A tool never wears out
   (`fishing-and-equipment`'s rule stays).
8. **Saves.**
   - **What it adds:** the `hand` line's owned field becomes four characters,
     one per tool in the picker's order, each a tier digit or `-` for not
     owned, in `Tool::ALL` order (rod, shovel, pickaxe, axe): `hand - 0-2-` is
     bare hands, a rod and an iron pickaxe. The old field was a bit mask of one
     or two digits, so the two can never be confused.
   - **A tool collected** writes the `hand` line first and then the
     station's `with` line. A crash between them leaves the tool owned and
     the job still queued; the next collect finds the tool is no longer an
     upgrade and clears the job. Nothing is ever lost.
   - **Reading old saves:** an old `hand` line reads as those tools at the
     stone tier. A world that never wrote a `hand` line opens with all four at
     the stone tier, as it always did.
   - **New worlds** start owning nothing, with empty hands. The world's
     identity file records it: `tools: 1`, written by this build for a world
     it makes; a world without it (every world made before) is an old world.
   - **Older builds:** they meet the new `hand` field as a damaged line and
     keep what they last read; a world that only ever wrote new `hand` lines
     opens in an older build with all four tools. An older build never takes a
     tool away. They ignore `tools` in the identity file, the `tree` record
     kind is held as text, and lines holding code 20 or the new goods are
     skipped as damaged (`farming-and-smelting` decision 12).
9. **What the player sees.**
   - **The pack's Craft row** (M29): below the hotbar, a drop-down of the
     hands' recipes (a recipe short of something is dimmed), the wells of
     what it takes lit or dark against what is carried, an arrow, the result,
     and CRAFT. A job's bar fills while the hands work, and the result lands
     by the give order. The pack's rock grows to M29's 268 × 204. M29's
     RECIPES tree panel is deferred: the row is the loop's need.
   - **The station panel** serves the forge and the workbench from one
     builder: the open station's lintel and recipes. The workbench's
     eleven recipes stand in two columns on the wider rock M29 rendered for
     the chest (372 × 168).
   - **The tool picker (G)** shows only owned tools, each named with its tier
     ("Stone pickaxe", "Iron axe"). With nothing owned it offers bare hands
     alone, which now "digs soft ground".
   - **Notes under the crosshair** say why nothing happens: "Hands cannot
     break rock: make a pickaxe", "This tree has given its branch: an axe
     takes more".
10. **No free stations; the kit after this change.** The phase-1
    coordinator's decision (2026-11-14), because `starter-missions` needs
    earned steps ("make fire", "find seeds"): a new world is dealt no
    station and no seeds.
    - **Kit v3** (farming's forge, 8 coal, 4 wheat and 4 turnip seeds) is
      emptied: the forge is made at the workbench (decision 5), seeds are
      found in dug grass and wild plants (`farming-and-smelting` decision 14).
      Coal goes with the forge it fed. Recommendation taken: the seeds go too
      (the mission is "find seeds x4", and grass turns one up in four cells).
    - **Kit v5** was held for this change and stays empty: tools are made.
    - **Kit v6** (cooking's free campfire) is emptied: the hands make one
      from loose stone x5 and logs x2 (`cooking.ron`'s `campfire` recipe,
      which was stone blocks and coal until there were logs).
    - **Saves:** the versions stay, in order, so the kit path deals nothing
      twice and skips nothing. No save outside the phase-1 branches was ever
      dealt v3 or v6 (none of it reached main), and a save that was keeps
      what it holds: nothing is taken away. A main-branch save (dealt v2)
      opening this build is dealt the pen's pieces (v4) and no station.
    - The pen's fences and gate (v4, `animal-husbandry`) are not a station
      and stay.
11. **Events.** Every collected craft raises `Gained { item, count, how: Made }`
    for each output, a tool as `Item::Tool`; fibre, loose stone and logs raise
    `Gained { how: Found }`. `starter-missions` counts them.

## Risks

- **A new world could stall** if the first log is hard to get. Decision 3 and
  two tests cover it: the route (a stone pickaxe from hand actions alone, in
  rules) and the ground (at least three trees, grass and earth within 200 m of
  the default spawn, measured off the generator and the tree rule).
- **Existing players see no change** except tier names in the picker. This is
  intended.
- **A tree is not felled.** A player who takes four logs sees the tree stand
  on; the note says it has given its wood.
- **The tree rule is in two places** (the shader and the core). A test reads
  the shader's own numbers.
