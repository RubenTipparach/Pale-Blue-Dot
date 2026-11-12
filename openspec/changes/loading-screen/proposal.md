# Proposal: a loading screen that builds the world once and warms the shaders

## Why

The owner, 2026-11-12: "You should have a loading screen to prewarm/cache
shaders."

The clean benchmark of that day (`docs/benchmarks/2026-11-12-lighting-model-clean/report.md`)
measured what opening a world costs today, on the RTX 3070 desktop:

- **The window is blank until the planet is built.** The first frame comes
  9-11 s after launch on a quiet machine with a warm driver cache, and 50-55 s
  on the first run of a new exe (a cold cache, in a contaminated sitting).
  Nothing on screen says the game is working.
- **Every open builds the planet twice.** `create_planet` builds it (5.2 s),
  then the towns are laid out in one frame (2.6-3.0 s), then `settle` in
  `towns.rs` installs the towns' ground and calls `planet::rebuild_planet` on
  the main thread: a 6.7-7.5 s frozen frame, 5.3 s of it a planet thrown away.
  A saved world pays the same rebuild before its first frame (13.0 s to first
  frame against 9.3 s). This is `town-frame-time` task 3.6.
- **Pipelines are made the first time something is drawn.** Bevy compiles a
  render pipeline when a material or pass is first queued, so the first sight
  of water, cloud, bloom or a reflection in play stalls or pops.

## What

1. **A loading screen from launch until the world is playable**, in the HUD's
   look (monospace, the menu's ink and mint, a pixel bar), naming the step in
   hand and its progress.
2. **The world is prepared off the main thread while the screen draws**: the
   sites read or surveyed, the towns read or laid out and stored, their ground
   installed, and only then the planet built, **once**, with the ground in
   place. No build-then-rebuild on opening a world.
3. **Every pipeline the opening view needs is compiled behind the screen**:
   the world runs and renders under an opaque overlay, the camera looks round
   once, and the screen stays until Bevy's pipeline cache has nothing waiting
   and the towns in range stand.
4. **A `READY` log line** marks the end of loading, with the seconds since
   launch and the frame number, so the perf suite counts its warm-up from it.
   Pipelines made after it are logged by name (`LATE_PIPELINE`), so the next
   prewarm knows what play still compiles.

## Saved games

Nothing a save holds changes. The sites, the towns and their "unsettled" marks
are stored by the same functions in the same order as today, and the ground is
derived from them as today. The planet built once with the ground installed is
the planet the old build-then-rebuild made: a test pins it byte for byte.
Opening a world never resets it.

## Not in this change

- **Switching worlds from the saves page** still rebuilds the planet in place
  (`load_world`); it is not behind the loading screen yet. It is a task here.
- **Pipelines for what the opening view never shows** (the map screen, a
  vehicle, underwater, rain at night) are not forced; the `LATE_PIPELINE` log
  names them for a follow-up.
- **wgpu's on-disk pipeline cache** is not used: Bevy 0.18's pipeline cache
  passes `cache: None` to every pipeline it creates (see the design). The
  NVIDIA driver's own cache already persists compiled pipelines between runs.
- No generation output, tuning constant, shader term or default moves.
