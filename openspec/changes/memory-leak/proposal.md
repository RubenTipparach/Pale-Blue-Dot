## Why

The owner, 2026-11-19: "there's a memory leak somewhere". A 15-minute walk
on `main` (`df18dcc4`, release, `PBD_WEATHER_SCRIPT=0:dry --world leak-walk
--time 10 --no-vsync --walk-distance 50000 --frame-log`, M4 Max) took the
process's resident size (RSS, what `ps` and `top` show) from 1.73 GB at
minute 1 to 2.53 GB at minute 15, 60-75 MB a minute from minute 6 on, and
it was still climbing when the run stopped.

Every cut here keeps the picture and the sound (CLAUDE.md, "Performance
improvements need no permission").

## What was measured

Instruments, both measurement only (CLAUDE.md, "Write it up before touching
code"): they count, and change nothing drawn or simulated.

- `PBD_MEMORY_TRACE=<csv>` (`memory_trace.rs`): every 2 s, the entities in
  the main and render worlds and the count and bytes of each asset kind
  (meshes, images, the material kinds).
- The `alloc-trace` cargo feature (`alloc_trace.rs`, `PBD_ALLOC_TRACE`):
  the allocator counts every allocation of 256 KiB or more by its call
  stack, walked off the frame pointers, and writes the table once a minute;
  `tools/alloc_trace.py` names the stacks with `atos`.
- `footprint <pid>` and `vmmap --summary <pid>` every 30-60 s beside RSS.
  `phys_footprint` is what macOS charges the process for (Activity Monitor's
  Memory column): dirty memory, GPU (IOAccelerator) memory included, and
  memory the allocator has freed and marked reusable excluded. RSS counts
  pages still mapped, reusable or not.

Findings (`docs/benchmarks/2026-11-19-memory-leak/`):

1. **Nothing the game holds grows.** Over a 15-minute walk the asset counts
   are flat after the first half minute (518-554 meshes, 213 images, 131
   standard and 39-43 field-lit materials, 11,648 entities, 24-27 MB of mesh
   data), and `phys_footprint` stays at 3.05-3.10 GB from minute 1 to
   minute 15, while RSS goes from 1.83 to 3.11 GB.
2. **What grows is memory the allocator has been handed back.**
   `footprint` puts the whole rise in the `Reclaimable` column: freed
   medium blocks (745 MB at minute 1, 1,085 MB at minute 15) and freed large
   blocks kept for reuse (`MALLOC_LARGE_REUSABLE`, 373 MB to 1,236 MB, 44
   regions to 403). macOS keeps these pages mapped, so RSS counts them,
   until the system needs them; the dirty columns do not move. Standing
   still (`--stand 600`) the same happens on a smaller scale and stops:
   RSS 1.77 GB to 2.23 GB by minute 5, flat after; footprint flat at 2.87-2.90
   GB throughout.
3. **Why the allocator holds so much: a large buffer churn of about 8 GB a
   minute.** Over a 5-minute window of the walk (minutes 5-10) the
   allocations of 256 KiB or more came to 41.9 GB:
   - **5.4 GB a minute: the fish schools' meshes, laid afresh every frame.**
     `fish::draw::schools` rebuilds each school's mesh every frame into new
     `Vec`s that start empty and double as each creature is laid
     (`bodies::emit_swimmer`, `emit_with`): six attribute buffers of
     300-400 KiB each, grown through every power of two, and a seventh,
     the flat normals, made new by `Mesh::compute_flat_normals`.
   - **1.7 GB a minute: Bevy's extraction copying those meshes.** A mesh
     kept in the main world (`RenderAssetUsages::default()`) is cloned
     whole into the render world each frame it changes
     (`RenderMesh::extract_render_asset`).
   - 0.5 GB a minute: the fine set and its tier (landings, water ticks).
   - 0.12 GB a minute: the river cache saved every 30 s while it grows
     (84 MB of file built in memory, then its memos copied out, about 150
     MB at a time).
   - 0.09 GB a minute: ambience beds decoded whole each time one comes up
     (7-14 MB each), dropped when it falls silent.

   The sizes change every frame (the creatures laid, their poses), so a
   freed block rarely fits the next request and the allocator keeps
   mapping more and keeping the old.

So this is not a leak: nothing is lost, and no count grows. It is churn,
and it costs twice: RSS that only memory pressure gives back, and the main
thread's time spent copying fish into new buffers every frame.

## The frame-time step

The first walk's step at minute 8 (p50 15 to 22 ms) is all in
`acquire_ms`, the CPU waiting for the GPU to hand back a frame; the main
world's own time (`cpu_main_ms`) stayed at 3 ms. It is not memory pressure:
in the fast-profile repeat, which stepped at minute 5, `phys_footprint` is
flat across the step, and so is every asset count. Whether it is the view
the walker reached is not settled: GPU timestamps read NaN on this Mac, and
the sitting for the before-and-after runs had the screen locked, which
paces every frame to 15.2 ms (report, section 4).

## What changes

1. **A school's mesh is laid into the buffers it already holds.**
   `fish::draw::build` takes each attribute's buffer back out of the mesh,
   empties it (the capacity stays), lays this frame's creatures into it and
   puts it back; the flat normals are written into their own kept buffer
   the way `compute_flat_normals` works them out (`bevy::mesh::
   triangle_normal` of each triangle, three times), so the vertices and the
   picture are the same, bit for bit. After a school's first frames a
   frame allocates nothing for it. The kelp's sway (`places::bend_mesh`)
   gets the same.
2. The extraction copy stays: it is how Bevy hands a mesh kept in both
   worlds to the GPU. It is one exact-size copy a frame rather than a chain
   of doublings, and it is freed in the same frame.
3. The river cache save, the beds and the fine-set churn are smaller and
   rarer; they are noted here and left for the owning changes
   (`river-cache`, `audio-and-ambience`, `fine-set-upload`).

## Not changed

- No picture or sound changes: the same vertices go to the same draw.
- No allocator swap (mimalloc and similar): it would hide the churn, not
  remove it, and changes every allocation in the game.

## Results

`docs/benchmarks/2026-11-19-memory-leak/report.md`:

- The branch's walk has no allocation of 256 KiB or more under
  `fish::draw` or `fish::bodies` (5.4 GB a minute on main).
- Release, main against the branch one after the other, screen locked
  (the only sitting available): the main thread 0.13-0.15 ms a frame
  faster every minute (p50 `cpu_main_ms` 3.43 to 3.30 ms), RSS 100-150 MB
  lower; both flat, since a locked screen paces the game and stops the
  fine set landing.
- Owed: the same walk with the screen unlocked, where the RSS climb shows.
