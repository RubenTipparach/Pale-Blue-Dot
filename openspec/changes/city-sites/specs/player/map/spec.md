## ADDED Requirements

### Requirement: The map shows every settlement site

The map SHALL show every site on the planet as a marker for its kind, with
its name. Towns SHALL be labelled at every zoom, and villages and camps from
a closer zoom. At close zoom each site's footprint SHALL be outlined. The
sites SHALL be one layer in the legend.

#### Scenario: The whole planet

- **WHEN** the map is zoomed all the way out with the sites layer on
- **THEN** every town is marked and named, and the other sites are marked

#### Scenario: Close in

- **WHEN** the map is zoomed in on a site
- **THEN** its name, its kind and its footprint's outline are shown

#### Scenario: The map and the list agree

- **WHEN** a site's marker is placed on the map
- **THEN** it is at the anchor the site list gives, through the same
  projection as the player's marker
