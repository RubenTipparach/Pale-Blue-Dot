# The pack, wider: captures

For `openspec/changes/tool-crafting` decision 12 and task 5.5 (owner,
2026-11-16: "this menu should be wider", "I expect craft to be a menu on the
side not on the bottom"). Captured 2026-11-16 on the M4 Max Mac (its GPU,
Metal), fixed-step `--capture` runs of the fast build, each on a new world,
saved as JPEG at quality 88. They show what things look like, not how
smoothly they run; frame cost was not measured.

Sizes: `2880` is the default window (1440 x 900 points, 2880 x 1800 pixels,
UI scale 4); `1280` is `PBD_WINDOW=640x360` (1280 x 720 pixels, UI scale 1,
the smallest).

Common flags: `target/fast/pbd-app --world <new name> --walk --spawn fields
--weather-at 1800 --time 10 --pitch -10`, then:

| file | flags | what it shows |
| --- | --- | --- |
| `before-2880.jpg`, `before-1280.jpg` | `--craft pack` on `claude/env-v2` at `60aea32` | Today's pack: the Craft row under the hotbar, its plaque (LOCKED, the workbench being locked in a new world) hanging outside the rock's right edge. |
| `after-pack-empty-2880.jpg`, `-1280` | `--craft pack-empty` | The wide pack, nothing chosen: the slots and hotbar left, the hands' six recipes right (rope, workbench and stone axe locked: disabled plaques, silhouettes), the wells empty and CRAFT disabled. |
| `after-pack-pick-2880.jpg`, `-1280` | `--craft pack-pick` | Planks chosen: its row ringed with the rune, the log it takes lit (carried), four planks as the result, CRAFT live. |
| `after-pack-locked-2880.jpg`, `-1280` | `--craft pack-locked` | As `pack-pick`, with the pointer held over the stone axe's locked row: its tooltip, "Opens with: Gather by hand", stands over it. |
| `after-pack-2880.jpg` | `--craft pack` | The old instrument on the new layout: the workbench forced as the choice while locked, its plaque reading LOCKED, inside the rock. |

The before shots have no empty or hovered state: the old build has no
instrument for either, and its drop-down opened with planks chosen.

Several after runs logged a wgpu "timed out while waiting on the last
submission" panic on exit, after `CAPTURE_SAVED`: other sessions were
rendering on the same GPU at the time. The pictures were saved first.
