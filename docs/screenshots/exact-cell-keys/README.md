# Exact cell keys: captures and the gate video

For `openspec/changes/exact-cell-keys`, tasks 3.2 and 5.1. Captured
2026-09-27 in a cloud container on lavapipe, at 1440x900, with no GPU. The
captures show what the game looks like, not how smoothly it runs; frame cost
was not measured (CLAUDE.md).

- **The old build** is `ecabf4f`, the commit before the fix.
- **The new build** is the fix.
- Both were built with `--profile fast` and run under `xvfb-run`.

## The pair

`--colliding-pairs` lists the finest cells whose old save ids collide,
nearest the spawn first. The video uses the first pair whose two cells stand
on the same ground layer, so an edit at one cell's surface is also at the
other's:

| cell | key | metres from the spawn | ground |
| --- | ---: | ---: | ---: |
| A | 1328120728 | 303 | 98 m |
| B | 1329517027 | 1,359 | 98 m |

Before the fix, both were saved as id 2419118392.

## The shots

Each shot is a fresh run of the game, which opens the `twins` world with the
player's saved pose moved to stand 6 m south of the cell, facing it, at noon
(`--walk --time 12`).

| file | what it shows |
| --- | --- |
| `old-1-dig.jpg`, `new-1-dig.jpg` | Standing on A, looking down: `--dig 1 --place 3` digs A's top layer and stacks three stones over it. The old build logs the edit as cell 2419118392; the new one as 1328120728. |
| `old-2-at-A.jpg`, `new-2-at-A.jpg` | A from 6 m, with the pit and the stones, on both builds. |
| `old-3-at-B.jpg`, `new-3-at-B.jpg` | B, 1.1 km away, which nobody touched. On the old build it has A's stones; on the new build it is grass. |
| `old-4-reload-A.jpg`, `new-4-reload-A.jpg` | A again after the world is closed and opened: its edit is kept. |
| `cell-B-before-after.jpg` | Shot 3, the two builds side by side. |
| `exact-cell-keys.webm` | The gate video: every shot, old beside new, captioned. 35 s. |

The dig and view-of-A files are byte-identical between the builds: the same
edit, drawn the same.

## The ground looks the same (task 3.2)

| view | old build | new build | pixels that differ |
| --- | --- | --- | ---: |
| `--view meadow --frames 120` | `meadow-old.png` | `meadow-new.png` | 0 of 1,296,000 |
| `--view surface --frames 120` | `surface-old.png` | `surface-new.png` | 0 of 1,296,000 |

- A first capture of the new build's surface view differed from the old one
  in 12 pixels of sky, each by one level of one colour channel.
- A second capture of the same new build differed from that first capture in
  2 pixels, and matched the old build exactly. So the 12 were the sky's
  run-to-run noise, not the fix, and the second capture is the one kept here.
- Two captures of the meadow view on the new build were identical.
