# Proposal: infrastructure-missions

Status: **planning only**. The write-up and M33's INFRASTRUCTURE tab
(`docs/mockups/mission-tree.html#infrastructure`; `#power` still opens it).
No game code, no data file and no save changes until the owner has looked at
the tab and asks for the build.

## Why

The owner, 2026-11-27: "where are missions for infrastructure and power".

Phase 4 is two parts worked in parallel (owner, 2026-11-26: "phase 4 is two
parts that can be worked on in parallel: infrastructure and survival").
Survival has its missions (`survival-mode` design 18, M33's SURVIVAL tab).
Infrastructure, phase 4a (town building, civil works and early power;
CLAUDE.md, "The first game loop"), has only the early power tree
(`early-power`, 17 missions on M33's POWER tab) and no mission at all for
growing a town or for civil works.

The same day, on the first draft of this tree: "Maybe move roads over to
this section". The town tree's Roads branch (the grinder, crushed stone, the
mixer, gravel, survey stakes, paving, joining a town) is road building, which
is civil works; it moves here, with a new mission for a paved road between
two towns.

## What changes

- **A third tree, INFRASTRUCTURE**, in the Missions screen beside STARTER
  and TOWN: its own file
  (`assets/config/infrastructure_missions.ron`), its own record
  (`infrastructure_missions`), its own tab. 37 missions in a trunk and four
  branches:
  - **Stoneworks** (the trunk, moved from the town tree): build a grinder,
    crush stone, build a mixer, mix gravel;
  - **Roads** (moved from the town tree, plus one): mark a road, pave it,
    join a town, and **Town to town**, a paved road between two towns;
  - **Civil works** (new): a mason's bench, a well, a retaining wall, a
    culvert, a stone bridge, a ferry, a mill race;
  - **Town** (new): lay a plot, raise a building from a blueprint, a street
    to its door, the building a town asks for, a town that grows a tier;
  - **Power** (early-power's 17 missions, folded in unchanged): Millwright,
    then its Wind, Water, Lines and Appliances arms.
- **It opens when the town tree's "A job from the board" is done**: the end
  of the town tree's trunk, where the Roads branch hung until now (design 1
  says why). No new cross-tree gate is needed to open it; the Power branch
  keeps the one hidden gate early-power already had (The iron age).
- **The town tree loses its Roads branch** (7 missions). Its Carts branch,
  which hung off "Pave it", hangs off "A job from the board", and the wheel's
  recipe moves with it (design 4).
- **Early power is no longer a tree of its own**: its missions, recipes and
  numbers are this tree's Power branch (`early-power` design 1 and 2 say so
  now). Its planned record `power_missions` is never written; nothing was
  built, so nothing migrates.
- **Saves keep every road mission a player has done**: on open, a world
  whose `town_missions` record holds any moved mission writes the
  infrastructure record with those missions done and their counts carried,
  and coins already paid are never paid again (design 5).
- **New objective kinds**: `site`, `raise`, `works`, `ask`, `grow`, and new
  things on `pave` (`towns`, `street`) and `travel` (`ferry`); the power
  kinds early-power listed (`rise`, `grind`, `drive`, `flow`, `power`,
  `pole`, `line`, `sluice`).
- **New goods, stations and blueprints**, proposals: the mason's bench,
  dressed stone, mortar, and the civil kit's blueprints (well, retaining
  wall, culvert, stone bridge, ferry, mill race, market stall).
- **M33 version 12**: the POWER tab becomes INFRASTRUCTURE (`#infrastructure`,
  with `#power` redirecting to it), the whole tree with Power as a branch;
  the TOWN tab no longer shows roads. MILLS stays.

## What does not change

- The game, until the owner asks: `town_missions.ron` and the town tree's
  code stay as they are (another change is editing `missions.ron` now); the
  move is task 2 of this change.
- Early power's missions, recipes and numbers.
- No player progress is lost: every moved id keeps its name, and the town
  record's entries for them stay in the file, unread.

## Capabilities

### New Capabilities

- `player/infrastructure-tree`: the infrastructure tree, when it opens, its
  branches, the roads moved from the town tree and how a save keeps them.

### Modified Capabilities

- None in `openspec/specs/`. `town-missions-tree` (design 13) and
  `early-power` (design 1, 2 and 12) are in-flight changes and carry notes.

## Impact

- `assets/config/infrastructure_missions.ron` (new); `town_missions.ron`
  loses the Roads branch and moves `wheel`; `locale_en.yaml` keys
  `infra_mission.<id>.*` (the moved missions' words move from
  `town_mission.<id>.*`).
- `pbd-core`: a third `Chain` beside the starter and town chains, counted by
  the same `Progress::record`; the new kinds; the open-time migration from
  the town record (design 5).
- `pbd-app`: the Missions tab's third tree tab; the HUD's one focus across
  every tree.
- Depends on unbuilt systems for some branches (design 6 lists them): towns
  that grow on the grid (M15, M22), early power, the civil kit's blueprints.
  Stoneworks and Roads are in the game today and move as they are.
