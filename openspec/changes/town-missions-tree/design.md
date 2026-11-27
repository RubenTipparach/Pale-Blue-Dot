# Design: town-missions-tree

## Context

The owner, 2026-11-22: "can you add a new mission tree for phase 3? this
would be an entirely separate tree since phase 3 is mostly interacting with
the town" (the whole message is in the proposal).

What exists:
- **The starter tree** (`starter-missions`, M33 version 4, built): 42
  missions in `assets/config/missions.ron`, a trunk and seven branches, a
  strict tree (one parent each, hidden `after` gates), counted by
  `pbd_core::missions::Progress::record` from `Happened` events, saved as the
  `missions` record (`Progress`: `done`, `counts`, `seen`, `focus`), shown in
  the pack's MISSIONS tab (O) and one HUD line, and unlocking recipes by
  `Chain::recipe_open`.
- **Town jobs** (`townsfolk` design 8, built): a board's jobs are mission rows
  of the same shape, counted by the same function, with kinds `Deliver`,
  `Escort`, `Repair`, `Sell`, `Buy`, `Light` and `Clear`, whose thing is named
  `<site>:<thing>` so a job counts in its own town only.
- **Coins** (`townsfolk` design 5, built): `Purse::change` with a reason;
  `Reason::Mission` already exists for a job's reward.
- **Phase 3's other parts, built**: trade, the board, the lost villager and
  repair jobs, roads (laid by the world and, today, by the place button),
  house building (`player-building`) and logging (`log-drives`).
- **Phase 3's parts written up in this round**: roads marked by waypoints and
  paved with gravel from a grinder and a mixer (`townsfolk` design 39 to 41),
  and hand carts and horse carts with a wheel speed off the road
  (`land-transport` design 9 and 10).

Every choice below the owner did not make is marked **recommendation taken
(ask only with screenshots)**; M33's Town tab is what the owner looks at.

## Goals / Non-Goals

Goals:
- a separate tree that walks a player through the town: people, trade, the
  board, the special jobs, roads, carts, a house and logs;
- coins as the reward, recipes as the unlock, the starter tree's rules for
  both;
- saves: no existing record changes; an old world opens the tree in place.

Non-Goals:
- the town's own board jobs, which stay as they are (`townsfolk` design 8):
  the tree asks the player to do some, it does not replace them;
- survival, predators and hunting (phase 4);
- the town tree's bot demo video (planned in tasks, made with the batch).

## Decisions

### 1. Its own tree, its own record, opened by reaching a town

- **Entirely separate**, as the owner asked: its own file
  (`assets/config/town_missions.ron`, the same `MissionFile` shape as
  `missions.ron`), its own `Chain`, its own `Progress` in its own record, its
  own tab. No mission of one tree is a parent of a mission of the other, and
  mission ids are their own namespace (the starter tree already has a `town`
  mission; the town tree's ids do not collide with it, and could not clash
  if they did).
- **The root opens the first time the player reaches a town**: the same
  `Reach town` event the starter tree's "Find a town" counts (the walker
  inside a settlement's footprint), whether or not the starter tree has got
  that far. Recommendation taken (ask only with screenshots). Why, and not
  "after the starter tree":
  - the starter tree is 42 missions and many hours; trade, the board and the
    people are in the game now and a player who walks to a town early should
    be shown what to do there;
  - nothing in the town tree's trunk needs a tool: talking, trading and a
    board job need only the player;
  - the branches that need tools ask for things the starter tree teaches
    (planks, rope, iron ingots for the grinder) as craft objectives, which a
    player cannot finish without them; no hidden gate is needed, so the two
    trees never name each other;
  - it gives the tree a place in the world: it starts where the town does.
- **Until then the tree's tab is not shown** and the record does not exist.
  The first `Reach town` writes the record (`opened`, the game day), the
  root opens and the HUD line names it if no mission has the focus.

### 2. The tree

A strict tree, as M33's (owner, 2026-11-15: "a child node cannot have
multiple parents"): every mission one parent, many children allowed. Counts
(the owner's "x" in the starter tree) are data, picked so each mission takes
one to five minutes. Rewards are coins paid when the mission is done.
**New** marks an objective kind the game does not raise yet (decision 4).

**0. Town** (the trunk; opens on reaching a town)

| id | mission | objectives | coins | unlocks |
| --- | --- | --- | ---: | --- |
| `meet` | Meet the townsfolk | **talk** to 3 people | 5 | |
| `first-trade` | Buy and sell | sell 5 goods; buy 1 good (any town) | 10 | |
| `first-job` | A job from the board | hand in 1 **job** | 15 | `grinder` |

**1. Trade** (off `first-job`)

| id | mission | objectives | coins | unlocks |
| --- | --- | --- | ---: | --- |
| `trade-more` | Trade in earnest | sell 20 goods | 20 | |
| `deliver` | Carry a parcel | deliver 1 parcel to another town | 20 | |
| `goodwill` | A friend in town | reach **goodwill** step 1 in any town | 25 | |

**2. Jobs** (off `first-job`)

| id | mission | objectives | coins | unlocks |
| --- | --- | --- | ---: | --- |
| `three-jobs` | Three jobs | hand in 3 **jobs** | 15 | |
| `lost` | Bring them home | walk 1 lost villager home | 30 | |
| `mend` | Mend a building | mend 1 broken wall | 25 | |
| `lamps` | Light the lamps | light 4 town lamps at dusk | 15 | |

`lamps` names `Light`, which the game raises for no event yet (the board
does not post lamp jobs: `townsfolk` task 7.4), so it is **pending** by the
starter tree's own mechanism (`awaited`): shown as not in this build, met for
anything that needs it. It has no children.

**3. Roads** (off `first-job`; `townsfolk` design 39 to 41)

| id | mission | objectives | coins | unlocks |
| --- | --- | --- | ---: | --- |
| `grinder` | Build a grinder | craft and place a grinder | 10 | `crushed-stone` |
| `crush` | Crush stone | make 9 crushed stone at the grinder | 10 | `mixer` |
| `mixer` | Build a mixer | craft and place a mixer | 10 | `mixed-gravel` |
| `gravel` | Mix gravel | make 8 gravel at the mixer | 10 | `survey-stakes` |
| `mark` | Mark a road | drive 2 **stakes**; **mark** 8 cells | 10 | |
| `pave` | Pave it | **pave** 8 cells | 30 | `wheel` |
| `join` | Join a town | **pave** a road that joins a town or a road of the world | 60 | |

**4. Carts** (off `pave`, so the first cart has a road to run on;
`land-transport` design 9, 10)

| id | mission | objectives | coins | unlocks |
| --- | --- | --- | ---: | --- |
| `wheels` | Make wheels | craft 2 wheels | 10 | `hand-cart` |
| `hand-cart` | A hand cart | craft and place a hand cart | 15 | |
| `haul` | Haul a load | **haul** goods 100 m in a cart | 20 | |
| `horse` | Buy a horse | buy a horse at a stable | 25 | `horse-cart` |
| `horse-cart` | Hitch up | craft and place a horse cart; **hitch** a horse to it | 25 | |
| `road-trip` | On the road | drive a horse cart 500 m on road | 50 | |

**5. Houses and logs** (off `first-job`)

| id | mission | objectives | coins | unlocks |
| --- | --- | --- | ---: | --- |
| `house` | A house of your own | **build** 6 walls, 1 door and a roof | 40 | |
| `logs` | Float logs | **float** 4 logs down a river until they stop | 20 | |

Twenty-five missions, 525 coins in all. A horse costs 120 at a stable
(`land-transport` design 10), so the tree pays for the horse by the time it
asks for one, with the board's pay beside it. Recommendation taken (ask only
with screenshots) for every count and reward.

**Unlocks.** The recipe codes are `townsfolk` design 41's (`grinder`,
`crushed-stone`, `mixer`, `mixed-gravel`, `survey-stakes`) and
`land-transport` design 10's (`wheel`, `hand-cart`, `horse-cart`). A recipe
the town tree names is locked until its mission is done, by the same
`Chain::recipe_open` the starter tree uses, now asking both chains: a recipe
is open when no mission of either tree names it, or a mission that names it
is done, or the world already holds its output (an old world keeps what it
could make). In the recipe list a locked one shows as a dim silhouette with
its mission's title, as the starter tree's do.

### 3. How it counts, and what "any town" is

- **One counting function.** The town tree's objectives are counted by
  `Progress::record` from the same `Happened` events as the starter tree and
  the board. Each event is offered to both trees' open missions and to the
  taken jobs; one event may count in all three (selling a fish counts for a
  fishing job and for "Buy and sell").
- **`any` as a town.** A board job names its thing `<site>:<thing>`. The town
  tree's objectives name `any:<thing>` (and `any:any`), which matches every
  site and, for `any`, every thing. `deliver` counts a parcel handed to a
  townsperson of any town but the giver's, which the parcel's own job already
  enforces.
- **Counts start when a mission opens**, as in the starter tree; a job handed
  in before the tree opened does not count for `first-job`.

### 4. The new kinds

Each is raised where the game already acts; none needs a new system.

| kind | counts | raised by |
| --- | --- | --- |
| `talk` | a townsperson talked to, once each (`person`; a person is their town and roster index, so talking to the same one twice counts once) | the talk panel opening (`desktop/talk.rs`) |
| `job` | a town job handed in (`any`, or a job kind) | the board's hand-in (`economy::jobs`) |
| `goodwill` | a town's goodwill reaching a step (`any`) | `economy::goodwill` raising a town's step |
| `stake` | a survey stake driven (`townsfolk` design 39) | the place action with stakes |
| `mark` | a cell marked for paving | the same, one per cell of the new segment |
| `pave` | a marked cell paved with gravel (`gravel`), or a plan whose paved cells reach a world road cell or a town's footprint (`joined`, once a plan) | the interact action with the shovel (design 40) |
| `haul` | metres moved by a cart with at least one slot of its hold full (`cart`) | the cart's step (`land-transport` design 10) |
| `hitch` | a horse put in a horse cart's shafts (`horse`) | the cart's hitch action |
| `build` | a building piece placed by the player (`wall`, `door`, `roof`) | `player-building`'s place (a roof counts when fitted) |
| `float` | a log come to rest after floating 20 m or more in a river (`log`) | `log-drives`' rest event |

Existing kinds reused: `Sell`, `Buy` (with `any:any` and `any:horse`),
`Deliver`, `Escort`, `Repair`, `Light`, `Craft`, `Place` (the grinder, mixer
and carts as placed things), and `Travel` with a new thing `cart-road`
(metres driven by a cart whose wheels are on road cells, any surface).

`Kind::LIVE` grows by each kind the build raises; a kind no event raises
yet is named in `awaited`, which makes its missions pending, exactly as the
starter tree's mechanism does today. So the town tree can ship before the
paving and the carts: the Roads branch past `gravel` and the Carts branch
read as not in this build until `stake`, `mark`, `pave`, `haul` and `hitch`
are raised.

### 5. Saved as its own record

- **`town_missions`**, schema 1, id 0, authored by the player, through the
  durable path like every other record. Its body is the starter tree's
  `Progress` (`done`, `counts`, `seen`, `focus`) plus `opened`, the game day
  the tree opened. A mission done and its coins paid are written in one line
  (the record and the `purse`), so a crash can neither pay twice nor finish a
  mission unpaid.
- **The starter tree's `missions` record is not changed** in shape or
  meaning.
- **Old worlds.** A world saved before this change has no `town_missions`
  record. On open, the game writes one (opened, root open) if the world holds
  any proof that the player has been to a town: a `purse` record, a
  `goodwill` record, a `town_jobs` record with a taken job, a `lost_villager`
  record, or the starter tree's `town` mission done. Otherwise the tree opens
  on the next `Reach town`, as in a new world. Nothing is reset or restarted.
- **Met by what the world holds**, as in the starter tree: a `craft` or
  `place` objective is met on opening a mission if the world already holds
  the thing (a placed grinder, a cart record). A `talk`, `job`, `sell` or
  `pave` objective counts from when its mission opens, so an old world is
  never paid for work done before the tree existed.
- **Ids are append-only**, never renamed, as in `missions.ron`; an id a
  later tree drops stays in an old world's record, unread.

### 6. On the screen

- **The Missions tab (O) shows two trees**, chosen by two oak tabs in its
  header, STARTER and TOWN, the TOWN tab shown once the tree is open. Each
  tree is drawn as the starter tree is today (M33's look, `TS.4`).
- **One HUD line, one focus.** Picking a mission on either tree makes it the
  focus and clears the other tree's; the HUD shows the focus, else the
  starter tree's first open mission, else the town tree's.
- **A finished mission** says so in the log line the starter tree uses
  ("DONE: Pave it"), adds a coin line ("+30") beside the coin count, and
  plays the starter tree's chime.
- **The bot plays it** (CLAUDE.md, "Demos"): goal "finish the town tree",
  each branch a section, through the player's own inputs; waits (a day for
  stock, a lost villager's walk) sped up with a caption.

### 7. Text: every word from the locale file

Game text is plain and short (CLAUDE.md, "Game text is plain"). The keys,
added with the code that reads them (the locale lint fails an unused key):

- `mission_tree.tree.starter: STARTER`, `mission_tree.tree.town: TOWN`;
- `mission_tree.branch.town: TOWN`, `.trade: TRADE`, `.jobs: JOBS`,
  `.roads: ROADS`, `.carts: CARTS`, `.houses_logs: "HOUSES & LOGS"`;
- `missions.coins: "+{count}"`;
- per mission `town_mission.<id>.title`, `.help` and `.objective."<n>"`,
  for example:

| key | text |
| --- | --- |
| `town_mission.meet.title` | Meet the townsfolk |
| `town_mission.meet.help` | Walk up to people in town and talk with F. |
| `town_mission.meet.objective."0"` | people |
| `town_mission.first-job.help` | Take a job from a town's board, do it and hand it in. |
| `town_mission.grinder.help` | Make a grinder at the workbench and set it down. |
| `town_mission.mark.help` | Hold survey stakes and drive one at each turn of your road. |
| `town_mission.pave.help` | Hold the shovel, carry gravel, and press F on a marked cell. |
| `town_mission.horse.help` | Stables in the larger towns sell horses. |
| `town_mission.road-trip.title` | On the road |

The other titles are the tables' (decision 2). No riddles, no colons that
explain, no notes.

## As built (2026-11-22)

What the build changed from the plan above, each a recommendation taken
(ask only with screenshots):

- **The coins' line.** A save writes one record a line, so a mission's done
  mark and its coins cannot share a line. The purse carries the mark
  instead: `Purse::paid` holds `town_missions:<id>` for every town mission
  it has paid, and `Purse::pay_once` pays a key once. A count that finishes
  missions writes the purse first, then the `town_missions` record; a write
  torn between them leaves the mission paid and not done, the next count
  finishes it again and pays nothing. Old purses read with nothing paid.
- **The record's body** is `(opened, progress)`, the starter tree's
  `Progress` nested whole rather than flattened (RON does not flatten).
- **A kind with an underscore** (`town_missions`, and the town's
  `town_jobs`, `town_market`, `town_repair`, `lost_villager`) was read back
  as a damaged journal line, so those records did not survive a reload.
  The journal now reads a kind of words joined by `_`
  (`records::is_kind`); records already written come back from their lines.
- **Branch shape** follows M33's Town tab, the approved shape: within a
  branch each mission needs the one before it (`deliver` after
  `trade-more`, `goodwill` after `deliver`, `lost` after `three-jobs` ...),
  each branch's first mission off `first-job`, Carts off `pave`.
- **`talk` and `job` name `any`** in the file (each person counts once by
  `<site>:<index>`); `goodwill` names `any:step`.
- **Pending in this build**: all of Roads, all of Carts. `lamps` is live
  with decision 8.
- **Icons** are existing item art standing in (the coin, the parcel, the
  lantern post, planks, stone, gravel, rope, leather, a log); new art for
  talk, the board, the grinder, the mixer, stakes, wheels and carts waits.
- **No chime** on a finished town mission: the starter tree has none to
  share yet. The coins show as `+N` in the coin count's notice line.
- **Capture instruments**: `--tree-town` shows the town tree on the tab,
  `--town-focus <id>` makes a town mission the focus once the tree opens.

## Follow-ups from the owner's known issues (2026-11-22)

The owner's notes on phase 3, in the jobs and missions: "need button to
abandon jobs too, add icon or job", "we need a marker", and, on lamps at
dusk, "ok do it". Each below is a recommendation taken (ask only with
screenshots).

### 8. Lamps at dusk

- **When.** Dusk is 17:00 to 22:00 by the sun's clock (`DUSK_FROM_H`,
  `DUSK_TO_H` in `pbd_core::economy::jobs`). A board shows its lamps job
  only from 17:00; before that the board's place keeps the job and shows
  the rest. `lamps` joins `jobs.ron`'s `posted`.
- **What counts.** With a lamps job of that town taken, the interact key
  within 2.5 m of one of the town's lamps (`record::lamps_of`, the lamps
  the town already draws) during dusk raises `light <site>:lamp`, once a
  lamp a job (the lamp's index in the town's list is the counted name, as a
  person is for `talk`). Out of dusk the key does nothing to a lamp. The
  lamps are lit by the town at night as they are today; nothing about
  their look changes.
- **The town tree's "Light the lamps"** counts the same event, so it is
  live: `Light` returns to `Kind::LIVE`.
- **Text.** The prompt is "Light lamp"; the job line keeps its words.

### 9. Abandon a job

- **The board's row** of a taken job that is not done shows ABANDON where a
  done one shows HAND IN.
- **Abandoning** takes the job out of the taken jobs and drops its counts;
  it is not handed in, so while its board place keeps it, the board shows
  it again. No coins, no goodwill change. A delivery's parcel leaves the
  pack if it is there. A lost villager's record is cleared, so they are
  back at their post. A repair's broken wall stays as it is: taking the
  job again from the board finds the same wall. The jobs record is written
  first, then the job's own.
- **Text.** "ABANDON", and the line "Job abandoned".

### 10. The lost villager's icon and marker

- **An icon of its own**: a 16 px item icon, `assets/items/missions/
  lost-villager.png`, a townsperson's head and shoulders with a question
  mark, in the item icons' style (a 1 px dark outline, as UI icons have),
  shown in the job's well on the board and the talk panel where a job with
  no good showed an empty well.
- **A marker**: while a lost villager's job is taken and not handed in, the
  map shows a marker at the record's point (where they stand, or where they
  follow the player), named "Lost villager", and the compass shows it as a
  waypoint is shown, with its distance.

## 11. The Carpentry branch (from `furniture`, 2026-11-23)

The owner, 2026-11-23: "Carpenter missions / And yes we need a carpentry
station for furniture". A sixth branch, `carpentry`, off the trunk's
`first-job`: nine missions, 115 coins, written up in `furniture` design 12
(the table of missions, the new kinds `sit`, `lie` and `loot`, and `build`'s
furniture names). The tree is then 34 missions paying 640 coins. M33's Town
tab draws the branch.

## 12. A toast when the tree opens (owner, 2026-11-23)

The owner: "make sure that when player finds town, the "town" mission tree
beocmes unlocked and the player gets a toast". The tree already opens on the
first `reach town` (design 1), but nothing said so: the HUD line shows "Find
a town" done and the tab's plaque appears, both easy to miss.

When `count_town` opens the record (a town reached, or an old world that has
been to one, on its first open with this build), it raises a flag the
desktop reads once: one toast in the item toasts' column (`item-toasts`)
with the town mission's lamp icon and "Town missions unlocked", shown and
faded as an item's toast is. A toast is a row of that column with its own
words in place of an item's name and count.

## Risks / Trade-offs

- **Two trees to keep in step.** The counting, unlocks and focus are one
  function each, taking both chains; only the data is two files.
- **The tree pays coins the board also pays.** Every reward is one-off, and
  525 coins is a few hours' trade; `values.ron` and the tree's file are where
  it is tuned.
- **Pending missions in a shipped tree.** `lamps` (and, until the paving and
  carts land, the Roads branch past `gravel` and the Carts branch) show as
  not in this build. The starter tree shipped the same way.

## Migration Plan

- New record kind only. A build without this change ignores the record; a
  build with it writes the record on the first `Reach town` or on opening an
  old world that has been to a town.
- Rollback: the previous build ignores `town_missions`; recipes the town tree
  locked are open again (no mission names them).

### 12.1 The tree tabs (owner, 2026-11-24)

"town and starter should be tabs below missions, with the buttons styled a
little different ... if I toggle between them the ui stops rendering
missions entirely", and "button shading also not clear which one is
clicked". The tree rebuilt on a switch was spawned hidden, and the menu
shows a panel only when the screen changes, so it stayed blank until the
tab was left and opened again; it is now spawned shown while the Missions
tab is open. The two plaques are stone (the header's are oak), in a centred
row of their own under the header, and the shown tree's tab has an amber
bar under it.

## 13. The Roads branch moves to the infrastructure tree (owner, 2026-11-27)

The owner, on the first draft of phase 4a's infrastructure tree: "Maybe move
roads over to this section". Roads are civil works, so the Roads branch
(`grinder`, `crush`, `mixer`, `gravel`, `mark`, `pave`, `join`) moves to the
infrastructure tree (`infrastructure-missions` design 3), under the same ids,
objectives, counts and coins. Written up, not built: `town_missions.ron` keeps
the branch until `infrastructure-missions` task 2 is asked for.

- **What pointed at roads here now points elsewhere**: `wheels` (the Carts
  branch's first) hangs off `first-job` instead of `pave`; `first-job`
  unlocks `wheel` instead of `grinder` (the grinder's recipe opens with the
  infrastructure tree, which `first-job` opens); `pave` no longer unlocks
  `wheel`. No other town mission names a road mission.
- **The tree is then** a trunk and five branches (Trade, Jobs, Carts, Houses
  and logs, Carpentry), 27 missions, 500 coins.
- **Saves keep every road mission done**: the infrastructure record is
  written on open from this record, moved missions done or open with their
  counts and focus as they were; this record is not rewritten and keeps its
  entries for the moved ids, unread; coins paid under `town_missions:<id>`
  are never paid again (`infrastructure-missions` design 5).
- M33's TOWN tab (version 12) no longer shows the Roads branch; the
  INFRASTRUCTURE tab does.
