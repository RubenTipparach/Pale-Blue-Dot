# Terrain: a landing's dissolve keeps the ground whole

## ADDED Requirements

### Requirement: A landing's dissolve never shows the sky through the ground

While a landing dissolves from the old partition to the new one, the new
ground SHALL stand aside for the old in a pixel only where the records hold
every detail level the old partition can draw there, including both levels
across a band's cross-fade ring; elsewhere the new ground SHALL keep the
pixel.

#### Scenario: Fast flight past a band ring

- **WHEN** a fine set lands after the camera has moved further than the rebuild margin, and the old partition's band ring lies where the new records lack its coarser level
- **THEN** that ring is drawn from the new partition alone, and no pixel there shows the sky through the ground
