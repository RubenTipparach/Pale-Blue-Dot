# Water environments: the shafts seen from under the water

## ADDED Requirements

### Requirement: Underwater light shafts are light in the water

The light shafts seen from under the water SHALL be light gathered in the
water along the view ray between the eye and the scene, sampled at the same
distances along every ray whatever the scene's distance, with the scene's
distance only ending the gathering. A shaft SHALL NOT draw the outline of the
scene behind it: across a step in the scene's depth, the light the shaft adds
SHALL change only by the light in the water between the two depths.

#### Scenario: A shaft crossing a pillar

- **WHEN** the eye is under shallow water by day and a shaft crosses a pillar standing in front of the far seabed
- **THEN** the shaft runs across the pillar's edge as one soft column of light
- **AND** no dark wedge or edge in the pillar's shape shows inside the shaft
