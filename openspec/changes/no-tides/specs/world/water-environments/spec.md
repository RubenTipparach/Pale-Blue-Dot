## RENAMED Requirements

- FROM: `### Requirement: The composite's waterline is the tided sea for an eye in air`
- TO: `### Requirement: The composite's waterline is the sea's surface for an eye in air`

## MODIFIED Requirements

### Requirement: The composite's waterline is the sea's surface for an eye in air

The composite SHALL measure the waterline from the surface of the water the
eye is in, and for an eye in air from the sea's drawn surface (the sheet,
half a metre under the sea's level), never a sphere of its own, so a jump or
a flight over a beach tints and wobbles none of the dry sand over the water
(`ocean-environments` B21). The sea's surface SHALL stand at one level: it
SHALL NOT rise and fall with the moon or the time of day (`no-tides`).

#### Scenario: A jump on a beach at low tide

- **WHEN** the eye rises from standing on a beach over the sea's water into air over it
- **THEN** the composite's waterline stays at the sea's drawn surface
  (`planet::water::tests::a_jump_over_a_beach_keeps_the_composites_waterline_at_the_sea`)
