# Equipment Specification

## Purpose
The tool in hand: one of the fishing rod, shovel, pickaxe and axe, in a tool
slot beside the ten item slots, changed with a hold of G and the wheel. What
the left button does is the tool's: the rod fishes and never digs.

## Requirements

### Requirement: One tool is in hand, in its own slot
The player SHALL hold exactly one tool (fishing rod, shovel, pickaxe or axe)
in a tool slot that is separate from the ten item slots. The tool slot SHALL
be drawn beside the item slots, with the held tool's own icon. A world with
no tool record SHALL be dealt all four tools with the fishing rod in hand. The
fishing rod SHALL NOT dig.

#### Scenario: A new world, or one saved before tools
- **WHEN** a world with no tool record is opened
- **THEN** the tool slot holds the fishing rod, the four tools are owned, and
  none of them occupies an item slot
  (`fish::tests::the_tool_slot_opens_on_the_rod_or_on_what_the_save_says`)

#### Scenario: The rod
- **WHEN** the rod is in hand
- **THEN** it is the one tool that does not dig, and the left button casts
  (`inventory::tests::the_tool_slot_starts_on_the_rod_and_changes_only_for_real`)

### Requirement: Holding G opens the tool picker; a tap still boards
On foot, holding G past the hold threshold SHALL open a picker beside the tool
slot that lists every owned tool by icon and name, with the held tool
highlighted. While it is open, the mouse wheel SHALL move the highlight and
SHALL NOT change the item slot or the camera zoom. Releasing G SHALL equip the
highlighted tool. A release of G before the threshold SHALL be a tap. Aboard
a craft, no picker SHALL open.

#### Scenario: Tap, hold and aboard
- **WHEN** G is tapped, held past the threshold, or held while not on foot
- **THEN** it is a tap, the picker opening and committing on release, or a tap
  (`controls::tests::g_is_a_tap_or_a_hold_and_never_both`)

#### Scenario: The wheel while the picker is open
- **WHEN** the wheel turns one step with the picker open
- **THEN** the highlight moves to the next owned tool and the selected item
  slot does not change
  (`desktop::equipment::tests::the_open_picker_takes_the_wheel`)

### Requirement: A tool change is saved at once
Changing the tool in hand SHALL be written to the durable save on the frame
it happens.

#### Scenario: Equip and reload
- **WHEN** a tool is put in hand and the world is reloaded
- **THEN** that tool is in hand
  (`saves::tests::catches_and_the_tool_in_hand_come_back`)
