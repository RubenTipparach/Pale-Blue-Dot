# Tree impostor depth: captures

For `openspec/changes/tree-impostor-depth`. Captured 2026-11-16 on the M4
Max Mac (its GPU, Metal), in fixed-step `--capture` runs of the fast build at
2880 x 1800. Each shot started on a fresh world, so the clock and the weather
match. Before is `claude/env-v2` at `a256f35a`, run with its own `assets/`;
after is this branch. Each sheet shows the same frame side by side. The
stills show what the trees look like, not how smoothly they run.

Common flags: `--weather-at 1800 --world <fresh> --rain 0 --frames 240 --fly
--at 28.706 3.031` (the jungle hillside), then:

| file | flags | what it shows |
| --- | --- | --- |
| `owner.jpg` | `--height 40 --yaw 270 --pitch -15 --time 12` | The owner's view. Before, the canopy is one flat dark green. After, the crowns have lit tops and shaded hollows, and the hillside reads as separate trees. |
| `side.jpg` | `--height 40 --yaw 180 --pitch -15 --time 8` | Into a morning sun. Before, the hillside is one tone (luma std 5.8). After, the crowns have lit rims toward the sun and shaded sides (std 12.9). |
| `high.jpg` | `--height 120 --yaw 270 --pitch -30 --time 12` | Higher up, every tree a billboard: crowns rounded by light rather than flat discs. |
| `morning.jpg`, `evening.jpg` | `--time 7.5` / `--yaw 90 --time 17` | A low sun behind the camera: every crown faces the light, so there is less to add. The oak and birch crowns on the left catch the evening sun. |
| `night.jpg` | `--time 2` | The far band at night stays dim (`flora-v2` 13.4 holds): +2 luma. |
| `fields.jpg` | `--spawn fields --height 40 --yaw 300 --pitch -15 --time 10` | The temperate fields' sparse oaks and birches. |
| `handoff-noon.jpg` | `--height 30 --yaw 270 --pitch -28 --time 10`, rings forced in `scatter.ron` | The same view three times: every tree as its mid cards before, as its near model (the reference), and as its mid cards after. Before, the cards are 21% darker than the models and flat. After, they are within 5% and shaded like them. |
| `handoff-noon-far.jpg` | the same | The same for the far billboard: -12% before, +17% after (design 4 says why). |
| `handoff-morning.jpg`, `handoff-morning-far.jpg` | the same, `--time 7.5` | The hand-off in a low sun: within 5% before and after. |
