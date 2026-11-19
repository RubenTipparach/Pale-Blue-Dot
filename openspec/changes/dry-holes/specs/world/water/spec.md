## MODIFIED Requirements

### Requirement: The sea floods what is opened onto it

At the tier's build, air below sea level joined to the sea's generated water
across the sides or downward SHALL fill as a source, within the tier.

On an edit, only the cell the edit opened SHALL fill at once, and only where
the sea's own water is over it or beside it under the sea's level. The rest
SHALL come in by the water rule.

Under the sea's level, in a cell joined to the sea through what is not
solid, water the rule would make flowing at the top level and that is not
falling SHALL be a source. So the sea:
- runs along a channel cell by cell;
- falls over a lip as falling water;
- fills a pit from its floor to the sea's level and no higher.

A source poured into a pocket not joined to the sea SHALL run by the rule
alone. A pocket not joined to the sea SHALL stay dry, and no sea sheet SHALL
be drawn over it.

Proven by `water_tier_tests` and
`liquid::tests::the_sea_fills_what_it_feeds_and_falls_over_a_lip`
(`dry-holes` design 4).

#### Scenario: A tunnel into the sea

- **WHEN** a tunnel under land reaches the sea below sea level
- **THEN** it is water to sea level, and a sealed pocket beside it is air

#### Scenario: A channel cut from the shore

- **WHEN** a channel dug dry from the land is opened to the sea below the
  sea's level
- **THEN** the sea fills it cell by cell over the rule's ticks, to the sea's
  level

#### Scenario: A pit's wall breached

- **WHEN** the wall between the sea and a dry pit dug below the sea's level
  is dug through at the sea's top layer
- **THEN** the sea pours over the lip as falling water, and the pit fills
  from its floor up to the sea's level and no higher, and only then does the
  sea's sheet lie over it

#### Scenario: A bucket in a sealed pit

- **WHEN** a source is poured into a sealed pit below the sea's level
- **THEN** it spreads as flowing water and makes no new source

## ADDED Requirements

### Requirement: A dug hole is dry until water reaches it

A hole a player digs SHALL hold no water, and SHALL NOT be drawn, seen or
swum as water, until water reaches it through a neighbouring water cell,
however far below the sea's level or a river's surface its floor goes.
The sea's reach SHALL NOT run through the air under the sea's level in a
dug column, and SHALL take one only where the sea's water or solid ground
is at the sea's top layer in it. Proven by
`a_pit_dug_below_the_sea_beside_a_beach_stays_dry` (`dry-holes` design 3).

#### Scenario: A pit dug beside a beach

- **WHEN** a pit is dug from the land next to a beach the high tide
  overtops, its floor three metres under the sea's level, and its walls
  touch no water
- **THEN** its cells are air after the rule has run and after a reload, no
  sea sheet is listed over it, and the eye and the walker in it are dry

#### Scenario: A pit dug beside a river

- **WHEN** a pit is dug beside a river, its floor under the river's surface,
  and its walls touch no water
- **THEN** it is dry, and stays dry until its wall is dug through
