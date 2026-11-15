# Provenance of the husbandry icons

The five 16x16 icons are new to Pale Blue Dot and drawn by
`tools/gen_item_icons.py` (`animal-husbandry` design 12, S6), whose pixel
grids and one palette (`FARM`) are the source: `python3
tools/gen_item_icons.py --check` fails if a PNG here differs from what it
draws.

| File | Item | What it shows |
| --- | --- | --- |
| `fence.png` | `Good::Fence` | Two posts and two rails. |
| `gate.png` | `Good::Gate` | A gate between posts, with its brace. |
| `egg.png` | `Good::Egg` | A cream hen's egg. |
| `wool.png` | `Good::Wool` | A shorn fleece. |
| `milk.png` | `Good::Milk` | An iron pail of milk. |
