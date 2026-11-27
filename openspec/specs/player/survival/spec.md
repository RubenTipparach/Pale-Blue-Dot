# Survival Specification

## Purpose
What it takes to stay alive in a Survival world, as the game does it today
(`openspec/changes/survival-mode`, moved requirement by requirement as each
is built and tested): the world's mode, health and death with the dropped
pack, stamina with its winded limits, the predators' hunt, townsfolk who
fight back, and weapons that hit on the swing. The rest of the capability is
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

### Requirement: Predators hunt in Survival
In Survival, predators SHALL hunt the player. Each SHALL give a visible and
audible warning before its first attack, SHALL flee when badly hurt, and
SHALL be bolder at night. In Explorer, predators SHALL keep their earlier
behaviour and never attack the player.


Pinned by `pbd_core::survival::predator::tests::{it_notices_within_range_and_further_at_night,
a_stalk_turns_to_a_warning_close_in,
the_warning_is_held_at_least_one_and_a_half_seconds_before_the_first_attack,
a_lunge_that_reaches_bites_for_the_species_damage, a_rewarn_is_short,
struck_it_fights_back_at_once, below_a_third_of_its_health_it_flees_then_calms,
out_of_reach_a_while_it_gives_up, a_downed_player_is_left_alone,
an_explorer_fox_never_attacks}` and `pbd_app::husbandry::hunt::tests::{a_survival_fox_stalks_warns_and_bites_and_an_explorer_fox_never_does,
the_lunge_gathers_springs_and_lands_and_the_warning_growls}` (the warning is
the crouch and the `predator_growl` cue).

#### Scenario: A fox at night
- **WHEN** a Survival player meets a red fox at night
- **THEN** it stalks, warns, and attacks, and flees once it is badly hurt

#### Scenario: The same fox in Explorer
- **WHEN** an Explorer player meets a red fox
- **THEN** it never attacks them

### Requirement: Townsfolk fight back
In Survival, a struck townsperson SHALL fight back or flee, nearby guards
SHALL come to fight, and the town SHALL remember the fight in the player's
standing with it. In Explorer, townsfolk SHALL NOT be harmed.


Pinned by `pbd_core::survival::standing::tests::*` and
`pbd_app::combat::tests::{a_struck_villager_brings_the_guards_and_lowers_the_towns_standing,
a_swing_at_a_townsperson_in_explorer_does_nothing}`.

#### Scenario: Striking a villager
- **WHEN** a Survival player strikes a villager in a town
- **THEN** the guards come for the player, and the town's standing with them
  drops

### Requirement: Weapons are crafted in tiers and hit on the swing
In Survival, spears, a bow with arrows, hunting axes and swords SHALL be
crafted from the existing recipe tables (by hand, at the workbench, and from
the forge's ingots), at stone, bronze and iron tiers, locked behind their
missions. A weapon's hit SHALL land at the swing's hit moment; a spear SHALL
also be thrown, and an arrow SHALL fly under the body's own gravity in the
floating origin. A fired arrow or a thrown spear that survives SHALL lie
where it stopped until picked up. In Explorer these recipes SHALL NOT exist.


Pinned by `pbd_core::survival::{melee, weapons}::tests::*`,
`pbd_app::combat::tests::{the_hit_lands_at_the_swings_hit_moment,
a_tools_drawn_impact_lands_on_the_rules_hit_frame, a_thrown_spear_lands_as_a_drop,
a_weapon_wears_per_hit_and_breaks}`, `pbd_app::held::weapon_swing::tests::*`,
and the recipe tests `station::tests::survival_recipes_are_offered_only_in_a_survival_world`
and `every_survival_unlock_in_the_missions_file_is_a_recipe`.

#### Scenario: An arrow drops with distance
- **WHEN** a Survival player looses an arrow level at full draw
- **THEN** it falls under the planet's gravity along its flight, and where it
  lands it can be picked up again or is broken
