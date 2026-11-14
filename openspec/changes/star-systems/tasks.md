# Tasks

Planning only. No game code starts until the owner asks for it (CLAUDE.md,
"Write it up before touching code").

## 0. The write-up and the mockup
- [x] 0.1 Survey Tenebris's bodies, textures, flora and fauna (design 1).
- [x] 0.2 Rename Halcyon to Aubade across the mockups, the art script, the
      tracker, the mockup list and the tracker skill (not ship-tiers.html or
      ocean.html, on instruction).
- [x] 0.3 Copy Tenebris's textures byte-identical with a README
      (`docs/mockups/planets/tenebris/`).
- [x] 0.4 Pixel art for the new bodies, species and resources
      (`tools/art/star_systems_art.py`), two passes against a contact sheet.
- [x] 0.5 planets.html: star map, Nocturne, Tallow and Gemel tabs, the
      Tenebris worlds tab, every new body in the deck, globe, vignette and
      ladder; one WebGL context for the page. travel.html: star lanes.
- [x] 0.6 Headless checks and screenshots (`docs/screenshots/mockup-planets/`),
      SwiftShader, stated as such.
- [ ] 0.7 GPU screenshots on the desktop, day and night (ART-DIRECTION gate).
- [ ] 0.8 Republish M1 and M2; the tracker's entries updated.
- [ ] 0.9 The owner's review on the tracker; S1 to S8 confirmed or overturned
      and folded back here.
- [ ] 0.10 The step's video, for the owner's batch.

## 1. Systems and frames (pbd-core), when asked
- [ ] 1.1 A system table in data (star, light, frame) and a system id in every
      body id and world id; the home save keeps working unchanged.
- [ ] 1.2 Bodies of every system on the ladder; a test per system that every
      radius sits on `R = 300 m * 2^(L-7)`.

## 2. Ported worlds, when asked
- [ ] 2.1 Each ported world a new generator id at v1 (Sequoia, Tenebris, Crag,
      Frostbite, the moons, the belt), its digest test taken before anything
      ships; Pale Blue's TENEBRIS v5 untouched.
- [ ] 2.2 Sequoia's blocks, biomes, trees and roster per design 1, in the
      game's atlas (Tenebris's 16 px textures as sources, with provenance).

## 3. Star lanes, when asked
- [ ] 3.1 Lane gates per travel.html first; star lanes after, with the seed run
      and the frame rebase at a tick boundary.
