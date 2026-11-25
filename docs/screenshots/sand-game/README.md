# sand-tiles: in-game captures

`openspec/changes/sand-tiles` task 1.5, after the owner approved M13's Sand
tab (2026-11-20). Captured in a cloud container on lavapipe (no GPU, software
rendering): they show the look, never a frame time. Each is a fresh run of
the fast build (`cargo build --profile fast -p pbd-app`) on a new memory-only
world (generator version 15), made by `tools/captures/sand_mountain.sh`
(`bash tools/captures/sand_mountain.sh beach desert placed`), 150 frames.
`beach-day.jpg` and `desert-day.jpg` are 1600 x 1000, `placed-desert-sand.jpg`
1280 x 800. The world's weather at those places was light rain.

| File | Flags | What it shows |
| --- | --- | --- |
| `beach-day.jpg` | `--walk --at 36.8594 58.2286 --time 11 --pitch -30 --yaw 0` | A beach by day: the redrawn shell sand, warm with cream, white, pink and coral shells and bits, no teal or blue anywhere, the beach's terraces into the sea |
| `desert-day.jpg` | `--walk --at 36.8675 0.3294 --time 11 --pitch -30 --yaw 0` | A desert by day: the new orange-ochre sand, fine grain with rust grains and quartz glints, no shells; the sand's riser shows the same sand |
| `placed-desert-sand.jpg` | `--walk --at 36.8675 0.3294 --time 11 --pitch -20 --yaw 0 --place-row "sand,desert sand,sand,desert sand" 4` | Placed blocks in the dunes: a block of sand (the beach's shell sand, top right) beside desert sand, which is drawn from the desert sheet and sits in the dunes as their own sand |

A capture does not warm the
pipelines, so its log lists them as `LATE_PIPELINE` by design (see
`docs/screenshots/cooking/README.md`).

The game writes PNG; the files here are those PNGs saved as JPEG at quality 90.
