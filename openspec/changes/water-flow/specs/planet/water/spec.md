# Water Specification

## ADDED Requirements

### Requirement: The water surface streams along a per-cell flow
The water cap SHALL advect its wave field along a per-cell flow vector on
horizontal faces and scroll it radially on vertical faces, and SHALL read that
vector from data rather than deriving one. A zero vector SHALL leave the wave
field exactly as the unadvected form.

#### Scenario: A world with no rivers
- **WHEN** every cell's flow vector is zero
- **THEN** the sea renders identically to a cap pass without the term

### Requirement: Flowing water pans its texture along its flow
The water top of a flowing cell SHALL pan its surface pattern along the cell's
flow vector, faster as the flow is faster, and a falling face SHALL pan it
downward; a still cell, a source the water does not leave, and the generated
sea SHALL not pan. The flow vector SHALL be baked per cell when the water is
meshed, as Tenebris C bakes it (`hex_mesher.c`, `bake_water_flow_uv`), so a
single phase serves and the pattern never stretches. The owner, 2026-11-15:
"Add UV scrolling and water texture or shaders to show the flow map. This is
how tenebris c does it." Proposed; the mockups (M25, M28) show it.

#### Scenario: A spring runs off a ledge
- **WHEN** a spring flows across a plain and over a ledge
- **THEN** the pattern on each flowing top moves away from the spring, the
  pattern on each falling face moves down, and a pool of sources at rest
  stands still

### Requirement: A flow-arrow overlay
The game SHALL offer an overlay, off by default, that draws an arrow on every
water cell whose flow vector is not zero, pointing along that vector and
longer the faster it flows, drawn just above the water, depth-tested so
nearer water and ground hide it, and toggled from the debug keys and the
player settings. It SHALL read the same flow vector the surface pans along.
The owner, 2026-11-15: "Add button to toggle arrows on off, the game should
have this too." Proposed; Tenebris C's F5 overlay
(`renderer_draw_water_flow_debug`) is the precedent, and the mockups' "Arrows:
on / off" toggle shows it.

#### Scenario: Checking a river
- **WHEN** the player turns the overlay on beside a river that falls over a
  ledge into a pool
- **THEN** every flowing top shows an arrow downstream, the pool's still
  water shows none, and turning it off removes every arrow

### Requirement: Flowing water is meshed per cell from its corner levels
The water mesher SHALL draw every water cell, not one surface per column, as
Luanti's liquid mesher does (`content_mapblock.cpp`, `drawLiquidNode`), on
the hex grid: each of a cell's six corner heights SHALL come from the three
cells sharing it (full where any has liquid above or is a source, else the
mean of the flowing levels, and at the floor where two of the three are
air); the top SHALL run through those corners and SHALL be left out where
liquid stands above; a side SHALL be drawn wherever the neighbour is neither
solid nor the same liquid, from the corner heights down to the cell's floor;
a bottom SHALL be drawn over air. A falling cell SHALL be a whole block with
its own sides, so a column falling over a cliff stacks block by block. The
flow vector SHALL be the corner slope's downhill direction, and the surface
pattern and the flow arrows SHALL read it. The owner, 2026-11-15: "you
missed a critical part of the luanti/minetest water implementation that
shows the water falling" and "water should generate blocks vertically as it
falls too". Proposed; the mockups (M25, M28) show it.

#### Scenario: A spring runs over a cliff
- **WHEN** a spring's water reaches the edge of a ledge six layers above a
  plain
- **THEN** its surface slopes down to the edge, a wall stands at every edge
  that faces air, and each falling cell under the lip is drawn as its own
  full block with sides, the column growing a block a tick to the plain


### Requirement: Rain rings never alias on the water
The rain's rings on the water SHALL fade with their footprint, so that where
a ring falls under a few pixels the water shows its waves and no per-pixel
speckle.

#### Scenario: A river in rain from the bank
- **WHEN** a river 10 to 60 m away is seen in full rain
- **THEN** the near water shows rings and the far water shows its waves, without white speckle

### Requirement: Water eases in and out between ticks
The game's water SHALL be drawn easing from what it was to what it is over
one tick (`liquid::TICK_S`) whenever it changes: by the rule's tick, by an
edit (water taken, placed or dug away) or by a fine set landing. A cell that
turns dry SHALL be drawn for one more tick, sinking toward the water under it
or toward its wettest side, and SHALL be gone at that tick's end. Every lane
a face is shaded by (its position, flow, normal and sky) SHALL ease, so no
part of the water steps at a tick. The ease is drawing only: the rule, its
tick and the save are unchanged. The owner, 2026-11-17: "The despawn is
still step based."

#### Scenario: A source stopped over a fall
- **WHEN** the source feeding a fall into a pit is taken away
- **THEN** the taken cell shrinks away over a tick, the fall drains from the
  top a cell a tick with its top sinking smoothly, and no frame shows a
  cell or a band of froth jump

#### Scenario: A dried cell mid-tick and at the tick's end
- **WHEN** a cell wet last tick is dry this tick
- **THEN** halfway through the tick it is drawn with its top between its
  last level and its floor, and at the tick's end it is not drawn
