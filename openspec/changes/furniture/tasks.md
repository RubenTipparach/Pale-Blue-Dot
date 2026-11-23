# Tasks: furniture

## 1. The write-up and the mockup

- [x] 1.1 This change: proposal, design, the `player/furniture` and
      `world/town-furniture` deltas, tasks; the Carpentry branch's delta in
      `town-missions-tree`; `openspec validate --all` passes.
- [ ] 1.2 M15's Furniture scene (`towns.html#furniture`): every piece close
      up, a furnished room by day and at night by its lamps, the player
      seated, a townsperson in bed, the chest panel; screenshots in
      `docs/screenshots/furniture/` (Chromium, SwiftShader).
- [ ] 1.3 M33's Town tab gains the Carpentry branch, its icons in the tab's
      style; screenshots.
- [ ] 1.4 The tracker's M15 and M33 entries and `docs/mockups/mockup-work.md`.
- [ ] 1.5 Published to M15's and M33's links (the parent session).
- [ ] 1.6 The owner's review.

## 2. The core

- [ ] 2.1 `pbd_core::furniture`: the kinds, footprints, models and uses.
      Verify: every piece fits against every side of the standard cell.
- [ ] 2.2 The occupancy rule. Verify: a held place refuses another body, a
      body holds one place, a bench holds two.
- [ ] 2.3 The `store` record and its moves; loot by trade. Verify: a round
      trip through the record, a move both ways, loot fixed by its seed and
      fitting the trade.
- [ ] 2.4 `Shape::Furniture` and its rules (in a wall, overlap, doorway,
      headroom, clear). Verify: each refused with its rule; an old `piece`
      record reads as before.
- [ ] 2.5 `settlement::pieces::furnish`. Verify: every house furnished, no
      piece in a doorway or over another, nothing else in the cut moves.
- [ ] 2.6 The carpenter's bench material, its station and the goods.

## 3. The game

- [ ] 3.1 The carpenter's panel and its recipes (`recipes.ron`).
- [ ] 3.2 The Furniture tool in build mode.
- [ ] 3.3 Sit and lie down for the player.
- [ ] 3.4 Townsfolk in seats by day and beds by night.
- [ ] 3.5 The storage panel and its durable lines.
- [ ] 3.6 Furnished town houses with loot to take.
- [ ] 3.7 The Carpentry branch's events.
- [ ] 3.8 Captures in `docs/screenshots/furniture/` (lavapipe).

## 4. Later

- [ ] 4.1 Stealing (design 10), on the owner's word.
- [ ] 4.2 Sleeping through the night, with survival's rules.
- [ ] 4.3 Townsfolk walking to their seat or bed through the door.
- [ ] 4.4 The furniture's video, with the batch (CLAUDE.md).
