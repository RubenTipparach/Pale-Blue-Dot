# Design: steer-and-sail

## Context

The proposal quotes the owner. The physics is the game's own:
- **Gravity**: 25 m/s^2 at 1 g.
- **Densities**: sea 1025 kg/m^3, air 1.225 kg/m^3.
- **Foils**: one foil model for wings, rotors, sails, keels and rudders.
- **Wind**: the atmosphere's wind field.
- **Bearings**: the map and the compass bar read them through `geo`'s
  compass functions, with north the -Y pole.

The mockups compute their numbers from these. Nothing here is built.

## Decisions

### 1. Rockets: thrust vectoring

- Each engine part carries a gimbal limit:
  - chemical: 6 degrees;
  - nuclear thermal: 3 degrees;
  - solid booster: 0 (fixed);
  - fusion: 2 degrees;
  - antimatter: 4 degrees.
- The pitch or yaw torque is the sum over engines of
  `T sin(delta) x d`, where `d` is the engine's distance below the centre of
  mass. A stack's turn rate is that torque over its moment of inertia about
  the centre of mass (each cell's mass x its distance squared), so the
  assembly building can show a turn rate per stage.
- Roll comes from engines off the axis gimballing apart, and needs at least
  two. A single-engine stack rolls only with fins in air.
- Fins are foils on the ring. In air they add a restoring torque that grows
  with dynamic pressure, and in vacuum they do nothing.
- **Controls**: W and S pitch, A and D yaw, Q and E roll, and Shift and Ctrl
  set the throttle. The gimbal follows the stick with a 0.15 s lag. The
  dampeners hold attitude when the stick is centred (the CLAUDE.md
  dampener rule).

### 2. Planes: control surfaces

- **Which cells**:
  - the outer two wing cells each side are ailerons;
  - the tailplane cells are the elevator;
  - the fin cells are the rudder.
  The builder marks them, and a plane with none of one kind fails that
  check ("no elevator: it cannot pitch").
- **What a surface does**: it is the foil of its cell with a deflection of
  up to 20 degrees, which changes that foil's lift coefficient by about
  0.05 per degree. The force is `q S dCL`, where q is dynamic pressure
  `0.5 rho v^2`, and the torque is that force times the cell's arm from the
  centre of mass. Roll, pitch and yaw rates follow per axis, so a long tail
  arm or wide-set ailerons give more authority. At rest, q is zero and so is
  the authority.
- **Controls**: the mouse or W and S for the elevator, A and D for the
  ailerons, Q and E for the rudder, and Shift and Ctrl for the throttle.
  Trim holds the elevator. Mouse look stays raw for the camera, and the
  plane's controls are separate.
- **Helicopters**: cyclic tilts the rotor disc, collective sets lift and the
  tail rotor sets yaw. Tiltrotors blend from helicopter controls to plane
  controls as the nacelles tilt, as the Kestrel does (`player/vehicles`,
  "The VTOL converts from rotor lift to wing lift").

### 3. Ships: wind, helm, waypoints

- **Sails**: a sail cell is a foil in the apparent wind, which is the
  atmosphere's wind at the masthead minus the ship's velocity. The yard or
  boom angle is set by the sheet. The keel and rudder are wet foils. This is
  the model the Tern and the cog already sail by, so a hex ship gets a polar:
  - it cannot point closer than about 45 degrees to the wind;
  - it is fastest on a beam reach;
  - it is slower dead downwind.
- **Helm**: A and D turn the wheel. The rudder is a foil, so steering needs
  way on: a ship with no speed does not turn.
- **Throttle** for powered ships: W and S step through ahead slow, half and
  full, and astern.
- **Waypoints**:
  - On the map screen (M), a click adds a waypoint at that place, and a drag
    moves one. The waypoints form a route, a stored record on the craft:
    ordered places in frame direction, saved through the durable path, and
    authored by the player.
  - A waypoint is reached within 30 m. The next one then becomes active.
- **GPS readout**, a line under the compass bar while a route is active:
  - the bearing to the active waypoint, through `geo::bearing`;
  - the distance along the ground;
  - the course and speed over the ground;
  - the cross-track error, the distance off the line from the last waypoint;
  - the time to the waypoint at the current speed.
  The compass bar shows the waypoint as a marker, and the map draws the
  route and the ship's track. No autopilot steers for the player at these
  levels; a later tier can add one.

### 3a. The GPS advises the helm and the sail

The owner, 2026-11-13: "the ship gps system should tell me what direction to
turn my ship and what direction to turn the sail to get the optimal speed
towards my waypoint".

- **Best heading.** From the ship's own polar at the present true wind, the
  GPS finds the heading with the most velocity made good (VMG) toward the
  active waypoint: speed through the water times the cosine of the angle
  between heading and bearing. Upwind, that is a close-hauled course on one
  tack. The GPS picks the tack that keeps the cross-track error smaller,
  and says when to tack: when the other tack's VMG becomes the better one.
  Powered ships head straight for the waypoint.
- **Helm advice.** It names the turn in words and degrees, for example
  "Turn 25 deg to port (head 040)", "Steady", or "Tack now: bring her
  through the wind to starboard". The compass bar marks the best heading as
  well as the waypoint.
- **Sail advice.** For the apparent wind at the best heading, the sheet
  angle that gives the most drive, for example "Ease the sail 10 deg (to
  35 deg)", "Haul in 5 deg" or "Trimmed". It also flags a sail that is
  luffing or stalled.
- **A sailing route** (the owner: "a "sailing" route if you will"). Between
  the ship and the active waypoint, the GPS plans and draws the course a
  sailor would sail, not the straight line:
  - straight where the waypoint can be laid on one board;
  - upwind, a zigzag of close-hauled legs between the laylines, with each
    tack point marked;
  - deep downwind, gybing angles when broad reaching is faster than a dead
    run.
  It is replanned as the wind changes or the ship strays. The helm advice
  steers along it, and the time to the waypoint is the route's time, not the
  straight line's.
- **Advice only.** The player steers and trims. The readout shows what the
  best course gains: the VMG now, and the VMG on the best heading.

### 4. Six levels of ship power

Each level is a block or a pair of blocks in the drydock's hex builder, one
cell each unless noted. Numbers are mockup first guesses.

| Level | Blocks | Mass | Power | Fuel and use | Note |
| --- | --- | ---: | ---: | --- | --- |
| 1 Rowing | Oar bench (1 cell, two rowers) | 0.4 t | 1.2 kW a bench | Crew stamina: 10 min hard, then half | Speed ~2 m/s; for small hulls |
| 2 Sailing | Mast, sail cells | as M23 | from the wind | none | Polar from the foil model |
| 3 Steam turbine | Boiler (2 cells) + turbine + bunker | 6 t + 3 t + 2 t | 400 kW | 0.12 t of coal (or 0.2 t of wood) an hour per 100 kW | Smoke, a funnel |
| 4 Oil engine | Diesel engine + oil tank | 3 t + 1.5 t | 600 kW | 25 kg of fuel oil an hour per 100 kW; a tank holds 1.2 t | Needs the refinery |
| 5 Fission | Reactor (2 cells, shielded) + turbine | 30 t + 3 t | 4 MW | One fuel rod set lasts 200 h of full power | M20's fission tier |
| 6 Battery | Battery bank + electric motor | 4 t a bank + 1 t | 800 kW per motor | 2 MWh a bank; recharged at a dock from the grid | Silent, no fuel aboard |

- **Speed from power**: the resistance is `R = 0.5 rho Cf S v^2` (wetted
  area S from the submerged cells) plus a wave term. The wave term rises
  steeply past hull speed: it multiplies by `(v / v_hull)^4` above
  0.9 v_hull. The top speed is where `P eta = R v`, with a propeller eta of
  0.6. The builder shows the top speed and the range on the fuel aboard.
- **Checks**:
  - an engine needs its fuel store aboard;
  - a boiler needs a funnel cell above it to the open air;
  - a reactor needs shielding cells round it;
  - a propeller cell sits aft, below the waterline.
- **Power is the industry's.** A battery ship charges from the dock's
  connection to the grid, and fuel oil and fuel rods come from M20's
  chains. The level a player can build follows what they have built on the
  power ladder.

### 5. Oil

- **Oil fields are a field, not cells.** A seed-derived 2D field of
  reservoir pressure under the land and the shallow sea, about 6% of the
  surface:
  - bigger under sedimentary biomes (fields, desert, swamp, shelf sea);
  - none under the high mountains.
  It is read like the climate: derived, never saved. No cell of any world
  changes, so no new generator version is needed and old worlds have the
  same fields as new ones (CLAUDE.md, "Saved games survive every change").
- **Prospecting**: the map gets an "Oil" overlay once the player has built a
  survey drill, and a dowsing readout shows the pressure under the player.
- **The pumpjack** stands on a cell over a field and draws crude:
  - up to 3 t a day;
  - falling as the well's share of the field's pressure is drawn down.
  A well is a stored record (position, drawn total), authored by whoever
  built it. Offshore, a rig on piles does the same.
- **The refinery**: a distillation column, two cells and four layers tall,
  splits 1 t of crude into:
  - 0.45 t of fuel oil;
  - 0.25 t of diesel;
  - 0.15 t of polymer feed (the polymer M3 already uses);
  - 0.15 t of residue (asphalt, used for roads).
- **The oil plant** is a 40 MW boiler and turbine on fuel oil. On the power
  ladder it sits between coal and wind, with smoke and an output figure
  like the coal plant's.

### 6. The mockups

- **M21 Rockets and Landers**:
  - The launch preview becomes flyable: W, A, S and D drive the gimbals
    (the engines visibly swivel and the plume follows), with a throttle.
  - The stage's turn rate is shown in the assembly building.
  - A ring-4 rocket turns slower than a small hop, as the moment-of-inertia
    numbers say.
- **M24 Hangar and Chibi Aircraft**:
  - A "Fly it" view takes a built design off the runway. The surfaces
    deflect visibly with the stick, and the camera chases.
  - The builder marks the control-surface cells and checks them.
  - Helicopters lift and fly on cyclic and collective.
- **M23 Drydock and Hex Ship Builder**:
  - The six power levels become blocks, each preset with its level (a
    rowing launch, the sloop, a steam packet, an oil coaster, a fission
    freighter, a battery ferry), with top speed and range.
  - A "Sail it" view shows a sea map drawn compass-north up:
    - the wind field as arrows;
    - waypoints the viewer clicks;
    - the ship's track;
    - the GPS readout;
    - a helm (A and D) and sheet or throttle (W and S).
  - A sailing ship must tack to reach a waypoint upwind.
  - A day and night toggle lights the ship's lanterns and running lights.
- **M20 Pits, Plants and Power**:
  - A pumpjack over an oil field, a refinery column and an oil plant are
    added to the scene and the power ladder, lit at night (flare stack,
    windows).
  - The map gets the oil overlay.
- **Every view stays still until the viewer acts** (CLAUDE.md, "Nothing in
  a mockup spins on its own"). Play, Fly and Sail start on a press.

### 7. As the drydock mockup built it (M23, 2026-11-13)

Recommendations taken (ask only with screenshots):
- **Wave term.** The drag factor is the larger of 1 and
  `(v / (0.9 v_hull))^4`, so drag rises smoothly. Read literally, decision
  4's wave term would have dropped at the threshold.
- **Turbine.** A turbine passes up to 4 MW, so one serves a boiler pair or a
  reactor pair.
- **Telegraph and sheet.** On a ship with an engine, W and S work the
  telegraph and the sails trim themselves. A pure sailing ship sheets on
  W and S.
- **Bunker.** A coal bunker holds 4 t.
- **Sail limit.** The sail luffs inside 25 degrees of apparent wind, tuned
  so a ship points no closer than about 45 degrees true. That is a stand-in
  for what the foil model gives.
- **Heel.** Hex hulls are stiff enough that heel is near zero.
- **The chart.** "Sail it" is a 2D chart of 3.6 x 2.4 km, rastered at 15 m.

The presets:

| Preset | Power | Top speed | Range |
| --- | --- | --- | --- |
| Rowing launch | 7 kW | 2.0 m/s | |
| Sloop | sail | 6.0 m/s on a reach | |
| Steam packet | 400 kW | 6.0 m/s | 357 km |
| Oil coaster | 600 kW | 6.6 m/s | 381 km |
| Fission freighter | 4 MW | 10.4 m/s | 7,460 km |
| Battery ferry | 800 kW | 7.6 m/s | 273 km |

## Risks

- **Controls on the keyboard.** Four craft kinds now share W, A, S and D
  with different meanings. Each mockup shows its keys, and the game's binding
  table carries them per craft, as it does for the cog.
- **Wind in the mockup is a stand-in.** The "Sail it" view uses a smooth
  field shaped like the game's (trades, westerlies), not the atmosphere
  itself.
- **Oil as a field** means a well never runs a cell dry. It draws down
  pressure. Digging into an oil field finds rock, not liquid. That keeps
  saves safe, and liquids in cells are a change of their own.
