# World: cloud lighting

## MODIFIED Requirements

### Requirement: Clouds are a lit slab with a thickness
The cloud layer SHALL be integrated through a shell of non-zero depth rather
than sampled at a single radius, accumulating transmittance and lighting each
sample toward the sun, so that a mass is brighter on top than underneath and a
grazing view crosses more of it than a vertical one. The light reaching a
sample SHALL be attenuated with the same extinction the view uses, and the
darkness of a cloud's underside SHALL follow from the cloud standing over it
rather than from a configured darkness.

#### Scenario: An overcast sky from below
- **WHEN** a player stands under full cover and looks up and then at the horizon
- **THEN** the horizon is more opaque than the zenith
- **AND** the underside of the cover is darker than its sunlit top

#### Scenario: A sunlit top is white
- **WHEN** a fair-weather cloud is seen from above with the sun 60 degrees high
- **THEN** its top is at least 85% as bright as an unshadowed cloud sample

#### Scenario: A thicker cloud has a darker base
- **WHEN** two clouds under the same sun differ only in how dense they are
- **THEN** the denser one has the darker base
- **AND** a storm's base is at most 15% as bright as its top
