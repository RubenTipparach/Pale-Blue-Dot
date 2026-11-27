# Fire Specification

## Purpose
Fire in a Survival world as the game does it today (`openspec/changes/wildfire`,
moved requirement by requirement as each is built and tested): how dry each
place and each plant is, small forest fires lit by a torch that spread
through what is dry and stop in rain, water and bare ground, the scar they
leave and how it grows back, and torches that the rain puts out. The dry
lightning strike is still the change's.

## Requirements

### Requirement: The weather knows how dry the ground is
The atmosphere SHALL keep the ground's wetness in each of its cells, raised
by rain and dried by sun, warmth and dry air, saved with the weather. A
world saved before it existed SHALL open with the wetness at its spun-up
value.

Pinned by `pbd_core::atmosphere::tests::{soaked_ground_dries_in_about_a_world_day_of_summer_sun, a_save_from_before_the_ground_wetness_restores_at_the_biomes_wetness}`.

#### Scenario: Drying out
- **WHEN** soaked ground has a world day of dry summer sun
- **THEN** its wetness falls far enough that cured grass on it can burn

#### Scenario: An old save
- **WHEN** a world saved before the ground's wetness existed is opened
- **THEN** its weather restores, with the wetness at each cell's starting
  value

### Requirement: Every plant burns by how dry it is, and green plants do not burn
Each cell SHALL have a plant dryness from 0 to 1, derived and never saved,
from the plant's kind, its season in its own hemisphere's year, the
ground's wetness, the air's humidity and warmth, shade under trees, water
nearby, the ground's hollows and rises, the soil's own patches, grazing and
snow. A cactus, a cell under snow and a flooded cell SHALL read 0. A cell
whose dryness is below one half SHALL never catch fire, and above it the
chance of catching SHALL grow steeply with dryness.

Pinned by `pbd_core::fire::tests::{plant_dry_matches_design_2s_table_of_examples, the_hazard_grows_as_the_square_of_the_margin_and_is_nought_when_green, a_cell_below_one_half_never_catches_from_a_neighbour, spring_green_grass_never_catches_anywhere_on_the_patch, a_front_goes_out_at_a_green_river_bank, the_dry_season_is_half_a_year_apart_across_the_equator, snow_water_and_a_cactus_never_burn}` and `pbd_app::wildfire::tests::a_torch_on_spring_grass_makes_a_damp_flame_and_explorer_none`.

#### Scenario: Spring-green grass
- **WHEN** a Survival player uses a lit torch on grass in spring
- **THEN** a small flame dies within a few seconds and nothing is saved

#### Scenario: A river's bank
- **WHEN** a fire reaches the grass beside a river in the dry season
- **THEN** that grass does not catch

#### Scenario: Each hemisphere its own season
- **WHEN** it is late summer in one hemisphere
- **THEN** the other hemisphere's open grass is too green to burn

### Requirement: Fires stay small
A fire SHALL spread only to touching cells, throw no embers, burn out where
its front meets green or wetter ground, and never burn more than its cell
limit or have more than its burning limit alight at once, with no more than
two fires burning in a world. A grass fire's front SHALL move at well under
a metre a second in any wind, and a tree SHALL burn far slower than grass,
in its crown only when it is a very dry conifer in a strong wind.

Pinned by `pbd_core::fire::tests::{a_late_summer_grass_fire_stays_small_and_slow, a_fire_on_open_dry_grass_stops_at_its_limits}`.

#### Scenario: A late-summer grass fire in a breeze
- **WHEN** a fire burns through dry fields in a 3 m/s wind
- **THEN** it burns at most 300 cells and is out within minutes of world
  time

### Requirement: Fire spreads by one rule the server runs
A forest fire SHALL spread cell to cell through grass, shrubs and trees by a
fixed one-second rule, faster downwind and uphill, using each cell's own
five or six neighbours, with every chance drawn from the world seed, the
fire, the cell and the tick. The server SHALL run it, and the same fire SHALL
burn the same cells on every machine and after a reload.

Pinned by `pbd_core::fire::tests::{the_same_blaze_burns_the_same_cells_on_two_runs_and_across_a_reload, a_pentagon_cell_tries_its_five_neighbours}` and `pbd_app::wildfire::tests::a_torch_on_late_summer_grass_starts_a_blaze_and_the_server_spreads_it`.

#### Scenario: A reload mid-fire
- **WHEN** a world is closed while a fire burns and reopened
- **THEN** the fire goes on from the cells it had reached, and burns the same
  cells it would have burned without the reload

### Requirement: Rain, water and bare ground stop fire, and the player's things never burn
Rain SHALL slow a fire and put it out, heavy rain stopping any cell from
catching. Water, sand, rock, snow, soil without grass, paths and roads SHALL
not burn, grazed pasture SHALL be too green to burn, and a bucket of water SHALL put out a burning cell and its neighbours.
Every cell, block and thing a player built, placed, tilled or planted SHALL
stop a fire and SHALL NOT burn.

Pinned by `pbd_core::fire::tests::{rain_stops_a_front, a_front_goes_out_at_a_green_river_bank, a_bucket_puts_out_a_cell_and_its_neighbours, a_dug_line_and_the_players_field_hold, a_burnt_cell_cannot_burn_again_until_its_grass_is_whole}`.

#### Scenario: A dug break
- **WHEN** a grass fire reaches a line the player dug down to soil
- **THEN** it does not cross it in calm air

#### Scenario: A field in the way
- **WHEN** a fire reaches a player's planted field and fence
- **THEN** the field and the fence are untouched

### Requirement: A fire's scar is saved and grows back
Every cell a fire burned SHALL be saved, authored by the fire as a world
process, with who lit it named. Burnt ground SHALL show ash, burnt trees
SHALL stand as charred snags that give charcoal, and the scar SHALL grow back
by world days of play: grass first, then shrubs, then saplings, then full
trees.

Pinned by `pbd_core::fire::tests::{the_scar_grows_back_by_day, a_torch_on_late_summer_grass_starts_a_blaze_and_a_third_is_a_damp_flame}` (the ignition's record and the blaze's author) and `pbd_app::wildfire::tests::{a_torch_on_late_summer_grass_starts_a_blaze_and_the_server_spreads_it, a_snag_chopped_gives_charcoal_and_is_felled}`.

#### Scenario: The day after
- **WHEN** a player returns to a burnt forest the next world day
- **THEN** its ground is ash and its trees stand charred

#### Scenario: Sixty days on
- **WHEN** sixty world days of play have passed since it went out
- **THEN** the forest stands as before

### Requirement: Torches go out in the rain and are relit, in Survival
In a Survival world, a torch held or placed under the open sky in rain SHALL
go out after a short time, and a roof SHALL keep it lit. A placed torch
going out SHALL leave the player's block unchanged and be kept as a record
of the world's. An out torch SHALL be relit from any fire, and a lit torch
SHALL light an out torch, a brazier, a candle and a lantern. In Explorer,
torches SHALL never go out.

Pinned by `pbd_core::fire::tests::{a_torch_goes_out_in_the_rain_under_the_sky_and_not_under_a_roof, a_placed_torch_out_is_the_worlds_record_and_relit_the_players}` and `pbd_app::wildfire::tests::{a_placed_torch_goes_out_in_forced_rain_and_a_roof_keeps_one_lit, an_out_torch_is_drawn_dark_and_lights_nothing}`.

#### Scenario: A torch by the field in a storm
- **WHEN** rain falls on a Survival player's placed torch under the sky
- **THEN** it goes out after about 20 s, stays out across a reload, and lights
  again when the player touches a lit torch to it
