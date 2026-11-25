# Gather by hand: design

Every decision below is a recommendation taken (ask only with screenshots).
None of them is the owner's own word. Screenshots of the result go to the
owner before the video gate.

## 1. What F takes

| piece (shader section) | gives | stack |
| --- | --- | --- |
| pebble (`SALT_ROCK`, `v` 234-287) | 1 `stone` (loose stone) | the good's |
| bush (`SALT_BUSH`, two leafy lumps) | 2 `fibre` | the good's |
| dead shrub (`SALT_SHRUB`) | 1 `fibre` | the good's |
| fern (`SALT_FERN`) | 1 `fibre` | the good's |

Mushrooms wait for food. Flowers and grass tufts are not taken.

Bare hands take every piece, so no tool is needed. The hands' pick-up time
is the wild plant's (`farm::pick`).

## 2. One placement rule

`pbd_core::scatter::pieces(cell, params) -> Vec<Piece>` returns, for one
cell, each pickable piece with:
- its kind and its index in the cell;
- its seat, in body-local metres;
- a pick radius.

It is computed from the same inputs the vertex shader reads:
- the cell's clutter id (`cell_key`'s old hash, which the draw rolls on);
- its corners and degree;
- the green and dry flags of its top;
- the `scatter.ron` chances and sizes.

`roll` and `hash` are the shader's, bit for bit (u32 arithmetic).

**Pinned to the shader.** A GPU test (`shader_tests`, lavapipe in a cloud
container) draws a handful of cells with the real `planet_surface.wgsl`
pipeline into a position target. It checks each pebble's and bush's centre
against `pieces`, within a centimetre. Two hand-written copies of an
expected value are never compared (CLAUDE.md).

## 3. Aiming

F's eye ray already finds crops and wild plants by their shapes
(`farm::plant_on_ray`). Clutter joins that search:
- each piece is a sphere of its pick radius, 0.25 m at the least;
- the search covers the cell the ray lands in and its neighbours;
- the reach is the hands' 3 m.

A crop or wild plant on the ray wins over clutter; otherwise the nearest hit
does. The hover prompt names the piece ("F  pick up stone",
"F  pull fibre"), as the wild plant's does.

## 4. What was taken

- **The record.** A `clutter` record per cell key holds:
  - a bitmask of the pieces taken;
  - the world time of each take.

  It goes through the save's durable path, authored by the player, in one
  line with the hands that got the goods (`store_with_hands`).
- **Regrowth.** A piece grows back `CLUTTER_REGROW_DAYS` (3) game days after
  it was taken. Stones too: the ground turns up more. Until then it stays
  taken in the save.
- **Old worlds.** They have no record, so nothing in them is taken.

## 5. The draw

`clutter::ClutterTaken` works as `trees_taken` does:
- the nearest 32 cells with a piece taken or growing back go into the
  planet uniform (`clutter_taken`, two a lane): each cell's clutter id and
  one byte a piece, how far it is sunk, 0 whole to 255 gone;
- the vertex shader scales the piece's height by one less that, so a taken
  piece sinks into the ground over `lod_fade_s` and rises again the same way
  when it grows back. That is how clutter already leaves at the edge of its
  reach (the `fade` term), so it never vanishes or appears in one frame. A
  piece sunk away collapses as one the cell never grew does;
- the list is rebuilt every frame while a piece is moving, and twice a second
  otherwise.

A taken piece beyond the 32 nearest cells is drawn again. A hand reaches 3 m,
so 32 cells is every take within a walk of the eye in a gathering session.
Measuring how often a player passes it is left to the owner's playtest.

*Built differently from the plan (recommendation taken, ask only with
screenshots):* the plan said "the detail-fade dither". Clutter has its own
fade, the sink, and a dither on a 0.16 m pebble is a few pixels of noise, so
the sink is used.

## 6. Missions

`How::Picked` counts `find` today. A pick-up of clutter is a new `How`,
`Gathered`, which counts `gather` and `find`, so "Gather by hand" counts it.
The mission's help line becomes:

> Pick up loose stones and pull fibre from shrubs with F, or dig soft
> ground with bare hands.

## 7. The bot

A `PickUp(Until)` act in `desktop/showcase.rs` walks to the nearest
pickable piece through `pieces`, aims and presses F.
`phase-1-trunk.ron`'s gather shot uses it for the fibre and the stone, and
digs only for the soil.

## 9. Fibre from felling (owner, 2026-11-21)

The owner: "quick fix, cutting down trees should get me fibers too". Each log
taken from a tree, by axe or by hand, also gives one plant fibre (its bark),
into the pack beside the log when there is room for it; a pack with room for
the log but not the fibre still takes the log. A toast says the fibre as it
says the log. Fibre is still gathered from the ground clutter (3) as before.

## 8. Risks

- **The port drifting from the shader.** This is the reason for the GPU
  test in 2; a change to the shader's rolls fails it.
- **Cost.** Every frame, the aim tests at most seven cells' pieces. The
  uniform grows by 64 x 16 bytes, and the draw's added test is a loop over
  64 ids for a clutter vertex. It is not measured in a cloud container: the
  owner runs `perf_suite` on the desktop.

## Taking a piece is 4x quicker (owner, 2026-11-22)

The owner, playtesting: "speed up gathering rocks and fiber animation by 4x".

The motion is the taken piece sinking into the ground (above). What sets its
pace today, measured in the code:
- the sink takes `scatter.lod_fade_s`, 1.0 s: the ground detail's own
  dissolve time (`detail-fade`), borrowed for it;
- the taken list is rebuilt twice a second while nothing is moving
  (`clutter::refresh`), so a sink can begin up to 0.5 s after F.
So a pebble is gone 1.0 to 1.5 s after F.

Change:
- a field of its own, `scatter.taken_sink_s`, 0.25 s, a quarter of today's,
  for the sink and for the rise when a piece grows back. `lod_fade_s` stays
  1.0 s for the ground detail it was made for.
- the list is rebuilt on the frame the save changes, as well as while a
  piece moves, so the sink starts on the frame of the take.
A pebble is then gone 0.25 s after F, 4 to 6 times sooner than now. The goods
reach the pack on the press, as before; nothing else in the rule changes.
