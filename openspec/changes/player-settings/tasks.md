# Tasks

## 1. Write-up
- [x] Proposal, design, spec deltas and tasks; TAA and DLSS recorded as
      later, in the owner's words.

## 2. The settings file
- [ ] `pbd_app::settings`: `PlayerSettings` (version, frame limit, render
      scale, auto step), the config-directory path, `PBD_SETTINGS`.
- [ ] Load: missing gives defaults; a broken file gives defaults and is left
      as it was; a file with no version is read as version 0; a newer file is
      read and never written; a scale off the list snaps to it.
- [ ] Keep: written only on a change.
- [ ] Tests: round trip, defaults, missing, broken, versionless, newer, snap.

## 3. Frame limit
- [ ] `FramePacer`: deadlines a period apart, re-anchored after a hitch,
      sleep then spin. Runs last in `Last`; stands down in a capture.
- [ ] Tests: pacing over a made-up clock; a hitch re-anchors; OFF never waits.

## 4. Render scale
- [ ] Below 100 %: the offscreen image, `RenderTarget::Image` on every
      `Camera3d`, the present camera with a bilinear sprite; remade on a
      resize or a new scale. At 100 %: back to the window, present camera gone.
- [ ] `--upscale nearest` for the filter comparison.
- [ ] Test: at 100 % no present camera, and the cameras target the window.

## 5. Auto step
- [ ] `WalkingConfig::auto_step`, and `MANUAL_STEP_M`; only `can_step` reads it.
- [ ] Tests: on climbs a layer, off stops at it, off with a jump climbs it, and
      off still climbs a flight of stairs.

## 6. Menu and flags
- [ ] FRAME LIMIT and RENDER SCALE rows under GRAPHICS; a MOVEMENT section
      with AUTO STEP; labels from one system.
- [ ] `--frame-limit`, `--render-scale`, `--no-auto-step`; captures and
      `--frame-log` runs skip the file.

## 7. Look check
- [ ] GPU captures at 100 % and 67 %, bilinear and nearest, into
      `docs/screenshots/player-settings/`.
- [ ] Survey question: the upscale filter, with the `compare` viewer.

## 8. Perf
- [ ] `tools/perf_suite.py`, release, quiet machine: old exe against new,
      interleaved, defaults; one run each at 60, 120 and 67 %. Report in
      `docs/benchmarks/2026-11-13-player-settings/`.

## 9. Ship
- [ ] fmt, clippy, tests, `openspec validate --all`; the requirements move to
      `openspec/specs` with the tests that prove them.

## Later (owner, 2026-11-13: "no taa or dlss for now")
- [ ] TAA: motion vectors for the terrain node and the clouds' history, the
      jitter in the terrain's projection, and MSAA off against the
      half-size traces (design 7).
- [ ] DLSS: the above, plus the SDK, the licence, Vulkan, and
      `MainPassResolutionOverride` in every custom pass (design 7).
