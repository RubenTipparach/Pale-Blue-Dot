# Design

## 1. Tools that stop digging

### What was looked at

Everything in `dig_and_place` that persists across frames or can return
before the ground is dug:

| Suspect | Finding |
| --- | --- |
| `Breaking` (`pbd_core::dig`) | Clears its target whenever the button is up or the aim is gone; the pause after a break only runs while the button stays down and is zeroed on release. Nothing sticks. |
| `Mining::lookup` (cached break time or refusal) | Keyed by block, digger (tool and tier) and fine-set version, which every edit, landing and dusk relight bumps. A cached refusal holds only for that block with that tool in that set. |
| A tool with no tier | `Equipment::from_tiers` and `hold` refuse a held tool that is not owned, so `digging()` always has a tier for a held digging tool; load goes through `from_tiers`. |
| The G picker, the pack's hand list | Both go through `Equipment::hold`; nothing else writes the hand. |
| Menus taking the pointer | `hand_over_pointer` hands the pointer back on the frame the menu shuts; while it is away the dig steps with the button up, so nothing is held over. |
| The rod and fishing | The rod is not a digger (by design). The line is reeled in the frame the rod leaves the hand. |
| `held::Swinging`, `MealEffects::keen` | Read only. `keen` is 1 or the strength (1.15); a break time divided by it stays finite and positive. |
| Contact and fine set out of step | `sample_at` answers nothing while they differ. Every writer of the fine set (edit, landing, water tick, dusk relight) moves the contact with it. |
| **The tree path** | **The cause.** See below. |

### The cause

With the button down and a digger in hand, `dig_and_place` first asks
`tree_on_ray` for a trunk nearer than the ground, and if there is one it works
that tree and returns. `tree_on_ray` tests every finest record that
`flora::grows_tree` says grows a tree: a pure function of the cell's seed
lanes, true forever, felled or not. A felled tree is not drawn
(`trees_taken::refresh` lists it for the draw to leave out) and its ground
is no longer rooted (`under_standing_tree` checks the `tree` record), but the
hit-test never looked at the record. So after a tree is felled:

- a ray through where its trunk stood (0.45 m round the cell's centre, up to
  3 m) or its crown (1 m round, 3 to 6.5 m) is taken by the tree path;
- `Breaking` runs on `(tree, TREE_LAYER)`, the tool swings on the dig's beat,
  and each "break" calls `take_log`, which says "No more wood" and gives
  nothing;
- the return skips the ground: the stump's cell and the ground just past it
  never dig, by any tool or by hand.

Standing on a felled tree's cell (which is where a player who just felled it
steps), the eye is inside the invisible trunk: looking down at the ground at
your feet always meets it first. From the player's side it reads as the tool
having stopped working, and it "comes back" on walking a few metres away.

### Fix

`tree_on_ray` takes a test of whether a tree still stands, and the dig passes
the same one `under_standing_tree` uses: the `tree` record's logs under
`axe_logs`. A felled tree is passed over and the ray goes on to the ground,
or to a standing tree behind it. One helper, `tree_standing`, serves both.

### Proof

`desktop::digging` app test: a real fine set, a tree near the fields spawn
marked felled in the save, a shovel in hand, the camera looking down past the
stump's trunk at its cell; holding the left button through `dig_and_place`
digs the cell's top layer. Before the fix the same test runs the tree path
and the cell is untouched.

## 2. Bait by the water

### What was there

`GatherSettings::bait(material, cell, layer, placed, altitude_m)` gave bait
from grass, soil, dirt or sand whose layer altitude lay in `bait_band_m`
(-2.0 to 1.0 m from the sea's level), one cell in `bait_one_in` (3). A river
runs at its reach's level, metres over the sea inland, so no river bank ever
qualified, and a beach above 1 m did not either.

The counting was checked end to end: `dig_and_place` writes one
`Gained { item: Good(BAIT), count: 1, how: Found }` per find;
`missions::happenings` turns `Found` into `Find` and `Gather` with
`item_code`, which is the good's code, `bait`; the mission's objective is
`(kind: gather, what: "bait", count: 3)`. That path was right; only the rule
gave nothing. (The objective counts finds while the mission is open, as
every gather objective does; bait dug before "Go fishing" is done is carried
but not counted.)

### The rule

Digging grass, dry grass, jungle grass, soil, dirt or sand turns up bait, one
cell in `bait_one_in` (3) by the cell and layer's hash as before, when the
ground is **by the water**:

- some column within `bait_reach_cells` (3) cells of the dug cell, the dug
  cell's own included, counted in steps across the finest neighbour table
  (5 or 6 a cell);
- holds a water cell (still or flowing: `Material::is_water`) whose layer is
  from 3 m under the dug layer to 2 m over it: `bait_water_m: (-3.0, 2.0)`,
  the water's altitude less the dug layer's.

Three cells is about 8.5 m at the gold-standard tile width: a river's bank and
the strip of ground beside it, the beach and the dune behind it. The height
window keeps a cliff top over the sea dry and a mine under a river dry. Never
from a block a player placed (the dug cell's own edit), as before.

Sand counts: the sea's edge is a band of beach sand, so leaving it out would
make bait at the sea nearly impossible to find, which is the complaint; the
owner's "(in dirt)" is read as "out of the ground", against stone and rock.
Snow, rock and stone give none. Recommendation taken (ask only with
screenshots).

Water a player poured from a bucket counts as water: the rule reads the
tier's cells, not where they came from. A pond dug and filled by hand grows
worms at its edge, which is harmless.

### Where the nearness lives

One rule in `pbd_core::gather`: `GatherSettings::by_water(from, layer,
neighbours, is_water)`, a walk of `bait_reach_cells` rings over whatever
neighbour table the caller has, asking the caller's columns whether a layer
is water. The CPU's authoritative water is the column tier: river sources
and flowing water written by `rivers::water` and the flow rule, the sea up to
its level, and every edit. `rivers::beside` was the other choice; it knows
only the generator's rivers and not the sea, nor water a player let in or
dug away, so it would answer for a world that is not the one on screen.

The app calls it with the fine set's `finest_neighbors` and each record's
column (a rim record with no column answers from the column the edit path
would adopt, which carries the river and the sea the same way).

### Proof

- `pbd_core::gather` tests on a small synthetic ring of cells: by a river
  inland, by the sea, nothing past the reach, nothing far over or under the
  water, nothing from a placed block, the share near a third.
- `desktop::digging` tests on the shipped world's own ground
  (`planet::test_ground`): an earth cell beside an inland river's water and
  one beside the sea are by the water; a cell in the fields far from water
  is not.
- `desktop::digging` mission test: three single bait finds, as the dig
  raises them, close "Dig for bait"'s `dig bait 3`.

## Files

`crates/pbd-core/src/gather.rs`, `assets/config/gather.ron`,
`crates/pbd-app/src/desktop/digging.rs`. Nothing in the tree-shake or water
work: `tree_on_ray`'s signature gains a parameter; its hit volumes are
unchanged.
