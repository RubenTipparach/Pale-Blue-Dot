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

## Decision: a stretch lies on the ground when the ground reaches it

A pier stretch's deck height is the higher of:

- the template's deck, `terrace + from[1]`, and
- the highest ground under the stretch (across its width, along its length
  and its laps) plus `DECK_M`, where that ground reaches the template's deck.

So a stretch over the beach lies on it, its planks' underside on the sand and
their top 0.12 m over it, as boards laid on a beach do; every stretch over the
water is where it was. The landward end now reads as a boardwalk on the sand
running out to the planks on piles.

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

- **A 0.12 m step** where a lifted stretch meets one that is not, and from the
  quay onto the planks. The walker climbs up to 0.35 m without a jump
  (`walking::HOLD_M`), and town and deck risers are 0.19 to 0.25 m.
- **Marenstrand's bank.** One stretch of its main pier crosses ground a metre
  over the deck (the planet's ground in the margin, where the template has
  beach a layer lower). It lies on the bank, 1.12 m over the stretches either
  side: a walker climbs onto it as onto a terrain layer. It is the one place
  in the six harbours; it is a stretch on the sand instead of planks buried
  in it.

The stretch's height is one function, `harbour::stretch_top`, used both to cut
the pier and by the dressing's stand query (`dressing::stand`), so a crate
on a lifted stretch stands on its planks.

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

## The speckle on the water round the piles

See "Not this change" below (filled in from the captures).
