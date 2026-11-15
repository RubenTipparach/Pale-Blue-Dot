# Design: travel between worlds

Everything here is proposed. Numbers are the mockup's (computed in the page,
`docs/mockups/travel.html`) and are recommendations to tune, not measurements
of the game. Where the rocket flies in an inverse-square field, the design is
provisional on ship-tiers' survey question G4 (recommendation B), and where it
mentions nodes, on G4a.

## 0. The owner, 2026-11-14

> Aubade Transit - you just lifted off with a vtol to another planet that makes
> no fucking sense lol

Read as: interplanetary travel must use M21's ships. An aircraft (the Kestrel,
M19/M24) flies only in air; it takes you to a spaceport. Leaving a world takes
a rocket of the right tier with real propellant and delta-v.

## 1. What M2 had, and what happens to each part

| 2026-11-12 draft | Now | Why |
| --- | --- | --- |
| Kestrel lifts off a town pad on a free lift drive, 600 m/s through the air | The Kestrel flies to the launch field and parks | An aircraft flies in air only (M21 section 8) |
| Hover compensation cancels any world's pull | Gone; a rocket's engines beat its weight or it stays on the pad | Thrust-to-weight is the point of tier 1 (M21) |
| Transit drive: 60 m/s^2 to 4 km/s, 30 km/s tank, 400 m/s reserve | Gone; delta-v is what a stack carries. Burn-flip-brake survives as the fusion nav computer's course at 20 m/s^2 | M21's stage maths; M21 section 5 |
| Takeoff table (time to space with hover) | "Leaving a world": low orbit, circular and escape speed, the climb's delta-v per body | Hover is gone |
| Four regimes (air, band, frame, space) | Gone; walkers and aircraft keep the anchor band, rockets the orbital field | M21 recommendation B |
| Dampeners on/off coast | Gone for rockets (no dampeners, M21 section 4) | |
| Frame sphere per body (T5) | The frame of the body the rocket is round, rebased at a tick | M21 design 11.3 |
| Highport fixed 12 km over the town | Highport in a circular orbit 2,400 m up | A station must orbit |
| In-system lane gates to Glaucus (T4) | Dropped | Fusion and antimatter are the fast way round; a gate would skip the tiers |
| Star lanes to Tallow and Gemel (T9) | Kept | star-systems design 4 |
| Destinations, stations, depots, fuel | Kept, recast by tier | |

## 2. The model

- **Pull**: `mu = 25 g R^2` (gravity-model's surface constant), inverse-square
  for rockets (M21 B, provisional on G4). Walkers, the Kestrel and every
  assisted craft keep the bounded anchor field (full to 1.4 R, zero by 1.8 R),
  as the flight spec pins today.
- **Low orbit**: 1.2 R over a body of 2,400 m or more (Sequoia 900 m up, as M21's planner), 1.25 R over a smaller
  one, 150 m up over the 300 m bodies (as M21's Taper). Tenebris: 960 m up,
  316 m/s (M21's).
- **The climb** to low orbit (and a powered descent back down):
  `v_circ (1 + 0.7 g)`, fitted to M21's two flown numbers: 535 m/s from
  Tenebris (the formula gives 538) and 110 m/s onto Lantern (114).
- **Landing**: by air where there is air (heat shield, chutes, 50 m/s to touch
  down); Ochre's thin air takes 40% of a powered descent; airless, a powered
  descent. Coming home to Tenebris is 50 m/s: the pod's heat shield.
- **Between planets** the star barely pulls (Aubade at Tenebris:
  1.4e-4 m/s^2; planets ride rails at 3 to 7 m/s; M21 11.3). A trip's cost is
  leaving one well and falling into the next: from low orbit
  `sqrt(v_inf^2 + v_esc^2) - v_circ` each end, coasting nearly straight at
  `v_inf`. The tables use **300 m/s**, M21's own example (Tenebris to Verdance:
  434 m/s between low orbits, 11 minutes): the formula gives 222 + 212 = 434.
- **The moons of Tenebris**: a Hohmann arc round Tenebris from low orbit
  (Lantern: +88 out, +51 capture, 4 min 22 s, as M21).
- **A moon of another planet**: arrive at `v_inf`, fall to the moon's orbit,
  best case tangent and prograde, capture into the moon's low orbit.
- **A giant**: capture into a low orbit over the cloud tops, or an aerostat
  dropped on a heat shield (entry at `sqrt(v_inf^2 + 2 mu / R)`).
- **Fusion** (M21): burn-flip-brake at 20 m/s^2 from Highport over the centre
  distance less Highport's low-orbit radius, peak capped at 3 km/s here (M21
  gives Lantern 1:04 and 1.27 km/s, Verdance at 200 km 3:20 and 4.0 km/s; the
  formula gives both). A lander goes up and down at each end.
- **Antimatter** (M21): surface to surface at 31.6 m/s^2 (M21's Tenebris to
  Cinder, 5:10 and 9.8 km/s, over about 760 km).
- **Distances** over the year from the planets mockup's rails (same ids, same
  phases); rocket times are the coast plus 2.5 minutes for the climb, burns and
  landing.

This is budget arithmetic for the write-up and the page's tables, the way
M21 states its Hohmann numbers. It is not a second model of motion in the game:
the ship is flown by the integrator, and the planner's line is that integrator
run forward (M21 11.3). No patched-conic solver is proposed.

## 3. The craft (M21's parts and stage maths)

| Craft | Stack | Mass | TWR at 1 g | Delta-v |
| --- | --- | ---: | ---: | ---: |
| Chemical | M21's First orbit with a chemical engine in place of the nuclear one (before fission fuel rods) | 51.5 t | 1.80 | 1,409 m/s |
| Nuclear thermal | M21's First orbit preset | 53.1 t | 1.75 | 2,122 m/s |
| Fusion | Fusion drive, three tanks, pod (M21's 20 t ship) | 18.1 t | 0.88 | 9,529 m/s |
| Antimatter | M21's antimatter lander | | 3.3 | 55 km/s |

## 4. Routes from Tenebris

One way is surface to surface (to orbit for a giant), in m/s: climb + leave +
capture + land. Home is the way back from the destination's surface. "There
and back" is the first craft whose delta-v covers both on one stack without a
refuel.

| Destination | km (year) | One way | Home | Round | Rocket | Fusion course | Antimatter | There and back |
| --- | --- | --- | ---: | ---: | --- | --- | --- | --- |
| Lantern | 26 | 538 + 88 + 51 + 114 = 791 | 215 | 1,006 | 6:52 | 1:04, 1.27 km/s | 0:57 | Chemical |
| Selene | 44 | 538 + 104 + 43 + 64 = 749 | 157 | 906 | 11:04 | 1:27, 1.75 km/s | 1:15 | Chemical |
| Taper (M21's practice moon, mockup only) | 16 | 538 + 67 + 41 + 89 = 735 | 180 | 915 | 4:59 | 0:45, 0.91 km/s | 0:45 | Chemical |
| Sequoia | 80-680 | 538 + 222 + 225 + 50 = 1,035 | 866 | 1,901 | 6:57-40:17 | 2:02-6:15 | 1:41-4:53 | Nuclear thermal |
| Sprout | 80-680 | 538 + 222 + 192 + 33 = 985 | 276 | 1,261 | 6:57-40:17 | 2:02-6:15 | 1:41-4:53 | Chemical |
| Crag | 184-420 | 538 + 222 + 214 + 246 = 1,220 | 510 | 1,730 | 12:43-25:50 | 3:09-4:48 | 2:33-3:51 | Nuclear thermal |
| Quartz | 184-420 | 538 + 222 + 217 + 56 = 1,033 | 323 | 1,356 | 12:43-25:50 | 3:09-4:48 | 2:33-3:51 | Chemical |
| Ochre | 120-480 | 538 + 222 + 222 + 77 = 1,059 | 465 | 1,524 | 9:10-29:10 | 2:31-5:08 | 2:03-4:06 | Nuclear thermal |
| Verdance | 150-750 | 538 + 222 + 212 + 50 = 1,022 | 608 | 1,630 | 10:50-44:10 | 2:50-6:38 | 2:18-5:08 | Nuclear thermal |
| Pebble | 350-950 | 538 + 222 + 275 + 30 = 1,065 | 355 | 1,421 | 21:57-55:17 | 4:22-7:45 | 3:30-5:47 | Nuclear thermal |
| Amber (no landing) | 700-1,300 | to orbit 538 + 222 + 370 = 1,130; aerostat 760 + entry at 1.3 km/s | | | 41:23-1:14:43 | 6:21-9:41 | | Chemical, one way |
| Cinder | 700-1,300 | 538 + 222 + 201 + 137 = 1,097 | 387 | 1,485 | 41:23-1:14:43 | 6:21-9:41 | 4:58-6:46 | Nuclear thermal |
| Rime | 700-1,300 | 538 + 222 + 178 + 125 = 1,063 | 354 | 1,417 | 41:23-1:14:43 | 6:21-9:41 | 4:58-6:46 | Nuclear thermal |
| Murk | 700-1,300 | 538 + 222 + 159 + 50 = 968 | 455 | 1,423 | 41:23-1:14:43 | 6:21-9:41 | 4:58-6:46 | Nuclear thermal |
| Glaucus (no landing) | 1,300-1,900 | to orbit 1,025; aerostat 760 + entry at 0.8 km/s | | | 1:14:44-1:48:03 | 9:41-13:01 | | Chemical, one way |
| Hollow | 1,300-1,900 | 538 + 222 + 172 + 64 = 996 | 286 | 1,282 | 1:14:44-1:48:03 | 9:41-13:01 | 6:46-8:10 | Chemical |
| Tidewater | 1,300-1,900 | 538 + 222 + 155 + 50 = 964 | 517 | 1,481 | 1:14:44-1:48:03 | 9:41-13:01 | 6:46-8:10 | Nuclear thermal |
| Frostbite | 2,101-2,700 | 538 + 222 + 212 + 312 = 1,284 | 574 | 1,858 | 1:59:12-2:32:28 | 14:08-17:28 | 8:36-9:45 | Nuclear thermal |
| Hoar | 2,101-2,700 | 538 + 222 + 223 + 47 = 1,030 | 320 | 1,350 | 1:59:12-2:32:28 | 14:08-17:28 | 8:36-9:45 | Chemical |

What the table says, honestly: because the wells dominate, a chemical stack
can reach every world one way, and the small airless moons are cheap to land
on, so it even comes back from some far ones. What the tiers buy is mostly
**time** (a rocket coasts for hours to the outer system; fusion crosses in
minutes) and **round trips** (the nuclear thermal stage goes there and back to
nearly everything without a refuel). The fusion tier needs refuelling for
long courses (6 km/s at the cap against a 9.5 km/s tank) and lives at
stations; landers do the up and down.

## 5. Stations, giants, fuel, star lanes

- **Highport**: circular orbit 2,400 m up (r 7,200 m), 283 m/s, 2 min 40 s a
  turn; about 570 m/s from the pad (538 + 17 + 16). Where landers meet fusion
  ships. Recommendation taken (ask only with screenshots).
- **Skimmer platform**: at the top of a giant's airship band (unchanged from
  star-systems design 6); reached by an aerostat drop or a fusion lander.
- **Rime depot**: water ice into hydrogen for nuclear thermal engines and
  propellant for chemical ones.
- **Fuel by tier**: chemical propellant (refinery), nuclear thermal (fission
  fuel rods, hydrogen from ice), fusion (helium-3 pellets, from the giants),
  antimatter cells (the collider). A ship that cannot get home is refuelled or
  towed; it stays where it is, boardable (CLAUDE.md, occupancy not existence).
  M2's 400 m/s reserve is withdrawn.
- **Star lanes** (star-systems design 4, unchanged): gates 1,800 km from each
  star, 30 s per light year. Reaching Aubade's gate is a trip like any other:
  a fusion ship in 11 to 14 minutes, or a nuclear thermal coast of 1.5 to 2
  hours.

## 6. The mockup

`docs/mockups/travel.html`, reworked:

- The 2026-11-12 draft's parts struck through with reasons, at the top.
- The trip door to door (Tenebris to Sequoia) as a chain with its budget.
- **One 3D view at a time, in tabs** (`#launch`, `#arrive`; the renderer is
  made when the first tab opens; one canvas, one WebGL context; only the open
  tab draws, and only while playing or when the view is dragged):
  - **Launch from the field**, at night: M21's First orbit stack built cell by
    cell from M21's part textures on a stone pad with an iron rim of lamps,
    flood masts, the gantry and its beacon, the assembly hall's lit door, Pad B,
    the Kestrel parked on the apron with its lights on, and the town it flew
    from 1.6 km off. Ground is the game's cells near the pad, a 12x far ring to
    2.4 km (as M21), and the globe beyond, all curved to Tenebris's radius. The
    flight is integrated in the page (semi-implicit Euler, 0.02 s, inverse
    square): straight up to 40 m, nose over with the square root of altitude,
    cut at a 960 m apoapsis, circularise. It reaches Ap 989 m, Pe 924 m on
    491 m/s in 28 s; M21 measured 535 to 548 m/s on its own turn. The core
    stage reaches orbit with about 94 m/s left; the page says so rather than
    stage early.
  - **Arrival: powered descent** onto Lantern's depot at night: the Lantern
    lander's upper stage (M21) from low orbit (300 m up, 92 m/s): a 4 s
    deorbit burn, a fall, a braking burn held retrograde, a vertical
    touchdown. Measured: 18 s, 149 m/s, touchdown at 4.5 m/s (under M21's 6).
    The table's 114 m/s assumes better guidance (M21's scripted landing on Taper took 151).
  - Nothing moves until Play; drag turns the camera, scroll zooms; Night is the
    default, Day is a button. `?t=` holds a moment for captures.
- Which tier reaches where: tier cards, the route table with the day slider,
  the system map coloured by the first tier that goes there and back.
- Leaving a world: the well table per body and a pull chart (inverse square
  for rockets, Tenebris's anchor band for walkers and aircraft).
- Landing (air, thin air, airless), gas giants (orbit, moons, aerostat; link to
  planets.html#descent), stations, fuel, star lanes, art, decisions.

Art: the game's atlas and settlement textures (`travel/game/`, README), M21's
part textures and icons copied byte-identical into `travel/rocket/`, a new
aerostat icon by `tools/art/travel_art.py` (balatro-pixel-art kit), the planets
mockup's globe maps.

Verified in headless Chromium on SwiftShader (no GPU in this cloud container;
no frame numbers taken): no console errors, both tabs draw, one WebGL context
across both tabs, no horizontal scroll at 390 px, and nothing moves until Play.
The GPU screenshot gate of ART-DIRECTION.md is for the owner's desktop.

## 6a. Matched to M21 as pushed on 2026-11-14

M21's later push (home world Tenebris, Sequoia at 380 km with a 900 m low
orbit in its planner, the practice moon Taper, no landing on gas giants) was
read from its branch and matched: names, Sequoia's low orbit, the First orbit
stack and its 548 m/s climb, the fusion and antimatter examples. One naming
difference is left for that session: M21's lander HUD says "Highport field"
for the ground pad, where this change keeps Highport as the station in orbit
and calls the ground the launch field.

## 7. Decisions

- T10, travel is M21's ships: **the owner's words** (2026-11-14).
- T11, a 300 m/s coast in the tables: recommendation taken (ask only with
  screenshots).
- T12, landing by air or by engine: recommendation taken (ask only with
  screenshots).
- T13, Highport in orbit: recommendation taken (ask only with screenshots).
- T14, gas giants: the owner's rule (star-systems).
- T15, star lanes only, no in-system lanes: recommendation taken (ask only
  with screenshots).
- T16, a new change rather than extending ship-tiers: recommendation taken
  (ask only with screenshots).
- Superseded and struck on the page: T1 to T6 of the 2026-11-12 draft. Kept:
  T7 (clamps), T8 (the star's heat), T9 (star lanes).
