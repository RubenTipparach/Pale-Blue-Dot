# Design: glowing mushrooms and the carried light

Written 2026-11-19, before any code. This change is the write-up and a
mockup. No game code is in it.

Revised the same day on the owner's notes on the v2 mockup: "no need for
shadow caster from my light source, please improve torch look". The carried
light now casts no shadow (design 7, 8, 13), and how a light looks in the
hand is new (Part C, design 14; mockup version 3).

## Context

The owner made three requests on 2026-11-19, in this order:

1. "Players also asked to pick up glowing mushrooms. You can eat them and glow
   light from your body temporarily."
2. "Allow players to pick up the glowing mushrooms that exist in the game
   today." This narrows request 1. There is no new species and no new
   placement: the plant is the glowing plant already in the game.
3. "Holding a light source, torches lamps or mushrooms or anything that is a
   light should illuminate surrounding area", and "Need to make this fast so
   this would be realtime for the light emanating from player."

The body glow from request 1 and the held light from request 3 are the same
mechanism: a light that goes where the player goes. This design has one
carried light for both (part B). Its speed is the first requirement.

What exists, read in the code:

| Piece | Where | What it does today |
| --- | --- | --- |
| Which cells grow the plant | `pbd_core::flora::glows(key, chance)` | Murmur3's finaliser on the exact cell key with its own salt, against `glow_flower_chance` (0.01, `column.ron`) |
| Carried to the GPU | `planet_column.rs` ~1889 | `GLOW_FLOWER_BIT` (bit 0 of the finest record's `spare[2]`), set for every finest cell when the tier is built |
| Drawn | `planet_surface.wgsl`, flower slot (vertices 216-233) | On a green cell with the bit, a flower is drawn whatever its own roll says. The rolls place and turn it: a stalk 6 cm wide and `flower_height_m` 0.32 tall, then a head of four triangles out to 7 cm with its centre 3 cm up. By day the head is `BLOOM_CODE` (244), pale and cool. While `GLOW_LIT_BIT` is set it is drawn as its own light: `(0.62, 1.0, 0.88) x GLOW_HEAD_GAIN` (0.25) |
| Its light | `ColumnTier::emitters` | From dusk to dawn, while the cell's ground is sod, a block-channel emitter at `GLOW_FLOWER_LEVEL` 3 over the cap. That lights its own cell only, since a step across costs 3 levels (`lamps-and-lanterns` decision 7). It is left out of the L7 lamp bake (`bake_lamps` filters `level > GLOW_FLOWER_LEVEL`), so its light is the flood's single warm tint |
| Picked things that persist | `desktop/fruit.rs`, `trees_taken.rs` | F on aimed fruit writes a `fruit` record by the tree's cell key (schema 1, day and count). It grows back after `FRUIT_REGROW_DAYS` (8). The draw leaves out picked trees' fruit |
| Eating | `pbd_core::cooking::Meals`, `desktop/kitchen.rs` | Right click a dish in hand. Its effect is stored in the `meal` record. The same effect never stacks (the longer time counts). Three effects at once. No hunger (cooking Q1) |
| Effects | `Effect::{Stride, Steady, Keen}` | "Glow waits for a light that follows the player" (cooking design S5, row Glow: "a 4 m light on the player") |
| Placed lights | `Material::emission` | Torch 14, post and wall lantern 13, hanging lantern 12, brazier 15, candle 8, campfire 13. `dusk_lit()` is true for the post and wall lantern |
| How a lamp lights a pixel | `lamp_bake::falloff`, `lamp_cell` in `planet_surface.wgsl` | `nl x w^2 / (1 + (d/2.2)^2) x GAIN`, with `w = 1 - (d/16)^4`, `GAIN` 2.4, scaled by level / 14. The CPU bake decides which lamps each cell sees, then the shader evaluates them per pixel |
| How things are lit like terrain | `field_light.rs`, `field_lit.wgsl` | `LitByField` things sample the field at eight corners each frame. Room lights are a uniform array of point lights evaluated per fragment (`room_lights`, up to 24) |
| Something carried | `held.rs` | `hand_light` shades the hand mesh. Nothing carried lights anything |

## Goals / Non-Goals

**Goals**
- Pick the glowing plant the game already grows, keep the pick across a
  reload, and eat it for a body glow.
- A torch, lantern or glowing mushroom in hand lights the world around the
  player, and so does the body glow. Both use one mechanism.
- That light follows the player every frame, with no lag and no steps from
  cell to cell. It casts no shadow (design 7, owner, 2026-11-19).
- Old saves open as they did.

**Non-Goals**
- A new species, new placement or a new biome. The fungal forest stays off
  this planet ("fungal forest should not be on this planet").
- Hunger, health or any status bar (cooking Q1; `survival-mode` is phase 3c).
- A coloured block channel or a re-tinted plant pool. Both are noted as
  follow-ups.
- Game code. This change is the write-up and a mockup.
- Frame cost. It cannot be measured in a cloud container. The plan for the
  owner's run is design 13.

---

## Part A: the glowing mushroom

### 1. It is the plant the game already draws, and it is called a mushroom

The code calls it a flower. What is drawn is a slender stalk 32 cm tall under
a cap 14 cm across that rises 3 cm to its centre. Its four triangles are a low
four-sided pyramid, which from a standing eye reads as a cap on a stalk, and
at night as a cyan point over a small pool of light
(`docs/screenshots/glowing-mushrooms/game-today-glow-plant.png`, enlarged from
`lamps-and-lanterns`' `meadow-night.jpg`). The owner calls it a mushroom.

- **Name.** "Glowing mushroom" wherever the player reads it: the item, its
  toast, the hint. The good's code is `glowing-mushroom`. In the code the
  constants keep their names (`GLOW_FLOWER_BIT`, `GLOW_FLOWER_LEVEL`,
  `glow_flower_chance`), because they are pinned by tests, and a rename buys
  nothing. Doc comments say "the glowing mushroom (once the glowing flower)".
- **Shape: unchanged here.** A rounder cap is a look change, and look changes
  go to the owner. The mockup offers a "Proposed cap" switch: an eight-sided
  dome, 16 cm across and 6 cm high, with a darker underside and the same
  stalk, the same colours and the same glow. Its screenshot goes beside
  today's shape in the survey. Until the owner answers, the game keeps
  today's four-sided cap. Recommended: the rounder cap, because it is read
  as a mushroom from every side, where today's shape reads as a flower from
  above. The cost is 12 more vertices in the flower slot's budget.

### 2. Where they grow and how often: as today

Placement is `glows(key, 0.01)` on every finest cell. The plant is drawn
where the cell is green, so it grows on every grassy biome's sod: the fields,
jungle, swamp, mountain meadows and tundra moss. That is one green cell in a
hundred, about one every 28 m across a meadow, alone rather than in patches.
This change moves none of it. The fungal forest that Glowcaps came from is not
on this planet, and the owner asked for the plant the game already has.

### 3. Picking with F

- **Aim at the plant itself**, the rule `fruit-aim` set ("picking fruits
  should be aimed at the actual fruit"). `pbd_core::flora::glow_plant_point`
  gives where the plant stands, the shader's own placement on the CPU: the
  corner `k`, the share `f0` and the height, from `roll(id, SALT_FLOWER + 1)`
  and `+ 2` on the record's clutter id, and `shader_hash`, which the core
  already mirrors. A test reads the shader's flower-slot lines and checks the
  CPU's points against them, as `hanging_fruit_points` does. The eye ray picks
  it within `aim::REACH_M` when the ray passes within `glow_pick_radius_m`
  (0.25 m) of the segment from the stalk's foot to the cap.
- **The offer** goes through `pbd_app::interact` as fruit's does. The prompt
  reads `F  Pick`.
- **What a pick gives:** one `glowing-mushroom` good, stack 32, with a 16 px
  icon drawn with the balatro-pixel-art kit from the plant's own colours
  (stalk, cyan cap), with a 1 px outline as UI icons have. It is not placed in
  the world: a dropped one is a 2D item, like every drop.
- **The toast** is `item-toasts`'s own: the icon, "Glowing mushroom" and the
  count, stacked when several are picked in a row.
- **Where it is written:** the pack and a `glow-plant` record in one line, as
  fruit is. The pick is acknowledged only after the save accepts it.

### 4. A picked cell draws no plant and gives no light; it grows back out of sight

- **The record.** `glow-plant` keyed by the cell's exact key, schema 1, with
  body `{ day: f64 }` (the world day it was picked).
  `GLOW_PLANT_REGROW_DAYS` = 8.0 world days, the same as fruit (6.4 hours of
  play at 48 minutes a day).
- **Seeing it go.** The tier's record build already decides the bit once per
  cell. That becomes
  `glows(key, chance) && !picked(key, day)`. A second bit,
  `GLOW_PICKED_BIT` (bit 1 of `spare[2]`), marks the cell as picked, so the
  flower slot draws nothing there. Without it, the slot's own 12% flower roll
  could put an ordinary flower where the mushroom was, and the cell would
  change under the player's hand. With the bit clear, `ColumnTier::emitters`
  adds no emitter, so the pool goes too.
- **On the pick**, the picked cell's record is rewritten in place (one record,
  the path a dig takes), and the tier re-lights as it does after a dig: the
  measured whole-tier bake, about 6 ms off the frame. The plant is gone on
  the frame of the pick. That is the player's own action, as fruit is. The
  pool fades over `BAKE_EASE_S` (1 s) as any re-bake does.
- **Regrowth, recommendation taken (ask only with screenshots): yes, after 8
  days, and only out of sight.** A record past its regrowth only clears the
  bit when the cell is built while it is farther than `clutter_radius_m`
  (120 m) from the camera. A cell that comes due inside that ring stays
  picked until a later build finds it far. A plant never appears in front of
  the player, which keeps the no-pop rule (priority 1) without a fade. The
  alternative, never regrowing, would make a world's supply finite, and a
  long-played world would go dark.
- **The near-camera list.** Like `trees_taken`, the build reads the save's
  `glow-plant` records within the tier's reach. They are few, one per pick,
  and a `HashSet<u32>` of the current picks is rebuilt when a record is
  written.

### 5. Saves and the generator: no new version

- **Placement is not the generator's.** `glows` reads the cell's exact key,
  which the topology and generator version already fix, and a config chance
  in `column.ron`. `TerrainConfig` and `TreeTable` do not change. No height,
  biome, top block, rock or tree moves, so no ground or tree digest moves.
  Versions 4 to 12, and `mountain-ground`'s planned 13, are untouched.
- **Only the picks are stored**, as with fruit. A world nobody has picked from
  writes nothing new. An old save opens exactly as it did, and its first pick
  writes the first `glow-plant` record. The kind is noted in `identity.ron`
  the first time one is stored, as `fruit` was.
- **What would break this, and the guard.** Once picks exist, the chance is
  part of the world. Moving `glow_flower_chance` would make old picks name
  cells with no plant, which is harmless because they expire, and would grow
  plants on cells nobody picked, which is new decoration and not a lost
  record. Even so, a test pins `glow_flower_chance` at 0.01 and the hash's
  salt with a digest over 10,000 sample keys. Its doc says that a change is
  made the way `TreeTable` was: a per-version table in `TerrainConfig`, with
  the old value carried for old worlds. This is the same guard `flora-v2`
  12.1 put on trees, applied before there is a reason to need it.
- **`Effect::Glow`** is a new value of a serde enum inside the `meal` record.
  A save without it reads as before. Saves are not read by older builds, so
  nothing reads `Glow` and fails.

### 6. Eating, and how several stack

- **Key:** right click with the glowing mushroom in the selected slot, the
  path dishes take (`kitchen::eat`). The panel's line "RIGHT CLICK A DISH IN
  HAND TO EAT IT" covers it.
- **The dish row** goes in `cooking.ron`:
  `(good: "glowing-mushroom", effect: Glow, minutes: 3.0)`. It is raw: no
  campfire, no recipe and no stars.
- **No hunger** (cooking Q1): not eating costs nothing.
- **Several**, recommendation taken: cooking's rule unchanged. Eating one
  while the glow runs sets its time to the longer of the two, so each
  mushroom refreshes the glow to a full 3 minutes. It never gets brighter
  and its time is never summed. Glow counts as one of the three effects. One
  rule for every effect is easier to learn than a special case, and a pocket
  of mushrooms still lights a long cave, one at a time. Summing up to a cap
  was considered and turned down, because it lets a player eat ten mushrooms
  at once and forget about them for half an hour.
- **The HUD** is cooking's buff icon over the hotbar with the time left
  (`m:ss`). The icon is the item's.
- **The missions** count it as an `Eat` of `glowing-mushroom`, as they count
  any dish.

---

## Part B: the carried light

### 7. Requirement zero: real time

The owner: "Need to make this fast so this would be realtime for the light
emanating from player." So:

- **It moves every frame.** On the frame the player moves, the light lights
  from the new position. There is no queue, no off-thread bake and no frame
  of lag.
- **It never steps.** Crossing from one cell to the next changes nothing in
  how it looks. Its falloff is worked out from the exact position, not from
  the cell the player is in.
- **It casts no shadow.** The owner, on the v2 mockup (2026-11-19): "no need
  for shadow caster from my light source, please improve torch look". The
  carried light, from a held item and from the body glow alike, is not
  stopped by anything. The column walk from each pixel toward the light
  (v2's design 8 A), its cached-per-cell fallback, and the CPU's walks to
  the eight corners of field-lit things are all dropped.
- **What the player will see because of that.** The light reaches through a
  thin wall: standing beside a one-cell stone wall with a torch, the ground
  on the far side is lit as if the wall were not there, within the light's
  16 m reach and falling off as it does in the open. In a cave or a house it
  lights the next chamber or the next room through the rock or the wall
  between, faintly at a few metres and not at all past its reach. A face
  turned away from the light still takes none (N·L), so the far face of the
  wall itself stays dark and the near face is lit. Placed lamps keep their
  shadows: the baked field is unchanged, and so is the L8 voxel walk for
  the lamps the bake hands to a pixel.
- **It costs little and is measured.** The budget is at most 0.1 ms of GPU
  time at p50 for one carried light at 1080p on the RTX 3070, on a night walk
  (design 8's estimate, with a margin).

### 8. Three ways to draw it, and the one recommended

All three keep the baked field for the placed lamps. They differ only in how
the light that moves is drawn.

**A. A point light in the shaders, with no shadow (recommended).**
- The light is a uniform: a position in the planet frame, an RGB colour times
  strength, and its reach. The terrain fragment (`planet_surface.wgsl`, which
  also draws the trees and the ground clutter), `field_lit.wgsl` and
  `water.wgsl` add `carried(p, n)`, which is `lamp_bake::falloff` on the exact
  distance and N·L, the function every baked lamp uses. One copy, as a WGSL
  function, held to the Rust one by the existing constants test.
- **No shadow** (design 7). Nothing is walked, read back or cached for it:
  the term is the falloff and N·L, and nothing else. v2 of this design walked
  the column records from each pixel toward the light and had a per-cell
  cache as its fallback; the owner dropped the shadow, and both went with it.
- **Early outs:** a pixel beyond the 16 m reach, or one facing away
  (N·L ≤ 0), skips the term. In daylight, with the sun far above the carried
  term, the light is skipped altogether when `lamp_share` is under 1/255.
- **Cost estimate (re-estimated for no shadow).** v2 estimated 0.3-0.5 ms on
  the RTX 3070 at 1080p, and nearly all of that was the walk: L8's voxel walk
  of up to 72 cells costs 1.50 ms of the terrain pass on wet pixels
  (`2026-11-12-l8-split`, 1440 x 900), and the carried walk was taken as a
  fifth of its steps. Without it, the carried term is one uniform read, a
  distance, a dot product and the falloff's few multiplies: about 20
  arithmetic operations, the size of one of the baked lamp terms the
  fragment already evaluates, on the pixels inside the reach. At 1080p that
  is 2 M pixels × about 20 operations, well under 0.05 ms on the 3070 even
  allowing for the extra registers it holds live, and about the same on the
  M4 Max. `field_lit.wgsl` gains the same handful of operations per fragment
  and the CPU loses its corner walks. So the estimate falls from 0.3-0.5 ms
  to **under 0.05 ms** for one light, and four lights (design 12) stay under
  0.1 ms. These are estimates, not measurements (design 13).

**B. A small light volume around the player, spread on the GPU each frame.**
- A 32 × 32 × 32 grid at 1 m (32 KB of occupancy, re-uploaded as the player
  moves a metre or a block changes). A compute pass injects the light into its
  8 nearest voxels by trilinear weight and spreads it 12 steps, blocked by
  occupancy. Every pixel samples it once.
- **Cost:** 32k voxels × 12 steps is about 0.4 M updates, about 0.05-0.1 ms on
  the 3070. The sample is about 0.02 ms, and the occupancy upload about
  0.1 ms on the frames it happens.
- **Why not:** it is a second light representation on a square grid laid over
  hex columns. The spread is a diamond, not a sphere. Trilinear sampling leaks
  through a one-metre wall. There is no N·L unless more is stored. The light's
  shape moves in 1 m steps unless the injection is perfectly smooth. It is
  cheaper than A, but it looks worse, and it duplicates what the field
  already is.

**C. Re-bake the field on the CPU as the player moves.**
- The light becomes an emitter in the block channel. The tier re-lights when
  the player enters a new cell.
- **Cost:** a whole-tier bake is about 6 ms (`lamps-and-lanterns`), plus the
  field's upload (the fine set's `gpu_light` part), on every cell crossed. At
  a sprint's 14 m/s that is about 5 cells a second. A bounded local relight
  would be about 0.3-1 ms, but it needs the removal pass the code itself
  warns about ("a dug cell that stayed dark forever").
- **Why not:** it changes only when a cell is crossed, so it steps, and it
  arrives one or more frames late. Both break requirement zero, whatever the
  cost. Ruled out.

**Recommended: A.** It is the only option that is exact every frame. It
reuses the falloff the game already has, and with no shadow it is the cost
of one more lamp term on the pixels near the light.

### 9. What it lights, and how each thing picks it up

| Thing | Drawn by | How it takes the carried light |
| --- | --- | --- |
| Ground, walls, caves | `planet_surface.wgsl` | `carried(p, n)`, added beside `lamp_near`. No shadow |
| Trees and ground clutter (tufts, flowers, the mushrooms themselves) | the same pass | The same term per fragment. Their normals are the ones the lamp term uses |
| Water | `water.wgsl` | The same term on the surface and the lit fog |
| `LitByField` / `LitLikeTerrain`: drops, craft, props, creatures, a held-up catch | `field_lit.wgsl` | A `CarriedLights` uniform beside `room_lights` (a position, colour times strength and reach per light). Per fragment: falloff and N·L. No occlusion, so nothing is walked on the CPU |
| The hand and the held item | `field_lit.wgsl` (they are `LitByField`, `lamps-and-lanterns` decision 10) | The same uniform. The distance is clamped to `carried_hand_min_m` (0.5 m), so the hand holding a torch is lit warm and not blown out |
| The sky, clouds, rain | - | Not lit. The light is local |

The terrain pass's clutter and trees are drawn in the same fragment path as
the ground, so "everything in the world is lit as the terrain is" holds
without extra work.

### 10. What gives light: one table from the item to its light

`pbd_core::light::carried_light(item: Item) -> Option<CarriedLight>`, with
`CarriedLight { level: u8, tint: [f32; 3], dusk_lit: bool }`. There is one
table, derived and never hand-copied:

| Held item | Level | Tint | Lit when |
| --- | ---: | --- | --- |
| `Item::Block(m)` with `m.emission() > 0` (torch 14, post and wall lantern 13, hanging lantern 12, brazier 15, candle 8, campfire 13) | `m.emission()` | `TORCH_TINT`, as the field gives every placed lamp | `m.dusk_lit()`: the post and wall lantern from dusk to dawn, the rest always |
| `Item::Good(glowing-mushroom)` | `flora::GLOW_FLOWER_LEVEL` (3) | `TORCH_TINT`, as its pool on the ground has today | dusk to dawn, as the plant glows |
| the body glow (`Effect::Glow` running) | `glow_level` in `cooking.ron` (10) | `glow_tint` in `cooking.ron` (0.62, 1.0, 0.88): the cap's own colour (`planet_surface.wgsl`'s head colour, moved to `pbd_core::flora::GLOW_HEAD_RGB` so both read one value) | always, while the effect runs |

- **Strength** is `level / 14`, the scaling the L7 bake gives every placed lamp
  (`bake_lamps`). So a held torch lights the ground at 3 m exactly as a placed
  torch 3 m away does.
- **Anything that emits light later** joins by giving its material an
  `emission()`. It needs no new row.
- **Not in hand, no light.** Only the selected hotbar slot counts. A torch in
  the pack or in another slot gives nothing. Placing the torch moves its light
  from the hand to the cell.
- **The held tint follows the placed light.** A glowing mushroom's light on
  the ground is warm today, because the field has one tint
  (`lamps-and-lanterns` decision 8). So the held one is warm too, and only the
  body glow, which has no placed version, takes the cap's cool colour. A cool
  pool for the planted mushroom, and so for the held one, needs a tint per
  lamp in the L7 bake. That is a follow-up look change, asked with
  screenshots, and not made here.

### 11. Body glow plus a held light: they add, as one light per player

Recommendation taken (ask only with screenshots): **they add.** Two lights on
the same spot add in the real world, and the baked lamps add in the shader.
"Brighter wins" would make a torch hide the glow you just ate. To keep the
cost at one light term, a player's sources are merged into one carried light: its
position is the stronger source's (the hand for a torch, the chest for the
glow alone), and its colour is the sum of colour × strength. The tonemapper
rolls the top off, so the sum never goes white.

- **Where it is:** a held light at the held item's position
  (`pbd_core::hand`'s grip point in the eye frame, put into the planet frame
  each frame). The body glow is at the chest, 0.55 m under the eye.
- **The fade at the end of the glow:** over the last `glow_fade_s` (20 s) the
  strength falls on a smoothstep from 1 to 0, so the light dims and goes out
  and never cuts. The HUD's timer counts down to 0:00 as it does.
- **Starting:** the glow comes up over `glow_rise_s` (0.6 s) after eating, a
  light being lit.

### 12. Other players, third person and the server

- **Seen on the body.** There is no player body yet (`lamps-and-lanterns`
  decision 10). When there is one, a third-person view or another player
  carries the light at the held item's bone, or at the chest for the glow.
  Until then the third-person position is the same offset from the eye.
- **How many.** `MAX_CARRIED_LIGHTS` = 4 per frame: your own and the three
  nearest others within 32 m. Each costs one falloff term on the pixels it reaches.
- **Multiplayer (phase 3b, `multiplayer`).** The server owns the picks (a
  `glow-plant` record is a world write, made through its durable path and
  named with its author), what is eaten (the `meal` record), and which slot
  each player holds. A client draws other players' carried lights from that
  replicated state: the held item in the selected slot, and the Glow effect's
  `until_s` with the shared world clock, so everyone sees the same fade at the
  same time. Nothing about a light is sent per frame beyond the position a
  player already sends. Two players picking the same mushroom at once: the
  server accepts the first write and refuses the second, and the second
  player gets no item and no toast.
- **Saves:** nothing about the carried light is stored. It is worked out from
  the slot and the `meal` record every frame.

### 13. Measuring (the owner runs it)

This session has no GPU (lavapipe/SwiftShader), so per CLAUDE.md nothing here
is a frame time. The plan, for when the light is built:

1. **A scenario**: `perf_suite.py`'s `walk` at 22:00 (`--time 22`), as
   `night-carry`. It walks the same 1,000 m with a torch held
   (`--hold torch`), then again having eaten a glowing mushroom
   (`--eat glowing-mushroom`), then with both. It also has a cave leg
   (`--at` a known cave mouth), where the most pixels are inside the reach.
2. **A/B in one sitting**: `--variant "before|<main exe>" --variant after`,
   plus `PBD_CARRIED=0` on the after build to price the light alone.
   Release build, nothing else running, frame latency as shipped, 1080p.
3. **Read**: p50, p95 and p99 against the spread between repeats, and the GPU
   per pass (`main_transparent_pass_3d`, where the terrain is drawn; and
   `main_opaque_pass_3d` for the field-lit things).
4. **Budget**: at most 0.1 ms at p50 for one light on the 3070. With no walk
   there is no fallback to take: if it is over, the early outs (design 8) are
   where to look, and it is measured again in the same sitting.
5. **Both machines**: the desktop (RTX 3070) and the M4 Max, the report in
   `docs/benchmarks/<date>-carried-light/`, and the HTML page republished.

The owner's own check of smoothness is a real-time `obs-record` walk at night
with a torch past the wall and through the doorway, where the light now
reaches the far side through the wall. A `--capture` sequence
steps at a fixed rate and cannot show it.

---

## Part C: how a carried light looks in hand

### 14. The torch, the lantern and the mushroom in the hand

The owner, on v2's held torch (2026-11-19): "please improve torch look". In
v2 (`held-torch.jpg`) the torch was a flat brown stick with a pale orange
cone on top, in a plain box hand, and it read as a pencil. This is a look
change, so it is drawn in the mockup first (v3) and the game changes only on
a request. The motion follows the `animation` skill: the flame and the hold
are drawn only, and nothing about them feeds a rule.

- **The haft.** A six-sided stick 4 cm across and 0.45 m long, in an
  8 x 32 pixel bark texture: strands down it from the wood ramp, darker
  grain runs, the odd pale fleck and a knot. Flat faces, so it reads as a
  stick at pixel size, as the placed torch's post does.
- **The head.** A rag wound round the top of the haft and soaked in pitch,
  14 cm tall and 8.6 cm across at its fattest, in a 16 x 14 texture: the
  cloth in diagonal wraps with dark seams and pitch stains, a cord binding
  its foot, the crown scorched to a charred crust with a few live embers in
  it. Lit like the terrain (and by its own flame, through the carried light
  with the hand's clamp), with no outline.
- **The flame.** Three camera-facing cards in a pixel flame sheet (eight
  12 x 20 frames, 11 a second), each card on a different frame. Every frame
  is quantised to the flame ramp of the palette: a white-yellow core, yellow,
  orange, and red tips, with no soft edge. The outer card keeps its red rim,
  the middle one only its orange and hotter texels, the inner one only its
  yellow and white heart, so the colours stack and the tongue licks
  irregularly. A few square ember pixels rise from it, drift, cool from
  yellow to red and go out, and they rise in the world, so a walking torch
  leaves them behind. A soft additive glow sits round the head. Fire moves
  on its own, which the "nothing spins on its own" rule allows.
- **The flicker drives the light.** One function gives the flame's height,
  its glow and the carried light's strength: a few slow waves plus a small
  step each time the sheet changes frame, about ±15 % in all. So the light on
  the ground flickers in step with the flame drawn. The held torch's light
  level is still `Material::emission` (14); the flicker only scales it about
  that value. In the game this is one more term on the uniform's strength,
  worked out on the CPU each frame.
- **The flame stands on the view's up in the hand.** A flame on the world's
  up splays outward at the edge of the view when the player looks down, so
  the held flame is drawn upright on the screen, as first-person games draw a
  held flame. A placed flame stands on the world's up. The flame leans back
  a little against the hand's motion when walking.
- **The hold.** The approved hand (M36 Tools in Hand, the Hand tab,
  hand-model design 5; the copy in `docs/mockups/river-craft/hand.js`) closes
  on the haft at the game's scale (`held_tools.ron`'s 0.0374 m a tool unit),
  held up and to the right with the head leaning a little away and in. The
  flame stands at the right of the view about chest-high, clear of the
  crosshair. The hold has a light idle sway: the step's side-to-side and bob
  while walking (one step per 0.75 m, 1 cm across, 0.8 cm of bob, 0.02 rad of
  roll, eased in and out over about a sixth of a second) and a 3 mm breath at
  rest. These are the hold's tunables, a block like `held_tools.ron`'s.
- **The lantern** is held by its bail, the fist closed on it palm down, the
  lantern hanging under it: an iron frame (a pixel iron texture), a roof and
  a base, and four lit panes, each a small flame behind glass, glowing as the
  game's lantern panes do from dusk to dawn and dark glass over a dead wick
  by day. It swings a little from the bail as the player walks, behind the
  step. Its light flickers at a third of the torch's.
- **The glowing mushroom** is held by its stalk, the cap a hand's width over
  the fist at its own size: the plant's own cap (today's four-sided one, or
  the proposed dome when the Cap switch says so) on a round stalk in a small
  pale-green texture, since the ground's flat stalk card reads as a blade
  that close. Its light does not flicker.
- **The placed torch is the same object.** The game's placed torch
  (`planet_surface.wgsl`'s lamp slot, `LAMP_TORCH`) is a four-sided post
  0.55 m tall in `LAMP_WOOD_CODE`'s flat brown with one flat triangle 26 cm
  wide and 16 cm tall on it, shaded orange to red: it looks worse than the
  new held torch and does not match it. The mockup's Placed row stands both
  side by side (`torch-v3-placed.jpg`). **Recommended, as a separate future
  look change:** the placed torch takes the held torch's head, flame cards,
  embers and flicker, on a 0.55 m haft, so a torch is the same object in the
  hand and on the ground or a wall. That is a change to the lamp slot's
  vertices (the flame needs cards, so more than its 30 vertices today) and
  to the lamp fragment's colours, so it goes to the owner with those
  screenshots and is not made here. Until then the held torch is drawn as
  above and the placed one as today.

## Tunables, with units (validated RON)

| Name | File | Default | Unit | Valid |
| --- | --- | ---: | --- | --- |
| `glow_flower_chance` | `column.ron` | 0.01 | share of finest cells | 0..1, pinned by a digest test (design 5) |
| `glow_plant_regrow_days` | `scatter.ron` | 8.0 | world days | > 0 |
| `glow_pick_radius_m` | `scatter.ron` | 0.25 | m | 0.05..1 |
| dish row `glowing-mushroom` | `cooking.ron` | Glow, 3.0 | minutes of play | > 0 |
| `glow_level` | `cooking.ron` | 10 | field level, 0..15 | 1..15 |
| `glow_tint` | `cooking.ron` | (0.62, 1.0, 0.88) | linear RGB | each 0..1 |
| `glow_fade_s` | `cooking.ron` | 20.0 | s of play | 0..minutes × 60 |
| `glow_rise_s` | `cooking.ron` | 0.6 | s | 0..5 |
| `carried_hand_min_m` | `held.ron` | 0.5 | m | 0.1..2 |
| `carried_max_lights` | `column.ron` | 4 | lights | 1..8 (the shader's array size; a test holds them together) |
| `carried_others_m` | `column.ron` | 32.0 | m | 0..128 |
| `torch_flicker` | `held.ron` | 0.15 | share of the light's strength, peak to mean | 0..0.5 |
| `flame_fps` | `held.ron` | 11 | frames a second | 4..24 |
| hold `grip`, `up`, `arm` per held light | `held_tools.ron` | design 14 (the mockup's `HOLD` block) | m and directions in the eye frame | as `held_tools.ron`'s |
| sway `step_m`, `side_m`, `bob_m`, `roll_rad`, `breathe_m`, `ease_hz` | `held.ron` | 0.75, 0.010, 0.008, 0.02, 0.003, 6 | m, rad, Hz | each ≥ 0 |

Levels for held lamps are not tunables. They are `Material::emission`, the
one authoritative value.

## Risks / Trade-offs

- **Light through walls.** With no shadow, a carried light reaches through a
  thin wall into the next room or chamber (design 7). The owner chose this
  ("no need for shadow caster from my light source"). It is faint past a few
  metres and gone past 16 m.
- **The estimate could be wrong.** It is now one lamp term per pixel in
  reach, and the budget is checked before shipping.
- **Several players with torches in a cave** make four light terms per pixel
  there. `carried_max_lights` caps it, and the scenario's cave leg measures
  it.
- **Tint mismatch.** A held glowing mushroom gives a warm light while its cap
  is cyan, because its planted pool is warm today. The fix is one look change
  for both (design 10), asked with screenshots.
- **Regrowth out of sight** means a player standing in one meadow for 8 days
  sees no regrowth until they walk away. That is acceptable, and far better
  than a pop.
- **The re-light on a pick** is the existing 6 ms whole-tier bake, the same as
  a dig, off the frame.

## Open questions

Each goes to the survey with the mockup's screenshots:

1. The rounder cap (design 1): today's four-sided cap or the proposed dome?
   Screenshots: `docs/screenshots/glowing-mushrooms/cap-today.png` and
   `cap-proposed.png`.
2. Regrowth in 8 days and out of sight (design 4): recommendation taken.
3. Add rather than brightest-wins (design 11): recommendation taken.
4. A cool pool for planted and held mushrooms (design 10): a follow-up, not
   asked until it has screenshots of its own.
5. A held glowing mushroom is faint. At its placed level 3 it lights about a
   metre round the hand (`held-compare.jpg`, bottom right), where the body
   glow lights about 4 m. That is what "one authoritative value" gives. If
   the owner wants a held mushroom to light more, the change is to the
   plant's own level, so the planted pool grows too. Raising only the held
   value would make a second copy. This goes to the survey with
   `held-compare.jpg`.
6. The placed torch matching the held one (design 14): a future look change,
   recommended, with `torch-v3-placed.jpg`. Not made here.

## Mockup

The Glow and carried light tab of the Foliage Detail Playground (M13,
`docs/mockups/foliage.html#glow`, version 3; version 2 drew the first
write-up, with the shadow and the box torch). M13 was chosen over M14
(Lighting) and M27 (Hearth and Kitchen) for three reasons:
- the plant is ground cover, and M13 is the flora page;
- its hex patch, the game's atlas and the Tenebris trees are already there;
- a new tab adds to the approved playground without changing it.

The playground and the tab each own a renderer. A renderer is made when its
tab first opens, and its loop stops while the other tab is open. Its
shader ports the terms named above: the lamp falloff with no shadow for the
carried light, the level-3 pools, and the cap's night colour. Version 3 adds
the held torch, lantern and mushroom of design 14 in the approved hand, a
Held switch (Before is v2's boxes, Now is v3), and a Placed switch that
stands the game's placed torch beside the matching one. Shots, rendered in
software (Chromium on SwiftShader), are in
`docs/screenshots/glowing-mushrooms/`, the v3 ones as `torch-v3-*.jpg`.
