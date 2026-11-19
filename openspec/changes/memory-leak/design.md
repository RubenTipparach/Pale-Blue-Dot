## Context

`fish::draw::schools` runs every frame in `Update`. For each school entity
it calls `build(mesh, ...)` on the school's `Mesh` (`meshes.get_mut`), which
starts six empty `Vec`s (positions, colours, uvs and the three swim
attributes `swim.wgsl` bends by), lays every creature into them through
`bodies::emit_swimmer`, inserts them, and calls `compute_flat_normals`,
which collects a seventh. The old buffers are dropped when the inserts
replace them. A school of a few thousand vertices is a few hundred KiB a
buffer, so each frame walks each buffer through every doubling from 4
entries to its size and frees the lot.

## Decisions

1. **Take the buffer back, keep its capacity.** `Mesh::remove_attribute`
   returns the attribute's `VertexAttributeValues`; the `Float32x2/3/4`
   arm is the `Vec` itself. It is cleared and laid into, then inserted
   again. A school whose mesh has never been built (its first frame, or an
   attribute of another type) starts from an empty `Vec`, as today.
2. **Flat normals written in place.** The same arithmetic as
   `Mesh::try_compute_flat_normals` (`triangle_normal(a, b, c)` once per
   three positions, repeated for the three vertices), into the normals
   buffer taken back the same way. `compute_flat_normals` would allocate a
   new `Vec` every call.
3. **The hidden triangle** a school with nobody drawn keeps (so its mesh
   keeps the layout the loading screen warmed) is laid into the same kept
   buffers rather than new `vec!`s.
4. **The kelp's sway** (`places::bend_mesh`) writes into the positions it
   holds when the count matches, rather than collecting a new `Vec`.
5. **Kept in both worlds, as today** (`RenderAssetUsages::default()`):
   making the school meshes render-world only would move the data instead
   of copying it, but a mesh whose data has gone to the render world cannot
   be laid into again; it would trade the extraction copy for a fresh
   buffer each frame, the same churn.

## Proof

- **Bit for bit**: a test lays a school into a fresh mesh and into a mesh
  already laid, and compares every attribute; another lays the same school
  twice and checks no attribute's buffer moved or grew (its pointer and
  capacity are the same), which is what "allocates nothing" means here.
- **Memory**: the same 15-minute walk as the finding, on `main` and on the
  branch, release, one after the other on a quiet machine, RSS every 5 s and
  `phys_footprint` every 30 s, compared minute by minute; the
  `alloc-trace` build's table for the same window, before and after.
- **Frame time**: the same runs' `--frame-log`, p50 by minute. The walk is
  the same command and seed; where the two walkers' paths part, minutes are
  compared only while their `walked_m` agree.

## Risks

- A buffer kept at its largest size holds that memory while the school
  lives: at most the size of its largest frame, freed with the school.
