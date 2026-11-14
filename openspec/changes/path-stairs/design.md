# Design: path-stairs

## The finding

### What a path is, and where it takes a step

A town's template paints each cell of its grid with a top and an area
(`assets/settlements/v1/*.json`). The ground a laid town stands on is its
stored footprint, each cell at its stored level (`record::Town::terrace_of`,
whole layers over the terrace), so a change of level between two cells is a
riser of whole layers of terrain.

A **path cell** here is a footprint cell the walker is meant to walk along:

- a paved top, `cobble`, `flag` or `dirt` (lanes, streets, the square, the
  quay, the mole, the plaza, the hamlet's lanes), on a cell that is no
  building's and not a field (an area named "... field", the hamlet's and
  the village's dirt furrows);
- a gate's passage: a masonry cell whose vault stands over its ground
  (`MasonryCell::from` over the cell's height). Every other masonry cell is
  wall, not path.

A **riser** is an edge between two path cells where one stands higher than
the other by more than the walker holds and no more than one layer and a
plank: from `walking::HOLD_M` (0.35 m) to 1.25 m. Lower than that the walker
already walks it without a jump (town and deck risers are 0.19 to 0.25 m,
the pier's beach boardwalk 0.12 m). Town levels are whole layers, so every
town riser in the data is exactly 1 m; a pier's raised stretch is 1.0 to
1.12 m over its neighbours. Nothing between two path cells is two layers or
more in any template.

### Measured: every riser between path cells, by template

Read off the templates with the stored levels' rule (`levels_of`: a built
cell at its template height over the datum), and checked against every town
of the shipped seed by `towns::tests::every_towns_path_risers_have_their_stairs`:

| Template | Lower path cells | Riser edges | Where |
| --- | ---: | ---: | --- |
| Village (Holbrook and 19 more) | 0 | 0 | laid flat on one terrace (slice 1) |
| Walled town | 12 | 18 | the mockup's 11 street-steps cells, and the north gate's passage onto the wall lane |
| Harbour | 6 | 12 | the up street's 4 street-steps cells, and 2 cells where it meets a lane one terrace up at its side |
| Desert | 0 | 0 | the plaza is level; the oasis is a pool's hollow a layer down, not a path |
| Tundra camp | 0 | 0 | its built ground is snow, with no paved path |
| Jungle village | 0 | 0 | laid flat (not terraced) |

The walled town's lower cells, as `(c, r)`, the directions of their higher
neighbours, and what the mockup made of them:

| Cell | Area | Up | Mockup |
| --- | --- | --- | --- |
| (19, 7), (20, 7) | Wall lane, steps | 1 2 3; 0 1 2 | street steps: the riser just inside the north gate (G1) |
| (19, 16), (20, 16), (19, 23), (20, 23) | Gate street, steps | 1; 1 2; 1 2; 2 | street steps |
| (4, 16), (4, 23), (35, 16), (35, 23) | Wall lane, steps | 1; 2; 1; 2 | street steps |
| (41, 6) | The hamlet, steps | 1 | street steps |
| (19, 6) | Town wall (the north gate) | 2 | none: the passage's corner onto the wall lane |

The harbour's:

| Cell | Up | Mockup |
| --- | --- | --- |
| (22, 21), (22, 27) | 1 2 3 | street steps |
| (23, 21), (23, 27) | 0 2 | street steps |
| (22, 20), (22, 26) | 2 | none: the up street's side onto the middle and top lanes |

And Marenstrand's main pier (`pier-landfall`): one stretch, 0.75 m long,
lies on a bank of the planet's ground a metre over the boardwalk either
side. Every other pier of the six harbours changes height by at most 0.12 m.

So the owner's shot (white plaster houses, a log house, a lane climbing in
one-metre steps between grass-edged terraces) is a harbour's up street: the
whitewash and driftwood kits, its terraces a layer apart, and bare ground
where the mockup cut street steps. Holbrook's lane, the village the shot
resembles, has no riser: the village stands on one terrace. Its shots are
kept as the check that nothing changed there.

### Why the riser is bare

The game ports the mockup's straight flight, newel stair and the desert's
outside stair as pieces (`Surface::Flight`, `Surface::Newel`,
`desert::outside_stair`), and the harbour's porch stair. The street steps
were deferred in slice 4b, with the reasoning that the walker's 1.05 m step
allowance climbs a layer. That is a hop: the eye rises a metre in one tick,
and with auto step off (`player-settings` 5) the walker cannot climb it at
all.

## Decision 1: two forms of stair, both from the mockup

**Street steps** (the mockup's `streetSteps`): stone steps across the whole
lower cell, climbing toward its higher neighbours, in 6 risers. They stand
where the mockup cut them: a cell whose area ends ", steps", or a lower cell
with two neighbouring edges up (a step cell by shape). The coast's four
step cells are named ", steps" in `coast.json` and the mockup's `makeCoast`,
as `makeTown` names the walled town's; the name is read by nothing stored
(the harbour is not a wild template, so its areas lay nothing).

- The climb is toward the mean of the up edges' outward directions: an
  edge's own direction for one, the corner between two neighbouring ones,
  the middle one of three.
- The pitch line reaches the full rise at five sixths of the way from the
  cell's foot to the nearest up edge's middle, so each up edge's middle is
  at the full rise. With one up edge that is the mockup exactly: the
  hexagon's 2.83 m flat to flat in 6 treads of 0.472 m, the last the
  landing. With up edges either side of the climb, the treads are shorter
  (25 to 31 degrees), and the far corners of the side edges are at most
  0.33 m under the street beside them, inside the walker's 0.35 m hold.
- Drawn as 6 bands of the hexagon, each a prism from the cell's ground to
  its tread, cut as the mockup's `clipPoly`. A band's side on an edge whose
  neighbour's ground is over its tread is not drawn: it would lie in that
  column's own side face.
- The walker stands on a `Surface::Steps`, the pitch line over the hexagon
  (the mockup's `stairTop` in its proposed mode, as every game stair is).

**A flight against the riser**, for a riser in a cell the mockup did not
step (the gate's passage, the up street's side): a straight flight in the
lower cell, 1.3 m wide (the mockup's `cliffStair`), its head on the middle
of the shared edge, climbing square to it in risers of about 0.19 m on
0.3 m treads (the porch stair's and the desert's): 5 risers and 1.5 m for a
layer. Solid steps of stone, drawn as the desert's outside stair draws its
own. A cell with two such risers on neighbouring edges would have its
flights cross, so it takes street steps instead (none in the data).

**A pier's stair**: where two of a pier's stretches (`harbour::stretches`)
differ by a riser, a straight flight of timber steps on the lower one, the
pier's width less 0.2 m, its head at the joint, climbing along the pier. At
Marenstrand's bank that is a stair up at one end and down at the other, as
W20 recommended. A flight longer than the lower run of planks is steepened
to fit it (0.22 m treads at the least).

Alternatives weighed:

- **Cut the riser into the terrain** (lower the upper cell's edge, or
  raise the lower cell in half-layers). The terrain holds whole layers, and
  the levels are stored: a new layout version, for towns laid from now on,
  while every town in a save keeps its bare riser. Rejected; the pieces
  reach every town.
- **Flights against every riser, street steps nowhere.** One form, but the
  mockup's step cells climb to two or three higher cells, and flights from
  neighbouring edges of one cell cross. Rejected.
- **Street steps in every lower cell.** The up street's side cell and the
  gate's passage carry level traffic past the riser; steps across the whole
  cell would hump it. Rejected.

## Decision 2: saves

Nothing stored changes, and no generator or layout version is needed:

- A stair is a piece of the town, cut from the town's stored chart and
  levels and from the template it was laid from, every time the town is cut
  (`record::build_town`), as the masonry, piers, outside stairs and dressing
  are (`cities-in-the-world` 4c, "What is stored"). It is never written.
- The town record, its cells, levels, tops, terrace, buildings and layout
  version are as they were. The ground (`TownGround`) and every column's
  height are the same numbers: the stair stands on the ground, it does not
  move it. So every player edit is at the same layer as before.
- The only file change outside code is the area name of four coast cells,
  which no laid town stores or reads.
- So a world saved before this change opens with the same towns, and each
  gains its stairs when it next stands. A test builds a stored town with
  every stair yielded (the build before this change) and without, and finds
  the same pieces, ground and records with the stairs added.

## Decision 3: edits

A stair never stands in a player's work. When a town is cut, a riser is
skipped if its lower or its upper cell holds an edit (`Edits::for_cell`),
and a pier's stair if any cell under it does. The app hands the cut the set
of the town's cells that hold an edit (`towns::build_yielding`). A town whose
ground held an edit before it was laid is unsettled and has no stairs at all
(task 4.4, unchanged).

What this does not do: a town that is standing is not re-cut when the player
then digs or builds in a stair's cell. No town piece is (a house dug under
stands until its town is cut again); the stair takes the same path, and goes
the next time the town comes into range. Taking a piece out of a standing
town is its own change.

## Not this change

- Steps on natural terrain outside towns: the owner's "path" is a town
  path, and the terrain's 1 m layers are the terrain's.
- The oasis's rim, the beach's rows, the mole onto the headland, a lane
  onto a yard: not path to path.
- A riser of two layers between path cells: none in any template. One would
  be reported by the test, not stepped.
