# Fishing Specification

## ADDED Requirements

### Requirement: A cast flies under the planet's gravity and floats on the drawn sea
With the rod in hand, holding and releasing the use button SHALL throw the
bobber from the rod tip at a speed set by how long it was held, under the
planet's own gravity. A bobber that reaches water SHALL float on the same sea
height the craft float on. A bobber that lands on ground, or is still flying
after a fixed time, SHALL come back empty.

#### Scenario: Casting onto the bank
- **WHEN** the bobber meets ground before water
- **THEN** the line comes back with nothing on it

#### Scenario: Riding a swell
- **WHEN** the bobber floats in a sea with waves
- **THEN** its height follows `pbd_core::sea` at its position, as a hull's
  does

### Requirement: The fish that bites is a fish that was swimming there
A bite SHALL come from a fish of a school simulated near the bobber, and the
species caught SHALL be that fish's species. There SHALL be no catch table
that is independent of the fish in the water.

#### Scenario: No school near
- **WHEN** no school is within its sense range of the bobber
- **THEN** no bite comes

#### Scenario: A school scents the lure
- **WHEN** a school is within three times its sense range of a floating
  bobber
- **THEN** it turns toward the lure

### Requirement: A bite opens a hook window that shortens with the fish's strength
A bite SHALL open a window of `0.9 * (1 - (strength - 1) * 0.13)` seconds,
never under 0.25 s. Using the rod inside the window SHALL hook the fish.
Using it during a nibble, before the bite, SHALL spook the fish. Letting the
window pass SHALL lose the fish.

#### Scenario: The strongest fish
- **WHEN** a fish of strength 5 bites
- **THEN** the window is 0.432 s

### Requirement: Reeling is held against line tension
While a fish is hooked, holding the use button SHALL shorten the line and
raise its tension, more while the fish runs. Releasing SHALL let tension
fall. Tension reaching its limit SHALL snap the line and lose the fish. A
fish drawn to the shore or the rod SHALL be caught.

#### Scenario: Reeling through a run
- **WHEN** the button is held without a break against a fish that is running
- **THEN** the line snaps before the fish is landed

#### Scenario: A patient reel
- **WHEN** the button is released whenever tension is high and held
  otherwise
- **THEN** the fish is landed

### Requirement: A catch goes into the slots with its own icon and is saved at once
A caught fish SHALL be given to the item slots as an item of its species, and
it SHALL be drawn with that species' own icon. The slots SHALL be written to
the durable save on the frame of the catch. A catch with no room SHALL be
released with a message, not dropped silently.

#### Scenario: Catch and quit
- **WHEN** a fish is caught and the process ends before any other event
- **THEN** the fish is in the slots when the world is loaded
