# Design: REFLECTIONS, OFF to HIGH

## The owner's words

2026-11-16: "add SSR quality to settings as well off, low, med, high (low not
having that raycast thing where it finds the geometry facing groundward)",
and "rain shadow should be under picture".

## 1. What "the raycast thing" is

L8's wet-ground reflection is two traces in order, "voxels first (owner); the
screen where they find nothing" (`planet_surface.wgsl`):

- `reflect_trace`, the **voxel walk**: a DDA over the hex prisms of the
  column tier, from the wet face along the reflected ray, until it enters a
  solid cell or walks 72 cells or `reflection_reach_m`. It then lights the
  face it entered (sky by the field, the sun with its cascades, the lamps).
  Its comment says why it exists: "The screen cannot see what the camera
  cannot: an eave's underside, a slab's, the far side of a wall." Those are
  faces turned toward the ground. This is a ray cast into the world's
  geometry that finds the geometry facing groundward: the owner's words fit
  it, and fit nothing else in the frame.
- `ssr_trace`, the **screen trace**: 32 steps through last frame's capture,
  reprojected. It can only show what was on screen. The sea's `sea_ssr` is
  the same march over the same capture.

So LOW is the screen-space reflection without the walk: what "SSR" names.

## 2. The levels

| Level | Capture (`ssr_capture`) | Voxel walk | Wet ground's screen trace | Sea's screen trace | Half-size passes (`reflect_half`, `reflect_compose`) | Wet ground's reach |
| --- | --- | --- | --- | --- | --- | --- |
| HIGH | yes | yes | yes | yes | yes | 48 m (`reflection_reach_m`) |
| MEDIUM | yes | yes | yes | yes | yes | 24 m (half) |
| LOW | yes | **no** | yes | yes | yes | 48 m |
| OFF | **no** | no | no | no | **no** | none |

What every level keeps, because it costs nothing extra: the wet film's
fresnel-weighted sky (`sh_radiance` off the sky's nine coefficients, floored
at `ground_sky`), the sun's glint and each near lamp's glint on wet ground;
the sea's reflected sky, its fresnel and its sun glint. Off, a reflection
that would have found a wall shows the sky there instead: the miss the traces
already draw where they find nothing, so the water is never black.

**MEDIUM's knob** is the reach: `reflection_reach_m` halved. The walk stops
at 24 m (or its 72-cell cap), the screen trace at 24 m (its steps grow with
distance, so the far steps go), and the hit fades over the last quarter of
the reach as before, never cut. The half-size target stays at
`reflect_render_scale` (0.5): the gather reads one requesting pixel of each
2 x 2, so a smaller target would leave requests unread. The sea keeps its
own 120 m march (0.14 ms in a storm, `lighting-perf` 1.1).

**OFF skips the passes; it does not run them and discard.** The capture node
returns before it begins its pass; the terrain is drawn with the inline
`fragment` entry (one target), so the half-size targets are not made and
`reflect_half` and `reflect_compose` do not run; the terrain's uniform says
walk and screen trace off (`reflect.x`, `reflect.w`) and the history unread
(`ssr_eye.w`), so the inline traces return at their first line; the sea's
`ssr_eye.w` is zero, so `sea_ssr` returns at its first line.

## 3. How the level reaches the render world

The developer's switches stay where they are: `lighting.ron`'s `reflections`
and `PBD_REFLECT` still turn parts off for pricing. The player's level is
laid over them:

- `config::Reflections` (`Off`, `Low`, `Medium`, `High`) and
  `config::ReflectQuality`, a resource extracted to the render world as
  `RainShadows` is.
- `settings::follow_reflections` copies `PlayerSettings::reflections` into
  it each frame it differs, so a change applies the next frame.
- `LightingSettings`'s `reflect_walk_on`, `reflect_screen_on`,
  `reflect_sea_on`, `reflect_capture_on` and `reflect_split_on` take the
  level, and a new `reflect_reach_m(level)` gives the reach. Every caller is
  in the render world and already reads `LightingSettings`.

## 4. No late pipelines

The levels use the pipelines HIGH uses, with one exception: the terrain's
inline `fragment` entry, which OFF draws with (HIGH, MEDIUM and LOW draw
`fragment_split` on the MSAA camera). `queue_planet` specialises both, for
every view, every frame (the specialiser caches, so it is a lookup after the
first), so both are compiled behind the loading screen whichever level the
game starts at. The capture, gather and compose pipelines are queued at
`RenderStartup` whatever the level.

A switch from OFF to anything else turns the capture on. Its history then
holds a frame from whenever it last ran, so the screen traces read it only
once a capture of this run of frames has been made: `ssr_eye.w` is one only
when the capture ran the frame before. No frame reflects a stale image.

## 5. The settings file

- `PlayerSettings::reflections`, default `High`; `ISOLATED` (captures and
  benchmarks) `High`.
- The file's version 9 adds `reflections` (version 8 was `interact:KeyF`,
  landed beside this change). A file of version 8 or older has
  no such field and reads `High`, the default: nobody's picture changes on
  upgrade. A test reads a version-8 file.
- `--reflections off|low|medium|high` overrides it for a launch.

## 6. The menu

PICTURE: Bloom, Frame limit, Render scale, **Reflections**, **Rain shadow**,
Pixel grass, Flow arrows. Reflections is a slider of four steps whose value
reads OFF, LOW, MEDIUM or HIGH, as Frame limit reads OFF, 60 FPS, 120 FPS.
No notes on the row ("Notes stay out of the game"). Rain shadow moves here
from WEATHER, which keeps the storm forcing; its value (`rain_shadows`) and
its flag (`--rain-shadows`) are unchanged.

## 7. Proof

- Captures, the same scene at each level, by day and at night with lamps:
  `docs/screenshots/reflections-quality/`, with a README of the flags.
- `tools/perf_suite.py` on the release build, the four levels as variants
  (`--reflections <level>`) in one sitting, `storm` and `walk`, two repeats:
  `docs/benchmarks/2026-11-16-reflections-quality/`.
- Launches to `READY` at each level with no `LATE_PIPELINE`, and a level
  switched in the menu with none.
