# Provenance of the light icons

The six lights' 16x16 icons are new to Pale Blue Dot and drawn by
`tools/gen_item_icons.py` (`lamps-and-lanterns` task 5.3). Each one's pixel
grid and the lights' one palette are in that file, which is the source:
`python3 tools/gen_item_icons.py --check` fails if a PNG here differs from
what it draws.

| File | Material | What it shows |
| --- | --- | --- |
| `torch.png` | `Torch` | A wooden brand with a flame on it. |
| `lantern_post.png` | `LanternPost` | A street lantern on its pole. |
| `lantern_wall.png` | `LanternWall` | A lantern on a bracket from a wall. |
| `lantern_hanging.png` | `LanternHanging` | A lantern on a chain. |
| `brazier.png` | `Brazier` | A fire in an iron bowl on legs. |
| `candle.png` | `Candle` | A candle in a holder. |

The lanterns' glass is the towns mockup's lamp colour (`#ffd9a0`), and the
flames run white-gold at the root to orange at the tip, as the game draws
them. `hotbar::light_icon` names each file, and
`kit_tests::every_light_has_its_own_icon_and_nothing_else_does` checks that
each is a 16x16 picture on a transparent ground.
