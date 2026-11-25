# Playtest to-do, 2026-11-22

One list of every note the owner gave during the 2026-11-22 playtest, and
the work still open from the bot run before it. Each item names the change
it is written up in. Its own `tasks.md` carries the same line.

Status words:
- **written**: the plan is in the change's design;
- **coded**: the source is edited in the working tree;
- **built**: compiled with `build-play --full`;
- **tested**: `cargo test` passes for it;
- **seen**: checked in the game, with screenshots in `docs/screenshots/`.

**Nothing below is built or tested yet.** The owner is playtesting, and no
build runs while a game they started is open (CLAUDE.md, "No builds while
the game is running").

## After the playtest, in this order

1. `cargo fmt --all`, then `build-play --full` (the release build).
2. `cargo test -p pbd-core` and `cargo test -p pbd-app` (missions, locale,
   station, craft, drops, digging, clutter, pack, menu). Fix what fails.
3. `cargo clippy` on the touched crates; `openspec validate --all`.
4. In the game, one item at a time, with screenshots (the checks below).
5. Commit to PR #144 only when the owner asks.

## Where it stands (2026-11-23, 01:00)

Every note below is coded and in the fast build (`target/fast/pbd-app.exe`,
00:53). Tests: `pbd-core` has only the 10 old ground-digest failures, which
also fail on a clean HEAD. `pbd-app` has only the old failures: the fish
wiki, the shader, lens and light-constant fixtures, and 2 vehicle fixtures,
all line-ending (CRLF) problems. `openspec validate --all` passes.

Bot run on `tree-run-6`, with the F3 graph and `--frame-log`:
- **reef** done (map goal, swim, mine reef rock);
- **town** done;
- **boat**: the raft is made; the bot walks into the sea and cannot place it
  there. Open.

Performance found and fixed tonight:
- **The bot's planner held the frame:** a plan took up to 1.85 s on the main
  thread. Now a 15 ms budget per plan, and the route's avoid test runs only
  on the cells the search reaches. Worst plan 1853 ms → 67 ms; frame p99 744
  → 75 ms on the same shot (`bot-path-planner` 13).
- **The reef's sway:** 401,000 vertices bent on the main thread each frame,
  28 ms. Now bent on every core: 15.6 ms. At the reef, 15 to 19 fps went to
  41 fps (`ocean-environments` B31). The full fix, the sway on the GPU, is
  written up and not built.
- **Swamp water:** the pool test is cached like the rivers: in memory, in the
  cache file and filled ahead (`swamps`). Not yet timed in a swamp.

Not seen in the game yet (needs the owner or a capture):
- the X buttons' place;
- the worlds scroll bar;
- the dust and leaves;
- the "1/3" counts;
- sprite drops;
- reefs at 10% on a new world (generator 16), which goes to the survey
  with screenshots.

## The owner's notes

### 1. The pack mission: Tab to open the pack, after gathering
- Change: `starter-missions`, "The first mission: open the pack" (revised
  the same day: after `gather`, not before it).
- Status: written, coded.
- What: a mission `pack` (one objective, `open pack`, 1) between "Gather by
  hand" and "Craft an axe". Its help: "Press Tab to open your pack and craft
  with what you gathered." Opening the pack (Tab or I) sends
  `Did(Open, "pack")`. `settle` back-fills a done mission's parents, so old
  worlds never meet it mid-tree.
- Files:
  - `assets/config/missions.ron`;
  - `assets/locale/locale_en.yaml` (`mission.pack`);
  - `crates/pbd-core/src/missions.rs` (`Kind::Open` names, the `settle`
    back-fill);
  - `crates/pbd-app/src/desktop/missions.rs` (the event, the walkthrough);
  - tests in `crates/pbd-core/src/missions/tests.rs` (44 missions, the
    pack event after gathering) and `crates/pbd-app/src/missions/tests.rs`
    (the line goes gather → "Open your pack 0/1" → "Craft an axe 0/1").
- Check: a new world; gather; the HUD reads "Open your pack 0/1"; Tab; it
  ticks and "Craft an axe" shows. An old world (`tree-run-6`) never shows
  it.
- Open: a pack icon (it uses `tools/hand` for now).

### 2. Gathering rocks and fibre 4x faster
- Change: `gather-by-hand`, "Taking a piece is 4x quicker".
- Status: written, coded.
- What: a taken pebble or bush sinks in `scatter.taken_sink_s` = 0.25 s (it
  borrowed `lod_fade_s`, 1.0 s). The taken list is rebuilt on the frame the
  save changes, not up to 0.5 s later.
- Files: `assets/config/scatter.ron`, `crates/pbd-app/src/config.rs`
  (the field, its default and its validation),
  `crates/pbd-app/src/clutter.rs`.
- Check: press F on a pebble; it is gone in about a quarter of a second. The
  ground detail's own fade is unchanged.

### 3. Reefs: "Get a reef", not "See a reef"
- Change: `starter-missions`, "Get a reef, not see one".
- Status: written, coded.
- What: the `reef` mission is `gather block:reef rock` 1. Its title is "Get
  a reef", its help "Dive to a warm reef and mine some reef rock with a
  pickaxe."
- Files: `assets/config/missions.ron`, the locale file, the core and app
  mission tests, `assets/showcase/phase-1-last.ron` (the bot's reef shot).
- Check: mine reef rock with a pickaxe; the mission ticks.

### 4. Reefs to about 10% of the sea
- Change: `ocean-environments`, B30.
- Status: written only.
- What:
  - measure today's shares on generator 15 (`examples/ocean_places.rs`);
  - grow reef toward 10% of the sea, taking from kelp in warm water. The
    levers: the temperature floor toward 18 C, a deeper band, wider patches,
    a lower `reef_above`;
  - a new generator version 16, for new worlds only, with a digest test of
    15 first;
  - to the owner's survey with screenshots (the Sea places map, 15 against
    16, and a reef under water) before it is the default.

### 5. The bot finds a reef from the map (global planning)
- Change: `mission-playthrough`, 5.8 (and 5.7, the swim to it).
- Status: written. The 20 m ring search for reef rock is coded as a
  fallback (`nearest_reef_rock` in `showcase.rs`). It is ahead of its
  write-up, and comes out once `MapGoal` works.
- What: a `MapGoal(layer, class)` act:
  - open the map and pick the Sea places layer;
  - find the nearest reef pixel and zoom in to make sure of it;
  - right-click a mark there;
  - `ToMark` in 120 m legs, then the swim (5.7): to the coast, into the
    water, across, and down to the rock.
- The same act serves towns, climates and the boat's bay.
- Check: a screenshot of the map with the mark on a reef; "mission done:
  reef" in the run's log.

### 6. A scroll bar for the worlds list
- Change: `save-slots`, "The worlds list scrolls".
- Status: written only.
- What: the rows go in a scroll area, scrolled with the wheel or the bar.
  NAME, NEW WORLD and BACK stay fixed under it. The bar shows only when the
  rows overflow.
- Check: screenshots with 12 and 30 worlds.

### 7. A dropped item that is not a block shows its sprite
- Change: `drop-and-trash`, section 5.
- Status: written, coded.
- What: `slots::own_icon` is one table for the pack and drops. Fish, tools,
  lights, goods, the forge, the workbench and the chest drop as a card
  wearing their icon. Until now, tools, fish and station blocks dropped
  undrawn. Terrain blocks stay prisms.
- Files: `crates/pbd-app/src/desktop/slots.rs`,
  `crates/pbd-app/src/desktop/drops_view.rs`.
- Check: with Q, drop an axe, a fish, a seed, a workbench and a dirt block;
  a screenshot of each on the ground.

### 8. A crafted tool goes to hand; spares to the pack; make as many as you like
- Change: `tool-crafting`, "A made tool goes to hand; a spare goes to the
  pack" (with its revision).
- Status: written, coded.
- What:
  - a new or better tool is owned and held (`Equipment::hold`) the moment
    it is collected;
  - any other copy goes into the pack as `Item::Tool`;
  - nothing about a tool is refused any more: `Refused::NotBetter` and
    "You already have this tool" are gone. Spares are needed for tool wear
    later.
- Files: `crates/pbd-core/src/station.rs` (`queue_for`, `collect_into`, the
  test `a_tool_goes_to_hand_and_a_spare_to_the_pack`), `craft.rs`,
  `forge.rs`, `kitchen.rs`, the locale file.
- Check: make a pickaxe and it is in hand; make two more and they are in
  the pack.
- Open: a tool in the pack has no tier of its own; tool wear will need one.
  Moving a pack tool into the tool slot is not built.

### 9. Tab closes whatever is open
- Change: `stone-and-wood-ui`, "Tab closes what is open".
- Status: written, coded.
- What:
  - in the world, Tab opens the pack;
  - over the world, Tab closes any screen: pack, station, chest, missions,
    map, guide, breeding, pause;
  - on the title, settings and worlds pages it does nothing;
  - I keeps its own toggle.
- Files: `crates/pbd-app/src/desktop/pack.rs` (`open`).
- Check: open each screen, press Tab, and it closes. The pack test
  (Tab opens and shuts it) still passes.

### 10. An X on every stone panel
- Change: `stone-and-wood-ui`, "an X on every stone panel".
- Status: written, coded.
- What: `menu::lintel` adds a small stone plaque, "X" (`menu.close`), at the
  panel's top-right corner. It acts as `MenuAction::Back`, Esc's step back.
- Files: `crates/pbd-app/src/desktop/menu.rs`, the locale file.
- Check: screenshots of the X on the pack, the workbench, a chest, the
  forge, the kitchen, the map, pause and worlds.
- Tune its place from them: it sits at -10 px off the corner, and a panel
  that clips (`Overflow::clip`) may cut it.

### 11. Recipe inputs show what is carried: "1/3"
- Change: `stone-and-wood-ui`, "What a recipe takes, against what is
  carried".
- Status: coded, then written (the write-up came after the code this once).
- What: every input shows `have/need` on the hands' panel, the workbench,
  the forge and the campfire. The forge and kitchen counts are repainted
  live (`NeedCount`, `craft::paint_need_counts`).
- Files: `craft.rs` (and its unit test), `forge.rs`, `kitchen.rs`,
  `desktop.rs` (the system registered).
- Check: the stone axe with 0, 1 and 3 stone carried. If a two-digit carried
  count crowds the 16 px well, show "9+".

### 12. A standing tree holds only its own block
- Change: `tool-crafting`, "A standing tree holds only its own block".
- Status: written, coded.
- What: `under_standing_tree` refuses only the root layer
  (`layer_at(height - 0.5)`), not whatever is the column's top now. The rest
  of the column digs and tills.
- Files: `crates/pbd-app/src/desktop/digging.rs`.
- Check: dig the block beside a tree's root, then down under it; only the
  root block says "Cut down the tree first".
- Open: a unit test for a block under the root.

### 13. No builds while the game runs
- Done: CLAUDE.md, "No builds while the game is running".

### 14. A punched log takes twice as long
- Change: `tree-punch-shake`, "A punched log takes twice as long".
- Status: written, coded.
- What: a field of its own, `dig.hand_log_speed` = 0.125, for a trunk by
  hand: 4.8 s a log (was 2.4 s), about 19 s a tree. Bare hands on dirt are
  unchanged (`hand_speed`). Still 4 logs a tree, the same as an axe.
- Files: `assets/config/dig.ron`, `crates/pbd-core/src/dig.rs` (the field,
  its default, its validation and test), `crates/pbd-app/src/desktop/digging.rs`.
- Check: punch a log in about 5 s. The bot's `Chop` by hand may need a
  longer `max_s`.

### 15. Dust and falling leaves when a tree is hit or felled
- Change: `tree-punch-shake`, "Dust and falling leaves".
- Status: written only.
- What:
  - on each hit, a puff of bark dust and a few leaves drifting down from
    the crown;
  - on the fell, a bigger burst and a shower of leaves;
  - pixel particles in the tree's and ground's palette, no outline, lit as
    the terrain, capped.
- Next: a mockup beside today's hit, on the tracker, for the owner's
  approval. Then the game, with the frame cost measured on a chop.

### 16. Swamp water loads slowly: it is not cached like rivers
- Change: `swamps`, "Swamp water is not cached the way rivers are".
- Status: written only, from reading the code (not timed yet).
- What was found:
  - every swamp top cell's pool test samples the full height noise 48
    times;
  - its river test scans the drainage map;
  - none of it is in the river cache, so it is redone on every rebuild
    and every launch.
- Next:
  - a `SWAMP_TIME` instrument, measured cold and warm in a swamp and by a
    river;
  - memoize `swamp_pool_held` and the swamp's `beside` in the river cache
    (exact by construction), saved to its file and filled ahead;
  - measure again.

### 17. No wild plant in a tree's cell
- Change: `farming-and-smelting`, "No wild plant in a tree's cell".
- Status: written, coded.
- What: a cell that grows a tree (`flora::grows_tree`) never grows a wild
  plant, whether the tree stands or is felled. The check is in both places,
  where F picks (`wild_at`) and where wild plants are drawn. Derived only;
  no save or ground changes.
- Files: `crates/pbd-app/src/desktop/farm.rs`.
- Check: the birch in the owner's screenshot has no wheat in its trunk.
  Wild plants still show on open grass.

## Open from before the playtest

- **Lag.**
  - The bot's walk: the planner's search cap and how often it re-plans.
    `PLAN_TIME` and `ROUTE_TIME` are logged; the walk still needs measuring
    before and after.
  - Idle, it measured 55 fps (p50 16.7 ms, p99 42 ms).
- **The F3 graph.**
  - Its move top-left over the goals tab (`GlobalZIndex(1000)`) is not seen
    in the game yet.
  - "The red line is continuous": the stall colour and scale need rework so
    rare stalls stand out.
- **Storage chests 2.2.** A chest made, filled, the world reopened, still
  full; screenshots in `docs/screenshots/storage-chests/`.
- **Missions not yet played by the bot:** reef (after item 5), town, boat,
  storage.
- **Measurement logs to take out before a commit:** `PACK_KEY` (`pack.rs`),
  `WALK_TRACE` (`walking.rs`), `PLAN_TIME`/`ROUTE_TIME`, the seek and plant
  `MEASUREMENT` logs (`showcase.rs`), the refused-materials count, the
  head-room refusal's materials (`digging.rs`), and `FISH_CAUGHT` (`fish.rs`,
  optional).
- **Earlier, still open:**
  - the punch's captures and sounds;
  - line speckle over water;
  - floating blocks;
  - the raft's line, the oar's normals, and no-sway, checked in the game;
  - the map herds screenshots.

## Found on the full bot run (2026-11-23)

- **The HUD mission tracker drawn blank, sometimes.** Its frame, its icon and
  its checkboxes show, with no title and no objective words: seen for "Three
  kinds of fish" and "A flock", each in a still taken just after the bot's
  Focus act closed the missions tab; the same act on "Read the map" and an
  earlier "Three kinds of fish" showed the words. The words come from the
  mission chain (the missions tab shows them), and `draw_kit_text` rebuilds a
  `KitText` on any change, with no fade. Not found yet; a repeat is wanted
  with the tracker's `KitText` logged on the frame it draws blank.
  `docs/screenshots/mission-playthrough/2026-11-23-tracker-blank-flock.png`.
  Narrowed (four cases now: "Three kinds of fish", "A flock", "Three
  crops", "Read the map"): blank from the moment the missions tab (O) closes
  after a mission is picked in it, and drawn again once a count changes ("Read
  the map" showed its words a minute later, at "open the map 1/1"). A player
  sees it the same way: O, pick a mission, close. `show_line` returns early
  while the tab is open and sets the words on the first frame after;
  `draw_kit_text` (PostUpdate) should rebuild on that change, so either the
  change is not seen or the glyphs are dropped after it.
- **The full bot run, 36 of 44** (`tree-run-7`, 2026-11-23): the stalls and their bot fixes are in `openspec/changes/mission-playthrough/tasks.md` section 7. Game fixes found on the way: the river cache's save no longer holds the frame (`Memo::put` never waits; the save every 2 minutes on 5% growth), and a cell's record by an index (`FineSet::finest_index_of`) where `material_of` and the tree test searched every record.
