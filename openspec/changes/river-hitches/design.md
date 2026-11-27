## 1. Measured

See the proposal. Instrument: `--frame-log` (the `fine_version` column steps
on each landing, on the same clock as `cpu_main_ms` and `cpu_render_ms`).

## 2. The trace

`--features trace` on the fast profile in its own target folder
(`target/trace`), the same 1 km walk. Rank spans in the frames over 33 ms.
Findings go here before any fix.

**Found** (400 m traced walk, the fast profile): after loading, six main
frames over 33 ms; in every one over 25 ms the time is Bevy's
`propagate_parent_transforms` (25 to 33 ms, all four workers busy), and one
more is the river cache's `save_on_exit` (447 ms, at exit only). Nothing of
the game's own systems is over 2 ms in those frames. A propagation that
long is the whole hierarchy recomputed at once, and the one thing that marks
every root changed is the render origin's rebase (`render_origin::rebase`),
which moves every root `Transform` and every `GlobalTransform` each time the
camera crosses its 16 m cell: every two seconds on foot.

**A/B** (release, the 1 km river walk, alternated in one sitting; the first
of each pair is a cold cache):

| run | frames | p50 | p99 | > 25 ms | > 33 ms | > 50 ms |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| normal 1 (cold) | 18,765 | 14.3 | 21.0 | 36 | 20 | 10 |
| origin held 1 | 12,956 | 13.5 | 21.3 | 31 | 9 | 6 |
| normal 2 | 10,491 | 12.1 | 20.3 | 25 | 6 | 1 |
| origin held 2 | 11,679 | 11.0 | 16.4 | 6 | 4 | 1 |

`PBD_HOLD_ORIGIN=1` (the origin never moves) takes the warm walk's frames
over 25 ms from 25 to 6 and its p99 from 20.3 to 16.4 ms.

## 4. The fix: the render origin rebases as the sim origin does

`world-frames` design already settled the sim origin's cell with the owner
("16 m is kinda small, 128 m probably good"): **128 m**, and a rebase only
once the anchor is more than 0.75 cell (96 m) off the origin on an axis, so
a boundary never thrashes and a rebase comes every 128 m of walking at
most. The render origin keeps its own 16 m grid and rebases on every
crossing. It takes the same rule (`pbd_core::world_frame::ORIGIN_CELL_M`
and the 0.75 cell hysteresis): eight times fewer rebases on foot, each the
same cost.

Precision: a render coordinate within 96 m of the origin holds 7.6 µm in
f32, against 2 µm at 16 m; nothing drawn moves by a visible amount, and the
camera's own transform is still small. The picture does not change, so this
is a perf cut that needs no permission (CLAUDE.md).


## 3. The playthrough scenario

`play-soak` is a showcase: the bot plays through the player's inputs. It is
a measurement instrument (CLAUDE.md, "Write it up before touching code") and
changes nothing in the game.

## 5. Measured after (same sitting, release, fresh world per run, `--time 10 --day 0`)

| run | walk | frames | p99 | > 25 ms | > 25 ms a minute |
| --- | ---: | ---: | ---: | ---: | ---: |
| before 1 | 323 s | 26,587 | 20.0 | 54 | 10.0 |
| after 1 | 242 s | 20,203 | 19.5 | 35 | 8.7 |
| before 2 | 249 s | 20,891 | 19.5 | 25 | 6.0 |
| after 2 | 168 s | 14,461 | 18.7 | 12 | 4.3 |
| before 3 | 182 s | 15,600 | 18.7 | 13 | 4.3 |
| after 3 | 200 s | 16,834 | 19.0 | 17 | 5.1 |

The walks take different routes (12 to 32 turns round obstacles), and the
runs get better as the shared cache warms, so the gain is inside the spread:
**a small one at most**. The change stays (fewer rebases, the sim origin's
rule, the picture the same), but it is not the owner's slowdown. On a clean
walk there are 0 to 7 frames over 33 ms in 1 km; the owner saw 30 fps
standing still and 12 fps in a town. That is in play, not in the walk:
the overnight playthroughs (the phase-1 tree and `play-soak`) look for it.

An earlier A/B reused one world for every run: the save carried the clock
and the bot's turns between runs, and its later runs (581 frames over 33 ms
in one) measured the hour and the route, not the change. Discarded.

## 6. Found in play: every block edit costs a 100 ms frame

The overnight phase-1 trunk (release, `--showcase phase-1-trunk`, frame
log, aligned to the log at the first landing; `dips.py` in the session):
**every dig the bot made, v3 to v12, is a frame of 100 to 123 ms, nearly all
of it the main thread** (`cpu_main_ms` 99 to 119, render 10 to 13, GPU
9 to 12). Nothing else in the run comes close; the walk had no edits, which
is why it missed them. Digging, chopping, placing, tilling and picking up
all edit the ground, so a player at work stutters on every action: the
owner's "slow right away".

The two light bakes logged beside each one (bounce 33 to 52 ms, lamps 15
to 20 ms) run on the async pool, not the main thread. What runs on the main
thread is `digging::apply_edit`:

1. `(*fine.set).clone()`: the whole `FineSet`, four levels of `GpuCell`
   records (about 200 B each; 65 MiB at the spawn, the startup log's
   "65.0 MiB records") and the column tier, copied for one block;
2. `set.apply`: the column, its record and its neighbours';
3. `set.finish_edits()`: the whole tier relit and the sea's sheet re-marked
   ("the measured six milliseconds", measured when the tier was smaller);
4. `contact.set_fine(&set)`: the contact tier rebuilt from the new set;

then, after it, the tier image (7 to 9 ms, `SPENT ... tier image`) and the
field corners.

**Across the whole phase-1 tree** (seven branches overnight, the frame log
alone: a frame where the set's `fine_version` changes is an edit or a
landing):

| branch | frames | version changes | their median | of them 80 ms or more | frames over 33 ms | of those at a change |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| trunk | 25,384 | 46 | 106 ms | 44 | 107 | 46 |
| fishing | 39,640 | 15 | 112 ms | 15 | 52 | 15 |
| farming | 37,302 | 18 | 111 ms | 17 | 75 | 17 |
| herding | 51,764 | 19 | 108 ms | 18 | 62 | 18 |
| cooking | 10,161 | 8 | 115 ms | 7 | 20 | 7 |
| mining | 43,338 | 134 | 109 ms | 134 | 219 | 138 |

Every change of the set, 241 of 246, is a frame of 80 ms or more; they are
about half of all frames over 33 ms. The rest (render-thread spikes of 30 to
90 ms as a bounce bake lands, the river cache's save) come after.

Its own timing (`edit cost: clone, repack+reconcile, relight, contact`) is
a `debug!` and is not in the logs. **Next: that split, measured**, on the
same trunk with the digging module at debug, before any fix is chosen. The
fix is one that keeps the picture (perf, no permission needed); the
candidates, by what the split shows:

- **the clone**: the coarse levels never change on an edit, so they become
  shared (`Arc`) and only what is written is copied; the finest level in
  shared pages, so an edit copies the pages it touches, not 37 MiB;
- **the relight**: a bounded pass round the edit (the reference's way)
  where the whole tier has grown past "six milliseconds";
- **the contact rebuild** (`FineTier::new`): a second copy of the finest
  level and a hash grid over its 191,000 cells, rebuilt on every edit,
  though an edit moves no cell: the grid and the neighbours are kept from
  the tier it replaces, and the records are read from the set, not copied.

## 7. The split, measured, and the fix

The trunk again (release, fresh world, `RUST_LOG=...digging=debug`), nine
digs, the edit's own `edit cost` line:

| part | each dig |
| --- | ---: |
| clone of the set | 0.8 to 2.7 ms |
| apply (repack, reconcile) | 0.2 ms |
| **relight** (`finish_edits`: the whole tier's light, and the sea's sheet) | **52 to 64 ms** |
| contact | 6.8 to 7.9 ms |
| total, in `apply_edit` | 70 to 84 ms |

The rest of the ~110 ms frame is what a new set version sets off after it
(the tier image, 7 ms; the field corners). The clone is cheap; the relight
is three quarters of it. Its own comment ("ours costs six milliseconds")
dates from a smaller tier: it is ten times that at the spawn now.

**Fix: the relight leaves the main thread** (as the bounce and lamp bakes
did). `apply_edit` writes the edit, marks the tier's light stale and
publishes the set at once; a system (`relight_edits`) bakes the light and
re-marks the sea on a copy on the async pool and swaps it in when it is
done, if the set is still the one it copied, the way `switch_dusk_lamps`
swaps its relit set (with `PlanetContact::relit`, so the contact is not
rebuilt). An edit that adopts a column past the tier's edge still relights
at once: it changes the tier's shape.

What changes on screen: a dug cell's light arrives with the bake, about 60
ms (four frames) after the block goes, where it arrived with it; no blend
is added. Light is the field's, so the picture once baked is the same cell
for cell. Taken as a perf cut (CLAUDE.md, "Performance improvements need no
permission"); reported to the owner with the numbers.

### 7.1 Measured after (the trunk, release, fresh world each, alternated, 4 min each)

| run | set changes | median frame at a change (main) | worst at a change | frames over 50 ms |
| --- | ---: | ---: | ---: | ---: |
| before 1 | 36 | 102 ms (100) | 118 ms | 38 |
| after 1 | 26 | 36 ms (29) | 56 ms | 15 |
| before 2 | 10 | 106 ms (104) | 110 ms | 9 |
| after 2 | 18 | 28 ms (26) | 57 ms | 6 |

An edit now changes the set twice (the edit, then its light 70 to 75 ms
later, `EDIT_RELIGHT on the pool`), each a frame of about 30 ms where it was
one of about 105. The over-50 counts include the loading screen's frames.
What is left at a change: the contact rebuild (7 ms, `FineTier::new`
copying the finest level and hashing its 191,000 cells), the tier image
(7 ms) and the rest of what a version sets off: the next cut (task 4.5).
