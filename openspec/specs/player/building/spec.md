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

### Requirement: A building stands on its ground and casts the sun's shadow

A floor of the player's with no floor or stair of its plot under it, over
ground lower than it, SHALL stand on a solid fieldstone footing down to that
ground. Every piece of a player's building but its lanterns' flames SHALL
cast into the sun's shadow cascades, as a town's building does.

(`building::tests::a_floor_over_lower_ground_stands_on_a_footing`, `pbd-app` `builds::tests::a_plot_casts_the_suns_shadow_but_its_flames_do_not`)

#### Scenario: A house on a terrace

- **WHEN** a house's ground floor stands two layers over the ground of some
  of its cells
- **THEN** a footing fills the space under those floors to the ground, and
  the walker cannot walk under the floor

### Requirement: A ridge beam needs a post at each end

A `beam` piece SHALL lie across the tops of the posts at two different
corners, both standing on its layer, and SHALL be refused (`posts`) unless a
post stands at each end. While it stands, neither post SHALL be taken away.
It is a timber of its kit's post, cut between the posts' tops
(`coop-missions` task 3.2). Build mode does not offer it yet: it is placed
by two players at once, which waits for the step for two.

(`building::tests::a_beam_needs_a_post_at_each_end`)

#### Scenario: One post

- **WHEN** a beam is asked for with a post at one end only
- **THEN** it is refused as needing a post at each end
