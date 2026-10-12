# Cloud ghosting: the history never carries cloud across a silhouette

## Why

Owner priority 1 (`CLAUDE.md`, 2026-09-25): clouds smear and ghost around the
objects in front of them. The owner reported it again after `cloud-budget`
landed ("lots of ghosting still").

`cloud-budget` fixed WHERE the history is reprojected from: the cloud's own
coverage-weighted depth, not the middle of the span. It never asked WHETHER the
texel it reads saw the same thing. The march blends 94% of the previous
frame's texel into every frame, whatever that texel held. So:

- sky revealed from behind a hill, a tree, the held tool or a boarded craft
  fades its cloud back in over about 0.2 to 0.6 s, leaving a cloudless print
  of the object trailing it;
- cloud beyond a new occluder is carried in front of it;
- the bilinear read averages across the edge and holds that for sixteen frames.

The held tool and a craft are fixed on screen, so any turn of the view does
this all along their outline, every frame. See `design.md`.

## What

1. The march keeps last frame's distances (the cloud's distance and where the
   march stopped) as a pair that flips with the history, and last frame's eye
   as a uniform lane.
2. The history is read as four texels. Each is kept only when last frame's
   march at that texel saw what this frame's sees: the upsample's own depth
   test, through one shared function. The kept texels are renormalised, and
   a texel with none kept stands alone for that frame.
3. `--turn DEG_PER_S`, a capture instrument that turns the walker's view at a
   steady rate, so a still capture can show what a moving view leaves behind.

## Not in scope

The blend factor, neighbourhood clamping, and clouds up close or from inside
them (priority 2) are separate.
