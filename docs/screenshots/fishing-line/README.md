# The fishing line meets the float (2026-11-16)

`fishing-and-equipment` design 4, "The line's far end". Taken on an M4 Max
Mac (Metal GPU), `--profile fast`, with
`pbd-app --capture <png> --fish --tide low --time 11 --frames N`.
"Before" is `claude/env-v2`'s line drawing with this branch's `--fish` script
(which now hands the walker a rod; without it a new world casts nothing).

| Frame | What | Before | After |
| --- | --- | --- | --- |
| 185 | the float in flight | `before-185.jpg` | `after-185.jpg` |
| 400 | the float on the water, low tide | `before-400.jpg` | `after-400.jpg` |
| 400 | close-up round the float | `before-400-close.jpg` | `after-400-close.jpg` |

Before: the owner's V. The line's tail was held over the mean sea, a metre
over the low-tide water, and its end hangs over the float. After: the line
is held over the drawn waves and their crests and comes down onto the top of
the float, white all the way. The float itself is no longer tinted as if
seen through water: an eye in the air now measures the composite's waterline
from the tided sea, not the mean (design 4, "The last stretch still read as
short"). Some runs hit a wgpu queue timeout while other agents were
rendering on the same GPU; those shots were retaken.
