# Design: player settings

Decisions marked **recommendation taken (ask only with screenshots)** are
Claude's recommendations, applied under CLAUDE.md's "Ask only with
screenshots" rule. They are not the owner's words. The one choice that can be
shown in pictures, the upscale filter, goes to the survey with its
screenshots (decision 6).

**New players' defaults, the owner's choice (2026-11-13, "default to this
settings"):** frame limit 60, render scale 85 %, auto step OFF, bloom ON
(bloom was already on by default off a Mac). These replace the "today's
behaviour" defaults in sections 3 to 5 for a player. Captures and
`--frame-log` runs keep today's engine (`PlayerSettings::ISOLATED`: uncapped,
100 %, auto step on) under their flags, so no capture or benchmark moved.

## 1. The menu: the settings page that already exists

**Recommendation taken (ask only with screenshots).** `Escape` opens the
pause menu, and SETTINGS opens the settings page (`menus-and-a-quiet-screen`).
That is where the new rows go. There is no second menu and no new key: a
second way into settings is a second thing to learn.

The page becomes, top to bottom:

```text
SETTINGS
GRAPHICS
  [ BLOOM: ON ]
  [ FRAME LIMIT: OFF ]
  [ RENDER SCALE: 100% ]
MOVEMENT
  [ AUTO STEP: ON ]
INPUT
  ... the binding table ...
[ BACK ]
```

The look is the page's own, which is the game HUD look in
`docs/mockups/ART-DIRECTION.md` ("Its HUD"): `PANEL_FILL` panel, `EDGE`
borders, `INK` text, `MINT` section headings, real hit-tested buttons with
idle, hover and press fills. Each row is one `button()` that cycles its value
on a press. Its label is rewritten by one system, like `label_graphics`
rewrites BLOOM's, so the words cannot disagree with the state. A cost hint
follows a value only where there is a measured cost to state.

## 2. The settings file

**Where.** `settings.ron` in the user's config directory, never under the
working directory:

| OS | Path |
| --- | --- |
| Windows | `%APPDATA%\PaleBlueDot\settings.ron` |
| Linux | `$XDG_CONFIG_HOME/pale-blue-dot/settings.ron`, else `~/.config/pale-blue-dot/settings.ron` |
| macOS | `~/Library/Application Support/PaleBlueDot/settings.ron` |

`PBD_SETTINGS=<path>` overrides the location, for A/B runs. The path is
worked out from the environment, with no new crate.

**What.**

```ron
(
    version: 1,
    frame_limit: Off,      // Off | Fps60 | Fps120
    render_scale: 100,     // percent, one of 50, 67, 75, 85, 100
    auto_step: true,
)
```

Every field has `#[serde(default)]`, so a file missing a field gets that
field's default and keeps the rest. The rules:

- **Missing file:** defaults. Nothing is written until the player presses
  a row.
- **Unreadable file** (not RON, or a value out of range): defaults, a
  warning in the log, and the file is left as it is. It is rewritten only
  when the player changes a setting, so a hand edit with a typo is never
  overwritten silently at startup.
- **Older version:** read field by field. Fields it lacks take defaults.
  Version 1 is the first, so "older" today means a file with no `version`,
  read as version 0 with the same fields.
- **Newer version** (a later build's file opened by this one): the fields
  this build knows are read and the rest ignored. Writing it back would drop
  the later build's fields, so a newer file is never written by an older
  build: a change is then kept for the session only, and the log says so.
- **A render scale not on the list** is snapped to the nearest one on it.
- **Saves.** The file is not in `saves/`, names no world and is read by no
  save path. Opening, loading or deleting a world never touches it, and it
  never touches a world.

**Captures and benchmarks do not read it.** A run with `--capture` or
`--frame-log` takes defaults, then flags: `--frame-limit <0|60|120>`,
`--render-scale <percent>`, `--no-auto-step`. The same flags also override
the file in a plain run. A capture is already "a function of its flags"
(`desktop.rs` on the clock), and a perf sitting on the owner's machine must
not move because they chose 67 % in game.

BLOOM keeps its own file, `saves/preferences.ron` (proposal, "Not in this
change").

## 3. Frame limit: OFF, 60, 120

**Default OFF**, which is today: the frame rate is whatever vsync, or
`--no-vsync`, gives.

**How.** A pacer in `Last`, after the frame log is written, holds the main
loop until the next deadline. The deadline advances by one period per frame
(16.667 ms or 8.333 ms), not "now plus a period", so the mean rate is exact
and does not drift by the sleep's error. If a frame lands more than one
period late (a hitch, a load), the deadline is re-anchored to now, so the
game never runs fast to catch up. The wait is an OS sleep to within 1.5 ms of
the deadline, then a spin, because a Windows sleep can overshoot by about a
millisecond. The spin costs at most that much of one core per frame. The
pacer is a plain struct over `Instant`s with no Bevy in it, so a test drives
it with a made-up clock.

**With vsync.** Present mode is not touched. On a 60 Hz display with vsync,
a cap of 60 does nothing and a cap of 120 does nothing. On a faster display
the cap holds the rate under the refresh. With `--no-vsync` the cap is the
only limit, which is how the perf runs show it. A vsync toggle is a separate
change (proposal).

**Live.** The next frame uses the new period. OFF removes the wait.

**Cost.** None on the GPU. The suite's 60 and 120 runs measure what it does
to pacing: p50 near 16.7 and 8.3 ms, and the spread.

**Not in a capture.** `--capture` steps the simulation in fixed steps and
does not want wall-clock pacing, so the pacer stands down whatever the flag
says.

## 4. Render scale: 100, 85, 75, 67, 50 %

**Default 100 %**, which is today.

**How.** At 100 % nothing changes: every 3D camera renders to the window,
as now, and nothing is added. Below 100 %:

1. An offscreen image of `round(window physical size * scale)` is made
   (`Rgba8UnormSrgb`, render attachment and texture binding). It is remade
   when the scale or the window's size changes.
2. Every `Camera3d` gets `RenderTarget::Image` of it. That is the walking,
   skiff and vehicle cameras, and a capture's own camera. Only one is active
   at a time (`walking::View`).
3. A **present camera** (`Camera2d`, order 1, on the window) draws one sprite
   of the image, stretched to the window, with **bilinear** filtering.
4. The UI draws on the present camera, because Bevy's default UI camera is
   the highest-order camera on the window. So the HUD and the menus stay at
   the window's full resolution: text and pixel icons are never upscaled.

Back at 100 %, the cameras target the window again and the present camera is
despawned, so the UI returns to them.

**How the passes inside the view stack.** Every pass is sized from the
view's target, which is now the smaller image, so each internal scale
multiplies:

| Pass | Its scale | At 100 % | At 67 % |
| --- | --- | ---: | ---: |
| The view | `render_scale` | 1.00 | 0.67 |
| Rain volume | `rain_render_scale` 0.5 | 0.50 | 0.335 |
| L8 traces (`reflect_half`) | `reflect_render_scale` 0.5 | 0.50 | 0.335 |
| Clouds and their history | `cloud_render_scale` 0.4 | 0.40 | 0.268 |
| Screen trace history (`planet_ssr`) | fixed 960 x 600 | same | same |

Each fraction is of the view, so one control halves the work everywhere, and
nothing in `assets/config` has to know about it. What it does to the clouds'
history: the history is sized from the view and is remade when the size
changes, as on a window resize today. So a change of scale drops the cloud
history once, the same as a resize. The terrain's LOD reads only the camera's
position, never the screen size (checked in `planet_lod.rs`), so every scale
draws the same geometry.

**Cost.** Fragment work drops about as the square of the scale (67 % is 45 %
of the pixels). Below 100 % one fullscreen sprite is added (a bilinear read of
a small image). The 67 % suite run measures it.

**Not in the view's own maths.** `world_to_viewport` and the like are not
used by the HUD (the compass and drops read camera transforms; picking is
UI-only), so nothing has to map between image and window coordinates. A
later caller that projects through the 3D camera into the UI must scale by
`window / image`, and the module says so.

**Found in play (owner, 2026-11-13, build `47e9c07`: "ui buttons not
working, a lot of ghosting"), and fixed:**

- **Dead buttons.** `walking.rs` puts `IsDefaultUiCamera` on the active
  view's camera, so below 100 % the HUD stayed on a camera that renders to
  the image. `ui_focus_system` takes cursor positions only from cameras that
  render to a window, so no button saw the pointer. The menu still drew,
  stretched with the view. Now the present camera holds the marker while the
  view is scaled; it is taken off any 3D camera a mode switch puts it on, and
  given back to the active view at 100 %. A test pins both.
- **The view's size after a change of target.** Bevy recomputes a camera's
  target size only on a window event, an image event or a projection change,
  never on a new `RenderTarget` alone. Going back to 100 % left the camera
  sized for the image. The scale now touches `Projection` whenever it
  retargets a camera, and runs before `CameraUpdateSystems`, so the new
  target is sized the frame it is set. Whether this is also the ghosting
  is to be confirmed by a capture against the `47e9c07` exe.
- **A crash launching at a reduced scale** (`cfd4a32`, "Attachments have
  differing sample counts"). Bevy keys a 2D camera's depth texture by its
  render target alone, not its MSAA. At launch the loading screen's
  `Camera2d` (Bevy's default 4x) and the present camera (`Msaa::Off`) were
  both on the window, so they shared a 4x depth while the present pass drew
  1x colour. It never showed in play because the scale was changed after
  loading. The present camera now takes the default MSAA, like every other 2D
  camera on the window. The cost is a 4x colour buffer at window size for one
  sprite and the UI.

## 5. Auto step: ON, OFF

**Default ON**, which is today: a rise up to `column::STEP_M` (1.05 m, one
terrain layer and the skin) is walked up without a key.

**OFF:** the walker climbs, on its own, only what a stair asks of it: rises
up to `MANUAL_STEP_M = 0.35 m`. That is `HOLD_M`, the walker's
stair-holding reach. Town stair risers are 0.19 to 0.25 m, and a deck's stair
is the same. So stairs, flights and ship stairs still walk. A terrain layer
(1 m) is a wall until the player jumps. The jump already clears it: the sweep
lets horizontal motion through once the feet are above the ledge.

**What moves.** Only the climb test (`can_step` in `sweep`) reads the
setting. How far up a town floor is looked for (`reach`) and the cave mouth
rule keep `STEP_M`. So with auto step OFF, a doorway is still a doorway and a
floor the walker stands on is still found. Swimming is unchanged: a swimmer
rises with the jump key held, as now.

**Live.** The next fixed tick uses it (`WalkingConfig::auto_step`, set from
the settings each frame).

## 6. The upscale filter: to the owner, with screenshots

Below 100 % the image can be stretched **bilinear** (soft, no shimmer at 67
%, the uneven ratio) or **nearest** (crisp, but at a ratio that is not a
whole number, rows and columns are doubled unevenly). Bilinear is the
recommendation and ships as the default. The survey asks with the 67 %
captures of both side by side (`compare` viewer). `--upscale nearest` exists
for that capture and for the owner's own A/B.

## 7. Later: TAA and DLSS

**The owner (2026-11-13): "alright no taa or dlss for now, we can install
those later".** Not built. What each needs, as found while writing this:

**Both** need correct motion vectors for everything on screen, and the
engine has none for most of it:

- **The terrain** draws in its own node (`planet_reflect.rs`, between
  `MainTransmissivePass` and `MainTransparentPass`), not in Bevy's prepass. It
  writes no motion vectors and no prepass depth. It projects with
  `ExtractedView::clip_from_view`, which is unjittered, so it would ignore
  the camera jitter both techniques depend on. A depth-reprojection pass
  (camera motion from depth and last frame's matrix, written where no mesh
  wrote) would cover static terrain and the sky. The held item and anything
  else carried with the camera would need their own.
- **The clouds** keep their own history (`planet_water.rs`, the
  `cloud-history-clip` and `cloud-ghosting` resolve). A second history on top
  (TAA's) would carry cloud across silhouettes, which is priority 2 ("the
  history blend must never carry cloud across a silhouette"). The clouds
  would have to be composited after TAA, or their resolve made the only
  history they go through.
- **MSAA off.** Bevy's TAA requires `Msaa::Off`, and the camera runs
  `Msaa::Sample4`. The terrain pipeline and the water and SSR passes already
  have non-MSAA variants. But the L8 traces at half size
  (`lighting-perf` 2.8) only split under MSAA (`reflect::SAMPLES`), so without
  it the traces go back inline at full size, which is a measured cost.

**DLSS** also needs (Bevy 0.18.1's `bevy_anti_alias/src/dlss/mod.rs`, read in
the local cargo registry, and the `dlss_wgpu` README):

- Bevy's `dlss` cargo feature, which pulls `dlss_wgpu` 2 and
  `bevy_render/raw_vulkan_init`;
- the **NVIDIA DLSS SDK**, which `dlss_wgpu` cannot redistribute: cloned
  from NVIDIA, with `DLSS_SDK` set to its absolute path, plus `VULKAN_SDK`
  and clang to build;
- the owner's acceptance of the **NVIDIA DLSS SDK licence**, and shipping
  `nvngx_dlss.dll` with its licence text (section 9.5 of the SDK's
  programming guide);
- the **Vulkan** backend (Windows or Linux) on an RTX GPU, with a
  `DlssProjectId` inserted before `DefaultPlugins`;
- every custom pass honouring `MainPassResolutionOverride`: DLSS renders
  the main pass smaller and upscales, so the terrain node, the water and
  cloud composite, rain and the reflect passes would each have to size
  themselves from it.

None of that was downloaded or accepted. It is the owner's call. DLSS is not
in the menu: a row that can only say "unavailable" is text on a screen that
is meant to be quiet (`menus-and-a-quiet-screen`). **Recommendation taken
(ask only with screenshots).**

## Risks

- **The present camera changes who draws the UI** below 100 %. A test checks
  that at 100 % no present camera exists and the 3D cameras target the
  window, which is the "as today" promise.
- **A busy-wait in the pacer** is up to 1.5 ms of a core a frame. It runs
  only with a cap set.
