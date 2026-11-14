## MODIFIED Requirements

### Requirement: Some flowers glow at night, and the CPU decides which
A share of flowers SHALL be a bioluminescent species that is a dusk-lit
emitter. Which cells carry one SHALL be decided once on the CPU and carried
in the cell record, never rolled independently by the shader. The share SHALL
be one cell in a hundred, and the flower SHALL be faint: its head drawn at a
quarter of its former brightness and its light on the field at level 3. A
smaller share SHALL keep a subset of the cells a larger one chose, so lowering
it thins the glowers rather than moving them.

#### Scenario: A meadow glows at night
- **WHEN** night falls over a meadow carrying the glowing species
- **THEN** the ground near those cells is lit, and by day it is not
  (`planet::column::tests::a_glowing_flower_lights_its_sod_at_night_and_is_put_out_by_digging_it`,
  `flora::tests::the_share_that_glows_is_the_chance`)

#### Scenario: The shader and the bake agree
- **WHEN** the shader draws a glowing flower head
- **THEN** it is on a cell the bake treated as an emitter
  (`planet::column::tests::a_tiers_glowing_flowers_agree_between_record_bake_and_bit`,
  `planet::terrain::tests::the_shader_carries_the_reference_light_constants`)

#### Scenario: A quarter of the glowers stay
- **WHEN** the share drops from 0.04 to 0.01
- **THEN** every cell that glows at 0.01 glowed at 0.04, and about a quarter
  of those do (`flora::tests::a_smaller_share_keeps_a_subset_of_the_glowers`)
