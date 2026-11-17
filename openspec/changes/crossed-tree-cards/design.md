# Design: crossed tree cards

## 1. Where the billboard was

The impostor draw invokes 30 vertices a tree (`args[1]` in
`planet_visibility.wgsl`, `foliage_vertices` in `planet.rs`), drawn by the
tree branch of `vertex` in `planet_surface.wgsl`:

- vertices 0-23: the mid ring's four cards, card `k` in the plane at
  `k * 45` degrees on the tree's turn (`t.x`, `t.z` from `SALT_TURN`), showing
  baked view `k` from its front and `k + 4` from its back;
- vertices 24-29: the far billboard, its axis `t.x * dir.y - t.z * dir.x`
  where `dir` is the eye's bearing in the tree's frame, so it turns to face
  the eye, showing the baked view nearest that bearing.

From above, `dir` points from the tree to the point under the eye, so every
billboard lies along a ray from the view centre: the owner's streaks.

## 2. Two fixed cards for the far ring

The far ring draws cards 0 and 2 of the mid ring: the plane on the tree's
turn and the one square to it, an X standing on the base along the local up.
They never turn. Each shows the bake taken from its own side, as the mid
cards do, so cards 0 and 2 show views 0/4 and 2/6, which the bake has
exactly. Three cards at 60 degrees were weighed and not taken: the bake's
bearings are 45 degrees apart, so the 60 and 120 degree planes would show a
view 15 degrees off their own, and the far ring would no longer be a subset
of the mid ring's planes (section 4).

A forest does not look tiled because every tree's X stands on its own turn,
`roll(id, SALT_TURN)`, the roll the near model and the mid cards already
use; nothing about placement or the rolls changes, and none of it is saved.

## 3. The crown cap

A vertical card seen from straight above is a line, so two cards are a plus
sign. Each tree therefore gets one horizontal quad, the cap:

- in the plane through the tree's axis at `CAP_HEIGHT` (0.55) of the
  model's height, spanning the bake's width on the tree's own `t.x`, `t.z`;
- showing a ninth baked view, the near model drawn orthographically from
  straight above (`tree_bake`, view 8), width by width over the view's
  48 x 72 pixels; the side views lay a bough's bark over the crown where it
  fills a block, but from above the crown hides the trunk, so in this view
  the bark shows only where no leaf is;
- snowed on all over in winter (a side view snows on its top third);
- drawn only while the eye is above its plane. The test is geometric, not a
  dither: the cap is edge-on, zero area, at the moment the eye crosses its
  plane, so it never appears or vanishes as a shape on screen. From the
  ground the far caps are above the eye and are not drawn.

**Is a cap within "2-4 billboards"?** Recommended yes: the owner's
complaint is cards that rotate, and the cap is fixed like the X; it is the
fourth or fifth fixed plane, and without it a forest from above, the
owner's own view, is a field of plus signs. Recommendation taken (ask only
with screenshots); the before-and-after shots from above are in
`docs/screenshots/crossed-tree-cards/`, where the owner can overrule it.

The cap is drawn on the mid ring too, so a tree seen from above does not
change its top when it crosses from mid to far.

## 4. The fade

`tree_show` gives each representation its fade (1 whole, `1 - t` going,
`2 + t` coming, 0 not drawn) and is unchanged. Cards 0 and 2 and the cap
are now drawn by both the mid and the far ring, so they take a combined
fade, `ring_show`: whole while the tree is mid or far at both ends of the
fade, coming in from near or none, going out to near or none. Cards 1 and 3
keep the mid ring's own fade. The mid-to-far change is then the two
diagonal cards dithering out over the fade time, and nothing pops; a still
camera sees no dithered tree once the fade time has passed, as before.

## 5. Lighting

Every card and the cap take the one normal the tree branch already computes
once per tree, toward the eye across the ground and `IMPOSTOR_LIFT` up
(`flora-v2` design 13.4). No plane is lit apart from the others, so the X
never shows one light arm and one dark, and the cap is no brighter than the
cards.

## 6. Cost

- Vertices invoked per tree: 30, unchanged (cards 0-3 then the cap in the
  slot the billboard had). Clutter and the column tier keep their first
  vertices.
- Triangles rasterised: a far tree 6 (was 2), a mid tree 10 (was 8).
- The kit: each bake is 9 views instead of 8, 432 x 72 words instead of
  384 x 72, 3,456 words (13.5 KiB) more a bake: the kit logged 6,078 KiB
  before and 6,672 KiB after. The bake runs at startup and draws one view
  more. The crown means the shader matches near and far colour by are still
  taken over the side views alone, so no tree's colour moved.
- Frame cost was not measured: the cloud container has no GPU and renders on
  lavapipe. The owner's perf sitting measures it.

## 7. Saves

Nothing saved changes. Where trees stand and which cells can be chopped are
`pbd_core::flora::grows_tree`'s, and a tree's species, variant, turn and
size are rolled from its id at draw time, exactly as before.

## 8. With the baked relief (merge, 2026-11-17)

`tree-impostor-depth` landed beside this change and lights each impostor
pixel by the near model's own baked normal and AO. Section 5's one normal
per tree is kept as that change's standing normal (what the knob
`tree_impostor_relief` mixes from), and each card and the cap now carry
their baked view's level direction for the relief to be rebuilt in. The
top view has its own relief strip (the strip and its relief are nine views,
`IMPOSTOR_RELIEF` 31104 words). Because the baked normals are the model's
own in the tree's frame, every plane of a tree takes the same light from
the same leaf, which is what section 5 asked for. Nothing of either change
is given up; `tree-impostor-depth` design 5 has the detail.
