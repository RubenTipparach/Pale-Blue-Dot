## ADDED Requirements

### Requirement: The player can read where the camera is
The HUD's debug line SHALL show the active camera's latitude and longitude as
`--at` takes them, the map's reading of the same place, its elevation above
sea level and its height above the ground. The log SHALL carry the same at
spawn, at `READY` and with each capture.

#### Scenario: A screenshot can be taken again
- **WHEN** a capture is saved
- **THEN** the log names the capture's latitude and longitude, elevation and height above ground
- **AND** launching with `--at` at that latitude and longitude puts the walker on the same spot
