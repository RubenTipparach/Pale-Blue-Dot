## ADDED Requirements

### Requirement: The cog is a sailing craft you walk on
The cog SHALL be a craft whose hull floats by its cells, whose square sail is
one foil on a yard, and whose keel and rudder are wet foils, like the Tern's.
Its deck, castles, stair, rail and gangway SHALL be walkable while it sails.
F at its tiller SHALL take the helm, and F again SHALL let go, leaving the
ship sailing with its yard and tiller as they were.

#### Scenario: A reach
- **WHEN** the helmsman braces the yard across a beam wind
- **THEN** the cog gathers way, and its heel stays under 15°

#### Scenario: Letting go of the helm
- **WHEN** the helmsman presses F at the tiller while the cog sails
- **THEN** the walker is on foot on the deck, and the cog sails on with its
  yard and tiller unchanged

### Requirement: A walker on a deck is saved on the deck
A walker who quits while standing on a craft's deck SHALL be saved with that
craft's ID and a position in its frame, and SHALL load on the deck at that
position, wherever the craft came to rest.

#### Scenario: Quitting on the aftcastle
- **WHEN** the player quits standing on the cog's aftcastle, and the world is
  reloaded
- **THEN** the player stands on the aftcastle, and the cog is where the save
  left it
