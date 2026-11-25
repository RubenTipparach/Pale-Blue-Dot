# Provenance of the critters' bodies

The beaver, the river otter and the sea otter (`openspec/changes/ocean-environments`
design B28 and B29), each one furred body sculpted as a signed distance field,
meshed, cut to a low-poly budget, skinned to its rig of twenty bones and painted
as pixel-art fur, with fur tufts. Made for this repository by
`tools/art/critter_sculpt.py`; the method is adapted from Mining Mike's Ice Wolf
(RubenTipparach/godot-sandbox at `1739e89`: the `organic-creature` and
`painted-bake` skills, `ice_wolf_sculpt_model.py`, `ice_wolf_bake.py`,
`gen_ice_wolf_sculpt.py`), with its SDF kit copied as `tools/art/sdf_kit.py`.

The game's copy is the approved mockup's (M25 version 5,
`docs/mockups/ocean/critters_mesh.js`), byte for byte:

    python3 tools/art/critter_sculpt.py --game --from-js

| File | What it is |
| --- | --- |
| `beaver.bin` | the beaver (`critters_mesh.js` `beaver`) |
| `river_otter.bin` | the river otter (`otter`) |
| `sea_otter.bin` | the sea otter (`sea`) |
| `../../textures/critters/fur-*.png` | their fur maps (`docs/mockups/ocean/fur-{beaver,otter,sea}.png`), nearest sampled |

`PBDCRIT1`, little endian: the magic; u32 version (1), bones (20), vertices and
triangles; each bone's rest joint (3 f32, mesh space: x forward, y up, z to the
left; in the order body, hind, chest, neck, jaw, hip/knee/foot 0 to 3, tail 0
to 2); then the vertices' positions (3 f32 each), normals (3 f32), UVs (2 f32),
joint indices (4 u8) and weights (4 u8, of 255), one array after another; then
the triangles (3 u16). The body's vertices come first and the fur tufts' (in
their dry shape) after them.
