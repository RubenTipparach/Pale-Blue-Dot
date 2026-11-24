# building-planner: the planner and sites in the game (task 3.7)

The `game-*` shots are the game's own, made by `tools/building_planner_shots.sh`
through the `PBD_PLAN_SHOT` capture instrument (`desktop/construction.rs`),
which opens the planner, sets a site out and strikes its pieces through the
same paths as play (the planner's state, the whole-site check, the pack's
goods, `Builds::place_with_hands`). The other files in this folder are M6's
mockup shots.

Fixed-step captures in a Claude Code cloud container: **lavapipe, no GPU**,
`--profile fast`, memory-only worlds, 1200 x 750, at the default spawn on
the desert. They show the look only: **frame time was not measured**. The
debug readout at the top left is the capture's developer overlay.

| Shot | What it shows |
| --- | --- |
| `game-planner-top.jpg` | The planner on the cottage, top down: presets and sizes, the player's blueprints and the town houses on the left; storeys, view, kit and roof over the plan; the tools under it; the checks, the bill of materials against the pack and Save as mine / Set out on the right. |
| `game-planner-iso.jpg` | The two-storey house in the isometric view, Floor 1 chosen: the back outer walls stand full height, the near and inner walls cut low, the storey below faint under it. |
| `game-planner-checks.jpg` | A cell painted on Floor 1 over nothing: it is red, the checks say "Floor 1: Nothing holds it up" (build mode's own reason), and Set out is dead. |
| `game-drafting-table.jpg` | The drafting table set down: the workbench's timber block with a drawing board tipped to the front and a linen sheet on it. |
| `game-site-before.jpg`, `game-site-set-out.jpg`, `game-site-before-after.jpg` | The ground ahead before and after the cottage is set out (BEFORE \| AFTER in one frame): the whole ghost in the blueprint tint, the lime line in the outer cells' tops and the survey stakes at the corners. |
| `game-site-half.jpg` | The cottage half built from the pack: the ground floor's walls stand in timber, the roof still a ghost; the HUD names the site, counts what is left and the goods still needed against the pack. |
| `game-site-upper.jpg` | The two-storey house struck to its upper floor: the ground storey's walls stand in fieldstone, the upper storey's walls and roof are ghosts, the HUD counts 17 of 51 left with the stone still short. |
| `game-site-change.jpg` | The cottage's plan changed after building: a cell dropped, its built pieces washed in chalk to take down. |
| `game-site-night.jpg` | The same at 22:00: the built walls as dark as the ground, the ghost unlit. |

What is wrong in them, and stays as a limitation for now:

- **No scaffolding went up.** With the ground storey's walls built and the
  upper storey's not, the site should have put scaffolding round the
  upper storey; the capture's log has no "scaffolding up" line, so it did
  not, and the cause is not found yet. The scaffold's geometry and cost
  are pinned by a core test; in the game it is unseen.
- **The ghost glows at night.** Design 5 leaves a ghost unlit by the field
  (a preview); at 22:00 its cyan is the brightest thing in the frame. A
  ghost dimmed with the hour is a look for the owner to judge.
- **A cactus stands inside the cottage.** The whole-site check refuses a
  standing tree on a block a piece takes, but the desert's cacti at this
  spawn are not seen by it (the capture's log reports no tree under any of
  the three ground cells), so the site was set out round one. Next: find
  which rule draws a cactus there that `grows_tree` does not read.
- **The lime line is faint.** It is the paving marks' band through the
  outer cells' middles, under the ghost; at this distance it hardly reads.
- The changed cottage marks its whole roof for taking down: a dropped cell
  changes the roof's cells, so the roof is a new piece and the old one is
  marked, as the diff says.
- The fieldstone walls of the two-storey house draw as grey stripes on
  lavapipe; their look is `player-building`'s cut, unchanged here.
