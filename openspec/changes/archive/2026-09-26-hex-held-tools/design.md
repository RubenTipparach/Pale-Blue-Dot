# Design: hex held tools

## One source, three outputs

`tools/gen_held_tools.py` holds each tool's shape (x along the handle from its
butt, y up the drawing, in tool units), its held pose and the hand. It writes:

- **the mockup's data** (between the `HELD-DATA` markers in
  `docs/mockups/held-tools.html`), which already exists;
- **`assets/models/held_tools.ron`**, new: what the game draws;
- **`assets/items/tools/*.png`**, taken over from `gen_item_icons.py`.

`--check` fails if any of the three is stale, as `gen_item_icons.py --check`
does today. Blender renders stay a `--render` option, for looking at.

The RON is model data like a mesh, not a tunable, so it lives in
`assets/models/`. The game reads it with `include_str!`, so it is compiled
in, as the icons are today. It carries, per tool:

| Field | Meaning |
| --- | --- |
| `spacing` | hex centre spacing, tool units (0.25: the 4x grid) |
| `hexes` | `(q, r, [r, g, b], depth)`: axial position, straight sRGB colour, depth in tool units |
| `rotation`, `scale`, `butt` | the rest pose in eye space: tool to eye rotation, metres per tool unit, where the butt (0, 0) sits |
| `fist` | where the fist sits in eye space, metres: the chop pivots here |
| `tip` | the far end of the tool in eye space: where the fishing line leaves the rod |
| `hand` | the hand's prisms in tool units: axis, centre, corner radius, length, an optional width across, and a straight sRGB colour |

Only the approved state ships: the 4x grid and the 2x size. The mockup's 1x and
lower-density switches stay mockup-only.

## The mesh (`pbd_core::hexel`)

It switches to axial coordinates on a grid of any spacing: pointy-top hexagons,
a hex's centre at `s * (q + r/2, -r * sqrt(3)/2)`. Each hex has a front and a
back at plus and minus half its own depth. A side wall is built toward a
neighbour only over the part of the hex's depth that stands above that
neighbour's, so a round handle is closed and has no inside faces. Faces are
shaded by direction as today.

The picture sampler (`hexels`, `SAMPLE_RADIUS`) goes: nothing samples icons any
more, and keeping it would be a second way to make a held model.

`prism` builds one hand part: a hexagonal prism along an axis, optionally
flattened to a width across (the back of the hand). It is shaded by its face's
direction against a light given in the part's own frame, so the hand is lit
as it is seen in the eye's frame at rest.

## In the app (`held.rs`)

- One holder per tool, a child of the walking camera, placed at the rest pose.
  It has two children: the tool's hex mesh and the hand's mesh. Both are unlit,
  with vertex colours, as the tool is today.
- The sway and the chop are as today, from `held.ron`. The chop pitches about
  the fist rather than a grip point.
- `held.ron` loses the icon-sampling fields (`anchor_m`, `reach_m`,
  `rod_grip_m`, `rod_tip_m`, `twist_rad`, `depth_px` and the four icon axes).
  The pose now comes from the model file. The shipped-equals-default test
  moves with it.
- The fishing line starts at the model file's rod `tip`. `fish.rs` stops
  reading the tip from `HeldConfig`.

## The icons

Each tool is drawn flat, handle diagonal from bottom left to top right as the
current icons are, scaled to fit 16 px. Each pixel takes the colour of the hex
under its centre, if the hex covers it. Everything else is transparent.
`gen_item_icons.py` drops its four tool grids.

## Checks

- Core: a hex model's mesh is closed; a hex taller than its neighbour gets a
  side wall only over the difference; a prism is closed and finite.
- App: the model file parses; every tool is one connected piece; only the tool
  in hand is shown; the line leaves the rod model's tip; the hand is drawn with
  the tool.
- `gen_held_tools.py --check` and `gen_item_icons.py --check`.
- Headless captures of each tool in hand, and one mid-chop.
