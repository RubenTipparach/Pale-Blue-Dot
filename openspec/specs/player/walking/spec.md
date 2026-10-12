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

### Requirement: A walker stands and walks on a moving deck
A walker on a craft's deck SHALL be simulated in the craft's frame. It SHALL
be carried by the craft's motion, turning included, and SHALL walk, climb
stairs and meet rails as it does on the ground, with no slide or jitter from
the craft's motion. Leaving the deck SHALL return it to the planet's frame at
its composed velocity.

Its feet SHALL keep their place on the deck and its body SHALL stand over
them along the planet's up, so a deck that heels does not tip the walker
(`vehicles::tests::a_walker_rides_a_cog_under_way_through_a_turn`,
`a_walker_climbs_a_cogs_stair_under_way`,
`a_walker_goes_over_a_cogs_side_with_its_way`; the moored deck's in
`walking::tests`).

#### Scenario: Standing still through a turn
- **WHEN** a walker stands on the cog's deck, 6 m from its mast, while the cog
  turns through 90°
- **THEN** the walker's position in the ship's frame moves less than 1 cm

#### Scenario: Up the stair under way
- **WHEN** a walker climbs the stair to the aftcastle while the cog sails
- **THEN** there is no eye jump over 0.1 m and no airborne tick, as on a stair
  ashore

#### Scenario: Over the side
- **WHEN** a walker steps off the rail while the cog sails
- **THEN** it falls into the sea with the ship's velocity at that point, and
  the cog sails on
