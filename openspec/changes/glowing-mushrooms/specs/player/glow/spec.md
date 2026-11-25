## ADDED Requirements

### Requirement: Eating a glowing mushroom gives the body glow

Right clicking with a `glowing-mushroom` in the selected slot SHALL eat one,
taking it from the slot and running the Glow effect for its dish row's minutes
of play, in one durable write with the `meal` record. Glow SHALL follow
cooking's rules: the same effect does not stack, the longer time counts, and
at most three effects run. No status SHALL drain while the player does not
eat.

#### Scenario: A second mushroom refreshes the glow

- **WHEN** a player with 1 minute of Glow left eats another glowing mushroom
- **THEN** Glow has 3 minutes left and is no brighter than before

#### Scenario: The glow survives a reload

- **WHEN** a player eats a glowing mushroom, plays 1 minute, saves and opens
  the world again
- **THEN** Glow has 2 minutes left, as world time passed only while the world
  was played

### Requirement: The body glow rises and fades, never cuts

While Glow runs, the player SHALL carry a light at `glow_level` in
`glow_tint`. Its strength SHALL rise from 0 over `glow_rise_s` after eating,
and fall on a smoothstep to 0 over the last `glow_fade_s` of the effect, so
it reaches 0 when the effect ends.

#### Scenario: The last seconds

- **WHEN** Glow has 10 s left and `glow_fade_s` is 20
- **THEN** the light is at half strength, and at 0 s it is out
