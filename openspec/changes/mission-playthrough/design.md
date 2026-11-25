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

A done mission's card is not picked: "Selecting a completed mission node is
not possible because it's already completed..so focus on that mission isn't
possible" (owner, 2026-11-20). A click on it changes nothing; its hover card
still shows it.

### 5.5 The F3 graph froze after half an hour (fixed)

`measure_frames` stopped recording at 100 000 frames, so after about 28
minutes at 60 fps the F3 graph and its fps line showed the same frames for
the rest of the session. The owner, on the bot's three-hour run: "why did
the frame graph not tick forward?". Past 100 000 the oldest half is now
dropped; the frame log numbers frames by a counter of its own.

## 5. The last six missions, played (2026-11-22)

On `tree-run-6`, after the rest branch (fresh water fixed), 36 of the 42
missions are done. No showcase plays the other six (task 1.6): three crops,
farm meal, feast, reef, town, boat. The owner: "make sure all missions work".
A new branch, `phase-1-last.ron`, plays them on the same world, through the
player's inputs as every branch does:

1. **Chili, where it grows.** Chili grows only in a hot climate (jungle,
   desert); turnip and frost kale only in a temperate or cold one. The world
   stands in jungle at the end of the rest branch: chili seed dug from grass
   there, sown, five days waited, harvested.
2. **Turnip and kale, in temperate ground.** `Seek("temperate")`, then the
   same with both seeds; `Plant` sows the seed in hand when the hand holds
   one (today it sows wheat whenever wheat seed is carried).
3. **Farm meal and feast** at a campfire (`Cook`, which places one when
   none is near): roast turnip, chili fish; then the three dishes eaten.
4. **Reef:** `Seek("reef")`, the nearest reef found by
   `ocean::places::environment`, walked to its shore and swum onto.
5. **Town:** `Seek("town")`, the nearest of the world's towns, walked to.
6. **Boat:** a raft made at the workbench (6 logs, 4 rope), launched on
   water from the shore (`Select("raft")`, `Use`), boarded with F
   (`Tap("F")`) and rowed (`Hold("W", secs)`) past 100 m.

New acts: `Seek(kind)` (`hot`, `desert`, `temperate`, `cold`, `reef`, `town`),
`Tap(key)` and `Hold(key, secs)` (a key pressed as the player presses it).
A seek searches rings out to 4 km round the walker (24 bearings, a ring every
40 m) and fails if none is found.

### 5.1 Open: the bot boxed in on the way to hot ground (2026-11-22)

`Seek("hot")` found jungle 80 m off and 30 m up a hill; the planner's route
dug the bot into a one-cell shaft at 20 m (`bot-path-planner` 7). A trace of
the walker (`PBD_WALK_TRACE`, a measurement) showed W held, grounded, and no
speed: walled in on every side, and every block set under the feet refused.
Six fixes to the climb out did not free it. The bot world's saved position
was then moved onto the hot ground the seek had found (a test world, the
bot's own state; nothing a mission counts), so the last six missions could
be played. The shaft is left open in `bot-path-planner`: the route planner
must not dig the walker into a shaft it cannot climb from.

### 5.2 The seed that would not come to hand (2026-11-22)

On the hot ground, `Select("chili-seed")` failed at once: "the pack is
full". The seed was in the pack, the hotbar and the pack both full, and the
only blocks in the hotbar were dry grass and stone, which the select's junk
list did not name, so there was no stack to spare for the swap. The list now
takes the ground's own blocks (dry grass, jungle grass, desert sand) and,
last, stone.

### 5.3 A climate's edge (2026-11-22)

The seek's hot point was jungle, but the field dug beside it was temperate:
"Chili does not grow in a temperate climate", every sowing refused. A climate
is now sought over a patch: the point and eight bearings at 8 and 15 m.
Measured next: still refused, on a patch hot for 15 m round. The bot digs and
tills the free cells nearest its home, and home was where the run started,
40 m back. A seek of a climate now makes the place found the bot's home.
Measured next (a log of each sowing): home was desert, Hot by the game's own
rule, but the field was dug 23 m off, past the 15 m patch, on temperate
ground. The digging roams 30 m for fresh cells, so the patch is now checked
out to 30 m (eight bearings at 8, 15, 22 and 30 m).
Measured next: 37 m off, temperate again; the digging roams with the bot. So
a seek of a climate is remembered, and fresh ground is dug and tilled only on
cells whose climate, by the game's own `farm::climate_at`, is the one sought.
Measured next: 72 roams and no fresh hot cell within 30 m of a desert home.
The bot's fresh-ground filter took soil, grass, sand and dirt, not desert sand
(material 29), the only ground a desert has, though `farming::tillable`
takes it. Desert sand is now fresh ground.
Measured next, in jungle: ten minutes of "Cut down the tree first", the bot
tilling cells the tree-growth test passed but the game's `under_standing_tree`
refuses. Fresh ground now leaves those out by the game's own rule.
Then, still in jungle: trees round every free cell. The chili is sown in a
desert instead (`Seek("desert")`: hot, open, desert sand tilled), which the
bot had found 40 m off on earlier runs.

### 5.4 Where the night ended (2026-11-22, 05:20)

The last six missions are not yet played: the bot never got a chili crop to
grow. Everything that went wrong was the bot, not a mission rule:
- the climate-sought field drifting off the climate (fixed: the game's own
  climate rule picks the field's cells);
- desert sand not fresh ground (fixed);
- the walk digging up the bot's own crop (fixed: tilled soil is never dug);
- a full pack, the wrong tool in hand (fixed);
- jungle cells under standing trees (fixed: the game's own tree rule);
- **open:** once under ground (a tunnel or a shaft of the planner's own
  digging), the bot cannot get out. Walking, jumping (tapped or held) and
  setting a block under the feet all fail; the walker's height does not
  move. A trace showed W held and the walker grounded with no speed. The
  long walk now goes over the surface (`bot-path-planner` 9), but a walker
  already under ground is still stuck.

Next: find why a walker in the bot's tunnels can neither move nor jump (a
capsule held by blocks set at its rim is the lead: blocks were set while
"is where the player stands" refusals were logged), then play the branch
again from a surface start.
Then, out of the owner's pillar and over the surface to hot ground, the field
found no fresh cell: every open cell round was the floor of the bot's own
digging, which fresh ground leaves out, and the bot's notion of a cell's top
was the generated one, metres over a dug floor. A cell's top is now the
ground as it is (within six layers of the generated), and in a sought climate
an edited cell is fresh ground when its top is tillable with air over it.
Measured next (a count of why fresh ground refused each cell): every cell
round the hot home refused for its material, the fine record's material
reading 4 (water) at 59 to 62 m on ground drawn as grass. The record's field
is not the top block there. Fresh ground now also takes a cell whose top
block, read as the game's own dig and till read it (`material_of`), tills.
The record saying water on dry grass is left open for a look: it may be a
river's water mark, and it may be a bug in the record.

### 5.5 The bot dug its own field (owner, watching, 2026-11-22)

The owner: "grass tilling looks wrong", "hes just digging lol". The log: "dug
Tilled" cell after cell between the turnip and kale sowings. The planner
leaves tilled soil alone (`bot-path-planner` 8), but the walk's older
fallbacks dig whatever is ahead. The bot's button stage now never presses
the dig button on tilled soil, whichever act asks. The sowing picks only
cells the game would take a seed in (tilled, air over, nothing growing).

Found with it: the bot's held keys were W, S, Shift and G only, so a held
Space (the climb's jump), A, D, Q and E never reached the game. All are held
now.

### 5.6 Long seeks walk in legs (owner: "game is lagging", 2026-11-22)

The walk to a reef 2645 m off held the game at three and a half cores: the
goal's cell was not loaded, so every frame the planners were asked for it and
found nothing. A seek now walks toward a point at most 120 m along the way,
on the ground, moved on as the walker goes; the place itself is the goal only
within 120 m.

### 5.7 To the sea for reef rock (owner, 2026-11-22)

The owner: "write up a planner script to head towards ocean water to gather
reefs", after the reef mission became "Get a reef" (`starter-missions`,
"Get a reef, not see one"): one block of reef rock, mined with a pickaxe.

**Why the shot stalled.** The bot's `Seek("reef")` looked on 40 m rings out
to 4 km, 24 bearings a ring: at 4 km the bearings are a kilometre apart, and
a reef (warm shallows, reef rock in 7 m patches) fits between them. From the
turnip field on `tree-run-6` it found "no reef within 4 km".

What is already in the working tree (bot code only, ahead of this write-up;
it stays in only if this plan holds): the search is a point every 20 m on
rings every 20 m, nearest first, out to the far side, for the nearest column
whose top is reef rock (`ocean::cells::top`, the game's own rule); the seek
ends within 6 m of that column; the bot's pickaxe counts reef rock and
basalt as breakable. A whole-planet search is about 1.8 million points; it
runs once per seek and stops at the first hit. Its time is measured on the
first run, and if it holds a frame past 50 ms it moves to a thread.

**The route, as a player would go.** Four legs, each one a phase of the
seek:

1. **To the coast.** The planner keeps off water (5.3), so it is not asked
   for a path into the sea. The goal is the shore point nearest the great
   circle to the reef: the `Shore` act's search (`shore`, used for the raft
   launch), started from a point 120 m back from the reef along that circle
   toward the bot, not from the bot. The walk goes in 120 m legs over loaded
   ground (5.6), on the surface route.
2. **Into the water.** From the shore point, W and steer toward the reef,
   with no planner. The ground falls under the sea's level and the bot wades,
   then swims (the game's swim: it floats while Space is held and sinks at
   about 2.5 m/s when it is not).
3. **Across.** On the surface, Space held a beat in every second so the head
   stays up, steering on the bearing to the reef. It logs its distance every
   4 s. A swim that gets no nearer in 30 s turns 30 degrees and tries again;
   after three turns it is a failure, logged with its position.
4. **Down to the rock.** Within 6 m of the reef column: let go of Space and
   sink. Looking down at about 60 degrees, with the pickaxe, `Dig` takes
   the reef-rock cell ahead or under it; it may dig only reef rock (sand
   round the patch is left). Once the block is in the pack, Space is held to
   rise, and the shot ends. The mission ticks on the gather.

**Checks, in the run's log:** the seek's target and distance, the search's
time, each leg's start, the swim's distance every 4 s, the dig, and
"mission done: reef". Screenshots: the bot at the shore, swimming, at the
rock under water, and the mission card done.

**Not in this plan:** breath (the calm mode has none); a raft to cross deep
water (the boat mission comes after the reef); and reefs too far to swim. If
the nearest reef is over 2 km off, the run says so and stops, and the bot
world's position is moved toward a warm coast, as in 5.1, recorded here.

### 5.8 Finding a reef from the map: global planning (owner, 2026-11-22)

The owner: "bot to find reef: open map, click sea zones, mark a marker where
reefs are, head towards that. one thing the bot is definitely missing is
global planning, and thus this shuold help with that".

This replaces the search in 5.7, which reads the generator directly (a rule
the player cannot see). The bot plans from the map, as the player does:

1. **Open the map** (M), as the `Map` act does now.
2. **Choose the Sea places layer** in the legend (`map_screen::places_layer`,
   "Sea places"), with a click on its row, the way a player picks it.
3. **Find a reef on the layer.** The bot reads the map's own reef colour,
   from the layer's texels for the tiles the map has drawn (the same
   `place_class` the legend's key uses). It works from the zoom the player
   sees, panning or zooming out until a reef is on screen. It takes the reef
   pixel nearest the player.
4. **Mark it** with a right click there (the player's waypoint;
   `map_screen::marks_of` reads it back). This is the "marked" mark the
   `reach marked` mission uses.
5. **Close the map and head for the mark** with `ToMark`, walking in 120 m
   legs (5.6). At the coast it goes on as 5.7's legs 2 to 4: into the water,
   across, and down to the rock.

**Why this is the global plan.** The planner (`bot-path-planner`) is local.
It sees loaded ground round the bot and plans a path of tens of metres. It
cannot choose between a reef 900 m east and one 3 km north, and before this
it had no goal past what the bot could see. The map is the game's own global
knowledge: biomes, sea places, towns. A goal taken from it is one a player
could take. The same steps serve the other far goals:
- `Seek("town")` marks the town on the map;
- `Seek("desert")` / `Seek("temperate")` read the Biomes layer;
- the boat mission's launch picks a bay.

**The act:** `MapGoal(layer, class)`. It opens the map, picks the layer,
finds the nearest pixel of that class, marks it and closes the map, leaving
the mark for `ToMark`. The reef shot becomes:

    Focus("reef"), MapGoal("places", "reef"), ToMark, Swim("reef"),
    Hand("pickaxe"), Dig(Have("block:reef rock", 1), 1), Still

Here `Swim("reef")` is 5.7's legs 2 to 4. The bot code put in for 5.7 (the
20 m ring search) stays only as the fallback when the map shows no reef. It
is taken out once `MapGoal` works.

**Checks:** a screenshot of the map with the Sea places layer and the
bot's mark on a reef; the run's log of the mark's distance and each leg;
"mission done: reef".

**Limits:**
- the map draws reefs at its tile resolution, and a reef smaller than a
  pixel at the zoom used is not seen, so the bot zooms in on a candidate
  before it marks;
- this waits on B30 (`ocean-environments`) only for how often reefs are
  near, not for how it works.

## 6. The town tree, played (owner, 2026-11-23: "begin testing phase 3 missions tonight")

The town tree (`town-missions-tree`, 34 missions on main) is played by the
same bot, on the same world, after the starter tree: it opens on the first
`reach town`, so the run carries on from "Find a town". Most of its
objectives need acts the bot does not have. Each goes in as the player
does it, through the player's own inputs (the trade panel and the board
pressed as the pointer presses them):

| Act | Through | Missions |
| --- | --- | --- |
| `Talk(any)`: walk to the nearest townsperson, F, close | the Talk screen | meet |
| `Sell(n)` / `Buy(code, n)`: Talk, the Trade tab, press a row | the Trade screen | first-trade, trade-more, horse |
| `TakeJob(kind)` / `HandIn`: the board, a job of a kind the bot plays (fish, gather) | the Board screen | first-job, three-jobs |
| `Deliver`, `Escort`, `Mend`, `Light` | the job's own walk and F | deliver, lost, mend, lamps |
| `Bench` at any station by its code (grinder, mixer, carpenter) | the station panel | crush, gravel, chair, table, bed, chest |
| `Stake`, `Mark`, `Pave` | the road tools in hand | mark, pave, join |
| cart acts: set down, haul, hitch | the cart's own inputs | hand-cart, haul, horse-cart, road-trip |
| `Build(piece, n)` in build mode (N) | the build ghost | house, furnish |
| `Furniture("sit"\|"lie"\|"loot")`: the nearest free town piece of that use, walked to, aimed at, F; a seat or bed held a moment and left with F, a store's coins (or one stack) taken and shut; a piece that does not take the bot is passed over | F on a piece | sit, lie, find |
| `Float(n)`: logs into a river | drop on water | logs |

Order: the trunk (meet, first-trade, first-job) first, since every branch
waits on it; then carpentry and houses (stations and pieces the bot nearly
places already), then trade and jobs, then roads and carts. Each new act
is written into this table's row before it is built, and each stall found
goes in section 7 of the tasks as the starter tree's did.

The starter tree's last missions are played first, on the merged build: the
fresh half of "Sea fishing" (the bot's fresh-water search fixed,
`fishing-anywhere` design 6), the three crops, the farm meal, the feast,
the reef, a town (with the town tree's toast, `town-missions-tree` design
12) and the boat (a raft crafted at the bench, then boarded; the inner tube
needs 3 leather, 6 to 8 game days of a kept animal, so it waits).
