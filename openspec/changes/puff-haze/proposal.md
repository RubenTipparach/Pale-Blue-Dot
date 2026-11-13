# Proposal: overlapping puffs without an outline

## Why

The owner, 2026-11-13, with a screenshot looking up at grey-white cumulus:
"overlapping clouds have a weird outline". Where one billboard puff
(`cloud-puffs`) lies in front of another, its silhouette is drawn as a hard,
slightly lighter line with a thin bright fringe just inside it, a cut-out
sticker edge instead of one cloud blending into the next.

This sits under priorities 2 to 4 in `CLAUDE.md` (clouds that blend, near and
far, with the atmosphere), and must not regress any of them.

## What was found

Reproduced at the default spawn under a half cover (`--weather-at 1800`,
cover 0.55, rain 0.01), looking north and south 35 degrees up, and from
900 m up at the deck along the limb, where it is the hard outline priority 4
names. Shots, crops and the launch lines are in
`docs/screenshots/puff-haze/`.

**The outline is the air's haze, measured to one distance for every puff in
the texel.** Set `cloud_haze` to 0 and the outline is gone, every overlap
blending softly (the diagnostic crop in the design).

- The puff pass blends each puff's light over the ones behind it correctly
  (premultiplied "over"), but its second target, the distances, keeps only
  the NEAREST puff's distance (blend `Min`). A puff writes its distance
  wherever its coverage passes the discard threshold (0.004), so the front
  puff's whole footprint, its almost invisible fringe included, holds the
  front puff's distance.
- The composite then fades the whole texel, the front puff and everything
  behind it, by the air in front of that one distance (`cloud_air`). The far
  puff seen through the near puff's thin fringe is hazed as if it were as
  near as the front puff; one texel further out it gets its own haze.
- With the shipped haze (`cloud_haze: 3`) the difference is large. From the
  spawn's 65.6 m, 25 degrees up, the air leaves 0.50 of a puff 800 m out and
  0.32 of one 1600 m out. The far puff is drawn about one and a half times as
  strong just inside the near puff's footprint as just outside it: the step
  is the outline, and the fringe is where the near puff is still almost
  transparent and the far puff shows through under too little haze.

The blend, the edge erosion and the 0.4-scale upsample were checked and are
not the cause: with the haze off, the same frame has no line at all.

## What changes

- **Each puff is hazed by the air in front of it, in its own pass.** The
  puff's fragment measures the air to the puff along its own ray
  (`cloud_air`, the same function the composite uses per pixel) and scales
  the puff's light and coverage by it.
- **A near puff hides the far one by its own coverage, not its hazed
  coverage.** The air in front of a puff adds its light over it; it does not
  make the puff transparent to the cloud behind. Fixed-function blending
  takes one factor per target from that target's alpha, so the hazed
  coverage is kept in a third, one-channel target at the history's size
  (design 3c). Laid plainly "over" instead, every stack of puffs came out up
  to 48 levels brighter inside (measured, design 3b).
- **The composite stops hazing the puffs' history** (it would haze them
  twice) and reads their coverage from the new target. The march renderer is
  unchanged: its history still carries one distance a texel and the
  composite still hazes it.
- The distances target and every test that reads it (the ghosting tests,
  the upsample's silhouette test) are unchanged.

No new tunables: the haze is the same `cloud_haze` and fog constants.

## Out of scope

- The march renderer (`PBD_CLOUD_RENDERER=march`), whose haze is per texel
  and one cloud deep.
- The puffs' look otherwise (erosion, silver, shading).
- Frame time: not measured here. The change moves a five-sample haze
  integral from every full-resolution composite pixel inside the cloud layer
  to every visible puff fragment at the 0.4 scale, overdraw included, and
  adds one 16-bit target there. Which costs more depends on the overdraw;
  the owner's `perf_suite.py` on real hardware is the measurement.
