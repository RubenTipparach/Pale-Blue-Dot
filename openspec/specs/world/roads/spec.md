# World Roads Specification

## Purpose
Road cells that slope instead of step, and the roads the world lays between
towns. Built by `townsfolk` (design 10, 10a and 11, tasks 8.2 and 8.4);
the player's own road building (task 8.3) is in it too, and cut and fill
(roads version 2) and the lamps near towns (design 32 and 34), and the
roads marked by survey stakes and paved with gravel from a grinder and a
mixer, and roads version 3's sparse network (design 39 to 41, 43 and 44).

## Requirements

### Requirement: A road slopes, never steps

A road cell SHALL be a hex whose top may tilt from one layer to the next
across the cell, rising at most one layer (1 m) per cell. Its slope SHALL be
its collision, so the walker moves up and down it without a step.

#### Scenario: Up a hill

- **WHEN** the walker walks a road climbing ten layers
- **THEN** its height changes smoothly and it never steps up

Pinned by `pbd_core::roads::tests::a_road_up_ten_layers_never_steps` and
`pbd-app` `walking::tests::a_walker_climbs_a_road_over_ten_layers_without_a_step`.

### Requirement: A road is its own shape over its ground

A road cell SHALL be a piece over its cell's ground with its own top, sides
and floor, its slope only the piece's (`planet/contact` still reports a
step as a step), and the walker SHALL stand on the top that is drawn. Only
the world's cutting for a version 2 road SHALL change the ground, and a
road SHALL be laid wherever the player has not touched its cell.

#### Scenario: Standing on a sloped road cell

- **WHEN** the walker stands anywhere on a sloped road cell
- **THEN** its feet are on the drawn road top there, within 3 cm

#### Scenario: A road's own cutting

- **WHEN** a road cell holds the world's edit (its cutting)
- **THEN** the road is laid there; with the player's edit it is not

Pinned by `pbd_core::roads::piece::tests::a_sloped_cell_is_stood_on_where_it_is_drawn`,
`pbd_core::roads::tests::a_cut_road_is_a_schema_2_record_and_digs_its_layers`
and `pbd-app` `roads::tests::a_road_is_never_laid_on_a_players_edit`.

### Requirement: Cut and fill reach a step a climb cannot

A road planned by roads version 2 SHALL, where the next cell stands more
than a layer from it, stand a cell up to two layers over its ground on
packed earth or cut it up to two layers in, never on water, in a town or on
a cell the player touched, and still never more than a layer between
cells. A world planned by version 1 SHALL keep its roads; version 1 SHALL
plan as it did.

#### Scenario: A two-layer cliff

- **WHEN** a link's only way crosses a step of two layers
- **THEN** version 1 finds no road, and version 2 finds one that cuts or
  fills within its bounds, the same every time

Pinned by `pbd_core::roads::tests::a_cliff_is_cut_and_filled_in_version_2_and_no_road_in_version_1`
and `cut_and_fill_keep_off_water_towns_and_the_player`.

### Requirement: The roads near a town are lit

Within 90 m of a settlement, every fifth cell of a road out of it SHALL
have a lantern post beside it, off the road, on dry ground the player has
not touched, the same every load.

#### Scenario: Leaving town at night

- **WHEN** the player walks a road out of a town at night
- **THEN** lamps stand beside it near the town, and none further out

Pinned by `pbd_core::roads::tests::a_road_near_a_town_is_lit_by_lamps_beside_it`.

### Requirement: Roads join the towns

The world SHALL plan roads once from the settlements and the seed and SHALL
save them as records it authored. A world planned from now on (roads
version 3) SHALL join each settlement to at most one other within 2 km, so
that no network of its roads joins three or more settlements; a world
planned by an earlier version SHALL keep the roads it has, cell for cell.
A road cell SHALL be laid only where the player has made no edit. A world
saved before roads SHALL gain them when next opened, without a restart.

#### Scenario: A new world

- **WHEN** a new world's roads are planned
- **THEN** every network of roads joins one or two settlements

#### Scenario: A world planned before

- **WHEN** a world whose roads version 2 planned is opened
- **THEN** its roads are the ones it was saved with

#### Scenario: A player's house on the route

- **WHEN** a planned road would cross a cell the player built on
- **THEN** the road goes round it and the player's cell is unchanged

#### Scenario: An old save

- **WHEN** a world saved before roads is opened
- **THEN** its towns are joined by roads, and nothing the player made has
  changed

Pinned by `pbd_core::roads::tests::version_2_links_are_pinned` (versions
1 and 2's links, taken before version 3),
`no_network_joins_three`, `every_settlement_reaches_its_two_nearest_within_2_km`,
`a_road_goes_round_the_players_work`, and `pbd-app`
`roads::tests::a_road_is_never_laid_on_a_players_edit` and
`a_save_from_before_roads_gains_them_once_in_place`.

### Requirement: Dirt and gravel

The player SHALL pave a marked cell by the interact key with the shovel in
hand, one gravel a cell for a gravel road or a ford and none for a dirt
road, the cell's slope cut by the world road's own rule from its road's
order, saved as the player's with the pack in the same line, and walking
SHALL be faster on gravel than on dirt. The place button SHALL NOT lay a
road cell. Every road cell a save already holds, gravel or dirt, SHALL be
kept and walked as it was.

#### Scenario: Building a road

- **WHEN** the player, shovel in hand, presses the interact key on a marked
  cell of a gravel road with gravel in the pack
- **THEN** one gravel is taken, the cell is a gravel road cell saved as the
  player's, and the walker moves faster on it

#### Scenario: No gravel

- **WHEN** the player presses the interact key on a marked cell of a gravel
  road with no gravel
- **THEN** nothing is taken and the cell stays marked

#### Scenario: A dirt road

- **WHEN** the player paves a cell of a road planned in dirt
- **THEN** nothing is taken and the cell is a dirt road cell

#### Scenario: An old player road

- **WHEN** a world holding road cells the player laid by the place button is
  opened
- **THEN** every one is drawn and walked as before

Pinned by `pbd_core::roads::planner::tests::a_road_up_a_slope_never_steps_and_paves_the_same_in_any_order`,
`pbd-app` `roads::tests::a_plan_is_marked_and_paved_and_survives_a_reload`
(one gravel a cell, the tufts gone at once),
`a_dirt_road_is_free_and_the_eraser_gives_its_stakes_back`,
`gravel_is_walked_faster_than_dirt` and
`the_player_lays_gravel_and_dirt_and_takes_them_up` (an old player road
still reads, draws and is taken up).

### Requirement: A road is marked before it is paved

The player SHALL mark a road for paving with survey stakes in hand by two
clicks, its start and its end. Nothing SHALL be planned before the first
click. The road's cells SHALL be the straight line of cells between its
stakes, the same both ways round, staked every `max_segment` cells, each
checked by the one rule the world's roads use: no step of more than a
layer, never the player's work, the sea, a town's footprint, water deeper
than a ford or a cell a tree stands on. A road with a clashing cell SHALL
be refused with nothing taken and the reason shown; only the clashing cells
SHALL be red in its ghost. A click on a stake of any road SHALL start a new
road from it, sharing that stake: roads are added, never replaced. The
eraser SHALL delete a whole road, its stakes coming back to the pack but
any another road holds.

#### Scenario: Two clicks on a hillside

- **WHEN** the player clicks a start, then an end 12 cells away up a slope
  that rises a layer a cell
- **THEN** the cells between are marked, and no two marked neighbours differ
  by more than a layer

#### Scenario: A tree on one cell

- **WHEN** a tree stands on one cell of the aimed line
- **THEN** that cell alone is red with "A tree in the way", and the end
  cannot be set until it is felled

#### Scenario: Too steep

- **WHEN** the line to the aimed cell climbs two layers between two cells
- **THEN** the higher cell is red, the road is refused, and the stakes stay
  in the pack

Pinned by `pbd_core::roads::planner::tests::a_line_is_the_same_both_ways_round`,
`each_reason_reds_only_its_own_cell`, `the_grade_reds_two_rises_too_close`,
`roads_are_added_from_a_stake_never_replaced`,
`the_eraser_takes_a_whole_road_and_its_stakes_back`,
`moving_and_taking_back_stakes` and `roads::tests::world_routes_are_pinned`.

### Requirement: Marks are saved

A plan's roads (their stakes, width, surface and lamps) SHALL be a durable
record of the player's, written with the pack in the same line, and SHALL
read back cell for cell whatever the road search's version. A plan saved
before roads were planned by their ends SHALL read as one road of width 1 in
gravel over the cells it stored.

#### Scenario: A plan after a reload

- **WHEN** the player marks a plan, closes the game and opens the world
- **THEN** the same cells are marked and the same stakes stand

#### Scenario: An old plan

- **WHEN** a world holding a plan of stakes and searched cells is opened
- **THEN** the same cells are marked, as one gravel road one cell wide

Pinned by `pbd_core::roads::planner::tests::a_schema_1_plan_reads_as_a_width_1_gravel_plan_and_a_plan_reads_back`
and `pbd-app` `roads::tests::a_plan_is_marked_and_paved_and_survives_a_reload`.

### Requirement: Gravel comes from a grinder and a mixer

Gravel SHALL be made at a mixer from crushed stone and sand, and crushed
stone at a grinder from stone; neither SHALL be made by hand. Gravel already
held SHALL stay usable.

#### Scenario: From stone to gravel

- **WHEN** the player crushes 2 stone at the grinder, then mixes 3 crushed
  stone with a block of sand at the mixer
- **THEN** the pack holds 4 gravel, and the pack's craft list has no gravel
  recipe

Pinned by `pbd_core::station::tests::the_shipped_recipes_resolve_and_agree_with_the_defaults`.
