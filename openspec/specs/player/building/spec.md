# Player Building Specification

## Purpose
How a player puts buildings together from the pieces the planet's towns are
made of (`player-building`): placement checked by the towns' own rules,
fitted roofs, and every piece kept as a world edit. What is here is built
and tested; build mode, snapping, blueprints and the build camera are still
in the `player-building` change.

## Requirements

### Requirement: A placement is checked by the towns' own rules

A placement SHALL be refused when it would break any layout rule a town is
held to:
- headroom at every walkable point;
- roofs never overlap;
- every stair lands on a layer line;
- no building on a pentagon or a pentagon's neighbour;
- furniture stands clear of walls;
- no two faces in one plane.

A refused ghost SHALL be shown in red with the rule it breaks. The check SHALL
be the same function the town layouts are checked with.

(`building::tests::each_rule_refuses_with_its_own_name`, `every_plain_template_building_is_accepted_whole_and_piece_by_piece`, `nothing_is_built_on_a_pentagon_or_beside_one`)

#### Scenario: A low floor over a stair

- **WHEN** the player places a floor that would leave less than the walker's
  height over a stair below
- **THEN** the ghost is red and says there is no headroom over the stair, and
  nothing is placed

#### Scenario: One rule, two callers

- **WHEN** a player-built house and a town's house are checked
- **THEN** both are checked by the same function, with the same result for
  the same pieces

### Requirement: A roof is fitted over what it covers

The roof tool SHALL fit the kit's roof over an enclosed footprint the player
picks, the way a town's roofs are fitted. A roof SHALL NOT be placed tile by
tile.

(`building::tests::the_roof_tool_roofs_a_two_by_four_house`)

#### Scenario: Roofing a house

- **WHEN** the player uses the roof tool on a house of two rows of four cells
  with walls all round
- **THEN** a gable roof is fitted over its rows, with its eaves, and it passes
  the roof-overlap check

### Requirement: What is built is kept, and can be taken back

Placing or removing a piece SHALL be a world edit that enters the durable
transaction path at once. A player SHALL be able to remove a piece they
placed. Removing a piece that something else stands on or hangs from SHALL
be refused, with the reason.

(`builds::tests::a_built_room_survives_a_reload_and_an_old_log_reads_as_it_did`, `building::tests::removing_a_floor_under_a_lantern_names_the_lantern`)

#### Scenario: Reloading

- **WHEN** the player builds a room and the world is reloaded
- **THEN** the room is there, piece for piece

#### Scenario: Taking out a floor under a table

- **WHEN** the player removes a floor a table stands on
- **THEN** the removal is refused and names the table
