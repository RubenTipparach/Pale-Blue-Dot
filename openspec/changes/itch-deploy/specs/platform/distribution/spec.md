# Distribution: a shipped build

## ADDED Requirements

### Requirement: A shipped build reads its assets beside its exe

The game SHALL read its assets from an `assets` directory beside its
executable when one exists, and from the checkout it was compiled in
otherwise. Every runtime asset read, Bevy's asset server among them, SHALL use
the same root.

#### Scenario: A packaged build is run from another directory

- **WHEN** the exe and an `assets` folder are copied together into a folder
  and the exe is launched with another working directory
- **THEN** the game reads that folder's assets and reaches `READY`

#### Scenario: A dev build is run

- **WHEN** the exe in `target/` has no `assets` folder beside it
- **THEN** the game reads the checkout's `assets/`, as before this change

### Requirement: A shipped build keeps its saves beside its exe

A shipped build (one whose `assets` folder is beside its exe) SHALL keep its
worlds in a `saves` folder beside its exe, whatever the working directory. A
dev build SHALL keep them where it did before this change, `saves` in the
working directory, so every existing dev world opens where it is.

#### Scenario: A shipped build is double-clicked

- **WHEN** a shipped build is started with `/` as its working directory
- **THEN** its worlds are written under `<exe dir>/saves`

#### Scenario: An existing dev world

- **WHEN** a dev build is started in a directory holding `saves/`
- **THEN** it lists and opens the worlds in that `saves/`

### Requirement: One manual pipeline ships every desktop platform to itch

The repository SHALL have one GitHub Actions workflow, run only by hand on any
branch, that builds the release game for Windows, Linux and macOS and
packages each with its assets. It SHALL keep each package as a workflow
artifact and, unless the run is a dry run, push each to its itch channel. A
missing `BUTLER_API_KEY` secret SHALL fail the deploy with a message saying
to add it.

#### Scenario: A deploy run

- **WHEN** the workflow is run with `deploy` ticked and the secret set
- **THEN** the channels `windows`, `linux` and `mac` (with the run's suffix) of
  `ruben-tipparach/pale-blue-dot` each receive that commit's build

#### Scenario: A dry run

- **WHEN** the workflow is run with `deploy` unticked
- **THEN** the three packages are built and kept as artifacts, and nothing is
  pushed to itch
