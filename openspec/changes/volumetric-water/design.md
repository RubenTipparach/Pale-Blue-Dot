# Design: three readers of the radius become readers of the column

## What the data already says

| Layer above the ground | Under land | Under the sea |
| --- | --- | --- |
| Above sea level | `Air` | (none: the sea's surface is sea level) |
| Below sea level, uncarved | (none: ground) | `Water` |
| Below sea level, carved by a worm | `Air`, and DRY is right | `Air`, and dry is WRONG |

The column is the authority for what is in a cell (the standing rule that
terrain and edits stay on the CPU and the GPU draws what it is sent). What
this change does is make the three readers that grew up on the heightfield
ask it.

## Measured

`water_below_sea_in_the_spawn_tier`, ignored, in `planet_column.rs`:

| Tier anchor | Columns | Ground at anchor | Water layers | Under LAND: dry air below sea (columns / layers) | Under the SEA: dry air below sea (columns / layers) |
| --- | ---: | ---: | ---: | ---: | ---: |
| The spawn | 3,105 | +74 m | 0 | 0 / 0 | 0 / 0 |
| Nearest shore to it | 3,894 | +1 m | 1,321 in 659 columns | **576 / 2,498** | **143 / 670** |
| Nearest shallows | 3,265 | -3 m | 608 in 320 columns | 154 / 624 | 42 / 170 |

The spawn stands seventy-four metres up and holds nothing below sea level,
which is why no capture from it has ever shown the owner's picture. At the
shore, one column in seven of the tier is a dry cave below sea level and is
tinted as sea today (the dry-cave case, right in the data and wrong on
screen), and one in twenty-seven is a dry tube under the ocean (wrong in the
data, phase 2's work). The instrument finds the shore and the shallows by
walking rings of the tangent plane out from the spawn, so it measures the
same ground on every run.

## Phase 1: water the renderer and the walker can see

**A render code for water.** `render_code(Material::Water)` is `0`, the same
as air, so `ColumnTier::gpu_materials` packs the two alike and `material_at`
cannot answer. Water takes a free four-bit code (`WATER_CODE`, 13; 0 to 12
are taken), the shader's tile table gives it no tile (it is never a face),
and `the_shader_draws_each_material_on_a_tile_named_for_it` learns that a
material with no face has no tile rather than failing on it. The faces a
water column draws are its seabed's, which is what draws today.

**The fragment asks the column.** In `planet_surface.wgsl`, a column-pass
face (`kind == 4`) or a cap in the tier (`lit_slot != 0`) is submerged only
if the layer on the AIR side of the face is water: for an up-facing face the
layer above, for a down-facing face the layer below, for a flank the layer
at the fragment's altitude. That is the same layer lookup the material term
already does, one call with the sign of `facing` flipped. Where it says
water, `water_depth` is what it is now (the radial depth under sea level,
which is the water's top over the sea); where it says air, the submerged
term is off. Off the tier the heightfield rule stands, because off the tier
there are no caves.

**The camera asks the column.** `submersion` gains a first question: the eye's
cell in the tier and the eye's layer in its column. Air is dry, whatever the
radius. Water is under, with the straddle band measured against the top of
that water run (sea level over the sea, so the sea's numbers do not move).
No column, and the radial rule stands. `PlanetContact` already resolves a
direction to its finest record and the record to its column, so this is the
lookup `sample_at` in the digging march makes, at the eye.

**The walker asks the column.** `Column::contact(altitude)` already answers
the floor and ceiling around an altitude; it answers the water too: the top
of the water run the altitude is in, or none. `PlanetContact::stand` sets
`water_depth` from that in the tier and from the heightfield outside it,
and the swim model needs no change because it reads `water_depth` and
nothing else.

**Tests.** A column carved under land below sea level reads air at the
carve and the contact reports no water there; the same carve under the sea
reads air too (phase 2 is what fills it) and the test says so; a submersion
at a dry cave eye is 0 and at a sea eye is unchanged; the shader material
term is exercised by the existing CPU audit with a water layer in the
neighbourhood.

## What phase 1 turned out to be, once written

Two things moved from the design as written.

**The fragment asks its COLUMN, not the air side of its face.** The design
said to read the material on the air side: exact for a cap, and impossible
for a flank, whose air is in the neighbour's column and whose slot the
fragment does not have. What replaced it is simpler and is the same answer
wherever the data is right: a fragment is submerged if the COLUMN it belongs
to holds water, and it is below that water's surface. A column under land
holds none, so its cave is dry at any depth; a column under the sea holds
water from its ground to sea level, so its seabed cap and the flanks below
read exactly as they did. The one case it answers differently from a
per-face rule is the dry tube under the ocean, which it calls wet - and that
column is the one phase 2 fills with water, so it is the answer that ages
correctly.

**The water surface is per column, not a flag.** It would have been one bit
today, because every water run in a generated column stops at sea level. It
is nine bits carrying the top of the run, because a pool is what phase 2
makes and a pool's surface is lower than the sea's. The shader caps it at
the sheet's radius, so a column whose water reaches sea level draws exactly
as before: the sea did not move.

## The picture, and what it measures

`--view seacave` moves the spawn 1,150 m to the shore and stands the camera
in a chamber whose floor is 7 m and whose roof is 1 m below sea level, under
land whose surface is above it: the owner's case. The same binary took both
frames, with only the shader's rule swapped, since the WGSL is loaded off
disk (`docs/screenshots/cave-below-sea-before-after.png`, both panels
brightened by the same 3.2 because the chamber is unlit and the difference
is otherwise real and invisible):

| | mean R | mean G | mean B |
| --- | ---: | ---: | ---: |
| Before, the radius rule | 2.0 | 49.3 | **92.9** |
| After, the column rule | 1.7 | 46.8 | **73.8** |

Blue moved by more than 8 of 255 across **99.04%** of the frame; red moved
across 0.00% and green across 0.11%. That is the shape of the fix: the sea's
own absorption and deep colour come off the whole cave and nothing else
changes. What is left is the dark blue of an unlit chamber's ambient, which
is what a cave at night looks like in this engine.

**And the sea did not move**, proved by accident. The first cut of the pick
asked only for a chamber below sea level and took a nook in the SEABED, open
to the water. Before and after of that frame are BYTE IDENTICAL: where the
column holds water, the column's answer is the sea's answer.

## Phase 2: flooding by connectivity

At `column::build`, after the carve and the edits, a bounded flood: from
every `Water` layer, into every face-adjacent `Air` layer at or below the
source's altitude, within the tier, until nothing changes. Face-adjacent
means the six (or five) neighbours at the same layer and the layer below;
never upward, because water does not climb. The tier's rim is solid, so the
flood cannot leave the tier, and the tier is rebuilt as the player moves, so
a tunnel that reaches the sea floods when the tier reaches the sea.

An edit runs the same flood from the edited cell's neighbours, bounded to
the tier, on the edit, and repacks what changed. Digging into the sea from a
dry tunnel is the case: the water comes in.

What this is not: aquifers. Minecraft fills some sealed caves at generated
levels; ours are dry until the generator gains a rule for it, and that is a
decision for the terrain generator's write-up rather than this one.

## Phase 3: flow, in Minecraft's terms

Recorded so phase 1's data shape does not fight it. A water layer carries a
level (source, or flowing 1 to 8) and the sheet draws the surface of a cell
at its level; a source spreads to seven cells on the flat, downward without
limit, choosing the shortest route to a drop within four cells; two sources
over solid make a source. That is a fluid state with a scheduler, which
`voxel-engine-foundation` reserves as CPU-authoritative with low-frequency
active cells, and the `water-flow` change's per-cell vector is what it
feeds. Nothing in phase 1 stores a level, and a `Water` layer is a source at
sea level until phase 3 says otherwise.

## What is not settled

- The straddle band inside a cave pool (phase 2 makes pools): the composite
  measures it against the run's top, which is the sea's number over the sea
  and a pool's over a pool; the swell amplitude in the band is wrong for a
  still pool and is left until a pool exists to look at.
- Whether the sheet should draw a cap over a flooded tunnel's water (its
  cell is not a water cell by the heightfield). Phase 2's question.
