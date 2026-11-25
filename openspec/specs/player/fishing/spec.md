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

### Requirement: The line runs from the rod tip to the float
While the float is out, the drawn line SHALL start at the rod's tip and end
at the drawn float, each frame, from the same frame's camera and float. The
points between SHALL lie on or over the water the float rides, its waves
included, not through it.

#### Scenario: A wave trough and a bite
- **WHEN** the float rides a wave trough, or is pulled under on a bite
- **THEN** the line's last point is the float and no point between is under
  the water (`fish::tests::the_line_runs_from_the_rod_tip_to_the_float`)

### Requirement: A strike at the bite lands the fish, and a lost fish is said
While the float is under, either the use button or the wind-in button SHALL
hook the fish. A hooked fish reeled in without snapping the line SHALL be in
the slots and the save on the frame it lands. Winding in, or losing the line,
with a fish nibbling, biting or hooked SHALL say the fish got away.

#### Scenario: Striking with the wind-in button
- **WHEN** the player presses the wind-in button while the float is under
- **THEN** the fish is hooked (`fishing::tests::a_right_click_at_the_bite_hooks_the_fish`)

#### Scenario: A struck bite reeled in
- **WHEN** a bite is struck inside its window and reeled patiently
- **THEN** the fish is in the slots and the save
  (`fish::tests::a_bite_struck_in_its_window_puts_the_fish_in_the_slots`)

#### Scenario: Winding in on a nibble
- **WHEN** the line is wound in while a fish nibbles
- **THEN** a `Lost` event names it before the line comes in
  (`fishing::tests::winding_in_on_a_nibble_says_the_fish_got_away`)

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

### Requirement: The fish at the lure pecks in time with the float
The fish the line is working SHALL come in to a stand-off point a short way
off the lure, on the side it came from, slowing as it nears it, and SHALL NOT
be drawn with its body fixed at the lure point or jump onto it when the
nibble starts. In each nibble its mouth SHALL reach the lure while the float
makes the nibble dip, and it SHALL move back off the lure between nibbles. At
the bite it SHALL take the lure and turn away with it; struck too early or
missed, it SHALL bolt and rejoin its school where it was drawn. The nibble and
bite timing rules SHALL be unchanged (`fish-behaviour` design 2).

#### Scenario: A fish nibbles, bites and is hooked
- **WHEN** a fish nibbles before it bites and is struck
- **THEN** its mouth meets the lure in each nibble's dip, is more than 10 cm
  off it before a long nibble's dart, and no frame turns it or its up more
  than 20 degrees or moves it more than 10 cm
  (`fauna::swim::tests::a_fish_pecks_in_time_with_the_float_takes_the_lure_and_fights_without_a_snap`)

#### Scenario: Struck too early
- **WHEN** the angler strikes during a nibble
- **THEN** the fish bolts, turning and moving smoothly, and its school has it
  back where it was drawn when the bolt ends
  (`fauna::swim::tests::a_fish_struck_too_early_bolts_and_rejoins_its_school_where_it_was_drawn`)

#### Scenario: The rules are the rules
- **WHEN** a line is fished as before
- **THEN** every fishing test passes unchanged, with the lured fish nibbling
  from its stand-off point (`fishing::tests::*`)

### Requirement: The fish on the line swings round its mouth
A hooked fish's mouth SHALL stay on the drawn lure and its body SHALL swing
round it: head away from the rod while it runs, toward the rod with head
shakes and a flashing roll while it is reeled. The turn between the two SHALL
be eased in rate, the shake and the flash SHALL fade in and out, and the drawn
float and lure SHALL NOT jump when the line's swing starts or stops
(`fish-behaviour` design 3, 3.1).

#### Scenario: The fish turns from a run to being reeled
- **WHEN** a hooked fish switches between running and resting
- **THEN** its mouth stays within a centimetre of the drawn lure and no frame
  turns it more than 20 degrees
  (`fauna::swim::tests::a_fish_pecks_in_time_with_the_float_takes_the_lure_and_fights_without_a_snap`)

### Requirement: A fish never turns faster than it can swim round
A drawn fish's heading SHALL turn every frame by yaw and pitch, eased and at
no more than a configured rate, and SHALL stay within a configured pitch of
level; it SHALL bank into a turn. A school's swim SHALL turn no faster than a
configured rate a step, and each fish SHALL burst and glide on a cycle of its
own (`fish-behaviour` design 4).

#### Scenario: A school swims
- **WHEN** a school swims freely for half a minute
- **THEN** no drawn fish turns faster than its limit or pitches past it, and
  fish bank into their turns
  (`fauna::swim::tests::a_free_fish_turns_no_faster_than_its_limit_and_banks_into_turns`)

#### Scenario: A school steps
- **WHEN** a school steps for forty seconds
- **THEN** no fish's swim turns more than the limit in a step, and its speed
  rises and falls (`fauna::tests::a_fish_turns_no_faster_than_its_limit_and_bursts_and_glides`)

### Requirement: A swimmer's body bends in a wave from head to tail
A drawn fish or eel SHALL bend along its body in the vertex stage in a wave
whose crests travel from the head toward the tail, with a head that yaws
against the tail, a beat and amplitude set by its speed, a straight body in a
glide and a curve into a turn; a ray's wings SHALL undulate nose to tail. The
normals SHALL turn with the bend, and the bend SHALL NOT change where the fish
is or which way it faces (`fish-behaviour` design 8).

#### Scenario: A cruising perch
- **WHEN** the wave's phase grows
- **THEN** its crest moves toward the tail and the nose swings against the
  tail (`fauna::swim::tests::the_body_wave_runs_from_the_head_to_the_tail`)

#### Scenario: The slope the normals turn by
- **WHEN** the shader's lateral offset is differentiated numerically
- **THEN** it matches the slope the normals are turned by
  (`fauna::swim::tests::the_body_wave_slope_is_the_derivative_of_its_offset`;
  `shader_tests::the_body_wave_module_compiles_and_the_school_vertex_stage_imports_it`)

#### Scenario: A perch turns, then glides
- **WHEN** a perch turns left at 90 deg/s, then slows hard
- **THEN** its body bows left and it banks into the turn, then its tail's
  amplitude falls to the glide's
  (`fauna::swim::tests::a_turning_fish_bends_into_its_turn_and_a_gliding_one_straightens`)

#### Scenario: No wave
- **WHEN** a body's amplitude and bend are zero
- **THEN** it is drawn as laid
  (`fauna::swim::tests::no_amplitude_and_no_bend_leave_the_body_as_laid`)
