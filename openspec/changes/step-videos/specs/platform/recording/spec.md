# Platform: recording

## Purpose

How the game shows its own work: scripted shot lists it plays with nobody at
the controls, recorded frame by frame at a fixed step, so each step of the
roadmap ends in a video the owner can watch and judge.

## ADDED Requirements

### Requirement: A showcase plays by itself

The game SHALL play a named shot list with no input from a person. A shot
SHALL set:
- where the camera or walker is, and how it moves;
- the time of day and its rate;
- the actions to perform;
- a caption naming what the shot shows.

The game SHALL print a line when the showcase starts and when it ends, and
SHALL exit when it ends.

#### Scenario: Playing a showcase

- **WHEN** the game is started with `--showcase lights`
- **THEN** it plays every shot of `assets/showcase/lights.ron` in order with
  nobody at the controls, prints the start and end lines, and exits

#### Scenario: A shot's caption

- **WHEN** a shot is playing
- **THEN** its caption is drawn in the frame for the whole shot

### Requirement: A recording is every frame at a fixed step

With `--record <dir>`, the game SHALL advance the simulation by a fixed step
per frame and SHALL write every rendered frame to the directory, numbered in
order. Two recordings of one showcase on one build SHALL have the same number
of frames, with the same shot starting on the same frame.

#### Scenario: Recording a showcase

- **WHEN** a showcase is played with `--record`
- **THEN** the directory holds one image per frame at the fixed step, and the
  frame count is the showcase's length times the frame rate

#### Scenario: Two recordings

- **WHEN** the same showcase is recorded twice on the same build
- **THEN** both have the same frame count, and each shot starts on the same
  frame in both

### Requirement: A gate's video says what it shows and where it was made

Every recording made for a gate SHALL be published with:
- its shot list, giving each shot's start time, what to look for, and the
  requirement it demonstrates;
- a contact sheet;
- a label saying whether it was rendered at a fixed step without a GPU, or in
  real time on the owner's hardware.

A fixed-step recording SHALL NOT be presented as evidence of smoothness or
frame rate.

#### Scenario: A cloud recording

- **WHEN** a gate's video was rendered in a cloud session
- **THEN** its page says it was rendered at a fixed step without a GPU, and
  shows what things look like, not how smoothly they run
