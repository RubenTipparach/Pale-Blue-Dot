# path-stairs: where a town's path takes a step, there is a stair

The owner (2026-11-14): "where theres a step on a path there should be
staris". The write-up is `openspec/changes/path-stairs/`.

Each pair is the same shot before (top, branch base `d43d018`) and after
(bottom). In-game fixed-step captures (`--capture --frames 200`) on the M4
Max Mac (Metal), the fast build, a new world for every shot, on foot. They
show what things look like, not how smoothly they run. `--time` is the clock
at longitude 0: Holinghaven (115 W) is day at 22 and night at 11, Ashenstead
(59 E) day at 8 and night at 18.5, Marenstrand (172 E) day at 0. Day shots
take `--rain 0 --weather-at 7200`, night shots `--weather-at 3600`; the
weather still differs between some pairs (rain in one, none in the other),
which is the weather's, not this change's.

| File | Where | Shows |
| --- | --- | --- |
| `pair-holinghaven-up-street-day.jpg` | Holinghaven's up street from the lane below its first steps, `--walk --at 23.41860 -115.06458 --yaw 103.4 --pitch -6 --time 22` | The owner's shot: the up street climbing a terrace in bare one-metre earth risers. After, the mockup's street steps in its two step cells, stone, six risers each; the next terrace's steps beyond. |
| `pair-holinghaven-up-street-night.jpg` | the same, `--time 11` | The same at night under the street lamps. |
| `pair-holinghaven-up-street-side-day.jpg` | beside the up street, `--at 23.39402 -114.97943 --yaw 49.4 --pitch -8 --time 22` | The up street's side cell (22, 20) onto the middle lane a terrace up: a straight flight against the riser (in front), and the street steps beside it. |
| `pair-ashenstead-gate-outside-day.jpg` | Ashenstead's north gate (G1), `--at 19.91775 59.50173 --yaw 62.5 --pitch -4 --time 8` | Through the gate: before, the metre of earth across the street just inside; after, the street steps in (19, 7) and (20, 7), and on the right the flight from the passage's corner onto the wall lane. |
| `pair-ashenstead-gate-passage-day.jpg` | in the passage, `--at 19.86646 59.47330 --yaw 51.7 --pitch -10 --time 8` | Standing at the foot of the steps, looking up them into the town. |
| `pair-ashenstead-gate-passage-night.jpg` | the same, `--time 18.5` | The same at night. |
| `pair-marenstrand-bank-day.jpg` | Marenstrand's main pier (W20), `--at 36.8932 171.7947 --yaw 96.3 --pitch -12 --time 0` | Before, the raised box of planks over the sand bank; after, a timber stair up onto it (and one down off its far end). |
| `pair-holbrook-lane-day.jpg`, `pair-holbrook-lane-night.jpg` | Holbrook's lane, `--walk --at 29.69673 1.19287 --yaw 179.4`, `--time 11` and `22.5` | The check: the village stands on one terrace, its lane has no riser, and nothing changed (the captures are byte for byte the same). |

What to look at:

- **Where two step cells meet at the top** (the gate, the up street), a sliver
  of the upper cell's earth shows: the step cells climb toward the middle of
  their higher edges, and the far corner of a side edge is up to 0.33 m under
  the street there (design, decision 1). Walked, it is inside the walker's
  hold.
- **The flights read paler than the street steps** in daylight. Both are the
  same `stone`; each vertex is lit by the light field 0.3 m in front of it,
  and the step cells sit a layer down between risers, where the field's sky
  is lower. Not investigated further.

Nothing stored changes: the stairs are pieces cut from the stored town and
its template; the town records, the ground and every player edit are as they
were.
