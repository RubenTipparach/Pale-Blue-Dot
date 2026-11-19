## MODIFIED Requirements

### Requirement: The line runs from the rod tip to the float
While the float is out, the drawn line SHALL start at the rod's tip and end
at the drawn float, each frame, from the same frame's camera and float. The
points between SHALL lie on or over the water the float rides, its waves
included, not through it.

#### Scenario: A wave trough and a bite
- **WHEN** the float rides a wave trough, or is pulled under on a bite
- **THEN** the line's last point is the float and no point between is under
  the water (`fish::tests::the_line_runs_from_the_rod_tip_to_the_float`)
