# Water Environments Specification

## Purpose
The water places of a body and how they look from inside: today, the light
under water (`pbd_core::ocean`, `water.wgsl`, `planet_surface.wgsl`). Where
each place occurs and the places' cells are designed in
`openspec/changes/ocean-environments` and not built. The sea has no tide: its
surface stands at one level (`no-tides`).

## Requirements

### Requirement: Daylight under water fades with depth
The light reaching a point under water SHALL fall with its depth below the
water's surface by a downwelling coefficient per colour, separate from the
view's absorption per metre of path. The view SHALL also scatter toward the
water's own colour by a turbidity read off the water under the eye (a
river's own; the sea's from the coast's in the shallows to the open ocean's
past the shelf), eased so it never steps.

#### Scenario: The zones fall inside the column
- **WHEN** the blue light left in ocean water is evaluated at 40 m and at
  90 m
- **THEN** it is 15% or more at 40 m (sunlit above) and 1.7% or less at
  90 m (midnight below), so every zone lies above `column::BASE_M`

#### Scenario: Red goes first
- **WHEN** the light left at 10 m is evaluated
- **THEN** red is under 2% while green and blue are above 50%

#### Scenario: A swim out never steps the turbidity
- **WHEN** the turbidity is read every 25 cm from the shore out past the
  shelf
- **THEN** no two readings differ by more than 0.002 per metre

### Requirement: The eye under any water sees the underwater view

When the eye is in water (the sea, a river, a pool, flowing water), the
underwater view SHALL be measured from the surface of the water the eye is
in: the depth fog and colour, the turbidity, the surface seen from below,
the light shafts and the distortion, as the sea gives them. The swell SHALL
be added only to the sea's surface.

#### Scenario: Down in a river

- **WHEN** the eye is under a river's water, above sea level
- **THEN** the view is fogged and tinted by the river's water from its own surface
- **AND** the river's sheet is drawn from below as the sea's is

### Requirement: The surface seen from below is Snell's window and the murk

Seen from under the water, the surface SHALL show Snell's window (what is
over the water, refracted, seen through the water between) and outside it
the water's own murk, the waves and the rain's rings only a gentle shading
on it (at most 12% of the murk either way). It SHALL NOT mirror the scene
under the water and SHALL NOT read the screen-space reflection history, at
any REFLECTIONS level, in the sheet or in the composite's stand-in for the
sheet the near plane clips (`ocean-environments` B21).

#### Scenario: Looking up from under the sea

- **WHEN** the eye is under the sea and looks at the surface outside Snell's window
- **THEN** the surface is the murk, shaded by the waves
- **AND** no part of the sea floor or the scene under the water is drawn in it

### Requirement: The composite's waterline is the sea's surface for an eye in air

The composite SHALL measure the waterline from the surface of the water the
eye is in, and for an eye in air from the sea's drawn surface (the sheet,
half a metre under the sea's level), never a sphere of its own, so a jump or
a flight over a beach tints and wobbles none of the dry sand over the water
(`ocean-environments` B21). The sea's surface SHALL stand at one level: it
SHALL NOT rise and fall with the moon or the time of day (`no-tides`).

#### Scenario: A jump on a beach

- **WHEN** the eye rises from standing on a beach over the sea's water into air over it
- **THEN** the composite's waterline stays at the sea's drawn surface
  (`planet::water::tests::a_jump_over_a_beach_keeps_the_composites_waterline_at_the_sea`)

### Requirement: New water places are a new generator version
Reef, kelp, vent, trench, sea-ice, stepped-river and marsh-pool cells SHALL be
made only by a new terrain generator version, for new worlds. A world SHALL
keep the generator version it names, cell for cell, and opening it SHALL NOT
rebuild it with the new places.

#### Scenario: An old world opens unchanged

- **WHEN** a world saved on the previous generator version is opened by a
  build that carries the new one
- **THEN** its ground digest matches the previous version's pinned digest

### Requirement: The sea's cells are ground a player digs
On a world of generator version 14 or later, a reef's top SHALL be reef rock
in patches, a vent's ground basalt with mussel beds in patches, and polar
water SHALL stand floes of sea ice one cell high where it is at least 2 m
deep. Each SHALL be a cell like any ground: dug with its tool, giving its
own block item, placed back where the player puts it, and drawn from its own
sheet's tile whether generated or placed, with the column's four-bit
material word unchanged. A world of an older version SHALL have none of
them.

#### Scenario: Reef rock is dug and placed
- **WHEN** the player digs a reef rock cell with a pickaxe
- **THEN** a reef rock block comes into the pack, and placed on land it is
  drawn as reef limestone

#### Scenario: An old world keeps its floes as pieces
- **WHEN** a world saved on generator version 12 is opened
- **THEN** its polar sea has no sea ice cells, its floes are the pieces
  they were, and its ground and top-block digests match version 12's
