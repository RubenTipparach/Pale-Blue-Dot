# wildfire 4.2 to 4.4 in the game

Headless `--capture` stills from a Claude Code cloud container with no GPU,
rendered on lavapipe (software Vulkan): they show what things look like,
never how fast they draw; no frame time was measured and `perf_suite.py`
was not run. Made by `tools/capture_wildfire.sh`, with no debug readout
(`--no-readout`).

The scene is set by the capture instrument `PBD_FIRE=<scene>` (nothing a
player meets): the memory-only world is made a Survival one; the air the
rule reads is held at two dry weeks of late summer with a 3 m/s west wind,
and the drawn sky is held dry to match; the instrument lights the driest
plant (or tree) in view 5 to 14 m ahead with the player's own
`wildfire::light`, then runs the server's own ticks (`wildfire::run_once`)
to the tick the fire burns widest, found by a dry run of the rule, or, for
a scar, until it is out, back-dated by its days. The fire shots stand on a
dry mountain meadow of grass and a few pines at `--at -37.5 -107.5`, found
by the ignored test `wildfire::tests::find_dry_grassland` (291 of its 421
cells near read `plant_dry` 0.7 or more in the dry season); the torch shots
on the survival meadow at `--at -37.5 -84`. `--time` is the world clock's
hour, not local: at longitude -107.5 the clock's 02:30 is dusk-dark and
19:12 is about local noon.

| Shot | Scene | What it shows |
| --- | --- | --- |
| `front-night.jpg` | `front:90`, night | A grass fire at its widest (31 cells alight, 44 burnt, 70 s in): a line of flame running along the slope ahead and a near cell burning a few metres off, each lighting the grass, the terraces and the pines round it through the cell light field, embers rising. The red edge and the health bar are the harm (`Cause::Fire`): the player stands beside a burning cell. |
| `tree-burning-night.jpg` | `tree:25`, night | A pine lit at its foot (the small pine on the right), and the fire it started in the dry grass round it, 28 cells alight 25 s in: flames on the torch's sheet, embers, the meadow lit orange. |
| `tree-snag-next-day.jpg` | `tree-snag:1`, local noon | The same kind of fire's scar a world day after it went out (165 cells), from a few metres off: the burnt ground ash grey with no tufts or clutter, and the pine standing charred where it burnt (centre: a ground fire's look, its own model with the needles browned and the foot black). The axe takes it for charcoal. |
| `tree-snag-day-30.jpg` | `tree-snag:30`, local noon | The same view 30 world days after: the grass and its tufts are back on every burnt cell (whole by day 8), the ash gone, and the burnt pine a sapling of its own species (right), a quarter of the way to whole; it stands full-grown again at day 60. |
| `torch-lit-rain.jpg` | none, night, rain 1 | BEFORE: a row of every lamp 3 m ahead (`--lamps`), the placed torch among them burning (its light the orange on the tree behind), and the torch in hand alight, in the rain. |
| `torch-out-rain.jpg` | `torch-out`, night, rain 1 | AFTER: the rain has put out the torch in hand (drawn, its head with no flame) and the placed torch (the world's `lamp_out` record; it stands just out of frame, and the orange its light threw on the tree behind is gone); the lanterns beside it, which the rain does not put out, still burn. |

Unsure of, for the owner:
- Smoke reads faintly at night: its puffs are lit by the field and glow a
  little over the flames, and go dark against the night sky above them.
- The ash is a flat warm grey with the tile's grain; the shoots of day 2
  are not drawn apart from it.
- A ground fire leaves a tree standing scorched (whole model, foot black,
  leaves browned); only a very dry conifer in 6 m/s of wind or more burns
  in its crown and leaves the bare snag, which no shot here shows.
