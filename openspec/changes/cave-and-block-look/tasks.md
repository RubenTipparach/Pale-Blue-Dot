# Tasks

## 1. The audit
- [x] `tools/block_audit.py`: every code, every sheet, the cap tile and the
      side tile off the real atlas, labelled from `biomes.json`.
- [x] Snow to the tundra sheet's wind packed snow; sand to each sheet's own
      ground.
- [x] `the_shader_draws_each_material_on_a_tile_named_for_it`.

## 2. The cave
- [x] Haze and rim multiplied by `skylight`.
- [x] A cave capture before and after.

## 3. The lens
- [x] `drop_uv` in the reference's y-up frame; the refraction's y negated.

## 4. The wall's foot and head
- [x] `light::CONTACT` with a fourth rung and `light::wall_corner`, tested.
- [x] `wall_light` transcribes it; the constants test pins `CONTACT_3`.
- [x] The pit capture probed: the foot darker than the middle.

## 5. Held
- [ ] The dithered face: a `--pitch -8` capture over a dug floor, then
      nearest-filtered mips if it shows.
- [ ] The owner's in-game look at all of it.
