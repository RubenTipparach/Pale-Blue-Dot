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
  (`half width + 9 m`, either bank: `--walk-river left|right`), so the
  walker keeps to the bank, a few metres off the water. It moves toward the
  point 24 m further along that path, sprinting, with the keys a player
  strafes with (W, A, S, D relative to the view), while the view turns (no
  faster than 70 degrees a second, as a mouse does) to look 50 m down the
  river's own line, so the river and its distance are what is filmed. A
  rise the auto step will not take is hopped; after 1.5 s stuck the walker
  turns its way 40 degrees away from the river; stuck twice in one place it
  takes the river's channel for 10 s, swimming. A plain launch's title
  screen is not opened in front of it, as for a capture.
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

## 9. Measured after (2026-11-17, M4 Max, `--profile fast`)

**At the owner's spot** (`--at 20.885 -6.393`, captures in
`docs/screenshots/river-cache/`, `origin/main`'s visibility shader against
this change's, the same executable otherwise):

- From 80 m up, pitched down 25 degrees (`owner-spot-air-*`): before, the
  water stops at the fall at the tier's edge and the channel is dry for the
  next ~100-150 m, then the far sheet resumes; after, the river is unbroken
  from the tier to the sea.
- From 12 m up looking downstream (`owner-spot-h12-*`,
  `owner-spot-past-the-fall-*`): before, past the fall a dry red bed; after,
  the river's sheet runs on past it.
- From the bank at eye height looking the owner's way (`owner-spot-bank-*`):
  the two are the same. From there the downstream reach turns behind the
  bank and drops under the eye; nothing of it is in view to draw. The owner's
  frame was taken facing along a reach that was in view.
- On arrival: the far sheet's record (`spare.w`) is uploaded with the
  ground's own record in the set that lands, and the listing that draws the
  ground is the one that now draws the sheet, so a river past the tier comes
  with its ground in the same frame; the tier's own water was already in the
  set (section 1). That is by construction and by the walk below (each
  second, every set): no arrival was filmed frame by frame.

**Along the whole river** (`--walk-river right --at 19.25 -6.75`, the
owner's river, 1,593 m from its source at level 88 to the sea, 35 lattice
cells, 12 falls; walked source to mouth in 261-269 s in each recording):
`RIVER_AHEAD`, the drawn water ahead each second, is a CPU model of the
listing test with the camera's own view, so each run logs both rules:

| | old rule (sea's sphere) | new rule (own surface) |
| --- | ---: | ---: |
| Median metres of river drawn ahead (of up to 600 measured) | 56 | 600 |
| Tenth percentile | 28-36 | 179-201 (the river's end in reach) |
| Seconds the whole measured stretch was drawn | 3-5% | 100% |

The old rule's median is the distance to the column tier's edge: past it,
nothing. The new rule's shortfalls from 600 m are all the river ending
(the mouth within 600 m), never a culled sheet.

**Frame time.** One branch and a bitcast per cell in the visibility pass,
and more water instances listed: the river cells actually in view (at most
a few hundred 18-vertex fans). The perf suite's verdict is in section 10.

**The recording.** `river-walk-before-after-2026-11-17-900p60.mp4`: the
before walk, the after walk, then the two side by side matched by progress
along the river, captioned with the share of the river, the metres walked
and `RIVER_AHEAD` under that recording's rule. Real time, OBS, one instance;
the game rendered at the window's 2880 x 1800 and the video is half that.
OBS's cadence check: median 46 (before) and 56 (after) new frames a second
of 60, on a machine other sessions were also using (load average 6-30), so
the video shows what is drawn, not how smoothly. The two walks are the same
script from the same start but not the same frames: the walker's path
differs a little run to run.

## 10. Frame time: not measured

`tools/perf_suite.py --scenarios walk,far-side --repeats 2` (before:
`origin/main`'s assets on the same release exe; after: this change) refused
to start at 22:05: the screen had locked, and macOS throttles a locked
screen's window, so every run would be paced, not measured. No number is
claimed. What the change adds per frame: one branch and a bitcast for each
listed cell in `compact_visible`, and an 18-vertex instance for each far
river cell now in view that was culled before (the river's cells past the
tier, at most a few hundred). The owner's sitting on an unlocked screen
decides it.

## 11. The whole river cached (owner, 2026-11-18)

**The report.** The owner, on this change as merged: "critical issue, the
"water" cache was built to help cache riivers ... this is not the full river
sim data, this is only partuial, please cahche the fiull river sim". They're
right. Nothing of a river was cached. Design 2 only sampled the drainage map
on more cores. Design 3's file was never built. Every rebuild worked out
again every river term of every altitude, column and far record, and the
rule's settle. So did every launch.

**What "the full river sim" is here.** Every derived river value the game
computes, all pure functions of the world's generator version, seed and river
settings:

| Part | Computed by | Cached as |
| --- | --- | --- |
| The drainage map | `RiverMap::build` | the map's nodes, in the cache file |
| Which lattice cells have a river within reach | (new) built with the map | a mask on the map: a point with no river node in its 5 x 5 window answers "no river" without the scan |
| The river's ground (bed, valley, levee, plunge pool) | `rivers::carve` | a memo by the point (its `f32` bits) and its unrivered altitude |
| The water's top in a channel (columns' sources, `floods`) | `top_at` | a memo by the point |
| The far sheet over a far cell | `rivers::far_surface` | a memo by the point and the cell's width |
| The flowing water the rule keeps | `ColumnTier::settle_rivers` | the settled cells, by cell key, kept where the tier's rim could not reach them |

**Exact by construction.** Every memo's key is the call's input, bit for
bit, and its value is what the call returned. A hit is the same number the
call would have made, so no ground, water or save changes. Cells keep fixed
directions whatever the anchor (the topology is fixed), so a cell asked
about by one rebuild is the same key in the next. Tests: a tier's columns,
the fine set's records and the far words are the same with the cache cold,
warm, and read back from its file.

**The settled water.** The rule's result near a tier's rim depends on the
rim (design 4). So only cells more than `SETTLE_KEEP_CELLS` (12, past the
rule's seven-cell range) from the rim are kept: by cell key, the layers the
settle changed and to what. A later tier stamps the kept cells that are
inside its own margin before it settles, and the settle then runs as before.
It finds them already at rest, and settles only the cells nobody has kept
yet. A test builds a tier at one anchor, then a tier 40 m on twice, cold and
stamped, and holds their water equal away from the rim. An edit drops every
kept cell within the rule's reach of it, so a dug bank never meets stale
water.

**Filled ahead: the whole river, not only what was walked.** After `READY`,
a background thread finds the rivers within 600 m of the player. For each, it
fills the cache along the whole course, from source to sea. That covers every
finest cell and every fine level's cell within the river's corridor
(`Lattice::cells_in_band` round each course point). It runs at low priority
on spare cores. A rebuild near any part of that river then finds its terms
already worked out.

**The file.** It sits beside the worlds, at
`<saves>/cache/rivers-<seed>-<digest of the whole config>.bin`. That answers
design 3's open decision ("as part of world data", the owner's words of
2026-11-17), and one file serves every slot of the same seed.
- **Header:** a magic, the format, the generator version, the seed, the
  settings' bits and a digest. Then the map's nodes, then the memos.
- **Writing:** atomic (a temporary file, then a rename). Written off the
  frame every 30 s while the cache has grown, and on exit.
- **Reading:** a missing, short, mismatched or corrupt file is rebuilt, and
  never fails a load.
- **Not player data:** it's never a record and is never in the durable path.
  Deleting it costs one rebuild.

**Measured** with `river_cost` (design 1) extended to a cold, a warm and a
filed cache. The numbers are in section 12.

## 12. The river run (owner, 2026-11-18)

**The report.** "a bot was used to validate ... this bot often got stuck on
the bank/sides of rivers this made the bot useless for actualy running
through a river fast enough to analyze the caching", "when validating river
cache, make sure to bump the time of day to day", "Only need to record about
5 minutes of river runs, also MAKE SURE THE PALYER DOESNOT GET STUCK,
measure ground deltas properly and detect if the player's position didnt
movemcuh in 1 second".

**Why it stuck.** `--walk-river` decided "stuck" when it hadn't moved 1.5 m
from an anchor point. A walker hopping against a terraced bank, or sliding
down it, moves more than that and resets the anchor, while it makes no way
down the river. So the 1.5 s turn rarely fired. And the bank path, half a
width plus 9 m off the line, runs along the valley's own slope, a stair of
one-metre layers (`carve`), where most of the steps are.

**The fix (an instrument change).** The owner, after the first trial:
"bot should have specifc hex by hex naviation, I think this would avoid it
trying to take shortcuts that cause it to bump into sides", then "The bot
isn't even in the river. You should start out spawn in middle of a river.
You know the map already. Do not artificially move the bot."

- **It starts in the river.** `--walk-river` spawns the walker on the
  drainage map's river nearest `--at`, 15 m down its line from its source,
  in its water. It is placed once, at the spawn, like any `--at`. After that
  it moves only by the player's inputs. The walk waits until the walker
  stands there, because a world opening can hold it elsewhere for a few
  frames.
- **Hex by hex** (`river_walk::HexRoute`). A route is planned over the
  finest cells (the lattice's own, each with its column's height), from the
  walker's cell to the cell nearest the river's line 40 m on. It's at most
  60 m at a time, a few hundred cells, a couple of milliseconds.
  - A rise of one layer is walked, two are jumped at a cost of four cells,
    and more is never taken (from water, more than one and a half).
  - Water costs less than land, so the run keeps to the river.
  - The walker walks centre to centre, so it never cuts a corner into a
    bank. It plans again as a route runs out, or when it is 6 m off it, no
    more than twice a second.
- **Stuck is measured on the ground, over a second.** The walker keeps its
  last second of positions. It is stuck when its ground distance (the arc
  over the planet, height left out) is under 3 m on foot or 1.5 m swimming;
  sprinting covers 14 m a second. A hop or a slide is not motion.
- **Unsticking escalates.** First a hop. Then the cells it was walking into
  are kept out and a new route goes round them. Then it backs out 1.2 s and
  takes a new route. Each step is logged (`RIVER_WALK_STUCK`).
- **Day.** `--time 11`. **The frame graph** is on (`--frame-graph`).
- **Each second** logs "in the river" or "on the bank", and `RIVER_CACHE`:
  the share of river lookups the cache answered and what it holds.

**What still stopped it, and the fixes** (owner, after the first
recording: "Fix these bot isuses"). The first recording stalled 8 and 11
times, always at 0.0 m in a second and at the same spots in both runs. A
log of what the bot held and aimed at at each stall found four causes:

1. **The router read the reach's level, not the water standing there.**
   A reach's water is held lower wherever its banks are, so a stepped
   channel looked flat and the route ran into its steps. Now each cell is
   read from the live column tier (`river_walk::cell_heights`): its floor,
   and the top of the water in it, the rule's flowing water included (a
   fall's lip, a sheet between reaches). The generator's values are used
   only outside the tier.
2. **A swimmer steps up nothing.** It is never grounded
   (`walking::sweep`'s `can_step`), so any rise of the bed above its feet is
   a wall. The router's step rule is now the walker's
   (`river_walk::step_allowed`):
   - swimming, it goes on only where there is 2 m of water to float over,
     or where the bed doesn't rise;
   - out of the water, only onto ground within 1.5 m of the surface;
   - on foot, one layer stepped and two jumped.
   Leaving the water costs 8 m more, so the route stays in the river.
3. **"Swimming" came from the generator.** The generator says "river" over
   a dry stretch of channel bed. Now the bot asks the live column for water
   at its feet (`PlanetContact::stand`), as the physics does.
4. **On land a jump is a press, not a hold** (`state.jump |=
   just_pressed`). The bot held the key whenever it believed it was
   swimming, so on a dry bed it jumped once and never again. It now holds
   the key only in water and taps it on land, jumping whenever the next
   cell's live floor stands 0.6 m over its feet.

After the fixes the whole river, source to mouth, ran with no stall four
times running (150-170 s each). A swimmer that does stall skips the hop
(it already holds the jump) and takes a new route round the spot.

**A lag found in the trial, and fixed.** The first trial's frames spiked to
1.4 s. Its walk started before the walker was placed, so it read its
progress 657 m down the river and planned routes hundreds of metres long:
tens of thousands of cells, on the main thread, every frame it was off its
route. Hence the start wait, the 60 m cap and the half-second limit. It was
the instrument's cost, not the game's.

**The recording.** About five minutes of river runs, real time, one OBS,
day. A run along the owner's river, then rivers near it, with each second's
`RIVER_WALK` progress, the rebuild timings and the cache's hit rate in the
log. Uploaded to the Drive videos folder.

## 13. Measured (2026-11-18, the owner's desktop, RTX 3070, `--profile fast`)

**`river_cost` before the cache** (design 1's instrument): the fine set
972-1,508 ms whole. Its records' altitudes 260-365 ms with the river terms
against 164-220 ms without (one thread). The far sheets 33-78 ms. The
drainage map 140 ms on 8 threads (512 ms on one).

**The river run, A/B** (the same walk down the owner's river from its
source, about 75 s each, new worlds, one sitting, day):

| | Cache on | `PBD_RIVER_CACHE=off` |
| --- | ---: | ---: |
| Fine set rebuild, mean | **3.6 s** (16 sets) | 5.9 s (11 sets) |
| River lookups answered by the cache | 97% | - |
| Frame, median / p99 (past 35 s) | 15.3 / 24.6 ms | 14.9 / 27.4 ms |
| Frames over 50 ms | 6 | 9 |
| Stuck | 0 | 13 |

The rebuilds while walking are longer than `river_cost`'s one-shot set
because the game builds on its background threads beside the frame. The
cache takes about two-fifths off them, so the river past the tier lands
sooner. Frame time is the same within the spread.

**The recording** (`river-cache-runs-v2-2026-11-18-450p60.mp4`, 5 min
40 s, in the Drive validation folder; it replaces the first,
`river-cache-runs-2026-11-18-450p60.mp4`, whose bot still stalled). Two
runs down the owner's river from its source, day, the frame graph on, real
time, one OBS, 1440 x 900 window, video at half:

| | Run 1, cold (no file) | Run 2, warm (run 1's file) |
| --- | ---: | ---: |
| Whole river, source to mouth | 170.5 s | 169.8 s |
| Stalls | **0** | **0** |
| Seconds on the bank | 0 | 0 |
| Lookups from the cache | 86.7% | 97.5% |
| Fine set rebuild, mean | 4.1 s (35 sets) | 4.2 s (35 sets) |

The warm run's rebuilds are not faster here: with the fill running ahead
of the walker, the cold run's lookups are already 87% hits. The cache's gain
is the A/B above, against no cache at all.

**Frames.** In play, past `READY`, no frame of the bot's runs went over
210 ms, and those are fine-set landings, as before this change. Writing the
cache file runs on its own thread. The one long frame, about 2 s, is the
game quitting: the cache is written once more on exit, before the window
closes.

**What it holds.** After a run of the one river and its feeders, about 7.6
million values (5.0 M ground, 1.9-2.4 M water tops, 0.35 M far sheets,
3,500 settled cells). That's a file of about 150 MB beside the worlds. The
near-river mask keeps everything away from rivers out of it. Its size as a
player wanders is owed: a cap, or keeping only values a river changes, is
the next cut if it grows past what a save folder should hold.

**Not measured:** `tools/perf_suite.py` on the release build.

## 14. Far rivers drawn the way terrain is (owner, 2026-11-18; proposed)

**The report.** The owner, on the v1 recording's frames: "some examples of
partial river, not full water sim", and "I also dont like how stuff keeps
fading in, the terrain isnt fading in because its lod is much bigger, why
cant we do the same for rivers?"

**Why terrain doesn't fade and rivers do.**
- **Terrain is one thing near and far.** The column tier's ground and every
  fine record read the same `surface_m`, so when the tier lands nothing
  changes in the picture.
- **River water is two things:**
  - **Inside the column tier** (`reach_m` 90 m from its anchor): the water
    the columns hold, with the rule's settled flow, drawn as flowing faces.
  - **Past it:** a stand-in, a still sheet per far cell at the reach's level
    (`rivers::far_surface`, `water.wgsl` `kind == 4`).
- **They hand over by a distance dither** across the tier's last 25 m. That's
  the fading.
- **The stand-in differs from the real water in three ways:**
  1. It is drawn only within half a cell or the river's half width of the
     line, while the valley's ground stands under water further out
     (`rivers::floods`). Those cells draw their bed as a bed with no sheet
     over it: the dry patches in the owner's frames.
  2. It is still: a fall is a step in it, with no face.
  3. It never has the rule's flowing sheets.
- **The tier lags a fast walker.** A rebuild takes about 4 s while walking
  (section 13), and the anchor is where the walker was when it was asked for.
  At swimming or sprinting pace the anchor is 50-80 m behind (the owner's HUD:
  "73 m from anchor"). The dither band is then 15-20 m ahead of the eye,
  always in view.

**The plan.** The owner: "there should be no stand in sheets, I think thats
for orbital view, or when terrain is in LOD". So the real water reaches as
far as the ground is drawn at full detail, and a stand-in sheet only where
the ground itself is a level of detail.

1. **The water is loaded exactly as the terrain is** (owner: "the full
   final water geometry should be loading out to a reasonable distance that
   once the player crosses a threshold another job starts loading water
   again. exactly the same process as terrain").
   - **The job:** the fine-set job (`generate_fine_live_on`, on the async
     pool) builds the river's final water geometry with the ground. That
     means the water cells, their settled flow and the flowing-water mesh
     itself (today `water_flow::rebuild_water`, 6-14 ms on the frame the set
     lands).
   - **Its reach:** the finest band, 300 m, where the ground is drawn at full
     detail.
   - **When it runs:** the same anchor and the same threshold: the walker
     crosses `regen_m` (40 m) from the anchor and the next job starts.
   - **How it lands:** whole, in the same frame as its ground, with the same
     set's cross-fade. Nothing about the water is built on the frame.
   - **What stays the tier's:** solid columns, collision, lighting and the
     live rule's edits, within its 90 m. Inside it the tier's water is the
     band's water, cell for cell.
   - **Cached, so a job retrieves it rather than builds it** (owner: "this
     data should be cached somewhere so it is easily retreivable"):
     - **By fixed pieces, not by anchor.** Each piece is the water of one
       drainage-lattice cell (0.5 degrees, about 42 m): its water cells,
       their settled flow and their mesh. Pieces are fixed on the planet, so
       a job at any anchor assembles its 300 m from the pieces it needs and
       builds only the ones nobody has built yet.
     - **Where it lives:** in memory for the session, and in the river cache
       file beside the worlds (design 11), so a second visit or a second
       launch loads the river's water as fast as it reads.
     - **Built once, kept for good** (owner: "since we know water WILL NOT
       change on its own"): a river's generated water is settled once and
       stays settled. Nothing in the world moves it but an edit, so a built
       piece never expires and is never rebuilt for time or distance.
       **The one rebuild is the player's** (owner: "water rebuild is only when
       a player influences the flow of the river, which they totally can"):
       digging a bank, damming the channel, pouring or taking water. The live
       rule moves the water from the edit as it does today
       (`water_after_edit`, saved as the world's edits). The pieces the change
       reaches are dropped and rebuilt from the generated water plus the
       save's edits, then cached again, so the player's river is the one
       every later job and launch reads.
     - **So loading is a read, never the sim** (owner: "then that becomes
       cached and so loading becomes instant without having to do the water
       sim calcs"). Every piece a job needs is read: the generated river as
       first built, or the player's river as last rebuilt. The rule's settle
       runs only to build a piece nobody has built yet, and once per player
       change. The save still holds the edits, so if the cache file is
       deleted, each piece is built once from them and cached again. Once a
       river's pieces are built, every later job and every later launch only
       reads them.
     - **Kept honest:** a piece is keyed by the generator version, seed and
       settings like the rest of the cache. An edit drops every piece within
       the rule's reach of it (the edit's water is the save's, and the rule
       rebuilds the piece with it). A missing or stale piece is built and
       stored, never trusted.
     - **Filled ahead:** the background fill builds the pieces along the
       whole river nearest the walker, so a job's pieces are usually there
       before it asks for them.
2. **The settled flow comes from the cache.** The background fill (design 11)
   settles each river it fills along its whole course, in tier-sized regions
   down the river, keeping each region's interior (`SETTLE_KEEP_CELLS`), as a
   tier does. So the water out to 300 m is what a tier there would hold:
   flowing sheets, riffles and falls, never a still stand-in. A region not yet
   settled draws its sources until the fill reaches it, which is within a
   second or two of the river being found (`RIVER_FILL`, 0.1-1.8 s a river).
3. **Stand-in sheets only where the ground is LOD.** The still sheet
   (`far_surface`, `kind == 4`) stays on the coarser levels (L10 to L8) and
   from orbit, where the ground is also an approximation. It's drawn over
   every cell the river floods (`floods`), so a far valley has no dry bed.
4. **One handover, the ground's own.** The river's full water and the
   stand-in hand over at the finest band's edge, through the same L11-to-L10
   cross-fade the ground uses, in the same pixels. The ground and its river
   change together, far out where the ground's own change is already accepted.
   The tier's 25 m water dither goes: inside the band, tier water and band
   water are the same water, and the tier's cells simply draw the tier's.
5. **The tier leads a moving walker.** A rebuild is anchored where the walker
   will be when it lands (its velocity times the last rebuild's time, at most
   the rebuild distance ahead), so collision and the live rule are centred on
   the walker rather than 70 m behind.

**What a player would see differently.** Out to 300 m the river is the
real water (full width, falls, flowing sheets). No band of fading water is in
front of the player; past 300 m the river changes with the ground, at the
ground's own fade. That's the owner's own request.

**Cost, to measure before and after** (`river-cache` perf study, section 15):
- the band's water records: one cache lookup per flooded finest cell, out to
  300 m (a river's corridor is a few thousand cells);
- the flowing-water mesh over those cells: more faces in the water pass,
  measured per pass with the F3 breakdown's spans;
- the background settle along each river, on its own thread.

## 15. Performance study (2026-11-18, the owner's desktop, RTX 3070, `--profile fast`)

The owner: "could you also do a performance study as well?". Frame logs
without vsync, day, new worlds, nothing else running. Raw logs stayed local.

**The walk down the owner's river, cache on and off, interleaved** (five
whole-river walks):

| Run | Frame median / p99 | Main thread median | GPU median | Rebuild mean | Landings a minute | Walk |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| on, cold | 15.8 / 31.9 ms | 13.9 ms | 9.6 ms | 5.0 s | 11.7 | 166 s |
| off | 12.3 / 23.7 | 11.1 | 8.3 | 6.7 s | 8.3 | 133 s |
| on, warm | 15.9 / 28.7 | 14.0 | 9.9 | 4.5 s | 11.6 | 168 s |
| off | 13.3 / 51.0 | 11.9 | 8.2 | 9.0 s | 6.5 | 127 s |
| on, warm | 16.4 / 33.3 | 14.4 | 10.3 | 4.5 s | 11.9 | 171 s |

**The walks can't price the cache.** The bot's path depends on the live
tier. With the cache on, rebuilds land sooner, the live tier is under the
walker more often, and it swims the channel. With it off it runs more of the
bed outside the tier: 127-133 s against 166-171 s. Swimming, underwater and
more landings each have their own cost. So:

**Standing still at the owner's spot, on and off interleaved** (60 s, last
50 s):

| | on | off | on | off |
| --- | ---: | ---: | ---: | ---: |
| Main thread median | 10.8 | 12.7 | 13.2 | 13.1 ms |
| Frame median | 13.9 | 14.7 | 15.1 | 15.1 ms |

No per-frame cost: the difference is inside the spread between runs. Read-write
locks in place of mutexes on the memo's shards changed nothing measurable
(a walk: 14.3 ms on, 11.7 ms off, the same as before), and are kept since
every warm access is a read.

**The rebuild job** (`river_cost`, a set and then the next set 40 m on,
eight threads, two sittings):

| Next set 40 m on | river 0 | river 1 | river 2 | river 3 | dry |
| --- | ---: | ---: | ---: | ---: | ---: |
| cache on | 926-1,036 | 768-769 | 812-852 | 926-936 | 878-925 ms |
| cache off | 960-969 | 989-1,007 | 1,171-1,235 | 1,034-1,141 | 1,412-1,420 ms |

The cache makes the next rebuild 3-38% faster, most where a river is near
but not under the set. In play a rebuild gets two of the eight cores
(`background_threads`, a measured choice that keeps the frame smooth), so it
takes 4.5-5.0 s with the cache against 6.7-9.0 s without.

**What the cache costs.**
- **Its size is the problem.** After a day of runs on one river and its
  feeders, the file is **342 MB, 16.3 million values** (8.1 M ground, 7.6 M
  water tops, 0.5 M far sheets). The near-river mask lets every point within
  a 5 x 5 lattice window (about 210 m) of a river be memoised. Most of them
  are points where the river changes nothing.
- **Saving it** copies every value under the shards' locks and writes the
  file: 1.5-2.0 s on its own thread every 30 s, and about 2 s on the frame
  the game quits.
- **The fill**: 0.1-1.8 s of one core per river found.

**What this means for design 14.** A point memo is the wrong unit for a
whole planet's rivers. Design 14's pieces (the final water of one lattice
cell, its cells, flow and mesh) replace it. They hold only cells the river
floods, at most a few thousand per piece. They are built once and kept, read
whole, and saved as they are made rather than by copying everything. The
point memo shrinks to what pieces don't cover, or goes.

**Other costs on the frame, measured in passing.**
- The flowing-water mesh is built on the frame a set lands: 6-14 ms
  (design 1). Design 14 moves it into the job.
- The terrain's GPU cost is as the play-budget study left it (median 8-10
  ms here); the water passes weren't priced separately in this sitting.

## 16. Design 14, built (2026-11-18, the owner's desktop, `--profile fast`)

The owner: "build it".

**What was built.**
- **The river band** (`river_band::build`, called from the fine-set job after
  the column tier).
  - Every finest cell within a river's banks' reach (`rivers::near`, 6 m)
    out to the finest band (300 m), less the column tier's deep cells. It's
    built over `column::build_with`, the tier's own build over any chosen
    cells, with its own settle, meshed by `water_flow::liquid_mesh`.
  - It lands with the set (`FineSet::river`) and is drawn after the tier's
    water in the same flowing-water pass.
- **Cached by cell.** A cell's water (its still vertices) is kept by cell key
  once the cell was at least `SETTLE_KEEP_CELLS + 1` (13) cells inside the
  tier it was built in. A job reads every cached cell and builds only the
  missing ones, grown by 14 rings so each is that deep.
  - In memory, and in `<saves>/cache/river-band-<seed>-g<version>.bin`,
    written with the river cache (every 30 s and on exit, atomic), read at
    start.
  - **An edit** within 40 m of a river drops the cached cells within 40 m
    (`FineSet::write_layer`), so the next job builds them with the edit.
- **The column tier draws only its deep interior** (13 cells in from its
  rim, `liquid_mesh_from`); the band draws the ring outside it, so neither
  draws water its rim has shaped.
- **No stand-in at full detail.** The still sheet (`kind == 4`) is dropped
  at the finest level, and the tier's 25 m water dither is gone. The flowing
  water fades out across the finest band's ring like its ground. On the
  coarser levels the still sheet now covers every cell the river floods
  (`rivers::flood_top`).
- **The water past the tier is real to the walker too.** Where the tier has
  no column, the eye's water test (`eye_water_at`) and the walker's contact
  (`PlanetContact::stand`) read the band's water tops (`RiverBand::tops`).
  - A held-set test (`PBD_HOLD_SET=<n>`, an instrument that stops new sets)
    found the walker walking a river's bed under the band's water with the
    eye treated as dry: a dry channel on screen.
- **The tier leads and keeps up.** A set is anchored where the walker will
  be when it lands (`led_anchor`: velocity times the last build's time, up
  to half the tier's reach ahead). A set is also requested whenever, at the
  walker's speed, it would reach within 15 m of the tier's edge before the
  next lands.
- **The point memo**: capped (2 M values each for the ground and the water
  tops, 0.5 M for far sheets and settled cells), and only values a river
  changes are written. Its file went from 342 MB to 37-62 MB. Stale
  temporary files from a stopped write are cleared after each write.
- **The arrival banner**: a world opened while the last one's banner is up
  takes it down. The owner saw two names drawn over each other.

**Tests.**
- `a_band_read_from_the_cache_is_the_band_built_fresh`: a set 40 m on reads
  most of its cells, and its water equals the same set built with the cache
  empty, vertex for vertex. It failed at a 2-cell keep (7,788 against 7,698
  vertices) and passes at 13.
- `the_cache_file_reads_back_or_is_refused`: only river-changed values are
  written.
- The water-flow tests pass with the rim and keep rules.

**Measured** (the whole river from its source, day, warm caches):

| | Before design 14 (section 15) | After |
| --- | ---: | ---: |
| Real water drawn to | the tier, 90 m from its anchor | 300 m |
| Stand-in sheets within 300 m | from the tier's edge out | none once the first set at the river lands |
| Seconds off the tier (of the walk) | not logged | 10 of 167 (6 at the start, 4 single ones) |
| Rebuild, mean | 4.5-5.0 s | 6.4-6.6 s |
| Band per rebuild | - | 6,080 cells, 6,050 read, 30 built, 60 k vertices, 72 ms |
| Frame median / p99 | 15.8-16.4 / 29-33 ms | 15.1 / 25.1 ms |
| Main thread median | 13.9-14.4 ms | 13.3 ms |
| GPU median | 9.6-10.3 ms | 9.8 ms |
| Frames over 50 ms (walk) | 12-36 | 1 |
| River cache files | 342 MB | 62 MB + 7.7 MB band |

- A cold band builds about 1,100 cells a rebuild (300 ms on the job).
- Rebuilds take longer than before (6.4 s against 4.5-5.0 s). They're
  requested earlier and back to back now, and anchored ahead, so they cover
  more new ground. The band itself is 72 ms of that, warm.
- The first ~6 s after a world opens still show the river by its stand-in
  where the first set is still the old world's. That's the launch, not
  play: a new world opens the last one first.

**Not measured:** `tools/perf_suite.py` on the release build.

**After the test run found two more faults** (fixed):
- **The cache was keyed by seed, not by save.** Water built in one save with
  its edits could be read in another save of the same seed. Now a cell
  within 40 m of any edit is never read from the cache nor kept in it: the
  cache holds only the generated river.
- **The band's tier gathered caves round its own anchor**, so a cell's
  column could depend on which anchor built it. Band columns are built
  without caves; a river's valley is sealed from them anyway.

**Checks.**
- The generator's ground is unchanged: the version 8, 9 and 10 ground
  digests are the same on this branch as on `origin/main`.
- `planet_gen`'s seven digest tests fail on main too (pinned values out of
  date there).
- The app's twelve CRLF and GPU test failures (shaders, fish wiki, vehicle
  records, the lens) are in files this change doesn't touch.

**The recording** (`river-band-2026-11-18-450p60.mp4`, 2 min 56 s, Drive
validation folder): the whole river from its source, day, the frame graph
on. Real water to the horizon, the walker off the tier 4 of 175 seconds,
and two stalls at one spot, each freed.

