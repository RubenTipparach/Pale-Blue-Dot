# Player: small boats

## Purpose

Boats the player makes, launches and takes up; the raft's hold; and fishing
from a paddled boat.

## ADDED Requirements

### Requirement: Boats up to a small rowboat are crafted

The recipe table SHALL hold an inner tube, a raft and a rowboat, each made
at the workbench. Each SHALL give its boat as a good, one to a
slot.

#### Scenario: Making an inner tube

- **WHEN** a player with four wool and two rope crafts INNER TUBE at a workbench
- **THEN** the wool and rope are taken and an inner tube lands in the slots

### Requirement: The tube and the raft float upright and paddle

The inner tube and the raft SHALL be craft of their own, stepped by the
paddled-boat model, each floating upright with its paddler aboard and moving
ahead under a forward stroke.

#### Scenario: A raft at rest

- **WHEN** a raft with its paddler aboard is left on calm water
- **THEN** it settles floating, upright, with its deck above the water

### Requirement: A boat good launches on water and a made boat is taken up

Placing a boat good on water in reach SHALL put that craft there as a new
record marked as made. A made craft with nobody aboard and an empty hold SHALL
be taken back into the slots. Harbour boats and a world's first boats SHALL
NOT be taken up.

#### Scenario: Launch and take up

- **WHEN** a player launches a raft and then takes it up
- **THEN** the raft good is back in the slots and the craft is gone from the fleet

### Requirement: A raft hauls goods in a hold that is saved with the slots

A raft SHALL have a hold of ten slots. Moving a stack between the hold and the
slots SHALL be one durable line holding both. A raft whose hold is not empty
SHALL NOT be taken up.

#### Scenario: A hold survives a reload

- **WHEN** a player moves logs into a raft's hold and the world is reopened
- **THEN** the hold still holds the logs and the slots do not

### Requirement: The rod fishes from a paddled boat

Aboard an inner tube, a raft, a canoe or a rowboat, the rod SHALL cast, hook
and reel by the same rules as on foot, with the line leaving from the seat.

#### Scenario: A cast from a raft

- **WHEN** a player aboard a raft with the rod in hand casts
- **THEN** the float flies from the seat and lands on the water ahead
