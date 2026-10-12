# Design: a fourth indirect draw for ground clutter

## Where it sits in the pipeline today

Three draws come out of one compute pass. `planet_visibility.wgsl` compacts the
live cell records into three lists and writes three `DrawArgs`:

| draw | list | vertices | first vertex | pipeline |
| --- | --- | ---: | ---: | --- |
| terrain cap, walls and the cut wall | `visible` | 60 | 0 | `planet_surface.wgsl`, cull back |
| foliage | `foliage` | 198 | 60 | the same, same pipeline |
| water cap | `water` | 18 | 0 | `water.wgsl`, cull none |

`args` is `array<DrawArgs,3>`, and `planet.rs` issues `draw_indirect` at byte
offsets 0 and 16 with two bind groups; `planet_water.rs` issues the third at 32.
The foliage draw is not a second pipeline: it is the same vertex shader with
`first_vertex = 60`, so `vertex - 60u` selects the tree branch. A clutter draw
follows that pattern exactly: `args[3]`, offset 48, `first_vertex = 258`.

## The eligibility decision belongs in the compute pass

`has_nearby_foliage` is already the sole foliage decision - the comment on it
says so, and the vertex path never rejects a tree. Clutter takes the same shape,
and for the same reason: an indirect draw that submits geometry it then discards
pays for every vertex of it, and the tree branch's own history is the warning
(the reference notes 108 wasted vertices per cell from exactly that mistake).

So `has_clutter(cell, center)` answers, in the compute pass:

- the cell is at the **finest level** only. Clutter has no coarser tier: a
  coarse cell covers 4 or 16 finest cells and spreading one cell's blades over
  that area would read as a thinning sward rather than a distant one, and the
  band it would appear in is past the two-pixel range anyway.
- both owners are fine, the same clause `has_nearby_foliage` uses, so a midpoint
  cell split between a fine and a coarse owner grows nothing: its centre is the
  cut.
- the material is a grass (2, 3 or 7), which is the reference's gate on the
  surface block written in this project's own material indices.
- `distance(camera, center) < clutter_radius`, the new short reach.
- `in_frustum(center, clutter_height)` with a bound that covers the tallest
  blade, which is 0.55 m plus its lean rather than the tree's 15 m.

Everything a blade needs downstream - the corner rays, the surface radius, the
skylight, the cell id for the hash, the material for the atlas column - is
already in the record. **No record field is added and no buffer grows**, except
the clutter index list itself, which is one `u32` per cell like the other three.

## What the vertex branch builds

`vertex - 258u` indexes a blade and a vertex within it:

```
blade = (vertex - 258u) / verts_per_blade
i     = (vertex - 258u) % verts_per_blade
```

Blades past this cell's own rolled count collapse to a degenerate triangle at
the cell centre, which is how the tree branch already handles a part a cell does
not carry. The per-blade hash stream is Tenebris's, one salt per decision, over
the cell's `metadata.w` id rather than its tile index:

| decision | reference | here |
| --- | --- | --- |
| does this cell grow grass | `hash2(tile, SALT_KIND) < grass_chance` | in the compute pass, beside the material gate |
| how many blades | `(0.6 + 0.4*h) * grass_blades` | the same expression |
| where on the cap | `place(h_a, h_b)`: lerp centre toward a hashed corner by `0.15 + 0.60*h` | the same, on the record's corner rays |
| which way it faces | `h * TAU` | the same |
| how tall | `grass_height_m * (0.8 + 0.4*h)` | the same |
| how wide | `grass_blade_w_m * (0.7 + 0.6*h)` | the same |
| which atlas slice | `tile_uv_slice(ground, Top, h)` | the same, on this project's 4x4 atlas |

A blade is `segs` stacked quads whose cross-section at height fraction `f` is
the reference's own row function: the centre walks `up * h * f + lean * f*f`, so
the blade curves rather than leaning straight; the half-width tapers by
`1 - 0.55*f`; and the UV row interpolates from the slice's bottom to its top.
`grass_base_shade` (0.55) darkens the root and eases to full light at the tip,
which is what makes a blade read as lush rather than flat.

**Two segments, not three.** The reference gives a blade 2 and earns a third
when it stands over 0.7 m; the shipped height is 0.55 m, so the third almost
never fires and a fixed 2 is what the vertex budget should carry. This is the
same deliberate divergence the pine's taper already took, and for the same
reason: a `vertex_count` is one number for every instance.

## Winding, and why there is no second pipeline

The reference emits **both windings** of every scatter quad because it has one
back-face-culling pipeline. Doubling the vertex count for that here would be a
waste, because the vertex shader knows where the camera is: a blade quad is
planar, so its winding can be chosen per vertex against
`dot(normal, camera - position)` and the front face always faces the viewer. One
pipeline, half the vertices, and the same result.

The alternative - a fourth pipeline with `cull_mode: None`, which is what the
water pass does - buys nothing here and costs a pipeline and a bind group.

## The budget, and what it fixes at

At Tenebris's `grass_blades: 18` upper bound and the hash's 0.6-1.0 band, a cell
grows 11 to 18 blades. `vertex_count` must cover the upper bound, so:

```
18 blades * 2 segments * 2 triangles * 3 vertices = 216 vertices
```

That makes the shared vertex shader's range 0..474 (60 terrain, 198 foliage,
216 clutter) and the clutter draw `first_vertex = 258`.

**The 11-to-18 spread is paid for whether it is used or not**, because a
degenerate blade still costs its six vertices of transform. At the mean roll
that is 17% waste. Fixing the count at the upper bound instead - every cell
grows 18 - would remove the waste and the variation together, and the variation
is what stops a field looking stamped. Keep the spread; the waste is 0.03 M
vertices at a 60 m reach, which is nothing.

## The fade at the edge

A hard `distance < radius` gate pops a whole cell's worth of blades into
existence at once, and at 60 m a cell is 2.8 m wide and its blades are two
pixels - a line of shimmer travelling with the player. The fade is one
multiplier on `h` in the vertex branch:

```
fade = clamp((radius - distance) / fade_m, 0, 1)
```

so blades shrink into the ground over the last `fade_m` metres rather than
appearing. It costs one `smoothstep` and no vertices, because the geometry is
built either way; what it buys is that the tier's edge is not visible, which is
the only thing that makes a 60 m tier acceptable at all.

## What this deliberately does not do

- **No coarser clutter tier.** See the eligibility rule above.
- **No CPU-side scatter.** Nothing about clutter touches `pbd-core`'s
  authoritative terrain, because clutter is derived state with no collision, no
  persistence and no gameplay effect - which is exactly the category the
  architecture rules put on the GPU.
- **No new atlas art.** The blade samples the cell's own ground column.
- **No `scatter_taken` equivalent.** See decision 5 in the proposal.
- **No sway.** It needs a wind direction this project's weather field does not
  carry. See decision 3.
