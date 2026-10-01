# Design: cities in the world

## Context

See `proposal.md` for why. What the design has to work with, on `main` and in
the changes this one follows (2026-09-27):

- **The settlements are designed and prototyped.**
  - `tenebris-towns` covers the cut (walls on edges, floors in cells, a
    three-layer storey), the three stairs, thin-solid collision and the
    walker's three rules. It also covers the kits, seven settlements, lights,
    no coplanar faces, furniture clearance, doors as world state, and no
    building on a pentagon.
  - Its delta to `world/settlements` holds those requirements.
  - Its tasks 1 to 5 are the engine work, and none is started.
  - `docs/mockups/towns.html` is the prototype.
- **The engine has none of it.**
  - `PlanetContact::stand` answers floors and ceilings per hex column.
  - The walker climbs up to 1.05 m in one tick.
  - There are no pieces, no kits and no settlement data.
- **Lights, after `lamps-and-lanterns`:**
  - the lantern family, the brazier and the candle, as materials;
  - dusk-lit emitters that re-bake the field at dusk and dawn;
  - moving things lit by the field;
  - a measured bake time for 300 lanterns.
- **Sites, after `city-sites`:** each world holds its resolved site list (id,
  kind, anchor, name) in its save. The footprints avoid pentagons and their
  neighbours.
- **Pop-in is the owner's first priority** (CLAUDE.md). `detail-fade`
  dither-fades terrain and trees. It still pops in a third to a half of
  landings in flight, and that is being fixed on its own.
- **The column generator** is a pure function of the direction and the
  config, cached per column. The edit log replays on top of it.

## Goals / Non-Goals

**Goals:**
- Every site has its town, and the town is the mockup's town: the same stairs,
  collision, kits and light.
- Towns sit in the landscape: on terraces cut into the real ground, with
  walkable edges.
- Towns read at every distance: pieces up close, a reduced form beyond, and
  lights from orbit at night.
- A played world's towns never change under it, and a town never overwrites
  the player's work.

**Non-Goals:**
- **Townsfolk.** The mockup's figures walk fixed paths. People who live in a
  town are their own change (what they do, and whether they trade or talk).
- **Roads, trade and factions** between towns.
- **A procedural town generator.** Each kind has an authored template with
  seeded variation. A generator that grows streets and plots could replace
  the templates later, behind the same interface.
- **Destroying town pieces.** A town's cells (the masonry wall, the terraces)
  can be mined like any cell. Its thin pieces (walls, floors, stairs, roofs)
  cannot be removed until `player-building` brings the tools that remove
  pieces. Doors open and shut.
- **Boats that sail.** The harbour's boats are fixed obstacles and the cog is
  boarded like a building, as in `tenebris-towns`.
  - Superseded for the small boats by survey T7 ("you can use any boat you
    find"; task 4.2b), and for the cog by `sail-the-cog` (T9).

## Decisions

**1. `tenebris-towns` is built first, as written, and this change only
places it.**
- Its tasks 1 to 5 are this change's group 1, done in its own change folder so
  its history stays whole. This change's design does not reopen the cut, the
  stairs or the collision.
- Its questions are answered by the owner before group 1 starts. Two are
  answered by the request:
  - towns are both generated and hand-placed, through `city-sites`'
    overrides;
  - the player builds with the same pieces (`player-building`).

**2. A layout is charted onto the sphere through the neighbour tables.**
- A template is authored on an axial grid, as the mockup's is.
- At a site, the anchor cell is axial (0, 0). The chart maps axial (q, r) to
  a cell by walking the neighbour table from the anchor: q steps along one
  edge direction, then r along the next, with the direction rotated by the
  site's variation.
- There is no pentagon within the footprint plus two cells, so every cell in
  the chart has six neighbours in consistent order. The walk is well defined,
  and two layout neighbours are sphere neighbours. This is tested on every
  site of the shipped seed.
- Every piece is then cut from its cell's real corners (`tenebris-towns`
  section 6), so the ±9% geodesic spread is absorbed by the stair's tread, not
  by a gap.
- *Alternative:* project the flat layout onto the tangent plane and take the
  nearest cell per point. Rejected: near the footprint's edge two layout
  cells can land on one sphere cell, or skip one.

**3. The ground is generated, not edited.**
- The column generator asks the world's settlement ground, a pure function of
  the stored settlement records, whether a column lies in a footprint or
  margin:
  - in a footprint, the surface is the layout's terrace layer;
  - in the margin (a band of cells set per kind), the surface eases from
    the terrace to the natural height, one layer a cell at most;
  - trees and clutter read a "cleared" bit in the same answer.
- Water rows in a harbour keep the real sea. The layout places its quay where
  the terrace meets the sea.
- **Hollows are part of the answer too.** The owner (survey T13, 2026-09-27),
  on where a cave town's chamber comes from: **"recommended"**, which is "the
  town digs its own chamber when it is placed, saved with the town".
  - A settlement record can hold hollows: cells whose column is solid below a
    floor layer and above a roof layer, and empty between.
  - A cave town's hollows are its chamber (up to 93 by 37 m, a 10 m roof),
    its tunnel (3 m high) and its carved rooms. They also include the one cell
    of its daylight shaft, open to the sky.
  - A cliff village's hollows are its rock-cut rooms under the terrace above,
    2.6 m high under 0.4 m of rock.
  - The column generator gives such a column two solid runs, the ground below
    the floor and the rock from the roof up to the natural surface. Nothing
    is edited or written to the journal. The rock over the chamber is the
    mountain's own, so the roof is as high as the site's rock
    (`city-sites` decision 5 checks there is 16 m of it).
  - Because the hollow is in the stored record, it never moves or refills,
    and a changed template does not reshape a cave town already made.
- *Alternative:* stamp the town as a batch of edits in the durable log.
  Rejected: that is about ten thousand cell edits per town, written the first
  time a player comes near, for something the stored record already
  determines.

**4. Variation is a seeded choice among layouts that pass the checks.**
- A site's seed picks the rotation (one of six), the mirror, each building's
  kit from its biome's list, and the plots left empty (up to a fifth).
- The layout checks from `tenebris-towns` (headroom, roof overlap, landings,
  furniture, coplanar faces) run on the result.
- A failure tries the next variation from the same seed, in a fixed order. The
  identity variation is last, and is proved to pass at build time.

**5. Towns are pieces in packed arrays, and a town in range is an active chunk.**
- Pieces are not entities (CLAUDE.md). A settlement in range becomes one
  entity, holding:
  - its piece arrays;
  - its thin-solid index for `stand`;
  - its meshes, built per settlement on the pool and published as a complete
    generation;
  - its emitter list.
- Out of range it is only its far form.

**6. Three forms, faded.**
- Pieces near, a reduced form (walls as boxes, roofs as their planes) beyond
  that, and at night a point list of lit windows and lanterns at any
  distance.
- Each change of form is a dither-fade on the same distance band terrain
  detail uses. It is tested by the same frame-by-frame check the
  `detail-fade` work uses, so it cannot pop where the terrain does not.
- The night point list is drawn by the planet pass. Orbit sees it without
  the settlement being loaded.

**7. Light.** The owner (survey C3): "I also hope to see the lights on
cities contribute to LOD hexes on the night side of the world too".
- Street and wall lanterns are `lamps-and-lanterns`' dusk-lit materials,
  placed by the template at the spacing its kind sets (the mockup's rule:
  densest in the walled town).
- A lit window is a new dusk-lit `WindowCandle`, placed in the room behind
  about half the windows, chosen by the site's seed. It lights the room, and
  the room's light shows through the pane.
- Hearths are always lit.
- **Far off, the lights light the ground, not only themselves.** Each
  settlement's lit windows and lanterns add a warm term to the coarse LOD
  hexes within its footprint and margin. The term is baked per settlement
  into a small per-hex glow value, from the same emitter list, so the night
  side from orbit shows lit patches of ground under each town as well as its
  points of light.
- The field bakes all of them as any other emitter. The bake-time measurement
  from `lamps-and-lanterns` task 5.5 decides whether the tier needs splitting
  before this lands.

**7a. Rooms are lit by their own fires (2026-09-30).** The owner, on the
`sun-shadows` shots: "did you add interior lights to the game like the
mockup? ... implement that!" The mockup's rooms are warm at every hour,
because a hearth and the stairs' sconces burn all day and candles burn behind
about half the windows by night (`docs/mockups/towns.html`, `hearth`,
`sconce`, `blockLighter`). They are placed and lit as the mockup does them:
- **What the cutter places**, from the building's definition, derived and
  never saved:
  - **A hearth**, in the chimney cell, against the first of its outer walls
    that has no door or window, trying edge 0 first as the mockup's
    `hearth(c1, front, 0)` does. It is a stone hearth 0.7 m deep and 1.4 m
    wide, with cheeks, a hood to the ceiling, logs and a fire. A chimney cell
    that holds the stair has none. Flat-roofed kits have no chimney, and the
    mockup's clay oven is later.
  - **A sconce** on a straight flight's boxed side, a quarter of the way
    up, 2.45 m over its foot. On a newel stair there is one a storey, 2.2 m
    over the tread, on the first of the stair's own walls without a doorway.
    Each sconce is an iron bracket with a flame.
  - **A candle** behind 55% of the windows (the mockup's town share),
    0.8 m inside the window, 0.2 m over its sill. Which windows is a hash of
    the window's place in the building's definition, so a town's candles are
    the same on every load. A candle has no mesh, as in the mockup: only its
    light shows, on the room and through the pane.
- **How a light lights**, the mockup's `blockLighter`:
  - a building's lights light only that building's room faces, and only
    within the light's own storey;
  - `p * (1 - (d/R)^2)^2 * (0.3 + 0.7 * max(n.l, 0)) * d^2 / (d^2 + 0.36)`;
  - a face turned away past -0.15 is left unlit.

  Hearths burn all day, 0.9 + 0.3 of the night. Sconces burn all day as
  hearths do. Candles burn only at night, 1.8 times the night past 0.25 over
  0.35.
  - Colours are the mockup's, in linear light: hearth `#ff9a4a`, sconce
    `#ffb870`, candle `#ffb060`.
  - Reaches are 7, 5.5 and 5 m, and powers 1, 0.75 and 0.45 to 0.8.
- **Not through the voxel field.** The field cannot see a house's walls, since
  a building is pieces and not voxels. A hearth baked into it would shine
  through its wall into the lane. Each building's lights, at most 24, ride in
  its rooms' material and are summed per pixel in `field_lit.wgsl`.
- **The flames are drawn unlit**, warm. A candle's light, and nothing else,
  waits for dusk.
- **As built (2026-09-30).**
  - The flames are drawn in timber-bracketed sconces rather than the
    mockup's iron. Iron under its own flame took none of its light and read
    as a black box.
  - The flames' colour is the terrain lantern fire's, kept under the
    tonemapper's shoulder.
  - Each building carries at most 24 lights: a Holbrook house has 22 to 29
    windows, so 12 to 16 candles.
  - The weight against the sky's fill is the mockup's own
    (`sun-shadows`, "Tuning across the day").
- **Not in 7a:** the door and street lanterns and the mockup's glow from a lit
  window into the street. Those light the ground, which is the terrain's, so
  they stay with the voxel field (task 5.2).

**8. A town is a stored record, and its buildings are definitions, not
pieces** (the owner's save model, `world-persistence` decisions 1 and 2).
- When a world is made, every site's settlement is generated: the chart, the
  variation, and each building. The result is written to the record store
  before anything is shown.
- A building is stored as its own definition: plot cells, walls and their
  openings, stairs, storeys, kit, roof and state. A cave town's or cliff
  village's hollows (decision 3) are stored with the settlement, cell by cell
  with their floor and roof layers. It is not a reference to a
  template. A revised template changes new worlds only, and a town already
  made keeps what it was built as.
- Pieces are derived from the definitions by the cut rules each time a town
  is built into the world, so a fix to a stair or a sill reaches every town.
- A building's state is standing, abandoned or ruined. Only standing is
  generated. An abandoned building keeps its window candles and hearth dark.
  This is the one piece of the owner's "towns can grow, buildings can be
  abandoned" built here, and it is exercised by `world-persistence`'s
  test-only process.
- A site is left unsettled if the journal holds a player-authored entry
  within its footprint or margin from before its settlement record was made.
- *Alternative:* keep every template version forever and regenerate towns
  from the version recorded in the save. Rejected: a town that can grow or
  lose a house cannot be a pure function of a template, and the owner has said
  towns will change over time.

**9. Slices, each ending in shots of a town in the game (2026-09-29).** The
owner: "I thought you were building cities, I wnat shots of the cities". The
groups above stay the plan, but they are built in slices that each put
something new on screen, not in group order. The walker and contact
(`tenebris-towns` 1 and 2) no longer come first. They are the second slice,
so the first can be looked at before it can be walked into.
- **Slice 1, Holbrook stands (to look at).**
  - The village template is the mockup's `makeVillage`: nine houses, the
    village hall, two straw huts, the lane and the green. It is ported as
    data: each building's cells, kit, storeys, door and windows.
  - The template is charted onto the real finest cells round the home
    town's anchor (decision 2).
  - The ground is terraced under it and eased at its margin (decision 3).
  - The pieces are cut from each cell's real corners: walls on outside edges
    with their door and window openings, corner posts, the gable roofs and
    the huts' cones.
  - The mockup's own textures are used, exported as PNGs
    (`tools/export_town_textures.js`, `assets/textures/settlement/`).
  - Not in this slice: stairs, furniture, lanterns, collision (you can see
    the houses and not yet enter them), the far form, and the other eight
    kinds.
- **Slice 1, how it hooks into the engine (measured 2026-09-29).**
  - **One height function.** Every height the planet has comes from
    `pbd_core::column::surface_m`:
    - the voxel columns;
    - every LOD record, through the app's `surface_height`;
    - the contact field, which reads the records and columns;
    - the map's raster.

    No shader computes a height. So the settlement ground is consulted
    there, and only there. It is a registry beside the terrain config's own
    global: `pbd_core::settlement::ground`, set when a world's towns are
    known, and keyed by seed and generator so it never reaches another
    world.
  - **The ground's answer for a direction** comes from the town's own cells
    (the finest cells round its anchor, from the lattice's cap search):
    - in the footprint, the terrace layer;
    - in the margin, the natural height held within `k` layers of the
      terrace at ring `k`;
    - elsewhere, nothing.

    A direction is matched to its cell by the nearest centre, through a
    bucket grid in the town's tangent plane. Every other direction pays one
    dot product per town.
  - **The terrace** is the footprint's median natural layer, so the terrace
    cuts as much as it fills.
  - **The ground's top.** A lane's top is dirt, and a building's cells are
    dirt under its floor. Every footprint cell carries a `cleared` bit in its
    record's material word (bit 24). The foliage pass, the one place a tree
    is chosen, grows none there.
  - **The planet is rebuilt round the player once the ground is set**, as a
    generator switch rebuilds it. The map's raster cache is named with the
    ground's digest, so a cached map never shows the ground from before.
  - **Drawing.** The town is one entity with a child mesh per texture. Each
    texture is a `StandardMaterial` over the exported PNG, sampled nearest.
    The root is `LitByField`, so the town takes the sun, the sky and the
    lamps as a drop or a craft does. Positions are planet-local, offset by
    the planet's render frame.
  - **Captures.** `--at <lat> <lon>` puts a new world's walker there, not at
    the level start, so a shot can stand in a town. The existing `--yaw`,
    `--pitch` and `--height` frame it.
    - **Found on the first shots (2026-09-30).** `--at` still stood the
      walker 4 m west of the spot.
      - A new world steps 4 m aside from its start, so that a tree's trunk
        does not fill the first view. `--at` inherited that step.
      - The inside shot then faced a wall 4 m from its door.
      - `--at` now stands exactly where it is asked. The new world's own
        start keeps its step.
    - **`--rain` is a storm forcing, not the weather.** `--rain 0` forces
      nothing, and the atmosphere's own weather still rains. A dry shot
      picks its time with `--weather-at <seconds>`.
      `towns::tests::print_the_rain_over_holbrook` reads the rain over
      Holbrook along the same path the capture takes.
      - Measured 2026-09-30, at 11:00 on days 0, 1 and 2, with the weather
        run 0 to 5 hours past the clock:
        - it rains at every start (1.00);
        - it is dry after two hours and after three (0.00 on each day);
        - it rains again after four.
      - `tools/capture_holbrook.sh` now takes its shots at `--weather-at
        7200`. That runs the weather two hours on and leaves the time of
        day at 11:00.
- **Slice 1, found on its first shots (2026-09-29).** The owner: "hmm...they
  dont seem to quite follow the same rules as the js prototype project", and
  "the roof shouldnt extend pass the floor plan like that".
  - Measured, not guessed: a fieldstone house's plan in its frame was
    7.65 x 9.93 m, where the mockup's is 9.92 x 5.72.
  - The frame's row direction had been taken from the building's centre
    toward its first cell's edge, not from that cell's own centre. That
    turned each frame up to 60 degrees off the rows. The roof, laid over the
    plan's box in that frame, sat crossways over the house.
  - Fixed. Every plan is now the mockup's within the cells' own spread:
    9.40 x 6.05 m for that house. `settlement::tests::every_plan_is_the_mockups_plan`
    holds every building of the village to within 12% of the mockup's
    plan, both ways.
  - The roof's plan is now the mockup's rule: its box, with a 0.45 m eave,
    and a gable end over an odd row's half-cell. Anything past that is a
    difference from the mockup, and is shown beside the mockup's own view of
    the same house.
- **Two empty columns between neighbouring houses (the owner, 2026-09-29).**
  "also those two buildings are too close to eachotehr, yea".
  - The village paired its houses one empty column apart:
    - walls a cell apart (2.8 m);
    - roofs 0.5 m apart, eaves included (`tenebris-towns` section 2's "one
      empty column ... is enough").
  - On the sphere the cells' spread took that half metre. Measured between
    one pair: their centres were 1.36 cells apart, not 1.5, and their eaves
    met (`settlement::tests::no_two_roofs_cut_into_each_other`).
  - Every pair along the lane is now two empty columns apart: walls 5.7 m,
    roofs 3.3 m. The outer house of each pair moves one column outward.
  - The change is made in the mockup's own `makeVillage` and exported again,
    so the prototype and the game stay one layout.
- **How Holbrook differs from the mockup's village (the owner, 2026-09-29:
  "how is this different than the js prototypes?").** Shown side by side
  in `docs/screenshots/cities-in-the-world/`.
  - The same: the layout (every building's cells, doors and windows,
    exported from the mockup), the kits, the textures, the storey and wall
    sizes, the roofs' rule, and the lanes.
  - Different by design:
    - The ground is the planet's own. The mockup paints a flat field with
      a hill by the mill; the game terraces the real ground under the
      footprint and eases it back over a margin (decision 3).
    - The cells are the sphere's, which vary about 9% either way. Each
      plan is within 12% of the mockup's.
    - The light is the game's sky and field light, not the mockup's lamps.
  - Not built yet, each in a later slice:
    - Stairs, furniture and hearths, and doors that open and shut (2b). A
      door is drawn open, swung in against its jamb.
    - Lanterns, lit windows and the night (slice 3).
    - What the mockup builds that is not a building: the smithy's shed,
      the well, the windmill, the crops and fences, the pond, the green's
      tree, and the people. The game's own trees grow round the town and
      are cleared from its footprint.
- **Slice 2, walk in.** Thin solids, the walker's rules, the stairs and
  doors (`tenebris-towns` 1, 2 and 5). It lands in two steps.
  - **2a, walls and doorways (written 2026-09-29, before the code).**
    - Every wall segment, post, sill-free jamb and chimney the cutter makes
      is also a **thin solid**: a convex outline in its building's plan with
      a height range (`tenebris-towns` section 4's first primitive).
    - A town's solids live beside its meshes, indexed by a bucket grid in
      the town's tangent plane.
    - The walker's swept step treats a solid its body overlaps as it treats
      a rise it cannot step. A solid overlaps when:
      - its outline, grown by the body's 0.3 m radius, holds the body's
        centre in plan;
      - its height range crosses the body's;
      - and the feet are not already on top of it.

      The tangential motion stops at the last clear point. Sliding along the
      face is the design's first rule, and comes with the rest of the walker
      rules in 2b.
    - **Doorways are open.** A door leaf is drawn swung inward against its
      jamb, and is no solid, until doors open and shut as world state (task
      5 of `tenebris-towns`, with the save).
    - The ground floor is the terrace, which the column tier already
      answers. The upper floors' slabs are ceilings to it: 2.8 m clear, a
      metre over the walker.
    - Verify: a core test that a point in a wall is blocked, a doorway is
      clear, and a point on the wall's top is not blocked. An app test
      walks the walker at a house: it stops at the wall, and goes in
      through the door. Then a capture from inside a house, looking out of
      its door.
    - **As built (2026-09-29).**
      - Walls, posts, chimneys and the upper floors' slabs are solids
        (`settlement::pieces`). The walker's `Structures` resource holds
        each building's solids, and `resolve_ground` asks it at every swept
        step.
      - Each building is first tested against its plan's reach, and its
        solids only when that passes. For one village of 13 buildings that
        stands in for the bucket grid, which waits for towns large enough to
        need it (slice 4).
      - The slabs and the door lintels are ceilings. The walker's jump rises
        about 2.7 m, which would carry its head through the floor above. So
        the lowest solid underside over the body joins the terrain's
        ceiling, and the head stops there as it does under a cave roof.
      - A body already inside a solid, because a town was built round it,
        is let walk out rather than held.
      - Not yet, and left for 2b: roofs are not solids. A hut's walls are
        2 m, so a jump from beside one lands on the wall's top and walks on
        over it.
  - **2b, the rest of the walker's rules (written 2026-09-30, before the
    code).** The owner: "commence 2b". Four parts, each from
    `tenebris-towns` sections 3 to 5 and 8, with the game's cutter porting
    the mockup's pieces.
    - **Floors from the town, not only walls.**
      - Today only the terrain holds the walker up: a town's solids stop
        it or cap its head, and an upper floor holds no one.
      - Each building gains *surfaces* beside its solids (`tenebris-towns`
        section 4). A surface is a region in plan whose top is a function
        of position, and it answers its underside too:
        - every upper floor, flat, over its cells less the stair well;
        - a straight flight, on its pitch line;
        - a newel stair, one sheet a turn, on its pitch line.
      - At each of its footprint points the walker takes the higher of the
        terrain's floor and the highest surface top within its step of the
        feet. The MAX floor over the footprint is kept.
      - A surface's underside joins the solids' as a ceiling. So a floor
        overhead is a ceiling, and the turn above a newel's walker leaves
        2.74 m.
    - **The stairs, cut from the stored definition** (derived, decision 8).
      The template gives only a building's stair cells. The kind, and where
      the stair starts, follow the mockup's `townHouse` rule:
      - one cell is a **newel stair**:
        - 15 winders a turn of 0.2 m, a turn a storey;
        - it starts on the first edge whose neighbour is in the building
          and is not the front door's cell, and leaves every upper storey
          by the same edge (3 m is 3 layers, so exit = entry);
        - a 30-degree landing at the top, then a rail;
        - walls on its inner edges, with a doorway at the foot and at each
          exit;
      - two cells in a row are a **straight flight**:
        - from the first cell's far flat to the second's, 16 risers of
          0.1875 m;
        - boxed below by walls either side;
        - the well railed up top, open at the landing;
        - the two cells floored upstairs only outside the flight's strip.
      - No floor is cut over a stair cell but those strips' triangles.
        Today's cutter floors every cell, which would roof each stair.
      - The newel turns toward the edge numbered next after its entry, as
        the mockup's does. In the game the angle is measured from the real
        edge midpoints, so the sense holds whichever way the chart turned.
    - **The three walker rules** (`tenebris-towns` section 4):
      - **The pitch line.** A stair's top at a point is the line from the
        foot of its first riser to the nosing of its landing, so the eye
        climbs at the stair's slope, with no jump.
      - **Held down 0.35 m.** A walker that was grounded, and is not rising
        from a jump, is held to a floor up to 0.35 m below. It comes down a
        stair without leaving it. A terrace drops a whole layer (1 m), so the
        terrain is unchanged: a walker still steps off a ledge.
      - **A refused move slides.** A body stopped by a wall, a door, a rise
        too tall to step or a passage too low keeps the part of its move
        along the face. It is swept again along that part, twice at most,
        for a corner.
        - A solid's normal is from its outline's nearest point to the body.
        - The terrain's is the fall of its floor across the body.

        A hex town's walls zigzag at 60 degrees, so a walker that stops
        dead catches on every corner.
    - **Doors open and shut, and are saved** (`tenebris-towns` section 8
      and task 5).
      - A door's leaf is its own entity, hinged at its jamb. Shut, it stands
        in the doorway and is a solid. Open, it lies swung inward against
        the wall and is none.
      - It swings inward. The mockup turns a leaf outward where furniture
        blocks its sweep, and the game places no furniture yet.
      - **E** opens or shuts the door in reach: the nearest doorway within
        2 m of the eye, in front of it.
      - **Saved as a record.**
        - The kind is `door`, schema 1, with the body `(open: bool)`.
        - Its id is its building's record id times 16, plus the door's
          number in the building.
        - It is written as the player's (`Author::Player(0)`) through the
          durable path.
        - No record is a shut door. Opening the world reads the records.
        - A world process that later shuts a door the player opened is
          refused, as a player-owned field is (`world-persistence`
          decision 6).
      - Doors start shut. That is my recommendation, taken because a
        question goes to the owner only with screenshots. Until now a door
        was drawn open and was no solid. A world played on that build
        finds its doors shut. Nothing it made changes.
      - A capture opens every door with `--open-doors`, as the player
        would. The inside shot looks out through its door as before.
    - **Not in 2b**, and next with the speeds (`tenebris-towns` task 2):
      - run 5, walk 3, sprint 8, crouch 1;
      - walking under a roof;
      - Caps Lock.

      The walker keeps today's 8 m/s walk and 14 m/s sprint. The mockup's
      stair walks were measured at 8.
    - **Verify.**
      - Core: a flight answers its pitch line, from the foot to the landing;
        a newel answers one sheet a turn, and the 2.74 m under the next;
        no floor over a stair cell but the flight's side triangles; a shut
        leaf is a solid and an open one is not.
      - App, on Holbrook's own houses, as `tenebris-towns` section 5 did in
        the mockup:
        - up and down a newel and a flight at the walking speed: no eye
          jump over 0.1 m in a tick, and not one tick in the air coming
          down;
        - along a wall at 8.6 degrees for a second, the walker slides on;
        - a shut door stops the walker, and the same door opened lets it in;
        - E opens a door, the record is on disk, and a reopened world has
          it open.
      - Captures, with the mockup's same view beside each: upstairs in a
        Fieldstone house looking down its newel; on a half-timbered
        house's flight; a shut door, and the same door open.
    - **As built (2026-09-30).**
      - `pbd_core::settlement::pieces`:
        - `Surface` (floor, flight, newel) and `DoorLeaf`;
        - `BuildingSolids::stand` and `push_normal`;
        - the stairs cut from the stair cells;
        - the wells left open.

        `settlement::record` adds the `door` records.
      - The walker (`walking.rs`):
        - `footprint_in` takes the town's floors;
        - `sweep` holds a grounded walker to a floor up to 0.35 m below;
        - `resolve_ground` slides twice at most.

        One existing test changed with the rule it pinned. Pushing straight
        into a terrace, the walker now slides along its face at 0.62 m/s:
        8 m/s by the sine of its 4.4 degrees off square. The test now holds
        how far it gets *into* the terrace.
      - Doors (`towns.rs`):
        - `TownDoor` entities under the town's root;
        - E through `use_doors`, which writes the save before the leaf moves;
        - `swing_doors` swings the leaf at 5 rad/s;
        - `door_states` reads the records when the town is built.
      - Captures: `--open-doors` opens every door without saving it, and
        `--up M` stands the walker on the highest town floor within M
        metres of the ground.
      - Tests:
        - core: `every_stair_is_cut_and_nothing_floors_its_well`,
          `a_flight_answers_its_pitch_line`, `a_newel_answers_a_sheet_a_turn`,
          `a_shut_door_holds_and_an_open_one_does_not`;
        - app: `a_walker_climbs_a_newel_and_comes_down_it_on_its_pitch_line`,
          `a_walker_climbs_a_flight_and_comes_down_it_on_its_pitch_line`,
          `a_walker_brushing_a_wall_slides_along_it`,
          `a_shut_door_stops_the_walker_and_e_opens_it_into_the_save`,
          `towns::tests::a_door_opened_is_open_when_the_world_is_opened_again`.

        The stair walks use the village's own houses, cut on the flat test
        land 25 m off its pentagon.
      - The village's newels climb one turn, so over the foot is the 30
        degree landing. The clearance under it is 2.65 m, not the 2.74 m
        under a winder. Both are well over the 1.8 m body.
- **Slice 3a, the town is a stored record (written 2026-09-30, before the
  code).** Task 4.5's storing half, taken ahead of 2b. PR #19 merged slice 1
  and 2a to `main` on 2026-09-29. Every world that build opens gets Holbrook
  built from the template, fresh each time. So the next re-export of the
  village would move houses in worlds already played. 3a stops that. Until
  it lands, the village template is not exported again.
  - **What is stored.** Decision 8 split three ways:
    - **Stored:** what the town was laid as.
    - **Derived:** what the rules make of it each time.
    - **Generation:** what the ground makes of it, pinned like the
      generator.
  - **Two record kinds, schema 1** (`pbd_core::settlement::record`):
    - **`settlement`, one per town, id = its site's id.**
      - `template` and `layout`: the template it was laid from and the
        laying-out rules' version. Both are kept to be read by people,
        never to rebuild the town.
      - `terrace`: the terrace's layer, a whole number of metres over the
        radius.
      - `cells`, one entry per footprint cell (built cells and yard rings):
        - its layout cell `(c, r)`;
        - its exact cell key;
        - which of its sides is the layout's direction 0;
        - the top it takes (dirt on a lane, none elsewhere).
      - `buildings`: its building records' ids, in order.
      - Written last, so that a write torn by a crash leaves no settlement
        and the town is made again whole. The site list does the same.
    - **`building`, one per building.**
      - Its id is the site's id times 65 536 plus its number in the town.
      - It holds the template's building as it was, in layout cells: kit,
        cells, storeys, tall storeys, doors, windows, roof, pitch, chimney
        and stair cells.
      - The ground floor is stored as `floor`, in layers over the terrace.
        The mockup's datum is gone from it.
      - It holds a `state`, which is only ever `Standing` when a town is
        made. Abandoned and ruined are for `world-persistence`'s process
        and the night lights (slice 3). They read it later without a new
        schema.
  - **Why the chart's cells, and not only the anchor.** The town stands on
    the cells it was laid on, whatever a later chart rule would walk to.
    - The neighbour walk (decision 2) runs once, when a town is made.
    - Storing only the anchor and its side would bind every future chart
      rule to reproduce every old town. The walk would then be generation
      code, carried forever.
    - Measured on the village (`settlement::tests::a_towns_records_are_small`):
      - the footprint's 799 cells take 18.5 KB, 23 bytes a cell;
      - its 12 buildings take 5.8 KB;
      - so the town is 24 KB in all.
      - Fifty such towns would be 1.2 MB, written once each. That is more
        than the risk below guessed, and it is what the cells cost.
    - A cell's key is exact (`exact-cell-keys`). Its side numbering is the
      topology's, which is part of the world's identity.
  - **Derived, never stored.** A fix to any of these reaches every town,
    which is decision 8's point:
    - the pieces, cut from the definitions (`settlement::pieces`);
    - the solids;
    - the meshes and textures;
    - the kits' faces and sizes, looked up by the kit's name. A kit a
      saved town names stays in `kits.ron` for good, as a generator
      version does. A test holds every stored kit name to a kit.
  - **Generation, pinned.** The ground is terrain: the terrace over the
    footprint, eased over the margin to the natural height. The margin's
    rings come from the stored terrace and footprint and the world's own
    generator. A test pins Holbrook's ground digest on the shipped seed, as
    a generator version's ground is pinned. A change to the easing is a new
    rule for new towns, never a reshaping of a made one.
  - **Made once, before it is shown.** The home village is laid out and
    stored once the world's sites are on disk (`WorldSites::ready`). If the
    world's save holds no `settlement` record for that site:
    - the town is laid from the template (decision 2's chart, the terrace,
      the footprint);
    - its records are queued as `Author::Creation`, buildings first;
    - the kinds are named in the identity.

    The town is built into the world only once the writer's mark has passed
    the settlement's line, as the site list waits. A world that holds the
    record is built from it, and the template is never read.
  - **One path.** A town just made is built from its records as well, not
    from the template it was laid from, so the town shown is always the
    record. A test holds the two to the same pieces.
  - **A damaged record is not remade.** A settlement record that is there
    but does not read is left alone, and the log says why. The same goes
    for a schema this build does not know, or a building it names that is
    missing. That town is not built, and nothing is written over it.
  - **Old worlds** get their record at their first open by a 3a build, from
    that build's template. That is the town the merged build already showed
    them. The rule that leaves a site unsettled where the player dug first
    (task 4.4) is not applied to the home village: the merged build already
    stood Holbrook in every world it opened. It applies from slice 4, to
    every site that no build has settled before. The player's edits are kept
    at their layers in either case (`column::BASE_M`). Recommendation taken
    (ask only with screenshots).
  - **Verify.**
    - Core tests:
      - Holbrook's records round-trip;
      - a town built from its records has the same pieces, solids and
        ground as the one laid from the template;
      - a moved door or a dropped building in the template leaves the
        town rebuilt from a made world's records unchanged;
      - the records' size;
      - the ground digest pinned.
    - App tests:
      - a new world stores its town once;
      - a second open writes nothing;
      - a world opened with a changed template builds the stored town;
      - a damaged settlement record builds no town and is not overwritten.
    - Screens: a capture of the lane, before and after, which must match.
  - **As built (2026-09-30).**
    - `pbd_core::settlement::record`:
      - `lay` lays a template into a `Town`;
      - `build` cuts a `Town` into its chart, ground, meshes and solids;
      - `to_records` and `from_records` store and read it;
      - `Stored` answers none, a town, or damaged.

      The app's `towns` module stores the town (`ensure`) and waits for the
      writer's mark, then builds it from the records (`stand`).
    - Holbrook on generator 6:
      - the terrace is 74 m;
      - the footprint is 799 cells, eased over a margin of 2051;
      - `towns::tests::holbrooks_ground_is_pinned` holds its ground digest.
    - `settlement::tests::a_town_built_from_its_record_is_the_town_its_template_lays`
      builds the ground as slice 1 laid it, straight from the template, and
      holds the record's ground to it. So a world the merged build opened
      keeps the ground it had.
    - The other tests:
      - core: `a_town_round_trips_through_its_records`,
        `a_revised_template_leaves_a_made_town_as_it_was`,
        `a_damaged_settlement_record_is_named_not_remade`,
        `a_towns_records_are_small`,
        `every_kit_a_saved_town_can_name_is_shipped`;
      - app: `a_world_stores_its_town_once_and_keeps_it_when_the_template_changes`,
        `a_damaged_settlement_is_neither_built_nor_written_over`.
    - The capture is taken from a new memory-only world, whose town goes
      through the same store and wait.
- **Slice 3, lit, stored and seen from afar.** Lanterns and candles (group
  5), settlements as records (task 4.5), and the far form and the night
  points (4.2, 4.3).
- **Slice 4, every kind (written 2026-09-30).** Asked how many towns were
  built, the answer was one, the home village. The owner then said: "alright
  once your tuning is done, begin working on other towns".
  - **Where it stands (measured 2026-09-30).** Each world stores about 55 sites:
    20 villages, 6 walled towns, 6 harbours, 6 jungle, 4 desert, 4 tundra,
    4 cliff, 3 swamp and 2 cave. Only the home village is laid and built.
    The mockup has ten settlements. Against the cutter as it stands:

    | Kind | Mockup | `building()` calls | Cut as-is | New |
    | --- | --- | ---: | ---: | --- |
    | Walled town | `makeTown` | 29 | 29 | terraces at 0 to 3 m, street steps, curtain wall and gates, 2 towers, the keep; clay houses lack their parapet, the exchange its columns |
    | Harbour | `makeCoast` | 18 | 3 | whitewash and driftwood kits, terraces, piers on piles, open-sided boathouses, stilts, the boats (task 4.2b) |
    | Desert | `makeDesert` | 11 | 0 | sandstone and adobe kits, walkable flat roofs with parapets, domes, outdoor stairs, the oasis |
    | Mountain | `makeMountain` | 14 | 0 | the alpine kit, terraces at 3 to 15 m, switchback stairs, rock-cut rooms (hollows, task 3.1a), the gorge bridge |
    | Tundra | `makeTundra` | 1 | 0 | the granite kit, igloos (a dome not cut to the cell, with its tunnel), the ice keep and wall (4c's masonry) |
    | Swamp | `makeSwamp` | 6 | 0 | the alder kit, stilt floors and piles, decks, boardwalks, the bayou's water |
    | Jungle | `makeJungle` | 5 | 0 | the jungle hut kit, platforms 9 m up round kapok trunks, rope bridges, the pole tower |
    | Caves | `makeCaves` | 0 | 0 | hollows (3.1a): chamber, tunnel, carved rooms and the shaft; lights that burn all day |
    | Mounds | `makeMounds` | 0 | 0 | turf domes cut by a plane, round doors, the vaulted back room; no site kind yet |

  - **Four things break before any new piece is written.**
    - **Heights are whole metres.** `BuildingDef.base` and `GroundCell.h`
      are `i32`, and the tundra, swamp, jungle, harbour and cave templates
      carry fractions.
    - **The footprint is read from a cell's top.** Every top that is not
      grass or sand counts as built. Outside the fields nearly every cell
      of the 50 × 34 grid would become footprint. It is read from the
      mockup's areas instead (plot, street, building).
    - **One terrace a town.** `record::lay` stands the whole footprint on
      its median layer. The walled town, the harbour, the mountain and the
      mounds are terraced at several levels.
    - **Eight kits are missing** (sandstone, adobe, granite, jungle hut,
      alder, whitewash, driftwood, alpine), with the dome roof and the flat
      roof's parapet.
  - **The order.** Sub-slices, each ending in shots of a town in the game.
    Recommendation taken (ask only with screenshots).
    - **4a, every village stands.** The plumbing every other kind needs:
      towns at sites that are not home, laid and stored on the world's first
      open, standing in range. No new pieces.
    - **4b, the walled town's streets and houses.** All 29 of its buildings
      cut as they stand, once heights are fractional, the footprint comes
      from areas, and a town stands on several levels.
    - **4c, the walled town's masonry.** Curtain wall, gates, towers and the
      keep. The tundra's ice keep and wall reuse them.
    - **4d, the harbour.** Two kits, 4b's levels, and piers on piles at the
      real sea.
    - **4e, the desert.** Flat roofs that can be walked, parapets, domes and
      outdoor stairs.
    - **4f, the mountain.** Its terraces are 4b's. Its rock-cut rooms are
      the first hollows (task 3.1a).
    - **4g, the tundra.** Igloos are the first dome not cut to the cell.
    - **4h, the swamp; 4i, the jungle.** Raised floors, decks, boardwalks,
      then platforms and rope bridges.
    - **4j, the caves.** The whole of 3.1a.
    - **4k, the mounds.** Turf domes. A share of the fields' village sites
      take the mound template, chosen by the site's seed (the parity list,
      row 35). A village already stored keeps its template.
    - The order follows what is reused. The plumbing comes first. The walled
      town's houses need no new piece. Its masonry serves the tundra. Levels
      serve the harbour, the mountain and the mounds. Hollows serve the
      mountain before the caves. Curved and raised work comes last.
  - **4a in detail.**
    - **Which sites.** Every site whose kind has a shipped template: the
      villages now, with each later sub-slice adding its kind.
    - **When they are laid.**
      - All of them are laid on the world's first open with this build, in
        site id order, and before any town stands. A new world lays them at
        creation.
      - Each is stored through the durable path as Holbrook is (slice 3a),
        with the settlement record last.
      - Laying and cutting a village took 0.30 s in a debug test
        (`a_town_built_from_its_record_is_the_town_its_template_lays`). The
        release time is measured on the first 4a build and recorded here.
    - **Unsettled sites (task 4.4).** A site with a player's edit in its
      footprint or margin is not laid. It is stored as unsettled, so it is
      never tried again. A world made before towns keeps what the player
      did there (CLAUDE.md, "Saved games survive every change").
    - **One ground, installed once.** Every laid town's ground is installed
      together when the world's towns are read, and the planet is rebuilt
      once. `settlement::ground` already holds a list of towns. A height
      that is in no town pays one dot product a town (55 at most). An
      instrument times `column::surface_m` with no towns and with twenty,
      and the cost is recorded.
    - **Standing in range (decision 5, tasks 4.1 and 4.3).**
      - A town's pieces are cut, spawned and given to the walker only
        within 1.2 km of the camera, and dropped past 1.5 km.
      - They are cut on the task pool and published whole.
      - A town fades in and out with a dither over a second: it never
        appears or vanishes in one frame (priority 1, no pop-in).
      - Its ground is there at every distance, so its terraces and lanes
        show before its houses. The far form (task 4.2) is later.
    - **Per town.** The walker's solids, the doors, the shadow casters and
      the room lights come from the standing towns only.
    - **Rotation (decision 4, in part).**
      - The home village keeps its layout's east as it is.
      - Every other village turns by its site's seed, to one of six sides.
      - The mirror and the empty plots come with task 2.2.
      - A stored town never turns: it is built from its records.
    - **Shots.** Two other villages at 11:00 and 22:30, and the walk from
      one village to the next, where a town fades in on the way.
  - **4a as built (2026-09-30).**
    - **Laid and stored.**
      - `towns::start_towns` lays every village site with no record,
        in site id order, through `ensure`, and stores each.
      - A world opened in the fast build lays all 20 villages in 0.44 s,
        about 0.04 s each (`every_village_lays_and_cuts_on_its_own_ground`,
        in the fast profile).
      - The ground is installed once for all of them, and the planet rebuilt
        once. That takes 8.4 s in the container, as Holbrook alone did.
    - **The turn.** `record::turn` mixes the site's id and deals the six
      sides about evenly: 6000 sites give 900 to 1100 each. The shipped
      seed's 19 other villages take all six. Holbrook keeps its east, and its
      lane shot is the same to the pixel.
    - **Unsettled.**
      - `TownGround::touches` looks for a player's edit on any footprint or
        margin cell, by exact key.
      - Such a site gets an `unsettled` record, schema 1. It reads back as
        `Stored::Unsettled` and is never laid again, even once the edit is
        gone.
    - **In range.**
      - `stand_in_range` cuts a town on the pool within 1.2 km of the camera
        (the walker where no camera is active). It drops the town past
        1.5 km, and in between the town stays as it is.
      - The fade is `Faded` on the town's root, carried to every piece's
        material as `centre.w`. `field_lit.wgsl` discards through the
        terrain's own `bayer4`, and a test holds the two copies equal.
      - Flames are unlit, so they show once a town is half there.
      - A town that stands when the world opens is whole at once, as its
        ground is.
    - **Per town.** A door and a room name their site and their building's
      number there. `Towns::index` finds the building in the walker's
      `Structures`, which holds only the standing towns, in the order they
      stood. A dropped town's buildings are taken out, and the later towns
      move down.
    - **One height a town, not twenty.**
      - With every village's ground installed, a height cost 17% more than
        with none. The instrument timed 200,000 directions in the fast
        build: 1215 ns against 1417 ns. Every height asked every town.
      - The world's ground now keeps the towns by square on a grid over the
        cube's faces: 8 a side, about 1 km across here.
      - A height asks only the towns in its own square. Twenty villages now
        time within the instrument's run-to-run noise of one (-8% to +6%).
      - `the_grid_finds_what_every_town_would` holds the grid to a look at
        every town, on the town, round its edge and past it.
    - **The dark towns (found on the shots).**
      - The first shots of other villages from above drew some towns near
        black: every face turned from the sun, walls and roofs alike.
      - **The cause was the capture, not the town.** A still with no walker
        (`--view column`, every view from above) never added the field-lit
        plugin. Towns there were drawn with Bevy's own sun and its default
        ambient, the harsh look `sun-shadows` decision 7 replaced. The
        field-lit plugin is now added in every run.
      - **Found with it:** a town piece's bounding box is square to the
        render frame's axes, so where a town's up is tilted from them the
        box reaches into the ground. At Theringford two of each box's eight
        corners read the rock's dark (sky 0). A corner under the ground now
        reads the field half a metre over it, and all eight read the open
        sky there.
    - **Not captured.** The fade itself is a second of frames. A still
      capture shows its end, so the app test
      (`a_village_stands_as_the_walker_comes_and_is_taken_down_as_it_leaves`)
      is its proof until the owner's video batch.
  - **4b in detail.**
    - **The export.** `tools/export_town_templates.js town` writes
      `assets/settlements/v1/town.json`.
    - **The template takes fractional heights and areas.** Serde reads the
      village's whole numbers as before.
    - **A level a footprint cell.**
      - Each cell stands at the terrace plus its template height, rounded to
        a layer.
      - The level goes in the cell's record: schema 2, where a schema 1
        record reads as level 0 (CLAUDE.md, "Saved games survive every
        change").
      - A cell a layer below its neighbour gets a street step.
    - **Out of 4b.** The lake at -3 m is dry ground until a lake takes
      water. The wall, towers and keep are 4c. The stalls, well, smithy,
      jetty and lamps come later (task 5.2 for the lamps).
    - **Shots.** The walled town's lane, square and market from the
      street, and from 60 m.
  - **4b as built (2026-09-30), and where it left the plan.**
    - **The export.** `tools/export_town_templates.js town` wrote
      `town.json`: 29 buildings, 38 street lamps, and ground from -3 m (the
      lake) to 9 m (the hillside past the hamlet). Its buildings stand on
      whole layers, 1 to 3. So heights stay whole metres, and fractions come
      with the kinds that carry them (the tundra, swamp, jungle and
      harbour).
    - **The footprint is as it was.**
      - It is the built cells (buildings, and every top that is not grass
        or sand) and two rings of yard. On the walled town that is the whole
        walled town, its quay and its hamlet.
      - It is not the lake, and not the hillside or the fields outside the
        walls, which keep the planet's own ground.
      - Reading the mockup's area names was not needed. The village's
        footprint does not change.
    - **A level a footprint cell.**
      - `Template::terraced` (the exporter sets it for the town, the harbour,
        the mountain and the mounds) lays each built cell at its own height
        over the datum.
      - A yard cell takes the level of the built cell nearest it, so no yard
        follows the mockup's lake down or its hillside up.
      - `TownGround::terraced` eases each margin cell toward the terrace of
        the cell it was reached from.
      - Every walled site of the shipped seed stands on four levels: the
        quay at -1, the town's three terraces at 0, 1 and 2.
    - **Records.**
      - A town on one level is written in schema 1, as before, so every
        village's records are byte for byte what 4a wrote.
      - A terraced town is written in schema 2, with a level for each cell.
        A build that reads only schema 1 refuses it as damaged, rather than
        laying it flat.
    - **No step pieces yet.** The walker climbs 1.05 m, so a 1 m change of
      level is walked as the terrain's own step. The mockup's stepped
      street cells stand at their own height for now, and half-steps are
      pieces for later.
    - **Streets are dirt.** The terrain has no cobble or flagstone material.
      The walled town's streets take the village lanes' dirt until one is
      added.
    - **Linenleigh stands 520 m from Holbrook** on the shipped seed. So the
      home village now has a walled town within sight, laid on a world's
      first open with this build.
  - **4c in detail (written 2026-09-30).** The mockup's masonry, as it is
    built (`buildWalls`, `wallTower`, the keep in `makeTown`).
    - **The curtain wall.**
      - 116 cells: rows 6 and 31, columns 3 to 36, and columns 3 and 36
        between.
      - Each cell is a prism of masonry cut from its real corners, from its
        ground to `WALL_TOP` (9 m on the mockup's datum, 6 to 8 m over the
        streets). Its sides are rubble and its top is flagstone.
      - Its top is the wall walk, a surface the walker stands on.
      - Two merlons stand on each edge that faces out of the town and not
        onto more wall.
    - **The gates.** The four gate cells (columns 19 and 20 of rows 6 and
      31) hold their masonry only from 4 m over their ground, so a passage
      4 m high runs under it. The walker walks through.
    - **The towers and the keep are 4c's second half.**
      - The two stair towers are one cell each: a stone newel from the
        street to the wall walk, with a doorway onto the walk and a
        pyramid roof. The cutter's newel and its cone roof on one cell are
        those pieces already. What is new is a doorway at the walk's height.
      - The keep is a ring of six cells round a newel, three storeys, with
        0.6 m walls and a crenellated flat roof. It needs the flat roof's
        parapet (the desert's too).
    - **What is stored.**
      - The masonry is not a record. It is the template's, `v1/town.json`,
        cut on the town's stored chart every time it is built, like a
        building's pieces.
      - So `v1` templates are frozen from 4c. A later change to a v1
        template's masonry, or to any other part of it, is a `v2` template
        for towns laid after it.
      - A walled town that a 4b build stored has no masonry of its own, and
        it takes the wall at its first open with 4c. The v1 town template
        takes its masonry there, before any v1 walled town is shipped.
    - **The towers and the keep, how they are cut (written 2026-10-01).**
      - Both are buildings, exported from the mockup's own `wallTower` and
        keep code as building definitions. A definition gains `newel` (how
        high the stair climbs, how high its walls rise, and its exits: an
        edge and a height each) and `parapet` (a flat roof that can be
        walked on, with merlons on its outer edges).
      - **A tower** is one cell, a single storey as high as its walls
        (`wall_top`, 3 m over the wall walk).
        - Its newel climbs to the walk at the mockup's rate, a turn every
          3 m, from the entry the mockup chose so that the climb ends facing
          the wall.
        - Its exit is a doorway at the walk's height in the wall's edge.
        - Its cone roof sits on its walls, as a hut's does.
      - **The keep** is the ring of six cells and its newel cell: three
        storeys of 0.6 m rubble walls.
        - The newel climbs on to the roof, with a doorway onto each floor
          and the roof on its entry edge.
        - Its roof over the ring is flat and walkable, with merlons on the
          outer edges.
        - The newel's own walls rise 2.6 m over the roof to a cone of slate,
          the turret.
      - Both are stored as buildings are, so a later change to how they are
        cut reaches every keep, and a change to the definitions is a new
        template.
    - **Export.** The exporter calls the mockup's own `isWall`, `gate` and
      `inTown`, and writes each wall cell's cell, its bottom (its ground, or
      4 m over it at a gate), `WALL_TOP` and its merlon edges. The game
      retypes nothing.
    - **Shots.** The walled town from 60 m with its wall, the north gate
      from the road outside, and the wall walk from the top of a tower.
- **Towns are stored before any of this ships.** A slice before task 4.5
  builds the town from its template each time. That is safe only while no
  saved world has towns, so no build with towns merges to `main` before
  settlements are stored records (CLAUDE.md, "Saved games survive every
  change").
  - PR #19 merged slices 1 and 2a to `main` on 2026-09-29, ahead of this
    rule. Nothing of the town was saved, so nothing was lost. But the
    village template now stays as it is until slice 3a stores the town.
    3a is the next slice, ahead of 2b.
- **Each building is cut in its own tangent frame.** The frame sits at the
  building's centre, with up along the radius there. The real cell corners
  are projected into it. Over a house's 8 m the sphere falls away by under
  7 mm, so walls, floors and roof meet as they do on the mockup's flat grid.
  Each wall still stands on its own cells' corners.

## Risks / Trade-offs

- [This is the largest change in the plan] → It is built in the order of the
  tasks, and each group ends in a capture the owner can see. The walker and
  contact (`tenebris-towns` 1 and 2) are tested on a synthetic stair before
  any town exists.
- [Hundreds of emitters in a walled town] → `lamps-and-lanterns` measures a
  300-lantern bake first. If it is over its 12 ms budget, the tier bakes
  settlements separately, and that is argued in this design before group 5
  starts.
- [Frame cost of towns: draw calls, thin-solid queries, light] → It cannot be
  measured in a cloud session. The owner runs `tools/perf_suite.py`, and a
  `town` scenario is added to the suite in this change.
- [Records grow as towns do] → A building definition is about 500 bytes,
  and a town's footprint cells about 23 bytes each. The village is 24 KB,
  which puts 50 towns near 1.2 MB (measured in slice 3a). Growth adds
  buildings to plots, so a town's record is bounded by its plots.
- [The far form pops even though the pieces fade] → The fade covers the far
  form's own entrance too, and the night points fade in at dusk rather than
  switching on.

## Migration Plan

- An old save gets its settlement records the first time it is opened with
  towns, written through the durable path before anything is shown. Its sites
  are checked for earlier player edits at that moment. The home village is
  the exception: the build merged in PR #19 already stood it in every world
  it opened, so it is stored as that build showed it (slice 3a).
- Rollback is the previous build. The towns vanish and the terraces return to
  natural ground. Door edits are ignored. No player edit is lost.

## Open Questions

For the owner, before group 1:
- `tenebris-towns`' questions 1, 2 and 5 to 9: walking speed indoors, what
  the Tenebris people look like, the stair feel, walkable roofs, which kits,
  settlements per biome, and boats.
- Should an unsettled site be built later, once the player's edits there are
  gone?
