# beavers-otters-game: the beavers and otters in the game (`ocean-environments` 20.3 with 8.4, design B29)

The owner, 2026-11-20, on M25 version 5's Beavers and otters tab
(https://claude.ai/artifact/Ky4u9ybCSUm31t72NRMAA1#critters): "its good
enough, send it". These are the first frames of the sculpted, skinned,
furred beaver, river otter and sea otter in the game, moving as the tab
moves them.

Fixed-step captures in the Claude Code cloud container: **lavapipe, no
GPU**, `--profile fast`, memory-only worlds, a 960 x 600 window. They show
poses and the look only: **frame time was not measured**, and nothing here
says how smoothly anything moves. Each `-close` file is the same frame
cropped round the animal and scaled up by whole pixels (nearest).

The animals are placed by the `--life ID:METRES` instrument a little past
their shyness (the beaver's 7 m, the otters' 8 m), so they are small in the
frame; nearer, a beaver slaps its tail and dives and an otter swims off.
`PBD_CRITTER_PLAN` (a capture instrument, B29) starts each drawn animal on a
chosen action instead of its first seeded draw. `PBD_BEAVER_AGE_DAYS=40`
founds the colony forty days ago, so its dam and lodge stand (B25). The
colony's site at 58.7425 135.6757 is the one in `underwater-2`'s
`beaver-dam-before-after.jpg`.

| Shot | Command | What it shows |
| --- | --- | --- |
| `beaver-swim.jpg`, `beaver-swim-close.jpg` | `PBD_CRITTER_PLAN=swim`, `--walk --at 58.7425 135.6757 --day 41 --yaw -60 --life beaver:9 --pitch -15 --time 4.75 --frames 420` | A beaver swimming in the colony's pond, its head and back above the water and the rest under it, dimmed by the water as the bed is. The spawner itself had put a beaver 19 m off in the pond (the log: "a beaver drawn 19 m off"): task 8.4, a beaver only by its colony. |
| `beaver-bank.jpg`, `beaver-bank-close.jpg` | `PBD_CRITTER_PLAN=bank`, the same place with `--yaw 60 --frames 760` | A beaver out on the bank of the pond, in the shade of the bank: it swam to the water's edge, hopped out, shook off and sat down to gnaw a stick. Its fur is still darkened by the wet. |
| `river-otter.jpg`, `river-otter-close.jpg` | `PBD_CRITTER_PLAN=look`, the same place with `--yaw 60 --life river_otter:10 --frames 460` | A river otter in the pond, treading water to look round. At 11 m and dimmed by the water it is a small dark shape: the weakest frame here. |
| `sea-otter.jpg`, `sea-otter-close.jpg` | `PBD_CRITTER_PLAN=crack`, `--walk --at 86.965 -17.827 --life sea_otter:10 --yaw 35 --pitch -12 --time 12 --frames 420` | A raft of three sea otters on their backs on a cold coast, each with an urchin held up between its forepaws to crack on the stone on its chest. |
| `beaver-night.jpg` | `PBD_CRITTER_PLAN=swim`, the pond with `--yaw 60 --life beaver:8 --pitch -18 --time 13 --torch --frames 420` | **Not the shot asked for.** Night at the pond with a torch on the bank under the eye: the bank by the torch is lit and the pond is dark, and the beaver swimming 10 m out (the log: "a beaver drawn 10 m off") is as dark as the water round it, so it cannot be picked out, even brightened. A run with a row of lamps (`--lamps 7`) lit the bank but drew the instrument's beaver 68 m off, for a reason not found; a beaver lit by a lamp is still owed, best taken on the owner's GPU. |

The motion itself (the gnaw's beat, planted feet, the forelimb's limits)
is pinned by tests, not by these frames: `critters::tests`.

## The loading screen

A normal launch (no `--capture`, lavapipe) at the colony,
`PBD_BEAVER_AGE_DAYS=40 pbd-app --walk --at 58.7425 135.6757 --day 41 --yaw 60 --life beaver:9 --time 4.75`,
showed 97 prewarm proxies behind the screen and reached `READY` with 120
pipelines; a river otter (from the spawner, 34 m off) and the beaver were
then drawn, and the log has no `LATE_PIPELINE` and no error. A capture
skips the warm-up, so every capture logs its pipelines as late by design.
