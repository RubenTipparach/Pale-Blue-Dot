# Water Specification

## ADDED Requirements

### Requirement: The drawn sea never shows an alias of a wave
A vertex SHALL omit every component shorter than 2.5 times its own spacing to
its neighbours, fading it in over the next factor of 1.6. The physics SHALL
float on the full table, and the table's shortest component SHALL be at least
2.5 times the water cap's vertex spacing.

#### Scenario: The water cap
- **WHEN** the table is validated against the cap's 1.64 m corner spacing
- **THEN** no component shorter than 4.1 m exists
