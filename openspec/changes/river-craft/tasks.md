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
      ropes and a cloth net, anchor holding by bed, set and lift (since
      version 4: a set anchor holds, design 4).
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
- [ ] 3.4 Anchors that set and hold (design 4), with tests that a set
      anchor holds a rowing craft within its rode's swing on every bed, that
      the rode stops at the bed unless a longer rope is set, and that only
      weighing frees it. One anchor per boat (owner, 2026-11-19).
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

## Rowing the raft (owner, 2026-11-19: "mockup better rowing on a raft")

- [x] Write-up: design 9 (today's stroke measured, what reads badly, the
      five stages, one beat, the surge and the turn from the force, the
      splash and swirls, first-person hands, tunables for `vehicles.ron`),
      and the spec delta "A raft is rowed in strokes on one beat".
- [x] Recommendation taken (ask only with screenshots): two oars on a
      rowing frame, rowed facing the bow. (Version 3 also kept a pole for the
      shallows; dropped in version 4 on the owner's word.)
- [x] Rowing tab in M34 (`#rowing`, version 2 of the page): Today / Proposed,
      Oars / Pole, first and third person, hold W S A D Q E to row, Show
      poses, Tune sliders printing the RON block, day and night (a lantern
      on the raft and the bank posts). The hand is M36's approved hand,
      copied into `river-craft/hand.js` from `tool-motion.html`.
- [x] Stills in `docs/screenshots/raft-rowing/` (software renderer): the
      catch, mid-drive with the splash, the recovery, today against
      proposed (day and night, first and third person), the one-oar turn,
      the swirls, the pole.
- [x] The raft's own lantern (owner, 2026-11-19: "Rafts should also have a
      light that comes with it at night"): design 9.8 and the spec delta
      "A raft carries its own lantern, lit at night"; in the Rowing tab the
      lantern lights the deck, water, oars, splash and hands in Proposed and
      nothing in Today; night stills today against proposed.
- [x] Oars only (owner, 2026-11-19: "no pole just oar is best, pole you
      have to actually move, and the pole is clipping, overall too much
      awkwardness"): design 9.2 rewritten, the shallow-water rule (the blade
      rests on the bed and pushes nothing off it; upstream only by rowing
      against the current, recommendation taken, ask only with screenshots),
      the pole's stages, tunables and spec scenario removed.
- [x] Version 4 of the page: the Pole gear, its Too deep chip, key and poses
      removed from the Rowing tab; the Raft tab rows with the same oars
      (`RC.Oarsman`); the stroke smoothed (design 9.10: seat, sweep, forearm,
      swivel locks, a rounded hand loop, no clipping at any stage by the
      clearance check); the rower faces the bow; first-person hands only in
      first person, the body only in third; today's pole no longer through
      the deck. Change-log line in the modal.
- [x] Stills `docs/screenshots/raft-rowing/oars-v4-*.png` (Chromium on
      SwiftShader): catch, drive, release, recovery in first and third
      person, night in both (third person as the owner's shot), today, the
      Raft tab.
- [x] Tube v4 (owner, 2026-11-19: "no light for tube guy, fix arms/body"):
      no lantern, the rider sits back in the ring, paddles with the hands in
      the water; stills `tube-v4-*.png`.
- [x] Anchor v4 (owner, 2026-11-19: "anchor in mockup should stop you from
      moving"): design 4 rewritten, a set anchor holds; the Anchored chip;
      stills `anchor-v4-*.png`.
- [x] Anchor v5 (owner, 2026-11-19: "by default anchor rope should stop
      extending as soon as it touches ground"): the rode stops at the bed.
- [x] The boat's menu, write-up (owner, 2026-11-19: "rope length can be
      controlled in the boat menu, yes boats have menus for inventory and
      various stuff"): design 10 (what the game has today, C to open,
      the hold, the anchor's rope, the lantern, per kind, what saves) and the
      spec deltas "A boat has a menu for its hold, anchor and lantern" and
      "The boat's settings survive saves". Recommendations taken (ask only
      with screenshots): C, one hold size of ten slots, no menu on the tube,
      no boat names or damage, a rode maximum of 40 m in the game.
- [x] One anchor per boat (owner, 2026-11-19: "also only really need one
      anchor per boat"): design 4 and the spec delta; the stern anchor and
      the two-point mooring dropped.
- [x] Version 6 of the page: `river-craft/boatmenu.js`, Boat (C) on the
      Anchor, Raft and Rowing tabs; the Anchor tab's one anchor on T and the
      menu, the rope control in place of Pay out and Shorten; the hold with
      a few things, a tap sends a stack; the lantern check; phone width;
      the stern anchor's mesh, rope, button and HUD line removed; the info
      modal and its change-log line.
- [x] Stills `docs/screenshots/raft-rowing/boat-menu-v6-*.png` (Chromium on
      SwiftShader): the menu open on the Anchor tab, the rope lengthened to
      14 m with the boat dropped back, night, phone width, the Rowing tab's
      raft.
- [ ] The owner's look at the boat's menu (the tracker, M34).
- [ ] The port of the boat's menu (design 10.2 to 10.5), only on a request.
- [ ] The owner's look at the Rowing tab (the tracker, M34).
- [ ] The port (design 9.9), only on a request after the owner's look.

## The port, part one (owner, 2026-11-20: "approve sand and mountains, and rivers")

Approved for the game 2026-11-19 at version 6; asked to be built
2026-11-20. Design 11. On `claude/phase-2-1`.

- [x] Recorded: the M34 row of `docs/mockups/mockup-work.md`, M34's entry in
      `docs/mockups/tracker.html`, this change.
- [x] 9.9 Rowing: `pbd_core::vehicle::oars` (the spec, `oar_pose`, the
      stroke clock), the raft rowed by it in `loon.rs`; tests: the five
      stages in order, the pose continuous, the catch on the catch's end,
      an oar joining at the next reach and resting with no jump, the arms
      reaching the handles through the stroke, the oars mirrored, a blade
      resting on a shallow bed, the keys picking the oars; in the sea: the
      raft rows ahead, one oar turns it away from that oar, a shallow bed
      pulls far less and pushes nothing off it.
- [x] 9.9 The drawing: the rowing frame, oars, oarlocks, the rower facing
      the bow (third person only) and M36's hands on the handles (first
      person only), lit as the raft is; the pole and the standing poler
      removed; the held tool put down while the oars row.
- [x] 3.4 The one anchor (design 4, 11.2): `pbd_core::vehicle::anchor`;
      tests: the rode stops at the bed, a set anchor holds a rowing raft in
      a current within its swing and never moves, the rope veered out and
      hauled back, only weighing frees it; the app's anchoring tests rowed
      through the haul.
- [x] 10.2-10.5 The boat's menu (11.3): C, the boat's panel beside the
      HOLD, the rope and the lantern; `rope_m` and `lantern_out` on the
      craft's record, written only when set; tests: both survive a save and
      an older record reads and writes unchanged, an anchored record comes
      back holding.
- [x] 9.8 The raft's lantern (11.4) as a moving lamp in the carried-light
      list; test: lit only while the dusk lamps are and not put out, none on
      the Loon or the tube.
- [x] The built requirements moved to `openspec/specs/world/river-craft/`
      (rowing, the anchor, the boat's menu, its saved settings, the raft's
      lantern), worded as built; the rest stay in this change's delta.
- [ ] Captures in `docs/screenshots/river-craft-game/` (rowing mid-stroke
      in first and third person, anchored, the boat's menu, the lantern at
      night), beside the mockup's shots.
- [ ] The splash, wash, drips and swirls (9.5).
- [ ] The bed's material deciding how fast an anchor sets.
- [x] The tube rider's pose (the inner tube, version 4; design 11.5): sits
      back, sits up to paddle with the hands beside the ring, no lantern.
- [ ] 3.1-3.3, 3.5, 3.6: the current from the cells, lines between craft,
      the tug and barges, logs and booms, nets.
- [ ] The owner's look in the game; the step's video for the batch.
