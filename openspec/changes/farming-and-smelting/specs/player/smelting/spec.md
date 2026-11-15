## ADDED Requirements

### Requirement: A station runs recipes on world time
A station SHALL hold a queue of jobs, each a recipe queued at a world time.
Jobs SHALL run one after another, each starting when it was queued or when
the one before it finished, whichever is later, and finishing its recipe's
time after it starts. A job's progress SHALL be derived from world time, and
SHALL write nothing while it runs.

#### Scenario: Two jobs queued together
- **WHEN** two eight-second jobs are queued at the same moment
- **THEN** the first is finished after 8 s and the second after 16 s

### Requirement: Queuing a recipe spends its inputs, fuel included
Queuing a recipe SHALL take every input, fuel included, from the slots, and
SHALL be refused, taking nothing, when the slots do not hold them all or the
station's queue is full. The queue and the slots after it SHALL reach the save
in one line.

#### Scenario: Not enough coal
- **WHEN** the player queues mixed ore with four ore and one coal
- **THEN** nothing is queued and nothing leaves the slots

### Requirement: Finished jobs are collected into the slots
While a station's panel is open, each finished job at the head of its queue
whose outputs fit in the slots SHALL be removed and its outputs given by the
give order, in one saved line. A finished job whose outputs do not fit SHALL
wait in the queue.

#### Scenario: Smelting mixed ore
- **WHEN** a mixed-ore job has run its time and the panel is open
- **THEN** two copper ingots, an iron ingot and a tin ore are in the slots

### Requirement: The forge is a placeable station
The forge SHALL be a block placed from the slots and broken with a tool like
any block, giving itself back, except that breaking a forge with jobs in its
queue SHALL be refused. The use key on a forge SHALL open its panel.

#### Scenario: Breaking a busy forge
- **WHEN** the player breaks a forge whose queue holds a job
- **THEN** the forge stays and the job is kept

### Requirement: A dug ore cell yields mixed ore or coal
A generated ore cell SHALL yield coal where a hash of its cell and layer says
it is a coal seam (one in three) and mixed ore elsewhere. An ore block a
player placed SHALL yield mixed ore.

#### Scenario: Digging the same ore cell in two worlds of one seed
- **WHEN** the same generated ore cell is dug in two worlds
- **THEN** both yield the same thing
