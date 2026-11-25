# Planet Viewer Specification

## ADDED Requirements

### Requirement: The viewer runs the game's own generator
The planet viewer SHALL compute every height, biome and top material it
shows by calling `pbd-core`, compiled to WebAssembly from the same commit,
and SHALL NOT carry a second implementation of any world rule.

#### Scenario: A generator change shows in the viewer
- **WHEN** a generator's config changes in `pbd-core` and the viewer is rebuilt
- **THEN** the viewer's heights at a fixed set of directions match the native build's, value for value

### Requirement: Sizes only on the ladder
The viewer SHALL offer body sizes only on the ladder `R = 300 m * 2^(L - 7)`,
and SHALL draw every cell at 2.833 m across and 1 m tall at every size.

#### Scenario: Picking a rung
- **WHEN** the viewer is set to L9
- **THEN** it shows a 1,200 m body, and its Ground tab shows cells of the same width as at L11

### Requirement: The viewer says what a size costs and whether saves can key it
For each rung the viewer SHALL show the radius, the surface cell count and
whether the game's cell key can address that level today.

#### Scenario: A rung the key cannot address
- **WHEN** the viewer is set to L7 or L12
- **THEN** its size card says the cell key cannot save edits at that level yet
