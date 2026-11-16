# Proposal: a REFLECTIONS setting, and RAIN SHADOW on the PICTURE tab

## Why

**The owner's words (2026-11-16):** "add SSR quality to settings as well off,
low, med, high (low not having that raycast thing where it finds the geometry
facing groundward)".

And, the same day: **"rain shadow should be under picture"**.

The reflections (`lighting-model` L8, priced and moved to half size by
`lighting-perf` 2.8) are the one large lighting cost the player cannot turn
down. They are three pieces:

1. **The capture** (`planet_ssr.rs`, `ssr_capture`): after the water's
   composite, the frame's colour and distance go into a small fixed-size
   history the next frame's screen traces read.
2. **The voxel walk** (`reflect_trace` in `planet_surface.wgsl`): a wet
   face's reflected ray walks the block grid, cell to cell, up to 72 cells or
   `reflection_reach_m` (48 m), and lights the face it meets. It exists for
   what the screen cannot show: "an eave's underside, a slab's, the far side
   of a wall" (the shader's own comment), the faces that look down at the
   ground. **This is the owner's "raycast thing where it finds the geometry
   facing groundward".**
3. **The screen traces**: wet ground's `ssr_trace` (where the walk found
   nothing) and the sea's `sea_ssr` in the water's composite, both 32-step
   marches through the capture.

Wet ground's walk and screen trace run at half size (`reflect_half`, then
`reflect_compose`) while the camera is MSAA.

The owner's words name SSR quality with four levels and say what low lacks:
the walk. Low keeps the screen-space reflection proper, which is what "SSR"
names, and drops the search through the world's own geometry.

## What changes

- A player setting **REFLECTIONS** on the PICTURE tab: OFF, LOW, MEDIUM,
  HIGH, a four-step slider in the Frame limit row's style.
- **HIGH** is today, exactly. **MEDIUM** halves the reach of wet ground's
  walk and screen trace (48 m to 24 m). **LOW** drops the voxel walk; the
  screen traces stay. **OFF** runs none of the passes: no capture, no
  half-size gather or compose, no sea trace. Water and wet ground keep the
  sky, the fresnel and the sun's and lamps' glints they already draw.
- The settings file goes to version 8 with `reflections`; an older file
  reads HIGH, so nobody's picture changes on upgrade. A new player gets
  HIGH (today's look is what ships). Captures and benchmarks take HIGH.
- `--reflections off|low|medium|high` for captures and perf runs.
- It changes live, and every pipeline any level uses is compiled behind the
  loading screen (no `LATE_PIPELINE` on a switch).
- The **RAIN SHADOW** row moves from WEATHER to PICTURE, beside REFLECTIONS.
  Its saved value and its flag do not change.

## Non-goals

- A new reflection technique (planar, probes). Low is cheaper by doing less.
- Changing what HIGH draws.
