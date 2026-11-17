# rod-motion

The rod in hand moving with the line, and the back of the hand shaded by its
true face normals (`openspec/changes/rod-motion`, tasks 1.2 and 2.5).

| Shot | Before | After |
| --- | --- | --- |
| The hand close (2x crop of the rest frame) | `hand-before.jpg` | `hand-after.jpg` |
| Rod at rest, the float not yet cast | `rest-before.jpg` | `rest-after.jpg` |
| Charging the cast, the button held | `charging-before.jpg` | `charging-after.jpg` |
| Just after the cast, the float in the air | `cast-before.jpg` | `cast-after.jpg` |
| The float on the water | `floating-before.jpg` | `floating-after.jpg` |
| A bite: the tip dips | (the still rod of `floating-before.jpg`) | `bite-after.jpg` |
| A fish on, reeling: rod up, rolled toward the pull, shaking | (the still rod of `floating-before.jpg`) | `hooked-after.jpg` |

- Before is `db8f17e8`, the commit before this change: the rod stands still
  in every phase, so a bite and a fight look like the float shot. Its
  `held.ron` is that commit's (the new one has the `rod` block it would
  refuse).
- After is this change: the charge draws the rod back over the shoulder, the
  cast flicks it forward, it settles forward of rest while the float is out,
  dips on a bite, and is held up with a fish on.
- The hand: before, the slanted faces on the back of the hand were shaded as
  if they pointed sideways, so the back read as patchy bands; after, each
  face is shaded by its own normal.

How they were taken:
- In-game captures on 2026-11-17 in a cloud container with no GPU, on
  lavapipe (software Vulkan), 1440 x 900, the `fast` build, a new
  memory-only world. They show how things look, never how smoothly: frame
  cost was not measured.
- One run per row group, each saving a burst of frames along the way with
  `PBD_CAPTURE_RUN=<dir>,<first>,<count>,<stride>`; the stills are frames of
  those bursts.
- Logs: no `ERROR` lines. 37 `LATE_PIPELINE` lines in each run, before and
  after alike (the same pipelines; lavapipe compiles late), none from this
  change.

Launch lines (on a river bank, facing east across it):

    PBD_CAPTURE_RUN=bite-burst,0,520,8 pbd-app --walk --at 28.25 0.63 --yaw 180 --pitch -15 --cast --time 11 --rain 0 --weather-at 7200 --capture after-bite.png --rod-cue bite --frames 520
    PBD_CAPTURE_RUN=hooked-burst,200,140,10 pbd-app --walk --at 28.25 0.63 --yaw 180 --pitch -15 --cast --time 11 --rain 0 --weather-at 7200 --capture hooked-after.png --rod-cue hooked --frames 340
    PBD_CAPTURE_RUN=before-burst,0,240,8 pbd-app --walk --at 28.25 0.63 --yaw 180 --pitch -15 --cast --time 11 --rain 0 --weather-at 7200 --capture before-float.png --frames 240

`--cast` takes up the rod and casts once (a capture instrument).
`--rod-cue nibble|bite|hooked` is a capture instrument added for these
stills: no fish is near this bank, so a second after the float lands it
draws the rod as if the line were in that phase (for `hooked`, at tension
0.7, reeling, the fish pulling right). It writes only the rod's cue, which
nothing but the drawn rod reads: the line, the float, the catch rules and
the fish are untouched, so in `bite-after.jpg` and `hooked-after.jpg` the
float still floats.
