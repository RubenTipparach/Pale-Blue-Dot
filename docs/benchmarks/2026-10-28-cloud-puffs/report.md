# cloud-puffs: march against puffs, on the Mac

`cloud-puffs` adds a second cloud renderer: one lit billboard a puff, laid out
by the weather maps (Crab-breeze's look, on the sphere), drawn into the
march's targets so the composite, haze and near fog are unchanged. This run
prices it against the shipped march and against no clouds, on one build
(`PBD_CLOUD_RENDERER=march|puffs`, `PBD_NO_CLOUDS=1`).

## Findings

1. **On the cloud leg the puffs cost a fifth of what the march does.** Frame
   p50 over the clouds-off floor: march +9.7 ms (17.9 against 8.2), puffs
   +1.9 ms (10.1). At the 95th percentile, march +19.3 ms, puffs +5.9 ms.
   Over the whole flight: 18.3 / 11.1 / 8.9 ms p50 (march / puffs / off),
   41 / 62 / 73 fps.
2. **The clouds' GPU span halves: 14.9 to 6.7 ms** at p50 on the cloud leg
   (16.3 to 7.3 at p95). The span runs from the first cloud pass's start to the
   last one's end, and on this GPU other work overlaps it, so it is more than
   the wall time the clouds add (1.9 ms for the puffs).
3. **What is left of the puffs' cost is mostly not the puffs.** Under puffs the
   span still holds the march pass making only the near fog (a full-screen pass
   at 0.4 scale with four depth reads a texel) and the full-resolution
   composite, at 2880 x 1800. The next saving is there: skip the near-fog pass
   when the eye is nowhere near the layer, and cheapen the composite.
4. **No storm result.** Every run after 16:12 (all `storm` and `cloud-hop`
   runs, both sittings) was paced by the OS, not measured: the Mac's screen
   was locked, macOS throttled the game's window and Bevy ran it unfocused at
   60 Hz. Their medians sit at 16.7-17.5 ms whatever the clouds (clouds off
   included), with half their frames between 15 and 18.5 ms against a tenth
   in the `clouds` runs. Found by the `frame-budget` investigation; the suite
   now refuses to start under a locked screen and marks a run whose window
   lost focus or was hidden. The storm is to be measured again, unlocked.
5. **Repeats agree**: p50 within 0.1 ms between the two runs of every variant
   on `clouds`.

## Limits

- **Not the owner's desktop.** MacBook Pro, Apple M4 Max (40-core GPU), Metal,
  release, window 1440 x 900 logical = 2880 x 1800 pixels (Retina), four times
  the pixels of the desktop's runs. The desktop (RTX 3070) was busy; its
  numbers will differ, and `tools/perf_suite.py` should be run there before
  the renderer is chosen.
- **GPU time on Metal is a span, not per pass.** Bevy's pass spans cannot time
  on Apple GPUs (no timestamps inside a pass); the new boundary timer
  (`planet_cloud_timer.rs`) times the cloud passes from outside. Per-pass pairs
  overlap on this GPU and are not reported. "GPU total" is not measured here.
- **`cloud-hop` did not run**: the fresh benchmark world had no cloud in that
  route's reach ("no cloud found on any heading"), and its runs were paced
  too.
- **Only the `clouds` sitting is kept** (11:06, interleaved march / puffs /
  off, screen unlocked). The first sitting's storm march runs also crashed:
  Bevy's render diagnostics plus the new timer ran out of Metal counter sample
  buffers ("Cannot allocate sample buffer"); on macOS Bevy's diagnostics plugin
  is now left out (it measured nothing there).
- **The look is not judged here.** Stills, march above and puffs below:
  `docs/screenshots/2026-10-28-cloud-puffs-orbit.jpg`, `-surface.jpg`,
  `-coast.jpg`. From orbit the puffs read as a cumulus field; from the ground
  they are softer and flatter than the march's; from inside cloud
  (`-coast.jpg`) they are not right yet (a light grey with a dark smear where
  the march draws an even fog).
