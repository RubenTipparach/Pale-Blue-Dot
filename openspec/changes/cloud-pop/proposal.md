# Proposal: a cloud tower grows without re-forming (cloud-pop)

## Why

The owner (2026-11-13): "several visual popins: clouds, terrain, and baked
lighting" and "we need some way to better transition those". CLAUDE.md
priority 1: nothing appears or changes in one frame.

The puffs (`cloud-puffs`) are rebuilt every frame from the weather maps the
view shows, mixed by the blend's `t`, so a site's cover and its cloud's top
move a little each frame. Most of what a site makes follows them smoothly
(its presence, radius, opacity, darkness). One thing does not
(`pbd_core::cloud_puffs::puffs_at`): a site's column is
`stack = ceil(height / (1.4 r))` billboards, each `height / stack` tall. When
the top crosses a step, the stack goes from k to k + 1 billboards and every
billboard in the tower is resized and moved in that frame: a tower of two
1.4 r billboards becomes three of 0.93 r, centres shifting by up to a third
of a billboard (design). A test shows the jump at the step.

## What changes

- The billboards keep one spacing, `max(1.4 r, height / 3)`, and grow up the
  column: billboard k is present by `clamp(height / spacing - k, 0, 1)`, so a
  growing tower's top billboard fades in rather than every billboard
  re-forming; its centre stays inside the column. The lowest is always
  whole. Nothing else about a puff changes.
- A test: every billboard's centre, size and opacity move by a bounded amount
  for a small change of the top, across the whole range of tops.

## Not in this change

- Overlapping puffs swapping draw order as the eye moves (sorted by centre
  distance): measured separately; the outline where puffs overlap is its own
  fix.
- The terrain's and the baked light's pop-ins (their own changes).
