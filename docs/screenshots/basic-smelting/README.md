# Basic smelting: captures

For `openspec/changes/farming-and-smelting`. Captured 2026-11-14 on the M4 Max
Mac (its GPU, Metal), fixed-step `--capture` runs of the fast build, each on
a new memory-only world, at 2880x1800, JPEG quality 88. Frame cost was not
measured.

`--forge [metres]` (task 5.1) places a forge that far ahead through the edit
path, deals 12 mixed ore and 8 coal, and queues two `mixed-ore` jobs through
the panel's own `queue`: the first three world seconds in (of eight), the
second waiting behind it. The clock is pinned in a capture, so the first job
stays three-eighths done. `--forge-panel` opens its panel.

Common flags: `target/fast/pbd-app --world <new name> --walk --spawn fields
--weather-at 1800 --yaw 270 --pitch -18 --forge 4`, then:

| file | flags | what it shows |
| --- | --- | --- |
| `forge-day.jpg` | `--time 10` | The forge: a block of the mountains sheet's slate bricks, its iron chimney, and the flame in its mouth while a job runs. |
| `forge-panel.jpg` | `--time 10 --forge-panel` | The panel over the running world: the pack's rock and the oak lintel, a plaque a recipe with its inputs and outputs (4 mixed ore + 2 coal into 2 copper ingots, an iron ingot and a tin ore; copper, tin and coal into 2 bronze), BRONZE greyed since the slots hold no copper yet, the job being worked with one waiting, and the bar three-eighths full. |
| `forge-night.jpg` | `--time 22` | The forge at night. The flame draws itself; the forge is not yet a light (design decision 8). |
