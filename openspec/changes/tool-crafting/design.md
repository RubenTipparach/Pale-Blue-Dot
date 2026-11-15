# Tool crafting: design

## Context

The loop's first step (CLAUDE.md, "The first game loop"). The rules come from
M3 "Forge and Fabricator":
- section 2: resource tiers;
- section 4: recipes;
- section 7: tool tiers.

The mechanism comes from `basic-smelting`: a placed station, timed recipes on
world time, and a table of goods with short codes. Recipes added here are data
on that mechanism.

## Decisions

The owner's own words are the go-ahead for the loop. Every decision below is a
recommendation taken (ask only with screenshots). Each can go into the survey
with a capture once it is built.

1. **Hands dig soft ground slowly.**
   - Soft ground is soil, dirt, grass, sand, snow and clay. Hands break it at a
     quarter of the shovel's speed: 0.25 against the stone tier's 1.0.
   - Rock, ore and wood need a tool.
   - This answers the owner's "can't dig dirt with my hand". It keeps the rule
     in `inventory-grid` decision 8 that hands draw nothing in hand.
2. **Tier 0 drops.**
   - Grass gives fibre one time in three.
   - Rock outcrops give loose stone. "Stone" here is the M3 item, not the
     `Stone` terrain block. The terrain block still needs a pick.
   - Sand and clay give themselves.
   - A felled tree gives logs. That needs an axe.

   Hands therefore reach a workbench in a few minutes of play: logs come from
   fallen branches. Decision 3 is how the first log is had before any axe.
3. **The first log.**
   - A tree drops one branch log the first time it is hit by hand. This is
     marked per tree in the world, as a stored record so it survives saves,
     and only on a world that has no axe.
   - Recommendation: one branch per tree. Rejected: an always-free first log,
     which would let the player skip the step.
4. **Recipes by hand.** These are made from the pack's craft tab, take 1 to 2
   s, and need no station:
   - planks ×4 from a log;
   - rope from fibre ×3;
   - workbench from planks ×6.
5. **The workbench.** It is a placed station: a material like the lamps,
   solid, with its panel opened by the use key. M3's recipes, at its times:
   - stone pick, shovel and axe from stone ×3, planks ×2 and rope ×1;
   - rod from planks ×2, rope ×2 and fibre ×2.
6. **Tiers.** A tier is a number on the owned tool, from 0 to 3: stone,
   bronze, iron, steel.
   - Speed by tier: 1.0, 1.6, 2.4, 3.6. Break time = hardness / speed.
   - Ore of tier `t` needs a pick of tier `t` or higher.
   - This slice crafts bronze and iron only. Steel waits for the steel
     furnace.
   - A crafted tool of a higher tier replaces the one owned. A lower one is
     refused.
7. **Durability:** none in this slice. A tool never wears out
   (`fishing-and-equipment`'s rule stays).
8. **Saves.**
   - What it adds: `owned` gains a tier digit per tool, and `-` means not
     owned.
   - Reading old saves: an old save's `owned` line (all four, no tiers) reads
     as all four at stone tier. Its `hand` line reads as before.
   - Older builds: an older build meets a tier digit as a damaged field and
     keeps what it last read.
   - New worlds start with nothing owned and empty hands.
9. **What the player sees.**
   - A Craft tab in the pack (I), listing the hand recipes.
   - A workbench panel in the stone-and-wood kit.
   - The tool picker (G) shows only owned tools, with their tier in the name
     ("Stone pickaxe").

## Risks

- **A new world could stall** if the first log is hard to get. Decision 3 and
  a test of the route cover it: a fresh world reaches a stone pick using only
  hand actions on ground within 200 m of the spawn.
- **Existing players see no change** except tier names in the picker. This is
  intended.
