# Design: distance-lod-fade

## 1. The rings

Each frame, from the live camera: level `k`'s outer radius
`OUT_k = B_k(h)` (`live_bands_m` at the current height, continuous) and inner
radius `IN_k = OUT_k - W_k`, `W_k = lod_fade_width * OUT_k` (`scatter.ron`,
validated 0..=0.5; 0 draws today's hard edges). A point's fade toward the
finer level at `k`:

```text
t_k(p) = clamp((dot(p, c) - cos(OUT_k)) / (cos(IN_k) - cos(OUT_k)), 0, 1)
```

1 inside `IN_k`, 0 outside `OUT_k`, round the centre `c`.

## 2. One partition per pixel

With the pixel's Bayer value `m` (the 4x4 screen-fixed mask, no temporal
change: the terrain has no history to absorb a moving pattern), the pixel's
partition puts level `k`'s edge at the radius where `t_k = m`. A fragment of a
cell at level `L` is kept when that partition draws the cell:

```text
kept = (L == base || t_L(owner) > m) && !(t_{L+1}(centre) > m)
```

`owner` is the cell's owner at the level below (the split midpoint cell
already picks its nearer owner per fragment). Outside the rings the `t` are 0
or 1 and this is today's `drawn`. Every Bayer class sees one complete
partition: every ground point is covered by exactly one kept cell, so there is
no double draw and nothing to z-fight.

**Visibility**: a cell is listed once when some `m` keeps it: its owner
inside `OUT_L` and its centre outside `IN_{L+1}`. No NEW/OLD marks.

**Vertex**: `t` at the cell's owners and centre ride to the fragment as one
flat vec3, with the split plane.

**Walls**: a cell's wall down to a finer neighbour's floor is built wherever
the neighbour is inside `OUT_{L+1}`: in every Bayer class the seam lies
somewhere in the ring, and a wall reaching the fine floor there is a skirt
under the neighbour's cap. The cut wall of a split midpoint cell is built
when its owners' `t` differ, kept when they fall on different sides of `m`.

**Records**: the coarse level's inner margin grows by `W` (`lay_band`), and
the floors are baked out to `OUT + regen + guard` (`floor_rule`): both levels
are resident across every ring for any centre within the fit.

## 3. The centre

`c` follows the camera while its distance from the fine set's anchor is
within what the records hold (the regeneration margin plus three tiles).
Beyond that it is held on the arc toward the camera, and after a landing it
catches up at a bounded speed (at least twice the ground speed). `c` at the
anchor always fits (it is today's partition), so the fit can never draw a
hole. The landing's timed cross-fade stays only as the fallback when a sharp
turn forces `c` to jump, logged as `LOD_FIT jump`.

## 4. Trees

- **Nesting.** A coarse cell's centre is a cell centre at every finer level
  (`planet_lattice`). A coarse cell stands a tree only when the level-11 cell
  at its centre has one: the CPU writes that cell's stable id into the
  record, and `has_nearby_foliage` rolls it at the plain density. Same point,
  same height and biome, same id: the coarse tree is the fine tree.
- **Across a ring** the survivor's width blends from fine to coarse by its
  group's `t` (the same in both copies, so it never visibly changes), and its
  three siblings dither out by their `t`: a quarter of the trees at four
  times the area, cover kept.
- **Range.** Trees stand on levels 8-11; the foliage range becomes level 8's
  band, and they fade to nothing across level 8's outer ring, as Unity's last
  LOD does. `tree_shown` / `tree_fade_m` retire.

## Order

1. The rings, the centre and the fit on the CPU, the records and floors, with
   instruments: the centre's distance to the anchor against the fit margin on
   the walk, far-side and cloud-hop runs.
2. The per-pixel partition in the visibility, vertex and fragment passes.
3. Recordings; retire `lod_fade_s` for terrain.
4. Tree nesting, then level-8 trees and the longer range.

## Verification

- CPU: every ring resident for a centre inside the fit; the fit continuous
  across a sequence of landings; for all 16 `m`, every sampled ground point
  covered by exactly one kept cell (a Rust copy of the kept test on real
  records); a coarse record names its centre cell's tree.
- GPU: the visibility pass lists each ring cell once; coarse trees stand at
  their centre cells.
- `PBD_NO_SKY` stills facing the 300 m and 600 m rings with the width at 0.5.
- Recordings: a slow flight low past hills in the rings, the scenic and
  cloud-hop flights, and a walk toward a forest 250-400 m out. Trees and
  blocks are seen to dissolve with distance; nothing appears or vanishes in
  one frame.
- The perf suite, old against new (the floors per rebuild are the main cost:
  roughly twice the sides baked).

## What building it found

- **The width.** At 0.15 the finest ring is 45 m deep at 300 m out, too
  narrow to read as a fade in flight; the default is 0.3 (rings of 90, 180,
  360 and 720 m on the ground), about Unity's 0.5 on this band ladder.
- **No holes.** `PBD_NO_SKY` at the 300 m seam, widths 0 and 0.5: magenta
  only in the sky and, at 0.5, along the far ridge where the stippled trees
  and the two levels' tops meet the sky, which is the cross-fade itself.
- **The water sheet** takes the same centre and rings and splits per pixel
  the same way; left on the anchor, it would have holed where the terrain's
  ring ran ahead of the anchor's band.
- **The dissolve's own mask.** A landing's dissolve now uses interleaved
  gradient noise, so it does not run in step with the Bayer mask the
  distance cross-fade divides the pixels by.
- **Capture frames.** The terrain's pipelines, grown by the per-pixel tests,
  compile later than before: a still at frame 120 caught the planet before
  its terrain pipeline was ready (only the sky's planet disc). Stills of the
  terrain are taken at frame 600.
- **Trees keep their place.** A finest cell stands its own tree, so near
  trees are exactly as before; a coarse cell stands the tree of the finest
  cell at its centre, so the forest thins by keeping one tree in four, each
  wider, rather than rolling new trees in new places at every ring.
- **A runaway fit (found on the owner's cloud-hop recording, 2026-09-26:
  "nearest LOD doesnt seem to be loading in when landed").** A set's records
  are widened to hold the partition they replace, round its centre, which
  stood up to that set's fit from its anchor; and the fit was taken from the
  rings as laid, widening included. So each landing's fit carried the last
  one's, the rings grew landing by landing until capacity cut them (level 9's
  records reached 1,511 m where they end near 1,270 m), and every build laid
  and baked several times the cells: 4.2-4.9 s a set against 1.0 s before,
  so in flight the camera ran 600 m past the anchor, the centre could not
  follow, and at touchdown the finest level had not landed when the route
  ended. **Fix:** a level's fit is capped at its own margin (the rebuild
  distance and three of its tiles); the widening holds the old partition and
  gives the new centre nothing. A test runs a chain of landings and holds the
  fit and the rings steady.
- **The landing's dissolve is 1 s** (`lod_fade_s`, the owner: "the cross fade
  should be 1s long for terrain"), from 0.6 s: when a set lands with resized
  bands or a clamped centre, the blocks dissolve over a second.
- **A second loop: the cover.** With the fit capped, flights still built in
  2.4-4.5 s (median 2.7). Measured on the ground with no set replaced, one
  build is 0.61 s at a width of 0, 0.75 at 0.15 and 0.89 at 0.3 (four
  threads, release): the rings cost a little. In flight the rest was the
  cover (`detail-fade` section 3), which widens a set's records by however far
  the anchor moved: a slower build moved it further, which widened the next
  set, which built slower still (the anchor 300-600 m behind at landing).
  With the centre following the camera the old and new partitions are the
  same at nearly every landing, so the cover is capped at a level's own
  margin; past it, section 4's fallback draws the new partition whole.

