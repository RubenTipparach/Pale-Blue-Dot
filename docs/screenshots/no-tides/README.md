# No tides (`no-tides`)

The owner, 2026-11-19: "Remove literal tide feature." The write-up is
`openspec/changes/no-tides/design.md`.

- **Renderer:** a Claude Code cloud container with no GPU: Mesa lavapipe
  (software), at a fixed step. The captures show what things look like,
  never how fast they run. Frame time was not measured.
- **Settings:** `--profile fast`, `xvfb-run`, `PBD_WINDOW=960x600`, a new
  memory-only world, `--fly --at 39.725 15.339 --time 12 --rain 0 --frames
  300`, the camera held by `PBD_EYE_PATH` (one key, `0 39.725 15.339 25 0
  -85`: 25 m over the sea's level, looking straight down). The corner label
  is the eye path's own height over the water under the eye.
- **Before:** `claude/phase-2-1` at `99edf9d4`, with `--tide high` (the
  tide's moon put over the camera, the sea 2 m over its mean there; the
  capture flag is gone with the tide). **After:** this branch.

| File | What it shows |
| --- | --- |
| `beach-from-above-before-after.jpg` | The owner's beach of `sea-hole-beach`. BEFORE, high tide: the sheet stands 2 m over the mean (`+23.50 m to water` at 25 m) and covers the whole beach, its sand seen through the water. AFTER: the sea at its one level (`+25.50 m to water`), half a metre under the sea's level; the beach is dry ground. |

What else the AFTER half shows, and is the change working as written up:

- **Trees on the strip the tide used to cover.** The flooded-ground line is
  the sea's level now (design 3), not 1.5 m over it, so the acacias the
  generator's own rules put behind a warm beach (`flora-v2` 12.13) now
  stand on ground the high tide used to keep bare. No generator code or
  value changed; only the line under which nothing is drawn growing moved.
- **The two dark cells in the beach** are the natural pit of
  `sea-hole-beach` (case A), a generated hollow beside the beach. The tide
  does not change them.
