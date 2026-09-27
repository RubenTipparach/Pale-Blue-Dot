# Player: map

## Purpose

A flat, 2D map of the whole planet that the player opens in the game: where
they are, where their ship is, what the land and the weather are doing, and
the layers later changes add (city sites, climate, fish).

## ADDED Requirements

### Requirement: The map opens and closes without leaving the world

The map SHALL open and close with one key. While it is open it SHALL hold the
pointer the way the other menus do, and the world SHALL keep running behind
it: the clock, the weather and the vehicles do not pause.

#### Scenario: Opening the map

- **WHEN** the player presses M
- **THEN** the map fills the screen, centred on the player, and the pointer is
  free to pan it

#### Scenario: Closing the map

- **WHEN** the player presses M or Escape with the map open
- **THEN** the map closes and walking or flying resumes with the same controls
  as before it opened

### Requirement: The map shows the whole planet and zooms to the ground

The map SHALL show the whole planet at its widest zoom and SHALL zoom in until
one cell is at least a pixel. Pan and zoom SHALL follow the pointer. The map
SHALL be equirectangular at every zoom, as the fish range maps are.

#### Scenario: The whole planet

- **WHEN** the map is zoomed all the way out
- **THEN** every land mass and both poles are on screen at once

#### Scenario: Down to the cell

- **WHEN** the map is zoomed all the way in around the player
- **THEN** a single cell of the terrain is at least one pixel wide

### Requirement: The map is drawn from the world's own functions

The base map SHALL show the planet as it looks: each place in the colour of
its ground's top block, shaded by relief, with its towns. It SHALL be computed
from the same generator functions the terrain uses, for the same seed and
generator version.
It SHALL NOT be a separately authored picture.

#### Scenario: The map agrees with the ground

- **WHEN** a point is sampled on the map and on the terrain at the same
  latitude and longitude
- **THEN** both give the same top block, the same biome and the same land or
  sea

### Requirement: The live layer shows what is happening now

The live layer SHALL show, updated while the map is open:
- the player's position and heading;
- the position of every vehicle the player owns, boarded or parked;
- the night side of the day-night line;
- the clouds and rain from the running weather.

#### Scenario: Walking with the map open

- **WHEN** the player's position changes while the map is open
- **THEN** the player's marker moves to the new latitude and longitude on the
  next frame

#### Scenario: A parked ship

- **WHEN** the player has left a ship parked and walks away
- **THEN** the ship's marker stays where it was parked

#### Scenario: Night

- **WHEN** part of the planet is in night
- **THEN** that part of the map is shaded as night, and the shading moves with
  the clock

### Requirement: Layers are chosen from a legend

The map SHALL list its layers in a legend, each of which can be shown or hidden.
The eight weather overlays (wind, jet stream, currents, cloud, rain, humidity,
sunlight, temperature) SHALL be map layers, and each SHALL be paintable onto
the 3D globe from the legend.

#### Scenario: Showing a weather layer

- **WHEN** the player turns on the rain layer in the legend
- **THEN** the map shows where it is raining now, with a scale in the legend

#### Scenario: An overlay greys the base map

- **WHEN** the player turns on the biome overlay
- **THEN** the base map beneath it is shown greyed out, and the overlay's colours
  are drawn over it

#### Scenario: Painting a layer on the globe

- **WHEN** the player chooses to show the wind layer on the globe and closes
  the map
- **THEN** the 3D globe carries the wind overlay as the M key's cycling did
  before
