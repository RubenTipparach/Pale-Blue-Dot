# The lamp that vanished (owner, 2026-11-16)

"I placed a lamp but it somehow despawned." The write-up is in
`openspec/changes/edit-pipeline/design.md`, "The owner met it as a lamp that
vanished". Fast build (`--profile fast`), this Mac (M4 Max, GPU), night
(`--time 0`), in a world on disk (`--world lv4`) so the reload is real.

| File | Run | What it shows |
| --- | --- | --- |
| `1-placed-night.jpg` | `--world lv4 --walk --time 0 --lamps 9 --pitch -10` | Every light placed in a row 9 m ahead, at night. |
| `2-reload-after-walk-rain-before-fix.jpg` | then `--world lv4 --walk-distance 200 --time 0 --rain 1` (fine sets land along the walk; no `REMOVED` line), the pose put back at the spawn in `world.ron`, then `--world lv4 --walk --time 0 --rain 1` | Back after the walk and a reload, in rain, before the fix: every light is in the save and lit (the lamp bake counts 7), but a cactus has grown back in the cell the walk dug in front of the hanging lantern, and hides it. The rebuilt tier drew that edited cell's cap as generated. |
| `2-reload-after-walk-rain-after-fix.jpg` | the same reload, after the fix | The hanging lantern is back in view; the dug cell shows its own top, as it did when it was dug. |

Not captured: a flight and landing, which no capture rig scripts yet; the app
test `planet::lod::landing_replay` stands for the set swap a landing makes.
Frame cost was not measured (no `perf_suite.py` sitting): the replay runs only
on a landing after an edit made while the set was building, and the check at
each landing is one pass over the finest records.
