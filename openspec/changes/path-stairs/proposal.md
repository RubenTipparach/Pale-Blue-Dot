# Proposal: where a town's path takes a step, there is a stair (path-stairs)

## Why

The owner, 2026-11-14, with a screenshot of a town at night (white plaster
houses, a log house, street lamps, a lane climbing between grass-edged
terraces in bare one-metre steps of earth): "where theres a step on a path
there should be staris".

Today a town's lanes climb its terraces in whole layers of bare ground. The
walker gets up a one-metre riser only by auto step (a 1.05 m allowance) or a
jump, and the eye jumps a metre as it does. The towns mockup never had
those risers: wherever its streets cross a terrace it cuts **street steps**,
one cell of stone steps in 6 risers of 0.167 m (`tenebris-towns` section 3,
"Street steps"). The game left them out (`cities-in-the-world` 4b: "No step
pieces yet ... a 1 m change of level is walked as the terrain's own step";
4d: "Street steps are 4b's open item").

The rule also answers two open survey questions:

- **G1** (`town-gates` task 3.1): the 1 m earth riser just inside every
  walled town's north gate. Recommendation was "a stone stair of 2 to 4
  steps".
- **W20** (`pier-landfall` task 1.6): Marenstrand's main pier crosses a sand
  bank, and 0.75 m of its planks lie a metre over the rest. Recommendation
  was "a short stair up and down at each end".

Both are answered by the owner's rule of 2026-11-14 and are built here.

## What changes

- **Street steps** in every cell where the mockup cut them: the lower cell of
  a street crossing a terrace, stone steps across the whole hexagon, 6 risers,
  climbing toward the higher street. The coast's four step cells are named as
  the walled town's already are (", steps") in its template and the mockup.
- **A flight against the riser** wherever else one path cell meets the next
  one layer up (the up street's side onto the lane above, the north gate's
  passage onto the wall lane): a straight stone flight in the lower cell,
  its head on the shared edge.
- **A timber stair at each end of a raised stretch of pier** (Marenstrand's
  bank, and any pier whose ground lifts a stretch a metre).
- Every stair is a **piece derived from the stored town and its template**
  when the town is cut, as the masonry and the piers are. It is never
  saved. No record, no ground, no generator and no layout version change, so
  a town already in a save gets its stairs the next time it stands.
- A stair is **not placed** where the player has dug or built: a riser whose
  cells hold an edit keeps its bare riser.

## Not in this change

- Steps in natural terrain outside towns. The owner's "path" is a town's.
- Steps into the desert's oasis (a pool's hollow, as the village's pond),
  down the harbour's beach, or between a lane and a yard, wall or field: none
  of those are path to path.
- A stair taken away while its town stands, when the player then digs or
  builds on it. Every town piece is cut when its town comes into range and
  meets edits only then; the stairs do the same (design, "Edits").
