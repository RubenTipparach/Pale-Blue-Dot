# Tasks

## 1. Build
- [ ] 1.1 `PlayerSettings.antialiasing` (a bool: OFF or ON (SMAA)), the file field with a version bump, older files read OFF, `--antialiasing on|off`, `ISOLATED` OFF; tests as REFLECTIONS has.
- [ ] 1.2 The camera follows it: `Smaa` (high) added or removed; never with `PBD_MSAA` samples.
- [ ] 1.3 The settings screen row, labels only (CLAUDE.md, "Notes stay out of the game").
- [ ] 1.4 The loading prewarm compiles SMAA's pipelines; a run that switches it on after READY logs no `LATE_PIPELINE`.
- [ ] 1.5 A test that SMAA's node follows the water's node in the graph.

## 2. Show
- [ ] 2.1 Stills of OFF against SMAA: under water with the rod, a town at night, terrain textures up close, in `docs/screenshots/smaa-option/`.
- [ ] 2.2 Frame time OFF against SMAA on the desktop (owner).
