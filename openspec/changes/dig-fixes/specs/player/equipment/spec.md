## ADDED Requirements

### Requirement: A felled tree does not take the dig

The use button SHALL work a tree only while it stands. Once a tree has given
all its wood it SHALL NOT catch the aim, and the ground where it stood and
past it SHALL dig as any other ground.

#### Scenario: Digging where a tree was felled

- **WHEN** a player holds the use button with a digging tool on the ground at the foot of a felled tree
- **THEN** that ground breaks and is taken, and no "No more wood" note shows
