# Ore drawn as ore (`flora-v2` design 12.12, task 9.9)

These are in-game captures from 2026-11-18. They were taken in a cloud
container with no GPU, on lavapipe at 1920 x 1200, using the fast build under
`xvfb-run` and a new memory-only world. A software render shows what things
look like. It says nothing about how smoothly they run, and the frame cost
was not measured.

**Before** is `claude/river-cache-full` at `6de31719`. **After** is this
branch. Both ran the same command in the same sitting.

Every shot used these common flags:

    PBD_WINDOW=1920x1200 xvfb-run -a -s "-screen 0 1920x1200x24" target/fast/pbd-app \
      --time 12 --rain 0 --weather-at 7200 --rain-shadows off <shot flags> --capture <file>

| files | shot flags | what it shows |
| --- | --- | --- |
| `before-placed-ore.jpg`, `after-placed-ore.jpg` | `--walk --spawn mountains --place-row ore,stone,ore,ore 4 --pitch -20` | A row of placed blocks, 4 m ahead: ore, stone, ore, ore. Before, the ore blocks (right, and the far left edge) are plain stone. After, they show coal (near black), copper (rust orange) and tin (steel grey) flecks on the fields' stone tile, and the stone block (left) is unchanged. |
| `placed-ore-crop.jpg` | (a crop of `after-placed-ore.jpg`, 1:1) | The flecks on the stone tile's own 32-texel grid. Each fleck is a 2 x 2 core with up to three texels beside it, lit upper left and with no outline. |
| `before-dug-vein.jpg`, `after-dug-vein.jpg` | `--walk --at 21.71525 0.79677 --yaw 0 --pitch -45 --tool pickaxe --dig 2 --dig-ahead` | A desert hill of rock. The scripted dig takes the rock cap and one layer of ore from the cell ahead (cell 1328067268, layers 228 and 227), which leaves the vein's next ore layer as the pit floor. The floating drops are the dug rock and a mixed ore block. Before, the floor is plain desert stone. After, it shows flecks. The pit walls are the neighbours' `Rock`, which never turns to ore. |
| `dug-vein-floor-crop.jpg` | (a crop of both, 2x nearest) | The pit floor, before on top and after below. After, it shows coal, a copper fleck and tin beside the drop. On the desert's red stone, copper reads less strongly than coal and tin. |
| `before-vein-floor.jpg`, `after-vein-floor.jpg` | `--walk --at 21.71544 0.75693 --tool pickaxe --dig 1 --pitch -85` | Standing on the vein's own cell and digging the rock cap from under the feet. The walker drops onto the ore floor and looks straight down at it from 0.6 m. After, coal and tin flecks show in the desert stone. |

Where to dig was found by a measurement instrument that was not committed: an
ignored test that built the column tier at `--at 21.737 0.724` and listed the
columns with ore just under their top, by the columns' own centres. Ore is
stone turned to ore (generator version 10), so it shows only where the ground
is dug or caved. The `Rock` of desert and mountain cliffs never turns.

On this branch, a capture from a fixed camera (`--view cave`, `--view column`)
panics on its first frame ("Message not initialized"). The build before this
change does the same. These shots therefore use walking captures.
