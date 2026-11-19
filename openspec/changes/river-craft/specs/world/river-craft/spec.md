## ADDED Requirements

### Requirement: Floating craft are carried by the river's current

A hull on flowing water SHALL feel drag toward the water's velocity at each
of its float points, split along and across the hull, so that it drifts with
the current, turns where the current differs across it, and is pushed
downhill by the water's slope. The current SHALL be derived from the flowing
water's cells and SHALL NOT be saved.

#### Scenario: A raft left to drift

- **WHEN** a raft is left with no input on a river running at 1 m/s
- **THEN** it comes up to about the current's speed and follows the channel

#### Scenario: Shear and an eddy

- **WHEN** a hull drifts past a rock that stands in the current
- **THEN** it turns as the current differs along it, and in the rock's lee
  the eddy carries it back upstream toward the rock

#### Scenario: Still water

- **WHEN** a hull floats on a level pool or a source
- **THEN** the current there is zero and the hull does not drift

### Requirement: A line between two craft pulls only when taut

A tow line between two craft SHALL pull nothing while its ends are closer
than its length and, when taut, SHALL pull both ends toward each other with
the game's mooring stiffness `min(40 m, 4000)` N/m and damping
`1.6 sqrt(k m)`, m the ends' reduced mass. A line SHALL be cast off and made
fast again within reach.

#### Scenario: Taking up the tow

- **WHEN** a tug with a slack line to a barge goes ahead
- **THEN** the line comes taut, the barge follows, and easing off lets the
  line go slack again

### Requirement: An anchor drags until it holds, by bed

An anchor on the bed SHALL hold up to the bed's holding force times how far
it has set times the lift of its rode, and SHALL drag toward its craft while
the rode's pull exceeds that. Mud SHALL set within about a metre of dragging,
sand SHALL never fully set, and rock SHALL hold once a fluke catches. A rode
hauled short SHALL break the anchor out so it can be weighed.

#### Scenario: Sand against mud

- **WHEN** the same skiff anchors in the same current on sand and on mud
- **THEN** on sand the anchor keeps dragging slowly, and on mud it bites and
  the skiff lies still to its rode

#### Scenario: Weighing anchor

- **WHEN** the rode of a holding anchor is hauled in
- **THEN** the craft is pulled up toward the anchor, the anchor breaks out
  and comes aboard

### Requirement: Logs drive, jam and are caught by a boom

A log in water SHALL float and drift as a hull. Logs SHALL collide with
each other, the banks and rocks, and SHALL jam where they cannot pass; a
pried key log SHALL be turned end-on and freed. A boom SHALL hold logs
against the current, and logs held at it SHALL be able to be bound into a
bundle that a craft can tow.

#### Scenario: A jam broken

- **WHEN** logs stall crosswise at a narrows and the key log is pried
- **THEN** the key log turns end-on to the flow and the drive moves on

#### Scenario: Bundling at the boom

- **WHEN** four or more logs are held at a boom and the player bundles them
- **THEN** they become one bundle that a craft can hitch and tow

### Requirement: A net drags, catches and is hauled

A net made fast to a boat SHALL be dragged by the water, and the sum of that
drag SHALL pull on the boat. Fish swept inside its cork line or swimming
into its twine SHALL be caught and SHALL add to its drag. Hauling SHALL draw
the net in and land the catch.

#### Scenario: A full net slows the boat

- **WHEN** a boat rowing with a net out sweeps through a school
- **THEN** the caught fish ride in the net, the net's pull grows and the
  boat slows; hauling in lands them

### Requirement: River craft survive saves

Craft, lines, anchors, booms and bundles SHALL each be a durable record, and
a world saved before river craft existed SHALL open unchanged.

#### Scenario: An old save

- **WHEN** a world saved before this change is opened
- **THEN** it opens with no river craft records and nothing else changed

### Requirement: A raft is rowed in strokes on one beat

A raft SHALL be rowed with two oars on pins, each stroke in five stages
(reach, catch, drive, release, recovery) timed by one stroke clock shared by
both oars. The raft's push SHALL be the drag of each oar's blade through the
water while it is under, at the blade, so that one oar alone turns the raft.
An oar whose key goes down mid-stroke SHALL join at the next reach, and one
whose key comes up SHALL finish its stroke; no drawn pose SHALL jump. The
drawing (arms, body, splash, swirls) SHALL read the stroke and SHALL NOT be
read by any rule.

#### Scenario: Rowing both oars

- **WHEN** the player holds W on a raft in still water
- **THEN** both blades enter on the same catch, the raft surges through each
  drive and glides through each recovery, at one steady stroke rate

#### Scenario: Rowing one oar

- **WHEN** the player holds A
- **THEN** only the right oar rows, and the raft turns left

#### Scenario: The pole past its reach

- **WHEN** the player poles where the bed is deeper than the pole's reach
- **THEN** the pole does not plant and pushes nothing

### Requirement: A raft carries its own lantern, lit at night

Every raft SHALL come with a lantern fitted to it, needing no crafting or
placing. It SHALL be lit from dusk to dawn on the town lamps' switch and,
while lit, SHALL light what is near it (the raft, the water round it, the
oars and the rower's hands) through the light field's lamp term. The raft
SHALL stay lit as the terrain is.

#### Scenario: Rowing at night

- **WHEN** a raft is rowed after dusk far from any town
- **THEN** its deck, the water round it and the rower's hands are lit by its
  lantern, and the water beyond the lantern's range stays dark

#### Scenario: Day

- **WHEN** the clock passes dawn
- **THEN** the raft's lantern goes out and adds no light
