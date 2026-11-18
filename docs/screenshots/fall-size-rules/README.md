# A fall's size (`water-flow` 6.7)

Before (`origin/main` at 2de3c03a with its own `assets/`) and after (this
branch), the game on the Mac (M4 Max, Metal), `--profile fast`, `--capture`
(fixed step: looks, not frame time), `PBD_PASS_OFF=lens`, `--time 13 --rain 0`.
`*-sheet.jpg`: before left, after right, HUD cropped.

| Shot | Command | What it shows |
| --- | --- | --- |
| `river-cascade` | `--walk --at 23.2255 -3.012 --pitch -20` | The owner's cascade of one-layer steps: white at every lip before; after, sub-metre steps have no mist, one faint puff at a step of a metre or more |
| `one-layer-pit` | `--walk --yaw 90 --pitch -25 --pour 1 --pour-ahead 12 --pit 1 --water-ticks 60` | Steps of `D` 0.1 to 0.9 m over a 1.00 m pool: puffs and rings gone |
| `tall-fall` | as above with `--pit 9`, `PBD_EYE_PATH` keys `0`/`150` at the spawn, `151 24.07649 -3.00759 60.09 121.6 -9` | `D` 9.8 m: the mist as approved, unchanged |
| `deep-cascade` | `--fly --at 11.33992 4.76730`, eye `0 11.33992 4.76730 14.0 -122.0 -35` | Pools 3 to 5 m deep under 7 to 9 m falls: the rings unchanged |
