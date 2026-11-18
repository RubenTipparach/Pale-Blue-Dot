# Proposal: a swing that strikes

## Why

The owner, 2026-11-17: "man that axe swing is weak, it should be more
visceral, more faster".

## What Changes

- The chop is no longer one sine wave on its own clock. Each stroke has
  stages: a quick wind-up, a strike that speeds up into the block, a short hold
  on impact, a small kick back and an eased return.
- Strokes are timed to the dig at one steady beat (design 5): a block with
  break time T and the pause after it get N = max(1, round((T + 0.15 s) /
  0.4 s)) strokes of one period, the last on the break. Break times do not
  change.
- The dig sounds play on the impacts, from the same schedule.
- Letting go eases the tool back to rest instead of snapping it.
- Optional, the owner's call: a small camera kick on impact (added on top of
  the view, never touching the raw mouse look) and a swing whoosh.

Shown first in the Dig and Chop tabs of the Tools in Hand mockup (M36,
`docs/mockups/tool-motion.html`, https://claude.ai/artifact/CwwZtbNGjZSVtjivQAMuzL; the owner on 2026-11-17: "make a new
mockup for all the tools stuff for digging, choppin and fishing"), as the
STRIKE stroke beside today's chop, running the same curve the game would. The
game changes only after the owner approves it.
