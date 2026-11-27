# Design: infrastructure-missions

Everything here is proposed. Counts and coins are first guesses to tune.
Every choice the owner did not make is marked **recommendation taken (ask
only with screenshots)**; the owner judges the tree on M33's INFRASTRUCTURE
tab (`docs/mockups/mission-tree.html#infrastructure`).

## Context

- **The owner, 2026-11-27**: "where are missions for infrastructure and
  power", and, on this tree's first draft, "Maybe move roads over to this
  section".
- **Phase 4a, infrastructure** (owner, 2026-11-26: "phase 4 is two parts
  that can be worked on in parallel: infrastructure and survival"): town
  building, civil works and early power (CLAUDE.md, "The first game loop").
- **The starter tree** (`missions.ron`) and **the town tree**
  (`town_missions.ron`, record `town_missions`) are built. The town tree's
  rules: its own file, its own record, its own tab; a strict tree, one parent
  each; no mission of one tree is a parent of another's; extra needs are
  hidden `after` gates. Its trunk is Meet the townsfolk, Buy and sell, and
  **A job from the board** (`first-job`), off which every branch hangs.
- **The town tree's Roads branch is in the game**: `grinder`, `crush`,
  `mixer`, `gravel`, `mark`, `pave`, `join` (`townsfolk` design 39 to 41;
  `town_missions.ron`'s `awaited` is empty, so `stake`, `mark` and `pave` are
  raised). Its Carts branch hangs off `pave`, which unlocks `wheel`.
- **Early power** (`early-power`) planned a tree of its own, POWER, opened by
  the starter tree's "The iron age" (`iron`), with one cross-tree hidden gate
  (`town:grinder`). None of it is built.
- **The building planner is built** (`building-planner`): presets set out as
  a site, built piece by piece, the `site` and `blueprint` records. **Towns
  that grow on the grid are not** (M15, M22; phase 4).
- **Player roads bridge water** with a plank deck up to 6 cells (`townsfolk`
  design 42); a ford is a shallow crossing. There is no stone bridge, culvert
  or ferry.

## Goals / Non-Goals

**Goals**: one tree for phase 4a, with town growth, civil works, roads and
power as its branches; the roads moved from the town tree without losing a
player's progress; early power kept whole as a branch.

**Non-Goals**: building the systems the branches need (town growth, the civil
kit, early power: each is its own change); M20's power ladder past early
power; any change to the starter tree.

## Decisions

### 1. Its own tree, opened by "A job from the board"

- **A third tree**, as the town tree is: `assets/config/infrastructure_missions.ron`
  (the same `MissionFile` shape), its own `Chain`, its own record
  `infrastructure_missions`, its own tab INFRASTRUCTURE on the Missions
  screen. Ids are their own namespace.
- **It opens when the town tree's `first-job` is done.** That is the town
  tree's natural end point for this purpose: the last mission of its trunk,
  the one every town branch hangs off, and the one the Roads branch hung off
  until now. Why this one and not a later one (recommendation taken, ask only
  with screenshots):
  - the roads move here (decision 3), so a player who could start paving
    yesterday must be able to start it today: opening at `first-job` keeps
    the moved missions open at exactly the moment they were;
  - civil works and town building are town work: a player has met the town,
    traded and done a job for its board before the town asks for wells and
    bridges;
  - "after the whole town tree" (34 missions) or after "Join a town" would
    shut away the roads behind work that has nothing to do with them.
- **The opening is the tree's own rule, not a gate on a mission**, as the
  town tree's opening on `Reach town` is: the first `job` hand-in that
  finishes `first-job` writes the record and opens the root. Until then the
  tab is not shown.

### 2. The tree

A strict tree: every mission one parent (the only line drawn), hidden
`after` gates for extra needs. **New** marks an objective kind or thing the
game does not raise yet (decision 7). Status: **in the game** (moves as it
is), **today** (the systems are built; only the mission and its kind are
new), or **waits on** a named unbuilt system.

Coins: the moved missions keep theirs; Civil works and Town pay coins (the
town pays for its works); Power pays none, as early-power had it.

**0. Stoneworks** (the trunk; moved from the town tree's Roads)

| id | Mission | Parent (after) | Objectives | Unlocks | Coins | Status |
| --- | --- | --- | --- | --- | ---: | --- |
| `grinder` | Build a grinder | (tree opens) | craft and place a grinder | `crushed-stone` | 10 | in the game |
| `crush` | Crush stone | `grinder` | make 9 crushed stone | `mixer` | 10 | in the game |
| `mixer` | Build a mixer | `crush` | craft and place a mixer | `mixed-gravel` | 10 | in the game |
| `gravel` | Mix gravel | `mixer` | make 8 gravel | `survey-stakes` | 10 | in the game |

**1. Roads** (off `gravel`; moved, plus one)

| id | Mission | Parent (after) | Objectives | Unlocks | Coins | Status |
| --- | --- | --- | --- | --- | ---: | --- |
| `mark` | Mark a road | `gravel` | `stake` 2; `mark` 8 cells | | 10 | in the game |
| `pave` | Pave it | `mark` | `pave` 8 `gravel` | | 30 | in the game |
| `join` | Join a town | `pave` | `pave` 1 `joined` | | 60 | in the game |
| `two-towns` | Town to town | `join` | `pave` 1 **`towns`** (new thing) | `milestone` | 80 | today |

`towns` counts once when the player's paved road, with the world's own road
cells it meets, joins the footprints of two towns, and the player paved at
least 30 cells of that run. It is the owner's "paved road between two towns".

**2. Civil works** (off `gravel`; new)

Each work is a **civil blueprint**: a preset of the building planner's civil
kit, set out as a site and built piece by piece with the mallet, exactly as a
house preset is (`building-planner` design 1 to 7). A work is finished when
its site is (`works`). Recommendation taken (ask only with screenshots): one
way to build everything, and the planner's checks (held up, headroom, the
water's edge) already exist.

| id | Mission | Parent (after) | Objectives | Unlocks | Coins | Status |
| --- | --- | --- | --- | --- | ---: | --- |
| `mason` | A mason's bench | `gravel` | craft and place a mason's bench | `dressed-stone`, `mortar`, `bp-well` | 10 | waits on the mason's bench |
| `well` | Dig a well | `mason` | set out a well (**`site`** `well`); finish it (**`works`** `well`) | `bp-retaining-wall` | 20 | waits on the civil kit |
| `retaining` | Hold the bank | `well` | finish a retaining wall 4 cells long (**`works`** `retaining-wall`, cells) | `bp-culvert`, `bp-mill-race` | 25 | waits on the civil kit |
| `culvert` | Under the road | `retaining` | finish a culvert where a road crosses a stream (**`works`** `culvert`) | `bp-stone-bridge` | 25 | waits on the civil kit |
| `stone-bridge` | A stone bridge | `culvert` | finish a stone bridge of 3 cells or more over a river (**`works`** `stone-bridge`) | `bp-ferry` | 60 | waits on the civil kit |
| `ferry` | A ferry | `stone-bridge` | rig a ferry over a river too wide to bridge (**`works`** `ferry`); cross on it (**`travel`** `ferry`) | | 30 | waits on the civil kit |
| `race` | Cut a mill race | `retaining` (`water-mill`) | finish a mill race to a water mill (**`works`** `mill-race`); run the mill at full output (**`power`** `full`) | | 40 | waits on the civil kit and early power |

- **The well** draws fresh water: a bucket filled at it is fresh
  (`survival-mode`'s drink, `bucket`), so it is a Survival world's water in
  town as well as a work.
- **The retaining wall** holds a cut bank: the cells behind it do not slump
  when the cell in front is dug (the dig rules already keep a placed block).
- **The culvert** carries a stream under a road cell: the road stays dry and
  paved over running water, where today it fords or decks.
- **The stone bridge** spans wider than a plank deck (up to 10 cells against
  6) and never rots; it is the bridge the world's own roads may later use.
- **The ferry** is a raft on a rope across water too wide to bridge ("Too
  wide to bridge" is the paving's refusal today), pulled with F.
- **The mill race** leads river water to a water mill's wheel through a
  channel: the mill reads the race's flow (full) rather than the river's, so
  a slow river can still turn a wheel at its rated 6 kW. It is the civil
  works' link to power, gated on `water-mill` (a hidden `after` inside this
  tree). A dam stays M20's tier II.

**3. Town** (off `grinder`; growing a town on the grid; new)

| id | Mission | Parent (after) | Objectives | Unlocks | Coins | Status |
| --- | --- | --- | --- | --- | ---: | --- |
| `plot` | Lay a plot | `grinder` | set out a site from a preset beside a town (**`site`** `any`) | `bp-market` | 15 | today |
| `raise` | Raise a building | `plot` | finish a site built from a preset (**`raise`** `any`) | | 30 | today |
| `street` | A street to the door | `raise` | pave 6 cells from its door to a town street (**`pave`** `street`) | | 20 | today |
| `ask` | What the town needs | `street` | raise the building a town's board asks for, a market stall or a well (**`ask`** `any`) | | 50 | waits on town growth |
| `grow` | The town grows | `ask` | a town you built for reaches its next tier (**`grow`** `any`) | | 80 | waits on town growth |

- `plot` and `raise` count the building planner's own events (a site set
  out, a site finished) and can be built today.
- `ask` is a new kind of town job: a town that wants a building posts it on
  its board ("Build a market stall"); raising that preset inside the town's
  footprint hands it in. It needs towns that grow (M15's growth on the grid),
  which decides what a town wants next.
- `grow` is a town reaching its next tier (hamlet, village, town) because of
  what was built for it.

**4. Power** (off `grinder`; early-power's tree, folded in)

Early power's 17 missions, recipes and numbers are this branch, unchanged
(`early-power` design 2 and 3). It is drawn as four arms off its trunk,
Millwright: Wind, Water, Lines and Appliances. Its root `gears` hangs off
`grinder` (the windmill turns a grinder; the grinder is where power starts).
What changes from early-power's own tree:

| id | Was | Now |
| --- | --- | --- |
| `gears` | the tree's root, opened by the starter tree's `iron` | parent `grinder`; hidden `after: ["starter:iron"]`, the one cross-tree gate (it needs an iron axle) |
| `wind-grinder` | `after: ["town:grinder"]` | no gate: `grinder` is its ancestor |
| `motor-grinder` | `after: ["town:grinder"]` | no gate: `grinder` is its ancestor |
| `motor-mixer` | `after: ["town:mixer"]` | `after: ["mixer"]`, inside this tree |

The two recipes early-power opened with its tree (`wooden-gear`, `axle`) open
when this tree opens. Status: **waits on early power** (every mission).

**Totals**: 37 missions (4 + 4 + 7 + 5 + 17); 625 coins (the moved 140, Town
to town 80, Civil works 210, Town 195; Power none). Ids are unique within the
tree (the power branch's `lamps` and the town tree's `lamps` are in different
trees and never meet).

### 3. Roads move from the town tree (owner, 2026-11-27)

The owner, on this tree's first draft: **"Maybe move roads over to this
section"**. Roads are civil works, so the town tree's Roads branch becomes
this tree's Stoneworks trunk (the grinder, the mixer and gravel, which every
road and work is made of) and its Roads branch (stakes, paving, joining),
with Town to town added.

- **The ids do not change**: `grinder`, `crush`, `mixer`, `gravel`, `mark`,
  `pave`, `join`. Their objectives, counts, coins and unlocks are the same,
  but for `pave`'s `wheel` (decision 4).
- **What pointed at roads in the town tree, and what it points at now:**

| Town mission | Pointed at | Now |
| --- | --- | --- |
| `wheels` (Carts' first) | parent `pave` | parent `first-job` |
| `first-job` | unlocks `grinder` | unlocks `wheel` (the grinder's recipe opens with the infrastructure tree, which `first-job` opens) |
| `pave` (moved) | unlocks `wheel` | unlocks nothing |

  No other town mission names a road mission as parent or `after` gate.
  Early power's `town:grinder` and `town:mixer` gates become gates inside
  this tree (decision 2, Power).
- **Carts off the board, not off paving.** A cart runs at a quarter of its
  speed off the road (`land-transport` design 9), and the world lays roads
  between towns, so a hand cart is useful before a player paves; "On the
  road" counts metres on any road cell. Recommendation taken (ask only with
  screenshots).
- **The town tree shrinks** to 27 missions in a trunk and five branches
  (Trade, Jobs, Carts, Houses and logs, Carpentry), 500 coins.

### 4. Recipes and unlocks

- **Moved unlocks** keep their codes: `crushed-stone`, `mixer`,
  `mixed-gravel`, `survey-stakes`. `wheel` moves to `first-job`.
- **The open rule** is the starter tree's `Chain::recipe_open`, now asking
  three chains: a recipe is open when no mission of any tree names it, or a
  mission that names it is done, or the world already holds its output.
- **New recipes and stations** (proposals; every input is a good the game
  has):

| Code | Station | Secs | Inputs | Outputs |
| --- | --- | ---: | --- | --- |
| `mason-bench` | workbench | 6 | 4 planks, 4 block:stone, 1 iron-ingot | 1 block:mason's bench |
| `dressed-stone` | mason's bench | 4 | 1 block:stone | 1 dressed stone |
| `mortar` | mixer | 6 | 2 sand, 1 crushed stone, 1 water bucket | 4 mortar |
| `milestone` | mason's bench | 6 | 2 dressed stone | 1 milestone |

- **Blueprints as unlocks** (`bp-*`): a civil preset is shown in the
  planner's list only once its mission's parent unlocks it, by the same open
  rule (a blueprint id is a code like a recipe's). The civil presets' pieces
  are dressed stone, mortar, planks, logs and rope:

| Blueprint | Pieces (proposal) | Size |
| --- | --- | --- |
| `bp-well` | 12 dressed stone, 4 mortar, 2 log, 1 rope, 1 bucket | 1 cell |
| `bp-retaining-wall` | 6 dressed stone, 2 mortar a cell | 1 to 12 cells, 1 to 3 layers |
| `bp-culvert` | 10 dressed stone, 4 mortar, 4 gravel | 1 road cell over a stream |
| `bp-stone-bridge` | 14 dressed stone, 6 mortar, 2 gravel a cell | 3 to 10 cells |
| `bp-ferry` | 8 planks, 4 log, 30 rope, 2 posts | a river 7 to 20 cells |
| `bp-mill-race` | 4 dressed stone, 1 mortar a cell; 1 sluice board | 3 to 16 cells to a mill |
| `bp-market` | 10 planks, 4 log, 4 cloth (wool), 2 dressed stone | 2 cells |

- **The milestone** is the Town to town reward: a stone a player may set by
  their road (a placed block, named on the map by the road's two towns).

### 5. Saves: every road mission a player has done is kept

Saved games survive every change (CLAUDE.md). The roads are in players'
worlds today, so the move keeps all of it:

- **The infrastructure record is written on open** of a world that has a
  `town_missions` record with `first-job` done (the tree's opening rule) and
  no `infrastructure_missions` record. Its body is the same as the town
  record's: `(opened, progress)`.
- **Moved missions migrate forward.** For each moved id (`grinder`, `crush`,
  `mixer`, `gravel`, `mark`, `pave`, `join`):
  - done in the town record: done in the infrastructure record;
  - open with counts in the town record: open with the same counts;
  - the town record's focus on a moved mission: the infrastructure record
    takes the focus.
  The new record is written first; the town record is not rewritten.
- **The town record keeps its entries for the moved ids, unread** (ids are
  append-only, never renamed; a later tree drops an id and leaves it in an
  old record). A build without this change still reads them, so a rollback
  loses nothing.
- **Coins are never paid twice.** The purse marks every paid mission
  (`Purse::paid` holds `town_missions:<id>`). A moved mission is paid under
  the key `town_missions:<id>` in either tree: the infrastructure tree pays a
  moved id with `pay_once("town_missions:<id>")` and every new id with
  `infrastructure_missions:<id>`. So a road paid before the move is never
  paid again, and one done after it is paid once.
- **Unlocks hold.** `crushed-stone`, `mixer`, `mixed-gravel` and
  `survey-stakes` are named by the same missions, done in the new record, so
  they stay open. `wheel` is named by `first-job`; a world whose `first-job`
  is done has the wheel open, and one that owned a wheel or a cart already
  keeps it open by what it holds.
- **Carts already under way.** `wheels` now hangs off `first-job`; any world
  where `wheels` was open (`pave` done) has `first-job` done too, so nothing
  that was open closes.
- **A world that never reached a town** gets no record until `first-job` is
  done, as today.
- **Ids are append-only** in the new file too.

### 6. What is built today, and what each branch waits on

| Branch | Missions | Needs | State |
| --- | ---: | --- | --- |
| Stoneworks | 4 | the grinder and mixer | in the game; moves as it is |
| Roads | 4 | paving (`townsfolk` 39 to 41) | 3 in the game; Town to town needs the `towns` thing on `pave` |
| Town | 5 | the building planner (built); town growth on the grid (M15, M22) | `plot`, `raise`, `street` today; `ask`, `grow` wait on town growth |
| Civil works | 7 | the mason's bench, the civil kit's presets and their rules | waits; `race` also waits on early power |
| Power | 17 | `early-power` | waits on early power |

A mission whose kind no event raises yet ships **pending** by the starter
tree's own mechanism (`awaited`): shown as NOT YET, met for whatever needs
it. So the tree can ship with Stoneworks, Roads and the first of Town live
and the rest pending, as the town tree shipped with its Roads pending.

### 7. New objective kinds and things

| Kind | Counts | Raised by |
| --- | --- | --- |
| `site` | a site set out from a preset (`<preset>`, or `any`) | the planner's set-out (`building-planner` 3.3) |
| `raise` | a site finished, every piece built (`<preset>`, `any`) | the site's last piece placed |
| `works` | a civil work finished (`well`, `retaining-wall` by cells, `culvert`, `stone-bridge`, `ferry`, `mill-race`) | the civil site's last piece |
| `ask` | a building a town asked for raised in its footprint (`<site>:asked`, `any`) | the board's new building job |
| `grow` | a town reaching its next tier (`any`) | town growth (M15) |

New things on existing kinds: `pave` `towns` (two towns joined by a paved
run, decision 2) and `pave` `street` (cells paved from a site's door to a
town street cell); `travel` `ferry` (a crossing on a ferry). Early power's
kinds (`rise`, `grind`, `drive`, `flow`, `power`, `pole`, `line`, `sluice`)
come with this tree's Power branch; `race` adds `power` `full` (a mill at its
rated output).

### 8. On the screen

- **The Missions tab shows three tree tabs**, STARTER, TOWN and
  INFRASTRUCTURE, the last shown once the tree is open, in the stone tab row
  `town-missions-tree` 12.1 built. Each tree is drawn as the others are.
- **One HUD line, one focus** across the three trees.
- **A toast when it opens**, as the town tree's (`town-missions-tree` 12):
  "Infrastructure missions unlocked".

### 9. Text

Plain and short, from the locale file (CLAUDE.md). Keys
`mission_tree.tree.infrastructure: INFRASTRUCTURE`,
`mission_tree.branch.stoneworks: STONEWORKS`, `.roads: ROADS`,
`.civil: "CIVIL WORKS"`, `.town_works: TOWN`, `.power: POWER`, and per mission
`infra_mission.<id>.title`, `.help`, `.objective."<n>"`. The moved missions'
words move from `town_mission.<id>.*` to `infra_mission.<id>.*` in the same
commit (the locale lint fails a key no code reads). Help lines as the
mockup's cards have them, for example:

| key | text |
| --- | --- |
| `infra_mission.two-towns.help` | Pave a road from one town to another. |
| `infra_mission.well.help` | Set out a well at the drafting table and build it. |
| `infra_mission.stone-bridge.help` | A stone bridge spans up to 10 cells. |
| `infra_mission.ferry.help` | Too wide to bridge? Rig a ferry and pull it across with F. |
| `infra_mission.ask.help` | Towns post the buildings they need on the board. |

### 10. The mockup (M33 version 12)

- **INFRASTRUCTURE** (`#infrastructure`; `#power` redirects to it): the
  whole tree as the Missions screen draws it, Power as a branch with its four
  arms, the side panel's MISSION and RECIPES (recipes, stations and
  blueprints with their inputs), and the bot run, off until Step or Play,
  which plays a mission only once its gates are met.
- **TOWN** (`#town`): the town tree without its Roads; Carts off "A job from
  the board".
- **MILLS** (`#mills`) stays as it is.
- New icons are stand-ins from `tools/art/infrastructure_icons.py`.

## Risks / Trade-offs

- **Three trees to keep in step.** Counting, unlocks and focus are one
  function each over every chain; only the data is three files.
- **Moving built missions** is a save migration; decision 5 keeps it to
  writing a new record from the old one, with the old one untouched.
- **Pending missions.** Civil works and Power will read NOT YET for a while;
  the town tree shipped the same way.
- **Power under the grinder.** Early power opened on The iron age alone; now
  it also needs `first-job` (the tree's opening) and a grinder. A player who
  reaches the iron age without visiting a town waits for power; power is
  town-scale work in this tier (lamps, a mill on the town's river), so this
  is meant.

## Decisions taken

- Roads move here: the owner's words (2026-11-27).
- The opening at `first-job`, the branches, their parents, the counts, the
  coins, the civil works as planner presets, Carts off `first-job`: every one
  a recommendation taken (ask only with screenshots); the owner judges them
  on the INFRASTRUCTURE tab.
