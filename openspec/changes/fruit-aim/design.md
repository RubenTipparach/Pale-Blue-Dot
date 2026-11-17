# Design

## Measured

The draw hangs up to four fruit on a near tree (`planet_surface.wgsl`, the
near tree's fruit part): item k in 0..4 is kept when
`roll(id, SALT_FRUIT+2+k) < keep` (the season's keep, as `hanging_fruit`
counts), hangs from anchor `min(roll(id, SALT_FRUIT+10+k) * anchors,
anchors-1)` of the tree's model (`tree_kit::build(species, variant).anchors`),
placed by `tree_point` in the tree's frame (`tree_kit::pick`'s turn and size,
the cell's tangent from the shader's reference axis), and is a card `size`
tall below that point, `size = 0.40 (0.46 for a cone) * (0.9 + 0.3 *
roll(id, SALT_FRUIT+30+k))`.

## Decisions

1. `pbd_core::flora::hanging_fruit_points(tree_id, biome, axis, radius, height,
   year, chance) -> Vec<([f32; 3], f32)>`: each kept fruit's centre (half its
   size under the anchor) and a hit radius (half its size plus 0.1 m for the
   sway the CPU does not model). The models' anchors are built once and kept.
2. `fruit_on_ray` tests the ray against those spheres for the fruiting trees
   near the eye, nearest hit within reach and before solid ground, and returns
   the tree's cell. A felled or picked tree has none to hit.
3. A test holds the port's literals to the shader's lines, as `hanging_fruit`'s
   test does.
