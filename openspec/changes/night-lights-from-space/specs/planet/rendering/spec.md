## ADDED Requirements

### Requirement: Settlements are lit points on the night side from afar
Every site on the world's saved site list SHALL be drawn, from afar, as a warm
cluster of lit points over a soft glow, sized and brightened by its kind. The
planet pass SHALL add them before the ground's haze, so clouds over the ground
hide them. Their strength SHALL be the sky's night term at the site, and SHALL
fade out as the camera nears the site, where the town itself stands.

#### Scenario: A town at night from orbit
- **WHEN** a site is on the night side and the camera is kilometres from it
- **THEN** its far light is drawn with its kind's full brightness

#### Scenario: The same town by day
- **WHEN** the sun stands over the site
- **THEN** its far light's strength is nought and nothing is drawn

#### Scenario: Close to the town
- **WHEN** the camera is within the distance at which the town stands
- **THEN** its far light's strength is nought, and only the town's own lamps
  light it

### Requirement: The player's lights show from afar past a threshold
The player's light blocks SHALL be grouped by distance, and a group of at
least the threshold count SHALL be a far light of its own, brightened by its
count, drawn as a settlement's is. The groups SHALL be derived from the saved
edits, never saved themselves, independent of the order the edits were made
in, and rebuilt when the edits change.

#### Scenario: A base below the threshold
- **WHEN** the player has placed fewer lights in one place than the threshold
- **THEN** no far light stands there

#### Scenario: A base past the threshold
- **WHEN** the player places one more light and the group reaches the threshold
- **THEN** a far light stands at the group's centre
