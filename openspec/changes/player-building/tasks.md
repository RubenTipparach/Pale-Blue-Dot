# Tasks

Step 3 of the owner's plan. The mockup (group 1) can be made once the owner
has approved the towns' pieces. The Rust starts after `cities-in-the-world`
has built them into the engine.

## 1. The build mockup, for the owner's approval

- [ ] 1.1 `docs/mockups/build.html` from the towns mockup's pieces, kits, walker and checks. It has build mode, the palette, the aim-snapped ghost with its reason, the roof tool and blueprints, on an empty plot and beside the walled town, with day and night. Verify: a headless test builds a two-storey house piece by piece and walks up its stair with no eye jump over 0.1 m, and a refused placement shows its reason.
- [ ] 1.2 Publish it and link it from the roadmap page and the PR. Verify: the link opens.
- [ ] 1.2a A walkthrough video of the build mockup: a two-storey house built piece by piece, a refused placement and its reason, the roof tool, and a blueprint. Verify: it is on the gate page.
- [ ] 1.3 **Gate:** the owner watches the video, builds with it, approves the feel, and answers the open questions. Record the words in `proposal.md`. Verify: the quote is present.

## 2. One set of checks

- [ ] 2.1 Split `settlement::check` into per-rule functions over the pieces within reach, and have the layout check call them. Verify: `tenebris-towns`' layout tests pass unchanged.
- [ ] 2.2 `building::can_place(pieces, candidate)` returns the first rule that fails. Verify: a test builds every template both whole and piece by piece, and both are accepted; and one test per rule is refused with that rule's name.

## 3. Piece edits

- [ ] 3.1 Add and remove piece edits in the edit log, through the durable path, with the format version bumped. Verify: format tests for the round trip and for an old save reading unchanged; an app test that a built room survives a reload.
- [ ] 3.2 Refused removals name their dependant. Verify: a test removing a floor under a table.

## 4. Build mode

- [ ] 4.1 B on foot opens and closes build mode, the hotbar becomes the kit's palette, and leaving restores it. B is added to `BINDINGS` for on foot. Verify: app tests that the hotbar and selection come back unchanged, and that B in a ship still brakes.
- [ ] 4.2 The aim-snapped ghost, its colour and its reason text. The wheel steps openings and stair rotations. Verify: `--capture` shots of a green and a red ghost with its reason, in `docs/screenshots/player-building/`.
- [ ] 4.3 The roof tool's flood fill and fit. Verify: a test roofing a two-by-four house, and a capture.
- [ ] 4.4 The blueprint picker and fill, both at once and piece by piece. Verify: an app test places a cottage blueprint, fills it, and walks in through its door.

## 5. The owner's check

- [ ] 5.1 Record that frame cost was not measured in the cloud session, and time `can_place` in a core test on the walled town's densest street. Verify: the time is in the risk note, and the note is in the PR.
- [ ] 5.1a The gate video (showcase `building`). Its shots are:
  - build mode opened;
  - a two-storey house built piece by piece;
  - a refused placement with its reason;
  - the roof tool;
  - a blueprint filled;
  - walking up the new stair;
  - a quit and reload showing the house piece for piece.

  Verify: the gate page is linked from the PR.
- [ ] 5.2 The owner watches the video, builds in the game, and accepts. Verify: the quote is in `proposal.md`. Sync `player/building`, and archive.
