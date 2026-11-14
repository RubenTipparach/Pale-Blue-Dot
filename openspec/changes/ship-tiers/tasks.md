# Tasks

Planning only. Nothing below starts until the owner asks for it (CLAUDE.md,
"Write it up before touching code"); G4 decides section 3.

## 0. The write-up and the mockup
- [x] 0.1 Read Tenebris's orbital mechanics (`tenebris-rs/crates/tenebris-core`:
      `orbit.rs`, `nav.rs`, `nav_ui.rs`, `planet.rs`) and summarise it (design 1).
- [x] 0.2 Mockup `docs/mockups/ship-tiers.html`: the hex VAB, tier 1 in orbit
      in both fields, tier 2 courses, freighters and a lander, tier 3 landers.
      (Its starship, warp and star map were taken out on 2026-11-12: "Let's
      not do warp".) Art per `docs/mockups/ART-DIRECTION.md`
      (`tools/art/ship_tiers_art.py`).
- [x] 0.2a The owner's review (2026-11-12): no warp; the assembly grid four
      rings out with crew decks, ladders and hatches inside; more presets.
      The mockup's VAB and tier 3 tab redone to match, and republished.
- [x] 0.3 GPU screenshots of every tab, day and night where there is ground,
      in `docs/screenshots/mockup-ship-tiers/`.
- [x] 0.4 Published, on the tracker, G4 in the survey with tab 2 and tab 3's
      screenshots.
- [x] 0.4a The owner's ask of 2026-11-14 (design 11): the launch field with a
      far LOD ring, Pad B and a walk view; Taper, the 600 m test moon; the
      planner tab with the Aubade system, the integrated line and maneuver
      nodes; Halcyon renamed Aubade. Verified in headless Chromium
      (SwiftShader), screenshots in `docs/screenshots/mockup-ship-tiers/`,
      republished.
- [ ] 0.4b G4a in the survey with the planner's screenshots: may maneuver
      nodes on the integrated line join the invariant? Folded in when answered.
- [ ] 0.5 The owner's answer to G4 folded in; the provisional flight delta
      kept, changed or dropped to match.
- [ ] 0.6 The owner approves the mockup for build on the tracker.
- [ ] 0.7 The step's video, for the owner's batch (CLAUDE.md, "Every major
      step ends in a video").

## 1. Parts and stages (pbd-core)
- [ ] 1.1 A versioned part catalogue in data with units (mass t, propellant t,
      thrust kN, exhaust m/s, where it may go, tier); a saved rocket names its
      catalogue version.
- [ ] 1.2 The stack rules and validation (core, ring, support, nozzles,
      symmetry), a test for each rule.
- [ ] 1.3 Stage maths: membership by decoupler, booster and core phases, TWR,
      burn time, delta-v; a test that the assembly building's delta-v equals
      what a flight spends.

## 2. The assembly building (pbd-app)
- [ ] 2.1 The VAB view: ghost cells, place and remove with symmetry, the stage
      table and checks; the hotbar holds parts.
- [ ] 2.2 Rollout to a pad and a launch: a rocket entity with its stack; spent
      stages break up and are not saved.
- [ ] 2.3 Rockets as stored records (ID, stack, pose in a named frame,
      propellant per stage), migrated forward like every save format.

## 3. Rocket flight (provisional on G4 = B)
- [ ] 3.1 The orbital field in `pbd-core`: inverse-square from every on-rails
      body and the star, one surface constant, beside the anchor field.
- [ ] 3.2 The rocket flight mode: thrust on the nose, no dampeners, SAS hold
      modes; the anchor field stays for assisted ships, hover and walkers.
- [ ] 3.3 The predicted path: the same integration run forward, Ap/Pe/T read
      off it, the HUD line in Tenebris's format and colours. A test that a
      coasting rocket stays on its predicted path.
- [ ] 3.4 Moon rails that agree with the field (Lantern's circular speed at
      26 km), as a versioned body layout.
- [ ] 3.5 The invariant's wording in CLAUDE.md and the flight spec changed in
      the commit that makes 3.1 to 3.3 true, with their tests.
- [ ] 3.6 The prediction's frame cost measured with `tools/perf_suite.py` on
      the desktop.

## 4. Tier 2: fusion
- [ ] 4.1 The hex-cell ship builder (ship-crafting's modules on hex cells) at a
      station yard.
- [ ] 4.2 The nav computer: rail-exact intercept, burn-flip-brake plot, engage.
- [ ] 4.3 Freighter routes as stored records; flown near a player, on rails
      away from one; cargo through the durable save path, authored by the
      freighter.
- [ ] 4.4 Landers: lift to low orbit, dock, refuel in orbit.

## 5. Tier 3: antimatter
- [ ] 5.2 Antimatter landers surface to surface.
