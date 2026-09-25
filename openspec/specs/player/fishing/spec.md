# Fishing Specification

## Purpose
The rod: a cast under the planet's gravity, a float on the drawn sea, a bite
from a fish that was swimming there, a hook window, a tension reel, and a
catch that lands in the slots and the save on the same frame. The state
machine is `pbd_core::fishing::Line`; the app draws the rod, float and line
and feeds it the world's water.

## Requirements

### Requirement: A cast flies under the planet's gravity and floats on the drawn sea
With the rod in hand, holding and releasing the use button SHALL throw the
float from the rod tip at a speed set by how long it was held, under the
planet's own gravity. A float that reaches water SHALL float on the sea height
the world draws, from the same wave table the craft float on. A float that
lands on ground SHALL come back empty.

#### Scenario: Casting onto the bank
- **WHEN** the float meets ground before water
- **THEN** the line comes back with nothing on it
  (`fishing::tests::a_cast_over_water_floats_and_a_cast_onto_the_beach_comes_back`)

#### Scenario: Riding a swell
- **WHEN** the float rides a sea with waves
- **THEN** its height is the sea table's height at its position
  (`fish::tests::the_float_rides_the_sea_the_hulls_ride`)

### Requirement: The fish that bites is a fish that was swimming there
A bite SHALL come from a fish of a school simulated near the float, and the
species caught SHALL be that fish's species. There SHALL be no catch table
that is independent of the fish in the water.

#### Scenario: No school near
- **WHEN** no school is within its sense range of the float
- **THEN** no bite comes (`fishing::tests::with_no_school_near_nothing_bites`)

#### Scenario: A school scents the lure
- **WHEN** a school is within three times its sense range of a floating lure
- **THEN** it turns toward the lure and reaches it
  (`fauna::tests::a_scented_school_reaches_the_lure`)

### Requirement: A bite opens a hook window that shortens with the fish's strength
A bite SHALL open a window of `0.9 * (1 - (strength - 1) * 0.13)` seconds,
never under 0.25 s. Using the rod inside the window SHALL hook the fish.
Using it during a nibble, before the bite, SHALL spook the fish. Letting the
window pass SHALL lose the fish. Cloud and rain SHALL make a bite come sooner.

#### Scenario: The strongest fish
- **WHEN** a fish of strength 5 bites
- **THEN** the window is 0.432 s (`fauna::tests::the_hook_window_is_tenebris_formula`)

#### Scenario: Too early, and too late
- **WHEN** the rod is used during a nibble, or not used during a bite
- **THEN** the fish is spooked, or missed
  (`fishing::tests::hooking_during_a_nibble_spooks_the_fish`,
  `fishing::tests::a_missed_bite_lets_the_fish_go`)

### Requirement: Reeling is held against line tension
While a fish is hooked, holding the use button SHALL shorten the line and
raise its tension, more while the fish runs. Releasing SHALL let tension
fall. Tension reaching its limit SHALL snap the line and lose the fish. A
fish drawn in to the rod SHALL be caught, and it SHALL leave its school.

#### Scenario: Reeling through a run
- **WHEN** the button is held without a break against a strong fish
- **THEN** the line snaps before the fish is landed
  (`fishing::tests::reeling_through_every_run_snaps_the_line`)

#### Scenario: A patient reel
- **WHEN** the button is released whenever tension is high and held otherwise
- **THEN** the fish that bit is landed and its school is one fish smaller
  (`fishing::tests::a_patient_reel_lands_the_fish_that_bit`)

### Requirement: A catch goes into the slots with its own icon and is saved at once
A caught fish SHALL be given to the item slots as an item of its species,
drawn with that species' own icon. The slots and the catch record SHALL be
written to the durable save on the frame of the catch. A catch with no room
SHALL be released with a message, not dropped silently.

#### Scenario: Catch and quit
- **WHEN** a fish is caught
- **THEN** the slots and the catch line are in the save that frame
  (`fish::tests::a_catch_lands_in_the_hotbar_and_the_save_at_once`)

#### Scenario: A full hotbar
- **WHEN** a fish is caught with no room for it
- **THEN** it is let go and the player is told
  (`fish::tests::a_full_hotbar_lets_the_fish_go`)
