# Tool crafting

## Why

The owner, 2026-11-14: "so initial game loop will be: crafting tools, fishing,
farming, herding, cooking". Crafting tools is the first step of that loop
(CLAUDE.md, "The first game loop"). Today a new world starts owning all four
tools (`inventory-grid` decision 8), so the loop has no first step. Bare hands
also break nothing. The owner noticed this the same day: "how come I can't dig
dirt with my hand".

M3 "Forge and Fabricator" (`docs/mockups/crafting.html`) already sets the rules:
- Tier 0 comes off the ground by hand: stone, flint, log, fibre, clay and sand.
- A workbench is made by hand from planks.
- Stone tools are made at the workbench.
- Metal tools come after the forge.

## What Changes

- **Hands gather tier 0.** Bare hands break soft ground (soil, dirt, grass,
  sand, snow and clay) at a quarter of the shovel's speed. They pick up loose
  stone, and fibre from grass. Hands still break no rock or ore, and fell no
  tree.
- **A recipe by hand and a workbench.** From the pack, the hands can make:
  - planks from a log;
  - rope from fibre;
  - the workbench, from planks.

  The workbench is a placed station: the generic station that `basic-smelting`
  builds, given new data. At it the player makes:
  - the stone shovel, stone pickaxe and stone axe;
  - the rod;

  each from stone or flint, planks and rope, as M3 sets them out.
- **Metal tiers through the forge.** With ingots from `basic-smelting`, the
  workbench makes bronze and iron tools. A tool's tier sets what it can break
  and how fast: M3 section 7's break time is hardness over speed, with speeds
  1.0, 1.6, 2.4 and 3.6.
- **New worlds start with empty hands and own no tools.** Existing worlds keep
  every tool they own; nothing a player has is taken away. A tool owned in an
  old save reads as a stone tool of that kind.
- **Felling trees needs an axe.** Whatever a tree gives today is unchanged.

## Capabilities

- `player/tool-crafting` (new): hands gather tier 0, the hand recipes, the
  workbench, tool tiers, and the starting kit.
- `player/inventory` (from `inventory-grid`): `Equipment` gains a tier per
  tool. A tool can be owned or not.

## Impact

- `pbd-core`: `inventory` (owned tools with a tier, the new-world kit) and
  `dig` (hardness against a tool's tier and speed, and hands).
- Data: the recipe table that `basic-smelting` introduces gains the hand and
  workbench recipes; M3's icons go in `assets/items/`.
- `pbd-app`: a hand-crafting tab in the pack, and the workbench's station
  panel (stone-and-wood UI).
- **Saves:** the `hand` line and `owned` gain a tier. Old saves read as stone
  tier, with all four owned as before.
- **Order:** built after `basic-smelting`, whose station, recipe and item
  table this change reuses. It sits beside `farming` and `animal-husbandry`
  and does not wait on them.
