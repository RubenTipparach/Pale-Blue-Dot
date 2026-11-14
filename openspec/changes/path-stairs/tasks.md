# Tasks

## 1. Measure

- [x] 1.1 Every riser between path cells in every template, read off the
      templates (design, "Measured"), and the spots for the shots:
      `towns::tests::print_the_harbours_up_street_spots` (an instrument) and
      `walled_town_gate_spots`.
- [x] 1.2 Before shots: Holbrook's lane by day and at night, Ashenstead's
      north gate, Holinghaven's up street, Marenstrand's bank.

## 2. Build

- [x] 2.1 `pieces::steps`: the path cells and their risers from a stored
      town and its template; street steps across a cell (`Surface::Steps`);
      a flight against a riser; the yield check.
- [x] 2.2 `harbour::pier`: a timber stair at each riser between stretches.
- [x] 2.3 `record::build_town_yielding` cuts the stairs after the buildings;
      `build_town` is it with nothing yielded. The app's
      `towns::build_yielding` hands it the town's cells that hold an edit.
- [x] 2.4 `coast.json` and the mockup's `makeCoast` name the up street's
      four step cells ", steps".

## 3. Verify

- [x] 3.1 Core: every riser between path cells in all six templates has a
      stair (`every_risers_path_has_its_stair`); a walker up and down each,
      no eye jump over 0.1 m, no tick in the air, with auto step off
      (`a_walker_climbs_every_path_stair_without_a_hop`).
- [x] 3.2 Core: a pier's raised stretch has a stair at each end and is
      walked (`a_pier_climbs_a_bank_by_a_stair_at_each_end`).
- [x] 3.3 Core: no stair where a cell holds an edit; a stored town builds
      to the same pieces with its stairs added
      (`a_stair_yields_to_an_edit`, `a_stored_town_gains_only_its_stairs`).
- [x] 3.4 App: every town of the shipped seed has a stair on each of its
      path risers, and Marenstrand's bank its two
      (`every_towns_path_risers_have_their_stairs`).
- [x] 3.5 After shots from the same spots, in `docs/screenshots/path-stairs/`.
      Shipped seed: 6 walled towns with 18 risers each and 12 stairs, 6
      harbours with 12 risers each and 6 stairs, Marenstrand's main pier
      with its two; 20 villages, 4 deserts, 4 tundra camps and 6 jungle
      villages with no riser. Holbrook's shots are byte for byte the same.

## 4. The owner

- [x] 4.1 G1 (`town-gates` 3.1) and W20 (`pier-landfall` 1.6) answered by
      the owner's rule of 2026-11-14 and recorded in their changes.
- [ ] 4.2 The owner sees the shots.
- [ ] 4.3 The video, with the batch (CLAUDE.md, screenshots stand in for
      now).
- [ ] 4.4 Later, its own change: take a stair out of a standing town when
      the player digs or builds in its cell (design, decision 3).
