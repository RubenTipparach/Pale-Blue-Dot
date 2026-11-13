## ADDED Requirements

### Requirement: The sky and the ground share one twilight
Seen from the ground, the sky's sunlit glow SHALL fade with the sun's
elevation over the same range as the ground's sky light, and SHALL be gone
when the ground's twilight is gone. No part of the sky seen from the ground
SHALL stay sunlit while the ground is lit as night.

#### Scenario: Deep night
- **WHEN** the walker stands on the ground with the sun 30 degrees under the horizon and a clear sky
- **THEN** no part of the sky glows with sunlight; only the stars and the moon light it

#### Scenario: Dusk
- **WHEN** the sun is 5 degrees under the horizon
- **THEN** the sky on the sun's side glows and the ground is lit by the same twilight
- **AND** no hard edge of the planet's shadow crosses the sky
