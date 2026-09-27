# Tasks

Step 2d, the last of step 2. It starts after the owner approves the lights,
the map, the sites, and the climate and fish layers, in that order.

## 1. The pieces: `tenebris-towns`, built

- [ ] 1.1 The owner answers `tenebris-towns`' open questions. Record the answers in its proposal. Verify: each question has an answer quoted.
- [ ] 1.2 `tenebris-towns` tasks 1 and 2: thin solids and surfaces in `stand`, and the walker's three rules. Verify: that change's tests, on synthetic stairs, with no eye jump over 0.1 m and no airborne tick.
- [ ] 1.3 `tenebris-towns` tasks 3, 3b, 3c and 3d: pieces, kits, settlement templates and the harbour, as data under `assets/settlements/v1/`. Verify: that change's layout checks pass on every template.
- [ ] 1.4 `tenebris-towns` tasks 4 and 5: drawing, light, furniture, trim and doors. Verify: that change's tests. Then sync its `world/settlements` and `player/walking` deltas and archive it.

## 2. Onto the sphere

- [ ] 2.1 The chart: axial to cells through the neighbour tables, with rotation and mirror. Verify: core tests on every site of the shipped seed that layout neighbours are sphere neighbours and no cell repeats.
- [ ] 2.2 Seeded variation with the layout checks and the fixed fallback order. Verify: tests that two village sites differ, that every variation built passes every check, and that the result is identical on one thread and many.

## 3. The ground

- [ ] 3.1 The settlement ground function (footprint, margin and cleared bit) and the column generator reading it. Verify: tests that terraces come out equal from inside-out and outside-in generation; that no step in the margin is over one layer; and that no tree or clutter is in a street or plot.
- [ ] 3.2 Sync the modified `planet/terrain` requirement with its new test. Verify: `openspec validate --all`.

## 4. In the world

- [ ] 4.1 A settlement becomes an active chunk entity in range, with its pieces, contact index and meshes built on the pool and published whole. Verify: an app test walking from the spawn to a village site and in through a door, and `--capture` shots in `docs/screenshots/cities-in-the-world/`.
- [ ] 4.2 The far form and the night points, drawn by the planet pass. Verify: captures of a town from a kilometre by day, and of the night side from orbit.
- [ ] 4.2a The night lights light the coarse LOD hexes under each settlement (survey C3). Verify: a capture of the night side from orbit before and after, and a test that a hex under a lit town reads a glow and one outside its margin reads none.
- [ ] 4.2b Every harbour boat is usable (survey T7, "you can use any boat you find"). The small boats are the game's existing sailing and paddle craft, parked as vehicle records at their moorings. Verify: an app test boards a moored boat from the pier and paddles it away, and a reload finds it where it was left.
- [ ] 4.3 The fade between the three forms. Verify: the frame-by-frame no-pop check on a scripted fly-in at cruise speed.
- [ ] 4.4 Unsettled sites: a site with earlier edits is skipped and marked on the map. Verify: an app test with an old save holding an edit in a footprint.
- [ ] 4.5 Settlements and their building definitions as records in `world-persistence`'s store, generated for the whole planet at creation or on an old world's first open. Pieces are derived from the definitions, and abandoned buildings stay dark. Verify: tests that a revised template leaves a made world's towns unchanged, that the test process's abandonment darkens a house and survives a reload, and that an old save gains its records once.

## 5. Light

- [ ] 5.1 `WindowCandle`, a dusk-lit material, placed behind about half the windows by the site's seed. Verify: core tests of its emission and dusk flag, and a test that a lit window's room is lit and its outer wall face is not.
- [ ] 5.2 Street and wall lanterns placed by the template's spacing. The tier bakes the whole town. Verify: the bake time is recorded against `lamps-and-lanterns`' 300-lantern figure, and captures of a walled town at 22:30 from the street and from above.

## 6. The owner's check

- [ ] 6.1 A `town` scenario in `tools/perf_suite.py`, and a note that frame cost was not measured in the cloud session. Verify: the scenario runs in a smoke test, and the note is in the PR.
- [ ] 6.2 The gate video (`step-videos`, showcase `towns`). Its shots are:
  - a walk from the fields into each kind of settlement;
  - up an inn's stair and a keep's newel;
  - a door opened;
  - dusk falling on the walled town, with its lanterns and windows coming on;
  - a flight in from a kilometre, showing the fade;
  - the night side from orbit;
  - a house abandoned by the test process, going dark.

  It is published with stills beside the mockup's views. Verify: the gate page is linked from the PR.
- [ ] 6.2a Ask the owner to record the fly-in and the dusk shots in real time with `obs-record`, since a fixed-step video cannot show pop-in or hitches. Verify: the request is in the PR.
- [ ] 6.2b Walk `docs/plans/towns-mockup-parity.md`: every row is built, or agreed mockup-only in the survey. Each built row has a shot in a gate video beside the same shot in the prototype. Verify: no row reads planned or ask.
- [ ] 6.3 The owner watches both videos and accepts. Verify: the quote is in `proposal.md`. Sync `world/settlements`, and archive.
