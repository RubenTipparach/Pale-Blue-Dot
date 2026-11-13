# Design: two-set-dissolve

## 1. What a landing does today, measured

### The mechanism

`upload_fine` (`planet.rs`) writes a landed set over the four fine regions of
the one cell buffer, then starts the landing's dissolve (`detail-fade`): for
`lod_fade_s` (1 s) each pixel shows the old partition or the new one by an
interleaved-gradient mask against the dissolve's progress. The old partition
is the anchor-relative band set of the set just replaced, round the fade
centre as it stood at the landing (`distance-lod-fade`).

Its cells are drawn from the records that are on the GPU, and those are now
the NEW set's. The new set's rings are widened to hold the old partition
(`cover_ring`), but the widening is capped at each level's own margin, the
rebuild distance and three of its tiles, because uncapped it fed back into
slower builds and wider rings landing by landing (`distance-lod-fade`, "A
runaway fit"). Where the records still do not hold the old partition,
`old_held` is false and the visibility pass lists the new partition's cell
unmarked, drawn in every pixel, while the old partition's cell there (which
is not among the records) is not drawn at all. That ring switches in one
frame. `fade_covered` names the first level that falls short in the log:
`LOD_FADE partial: level N ...; that ring switches at once`.

### The numbers

Fixed-step captures of the `fast` build on the M4 Max Mac, a fresh world each
run (`claude/perf-tuning` at `40040db`; the README in
`docs/screenshots/two-set-dissolve/` has the commands). The scenic flight was
flown twice, with the same landings failing in both.

| Run | Landings | `LOD_FADE partial` |
| --- | ---: | ---: |
| scenic flight, run 1 | 33 | 6 (sets 5, 27, 28, 29, 30, 31) |
| scenic flight, run 2 | 33 | 6 (sets 5, 27, 28, 29, 30, 31) |
| 1 km walk | 38 | 1 (set 2) |

The flight's failing landings, with the request that made each (height above
the ground, the live bands, how far the camera had come from the anchor):

| Set | Height | Moved | Bands, m | Log |
| ---: | ---: | ---: | --- | --- |
| 5 | 341 m | 148 m | 2318, 1114, 477, **0** | level 11: the old band reaches 353 m, the new records 0 m |
| 27 | 47 m | 116 m | 2400, 1196, 596, 295 | level 9: 1302 m against 1285 m |
| 28 | 44 m | 111 m | 2400, 1197, 596, 295 | level 9: 1307 m against 1285 m |
| 29 | 89 m | 85 m | 2400, 1189, 588, 284 | level 10: 681 m against 681 m |
| 30 | 93 m | 80 m | 2400, 1188, 587, 282 | level 11: 364 m against 361 m |
| 31 | 61 m | 79 m | 2400, 1194, 594, 292 | level 11: 362 m against 359 m |

Two causes:

- **A level the new set does not lay.** Climbing past 300 m the finest band's
  live radius is zero (`live_bands_m`), so the new set has no level 11 at all,
  and the old partition's level-11 disc, 353 m round the camera, is the ground
  directly below and ahead of it. No widening of rings can hold a level that
  is not laid.
- **The capped widening.** In the descent the sets take 2.5 to 2.9 s to build
  (wall clock, with the capture's slower frames), so the camera is 80 to 117 m
  from the anchor when the next is asked for, and the old fade centre stood up
  to its fit beyond that. The widening stops at the margin, a few metres to
  30 m short of the old band.

The walk's one is its first set after the spawn: the walker stands 443 m from
where the startup set was laid, so the old level 8 reaches 2,843 m from the
new anchor against records to 2,509 m. Every later walking landing moved
40 m with unchanged bands and faded whole (on foot the old and new partitions
are the same round a fade centre that follows the camera, so nearly every cell
is drawn once for both).

**The log is a lower bound.** `fade_covered` measures from the old anchor and
stops at the first level short; `old_held` in the shaders works round the old
fade centre, up to `fit_m` further on, and asks both levels across a band's
ring (`dissolve-coverage`). A landing can log nothing and still draw a ring
whole.

### Other one-frame switches at a landing (read off the code)

1. **A landing during a dissolve.** `lod_prev` is replaced, so the old
   partition's remaining pixels switch to the intermediate one at once. In
   these captures landings were 1.2 s or more apart in wall-clock time, but a
   capture runs slower than real time; at 150 m/s in real time a 0.3 s build
   and a 70 to 180 m margin land sets well inside a second of each other.
2. **An edit or a relight during a dissolve.** A version that does not move
   the partition sets `lod_prev = None`, ending the dissolve in one frame. A
   player digging just after a landing does this.
3. **The first set after an empty one.** `first` (the old bands are the
   base-only ones) skips the dissolve, so descending through 2.5 km the whole
   fine detail appears at once.
4. **Trees.** A tree both partitions draw is listed once, unmarked, and drawn
   with the new partition's band fades, so where the two partitions put it at
   different depths in a ring its dither density jumps at the landing.
5. **The clock.** The dissolve runs on `Instant`, the wall clock. In real time
   that is the game's time; in a fixed-step capture it is not, so a still or a
   frame sequence does not show what play shows.

## 2. The old set stays for its dissolve

### The layout

The cell buffer becomes the base, then **two halves** of four fine regions:

```text
slot:  [ base | half 0: L8 L9 L10 L11 | half 1: L8 L9 L10 L11 ]
         base_count  + 4 x FINE_CAPACITY   + 4 x FINE_CAPACITY
```

`PlanetGpu` keeps which half holds the current set (`half`), its live counts
(`counts`), and the old half's counts while a dissolve runs (`counts_prev`,
zero otherwise). The uniform carries them: `lod_offsets.z` the current half,
`lod_counts` its counts, and `lod_counts_prev` (the lane the records' rings
used, `records_in`) the old half's. `records_out` becomes a spare lane.

A landing that moves the partition writes the new set into the other half and
makes it current: `counts_prev = counts`. A version that does not move it (an
edit, the dusk relight) writes over the current half, as today, and leaves the
dissolve and the old half alone.

### The visibility pass

A slot is one of: **base**, **now** (the current half, below its count),
**before** (the old half, below `lod_counts_prev`) or dead. With `draws_now`
and `draws_before` the partition tests as today (`drawn`), `fading` whether a
dissolve runs, and `alike` that both draw the cell and `same_in_both` holds:

| Slot | Listed |
| --- | --- |
| base | as today: once unmarked when `alike` or no dissolve, else NEW where `draws_now`, OLD where `draws_before` |
| now | if `draws_now`: unmarked when no dissolve or `alike`, else NEW |
| before | if `draws_before` and not `alike`: OLD |

A fine cell both partitions draw alike is drawn once, from the current half;
its twin in the old half stands aside. The rule is evaluated on each twin's own
record, so the two copies must agree on it: a cell's position, corners and
owners are pure functions of its lattice address (`planet_lattice`), the same
floats in every set, and a test holds two sets laid round different anchors to
that, bit for bit.

Everything the old partition drew the frame before the landing is in the old
half, because that is where it was drawn from; so the old half of the
dissolve is the old picture, cell for cell, and the new half the new one. There
is nothing to check and no ring to draw whole: `old_held`, the `whole` rule,
`records_cosines`, `fade_covered` and `PlanetGpu::records` retire.

The column list and the clutter take only base and current-half slots, as they
take only the current partition today. The sea sheet is listed for the current
partition only, as today (section 6).

Trees take the same three rows with `has_nearby_foliage` for `drawn` and
`alike` as for the ground, so a tree the two partitions draw differently is
listed once per partition and dissolves (section 1, item 4).

### The surface pass

- **The dissolve** keeps a NEW pixel where the mask is under the progress and
  an OLD one elsewhere, nothing else: `old_held` goes from the fragment too.
- **The column tier.** A finest record carries its column's slot in the tier,
  and the tier's buffers (runs, light, materials, centres, bounce, lamp bake)
  are rewritten with every landing. A record in the old half names a slot in
  the tier that is gone, so an OLD instance is drawn with no column: its cap
  and walls from its own record (which already holds the old surface,
  reconciled), lit without the voxel field. That is the old half of a cell the
  two partitions draw differently inside the old tier's 90 m, for under a
  second, in flight low enough to have a level 11; on foot the partitions agree
  there and the cell is drawn once, from the current half, with its column.

### A dissolve is never cut short

A set that finishes building while the previous landing's dissolve runs
waits in `LodRefresh` until `lod_fade_s` has passed since that landing, then
is swapped in (the set and the contact together, as a landing always is). The
main world and the render world therefore never disagree about which set is
drawn, which the bounce and lamp bakes rely on (both are checked against the
main world's anchor). The log says `LOD_FADE held: set N waited X s for the
last dissolve`. `in_flight_s` reports a held set as in flight, so the dig rig,
which waits for a landing before it edits, waits for this one too; the race it
avoids (a set built before an edit undoing it, `edit-pipeline`) is no wider
than a build's.

On foot landings are about five seconds apart and nothing is held. In flight
the hold costs at most `lod_fade_s` of staleness on a set that is already one
build behind.

Should a moved version still reach the render world mid-dissolve (nothing
makes one today), it is drawn at once over the old half and the log says
`LOD_FADE cut short`.

### Which landings dissolve

Every landing that moves the partition, but:

- the set that ends startup, which replaces nothing and lands behind the
  loading screen;
- the first set after a generator switch, whose old half is another planet.

The first set after an empty one dissolves out of the base: the old half is
empty, the old partition is the base alone, and the base's cells are listed
OLD where the new partition draws fine ones. A set that empties the fine levels
(climbing past 2.5 km) dissolves into the base the same way.

### The clock

The dissolve's progress is `(now - landed) / lod_fade_s` on the planet clock
(`PlanetClock`, the game's elapsed time), as the lamp bake's half-second fade
already is, and the hold above measures on the same clock. In play it is the
wall clock; in a fixed-step capture it is the capture's own time, so a frame
sequence shows the dissolve the player would see.

## 3. Cost

- **Memory:** 262,144 more record slots of 192 bytes, 50.3 MB, and the per-slot
  lists the main view (28 bytes a slot) and the sun cascades (20) allocate,
  12.6 MB: about 63 MB of GPU memory.
- **The visibility dispatch** covers 688,130 slots instead of 425,986. The
  idle half returns on its first test (a division and a compare).
- **The dissolve's double draw** is the cells the two partitions draw
  differently, for `lod_fade_s`, as today; twins drawn alike are drawn once.
- **Builds** are unchanged: the records a set lays are the same bytes.

Not measured: this session has no perf suite. A change that can move frame
time runs `tools/perf_suite.py` before it is pushed; the owner runs it on the
desktop, old exe against new.

## 4. What it retires, and what it does not

Retired: `old_held` and `tile_angle` in both shaders and the test that held
the copies equal; `records_cosines`; `fade_covered` and its `LOD_FADE partial`
line; the `whole` listing rule; `PlanetGpu::records`.

Kept: `cover_ring` and the floors baked for the replaced partition
(`floor_rule`'s `also`). Nothing reads them any more (the old partition is
drawn from its own records, with its own floors), so they only cost build
time; retiring them changes how long a build takes, which is the perf suite's
question, and is a follow-up (task 7), not part of this change.

## 5. Instruments and evidence

**Landing frames** (`PBD_LANDING_SHOTS=<sets>`, a measurement instrument
only): from the frame set N is asked for until 40 frames after it lands, every
frame is saved next to the `--capture` path as `landing-<N>-f<frame>.png`, and
a `LANDING_SHOT` line logs the frame, the set version drawn and the frames
since the landing. A landing happens when the build ends, which is wall-clock
and not repeatable frame for frame between runs, so the sequence round a
landing has to come out of one run. It changes nothing drawn.

Evidence, in `docs/screenshots/two-set-dissolve/`: the frames round a failing
landing of the scenic flight before the change (the ring switching between two
frames) and the same landing after (dissolving over the following frames),
crops and difference images, and the landing and `LOD_FADE` counts of the
scenic flight and the walk after the change.

## 6. Left open

- **The column tier** is rebuilt round each anchor and switches at a landing:
  where a cave mouth lowers a record at the tier's edge, and the voxel light on
  the caps, change in one frame 50 to 90 m ahead of a walker (`detail-fade`
  section 3). Dissolving it needs the old tier kept as the old records now are.
- **The sea sheet** is listed for the current partition only, so at a landing a
  coastline inside a band's ring takes the new partition's shape at once. It can
  take the same halves (list the old half's water cells OLD and dissolve in
  `water.wgsl`).
- **The cover**: retire `cover_ring` and the replaced partition's floors once
  the perf suite says what the build gains.
