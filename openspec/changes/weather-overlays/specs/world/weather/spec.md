# World: weather overlays

## ADDED Requirements

### Requirement: The weather can be shown
The player SHALL be able to show, one at a time over the planet, the surface
wind, the cloud-level wind, the ocean current, cloud cover, precipitation,
humidity, sunlight at the ground and temperature, with a legend giving the
units. Each overlay SHALL read the same simulated state the clouds are drawn
from.

#### Scenario: Wind from orbit
- **WHEN** the wind overlay is shown from orbit
- **THEN** streamlines follow the simulated wind, coloured by its speed

#### Scenario: Off by default
- **WHEN** a world opens
- **THEN** no overlay is drawn
