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
- [x] Eight species (five from Tenebris with their icons, three new), three
      water zones and a field guide with one entry per species, added to the
      mockup and driven headless.
- [x] Spawning planned from the climate: water classes and temperature
      windows, measured on the real planet by a measurement instrument
      (`examples/fish_ranges.rs`, `tools/fish_ranges.py`), with range maps
      published as the Fish Range Atlas. Found and wrote up the sea's drift.
- [ ] The owner's answers to the eight questions in the proposal.

## 1. Equipment in the core
- [x] `Tool { Rod, Shovel, Pickaxe, Axe }` replaces the `Pick` placeholder;
      `Equipment { owned, held }` beside `Slots`; `Tool::digs` is false for
      the rod only.
- The break-time rule moved to section 3 (`pbd_core::dig`, `dig.ron`).

## 2. Input and HUD
- [x] One G reader (`controls::read_picker_key`). First built as a tap/hold
      split with boarding on the tap; revised by the owner: F boards and
      leaves, R is walk/fly, H returns to the spawn, and G only holds the
      picker open, from the press.
- [x] Tool slot left of the ten; picker column over it; the wheel drives the
      highlight while it is open (`PickerWheel`, the one wheel reader hands it
      over) and is drained either way; closed with `Display::None`. An open
      menu forgets the key, so Escape closes it without equipping.
- [x] The binding table gains "hold G: tools" and "J: the field guide".
- [x] Tests: a tap boards, a hold opens the picker and boards nothing, the
      wheel leaves the item slot alone while the picker is open.

## 3. Digging by tool
- [x] `pbd_core::dig`: the material classes, `right_tool`, `secs`, and the
      `Breaking` state machine (start, hold, reset on release or a new target,
      finish, the pause between blocks); `assets/config/dig.ron` with units
      and a shipped-equals-default test.
- [x] `dig_and_place` becomes a hold on the `Breaking` state; the edit path is
      unchanged; the rod never breaks anything.
- [x] `tools/gen_break_stages.py` writes ten 32 px crack stages with
      `--check`, and a test that each stage contains the one before.
- [x] The crack overlay: a prism over the targeted cell from its record, the
      terrain's UV mapping, unlit and blended, one entity, stage by progress.
- [x] `--break SECONDS` for a capture of the cracks (`docs/screenshots/breaking-cracks.png`, the pickaxe held 1 s on grass: stage 5); `--dig` stays instant.
- [x] Tests: the times by class and tool; the wrong-tool multiple; reset on
      release and on a new target; the rod never; the stage by progress.
- [x] The owner's revision: a matrix of one time per tool for each class
      (dirt, stone, rock, ore, wood, placed) replaces right tool times four;
      a test that each row's fastest tool is the one the design bolds.

## 3b. The tool in hand
- [x] `pbd_core::hexel`: an RGBA image to hexels on the offset hex grid, and
      their mesh (front, back, and sides only where a neighbour is empty),
      with baked directional shade; tests for coverage, closedness and a
      connected diagonal.
- [x] `assets/config/held.ron`: the anchor, the rod's grip and tip, each
      tool's icon grip and head, the twist, sway and swing, validated, with a
      shipped-equals-default test.
- [x] The held tool: one model per tool built from its icon at startup, a
      child of the walking camera, the one in hand shown, the sway, and the
      chop arc while a block is being broken. It replaces the rod's frustum.
- [x] The fishing line leaves from the rod model's tip, read from the same
      data; a test that the two agree.
- [x] A capture of each tool in hand (`docs/screenshots/held-*.png`; the
      pickaxe mid-chop on a cracking block).

## 4. Trees
Built by `tool-crafting` (its task 1.4, decision 3), in a different shape
from this plan; recorded here so the two changes agree.
- [x] The CPU tree rule: `pbd_core::flora::grows_tree(tree_id, word)` with
      `shader_hash` and `TREE_DENSITY`, the foliage pass's rule on the CPU
      (`crates/pbd-core/src/flora.rs`); `planet_lod.rs`
      `tool_crafting_ground` counts trees by it round the spawn.
- [x] The WGSL reads its density table from the uniform (`flora-v2` design
      12.1, built 2026-11-15 with generator version 10): the table is the
      world's (`TerrainConfig::trees`), packed into `plant_means[b].w`, and
      the literals are gone. Agreement is held by
      `flora::tests::the_tree_rule_is_the_foliage_pass_s`: the shader's rule
      lines must stand word for word in both passes, and a Rust mirror of
      those lines agrees with `grows_tree` over 10,000 cell IDs, every biome
      and material, both tables. Not a compute-shader run: the cloud session
      has no GPU to run one on; the launch to READY compiles the pipelines.
- [x] Aim tests the trunk and canopy: `desktop/digging.rs` `tree_on_ray`
      (a trunk column 0.45 m to 3 m, a canopy 1 m to 6.5 m). Wood is a good,
      `Good::LOG`, not a placeable `Material::Wood` (the planks of
      `tool-crafting` are the placeable wood).
- [x] What a tree has given is a stored record, `tree` by the cell's key
      (`gather::TREE_KIND`, `TreeTaken`), written with the slots in one
      `store_with_hands` line; hands take one branch, an axe up to
      `gather.axe_logs` (4). It survives a reload as every record does.
- [x] A spent tree no longer stands (`flora-v2` design 12.5, 2026-11-15):
      `trees_taken` lists the 64 felled trees nearest the camera (a `tree`
      record at `axe_logs`) by tree id in the uniform, and both passes leave
      them out; the list is rebuilt from the records twice a second, so it
      holds after a tier rebuild. Test
      `trees_taken::tests::the_lists_pack_nearest_first_and_a_spent_tree_is_felled`;
      capture with `--take-trees` (`docs/screenshots/flora-v2/`). Limit: a
      felled tree beyond the 64 nearest is drawn again.

## 5. Fauna
- [x] `pbd_core::fauna::School`: packed arrays, the boid terms of design
      section 6, bounds from the sea and ground queries, a seeded goal walk.
- [x] `assets/config/fauna.ron`: the eight species of design section 7 with
      their field-guide entries and tips, water classes and windows; tests for
      entries, icons, disjoint bodies, day-one coverage and empty lifeless
      rosters, and that the shipped file equals the code defaults.
- [x] Spawn gate: water class from the terrain, temperature now from
      `Atmosphere::sample(dir).temperature`, nothing in frozen water.
- [x] `tools/fish_ranges.py` reads its rules from `fauna.ron` (through
      `tools/ron_read.py`), so every species on the roster, the water life
      of `ocean-environments` 7.1 included, gets a range map; the atlas page
      takes each species' icon and entry from the run, and is regenerated
      from the built roster (`docs/wiki/fish-ranges/`, day one and 200 days
      of the shipped settings).
      Regenerated 2026-11-15 with the full roster (1 and 200 days of the
      shipped settings; the sea now holds near 17 C and nothing freezes all
      year, so the old 'sea that freezes' series is gone).
- [x] Bottom dwellers: the goal off the bed and the vertical hold.
- [x] App: spawn ring, caps, despawn, a fixed 30 Hz step. Drawn as one mesh
      per school rebuilt from its arrays rather than an instanced draw per
      species: a handful of schools of at most forty fish did not need one.
- [x] Tests: in the sea and off the bed; deterministic over two runs; no
      spawn on an empty roster.

## 6. Fishing
- [x] `pbd_core::fishing`: the state machine of design section 4, the hook
      window formula, tension, and the weather bite factor from
      `Atmosphere::sample`.
- [x] App: rod model at Tenebris's grip and tip, a line clamped to the sea,
      the float on `LocalSea`, and the cast and tension meter. Digging is
      gated on `Tool::digs`, and right click winds a line in rather than
      placing.
- [x] Tests: the window over strengths 1 to 5; a patient reel lands the fish
      and a held one snaps; the float rides the sea.

## 7. The field guide
- [x] In-game panel on J, opening on the fish in the selected slot: the list,
      the entry with a 96 px nearest-sampled icon, numbers read from the
      species record (`fish::guide_facts`), and the player's catch record.
      A click on a slot does not open it: the pointer is captured while
      playing, so there is no cursor over the slots to click with.
- [x] `tools/gen_fish_wiki.py` writes `docs/wiki/fish.md` from `fauna.ron`:
      one section per species in roster order (all 18, the eight fish and
      `ocean-environments` 7.1's water life), with its icon, kind, origin, entry,
      the field guide's numbers in its formatting, its places, whether it
      takes a line and its tip; `--check` holds the page.
      `fish::tests::the_wiki_page_holds_to_the_roster` (pbd-app) reads the
      committed page back against the roster and the field guide's own
      `guide_facts`.
- [x] Tests: J opens the guide; a catch raises the count and best length, and
      both survive a reload.

## 8. Saves and icons
- [x] `catch` and `hand` lines; the catch record folded on load;
      `f{species},{n}` and `t0..t3` codes; round-trip tests. The tools are not
      a kit grant: a world with no `hand` line is dealt the default
      `Equipment` (all four, rod in hand), which is the same once-only answer
      without a grant to version.
- [x] No `fell` line: `tool-crafting` saves what a tree gave as a `tree`
      record (section 4 above), which the record path already carries.
- [x] Copy Tenebris's five fish PNGs into `assets/items/fish/` with
      `PROVENANCE.md` and recorded hashes; a test checks them.
- [x] `tools/gen_item_icons.py` writes 16x16 PNGs for the tools and the three
      new fish (`--check` holds them); tests that every tool and species has one.

## 9. Prove it
- [x] Capture: `--fish` stands on a warm shore and scripts one cast, logging
      the schools (`docs/screenshots/fishing-cast.png`).
- [ ] Captures: the picker open, the field guide open, a felled tree.
      (2026-11-15, cloud session: the field guide taken with `--menu guide`,
      `docs/screenshots/fishing-and-equipment/field-guide.jpg`, all 18
      species of the water roster in the list; a felled tree's capture is
      `flora-v2`'s `--take-trees` shot. Still open: the picker has no
      capture flag.)
- [x] `openspec validate --all`, fmt, clippy and tests.
- [x] Moved into `openspec/specs`: `world/fauna`, `player/fishing`, the tool
      slot, picker and saved tool change of `player/equipment`, and the tap
      rule of `player/vehicles`. Timed breaking, felling, and the generated
      wiki page stay here.
- [ ] The owner's in-game check of the feel of the bite and reel.

## 10. The line meets the float (owner, 2026-11-16)
- [x] Write up the cause (design 4, "The line's far end"): the line was held
      over the mean sea while the float rides the tide and the waves.
- [x] `draw::line_points`: first point the tip, last point the float, the
      points between over the float's own water; the float and line placed in
      `PostUpdate` after the camera moves.
- [x] Test `the_line_runs_from_the_rod_tip_to_the_float`.
- [ ] The owner's check in game, at low tide, that the line reaches the float.

## 11. The bite is heard and the strike lands (owner, 2026-11-16)
- [x] Measure with `--fish-play` and `--audio-log` (design 4, "No bite
      heard, and no fish"): the catch path lands the fish; the bite cue was
      -16 dB at 18 m.
- [x] `--fish` hands the walker a rod (a new world has none).
- [x] Either button strikes at the bite; `Event::Lost` says a fish got away.
- [x] `bite_splash` full out to the line's reach.
- [x] Tests: `a_right_click_at_the_bite_hooks_the_fish`,
      `winding_in_on_a_nibble_says_the_fish_got_away`,
      `a_bite_struck_in_its_window_puts_the_fish_in_the_slots`.
- [ ] The owner's check in game that the bite is heard and a strike lands.
