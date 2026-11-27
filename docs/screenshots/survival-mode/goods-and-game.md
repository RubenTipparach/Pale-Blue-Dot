# Survival's goods, weapons and game in the game (tasks 4.10, 4.11)

Headless `--capture` shots in a Claude Code cloud container, rendered on
lavapipe (software Vulkan, no GPU) at 1440 x 900 under Xvfb: they show what
things look like, never how fast they draw. Every shot is a memory-only world
at `--at -37.5 -84`; the commands are `tools/capture_survival_goods.sh`.

`--hold <good>`, `--lay-weapons <m>`, `--survival-kit` and `--husbandry
game-stand` are capture instruments (nothing a player meets): the held good in
the selected slot, a row of every weapon laid as a drop, every survival good in
the pack, and one of each game species held still broadside.

| Shot | What it shows |
| --- | --- |
| `held-spear-day.jpg` | The stone spear in hand at 16:00, leant forward at the ready (`held.ron` `weapon.spear_up`): the mockup's lathed shaft with its fibre binding painted near the top, the knapped head. |
| `held-sword-day.jpg` | The bronze sword in hand, its flat to the eye: the extruded blade with its fuller painted, the curved guard, the leather grip. |
| `held-axe-day.jpg` | The iron hunting axe in hand: the swept handle with its wrap painted down one edge, the bearded head extruded with a rounded bevel. |
| `held-bow-night.jpg` | The bow in hand at 03:48 beside a row of the game's lamps: unlit by the sky, lit by the lamps through the field, as the tools are. |
| `weapons-lying-day.jpg`, `weapons-lying-night.jpg` | Every weapon laid as a drop (as a thrown spear lands): spears, bow, the three arrows, the hunting axes and the swords on their flat sides, lit like the ground (`LitByField`, `LitLikeTerrain`), by day and by lamplight at 03:48. |
| `game-day.jpg`, `game-night.jpg` | The game, left to right: the hare, the red fox, the ridge goat, the rootling and the dust hound, in the herd's approved blocky family (M26), each in its own hide strip, by day and lit by lamps at 03:48. |
| `pack-icons.jpg` | The pack holding every survival good and the two stations, their icons the M33 Survival tab's drawings with the inventory icon's 1 px outline. |

Look choices to judge, recorded in `openspec/changes/survival-mode/tasks.md`:

- The game is drawn blocky, as the herd is (`animal-husbandry` design 17, the
  owner: "please go back to blocky style for animals"), not smooth-lofted.
- A weapon lying in the world is its model, not its icon card (CLAUDE.md says a
  dropped item is an icon). A thrown spear landing as a floating card read
  wrong; taking the weapons out of `DropLooks::models` puts the cards back.
- Held weapons rest in the hand; their swings wait on their mockup (task 1.6).
