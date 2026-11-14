# Proposal: a walk along the coast for the proof video (coast-walk)

## Why

The owner (2026-11-13), after the day's fixes (ghosting, dusk, stars, lens
drops, cloud outline, water, cloud, terrain and light pop-in): "record video
so that we can prove this all works. do a 2km walk, walk not run, and do try
to navigate somewhat instead of running in a straight line, such as following
a coast and going over rivers, avoiding cliffs etc".

The scripted walks today go straight: `--walk-distance` sprints ahead and digs
through what stops it; `--stroll` paces one lane. Neither follows a coast,
crosses rivers or keeps off cliffs.

## What changes

A recording instrument, `--coast-walk METRES` (with `--walk --at`): the walker
walks (no sprint) along the shore with the sea on one side, crosses a river
where it meets one, steps round cliffs, never digs, and logs `COAST_START`,
`COAST_RIVER` and `COAST_DONE`, then quits. It steers by the generator's
height field (`surface_height`), not by what is drawn, so a simulation of the
same steering over the same height field can choose the start and prove the
route before anything is recorded. It changes nothing in play.
