# Proposal: player settings - a frame limit, a render scale, auto step

## Why

**The owner's words (2026-11-13):** "can you add some settings:
* frame limit to 60 fps or 120 fps
* resolution scaling
* temporal AA
* dlss?
* toggle auto jump on hex step"

And, the same day, before any of it was built: **"alright no taa or dlss for
now, we can install those later".** This change builds the other three and
writes down what the two deferred ones need (design, "Later: TAA and DLSS"),
so they are put off and not forgotten.

What exists today:

- The settings page (`Escape`, then SETTINGS) has one GRAPHICS row, BLOOM,
  kept in `saves/preferences.ron`, and the INPUT binding table.
- Nothing caps the frame rate but vsync (`AutoVsync`, or `--no-vsync`).
- The 3D view always renders at the window's full size. Some passes inside it
  already run smaller (`rain_render_scale`, `reflect_render_scale`,
  `cloud_render_scale`), as fractions of the view.
- The walker climbs any rise up to `column::STEP_M` (1.05 m) on its own: one
  terrain layer is walked up without a key. Nothing turns that off.
- Every tunable in `assets/config/*.ron` is the developer's, read at startup.
  None of it is the player's.

## What

**A per-player settings file**, outside the repository's `assets/` and
outside `saves/`: `settings.ron` in the user's config directory. It is
versioned. A missing, unreadable or older file gives defaults, and it is
written only when the player changes something. It never reads or writes a
save. Captures and benchmark runs (`--capture`, `--frame-log`) do not read
it: they take defaults and the flags they are given. Then a picture or a
perf number stays a function of its command line.

**Three settings on the existing settings page.** Each row is a button that
cycles its value and says the value, like BLOOM does:

| Setting | Values | Default | Applies |
| --- | --- | --- | --- |
| FRAME LIMIT | OFF, 60, 120 | OFF (as today) | next frame |
| RENDER SCALE | 100, 85, 75, 67, 50 % | 100 % (as today) | next frame, and on resize |
| AUTO STEP | ON, OFF | ON (as today) | next tick |

With defaults the game draws and runs exactly as it does now: no extra
camera, no extra pass and no sleep. **The frame cost was not measured.** The
owner, 2026-11-13: "ok skip the benchmark, just build so I can play". The
planned sitting (old exe against new, interleaved, plus one run each at 60,
120 and 67 %) stays an open task.

## Not in this change

- **TAA and DLSS**, by the owner's word. Their needs are in the design.
- **A vsync toggle.** Present mode stays as today: vsync, or `--no-vsync`.
  The design says how the frame limit sits with each.
- **Moving BLOOM into the new file.** It stays in `saves/preferences.ron`,
  where captures already read it. Moving it would change what a capture or a
  perf run draws on this machine. That is a follow-up with its own
  before-and-after.

## Impact

- `crates/pbd-app/src/settings.rs` (new): the file, its defaults and
  version, the frame pacer, and the render-scale target.
- `crates/pbd-app/src/walking.rs`: `WalkingConfig::auto_step`, and the step
  test reads it.
- `crates/pbd-app/src/desktop.rs` and `desktop/menu.rs`: the flags, the
  rows, and wiring.
- No save format, generator, shader or `assets/config` change.
