# mountain-ground: in-game captures

`openspec/changes/mountain-ground` tasks 4.1 and 6.5, after the owner
approved M13's Mountains tab (2026-11-20). Captured in a cloud container on
lavapipe (no GPU, software rendering) at 1280 x 800: they show the look,
never a frame time, and the frame cost was not measured. Each is a fresh run
of the fast build on a new memory-only world, 150 frames, made by
`tools/captures/sand_mountain.sh` (`WIN=1280x800 bash
tools/captures/sand_mountain.sh air14 air15 foot14 foot15 meadow line snow
scree night`). Places were found by `cargo run -p pbd-core --example
mountain_places`: one range at about 31 N, 8.5 W.

"Version 14" is today's ground (stone under a 200 m snowcap, no trees) as
this build draws it: the draw-time looks (rock strata, lichen, scree, the
risers' colours) reach every world, so a v14 shot shows them too. "Version
15" is what a new world is made on.

| File | Flags | What it shows |
| --- | --- | --- |
| `flank-air-v14.jpg` | `--generator 14 --fly --height 60 --at 31.30 -8.70 --time 11 --rain 0 --pitch -30 --yaw 60` | The flank from the air on version 14: all rock, now in strata of the sheet's rocks with lichen-stone patches, where it was one slate |
| `flank-air-v15.jpg` | the same, version 15 | The same flank on version 15: its flats are alpine turf, a flat bench near the top holds snow, the steep cells stay rock |
| `flank-foot-v14.jpg` | `--generator 14 --walk --at 31.4450 -8.4220 --time 11 --rain 0 --pitch -8 --yaw 200` | On foot at 142 m on version 14: bare rock with lichen in its texels |
| `flank-foot-v15.jpg` | the same, version 15 | The same place on version 15: the alpine meadow (turf, tufts from the turf's colours, flowers, juniper and heather, sedge, cushions) and firs at the tree line |
| `meadow-close.jpg` | `--walk --at 31.2776 -8.4563 --time 10 --rain 0 --pitch -40 --yaw 30` | The meadow close: the turf top, a riser of the fields' sod side brought to the turf's green, and the alpine soil under it |
| `tree-line.jpg` | `--walk --at 31.2656 -8.8345 --time 15 --pitch -4 --yaw 0` | Firs (the taiga pine at 5 to 8 m) thinning up the slope to the tree line, in light rain |
| `snow-line.jpg` | `--walk --at 30.5872 -9.3497 --time 12 --pitch -20 --yaw 90` | A flat just under the snow line: snow lingering in patches on the turf, in light rain |
| `scree.jpg` | `--walk --at 28.5972 -11.7858 --time 12 --rain 0 --pitch -30 --yaw 0` | Rock at a cliff's foot: the scree tile, the rocks' strata and lichen, the range's firs below |
| `night-lamp.jpg` | `--walk --at 31.2776 -8.4563 --time 22 --pitch -30 --yaw 30 --lamps 4` | The meadow at 22:00 with the scripted lamps ahead and a held torch: the turf, the riser and the soil lit by the lamps and the torch as the terrain is |

A capture does not warm the pipelines, so its log lists them as
`LATE_PIPELINE` by design (see `docs/screenshots/cooking/README.md`). A
normal warming launch of the same build at the version 15 meadow among the
firs (`--walk --at 31.4450 -8.4220 --time 11 --yaw 200`, 170 s) reached
`READY` (113 s, lavapipe) and logged no `LATE_PIPELINE` line at all.

The lichen-stone patches were first a per-fragment noise, which cut cells
into camouflage in an early `scree.jpg`; they are now chosen a whole cell at
a time, and the shots of rock were retaken with that (`tree-line.jpg`,
`snow-line.jpg` and `night-lamp.jpg`, mostly turf, were taken before it, and
`tree-line.jpg` before the riser's grass was brought to the turf's green). The weather at the range
was overcast with showers; `--rain 0` stops the rain but not the cover.

The game writes PNG; the files here are those PNGs saved as JPEG at quality 90.
