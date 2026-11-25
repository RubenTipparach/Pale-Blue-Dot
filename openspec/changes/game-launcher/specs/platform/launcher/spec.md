# Spec Delta

## Purpose

A small launcher per operating system that keeps a player's game on the
latest build published to itch.io, installs updates without ever breaking the
game or touching saves, and keeps itself up to date.

## ADDED Requirements

### Requirement: The launcher fetches the build for its operating system
The launcher SHALL check the itch.io channel that matches the operating
system it runs on (Windows, Linux or Mac) and SHALL fetch only that
channel's build.

#### Scenario: A Linux player
- **WHEN** the launcher starts on Linux
- **THEN** it asks itch for the latest build on the Linux channel and never
  downloads the Windows or Mac build

### Requirement: The game starts only on the latest build it can reach
On every start, the launcher SHALL ask itch for the latest version before it
starts the game. When a newer version exists and can be downloaded, it SHALL
install it before starting the game. When itch cannot be reached, it SHALL
start the last installed build and say in one short line that it is offline.

#### Scenario: A new build was published
- **WHEN** a player starts the launcher after a new build was pushed to their
  channel
- **THEN** the new build is downloaded and installed, and the game that
  starts is the new build

#### Scenario: No connection
- **WHEN** the launcher starts with no network
- **THEN** the last installed build starts, and the launcher shows that it is
  offline

#### Scenario: First start with no connection
- **WHEN** the launcher starts with no network and no build installed
- **THEN** it shows that it needs a connection to download the game, and
  starts nothing

### Requirement: An update is atomic and reversible
A new build SHALL be downloaded and unpacked into its own folder beside the
current one and checked to be complete before the launcher switches to it.
The switch SHALL be a single step, and the previous build SHALL be kept until
the new one has started once.

#### Scenario: The download is cut off
- **WHEN** the connection drops halfway through a download
- **THEN** the current build is untouched and still starts, and the next
  start resumes or retries the download

#### Scenario: The new build does not start
- **WHEN** a freshly installed build exits with an error before its loading
  screen finishes
- **THEN** the next start runs the previous build and offers the update again
  later

### Requirement: Saves and settings are never inside a build
Worlds and player settings SHALL live in the player's own data folder, apart
from every build folder, so installing or removing a build SHALL never change
them. Worlds found beside an older game exe SHALL be moved to the data folder
once, and SHALL NOT be deleted from their old place until the move is
verified.

#### Scenario: Updating after playing
- **WHEN** a player who has made progress in a world updates to a new build
- **THEN** the world opens in the new build exactly as they left it

#### Scenario: A world from a hand-downloaded zip
- **WHEN** the launcher first runs for a player whose worlds are in the
  `saves` folder beside an older game exe
- **THEN** those worlds are copied to the data folder, open in the new build,
  and the old copies stay until the copy is verified

### Requirement: The launcher keeps itself up to date
The launcher SHALL check its own itch channel on start and SHALL replace
itself with a newer version before it checks the game, restarting itself
once to do so.

#### Scenario: A new launcher was published
- **WHEN** a newer launcher exists on the launcher's channel
- **THEN** the launcher downloads it, replaces itself, restarts, and then
  updates the game as usual

### Requirement: A running game knows an update is waiting
While the game runs from the launcher, it SHALL show "Update ready" on its
menu when a newer build has been published, and a multiplayer join refused
for a version difference SHALL tell the player to restart the launcher.

#### Scenario: A build lands mid-session
- **WHEN** a new build is published while a player is in the game
- **THEN** the game's menu shows "Update ready", and the next launcher start
  installs it
