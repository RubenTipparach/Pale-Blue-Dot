# Glowing mushrooms in the game: captures

For `openspec/changes/glowing-mushrooms` (tasks 1.2, 1.7, 2.3, 2.4, 2.5).
Taken 2026-11-19 and 20 in a Claude Code cloud container: headless
`--capture` runs of the fast build (`--profile fast`) on lavapipe (software
Vulkan, no GPU) under `xvfb-run -a -s "-screen 0 1440x900x24"` with
`PBD_WINDOW=1440x900`, each on a memory-only world. They show what things
look like, not how smoothly they run: **the frame cost was not measured**
(no GPU here). The `night-carry` A/B of design 13 (`PBD_CARRIED=0` against
the build, on the RTX 3070 and the M4 Max) is the owner's. The debug readout
(the top 120 px) is cropped off every shot but the zooms.

Common flags: `target/fast/pbd-app --walk --time 23 --weather-at 1800
--at 32.9505 4.01833`: a fields meadow edge at 23:00 (dusk lamps lit), 2 m
from a glowing mushroom the pick instrument found.

| file | extra flags | what it shows |
| --- | --- | --- |
| `pick-before.jpg` | `--pick-glow 9999` | The glowing mushroom at night under the crosshair, and the prompt `F  PICK GLOWING MUSHROOM`. The pick script walked and turned onto it as a player would and never pressed F. |
| `pick-after.jpg` | `--pick-glow 30` | The same pick played with F: the plant is gone from its cell (no other flower in its place), the item toast `GLOWING MUSHROOM X1`, the mushroom in the hotbar's selected slot and in hand. The log reads `picked the glowing mushroom at cell 1327010864 on day 0.96` and `GLOW_PICKS 1 glowing mushrooms brought round to the picks`. The ground in front is warmer than before: the held mushroom's carried light (level 3, warm; survey G5 and A4 open, the recommendations built). |
| `pick-zoom.jpg` | | The plant and the prompt in `pick-before`, enlarged twice. |
| `torch-today.jpg`, `torch-now.jpg`, `torch-today-now.jpg` | `--yaw 90 --pitch -20 --hold block:torch`; today with `PBD_CARRIED=0` | A torch held at night. Today (top of the pair) no light leaves the hand: the meadow is lit only by the night. Now (bottom) the torch lights the grass, the tufts and the bank around the player from its exact place, falling off as a placed torch's does, with no shadow. **Today's frame is this build with the carried light switched off**, so the new held torch is drawn in both; today's game drew nothing in hand for a torch. |
| `torch-close.jpg` | | The held torch in first person, enlarged twice from `torch-now`: the bark haft in the hand, the wrapped and charred head, the three-card pixel flame with an ember over it, and the soft glow. |
| `glow.jpg` | `--yaw 90 --pitch -20 --eat glowing-mushroom` | The body glow: a glowing mushroom eaten with the right button (the note `ATE THE GLOWING MUSHROOM: GLOW FOR 3 MIN`, and the effects row over the hotbar shows the mushroom and `GLOW 3:00`). The ground round the player takes a faint cool light from the chest, merged with the held one; on lavapipe at this night exposure it reads weakly, a look for the owner on a GPU. One mushroom of the two the instrument deals is left in hand, so its own warm level-3 light is merged in. |

## What the shots show to look at again on a GPU

- The held flame reads pinkish-white at its heart on lavapipe at night. It is
  drawn at 0.75 of its texture, as the placed flame is kept under the
  tonemapper's shoulder; the night's exposure on this renderer lifts it
  further. Worth the owner's look on the desktop beside M13 v3's
  `torch-v3-night-fp.jpg`.
- Not captured here (each run takes about 20 minutes on lavapipe, and two
  container restarts cut the batch): the lantern in hand and the torch in the
  left hand beside a tool. Both are built (`held/light.rs`); `--hold
  "block:hanging lantern"` and `--hold block:torch --tool pickaxe` take them.
