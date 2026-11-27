# Power Specification

## ADDED Requirements

### Requirement: The Power branch opens with the iron age
The infrastructure tree SHALL have a Power branch whose first mission hangs
off "Build a grinder" with a hidden gate on the starter tree's "The iron
age", saved in the infrastructure tree's record, and whose missions unlock
the mills, lines and appliances as their rewards.

#### Scenario: A world that already has the iron age
- **WHEN** a world whose starter tree has "The iron age" done and whose infrastructure tree has "Build a grinder" done is opened for the first time with this branch
- **THEN** the Power branch's first mission is open and no mission in it is done

### Requirement: A windmill grinds only with wind and on a rise
A windmill SHALL be placed only where its cell stands at least 2 layers above
every cell within 3, and SHALL run its queue at a share of its speed set by
the wind at the mill: none below 3 m/s or above 20 m/s, rising from 3 to
8 m/s, full from 8 to 20 m/s. It SHALL make no electricity.

#### Scenario: A calm day
- **WHEN** a windmill holds 3 wheat in its queue and the wind at the mill is 2 m/s
- **THEN** no flour is made and the wheat stays in the queue

### Requirement: A water mill makes power from the river's flow
A water mill SHALL be placed only on a river cell flowing at 0.4 m/s or more,
and SHALL make `rated_kw * clamp(flow / full_flow, 0, 1)` while its sluice is
open, and nothing while it is shut or the river is frozen.

#### Scenario: The sluice shut
- **WHEN** a water mill on a river flowing at 1.5 m/s has its sluice shut
- **THEN** it makes 0 kW

### Requirement: Power is used as it is made
A power network SHALL store no energy. Each tick its supply SHALL be what its
mills make that tick; when demand is above supply every appliance on it SHALL
run at `supply / demand` of its speed, and when supply is zero every
appliance SHALL stop and every lamp on it SHALL be dark.

#### Scenario: The wheel stops at night
- **WHEN** the only water mill on a network with three lit lamps has its sluice shut
- **THEN** the three lamps are dark on the same tick and light again the tick the sluice opens

### Requirement: Poles join one network
A power pole SHALL join every pole within 4 cells, and a mill, appliance or
lamp SHALL join a pole within 1 cell; every joined run SHALL be one network
sharing its supply.

#### Scenario: A gap in the line
- **WHEN** two runs of poles are 5 cells apart
- **THEN** they are two networks and a mill on one powers nothing on the other
