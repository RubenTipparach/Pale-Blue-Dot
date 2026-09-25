# Design: fishing and equipment

Every number here is either measured in this repository, read from Tenebris
(`tenebris-rs`, whose path and line are given), or a proposal marked as a
tunable that goes into validated RON. The mockup
(`docs/mockups/fishing.html`,
<https://claude.ai/artifact/9XiEvkxSFJMtqaktg4eovj>) runs every mechanic
below. Where the mockup differs from what is proposed, the difference is
stated.

## 1. What Tenebris does, and what is taken from it

Measured by reading `tenebris-rs/crates/` (the explore report is summarised
here). Tenebris is the reference, but only its rules are ported, never its
layout.

| Quantity | Tenebris | Where | Here |
| --- | ---: | --- | --- |
| Cast speed, lift | 11.0 m/s, 3.5 m/s | `tenebris-core/src/fishing.rs:41-57`, `assets/config/fishing.yaml` | 5 to 18 m/s by charge, plus 3 to 6 m/s lift (section 4) |
| Gravity on the bobber | 16 m/s² | same | 25 m/s², the planet's own |
| Wait for a bite | 2 to 6 s, a hash | `tenebris-client/src/fishing.rs:249-256` | Emergent: the time for a school to arrive (section 4) |
| Hook window | `0.9 * (1 - (strength-1) * 0.13)`, floor 0.25 s | `client fishing.rs:338-343` | **Taken as is** |
| Strengths | minnow 1, ray 2, eel 3, serpent 4, reef 5 | `tenebris-core/src/fauna.rs:297-305` | Per species in `fauna.ron` |
| Reel | a 0.9 s lerp to the rod tip, no tension | `client fishing.rs:286-301` | Held reel against a tension meter (owner question 5) |
| Catch | 1 fish, 15% chance of a second | `client fishing.rs:305-331` | 1 fish; the bonus is a tunable at 0 |
| Where the catch comes from | A weighted table. Live fish have no effect | `fauna.rs:354-404` | **The fish that bit is the fish caught** |
| Fish movement | Each fish alone: idle, walk, flee; no schooling | `fauna.rs:686-725` | Boids per school |
| Swim band | floor + 0.4 m to sea − clearance (0.72 to 1.40 m by species) | `fauna.rs:237-242, 1143-1203` | Per species depth band in `fauna.ron` |
| Spawn ring, despawn | 14 to 32 m, despawn beyond 70 m, 8 animals total | `fauna.rs:819-880`, `client animals.rs:37-55` | 20 to 60 m ring, despawn beyond 90 m, per-species cap |
| Rod grip, tip (eye frame) | grip right 0.18, up −0.30, fwd 0.22; tip right 0.26, up 0.05, fwd 0.95 | `client fishing.rs:85-91, 372-432` | **Taken as is**; the mockup uses them |
| Tool kinds | pick for rock/ore, axe for wood, shovel for soil/sand/snow | `tenebris-core/src/mining.rs:75-100` | **Taken as is** |
| Break time | base × tier multiplier; wrong tool × 2.0 | `mining.rs:206-221` | Base per material; wrong tool × 4 (owner question 4) |
| Equipping | whatever is in the selected hotbar slot | `client interact.rs:80-91` | A dedicated tool slot (section 2) |

The main departure is that **the fish you see are the fish you catch.**
Tenebris rolls its catch from a table no matter what swims past. That is why
its known gap, per-planet water rosters, is a gap at all: the table and the
spawner are two sources of truth that disagree. Here one roster feeds the
spawner, and the catch is always a fish the spawner made, so the two cannot
drift apart.

## 2. The tool slot and the picker

**The slot.** It is a new store beside `Slots<10>`, in `pbd_core::inventory`:

```rust
pub enum Tool { Rod, Shovel, Pickaxe, Axe }
pub struct Equipment { owned: ToolSet, held: Tool }   // ToolSet: one bit per tool
```

- It is kept separate from the ten slots because a tool is not a stack. Keeping
  the tools out of the ten means the tools never compete with blocks and fish
  for room.
- `Tool::Pick`, the placeholder, becomes `Tool::Pickaxe`. It was never
  constructed, so no save contains it (section 8).
- `Item::Tool` stays, because a future dropped tool or a chest is still an item.

**Revised by the owner after the first build (2026-09-25): boarding is F, not
a tap of G.** E, the owner's first choice, is taken aboard (Q and E are the
Kestrel's yaw, the Tern's hiking and the Loon's blade), so interaction is F,
as Tenebris binds it. F was the walk/fly switch, which moved to R (Tenebris's
key for it), and return-to-spawn moved from R to H. G is therefore only the
tool picker: it opens on the press, with no threshold, and boarding is back on
the press of F. The table below is the first build's design and is kept for
the record.

**Input**, on foot only (first build, superseded above):

| Event | Result |
| --- | --- |
| G down | Note the time. Nothing else happens yet. |
| G held 0.18 s | The picker opens beside the tool slot, highlighting the held tool. |
| Wheel while the picker is open | Moves the highlight and wraps. The slots and the camera zoom do not see it. |
| G up with the picker open | Equips the highlight. A change of tool is a save (section 8). |
| G up before 0.18 s | A tap: board the craft in reach, as today. |
| Esc with the picker open | Closes it without a change. |

- **Aboard, G is only board and leave.** A tool cannot be changed in a seat.
- **The one arbitration point** is a small system that turns raw G into either
  a `Tap` or the picker. `board_or_leave` then reads `Tap` rather than
  `just_pressed(KeyG)`. It is one reader of the key, so the two uses cannot
  both fire on the same press.
- **0.18 s** is the mockup's number, a tunable in `tools.ron`. It sits above a
  deliberate tap (about 0.08 to 0.12 s) and well below a hold anyone would
  notice.

**The cost to vehicles.** Boarding moves from the press to the release, which
is the length of a tap later.
- `vehicles/tests.rs::tap` presses, updates, then releases and clears without
  another update. It must gain an update after the release, in the same commit.
- The vehicles spec ("One interaction key boards and leaves") still holds and
  gains a scenario for the hold.

**The HUD**, in the mockup:
- The tool slot is 52 px, amber-bordered and labelled "Tool · G", sitting left
  of the ten 44 px slots.
- The picker is a column above it: an icon, the name, and one line on what the
  tool is for.
- The selected row is amber. The column draws every owned tool and nothing
  else.
- It uses the same bordered-square style the slots already use. Nothing is
  written as a list of names without an icon.

## 3. Digging asks what is in hand

`dig_and_place` changes from a click to a hold.

- **Starting.** On the first frame the left button is down with a target, a
  `Breaking { cell, layer, elapsed }` starts, and the time it needs is looked
  up once from the material and the tool in hand.
- **Progress.** It advances while the button stays down and the aim stays on
  the same cell and layer. Letting go, or moving the aim, resets it to nought:
  Minecraft's rule and Tenebris's (`tenebris-client/src/interact.rs:578-596`).
- **Finishing.** At `secs(material, tool)` it takes the layer through the
  existing `apply_edit(Hands::Take, ...)`. The durable path, the GPU update and
  the slot give are all unchanged.
- **Holding on.** With the button still down, the next block starts after a
  short pause (`between_s`, 0.15 s), so a player tunnelling holds the button
  rather than clicking once per block. Minecraft pauses five ticks (0.25 s)
  for the same reason: without it the next block, usually the one behind,
  starts on the frame the first one goes and reads as the tool skipping it.

The rule is a pure state machine in the core (`pbd_core::dig::Breaking`), so
it is tested without an engine; the app hands it the target, the button and
the frame's time.

```text
secs = matrix[class(material)][tool]
rod  -> never: the rod casts, and the left button is the fishing line's
```

**A matrix, not a right tool and a penalty.** The first build gave every
material one right tool and charged any other four times as much, which
makes every wrong tool equally bad: a pickaxe was as poor at dirt as a shovel
at stone, and the axe, with no trees to fell, was simply a slow everything.
The owner asked for a matrix instead, one time per tool for each kind of
material, so each tool has its own character against dirt, rock and wood.

The times with the best tool are tuned against Minecraft (0.4 to 0.75 s for
dirt, 0.4 to 1.15 s for stone with the right tool) and are slower than
Tenebris's own (2.5 s soft, 5 s rock with a wooden pick,
`tenebris-rs/assets/config/mining.yaml`) only where Tenebris has tool tiers
to buy the time back with, which this has not. The first draft's 0.25 s for
soft ground is too fast to see a crack: fifteen frames across ten stages.

| Class | Materials | Shovel, s | Pickaxe, s | Axe, s |
| --- | --- | ---: | ---: | ---: |
| dirt | Grass, DryGrass, JungleGrass, Soil, Dirt, Sand, Snow | **0.5** | 1.5 | 1.2 |
| stone | Stone | 4.0 | **1.2** | 3.0 |
| rock | Rock | 5.0 | **1.6** | 4.0 |
| ore | Ore | 6.5 | **2.0** | 5.5 |
| wood | Wood (section 5, not built) | 2.5 | 2.0 | **0.6** |
| placed | Torch | 0.1 | 0.1 | 0.1 |

The reasoning behind the off-diagonal cells, so they can be argued with:

- **Dirt.** A pickaxe breaks it but lifts none of it: three times the shovel.
  An axe chops through roots and turf a little better than a pick: 2.4 times.
- **Stone, rock, ore.** A shovel blade only scrapes: a little over three times
  the pickaxe. An axe chips stone with its edge: two and a half times.
- **Wood.** A pickaxe splits a log faster than a shovel does, and neither is
  close to the axe.
- **Placed things**, a torch, come away with any tool at once.
- **The rod** breaks nothing, whatever the row. Water and air are never a
  target, since the aim ray passes through both.

The best tool in each row is the one the table bolds, and a test holds the
shipped defaults to that, so a retune that made the pickaxe the best shovel
would fail loudly rather than quietly.

The classes and `secs` live in `pbd_core::dig`, with the matrix in
`assets/config/dig.ron` as one row per class and one field per tool, and a
test that the shipped file equals the code defaults. That is the same
pattern as `vehicles.ron`.

**The wood row is ready before the wood is.** There is no wood block yet:
the trees are drawn on the GPU alone (section 5), and a new material needs a
shader code (the column pass packs materials in 4 bits, and 15 of the 16
codes are spent), an atlas tile, a save code and a slot thumbnail. Until
trees can be felled the row is data a test reads, and the axe's column is
what it does to dirt and stone.

### The crack overlay: the block shows how far along it is

The owner asked for Minecraft's breaking effect: dark cracks drawn over every
face of the block being mined, growing as it is mined. Tenebris does the same
thing (`break_stages.png`, six 32 px stages, blended at 0.9 over the mined
cell by a crack channel in its hex shader).

- **Ten stages**, Minecraft's `destroy_stage_0` to `_9`. Stage `k` is shown
  while progress is in `[k/10, (k+1)/10)`. Tenebris has six, and its jump from
  stage 1 to stage 2 is visibly the biggest; ten even steps read as a crack
  growing.
- **One set of fractures, revealed.** A generator
  (`tools/gen_break_stages.py`, stdlib and zlib, `--check`) walks a fixed set
  of jagged fracture lines out from the middle of a 32 px square and gives
  every crack pixel an order. Stage `k` draws the first `(k+1)/10` of them,
  so each stage contains the one before it and the block reads as one block
  cracking, never as ten different pictures. Crack pixels are near black at
  0.85 alpha with a lighter pixel beside some of them, which is what gives
  Minecraft's cracks their chipped edge. It writes
  `assets/textures/break/stage_0.png` to `stage_9.png`.
- **Its own mesh, not a terrain shader term.** The overlay is a small prism
  built on the CPU from the targeted cell's record (its degree, its six or
  five corner rays) between the layer's bottom and top, pushed out 1 cm so it
  sits on the block's faces rather than fighting them for depth. It is drawn
  unlit and alpha blended, one entity, rebuilt only when the target changes and
  given a new stage's material when progress crosses a stage. Tenebris put its
  crack in the terrain shader, and the cost there is a uniform and a branch on
  every terrain fragment for a decal that covers one block; here it touches
  nothing the terrain pass does, and the overlay is a few dozen triangles.
- **Its pixels land on the block's pixels.** The overlay uses the terrain
  shader's own UV mapping (`planet_surface.wgsl`): a side face runs `u` 0 to 1
  along its edge and one tile per metre of height, and a top is the
  tangent-plane projection at `1.5 * tile` about the cell's axis, with the
  same reference vector. The crack texture is 32 px like the atlas tiles and
  sampled nearest, so a crack pixel is exactly a block pixel.
- **Nothing is drawn when nothing is being broken**, and nothing while the
  rod is in hand.

The progress ring round the reticle that the first draft had is dropped: the
cracks are the progress, on the block itself, where the player is looking.

**Placing does not change.** Right click puts the selected slot's block
wherever it goes today, whatever tool is in hand. The one exception is while a
line is out: then right click reels in and places nothing.

The scripted `--dig` capture is a measurement instrument that digs N blocks in
one frame, and stays that way: a picture of a hole needs the hole, not the
time it took. A new `--break SECONDS` holds the button on the block under the
reticle for that long, so a capture can photograph the cracks at a chosen
stage.

## 4. Fishing

**States**, all in `pbd_core::fishing` so single player and a future
multiplayer share them:

```text
Ready -> Charging -> Flying -> Floating -> Nibble -> Bite -> Hooked -> Ready
                          \-> Ready (landed on ground / flew 5 s)    \-> Ready (snapped / reeled in)
```

- **Charge and cast.** Holding left fills `power` over 1.2 s. On release the
  bobber leaves the rod tip at `look * (5 + 13*power) + up * (3 + 3*power)`
  m/s and falls under the planet's own gravity (from `gravity.rs`, not a
  separate constant).
  - Tenebris uses a fixed 11 + 3.5 m/s under 16 m/s². With a level look from
    a 1.6 m eye, its reach is about 8 m. Here, with the same level look, a full
    charge reaches about 12 m and a tap about 2.5 m, so the charge is what
    chooses the spot.
  - If it lands on ground (`ground_under(point) > sea radius`), or has flown
    for 5 s (Tenebris's timeout), it returns empty.
- **Floating.** The bobber's height is `pbd_core::sea::LocalSea::height`, the
  same function the hulls float on, so the bobber and the boats ride one sea.
  Its dip during a bite is 0.14 m (Tenebris uses 0.28 m on a smaller bobber).
- **Scent and approach**, checked every 0.5 s once the splash has faded
  (1.2 s):
  - A school whose centre is within 3 × `sense` of the bobber turns its goal
    to a point under the lure, at half the local depth clamped into its band,
    and swims with the stronger `lured` weight.
  - A school within `sense` rolls `bite * weather * 0.175` each check. On
    success, the nearest fish that is not spooked leaves the school for the
    lure.
  - **The wait for a bite is therefore the time a school takes to arrive**,
    not a hash. Measured in one headless mockup run with the lure 11 m out:
    the first nibble came about 9 s after the splash, and a second 17 s after
    that fish spat the hook. A run from before the scent rule had no visitor
    in 40 s of simulated time.
  - Sense ranges are the tunable that sets the wait. The design target is
    Tenebris's 2 to 6 s once a school is near. A player who casts where no
    school is waits longer, which is intended: where you cast matters.
- **Nibble.** One to three nibbles of 0.6 to 1.4 s each, with a small dip.
  Hooking during a nibble spooks the fish (2 s) and returns to Floating.
- **Bite.** The hook window is Tenebris's formula. If it passes without a
  hook, the fish spits the hook and swims away spooked.
- **Hooked.** The fish alternates between running (0.8 to 2 s, pulling at its
  species' `pull`) and resting (1.2 to 3 s).
  - While the button is held, the line shortens at 1.8 m/s (slower during a
    run) and tension rises by `0.35 + 0.8 * pull * stamina` per second.
  - Released, tension falls at 0.7/s and a running fish takes line back.
  - Stamina falls while the fish runs against tension, so a patient player
    always wins.
  - Tension 1 snaps the line and the fish is lost. Line under 1.3 m, or the
    bobber in water under 0.15 m deep, lands the fish.
- **Catch.** `Slots::give(Item::Fish(species), 1)`. A full hotbar refuses it,
  and the fish is released with a message rather than lost silently.
  Recording it is section 8.

**Weather.** The bite multiplier reads the atmosphere at the bobber: clear
×1.0, overcast ×1.35 and rain ×1.7 in the mockup. Here it becomes a smooth
function of cloud cover and rain rate from `Atmosphere::sample`, with those
three values as its anchor points in `fauna.ron`. The sea state already
rises with the wind, so the bobber rides rougher water in a storm without
extra code.

**The rod model and line.**
- The rod is a tapered box from Tenebris's grip to Tenebris's tip, in the
  eye's frame, drawn on the walker camera like any first-person model.
- The line is a 24-point curve that sags with slack and straightens with
  tension. Each point is clamped to lie on the sea surface rather than hang
  through it. The mockup found this: a sagging line otherwise dips under the
  water and loops.
- The bobber is two half-spheres, red over white.

## 5. The axe and trees

**The measured fact.** A tree is decided on the GPU and nowhere else.
`planet_visibility.wgsl::has_nearby_foliage` (lines 171-196) calls it "the
sole foliage eligibility decision":
- There must be a grass top, or snow in the tundra.
- `hash(cell_id) & 0xff < density(biome) * cover(level)`, with densities
  jungle 115, swamp 34, fields and beach 13, tundra 2, out of 256.
- It applies only on the three finest levels.

The CPU cannot say whether a tree stands in a cell, so an axe has nothing to
aim at.

**What felling needs:**
1. **One rule, in the core.** A new `pbd_core::flora::tree_at(cell_id, top,
   biome) -> bool` at the finest level, with the shader's hash and densities.
   It carries the density table, which the WGSL then reads from a uniform
   rather than repeating the literals.
   - The WGSL remains a transcription, so it gets the treatment the sea table
     got: a compute-shader test on a headless adapter that runs the shader's
     `has_nearby_foliage` over 10,000 cell IDs and compares against the core.
   - This is the "validate actual artifacts, not two hand-written copies"
     rule in CLAUDE.md.
2. **A ray against the tree.** The aim march samples the fine set, and trees
   are not in it. The aim also tests the target cell's trunk prism (0.20 of
   the cell, trunk height 3 to 6 m by biome, from the vertex shader) and
   canopy. The nearer hit wins.
3. **A durable "felled" edit.** An `Edits` entry gains a cell-level flag. It
   is uploaded with the fine set, and the visibility pass then treats the cell
   as having no tree. The flag goes in the save log (section 8).
4. **Wood.** A new `Material::Wood`. The tileset already has a wood tile at
   (2,1) of `fields.png`, so the thumbnail test passes with no new art, and
   wood is placeable as a block. Felling gives
   `Wood × (trunk height)`, 3 to 6. The mockup gives 4 logs.

The core and the WGSL will read the `Material` enum's new variant through the
existing material table and its test. Nothing matches on a raw number.

## 6. Fish schools

**Where they run.** `pbd_core::fauna`: a `School` holds packed `positions`,
`velocities` and `spook` for its fish, plus a goal and a seed. It depends on
nothing but `glam` and the sea and ground queries it is handed.

The app owns spawning around the player and one `InstancedMesh`-style draw per
species. CLAUDE.md puts cosmetic particles in packed arrays or GPU buffers.
Fish are not purely cosmetic, because the one that bites is the one caught, so
they are simulated on the CPU (the authority) and uploaded as an instance
buffer. Nothing is read back.

**The rules, per fish, per step**:

| Term | Weight | Radius |
| --- | ---: | --- |
| Separation, `Σ (p − q) / d²` | 2.6 | 0.55 m |
| Alignment to neighbours' mean velocity | 1.0 | 2.2 m |
| Cohesion to neighbours' centre | 0.7 | 2.2 m |
| Goal, normalised, clamped at 1 m | 0.45, or 1.4 when lured | - |
| Walls: surface − 0.25 m, bed + 0.25 m, water shallower than the band | 6 per m of intrusion | 0.3 m margin |
| Splash, away from the bobber | 12 × (1 − d/4 m) | 4 m, for 1.2 s |

- Speed is clamped to the species' band, doubled while spooked.
- The goal moves to a new random point in the species' water every 6 to 12 s.
- These are the mockup's weights. They go into `fauna.ron` with units.

**Cost.** Each school is O(n²) in its own fish, 34 fish at most in the
mockup, so about 1,200 pair tests. At 30 Hz with six schools resident that is
about 0.2 M pair tests a second. The app steps fauna at a fixed 30 Hz on the
simulation clock (`platform/clocks`), not at the frame rate, and interpolates
the draw.

**Determinism.**
- A school's seed is `hash(cell_id, species)` of the cell it spawned in.
- Its step is a pure function of its state, the step and the sea at the saved
  world time.
- School order is by spawn cell ID, never hash-map order.
- Fish are not saved. They are respawned from the ring when a world loads,
  which is what "they are not world mutations" means. A catch is saved; the
  population is not.

**Spawning.**
- The ring is 20 to 60 m from the player.
- Candidates are fine-set cells whose water depth covers the species' band
  plus 0.3 m. Depth is `sea radius − ground_under`, the same ground rule the
  boats use.
- Each species has a cap on resident schools (`max_schools`, section 7),
  and a school is dropped beyond 90 m.
- A candidate cell also has to match one of the species' **water classes**,
  with the water temperature inside its window (section 7).
- **Gate:** a body whose `fauna.ron` roster is empty spawns nothing, and the
  rod's aim line says "nothing lives in this water". That covers the airless,
  frozen and asteroid catalog worlds.

The roster and each species' entry are section 7.

## 7. Species, where they live, and the field guide

**The owner's addition:** "one more thing to add to the design, is different
species of fish, you may reuse fish from tenebris, but make sure each fish has
a wiki entry, and a thumbnail to represent them in the inventory".

### Eight species: five from Tenebris, three new

Tenebris's water roster (`tenebris-core/src/fauna.rs:145-305`, pinned at
`ef98651`, the same pin as `docs/source-migration.md`) has five catchable fish
and a turtle. The turtle stays behind: Tenebris itself marks it as not on the
hook (`is_fish`, `fauna.rs:283-293`), and a creature you cannot catch belongs
in a change about wildlife rather than one about fishing.

| Species | From | Water | Window, °C | Swims at, m | Found | Strength | Hook window, s | Tenebris speed, m/s |
| --- | --- | --- | --- | --- | --- | ---: | ---: | ---: |
| Minnow | Tenebris `Fish` | rivers, shallows | 10 to 30 | 0.3-1.8 | schools of 24-36 | 1 | 0.90 | 1.6 |
| Silverfin | new | shallows, shelf | −1.8 to 17 | 0.4-2.5 | schools of 18-30 | 1 | 0.90 | - |
| Banded perch | new | rivers | −1.8 to 24 | 1.0-4.0 | schools of 8-14 | 2 | 0.78 | - |
| Ray | Tenebris `FlatFish` | shallows, shelf | 16 to 32 | on the bed, 1.5-6.0 | alone or in pairs | 2 | 0.78 | 1.0 |
| Eel | Tenebris `LongFish` | rivers, shallows | 6 to 28 | on the bed, 1.0-5.0 | alone or in pairs | 3 | 0.67 | 1.9 |
| Reef fish | Tenebris `LargeFish` | shallows | 23 to 32 | 0.8-3.0 | schools of 6-12 | 5 | 0.43 | 1.2 |
| Deepback | new | shelf, deep | −1.5 to 9 | 2.5-6.0 | schools of 4-7 | 4 | 0.55 | - |
| Sea serpent | Tenebris `SerpentFish` | deep | −1.8 to 32 | 3.5-6.0 | alone, one per region | 4 | 0.55 | 2.2 |

- **Taken from Tenebris as is:** the strengths (`fish_strength`), the hook
  window formula, and each species' 16×16 icon. Each Tenebris speed is the top
  of our speed band. Schooling, depth bands and habitats are new, because
  Tenebris has none of the three.
- **The three new species are placeholders** from the first mockup. Their
  names and looks are the owner's to change.
- **The depths fit the shelf a player can fish from.** The mockup's bed falls
  to 6 m within 16 m of the shore. Real open ocean is deeper, and the deep
  species' bands can widen once boats carry a rod (owner question 6).
- **Bottom dwellers** (ray, eel) carry `bed: true`. Their goal sits 0.3 m off
  the bed, and a vertical term holds them there. The same boid rules apply
  with one or two fish per school: separation and alignment do nothing,
  cohesion keeps a pair together, and the goal walk does the rest.
- **Behaviour the table does not show:** the sea serpent's bite factor is 0.12
  against the minnow's 0.6, and there is at most one serpent in a region at
  a time. That is Tenebris's "rarest on the line" expressed as a spawn cap
  rather than a table weight.

### Where each species spawns: a water class and a temperature window

The owner asked for spawning planned from the climate: "based on climate
conditions, plan out where the fish should spawn". The rule has two parts, and
both are read off the world rather than authored per place.

1. **The water class**, from the terrain generator:

   | Class | What it is |
   | --- | --- |
   | river | water that is there only because of the river cut. The same generator run with the cut switched off calls it land |
   | shallows | sea 6 m deep or less: what a cast from the shore reaches |
   | shelf | sea 6 to 40 m deep |
   | deep | sea over 40 m deep; the deepest point on the planet is 125 m |

   Both depth limits live in `fauna.ron`.

2. **The water temperature now**, `Atmosphere::sample(dir).temperature`. The
   atmosphere documents this as the sea surface over the sea, and as the
   ground at sea level on land, which is the water standing in a river
   channel. Water under −1.8 °C is frozen, and nothing spawns in it.

A species lists the classes it lives in and a temperature window. A school
can spawn at a candidate cell only when the class matches and the temperature
is inside the window. **Because the spawner reads the temperature now,
ranges follow the climate by construction**: when the seasons or the weather
move the water, the fish move with it, and no code says so. This replaces the
three fixed zones the previous revision proposed. Tropical, temperate and
cold survive only as words in the field guide.

### The ranges, measured: the Fish Range Atlas

<https://claude.ai/artifact/DwruVvwvPRm71T5tnuNN9s> (`docs/wiki/fish-ranges/`). It has:
- one greyscale range map per species, in the style of a field guide: land
  grey, water paler with depth, frozen water hatched, the species' range in
  red, and the part of its range it holds only some of the year in orange;
- overview maps of the species count, the water temperature and the water
  classes.

The maps are drawn by a **measurement instrument**, which changes nothing in
the game:
- `crates/pbd-core/examples/fish_ranges.rs` samples the terrain at 1440 × 720
  points and runs the world's own atmosphere and ocean forward, recording
  each cell's daily-mean water temperature.
- `tools/fish_ranges.py` applies the rules above and draws the maps.

The rules live in that script only for this proposal. When the change is
built they move to `fauna.ron`, and the script reads them from there.

**The water, measured** (share of all water, by true area):

| River | Shallows | Shelf | Deep | Frozen on day one |
| ---: | ---: | ---: | ---: | ---: |
| 6.1% | 7.6% | 47.3% | 39.0% | 9.6% |

**The ranges on day one of a new world**, the climate the world is built to
have (share of all water):

| Species | Range | Latitudes |
| --- | ---: | --- |
| Minnow | 6.8% | 39°S to 40°N |
| Silverfin | 23.7% | 55°S to 56°N |
| Banded perch | 4.5% | 52°S to 57°N |
| Ray | 25.9% | 31°S to 31°N |
| Eel | 8.2% | 44°S to 44°N |
| Reef fish | 0.8% | 17°S to 16°N |
| Deepback | 15.4% | 54°S to 55°N |
| Sea serpent | 37.3% | 55°S to 55°N |

What the maps show:
- **The coasts are layered by temperature.** Reef fish hold the tropical
  shallows. Minnow and eel take the warm and temperate coasts and river
  mouths. Silverfin takes the cool coasts up to the ice.
- **The shelf is split between ray and silverfin.** Ray holds warm shelf and
  silverfin cool shelf; their windows overlap between 16 and 17 °C.
- **Deepback and serpent** hold the deep.
- **Perch alone has the cold rivers.**

**Every point of open water has at least one species on day one.** The first
draft of the windows left 4.6% of all water empty: shelf between 14 and 18 °C,
where silverfin stopped and ray had not started. Cold rivers and water just
above freezing were also empty. The windows were widened to overlap, and a
test pins the coverage (below).

Rivers exist only in lowland under 40 m (`river_max_elev_m`), so a river
range is a lacework along low coasts rather than lines across continents.
That is the terrain's rule, and the maps show it rather than hide it.

### A finding the fish depend on: the sea does not hold its climate

The instrument was built to measure seasons. What it measured instead is a
slide. Mean sea-surface temperature, from a new world, under the shipped
settings:

| Day | 10 | 20 | 30 | 40 | 60 | 80 | 100 | 130 | 160 | 200 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Mean sea surface, °C | 7.7 | 2.0 | −2.8 | −6.5 | −13.0 | −16.9 | −19.0 | −22.6 | −23.3 | −23.5 |

It levels off at about −23.5 °C from day 130. In the second 100 days **every
point of water on the planet is below −1.8 °C for the whole period**,
tropics included, so the spawn rule puts no fish anywhere: every species'
year-2 map is empty.

A world starts at the latitude climatology `28 − 45·sin²(lat)`, which averages
+13 °C, and drains from there. At 48 minutes a day, 20 days is 16 hours of
play, so **a player's own world freezes over its first few days of play**. The
spawn rule does exactly what it should with that: the maps for days 1-100 and
101-200 show the warm species' ranges retreating toward the equator and then
vanishing under ice.

The budget, from `atmosphere.ron`:
- Absorbed: `solar_wm2 · cosZ · (1 − 0.6 · cover) · (1 − albedo)`.
- Outgoing: `203 + 2.09 T − 40 · cover`.

Two terms are out of balance:

1. **`solar_wm2` is 1000**, a clear-sky surface value. Budyko's constants
   (203, 2.09) are calibrated against a top-of-atmosphere sun of about 1360.
2. **The clouds are a net cooler about ten times Earth's.** In the tropics,
   at 0.9 or more cover, the cloud albedo reflects about 230 W/m² at the
   equator while cloud greenhouse returns a flat 40 W/m². That is about
   −190 W/m² net, against about −20 W/m² for Earth's clouds.

The first term alone is not the fix, and that is measured, not argued. The
same world with `solar_wm2: 1360` (the instrument takes the `ATMOSPHERE`
override the climate report takes) slides a little more slowly:

| Day | 10 | 20 | 30 | 70 | 100 | 140 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Mean sea surface, °C | 8.7 | 3.8 | 0.0 | −9.4 | −11.3 | −13.7 |

It was still falling when the run was stopped at day 140. Both runs' logs are
committed beside the atlas.

**This belongs to `atmospheric-circulation`, not to fishing, and nothing here
changes it.** The fish plan assumes a sea that holds roughly the climate it
starts with. Until the atmosphere does, the day-one maps are the plan, and the
drift maps are the reason to fix the atmosphere first. It is owner question 8.

### The roster, in data

`assets/config/fauna.ron`, per body. It holds the numbers the simulation reads
and the field-guide text, side by side, for one species at a time:

```ron
(water: (shallows_max_m: 6.0, shelf_max_m: 40.0, freezes_c: -1.8),
 bodies: { "pale-blue-dot": (species: [
    (id: "minnow", name: "Minnow", from: Tenebris("Fish"),
     water: [River, Shallows], temp_c: (10.0, 30.0),
     length_m: 0.14, speed_mps: (1.0, 2.6),
     depth_m: (0.3, 1.8), bed: false, school: (24, 36), max_schools: 2,
     sense_m: 8.0, bite: 0.6, pull: 0.4, strength: 1,
     colour: (0.47, 0.59, 0.71), shape: (0.5, 0.5, 1.0), icon: "fish/minnow",
     guide: (
       entry: "The first fish most anglers land. Minnows crowd the shallows...",
       tip: "Cast anywhere near the shore. A school finds the lure in seconds.",
     )),
    // ... silverfin, perch, ray, eel, reef, deepback, serpent
])})
```

- `from` records provenance per species. The entry shows it as a "From
  Tenebris" chip, and the provenance test checks it against the copied icons
  (section 9).
- **The roster is append-only**, because a saved fish is its index in it
  (section 8). A test pins the order.

### The field guide: one entry per species

The owner asked for a wiki entry for each fish. The entry is the species' own
record, drawn in two places from one source:

1. **In the game: the field guide.**
   - **Opening it:** J opens it, and so does clicking a fish in the slots
     while the pointer is free. The mockup's world panel also lists "In this
     water", one thumbnail per species present, and each opens its entry.
   - **The list:** every species on the body, with its icon and your catch
     count.
   - **The entry:**
     - the icon at 96 px, sampled nearest-point, so the 16×16 art stays crisp;
     - the name, a provenance chip, the waters it lives in, and a bottom
       dweller chip where it applies;
     - the entry text;
     - its numbers: depth, how it is found, length, strength as five pips,
       hook window, speed;
     - your own record: how many you have caught and your best length;
     - a one-line angler's tip.
   - **Where the numbers come from:** every one is read from the same record
     the school and the hook read. The hook window shown is
     `hook_window(strength)`, the same function that times the bite. **An
     entry therefore cannot disagree with the fish.**
2. **Outside the game: `docs/wiki/fish.md`**, generated from `fauna.ron` by
   `tools/gen_fish_wiki.py`. It gives one section per species with its icon, and
   a test fails when the committed page differs from what the generator writes.
   That is the same "shipped artifact equals its source" check the RON files
   have. Tenebris's wiki (`public/wiki.html`) has no creatures at all, so this
   page is new rather than ported.

**The catch record** is new durable state: per species, a count and a best
length.
- It rides the `slots` line a catch already writes (section 8), as
  `catch {species} {cm} s0..s9`, so the fish and its record reach the disk in
  one line.
- A fish's length is drawn from a deterministic hash of the school seed and
  the fish's index, scaled to 85 to 125% of the species' length, so the same
  fish always measures the same.

**Tests:**
- every species on every body has a non-empty entry and tip, and an icon in
  the manifest;
- no species id appears on two bodies;
- the lifeless bodies' rosters are empty;
- every species lists at least one water class and a window inside
  −1.8 to 40 °C;
- on day one of the reference world, every point of open water is inside at
  least one species' range (the atlas measured 0.000% uncovered);
- the generated wiki page equals the committed one.

The last three are new. The rest extend the rule Tenebris's CLAUDE.md set
after its own roster leak.

## 8. What is saved, and when

The save log (`saves/format.rs`) has two line kinds today:

| Line | Fields |
| --- | --- |
| Edit | `cell layer material s0..s9` |
| Kit | `kit version s0..s9` |

A slot is `-`, `b{material},{n}` or `t0,{n}`. The parser takes any head that
is not `kit` to be a cell number, so a new line head must be added to
`parse_line` before any save carries one.

New content:

| Event | Line, written that frame | Why |
| --- | --- | --- |
| A catch | `catch {species} {cm} s0..s9` | The slots changed without an edit, and the field guide's record grew. One line carries both. |
| A tool change | `hand {tool} {owned bits}` | The equipped tool is player state that must survive a reload. |
| A tree felled | `fell {cell} s0..s9` | The world changed, and the wood went into the slots, in one line. |

- **Slot codes** gain `f{species},{n}`. Species is its index in the body's
  roster, and the roster is append-only for that reason, like `KIT_GRANTS`.
- **Tool codes** map `t0` to Pickaxe, keeping the existing code, and add `t1`
  Shovel, `t2` Axe and `t3` Rod. A tool is never in the ten slots today, so
  these appear only if a future chest holds one.
- **The kit** is versioned by `KIT_GRANTS.len()`. Granting the tools is one
  appended grant. `KIT_GRANTS` holds `(Material, u16)` today and becomes
  `(Grant, u16)`, where a grant is an item or a tool. A saved world therefore
  gets the four tools, with the rod in hand, once, exactly as it got its
  torches.
- **The field guide's record** is rebuilt on load by folding the `catch`
  lines: a count and a best length per species. There is no second store to
  keep in step.
- **Old saves** load unchanged: they have no `catch`, `hand` or `fell` lines.
  A save written by this change is not readable by an older build. That is
  the same direction every save change here has taken, and the world ID
  records the format.

## 9. Icons

Every new item gets a thumbnail in the same change, and the field guide draws
the same thumbnail as the slot.
- **Five fish reuse Tenebris's own icons.** `fish_minnow.png`,
  `fish_reef.png`, `fish_eel.png`, `fish_ray.png` and `fish_serpent.png` from
  `tenebris-rs/assets/textures/items/` at `ef98651` are 16×16 RGBA, 131 to 154
  bytes each. They are copied byte for byte into `assets/items/fish/` and never
  referenced from `.reference/`, per the provenance rule. A `PROVENANCE.md`
  beside them names the source path and commit, and a test hashes each copy
  against the hash recorded there. Tenebris's Rust tree is MIT
  (`docs/source-migration.md`). The mockup embeds these exact PNGs.
- **Three new fish** (silverfin, banded perch, deepback) get 16×16 PNGs drawn
  on the same rules as Tenebris's generator (`tools/gen-fish-sprites.py`):
  head to the right, a dark, mid and light ramp per species, a 1 px eye, and a
  transparent ground.
- **Tools** get four 16×16 PNGs for the picker and the slot.
- **Wood** needs nothing new: it is a block, so its icon is its terrain tile.

The new PNGs are generated by `tools/gen_item_icons.py` (stdlib and zlib only)
from the same pixel grids the mockup draws. The tool writes the new icons only:
the copied ones are sources, not outputs. Nearest-point sampling, per the art
rule.

The existing test that every material has a thumbnail gains siblings: every
tool and every species on every roster has an icon in the manifest, and the
five copied fish match their recorded hashes.

## 10. What the mockup found

- **A sagging line runs under the water** unless each point is clamped to the
  sea surface.
- **Schools that only wander rarely meet the lure.** With a 7 to 12 m sense
  and random goals, a bobber could sit for 40 s of simulated time without a
  visit. The fix is the lure scent (section 4): a school within 3 × `sense`
  turns toward the lure.
- **A picker and a meter drawn with `display: grid`** override the `hidden`
  attribute. This one is web-specific. The Bevy equivalent is using
  `Display::None` rather than `Visibility::Hidden` for a closed picker, so it
  is not laid out or picked.
- **A single large mouse delta** (a pointer re-grab) threw the look to the
  sky. Clamping each event's delta fixed it. The walker already reads Bevy's
  accumulated motion, but the fix is noted here for the vehicle seat and the
  picker.

## 11. Proof, when it is built

- **Core tests:**
  - boids stay between the bed and the surface, and inside the band;
  - a school scented at 3 × `sense` reaches the lure;
  - the hook window formula for strengths 1 to 5, with the 0.25 s floor;
  - tension snaps at 1, and a patient reel always lands the fish;
  - `secs` for every material and tool;
  - `tree_at` agrees with the WGSL over 10,000 IDs on a headless adapter;
  - equipment round-trips through the save format;
  - `fauna.ron` equals the code defaults;
  - the roster tests of section 7: entries, icons, disjoint bodies, water classes and
    windows, day-one coverage
    covered, and the generated wiki page equals the committed one;
  - the copied Tenebris icons match their recorded hashes.
- **App tests:**
  - a tap of G boards and a hold opens the picker;
  - the wheel moves the highlight and not the slots while the picker is open;
  - a catch writes a `catch` line in the same update, and the field guide's
    count and best length come back after a reload;
  - J and a click on a caught fish open that species' entry;
  - a felled tree stays felled after the tier is rebuilt and after a reload;
  - a lifeless body spawns no school.
- **Captures:** the picker open over the HUD, a cast bobber with a school
  under it, a felled tree, and the field guide open on a species.
- **Limitation:** the feel of the bite and reel, and whether the schools read
  as schools on screen, can only be confirmed by the owner in game.
