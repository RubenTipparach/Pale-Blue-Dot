# Hex held tools: the approved mockup, in the game

## Why

The tools in hand are built by sampling each tool's 16 px icon onto a hex
grid (`held.rs`, `pbd_core::hexel::hexels`). The owner rejected that: a
diagonal handle sampled onto hex rows is a zigzag, there is no hand, and the
tools are small. The owner then worked the replacement out in a mockup
(`docs/mockups/held-tools.html`, [Hex Tool Bench](https://claude.ai/artifact/YEf1BuLEk9Dppk55znZ5gr)),
checked from several angles in Blender, and approved it ("Perfect"). This
change builds that mockup into the game.

## What

1. **The tools are drawn on the hex grid, not sampled from icons.**
   `tools/gen_held_tools.py` is already the one source for the tool shapes,
   poses and hand. It gains an output, `assets/models/held_tools.ron`: each
   tool's hexes (a quarter-unit grid, a colour and a depth each), its held
   pose, and the hand's prisms. The game builds its meshes from that file.
2. **Each hex has its own depth**, so handles are round and blades thin.
   `pbd_core::hexel` builds a side wall only over the height one hex stands
   above its neighbour.
3. **A hand.** A right hand from Blender's Rigify human metarig, closed round
   the grip by `tools/gen_held_hand.py`. Its bones become stubby hexagonal
   prisms, drawn as their own mesh that swings with the tool.
4. **The approved poses at 2x size.** The pickaxe, axe and shovel are held
   upright with the head pointing ahead. The rod stands at 80 degrees.
5. **The slot icons are drawn from the same hexes.** `gen_held_tools.py`
   writes the four tool PNGs; `gen_item_icons.py` stops drawing them. The icon
   and the model then come from one shape.

## Not in scope

The tool-in-hand clipping into walls (a separate pass), felling trees, and
the 1x size and the other densities, which exist only in the mockup's
switches.
