# Water Environments Specification

## Purpose
The water places of a body and how they look from inside: today, the light
under water (`pbd_core::ocean`, `water.wgsl`, `planet_surface.wgsl`). Where
each place occurs, tides and the places' cells are designed in
`openspec/changes/ocean-environments` and not built.

## Requirements

### Requirement: Daylight under water fades with depth
The light reaching a point under water SHALL fall with its depth below the
water's surface by a downwelling coefficient per colour, separate from the
view's absorption per metre of path. The view SHALL also scatter toward the
water's own colour by a turbidity read off the water under the eye (a
river's own; the sea's from the coast's in the shallows to the open ocean's
past the shelf), eased so it never steps.

#### Scenario: The zones fall inside the column
- **WHEN** the blue light left in ocean water is evaluated at 40 m and at
  90 m
- **THEN** it is 15% or more at 40 m (sunlit above) and 1.7% or less at
  90 m (midnight below), so every zone lies above `column::BASE_M`

#### Scenario: Red goes first
- **WHEN** the light left at 10 m is evaluated
- **THEN** red is under 2% while green and blue are above 50%

#### Scenario: A swim out never steps the turbidity
- **WHEN** the turbidity is read every 25 cm from the shore out past the
  shelf
- **THEN** no two readings differ by more than 0.002 per metre

### Requirement: The eye under any water sees the underwater view

When the eye is in water (the sea, a river, a pool, flowing water), the
underwater view SHALL be measured from the surface of the water the eye is
in: the depth fog and colour, the turbidity, the surface seen from below,
the light shafts and the distortion, as the sea gives them. The swell SHALL
be added only to the sea's surface.

#### Scenario: Down in a river

- **WHEN** the eye is under a river's water, above sea level
- **THEN** the view is fogged and tinted by the river's water from its own surface
- **AND** the river's sheet is drawn from below as the sea's is
