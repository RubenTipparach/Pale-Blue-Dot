# Player Building Planner Specification

## Purpose
How a player designs a structure before building it (`building-planner`):
a plan in its own hex coordinates, checked by build mode's own rules, house
and building presets, construction sites set out and changed mid-build, all
kept as stored records. What is here is built and pinned by tests in
`pbd_core::building::planner_tests` and `pbd_app::construction`; the
planner's screen, the mallet's strokes and the scaffolding's look are still
in the `building-planner` change.

## Requirements

### Requirement: The planner checks a plan with build mode's rules

The planner SHALL check a plan with the same rule functions build mode uses
for a single piece, on the plan charted onto a flat patch of its own. A
broken rule SHALL be reported on the piece that breaks it with the rule's
own reason. A plan larger than the planner's limits, or taller, SHALL be
refused with its own reason; a plan with no door on the ground storey
SHALL be warned about but not refused. A plan with a refusal SHALL NOT be
set out.

#### Scenario: An upper floor over nothing

- **WHEN** a cell of the upper storey stands over no room and no wall below it
- **THEN** the checks name the held-up rule's reason on that storey

#### Scenario: A plan too big

- **WHEN** a plan has more cells than `max_cells` or a cell past `max_radius`
- **THEN** the checks say "Too big to plan" and the plan may not be set out

### Requirement: Every preset passes the planner at every size

The game SHALL ship ten presets (a straw hut, a cottage, a two-storey
house, a townhouse, a workshop, a shop, a barn, a stable, a warehouse and a
tower), each at a small, standard and large size, laid out by rule, each
of a kit the cost table builds, and each SHALL pass every planner check. A
plan SHALL chart onto the planet's cells at any of six turns to the same
pieces turned.

#### Scenario: Six turns

- **WHEN** a preset is charted at a turn, and the preset turned k sixths is
  charted k sixths on
- **THEN** both are the same pieces on the same cells

### Requirement: A site is checked whole against the world

Before a site is set out it SHALL be checked whole against the world, the
first failure named: a five-sided cell under or beside it, ground more than
`max_footing` layers out of level, something in the way on a block a piece
takes (only that block, never its column), a piece already in one of its
slots, and build mode's rules against the world's own pieces.

#### Scenario: A tree in one cell

- **WHEN** a tree stands on the ground block of one cell of the site
- **THEN** the site is refused as "Something is in the way" on that cell's
  ground-storey pieces only, and a tree a storey higher refuses nothing

### Requirement: Changing a site marks what is built

A changed plan SHALL be diffed against the site's old plan by piece id:
pieces only in the new plan are new, unbuilt pieces it drops are gone, and
built pieces it drops are marked for removal, their goods coming back at
`refund_share`. A house built outside any site SHALL read back as the plan
that charts to its own pieces.

#### Scenario: Dropping a room half built

- **WHEN** a site's room is dropped from its plan after its floor and walls
  are built
- **THEN** its built pieces are marked, its unbuilt ones are gone, and the
  planks they cost come back

### Requirement: Blueprints and sites are stored records

A player's blueprint and a construction site SHALL each be a stored record
(`blueprint`, `building-site`) written through the durable path. Whether a
site's piece is built SHALL be derived from the piece records. A world
saved before the planner SHALL open unchanged and SHALL build as Free; a
world made since SHALL be Planned.

#### Scenario: A site across a restart

- **WHEN** a site and a blueprint are stored and the world is reopened
- **THEN** both read back as they were, and an old log's edits are untouched

#### Scenario: An old save

- **WHEN** a world whose identity has no building setting is opened
- **THEN** its building setting reads as Free
