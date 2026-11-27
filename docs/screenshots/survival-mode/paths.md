# Survival's paths and the path picker (`survival-mode` 6.5)

Captured 2026-11-27 in a Claude Code cloud container with no GPU, on
lavapipe (software rendering) under `xvfb-run`, 1440 x 900, from the `fast`
build on a new memory-only world. They show what it looks like, never how
smoothly it runs; frame cost was not measured.

Common flags: `--walk --spawn fields --pitch -8 --rain 0 --weather-at 7200
--mission-mode survival --missions 21`. `--mission-mode survival` (a capture
instrument) gives the world Survival's tree until worlds carry their mode;
`--missions 21` raises the script's steps from the first dig to the campfire
set down, as the game's systems raise them, so "Make a fire" completes and
the picker opens on its own.

| file | flags | what it shows |
| --- | --- | --- |
| `picker-day.jpg` | `--time 12 --frames 300` | The fire just lit: the Missions screen opened by itself with CHOOSE A PATH over the tree, "The fire is lit.", a card a path (icon, name, line, first mission, count), Later. The card under the screen-centred pointer shows the carved-oak hover. |
| `picker-night.jpg` | `--time 22 --frames 300` | The same at night. The picker sits on the Missions screen's own night-blue backing, so the hour does not change it. |
| `path-hud.jpg` | `--time 12 --pick-path eat-drink --frames 300` | Eat and drink picked (`--pick-path`, as a click on its card): back in play, the HUD's tracker has the path's name over its Now, "Drink water", and its objective. |

In this build the Hunting card's count leaves out the missions still
pending on goods and game not yet in the build (the file's `awaited`).
