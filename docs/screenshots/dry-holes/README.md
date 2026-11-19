# Dry holes, and a sea that flows in (`dry-holes`)

The owner, 2026-11-18: "We should be able to dig holes below water line and
not have it filled with water too", and "Ocean needs to work like rivers. In
the sense that the water flows." The write-up is
`openspec/changes/dry-holes/design.md`.

- **Renderer:** a Claude Code cloud container with no GPU: Mesa lavapipe
  (software). The captures show what things look like, never how fast they
  run.
- **Settings:** `--profile fast`, `xvfb-run`, `PBD_WINDOW=960x600`, `--time
  12 --rain 0 --frames 120`, a fixed-step capture.
- **Before:** `claude/phase-2-1` at `07e22b17`. **After:** this branch.
- **The rig:** `--pit N --pour-ahead 0.1` digs a seven-cell pit (the cell
  under the camera and its six neighbours), each cell `N` layers down from
  its own top, through the edit path a player's dig takes.
  - The views from above fly the camera with `PBD_EYE_PATH` (one key:
    `0 <lat> <lon> <alt> 30 -80`). Its corner label is the eye path's own:
    `+9.98 m to water` means a water surface is under the eye, and
    `no water` means none is.
  - `PBD_PIT_SEA_LAST=1` (a capture instrument) digs the pit's cells beside
    the sea last, so the pit is dug dry first and then opened to the sea.

| File | Place | What it shows |
| --- | --- | --- |
| `beach-pit-inside-before-after.jpg` | `--walk --at 39.61458 15.39634 --pit 5 --pour-ahead 0.1 --pitch -15`: land 1-2 m up beside a beach the high tide covers, the sea three cells off | Standing in the pit, its floor 3 m under the sea. BEFORE: the eye is "under the sea", with blue-green murk and the sea's surface overhead, though every cell of the pit is air. AFTER: a dry, sunlit sand pit under the sky. |
| `beach-pit-above-before-after.jpg` | the same pit, the camera 9 m up | BEFORE: the sea's sheet lies over the pit (`+9.98 m to water`). AFTER: no sheet (`no water`): the pit's floor and walls bare. |
| `river-pit-above-before-after.jpg` | `39.51022 15.83776`, `--pit 4`: a river bank inland, the pit's floor 3 m under the river's surface, a cell of bank between | Dry before and after: no sea's reach so far inland, and the river does not leak into a pit whose wall stands. The rule only moves water across a neighbour. |
| `shore-pit-sea-flows-in-before-after.jpg` | `39.50670 15.41653`, `--pit 4`: a pit dug on the shore itself, beside the sea's water | BEFORE: the edit floods the whole pit at once. AFTER, 2 s in (two of the rule's ticks): the sea is flowing in. The pit's far cells hold water below the sea's level, drawn by the mesher, with no sheet over them yet. AFTER, `--water-ticks 40`: full to the sea's level, under the sea's sheet (the rule changed 11 cells; the rest filled at the edits that opened them onto the sea). |

The same, measured on the CPU with `dry_hole_report` (an ignored test in
`planet_water.rs`): the beach pit at `39.61458 15.39634` before was listed
under the sheet, reached, and the eye in it was `Water { surface 4800.0 }`.
After, the pit is dry and unlisted and the eye reads `Air`, after the dig,
after the rule and after a reload.

## No sea shading in a dry pit (design 10, 2026-11-19)

The owner, on the two beach-pit captures above: "It shouldn't shade dry
holes at all", and on the view from above: "This is green as fuck." The
green was the water's compose taking every pixel under the sea's surface
for sea (design 10 has the instrument and its five shots). The crosshatch in
the captures above was the dig's dither fade still running; these are taken
at 400 frames, long past it (`lod_fade_s` 1 s, 60 frames).

- **Renderer:** lavapipe in a cloud container, a fixed step, no GPU. Looks
  only; frame time was not measured.
- **Settings:** `--profile fast`, `xvfb-run`, `PBD_WINDOW=960x600`, a new
  memory-only world, `--at 39.61458 15.39634 --pit 5 --pour-ahead 0.1 --time
  12 --rain 0 --frames 400` (the sun 70 degrees up). Captured at the size
  shown, 960 x 600 a half, no resize.
- **Before:** this branch at `8b8ebec8` (no tides, the compose as it was).
  **After:** the same build with this change's two shaders
  (`planet_surface.wgsl`, `water.wgsl`); no Rust changed for it.

| File | View | What it shows |
| --- | --- | --- |
| `beach-pit-inside-dry-before-after.jpg` | `--walk --pitch -15`, standing on the pit's floor | BEFORE: the walls green under the sheet's level and greener deeper, the floor blue-green. AFTER: the beach's tan sand to the floor; the lower walls are only as dark as their shade. |
| `beach-pit-floor-sun-before-after.jpg` | `--walk --pitch -65`, looking down at the floor in the noon sun | The sun on the floor, so shade cannot be taken for tint. BEFORE: the sunlit floor and walls green. AFTER: the sunlit floor cells the beach's tan; the cell underfoot is the next biome's own stone tile (its sheet is not the beach's: design 10's third shot), dark and mossy as that sheet draws its stone, not a tint. |
| `beach-pit-above-dry-before-after.jpg` | from 9 m up (`--fly`, `PBD_EYE_PATH` key `0 39.61458 15.39634 9 30 -80`) | BEFORE: the pit's floor and lower walls green-grey. AFTER: the walls the beach's tan sand; the west floor cells the beach's own stone (tan boulders); the east floor cells the stone of the biome the dark ground to the east belongs to, which its sheet draws mossy and green. That is the cell's own tile (design 10's third shot: not the beach's sheet, not the ocean's), not the sea's murk. The acacia over the beach is `no-tides`: the strip the tide used to cover grows what the beach's back grows. |
