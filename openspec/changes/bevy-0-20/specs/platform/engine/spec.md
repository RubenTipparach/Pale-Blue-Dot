# Engine: the Bevy 0.20 upgrade keeps worlds and pictures

## ADDED Requirements

### Requirement: An engine upgrade leaves every generator version's ground unchanged

After a change of engine, math or physics library version, the ground
digest of every generator version a save can name SHALL equal the digest
recorded before the change.

#### Scenario: The digests after moving to Bevy 0.20.1 and glam 0.33

- **WHEN** the digest tests run on the upgraded workspace
- **THEN** every generator version's ground digest equals the one recorded
  on Bevy 0.18.1

### Requirement: Custom render passes run in a fixed order

Each of the game's own render passes (the planet cull, the shadow cascades,
the water composite, the reflection capture, the lite bloom and the tone
grade) SHALL be ordered by explicit dependencies relative to the passes
whose output it reads. Its position in the frame SHALL NOT depend on the
scheduler's choice.

#### Scenario: A weather change captured frame by frame

- **WHEN** a `--capture` sequence records every frame across a change of
  weather
- **THEN** no frame shows the water composite, reflections or bloom applied
  out of order, such as fog drawn before the sky or bloom missing for one
  frame
