# Tasks

Step 2b of the owner's plan. It starts after `world-map` has its approved
mockup and `bigger-biomes` has its chosen scale, because a site's kind reads
the biome.

## 1. The rules on the mockup, for the owner's approval

- [ ] 1.1 The site rules in the `world-map` instrument: screen, score, greedy pass and full check, on the shipped seed at the chosen biome scale. It writes the list as JSON and prints the time each stage took. Verify: the times go into this design's risk note, and the JSON is committed next to the mockup's rasters.
- [ ] 1.2 The mockup's sites layer drawn from that JSON: a marker for each kind, names, and footprint outlines at close zoom. Verify: a headless screenshot at world zoom and at a close zoom.
- [ ] 1.3 The mockup's site editor: drag, add, strike, rename, and "copy as RON". Verify: a headless test drags a site, and the copied RON holds its new anchor.
- [ ] 1.3a A walkthrough video of the mockup's sites: world zoom, each kind close in, and the editor dragging a pin and copying the RON. Verify: it is on the gate page.
- [ ] 1.4 **Gate:** the owner watches the video, approves the sites on the map, and answers the open questions. Record their words in `proposal.md`, and any pins they placed in `assets/config/sites.ron`'s first draft. Verify: the quote is present.

## 2. The site list in the core

- [ ] 2.1 `assets/config/sites.ron` and its validated struct: per-kind radius, flatness, spacing and count in metres, the name tables, and the overrides. Verify: tests that a zero spacing, an unknown kind and a pin at sea are each refused with the field named.
- [ ] 2.2 `pbd_core::sites::generate(cfg, terrain, overrides)` with the screen, score, greedy pass and full check. Verify: the spec's tests (identical on one thread and many; fewer towns keep the same sites; flat, dry ground; no pentagons; a harbour on the water; a cliff village climbs; a cave town has rock over it; the kind matches the biome; every kind occurs; spacing on every pair).
- [ ] 2.2a The small town near the spawn and the capital elsewhere (survey C2, C3). Verify: a test on the shipped seed that a village or small walled town is within 500 m of the spawn and the capital is on another land mass.
- [ ] 2.3 Names from the tables, six peoples with the mountain people's (survey T12). Verify: tests for unique names on the shipped seed, a pinned name kept and reserved, and a name unchanged across two runs.
- [ ] 2.4 The same list from the instrument and the core. Verify: the instrument calls `generate`, and its JSON matches the core test's list byte for byte.

## 3. The world keeps its list

- [ ] 3.1 The list stored as `site` records (`world-persistence` group 3, which lands first), generated for the whole planet at creation or on an older world's first open, written through the durable path before any site is shown, and read after. Verify: format tests for the round trip and an old save gaining its list; an app test that a changed `sites.ron` does not change an opened world's list.

## 4. On the map

- [ ] 4.1 The sites marker layer in the `world-map` registry, with label culling and footprint outlines. Verify: an app test that every site has a marker at its anchor's projection, and a capture at world and close zoom in `docs/screenshots/city-sites/`.
- [ ] 4.2 Sync `world/sites` and the `player/map` addition into `openspec/specs`, naming each test. Verify: `openspec validate --all`.
- [ ] 4.2a The gate video (showcase `sites`): the in-game sites layer at world and close zoom, then a flight to three sites of different kinds, showing the flat, dry ground each stands on. Verify: the gate page is linked from the PR.
- [ ] 4.3 The owner watches the video and accepts the in-game map against the approved mockup. Verify: the quote is in `proposal.md`. Archive.
