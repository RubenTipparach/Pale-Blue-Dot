## 1. Mockup

- [ ] 1.1 (Made 2026-11-27: M40 version 3, https://claude.ai/artifact/M4Mh9gpMWZYjnsVfnJYrh4#character, with the title menu; U1 and U2 in the survey with its screenshots; waits for the owner's approval.) M40's Character tab: the screen, with every choice from M5's parts and palettes and a Random button; the figure seated in a rowing boat, a hand cart, on a horse, in a car and in a plane, by day and lit at night. Shown as a page of the Options menu. Published, on the tracker, the open questions in the survey with its screenshots (owner, 2026-11-27: "Questions go in the survey for later"). Verify: the owner approves it for build.
  - Made 2026-11-27 (M40 version 3, `docs/mockups/multiplayer.html#character`, as a tab of the Options menu at the owner's word), with screenshots in `docs/screenshots/player-character/`; not yet published, on the tracker or in the survey. Waits for the owner's approval.

## 2. The look

- [ ] 2.1 `Look`'s RON and wire encodings, validated against the parts and palettes. Verify: a round trip of every part, and a look with an unknown part is refused.
- [ ] 2.2 `saves/character.ron`, and the rolled fallback from the install's key. Verify: the same install rolls the same look twice, and a saved look is read back.

## 3. The screen

- [ ] 3.1 The Character page of the Options menu in the UI kit (title and pause menu), offered once on a new world, every word from the locale file. Verify: `--capture` shots by day and at night.

## 4. Seen

- [ ] 4.1 The rower drawn as the player's figure in the chase view. Verify: `--capture` shots beside the mockup's.
- [ ] 4.2 The seats of the carts, the horse and the plodder, the car and the plane, each with its pose. Verify: a shot of each.
- [ ] 4.3 `Hello` carries the look; other players are drawn with it. Verify: a protocol round trip and a two-machine screenshot on the owner's machines.
