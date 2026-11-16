# Tasks

## 1. The mockup

- [x] 1.1 `docs/mockups/river-craft.html` with `river-craft/*.js`: five tabs
      (Raft, Tug, Log drive, Nets, Anchor), one 3D view at a time, a tab's
      renderer made when it first opens, the open tab a bare `#anchor`.
- [x] 1.2 The river: a flow field over a channel of the game's hex cells
      (fast mid-stream, slow at the banks, eddies behind rocks, a narrows,
      a one-layer fall), flow arrows on a toggle.
- [x] 1.3 The physics: fixed 1/120 s step, semi-implicit Euler, float
      points per hull, the game's mooring rule for every line, position-based
      ropes and a cloth net, anchor holding by bed, set and lift.
- [x] 1.4 Play, Step and Speed, paused on load; nothing moves before Play
      or a control except the water; the view turns only when dragged.
- [x] 1.5 Day and Night; at night every boat carries a lantern and a
      lantern post stands on the bank, each lighting what is near it; a dim
      moon.
- [x] 1.6 The stone-and-carved-wood UI kit; keys and an on-screen pad that
      work on a phone; every explanation in the info modal; the tracker link.
- [x] 1.7 Checked in headless Chromium (software renderer): no page errors,
      each tab renders, nothing moves before Play, night is lit. Screenshots
      by day and by night in `docs/screenshots/mockup-river-craft/`.
- [ ] 1.8 Published, and on the mockup tracker as M34 (the owner's session
      publishes and tracks it).

## 2. The owner's look

- [ ] 2.1 The owner plays the mockup and approves it for build, or asks for
      changes; changes go back into 1.
- [ ] 2.2 Open questions, with screenshots, to the survey if any remain
      (design, "Open questions").

## 3. The port (only on a request, after 2.1)

- [ ] 3.1 A current in `Surroundings`: water velocity at a point from the
      flowing water's cells (design 1), with a test that a still pool gives
      zero and a falling level gives flow downhill.
- [ ] 3.2 Drag toward the current at a hull's float points (design 2), and
      a test that a drifting hull reaches the current's speed.
- [ ] 3.3 Lines between craft (design 3), generalising `Mooring`, with a
      test that a slack line pulls nothing and a taut one pulls both ends.
- [ ] 3.4 Anchor holding by bed, set and lift (design 4), with tests for
      sand dragging and mud holding under the same load.
- [ ] 3.5 Raft, tug, barge, log and bundle kinds; the boom (design 5).
- [ ] 3.6 Nets and their catch, beside the rod (design 6).
- [ ] 3.7 Durable records for craft, lines, anchors, booms and bundles;
      old saves open unchanged (design 7).
- [ ] 3.8 In-game captures of each, beside the mockup's shots, and the
      step's video task left open for the batch (CLAUDE.md).

## Inner tube (2026-11-16)

- [x] Tube tab in M34: drift, slow turn from the current's shear, hand
      paddling, get on and off at the bank, a lantern at night; stills
      `docs/screenshots/mockup-river-craft/tube-*.png`.
- [ ] The owner's look at the Tube tab; the recipe (pine pitch, inner tube)
      goes into the port on a request.
