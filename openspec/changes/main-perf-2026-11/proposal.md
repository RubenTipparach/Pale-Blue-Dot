# Proposal: what main's new work costs a frame, found and cut

## Why

The owner, 2026-11-15: "get latest from main and start inspecting what the
performance hit is on the new changes", then "go ahead and do
optimizations". Since the last sitting that held every scenario inside
16.7 ms at p99 (`27c2d11`, `docs/benchmarks/2026-11-12-worst-case-60/`),
eleven PRs landed: the stone-and-wood UI, phase 1 with its audio, env-v2's
water, rivers and foliage, the doubled LOD distances.

The owner asked for no second build ("no other builds please to save
space", "kep it in target/fast"), so there is no old-against-new sitting:
everything below is main's `fast` build (`c14c8cd`, no LTO, so a little
slower than release) on the desktop (RTX 3070), sound on against
`--no-audio`, interleaved, 2 repeats (`output/perf/2026-11-15-main-fast`).

## What was measured

| scenario | p50 on / off (ms) | p99 on / off (ms) | frames > 30 ms on / off |
| --- | --- | --- | --- |
| clouds | 6.14 / 6.04 | 13.0 / 12.1 | 0 / 0 |
| storm | 6.33 / 6.03 | 18.96 / 11.76 | 13 / 0 |
| far-side | 5.71 / 5.22 | 13.0 / 9.3 | 6 / 0 |
| walk | 9.18 / 7.07 | 76.7 / 15.0 | 1156 / 2 |
| town | 10.16 / 10.17 | 29.4 / 17.4 | 103 / 25 |

- The flights fly one path whatever the frame time, so they price sound:
  0.1 to 0.5 ms at p50, and the storm out of the 60 fps floor at p99.
- The walks do not. The walker steers by what it meets each frame, and with
  sound on it took 3 to 27 turns against 2 and 150 to 270 s against 130 s
  for the same 1000 m. The runs that wandered reached ground where the GPU
  time climbs from 6 to 40-60 ms, with town light bakes and fine-set
  rebuilds beside the slow frames. Sound uses no GPU: that cost is the
  place's, which a player can walk to.
- No voice leak: the audio log holds 0 to 4 voices through a walk; the
  process holds about 870 MB through it.
- `cloud-hop` gave no valid run with sound on or off.

## What changes

1. A measurement instrument: every sound system logs `SPENT <ms> ms audio
   <system>` past 1 ms, so the frame graph names the one that stalls.
2. Cut what it names (design, filled in once measured).
3. Pin the walk so on and off cover the same ground, then find the place
   the wandering walks reached and what costs it there.

## What was found and cut (2026-11-15)

The instrument named one system: `mix_frame`, 5 to 10 ms on 5 to 7
frames a flight, every one a bed starting. Timed apart, opening a bed's
file took 1.4 to 3.5 ms on the main thread and starting it 0.2 to 0.4 ms;
a scene change opened three in one frame (9.3 ms), and the low wind was
reopened three times as its level hovered at the one start-and-stop edge.

1. **Beds open on the IO pool.** Only the 0.3 ms start stays on the main
   thread. A capture still opens in the frame (it steps a fixed clock).
   After it, no sound system took over 1 ms past startup, but the storm
   still read 19.75 ms at p99 with sound against 11.98 without: the start
   was not the cost.
2. **Beds are decoded whole, not streamed.** kira 0.12 gives every
   streaming sound a thread that wakes each millisecond
   (`decode_scheduler.rs`, `DECODER_THREAD_SLEEP_DURATION`). A storm plays
   five to seven beds at once on the desktop's 8 cores (i7-9700F, no SMT),
   and with sound on twice as many slow frames fell inside a fine-set
   rebuild (41-77 a run against 23-27). Decoded on the IO pool, a playing
   bed holds about 13 MB and runs no thread of its own.
3. **A bed starts above 0.02 and stops below 0.01**, so one at the edge is
   not reopened again and again.

| flight | p99 on / off before (ms) | p99 on / off after (ms) | > 30 ms on / off after |
| --- | --- | --- | --- |
| storm | 18.96 / 11.76 | 11.62 / 12.38 | 0 / 0 |
| far-side | 13.0 / 9.3 | 9.40 / 9.18 | 2 / 0 |
| clouds | 13.0 / 12.1 | 13.00 / 11.68 | 0 / 0 |

`fast` build, the desktop, 2 repeats interleaved
(`output/perf/2026-11-15-bed-static`). The spread between repeats is about
1 ms at p99, so sound is now inside it on every flight.

## Still open

- Streamed cues (thunder, the long loops: `stream: true` in `cues.ron`)
  still get kira's thread each. A thunder roll is one at a time and short;
  not measured apart.
- The walks are not a measure of anything that changes frame timing: the
  walker turns where it stalls, and the turn points move between repeats
  with or without sound (the town walk turned at 82, 83, 92 and 93 m).
  The walker needs pinning to one route before a walk can price a change.
- The ground the wandering walks reached, where the GPU climbs from 6 to
  40-60 ms, is not yet found.
- `cloud-hop` gives no valid run.

## The shadow pass (2026-11-16)

The rain stroll (Holbrook's lane, scripted, the same ground every run) is
the on-foot measure. Against 2026-11-12's sitting (GPU time does not
depend on the build profile), the GPU frame is 1.6 to 2 ms heavier and the
p99 19.9 ms against 12.9-13.1. The shadow pass grew most: 2.14-2.36 ms to
3.35 ms; terrain 2.01-2.19 to 2.63; half-size reflections 0.34 to 0.53.

An instrument, `PBD_SHADOW_SKIP=terrain,foliage,column,towns`, leaves
casters out of every cascade. Priced on the stroll, 2 repeats
(`output/perf/2026-11-16-shadow-parts`):

| left out | shadow pass (ms) | so the part costs |
| --- | --- | --- |
| nothing | 3.41 | |
| terrain | 2.27 | 1.14 |
| column tier | 2.46 | 0.95 |
| foliage | 2.73 | 0.68 |
| towns | 3.21 | 0.20 |

The rest, about 0.44 ms, is each cascade's cull. Every cascade draws every
caster its box holds, and the boxes nest, so ground near the eye is drawn
into all of them. Next: keep a caster out of a cascade whose band of
receivers it cannot shade.
