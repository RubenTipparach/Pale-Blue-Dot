# Tasks: building-planner

Phase 3 (owner, 2026-11-23). Section 1 is the write-up and the mockup, done
now; the rest waits for the owner to tune M6's Planner and Site tabs and ask
for it to be built.

## 1. The write-up and the mockup

- [x] 1.1 This change: proposal, design, tasks and the
      `player/building-planner` spec delta; `openspec validate --all` passes.
- [x] 1.2 M6 Hex House Builder (`docs/mockups/building.html`): a Planner tab
      (presets in three sizes, rooms, walls, doors, windows, stairs,
      lanterns, slots, the roof and kit, the checks in red with their
      reasons, the bill of materials, save as mine) and a Site tab (the 3D
      site at night by its own lamps with a day toggle: stakes and lime
      lines, ghost and built pieces, the count left, scaffolding, building
      piece by piece from a pack, and changing the plan mid-build with the
      goods that come back); the old builder is the Free building tab; one
      3D view at a time, made when its tab opens; the notes in an info
      modal.
- [x] 1.3 Live tunables in the mockup: ghost look, build rate, rotation
      step, preset sizes, scaffolding, removal mode, refund share, cost
      scale, plan limits, and the world's building setting.
- [x] 1.4 Screenshots of every tab at desktop and phone sizes, day and
      night, in `docs/screenshots/building-planner/` (headless Chromium,
      software GL, in a cloud session; no frame times taken).
- [x] 1.5 CLAUDE.md's phase 3 bullet, the tracker's Roadmap step, and M6's
      row in `docs/mockups/mockup-work.md`.
- [ ] 1.6 Published (the owner's session) and the owner's tuning folded
      back into design 8 and 12.

## 2. Core (`pbd-core`)

- [x] 2.1 `building::plan`: plan pieces in plan coordinates (design 1),
      charting a plan onto a patch at an origin, a turn and a base, through
      `blueprint`'s charting; a test that every preset charts at all six
      turns to the same pieces turned.
- [x] 2.2 The planner's checks (design 2): `check_layout` on the plan
      charted onto a flat disc, plus `plan_size`, `storeys` and `way_in`;
      a test per rule with its reason.
- [x] 2.3 The whole-plan check against the world (design 5), first failure
      named; tests for a pentagon, a slope past `max_footing`, a tree, a
      piece already in a slot.
- [x] 2.4 Costs (design 8): `building_costs.ron` read and validated, a
      plan's bill of materials, `cost_scale`; a test that every kit's
      pieces have a cost or are locked.
- [x] 2.5 The site diff (design 7): unchanged, new, gone, marked, kit
      swapped; refunds by `refund_share`; tests for dropping a room and
      adding a storey.
- [x] 2.6 Records (design 9): `blueprint` and `site`, schema 1, through
      `WorldSave::store`, noted in the world identity; built derived from
      piece records; a test that an old save opens unchanged and a site
      survives a restart.
- [x] 2.7 The world setting (design 11): missing reads Free; a test.
- [x] 2.8 Presets (design 4): `blueprints.ron`, three sizes each, and the
      templates' plain buildings; a test that every preset passes the
      planner's checks at every size.

## 3. Game (`pbd-app`)

- [x] 3.1 The drafting table station and the mallet tool, with their
      recipes and icons.
- [x] 3.2 The planner screen (design 3) in the stone-and-wood UI, every word
      from the locale file.
- [x] 3.3 Setting out: the whole ghost, shift-wheel in sixths, the reason
      line; stakes and lime lines painted into the cell tops (`townsfolk`
      39a's bits and texture).
- [x] 3.4 Building: the mallet's strokes on F, the compass tick, the HUD's
      count and bill; each piece a piece record with the pack.
- [x] 3.5 Scaffolding (design 6): poles and board walks, solid to the
      walker, `LitByField` and `LitLikeTerrain`, put up and struck by the
      site. It did not go up in the first capture: the rule wanted every
      wall below built first (design, "The captures' faults"); it now goes
      up once the storey can be worked on, and the capture shows it.
- [x] 3.6 Editing a site and adopting a house (design 7), the change
      summary, marked pieces drawn with the chalk wash. The planner lists
      the pieces added, the pieces removed and the goods back over Save
      change (`construction::change_of`, the same diff Save change applies).
- [x] 3.7 Captures of each step in the game in `docs/screenshots/`.

## 4. Verification

- [x] 4.1 `cargo fmt`, clippy, `cargo test`, the locale lint,
      `openspec validate --all`.
- [ ] 4.2 Frame cost of a site's ghost and scaffolding, measured on the
      owner's desktop (not in a cloud session). Open: built in a cloud
      session with no GPU, so the frame cost was not measured.
- [ ] 4.3 A gameplay demo: a bot plans a cottage from the preset, sets it
      out, gathers the planks, builds it, then adds a room mid-build
      (CLAUDE.md, demos). Open: waits for the batch of videos.

As built (2026-11-24): see design, "As built". Left out of 3.x and said
there: the mallet is carried in the pack (no held swing yet); the
scaffold has no ladder bay, so an upper storey is reached by the house's
own stair; a road cell or a town's footprint is not yet "in the way" (a
solid block, a standing tree, a cactus and a wild plant are); a dropped piece taken down with a
full pack is refused rather than left in a crate; the turnable 3D preview
waits for the build camera.
