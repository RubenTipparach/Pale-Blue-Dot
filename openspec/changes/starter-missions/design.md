# Starter missions: design

## Decisions

Recommendations taken (ask only with screenshots) unless quoted.

1. **Data, not code.** Each mission is
   `{ id, title, help, needs: [ids], objectives: [{ kind, what, count }] }`.
   - Kinds: `craft`, `place`, `gather`, `catch`, `cook`, `find`, `plant`,
     `harvest`, `tame`, `pen`, `collect`, `smelt`, `fell`, `cast`.
   - Ids are append-only short codes, so a saved record keeps its meaning.
2. **Counted from events the game already raises.** The systems that do the
   thing send an event, and the missions system counts it:
   - the recipe completes;
   - `apply_edit` places a block;
   - the fishing catch is landed;
   - the harvest happens.

   No mission reads the screen, and no mission gives an item. Rewards are the
   next mission. Coins come with the townsfolk phase.
3. **Only the current missions count.**
   - An open mission counts events from the moment it opens.
   - Events from before the mission opened are not counted, except that
     state-shaped objectives can already be met when it opens: `craft` of a
     tool owned, or `place` of a station already in the world.
4. **A new world starts the chain. An old world joins it**, with every
   state-met mission marked done on open.
5. **One line on the HUD** shows the current mission and its count ("Catch
   fish 1/3"). The pack's Missions tab lists the chain: done, current, and
   locked missions with their titles.
6. **Skippable.** A setting hides the HUD line. The chain still records
   progress.

## Risks

- **The chain can name something that is not built yet.** Each mission is
  added to the data only once its system is in the game, and a data test
  checks every `what` against the goods, recipes and species tables.

## Build decisions (2026-11-14)

Recommendations taken (ask only with screenshots) while building the change
on `claude/starter-missions`, branched from `claude/phase-1` (farming,
smelting, animal husbandry and cooking merged). Tool crafting is built in
parallel on `claude/tool-crafting` and was not on the remote when this was
written, so its codes are the ones its design names.

7. **The chain is `assets/config/missions.ron`, read by
   `pbd_core::missions`.** A mission is
   `(id, title, help, needs, objectives)`; an objective is
   `(kind, what, count, label)`.
   - `what` is one name, several names joined by `|` (any of them counts),
     `any`, or a group: `seed` (any crop's seed), `crop` (any crop's
     produce), `ingot` (a good whose code ends `-ingot`), `fish` (any
     species).
   - Names are the recipe table's own: a good by its code (`wheat-seed`), a
     block as `block:<name>` (`block:forge`), a tool as `tool:<name>`
     (`tool:pickaxe`), a recipe as `recipe:<code>` (`recipe:bronze`), an
     animal or a fish by its roster name, lower case (`yard hen`).
   - Kinds, and what each counts (the event that raises it):
     `gather` (a block dug, or a good turned up), `craft` (an item made at a
     station or by hand; `recipe:<code>` for one recipe), `place` (a block
     placed by the player), `cast` (the float lands on water), `catch` (a
     fish landed), `cook` (a dish collected, by recipe code), `find` (a good
     found in grass or picked from a wild plant), `till` (a cell tilled),
     `plant` (a seed planted, by seed code), `harvest` (produce harvested),
     `tame`, `pen` (a kept animal let go in a pen), `build_pen` (a pen closed
     with a gate), `collect` (an egg, wool or milk), `smelt` (an ingot made),
     `fell` (a tree felled).
8. **Pending rows.** An objective whose `what` this build cannot resolve is
   pending when every unknown name is listed in the file's `awaited` (the
   tool-crafting codes); any other unknown name is an error at load, as in
   `cooking.ron`. A `craft` of an item no recipe makes (`tool:pickaxe`
   today) is pending too, and `fell` is pending while the game has no
   felling. "Make a fire" plays: `cooking.ron` already has the campfire's
   hand recipe, and a new world is dealt a campfire (kit version 6), so its
   craft is met by carrying one and it is done when placed. A mission with a pending objective is **skipped**:
   logged at load (`mission <id> is pending: ...`), listed in the tab as
   "not in this build yet", and treated as met by the missions that need
   it. So on this branch the chain plays from "Make a fire", and the missions
   before it light up after the tool-crafting merge, with the state-met rule
   (decision 4) closing any the world already holds.
9. **Current missions.** A mission is open when every mission it needs is
   done or pending. Several can be open; the first open one in file order is
   *the* current mission, shown on the HUD. Every open mission counts.
10. **State-met objectives** are checked when a mission opens and when a
    world opens: `craft` of a tool the world owns or an item the slots hold
    `count` of (for a block, carried and placed together), and `place` of a
    block the world's edits hold `count` of
    (the save's edits, so a forge dealt in the kit counts only once placed).
    Nothing else is state-met: a fish caught before the mission opened is not
    counted.
11. **The record.** Kind `missions`, id 0, schema 1, a player record stored
    through `WorldSave::store` as the player's: the done ids and the counts
    of the open missions. Written on every change of count (a handful of
    lines a minute at most). A world with no record (every world before this
    change, and every new one) starts the chain and joins it with the
    state-met missions done; done ids that this build's chain lacks are kept
    in the record as they are, so a newer chain's progress survives an older
    build.
12. **The HUD line** sits top-left under the compass strip, in the kit's
    face with its shadow: `Catch fish 1/3`, or for a mission of several
    objectives the first unmet one, `Make a fire: place 0/1`. When a mission
    is done it reads `Done: <title>` for four seconds, then the next. The
    completion is logged (`mission done: <id>`). There is no audio engine
    yet, so the `ui_*` chime waits for it.
13. **The Missions tab.** `O` opens it on foot, as `I` opens the pack, and
    a pair of oak tabs over both panels (PACK, MISSIONS) switches between
    them. It lists the chain in order: done (ticked, dim), current (bright,
    with each objective's count), open, locked, and pending.
14. **The setting** "Mission line" (Settings, World tab) hides the HUD line;
    the chain still counts. On by default. The settings file goes to
    version 4; an older file reads it as on.
15. **Events.** The missions system reads the messages the game already
    raises (`hotbar::Gained`, `hotbar::Planted`, `husbandry::Deed`,
    `kitchen::Cooked`) and one new message, `missions::Did`, raised where
    nothing said it before: a block dug or placed by the player
    (`digging`), the float landing on water and a fish landed (`fish`), a
    cell tilled (`farm`), and a station recipe collected (`forge`, as
    `recipe:<code>`). Tool crafting raises `Gained { how: Made }` for what
    it makes and, for its workbench, `Did` with `recipe:<code>`.
16. **Counts** are picked by the time each takes in this build, not
    measured with a stopwatch: three fish, three dishes, four seeds, four
    tilled and planted cells, four produce, one animal, one pen, one
    product, three ingots. Harvest waits on a crop's growth, which is world
    time (`farming.ron`); its count is the one harvest of two plots.
17. **The capture instrument** `--missions` (a capture flag) raises the
    messages the systems raise, in the chain's order, one every few frames,
    through the same missions system the game runs, so the HUD line, the tab
    and the record are photographed mid-chain on a fresh world. It is a
    measurement instrument: it plays no fishing or farming itself.

## After the tool-crafting merge (2026-11-14)

`claude/phase-1` gained tool crafting while this was built, and was merged
in. Recommendations taken (ask only with screenshots):

18. **Nothing is pending.** Its goods (`fibre`, `stone`, `log`, `planks`,
    `rope`), the workbench (`block:workbench`) and its recipes (`planks`,
    `rope`, `workbench`, `stone-pickaxe`, `stone-shovel`, `stone-axe`,
    `rod`, `forge`, `bronze-*`, `iron-*`, and `campfire` in `cooking.ron`)
    are named in the chain as they are, and `awaited` is empty. The pending
    machinery (decision 8) stays for the next chain that names another
    branch's work.
19. **"Make an axe, and fell a tree" is "make an axe and cut logs x3".**
    Trees give logs (a branch by hand, up to four with an axe) and are not
    felled, so the objective is `gather log x3`. `fell` stays a kind that
    nothing raises, and so is pending wherever a chain names it.
20. **A tool counts by kind and by recipe.** Tool crafting gives a tool to
    the tool slot and raises `Gained { item: Tool(..), how: Made }`, whose
    item has no tier, so `tool:pickaxe` is any pickaxe. The station collect
    (`desktop/craft.rs`, every station and the hands) also raises
    `missions::Did(craft, recipe:<code>)`, which is how "Make a metal tool"
    names `recipe:bronze-axe` and the rest.
21. **A world joins where it stands** (refines decision 10). State-shaped
    objectives are checked when the world opens and after every count, and
    a mission the world holds whole closes even while locked; closing a
    mission closes the missions it needs, in order. So an old world, which
    owns all four tools at the stone tier and has a campfire placed, opens
    at "Go fishing", and a new world (no tools, no free stations since
    tool crafting) opens at "Gather by hand".
22. **The chain's order** follows the proposal with two moves: "Farm
    plants" (harvest) and "Collect from the herd" come after "Make a metal
    tool", because a crop ripens in four to six world days and an egg comes
    once a day; planting and taming come early so that the waiting happens
    while the player smelts. Those two are the missions that cannot be one
    to three minutes; every other count is.

## Missions that show off the game, and a bot that plays them (owner, 2026-11-15)

The owner: *"think about the missions systems, design missions that show off
gameplay and make sure the bot completes it"*. This came after the demo rule
in CLAUDE.md ("Demos: tech montages and a bot playing to a goal").

**Two goals for the bot.** The phase-1 showcase (`step-videos` 5.2) is a bot
whose stated goal is a mission chain. It plays the chain through the
player's inputs, and the missions' own completion lines prove it. The goals
are:
1. **"Finish the starter missions"**: the nineteen that exist, unchanged. They
   teach each verb once.
2. **"Homestead"**: a second chain, opened when the starter chain is done. It
   is built to show the verbs working together, each mission a small story
   with a payoff on screen.

**The Homestead chain** (new ids, append-only; counts kept small so the bot's
run stays watchable):

| id | title | objectives | what it shows |
| --- | --- | --- | --- |
| `shore-lunch` | Shore lunch | catch 2 fish of 2 species; cook 2 `grilled-fish`; **eat** 1 | the rod's bite and reel, species, the campfire, eating with its effect icon |
| `bread` | Bread from your field | till 4; plant 4 `wheat-seed`; harvest 4 `wheat`; cook 2 `flatbread` | a field from bare grass to bread, on a sped-up clock |
| `flock` | Start a flock | tame 2 `yard hen`; build a pen with a gate; collect 2 `egg`; cook 1 `fried-egg` | taming by feeding, fences on hex edges, the coop's eggs into the pan |
| `bronze` | The bronze age | smelt 2 `bronze-ingot`; craft `tool:pickaxe` at bronze; dig 3 stone with it | the forge's alloy and a faster pick, the dig time shown |
| `lamplight` | Light the farm | place 4 lights at the pen and field; be there at night | the lit-at-night look on the player's own farm |
| `feast` | A farm feast | **eat** 3 different dishes | every effect icon at once: the payoff of the whole loop |

**Two objective kinds are added.**
- **`eat`**: a dish eaten from the hotbar. Eating exists (`cooking`), and
  the event is already raised where the effect starts.
- **`fell`**: a tree felled with the axe, now that felling is built
  (`tool-crafting`). `fell` stops being "pending".

`lamplight`'s "be there at night" is `place` of lights plus a new **`night`**
objective: the clock passes 20:00 with the player within 30 m of a light
they placed. If a new kind is not wanted, `night` falls back to `place`
alone.

**What the bot does and does not do.**
- It plays every objective through the player's inputs (`step-videos` 5.1).
- It may speed the clock while it waits on a crop, an egg or the night, and
  a caption says so.
- It never sets a mission's count or an item directly. A mission the bot
  cannot finish fails the run: `SHOWCASE_END` is not printed and the log
  says which objective stalled.
- The test of the showcase is that both chains are done in the save at the
  end.

**Saves.** New mission ids only, appended. A world from before has none of
them, and opens with the Homestead chain locked behind its starter chain as
usual.

## Missions are a tree that unlocks the tech (owner, 2026-11-15)

The owner: *"I expect the bot to play through missions. The point of the
missions is to get the player to play through the content and unlock new
tech. So an example of missions could be craft an axe, gather 10 wood, make
some planks, build a craft bench then make a fishing rod, go fishing. Another
branch would be farming, and other branch could be herding and another branch
could be mining and smelting, and later on down the line cooking"*.

This replaces the single line of nineteen and the Homestead chain above. The
same verbs and many of the same ids are rearranged into a tree, and finishing
a mission **unlocks** the recipes and stations of the next ones.

**The trunk: tools and fishing.** Each mission unlocks what the next needs.

| id | mission | unlocks |
| --- | --- | --- |
| `axe` | Craft an axe (by hand: stone, fibre, a stick) | felling trees |
| `wood` | Gather 10 wood (fell trees with the axe) | planks |
| `planks-rope` | Make planks and rope | the crafting bench |
| `crafting-table` | Build a crafting bench | the bench's recipes: the rod, stone tools |
| `rod` | Make a fishing rod | casting |
| `go-fishing` | Go fishing: catch 3 fish | **the branches open** |

**Four branches** open off the trunk. They can be played in any order, and
each ends in a payoff that unlocks the next tier:

- **Farming:** `find-seeds` (dig grass, pick wild plants), then `till` (a
  hoe or shovel, unlocked here), then `plant` 4, then `harvest` 4, which
  unlocks the next crops' seeds and the compost.
- **Herding:** `fence` (unlocks fences and gates), then `tame` an animal by
  feeding, then `pen` (a pen with a gate), then `collect` eggs, milk or wool,
  which unlocks the coop and the second species.
- **Mining and smelting:** `mining-tools` (a stone pick), then `ore` (dig
  copper and tin), then `forge` (unlocks the forge), then `smelt` (bronze),
  then `metal-tool`, which unlocks bronze tools, then iron.
- **Cooking, later down the line:** it opens when fishing and one of farming
  or herding are done. `fire` (the campfire, unlocked here), then
  `cook-fish`, then `farm-meal` (a dish from a crop or an egg), then
  **eat**, which unlocks the kitchen stations when `cooking` builds them.

**Unlocking.**
- A recipe or station is **locked** until its mission is done.
- In the recipe list it shows as a dim silhouette, with the name of the
  mission that opens it. The game says where to go next and shows no other
  explanation (CLAUDE.md, "Notes stay out of the game").
- The unlock list is data on each mission (`unlocks: [recipe codes]`) in
  `missions.ron`, so tech and missions are tuned in one file.
- A recipe no mission names is unlocked from the start.

**Saves.**
- An unlock is derived from the world's `missions` record and never stored
  on its own.
- A world from before the tree opens with every mission it already
  finished done.
- Any recipe it has already crafted, and anything the player carries or
  placed, counts as unlocked too, so no old world loses a tool it could
  make. A test pins this.
- Mission ids stay append-only. The old line's ids map onto the tree's,
  and new ids are added for the new steps (`wood`, `fence`, `ore`, `forge`,
  `eat`).

**The bot plays the tree** (CLAUDE.md, "Demos: tech montages and a bot
playing to a goal"). Its goals:
1. "Finish the trunk": axe to fishing.
2. Each branch as its own goal and video section: farming, herding, mining
   and smelting.
3. Cooking last.

It plays each mission through the player's inputs. The proof is the
mission's own completion line and the unlock appearing in the recipe list.
The Homestead missions above are kept only as later goals for a branch's
payoff, not as a second chain.

### The full tree (owner, 2026-11-15: "make a comprehensive mission tree that would encourage the bot to try all these game play mechanics")

Every mechanic the game has today has a mission that needs it: hands, tools
at three tiers, building, light, water, fishing, farming, herding, mining
and smelting, cooking and eating, and getting about. Names are the real
recipe, good and species codes (`recipes.ron`, `pbd_core::goods`,
`husbandry.ron`, `fauna.ron`, `farming.ron`). **New** marks an objective
kind or a recipe the game does not have yet. Every mission's `unlocks`
lists what it opens.

**0. Trunk: hands to tools** (in order; it opens everything else)

| id | mission | objectives | unlocks |
| --- | --- | --- | --- |
| `gather` | Gather by hand | gather 4 fibre, 4 stone, 4 soil by hand | `stone-axe` |
| `axe` | Craft an axe | craft `stone-axe` (in the pack) | felling |
| `wood` | Gather wood | **fell** 2 trees; gather 10 `log` | `planks`, `rope` |
| `planks-rope` | Planks and rope | craft 8 `planks`, 2 `rope` | `workbench` |
| `crafting-table` | Build a crafting bench | craft and **place** `workbench` | `stone-pickaxe`, `stone-shovel`, `rod` |
| `stone-tools` | Stone tools | craft `stone-pickaxe`, `stone-shovel` | the four branches |

**1. Fishing** (opens after the trunk)

| id | mission | objectives | unlocks |
| --- | --- | --- | --- |
| `rod` | Make a fishing rod | craft `rod` | casting |
| `go-fishing` | Go fishing | cast onto water; catch 3 fish | `bucket` |
| `bait` | Dig for bait | dig bait from wet ground; catch 1 fish with bait | — |
| `three-kinds` | Three kinds of fish | catch 3 different species (the field guide, J, fills in) | the cooking branch, with farming or herding |
| `sea-fishing` | Off the shore | catch 1 fish from the sea and 1 from a river or lake | — |

**2. Farming**

| id | mission | objectives | unlocks |
| --- | --- | --- | --- |
| `find-seeds` | Find seeds | dig grass for a seed; pick 1 wild plant | tilling with the shovel |
| `till` | Break ground | till 4 cells | — |
| `plant` | Plant a field | plant 4 `wheat-seed` | — |
| `water-field` | Water the field | **pour** a `water-bucket` beside the field (**new** kind `pour`) | — |
| `harvest` | Harvest | harvest 4 `wheat` | `turnip-seed`, `chili-seed` |
| `three-crops` | A mixed field | harvest `turnip`, `chili` and `frost-kale` (kale where it is cold) | the cooking branch's farm dishes |

**3. Herding**

| id | mission | objectives | unlocks |
| --- | --- | --- | --- |
| `fence` | Make fences | craft 12 `fence`, 1 `gate` (**new** recipes, from planks and rope) | — |
| `tame` | Tame an animal | tame a `Yard hen` by feeding | — |
| `pen` | Build a pen | build a pen with a gate round tamed animals | — |
| `collect` | Collect from the herd | collect 2 `egg` | — |
| `flock` | A mixed herd | tame a `Woolback` or a `Plodder`; collect `wool` or `milk` | the egg dishes |

**4. Mining and smelting**

| id | mission | objectives | unlocks |
| --- | --- | --- | --- |
| `dig-deep` | Dig into the hill | dig 10 stone with the stone pickaxe | `forge` |
| `ore` | Find ore | dig `mixed-ore` (copper and tin) and `coal` | — |
| `forge` | Build a forge | craft and place `forge` | `bronze` |
| `smelt` | Smelt bronze | smelt 2 `bronze-ingot` | `bronze-axe`, `bronze-pickaxe`, `bronze-shovel` |
| `bronze-tool` | Bronze tools | craft `bronze-pickaxe`; dig 5 stone with it | iron smelting |
| `iron` | The iron age | smelt `iron-ingot`; craft `iron-pickaxe` | `iron-axe`, `iron-shovel` |

**5. Cooking** (opens after fishing's `three-kinds` and one of farming or herding)

| id | mission | objectives | unlocks |
| --- | --- | --- | --- |
| `fire` | Make a fire | craft and place a campfire (**new** recipe) | the campfire's dishes |
| `cook-fish` | Cook fish | cook 2 `grilled-fish` | — |
| `bread` | Bake bread | cook 2 `flatbread` from your wheat | — |
| `eat` | Eat well | **eat** 1 dish (**new** kind `eat`); its effect shows | — |
| `farm-meal` | A farm meal | cook `roast-turnip`, `kale-skewer` or `chili-fish` | — |
| `fried-egg` | Breakfast | cook a `fried-egg` from your hen's egg | — |
| `feast` | A feast | **eat** 3 different dishes | the kitchen stations, when `cooking` builds them |

**6. Light and building** (opens after the trunk)

| id | mission | objectives | unlocks |
| --- | --- | --- | --- |
| `build` | Build a shelter | **place** 20 blocks (planks, stone or soil) | torches |
| `torch` | Light the night | craft and **place** 4 torches (**new** recipe); be beside them after 20:00 (**new** kind `night`) | lanterns, when they get a recipe |

**7. Getting about** (opens after fishing; each goal is something to see)

| id | mission | objectives | unlocks |
| --- | --- | --- | --- |
| `map` | Read the map | open the map (M) and **reach** a marked place 200 m away (**new** kind `reach`) | — |
| `swim` | Into the water | **swim** 20 m (**new** kind `swim`) | — |
| `reef` | See a reef | **reach** a reef or kelp forest under water | — |
| `town` | Find a town | **reach** a town | — |
| `boat` | Take a boat | **board** a rowboat or a cog and travel 100 m (**new** kind `board`) | — |

**New kinds:** `fell`, `eat`, `pour`, `night`, `reach`, `swim`, `board`.
Each is raised where the game already acts: felling, eating, the bucket,
the clock, the walker's position against a site or the sea places map, the
swimmer, and boarding.

**New recipes:** `fence`, `gate`, `campfire` and `torch`. Today they come
in the starting kit. The kit keeps them for a world from before the tree
(T.3).

**The bot.** It plays the trunk, then each branch in order: fishing,
farming, herding, mining and smelting, cooking, light and building, getting
about. Each branch is a goal and a section of the video. Waits are sped up
and captioned. A branch the bot cannot finish fails the run and the log
names the stalled objective. That is the test that every mechanic is
reachable by play.

### One parent per mission (owner, 2026-11-15)

The owner, on M33 version 2: *"the mission tree is a mess, please dont make
multi dependencies. a child node cannot have multiple parents, parents can
have multiple child nodes. if something depends on something to be done,
simply dont unlock it until the dependent thing is done. space nodes out so
the connections are clear"*.

The tree above is therefore a strict tree:
- **Every mission has exactly one parent**, the one line drawn. A mission
  may have many children.
- **An extra requirement is a hidden gate** (`after` in the data). The
  mission stays locked until the gate is done, and no line is drawn for it.
- The tree is laid out from its structure, left to right. A branch is one
  straight line, and side chains hang below it.

What moved, so the real needs still hold:
- **`fire` (Make a fire) joins the trunk** after `stone-tools`. Every
  branch hangs off it, so the campfire is there before any cooking.
- **Cooking is spread to where its food comes from:**
  - `cook-fish`, then `eat`, hang off `go-fishing`;
  - `bread` hangs off `harvest`;
  - `farm-meal`, then `feast`, hang off `three-crops`;
  - `fried-egg` hangs off `collect`.
- **`water-field`** keeps `plant` as its parent, with a hidden gate on
  `go-fishing` (the bucket).
- **The dotted "uses" links are gone.**

`needs` in `missions.ron` becomes one parent per mission. The all-of
groups used today (cooking on fishing and `harvest`) are replaced by this
placement plus `after` gates.

### Zoom by whole pixels; converging links as an option (owner, 2026-11-16)

The owner sent two pictures: the tab's card frame tearing at its corner
(`docs/screenshots/mockup-mission-tree/owner/nine-slice-seam.png`), and a
reference tree with hex nodes joined by chevron links
(`.../owner/reference-converging-tree.webp`). Their words: *"an idea if you
want a node to have multiple parents. Also lets make zoom in integers so that
we dont have 9 slice seems - or virtualize the zooms"*.

**Zoom.** Today the tab scales the whole tree by any factor (`UiTransform`,
x1.2 a wheel step, 0.2 to 2.0), so a 9-slice frame lands between screen
pixels and its slices pull apart. Instead:
- **Cards only at whole-pixel scales.** A card's art is drawn only at a
  scale where one art pixel is a whole number of screen pixels:
  `UiScale x zoom` is an integer, so `zoom = k / UiScale` for k = 1, 2, 3....
- **Below 1:1, zoom is virtual.** The cards keep their 1:1 size, and
  zooming out shrinks the spacing between them (and so the canvas). At the
  farthest steps a card shows only its icon and title. Zooming in uses the
  integer scales above.
- **The pan snaps to whole screen pixels.**
- **The wheel steps through the levels.** F fits to the largest level
  that shows the whole tree.
- **The mockup does the same,** in CSS pixels times the device pixel ratio.

**Converging links (an option, not built).** The approved tree has one
parent per mission. If a node is ever to have more than one parent, the
reference shows how it can read clearly:
- links are directed: chevrons point from parent to child;
- several links can flow into one node, like streams joining;
- the node opens when all of its parents are done.

This stays a proposal until the owner asks for it. The strict tree remains
the approved design.


### Goals worth doing: more of the common things (owner, 2026-11-16)

The owner, on M33's "Find seeds" card ("dig grass for a seed 0/1, pick a
wild plant 0/1"): "for items like this, I think there should be more.
instead of get 1 resource, more common resource should be either 5-10, for
uncommon things like fish, 3-5 is fine", and "crafting 1 item is fine,
unless a recipe requires multiples".

**Rule.** A goal to gather a common thing asks for 5 to 10; an uncommon
thing (fish, ore, coal, the less common crops, wool and milk) 3 to 5; a
goal to craft or place asks for 1, unless the recipe the player is heading
for takes several, in which case it asks for what that recipe takes.

**The counts**, in `assets/config/missions.ron` (the game) and M33's
`docs/mockups/mission-tree/tree.js` alike:

| Goal | Was | Now | Why |
| --- | ---: | ---: | --- |
| Gather fibre | 4 | 8 | common |
| Gather stone | 4 | 9 | common; three stone tools take 3 each |
| Gather soil | 4 | 6 | common |
| Find seeds: dig grass, pick a wild plant | 1 + 1 (the game: 2 seeds) | 5 + 5 (the game: 10 seeds) | common |
| Till; plant wheat; harvest wheat | 4 | 6 each | common field work |
| Harvest turnip, chili, frost kale | 1 each | 3 each | uncommon crops |
| Collect eggs | 2 | 5 | common |
| Collect wool or milk | 1 | 3 | uncommon |
| Dig bait; catch with bait | 1 | 3 | uncommon |
| Catch in the sea; in fresh water | 1 each | 3 each | fish |
| Dig mixed ore | 1 | 4 | the forge's smelt takes 4 |
| Dig coal | 1 | 3 | uncommon |
| Smelt iron | 1 | 3 | the iron pickaxe takes 3 ingots |

Unchanged: crafting and placing at 1, or at the multiple a recipe needs
(fences, planks, rope, bronze, torches); fish caught (3), species (3), logs
(10), stone dug (10), and the explorer's goals.

### The mission card's frame (owner, 2026-11-16: "9 slice lines broo")

The mockup's cards and log drew the kit's `tooltip` piece whole. That PNG
(24 x 18 native, 8 render pixels a native pixel) keeps one native pixel of
render background round the slab, and its right and bottom bevels fall into
near-black shade. Stretched down a card's side, those two outer columns read
as a black stripe and then a pale blue line outside the frame: the seam
lines the owner saw. The mockup now slices a copy trimmed by that native
pixel on every side (`mission-tree/ui/*-tooltip-trim.png`, slice 40 = 5
native, border 10 px so a native pixel stays 2 CSS pixels). The game's
mission tree (`desktop/mission_tree.rs`) draws the same `tooltip` piece; the
lasting fix is in the art (`tools/art/ui_panels_blend.py` `r_tooltip` and
`r_oak_tooltip`, whose outside should render transparent), which needs
Blender and waits for a request.

The overview level (TS.3's level 0) showed bare frames; the owner on it:
"bad, wheres the icons". Level 0 is now the tree of icons (52 px cards,
each its mission's icon in a 32 px slot), and level 1 keeps the icon
beside the title.
