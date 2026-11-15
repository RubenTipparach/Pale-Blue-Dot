# Tasks

Planning only. No game code starts until the owner asks for it (CLAUDE.md,
"Write it up before touching code"). The rocket's own work (parts, stages,
the orbital field, the planner, the nav computer, landers) is ship-tiers';
this change only adds what the trip between worlds needs on top of it.

## 0. The write-up and the mockup
- [x] 0.1 The owner's words of 2026-11-14 recorded; M2's draft parts listed
      with what happens to each (design 1).
- [x] 0.2 The budget model and the route table from the bodies' radii, gravity
      and orbits, consistent with M21's numbers (design 2 to 4).
- [x] 0.3 travel.html reworked: the lift drive, hover compensation, transit
      drive, takeoff table, regimes and in-system lanes removed or struck with
      reasons; launch and arrival 3D tabs at night; tier ladder, wells, giants,
      stations, fuel, star lanes, decisions.
- [x] 0.4 Art: M21's part textures and icons copied into `travel/rocket/`,
      settlement textures into `travel/game/`, the aerostat icon
      (`tools/art/travel_art.py`); the runabout and dampener icons removed.
- [x] 0.5 Headless checks and before/after screenshots in
      `docs/screenshots/mockup-travel/` (SwiftShader, stated as such).
- [ ] 0.6 The tracker's M2 summary and `docs/mockups/mockup-work.md` updated
      to the rework, and the page republished (owned by another session).
- [ ] 0.7 GPU screenshots on the owner's desktop, day and night
      (ART-DIRECTION.md's gate).
- [ ] 0.8 The owner approves the mockup for build on the tracker.
- [ ] 0.9 The step's video, for the owner's batch (CLAUDE.md, "Every major
      step ends in a video").

## 1. Travel on top of ship-tiers (after its sections 1 to 3)
- [ ] 1.1 Aircraft are kept out of space: an assisted craft has no way to
      leave the anchor field's band; a test that the Kestrel's ceiling is
      inside the air.
- [ ] 1.2 The launch field as a place: pads, the Kestrel's apron, propellant
      at the pad through the durable save path.
- [ ] 1.3 Landing: heat shield and chutes where a body has air, a powered
      descent where it has none; landed under 6 m/s; a test on an airless
      body that no aerodynamic braking applies.
- [ ] 1.4 Highport as a station on a circular orbit that agrees with
      Tenebris's field (ship-tiers 3.4's versioned body layout).
- [ ] 1.5 Gas giants: an aerostat craft for the airship band, dropped on a
      heat shield; no landing (star-systems' requirement).
- [ ] 1.6 Refuelling at the field, Highport, the Rime depot and the skimmer
      platforms as world mutations through the durable save path.
- [ ] 1.7 The game's own route budgets measured by flying them, replacing the
      mockup's arithmetic.
