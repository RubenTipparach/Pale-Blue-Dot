# Design: ship tiers

Everything here is proposed. Numbers are the mockup's and are recommendations
to tune, not measurements of the game, except where a line says it was
measured (the ascent figures were measured by flying the mockup's integrator,
which uses the game's 25 m/s^2, Pale Blue's 4,800 m radius and a semi-implicit
Euler step like the game's).

## 1. What Tenebris does

Read from `C:\Users\santi\repos\tenebris\tenebris-rs\crates\tenebris-core`
(2026-11-12); the TypeScript original has no orbital mechanics, only a gravity
well round the nearest planet (`src/config/RocketConfig.ts`).

| Part | What it does | Where |
| --- | --- | --- |
| Bodies | Circular rails (radius, period, phase, inclination), moons relative to their parent; a closed-form function of the clock, no n-body. For physics the bodies are **parked** at t = 0; the rails only move the sky. | `orbit.rs` 1-7, 52-70, 229-253; `planet.rs` 689-693; `nav.rs` 8-9 |
| Ship motion | Newtonian, semi-implicit Euler: thrust along the nose, then gravity, then position. Gravity is `orbital_gravity_at`: true inverse-square from every body (scaled so each matches its surface gravity at 25 m/s^2 a g) plus the star, uncapped. | `planet.rs` 886-936; `player_ctrl.rs` 1895-1918 |
| Prediction | **Patched conics.** `Conic::from_state` from the eccentricity vector, Kepler's equation by Newton's method; the sphere of influence is whichever body pulls hardest; 256 coarse steps find SOI crossings, up to 5 hops over 7,200 s; drawn as 768-segment polylines with Ap/Pe ticks. | `nav.rs` 32-59, 101-164, 226-310, 418-503; `nav_ui.rs` 876-945 |
| Maneuver nodes | **Yes.** Click the path to place one; prograde, normal and radial handles; post-burn orbit drawn in purple; need/have delta-v from the rocket equation. | `nav.rs` 509-568; `nav_ui.rs` 519-520, 702-782 |
| Autopilot | Attitude hold only (KSP's SAS): prograde/retrograde, normal, radial, target, node, kill rotation. Nothing flies a burn for you. | `nav.rs` 575-660; `nav_ui.rs` 259-312 |
| HUD | `orbit [Body]  Ap N m  Pe N m  T N s` in cyan when bound, `ESCAPE` in orange; navball; 12 px monospace. Colours: bound cyan (0.35, 0.9, 0.95), escape orange (0.95, 0.6, 0.25), post-burn purple (0.72, 0.38, 0.95), target pink (0.98, 0.35, 0.75), station track green (0.4, 0.8, 0.6). | `nav_ui.rs` 58-104, 1940-1997 |

So Tenebris is not on-rails for ships: it integrates them in an n-body *field*
(bodies fixed, pull summed) and then draws a second, analytic model (conics) of
where they will go. The mockup's tab 2 borrows its readout format and colours.

## 2. The decision: how chemical rockets meet orbits (survey G4)

| | A. Keep the invariant | **B. Orbital field, no nodes (recommended)** | C. All of Tenebris |
| --- | --- | --- | --- |
| Rocket's field | Today's anchor band | Inverse-square from every body + star | Same as B |
| Orbits | Inside 1.8 R only; they rosette | Any height; ellipses close | Same as B |
| Prediction line | Integrated forward | Integrated forward | Kepler conics, SOI hops |
| Maneuver nodes | No | No (the nav computer plans, tier 2) | Yes |
| Moon at 26 km | Straight coast | Hohmann arc, 4 min 22 s | Same as B |
| CLAUDE.md | Unchanged | One clause changes | Most of the clause goes |

Why B: the owner wants Tenebris's orbital play. What the invariant was written
against is a second model of motion that has to agree with the integrator (the
conic solver) and a planning UI that turns flying into bookkeeping (nodes,
windows). B keeps the physics and leaves both out: the drawn path *is* the
integrator run ahead, so it cannot disagree with what the ship does, and the
plan is to fly by the line. The nav computer at tier 2 is the planner, which
fits the owner's own "navigation computers plot courses".

What B changes in the invariant, as proposed wording (applied only if chosen,
in the commit that builds it):

> Assisted ships, hover and walkers use the bounded anchor field. Rockets use
> the orbital field: inverse-square pull from every on-rails body and the star,
> integrated Newtonian, with the predicted path drawn by integrating the same
> field forward. Do not restore patched conics, maneuver nodes, transfer-window
> requirements, or ship-to-ship gravity.

Consequences of B that are part of the plan:

- **Rails must agree with the field.** A moon's rail speed must equal circular
  speed in its parent's inverse-square field, or a ship parked in the moon's
  orbit drifts off it. Lantern at 26 km round Pale Blue (mu = 25 x 4800^2 =
  5.76e8 m^3/s^2) circles in 18.2 minutes at 149 m/s. The planets mockup's
  Lantern period has to follow, as a new generator-independent body version
  (bodies' rails are not terrain, but a save names its system's layout, so the
  rail change is versioned like everything a save depends on).
- **The star barely matters at this scale.** With planets on 3 to 7 m/s rails
  (travel.html), the star's pull at 300 km is about 3e-5 m/s^2: over an
  11-minute coast it bends a path by tens of metres. Arcs happen round a body
  and its moons; between planets a chemical trip is a climb out on a curl and a
  near-straight coast. The mockup shows this honestly (tab 3, "To Verdance").
  Making interplanetary chemical trips into big arcs would need the planets on
  fast rails, which travel.html and planets.html deliberately avoided.
- **Prediction is cheap.** 0.5 s steps for 6,000 steps covers one low orbit
  (114 s) many times over; the mockup recomputes it every 3 s of sim time.
  Determinism: the same Euler step and the same rail functions; the line is a
  preview, never saved.

Until G4 is answered, the plan and the spec deltas are written for B and
marked provisional. Picking A drops the orbital-field tasks and the flight
delta; picking C adds a conic solver and nodes and removes most of the clause.

## 3. The hex VAB

- **The grid.** Axial hex cells (the ground lattice: pointy along z,
  2.833 m flat to flat) and integer layers 1 m tall. The rocket's core column
  is cell (0, 0), and the grid reaches **four rings** out from it (the owner,
  2026-11-12: "Allow larger rockets with more leg room to walk around in.
  (3-4 radius? Blocks)"). Ring 1 makes a cluster 3 cells (8.5 m) across,
  ring 3 one 7 cells (19.8 m) across, ring 4 one 9 cells (25.5 m) across:
  61 cells a layer. A layer is edited as a hex disc in plan, as the drydock's
  builder does, with the stack drawn beside it in profile.
- **Room inside.** Rockets past ring 1 have room to walk. Interior parts,
  one cell each: **crew deck** (a floor on the cell's top, walkable),
  **cabin** (berths and a window), **ladder** (between decks, through the
  floor), **hatch** (an outer door and its gangway), **airlock**, and
  **hull** (an outer wall cell, sealed). A walkable floor has two clear
  layers above it, as in the drydock and the town's houses; a storey is three
  layers. The checks add: the command pod can be walked to from the hatch,
  and every crew deck is reached by a ladder. A walker stands on the decks in
  the rocket's frame, as on the cog's deck, on the pad and in flight.
- **As the mockup builds it** (recommendations taken, ask only with
  screenshots): interior masses deck 0.15 t, cabin 0.4, ladder 0.05, hatch
  0.3, airlock 0.5, hull 0.15; a hatch or airlock is two layers tall,
  standing on a deck, and a cabin counts as headroom where a hull wall does
  not; the core's decoupler splits its whole layer, and decouplers elsewhere
  on that layer only draw the separation ring. A stack holds together when
  every cell touches the core column through other cells. Presets: Small
  hop, First orbit, Too heavy, Lantern lander, Crew rocket (ring 2, two
  decks), Heavy lifter (ring 3, cargo deck, airlock), Colony lander (ring 4,
  752 cells, three decks, fifteen cabins; its upper stage lands on a light
  world like Lantern, not Pale Blue) and Antimatter lander.
- **Parts, one cell each** (mockup numbers; tonnes, kN, m/s):

  | Part | Where | Mass | Propellant | Thrust | Exhaust |
  | --- | --- | ---: | ---: | ---: | ---: |
  | Command pod | core, top | 1.3 | | | |
  | Fuel tank | any | 0.4 | 4.2 | | |
  | Chemical engine | any | 1.0 | | 520 | 600 |
  | Nuclear thermal engine | any | 2.6 | | 220 | 1,500 |
  | Decoupler | core | 0.25 | | | |
  | Solid booster (a cell) | ring | 0.35 | 2.4 | 200 | 480 |
  | Fin | ring | 0.08 | | | |
  | Landing legs | ring, bottom | 0.15 | | | |

  Exhaust speeds are the world's scale, not Earth's: low orbit here is
  316 m/s, so a 3 km/s engine would make staging pointless. Measured on the
  mockup's flights, low orbit costs **535 m/s** in the orbital field and
  **574 m/s** in the anchor band with this gravity turn (straight up to 40 m,
  nose over with the square root of altitude, cut when the predicted Ap
  reaches 960 m, circularise at Ap).
- **Rules.** The core stacks straight up, pod on top, no gaps. A ring cell sits
  beside a core cell of its layer or on a ring cell below. An engine's nozzle
  needs the cell under it empty, or a decoupler (core), or the pad. Symmetry
  copies a ring placement round the stack 1, 2, 3 or 6 times.
- **Staging.** Decouplers in the core column split the stack; a decoupler goes
  with the stage below it. A stage's tanks feed its liquid engines; its solid
  boosters burn alongside them and drop when spent; then the stage drops at
  its decoupler. Per stage: mass at ignition (everything above included),
  thrust, TWR against a chosen body, burn time and delta-v
  (`ve ln(m0/m1)`, with a thrust-weighted exhaust speed while boosters burn).
- **TWR is the gate.** A first stage under 1.0 against the launch body cannot
  leave the pad; the launch preview shakes on the pad and says so. The mockup's
  "Too heavy" preset is 0.55.
- **One grid for all ships.** `ship-crafting.html`'s module grid moves onto the
  same hex cells and becomes the tier-2 ship builder (recommendation taken).
  The VAB builds only hex stacks, as the owner said.

## 4. Tier 1 in flight

A launched stack is a rocket: thrust along its nose, no dampeners (it has no
RCS authority to null drift at orbital speed), attitude hold with Tenebris's
SAS modes, and the HUD line `orbit [Pale Blue]  Ap  Pe  T`. Burns are flown
by hand against the predicted line (cyan bound, orange escape or impact).
Re-entry is the pod's heat shield; spent stages break up and are not saved
(recommendation taken).

Tier 1 reaches: low orbit (535 m/s), Lantern's orbit on a Hohmann arc
(+88 m/s out, +52 m/s capture), Lantern's surface (+110 m/s), and home.

## 5. Tier 2: fusion

- **Drive:** 400 kN at 8 km/s exhaust; 20 m/s^2 on a 20 t ship. It cannot lift
  its own weight at 1 g and its exhaust would cook a pad, so fusion ships live
  in orbit and at stations; they are built at a station yard on the hex grid.
- **Nav computer.** Given a destination (a body's orbit, a station, a ship) it
  solves for the departure direction that meets the target's rail position at
  arrival (rails are closed-form, so the meeting point is exact), then plots
  burn, flip and brake with times and delta-v, and flies it on a key. To
  Lantern: 1 min 4 s and 1.27 km/s against the chemical arc's 4 min 22 s and
  140 m/s. To Verdance (200 km on the day shown): 3 min 20 s and 4.0 km/s
  against about 11 minutes and 434 m/s. travel.html's transit drive is this.
- **Freighters.** A route is a stored record: a freighter's ID, its stops, what
  it loads and unloads, a schedule. Near a player the freighter flies its
  course in the world; away from every player it moves along the same course
  on rails (closed-form in time), so it is where the course says when someone
  arrives. Loading and unloading are world mutations through the durable save
  path, authored by the freighter, and never take from a player's own store
  (CLAUDE.md, "The player is not the only author of change"). World time runs
  only while the world is played.
- **Landers.** Hex-stack rockets with legs that lift to low orbit and come
  down again. The mockup's: chemical engine, three tanks, pod, three legs;
  16.6 t, TWR 1.26, 860 m/s: up on one tankful, refuelled in orbit for the
  powered landing. travel.html's free hover compensation is a lander feature
  from this tier on, never a tier-1 rocket's.

## 6. Tier 3: antimatter

- **Landers anywhere.** An antimatter core (1,000 kN, 60 km/s exhaust, 1.0 t;
  the mockup's four-core lander makes 55 km/s at a TWR of 3.3) gives about
  60 km/s of delta-v at a TWR of
  3 at 1 g: a lander flies surface to surface (Pale Blue to Cinder, 5 min 10 s,
  9.8 km/s) with no orbit stop.
- **No warp.** The owner, 2026-11-12: "Let's not do warp." There are no
  starships, no warp ring and no other star systems; antimatter is the top
  tier, and its landers reach anywhere in the one system.

## 7. The power ladder (industry.html)

| Tier | Power that unlocks it | Fuel it burns |
| --- | --- | --- |
| Before (Kestrel, Tern, Loon) | none | none in the air |
| 1. Chemical, nuclear thermal | coal to fission (the late reactor) | propellant from the refinery; fuel rods for the NTR |
| 2. Fusion | a fusion plant | fusion pellets |
| 3. Antimatter | the collider | antimatter cells |

The industry mockup is being drawn in parallel; its fusion and antimatter
rungs are named here from its icon set and are its to define. This change
only says which tier each rung unlocks.

## 8. Where the existing craft sit

- **Kestrel** (tilt-rotor), **Tern** (sail), **Loon** (canoe): the craft before
  the tiers. Assisted flight and the sea, built now (`openspec/specs/player/vehicles`).
  None reaches space; all stay useful. The Kestrel parks by the pad in tab 1.
- **ship-crafting.html** modules (cockpit, thrusters, lift jets, gyros,
  reactor): the tier-2 builder's parts, moved onto hex cells; their
  `FlightLimits` formulas carry over for assisted ships.

## 9. Decisions

- **G4 (asked, with tab 2 and tab 3 screenshots):** A, B or C above.
  Recommended B; built as B until answered.
- One build grid, hex cells for every ship: recommendation taken (ask only
  with screenshots).
- World-scale exhaust speeds (600 / 480 / 1,500 / 8,000 / 60,000 m/s):
  recommendation taken (ask only with screenshots).
- Spent stages break up and are not saved: recommendation taken (ask only with
  screenshots).
- Freighters on rails away from players, flown near them: recommendation taken
  (ask only with screenshots).
- Starships and warp: dropped by the owner ("Let's not do warp").
- The assembly grid four rings out, with crew decks inside: the owner's ask
  ("3-4 radius? Blocks"); four rings taken, the larger of the two.

## 10. Measured for this write-up

The mockup's ascent integrator (25 m/s^2 at 1 g, R 4,800 m, 0.05 s Euler
steps), flown with each example stack:

| Stack | Field | Result | Time | Delta-v to orbit |
| --- | --- | --- | ---: | ---: |
| First orbit (3 boosters, chemical core, NTR upper) | orbital (B) | orbit | 23 s | 535 m/s |
| First orbit | anchor (A) | orbit | 22 s | 574 m/s |
| Lantern lander | orbital (B) | orbit | 26 s | 539 m/s |
| Single stage (532 m/s) | orbital (B) | orbit, 0 left | 15 s | 532 m/s |
| Single stage | anchor (A) | out of fuel | | |
| Too heavy (TWR 0.55) | either | does not leave the pad | | |

This is a mockup's integrator, not the game's; the game's own numbers are a
task.
