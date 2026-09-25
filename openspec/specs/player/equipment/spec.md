# Equipment Specification

## Purpose
The tool in hand: one of the fishing rod, shovel, pickaxe and axe, in a tool
slot beside the ten item slots, changed by holding G and turning the wheel. What
the left button does is the tool's: the rod fishes and never digs, and every
other tool breaks a block over a time set by the material and the tool, with
cracks on the block showing how far along it is.

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

### Requirement: Holding G opens the tool picker
On foot, holding G SHALL open a picker beside the tool slot that lists every
owned tool by icon and name, with the held tool highlighted. While it is
open, the mouse wheel SHALL move the highlight and SHALL NOT change the item
slot or the camera zoom. Releasing G SHALL equip the highlighted tool. Aboard
a craft or flying, G SHALL do nothing, and a menu opening SHALL close the
picker without equipping. G SHALL NOT board a craft: that is F.

#### Scenario: Hold, release, and not on foot
- **WHEN** G is held and released on foot, or pressed while not on foot
- **THEN** the picker opens on the press and commits on the release, or
  nothing happens
  (`controls::tests::g_holds_the_picker_open_only_on_foot`)

#### Scenario: The wheel while the picker is open
- **WHEN** the wheel turns one step with the picker open
- **THEN** the highlight moves to the next owned tool and the selected item
  slot does not change
  (`desktop::equipment::tests::the_open_picker_takes_the_wheel`)

### Requirement: Breaking a block takes time, set by the tool in hand
Taking a layer SHALL require holding the use button on the same layer for the
break time of its material. That time SHALL be the material's base time with
the right tool (the shovel for soft ground, the pickaxe for stone, rock and
ore), and a fixed multiple of it with any other tool. The fishing rod SHALL
NOT break any layer. Releasing the button or moving the aim SHALL reset the
progress. With the button still held, the next layer SHALL start after a
short pause. The times, the multiple and the pause SHALL be validated data
with units (`assets/config/dig.ron`).

#### Scenario: Dirt with the shovel and with the pickaxe
- **WHEN** the player holds the use button on dirt with the shovel, then on
  dirt with the pickaxe
- **THEN** the first takes 0.5 s and the second four times as long
  (`dig::tests::the_right_tool_takes_the_base_time_and_any_other_takes_four`)

#### Scenario: Letting go early
- **WHEN** the button is released, or the aim leaves the layer, before the
  break time
- **THEN** nothing is taken and the progress starts again from nought
  (`dig::tests::letting_go_or_looking_away_starts_again`)

#### Scenario: The rod
- **WHEN** the rod is in hand and the use button is held on stone
- **THEN** no layer changes
  (`dig::tests::the_rod_breaks_nothing_and_nothing_breaks_air_or_water`)

#### Scenario: Holding on
- **WHEN** a layer breaks with the button still held
- **THEN** the next one starts after the pause, and a fresh press starts at
  once (`dig::tests::the_next_block_waits_a_moment`)

### Requirement: The block being broken shows cracks that grow with progress
While a layer is being broken, its faces SHALL show a crack overlay chosen
from ten stages by the fraction of the break time elapsed. Each stage SHALL
contain every crack pixel of the stage before it. The overlay's pixels SHALL
use the terrain's own face UV mapping and its 32 px texel, so a crack pixel
covers exactly one block pixel. Nothing SHALL be drawn when nothing is being
broken.

#### Scenario: Halfway through
- **WHEN** a layer has been held for half its break time
- **THEN** stage 5 of 0 to 9 is drawn over it
  (`dig::tests::the_stage_follows_progress_in_tenths`)

#### Scenario: The stages grow
- **WHEN** the shipped stage images are compared in order
- **THEN** every crack pixel of stage `k` is a crack pixel of stage `k + 1`
  (`desktop::cracks::tests::the_stages_grow_and_match_the_atlas_texel`)

#### Scenario: The overlay wraps the layer
- **WHEN** the overlay is built for a cell and a layer
- **THEN** it is a closed prism around exactly that layer with every face
  turned out (`desktop::cracks::tests::the_prism_wraps_the_layer_and_faces_out`)

### Requirement: A tool change is saved at once
Changing the tool in hand SHALL be written to the durable save on the frame
it happens.

#### Scenario: Equip and reload
- **WHEN** a tool is put in hand and the world is reloaded
- **THEN** that tool is in hand
  (`saves::tests::catches_and_the_tool_in_hand_come_back`)
