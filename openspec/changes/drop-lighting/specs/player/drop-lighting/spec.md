## ADDED Requirements

### Requirement: A dropped item is lit like the ground at its place

A dropped item floating in the world SHALL be lit by the terrain's own light
at its place: the terrain's sun with its shadows, and the cell field's sky and
lamp light.

#### Scenario: A block dug from a pit

- **WHEN** a sand block is dug from the bottom of a shaded pit by day
- **THEN** its drop is about as bright as the shaded sand around it, not white
