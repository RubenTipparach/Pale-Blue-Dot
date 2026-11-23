# Proposal: building-planner

## Why

**The owner, 2026-11-23:** "Make presets for houses and buildings too. I
want the player to be able to design structures and build according to blue
prints. That way we avoid the jankiness of free building. Buildings can be
modified mid builds in the building planner too. Please mock this up so I
can fine tune it. Add this requirement to.phase 3".

Build mode today (`player-building`, built on `claude/phase-2-1`) places one
piece at a time: a wall on an edge, a floor in a cell, a stair, a post, a
lantern, a fitted roof. A blueprint (decision 5) is a town template's
building, charted onto the cells at the aim and filled whole in one click,
because only free building exists (decision 7). Two things follow:

- **A house is put together by aiming.** Every wall is aimed at its edge on
  foot, in the world, at the storey under the crosshair. It works, and it is
  the jankiness the owner names: a wrong edge, a missed storey, a roof that
  will not fit because one wall is one edge off, found only once it is up.
- **A blueprint is all or nothing.** It is the towns' buildings only, it
  cannot be changed, and it costs nothing because nothing costs anything.

Phase 3 is the townsfolk and their towns: carpentry, house building, roads,
carts. A player who builds beside a town should design the house first, at a
table, the way the town's own houses are designed (as templates), and then
build that design.

## What Changes

- **The building planner.** A screen, opened at a drafting table (a new
  station, made at the workbench) or from build mode, where the player
  designs a structure on the hex grid: storeys, rooms painted cell by cell,
  walls with doors and windows, inner walls, stairs, lanterns, the roof, and
  furniture slots. It runs the same rule functions as build mode
  (`pbd_core::building::rules`) on the plan, and a broken rule is shown red
  with the same plain reason (`build.refused.<rule>`).
- **Presets.** A library of house and building presets fitting the towns:
  straw hut, cottage, two-storey house, townhouse, workshop, shop, barn,
  stable, warehouse and tower, each in a small, standard and large size, plus
  every plain building of the settlement templates. Any preset can be picked,
  edited and saved as the player's own blueprint.
- **Placing a blueprint.** A blueprint goes into the world as a construction
  site: set out with survey stakes at its corners and lime lines along its
  walls (the paving marks' look, `townsfolk` 39a), every piece a ghost,
  turned in sixths and checked whole before it is set out.
- **Building it.** The player walks up with the mallet and the materials in
  the pack and puts the pieces in, one stroke-built piece at a time, each
  becoming an ordinary `piece` record. The site shows what is left and its
  bill of materials. Upper storeys go up from scaffolding, set up and taken
  down by the site itself.
- **Editing mid-build.** A site opens in the planner. When the changed plan
  is saved, pieces still to come update, unbuilt pieces the plan drops go
  away, and built pieces it drops are marked for removal and come back as
  materials when taken down.
- **Saving.** Blueprints and sites are stored records on the durable path
  (`blueprint`, `site`, schema 1). A built piece is the piece record build
  mode already writes. Old saves load unchanged.
- **Free building becomes a world setting beside planned building.** New
  worlds build by plan, from the pack; free building (today's single pieces
  and whole blueprints at no cost) stays as a world setting, and every world
  already made opens as it was made: free. Recommendation taken (ask only
  with screenshots); the M6 Site tab shows both for the owner's tuning.
- **Costs.** Planned building needs what each piece is made of: a table of
  goods per piece class and kit (`building_costs.ron`), the first costs in
  building (decision 7 said only free exists "until the materials change
  lands": this is that change for planned worlds).
- **The mockup.** M6 Hex House Builder (`docs/mockups/building.html`) gains
  a Planner tab and a Site tab beside today's builder (now the "Free
  building" tab), with live tunables for what the owner will fine-tune.
- **Phase 3.** CLAUDE.md's roadmap and the tracker's Roadmap list it with the
  townsfolk.

## Capabilities

### New Capabilities

- `player/building-planner`: the planner, presets and the player's
  blueprints, construction sites, building a site from the pack, editing a
  site mid-build, and the world's building setting.

### Modified Capabilities

- None in `openspec/specs`. `player-building` is itself a change; its
  decisions 5 and 7 are extended here (design 11), not rewritten.

## Impact

- `pbd-core`: a `building::plan` module (plan coordinates, charting a plan
  onto cells, the planner's checks through the existing rule functions, the
  site diff), costs, and the `blueprint` and `site` records.
- `pbd-app`: the planner screen, the drafting table station, the site's
  ghost, stakes, lime lines and scaffolding, the mallet, and the HUD's
  remaining count and bill of materials.
- Data: `assets/config/blueprints.ron` (the presets), `building_costs.ron`,
  `building_planner.ron` (tunables), locale keys under `planner.*`,
  `site.*` and `blueprint.*`.
- Saves: two new record kinds; nothing existing changes shape.
- Nothing is built by this change until the owner approves M6's Planner and
  Site tabs and asks.
