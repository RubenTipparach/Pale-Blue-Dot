# Water Specification

## ADDED Requirements

### Requirement: Underwater pixels are fogged by their water travel
A view that is under or straddling SHALL fog geometry pixels by the distance
the ray travels through water, and SHALL fog sky pixels by the analytic exit
distance off the mean sea sphere, saturating for rays that never surface.
Pixels above the waterline SHALL stay clear.

#### Scenario: Looking along the surface from just under it
- **WHEN** the eye is a metre under and looks toward the horizon
- **THEN** the far water saturates to the deep colour rather than showing the
  sky through a bright band

### Requirement: The lens is wet after surfacing
The renderer SHALL draw refracting droplets on the lens while rain is
non-zero and for a fixed dry-off after the camera leaves the water, masked to
the part of the view above the waterline while straddling and skipped while
under.

#### Scenario: Surfacing in clear weather
- **WHEN** the camera rises out of the water with no rain
- **THEN** drips run down the lens and fade over the dry-off
- **AND** no drop is drawn on the part of the view that is still underwater
