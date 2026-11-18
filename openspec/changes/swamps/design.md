# Design

## Measured (2026-11-18, version 10, the shipped seed)

The instrument is `planet_gen::tests::swamp_report` (ignored; run with
`PBD_SWAMP_VERSIONS=10,11 cargo test -p pbd-core --lib swamp_report --
--ignored --nocapture`): 200,000 spread directions, 83,506 of them land, each
classified, its swamp cells located (latitude, height, a river within 60 m,
the sea within 120 m) and its low land split by moisture third. A
measurement instrument only: it moves nothing the generator makes.

| | version 10 |
| --- | ---: |
| Swamp, share of the land | **1.19%** |
| Jungle / Fields / Desert | 24.96 / 26.10 / 10.45% |
| Beach / Mountains / Tundra | 10.02 / 4.51 / 22.78% |
| Swamp cells below 5 m over the sea | 100% |
| Swamp cells within 120 m of the sea | 98% |
| Swamp cells within 60 m of a river | 69% (the rivers' mouths) |
| Swamp top: dirt / grass / water | 64 / 29 / 7% |
| Jungle above 30 m | 64% |

Why there are so few, and why they do not read:

1. **The 5 m cut.** `biome_at` makes a cell Swamp only when its surface is
   under `swamp_max_elev_m` (5 m). The beach takes everything under 2 m, so
   the swamp is the 3 m between, which on this body's slopes is a strip a
   few cells wide round the coast.
2. **The jungle mask.** Only the wet third of the moisture (over
   `wet_above`, 0.524) can be swamp, and the jungle stands high: 64% of it
   above 30 m, 1.2% of the land is both wet and under 5 m.
3. **The moisture field** (750 m, `bigger-biomes`) does not know where the
   water is: the rivers (version 8) and the sea are not in it, so nothing
   raises the swamp near them, and a river inland never has one.
4. **The ground rule.** Water in the lowest layer (rimmed, 2 to 3 m), dirt
   in the next (3 to 4 m), grass only above 4 m: in a 3 m strip that is two
   thirds bare dirt, which stands no tree (the trees stand on the grasses),
   so the swamp's willows are almost never drawn.

Where the few are, by latitude band from the equator (10-degree bands): 12,
19, 11, 20, 21, 16, 1% and none past 60 degrees, the cold band being tundra.

Low land is scarce on this body: temperate land between 2 m and 12 m over the
sea is 9.9% of the land, a third of it dry. So a swamp of several percent
needs either a taller coastal band or the rivers, and the design takes both.

## Decisions

1. **Where a swamp stands** (`swamp_holds`, version 11 only). After the
   ocean, beach, tundra, mountains and desert tests, a cell is Swamp when:
   - **by the sea:** its surface is under `coast_m` (12 m) over the sea
     times a smoothstep of the moisture from `wet_from` (0.47) to
     `wet_full` (0.56): 12 m where it is wettest, nothing at 0.47, so the
     swamp thins toward the fields instead of stopping at a line; or
   - **by a river:** moisture at least `river_wet_from` (0.47), within
     `river_m` (40 m) of a river's banks (`rivers::beside`, the drainage
     map's own lookup), and no more than 1.5 to 3 m (by wetness) over the
     reach's water: the river's floodplain, at any height.

   Version 10's strip (wet and under 5 m) is kept inside it, so every
   version 10 swamp is a version 11 swamp. Every term is a smooth field
   (height, moisture, a river's distance and level), never a cell's
   neighbours, so a swamp has edges a walker crosses, not a speckle.
   "Flat" in the owner's sense is the low land itself: the shore's easing
   and the river valley's floor are the flattest ground there is; a
   per-cell slope test was measured (half the low land is within a metre
   of all six neighbours) and left out, because it breaks the swamp into
   single cells.

2. **The share** (recommendation taken, ask only with screenshots): about
   6.6% of the land, measured 6.57%, inside the 4 to 8% asked for.
   `swamps_hold_several_percent_of_the_land` holds 4 to 9% on the shipped
   seed and two others. Where it comes from, measured: jungle 64%, fields
   18%, version 10's swamp 18%. 82% of it is in the wet third, 31% stands
   above 30 m (the rivers' floodplains inland), 71% within 120 m of the sea.
   By latitude band: 15, 16, 15, 15, 17, 20, 2% and none in the cold band.

   | | v10 | v11 |
   | --- | ---: | ---: |
   | Swamp | 1.19 | **6.57** |
   | Jungle | 24.96 | 20.77 |
   | Fields | 26.10 | 24.90 |
   | Desert, Beach, Mountains, Tundra | unchanged | unchanged |

3. **The ground** (`top_material`, version 11's swamp branch). A cell is
   **water** where the ground round it holds a pool
   (`swamp_pool_held`): every point of three rings of sixteen, at 2.5, 2.9
   and 3.3 m (the spread of the cells' widths), stands at least 0.25 m over
   the pool's floor, so no neighbour, whichever way it lies, is lower; and
   in the coast's lowest layer as version 10 has it. A patch field
   (`swamp_patch`, 36 m) gates where pools may stand (over `pool_above`,
   0.3: nearly everywhere) and lays the **mud** (`Dirt`): a cell that holds
   no pool is mud where the field is over `mud_above` (0.6), and in the
   coast's second layer; **swamp moss** (`Grass`, drawn with the swamp
   sheet's moss) elsewhere. Measured: moss 66%, water 9%, mud 25% (version
   10: 29, 7, 64).

   Version 8's `rivers::rimmed` (six points at 2.9 m) was tried first and
   measured to spill: a fresh tier by a river
   (`river_tier_tests::a_freshly_built_tier_s_rivers_are_what_the_rule_keeps`)
   held 1,761 flowing cells outside a river's banks, pools running over
   their rims between the six points and draining into caves. Two changes
   make the test pass with none: the three rings with their margin, and
   **no cave opens within `SWAMP_SEAL_M` (6 m) of a version 11 swamp's
   surface** (`column::generate_edited`, as a river's valley is sealed).
   The rings cost up to 48 altitudes for a swamp cell, at generation only;
   most cells fail on the first few.

4. **Willows.** The tree table is version 10's (`TreeTable::V10`, the swamp
   67 in 256 of its moss cells, M13's `treeP` 0.26), the willow 0.75 and
   the birch 0.25 (`tree_kit`). Nothing in the table changes: the swamp
   simply has moss now, so its trees are drawn.

5. **Its clutter.** What the swamp grows is already its own
   (`flora::BIOME_FLORA`, `TUFT_RAMPS`): moss tufts in the swamp's ground
   ramp, twice the mushrooms, white and yellow flowers, vines on the
   willows. M13 (the approved foliage mockup, `ferns: { kind: "reed" }`)
   draws reeds where the other biomes have ferns, and the game drew ferns.
   The swamp's fern piece is drawn as reeds: a few near-upright stalks a
   metre or so tall, each with a dark head, in the swamp's leaves. Draw-time
   (`planet_surface.wgsl`), no generator moves, so version 10's swamps get
   them too. The piece is still the fern piece to a hand (`scatter`), so
   gathering is unchanged.

6. **Sites** are surveyed on ground without the swamps (`site_ground`
   strips them, as it strips version 8's rivers): version 11's site list is
   version 10's, the one approved for a new world. Measured
   (`sites::tests::sites_in_version_11_swamps`): the lists are equal (55
   sites), no anchor but the three swamp villages' stands in a version 11
   swamp.

7. **Climate.** The atmosphere reads the biomes at its grid's centres
   (wetness, albedo). It reads `climate_ground`, version 11 without its
   swamps, so version 11's settled climate is version 10's, and
   `settled-g11-l{3,5}` ship as version 10's bytes, as version 10's were
   version 9's. A level-5 settle takes hours and cannot run here; the
   swamp's slightly wetter air (wetness 1.0 against the jungle's 0.9) is
   left out of the weather for now.

8. **Version numbers.** Swamps take version 11. `mountain-ground` planned
   version 11 and is not built; its write-up now says 12 and that it builds
   on version 11.

## What stays

- Every height: version 11's altitudes are version 10's bit for bit, so
  the rivers' drainage map, the sea's places, the caves and the ore are the
  same. (The river map is built per config, so a version 11 world builds or
  reads its own cache file once, with the same contents.)
- Versions 4 to 10: their configs are untouched (`swamps: None`), version
  10's ground pinned by digest before this landed
  (`version_10_keeps_its_ground`).

## Risks

- **Cost.** `biome_at` now reads a river lookup on wet land above the
  coastal band (a 5 x 5 window of the drainage map, skipped where
  `may_reach` says no river is near), and a swamp's top reads the patch
  field and the rim (six altitudes) on swamp cells only. Generation time
  was not measured in the cloud; the owner's perf sitting covers it.
- **Climate drift.** Decision 7 keeps the weather blind to the new swamps;
  a later settle on the owner's machine can fold them in as its own change.
- **A pool at a tier seam** is decided by the same pure function on both
  sides, so the tiers agree, as the version 8 marsh does.
- **Generation cost of the pool rings** (decision 3): the river tier test
  took 3 s with six points and about 14 s with the rings (this container,
  under load from other jobs, so a rough figure). The owner's machine
  should measure tier build time before and after (task 4.3).
