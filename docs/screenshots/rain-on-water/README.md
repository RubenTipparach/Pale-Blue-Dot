# Rain splashes on the water (`water-flow` 6.3)

- **Renderer:** a Claude Code cloud container, no GPU: lavapipe (software
  Vulkan) under xvfb, `--profile fast`, `--capture` at frame 70, 1440 x 900,
  the lens off (`PBD_PASS_OFF=lens`). Looks only, never frame time.
- **Before:** `claude/phase-2-1` at `30e9c3a` with its own assets. **After:**
  this change.

| Files | Command | What it shows |
| --- | --- | --- |
| `sea-before/after.jpg`, `-crop-x2.png` | `--fly --at -34.6485 101.3415 --height 4.5 --yaw -15 --pitch -40 --time 11 --day 27 --rain 1` | A sea inlet in full rain (logged `cover 1.00, rain 1.00`). Before: rings only. After: small pale crowns on the sea out to the 20 m reach (6,387 pixels differ by more than 8 levels, all on the water). |
| `river-before/after.jpg` | `--fly --at 23.84345 3.87352 --height 4 --yaw 0 --pitch -40 --time 11 --rain 1` | A calm river 1 m deep, 3 m under the eye, in full rain. After: crowns over the river's rings. |

The sea "after" frame was taken before the draw was moved ahead of the fall
splash in the same pass; there is no fall in that view, so the order does
not change it. A night shot by lamps (`--time 22 --lamps 4`) at a fall's pool
showed the crowns warm under the lamps and dark away from them; it is not
kept because the fall's mist filled the frame.
