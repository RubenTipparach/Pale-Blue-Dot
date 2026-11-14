# Industry Specification

## ADDED Requirements

### Requirement: Only marked cells are dug, by whoever digs them
A crew, an excavator, a bucket-wheel excavator or a drone SHALL dig only cells
the player has marked, and a quarry frame only the cells inside it. Every dug
cell SHALL be an edit on the durable save path naming its author.

#### Scenario: A crew beside an unmarked cell
- **WHEN** a hired crew works a pit whose next layer has one unmarked cell
- **THEN** that cell is left in place and every other cell dug is saved as an edit naming the crew

### Requirement: An offshore platform is a rig and a power plant
An offshore platform SHALL draw crude from its wells and SHALL burn their gas
in its own turbines, exporting its surplus power to a land grid only through a
built sea cable.

#### Scenario: A platform with no sea cable
- **WHEN** a platform produces 4,000 kW and uses 900 kW with no sea cable built
- **THEN** no shore network receives its power and its crude still fills its tanks

### Requirement: Each carrier network carries one kind of thing
A pipe network SHALL carry one fluid at a time, a wire network electricity, a
belt bulk items and a pneumatic tube network addressed capsules of small
items.

#### Scenario: Two fluids into one pipe
- **WHEN** a pipe network holding diesel is joined to a water tank without a valve
- **THEN** the join is refused and names the fluid already in the network

### Requirement: Freighters are hex-built and carry hex pods

A freighter's hull SHALL be built of whole hex cells in the drydock builder.
Its cargo SHALL travel in hex pods, each one cell across and 2 layers tall.
A goods pod SHALL hold up to 10 t of stacks, and a tank pod SHALL hold up to
10 t of one fluid. Each pod SHALL be a stored record that keeps its contents
when it moves between ship, quay and crane.

#### Scenario: Loading a pod

- **WHEN** a quay crane lifts a goods pod from the pod yard onto a free deck
  cell of a moored freighter
- **THEN** the pod locks to that cell and its record names the freighter as
  its carrier, and the freighter's builder check counts its mass

