# Waterline, third pass (ocean-environments B19)

The owner, 2026-11-16: "the water line shades above it" (in shallow sea at
`52.053 -118.367`), "you must be submerged at just the right height", and
under a river at `56.322 -117.795`: "getting better, but still lots of work
to do". And on the last round's videos: "if you smooth the up and down test
for water line, then that might be a better test".

## The videos

On Drive, `My Drive/dungeon-crawler-2026/pale-blue-dot-validation/`, H.264
1080p60, before and after at each place:

| Place | File stem | Where (`--at`), heading | Surface | Down to |
| --- | --- | --- | --- | --- |
| The owner's shallows | `waterline3-owner-shallows-<before\|after>-1080p60.mp4` | `52.053 -118.367`, west | -0.99 m (the tided sheet) | 0.6 m under |
| The owner's river | `waterline3-river-under-<before\|after>-1080p60.mp4` | `56.322 -117.795`, north (down the channel) | 38.81 m | 2.0 m under |
| A sea shore | `waterline3-sea-shore-<before\|after>-1080p60.mp4` | `18.86 27.54`, east | -0.14 m | 1.4 m under |
| A river mouth | `waterline3-mouth-<before\|after>-1080p60.mp4` | `20.75 10.12`, south | 1.41 m | 1.4 m under |

**The path** (smooth, as the owner asked): the place and the look are fixed
for the whole shot; no teleport, and the height never follows the water.
The eye goes from 0.6 m over the surface down to 0.4 m under it at a steady
0.1 m/s (10 s), on down to the depth in the table, holds 1 s, comes back up
to 0.4 m under, and rises through the surface at 0.1 m/s to 0.6 m over. 1500
frames, 25 s. The label in the corner is the eye's height against the
renderer's own surface (`eye_surface_radius`), its state and the frame.

**How they were made:** the Mac (M4 Max, Metal), `--profile fast`, a fixed
60 fps step (looks, not frame time), a 960x540 window scaled to 1080p, by day
(`--time 17`), no rain, lens drops off. Before is `origin/claude/env-v2` at
`b9635f4`; after is `claude/waterline-3`. Only shaders differ, and the two
were rendered from the same binary with the shaders swapped.

## The stepped frames

Every frame within ±0.3 m of each crossing (about 360 a crossing, 720 a
video) is listed in `<place>-<tag>-frames.tsv`: its height to the surface,
the eye's state, the mean brightness of five bands down the frame and the
largest change from the frame before. `<place>-<tag>-crossing<1|2>.jpg`
shows every sixth of them (1 cm apart), labelled; `<place>-<tag>-sheet.jpg`
is 12 frames across the whole video.

**What every crossing shows.** In all eight videos the eye's state flips
exactly once each way, within 2 mm of the surface (the `.tsv` files), and
never flickers.

| Video | Over the surface (+0.3 to 0) | Under (0 to -0.3) | Frame-to-frame jumps over 6 (of 255) |
| --- | --- | --- | --- |
| owner-shallows before | the shallows and the bank, clean | **the bug**: for 2 to 3 cm a band of clear sky and dark tree silhouettes over the bank (the near-plane hole), then the whole upper half a flat dark navy band from the waterline up, where the bank was | 25: the crossing (95) and 24 frames of the hole and the band changing as the eye sinks |
| owner-shallows after | the same | the bank under water, and above the waterline its mirror in the surface, rippled; no navy band, no sky | 2: the two crossings (36, 40) |
| river-under before | the river and its sandy bank | the hole (sky and a black cactus) for 3 to 5 cm, then a flat navy ceiling with blue blotches over the lit channel walls; speckle at the far end | 24: the crossing (44) and the hole closing |
| river-under after | the same | the walls meet their mirror in the surface, the ceiling as bright as the water, blue sky patches only in Snell's window; a thin sliver of sky at the right horizon for the 2 to 5 cm under (see Open) | 2: the two crossings (74, 73) |
| sea-shore before | the shore and its terraces, clean | the hole for 2 cm, then the upper half flat navy over the drowned terraces | 3: the two crossings (77, 78) and one 6.5 |
| sea-shore after | the same | the terraces and their mirror; no navy band, no hole | 3: the crossings (58, 54) and one 6.5 |
| mouth before | the mouth and the dirt bank | a flat navy band over the bank from the waterline | 0 |
| mouth after | the same | the bank's mirror, which streaks vertically under the waves where a bright block is mirrored | 2: the crossings (18, 23) |

**Just over the surface** (the owner's "the water line shades above it" with
the eye above): no frame in the after videos at +0.3 to 0 m darkens the bank
above the waterline. The before videos did not show the far crest band
(design B19 item 3) either: the sea was calm on these shots, so that fix is
in by reading, not by a video.

**Under, away from the surface:** the river's far end no longer speckles
(compare `river-under-before-sheet.jpg` and `-after-sheet.jpg` at 2 m
down); the shafts still read as soft beams.

## Open

- **A sliver of sky, river only, 2 to 5 cm under.** Where the river steps
  down ahead, a ray just over the horizon leaves the eye's water over the
  lower reach without meeting a surface, and shows the sky. It was the whole
  upper frame before; it is a sliver for a few frames now.
- **The mirror streaks** where a bright block is mirrored under rough
  water (the mouth). It is a reflection, rippled by the waves' normals at a
  quarter strength.
- **A hairline at the waterline** for the first centimetres under, where the
  composite's stand-in for the clipped sheet meets the drawn sheet.
- **The B18 capture items.** Mouth f804-808 was the capture tool's 40 m
  teleport while the tier rebuilt (eye `Unknown`): not reproduced with the
  smooth path, which never teleports. The tributary riffle f512 was the eye a
  few centimetres under a thin cell's top, put there by a path that followed
  the water up a step: the picture was right for where the eye was. The
  smooth paths do not follow the water and do not produce it.
- **Frame cost not measured.** The sheet's from-below path now runs the
  screen trace (only for an eye under water). The perf suite waits for a
  quiet machine, as the other water items do.
