# Platform: mockup playtests

## ADDED Requirements

### Requirement: Every mockup has a committed playtest

Each mockup on the tracker SHALL have a playtest file in
`tools/mockups/playtests/` that lists its milestones. Each milestone SHALL
list its steps, its assertions and its recorded shots, and SHALL name the
requirement it demonstrates.

#### Scenario: Checking a mockup

- **WHEN** `tools/mockup_playtest.js industry.html` is run
- **THEN** every milestone in `industry.json` is played in a fresh page, every
  assertion is checked, and the run exits non-zero on a failure, naming the
  milestone, the assertion and the value read

### Requirement: The standing mockup rules are checked on every page

The runner SHALL check, on every page and without a line in its playtest
file:
- that the page raises no errors;
- that the tracker link is near the top;
- that only one WebGL view draws at a time, and a tab's renderer is made when
  the tab first opens;
- that nothing moves while idle outside declared world-motion regions;
- that lamps light their ground at night;
- that Run starts off.

A page SHALL be exempt from a rule only by a named exemption that the gate
page prints.

#### Scenario: A page that spins on its own

- **WHEN** a page turns a preview on a turntable with Run off
- **THEN** the stillness check fails, naming the page and the share of pixels
  that changed in 2 s

### Requirement: A recorded playtest is the same playtest

Record mode SHALL play the same steps as check mode, with video on. It SHALL
write a video per milestone and a captioned contact sheet, and SHALL mark a
shot whose assertion failed. A recording made without a GPU SHALL say so in
its first frame.

#### Scenario: Recording the batch

- **WHEN** `tools/mockup_playtest.js --all --record out/` runs in a cloud session
- **THEN** each mockup has a video per milestone, labelled as rendered without
  a GPU, and a contact sheet with one captioned frame per shot
