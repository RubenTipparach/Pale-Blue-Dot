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
| `game-site-half.jpg` | The cottage half built from the pack (retaken 2026-11-24): the ground floor's walls stand in timber, the roof still a ghost; the HUD names the site, counts what is left and the goods still needed. The cactus that stood inside it in the first take now refuses the site there, so the capture sets it out on the nearest clear cell; the cactus stands outside the wall. |
| `game-site-scaffold.jpg` | The two-storey house struck lowest first to 34 pieces: the ground storey's fieldstone walls all stand, and scaffolding rings the upper storey (timber poles, plank walks, rope at the crossings, lit like the ground), its walls and roof still ghosts. Replaces `game-site-upper.jpg`, which had none. Retaken 2026-11-24 with the ladder bay (design, "The scaffold's ladder bay"): its treads rise off the first bay on the left, half behind the cactus; from this height it reads poorly, and a closer shot is still owed. |
| `game-site-change.jpg` | The cottage's plan changed after building: a cell dropped, its built pieces washed in chalk to take down. |
| `game-planner-change.jpg` | The planner open over the half-built cottage with a cell taken out, before Save change: "Changing Cottage", then the change: 2 pieces added, 7 pieces removed, 31 planks back. |
| `game-site-night.jpg` | The two-storey house at 22:00 with its scaffolding: the built walls and the scaffold as dark as the ground; the ghost glowing at `ghost_night` 0.12 (survey P3, 2026-11-24: "The night is too faint, please make it glow a bit more at night"; it was 0.05). |
| `game-site-night-0.2.jpg` | The same at `ghost_night` 0.2, the first try for P3: the ghost the brightest thing in the frame. Kept beside the chosen 0.12. |

What was wrong in the first take, and what was done (design, "The
captures' faults, found and fixed"):

- **No scaffolding went up**: the rule wanted every ground wall built
  first, and on this uneven ground the capture's pass-by-pass striking put
  the upper floor and walls up before seven ground walls. Fixed in the
  rule; the capture now also strikes lowest first.
- **A cactus stood inside the cottage**: the check read only trees that
  can be chopped. Fixed: a cactus or an unpicked wild plant in a piece's
  cell refuses the site, with "Something is in the way".
- **The ghost glowed at night**: now dimmed with the sun. It is still a
  faint blueprint cyan at night, not lit by the field (design 5).
- **The lime line was faint**: laid stronger in the terrain shader. It is
  still under the ghost floor, so it reads best once the floor is built.

Still as before:

- The changed cottage marks its whole roof for taking down: a dropped cell
  changes the roof's cells, so the roof is a new piece and the old one is
  marked, as the diff says.
- The fieldstone walls of the two-storey house draw as grey stripes on
  lavapipe; their look is `player-building`'s cut, unchanged here.
