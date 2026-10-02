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
  - **Built towns on the map (2026-10-01).** The owner: "highlight built
    cities on the map please".
    - **Built** means a town the world holds and stands when you come near:
      laid, stored and read back.
    - The map's sites layer (`city-sites` task 4.1) draws a built town's
      marker in cream with a green ring round it.
    - A site of a kind with no template yet is drawn dimmed. An unsettled
      site (task 4.4) is dimmed with a red edge.
    - The legend's line counts them: "55 settlements on the map: 26 built,
      29 still to come."
    - Shots: the spawn's continent at 12 m a pixel, and the whole planet.
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
      - **As built (2026-10-01).**
        - The exporter wraps the mockup's own `newelStair` and `edgeWall`
          outside any `building()`. It writes the two towers (one cell, entry
          1 and 4, climbing 7 m to an exit onto the walk, walls to 10 m) and
          the keep (seven cells, three storeys, its door and 18 windows, the
          newel climbing to the roof at 9 m with exits at 3, 6 and 9 m,
          walls to 11.6 m).
        - Two kits are new: `tower` (stone, a slate cone) and `keep` (0.6 m
          rubble outside, stone in).
        - A definition's `newel` and `parapet` are written into its building
          record only where there is one, so every house's record is as it
          was.
        - `a_stair_tower_climbs_to_the_walk_and_the_keep_to_its_roof` holds
          each tower's way out to the walk's height across it, within 5 cm,
          and the keep's roof as walkable over its six ring cells.
        - A walled town stored by a 4b build keeps the 29 buildings it was
          stored with. It gets its wall, which is the template's, but not
          the towers or the keep, which would be new buildings.
    - **The seam on the walk (found 2026-10-01 by the walker test).**
      - **The finding.** Each masonry cell is a prism standing straight up
        from its own cell, in its own frame. Two neighbouring cells' "up"
        differ by the angle between their centres, so their prisms lean
        apart. Up on the walk the floors leave a wedge between them, as wide
        as the walk's height times the cell spacing over the radius.
        - `a_walker_goes_through_a_gate_and_up_a_tower_onto_the_walk`, on
          the 300 m gold-standard body, measured 61 mm at 7 m. The gap was
          the same between the tower and its wall cell and between two wall
          cells, and nothing answered a floor inside it.
        - On the shipped 4800 m planet it is 7 × 2.833 / 4800 = 4 mm. A
          walker at about 7 cm a tick lands in it on about one crossing in
          15 and falls for that tick.
        - A house has no seam: all its cells share one frame.
      - **The fix.** A wall cell's walk reaches across the seam on each
        edge that has no merlons, the edges that face more wall, a tower or
        the town.
        - The reach is a strip as wide as the wedge, the walk's height
          times the distance between the two cells' centres over the
          radius: 66 mm at 7 m on 300 m, 4 mm on 4800 m.
        - The strip is a floor only, 0.1 m deep under the walk's top. The
          walker stands on it; nothing is drawn and nothing is solid.
        - The drawn crack is left alone. At 4 mm it is under a pixel from
          the walk.
      - **Verify.** The same walker test walks out of each tower's doorway
        onto the walk without falling, sampled at 40 points.
    - **Export.** The exporter calls the mockup's own `isWall`, `gate` and
      `inTown`, and writes each wall cell's cell, its bottom (its ground, or
      4 m over it at a gate), `WALL_TOP` and its merlon edges. The game
      retypes nothing.
    - **Shots.** The walled town from 60 m with its wall, the north gate
      from the road outside, and the wall walk from the top of a tower.
  - **4d in detail, the harbour (written 2026-10-01).** The mockup's
    `makeCoast`, on the six harbour sites a world stores. Each choice below
    is a recommendation taken (ask only with screenshots). The owner sees
    them in 4d's shots beside the mockup's.
    - **What the mockup builds.** Rows 0 to 11 are sea over a sand bed
      shelving from 4 m to 0.35 m deep, 12 to 14 the beach (0.25 to 0.75 m),
      15 the stone quay (1 m), then the village on terraces at 1, 2 and 3 m
      and the hills behind. Its pieces:
      - 14 houses, 7 whitewash, 4 driftwood, 2 fieldstone and 1 timber,
        among them the Gull inn;
      - 2 boathouses on the beach, open on their two seaward edges;
      - 2 fish huts on stilts off the hut pier, each with a deck and a
        porch stair down to the pier;
      - 11 piers (a main pier, its head, the west, east and hut piers, six
        finger piers), planks on piles a metre over the water;
      - the mole (a strip of flags 1.2 m up across the harbour mouth) and
        its light, a round stone tower 8 m high with a beacon fire;
      - 21 street lamps, 5 lanterns along the quay and 5 at the piers'
        ends;
      - besides these: two dozen boats, the cog with a lantern on its
        stern, the gangplank, the fish
        market's stalls, nets, racks, pots, barrels and crates, the shipyard's
        hull in frame and its slip, and the people.
    - **It stands on the sea.** Every other town stands on its site's
      ground. A harbour stands on the water.
      - Its terrace is the sea level, 0 m. Each cell's level is its height
        in the template, rounded to a whole layer: the beach at 0 and 1, the
        quay at 1, the village at 1 to 3, the mole at 1. The planet's water
        fills every layer under 0 m above the ground (`column.rs`), so the
        sea is the planet's own.
      - **Its sea is the planet's.** No cell the template puts under the sea
        is laid, and no yard ring reaches into one. The seabed under the
        piers is the natural ground, and the quay meets whatever water the
        planet has there.
      - The land is cut or filled to the template's layers, as every
        terraced town's is, and eased back to the natural ground over the
        margin.
      - No fractions are needed after all. 4b expected the harbour to bring
        fractional heights. A building's base rounds as the ground does:
        the boathouses stand at 0, and the fish huts' floors at 2 m over
        the sea.
    - **It faces its sea.** A village's turn is the site's seeded one. A
      harbour's is chosen from the ground, with a shift as well.
      - Of the six turns, and every anchor shift up to 8 cells, take the
        placement where the most template cells agree with the planet about
        being sea (the natural ground under 0 m) or land. Ties go to the
        seeded turn, then to the smaller shift.
      - The scan reads the natural height of each patch cell once:
        6 × 217 placements of about 1,700 cells. Its time is measured on the
        first build and recorded here.
      - The site's harbour rule already keeps the anchor within 60 m of
        shelf water, with shallows in the footprint (`sites.rs`), so a
        match is near.
      - The record needs nothing new. It stores the layout cell on the
        site's anchor and each cell's patch cell and side, so the chosen
        placement is stored as any town's is.
    - **New pieces.** Each is cut from the template on the town's stored
      chart, as 4c's masonry is, and stored as a building where it is one.
      - **The kits** `whitewash` (whitewash outside, plaster in, 0.5 m, slate
        at pitch 0.9, a flag floor and a chimney) and `driftwood` (driftwood
        both faces, 0.25 m, thatch, a plank floor). Their textures are
        already exported.
      - **An open side.** A building's `open` edges get no wall: the
        boathouses' seaward edges, the mockup's `skipWall`.
      - **Piers.** A pier is a deck of planks between two points, its width,
        and its height, with a pile on each side about every 2.2 m (the
        mockup's `L / 2.2`) down to the natural ground under it. Its top is a floor and its edges are open.
        The template writes each pier as the mockup's `bridge` call gives it.
      - **Stilts, a deck and a porch stair.** These are the fish huts'. The
        swamp (4h) builds its village from the same three.
        - A raised building stands on piles to the natural ground.
        - A deck is a floor round it on the same piles, railed on its outer
          edges except at the stair.
        - A porch stair is a straight, open, railed flight outside the
          building, from the deck down to its foot.
      - **The light.** A round stone tower on the mole, solid to its top.
        It has an iron cage and a beacon that burns from dusk as a lamp does
        (task 5.4's kinds take a `beacon`).
      - **Lanterns off the grid.** A lantern has a position and a height in
        the template, not only a cell: the piers' lanterns stand over the
        water at the piers' height.
    - **Not in 4d.**
      - The boats are task 4.2b: the game's own craft, parked as vehicle
        records at their moorings. The cog and its gangplank are
        `sail-the-cog`.
      - The stalls, nets, racks, pots, barrels, crates, the shipyard and the
        people come with the other towns' dressing.
      - Street steps are 4b's open item.
      - The lanes and the quay are dirt until the terrain has cobble and
        flags, as the walled town's streets are.
    - **What is stored.** A harbour's town and buildings are records, as
      every town's are. Its piers, stilts, decks and light are the
      template's, like the masonry. No world has a harbour yet. An old save
      gains its harbours once on its next open, as it gained its villages
      in 4a, unless the player has worked their ground.
    - **Verify.**
      - Core: the two kits load, and a saved town can name them
        (`every_kit_a_saved_town_can_name_is_shipped`).
      - Core: `coast.json` lays on a test patch. No cell under the
        template's sea is in its footprint, and the natural ground there is
        untouched. The quay is 1 m over the water.
      - Core: on the shipped seed, every harbour site's chosen placement
        puts at least three quarters of the template's sea cells over the
        planet's sea.
      - Core walker tests, as 4c's: along the main pier from the quay to
        its head, with the feet on the planks all the way; into a boathouse
        from the sea side; and up a fish hut's porch stair onto its deck and
        in at its door.
      - App: every harbour lays and cuts on its own ground, and a world
        stores each once.
      - Shots beside the mockup's: the harbour from the pier head, from
        60 m by day and at night, a fish hut, and a boathouse.
  - **4d as built.**
    - **The export (2026-10-01).** `tools/export_town_templates.js coast`
      wrote `coast.json`.
      - It holds 18 buildings (7 whitewash, 8 driftwood, 2 fieldstone,
        1 timber), 21 street lamps, 11 piers, 10 lanterns and the light.
      - Every height is a whole layer: the ground runs from -4 to 5, the
        boathouses stand at 0 and the fish huts at 2.
      - Wrappers round the mockup's `bridge` (only one with piles is a
        pier), `stiltHouse` and `lantern` write what stands over the water.
        A building's `skipWall` edges are its `open` edges.
      - The lantern on the cog's stern is left to `sail-the-cog`.
      - Every new field is written for a sea scene only, so `village.json`
        and `town.json` re-export byte for byte.
      - `whitewash` and `driftwood` are in `kits.ron`, and
        `every_kit_a_saved_town_can_name_is_shipped` reads the harbour too.
      - A building record writes `open` and `stilts` only where there are
        some, so every record already saved is as it was.
    - **Lamps are not drawn in any town yet.** The game reads no template's
      `lamps`; a town's street lamps are task 5.2. The light of the
      Linenleigh night shots is its rooms' and the glowing flowers'. The
      harbour's lanterns are in its template for 5.2, not drawn in 4d, and
      the light's beacon waits with them.
    - **Laying it on the sea (2026-10-01).** `record::lay_at_sea`, and
      `settlement::sea`.
      - **The record does need something new**, unlike what "It faces its
        sea" said. A town's stored cells are its footprint. The fish huts,
        their decks and the cells under the piers are over the water, so they
        are not in the footprint, and a town built from its record could not
        chart them.
        - A harbour stores those cells too, as `over_sea`: charted, each by
          its exact key and side, and not laid.
        - It is written in a new settlement schema, 3, only when there are
          some. A build that does not know the sea calls a harbour damaged
          rather than cutting it without its fish huts.
        - Every village's and walled town's record is as it was.
      - **Tops.** A town's footprint took one top, dirt. A town on the sea
        keeps its beach as sand and its headland as bare rock (`Top::Sand`,
        `Top::Stone`). A land template's tops are as they were.
      - **The footprint** is the template's dry built cells and two yard
        rings over dry cells only. The margin eases the natural ground toward
        the beach a metre a ring, as at any town's edge, so the seabed
        shelves down from the shore. The open sea under the piers is the
        planet's.
      - **The placement scan** charts each of the six turns once, over the
        template grown by the 8-cell shift on every side. A shift is a
        translation on the hex grid, so each of the 217 shifts is scored by
        looking cells up in the grown chart (`chart::chart_reach` leaves out
        what cannot be laid rather than failing).
        - On the test coast it picks the same placement as charting every
          one of the 1,302 placements did. That took 3.9 s, and this takes
          0.36 s (dev profile, place and lay).
        - 87.6% of the harbour's cells agree with the coast about the sea.
      - `a_harbour_lies_with_its_sea_over_the_planets`,
        `a_harbour_stands_on_the_sea` and `a_harbour_is_stored_in_schema_3`
        hold it.
    - **Its pieces (2026-10-01).** `pieces::harbour`.
      - **Open edges.** The cutter gives an edge in a building's `open` no
        wall in any storey.
      - **Stilts.** A fish hut is cut as any building, on its floor 2 m over
        the sea, and its stilts are added to its own cut, in its frame.
        - Its floor gets an underside, and the walker a floor surface,
          because no ground is under it.
        - Its deck is a plank slab, railed on its outer edges but at the
          porch.
        - Every outside corner of hut and deck has a pile down to the
          ground under it.
        - The porch stair is a straight open flight, 0.3 m treads, down to
          the pier at 1 m.
      - **Piers.** A pier is cut in stretches of about 2.2 m, each flat in
        its own frame, so no stretch bows off the sphere. Each stretch laps
        the next by 5 cm, so the planks have no seam, the 4c lesson. Each
        joint has a pile either side, the mockup's spacing.
      - **The light.** A twelve-sided stone tower, solid to its flagged top,
        with an iron cage and a slate cap.
      - **Where the pieces stand.** A point in the mockup's metres is placed
        in its cell: as far toward the real centres across the cell's edges
        0 and 1 as the mockup puts it toward its own (`sea::point`).
      - **The ground under them.** A pile reaches the ground the column
        has: the town's where it laid or eased it, the planet's elsewhere.
      - **Walker tests**, on the test coast, asking the town what the
        walker asks:
        - `a_walker_goes_down_the_main_pier_to_its_head`: the planks at
          1 m all the way, 120 steps.
        - `a_walker_comes_into_a_boathouse_from_the_sea`: every open edge
          lets the walker by, and every walled one stops it.
        - `a_walker_climbs_a_fish_huts_porch_and_goes_in`: from the hut
          pier up the porch, across the deck and in at the door, both huts.
    - **In the game (2026-10-01).** Every harbour site is laid and stored
      as the villages and walled towns are (`TownAssets::harbour`, the
      `coast` template).
      - **The shift is 24 cells, not 8.** The six harbour sites of the
        shipped seed are mostly sea: 63 to 83% of the ground within 90 m
        is under water, against 26% in the template.
        - The site rule looks for shallows and a low shore, not for room on
          land, so with 8 cells of shift three sites agreed at only 54 to
          62%.
        - At 16 cells they agreed at 64 to 87%.
        - At 24 cells (about 68 m) they agree at 76 to 92%: Marenstrand 76,
          Wickingstrand 84, Holinghaven 92, Coringport 92, Selingquay 79,
          Corowstrand 86.
        - A placement counts only if the whole template lands on the patch,
          which for a harbour is 72 m wider (`patch_m`).
        - Placing takes 0.24 to 0.38 s a harbour, and laying and cutting
          0.6 to 0.9 s (dev profile).
      - **Where they still disagree, it degrades gently.** Template sea over
        the planet's land is left as the land: the piers stand over a beach.
        Template land over the planet's sea is laid as land, reclaimed. The
        owner judges both in the shots.
      - **A better rule for new worlds** would look for room on land too.
        Sites are stored, so it would change only new worlds. It is not done
        here.
      - **The log** names each town at its own footprint's middle, not its
        site's marker, so a harbour's `--at` stands over it.
      - `every_harbour_lays_on_its_sea_and_cuts` and
        `a_world_stores_every_harbour_once` (schema 3, a second open
        writes nothing) hold it. `print_where_the_harbours_stand` prints
        where to stand for shots.
    - **The owner (2026-10-01): "We're good on the mockup already.
      Approved. Implement in game".** The harbour mockup is approved. The
      rest of what it shows comes into the game in this order, each written
      up before it is built. Recommendation taken (ask only with
      screenshots).
      1. Street lamps and lanterns (task 5.2, below). Every town is dark at
         night without them, not only the harbour.
      2. The moored boats (task 4.2b).
      3. The dressing: the fish market's stalls, nets, racks, pots,
         barrels and crates, and the shipyard.
      4. The cog, `sail-the-cog`'s own change.
- **4e and 4g in detail, the desert and the tundra (written 2026-10-02).**
  The owner: "work on snow and desert cities next". These are the mockup's
  `makeDesert` and `makeTundra`, on the 4 desert and 4 tundra sites a world
  stores. The desert comes first, then the tundra, each ending in shots
  beside the mockup's. Each choice below is a recommendation taken (ask
  only with screenshots).
  - **What is shared.**
    - **Footprint by area.** The rule that every top other than grass or
      sand is built (4b) paves both scenes whole. The desert's dunes are
      `sand2`, and the tundra's open ground is snow and sage. So a template
      may name its wild areas (`wild`): "The dunes" and "The tundra". Their
      cells are not built. The footprint is the buildings, the masonry and
      every other painted cell, and two yard rings. A template without
      `wild` is as it was, so the village, the walled town and the harbour
      re-export byte for byte.
    - **Tops.** A template may map its tops to the terrain's (`tops`). The
      desert's are `sand2` and `sand` to sand, and the plaza's `flag` to
      stone. The tundra's are `snow` and `ice` to snow, and `sage` to the
      planet's own. `Top` gains `Snow`. Every other template keeps its rule,
      and a stored town keeps the tops it was laid with.
    - **Laid on levels.** Both are terraced (4b), at the site's seeded
      turn as a village is. The desert's plateau is level 2 and its oasis
      level 1. The tundra's camp and castle are level 1. Its lake is 0.6 m
      in the mockup, which is laid at level 0, with the ice at 0.6 m.
    - **Lights.** The mockup's lanterns, braziers, torches and the camp's
      fire go through task 5.2's lamps, in the first air layer over their
      cells: a lantern is a `LanternPost`, a brazier and the fire pit a
      `Brazier`, and a torch a `Torch`. The exporter writes `lanterns`
      for every scene. Until now it wrote them only for the sea.
    - **Saves.** A town is stored the first time it is met, as every town
      is. Desert and tundra sites have no town yet, so an old save gains
      them on its next open, unless the player has worked their ground
      (4a). Both are terraced, so they are written in schema 2. A build that
      does not know `Top::Snow` refuses a tundra's record as damaged rather
      than laying it wrong.
  - **4e, the desert.** A sandstone town on a plateau, round a sunken oasis.
    - **The kits**, field for field from the mockup's `KITS`:
      - `sandstone`: sandstone outside, salt plaster in, 0.5 m. A flat roof
        topped with clay tile, a clay-tile floor, and 0.6 × 0.8 m windows.
      - `adobe`: clay outside, salt plaster in, 0.5 m. A dome of salt
        plaster, a sand floor, and 0.5 × 0.6 m windows.
    - **A walked flat roof with a gap.** The five sandstone houses have the
      keep's walked roof (4c): its top in the kit's roof material, with
      merlons on its outer edges. A building's `parapet_gaps` leave one edge
      without merlons, where its outside stair arrives (the mockup's
      `parapetGaps`).
    - **An outside stair.** The mockup's `stairRun`: a straight flight of
      solid steps, 1.1 m wide, from the ground at the house's east end up to
      the roof's gap, about 17 risers of 0.19 m. The exporter writes each as
      `stairs: [{from, to, width_m, material}]`. The game cuts it as the
      harbour's porch stair, with solid steps rather than open treads. The
      walker climbs it as any stair.
    - **The dome.** Roof `dome`: a 0.25 m cap over every cell, and a
      half-ellipsoid on it, the mockup's `domeCap`. One cell has a dome
      0.46 of a cell across. Seven cells (the caravan hall) have a dome
      1.35 cells across over the middle, as high as it is across. Inside, the
      cap's underside is the ceiling. The hall is one tall storey (`tall:
      2`), with an open doorway, no leaf.
    - **The plaza and the oasis.** The plaza's flags are stone, and the
      oasis is a sand hollow a metre down. The mockup draws no water there,
      so the game draws none. A pool in a town is the swamp's work (4h).
    - **Dressing.** The four market stalls, the hall's crates and the
      cacti round the oasis are 4.2c's pieces, written for every scene. The
      cactus is new: a ribbed column with two arms, the mockup's `cactus`.
      The dunes' cacti are the planet's own flora. The pots, cloth and tables
      inside the houses are furniture, which no town has yet.
    - **Verify.**
      - Core: `desert.json` lays on a test patch with the dunes outside the
        footprint, and the plaza at one level and the oasis a level under
        it.
      - Core: a walker climbs a sandstone house's outside stair onto its
        roof, through the parapet's gap, and is held by the merlons.
      - Core: a walker goes into a domed house and is held by its walls,
        and the dome is over the walker's head.
      - App: every desert site lays and cuts, and a world stores each once.
      - Shots beside the mockup's, by day and at 22:30: the plaza, a roof
        from its stair, a domed house, and the caravan hall.
  - **4g, the tundra.** Igloos round a fire, a granite longhouse, and an
    ice castle on a frozen lake's shore.
    - **The kits:**
      - `granite`: granite outside, plank in, 0.6 m. A gable of sage turf
        at pitch 0.5, a plank floor, and a hearth (decision 7a's).
      - `ice`: ice outside and in, 0.6 m, and a snow floor. Its walked roof
        is topped with snow, and its turret is a spire of ice.
      - `igloo`: snow blocks, cut by its own shape (below).
    - **The ice keep, its wall and its towers.** These are 4c's masonry,
      in ice.
      - The keep is the mockup's `ringKeep`: two storeys round a newel,
        and a walked roof. Its turret is a 5 m spire of ice where the
        town's is 2.6 m of slate. The exporter writes it from the
        `ringKeep` call rather than from the town's area names.
        `NewelDef` gains `spire_m`.
      - The curtain wall is the hex ring of four round the keep, 5 m over
        the camp, with its gate. Merlons stand on its outward edges, and the
        gate's arch is 4 m, as the town's.
        - The exporter writes it from the `curtainWall` call. The template
          names the masonry's material (`masonry_material: ice, snow`),
          where the town's is rubble and stone.
      - The two ice towers are 4c's stair towers, from the `wallTower`
        call, in the `ice` kit with a 5 m spire.
    - **The igloo.** It is the first building not cut to its cell.
      - A dome of snow blocks 4.6 m across and 2.5 m high stands on one
        cell and spills over its ring.
      - A vaulted tunnel runs out along its door's direction, 2 m wide and
        2.25 m high outside.
      - Both are cut as the mockup cuts them: the dome is cut to the
        tunnel's outer profile, and the tunnel's shell reaches 2 cm into
        the dome, so there is no seam (12b).
      - The walker is held as the mockup holds it. Sixteen wall segments
        inside the dome's low edge stop at the tunnel. The tunnel's walls
        stand either side, with its roof over them. The dome's underside is
        the ceiling.
      - An igloo is a building record (kit `igloo`, its cell and its door),
        so a town can say who lives in one later.
    - **The longhouse** is a granite building of ten cells, with its
      hearth. Its benches and beds are furniture, as the desert's are.
    - **The frozen lake.** It is laid a layer down and floored with a
      sheet of ice at 0.6 m. The sheet is a walked floor in the `ice`
      texture over the snow, and it is the template's, like the piers.
    - **Dressing.** The fire pit is a stone ring with a brazier's light, and
      the two drying racks are posts with a hide. The shrubs and boulders
      round the camp are the planet's own flora.
    - **Verify.**
      - Core: `tundra.json` lays with the open tundra outside the
        footprint, and the lake a level under the camp.
      - Core: a walker goes in through an igloo's tunnel, stands up inside,
        and is held by its wall. It cannot walk out through the dome.
      - Core: a walker goes through the ice gate, up a tower and onto the
        wall walk, as 4c's test does in the town.
      - App: every tundra site lays and cuts, and a world stores each once.
      - Shots beside the mockup's, by day and at 22:30: the camp, an
        igloo, the longhouse, the gate and the keep's roof.
  - **4e and 4g as built (2026-10-02).**
    - **The export.** `tools/export_town_templates.js desert tundra` writes
      `desert.json` and `tundra.json`. The village, the walled town and the
      harbour re-export byte for byte: each new field is written only for
      the scenes that have it.
      - The desert has 11 buildings (5 sandstone, 5 domed, the hall), 5
        roof stairs, 5 braziers, 2 lanterns, and 4 stalls, 3 crates and 6
        cacti.
      - The tundra has 9 buildings: the longhouse, the ice keep, 2 ice
        towers and 5 igloos. It also has 24 cells of ice wall (one of
        them a gate), a frozen lake of 55 cells with its ice at 0.6 m,
        and 4 torches and the camp fire.
      - The tundra's keep, towers and wall are written from their own
        `ringKeep`, `wallTower` and `curtainWall` calls. The town's are
        still found by its area names.
    - **The footprint** is every cell outside the wild areas. It also
      takes the cells a fire, a lantern or a dressing thing stands on:
      the tundra's camp fire stands on the open tundra, where it would
      otherwise be off the town's chart, and so out of its lamps.
    - **The parapet is a wall, not merlons.** The mockup's `flatRoof`
      raises a parapet 0.65 m high and 0.4 m thick on every outer edge but
      the gap. It has a post at each corner, so no top is shared, and the
      beam ends show under it.
    - **The roof stair meets the roof.** Placed by the mockup's metres
      alone, its last tread stopped 16 cm short of the roof: the house is
      cut on its cells' real corners.
      - Its head now ends on the real corners of the gap edge it climbs to.
      - A landing 0.4 m long runs on over the roof. The stair's frame and
        the house's are flat apart and part by up to 10 cm.
      - A step taller than a walker steps (1.05 m) is solid under its tread,
        so the flight is not walked into from beside it.
    - **The tundra's masonry** is the walled town's, in ice. The template
      names its material (ice under snow, ice under the gate's vault too).
      - The keep's walked roof is topped with its kit's floor: flagstones
        in the town, snow here.
      - The keep's spire and the towers' cones are as high as the template
        says, 5 m.
      - Each ice tower's doorway is about 5 degrees short of its stair's
        landing, where the town's are on it, so the walker steps straight
        out. The walls test, now shared by both towns, allows 10 degrees.
    - **The igloo** is cut as the mockup cuts it: the dome is cut to its
      tunnel's profile, and it has its snow bench, furs and lamp. A candle
      lights it.
      - One departure: a second ring of wall round the dome's foot. In the
        mockup, a walker outside walks 0.6 m into the dome's shell before
        the inner ring stops it.
    - **The keep's roof shot.** A walker is placed on the terrain under its
      spot, which under the keep is its ground floor, so the roof is shot
      from the column view at the roof's height. The column view faces
      east; it now takes `--yaw` as the walk does (east turned about the
      up by minus the yaw, 0 leaving it east), so it can look where the
      mockup's `keepRoof` looks. A capture camera only: the game's own
      cameras do not change.
    - **Seen in the tundra's shots (2026-10-02).**
      - The camp's fire pit is a brazier block where the mockup has a ring
        of stones round embers. The gate's torches are the player's torch,
        a stick of 0.55 m, where the mockup's stand 2 m on poles.
      - By day the camp fire and the gate's torches turn the snow round
        them tan, like sand. They are always lit, and `lamps-and-lanterns`
        adds a lamp's light without scaling it by daylight ("a torch is as
        bright at noon as at midnight"). On white snow that warm term is
        stronger than on any ground the lights were judged on. Changing it
        changes an approved decision, so it is asked as survey L4 with the
        camp's and the gate's shots. **Answered (2026-10-02):
        "recommended"**: a lamp's light fades where the sun reaches it,
        `lamps-and-lanterns` decision 15.
    - **Finding: a lamp lights the ground by a wall but not the wall
      (2026-10-02).** At 22:30 the ice gate's torches light the snow
      under them, and the ice of the wall a metre away stays as dark as
      the rest; the keep's merlons are the same. The mockup's torches warm
      the ice beside them.
      - A town's pieces take the light field as a craft does
        (`lamps-and-lanterns` decision 2): eight samples at the corners of
        each mesh's bounds, blended across them. A town's meshes are one a
        texture, so the camp's ice is one mesh some 40 m across, and its
        corners are far from any torch, whose light reaches about 14 m.
      - **Proposed, not built:** cut a town's meshes into pieces no wider
        than a lamp's reach (a building, or a run of wall a few cells
        long), so the corners that light a piece are near what lights it.
        It costs draws, which the cloud session cannot measure. It belongs
        to the lights as much as to the towns, and it waits for its own
        write-up.
    - **Finding: the igloo's candle lit nothing (2026-10-02).** At 22:30
      the mockup's igloo glows warm through its tunnel and the game's is
      grey inside.
      - A room's light reaches only the faces cut as its room's, and a face
        is the room's when its air is inside the building's `Indoors`.
        `igloo` set its `Indoors` after cutting every face, where a building
        sets it before, so no face of an igloo was its room's.
      - Its `Indoors` was its cell's hex besides, 1.42 m from centre to
        edge, under a dome of 2.3 m: most of the dome's inside would have
        been outside it.
      - And an igloo had no floor of its own. The walker stands on the
        planet's snow, which a room's light never reaches.
      - **The fix.** An igloo's inside is its dome's own air, `(r / R)^2 +
        (y / H)^2 < 1` round its centre (the dome is that ellipsoid), set
        before it cuts. A floor of its kit's snow is cut under the dome, a
        centimetre over the ground as a building's is. So the dome's inside,
        the bench, the furs and the floor are its room's and the candle
        lights them; the dome's outside and the tunnel past it are not.
      - **Verify.** A core test that every face of an igloo whose air is
        under its dome is cut as its room's and none outside it is, and
        that it has a floor; the igloo's shot at 22:30 retaken.
      - **Fixed (2026-10-02).** `Indoors::dome` holds the dome's air, and
        `an_igloos_room_is_the_air_under_its_dome` finds every room face's
        air under the dome, over 500 faces of the dome's inside and the
        floor among them. On the old cut it failed: an igloo had no room
        faces at all.
    - **Not built.**
      - The plaza's flags become the terrain's stone, which reads red in a
        desert, where the mockup's are pale. The shots show it.
      - The longhouse's open fire, the drying racks and every room's
        furniture wait for furniture.
    - **Cost.** Each desert or tundra lays and cuts in 0.04 to 0.08 s
      (dev profile). The frame cost was not measured in this cloud session.
    - `the_desert_lays_with_its_dunes_wild_and_its_oasis_a_level_down`,
      `a_walker_climbs_a_sandstone_houses_stair_onto_its_roof`,
      `a_walker_goes_into_a_domed_house_under_its_dome`,
      `the_tundra_lays_with_its_open_ground_wild_and_its_lake_a_level_down`,
      `a_walker_goes_into_an_igloo_through_its_tunnel`,
      `a_walker_goes_through_the_ice_gate_and_up_a_tower_onto_the_walk`,
      and the app's `every_desert_lays_and_cuts`,
      `every_tundra_camp_lays_and_cuts` and the two store-once tests hold
      it.
- **4i in detail, the jungle (written 2026-10-02).** The owner: "alright
  now jungle city". This is the mockup's `makeJungle`, on the jungle sites
  a world stores. Three kapok trees stand in the jungle with a plank
  platform round each, 9 m up. Rope bridges join the platforms. A pole
  tower climbs to the first platform. There are huts on the platforms, two
  huts on stilts on the ground, and a clearing with a fire pit. Each choice
  below is a recommendation taken (ask only with screenshots).
  - **The footprint is what the village built.**
    - The whole floor is one area, "The jungle floor", and the stream is
      "The stream". Both are wild (4e's `wild`), so the planet's own jungle,
      trees and all, stands between the village's pieces, as the mockup's
      scatter of trees does.
    - The footprint is the cells the village builds on or over:
      - every building's cells;
      - each platform's seven cells;
      - every cell a bridge's line crosses;
      - the tower's door cell, and the stilt huts' decks and stair feet;
      - the clearing: its fire pit's cell and the three the mockup keeps
        its trees off;
      - the cells its fires stand on (4e's rule).
    - A planet tree in a footprint cell would grow through a platform or a
      bridge, so none grows there (4a's clearing). And the ground under a
      bridge is the terrace, so a bridge 8 m over it stays 8 m over it on
      a hilly site.
    - The exporter writes the clearing as `cleared`: the cells the mockup's
      `NO_TREE` holds that no platform, building or deck already names.
  - **Tops.** `redloam` is dirt and `litter` keeps the planet's top (4e's
    `tops`). So the clearing and the paths read as the mockup's red earth.
  - **Laid flat.** Every cell the village builds is on the floor at 1 m, the
    stream is wild, and the scene's raised edges are wild too. So it is laid
    as a village is, on one terrace at the template's 1 m, not terraced.
    Its pieces' heights are over that datum: the platforms are 8 m over the
    terrace.
  - **The kapok.** The mockup's `kapok`, one at each platform's centre:
    - A trunk 2.3 m across and 15 m tall, a 12-sided prism of `kapok`
      bark. Six buttress fins round its foot. Five branches at its top,
      fourteen blocks of `leaves` for its crown, and eight hanging vines.
    - The mockup seeds each tree's fins, branches and leaves from where it
      stands. The exporter writes each part as the mockup made it: its
      material, centre, size and turn, in the mockup's metres. So the game
      draws the mockup's trees, not a second copy of its random numbers.
    - The trunk and the fins are solids (the mockup's `addSolid`), from
      the ground to 2.2 m for a fin. The crown is not: nothing walks there.
    - Each is cut in its own frame at its trunk, as a building is.
    - The kapok is the template's, not the planet's flora. A world's jungle
      trees are unchanged.
  - **The platforms.** The mockup's `deck` at 9 m, a plank slab on each of
    a platform's cells but the huts'.
    - Each cell is the slab the stilts' deck is (4d), with its floor for
      the walker. The centre cell's slab runs into the trunk, which holds
      the walker off it.
    - Rails stand on the outer edges, except where a bridge or the tower
      comes in, and except an edge onto a hut. The exporter writes the
      railed edges, so the game does not work out the gaps again.
    - Six beams of kapok run from the trunk out under each ring cell,
      2.4 m under the deck, as the mockup's.
  - **The rope bridges.** The mockup's `bridge` with sag, three of them,
    1.2 m wide, each sagging 0.8 m at its middle:
    `h(t) = A + (B - A) t - 4 sag t (1 - t)`.
    - Each is cut as a pier is (4d): in stretches, each flat in its own
      frame, but sloped. A stretch is the mockup's rope span, about 1.4 m,
      and its walk is a `Ramp` from `h` at its start to `h` at its end. The
      steepest stretch is at the ends, at about 10 degrees on these bridges.
    - The planks are the mockup's slats, one every 0.34 m, each a box in
      its stretch's frame at `h` under it.
    - Two ropes run each side, 0.5 m and 0.95 m up, with a post at each
      span's start. At each end the ropes end on the platform edge's
      corners, where its rails end (the mockup's `ends`, section 12b).
    - Each span's rope is a wall solid to 1 m over the walk, so the walker
      cannot step off the side. The ends are open.
    - Its lanterns hang off the ropes every 5.5 m, on alternate sides: the
      mockup's `bridgeLamps`. Each is a `LanternHanging` lamp in the
      column under where it hangs.
  - **The pole tower.** The mockup's `wallTower` in the `poles` kit: 4c's
    stair tower, a newel from the ground up to 9 m, its exit onto the first
    platform, its walls to 12 m, and a cone of `leafthatch` 2.6 m high. The
    exporter writes it as the tundra's towers, from the call. The `poles`
    kit is poles out, in and at the edges, 0.5 m thick, roofed in
    `leafthatch`.
  - **The tree huts.** The mockup's `building` with `raised: true`, on
    ring cells at 9 m: two on the first platform, and one on each of the
    others (the third the "Lookout hut").
    - The `junglehut` kit, field for field: poles outside and in, timber
      at the edges, 0.25 m thick. A `leafthatch` gable at pitch 1.2, a
      plank floor, a door 0.9 × 1.9 m and windows 0.7 × 0.6 m. One storey,
      a hut.
    - A raised building has no ground under its floor. It gets the stilts'
      floor slab under each of its cells, and a floor for the walker, but no
      piles. The exporter writes `raised`, which every other template lacks,
      so they re-export byte for byte.
  - **The stilt huts.** The mockup's `stiltHouse`, as 4d's: two huts of
    the `junglehut` kit, 1.2 m over the floor, each with a deck and a porch
    stair. The exporter writes `stilts` for the jungle as for the sea.
    - A stair's foot is stored over the town's terrace: the lay takes the
      datum off `foot_m` (4d), so the jungle's lands on its floor as the
      harbour's lands on its beach.
  - **Lights at 9 m.**
    - Two torches stand at each platform's rails, and a fire burns on the
      second platform. Each is a lamp (task 5.2): a torch a `Torch`, the
      fire a `Brazier`. A torch stands at each stilt hut's stair foot, and
      the clearing has its fire pit.
    - The exporter writes the platform's fire (the mockup's `addFire` on
      the deck) as a fire. A fire inside a hearth or a pit is not written
      twice.
    - **A lamp's height (a fix).** `lamps_of` puts a lamp on a cell of the
      town at the cell's terrace, whatever height the template gives it.
      Only a lamp off the town's cells gets the template's height. A
      platform's torch would stand on the ground 8 m under it.
    - The rule becomes: a lamp stands at the higher of its cell's terrace
      and the town's terrace plus its height over the datum.
    - A lamp already standing on its ground keeps its layer. A test pins
      every lamp of the village, the walled town, the harbour, the desert
      and the tundra to the layer it has today, so only a raised lamp
      moves.
    - A lamp stands at its column's centre, as every town lamp does. So a
      platform's torch stands at its ring cell's centre, 0.9 m in from where
      the mockup's stands, and a bridge's lantern up to 1.4 m off the
      bridge's line.
  - **Not built.**
    - The huts' beds, the platform's sleeping mat and the people wait for
      furniture and folk.
    - The scatter's basalt boulders are the planet's own ground.
  - **Night.** At 22:30 the torches will light the ground under them and
    the planks very little. A town's planks are one mesh, lit at the
    corners of its bounds: the finding above ("a lamp lights the ground by
    a wall but not the wall"). The jungle's planks are a platform 8 m up
    and three bridges, so they show that finding more than any town yet.
    Its fix, splitting a town's meshes, stays proposed, and the night shots
    will show what it is for.
  - **Saves.** A town is stored the first time it is met, as every town
    is. Jungle sites have no town yet, so an old save gains them on its
    next open, unless the player has worked their ground (4a). The kapoks,
    platforms, bridges and lights are the template's and derived, never
    saved. The lamp height fix moves no lamp of a town that already
    stands.
  - **Verify.**
    - Core: `jungle.json` lays with the floor and the stream outside the
      footprint, and the platforms, the bridge cells, the tower and the
      clearing in it.
    - Core: a walker goes in at the tower's door, climbs its newel and
      steps out onto the first platform 8 m up. It walks to a bridge's end
      and across it, down the sag and up again, held on the walk by its
      ropes, to the second platform.
    - Core: a walker on a bridge cannot step off its side.
    - Core: a walker goes into a tree hut through its door, and onto a stilt
      hut's deck by its porch stair.
    - Core: every lamp of the other five templates stands where it stood,
      and a platform's torches stand 8 m up.
    - App: every jungle site lays and cuts, and a world stores each once.
    - Shots beside the mockup's, by day and at 22:30: the clearing, the
      tower's door, the platform, the bridge, a tree hut's door and the
      lookout, and the village from above.
  - **4i as built (2026-10-02).**
    - **The export.** `tools/export_town_templates.js jungle` writes
      `jungle.json`. The village, the walled town, the harbour, the desert
      and the tundra re-export byte for byte.
      - Six buildings: three tree huts at 9 m (the third the lookout), two
        stilt huts, and the pole tower. The mockup asks for two huts on the
        first platform but places one there, as no two of its free ring
        cells are neighbours; the game has what it placed.
      - Three kapoks of 33 parts each (six of them solid fins), three
        platforms of six cells (the seventh is a hut's), 18 beams, and three
        bridges of 22.5, 18.6 and 27.9 m. Their steepest spans are 6.5 to
        9.8 degrees.
      - Ten fires: six platform torches, the platform's fire, the clearing's
        pit and a torch at each stilt hut. Nine bridge lanterns.
    - **The footprint** on the test patch is 74 cells of the 1,700; the
      rest keep the planet's jungle.
    - **One lamp a column.** The third bridge passes over the clearing's
      fire pit, and the lantern over it shares the pit's column. So 8 of
      the 9 lanterns stand, and the pit keeps its column.
    - **The platform's floor laps its cells by 6 cm.** The pole tower's newel
      is cut in a frame on the ground and the platform in one 8 m up. Each
      finds the shared edge from its own frame, so on the 300 m test body a
      walker stepping out of the tower fell through a 4 cm gap (2 mm on the
      game's planet). The lap closes it; the platform's outer edges are
      railed, so it is never walked off.
    - **A flat platform is 2.5 cm higher at its edge** than at its middle on
      the test body (1.7 mm on the game's), which the walk tests allow.
    - **The shots' height.** The column view stands its eye over the ground
      it finds when it starts, the natural ground, before the town's ground
      is installed. So a spot on the platforms gives its height over the
      natural ground under it, as `every_jungle_village_lays_and_cuts`
      prints it. A capture camera only.
    - **The shots' doors.** `--open-doors` opened a town's doors only when a
      walker is spawned, so in the column view the tree hut's door stood
      shut where the mockup's stands open. It now opens them in the column
      view too. A capture flag only: no door in the game changes.
    - **Cost.** Each jungle village lays and cuts in 0.04 s (dev profile).
      The frame cost was not measured in this cloud session.
    - `the_jungle_lays_with_its_floor_wild_and_its_platforms_built`,
      `a_walker_climbs_the_pole_tower_and_crosses_a_rope_bridge`,
      `a_walker_goes_into_the_tree_huts_and_up_onto_the_stilt_huts`,
      `no_lamp_of_an_older_town_moves`, and the app's
      `every_jungle_village_lays_and_cuts` and
      `a_world_stores_every_jungle_village_once` hold it.
  - **Finding: a column can take its neighbour's ground (2026-10-02).**
    Found when the crust test (T14) took in the jungle's six villages: in
    Ixapaya, the footprint cell (30, 23) has no crust. Its own column finds
    a margin cell 2.6 m away as its cell.
    - A town's ground finds a direction's cell by the largest dot product
      of the direction with each cell's centre (`TownGround::at`). At a
      radius of 4800 m, neighbouring centres differ in that dot by about
      1.5e-7. A product of two unit `f32` vectors is good to about 1e-7.
      So both read 1.0 here, and the tie goes to the lower index. It is
      the same flaw as the lamps' (finding above), in the lookup every
      column's ground goes through.
    - In the older towns a footprint cell's neighbours are mostly
      footprint cells on the same terrace, so a wrong pick changes
      nothing. The jungle's footprint is thin: a lone torch's cell, a
      bridge's strip. There a wrong pick makes a footprint column a margin
      column: no crust, eased rather than laid, and a tree may grow in it.
    - **Measuring before fixing.** The fix is to compare chords, as the
      lamps now do. But this lookup gives every column of every town its
      height, top and trees, so a fix could move ground in worlds already
      saved. An instrument (an ignored test) counts, over every footprint
      and margin column of the shipped seed's older towns, how many find
      another cell by chord than by dot, and how many of those would take
      a different height, top, crust or clearing.
    - **Measured (2026-10-02),** `print_the_ground_lookup_against_the_chord`
      on the shipped seed:

      | Towns | Columns | Another cell by dot | Another ground | Towns touched |
      | --- | ---: | ---: | ---: | ---: |
      | Villages | 49,092 | 4,638 | 1,056 | 20 |
      | Walled towns | 13,071 | 1,173 | 394 | 6 |
      | Harbours | 16,198 | 894 | 267 | 6 |
      | Desert towns | 6,312 | 222 | 25 | 4 |
      | Tundra camps | 6,088 | 517 | 148 | 4 |
      | Jungle villages (new) | 2,651 | 216 | 64 | 6 |

      So 1,860 columns of the 40 older towns would take another height,
      top, crust or clearing. That is ground already in saved worlds.
    - **Decision (recommendation taken, ask only with screenshots).** The
      lookup is part of a town's laying-out rules, which each stored town
      already names (`layout`, 1 so far, "a town is never rebuilt by it").
      `LAYOUT_VERSION` becomes 2: a town laid from now on finds a
      direction's cell by the nearest chord, and a stored town of layout 1
      keeps the dot lookup, column for column.
      - A town is stored the first time it is met, so every layout-2 town
        is ground no one has seen. That includes every jungle village, and
        an old save's sites not yet visited.
      - Rejected: the chord everywhere (moves 1,860 columns under saved
        towns), and the chord for the jungle only (the next template would
        need the same exception).
    - **Verify.** Core: a layout-2 town's every footprint and margin column
      finds its own cell; a layout-1 town's lookup is unchanged (its ground's
      digest and the measured counts above). App: the crust test holds on
      every town, the jungle's included.
- **Task 5.2 in detail, street lamps (written 2026-10-01).** Decision 7
  already puts a town's street lanterns in the voxel field as
  `lamps-and-lanterns`' dusk-lit materials. A street lamp is a
  `LanternPost` block, and the tier's bake lights it as it lights one the
  player places. A lamp block is neither solid nor opaque, so one in a lane
  is walked through.
  - **Which lamps.**
    - A template's `lamps` cells: the mockup's street lamps (3 in the
      village, 38 in the walled town, 21 in the harbour).
    - A sea template's `lanterns`: the harbour's 10 along its quay and at
      its piers' ends, each in the cell under it.
  - **Where.** In the first layer over the cell's ground: its terrace where
    the town laid it. Over the water, a pier's lantern stands in the layer
    at the pier's height over the sea.
  - **Derived, not stored.** A town's lamps come from its frozen template
    on its stored chart, as its masonry does, so every town already in a
    save gains its lamps without a record changing. A player's edit to a
    lamp's cell wins over the lamp, as over any generated block.
  - **How the column gets one.** Each town's ground carries its lamps:
    each cell's centre, the layer's altitude and the material.
    `ground::lamp` answers the column generator for a column at a lamp's
    cell. `column::generate_solid` puts the lamp in that layer if the
    layer is air.
  - **Cost.** The village has 3 lamps and the walled town 38, against
    `lamps-and-lanterns`' 300-lantern figure (task 4.0). The bake time is
    not measured in the cloud session.
  - **Verify.**
    - Core: a town's lamps land in the first air layer over their cells,
      and a column away from them gets none.
    - App: the walled town's 38 lamps and the harbour's 31 are installed.
    - Shots at 22:30, from the street and from above.
- **Finding: caves open into towns' ground (2026-10-01).** A harbour shot
  showed a dark hexagonal hole in the beach at Coringport.
  `print_where_caves_open_into_towns` asks the planet's worms which layers
  they open under each town's footprint.
  - On the shipped seed they open the top two layers of a town's ground in
    26 of its 32 towns: 1 to 50 cells a town, in lanes, yards and under
    floors.
  - Holbrook's lane has one cave mouth of 4 cells, (25, 23) to (26, 24).
    Holbrook is on `main`, so saves made since #19 already have it.
  - Coringport's hole is not in its footprint. It is most likely one of the
    natural cave mouths the worms make on purpose (`surface_share`), on the
    natural beach past the town.
  - **The choice is the owner's**, because closing them changes the
    generated ground of saves that exist (CLAUDE.md, "Saved games survive
    every change"). It goes in the survey with a shot of Holbrook's lane.
    - **Recommended: a crust.** Worms carve nothing in the top three layers
      of a town's footprint, as `cleared` keeps its trees off, so a lane or a
      floor is never a pit. A player's edit still wins. It changes 4 cells of
      Holbrook in existing saves, and only where no edit is. Nothing a
      player made is lost.
    - **Or leave them.** A cave mouth in a lane is a way down, as anywhere
      on the planet.
    - **Asked as survey T14 (2026-10-01)**, with the shot of Holbrook's
      lane, on its own page (https://claude.ai/artifact/RriyuLo9EEZ1gTzGq58A2r).
      This session's Docs connector could not edit the survey doc. The page
      keeps the answer, and it moves into the survey doc when the connector
      is back. Nothing is built until the owner answers, since it changes
      existing saves.
  - **Answered (2026-10-01, read 2026-10-02): "crust"**, the
    recommendation, on its page. It moved into the survey doc's "Already
    decided".
  - **How it is built.**
    - A town's ground answers, for a direction, the altitude of its
      ground there when the direction is in its footprint (`crust`), and
      nothing elsewhere: the margin's rings are eased natural ground and
      keep their caves.
    - The column's carve takes that altitude and opens no layer whose
      centre is within `CRUST_LAYERS` (3) under it. Deeper layers carve as
      before, so a cave under a town is still there, roofed.
    - It is the one carve every column takes (`generate_edited`), so the
      collision, the light and the drawing all see the same crust.
    - A player's edits are applied after the carve, as before, so a hole
      someone dug stays dug.
  - **Saves.** Generated ground, never stored. In a save made since the
    towns shipped, a cave mouth in a town's lanes, yards or floors fills
    in where nobody has edited it: Holbrook's 4 cells and the rest of the
    26 towns'. The owner chose this knowing it.
  - **Verify.** A core test carves a surface-starting worm's column with
    and without a crust: without, its top three layers open; with, they
    hold and the layers under them still open. An app test asks every
    town on the shipped seed: no footprint cell's top three layers open,
    where 26 of 32 towns had some. Holbrook's lane before and after.
  - **Built (2026-10-02).** `GroundAt::crust` and `ground::crust` answer
    the footprint's terrace; `column::carve_worms` is the carve, with
    `CRUST_LAYERS` 3. `a_crust_keeps_a_cave_mouth_out_of_the_top_three_layers`
    carves 400 columns round a cave mouth with and without it: the crust
    holds in every one, and everything under it opens as before.
    `no_cave_opens_into_a_towns_ground` asks all 46 towns of the shipped
    seed (the deserts, tundra camps and jungle villages among them): none of
    their 36,074 footprint cells opens in its top three layers, where 32 of
    the 46 had a cave mouth without the crust. The jungle's needed the
    layout-2 ground lookup (finding below): a thin footprint read some of
    its own columns as margin.
- **Finding: half of every town's lamps stand nowhere (2026-10-02).** The
  tundra camp's shot at Torifjell has no camp fire where the mockup has one
  3 m ahead, and the desert's plaza shows none of its five braziers.
  - An instrument (now `print_the_lamps_in_their_columns`) reads the
    column the planet generates at each lamp, with the town's ground installed. Each
    lamp's altitude is its cell's terrace, and the ground there is at it.
    Only 24 of the 48 lamps of the 8 deserts and tundra camps are in their
    columns. The layer of each of the rest is plain air, lantern posts as
    much as braziers and torches: none at all at Vasefjell, the fire
    missing at Torifjell.
  - **Why.** `TownGround::lamp` finds the lamp in a column by
    `lamp · column > cos(0.5 m / R)`. On the 300 m test body that cosine
    is 0.9999986. On the game's planet, R = 4800 m, it is 1 - 5.4e-9, which
    is exactly 1.0 in `f32`. So a lamp is found only where the dot product
    of two equal unit vectors happens to round above 1.0: a coin toss a
    lamp, the same toss on every run. The village's 3, the walled town's 38
    and the harbour's lamps are subject to it too. The core tests run on
    the 300 m body, where it does not happen.
  - **The fix.** Compare the chord instead: a lamp is in a column when
    `|lamp - column| * R < 0.5 m`. The difference of two unit vectors
    holds its precision where their dot product does not.
  - **Saves.** A town's lamps are derived from its template, never saved
    (task 5.2), so every saved town gains its missing lamps on open. A
    player's edit in a lamp's cell still wins over it. Nothing saved
    changes.
  - **Verify.** A core test finds a lamp at its own column and none at the
    next cell on a body of the game's radius; an app test reads every
    lamp of every town on the shipped seed in its column; the desert's
    and the tundra's shots are retaken.
  - **Fixed (2026-10-02).** `a_towns_lamps_stand_over_their_cells` now
    repeats its lookups on the same patch at 4800 m, and failed on the
    old lookup. `every_towns_lamps_are_found_at_their_columns` asks each
    of the 36 towns' grounds for each of its lamps: the old lookup found
    255 of the 522, the chord finds all 522. It asks the ground it is
    given, not the installed one, so it cannot disturb a test beside it.
    The instrument `print_the_lamps_in_their_columns`, run alone, installs
    each town's ground and reads all 522 in the columns the planet
    generates.
- **Task 4.2b in detail, the moored boats (written 2026-10-01).** Survey
  T7: "you can use any boat you find".
  - **What the mockup moors.** 23 boats on the water: 14 rowboats, 6
    sailing boats and 3 canoes. Two more rowboats lie beached and two sit in
    the boathouses; those four are dressing, not craft.
  - **Which craft.** The game has two boats, the Tern (a sailing keelboat)
    and the Loon (a paddle canoe). A sailing boat is a Tern. A rowboat and a
    canoe are Loons, the game's nearest small boat. A rowboat of its own is
    a new craft for later. Recommendation taken (ask only with screenshots).
  - **The export.** A wrapper round the mockup's `boat` writes each boat on
    the water: its kind, where it lies in the mockup's metres, and its
    heading.
  - **A harbour's boats are craft records, made once.**
    - When the fleet first meets a harbour it holds no boats of, it makes
      them. Each is tagged with its berth, the harbour's site and its number
      there, so it is never made twice, even after the player sails it away.
    - Each lies at its berth on the planet's water, anchored to the seabed
      under it with the rode the game already gives an anchor.
    - A berth over land, or over water shallower than the craft draws
      (Tern 3 m, Loon 1.2 m), is skipped, and the skip is logged.
  - **Stowing.** Today every craft is stepped every tick, wherever it is.
    Six harbours would make that about 140.
    - The fleet keeps a craft further than 1.5 km from the viewer as its
      record only, stowed, and brings it back within 1.2 km, the towns'
      ranges. The craft the player is aboard is never stowed.
    - The save's fleet file is the live craft and the stowed ones
      together, so nothing is lost by stowing.
    - At a harbour about 23 craft are stepped, against 2 today. The cost is
      not measured in the cloud session. If it shows, a moored craft at rest
      can sleep.
  - **Records.** A craft's record gains `berth`, written only where there is
    one, so a fleet without harbour boats is written as it always was.
  - **Verify.**
    - App: a harbour's boats are made once and saved, and a second open
      makes none.
    - App: a craft is stowed past 1.5 km and back within 1.2 km at the
      pose it was left in.
    - App: a moored boat is boarded from the pier and paddled away, and a
      reload finds it where it was left.
    - Shots of the harbour with its boats.
  - **As built (2026-10-01).**
    - **The export** writes the 23 boats on the water (14 rowboats, 6 sailing
      boats, 3 canoes) from a wrapper round `boat`. The cells under them join
      the harbour's cells over the water, so they are charted.
    - **The draughts are the boats' own, not the spawn's.** A harbour's water
      is whole layers, mostly 1 to 2 m deep at the berths. The spawn places a
      new world's boats with margins (3 m for the Tern, 1.2 m for the Loon),
      and those left Holinghaven with no boats and Coringport with one.
      - From `vehicles.ron`: the Tern's keel reaches about 1.7 m under its
        waterline, and the Loon's hull is 0.36 m deep. So a mooring wants
        2 m for a Tern and 1 m for a Loon.
      - A sailing berth too shallow for a Tern takes a Loon, so the berth
        keeps a boat.
    - **On the shipped seed** 120 boats moor across the six harbours, 0 to 5
      berths skipped at each: Marenstrand 19 (5 Terns), Wickingstrand 18
      (4), Holinghaven 21 (1), Coringport 19 (3), Selingquay 20 (2),
      Corowstrand 23 (5).
    - **Stowing** runs once a second. The save's fleet file is the live
      craft and the stowed records together.
    - `every_harbours_boats_are_made_once_at_their_berths` and
      `a_craft_far_away_is_stowed_and_comes_back_where_it_was_left` hold it.
    - **The boarding test** (`a_harbour_boat_is_paddled_away_and_kept_where_it_was_left`):
      - a harbour's boat at anchor, tagged with its berth, is boarded, cast
        off and paddled more than 3 m away;
      - the world is put away and opened again, and the boat is back where
        it was left, the same craft, still tagged with its berth.
      - The walker boards from beside it, not from a pier, because the
        vehicles' test planet has no harbour. Boarding from a pier is the
        same F within reach.
      - Recommendation taken (ask only with screenshots).
  - **Finding: the drawn sea is half a metre under the layers' sea level
    (2026-10-01).** In the game's run at Holinghaven, 1 boat moored and 22
    berths were skipped. The app test moors 21 there.
    - `assets/config/water.ron` draws the sea `depth_offset_m` (0.5 m) under
      the sea level the layers are filled to. The game's `Sea::radius` is
      that drawn sheet, and the boats float on it.
    - The test took the layers' sea level for the sea, so every berth held
      half a metre more water in the test than in the game.
    - `print_the_seabed_under_the_harbours_berths` rules out the other
      suspect. The harbour's margin, eased to its terrace, lifts the seabed
      at 1 to 13 of a harbour's 23 berths, but leaves only 5 or 6 of them
      under a metre of water.
    - **Fix: the draughts are measured against the drawn sea.**
      - A Loon's hull is 0.36 m deep, so it moors in 0.5 m of drawn water,
        the water over a seabed one layer down.
      - A Tern keeps 2 m, its keel's 1.7 m and a hand.
      - The app test asks the depth of the same drawn sea the game does.
    - The cog floats on the drawn sea too (`sail-the-cog` design 6). A
      harbour's quay and piers stay where they are, whole layers over the
      layers' sea level, and so 1.5 m over the drawn water rather than the
      mockup's 1 m.
  - **Finding: the harbour has no sailboats (the owner, 2026-10-01, on the
    harbour shots: "where are all the boats in the harbor?", "Did they
    capsize?", "The original showed sailboats with tall masts").**
    - **Nothing capsized.** Every Holinghaven boat is made upright, deck up,
      on the drawn sea. An empty moored Loon stays upright at every depth
      from 0.3 m to 40 m over 30 s. Both instruments are in the tests
      (`print_the_pose_of_holinghavens_boats`,
      `print_an_empty_moored_boat_by_depth`). The game's boats read as
      upturned because each is the Loon, a bare teal shell with no rim,
      seats or paddler.
    - **No sailing berth floats a Tern** at Holinghaven. Its six sailing
      berths hold 0.5 m or 1.5 m of drawn water, and one is on land. Water
      2 m deep is 7 to 29 m away, or not within 30 m
      (`print_the_water_at_the_sailing_berths`). So every one took a Loon.
      Moving the berths out would break the mockup's layout, where the
      sailboats lie along the piers.
    - **The mismatch is the keel.** The mockup's sailboat is 6.6 m by 2.2 m
      with a 7.4 m mast, close to the Tern's 6.2 m by 2.3 m and 7 m. But it
      draws 0.45 m, a harbour boat. The Tern hangs 220 kg of ballast 1.45 m
      down on a fixed keel and needs 2 m of water.
    - **Measured: a Tern with its keel raised floats in shallow water.**
      With its ballast at the hull's bottom and its keel's foil and
      grounding point just under the hull, it draws 0.3 m. Moored in 0.5 m
      of water, empty, it heels 4° in a 6 m/s wind, 13° at 10 and 21° at
      14 (`print_a_tern_with_its_keel_raised`).
  - **Fix: the Tern gets a lifting keel (written 2026-10-01, before code).**
    Recommendation taken (ask only with screenshots).
    - Its keel, with the ballast in it, goes down as far as the water under
      it allows and no further. It rises at once when the seabed comes up
      under it, as a keel kicks up on the bottom, and lowers at a steady
      rate when the water deepens. In open water it is all the way down, as
      it is now, so the Tern sails as it did.
    - `TernSpec` gains `keel_rise_m` (1.08 m: the keel's tip from 1.5 m
      down to just under the hull), `keel_rate` (a third of the way a
      second) and `keel_clearance_m` (0.1 m). Raising it lifts the keel
      foil, the ballast and the grounding point together. A raised keel's
      foil is short and shallow: its area is the lowered share of it, never
      under a quarter, so a Tern in the shallows makes leeway.
    - How far it is down is `TernState::keel`, not saved. A loaded Tern
      starts with it down, and the water under it lifts it on the first
      tick.
    - It is drawn where it is, and the HUD reads "keel up" in shallow water.
    - **The berths.** A Tern now moors in 0.5 m of drawn water, as a Loon
      does. Every sailing berth that is not on land gets its Tern, tall
      mast and sail, as the mockup's do.
    - **Saves.** A world whose harbour already made its boats keeps them as
      they are, with Loons at the sailing berths. Its boats are records it
      holds, and none is replaced. A harbour first met from now on makes
      Terns.
    - **Verify.**
      - Core: in 0.5 m of water the keel rises clear of the seabed and the
        Tern floats upright. In deep water it is all the way down and the
        close-reach and in-irons tests pass unchanged. Lowered from raised,
        it takes about three seconds.
      - App: each harbour's sailing berths over water make Terns.
      - A shot of Holinghaven's quay with its sailboats, beside the
        mockup's.
    - **As built (2026-10-01).** As written. `TernSpec::lift` (rise 1.08 m,
      a third of its travel a second, 0.1 m off the seabed; the ballast is
      part 1 and the tip contact 0). On the shipped seed every sailing berth
      over water now makes its Tern: Holinghaven 5 (none before), 32 across
      the six harbours. `a_terns_keel_lifts_to_the_water_under_it` holds the
      keel, and the requirement is in `openspec/specs/player/vehicles`. The
      Tern's open-water tests (close reach, in irons, hull speed) pass
      unchanged.
  - **Finding: teal canoes the mockup never had (the owner, 2026-10-01:
    "there were no teal boats in the original", "I never asked for teal
    canoes", "make the row boats like the original!!").** The harbour's 14
    rowboats and 3 canoes were all moored as the Loon, the game's paddle
    canoe from the vehicles work (2026-09-24), drawn as a bare teal shell.
    That was recorded above as "A rowboat and a canoe are Loons" and taken
    as a recommendation, but the owner was never shown that it would not
    look like the mockup's boats. It did not, and it was wrong to take it.
  - **Fix: the small boats are the mockup's (written 2026-10-01, before
    code).** The owner's words are the decision.
    - **One cut for the mockup's small boats**, shared by the harbour's
      dressing and the craft. `pieces::dressing::small_boat` cuts the
      mockup's `boat` (its `hullGeometry` lofted in `boards` or
      `driftwood`, its thwarts, the rowboat's two oars laid across, the
      canoe's paddle) in a craft's frame: its gunwale at the craft's sheer,
      its bow forward.
    - **Its inside is dry.** The game draws the sea as one sheet, and an
      open boat floats with its inside floor under it, so the sea would
      show inside as if it were swamped (the mockup hides it with a
      stencil). The cut lays floorboards across the hull 3 cm over the
      waterline it floats at with someone aboard, as wide as the hull is
      there, so the sheet stays under them.
    - **The canoe is drawn as the mockup's canoe**, in driftwood, at the
      Loon's own size (5 m by 0.92 m), wherever the game has a Loon: the
      harbour's and the player's own. No craft is teal.
    - **The rowboat is its own craft, `Kind::Rowboat`**, at the mockup's
      size: 4.2 m by 1.45 m, 0.6 m deep, drawing about 0.25 m. It is the
      Loon's model on its own spec in `vehicles.ron` (`rowboat`): a hull of
      cells, a lateral plane and skeg, and the Loon's stroke for its oars,
      W and S to row and A and D to steer. Oars that pivot in rowlocks are
      a later refinement, noted, not built here.
    - **The berths.** A rowboat berth makes a Rowboat, a canoe berth a
      Loon, a sailing berth a Tern.
    - **Saves.** A harbour already met keeps the boats it made: its
      rowboat berths keep their canoes, now drawn in driftwood. A harbour
      first met from now on makes rowboats.
    - **Verify.**
      - Core: the small boat's floorboards stand over the loaded waterline
        and inside the hull; a rowboat floats upright empty and with its
        rower, and rows ahead; the spec validates.
      - App: each harbour's rowboat berths make Rowboats.
      - Shots of the quay beside the mockup's.
    - Recommendation taken (ask only with screenshots).
    - **As built (2026-10-01).**
      - `pieces::dressing::small_boat` lofts the mockup's hull at the
        craft's length, beam and depth with the mockup's sheer, and draws
        the rowboat's thwarts and oars from the same `fittings` the
        harbour's beached and slipway boats use. The canoe's paddle is cut
        apart (`laid_paddle`) and drawn only while nobody is aboard: the
        paddler holds the Loon's moving one.
      - **The floor follows the trim.** Laid level 3 cm over the level
        waterline, the canoe's floor was 2 cm under the sea at its stern:
        its paddler sits aft and trims it by the stern. `Hull::waterline`
        now solves the trim (the centre of buoyancy under the centre of
        mass), and `Craft::floor` lays the boards 3 cm over that, rising
        aft with it: about 1.7 degrees for the canoe, 1.1 for the
        rowboat. Through a whole swell their corners stay 2.6 and 2.7 cm
        over the sea.
      - **The rowboat's floor is flat.** On 0.2 m cells and a round
        section (power 2.5) it took its rower at a steady 14 degrees of
        heel: its waterline, in the bottom row of cells, was 0.6 m wide.
        On 0.1 m cells and a section of power 3.5, nearer the mockup's U
        (about 3 to 4), it floats within 0.03 degrees of upright with its
        rower and rows at 1.0 m/s. The Loon carries the same note.
      - The mockup tints some of its boats (a beige and a pale grey-blue
        wash); the game's berths carry no tint, so every boat is bare
        wood. Not exported; noted.
      - Oars that pivot in rowlocks are still the later refinement: the
        rowboat's oars lie across its thwarts while it is rowed.
      - Requirements in `openspec/specs/player/vehicles`: "The rowboat is
        its own craft and rows" and "An open boat's floor stays over the
        sea".
- **Task 4.2c in detail, the harbour's dressing (written 2026-10-01).**
  Third in the owner's order: "We're good on the mockup already. Approved.
  Implement in game".
  - **What the mockup puts in its harbour** (`makeCoast`), besides its
    buildings, piers, light, lanterns and boats:
    - **The fish market:** four stalls on the quay. Each has four posts, a
      plank counter, a sloped cloth awning and five boxes of goods (fish and
      wool).
    - **On the beach:** three net racks (posts, a bar and a see-through net
      hung both sides), two fish racks (seven fish on a bar), and two piles
      of lobster pots (5 and 3).
    - **At the piers:** a barrel and a crate at each of three pier heads.
      Twelve bollards line the main pier.
    - **On the quay:** three crates and two barrels.
    - **The boathouses:** each has a rowboat on the sand, three oars on end
      and four lobster pots.
    - **Two rowboats** lie keel up on trestles on the beach.
    - **The shipyard:**
      - a hull in frame on the beach: the lower planks on, eleven ribs bare
        above them, stem and stern posts;
      - its keel on blocks, with fourteen shores;
      - a stack of planks;
      - the slip, 4 m wide, down from the beach into the water.
    - **Not this step:** the people (townsfolk are a non-goal), and the
      cog's own barrel, crate and gangplank, which are `sail-the-cog`'s.
  - **The export.** Wrappers round the mockup's `marketStall`, `netRack`,
    `fishRack`, `lobsterPots`, `crate`, `barrel`, the bollards and the
    beached and housed boats write a `dressing` list into the template.
    - Each entry has its kind, where it stands in the mockup's metres, its
      height and its turn. A pile of pots also has its count, a crate its
      side and a stall its cloth.
    - The shipyard is written as one entry: where its hull lies, the slip's
      two ends and width, and where its planks are stacked. Its sizes are
      the mockup's constants, carried in the piece.
    - Only the harbour is written. `village.json` and `town.json` must
      re-export byte-identical (see below).
  - **Derived, never saved.** A town's dressing is placed from its template
    and its stored chart, as its lamps are. Nothing about it enters the
    town's record. The cells under it join the harbour's cells over the
    water, so the slip's are charted.
  - **Where each thing stands.** It stands on what is under it in the game,
    not at the mockup's height. That is the highest of a pier's deck, a
    building's floor and its cell's ground that is no more than half a
    metre over the mockup's own height.
    - A sea template rounds heights to whole layers, so the mockup's 0.75 m
      beach is 1 m in the game. The racks and pots on it rise with it.
    - A barrel at a pier head stands on the deck. A pot in a boathouse
      stands on its floor.
  - **How each is cut.** Each is cut in its own frame at its point, as the
    light is, from the boxes, cylinders and prisms the pieces already have.
    - **A new piece, the hull.** The mockup's `hullGeometry` and
      `hullSection` are ported into the pieces: a hull lofted from U-shaped
      sections. The beached and housed rowboats use it whole. The shipyard
      uses it planked to 0.45 of its depth, with its ribs drawn on the same
      sections. The cog will need it too.
    - **A new surface, the ramp.** It is a straight slope the walker stands
      on, from one height to another, as the mockup's `bridge` with `ramp`
      is. The slip is the first. The cog's gangplank will be the second.
  - **What the walker goes round.** These are the mockup's solids:
    - every barrel, crate and pile of pots;
    - each stall's counter and posts;
    - each rack, as a thin wall;
    - each bollard;
    - each beached boat, to its keel;
    - the shipyard's hull, 10.2 by 3.4 m.
    - The awnings stand over head height. The oars and the goods are
      drawn only.
  - **The net is see-through.** The mockup draws it from a 32 px texture
    with cords every 5 px, cut out where it is clear.
    - `export_town_textures.js` also writes the mockup's custom textures.
      The manifest marks the ones the mockup cuts out (its `ALPHA` set).
    - The game draws a cut-out texture alpha-masked at 0.5, on both sides.
    - The net casts no shadow. The shadow pass casts whole triangles, so a
      net would throw a solid sheet of shadow onto the beach.
    - Recommendation taken (ask only with screenshots).
  - **Only the harbour, for now.** The village and the walled town have
    dressing of their own: the market's stalls, the coopers' barrels and
    crates. Adding it to their `v1` templates would stand new solids in
    towns that saves already hold, Holbrook among them, where a player may
    have built.
    - Their dressing is its own step, as a new template version for new
      towns that leaves `v1` as it is.
    - The harbour reached `main` with #20, merged 2026-10-01 at 13:26, so
      a save made since holds its harbours. They were stored before the
      dressing and the cog charted their cells over the water.
      - In such a save, the dressing on dry ground and on the piers
        stands. What stands over cells the save never charted (the slip,
        the cog and its gangplank) is left out and counted (`Built`'s
        `dressing_skipped`, in the town's log line).
      - Nothing a player made is moved or removed. The dressing is derived
        and stands over whatever is there, as the lamps do.
    - Recommendation taken (ask only with screenshots).
  - **The cost.** The dressing is triangles and solids in each harbour's
    town meshes, logged with the town's count. Its frame cost is not
    measured in the cloud session.
  - **Verify.**
    - Core: every dressing thing of the harbour stands within 2 cm of the
      deck, floor or ground under it.
    - Core: a walker on the quay is held by a stall's counter and goes
      round a barrel.
    - Core: a walker goes down the slip from the beach into the water.
    - App: every harbour cuts with its dressing.
    - Export: the village and the walled town re-export byte-identical.
    - Shots of the fish market and the nets at 11:00 and 22:30, and of the
      shipyard at 11:00.
  - **As built (2026-10-01).**
    - **The export** writes 58 things and the shipyard: 4 stalls, 3 net
      racks, 2 fish racks, 16 pots, 5 barrels, 6 crates, 12 bollards, 6
      oars and 4 boats on land.
      - The coopers' barrels and crate stand inside their houses. They are
        furniture, which every town's houses take in their own step, and
        are left out.
      - A pot in a pile is written with its lift over the pile's foot, so
        the pile stands as a pile on whatever is under it.
      - `village.json`, `town.json` and the 70 painted textures re-export
        byte-identical. `net.png` is new, marked `cut` in the manifest.
    - **Placing a thing.** Its frame is read from the chart at its point and
      a metre east and south of it. The mockup's offsets go through that
      local map, which carries the chart's turn, any mirroring and its
      stretch together. Those cells join the harbour's cells over the water.
    - **Two departures from the mockup**, both on the beached rowboats:
      - the mockup stands each one a hull's depth over its trestles, and
        here its gunwale rests on them;
      - the mockup runs its trestles along the keel, and here they run
        across the boat.
    - **A harbour stored before this** charted no cells for its slip, so a
      thing off the chart is left out and counted, not an error. The town's
      log line names how many stand and how many are off.
    - **The net** is drawn alpha-masked at 0.5 through the field-lit
      material's own `alpha_discard`, and left out of the town's casters.
    - `a_harbours_dressing_stands_on_what_is_under_it` (18 or more things
      on a pier's deck, the rest on the ground),
      `a_walker_is_held_by_a_stall_and_goes_round_a_barrel`,
      `a_walker_goes_down_the_slip_into_the_water` and, in the app,
      `every_harbour_lays_on_its_sea_and_cuts` (each of the six harbours
      has 61 dressing pieces, none off its chart) hold it.
    - The main pier's walker test now goes by the pier's own pieces: the
      mockup's crate at the head stands on the pier's middle.
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
