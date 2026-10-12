# Proposal: grass and ground clutter, Tenebris's scatter on a GPU pipeline

## Why

**The owner asked: "can we port over grass and ground clutter?"** Between the
trees this planet's ground is a flat painted hexagon. Tenebris's is not: it
carries twelve kinds of decorative scatter, and the grass is the one that
changes what standing on the surface feels like, because it is the only thing in
the frame at the scale of the avatar's own feet.

The rules are all in the reference and all measured below. The geometry is not
portable, and that is the whole of the design problem: Tenebris bakes its
scatter into a CPU chunk mesh, and this project has no chunk mesh.

## What Tenebris's scatter actually is

Measured off `tenebris-rs`: `tenebris-client/src/hex_mesher.rs::build_scatter`
(the whole system, 500 lines), `tenebris-core/src/scatter.rs` (the knobs) and
`assets/config/scatter.yaml` (the shipped values).

**Twelve kinds, each gated on the surface block and the biome**, every one
placed by a deterministic per-tile hash so a chunk rebuilt after mining regrows
the identical scatter:

| kind | gate | chance | size | geometry |
| --- | --- | ---: | ---: | --- |
| grass tuft | any grass top | 0.80 | 0.55 m tall, 0.085 m half-width | 11-18 blades, each 2-3 stacked tapering quads |
| flower | any grass top | 0.12 (x1.6 fields, x0.4 jungle/swamp) | 0.32 m stem | a blade plus a 4-triangle diamond head |
| rock | bare ground, x0.3 on grass | 0.10 | 0.16 m radius | one squat hex prism |
| bush | grass or dirt, x3 jungle | 0.05 | 0.34 m radius | two leafy hex prisms |
| cactus | desert sand | 0.16 | 2.2 m tall, 0.22 m radius | a trunk prism plus two arm prisms |
| fern | jungle grass | 0.45 | 0.75 m fronds | 5-7 blades arched outward from one point |
| reed | swamp grass or dirt | 0.50 | 1.10 m stalk | 2-3 stalks, each capped by a dark prism |
| dead shrub | desert sand, tundra snow | 0.14 | 0.38 m twigs | 4-6 leaning blades from one point |
| kelp | sea floor | 0.20 | 0.65 m per segment, up to 6 | crossed ribbons zig-zagging up, depth-capped |
| seaweed | sea floor | 0.35 | 0.30 m | 3-4 short blades |
| vine | swamp canopy | 0.80 | 2.2 m strands | strands hanging off a leaf top |
| (sway) | every plant | - | 0.07 m tip travel | a shortened-normal channel the VS reads |

Four facts about it are worth naming, because each one is a decision this port
has to take a position on:

- **Placement is one hash family.** Seven salts (`SALT_KIND`, `SALT_COUNT`,
  `SALT_POS_A/B`, `SALT_SIZE`, `SALT_ANGLE`, `SALT_VARIANT`) over
  `hash2(tile_idx, salt)`, so every decision about a tile is a pure function of
  its index. Nothing is stored and nothing is random.
- **Nothing needs new art.** A blade samples the atlas column of *the ground
  tile it grows on*, through `tile_uv_slice`, which crops a narrow vertical
  slice chosen by a per-blade hash. The slightly-off shades between neighbouring
  blades are what make a sward read as a sward, and they come free from art the
  project already ships.
- **Blades are two-sided and lit by the ground.** `push_quad_2side` emits both
  windings and every vertex takes the *surface up* as its normal, so a blade
  shades exactly as the cap it stands on and never pops against it.
- **Two kinds are gatherable and the rest are pure decoration.** A `scatter_taken`
  bitset in `tenebris-core/src/world.rs` holds one bit per tile; picking the
  flint pebble or the leafy bush sets it, and `tile_has_rock` / `tile_has_bush`
  re-derive the same roll so the gate and the mesh agree. Grass, flowers and
  everything else cannot be taken.

## Why the geometry does not port, and what does

Tenebris's scatter is **baked CPU vertices in the chunk's opaque mesh**: the
comment over `build_scatter` says so outright - thousands of pieces ride the
existing per-chunk bind-and-draw, "instanced in effect without a per-instance
pipeline". That works because its planet is 300 m across and 163,842 tiles are
one chunked mesh that is rebuilt only when a block changes.

This project has no chunk mesh at all. A cell is a persistent GPU record, a
compute pass compacts the visible ones into three lists, and three indirect
draws build every triangle in the vertex shader from the record's own corner
rays. So the port is the **rules** - the hashes, the densities, the sizes, the
gates, the per-biome roster - onto a **fourth indirect draw**, exactly as the
tree port took `block_hex_width` and `shrink_corner` and left the mesher behind.

That is not a loss. The same structure that forced it also pays for it: clutter
needs no CPU work, no memory, and no rebuild, and a blade is a handful of
vertices computed from a hash the shader already has.

## What it costs, measured

The cell is the same size on both sides - 2.833 m flat-to-flat, 6.951 m^2 - so
Tenebris's densities and sizes carry over **unchanged**, which is the payoff of
the gold-standard rule. What does not carry over is the reach, because its
planet is smaller than this one's finest LOD band.

**Vertex cost.** A blade of two segments is 2 quads = 12 vertices one-sided.
At Tenebris's 12-18 blades that is 144-216 vertices per grassy cell, which is
comparable to the whole 198-vertex tree. Grass is on most of the ground rather
than 5% of it, so the reach is what decides the bill:

| clutter radius | grassy cells | at 144 verts/cell | against the frame |
| ---: | ---: | ---: | --- |
| 40 m | 564 | **0.08 M** | a quarter of today's tree draw |
| 60 m | 1,269 | **0.18 M** | half of today's tree draw |
| 80 m | 2,256 | **0.33 M** | today's tree draw |
| 300 m (the whole fine band) | 31,729 | **4.57 M** | 23% of the terrain draw |

For scale, the shipped frame today is about **19.6 M** terrain vertices (the
visible hemisphere at 60 each) and **0.32 M** foliage vertices. So grass out to
60 m costs *less than the trees do*, and grass over the whole fine band costs
fifteen times the trees.

**And past 60 m it is not grass any more.** At the 1440x900 capture with a 60
degree vertical field of view, one pixel subtends 0.00128 m per metre of range,
so a 0.17 m-wide blade covers:

| range | 5 m | 10 m | 20 m | 40 m | 60 m | 80 m | 120 m | 300 m |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| blade width, pixels | 26.5 | 13.2 | 6.6 | 3.3 | **2.2** | 1.7 | 1.1 | 0.4 |

Under about two pixels a blade is a shimmering sub-pixel line that costs a
vertex and adds aliasing, not grass. The cost table and the pixel table point
at the same answer from two directions, which is the strongest kind of number
this repository keeps: **the clutter tier is short because grass stops being
visible, and it is cheap because it is short.**

**What grows it.** 39.3% of the sphere carries a grass material and 49.6% is
ocean, so grass covers **78% of the land** - it is the common case, not a
sprinkle. Measured on the shipped generator: DryGrass 37.5%, JungleGrass 1.8%,
swamp grass under 0.1%.

## The decisions this needs before any of it is built

1. **How far the clutter reaches.** Recommend **60 m, faded to nothing over the
   last 15**, for the two measurements above. A hard cutoff is a visible line
   where the grass stops, and the fade is a blade height scaled to zero over the
   last stretch, which the vertex shader can do per instance for free.
2. **Which kinds land first.** Recommend the **four that cover the whole
   planet** - grass, flowers, rocks, bushes - and holding the eight biome
   specials (cactus, fern, reed, dead shrub, kelp, seaweed, vine) for a second
   pass. Each special is a different vertex budget and they would each need
   their own indirect draw or a shared upper bound that the common case pays
   for. Fields is 37.8% of the sphere and jungle 1.8%: the first cut should buy
   the 37.8%.
3. **Sway, or still grass.** Tenebris's sway follows its weather sim's **wind
   direction** and rolls a gust wave downwind at a 40 m wavelength. This
   project's weather field carries **rain amount only** (`params.weather.x`).
   Sway therefore needs a wind direction in that field - and the clouds should
   read the same one, so it is one fact in one place rather than two. Recommend
   shipping the geometry first with sway held, then wind as its own change.
4. **Where the knobs live.** `assets/config/scatter.ron`, mirroring the
   reference's `scatter.yaml` field for field, through the existing loader with
   its partial-override and validation behaviour. No scatter number becomes a
   Rust `const` or a shader literal.
5. **Gathering.** Tenebris's rocks and bushes are pickable, gated on a per-tile
   bitset. This project has no gathering. Recommend porting the *geometry* only
   and leaving the bit out entirely, rather than adding state nothing reads: the
   eligibility rule is a pure function of the cell, so the bitset can be added
   beside it the day gathering exists.

## What success looks like

Standing in a pasture, the ground between the trees is a sward of individual
blades a player can see move past their feet, with the odd flower, pebble and
bush in it, and the frame cost is under the tree draw's. Beyond 60 m the ground
reads as the painted hexagons it does today, and there is no line where the
change happens.
