# Design: dry holes, and a sea that flows in

The owner, 2026-11-18: "We should be able to dig holes below water line and
not have it filled with water too", then "Ocean needs to work like rivers.
In the sense that the water flows."

## 1. The finding: a dry hole painted over by the sea's reach

**Measured** with `dry_hole_report` (an ignored test in `planet_water.rs`,
a measurement instrument). It builds the fine set at a place as the game
does, finds a pit site, and digs a seven-cell pit (a cell and its six
neighbours) through the edit's own path: `FineSet::write_layer`, the
relight, `mark_sea`. It then reports for every dug cell:
- what the column holds;
- whether the sea's sheet is listed over it (`sheet_listed`);
- the reach and dry marks;
- what the eye 1.6 m over the pit's floor is told (`eye_water_at`) and
  what the walker reads (`column_water`, `sea_reached`).

It does this after the dig, after the rule has run on everything the dig
queued, and after a reload (the tier built again from the save).
`PBD_AT='lat lon'` picks the place, and `PBD_RIVER=1` or `PBD_SHORE=1` a
river's bank or the shore itself.

| Place | Pit | Columns | Sheet listed | Eye in the pit | After the rule / reload |
| --- | --- | --- | --- | --- | --- |
| `39.61458 15.39634`, land +1 to +2 m, beach cells beside it, the sea three cells off | floor 3 m under the sea | all air | yes, all 7 | `Water { surface 4800.0 }` | the same |
| `18.97859 27.63335`, land +2 to +5 m, a beach cell beside it | floor 3 m under | all air | yes, all 7 | `Water` | the same |
| `39.55576 16.31881`, land +4 to +6 m, no ground under the high water within two cells (`PBD_HIGH=1`) | floor 3 m under | all air | no, dry | `Air` | the same |
| `39.63239 15.47088`, a river bank near the coast | floor at +0 m, 3 m under the river | all air | yes | `Water` | the same |
| `39.51022 15.83776`, a river bank inland, river top +4 m | floor at +1 m, 3 m under the river | all air | no, dry | `Air` | the same |

**No water is in the hole.** The liquid rule seeds only from neighbouring
water. The sea's flood (`flood_from`) fills only what is joined to the sea's
water below sea level. Neither touches these pits, and after the rule runs
and after a reload their columns are still air.

**What fills it is a picture: the sea's reach.** `ColumnTier::sea_reach`
(`ocean-environments` B22) decides where the sea's analytic sheet lies. It
walks from the open sea through every cell that is not solid, at every
layer up to the high-tide line. That line is 1.5 m over the mean sea level
(`high_water_m`: the tide's highest, +2 m, less the sheet's 0.5 m offset).
The walk crosses:
1. the air over low ground near the coast: a beach at +1 m, a river bank
   near the sea;
2. into a pit dug beside that ground;
3. and down the pit through its air, however deep.

The pit is then marked reached (`SEA_REACH_BIT`), and every reader of that
mark treats the dry hole as the sea:
- **the sheet** (`planet_visibility.wgsl` lists it over a reached record) is
  drawn over the pit at the tided sea level, so from above the pit looks
  full of sea;
- **the eye** (`eye_water_at`'s "air the sea's surface reaches" arm) is
  under the sea, so the compose draws underwater fog and distortion, and
  from inside the pit the sheet hangs overhead;
- **the walker** (`swim_surface`) swims in it.

The captures (`docs/screenshots/dry-holes/`, "before") show it: standing in
a pit dug 3 m under the sea's level at `39.61458 15.39634`, the eye sees
dry sand walls through blue-green murk, with the sea's surface over its
head.

B22 made this so on purpose. A pit behind a beach that the high tide
overtops was meant to read as a tide pool, and that fixed the owner's
"holes in the sea" at generated caverns beside the beach. For a pit the
player digs it is wrong twice over:
- it fills a hole that touches no water at mean tide;
- it fills it to the tide's height at every tide, low included, because
  the reach is a mark, not water.

Further inland, where no ground within reach of the pit is under the high
water, a dug pit is already dry today. That is the third row, the fifth
row and the CPU half of the tests.

The other three candidates were checked and are not the cause:
- **(b) the river surface field** (`river-cache`, `river_band`): it draws
  only for cells off the column tier, and a dug cell is on the tier (an
  edit past the edge adopts a column). The inland river pit is dry today.
- **(c) generation**: a column adopted after an edit is generated solid
  with its edits (`generate_edited_solid`). Generation's water is only over
  the ground, and the dug layers are written over it.
- **(d) the liquid sim**: nothing seeds it.

## 2. How the sea is held today (measured)

- **Generation** (`column::generate_solid`) fills every layer from the
  ground up to the sea's level with `Material::Water` sources, so the open
  sea on the tier is cells, and its top layer is `sea_layer() - 1` (layer
  144 under a sea layer of 145 on the default world). Off the tier the sea
  is the height field's: everything under 0 m is sea.
- **The sea's surface and the tides are analytic.** The sheet is one mesh
  per listed record at `sea - 0.5 m + tide`, the tide from the moon
  (`pbd_core::sea`, range 3 m, up to +1.5 m over the mean at the sheet).
  The waves are a vertex function. Where the sheet is drawn, the mesher
  leaves the sea's top layer to it (`sheet_listed`).
- **The flood.** At the tier's build (`sea_seeds` and `flood_from`) and on
  every edit (`water_after_edit`), air under the sea's level joined to the
  sea's generated water across the sides or downward is filled as sources,
  all at once, up to `FLOOD_LIMIT` cells. It is derived: never saved, and
  the same tier floods the same way at every build.
- **What an edit seeds.** `water_after_edit` seeds the flood from the
  edited cell and its eight neighbours wherever they hold generated water.
  It then queues the cell for the rule (`water-flow` decision 5).
- **The rule** (`pbd_core::liquid`, Luanti's) moves poured and uncovered
  water, and saves what it writes as edits authored by the water. A source
  is renewed only between three sources. A lone source makes flowing water
  that runs out at seven cells, so a river's reach is held by its sources.
  The sea is never seen by the rule as anything but generated sources.

So the sea comes into a dug channel all at once, with no flow and no fall,
and its surface over a dry hole is decided by a walk through air.

## 3. Decision: a dug hole is dry until the sea's water reaches it

The sea's reach walks through the air under the sea's level only in
columns no player has dug or built in. A column with any edit other than
water (`TierWater::dug`: set at the build from the save's edits, and by
`water_after_edit` on every edit) is entered:
- over the sea's level, through the tide's air, as before;
- under it, only through water.

A dug column the walk met only over the sea's level, whose top layer under
the sea's level is air, is not reached. A dug column is the sea's when the
sea's water stands at its top layer, or when that layer is solid (a beach
the player dug or built on, which the tide covers as before).

So:
- a dug pit that joins no water is dry at every depth and every tide, with
  no sheet, no underwater view and no swimming;
- a pit the sea is still flowing into is not under the sheet until its top
  layer is the sea's: the mesher draws its water meanwhile, falls included;
- generated caves and pits behind a beach keep B22's sheet. That is the
  owner's earlier ask, and nothing a player did changed them.

**Recommendation taken (ask only with screenshots):** at a tide over the
rim of a pit dug in an intertidal beach, the sheet stops at the pit's edge
(design 7). The tide is drawn, not simulated, and a hole the player made
is dry until water reaches it. The other choice is B22's, where the pit
fills to the drawn tide's height at every tide, low included. That is the
filled hole the owner asked to be rid of.

## 4. Decision: the sea flows in by the rule

1. **The edit no longer floods.** `water_after_edit` fills only the cell
   the edit opened, and only where the sea's own water (generated, under its
   level, not the edits') is over it or beside it. So digging the seabed
   under the sea leaves no bubble. The fill is derived like the flood it
   replaces: never saved, and the build's flood fills the same cell from
   the same water. The cell and its neighbours are queued for the rule.
2. **The rule's sea clause** (`liquid::transform_at`, `Grid::sea`). A cell
   that is under the sea's level and joined to the sea there becomes a
   source when the rule would make it flowing water at the top level (fed
   by a source on a side or above, or by a fall) and it is not falling
   (it stands on something solid, or on a source). A source the clause
   makes queues the cell over it. So:
   - the sea's water spreads along a channel a cell a tick, each cell a
     source, and stands at the sea's level as the sea does;
   - over a lip, water with air under it still falls: the rule's falling
     water, drawn as every fall is (look unchanged);
   - at the fall's foot the floor fills, and the fall fills from its foot
     up, layer by layer, to the sea's top layer and no higher;
   - off the sea, or with the clause off, the rule is Luanti's exactly.
     `transform` is `transform_at(.., false)`, so the mockup's fixture
     (`tests/water_flow_presets.rs`) holds it to the same field.
3. **Joined** (`ColumnTier::sea_joined`, kept in `TierWater::joined`) is a
   walk from the open sea's top layer through every cell under the sea's
   level that is not solid, air or water alike. It is a function of the
   solids alone, so the water the rule moves never changes it. It is
   recomputed by `FineSet::mark_sea` after every edit, and at the end of the
   tier's build. It is empty while the build's rivers settle, so generated
   rivers settle by the rule alone, as before. It gates the clause:
   - a breached pit is joined, so the sea fills it;
   - a sealed pit is not, so a source poured into it is the bucket's,
     flowing water that stops at the range, and never the sea.

Measured by the tests on a line of columns (`water_tier_tests`):
- a channel two layers deep and five cells long fills along its length in
  5 to 12 ticks;
- a pit breached at the sea's top layer shows falling water over the lip on
  the first tick, and fills over several ticks to the sea's level, no
  higher.

In the game (`--pit` at the shore with `PBD_PIT_SEA_LAST=1`), a seven-cell
pit four layers deep opened to the sea settles after the rule has changed
11 cells. The rest filled at the edits that opened them onto the sea.

## 5. What stays analytic

The open sea's surface, its waves and its tides are as they were: a sheet
over listed records, drawn at the tided level, and the height field's sea
off the tier. Generated sea cells stay sources and are never queued, so
opening a world moves no water. Only what a player opens onto the sea
flows. That is the part a player can dig near, and the only part where
cells and the picture could disagree.

## 6. Cost

- **Per edit:** one more walk over the tier, `sea_joined`, beside the
  `sea_reach` walk it already makes. On a coastal tier of 3,923 columns at
  `39.725 15.339` (this cloud container's CPU, `--profile fast`):
  `sea_reach` 1.64 ms, `sea_joined` 1.16 ms. The edit already re-bakes the
  whole tier's light, which takes milliseconds.
- **Per tick:** `Grid::sea` is one lookup. The ticks run only while water is
  queued, as before.
- **Memory:** a bool per cell under the sea's level, about 0.6 MB for that
  tier.
- **Saves:** the sea cells the rule writes are saved as edits authored by
  the water, as every flowing cell is. A breach into a large dry cave
  writes one edit per cell the sea fills (the old flood wrote none). That is
  bounded by the rule's `LOOP_MAX` per tick, and by the tier.
- **Frame time was not measured** (a cloud session has no GPU: CLAUDE.md).
  No shader changed.

## 7. Not built now: the tide as flow (dropped)

**Dropped, 2026-11-19.** The owner had the tide removed ("Remove literal
tide feature"; `no-tides`). The sea's surface stands at one level, so there
is no intertidal beach and nothing for the tide to flow into; task 4.1 is
closed as dropped. The rest of this section is the record of the plan.


The tide stays a picture. A dug pit in a beach the high tide overtops is
dry, and at a tide over its rim the sheet stops at its edge. The sheet has
no sides, so its edge is open at the pit. The full answer is for the sea's
cells near the player to rise and fall with the tide, so that the high tide
flows into such a pit by the rule and the ebb leaves it as a tide pool.
That changes what is saved: tide water the rule writes would have to be the
tide's, not a player's, and drain on the ebb. So it is written up here as
the next task (tasks 4.1) and not built in this pass.

## 8. Saves

- No save format change and no generator version: generation is untouched.
- The `dug` mark is derived from the save's edits at every build, so an old
  save's dug holes are dry as soon as it opens, and its holes beside the sea
  are flooded at the build exactly as before. The build's flood is
  unchanged.
- Water the rule wrote in an old save (`Flow*` and sources) reads as it
  did. A world closed with the sea part way into a pit picks up where it
  was (`queue_edited_water`), and the build's flood fills the rest at once.

## 9. Validation

- `pbd_core::liquid::tests::the_sea_fills_what_it_feeds_and_falls_over_a_lip`
- `water_tier_tests`:
  - `a_pit_dug_below_the_sea_beside_a_beach_stays_dry`
  - `a_breached_pit_fills_from_the_sea_over_its_lip`
  - `a_channel_cut_from_the_shore_fills_along_it`
  - `a_bucket_in_a_sealed_pit_under_the_sea_is_not_the_sea`
  - `digging_through_to_the_sea_lets_it_flow_in`, which replaces
    `digging_through_to_the_sea_floods_on_the_edit`
  - B22's `the_sea_reaches_a_cavern_behind_the_beach_and_not_a_pit_inland`,
    unchanged
- `dry_hole_report` at the places above, before and after.
- Captures, before and after, in `docs/screenshots/dry-holes/` (lavapipe,
  fixed step: looks, not frame time).

## 10. A dry hole is not shaded as sea (the owner, 2026-11-19)

The owner, on `beach-pit-inside-before-after.jpg` and
`beach-pit-above-before-after.jpg`: "It shouldn't shade dry holes at all",
and on the second one's AFTER half: "This is green as fuck." In both AFTER
halves the pit's floor and lower walls are green, and its upper walls the
tan sand of the beach. The boundary between the two is level, about half a
metre under the sea's level: the height of the drawn sheet.

**Question.** Is that (a) the sea's underwater shading
(`planet_surface.wgsl`, "Submerged terrain": downwelling attenuation, the
deep colour mix, caustics, the `above_water` gate on rain wetness) still
reaching fragments of a dry pit, or (b) ordinary shade, sand lit only by the
blue sky in the pit's shadow?

**The instrument.** Copies of the capture build with a modified
`planet_surface.wgsl` in a scratch asset folder beside the exe (a "shipped"
build reads its own `assets/`, `paths::assets_dir`), so nothing in the
repository's source changes for it. Every fragment the submerged branch
tints (`water_depth > 0`) is painted by why its water surface stood over it:
- blue: `input.slot == 0`, off the tier: the sea's sphere, the sheet radius;
- red: on the tier, the column's packed water top within half a metre of
  the sea's level, which leaves the sea's sphere in place;
- magenta: on the tier, the column's own water over the fragment.
A fragment the branch leaves alone is drawn as the game draws it. The same
pit, inside and above, as the captures above.
