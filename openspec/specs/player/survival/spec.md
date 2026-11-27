# Survival Specification

## Purpose
What it takes to stay alive in a Survival world, as the game does it today
(`openspec/changes/survival-mode`, moved requirement by requirement as each
is built and tested): the world's mode, health and death with the dropped
pack, and stamina with its winded limits. The rest of the capability is
still the change's.

## Requirements

### Requirement: A world is Explorer or Survival
A world SHALL have a mode, Explorer or Survival, chosen when it is made and
stored with it. In Explorer, no rule in this capability SHALL apply. A world
saved before modes existed SHALL open as Explorer.

Pinned by `pbd_core::survival::mode::tests::{an_old_save_opens_as_explorer,
a_survival_world_reopens_as_survival}`,
`saves::format::tests::an_old_save_opens_as_explorer_and_survival_reopens_as_survival`,
`pbd_app::survival::tests::an_old_save_opens_as_explorer_and_has_no_bars`
and `pbd_server::tests::an_explorer_world_has_no_vitals_and_an_old_save_reads_explorer`.

#### Scenario: An old save
- **WHEN** a world saved before this change is opened
- **THEN** it opens as Explorer, with no health, hunger or breath shown, and
  plays as before

#### Scenario: A new Survival world
- **WHEN** a player makes a world and picks Survival
- **THEN** the world opens with full health, hunger and breath, and keeps
  Survival every time it is opened

### Requirement: Health and death
In Survival, the player SHALL have health that damage lowers and that
refills slowly while hunger is not empty. At zero health the player SHALL
die, their pack SHALL drop at that place as a bag that stays until picked up,
and they SHALL wake at their bed, or at the world's spawn if they have none.

Pinned by `pbd_app::survival::tests::{a_death_drops_every_item_in_one_bag_that_survives_a_reload_and_is_picked_up_whole,
what_does_not_fit_stays_in_the_bag, a_death_wakes_at_a_laid_bedroll,
a_long_dive_drains_health_and_surfacing_refills_breath}`,
`pbd_server::tests::{a_long_dive_drains_health_and_surfacing_refills_breath,
vitals_are_journaled_on_a_hit_and_every_few_seconds_and_read_back}` and the
core's `survival::health` and `survival::body` tests.

#### Scenario: Dying with a full pack
- **WHEN** the player dies carrying items
- **THEN** a bag holding all of them lies where they died, it is still there
  after the world is closed and reopened, and walking into it returns the
  items

### Requirement: Stamina is spent by effort and refilled from food
In Survival, the player SHALL have a stamina bar that sprinting, swimming,
treading water and climbing spend, and a jump spends a little. Tool work
SHALL NOT spend it. After a short pause it SHALL refill on its own, and
every point refilled SHALL cost food: less a point while the bar is at or
below half than above it. Points spent sprinting or swimming SHALL also
burn food while they are spent, and SHALL cost more food to refill than
other points. In Explorer there SHALL be no stamina bar.

Pinned by the core's `survival::stamina::tests::*` and
`pbd_app::survival::tests::a_sprint_drains_the_bar_and_rest_refills_it`.

#### Scenario: A minute of sprinting
- **WHEN** a Survival player with full stamina and 80 food sprints for 60 s
  and then walks
- **THEN** the bar falls to about 40 and is not empty, refills in a few
  seconds, and food falls by about 6.3 in all, against about 1.6 for the
  same time walking

#### Scenario: The bottom half is cheaper
- **WHEN** the bar refills from 0 to 50, and then from 50 to 100
- **THEN** the first half costs less food than the second

#### Scenario: Tool work is free
- **WHEN** a Survival player digs and chops for a minute without sprinting
- **THEN** stamina stays full

### Requirement: An empty bar leaves the player winded, never drowned
In Survival, at zero stamina the player SHALL be winded until a quarter of
the bar is back: no sprint, slower swimming and lower jumps. A winded
swimmer SHALL float with the head out of the water.

Pinned by `pbd_core::survival::stamina::tests::empty_is_winded_until_a_quarter_is_back`,
`body::tests::a_winded_swimmer_floats_and_breathes` and
`pbd_app::survival::tests::a_winded_swimmer_floats_and_cannot_sprint_or_climb_out`.

#### Scenario: Winded far from shore
- **WHEN** a Survival player's stamina runs out while swimming on the surface
- **THEN** they float with the head above the water, can still swim slowly,
  and the bar refills once they stop swimming
