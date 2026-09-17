# Water Specification

## ADDED Requirements

### Requirement: The camera's side of the surface is one of three states
The renderer SHALL classify every view as dry, straddling or under from the
camera's body-local radius against the sea radius and whether its direction is
over water. The straddling band SHALL be at least as wide as the wave
amplitude, so the wave function is not evaluated a second time on the CPU.

#### Scenario: Diving through the surface
- **WHEN** a camera descends from above the sea through the surface band
- **THEN** the state passes dry, straddling, under in that order
- **AND** an emerge window arms when it comes back up and expires 2.6 s later

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
