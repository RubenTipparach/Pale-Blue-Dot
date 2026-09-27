## MODIFIED Requirements

### Requirement: The player carries ten slots
The player SHALL have ten inventory slots, each holding either nothing or a
stack of one item kind with a count, and a pack of thirty more slots of the
same kind. A hotbar slot SHALL be selectable, exactly one SHALL be selected at
a time, and the selection SHALL wrap in both directions. A give SHALL fill
matching stacks with room first, the hotbar before the pack, then the first
empty hotbar slot, then the first empty pack slot.

#### Scenario: Taking a second stack of the same block
- **WHEN** a block is given and a slot already holds that block below its limit
- **THEN** it joins that stack rather than taking a new slot

#### Scenario: A stack at its limit
- **WHEN** a block is given and every matching stack is at its limit
- **THEN** it takes the first empty hotbar slot, or the first empty pack slot
  when the hotbar is full
- **AND** if there is none, the give is refused rather than silently dropped

#### Scenario: Emptying a slot
- **WHEN** the last item is taken from a stack
- **THEN** the slot holds nothing, rather than a stack of zero

## ADDED Requirements

### Requirement: The pack opens as a grid
The pack SHALL open and close with I on foot, and close with Escape. It
SHALL be drawn as a grid of the same bordered squares as the hotbar, three
rows of ten, with the hotbar's ten under them. While it is open, the pointer
SHALL be free and walking input SHALL be ignored, and the world SHALL keep
running. A stack picked up SHALL stay in its slot until it is put down, and
each completed move SHALL enter the durable log as one record.

#### Scenario: Moving a stack
- **WHEN** a stack is clicked in the pack and then an empty hotbar slot is
  clicked
- **THEN** the stack is in the hotbar slot and the pack slot is empty, and the
  change is in the durable log

#### Scenario: Closing with a stack in hand
- **WHEN** the pack is closed while a stack is held on the pointer
- **THEN** the stack goes back to the slot it came from

#### Scenario: Taking half, and sending across
- **WHEN** a stack is right-clicked and then an empty slot is clicked
- **THEN** the larger half is in the empty slot and the rest where it was
- **WHEN** a hotbar stack is shift-clicked
- **THEN** it goes into the pack, onto matching stacks first

### Requirement: A dug block drops into the world and is picked up
A dug block SHALL drop into the world as a floating block of its material at
the dug cell, and SHALL be drawn to the player and picked up when the player
comes near it. A drop that does not fit SHALL stay floating in the world. A
drop SHALL last 300 s of world time and SHALL survive a save and a load
within that time.

#### Scenario: Digging beside the player
- **WHEN** a block is dug within reach and the pack has room
- **THEN** it drops, is pulled in, and lands in the hotbar or the pack

#### Scenario: Digging with everything full
- **WHEN** the hotbar and the pack are full and hold no stack of the dug block
  with room
- **THEN** the cell is dug, and the block floats where it was cut until there
  is room or its time runs out

#### Scenario: A drop outlives a quit
- **WHEN** the world is saved and loaded with a drop floating in it
- **THEN** the drop is where it was, with the time it had left
