# Mission playthrough: design

## 1. The audit (code, every objective)

Every name in the chain resolves (`the_shipped_chain_names_only_what_the_build_knows`),
and every recipe a mission needs is unlocked before the mission opens. What
the audit found against play:

| Mission | Objective | Finding |
| --- | --- | --- |
| `ore` | coal ×3 | **Blocked.** A dug ore cell's yield (`station::ore_yield`: mixed ore, or coal one cell in three) goes into a floating drop, and the dig raises `Gather "block:ore"` only. The drop's pickup raises `PickedUp`, which the mission counter does not read. So `Gather "coal"` is never raised in play. Tests did not see it because they raise the message by hand. |
| `reef` | reach a reef | **Risky.** `about_watch` latches `reef` once per run. A swimmer who crosses a reef while `swim` is open uses it up, and `reef` cannot be met until the game is restarted. `town` and `board` re-arm; `reef` does not. |
| `three-crops` | turnip, chili, frost kale | **Help text wrong.** "Turnip and chili seeds come with the first harvest": a harvest gives only its own crop's seed. Seeds come from wild plants and from digging or tilling grass, by the cell's climate, and chili only where it is hot (desert, jungle). |
| `iron` | iron ×3 | **Help text wrong.** "Iron ore smelts hotter: feed the forge more coal" describes no recipe. Iron comes out of smelting mixed ore (2 copper, 1 iron and 1 tin ore from 4 ore and 2 coal). |
| `harvest`, `collect`, `flock` | | Long in play (wheat six game days; an egg a hen a day, only from a hen). Not a fault. |

## 2. The playthrough (the bot, release build)

Run: `--showcase phase-1-<branch> --world tree-dry-1`, branch after branch.

| Branch | Result |
| --- | --- |
| trunk | all seven missions, 292 s |
| fishing | `rod`, `go-fishing`, 191 s |
| farming | stalled three times, each the bot's fault (below) |

The bot's faults:

1. **Pick: a plant F offers nothing for.** The bot found a wild plant on a
   cell where the game's own F offer was empty (the status line, with the
   offer added to it, read `F None`). It pressed F every 1.4 s and reset
   its own timer at each press, so its 3 s give-up never came: 150 s, then
   a stall. Fixed: three presses that pick nothing give the plant up; cells
   under a standing tree are not chosen.
2. **Farming counts out of date.** The shots sow 4 and harvest 4; the
   missions ask for 6 and 6.
3. **The pour.** After filling, the bot poured without picking the full
   bucket; the right button did nothing. Selecting `water-bucket` first
   pours, and `water-field` is done.

The rest of the playthrough is recorded in `tasks.md` as it is run.

## 3. The fixes

### 3.1 Coal counts when dug

In the dig (`desktop/digging.rs`), when the cell dug is ore, its yield is
read before the edit, and when the yield is a good (coal) the dig also
raises `Did(Gather, <good's code>, 1)`, beside the `Gather "block:ore"` it
raises now. Counted at the cut, as stone is, not at the pickup: a pickup
counts again when a player drops and retakes an item.

### 3.2 The reef re-arms

`AboutState::reef` becomes "on a reef now": set while the swimmer is over
a reef, cleared when they leave it or the water. A reef is reached each
time it is come onto, as a town is.

### 3.3 Help lines

- `three-crops`: "Wild plants give turnip, chili and frost kale seed. Chili
  grows where it is hot, frost kale where it is cool."
- `iron`: "Smelting mixed ore gives iron as well as copper and tin."

Plain game text (CLAUDE.md, "Game text is plain").

### 3.4 Tests

- A test that digs a generated coal cell through the dig system and sees
  `Gather "coal"` counted.
- A test that a swimmer leaving and coming back onto a reef is counted
  twice.

## 4. Found in play, fixed (2026-11-19, with the owner watching)

### 4.1 One mission at a time, shown properly (3.5)

The owner, on a HUD that said "Dig for bait" while the bot mined coal:
"conflicting goals", "ONLY ONE SHOULD SHOW", "THAT IS AN ERROR, CANNOT DO TWO
MISSIONS, FOCUS = 1", "1 style of MISSION DISPLAY NOW TWO", "WHY CANT YOU SHOW
HOW MUCH IS NEEDED", "USE ICONS", "When selecting a mission from the mission
tree ONLY 1 MISSION SHOULD SHOW", "improve mission tracker on hud", and on the
tree "how do I tell whats selected?".

- `Progress::focus`: the one mission the player is on, saved in the record
  (absent from old records). Picking an open mission's card on the tree sets
  it; when it is done the focus goes on to the next of its branch.
- The HUD's tracker replaces the one line: on the kit's stone, the focused
  mission's icon and title, then every objective with its icon (its item; the
  rod, axe or shovel for an act with none; else the mission's own) and count.
  A finished objective is dimmed.
- On the tree, the focused card is framed in the rune's glow, two pixels, at
  every zoom; the card open in the side panel is framed white.
- The showcase bot picks each shot's mission on the tree first (`Focus`), and
  its captions show only the goal and a sped-up wait: one goal on screen.

### 4.2 Casting (3.6, 3.9)

- No cast from under the water ("the line casting shouldnt work underwater"):
  the rod's prompt reads UNDERWATER and the press does nothing.
- A float on a sea-level flat is on water: the flat is drawn under the sea
  but the line read the sheet's rest (0.5 m lower) as the surface, so open
  water said "The float came down on dry ground". Measured: bed 0.00 m,
  surface -0.50 m at every such landing.
- "Go fishing" drops its cast objective ("cast and fish are kind of
  redundant ... it should say catch fish"); a record holding two counts for
  it keeps the second.

### 4.3 Bait (3.10)

The owner: "you should make bait show up in all dirt blocks", "1 in 5 dirt
blocks have worm". Measured first: within 120 m of the coast start, 6,600
earth cells and none counted "by the water", because the sea is drawn over
its cells and not stored in their columns, so the water test never saw it.
Bait now comes from any earth, one cell in five (`gather.bait_one_in: 5`).
Direction from the owner, not built: "worms help with farming and fishing".

### 4.4 Crafting (3.8)

"the progress bar for crafting is not showing, and the percent number is
flickering"; "need some way to queue up crafting too". The pack has its own
bar under the craft row; the plaque says CRAFT, then QUEUE, QUEUE +n while
jobs wait; the hands take four jobs (they took one).

### 4.5 The showcase bot (instrument)

- Walks a hex route (`river_walk::HexRoute::plan_where`, dry mode): land
  first, round trees, its own pits and stations; on the target's own cell,
  straight at the target ("why does the bot keep rotating around").
- Stuck four seconds: faces the goal, takes the shovel or the pickaxe for
  what is in the way, and digs through (the owner's own rule).
- Casts only where the float would land in water 1.5 m deep, the shortest
  hold that gets there ("you dont have to cast at full strength").
- Never fishes standing in water; `Mine` cuts a stair and follows ore.
- Scripted runs leave the owner's pointer free ("make my mouse not locked").

### 4.6 No canoe at a new world's spawn, settings kept in bot runs, bait to fish

- The owner, on the Loon moored at the spawn's shore: "this boat shoulve been
  removed". New worlds moor no craft at the start; boats are made (the raft,
  the tube) and a town's quay moors canoes for "Take a boat". An old world
  keeps the Loon it was given (saved games survive every change).
- "it shouldnt skip settings save": a bot playing the game (`--showcase`)
  reads and keeps the player's settings file; captures and benchmarks still
  run on the fixed defaults.
- "do not allow fishing without bait", "show a message that bait is
  required": a cast needs a worm in the pack; without one the rod's prompt
  reads NEED BAIT and a press says "Needs bait". "Go fishing" says to dig
  worms first. Each catch still takes one worm.
- "theres multiple dirt types, so make sure to cover all dirt": bait comes
  from all seven: the three grasses, dirt, soil, tilled soil and sand.

## 5. Found in play, 2026-11-20

### 5.1 Missions confirmed playable

- **Light** (`build`, `torch`) and **about** (`map`, `swim`) on `tree-run-5`,
  after fixes to the bot (it carries soil, not a dirt block; the mark search
  tries 32 bearings and allows a little water on the way).
- **Flock** (`A mixed herd`): tame a woolback, pen it, collect wool. A loose
  tamed woolback gave one wool and then none; penned, it gave two a collect.
  The mission's help says "tame one as you did the hen", which implies the
  pen. Recommendation taken (ask only with screenshots): leave the rule as it
  is, and say "pen it" in the help text when the next text pass comes.
- **Sea fishing** (`Off the shore`): three sea catches done. Fresh water not
  yet, see 5.2.

### 5.2 An invisible wall after digging (a game bug, not fixed)

On `tree-run-6`, the bot held W facing an open, dug-out patch for minutes
without moving. A plain scripted `Walk(3.0)` that way did not move it either.
The same walk after turning round moved it about a metre. On `tree-run-5` the
walker stood 2 m above where the fine set's columns put the ground. Both
times it was among blocks the bot itself had dug and placed. The likely
cause is the contact tier's collision lagging, or staying out of step with,
the fine set after edits (`planet_contact.rs`, "answers even while the tier
lags a newer set"). Not investigated further; written up for its own change.
The bot works round it by planning round a cell it is held short of
(`bot-path-planner`).

### 5.3 Bait from every dirt block; the bot keeps off the water (owner, 2026-11-20)

The owner: "all dirt blocks should have a chance to get bait, approv on other
items". The game's rule stays as it is: one in five of any earth block gives
a worm (`gather.bait_one_in: 5`, design 4.3). The bot keeps away from the
water when it digs:
- never a cell by the water (`digging::by_water`), nor one under 1.2 m above
  the sea;
- its planner reads open air under the sea's level as sea, since the sea is
  not stored in the columns;
- under water, it swims up and on.
On `tree-run-6` a bait pit dug at the shore flooded and the bot ended 5 m
under the sea.

### 5.4 The mission tree's colours (owner, 2026-11-20)

"Connections are other blue or not based on unlocks.. yellow is too
confusing. Locked nodes should be faded. Selected node should be yellow..
all other nodes should be regular." A line is blue (`RUNE_DIM`) when the
mission it leads to is unlocked (its first need done), plain grey otherwise.
There are no yellow or white lines. The selected card is framed yellow
(`TITLE`); with none selected, the one the player is on is. Every other card
is plain, and a locked card fades to 35% with its icon.

The tree also takes a press on a card itself, as the pack does: the
showcase bot's pointer was overrun by the real mouse and its click became
a pan.
