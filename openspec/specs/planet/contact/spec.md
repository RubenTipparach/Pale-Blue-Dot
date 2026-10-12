# Surface Contact Specification

## Purpose
An actor standing on the ground must stand on the ground that is drawn. Contact
queries resolve against the exact triangles the surface pass builds, so there is
no second, smoother, invisible terrain underneath the visible one.

## Requirements

### Requirement: Contact matches the rendered geometry
Surface contact SHALL resolve against the same cap triangles the renderer
builds, including on pentagons, rather than against an interpolated height
field.

#### Scenario: Querying any cell
- **WHEN** contact is queried inside a hexagonal or pentagonal cell
- **THEN** the height returned is the height of the actual uploaded fan
  triangle at that point

### Requirement: Steps are reported as steps
A boundary between two cells at different heights SHALL report the real step
height. It SHALL NOT be smoothed into a ramp by interpolation.

#### Scenario: Standing at a raised cell boundary
- **WHEN** contact is queried across the edge between a cell and a higher
  neighbour
- **THEN** the reported height changes by the full step
- **AND** no intermediate interpolated height is reported

### Requirement: Contact accuracy holds at production resolution
Contact SHALL keep millimetre accuracy at the shipped subdivision level, and its
acceleration index SHALL stay valid at seams, at the poles and where water
meets land.

#### Scenario: The shipped globe
- **WHEN** contact is queried across the production-resolution planet
- **THEN** the result is accurate to the millimetre
- **AND** seam, pole and water-spawn lookups all resolve to valid cells
