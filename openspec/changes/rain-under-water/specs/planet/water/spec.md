## ADDED Requirements

### Requirement: The rain volume is in the air, never in the water
The rain volume SHALL be gathered only along air. From a camera under the sea
a pixel's rain SHALL start where its ray leaves the water, on the refracted
direction, and SHALL be weighed by the Fresnel transmission at the surface
and the water's absorption over the path to it. A ray outside Snell's window,
a ray that does not rise to the surface, and a ray that meets something in
the water first SHALL gather no rain. Above the sea the rain SHALL stop at
the sea surface and at the scene's depth.

#### Scenario: Under water in rain
- **WHEN** the camera is 5.4 m under the sea in rain with the rain volume on
- **THEN** the view outside Snell's window is the same as with the rain volume off
- **AND** nothing in the water in front of the surface is fogged by the rain

#### Scenario: Looking out of the window
- **WHEN** a ray from a camera under the sea leaves the water inside Snell's window
- **THEN** the rain it gathers is laid over the frame at no more than the surface's transmission times the water's transmittance along that ray
