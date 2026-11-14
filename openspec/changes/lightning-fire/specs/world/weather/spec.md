## ADDED Requirements

### Requirement: A lightning strike lands on the tallest point near it
A lightning strike within sight of the player SHALL come down on the highest
solid top within 24 m of where the atmosphere put it, never on water. The
landing point SHALL be a deterministic function of the strike and the
terrain.

#### Scenario: A tree beside open ground
- **WHEN** a strike comes down 10 m from a tree on flat grass
- **THEN** the bolt ends at the tree's crown

#### Scenario: Water
- **WHEN** a strike comes down over a lake with a shore inside 24 m
- **THEN** the bolt ends on the shore, not on the water

### Requirement: A strike starts a short fire that the rain puts out
A strike that lands on wood, leaves or plants SHALL start a fire there that
lights its surroundings and goes out after a time set by the rain: 3 s plus
or minus 1 s in the heaviest rain, longer in light rain, at most 12 s. A strike on
stone, sand or snow SHALL make only the impact burst. A fire SHALL change
nothing in the world and save nothing.

#### Scenario: A tree in heavy rain
- **WHEN** a strike hits a tree's crown in heavy rain
- **THEN** the crown burns and lights the ground round it, and the fire is out within 5 s

#### Scenario: Nothing is kept
- **WHEN** the fire has gone out and the world is reopened
- **THEN** the tree and the ground are as they were before the strike
