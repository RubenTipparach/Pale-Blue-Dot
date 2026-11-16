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
the frame limit and render scale; no rebinding, which is new behaviour; section 2 below adds both), the
loading slab has no Play button (the game starts itself), and the map's
drawer is always open.

`LATE_PIPELINE` in these logs: a `--capture` lifts the loading screen on its
first open frame without warming, so every capture logs late pipelines, before
as after. The kit adds one, `ui_texture_slice_pipeline`. See the real-time run
below for a launch that warms.

**The real-time run.** `realtime-pause-after.jpg` is the same pause shot with
`--real-time --frames 60`: the launch warms behind the loading screen as a
player's does, then the shot is taken. Its log has `READY playable 112.80 s
... (warm 90.23 s, 78 pipelines)` and one `LATE_PIPELINE` after it,
`screenshot-to-screen`, the capture's own instrument. No UI pipeline compiles
late: the kit's `ui_texture_slice_pipeline` is compiled during the warm by the
loading slab's sliced bar, on the game's camera. On lavapipe the warm ran
into its 90 s cap rather than going quiet, so this is a check, not a proof;
the owner's desktop run is the proof.

## Section 2: every screen by day and by night (2026-11-14)

The branch after section 2 (Input in sub-tabs with rebinding, the stone
sliders, the kit's tooltips), made the same way: a Claude Code cloud
container with **no GPU**, Mesa's lavapipe under `xvfb-run` at 1440 x 900
(UI scale 2), the **`--profile fast`** build, headless `--capture`, a new
memory-only world, walking, facing compass north, clear weather. A cloud
capture is a fixed-step render: it shows what the screens look like, never
how fast they run. **The frame cost was not measured** (`perf_suite.py` is
for the owner's desktop). Day is `--time 12`, night `--time 0`:

    xvfb-run -a -s "-screen 0 1440x900x24" target/fast/pbd-app --capture <shot>.png \
      --walk --yaw 90 --pitch 4 --rain 0 --weather-at 7200 --frames 90 --time 12|0 <flags>

The instruments these shots needed (none changes play): `--menu guide`;
`--settings-tab TAB` and `--input-tab SUB` (the tab the settings open on);
`--rebind SLOT=KEY` (a rebind for this run only: a capture reads no settings
file); `--cursor X Y` (the pointer held there, so a headless run can hover a
slot for its tooltip).

| Shots | Flags | What it shows |
| --- | --- | --- |
| `s2-pause-{day,night}.jpg` | `--menu pause` | Pause in the carved oak, Resume with the rune focus |
| `s2-saves-{day,night}.jpg` | `--menu saves` | The worlds page in oak: the name field, New world, Back |
| `s2-settings-picture-{day,night}.jpg` | `--menu settings --settings-tab picture` | Bloom's checkbox; the frame limit and render scale as stone sliders with oak grips and their values beside them (the value 4 px clear of the grip at the top step) |
| `s2-settings-weather-{day,night}.jpg` | `--settings-tab weather` | The rain shadow (off by default) and the storm bar. Since `reflection-quality` (2026-11-16) the rain shadow is on PICTURE: `--settings-tab picture` shows it, beside Reflections |
| `s2-settings-world-{day,night}.jpg` | `--settings-tab world` | The clock's bar with Day and Night, the globe overlay |
| `s2-settings-movement-{day,night}.jpg` | `--settings-tab movement` | Auto step |
| `s2-settings-input-{day,night}.jpg` | `--settings-tab input --input-tab on-foot` | Input's five oak sub-tabs, On foot open; each changeable key an oak plaque, Reset tab |
| `s2-input-{hands,flying,vehicles,menus}-day.jpg` | `--input-tab hands` etc. | The other sub-tabs: the mouse, wheel and number row in flat caps (not changeable); Flying holds the Kestrel's group, Vehicles each boat's, Menus the world and screen keys with Esc fixed |
| `s2-input-rebound-day.jpg` | `--input-tab on-foot --rebind on_foot:ShiftLeft=KeyE --rebind on_foot:KeyW=ArrowUp` | A rebind: move reads ↑ A S D, sprint reads E and says "also: open or shut a door" |
| `s2-pack-{day,night}.jpg` | `--menu pack --cursor 600 537` | The pack, the pointer on the hotbar's stone: the tooltip "Stone x 48" stands above the slot, clear of the counts |
| `s2-tool-tip-day.jpg` | `--menu pack --cursor 448 857` | The tool slot's tooltip, "Bare hands: breaks nothing", above the HUD's tool slot |
| `s2-hud-{day,night}.jpg` | (none) | The quiet screen: compass, hotbar, tool slot; no tooltip in play |
| `s2-map-{day,night}.jpg` | `--menu map --map-mpp 12` | The map frame and its drawer, open by default as before (shutting it by default is a recommendation, not done) |
| `s2-guide-{day,night}.jpg` | `--menu guide` | The field guide on its stone |
| `s2-loading-{day,night}.jpg` | `--capture-loading` (and `--time`) | The loading slab while the planet builds |
| `s2-realtime-input-day.jpg` | `--menu settings --settings-tab input --real-time --frames 60` | The LATE_PIPELINE check below |

### Corners: 28 px, pixel exact at scale 2

Measured off the PNG of `s2-settings-input-day` (the oak frame, 324 native
px wide) against the frame as the game bakes it at scale 2 (`cargo test
--profile fast -p pbd-app --lib -- --ignored dump_the_frames_baked_at_scale_two`
writes it to `target/kit-dump/`):

    python3 tools/art/measure_corners.py <capture.png> target/kit-dump/oak-frame@2.png --tolerance 0

With no tolerance at all, every opaque texel of both 28 x 28 corner blocks
is drawn exactly, and the panel is drawn 648 px wide, its 324 native px at
scale 2. At 30 px the bottom-right block, where both stretched sides meet
it, matches 13.6 %: the corner ends at 28 px. (The top-left block still
matches 95 % at 30 px because the top side's first columns sample nearly
the same texels.) Measured on the PNG; the JPGs here are for looking at.

```
capture s2-settings-input-day (the PNG before JPG) 1440x900
baked target/kit-dump/oak-frame@2.png 96x96, corner 14 native px x 2 = 28 px
  N=26 top-left     opaque  667  match 100.00 %  at (396, 163)
  N=26 bottom-right opaque  667  match 100.00 %  at (1018, 711)
  N=28 top-left     opaque  775  match 100.00 %  at (396, 163)
  N=28 bottom-right opaque  775  match 100.00 %  at (1016, 709)
  N=30 top-left     opaque  891  match  94.95 %  at (396, 163)
  N=30 bottom-right opaque  891  match  13.58 %  at (444, 707)
  N=32 top-left     opaque 1015  match  90.54 %  at (396, 163)
  N=32 bottom-right opaque 1015  match  18.92 %  at (899, 705)
panel drawn 648 x 574 px from (396, 163)
corners 28 px pixel exact: True; the bottom-right block at 30 px matches 13.58 % (the stretched sides begin)
```

### LATE_PIPELINE

`s2-realtime-input-day.jpg` is the Input page with `--real-time --frames
60`: the launch warms behind the loading screen as a player's does. Its log:
`READY playable 161.89 s after launch ... (world 68.79 s, warm 92.80 s, 78
pipelines)`, no `LATE_PIPELINE` before it, and one after it,
`screenshot-to-screen`, the capture's own instrument. Nothing the UI draws
(the sub-tabs, the caps, the sliders, the tooltip's stone) compiles late. On
lavapipe the warm runs into its 90 s cap, so this is a check, not a proof.
The fixed-step shots above lift the loading screen without warming and so
log late pipelines as every `--capture` does (section 1 above).
