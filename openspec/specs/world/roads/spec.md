# World Roads Specification

## Purpose
Road cells that slope instead of step, and the roads the world lays between
towns. Built by `townsfolk` (design 10, 10a and 11, tasks 8.2 and 8.4);
the player's own road building (task 8.3) is in it too, and cut and fill
(roads version 2) and the lamps near towns (design 32 and 34).

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

The world SHALL plan roads once from the settlements and the seed, joining
each settlement to its two nearest neighbours within 2 km and the largest
towns to each other, and SHALL save them as records it authored. A road cell
SHALL be laid only where the player has made no edit. A world saved before
roads SHALL gain them when next opened, without a restart.

#### Scenario: A player's house on the route

- **WHEN** a planned road would cross a cell the player built on
- **THEN** the road goes round it and the player's cell is unchanged

#### Scenario: An old save

- **WHEN** a world saved before roads is opened
- **THEN** its towns are joined by roads, and nothing the player made has
  changed

Pinned by `pbd_core::roads::tests::every_settlement_reaches_its_two_nearest_within_2_km`,
`a_road_goes_round_the_players_work`, and `pbd-app`
`roads::tests::a_road_is_never_laid_on_a_players_edit` and
`a_save_from_before_roads_gains_them_once_in_place`.

### Requirement: Dirt and gravel

The player SHALL be able to lay a dirt road cell with the shovel and a
gravel road cell with gravel, each saved as the player's with the pack in
the same line, and walking SHALL be faster on gravel than on dirt.

#### Scenario: Building a road

- **WHEN** the player places a gravel road cell
- **THEN** one gravel is taken, the cell is saved as the player's, and the
  walker moves faster on it

Pinned by `pbd-app` `roads::tests::the_player_lays_gravel_and_dirt_and_takes_them_up`
and `gravel_is_walked_faster_than_dirt`.
