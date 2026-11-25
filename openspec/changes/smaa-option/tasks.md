# Tasks

## 1. Build
- [x] 1.1 `PlayerSettings.antialiasing` (a bool: OFF or ON (SMAA)), the file field with a version bump, older files read OFF, `--antialiasing on|off`, `ISOLATED` OFF; tests as REFLECTIONS has.
      Done: settings file version 11, `a_file_from_before_antialiasing_reads_it_off_and_keeps_the_rest`, the round trip, the flag, and the repro record carries it.
- [x] 1.2 The camera follows it: `Smaa` (high) added or removed; never with `PBD_MSAA` samples.
      Done: `settings::follow_antialiasing` on every 3D camera with `Msaa::Off`; `the_cameras_follow_antialiasing_and_never_stack_it_with_msaa`. The tone grade is ordered before SMAA (`tone::order_smaa`), since Bevy leaves the two unordered.
- [x] 1.3 The settings screen row, labels only (CLAUDE.md, "Notes stay out of the game").
      Done: an "Antialiasing" checkbox on PICTURE, under Shadow quality, with no note.
- [x] 1.4 The loading prewarm compiles SMAA's pipelines; a run that switches it on after READY logs no `LATE_PIPELINE`.
      Done: SMAA is on the cameras while the screen warms; a `--real-time` run switching it on after READY (`PBD_CYCLE_ANTIALIASING`) logged no `LATE_PIPELINE` (`docs/screenshots/smaa-option/README.md`).
- [x] 1.5 A test that SMAA's node follows the water's node in the graph.
      Done: `planet::water::tests::smaa_runs_after_the_water_and_the_grade` builds the 3D graph headless with Bevy's plugins and the game's own calls, and checks SMAA comes after the water's node and the grade.

## 2. Show
- [ ] 2.1 Stills of OFF against SMAA: under water with the rod, a town at night, terrain textures up close, in `docs/screenshots/smaa-option/`.
- [ ] 2.2 Frame time OFF against SMAA on the desktop (owner).
