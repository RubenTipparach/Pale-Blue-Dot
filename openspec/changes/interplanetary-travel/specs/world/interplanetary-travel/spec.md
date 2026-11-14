# Interplanetary Travel Specification

## ADDED Requirements

### Requirement: Aircraft fly only in air
An aircraft (the Kestrel, the Tern, any assisted craft without a rocket stage)
SHALL fly only inside a body's air and SHALL NOT reach orbit or another body.
No craft SHALL have thrust that costs nothing or a field that cancels a body's
gravity.

#### Scenario: The Kestrel climbs as high as it can
- **WHEN** the Kestrel climbs over Tenebris at full power
- **THEN** it stays inside Tenebris's air and is never in orbit

### Requirement: Leaving a world takes a rocket of the right tier
A trip from one body to another SHALL be made by a rocket or a ship of the
ship tiers (chemical, nuclear thermal, fusion, antimatter): a climb to orbit on
thrust greater than its weight (from a pad at tier 1), a departure burn, a
coast, a capture and a landing or a docking. Every burn SHALL spend propellant
the craft carries, by the rocket equation.

#### Scenario: A stack without the delta-v
- **WHEN** a stack whose delta-v is less than the climb to Tenebris's low orbit is launched
- **THEN** it does not reach orbit

#### Scenario: A fusion ship at a planet's surface
- **WHEN** a fusion ship, whose thrust is less than its weight at 1 g, is asked to lift off from Tenebris
- **THEN** it cannot, and a lander makes the trip between the surface and orbit

### Requirement: Landing uses the air or the engine
A craft landing on a body with air SHALL be able to shed orbital speed on a
heat shield and chutes. A craft landing on an airless body SHALL get no
aerodynamic braking and SHALL land by a powered descent; it is landed only
below the landed speed limit.

#### Scenario: Coming down on Lantern
- **WHEN** a lander descends from low orbit onto Lantern, which has no air
- **THEN** all of its speed is taken off by its engine, and it is landed only if it touches down under the limit

### Requirement: Gas giants are orbited, never landed on
A craft SHALL be able to orbit a gas giant, land on its moons, or drop an
aerostat into its airship band. No craft SHALL land on a gas giant.

#### Scenario: An aerostat drop into Amber
- **WHEN** an aerostat is dropped into Amber on a heat shield
- **THEN** it floats in the airship band and never reaches a surface

### Requirement: Stations orbit
A station in space SHALL ride a rail that agrees with the field round its body
(circular speed at its radius), and SHALL NOT hang still over a point on a
turning body.

#### Scenario: Highport over Tenebris
- **WHEN** Highport is placed 2,400 m over Tenebris
- **THEN** it circles at Tenebris's circular speed for that radius
