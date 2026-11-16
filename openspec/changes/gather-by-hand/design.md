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

`clutter_taken` works as `trees_taken` does:
- the nearest 64 cells with a live take go into the planet uniform, each
  with its id and its piece mask;
- the vertex shader drops a taken piece;
- a take less than `lod_fade_s` old dissolves out through the detail-fade
  dither rather than vanishing in one frame;
- a regrown piece dissolves in the same way.

A taken piece beyond the 64 nearest is drawn again. A test keeps that out of
sight: the list covers the clutter's 120 m reach at the take density a
player can reach in a sitting.

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

## 8. Risks

- **The port drifting from the shader.** This is the reason for the GPU
  test in 2; a change to the shader's rolls fails it.
- **Cost.** Every frame, the aim tests at most seven cells' pieces. The
  uniform grows by 64 x 16 bytes, and the draw's added test is a loop over
  64 ids for a clutter vertex. It is not measured in a cloud container: the
  owner runs `perf_suite` on the desktop.
