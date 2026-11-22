# Log Drives Specification

## Purpose
Logs on the water (`log-drives`): a log thrown into a river is carried by
the water's own current and comes to rest where the water stops. What is
here is built and tested; loading a raft is still in the `log-drives`
change.

## Requirements

### Requirement: Logs in a river flow with it and stop where it stops
A log placed in flowing water SHALL be carried by the water's flow, and SHALL
come to rest where the water stops flowing or is too shallow to float it. A
resting log in water that flows again SHALL float again. Each log SHALL be a
stored record, written when it is made, when it stops or starts, at most every
five seconds while it moves, and when it is taken, so a reload finds it.
(`log_drive::tests`; `logs::tests::a_log_comes_back_where_it_was_and_a_taken_one_does_not`)

#### Scenario: A log dropped upstream
- **WHEN** a log is dropped into a river
- **THEN** it drifts downstream with the current and stops in the still water where the river ends
  (`log_drive::tests::a_log_drifts_down_the_river_and_rests_where_it_stops_flowing`)

#### Scenario: Shallows and banks
- **WHEN** a drifting log reaches water shallower than its draught, or the bank
- **THEN** it comes to rest there
  (`log_drive::tests::a_log_grounds_in_the_shallows_and_on_a_bank`)
