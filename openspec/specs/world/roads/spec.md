# World Roads Specification

## Purpose
Road cells that slope instead of step, and the roads the world lays between
towns. Built by `townsfolk` (design 10, 10a and 11, tasks 8.2 and 8.4);
the player's own road building (task 8.3) is still in that change.

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

### Requirement: A road is its own shape over unedited ground

A road cell SHALL be a piece laid over its cell's generated ground, with its
own top, sides and floor, and SHALL NOT change the ground: the terrain's
contact keeps reporting a step between cells as a step
(`planet/contact`), and a road's slope comes only from the road piece. The
floor the walker stands on SHALL be the top that is drawn.

#### Scenario: Standing on a sloped road cell

- **WHEN** the walker stands anywhere on a sloped road cell
- **THEN** its feet are on the drawn road top there, within 3 cm
- **AND** the cell's column is the generator's, with no edit written

Pinned by `pbd_core::roads::piece::tests::a_sloped_cell_is_stood_on_where_it_is_drawn`
and `pbd-app` `roads::tests::a_save_from_before_roads_gains_them_once_in_place`
(the plan writes records and no edit).

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
