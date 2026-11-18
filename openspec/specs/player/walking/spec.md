# Walking Specification

## Purpose
The desktop explorer starts on foot on dry land. Looking around is the most
frequently exercised interaction in the game, so it is held to a hard rule: the
view follows the mouse this frame, with nothing between them.

## Requirements

### Requirement: Mouse look is immediate
Player view SHALL use the current frame's raw mouse displacement. Camera
easing, smoothing, interpolation delay, and any dependence on ship angular
response SHALL NOT be applied to it. Physical limits on a ship's own attitude
remain separate and still apply.

#### Scenario: A fast swipe
- **WHEN** the mouse moves a large distance in one frame
- **THEN** the full displacement reaches the view that frame
- **AND** the result does not depend on how long the frame took

#### Scenario: A frame with no physics tick
- **WHEN** the renderer runs a frame in which no fixed physics tick occurs
- **THEN** the mouse movement still reaches the camera that frame

### Requirement: The walker starts standing on drawn ground
A new walker SHALL be placed on dry land, resting on the rendered surface, with
the camera at eye height above the feet.

#### Scenario: A fresh session
- **WHEN** the desktop explorer starts
- **THEN** the walker's feet rest on the rendered ground
- **AND** the camera sits one eye height above them

### Requirement: Terraces block without breaking movement
Walking into a terrace wall SHALL stop horizontal motion into it without
removing the ability to jump to full height or to walk away along it. Landing
beside a terrace SHALL leave the walker free to move.

#### Scenario: Pushing into a wall
- **WHEN** the walker holds movement into a terrace face
- **THEN** ground velocity into the face is zero
- **AND** a jump from there still reaches full height

#### Scenario: Landing beside a terrace
- **WHEN** the walker lands next to a terrace
- **THEN** the footprint is clear and the walker can walk away

### Requirement: Jumping uses the physics gravity and needs a fresh press
A jump SHALL use the same gravity the physics engine integrates, and holding
the key down SHALL NOT produce repeated hops on landing.

#### Scenario: Holding the jump key
- **WHEN** the jump key is held through a landing
- **THEN** the walker does not immediately jump again

### Requirement: Mode handoff keeps one camera and one ship
Switching between walking and flying SHALL enable exactly one camera and SHALL
leave the ship in the world as the same entity.

#### Scenario: Handing off to flight
- **WHEN** the player switches modes
- **THEN** exactly one camera is active
- **AND** the ship is the same ship, relocated rather than replaced

### Requirement: Auto step is the player's choice

Auto step SHALL be off for a new player (the owner's choice, 2026-11-13).
With auto step on, a grounded walker SHALL walk up a
rise of one terrain layer without a key. With it off, a grounded walker
SHALL walk up only rises no taller than a stair's (0.35 m), and a terrain
layer SHALL stop it until it jumps.

#### Scenario: Auto step on

- **WHEN** auto step is on and the walker walks into a one-layer rise
- **THEN** it ends up standing on top of the rise

#### Scenario: Auto step off

- **WHEN** auto step is off and the walker walks into a one-layer rise
- **THEN** it stays below the rise

#### Scenario: Jumping up with auto step off

- **WHEN** auto step is off and the walker jumps while walking into a
  one-layer rise
- **THEN** it ends up standing on top of the rise

### Requirement: A swimmer is half as fast as before
A walker swimming (eyes under and not wading, or in the water with its feet
off the bottom) SHALL move at `swim_speed_mult` (0.5, `swimming.ron`) of the
speed it had in the water before, at the surface, under it and sprinting.
A wader SHALL keep its speed
(`walking::swim_pace_tests::a_swimmer_goes_half_as_fast_and_a_wader_does_not`).

#### Scenario: Swimming ahead
- **WHEN** a walker swims ahead in still deep water
- **THEN** it makes half the speed it made before this change

### Requirement: The water's current pushes a swimmer whose feet are off the bottom
While a walker is in the water with its feet off the bottom, it SHALL be
carried toward `current_share` (0.5) of the water's velocity there, capped
at `current_max_mps` (1 m/s), easing at `current_rate_per_s` (1.5 /s). With
its feet down it SHALL NOT be pushed, and its carried way SHALL ease to zero
(`walking::swim_pace_tests::a_current_carries_a_swimmer_downstream_and_not_one_standing`,
`walking::swim_pace_tests::the_push_eases_without_a_jolt`).

#### Scenario: Floating in a river
- **WHEN** a swimmer floats with no input in a river flowing at 1.2 m/s
- **THEN** it drifts downstream, along the flow, at 0.6 m/s within a few
  seconds

#### Scenario: Standing on the bed
- **WHEN** a walker stands on the river's bed
- **THEN** the current does not move it
