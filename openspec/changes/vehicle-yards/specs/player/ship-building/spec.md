# Player: ship building

## Purpose

How a player builds an ocean ship out of the world's own hex cells in a
drydock, floats it out, and walks its decks while it sails. Proposed with the
M23 mockup; nothing here is built.

## ADDED Requirements

### Requirement: A ship is made of the world's cells, in its own frame

An ocean ship's blocks SHALL be cells of the world's size: 2.833 m flat to
flat and 1 m a layer. They SHALL be keyed `(q, r, layer)` on a flat axial grid
in the ship's own frame, with the bow along `+q`. Its fittings (rails, masts,
helm, ladders, lanterns) SHALL be the town's pieces, placed on its cell edges
and in its cells. A ship SHALL be at least two cells in the beam.

#### Scenario: A ship's cell is a ground cell's size

- **WHEN** a ship's hull cell is measured flat to flat
- **THEN** it is 2.833 m, the same as a cell of the ground it sails past

#### Scenario: A one-cell-wide hull is refused

- **WHEN** the builder is given a hull one cell wide
- **THEN** the check fails with "a ship is at least two cells in the beam"

### Requirement: A hex ship floats by its sealed cells

A hex ship SHALL float by `hull::float`, with each hex cell sampled as seven
sub-cells so that a buoyancy cell is at most a third of the beam. Hull, keel,
ballast and bulkhead cells SHALL keep the sea out. A hold or cabin cell SHALL
be buoyant only when it is enclosed below the waterline and decked over. An
open cell below the waterline SHALL flood its compartment through the bilge
model.

#### Scenario: The solved draft is where it floats

- **WHEN** a ship's waterline is solved from its cells and it is set on calm
  sea
- **THEN** it settles within 0.1 m of that draft

#### Scenario: A breach floods

- **WHEN** a hull cell below the waterline is broken
- **THEN** the compartment behind it fills, and the ship sits lower and lists
  toward the breach

### Requirement: The builder's checks keep a ship sailable and walkable

The builder SHALL refuse to float out a design unless every check passes, and
it SHALL name the failing one. The checks are:
- a helm on the main deck;
- a rudder below the waterline, aft;
- every mast stepped on a keel cell;
- it floats with at least 1 m of freeboard;
- its metacentric height is above zero;
- every cell is joined to the rest;
- a walker can walk from the gangway to the helm over walkable floors, steps
  of one layer, and ladders.

#### Scenario: An unreachable helm is refused

- **WHEN** the helm stands on a deck with no ladder or stair to it
- **THEN** the check fails with "the helm cannot be walked to from the
  gangway"

#### Scenario: A top-heavy design is refused

- **WHEN** a design's centre of mass is above its metacentre
- **THEN** the check fails with its GM in metres and "capsizes"

### Requirement: A drydock builds a ship dry and floats it out

A drydock SHALL be a basin of world cells below sea level, closed by a gate
and drained by a pump. A ship SHALL be built in it on keel blocks while it is
dry. Flooding it SHALL float the ship off its blocks at its draft, and opening
the gate SHALL let it out. The dock's size SHALL cap the ship's length, beam
and depth.

#### Scenario: The ship lifts at its draft

- **WHEN** a dry dock with a finished ship is flooded
- **THEN** the ship stays on its blocks until the water reaches its draft, and
  then floats

#### Scenario: A ship too big for the dock is refused

- **WHEN** a design is longer than the dock
- **THEN** the builder refuses it with the dock's length and the design's

### Requirement: A player walks a hex ship's decks under way

A walker SHALL stand on a hex ship's walkable cells in the ship's frame, as on
the cog's deck. They SHALL stay on the deck through a turn and under way, and
climb its ladders and stairs.

#### Scenario: Standing through a turn

- **WHEN** a walker stands on the main deck while the ship turns at its
  fastest rate
- **THEN** the walker stays on the same deck cell

### Requirement: A ship is a stored record that never changes shape by itself

A ship SHALL be a stored record. The record SHALL hold:
- its stable ID;
- its cells with their blocks, and its pieces;
- the block set's version;
- its pose in a named frame;
- its hold and its flooding.

Placing or breaking a ship's block SHALL be an edit in the ship's frame
through the durable path, naming its author. A block retuned after a ship was
built SHALL keep the ship's numbers, or migrate them on open, and never change
its cells.

#### Scenario: A built ship survives a block retune

- **WHEN** a hull block's mass is retuned in a later version and an old world
  is opened
- **THEN** the ship has the same cells, and it floats as it did
