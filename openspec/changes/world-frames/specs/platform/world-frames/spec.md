## ADDED Requirements

### Requirement: Simulated positions are local to one origin

Every simulated position near the player (the walker's and every Avian body's
`Position`, the fish, the line's float) SHALL be an f32 relative to one
`SimOrigin`, an f64 point in a named frame near the player. "Up", a radius
from a body's centre, and a planet point SHALL be taken only through the
origin's methods, in f64 before the cast. A position kept across a rebase or
saved SHALL be a `WorldPos` (a frame and an f64 point) or the home body's
frame point, converted at the boundary.

#### Scenario: A slow walk far from the centre

- **WHEN** the walker walks at 1 m/s on the home planet, 4 800 m from its
  centre
- **THEN** each physics step's move differs from `velocity * dt` by under
  1e-5 m, where with the origin at the planet's centre it differs by a
  tenth of a millimetre or more

#### Scenario: A fish and the float far from the centre, and on a far body

- **WHEN** a school steps at 30 Hz and the float bobs, at 4 800 m and at
  4 800 km from the body's centre, with the body at the system's origin or
  1 AU from it
- **THEN** each step's move differs from what it was asked by under 1e-5 m

### Requirement: The origin rebases at a tick boundary without moving anything

The origin SHALL follow the player by whole grid cells (16 m on foot, larger
at speed), between physics steps. A rebase SHALL move every Avian body, every
registered piece of simulation state and the render origin together, so that
every planet-frame position is unchanged to 1e-6 m, no velocity changes, and
no drawn transform is moved.

#### Scenario: Walking across a cell

- **WHEN** the walker walks out of the origin's cell and the origin moves
- **THEN** the walker's planet-frame motion across that step is its
  `velocity * dt`, to 1e-5 m

#### Scenario: A rebase

- **WHEN** the origin is rebased by a cell with the walker and the skiff in
  the world
- **THEN** every body's planet-frame point is where it was to 1e-6 m and
  every velocity is the same

#### Scenario: An animal across a rebase

- **WHEN** a herd animal walks at 1 m/s or runs at 8 m/s, 4 800 m or
  4 800 km from its body's centre, and the origin rebases under it
- **THEN** each step's planet-frame move is the step it was asked to 1e-5 m,
  where planet-centred it is off by more

#### Scenario: A drop and a deck across a rebase

- **WHEN** the origin is rebased by a cell with a drop floating and a
  moored deck swinging 4.4 km from the planet's centre
- **THEN** the drop's planet-frame point is unchanged and it is drawn where it
  was, and the deck's planet-frame pose is unchanged to the f32 step of a
  local value and a point it carries moves on as with the origin held

#### Scenario: A deck under sail away from the planet's centre

- **WHEN** a walker's foot is over a sailing cog's deck with the origin
  4.4 km from the planet's centre
- **THEN** the walker finds the deck's floor at its f64 radius

### Requirement: Frames convert exactly between any two

Positions and velocities SHALL convert between any two frames of the frame
tree (the system, inertial and turning bodies, vessels) through their lowest
common ancestor, with every turning frame's `ω × r`, and a handoff of the
local space to another frame SHALL keep every position and velocity.

#### Scenario: A trip through the system and back

- **WHEN** the local space is handed from a planet's surface to the planet,
  the system, a second planet, and back, with the planet 1 AU out
- **THEN** the walker's position in the system is unchanged to 1 mm and its
  velocity to 1e-6 of itself at every handoff

#### Scenario: Walking in a fast ship

- **WHEN** a walker walks at 1 m/s inside a ship moving at 10 km/s, with the
  origin rebased every tick
- **THEN** each step's move is exact to 1e-5 m

### Requirement: Saved poses survive the local origin

A pose saved by a build before the local origin SHALL load to the same place,
and a pose saved, loaded and saved again SHALL be the same to the bit.

#### Scenario: A pose round trip

- **WHEN** a pose is restored far from the origin and saved again
- **THEN** the saved position equals the original
