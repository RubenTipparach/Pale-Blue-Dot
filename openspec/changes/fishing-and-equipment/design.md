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
  constructed, so no save contains it (section 7).
- `Item::Tool` stays, because a future dropped tool or a chest is still an item.

**Input**, on foot only:

| Event | Result |
| --- | --- |
| G down | Note the time. Nothing else happens yet. |
| G held 0.18 s | The picker opens beside the tool slot, highlighting the held tool. |
| Wheel while the picker is open | Moves the highlight and wraps. The slots and the camera zoom do not see it. |
| G up with the picker open | Equips the highlight. A change of tool is a save (section 7). |
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
  `Breaking { cell, layer, since }` starts.
- **Progress.** It advances while the button stays down and the aim stays on
  the same layer. A ring round the reticle shows the progress. Letting go, or
  moving the aim, resets it.
- **Finishing.** At `secs(material, tool)` it takes the layer through the
  existing `apply_edit(Hands::Take, ...)`. The durable path, the GPU update and
  the slot give are all unchanged.

```text
secs = base(material) * (tool == right_tool(material) ? 1 : wrong_tool)
rod  -> never: "a rod digs nothing" on the aim line
```

| Class | Materials | Right tool | Base, s (tunable) |
| --- | --- | --- | ---: |
| soft | Grass, DryGrass, JungleGrass, Soil, Dirt, Sand, Snow | Shovel | 0.25 |
| stone | Stone | Pickaxe | 0.6 |
| rock | Rock | Pickaxe | 0.8 |
| ore | Ore | Pickaxe | 1.0 |
| wood | Wood (new, section 5) | Axe | 1.0 |
| placed | Torch | any tool but the rod | 0.1 |

`wrong_tool` is 4. `right_tool` and `base` live in `pbd_core` (a new
`tools.rs`), with the numbers in `assets/config/tools.ron` and a test that the
shipped file equals the code defaults. That is the same pattern as
`vehicles.ron`.

**Placing does not change.** Right click puts the selected slot's block
wherever it goes today, whatever tool is in hand. The one exception is while a
line is out: then right click reels in and places nothing.

The scripted `--dig` capture keeps working because it runs through the same
path with the shovel held.

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
  Recording it is section 7.

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
   as having no tree. The flag goes in the save log (section 7).
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
- Each species has a cap on resident schools (2, 1 and 1 in the proposal),
  and a school is dropped beyond 90 m.
- **Gate:** a body whose `fauna.ron` roster is empty spawns nothing, and the
  rod's aim line says "nothing lives in this water". That covers the airless,
  frozen and asteroid catalog worlds.

**The roster** (`assets/config/fauna.ron`), per body:

```ron
(bodies: { "pale-blue-dot": (species: [
    (id: "silverfin", name: "Silverfin", length_m: 0.22, speed_mps: (1.2, 3.2),
     depth_m: (0.4, 2.5), school: (18, 34), sense_m: 7.0, bite: 0.55,
     pull: 0.55, strength: 1, colour: (0.81, 0.89, 0.92), icon: "fish/silverfin"),
    (id: "banded-perch", ...strength: 2...), (id: "deepback", ...strength: 4...),
])})
```

- The three species are the mockup's placeholders. Their names and looks are
  the owner's to change.
- A test asserts that every species has an icon in the manifest, that no
  species id appears on two bodies, and that the lifeless bodies' rosters are
  empty. This is the rule Tenebris's CLAUDE.md set after its own leak.

## 7. What is saved, and when

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
| A catch | `slots s0..s9` | The slots changed without an edit. |
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
- **Old saves** load unchanged: they have no `slots`, `hand` or `fell` lines.
  A save written by this change is not readable by an older build. That is
  the same direction every save change here has taken, and the world ID
  records the format.

## 8. Icons

Every new item gets a thumbnail in the same change.
- **Fish** get three 16×16 PNGs.
- **Tools** get four 16×16 PNGs for the picker and the slot.
- **Wood** needs nothing new: it is a block, so its icon is its terrain tile.

The PNGs are committed under `assets/items/` with a manifest, and generated by
`tools/gen_item_icons.py` (stdlib and zlib only) from the same pixel grids the
mockup draws. Nearest-point sampling, per the art rule.

The existing test that every material has a thumbnail gains siblings: every
tool, and every species on every roster, has an icon in the manifest.

## 9. What the mockup found

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

## 10. Proof, when it is built

- **Core tests:**
  - boids stay between the bed and the surface, and inside the band;
  - a school scented at 3 × `sense` reaches the lure;
  - the hook window formula for strengths 1 to 5, with the 0.25 s floor;
  - tension snaps at 1, and a patient reel always lands the fish;
  - `secs` for every material and tool;
  - `tree_at` agrees with the WGSL over 10,000 IDs on a headless adapter;
  - equipment round-trips through the save format;
  - `fauna.ron` equals the code defaults.
- **App tests:**
  - a tap of G boards and a hold opens the picker;
  - the wheel moves the highlight and not the slots while the picker is open;
  - a catch writes a `slots` line in the same update;
  - a felled tree stays felled after the tier is rebuilt and after a reload;
  - a lifeless body spawns no school.
- **Captures:** the picker open over the HUD, a cast bobber with a school
  under it, and a felled tree.
- **Limitation:** the feel of the bite and reel, and whether the schools read
  as schools on screen, can only be confirmed by the owner in game.
