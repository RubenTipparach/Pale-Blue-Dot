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
