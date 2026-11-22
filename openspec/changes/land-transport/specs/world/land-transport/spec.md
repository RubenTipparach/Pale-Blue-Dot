# World: land transport

## Purpose

Carts, wagons, ridden horses, cars and trucks on the road cells, and trains
on track laid on the hex grid: how each sits on what it runs on, and how each
is stored. Proposed for phase 5, but for the hand cart, the horse cart and
wheels off the road, which are phase 3 (owner, 2026-11-22); nothing here is
built.

## ADDED Requirements

### Requirement: Road vehicles fit the road cells

Every road vehicle (a ridden horse, a two-wheeled cart, a four-wheeled wagon,
an early car, a small truck) SHALL be no wider than 2.0 m, so that it fits
inside one road cell of 2.833 m. Its sizes SHALL be those of the M38 mockup,
beside a 1.55 m townsperson.

#### Scenario: The widest vehicle

- **WHEN** the small truck stands on a straight road cell
- **THEN** it is 1.9 m wide and both its sides are inside the cell

### Requirement: A road vehicle stands on the road's own surface

A road vehicle SHALL be placed from the road cells alone: each axle (or a
horse's two ends) at the height of the road under it, which is a bridge
deck's top, a sloped road cell's plane, or the cell's top. Its yaw SHALL come
from the line between its axles and its pitch from their heights. It SHALL
NOT need a GPU readback.

#### Scenario: On a climb

- **WHEN** a car's front axle is on a sloped road cell and its rear axle on
  the flat cell below
- **THEN** the car is pitched by the difference in their heights over its
  wheelbase

#### Scenario: On a bridge

- **WHEN** a cart crosses a timber bridge
- **THEN** its wheels stand on the deck's top, not on the ravine's floor

### Requirement: A bend is driven as an arc

A road vehicle SHALL drive a 60° bend in a road as an arc tangent to the
straights at the middle of the cell before the bend and the middle of the
cell after it (a radius of 4.9 m), or tangent halfway to those middles when
the next cell bends too (2.45 m). The arc SHALL stay on the road's cells.

#### Scenario: A single bend

- **WHEN** a car drives through one 60° bend between two straights
- **THEN** its drive line passes 0.76 m inside the bend cell's centre and
  never leaves the road cells

### Requirement: Vehicles slow on a climb

A road vehicle SHALL slow on a road cell that climbs: a horse to about 70% of
its pace and an early car to about 60% of its speed on a climb of one layer a
cell. It SHALL NOT stall there.

#### Scenario: A cart up the hill road

- **WHEN** a horse and cart go up a road climbing a layer a cell
- **THEN** they keep moving, at about 70% of their pace on the flat

### Requirement: Track lies on the hex grid

A straight piece of track SHALL run from cell centre to cell centre along
one of the six hex directions. A curve SHALL turn 60° from one cell centre to
another, `k` cells along the old heading and `k` along the new, with a radius
of `k * sqrt(3) * 2.833 m`; `k` SHALL be 2 or 3.

#### Scenario: A loop closes

- **WHEN** six curves of the same `k` are laid with straights of lengths
  a, b, c, a, b, c between them
- **THEN** the last curve ends on the first straight's starting cell centre,
  heading along it

### Requirement: Track climbs gently on a bed of whole cells

Track SHALL climb no steeper than one layer over six cells. It SHALL lie on a
ballast bed whose top follows the grade; the cells under the bed SHALL be cut
down to, or filled up to, the layer below the bed, as stored cell edits
naming their author, and the bed's bank SHALL make up the part of a layer
between.

#### Scenario: A grade across a terrace

- **WHEN** track is laid up a one-layer terrace over six cells
- **THEN** the low half runs on an embankment and the high half in a cutting,
  and every terrain cell under it is a whole number of layers tall

### Requirement: Vehicles, trains and track are stored

A road vehicle SHALL be a stored record with its ID, kind, pose in the body's
frame (`WorldPos`, f64), cargo, damage, owner and the ID of the animal in its
shafts. A train SHALL be an ordered list of its cars' IDs and its place on its
track. Track pieces SHALL be stored records. All of them SHALL be written on
the durable path, and a save without them SHALL open unchanged.

#### Scenario: A cart left on the road

- **WHEN** the player leaves a loaded cart on a road, its horse still in the
  shafts, and closes the game
- **THEN** on opening the world the cart is where it was, loaded, with the
  same horse in its shafts

### Requirement: Vehicles are lit as the ground is

Every vehicle, animal and piece of track in the world SHALL carry
`LitByField` and `LitLikeTerrain`. Its lamps (a cart's lantern, headlamps, an
engine's lamp, a coach's windows) SHALL be lights in the cell light field.

#### Scenario: A cart in a cutting at night

- **WHEN** a cart stands in a cutting at night with its lantern out
- **THEN** it is as dark as the cutting's floor beside it

### Requirement: Wheels are slow off the road

A wheeled vehicle SHALL move at its road speed times the factor of the
surface under its wheels: 1.0 on a gravel road, a bridge deck or a town's
lanes, 0.6 on a dirt track, 0.5 in a ford, and 0.25 anywhere else,
including a cell marked for paving and not yet paved. With its axles on two
surfaces it SHALL take the lower factor. The climb's slowing SHALL apply on
top. Walking SHALL NOT read this table.

#### Scenario: Off the road

- **WHEN** a horse cart leaves a gravel road for open grass on the level
- **THEN** it moves at a quarter of its speed on the road

#### Scenario: One wheel off

- **WHEN** a hand cart runs with one wheel on gravel and one on grass
- **THEN** it moves at a quarter of its road speed

### Requirement: Hand carts and horse carts are vehicles with a hold

A hand cart and a horse cart SHALL be made at the workbench, set down from
the pack as a vehicle record with a hold, and taken up again only when
empty. Letting go of a hand cart or stepping down from a horse cart SHALL
change who is with it, never whether it exists.

#### Scenario: A loaded cart left on a road

- **WHEN** the player lets go of a hand cart holding goods, walks away and
  reopens the world
- **THEN** the cart stands where it was with the same goods in its hold

### Requirement: A horse is bought and never changes the wild herds

A horse SHALL be bought for coins from a town's stable keeper as a kept
animal of a species no wild herd rolls, so that every existing world's wild
herds stay as they were. Hitching a horse to a horse cart SHALL write both
records in one transaction.

#### Scenario: An old world's herds

- **WHEN** a world saved before horses is opened by a build with them
- **THEN** its wild herds are the same animals in the same places

#### Scenario: Hitching

- **WHEN** the player hitches a bought horse to a horse cart and the game
  stops at that moment
- **THEN** on reopening, either both records name each other or neither does
