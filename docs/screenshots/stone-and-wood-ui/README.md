# Stone and wood UI: before and after (`stone-and-wood-ui`, tasks section 1)

The game's real screens before the kit (commit `7e2e4c8`, the branch's head
before section 1) and after it (the smooth rock in play, the smooth carved oak
out of play; owner, 2026-11-14: "Smooth is the best").

**How they were made.** A Claude Code cloud container with **no GPU**: Mesa's
software Vulkan (lavapipe, llvmpipe), under `xvfb-run` at 1440 x 900, the
`--profile fast` build of each commit. A window 900 px tall gives a UI scale
of 2. A cloud capture is a fixed-step render: it shows what the screens look
like, never how fast they run; the frame cost was not measured
(`perf_suite.py` is for the owner's desktop). Every shot is a new
memory-only world, walking, facing compass north, in clear weather:

    xvfb-run -a -s "-screen 0 1440x900x24" target/fast/pbd-app --capture <shot>.png \
      --walk --yaw 90 --pitch 4 --rain 0 --weather-at 7200 --frames 90 <flags>

| Shot | Flags | What it shows |
| --- | --- | --- |
| `hud-day-{before,after}.png` | `--time 12` | The quiet screen at noon: crosshair, hotbar and tool slot, compass. After: the light dressed-stone compass strip with its letters cut in, N inlaid red, the oak pointer, the town studs and the town named under it in the kit's font; the hotbar as stone wells, the selected one ringed with the rune, counts in the corner outlined in ink; the near-field readout under the compass, not behind it |
| `hud-night-{before,after}.png` | `--time 0` | The same at midnight: the UI is unlit, the same by night as by day (design 6) |
| `pack-{before,after}.png` | `--time 12 --menu pack` | The pack. After: the base rock baked at the pack's own 268 x 152, the oak lintel PACK, thirty stone slots and the hotbar row; no close button and no instruction line |
| `pause-{before,after}.png` | `--time 12 --menu pause` | Pause. After: the carved-oak frame and lintel, four oak plaques, Resume with the rune focus; the world dimmed behind it. The weather, clock and globe-overlay controls moved to Settings, so the panel fits a 900 px window |
| `settings-{before,after}.png` | `--time 12 --menu settings` | Settings. After: oak frame, oak tabs (Picture open, with its cap), the bloom checkbox, the frame limit and render scale as oak plaques, Back |
| `map-{before,after}.png` | `--time 12 --menu map --map-mpp 12` | The map. After: the stone moulding with the rock corner blocks at the window's edge, the legend as a stone drawer with stone plaques (the chosen one pressed), the readout on a stone tooltip |
| `loading-{before,after}.png` | `--capture-loading` | The loading screen while the planet builds. After: the rock slab baked at 150 x 148, the emblem, PALE BLUE DOT carved, the step in VT323, the stone band with its progress fill |

Compared against the mockup's Smooth look (`docs/mockups/ui.html`,
`docs/screenshots/mockup-ui/compare/`): the pause, pack, compass, hotbar and
loading slab match it piece for piece. Known differences, on purpose: the
settings page keeps the game's own controls (value plaques, not sliders, for
the frame limit and render scale; no rebinding, which is new behaviour), the
loading slab has no Play button (the game starts itself), and the map's
drawer is always open.

`LATE_PIPELINE` in these logs: a `--capture` lifts the loading screen on its
first open frame without warming, so every capture logs late pipelines, before
as after. The kit adds one, `ui_texture_slice_pipeline`. See the real-time run
below for a launch that warms.
