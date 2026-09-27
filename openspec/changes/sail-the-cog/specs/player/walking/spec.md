## ADDED Requirements

### Requirement: A walker stands and walks on a moving deck
A walker on a craft's deck SHALL be simulated in the craft's frame. It SHALL
be carried by the craft's motion, turning included, and SHALL walk, climb
stairs and meet rails as it does on the ground, with no slide or jitter from
the craft's motion. Leaving the deck SHALL return it to the planet's frame at
its composed velocity.

#### Scenario: Standing still through a turn
- **WHEN** a walker stands on the cog's deck, 6 m from its mast, while the cog
  turns through 90°
- **THEN** the walker's position in the ship's frame moves less than 1 cm

#### Scenario: Up the stair under way
- **WHEN** a walker climbs the stair to the aftcastle while the cog sails
- **THEN** there is no eye jump over 0.1 m and no airborne tick, as on a stair
  ashore

#### Scenario: Over the side
- **WHEN** a walker steps off the rail while the cog sails
- **THEN** it falls into the sea with the ship's velocity at that point, and
  the cog sails on
