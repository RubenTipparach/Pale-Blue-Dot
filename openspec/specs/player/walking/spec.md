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
