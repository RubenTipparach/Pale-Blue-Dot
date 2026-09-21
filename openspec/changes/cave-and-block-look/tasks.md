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

## 5. A face wears its voxel
- [x] `gpu_materials`: every layer's render code, eight to a word, beside the
      light; a test packs and reads back a column with a placed stone in it.
- [x] `material_at` in the shader; kind-4 faces take the layer's own code
      with the reference's side rule.
- [x] `--place N` stacks N stones; a tower captured.
- [x] The surface and visibility shaders parse and validate under naga in
      the suite (`shader_tests.rs`): the first cut of `material_at` shadowed
      the fragment's `altitude`, which WGSL forbids, and the pipeline cache
      logged it while every test stayed green and the planet drew nothing.

## 6. Held
- [ ] The dithered face: the `--pitch -8` captures over a dug floor and
      beside placed stone show nothing on the software rasteriser
      (`placed-stone-grazing.png`); nearest-filtered mips stay proposed,
      pending the owner's in-game look.
- [ ] The owner's in-game look at all of it.
