## Findings (2026-11-28, the owner's desktop, RTX 3070)

The instrument (`--features system-times`, `fast` profile): each system's
main-thread time summed by its span, its slowest call, the entity, mesh,
material and image counts, and the meshes modified a frame, logged every
30 s as `SYSTEM_TIMES` and `SYSTEM_WORST`; and, at debug level,
`TICK_PARTS`, a water tick's parts.

### What it is not

| Run | Minutes | fps | Grows? |
| --- | ---: | ---: | --- |
| World 19 copy, standing where the owner quit | 6 | ~57 | nothing: memory 1.245 GB, flat |
| World 19 copy, 5 km straight walk (stalled at 815 m) | 5 | ~40 | nothing: ~920 meshes, ~11,250 entities, 1.29 GB |
| Fresh world, the river walk (`--walk-river right --at 19.25 -6.75`) | 3 | ~60 | main 13 to 15 ms |
| Fresh world, three pours running 4 min by the cascade, standing | 4 | ~70 | nothing |

- **Not a leak of entities, meshes, materials or images.** None of them grows
  on any run.
- **Not the time played.** Standing still it does not move.
- **Not flowing water alone.** Water that never settles (three pours, a
  tick a second, 2 to 13 cells moved a tick) runs at ~70 fps. A tick that
  moves water costs the main thread ~8 ms to copy the fine set
  (`run_ticks`' `(*fine.set).clone()`), ~5 ms to hand it to the contact
  tier and 3.5 ms to remesh the water, plus the render thread's whole-tier
  upload (`cascade-perf`, 8-11 ms): a hitch a second, not a slow frame.
- Two of the pour runs had main-thread freezes of 0.7-1.0 s, about two a
  second for 60-90 s; the two repeats with the slowest-call report had
  none, and nothing in the game's log lines up with them. The owner's
  other programs were running (Discord with its clip recorder, Steam,
  Chrome, another game). Not reproduced; not counted.

### What the owner's run had

From its frame log and log (`output/perf/play-20261128-0236.csv`,
`logs/play-release.err.log`):
- minutes 1-5, main 13-15 ms; 6-7, 18-19 ms; 8-10, 24-31 ms with the render
  thread 25-33 ms and the GPU 12 ms;
- the fine set's version rose about once a second from minute 9 (25, 66,
  84): water running where the owner dug by the river (`d82`, `d83` and
  `@wwater:flow` in the save's edits);
- the fine set's landings took 6 s until 08:44:33, then 10.7 s, then 16-18
  s each;
- at 08:42-08:44 the field corners re-read ~100 rows every frame
  (`SPENT 2-5 ms field corners`, 459-729 times a minute) and the town bakes
  swapped 247 times in one minute;
- the river cache was full (2,000,000 ground and 2,000,000 water tops), and
  12 % of lookups hit it: past its cap it keeps nothing new, so ground away
  from what was cached first is worked out again on every rebuild. The file
  carries the full cache to the next launch.

## Plan

1. The digging river walk (`--walk-distance 2000` from the cascade, which
   digs when it stalls on the bank, as a player does), 10 minutes, to
   reproduce what the owner's run had together: new ground landing every
   few seconds while water runs in what was dug.
2. Fix what that shows, measured on the same walk before and after.
3. Either way, the river cache stops freezing at its cap (below).

### The river cache evicts instead of freezing

`river_cache::Memo` keeps nothing past its cap (`river-cache` design 15).
Once the world's rivers near where the player first played have filled it,
every value elsewhere is a miss for the rest of the save's life, file
included. Each shard keeps two generations: a value is put in the young
one; when the young one reaches its share of the cap, the old one is
dropped and the young one becomes old. A read looks in the young, then the
old. Values are exact (a key's value is the call's own result), so which
are kept changes no number the game uses; only what is worked out again.

## Reproduced: the digging river walk (2026-11-28)

`--walk --at 23.2255 -3.012 --yaw 0 --pitch -10 --walk-distance 2000` (the
`cascade-walk` scenario, 2 km), `fast` with the instrument, a fresh world:

| Walk time | main p50 | render p50 | fps |
| --- | ---: | ---: | ---: |
| 60-130 s | 12.7-15.3 ms | 12-13 ms | ~55-60 |
| ~158 s | 43.7 ms | 45.7 ms | ~15 |
| ~190 s | 14.4 ms | 12.1 ms | ~59 |
| ~223 s | 52.2 ms | 56.0 ms | ~12 |
| ~254 s | 30.5 ms | 19.8 ms | ~24 |
| 286 s on | 12-15 ms | 6-7 ms | 45-75 |

Both threads slow together, as on the owner's run (main 30, render 32), and
every system with them (the physics step's slowest call 91-96 ms, the
fixed schedule's 154-184 ms): the threads are short of CPU, not doing more
work. The windows are when the game's own background work stacks up on the
desktop's 8 logical cores:
- the fine set's build: the pool's task and its 2 threads (`far-side-flight`
  already cut it from every core to a quarter, for this same starvation);
- the swamp fill, 7 times in 2 minutes, one every 5-15 s, each longer than
  the last (1.3 s to 4.6 s): it warms the swamp pools' memo, capped like
  the others at 1,000,000 (its fill is not logged, so whether it was full
  is not known);
- the river fills (0.6-1.7 s each);
- the bounce and lamp bakes on the async pool (37-107 ms each, every few
  seconds);
- the river cache's save; and the machine's own load (the owner's Discord
  clip recorder, Steam, Chrome).

The fine set's landings took 10-12 s in those windows, against 6-9 s
either side: the owner's 16-18 s, on a world whose cache had been full for
days, is the same starvation, longer.

## The A/B in release: no slowdown either way (2026-11-28)

The same digging walk, release, fresh worlds, interleaved, 380 s each
(`target/ab/before.exe`, the owner's build of 02:30; `after`, the memos
rolling over and the background threads below normal priority):

| Run | fps | wall p50 / p99 | main p50 | worst 30 s |
| --- | ---: | --- | ---: | ---: |
| before | 91.1 | 10.8 / 18.0 ms | 9.6 ms | 51.5 fps |
| after | 79.7 | 12.3 / 22.3 ms | 10.7 ms | 52.7 fps |
| before | 88.6 | 11.4 / 15.9 ms | 10.3 ms | 51.7 fps |
| after | 87.2 | 11.5 / 16.2 ms | 10.4 ms | 50.3 fps |

Neither build slows down. The windows of 44-56 ms above were the
instrumented `fast` build: Bevy's per-system spans cost every system, and
the report itself once held the main thread 3.5 s. **The owner's slowdown is
not reproduced**, and the two fixes show no gain:
- the background priority (a scheduling change with no measured benefit)
  is taken out again;
- the memos rolling over stays: past its cap the cache kept nothing new for
  the rest of the save's life, which is a fault whatever its cost; values
  are exact, so nothing seen or played changes.

## Still open

What differs between the owner's run and every bot run: World 19 itself
(its own seed, edits and full cache), 10 minutes of the owner's own play
(a raft, fishing, a town at night, digging by a river), and the owner's
other programs. Next: the instrument at release (`--release --features
system-times`; spans cost less there, measured against plain release) in
the owner's next session, so the system that grows is named on the run that
shows it.
