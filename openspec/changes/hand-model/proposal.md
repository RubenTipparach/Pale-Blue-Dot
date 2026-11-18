# A better hand

## Why

The owner, 2026-11-17:
- "The dudes hand kinda of has weird normals";
- after the side-face normal fix (`rod-motion` 1.1): "hand looks the same still";
- "hey I asked for improved hand model, better normals, better shading, more uniform colors, hell
  throw in some textures".

The first-person hand today is in `assets/models/held_tools.ron`, written by
`tools/gen_held_tools.py` from `tools/held_hand.json`. It is 19 hexagonal prisms, posed from
Blender's Rigify metarig and drawn by `hexel::prism`. Four things make it look wrong:

1. **Colours.** The middle segment of every finger is painted a second, darker skin tone,
   (195, 151, 118) against (227, 176, 137). Each finger gets a dark band that reads as a
   badly shaded face.
2. **Shading.**
   - The baked shade only runs from 0.62 to 1.0, so the hand's form barely shows.
   - The light is fixed in the tool's frame at the rest pose, so it is wrong once the tool
     swings.
3. **Model.**
   - Hexagonal prisms meet at the joints with their caps showing.
   - The palm is one flattened prism.
   - There are no nails and no wrist.
4. **No texture.** Every face is one flat colour.

## What changes

A new hand, still in the game's low-poly style:
- a bevelled palm;
- three-segment fingers and thumb that close round each tool's grip without gaps;
- nail plates and a wrist into the cuff.

It has:
- one skin tone, one sleeve cloth and one cuff band;
- correct face normals;
- wrapped light in eye space each frame, with a soft fill and the field light's colour;
- small pixel textures, nearest-sampled: skin with knuckle creases and nails, a cloth weave on
  the sleeve, stitching on the cuff.

It is designed in the Hand tab of the Tools in Hand mockup (M36), beside today's hand. The game
changes after the owner approves it there.

## Impact

- `tools/gen_held_tools.py`, `tools/held_hand.json`, `assets/models/held_tools.ron`.
- New `assets/textures/hand/*.png`, with provenance (drawn in code).
- `crates/pbd-core/src/hexel.rs` or a new hand mesh builder.
- `crates/pbd-app/src/held.rs`: the material gets a texture, and its light follows the eye.
- No rule, save or generation change.
