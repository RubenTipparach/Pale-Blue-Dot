# Farming: captures

For `openspec/changes/farming-and-smelting`. Captured 2026-11-14 on the M4 Max
Mac (its GPU, Metal), fixed-step `--capture` runs of the fast build, each on
a new memory-only world, at 2880x1800, saved as JPEG at quality 88. They show
what things look like, not how smoothly they run; frame cost was not
measured. Videos wait for the owner's batch (CLAUDE.md, 2026-09-27).

`--farm [metres]` is the change's capture instrument (task 5.1): it tills two
rows of four cells that far ahead through the same durable path a player's
`E` takes, sows wheat (front row) and turnip (back row) at staggered world
times so each row shows its four stages left to right (10%, 40%, 80% and
120% grown: sown, sprout, growing, ripe), tills one grassy cell in front of
the field so the seed it turns up floats over it, and stands a street lantern
by the field. `--farm-harvest` then harvests the two ripe cells and picks the
nearest wild plant. It also logs where the wild plants near the camera are.

Common flags: `target/fast/pbd-app --world <new name> --walk --spawn fields
--weather-at 1800`, then:

| file | flags | what it shows |
| --- | --- | --- |
| `field-day.jpg` | `--time 10 --yaw 270 --pitch -24 --farm 6` | The field at 10:00: wheat and turnip at every stage, tilled soil drawn with the fields sheet's tilled-soil tile, and a wheat seed floating over the grass cell just tilled. |
| `field-night.jpg` | `--time 22 --yaw 270 --pitch -24 --farm 6` | The same at 22:00, lit by the street lantern on the right: the crops near it are lit warm, the rest by the moon (lit at night). |
| `harvest-field.jpg` | `--time 10 --yaw 270 --pitch -24 --farm 6 --farm-harvest` | After the harvest: the ripe wheat and turnip are gone and their cells are bare tilled soil. The note under the crosshair is the last use's: the wild frost kale picked. |
| `harvest-pack.jpg` | the same with `--menu pack` | The pack after it: the kit's forge, coal and seeds (kit v3), the wheat and turnip harvested with their seeds back, and the wild kale's seeds and leaf. |
| `wild-plant.jpg` | `--time 10 --yaw 110 --pitch -12` | Wild plants in the meadow, derived from the world seed: a frost kale, a turnip and a wheat. No `--farm`. |
| `wild-plant-crop.jpg` | | Part of `wild-plant.jpg`, enlarged three times (nearest). |

Checked in the logs: every scripted till and sowing landed, and a normal
(warming) launch of the same build logged `READY` with no `LATE_PIPELINE`
line (`--world warm-a1 --walk --spawn fields`, 45 s). A capture does not
warm, so its logs list pipelines as late by design.
