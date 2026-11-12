# Startup: opening a world

## ADDED Requirements

### Requirement: A loading screen is shown from launch until the world is playable

The desktop game SHALL draw a loading screen from its first frame until the
world is playable, naming the step in hand and showing its progress, in the
HUD's monospace pixel look. The window SHALL keep drawing frames while the
world is prepared.

#### Scenario: A world is opened at launch

- **WHEN** the game is launched onto a world
- **THEN** the first frame shows the loading screen, and it stays until the
  `READY` line is logged

### Requirement: The planet is built once when a world opens

Opening a world at launch SHALL read or make its sites and towns, install the
towns' ground, and only then build the planet, once, off the main thread. The
towns SHALL NOT trigger a second planet build.

#### Scenario: A saved world with towns is opened

- **WHEN** a world whose save holds its sites and towns is opened at launch
- **THEN** the planet is built once, with the towns' ground installed, and no
  "planet rebuilt" line is logged before `READY`

### Requirement: The planet built once equals the planet built twice

For the same world and direction, the planet built once with the towns'
ground installed SHALL equal, byte for byte, the planet built without it and
then rebuilt with it; and the records the loading path stores SHALL equal the
records today's sites and towns systems store.

#### Scenario: The two paths are compared

- **WHEN** a fresh world is prepared through the loading path and the same
  world is built the old way
- **THEN** the base records, every fine level's records and the column tier
  are identical, and so are the stored records

### Requirement: Pipelines are compiled behind the loading screen

The world SHALL run and render under the loading screen after it is built, and
the screen SHALL lift only once the pipeline cache has had nothing waiting for
15 frames, the camera has looked round, and the towns in range stand, or after
90 s. The world clock SHALL NOT advance behind the screen.

#### Scenario: Shaders compile while loading

- **WHEN** the world has been built and pipelines are still compiling
- **THEN** the screen shows the compiled and total pipeline counts and stays up

### Requirement: The end of loading is logged

The game SHALL log one `READY` line when the loading screen lifts, giving the
seconds since launch, the frame number, and the pipeline count, and SHALL log
each pipeline created after it as `LATE_PIPELINE` with its label.

#### Scenario: A perf run reads its warm-up

- **WHEN** a `--frame-log` run reaches the end of loading
- **THEN** the log has a `READY` line whose frame number matches a row of the
  frame log
