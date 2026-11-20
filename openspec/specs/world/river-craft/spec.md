# River Craft Specification

## Purpose
Craft on the rivers, as M34 River Craft designs them (`river-craft`):
the raft rowed with two oars on one beat, one anchor per boat that sets and
holds, the boat's own menu for its hold, anchor and lantern, and the raft's
lantern lit at night. What is here is built and tested; the current from the
cells, lines between craft, the tug, logs, booms and nets are still in the
`river-craft` change.

## Requirements

### Requirement: A raft is rowed in strokes on one beat
A raft SHALL be rowed with two oars on pins and no pole, each stroke in five
stages (reach, catch, drive, release, recovery) timed by one stroke clock
shared by both oars. The raft's push SHALL be the drag of each oar's blade
through the water while it is under, at the blade, so that one oar alone
turns the raft; nothing SHALL push off the bed. An oar whose key goes down
mid-stroke SHALL join at the next reach, and one whose key comes up SHALL
finish its stroke; no pose SHALL jump. The drawing (the oars, the rower's
body and arms, the first-person hands) SHALL read the stroke and SHALL NOT
be read by any rule; the rower's body SHALL be drawn only from the chase
view and the first-person hands only in the seat.
(`pbd_core::vehicle::oars` tests; `vehicle::tests::a_tube_and_a_raft_float_upright_and_paddle_ahead`)

#### Scenario: Rowing both oars
- **WHEN** the player holds W on a raft in still water
- **THEN** both blades enter on the same catch, once a stroke each, and the
  raft makes way ahead

#### Scenario: Rowing one oar
- **WHEN** the player holds A
- **THEN** only the right oar rows, and the raft turns left
  (`vehicle::tests::a_raft_rowed_on_one_oar_turns_away_from_it`)

#### Scenario: Rowing in the shallows
- **WHEN** a blade comes down where the bed is shallower than its stroke
- **THEN** it rests on the bed and pushes nothing off it; only the water
  above the bed pulls on it
  (`vehicle::tests::an_oar_over_a_shallow_bed_rests_on_it_and_pushes_nothing_off_it`)

#### Scenario: Letting go mid-stroke
- **WHEN** the row key comes up during the drive
- **THEN** the oar finishes its stroke and comes to rest with no jump
  (`oars::tests::an_oar_joins_at_the_next_reach_and_rests_without_a_jump`)

### Requirement: A set anchor holds the craft
A boat that anchors SHALL have one anchor, at its bow. Dropped, it SHALL fall
to the bed and the rode SHALL stop paying out when it touches, unless a
longer rope is set in the boat's menu, which SHALL then be veered out. It
SHALL set within two seconds. A set anchor SHALL hold: neither the current,
nor rowing, nor shortening the rope SHALL move it, and the craft SHALL move
only within its swing on the rode. Weighing SHALL be the only way to free
it. The inner tube SHALL have no anchor. A rode read from a save SHALL be an
anchor set and holding on the length it was saved with.

#### Scenario: The rope stops at the bed
- **WHEN** an anchor is dropped with its rope set to the bed
- **THEN** the rode stops paying out when the anchor touches the bed
  (`vehicle::tests::a_dropped_anchors_rode_stops_at_the_bed`)

#### Scenario: Rowing against a set anchor in a current
- **WHEN** a raft with its anchor set rows in a current
- **THEN** it swings no further than its rode and the anchor does not move
  (`vehicle::tests::a_set_anchor_holds_a_rowing_raft_and_only_weighing_frees_it`)

#### Scenario: Lengthening and shortening the rope
- **WHEN** a holding anchor's rope is set to 12 m, then back to the bed
- **THEN** the rode is veered to 12 m and the boat drops back on it, then
  hauled in, and the anchor still holds

#### Scenario: Weighing anchor
- **WHEN** the anchor is weighed
- **THEN** the rode is hauled in, the anchor comes aboard and the craft is
  free

### Requirement: A boat has a menu for its hold, anchor and lantern
A boat with an anchor SHALL have a menu, opened with C while aboard or while
on foot within its boarding reach, and closed with C or Escape. It SHALL be
the pack with the boat's panel beside its hold: the boat's kind and whether
water is aboard; its anchor's state with drop or weigh; the rope's length
from "To the bed" up to the rode the boat carries; and, on a boat with a
lantern, the lantern on or out. A stack SHALL move between the hold and the
player's slots by the pack's own rule. Its text SHALL be plain labels. An
inner tube SHALL have no menu.

#### Scenario: Putting the lantern out
- **WHEN** a raft's lantern is put out in the menu at night
- **THEN** it adds no light until it is put back on
  (`vehicles::lamps::tests::the_rafts_lantern_lights_at_night_only_and_no_other_kind_has_one`)

### Requirement: The boat's settings survive saves
A boat's rope length and whether its lantern is out SHALL be kept on the
boat's own record through the durable path when they change, each written
only when set. A world saved before them SHALL open with every rope "to the
bed", every lantern on, and every anchored boat's rode at the length it was
saved with.

#### Scenario: Reopening a world
- **WHEN** a boat's rope is set to 14 m and its lantern put out, and its
  record is written and read back
- **THEN** the rope is still 14 m and the lantern still out, and a record
  from before reads with neither
  (`vehicle::tests::the_boats_settings_survive_a_save_and_old_records_read_unchanged`)

### Requirement: A raft carries its own lantern, lit at night
Every raft SHALL come with a lantern fitted to its bow post, needing no
crafting or placing. Unless it is put out in the boat's menu, it SHALL be
lit from dusk to dawn on the town lamps' switch and, while lit, SHALL light
what is near it (the raft, the water round it, the oars and the rower's
hands) with the placed post lantern's light, through the moving lights the
terrain, the water and every field-lit mesh add. No other kind SHALL carry
one. The raft SHALL stay lit as the terrain is.

#### Scenario: Day
- **WHEN** the dusk lamps are out
- **THEN** the raft's lantern adds no light
  (`vehicles::lamps::tests::the_rafts_lantern_lights_at_night_only_and_no_other_kind_has_one`)
