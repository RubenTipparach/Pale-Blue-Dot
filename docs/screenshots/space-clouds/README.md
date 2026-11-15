# space-clouds: solid clouds of many sizes from space

The owner, 2026-11-14: "clouds are also a bit too transparent from space" and
"their size is kinda uniform". The write-up is `openspec/changes/space-clouds/`.

Fixed-step captures (`--capture`, `--frames 200`) on the M4 Max Mac (Apple M4
Max, Metal, GPU), 2880 x 1800, downscaled to 1200 x 750. Each run in a fresh
world. The before is `claude/perf-tuning` 3b44fb5; the after is this branch
with `space-clouds` only (before `dusk-clouds`). Same arguments both sides.

## Launch lines

`<w>` a fresh world name, from the repository root:

```text
day-down:   target/fast/pbd-app --world <w> --view column --at 24.117 -2.964 --height 9000 --pitch -89 --time 12 --weather-at 1800 --frames 200 --capture day-down.png
day-limb:   ... --height 9000 --pitch -68 --yaw 90 --time 12 --weather-at 1800
dusk-limb:  ... --height 9000 --pitch -68 --yaw 90 --time 18.6 --weather-at 1800
storm-down: ... --height 6000 --pitch -75 --yaw 90 --time 12 --weather-at 1800 --rain 1
high-deck:  ... --height 900 --pitch -36 --yaw 90 --time 12 --weather-at 1800   (puff-haze's high shot)
ground-n:   target/fast/pbd-app --world <w> --walk --at 24.117 -2.964 --time 12 --pitch 35 --yaw 90 --weather-at 1800 (puff-haze's north shot)
```

## Files

`<shot>-before.jpg`, `<shot>-after.jpg` and `<shot>-before-after.jpg` (the
two side by side, before on the left).

## What they show

- **From 9 km and 6 km** (`day-down`, `day-limb`, `dusk-limb`, `storm-down`):
  before, every puff about one size, half transparent, the land and sea
  plain through them. After, the clouds are solid white where they are thick,
  gather into large masses with small cumulus scattered round them, and the
  ground no longer shows through. Along the limb they still fade into the air
  (the sky is behind them there), as `horizon-haze` decided.
- **From 900 m** (`high-deck`): the deck is denser and whiter; the far deck
  along the limb fades as before.
- **From the ground** (`ground-n`): the same cover (0.55) now reads heavier.
  The clump field put a large mass overhead at this spot, and the puffs are
  more opaque, so more of the sky is cloud than before. This is the side
  effect to judge; `cloud_puff_optical_depth` (now 4, was 15) is the knob.

## Puff counts (the log's `PUFFS built / over the horizon / in view`)

| Shot | Before | After |
| --- | --- | --- |
| day-down | 3061 / 1562 / 1562 | 3334 / 1657 / 1657 |
| day-limb | 3061 / 1562 / 916 | 3334 / 1657 / 962 |
| dusk-limb (full cover) | 2041 / 1490 / 1021 | 2135 / 1547 / 1098 |
| storm-down | 2975 / 1379 / 924 | 3234 / 1472 / 965 |
| high-deck | 3061 / 555 / 201 | 3334 / 592 / 213 |
| ground-n | 3061 / 252 / 46 | 3334 / 264 / 72 |

About 9 % more billboards: the small puffs under a tall top stack to three
sooner (`each = max(1.4 r, height / 3)`). Frame time not measured here.

The one log error in both runs (`flame.png` not found) predates this change.
