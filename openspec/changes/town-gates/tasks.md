# Tasks

## 1. See it

- [x] 1.1 `towns::tests::walled_town_gate_spots` (an instrument): prints the
      outside, passage and side-wall spots for each walled town's gates.
- [x] 1.2 Before shots, `--capture`, in Ashenstead's north gate: from outside,
      along the passage, and at the side wall. The same for a tundra camp's
      ice gate. (Ashenstead's north gate is taken; the tundra camp's ice
      gate is covered by the same code and test, and its shots are still to
      take.)

## 2. Fix

- [x] 2.1 `cut_masonry` takes each neighbour's (bottom, top) and draws each
      side over its own span minus the neighbour's. `build_town` passes them.
- [x] 2.2 Test, the walled template: from each gate cell's middle, 2 m up,
      a line toward each wall cell beside it meets a wall face
      (`a_gate_passage_is_walled_on_both_sides`), and `open_spans` by case.
- [ ] 2.2b The same test over the tundra template's ice gate.
- [x] 2.3 After shots from the same spots; the owner sees both.

## 3. Ask

- [ ] 3.1 The riser inside the north gate, with shots: leave it, or make it a
      stair (a template change, a new template version, so towns already
      laid keep theirs). Asked as G1 in the owner survey, 2026-10-08, with
      the outside and passage shots; recommendation: a stone stair.
