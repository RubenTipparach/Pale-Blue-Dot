## ADDED Requirements

### Requirement: The fish at the lure pecks in time with the float

The fish the line is working SHALL come to rest a short distance off the lure,
facing it, and SHALL NOT be drawn with its body fixed at the lure point. In
each nibble its mouth SHALL reach the lure while the float makes the nibble
dip, and it SHALL move back off the lure between nibbles. The nibble and bite
timing rules SHALL be unchanged.

#### Scenario: A perch nibbles three times

- **WHEN** a perch nibbles three times before it bites
- **THEN** it darts in three times, its mouth meeting the lure each time the
  float dips, and backs off between the dips

#### Scenario: The bite

- **WHEN** the line enters the bite
- **THEN** the fish takes the lure and turns away with it, and the float goes
  under

### Requirement: A fish never turns faster than it can swim round

A drawn fish's heading SHALL turn smoothly every frame, at no more than a
configured rate, and SHALL stay within a configured pitch of level. A fish
SHALL NOT be drawn pointing straight up or down because the lure moved
vertically, and SHALL NOT take a fixed fallback heading when it is still.

#### Scenario: The float dips under a nibbling fish

- **WHEN** the float dips and recovers while a fish is at the lure
- **THEN** the fish stays level within its pitch limit

#### Scenario: The fish turns from a run to being reeled

- **WHEN** a hooked fish stops running and is reeled toward the angler
- **THEN** it turns about its mouth over a fraction of a second, never in one
  frame

### Requirement: A fish's tail beat follows its speed

The tail beat's frequency SHALL rise with the fish's speed in body lengths a
second, and its amplitude SHALL differ between hovering, cruising, bursting and
gliding.

#### Scenario: A gliding fish

- **WHEN** a fish is slowing in a glide
- **THEN** its tail beats with a small amplitude, not the cruising one
