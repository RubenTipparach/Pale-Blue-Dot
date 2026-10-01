# Tasks

After the roadmap, at the owner's word (survey T9). It needs `tenebris-towns`
tasks 1 and 3d (thin solids and surfaces in `stand`, and the ship piece).

## 0. The cog moored (design 6, step 1)

- [ ] 0.1 The export writes the cog (where it lies, its heading, its gangway side) and its gangplank; the ship piece is cut at its berth from the template, with its hull, deck, rail and gangway gap, castles, stair, mast, furled sail, shrouds, barrel and crate, and the gangplank as a ramp. Verify: core walker tests up the gangplank, across the deck and up the stair to the aftcastle; the village and walled town re-export byte-identical; shots beside the mockup's `cog` and `castle` views.

## 1. A turning frame

- [ ] 1.1 `LocalFrame` with orientation and angular velocity, composing with the `ω × r` term. Verify: the `world/frames` scenarios as core tests (a point on a turning deck, out and back).

## 2. The cog as a craft

- [ ] 2.1 `Kind::Cog` with its hull, square-sail, keel, rudder and seat specs in `vehicles.ron`, validated. Verify: settings tests refuse a hull cell over a third of the beam, and a sail with no yard.
- [ ] 2.2 The square sail: a foil on a yard braced round the mast. Verify: core tests that the yard's angle of attack follows the braces and the apparent wind, and that a scripted reach, beat and run log speed and heel, with the heel under 15° on the reach.
- [ ] 2.3 The ship piece (`tenebris-towns` 3d) built in the craft's frame, and the craft drawn from the same model the harbour's cog is. Verify: a capture of the cog at its mooring matches the harbour's building cog.

## 3. Walking on the deck

- [ ] 3.1 The ground query asks decks of craft in reach in their frames before the terrain. Verify: an app test that a walker set on the moored deck stands on it.
- [ ] 3.2 The walker aboard keeps a local pose in the ship's frame, composed for drawing and the camera; leaving the deck hands it back to the planet's frame at the composed velocity, and climbing on hands it in. Verify: the `player/walking` scenarios as app tests (standing through a turn, up the stair under way, over the side).

## 4. The helm

- [ ] 4.1 F at the tiller takes and lets go of the helm; W and S brace the yard, A and D steer. Verify: the `player/vehicles` scenarios as app tests (a reach; letting go of the helm), and the binding table lists the helm keys.

## 5. Saves

- [ ] 5.1 The cog's record, and a walker saved on a deck with the craft's ID and local position, at `RECORD_VERSION` 2; a version-1 file reads with none. Verify: format tests, and an app test that quits on the aftcastle and loads there.
- [ ] 5.2 A harbour's building cog becomes the craft at the same mooring (design 6: step 1's piece is never saved, so this is the handover in step 3). Verify: an app test on a world made before this change.

## 6. The owner's check

- [ ] 6.1 A `sail` scenario in `tools/perf_suite.py`, and a note that frame cost was not measured in the cloud session. Verify: the scenario runs.
- [ ] 6.2 The gate video (`step-videos`, showcase `cog`), each shot beside the mockup's cog: up the gangplank; casting off and taking the helm; a reach with the walker then letting go and walking to the aftcastle while the ship sails and turns; over the side and swimming back. Verify: the gate page is linked from the PR.
- [ ] 6.3 The owner watches and accepts. Verify: the quote is in `proposal.md`. Sync the three specs, and archive.
