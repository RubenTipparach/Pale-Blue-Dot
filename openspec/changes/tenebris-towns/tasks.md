# Tasks

## 0. Mockup (done, awaiting the owner's verdict)
- [x] Three.js town on the engine's cell and layer: an inn, twelve houses, a
      smithy, a moot hall, a keep, two wall towers, a quay, a market and
      townsfolk, every room enterable (`docs/mockups/towns.html`, published).
- [x] The three stairs, doors, thin-solid collision and the three walker rules,
      with a switch to today's `walking.rs` rules on the same stairs.
- [x] Scripted walks in both walkers (design section 5); two layout bugs found
      and fixed in the design (an eave over a tower, a landing that stole
      headroom).
- [ ] The owner's verdict, and answers to the six questions in the proposal.

## 1. Contact (`pbd-core`, `planet_contact.rs`)
- [ ] Thin solids: a convex outline, a height range and a step class, indexed
      by the cell they belong to.
- [ ] Surfaces: stair and roof tops as functions of position, the newel
      answering one sheet per turn; the straight flight, newel and street-step
      heights in one place both the walker and the tests read.
- [ ] `stand` folds solids and surfaces in with the column runs.
- [ ] Tests: a point on each stair returns its pitch line; a point under a
      newel's landing returns the landing as its ceiling; a point inside a
      wall is not a floor.

## 2. The walker (`walking.rs`)
- [ ] The body is pushed out of thin solids and too-tall column faces along
      the face, keeping the tangential motion; headroom stays a hard stop.
- [ ] Hold a grounded walker to a floor up to 0.35 m below.
- [ ] Tests mirroring design section 5: up and down each stair with no eye
      jump over 0.1 m and no airborne tick; a shallow wall brush slides; a
      closed door stops; a table is not stepped onto; a jump indoors stops at
      the boards; a 1 m terrace is still a fall.

## 3. Pieces (`pbd-core::settlement`)
- [ ] Walls on edges with openings, posts at corners, floors, the three
      stairs, gable and pyramid roofs, all from each cell's real corners.
- [ ] Layout checks: edge ownership, newel exits on their edges, stair foot
      and landing cells, headroom at every walkable point, no pentagons.
- [ ] A town layout as versioned data, part of the saved world ID.

## 4. Drawing
- [ ] Piece meshes with the pixel textures (stone, rubble, half-timber,
      plaster, planks, thatch, shingle, slate), nearest magnification.
- [ ] Interior sky light from the voxel-light skylight; hearths, forge and
      lamps as block light.

## 5. Doors and saves
- [ ] E opens and closes the door in reach; the state change goes through the
      durable transaction path.
- [ ] Test: open a door, reload, it is open.
