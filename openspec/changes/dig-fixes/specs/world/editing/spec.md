## ADDED Requirements

### Requirement: Bait is dug from earth by the water

Digging grass, soil, dirt or sand SHALL turn up bait in a share of cells
(`bait_one_in`) when a water cell, of a river or the sea, stands within
`bait_reach_cells` cells of the dug cell and within `bait_water_m` of its
layer. It SHALL NOT turn up from a block a player placed, nor from ground away
from water. The nearness SHALL be one rule in `pbd_core::gather`, read on the
column tier's water cells.

#### Scenario: A river bank inland

- **WHEN** a player digs earth beside a river's water, far above the sea
- **THEN** about a third of such cells give bait

#### Scenario: The shore

- **WHEN** a player digs earth or sand beside the sea
- **THEN** about a third of such cells give bait

#### Scenario: Away from water, or a placed block

- **WHEN** a player digs earth more than a few cells from any water, or a block they placed
- **THEN** no bait is given

#### Scenario: The mission counts it

- **WHEN** three baits are dug while "Dig for bait" is open
- **THEN** its "dig bait" objective is met
