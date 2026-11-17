# mist_puffs

The waterfall's mist puffs (`water-flow` 6.5; owner, 2026-11-16: "design
better mist instead of just a gradient circle puff ... look at crab breeze
smoke puffs as a model"). Shown in the mockup `docs/mockups/waterfall-mist.html`;
not used by the game until the owner approves it.

- `albedo.png`: 128 x 128, a 4 x 4 sheet of 32 x 32 cells, sampled nearest
  with no mipmaps. Rows are four puffs, columns four frames of one puff's
  life (tight, swelling, swollen, fraying). rgb is the puff's shade on a
  five-tone ramp lit from above; alpha is its cover, faded over the outer
  three texels. No outline.
- `normal.png`: the same cells' camera-space normals (0.5 + 0.5 n, blue toward
  the eye), for lighting the puffs by the sun as Crab Breeze's `smoke_lit` is.

Made in this repository, nothing taken from outside it:

1. `tools/fx/mist_puffs_blender.py`, run headless in Blender 4.5.4:
   `Blender -b -P tools/fx/mist_puffs_blender.py -- <render dir>`. Each puff
   is a cluster of metaballs, converted to a mesh and roughened by a Voronoi
   clouds displacement, rendered face-on and orthographic in Cycles (`lit.png`)
   and again with its normal as colour (`normal.png`).
2. `tools/fx/mist_puffs_post.py <render dir> assets/textures/fx/mist_puffs`
   cuts the render to 32-texel cells (premultiplied box filter), shades it
   from the normals against light from above onto the ramp, and fades the
   edge.

The model is Crab Breeze's `assets/textures/fx/smoke_puffs` and `smoke_lit`
(a 4 x 4 sheet of lumpy puffs with normals, Material Maker); only the idea is
taken, not its pixels.
