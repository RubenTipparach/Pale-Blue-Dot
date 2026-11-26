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

**Cuts made (2026-11-25), each measured on the same 4.5 minutes of
`town-route` (frame p50 / p99 ms, main thread ms):**

| Build | p50 | p99 | main | over 16.7 ms |
|---|---|---|---|---|
| Before | 31.6 | 196.5 | 33.1 | 100% |
| Corners' ground asked once per 25 cm (`GROUND_REASK_M`) | 25.8 | 42.2 | 24.1 | 100% |
| Town bake's ground asked once per vertex (`BakedField::ground`) | 22.3 | 37.3 | 21.2 | 96% |
| The ground check asks the pieces near the body (`Meets::near`) | 17.7 | 24.4 | 16.4 | 78% |

The first two: a moving townsperson's corners, and a town's outside on each
set that lands, asked the generator (`column::surface_m`, noise and the
towns' ground) for the ground every time, about 1,600 times a frame. The
third: every probe of every sweep of the walker walked every loaded town's,
build's and road's pieces. A check run beside it (3 minutes, every `stand`
and `holds` asked both ways) found no answer the near pieces gave unlike all
of them: walking is unchanged. Picture unchanged in all three.

The bot's walk to the other town stalls near Holbrook with the third cut in
(two runs), where at 22 ms it arrived: its planner, not the walker (the
check above), at the faster frame. The last row is the bot standing near the
town more than walking; the route's legs are measured again once the bot's
stall is fixed.

`push_normal` asks every piece again (the near pieces alone stalled the
bot's walk out of Holbrook in two runs; with every piece, and with
`PBD_NEAR_OFF`, it walked). With it, on a quiet machine: p50 18.3, p90 21.8,
p99 28.3 ms, main 17.2 ms, 82% over 16.7 ms, one hitch over 100 ms (105
before). A run started seconds after an 8-minute build showed 400 ms to 12 s
stalls in every system at once: the machine, not the game; not counted.

**Next: the UI.** Traced, the UI's layout, clipping, stack and focus are
10 ms a frame, the most of anything. `PBD_MEMORY_TRACE` now logs the UI's
nodes by root (`UI_NODES`): 8,452, of which about 6,500 are in screens that
are shut (`Display::None`): the settings menu 1,816, the mission tree 1,702,
the map 688, four more of 425 to 950. Bevy stacks, clips and hit-tests every
node each frame, shown or not. Cut 4: a big screen is built when it opens
and despawned when it shuts, its first build prewarmed on the loading screen
so opening it never hitches. The picture is the same.

**Cut 4 as built: the UI laid out every frame.** `UI_CHANGES` (the nodes
whose `Node`, `Text` or `Visibility` changed, each frame) found the F3
graph's 240 bars resized every frame (about 200 nodes a frame) and the
compass bar's 48 marks written every frame, changed or not. Either marks the
UI changed, and Bevy lays it all out again. The graph's bars are drawn into
one image (`FrameGraphBars`), the compass bar writes a mark only when it
moves (`put_display`, `put_left`): 250 changed nodes a frame to 8. Measured
on the route (the bot stalled in the second run, so it stood more):

| Build | p50 | p90 | p99 | main | over 16.7 ms |
|---|---|---|---|---|---|
| The three cuts | 18.3 | 21.8 | 28.3 | 17.2 | 82% |
| + the graph as an image | 18.0 | 21.5 | 28.3 | 16.8 | 71% |
| + the compass bar's guarded writes | 16.5 | 19.3 | 23.4 | 15.4 | 45% |

The bot's stall near Holbrook comes and goes (a different cell each run),
so further A/Bs use a fixed scene that does not walk.

**Cut 5: shut screens are parked out of the UI (owner, 2026-11-25: "100%
frames above 40, and about 80% 60 or above").** Measured on the route with
cut 4: p50 15.9, p80 18.5, p99 28.2 ms; 64% of frames at 60 fps or better,
97.4% at 40 or better. The main thread sets the pace (13.5 ms on a fast
frame, 16.7 on a slow one). Traced, Bevy's UI is still the biggest cost:
layout 4.0 ms, clipping 0.8, stack 0.8, focus 0.5 a frame, because each
walks every node from every root each frame, and a shut screen's root is a
root like any other. The cut: a root that has been shut (`Display::None`)
for 30 frames, with 64 nodes or more under it, is made the child of one
hidden entity that is not a UI node (`ui_park`). It is then no root, and
no UI pass reaches it; the game's own systems still find and fill it. When
anything sets its display back, it is unparked before that frame's layout,
so a screen opens on the frame it is asked to, as today. Nothing drawn
changes: a shut screen draws nothing either way.

Measured on the route (150 s, `--no-audio`, so no audio mix in either):

| Build | p50 | p80 | p99 | max | at 60 fps | at 40 fps | main p50 |
|---|---|---|---|---|---|---|---|
| Cut 4 | 15.9 | 18.5 | 28.2 | 201 | 64% | 97.4% | 14.4 |
| + shut screens parked | 11.1 | 13.5 | 27.7 | 56 | 89% | 98.2% | 10.0 |
| Cut 4, after the first 65 s | 15.4 | 17.0 | 23.6 | 201 | 76% | 99.4% | 14.0 |
| Parked, after the first 65 s | 10.8 | 11.9 | 15.6 | 38 | 99% | 99.9% | 9.9 |

The UI's roots hold 1,056 nodes, not 8,452. The `ui-check` showcase opens
the pause menu, the missions, the pack and the map after they were parked,
and each opens whole, the same as with `PBD_UI_PARK_OFF` (the bot's `Tap`
now knows O, M, Tab and Escape). What is left under 40 fps is the first
minute: while the near terrain's fine tier builds in (versions 1 to 8,
25 to 60 s), the GPU's terrain pass is 12 to 16 ms and its shadow 8 to 12.

**Cut 6: the terrain shaded once a pixel (owner, 2026-11-26: "Get me as
close to 100% ... 60 fps ... over the course of 30 minutes").** Measured on
`turn-check` (standing in Holbrook, a full turn in twelve steps, 150 s,
cut 5 in): p50 11.8, p90 16.5, p99 31.5 ms; 91% of frames at 60 fps or
better, 98.6% at 40. The frames over 16.7 ms are mostly the GPU's: its
terrain pass is 4 ms facing the town at the start and 7.7 ms facing the
same way at the end, with the same triangles (3.3 million vertices, 1.1
million triangles) and 40% more fragments. With the wet film compiled out
(`PBD_TERRAIN_OFF=wet`) it stays near 4.5 ms: the ground soaking in a
light rain (`weather.x` over 0.001) turns on the wet film's work on every
terrain fragment. That is the picture (the ground does go dark and glossy),
so the work stays; what it is spent on is the cut.

The terrain pass shades about 5 million fragments for a 1440 x 900 window
(1.3 million pixels): each pixel about four times. The ground, the trees,
the near trees, the clutter and the columns draw in that order in one pass,
in the compute pass's order within each, and the shader can discard (the
plants' cut-outs, the dither of a fade), so a fragment is shaded before
anything in front of it has written its depth. The cut: the same draws once
first with a depth-only fragment (`shadow_fragment`, the same cut-out and
dither test the shadow pass uses), writing depth and no colour; then the
colour pass with depth writes off, so the GPU's early depth test leaves
every hidden fragment unshaded. The vertex shader's position is
`@invariant`, so both passes put a pixel at the same depth. Nothing drawn
changes: the same nearest surface wins each pixel. The vertex work is done
twice; the flat price (`PBD_TERRAIN_OFF=flat`) says what that costs.

Measured on `turn-check`, the same sitting, after the first 30 s
(`PBD_TERRAIN_DEPTH_OFF` for the one-pass row):

| Build | p50 | p80 | p90 | p99 | at 60 fps | at 40 fps | terrain pass, start / soaked |
|---|---|---|---|---|---|---|---|
| One pass | 12.1 | 15.1 | 16.5 | 20.6 | 91% | 99.9% | 4.0 / 7.7 ms |
| Depth pass first | 10.6 | 12.6 | 13.9 | 18.9 | 98% | 99.5% | 2.6 / 4.9 ms |

The terrain pass's time includes both its draws. Stills from where the bot
stands, both ways round, differ only where the grass sways, a townsperson
walks and the clouds move (`still-check`). The one frame of 288 ms in the
second run is the main thread: traced, the spikes left standing in town are
`fish::places::tiles` (27 to 45 ms when it runs long), the next cut.

**The half hour, after cut 6.** `town-soak` from Holbrook, 31 minutes,
`--no-audio` (the bot did not finish its first gathering trip: it kept
restarting its walk away from town, so it played round the town's edge, not
the whole loop). Over the run: p50 12.1, p80 15.8, p99 22.1 ms; 88% of
frames at 60 fps or better, 99.6% at 40; memory and entities flat
(12,472 entities, 40.6 MB of mesh at the start and the end). By stretch:

| Minutes | GPU | terrain | shadow | main | at 60 fps |
|---|---|---|---|---|---|
| 0.5 to 2 | 9.7 | 3.4 | 5.1 | 8.5 | 98% |
| 2 to 10 | 11.3 to 12.4 | 5.1 to 5.9 | 4.7 to 5.0 | 10.1 to 14.0 | 68 to 94% |
| 10 to 20 | 14.0 to 15.1 | 4.7 to 6.4 | 6.3 to 9.3 | 5.9 to 7.0 | 60 to 78% |
| 20 to 24 | 10.9 to 12.4 | 4.2 to 4.6 | 6.1 to 7.1 | 5.7 to 5.9 | 90 to 96% |
| 24 to 31 | 7.7 to 8.2 | 4.1 | 3.0 to 3.4 | 5.9 | 100% |

Two things are left. In minutes 2 to 10 the main thread climbs to 14 ms
while the bot walks round the town's edge (traced next). In minutes 10 to
20 the sun is low (the day runs about two minutes an hour: 10:00 at the
start, the dusk lamps at 20:24) and the shadow pass is 6 to 9 ms.

Priced at 18:30 in Holbrook (`shadow-check`, `PBD_SHADOW_SKIP`): the pass
is 5.9 ms and its four cascades shade 85 million fragments a frame; with
the trees' crowns left out, 2.5 ms and 21 million, and with the ground
left out, 4.8 ms. A low sun looks through a forest sideways: each texel's
ray crosses crown after crown, and every leaf card's fragment runs the
cut-out test, since a card with a discard cannot be rejected before its
shader by depth it has not written yet.

**Cut 7: a town's bake no longer re-uploads the town.** Traced over the
soak's first 3.5 minutes (the trace build; every number inflated), the
main thread's climb in minutes 2 to 10 is `field_light::bake_town_outsides`
(0 in town, 14.7 ms a frame at the town's edge), the mesh allocator (0.5 to
8.6) and the mesh extract (2.1). Each fine set that lands, and each dig,
changes the field's light version, and every town in reach is baked again,
a slice a frame. The bake took each town's mesh mutably on every frame of
it, even a frame it only read or had no budget left: a mutably taken mesh
is a changed one, so Bevy uploaded the whole town again every frame a bake
was in progress. The cut: the mesh is read with `get` and taken mutably
only to swap the finished bake in. Nothing drawn changes.

`town-soak` minutes 2 to 4, against the half hour above: the main thread
10 ms (was 13.5 to 14), frames at 60 fps 87 to 94% (were 68 to 79%). The
run had main-thread stalls of 0.2 to 1.5 s in its first 40 s of play, when
the bot plans its first walk (`showcase: planned 7 steps`); the bot's
planner is the measuring instrument's cost, not the game's, and is taken
off the main thread next so the numbers are the game's.

The held light's flame, glow and embers were written (empty) every frame
with no torch in hand, three meshes uploaded each frame for nothing; with
no flame and no ember alight they are now left as they are. Measured again
on `town-soak`'s first 4.5 minutes with both: p50 11.5, p99 18.4, max 30 ms;
97% of frames at 60 fps, 99.9% at 40, main 8.6 ms. The stalls of the run
before it did not come back: it started seconds after a 17 GB file was
deleted, and is not counted.

**Cut 9: a town keeps its bake while its light is unchanged (owner,
2026-11-26: "Could you cache town lighting so we don't end up with
rebuilding?").** A town's outside carries the field's light baked into its
vertices (36,000 for Holbrook), tagged with the fine set it was read from.
Walking, a new fine set lands every few seconds, and each one counts as new
light: every town in reach is baked again, all of its vertices, a 2 ms
slice a frame, though nothing about its light has moved. The cut: each bake
also keeps where its vertices read the field (each finest cell a vertex
samples, with the two layers it reads between) and a digest of what the
field holds there: the light of those cells and their neighbours at those
layers, whether each is solid, and whether each neighbour is in the
region, which is everything `light::sample` reads. A new set first takes
the digest (a few thousand reads, not 36,000 samples); when it matches, the
town keeps its bake and is tagged with the new set. A dig, a placed lamp,
dusk and dawn change those values and bake it again, as now. The picture
cannot change: a matching digest is the same inputs to the same sample.

Measured on `town-soak`'s first 3 to 4.5 minutes: the town baked again at
two moments in a 3-minute run (every probe on the tier both times: the
bot digs beside the town, and the light there does change), and kept its
bake on the other sets that landed; 96% of frames at 60 fps, 99.9% at 40,
max 42 ms. A digest of Holbrook's 1,161 probes is 4 to 5 ms for eleven
towns' pieces at once, against 2 ms a frame for some 20 frames to bake
them.

**The shadow pass draws nearest the sun first** (cut 8, part). Priced at
18:30 in Holbrook with `PBD_SHADOW_PRICE`: the casters' cut-out test is
nearly all of the pass (6.3 ms with it, 1.45 ms with an empty fragment,
whose casters the GPU rejects by depth before shading). Drawn towns, crowns,
then ground, the crowns' discards no longer hide the ground under them
from the early depth test: 7% fewer fragments shaded, 6.07 to 5.90 ms
(`PBD_SHADOW_ORDER_OLD` for the A/B). The maps are the same: the nearest
depth wins each texel in any order. The rest of it is the crowns' own
overdraw under a low sun; the leaf cut-out's share is priced next
(`PBD_SHADOW_PRICE=no_leaf,no_part`).
