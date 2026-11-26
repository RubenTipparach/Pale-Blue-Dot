# Proposal: early power, windmills, water mills and the first lines

Status: **planning only**. Nothing here is built. The mockup is M33's POWER
tab (`docs/mockups/mission-tree.html#power`, the tree and its recipes) and its
MILLS tab (`#mills`, a windmill on a rise and a water mill on a river joined
by a pole line, by day and lit at night). The game code waits on the owner's
look at the mockup and a request to build it.

## Why

The owner, 2026-11-26, verbatim:

> draft up a reasearch tree mockup (addon to current research tree mockups)
> for power generation: windmills - for processing food; water mills for
> generating hydro power - this can power some appliances once they are
> built; power lines to carry electricity, no battery storage for awhile

and, the same day: "phase 4 is two parts that can be worked on in parallel:
infrastructure and survival". Early power is phase 4's infrastructure part,
beside town building and civil works (CLAUDE.md, "The first game loop";
power generation's first rungs, water and wind, were bumped to phase 4 on
2026-11-21).

M20 (`industry-tiers`, `docs/mockups/industry.html`) already draws the later
power ladder: wind turbines and hydro dams at tier II, coal and oil at tier
I, then fission and on, with wire on wood poles 4 cells apart. It has no
rung below it: nothing a player with an iron axe and a forge can build. This
change is that rung, and it leads into M20 without contradicting it.

## What changes

- **A tree of its own, POWER**, in the Missions screen beside the starter
  and town trees: a trunk (Millwright) and four branches (Wind, Water,
  Lines, Appliances), 17 missions, whose rewards unlock the recipes and
  stations below. It opens when the starter tree's "The iron age" is done.
- **The windmill** grinds food: wheat into flour, flour into a loaf at the
  campfire, wheat and turnip into feed mash for the herd, and it drives a
  grinder set on its floor. It needs wind (the weather's wind at the place)
  and a rise. It makes no electricity in this tier.
- **The water mill** stands on flowing river water and turns a dynamo: hydro
  power, a few kW, by the river's flow.
- **Power lines**: copper wire on wood poles 4 cells apart (M20's spacing),
  carrying the mill's power to appliances. One network per joined run.
- **No batteries.** Power is used as it is made. When the wheel stops (the
  sluice shut, the river low or frozen) every appliance on the line stops
  and its lamps go dark; when demand is above supply everything on the line
  slows by the same share and lamps dim.
- **The first appliances**, proposals: an electric lamp, a motor for the
  grinder and the mixer (`townsfolk` design 41's stations), and a bellows
  for the forge.
- **New goods and blocks**, proposals: wooden gear, axle, millstone, flour,
  loaf, feed mash, copper wire, dynamo, power pole, electric lamp, motor,
  bellows; the windmill and the water mill.
- **Config**: `assets/config/power.ron` (proposed shape in design 8), every
  number a first guess with its unit.

## What does not change

- Nothing in the game until the owner asks. No save changes: the tree's ids
  are a new record (`power_missions`), append-only, absent from every
  existing world.
- M20's ladder: its wind turbines, hydro dam and pole lines keep their
  numbers. This tier's line is a light line on the same poles that M20's
  wire later replaces.

## Impact

- `assets/config/power.ron` (new), `recipes.ron` (new rows),
  `power_missions.ron` (new), `locale_en.yaml` (names and help lines).
- `pbd-core`: a power network (`power.rs`) beside `station.rs`; the
  windmill's and water mill's rules read `wind::wind_at` and the river's
  flow (`water-flow`, `log_drive`'s water sample).
- `pbd-app`: the two mills, the poles and the line as lit, textured models
  (`field_light::LitByField`, `LitLikeTerrain`).
- Mockup: M33 version 10, POWER and MILLS tabs.
