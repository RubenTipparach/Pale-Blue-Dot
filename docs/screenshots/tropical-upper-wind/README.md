# tropical-upper-wind: the jet's calm band at the equator

The day's mean wind at cloud height, the value the Jet overlay draws, on the
overlay's own ramp (0-45 m/s, `pbd_app::overlay::RAMPS`). It was drawn over
version 5's settled climate at level 5, through one day. Dotted lines mark
the equator and 15 and 30 degrees either side.

The shots come from a scratch instrument, which is not committed. It reads
the model's surface wind and air temperature, and works out the
cloud-level wind under each candidate rule. `openspec/changes/tropical-upper-wind/design.md`
describes it and has the numbers.

| file | what it shows |
| --- | --- |
| `options.png` | the four side by side |
| `jet-built.png` | as built: calm within 6 degrees of the equator, and the jet at its 45 m/s cap from 12 degrees north |
| `jet-a.png` | A: the cap before the fade; the edge spreads out, and the band stays calm |
| `jet-b.png` | B: A, with an 8 m/s easterly inside the band |
| `jet-c.png` | C (recommended): B, with the jet fading in across the tropics and full by 30 degrees |

These are not captures from the game. The game's before-and-after shots of
the globe and the map (task 4.2) follow once a candidate is chosen and built.

## In the game, before and after (task 4.2)

Captured 2026-09-29 in a cloud container on lavapipe at 1440 x 900, with no
GPU. Each is a fresh run of the fast build on a new, memory-only world, which
opens on version 5's level-5 settled climate at noon on day 0. They show what
the overlay looks like, not how smoothly it runs; frame cost was not measured
(CLAUDE.md). The `before` build is the commit before the change
(`3c46b54`).

| files | flags | what it shows |
| --- | --- | --- |
| `game-globe-before.jpg`, `game-globe-after.jpg` | `--view orbit --overlay jet --time 12` | The globe's Jet overlay from orbit. Before: a dark calm belt across the equator, with the jet at its cap right against its northern edge. After: the belt blows from the east at about 8 m/s, its streaks moving, and the jet grows out of it toward 30 degrees. |
| `game-map-before.jpg`, `game-map-after.jpg` | `--walk --menu map --map-overlay jet --map-mpp 40 --time 12` | The whole planet on the map. Before: the flat dark band with hard edges the owner asked about. After: a blue-teal easterly belt with a thin, slower line near 12 N, where the easterly turns into the jet. Poleward of 30 degrees the two are the same. |
