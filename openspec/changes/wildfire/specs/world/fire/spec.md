# Spec Delta

## Purpose

Fire in a Survival world: how dry each place and each plant is, small
forest fires that start in the dry season, spread through what is dry and stop in rain, water and bare
ground, what they leave and how it grows back, and torches that the rain
puts out and that light things.

## ADDED Requirements

### Requirement: Forest fires start only in the dry season, in Survival
In a Survival world, a lit torch used on a dry enough plant, or a lightning
strike from a dry storm on a very dry plant, SHALL start a fire, with
lightning starting at most one in a quiet spell over the same ground. A
placed campfire, brazier, torch or lantern SHALL never start one. In
Explorer there SHALL be no forest fires.

#### Scenario: A torch on dry grass
- **WHEN** a Survival player uses a lit torch on grass in late summer after
  dry weeks
- **THEN** that cell burns, and the fire may spread to its dry neighbours

