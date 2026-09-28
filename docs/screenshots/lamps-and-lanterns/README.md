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

## The ship parked in a cave (task 3.3, decision 14)

Captured 2026-09-28, same container and settings. `--walk --ship-in-cave`
starts the walker in a cave chamber (the same search `--view cave` uses) and
holds the Kestrel 9 m down the tunnel at half the chamber's height, nose away
from the camera (`CraftHold`, pinned by
`vehicles::tests::a_held_kestrel_stays_where_the_capture_put_it`). The log
says where: "the walker starts in a 4 m cave chamber and the Kestrel is parked
9 m down it, 2.0 m above the floor".

The first midnight shot found a fault, now decision 14. At midnight in a
sealed cave the Kestrel's belly came out white, because Bevy's sun, which has
no shadows, lit it from under the horizon through the rock. The fix keeps
Bevy's lighting only where the sun is up and the sky reaches, and gives
everything else the albedo in the terrain's own fill. "Before" and "after"
below are the same binary, with only `field_lit.wgsl` swapped.

| file | flags | what it shows |
| --- | --- | --- |
| `cave-ship-midnight-before.jpg`, `cave-ship-midnight-after.jpg` | `--time 0` | Before, the hull's right side reads 106 (sRGB, 0 to 255) against the stone's 18: the sun through the planet. After, 20 to 28, a dark shape at the walls' level. |
| `cave-ship-noon-before.jpg`, `cave-ship-noon-after.jpg` | `--time 12` | At noon the lit face was the top, out of sight, so before looked right: 14 to 22 against 19. After, 24 to 32: the same fill as the night, since no sky reaches either. |
| `cave-ship-torch-before.jpg`, `cave-ship-torch-after.jpg` | `--time 0 --torch` | A torch on the floor under the camera. The chamber, the hull and the hand all take its warm light. Before, the hull's right side had the sun on top of the torch (155 against 130 on the left). After, both sides are even (131, 134). |
| `open-ship-night-before.jpg`, `open-ship-night-after.jpg` | `--aboard kestrel --time 0` | Midnight in the open, from the chase view. Before, the wings' tops were near black (14) and the hull had a warm panel from the sun below the ground. After, the wings and hull take the night's cool fill (34 to 47), the level of the grass around them (36). |
| `open-ship-noon.jpg` | `--aboard kestrel --time 12` | Noon in the open, after. It is byte-for-byte the before frame, which is decision 10's promise kept: by day in the open the ship is Bevy's own. |

`--torch` puts a torch on the ground under the camera. In the first attempt it
was placed and then undone, and the frame was byte-for-byte the frame without
it. The torch went in while a fine set was being rebuilt, and that set had
copied the edits before the torch existed, so when it landed the torch was
gone. The same thing happens to a player who digs while a rebuild is in
flight: the hole fills back in until the next rebuild. It is written up in
`openspec/changes/edit-pipeline` ("undone when it lands") and not yet fixed.
Until it is, the capture rig waits for the landing before it places anything.

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
