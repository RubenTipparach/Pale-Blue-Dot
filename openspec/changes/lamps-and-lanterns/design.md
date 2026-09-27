# Design: lamps and lanterns

## Context

See `proposal.md` for why. What the design has to work with, observed on
`main` (2026-09-27):

- **The light field.** `pbd_core::light` bakes one byte per cell: four bits of
  sky light and four of block light. Both are flooded one level per cell and
  stopped by solid cells. `ColumnTier::relight` re-bakes the whole tier of
  3,105 columns in about 6 ms. It runs after every edit, and it derives its
  emitter list from the columns (`emitters()`); nothing keeps a second list.
- **Emission belongs to the material.** `Material::emission()` gives
  `Torch = 14` and 0 for everything else. The torch is neither solid nor
  opaque. The shader draws its geometry from a torch band in the column
  record.
- **The shader** adds the block term with a warm tint and a gain
  (`TORCH_TINT`, `TORCH_GAIN`) that exist only in `planet_surface.wgsl`. It
  does not scale that term by daylight, so a torch is as bright at noon as at
  midnight.
- **Things that move** (the player's body, the held tool, the ship) are drawn
  with their own materials. They take the sun and the sky, and nothing from
  the field. `ColumnTier::light_at(slot, layer)` answers one cell and has one
  caller.
- **The sun** comes from `daylight::Clock`. One call site,
  `desktop.rs:1491`, still frames the "midnight" and "nightshore" captures
  from the fixed noon sun (`sky::SUN_DIRECTION`).

## Goals / Non-Goals

**Goals:**
- A city's full set of lights, each placeable and saved like the torch.
- Moving things lit by the same field as the ground, as the owner asked when
  lighting was first added.
- Lights that come on at dusk, so a city is lit at night without the player
  placing every lamp.
- The lighting that is built recorded in `openspec/specs/planet/light`, and
  the two half-open changes closed.

**Non-Goals:**
- **Coloured light.** The field has one block channel, tinted warm in the
  shader. Coloured light needs three channels and a field three times the
  size; a city of warm flames does not need it.
- **An incremental relight.** A full bake is 6 ms. Two extra bakes a day, at
  dusk and at dawn, do not change that.
- **Per-corner light on the ground clutter.**
- **Lit windows as a surface glow.** A window glow belongs to the city's
  buildings in `cities-in-the-world`. This change gives it the candle that
  lights the room behind it.
- **A moon, stars, fuel or fire spread**, as `night-and-lamps` held them.

## Decisions

**1. Dusk-lit emitters re-bake the field at dusk and at dawn, rather than
having a third light channel or a gate in the shader.**
- The shader cannot tell which part of a cell's block light came from a
  dusk-lit lamp and which from a torch, so it cannot gate one without the
  other.
- A third nibble per cell would add half as much again to the field's upload
  for one yes-or-no.
- A re-bake is exact and costs two 6 ms bakes a day. `daylight` answers
  `is_dusk_lit(clock)` from the sun's elevation, with some hysteresis so a
  clock held at dusk does not flicker. The tier re-bakes when that answer
  changes.
- *Alternative:* two block channels, one for each class. Rejected for the same
  field cost.

**2. The sampler blends the cells around a point, and a moving thing samples
the eight corners of its bounds.**
- `light_at(point)` blends the field across the hex cells around the point and
  the two layers it sits between, returning both channels.
- Each moving thing samples the eight corners of its bounding box once a frame
  and hands them to its material as a uniform. The shader blends the eight
  across the model, so a ship half out of a cave mouth is lit at its bow and
  dark at its stern.
- Beyond the tier, the sampler returns full sky and no block light, which is
  what the far terrain is drawn with.
- *Alternative:* one sample at the centre. Rejected, because the ship is 15 m
  long and would light as one flat value.
- *Alternative:* a per-vertex proximity sum over nearby emitters, the
  reference's approach. Rejected because it leaks through walls, which is the
  flaw `night-and-lamps` was written to avoid.

**3. Every light is a material, and a wall lantern's facing is derived, not
stored.**
- New materials: `LanternPost`, `LanternWall`, `LanternHanging`, `Brazier`,
  `Candle`.
- A wall lantern hangs on the side of its cell with a solid neighbour, found
  when the geometry is built, the same way the torch band is. This spends no
  bits in the cell record and follows the wall if the wall moves.
- A hanging lantern hangs from the solid cell above it.
- Emission levels on the field's 0-15 scale: torch 14 (as now), post and wall
  lanterns 13, hanging lantern 12, brazier 15, candle 8. The candle's 8 lights
  a room of about two cells across, and no further. The levels are tuned on
  captures, and the owner sees them.

**4. Which lights are dusk-lit is a property of the material,** not of where a
light stands:
- dusk-lit: the post and wall lanterns, and the glowing flowers;
- always lit: the torch, the hanging lantern, the brazier and the candle.

A city built in `cities-in-the-world` places its street lanterns as dusk-lit
materials. What the player places behaves the same way, so a street the player
builds works like a city's.

**5. The torch's tint and gain move into the core.** `light::TORCH_TINT` and
`light::TORCH_GAIN` sit beside the constants the WGSL test already holds, and
that test gains two lines. This is the fix the verification found, not a new
look.

**6. The built requirements are synced before anything new is built.**
- `voxel-light` has three built requirements, and `night-and-lamps` has two
  (the sun, and the block channel). Their deltas are written against
  `specs/planet/spec.md`, a path the main specs do not use, so they move to
  `planet/light` and are synced with `/opsx:sync`.
- `night-and-lamps`' requirements that are not built (the torch's icon, the
  flowers, the sampler) are carried here, in this change's delta, and removed
  from its delta before it is archived. A requirement never reaches the main
  spec ahead of its code.

## Risks / Trade-offs

- [A level-13 lantern floods about 13 cells, over 30 m across 2.833 m cells,
  and could light a whole street from one post] → The levels are tuned on
  captures of a street and a room, and the owner sees them. If one step per
  cell is too long horizontally, the flood's step becomes two levels per
  horizontal cell, which is a single constant in `light`.
- [Hundreds of emitters in a city make the bake slower] → The flood's cost
  follows the lit volume, not the number of emitters. A synthetic city of 300
  lanterns is baked and timed in the tests before any city exists.
- [Sampling every moving thing each frame] → Eight cell reads and a blend per
  thing, for fewer than ten things. It cannot be measured in a cloud session
  (see CLAUDE.md), so the owner runs the performance suite on real hardware.
- [The dusk re-bake happens on the frame the clock crosses dusk, and a 6 ms
  hitch at dusk may be visible] → The re-bake runs on the same worker the edit
  path uses, and the new field is swapped in when it is ready.

## Migration Plan

- New materials take new ids after `Torch = 12`, so an old save's edit log
  reads unchanged.
- The kit gains grant version N+1, which deals a few of each new light once to
  an existing save, as the torch grant did.
- Rollback is the previous build. A save that holds the new materials cannot
  be opened by an older build, which is the same as for any material added
  before.
