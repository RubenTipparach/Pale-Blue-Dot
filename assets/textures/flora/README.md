# Tree bark the atlas does not hold (`flora-v2` design 10.2)

Byte-identical copies of the approved M13 foliage mockup's in-world bark
(`docs/mockups/foliage/`). The detailed trees' kit (`pbd_core::tree_kit`,
`KIT_TILES`) loads them with the settlement `leaves.png` and `needles.png`;
they are drawn nearest-sampled, as every texture in the world is.

| File | Made by |
| --- | --- |
| `bark_birch.png` | `docs/mockups/foliage/world_art.py` (`tile_birch_soft`: no outline, the lenticels in the bark's own slate) |
| `bark_palm.png` | `tools/art/foliage_art.py` |

Change the mockup's scripts and copy again; never edit these by hand.
