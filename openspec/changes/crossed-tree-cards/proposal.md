# Proposal: far trees as fixed crossed cards, not billboards

## Why

The owner, 2026-11-17, with a screenshot from high up over forest: "I dont
really like trees being bilborard, 2 - 4 billboards with no rotation is
probably mocre preferd". In the shot every far tree is one card turned to
face the camera. Seen from above, a card turned to the eye lies along the
line from the view centre to the tree, so the forest smears into long
streaks radiating from the middle of the screen, and they swing as the
camera moves.

## What was found

- The far ring is a camera-facing billboard: `planet_surface.wgsl`, the
  tree branch of `vertex`, under "The far billboard, turned to the eye",
  sets the card's axis from `dir`, the eye's bearing in the tree's frame,
  and picks the baked view nearest that bearing. It is drawn past
  `tree_mid_m * 0.85` (76 m with the default 90 m) out to the trees' range.
- The mid ring (37 to 76 m) is already four crossed cards at fixed angles
  (0, 45, 90 and 135 degrees on the tree's seeded turn), each showing the
  bake taken from its own side.
- The bake (`pbd_core::tree_bake`) draws each variant's near model
  orthographically from 8 bearings, 0.55 rad down, into a 384 x 72 strip of
  48 x 72 views. It has no view from above.
- The cross-fade is `tree_level` / `tree_show` (a moment in time, `flora-v2`
  design 13.3) and the Bayer mask in `tree_dropped`.
- Every tree in the impostor draw invokes 30 vertices: 24 for the mid cards
  and 6 for the billboard.

## What changes

1. A far tree is two of the mid ring's cards, the ones at 0 and 90 degrees
   on its seeded turn: an X standing on the tree's base, never turned to
   the eye. The mid ring keeps all four.
2. Both rings carry a small horizontal crown cap a little over halfway up
   the tree, showing a new ninth baked view taken straight down, so a
   forest seen from above reads as crowns and not as crossed lines. Within
   "2-4 billboards" this is recommended as acceptable and taken
   (recommendation taken, ask only with screenshots); the screenshots are
   in `docs/screenshots/crossed-tree-cards/` for the owner to overrule it.
3. The X's two cards and the cap are shared by the mid and far rings: they
   are drawn whole while a tree is in either, so the mid-to-far change
   dithers out only the two diagonal cards. The fade into and out of the
   near model and the trees' range is unchanged.
4. One lighting normal per tree for all its cards and its cap, as today.

Unchanged: where every tree stands, its species, variant, turn and size
(all derived from its id, none of it saved), the near models, the ring
distances, and the vertices invoked per tree (30).

## Impact

- `assets/shaders/planet_surface.wgsl` (the tree branch, `impostor_pixel`,
  `IMPOSTOR_W`), `crates/pbd-core/src/tree_bake.rs` (the ninth view); the
  `planet_trees.rs` constant test follows `STRIP_W`.
- The kit grows by one view per bake (9 instead of 8).
- Frame cost was not measured: built in a cloud container with no GPU. A
  far tree rasterises 6 triangles instead of 2 and a mid tree 10 instead of
  8; the vertices invoked per tree stay 30.
