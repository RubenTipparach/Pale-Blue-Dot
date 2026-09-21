# Design: what each fix touches, and what the audit is

## The tile table, read rather than retyped

`planet_surface.wgsl` picks a tile per material code in one `if code==Nu`
table, and `assets/tilesets/biomes.json` names all sixteen tiles of every
sheet in row-major order. `tools/block_audit.py` reads BOTH - the table with
a regex over the shipped shader, the names from the JSON - and lays out, per
code and per sheet, the tile the cap samples and the tile the side samples,
cut from `atlas.png` exactly as `pixel_tile` addresses it. Its text output is
the same fact as a table, which is how the two wrong rows were found:

| code | what it is | tile it sampled | the tundra / beach sheet names it |
| --- | --- | --- | --- |
| 6 | snow | `(3,0)` = #3 | cold granite |
| 0, 1, 4 | seabed, beach, desert sand | `(3,2)` = #11 | frozen footpath / packed beach trail |

Every sheet keeps the same layout - `#0` its ground, `#1` that ground over
the earth, `#2` the earth, `#3` the stone - and the sand codes now take `#0`
of their own sheet (seabed sand, dry shell sand, pale sand), while snow takes
the tundra sheet's `#10`, "wind packed snow": the snowfield rather than the
snowy moss at `#0`, because the Snow material is what caps a field above the
snow line in any biome, and its side `#1` "snow over soil" pairs with either.

`the_shader_draws_each_material_on_a_tile_named_for_it` reads the shader's
table and `biomes.json` in Rust and holds: snow's tile is named with "snow" on
the tundra sheet; the three sand codes' tiles are named with "sand" on the
beach, desert and ocean sheets; stone is `#3` and grass `#0` on the fields
sheet. It is the audit's text output as an assertion, so the next code that
points at a path or a granite fails a test rather than waiting for a
screenshot.

Two rows the audit shows and this change leaves: leaves (`#8`) on a sheet with
no tree is "beach grass ground" or "lichen stone", and wood (`#6`) on the
mountains sheet is "granite". A tree draws from its cell's biome sheet, so a
tree on a beach or a mountain would wear those. The reference grows no trees
there and neither does the generator today; it is recorded so that the day a
tree does, the picture has a name.

## The haze, gated

```wgsl
let fog = (1.-exp(-distance_to_camera*0.00036))*air*daylight;   // before
let fog = (1.-exp(-distance_to_camera*0.00036))*air*daylight*skylight;
```

and the rim term takes the same factor. `skylight` is already the record's
baked occlusion times the field's own level at this vertex, so a cave mouth
fades its haze in over the fifteen cells the light takes to fade out, and a
sealed room has none. This is exactly Tenebris's `rim_gate * v_sky_light`.

## The droplets, in the frame the math assumes

`rd_drop_layer` is the reference's function line for line, and the reference's
comment says what frame it is in: "v_uv is y-up, so high y = high on screen",
and the fall is `y = 0.92 - ti*0.88`, high to low. Bevy's fullscreen `uv` is
y-down. `drop_uv` is now `(uv.x - 0.5, 0.5 - uv.y)`, and the refraction offset
it derives, which is a gradient in that frame, has its y negated on the way
back to the y-down sample. Nothing inside the reference's math changed.

## The vertical contact term

The reference's ladder counts the two SIDE columns at the face's own layer.
For a cap that is the complete rule - three columns meet at a corner and the
plane in front of the face is the air layer above it. For a wall, the plane
in front of the face is the column ACROSS, and a vertex on the wall's bottom
edge has three cells that can shadow it: the side column at the wall's layer
(counted), the across column one layer below (the floor it stands on), and the
side column one layer below. Minecraft's three-neighbour AO, which is where
the reference's "Notch's ladder" comes from, counts all three; the reference
kept the one. So did the port. A wall on a floor is therefore lit the same at
its foot and its middle - 92 and 92 of 255, probed on the pit capture - and a
wall's top edge under an overhang is lit as if nothing were over it.

`light::CONTACT` gains a fourth rung, `[1.0, 0.85, 0.70, 0.55]`, and
`light::wall_corner` counts the two cells past the edge - across and beside,
one layer toward the face's edge - on top of `corner`'s side count. The
shader's `wall_light` transcribes it: `corner_at` returns its occluder count
instead of applying the ladder, and the wall adds its two before indexing.
The cap path is untouched. A test holds a wall on a flat floor to a foot
darker than its middle, and a wall under a lid to a head darker than its
middle; the constants test gains `CONTACT_3`.

Measured on the pit capture: the wall's foot 78 of 255 against 86 at its
middle (92 against 92 before), and the cave's mean colour from (21.8, 25.3,
28.2) - a blue cast - to (19.0, 19.6, 20.2) with the haze gated.

This diverges from the reference on purpose and says so. The owner's picture
is the three-neighbour rule; the reference had the two-neighbour rule and its
own comment names Notch's ladder as the source.

## The dithered face: what was found, and what would settle it

`--dig 3 --dig-ahead --place` on the fixed build draws the placed stone clean:
one cap, one material, no second face. The side-face audit passed on that
scene, and the horizontal faces of a cell cannot coincide by construction -
a run's top cap is skipped where it is the record's cap, and runs are a layer
apart. What the picture shows is a `DIRT` cap seen at a grazing angle: tile
`#2` "crumbly dirt" is brown with grey pebbles, and `pixel_tile` is
`textureLoad` with no mip chain, so at a grazing angle every texel fights
its neighbour and the pebbles read as a second texture. The honest next step
is a capture at `--pitch -8` over a dug floor, which is the angle in the
picture; if the dither is there, the fix is a nearest-filtered mip level
chosen by distance, which keeps the point-sampling rule and stops the
shimmer. Held until that capture says so.

## A face wears its own voxel, not a depth rule

The owner built a tower of placed stone and its sides came out sod, then
earth, then stone by height. The STORAGE was right: `apply_edit` writes the
held item's material into the column and the log, and the column is what the
walker, the aim ray and the bake read. What inferred was the DRAWING: a flank
carried one material per run - the run's top - and `face_code` turned that
into sod, earth and stone by depth under the run's top. That rule is the
generator's own stack, and it is exactly right for a column the generator
made, which is why it survived; it is exactly wrong for a column somebody
built, because a placed stone is a stone at whatever depth it sits.

Tenebris draws every face in its own voxel's tile: `face_tile(block, cap)`,
with the top of a grass block the grass, its side the transition and its
underside dirt. So does this now. `ColumnTier::gpu_materials` packs every
layer's render code - four bits, eight to a word, forty words a column - into
a storage buffer beside the light, uploaded with it; `material_at(slot,
layer)` reads it back in the shader; and a column-pass face (kind 4) takes its
code from the layer it stands on rather than from its run: the block below an
up-facing cap, the block above a down-facing one, and for a flank the layer at
the fragment's own altitude. The side rule is the reference's `face_tile`:
the sod's side is the transition, snow's side its transition, a grass block's
underside earth, everything else its own tile.

The generator fills its layers by `material_at_depth`, so a natural hillside
draws exactly what the depth rule drew - sod, four metres of earth, stone -
because those are the layers. Nothing was inferred; the layers were always
there and the drawing stopped asking the run for them. The heightfield's own
walls (kind 1), which stand only where a cell has no column, keep `face_code`,
because there is no layer to ask.

`--place N` in the harness stacks N stones on the last hole, so a built tower
can be photographed; before this it placed one.
`docs/screenshots/placed-stone-tower.png` is five placed stones seen from
the pit they stand in: stone on every face, at every height, with the pit's
dirt floor under them. `docs/screenshots/dug-pit-materials.png` is a three
layer pit with nothing placed: the sod's side on the top layer, earth below,
which is exactly what those layers are, so a natural wall draws what it
always did. `docs/screenshots/cave-stone-no-haze.png` is the cave view on the
same build, stone all round and no atmosphere in it.

