# Rain drops on the lens: before and after (`lens-drop-refraction`)

The owner, 2026-11-13, on this beach in heavy rain: the drops looked "flipped
upside down", with "lots of black drops on top". The cause and the fix are in
`openspec/changes/lens-drop-refraction/design.md`.

- **Before:** each drop sampled the scene 70-450 px away (at 1440x900),
  outward from its centre. A drop over the sky near the trees had a dark
  crescent along its lower rim, and a drop on the trees had a sky-white
  crescent along its upper rim.
- **After:** each drop shows only what is behind it, magnified toward its
  centre (`rain_lens_bend` 0.5). It has a soft darker edge
  (`rain_lens_rim` 0.2), and the sky's light gathers on its lower edge
  (`rain_lens_glint` 1.0). Over the sky a drop is a faint
  darker-above, lighter-below ring of sky. Over the trees and the wet soil
  it shows a light crescent underneath.

## How they were taken

Every shot is a fixed-step capture on the Mac (Apple GPU, `--profile fast`,
2880x1800 framebuffer), at frame 240. Each was taken in a world made fresh
for that shot and deleted after it. The captures here are scaled to
1440x900, and the `crop-*` files are 1:1 from the 2880x1800 frame.

- **Before:** the `claude/perf-tuning` build at `65d414a` with its own
  `water.wgsl` and `weather.ron`.
- **After:** the same base with this change.

Each before-and-after pair has the same arguments and the same frame.

Every shot is at the same place. From the `PLACE capture` line:

```text
at 12.948 8.135 (map 12.948S 8.135E) | 2.6 m elevation | 1.6 m above ground
```

That is `--at 12.948 8.135`, the owner's beach, on foot, eye 2.6 m above sea
level and 1.6 m above the ground.

Weather, from the `Weather here` line, forced by `--rain 1`, in every shot:
`cover 1.00, rain 1.00`.

| Shot | `--yaw` (facing) | `--time` | Sun ("sun from the spawn at --time") |
| --- | --- | --- | --- |
| `treeline` | 0 (west) | 12 | 68.2 degrees up (`--yaw -148.3 --pitch 68.2`) |
| `hill` | 90 (north) | 12 | 68.2 degrees up |
| `sea` | 180 (east) | 12 | 68.2 degrees up |
| `beach` | 270 (south) | 12 | 68.2 degrees up |
| `night` | 180 (east) | 0 | 48.6 degrees below the horizon (`--yaw -61.4 --pitch -48.6`) |

The launch command, with `<yaw>` and `<time>` from the table:

```text
pbd-app --world <new> --walk --at 12.948 8.135 --rain 1 --time <time> --yaw <yaw> --pitch 5 --frames 240 --capture <png>
```

## The shots

- `before-<shot>.jpg`, `after-<shot>.jpg`, and `before-after-<shot>.jpg`
  (before on the left).
- `crop-treeline-sky.jpg` (`treeline`, 1:1, the sky over the trees, before
  on the left). Before: dark crescents under the drops on the sky and white
  arcs over the drops on the trees. After: sky-coloured drops, lighter
  underneath, and light crescents under the drops on the trees.
- `crop-sea-sky.jpg` (`sea`, 1:1, open overcast sky). Before: small sharp
  dark rings. After: soft rings of sky, darker above and lighter below.
- `crop-sea-ground.jpg` (`sea`, 1:1, sand, wet soil and water). Before: the
  drops on the soil and the water were sky-blue and sand-coloured specks
  from far off. After: magnified patches of what is behind them, each with a
  light crescent underneath.
- `night`: the light on a drop's lower edge is the sky's mean light, so at
  midnight it is faint. The drops show only as dim smudges on the dark sky.
  The before's dark specks are gone.

The bend 0.5 and 0.8 shots looked alike at this size, so 0.5 was kept (fewer
drop centres inverted). Recommendation taken: ask only with screenshots.
