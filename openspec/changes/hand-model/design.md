# Design

## 1. Measured today

- **Parts:** 19 prisms per tool from `held_tools.ron`, all in tool units:
  - one palm prism, width 2.0 and height 0.70;
  - four fingers of three prisms each;
  - the thumb, three prisms;
  - the wrist, the cuff and the sleeve.
- **Colours:**
  - the middle finger segments are (195, 151, 118);
  - every other skin part is (227, 176, 137);
  - the cuff is (58, 82, 110) and the sleeve (77, 106, 140).
- **Shade:** `0.62 + 0.38·max(n·l, 0)`, with `l` = (0.3, 0.8, 0.55) in eye space, taken into the
  tool frame at the rest pose and baked into vertex colours. The material is unlit, and its colour
  is the field light at the eye (`held::light_held`).

## 2. To decide in the mockup

These are shown in the mockup's Hand tab. The owner picks one of each.

- The palm and fingers:
  - bevelled blocks with flat faces;
  - or flat faces with a chamfer band;
  - or smooth normals.
- The light:
  - baked into vertex colours but recomputed when the pose changes;
  - or a small lit material, with the field light as an ambient term.
- The texture:
  - texel size and atlas layout;
  - nearest sampling (CLAUDE.md, "Terrain textures use nearest point sampling", applied to the
    hand too).

## 3. Port

- **Generator.** `tools/gen_held_tools.py` writes the hand's parts and UVs into `held_tools.ron`,
  and the textures into `assets/textures/hand/`. The mockup's exported hand data is its input.
- **Tests:**
  - every part's normals point outward;
  - the fist closes on each handle without passing through it;
  - the textures are on the palette and exactly the sizes stated.
