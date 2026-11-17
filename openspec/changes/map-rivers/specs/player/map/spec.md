## ADDED Requirements

### Requirement: The map draws the rivers' water

The base map SHALL draw a river's water in the map's water colour wherever
the terrain's column holds river water, at the width that water has in the
world, using the same river test as the column (`rivers::floods`, none in a
town's footprint). A pixel that the water covers in part SHALL be painted in
part. The sea SHALL be drawn as it is without rivers.

#### Scenario: A river on the map

- **WHEN** the map is drawn over a place where the column holds a river's
  water above the sea's level
- **THEN** that place is drawn in the map's water colour, not as the ground
  under the water

#### Scenario: Dry land and the sea

- **WHEN** the map is drawn over dry land away from a river, or over the sea
- **THEN** it is drawn exactly as it was before rivers were filled
