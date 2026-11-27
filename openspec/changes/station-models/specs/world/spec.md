## ADDED Requirements

### Requirement: A chest and a workbench are drawn as models
A placed chest or workbench SHALL be drawn as a model standing in its cell,
not as a hex of terrain. The cell SHALL stay solid: walked on, blocking
light, aimed at, dug and saved as before.

#### Scenario: The terrain does not draw a chest's cell
- **WHEN** a column holds a chest on its top layer
- **THEN** its drawn runs end below the chest, and the chest's layer is still solid
