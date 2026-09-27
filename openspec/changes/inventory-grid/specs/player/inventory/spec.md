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
The pack SHALL open and close with I, and close with Escape. It SHALL be
drawn as a grid of the same bordered squares as the hotbar, ten wide, under
the hotbar. While it is open, the pointer SHALL be free and walking input
SHALL be ignored, and the world SHALL keep running.

#### Scenario: Moving a stack
- **WHEN** a stack is clicked in the pack and then an empty hotbar slot is
  clicked
- **THEN** the stack is in the hotbar slot and the pack slot is empty, and the
  change is in the durable log

#### Scenario: Closing with a stack in hand
- **WHEN** the pack is closed while a stack is held on the pointer
- **THEN** the stack goes back to the slot it came from

### Requirement: Nothing dug is lost to a full pack
A dig SHALL be refused, before it is saved, when the block it yields cannot
be carried, and the HUD SHALL say why.

#### Scenario: Digging with everything full
- **WHEN** the hotbar and the pack are full and hold no stack of the dug block
  with room
- **THEN** the cell is not dug, nothing is written to the log, and the HUD
  says the pack is full
