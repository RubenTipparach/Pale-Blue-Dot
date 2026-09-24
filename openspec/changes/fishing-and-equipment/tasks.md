# Tasks

## 0. Mockup and write-up
- [x] Three.js prototype: three schools of boids; the rod's charge, cast,
      nibble, bite, hook and tension reel; the tool slot with a hold-G picker
      cycled by the wheel; shovel, pickaxe and axe on sample blocks and a tree
      (`docs/mockups/fishing.html`, published).
- [x] Driven headless end to end: cast, then a school scents the lure, then
      nibble, bite, hook, reel, and the catch lands in the hotbar. Separately,
      a shovel dug dirt. No page errors. Four defects found and fixed in the
      mockup (design section 9).
- [x] Tenebris's fishing, fauna and tool rules measured (design section 1).
- [ ] The owner's answers to the six questions in the proposal.

## 1. Equipment in the core
- [ ] `Tool { Rod, Shovel, Pickaxe, Axe }` replaces the `Pick` placeholder;
      `Equipment { owned, held }` beside `Slots`.
- [ ] `tools.rs`: `right_tool(material)`, `base(material)`,
      `secs(material, tool)`; `assets/config/tools.ron` with units; test that
      the shipped file equals the code defaults.
- [ ] Tests: `secs` for every material and tool; the rod breaks nothing.

## 2. Input and HUD
- [ ] One G arbiter: tap versus hold at the threshold in `tools.ron`;
      `board_or_leave` reads the tap. Update `vehicles/tests.rs::tap` to run
      the release update.
- [ ] Tool slot left of the ten; picker column over it; the wheel drives the
      highlight while it is open and is drained either way; Esc closes it;
      closed with `Display::None`.
- [ ] The binding table gains "hold G: tools".
- [ ] Tests: a tap boards, a hold opens the picker, the wheel leaves the item
      slot alone while the picker is open.

## 3. Digging by tool
- [ ] `dig_and_place` becomes a hold with a `Breaking` state and a reticle
      ring; the edit path is unchanged.
- [ ] The scripted `--dig` capture holds the shovel.

## 4. Trees
- [ ] `pbd_core::flora::tree_at`; the WGSL reads its density table from a
      uniform; a compute-shader test that it agrees over 10,000 cell IDs.
- [ ] Aim tests the trunk and canopy; `Material::Wood` on the (2,1) tile.
- [ ] A felled flag in `Edits`, uploaded with the fine set and honoured by the
      visibility pass; a `fell` save line.
- [ ] Test: felled after a tier rebuild and after a reload.

## 5. Fauna
- [ ] `pbd_core::fauna::School`: packed arrays, the boid terms of design
      section 6, bounds from the sea and ground queries, a seeded goal walk.
- [ ] `assets/config/fauna.ron`: per-body rosters; tests for icons, disjoint
      species and empty lifeless rosters.
- [ ] App: spawn ring, caps, despawn, a fixed 30 Hz step, and one instanced
      draw per species.
- [ ] Tests: in the band and in the sea; deterministic over two runs; no
      spawn on an empty roster.

## 6. Fishing
- [ ] `pbd_core::fishing`: the state machine of design section 4, the hook
      window formula, tension, and the weather bite factor from
      `Atmosphere::sample`.
- [ ] App: rod model at Tenebris's grip and tip, a line clamped to the sea,
      the bobber on `LocalSea`, and the tension meter.
- [ ] Tests: the window over strengths 1 to 5; a patient reel lands the fish
      and a held one snaps; the bobber rides the sea.

## 7. Saves and icons
- [ ] `slots`, `hand` and `fell` lines; `f{species},{n}` and `t0..t3` codes;
      `KIT_GRANTS` widened to grants of tools; round-trip tests.
- [ ] `tools/gen_item_icons.py` writes 16x16 PNGs for the tools and fish, with
      a manifest; tests that every tool and species has one.

## 8. Prove it
- [ ] Captures: the picker open, a school under a floating bobber, a felled
      tree.
- [ ] `openspec validate --all`, fmt, clippy and tests.
- [ ] Move the requirements into `openspec/specs` in the commits that make
      them true.
- [ ] The owner's in-game check of the feel of the bite and reel.
