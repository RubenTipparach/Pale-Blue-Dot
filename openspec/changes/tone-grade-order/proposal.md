# Proposal: the tone grade runs after tonemapping, every frame (ghosting)

## Why

The owner (relayed from the desktop session, 2026-11-13): turning the view in
a clear sky smears sky objects into trails of repeated copies - the moon or a
dark puff leaves ten or more fading copies, stars become dotted arcs - and it
"makes the game unplayable".

Reproduced on the Mac in a fixed-step capture (`--walk --at 20 40 --time 2
--pitch 40 --turn 45 --frames 240`: on foot, 02:00 on the planet clock, sun
30 degrees under the horizon, `Weather here: cover 0.00, rain 0.00`), and
bisected (design):

- main before the lighting model (`db25fe2`): clean;
- `588ded9` "The new lighting model in the game, every part" (PR #27): smears;
- every later commit, main `f15a647` and `claude/perf-tuning` `b76c3a9`
  included: smears.

So it came with the lighting model's display transform (L6), not with the
perf work on `claude/perf-tuning`, which only carried it.

## What changes

- `ToneGrade`'s `FullscreenMaterial` names its sub-graph (`Core3d`), so Bevy
  adds its node to the 3D graph once, when the plugin is built, with both of
  its edges. No shader, no setting and no default changes.

## Not in this change

- The cloud history's own blend (`CLOUD_BLEND`) and reprojection: switching
  the clouds off (`PBD_NO_CLOUDS=1`) left the trails as they were, so they are
  not the cause.
