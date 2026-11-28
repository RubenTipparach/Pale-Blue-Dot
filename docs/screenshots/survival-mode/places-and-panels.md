# Survival's places and panels in the game (tasks 4.2, 4.6, 4.8, 6.3, 6.6)

Fixed-step `--capture` stills from a Claude Code cloud container with no GPU:
software rendering (lavapipe) at 1440 x 900 under Xvfb (the town's at
1200 x 750). They show the look, never the frame rate. Retaken 2026-11-27 (round 2) with `--no-readout`: the stats, both repairs,
the still and the map. Each scene's state is
set by the capture instrument `PBD_SURVIVAL_STATE=<state>`
(`desktop::survival_play::instrument`: the capture's world made a Survival
one):

```
PBD_SURVIVAL_STATE=<state> xvfb-run -a -s "-screen 0 1280x720x24" \
  target/fast/pbd-app --walk --rain 0 --no-readout <args> --capture <png>
```

| File | State | Args | Shows |
| --- | --- | --- | --- |
| `pack-stats-day.jpg` | `thirsty25` | `--frames 300 --time 12 --survival-kit --menu pack` | The pack's STATS panel left of the pack (G7): health 88, stamina 100, food 42, water 25 as bars with their numbers, the thermometer at 37.0 C, the two worn slots (nothing worn) |
| `repair-bench.jpg` | `worn` | `--frames 300 --time 12 --tool pickaxe --craft bench 4` | The workbench open with a worn stone pickaxe in hand: the REPAIR panel beside the workbench's, the pickaxe with its red wear bar and the price, 2 stone (half the head of its recipe) |
| `repair-town.jpg` | `worn` | `PBD_KEY_SCRIPT=120:KeyF`, `--frames 170 --tool pickaxe --at 29.208122 8.793697 --yaw 87.5 --time 11` | F on Linenleigh's merchant: the talk panel's REPAIR between TRADE and BYE, 9 coins, greyed (a new world's purse is 0) |
| `still-fire-night.jpg` | `still` | `--frames 600 --time 22 --pitch -18 --place-row campfire,still 4` | A still (right, the stone pot and its cane spout) and a lit campfire (left) at night, both placed on the ledges either side, the still lit by the fire; a sea bucket's four sips poured in. A sip boils a minute of play, longer than the capture's ten seconds, so no fresh water has come yet (`the_still_makes_fresh_water_only_beside_a_lit_fire` holds the rule) |
| `still-water-dusk.jpg`, `still-water-cup.jpg` | `still` | `--frames 600 --time 19.3 --pitch -38 --place-row still 2.8` | Round 2: a still on a stone block at dusk, its cup (top right) with three of its six sips of fresh water standing in it (the instrument boils three minutes); the second file is the cup enlarged four times, nearest. The view is pitched too low to show the whole pot. |
| `bedroll-day.jpg`, `bedroll-night.jpg` | `bedroll` | `--frames 300 --time 12 --pitch -40` (night: `--time 22 --lamps 5 2`, a torch in hand) | The bedroll laid a step ahead: one shaped piece, its wool and hide texture, its long edges rolled into the ground and its head end rolled, lit as the ground is by day and by the torch at night |
| `bag-day.jpg` | `bag` | `--frames 300 --time 12 --pitch -30 --survival-kit` (dropped 1.6 m ahead) | The dropped pack standing as the pack's own icon on the ground, turned to the eye |
| `bag-map.jpg` | `bag` | `--frames 300 --time 12 --survival-kit --menu map --map-mpp 0.2` | The map (zoomed in as far as it goes, 0.7 m a pixel): the bag's marker, the pack icon, drawn over the player's arrow and standing on its point, 2.6 m away |

What is wrong in them, honestly:

- The stats panel's X (every panel's lintel carries one) sits in the gap
  between it and the pack.
- The break's crack (`tool_break`) is judged by measurement only (Claude
  cannot listen); only the owner's ears approve it.
- `bedroll-*.jpg` and `bag-day.jpg` were retaken on 2026-11-27 without the
  readout (`tools/capture_survival_hud.sh`); at night the bedroll is lit by
  the capture's lamps, the torch in the hotbar but not in hand.
