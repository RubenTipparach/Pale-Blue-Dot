# Flight: a flyover of every kind of town

## ADDED Requirements

### Requirement: The town flyover visits one town of each built kind in one shot

`--route towns` SHALL fly the camera, without a cut, to one town of each kind
that has a template, chosen from the world's stored sites as the set and order
with the shortest flight, sweep an arc round each town looking at it, and end
by itself.

#### Scenario: The shipped seed

- **WHEN** `--route towns` is run in a new world of the shipped seed
- **THEN** the camera visits a harbour, a desert town, a village, a walled town and a tundra camp, in the order of the shortest tour, and the run ends when the last arc is done
