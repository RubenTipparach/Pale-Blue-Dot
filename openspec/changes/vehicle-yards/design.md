# Design: vehicle-yards

## Context

The proposal quotes the owner. Three asks:
- larger fabricators, cranes and printers;
- drydocks and ships made of hex blocks that the player walks;
- chibi aircraft built in a hangar with a builder.

This design fixes what each mockup shows and the rules its numbers come from,
so that the pages agree with each other and with the game.

What the game gives us:
- **The cell.** 2.833 m flat to flat and 1 m tall on every body. Its plan
  area is (sqrt 3 / 2) x 2.833^2 = 6.95 m^2, so a full cell is 6.95 m^3.
- **Gravity.** The game's 1 g is 25 m/s^2 (`SURFACE_GRAVITY_MPS2_PER_G`), and
  craft feel it.
- **Densities.** Sea water is 1025 kg/m^3 and air 1.225 kg/m^3
  (`pbd_core::vehicle`).
- **Hulls.** A hull floats by its cells (`hull::float`), and a buoyancy cell
  may be at most a third of the beam (`HullSpec::validate`).
- **Foils.** Every lifting surface is one foil model: wings, rotors, sails
  and keels (`openspec/specs/player/vehicles`).
- **Walking on a deck.** The walker stands on a craft's deck in the craft's
  frame, through a turn and under way (`sail-the-cog` 3.1, 3.2, built).
- **Building pieces.** The town's cut puts walls on cell edges and floors in
  cells, and a storey is three layers (`player-building`, `tenebris-towns`).

## Decisions

### 1. Three pages, three ids

| Id | Page | Title | Answers |
| --- | --- | --- | --- |
| M22 | `docs/mockups/construction.html` | Cranes and Printers | M3's note: advanced fabricators, cranes, printers for buildings |
| M23 | `docs/mockups/drydock.html` | Drydock and Hex Ship Builder | drydocks; ocean ships of hex blocks, walked while they sail |
| M24 | `docs/mockups/hangar.html` | Hangar and Chibi Aircraft | chibi planes, helicopters, VTOLs; a hangar; a builder like M4's |

Each page follows `docs/mockups/ART-DIRECTION.md`:
- **The game's own art first**, copied byte-identical into
  `docs/mockups/<page>/game/` with a README that names each source.
- **New art from the balatro-pixel-art kit**, made by `tools/art/<page>_art.py`.
- **Nearest sampling** everywhere.
- **The ground** is the game's hex columns.
- **The HUD** where one is shown: the debug line and the hotbar.
- **The tracker link** sits under the title.
- **Lit at night** (CLAUDE.md): every lamp lights what is near it wherever
  the camera is, night is dark between pools, and the moon is dim. Day, dusk
  and night buttons, with night the default.

### 2. M22: the fabricator ladder past M3

M3 ends at the fabricator, which hands over pack-sized items. Above it:

| Tier | Machine | Makes | Needs |
| --- | --- | --- | --- |
| 4 | Assembly bay | Large items, delivered as a crated object in the world, not a pack stack: a mast, a wing pair, an engine, a wall panel, a hull section | Fabricator parts; power from M20's grid |
| 5 | Hex printer | A building, one 1 m layer per pass, printed into the plot's cells | A blueprint (M6), printstone fed by hopper |
| 5 | Tower crane | Lifts crated items into place | A footing of four stone cells |
| 5 | Gantry crane | Lifts along rails, over a bay or a drydock | Rails on cell edges |

- **The printer prints the town's pieces.** These are floors in cells, walls
  on cell edges, and solid cells for footings, because the town's cut is what
  a building is (`player-building`). A layer is 1 m, and the head traces that
  layer's wall lines. One storey is three passes. Each finished layer is one
  world edit through the durable path, with the printer named as its author
  (CLAUDE.md, "The player is not the only author of change"). A print that
  stops part-way leaves the layers it finished: they are real and saved.
- **The crane's load chart is a moment.** A tower crane lifts at most
  `M / r` at radius `r`, with `M` its rated moment, never more than its hook
  limit. The page's crane is rated 120 t m with an 8 t hook limit, so it lifts
  8 t out to 15 m and 3 t at its 40 m tip. A load past the chart is refused
  with the number. The cab is a seat: slew, trolley and hoist are the three
  controls, and mouse look stays raw.
- **Crated items** are records with a stable ID, a mass and a footprint in
  cells. They take the same durable path as a ship. A crate set down snaps to
  the cut, and a crate lifted into a printed building's frame becomes its
  piece.

The page:
1. **The yard** in 3D: the printer mid-print, the tower crane over it, the
   assembly bay, stacked crates and a site office. A print slider runs the
   building from bare plot to roof. Crane controls (slew, trolley, hoist)
   move the hook and its load, and the load chart refuses an overload.
2. **The assembly bay's queue**: large recipes with mass, footprint, time and
   inputs (icons from M3 where they exist).
3. **The printer's pass plan**: one layer drawn in plan with the head's path.
4. **The ladder table** above, and the save rules.

### 3. M23: ocean ships are hex cells in their own frame

- **The cell is the world's cell**: 2.833 m across, 1 m a layer (the owner:
  "made of hex blocks"). A ship's blocks are cells. Its rails, masts, helm,
  ladders and lanterns are the town's pieces, placed on cell edges and in
  cells as a house's are, so walking a ship is walking a building.
- **A ship's grid is flat and its own.** A ship is a body that moves, not a
  patch of the planet. Its cells are keyed `(q, r, layer)` in the ship's
  frame on a flat axial grid, with the bow along `+q`. The no-flat-grid rule
  (CLAUDE.md) is about faking a sphere, and a ship is not one. From the keel
  laid in the drydock onward the ship is in its own frame, so floating it out
  converts nothing.
- **It floats by sub-cells.** `hull::float` wants a buoyancy cell at most a
  third of the beam. Each hex cell is therefore floated as seven sub-cells
  (its centre and six around it at half the circumradius), each about 1.07 m
  across and 0.5 m tall. That meets the rule for any ship two cells (5.67 m)
  or more in the beam. **A hex ship is at least two cells in the beam.**
  Smaller craft stay lofted: the Tern and the Loon.
- **Which cells keep the sea out.** Hull, keel, ballast and bulkhead cells
  are solid or sealed. A hold or cabin cell is buoyant when it is enclosed: no
  open side below the waterline, and the cell above it decked. An open cell
  below the waterline floods. Its compartment fills by the bilge model the
  cog already uses (`BilgeSpec`).
- **Numbers from the cells**, worked out live by the builder:
  - **Mass**: the sum of each block's mass.
  - **Displacement and draft**: the waterline is found by bisection where
    the weight of the submerged volume equals the mass. A full submerged cell
    displaces 6.95 m^3, or 7.12 t of sea.
  - **Freeboard**: from the waterline to the lowest open deck edge.
  - **Stability**: `GM = KB + BM - KG`, where `BM` is the waterplane's
    second moment about the centreline over the displaced volume. Above
    0.5 m reads "stiff", 0 to 0.5 m "tender", and below 0 "capsizes".
  - **Sail area**: 2.83 m^2 per sail cell. A sail cell is above the main
    deck and joins a mast through other sail cells on its layer, so a yard
    can span the ship (the mockup's D4). Counting only the cells beside the
    mast, the sloop carries 57 m^2 and makes 2.4 m/s on a reach; counting
    along the yard, 147 m^2 and 6.0 m/s.
  - **Hull speed**: `sqrt(g L / 2 pi)` with the game's g of 25 m/s^2, which
    is 2.0 sqrt(L) m/s for a waterline of L metres.
  - **Speed on a reach**: hull speed x min(1, SA/D / 18), where SA/D is the
    sail area over the displaced volume to the 2/3. That is the
    sail-area-to-displacement ratio sailors use.
  - **Hold and berths**: counted from the hold and cabin cells.
- **Checks**, each with its reason:
  - a helm, at the stern on the main deck;
  - a rudder below the waterline, aft;
  - each mast stepped on a keel cell;
  - it floats with freeboard of at least 1 m;
  - GM above 0;
  - every cell joined to the rest;
  - the beam at least two cells;
  - headroom: a walkable floor has two clear layers above it, or a hatch and
    ladder;
  - **a walker can get from the gangway to the helm.** A flood fill over
    walkable floors, steps of one layer, and ladders.
- **The drydock** is world cells: a basin cut into the shore below sea level.
  - Its walls step down in altars, as a stone dock's do.
  - Its floor has keel blocks, and a caisson gate closes it to the sea.
  - A pump house drains it, and a gantry crane runs on rails along both
    sides.
  - States: **flooded, draining, dry, flooding, gate open**. A ship built
    dry floats as the dock floods and is warped out through the gate.
  - The dock's size caps the ship. The page's dock takes a ship 10 cells long
    and 5 rows wide (28 x 13.1 m), 7 layers deep from keel to waterline.
- **Odd rows (the mockup's D2).** A ship's grid keeps the world's half-cell
  offset between rows, so a hull is the same to port and starboard only
  with an odd number of rows. The presets have three (the sloop, 8.2 m) and
  five (the three-master, 13.1 m). The check still allows two rows, the
  least that floats by the sub-cell rule.
- **Block masses are the mockup's own** (D5), chosen so the presets float
  and stand: the sloop draws 0.45 m, the trader 1.22 m and the three-master
  1.39 m, with GM 5.5, 5.2 and 2.4 m. Hex ships float high and stiff. Real
  numbers come with the build.
- **Saves.** A ship is a stored record. It holds:
  - its stable ID and its cells with their blocks, keyed in its frame;
  - its pieces and the block set's version;
  - its pose in a named frame, its hold, and its flooding.
  Placing or breaking a ship's block is an edit in the ship's frame through
  the durable path, with its author named. A retuned block keeps its old
  numbers for ships already built, or migrates them forward on open. A
  player's ship never changes shape on its own.

The page:
1. **The drydock** in 3D, with a ship on its keel blocks under the cranes.
   - A water slider runs the five states. The ship lifts off its blocks as
     the water reaches its draft, then the gate swings and the ship floats
     out.
   - The game's Tern and Loon are moored at a jetty beside it for scale.
   - At night: floodlights, lamp posts along the copings, welding flashes
     on the hull, the pump house windows and the ship's own lanterns.
2. **The builder**: a hex grid per layer.
   - Layer tabs run from the keel up to the castle, and a side profile draws
     every layer with the solved waterline.
   - A palette, presets, live stats and the checks.
   - "Float it out" puts the design in the dock.
3. **Walk the deck**: a first-person view aboard the ship as it sails a
   slow circle. The walker stays on the deck while the horizon turns.
4. **How the numbers follow**, the blocks, a progression from a two-cell
   sloop to a three-masted ship, the save rules, and the questions.

### 4. M24: chibi aircraft, built in a hangar

- **Chibi**: these are the owner's reference pictures read as proportions.
  - The canopy is about a third of the body.
  - The fuselage is short and fat, the wings stubby and thick, and the wheels
    big.
  - The rotors are fat-bladed, and the colours are bright.
  - Made of boxes and prisms with pixel textures, never spheres or capsules
    (ART-DIRECTION). An octagonal prism is the round thing.
  - Liveries are the player's: base colour, trim, a shark mouth, a nose
    number and a roundel of the player's own. There is no real air force's
    insignia.
- **A hangar is required.** The builder opens at the hangar's workbench, and
  an aircraft is kept in a hangar bay. A built aircraft rolls out of the
  doors to the apron, the runway or the helipad. A bay's door width caps the
  span.
- **The builder is M4's**: a top-view grid of 1 m cells with the nose to the
  right.
  - Mirror painting is on by default.
  - Parts include: cockpit, fuselage, nose prop engine, jet, wing, tailplane,
    fin, main rotor mast, tail rotor, tilt nacelle, landing gear, float, fuel
    tank, cargo and gun-free utility pods.
  - A live 3D preview of the chibi built from the grid sits beside it, with
    the livery.
- **Numbers from the parts**, at the game's g:
  - **Wing area**: 1 m^2 per wing cell.
  - **Stall speed**: `sqrt(2 m g / (rho S CLmax))`, with CLmax 1.4.
  - **Hover margin**: rotor thrust over weight. Thrust is from momentum
    theory, `(2 rho A)^(1/3) P^(2/3)` x 0.8, where A is the disc area. Above
    1.1 it can hover.
  - **Top speed**: `(2 P eta / (rho Cd A_front))^(1/3)`.
  - **Balance**: the centre of mass must be ahead of the wing's centre of
    lift, by 0 to 1.5 m.
  - **Range**: fuel over burn at cruise.
- **Checks a pilot would feel**:
  - a single main rotor needs a tail rotor at least 3 m behind its mast;
  - a plane needs wings that stall under 45 m/s to take off from the page's
    runway. A VTOL that stalls above it still passes, with a note that it
    takes off vertically (Puffin stalls at 52 m/s);
  - a VTOL's nacelles must sit in mirror pairs;
  - wheels or floats to stand on;
  - a cockpit with a clear view forward.
- **The foil model flies them.** Each wing cell is a foil segment and each
  rotor a disc of foils, as the Kestrel's are. A chibi aircraft flies by the
  same rules as the Kestrel, which stands in the hangar as a scale reference.
- **Saves**: an aircraft is a stored record like a ship, with its parts
  grid, livery, pose, fuel and damage.

The page:
1. **The airfield** in 3D: the hangar with its doors open, the presets parked
   inside and on the apron, the runway, the helipad, a windsock and the
   Kestrel.
   - At night: runway edge lights, threshold lights, helipad lights, a
     rotating beacon, the warm hangar interior and nav lights on every craft.
   - Rotors and props turn.
2. **The builder** with presets:
   - **Sharky**, a prop fighter after the owner's first reference, with a
     shark mouth and a nose number;
   - **Bumble**, a utility helicopter after the second;
   - **Puffin**, a tiltrotor VTOL, a chibi cousin of the Kestrel;
   - **Gull**, a floatplane;
   - **Dart**, a little jet.
   "Roll out" puts the design on the apron.
3. **How the numbers follow**, the parts, the hangar's rules, the save rules,
   and the questions.

### 5. Questions are asked with pictures, or taken as recommended

By the owner's rule, a question goes to the survey only with screenshots
beside it. Each page lists its decisions with a recommendation. The ones
judged by eye go to the survey with the page's screenshots:
- **The chibi proportions** (M24).
- **How blocky a hex hull reads** (M23).
Everything else is "recommendation taken (ask only with screenshots)".

Asked on 2026-11-12 as **A1** (chibi proportions) and **A2** (whole-cell hulls)
in the survey (https://claude.ai/artifact/3ViY6udhFkkx5Wzx4jn3w5), each with
the mockups' screenshots. The rest of decisions 2 to 4, and the decisions
each page lists (C1 to C8, D1 to D6 and H1 to H6 on the pages), are
recommendations taken (ask only with screenshots), recorded under the
survey's Already decided.

The mockups' links:
- M22 Cranes and Printers: https://claude.ai/artifact/2LjxDc7LuNFX6yjPbMLMwc
- M23 Drydock and Hex Ship Builder: https://claude.ai/artifact/DLRzGBKMJzhhFdayrbCVdx
- M24 Hangar and Chibi Aircraft: https://claude.ai/artifact/HrLdbcwrwbAD7PN1KojKKu

## Risks

- **Hex hulls are blocky.** A hull of whole 2.833 m cells cannot curve, so a
  bow is a stepped point. The owner asked for blocks, and the mockup shows how
  it reads. The pieces (rails on edges) soften the deck line.
- **Big ships, many cells.** A three-masted ship of 10 x 4 x 9 is about 250
  cells, or 1,750 buoyancy sub-cells a tick. That is affordable on the CPU,
  but it is a number to measure when built, not now.
- **The chibi aircraft fly in 25 m/s^2.** Their stall speeds come out higher
  than the same aircraft on Earth (the 1 g is 2.5 times Earth's). The page
  shows the numbers as they fall out. A lighter chibi or bigger wings is the
  lever.
- **Software rendering.** The pages are screenshotted in a cloud container
  without a GPU (SwiftShader). The pictures show the look, not the frame
  rate.
