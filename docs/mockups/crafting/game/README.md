# The game's own item icons, as the crafting mockup uses them

Byte-identical copies, so the published mockup can load them; nothing here is
edited. The originals are the source of truth: if one changes, copy it again.
All are 16 x 16 and shown at whole-number scales (2x in a slot, 6x for the pickaxe).

| File | Source | Used for |
| --- | --- | --- |
| `hand.png` | `assets/items/tools/hand.png` | Hotbar slot 1, the bare hand |
| `torch.png`, `lantern_post.png`, `candle.png` | `assets/items/lights/` | Lights in the hotbar, as the game's own hotbar carries them |
| `pickaxe.png` | `assets/items/tools/pickaxe.png` | Section 7, beside the four pick tiers, as the one pickaxe the game ships today |

The game ships no icon for any other item on this page, so the rest are drawn
by `tools/art/crafting_art.py` at 32 x 32 with the balatro-pixel-art kit.
