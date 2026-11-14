# Tasks

Mockups first; game code waits for the owner's approval and a request.

## 1. The mockups

- [x] 1.1 M21 Rockets and Landers: a flyable launch preview steered by the gimbals (engines visibly swivel), turn rate per stage in the assembly building. Verify: screenshots of a gimballed turn by day and at night in `docs/screenshots/mockup-ship-tiers/`.
- [x] 1.2 M24 Hangar and Chibi Aircraft: "Fly it", with ailerons, elevator and rudder marked in the builder, checked, and deflecting in flight; helicopters on cyclic and collective. Verify: screenshots in `docs/screenshots/mockup-hangar/`; a design with no tailplane is refused.
- [x] 1.3 M23 Drydock and Hex Ship Builder: the six power levels as blocks with presets, and "Sail it" with the wind field, waypoints on a compass-north map, the GPS readout and a helm. Verify: screenshots in `docs/screenshots/mockup-drydock/` of a sailing ship tacking to an upwind waypoint, by day and at night.
- [x] 1.2a M24: airships (decision 3b), an Airship tab with 2 m cells, two presets, a shed and a mast, and Fly it for them. Verify: screenshots of a moored airship at night and one flying.
- [x] 1.4 M20 Pits, Plants and Power: oil fields, pumpjack, refinery and oil plant on the power ladder, lit at night. Verify: screenshots in `docs/screenshots/mockup-industry/`.
- [x] 1.5 Republish the four, update the tracker and `docs/mockups/mockup-work.md`.
- [x] 1.6 M23 Sail it, rebuilt on the owner's report of 2026-11-13 (design section 9): sails drive only by canvas set and trim in the apparent wind, with a no-go zone; oars row in strokes; wind arrows, a pennant and a wind instrument; the crow's-feet sea artefact and the cyan route bars replaced by a course line, a dotted route and a wake; walking the deck fixed; oars shipped and a gangway toggle. Verify: 29 Playwright checks pass in headless Chromium (furled she does not move; trimmed on a beam reach she accelerates; head to wind she stalls; best trim beats worst; walk helm to bow and back under way; oars sweep, thrust pulses, more benches more speed, shipped she coasts to a stop; the gangway); screenshots `sail-fix-*.png` in `docs/screenshots/mockup-drydock/`.
- [ ] 1.7 Republish M23 and note the rebuild on the tracker (the parent session).

## 2. The owner's review

- [ ] 2.1 Read the tracker's reviews and the survey, and fold them in.

## 3. Build (on request, after approval)

- [ ] 3.1 Oil fields as a seed-derived field. Verify: a digest test that every shipped generator version's cells are unchanged.
- [ ] 3.2 Gimbals, control surfaces, and helm and throttle. Verify: the craft-control scenarios as core tests.
- [ ] 3.3 Routes and the GPS readout. Verify: the navigation scenarios.
- [ ] 3.4 Ship power blocks. Verify: the ship-power scenarios.
