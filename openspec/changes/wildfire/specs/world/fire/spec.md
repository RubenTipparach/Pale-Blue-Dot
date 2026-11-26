# Spec Delta

## Purpose

Fire in a Survival world: how dry each place and each plant is, small
forest fires that start in the dry season, spread through what is dry and stop in rain, water and bare
ground, what they leave and how it grows back, and torches that the rain
puts out and that light things.

## ADDED Requirements

### Requirement: The weather knows how dry the ground is
The atmosphere SHALL keep the ground's wetness in each of its cells, raised
by rain and dried by sun, warmth and dry air, saved with the weather. A
world saved before it existed SHALL open with the wetness at its spun-up
value.

#### Scenario: Drying out
- **WHEN** soaked ground has a world day of dry summer sun
- **THEN** its wetness falls far enough that cured grass on it can burn

#### Scenario: An old save
- **WHEN** a world saved before the ground's wetness existed is opened
- **THEN** its weather restores, with the wetness at each cell's starting
  value

### Requirement: Every plant burns by how dry it is, and green plants do not burn
Each cell SHALL have a plant dryness from 0 to 1, derived and never saved,
from the plant's kind, its season in its own hemisphere's year, the
ground's wetness, the air's humidity and warmth, shade under trees, water
nearby, the ground's hollows and rises, the soil's own patches, grazing and
snow. A cactus, a cell under snow and a flooded cell SHALL read 0. A cell
whose dryness is below one half SHALL never catch fire, and above it the
chance of catching SHALL grow steeply with dryness.

#### Scenario: Spring-green grass
- **WHEN** a Survival player uses a lit torch on grass in spring
- **THEN** a small flame dies within a few seconds and nothing is saved

#### Scenario: A river's bank
- **WHEN** a fire reaches the grass beside a river in the dry season
- **THEN** that grass does not catch

#### Scenario: Each hemisphere its own season
- **WHEN** it is late summer in one hemisphere
- **THEN** the other hemisphere's open grass is too green to burn

### Requirement: Forest fires start only in the dry season, in Survival
In a Survival world, a lit torch used on a dry enough plant, or a lightning
strike from a dry storm on a very dry plant, SHALL start a fire, with
lightning starting at most one in a quiet spell over the same ground. A
placed campfire, brazier, torch or lantern SHALL never start one. In
Explorer there SHALL be no forest fires.

#### Scenario: A torch on dry grass
- **WHEN** a Survival player uses a lit torch on grass in late summer after
  dry weeks
- **THEN** that cell burns, and the fire may spread to its dry neighbours

### Requirement: Fires stay small
A fire SHALL spread only to touching cells, throw no embers, burn out where
its front meets green or wetter ground, and never burn more than its cell
limit or have more than its burning limit alight at once, with no more than
two fires burning in a world. A grass fire's front SHALL move at well under
a metre a second in any wind, and a tree SHALL burn far slower than grass,
in its crown only when it is a very dry conifer in a strong wind.

#### Scenario: A late-summer grass fire in a breeze
- **WHEN** a fire burns through dry fields in a 3 m/s wind
- **THEN** it burns at most 300 cells and is out within minutes of world
  time

### Requirement: Fire spreads by one rule the server runs
A forest fire SHALL spread cell to cell through grass, shrubs and trees by a
fixed one-second rule, faster downwind and uphill, using each cell's own
five or six neighbours, with every chance drawn from the world seed, the
fire, the cell and the tick. The server SHALL run it, and the same fire SHALL
burn the same cells on every machine and after a reload.

#### Scenario: A reload mid-fire
- **WHEN** a world is closed while a fire burns and reopened
- **THEN** the fire goes on from the cells it had reached, and burns the same
  cells it would have burned without the reload

### Requirement: Rain, water and bare ground stop fire, and the player's things never burn
Rain SHALL slow a fire and put it out, heavy rain stopping any cell from
catching. Water, sand, rock, snow, soil without grass, paths and roads SHALL
not burn, grazed pasture SHALL be too green to burn, and a bucket of water SHALL put out a burning cell and its neighbours.
Every cell, block and thing a player built, placed, tilled or planted SHALL
stop a fire and SHALL NOT burn.

#### Scenario: A dug break
- **WHEN** a grass fire reaches a line the player dug down to soil
- **THEN** it does not cross it in calm air

#### Scenario: A field in the way
- **WHEN** a fire reaches a player's planted field and fence
- **THEN** the field and the fence are untouched

### Requirement: A fire's scar is saved and grows back
Every cell a fire burned SHALL be saved, authored by the fire as a world
process, with who lit it named. Burnt ground SHALL show ash, burnt trees
SHALL stand as charred snags that give charcoal, and the scar SHALL grow back
by world days of play: grass first, then shrubs, then saplings, then full
trees.

#### Scenario: The day after
- **WHEN** a player returns to a burnt forest the next world day
- **THEN** its ground is ash and its trees stand charred

#### Scenario: Sixty days on
- **WHEN** sixty world days of play have passed since it went out
- **THEN** the forest stands as before

### Requirement: Torches go out in the rain and are relit, in Survival
In a Survival world, a torch held or placed under the open sky in rain SHALL
go out after a short time, and a roof SHALL keep it lit. A placed torch
going out SHALL leave the player's block unchanged and be kept as a record
of the world's. An out torch SHALL be relit from any fire, and a lit torch
SHALL light an out torch, a brazier, a candle and a lantern. In Explorer,
torches SHALL never go out.

#### Scenario: A torch by the field in a storm
- **WHEN** rain falls on a Survival player's placed torch under the sky
- **THEN** it goes out after about 20 s, stays out across a reload, and lights
  again when the player touches a lit torch to it
