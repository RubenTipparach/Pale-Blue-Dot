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
- [x] Second round, on the owner's review: roofs of neighbouring houses
      overlapped (now checked at layout, 34 roofs, none overlap); the phone
      stick was cancelled by the page's own touch gestures (now touch-action
      none, tested with touch events: 3.66 m walked); eleven kits and a hamlet
      of huts added, and every hut and kit house walked into through its door.
- [x] Third round: a settlement per biome (village, desert, tundra, jungle,
      swamp) from each biome's catalogue materials, with domes, decks, rope
      bridges, boardwalks and outdoor stairs; every new route walked
      (design section 10).
- [x] Fourth round: a fishing harbour with piers, finger piers, 23 moored
      boats, a boardable cog, boathouses, a shipyard, fish huts on stilts and
      a light; driftwood and whitewash kits; every route walked, including up
      the gangplank and onto the aftcastle (design section 10, the harbour).
- [x] Fifth round, on the owner's word that every mockup is lit at night:
      block light baked from every lamp and fire, kept to its room; a lantern
      by every house door, street lanterns, and each settlement's own torches,
      braziers and hearths (design section 7); every walk rerun unchanged.
- [x] Sixth round: street lanterns on the house fronts, densest in the
      walled town; about half the windows lit, with a candle in the room and
      light on the street; small lanterns on the jungle's rope bridges; every
      walk rerun unchanged (design section 7).
- [x] Seventh round, on the owner's review: window sills flickered against
      the wall below. Every coplanar overlap was scanned and fixed (design
      section 11). The overview's floating hint is now part of the controls
      list. Day and Night buttons and the L key were added. Overview panning
      (right-drag, WASD, two fingers) now follows the camera's heading, and
      drag grabs the ground.
- [x] Eighth round: furniture pushed clear of walls, doors, posts and hearths
      at placement; 55 of 342 pieces were in a wall, now none (design section
      11).
- [x] The owner's answers to the proposal's questions (survey, 2026-09-27),
      recorded in the proposal.
- [x] Ninth round, on the owner's word (T6): mound houses dug into a hill
      under turf, a cliff village on five terraces with rooms cut into the
      rock, and a town in a chamber inside a mountain, each from its own
      biome's materials and lit at night (the cave lit only by its own fires,
      day and night). All three load with no page errors and no roof
      overlaps. Seven new routes were walked with no airborne tick and no eye
      jump over 0.1 m, and every earlier walk reruns unchanged (design
      section 10, the ninth round).
- [ ] A parity list of everything in the mockup and where the game builds it
      (`docs/plans/towns-mockup-parity.md`). Verify: every row names a change
      and a task, or an exclusion the owner has agreed to.

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
- [ ] Speeds, sprint and crouch (the owner, T1, T8, T8b and T8c): run at
      5 m/s by default; Caps Lock switches between run and walk at 3 m/s;
      Shift sprints at 8 m/s over either; Ctrl crouches at 1 m/s with the
      body 1.2 m tall; and under a roof the walker walks unless Shift is
      held. Verify: a test of each gait on flat ground, of Caps Lock toggling
      and Shift over both, of the walk under a roof and the gait restored
      outside, and that a crouched walker passes under a 1.5 m beam a
      standing one cannot and stays crouched under it when Ctrl is let go.
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

## 3b. Kits
- [ ] Kits as data (`assets/config/kits.ron`): wall faces per storey, posts,
      roof kind and material, gable, floor, door and window sizes, the hut
      storey. Validated: a kit never changes the cut.
- [ ] Roof kinds: gable over the footprint's box, six-sided cone, flat with a
      parapet; each registered for the overlap check.

## 3c. Settlements per biome
- [ ] Settlement templates as data per biome, with the biome's kits and plants
      and nothing from another biome.
- [ ] Pieces: dome, deck on piles, walkway between two points (with sag),
      outdoor stair between two points; each answering `stand` and rails.
- [ ] Tests: across a sagging bridge grounded every tick; up a porch stair
      with no eye jump; into an igloo through its tunnel.

## 3d. The harbour
- [ ] Boat hulls lofted from sections as a mesh piece (drawn), each with a
      fixed hull solid up to its gunwale that does not follow the bobbing.
- [ ] A ship as a piece: its deck a step solid following the hull's plan, its
      rail with a gangway, castles and their stair; a gangplank as a sloped
      walkway from the pier to the deck's edge.
- [ ] Tests: up the gangplank onto the deck and up to the aftcastle with no
      eye jump over 0.1 m and no airborne tick; wading at a moored boat stops
      at its side.

## 4. Drawing
- [ ] Piece meshes with the pixel textures (stone, rubble, half-timber,
      plaster, planks, thatch, shingle, slate, three bricks, marble, clay,
      cob, reed, boards, turf, fieldstone, clay tile, driftwood, whitewash,
      ivory sand, and a net with transparency), nearest magnification.
- [ ] Interior sky light from the voxel-light skylight; hearths, forge,
      lamps, torches and braziers as block light, the night lights lit after
      dusk.
- [ ] Each settlement template carries its lights: a lantern by every house
      door that opens outdoors, street lanterns at a spacing set by its size,
      lit windows, lanterns on its bridges, and its biome's own.
- [ ] Tests: a hearth lights its room and not the outside of its wall; a lamp
      lights its street at any camera distance.
- [ ] Furniture placed with the least move clear of walls, posts, doors and
      hearths. Test: no furniture overlaps them in any settlement template.
- [ ] Trim built proud of what it meets (sills, thresholds, floors over wall
      tops). Test: no two visible faces of different pieces share a plane over
      the same area in a built settlement.

- [ ] Moving parts, as the mockup has them: windmill sails that turn, moored
      boats that bob on the swell while their collision stays fixed, and
      hearths, torches and braziers whose flames flicker. Verify: captures
      over a few seconds show each moving, and the boat's collision test
      passes while it bobs.
- [ ] The mockup's inspection tools as debug keys in the game: a collision
      view (C in the mockup), flying through walls (N), and a readout of
      feet height, cell, headroom, speed and eye jumps. Not the T key, which
      swaps in today's walker: the owner dropped it (T11), since the
      side-by-side videos show the difference. Verify: each is
      listed in the controls under debug, and a capture of the collision view
      in the walled town.

## 5. Doors and saves
- [ ] E opens and closes the door in reach; the state change goes through the
      durable transaction path.
- [ ] Test: open a door, reload, it is open.

## 6. The owner's walk-through notes (design section 12)
- [ ] 6.1 (Built 2026-09-30, awaiting the owner's look: `tools/mockup_towns_checks.js`, `docs/screenshots/tenebris-towns-notes/`.) Mockup: hulls mask the water inside their waterline; faces take the polygon's own normal and drop repeated corners; every window has double-sided glass and none glows; chimneys start above the soffit and the coplanar scan checks downward faces; doors swing outward where inward is blocked; jungle bridges end on their edge's corners; sconces in every stairwell; the igloo's tunnel is one arched shell joined to the dome. Verify: before-and-after shots of each in `docs/screenshots/tenebris-towns-notes/`, the page's checks (no face with a zero normal; the coplanar scan both ways; doors turned outward, per settlement; every step cell covered and every top face facing up), and the owner's look.
- [ ] 6.1b (Built 2026-09-30, awaiting the owner's look: `docs/screenshots/tenebris-towns-notes/12b-*.jpg`.) The owner's second look (section 12b): each rope bridge ends on its own platform edge; hull masks lie on the water, cut from the hull each frame; the igloo's dome is cut exactly to the tunnel, and the brick courses are continuous. Verify: `tools/mockup_towns_checks.js` reads no shared bridge ends, no rails over a deck, no water in a hull at any pose or view, and no ray out of the igloo but by its mouth; and the owner's look.
- [ ] 6.2 The game's cutter takes the render-only fixes (section 12, "For the engine"): the polygon's own normal, the chimney above the soffit, window glass. Verify: `settlement::tests` pin no face dropped for a zero normal and no downward face in the soffit's plane.
