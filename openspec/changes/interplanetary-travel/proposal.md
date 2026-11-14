# Proposal: travel between worlds on the tiers' ships

Status: **planning only**. Nothing here is built, and no game code changes. The
mockup is `docs/mockups/travel.html` ("Aubade Transit", M2 on the tracker),
reworked on 2026-11-14. Screenshots: `docs/screenshots/mockup-travel/`.

## Why

The owner, 2026-11-14, verbatim:

> Aubade Transit - you just lifted off with a vtol to another planet that makes
> no fucking sense lol

M2 was drafted on 2026-11-12, before the ship tiers (M21,
`openspec/changes/ship-tiers`). In it the Kestrel, the M19 tilt-rotor
aircraft, lifted off a town pad on a free "lift drive", climbed through the air
at 600 m/s with "hover compensation" cancelling any world's gravity, lit a
"transit drive" with a 30 km/s tank at the gravity band's edge, and flew
burn-flip-brake to another planet and docked. That contradicts M21, where
leaving a world takes a rocket of the right tier, stacked from hex cells, with
real propellant, thrust-to-weight and delta-v, staging, a launch from a pad,
orbit, transfers, capture and landing.

M2 had no OpenSpec change of its own. This change is its home.

## What changes

1. **Aircraft fly only in air.** The Kestrel takes the crew to the launch field
   (M21's 2.4 km field) and parks; it never leaves the atmosphere.
2. **Every trip off a world starts with a rocket**: from the launch field at
   tier 1, from orbit and stations at the fusion tier, from any surface at the
   antimatter tier. A trip is a climb to low orbit, a departure burn, a coast, a
   capture, and a landing (by air where there is air, by engine where there is
   none) or a docking.
3. **Delta-v budgets per route**, computed from each body's radius and gravity
   (the game's 25 m/s^2 per g, `mu = 25 g R^2`) and orbits from the planets
   mockup, with M21's parts and stage maths for the craft. The tier ladder of
   where each tier reaches is stated (design 3).
4. **Gas giants have no landing** (the owner's rule, star-systems): orbit, the
   moons, or an aerostat dropped into the airship band on a heat shield.
5. **Airless bodies**: landing is a powered descent, costed like the climb.
6. **Highport moves into orbit** (2,400 m up), since a station that hangs still
   over one place is hover.
7. **Removed from M2**, struck through on the page with the reason: the free
   lift drive, hover compensation, the transit drive and its 30 km/s tank and
   400 m/s reserve, the takeoff table, the four-regimes diagram, the
   dampeners-on coast, in-system lane gates. Kept: destinations, stations,
   star lanes to Tallow and Gemel, travel times (now by tier), fuel and depots.
8. **The mockup's 3D views** (one at a time, in tabs): the launch from the
   field at night, lit by the pad's lamps and the engines, to low orbit; and a
   powered descent onto Lantern's depot at night.

## Why a new change rather than extending ship-tiers

`ship-tiers` is how a ship is built and flown: the VAB, stages, the orbital
field (G4), the planner (G4a), the fusion nav computer, landers. This change is
the trip between worlds that those ships make: routes, budgets, stations,
giants, star lanes, and what the M2 page shows. It depends on ship-tiers and
restates none of its rules; it links to M21 for the flying. Ship-tiers' page
also belongs to another session. Recommendation taken (ask only with
screenshots).

## Capabilities

- **New (delta only, not built)**: `world/interplanetary-travel`, in
  `specs/world/interplanetary-travel/spec.md`. Every requirement is provisional
  on ship-tiers' G4 where it needs the rocket's orbital field.

## Impact

- Docs and mockups only: `docs/mockups/travel.html`, `docs/mockups/travel/`
  (new: `rocket/` with M21's part textures and icons copied byte-identical,
  `game/granite.png`, `game/stone.png`, `game/slate.png`, `icon_aerostat.png`;
  removed: `icon_ship.png`, `icon_dampener.png`), `tools/art/travel_art.py`,
  `docs/screenshots/mockup-travel/`.
- Not touched, on instruction: `ship-tiers.html`, `planets.html`,
  `tracker.html`, `mockup-work.md`. The tracker's M2 summary and
  `mockup-work.md` still describe the transit drive; whoever owns them updates
  them (task 0.6).
- Saves: none. Frame time: none measured (a mockup, rendered on SwiftShader in
  a cloud container with no GPU).
