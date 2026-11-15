# Farming mockup art (M8)

`farming.html` loads everything here by a relative path. `game/` holds byte-identical copies of the
game's textures (see `game/README.md`). The rest is mockup-only pixel art:

| Files | Made by | Notes |
| --- | --- | --- |
| `<crop>_<0..3>.png`, `produce_*.png`, `tool_*.png` | `tools/art/farming_art.py` | Icons and UI stages (crop table, cell panel, seed belt, harvest slots), with their 1 px outline. |
| `world/<crop>_<0..3>.png` | `world_art.py` (this folder) | What the 3D plot draws: the same stages with no outline, no seams and near-black tones lifted one step (owner, 2026-11-15: "outlines are for sprites in the ui inventory system"). Also copied to `../foliage/crops/` for M13. |

Regenerate with `python tools/art/farming_art.py`, then `python docs/mockups/farming/world_art.py`.
