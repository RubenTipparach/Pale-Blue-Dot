# Proposal: rivers are there the moment the ground is

## Why

The owner, 2026-11-17: "we also need a better way to cache river data as part
of world data, so it loads more immediate, instead of having to generate water
lol".

Before designing a cache, this change measured where a river's water actually
comes from and what it costs. The answer is not what the request assumes.

**River water is not flooded in by the flow rule over time.** It is derived,
in three parts:

1. **The drainage map**, once per world per launch
   (`crates/pbd-core/src/rivers.rs:147`, `RiverMap::build`, reached lazily
   through `rivers::map` at `rivers.rs:384` by the first altitude anything
   asks for). It samples the unrivered ground at every cell of a 720 x 360
   lattice (259,200 altitudes), then runs a priority flood, catchments and
   levels. One thread.
2. **Per column and per altitude, analytically.** Every altitude the
   generator gives passes through `rivers::carve` (`rivers.rs:465`, called
   from `planet_gen::surface_altitude`, `planet_gen.rs:471`), every column
   stands its sources from `rivers::water` (`rivers.rs:524`, called from
   `column::generate_solid`, `column.rs:512`), and every far record asks
   `rivers::far_surface` (`rivers.rs:584`, through `planet_lod.rs:264`).
3. **The rule's settle at the tier's build**: `ColumnTier::settle_rivers`
   (`crates/pbd-app/src/planet_column.rs:389`, called from `build` at
   `planet_column.rs:1426`) runs `pbd_core::liquid`'s rule over the tier's
   above-sea water until nothing moves, so the sheets between reaches and the
   falls are what the rule keeps. Derived, never saved.

All three run before the set they belong to lands: on the loading thread
before `READY` for the first set (`loading.rs:279`, `planet.rs:410`), and on
the fine-set task for every rebuild (`planet_lod.rs:1624`). **The player
never sees a river fill in**; they wait for the set it is part of. The
flowing-water mesh is then made on the frame the set lands
(`water_flow.rs:1149`, `rebuild_water`).

**Measured** in a Claude Code cloud container (4 cores, no GPU; CPU work on
this machine, `--profile fast`, never a frame time; instrument
`planet::column::river_cost`):

| Part | Cost here |
| --- | ---: |
| Drainage map, once a world (one thread) | **465-511 ms** |
| of which the lattice's 259,200 altitudes | 372 ms |
| The rule's settle at a tier's build | **6.2-9.4 ms** (12-45 ticks, 2 sweeps) |
| Re-settling an already settled tier | 5.7-9.4 ms, changes nothing |
| River terms in the tier's columns (with against without) | 24-44 ms of 65-257 ms |
| River terms in the fine set's record altitudes (one thread) | 72-203 ms of 161-378 ms |
| Whole fine set, four threads | 2,479-3,434 ms |
| Flowing-water mesh on the landing frame | 6.4-13.6 ms |

So the rule's settle, the thing a "settled water cache" would save, is under
ten milliseconds a tier and under half a percent of a rebuild. The cost the
owner can feel is the drainage map: half a second on every launch of every
version 8 world, on one core, inside the loading screen, and paid again by
any process that opens the world.

## What changes

- **The drainage map's ground is sampled on every core** (design 2). The
  259,200 altitudes are independent and pure; they are split into chunks on
  scoped threads and joined in order, so the map is the one-thread map,
  node for node, and a test holds it to that. Built here.
- **A drainage-map cache file** (design 3), so a world opened a second time
  reads its map instead of building it: keyed by the generator version, the
  seed and the river settings, checked by a digest, safe to delete, never a
  player record. Designed, not built: where it lives is an open decision.
- **No cache of the rule's settled water** (design 4): measured at 6-9 ms a
  tier, a per-region cache would cost more in invalidation (edits, the rim,
  generator versions) than it saves, and its result near the tier's rim is
  not a function of the cell alone. The settle stays as it is.
- **Cheaper river lookups** (design 5), later: the river terms are a fifth to
  a half of every altitude near a river.

## Follow-up, 2026-11-17: the river that "doesn't load"

The owner, on one river at 90 m elevation: "river still not cached, meaning
it takes a while for it to load", then "it doesnt even load in the
distance" (a dry terraced channel past a fall, even standing still). The
measurement above was right about the data and wrong about the picture: the
water was computed and uploaded, and then culled. The visibility pass tested
every water sheet's bounds on the sea's sphere, and a far river's sheet
stands at its own surface, up to hundreds of metres higher; from a river's
bank everything past the column tier's edge failed the frustum test
(design 7). One cause for both reports: walking, the tier's edge is as
little as 20-50 m ahead and the river past it was dry until the next tier
landed.

- **The listing tests a far river's sheet at its own surface** (design 7).
  Built, with a GPU test on the shipped shader.
- **`--walk-river`**, a recording instrument: the walker walks a whole river
  from its source to the sea on the bank, logging its progress and, each
  second, how far ahead the river is drawn under the old rule and the new
  (design 8).

## Impact

- `pbd_core::rivers::RiverMap::build`: the ground sampled on scoped threads.
- No change to any world's ground, water, save or look: the map is the same
  bytes, so every column, record and river is the same.
- `openspec/specs/world/water`: the built requirement ("A world's drainage
  map does not depend on the core count"), with its test, in this commit.
- Loading: the map's share of the world's preparation falls with the core
  count.
