# Design: Luanti's liquid rule on the hex grid

## Context

**Luanti's rule** (`src/servermap.cpp` at `a10dc8d`,
`transformLiquidsLocal` and `get_max_liquid_level`):
- **Node states.** A node is a liquid *source*, a *flowing* node with a
  3-bit level (7 nearest the source, 0 the thinnest film) and a "flowing
  down" bit, or something else. That something is either *floodable* (air)
  or *neutral* (solid, or another liquid).
- **The queue.** Only nodes in a queue are looked at. An edit queues the
  node, and a changed node queues its neighbours. A tick takes at most the
  queue's length at its start (`liquid_update`, 1 s; `liquid_loop_max`,
  100,000), so water spreads about a node a second.
- **One node's update.** It reads its six neighbours: up, four sides, down.
  - A source stays a source.
  - A node with **two or more same-level or upper sources around it** becomes
    a source if the liquid is renewable. The source below is not counted.
  - Else a node beside a source takes level 7.
  - Else it takes the highest level its neighbours offer:
    - a flowing node above gives its level plus 4 (`WATER_DROP_BOOST`),
      capped at 7;
    - a side neighbour gives its level less 1, if it is not itself falling.
  - It is liquid only if that level is within `liquid_range` (8, so levels
    0 to 7 all count); otherwise it is air again.
- **Falling first.** "Flowing down" is set when the node below is air or
  flowing. A falling node offers nothing sideways, so water drops before it
  spreads.
- **Viscosity** (water 1, lava 7) slows how fast the level changes.

**Here today**: water is one static material at sea level (proposal).

## Decisions

**1. The hex neighbourhood is eight: up, six sides, down.**
- Luanti's order is kept: upper before same level before lower.
- A pentagon cell has five sides. The arrays hold eight and count what is
  there, as the topology's explicit 5/6 neighbour tables already do.

**2. Renewing takes three sources on the hex grid, not two.**
- On Luanti's square grid, two sources side by side share no neighbour, so a
  pair does not spread.
- On the hex grid, two adjacent cells share two neighbours. With Luanti's
  threshold of two, each shared cell becomes a source, which makes new
  pairs, and any two sources poured side by side flood every floodable cell
  on their level. The mockup shows this runaway (the preset "Luanti's
  threshold on hex").
- With three: a cell beside a straight row of sources touches two of them,
  and a cell beside a triangle of three touches at most two, so neither
  grows. A cell in a concave corner touches three and fills. This is the
  behaviour Luanti's two-of-four gives on squares: pools fill their corners,
  and lines stay lines.
- Recommendation taken (ask only with screenshots). The mockup has the
  toggle, so the owner can see both.

**3. Range and levels stay Luanti's: 8 levels, range 8.**
- Seven cells of spread is 20 m on the hex grid (2.833 m cells), where it is
  7 m in Luanti. That is a gentler sheet of water for the same 1 m layer.
- The mockup has a range slider (1 to 8) to judge it. Recommendation taken:
  8, Luanti's default, until the owner says otherwise from the mockup.

**4. A flowing cell's surface stands at (level + 1) / 8 of its 1 m layer.**
- That is Luanti's own drawing rule. A source and a falling cell are full.
- On the hex grid, each cell's top is one flat hexagon at that height, so a
  spreading sheet steps down by eighths. Luanti instead slopes its corners
  between neighbours, which is not a cell look.

**5. Only edits start flow; old worlds do not move.**
- Generated water is all source and is never queued. The queue starts only
  from an edit: a dig, a placed block, or a poured or removed source.
- So opening a world never changes it. A world made before this change has
  the same water cell for cell until someone digs beside it, and then only
  that water moves (CLAUDE.md, "Saved games survive every change").
- No generator change, so no new generator version.

**6. Sources are saved; settled flowing water is derived.**
- Placing or removing a source is a world edit. It takes the durable path
  and names its author. A source the renew rule makes is authored by the
  water. The rule only floods floodable cells (air), never a player's block,
  so the world's change never overwrites the player's.
- Flowing cells are not saved one by one. On load, every saved source
  beside air is queued, and the flow re-runs to the same settled state.
- This relies on settling being a function of the sources and the solids
  alone. The mockup checks it: "Settle from scratch" re-runs every source
  and compares the result with the incremental state, cell for cell.
- Water still moving when the game closes picks up where the sources put
  it. Nothing a player made is lost; at worst a falling sheet restarts.

**7. Save format, rendering and swimming (later, not in the mockup).**
- Save format: flowing water needs its level. Either a material per level or
  a level field. A new save format version is needed, with the old one
  readable: its water is all source.
- Rendering: the water top is today the top of the column's water run
  (`Column::water_surface`, the shader's water word). It must read the
  flowing level instead. That is a shader interface change, so the pipeline
  is validated when it lands.
- Swimming: the player swims in flowing water as in a source. Boats float
  on the sea's table only, as now.
- Cost: the queue is bounded per tick like Luanti's. It is measured on real
  hardware before it ships (perf rule).

**7a. The mockup draws water as M25's water sim does (owner, 2026-11-15).**
"sync m28 with these changes", after the owner asked of M25's Water sim
tab for "the frothy walls and edges like the river demo you had and show
flow direction" and found it "missing the wall/waterfalls part". The look
is shared, as the rule is: `docs/mockups/water-flow/water-look.js` holds
M25's water material (waves, foam, lamps, haze), its hex caps with a
foaming fall sheet and lip on every step down, and the sim's water builder
(each column's top water surface, a flow vector towards the neighbours
whose water stands lower, by how much lower, and a chevron on each flowing
cap behind a Flow arrows toggle). Both pages load it, as both load
`water-flow/liquid.js`. M28 feeds it its own lamps, a depth map baked from
the patch, and its fog. One fix rides with it, for both pages: a source is
drawn 6 cm below the level-7 water beside it, so the flow is worked out on
the head (a source an eighth above level 7) and no fall is drawn for a step
under 7 cm; arrows no longer run into a pool of sources. The rule is untouched: every preset settles in the
same ticks to the same cell counts as before. This is the mockup's look,
not a decision about the game's water renderer, which stays decision 7.

**7b. The mockups' water reads right from any side (owner, 2026-11-15).**
Looking at M28 on a phone at its default view, the owner wrote four notes.
They are the mockup's look, and the rule is untouched (every preset settles
in the same ticks to the same counts):

- *"The very front part of the cell doesn't have any geometry. Front lines
  of generation?"* M28 drew every column's side down to the patch's base,
  but a sea sheet ran on past the patch to the horizon at sea level and,
  near-opaque at a glance, covered those walls from 0 down to -6, so the
  near edge read as a grass lid on nothing. M25's sim skipped the bottom
  layer's outer sides outright, so its 1-layer front had no wall at all.
  Now the patch is cut like a cake: every side down to a stone base, a stone
  floor under it, the outer sea sheet gone (M28's sea is its own sea cells),
  and where water meets the patch's edge it stands as a still wall down to
  its bed (`simWater`'s cuts, drawn by `makeCaps` as a third kind of face).
- *"some zdepth issues with the waterfall/transparency sorting"*. The caps,
  lips and falls were one transparent mesh with no depth written, so its
  triangles blended in build order and a fall behind a cap could paint over
  it. `makeCaps` now draws a depth-only twin of the mesh first (the same
  shaders, so the same waves; render order 3.5, after every opaque thing),
  and the colour pass keeps only the nearest water at each pixel. Water
  behind water is hidden, as the game's single water layer is; the arrows
  test against that depth, so a fall in front hides the arrow behind it.
- *"Add button to toggle arrows on off, the game should have this too."*
  The toggle reads "Arrows: on" / "Arrows: off" with a lit dot when pressed,
  in both pages. The game's overlay is requirement "A flow-arrow overlay" in
  `specs/planet/water` (task 3.4).
- *"the edges also dont have flow ... we can see 1 arrow in a cell with 4
  pads."* A cap pointed only toward lower water, so the spreading front
  (dry beyond) and a ring at rest against a wall came out with no vector.
  The vector is now the way the water is going, down the head: toward lower
  water; away from higher water that reaches this layer; toward a dry cell
  it can spread into (open at this layer, and a level above 0 to give) or
  fall into (open at this layer and the one below). A source points only
  where water leaves it, and a still pool (every head equal) has none.
  Basin: 18 caps with arrows where there were 6; spring: 230 where there
  were 186, with every flowing cap that has somewhere to go carrying one.

**7c. Flow is shown by panning a water texture, Tenebris C's way (owner,
2026-11-15).** "Add UV scrolling and water texture or shaders to show the
flow map. This is how tenebris c does it." Read at `RubenTipparach/tenebris`
commit `ef9865166349c7a3c4ae09acdd20640daf85b4fd` (2026-08-15, "Merge pull
request #89"), cloned read-only:

- `tenebris-c/src/client/hex_mesher.c`, `bake_water_flow_uv`: at mesh time
  each water cell's top-cap vertices get the cell's flow direction, from
  `world_water_flow_direction` (`src/core/world_water.c`: the cell's centre
  minus the mean position of the source cells it descends from, normalised),
  projected onto the tile's tangent plane and written into the vertex's UV
  channel. Side walls, bottom caps and cells with no origin (generated sea,
  sources) get (0, 0): "spawn blocks stay static, flow blocks pan".
- `tenebris-c/src/shaders/water.glsl`, `water_fs` (the desktop path): no
  texture. The surface is a three-octave gradient-noise fBm (`gnoise3`,
  `fbm3`) whose finite-difference gradient bends the normal. Flow moves the
  noise's sample point: on a cap, `flow_pan = -flow_world * u_time * 0.35`
  (weighted by how horizontal the face is); on a vertical face, a shift along
  the radial by `u_time * u_flow_speed_falling` (`flow_uv_speed_falling`,
  1.0 in `assets/config/water.yaml`), so the pattern streams down a fall.
  One phase, no reset: the vector is constant over a cell, so the pan never
  stretches the pattern and no two-phase blend is needed.
- `tenebris-c/src/shaders/water_mobile.glsl`, `water_mobile_fs` (the phone
  path): the same idea with a texture, `assets/textures/water_normal.png`
  (128 px, made by `tools/gen-water-normal.py`), sampled triplanar in two
  layers at two scales and drift speeds, the fall shift added to the sample
  point on vertical faces (no cap pan on mobile).
- `tenebris-c/water.md` section 8 plans the rest: the cap's UV scrolled along
  the net flow at `flow_uv_speed_per_level` per level of drop (0.25; parsed
  in `config.c`, not yet used by the shader), a falling side scrolled from
  V 1 to 0, still and flowing atlas slots.
- `tenebris-c/src/client/renderer.c`, `renderer_draw_water_flow_debug` and
  `flow_arrow_visit`, toggled by F5 in `src/client/main_web.c`: a debug
  overlay of a red line arrow on every water cell with a flow direction, a
  hair above the cap, depth-tested and not written.

The mockups do it the same way with a pixel texture. `waterMat` samples
Tenebris's own water tile (`tenebris-rs/assets/textures/water.png`, 16 x 16,
three colours; byte-identical in `docs/mockups/planets/tenebris/water.png`),
embedded in `water-look.js` as its 256 colour indices so no file travels with
it, nearest-sampled, 2 m a tile. The three indices are drawn as three tones
of the water they are on (its own colour, a ripple lifted toward the sky, a
glint lit as the foam), so the pattern is palette-true and lit by the lamps
at night. Each cap pans the tile along its own flow vector at 1.6 m/s per
unit of flow, the tile's streaks laid along the flow; a falling sheet pans
it down at 2.4 m/s, streaks standing; a still cap and a cut stand still. As
in Tenebris, the vector is constant per cell, so one phase serves. The fall
froth now comes from the tile (its old hash streaks are kept only where a
page asks for no texture), the cap streak foam is lighter, and the pattern
fades out between 45 and 120 m, where a 12 cm pixel would shimmer. The sea
(`tex` 0) is unchanged. Measured in M28 (SwiftShader, two frames a second
apart): a flowing cap changes 10.8 % of its pixels by more than 24 levels,
still sea 0.4 %.

**7d. The game's flowing water (proposed, not built).** The game's cap
already has a flow hook (requirement "The water surface streams along a
per-cell flow"). When flowing water lands, its look is Tenebris's: the
flow vector baked per cell at mesh time, the top's pattern panned along it
by speed, a falling face panned down, still water still, and a flow-arrow
overlay to check it (tasks 3.4, 3.5). Proposed only; no game code until the
owner approves the mockup and asks.

**7e. Water is meshed per cell, as Luanti's liquid mesher does (owner,
2026-11-15).** "Same issues as before for the leading edge, also theres no
flow maps on the edge as well and no walls, note that you missed a critical
part of the luanti/minetest water implementation that shows the water
falling", and "water should generate blocks vertically as it falls too".
The flat caps of 7a to 7c drew one cap per *column*, at its top water, with a
sheet from that cap down to the next column's top: every cell under the top
of a column was never drawn, so a falling column showed as one flat cap
hanging at the ledge's height a cell out from the cliff, and the front of a
sheet ended in a flat, wall-less edge.

*The rule was not the problem.* Measured on M28's spring (a Node run of
`water-flow/liquid.js` with the page's ground, no page code): the first
falling cell over the cliff appears at tick 4 (`(3, -1)` at layer 6, level
4, `down` set); by tick 6 the same column holds falling cells at layers 6, 5
and 4, by tick 8 at 6 to 2, by tick 10 at 6 to 0, the column growing one
layer a tick, every cell below the top at level 7 (its upper neighbour's
level plus `WATER_DROP_BOOST`, capped) with `down` set, standing on the
plain's own sheet. Settled (tick 40), twelve columns fall the six layers
from the ledge to the plain. Luanti's rule already makes real falling cells,
block by block; they were simply not drawn. The rule is unchanged, and every
preset settles in the same ticks to the same counts.

*Luanti's mesher*, read at `a10dc8dbff3b0af8295ae55df44f65b1d9ee92c7`,
`src/client/content_mapblock.cpp` (LGPL-2.1-or-later):
- `drawLiquidNode` draws one *flowing* node (`NDT_FLOWINGLIQUID`):
  `prepareLiquidNodeDrawing`, `getLiquidNeighborhood`,
  `calculateCornerLevels`, `drawLiquidSides`, then `drawLiquidTop` unless the
  node above is the same liquid, and `drawLiquidBottom` if the node below is
  neither the same liquid nor solid. A *source* (`NDT_LIQUID`) is drawn by
  `drawSolidNode<true>` as a full cube, a face wherever the neighbour is not
  the same liquid, not solid and not `CONTENT_IGNORE`.
- `prepareLiquidNodeDrawing`: `top_is_same_liquid` (the node above is the
  source or flowing form) and `draw_bottom`.
- `getLiquidNeighborhood`: the 3 x 3 nodes round it on its own layer, each
  with a `level` in node units (-0.5 the floor, +0.5 the top): a source
  +0.5, a flowing node `-0.5 + (level + 0.5) / range` (range 8, so level 7
  stands at 15/16 of the node), anything else -0.5; and whether the node
  above that neighbour is liquid. `CONTENT_IGNORE` is left out.
- `getCornerLevel`, for each corner over the 2 x 2 nodes sharing it: if any
  of them has liquid above it, or is a source, the corner is full (+0.5).
  Else the flowing nodes' levels are averaged, and air is counted; two or
  more air of the four pull the corner to the floor (`-0.5 + 0.2 / BS`, 2 cm
  up). No flowing node and fewer than two air: the middle (0).
- `drawLiquidSides`, for each side: no face against the same liquid, unless
  this node has liquid above and that neighbour does not; no face against a
  solid. The face runs from this node's two corner heights (or the top, if
  liquid is above) down to the floor, or, against the same liquid, down to
  the corner heights. These are the walls at the front of a sheet and the
  sides of every falling node.
- `drawLiquidTop`: a quad through the four corner heights. Its flowing
  texture is turned to the flow: `dz` is the corners on -Z less those on +Z,
  `dx` those on -X less those on +X, so the direction is down the corner
  gradient; a symmetric top gets (1, 0). Without reflections the normal is
  `(dx, 1, dz)` normalised. The sides carry the flowing texture, which runs
  down.
- `drawLiquidBottom`: a flat face at the floor.

A falling node therefore has liquid above it, so all its corners are full,
it draws no top, and its sides stand the whole node down to its floor: a
falling column is a stack of whole blocks, each drawn with its own sides,
and the node at the top of the fall, with nothing above it, slopes from the
lip down to its own level.

*On the hex grid* (`PBDWater.liquidMesh` in `water-flow/water-look.js`, which
`simWater` now calls, so M25's Water sim and M28 both take it):
- A cell has six corners, each shared by three cells: the cell and its two
  side neighbours on either side of that corner (corner *i* lies between
  sides *i - 1* and *i*). `getCornerLevel` runs over those three: full if any
  has liquid above or is a source; else the mean of the flowing levels; air
  pulls the corner to the floor at two or more of the three (Luanti's "half
  or more" of the cells round a corner); a cell outside the patch is left
  out, as `CONTENT_IGNORE` is.
- Heights are Luanti's: level *L* stands at `(L + 0.5) / 8` of its layer, a
  source and any cell with liquid above it at the full layer. (Decision 4's
  `(level + 1) / 8` and 7a's 0.94 for a source are retired with the caps.)
- The top is six triangles from the corners to a centre at their mean, drawn
  unless the cell has liquid above. Its normal follows the slope.
- A side is drawn per Luanti's `drawLiquidSides` across each of the six
  edges: against air down to the cell's floor; against the same liquid only
  where Luanti draws it. The patch's own boundary is the one change: Luanti
  draws no source face against `CONTENT_IGNORE`, but the mockups cut the
  patch like a cake (7b), so every water cell at the boundary draws its side
  there, down to its floor, for sources and flowing cells alike.
- The bottom is drawn where the cell below is air (Luanti's `draw_bottom`).
- The flow vector is Luanti's: down the corner gradient. On hex it is the
  least-squares slope of the six corners, `-sum(h_i c_i) / (3 R^2)` with
  `c_i` the corner offsets; a symmetric top has none (a pool stands still,
  where Luanti turns it to +X). It drives Tenebris's tile and the arrows: the
  tile pans along it, faster on steeper water; the arrow lies on the slope.
- Walls: a flowing cell's sides run Luanti's flowing texture downward, the
  Tenebris tile panning down with froth, and every falling cell is a whole
  block of it; a source's sides stand still. A falling cell's open sides
  also carry a down-pointing arrow, so a fall shows its flow with the
  arrows on.
- The depth pre-pass, the night lamps and the arrows' toggle are unchanged;
  the column-top record (`SW.cells`) is kept for M28's depth map.

The game's water mesher should do the same (requirement "Flowing water is
meshed per cell from its corner levels", task 3.6). Proposed only; no game
code until the owner approves the mockup and asks.

**7f. Rivers run on the rule too; M28 folds into M25 (owner, 2026-11-15).**
"water that flows - make sure the rivers use that too, so merge these
artifacts together so we get it more consistent". M25's river, dam stream and
pond, and marsh pools now run on `liquid.js` and draw with `liquidMesh`
(`ocean-environments` design 12), and M25's Water sim tab carries M28's
patch, presets and controls; `water-flow.html` points there. The rule is
unchanged and every preset settles as before.

What a river asks of the rule, measured on M25's river (a Node run of
`liquid.js` on its cells): a flowing reach runs eight cells from where it is
fed and dries beyond, and a step down feeds the reach below to the lip's
level plus three for one layer, to the full seven for two or more. A river on
cells therefore steps a layer at most four cells (11 m) apart, or two layers
at most seven apart; the river's old meanders, a layer every 22 to 28 m,
dried 17 m below each step. Still water (a spring, a pool, a pond, the bed
under a deep reach) is sources, and a reach's lip is kept one layer deep so
the still water of one reach never touches the flowing water of the next,
where renewal could turn the river to sources. The game's river pass
(`ocean-environments` task 8.1) has to lay rivers out the same way: proposed
only.

**8. The game's code is written from the rule, not translated.** The mockup
is a line-by-line port of LGPL-2.1-or-later code and says so. The Rust
version is written from decisions 1 to 6 and Luanti's rule as described in
Context, and the mockup is the reference for behaviour. Recommendation taken
(ask only with screenshots). The owner can choose to take the LGPL code into
the game instead, and that is the owner's call.

## In the engine (owner, 2026-11-15: "commence implementation of all env changes")

M25 and M28 are approved, and the request to build came with them. How the
rule goes into the game, in the order it is built (branch `claude/env-water`):

**E1. A flowing cell is a material.** `Material::Flow0` to `Flow7` and
`Fall0` to `Fall7` (codes 64 to 79, far from the blocks' run) are flowing
water at a level, still and falling; `Water` stays the source. The column
keeps two bytes a layer, so the tier costs nothing more to hold or clone.
`Material::is_water` is what every reader asks (the walker swims in both,
decision 7; the renderer's water code is both). A flowing top's surface is
Luanti's `(level + 0.5) / 8` of its layer (`liquid::top_fraction`).

**E2. The rule is `pbd_core::liquid`**, written from decisions 1 to 6
(decision 8): `transform` is a pure function of a cell and its eight
neighbours (`None` for a pentagon's missing side or off the region), and
returns the new state and which neighbours to queue, in Luanti's order;
`Queue` is the unique queue and the tick, at most the queue's length and
`LOOP_MAX` a tick. Held to the mockup by a fixture the mockup's own
`liquid.js` writes (`tools/water_flow_fixture.mjs`): all eight presets
settle in the same ticks to the same field, cell for cell. Water enters
only air: a block, a lamp or anything else is neutral, so the rule never
removes a thing a player made.

**E3. It runs over the column tier**, once a second (Luanti's
`liquid_update`): the tier is the `Grid`, a cell is a slot and a layer, the
rim and every cell off the tier are neutral. Each tick's changes go in as
an edit does (the changed records and their neighbours repacked, the
contact and the fine set's version moved), without the relight: water does
not stop light in the field.

**E4. Saves (this replaces decision 6's "derived").** Every change the rule
makes is an edit authored by the water (`@wwater:flow/0`), with no slots:
a source as `Water`, a flowing cell as its `Flow`/`Fall` code, so its level
is saved. Derived was the plan, but the tier is regenerated from the edits
each time it moves, and a derived field would re-run on every rebuild. The
water's edits do not pass the world's yield set: water filling a hole the
player dug is what water does, and it only ever fills air. Opening a world
queues every cell of the tier that holds a water edit, and its neighbours,
so water still moving at the close picks up and a settled field changes
nothing. A save from before has no water edits, so nothing is queued and
every cell is where it was (decision 5; tested).

**E5. Pouring.** There is no bucket, and nothing yields a water block,
though the place path places any held block and water has its thumbnail.
Recommendation taken (ask only with screenshots): the place path is the
pour (a held water block places a source, which queues the flow), and a
debug key, F6, pours one source where the place ray lands, through the same
edit path. A dig or a placed block next to water queues the flow the same
way. Captures pour with `--pour`.

**E6. The water mesher** (task 3.6) is a CPU mesh of the tier's water,
rebuilt when the fine set's version moves: Luanti's `drawLiquidNode` on hex
as design 7e has it (corner levels from the three cells round a corner,
tops through them, sides where water meets air, falling cells whole blocks,
bottoms over air, the flow down the corner slope). It leaves out what the
sea sheet already draws: the top of a column's water at sea level where the
cell's record is under the sea. One mesh, Tenebris's water tile
nearest-sampled and alpha-blended, unlit with the tier's light field in its
vertex colour (sky by the daylight, the block light warm), so it is dark at
night and lit by lamps. The mesh exists from the first frame (an empty
placeholder), so the loading screen builds its pipeline.

**E7. The flow is shown as design 7c says** (task 3.5): each frame the
tops' texture coordinates move along the cell's flow vector, faster on a
steeper slope, and a falling face's move down; a still cell and a source
the water does not leave stand still. The flow-arrow overlay (task 3.4) is
a second mesh of arrows over every flowing top and down every fall, hidden
unless F5 or the player setting "Flow arrows" (off by default) turns it on.

**E8. What was built, and what was not (2026-11-15).** All of E1 to E7 is
built and tested in `pbd-core` and the tier (see tasks). Not measured: the
frame cost (3.3). Not checked: the pan's "two frames a second apart" (3.5).
A set built while water is ticking would land without the ticks in flight,
so the tick waits while a set is in flight; the existing "undone when it
lands" behaviour of edits is unchanged. The sea floods a channel dug from it
as sources to sea level (`volumetric-water` phase 2) rather than as M28's
"Channel from the sea" flowing sheet: in the game the sea is the sea's
table. Recommendation taken (ask only with screenshots).

## Second pass (lead's request, 2026-11-15)

**E9. One water: the game's water is drawn by the sea's own fragment.** The
first cut drew poured water with an unlit tile mesh, and it read as dark
confetti beside the sea. Now the mesher's triangles go to the water pass as
a storage buffer (three lanes a vertex: position and the sky's share; the
face's normal and kind; the flow in the face's tangent frame) and are drawn
by a second vertex entry, `vertex_flowing`, into the cap's own fragment, in
the cap's render pass and depth. So a pool and the sea are one water: the
sea's colour by depth, its light by the hour and the cover, its sky
reflection, fog and glint, and no seam where they meet. The flow streams the
sea's waves along a top (Tenebris C's desktop way, 7c, the cap's flow hook)
and down a falling face, which carries white water; a still face neither.
A face seen from behind from dry air is discarded (its front draws the
block); from inside the water it is the murk, as under the sheet. The tile
is gone from the game; it stays the mockups' look. Recommendation taken
(ask only with screenshots).

**E10. The walker swims in what is drawn.** The walker's water was the
height field's sea against the untided sheet. It now asks the column first:
the water the body is in, a pool or a flowing cell at its own level, and the
sea's water at the drawn sheet with the tide on it; a dry column under open
sky still swims where the tide stands over its ground (a beach at high
water); off the tier, the height field's sea to the tided sheet.

**E11. The wooden bucket.** The way a player uses water, after E5's debug
key: two goods appended to the goods table, `bucket` (a wooden bucket, 4 to
a slot) and `water-bucket` (a bucket of water, 1 to a slot), with 16 px
icons from `tools/gen_item_icons.py` in the fence's wood. Made at the
workbench from 3 planks and 1 rope (`recipes.ron`, `bucket`). Held, the
right button:
- with an empty bucket, aimed at a source (the ray stops at water for it,
  as it does not for a block), takes the source and the bucket is full;
  flowing water cannot be taken, as in Luanti;
- with a full bucket, pours a source where a block would be placed, and the
  bucket is empty.
Both are one edit and one hands change on one line, the place path's own
transaction (`Hands::Swap`), so the source taken and the bucket filled reach
the disk together. The rule is `pbd_core::liquid::bucket`, tested.

## The owner's notes on falls (2026-11-15)

**E12. No rain on vertical water.** The owner: "when raining - no droplets
on vertical water." The rain's rings are scaled by how much a face looks up
(`smoothstep(0.7, 0.95, horizontal)` in `water.wgsl`), so a fall's or a
wall's face takes none and a top takes them all.

**E13. Froth on falls.** The owner: "vertical water should have a bit froth
to it, probably something easy to do via shaders." A falling face carries
white streaks of froth, noise stretched down the face and scrolled down it
with the fall, denser at a fall's lip and foot (the mesher's froth weight,
`flow_uv.x` on a falling face); a still wall keeps none.

**E14. Splash at a fall's foot.** The owner: "can you also add some particle
fx to bottom of the waterfall? more fx based on distance it fell". The mesher
records every fall's foot (water not falling with falling water on it) and
how many layers fell onto it; each frame, spray drops are thrown up and out
and fall back, and a low white mist puffs, both more, higher and wider for a
taller fall (3 + 6 per layer drops up to 160, a puff per two-thirds of a
layer up to 24), at most 4,000 quads, out to 70 m and faded over the last
15 m, lit by the sky's share and the hour. A pure function of time: nothing
is stored. One unlit alpha-blended mesh, born at startup, so the loading
screen builds its pipeline.

**E15. The rivers' water is the rule's.** The generator seeds only sources,
on their own bed and never over open air; a version 8 tier's build runs the
rule over them until nothing is queued and a sweep of every water cell
changes nothing (`ColumnTier::settle_rivers`), so falls, sheets and riffles
are exactly what the rule keeps (`a_freshly_built_tier_s_rivers_are_what_the_rule_keeps`).
Derived, never saved; older worlds are not settled. The banks are held on
every side (the water's top is a metre under the lowest ground round the
valley's rim), yet a fall's foot can still spread its eight cells past a
river's valley (fixed in the fifth pass, E16).

## Fifth pass (lead's request, 2026-11-15)

**E13, again: froth as streaks.** The fourth pass's froth read as camo
blotches. It is now thin bright vertical ribbons: two octaves of gradient
noise stretched about seven times along the fall, scrolled down at 4 m/s,
thresholded to narrow bands, over a soft white veil, with a churned band at
the lip and the foot (the mesher's froth weight). The noise's coordinates
are taken modulo 128 m first: at planet-sized coordinates the noise's hash
loses its precision and the streaks went flat.

**E13, a third time: the lip and the speed.** The owner, on the after shot:
"can we have the edge transition into froth? and somehow show that it is
speeding up in velocity/flow in to the wall". The mesher gives each top
corner a lip weight (one where a side's water falls, falls from under it,
or this cell's own water falls below) and each falling face's corners how
far under their fall's lip they are, in the flow lane's third component.
On a top, the last metre or so before a lip whitens into streaks along the
flow that run faster toward it (two phases of the flow half a cycle apart,
blended, so the speed never shears the pattern). On a fall, the ribbons are
read at the time the water left the lip, free fall from 1 m/s,
`(sqrt(v0^2 + 2 g d) - v0) / g`, so they accelerate down the face and
stretch with sqrt(2 g d): clear just over the lip, white ribbons lower down,
the churn at the foot (`froth-lip-before-after.jpg`).

**E16. A river is held: levee and sealed rock.** Measured on four tiers
over falls (`a_freshly_built_tier_s_rivers_are_what_the_rule_keeps`), the
water that spread past a river's banks was not on the surface: it ran down
through caves the worms open under and beside a river's bed, and spread in
them, tens of metres from the river and tens of metres under the ground. Two
changes to version 8 (not yet in any played save): every column within a
river's banks plus 15 m keeps its rock, cave-free, from four metres under
the sea's level up (`rivers::sealed_from`, applied after the worms in
`column::generate_edited`); and a 3 m levee round a valley stands a layer
over the reach's water and its fall, its banks rising to it and its outside
falling away at one in one (`rivers::carve`). With the levee, the water's
top goes back to a layer under the lowest of six points round the rim. A
plunge pool's extra depth eases out up the bank, so the seam report holds
at none of 4,000 edges. The test now
asserts no flowing water over the sea's layer outside a river's banks, on
four tiers. Water at the sea's layer in a sea cave is the sea's and is not
counted.

**E17. The capture rig flies anywhere.** `--fly` takes `--yaw` (compass
degrees, 0 north, 90 east) and `--pitch` (degrees, negative down) beside
`--at` and `--height`, so a capture can look down on a river from the air.

## The fall look, in a mockup first (owner, 2026-11-15)

Two in-game fall looks were rejected (pass 5, `c620183`: "all the afters are
bad"; the bowed strands, `b9c81ec`: "too smeared, theres also a geometric gap
at the edge"). From here a fall's look is approved in a mockup before the game
changes (CLAUDE.md, "Waterfalls are designed in a mockup first"). The mockup is
`docs/mockups/waterfall.html` (`waterfall/waterfall.js`). It shows the game's
test case (`--pour 1 --pour-ahead 12 --pit 9`: a source beside a 9-deep pit of
seven cells) and a 2-layer step, on 2.833 m cells with 1 m layers. It shows them
by day and at night under three lamps, with toggles for rain and splash. Its
screenshots are in `docs/screenshots/mockup-waterfall/`. Not built in the game.

The froth is a GLSL port of `water.wgsl` at `dc0c5cb`, the falling-face branch
(`in.kind == 2u && horizontal < 0.5`). It keeps the same `gnoise3` (integer
lattice hash), the same `fbm3`, the same tangent frame (body X projected off the
face) and the same constants. Variant A is that port unchanged.

One finding from the port: the froth's `along.y*0.35` was meant to stretch
it down the fall, but it never could. On a vertical face the radial lies in
the face's own plane, so it is already read at 5x through `tangent_u` and
`tangent_v`. The 0.35 term adds about 0.1% to the noise's rate down the face.
The approved blotches are round for that reason. B stretches that real look.

| Variant | What it is | Game change to port it |
| --- | --- | --- |
| A Today | dc0c5cb as it is | none |
| B Stretched | the same pattern, its height below the lip divided by `stretch` (default 2.0, range 1-4); a speed multiplier on the froth's time terms (default 1.0) | in the froth branch, read the noise at `p' = advected - radial*(h*(1 - 1/FALL_STRETCH))`, where `h` is the height above the lip's top (the mesher puts the lip height, or the metres under the lip it already writes in `flow_uv.z`, on each vertex). Multiply `time` by `FALL_FROTH_SPEED` in `advected`'s radial term and in the `+ time*0.6` term for the froth only |
| C Leaning | B on a sheet that leans out along the flow: offset `lean * profile(depth/drop)` with `profile(u) = (sqrt(u+0.05) - sqrt(0.05)) / (sqrt(1.05) - sqrt(0.05))`, default lean 0.8 m; nothing above the lip's ground moves, so the top and the sheet share their rim vertices (no gap) | `water_flow.rs`: move each falling face's vertex out along the fall's flow direction by that offset, the same for every face at one depth (the column's corners stay joined). Carry the unmoved position as one more vertex attribute and read the froth there, so the pattern is B's, bent |
| D Lip + speed-up | C, plus one amount (default 0.5): (1) the froth is read on one frame per fall (the face toward the flow) at the point where the flow leaves the falling cell, so the top and the face are one pattern across the edge; (2) the top whitens over its last 1.5 m (`smoothstep(-1.5, 0, s)*amount`); (3) the path below the lip is warped to time of flight, `c(s) = mix(s, (sqrt(1 + 2*3*s) - 1)/3, amount)` (1 m/s at the lip, 3 m/s^2), so the blotches lengthen as the water speeds up; on the top, `c = s - 0.6*amount*s^2` | mesher: per top vertex, the ray from the source cell's centre to where it leaves the falling cell (exit point and signed distance `s`), and per face vertex `s` = metres under the lip; shader: the froth's noise point `(exit.x, lip - c(s)/stretch, exit.z)` in body space, the ramp on tops |

Splash and mist (every variant, toggle): drops and a mist sized by the fall's
height H. There are about 26H drops and 7H mist puffs, thrown at
`sqrt(H)`-scaled speeds. They start on the open side of the column, past the
hex's corner radius from the leaned foot. They are never drawn below the
pool's surface + 5 cm ("only outside"). Rain rings are multiplied by
`smoothstep(0.7, 0.95, horizontal)` on the face's own normal, as in the game,
so no falling face takes any.

What the mockup does not port: the game's light and tone mapping (the
sand, sky light and white are matched to the screenshot by eye), the
screen-space reflection, the refraction offset, the distance fog and the
game's sky light per vertex (a depth ramp stands in). The noise is read at
a stand-in body position (a 2400 m planet at the capture's 24.1 N), so A
draws the game's kind of blotches, not the same ones in the same places.

## Smooth water between ticks, in a mockup first (owner, 2026-11-15)

The owner: *"for water is there a better way to lerp the sim? so it doesn look
like it just pops in?"*, then *"yea mock it up"*.

**Today.** The water rule ticks once a second (`liquid::TICK_S = 1.0`). Each
tick the CPU rebuilds the whole water mesh (`water_flow.rs`), and
`vertex_flowing` draws it as given. A cell that turns wet appears whole in one
frame. A level that moves steps 1/8 of a layer at once. A new fall stands at
full height at once, and a dried cell vanishes.

**Proposal.** The rule, its tick, and what is saved are unchanged; only the
drawing between two ticks moves. The mesher keeps the last tick's cells and
gives every vertex what it was as well as what it is. The vertex shader
eases from one to the other over the tick, with `t = clamp(since_tick /
TICK_S, 0, 1)` passed as a uniform and smoothstepped.

- **S1. Levels ease.** A top vertex carries its last-tick height. A pool
  rises and drains smoothly, and a slope tilts smoothly.
- **S2. New water flows in from its source.** A cell wet this tick and dry
  the last starts as a thin sheet on its source side, the neighbour it took
  its water from. Its leading edge sweeps across the cell along the flow
  vector over the tick, so the front visibly creeps. No fade in place.
- **S3. Falls pour down.** A new falling column's bottom moves from the lip
  to its foot over the tick. The splash at the foot starts when the column
  lands, not when the tick starts. *This changes how falls look, so the
  rule "Waterfalls are designed in a mockup first" applies: it is shown in
  M31 beside today's look, and ported only on approval.*
- **S4. Drying recedes.** A cell dry this tick and wet the last is drawn for
  one more tick, shrinking back toward its source, then dropped.

**Cost (estimated, not measured).**
- one more float per top vertex, the previous height;
- the dried cells of one tick kept for that tick;
- one uniform.

The mesher already runs once per tick. Saves, the rule and its tests are
untouched.

**The mockup.** M31 (`docs/mockups/waterfall.html`) gets a second 3D tab,
**Pour**. A live pour runs on `water-flow/liquid.js`, the rule the game was
ported from, at the game's 1 s tick. It is drawn with M31's port of the
game's `dc0c5cb` fall look, over a terraced slope with a drop into a pool.
A **Pops (today) / Smoothed** switch sits beside **Pour again** and a tick
speed. The first tab, the fall look, is unchanged. Only one 3D view draws
at a time.

**Approved (2026-11-15).** The owner, on M31 version 2: *"the waterfall look
is pretty good, need to be a bit careful with perf on that, on my laptop it is
a bit rough, so hopefulyy in game it will work well"*.

The mockup's cost does not carry over to the game. The page rebuilds its
geometry in JavaScript every tick and copies the framebuffer for refraction
every frame through three.js. The game already meshes on the CPU once a
tick and already draws this froth.

The port adds:
- one float per top vertex;
- one uniform;
- one tick's dried cells.

It is measured, not assumed. A `perf_suite.py` A/B on the release build
(main's water against the smoothed water), on a pour scene and the walk,
goes in `docs/benchmarks/` with the port. Any slower frame is fixed before
the push.

## Splashes that show, and better ones (owner, 2026-11-16)

The owner, on a river in rain: "these water splash fx are weak, I expected
them to be made really well like the crab breeze fx", "they also dont render
when they are in front of water either", and "I approved of ground splash a
while ago from rain drops, I dont see them here".

Measured (read off the code, 2026-11-16):

- **The fall-foot spray and mist are the only splash particles.** CPU
  billboards rebuilt each frame (`water_flow.rs` `spawn_splash`,
  `animate_splash`), up to 4000 soft white dots within 70 m, an unlit
  `StandardMaterial` with `AlphaMode::Blend`: Bevy's Transparent3d phase,
  no depth written. Nothing splashes where the player, a craft, a float or
  a fish meets water; the fishing splash is a sound only.
- **Why a splash vanishes over water.** All water is drawn by
  `WaterCompositeNode`, after `Node3d::EndMainPass`, so after Transparent3d
  (`planet_water.rs`). The flowing water reads the finished scene (spray
  included) as "what is behind the water", absorbs it by the path through
  the water and draws over it; it is discarded only where the main pass's
  depth is nearer (`water.wgsl`, `own_depth > clip.z`), and the spray wrote
  none. So a splash standing over a pool, which is where every fall's foot
  is, is erased; it shows only against land or sky. The near rain shower
  has the same flaw over water.
- **The white ribbons along a bank** are the fall froth on the side faces of
  flowing water: the mesher marks any flowing cell's side open to air as
  falling (`falling || above_water && flow.is_some()`), so a river's edges
  and level steps get the lip's streaked froth.
- **The rain splashes** (M32, `weather-rain` 6.3, approved 2026-11-15) are
  built on `claude/rain-splashes` (`weather-rain` 6.4, `planet_splash.rs`,
  captures 6.6) and now merged into `claude/env-v2`; 6.5, the perf A/B
  that gates the push to main, is open. A build from main before that merge
  has none, which is what the owner saw. As designed they stand on ground
  tops only, so a river or the sea gets none, and drawn before the water
  composite a droplet in front of water would meet the same overdraw as the
  spray.
- **Crab Breeze's effects** are not in this repository or `.reference/`:
  the repo cites Crab Breeze for its water lighting, cloud puffs, frame
  latency and audio tools, never for particles. Matching it needs its
  source or a recording.

The plan, in order:

1. **Splashes drawn after the water (a fix, not a look change).** The
   blended effects that stand over water (fall spray and mist, the near
   shower, and the rain splashes when they land) draw in their own pass
   after `WaterCompositeLabel`, depth-tested against the water's own depth,
   so a splash in front of water shows and one behind it does not. This
   restores what was approved; captures before and after, at a fall's foot.
2. **River banks without lip froth.** Froth only on faces that really fall
   (a step of a layer or more); a level river's side keeps the plain water
   sides. This touches the fall look, so it is shown in M31 beside today's
   first (the waterfall rule).
3. **The rain splashes on water too** (built for ground in `weather-rain`
   6.4): rings already mark rain on water; a crown of droplets over it is
   the M32 crown on the water's surface. The perf A/B (6.5) that gates the
   push to main runs on the owner's desktop.
4. **Better splashes, in a mockup first.** A splash kit for every place
   water is struck: fall foot, a player or craft entering water, a float's
   cast, a fish breaking the surface, a wake. Built in M31 (a Splash tab)
   with droplets that arc and fall under gravity, a crown, foam that
   spreads and fades on the surface, and mist, all lit by the scene; shown
   beside today's spray for the owner's approval before any game code.
   Crab Breeze's source or a clip of its splashes would be the reference.

## Risks / Trade-offs

- **Infinite water.** Renewing makes infinite water, as in Luanti. With
  three sources it needs a real pool, not a pair, to start.
- **Large floods are slow.** A flood moves at a cell a second, by design. A
  dam break across a valley takes minutes, which is the look Luanti players
  know.
- **The sea is a source everywhere.** Dig a channel from the sea and it
  fills to sea level, then flows on down any slope below it. Draining the
  sea is impossible, which matches the sea's rails.

## Migration Plan

- Mockup first (tasks section 1). The game waits for the owner's approval of
  the mockup, then a request (CLAUDE.md, the mockup tracker).
