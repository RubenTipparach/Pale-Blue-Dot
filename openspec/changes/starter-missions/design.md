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

