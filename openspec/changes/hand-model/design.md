# Design

## 1. Measured today

- **Parts:** 19 prisms per tool from `held_tools.ron`, all in tool units:
  - one palm prism, width 2.0 and height 0.70;
  - four fingers of three prisms each;
  - the thumb, three prisms in the game; the proposed hand draws it as two
    (owner, 2026-11-17: "Two thumbs segments not 3"), shortened to 0.72 of
    the game's length ("thumb is too long"), the two halves equal ("should
    be 50/50 for the thumb");
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

## 4. As built (2026-11-17)

The proposed hand of Tools in Hand version 8, approved by the owner.

- **Blocks.** `tools/gen_held_tools.py` ports `proposedHandGeometry`: each of
  the game's bones becomes a chamfered block (the octagonal section, the
  bevelled ends, the fingers at 0.94 of the bone turned so a flat face is the
  back, the back of the hand 1.08 deep, the wrist 1.15 long into the cuff),
  the thumb two segments of the 0.72-shortened chain split half way along it,
  a nail plate on each tip, a hem ring on the sleeve, and the wrist joint the
  arm turns about. They go into `held_tools.ron` as `hand` blocks (with
  `part` names) and `wrist`, in place of the 19 prisms. The sleeve's extra
  length is the game's (`held.ron` `stroke.sleeve_extra`, 6 tool units),
  along the sleeve block's `grow`. `pbd_core::hand::block` builds the faces,
  each wound out from its block's centre with its own flat normal, a planar
  UV at one texel per 0.24 tool units (along the cuff, end to end), and the
  knuckle crease on the bevels.
- **Textures.** `handTextures` ported with its generator: the four 16 px PNGs
  in `assets/textures/hand/` are the mockup's pixel for pixel (checked
  against the page's own function run in node). Nearest-sampled and tiled;
  the v coordinate is flipped for wgpu.
- **Light.** The cheaper honest one: the vertex colours are re-baked on the
  CPU (`held::shade_hands`) whenever the hand's turn into eye space changes
  by more than a milliradian, with the light and the fill taken into the
  model through the drawn pose (the arm's through its bend). A shader would
  be a new material and pipeline for about 4,600 vertices a hand; the bake
  touches only the shown hand, and only while it turns. The shade's three
  values are `held.ron` `hand_light` (the mockup prints them as a comment).
  The materials stay unlit and take the field's light at the eye
  (`light_held`), as before.
- **One skin.** The hand's colour is the textures': skin, nail, cloth and
  cuff. The hem is drawn in the cloth texture, as the mockup's textured hand
  draws it.
- **The mockup's own data.** `tool-motion.html`'s `HELD` data still holds
  the game's old prisms, which its Today mode draws; it is not regenerated.

Frame cost was not measured (a cloud container, no GPU).

## 5. Palm and back (proposed, 2026-11-17)

The owner, on the hand in the game, with two reference photos of real
hands: "usually back of the hand and fingers a little darker than palm,
and palm has kind of that palm texture". One skin tone (section 2) reads
too even next to a real hand, where the back is darker and a little
browner and the palm is paler, pinker and creased.

Proposed (in the mockup first; the game waits for the owner's look):

- **Two sides of one skin.** Every face of the hand's blocks (the back of
  the hand, the fingers, the thumb, the wrist) is the back or the palm by
  its normal: a face turned toward the handle (the palm's side of the
  fist, the inside of each curled finger and the thumb) is the palm, the
  rest is the back. The side is decided per face from the part's own
  frame, so it holds in every grip.
- **The back** is the skin tone a little darker (`back_k`, 0.92 of the
  skin), with the same mottle.
- **The palm** is the skin lifted toward a pale pink (`palm_mix`, 0.28 of
  the way to (246, 206, 190)), with its own 16 × 16 texture: the mottle,
  plus darker crease rows across it (the palm's lines and the finger
  joints' folds), nearest-sampled at the same one texel per 0.24 tool
  units.
- The nails stay as they are; the skin picker's other tones get the same
  two sides.
- A **Palm and back** switch on the Hand tab shows it against version 8's
  one tone.

In first person the back of the fist is what the player sees most, so the
whole hand reads slightly darker; the palm shows in the gaps of the grip,
inside the curled fingers and when the hand turns.

### 5.1 Version 10: stronger tones, the palm unwrapped (owner, 2026-11-17)

On version 9: "Colors suck. I don't see any difference on palm vs hand,
palm lines look wrong. You need to properly UV unwrap hand so the texture
for the palm work". Measured on version 9: the back was 0.92 of the skin
and the palm 0.28 toward pink, a step smaller than the wrapped light's own
range, so the shade hid it; and the palm's 16 × 16 tile repeated every
3.84 tool units in a planar UV per face, so its crease rows landed as
straight stripes in random places.

- **Tones:** the back is the skin darker and browner (red × 0.86, green
  × 0.82, blue × 0.80: (195, 144, 110) on the game's skin); the palm is
  0.45 of the way to (252, 214, 204): (238, 193, 165).
- **The palm is unwrapped:** every palm-side face of the back of the hand's
  block takes one UV island, projected on the palm's plane in the block's
  own frame: u across the palm from the thumb's side (0) to the little
  finger's (1), v from the wrist (0) to the knuckles (1). Its texture is
  32 × 32 and drawn for that layout: the heart line under the fingers,
  running in from the little finger's edge; the head line across the
  middle; the life line curving round the thumb's base to the wrist; a
  row of short folds at the fingers' base.
- **Fingers, thumb and wrist:** their palm-side faces take one island per
  segment, u across and v along the bone, with a 16 × 16 texture that has
  a fold at each joint (both ends of the segment); on the wrist those are
  the wrist's creases.

### 5.2 Version 10: the shape (owner, 2026-11-17)

On version 9 and its first fixes, with two photos of the owner's own fist
seen from the eye ("Use my hand as reference") and the Sketchfab "Female
hands" scan (reference for colour and form only; nothing copied): "Tip of
thumb could be rounder. It doesnt look much like a thumb", "Bake some ao",
"Maybe palm doesn't work as a square", "Thumb needs to be more connected
to the palm", "I REPEAT A RECTANGLE IS NOT A GOOD PALM SHAPE", "fingers
look like sausges", "thumb is really big".

- **The palm is one traced piece:** its outline is taken off the bones in
  the palm's plane (the four knuckle bases, the wrist, the thumb's root):
  full across the knuckles, narrowing to the wrist on the little finger's
  side, swelling out on the thumb's side (the mound at the thumb's base) up
  to the thumb's root, and the web back to the index knuckle. It is
  extruded through the hand's thickness (thinner toward the wrist) with
  bevelled edges and a slightly domed back. The rectangular block and the
  separate mound of the first fix are gone.
- **Fingers:** 0.8 of the bone's width and 0.84 of its depth, flatter
  sides (chamfer 0.26), each segment tapering toward the tip.
- **Thumb:** 1.08 × 1.04 of a finger, its tip a rounded pad (a two-step
  bevel), its nail broader and further back.
- **Which side is the palm:** where the fingertips curl to, square to the
  bone (the first rule, toward the handle, picked the wrong face).
- **Baked AO:** each vertex darkened by the other bones and the handle,
  each a few spheres, by sum (r/d)² · max(0, n·d), k ×= 1 − 0.6 · min(1,
  sum); once, at build.
- **The fingertips flush redder:** the last segment of each finger and the
  thumb tinted 0.22 toward (226, 140, 128).
- **Grip turn:** the hand, wrist and sleeve turned about the handle through
  the fist (`grip_turn`), so the fist stays closed on it. The owner's photo
  shows the back of the hand and the thumb toward the eye; 45° matches it.
  It changes the arm's line in every stroke, so the owner judges it in the
  Dig and Chop tabs too.
