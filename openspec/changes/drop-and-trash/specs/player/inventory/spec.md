## ADDED Requirements

### Requirement: A player can drop or destroy what they carry
A player SHALL be able to drop the held hotbar stack with Q, drop a stack by dragging it off the pack onto the background, and destroy a stack by putting it into the pack's trash slot. A dropped stack SHALL float as a pickup beyond the pickup magnet, and both SHALL be saved on the durable path.

#### Scenario: Drag off the pack
- **WHEN** a player in the pack picks up 99 sand and clicks the background
- **THEN** the sand is gone from the pack and floats in front of them as a pickup

#### Scenario: Trash
- **WHEN** a player puts 99 soil into the trash slot
- **THEN** the soil is gone and the save's next load agrees
