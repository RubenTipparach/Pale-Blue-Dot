## ADDED Requirements

### Requirement: A fish comes to a resting float

While a cast float rests on water, a school of a fish that lives at the float's spot SHALL spawn near the float after a random 10 to 30 s wait (`fauna.ron` `spawn.float_s`), drawn afresh for each school, whether or not the schools round the player are at their cap.

#### Scenario: A float in the sea draws a fish

- **WHEN** a float has rested on sea water where a biting species lives for 30 s
- **THEN** at least one school of that species has spawned within `spawn.float_m` of the float

#### Scenario: Water with no fish stays empty

- **WHEN** a float rests on water where no biting species lives
- **THEN** no school spawns for it
