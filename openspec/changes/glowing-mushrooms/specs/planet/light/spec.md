## ADDED Requirements

### Requirement: A light held or eaten lights the world around the player every frame

A player SHALL carry a light while the selected hotbar slot holds an item
whose `pbd_core::light::carried_light` is some, or while the Glow effect runs.
The light's level SHALL be the source's placed value (`Material::emission`,
`GLOW_FLOWER_LEVEL`, or `glow_level` for the body glow), with no second copy
of any number. Its strength SHALL be `level / 14` (a torch's flickering in
step with its flame about that value), and it SHALL fall off by
`lamp_bake::falloff`. It SHALL be evaluated from the light's exact position
on the frame it is drawn, with no lag behind movement and no change at a cell
boundary. A dusk-lit source SHALL light only from dusk to dawn. An item in
the pack, or in an unselected slot, SHALL give no light.

#### Scenario: A held torch lights as a placed one

- **WHEN** a torch is held 3 m from a grass face and, separately, a torch is
  placed 3 m from the same face
- **THEN** the face is lit by the same amount in both cases, within the
  bake's rounding

#### Scenario: No step at a cell boundary

- **WHEN** a player carrying a light walks across a cell boundary at a
  constant speed
- **THEN** the light on a face 2 m ahead changes smoothly frame to frame, with
  no jump larger than the frame-to-frame change either side of the boundary

#### Scenario: In the pack it gives no light

- **WHEN** the selected slot holds a pickaxe and the pack holds a torch
- **THEN** the player carries no light

### Requirement: The carried light casts no shadow

The carried light SHALL NOT be occluded: no walk, cache or read-back SHALL
decide whether a face takes it. A face SHALL take it by the falloff and N·L
alone, so it reaches through a wall within its reach. Placed lamps SHALL keep
their shadows.

#### Scenario: A thin wall

- **WHEN** a player holds a torch beside a stone wall one cell thick
- **THEN** the ground on the far side of the wall, within the light's reach,
  is lit by the falloff from the torch's exact position, as it would be with
  no wall, and the wall's face turned away from the torch takes none of it

### Requirement: Everything lit like the terrain takes the carried light

The terrain, its trees and ground clutter, the water and every `LitByField`
thing (drops, craft, props, creatures, the hand and the held item) SHALL add
the carried light by the same falloff. The hand and held item SHALL take it
at a distance of at least `carried_hand_min_m`.

#### Scenario: A drop beside the player at night

- **WHEN** a player holding a lantern stands at midnight 2 m from a dropped
  item
- **THEN** the drop is lit on the side facing the lantern, as the ground
  under it is

### Requirement: A player's carried sources add as one light

The body glow and a held light SHALL add: one carried light per player at the
stronger source's position, its colour the sum of each source's colour times
strength. At most `carried_max_lights` carried lights SHALL be drawn: the
player's own, then the nearest others within `carried_others_m`.

#### Scenario: Glow and torch together

- **WHEN** a player with the Glow effect running holds a torch
- **THEN** one light is drawn at the torch, brighter than the torch alone by
  the glow's colour times strength
