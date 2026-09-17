# Surface Rendering Specification

The hexagons-at-every-distance, resident-set, no-readback, gold-standard and
level-quantisation requirements moved into
`openspec/specs/planet/rendering/spec.md` with the commit that built them;
`planet::lod`, `planet::lattice` and the GPU visibility regression pin them.
What stays here is the seam, which only a picture settles.

## ADDED Requirements

### Requirement: A band boundary is not visible as a seam
Where two levels of detail meet, the surface SHALL remain closed: no crack to
space, no double-drawn ground, and no boundary that sweeps visibly across the
terrain as the camera moves.

#### Scenario: Crossing a level boundary
- **WHEN** the camera moves so that a region changes level
- **THEN** no gap opens between the two levels
- **AND** the change does not read as a line travelling over the ground
