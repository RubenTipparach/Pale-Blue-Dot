# A hole in the sea on a beach (`ocean-environments` B22 follow-up)

The owner, 2026-11-17: "was #97 or 98 supposed to fix water holes where there
are caverns next to sea?", with a screenshot at `39.725 15.339`, 1.6 m up:
a square pit one cell wide and one or two layers deep under a one-layer sand
cliff. Its sand walls and floor are dry and lit, and shallow sea lies round it.

**Result: not reproduced on `main` (`db8f17e`, which holds #93's sea reach
through #98).** It was not reproduced on the build before #93 either
(`8fa25d6`, `claude/env-v2` just before the merge), at this spot, with the
pits tried here.

## How

- The Mac (M4 Max, Metal, the GPU), `--profile fast`, a fresh world for every
  run (memory-only save; `--world` in the worktree's own `saves/` for the
  reload).
- `--at 39.725 15.339`, `--time 17`, `--rain 0`, at low tide, the day's
  tide and high tide (`--tide low|high`).
- **Case A, a natural pit:** the spot has one. It is a shaft 4+ m deep, 3 to
  5 m east of the owner, beside a one-layer sand slab whose underside is air.
  `sea_cave_report` maps it as `c`, a cavern the sea reaches.
- **Case B, a dug pit:** made with the capture's own dig (`--dig N`, plus
  `--dig-ahead` where it says so), which runs a player's edit path
  (`apply_edit`).
  - Beside the water: one layer, two cells.
  - Beside the natural pit: one layer.
  - Two layers deep in one cell 1 m from the owner, not joined to any water
    (`--fly --height 1.6 --dig-ahead --pitch -50`).
  - Two layers deep straight down under the owner (`--fly --height 2.5
    --dig 2`), then the save reloaded (a second run on the same `--world`).

## The stills

Before (`8fa25d6`) is the top row; main is the bottom row.

| File | What |
| --- | --- |
| `owner-spot-main-four-ways.jpg` | main, the owner's spot and height, four headings |
| `natural-pit-before-main-tides.jpg` | case A, low / the day's / high tide |
| `natural-pit-from-above-before-main.jpg` | case A from 2.5 m, the day's tide and high tide |
| `dug-beside-water-before-main-tides.jpg` | case B, dug beside the water, three tides |
| `dug-beside-pit-before-main-tides.jpg` | case B, dug beside the natural pit, three tides |
| `dug-2-deep-before-main-tides.jpg` | case B, two deep, 1 m off, three tides |
| `dug-2-deep-inland-before-main.jpg` | case B, two deep under the owner, the day's tide and high tide |
| `dug-2-deep-inland-reloaded-before-main.jpg` | the same pit after reloading the save |

The tides look as they should in every one of them:

- at low tide the pit is dry, and so is the beach round it;
- at the day's tide the pit holds the sea's surface;
- at high tide the sheet lies over the pit and the beach alike.

None shows a dry pit with water round it.

## The measurement

`sea_hole_dig_report` (`crates/pbd-app/src/planet_water.rs`) is an ignored
test, a measurement instrument. It builds the fine set at a place as the game
does, then takes every beach cell within `PBD_SPAN` metres: ground from 3 m
under the sea's top layer to 1 m over it.

On each cell it plays five edit sequences through `FineSet::write_layer`,
the relight and `mark_sea`, the path a player's edit takes:

- dig 1;
- dig 2;
- place dirt and take it back;
- dig, fill with dirt, dig 2;
- put dirt in the water over the cell, take it back and dig the ground.

It then rebuilds the set from the save, as a reload does. A cell counts as a
hole when all three of these hold:

- its ground is under the high water;
- no sheet is listed over it (`sheet_listed`);
- no water stands at the sea's top layer in its column.

At `39.725 15.339` with `PBD_SPAN=12` it played 380 sequences on 76 cells and
found **0 holes**, live or after the reload.

```
PBD_AT='39.725 15.339' PBD_SPAN=12 cargo test --profile fast -p pbd-app --lib sea_hole_dig_report -- --ignored --nocapture
```

## What is left

- The owner's build is not known. #93 landed on `claude/env-v2` at 04:30 and
  reached `main` through #98 at 04:45 (-0600) on 2026-11-17, so a build from
  before then does not have the reach.
- The pre-#93 build did not make the hole with these pits either. So the
  owner's hole came either from a build older than `8fa25d6`, or from
  something not tried here. Candidates: a pit somewhere other than the
  logged position, a different hour's tide, or an edit sequence not in the
  five above.
- A save from that world, or the HUD line of the screenshot (its tide and
  build), would settle it.
