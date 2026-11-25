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

## The reef sways on the GPU (owner, 2026-11-25)

The owner: "Coral reefs and long play seems to cause memory leaks after
walking around or back and forth for a while causing frame drops", and "We
have gpu vertex shaders to animate simple animations ... The reef is easiest
fix, do that first".

**Finding (read in the code).** `places::bend_all` ran every frame for every
reef tile within 30 m: it spawned up to eight OS threads
(`std::thread::scope`), worked out every swaying vertex's sine on the CPU
(`fill_bent`), and inserted a new position buffer into each swaying mesh and
kelp mesh, so Bevy re-extracted and re-uploaded every one of them each frame.
The more reef in reach, the more of the frame it took; walking along a reef
keeps it in reach.

**Change.** The sway is the same sum, moved to the vertex stage:
- each swaying vertex carries `ATTRIBUTE_SWAY` (xyz its axis times its
  reach, `min(0.07 h, 0.7)`; w its phase with the height's share,
  `phase + 0.6 h`), written once when its tile is built;
- `sway_vertex.wgsl` moves it by `sin(clock * 1.1 + w)` times the bend,
  which eases from 1 to 0 over the last `SWAY_EASE_M` before `NEAR_M` from
  the eye, measured from the tile's anchor (the mesh origin), as the CPU
  did;
- the clock is the sun clock's seconds wrapped at an hour, as before, sent
  in the field's view buffer (`FieldView::sway`, `Scene.sway`);
- `FieldLit::specialize` gives the stage to a mesh carrying the attribute,
  as it gives `swim_vertex.wgsl` to a school; `bend_all`, `fill_bent` and
  the tiles' kept rest poses are gone.

The look is unchanged except where the eye stands well above or below the
reef: the CPU measured the tile's distance along the surface, the shader
measures it straight from the camera, so the sway's last 4 m of easing may
sit a little nearer when the eye is far above the sea. Normals were the rest
pose's before and are now.

**Not yet.** The ambient life and vent smoke are still written per tile per
frame (`write_mesh`), the reef's meshes still carry `NoFrustumCulling`, and
there is no instancing; they are next, measured the same way.

**Proof.** Tests: `a_swaying_vertex_carries_its_reach_and_phase` (the GPU's
sum against the old CPU sum) and `the_shader_keeps_the_sway_distances`.
Before and after: the reef soak (`assets/showcase/reef-soak.ron`, a reef's
coast and 150 m inland, ten times), `--frame-log` and `SEA_PLACES tiles_ms`,
the same world and build profile.

**Half as dense (owner, 2026-11-25: "Lower its density by 2x").** The
reef's lattice step goes from 0.7 m to 0.99 m (times the square root of
two), so a tile holds half the pieces and half the triangles. The pieces
are drawn only, from the seed, never saved, so no world changes.

**Measured (2026-11-25, the desktop, RTX 3070, fast build).** At the reef
of `ocean-life` (13.096, 94.896), 11:00, standing 150 s, frames after the
first 40 s, the build before against the GPU sway with half the density,
interleaved twice (each baseline run with its own `field_lit.wgsl`):

| | Before | After |
| --- | ---: | ---: |
| Frame p50 | 97.6, 98.1 ms | 51.2, 52.5 ms |
| Frame p99 | 163.5, 169.8 ms | 71.4, 74.6 ms |
| Main thread p50 | 58.6, 59.0 ms | 46.4, 47.7 ms |
| GPU p50 | 11.1, 11.0 ms | 10.3, 10.3 ms |
| Memory (working set) | 1885, 1854 MB | 1364, 1369 MB |
| Reef triangles | 2.73 M | 1.38 M |

**What is left: the ambient life.** `SEA_PLACES ambient_ms` (a measurement
instrument, the share of `tiles` spent in `ambient`) is 34-38 ms of the
47 ms main thread: every tile in reach lays its whole anthias cloud (40
swimming bodies), its clownfish and its vents' 36 puffs on the CPU each
frame and uploads them. Each is a function of time alone, so it moves to the
vertex stage as the sway did: laid once, moved on the GPU (task 5.3).

## The reef's budget and its fish on the GPU (owner, 2026-11-25)

The owner, on "is reef up to 40-60 fps? it seems still high at 19fps":
"each reef budget: 500 triangles, number of reefs 1000 or less", "optimize
fish and smoke stuff", "fish should be batched into the gpu and drawn like
the swarm dudes in the swarm demo, so their impact is near zero due to the
boid sim being on gpu", "and animation".

**Where it stands.** With the ambient life on every core, the reef's frame
is 40.5 ms p50 (25 fps), 21 ms of it the fish and plumes laid on the CPU.
Pieces' triangles (`print_piece_triangles`, a measurement): every kind is
7 to 244 but the hero anemone, 1800. The 1.38 M triangles in reach are
about 10 000 pieces over 66 tiles: the count, not the pieces, is the cost.

**The budget.**
- A piece is at most 500 triangles: the hero anemone's tentacles are cut
  to fit, and a test over `library()` holds every kind to it.
- At most 1000 pieces are drawn. Tiles are built nearest first; each keeps
  its pieces in order of size, and the tiles' allowance is shared out by
  distance so the near tiles keep every piece and the far ones only their
  largest, never more than 1000 in all. A test holds a full reach to it.

**The fish on the GPU (swarm-demo's pattern, `.reference/swarm-demo`,
`swarm_app/src/swarm.rs` and `assets/shaders/swarm.wgsl`).**
- One storage buffer of reef fish, written once when a tile is built
  (where, its cloud's centre and up, its kind, size and seed), freed with
  the tile; nothing about a fish comes back to the CPU.
- A compute pass before the cameras draw moves each fish: its cloud's orbit
  as today's anthias and clownfish swim it, kept as the look, with the
  boids' separation and alignment on top within a cloud.
- The buffer is the instance buffer of one instanced draw of one body mesh
  per kind; the vertex stage places it and bends it with `swim.wgsl`'s body
  wave, and the field-lit fragment shades it, so it is lit as the reef is.
- `ambient` keeps the seals; the plumes follow the same pattern as GPU
  particles.
- Proof: the same reef before and after, frame time and `ambient_ms`; a
  capture of the fish clouds beside today's.

## The long play: towns at 60 fps (owner, 2026-11-25)

The owner: "Played the game, run around for about 15-30 minutes. This might
have something to do with towns and NPCs", and "We should aim for 60 fps or
higher at all times".

Measured (desktop, `fast` build, `town-route`: Holbrook to Linenleigh and
back, 518 m, 30 minutes, a fresh world): no leak. Private memory rose 2.14
to 2.32 GB and fell back to 2.23; Bevy's meshes, images and materials held
flat. Every frame missed 16.7 ms: p50 27.5, p99 76, 262 hitches over 100 ms.
The main thread is the bound (28.3 ms), the GPU is not (9.2 ms).

Where the main thread goes, from the game's own `SPENT` lines (over 2 ms):
the field-lit meshes' corners (`light_from_the_field`) 7.4 ms on 49,924 of
55,827 frames, about 6.6 ms a frame; the town bake 3.3 ms on one frame in
ten; the audio mix about 2 ms. A Chrome trace (`--features trace`) ranks
physics next (9 ms traced, `resolve_ground` 6.2 of it).

**Cut 1, the corners on every core.** 150 to 270 meshes that move each
frame (the townsfolk's parts) read the field at eight corners each, one
after another. The reads are independent and read-only, so they are made on
the compute pool, and the rows written after, as the reef's ambient life is
(5.3a). The same samples, the same rows: the picture does not change.

Tried and reverted (2026-11-25): the corners read on the compute pool made
the pass slower (7.4 to 14.2 ms when over 2 ms; frame p50 31.6 to 33.7 ms
over the same 5 minutes of the route). Measured after, with the read's cause
logged: in a town about 180 to 215 meshes re-read their corners every frame
because they **moved** (none because the field changed), 9 to 14 ms. That
is 1,400 to 1,700 field reads at 6 to 8 µs each; the read's cost is the
finest-cell lookup (`PlanetContact::finest_cell`). The next cut is that
lookup, or fewer moving rows (a townsperson's parts lit as one).
