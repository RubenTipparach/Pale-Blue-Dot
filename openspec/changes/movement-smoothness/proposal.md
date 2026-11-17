## Why

The owner, 2026-11-17: "objects kind move with some weird jitter, the player
included, the fishing line, the fish biting. Is there some floating point
issue? Are you not using floating origin so animations and movement remain
smooth?" And: "its not frame rate, my camera moves fine".

Measured (design 2), two causes:

1. **Render precision.** There is no floating origin. Render space is the
   planet's own frame, centre at the origin, so the camera and everything near
   the player sit about 4 800 m from it, where an f32 holds 0.49 mm. Bevy's
   mesh path rounds each world position there and multiplies it by a view
   matrix carrying the same 4 800 m. A thing drawn half a metre from the eye
   (the held tool and hand) is drawn up to a millimetre off its pose, and the
   error changes every frame. The held tool shimmers 2.6 px rms, 6.2 px p99,
   from one frame to the next while walking. With the origin rebased to the
   camera, the same frames give 0.004 px.
2. **The fish step at 30 Hz.** Schools run on their own 30 Hz clock and were
   drawn at the raw step. While the camera moves smoothly, a fish stands still
   for whole frames and then jumps: in 37% of the fishing trace's frames the
   fish did not move at all. A fish held at the lure (nibbling, biting, on the
   line) was put there by that step too, so it trailed the float, which moves
   every frame.

What was ruled out, with numbers (design 2): late transforms (the tackle's
frame-late camera was the only one, and it was already fixed), a large f32
clock (every clock that moves the water, the float or the fish is folded in
f64 or wrapped), and a rebase spike (nothing rebases).

## What Changes

- **Built in this change:**
  - A school is drawn between its last two steps, at the fraction its clock
    has run. The fish on the line is drawn at the lure this frame.
  - The `PBD_MOTION_TRACE` measurement instrument and `tools/motion_graph.py`.
- **Proposed, not built:** a camera-relative render origin (design 4). The
  render origin follows the camera, and physics and simulation stay in the
  planet's frame, so the walker's `up = position.normalize()` and the other
  places that assume a planet-centred frame keep working. It is a broad
  change, so it waits for a go-ahead.
- Unchanged: mouse look stays this frame's raw displacement, and the walking
  camera keeps following the body directly (the owner: "my camera moves
  fine"). Design 3 records why camera interpolation is left as an option.

## Capabilities

### Added
- `platform/render-precision`: what is drawn near the eye is drawn where it
  is, to well under a pixel (proposed; design 4).
- `player/fishing`: a school is drawn between its steps (built).

## Impact

- `crates/pbd-core/src/fauna/school.rs`: `School::before`, `School::drawn`.
- `crates/pbd-app/src/fish.rs`: `Fishery::step_alpha`; schools are built from
  `drawn`.
- `crates/pbd-app/src/motion_trace.rs`, `tools/motion_graph.py`: measurement
  only. Unset, nothing changes.
- `crates/pbd-app/src/desktop.rs`: in a real-time run (no `--capture`),
  `--fish` counts its frames from `READY` and casts again while the line is
  in. Captures are unchanged.
- Saves: unaffected. Schools are not saved.
