# Lamps and lanterns: captures

For `openspec/changes/lamps-and-lanterns`. Captured 2026-09-27 in a cloud
container on lavapipe, at 1440x900, with no GPU. They show what the game looks
like, not how smoothly it runs. Frame cost was not measured (CLAUDE.md): the
owner runs `tools/perf_suite.py` on real hardware. Videos are deferred to the
owner's batch (CLAUDE.md, 2026-09-27), so these stills are the check for now.

Every shot is a fresh run of the fast build (`--profile fast`) under
`xvfb-run`, on a memory-only save, at the spawn meadow with the view turned 90
degrees from the parked ship (`--walk --yaw 90`), unless it says otherwise.
The clock is pinned with `--time`: 0 is midnight and 12 is noon. The captures
are JPEG at quality 90, converted from the PNGs the game wrote.

`--lamps [metres [across]]` puts one of every light in a row across the view,
one cell apart, that far ahead and shifted that far right. The row, left to
right, is:

1. the torch;
2. the street lantern on its post;
3. the wall lantern, on a two-stone pillar;
4. the hanging lantern, under a stone;
5. the brazier;
6. the candle.

It also deals a stack of each light into the hotbar, so those shots show the
six icons.

## The lights (tasks 5.2 and 5.3)

| file | flags | what it shows |
| --- | --- | --- |
| `all-night.jpg`, `all-noon.jpg` | `--lamps 9 --pitch -10` | All six lights in one frame. At noon the street and wall lanterns are out, since they burn only from dusk to dawn. The hotbar shows each light's own icon. |
| `lanterns-night.jpg`, `lanterns-noon.jpg` | `--lamps 5 --pitch -14` | The wall and hanging lanterns up close: lit panes in an iron frame at night, dark glass by day. |
| `fire-night.jpg`, `fire-noon.jpg` | `--lamps 3 -5.8 --pitch -30` | The brazier and the candle up close. A fire is gold at its root and orange at its tip. |
| `post-night.jpg`, `post-noon.jpg` | `--lamps 4 5.8 --pitch -5` | The torch and the street lantern up close. |

## How far a light reaches (decisions 7 and 9)

| file | what it shows |
| --- | --- |
| `pools-before.jpg` | The old light model, one level per cell, at midnight, from 16 m back (`--lamps 16 --pitch -9`). The six lights light the meadow nearly as bright as noon, right up to the camera. |
| `pools-after.jpg` | The same view with a level worth about a metre and the mockup's falloff curve. Each light has its own pool, and the foreground falls back to moonlight. The grass by a lamp is lit, and away from the lamps it is not (decision 9). |

`pools-before.jpg` was drawn by a build with `BLOCK_ACROSS` set to 1 and
`lamp_strength` returning its input, which is the model before decision 7.
That build was made once for this picture and then put back. It also predates
decisions 8 and 9, so its grass is black and it has no glowing flowers.

## Glowing flowers (task 6.2)

| file | what it shows |
| --- | --- |
| `meadow-night.jpg`, `meadow-noon.jpg` | The meadow with no lamps (`--pitch -9`). At night the patches of warm light on the grass are glowing flowers' light. By day there is none. |
| `meadow-glow-crop.png` | Part of `meadow-night.jpg`, enlarged twice: one glowing head, the cyan speck, and the grass it lights. |

The head is small, as every flower head is (14 cm across), so at night the
glow on the ground is what reads, more than the head itself.

## What moves (task 3.3)

| file | what it shows |
| --- | --- |
| `ship-night-before.jpg`, `ship-night-after.jpg` | The Kestrel at midnight from the chase view (`--aboard kestrel --time 0`). Before, it is lit as at noon; after, it is dark in the night's light. |
| `ship-noon-before.jpg`, `ship-noon-after.jpg` | The same at noon: unchanged, which is what decision 10 promised. |
| `ship-lamps-night.jpg` | The Kestrel at midnight with the lamp row laid 8 m ahead of the chase camera. |
| `held-lamps-before.jpg`, `held-lamps-after.jpg` | The `all-night.jpg` view drawn by the build before decision 10 and by this one. Before, the hand and rod are full-bright; after, they take the lamps' warm light, as the grass around them does. |

"Before" is a build made earlier the same day from this branch, before
decision 10 was wired: the same lights, flowers and lit grass, with the held
tool full-bright and the ship on Bevy's own lighting. `held-lamps-before.jpg`
also has the earlier, paler flame colour.

Not captured yet: the ship parked in a cave. There is no capture rig that
parks it in one, so task 3.3 stays open for that shot. A sealed cave reading
dark at the eight corners is pinned by
`planet::column::tests::a_sealed_cave_gives_eight_dark_corners_and_the_open_air_the_sky`.

`--torch`, which should put a torch on the ground under the camera, placed
nothing in this batch: its placement ray found no spot, and it says nothing
when that happens. The held-tool pair uses the lamp row instead.

## The night captures frame from the clock's sun (task 2.1)

| file | what it shows |
| --- | --- |
| `midnight-view.jpg` | `--view midnight --time 0`, now aimed from the sun where the clock puts it rather than from the deleted `sky::SUN_DIRECTION`. It finds the antisolar point: a black sky over the night-side sea. |

## Daylight falls like lamplight (decision 11, survey L1)

| file | flags | what it shows |
| --- | --- | --- |
| `mouth-noon-before.jpg`, `mouth-noon-after.jpg` | `--view mouth --spawn mouth --time 12` | A tunnel mouth at noon. Before, daylight lost one level a cell and lit the tunnel to its back wall. After, it loses three a cell, as a lamp's light does, and the tunnel darkens a few metres in. |
| `cave-noon-before.jpg`, `cave-noon-after.jpg` | `--view cave --time 12` | Deep in a cave at noon, the control: dark before and after, with the far opening lit. |

"Before" is the build at `463854b`, the lights as the owner approved them.
"After" is the same build with `light::ACROSS` for both channels and kit
grant 2, so its hotbar shows the new kit: grass, dirt, stone, sand, the
torches and the five lights.
