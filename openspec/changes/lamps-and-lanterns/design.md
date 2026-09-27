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
- **A moon, stars, seasons, fuel or fire spread**, as `night-and-lamps` held
  them. That change is archived with this one's group 1 (survey L3).

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

**7. A lamp's level is a metre: a step across a cell costs three levels, a
step up or down one (found on the task 5.2 captures, 2026-09-27).**

The risk below came true on the first capture of the six lights. At midnight a
row of them, spaced one cell apart 5 m ahead, lit the meadow around them nearly
as bright as noon, out past 30 m. With the lights taken away, the same view is
the dark night it should be (`docs/screenshots/lamps-and-lanterns/`).

- **Why.** The flood spends one level per step in every direction, and a step
  across is a 2.833 m cell while a step up is a 1 m layer. A level-15 brazier
  lit 14 cells, about 40 m, sideways and 14 m upward. The brightness is also
  linear in the level, so 14 m from a brazier was still at two thirds.
- **What the owner approved.** The towns mockup gives each light a reach in
  metres, with a smooth fall to nothing: lamp 9, torch 10, brazier 11, candle
  5 (`LIGHT_REACH`), at a strength of `(1 - (d/R)^2)^2`.
- **The fix, part one: the block channel's sideways step costs three levels.**
  Three levels is one cell's 2.833 m, rounded, so a level is about a metre in
  every direction. The reaches become:

  | light | level | cells across it lights | metres |
  | --- | ---: | ---: | ---: |
  | brazier | 15 | 4 | 11.3 |
  | torch | 14 | 4 | 11.3 |
  | post or wall lantern | 13 | 4 | 11.3 |
  | hanging lantern | 12 | 3 | 8.5 |
  | candle | 8 | 2 | 5.7 |

  Those are the mockup's reaches to within a metre or two, from the levels
  decision 3 already chose. Upward a light still climbs one level per layer,
  so a lantern lights a ceiling 3 m over it at 10 of 13.
- **The fix, part two: brightness follows the mockup's curve.** With `f` the
  level over 15, the shader adds `(f (2 - f))^2` of the lamp colour rather
  than `f`. It is the mockup's `(1 - (d/R)^2)^2` written in the level: near a
  light it is brighter than linear (0.92 at one cell from a brazier, against
  0.8), and at the edge it falls smoothly to nothing (0.13 at four cells,
  against 0.2).
- **The sky channel keeps one level per step (until decision 11).** Daylight into a cave is
  `voxel-light`'s, and the owner has not yet judged its caves (task 1.4). The
  same rule for the sky would take a tunnel's twilight from about 42 m to
  15 m. That is a change to the look of every cave mouth, so it is a survey
  question (L1), not part of this fix. The flood stays one implementation, and
  its sideways cost is a parameter of the channel.
- **Cost.** A block flood touches about a ninth of the cells it did, because
  each light's lit disc is a third as wide. The bake timing of task 5.5 is
  measured after this change.
- *Alternative:* keep the flood and only steepen the shader's curve. Rejected:
  no curve on the level can make a light reach 11 m sideways and also light a
  ceiling 3 m up, because the level does not know which way it travelled.
- *Alternative:* lower every level. Rejected for the same reason. A candle at
  level 2 would light across one cell and up two layers.

**8. A glowing flower is chosen on the CPU from the cell's exact key, and
its bit makes the flower as well as the glow (written 2026-09-27, before task
6.1).**
- **Which cells.** `pbd_core::flora::glows(key, chance)` hashes the cell's
  exact key (`cell_key`) and compares it with `glow_flower_chance` in
  `scatter.ron`, a share of green cells: 0.04, a third of `flower_chance`'s
  0.12. One glowing flower in about 25 green cells is one every 14 m or so
  across a meadow, close enough that their pools touch.
- **Where it is carried.** The fine set sets bit 0 of the record's
  `spare[2]` when it builds the record, so the decision is made once, where
  the key is made. The column tier keeps the same answer per slot and adds
  the cell as a dusk-lit emitter at the layer over its ground, level 6, while
  that ground is grassy and the tier is at dusk. Digging the sod out takes
  the flower and its light with it.
- **What the shader does with the bit.** A cell with the bit grows a flower
  whether or not its own roll gave it one: the bit is the one decider, and
  the shader's roll only places and turns the flower, as it does for every
  flower now. Its head is drawn glowing while the column record's new
  `GLOW_LIT_BIT` (bit 29 of `more[3]`) is set, which the tier sets with the
  lanterns' lit bit at dusk. Past the column tier there is no field and no
  bit, and the head is drawn in its day colour. The clutter's reach is well
  inside the tier, so no flower is drawn there anyway.
- **Colour.** The head is a pale cyan-green, the colour the owner's words
  suggest ("some flowers illuminate the ground"). The light it puts on the
  ground is the field's one warm tint, because the block channel carries a
  level and not a colour. A second, cool channel would be a third nibble a
  cell, which decision 1 already turned down for its cost.
- *Alternative:* port the shader's flower roll to Rust and glow a share of
  the rolled flowers. Rejected: two copies of one roll in two languages is
  the thing CLAUDE.md asks to validate against each other, and a bit that
  makes the flower needs no second copy.

**9. Trees and ground clutter take the field at their own cell (found on the
task 5.2 captures, 2026-09-27).**
- **What the captures showed.** Beside a lamp at midnight the ground is lit
  and every grass blade standing on it is black. The clutter and the trees
  never read the field: their vertices keep the default of full sky and no
  lamp, so a lamp cannot light them, and the night's sky term alone draws
  them near black.
- **The fix.** Where the cell has a column, a clutter vertex takes the field
  at the air layer over the cap, as the cap's own centre does, and a tree
  vertex takes it at the layer of its own height, so a canopy over a lantern
  is lit from below and a trunk in a cave is dark. Off the tier nothing
  changes: there is no field there, and full sky is what the far terrain is
  drawn with.
- *Alternative:* light clutter from the moving-thing sampler of decision 2.
  Rejected: clutter is drawn in the terrain pass, which already reads the
  field directly, one lookup a vertex.

**10. How each moving thing takes the field, from what it is drawn with
today (surveyed 2026-09-27, before group 3).** Decision 2 said what is
sampled; this says what the sample does to each thing, because they are drawn
three different ways.
- **What is there.** There is no player body: what a player sees of
  themselves is the held tool and the hand (`held.rs`). Those are an unlit
  material over baked vertex shades, so they are full-bright at midnight and
  in a sealed cave. The ship (the Kestrel's model, the Tern and the Loon), the
  fish and the float are Bevy's lit PBR material, lit by one directional sun
  at a fixed 15,000 lux and a constant ambient. At midnight, and in a cave at
  noon, they are drawn as at noon.
- **The sampler** (task 3.1) is `pbd_core::light::sample`. It takes the
  column a point is in, the point's direction and its fractional layer, and
  the columns' centres. It blends the column and its open neighbours by
  distance, and the two layers the point sits between, and answers sky and
  block in 0..1. Solid neighbours are left out, so a point by a wall reads the
  air and not the rock. With no column, or off the column's span, it answers
  full sky and no block, which is what the far terrain is drawn with. The
  column tier keeps each slot's centre so the app can ask it.
- **The held tool and hand** take ONE sample, at the eye. They are half a
  metre across, so eight corners would be eight equal samples. Each frame
  their shared material's colour becomes the terrain's own lighting term for
  that sample: the sky's fill, `max(floor, mix(night, 1, daylight) * sky)`,
  plus the lamps' `tint * strength(block) * gain`, all from the constants the
  terrain shader uses (decision 5 moves the tint and gain into the core for
  this). Their baked face shades stay, so they keep their form.
- **The ship, the fish and the float keep their PBR look and are scaled by the
  field.** Each gets an extension of its material. Its uniform carries the
  eight corner samples of the thing's bounds in the render frame, those
  bounds, the planet's centre and the sun. The fragment blends the eight
  across the bounds, which is decision 2's bow-lit, stern-dark ship. It
  multiplies Bevy's lit colour by the same sky fill as the held tool, with
  daylight taken at the fragment, and adds the same lamp term. By day in the
  open that is Bevy's own colour, unchanged, so the ship looks as the owner
  has seen it. In a cave it falls to the floor, and at night to the night's
  fill, with lamplight added on the side a lamp is.
- *Alternative:* replace the PBR lighting with the terrain's model outright.
  Rejected for now: it changes how the ship looks by day, which no one asked
  for. Scaling keeps the day look and fixes the night and the cave.
- *Alternative:* dim the directional sun with the clock. Rejected: the same
  light lights the moon, which is in space and in sunlight at midnight.

**11. Daylight crosses a cell as a lamp's light does: three levels (the
owner, survey L1, 2026-09-27: "sounds good, makes sense").** Decision 7 left
the sky at one level per cell and asked the owner whether daylight should
follow the lamps' rule.
- **What changes.** A step to a neighbouring column costs every channel three
  levels. A step up or down still costs one, so a shaft stays at full
  strength for 14 layers, as the reference's comment wants. The flood keeps
  one implementation, and the per-channel cost goes: `light::ACROSS` replaces
  `BLOCK_ACROSS` and `Channel::across`.
- **What it does to a tunnel.** From a mouth at 15, a level tunnel reads 12,
  9, 6 and 3 in its first four cells, and is dark from the fifth, 14 m in.
  At one level per cell it was lit for 14 cells, about 40 m. A cell under an
  overhang beside open ground reads 12, not 14. Open ground is unchanged,
  because an open column is seeded at 15 all the way down to its floor.
- **What it does not change.** The corner darkening (voxel-light's crease)
  reads solid neighbours, not the field. The shader's curve on the sky level
  is unchanged. Only the level that reaches a cell under cover is lower.
- **How it is checked.** The tunnel tests pin 12, 9, 6, 3 and dark. A
  `--spawn mouth` capture at noon is taken before and after, in
  `docs/screenshots/lamps-and-lanterns/`.

**12. The kit makes room for the lights, and the rest waits for an inventory
(the owner, survey L2, 2026-09-27: "need to give the player an inventory
grid, just drop something, dont need those blocks in the inventory really").**
The hotbar has ten slots and the kit filled eight.
- **A new world** is no longer dealt snow, rock or ore: digging gives those
  back. It is dealt grass, dirt, stone and sand, the torches, and grant 2:
  8 street lanterns, 8 wall lanterns, 8 hanging lanterns, 4 braziers and 16
  candles. That is ten slots, all full.
- **An old save** is dealt grant 2 once, into whatever room it has. Anything
  that does not fit is named in the log and not dealt, because there is no
  inventory to put it in. A save whose hotbar is full gets nothing, and the
  log says so.
- **The inventory grid** the owner asked for is its own change,
  `inventory-grid`: a written plan now, built when the owner schedules it
  (survey I1). Once it exists, a grant that does not fit the hotbar lands in
  the grid.

**13. The dusk re-bake takes the contact with it (found 2026-09-27, on the
inventory captures).** Every capture pinned at midnight dug nothing: the
scripted dig's ray found no ground, frame after frame, while the same flags at
noon dug four blocks. `switch_dusk_lamps` replaces `PlanetFine`'s set with a
relit copy, and nothing told `PlanetContact`, whose fine tier still holds the
set it was built from. Everything that asks `contact.serves(&fine.set)`
before trusting an index into the set stops answering until the streaming
next lands a set:
- digging and placing find no ground (the edit would otherwise be made
  against the wrong set's records);
- everything lit by the field (decision 10: the fish, the float, the ship,
  a drop) falls back to open sky and no lamp, so a lantern stops lighting
  the things beside it just as it comes on.

A player standing still at dusk hits it; one walking far enough to land a new
set does not, which is why the lights' captures, all walks or relit by a
landing, never showed it. The fix: a relight changes the light field and
nothing else, and the contact holds the set only for its columns, so at a
relight the contact takes the relit set in place of the old one
(`PlanetContact::relit`), with no rebuild. It refuses where its tier was
built from some other set, which a landing is about to replace anyway.

## Risks / Trade-offs

- [A level-13 lantern floods about 13 cells, over 30 m across 2.833 m cells,
  and could light a whole street from one post] → It did, on the first
  capture. Decision 7 makes a sideways step cost three levels.
- [Hundreds of emitters in a city make the bake slower] → The flood's cost
  follows the lit volume, not the number of emitters. A synthetic city of 300
  lanterns is baked and timed in the tests before any city exists
  (`a_city_of_three_hundred_lanterns_bakes_every_lantern_and_nothing_past_its_reach`).
  Measured 2026-09-27 in the cloud container (a 4-core Xeon at 2.8 GHz,
  release build, nothing else running, five runs), on a 3,136-column patch:

  | bake | median | range |
  | --- | ---: | ---: |
  | with 300 street lanterns | 21 ms | 19.6 to 38.5 ms |
  | with no lanterns | 18.6 ms | 18.2 to 23.4 ms |

  The lanterns cost 1 to 3 ms; the sky flood is the rest. That is over the
  12 ms this note set, so the follow-up is `cities-in-the-world` task 4.0:
  the owner times the same test on real hardware, where a tier bake was 6 ms,
  and the relight moves off the main thread or goes incremental if a city
  is still over 12 ms there.
- [Sampling every moving thing each frame] → Eight cell reads and a blend per
  thing, for fewer than ten things. It cannot be measured in a cloud session
  (see CLAUDE.md), so the owner runs the performance suite on real hardware.
- [The dusk re-bake happens on the frame the clock crosses dusk, and a 6 ms
  hitch at dusk may be visible] → This note first said the re-bake would run
  on "the same worker the edit path uses". The edit path has no worker: an
  edit relights the whole tier on the main thread (`digging.rs`,
  `apply_edit`), which is the six milliseconds the voxel-light change
  measured. The dusk switch (`planet_lod::switch_dusk_lamps`) does the same,
  once at dusk and once at dawn. A tier the streaming builds on its worker is
  baked there at the dusk state it is built for, so a new tier never
  arrives in the wrong state. In the cloud container the switch logged 30 to
  37 ms, with lavapipe drawing on the same CPU, which says nothing about real
  hardware. If the owner sees the hitch, the switch moves to a worker that
  re-bakes a copy and swaps it in only if no edit landed meanwhile.

## Migration Plan

- New materials take new ids after `Torch = 12`, so an old save's edit log
  reads unchanged.
- The kit gains grant 2, which deals the five new lights once to an existing
  save, into whatever room its hotbar has (decision 12).
- Rollback is the previous build. A save that holds the new materials cannot
  be opened by an older build, which is the same as for any material added
  before.
