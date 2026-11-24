# Design: building-planner

## Context

`player-building` is built (phase 3, `claude/phase-2-1`): pieces on the
planet's own cells (B1), one rule function each with the first to fail as
the reason (B2), each piece a `piece` record on the durable path (B3), the
plot's pieces cut like a town's (B4), and build mode on N with a palette, a
ghost and a whole-blueprint fill (B5). Its decision 5 makes a blueprint a
template's building; decision 7 makes free building a world setting and the
only one that exists. Doors, the build camera (task 4.5) and the footing
(decision 8) are being built beside this write-up; furniture is its own
change (`furniture`).

The owner (2026-11-23) asks for presets for houses and buildings, for the
player to design structures and build to blueprints instead of free
building, for a building planner that can change a building mid-build, and
for a mockup to fine-tune.

## Goals / Non-Goals

**Goals:**
- A planner screen that designs a structure on the hex grid, checked by the
  same rule functions as build mode.
- A preset library fitting the towns, editable and savable as the player's
  own blueprints.
- A blueprint set out in the world as a construction site, checked whole,
  turned in sixths, built piece by piece from the pack.
- A site changed mid-build: what is left updates, what is dropped comes
  back.
- Blueprints and sites as stored records, old saves unchanged.
- A mockup with the numbers the owner will tune.

**Non-Goals:**
- Furniture itself: the planner marks a furniture slot; what fills it is
  `furniture`'s.
- Towns building from player blueprints (phase 4, town building).
- Multiplayer sharing of blueprints (phase 3b carries records already; a
  shared library is later).
- New kits or materials. Kits whose material does not exist yet (brick) are
  shown and locked in planned worlds, "Needs bricks".

## Decisions

**1. A plan is in its own coordinates, charted onto cells when set out.**
- A plan is a list of plan pieces on a flat hex grid of its own: axial
  `(q, r)` from the plan's origin cell, a side `0..5`, a storey `0..`. The
  same six kinds as `building::Shape`, plus a furniture slot:
  - `Wall { cell, side, storey, opening }` (outer walls and inner walls);
  - `Floor { cell, storey }`;
  - `Stair { foot, side, storey }` (the head is the foot's neighbour across
    `side`);
  - `Post { cell, corner, storey }`;
  - `Lantern { cell, storey }`;
  - `Roof { storey, shape }`, fitted over the storey's cells not under a
    floor of the storey above, by `blueprint::fit_roof`;
  - `Slot { cell, storey, side, kind }`, a furniture slot: `kind` one of
    `furniture` design 2's pieces (bed, table, chest, bench, chair...), `side`
    0 to 5 for its back to that side or 6 for the cell's middle, exactly as
    `building::Shape::Furniture` (`furniture` design 3) stands.
- Charting a plan onto the planet is the template charting
  `building::blueprint` already does (`chart(patch, first, at, turn, ..)`),
  generalised to plan pieces: the origin on the aimed cell, side 0 on the
  cell's side `turn`, the ground storey on `base_over`. Each plan piece
  becomes the `Piece` build mode would place there, with the plan's kit.
- *Why its own coordinates:* a plan drawn at a table has no place yet, and
  the same blueprint is set out many times, anywhere, at any of six turns.

**2. The planner checks a plan with build mode's rules.**
- The planner charts the plan onto a flat stand-in patch: a hex disc of the
  plan's radius with no pentagons and its ground at storey 0. It runs
  `building::check_layout` on the charted pieces, so `held_up`, `headroom`,
  `stair_cut`, `landing`, `room`, `roof_overlap` and `opening` are the
  game's own functions, with their `build.refused.<rule>` reasons.
- The planner's own checks are only what a plan has and a piece does not:
  - `plan_size`: more cells than `max_cells` or wider than `max_radius`
    (so a site always cuts in one plot, `PLOT_M`), "Too big to plan";
  - `storeys`: more than `max_storeys`, "Too many storeys";
  - `way_in` (a warning, not a refusal): no door on the ground storey,
    "No way in".
- A broken rule shows on the plan where it breaks: the piece red, its
  reason in the checks list. A plan with a refusal can be saved as a
  draft; it cannot be set out.
- *Why not new rules:* one rule set for towns, build mode and the planner is
  what `player-building` decision 3 is for. A plan accepted at the table is
  refused in the world only by the world (decision 5).

**3. The planner screen.**
- **Opened** at a drafting table (`drafting-table`, a station made at the
  workbench from 6 planks and 2 rope, its recipe unlocked by the town tree's
  house mission), with F; or in build mode with O (free in build mode, from
  B5's list: I, K, N, O, U, Y; recommendation taken, ask only with
  screenshots). Aimed at a site or at a player's house, O opens that one.
- **Layout**, in the stone-and-wood UI (M29): the plan in the middle, seen
  from above, one storey at a time with the storey below faint under it;
  the presets and the player's blueprints on the left; the checks and the
  bill of materials on the right; the storey picker, the kit and the roof
  shape over the plan. A turnable 3D preview of the whole plan uses the build
  camera (`player-building` task 4.5) once that is built.
- **Tools:** Room (paint and erase cells on this storey; outer walls follow
  the room's edge), Wall (an inner wall on an edge), Door and Window (on any
  wall), Stair (a foot cell, then the wheel turns it), Lantern, Slot (bed,
  table, chest, bench, chair, any of `furniture` design 2; the wheel turns it), Erase. Ctrl-Z undoes.
- **Outer walls are drawn by the room.** Painting cells makes their outside
  edges walls; an opening on an outer edge stays when the room grows past it
  only if the edge is still outside. So a plan cannot have a missing wall by
  accident, which is the jankiness free building has.
- **Names.** A saved blueprint takes a name the player types; the presets'
  names are locale keys (`blueprint.preset.<id>`).

**4. Presets: data, sized, and the towns' own.**
- `assets/config/blueprints.ron` holds the presets as plans, each with an id,
  a locale key, its kit, and three sizes (small, standard, large). Ids are
  saved in a site's record, so a preset id is never renamed once shipped.
- The library at first (standard size, cells x storeys):
  | Preset | Kit | Small / standard / large (cells x storeys) | Notes |
  | --- | --- | --- | --- |
  | Straw hut | straw | 1x1, 1x1, 2x1 | the first night's shelter, cone roof |
  | Cottage | timber | 2x1, 3x1, 6x1 | a door, windows, table, bed and chair slots |
  | Two-storey house | fieldstone | 3x2, 6x2, 8x2 | a stair and its landing, beds upstairs |
  | Townhouse | half-timbered | 6x2, 6x3, 8x3 | narrow, for a town lane, a stair each storey |
  | Workshop | timber | 4x1, 6x1, 8x1 | a double door, a bench slot |
  | Shop | half-timbered | 3x1, 6x1, 8x1 | front windows, a counter (table) slot |
  | Barn | timber | 6x1, 8x1, 12x1 | a double door, store slots |
  | Stable | timber | 3x1, 4x1, 8x1 | a double door, for the horse cart (`land-transport` 10) |
  | Warehouse | fieldstone | 6x1, 12x1, 15x1 | a double door, chest slots |
  | Tower | fieldstone | 7x2, 9x3, 12x3 | a stair a storey, a flat roof with a parapet |
- Doors, windows, lanterns and slots are laid out by rule from the
  footprint (the door on the front's middle edge, two side by side for a
  double door, windows spread round each storey, a lantern in each
  storey's middle), so a size is only a footprint and a storey count. The
  mockup's three sizes are these; every one passes the planner's checks.
- **Every plain building of the settlement templates** (`assets/settlements/`,
  `blueprint::plain`) is in the library too, under the town's name, as
  `player-building` decision 5 has them. So the towns' houses are presets
  without being copied by hand.
- A preset is edited like any plan; saving it makes the player's blueprint
  (decision 9). The preset itself never changes.

**5. Setting out a site: checked whole against the world.**
- In build mode the Blueprint slot holds the presets and the player's
  blueprints (the wheel steps them, as B5's blueprint choice). The whole
  plan is a ghost at the aim; the wheel with shift turns it a sixth
  (`rotation_step`, 1 sixth; the grid allows nothing finer).
- The whole plan is checked before anything is set out, the first failure
  named:
  - charting: a pentagon under or beside it, "A five-sided cell";
  - the ground: more than `max_footing` layers between its highest and
    lowest cell under the ground floor, "Too steep here" (a footing,
    `player-building` decision 8, fills up to that);
  - in the way: a solid block (a player's edit or a rock above the ground
    floor), a tree, a road cell or a town's footprint, "Something is in the
    way". Each of these takes only the block it is on, never its whole
    column (owner, 2026-11-23: "The tree being in the way thing should only
    apply to the block three is actually on. Not the whole column"): a tree
    refuses only the plan's pieces in its cell on the ground storey, so a
    storey can stand over it once that cell is left out of the ground floor,
    and a road cell or a lamp likewise;
  - existing pieces: any piece of the plan failing the slot rule against
    pieces already in the world, "Something is already here";
  - and `check_layout` on the charted pieces with the world's own, so a plan
    against a town's wall is refused as a single piece would be.
- **Place sets it out.** It costs nothing and takes nothing: the site record
  is written (decision 9), survey stakes stand at its outer corners and lime
  lines are painted along its outer walls into the cells' tops, in the
  paving marks' look and bits (`townsfolk` 39a: lime blended into the
  biome's own top, ragged, lit as the ground is; stakes of split timber with
  dyed cloth, one form each, `LitByField` and `LitLikeTerrain`).
- **The ghost.** Every piece still to build is drawn translucent, in the
  piece's own cut (as build mode's ghost, B5), tinted `ghost_tint` at
  `ghost_opacity`. Pieces that can be built now (held up by what is built)
  are at full ghost strength; pieces waiting on others at
  `ghost_waiting_opacity`. No outline: the ghost is a tint of the shape,
  never an edge drawn round it. A ghost is a preview and is not lit by the
  field; it is drawn only within `ghost_reach_m` of the player and on the
  map.

**6. Building a site, piece by piece.**
- **Mallet in hand, materials in the pack, interact on a ghost.** With the
  mallet (`tool:mallet`, a tool made at the workbench from 1 log and 1 stone,
  unlocked with the drafting table) in the tool slot, F on a ghost piece
  within reach (4 m) that is buildable now strikes it. A piece takes
  `strokes` strokes on the mallet's stroke beat (`held-swing`); held, F keeps
  striking and moves to the next buildable piece in reach, so walking a
  wall line with F held builds it, as paving does (`townsfolk` 40).
- On the last stroke the piece's goods are taken from the pack and the piece
  is placed: an ordinary `piece` record, through the same placement check as
  build mode, with the pack in the same journal line. Refused, nothing
  taken, with one plain line: "Needs a mallet", "Needs 4 planks", or the
  rule's own reason if the world changed under the site.
- **A furniture slot is filled, not struck.** With the slot's piece of
  furniture in the pack (made at the carpenter's bench, `furniture` design
  1), F on its slot places it: an ordinary `Shape::Furniture` piece record,
  checked by furniture's own rules (design 4 there). A slot left empty does
  not keep a site from finishing; it stays in the plan for later.
- **Order.** Any buildable piece can be struck: a floor on the ground, a
  wall on a built floor or the ground, an upper floor once what holds it is
  built, a roof once its room is closed. The compass bar shows a tick on
  the nearest buildable piece, as on a plan's next stake.
- **The HUD** shows the site while the player is within `ghost_reach_m`: its
  name, "12 of 38 left", and the bill of materials still needed against the
  pack, each good by its icon. The planner shows the same for the whole
  plan.
- **Scaffolding** (on by default, `scaffold`): when a storey above the
  ground has a buildable piece, the site puts up scaffolding round that
  storey's outer walls: timber poles lashed with rope, a board walk at the
  storey's floor line, outside each outer edge. It costs
  `scaffold_cost` (2 planks a bay, and a rope for every three bays), taken
  when it goes up and given back when the
  storey's walls are all built and it comes down. It is solid to the walker
  and climbed by its ladder bay, so the upper walls are in reach. A pole is
  one tapered hexagonal post, bound to its boards where they cross, lit as
  the ground is. *Off*, upper pieces are reached from built floors and
  stairs only. The scaffold is the site's waypoint look: like a plan's
  stakes, it shows where the work is next.

**7. Editing a site mid-build.**
- O on a site, or the site in the drafting table's list, opens the site's
  plan in the planner. Saving it changes the site; nothing changes until it
  is saved.
- The new plan is charted where the site is (same origin, turn, base) and
  diffed against the old, piece by piece, by piece id (the slot and class,
  B3):
  - **in both, same kit:** unchanged; built stays built, a ghost stays a
    ghost;
  - **only in the new plan:** a new ghost;
  - **only in the old plan, not built:** gone;
  - **only in the old plan, built:** **marked for removal** (`removal`
    "mark", the default): drawn with a pale chalk wash, struck with the
    mallet like a ghost, and taken down top first (the dependant rule,
    B2's removal), each giving back `refund_share` of its goods to the pack
    or, with a full pack, to a crate at the site;
  - **in both, kit changed:** the old piece is marked, the new kit's is a
    ghost on the same slot once it is down.
- Before saving, the planner lists what the change does: new pieces, pieces
  gone, pieces to take down, and the goods that come back.
- The changed plan is checked whole against the world (decision 5) with the
  site's own built pieces counted as its own; a plan the world refuses is not
  saved.
- `removal` "return" (a tunable, the mockup shows both): dropped built
  pieces come down at once on save and their goods go to the pack. Not the
  default: a wall vanishing from a menu is the jankiness the owner asked to
  avoid, and taking it down is a stroke the player sees.
- **A house outside any site** (built in a free world, or a world's old
  pieces) can be adopted: O on it charts its pieces into a plan with every
  piece built, and saving makes it a site. So every player building can be
  changed the same way.

**8. Costs.**
- `assets/config/building_costs.ron`: goods per piece class and kit, the
  roof per cell of its shape and material. The first table (goods are
  `pbd_core` codes; the kits' material families):
  | Piece | Straw / mud | Timber | Fieldstone, ashlar | Half-timbered |
  | --- | --- | --- | --- | --- |
  | Wall | 3 fibre, 1 log | 4 planks | 5 stone, 1 planks | 3 stone, 2 planks |
  | Door | 2 fibre, 1 log, 2 planks, 1 rope | 5 planks, 1 rope | 4 stone, 3 planks, 1 rope | 2 stone, 4 planks, 1 rope |
  | Window | 2 fibre, 1 log, 1 planks | 4 planks | 4 stone, 2 planks | 2 stone, 3 planks |
  | Floor | none (earth) | 2 planks | 3 stone | 3 stone |
  | Stair | 4 planks | 4 planks | 4 stone | 4 planks |
  | Post | 1 log | 1 log | 1 log | 1 log |
  | Roof, a cell | thatch: 3 fibre, 1 log | shingle: 3 planks | thatch: 3 fibre, 1 log; slate: 3 stone, 1 planks | thatch: 3 fibre, 1 log |
  | Lantern | 1 lantern | 1 lantern | 1 lantern | 1 lantern |
  | Furniture slot | none: filled by placing the furniture item |
- `cost_scale` scales every number (rounded up, never below 1 where the
  table has 1). Kits with a good the game does not have (brick, clay tile,
  marble) are locked in planned worlds with "Needs bricks" and their own
  locale reason.
- Free worlds ignore the table: they cost nothing, as today.

**9. Records.**
- **`blueprint`**, the player's, schema 1, id counted up and never reused:
  its name (the player's text, not a locale key), its kit, its preset id and
  size if it came from one, and its plan pieces. Stored in the world's
  journal like every record, so a world's blueprints are the world's; a
  library across worlds is a later question (it would be a profile file,
  not a world record).
- **`site`**, the player's, schema 1, id counted up: the plan copied in (a
  site never follows a later edit of the blueprint it came from), its kit,
  its chart (origin cell key, turn, base layer), the ids of pieces marked
  for removal, and its scaffold bays standing. Written on set out, on every
  plan change, and when scaffolding goes up or down, with the pack in the
  same line.
- **Built is derived:** a site's piece is built when a `piece` record of the
  same id and kit exists. Nothing in the site record changes when a piece is
  struck, so a site and its pieces cannot disagree, and a piece taken down
  by build mode in a free world is simply a ghost again.
- A site whose pieces are all built and none marked is **finished**: its
  stakes and lime lines go, the record stays (so it can be edited later).
- Both kinds go through `WorldSave::store` as `rec` lines, noted in the
  world's identity by `note_record_kinds` (B3's version bump). The edit log's
  line format does not change. An old save has neither kind and opens
  unchanged; an older build opening a newer world keeps the lines as text
  and draws the built pieces, which are ordinary piece records.

**10. Locale and game text.** Every word is a key in `locale_en.yaml`:
`planner.*` (tools, storeys, checks list, the change summary),
`blueprint.preset.<id>` and `blueprint.size.*`, `site.*` ("{left} of
{total} left", "Needs {count} {good}", "Too steep here", "Something is in
the way", "Something is already here", "No way in"), and the rule reasons
reuse `build.refused.<rule>`. Plain and short, as CLAUDE.md asks.

**11. Free building and planned building (extends `player-building` 7).**
- The world setting is `building: Planned | Free`, chosen when the world is
  made and saved with it.
  - **Planned** (the default for new worlds): the planner, presets, sites
    and costs. Build mode's palette is Blueprint, Build (the mallet's
    target, shown), Plan (O) and Take down; no single piece is placed
    outside a site, so a one-wall change is a plan edit of the house.
  - **Free**: today's build mode, single pieces and whole blueprints at no
    cost, plus the planner (a blueprint placed in a free world is filled
    whole at once, as B5).
- **Every world already made is Free**: the setting is missing from its
  save, and missing reads as Free, because that is how those worlds were
  built. Nothing in them changes.
- Recommendation taken (ask only with screenshots). M6's Site tab has a
  world-setting toggle that shows both, so the owner can try each; if the
  owner wants single pieces inside planned worlds after trying it, they are
  a plan edit away.

**12. Tunables.** `assets/config/building_planner.ron`, validated, units
named, defaults in one place:
| Tunable | Default | Unit | What it changes |
| --- | --- | --- | --- |
| `max_cells` | 24 | cells | the biggest plan |
| `max_radius` | 4 | cells from origin | fits one plot (`PLOT_M`) |
| `max_storeys` | 3 | storeys | the tallest plan |
| `rotation_step` | 1 | sixths | how far shift-wheel turns a ghost |
| `max_footing` | 2 | layers | the steepest ground a site sits on |
| `ghost_opacity` | 0.35 | 0..1 | a buildable ghost |
| `ghost_waiting_opacity` | 0.15 | 0..1 | a ghost waiting on others |
| `ghost_tint` | blueprint cyan | linear RGB | the ghost's colour |
| `ghost_reach_m` | 40 | m | how near the ghost and HUD show |
| `strokes` | wall 3, floor 2, roof 2 a cell, stair 4, post 1, lantern 1 | strokes | how long a piece takes |
| `stroke_s` | 0.5 | s | the mallet's beat (`held-swing`) |
| `reach_m` | 4 | m | how far F reaches a ghost |
| `scaffold` | true | | scaffolding for upper storeys |
| `scaffold_cost` | 2 planks a bay, 1 rope every 3 bays | | given back when it comes down |
| `removal` | mark | mark / return | dropped built pieces |
| `refund_share` | 1.0 | 0..1 | goods back from a piece taken down |
| `cost_scale` | 1.0 | x | every cost |
| preset sizes | small, standard, large | | each preset's three footprints |

## The mockup (M6 version 2)

`docs/mockups/building.html`, tabs `#planner`, `#site` and `#builder` (today's
build mode, "Free building"); the code of the new tabs is
`docs/mockups/building/planner.js`. It runs the plan model of decision 1,
the planner's checks of decision 2 (in M6's words), the presets of decision
4, the world checks of decision 5, building and scaffolding of decision 6,
the diff of decision 7 and the cost table of decision 8, with the tunables
of decision 12 as live sliders. Mockup-only, and not a claim about the
game: the stand-in furniture slot shapes (amber ghosts until `furniture` is
drawn), the plot and its neighbour, and the plan drawn flat in the planner
where the game will add the build camera's preview. Screenshots:
`docs/screenshots/building-planner/` (made with
`tools/mockup_building_planner_shots.js`, headless Chromium on SwiftShader in
a cloud session; no frame times taken).

**M6 version 3 (owner, 2026-11-24).** Two asks on the planner: "need
eraser here still" and "I also would like this to be able to toggle between
top down and isometric".
- **Erase has its own icon**, a rubber (`icon_eraser.png`, drawn by
  `tools/art/building_art.py`), not the pickaxe, which stays Free building's
  remove. It removes a door or window on an edge (the wall comes back), an
  inner wall, a stair, lantern or slot, and then the cell.
- **An outside edge can be clicked from outside.** Door, Window, Wall and
  Erase aimed at a room's outer edge from the cell beyond it act on the
  room's edge, where they used to say "Paint a room here first".
- **Top down or isometric**, a toggle beside the storeys. Isometric turns the
  plan 45 degrees and stands the walls up, the back outside walls full
  height and the near and inner walls cut low (a cutaway), so the floor, the
  doors and what is on it stay in sight and every tool still works on it.
  The choice is remembered on the device. In the game this is the build
  camera's preview (decision 12); the planner keeps both views.

## As built (2026-11-24): what the design left open

The owner approved M6 for the game on 2026-11-24 ("cool implement all the
phase 3 stuff!"). Building it needed these choices the decisions above leave
open. Each is a recommendation taken (ask only with screenshots).

- **The site record's kind is `building-site`.** `site` is already the kind
  of a city site (`pbd_core::sites::SITE_RECORD`), and two meanings of one
  kind would make a world's sites and its towns' sites read each other.
  `blueprint` is free and is used as designed.
- **The mallet is carried in the pack, not the tool slot.** The tool slot is
  four tools in a fixed saved line (`hand <tool> <four tiers>`); a fifth
  widens that line and every tool picker. A mallet is a good (`mallet`, one
  to a stack) and "in hand" for decision 6 means "in the pack": F on a
  ghost strikes when one is carried, else "Needs a mallet". Its recipe (1 log,
  1 stone at the workbench) is open from the start: in a planned world the
  town tree's house mission is built with it, so locking it behind that
  mission would leave the mission unreachable. The drafting table's recipe
  (6 planks, 2 rope) unlocks on the house mission as designed.
- **The drafting table is a block** (`Material::DraftingTable`, code 41),
  drawn in the workbench's wood with a sloped board on its top, set down like
  every station; F on it opens the planner. It queues no jobs.
- **A doorway's leaf comes with its wall.** A plan's door opening charts to
  the wall with its doorway and a door piece (`Shape::Door`) hung in it; the
  cost table's door row pays for both, and the leaf is placed on the wall's
  last stroke, in the same journal line. No door is taken from the pack.
- **Posts.** The planner has no post tool (the mockup has none); a plan keeps
  posts only when it is adopted from a house that has them.
- **The planner's stand-in patch** is a flat hex disc built for the check
  (`plan::flat_patch`): exact neighbours and corners, no pentagons, its
  ground at one layer, so build mode's rule functions run on it unchanged.
- **The game's rules decide, not the mockup's.** Where the mockup words a
  check its own way (a stair with no floor at its foot, an upper floor over
  nothing), the game reports the rule that breaks (`held_up`, `landing`...)
  with its `build.refused.*` reason. One consequence: a storey that leaves
  part of the one below open makes that part a flat roof, and two roofs a
  storey apart over neighbouring cells are refused (`roof_overlap`), as they
  are in build mode.
- **The world setting is the identity's `building` field.** Missing reads
  Free (every world made before); a world this build makes is written
  Planned. An older build ignores the field and plays the world as Free.
- **The plan is drawn as a picture.** The planner's plan view, top down and
  isometric, is painted on the CPU into one image the UI shows, redrawn only
  when the plan, the storey, the view or the hover changes; a click is
  mapped back through the same projection. The turnable 3D preview still
  waits for the build camera (decision 3).
- **Lime lines use the paving marks' bits.** A site's outline is marked as
  a loop of cells just inside its outer walls, each cell's band running to
  the cells before and after it on the loop, through the same `spare[2]`
  bits and texture as a road's marks (`townsfolk` 39a). So the lime runs
  through the outer cells' middles rather than exactly under the walls; the
  band and the stakes go when the site is finished.
- **What the first cut leaves out**, each said here so it is not
  mistaken for done: the scaffold has no ladder bay (an upper storey is
  reached by the house's own stair, which is buildable before the upper
  walls); "in the way" is a solid block or a standing tree, not yet a
  road cell or a town's footprint; a marked piece taken down with a full
  pack is refused ("No room") rather than its goods left in a
  crate; in a planned world build mode's palette is unchanged but every
  slot except Blueprint, Remove and Furniture says "Build from a plan".
- **Turning.** Charting a plan one sixth on turns it one sixth: a plan
  turned k sixths and charted k sixths on stands on the same cells
  (`Plan::turned`, pinned by a test at all six turns).
- **No held-swing yet.** A stroke is F on the mallet's beat (`stroke_s`),
  with the compass tick and the count; the first-person mallet swing waits
  for a mallet model in `held-swing`.

## Risks / Trade-offs

- **Two ways to build.** Free and planned worlds behave differently. Kept to
  a world setting, made once, so a world never mixes them; the planner is in
  both.
- **The planner is a big screen.** It is the largest UI the game will have.
  It reuses the stone-and-wood kit (M29) and build mode's pieces and icons;
  the preview waits for the build camera.
- **Charting at a slope.** A plan is flat; the ground is not. The footing
  (`player-building` 8) and `max_footing` decide how far a site may sit on a
  slope; beyond that it is refused, never bent.
- **Sites and towns.** A town growing yields to a site's cells as it yields
  to a player's edits (`world-persistence`): the site record names its cells
  as the player's from set out, so a town never builds into a half-built
  house.
- **Costs are new.** Every number in decision 8 is a first guess, in the
  mockup's sliders; the owner tunes them there.

## Migration Plan

- New record kinds only; nothing read today changes. Worlds without the
  `building` setting read as Free (decision 11). Free worlds' existing piece
  records become adoptable houses (decision 7) without being rewritten.
- A digest of the piece records of a free-built test world is taken before
  this lands and held after it: the planner never touches pieces it was not
  asked to.

## Open Questions

- None for the owner without screenshots. The Site tab's world-setting
  toggle and the removal toggle are there for the owner to try; decisions 7
  and 11 are recommendations taken (ask only with screenshots) until the
  owner tunes them in M6.
