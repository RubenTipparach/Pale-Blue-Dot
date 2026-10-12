# Tasks

Step 2e, after `cities-in-the-world`, whose settlements they live in, and
before `player-building`. It exists so that every feature of the approved
towns mockup is on the roadmap (the owner, 2026-09-27).

## 1. Who and where

- [ ] 1.1 Roles, posts and paths in each settlement template, ported from the mockup's `person()` calls. Verify: a test that the walled town has 16 people, with a smith and a gate guard among them.
- [ ] 1.2 `pbd_core::townsfolk` derives each settlement's people from its record and the seed. Verify: tests that the same record gives the same people twice, and that no post or path crosses a wall.

## 2. How they look and move

- [ ] 2.1 The hexagonal-prism figure in the terrain's pixel style (the owner, T2), instanced. Verify: a `--capture` a metre from a person showing six-sided parts, in `docs/screenshots/townsfolk/`.
- [ ] 2.2 The walk swing, the work loops (hammer, wipe) and the idle look, as the mockup animates them. Verify: a capture sequence of each.

## 3. Solid and lit

- [ ] 3.1 Each person's circle in the settlement's moving-solid list, checked by the walker. Verify: an app test that walking straight into a working smith slides round them with no overlapping tick.
- [ ] 3.2 Lighting through the field's sampler. Verify: a capture of the smith at the forge at 22:30.

## 4. The owner's check

- [ ] 4.1 The gate video, with the prototype beside it (`step-videos`): each settlement's people shot by shot, a walk into a crowd, and the forge at night, each beside the same shot in the towns mockup. Verify: the gate page is linked from the PR, with a note that frame cost was not measured in the cloud session.
- [ ] 4.2 The owner watches and accepts. Verify: the quote is in `proposal.md`. Sync `world/townsfolk`, and archive.
