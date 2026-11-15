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
- [x] 0.5 planets.html: star map, Tallow and Gemel tabs, the Tenebris and
      Sequoia tab, every new body in the deck, globe, vignette and ladder; one
      WebGL context for the page. travel.html: star lanes.
- [x] 0.6 Headless checks and screenshots (`docs/screenshots/mockup-planets/`),
      SwiftShader, stated as such.
- [x] 0.6a The owner's correction, 2026-11-14 ("pale blue = tenebris, so call
      it that, sequoia is the sister planet"): the home world is Tenebris in
      planets.html, travel.html and industry.html (not ship-tiers.html or
      ocean.html, on instruction); Sequoia and Tenebris's other bodies moved
      into Aubade (design 3); the duplicate Tenebris removed; Nocturne retired;
      the radius rule redone at sixteen times (design 2); travel.html's
      destinations and star lanes, the tracker's M1 and M2 summaries updated.
- [x] 0.6b Gas giants, the owner, 2026-11-14 (no landing; an airship band; the
      limit where every craft is destroyed, well above the hydrogen sea): the
      Gas giants tab, its descent view and profile, the hull gauge and the lost
      state; giants out of the Surface picker (design 6).
- [x] 0.6c A giant and its moons drawn at their own scale, moons clear of the
      rings (the owner on a screenshot, 2026-11-14; design 7).
- [x] 0.6d Screenshots redone (SwiftShader): Aubade with Tenebris and Sequoia
      side by side, day and night; the giants' descent by day and night, the
      hull warning and the lost state; Amber's moons clear of its rings;
      Nocturne's and the removed bodies' shots deleted.
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
- [ ] 2.1 Each body from Tenebris a new generator id at v1 (Sequoia, Crag,
      Frostbite, Selene, Sprout, Quartz, Hoar, the ten belt rocks), its digest
      test taken before anything ships; the home world's TENEBRIS v5
      untouched (it is Tenebris already).
- [ ] 2.2 Sequoia's blocks, biomes, trees and roster per design 1, in the
      game's atlas (Tenebris's 16 px textures as sources, with provenance).

## 3. Gas giants, when asked
- [ ] 3.0 Giants as bodies with no cells: the floating platform, the airship
      and its band, the limit depth that destroys any craft, per giant data
      (design 6), with a test that a craft below the limit is destroyed.

## 4. Star lanes, when asked
- [ ] 4.1 Lane gates per travel.html first; star lanes after, with the seed run
      and the frame rebase at a tick boundary.
