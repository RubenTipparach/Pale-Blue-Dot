## MODIFIED Requirements

### Requirement: Relief is authored for a walker
Terrain relief SHALL be authored so that a mountain is something a walker
climbs in one-metre steps rather than scenery: the summits land in the band
the owner chose (survey H1: 250 to 290 m, under the cloud base),
the ranges rising in regions of their own and wide enough to walk up, and the
ocean floor 60 to 145 m below the sea, with the seeded coastline unchanged. A
test SHALL pin the range.

#### Scenario: Measuring the relief
- **WHEN** `surface_height` is sampled evenly over the sphere
- **THEN** the highest point lies in the chosen band (250 m to 290 m)
- **AND** the lowest lies between 60 m and 145 m below the sea
