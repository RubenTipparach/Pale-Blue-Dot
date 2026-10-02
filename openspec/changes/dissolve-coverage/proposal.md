# Proposal: a landing's dissolve never shows the sky through the ground

## Why

The owner, on the first town-flyover take (2026-10-02): "There are gaps in the
terrain on the sides of hex faces", and then "fix all of take 1s holes first
(redo take 1 until holes fixed)", choosing the proper fix over switching
either fade off ("fix it properly, A").

Measured on take 1's path with the sky painted magenta (`PBD_NO_SKY=1`), a
frame saved every 0.1 s of real time (`--snap-every`), and a detector that
counts sky-coloured pixels with ground above and below them:

| Run | Worst frame, hole pixels |
| --- | ---: |
| Both fades on, as shipped | about 30,000 - 44,000 |
| Landing dissolve off (`lod_fade_s: 0`) | about 6,000 (the detector's false positives: dithered treetops on the horizon, cloud edges) |
| Band-ring dither off (`lod_fade_width: 0`) | 5,870 |

Each fade alone is hole-free; the two together are not. The holes are the
landing dissolve's old half: pixels where the new ground stands aside for the
old and the old ground is not drawn.

## What changes

The test that decides where the new ground may stand aside for the old one
(`old_held`, in both the visibility pass and the surface pass) asks whether
the records hold every level the old partition can draw at a point, not only
the finest. Across a band's cross-fade ring (`distance-lod-fade`) the old
partition draws two levels, and in fast flight the records the new set laid
often lack the coarser of the two there. Where they do, the new ground keeps
the pixel, as it already does where the finest level is missing
(`detail-fade` section 4).

Nothing else changes: no constant, no default, no config value. Where the
records hold the old ground, the landing still dissolves; where they do not,
that ring switches at the landing, which is the existing fallback.

## Impact

- `assets/shaders/planet_visibility.wgsl` and `assets/shaders/planet_surface.wgsl`:
  `old_held`, the same function in both, held identical by a test.
- A GPU test that runs the shipped `old_held` against real fine sets
  before and after a fast move and finds every point it calls held where
  the records lack a cell the old partition draws.
