## ADDED Requirements

### Requirement: The fish layer shows where a species can spawn now

The map SHALL have a fish layer. For the species chosen in the legend, it
SHALL show every place where a school of that species could spawn at this
moment, decided by the same function the spawner uses: the place's water
class, the depth, the live water temperature, and the freezing point. The map
SHALL NOT carry its own copy of that rule.

#### Scenario: The map and the spawner agree

- **WHEN** ten thousand seeded water points are checked for a species, both on
  the fish layer and by the spawner's rule, at the same moment
- **THEN** every point gets the same answer from both

#### Scenario: A river fish

- **WHEN** the banded perch is chosen
- **THEN** only river channels are marked, and no open sea

#### Scenario: Frozen water

- **WHEN** the water at a place is below its freezing point
- **THEN** no species is marked there

### Requirement: The fish layer shows a species' range through the year

For the chosen species, the fish layer SHALL also show, from the climate
normals:
- the water it can live in all year;
- the water it can live in for part of the year.

This SHALL follow the field-guide range maps' convention. The field guide's
entry for a species SHALL open the map on this layer for that species.

#### Scenario: Reef fish

- **WHEN** the reef fish is chosen
- **THEN** only shallows are marked, in either range, and no shelf, deep water
  or river

#### Scenario: From the field guide

- **WHEN** the player chooses "show on map" on a species' guide entry
- **THEN** the map opens on the fish layer with that species chosen

### Requirement: The climate layer shows the year, season by season

The map SHALL have a climate layer drawn from the climate normals. It SHALL
show, one at a time:
- the mean temperature;
- the yearly rain;
- the sea ice;
- the climate class.

A season control SHALL scrub the temperature and the ice through the year.
Where the normals are unavailable for this world, the layer SHALL say so
rather than draw anything.

#### Scenario: Winter ice

- **WHEN** the season control is set to the coldest season with the sea ice
  view on
- **THEN** the water that freezes in part of the year is marked frozen

#### Scenario: Unavailable

- **WHEN** the climate asset was made for another seed or generator version
- **THEN** the climate layer is greyed out in the legend, with the reason
