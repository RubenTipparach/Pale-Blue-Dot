# Design: river-cache

## 1. The measurement (an instrument, said here before it was written)

**What was measured, and why.** The owner asks for river data to be cached
"so it loads more immediate, instead of having to generate water". Before a
cache is designed, the write-up needs to know which part of a river's water is
expensive: the once-a-world drainage map, the river terms in every altitude
and column, or the flow rule's settle at a tier's build. So:

- `planet::column::river_cost::what_a_world_s_rivers_cost_to_build`
  (`crates/pbd-app/src/planet_column.rs`, ignored, run with
  `cargo test -p pbd-app --profile fast --lib river_cost -- --ignored --nocapture --test-threads 1`);
- `BuildPhases` / `LAST_BUILD`, a test-only (`#[cfg(test)]`) phase timer in
  `planet_column::build` and `settle_rivers`: columns, flood, settle (with its
  queued cells, ticks, sweeps and changed cells) and relight. Compiled out of
  the game; it changes nothing the game does.

It builds the drainage map fresh on one thread and on every core, then for
four river anchors with a fall (as `river_tier_tests` picks them) and a dry
inland anchor: the whole fine set, every fine record's altitude with the
river terms and without, `rivers::far_surface` over the records, the tier's
columns with the river terms and without, the tier's phases (best of three),
`rivers::water` over the tier, a second settle over the settled tier, and the
flowing-water mesh made when the set lands.

**The machine.** A Claude Code cloud container: four cores, no GPU (lavapipe),
`--profile fast` (release optimisation without LTO). These are CPU timings on
this machine, not the game's frame times, and not the owner's desktop. No
`perf_suite.py` run: the container has no GPU.

### Where river water comes from

| Stage | Where | When | Saved? |
| --- | --- | --- | --- |
| Drainage map: lattice altitudes, priority flood, catchment, levels | `pbd_core::rivers::RiverMap::build` (`rivers.rs:147`), through `rivers::map` (`rivers.rs:384`) | Once per world per process, lazily, on the first altitude asked | No |
| River ground (valley, levee, bed, plunge pool) | `rivers::carve` (`rivers.rs:465`) in `planet_gen::surface_altitude` (`planet_gen.rs:471`) | Every altitude | No |
| River sources in a column | `rivers::water` (`rivers.rs:524`) in `column::generate_solid` (`column.rs:512`) | Every column built | No |
| Far river sheet | `rivers::far_surface` (`rivers.rs:584`) via `planet_lod::river_word` (`planet_lod.rs:264`) | Every fine record | No |
| Flowing sheets and falls between reaches | `ColumnTier::settle_rivers` (`planet_column.rs:389`) in `build` (`planet_column.rs:1426`), the rule of `pbd_core::liquid` | Every tier build | No (derived, like the sea's flood) |
| Flowing-water mesh | `water_flow::rebuild_water` (`water_flow.rs:1149`) | The frame a set lands | No |

Water a player poured, and water the rule moved from an edit, are edits and
are saved (`world/water`, "Only edits start water moving"). Nothing of the
generated river is.

### What a load or a walk does

- **A load:** `loading::prepare` (`loading.rs:209`) reads or surveys the
  sites, lays the towns, then builds the planet (`planet.rs:410`) whose first
  fine set builds the column tier, settled rivers and all. The drainage map is
  built inside whichever of these first asks for an altitude, on the loading
  thread, under the loading screen. Nothing of a river is computed after
  `READY` that was not already in the first set: the player waits for the
  rivers, never watches them fill.
- **Walking:** every 40 m (`REGEN_DISTANCE_M`) a new fine set is built on the
  `AsyncComputeTaskPool` (`planet_lod.rs:1624`). Its columns carry the river
  sources, its settle runs the rule, and it lands whole. The drainage map is
  not rebuilt. The rule's queue then holds only edited water
  (`queue_edited_water`).
- **What is recomputed every rebuild:** the river terms in every altitude
  and column, and the settle. **What is computed once a world:** the
  drainage map.

### The numbers (this container, CPU)

Before (the build at `66780be5` plus the instrument):

| | river 0 | river 1 | river 2 | river 3 | dry |
| --- | ---: | ---: | ---: | ---: | ---: |
| Tier columns | 3,913 | 3,095 | 3,133 | 3,104 | 3,105 |
| Tier build, ms | 322.7 | 199.9 | 162.6 | 324.2 | 184.5 |
| of which columns | 231.4 | 129.1 | 102.0 | 210.8 | 114.6 |
| of which the sea's flood | 1.2 | 1.1 | 1.0 | 2.5 | 1.0 |
| **of which the river settle** | **8.6** | **6.8** | **6.5** | **7.2** | **6.2** |
| of which relight | 30.2 | 25.4 | 23.5 | 24.8 | 20.8 |
| Settle: cells queued / ticks / sweeps | 176 / 34 / 2 | 68 / 15 / 2 | 113 / 17 / 2 | 184 / 45 / 2 | 69 / 12 / 2 |
| Settle: cells it changed | 1,029 | 397 | 350 | 747 | 198 |
| Settle again over the settled tier, ms | 9.4 | 5.9 | 5.9 | 5.9 | 5.7 |
| Tier columns with / without river terms, ms | 257 / 213 | 117 / 82 | 94 / 65 | 214 / 218 | 116 / 92 |
| Fine records' altitudes with / without, ms (1 thread) | 281 / 209 | 294 / 166 | 339 / 161 | 260 / 170 | 378 / 175 |
| Whole fine set, ms (4 threads) | 2,479 | 2,676 | 3,186 | 2,547 | 3,434 |
| Flowing-water mesh at landing, ms | 10.5 | 7.5 | 6.6 | 13.6 | 6.4 |

Drainage map, fresh, one thread: **465 and 511 ms** in two sittings, of which
**372 ms** is the lattice's 259,200 altitudes and the rest the flood.

**Reading it.**

- The flow rule's settle costs 6-9 ms a tier and settles in 12-45 ticks.
  About 6 ms of it is the sweeps' check, which costs the same over a tier
  that is already settled. A cache of the settled water would save at most
  those few milliseconds of a 2.5-3.4 s rebuild.
- The river terms in the altitudes are a real share of the fine set: up to
  about 200 ms of one-thread CPU over a set's records, plus 24-44 ms in the
  tier's columns, before `far_surface`. They are per point (the meander is a
  noise field read at the point), so they are not a stored level per cell.
- The drainage map is the one cost a load pays for rivers alone: about half
  a second, on one core, every launch. It is the clearest target.

## 2. The drainage map's ground on every core (built)

`RiverMap::build` samples `surface_altitude` of the unrivered ground at every
lattice cell, one after another. Each sample is a pure function of the cell's
centre and the config. `RiverMap::build_on(cfg, settings, threads)` splits the
lattice into `threads` contiguous chunks on `std::thread::scope`, each writes
its own slice, and the flood after them reads the joined array exactly as
before. `build` calls it with `available_parallelism`. The priority flood,
catchment and levels stay on one thread: they are about 100-140 ms and
ordered by construction.

The map is the same, node for node: the altitudes are the same floats in the
same places, and everything after is unchanged code. Test
`rivers::tests::the_map_built_on_every_core_is_the_one_thread_map` compares a
one-thread map with three- and seven-thread maps by `PartialEq` over every
node.

This is a pure CPU cut: no generator version, no save, no picture changes
(`Performance improvements need no permission`). The lazy build happens on
the loading thread or a fine-set task, both of which may use the other cores;
the fine-set task's own workers wait on the `OnceLock` while it builds, and
the sampling threads never touch the lock (they read the unrivered config).

## 3. A drainage-map cache file (designed, not built)

So a world opened a second time reads its map rather than building it.

- **What:** the map's nodes (down, river, level, drop, half width), about
  259,200 x 17 bytes (4.4 MB raw), in a small binary file with a header:
  a magic, a format version, the generator version, the seed, the
  `RiverSettings` (every field, as bits) and a digest of the nodes.
- **Key:** generator version + seed + settings + format version, in the file
  name and the header. A world of another version or seed never reads it.
  An old version's map is never edited; a new version is a new key.
- **Safe to delete:** a missing, short, unreadable or mismatched file is
  rebuilt (design 2) and rewritten. Reading never fails a load.
- **Never player data:** the map is what the seed and the rules decide
  (CLAUDE.md, "The player is not the only author of change"), so it is not a
  record, takes no part in the durable transaction path and is not in the
  world's identity. It is written atomically (temporary file, rename) after
  the map is built, off the frame, and its failure is a log line.
- **Edits never touch it:** the map is built from the generator's own
  ground, before any edit, and no edit changes it (a dug riverbank changes
  the column, not the drainage). So edits do not invalidate it.
- **Loading screen:** the map is read or built before `READY` exactly where
  it is built today, so nothing appears after `READY`.
- **Proof it is equal:** a test builds a map, writes it, reads it back and
  compares by `PartialEq`; a test that a header with another seed, version or
  setting is refused and rebuilt.

**Open decision (where it lives):** the slot's directory
(`<saves>/<slot>/cache/rivers.bin`, deleted with the world, one per slot) or
a shared per-user cache keyed by seed and version (one per seed, shared by
every slot of that seed, and by new worlds of a seed already seen). The
recommendation is the slot's directory: it is where a player looks, and
deleting a world deletes its cache. Not built until that is settled, because
it is a new file in the save folder.

With design 2 in place the file saves the remainder (the flood's ~100-140 ms
and the parallel sampling) on a second open; whether that is worth a new file
in the save folder is part of the decision.

## 4. Why not cache the rule's settled water

The request names this one, and it was considered first.

- **It is cheap:** 6-9 ms a tier (section 1). A cache read and the check that
  it is still valid would cost a large share of that.
- **It is not a function of a cell.** The tier's rim is neutral to the rule
  (`ColumnTier::water_at`), so a sheet within a few cells of the rim settles
  against the rim of that tier; the same cell settles differently in a tier
  anchored elsewhere. A per-cell cache would have to settle over fixed
  regions with margins, which is a second, different answer near every tier
  edge, and the picture would change there.
- **Edits invalidate it in ways that are hard to bound:** a dug bank lets a
  sheet run out, and the change can reach seven cells along the rule's range
  and further down a fall. The rule already does this from the edit
  (`water_after_edit`).
- **Saving it would be saving derived data.** The settle is what the rule
  keeps of generated water. Only the rule's changes from edits are the
  water's own edits, and those are saved already.

So the settle stays, derived at every build. If a later measurement on the
owner's desktop found it mattered, the cheap cut is in the sweeps (their
check is most of the 6 ms), not a cache.

## 5. Cheaper river lookups (later, measured first)

Near a river an altitude costs several times one without (the tier's
per-column altitude: 12-21 ms with rivers against 3-5 ms without over ~3,100
columns). `RiverMap::at_widened` scans a 5 x 5 window of lattice cells,
computing each river cell's centre and its downstream centre with four
trigonometric calls each, and reads the meander noise; `carve` in a channel
adds six unrivered altitudes for the rim (`top_here`) and two `top_at` calls
for a plunge pool, each six more. Candidates, all keeping every value:

- store each node's centre (and its downstream centre) in the map, as the
  map is built, instead of recomputing them per call;
- skip the window when no river cell is within it (a per-cell flag made with
  the map);
- reuse `top_here`'s six rim samples between `carve` and `rivers::water` in
  the same column.

Each is measured with this instrument before and after, with a test that
the altitudes are bit-for-bit the same over a tier. Not built here.

## 6. Results after design 2

The same instrument, the before and after test binaries interleaved
(before, after, before, after) in one sitting. The container was shared with
other sessions running the game (load average 10-13 on four cores, two other
processes at about 130% each), so few cores were free and the parallel gain
is mostly masked; these are not the desktop's numbers.

| Drainage map, fresh | Run 1 | Run 2 | Run 3 | Run 4 |
| --- | ---: | ---: | ---: | ---: |
| Before (one thread) | 430 | 507 | 419 | 586 |
| After, `build_on(.., 1)` | 522 | 448 | 391 | 463 |
| After, `build` (4 threads) | 393 | 368 | 360 | 426 |

(ms.) After on four threads is 7-27% under the same run's one-thread build
in every run. Everything else in the instrument is unchanged within the
spread between repeat runs (the tier's settle 5.8-9.2 ms, the fine set
1.9-3.3 s), as it should be: only the map's build changed. The lattice's
ground alone is 318-402 ms of one-thread work, so on an idle machine with
`n` free cores the map's build is about `flood + ground / n`, roughly
100-140 ms plus 50 ms on the owner's eight-core desktop. That is an
estimate, not a measurement: task 3 asks for the desktop's `READY` line.

`rivers::far_surface` over a fine set's records, measured after only: 30-78
ms of one-thread CPU (design 5's candidates apply to it too).

## 7. The dry channel: far river sheets culled at the sea's level (2026-11-17)

**The report.** The owner, twice on one river (map 20.150S 6.618W and
20.885S 6.393W, 90-92 m elevation, generator version 10): "river still not
cached, meaning it takes a while for it to load", with the debug line "70 m
from anchor | 3117 columns | rebuilding 1.9 s"; then, standing on the bank
looking downstream, "it doesnt even load in the distance": full near the
player, and past a small fall a few tens of metres down, a dry terraced
channel to the horizon, even after standing there.

**What design 1 promised, and what it missed.** Sections 1-6 measured the
CPU: river water is derived with the set it belongs to, the settle is 6-9
ms, and "the player never sees a river fill in". That is true of the data and
not of the picture. Nothing a cache would save is what the owner saw.

**Reproduced** at `--walk --at 20.885 -6.393` (the owner's spot to the
decimetre: 91.6 m elevation) on `origin/main` (`2de3c03a`), `--profile
fast`, M4 Max. A measurement instrument, not committed: the far river's
sheet (`kind == 4` in `water.wgsl`) drawn flat magenta after its depth test.

- The generator's data is whole. An ignored probe over a 600 m square at 2
  m (`rivers::far_surface` at the finest cell width against
  `rivers::floods`): 3,317 points the column water floods, 2,728 with a far
  surface, the rest within a cell of the bank. The river runs unbroken
  1,593 m from its source (19.250 -6.750, 140 m upstream) to the sea, with
  12 falls.
- Seen from 80 m up, pitched down 25 degrees: the column tier's water near
  the anchor, then **no water at all** for the next ~90-150 m of channel,
  then the far sheet. From the bank, eye 1.6 m up and level, the sheet is
  never drawn closer than about 250 m, which on this river is behind the
  banks: the whole downstream channel reads dry.

**The cause.** `compact_visible` in `planet_visibility.wgsl` lists a cell's
water sheet when its sheet's bounding sphere is in the frustum, and it puts
that sphere **on the sea's sphere**, `direction * (R - depth_offset)`, for
every sheet. That was right for the sea. A far river's sheet is drawn at its
own surface (`water.wgsl`'s vertex: `R + max(surface_m, ground + 0.05)`),
here 88-90 m above the sea. A cell 100 m ahead of an eye on that bank has its
tested point 92 m under the eye, 43 degrees down; the camera's vertical
half-angle is smaller, so it is culled while the river on it is in plain
view. The cull distance is `height over the sea / tan(half the vertical
field of view)` and grows with the river's height: every inland river
loses its far sheet from the tier's edge out to a distance set by the camera's
pitch, and a mountain river all of it in view.

It is one cause for both reports:

- **"Doesn't load in the distance":** past the column tier (90 m from its
  anchor, dithered over its last 25 m) only the far sheet can draw a river.
  Culled, the channel is dry.
- **"Takes a while to load":** walking, the tier is rebuilt 40 m from its
  anchor and a rebuild takes 1.9-3.4 s (section 1), so the tier's edge is as
  little as 20-50 m ahead. Everything past it was dry until the next tier
  landed and brought its column water: the river "loaded" in steps behind the
  player.

**The fix (built).** The listing tests a far river's sheet at the radius the
vertex draws it at: `R + max(surface, ground + 0.05)` where `spare.w` holds a
river, the sea's sphere otherwise. The sea's listing is unchanged. No data,
generator, save or shading changes; a sheet that was culled while in view is
now drawn, and nothing else is listed. Cost: one branch and a bitcast in a
pass that already reads the record; the extra sheets are the river cells in
view, a few hundred instances of an 18-vertex fan.

Test: `planet_visibility_tests::actual_gpu_lists_a_far_river_s_sheet_at_its_own_surface`
runs the shipped shader on a real GPU with a river cell 90 m over the sea,
100 m ahead of an eye on its bank looking level: listed; behind the eye: not.

**Measured after** (same spot, same flags): see section 9.

**What this does not change.** The tier's own water and the far sheet still
meet on a dither across the tier's last 25 m that moves with the anchor when a
set lands; the two draw the same surface, so the swap is not a gap. The far
sheet is a still sheet at each cell's surface: falls between reaches are
steps in it, not flowing faces, past the tier.

### Why not the cache (sections 3-4) for this

The water the owner saw missing was computed and uploaded; it was culled.
A drainage-map cache would shave the ~0.4 s the map costs inside the
loading screen and would not have drawn a single missing cell. Section 3
stays designed and unbuilt, waiting on its decision; it is not this bug.

## 8. The river walk (a recording instrument)

The owner asked for the fix to be shown "with an uploaded recording of
walking along a whole river". `--walk-river` walks the walker down a whole
river through the player's own inputs (W, sprint, jump, the view turned as
a mouse turns it), in the shape of `--coast-walk`:

- **The river:** the drainage map's course through the lattice cell nearest
  the start (`--at`): up to its source (each step up the widest feeder) and
  down to the sea. Its length is logged at the start.
- **The path:** the course's cell centres, offset a bank's width to one side
  (`half width + bank + 4 m`), so the walker keeps to the bank, a few metres
  off the water, and the river is in view ahead and to one side. It steers
  toward the point 24 m further along the path; stalls are hopped, and
  after 1.5 s stuck it turns 40 degrees away from the river and walks on, as
  the coast walk does. In water it swims at the surface.
- **The log:** `RIVER_WALK_START <length> m`, a `RIVER_WALK` line every
  second (seconds, metres walked, the share of the river's length passed,
  place), and `RIVER_WALK_DONE` at the mouth or when the river's last cell
  is behind it; then it quits.
- **The water ahead, per second (a model, not pixels):** along the river's
  course ahead of the walker, out to 600 m, the first point where the river
  is in the camera's view but would not be drawn, under each listing rule:
  the old one (the sheet tested at the sea's sphere) and the new (at its own
  surface); inside the column tier the tier's water counts as drawn.
  `RIVER_AHEAD old <m> new <m>` each second. It mirrors the shader's frustum
  test on the CPU with the camera's own view and projection; the video is
  the check that the pixels agree.

The recording: the same build, run once with `origin/main`'s
`planet_visibility.wgsl` beside it (the shader is read at launch from the
`assets` folder by the executable) and once with this change's, so the two
walks differ in that one file and nothing else. Real time, `obs-record`,
one OBS.
