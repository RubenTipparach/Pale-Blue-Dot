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
- [ ] 4.3 The fade between the three forms. Verify: the frame-by-frame no-pop check on a scripted fly-in at cruise speed.
- [ ] 4.4 Unsettled sites: a site with earlier edits is skipped and marked on the map. Verify: an app test with an old save holding an edit in a footprint.
- [ ] 4.5 `WorldFile` records the template version. Templates are loaded per version. Verify: format tests, and a test that a world on v1 still builds v1 after a v2 exists.

## 5. Light

- [ ] 5.1 `WindowCandle`, a dusk-lit material, placed behind about half the windows by the site's seed. Verify: core tests of its emission and dusk flag, and a test that a lit window's room is lit and its outer wall face is not.
- [ ] 5.2 Street and wall lanterns placed by the template's spacing. The tier bakes the whole town. Verify: the bake time is recorded against `lamps-and-lanterns`' 300-lantern figure, and captures of a walled town at 22:30 from the street and from above.

## 6. The owner's check

- [ ] 6.1 A `town` scenario in `tools/perf_suite.py`, and a note that frame cost was not measured in the cloud session. Verify: the scenario runs in a smoke test, and the note is in the PR.
- [ ] 6.2 A capture page of every kind of settlement in the world, by day and at night, beside the mockup's views. Verify: published and linked from the PR.
- [ ] 6.3 The owner accepts. Verify: the quote is in `proposal.md`. Sync `world/settlements`, and archive.
