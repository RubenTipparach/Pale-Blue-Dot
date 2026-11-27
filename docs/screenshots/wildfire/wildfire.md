# wildfire 4.2 and 4.3 in the game

Headless `--capture` stills from a Claude Code cloud container with no GPU,
rendered on lavapipe (software Vulkan): they show what things look like,
never how fast they draw; no frame time was measured and `perf_suite.py`
was not run. Made by `tools/capture_wildfire.sh`; the scene is set by the
capture instrument `PBD_FIRE=<scene>` (nothing a player meets): the
memory-only world is made a Survival one, the weather the rule reads is
held at two dry weeks of late summer, and the instrument lights the driest
plant in view 5 to 20 m ahead (24 for a scar) with the player's own `wildfire::light`, then
runs the server's own ticks (`wildfire::run_once`). The fire shots stand on
a fields wood at `--at 32.9505 4.01833`; the torch shots on the survival
meadow at `--at -37.5 -84`.

| Shot | Scene | What it shows |
| --- | --- | --- |
| `front-night.jpg` | `front:25`, 02:30 | A fire at night, ahead between the oaks: the torch sheet's flames on the burning cell, and its light in the cell light field on the trunks, the tufts and the ground round it, under a dim moon. The plant it was lit on reads `plant_dry` 0.77 (an oak wood's floor in late summer), so it is a one- or two-cell fire, as decision 12 wants on marginal ground; the shot is stopped while it still burns. |
| `scar-next-day.jpg` | `scar:1`, 15:00 | A 20-cell fire's scar a world day later, about 20 m ahead beyond the birch: the burnt cells' tops ash grey in their own texels, no tufts or clutter on them, the green meadow round it untouched (the fire went out at greener ground on its own). |
| `scar-day-30.jpg` | `scar:30`, 15:00 | The same fire 30 days on: the grass and its tufts are back on every cell (whole by day 8) and the ash is gone. No tree burnt in this fire; a burnt tree would stand here as a sapling of its species, a quarter of the way to whole. |
| `torch-lit-rain.jpg` | none, 02:30, rain 1 | BEFORE: a placed torch (`--torch`, under the camera) lighting the wet ground round it in the rain. |
| `torch-out-rain.jpg` | `torch-out`, 02:30, rain 1 | AFTER: the same torch put out by the rain (the world's `lamp_out` record): the field round it is dark. The torch itself stands below the frame; it is still drawn, with no flame (test `wildfire::tests::an_out_torch_is_drawn_dark_and_lights_nothing`). |

Unsure of, for the owner:
- The fires in these shots are small: the wood's floor is shaded and near
  the generator's streams, so it sits near the 0.5 line. A larger front
  wants an open, dry field away from water, which this capture's spot is not.
- The ash is a flat warm grey with the tile's grain; the mockup's has more
  variation, and the shoots of day 2 are not drawn apart from it.
- Smoke at night reads faintly; its puffs are lit by the field, so they are
  dark away from the flames.
