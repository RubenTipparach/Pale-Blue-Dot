## ADDED Requirements

### Requirement: Stars keep a steady size and brightness
Every star SHALL be drawn at least two pixels across with a soft falloff,
sized in screen pixels rather than metres, so that it neither vanishes nor
flickers as the sky turns, the camera moves or the render scale changes.

#### Scenario: A still night sky
- **WHEN** the walker stands still under a clear night sky
- **THEN** each star stays at the same brightness from frame to frame
