# Design: pier-landfall

## The finding

### What a pier is

`pieces::harbour::pier` cuts a pier (`cities-in-the-world` 4d) from the
`coast` template: planks `width_m` wide from `from` to `to`, at the template's
height `from[1]` (1 m) over the harbour's terrace (the sea's surface, floored:
0 m on the shipped seed), in stretches of about 2.2 m, each flat in its own
frame. The plank prism runs from `-DECK_M` (0.12 m) to 0 in the stretch's
frame: its top is the deck height, its underside 0.12 m under it.

The ground under it is the column's: the town's terrace where the harbour
lays a cell, the margin's eased ground round that, and the planet's own
ground under the cells over the water (`over_sea`). Every height is a whole
layer (`column::surface_m` floors), and the top of a cell's ground is its
layer's height.

### Why the planks are in the sand

The mockup's beach slopes under the piers: rows 12, 13 and 14 stand at 0.25,
0.5 and 0.75 m, the quay at 1.0 m, and the pier's planks over the quay's
flags are lifted a centimetre (`ZLIFT`, "its first planks lie over the
quay's flags"). The export rounds every height to a whole layer, so rows 13
and 14 become 1 m: the deck's own height. In the game the planks over those
two beach rows and the quay's flag row lie wholly inside the top layer of the
ground, their top face in the sand's plane. The sand is drawn over the planks
(the two faces fight for the pixel, and the wet sand wins most of them), the
piles stand in the sand, and the deck seems to run down into the beach.

### Measured

`towns::tests::print_the_ground_under_the_harbour_piers` (an instrument,
`--ignored`) cuts each pier into the same stretches `harbour::pier` does and
reads the highest ground under each, across the deck's width every 0.25 m,
from the harbour's stored footprint and the planet's ground, against the
deck. On the shipped seed (six harbours, eleven piers each):

| Where | Stretches with the ground at the deck (+0) | Over it |
| --- | --- | --- |
| Every harbour, main pier (0) | the first 3 of 15 | |
| Every harbour, west and east piers (4, 7) | the first 3 | |
| Every harbour, hut pier (10) | the first 2 | |
| Marenstrand, main pier | 2 more, mid-pier (the margin's eased ground) | 1 stretch, +1 m |
| Marenstrand, a finger pier (1) | all 4 | |
| Wickingstrand, west pier | 2 more, mid-pier | |
| Selingquay, main pier; a finger pier (8) | 2 more; 2 of 3 | |

Every other stretch has its ground at least a metre under the deck. So the
owner's shot is not one harbour's bad luck: every pier that reaches the quay
does this at its landward end, in every harbour.

## Decision: planks lie on the ground where the ground reaches them

The pier is cut where the ground under it changes, not in fixed lengths
(`harbour::stretches`):

- The ground is read every 0.25 m along the pier, across its width (five
  points, edge to edge), from the column's own heights.
- Each 0.25 m interval takes a top: the template's deck, `terrace + from[1]`,
  where the highest ground at its two ends is under the planks' underside;
  otherwise that ground plus `DECK_M` (0.12 m), so the planks lie on it.
- Intervals of one top join into a run, and a run is cut into flat pieces of
  about 2.2 m, as the whole pier was before. A pier whose ground never
  reaches its planks is one run, cut exactly as it always was.
- A piece on piles has a pile pair at its start, and at its end where the
  next piece is at another height or there is none. A piece on the ground
  gets none (a pile stops where the ground is, `pile` skips it).

So the beach rows at 1 m carry the boardwalk on the sand, 0.12 m proud, as
boards laid on a beach are, and the planks step down onto their piles where
the beach drops to 0 m. Every piece over the water is where it was.

A first cut of this kept the fixed 2.2 m stretches and lifted a whole stretch
when any of its ground reached the deck. At Marenstrand that raised the
whole 2.2 m stretch from 13.1 to 15.3 m along the main pier to 2.12 m,
most of it over ground at 1 m. Cutting at the ground's own changes keeps
the raised piece to the 0.75 m (12.42 to 13.17 m) the bank is under, with
the boardwalk on the 1 m sand either side of it (the instrument prints each
pier's cut).

Alternatives weighed:

- **Lower the beach under the pier.** The ground is the generator's and the
  stored town's: changing it changes a world's terrain under a player's work,
  and would be a new layout version for new worlds only. Old saves would keep
  the clipping. Rejected.
- **End the deck where the ground reaches it.** No step anywhere, but the
  mockup's boardwalk runs from the quay across the beach, and ending it
  leaves the piers starting from a sand bank. Rejected; it was the next best.
- **Lift the whole pier.** Clears the sand, but moves every deck off the
  porch stairs and the gangplank, which land at 1 m. Rejected.
- **A one-centimetre lift, as the mockup's.** The mockup's quay is at
  exactly 1 m; the game's beach is two whole rows at 1 m, and a centimetre
  over a hex top is a plank painted on the sand, still fighting it from any
  distance. Rejected.

What it costs:

- **A 0.12 m step** where the boardwalk on the sand meets the planks on
  piles, and from the quay onto the boardwalk. The walker climbs up to
  0.35 m without a jump (`walking::HOLD_M`); town and deck risers are 0.19
  to 0.25 m.
- **Marenstrand's bank.** One place on its main pier crosses ground a metre
  over the deck (the planet's ground in the margin, ring 2, where the
  template has beach a layer lower). The planks over it lie on the bank,
  a metre over the boardwalk either side, for the 0.75 m the bank is under
  the deck. From the pier it reads as a raised box of planks with a post at
  each corner: no plank is in the sand any more, but it is not pretty. A
  walker climbs it as a terrain layer. It is the one such place in the six
  harbours, and the open question this change leaves (the bank shot):
  planks on the sand bar as now, a gap in the planks where the bank is, or
  a short stair. Recommendation: leave it until the owner has seen the shot.

The cut is one function, `harbour::stretches`, used both to cut the pier and
by the dressing's stand query (`dressing::stand`, through `harbour::top_at`),
so a crate on the boardwalk stands on its planks.

## Saves

- Nothing stored changes. A pier is the template's piece, cut from the town's
  stored chart as the masonry is (`cities-in-the-world` 4d, "What is
  stored"); it is never in a record. The town record, its footprint, its
  `over_sea` cells, its terrace and the layout version are as they were.
- The ground is untouched: the height every column has, and so every player
  edit, is the same number before and after.
- So this is a drawing-and-collision change to a derived piece, not a
  generator change, and needs no new version. A world saved before it opens
  with the same harbour, its planks now on the sand.

## Not this change: the speckle on the water round the piles

The owner's shot also shows the sea between the piers covered in bright
green-cyan per-pixel speckle, noisier than the open sea. It is a different
cause, in the sea's shader (`water.wgsl`, the sheet's fragment), and is not
fixed here. Measured on Holinghaven's shallows at night with boats and piles
(`docs/screenshots/pier-landfall/`, the speckle crops, same spot and frame):

- **Not the sea's screen reflection.** `PBD_REFLECT=sea` (the sea's trace
  off) leaves the speckle as it was.
- **Mostly the refraction.** `refract_amount: 0.0` in `water.ron` (a
  temporary edit, not committed) takes out most of the dark and bright
  per-pixel noise. The refraction offsets each pixel's look through the
  water by the ripple normal, which the rain rings make change from pixel to
  pixel. The sampled point then lands on the seabed one pixel and on a hull,
  a pile or a hex column's side the next, and both the colour seen and the
  depth the shallows' colour is chosen from (`depth_under`, `path_length`)
  jump with it. The green-cyan is the shallows' own albedo (`shallow_color`)
  where the water read as shallow. The open sea has nothing near the
  surface for its samples to jump between, so it stays smooth.
- What is left with refraction off is a finer, fainter sparkle in the same
  place, not examined further.

A fix belongs in its own change: filter the refraction offset by the
ripple's pixel footprint as the specular already is (`detail`), or reject a
refracted sample whose depth jumps from its neighbours'. Either changes the
sea's look everywhere and wants its own before and after.
