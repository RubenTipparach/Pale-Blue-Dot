# Proposal: town-missions-tree

## Why

**The owner, 2026-11-22:** "can you add a new mission tree for phase 3? this
would be an entirely separate tree since phase 3 is mostly interacting with
the town. Also building roads shjould be like a way point system, where you
have to mark road for pavving and then you walk up to blocks with gravel and
pave it. to make gravel you need a grinder and mixer. which needs to be
crafted in phase 3. with gravel you just equip a shovel and interact with the
blocks marked for crafting a road. wheeled vihecles will move slower without
the road (like 1/4 speed), we also need to add basic carts to phase 3 as
well, and horse carts too".

The starter tree (`starter-missions`, M33) walks a new player through the
first game loop: hands to tools, fishing, farming, herding, mining, cooking,
light and getting about. Phase 3 put towns, people, coins, trade, a job
board, roads, house building and logging into the game (`townsfolk`,
`player-building`, `log-drives`), and nothing walks a player through them.
A player who reaches a town finds a trader and a board with no line telling
them what to try first.

**This change is the write-up and the mockup only.** Nothing here is built.
The game changes on a request, after the owner has looked at M33's new Town
tab.

## What Changes

- **A second mission tree, the town tree**, entirely separate from the starter
  tree: its own file (`assets/config/town_missions.ron`), its own record
  (`town_missions`), its own tab on the Missions screen. Twenty-five missions
  in a trunk and five branches:
  - **Town** (the trunk): meet the townsfolk, buy and sell, take a job from the
    board;
  - **Trade**: trade in earnest, carry a parcel to another town, a friend in
    town (goodwill);
  - **Jobs**: three jobs, bring a lost villager home, mend a building, light
    the lamps;
  - **Roads**: build a grinder, crush stone, build a mixer, mix gravel, mark a
    road with survey stakes, pave it, join a town;
  - **Carts** (after paving): wheels, a hand cart, haul a load, buy a horse,
    hitch a horse cart, a long drive on the road;
  - **Houses and logs**: a house of your own, float logs down a river.
- **It opens the first time the player reaches a town**, whatever the starter
  tree's state (design 1 says why).
- **Each mission pays coins** into the purse (`townsfolk` design 5), from 5 to
  60, 525 in all, and many unlock a recipe: the grinder, the mixer, gravel,
  survey stakes, wheels, the hand cart and the horse cart are locked until
  their mission, by the starter tree's own unlock rule.
- **New objective kinds**: `talk`, `job`, `goodwill`, `stake`, `mark`, `pave`,
  `haul`, `hitch`, `build` and `float`, and `any` as a town in the jobs'
  `<site>:<thing>` names. Each is raised where the game already acts.
- **Saved as its own record**, so the starter tree's record is never touched,
  and an old world opens the tree in place when it holds proof the player has
  been to a town.
- **Every word from the locale file**: the planned keys are listed in the
  design; game text is plain.
- **M33 gains a Town tab** (`mission-tree.html#town`), drawn the way M33 draws
  the starter tree, with its own bot run.

The road paving, the grinder and mixer, and the wheel speeds the Roads and
Carts branches count are written up where they belong: `townsfolk` design 39
to 41 (marking, paving, gravel) and `land-transport` design 9 and 10 (wheel
speed off the road, hand carts and horse carts in phase 3).

## Capabilities

### New Capabilities

- `player/town-tree`: the town tree, when it opens, how it counts and pays,
  how it unlocks, and how it is saved.

### Modified Capabilities

- None. The starter tree's `player/missions` (in `starter-missions`) is not
  changed: the town tree reuses its counting function and unlock rule.

## Impact

- `assets/config/town_missions.ron` (new), read by `pbd_core::missions` as a
  second `Chain` beside the starter tree's.
- `pbd-core`: the new kinds in `missions::Kind`; `any` as a town; the coin
  reward through `economy::coins::Purse::change` with `Reason::Mission`.
- `pbd-app`: the Missions tab's two trees; the HUD line's one focus across
  both; the events the new kinds count.
- `assets/locale/locale_en.yaml`: the keys in design 7, added with the code
  that reads them (the locale lint fails a key no code reads).
- Saves: a new record kind; no existing record changes.
- Order: after `townsfolk`'s paving (design 39 to 41) and `land-transport`'s
  carts (design 9, 10), whose events the Roads and Carts branches count. The
  Town, Trade, Jobs and Houses branches count events the game raises today
  and can be built first.
