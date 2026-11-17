# No mirror under the water; the split frame; the jump on the beach (ocean-environments B21)

The owner, 2026-11-16:
- "some weirdness happening here when I'm partially submerged camera"
  (shallow sea at `39.566 14.326`, eye at the surface);
- "do we really need SSR from bottom of the water?", "I dont think it looks
  that good" (under the sea at `40.455 14.582`, looking up in rain);
- "dry ground had the weird water blur pass that we fixed earlier. This only
  happens when you jump above water", "you can see the subtle sand tinted
  blue green" (a jungle beach beside shallow water).

## The videos

On Drive, `My Drive/dungeon-crawler-2026/pale-blue-dot-validation/`, H.264
1080p60, before and after at each place:

| Place | File stem | Where (`--at`), heading | What the eye does |
| --- | --- | --- | --- |
| The owner's split frame | `underwater-waterline-owner-split-<before\|after>-1080p60.mp4` | `39.566 14.326`, level, yaw 90 | 0.6 m over the tided sea (-1.47 m) down to 0.4 m under at 0.1 m/s, holds 2 s, back up at 0.1 m/s |
| The owner's view up in rain | `underwater-waterline-owner-rain-under-<before\|after>-1080p60.mp4` | `40.455 14.582`, 20 degrees up, yaw 90, rain | 0.6 m over down to 0.4 m under at 0.1 m/s, on to 2 m under, holds 2 s, back up through the surface at 0.1 m/s |
| A jump on a beach | `underwater-waterline-jump-beach-<before\|after>-1080p60.mp4` | `39.566 14.026` (jungle sand at 0 m, low tide), 12 degrees down, yaw 180 | stands (eye 1.6 m), three jumps at play speed (apex 2.85 m), then a slow rise at 0.1 m/s from 1.6 to 3.0 m and back down |

**How they were made:** the Mac (M4 Max, Metal, the GPU), `--profile fast`,
a fixed 60 fps capture step (looks, not frame time), a 960x540 window scaled
to 1080p, `--time 14`, lens drops off. The camera is held by `PBD_EYE_PATH`;
the label in the corner is the eye's height against the renderer's own
surface (`eye_surface_radius`), its state and the frame. Before is
`origin/claude/env-v2` at `953d0d3` (its binary with its own `assets/`);
after is `claude/underwater-waterline`.

## The stepped frames

`<place>-<tag>-frames.tsv` lists every frame within ±0.3 m of each crossing:
the height to the surface, the eye's state, the mean brightness of five
bands down the frame and the largest change from the frame before.
`<place>-<tag>-crossing<1|2>.jpg` shows every sixth of them (1 cm apart);
`<place>-<tag>-sheet.jpg` is 12 frames across the whole video.

For the jump, the "crossing" is the eye leaving the high tide's reach (2 m
over the sea's level), where it goes from "over the sea's water" to air:
`jump-beach-<tag>-frames.tsv` lists the 12 frames each side of every such
flip with the mean colour of the beach band (35% to 60% down the frame) and
its change; `jump-beach-<tag>-flips.jpg` shows that band two frames before
and after each flip.

| Video | Over the surface (+0.3 to 0) | Under (0 to -0.3) | Frame-to-frame jumps over 6 (of 255) |
| --- | --- | --- | --- |
| owner-split before | the shallows and the bank, clean | **the bug**: the sea floor mirrored over the waterline, a bright teal slab in the upper half, hard-edged where the trace's hits end, over a band of horizontal streaks; flat navy above it | 5: the two crossings (41, 41.5) and three streak frames under (6.3, 7.6, 6.2) |
| owner-split after | the same | the floor under water, and over the waterline the ceiling, the murk with the waves' faint shading; no mirror, no streaks | 3: the two crossings (90, 90) and one just over (6.2, the waves) |
| owner-rain-under before | the bank and the rain | a dark mirror of the water with rain rings, a speckled band toward the horizon | 2: the crossings (111, 111) |
| owner-rain-under after | the same | the murk with the rings as gentle shading, Snell's window lighter overhead, no speckle | 4: the crossings (111, 111) and one each side of them (6.8, 6.7) |

The eye's state flips exactly once each way in every crossing, within 2 mm
of the surface.

| Jump video | Band colour step at each flip, of 255 |
| --- | --- |
| jump-beach before | 2.5 to 3.1 at all 8 flips (6 in the jumps, 2 in the slow rise): the low sand terrace by the water takes the underwater tint and wobble the frame the eye is in air, and loses it the frame it is back in the tide's reach |
| jump-beach after | none over 1.0 at any of the 8 flips |

## Open

- **The ceiling is darker than the shallows' floor.** With the mirror gone,
  the surface from below outside Snell's window is the murk, as the owner
  asked. In a bright shallow sea that is a dark navy over a lit teal floor
  (`owner-split-after-sheet.jpg`); it is the colour the far water fogs to,
  so it meets the far water with no seam. A lighter ceiling would be a look
  change, for the owner with these sheets beside it.
- **`LATE_PIPELINE`**: a plain launch of the after build reaches READY with
  none. The `--fly` capture runs log 32 to 38 of them, before and after
  alike (the capture path compiles pipelines after its own READY); not from
  this change.
- **The frame cost is owed** (task 16.3): the Mac was not quiet (other
  sessions' builds, captures and a perf run). The held under-water view is
  ready to run: `PBD_EYE_PATH` holding the eye at `40.455 14.582`, -3.4 m,
  20 degrees up, in rain, `--no-vsync --frame-log`, release, the two builds
  interleaved three times each.
