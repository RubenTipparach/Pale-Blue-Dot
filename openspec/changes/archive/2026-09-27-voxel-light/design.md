# Design: a field on the CPU, a corner in the shader

## The two halves, and which project each comes from

The owner named both: "voxel lighting algorithm combined with baked vertex
colors". They are different jobs and they live in different places in the
reference, so they do here too.

| | Tenebris | here |
| --- | --- | --- |
| The FIELD: what reached this cell | `tenebris-core/src/world_light.rs` | `pbd_core::light::bake` |
| The CORNER: what one vertex is lit to | `tenebris-client/src/hex_mesher.rs` | `planet_surface.wgsl` |

**The field**, read rather than guessed at. `sky_seed_one_column` walks each
column down from the top, breaks at the first covering block, and sets every
cell above it to `voxel_sky_max` (15). `bfs_one_pop` is a FIFO flood: a cell
hands `level - cost` to every neighbour that is not solid and not already at
least that bright, with `voxel_sky_lateral_loss` shipped at 1 and the vertical
step at 1 as well - its own comment says "Vertical neighbours stay at 1 per
step so daylight shafts remain at full strength". Solid cells are never written
and never propagate. Range is therefore 15 cells from the lit frontier, in any
direction.

**The corner**, from `smooth_corner_sample_n`. For a corner shared by the
face's own tile and the two that flank it, at the AIR layer the face opens
onto: average the sky level over the cells there that are not solid, then
multiply by a three-step ladder on how many of the two SIDE neighbours are
solid - `1.0`, `0.85`, `0.70`. The reference's own comment says what that is
for, and it is exactly what the owner asked about: "how much corner is wedged
into adjacent stone, which is the visual the user circled at the base of a
column meeting flat ground - the cap corner touching the column should darken,
the cap centre should stay bright, and the GPU interpolates linearly between
them."

The face's own cell is left out of the ladder on purpose: for a cap it is
always air and for a flank always rock, so neither says anything about how
exposed the corner is.

## Why the corner is in the shader and the field is not

**There is no CPU mesh here.** Tenebris meshes on the CPU and writes
`sky_light` into a vertex attribute, which is what "baked vertex colours"
means there. Every vertex in this renderer is generated in
`planet_surface.wgsl` from the cell records and the run words; nothing is
expanded to a buffer. So the corner sample happens where the vertex is made.

That is a second representation of one rule, which this repository's rules
allow only with a check on the real artifact:
`the_shader_carries_the_reference_light_constants` reads the shipped WGSL and
holds `LIGHT_MAX`, `CONTACT_1`, `CONTACT_2`, `LIGHT_LAYERS` and `LIGHT_WORDS`
to the core's own values. It proves the CONSTANTS agree and says so; the
arithmetic is checked by the captures.

The field stays on the CPU because it is a fact about the world rather than
about a frame: the tier is already built on a background task, the columns are
already there, and a BFS is exact where the Jacobi relaxation in
`voxel_light.wgsl` is an approximation wrong by however many iterations it was
short. That shader stays unbound. What would make it worth binding is light
that changes every frame - a moving lamp - and nothing here has one.

## The buffer

One byte per (slot, layer): `COLUMN_CAPACITY` x 320, four to a `u32`, 5.2 MiB
at full capacity and **0.95 MiB** for the 3,105 columns the default tier
holds. It is binding 5 of the draw layout, beside the column records it
lights, and it is written in the same `write_buffer` pass as those records -
two uploads a frame apart would draw one frame of new geometry lit by the old
world.

Zero-initialised, which is the safe way round: a slot nothing has written is
DARK. A light buffer defaulting to full daylight would light every cave in the
tier on the frame before its bake arrived.

## Measured

| | |
| --- | ---: |
| Bake, 3,105 columns (993,600 cells) | **6.0 ms** |
| Light for that tier | 0.95 MiB |
| Air cells the sky never reached | 2,815 |
| Cave interior, frame mean of 255 | 74.2 -> **26.4** |
| Open meadow, frame mean of 255 | 92.5 -> 92.3 |

The meadow is the control and it is the number that matters most: open ground
sees full sky, the field answers one, and the picture is what it was to within
0.2%. What changed is everything that is not open.

**Six milliseconds decides the invalidation strategy.** The reference runs a
bounded incremental relight per edit - a two-pass removal and addition over
roughly `sky_max^3` cells - because a full pass on its 160k-tile body is
"~1000 ms / 17M ops". Ours is one tier of 3,105 columns and a whole rebake is
six. So a dig relights the tier, and the removal pass, which is where the
reference records its own scar (a dug cell that "stayed dark forever"), does
not have to exist.

## Three defects found while building it, all invisible before

1. **The ambient floor was missing, and this is what made it show.**
   `hex.fs` has `max(0.05, ...)` inside its ambient term;
   `docs/tenebris-comparison.md` has recorded its absence here since the port
   ("**no floor**"). Nothing needed it while every cell was four fifths lit.
   With a real field behind the sky term the cave read **11 of 255**, which is
   not a dark room, it is a black screen with a hotbar on it.
2. **Three colour branches, and only the first had the floor.** The fragment
   computed `color` and then two later `if`s rebuilt it for walls and for cave
   faces. A cave is made almost entirely of the third kind, so the term that
   stops a cave being a black screen was missing from exactly the faces that
   needed it. They are one expression with the fill chosen now.
3. **A one-metre terrace riser came out pitch black**, a hard dark line along
   every step in the world. The cause is the same one twice: the layer at a
   face's own altitude is the last SOLID one, not the air it opens onto. A
   heightfield cap sits at 76.3 m, which is inside the metre the ground fills.
   `air_above` steps a cap up to the first layer that is really air, and
   `corner_light` steps up one layer where every cell it sampled was rock -
   which is the reference's own floor-contact fallback arrived at from the
   other side ("a floor-level corner always has SOME adjacent air to derive
   light from, never a hard 0").

## The stand-in this replaces is deleted

`column.ron` carried `cave_dark` and `cave_dark_depth_m`, and the shader faded
a cave face toward that floor over that depth. Its own comment named it: "A
STAND-IN for the baked voxel light this change defers." It is gone, the two
uniform lanes read zero, and the config fields with them. Depth is not
darkness - a cave mouth is deep and bright - and keeping both would be two
rules for one fact with the wrong one winning at every mouth.

## Held, with the reasons

- **Torches, and the block channel with them.** The reference's byte is
  `(sky << 4) | block` and in the running game **the block nibble is always
  zero**: `hex_mesher.rs` does not call `set_torches`, because a torch
  placement measured "~1000 ms / 17M ops" against "0.4 ms" for an ordinary
  edit. Its torches are a separate per-vertex inverse-square sum with no
  occlusion test at all (`LIGHT_RANGE_M = 6.0`, `LIGHT_DECAY = 2.0`). So this
  ships the sky channel alone: a second nibble nobody fills would be a field
  that looks implemented. When a lamp exists here it follows the model that
  RUNS there, not the one that merely compiles.
- **An incremental relight**, if a tier ever grows past the size where six
  milliseconds is affordable.
- **A light-level curve.** The reference is a plain linear `value / 15`;
  Minecraft's `0.8^(15-level)` is absent there and here.
- **Light on the clutter.** Grass blades, pebbles and trees take the cell's
  value, not a corner's. The reference overloads `sky_light` on a blade as a
  root-to-tip gradient, which this shader already does with `shade`.
