# Survival Specification

## Purpose
What it takes to stay alive in a Survival world, as the game does it today
(`openspec/changes/survival-mode`, moved requirement by requirement as each
is built and tested): the world's mode, health and death with the dropped
pack, falls, hunger, stamina with its winded limits, body temperature, tool
wear, the predators' hunt, hunting, townsfolk who fight back, and weapons
that hit on the swing. The rest of the capability is
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

### Requirement: Falling hurts by impact speed
In Survival, landing SHALL cause damage when the speed at impact exceeds a
safe speed, growing with the excess. The safe speed SHALL be computed in the
body's own gravity, so the same fall height hurts less on a lower-gravity
body. Landing in water deep enough to swim in SHALL cause no fall damage.

Pinned by `pbd_core::survival::fall::tests::{a_two_layer_step_never_hurts, a_twenty_layer_fall_takes_about_80, the_same_fall_hurts_less_on_a_half_gravity_body, a_fall_into_deep_water_does_not_hurt, impact_speed_is_along_the_bodys_up}`.

#### Scenario: A short drop
- **WHEN** the player steps off a ledge two layers high
- **THEN** they take no damage

#### Scenario: A cliff
- **WHEN** the player falls from twenty layers onto rock
- **THEN** they take damage that a fall from ten layers does not exceed

#### Scenario: Into a lake
- **WHEN** the player falls from twenty layers into deep water
- **THEN** they take no damage

### Requirement: Hunger empties with play and meals refill it
In Survival, hunger SHALL empty slowly with play time only, never while the
game is closed. A meal SHALL refill it as well as giving its buff. While
hunger is empty, health SHALL NOT refill and SHALL drain slowly.

Pinned by `pbd_core::survival::hunger::tests::{standing_about_a_full_bar_lasts_75_minutes_and_shivering_burns_faster, a_one_star_meal_fills_its_own_amount, a_two_star_meal_fills_115_percent, a_three_star_meal_fills_130_percent}` and `pbd_core::survival::body::tests::{time_with_the_game_closed_changes_nothing, hunger_drains_at_the_idle_rate_and_health_refills, starving_drains_health_a_point_every_ten_seconds, meals_fill_food_and_stew_fills_water}`.

#### Scenario: Closing the game hungry
- **WHEN** a player quits with hunger nearly empty and returns a day later
- **THEN** hunger is where they left it

#### Scenario: Eating
- **WHEN** a hungry player eats a cooked dish
- **THEN** hunger rises by the dish's amount and its buff starts

### Requirement: Tools wear out in Survival
In Survival, each owned tool SHALL carry the uses it has left, starting at
the number its tier sets. A finished action with the tool (a block broken, a
tree felled, a cell tilled, a fish landed, a hit landed) SHALL spend one use,
or two for a block of a higher tier than the tool.  At zero uses the tool SHALL become a broken
tool in the same slot that works as bare hands, and repairing it at a
workbench or with a town's smith SHALL restore all its uses. In Explorer,
tools SHALL NOT wear.

Pinned by `pbd_core::survival::wear::tests::*` and `pbd_app::survival::tests::{a_tool_wears_breaks_survives_a_reload_and_is_repaired, a_workbench_repair_takes_half_the_head_of_the_tools_recipe, a_town_repair_takes_coins_and_restores_the_tool, a_smith_repairs_for_the_share_worn, a_broken_weapon_is_repaired_at_the_workbench_for_half_its_head, a_town_repairs_a_weapon_for_coins_and_the_repair_survives_a_reload}`.

#### Scenario: A stone pickaxe wears out
- **WHEN** a Survival player breaks stone with a stone pickaxe until its
  uses reach zero
- **THEN** the pickaxe stays in its slot as a broken pickaxe, digs as bare
  hands do, and is still there after the world is reopened

#### Scenario: Repair at the workbench
- **WHEN** the player repairs a broken stone pickaxe at the workbench with
  half its head materials
- **THEN** it has its full uses again and keeps its tier

#### Scenario: A missed swing is free
- **WHEN** the player swings at nothing, or stops a dig half way
- **THEN** the tool spends no use

#### Scenario: An old save
- **WHEN** a world saved before tool wear is opened in Survival
- **THEN** every tool it holds has its full uses

#### Scenario: Explorer never wears
- **WHEN** an Explorer player breaks a thousand blocks with one pickaxe
- **THEN** the pickaxe has not changed

### Requirement: Body temperature follows the weather at the player
In Survival, the player's body temperature SHALL move toward a target set by
how the place feels: the atmosphere's own air temperature at the player
(which carries the biome, season, hour and altitude), the sun unless shaded,
humid heat, wind chill, rain, being wet or in water, a nearby fire,
shelter, and what the player wears. Too hot SHALL use water faster; too cold
SHALL slow stamina's refill and burn food faster; past either end, health
SHALL drain. The shade test SHALL be made on the CPU from the terrain, never
read back from the GPU.

Pinned by `pbd_core::survival::temperature::tests::*`, `pbd_core::survival::water::tests::heat_uses_water_faster_and_cold_slower`, `pbd_core::survival::stamina::tests::cold_halves_the_refill`, `pbd_core::survival::hunger::tests::standing_about_a_full_bar_lasts_75_minutes_and_shivering_burns_faster`, `pbd_core::survival::body::tests::past_either_end_health_drains` and `pbd_app::survival::tests::a_hat_and_a_cloak_change_feel_and_are_worn_from_the_hand`.

#### Scenario: Shade at noon
- **WHEN** a Survival player stands in a desert at noon, then in the shade
  of a rock
- **THEN** the body heats in the sun and settles cooler in the shade

#### Scenario: A cloak and a fire
- **WHEN** a Survival player stands in a tundra night, then puts on a wool
  cloak, then stands beside a campfire
- **THEN** each step leaves the body warmer than the one before

### Requirement: Wild game is hunted for meat, hide and bone
In Survival, wild game SHALL take damage and flee, and a kill SHALL drop its
species' meat and bones, and its hide whole when the carcass is dressed with
a hunting axe. Raw meat SHALL cook at the campfire, hide SHALL become
leather at a tanning rack, and bone SHALL make arrows and fertiliser. A
killed herd member SHALL come back after days of play, and the kill SHALL be
kept in the save. Protected species, and animals a player keeps or bought,
SHALL NOT be harmed.

Pinned by `pbd_core::survival::hunting::tests::{a_woolback_hunted_and_dressed, another_blade_takes_the_hide_about_half_the_time, protected_and_kept_animals_are_refused, a_herd_member_is_back_after_three_days_and_not_before_a_reload}` and `pbd_app::husbandry::hunt::tests::{wild_game_gives_meat_hide_and_bone_and_each_has_its_use, a_kill_names_its_herd_member, explorer_and_kept_animals_are_never_struck}`.

#### Scenario: A woolback hunted and dressed
- **WHEN** a Survival player kills a wild woolback and dresses it with a
  hunting axe
- **THEN** raw meat, a hide, wool and bones lie at the kill, and the herd has
  its member back after three world days

#### Scenario: A protected otter
- **WHEN** a Survival player strikes a river otter
- **THEN** it takes no damage
