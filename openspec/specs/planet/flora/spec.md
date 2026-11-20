# Flora Specification

## Purpose
What grows on the ground and how it is decided: plants are decoration drawn
from the cell record at draw time. The rest of flora v2 is designed in
`openspec/changes/flora-v2` and moves here as it is built.

## Requirements

### Requirement: Nothing grows where water stands

A cell whose top has water standing on it SHALL be drawn as a bed: a grass
of any kind or snow as dirt, sand as the seabed's sand. No tuft, flower,
fern, mushroom, bush or tree SHALL be drawn on it or be workable there. The
rule SHALL be decided at draw time from the column (and the generator's
river beyond the column tier), SHALL NOT change any cell's material or any
save, and SHALL need no new generator version.

#### Scenario: A river's bed

- **WHEN** the player looks down at a river's bed in any world, old or new
- **THEN** the bed is dirt, with no grass, flowers, mushrooms or trees on it

### Requirement: A glowing mushroom can be picked and stays picked

A player SHALL pick a glowing mushroom (the cell chosen by
`pbd_core::flora::glows` on its exact key) with F while the eye ray passes
within `glow_pick_radius_m` of the plant, as the plant is drawn, within reach.
A pick SHALL give one `glowing-mushroom` good and write a `glow-plant` record
keyed by the cell's exact key, in one durable write with the pack. The pick
SHALL be acknowledged only after the save accepts it. A picked cell SHALL draw
no plant in its flower slot and SHALL give no light until it grows back.

#### Scenario: A pick survives a reload

- **WHEN** a player picks a glowing mushroom, saves and opens the world again
- **THEN** that cell draws no plant and adds no emitter to the light field
  (`planet::column::tests::a_picked_glowing_mushroom_is_neither_drawn_nor_lit`)

#### Scenario: Aim at the plant, not the cell

- **WHEN** the eye ray meets the cell's ground a metre from the plant
- **THEN** F offers nothing, and it offers `Pick` once the ray passes the
  plant's stalk or cap
  (`desktop::glow_plants::tests::the_ray_meets_the_plant_not_its_ground`,
  `flora::tests::the_mushroom_to_pick_is_where_the_draw_stands_it`)

#### Scenario: A picked cell grows no other flower in its place

- **WHEN** a picked cell's own flower roll would have grown an ordinary flower
- **THEN** its flower slot draws nothing while the pick lasts
  (`planet::column::tests::a_picked_glowing_mushroom_is_neither_drawn_nor_lit`,
  `planet::terrain::tests::the_shader_carries_the_reference_light_constants`)

### Requirement: A picked glowing mushroom grows back out of sight

A `glow-plant` record older than `glow_plant_regrow_days` world days SHALL
read as no pick, but only when the tier builds the cell while it lies farther
than `clutter_radius_m` from the camera. A cell that comes due within that
distance SHALL stay picked until a later build finds it farther.

#### Scenario: No plant appears in front of the player

- **WHEN** a pick passes its regrowth time while the player stands 20 m from
  the cell
- **THEN** the cell draws no plant until it has been built farther than
  `clutter_radius_m` away
  (`edits::tests::a_pick_grows_back_only_out_of_sight`,
  `planet::column::tests::a_picked_glowing_mushroom_is_neither_drawn_nor_lit`)

### Requirement: Placing glowing mushrooms is not the generator's, and a world never picked from stores nothing

Which cells grow a glowing mushroom SHALL depend only on the cell's exact key
and `glow_flower_chance`. Picking SHALL add no generator version. A world with
no pick SHALL save nothing it did not save before. `glow_flower_chance` and
the hash SHALL be pinned by a digest over sample keys.

#### Scenario: An old save opens unchanged

- **WHEN** a world saved before this change is opened
- **THEN** its generator version, ground and tree digests, and records are
  unchanged, and the same cells grow glowing mushrooms as before
  (`flora::tests::which_cells_glow_is_pinned`,
  `config::tests::the_glowing_mushroom_chance_is_pinned`)
