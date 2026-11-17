## ADDED Requirements

### Requirement: A tool's strikes land with the dig

While a block is being broken, the tool in hand SHALL strike in strokes timed
to the dig's progress, the last impact on the break, each stroke fastest at
the impact.

#### Scenario: Chopping wood with the stone axe

- **WHEN** the stone axe breaks wood that takes 0.6 s
- **THEN** it strikes twice, the second impact when the block breaks
