# underwater-fish-light: fish under the water at full brightness (`ocean-environments` B23)

The owner, 2026-11-17: "fish under water have full light brightness, the
water should lower light brightness".

Fixed-step captures on the M4 Max Mac (Metal, GPU), `--profile fast`,
memory-only worlds, generator version 9. "Before" is `claude/env-v2` at
`db8f17e`; "after" is this branch. Looks only: frame time was NOT measured.

| Shot | Command |
| --- | --- |
| `shallow-*` (the owner's shot: feet on a 3.4 m bed, eye about 1.8 m under, level, reef fish 3 m off) | `--walk --at 13.096 94.896 --life reef:3 --pitch 0 --time 11 --frames 370` |
| `deep-*` (feet 9.4 m down, the eye about 8 m under, a ray 4 m off) | `--walk --at -4.125 -11.125 --life ray:4 --pitch -10 --time 11 --frames 360` |
| `above-*` (from the shore, looking down into the water at perch) | `--walk --at 24.000 122.469 --yaw 180 --life perch:3 --pitch -35 --time 11 --frames 370` |
| `night-*` (the reef at 23:00 with a torch placed) | `--walk --at 13.096 94.896 --life reef:3 --lamps 2.5 0 --pitch 0 --time 23 --frames 370` |

- `before-after-sheet.jpg`: the four shots, before left, after right.
- `deep-ray-crop-before-after.jpg`: the ray, before (top) and after.
- `shallow-school-crop-before-after.jpg`: the reef fish, before (left) and after.

## Honest read

- **Deep (about 8 m)**: the clearest change. Before, the ray was a glowing cyan
  sliver over a dim bed: Bevy's PBR picture mirrored the open sky's light
  off its back, which no water dims. After, it is lit by the terrain's
  light at its depth and sits in the water as dark as the bed beside it.
- **Shallow (the owner's shot)**: the reef fish lose the bright blue sheen
  on their fins and backs (the mirrored sky) and take the water's tint;
  the reef's own pieces, lit by the same PBR picture before, now sit at the
  seabed's brightness instead of glowing above it. The fish are about as
  bright as before overall: they swim 1 to 2 m under the surface, where the
  daylight left is most of it, so they are lit more than the bed 3.4 m
  down. That is the water's rule, not a sticker: their colour is the
  seabed's own rule at their depth.
- **From above**: perch in 0.5 to 2 m of water; little changes, as little
  water lies over them. The water's transmission is the sheet's and was
  already applied.
- **Night**: little changes. The fish are lit by the night sky's light as
  the reef is; the torch the instrument placed landed 7 m to the side, so
  its light does not reach the school in this frame. Not shown: a lamp
  beside a school under water.

READY with no new `LATE_PIPELINE`: each shot's list is the same before and
after (32 to 34 lines, the capture instruments' own).

## The owner's view: perch near the surface (second pass, 2026-11-17)

The owner's fish are **perch** (yellow back `#cea834`, five dark bars,
orange fins `#e8622c`): a school (`fish.rs` `draw::schools`), not the
reef's ambient fish or the lineup. Reproduced in a shallow sea off the
fishing shore, eye about 0.8 m under, the school 3 to 6 m off near the
surface, against the sand at the same distance:

`--walk --at 24.000 122.66 --yaw 0 --life perch:3 --pitch 10 --time 11 --frames 370`

- `owner-before.jpg`, `owner-after.jpg`, `owner-school-crop-before-after.jpg`
  (before on top).
- `owner-spot-bodies-on-the-bed.jpg`: the same spot, `--life lineup --pitch
  -25`, every body lying ON the sand at the sand's depth and distance, with
  its texture (top) and with `PBD_PASS_OFF=life_texture` (white albedo,
  bottom). The lineup is tinted by its depth as the schools are (this pass).

Measured (sRGB 0-255 means; luma Rec.709; saturation (max-min)/max):

| | luma | saturation | mean colour |
| --- | ---: | ---: | --- |
| perch school, before | 32.9 | 0.73 | 34, 35, 9 |
| perch school, after | 33.9 | 0.86 | 37, 36, 5 |
| sand at the same distance | 45.2 | 0.98 | 1, 58, 51 |
| perch lying on the sand (lineup) | 33.9 | 0.71 | 12, 42, 21 |
| sand beside it | 66.8 | 0.89 | 10, 84, 69 |

The water's transfer at one spot on the bed, linear (pixel / albedo): a
white body lying on the sand 0.0084, 0.134, 0.141; the sand itself (beach
albedo 0.73, 0.58, 0.32) 0.004, 0.151, 0.187. **The fish take the seabed's
treatment**: the same light, the same depth tint, the same murk by distance
(the composite's, which they do get: they are drawn in the main pass and
write depth). A white fish on the bed reads as the bed; a perch on the bed
is darker than the sand.

What the owner sees is **hue, not brightness**: the school is darker than
the sand behind it (luma 34 against 45) but keeps its red. The sand is 2.4 m
down and 4 to 6 m off; the perch swim 0.3 to 1 m under the surface, 3 to 4
m off, so by the water's own rule they keep about three times the sand's
red, and their albedo is the strongest yellow-orange in the atlas. Taking
away the mirrored sky (the first pass) also took away a bluish sheen that
had been greying them, so afterwards they read a little MORE yellow. Making
near-surface fish teal like the bed would mean tinting them by more water
than lies over them: a look decision for the owner (survey, with these
frames), not a rule shared with the seabed.
