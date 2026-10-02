# Flight: a flyover of every kind of town

## ADDED Requirements

### Requirement: The town flyover visits one town of each built kind in one shot

`--route towns` SHALL fly the camera, without a cut or a jump, to one town of
each kind that has a template, chosen from the world's stored sites as the set
and order with the shortest flight; pass through each town at rooftop height;
climb into the sky between towns and glide down into the next; and end by
itself. Its camera SHALL never lose its roll when it looks steeply down or
up.

#### Scenario: The shipped seed

- **WHEN** `--route towns` is run in a new world of the shipped seed
- **THEN** the camera visits a harbour, a desert town, a village, a walled town, a tundra camp and a jungle village, in the order of the shortest tour, passing through each at rooftop height and climbing between them, and the run ends when the last pass is done

#### Scenario: Looking straight down

- **WHEN** the wanted look turns through straight down over a town
- **THEN** the camera's roll changes by no more than its eased turn allows in any frame
