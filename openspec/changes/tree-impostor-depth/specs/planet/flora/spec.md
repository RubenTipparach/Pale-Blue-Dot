## ADDED Requirements

### Requirement: Far trees are lit by the near model's relief
A tree's mid cards and far billboard SHALL be lit per pixel with the normal
and root-to-tip shade of the near model at that pixel, baked from the near
model beside the impostor's colour, so that a far crown shows a sunlit side
and a shaded side as the near crown does. How much of that relief is taken
SHALL be a validated setting, 0..1, where zero is a single standing normal
and the crown's mean shade.

#### Scenario: A forested hillside against a low sun
- **WHEN** the camera looks over far trees toward a sun low in the sky
- **THEN** each crown is brighter on the side toward the sun than away from it
- **AND** the canopy is not one even tone

#### Scenario: The hand-off at the near ring
- **WHEN** the same trees are drawn as their near models and as their mid cards in one view
- **THEN** the mid cards' mean brightness is within a tenth of the near models'
