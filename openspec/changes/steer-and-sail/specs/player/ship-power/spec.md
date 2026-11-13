# Player: ship power

## Purpose

The six levels of ship propulsion as blocks of the hex ship builder.
Proposed with the mockups; nothing here is built.

## ADDED Requirements

### Requirement: A ship's power is blocks with their fuel aboard
Rowing benches, sails, steam boilers and turbines, oil engines, fission
reactors, and battery banks with electric motors SHALL be blocks of the hex
ship builder. An engine SHALL refuse to run without its fuel store aboard,
and the builder SHALL show the top speed and the range on the fuel aboard.

#### Scenario: An oil engine with no tank
- **WHEN** a design has an oil engine and no oil tank
- **THEN** the builder refuses it with "the oil engine has no fuel aboard"

### Requirement: Top speed follows from power and resistance
A powered ship's top speed SHALL be where its delivered power equals its
resistance times its speed, the resistance growing steeply past hull speed.

#### Scenario: Power past hull speed buys little
- **WHEN** a hull already at 0.95 of its hull speed doubles its power
- **THEN** its top speed rises by less than 10%
