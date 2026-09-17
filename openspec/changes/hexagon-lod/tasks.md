# Tasks

## 1. Prove the seam before building the tiers
- [ ] Prototype the band boundary: two adjacent LOD levels on the real topology,
      in a still frame, at the worst angle. A mockup before a large feature is
      this repository's rule, and a seam is what a still frame settles.
- [ ] Pick the selection rule from `design.md` on that evidence.

## 2. Per-level topology, uploaded once
- [ ] Build corner rings for every level from the finest down, and confirm the
      measured total is about `4/3` of the finest level alone.
- [ ] Confirm against the real `dual_sphere` that cell `i` is the same direction
      at every level that contains it, as a test rather than a comment.
- [ ] Coarse levels index the existing height array; no per-level height data.

## 3. GPU level selection and culling, one-way
- [ ] Extend `planet_visibility.wgsl` to write a level per visible tile
      alongside the compaction it already does. One pass, not two.
- [ ] Keep the draw indirect and the readback at zero. Nothing about a tile
      returns to the CPU.
- [ ] Keep the 32-group coarse cull (12 icosahedron vertices plus 20 face
      centres) if it measures better than testing every tile.

## 4. The far tier is hexagons
- [ ] Draw the far tier from the same vertex-pulling path and the same shading
      terms as the near tier.
- [ ] Do not port `distant.fs.glsl` or any triangle impostor.

## 5. The culling the pass already pays for but does not do

Found while exploring, independent of LOD and cheap. These belong here rather
than in their own change because they touch `planet_visibility.wgsl`, and
touching one pass twice for two reasons is exactly the divergent-path risk this
project's rules warn about.

- [x] **Frustum cull.** Use the bound body-local clip matrix in the visibility
      pass, retaining conservative cell bounds that intersect a frustum plane.
      The previous pass performed only sphere-horizon culling.
- [x] **Per-cell foliage distance.** Select nearby vegetated cells before vertex
      submission, instead of submitting 108 tree vertices for every visible cell
      and degenerating the distant/nonvegetated instances in the vertex shader.
      Keep the existing conservative 3,200 m altitude gate as an early disable;
      the GPU owns the per-cell 2,300 m range and material/seed predicate.
- [ ] Measure both, before and after, in a reproducible release scene. Neither is
      a speedup until it has a number.

Implementation: the GPU tests body-local cap/wall bounding spheres against all
six homogeneous clip planes, and retains intersecting bounds conservatively.
Nearby vegetated cells compact into a separate ID buffer and 108-vertex
indirect foliage draw; terrain uses 54 vertices. The compute pass owns the only
foliage eligibility test. There is no runtime GPU readback. This implements the
independent culling fixes; tier selection, seams and performance measurement
remain open.

## 6. Then the radius
- [ ] With LOD landed, revisit the radius in
      `preview-scale-and-shader-parity`. The resident cost stops being
      `10*4^L + 2` for the whole globe, so the ladder stops being the
      constraint that picks the planet size.
