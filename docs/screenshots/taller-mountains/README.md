# Taller mountains: screenshots

For `openspec/changes/taller-mountains` task 3.2: generator version 5 beside
version 6, each from the same flags. They were captured on 2026-09-29 in a
cloud container on lavapipe at 1440 x 900, with no GPU. Each is a fresh run
of the fast build (`--profile fast`) at `--time 12` on day 0. They show what
the ground looks like, not how smoothly it runs. Frame cost was not
measured (CLAUDE.md).

- **Version 5** is a scratch slot whose identity names generator 5, opened
  with `--world "Version 5"` and made again before every shot, so no run
  starts where the last one ended. It opens on version 5's settled climate
  (`settled-g5-l5`).
- **Version 6** is a new, memory-only world on version 6's settled climate
  (`settled-g6-l5`).

Both open in rain at the spawn, since that is what each settled climate holds
at noon on day 0. The candidates page
(https://claude.ai/artifact/BXtLUoFSif3oVt1eutiuEr) shows version 6's ranges
at their summits in clear air, from the same generator term.

| files | flags | what it shows |
| --- | --- | --- |
| `map-v5.jpg`, `map-v6.jpg` | `--walk --menu map --map-mpp 12` | The spawn's continent, shaded by relief. Version 6's grey ranges are massifs several kilometres across, where version 5's were patches. The coasts and the lowland are where they were. |
| `map-biomes-v5.jpg`, `map-biomes-v6.jpg` | `--walk --menu map --map-layer biomes --map-mpp 40` | The whole planet's biomes and their shares of the land. Mountains go from 2% to 5%. Fields, desert and jungle each give up a point (23, 23 and 22% to 22, 22 and 21%). |
| `surface-v5.jpg`, `surface-v6.jpg` | `--view surface` | 90 m over the spawn, looking west. Version 5: a desert slope up to a low grey ridge. Version 6: a stone massif fills the view. |
| `range-v5.jpg`, `range-v6.jpg` | `--view column --spawn mountains --height 40 --pitch -18` | 40 m over the nearest Mountains, which is 557 m from the spawn on version 5 and 115 m on version 6. Both are bare stone in 1 m terraces. On version 6 the ridge in front of the camera is the flank of the range the spawn stands under. |
| `orbit-v5.jpg`, `orbit-v6.jpg` | `--view orbit` | The planet from 7,800 m, much of it under cloud in both. |
| `spawn-v5.jpg`, `spawn-v6.jpg` | `--walk` | Eye level at the spawn. **Version 6's is the finding that makes survey H4**. The spawn is on the range's foothill, 94 m up, so the walker faces a wall of dirt and the Kestrel has no level pad within 40 m ("no berth for the Kestrel near the spawn" in the log). Version 5 stands in its field beside the Kestrel. The design's "Found building it" has the numbers. |
