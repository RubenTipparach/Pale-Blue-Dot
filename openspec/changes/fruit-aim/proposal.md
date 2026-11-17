# Proposal: pick the fruit you aim at

## Why

The owner, 2026-11-17: "picking fruits should be aimed at the actual fruit not
the tree". Today F picks a fruiting tree's fruit whenever the eye ray meets the
tree's trunk or crown column (`digging::tree_on_ray`), wherever the fruit hangs.

## What Changes

- F picks when the eye ray meets one of the fruit hanging on the tree, within
  reach. Aiming at the trunk or the leaves offers nothing.
- What a pick takes, the record it writes and the regrowth are unchanged.

## Impact

- `pbd_core::flora`: `hanging_fruit_points`, where each kept hanging fruit is,
  the draw's own placement on the CPU (the near tree's model anchors, its frame
  and its rolls), pinned by a test against the shader's lines.
- `desktop/fruit.rs`: `fruit_on_ray` replaces `tree_on_ray` in offer and pick.
