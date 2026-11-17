## Why

The owner, 2026-11-17: "can you do some data gathering, and get me ms times
for GPU vs cpu for walking around doing missions? ... when it comes to
fishing the bot will need to find a large body of water, preferably ocean",
and "i want to see where the GPU budget is going". The F3 graph on the
owner's desktop read 44 fps (p50 22.6 ms, p99 30.1 ms) in ordinary play.
The fixed perf scenarios (`clouds`, `storm`, `walk`, `town`, `rain-stroll`)
fly or walk a set lane; none of them plays the game: crafting, chopping,
digging, fishing at the sea, farming, herding and cooking, with the panels
that go with them, over half an hour or more.

Before any effect is dialled back, the frame has to be split into where it
goes:
- the GPU, per render pass (the frame log already writes this,
  `lighting-perf` 1.2);
- the CPU, which nothing measures yet. Bevy renders pipelined: the main
  world (the game's systems) runs one frame while the render world encodes
  the last, so a frame is as long as the slowest of the main world, the
  render world and the GPU.

## What changes

A measurement instrument only (CLAUDE.md, "Write it up before touching
code", the instrument exception). Nothing a player sees or hears changes.

- `--frame-log` gains four columns at the end of each row:
  - `cpu_main_ms`: the main world's update, from the start of `First` to
    the end of `Last` before the frame limiter;
  - `cpu_render_ms`: the render world's `Render` schedule, extraction's
    commands to cleanup;
  - `acquire_ms`: the part of that spent in `ManageViews`, where the
    swapchain image is taken. It is waiting on the GPU and the display,
    not CPU work;
  - `shot`: the showcase shot being played (0 outside a showcase), so each
    stretch of the log is tied to the activity in it.
- The showcase bot plays the phase-1 branches (trunk, fishing, farming,
  herding, cooking) on one world with `--frame-log`. Fishing is played from
  the shore of the sea: the showcases start on the pasture with the sea
  within 30 m, and the bot's `Shore` act walks to the nearest water deeper
  than 0.6 m.
- `tools/play_budget.py` reads the logs and reports, per activity: the
  frame's p50/p95/p99, GPU total against the main and render CPU, which of
  them bounds the frame, and the GPU passes costliest first.

## Measured on

The owner's desktop, RTX 3070, the `fast` build (memory: perf runs use
`target/fast` for now, no extra builds), windowed, the player settings'
defaults as every `--frame-log` run does. Results and the report go in
`docs/benchmarks/2026-11-17-play-budget/`.

## Not in this change

Any cut. Effects whose cost the report finds are proposed afterwards: a cut
that keeps the picture is made (CLAUDE.md, "Performance improvements need
no permission"); one the player would see goes to the owner with
screenshots.
