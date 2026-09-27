# Inventory grid: captures

For `openspec/changes/inventory-grid` (tasks 3.3 and 4.3) and
`lamps-and-lanterns` decision 13. Captured 2026-09-27 in a cloud container on
lavapipe, at 1440x900, with no GPU. They show what the game looks like, not
how smoothly it runs. Frame cost was not measured (CLAUDE.md): the owner runs
`tools/perf_suite.py` on real hardware. Videos are deferred to the owner's
batch (CLAUDE.md, 2026-09-27), so these stills are the check for now.

Every shot is a fresh run of the fast build (`--profile fast`) under
`xvfb-run`, on a memory-only save, at the spawn meadow with the view turned
90 degrees from the parked ship (`--walk --yaw 90`). `--dig N --dig-ahead`
digs N blocks along the look; `--lamps 4` puts one of every light in a row
4 m ahead and deals them into the hotbar; `--menu pack` opens the pack. The
clock is pinned with `--time`: 0 is midnight and 12 is noon. It was raining
at the spawn.

| file | flags | what it shows |
| --- | --- | --- |
| `drop-in-its-hole.jpg` | `--dig 6 --pitch -30 --time 12` | One dug block floating at its cell's centre as a small hex prism of its own grass tile. It is 3 m off, outside the magnet's 2.6 m, so the hotbar still reads 64 grass. |
| `drops-noon.jpg` | `--dig 2 --pitch -45 --lamps 4 --time 12` | Two drops, grass and soil, floating in the pit beside the lamp row. |
| `drops-midnight.jpg` | the same at `--time 0` | The same pit at midnight, the drops and the ground lit warm by the lamps. |
| `midnight-dig-before-fix.jpg` | the same, on the build before decision 13 | Before the fix: at midnight the dusk re-bake left the contact behind, and the dig found no ground. The lamps stand; there is no hole. |
| `pack-kit.jpg` | `--menu pack --time 12` | The pack open: thirty empty pack slots, the kit's hotbar under them, and the three mouse moves. |
| `pack-after-dig.jpg` | `--dig 4 --pitch -70 --time 12 --menu pack` | Four blocks dug under the walker, who falls into the pit. The dry grass and soil the dig cut were pulled in by the magnet and, with the hotbar full, landed in the pack's first two slots. The log says so: `picked up 1 of Block(DryGrass) (drop 0)`, `picked up 1 of Block(Soil) (drop 1)`. The two stone drops came to rest 4 m away and float. |

Dry grass and soil draw exactly like the kit's grass and dirt and do not stack
with them: survey I4.
